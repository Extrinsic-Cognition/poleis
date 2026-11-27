//! Harmony NFT instructions, with no wallet, signing, network or deployment.
//!
//! A completion issuer signs `permit_message`; the buyer's wallet signs the
//! transaction containing the returned Ed25519/mint pair, in that order. A
//! caller must authenticate the fetched sale account and its chain separately.
//! None of these constructors establish that onboarding was completed.

use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

pub use poleis_treasury::completion_permit::{PermitError, PermitFields, PERMIT_LEN};
pub use poleis_treasury::nft_seat_state::{MintCurve, NftSeat};
pub use poleis_treasury::nft_wire::{Metadata, WireError};
use poleis_treasury::{completion_permit, nft_seat_state, nft_wire};

use crate::{
    config_pda_of, distribution_pda, ed25519_instruction, find_pda, fund_pda, ix, registry_pda,
    session_pda, sha, system_meta, tally_pda, treasury_pda, Payload, SignedEnvelope,
    INSTRUCTIONS_SYSVAR_ID, PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
};

pub use poleis_treasury::{
    IX_ATTEST_NFT, IX_CLAIM_DIVIDEND_NFT, IX_MINT_NFT, IX_OPEN_NFT_GENESIS, IX_PROPOSE_NFT,
    IX_RATIFY_NFT_GENESIS, IX_RETIRE_BURNED_NFT, IX_WITHDRAW_RETIRED_NFT,
};

/// The network packet bound, including transaction signatures and message.
pub const MAX_TRANSACTION_BYTES: usize = 1_232;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildError {
    Metadata(WireError),
    Permit(PermitError),
    InvalidTerms,
    InvalidProposal,
    InvalidChoice,
    TooManyAccounts,
    TransactionTooLarge(usize),
}

impl From<WireError> for BuildError {
    fn from(value: WireError) -> Self {
        Self::Metadata(value)
    }
}
impl From<PermitError> for BuildError {
    fn from(value: PermitError) -> Self {
        Self::Permit(value)
    }
}

pub fn curve_pda(unit: u64) -> Pubkey {
    find_pda(&[nft_seat_state::CURVE_SEED, &unit.to_le_bytes()])
}
pub fn mint_pda(unit: u64, completion: &[u8; 32]) -> Pubkey {
    find_pda(&[nft_seat_state::MINT_SEED, &unit.to_le_bytes(), completion])
}
pub fn seat_pda(mint: &Pubkey) -> Pubkey {
    find_pda(&[nft_seat_state::SEAT_SEED, mint.as_ref()])
}
/// Only the initial token account. After resale use the current owner's actual
/// token account, never derive it again from the immutable first buyer.
pub fn initial_token_pda(mint: &Pubkey, buyer: &Pubkey) -> Pubkey {
    find_pda(&[b"seat-token", mint.as_ref(), buyer.as_ref()])
}
pub fn validation_pda(mint: &Pubkey) -> Pubkey {
    find_pda(&[nft_wire::EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()])
}

fn metadata_bytes(metadata: Metadata<'_>) -> Result<Vec<u8>, BuildError> {
    let mut bytes = [0u8; nft_wire::MAX_METADATA_INITIALIZE_LEN];
    let len = nft_wire::initialize_metadata(metadata, &mut bytes)?;
    Ok(bytes[..len].to_vec())
}

/// Hashes the canonical Initialize TokenMetadata instruction, including its
/// eight-byte discriminator, exactly as the issuance program does.
pub fn metadata_digest(metadata: Metadata<'_>) -> Result<[u8; 32], BuildError> {
    Ok(sha(&[&metadata_bytes(metadata)?]))
}

