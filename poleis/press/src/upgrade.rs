//! THE PRESS IS READY FOR MAINNET (Nov 24 2026, lane/the-press-is-ready-for-mainnet): a sealed upgrade of the code, carried out by the crank,
//! extends its program's data first when the new build is longer.
//!
//! The press deploys each program at its own length (`deploy_with_max_data_len`, the build's bytes), so the program's data holds that build and
//! not a byte more. The loader's `Upgrade` refuses a buffer whose build is longer than the data holds ("ProgramData account not large enough",
//! AccountDataTooSmall: Agave 4.2.2's loader), and Poleis's kind 6 (`CODEX_UPGRADE`, its own code) and kind 14 (`CODEX_UPGRADE_PROGRAM`, the
//! market's or the $WARM program's) both end in that `Upgrade`. So a sealed upgrade to any longer build, the first one that adds a feature,
//! would pass the whole tree and then be refused at its execute, every time. Devnet's upgrade of Nov 24 2026 needed 98,744 bytes more for
//! Poleis (`ceremony/the-upgrade-rehearsed-on-every-devnet-swarm-2026-11-24.json`), 105,744 with AN OLDER FUND HAS ITS ROAD.
//!
//! The loader's `ExtendProgram` asks no authority (`common_extend_program(.., false)`): anyone pays the rent of the new length. It takes at
//! least 10,240 bytes at a time (SIMD-0431), or exactly what is left under the 10 MiB an account may hold; and it re-deploys the program at the
//! extend's slot, after which the loader refuses an `Upgrade` in that same slot ("Program was deployed in this block already"). So the crank
//! sends the extend in a transaction of its own, the crank its payer, waits for the slot after it, and then the execute.

use crate::acts::Act;
use crate::chain::Sitting;
use crate::loader::{self, BUFFER_HEADER, LOADER_V3, PROGRAMDATA_HEADER};
use crate::rpc::Rpc;
use crate::wire::{self, Key};
use crate::Res;
use poleis_client::{self as client, Payload, CLASS_CODEX, ROOT_UNIT};
use poleis_treasury::{CODEX_UPGRADE, CODEX_UPGRADE_PROGRAM};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use std::time::Duration;

/// The loader's least extension (SIMD-0431: `MINIMUM_EXTEND_PROGRAM_BYTES`).
pub const MIN_EXTEND: u32 = 10_240;
/// The most an account may hold (`MAX_PERMITTED_DATA_LENGTH`, 10 MiB).
pub const MAX_DATA: usize = 10 * 1024 * 1024;

/// The bytes a program's data of `programdata_len` (its whole account, the 45-byte header with it) must grow by to take a build of `build_len`:
/// none when it holds it; else what it lacks, at least MIN_EXTEND, or all that is left under MAX_DATA when that is less. Refused past MAX_DATA.
pub fn extend_by(programdata_len: usize, build_len: usize) -> Res<Option<u32>> {
    let need = PROGRAMDATA_HEADER + build_len;
    if programdata_len >= need {
        return Ok(None);
    }
    if need > MAX_DATA {
        return Err(format!("a build of {build_len} bytes needs a program data of {need}, past the {MAX_DATA} an account may hold"));
    }
    let (short, room) = (need - programdata_len, MAX_DATA - programdata_len);
    let by = short.max(MIN_EXTEND as usize).min(room);
    u32::try_from(by).map(Some).map_err(|_| format!("an extension of {by} bytes"))
}

/// A sealed upgrade as its proposal stores it: which program's code moves, from which buffer, its rent to which spill.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sealed {
    /// CODEX_UPGRADE (Poleis's own code) or CODEX_UPGRADE_PROGRAM (a program the seal holds, named)
    pub kind: u8,
    pub program: Pubkey,
    pub buffer: Pubkey,
    pub spill: Pubkey,
}

impl Sealed {
    /// The payload it was proposed as, of `origin` (the execute's account list is built from it; the chain executes the stored bytes).
    pub fn payload(&self, origin: u64) -> Payload {
        match self.kind {
            CODEX_UPGRADE => Payload::Upgrade { origin, buffer: self.buffer, spill: self.spill },
            _ => Payload::UpgradeProgram { origin, program: self.program, buffer: self.buffer, spill: self.spill },
        }
    }
}

/// A proposal's stored payload read as a sealed upgrade: `[CLASS_CODEX][6][buffer:32][spill:32]`, Poleis's own code (`poleis`, the sitting's
/// id), or `[CLASS_CODEX][14][buffer:32][spill:32][program:32]`. Any other payload is no upgrade.
pub fn sealed_of(payload: &[u8], poleis: &Pubkey) -> Res<Sealed> {
    let key = |at: usize| Pubkey::new_from_array(payload[at..at + 32].try_into().unwrap());
    match payload {
        [CLASS_CODEX, CODEX_UPGRADE, ..] if payload.len() == 66 => Ok(Sealed { kind: CODEX_UPGRADE, program: *poleis, buffer: key(2), spill: key(34) }),
        [CLASS_CODEX, CODEX_UPGRADE_PROGRAM, ..] if payload.len() == 98 => Ok(Sealed { kind: CODEX_UPGRADE_PROGRAM, program: key(66), buffer: key(2), spill: key(34) }),
        _ => Err("its payload is no sealed upgrade of the code (CODEX_UPGRADE, 66 bytes, or CODEX_UPGRADE_PROGRAM, 98 bytes)".into()),
    }
}

