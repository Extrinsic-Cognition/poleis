//! THE BUILD, RECORDED. The sitting's own copy of the programs (Poleis, the market, the $WARM program), their client and the press, laid in
//! the sitting's folder; each `declare_id!` set to its fresh id, and Poleis's `FUND_ADDRESS` and `FUND_BUMP` to that id's `["fund"]` PDA (a
//! fresh id moves three constants of Poleis, not one: THE CEREMONY, REHEARSED, finding 1); each program built as the tests build it (the
//! same `cargo-build-sbf` road as `poleis_treasury/tests/built/mod.rs`, the compiler's own variables cleared), for the architecture a deploy
//! takes (`ARCH`), into folders of its own, and its binary's sha256 recorded; then the sitting's own press built from that copy, so every id, seed and builder it speaks is the sitting's,
//! and held to this press's own derivation of them before anything is sent. The deploy refuses bytes whose hash is not the recorded one.

use crate::Res;
use solana_pubkey::Pubkey;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What is copied, from the house into the sitting's copy, at the same places (the market's program reaches `poleis-futarchy` by
/// `../../../poleis-futarchy`; the mint door's seat law, `poleis/seat-wasm`, speaks the Poleis it is built against: THE FLASK TAKES ITS SEAT; the press itself reads the door's law, `poleis/door`, since STAKE IN HER, SEND YOUR LIGHT,
/// and the stake book's law, `poleis-stake`, since THE LIGHT PAYS A WISH: the first launch rehearsal after it stopped at the sitting's press build without it).
pub const PARTS: [&str; 9] = ["poleis/program/poleis_treasury", "poleis/program/poleis_roads", "poleis/program/warm_mine", "poleis/client", "poleis/door", "poleis/press", "poleis/seat-wasm", "poleis-futarchy", "poleis-stake"];

/// The three programs: their name in the record, their crate folder, the binary the build tool writes, and the folder it goes into (under
/// Poleis's crate, beside the tests' own). The deploy's order: the market and the $WARM program first, Poleis last, so root 0 is founded
/// the slot Poleis becomes callable.
pub const PROGRAMS: [(&str, &str, &str, &str); 3] = [
    ("market", "poleis/program/poleis_roads", "poleis_roads.so", "target/press-v3-roads"),
    ("warm", "poleis/program/warm_mine", "warm_mine.so", "target/press-v3-warm"),
    ("poleis", "poleis/program/poleis_treasury", "poleis_treasury.so", "target/press-v3"),
];

/// The architecture a deploy takes. The tests build for the tool's default, sBPF v0, which a validator loads at genesis and Mollusk runs,
/// and which the loader refuses to deploy under today's features ("Detected sbpf_version required by the executable which are not
/// enabled", the rehearsal's first run, Nov 21 2026; the road of Oct 28: "a validator with current feature gates refuses an sBPF v0
/// binary"): the press builds the same sources, by the tests' own command, for v3 (the Oct 28 ceremony's `--arch v3`), into folders of
/// its own, so a test's build never overwrites the bytes the record names.
pub const ARCH: &str = "v3";

/// The folder a program's binary is built into, in a copy: under Poleis's crate, as the tests build them.
pub fn out_dir(copy: &Path, out: &str) -> PathBuf {
    copy.join("poleis/program/poleis_treasury").join(out)
}

/// Lay `from` over `to`: every file whose bytes differ is written (an unchanged file keeps its time, so a second build of the same tree
/// builds nothing), every file the house no longer has is removed; a `target` folder and a build key are never copied.
pub fn sync(from: &Path, to: &Path) -> Res<usize> {
    let mut written = 0;
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    let mut seen = std::collections::BTreeSet::new();
    for e in std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let e = e.map_err(|e| e.to_string())?;
        let name = e.file_name().to_string_lossy().to_string();
        if name == "target" || name == ".git" || name.ends_with("-keypair.json") {
            continue;
        }
        seen.insert(name.clone());
        let kind = e.file_type().map_err(|e| e.to_string())?;
        let (src, dst) = (e.path(), to.join(&name));
        if kind.is_dir() {
            written += sync(&src, &dst)?;
        } else if kind.is_file() {
            let bytes = std::fs::read(&src).map_err(|e| format!("{}: {e}", src.display()))?;
            if std::fs::read(&dst).ok().as_deref() != Some(&bytes[..]) {
                std::fs::write(&dst, &bytes).map_err(|e| format!("{}: {e}", dst.display()))?;
                written += 1;
            }
        }
    }
    for e in std::fs::read_dir(to).map_err(|e| format!("{}: {e}", to.display()))?.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name != "target" && !seen.contains(&name) {
            let p = e.path();
            let _ = if p.is_dir() { std::fs::remove_dir_all(&p) } else { std::fs::remove_file(&p) };
        }
    }
    Ok(written)
}

