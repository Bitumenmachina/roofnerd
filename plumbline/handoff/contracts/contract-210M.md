# Contract 210M: another trade's model into a job that has a building, the model's side (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 210" (points
1 to 5 are yours; 6 is the window's), "### Call 209" point 4 and its "as built",
"### Call 206" point 3, "### Call 202"; plumbline/STATUS.md "Another trade's model
into my job: a demonstration"; `crates/ifc/src/lib.rs` (the reader: where a thing
is brought in, where it is refused for lying over what the job has, how a level is
matched), `crates/world/src/building/clash.rs` (the clash list and its kinds).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-210M-report.md. Commit nothing, push nothing.
When done: `git add -N . && git diff --binary -- plumbline > SCRATCH/review/call210M.patch`.

## Before any code, in your report
1. BIMCraft (`/home/user/ipedrax/bimcraft`): how its import chooses what to bring
   in, if it does (`grep -rn "filter\|only\|categories\|disciplines" crates/core/src/import.rs crates/core/src/ifc.rs | head -60`,
   then read). Port what fits; `PORTMAP.md` either way; portmap must pass.
2. What the reader does today, step by step, with the demonstration's model
   brought into the sample house: what is refused, by which check, and what
   would come in if that check were passed over for walls and floors.
3. The word for the choice at the command line (extend `import ifc`'s own words;
   the lead's working answer is `--only runs`) and the function the window will
   call, with what it takes and gives back.
4. How a run or a thing is put on a level when its own level in the file is at a
   height the job does not have (point 2): show the rule on one case by hand.
5. The clash kinds and how the count "before and after" will be taken (the same
   function `plumbline clashes` uses, called twice: no second way of counting).

## What is built (call 210, points 1 to 5)
- The choice, the levels, the reply with what was not asked for by kind and with
  the clashes made, each worked by the model. Everything brought in by the change
  a tool makes, Assumed, sourced to the file.
- Tests FIRST, run red, quoted: the demonstration's six steps on the model; a
  model with nothing but walls brought in as runs only (nothing comes in, said);
  a run on a level at a height the job lacks; a fixture connected to a run in the
  file (they come in connected); a model with two systems.
- `demos/ifc-runs.sh` (new) to the six steps; its row in `README.md`.
- **Every existing expectation that moves is listed before one is changed**; one
  moves only where point 1 of the demonstration's refusal gained its words.

## Rules
- Files you may touch: `crates/ifc/src/`, `crates/ifc/tests/`, `crates/world/src/`
  only where the reader needs a function of the model opened to it (say which; no
  figure's meaning changes), `crates/cli/src/`, `demos/ifc-runs.sh`, existing tests
  and demonstrations only under the rule above, `README.md`, `PORTMAP.md`. Nothing
  in `crates/app/src`.
- The saved job's format does not change. IfcOpenShell is run, never linked. No
  number from a real job.
- Build ONLY the command-line crates (`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
  nice -n 19 cargo ... -p plumbline-world -p plumbline -p plumbline-plans
  -p plumbline-sheets -p plumbline-ifc -p plumbline-roofing -j2`), tests one crate
  at a time, never with no `-p`. NEVER run a demonstration without `PLUMBLINE_BIN`
  set. Check `df -h /` before each build; under 3 GB free, stop.
- Everything in the foreground; no timers or background runs left when you hand
  back. At the end run every command-line demonstration and search their words
  for FAIL and FAILED as well as reading how each ended.
- The three root gates, `plumbline/tools/licenses.sh` and
  `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
The report has the five "before any code" answers; the new tests pass (red runs
quoted); `demos/ifc-runs.sh` passes; every command-line crate's tests and every
command-line demonstration pass or are on the list of expectations moved; no FAIL
in their words; the gates pass.