/// Constructs the exact public message for the configured completion issuer.
/// Its private key never enters this client. The issuer authenticates the
/// completion before returning a signature; the transaction remains unsigned.
pub fn permit_message(
    curve: &MintCurve,
    buyer: Pubkey,
    completion: [u8; 32],
    metadata: Metadata<'_>,
    max_price_lamports: u64,
    expiry_slot: u64,
) -> Result<[u8; PERMIT_LEN], BuildError> {
    curve.encode().map_err(|_| BuildError::InvalidTerms)?;
    Ok(completion_permit::encode(&PermitFields {
        program_id: PROGRAM_ID.to_bytes(),
        chain_genesis_hash: curve.genesis_hash,
        unit: curve.unit,
        curve: curve_pda(curve.unit).to_bytes(),
        buyer: buyer.to_bytes(),
        completion_digest: completion,
        metadata_digest: metadata_digest(metadata)?,
        max_price_lamports,
        expiry_slot,
    })?)
}

/// Builds a transaction's two instructions from an issuer-signed permit.
/// The runtime verifies the signature. The buyer must still sign and approve
/// the price ceiling. Keep these two instructions adjacent, in this order.
/// Packet validation assumes the buyer pays transaction fees; adding further
/// instructions requires checking `legacy_transaction_size` again.
pub fn mint_instructions(
    curve: &MintCurve,
    permit: &[u8; PERMIT_LEN],
    issuer_signature: [u8; 64],
    metadata: Metadata<'_>,
    buyer_max_price_lamports: u64,
) -> Result<[Instruction; 2], BuildError> {
    curve.encode().map_err(|_| BuildError::InvalidTerms)?;
    let fields = completion_permit::parse(permit)?;
    for (matches, error) in [
        (
            fields.program_id == PROGRAM_ID.to_bytes(),
            PermitError::WrongProgram,
        ),
        (
            fields.chain_genesis_hash == curve.genesis_hash,
            PermitError::WrongChain,
        ),
        (fields.unit == curve.unit, PermitError::WrongUnit),
        (
            fields.curve == curve_pda(curve.unit).to_bytes(),
            PermitError::WrongCurve,
        ),
        (
            fields.metadata_digest == metadata_digest(metadata)?,
            PermitError::WrongMetadata,
        ),
    ] {
        if !matches {
            return Err(error.into());
        }
    }
    if issuer_signature == [0; 64] {
        return Err(PermitError::ZeroSignature.into());
    }
    if buyer_max_price_lamports == 0 {
        return Err(PermitError::PriceExceeded.into());
    }
    let buyer = Pubkey::new_from_array(fields.buyer);
    let mint = mint_pda(curve.unit, &fields.completion_digest);
    let metadata = metadata_bytes(metadata)?;
    let mut data = vec![IX_MINT_NFT];
    data.extend_from_slice(&buyer_max_price_lamports.to_le_bytes());
    data.extend_from_slice(&metadata[8..]);
    let pair = [
        ed25519_instruction(&[SignedEnvelope {
            pubkey: curve.completion_issuer,
            signature: issuer_signature,
            message: permit.to_vec(),
        }]),
        ix(
            data,
            vec![
                AccountMeta::new(buyer, true),
                AccountMeta::new_readonly(config_pda_of(curve.unit), false),
                AccountMeta::new(curve_pda(curve.unit), false),
                AccountMeta::new(registry_pda(curve.unit), false),
                AccountMeta::new(treasury_pda(curve.unit), false),
                AccountMeta::new(seat_pda(&mint), false),
                AccountMeta::new(mint, false),
                AccountMeta::new(initial_token_pda(&mint, &buyer), false),
                AccountMeta::new(validation_pda(&mint), false),
                AccountMeta::new(fund_pda(), false),
                AccountMeta::new(tally_pda(curve.unit), false),
                AccountMeta::new_readonly(INSTRUCTIONS_SYSVAR_ID, false),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
                system_meta(),
            ],
        ),
    ];
    let size = legacy_transaction_size(buyer, &pair)?;
    if size > MAX_TRANSACTION_BYTES {
        return Err(BuildError::TransactionTooLarge(size));
    }
    Ok(pair)
}

