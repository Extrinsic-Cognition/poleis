//! Account state, laid out byte by byte, as Poleis and the roads lay their own: no macros, the layout you read is the layout on chain.
//! Every account starts with a one-byte discriminator and a one-byte layout version; both, the owner and the size are checked on every
//! read, and a standing account is held to its own PDA by its stored bump. All integers little-endian. Offsets are public so clients and
//! tests read the same numbers the program writes.

use crate::error::WarmError;
use pinocchio::{
    account::{Ref, RefMut},
    error::ProgramError,
    AccountView, Address,
};

/// Bumped whenever any layout below changes shape.
pub const LAYOUT_VERSION: u8 = 1;

pub const DISC_MINE: u8 = 1;
pub const DISC_FLASK: u8 = 2;
pub const DISC_PRIOR: u8 = 3;

pub const OFF_DISC: usize = 0;
pub const OFF_VERSION: usize = 1;

/// The mine, the program's one state and its hand: the $WARM mint's authority and the vault's holder.
pub const SEED_MINE: &[u8] = b"mine";
/// The $WARM mint.
pub const SEED_MINT: &[u8] = b"warm";
/// The vault: the token account that holds the $WARM paid back, until a spend takes it out again.
pub const SEED_VAULT: &[u8] = b"vault";
/// A flask's record: `[SEED_FLASK, flask]`.
pub const SEED_FLASK: &[u8] = b"flask";
/// A prior deposit, by its transaction's signature: `[SEED_PRIOR, signature[..32], signature[32..]]`.
pub const SEED_PRIOR: &[u8] = b"prior";

#[inline(always)]
fn rd_u32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(d[o..o + 4].try_into().unwrap())
}
#[inline(always)]
fn rd_u64(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}
#[inline(always)]
fn rd_addr(d: &[u8], o: usize) -> Address {
    Address::new_from_array(d[o..o + 32].try_into().unwrap())
}
#[inline(always)]
fn wr_u32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
#[inline(always)]
fn wr_u64(d: &mut [u8], o: usize, v: u64) {
    d[o..o + 8].copy_from_slice(&v.to_le_bytes());
}
#[inline(always)]
fn wr_addr(d: &mut [u8], o: usize, v: &Address) {
    d[o..o + 32].copy_from_slice(v.as_ref());
}
#[inline(always)]
fn stamp(d: &mut [u8], disc: u8) {
    d[OFF_DISC] = disc;
    d[OFF_VERSION] = LAYOUT_VERSION;
}

fn checked<'a>(acc: &'a AccountView, disc: u8, size: usize) -> Result<Ref<'a, [u8]>, ProgramError> {
    if !acc.owned_by(&crate::ID) {
        return Err(WarmError::InvalidAccount.into());
    }
    let data = acc.try_borrow()?;
    if data.len() != size || data[OFF_DISC] != disc {
        return Err(WarmError::InvalidAccount.into());
    }
    if data[OFF_VERSION] != LAYOUT_VERSION {
        return Err(WarmError::LayoutVersion.into());
    }
    Ok(data)
}

fn sized<'a>(acc: &'a mut AccountView, size: usize) -> Result<RefMut<'a, [u8]>, ProgramError> {
    let d = acc.try_borrow_mut()?;
    if d.len() != size {
        return Err(WarmError::InvalidAccount.into());
    }
    Ok(d)
}

/// `acc` is the PDA of `seeds` and `bump` under this program: one hash (the chain's sha256 syscall), never a search.
fn expect_at<const N: usize>(acc: &AccountView, seeds: &[&[u8]; N], bump: u8) -> Result<(), ProgramError> {
    if acc.address() != &Address::derive_address(seeds, Some(bump), &crate::ID) {
        return Err(WarmError::InvalidPda.into());
    }
    Ok(())
}

