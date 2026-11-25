# Contract 211M: where an item connects, the model's side (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 211"
(every numbered point), "### Call 195" (step D), "### Call 202", "### Call 203" and
"### Call 205" (how a shape's rows were made data, and how they are changed and
checked: this job follows that pattern), plumbline/STATUS.md "Where an item
connects: a demonstration" (the figures your work comes to), and
`research/survey-connections.md` (where each thing is today, file and line: check a
line before relying on it). SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-211M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call211M.patch`.

## Before any code, in your report
1. BIMCraft at /home/user/ipedrax/bimcraft: read `crates/core/src/family.rs`
   (`Connector`, `check_connectors`, the starter families' `"connectors"`),
   `crates/core/src/model.rs` (`component_connectors`), `crates/core/src/fixtures.rs`
   and `connector_tests.rs`. Say what is ported (which connections each starter
   fixture has, which way each goes and faces, the rules that apply here, the
   tests' ideas) and what is ours. Record every new public item in
   `plumbline/PORTMAP.md`; `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh`
   must pass; `NOTICE` if anything is ported.
2. Work the demonstration's steps 2 and 6 by the code that turns a shape's block
   into a place on a thing set down (find it; quote it), and set your figures
   beside the lead's. Where they differ say which is right and why, and build on
   the program's own way: the lead's figures are then corrected in your report,
   not the program bent to them.
3. Say exactly how a run is known to be joined to a connection today (by where its
   end stands? by something stored?), with the code quoted, and so what "joined"
   will mean under point 6 without a second way of knowing it.
4. List every place a `Connector` or an `Item` is written or read (the saved job,
   SPEC.md, the tables, the BIM model, `takeoff`, the clash list, `placed_follow`)
   and say what each needs. A place you find and do not handle is named.

## What is built (call 211, points 1 to 9, every clause)
- Rows on an item; the table in and out (`library/connections.csv` with the
  starter rows; `import connections`, `export connections`; brought in with a
  chest that lacks them it changes nothing); verbs to add, change and take away
  one row by hand (`plumbline connection <job> <item> add|set|take ...`, replying
  in the item's own size and in inches), and `items` showing how many each item
  has and how many starter items have none.
- The checks of point 3, each with a test and a refusal a builder can read. No
  refusal speaks in feet with decimals, in coordinates, or of a "small size".
- Set down (point 4), follow (point 5), joined (point 6), the clash's added words
  (point 7). One arithmetic: across, back and up become a place by the shape's own
  function; a test holds a block and a connection of the same figures to the same
  point, turned and not.
- The saved job (point 2): a job with no rows and no facing is byte for byte what
  the program wrote before (a test against a job written by the tree you start
  from; say how you made that baseline). SPEC.md says the new keys.
- The starter rows (point 8). For each of the four fixtures, each row's place and
  size from a published US rough-in you can name: search the web with plain trade
  words (never a person's or this project's name), give the address and quote the
  line in `research/connection-places.md`. A place you cannot source is not
  written; say which. Every starter place and size is Assumed, "a typical
  rough-in; the product is not chosen".
- The sample house does not move (point 9): `plumbline sixteen` byte for byte,
  held by `demos/connections.sh` step 10 against a file written before your work.
- Tests FIRST, run red, quoted. `demos/connections.sh` (new) to the ten points in
  STATUS.md; every step ends the script when the program does not say what the
  step expects; its row in `README.md`.

## Rules
- Files you may touch: `crates/world/src/` and `crates/world/tests/`,
  `crates/cli/src/`, `crates/ifc/src/` (only if a facing must be carried; say why),
  `library/connections.csv`, `demos/connections.sh`, `README.md` (rows), `SPEC.md`,
  `PORTMAP.md`, `NOTICE`, `research/connection-places.md`. Nothing in
  `crates/app/src`.
- **No existing expectation is edited.** Run every command-line crate's tests with
  `--no-fail-fast` and every command-line demonstration; an existing test or
  demonstration that fails is handed back with what was seen (file, line, the
  figure expected, the figure got), not changed. Nothing here should move one.
- No rounding rule, no status rule and no other figure's meaning changes. Nothing
  from a real job. A connection is never Known on an item's or a file's say-so.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates (`-p plumbline-world -p plumbline -p
  plumbline-plans -p plumbline-sheets -p plumbline-ifc -p plumbline-roofing`), one
  crate's tests at a time. NEVER run a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug` (without it the script builds
  the window's crates and fills the disk). Watch `df -h /`: under 4 GB free, stop
  and delete your own old test programs (literal absolute paths).
- Search each demonstration's output for FAIL as well as its exit.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Work in the foreground; leave no timers or background jobs behind when you hand
  back. Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do
  not have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the four "before any code" answers; the new tests pass (red runs
quoted); `demos/connections.sh` passes; every command-line crate's tests and every
command-line demonstration pass with none edited; the gates pass; the report says
what is not right.
