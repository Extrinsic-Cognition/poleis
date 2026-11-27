# The order marches — receipt

56 tests pass: 25 in `embertide-war`, 31 in `poleis-wars`. The 40 existing tests and the reference files are unchanged. All four reference wars retain every checked coin, lamport, fee, outcome and minute at `march_s = 0`, in the law and the program's SBF execution. No commit, push, deployment, branch switch, live account or external service was used.

## Built

`Rules::march_s` defaults to zero, including `MEME_ROYALE` and `Default`. `Order::accept` records the input, kind, minimum, acceptance instant and landing instant; `land_on` prices against the landing curve. Both replay and program call this same integer law. No new dependency, random choice, timer or background worker.

The timed replay adds `enter_at`, `sell_at`, `cross_at`, `desert_at`, `minute_at`, `land` and explicit quote funding. Its public FIFO includes the maker and house. Sales remove coins and proportional cost basis at acceptance; buys remove their entire spend from the funded quote balance. Existing signatures remain. When a raid enables marching, the old untimed trade methods refuse and direct callers to the timed methods rather than inventing a timestamp. `minute(m)` uses second `m * 60`; `minute_at` accepts the exact second. Timed replay rejects backward input instants. After settlement the winning curve can trade immediately again.

The program adds a public raid queue at PDA `["march", raid]`. Its 708 bytes contain up to eight orders, owner, house side, sequence, amount, minimum, accepted/landing seconds and sale cost basis. Buy lamports stay in this queue, outside the houses' reserves and fees. Sale coins leave their positions and the house's `held` count; the curve does not change until arrival. Same-second orders retain acceptance sequence. The first accepter funds queue rent; later acceptance reuses it.

Anyone may land every due order, supplying its recorded owner and position in FIFO order. An incomplete list, reordered destinations, or attempting to include a future order fails atomically. Before a due order lands, neither a burn, a change of sides nor a raid crank can overtake it. Ordinary buys during an enabled raid cannot use the legacy account list to bypass the queue. A failed minimum or an unfillable trade refunds its input and does not poison the head.

Payouts, refunds and buy change are quote credits above the existing position account's rent. Its owner claims them to a chosen recipient. This prevents an emptied or reassigned wallet from blocking everyone else's FIFO through a rent failure. The existing conversion closes a losing position and returns its unclaimed quote with its rent. Old position bytes are unchanged.

## Wire and compatibility

All existing tags, data widths, account layouts and zero-delay economic behavior remain. Configs of 223 bytes read as zero delay. A positive-delay config is 227 bytes: its original bytes followed by `march_s:u32` at offset 223. Layout version remains 2; no existing account needs migration.

- **14 `init_march_config`:** existing init-config data followed by `march_s:u32`, same accounts. A zero value produces the original 223-byte layout. Existing tag 0 still accepts exactly its original data and always creates zero delay. Rules stay immutable per generation.
- **2/3 buy/sell during an enabled raid:** original 16 data bytes; accounts `[owner(s,w), config, house(w), position(w), fees, system, raid, queue(w)]`. Outside a raid use the existing instructions/account lists.
- **5 desert, 6 change_sides, 7 crank, 8 settle:** append the raid queue to their original account lists for positive-delay configs. It is read-only for desert/crank/settle, writable for change_sides. An absent queue is checked at its canonical PDA and counts as empty. Change-sides burns immediately and escrows the proceeds as a buy of the other house.
- **15 `land_orders`:** no data; `[config, raid, attacker(w), defender(w), fees(w), queue(w), (owner, position(w))*]`. No participant signature. Include exactly all due orders; after decision include every remaining order for refund. Owner/position pairs may repeat.
- **16 `claim_march_quote`:** no data; `[owner(s), position(w), recipient(w)]`. Transfers all lamports above position rent; no coins or cost basis move.

## Measured cost

Mollusk executes this tree's freshly built SBF. These are the same `tests/cost.rs` fixtures before/after; PDA birth searches make costs address-dependent. The original binary was measured before the new path was compiled. Units are CU, not wall-clock microseconds.

| Instruction / shape | Before | After, delay 0 |
|---|---:|---:|
| init_config | 10,611 | 10,604 |
| init_house | 7,004 | 7,011 |
| buy, position born (40 buyers) | 8,816–16,317 | 8,894–16,395 |
| buy, position standing | 7,766 | 7,844 |
| buy in raid, position born / standing | 9,358 / 8,307 | 9,444 / 8,393 |
| sell, ordinary / dump tax / after tax | 7,282 / 7,499 / 7,289 | 7,332 / 7,549 / 7,339 |
| stake, proposal born / still proposed / opens | 14,902 / 12,514 / 13,245 | 14,952 / 12,564 / 13,295 |
| desert | 6,629 | 6,700 |
| change_sides, position born / standing | 13,927 / 12,877 | 14,074 / 13,024 |
| crank, open / opens stand / stand / decides | 5,753 / 6,009 / 5,755 / 5,767 | 5,963 / 6,219 / 5,965 / 5,977 |
| settle | 41,245 | 41,447 |
| convert, position born / standing | 12,418 / 8,368 | 12,469 / 8,419 |
| unlock | 4,003 | 4,004 |
| unstake | 4,520 | 4,521 |
| pay_creator, active or rekt | 2,189 | 2,189 |
| pay_publisher | 2,623 | 2,630 |

