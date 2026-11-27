# HIS ANSWERS ARE IN THE PARAMS: the two charters are his own words, and the seats' window is one day (Nov 23 2026, claude-f32e080d)

## His words, UTC

He was asked four questions in one dialog, in seat claude-f32e080d; his answers came back at 18:17:41:

| the question | the option he pressed |
|---|---|
| "What should the Dyson Society's and Nefertiti's charters say?" | "Your own words (Recommended)": his Sept 9 2026 15:56 line for the society, his Sept 18 2026 10:23 line for Nefertiti |
| "Which three agents hold Nefertiti's chairs?" | "Three new agents": their keys made at the sitting |
| "How many days can people take seats?" | "1 day" |
| "When someone spends their $WARM as credits, what happens to it on the chain?" | "Back to the mine (Recommended)" |

Before it, the sheet of mainnet's unnamed launch values (Canvas, the Claude page, `harmony-claude:widget-b690d6b1-4727-4367-924a-b568be54dc41`)
counted 40 placeholders in `poleis/press/params.mainnet.json`, not the 41 the brief said: one PLACEHOLDER was a word in the file's note.

## What changed

| | before (main 5759bc4c0e) | after |
|---|---|---|
| `genesis.charter` | PLACEHOLDER | `c1128b90…942ac`, the sha256 of `poleis/charters/the-dyson-society.txt` (160 bytes, his words) |
| `posiverse.charter` | PLACEHOLDER | `e7abb977…8de4e`, the sha256 of `poleis/charters/nefertiti.txt` (68 bytes, his words) |
| `floor.windowSlots` | PLACEHOLDER | 216,000 slots: one day at 400 ms a slot (Poleis takes 1 to 1,296,000) |
| the params' note | | names his answers and their stamp |
| placeholders left | 40 | 37 |
| `poleis/charters/` | the 12- and 8-article drafts of fc675337b2 | his two lines, one file each, and a README naming each file's sha256 |

Not here, on purpose:
- **The three agents.** Their keys are made at the sitting, so `posiverse.agents` stay placeholders until a `press` step makes them. The
  answer went to the seat building the agents' vote (claude-0733cf6f, `claude/hopeful-almeida-e6d0d3`), which owns that road.
- **$WARM back to the mine.** That answer went to the seat that built the mine's door and crank (claude-d276d500, `lane/the-mine-is-live`).
- **The market's judge.** This seat's pick, read from his Nov 22 2026 09:50:51 ruling ("the fucking entire tree perfect bicameralism"):
  the swarm reports a passed road's worth by its own sealed act. He has not pressed it, so `market.judge` stays a placeholder.
- **The other 33 picks** of the sheet (devnet's and the rehearsal's values, and three of his readings): he has marked none of them, so none
  is written.

## Cost

Two text files of 228 bytes, a README, three values and one note in a JSON file. No code, no JavaScript, no key made or read, nothing sent
to any cluster.

## Checked

`cargo test --offline` of `poleis/press` on this tree at nice 19: 74 passed, 0 failed, in 15 test binaries, among them the three that read
`params.mainnet.json` (`tests/params.rs`, `tests/seal.rs`, `tests/seat.rs`). Not run: a launch rehearsal (the params a rehearsal runs are
`params.rehearsal.json`, which this lane does not touch).

## Owed before the founding

The two charter files published where a buyer can read them before the founding (the launch plan's "published before"). Where is his call.
