//! The record of a sitting: one JSON file in the sitting's folder, rewritten whole after every transaction (a new file, then a rename, so a
//! sitting cut short leaves the last whole record and never half of one). What it says is checked against the chain at every step: the
//! chain is the truth, and the record is what the birth certificate is made of.

use crate::Res;
use serde_json::{json, Value};
use std::io::Write as _;
use std::path::{Path, PathBuf};

pub struct Record {
    pub path: PathBuf,
    pub v: Value,
}

/// One transaction the press sent and the cluster confirmed.
#[derive(Clone, Debug)]
pub struct Landed {
    pub step: String,
    pub what: String,
    pub signature: String,
    pub slot: u64,
    pub fee: u64,
    pub units: Option<u64>,
    pub bytes: usize,
    pub signatures: usize,
}

impl Record {
    pub fn open(dir: &Path) -> Res<Self> {
        let path = dir.join("record.json");
        let v = match std::fs::read_to_string(&path) {
            Ok(t) => serde_json::from_str(&t).map_err(|e| format!("{}: {e}", path.display()))?,
            Err(_) => json!({ "what": "THE LAUNCH PRESS: the record of one sitting", "transactions": [], "resumes": [] }),
        };
        Ok(Self { path, v })
    }
    pub fn save(&self) -> Res<()> {
        let tmp = self.path.with_extension("json.part");
        let mut f = std::fs::File::create(&tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
        f.write_all(serde_json::to_string_pretty(&self.v).unwrap().as_bytes()).and_then(|_| f.sync_all()).map_err(|e| format!("{}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &self.path).map_err(|e| format!("{}: {e}", self.path.display()))
    }
    pub fn get(&self, key: &str) -> &Value {
        &self.v[key]
    }
    pub fn set(&mut self, key: &str, value: Value) -> Res<()> {
        self.v[key] = value;
        self.save()
    }
    /// A landed transaction, kept with its step.
    pub fn landed(&mut self, l: &Landed) -> Res<()> {
        let row = json!({
            "step": l.step, "what": l.what, "signature": l.signature, "slot": l.slot, "fee": l.fee, "units": l.units, "bytes": l.bytes, "signatures": l.signatures,
        });
        self.v["transactions"].as_array_mut().ok_or("the record's transactions")?.push(row);
        self.save()
    }
    /// Many landed at once (a buffer's writes), saved once.
    pub fn landed_all(&mut self, ls: &[Landed]) -> Res<()> {
        let rows = self.v["transactions"].as_array_mut().ok_or("the record's transactions")?;
        for l in ls {
            rows.push(json!({ "step": l.step, "what": l.what, "signature": l.signature, "slot": l.slot, "fee": l.fee, "units": l.units, "bytes": l.bytes, "signatures": l.signatures }));
        }
        self.save()
    }
    /// A step found standing on the chain what an earlier pass of the sitting had begun: said in the record, where a resumed sitting resumed.
    pub fn resumed(&mut self, what: String) -> Res<()> {
        eprintln!("  resumed: {what}");
        self.v["resumes"].as_array_mut().ok_or("the record's resumes")?.push(json!(what));
        self.save()
    }
    pub fn transactions(&self) -> Vec<Value> {
        self.v["transactions"].as_array().cloned().unwrap_or_default()
    }
}
