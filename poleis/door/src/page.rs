//! The page's door: the same law, asked by the Pantheon and a wish's page through one WASM module (`build.sh` writes it to
//! embertide-app/ui/intro/dyson-society/app/wasm/poleis-door.wasm; `her-door.mjs` beside it is the glue). No bindgen, no imports: the page writes
//! its inputs into IN at the slots below, calls one door and reads OUT. A door answers the length of what it wrote, or minus the length of the
//! words that say why it refused; every word a visitor reads of a refusal is written here.

use crate::*;
use core::ptr::addr_of_mut;
use core::str::FromStr;

/// IN: the slots below, then (THE STAKE EARNS) her pool's validator list's bytes at LIST_AT, room for a list of 100 validators (7,309 bytes).
pub const LIST_AT: usize = 2048;
pub const LIST_LEN: usize = 8192;
pub const IN_LEN: usize = LIST_AT + LIST_LEN;
pub const OUT_LEN: usize = 2048;
/// IN's slots: four base58 texts of at most 48 bytes (her pool, its owner, the staker, a recent blockhash), the epoch rewards sysvar's bytes,
/// one text (an amount typed, a unit, a memo read), then the pool account's bytes.
pub const POOL_AT: usize = 0;
pub const OWNER_AT: usize = 48;
pub const STAKER_AT: usize = 96;
pub const HASH_AT: usize = 144;
pub const REWARDS_AT: usize = 192;
pub const REWARDS_LEN: usize = 96;
pub const TEXT_AT: usize = 288;
pub const TEXT_LEN: usize = 160;
pub const DATA_AT: usize = 448;

static mut IN: [u8; IN_LEN] = [0; IN_LEN];
static mut OUT: [u8; OUT_LEN] = [0; OUT_LEN];

fn input() -> &'static [u8] {
    unsafe { &*addr_of_mut!(IN) }
}

fn answer(r: Res<Vec<u8>>) -> i32 {
    let out = unsafe { &mut *addr_of_mut!(OUT) };
    match r {
        Ok(b) if b.len() <= OUT_LEN => {
            out[..b.len()].copy_from_slice(&b);
            b.len() as i32
        }
        Ok(b) => answer(Err(format!("An answer of {} bytes, past the door's {OUT_LEN}.", b.len()))),
        Err(why) => {
            let w = &why.as_bytes()[..why.len().min(OUT_LEN)];
            out[..w.len()].copy_from_slice(w);
            -(w.len().max(1) as i32)
        }
    }
}

fn text(at: usize, len: usize, max: usize) -> Res<&'static str> {
    let s = &input()[at..at + len.min(max)];
    let n = s.iter().position(|&b| b == 0).unwrap_or(s.len());
    core::str::from_utf8(&s[..n]).map_err(|_| "A text that is not UTF-8.".to_string())
}

fn key(at: usize, what: &str) -> Res<Pubkey> {
    Pubkey::from_str(text(at, 48, 48)?).map_err(|_| format!("{what} is not an address."))
}

fn data(len: u32) -> Res<&'static [u8]> {
    let n = len as usize;
    if n > LIST_AT - DATA_AT {
        return Err(format!("A pool account of {n} bytes, past the door's {}.", LIST_AT - DATA_AT));
    }
    Ok(&input()[DATA_AT..DATA_AT + n])
}

/// Her pool, read from the account the page fetched, and the staker.
fn her(data_len: u32) -> Res<(Keys, PoolNow, Pubkey)> {
    let pool = key(POOL_AT, "Her pool")?;
    let d = data(data_len)?;
    Ok((Keys::of_account(&pool, &key(OWNER_AT, "Her pool's owner")?, d)?, pool_now(d)?, key(STAKER_AT, "Your wallet")?))
}

