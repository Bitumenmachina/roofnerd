# Status

What is so now. This file is short and is rewritten in place (DECISIONS.md call 238):
nothing is added below, and what is no longer so is taken out. How it came to be so
is the calls in `DECISIONS.md` (`DECISIONS-index.md` finds one). Everything this file
held before 2026-11-26 is `STATUS-history.md`.

**As of 2026-11-28, the lead. It is not a release candidate and it is not tagged.**

## Where the work is

Patrick's ruling of 2026-11-26 (call 237): the repository was not acceptable as it
stood. It is cleaned, in the nine steps below, before any new function is built; then
the build goes on without asking him, and what he is sent is the manual, changed.

| Step | What | Where it stands |
|---|---|---|
| 1 | The lead's checks in the repository | **Done.** `tools/check.sh`, `tools/window-run.sh`, `tools/quiet.sh`, `tools/index.sh`. |
| 2 | Window demonstrations made sure; all forty-five run with nothing beside them | **Done.** The kit is in (20a15ea): a press waits until the window has drawn. The lead's own run of all forty-five, one at a time with nothing beside it, has ended: all forty-five pass ("The window" below). Two had failed on their own stale expectations, were put right (call 241) and pass. |
| 3 | The manual; `demos/captured/` let go | **Done, and it goes on**: `manual/index.html` (written by `tools/manual.py` from `manual/manual.md`, this file and `manual/pictures/`) is published to Patrick as one artifact, `https://claude.ai/artifact/DTAkAoTGMcTYEkVAXdHLYt` (to change it: `tools/manual.py take` the new pictures, `tools/manual.py build`, `tools/manual.py page <file>`, and publish that file with the pictures to the same address). Seventeen pictures, each stamped with the commit it is of: all seventeen are of the build of 20a15ea, from the lead's run. `demos/captured/` is no longer kept in the repository. |
| 4 | The documents put right | **Done.** Four lists are written by tools and checked by the commit hook (`DECISIONS-index.md`, the README's demonstrations, the table in `LICENSES.md`, the manual's page). `TRY-IT.md` is gone: the manual takes its place. |
| 5 | The earlier program off the branch; the commit hook BuildOS's own | **Done.** It is on `main`, whole. |
| 5b | **Design review, and the window laid out again** (calls 239, 240) | **Judged; being built.** A design judge on another model measured the screens and searched what shipping programs do (`research/judgment-design-1126.md`): Patrick is right, and it is now in figures. `VISUAL.md` section G has sixteen numbered lines; `demos/quietw.md` is the check, to be proved red first. Part A is contract 240A, with its builder now. Part B is designed (call 242): B1 the layout (the job's menu, the level and layers in the view's cluster, the tools in two levels), then B2 the files. Part C, the estimate window, is designed (call 243): the sixteen divisions, their items and an item's places as one table that opens; four tabs. The outside designer's prompt is `handoff/design-review-prompt.md`; one reading is put to Patrick (`QUESTIONS.md` Q37). |
| 6 | The window takes every kind of file the command line takes | **Designed (call 242), not built.** Bring in is one list of the job's files, each with what it is; Write out is one list of what can be written. What a file is, and the reading and writing of every kind, go into one crate the command line and the window both call (`demos/kind.md`, contract 242M); the window's sheets follow (`demos/filesw.md`, part B2). |
| 7 | The eleven files over 2,000 lines cut | **Eight are cut and accepted** (call 245): the model's, the plans', the BIM file's and the command line's, as a move proved line for line, with the whole check passing on the cut tree. The window's three (`panels.rs`, `scene.rs`, `tools.rs`) are cut after the layout work, which is rewriting them. |
| 8 | The name BuildOS through the code | Not begun. Last, alone. |
| 9 | Items 9 to 12 judged again | Not begun. |

## What was run, and on which commit

- **The command line's side, on the tree with the eight files cut** (the lead's own run, 2026-11-28, `tools/check.sh` with the port map): 1,344 tests
  of the six crates pass (the model 759, the command line 94, the workbook 7, the BIM
  file 131, the roof 122, the plans 231); all 69 command-line demonstrations pass and
  none says FAIL; the three gates, the port map and the licence check pass; the sample
  whole house (`plumbline sixteen`) is 880d9fd3c364b5f7, as it has been, and prices at
  $393,981.66.
