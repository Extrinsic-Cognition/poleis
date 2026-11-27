//! $WARM IS MINED BY USE, on Mollusk with the real SPL Token and Associated Token programs: the mine opened once by the program's upgrade
//! authority, the mint the program's and nothing minted at its opening; deposits counted on chain, ten dollars the least; prior deposits
//! counted by the authority, each signature once; spends mined by the miner alone, in rising order, never past the clock and never past
//! a flask's deposits (THE BOUND); the law's $WARM to the flask's own account; what is paid back emitted again before any mint; every
//! refusal named, and nothing of a refused instruction left behind. THE BOOK (tests/common) is checked after every instruction that lands.
mod built;
mod common;
use common::*;
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;
use warm_mine::error::WarmError;
use warm_mine::law::{halving_at, warm_nanos};

/// The live router's opening, Nov 15 2026 00:00 UTC: the clock (his word, Nov 21 17:34:37) stands six days after it.
const OPENING: u64 = 1_794_700_800_000;
/// An opening two years and more before the clock, Aug 31 2024 12:00 UTC: its halvings are Feb 28 2025, Aug 31 2025, Feb 28 2026 and Aug
/// 31 2026, all behind the clock, and Feb 28 2027 ahead of it.
const OPENED_LONG_AGO: u64 = 1_725_105_600_000;

fn spend(flask: Pubkey, usd: u64, at: u64, seq: u64) -> Spend {
    Spend { flask, usd, at, seq }
}

/// A mine opened for `polis` at `opening`, its miner named, and flasks holding `usd` each that deposited it all.
fn mining(opening: u64, deposits: &[u64]) -> (World, Pubkey, Pubkey, Vec<Pubkey>) {
    let polis = Pubkey::new_unique();
    let mut w = World::opened(WORD_S, &polis, opening);
    let miner = w.wallet(10 * SOL);
    w.ok(&set_miner(polis, &miner), "the authority names the miner");
    let (treasury, dollar) = (w.treasury, w.dollar);
    let mut flasks = Vec::new();
    for &usd in deposits {
        let (f, source) = w.flask(usd);
        w.ok(&deposit(f, f, &source, &treasury, &dollar, usd), "a flask deposits");
        flasks.push(f);
    }
    (w, polis, miner, flasks)
}

#[test]
fn init_makes_the_mint_the_programs_own_and_mints_nothing() {
    let mut w = World::new(WORD_S);
    let polis = Pubkey::new_unique();
    let (deployer, treasury, dollar) = (w.deployer, w.treasury, w.dollar);
    w.ok(&init(deployer, deployer, &polis, OPENING, &treasury, &dollar), "the deployer opens the mine");
    assert_eq!(
        w.mine(),
        MineView {
            authority: polis,
            treasury,
            dollar,
            miner: Pubkey::default(),
            mint: mint_pda(),
            vault: vault_pda(),
            opening_ms: OPENING,
            last_seq: 0,
            last_at_ms: 0,
            spends: 0,
            deposited: 0,
            prior: 0,
            spent: 0,
            minted: 0,
            from_vault: 0,
            paid_back: 0,
            vault_balance: 0
        }
    );
    assert_eq!(w.account(&mint_pda()).owner, TOKEN, "the mint is the token program's account");
    assert_eq!(
        mint_view(&w.data(&mint_pda())),
        MintView { authority: Some(mine_pda()), supply: 0, decimals: 9, initialized: true, freeze: None },
        "its minting authority is the program's PDA, nothing is minted, nine decimals, no freeze authority"
    );
    assert_eq!(token_view(&w.data(&vault_pda())), TokenView { mint: mint_pda(), owner: mine_pda(), amount: 0, state: 1 }, "the vault: the mine's, empty");
    let warm_accounts = w.accounts.iter().filter(|(_, a)| a.owner == TOKEN && a.data.len() == 165 && a.data[..32] == mint_pda().to_bytes()).count();
    assert_eq!(warm_accounts, 1, "no account of $WARM but the vault: no premine, no sale, no share");
}

