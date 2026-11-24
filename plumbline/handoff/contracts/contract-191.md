# Contract 191: the judge's findings fixed (command line only)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd (the window builder has it and runs timed window runs there).
Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 191" (every numbered point is
yours), plumbline/research/judgment-scale-plan-trim.md (the judge's notes: its
commands reproduce each finding; its sheets and scripts are in
SCRATCH/judges/scale-plan-trim/), and plumbline/STATUS.md "The judge's findings
fixed: a demonstration" (the figures your new steps come to). SCRATCH is
SCRATCH.
Notes as you go to SCRATCH/review/W-191-report.md. Commit nothing, push nothing.
When done, from the working copy's root:
`git add -N . && git diff --binary -- plumbline > SCRATCH/review/call191.patch`.

## How
- For EACH point: first reproduce the judge's finding on the program as it is and
  quote what it said; then a test that is red for that reason, quoted; then the fix;
  then the new step of the demonstration (`demos/planwalls.sh` for A and B,
  `demos/trim.sh` for C) to the figures in STATUS.md.
- Points A2, A3 and A4 change rules that existing tests and steps rest on (how wide
  a gap is an opening; how far apart walls of one drawing may be; when a sheet lays
  nothing unasked). **If any existing test or demonstration step fails under the new
  rule, do NOT edit it: stop, and write in your report which one, what it expected,
  what it now gets and why, for the lead to rule on.** Then go on with the other
  points. The same for any hand figure in STATUS.md the program's rule contradicts.
- After A1 to A4, run the three real city sheets again (scratch only: SCRATCH/real/pdf,
  with the jobs in SCRATCH/real/jobs188 as the lead ran them; nothing of them enters
  the repository) and say what each says now, above all the sheet with a floor plan
  and a roof plan: it must still lay nothing unasked.

## Rules
- Files you may touch: `crates/plans/src/drawn.rs`, `drawn_tests.rs`,
  `tests/drawn_pages.rs`, `crates/cli/src/found.rs` and `main.rs` (usage, the trim
  verb's list), `crates/world/src/plans.rs` (the scale's status and what is said of
  walls laid from a sheet), `crates/world/src/trim.rs` and `trim/lay.rs`,
  `crates/world/tests/call184t.rs` and a new test file, `demos/planwalls.sh`,
  `demos/trim.sh`, `README.md` (a verb's line). Nothing in `crates/app`.
- The saved job's format does not change. If point 8 cannot be done without a new
  field in the saved job, STOP on that point and say what is needed.
- No number changes other than those call 191 states (the laid LF of A1 and A2).
  Point 12 changes no number: the soffit's formula stays.
- Everything found is Assumed; nothing is Known on the program's say-so.
- `CARGO_INCREMENTAL=0 nice -n 19 cargo ... -j2` from `plumbline/`, only
  `-p plumbline-plans -p plumbline-world -p plumbline`. Watch `df -h /`: under 4 GB
  free, stop and delete your own old test programs (literal absolute paths).
- Demonstrations with `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
The three crates' tests pass (counts; each red run quoted); `demos/planwalls.sh` and
`demos/trim.sh` pass from the top; the gates pass; the report goes point by point
(1 to 13): reproduced, red, fixed, shown, or stopped on with why; and the real
sheets' figures before and after.
