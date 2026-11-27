//! Harmony mint integration against the compiled Poleis SBF, the actual
//! Token-2022 ELF and runtime Ed25519 precompile. All keys, prices, completion
//! hashes and metadata here are synthetic test fixtures; no RPC or wallet.

mod built;
use ed25519_dalek::{Signer, SigningKey};
use mollusk_svm::{
    program::{create_program_account_loader_v3, keyed_account_for_system_program},
    Mollusk,
};
use mollusk_svm_programs_token::token2022;
use poleis_client::{
    attest_batch, config_pda, distribution_pda, ed25519_instruction, fund_pda, init_config,
    proposal_hash, proposal_pda, propose, registry_pda, seal, session_pda, tally_pda, treasury_pda,
    vote_envelope, Payload, RegistryView, SessionView, SignedEnvelope, INSTRUCTIONS_SYSVAR_ID,
    PROGRAM_ID,
};
use poleis_treasury::{
    completion_permit::{self, PermitFields, PERMIT_LEN},
    nft_seat_state::{MintCurve, NftSeat},
    nft_wire::{self, Metadata},
    seat_nft::{self, RawAccount},
    state::{Distribution, Registry, DISC_DISTRIBUTION, LAYOUT_VERSION},
};
use sha2::{Digest, Sha256};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

const SOL: u64 = 1_000_000_000;
const START: u64 = 100_000_000;
const STEP: u64 = 10_000_000;
const WINDOW: u64 = 1_000;
const OPEN: u64 = 100;
const CLOSE: u64 = 10_000;
const GENESIS: [u8; 32] = [4; 32];
const UNIT: u64 = 0;
const OPEN_NFT: u8 = 46;
const MINT_NFT: u8 = 47;
const RATIFY_NFT: u8 = 48;
const RETIRE_NFT: u8 = 52;
const WITHDRAW_NFT: u8 = 53;
const META: Metadata<'static> = Metadata {
    name: "Test Lyssajudron",
    symbol: "TEST",
    uri: "https://example.invalid/nft.json",
};

fn w(key: Pubkey) -> AccountMeta {
    AccountMeta::new(key, false)
}
fn r(key: Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(key, false)
}
fn s(key: Pubkey) -> AccountMeta {
    AccountMeta::new(key, true)
}
fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &PROGRAM_ID).0
}
fn curve_pda() -> Pubkey {
    pda(&[b"mint-curve", &UNIT.to_le_bytes()])
}
fn program_ix(tag: u8, bytes: &[u8], accounts: Vec<AccountMeta>) -> Instruction {
    let mut data = vec![tag];
    data.extend_from_slice(bytes);
    Instruction {
        program_id: PROGRAM_ID,
        accounts,
        data,
    }
}
fn token_ix(data: Vec<u8>, accounts: Vec<AccountMeta>) -> Instruction {
    Instruction {
        program_id: token2022::ID,
        accounts,
        data,
    }
}

struct World {
    svm: Mollusk,
    accounts: Vec<(Pubkey, Account)>,
    payer: Pubkey,
    issuer: SigningKey,
}

impl World {
    fn new() -> Self {
        Self::new_with_genesis(GENESIS)
    }

    fn new_with_genesis(genesis: [u8; 32]) -> Self {
        std::env::set_var(
            "SBF_OUT_DIR",
            built::program_dir(),
        );
        let mut svm = Mollusk::new(&PROGRAM_ID, "poleis_treasury");
        token2022::add_program(&mut svm);
        svm.warp_to_slot(OPEN);
        let payer = Pubkey::new_unique();
        let mut world = Self {
            svm,
            accounts: vec![
                keyed_account_for_system_program(),
                token2022::keyed_account(),
                (PROGRAM_ID, create_program_account_loader_v3(&PROGRAM_ID)),
                (payer, Account::new(50 * SOL, 0, &Pubkey::default())),
            ],
            payer,
            issuer: SigningKey::from_bytes(&[71; 32]),
        };
        world.tx(&[init_config(payer, Pubkey::default())], true, &[]);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[9; 32]);
        for number in [10 * SOL, 100_000, WINDOW] {
            bytes.extend_from_slice(&number.to_le_bytes());
        }
        bytes.push(1); // rung one: twenty sale seats, distinct from the chairs
        for number in [START, STEP, OPEN, CLOSE] {
            bytes.extend_from_slice(&number.to_le_bytes());
        }
        bytes.extend_from_slice(&world.issuer.verifying_key().to_bytes());
        bytes.extend_from_slice(&genesis);
        bytes.push(1);
        bytes.extend_from_slice(payer.as_ref());
        bytes.extend_from_slice(&[4, b'T', b'e', b's', b't', 0]);
        let ix = program_ix(
            OPEN_NFT,
            &bytes,
            vec![
                s(payer),
                w(config_pda()),
                w(treasury_pda(UNIT)),
                w(registry_pda(UNIT)),
                w(curve_pda()),
                r(Pubkey::default()),
            ],
        );
        let client_open = poleis_client::nft::open_genesis(
            payer,
            &poleis_client::nft::Genesis {
                root: UNIT,
                charter: [9; 32],
                plan_cap: 10 * SOL,
                plan_period: 100_000,
                governance_window: WINDOW,
                rung: 1,
                start_lamports: START,
                step_lamports: STEP,
                opens_slot: OPEN,
                closes_slot: CLOSE,
                completion_issuer: Pubkey::new_from_array(world.issuer.verifying_key().to_bytes()),
                chain_genesis_hash: genesis,
                founders: &[payer],
                name: "Test",
                description: "",
            },
        )
        .unwrap();
        assert_eq!(
            client_open, ix,
            "public genesis builder disagrees with independent wire fixture"
        );
        world.tx(&[client_open], true, &[]);
        world
    }

    fn wallet(&mut self) -> Pubkey {
        let key = Pubkey::new_unique();
        self.accounts
            .push((key, Account::new(10 * SOL, 0, &Pubkey::default())));
        key
    }

    fn account(&self, key: &Pubkey) -> Account {
        self.accounts
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, a)| a.clone())
            .unwrap_or_default()
    }

    fn touch(&mut self, key: Pubkey) {
        if key != INSTRUCTIONS_SYSVAR_ID && !self.accounts.iter().any(|(k, _)| *k == key) {
            self.accounts.push((key, Account::default()));
        }
    }

    fn replace(&mut self, key: Pubkey, account: Account) {
        self.touch(key);
        self.accounts.iter_mut().find(|(k, _)| *k == key).unwrap().1 = account;
    }

    /// On expected failure compare the SVM's actual returned accounts, before
    /// selectively merging anything. A client-side decision not to merge is
    /// not evidence of atomic rollback. Fee-payer lamports are excluded because
    /// Solana may still charge a failed transaction's fee.
    fn tx(&mut self, instructions: &[Instruction], success: bool, unchanged: &[Pubkey]) {
        for ix in instructions {
            for meta in &ix.accounts {
                self.touch(meta.pubkey);
            }
        }
        let before: Vec<_> = unchanged
            .iter()
            .map(|key| (*key, self.account(key)))
            .collect();
        let result = self.svm.process_transaction_instructions(
            instructions,
            &self.accounts,
            Some(&self.payer),
        );
        assert_eq!(
            result.raw_result.is_ok(),
            success,
            "result: {:?}",
            result.raw_result
        );
        if std::env::var_os("POLEIS_CU").is_some() {
            eprintln!(
                "NFT tx={} CU={} success={success}",
                instructions.last().map(|i| i.data[0]).unwrap_or(255),
                result.compute_units_consumed
            );
        }
        for (key, expected) in before {
            let actual = result
                .resulting_accounts
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, a)| a.clone())
                .unwrap_or_else(|| self.account(&key));
            assert_eq!(actual, expected, "failed transaction changed {key}");
        }
        if success {
            let accounts = result.resulting_accounts.clone();
            for (key, account) in accounts {
                self.replace(key, account);
            }
        }
    }

    fn curve(&self) -> MintCurve {
        MintCurve::decode(&self.account(&curve_pda()).data).unwrap()
    }
    fn registry(&self) -> RegistryView {
        RegistryView::parse(&self.account(&registry_pda(UNIT)).data).unwrap()
    }
    fn warp(&mut self, slot: u64) {
        self.svm.warp_to_slot(slot);
    }

    fn mint_request(&self, buyer: Pubkey, completion: u8, cap: u64, expiry: u64) -> MintRequest {
        let completion = [completion; 32];
        let mint = pda(&[b"seat-mint", &UNIT.to_le_bytes(), &completion]);
        let seat = pda(&[b"nft-seat", mint.as_ref()]);
        let token = pda(&[b"seat-token", mint.as_ref(), buyer.as_ref()]);
        let validation = pda(&[nft_wire::EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()]);
        let mut metadata_bytes = [0; nft_wire::MAX_METADATA_INITIALIZE_LEN];
        let metadata_len = nft_wire::initialize_metadata(META, &mut metadata_bytes).unwrap();
        let permit = completion_permit::encode(&PermitFields {
            program_id: *PROGRAM_ID.as_array(),
            chain_genesis_hash: self.curve().genesis_hash,
            unit: UNIT,
            curve: *curve_pda().as_array(),
            buyer: *buyer.as_array(),
            completion_digest: completion,
            metadata_digest: Sha256::digest(&metadata_bytes[..metadata_len]).into(),
            max_price_lamports: cap,
            expiry_slot: expiry,
        })
        .unwrap();
        let mut data = cap.to_le_bytes().to_vec();
        data.extend_from_slice(&metadata_bytes[8..metadata_len]);
        let ix = program_ix(
            MINT_NFT,
            &data,
            vec![
                s(buyer),
                r(config_pda()),
                w(curve_pda()),
                w(registry_pda(UNIT)),
                w(treasury_pda(UNIT)),
                w(seat),
                w(mint),
                w(token),
                w(validation),
                w(fund_pda()),
                w(tally_pda(UNIT)),
                r(INSTRUCTIONS_SYSVAR_ID),
                r(token2022::ID),
                r(Pubkey::default()),
            ],
        );
        MintRequest {
            buyer,
            mint,
            seat,
            token,
            validation,
            permit,
            ix,
        }
    }

    fn sign(&self, request: &MintRequest) -> [Instruction; 2] {
        self.sign_with(request, &self.issuer)
    }

    fn sign_with(&self, request: &MintRequest, signer: &SigningKey) -> [Instruction; 2] {
        let message = request.permit.to_vec();
        let signature = signer.sign(&message).to_bytes();
        [
            ed25519_instruction(&[SignedEnvelope {
                pubkey: signer.verifying_key().to_bytes(),
                signature,
                message,
            }]),
            request.ix.clone(),
        ]
    }

    fn mint(&mut self, buyer: Pubkey, completion: u8) -> MintRequest {
        let request = self.mint_request(buyer, completion, SOL, CLOSE);
        self.tx(&self.sign(&request), true, &[]);
        request
    }

    fn destination(&mut self, mint: Pubkey, owner: Pubkey) -> Pubkey {
        let token = Pubkey::new_unique();
        let mut create = Vec::new();
        create.extend_from_slice(&0u32.to_le_bytes()); // System CreateAccount
        create.extend_from_slice(
            &self
                .svm
                .sysvars
                .rent
                .minimum_balance(nft_wire::TOKEN_ACCOUNT_LEN)
                .to_le_bytes(),
        );
        create.extend_from_slice(&(nft_wire::TOKEN_ACCOUNT_LEN as u64).to_le_bytes());
        create.extend_from_slice(token2022::ID.as_ref());
        self.tx(
            &[
                Instruction {
                    program_id: Pubkey::default(),
                    accounts: vec![s(self.payer), s(token)],
                    data: create,
                },
                token_ix(
                    nft_wire::initialize_immutable_owner().to_vec(),
                    vec![w(token)],
                ),
                token_ix(
                    nft_wire::initialize_account3(owner.as_array())
                        .unwrap()
                        .to_vec(),
                    vec![w(token), r(mint)],
                ),
            ],
            true,
            &[],
        );
        token
    }

    fn tokens(&self, key: Pubkey) -> u64 {
        u64::from_le_bytes(self.account(&key).data[64..72].try_into().unwrap())
    }
}

