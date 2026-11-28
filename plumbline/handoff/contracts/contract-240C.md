# Contract 240C: the estimate window at three depths (window builder)

**Not issued until part B1 (contract 240B1) is accepted; the lead names the commit.**
You are the one window builder, in /home/user/roofnerd. Read `CLAUDE.md` whole, then
`plumbline/DECISIONS.md` calls 240, 242 (point 3) and **243 (every point)**,
`demos/estimatew.md` (your demonstration, written first), `VISUAL.md` sections F and
G, `research/judgment-design-1126.md` section 7, and `crates/app/src/sheet.rs` whole
with `crates/world/src/view.rs` (`by_item`, `ItemTotal`, `SheetLine`). Notes as you go
to `SCRATCH/review/W-240C-report.md`, after every run. Commit nothing, push nothing.

## Before any code, in your report

1. `demos/estimatew.sh` written and run red on the tree as B1 left it: each numbered
   check with what it gets today.
2. **Call 243, point 3**: is an item of `by_item` what `plumbline set <file> <item>
   price` prices by name? Work "Wall framing, 2x6 at 16 in" on the sample house both
   ways at the command line (the lines it says it prices against the item's eight
   places) and say plainly: the same road, or not. Build nothing on it until the
   lead answers.
3. Every control of the estimate window that a demonstration names (`grep 'sheet "'
   demos/*.sh` and the names built in the scripts), and for each whether it is in
   view at rest after this part or reached by a way. None is edited.
4. How long the map takes to write with the ways of 432 lines' fields in it, against
   today (a figure). If a frame is slower by more than a tenth, say so and stop on
   that point.

## What is built

Call 243, points 1 to 8, as written. No figure is worked in the window. Every
control keeps its name on the map. The saved job is byte for byte what it was; what
is open is kept in the screen's memory only.

## What must not change

No number, no meaning, no file format, no word a builder would not say. No
expectation of an existing demonstration is edited by you: where one fails because
this part moved what it measures, stop on that one, write what it expects, what it
gets and a picture's name, and go on. Files you may touch: `crates/app/src/*.rs`,
`demos/estimatew.sh`, `demos/window.sh` (only to add a helper), `README.md` (a row).

## How it is run

Every cargo command through `tools/quiet.sh` with `CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 -j2` and the `CARGO_TARGET_DIR` the lead names; every
window session through `tools/window-run.sh`; long commands in the background;
nothing left running at hand-back. Look at every picture as an estimator who reads
this window for hours: is the price found at once, is a division's figure read
without hunting, is anything said twice. Two pictures at 880 by 960 (at rest; an
item opened and a price being typed) in `SCRATCH/review/240C/`. If a tool asks a
permission you do not have, stop and say so; do not ask the lead to do it.

## Done when

`cargo test -p plumbline-app` passes (the count); `estimatew` passes every check;
`quietw` line 16 passes; the eleven demonstrations of check 11 pass by
`tools/window-run.sh` on your last frozen build, or stand in the report with what
moved; the three gates pass; the report has the four answers, the results, the two
pictures and what is not right.
