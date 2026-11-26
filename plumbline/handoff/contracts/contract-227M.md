# Contract 227M: marks on a PDF read as quantities; a TIFF as a plan (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md ("Imports matter as much as exports"; "Flag,
never fabricate"; "Real job, client or supplier data never enters the
repository"), then plumbline/DECISIONS.md "### Call 227" (every numbered point),
"### Call 220" (the table: the two rows this job takes), "### Call 221, as built"
(how the last new file type was taken in: this job follows that pattern),
plumbline/STATUS.md "Marks on a PDF read as quantities, and a TIFF as a plan: a
demonstration", and in the code: `crates/plans/src/lib.rs` (`sort_of`, `read`,
`size_of`, `strings`, `words`: how a PDF and a picture are read today, and what
the linked PDF crate hands out), `crates/plans/Cargo.toml` (what is linked and
why), `crates/world/src/plans.rs` (what a drawing in the job holds),
`crates/cli/src/main.rs` (`drawing add`; `import estimate`: how a prior
estimate's lines come in, what a line with no division or no price is, how the
same file brought twice is known), `crates/cli/src/cad.rs` (the DXF verbs),
`demos/dxf.sh`, `demos/imports.sh`.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-227M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call227M.patch`.

## Before any code, in your report
1. **The standard, from its source.** Search the web with plain words (never a
   person's or this project's name) for how a PDF holds a measured note (the PDF
   standard's sections on line, polyline, polygon and other notes, their points,
   the measure dictionary and its scale, the note's own text, its label and its
   own name) and for what the best-known plan-marking program's maker publishes
   about the marks it writes (length, perimeter, area, count; how the figure and
   the label are stored). Write `research/pdf-marks.md`: each kind, each key
   read, the address and the line quoted. Say plainly what you could not confirm
   from a source; **a key you could not source is not relied on**.
2. What the linked PDF crate hands out of a page's notes, file and line in its
   source under `~/.cargo/registry` (search all of it before saying it hands out
   nothing). If it hands out nothing, how the notes are read with code already
   linked or written here; no new crate without `tools/licenses.sh` passing and
   a line in `Cargo.toml` saying why.
3. How the made-up marked PDF is written (by a script of yours kept with the
   demonstration, so anyone can see what is in it), and proof that an ordinary
   PDF reader opens it (the page drawn to a picture by the program's own reader,
   looked at).
4. How a prior estimate's line is held today when it has no division and no
   price, and so how a mark's line is held: say what `estimate` prints for it.
5. The TIFF part of the picture crate: its licence, the packings it reads, what
   it says of one it does not, and how a page is chosen.
6. BIMCraft at /home/user/ipedrax/bimcraft: `grep -rln -i
   "annot\|markup\|measure\|tiff" crates` and read the whole list; say what is
   there. Every new public item in `PORTMAP.md`.

## What is built (call 227, points 1 to 9, every clause)
Tests FIRST, run red, quoted. `demos/marks.sh` (new) to the ten steps in
STATUS.md, with its files in `demos/marks/` and the script that makes them; every
step ends the script when the program does not say what the step expects; its row
in `README.md`; `SPEC.md` for what a drawing now holds.

## Rules
- Files you may touch: `crates/plans/src/`, `crates/plans/Cargo.toml` (only the
  TIFF part of the picture crate, or a crate whose licence you have shown
  passes), `crates/world/src/` and `crates/world/tests/`, `crates/cli/src/` and
  `crates/cli/tests/`, `demos/marks.sh`, `demos/marks/`, `README.md` (rows),
  `SPEC.md`, `PORTMAP.md`, `research/pdf-marks.md`. Nothing in `crates/app/src`.
- **No existing expectation is edited.** One that fails is handed back with
  file, line, the figure expected and the figure got.
- A mark is never Known. A mark that cannot be read is named and left out, never
  brought in as nought. No price, quantity rule, rounding rule or status rule
  changes. A job with no marks is saved byte for byte as before (a test against
  a job written by the tree you start from; say how you made it).
- Nothing from a real job: every label, figure and file is made up; no file
  written by a marking program is fetched, kept or read by a test.
- Every word a person reads is an estimator's: a mark, a length, an area, a
  count, a note; never the standard's own words for them.
- Every cargo command and every demonstration runs in the idle class
  (`chrt -i 0 nice -n 19 ...`); no lock is waited on; only the command-line
  crates, one crate's tests at a time; NEVER a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. `df -h /` before every
  build; delete the test programs after every `cargo test`. Scratch files in a
  folder of your own (`mktemp -d -p SCRATCH`); delete only what you made.
- Search each demonstration's output for FAIL as well as its exit. The three
  root gates, `tools/portmap.sh` and `tools/licenses.sh` pass. The program
  opens no network connection.
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the six "before any code" answers; the new tests pass (red runs
quoted); `demos/marks.sh` passes; every command-line crate's tests and every
command-line demonstration pass with none edited; the gates pass; the report says
what could not be confirmed from a source and what is not right.
