# Contract 233M: starter shapes and items for appliances and small devices (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 233"
(every numbered point), "### Call 198", "### Call 203" and "### Call 205" (a shape
is rows of a table: how they are written, checked and changed), "### Call 211"
(where an item connects; how a starter place is sourced), plumbline/STATUS.md
"Starter shapes and items for appliances and small devices: a demonstration", and
in the code: `library/shapes.csv`, `library/items.csv`, `library/connections.csv`,
`crates/world/src/building/forms.rs` and `shape.rs` (the forms a block can be; how
a shape's rows become solids; how a thing set down finds its shape),
`crates/world/src/tables.rs` (the tables' readers and their checks),
`demos/shapes.sh`, `demos/connections.sh`, `demos/database.sh`,
`research/connection-places.md` (how a published size was cited last time).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-233M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call233M.patch`.

## Before any code, in your report
1. How a thing set down finds what it is drawn as today (by its item's "Drawn
   as"; by a word in its name; else a box), file and line, and so exactly which
   things of the sample whole house would be drawn differently once the new
   shapes exist, and whether any solid the clash test uses would change. If one
   would, STOP on that thing and hand it back; go on with the rest.
2. **Every place a test or a demonstration counts** the starter shapes, their
   rows, the starter items, the rows of where items connect, or the database's
   tabs (`grep -rn` for the figures 13, 48, 49, 50 and the tables' names through
   `crates/*/tests`, `crates/*/src` test modules and `demos/`), each with file,
   line, the count it holds and the count it will hold. **That list is the whole
   of what you may move in existing expectations**, each with the comment
   "call 233: the starter chest grew".
3. For each new item, its width, depth and height with the published source
   (search the web with plain trade words, never a person's or this project's
   name; a standard's own page or a maker-neutral reference; the address and
   the line quoted in `research/device-sizes.md`). A size you cannot source is
   Unknown in the table and is said.
4. For each new shape, its blocks drawn on paper first: a small table of its
   rows with what each block is, so the lead can read the shape without running
   anything.
5. BIMCraft at /home/user/ipedrax/bimcraft: `grep -rln -i
   "receptacle\|outlet\|refrigerator\|dishwasher\|luminaire\|light_fixture" crates`
   and read the whole list; say what starter families it has for these and
   whether any of their proportions are worth porting (record in `PORTMAP.md`
   and `NOTICE` if anything is).

## What is built (call 233, points 1 to 6, every clause)
Tests FIRST, run red, quoted: every new shape true to its item's sizes at two
sizes (the test of call 203 that holds the thirteen is the pattern); the seven
items' sizes unmoved; the tables out and in byte for byte; the sample whole
house unmoved. `demos/devices.sh` (new) to the six steps in STATUS.md; every step
ends the script when the program does not say what the step expects; its row in
`README.md`.

## Rules
- Files you may touch: `library/shapes.csv`, `library/items.csv`,
  `library/connections.csv`, `crates/world/src/` and `crates/world/tests/` (code
  only if a form a shape needs does not exist: say which, and why no existing
  form will do), `crates/cli/tests/`, `demos/devices.sh`, `README.md` (a row),
  `PORTMAP.md`, `NOTICE` (only if something is ported), `research/device-sizes.md`;
  and, ONLY for the lines your answer 2 lists, existing tests and command-line
  demonstrations. Nothing in `crates/app/src`, no `demos/*w.sh`: where a window
  test or window demonstration counts the starter chest, list it for the lead
  with file and line and do not edit it.
- No price, quantity, rounding rule or status rule changes. Every new size is
  Assumed or Unknown, never Known. Nothing from a real job or a real maker's
  catalog: sizes are typical ones from a standard or a neutral reference, and
  no maker is named in the tables.
- The sample whole house does not move: `plumbline sixteen` (sha256 beginning
  880d9fd3c364b5f7) and its `estimate`, `clashes` and `vif` text held to copies
  made BEFORE your work.
- Every cargo command and every demonstration runs in the idle class
  (`chrt -i 0 nice -n 19 ...`). Before every cargo command and every
  demonstration wait while `SCRATCH/quiet.lock` exists; wait on no other lock.
  Only the command-line crates; for a crate with several test files run `--lib`
  and then one `--test <name>` at a time, deleting the test programs after each
  (the model's fifty together are over 6 GB). NEVER a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. `df -h /` before every
  build. Scratch files in a folder of your own (`mktemp -d -p SCRATCH`); delete
  only what you made.
- Search each demonstration's output for FAIL as well as its exit. The three
  root gates, `tools/portmap.sh` and `tools/licenses.sh` pass. Every word a
  person reads is a builder's or a trade's own (a receptacle, not an outlet
  socket; a plate; a register).
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the five "before any code" answers; the new tests pass (red runs
quoted); `demos/devices.sh` passes; every command-line crate's tests and every
command-line demonstration pass; the gates pass; the report lists every existing
expectation moved and every window-side one left for the lead, what could not be
sourced, and what is not right.
