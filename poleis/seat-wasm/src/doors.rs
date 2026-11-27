//! The doors (wasm32 only): `#[no_mangle] extern "C"` over two flat byte buffers, no bindgen, as embertide-mint's. The mint door's glue (the seat's law
//! in embertide-app/ui/intro/dyson-society/worker/space-router-seat.mjs) is their only reader; `ask_of` and `answer` (lib.rs) are their layouts.
//!
//! - `seat_in()`, `seat_out()`: where the door writes its ask, where it reads the answer.
//! - `seat_program()`: OUT <- the Poleis this build speaks (32 bytes).
//! - `seat_mint()`: IN: a deposit's completion (32 bytes) -> OUT: the mint root 0 names its seat by.
//! - `seat_offer(n)`: IN: an ask of n bytes -> OUT: the answer; its message's length, or a refusal's number negated.
//! - `mine_in(n)`, `mine_ask(n)`, `mine_out()`: THE MINE IS LIVE (mine.rs `ask`, JSON in and JSON out, as long as its receipts are): where the door
//!   writes an ask of n bytes; the ask answered, its answer's length; where the answer is read.
use core::ptr::{addr_of, addr_of_mut};

const N: usize = 2048;
static mut IN: [u8; N] = [0; N];
static mut OUT: [u8; N] = [0; N];

fn inp() -> &'static [u8] {
    unsafe { core::slice::from_raw_parts(addr_of!(IN) as *const u8, N) }
}
fn out() -> &'static mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(addr_of_mut!(OUT) as *mut u8, N) }
}

#[no_mangle]
pub extern "C" fn seat_in() -> *mut u8 {
    addr_of_mut!(IN) as *mut u8
}
#[no_mangle]
pub extern "C" fn seat_out() -> *const u8 {
    addr_of!(OUT) as *const u8
}
#[no_mangle]
pub extern "C" fn seat_program() {
    out()[..32].copy_from_slice(crate::PROGRAM.as_ref());
}
#[no_mangle]
pub extern "C" fn seat_mint() {
    let mut c = [0u8; 32];
    c.copy_from_slice(&inp()[..32]);
    out()[..32].copy_from_slice(crate::mint_of(&c).as_ref());
}
#[no_mangle]
pub extern "C" fn seat_offer(n: u32) -> i32 {
    let Some(a) = inp().get(..n as usize).and_then(crate::ask_of) else {
        return -(crate::Refusal::Permit as i32);
    };
    match crate::offer(&a) {
        Ok(o) => {
            let w = crate::answer(&o);
            if w.len() > N {
                return -(crate::Refusal::Size as i32);
            }
            out()[..w.len()].copy_from_slice(&w);
            o.message.len() as i32
        }
        Err(r) => -(r as i32),
    }
}

static mut ASK: Vec<u8> = Vec::new();
static mut SAID: Vec<u8> = Vec::new();

#[no_mangle]
pub extern "C" fn mine_in(n: u32) -> *mut u8 {
    let ask = unsafe { &mut *addr_of_mut!(ASK) };
    ask.clear();
    ask.resize(n as usize, 0);
    ask.as_mut_ptr()
}
#[no_mangle]
pub extern "C" fn mine_ask(n: u32) -> u32 {
    let (ask, said) = unsafe { (&*addr_of!(ASK), &mut *addr_of_mut!(SAID)) };
    let text = ask.get(..n as usize).and_then(|b| core::str::from_utf8(b).ok()).unwrap_or("");
    *said = crate::mine::ask(text).into_bytes();
    said.len() as u32
}
#[no_mangle]
pub extern "C" fn mine_out() -> *const u8 {
    unsafe { (*addr_of!(SAID)).as_ptr() }
}