#[test]
fn only_the_upgrade_authority_opens_the_mine_and_only_once() {
    let mut w = World::new(WORD_S);
    let polis = Pubkey::new_unique();
    let (deployer, treasury, dollar) = (w.deployer, w.treasury, w.dollar);
    let stranger = w.wallet(10 * SOL);
    w.refuse(&init(stranger, stranger, &polis, OPENING, &treasury, &dollar), code(WarmError::Unauthorized), "a stranger opens nothing");
    w.refuse(&init(stranger, deployer, &polis, OPENING, &treasury, &dollar), code(WarmError::Unauthorized), "nor with the deployer paying");
    let mut unsigned = init(deployer, deployer, &polis, OPENING, &treasury, &dollar);
    unsigned.accounts[0].is_signer = false;
    unsigned.accounts[1] = solana_instruction::AccountMeta::new(stranger, true);
    w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "the upgrade authority must sign");
    // program data at any other address, or of another owner, is not the program's
    let fake = Pubkey::new_unique();
    w.put_owned(fake, LOADER_V3, program_data_bytes(Some(&stranger)));
    let mut elsewhere = init(stranger, stranger, &polis, OPENING, &treasury, &dollar);
    elsewhere.accounts[2].pubkey = fake;
    w.refuse(&elsewhere, code(WarmError::InvalidAccount), "a program data the program did not deploy with");
    let real = w.account(&program_data());
    w.put_owned(program_data(), Pubkey::new_unique(), real.data.clone());
    w.refuse(&init(deployer, deployer, &polis, OPENING, &treasury, &dollar), code(WarmError::InvalidAccount), "a program data the loader does not own");
    // a program whose upgrades were turned off names no authority: none opens it
    w.put_owned(program_data(), LOADER_V3, program_data_bytes(None));
    w.refuse(&init(deployer, deployer, &polis, OPENING, &treasury, &dollar), code(WarmError::Unauthorized), "no upgrade authority, no opening");
    w.put(program_data(), real);
    // the treasury must hold a dollar: a mint of six decimals
    let nine = w.lay_mint(None, 9);
    let not_usdc = w.lay_token(&nine, &Pubkey::new_unique(), 0);
    w.refuse(&init(deployer, deployer, &polis, OPENING, &not_usdc, &nine), code(WarmError::NotADollar), "a treasury of a mint of nine decimals");
    w.refuse(&init(deployer, deployer, &polis, OPENING, &not_usdc, &dollar), code(WarmError::NotADollar), "a treasury of another mint than the dollar");
    w.refuse(&init(deployer, deployer, &Pubkey::default(), OPENING, &treasury, &dollar), code(WarmError::InvalidAccount), "an authority of nobody");
    // the deployer opens it; then nobody, the deployer included, opens it again
    w.ok(&init(deployer, deployer, &polis, OPENING, &treasury, &dollar), "the deployer opens the mine");
    let opened = w.mine();
    w.refuse(&init(deployer, deployer, &polis, OPENING + 1, &treasury, &dollar), code(WarmError::MineStands), "a second init");
    w.refuse(&init(stranger, stranger, &stranger, 0, &treasury, &dollar), code(WarmError::Unauthorized), "a stranger's second init");
    assert_eq!(w.mine(), opened);
}

#[test]
fn the_authority_alone_moves_the_treasury_and_names_the_miner() {
    let polis = Pubkey::new_unique();
    let mut w = World::opened(WORD_S, &polis, OPENING);
    let (stranger, miner, deployer) = (w.wallet(SOL), w.wallet(SOL), w.deployer);
    let dollar = w.dollar;
    let second = w.lay_token(&dollar, &Pubkey::new_unique(), 0);
    w.refuse(&set_treasury(stranger, &second), code(WarmError::Unauthorized), "a stranger moves no treasury");
    w.refuse(&set_treasury(deployer, &second), code(WarmError::Unauthorized), "nor does the upgrade authority: it opened the mine and holds no role in it");
    w.refuse(&set_miner(stranger, &stranger), code(WarmError::Unauthorized), "a stranger names no miner");
    w.refuse(&set_miner(deployer, &deployer), code(WarmError::Unauthorized), "nor the upgrade authority");
    let mut unsigned = set_miner(polis, &miner);
    unsigned.accounts[0].is_signer = false;
    w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "the authority signs its word");
    let other = w.lay_mint(None, 6);
    let not_the_dollar = w.lay_token(&other, &Pubkey::new_unique(), 0);
    w.refuse(&set_treasury(polis, &not_the_dollar), code(WarmError::NotADollar), "a treasury of another dollar");
    let before = w.mine();
    w.ok(&set_treasury(polis, &second), "the authority moves the treasury");
    w.ok(&set_miner(polis, &miner), "the authority names the miner");
    let renamed = w.wallet(SOL);
    w.ok(&set_miner(polis, &renamed), "and renames it");
    let after = w.mine();
    assert_eq!((after.treasury, after.miner), (second, renamed));
    assert_eq!((after.authority, after.opening_ms, after.dollar, after.mint), (before.authority, before.opening_ms, before.dollar, before.mint), "the authority, the opening and the dollar never move");
    // a deposit now lands in the treasury named now
    let treasury = w.mine().treasury;
    let flask = w.flask(20 * USD);
    w.ok(&deposit(flask.0, flask.0, &flask.1, &treasury, &dollar, 20 * USD), "a deposit to the new treasury");
    assert_eq!(w.tokens(&second), 20 * USD);
    let first = w.treasury;
    w.refuse(&deposit(flask.0, flask.0, &flask.1, &first, &dollar, 10 * USD), code(WarmError::InvalidAccount), "never to the treasury it left");
}

