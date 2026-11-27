//! THE LIGHT'S DOORS STAND ON TODAY'S PAGES (Nov 24 2026, his 17:03:50 UTC "build all and ship to prod", over the drafts of
//! ec-workspace docs/swarmbook-is-the-wishs-road-2026-11-24/the-gaps-drafted.jpg). The doors' markup on the two pages main holds, read
//! in Rust so the page's JavaScript does not grow: her stake under the Pantheon's call (after-the-mint.html), its light's three ways
//! (a wish, the futarchy, a new wish: his 15:26:58 "the making of wish is a capital allocator action"), a swarm's blue door right
//! after the knock (polis.html), and the glue's gate: no door and no read until a chain on this machine names her pool.

use std::fs;
use std::path::PathBuf;

fn page(name: &str) -> Option<String> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../embertide-app/ui/intro/dyson-society/app").join(name);
    if !p.exists() { return None; }
    Some(fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display())))
}

#[test]
fn her_stake_stands_under_the_pantheons_call_hidden() {
    let Some(p) = page("after-the-mint.html") else { return; };
    assert!(p.contains(r#"<p class="stake-state">Her pool opens at her launch.</p><button class="duo-door" type="button" data-her-door hidden>Stake SOL in her ↗</button></div>"#));   // THE PANTHEON IS THE STAKE (his Nov 26 2026): her door stands in the stake beside the ring
    assert!(p.contains(r#"<dialog class="her-dialog" id="her-door" aria-labelledby="her-title">"#));
    assert!(p.contains(r#"<script type="module" src="./her-door.mjs?v=light-3"></script>"#));
}

#[test]
fn the_light_goes_three_ways() {
    let Some(p) = page("after-the-mint.html") else { return; };
    let choice = p.split(r#"<div class="her-choice" data-her-choice hidden>"#).nth(1).and_then(|s| s.split("</div>").next()).expect("the choice");
    assert_eq!(
        choice,
        concat!(
            r#"<a class="her-go" href="./index.html#wall" data-carry>Send it to a wish</a>"#,
            r#"<button class="her-go her-alt" type="button" data-her-futarchy>Let the futarchy place it</button>"#,
            r#"<a class="her-go her-new" href="./index.html#wish-card" data-carry data-her-new>+ Start a new wish</a>"#
        )
    );
}

#[test]
fn a_swarms_light_stands_beside_its_knock_hidden() {
    let Some(p) = page("polis.html") else { return; };
    assert!(p.contains(concat!(
        r#"<div class="doors" id="doors"></div><div class="her-light" data-her-light hidden>"#,
        r#"<button class="door her-send" type="button" data-her-send>Send your light here ↗</button>"#,
        r#"<p class="her-note" data-her-note role="status">from your stake in her</p></div>"#
    )));
    assert!(p.contains(r#"<script type="module" src="./her-door.mjs?v=light-3"></script>"#));
}

#[test]
fn no_pool_no_door_no_read() {
    let Some(g) = page("her-door.mjs") else { return; };
    assert!(g.contains(r#"POOL = /^http:\/\/(127\.0\.0\.1|localhost):\d+\/?$/.test(RPC) ? q.get('pool') : null;"#), "only a practice chain on this machine names her pool today");
    assert!(g.trim_end().ends_with("if (POOL) { pantheon(); wish(); }"), "no pool, no door, no read");
}
