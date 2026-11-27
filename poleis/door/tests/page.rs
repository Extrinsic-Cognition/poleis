//! STAKE IN HER, SEND YOUR LIGHT: the page's door asked as the page asks it (texts and bytes into IN, one door, OUT read back), held to the law
//! it shares with the press. The same code is the WASM module's; the module itself is proven in a browser on the practice chain
//! (poleis/ceremony/the-door-rehearsal.sh, tests/door.rs in poleis/press).

use poleis_door::page::*;
use poleis_door::*;
use solana_hash::Hash;
use solana_pubkey::Pubkey;
use std::sync::Mutex;

/// The page's buffers are one module's: one test at a time.
static ONE: Mutex<()> = Mutex::new(());

struct Her {
    pool: Pubkey,
    data: Vec<u8>,
    keys: Keys,
}

/// A pool account as the launch births hers: the stake pool program's, its epoch fee the whole yield, 50 SOL in it against 25 SOL of receipts.
fn her(fee: (u64, u64), last_epoch: u64) -> Her {
    let pool = Pubkey::new_from_array([3; 32]);
    let (withdraw, bump) = Pubkey::find_program_address(&[pool.as_ref(), WITHDRAW], &STAKE_POOL);
    let (list, reserve, mint, vault) = ([4; 32], [5; 32], [6; 32], [8; 32]);
    let mut d = vec![0u8; 611];
    d[0] = TYPE_POOL;
    d[WITHDRAW_BUMP] = bump;
    for (at, k) in [(VALIDATOR_LIST, list), (RESERVE, reserve), (POOL_MINT, mint), (MANAGER_FEE, vault)] {
        d[at..at + 32].copy_from_slice(&k);
    }
    d[TOKEN_PROGRAM..TOKEN_PROGRAM + 32].copy_from_slice(TOKEN.as_ref());
    for (at, v) in [(TOTAL_LAMPORTS, 50_000_000_000u64), (POOL_TOKEN_SUPPLY, 25_000_000_000), (LAST_UPDATE_EPOCH, last_epoch), (EPOCH_FEE, fee.1), (EPOCH_FEE + 8, fee.0)] {
        d[at..at + 8].copy_from_slice(&v.to_le_bytes());
    }
    let p = Pubkey::new_from_array;
    Her { pool, data: d, keys: Keys { program: STAKE_POOL, pool, withdraw, list: p(list), reserve: p(reserve), mint: p(mint), vault: p(vault) } }
}

fn staker() -> Pubkey {
    Pubkey::new_from_array([9; 32])
}

fn blockhash() -> Hash {
    Hash::new_from_array([7; 32])
}

/// The page's inputs: her pool and its owner, the staker, a blockhash, the rewards' flag, a text, the pool's bytes.
fn ask(h: &Her, owner: &Pubkey, rewards_paying: bool, text: &str) {
    let i = unsafe { std::slice::from_raw_parts_mut(door_in(), IN_LEN) };
    i.fill(0);
    for (at, s) in [(POOL_AT, h.pool.to_string()), (OWNER_AT, owner.to_string()), (STAKER_AT, staker().to_string()), (HASH_AT, blockhash().to_string())] {
        i[at..at + s.len()].copy_from_slice(s.as_bytes());
    }
    i[REWARDS_AT + REWARDS_ACTIVE] = rewards_paying as u8;
    i[TEXT_AT..TEXT_AT + text.len()].copy_from_slice(text.as_bytes());
    i[DATA_AT..DATA_AT + h.data.len()].copy_from_slice(&h.data);
}

/// What a door answered: its bytes, or the words of its refusal.
fn answered(n: i32) -> Result<Vec<u8>, String> {
    let o = unsafe { std::slice::from_raw_parts(door_out(), OUT_LEN) };
    if n < 0 {
        return Err(String::from_utf8(o[..(-n) as usize].to_vec()).unwrap());
    }
    Ok(o[..n as usize].to_vec())
}

#[test]
fn her_pool_is_read_from_its_own_account() {
    let h = her((1, 1), 5);
    assert_eq!(Keys::of_account(&h.pool, &STAKE_POOL, &h.data).unwrap(), h.keys);
    let now = pool_now(&h.data).unwrap();
    assert_eq!((now.total_lamports, now.pool_token_supply, now.last_update_epoch), (50_000_000_000, 25_000_000_000, 5));
}