#[test]
fn deposits_are_counted_on_chain_and_ten_dollars_is_the_least() {
    let polis = Pubkey::new_unique();
    let mut w = World::opened(WORD_S, &polis, OPENING);
    let (treasury, dollar) = (w.treasury, w.dollar);
    let (flask, source) = w.flask(100 * USD);
    let relayer = w.wallet(SOL);
    w.refuse(&deposit(flask, flask, &source, &treasury, &dollar, 10 * USD - 1), code(WarmError::UnderMinimum), "a deposit under 10 USDC");
    let mut unsigned = deposit(flask, relayer, &source, &treasury, &dollar, 10 * USD);
    unsigned.accounts[0].is_signer = false;
    w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "the flask's key signs its deposit");
    w.ok(&deposit(flask, flask, &source, &treasury, &dollar, 10 * USD), "10 USDC, the least, its first: the record is made");
    let r = w.record(&flask);
    assert_eq!((r.flask, r.warm_account, r.deposited, r.spent, r.mined, r.paid_back, r.priors), (flask, ata(&flask, &mint_pda()), 10 * USD, 0, 0, 0, 0));
    let rent = w.lamports(&record_pda(&flask));
    let relayer_before = w.lamports(&relayer);
    w.ok(&deposit(flask, relayer, &source, &treasury, &dollar, 15 * USD + 7), "any payer: the house's relay");
    assert_eq!(w.lamports(&relayer), relayer_before, "a standing record costs the payer nothing");
    assert_eq!((w.record(&flask).deposited, w.mine().deposited), (25 * USD + 7, 25 * USD + 7));
    assert_eq!((w.tokens(&source), w.tokens(&treasury)), (75 * USD - 7, 1_000 * USD + 25 * USD + 7), "the flask's USDC is in the treasury");
    assert!(rent > 0);
    // only from the flask's own USDC, only of the dollar, only to the treasury
    let (other, other_source) = w.flask(50 * USD);
    w.refuse(&deposit(other, other, &source, &treasury, &dollar, 10 * USD), code(WarmError::InvalidAccount), "a flask deposits only its own USDC");
    let other_dollar = w.lay_mint(None, 6);
    let not_usdc = w.lay_token(&other_dollar, &other, 50 * USD);
    w.refuse(&deposit(other, other, &not_usdc, &treasury, &dollar, 10 * USD), code(WarmError::InvalidAccount), "a source of another mint");
    let elsewhere = w.lay_token(&dollar, &Pubkey::new_unique(), 0);
    w.refuse(&deposit(other, other, &other_source, &elsewhere, &dollar, 10 * USD), code(WarmError::InvalidAccount), "a deposit lands only in the treasury");
    w.refuse(&deposit(other, other, &other_source, &treasury, &other_dollar, 10 * USD), code(WarmError::InvalidAccount), "the dollar is the mine's");
    w.refuse(&deposit(other, other, &other_source, &treasury, &dollar, 51 * USD), ProgramError::Custom(1), "the token program's own refusal: not that much USDC");
    // a treasury a flask's key held would let it count its own dollars from the treasury to itself: a deposit never comes from the treasury
    // (who holds the treasury is the authority's to name: the polis's own vault)
    let held = Pubkey::new_unique();
    w.put_owned(held, TOKEN, token_bytes(&dollar, &other, 0));
    w.ok(&set_treasury(polis, &held), "the authority names an account the flask holds, by mistake");
    w.refuse(&deposit(other, other, &held, &held, &dollar, 10 * USD), code(WarmError::InvalidAccount), "a deposit from the treasury to itself");
}

#[test]
fn a_prior_deposit_is_counted_by_the_authority_once_and_for_one_flask() {
    let polis = Pubkey::new_unique();
    let mut w = World::opened(WORD_S, &polis, OPENING);
    let (flask, other, stranger, payer) = (Pubkey::new_unique(), Pubkey::new_unique(), w.wallet(SOL), w.wallet(SOL));
    let (s1, s2) = ([0x51u8; 64], [0x52u8; 64]);
    w.refuse(&record_prior(stranger, stranger, &flask, &s1, 12 * USD), code(WarmError::Unauthorized), "a stranger counts no prior deposit");
    w.refuse(&record_prior(polis, payer, &flask, &s1, 0), code(WarmError::InvalidAmount), "a prior deposit of nothing");
    w.ok(&record_prior(polis, payer, &flask, &s1, 12 * USD), "the authority counts his pre-deposit, by its signature");
    let p = prior_view(&w.data(&prior_pda(&s1)));
    assert_eq!((p.signature, p.flask, p.usd, p.slot), (s1, flask, 12 * USD, w.m.sysvars.clock.slot));
    let r = w.record(&flask);
    assert_eq!((r.deposited, r.priors, r.warm_account), (12 * USD, 1, ata(&flask, &mint_pda())), "a flask that never deposited through the program has its record now");
    w.refuse(&record_prior(polis, payer, &flask, &s1, 12 * USD), code(WarmError::PriorCounted), "the same signature twice");
    w.refuse(&record_prior(polis, payer, &other, &s1, 12 * USD), code(WarmError::PriorCounted), "nor for another flask");
    w.ok(&record_prior(polis, payer, &flask, &s2, 3 * USD + 1), "another deposit made through today's door");
    assert_eq!((w.record(&flask).deposited, w.record(&flask).priors), (15 * USD + 1, 2));
    assert_eq!((w.mine().deposited, w.mine().prior), (15 * USD + 1, 15 * USD + 1));
    assert_eq!(w.tokens(&w.treasury.clone()), 1_000 * USD, "a prior deposit moves nothing: it reached the treasury before");
}