// ---- Mine: [disc:1][ver:1][authority:32][treasury:32][dollar:32][miner:32][mint:32][vault:32][opening_ms:8][last_seq:8][last_at_ms:8]
//            [spends:8][deposited:8][prior:8][spent:8][minted:8][from_vault:8][paid_back:8][vault_balance:8][bump:1][mint_bump:1][vault_bump:1] ----
// One per program, at `[SEED_MINE]`. Its AUTHORITY and its OPENING are named once, at `init`, and nothing writes them after; the TREASURY
// (the USDC account deposits go to) moves only by the authority's `set_treasury`, and only to another account of the same DOLLAR (the
// six-decimal mint `init` read off the treasury); the MINER only by the authority's `set_miner` (none, all zero, until it names one).
// THE BOOK it keeps: `deposited` every micro of USDC counted for the flasks (by `deposit`, and by `record_prior`, `prior` of it), `spent`
// every micro mined against them; `minted` every nano of $WARM the mint ever made, `from_vault` every nano a spend took out of the vault,
// `paid_back` every nano a flask paid into it, and `vault_balance` what stands in it of those (paid_back - from_vault). So the $WARM every
// spend issued is minted + from_vault, and the mint's supply is minted.
pub struct Mine {
    pub authority: Address,
    pub treasury: Address,
    pub dollar: Address,
    pub miner: Address,
    pub mint: Address,
    pub vault: Address,
    pub opening_ms: u64,
    pub last_seq: u64,
    pub last_at_ms: u64,
    pub spends: u64,
    pub deposited: u64,
    pub prior: u64,
    pub spent: u64,
    pub minted: u64,
    pub from_vault: u64,
    pub paid_back: u64,
    pub vault_balance: u64,
    pub bump: u8,
    pub mint_bump: u8,
    pub vault_bump: u8,
}
impl Mine {
    pub const SIZE: usize = 285;
    pub const OFF_AUTHORITY: usize = 2;
    pub const OFF_TREASURY: usize = 34;
    pub const OFF_DOLLAR: usize = 66;
    pub const OFF_MINER: usize = 98;
    pub const OFF_MINT: usize = 130;
    pub const OFF_VAULT: usize = 162;
    pub const OFF_OPENING: usize = 194;
    pub const OFF_LAST_SEQ: usize = 202;
    pub const OFF_LAST_AT: usize = 210;
    pub const OFF_SPENDS: usize = 218;
    pub const OFF_DEPOSITED: usize = 226;
    pub const OFF_PRIOR: usize = 234;
    pub const OFF_SPENT: usize = 242;
    pub const OFF_MINTED: usize = 250;
    pub const OFF_FROM_VAULT: usize = 258;
    pub const OFF_PAID_BACK: usize = 266;
    pub const OFF_VAULT_BALANCE: usize = 274;
    pub const OFF_BUMP: usize = 282;
    pub const OFF_MINT_BUMP: usize = 283;
    pub const OFF_VAULT_BUMP: usize = 284;

    /// The mine, checked: this program's, its discriminator, version and size, and at its own PDA.
    pub fn read_pda(acc: &AccountView) -> Result<Self, ProgramError> {
        let m = {
            let d = checked(acc, DISC_MINE, Self::SIZE)?;
            Self {
                authority: rd_addr(&d, Self::OFF_AUTHORITY),
                treasury: rd_addr(&d, Self::OFF_TREASURY),
                dollar: rd_addr(&d, Self::OFF_DOLLAR),
                miner: rd_addr(&d, Self::OFF_MINER),
                mint: rd_addr(&d, Self::OFF_MINT),
                vault: rd_addr(&d, Self::OFF_VAULT),
                opening_ms: rd_u64(&d, Self::OFF_OPENING),
                last_seq: rd_u64(&d, Self::OFF_LAST_SEQ),
                last_at_ms: rd_u64(&d, Self::OFF_LAST_AT),
                spends: rd_u64(&d, Self::OFF_SPENDS),
                deposited: rd_u64(&d, Self::OFF_DEPOSITED),
                prior: rd_u64(&d, Self::OFF_PRIOR),
                spent: rd_u64(&d, Self::OFF_SPENT),
                minted: rd_u64(&d, Self::OFF_MINTED),
                from_vault: rd_u64(&d, Self::OFF_FROM_VAULT),
                paid_back: rd_u64(&d, Self::OFF_PAID_BACK),
                vault_balance: rd_u64(&d, Self::OFF_VAULT_BALANCE),
                bump: d[Self::OFF_BUMP],
                mint_bump: d[Self::OFF_MINT_BUMP],
                vault_bump: d[Self::OFF_VAULT_BUMP],
            }
        };
        expect_at(acc, &[SEED_MINE], m.bump)?;
        Ok(m)
    }
    /// Write every field: on a freshly made account (stamped here) or one read and checked before.
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = sized(acc, Self::SIZE)?;
        stamp(&mut d, DISC_MINE);
        wr_addr(&mut d, Self::OFF_AUTHORITY, &self.authority);
        wr_addr(&mut d, Self::OFF_TREASURY, &self.treasury);
        wr_addr(&mut d, Self::OFF_DOLLAR, &self.dollar);
        wr_addr(&mut d, Self::OFF_MINER, &self.miner);
        wr_addr(&mut d, Self::OFF_MINT, &self.mint);
        wr_addr(&mut d, Self::OFF_VAULT, &self.vault);
        wr_u64(&mut d, Self::OFF_OPENING, self.opening_ms);
        wr_u64(&mut d, Self::OFF_LAST_SEQ, self.last_seq);
        wr_u64(&mut d, Self::OFF_LAST_AT, self.last_at_ms);
        wr_u64(&mut d, Self::OFF_SPENDS, self.spends);
        wr_u64(&mut d, Self::OFF_DEPOSITED, self.deposited);
        wr_u64(&mut d, Self::OFF_PRIOR, self.prior);
        wr_u64(&mut d, Self::OFF_SPENT, self.spent);
        wr_u64(&mut d, Self::OFF_MINTED, self.minted);
        wr_u64(&mut d, Self::OFF_FROM_VAULT, self.from_vault);
        wr_u64(&mut d, Self::OFF_PAID_BACK, self.paid_back);
        wr_u64(&mut d, Self::OFF_VAULT_BALANCE, self.vault_balance);
        d[Self::OFF_BUMP] = self.bump;
        d[Self::OFF_MINT_BUMP] = self.mint_bump;
        d[Self::OFF_VAULT_BUMP] = self.vault_bump;
        Ok(())
    }
}