/// Whether the pool must be brought to the epoch first, and its list's validators to name when it must (the list's bytes at LIST_AT, `list_len`
/// of them; none written, none named); while the epoch's rewards are paid it cannot be, and the door waits.
fn behind(k: &Keys, now: &PoolNow, epoch: u64, list_len: u32) -> Res<Option<Vec<Pair>>> {
    if now.last_update_epoch >= epoch {
        return Ok(None);
    }
    if rewards_paying(&input()[REWARDS_AT..REWARDS_AT + REWARDS_LEN]) {
        return Err(WAIT.into());
    }
    let n = list_len as usize;
    if n > LIST_LEN {
        return Err(format!("A validator list of {n} bytes, past the door's {LIST_LEN}."));
    }
    if n == 0 {
        return Ok(Some(Vec::new()));
    }
    pairs(k, &input()[LIST_AT..LIST_AT + n]).map(Some)
}

fn blockhash() -> Res<Hash> {
    Hash::from_str(text(HASH_AT, 48, 48)?).map_err(|_| "The chain's blockhash is not a hash.".to_string())
}

#[no_mangle]
pub extern "C" fn door_in() -> *mut u8 {
    addr_of_mut!(IN) as *mut u8
}

#[no_mangle]
pub extern "C" fn door_out() -> *mut u8 {
    addr_of_mut!(OUT) as *mut u8
}

/// Where IN's slots begin, by their order: her pool, its owner, the staker, a blockhash, the rewards sysvar, the text, the pool's bytes.
#[no_mangle]
pub extern "C" fn door_at(slot: u32) -> u32 {
    [POOL_AT, OWNER_AT, STAKER_AT, HASH_AT, REWARDS_AT, TEXT_AT, DATA_AT, LIST_AT].get(slot as usize).map_or(u32::MAX, |&at| at as u32)
}

/// THE STAKE: the amount typed (the text, in SOL) into her pool, an unsigned transaction the staker's wallet signs. `list_len`: the bytes of
/// her pool's validator list written at LIST_AT (a page that writes none passes nothing, read as 0: no validator named).
#[no_mangle]
pub extern "C" fn door_stake(data_len: u32, text_len: u32, epoch: u64, list_len: u32) -> i32 {
    answer((|| {
        let (k, now, staker) = her(data_len)?;
        let lamports = sol_of(text(TEXT_AT, text_len as usize, TEXT_LEN)?)?;
        Ok(unsigned(&stake(&k, &staker, lamports, behind(&k, &now, epoch, list_len)?.as_deref()), &staker, &blockhash()?))
    })())
}

/// THE WAY OUT: every receipt the staker holds burned, their SOL back.
#[no_mangle]
pub extern "C" fn door_unstake(data_len: u32, receipts: u64, epoch: u64, list_len: u32) -> i32 {
    answer((|| {
        let (k, now, staker) = her(data_len)?;
        if receipts == 0 {
            return Err("Nothing of yours is in her.".into());
        }
        Ok(unsigned(&unstake(&k, &staker, receipts, behind(&k, &now, epoch, list_len)?.as_deref()), &staker, &blockhash()?))
    })())
}

/// THE LIGHT'S CHOICE: the futarchy, or the wish whose unit the text names; one memo the staker signs, a staker who holds receipts of her pool.
#[no_mangle]
pub extern "C" fn door_light(futarchy: u32, text_len: u32, receipts: u64) -> i32 {
    answer((|| {
        let staker = key(STAKER_AT, "Your wallet")?;
        if receipts == 0 {
            return Err("Stake SOL in her first: the light is your stake's, and you have none in her.".into());
        }
        let choice = if futarchy != 0 {
            Light::Futarchy
        } else {
            let unit = text(TEXT_AT, text_len as usize, TEXT_LEN)?;
            light_of(&format!("{LIGHT} wish {unit}")).ok_or_else(|| format!("“{unit}” names no wish."))?
        };
        Ok(unsigned(&[light(&staker, choice)], &staker, &blockhash()?))
    })())
}

