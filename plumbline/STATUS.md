# Status

What is so now. This file is short and is rewritten in place (DECISIONS.md call 238):
nothing is added below, and what is no longer so is taken out. How it came to be so
is the calls in `DECISIONS.md` (`DECISIONS-index.md` finds one). Everything this file
held before 2026-11-26 is `STATUS-history.md`.

**As of 2026-11-26, the lead. It is not a release candidate and it is not tagged.**

## Where the work is

Patrick's ruling of 2026-11-26 (call 237): the repository was not acceptable as it
stood. It is cleaned, in the nine steps below, before any new function is built; then
the build goes on without asking him, and what he is sent is the manual, changed.

| Step | What | Where it stands |
|---|---|---|
| 1 | The lead's checks in the repository | **Done.** `tools/check.sh`, `tools/window-run.sh`, `tools/quiet.sh`, `tools/index.sh`. |
| 2 | Window demonstrations made sure; all forty-five run with nothing beside them | **Begun.** See "The window" below. |
| 3 | The manual; `demos/captured/` let go | Not begun. It is made from the pictures of step 2's run. |
| 4 | The documents put right | **Done but for one**: `TRY-IT.md` stands until the manual takes its place. Three lists are now written by tools and checked by the commit hook (`DECISIONS-index.md`, the README's demonstrations, the table in `LICENSES.md`). |
| 5 | The earlier program off the branch; the commit hook BuildOS's own | **Done.** It is on `main`, whole. |
| 6 | The window takes every kind of file the command line takes | Not begun. |
| 7 | The eleven files over 2,000 lines cut | Not begun. |
| 8 | The name BuildOS through the code | Not begun. Last, alone. |
| 9 | Items 9 to 12 judged again | Not begun. |

## What was run, and on which commit

- **The command line's side, on b6dc2ab** (the lead's own run, 2026-11-26): 1,344 tests
  of the six crates pass (the model 759, the command line 94, the workbook 7, the BIM
  file 131, the roof 122, the plans 231); all 69 command-line demonstrations pass and
  none says FAIL; the three gates, the port map and the licence check pass; the sample
  whole house (`plumbline sixteen`) is 880d9fd3c364b5f7, as it has been, and prices at
  $393,981.66.
- **The window's own tests, on the head with the double-click repair**: 109 pass.
- **`tools/committed.sh` passed on 1edb650** (108 test runs from a clean copy). It has
  not been run since; it is owed before the next push that changes code.
- After the earlier program left the branch, the roofing crate's 122 tests were run
  again and pass (three of them read two of its files, now kept byte for byte in
  `crates/roofing/tests/engine/`). The two rewritten gates were each proved red and
  green (`tools/gate-language.sh --self-test`; a socket and a network crate for
  `tools/gate-egress.sh`).

## The window

**Not proven on the head.** The window builds on the head and its 109 tests pass.
Of the forty-five window demonstrations, the lead's last runs (2026-11-26, on the
build before the head, with builds running beside them) gave: `screensw` passed;
`twoplansw` failed twice at its step 7 (a double-click in a plan selected and did not
fly); `tradesw` failed twice at a different click each time; the other forty-two
were not run. Ten had passed on the build before that. **A repair of the
double-click is in the working copy, not committed**: the demonstration's two presses
were 0.75 s apart and the program tells a double-click inside half a second of its own
clock; the presses now go 0.4 s apart, and the model window's double-click takes the
same one function as the other two (`nav::again`). It is committed when `twoplansw`
passes in the lead's own run with nothing beside it.

**The rule that was broken, and is now held by a tool.** No build runs on the machine
while a timed window demonstration runs. On 2026-11-26 the lead ran builds beside them
in the idle class; `navw`, `tradesw` and `twoplansw` lost clicks or time, and a day's
window evidence with them. `tools/window-run.sh` holds a lock and waits for cargo to
be gone; every other cargo command goes through `tools/quiet.sh`.

**What the window cannot do that the command line can** (step 6): bring in a CAD
drawing (DXF), a TIFF, a scanner's own files (LAS, LAZ, E57), the measured marks on a
PDF, a maker's model, the database, the field's sheet; write out the issue list (BCF),
the house as glTF, the field's sheet, the database, the cabinet rules. It brings in
PDF, PNG, JPEG, CSV, XLSX, IFC and PLY.

