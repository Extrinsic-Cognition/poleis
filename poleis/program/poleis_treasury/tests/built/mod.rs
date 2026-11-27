//! The program the tests load, built from this tree once per test binary, before the first test reads it (THE TESTS BUILD WHAT THEY LOAD, Nov 3 2026).
//! Until then every test needed a `cargo-build-sbf` run by hand first, and the lander's crate gate, which runs `cargo test` alone, read all nine targets
//! red for a missing file ("[MOLLUSK]: Program file not found: poleis_treasury.so", THE-CRATES-ARE-GATED-LANDING.md). Built here, the bytes a test runs
//! are always this tree's: a stale `.so` from another commit can no longer pass for the code under test.
#![allow(dead_code)]
use std::{path::PathBuf, process::Command, sync::{Mutex, OnceLock}};

/// One build at a time: two tests that load two programs (THE TWO PROGRAMS IN ONE WORLD, THE ROUTER DRAWS ITS MONTH) ask the roads and the warm
/// program at once, and on Nov 24 2026 both builds failed side by side in one run of the suite; each now waits for the other here.
static BUILDING: Mutex<()> = Mutex::new(());

/// The folder the built program is in: `SBF_OUT_DIR` for Mollusk, and where a test reads the ELF itself.
pub fn program_dir() -> String {
    static DIR: OnceLock<String> = OnceLock::new();
    DIR.get_or_init(|| {
        let krate = env!("CARGO_MANIFEST_DIR");
        let out = format!("{krate}/target/deploy");
        let _one = BUILDING.lock().unwrap_or_else(|e| e.into_inner());
        let tool = builder().expect("cargo-build-sbf: not on PATH and no Solana release under ~/.local/share/solana/install");
        let status = Command::new(&tool)
            .args(["--offline", "--manifest-path", &format!("{krate}/Cargo.toml"), "--sbf-out-dir", &out])
            .env_remove("RUSTC")
            .env_remove("RUSTC_WRAPPER")
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env_remove("CARGO_BUILD_TARGET")
            .status()
            .unwrap_or_else(|e| panic!("{}: {e}", tool.display()));
        assert!(status.success(), "{} failed: {status}", tool.display());
        // the deploy keypair the build tool writes beside it is no key of the program's (its id is its source's): removed, unread
        let _ = std::fs::remove_file(format!("{out}/poleis_treasury-keypair.json"));
        out
    })
    .clone()
}

/// The folder the roads program (`../poleis_roads`, the market beside Poleis) is built into for THE TWO PROGRAMS IN ONE WORLD: built from
/// this tree once per test binary as Poleis is, into a folder of its own, by its own manifest (no Cargo dependency, no lock change). The
/// build tool writes a deploy keypair beside it whenever none is there; no key of that program is kept by a test: it is removed, unread.
pub fn roads_program_dir() -> String {
    static DIR: OnceLock<String> = OnceLock::new();
    DIR.get_or_init(|| {
        let krate = env!("CARGO_MANIFEST_DIR");
        let out = format!("{krate}/target/deploy-roads");
        let _one = BUILDING.lock().unwrap_or_else(|e| e.into_inner());
        let tool = builder().expect("cargo-build-sbf: not on PATH and no Solana release under ~/.local/share/solana/install");
        let status = Command::new(&tool)
            .args(["--offline", "--manifest-path", &format!("{krate}/../poleis_roads/Cargo.toml"), "--sbf-out-dir", &out])
            .env_remove("RUSTC")
            .env_remove("RUSTC_WRAPPER")
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env_remove("CARGO_BUILD_TARGET")
            .status()
            .unwrap_or_else(|e| panic!("{}: {e}", tool.display()));
        assert!(status.success(), "{} failed: {status}", tool.display());
        let _ = std::fs::remove_file(format!("{out}/poleis_roads-keypair.json"));
        out
    })
    .clone()
}

/// The folder the warm program (`../warm_mine`, $WARM's mint and its mining by the Space Router's law) is built into for the third
/// program of THE TWO PROGRAMS IN ONE WORLD: built from this tree once per test binary as the roads program is, into a folder of its own,
/// by its own manifest (no Cargo dependency, no lock change). Its deploy keypair, written by the build tool, is removed unread.
pub fn warm_program_dir() -> String {
    static DIR: OnceLock<String> = OnceLock::new();
    DIR.get_or_init(|| {
        let krate = env!("CARGO_MANIFEST_DIR");
        let out = format!("{krate}/target/deploy-warm");
        let _one = BUILDING.lock().unwrap_or_else(|e| e.into_inner());
        let tool = builder().expect("cargo-build-sbf: not on PATH and no Solana release under ~/.local/share/solana/install");
        let status = Command::new(&tool)
            .args(["--offline", "--manifest-path", &format!("{krate}/../warm_mine/Cargo.toml"), "--sbf-out-dir", &out])
            .env_remove("RUSTC")
            .env_remove("RUSTC_WRAPPER")
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env_remove("CARGO_BUILD_TARGET")
            .status()
            .unwrap_or_else(|e| panic!("{}: {e}", tool.display()));
        assert!(status.success(), "{} failed: {status}", tool.display());
        let _ = std::fs::remove_file(format!("{out}/warm_mine-keypair.json"));
        out
    })
    .clone()
}

/// The built program's bytes, for a test that deploys them itself.
pub fn program_elf() -> Vec<u8> {
    std::fs::read(format!("{}/poleis_treasury.so", program_dir())).expect("the built program")
}

/// `cargo-build-sbf` on PATH, else the active Solana release, else the newest release installed.
fn builder() -> Option<PathBuf> {
    let on_path = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .map(|d| d.join("cargo-build-sbf"))
        .find(|f| f.is_file());
    if on_path.is_some() {
        return on_path;
    }
    let install = PathBuf::from(std::env::var_os("HOME")?).join(".local/share/solana/install");
    let active = install.join("active_release/bin/cargo-build-sbf");
    if active.is_file() {
        return Some(active);
    }
    std::fs::read_dir(install.join("releases"))
        .ok()?
        .flatten()
        .map(|e| e.path().join("solana-release/bin/cargo-build-sbf"))
        .filter(|f| f.is_file())
        .max_by_key(|f| f.metadata().and_then(|m| m.modified()).ok())
}
