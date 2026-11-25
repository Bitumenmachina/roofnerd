# Contract 204: a corner board as tall as its wall at that corner; three things a model leaves out (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 204" (every
numbered point), with "### Call 192" (the gable and its top line), "### Call 194"
(`corner_board_length_above_<entry>`) and "### Call 202" before it; plumbline/STATUS.md
"A corner board as tall as its wall at that corner: a demonstration" (the figures
your work comes to); `crates/world/src/trim.rs` and `trim/lay.rs` (the corner
boards' figure and pieces), `crates/world/src/gable.rs` (the wall's top line: use
it, work nothing of it again), `crates/ifc/src/lib.rs` and `write.rs`.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-204-report.md. Commit nothing, push nothing.
Do part 1, write `SCRATCH/review/call204-trim.patch`
(`git add -N . && git diff --binary -- plumbline > ...`), then part 2 and
`SCRATCH/review/call204.patch` holding both.

## Part 1: the corner board (call 204, points 1 to 4)
**Before any code, in your report**: (a) run `plumbline gable`, `trim` and
`trim --pieces` on the three houses of `demos/gable.sh` and set what the program
says today beside the lead's hand figures in STATUS.md; where the model's top line
at a corner is not the lead's figure, say which is right and stop on that point;
(b) list every test and demonstration, the window's too (`grep -ln "corner" demos/*.sh crates/*/tests crates/app/src`),
that prices or prints a corner board on a wall with a gable, and the figure each
holds today and will hold. (c) BIMCraft: search all of it for a corner board or a
trim piece that follows a wall's profile; port what is there; `PORTMAP.md` either
way; `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass.
**Built**: points 1 to 3. Tests first, run red, quoted. `demos/gable.sh` and
`demos/trim.sh` extended to the seven steps. **Every expectation that moves is
listed before one is changed**, and moves only by the figures of the STATUS.md
section or what follows from them by arithmetic you show; any other movement is
NOT edited and is handed back. For window demonstrations you cannot run
(`gablew.sh`, `gableendw.sh`, `trimw.sh`): list each expectation that will move,
change it under the same rule, and say plainly none was run.

## Part 2: a model (call 204, points 5 to 7)
**Before any code**: time `import ifc` of the sample house's own file three times
and say where the time goes (a profile, or timed stages: say how you measured);
say what a beam and a column are in the files the program reads and writes today.
**Built**: points 5 to 7, each thing brought in by the change a tool makes;
`demos/ifc-whole.sh` extended; the file still passes IfcOpenShell (run it, never
link it); expectations listed before any is changed (counts of what a file holds
and of what is left out will move: say each).

## Rules
- Files you may touch: `crates/world/src/` (the trim's figure and pieces; only
  what the reader needs opened of a change a tool already makes), `crates/world/tests/`,
  `crates/ifc/src/`, `crates/ifc/tests/`, `crates/cli/src/` (what a verb prints),
  `demos/gable.sh`, `demos/trim.sh`, `demos/ifc-whole.sh`, other existing tests and
  demonstrations only under the rule above, `README.md`, `PORTMAP.md`, `NOTICE`
  (only if something is ported). Nothing in `crates/app/src`.
- `demos/trim.sh` step 7 holds the sample houses to `demos/trim-before.sha256`: it
  must still pass unchanged (the sample has a hip roof). If it does not, stop.
- The saved job's format does not change. No rounding rule, no status rule other
  than point 2. No number from a real job.
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
The report has the "before any code" answers of both parts; the new tests pass (red
runs quoted); `gable.sh`, `trim.sh` and `ifc-whole.sh` pass; every command-line
crate's tests and every command-line demonstration pass or are on the list of
expectations moved, each with its arithmetic; the gates pass.