/// One `declare_id!("...")` in a source, its id swapped; exactly one, or the copy is refused.
pub fn swap_id(src: &str, id: &Pubkey, what: &str) -> Res<String> {
    let mark = "declare_id!(\"";
    let hits: Vec<usize> = src.match_indices(mark).map(|(i, _)| i).collect();
    if hits.len() != 1 {
        return Err(format!("{what}: {} declare_id! in its lib.rs, one expected", hits.len()));
    }
    let at = hits[0] + mark.len();
    let end = at + src[at..].find('"').ok_or_else(|| format!("{what}: a declare_id! never closed"))?;
    let old = &src[at..end];
    if !(32..=44).contains(&old.len()) || !old.bytes().all(|b| b.is_ascii_alphanumeric() && !b"0OIl".contains(&b)) {
        return Err(format!("{what}: a declare_id! that is not an id: {old:?}"));
    }
    Ok(format!("{}{id}{}", &src[..at], &src[end..]))
}

/// The id a source declares.
pub fn declared(src: &str) -> Res<Pubkey> {
    let mark = "declare_id!(\"";
    let at = src.find(mark).ok_or("no declare_id!")? + mark.len();
    let end = at + src[at..].find('"').ok_or("a declare_id! never closed")?;
    src[at..end].parse().map_err(|_| format!("{:?} is not an id", &src[at..end]))
}

/// Poleis's two fund constants set to `fund` and `bump`: the one span of `FUND_ADDRESS`'s bytes and the one `FUND_BUMP`.
pub fn swap_fund(src: &str, fund: &Pubkey, bump: u8) -> Res<String> {
    let head = "pub const FUND_ADDRESS: Address = Address::new_from_array([";
    if src.matches(head).count() != 1 {
        return Err("Poleis's lib.rs: one FUND_ADDRESS expected".into());
    }
    let at = src.find(head).unwrap() + head.len();
    let end = at + src[at..].find("]);").ok_or("FUND_ADDRESS never closed")?;
    let b = fund.to_bytes();
    let hexes: Vec<String> = b.iter().map(|x| format!("0x{x:02x}")).collect();
    let body = format!("\n    {},\n    {},\n", hexes[..16].join(", "), hexes[16..].join(", "));
    let src = format!("{}{body}{}", &src[..at], &src[end..]);
    let bump_head = "pub const FUND_BUMP: u8 = ";
    if src.matches(bump_head).count() != 1 {
        return Err("Poleis's lib.rs: one FUND_BUMP expected".into());
    }
    let at = src.find(bump_head).unwrap() + bump_head.len();
    let end = at + src[at..].find(';').ok_or("FUND_BUMP never closed")?;
    if src[at..end].parse::<u8>().is_err() {
        return Err("FUND_BUMP is not a number".into());
    }
    Ok(format!("{}{bump}{}", &src[..at], &src[end..]))
}

/// The ids the copy is built for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ids {
    pub poleis: Pubkey,
    pub market: Pubkey,
    pub warm: Pubkey,
}

/// Lay the house's parts over the copy and patch its ids: what was written, and the fund's PDA under the fresh Poleis id.
pub fn copy_and_patch(house: &Path, copy: &Path, ids: &Ids) -> Res<(usize, Pubkey, u8)> {
    let mut written = 0;
    for part in PARTS {
        written += sync(&house.join(part), &copy.join(part))?;
    }
    let (fund, bump) = Pubkey::find_program_address(&[b"fund"], &ids.poleis);
    for (dir, id, what) in [("poleis/program/poleis_treasury", ids.poleis, "Poleis"), ("poleis/program/poleis_roads", ids.market, "the market"), ("poleis/program/warm_mine", ids.warm, "the $WARM program")] {
        let lib = copy.join(dir).join("src/lib.rs");
        let src = std::fs::read_to_string(&lib).map_err(|e| format!("{}: {e}", lib.display()))?;
        let mut out = swap_id(&src, &id, what)?;
        if what == "Poleis" {
            out = swap_fund(&out, &fund, bump)?;
        }
        if out != src {
            std::fs::write(&lib, out).map_err(|e| format!("{}: {e}", lib.display()))?;
        }
    }
    Ok((written, fund, bump))
}