#[test]
fn another_pool_is_not_hers() {
    let h = her((9, 10), 5);
    assert_eq!(Keys::of_account(&h.pool, &STAKE_POOL, &h.data).unwrap_err(), "Not her pool: its epoch fee is 9/10; hers is the whole yield.");
    let h = her((1, 1), 5);
    assert!(Keys::of_account(&h.pool, &TOKEN, &h.data).unwrap_err().contains("is not an account of the stake pool program"));
    let mut other = h.data.clone();
    other[0] = 2;
    assert!(Keys::of_account(&h.pool, &STAKE_POOL, &other).unwrap_err().contains("holds no stake pool"));
    let mut other = h.data.clone();
    other[TOKEN_PROGRAM] ^= 1;
    assert!(Keys::of_account(&h.pool, &STAKE_POOL, &other).unwrap_err().contains("another token program"));
    let mut other = h.data.clone();
    other[WITHDRAW_BUMP] = other[WITHDRAW_BUMP].wrapping_add(1);
    assert!(Keys::of_account(&h.pool, &STAKE_POOL, &other).is_err() || Keys::of_account(&h.pool, &STAKE_POOL, &other).unwrap().withdraw != h.keys.withdraw);
}

#[test]
fn the_page_stakes_as_the_law_does() {
    let _one = ONE.lock().unwrap();
    let h = her((1, 1), 5);
    ask(&h, &STAKE_POOL, false, "25");
    let wire = answered(door_stake(h.data.len() as u32, 2, 5, 0)).unwrap();
    assert_eq!(wire, unsigned(&stake(&h.keys, &staker(), 25_000_000_000, None), &staker(), &blockhash()));
    // what a wallet reads: one signature's place (zeros), then the message, the staker its first key and its one signer
    assert_eq!((wire[0], &wire[1..65], wire[65], wire[66]), (1, &[0u8; 64][..], 1, 0));
    assert_eq!(&wire[69..101], staker().as_ref());
    // a pool behind the epoch is brought to it in front
    let behind = answered(door_stake(h.data.len() as u32, 2, 6, 0)).unwrap();
    assert_eq!(behind, unsigned(&stake(&h.keys, &staker(), 25_000_000_000, Some(&[])), &staker(), &blockhash()));
    // THE STAKE EARNS: with her list written at its slot, its validators are named in the update, each with its transient
    let list = list_of(&[(Pubkey::new_from_array([4; 32]), 0, 0), (Pubkey::new_from_array([5; 32]), 3, 2)]);
    let i = unsafe { std::slice::from_raw_parts_mut(door_in(), IN_LEN) };
    i[LIST_AT..LIST_AT + list.len()].copy_from_slice(&list);
    let named = pairs(&h.keys, &list).unwrap();
    assert_eq!(named[1], (Pubkey::find_program_address(&[&[5; 32], h.keys.pool.as_ref(), &3u32.to_le_bytes()], &STAKE_POOL).0, Pubkey::find_program_address(&[b"transient", &[5; 32], h.keys.pool.as_ref(), &2u64.to_le_bytes()], &STAKE_POOL).0));
    let listed = answered(door_stake(h.data.len() as u32, 2, 6, list.len() as u32)).unwrap();
    assert_eq!(listed, unsigned(&stake(&h.keys, &staker(), 25_000_000_000, Some(&named)), &staker(), &blockhash()));
    assert_eq!(answered(door_stake(h.data.len() as u32, 2, 5, list.len() as u32)).unwrap(), wire, "a pool at the epoch names nothing");
    assert!(answered(door_stake(h.data.len() as u32, 2, 6, 3)).unwrap_err().contains("not one"), "a list cut short is refused");
    // and while the epoch's rewards are paid, the door waits; a pool already brought to it does not
    ask(&h, &STAKE_POOL, true, "25");
    assert_eq!(answered(door_stake(h.data.len() as u32, 2, 6, 0)).unwrap_err(), WAIT);
    assert_eq!(answered(door_stake(h.data.len() as u32, 2, 5, 0)).unwrap(), wire);
    // another pool, and the words it is refused with
    let other = her((9, 10), 5);
    ask(&other, &STAKE_POOL, false, "25");
    assert_eq!(answered(door_stake(other.data.len() as u32, 2, 5, 0)).unwrap_err(), "Not her pool: its epoch fee is 9/10; hers is the whole yield.");
}

