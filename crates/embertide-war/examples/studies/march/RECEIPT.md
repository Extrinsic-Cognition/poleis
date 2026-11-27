# How long an order marches · call B

Before the study there was no timing sweep in the examples. The zero-slot control executes immediately with zero pending orders. The new example executes 1,480 fills across 280 runs from 20 reference checkpoints, peaks at five pending orders, and adds zero production code paths, dependencies, or idle work. The first run of the final release binary took 0.26 s wall time. Two warmed runs were below the 0.01 s resolution of `time -p`.

## Measured

P&L and coin losses are totals across the 20 paired checkpoints. Front jumps are maxima across individual fills, including fills sharing a slot. The wait was the same for every fill in its row.

| Delay (slots) | Wait (slots / nominal ms) | Honest max front jump (bp) | Forced FIFO P&L (lamports) | Priority P&L (lamports) | Priority buyer loss (base units / bp) | Priority max front jump (bp) | Peak pending |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 0 / 0 | 997 | — | +8,950,181,394 | 14,271,723,122,258 / 220 | 916 | 0 |
| 1 | 1 / 50 | 997 | −987,654,380 | +8,950,181,394 | 14,271,723,122,258 / 220 | 916 | 3 |
| 4 | 4 / 200 | 997 | −987,654,380 | +8,950,181,394 | 14,271,723,122,258 / 220 | 916 | 3 |
| 16 | 16 / 800 | 997 | −987,654,380 | +8,950,181,394 | 14,271,723,122,258 / 220 | 916 | 5 |
| 64 | 64 / 3,200 | 997 | −987,654,380 | +8,950,181,394 | 14,271,723,122,258 / 220 | 916 | 5 |

An observer reacting to an accepted whale got behind both target orders at every positive delay. Forcing that observer to trade lost money in every checkpoint and cost the steady buyer zero coins. An observer who abstains earns zero. Zero delay has no pending accepted order to observe. Before-acceptance priority, when granted, let the same observer earn 8.950181394 SOL across the 20 checkpoints. The 220 bp buyer loss is 2.20% of the coin output across all 60 steady buys, rounded down to whole basis points.

**Pick, his call B.** Start with one slot if one slot gives the glass enough time to show a march. It had the same FIFO behavior and per-fill front movement as 4, 16 and 64, with the shortest wait and a peak of three pending orders in this workload. Increasing the delay preserved the profitable sequence whenever the observer had priority before acceptance. Ordering before acceptance is a separate decision. The study leaves the minimum visible march time open.

## What ran

The [Rust example](../../march_study.rs) replays “The war as before, now on the clock, in SOL” through settlement. The underlying [reference fixture](../../../tests/reference/reference-in-sol.json) has a 195-minute raid and a Crow victory decided at minute 150 after the Wolf's last stand. Each trial forks from the two curves after the reference minute's events and crank at minutes 0, 5, …, 95. The forks are independent. Their trades do not move later reference checkpoints.

The steady buyer spends 1 SOL at slots 0, 4 and 8. A whale buying the same house follows the steady buy in slot 8 and spends 4–8 SOL from a fixed SplitMix64 seed, `0x6a09e667f3bcc909`. Each placement and delay gets those same orders. The honest control runs them alone. In the forced FIFO case the 2 SOL observer buys after seeing the whale at the tail of the committed queue. In the priority case the observer sees the proposed batch and gets accepted first in slot 8. Both observer cases sell the entire position as soon as their purchase fills and let that sale march for the same delay. P&L is net sale proceeds less actual purchase cost, including both curve fees. The honest buyer's loss is the shortfall in coin output against its paired control at equal spend.

The example uses `Order::accept` and `land_on` for the integer delay, curve and fees. The harness processes due slots and preserves acceptance order within a slot. A slot is only an integer input there. The reference replay separately uses the law's raid minutes. There is no wall clock in the simulator.

## His calls and verification limits

- **his call A — front.** The study uses `floor(10,000 × attacker market cap / combined market cap)` and measures the absolute before/after difference. The draft leaves the production front open. Orders landing together could appear as a single larger jump on a glass that samples only once per slot.
- **his call B — visibility and timing.** The wait starts at acceptance and ends at scheduled execution. The nominal 50 ms per slot comes from the owner’s 6G-TA lineage. Real slot cadence, transaction acceptance, keeper or finality delay, propagation to a glass, and visible march time were outside this run. The current march program uses integer seconds. No slot-based on-chain change was made.
- **his call — what the observer can see and buy.** Before-acceptance priority is a modeled capability. The zero-slot priority row permits a same-raid sale for comparison. The program's zero-delay path refuses ordinary sales in a raid, so that row is a counterfactual and gives no estimate of alternate zero-delay exits. Buy limits, queue spam, transaction and rent costs, competing sellers, and order-flow changes during the observer's holding time were outside this workload. Only the first reference war supplied market checkpoints.
- **his call — workload.** The checkpoints, buy sizes, common house, and order cadence are study inputs. Larger orders, differently timed interest, price limits, and asynchronous keeper behavior could change the measurements. Here every buy and sale filled, zero orders refunded, and the peak remained below the eight-order queue cap.

## Verification and reproduction

Both test runs used the same command below. Each passed 28 tests, with 16 law tests, eight march tests, one reference test covering all four wars, and three example tests. The existing Python regeneration test was filtered out under the brief's language constraint. The Rust reference test still checked the stored fixture. Two final release runs produced byte-identical stdout. The sandbox rejected `nice`'s priority change on each invocation and still ran the commands. The default `rustfmt` failed to load its LLVM library, and toolchain 1.96 lacked `rustfmt`. Formatting was applied and checked with the installed 1.93.1 formatter. Builds and tests used Rust 1.99.0 offline. No live house, validator, app, or network request was used.

```sh
export CARGO_TARGET_DIR=~/.cache/seat-e216f56d/target-q-16-the-march-study
nice -n 19 cargo test --offline --locked --manifest-path embertide-war/Cargo.toml --tests --examples -- --skip the_copied_model_regenerates_the_reference
nice -n 19 cargo run --offline --locked --release --manifest-path embertide-war/Cargo.toml --example march_study
```