// ---- Flask: [disc:1][ver:1][flask:32][warm_account:32][deposited:8][spent:8][mined:8][paid_back:8][priors:4][bump:1] ----
// A flask's record, at `[SEED_FLASK, flask]`, made at its first deposit (or by `record_prior`, or at a pay-back before any), its payer
// paying the rent. `deposited` is every micro of USDC counted for it, `spent` every micro mined against it (THE BOUND: spent never passes
// deposited), `mined` every nano of $WARM its spends issued, `paid_back` every nano it paid into the vault, `priors` how many prior
// deposits the authority recorded for it. `warm_account` is its associated token account for $WARM, found once when the record is made,
// so a spend checks it by comparison, never by a search.
pub struct Flask {
    pub flask: Address,
    pub warm_account: Address,
    pub deposited: u64,
    pub spent: u64,
    pub mined: u64,
    pub paid_back: u64,
    pub priors: u32,
    pub bump: u8,
}
impl Flask {
    pub const SIZE: usize = 103;
    pub const OFF_FLASK: usize = 2;
    pub const OFF_WARM_ACCOUNT: usize = 34;
    pub const OFF_DEPOSITED: usize = 66;
    pub const OFF_SPENT: usize = 74;
    pub const OFF_MINED: usize = 82;
    pub const OFF_PAID_BACK: usize = 90;
    pub const OFF_PRIORS: usize = 98;
    pub const OFF_BUMP: usize = 102;

    /// A flask's record, checked: this program's, its discriminator, version and size, and at its own flask's PDA.
    pub fn read_pda(acc: &AccountView) -> Result<Self, ProgramError> {
        let r = {
            let d = checked(acc, DISC_FLASK, Self::SIZE)?;
            Self {
                flask: rd_addr(&d, Self::OFF_FLASK),
                warm_account: rd_addr(&d, Self::OFF_WARM_ACCOUNT),
                deposited: rd_u64(&d, Self::OFF_DEPOSITED),
                spent: rd_u64(&d, Self::OFF_SPENT),
                mined: rd_u64(&d, Self::OFF_MINED),
                paid_back: rd_u64(&d, Self::OFF_PAID_BACK),
                priors: rd_u32(&d, Self::OFF_PRIORS),
                bump: d[Self::OFF_BUMP],
            }
        };
        expect_at(acc, &[SEED_FLASK, r.flask.as_ref()], r.bump)?;
        Ok(r)
    }
    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = sized(acc, Self::SIZE)?;
        stamp(&mut d, DISC_FLASK);
        wr_addr(&mut d, Self::OFF_FLASK, &self.flask);
        wr_addr(&mut d, Self::OFF_WARM_ACCOUNT, &self.warm_account);
        wr_u64(&mut d, Self::OFF_DEPOSITED, self.deposited);
        wr_u64(&mut d, Self::OFF_SPENT, self.spent);
        wr_u64(&mut d, Self::OFF_MINED, self.mined);
        wr_u64(&mut d, Self::OFF_PAID_BACK, self.paid_back);
        wr_u32(&mut d, Self::OFF_PRIORS, self.priors);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}

// ---- Prior: [disc:1][ver:1][signature:64][flask:32][usd_micros:8][slot:8][bump:1] ----
// A deposit that reached the treasury before this program, counted for a flask by the authority (`record_prior`, a sealed act of the
// polis): the transaction's signature is the account's seeds and its first field, so the seal names its proof and no signature is ever
// counted twice, for that flask or any other. `slot` is when it was counted.
pub struct Prior {
    pub signature: [u8; 64],
    pub flask: Address,
    pub usd_micros: u64,
    pub slot: u64,
    pub bump: u8,
}
impl Prior {
    pub const SIZE: usize = 115;
    pub const OFF_SIGNATURE: usize = 2;
    pub const OFF_FLASK: usize = 66;
    pub const OFF_USD: usize = 98;
    pub const OFF_SLOT: usize = 106;
    pub const OFF_BUMP: usize = 114;

    pub fn write(&self, acc: &mut AccountView) -> Result<(), ProgramError> {
        let mut d = sized(acc, Self::SIZE)?;
        stamp(&mut d, DISC_PRIOR);
        d[Self::OFF_SIGNATURE..Self::OFF_SIGNATURE + 64].copy_from_slice(&self.signature);
        wr_addr(&mut d, Self::OFF_FLASK, &self.flask);
        wr_u64(&mut d, Self::OFF_USD, self.usd_micros);
        wr_u64(&mut d, Self::OFF_SLOT, self.slot);
        d[Self::OFF_BUMP] = self.bump;
        Ok(())
    }
}
