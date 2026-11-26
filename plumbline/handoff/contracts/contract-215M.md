# Contract 215M: a cabinet by rule (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 215"
(every numbered point), "### Call 195" (step F), "### Call 155" (the cabinet codes
and what was owed), "### Call 203" (how the thirteen shapes were made rows and held
to the code they replaced: this job follows that pattern), plumbline/STATUS.md "A
cabinet by rule: a demonstration", `research/survey-maker-model-and-cabinets.md`
(step F: where each thing is today, file and line: check a line before relying on
it), `research/renovation-kitchen-and-phasing.md` (what the codes rest on),
`library/items.csv` (the thirty-five cabinet rows) and `demos/cabinets.sh` if there
is one (`ls demos | grep -i cab`).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-215M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call215M.patch`.

## Before any code, in your report
1. BIMCraft at /home/user/ipedrax/bimcraft: how a family's types and parameters
   give a size (`crates/core/src/family.rs`: `types`, `parameters`, `resolve`, the
   starter Base Cabinet) and whether anything there reads a size out of a name.
   Say what is ported and what is ours; `PORTMAP.md` for every new public item;
   `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass.
2. The table of rules as you will write it: its headings in a builder's words,
   and the eight starter rows, each set beside the starter items it must make.
   Run the thirty-five through it by hand or by a first test and say which, if
   any, do not come out as their row: STOP on those for a ruling; bend neither.
3. Before changing anything, write the baseline step 8 needs: with the build
   that is there, lay B15, B30 and SB36 with a countertop in a new job and keep
   the saved job and its checksum.
4. Every place the program asks whether a box is a wall box or reads a cabinet's
   name (`grep -rn "starts_with('W')\|box_codes\|fn cabinets" crates`), in the
   model and in the window's crate. The window's crate is not yours: any function
   it calls keeps its arguments and what it gives back; say how.

## What is built (call 215, points 1 to 7, every clause)
Tests FIRST, run red, quoted. `demos/cabinet-rules.sh` (new) to the eight steps in
STATUS.md, every step ending the script when the program does not say what the
step expects; its row in `README.md`; `SPEC.md` for the one new key.

## Added 2026-11-26 (call 223, as built, point 6)
The table of cabinet rules joins the database: `export database` writes it as a
tab (and a file in the folder) named for what a cabinet maker would call it, and
`import database` brings it back with the rest, all or none, in its place in the
order (after Items, which it names). `crates/world/src/database.rs` is yours to
touch for that; `demos/database.sh` is NOT edited (its counts are of the five
tabs it names: if a sixth tab moves one of its checks, hand that back with what
was seen). Your own demonstration shows the tab written out and brought back.

Sharing the machine (this replaces the cargo line below): every cargo command and
every demonstration runs as `chrt -i 0 nice -n 19 ...`, and no lock is waited on.

## Rules
- Files you may touch: `crates/world/src/` and `crates/world/tests/`,
  `crates/cli/src/`, `library/cabinet-rules.csv`, `demos/cabinet-rules.sh`,
  `README.md` (rows), `SPEC.md`, `PORTMAP.md`, `NOTICE` (only if something is
  ported). Nothing in `crates/app/src`, and nothing in `library/items.csv`.
- **No existing expectation is edited.** An existing test or demonstration that
  fails is handed back with what was seen.
- No price, quantity, rounding rule or status rule changes. Every rule and every
  box made by one is Assumed. Nothing from a real maker's catalog: the stocked
  widths are the ones the starter chest already has.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates (`-p plumbline-world -p plumbline -p
  plumbline-plans -p plumbline-sheets -p plumbline-ifc -p plumbline-roofing`), one
  crate's tests at a time. NEVER run a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. Watch `df -h /`: under 4 GB
  free, stop and delete your own old test programs (literal absolute paths).
- Search each demonstration's output for FAIL as well as its exit.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Every word a person sees is a builder's or a cabinet maker's: a base, a wall
  box, a tall box, a filler, a sink base; inches.
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the four "before any code" answers; the new tests pass (red runs
quoted); `demos/cabinet-rules.sh` passes; every command-line crate's tests and
every command-line demonstration pass with none edited; the gates pass; the report
says what is not right.