#[test]
fn mining_is_by_the_miner_in_rising_order_and_never_past_the_clock() {
    let (mut w, polis, miner, flasks) = mining(OPENING, &[50 * USD, 20 * USD]);
    let (a, b) = (flasks[0], flasks[1]);
    let now = w.now_ms();
    let stranger = w.wallet(SOL);
    w.refuse(&mine(stranger, &[spend(a, USD, now, 1)]), code(WarmError::Unauthorized), "a stranger mines nothing");
    w.refuse(&mine(polis, &[spend(a, USD, now, 1)]), code(WarmError::Unauthorized), "nor does the authority: the miner mines");
    let mut unsigned = mine(miner, &[spend(a, USD, now, 1)]);
    unsigned.accounts[0].is_signer = false;
    w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "the miner signs");
    w.refuse(&mine(miner, &[spend(a, USD, now + 1, 1)]), code(WarmError::Future), "a spend later than the chain's clock");
    w.refuse(&mine(miner, &[spend(a, 0, now, 1)]), code(WarmError::InvalidAmount), "a spend of nothing");
    w.refuse(&mine(miner, &[spend(a, USD, now - DAY, 0)]), code(WarmError::SeqNotRising), "a seq of zero is under the first");
    w.ok(&mine(miner, &[spend(a, USD, now - DAY, 1), spend(b, 2 * USD, now - DAY, 2), spend(a, 3 * USD, now - HOUR, 5)]), "three spends, two flasks, seqs rising with gaps");
    let m = w.mine();
    assert_eq!((m.last_seq, m.last_at_ms, m.spends, m.spent), (5, now - HOUR, 3, 6 * USD));
    w.refuse(&mine(miner, &[spend(a, USD, now, 5)]), code(WarmError::SeqNotRising), "a seq mined already");
    w.refuse(&mine(miner, &[spend(a, USD, now, 4)]), code(WarmError::SeqNotRising), "a seq under the last");
    w.refuse(&mine(miner, &[spend(a, USD, now - HOUR - 1, 6)]), code(WarmError::Backwards), "a spend before the last spend mined");
    w.refuse(&mine(miner, &[spend(a, USD, now, 6), spend(b, USD, now, 6)]), code(WarmError::SeqNotRising), "within a batch too");
    w.refuse(&mine(miner, &[spend(a, USD, now, 6), spend(b, USD, now - 1, 7)]), code(WarmError::Backwards), "within a batch too");
    w.ok(&mine(miner, &[spend(a, USD, now - HOUR, 6), spend(b, USD, now, 7)]), "the same instant as the last, and the clock's own");
    // a spend names its flask's own record and its flask's own $WARM account, and a flask with a record
    let nobody = Pubkey::new_unique();
    w.refuse(&mine(miner, &[spend(nobody, USD, now, 8)]), code(WarmError::InvalidAccount), "a flask with no record has no deposits");
    let mut swapped = mine(miner, &[spend(a, USD, now, 8)]);
    swapped.accounts[8].pubkey = record_pda(&b);
    w.refuse(&swapped, code(WarmError::InvalidAccount), "another flask's record");
    let mut elsewhere = mine(miner, &[spend(a, USD, now, 8)]);
    elsewhere.accounts[9].pubkey = ata(&stranger, &mint_pda());
    w.refuse(&elsewhere, code(WarmError::InvalidAccount), "an account that is not the flask's own");
    let mut wrong_vault = mine(miner, &[spend(a, USD, now, 8)]);
    wrong_vault.accounts[3].pubkey = Pubkey::new_unique();
    w.refuse(&wrong_vault, code(WarmError::InvalidAccount), "the vault is the mine's");
    // one refusal refuses the whole batch: the good spend before it is not mined either
    w.refuse(&mine(miner, &[spend(a, USD, now, 8), spend(b, USD, now + SECOND, 9)]), code(WarmError::Future), "a batch with one spend too late");
    assert_eq!(w.mine().last_seq, 7);
    // and the miner renamed mines no more under its old key
    let next = w.wallet(SOL);
    w.ok(&set_miner(polis, &next), "the authority renames the miner");
    w.refuse(&mine(miner, &[spend(a, USD, now, 8)]), code(WarmError::Unauthorized), "the old crank key");
    w.ok(&mine(next, &[spend(a, USD, now, 8)]), "the new one");
}

#[test]
fn nothing_is_mined_before_a_miner_is_named() {
    let polis = Pubkey::new_unique();
    let mut w = World::opened(WORD_S, &polis, OPENING);
    let (treasury, dollar) = (w.treasury, w.dollar);
    let (f, source) = w.flask(10 * USD);
    w.ok(&deposit(f, f, &source, &treasury, &dollar, 10 * USD), "a deposit");
    let now = w.now_ms();
    for key in [polis, f, w.deployer] {
        w.refuse(&mine(key, &[spend(f, USD, now, 1)]), code(WarmError::Unauthorized), "no miner is named");
    }
}

