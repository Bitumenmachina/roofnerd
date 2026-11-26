# Contract 226M: loose ends at the command line (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 226"
(every numbered point), "### Call 219, the model's side, as built" (points 5 and
6), "### Call 223, as built" (point 7), "### Call 225" (point 5), "### Call 211"
(what a refusal may not say), plumbline/STATUS.md "Loose ends at the command
line: a demonstration", and in the code: `crates/world/src/editing/trying.rs`
(`try_change`: how the job is put back today), `crates/world/src/editing.rs`
(`all_or_nothing`; the undo list), `crates/world/src/units.rs` (`feet_said`,
`inches_said`: how a length is said), `crates/world/src/database.rs`,
`crates/world/src/building/cabinet.rs` and the cabinet rules' table in
`tables.rs`, `library/cabinet-rules.csv`, `demos/tools-feel.sh`,
`demos/database.sh`, `demos/cabinet-rules.sh`.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-226M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call226M.patch`.

## Before any code, in your report
1. What a try costs today, in the build the tests use and in the build a person
   is given (`cargo build --release -p plumbline`, the command line only: watch
   the disk), on the sample whole house: five tries of a partition, each time
   printed, with where the time goes. Then how the job will be put back if it
   must be quicker, and why the three proofs still hold.
2. Every sentence the model and the command line can give that writes a length
   with decimals (`grep -n '\.2} ft\|\.2} in\|:.2}\s*ft\|:.2}\s*in' crates/world/src
   crates/cli/src -r`, and read the whole list): for each, whether it is a
   sentence (it changes) or a quantity on a line or a list (it stays), and the
   existing tests and demonstrations that hold its old writing, file and line.
   **That list is the whole of what you may move in existing expectations**, and
   each moves only by how the length is written.
3. The cabinet rules' table with "Stocked heights": the heading, what each
   starter row takes (the heights of the boxes the chest has for that code), how
   a table without the column reads, and that the starter 27 are still exactly
   what their rules make.
4. Taking a rule away: the verb's words, each of the three cases of call 226
   point 4, and what the reply says.

## What is built (call 226, points 1 to 6, every clause)
Tests FIRST, run red, quoted. `demos/loose-ends.sh` (new) to the six steps in
STATUS.md; every step ends the script when the program does not say what the step
expects; its row in `README.md`. `SPEC.md` where a table's headings change.
`PORTMAP.md` for every new public item (say where in BIMCraft you looked).

## Rules
- Files you may touch: `crates/world/src/` and `crates/world/tests/`,
  `crates/cli/src/` and `crates/cli/tests/`, `library/cabinet-rules.csv`,
  `demos/loose-ends.sh`, `README.md` (a row), `SPEC.md`, `PORTMAP.md`; and, ONLY
  for the lines your answer 2 lists, existing tests and command-line
  demonstrations, each edit with the comment "call 226: a length in a sentence is
  said as a builder says it". Nothing in `crates/app/src`, no `demos/*w.sh`.
- No other existing expectation is edited: one that fails for any other reason
  is handed back with file, line, the figure expected and the figure got.
- No price, quantity, rounding rule or status rule changes; no quantity on a
  line of the estimate or of a list changes its writing. The sample whole house
  does not move (`plumbline sixteen`: sha256 beginning 880d9fd3c364b5f7), and its
  `estimate`, `takeoff`, `vif` and `clashes` text is what it was (hold each to a
  copy made before your work, and say how you made it).
- Every cargo command and every demonstration runs in the idle class
  (`chrt -i 0 nice -n 19 ...`); no lock is waited on; only the command-line
  crates, one crate's tests at a time; NEVER a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. `df -h /` before every
  build; delete the test programs after every `cargo test`, and the release
  build's folder when you have your figures. Scratch files in a folder of your
  own (`mktemp -d -p SCRATCH`); delete only what you made.
- Search each demonstration's output for FAIL as well as its exit. The three
  root gates, `tools/portmap.sh` and `tools/licenses.sh` pass.
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the four "before any code" answers; the new tests pass (red runs
quoted); `demos/loose-ends.sh` passes; every command-line crate's tests and every
command-line demonstration pass; the gates pass; the report lists every existing
expectation moved, and says what is not right.
