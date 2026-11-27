//! STAKE IN HER, SEND YOUR LIGHT, proven in a browser (Nov 22 2026, lane/stake-in-her-send-your-light; his tap "Yes, build them" on "Build the page
//! doors next, on the Pantheon session's page?"). The site's own pages, served from this tree (DOOR_PAGE), in a headless Chrome off every screen
//! (DOOR_CHROME), on a validator holding mainnet's stake pool program (DOOR_VALIDATOR): poleis/ceremony/the-door-rehearsal.sh sets all three. A pool
//! of the launch's shape born by a stand-in manager; a stand-in staker's wallet in the page (it signs whatever the page hands it, as a wallet's
//! yes would); then the visitor's road, each step read back from the chain: the Pantheon's door after her twelve are home, the stake, the one
//! choice for its light (the futarchy), a wish's page and its door (the light poured there, and read back on a new load), and the way out at par.
//! Without DOOR_PAGE it says so and passes.

use poleis_client::{ACT_SIGNER, ACT_WRITABLE};
use poleis_press::door::{self, Light};
use poleis_press::pool::{Pool, MINT_LEN, RESERVE_LEN, STAKE_POOL};
use poleis_press::rpc::{self, Cluster, Rpc};
use poleis_press::wire::{self, Key};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::fs;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

const SOL: u64 = 1_000_000_000;

/// One headless Chrome, one page, driven over its DevTools socket: no window, a throwaway profile removed when it ends.
struct Chrome {
    child: Child,
    ws: WebSocket<MaybeTlsStream<TcpStream>>,
    id: u64,
    session: String,
    profile: PathBuf,
}

