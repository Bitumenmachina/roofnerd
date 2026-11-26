# Contract 230M: a scanner's own files taken in, LAS and E57 (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md ("Real job, client or supplier data never
enters the repository"), then plumbline/DECISIONS.md "### Call 230" (every
numbered point), "### Call 220" (the table) and the calls on a scan it names
(search DECISIONS.md for "PLY"), plumbline/STATUS.md "A scanner's own files: a
demonstration", and in the code: `crates/plans/src/scan.rs` (`read_ply`, `Scan`,
`cut`, `bring_in`: a LAS or an E57 becomes the same `Scan` and goes down the same
road), `crates/plans/src/lib.rs` (`sort_of`: how a file's kind is told),
`crates/cli/src/main.rs` (every verb that takes a scan), `demos/scan.sh`,
`demos/findwalls.sh`.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-230M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call230M.patch`.

## Before any code, in your report
1. **LAS, from its source**: search the web with plain words for the format's
   own published text (its versions, the header, the kinds of point record, how
   a point's place is a whole number times a scale plus an offset, where a unit
   is or is not said). Write `research/scan-files.md` with the address and the
   line quoted for each thing relied on. Say which versions and kinds the
   reader will read and why those.
2. **LAZ and E57**: every crate that reads either, its licence from its own
   manifest (not from memory), what it links in turn (`cargo tree`), and whether
   `tools/licenses.sh` passes with it. Your recommendation for each: read, or
   refused by name with what to save it as. A crate under GPL, AGPL or LGPL, or
   one that links a library under them, is not linked.
3. How the made-up room's files are written (a script kept with the
   demonstration) and why the PLY and the LAS hold the same points.
4. Units and up: what each file says of them, and how what it says is offered
   and never silently taken.
5. BIMCraft at /home/user/ipedrax/bimcraft: `grep -rln -i "las\b\|laz\|e57\|point_cloud\|pointcloud" crates`
   and read the whole list; say what is there.

## What is built (call 230, points 1 to 7, every clause)
Tests FIRST, run red, quoted. `demos/scan-files.sh` (new) to the seven steps in
STATUS.md, with its files in `demos/scan-files/` and the script that makes them;
every step ends the script when the program does not say what the step expects;
its row in `README.md`.

## Rules
- Files you may touch: `crates/plans/src/`, `crates/plans/Cargo.toml` (only a
  crate whose licence you have shown passes), `crates/world/src/` and
  `crates/world/tests/` (only if a scan's record in the job must say its kind),
  `crates/cli/src/` and `crates/cli/tests/`, `demos/scan-files.sh`,
  `demos/scan-files/`, `README.md` (a row), `SPEC.md` (only if the job's record
  changes), `PORTMAP.md`, `research/scan-files.md`. Nothing in `crates/app/src`.
- **No existing expectation is edited.** A PLY is read exactly as it was.
- No real scan is fetched, kept or read by a test; every file is made up by the
  script. A scan is never Known.
- Every cargo command and every demonstration runs in the idle class
  (`chrt -i 0 nice -n 19 ...`). Before every cargo command and every
  demonstration wait while `SCRATCH/quiet.lock` exists; wait on no other lock.
  Only the command-line crates, one crate's tests at a time; NEVER a
  demonstration without `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
  `df -h /` before every build; delete the test programs after every `cargo
  test`. Scratch files in a folder of your own (`mktemp -d -p SCRATCH`); delete
  only what you made.
- Search each demonstration's output for FAIL as well as its exit. The three
  root gates, `tools/portmap.sh` and `tools/licenses.sh` pass. The program
  opens no network connection. No new crate unless `tools/licenses.sh` passes
  with it (permissive licences only: never GPL, AGPL or LGPL) and `Cargo.toml`
  says why.
- A length in a sentence is said as a builder says it (`units::length_said`);
  every word a person reads is a builder's.
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it. No web
  search with a person's or the project's name in it.

## Done when
The report has the five "before any code" answers; the new tests pass (red runs
quoted); `demos/scan-files.sh` passes; every command-line crate's tests and every
command-line demonstration pass with none edited; the gates pass; the report says
which kinds are read and which refused and why, what could not be confirmed from
a source, and what is not right.
