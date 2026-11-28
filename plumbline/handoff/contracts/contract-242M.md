# Contract 242M: one place that reads and writes files (model and command-line worker)

You work in the copy and on the commit the lead names when it starts you. Read
`CLAUDE.md` whole, then `plumbline/DECISIONS.md` "### Call 242" (every point) and
"### Call 220", **`demos/kind.md` (your demonstration, written first)**, and in
`crates/cli/src` every `import`, `export`, `drawing`, `layer` and `marks` verb as it
stands. Notes as you go to `SCRATCH/review/W-242M-report.md`, after every run.
Commit nothing, push nothing.

## What is built

1. **`crates/files`**, a crate above `world`, `plans`, `sheets` and `ifc` and below
   `cli` and `app`, with a permissive licence and no new crate linked from outside.
2. **`kind`**: the rule of call 242 point 1, whole. A table's kinds come from the
   tables' own readers (their headings tests made callable without reading the
   rows in; no second list of headings). One function gives the kinds in order and
   the words; one gives the line `plumbline kind` prints.
3. **`bring` and `write`**: the body of each verb that brings a file in or writes
   one out, **moved** from the command line into `crates/files` as functions that
   take the job, the file, the kind and what the verb's flags say, and give back
   what is said. The command line's verbs become calls. This is a move: prove it as
   contract 237S proves a cut (the moved lines are the old lines; the test names
   are the same; `tools/check.sh` passes with the same counts).
4. **`plumbline kind <file> [<file> ...]`** and `demos/kind.sh`, with a row in
   `README.md`.

## Before any code, in your report

- Each table reader's headings test run over the five files `demos/kind.md` marks
  "(first)" and over the two price lists and the prior estimate: the whole line for
  each. **Decide nothing for the five**: write the lines in the report, have
  `demos/kind.sh` print them without passing or failing them, and go on with the
  rest. The lead rules on each line.
- The list of verbs you will move, with the lines each has today, and which the
  window already spells itself in `crates/app/src/bring.rs` and `plans.rs` (do not
  change the window: that is contract 240B2).

## What must not change

No number, no meaning, no saved job, no file written out, no sentence the command
line says today, no expectation of a test or a demonstration. If something will not
move without changing one of those, leave it where it is and say why. Files you may
touch: `crates/files/**`, `crates/cli/src/**`, `crates/world/src/tables.rs` and its parts in
`tables/` (call 245), `fieldsheet.rs`, `database.rs` (only to make a headings test callable),
`Cargo.toml`, `Cargo.lock`, `demos/kind.sh`, `README.md`, `LICENSES.md` by its tool.

## How it is run

Every cargo command through `tools/quiet.sh` with `CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 -j2`; long commands in the background; waits of an hour
while a window run holds the lock are normal. If a tool asks a permission you do
not have, stop and say so in the report; do not ask the lead to do it.

## Done when

`BIMCRAFT=<the path the lead names> tools/check.sh` prints `check: PASS` with every
count as before but the new crate's tests and the one new demonstration;
`demos/kind.sh` passes the rows worked by hand and prints the five for the lead;
`tools/licenses.sh --check` and the three gates pass; the sample house is
`880d9fd3c364b5f7`; the report has the move's proof and what is not right.