#[test]
fn the_bound_no_key_mines_a_dollar_no_flask_put_in() {
    let (mut w, polis, miner, flasks) = mining(OPENING, &[10 * USD, 30 * USD]);
    let (a, b) = (flasks[0], flasks[1]);
    let now = w.now_ms();
    w.ok(&mine(miner, &[spend(a, 4 * USD, now - DAY, 1), spend(a, 6 * USD, now - DAY, 2)]), "a's ten dollars, spent whole");
    assert_eq!((w.record(&a).spent, w.record(&a).deposited), (10 * USD, 10 * USD));
    w.refuse(&mine(miner, &[spend(a, 1, now, 3)]), code(WarmError::PastDeposits), "one micro-dollar past a's deposits");
    w.refuse(&mine(miner, &[spend(b, USD, now, 3), spend(a, 1, now, 4)]), code(WarmError::PastDeposits), "and the batch it stands in: b's spend is not mined");
    assert_eq!(w.record(&b).spent, 0);
    w.refuse(&mine(miner, &[spend(b, 30 * USD + 1, now, 3)]), code(WarmError::PastDeposits), "nor one spend past them");
    w.refuse(&mine(miner, &[spend(b, u64::MAX, now, 3)]), code(WarmError::PastDeposits), "nor a spend no deposit could hold");
    w.ok(&mine(miner, &[spend(b, 30 * USD, now, 3)]), "b's thirty, whole");
    w.refuse(&mine(miner, &[spend(b, u64::MAX, now, 4)]), code(WarmError::MathOverflow), "a spend no integer holds beside what was spent");
    // a deposit, or a prior deposit, is what opens more room, to the micro
    w.ok(&record_prior(polis, miner, &a, &[0xa1; 64], 2 * USD), "the authority counts a's pre-deposit");
    w.refuse(&mine(miner, &[spend(a, 2 * USD + 1, now, 4)]), code(WarmError::PastDeposits), "past it by a micro");
    w.ok(&mine(miner, &[spend(a, 2 * USD, now, 4)]), "to it");
    assert_eq!(w.mine().spent, w.mine().deposited, "every dollar put in, mined; not one more");
}

#[test]
fn the_law_a_dollar_mines_one_warm_halved_every_six_calendar_months() {
    let (mut w, _, miner, flasks) = mining(OPENED_LONG_AGO, &[100 * USD]);
    let f = flasks[0];
    let o = OPENED_LONG_AGO;
    let h: Vec<u64> = (1..=5).map(|n| halving_at(o, n).unwrap()).collect();
    assert!(h[3] < w.now_ms() && w.now_ms() < h[4], "four halvings behind the clock, the fifth ahead");
    let at = [o - 30 * DAY, o, h[0] - 1, h[0], h[1], h[2], h[3], w.now_ms()];
    let want = [1_000_000_000u64, 1_000_000_000, 1_000_000_000, 500_000_000, 250_000_000, 125_000_000, 62_500_000, 62_500_000];
    let miner_before = w.lamports(&miner);
    for (k, (&t, &warm)) in at.iter().zip(&want).enumerate() {
        let before = w.warm_of(&f);
        w.ok(&mine(miner, &[spend(f, USD, t, k as u64 + 1)]), "a dollar spent");
        assert_eq!(w.warm_of(&f) - before, warm, "a dollar at {t} (the {k}-th instant) mines {warm} nanos");
        assert_eq!(warm as u128, warm_nanos(USD, t, o));
    }
    assert!(w.lamports(&miner) < miner_before, "the miner paid the flask's $WARM account's rent, once");
    assert_eq!(token_view(&w.data(&ata(&f, &mint_pda()))).owner, f, "the $WARM is in the flask's own account");
    // a micro mines a thousand nanos at the opening's rate and a fraction of a nano at the fourth halving, which the chain does not mint
    w.ok(&mine(miner, &[spend(f, 1, w.now_ms(), 100)]), "a micro-dollar at the fourth halving");
    assert_eq!(w.record(&f).mined, want.iter().sum::<u64>() + (1_000 >> 4));
}

#[test]
fn a_flask_s_warm_account_made_by_another_is_used_as_it_stands() {
    let (mut w, _, miner, flasks) = mining(OPENING, &[10 * USD]);
    let f = flasks[0];
    let mint = mint_pda();
    w.lay_token(&mint, &f, 0);
    let now = w.now_ms();
    let miner_before = w.lamports(&miner);
    w.ok(&mine(miner, &[spend(f, 2 * USD, now, 1)]), "a spend to a $WARM account that stands");
    assert_eq!((w.warm_of(&f), w.lamports(&miner)), (2 * WARM, miner_before), "no rent paid: nothing made");
}

