//! poleis-war: the price wars' one law.
//!
//! His words, UTC. Nov 24 2026 01:08:33: "you should follow always the meme royale formula"; 01:14:54, on a deserter: "Half"; his "Yes, burn
//! them" (a deserter's coins are burned, never sold into his house's curve); 09:43:11, "Yes, run it", to running his war on Meme Royale's whole
//! rulebook; and his "Yes, build it" to "Build the war's program first, on a local validator?".
//!
//! `law` is the war's one law in integers: pump.fun's curve priced in SOL, the stake that opens a raid, the line each side must beat, the last
//! stand, his burn for whoever deserts or changes sides, the settlement that converts the loser's holders into the winner's coin, and the dump
//! tax. It is the CORE: no allocation, no std, no dependency (it builds with no feature at all), what the war's program on a chain
//! (poleis/program/poleis_wars) calls whole, one call for each number it moves.
//!
//! `war` (feature std, on by default) is the whole war run minute by minute over a script, the way the owner's model runs it
//! (tests/reference/meme-royale-rulebook-sim.py, `fight(name, script, stand_reserve, burn=True, nested=False)`), built on the core: the same
//! arithmetic, not a copy of it. Its tests replay the model's four reference wars and hold every number to the unit
//! (tests/reference/reference-in-sol.json).
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
pub mod law;
#[cfg(feature = "std")]
pub mod war;
#[cfg(feature = "std")]
pub mod march;
