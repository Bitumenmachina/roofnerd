# Contract 219M: how tools feel, the model's side (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 219"
(points 2, 5, 6 and 7 are yours; the rest is the window's) and "### Call 217" (what
it is for), plumbline/STATUS.md "How tools feel, the model's side: a demonstration",
`research/review-1125-window-behaviours.md` (what the window does today and where),
and in `crates/world/src`: `editing.rs` (`all_or_nothing`, `move_by`, `move_wall`,
how a door and a window are held on a wall and added: search `opening`, how a thing
is set down from an item: `place_item_said`), `editing/history.rs` (what an undo
step is), `building/clash.rs` (how the clash list is worked, and how a clash is
said), `building.rs` (`takeoff`, a wall's face less openings), `view.rs` (the
estimate's lines and names). SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-219M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call219M.patch`.

## Before any code, in your report
1. BIMCraft at /home/user/ipedrax/bimcraft: how it tries a script and takes it back
   (`crates/core/src/engine.rs` `script_dry_run`, `dry_run_tests.rs`), how it moves
   an opening along and between walls (`grep -rn "fn move\|host" crates/core/src/walls* crates/core/src/commands.rs | head -40`),
   how it copies and arrays (`grep -rln "fn copy\|array\|duplicate\|mirror" crates/core/src`),
   and whether it has a find (`find.rs` in `crates/ui-egui/src`). Say what is ported
   and what is ours; every new public item in `plumbline/PORTMAP.md`; `NOTICE` if
   anything is ported; `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` passes.
2. **Trying a change**: the one function the window will call ("try this change,
   tell me what it would make, what it would clash with and whether it would be
   refused, and leave the job as it was"), its arguments and what it gives back,
   and the proof that the job is byte for byte what it was after any try (the
   saved text before and after; the undo history unchanged; the next path the job
   would give out unchanged). Say what a try costs in time on the sample whole
   house for a wall, a run and an item (measure it), since the window will call it
   whenever the pointer rests.
3. How a door and a window are held today (on which wall, how far along, measured
   from where), every figure that depends on it (the wall's face less openings,
   casing, trim pieces, framing, the swing), and so everything a moved opening must
   carry with it. A figure you find and do not handle is named.
4. What a copy of each kind of thing must carry (type, sizes, cost lines, item,
   connections, work mark existing/new, openings of a wall) and what it must not
   (a path, a Known status, a join to a run). Say how each value of the copy is
   marked and what its source reads.
5. What can be found by name in a job today (levels, rooms, things set down, types,
   items, estimate lines, systems): where each name is, and the rule for the order
   of results. The words each result is said in.
6. Work the demonstration's ten steps by the code as it is and set your figures
   beside the lead's; where they differ say which is right and why.

## What is built (call 219, points 2, 5, 6, 7)
- **Tried and taken back**: the function of answer 2, and at the command line a
  way to try any change that makes or moves a thing and see the reply without the
  file changing.
- **An opening moved** along its wall and to another wall, refused where it would
  overrun or lie over another opening, saying by how much, in feet and inches.
- **A copy; a row of copies; a wall with its openings.** All or nothing where the
  model would refuse one of a row.
- **Find.**
- Verbs for each, with usage lines in a builder's words.
- Tests FIRST, run red, quoted. `demos/tools-feel.sh` (new) to the ten steps in
  STATUS.md, every step ending the script when the program does not say what the
  step expects; its row in `README.md`.

## Rules
- Files you may touch: `crates/world/src/` and `crates/world/tests/`,
  `crates/cli/src/` and `crates/cli/tests/`, `demos/tools-feel.sh`, `README.md`
  (rows), `SPEC.md` (only if a key is added: none should be), `PORTMAP.md`,
  `NOTICE`. Nothing in `crates/app/src`. No public function the window calls
  changes its arguments (`grep -rn` it in `crates/app/src` first); add beside.
- **No existing expectation is edited.** An existing test or demonstration that
  fails is handed back with what was seen.
- The saved job's format does not change. No quantity, price, rounding rule or
  status rule changes. A copy is never Known. Nothing from a real job.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates (`-p plumbline-world -p plumbline -p
  plumbline-plans -p plumbline-sheets -p plumbline-ifc -p plumbline-roofing`), one
  crate's tests at a time. Before EVERY cargo command and every demonstration,
  wait while the file SCRATCH/window-run.lock exists (a timed window run is going;
  check every 15 seconds); never create or delete it. NEVER run a demonstration
  without `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. Watch `df -h /`:
  under 5 GB free, stop and delete your own old test programs (literal absolute
  paths).
- Search each demonstration's output for FAIL as well as its exit.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Every word a person sees is a builder's; lengths in feet and inches as the
  program writes them; no coordinates in a refusal about a door.
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the six "before any code" answers; the new tests pass (red runs
quoted); `demos/tools-feel.sh` passes; every command-line crate's tests and every
command-line demonstration pass with none edited; the gates pass; the report says
what a try costs in time and what is not right.