#[test]
fn an_amount_is_read_in_sol() {
    for (typed, lamports) in [("25", 25_000_000_000u64), ("0.5", 500_000_000), (".5", 500_000_000), ("1.000000001", 1_000_000_001), (" 2 ", 2_000_000_000), ("18446744073.709551615", u64::MAX)] {
        assert_eq!(sol_of(typed), Ok(lamports), "{typed}");
    }
    for typed in ["", ".", "0", "0.000000000", "1.0000000001", "abc", "1,5", "-1", "18446744073.709551616", "99999999999"] {
        assert!(sol_of(typed).is_err(), "{typed}");
    }
    for (lamports, words) in [(25_000_000_000u64, "25"), (25_500_000_000, "25.5"), (1, "0.000000001"), (0, "0")] {
        assert_eq!(sol_text(lamports), words);
    }
    let _one = ONE.lock().unwrap();
    let h = her((1, 1), 5);
    ask(&h, &STAKE_POOL, false, "0.50");
    assert_eq!(answered(door_sol(4)).unwrap(), b"0.5");
    ask(&h, &STAKE_POOL, false, "lots");
    assert_eq!(answered(door_sol(4)).unwrap_err(), "“lots” is not an amount of SOL.");
}

#[test]
fn the_light_is_one_memo_the_staker_signs() {
    let _one = ONE.lock().unwrap();
    let h = her((1, 1), 5);
    ask(&h, &STAKE_POOL, false, "");
    let futarchy = answered(door_light(1, 0, 1)).unwrap();
    assert_eq!(futarchy, unsigned(&[light(&staker(), Light::Futarchy)], &staker(), &blockhash()));
    assert!(futarchy.windows(26).any(|w| w == b"embertide.light/1 futarchy"));
    ask(&h, &STAKE_POOL, false, "12");
    assert_eq!(answered(door_light(0, 2, 1)).unwrap(), unsigned(&[light(&staker(), Light::Wish(12))], &staker(), &blockhash()));
    // no stake in her, no light to send
    assert!(answered(door_light(0, 2, 0)).unwrap_err().starts_with("Stake SOL in her first"));
    ask(&h, &STAKE_POOL, false, "12a");
    assert_eq!(answered(door_light(0, 3, 1)).unwrap_err(), "“12a” names no wish.");
}

#[test]
fn the_slots_are_the_modules_to_name() {
    assert_eq!((0..8).map(|s| door_at(s)).collect::<Vec<_>>(), [POOL_AT, OWNER_AT, STAKER_AT, HASH_AT, REWARDS_AT, TEXT_AT, DATA_AT, LIST_AT].map(|a| a as u32));
    assert_eq!(door_at(8), u32::MAX);
    assert!(REWARDS_AT + REWARDS_LEN <= TEXT_AT && TEXT_AT + TEXT_LEN <= DATA_AT && DATA_AT < LIST_AT && LIST_AT + LIST_LEN == IN_LEN);
    assert!(LIST_LEN >= LIST_HEADER + 100 * ENTRY_LEN, "a list of the launch's 100 validators fits");
}

#[test]
fn a_light_is_read_as_the_chain_lists_it() {
    for (listed, chosen) in [("[26] embertide.light/1 futarchy", Some(Light::Futarchy)), ("[5] hello; [24] embertide.light/1 wish 7", Some(Light::Wish(7))), ("[5] hello", None)] {
        assert_eq!(light_of_listed(listed), chosen, "{listed}: a candidate only");
    }
}