#[test]
fn what_returns_is_emitted_again_from_the_vault_before_any_mint() {
    let (mut w, _, miner, flasks) = mining(OPENING, &[10 * USD, 10 * USD]);
    let (a, b) = (flasks[0], flasks[1]);
    let now = w.now_ms();
    w.ok(&mine(miner, &[spend(a, 4 * USD, now - DAY, 1)]), "a spends four dollars");
    assert_eq!((w.warm_of(&a), w.mine().minted), (4 * WARM, 4 * WARM));
    let a_warm = ata(&a, &mint_pda());
    let stranger = w.wallet(SOL);
    // nobody but the flask moves its $WARM, and only into the vault
    w.refuse(&pay_back(stranger, stranger, &a_warm, WARM), code(WarmError::InvalidAccount), "a stranger moves no flask's $WARM");
    let mut unsigned = pay_back(a, stranger, &a_warm, WARM);
    unsigned.accounts[0].is_signer = false;
    w.refuse(&unsigned, ProgramError::MissingRequiredSignature, "the flask's key signs its pay-back");
    let mut elsewhere = pay_back(a, a, &a_warm, WARM);
    elsewhere.accounts[5].pubkey = ata(&stranger, &mint_pda());
    w.refuse(&elsewhere, code(WarmError::InvalidAccount), "a pay-back lands only in the vault");
    w.refuse(&pay_back(a, a, &a_warm, 0), code(WarmError::InvalidAmount), "a pay-back of nothing");
    w.refuse(&pay_back(a, a, &a_warm, 4 * WARM + 1), ProgramError::Custom(1), "the token program's own refusal: not that much $WARM");
    w.ok(&pay_back(a, a, &a_warm, 3 * WARM / 2), "a drink paid in $WARM: one and a half back to the vault");
    let m = w.mine();
    assert_eq!((m.paid_back, m.vault_balance, m.minted, w.tokens(&vault_pda()), w.record(&a).paid_back), (3 * WARM / 2, 3 * WARM / 2, 4 * WARM, 3 * WARM / 2, 3 * WARM / 2));
    // the next spend is paid from the vault, whole: nothing minted
    w.ok(&mine(miner, &[spend(b, USD, now - HOUR, 2)]), "b spends a dollar");
    let m = w.mine();
    assert_eq!((w.warm_of(&b), m.minted, m.from_vault, m.vault_balance), (WARM, 4 * WARM, WARM, WARM / 2), "one $WARM out of the vault, none minted");
    // the next one empties the vault and mints only the rest
    w.ok(&mine(miner, &[spend(b, 2 * USD, now, 3)]), "b spends two");
    let m = w.mine();
    assert_eq!((w.warm_of(&b), m.minted, m.from_vault, m.vault_balance), (3 * WARM, 4 * WARM + 3 * WARM / 2, 3 * WARM / 2, 0), "half a $WARM from the vault, one and a half minted");
    assert_eq!(mint_view(&w.data(&mint_pda())).supply, 4 * WARM + 3 * WARM / 2);
    // $WARM that changed hands is paid back by its holder's key: a flask with no record pays back too, its record made, its payer paying
    let c = w.wallet(SOL);
    let (b_warm, c_warm) = (ata(&b, &mint_pda()), ata(&c, &mint_pda()));
    w.put_owned(c_warm, TOKEN, token_bytes(&mint_pda(), &c, 0));
    let give = spl_token_interface::instruction::transfer(&TOKEN, &b_warm, &c_warm, &b, &[], WARM).unwrap();
    w.ok(&give, "b gives c one $WARM, by the token program alone");
    w.ok(&pay_back(c, c, &c_warm, WARM), "c pays it back: its record is made");
    let r = w.record(&c);
    assert_eq!((r.deposited, r.paid_back, w.mine().vault_balance), (0, WARM, WARM));
    w.refuse(&mine(miner, &[spend(c, 1, now, 4)]), code(WarmError::PastDeposits), "a record made by a pay-back holds no dollar to mine");
}

