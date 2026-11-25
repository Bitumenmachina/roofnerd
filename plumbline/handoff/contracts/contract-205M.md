# Contract 205M: a shape changed by hand, the model's changes (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 205" (points
1 to 5 are yours; 6 is the window's), "### Call 203" and "### Call 198" before it,
plumbline/STATUS.md "A shape changed by hand: a demonstration",
`crates/world/src/building/shape.rs` whole, `crates/world/src/editing.rs` (how an
item is changed: `item_size`, `rename_item`, what each gives back to be said),
`crates/world/src/tables.rs` (shapes in and out) and `research/worker-report-call198.md`.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-205M-report.md. Commit nothing, push nothing.
When done: `git add -N . && git diff --binary -- plumbline > SCRATCH/review/call205M.patch`.

## Before any code, in your report
1. BIMCraft (`/home/user/ipedrax/bimcraft`): how a family's form is added, changed
   and removed, and how a family is duplicated under a new name
   (`grep -rn "fn .*form\|duplicate\|fn .*family" crates/core/src/family.rs crates/core/src/*.rs | head -80`,
   then read). Port what fits; `PORTMAP.md` either way; portmap must pass.
2. What the model does today when a job's shape has a starter shape's name (the
   last worker says it takes the starter's place in the model): show it, and list
   every test and demonstration that relies on it, before point 3 forbids it.
3. The changes as you will add them: each one's name, what it takes and what it
   gives back to be said (follow `item_size`'s pattern: a thing the window and the
   command line can both say), and the verb or verbs of the command line that
   call them (extend one that fits; add one only if none does).

## What is built (call 205, points 1 to 5)
- The changes, each all-or-nothing, each checked as a shape is on the way in, each
  saying what follows (items drawn as it, things set down). Undo and redo carry
  them as they carry any change (say how the job's history takes them).
- Tests FIRST, run red, quoted: each of the demonstration's seven steps as a test
  on the model, and: a name already in the job; a block number that is not there;
  a figure that is not a formula; a figure in a name the item does not have
  (`length`); "There when" set and unset; a shape no item is drawn as.
- `demos/shapes.sh` extended to the seven steps; its row in `README.md`.
- **Every existing expectation that moves is listed before one is changed**; one
  moves only where it relied on a job's shape taking a starter's name.

## Part 2, after the above is whole: what is set down on a level, counted by the model (call 197, point 10)
The sheet window lists the things set down on each level by kind, and counts them
itself from the model's list (`crates/app/src/sheet.rs`, `set_down`, with a comment
saying the count is owed to the model). A count is a figure and is the model's.
- Add to a level's totals (`building::storey_totals`, what `takeoff` prints for a
  level) how many things are set down on it, by kind (equipment, fixtures,
  furnishings), with the names, and how many of them are priced on nothing. Each
  as sure as the rest of a level's counts.
- `plumbline takeoff` prints it under the level, in the words the sheet window
  uses (read them in `crates/app/src/sheet.rs`; do not edit that file).
- Tests first, run red, quoted: a level with nothing set down says none; two
  vanities and a water heater; a thing in a cabinet run is not counted twice; the
  same job's count by the model equals what the window's grouping gives (state the
  window's rule from its code and test the model against it).
- Say in the report the call the window will make in place of its own grouping.
- **Every expectation that moves is listed before one is changed**: `takeoff`'s
  output gains lines, so demonstrations that hold its whole text will move; say
  each and move only those.

## Rules
- Files you may touch: `crates/world/src/` (the shape, the editing changes, the
  tables' refusal of a starter's name), `crates/world/tests/` (a new test file),
  `crates/cli/src/`, `demos/shapes.sh`, `SPEC.md` (only if what the saved job
  holds changes: it should not), `README.md`, `PORTMAP.md`, `NOTICE` (only if
  something is ported). Nothing in `crates/app/src`: say in the report, for each
  change, the call the window will make and what it gets back.
- The saved job's format does not change. No figure of any cost line changes. No
  number from a real job.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates, one crate's tests at a time. Check
  `df -h /` before each build. Demonstrations with
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
- At the end run every command-line demonstration and search their words for FAIL
  and FAILED as well as reading how each ended.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it. No timers left running when you hand back.

## Done when
The report has the three "before any code" answers; the new tests pass (red runs
quoted); `demos/shapes.sh` passes; every command-line crate's tests and every
command-line demonstration pass or are on the list of expectations moved; the
gates pass.
