# Contract 206: the command-line judge's findings fixed (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 206" (every
numbered point is yours), plumbline/research/judgment-cli-1125.md (the judge's notes:
each finding has the commands that show it; its files are in
SCRATCH/judges/cli-1125/, READ ONLY), plumbline/STATUS.md "The command line judged
(2026-11-25), and its findings fixed: a demonstration", and the calls each point
names (196, 199, 201, 203, 205, 197, 200, 202).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-206-report.md. Commit nothing, push nothing.
Write a patch after each of the three parts
(`git add -N . && git diff --binary -- plumbline > SCRATCH/review/call206-N.patch`,
N = 1, 2, 3, each holding all the work so far).

## The order, and how each is done
For each point: reproduce the judge's finding first on your own build of the tree
as it is, and quote what the program said; then a test, run red, quoted; then the
fix; then its step of `demos/judged-1125.sh` (new).
- **Part 1: points 1, 2 and 3**, in that order. They are the ones that are more
  than words.
  - Point 1: find every place a line is taken off a thing by a change (an assembly
    changed, an item priced as another thing, an assembly taken off) and make one
    function say what a person entered on a line; every such place uses it.
  - Point 2: say every place a shape meets an item (drawn as, an item's size, a
    block added or set, a table brought in when items already name the shape) and
    check at the items' own sizes in each. The three sizes stay for a table.
  - Point 3: before any code say what the reader does today with a level, where a
    room's figures come from when walls stand on two levels at one height, and
    what "the same height" will be. The reply's list of rooms that moved is worked
    by the model (the rooms' figures before and after), not by the command line.
- **Part 2: points 4 to 14.** Words and refusals. Each is a builder's words
  (`tools/gate-language.sh`).
- **Part 3**: every command-line crate's tests one at a time; every command-line
  demonstration; search their output for FAIL and FAILED; the gates, portmap,
  licenses.
- **Every existing expectation that moves is listed before one is changed**, with
  what it held and what it holds. One moves only where a reply gained the words a
  point asks for, or where point 3 puts a file's level on the job's level; a
  figure of money or quantity that moves any other way is NOT edited and is handed
  back.

## Rules
- Files you may touch: `crates/world/src/`, `crates/world/tests/`, `crates/ifc/src/`,
  `crates/ifc/tests/`, `crates/cli/src/`, `demos/judged-1125.sh`, existing tests and
  demonstrations only under the rule above, `README.md` (a row), `PORTMAP.md`
  (new public items: say where in BIMCraft you looked;
  `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass), `SPEC.md` (only
  where a point changes what a figure or a file means: say which). Nothing in
  `crates/app/src`: for each point say in the report what the window will now say
  or need, unchanged by you.
- The saved job's format does not change. No rounding rule and no status rule
  changes. No number from a real job.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates, one crate's tests at a time. Check
  `df -h /` before each build. Demonstrations with
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
- Everything in the foreground; no timers or background runs left when you hand
  back; do not hand back while something of yours runs.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
Each point has its finding reproduced, its red run and its step of the
demonstration in the report; `demos/judged-1125.sh` passes; every command-line
crate's tests and every command-line demonstration pass or are on the list of
expectations moved; no FAIL in their words; the gates pass.