Positive-delay fixtures use ten seconds. The new instructions have no before measurement because they did not exist. Existing guarded-instruction numbers below use different fixtures from `cost.rs`, so they are not paired deltas.

| Marching instruction / shape | Measured CU |
|---|---:|
| init_march_config, positive fixtures | 10,609–28,609 |
| buy acceptance, queue + position born | 14,104 |
| buy acceptance, queue standing + position born | 9,934 |
| buy acceptance, both standing | 8,946 |
| sell acceptance | 8,257 |
| guarded desert | 8,634 |
| change_sides, burn + buy acceptance | 21,751 |
| guarded crank / settle | 8,300 / 38,623 |
| land, none due / one buy / two buys | 5,570 / 9,698 / 14,007 |
| land, buy then sale | 13,742 |
| land, all eight buys | 39,288 |
| refund, buy + sale / eight buys | 10,302 / 37,066 |
| claim quote | 2,112 |

Before: immediate execution, no queue, zero idle work. Predicted: bounded O(8) landing, one queue, no idle work. Measured: eight buys use 39,288 CU, below 200,000; the zero-delay cost fixture's maximum rises from 41,245 to 41,447 CU. SBF grows from 143,792 to 172,536 bytes (+28,744). The no_std law and program allocate no heap. The std replay clones its state for atomic timed operations; its memory/throughput was not benchmarked.

Legacy Config/Fees/House/Position/Raid remain 223/67/240/123/255 bytes. Positive config adds four bytes and 27,840 lamports rent at the test cluster's rate. Queue: 708 bytes, 5,818,560 lamports rent. Position quote credits add no data bytes or rent.

## His calls

- **his call — duration:** no live duration chosen or enabled. Zero remains the required default; ten seconds is only the test fixture. An authority chooses a fixed positive `u32` number of real seconds when founding a generation. Both houses and both trade kinds get the same delay, preserving FIFO arrival order.
- **his call — withdrawal:** no voluntary withdrawal. Input remains committed until execution, failed-price refund, expiration or raid-decision refund. This avoids a cancellable displayed order that disappears when the other side responds.
- **his call — orders still marching at decision/settlement:** due orders must be processed before the deciding crank. Future orders are canceled by the decision and returned, even before their scheduled arrival. Settlement waits for this refund pass, so sale coins and cost basis are restored before holder weights are fixed. It does not wait for owners to claim quote credits. Arrivals outside the last permitted raid minute refund without trading, including when the keeper is late.
- **his call — deserter burn:** immediate, preserving his half-and-burn rule, but blocked behind already-due orders. A change of sides burns now and its resulting buy marches. The burn does not reverse if that later buy refunds.
- **his call — raid sales:** the explicit brief enables ordinary queued sales during positive-delay raids; they use `sell_on`, not the deserter burn/half payout. Zero delay retains the existing `InRaid` sale refusal. Queued raid sales carry no dump tax; existing post-settlement sales and their tax remain unchanged. This distinction permits a delayed market sale instead of immediate desertion and needs his economic ruling before enabling it.
- **his call — capacity:** eight pending orders per raid, shared across both sides. A ninth is refused without taking funds until space is available. This bounds all-due execution and account count to one ordinary transaction; it is not a high-throughput market queue. Saturating this cap can deny new acceptance until arrivals clear it; no spam economics were invented here.
- **his call — price/curve failure:** refund and advance, charging no trade fee. The minimum is committed at acceptance and cannot be changed. Holding a failing head forever would let one order stop the war.
- **his call — quote custody and rent:** landing credits the existing position; its owner makes a separate claim to a recipient. The first accepter pays queue rent, which stays in the reusable queue, as existing program accounts retain rent. Donations to a position are claimable by its owner; donations to a queue are not traded or paid as fees.
- **his call — decision timing:** preserve the existing program's once-per-minute, caller-cranked decision rule. Orders due at the crank's chain timestamp are included first. This adds no autonomous keeper or alternative raid clock; it does not address validator transaction ordering before acceptance.

## Verification and run ledger

Every cargo command used `nice -n 19`, `--offline`, and `CARGO_TARGET_DIR=~/.cache/seat-e216f56d/target-war`. The sandbox printed `nice: setpriority: Operation not permitted`, so lowering priority could not be verified. Later commands had a 600-second alarm. No network was needed. Only these two crates' test suites ran; the existing publisher test builds/loads `poleis_treasury` as its unchanged fixture, without running that crate's suite or editing its sources.

