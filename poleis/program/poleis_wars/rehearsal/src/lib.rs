//! The war's rehearsal on a real local validator: the press's own JSON-RPC (`rpc`, by the system's curl, to 127.0.0.1 or [::1] only) and its
//! wire (`wire`: stand-in keys in memory, Solana's legacy message), copied whole from poleis/press; the war's instructions as the program's
//! tests build them and its accounts as they read them (`war`).

pub mod rpc;
pub mod war;
pub mod wire;

pub type Res<T> = Result<T, String>;
