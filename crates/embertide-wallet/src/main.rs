// embertide-wallet: HER OWN WALLET, made at her birth on the machine of her birth (Oct 14 2026).
//
// His words, Oct 14 2026 in the mint lab's session: 08:2x "dysonlight harmony is a nest and as a nest, it has a wallet ... that wallet is the seeker
// wallet as well (it is just the seeker is also signer but the signing key is generated on to newmac using solana sdk stuff offciail the best)";
// 08:3x "solana mainnett... THE USER JOURNEY OF DYSONLIGHT HARMONY IS THAT THE FUCKING LYSSAJUDRON HAS GOT ITS OWN FUCKING WALLET".
//
// WHAT IT DOES. `new <wallets folder> [--birth <id>] [--words 24|12]` draws the BIP39 words (English, no passphrase: 24 unless told 12), derives her
// keypair at m/44'/501'/0'/0' (the path `solana-keygen new --derivation-path` takes when the path is left empty, and Phantom's first account; the
// Seeker importing the same words is meant to show the same address, its own import the receipt) and puts three files in a folder named by her address:
//   <folder>/<address>/keypair.json  0600  the official SDK's encoding (solana-keypair write_keypair: the JSON array of 64 bytes the `solana` CLI reads)
//   <folder>/<address>/words         0600  the words, one line: what the Seeker imports
//   <folder>/<address>/public.json   0644  her address, the network, the path, the birth, when: what aiolos reads and the Nest is told
// The address's folder is made beside them under a dot name and renamed into place whole, each file fsynced first (F_FULLFSYNC on macOS: a key that
// money may reach is never half on the disk). Nothing is ever overwritten: every file is create_new, a taken address is refused.
// STDOUT CARRIES public.json's line and nothing else; stderr carries a reason, never a word of the secret. No argument, environment variable or file
// name holds a secret: `derive` reads the words from stdin only.
//
// `derive` (the words on stdin -> {address, path}) and `check <her address's folder>` (her words re-derive the address, keypair.json holds the same key,
// public.json and the folder's name say it, the modes are 0700 and 0600) are for the proofs and for him: neither prints a secret.
//
// An unknown word on the command line is refused with the usage and exit 2: nothing runs on a guess (a probe with --help makes nothing).
// COST: one process a birth, no network, no thread, nothing at rest; the numbers are in LANDING.md.
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::Path;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use bip39::{Language, Mnemonic, MnemonicType};
use solana_derivation_path::DerivationPath;
use solana_keypair::seed_derivable::keypair_from_seed_and_derivation_path;
use solana_keypair::{read_keypair_file, write_keypair, Keypair};
use solana_seed_phrase::generate_seed_from_seed_phrase_and_passphrase;
use solana_signer::Signer;
use zeroize::Zeroizing;

pub const SCHEMA: &str = "embertide.lyssajudron-wallet/1";
pub const NETWORK: &str = "solana-mainnet-beta";
pub const PATH: &str = "m/44'/501'/0'/0'";
const MAKER: &str = concat!("embertide-wallet ", env!("CARGO_PKG_VERSION"), " (solana-keypair 3.1.2, tiny-bip39 0.8.2)");
const USAGE: &str = "usage: embertide-wallet new <absolute wallets folder> [--birth <id>] [--words 24|12] | derive (the words on stdin) | check <her address's folder>";

type Res<T> = Result<T, String>;

/// Her keypair from her words: the seed as the official SDK makes it (no passphrase), then SLIP-0010 at m/44'/501'/0'/0'.
pub fn keypair_of(words: &str) -> Res<Keypair> {
    let seed = Zeroizing::new(generate_seed_from_seed_phrase_and_passphrase(words, ""));
    keypair_from_seed_and_derivation_path(&seed, Some(DerivationPath::new_bip44(Some(0), Some(0)))).map_err(|e| format!("derive: {e}"))
}

