# Contract 234M: the field's measurements brought back by file (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md ("The loop"; "Flag, never fabricate"; step 4 of
"Order of work"), then plumbline/DECISIONS.md "### Call 234" (every numbered
point), the calls on field entry it rests on (search DECISIONS.md for "Field
entry" and "laser"), plumbline/STATUS.md "The field's measurements brought back by
file: a demonstration" and the section "Field entry" it names, and in the code:
`crates/world/src/vif.rs` (what a question is, what it rides on, its order),
`crates/world/src/export.rs` (the printed sheet, `export vif`),
`crates/cli/src/main.rs` (`vif`, `export vif`, and the whole road of a typed
measurement: `set ... --status Known`, what it replies, how a length a laser types
is read), `crates/world/src/editing.rs` (`all_or_nothing`), `demos/field.sh`,
`demos/field-job.sh`, `demos/vif.sh`.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-234M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call234M.patch`.

## Before any code, in your report
1. The road of a typed measurement, file and line: the function that sets a
   value Known, what follows it (what moves, which clash is confirmed or
   cleared), and every sentence it replies. **The sheet's rows go down this
   road and no other**: say how, and how "all of it or none of it" is kept
   across many rows.
2. What a question of the verify-in-field list is made of, and so the rows it
   becomes: for each kind of question (a size, what a thing is, where it is, a
   clash "measure either one", a price), which values are measurable, what the
   row says to measure in a builder's words, and which kinds have no value a
   tape or a laser can answer (named: they are not rows, and the sheet says how
   many questions it could not make rows of and why).
3. The reference a row carries: what it is made of so that it names one value
   of one thing and survives sorting, deleting and a workbook program saving
   the file again; and what a row says the job had when the sheet went out.
4. Every reading of a length a typed measurement takes today (a laser in
   keyboard mode, feet and inches, a bare number), with the function, so the
   sheet reads the same; and what a workbook program may do to a cell on the
   way (a number in place of text, a date in place of a fraction): say what you
   guard against and how.
5. The figures `demos/field.sh` comes to (run it and quote its last lines): the
   measurements it types, the price before and after, which clash clears and
   which becomes a finding. These are your demonstration's figures.
6. BIMCraft at /home/user/ipedrax/bimcraft: `grep -rln -i
   "field_sheet\|punch\|verify\|survey_import\|measured" crates` and read the
   whole list; say what is there.

## What is built (call 234, points 1 to 8, every clause)
Tests FIRST: each test written and run red before the code it tests, the red run
quoted; where one could not be, say which and why, and prove it by breaking the
code. `demos/field-sheet.sh` (new) to the nine steps in STATUS.md; every step
ends the script when the program does not say what the step expects; its row in
`README.md`; `SPEC.md` only if the job's own record changes (it should not: a
measurement by file is a measurement).

## Rules
- Files you may touch: `crates/world/src/` and `crates/world/tests/`,
  `crates/sheets/src/` (only to read or write the sheet's cells),
  `crates/cli/src/` and `crates/cli/tests/`, `demos/field-sheet.sh`, `README.md`
  (a row), `PORTMAP.md`. Nothing in `crates/app/src`.
- **No existing expectation is edited.** `demos/field.sh`, `demos/vif.sh` and the
  printed sheet are exactly as they were (hold `export vif` of the sample whole
  house to a copy made before your work).
- One arithmetic: a row sets a value by the function a typed measurement uses;
  nothing about what follows is worked out again. A value is Known only by a
  row that has something As found; nothing is Known on the sheet's say-so
  beyond that. No price, quantity, rounding rule or status rule changes.
- Nothing from a real job: every name, figure and date is made up.
- Every word a person reads is a builder's (a sheet, a row, as found; a length
  in a sentence said by `units::length_said`).
- Every cargo command and every demonstration runs in the idle class
  (`chrt -i 0 nice -n 19 ...`). Before every cargo command and every
  demonstration wait while `SCRATCH/quiet.lock` exists; wait on no other lock.
  Only the command-line crates; for a crate with several test files run `--lib`
  and then one `--test <name>` at a time, deleting the test programs after
  each. NEVER a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. `df -h /` before every
  build. Scratch files in a folder of your own (`mktemp -d -p SCRATCH`); delete
  only what you made.
- Search each demonstration's output for FAIL as well as its exit. The three
  root gates, `tools/portmap.sh` and `tools/licenses.sh` pass. The program
  opens no network connection. No new crate.
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it. No web
  search with a person's or the project's name in it.

## Done when
The report has the six "before any code" answers; the new tests pass (red runs
quoted, and which were before the code); `demos/field-sheet.sh` passes; every
command-line crate's tests and every command-line demonstration pass with none
edited; the gates pass; the report says which kinds of question have no row and
why, and what is not right.
