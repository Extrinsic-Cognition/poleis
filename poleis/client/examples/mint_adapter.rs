//! Bounded host JSON adapter. No RPC, file/key reads, signing or submission.
//! Supplied accounts are observations, not chain proofs: the host must verify
//! mainnet genesis, deployed code/authority and a coherent RPC context before
//! passing snapshots here. See mint_adapter.md for the exact protocol.

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use poleis_client::{
    self as client, nft,
    nft_receipt::{self, OwnershipObservation, ReceiptExpectation},
    Config, ConfigView, FeeTally, Fund, FundView, Registry, RegistryView, TallyView, Treasury,
    TreasuryView, PROGRAM_ID,
};
use poleis_treasury::{nft_seat_state, nft_wire, seat_nft::RawAccount};
use serde::Deserialize;
use serde_json::{json, Value};
use solana_hash::Hash;
use solana_message::legacy::Message;
use solana_pubkey::Pubkey;
use std::{
    io::{Read, Write},
    str::FromStr,
};

const GENESIS: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";
const MAX_INPUT: usize = 256 * 1024;
const MAX_OUTPUT: usize = 64 * 1024;
const MAX_ACCOUNT: usize = 16 * 1024;
type Result<T> = std::result::Result<T, &'static str>;

#[derive(Deserialize)]
#[serde(
    tag = "method",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Request {
    Derive {
        unit: String,
        buyer: String,
        completion_digest: String,
    },
    InspectSale {
        unit: String,
        buyer: String,
        completion_digest: String,
        metadata: Metadata,
        current_slot: String,
        max_price_lamports: String,
        expiry_slot: String,
        accounts: Vec<Snapshot>,
    },
    BuildMint {
        unit: String,
        buyer: String,
        completion_digest: String,
        metadata: Metadata,
        current_slot: String,
        max_price_lamports: String,
        expiry_slot: String,
        curve: Snapshot,
        issuer_signature_base64: String,
        recent_blockhash: String,
    },
    InspectReceipt {
        unit: String,
        buyer: String,
        completion_digest: String,
        metadata: Metadata,
        max_price_lamports: String,
        receipt: Snapshot,
        mint: Snapshot,
        token: Option<Snapshot>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    name: String,
    symbol: String,
    uri: String,
}
impl Metadata {
    fn wire(&self) -> Result<nft::Metadata<'_>> {
        let m = nft::Metadata {
            name: &self.name,
            symbol: &self.symbol,
            uri: &self.uri,
        };
        m.validate().map_err(|_| "invalid-metadata")?;
        if [&self.name, &self.symbol, &self.uri]
            .iter()
            .any(|s| s.chars().any(char::is_control))
        {
            return Err("invalid-metadata");
        }
        Ok(m)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    address: String,
    // A missing RPC member is not an observation of absence.
    #[serde(deserialize_with = "required_account")]
    account: Option<Account>,
}
fn required_account<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<Account>, D::Error> {
    Option::<Account>::deserialize(d)
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Account {
    owner: String,
    lamports: String,
    data_base64: String,
    executable: bool,
}
struct Decoded {
    address: Pubkey,
    owner: Pubkey,
    lamports: u64,
    data: Vec<u8>,
    exists: bool,
}
impl Snapshot {
    fn decode(self) -> Result<Decoded> {
        let address = key(&self.address)?;
        match self.account {
            None => Ok(Decoded {
                address,
                owner: Pubkey::default(),
                lamports: 0,
                data: vec![],
                exists: false,
            }),
            Some(a) => {
                if a.executable {
                    return Err("unexpected-executable");
                }
                Ok(Decoded {
                    address,
                    owner: key(&a.owner)?,
                    lamports: number(&a.lamports)?,
                    data: bytes(&a.data_base64, MAX_ACCOUNT)?,
                    exists: true,
                })
            }
        }
    }
}
impl Decoded {
    fn raw(&self) -> RawAccount<'_> {
        RawAccount {
            address: self.address.as_array(),
            program_owner: self.owner.as_array(),
            data: &self.data,
        }
    }
    fn fresh(&self) -> bool {
        self.owner == Pubkey::default() && self.data.is_empty()
    }
    fn program(&self) -> Result<()> {
        if !self.exists || self.owner != PROGRAM_ID {
            return Err("wrong-account-owner");
        }
        Ok(())
    }
}
fn number(s: &str) -> Result<u64> {
    if s.is_empty()
        || s.len() > 20
        || (s.len() > 1 && s.starts_with('0'))
        || !s.bytes().all(|b| b.is_ascii_digit())
    {
        return Err("invalid-u64");
    }
    s.parse().map_err(|_| "invalid-u64")
}
fn key(s: &str) -> Result<Pubkey> {
    if s.len() > 44 {
        return Err("invalid-address");
    }
    let p = Pubkey::from_str(s).map_err(|_| "invalid-address")?;
    if p.to_string() != s {
        return Err("invalid-address");
    }
    Ok(p)
}
fn digest(s: &str) -> Result<[u8; 32]> {
    if s.len() != 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid-completion");
    }
    let mut value = [0; 32];
    for (i, v) in value.iter_mut().enumerate() {
        *v = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| "invalid-completion")?;
    }
    if value == [0; 32] {
        return Err("invalid-completion");
    }
    Ok(value)
}
fn bytes(s: &str, max: usize) -> Result<Vec<u8>> {
    if s.len() > max.div_ceil(3) * 4 {
        return Err("data-too-large");
    }
    let value = B64.decode(s).map_err(|_| "invalid-base64")?;
    if value.len() > max || B64.encode(&value) != s {
        return Err("invalid-base64");
    }
    Ok(value)
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

struct Identity {
    unit: u64,
    buyer: Pubkey,
    completion: [u8; 32],
    accounts: [Pubkey; 14],
}
impl Identity {
    fn parse(unit: &str, buyer: &str, completion: &str) -> Result<Self> {
        let unit = number(unit)?;
        let buyer = key(buyer)?;
        if buyer == Pubkey::default() {
            return Err("invalid-buyer");
        }
        let completion = digest(completion)?;
        let mint = nft::mint_pda(unit, &completion);
        let accounts = [
            buyer,
            client::config_pda_of(unit),
            nft::curve_pda(unit),
            client::registry_pda(unit),
            client::treasury_pda(unit),
            nft::seat_pda(&mint),
            mint,
            nft::initial_token_pda(&mint, &buyer),
            nft::validation_pda(&mint),
            client::fund_pda(),
            client::tally_pda(unit),
            client::INSTRUCTIONS_SYSVAR_ID,
            client::TOKEN_2022_PROGRAM_ID,
            Pubkey::default(),
        ];
        if accounts
            .iter()
            .enumerate()
            .any(|(i, k)| accounts[..i].contains(k))
        {
            return Err("account-alias");
        }
        Ok(Self {
            unit,
            buyer,
            completion,
            accounts,
        })
    }
    fn derived(&self) -> Value {
        json!({ "program": PROGRAM_ID.to_string(), "genesisHash": GENESIS,
            "programData": client::programdata_pda().to_string(),
            "accounts": self.accounts.map(|k| k.to_string()),
            "readAddresses": self.accounts[..11].iter().map(ToString::to_string).collect::<Vec<_>>(),
            "curve": self.accounts[2].to_string(), "mint": self.accounts[6].to_string(),
            "seat": self.accounts[5].to_string(), "token": self.accounts[7].to_string(),
            "validation": self.accounts[8].to_string() })
    }
    fn curve(&self, a: &Decoded) -> Result<nft::MintCurve> {
        a.program()?;
        if a.address != self.accounts[2] {
            return Err("wrong-curve-address");
        }
        let curve = nft::MintCurve::decode(&a.data).map_err(|_| "invalid-curve")?;
        let (_, bump) = Pubkey::find_program_address(
            &[nft_seat_state::CURVE_SEED, &self.unit.to_le_bytes()],
            &PROGRAM_ID,
        );
        if curve.bump != bump
            || curve.unit != self.unit
            || curve.genesis_hash != key(GENESIS)?.to_bytes()
        {
            return Err("wrong-curve-identity");
        }
        Ok(curve)
    }
}
fn bump(a: &Decoded, offset: usize, seeds: &[&[u8]]) -> Result<()> {
    let (address, expected) = Pubkey::find_program_address(seeds, &PROGRAM_ID);
    if a.address != address || a.data.get(offset) != Some(&expected) {
        return Err("invalid-account-pda");
    }
    Ok(())
}
fn permit(
    id: &Identity,
    curve: &nft::MintCurve,
    metadata: nft::Metadata<'_>,
    now: u64,
    max: u64,
    expiry: u64,
) -> Result<[u8; nft::PERMIT_LEN]> {
    curve.quote(now, max).map_err(|_| "sale-unavailable")?;
    if expiry < now || expiry >= curve.closes_slot {
        return Err("invalid-expiry");
    }
    nft::permit_message(curve, id.buyer, id.completion, metadata, max, expiry)
        .map_err(|_| "invalid-permit")
}

#[allow(clippy::too_many_arguments)]
fn inspect_sale(
    id: Identity,
    metadata: Metadata,
    current: &str,
    max: &str,
    expiry: &str,
    snapshots: Vec<Snapshot>,
) -> Result<Value> {
    if snapshots.len() != 11 {
        return Err("invalid-account-set");
    }
    let accounts = snapshots
        .into_iter()
        .map(Snapshot::decode)
        .collect::<Result<Vec<_>>>()?;
    if accounts
        .iter()
        .zip(&id.accounts)
        .any(|(a, k)| a.address != *k)
    {
        return Err("invalid-account-set");
    }
    let a = &accounts;
    if !a[0].fresh() {
        return Err("invalid-buyer-account");
    }
    for account in [&a[1], &a[2], &a[3], &a[4]] {
        account.program()?;
    }
    let config = ConfigView::parse(&a[1].data).ok_or("invalid-config")?;
    if config.root != id.unit
        || ![client::MODE_GENESIS, client::MODE_COUNCIL].contains(&config.mode)
    {
        return Err("invalid-sale-mode");
    }
    bump(
        &a[1],
        Config::OFF_BUMP,
        &[b"config", &id.unit.to_le_bytes()],
    )?;
    let curve = id.curve(&a[2])?;
    // The older RegistryView assumes its count fits its allocated rows. Prove
    // that before entering it, so arbitrary RPC bytes cannot panic this host.
    let data = &a[3].data;
    if !(Registry::MIN_SIZE..=Registry::MAX_SIZE).contains(&data.len())
        || (data.len() - Registry::MIN_SIZE) % Registry::HOLDER_LEN != 0
        || data[Registry::OFF_HOLDER_COUNT] as usize
            > (data.len() - Registry::MIN_SIZE) / Registry::HOLDER_LEN
    {
        return Err("invalid-registry");
    }
    let registry = RegistryView::parse(data).ok_or("invalid-registry")?;
    bump(
        &a[3],
        Registry::OFF_BUMP,
        &[b"registry", &id.unit.to_le_bytes()],
    )?;
    if registry.unit != id.unit
        || registry.root != id.unit
        || registry.frozen
        || registry.holders.len() >= client::MAX_HOLDERS
        || registry.rung as usize >= client::LADDER.len()
        || registry
            .total_seats
            .checked_add(1)
            .ok_or("arithmetic-overflow")?
            > client::ladder_hexagons(registry.rung as usize)
    {
        return Err("registry-unavailable");
    }
    let treasury = TreasuryView::parse(&a[4].data).ok_or("invalid-treasury")?;
    if treasury.unit != id.unit {
        return Err("wrong-treasury-unit");
    }
    bump(
        &a[4],
        Treasury::OFF_BUMP,
        &[b"treasury", &id.unit.to_le_bytes()],
    )?;
    if a[5..9].iter().any(|account| !account.fresh()) {
        return Err("issuance-already-present");
    }
    let now = number(current)?;
    let max = number(max)?;
    let metadata = metadata.wire()?;
    let permit = permit(&id, &curve, metadata, now, max, number(expiry)?)?;
    let quote = curve.quote(now, max).map_err(|_| "sale-unavailable")?;
    let mut fee_bps = 0u16;
    if !a[9].fresh() {
        a[9].program()?;
        let fund = FundView::parse(&a[9].data).ok_or("invalid-fund")?;
        if a[9].data[Fund::OFF_BUMP] != client::FUND_BUMP || fund.fee_bps as u64 > client::BPS {
            return Err("invalid-fund");
        }
        fee_bps = fund.fee_bps;
    }
    let fee =
        |amount: u64| -> u64 { (amount as u128 * fee_bps as u128 / client::BPS as u128) as u64 };
    let mut allocations = Vec::new();
    let mut allocate = |account: &Decoded, size: usize| {
        allocations.push(
            json!({ "address": account.address.to_string(), "size": size,
            "currentLamports": account.lamports.to_string() }),
        );
    };
    allocate(&a[5], nft::NftSeat::SIZE);
    allocate(
        &a[6],
        nft_wire::metadata_mint_len(metadata).map_err(|_| "invalid-metadata")?,
    );
    allocate(&a[7], nft_wire::TOKEN_ACCOUNT_LEN);
    allocate(&a[8], nft_wire::EXTRA_ACCOUNT_METAS_LEN);
    let next_registry_size = Registry::size_for(registry.holders.len() + 1);
    if next_registry_size > a[3].data.len() {
        allocate(&a[3], next_registry_size);
    }
    // Rent budgeting covers any price within the approved cap: a tiny current
    // quote can round the fund fee to zero while a later allowed price does not.
    if fee(max) > 0 {
        if a[10].fresh() {
            allocate(&a[10], FeeTally::SIZE);
        } else {
            a[10].program()?;
            let tally = TallyView::parse(&a[10].data).ok_or("invalid-tally")?;
            if tally.root != id.unit {
                return Err("wrong-tally-root");
            }
            bump(
                &a[10],
                FeeTally::OFF_BUMP,
                &[b"fees", &id.unit.to_le_bytes()],
            )?;
        }
    }
    let mut result = id.derived();
    result["sale"] = json!({ "unit": curve.unit.to_string(), "startLamports": curve.start_lamports.to_string(),
        "stepLamports": curve.step_lamports.to_string(), "mintedCount": curve.minted_count.to_string(),
        "capacity": curve.capacity.to_string(), "opensSlot": curve.opens_slot.to_string(),
        "closesSlot": curve.closes_slot.to_string(), "completionIssuer": Pubkey::new_from_array(curve.completion_issuer).to_string(),
        "genesisHash": GENESIS });
    result["config"] = json!({ "root": config.root.to_string(), "authority": config.authority.to_string(), "mode": config.mode });
    result["quote"] = json!({ "priceLamports": quote.price_lamports.to_string(), "fundFeeLamports": fee(quote.price_lamports).to_string() });
    result["allocations"] = json!(allocations);
    result["issuer"] = json!(Pubkey::new_from_array(curve.completion_issuer).to_string());
    result["permitBase64"] = json!(B64.encode(permit));
    result["buyerLamports"] = json!(a[0].lamports.to_string());
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn build_mint(
    id: Identity,
    metadata: Metadata,
    current: &str,
    max: &str,
    expiry: &str,
    snapshot: Snapshot,
    signature: &str,
    blockhash: &str,
) -> Result<Value> {
    let curve = id.curve(&snapshot.decode()?)?;
    let metadata = metadata.wire()?;
    let max = number(max)?;
    let permit = permit(
        &id,
        &curve,
        metadata,
        number(current)?,
        max,
        number(expiry)?,
    )?;
    let signature: [u8; 64] = bytes(signature, 64)?
        .try_into()
        .map_err(|_| "invalid-issuer-signature")?;
    // Verify the issuer, not only its signature's length. The runtime and Node
    // proof verifier independently repeat this before any wallet submission.
    let issuer = ed25519_dalek::VerifyingKey::from_bytes(&curve.completion_issuer)
        .map_err(|_| "invalid-issuer")?;
    issuer
        .verify_strict(&permit, &ed25519_dalek::Signature::from_bytes(&signature))
        .map_err(|_| "invalid-issuer-signature")?;
    let instructions = nft::mint_instructions(&curve, &permit, signature, metadata, max)
        .map_err(|_| "invalid-mint-instructions")?;
    let actual: Vec<_> = instructions[1].accounts.iter().map(|m| m.pubkey).collect();
    if actual != id.accounts {
        return Err("account-set-mismatch");
    }
    let blockhash = key(blockhash)?;
    if blockhash == Pubkey::default() {
        return Err("invalid-blockhash");
    }
    // Official compiler/serializer, rather than a second implementation of
    // account ordering, shortvec framing or the legacy message wire format.
    let message = Message::new_with_blockhash(
        &instructions,
        Some(&id.buyer),
        &Hash::new_from_array(blockhash.to_bytes()),
    );
    if message.header.num_required_signatures != 1
        || message.header.num_readonly_signed_accounts != 0
        || message.header.num_readonly_unsigned_accounts != 6
        || message.account_keys.len() != 16
    {
        return Err("unsupported-message-shape");
    }
    let message = message.serialize();
    let mut transaction = vec![0; 65];
    transaction[0] = 1; // one canonical compact signature count and one zero signature
    transaction.extend_from_slice(&message);
    if transaction.len() > nft::MAX_TRANSACTION_BYTES
        || transaction.len()
            != nft::legacy_transaction_size(id.buyer, &instructions)
                .map_err(|_| "invalid-message")?
    {
        return Err("transaction-too-large");
    }
    let mut result = id.derived();
    result["transactionBase64"] = json!(B64.encode(transaction));
    result["messageBase64"] = json!(B64.encode(message));
    result["permitBase64"] = json!(B64.encode(permit));
    result["metadataDigest"] = json!(hex(
        &nft::metadata_digest(metadata).map_err(|_| "invalid-metadata")?
    ));
    Ok(result)
}

fn inspect_receipt(
    id: Identity,
    metadata: Metadata,
    max: &str,
    receipt: Snapshot,
    mint: Snapshot,
    token: Option<Snapshot>,
) -> Result<Value> {
    let receipt = receipt.decode()?;
    let mint = mint.decode()?;
    if receipt.address != id.accounts[5] || mint.address != id.accounts[6] {
        return Err("wrong-receipt-address");
    }
    let expected = ReceiptExpectation {
        unit: id.unit,
        completion: id.completion,
        first_buyer: id.buyer,
        max_price_lamports: number(max)?,
        metadata: metadata.wire()?,
    };
    if expected.max_price_lamports == 0 {
        return Err("invalid-max-price");
    }
    if receipt.fresh() && mint.fresh() {
        return Ok(json!({ "state": "absent" }));
    }
    let token = token.map(Snapshot::decode).transpose()?;
    let result = nft_receipt::validate_receipt(
        receipt.raw(),
        mint.raw(),
        token.as_ref().map(Decoded::raw),
        &expected,
    )
    .map_err(|_| "invalid-receipt")?;
    let (owner, burned, observation) = match result.ownership {
        OwnershipObservation::NotObserved => (None, false, "not-observed"),
        OwnershipObservation::HeldBy(owner) => (Some(owner.to_string()), false, "held"),
        OwnershipObservation::Burned => (None, true, "burned"),
    };
    Ok(
        json!({ "state": "issued", "mint": result.mint.to_string(), "receipt": result.seat.to_string(),
        "originalBuyer": Pubkey::new_from_array(result.receipt.first_buyer).to_string(),
        "owner": owner, "burned": burned, "ownership": observation,
        "paidLamports": result.receipt.paid_lamports.to_string(), "serial": result.receipt.serial.to_string() }),
    )
}

fn dispatch(request: Request) -> Result<Value> {
    match request {
        Request::Derive {
            unit,
            buyer,
            completion_digest,
        } => Ok(Identity::parse(&unit, &buyer, &completion_digest)?.derived()),
        Request::InspectSale {
            unit,
            buyer,
            completion_digest,
            metadata,
            current_slot,
            max_price_lamports,
            expiry_slot,
            accounts,
        } => inspect_sale(
            Identity::parse(&unit, &buyer, &completion_digest)?,
            metadata,
            &current_slot,
            &max_price_lamports,
            &expiry_slot,
            accounts,
        ),
        Request::BuildMint {
            unit,
            buyer,
            completion_digest,
            metadata,
            current_slot,
            max_price_lamports,
            expiry_slot,
            curve,
            issuer_signature_base64,
            recent_blockhash,
        } => build_mint(
            Identity::parse(&unit, &buyer, &completion_digest)?,
            metadata,
            &current_slot,
            &max_price_lamports,
            &expiry_slot,
            curve,
            &issuer_signature_base64,
            &recent_blockhash,
        ),
        Request::InspectReceipt {
            unit,
            buyer,
            completion_digest,
            metadata,
            max_price_lamports,
            receipt,
            mint,
            token,
        } => inspect_receipt(
            Identity::parse(&unit, &buyer, &completion_digest)?,
            metadata,
            &max_price_lamports,
            receipt,
            mint,
            token,
        ),
    }
}
fn execute(input: &[u8]) -> Value {
    let result = if input.len() > MAX_INPUT {
        Err("input-too-large")
    } else {
        serde_json::from_slice::<Request>(input)
            .map_err(|_| "invalid-request")
            .and_then(dispatch)
    };
    match result {
        Ok(result) => json!({ "ok": true, "result": result }),
        Err(error) => json!({ "ok": false, "error": error }),
    }
}
fn main() {
    let mut input = Vec::new();
    let response = match std::io::stdin()
        .lock()
        .take((MAX_INPUT + 1) as u64)
        .read_to_end(&mut input)
    {
        Ok(_) => execute(&input),
        Err(_) => json!({ "ok": false, "error": "input-unavailable" }),
    };
    let mut output = serde_json::to_vec(&response)
        .unwrap_or_else(|_| br#"{"ok":false,"error":"encoding-failed"}"#.to_vec());
    if output.len() > MAX_OUTPUT {
        output = br#"{"ok":false,"error":"output-too-large"}"#.to_vec();
    }
    output.push(b'\n');
    let _ = std::io::stdout().lock().write_all(&output);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn id() -> Identity {
        Identity::parse(
            "0",
            &Pubkey::new_from_array([7; 32]).to_string(),
            &"11".repeat(32),
        )
        .unwrap()
    }
    fn metadata() -> Value {
        json!({"name":"Test Seat","symbol":"TEST","uri":"ipfs://fixture"})
    }
    fn curve() -> nft::MintCurve {
        nft::MintCurve {
            bump: Pubkey::find_program_address(
                &[nft_seat_state::CURVE_SEED, &0u64.to_le_bytes()],
                &PROGRAM_ID,
            )
            .1,
            unit: 0,
            start_lamports: 100,
            step_lamports: 10,
            minted_count: 0,
            capacity: 20,
            opens_slot: 100,
            closes_slot: 400,
            completion_issuer: SigningKey::from_bytes(&[71; 32]).verifying_key().to_bytes(),
            genesis_hash: key(GENESIS).unwrap().to_bytes(),
        }
    }
    fn snapshot(address: Pubkey, owner: Pubkey, data: Vec<u8>) -> Value {
        json!({"address":address.to_string(),"account":{"owner":owner.to_string(),
            "lamports":"1000","dataBase64":B64.encode(data),"executable":false}})
    }
    fn empty(address: Pubkey) -> Value {
        json!({"address":address.to_string(),"account":null})
    }
    fn intent(method: &str) -> Value {
        json!({"method":method,"unit":"0","buyer":id().buyer.to_string(),
            "completionDigest":"11".repeat(32),"metadata":metadata(),
            "currentSlot":"100","maxPriceLamports":"100","expirySlot":"150"})
    }
    fn run(request: Value) -> Value {
        execute(&serde_json::to_vec(&request).unwrap())
    }
    fn sale() -> Value {
        let id = id();
        let mut accounts: Vec<_> = id.accounts[..11].iter().map(|k| empty(*k)).collect();
        let mut config = vec![0; Config::SIZE];
        config[0] = client::DISC_CONFIG;
        config[1] = client::LAYOUT_VERSION;
        config[Config::OFF_AUTHORITY..Config::OFF_AUTHORITY + 32]
            .copy_from_slice(id.buyer.as_ref());
        config[Config::OFF_MODE] = client::MODE_GENESIS;
        config[Config::OFF_BUMP] =
            Pubkey::find_program_address(&[b"config", &0u64.to_le_bytes()], &PROGRAM_ID).1;
        accounts[1] = snapshot(id.accounts[1], PROGRAM_ID, config);
        accounts[2] = snapshot(
            id.accounts[2],
            PROGRAM_ID,
            curve().encode().unwrap().to_vec(),
        );
        let mut registry = vec![0; Registry::MIN_SIZE];
        registry[0] = client::DISC_REGISTRY;
        registry[1] = client::LAYOUT_VERSION;
        registry[Registry::OFF_BUMP] =
            Pubkey::find_program_address(&[b"registry", &0u64.to_le_bytes()], &PROGRAM_ID).1;
        registry[Registry::OFF_RUNG] = 1;
        accounts[3] = snapshot(id.accounts[3], PROGRAM_ID, registry);
        let mut treasury = vec![0; Treasury::SIZE];
        treasury[0] = client::DISC_TREASURY;
        treasury[1] = client::LAYOUT_VERSION;
        treasury[Treasury::OFF_BUMP] =
            Pubkey::find_program_address(&[b"treasury", &0u64.to_le_bytes()], &PROGRAM_ID).1;
        accounts[4] = snapshot(id.accounts[4], PROGRAM_ID, treasury);
        let mut request = intent("inspectSale");
        request["accounts"] = json!(accounts);
        request
    }
    fn build() -> Value {
        let mut request = intent("buildMint");
        let id = id();
        let meta: Metadata = serde_json::from_value(metadata()).unwrap();
        let permit = permit(&id, &curve(), meta.wire().unwrap(), 100, 100, 150).unwrap();
        let signature = SigningKey::from_bytes(&[71; 32]).sign(&permit).to_bytes();
        request["curve"] = snapshot(
            id.accounts[2],
            PROGRAM_ID,
            curve().encode().unwrap().to_vec(),
        );
        request["issuerSignatureBase64"] = json!(B64.encode(signature));
        request["recentBlockhash"] = json!(Pubkey::new_from_array([9; 32]).to_string());
        request
    }

    #[test]
    fn input_is_typed_bounded_and_sanitized() {
        let valid = json!({"method":"derive","unit":"0","buyer":id().buyer.to_string(),"completionDigest":"11".repeat(32)});
        assert_eq!(run(valid.clone())["ok"], true);
        for change in [json!(0), json!("00"), json!("18446744073709551616")] {
            let mut request = valid.clone();
            request["unit"] = change;
            assert_eq!(run(request)["ok"], false);
        }
        let mut request = valid.clone();
        request["privateKey"] = json!("never-echo-this");
        let result = run(request);
        assert_eq!(result, json!({"ok":false,"error":"invalid-request"}));
        assert!(!result.to_string().contains("never-echo-this"));
        assert_eq!(
            execute(&vec![b' '; MAX_INPUT + 1])["error"],
            "input-too-large"
        );
        assert_eq!(execute(b"{} {} ")["error"], "invalid-request");
    }
    #[test]
    fn sale_authenticates_snapshots_quotes_and_prepares_shared_permit() {
        let result = run(sale());
        assert_eq!(result["ok"], true, "{result}");
        let r = &result["result"];
        assert_eq!(r["quote"]["priceLamports"], "100");
        assert_eq!(r["quote"]["fundFeeLamports"], "0");
        assert_eq!(r["allocations"].as_array().unwrap().len(), 5);
        let permit = bytes(r["permitBase64"].as_str().unwrap(), nft::PERMIT_LEN).unwrap();
        let fields = poleis_treasury::completion_permit::parse(&permit).unwrap();
        assert_eq!(fields.buyer, id().buyer.to_bytes());
        assert_eq!(fields.max_price_lamports, 100);
        assert_eq!(fields.chain_genesis_hash, key(GENESIS).unwrap().to_bytes());
        assert_eq!(r["programData"], client::programdata_pda().to_string());
    }
    #[test]
    fn malformed_or_stale_sale_accounts_fail_before_host_views() {
        for variant in 0..7 {
            let mut request = sale();
            match variant {
                0 => {
                    request["accounts"][1]["account"]["owner"] =
                        json!(Pubkey::default().to_string())
                }
                1 => request["accounts"][2]["address"] = json!(id().accounts[3].to_string()),
                2 => request["accounts"][3]["account"]["dataBase64"] = json!(B64.encode([0; 2])),
                3 => {
                    let mut data = bytes(
                        request["accounts"][3]["account"]["dataBase64"]
                            .as_str()
                            .unwrap(),
                        MAX_ACCOUNT,
                    )
                    .unwrap();
                    data[Registry::OFF_HOLDER_COUNT] = 255;
                    request["accounts"][3]["account"]["dataBase64"] = json!(B64.encode(data));
                }
                4 => request["currentSlot"] = json!("400"),
                5 => request["maxPriceLamports"] = json!("99"),
                _ => request["accounts"][4]["account"]["executable"] = json!(true),
            }
            assert_eq!(run(request)["ok"], false, "variant {variant}");
        }
    }
    #[test]
    fn build_uses_official_message_and_zero_wallet_signature() {
        let result = run(build());
        assert_eq!(result["ok"], true, "{result}");
        let r = &result["result"];
        let transaction = bytes(r["transactionBase64"].as_str().unwrap(), 1232).unwrap();
        let message = bytes(r["messageBase64"].as_str().unwrap(), 1232).unwrap();
        assert_eq!(transaction[0], 1);
        assert_eq!(&transaction[1..65], &[0; 64]);
        assert_eq!(&transaction[65..], message);
        assert_eq!(&message[..3], &[1, 0, 6]);
        assert_eq!(r["accounts"].as_array().unwrap().len(), 14);
    }
    #[test]
    fn build_rejects_wrong_issuer_domain_and_mutated_intent() {
        for variant in 0..6 {
            let mut request = build();
            match variant {
                0 => request["issuerSignatureBase64"] = json!(B64.encode([1; 64])),
                1 => request["metadata"]["name"] = json!("Changed"),
                2 => request["maxPriceLamports"] = json!("101"),
                3 => request["recentBlockhash"] = json!(Pubkey::default().to_string()),
                4 => {
                    let mut wrong = curve();
                    wrong.genesis_hash = [4; 32];
                    request["curve"]["account"]["dataBase64"] =
                        json!(B64.encode(wrong.encode().unwrap()));
                }
                _ => request["curve"]["account"]["owner"] = json!(Pubkey::default().to_string()),
            }
            assert_eq!(run(request)["ok"], false, "variant {variant}");
        }
    }
    #[test]
    fn absent_receipt_requires_both_canonical_fresh_accounts() {
        let id = id();
        let request = json!({"method":"inspectReceipt","unit":"0","buyer":id.buyer.to_string(),
            "completionDigest":"11".repeat(32),"metadata":metadata(),"maxPriceLamports":"100",
            "receipt":empty(id.accounts[5]),"mint":empty(id.accounts[6])});
        assert_eq!(run(request.clone())["result"]["state"], "absent");
        let mut changed = request.clone();
        changed["receipt"]
            .as_object_mut()
            .unwrap()
            .remove("account");
        assert_eq!(run(changed)["error"], "invalid-request");
        let mut changed = request.clone();
        changed["receipt"] = snapshot(id.accounts[5], PROGRAM_ID, vec![0; 123]);
        assert_eq!(run(changed)["error"], "invalid-receipt");
        let mut changed = request;
        changed["mint"]["address"] = json!(id.accounts[7].to_string());
        assert_eq!(run(changed)["error"], "wrong-receipt-address");
    }
}
