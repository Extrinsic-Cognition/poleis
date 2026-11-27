//! embertide-futarchy: how the swarm decides.
//!
//! His words, UTC. Oct 26 2026 18:18: "i am very fond of futarchy as a way to have ai automated decision markets for a collective". Oct 29 19:10:
//! "Nefertiti will spawn bunch of possible roads to be taken, market will then run futarchy on the proposed avenues... it is not hard"; 19:12:
//! "$WARM is liquid upfront qnd is what the futarchy runs decisions on". Nov 11 00:55: "i think we should get into ai futarchy with formal
//! verification for governance"; then, over the drawing of the rule on the Swarmbook page: "ok, perfect. can you implement that?".
//!
//! `nest` is the market on the Nest (a road is a ref, a trade a signed cause, the decision a ref) and `embertide-futarchy` is its command.
//! `law` is the market's one law. The text it outgrows, tools/embertide-mesh/futarchy/futarchy-law.mjs (Oct 30 2026), stays as its witness: on the
//! grid the two give the same numbers (tests/witness.rs, the text's own answers kept as data), and off the grid the text is wrong (law.rs THE GRID).
//!
//! The law is read two ways (Nov 20 2026). `law` is its CORE: one trade, one boundary of the grid, the decision and one purse's settle, with no
//! allocation and no std, what a program on a chain depends on (default-features = false). `fold` (feature fold) is the fold over a road's whole
//! list of entries, built on the core: the same arithmetic, not a copy of it. When the fold is built `law` names it too, so `law::*` gives both.
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#[cfg(feature = "fold")]
extern crate alloc;
pub mod law;
#[cfg(feature = "fold")]
pub mod fold;
#[cfg(feature = "nest")]
pub mod nest;
