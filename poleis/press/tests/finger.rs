//! NO KEY FILE IS READ WITHOUT HIS FINGER. A sitting that reads key files asks the house's Touch ID asker before each read
//! (`chain::finger`: it must exit 0 AND say "ok"), reads only a folder of mode 700 and files of mode 600 that are his user's
//! (`chain::check_private`), and on mainnet hears no asker but the house's own, its bytes the ones the house's commit holds
//! (`chain::asker`). The askers here are stand-ins made in a folder of the test's own: one that says yes, and the ways of not saying it.
//! What this does not close, said in the lane's receipt: a key file is a file, and a process running as his user can read it without
//! the press; the finger guards the press's road, the folder's mode guards the file from other users.
use poleis_press::chain::{asker, check_private, finger, KeySource};
use poleis_press::rpc::Cluster;
use poleis_press::wire::Key;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Dir(PathBuf);
impl Dir {
    fn new(name: &str) -> Self {
        let p = std::env::temp_dir().join(format!("press-finger-{name}-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn put(path: &Path, text: &str, mode: u32) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

/// A stand-in asker: a shell script that says `says` and exits `code`.
fn an_asker(dir: &Path, name: &str, says: &str, code: u8) -> PathBuf {
    let p = dir.join(name);
    put(&p, &format!("#!/bin/sh\nprintf '%s\\n' '{says}'\nexit {code}\n"), 0o755);
    p
}

#[test]
fn his_finger_is_a_yes_only_when_the_asker_exits_well_and_says_ok() {
    let d = Dir::new("asker");
    finger(&an_asker(&d.0, "yes", "ok", 0), "a test").unwrap();
    for (name, says, code) in [("no", "no", 1), ("ok-and-fails", "ok", 1), ("silent", "", 0), ("other-words", "okay", 0), ("cancelled", "cancel", 2)] {
        let e = finger(&an_asker(&d.0, name, says, code), "a test").expect_err(name);
        assert!(e.contains("his finger did not say yes"), "{name}: {e}");
    }
    assert!(finger(&d.0.join("no-such-asker"), "a test").unwrap_err().contains("his finger: the asker"));
}

#[test]
fn a_key_file_is_read_only_after_his_finger_and_only_from_a_private_place() {
    let d = Dir::new("keys");
    let keys = d.0.join("keys");
    std::fs::create_dir_all(&keys).unwrap();
    std::fs::set_permissions(&keys, std::fs::Permissions::from_mode(0o700)).unwrap();
    let k = Key::from_seed(&[7; 32]);
    put(&keys.join("deployer.json"), &k.cli_text(), 0o600);
    let (yes, no) = (an_asker(&d.0, "yes", "ok", 0), an_asker(&d.0, "no", "no", 1));
    let files = |touchid: &Path| KeySource::Files { dir: keys.clone(), touchid: touchid.to_path_buf() };
    // his finger says yes: the key is read, and it is the file's
    let got = files(&yes).load(&["deployer"], "a test").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(got[0].address, k.address);
    // his finger does not: nothing is read (the refusal is the finger's even when the file is no key at all)
    put(&keys.join("deployer.json"), "not a key", 0o600);
    assert!(files(&no).load(&["deployer"], "a test").err().unwrap().contains("his finger did not say yes"));
    assert!(files(&yes).load(&["deployer"], "a test").err().unwrap().contains("a key file is a JSON array"));
    put(&keys.join("deployer.json"), &k.cli_text(), 0o600);
    // a key file others can read, or a folder others can enter, is no key's place
    std::fs::set_permissions(keys.join("deployer.json"), std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(files(&yes).load(&["deployer"], "a test").err().unwrap().contains("is mode 644, not 600"));
    std::fs::set_permissions(keys.join("deployer.json"), std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::set_permissions(&keys, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(files(&yes).load(&["deployer"], "a test").err().unwrap().contains("is mode 755, not 700"));
    assert!(check_private(&keys).is_err() && check_private(&keys.join("deployer.json")).is_ok());
    std::fs::set_permissions(&keys, std::fs::Permissions::from_mode(0o700)).unwrap();
    // a key the sitting names and the folder does not hold
    assert!(files(&yes).load(&["deployer", "poleis"], "a test").is_err());
}

fn git(house: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .args(["-c", "commit.gpgsign=false", "-c", "init.defaultBranch=main"])
        .arg("-C")
        .arg(house)
        .args(args)
        .envs([("GIT_AUTHOR_NAME", "a test"), ("GIT_AUTHOR_EMAIL", "test@localhost"), ("GIT_COMMITTER_NAME", "a test"), ("GIT_COMMITTER_EMAIL", "test@localhost")])
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

#[test]
fn on_mainnet_the_asker_is_the_houses_committed_one_and_no_other() {
    let d = Dir::new("house");
    let house = d.0.join("house");
    let own = house.join("tools/embertide-mesh/touchid/touchid");
    let named = an_asker(&d.0, "a-yes-of-a-seats-own", "ok", 0);
    // off mainnet (a rehearsal with key files): the environment may name another, else the house's
    assert_eq!(asker(Cluster::Localnet, &house, Some(named.clone().into_os_string())).unwrap(), named);
    assert_eq!(asker(Cluster::Localnet, &house, None).unwrap(), own);
    // on mainnet: no asker at the house's place, no key read
    assert!(asker(Cluster::Mainnet, &house, Some(named.clone().into_os_string())).unwrap_err().contains("his finger's asker is not at"));
    // an asker in a folder that is no checkout of the house: not the house's committed one
    put(&own, "#!/bin/sh\necho ok\n", 0o755);
    assert!(asker(Cluster::Mainnet, &house, None).unwrap_err().contains("not the house's committed one"));
    // the house's own, committed: heard, and an asker the environment names is not (a commit laid by git's plumbing: no key signs it)
    git(&house, &["init", "-q"]);
    git(&house, &["add", "tools/embertide-mesh/touchid/touchid"]);
    let tree = git(&house, &["write-tree"]);
    let commit = git(&house, &["commit-tree", &tree, "-m", "the asker"]);
    git(&house, &["update-ref", "HEAD", &commit]);
    assert_eq!(asker(Cluster::Mainnet, &house, Some(named.into_os_string())).unwrap(), own);
    // swapped after the commit (a seat's own yes put in its place): refused
    put(&own, "#!/bin/sh\necho ok\nexit 0\n", 0o755);
    assert!(asker(Cluster::Mainnet, &house, None).unwrap_err().contains("not the house's committed one"));
}