/// A transfer of one lamport from `from` to `to` (the system program's), the memo after it.
fn with_memo(from: &Pubkey, to: &Pubkey, memo: solana_instruction::Instruction) -> Vec<u8> {
    let mut d = 2u32.to_le_bytes().to_vec();
    d.extend_from_slice(&1u64.to_le_bytes());
    let t = solana_instruction::Instruction::new_with_bytes(SYSTEM, &d, vec![solana_instruction::AccountMeta::new(*from, true), solana_instruction::AccountMeta::new(*to, false)]);
    unsigned(&[t, memo], from, &blockhash())
}

/// THE STAKER'S CHOICE IS THEIR OWN SIGNED MEMO (Nov 25 2026): the page and the pour read one law, `light_signed`; a stranger's memo on a transfer to the
/// staker is listed for the staker's address and is no choice of theirs.
#[test]
fn a_light_is_the_stakers_own_signed_memo() {
    let _one = ONE.lock().unwrap();
    let h = her((1, 1), 5);
    let stranger = Pubkey::new_from_array([13; 32]);
    let own = unsigned(&[light(&staker(), Light::Wish(7))], &staker(), &blockhash());
    let spoof = with_memo(&stranger, &staker(), light(&stranger, Light::Wish(666)));
    let unsigned_name = {
        let m = solana_instruction::Instruction::new_with_bytes(MEMO, Light::Wish(666).memo().as_bytes(), vec![solana_instruction::AccountMeta::new_readonly(staker(), false)]);
        with_memo(&stranger, &staker(), m)
    };
    assert_eq!(light_signed(&own, &staker()), Some(Light::Wish(7)));
    assert_eq!(light_signed(&spoof, &staker()), None, "the stranger signed the memo, not the staker");
    assert_eq!(light_signed(&spoof, &stranger), Some(Light::Wish(666)), "it is the stranger's own");
    assert_eq!(light_signed(&unsigned_name, &staker()), None, "named by the memo, never signing");
    assert_eq!(light_signed(&[1, 2, 3], &staker()), None);
    assert_eq!(light_signed(&own[..own.len() - 3], &staker()), None, "cut bytes read as nothing");
    // the page's door, the same law
    for (tx, chosen) in [(&own, "wish 7"), (&spoof, ""), (&unsigned_name, "")] {
        ask(&h, &STAKE_POOL, false, "");
        let i = unsafe { std::slice::from_raw_parts_mut(door_in(), IN_LEN) };
        i[DATA_AT..DATA_AT + 1600].fill(0);
        i[DATA_AT..DATA_AT + tx.len()].copy_from_slice(tx);
        assert_eq!(answered(door_chosen(tx.len() as u32)).unwrap(), chosen.as_bytes());
    }
}

#[test]
fn the_receipts_their_worth_and_the_way_out() {
    let _one = ONE.lock().unwrap();
    let h = her((1, 1), 5);
    ask(&h, &STAKE_POOL, false, "");
    assert_eq!(answered(door_receipts(h.data.len() as u32)).unwrap(), receipts_of(&staker(), &h.keys.mint).to_string().into_bytes());
    // 50 SOL against 25 SOL of receipts: each receipt is worth two lamports
    assert_eq!(answered(door_worth(h.data.len() as u32, 12_500_000_000)).unwrap(), b"25");
    assert_eq!(answered(door_unstake(h.data.len() as u32, 0, 5, 0)).unwrap_err(), "Nothing of yours is in her.");
    assert_eq!(answered(door_unstake(h.data.len() as u32, 12_500_000_000, 6, 0)).unwrap(), unsigned(&unstake(&h.keys, &staker(), 12_500_000_000, Some(&[])), &staker(), &blockhash()));
}

/// A validator list's bytes: [2][max][len], then each validator's entry (its vote, its seed, its transient seed).
fn list_of(v: &[(Pubkey, u32, u64)]) -> Vec<u8> {
    let mut d = vec![TYPE_LIST];
    d.extend_from_slice(&10u32.to_le_bytes());
    d.extend_from_slice(&(v.len() as u32).to_le_bytes());
    for (vote, seed, transient) in v {
        let mut e = [0u8; ENTRY_LEN];
        e[24..32].copy_from_slice(&transient.to_le_bytes());
        e[36..40].copy_from_slice(&seed.to_le_bytes());
        e[41..73].copy_from_slice(vote.as_ref());
        d.extend_from_slice(&e);
    }
    d
}