#[test]
fn spent_as_credits_it_returns_by_the_flasks_own_approval_and_nothing_twice() {
    // his press of Nov 23 2026 18:17:41 UTC, "One $WARM, the token: spending it as credits returns it, to be mined again": a flask keeps its
    // $WARM; what its tank drank as credits comes back, by the miner, the mine signing as the delegate the flask approved
    let (mut w, _, miner, flasks) = mining(OPENING, &[10 * USD, 10 * USD]);
    let (a, b) = (flasks[0], flasks[1]);
    let now = w.now_ms();
    w.ok(&mine(miner, &[spend(a, 4 * USD, now - DAY, 1)]), "a spends four dollars and mines four $WARM");
    let stranger = w.wallet(SOL);
    w.refuse(&take_back(miner, &[(a, WARM)]), ProgramError::Custom(4), "no approval: the token program refuses, nothing moves (OwnerMismatch: the mine is not its delegate)");
    w.ok(&approve(&a, &mine_pda(), u64::MAX), "a's one approval: the mine its delegate, for all of it");
    assert_eq!(w.warm_of(&a), 4 * WARM, "the approval moves nothing: a keeps its $WARM");
    w.refuse(&take_back(stranger, &[(a, WARM)]), code(WarmError::Unauthorized), "a stranger takes nothing back");
    let mut elsewhere = take_back(miner, &[(a, WARM)]);
    elsewhere.accounts[2].pubkey = ata(&stranger, &mint_pda());
    w.refuse(&elsewhere, code(WarmError::InvalidAccount), "it lands only in the vault");
    let mut another = take_back(miner, &[(a, WARM)]);
    another.accounts[6].pubkey = ata(&b, &mint_pda());
    w.refuse(&another, code(WarmError::InvalidAccount), "from the record's own flask's account alone");
    // its tank drank one and a half $WARM as credits: that comes back, and the rest stays a's
    w.ok(&take_back(miner, &[(a, 3 * WARM / 2)]), "what a's tank drank comes back");
    let m = w.mine();
    assert_eq!((w.warm_of(&a), w.record(&a).paid_back, m.paid_back, m.vault_balance, w.tokens(&vault_pda()), m.minted), (5 * WARM / 2, 3 * WARM / 2, 3 * WARM / 2, 3 * WARM / 2, 3 * WARM / 2, 4 * WARM));
    w.ok(&take_back(miner, &[(a, 3 * WARM / 2)]), "the same batch again");
    w.ok(&take_back(miner, &[(a, WARM)]), "a target the record has passed");
    assert_eq!((w.warm_of(&a), w.record(&a).paid_back), (5 * WARM / 2, 3 * WARM / 2), "a target, never an amount: nothing moves twice");
    w.refuse(&take_back(miner, &[(a, 5 * WARM)]), ProgramError::Custom(1), "never past what a holds: the token program refuses the whole batch");
    // a delegation of less than it holds bounds what can come back
    w.ok(&mine(miner, &[spend(b, 2 * USD, now - DAY + 1, 2)]), "b mines two: the vault's one and a half first, half a $WARM minted");
    assert_eq!((w.mine().vault_balance, w.mine().minted), (0, 4 * WARM + WARM / 2));
    w.ok(&approve(&b, &mine_pda(), WARM / 2), "b approves half a $WARM");
    w.refuse(&take_back(miner, &[(b, WARM)]), ProgramError::Custom(1), "past b's delegation: refused (InsufficientFunds on the delegated amount)");
    w.ok(&take_back(miner, &[(b, WARM / 2), (a, 2 * WARM)]), "two flasks in one batch");
    assert_eq!((w.warm_of(&b), w.warm_of(&a), w.mine().vault_balance, w.record(&a).paid_back), (3 * WARM / 2, 2 * WARM, WARM, 2 * WARM));
    // and what came back pays the next spends before anything is minted
    let minted = w.mine().minted;
    w.ok(&mine(miner, &[spend(b, USD, now, 3)]), "b spends a dollar");
    assert_eq!((w.mine().minted, w.mine().vault_balance, w.warm_of(&b)), (minted, 0, 5 * WARM / 2), "a $WARM out of the vault, none minted");
}

#[test]
fn a_batch_carries_many_spends_and_every_flask_s_account_is_made_once() {
    let deposits = [10 * USD; 6];
    let (mut w, _, miner, flasks) = mining(OPENING, &deposits);
    let now = w.now_ms();
    let spends: Vec<Spend> = flasks.iter().enumerate().map(|(k, f)| spend(*f, USD + k as u64, now - DAY + k as u64, k as u64 + 1)).collect();
    w.ok(&mine(miner, &spends), "six flasks, six spends, six accounts made");
    let again: Vec<Spend> = flasks.iter().enumerate().map(|(k, f)| spend(*f, USD, now, 10 + k as u64)).collect();
    w.ok(&mine(miner, &again), "six more, the accounts standing");
    let same: Vec<Spend> = (0..6).map(|k| spend(flasks[0], USD, now, 20 + k)).collect();
    w.ok(&mine(miner, &same), "one flask's six spends in one batch: its record carries each");
    assert_eq!(w.record(&flasks[0]).spent, 8 * USD);
    for (k, f) in flasks.iter().enumerate() {
        assert_eq!(w.warm_of(f), w.record(f).mined, "flask {k}: its account holds what its spends mined");
    }
}