**Owed to the window from work already in the tree**: its three counts of the chest
(48) become 57 (`cabinetsw`, `turnw`, `itemlibw`; call 235); a cabinet made by a rule
says so; a starter shape is still drawn through the program's list of forms and a
washer's door is square (call 235, points 4 and 5).

## The release candidate: the thirteen, as they stand

None is complete with evidence on one commit. "Judged" is by an agent that did not
build it.

| # | Item | Where it stands |
|---|---|---|
| 1 | A clean clone installs on Linux from the README in one pass | A clean copy of 1edb650 builds and passes. The README's steps were last walked on a fresh system on 2026-11-22. |
| 2 | Every import works, each file picked from a list | Every kind is read at the command line. The window takes seven kinds (step 6). |
| 3 | A real residential plan PDF opened, scaled, traced; the 3D house builds | `firstw` does it on a public-domain plan; not run on the present build. |
| 4 | Clashes show their certainty; the VIF list is sorted by dollars | So at the command line: 11 in the sample house, 5 findings and 6 to verify; 85 questions, the most dollars first. `clashw` not run on the present build. |
| 5 | A typed or lasered measurement makes a value Known; the clash clears; the price moves in the second window | `fieldw` and `moneyw` passed on earlier builds; not run on the present one. A sheet filled in the field comes back by file (call 234). |
| 6 | A price per SF or LF grows into line items and rolls back up | `d4` passes; `d4w` not run on the present build. |
| 7 | The estimate sheet: stretch-out, labor per line, overhead and profit, three suppliers | `d5` passes; `d5w` and `pricew` not run on the present build. |
| 8 | xlsx with live formulas, CSV, IFC4 passed by IfcOpenShell; three lenses | So at the command line (`export`; IfcOpenShell 0.9.0 passes the file). Not all of it from the window. |
| 9 | A judge lists every word on every screen; none is a software term | **Failed three times on 2026-11-16. Not judged since.** The language gate passes. |
| 10 | A veteran with no software background completes a whole-house takeoff from a PDF | Passed once, on d20ac60 (2026-11-16). The window has changed much since. To judge again. |
| 11 | The visual judge passes every screen against `VISUAL.md` | **Failed three times on 2026-11-16. Not judged since**; the glass and the new movement came after. |
| 12 | A fresh judge cannot break the money path | Failed on 2026-11-16. Window judges of 2026-11-24 and 2026-11-25 found no wrong money; the item has not been judged whole since. |
| 13 | A hand estimate in `refs/` reproduced, every difference explained | **Pending**: there is none in `refs/`. Only Patrick can place one. |

## What is not right

- The window is behind the command line and unproven on the head (steps 2 and 6).
- Eleven source files are over 2,000 lines (step 7): `crates/world/tests/world.rs`
  6,713; `crates/cli/src/main.rs` 4,310; `crates/app/src/panels.rs` 4,017;
  `crates/world/src/building.rs` 3,944; `crates/app/src/scene.rs` 3,248;
  `crates/plans/src/found.rs` 2,930; `crates/ifc/src/lib.rs` 2,852;
  `crates/world/src/editing.rs` 2,666; and three more.
- The repository's history is 1.76 GB, most of it pictures. History is not rewritten
  (call 237). From step 3 pictures are not kept in the repository and the README's
  clone line takes only the newest commit.
- The folder, the crates and the two commands are still named plumbline (step 8).
- Neither wall finder turns a real phone scan, or a crowded plan, into walls by
  itself: a person traces and the finder helps.
- A double-click is told by the program's own clock, and a demonstration's two presses
  must fall inside half a second of it. `twoplansw` was repaired for that;
  `navw`'s north arrow still sends its presses 0.6 s apart and passes only where a
  frame is slow. It is put right in step 2.
- Designed and not built: an issue list (BCF) read back; a mesh brought in; glTF with
  each thing as its own shape; the sixteen division harnesses (`PROPOSALS.md` 21).

## What only Patrick can give (nothing waits on it)

A sanitized hand estimate in `refs/` (item 13); a sanitized PDF with his own measured
marks (`QUESTIONS.md` Q36); a real LAZ and a real E57, sanitized; the production
ruling, after the tag.

## How the work is run

`CLAUDE.md` has the rules and the seats. `handoff/README.md` has how a worker and a
judge are briefed, with the contracts as patterns. Two working copies at most are
built in at once, and never while a window run holds the lock.