/// `cargo-build-sbf`: CARGO_BUILD_SBF, else on PATH, else the newest Solana release installed that holds it.
pub fn build_tool() -> Res<PathBuf> {
    if let Some(p) = std::env::var_os("CARGO_BUILD_SBF") {
        return Ok(PathBuf::from(p));
    }
    if let Some(p) = std::env::var_os("PATH").into_iter().flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>()).map(|d| d.join("cargo-build-sbf")).find(|f| f.is_file()) {
        return Ok(p);
    }
    let home = std::env::var_os("HOME").ok_or("no HOME")?;
    std::fs::read_dir(PathBuf::from(home).join(".local/share/solana/install/releases"))
        .map_err(|_| "cargo-build-sbf: not on PATH and no Solana release installed".to_string())?
        .flatten()
        .map(|e| e.path().join("solana-release/bin/cargo-build-sbf"))
        .filter(|f| f.is_file())
        .max_by_key(|f| f.metadata().and_then(|m| m.modified()).ok())
        .ok_or_else(|| "cargo-build-sbf: not on PATH and no Solana release installed".into())
}

/// A command with the compiler's own variables cleared, as the tests build, and the 26.5 SDK where it stands (the 27.0 linker refuses
/// the host's build scripts: THE CEREMONY, REHEARSED, finding 8).
fn clean(mut c: Command) -> Command {
    for v in ["RUSTC", "RUSTC_WRAPPER", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_TARGET", "CARGO_TARGET_DIR"] {
        c.env_remove(v);
    }
    let sdk = "/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk";
    if std::env::var_os("SDKROOT").is_none() && Path::new(sdk).is_dir() {
        c.env("SDKROOT", sdk);
    }
    c
}

/// One program built as the tests build it; its build key, written by the tool beside it, removed unread. Its bytes.
pub fn build_program(copy: &Path, krate: &str, so: &str, out: &str) -> Res<Vec<u8>> {
    let tool = build_tool()?;
    let treasury = copy.join("poleis/program/poleis_treasury");
    let dir = out_dir(copy, out);
    let mut c = clean(Command::new(&tool));
    c.current_dir(&treasury).args(["--offline", "--arch", ARCH, "--manifest-path"]).arg(copy.join(krate).join("Cargo.toml")).arg("--sbf-out-dir").arg(&dir);
    let status = c.status().map_err(|e| format!("{}: {e}", tool.display()))?;
    for e in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        if e.file_name().to_string_lossy().ends_with("-keypair.json") {
            let _ = std::fs::remove_file(e.path());
        }
    }
    if !status.success() {
        return Err(format!("{krate}: the build failed ({status})"));
    }
    std::fs::read(dir.join(so)).map_err(|e| format!("{}: {e}", dir.join(so).display()))
}

/// THE FLASK TAKES ITS SEAT (Nov 22 2026): the mint door's seat law, built for wasm32 from the copy by its own build.sh, so the Poleis it speaks is the
/// sitting's (its PROGRAM_ID is baked in at the build): its bytes and their path, the file that lands as the door's worker/poleis-seat.wasm with its vars.
pub fn build_seat(copy: &Path) -> Res<(Vec<u8>, PathBuf)> {
    let krate = copy.join("poleis/seat-wasm");
    let out = krate.join("target/poleis-seat.wasm");
    let mut c = clean(Command::new("sh"));
    c.arg(krate.join("build.sh")).arg(&out);
    let status = c.status().map_err(|e| format!("sh {}: {e}", krate.join("build.sh").display()))?;
    if !status.success() {
        return Err(format!("the door's seat law: the build failed ({status})"));
    }
    Ok((std::fs::read(&out).map_err(|e| format!("{}: {e}", out.display()))?, out))
}

