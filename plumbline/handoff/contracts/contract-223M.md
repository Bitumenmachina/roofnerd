# Contract 223M: the database in one hand (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md ("The main thing to remember": "they own their
database"), then plumbline/DECISIONS.md "### Call 223" (every numbered point) and
"### Call 220" (point 2), plumbline/STATUS.md "The database in one hand: a
demonstration", and in the code: `crates/world/src/tables.rs` (every table's
reading and writing: items, shapes, where items connect, assemblies; the headings
and the three columns a value takes), `crates/world/src/pricelist.rs` (prices by
supplier as rows), `crates/sheets` (how a workbook and a plain table are written
and read), `crates/cli/src/main.rs` (every `import ...` and `export ...` verb:
follow one table whole, from the words typed to the saved job and back),
`demos/imports.sh`, `demos/shapes.sh`, `demos/connections.sh`, `demos/pricelist.sh`.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-223M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call223M.patch`.

## Before any code, in your report
1. Every table the program can write out and bring back today: its verb, its
   headings, the function that writes it and the one that reads it, what it
   depends on (connections need their items; items name shapes), and whether out,
   in and out again is byte for byte today (run each and say). A table that
   cannot yet be written out or brought back whole is named: STOP on it for a
   ruling; do not paper over it.
2. The order the tabs must be brought in, and why. What "all of it or none of it"
   will rest on (the model's own all-or-nothing).
3. How a workbook with several tabs is written and read today (`crates/sheets`),
   and whether writing the same job twice gives the same bytes (if not, say what
   differs and compare cell for cell instead).
4. The reply's words, tab by tab, set beside the words each single table's verb
   uses today, so a person reads one language.
5. Work the demonstration's seven steps by the tables as they are and give the
   counts the program holds today (items, shapes, connection rows, assemblies and
   their lines); the lead's are from an earlier day.

## What is built (call 223, points 1 to 6)
`export database <job> <file.xlsx | folder>` and `import database <job> <file.xlsx
| folder | starter>`, each tab or file being exactly its own table's, with a tab
for cabinet rules only if that table exists when you build this. Tests FIRST, run
red, quoted. `demos/database.sh` (new) to the seven steps in STATUS.md, every step
ending the script when the program does not say what the step expects; its row in
`README.md`.

## Rules
- Files you may touch: `crates/world/src/` and its tests (a new module for the
  whole database; the single tables' functions are called, not copied or
  changed), `crates/sheets/src/` (only if a workbook of several tabs needs
  something it lacks: say what), `crates/cli/src/` and its tests,
  `demos/database.sh`, `README.md` (rows), `PORTMAP.md`. Nothing in
  `crates/app/src`. No public function the window calls changes its shape.
- **No existing expectation is edited**, and no single table's headings, order or
  bytes change: the whole is made of the parts as they are.
- The saved job's format does not change. Every value keeps how sure it is and
  where it came from through the round trip.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates, one crate's tests at a time. Before
  EVERY cargo command and every demonstration, wait while SCRATCH/window-run.lock
  or SCRATCH/lead-run.lock exists (check every 15 seconds); never create or delete
  either. NEVER run a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. Check `df -h /` before every
  build: under 5 GB free, stop and delete your own old test programs (literal
  absolute paths); delete test programs after each crate's tests.
- Search each demonstration's output for FAIL as well as its exit.
- The three root gates, `plumbline/tools/licenses.sh` and
  `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` pass.
- Every word a person sees is a builder's: "my database", a tab, a table; never
  a schema, a record or a field.
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the five "before any code" answers; the new tests pass (red runs
quoted); `demos/database.sh` passes; every command-line crate's tests and every
command-line demonstration pass with none edited; the gates pass; the report says
what is not right.
