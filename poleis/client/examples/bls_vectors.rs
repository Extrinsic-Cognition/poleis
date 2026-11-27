//! Reference vectors for the aggregate vote, for other language clients:
//! a key from a fixed seed, the message point of a fixed envelope (its
//! counter, root and cleared bytes), the signature, the proof of
//! possession, and the two instructions' bytes, printed as JSON.
//!
//! Run: cargo run -q --example bls_vectors

use poleis_client::bls::{attest_bls_side, point_bytes, pop_point, registration, vote_point, BlsKeypair};
use poleis_client::{bls::bls_key_pda, proposal_hash, Payload, ROOT_UNIT, VOTE_PRO};
use solana_pubkey::Pubkey;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn main() {
    let holder = Pubkey::new_from_array([2u8; 32]);
    let crank = Pubkey::new_from_array([9u8; 32]);
    let key = BlsKeypair::from_seed(&[7u8; 32]);
    let pk = key.pk_bytes();
    let pop = pop_point(ROOT_UNIT, &holder, &pk);
    let pop_sig = key.sign(&pop.point);
    let manifest = [3u8; 32];
    let payload = Payload::Window { origin: ROOT_UNIT, window: 2_000, codex_window: 6_000 };
    let hash = proposal_hash(payload.class(), ROOT_UNIT, &manifest, &payload.hash(), 42);
    let point = vote_point(&hash, ROOT_UNIT, VOTE_PRO);
    let sig = key.sign(&point.point);
    let reg = registration(holder, ROOT_UNIT, &key);
    let att = attest_bls_side(crank, hash, ROOT_UNIT, VOTE_PRO, &[(holder, &key)]);
    println!("{{");
    println!("  \"seed\": \"{}\",", hex(&[7u8; 32]));
    println!("  \"pk\": \"{}\",", hex(&pk));
    println!("  \"key_pda\": \"{}\",", bls_key_pda(ROOT_UNIT, &holder));
    println!("  \"pop\": {{ \"ctr\": {}, \"y\": \"{}\", \"point\": \"{}\", \"sig\": \"{}\" }},", pop.ctr, hex(&pop.y), hex(&point_bytes(&pop.point)), hex(&point_bytes(&pop_sig)));
    println!("  \"hash\": \"{}\",", hex(&hash));
    println!("  \"vote\": {{ \"ctr\": {}, \"y\": \"{}\", \"point\": \"{}\", \"sig\": \"{}\" }},", point.ctr, hex(&point.y), hex(&point_bytes(&point.point)), hex(&point_bytes(&sig)));
    println!("  \"register_bls_key\": \"{}\",", hex(&reg.data));
    println!("  \"attest_bls\": \"{}\"", hex(&att.data));
    println!("}}");
}