/// The sitting's own press, built from the copy by its own lock, offline: its path.
pub fn build_press(copy: &Path) -> Res<PathBuf> {
    let manifest = copy.join("poleis/press/Cargo.toml");
    let mut c = clean(Command::new("cargo"));
    c.args(["build", "--release", "--offline", "--locked", "--bin", "press", "--manifest-path"]).arg(&manifest);
    let status = c.status().map_err(|e| format!("cargo: {e}"))?;
    if !status.success() {
        return Err(format!("the sitting's press: the build failed ({status})"));
    }
    Ok(copy.join("poleis/press/target/release/press"))
}

/// The ids this binary was built for: the client's Poleis id, the $WARM program's, and the market's as its source declared it, with Poleis's
/// fund constants and their derivation beside them.
pub fn compiled() -> Res<(Ids, Pubkey, u8, Pubkey, u8)> {
    let poleis = poleis_client::PROGRAM_ID;
    let market = declared(include_str!("../../program/poleis_roads/src/lib.rs"))?;
    let warm = Pubkey::new_from_array(warm_mine::ID.to_bytes());
    let (fund, bump) = Pubkey::find_program_address(&[b"fund"], &poleis);
    let fixed = Pubkey::new_from_array(poleis_client::FUND_ADDRESS.to_bytes());
    Ok((Ids { poleis, market, warm }, fund, bump, fixed, poleis_client::FUND_BUMP))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_id_moves_the_three_constants_and_nothing_else() {
        let src = include_str!("../../program/poleis_treasury/src/lib.rs");
        let id = Pubkey::new_from_array([7; 32]);
        let (fund, bump) = Pubkey::find_program_address(&[b"fund"], &id);
        let out = swap_fund(&swap_id(src, &id, "Poleis").unwrap(), &fund, bump).unwrap();
        assert_eq!(declared(&out).unwrap(), id);
        assert!(out.contains(&format!("pub const FUND_BUMP: u8 = {bump};")));
        let b = fund.to_bytes();
        assert!(out.contains(&format!("0x{:02x}, 0x{:02x}", b[0], b[1])) && out.contains(&format!("0x{:02x},\n]);", b[31])), "the fund's bytes, sixteen a line");
        // everything else is the source's own: the diff is the id's line, the fund's two lines and the bump's (when the bump moved)
        let differ = src.lines().zip(out.lines()).filter(|(a, b)| a != b).count();
        let bump_moved = usize::from(bump != poleis_client::FUND_BUMP);
        assert_eq!((src.lines().count(), differ), (out.lines().count(), 3 + bump_moved), "the id, the fund's two rows, the bump: nothing else moves");
        // and the tree's own constants are its placeholder id's fund, so a copy never ships another program's fund
        let (tf, tb) = Pubkey::find_program_address(&[b"fund"], &declared(src).unwrap());
        assert_eq!((tf.to_bytes(), tb), (poleis_client::FUND_ADDRESS.to_bytes(), poleis_client::FUND_BUMP), "the tree's FUND_ADDRESS and FUND_BUMP are its own id's");
    }

    #[test]
    fn a_source_with_two_ids_or_none_is_refused() {
        let one = "x\nsolana_address::declare_id!(\"CmdsHtG8xJxaxi2GTjyeG3ePZrW9PdeYU8ENUoujE2q6\");\n";
        let id = Pubkey::new_from_array([9; 32]);
        assert_eq!(declared(&swap_id(one, &id, "w").unwrap()).unwrap(), id);
        assert!(swap_id(&format!("{one}{one}"), &id, "w").is_err());
        assert!(swap_id("no id here", &id, "w").is_err());
        assert!(swap_id("declare_id!(\"not an id\")", &id, "w").is_err());
    }

    #[test]
    fn the_market_and_warm_sources_declare_one_id_each() {
        for src in [include_str!("../../program/poleis_roads/src/lib.rs"), include_str!("../../program/warm_mine/src/lib.rs")] {
            assert!(swap_id(src, &Pubkey::new_from_array([3; 32]), "a program").is_ok());
        }
        let (ids, fund, bump, fixed, fixed_bump) = compiled().unwrap();
        assert_eq!((fund, bump), (fixed, fixed_bump), "this build's fund constants are its Poleis id's own");
        assert_eq!(ids.warm, declared(include_str!("../../program/warm_mine/src/lib.rs")).unwrap());
    }
}