| Run | Count and result |
|---|---|
| Initial law baseline | 18 passed: law 16, reference 2 |
| Initial program baseline/cost capture | id 1, brought 3, cost 1, publisher 1 passed; refusals 7 passed / 7 failed; wars not reached. I edited the shared law while the baseline suite's per-binary SBF builds were still running, briefly leaving Config's explicit Rules initializer incomplete. Cost capture had completed on the original binary. This was a sequencing mistake, not an existing-test change. |
| Law after first core addition | 18 passed |
| Program cost after initial march implementation | 1 passed |
| Initial new program march suite | 7 passed |
| Law with initial march suite | 23 passed: 16 + 5 + 2 |
| First complete program pass | 29 passed: existing 22 + march 7 |
| Program march after rent-safe credits and boundary cases | 9 passed |
| Complete program after credits/time review | 31 passed |
| Law after monotonic-time guard | law 16 passed, march 5 passed / 1 failed; reference not reached. A new fixture called second 10 before deserting at second 5; corrected the fixture's order. No existing test changed. |
| Law after correcting the chronological fixture | 24 passed: law 16, march 6, reference 2 |
| Final law, also checking immediate trades through the new timed APIs respect their minimum | 25 passed: law 16, march 7, reference 2; zero doctests |
| Final program, existing shared tests restored byte-for-byte after rustfmt's recursive whitespace changes | 31 passed: id 1, brought 3, cost 1, march 9, publisher 1, refusals 14, wars 2; zero doctests |

Final commands: `cargo test --offline --manifest-path embertide-war/Cargo.toml` and `cargo test --offline --manifest-path poleis/program/poleis_wars/Cargo.toml -- --nocapture --test-threads=1`, under the prefix above. The program harness builds its own SBF and tests those bytes. The law reference suite also ran the unchanged Python model and compared regenerated JSON byte-for-byte. Existing test and reference diffs are empty. `git diff --check` passes. Formatting used installed rustfmt 1.93.1 after the default formatter failed with a local LLVM loader mismatch and 1.99 lacked the component; no toolchain was installed.

Final local logs: `/tmp/order-marches-law-final.log`, `/tmp/order-marches-program-final.log`; original cost capture: `/tmp/order-marches-before.log`. These are local logs, not required artifacts.

Not verified: a local validator or network deployment, live keeper latency, mainnet scheduling, high-throughput operation, economic sandwich resistance beyond FIFO after acceptance, and changes in crates outside this brief. The program cannot execute itself at a deadline: the permissionless crank applies the scheduled-price state later, and the guards prevent intervening curve mutations from changing that price. No window, app or browser was opened. No key material or prohibited account/config directories were read. No long-lived process was started; test/build subprocesses ended with their runs. The oath was sworn in this trace, not posted to a live Nest because this brief permits no such network access.

## The march’s three holes

- The active-raid regression submits the seven-account buy and checks that the refusal leaves all account bytes and lamports intact. Deleting the guard at `trade.rs:41` made the test fail when the buy succeeded. Restoring the original bytes made it pass.
- `march_min_order` and `march_max_per_owner` limit admission in positive-delay raids. The first is the least input in lamports for a buy or base coin units for a sale. The second counts an owner's pending orders across both sides and all order kinds. Both default to zero. Eight one-lamport refund orders still fit under those defaults. The enabled fixture refuses them and admits honest orders. Existing zero-delay and default-limit config bytes are preserved.
- Signed position owners may close an empty position in a positive-delay generation. A stake, coins, quote credit or an order for that position holds it open. The current raid and queue are authenticated during a raid. After the buyer claims their refund, close returns the full 1,746,960 lamports of fixture position rent.
- The limited-config initializer measured 13,652 CU. Enabled buy admission measured 12,817 CU for the first queued buy and 11,659 CU for the second owner's buy, sale measured 8,600 CU, and change of sides measured 15,313 CU. Close measured 6,901 CU during a raid and 4,721 CU after settlement. The legacy cost fixture topped out at 41,528 CU, up from 41,447. The SBF artifact grew from 172,536 to 178,880 bytes.
- **his call.** The enabled test fixture uses 1 SOL in lamports for a buy, the same numeric floor in base coin units for a sale, and one marching order per owner. The inherited test delay is ten seconds. No live values were set. Native input units, a cap scoped to the raid, a close tied to positive delay, the optional nine-byte config tail and tag 17 are recorded in the lane receipt.

61 tests passed offline with freshly built SBF, including 26 in `embertide-war` and 35 in `poleis-wars`. The complete run ledger, compatibility checks, CU tables, wire details and verification limits are in the [lane receipt](../../../RECEIPT.md).
