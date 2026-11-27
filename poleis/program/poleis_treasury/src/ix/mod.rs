pub mod auction;
pub mod bls;
pub mod council;
pub mod door;
pub mod economy;
pub mod fund;
pub mod market;
pub mod nft;
pub mod nft_sale;
pub mod proposal;
pub mod sale;
pub mod treasury;
pub mod upgrade;
pub mod vault;

use crate::error::PoleisError;
use crate::state::{Config, Registry, RegistryHead, TokenPlan, Treasury};
use crate::util::{expect_pda, expect_signer, move_lamports, receipt};
use pinocchio::{error::ProgramError, AccountView};

/// The crank's tip for a landed batch or a seal: what the CODEX set, out of
/// the unit's treasury, under the plan. Skipped, never failing, when the
/// plan or the treasury cannot pay: liveness never depends on it.
pub fn tip(
    registry: &mut AccountView,
    treasury: &mut AccountView,
    crank: &mut AccountView,
    unit_id: u64,
    hash: &[u8; 32],
    now: u64,
) -> Result<(), ProgramError> {
    let amount = Registry::head(registry)?.crank_tip;
    if amount == 0 {
        return Ok(());
    }
    read_treasury(treasury, unit_id)?;
    if charge_plan(registry, amount, now).is_err() {
        return Ok(());
    }
    if move_lamports(treasury, crank, amount).is_err() {
        let mut h = Registry::head(registry)?;
        h.plan_spent = h.plan_spent.saturating_sub(amount);
        return Registry::set_head(registry, &h);
    }
    Treasury::add_out(treasury, amount)?;
    receipt(crate::RCPT_TIP, unit_id, hash, crank.address().as_array(), &[0u8; 32], amount, 0);
    Ok(())
}

/// Read and verify a unit's treasury PDA.
pub fn read_treasury(acc: &AccountView, unit_id: u64) -> Result<Treasury, ProgramError> {
    let t = Treasury::read(acc)?;
    if t.unit_id != unit_id {
        return Err(PoleisError::UnitMismatch.into());
    }
    expect_pda(acc, &[b"treasury", &unit_id.to_le_bytes()], t.bump)?;
    Ok(t)
}

/// Read and verify a config PDA: one per tree, seeded by its root unit.
pub fn read_config(config_acc: &AccountView) -> Result<Config, ProgramError> {
    let config = Config::read(config_acc)?;
    expect_pda(config_acc, &[b"config", &config.root.to_le_bytes()], config.bump)?;
    Ok(config)
}

/// The config of one particular tree.
pub fn read_config_of(config_acc: &AccountView, root: u64) -> Result<Config, ProgramError> {
    let config = read_config(config_acc)?;
    if config.root != root {
        return Err(PoleisError::UnitMismatch.into());
    }
    Ok(config)
}

/// Read and verify a unit's registry PDA.
pub fn read_registry(registry_acc: &AccountView, unit_id: u64) -> Result<RegistryHead, ProgramError> {
    let head = Registry::head(registry_acc)?;
    if head.unit_id != unit_id {
        return Err(PoleisError::UnitMismatch.into());
    }
    expect_pda(registry_acc, &[b"registry", &unit_id.to_le_bytes()], head.bump)?;
    Ok(head)
}

/// The solo authority, for the instructions that exist only before a
/// council: creating units by hand, opening a lot auction, opening genesis.
pub fn expect_solo_authority(config_acc: &AccountView, signer: &AccountView) -> Result<Config, ProgramError> {
    expect_signer(signer)?;
    let config = read_config(config_acc)?;
    if config.mode != crate::MODE_SOLO {
        return Err(PoleisError::InvalidMode.into());
    }
    if signer.address() != &config.authority {
        return Err(PoleisError::Unauthorized.into());
    }
    Ok(config)
}

/// The tree must be a council for anything under the double seal.
pub fn expect_council(config_acc: &AccountView, root: u64) -> Result<Config, ProgramError> {
    let config = read_config_of(config_acc, root)?;
    if config.mode != crate::MODE_COUNCIL {
        return Err(PoleisError::InvalidMode.into());
    }
    Ok(config)
}