/// THE EXTEND, FIRST: what the program's data lacks for the buffer's build, as the chain holds both now (`programdata` and `buffer`, their
/// whole accounts), and the loader's `ExtendProgram` that gives it, `payer` paying its rent; none when the data holds the build.
pub fn extend_for(program: &Pubkey, programdata: &[u8], buffer: &[u8], payer: &Pubkey) -> Res<Option<(u32, Instruction)>> {
    if !matches!(loader::state(programdata)?, loader::State::ProgramData { .. }) {
        return Err(format!("{program}'s program data is not a program data of the loader"));
    }
    if !matches!(loader::state(buffer)?, loader::State::Buffer { .. }) {
        return Err("the upgrade's buffer is not a buffer of the loader".into());
    }
    let build = buffer.len().checked_sub(BUFFER_HEADER).ok_or("the upgrade's buffer is shorter than its header")?;
    Ok(extend_by(programdata.len(), build)?.map(|n| (n, loader::extend_program(program, payer, n))))
}

/// The same, read off the chain: the program's data and the buffer as `rpc` holds them (owned by the loader, or refused).
pub fn extend_on(rpc: &Rpc, program: &Pubkey, buffer: &Pubkey, payer: &Pubkey) -> Res<Option<(u32, Instruction)>> {
    let pd = loader::programdata_of(program);
    let read = |k: &Pubkey, what: &str| -> Res<Vec<u8>> {
        let a = rpc.account(&k.to_string())?.ok_or_else(|| format!("{what} {k} is not on the cluster"))?;
        if a.owner != LOADER_V3.to_string() {
            return Err(format!("{what} {k} is owned by {}, not the loader", a.owner));
        }
        Ok(a.data)
    };
    extend_for(program, &read(&pd, &format!("{program}'s program data"))?, &read(buffer, "the upgrade's buffer")?, payer)
}

/// A sealed upgrade the chain holds (the proposal of `hash`, its payload a CODEX_UPGRADE or a CODEX_UPGRADE_PROGRAM of root 0) as an act the
/// crank carries like the launch's: its number `n`, after the eight, its execute built for the deployer (`Act::execute_by` names the crank).
pub fn act_of(view: &client::ProposalView, hash: [u8; 32], n: usize, poleis: &Pubkey, crank: &Pubkey) -> Res<Act> {
    if view.hash != hash {
        return Err(format!("the proposal at {}'s address holds another hash", wire::hex(&hash)));
    }
    if view.origin != ROOT_UNIT {
        return Err(format!("the upgrade {} is unit {}'s: only root 0 upgrades the code, over the whole tree", &wire::hex(&hash)[..12], view.origin));
    }
    let sealed = sealed_of(&view.payload, poleis).map_err(|e| format!("the proposal {}: {e}", &wire::hex(&hash)[..12]))?;
    let payload = sealed.payload(view.origin);
    if payload.hash() != view.payload_hash || payload.bytes() != view.payload {
        return Err(format!("the proposal {}: its payload does not read back as the upgrade it names", &wire::hex(&hash)[..12]));
    }
    let what = if sealed.kind == CODEX_UPGRADE { "Poleis's own code".to_string() } else { format!("the code of {}", sealed.program) };
    Ok(Act {
        n,
        name: "a sealed upgrade of the code",
        file: "",
        class: CLASS_CODEX,
        payload: view.payload.clone(),
        payload_sha256: view.payload_hash,
        manifest: view.manifest,
        nonce: 0,
        hash,
        impacted: view.impacted.clone(),
        execute: client::execute(*crank, hash, &view.impacted, &payload),
        before: Vec::new(),
        needs: None,
        does: vec![format!("replace {what} from the buffer {}, its rent to {}; the program's data extended first, the crank paying, when the buffer's build is longer than it holds", sealed.buffer, sealed.spill)],
    })
}

impl Sitting {
    /// THE EXTEND, FIRST, on this sitting's chain: before the execute of a sealed upgrade, the program's data grown to take the buffer's build
    /// when it is longer, in a transaction of its own that `crank` signs and pays, then the slot after it waited for (the loader refuses an
    /// upgrade in the slot of an extend). The bytes extended, or none. A second ask once it is extended sends nothing.
    pub fn extend_first(&mut self, sealed: &Sealed, crank: &Key) -> Res<Option<u32>> {
        let Some((n, ix)) = extend_on(&self.rpc, &sealed.program, &sealed.buffer, &crank.address)? else { return Ok(None) };
        let pd = loader::programdata_of(&sealed.program);
        let now = self.account(&pd)?.map(|a| (a.data.len(), a.lamports)).unwrap_or_default();
        let rent = self.rpc.rent(now.0 + n as usize)?.saturating_sub(now.1);
        let what = format!("the program data of {} extended by {n} bytes ({} to {}), its rent {rent} lamports paid by the crank {}, before the sealed upgrade from {}", sealed.program, now.0, now.0 + n as usize, crank.address, sealed.buffer);
        let l = self.send("crank", &what, vec![ix], &[crank])?;
        crate::rpc::wait(Duration::from_secs(60), || Ok((self.rpc.slot()? > l.slot).then_some(())))?;
        Ok(Some(n))
    }
}