/// Exact legacy serialization size, including compact lengths, all required
/// 64-byte wallet signatures, header, blockhash and compiled account indices.
/// This does not serialize, sign, send or authenticate a transaction.
pub fn legacy_transaction_size(
    payer: Pubkey,
    instructions: &[Instruction],
) -> Result<usize, BuildError> {
    fn compact_len(mut value: usize) -> usize {
        let mut len = 1;
        while value >= 128 {
            value >>= 7;
            len += 1;
        }
        len
    }
    let mut keys = vec![(payer, true)];
    for instruction in instructions {
        for (key, signer) in instruction
            .accounts
            .iter()
            .map(|a| (a.pubkey, a.is_signer))
            .chain(core::iter::once((instruction.program_id, false)))
        {
            if let Some(existing) = keys.iter_mut().find(|(k, _)| *k == key) {
                existing.1 |= signer;
            } else {
                keys.push((key, signer));
            }
        }
    }
    let signatures = keys.iter().filter(|(_, signer)| *signer).count();
    if keys.len() > 256 || signatures > 255 {
        return Err(BuildError::TooManyAccounts);
    }
    let mut size = compact_len(signatures)
        + signatures * 64
        + 3
        + compact_len(keys.len())
        + keys.len() * 32
        + 32
        + compact_len(instructions.len());
    for instruction in instructions {
        size += 1
            + compact_len(instruction.accounts.len())
            + instruction.accounts.len()
            + compact_len(instruction.data.len())
            + instruction.data.len();
    }
    Ok(size)
}

pub struct Genesis<'a> {
    pub root: u64,
    pub charter: [u8; 32],
    pub plan_cap: u64,
    pub plan_period: u64,
    pub governance_window: u64,
    pub rung: u8,
    pub start_lamports: u64,
    pub step_lamports: u64,
    pub opens_slot: u64,
    pub closes_slot: u64,
    pub completion_issuer: Pubkey,
    pub chain_genesis_hash: [u8; 32],
    pub founders: &'a [Pubkey],
    pub name: &'a str,
    pub description: &'a str,
}

/// Follows `init_config_of(root, ...)`; the program requires its existing
/// solo authority. All sale values are explicit, never selected by this client.
pub fn open_genesis(payer: Pubkey, terms: &Genesis<'_>) -> Result<Instruction, BuildError> {
    if terms.rung == 0
        || terms.rung as usize >= poleis_treasury::LADDER.len()
        || terms.founders.len() > crate::PENTAGONS
        || terms.name.len() > crate::MAX_NAME
        || terms.description.len() > crate::MAX_DESC
        || !(crate::MIN_WINDOW_SLOTS..=crate::MAX_WINDOW_SLOTS).contains(&terms.governance_window)
    {
        return Err(BuildError::InvalidTerms);
    }
    MintCurve {
        bump: 0,
        unit: terms.root,
        start_lamports: terms.start_lamports,
        step_lamports: terms.step_lamports,
        minted_count: 0,
        capacity: poleis_treasury::ladder_hexagons(terms.rung as usize),
        opens_slot: terms.opens_slot,
        closes_slot: terms.closes_slot,
        completion_issuer: terms.completion_issuer.to_bytes(),
        genesis_hash: terms.chain_genesis_hash,
    }
    .encode()
    .map_err(|_| BuildError::InvalidTerms)?;
    let mut data = vec![IX_OPEN_NFT_GENESIS];
    data.extend_from_slice(&terms.charter);
    for value in [terms.plan_cap, terms.plan_period, terms.governance_window] {
        data.extend_from_slice(&value.to_le_bytes());
    }
    data.push(terms.rung);
    for value in [
        terms.start_lamports,
        terms.step_lamports,
        terms.opens_slot,
        terms.closes_slot,
    ] {
        data.extend_from_slice(&value.to_le_bytes());
    }
    data.extend_from_slice(terms.completion_issuer.as_ref());
    data.extend_from_slice(&terms.chain_genesis_hash);
    data.push(terms.founders.len() as u8);
    for founder in terms.founders {
        data.extend_from_slice(founder.as_ref());
    }
    for text in [terms.name, terms.description] {
        data.push(text.len() as u8);
        data.extend_from_slice(text.as_bytes());
    }
    Ok(ix(
        data,
        vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(config_pda_of(terms.root), false),
            AccountMeta::new(treasury_pda(terms.root), false),
            AccountMeta::new(registry_pda(terms.root), false),
            AccountMeta::new(curve_pda(terms.root), false),
            system_meta(),
        ],
    ))
}