/// THE BOOK over a seeded sweep: flasks deposit and are counted priors, the miner mines spends at instants that cross four halvings, and
/// flasks pay $WARM back, with refusals in between (a spend past the deposits, a seq that does not rise, a spend before the last, after
/// the clock); after every instruction that lands the book is read (tests/common: `book`), and at the end the $WARM issued is the law's
/// over every spend, to the nano.
#[test]
fn the_book_holds_over_a_seeded_sweep() {
    let polis = Pubkey::new_unique();
    let start_s = (OPENING / 1_000) as i64 - 30 * 86_400;
    let mut w = World::opened(start_s, &polis, OPENING);
    let miner = w.wallet(100 * SOL);
    w.ok(&set_miner(polis, &miner), "the miner");
    let (treasury, dollar) = (w.treasury, w.dollar);
    let mut rng = Rng(0x5eed_2026_1121);
    let mut flasks: Vec<(Pubkey, Pubkey)> = Vec::new();
    for _ in 0..8 {
        flasks.push(w.flask(10_000 * USD));
    }
    let (mut seq, mut last_at, mut landed, mut refused, mut priors) = (0u64, 0u64, 0usize, 0usize, 0u8);
    for step in 0..400 {
        // the clock moves on, across the halvings of two years
        let s = w.m.sysvars.clock.unix_timestamp + rng.below(4 * 86_400) as i64;
        w.set_clock_s(s);
        let now = w.now_ms();
        let (f, source) = flasks[rng.below(flasks.len() as u64) as usize];
        match rng.below(10) {
            0..=1 => {
                let usd = 10 * USD + rng.below(90 * USD);
                if w.tokens(&source) >= usd {
                    w.ok(&deposit(f, miner, &source, &treasury, &dollar, usd), "a deposit");
                    landed += 1;
                }
            }
            2 => {
                priors += 1;
                w.ok(&record_prior(polis, miner, &f, &[priors; 64], 1 + rng.below(50 * USD)), "a prior deposit");
                landed += 1;
            }
            3 => {
                let held = w.warm_of(&f);
                if held > 0 {
                    w.ok(&pay_back(f, f, &ata(&f, &mint_pda()), 1 + rng.below(held)), "a pay-back");
                    landed += 1;
                }
            }
            _ => {
                let n = 1 + rng.below(4) as usize;
                let mut batch = Vec::new();
                let mut room: std::collections::BTreeMap<Pubkey, u64> = std::collections::BTreeMap::new();
                for _ in 0..n {
                    let (g, _) = flasks[rng.below(flasks.len() as u64) as usize];
                    let left = room.entry(g).or_insert_with(|| if w.data(&record_pda(&g)).is_empty() { 0 } else { w.record(&g).deposited - w.record(&g).spent });
                    if *left == 0 {
                        continue;
                    }
                    let usd = 1 + rng.below((*left).min(20 * USD));
                    *left -= usd;
                    seq += 1 + rng.below(3);
                    // an instant between the last spend mined (or a day before the opening) and the clock
                    let lo = last_at.max((OPENING - DAY).min(now));
                    last_at = lo + rng.below(now - lo + 1);
                    batch.push(spend(g, usd, last_at, seq));
                }
                if batch.is_empty() {
                    continue;
                }
                // now and then a batch carries one spend it must not: it is refused whole, and the world stands as it stood
                match step % 9 {
                    0 => {
                        let mut bad = batch.clone();
                        let g = bad[0].flask;
                        let over = w.record(&g).deposited - w.record(&g).spent + 1;
                        bad.push(spend(g, over, last_at, seq + 1));
                        w.refuse(&mine(miner, &bad), code(WarmError::PastDeposits), "one spend past a flask's deposits");
                        refused += 1;
                    }
                    1 => {
                        let mut bad = batch.clone();
                        bad.push(spend(bad[0].flask, 1, last_at, seq));
                        w.refuse(&mine(miner, &bad), code(WarmError::SeqNotRising), "a seq that does not rise");
                        refused += 1;
                    }
                    2 if last_at > 0 => {
                        let mut bad = batch.clone();
                        bad.push(spend(bad[0].flask, 1, last_at - 1, seq + 1));
                        w.refuse(&mine(miner, &bad), code(WarmError::Backwards), "a spend before the last");
                        refused += 1;
                    }
                    3 => {
                        let mut bad = batch.clone();
                        bad.push(spend(bad[0].flask, 1, now + SECOND, seq + 1));
                        w.refuse(&mine(miner, &bad), code(WarmError::Future), "a spend after the clock");
                        refused += 1;
                    }
                    _ => {}
                }
                w.ok(&mine(miner, &batch), "a batch of spends");
                landed += 1;
            }
        }
    }
    let m = w.mine();
    let eras = w.spent.iter().map(|s| warm_mine::law::era_at(s.2, OPENING)).max().unwrap_or(0);
    let law: u128 = w.spent.iter().map(|s| s.3 as u128).sum();
    assert_eq!(m.minted as u128 + m.from_vault as u128, law, "the $WARM issued is the law's over every spend");
    assert!(eras >= 3 && m.from_vault > 0 && m.prior > 0 && refused >= 20, "the sweep crossed {eras} halvings, re-emitted {} nanos, counted {} in priors, refused {refused}", m.from_vault, m.prior);
    eprintln!(
        "sweep: {landed} instructions landed, {refused} refused; {} spends over {eras} halvings; deposited {} micro-dollars ({} of them priors), spent {}; issued {law} nanos: minted {}, from the vault {}; paid back {}",
        w.spent.len(),
        m.deposited,
        m.prior,
        m.spent,
        m.minted,
        m.from_vault,
        m.paid_back
    );
}