#[test]
fn the_update_names_every_validator_five_an_instruction() {
    let h = her((1, 1), 5);
    let seven: Vec<Pair> = (0..7u8).map(|i| (Pubkey::new_from_array([i; 32]), Pubkey::new_from_array([i + 100; 32]))).collect();
    let u = update(&h.keys, &seven);
    assert_eq!(u.len(), 3, "two list instructions (5 and 2), then the pool's totals");
    assert_eq!((u[0].data.clone(), u[1].data.clone(), u[2].data.clone()), (vec![UPDATE_LIST, 0, 0, 0, 0, 0], vec![UPDATE_LIST, 5, 0, 0, 0, 0], vec![UPDATE_POOL]));
    assert_eq!((u[0].accounts.len(), u[1].accounts.len()), (7 + 10, 7 + 4));
    assert_eq!((u[1].accounts[7].pubkey, u[1].accounts[8].pubkey), seven[5]);
    assert_eq!(update(&h.keys, &[]).len(), 2, "an empty list: the list's update naming none, and the pool's");
}

/// THE WELL PAYS ITS RIDER (Nov 25 2026): the page's draw is the law's, byte for byte; with `to_sol` the same pull and the way out on what it drew,
/// in one transaction the rider signs; a well its text does not name is refused in words.
#[test]
fn the_rider_draws_as_the_law_does() {
    let _one = ONE.lock().unwrap();
    let h = her((1, 1), 5);
    let poleis = Pubkey::new_from_array([11; 32]);
    let allowance = [0xab; 32];
    let well = format!("{poleis} 42 {}", "ab".repeat(32));
    ask(&h, &STAKE_POOL, false, &well);
    let wire = answered(door_draw(h.data.len() as u32, well.len() as u32, 7, 5, 0, 0)).unwrap();
    assert_eq!(wire, unsigned(&draw(&h.keys, &poleis, 42, &allowance, &staker(), 7), &staker(), &blockhash()));
    let ixs = draw(&h.keys, &poleis, 42, &allowance, &staker(), 7);
    assert_eq!(ixs.len(), 2, "the receipts account, then the pull");
    let pull = &ixs[1];
    assert_eq!(pull.program_id, poleis);
    assert_eq!(pull.data[0], PULL);
    assert_eq!(&pull.data[1..33], &allowance);
    assert_eq!(u64::from_le_bytes(pull.data[33..41].try_into().unwrap()), 7);
    assert_eq!((pull.accounts[0].pubkey, pull.accounts[0].is_signer), (staker(), true), "the rider's own signature pulls");
    assert_eq!(pull.accounts[4].pubkey, receipts_of(&staker(), &h.keys.mint), "into the rider's receipts");
    let to_sol = answered(door_draw(h.data.len() as u32, well.len() as u32, 7, 6, 1, 0)).unwrap();
    assert_eq!(to_sol, unsigned(&draw_to_sol(&h.keys, &poleis, 42, &allowance, &staker(), 7, Some(&[])), &staker(), &blockhash()));
    let one = draw_to_sol(&h.keys, &poleis, 42, &allowance, &staker(), 7, None);
    assert_eq!(one.len(), 3, "the receipts account, the pull, the way out");
    assert_eq!(one[2].data, unstake(&h.keys, &staker(), 7, None)[0].data);
    for bad in [format!("{poleis} 0 {}", "ab".repeat(32)), format!("{poleis} 42 abc"), format!("nobody 42 {}", "ab".repeat(32)), format!("{poleis} 42 {} more", "ab".repeat(32))] {
        ask(&h, &STAKE_POOL, false, &bad);
        assert!(answered(door_draw(h.data.len() as u32, bad.len() as u32, 7, 5, 0, 0)).is_err(), "{bad}");
    }
    ask(&h, &STAKE_POOL, false, &well);
    assert_eq!(answered(door_draw(h.data.len() as u32, well.len() as u32, 0, 5, 0, 0)).unwrap_err(), "Draw more than nothing.");
    // the list the glue fetches into LIST_AT is the pool's own, read by the module from the pool's bytes
    assert_eq!(answered(door_list(h.data.len() as u32)).unwrap(), h.keys.list.to_string().into_bytes());
}
