// Where this file came from: poleis/press/src/wire.rs at b00d47e013 (the launch press's own wire: keys, Solana's legacy message, base58 and base64), copied whole below this line for the war's rehearsal; it stays the press's.
//! The wire: a key, Solana's legacy message compiled by Solana's own compiler (solana-message 4.4.1), the transaction its signers sign,
//! and the text the RPC speaks (base58, base64). The only keys the press signs with are Solana's own: the deployer (the fee payer of every
//! transaction the press sends, the buffers' and the programs' authority until the handoff) and the program accounts it makes; a seat
//! owner's key never comes here, and an agent's comes only to `press chair`, the agent's own step, which reads no key of the sitting
//! (chair.rs). A ballot is checked here against its voter (`verifies`) before the crank sends anything.

use crate::Res;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Signer as _, SigningKey, VerifyingKey}; // classical: Solana's fee payer (the deployer and the program accounts it makes) and its precompile's one curve (an agent's envelope)
use sha2::{Digest, Sha256};
use solana_hash::Hash;
use solana_instruction::Instruction;
use solana_message::legacy::Message;
use solana_pubkey::Pubkey;

/// The size a validator holds every transaction to: an IPv6 packet's 1,280 bytes less its headers.
pub const PACKET: usize = 1_232;

/// A key the press signs with: its 32-byte seed in memory, its address. Dropped, the seed is wiped (the signing crate's own zeroize).
pub struct Key {
    sk: SigningKey,
    pub address: Pubkey,
}

impl Key {
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let sk = SigningKey::from_bytes(seed);
        let address = Pubkey::new_from_array(sk.verifying_key().to_bytes());
        Self { sk, address }
    }
    /// A fresh key: 32 bytes of the system's randomness.
    pub fn fresh() -> Res<Self> {
        use std::io::Read as _;
        let mut seed = [0u8; 32];
        std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut seed)).map_err(|e| format!("the system's randomness: {e}"))?;
        let k = Self::from_seed(&seed);
        seed.fill(0);
        Ok(k)
    }
    /// A key in the Solana CLI's file shape: a JSON array of 64 numbers, the seed then the address; the address held to the seed's.
    pub fn from_cli_text(text: &str) -> Res<Self> {
        let v: Vec<u64> = serde_json::from_str(text).map_err(|_| "a key file is a JSON array of 64 bytes".to_string())?;
        if v.len() != 64 || v.iter().any(|b| *b > 255) {
            return Err("a key file is a JSON array of 64 bytes".into());
        }
        let bytes: Vec<u8> = v.iter().map(|b| *b as u8).collect();
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes[..32]);
        let k = Self::from_seed(&seed);
        seed.fill(0);
        if k.address.to_bytes()[..] != bytes[32..] {
            return Err("a key file whose address is not its seed's".into());
        }
        Ok(k)
    }
    /// The Solana CLI's file shape of this key.
    pub fn cli_text(&self) -> String {
        let mut all = self.sk.to_bytes().to_vec();
        all.extend_from_slice(&self.address.to_bytes());
        let t = format!("[{}]", all.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(","));
        all.fill(0);
        t
    }
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.sk.sign(message).to_bytes()
    }
    /// The key's seed handed to `f`, and the copy wiped after: for the writer of a key's own file (a new agent's Nest key, chair.rs).
    pub fn with_seed<T>(&self, f: impl FnOnce(&[u8; 32]) -> T) -> T {
        let mut seed = self.sk.to_bytes();
        let t = f(&seed);
        seed.fill(0);
        t
    }
    /// The key as a Worker secret holds one (the mint door's POLEIS_ISSUER_KEY, read as its fee key is): the seed then the address, 64 bytes in
    /// base58. Only `press issuer --secret` asks for it: his press, after his finger, into a pipe.
    pub fn secret_base58(&self) -> String {
        let mut all = self.sk.to_bytes().to_vec();
        all.extend_from_slice(&self.address.to_bytes());
        let t = base58(&all);
        all.fill(0);
        t
    }
}

/// Is `signature` the key at `address`'s own over `message`? Strictly, as Solana's precompile reads one (an agent's envelope, a ballot).
pub fn verifies(address: &Pubkey, message: &[u8], signature: &[u8; 64]) -> bool {
    VerifyingKey::from_bytes(&address.to_bytes()).is_ok_and(|k| k.verify_strict(message, &Signature::from_bytes(signature)).is_ok())
}

/// A stand-in key of a rehearsal: sha256 of its name and the rehearsal's seed (held in memory, never written), so a rehearsal cut short
/// resumes with the same keys and no key is ever on a disk.
pub fn standin(seed: &[u8; 32], name: &str) -> Key {
    let mut h = Sha256::new();
    h.update(b"the launch press rehearsal, a stand-in key: ");
    h.update(name.as_bytes());
    h.update(seed);
    let mut s: [u8; 32] = h.finalize().into();
    let k = Key::from_seed(&s);
    s.fill(0);
    k
}

