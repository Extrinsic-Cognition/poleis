//! The program the tests load, built from this tree once per test binary, before the first test reads it: Poleis's harness
//! (poleis_treasury/tests/built/mod.rs, THE TESTS BUILD WHAT THEY LOAD, Nov 3 2026) for warm_mine.so. Built here, the bytes a test runs
//! are always this tree's: a stale `.so` from another commit can no longer pass for the code under test.
#![allow(dead_code)]
use std::{path::PathBuf, process::Command, sync::OnceLock};

/// The folder the built program is in: `SBF_OUT_DIR` for Mollusk, and where a test reads the ELF itself.
pub fn program_dir() -> String {
    static DIR: OnceLock<String> = OnceLock::new();
    DIR.get_or_init(|| {
        let krate = env!("CARGO_MANIFEST_DIR");
        let out = format!("{krate}/target/deploy");
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
        // cargo-build-sbf writes a deploy keypair beside the program whenever none is there (it has no switch to skip it). No key of this
        // program is kept by a test: it is removed here, unread, at once. The program's id is a placeholder; a deploy is the owner's ceremony.
        let _ = std::fs::remove_file(format!("{out}/warm_mine-keypair.json"));
        out
    })
    .clone()
}

/// The built program's bytes, for a test that deploys them itself.
pub fn program_elf() -> Vec<u8> {
    std::fs::read(format!("{}/warm_mine.so", program_dir())).expect("the built program")
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
