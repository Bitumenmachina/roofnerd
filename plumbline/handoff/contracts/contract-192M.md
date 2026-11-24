# Contract 192M: the gable, the model's half (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 192" (every
numbered point), plumbline/STATUS.md "A wall under a rake rises to the roof: a
demonstration" (the figures your work comes to), `crates/world/src/roof.rs`,
`crates/world/src/building.rs` (a wall's figures: search `net_area`, `net_length`),
`crates/world/src/trim.rs` (how it finds the roof edge over a wall: you need the
same), and `library/sixteen-divisions.csv` (the starter outside-wall assemblies).
SCRATCH is SCRATCH.
Notes as you go to SCRATCH/review/W-192M-report.md. Commit nothing, push nothing.
When done, from the working copy's root:
`git add -N . && git diff --binary -- plumbline > SCRATCH/review/call192M.patch`.

## Before any code, in your report
1. Search ALL of BIMCraft at /home/user/ipedrax/bimcraft for a wall attached to a
   roof, a wall's top profile, a gable (`grep -rln -i "gable\|attach\|top_profile\|profile" crates apps`
   and read the whole list). Port what is there; record in `plumbline/PORTMAP.md`
   either way; `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass.
2. Say what kinds of roof edge the model has and how it tells which edge lies over a
   wall; how the high edge of a shed roof is typed; and whether the lead's rule
   (point 1: "not an eave") can be told from what the model holds. If it cannot, say
   what can, and stop on that point for a ruling.
3. Say what the model gives for the roof's height over each wall's line on the
   demonstration's house, against the lead's hand figures (9.5, 13.5, 17.5 ft). Do
   not bend the program to a hand figure: say which is right.

## What is built
- On a wall: the figure `gable_area` (SF) and its top as a line of points
  (`(feet along the wall's line, feet above the job's datum)`), as call 192 points 1
  and 2 say: worked out, never typed, Assumed where what they rest on is, each
  saying what it rests on; "none" with the reason under an eave or with no roof.
  The wall's list of figures (`plumbline show`, `takeoff` where a wall's figures are
  listed) says it. A formula on a cost line may name `gable_area`.
- `net_area` and every other existing figure are as they were.
- The starter outside-wall assemblies' lines for the outside of the wall priced on
  `net_area + gable_area` (point 3). Read the assemblies and list in your report
  which lines you take as outside and which as inside, before changing them. A
  wall with a stone wainscot keeps its `net_area - 3 * net_length` form with
  `+ gable_area` added to the siding.
- **Every existing test and demonstration expectation that moves is NOT edited.**
  Run the three crates' tests and every command-line demonstration; list each
  failure with the figure it expected, the figure it now gets, and the hand
  arithmetic that shows the new figure is the old one plus the gable; then STOP on
  those and hand the list back for the lead's ruling. (Pass `--no-fail-fast`; go on
  to finish everything that does not depend on the ruling.)
- A gable's frame is not counted: the wall's framing figures say so where the wall
  has a gable (a sentence, not a number).
- Tests FIRST, run red, quoted. A demonstration `demos/gable.sh` (new; command line
  only) to the figures in STATUS.md.

## Rules
- Files you may touch: `crates/world/src/` (the wall's figures, a new file for the
  gable, `formula.rs`/`derive.rs` only to name the new figure), `crates/world/tests/`
  (a new test file), `crates/cli/src/` (what a verb lists), `library/sixteen-divisions.csv`
  (the formulas of point 3 only), `demos/gable.sh`, `README.md` (a row),
  `PORTMAP.md`, `NOTICE` (only if something is ported). Nothing in `crates/app`.
- The saved job's format does not change: the gable is worked out, not saved.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only `-p plumbline-world -p plumbline -p plumbline-plans`. Watch
  `df -h /`: under 4 GB free, stop and delete your own old test programs (literal
  absolute paths). Do not run cargo before the time the starting message gives.
- Demonstrations with `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
The report has the three "before any code" answers; the new tests pass (red runs
quoted); `demos/gable.sh` passes; the list of every existing expectation that moved
is complete, each with its arithmetic, and none of them edited; the gates pass.
