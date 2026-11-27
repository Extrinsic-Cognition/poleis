//! THE PROOFS HOLD: every file of proofs/ states the negation of a claim about the law's arithmetic, and z3 answers `unsat` (no numbers break it:
//! the integers, or the reals where a file states its claim before any floor).
//! On a Mac with no z3 the proofs do not run and this test says so on stderr: read the receipt for where they ran (FUTARCHY_PROOFS=require makes
//! their absence red).
use std::process::Command;

fn z3() -> Option<String> {
    ["z3", "/opt/homebrew/bin/z3", "/usr/local/bin/z3"].iter().find(|p| Command::new(p).arg("--version").output().map(|o| o.status.success()).unwrap_or(false)).map(|p| p.to_string())
}

#[test]
fn the_proofs_hold() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("proofs");
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().map(|x| x == "smt2").unwrap_or(false)).collect();
    files.sort();
    assert_eq!(files.len(), 12, "twelve claims stand in proofs/");
    let Some(z3) = z3() else {
        assert!(std::env::var("FUTARCHY_PROOFS").as_deref() != Ok("require"), "z3 is not on this Mac and FUTARCHY_PROOFS=require");
        eprintln!("THE PROOFS DID NOT RUN: no z3 on this Mac ({} claims unread)", files.len());
        return;
    };
    for f in files {
        let out = Command::new(&z3).arg("-T:60").arg(&f).output().unwrap();
        let said = String::from_utf8_lossy(&out.stdout).trim().to_string();
        assert_eq!(said, "unsat", "{}: z3 said {:?} (unsat means the claim holds for every number it speaks of)", f.file_name().unwrap().to_string_lossy(), said);
        eprintln!("proven: {}", f.file_name().unwrap().to_string_lossy());
    }
}

/// a proof that proves nothing would also say unsat if its premises contradicted each other: each file's premises alone must be satisfiable
#[test]
fn the_premises_are_not_empty() {
    let Some(z3) = z3() else { eprintln!("THE PROOFS DID NOT RUN: no z3 on this Mac"); return };
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("proofs");
    for e in std::fs::read_dir(&dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().map(|x| x != "smt2").unwrap_or(true) { continue; }
        let text = std::fs::read_to_string(&p).unwrap();
        let cut = text[..text.find("(assert (not").expect("a claim to negate")].to_string() + "(check-sat)\n";
        let tmp = std::env::temp_dir().join(format!("futarchy-premises-{}-{}", std::process::id(), p.file_name().unwrap().to_string_lossy()));
        std::fs::write(&tmp, cut).unwrap();
        let out = Command::new(&z3).arg("-T:60").arg(&tmp).output().unwrap();
        let _ = std::fs::remove_file(&tmp);
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "sat", "{}: its premises alone must hold for some integers", p.file_name().unwrap().to_string_lossy());
    }
}