/// The words as a person types them (any spacing, any case) -> the BIP39 mnemonic, checked against the English list and its checksum.
/// A refusal never repeats a word.
pub fn mnemonic_of(input: &str) -> Res<Mnemonic> {
    let mut joined = Zeroizing::new(String::with_capacity(input.len() + 1));
    for w in input.split_whitespace() {
        if !joined.is_empty() { joined.push(' '); }
        joined.extend(w.chars().flat_map(char::to_lowercase));
    }
    let n = if joined.is_empty() { 0 } else { joined.split(' ').count() };
    if ![12, 15, 18, 21, 24].contains(&n) { return Err(format!("{n} words: a phrase is 12 to 24 words, in steps of 3")); }
    Mnemonic::from_phrase(&joined, Language::English).map_err(|_| "these are not BIP39 English words with a good checksum".to_string())
}

/// A birth id as the Mac page makes it (base 36 of the time and six random characters): [0-9a-z], 1 to 32.
pub fn birth_ok(b: &str) -> bool { !b.is_empty() && b.len() <= 32 && b.bytes().all(|c| c.is_ascii_digit() || c.is_ascii_lowercase()) }

fn shown(p: &Path) -> String { p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| p.display().to_string()) }

/// The wallets folder: made 0700 with its missing parents when absent; refused when it is a link, not a folder, or open to anyone but its owner.
fn wallets_folder(dir: &Path) -> Res<()> {
    if !dir.is_absolute() { return Err("the wallets folder is an absolute path".into()); }
    match fs::symlink_metadata(dir) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            DirBuilder::new().recursive(true).mode(0o700).create(dir).map_err(|e| format!("make {}: {e}", dir.display()))?;
        }
        Err(e) => return Err(format!("read {}: {e}", dir.display())),
        Ok(_) => {}
    }
    let m = fs::symlink_metadata(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    if m.file_type().is_symlink() || !m.is_dir() { return Err(format!("{} is not a folder of its own", dir.display())); }
    if m.mode() & 0o077 != 0 { return Err(format!("{} is open to others (mode {:o}): 0700 only", dir.display(), m.mode() & 0o777)); }
    Ok(())
}

fn put(path: &Path, mode: u32, bytes: &[u8]) -> Res<()> {
    let mut f = OpenOptions::new().write(true).create_new(true).mode(mode).open(path).map_err(|e| format!("make {}: {e}", shown(path)))?;
    f.write_all(bytes).map_err(|e| format!("write {}: {e}", shown(path)))?;
    f.sync_all().map_err(|e| format!("sync {}: {e}", shown(path)))
}