- **The window's own tests, on the build of 20a15ea**: 110 pass.
- **`tools/committed.sh` passed on 2b2b231**, the commit that holds the cut (120 test
  runs from a clean copy of the commit, the window's crate among them). The commits
  after it change records and tools only.
- After the earlier program left the branch, the roofing crate's 122 tests were run
  again and pass (three of them read two of its files, now kept byte for byte in
  `crates/roofing/tests/engine/`). The two rewritten gates were each proved red and
  green (`tools/gate-language.sh --self-test`; a socket and a network crate for
  `tools/gate-egress.sh`).

## The window

**The lead's run of all forty-five, each alone, on the build of 20a15ea** (scripts of
9b656b1, with the two put right under call 241; `tools/window-run.sh`, nothing beside
it; ended 2026-11-28). **All forty-five pass.** Forty-three passed on their first
run. **Two failed twice, on expectations that had gone stale, not on the program**
(call 241): `quantityw` expected a wall's list of figures as it was before calls 185
and 192; `scanw`'s search for file words did not leave out a row call 191 added. Both
were put right and each then passed on its first run. (`firstw` was once started
with its plan not yet in place and stopped within the minute, twice; with the plan
there it passed on its first run.) The window's 110 tests pass. The layout work of
step 5b changes the window: its builder's runs, and then the lead's, are owed on the
new layout.

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

**Owed to the window from work already in the tree**: a cabinet made by a rule
says so; a starter shape is still drawn through the program's list of forms and a
washer's door is square (call 235, points 4 and 5).

## The release candidate: the thirteen, as they stand

None is complete with evidence on one commit. "Judged" is by an agent that did not
build it.

| # | Item | Where it stands |
|---|---|---|
| 1 | A clean clone installs on Linux from the README in one pass | A clean copy of 1edb650 builds and passes. The README's steps were last walked on a fresh system on 2026-11-22. |
| 2 | Every import works, each file picked from a list | Every kind is read at the command line. The window takes seven kinds (step 6). |
| 3 | A real residential plan PDF opened, scaled, traced; the 3D house builds | `firstw` does it on a public-domain plan and passed on the build of 20a15ea (the lead's run). |
| 4 | Clashes show their certainty; the VIF list is sorted by dollars | So at the command line: 11 in the sample house, 5 findings and 6 to verify; 85 questions, the most dollars first. `clashw` and `tradesw` passed on the build of 20a15ea. |
| 5 | A typed or lasered measurement makes a value Known; the clash clears; the price moves in the second window | `fieldw` and `moneyw` passed on the build of 20a15ea. A sheet filled in the field comes back by file (call 234). |
| 6 | A price per SF or LF grows into line items and rolls back up | `d4` passes; `d4w` passed on the build of 20a15ea. |
| 7 | The estimate sheet: stretch-out, labor per line, overhead and profit, three suppliers | `d5` passes; `d5w` and `pricew` passed on the build of 20a15ea. |
| 8 | xlsx with live formulas, CSV, IFC4 passed by IfcOpenShell; three lenses | So at the command line (`export`; IfcOpenShell 0.9.0 passes the file). Not all of it from the window. |
| 9 | A judge lists every word on every screen; none is a software term | **Failed three times on 2026-11-16. Not judged since.** The language gate passes. |
| 10 | A veteran with no software background completes a whole-house takeoff from a PDF | Passed once, on d20ac60 (2026-11-16). The window has changed much since. To judge again. |
| 11 | The visual judge passes every screen against `VISUAL.md` | **Failed three times on 2026-11-16. Not judged since**; the glass and the new movement came after. |
| 12 | A fresh judge cannot break the money path | Failed on 2026-11-16. Window judges of 2026-11-24 and 2026-11-25 found no wrong money; the item has not been judged whole since. |
| 13 | A hand estimate in `refs/` reproduced, every difference explained | **Pending**: there is none in `refs/`. Only Patrick can place one. |

## What is not right

- The window is behind the command line (step 6).
- **The window is busy and its tools are cramped** (Patrick, call 239). At 1600 by 960
  the panels and bars cover a little over half the window; about ninety controls are
  in view at once; eighteen tools sit in one row with labels near 10 px; Show is said
  twice; the Details panel repeats the estimate window. `VISUAL.md` now has the numbers (section G); the
  window does not yet meet them. Step 5b.
- Three source files are over 2,000 lines, all the window's (step 7):
  `crates/app/src/panels.rs` 4,017; `scene.rs` 3,248; `tools.rs` 2,330. Three
  functions are over 900 lines each and were not cut inside (call 245).
- The repository's history is 1.76 GB, most of it pictures. History is not rewritten
  (call 237). Pictures are no longer kept in the repository but for the manual's
  (one fixed name each, about 3 MB together), and the README's clone line takes only
  the newest commit.
- The folder, the crates and the two commands are still named plumbline (step 8).
- Neither wall finder turns a real phone scan, or a crowded plan, into walls by
  itself: a person traces and the finder helps.
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
