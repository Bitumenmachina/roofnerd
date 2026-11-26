# Contract 228M: clashes handed over as an issue list, BCF (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 228"
(every numbered point), plumbline/PROPOSALS.md 36, plumbline/STATUS.md "Clashes
handed over as an issue list: a demonstration", and in the code:
`crates/world/src/building/clash.rs` (what a clash holds: the two things, the
place, by how much, its certainty), `crates/world/src/vif.rs` (the dollars riding
on a question), `crates/ifc/src/write.rs` (how a thing's id in the BIM file is
made: the issue list must give the same id for the same thing, by the same
function, never by a second way of making it), `crates/cli/src/main.rs` (`clashes`,
`vif`, `export ifc`), `tools/ifc-check.py` (how an outside checker is run and
never linked), `crates/sheets` (how a zip of text files is written today, if it
is; else what is linked that writes one).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-228M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call228M.patch`.

## Before any code, in your report
1. **The format, from its source.** Search the web with plain words (never a
   person's or this project's name) for buildingSMART's own published BCF: the
   files in the zip, what each holds, which parts are required, how a thing is
   named (by its id in the model), how a view is given, and where its schema
   files are published and under what terms. Write `research/bcf.md`: each
   file, each part written, the address and the line quoted; which version is
   written and why (which the common coordination programs open, from their
   makers' own pages). What you could not confirm is said and not relied on.
2. How a thing's id in the BIM file is made today (file and line), and proof by
   a test that the issue list's ids are the BIM file's ids.
3. How the zip is written with what is already linked (no new crate without
   `tools/licenses.sh` passing and a line in `Cargo.toml` saying why).
4. The outside checker: what it is, how it is fetched or installed in this
   environment, that it is only run (`tools/bcf-check.*`, beside
   `tools/ifc-check.py`), and what the demonstration does where it is not
   installed (it says so and does not pass in silence).
5. The view: how the eye and the place looked at are worked from a clash, in the
   format's own units and axes (say what they are, from the source).
6. BIMCraft at /home/user/ipedrax/bimcraft: `grep -rln -i "bcf\|issue\|viewpoint"
   crates` and read the whole list; say what is there. Every new public item in
   `PORTMAP.md` where the port map takes it.

## What is built (call 228, points 1 to 8, every clause)
Tests FIRST, run red, quoted. `demos/bcf.sh` (new) to the nine steps in
STATUS.md; every step ends the script when the program does not say what the step
expects; its row in `README.md`. Where a figure in STATUS.md is wrong, say which
is right and why; do not bend the program to it.

## Rules
- Files you may touch: `crates/world/src/` and `crates/world/tests/`,
  `crates/ifc/src/` and `crates/ifc/tests/` (the writer lives best beside the BIM
  file's: say where you put it and why), `crates/cli/src/` and `crates/cli/tests/`,
  `tools/bcf-check.*`, `demos/bcf.sh`, `README.md` (a row), `PORTMAP.md`,
  `research/bcf.md`. Nothing in `crates/app/src`.
- **No existing expectation is edited.** The clash list, the verify-in-field list
  and the BIM file are written exactly as they were (hold each, for the sample
  whole house, to a copy made before your work).
- One arithmetic: nothing about a clash is worked out again for the issue list;
  it says what the clash list says. A clash's certainty is never raised.
- No person's name, no machine's name and no path is written into the file.
- Every word a person reads in the issues is a builder's; no refusal or
  description in feet with decimals or in coordinates alone.
- Every cargo command and every demonstration runs in the idle class
  (`chrt -i 0 nice -n 19 ...`); no lock is waited on; only the command-line
  crates, one crate's tests at a time; NEVER a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. `df -h /` before every
  build; delete the test programs after every `cargo test`. Scratch files in a
  folder of your own (`mktemp -d -p SCRATCH`); delete only what you made.
- Search each demonstration's output for FAIL as well as its exit. The three
  root gates, `tools/portmap.sh` and `tools/licenses.sh` pass. The program
  opens no network connection (a checker fetched by a tool script is the
  tool's, never the program's, and nothing fetched enters the repository unless
  its terms allow it and you say so).
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the six "before any code" answers; the new tests pass (red runs
quoted); `demos/bcf.sh` passes; every command-line crate's tests and every
command-line demonstration pass with none edited; the gates pass; the report says
what could not be confirmed from a source and what is not right.