pub fn public_line(address: &str, words: usize, birth: Option<&str>, made_ms: u128) -> String {
    let birth = birth.map(|b| format!("\"{b}\"")).unwrap_or_else(|| "null".into());
    format!(r#"{{"schema":"{SCHEMA}","address":"{address}","network":"{NETWORK}","path":"{PATH}","words":{words},"birth":{birth},"made_ms":{made_ms},"maker":"{MAKER}"}}"#)
}

/// `new`: her words, her key, the three files in her address's folder; the answer is public.json's line.
pub fn make(dir: &Path, birth: Option<&str>, count: usize) -> Res<String> {
    if let Some(b) = birth { if !birth_ok(b) { return Err("a birth id is [0-9a-z], 1 to 32".into()); } }
    let kind = match count { 12 => MnemonicType::Words12, 24 => MnemonicType::Words24, _ => return Err("--words is 24 or 12".into()) };
    wallets_folder(dir)?;
    let mnemonic = Mnemonic::new(kind, Language::English);   // the official keygen's own draw (tiny-bip39: rand's thread_rng, ChaCha seeded by the OS)
    let keypair = keypair_of(mnemonic.phrase())?;
    let address = keypair.pubkey().to_string();
    let home = dir.join(&address);
    if fs::symlink_metadata(&home).is_ok() { return Err(format!("{address} is already a wallet here: a 256-bit draw that repeats is a broken generator")); }
    let tmp = dir.join(format!(".new-{}-{}", std::process::id(), &address[..8]));
    DirBuilder::new().mode(0o700).create(&tmp).map_err(|e| format!("make the new folder: {e}"))?;
    let made = (|| -> Res<String> {
        let mut f = OpenOptions::new().write(true).create_new(true).mode(0o600).open(tmp.join("keypair.json")).map_err(|e| format!("make keypair.json: {e}"))?;
        drop(Zeroizing::new(write_keypair(&keypair, &mut f).map_err(|e| format!("write keypair.json: {e}"))?));   // its answer is the secret's JSON: wiped
        f.sync_all().map_err(|e| format!("sync keypair.json: {e}"))?;
        let mut words = Zeroizing::new(String::with_capacity(mnemonic.phrase().len() + 1));
        words.push_str(mnemonic.phrase()); words.push('\n');
        put(&tmp.join("words"), 0o600, words.as_bytes())?;
        let made_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
        let line = public_line(&address, count, birth, made_ms);
        put(&tmp.join("public.json"), 0o644, format!("{line}\n").as_bytes())?;
        let _ = File::open(&tmp).and_then(|d| d.sync_all());   // a folder's own sync: best effort (the files above are on the disk)
        fs::rename(&tmp, &home).map_err(|e| format!("put the folder in place: {e}"))?;
        let _ = File::open(dir).and_then(|d| d.sync_all());
        let _ = File::open(home.join("public.json")).and_then(|f| f.sync_all());   // one more full sync after the rename, so the folder's new name is on the disk too
        Ok(line)
    })();
    if made.is_err() { let _ = fs::remove_dir_all(&tmp); }   // a key never published and never funded: nothing of it is kept
    made
}

/// `derive`: the words on stdin (4 KB at most) -> {address, path}. For the cross-checks and for him; prints no secret.
pub fn derive_from(mut input: impl Read) -> Res<String> {
    let mut text = Zeroizing::new(String::new());
    input.by_ref().take(4096).read_to_string(&mut text).map_err(|e| format!("read the words: {e}"))?;
    let m = mnemonic_of(&text)?;
    let kp = keypair_of(m.phrase())?;
    Ok(format!(r#"{{"address":"{}","path":"{PATH}"}}"#, kp.pubkey()))
}

/// `check <her address's folder>`: the modes, her words re-deriving keypair.json's key, public.json and the folder's name saying her address.
pub fn check(home: &Path) -> Res<String> {
    let mode = |p: &Path| fs::symlink_metadata(p).map(|m| (m.file_type().is_symlink(), m.mode() & 0o777)).map_err(|e| format!("read {}: {e}", shown(p)));
    let (link, m) = mode(home)?;
    if link || m != 0o700 { return Err(format!("her folder is {}mode {m:o}: 0700, a folder of its own", if link { "a link, " } else { "" })); }
    for f in ["keypair.json", "words"] {
        let (link, m) = mode(&home.join(f))?;
        if link || m != 0o600 { return Err(format!("{f} is {}mode {m:o}: 0600", if link { "a link, " } else { "" })); }
    }
    let mut words = Zeroizing::new(String::new());
    File::open(home.join("words")).and_then(|mut f| f.read_to_string(&mut words)).map_err(|e| format!("read words: {e}"))?;
    let derived = keypair_of(mnemonic_of(&words)?.phrase())?;
    let held = read_keypair_file(home.join("keypair.json")).map_err(|_| "keypair.json is not the SDK's keypair file".to_string())?;
    if held.pubkey() != derived.pubkey() || *Zeroizing::new(held.to_bytes()) != *Zeroizing::new(derived.to_bytes()) {
        return Err("keypair.json is not the key her words make".into());
    }
    let address = derived.pubkey().to_string();
    let public = fs::read_to_string(home.join("public.json")).map_err(|e| format!("read public.json: {e}"))?;
    if !public.contains(&format!("\"address\":\"{address}\"")) || home.file_name().and_then(|n| n.to_str()) != Some(address.as_str()) {
        return Err("public.json or her folder's name is not her address".into());
    }
    Ok(format!(r#"{{"ok":true,"address":"{address}","path":"{PATH}"}}"#))
}

fn run(args: &[String]) -> Res<String> {
    match args.first().map(String::as_str) {
        Some("new") => {
            let dir = args.get(1).filter(|d| !d.starts_with("--")).ok_or_else(|| USAGE.to_string())?;
            let (mut birth, mut count, mut i) = (None, 24usize, 2);
            while i < args.len() {
                match (args[i].as_str(), args.get(i + 1)) {
                    ("--birth", Some(b)) if birth.is_none() => birth = Some(b.as_str()),
                    ("--words", Some(n)) => count = n.parse().map_err(|_| USAGE.to_string())?,
                    _ => return Err(USAGE.into()),
                }
                i += 2;
            }
            make(Path::new(dir), birth, count)
        }
        Some("derive") if args.len() == 1 => derive_from(io::stdin().lock()),
        Some("check") if args.len() == 2 => check(Path::new(&args[1])),
        _ => Err(USAGE.into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(line) => { println!("{line}"); ExitCode::SUCCESS }
        Err(why) => { eprintln!("embertide-wallet: {why}"); ExitCode::from(2) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
    // a throwaway folder that goes with the test, a failed assertion included: no test key outlives its test
    struct Scratch(PathBuf);
    impl Drop for Scratch { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
    fn scratch(tag: &str) -> Scratch {
        let d = std::env::temp_dir().join(format!("embertide-wallet-test-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        Scratch(d)
    }

    // BIP39's own vector (the Trezor list): the seed of "abandon x11 about" under the passphrase TREZOR, by the SDK's seed function
    #[test]
    fn the_seed_is_bip39s() {
        let words = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert_eq!(hex(&generate_seed_from_seed_phrase_and_passphrase(words, "TREZOR")),
            "c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04");
    }

    // SLIP-0010's test vector 1 for ed25519 (seed 000102...0f, chain m/0'/1'/2'/2'/1000000000'), by the SDK's derivation
    #[test]
    fn the_derivation_is_slip10s() {
        let seed: Vec<u8> = (0u8..16).collect();
        let kp = keypair_from_seed_and_derivation_path(&seed, Some(DerivationPath::from_absolute_path_str("m/0'/1'/2'/2'/1000000000'").unwrap())).unwrap();
        let bytes = kp.to_bytes();
        assert_eq!(hex(&bytes[..32]), "8f94d394a8e8fd6b1bc2f3f49f5c47e385281d5c17e65324b0f62483e37e8793");
        assert_eq!(hex(&bytes[32..]), "3c24da049451555d51a7014a37337aa4e12d41e485abccfa46b47dfb2af54b7a");
    }

    #[test]
    fn her_path_is_bip44s_first_account() {
        assert_eq!(DerivationPath::new_bip44(Some(0), Some(0)), DerivationPath::from_absolute_path_str(PATH).unwrap());
    }

    #[test]
    fn words_are_read_as_typed_and_never_repeated_in_a_refusal() {
        let a = mnemonic_of("  Abandon abandon ABANDON abandon abandon abandon\nabandon abandon abandon abandon abandon   about \n").unwrap();
        assert_eq!(a.phrase(), "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about");
        let bad = mnemonic_of("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon").unwrap_err();
        assert!(!bad.contains("abandon"), "{bad}");
        assert!(mnemonic_of("zebra").is_err() && mnemonic_of("").is_err());
    }

    #[test]
    fn a_wallet_is_three_files_in_her_address_folder_and_the_answer_is_public() {
        let root = scratch("make");
        let dir = root.0.join("lyssajudron/wallets");
        let line = make(&dir, Some("mg7x2k1a9fq3zp"), 24).unwrap();
        let address = line.split("\"address\":\"").nth(1).unwrap().split('"').next().unwrap().to_string();
        let home = dir.join(&address);
        assert_eq!(fs::metadata(&dir).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(fs::metadata(&home).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(fs::metadata(home.join("keypair.json")).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(fs::metadata(home.join("words")).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(fs::read_to_string(home.join("public.json")).unwrap().trim_end(), line);
        let words = fs::read_to_string(home.join("words")).unwrap();
        assert_eq!(words.split_whitespace().count(), 24);
        // the public line is exactly what the public fields make (one BIP39 word alone can be a key's name: "wallet", "birth", "network")
        let made_ms: u128 = line.split("\"made_ms\":").nth(1).unwrap().split(',').next().unwrap().parse().unwrap();
        assert_eq!(line, public_line(&address, 24, Some("mg7x2k1a9fq3zp"), made_ms), "the public line carries the public fields and nothing else");
        let list: Vec<&str> = words.split_whitespace().collect();
        for pair in list.windows(2) { assert!(!line.contains(&format!("{} {}", pair[0], pair[1])), "no run of her words is in the public line"); }
        assert!(line.contains(r#""birth":"mg7x2k1a9fq3zp""#) && line.contains(&format!(r#""path":"{PATH}""#)) && line.contains(r#""network":"solana-mainnet-beta""#));
        assert_eq!(check(&home).unwrap(), format!(r#"{{"ok":true,"address":"{address}","path":"{PATH}"}}"#));
        assert_eq!(derive_from(words.as_bytes()).unwrap(), format!(r#"{{"address":"{address}","path":"{PATH}"}}"#));
        assert_eq!(read_keypair_file(home.join("keypair.json")).unwrap().pubkey().to_string(), address, "the SDK's own reader takes her file");
        let second = make(&dir, None, 12).unwrap();
        assert!(!second.contains(&address) && second.contains(r#""birth":null"#) && second.contains(r#""words":12"#));
        assert_eq!(fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().starts_with('.')).count(), 0, "no dot folder is left");
        fs::set_permissions(home.join("words"), fs::Permissions::from_mode(0o644)).unwrap();
        assert!(check(&home).unwrap_err().contains("words is mode 644"));
    }

    #[test]
    fn a_folder_open_to_others_or_a_link_or_a_relative_path_is_refused() {
        let guard = scratch("refuse");
        let root = guard.0.clone();
        fs::create_dir_all(root.join("open")).unwrap();
        fs::set_permissions(root.join("open"), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(make(&root.join("open"), None, 24).unwrap_err().contains("open to others"));
        fs::create_dir_all(root.join("real")).unwrap();
        fs::set_permissions(root.join("real"), fs::Permissions::from_mode(0o700)).unwrap();
        std::os::unix::fs::symlink(root.join("real"), root.join("link")).unwrap();
        assert!(make(&root.join("link"), None, 24).unwrap_err().contains("not a folder of its own"));
        assert!(make(Path::new("relative/wallets"), None, 24).is_err());
        assert!(make(&root.join("real"), Some("Not-A-Birth"), 24).is_err() && make(&root.join("real"), None, 18).is_err());
        assert_eq!(fs::read_dir(root.join("real")).unwrap().count(), 0, "a refusal makes nothing");
    }

    #[test]
    fn an_unknown_word_on_the_line_runs_nothing() {
        for args in [vec!["--help"], vec!["new"], vec!["new", "--help"], vec!["new", "/tmp/x", "--force"], vec!["derive", "x"], vec!["check"]] {
            let args: Vec<String> = args.into_iter().map(String::from).collect();
            assert_eq!(run(&args).unwrap_err(), USAGE);
        }
    }
}