/// THE RIDER'S DRAW (THE WELL PAYS ITS RIDER, Nov 25 2026): the text names the well, `<Poleis's id> <the wish's unit> <its allowance's hash, 64 hex>`;
/// the staker's slot holds the rider's wallet. `amount` of her pool's token pulled into the rider's receipts, and with `to_sol` burned again at once
/// for SOL from her reserve at par: one unsigned transaction the rider's wallet signs. `list_len`: her list's bytes at LIST_AT, as the stake's.
#[no_mangle]
pub extern "C" fn door_draw(data_len: u32, text_len: u32, amount: u64, epoch: u64, to_sol: u32, list_len: u32) -> i32 {
    answer((|| {
        let (k, now, rider) = her(data_len)?;
        let words = text(TEXT_AT, text_len as usize, TEXT_LEN)?;
        let mut w = words.split(' ');
        let poleis = w.next().and_then(|s| Pubkey::from_str(s).ok()).ok_or("The well names no program.")?;
        let unit = w.next().and_then(|s| s.parse::<u64>().ok()).filter(|u| *u != 0).ok_or("The well names no wish.")?;
        let hex = w.next().filter(|h| h.len() == 64 && w.next().is_none()).ok_or("The well names no allowance.")?;
        let mut allowance = [0u8; 32];
        for (i, b) in allowance.iter_mut().enumerate() {
            *b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).map_err(|_| "The well names no allowance.".to_string())?;
        }
        if amount == 0 {
            return Err("Draw more than nothing.".into());
        }
        let ixs = if to_sol != 0 { draw_to_sol(&k, &poleis, unit, &allowance, &rider, amount, behind(&k, &now, epoch, list_len)?.as_deref()) } else { draw(&k, &poleis, unit, &allowance, &rider, amount) };
        Ok(unsigned(&ixs, &rider, &blockhash()?))
    })())
}

/// Her pool's validator list's address in base58, read from the pool account: the account the page fetches into LIST_AT, so the stake, the way out
/// and the draw name her validators when the pool is behind the epoch.
#[no_mangle]
pub extern "C" fn door_list(data_len: u32) -> i32 {
    answer(her(data_len).map(|(k, _, _)| k.list.to_string().into_bytes()))
}

/// The staker's receipts account, its address in base58.
#[no_mangle]
pub extern "C" fn door_receipts(data_len: u32) -> i32 {
    answer(her(data_len).map(|(k, _, staker)| receipts_of(&staker, &k.mint).to_string().into_bytes()))
}

/// What a number of receipts is worth in her pool, in SOL as words.
#[no_mangle]
pub extern "C" fn door_worth(data_len: u32, receipts: u64) -> i32 {
    answer(data(data_len).and_then(pool_now).map(|now| sol_text(worth(receipts, &now)).into_bytes()))
}

/// The light the staker chose in one landed transaction (its wire bytes in the data's slot, as getTransaction's `base64` gives them; the staker in
/// theirs): `futarchy`, `wish <unit>`, or nothing, by `light_signed`: a memo counts only in a transaction the staker signed where the memo program
/// names them, so a stranger's memo on a transfer to them is nothing.
#[no_mangle]
pub extern "C" fn door_chosen(tx_len: u32) -> i32 {
    answer((|| {
        let staker = key(STAKER_AT, "Your wallet")?;
        let tx = data(tx_len)?;
        Ok(match light_signed(tx, &staker) {
            Some(Light::Futarchy) => b"futarchy".to_vec(),
            Some(Light::Wish(unit)) => format!("wish {unit}").into_bytes(),
            None => Vec::new(),
        })
    })())
}

/// An amount typed, as the door reads it: the SOL in words, or why it is not one.
#[no_mangle]
pub extern "C" fn door_sol(text_len: u32) -> i32 {
    answer(text(TEXT_AT, text_len as usize, TEXT_LEN).and_then(sol_of).map(|l| sol_text(l).into_bytes()))
}