/// A council of some tree: for execute, whose binding to its tree happened
/// at propose (a session says PASSED only under that tree's council) and
/// whose mode can never regress.
pub fn expect_any_council(config_acc: &AccountView) -> Result<Config, ProgramError> {
    let config = read_config(config_acc)?;
    if config.mode != crate::MODE_COUNCIL {
        return Err(PoleisError::InvalidMode.into());
    }
    Ok(config)
}

/// A member of a unit: a key holding hexagon seats, or a key in a live chair.
pub fn expect_member(registry: &AccountView, signer: &AccountView, now: u64) -> Result<(), ProgramError> {
    expect_signer(signer)?;
    let key: [u8; 32] = *signer.address().as_array();
    if Registry::find_holder(registry, crate::HOLDER_KEY, &key)?.is_some() {
        return Ok(());
    }
    if Registry::chairs_of(registry, &key, now)? != 0 {
        return Ok(());
    }
    Err(PoleisError::NotMember.into())
}

/// Charge an outflow against the unit's spending plan for the current
/// period, rolling the period forward first. The plan is the wall against
/// the drain: whatever the seals said, a breach refuses.
pub fn charge_plan(registry: &mut AccountView, amount: u64, now: u64) -> Result<(), ProgramError> {
    let mut head = Registry::head(registry)?;
    if head.plan_period > 0 && now >= head.plan_start.saturating_add(head.plan_period) {
        let elapsed = now - head.plan_start;
        let periods = elapsed / head.plan_period;
        head.plan_start = head.plan_start.saturating_add(periods.saturating_mul(head.plan_period));
        head.plan_spent = 0;
    }
    let after = head.plan_spent.checked_add(amount).ok_or(PoleisError::MathOverflow)?;
    if after > head.plan_cap {
        return Err(PoleisError::PlanBreached.into());
    }
    head.plan_spent = after;
    Registry::set_head(registry, &head)
}

/// The token side of the plan. The default mint (named at init_config,
/// inherited by every unit: USDC on a real cluster) is charged to the
/// registry's own counter, `token_plan_cap` per period, set by CODEX Plan.
/// Any other mint must have been allowed by CODEX AllowMint and is charged
/// to its own TokenPlan account, in its own units, over the same period.
/// Receiving is always free; spending an unlisted mint refuses
/// `MintNotAllowed`, a breach `TokenPlanBreached`.
pub fn charge_token_plan(
    registry: &mut AccountView,
    mint: &[u8; 32],
    amount: u64,
    now: u64,
    plan: Option<&mut AccountView>,
) -> Result<(), ProgramError> {
    let mut head = Registry::head(registry)?;
    if *mint == head.default_mint && *mint != [0u8; 32] {
        if head.plan_period > 0 && now >= head.plan_start.saturating_add(head.plan_period) {
            let elapsed = now - head.plan_start;
            let periods = elapsed / head.plan_period;
            head.plan_start = head.plan_start.saturating_add(periods.saturating_mul(head.plan_period));
            head.plan_spent = 0;
            head.token_plan_spent = 0;
        }
        let after = head.token_plan_spent.checked_add(amount).ok_or(PoleisError::MathOverflow)?;
        if after > head.token_plan_cap {
            return Err(PoleisError::TokenPlanBreached.into());
        }
        head.token_plan_spent = after;
        return Registry::set_head(registry, &head);
    }
    let plan = plan.ok_or(PoleisError::MintNotAllowed)?;
    let mut tp = TokenPlan::read(plan).map_err(|_| PoleisError::MintNotAllowed)?;
    if tp.unit_id != head.unit_id || tp.mint != *mint {
        return Err(PoleisError::MintNotAllowed.into());
    }
    expect_pda(plan, &[b"tokenplan", &head.unit_id.to_le_bytes(), mint], tp.bump)?;
    if tp.cap == 0 {
        return Err(PoleisError::MintNotAllowed.into());
    }
    if head.plan_period > 0 && now >= tp.start.saturating_add(head.plan_period) {
        let periods = (now - tp.start) / head.plan_period;
        tp.start = tp.start.saturating_add(periods.saturating_mul(head.plan_period));
        tp.spent = 0;
    }
    let after = tp.spent.checked_add(amount).ok_or(PoleisError::MathOverflow)?;
    if after > tp.cap {
        return Err(PoleisError::TokenPlanBreached.into());
    }
    tp.spent = after;
    tp.write(plan)
}