struct MintRequest {
    buyer: Pubkey,
    mint: Pubkey,
    seat: Pubkey,
    token: Pubkey,
    validation: Pubkey,
    permit: [u8; PERMIT_LEN],
    ix: Instruction,
}

impl MintRequest {
    fn economic_accounts(&self) -> Vec<Pubkey> {
        vec![
            curve_pda(),
            registry_pda(UNIT),
            treasury_pda(UNIT),
            fund_pda(),
            tally_pda(UNIT),
            self.mint,
            self.seat,
            self.token,
            self.validation,
        ]
    }

    fn transfer(&self, destination: Pubkey, owner: Pubkey) -> Instruction {
        // TransferChecked(amount1, decimals0), with literal hook extras and both
        // the hook program and its validation PDA available to Token-2022 CPI.
        token_ix(
            vec![12, 1, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![
                w(self.token),
                r(self.mint),
                w(destination),
                s(owner),
                r(self.seat),
                w(registry_pda(UNIT)),
                r(PROGRAM_ID),
                r(self.validation),
            ],
        )
    }

    fn retire(&self) -> Instruction {
        program_ix(
            RETIRE_NFT,
            &[],
            vec![r(self.seat), r(self.mint), w(registry_pda(UNIT))],
        )
    }

    fn proposal(&self, owner: Pubkey, token: Pubkey, nonce: u64) -> ([u8; 32], Instruction) {
        let payload = Payload::Name {
            origin: UNIT,
            name: b"A test vote".to_vec(),
            desc: vec![],
        };
        let manifest = [9; 32];
        let hash = proposal_hash(payload.class(), UNIT, &manifest, &payload.hash(), nonce);
        let mut ix = propose(owner, hash, manifest, &payload, &[UNIT]);
        ix.data[0] = 50;
        let mut metas = vec![r(self.seat), r(self.mint), r(token)];
        metas.extend(ix.accounts);
        ix.accounts = metas;
        (hash, ix)
    }

    fn vote(&self, owner: Pubkey, token: Pubkey, hash: &[u8; 32], choice: u8) -> Instruction {
        let mut bytes = hash.to_vec();
        bytes.extend_from_slice(&UNIT.to_le_bytes());
        bytes.push(choice);
        program_ix(
            49,
            &bytes,
            vec![
                s(owner),
                r(self.seat),
                r(self.mint),
                r(token),
                w(registry_pda(UNIT)),
                w(session_pda(hash, UNIT)),
            ],
        )
    }

    fn claim(&self, owner: Pubkey, token: Pubkey, hash: &[u8; 32]) -> Instruction {
        let mut data = hash.to_vec();
        data.extend_from_slice(self.seat.as_ref());
        data.push(poleis_treasury::HOLDER_NFT);
        program_ix(
            51,
            &data,
            vec![
                r(self.seat),
                r(self.mint),
                r(token),
                s(owner),
                w(registry_pda(UNIT)),
                w(distribution_pda(hash)),
                w(owner),
            ],
        )
    }
}

#[test]
fn real_token_program_mints_one_immutable_nft_and_one_stable_seat() {
    let mut world = World::new();
    let buyer = world.wallet();
    let treasury_before = world.account(&treasury_pda(UNIT)).lamports;
    let nft = world.mint(buyer, 11);
    assert_eq!(world.curve().minted_count, 1);
    assert_eq!(
        world.account(&treasury_pda(UNIT)).lamports - treasury_before,
        START
    );
    assert_eq!(world.tokens(nft.token), 1);
    let mint = world.account(&nft.mint);
    let token = world.account(&nft.token);
    assert_eq!(mint.owner, token2022::ID);
    assert_eq!(mint.data.len(), nft_wire::metadata_mint_len(META).unwrap());
    let proof = seat_nft::verify_owner(
        RawAccount {
            address: nft.mint.as_array(),
            program_owner: mint.owner.as_array(),
            data: &mint.data,
        },
        RawAccount {
            address: nft.token.as_array(),
            program_owner: token.owner.as_array(),
            data: &token.data,
        },
        nft.mint.as_array(),
        PROGRAM_ID.as_array(),
        buyer.as_array(),
        true,
    )
    .unwrap();
    assert_eq!(proof.owner(), buyer.as_array());
    let receipt = NftSeat::decode(&world.account(&nft.seat).data).unwrap();
    assert_eq!(
        (receipt.serial, receipt.paid_lamports, receipt.first_buyer),
        (0, START, *buyer.as_array())
    );
    let registry = world.registry();
    assert_eq!(registry.total_seats, 1);
    assert_eq!(registry.holders.len(), 1);
    assert_eq!(registry.holders[0].holder, *nft.seat.as_array());
    assert_eq!(registry.holders[0].kind, poleis_treasury::HOLDER_NFT);
    assert_eq!(
        world.account(&nft.validation).data,
        nft_wire::extra_account_metas(nft.seat.as_array(), registry_pda(UNIT).as_array())
    );
    // Revoked authority must be enforced by the actual Token-2022 program.
    world.tx(
        &[token_ix(
            nft_wire::mint_to_one_checked().to_vec(),
            vec![w(nft.mint), w(nft.token), s(buyer)],
        )],
        false,
        &[nft.mint, nft.token],
    );
}

#[test]
fn prices_increase_and_rejected_stale_caps_or_replays_roll_back_everything() {
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let first = world.mint(alice, 12);
    let stale = world.mint_request(bob, 13, START, CLOSE);
    world.tx(&world.sign(&stale), false, &stale.economic_accounts());
    assert_eq!(world.curve().minted_count, 1);
    let before = world.account(&treasury_pda(UNIT)).lamports;
    let second = world.mint(bob, 13);
    assert_eq!(
        NftSeat::decode(&world.account(&second.seat).data)
            .unwrap()
            .paid_lamports,
        START + STEP
    );
    assert_eq!(
        world.account(&treasury_pda(UNIT)).lamports - before,
        START + STEP
    );
    assert_eq!(world.curve().minted_count, 2);
    world.tx(&world.sign(&first), false, &first.economic_accounts());
    // A newly signed permit with another wallet/expiry/ceiling still cannot
    // issue the same completion again: identity is independent of signature.
    let replay = world.mint_request(bob, 12, 2 * SOL, CLOSE - 1);
    world.tx(&world.sign(&replay), false, &replay.economic_accounts());
    assert_eq!(world.registry().total_seats, 2);
}

#[test]
fn genuine_precompile_rejects_forged_signature_wrong_issuer_and_expired_permit() {
    let mut world = World::new();
    let buyer = world.wallet();
    let request = world.mint_request(buyer, 14, SOL, CLOSE);
    let mut forged = world.sign(&request);
    forged[0].data[48] ^= 1;
    world.tx(&forged, false, &request.economic_accounts());
    let wrong_issuer = SigningKey::from_bytes(&[72; 32]);
    world.tx(
        &world.sign_with(&request, &wrong_issuer),
        false,
        &request.economic_accounts(),
    );
    let expired = world.mint_request(buyer, 14, SOL, OPEN - 1);
    world.tx(&world.sign(&expired), false, &expired.economic_accounts());
    // Omitting the precompile must fail even though permit bytes are valid.
    world.tx(&[request.ix.clone()], false, &request.economic_accounts());
    assert_eq!(world.curve().minted_count, 0);
    world.tx(&world.sign(&request), true, &[]);
}

#[test]
fn permit_binds_chain_metadata_and_buyers_runtime_signature() {
    let mut world = World::new();
    let buyer = world.wallet();
    let request = world.mint_request(buyer, 15, SOL, CLOSE);
    let mut wrong_chain = world.mint_request(buyer, 15, SOL, CLOSE);
    wrong_chain.permit[completion_permit::CHAIN_OFFSET] ^= 1;
    world.tx(
        &world.sign(&wrong_chain),
        false,
        &request.economic_accounts(),
    );
    let mut altered_metadata = world.mint_request(buyer, 15, SOL, CLOSE);
    *altered_metadata.ix.data.last_mut().unwrap() ^= 1;
    world.tx(
        &world.sign(&altered_metadata),
        false,
        &request.economic_accounts(),
    );
    let mut unsigned_buyer = world.mint_request(buyer, 15, SOL, CLOSE);
    unsigned_buyer.ix.accounts[0].is_signer = false;
    world.tx(
        &world.sign(&unsigned_buyer),
        false,
        &request.economic_accounts(),
    );
    let mut buyer_cap = world.mint_request(buyer, 15, SOL, CLOSE);
    buyer_cap.ix.data[1..9].copy_from_slice(&(START - 1).to_le_bytes());
    world.tx(&world.sign(&buyer_cap), false, &request.economic_accounts());
    let mut issuer_cap = world.mint_request(buyer, 15, START - 1, CLOSE);
    issuer_cap.ix.data[1..9].copy_from_slice(&SOL.to_le_bytes());
    world.tx(
        &world.sign(&issuer_cap),
        false,
        &request.economic_accounts(),
    );
    assert_eq!(world.curve().minted_count, 0);
}

#[test]
fn dust_prefunded_addresses_cannot_block_an_authorized_mint() {
    let mut world = World::new();
    let buyer = world.wallet();
    let request = world.mint_request(buyer, 25, SOL, OPEN);
    for key in [
        request.mint,
        request.seat,
        request.token,
        request.validation,
    ] {
        world.replace(key, Account::new(1, 0, &Pubkey::default()));
    }
    world.tx(&world.sign(&request), true, &[]);
    assert_eq!(world.tokens(request.token), 1);
    assert_eq!(world.curve().minted_count, 1);
}

#[test]
fn real_transfer_enforces_rest_frozen_state_and_preserves_seat_index() {
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let nft = world.mint(alice, 16);
    let destination = world.destination(nft.mint, bob);
    let stable = [
        nft.token,
        destination,
        nft.mint,
        nft.seat,
        registry_pda(UNIT),
    ];
    world.tx(&[nft.transfer(destination, alice)], false, &stable);
    world.warp(OPEN + WINDOW);
    let mut registry = world.account(&registry_pda(UNIT));
    registry.data[Registry::OFF_FROZEN] = 1;
    world.replace(registry_pda(UNIT), registry.clone());
    world.tx(&[nft.transfer(destination, alice)], false, &stable);
    registry.data[Registry::OFF_FROZEN] = 0;
    world.replace(registry_pda(UNIT), registry);
    let before = world.registry();
    world.tx(&[nft.transfer(destination, alice)], true, &[]);
    assert_eq!((world.tokens(nft.token), world.tokens(destination)), (0, 1));
    let after = world.registry();
    assert_eq!(after.total_seats, before.total_seats);
    assert_eq!(after.holders.len(), before.holders.len());
    assert_eq!(after.holders[0].holder, before.holders[0].holder);
    assert_eq!(after.holders[0].last_active, OPEN + WINDOW);
    assert_eq!(world.curve().minted_count, 1);
}

#[test]
fn direct_hook_and_mutated_extra_account_metadata_cannot_forge_a_transfer() {
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let nft = world.mint(alice, 17);
    let destination = world.destination(nft.mint, bob);
    world.warp(OPEN + WINDOW);
    let mut data = seat_nft::EXECUTE_DISCRIMINATOR.to_vec();
    data.extend_from_slice(&1u64.to_le_bytes());
    let direct = Instruction {
        program_id: PROGRAM_ID,
        data,
        accounts: vec![
            r(nft.token),
            r(nft.mint),
            r(destination),
            r(alice),
            r(nft.validation),
            r(nft.seat),
            w(registry_pda(UNIT)),
        ],
    };
    let stable = [nft.token, destination, registry_pda(UNIT)];
    world.tx(&[direct], false, &stable);
    // Corrupt only fixture state to exercise validation-account authentication;
    // no public instruction exists to edit this immutable program-owned data.
    let mut validation = world.account(&nft.validation);
    validation.data[17] ^= 1;
    world.replace(nft.validation, validation);
    world.tx(&[nft.transfer(destination, alice)], false, &stable);
}

#[test]
fn burn_reconciliation_retires_the_seat_and_never_recycles_its_history_row() {
    let mut world = World::new();
    let alice = world.wallet();
    let nft = world.mint(alice, 18);
    world.tx(
        &[nft.retire()],
        false,
        &[registry_pda(UNIT), nft.seat, nft.mint],
    );
    // BurnChecked(amount1, decimals0), executed by the real Token-2022 program.
    world.tx(
        &[token_ix(
            vec![15, 1, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![w(nft.token), w(nft.mint), s(alice)],
        )],
        true,
        &[],
    );
    assert_eq!(
        world.registry().total_seats,
        1,
        "burn has no transfer hook; reconcile explicitly"
    );
    let receipt = world.account(&nft.seat);
    world.tx(&[nft.retire()], true, &[]);
    assert_eq!(world.registry().total_seats, 0);
    assert_eq!(
        world.registry().holders[0].kind,
        poleis_treasury::HOLDER_RETIRED_NFT
    );
    assert_eq!(
        world.account(&nft.seat),
        receipt,
        "receipt remains a permanent replay barrier"
    );
    let next = world.mint(alice, 19);
    let registry = world.registry();
    assert_eq!(registry.holders.len(), 2);
    assert_eq!(registry.holders[0].holder, *nft.seat.as_array());
    assert_eq!(registry.holders[1].holder, *next.seat.as_array());
    world.tx(&world.sign(&nft), false, &nft.economic_accounts());
}

#[test]
fn ratification_requires_a_completed_sale_with_a_real_minted_seat() {
    let mut world = World::new();
    let ratify = program_ix(
        RATIFY_NFT,
        &[],
        vec![
            s(world.payer),
            w(config_pda()),
            r(registry_pda(UNIT)),
            r(curve_pda()),
        ],
    );
    world.tx(&[ratify.clone()], false, &[config_pda()]);
    let buyer = world.wallet();
    world.mint(buyer, 20);
    world.tx(&[ratify.clone()], false, &[config_pda()]);
    world.warp(CLOSE);
    world.tx(&[ratify], true, &[]);
    assert_eq!(
        world.account(&config_pda()).data[poleis_treasury::state::Config::OFF_MODE],
        poleis_treasury::MODE_COUNCIL
    );
}

#[test]
fn resale_moves_proposal_and_vote_authority_without_a_second_vote() {
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let nft = world.mint(alice, 21);
    let destination = world.destination(nft.mint, bob);
    world.warp(CLOSE);
    world.tx(
        &[program_ix(
            RATIFY_NFT,
            &[],
            vec![
                s(world.payer),
                w(config_pda()),
                r(registry_pda(UNIT)),
                r(curve_pda()),
            ],
        )],
        true,
        &[],
    );
    let (hash, proposal) = nft.proposal(alice, nft.token, 1);
    world.tx(&[proposal], true, &[]);
    world.tx(
        &[nft.vote(alice, nft.token, &hash, poleis_treasury::VOTE_PRO)],
        true,
        &[],
    );
    let voted = world.account(&session_pda(&hash, UNIT));
    assert_eq!(SessionView::parse(&voted.data).unwrap().votes, 1);
    world.warp(CLOSE + WINDOW);
    world.tx(&[nft.transfer(destination, alice)], true, &[]);
    world.tx(
        &[nft.vote(alice, nft.token, &hash, poleis_treasury::VOTE_CONTRA)],
        false,
        &[session_pda(&hash, UNIT), registry_pda(UNIT)],
    );
    // Bob controls the seat now, but its already-counted session bit survives.
    world.tx(
        &[nft.vote(bob, destination, &hash, poleis_treasury::VOTE_CONTRA)],
        true,
        &[],
    );
    assert_eq!(world.account(&session_pda(&hash, UNIT)), voted);
    let (next_hash, old_owner) = nft.proposal(alice, destination, 2);
    world.tx(
        &[old_owner],
        false,
        &[proposal_pda(&next_hash), session_pda(&next_hash, UNIT)],
    );
    let (_, new_owner) = nft.proposal(bob, destination, 2);
    world.tx(&[new_owner], true, &[]);
    world.tx(
        &[nft.vote(bob, destination, &next_hash, poleis_treasury::VOTE_PRO)],
        true,
        &[],
    );
    assert_eq!(
        SessionView::parse(&world.account(&session_pda(&next_hash, UNIT)).data)
            .unwrap()
            .votes,
        1
    );
}

/// A pre-existing, executed SOL distribution fixture. These tests exercise
/// claims, not the separate constitutional path that creates a distribution.
fn distribution_fixture(world: &mut World, hash: &[u8; 32], amount: u64) {
    let (address, bump) = Pubkey::find_program_address(&[b"distribution", hash], &PROGRAM_ID);
    let mut data = vec![0; Distribution::SIZE];
    data[0] = DISC_DISTRIBUTION;
    data[1] = LAYOUT_VERSION;
    data[Distribution::OFF_HASH..Distribution::OFF_HASH + 32].copy_from_slice(hash);
    data[Distribution::OFF_UNIT..Distribution::OFF_UNIT + 8].copy_from_slice(&UNIT.to_le_bytes());
    data[Distribution::OFF_AMOUNT..Distribution::OFF_AMOUNT + 8]
        .copy_from_slice(&amount.to_le_bytes());
    data[Distribution::OFF_TOTAL_WEIGHT..Distribution::OFF_TOTAL_WEIGHT + 8]
        .copy_from_slice(&1u64.to_le_bytes());
    data[Distribution::OFF_BUMP] = bump;
    data[Distribution::OFF_PAYER..Distribution::OFF_PAYER + 32]
        .copy_from_slice(world.payer.as_ref());
    data[Distribution::OFF_CREATED..Distribution::OFF_CREATED + 8]
        .copy_from_slice(&world.svm.sysvars.clock.slot.to_le_bytes());
    world.replace(
        address,
        Account {
            lamports: amount + world.svm.sysvars.rent.minimum_balance(data.len()),
            data,
            owner: PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
}

#[test]
fn dividends_follow_current_owner_and_claim_bits_survive_resale() {
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let nft = world.mint(alice, 22);
    let destination = world.destination(nft.mint, bob);
    let hash = [31; 32];
    distribution_fixture(&mut world, &hash, START);
    let balance = world.account(&alice).lamports;
    world.tx(&[nft.claim(alice, nft.token, &hash)], true, &[]);
    assert_eq!(world.account(&alice).lamports - balance, START);
    world.warp(OPEN + WINDOW);
    world.tx(&[nft.transfer(destination, alice)], true, &[]);
    world.tx(
        &[nft.claim(bob, destination, &hash)],
        false,
        &[distribution_pda(&hash), registry_pda(UNIT)],
    );
    let fresh = [32; 32];
    distribution_fixture(&mut world, &fresh, STEP);
    world.tx(
        &[nft.claim(alice, destination, &fresh)],
        false,
        &[distribution_pda(&fresh), registry_pda(UNIT)],
    );
    let balance = world.account(&bob).lamports;
    world.tx(&[nft.claim(bob, destination, &fresh)], true, &[]);
    assert_eq!(world.account(&bob).lamports - balance, STEP);
}

/// A SOL distribution cut now with its roster, as `open_distribution`
/// writes it: every registry row's identity, kind and seats × weight as it
/// stands. Returns its bytes' length.
fn rostered_distribution_fixture(world: &mut World, hash: &[u8; 32], amount: u64) -> usize {
    let registry = world.account(&registry_pda(UNIT)).data;
    let count = registry[Registry::OFF_HOLDER_COUNT] as usize;
    let (address, bump) = Pubkey::find_program_address(&[b"distribution", hash], &PROGRAM_ID);
    let mut data = vec![0; Distribution::SIZE + Distribution::ROSTER_HEADER + count * Distribution::ROSTER_ROW];
    data[0] = DISC_DISTRIBUTION;
    data[1] = LAYOUT_VERSION;
    data[Distribution::OFF_HASH..Distribution::OFF_HASH + 32].copy_from_slice(hash);
    data[Distribution::OFF_UNIT..Distribution::OFF_UNIT + 8].copy_from_slice(&UNIT.to_le_bytes());
    data[Distribution::OFF_AMOUNT..Distribution::OFF_AMOUNT + 8].copy_from_slice(&amount.to_le_bytes());
    data[Distribution::OFF_BUMP] = bump;
    data[Distribution::OFF_PAYER..Distribution::OFF_PAYER + 32].copy_from_slice(world.payer.as_ref());
    data[Distribution::OFF_CREATED..Distribution::OFF_CREATED + 8]
        .copy_from_slice(&world.svm.sysvars.clock.slot.to_le_bytes());
    data[Distribution::SIZE..Distribution::SIZE + 8].copy_from_slice(&Distribution::ROSTER_MAGIC);
    data[Distribution::SIZE + 8] = count as u8;
    let mut total = 0u64;
    for i in 0..count {
        let from = Registry::OFF_HOLDERS + i * Registry::HOLDER_LEN;
        let to = Distribution::SIZE + Distribution::ROSTER_HEADER + i * Distribution::ROSTER_ROW;
        data[to..to + 33].copy_from_slice(&registry[from..from + 33]);
        let weight = u64::from_le_bytes(registry[from + 33..from + 41].try_into().unwrap())
            * u64::from_le_bytes(registry[from + 41..from + 49].try_into().unwrap());
        data[to + 33..to + 41].copy_from_slice(&weight.to_le_bytes());
        total += weight;
    }
    data[Distribution::OFF_TOTAL_WEIGHT..Distribution::OFF_TOTAL_WEIGHT + 8].copy_from_slice(&total.to_le_bytes());
    let length = data.len();
    world.replace(
        address,
        Account {
            lamports: amount + world.svm.sysvars.rent.minimum_balance(length),
            data,
            owner: PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    length
}

#[test]
fn a_rostered_dividend_pays_the_seat_the_cut_named_to_whoever_owns_it_now() {
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let carol = world.wallet();
    let nft = world.mint(alice, 41);
    let destination = world.destination(nft.mint, bob);
    // the cut: the floor as it stands, alice's seat on it
    let hash = [41; 32];
    rostered_distribution_fixture(&mut world, &hash, START);
    let cut = world.account(&distribution_pda(&hash)).data;
    let (_, weight) = Distribution::roster_find(&cut, poleis_treasury::HOLDER_NFT, nft.seat.as_array())
        .unwrap()
        .expect("the cut names alice's seat");
    let total = u64::from_le_bytes(
        cut[Distribution::OFF_TOTAL_WEIGHT..Distribution::OFF_TOTAL_WEIGHT + 8].try_into().unwrap(),
    );
    let share = (START as u128 * weight as u128 / total as u128) as u64;
    assert!(weight > 0 && share > 0);
    // after the cut: carol mints a seat of her own, and alice resells hers to bob
    let late = world.mint(carol, 42);
    world.warp(OPEN + WINDOW);
    world.tx(&[nft.transfer(destination, alice)], true, &[]);
    // a seat minted after the cut is not on its roster: it claims nothing
    world.tx(
        &[late.claim(carol, late.token, &hash)],
        false,
        &[distribution_pda(&hash), registry_pda(UNIT)],
    );
    // the seller owns the seat no more: nothing
    world.tx(
        &[nft.claim(alice, nft.token, &hash)],
        false,
        &[distribution_pda(&hash), registry_pda(UNIT)],
    );
    // the seat the cut named is paid, to whoever owns it now, and the claim
    // writes nothing in the registry
    let balance = world.account(&bob).lamports;
    let registry = world.account(&registry_pda(UNIT)).data;
    world.tx(&[nft.claim(bob, destination, &hash)], true, &[]);
    assert_eq!(world.account(&bob).lamports - balance, share);
    assert_eq!(world.account(&registry_pda(UNIT)).data, registry);
    // once
    world.tx(
        &[nft.claim(bob, destination, &hash)],
        false,
        &[distribution_pda(&hash), registry_pda(UNIT)],
    );
}

#[test]
fn only_current_owner_can_withdraw_compensation_after_retirement() {
    let mut world = World::new();
    let alice = world.wallet();
    let impostor = world.wallet();
    let nft = world.mint(alice, 23);
    let withdraw = |owner| {
        let expected = program_ix(
            WITHDRAW_NFT,
            &[],
            vec![
                s(owner),
                w(nft.seat),
                r(nft.mint),
                r(nft.token),
                r(registry_pda(UNIT)),
            ],
        );
        let instruction = poleis_client::nft::OwnerProof {
            owner,
            mint: nft.mint,
            token: nft.token,
            unit: UNIT,
        }
        .withdraw_retired();
        assert_eq!(instruction, expected);
        instruction
    };
    let mut seat = world.account(&nft.seat);
    seat.lamports += STEP;
    world.replace(nft.seat, seat);
    world.tx(&[withdraw(alice)], false, &[nft.seat]);
    // Fixture for the post-expulsion state: the NFT still exists and the
    // constitutional action has retired its row and left compensation at PDA.
    let mut registry = world.account(&registry_pda(UNIT));
    registry.data[Registry::OFF_HOLDERS + 32] = poleis_treasury::HOLDER_RETIRED_NFT;
    registry.data[Registry::OFF_HOLDERS + 33..Registry::OFF_HOLDERS + 57].fill(0);
    registry.data[Registry::OFF_TOTAL_SEATS..Registry::OFF_TOTAL_SEATS + 8].fill(0);
    world.replace(registry_pda(UNIT), registry);
    world.tx(&[withdraw(impostor)], false, &[nft.seat]);
    let balance = world.account(&alice).lamports;
    world.tx(&[withdraw(alice)], true, &[]);
    assert_eq!(world.account(&alice).lamports - balance, STEP);
    assert_eq!(
        world.account(&nft.seat).lamports,
        world.svm.sysvars.rent.minimum_balance(NftSeat::SIZE)
    );
    assert!(NftSeat::decode(&world.account(&nft.seat).data).is_ok());
}

fn shortvec(mut value: usize, out: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// Actual legacy transaction wire shape, including signatures and compiled
/// account indices. Zero signatures/blockhash are deliberate size-only fixture
/// bytes; this serializer neither signs nor sends a transaction.
fn legacy_transaction_bytes(payer: Pubkey, instructions: &[Instruction]) -> Vec<u8> {
    let mut keys = vec![(payer, true, true)];
    for ix in instructions {
        for meta in ix.accounts.iter().cloned().chain([r(ix.program_id)]) {
            if let Some(existing) = keys.iter_mut().find(|(key, _, _)| *key == meta.pubkey) {
                existing.1 |= meta.is_signer;
                existing.2 |= meta.is_writable;
            } else {
                keys.push((meta.pubkey, meta.is_signer, meta.is_writable));
            }
        }
    }
    keys.sort_by_key(|(key, signer, writable)| {
        if *key == payer {
            0
        } else {
            1 + (!*signer as u8) * 2 + (!*writable as u8)
        }
    });
    let signers = keys.iter().filter(|(_, signer, _)| *signer).count();
    let readonly_signed = keys
        .iter()
        .filter(|(_, signer, writable)| *signer && !*writable)
        .count();
    let readonly_unsigned = keys
        .iter()
        .filter(|(_, signer, writable)| !*signer && !*writable)
        .count();
    let mut bytes = Vec::new();
    shortvec(signers, &mut bytes);
    bytes.resize(bytes.len() + signers * 64, 0);
    bytes.extend_from_slice(&[
        signers as u8,
        readonly_signed as u8,
        readonly_unsigned as u8,
    ]);
    shortvec(keys.len(), &mut bytes);
    for (key, _, _) in &keys {
        bytes.extend_from_slice(key.as_ref());
    }
    bytes.extend_from_slice(&[0; 32]);
    shortvec(instructions.len(), &mut bytes);
    for ix in instructions {
        bytes.push(
            keys.iter()
                .position(|(key, _, _)| *key == ix.program_id)
                .unwrap() as u8,
        );
        shortvec(ix.accounts.len(), &mut bytes);
        for meta in &ix.accounts {
            bytes.push(
                keys.iter()
                    .position(|(key, _, _)| *key == meta.pubkey)
                    .unwrap() as u8,
            );
        }
        shortvec(ix.data.len(), &mut bytes);
        bytes.extend_from_slice(&ix.data);
    }
    bytes
}

#[test]
fn launch_metadata_fits_a_real_legacy_wallet_transaction_without_lookup_tables() {
    let mut world = World::new();
    let buyer = world.wallet();
    let request = world.mint_request(buyer, 24, SOL, CLOSE);
    let instructions = world.sign(&request);
    let bytes = legacy_transaction_bytes(buyer, &instructions);
    assert!(
        bytes.len() <= 1_232,
        "mint needs {} transaction bytes",
        bytes.len()
    );
    assert_eq!(
        instructions[0].data.len(),
        completion_permit::ED25519_DATA_LEN
    );
    assert_eq!(
        instructions[1].data.len(),
        1 + 8 + 12 + META.name.len() + META.symbol.len() + META.uri.len()
    );
    assert_eq!(bytes[0], 1, "only the buyer wallet signs the transaction");
    if std::env::var_os("POLEIS_CU").is_some() {
        eprintln!("NFT legacy transaction bytes={}", bytes.len());
    }
}

/// Snapshot fixtures for weight and seven independent chairs. All mints,
/// ownership changes, votes and burns under test still execute actual programs.
fn electorate_fixture(world: &mut World, weights: &[(u64, u64)]) -> Vec<SigningKey> {
    let chairs: Vec<_> = (81..88)
        .map(|seed| SigningKey::from_bytes(&[seed; 32]))
        .collect();
    let mut account = world.account(&registry_pda(UNIT));
    for (i, (weight, active)) in weights.iter().enumerate() {
        let base = Registry::OFF_HOLDERS + i * Registry::HOLDER_LEN;
        account.data[base + 41..base + 49].copy_from_slice(&weight.to_le_bytes());
        account.data[base + 49..base + 57].copy_from_slice(&active.to_le_bytes());
    }
    for (i, key) in chairs.iter().enumerate() {
        let base = Registry::OFF_PENTAGONS + i * Registry::PENTAGON_LEN;
        account.data[base..base + 32].copy_from_slice(&key.verifying_key().to_bytes());
        account.data[base + 32] = poleis_treasury::HOLDER_KEY;
        account.data[base + 33..base + 49].fill(0);
    }
    world.replace(registry_pda(UNIT), account);
    chairs
}

fn chairs_pro(world: &mut World, hash: &[u8; 32], chairs: &[SigningKey]) {
    let message = vote_envelope(hash, UNIT, poleis_treasury::VOTE_PRO);
    for chunk in chairs.chunks(4) {
        let signed: Vec<_> = chunk
            .iter()
            .map(|key| {
                (
                    Pubkey::new_from_array(key.verifying_key().to_bytes()),
                    poleis_treasury::VOTE_PRO,
                    key.sign(&message).to_bytes(),
                )
            })
            .collect();
        world.tx(&attest_batch(world.payer, *hash, UNIT, &signed), true, &[]);
    }
}

fn ratify(world: &mut World) {
    world.tx(
        &[program_ix(
            RATIFY_NFT,
            &[],
            vec![
                s(world.payer),
                w(config_pda()),
                r(registry_pda(UNIT)),
                r(curve_pda()),
            ],
        )],
        true,
        &[],
    );
}

fn burn(world: &mut World, nft: &MintRequest) {
    world.tx(
        &[
            token_ix(
                vec![15, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                vec![w(nft.token), w(nft.mint), s(nft.buyer)],
            ),
            nft.retire(),
        ],
        true,
        &[],
    );
}

fn assert_stays_split(world: &mut World, hash: &[u8; 32]) {
    let session = session_pda(hash, UNIT);
    world.tx(
        &[seal(world.payer, *hash, UNIT)],
        false,
        &[session, registry_pda(UNIT)],
    );
    let deadline = SessionView::parse(&world.account(&session).data)
        .unwrap()
        .deadline;
    world.warp(deadline + 1);
    world.tx(&[seal(world.payer, *hash, UNIT)], true, &[]);
    assert_eq!(
        SessionView::parse(&world.account(&session).data)
            .unwrap()
            .state,
        poleis_treasury::STATE_DEADLOCK
    );
}

#[test]
fn a_transfer_cannot_turn_an_already_cast_minority_into_a_majority() {
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let buyer = world.wallet();
    let first = world.mint(alice, 26);
    world.mint(bob, 27);
    let destination = world.destination(first.mint, buyer);
    world.warp(CLOSE);
    ratify(&mut world);
    let chairs = electorate_fixture(&mut world, &[(3, CLOSE), (4, CLOSE)]);
    let (hash, proposal) = first.proposal(alice, first.token, 26);
    world.tx(&[proposal], true, &[]);
    world.tx(
        &[first.vote(alice, first.token, &hash, poleis_treasury::VOTE_PRO)],
        true,
        &[],
    );
    chairs_pro(&mut world, &hash, &chairs);
    world.warp(CLOSE + WINDOW + 1);
    world.tx(&[first.transfer(destination, alice)], true, &[]);
    assert_eq!(world.registry().holders[0].weight, 1);
    assert_stays_split(&mut world, &hash);
}

#[test]
fn burning_a_voted_seat_cannot_shrink_its_session_denominator() {
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let first = world.mint(alice, 28);
    world.mint(bob, 29);
    world.warp(CLOSE);
    ratify(&mut world);
    let chairs = electorate_fixture(&mut world, &[(3, CLOSE), (4, CLOSE)]);
    let (hash, proposal) = first.proposal(alice, first.token, 28);
    world.tx(&[proposal], true, &[]);
    world.tx(
        &[first.vote(alice, first.token, &hash, poleis_treasury::VOTE_PRO)],
        true,
        &[],
    );
    chairs_pro(&mut world, &hash, &chairs);
    burn(&mut world, &first);
    assert_stays_split(&mut world, &hash);
}

#[test]
fn retiring_the_whole_floor_does_not_convert_a_session_to_chair_only_rule() {
    let mut world = World::new();
    let alice = world.wallet();
    let first = world.mint(alice, 30);
    world.warp(CLOSE);
    ratify(&mut world);
    let chairs = electorate_fixture(&mut world, &[(1, CLOSE)]);
    let (hash, proposal) = first.proposal(alice, first.token, 30);
    world.tx(&[proposal], true, &[]);
    chairs_pro(&mut world, &hash, &chairs);
    burn(&mut world, &first);
    assert_eq!(world.registry().total_seats, 0);
    assert_stays_split(&mut world, &hash);
}

#[test]
fn staggered_dormant_voters_keep_each_original_weight_even_after_one_burns() {
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let baseline = world.wallet();
    let first = world.mint(alice, 31);
    let second = world.mint(bob, 32);
    world.mint(baseline, 33);
    let now = poleis_treasury::ACTIVE_WINDOW_SLOTS + CLOSE;
    world.warp(now);
    ratify(&mut world);
    let chairs = electorate_fixture(&mut world, &[(3, 0), (3, 0), (7, now)]);
    let (hash, proposal) = first.proposal(alice, first.token, 31);
    world.tx(&[proposal], true, &[]);
    world.tx(
        &[first.vote(alice, first.token, &hash, poleis_treasury::VOTE_PRO)],
        true,
        &[],
    );
    burn(&mut world, &first);
    world.tx(
        &[second.vote(bob, second.token, &hash, poleis_treasury::VOTE_PRO)],
        true,
        &[],
    );
    chairs_pro(&mut world, &hash, &chairs);
    // Six PRO over the union's thirteen, not six over scalar max(active)=ten.
    assert_stays_split(&mut world, &hash);
}

#[test]
fn a_new_owner_votes_the_original_session_weight_after_resale() {
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let buyer = world.wallet();
    let first = world.mint(alice, 34);
    world.mint(bob, 35);
    let destination = world.destination(first.mint, buyer);
    world.warp(CLOSE);
    ratify(&mut world);
    electorate_fixture(&mut world, &[(3, CLOSE), (4, CLOSE)]);
    let (hash, proposal) = first.proposal(alice, first.token, 34);
    world.tx(&[proposal], true, &[]);
    world.warp(CLOSE + WINDOW + 1);
    world.tx(&[first.transfer(destination, alice)], true, &[]);
    assert_eq!(world.registry().holders[0].weight, 1);
    let session = session_pda(&hash, UNIT);
    world.tx(
        &[first.vote(alice, first.token, &hash, poleis_treasury::VOTE_PRO)],
        false,
        &[session, registry_pda(UNIT)],
    );
    world.tx(
        &[first.vote(buyer, destination, &hash, poleis_treasury::VOTE_PRO)],
        true,
        &[],
    );
    let votes = SessionView::parse(&world.account(&session).data).unwrap();
    assert_eq!(votes.hex_pro, 3);
    assert_eq!(votes.active_weight, 7);
    assert_eq!(votes.votes, 1);
}

#[test]
fn public_client_builders_execute_mint_owner_actions_resale_and_burn() {
    use poleis_client::nft;
    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let curve = world.curve();
    let permit = nft::permit_message(&curve, alice, [36; 32], META, SOL, CLOSE).unwrap();
    let pair = nft::mint_instructions(
        &curve,
        &permit,
        world.issuer.sign(&permit).to_bytes(),
        META,
        SOL,
    )
    .unwrap();
    assert_eq!(
        nft::legacy_transaction_size(alice, &pair).unwrap(),
        legacy_transaction_bytes(alice, &pair).len(),
    );
    world.tx(&pair, true, &[]);
    let mint = nft::mint_pda(UNIT, &[36; 32]);
    let mut proof = nft::OwnerProof {
        owner: alice,
        mint,
        token: nft::initial_token_pda(&mint, &alice),
        unit: UNIT,
    };
    assert_eq!(world.tokens(proof.token), 1);
    world.warp(CLOSE);
    world.tx(&[nft::ratify_genesis(world.payer, UNIT)], true, &[]);
    let payload = Payload::Name {
        origin: UNIT,
        name: b"Client test".to_vec(),
        desc: vec![],
    };
    let hash = proposal_hash(payload.class(), UNIT, &[9; 32], &payload.hash(), 36);
    world.tx(
        &[proof
            .propose(UNIT, hash, [9; 32], &payload, &[UNIT])
            .unwrap()],
        true,
        &[],
    );
    world.tx(
        &[proof.attest(hash, poleis_treasury::VOTE_PRO).unwrap()],
        true,
        &[],
    );
    let dividend = [37; 32];
    distribution_fixture(&mut world, &dividend, 1_000_000);
    world.tx(&[proof.claim_dividend(dividend)], true, &[]);
    world.warp(CLOSE + WINDOW + 1);
    let destination = world.destination(mint, bob);
    world.tx(&[proof.transfer(destination)], true, &[]);
    assert_eq!(world.tokens(destination), 1);
    proof.owner = bob;
    proof.token = destination;
    world.tx(&proof.burn_and_retire(), true, &[]);
    assert_eq!(world.registry().total_seats, 0);
}

#[test]
fn post_opening_legacy_admissions_and_replaced_rows_cannot_join_the_electorate() {
    fn legacy_row(world: &mut World, index: usize, key: &SigningKey) {
        let mut account = world.account(&registry_pda(UNIT));
        let previous_count = account.data[Registry::OFF_HOLDER_COUNT] as usize;
        let count = previous_count.max(index + 1);
        account
            .data
            .resize(Registry::OFF_HOLDERS + count * Registry::HOLDER_LEN, 0);
        account.data[Registry::OFF_HOLDER_COUNT] = count as u8;
        if count > previous_count {
            let total = world.registry().total_seats + 1;
            account.data[Registry::OFF_TOTAL_SEATS..Registry::OFF_TOTAL_SEATS + 8]
                .copy_from_slice(&total.to_le_bytes());
        }
        let row = Registry::OFF_HOLDERS + index * Registry::HOLDER_LEN;
        account.data[row..row + 32].copy_from_slice(&key.verifying_key().to_bytes());
        account.data[row + 32] = poleis_treasury::HOLDER_KEY;
        account.data[row + 33..row + 41].copy_from_slice(&1u64.to_le_bytes());
        account.data[row + 41..row + 49].copy_from_slice(&3u64.to_le_bytes());
        account.data[row + 49..row + 57].copy_from_slice(&CLOSE.to_le_bytes());
        account.lamports = world.svm.sysvars.rent.minimum_balance(account.data.len());
        world.replace(registry_pda(UNIT), account);
    }
    let mut world = World::new();
    let alice = world.wallet();
    let first = world.mint(alice, 38);
    world.warp(CLOSE);
    ratify(&mut world);
    let original = SigningKey::from_bytes(&[91; 32]);
    let newcomer = SigningKey::from_bytes(&[92; 32]);
    let replacement = SigningKey::from_bytes(&[93; 32]);
    legacy_row(&mut world, 1, &original);
    let (hash, proposal) = first.proposal(alice, first.token, 38);
    world.tx(&[proposal], true, &[]);
    let session = session_pda(&hash, UNIT);
    let before = world.account(&session);
    assert_eq!(SessionView::parse(&before.data).unwrap().active_weight, 4);
    legacy_row(&mut world, 2, &newcomer);
    legacy_row(&mut world, 1, &replacement);
    world.warp(CLOSE + 1);
    let message = vote_envelope(&hash, UNIT, poleis_treasury::VOTE_PRO);
    let votes: Vec<_> = [&newcomer, &replacement]
        .into_iter()
        .map(|key| {
            (
                Pubkey::new_from_array(key.verifying_key().to_bytes()),
                poleis_treasury::VOTE_PRO,
                key.sign(&message).to_bytes(),
            )
        })
        .collect();
    world.tx(&attest_batch(world.payer, hash, UNIT, &votes), true, &[]);
    assert_eq!(
        world.account(&session),
        before,
        "late identities acquired a session voice"
    );
    assert_eq!(world.registry().holders[1].last_active, CLOSE);
    assert_eq!(world.registry().holders[2].last_active, CLOSE);
    world.tx(
        &[first.vote(alice, first.token, &hash, poleis_treasury::VOTE_PRO)],
        true,
        &[],
    );
    let after = SessionView::parse(&world.account(&session).data).unwrap();
    assert_eq!(after.hex_pro, 1);
    assert_eq!(after.active_weight, 4);
    assert_eq!(after.votes, 1);
}

#[test]
fn client_receipt_observes_real_issuance_resale_and_burn_without_a_signer_claim() {
    use poleis_client::nft_receipt::{
        validate_receipt, OwnershipObservation, ReceiptError, ReceiptExpectation,
    };

    fn raw<'a>(key: &'a Pubkey, account: &'a Account) -> RawAccount<'a> {
        RawAccount {
            address: key.as_array(),
            program_owner: account.owner.as_array(),
            data: &account.data,
        }
    }

    let mut world = World::new();
    let alice = world.wallet();
    let bob = world.wallet();
    let nft = world.mint(alice, 47);
    let mut expected = ReceiptExpectation {
        unit: UNIT,
        completion: [47; 32],
        first_buyer: alice,
        max_price_lamports: START,
        metadata: META,
    };
    let receipt = world.account(&nft.seat);
    let mint = world.account(&nft.mint);
    let token = world.account(&nft.token);
    let issued = validate_receipt(
        raw(&nft.seat, &receipt),
        raw(&nft.mint, &mint),
        Some(raw(&nft.token, &token)),
        &expected,
    )
    .unwrap();
    assert_eq!(issued.mint, nft.mint);
    assert_eq!(issued.seat, nft.seat);
    assert_eq!(issued.receipt.serial, 0);
    assert_eq!(issued.receipt.paid_lamports, START);
    assert_eq!(issued.ownership, OwnershipObservation::HeldBy(alice));
    assert_eq!(
        validate_receipt(
            raw(&nft.seat, &receipt),
            raw(&nft.mint, &mint),
            None,
            &expected,
        )
        .unwrap()
        .ownership,
        OwnershipObservation::NotObserved
    );
    expected.max_price_lamports -= 1;
    assert_eq!(
        validate_receipt(
            raw(&nft.seat, &receipt),
            raw(&nft.mint, &mint),
            None,
            &expected,
        ),
        Err(ReceiptError::PriceExceeded)
    );
    expected.max_price_lamports = START;

    // Use genuine Token-2022 execution and the transfer hook. The permanent
    // receipt stays Alice's issuance while the token's observed owner is Bob.
    world.warp(OPEN + WINDOW);
    let destination = world.destination(nft.mint, bob);
    world.tx(&[nft.transfer(destination, alice)], true, &[]);
    let receipt = world.account(&nft.seat);
    let mint = world.account(&nft.mint);
    let sold_token = world.account(&nft.token);
    let current_token = world.account(&destination);
    let resale = validate_receipt(
        raw(&nft.seat, &receipt),
        raw(&nft.mint, &mint),
        Some(raw(&destination, &current_token)),
        &expected,
    )
    .unwrap();
    assert_eq!(resale.receipt, issued.receipt);
    assert_eq!(resale.ownership, OwnershipObservation::HeldBy(bob));
    assert_eq!(
        validate_receipt(
            raw(&nft.seat, &receipt),
            raw(&nft.mint, &mint),
            Some(raw(&nft.token, &sold_token)),
            &expected,
        ),
        Err(ReceiptError::InvalidToken(
            seat_nft::SeatNftError::InvalidAmount
        ))
    );
    expected.first_buyer = bob;
    assert_eq!(
        validate_receipt(
            raw(&nft.seat, &receipt),
            raw(&nft.mint, &mint),
            Some(raw(&destination, &current_token)),
            &expected,
        ),
        Err(ReceiptError::WrongIdentity)
    );
    expected.first_buyer = alice;

    world.tx(
        &[
            token_ix(
                vec![15, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                vec![w(destination), w(nft.mint), s(bob)],
            ),
            nft.retire(),
        ],
        true,
        &[],
    );
    let receipt = world.account(&nft.seat);
    let mint = world.account(&nft.mint);
    let empty_token = world.account(&destination);
    for token in [None, Some(raw(&destination, &empty_token))] {
        let burned = validate_receipt(
            raw(&nft.seat, &receipt),
            raw(&nft.mint, &mint),
            token,
            &expected,
        )
        .unwrap();
        assert_eq!(burned.receipt, issued.receipt);
        assert_eq!(burned.ownership, OwnershipObservation::Burned);
    }
}

/// An opt-in artifact for the Node/Rust ABI test, made only from executed SVM
/// accounts. The public mainnet domain is initialized by the real instruction;
/// all keys, amounts and completions remain synthetic, with no network access.
#[test]
fn export_synthetic_mint_accounts_for_the_host_adapter() {
    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    use std::str::FromStr;

    if std::env::var("POLEIS_EXPORT_MINT_FIXTURE").as_deref() != Ok("1") {
        return;
    }
    fn snapshot(world: &World, address: Pubkey) -> String {
        let account = world.account(&address);
        if account.lamports == 0 && account.data.is_empty() && account.owner == Pubkey::default() {
            return format!(r#"{{"address":"{address}","account":null}}"#);
        }
        format!(
            r#"{{"address":"{address}","account":{{"owner":"{}","lamports":"{}","dataBase64":"{}","executable":{}}}}}"#,
            account.owner,
            account.lamports,
            B64.encode(&account.data),
            account.executable
        )
    }
    fn snapshots(world: &World, request: &MintRequest) -> String {
        format!(
            "[{}]",
            request.ix.accounts[..11]
                .iter()
                .map(|a| snapshot(world, a.pubkey))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
    let genesis = Pubkey::from_str("5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d").unwrap();
    let mut world = World::new_with_genesis(genesis.to_bytes());
    let buyer =
        Pubkey::new_from_array(SigningKey::from_bytes(&[83; 32]).verifying_key().to_bytes());
    let next_owner =
        Pubkey::new_from_array(SigningKey::from_bytes(&[84; 32]).verifying_key().to_bytes());
    for key in [buyer, next_owner] {
        world.replace(key, Account::new(10 * SOL, 0, &Pubkey::default()));
    }
    let expiry = CLOSE - 1;
    let nft = world.mint_request(buyer, 61, SOL, expiry);
    let before = snapshots(&world, &nft);
    world.tx(&world.sign(&nft), true, &[]);
    assert_eq!(world.tokens(nft.token), 1);
    assert_eq!(world.curve().minted_count, 1);
    let issued = snapshots(&world, &nft);
    let next = world.mint_request(buyer, 62, SOL, expiry);
    let second = snapshots(&world, &next);

    world.warp(OPEN + WINDOW);
    let destination = world.destination(nft.mint, next_owner);
    world.tx(&[nft.transfer(destination, buyer)], true, &[]);
    assert_eq!(world.tokens(destination), 1);
    let transferred = snapshots(&world, &nft);
    let transferred_token = snapshot(&world, destination);
    world.tx(
        &[
            token_ix(
                vec![15, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                vec![w(destination), w(nft.mint), s(next_owner)],
            ),
            nft.retire(),
        ],
        true,
        &[],
    );
    assert_eq!(world.tokens(destination), 0);
    let burned = snapshots(&world, &nft);
    let burned_token = snapshot(&world, destination);
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
    let program = built::program_elf();
    let output = format!(
        r#"{{"schema":"poleis.mint-svm-fixture/1","unit":"{UNIT}","buyer":"{buyer}","nextOwner":"{next_owner}","issuer":"{}","completionDigest":"{}","nextCompletionDigest":"{}","currentSlot":"{OPEN}","expirySlot":"{expiry}","maxPriceLamports":"{SOL}","priceLamports":"{START}","nextPriceLamports":"{}","metadata":{{"name":"{}","symbol":"{}","uri":"{}"}},"programElfBase64":"{}","programCodeHash":"{:x}","before":{before},"issued":{issued},"second":{second},"transferred":{transferred},"transferredToken":{transferred_token},"burned":{burned},"burnedToken":{burned_token}}}"#,
        Pubkey::new_from_array(world.issuer.verifying_key().to_bytes()),
        "3d".repeat(32),
        "3e".repeat(32),
        START + STEP,
        META.name,
        META.symbol,
        META.uri,
        B64.encode(&program),
        Sha256::digest(&program)
    );
    std::fs::write(target.join("mint-svm-fixture.json"), output).unwrap();
}