pub fn ratify_genesis(crank: Pubkey, root: u64) -> Instruction {
    ix(
        vec![IX_RATIFY_NFT_GENESIS],
        vec![
            AccountMeta::new_readonly(crank, true),
            AccountMeta::new(config_pda_of(root), false),
            AccountMeta::new_readonly(registry_pda(root), false),
            AccountMeta::new_readonly(curve_pda(root), false),
        ],
    )
}

/// Public current-owner account references; the program verifies the token
/// amount, mint extensions, wallet signature and live registry membership.
#[derive(Clone, Copy, Debug)]
pub struct OwnerProof {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub token: Pubkey,
    pub unit: u64,
}

impl OwnerProof {
    fn prefix(&self) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new_readonly(seat_pda(&self.mint), false),
            AccountMeta::new_readonly(self.mint, false),
            AccountMeta::new_readonly(self.token, false),
        ]
    }

    pub fn attest(&self, hash: [u8; 32], choice: u8) -> Result<Instruction, BuildError> {
        if ![crate::VOTE_PRO, crate::VOTE_CONTRA, crate::VOTE_ABSTENTIO].contains(&choice) {
            return Err(BuildError::InvalidChoice);
        }
        let mut data = vec![IX_ATTEST_NFT];
        data.extend_from_slice(&hash);
        data.extend_from_slice(&self.unit.to_le_bytes());
        data.push(choice);
        let mut accounts = vec![AccountMeta::new_readonly(self.owner, true)];
        accounts.extend(self.prefix());
        accounts.extend([
            AccountMeta::new(registry_pda(self.unit), false),
            AccountMeta::new(session_pda(&hash, self.unit), false),
        ]);
        Ok(ix(data, accounts))
    }

    pub fn propose(
        &self,
        root: u64,
        hash: [u8; 32],
        manifest: [u8; 32],
        payload: &Payload,
        impacted: &[u64],
    ) -> Result<Instruction, BuildError> {
        if payload.origin() != self.unit
            || !impacted.contains(&self.unit)
            || impacted.len() > crate::MAX_IMPACTED
            || payload.bytes().len() > crate::MAX_PAYLOAD
        {
            return Err(BuildError::InvalidProposal);
        }
        let mut instruction =
            crate::propose_of(root, self.owner, hash, manifest, payload, impacted);
        instruction.data[0] = IX_PROPOSE_NFT;
        let mut accounts = self.prefix();
        accounts.extend(instruction.accounts);
        instruction.accounts = accounts;
        Ok(instruction)
    }

    /// The same wire with the class byte and the payload's bytes given, for a
    /// kind no `Payload` names (a CODEX_MARKET law) or a class with its bits
    /// (the market's, the SUM's): the origin is this seat's unit. A proposal
    /// that asks for the market or the SUM still carries the origin's market
    /// or rule account last, pushed by the caller after this.
    pub fn propose_raw(
        &self,
        root: u64,
        hash: [u8; 32],
        class: u8,
        manifest: [u8; 32],
        payload: &[u8],
        impacted: &[u64],
    ) -> Result<Instruction, BuildError> {
        if !impacted.contains(&self.unit)
            || impacted.len() > crate::MAX_IMPACTED
            || payload.len() > crate::MAX_PAYLOAD
        {
            return Err(BuildError::InvalidProposal);
        }
        let mut instruction = crate::propose_raw_of(
            root, self.owner, hash, class, self.unit, manifest, payload, impacted,
        );
        instruction.data[0] = IX_PROPOSE_NFT;
        let mut accounts = self.prefix();
        accounts.extend(instruction.accounts);
        instruction.accounts = accounts;
        Ok(instruction)
    }

    /// Native SOL dividend claim. Its stable seat index survives resale; the
    /// current owner receives payment and cannot re-claim an already paid bit.
    pub fn claim_dividend(&self, hash: [u8; 32]) -> Instruction {
        let mut data = vec![IX_CLAIM_DIVIDEND_NFT];
        data.extend_from_slice(&hash);
        data.extend_from_slice(seat_pda(&self.mint).as_ref());
        data.push(poleis_treasury::HOLDER_NFT);
        let mut accounts = self.prefix();
        accounts.extend([
            AccountMeta::new(self.owner, true),
            AccountMeta::new(registry_pda(self.unit), false),
            AccountMeta::new(distribution_pda(&hash), false),
            AccountMeta::new(self.owner, false),
        ]);
        ix(data, accounts)
    }

    /// Destination must already be a valid Token-2022 account for this mint.
    /// Token-2022 invokes the hook; clients must never call the hook directly.
    pub fn transfer(&self, destination: Pubkey) -> Instruction {
        Instruction {
            program_id: TOKEN_2022_PROGRAM_ID,
            data: vec![12, 1, 0, 0, 0, 0, 0, 0, 0, 0],
            accounts: vec![
                AccountMeta::new(self.token, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new(destination, false),
                AccountMeta::new_readonly(self.owner, true),
                AccountMeta::new_readonly(seat_pda(&self.mint), false),
                AccountMeta::new(registry_pda(self.unit), false),
                AccountMeta::new_readonly(PROGRAM_ID, false),
                AccountMeta::new_readonly(validation_pda(&self.mint), false),
            ],
        }
    }

    /// An irreversible burn, constructed only: execute both instructions in
    /// one transaction so the registry retires the seat with the token burn.
    pub fn burn_and_retire(&self) -> [Instruction; 2] {
        [
            Instruction {
                program_id: TOKEN_2022_PROGRAM_ID,
                data: vec![15, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                accounts: vec![
                    AccountMeta::new(self.token, false),
                    AccountMeta::new(self.mint, false),
                    AccountMeta::new_readonly(self.owner, true),
                ],
            },
            retire_burned(self.unit, self.mint),
        ]
    }

    /// Withdraw compensation after constitutional retirement while the NFT
    /// still exists. A burned NFT cannot supply current-owner proof.
    pub fn withdraw_retired(&self) -> Instruction {
        ix(
            vec![IX_WITHDRAW_RETIRED_NFT],
            vec![
                AccountMeta::new(self.owner, true),
                AccountMeta::new(seat_pda(&self.mint), false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(self.token, false),
                AccountMeta::new_readonly(registry_pda(self.unit), false),
            ],
        )
    }
}

/// Permissionless reconciliation after a burn; the program requires supply0.
pub fn retire_burned(unit: u64, mint: Pubkey) -> Instruction {
    ix(
        vec![IX_RETIRE_BURNED_NFT],
        vec![
            AccountMeta::new_readonly(seat_pda(&mint), false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(registry_pda(unit), false),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve() -> MintCurve {
        MintCurve {
            bump: 0,
            unit: 0,
            start_lamports: 100_000_000,
            step_lamports: 10_000_000,
            minted_count: 0,
            capacity: 20,
            opens_slot: 100,
            closes_slot: 10_000,
            completion_issuer: [7; 32],
            genesis_hash: [8; 32],
        }
    }
    const META: Metadata<'_> = Metadata {
        name: "Test Lyssajudron",
        symbol: "TEST",
        uri: "https://example.invalid/nft.json",
    };

    #[test]
    fn signed_permit_is_embedded_once_and_binds_metadata_and_curve() {
        let curve = curve();
        let buyer = Pubkey::new_from_array([9; 32]);
        let permit = permit_message(&curve, buyer, [10; 32], META, 200_000_000, 500).unwrap();
        let pair = mint_instructions(&curve, &permit, [11; 64], META, 190_000_000).unwrap();
        completion_permit::check_ed25519_instruction(
            &pair[0].program_id.to_bytes(),
            &pair[0].data,
            &curve.completion_issuer,
            &permit,
        )
        .unwrap();
        assert_eq!(
            pair[1].data.len(),
            1 + 8 + 12 + META.name.len() + META.symbol.len() + META.uri.len()
        );
        assert_eq!(
            pair[1]
                .accounts
                .iter()
                .filter(|meta| meta.is_signer)
                .count(),
            1
        );
        assert_eq!(pair[1].accounts[0].pubkey, buyer);
        assert_eq!(
            pair[1].accounts[5].pubkey,
            seat_pda(&mint_pda(0, &[10; 32]))
        );
        assert_eq!(
            mint_instructions(
                &curve,
                &permit,
                [11; 64],
                Metadata {
                    uri: "https://example.invalid/other.json",
                    ..META
                },
                190_000_000
            ),
            Err(BuildError::Permit(PermitError::WrongMetadata))
        );
        let mut wrong = curve;
        wrong.genesis_hash = [12; 32];
        assert_eq!(
            mint_instructions(&wrong, &permit, [11; 64], META, 190_000_000),
            Err(BuildError::Permit(PermitError::WrongChain))
        );
        assert_eq!(
            mint_instructions(&curve, &permit, [0; 64], META, 190_000_000),
            Err(BuildError::Permit(PermitError::ZeroSignature))
        );
    }

    #[test]
    fn individually_valid_metadata_is_rejected_if_the_wallet_packet_is_too_large() {
        let name = "N".repeat(nft_wire::MAX_NAME_BYTES);
        let symbol = "S".repeat(nft_wire::MAX_SYMBOL_BYTES);
        let uri = "U".repeat(nft_wire::MAX_URI_BYTES);
        let metadata = Metadata {
            name: &name,
            symbol: &symbol,
            uri: &uri,
        };
        let curve = curve();
        let permit = permit_message(
            &curve,
            Pubkey::new_from_array([9; 32]),
            [10; 32],
            metadata,
            200_000_000,
            500,
        )
        .unwrap();
        assert!(
            matches!(mint_instructions(&curve, &permit, [11; 64], metadata, 190_000_000), Err(BuildError::TransactionTooLarge(size)) if size > MAX_TRANSACTION_BYTES)
        );
    }

    #[test]
    fn genesis_rejects_non_increasing_or_unencodable_terms() {
        let payer = Pubkey::new_from_array([9; 32]);
        let founders = [payer];
        let mut terms = Genesis {
            root: 0,
            charter: [1; 32],
            plan_cap: 1_000_000_000,
            plan_period: 10_000,
            governance_window: crate::MIN_WINDOW_SLOTS,
            rung: 1,
            start_lamports: 100_000_000,
            step_lamports: 10_000_000,
            opens_slot: 100,
            closes_slot: 10_000,
            completion_issuer: Pubkey::new_from_array([7; 32]),
            chain_genesis_hash: [8; 32],
            founders: &founders,
            name: "Test",
            description: "",
        };
        let instruction = open_genesis(payer, &terms).unwrap();
        assert_eq!(instruction.data[154], 1);
        assert_eq!(&instruction.data[90..122], &[7; 32]);
        assert_eq!(&instruction.data[122..154], &[8; 32]);
        assert_eq!(instruction.accounts[4].pubkey, curve_pda(0));
        terms.step_lamports = 0;
        assert_eq!(open_genesis(payer, &terms), Err(BuildError::InvalidTerms));
    }

    #[test]
    fn an_owners_raw_proposal_is_its_payload_proposal_byte_for_byte() {
        let proof = OwnerProof {
            owner: Pubkey::new_from_array([3; 32]),
            mint: Pubkey::new_from_array([4; 32]),
            token: Pubkey::new_from_array([5; 32]),
            unit: 0,
        };
        let payload = Payload::Budget { origin: 0, address: Pubkey::new_from_array([6; 32]), amount: 1 };
        let by_payload = proof.propose(0, [7; 32], [8; 32], &payload, &[0, 1]).unwrap();
        let raw = proof.propose_raw(0, [7; 32], payload.class(), [8; 32], &payload.bytes(), &[0, 1]).unwrap();
        assert_eq!(raw, by_payload);
        assert_eq!(raw.data[0], IX_PROPOSE_NFT);
        assert_eq!(
            raw.accounts[..4].iter().map(|m| (m.pubkey, m.is_signer)).collect::<Vec<_>>(),
            vec![(seat_pda(&proof.mint), false), (proof.mint, false), (proof.token, false), (proof.owner, true)],
            "the seat, its mint, the owner's token, then the proposer"
        );
        // a class with its bits rides as given, and the origin is the seat's unit
        let marked = proof.propose_raw(0, [7; 32], payload.class() | crate::CLASS_MARKET, [8; 32], &payload.bytes(), &[0]).unwrap();
        assert_eq!((marked.data[33], &marked.data[34..42]), (payload.class() | crate::CLASS_MARKET, &0u64.to_le_bytes()[..]));
        // a set that leaves the seat's unit out, or a payload past the bound, is refused
        assert_eq!(proof.propose_raw(0, [7; 32], payload.class(), [8; 32], &payload.bytes(), &[1]), Err(BuildError::InvalidProposal));
        assert_eq!(proof.propose_raw(0, [7; 32], payload.class(), [8; 32], &vec![0; crate::MAX_PAYLOAD + 1], &[0]), Err(BuildError::InvalidProposal));
    }

    #[test]
    fn session_view_validates_snapshot_shape_and_active_sum() {
        use crate::{Session, SessionView, DISC_SESSION, LAYOUT_VERSION};
        let mut bytes = vec![0; Session::SIZE];
        bytes[0] = DISC_SESSION;
        bytes[1] = LAYOUT_VERSION;
        assert!(SessionView::parse(&bytes).is_some());
        bytes.push(0);
        assert!(SessionView::parse(&bytes).is_none());
        bytes.resize(
            Session::SIZE + Session::ELECTORATE_HEADER + Session::ELECTORATE_ROW,
            0,
        );
        bytes[Session::SIZE..Session::SIZE + 8].copy_from_slice(&Session::ELECTORATE_MAGIC);
        bytes[Session::SIZE + 8] = 1;
        let row = Session::SIZE + Session::ELECTORATE_HEADER;
        bytes[row..row + 32].fill(9);
        bytes[row + 32] = poleis_treasury::HOLDER_NFT;
        bytes[row + 33..row + 41].copy_from_slice(&3u64.to_le_bytes());
        bytes[row + 41] = 1;
        assert!(SessionView::parse(&bytes).is_none());
        bytes[Session::OFF_ACTIVE_WEIGHT..Session::OFF_ACTIVE_WEIGHT + 8]
            .copy_from_slice(&3u64.to_le_bytes());
        assert_eq!(SessionView::parse(&bytes).unwrap().active_weight, 3);
        bytes[row + 41] = 2;
        assert!(SessionView::parse(&bytes).is_none());
        bytes[row + 41] = 1;
        bytes.pop();
        assert!(SessionView::parse(&bytes).is_none());
    }
}