impl Chrome {
    fn launch(chrome: &str, profile: PathBuf) -> Chrome {
        let _ = fs::remove_dir_all(&profile);
        fs::create_dir_all(&profile).unwrap();
        let child = Command::new(chrome)
            .args(["--headless=new", "--remote-debugging-port=0", &format!("--user-data-dir={}", profile.display()), "--no-first-run", "--no-default-browser-check", "--mute-audio", "--hide-scrollbars", "--window-size=390,844", "about:blank"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("headless Chrome");
        let file = profile.join("DevToolsActivePort");
        let text = rpc::wait(Duration::from_secs(30), || Ok(fs::read_to_string(&file).ok().filter(|t| t.lines().count() >= 2))).expect("Chrome's DevTools door");
        let mut lines = text.lines();
        let (port, path) = (lines.next().unwrap().trim().to_string(), lines.next().unwrap().trim().to_string());
        let (ws, _) = tungstenite::connect(format!("ws://127.0.0.1:{port}{path}")).expect("Chrome's DevTools socket");
        let mut c = Chrome { child, ws, id: 0, session: String::new(), profile };
        let target = c.call(false, "Target.createTarget", json!({ "url": "about:blank" }))["targetId"].as_str().unwrap().to_string();
        c.session = c.call(false, "Target.attachToTarget", json!({ "targetId": target, "flatten": true }))["sessionId"].as_str().unwrap().to_string();
        c.call(true, "Page.enable", json!({}));
        c.call(true, "Runtime.enable", json!({}));
        c.call(true, "Emulation.setDeviceMetricsOverride", json!({ "width": 390, "height": 844, "deviceScaleFactor": 2, "mobile": true }));
        c
    }

    fn call(&mut self, on_page: bool, method: &str, params: Value) -> Value {
        self.id += 1;
        let mut m = json!({ "id": self.id, "method": method, "params": params });
        if on_page {
            m["sessionId"] = json!(self.session);
        }
        self.ws.send(Message::text(m.to_string())).unwrap();
        loop {
            let Message::Text(t) = self.ws.read().unwrap() else { continue };
            let v: Value = serde_json::from_str(t.as_str()).unwrap();
            if v["id"].as_u64() == Some(self.id) {
                assert!(v["error"].is_null(), "{method}: {}", v["error"]);
                return v["result"].clone();
            }
        }
    }

    fn eval(&mut self, expr: &str) -> Value {
        let r = self.call(true, "Runtime.evaluate", json!({ "expression": expr, "awaitPromise": true, "returnByValue": true }));
        assert!(r["exceptionDetails"].is_null(), "{expr}: {}", r["exceptionDetails"]);
        r["result"]["value"].clone()
    }

    /// The expression's answer once it is one (true, or a text that is not empty), within its seconds.
    fn until(&mut self, what: &str, expr: &str, secs: u64) -> Value {
        let end = Instant::now() + Duration::from_secs(secs);
        loop {
            let v = self.eval(expr);
            if v == json!(true) || v.as_str().is_some_and(|s| !s.is_empty()) {
                return v;
            }
            if Instant::now() > end {
                let note = self.eval("[...document.querySelectorAll('[data-her-note]')].map((n) => n.textContent).join(' | ')");
                panic!("{what}: not in {secs} s (the page says: {note})");
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    fn go(&mut self, url: &str) {
        self.call(true, "Page.navigate", json!({ "url": url }));
    }

    fn shot(&mut self, dir: &Option<PathBuf>, name: &str) {
        let Some(dir) = dir else { return };
        let r = self.call(true, "Page.captureScreenshot", json!({ "format": "jpeg", "quality": 82 }));
        fs::write(dir.join(name), wire::unb64(r["data"].as_str().unwrap()).unwrap()).unwrap();
    }
}

impl Drop for Chrome {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.profile);
    }
}

/// A pool of the launch's shape (its epoch fee 1/1, no deposit authority) born by a stand-in manager, its fee account the manager's receipts.
fn born(rpc: &Rpc, manager: &Key) -> Pool {
    let mut p = Pool::derive(&manager.address, "door1", 10, &STAKE_POOL, &Pubkey::new_from_array([7; 32])).unwrap();
    p.agent = manager.address;
    p.vault = door::receipts_of(&manager.address, &p.mint);
    let mut first = p.reserve_seeded().making(&p.base, rpc.rent(RESERVE_LEN).unwrap(), None).unwrap();
    first.push(p.reserve_init());
    first.extend(p.mint_seeded().making(&p.base, rpc.rent(MINT_LEN).unwrap(), None).unwrap());
    first.push(p.mint_init());
    land(rpc, &first, manager);
    land(rpc, &[door::receipts_made(&manager.address, &manager.address, &p.mint)], manager);
    let mut birth = Vec::new();
    for s in p.in_the_execute() {
        birth.extend(s.making(&p.base, rpc.rent(s.space).unwrap(), None).unwrap());
    }
    let accounts = p.initialize_accounts().into_iter().map(|(k, f)| AccountMeta { pubkey: k, is_signer: f & ACT_SIGNER != 0, is_writable: f & ACT_WRITABLE != 0 }).collect();
    birth.push(Instruction::new_with_bytes(p.program, &p.initialize_data(), accounts));
    land(rpc, &birth, manager);
    p
}

fn land(rpc: &Rpc, ixs: &[Instruction], payer: &Key) {
    let blockhash = rpc.blockhash().unwrap();
    let (w, _) = wire::signed(ixs, &payer.address, &blockhash, &[payer]).unwrap();
    let sig = rpc.send(&w, true).unwrap();
    rpc::wait(Duration::from_secs(60), || match rpc.statuses(&[sig.clone()])?.pop().flatten() {
        Some((_, err, _)) if !err.is_null() => Err(format!("{sig}: {err}")),
        Some((_, _, s)) if s == "confirmed" || s == "finalized" => Ok(Some(())),
        _ => Ok(None),
    })
    .unwrap();
}

fn receipts(rpc: &Rpc, owner: &Pubkey, p: &Pool) -> u64 {
    rpc.account(&door::receipts_of(owner, &p.mint).to_string()).unwrap().map(|a| door::amount_of(&a.data).unwrap()).unwrap_or(0)
}

/// The light a staker chose last: the newest of the staker's own signed memos (`poleis_door::light_signed`, the page's law and the pour's).
fn light_now(rpc: &Rpc, staker: &Pubkey) -> Option<Light> {
    let listed = rpc.call("getSignaturesForAddress", json!([staker.to_string(), { "limit": 25, "commitment": "confirmed" }])).unwrap();
    listed.as_array()?.iter().filter(|s| s["err"].is_null() && s["memo"].is_string()).find_map(|s| {
        let t = rpc.call("getTransaction", json!([s["signature"], { "encoding": "base64", "commitment": "confirmed", "maxSupportedTransactionVersion": 0 }])).ok()?;
        poleis_door::light_signed(&poleis_press::wire::unb64(t["transaction"][0].as_str()?).ok()?, staker)
    })
}

/// A press of the page and what came of it: the job's answer once the page is no longer busy, retried while her pool waits on the epoch's rewards.
fn act(c: &mut Chrome, what: &str, busy: &str, press: &str, done: &str) {
    for _ in 0..20 {
        c.eval(&format!("{press}; true"));
        let r = c.until(what, &format!("(() => {{ if (document.querySelector('{busy}')?.hasAttribute('aria-busy')) return ''; if ({done}) return true; return document.querySelector('{busy} [data-her-note]')?.textContent || ''; }})()"), 120);
        if r == json!(true) {
            return;
        }
        assert_eq!(r.as_str(), Some(door::WAIT), "{what}");
        std::thread::sleep(Duration::from_secs(2));
    }
    panic!("{what}: her pool waited on the epoch's rewards twenty times");
}

#[test]
fn the_doors_on_the_page_in_a_browser() {
    let (Ok(url), Some(page)) = (std::env::var("DOOR_VALIDATOR"), std::env::var("DOOR_PAGE").ok().filter(|p| !p.is_empty())) else {
        eprintln!("DOOR_VALIDATOR or DOOR_PAGE is not set: the browser's half runs under poleis/ceremony/the-door-rehearsal.sh");
        return;
    };
    let chrome = std::env::var("DOOR_CHROME").unwrap_or_else(|_| "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".into());
    let shots = std::env::var("DOOR_SHOTS").ok().filter(|s| !s.is_empty()).map(PathBuf::from);
    if let Some(d) = &shots {
        fs::create_dir_all(d).unwrap();
    }
    let rpc = Rpc::new(&url, Cluster::Localnet).unwrap();
    let mut said: Vec<Value> = Vec::new();
    let mut say = |what: &str, v: Value| {
        eprintln!("{what}: {v}");
        said.push(json!({ "what": what, "is": v }));
    };

    // the module the page runs: the bytes poleis/door/build.sh wrote beside it
    let module = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../embertide-app/ui/intro/dyson-society/app/wasm/poleis-door.wasm");
    let bytes = fs::read(&module).expect("the page's module");
    say("the page's module", json!({ "bytes": bytes.len(), "sha256": wire::hex(&wire::sha256(&bytes)) }));

    // two stand-ins, their seed the run's own; the staker's key is the page's wallet
    let seed: [u8; 32] = wire::sha256(format!("the doors' rehearsal {url} {}", std::process::id()).as_bytes());
    let manager = wire::standin(&seed, "the doors' manager");
    let staker = wire::standin(&seed, "the doors' staker");
    for k in [&manager, &staker] {
        rpc.airdrop(&k.address.to_string(), 100 * SOL).unwrap();
    }
    rpc::wait(Duration::from_secs(60), || Ok((rpc.balance(&staker.address.to_string())? >= 100 * SOL && rpc.balance(&manager.address.to_string())? >= 100 * SOL).then_some(()))).unwrap();
    let p = born(&rpc, &manager);
    say("her pool born", json!({ "pool": p.pool.to_string(), "mint": p.mint.to_string() }));

    let secret: Vec<u8> = wire::unbase58::<64>(&staker.secret_base58()).unwrap().to_vec();
    let wallet = format!(
        "(() => {{ const secret = Uint8Array.from({secret:?}), pk = '{pk}'; window.solana = {{ publicKey: null, async connect() {{ this.publicKey = {{ toBase58: () => pk, toString: () => pk }}; return {{ publicKey: this.publicKey }}; }}, async signTransaction(tx) {{ tx.partialSign(window.solanaWeb3.Keypair.fromSecretKey(secret)); return tx; }} }}; }})();",
        pk = staker.address
    );
    let mut c = Chrome::launch(&chrome, std::env::temp_dir().join(format!("the-doors-chrome-{}", std::process::id())));
    c.call(true, "Page.addScriptToEvaluateOnNewDocument", json!({ "source": wallet }));
    let query = format!("rpc={url}&pool={}", p.pool);

    // HER: the Pantheon's one door, under its call on today's page (after-the-mint.html, since swarmbook.html was retired; Nov 24 2026)
    let pantheon = |c: &mut Chrome| {
        // ?side=pool: the visit the split's blue side sends (before the mint's window closes, a plain visit goes on to the mint page); its film
        // skipped by the visitor's own Skip, the bar stands and the Pantheon with it
        c.go(&format!("{page}after-the-mint.html?side=pool&{query}"));
        c.until("the Pantheon's door", "(() => { const d = document.querySelector('[data-her-door]'); return !!d && !d.hidden; })()", 60);
        c.until("the Pantheon on the glass", "(() => { const s = document.querySelector('.am-skip'); if (s && s.checkVisibility()) s.click(); const d = document.querySelector('[data-her-door]'); d.scrollIntoView({ block: 'center' }); return d.checkVisibility({ opacityProperty: true, visibilityProperty: true }); })()", 90);
        std::thread::sleep(Duration::from_millis(2500));
    };
    pantheon(&mut c);
    let shown = c.eval("(() => { const d = document.querySelector('[data-her-door]'), r = d.getBoundingClientRect(); return d.checkVisibility({ opacityProperty: true, visibilityProperty: true }) && r.width > 0 && r.bottom > 0 && r.top < innerHeight; })()");
    assert_eq!(shown, json!(true), "the door stands under the Pantheon's call, on the glass");
    c.shot(&shots, "1-her.jpg");

    // THE STAKE: 25 SOL typed and staked; the one choice for its light comes up
    c.eval("document.querySelector('[data-her-door]').click(); true");
    c.until("the dialog", "(() => { const d = document.querySelector('#her-door'); return d.open && !d.hasAttribute('aria-busy'); })()", 30);
    assert_eq!(c.eval("document.querySelector('[data-her-out]').hidden"), json!(true), "nothing of a new wallet's is in her");
    c.eval("document.querySelector('#her-sol').value = '25'; true");
    c.shot(&shots, "2-the-stake.jpg");
    let before = rpc.balance(&staker.address.to_string()).unwrap();
    act(&mut c, "the stake", "#her-door", "document.querySelector('[data-her-stake]').requestSubmit()", "!document.querySelector('[data-her-choice]').hidden");
    let held = receipts(&rpc, &staker.address, &p);
    assert_eq!(held, 25 * SOL, "one receipt a lamport");
    let eyebrow = c.eval("document.querySelector('[data-her-eyebrow]').textContent");
    assert_eq!(eyebrow, json!("Your 25 SOL is in her"));
    let ways = c.eval("[...document.querySelectorAll('[data-her-choice] > *')].map((e) => [e.textContent, (e.getAttribute('href') || '').replace(/[?&](rpc|pool)=[^&#]*/g, '')]).flat().join(' | ')");
    assert_eq!(ways, json!("Send it to a wish | ./index.html#wall | Let the futarchy place it |  | + Start a new wish | ./index.html#wish-card"), "the light's three ways (each link carries the visit's chain, here the practice chain's, as the page's data-carry does)");
    c.shot(&shots, "3-the-choice.jpg");
    say("the stake", json!({ "typed": "25", "receipts": held, "the staker paid": before - rpc.balance(&staker.address.to_string()).unwrap(), "the page said": eyebrow, "the three ways": ways }));

    // ITS LIGHT: the futarchy, one memo the wallet signed
    act(&mut c, "the futarchy", "#her-door", "document.querySelector('[data-her-futarchy]').click()", "document.querySelector('#her-door [data-her-note]').textContent.includes('futarchy places it')");
    assert_eq!(light_now(&rpc, &staker.address), Some(Light::Futarchy));
    say("its light", json!({ "chosen": "the futarchy", "read back": format!("{:?}", light_now(&rpc, &staker.address)) }));

    // A WISH: its page's door pours the light there; a new load reads it back
    let wish = format!("{page}polis.html?civ=7&{query}");
    c.go(&wish);
    c.until("the wish's door", "(() => { const b = document.querySelector('[data-her-light]'); return !!b && !b.hidden && !b.hasAttribute('aria-busy'); })()", 60);
    assert_eq!(c.eval("document.querySelector('[data-her-send]').textContent"), json!("Send your light here ↗"), "the light is the futarchy's, not this wish's");
    c.shot(&shots, "4-a-wish.jpg");
    act(&mut c, "the wish", "[data-her-light]", "document.querySelector('[data-her-send]').click()", "document.querySelector('[data-her-send]').classList.contains('held')");
    assert_eq!(light_now(&rpc, &staker.address), Some(Light::Wish(7)));
    c.go(&wish);
    c.until("the light read back", "(() => { const s = document.querySelector('[data-her-send]'); return !!s && s.classList.contains('held'); })()", 60);
    let note = c.eval("document.querySelector('[data-her-light] [data-her-note]').textContent");
    assert_eq!(note, json!("Your 25 SOL stays in her. Change it any epoch."));
    c.shot(&shots, "5-your-light-there.jpg");
    // BESIDE THE KNOCK (draft 3): no swarm stands on this practice chain, so the knock is written by the page's own writer (polis-knock.mjs doorOf)
    // with stand-in numbers, on a desktop's width: a layout frame, not a read of the chain
    c.call(true, "Emulation.setDeviceMetricsOverride", json!({ "width": 1280, "height": 900, "deviceScaleFactor": 1, "mobile": false }));
    let beside = c.eval("(async () => { const { doorOf } = await import('./polis-knock.mjs?v=knock-1'); document.querySelector('#doors').innerHTML = doorOf({ at: 7, seats: 8, sol: (x) => x, dur: (x) => x }); document.querySelector('.her-light').scrollIntoView({ block: 'center' }); await new Promise((r) => setTimeout(r, 400)); const k = document.querySelector('#doors .door').getBoundingClientRect(), l = document.querySelector('[data-her-send]').getBoundingClientRect(); return Math.abs(k.top - l.top) < 2 && l.left > k.right; })()");
    assert_eq!(beside, json!(true), "the blue door stands beside the red knock");
    c.shot(&shots, "5b-beside-the-knock.jpg");
    c.call(true, "Emulation.setDeviceMetricsOverride", json!({ "width": 390, "height": 844, "deviceScaleFactor": 2, "mobile": true }));
    say("a wish", json!({ "unit": 7, "read back": format!("{:?}", light_now(&rpc, &staker.address)), "the page said on a new load": note }));

    // THE WAY OUT: from her, every receipt burned, the SOL back at par
    pantheon(&mut c);
    c.eval("document.querySelector('[data-her-door]').click(); true");
    c.until("the way out's door", "(() => { const d = document.querySelector('#her-door'), o = document.querySelector('[data-her-out]'); return d.open && !d.hasAttribute('aria-busy') && !o.hidden && o.textContent; })()", 30);
    assert_eq!(c.eval("document.querySelector('[data-her-out]').textContent"), json!("Take your 25 SOL back ↗"));
    c.shot(&shots, "6-the-way-out.jpg");
    let before = rpc.balance(&staker.address.to_string()).unwrap();
    act(&mut c, "the way out", "#her-door", "document.querySelector('[data-her-out]').click()", "document.querySelector('#her-door [data-her-note]').textContent.includes('back in your wallet')");
    let after = rpc.balance(&staker.address.to_string()).unwrap();
    assert_eq!(receipts(&rpc, &staker.address, &p), 0, "every receipt burned");
    assert!(after > before && after - before > 25 * SOL - 100_000, "25 SOL back, less the fee: {before} to {after}");
    say("the way out", json!({ "lamports back, less the fee": after - before }));
    drop(c);

    if let Ok(path) = std::env::var("DOOR_PAGE_CERTIFICATE") {
        if !path.is_empty() {
            let cert = json!({ "schema": "embertide.door-page-rehearsal/1", "validator": "solana-test-validator, local, 32 slots an epoch", "browser": "headless Chrome, 390 by 844 at 2x (the layout frame 1280 by 900), a stand-in's wallet", "said": said });
            fs::write(&path, serde_json::to_string_pretty(&cert).unwrap()).unwrap();
        }
    }
}