/// One legacy message, compiled by Solana's own compiler: its bytes, and the keys that sign it, the fee payer first.
pub fn message(ixs: &[Instruction], payer: &Pubkey, blockhash: &[u8; 32]) -> (Vec<u8>, Vec<Pubkey>) {
    let m = Message::new_with_blockhash(ixs, Some(payer), &Hash::new_from_array(*blockhash));
    let n = m.header.num_required_signatures as usize;
    (m.serialize(), m.account_keys[..n].to_vec())
}

/// The keys of a message's header and where its blockhash sits: after the 3-byte header, the compact key count and 32 bytes a key.
pub fn layout(message: &[u8]) -> Res<(usize, Vec<Pubkey>, usize)> {
    let (mut n, mut shift, mut at) = (0usize, 0u32, 3usize);
    loop {
        let b = *message.get(at).ok_or("a message cut short")?;
        at += 1;
        n |= ((b & 0x7f) as usize) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 14 {
            return Err("a key count past sixteen bits".into());
        }
    }
    let keys: Vec<Pubkey> = (0..n)
        .map(|i| message.get(at + 32 * i..at + 32 * (i + 1)).map(|k| Pubkey::new_from_array(k.try_into().unwrap())).ok_or("a key cut short"))
        .collect::<Result<_, _>>()?;
    let signers = *message.first().ok_or("an empty message")? as usize;
    Ok((signers, keys, at + 32 * n))
}

/// compact-u16, Solana's length on the wire.
pub fn compact(n: usize, out: &mut Vec<u8>) {
    let mut n = n as u16;
    loop {
        let low = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(low);
            return;
        }
        out.push(low | 0x80);
    }
}

/// A transaction on the wire: its signatures, in the order of the keys that sign, then the message.
pub fn transaction(message: &[u8], signatures: &[[u8; 64]]) -> Vec<u8> {
    let mut t = Vec::with_capacity(1 + 64 * signatures.len() + message.len());
    compact(signatures.len(), &mut t);
    for s in signatures {
        t.extend_from_slice(s);
    }
    t.extend_from_slice(message);
    t
}

/// Every signer of a message signs it with its own key, in the message's order; a signer the press holds no key for refuses it.
pub fn signed(ixs: &[Instruction], payer: &Pubkey, blockhash: &[u8; 32], keys: &[&Key]) -> Res<(Vec<u8>, [u8; 64])> {
    let (m, signers) = message(ixs, payer, blockhash);
    let mut sigs = Vec::with_capacity(signers.len());
    for s in &signers {
        let k = keys.iter().find(|k| &k.address == s).ok_or_else(|| format!("{s} signs this transaction, and the press holds no key for it"))?;
        sigs.push(k.sign(&m));
    }
    let first = sigs[0];
    Ok((transaction(&m, &sigs), first))
}

const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Base58 as Solana writes a key or a signature.
pub fn base58(bytes: &[u8]) -> String {
    let mut digits: Vec<u8> = Vec::new();
    for &byte in bytes {
        let mut carry = u32::from(byte);
        for d in digits.iter_mut() {
            carry += u32::from(*d) << 8;
            *d = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    let zeros = bytes.iter().take_while(|b| **b == 0).count();
    std::iter::repeat_n('1', zeros).chain(digits.iter().rev().map(|d| ALPHABET[usize::from(*d)] as char)).collect()
}

/// Base58 read back into `N` bytes (a blockhash, a signature).
pub fn unbase58<const N: usize>(s: &str) -> Res<[u8; N]> {
    let mut bytes: Vec<u8> = Vec::new();
    for c in s.bytes() {
        let v = ALPHABET.iter().position(|a| *a == c).ok_or_else(|| format!("{s:?} is not base58"))? as u32;
        let mut carry = v;
        for b in bytes.iter_mut() {
            carry += u32::from(*b) * 58;
            *b = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    let zeros = s.bytes().take_while(|c| *c == b'1').count();
    bytes.extend(std::iter::repeat_n(0, zeros));
    bytes.reverse();
    bytes.try_into().map_err(|_| format!("{s:?} is not {N} bytes"))
}

pub fn b64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}
pub fn unb64(s: &str) -> Res<Vec<u8>> {
    STANDARD.decode(s).map_err(|e| format!("base64: {e}"))
}

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn unhex<const N: usize>(s: &str) -> Res<[u8; N]> {
    if s.len() != 2 * N || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("{s:?} is not {N} bytes of hex"));
    }
    let mut out = [0u8; N];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
    }
    Ok(out)
}
