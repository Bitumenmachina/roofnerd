# Contract 197I: a model brought in whole (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 197"
(points 1 to 7 are yours), plumbline/STATUS.md "The second window judgment ... a
model brought in whole: a demonstration" (the figures your work comes to),
plumbline/research/judgment-window-1124b.md section B (what the judge saw),
`crates/ifc/src/lib.rs` (the reader: what it brings in, what it leaves and the
reason it gives), `crates/ifc/src/write.rs` (what the program writes for a footing,
a roof, a fitting, a fixture, equipment, furniture), `crates/world/src/editing.rs`
(`place`, `footings`, the fitting and roof changes the tools make) and `demos/ifc.sh`
and `demos/export.sh` (the checks there are).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-197I-report.md. Commit nothing, push nothing.
When done, from the working copy's root:
`git add -N . && git diff --binary -- plumbline > SCRATCH/review/call197I.patch`.

## Before any code, in your report
1. Search ALL of BIMCraft at /home/user/ipedrax/bimcraft for reading a model
   (`grep -rln -i "ifc" crates apps` and read the whole list; look for how a fitting,
   a terminal, a footing and a roof are read or written). Port what is there; record
   in `plumbline/PORTMAP.md` either way; `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass.
2. Run `plumbline sixteen`, `export ifc`, `import ifc` into an empty job on the tree
   as it is, and set beside the lead's figures in STATUS.md what the first job has
   and what the second gets (counts by kind, open ends by system, footings, roof,
   names). Do not bend the program to a figure: where they differ, say which is
   right and stop on that point for a ruling.
3. **The roof's property set, proposed before it is written**: its name (not
   `Pset_...`), each property's name, type and unit, and one roof written out as it
   would read in the file. Say what the roof holds in the model and show that the
   set is enough to make the same roof by the change the Roof tool makes (and a
   gable made on an edge). What is written out is a promise to files already
   written: go on with everything else, and build the set as you proposed it; the
   lead reads the proposal when you hand back and may ask for a name changed.
4. Say how a fitting in the file names the runs it joins (ports, a relation, or
   only its place) in the program's own file, and what can be read from a file
   that has only its place.

## What is built (call 197, points 1 to 7, every clause)
- Fixtures, equipment and furnishings brought in as things set down (point 1);
  fittings on their runs (point 2); strip and pad footings (point 3); the
  program's own roof (point 4); names kept (point 5); what is left out says what
  follows (point 6); the two small words (point 7).
- Everything brought in is Assumed (or as the person bringing it in says, as
  today), sourced to the file and the thing's number in it. Nothing is Known on
  the file's say-so. What cannot be placed is left out and listed with a reason
  that is true; nothing is dropped without a word and nothing is made up.
- **One arithmetic**: a thing brought in is made by the same change a tool makes
  (`place`, the footing and roof changes), not by a second way of making it.
- The written file stays valid IFC4: `demos/export.sh` (IfcOpenShell, run as an
  outside checker only, never linked) passes with the roof's set in it.
- **Every existing test and demonstration expectation that moves is listed before
  one is changed** (file and line, the figure it wanted, the figure it gets, why).
  One that counts what is left out moves where this work brings that thing in, and
  only there; any other movement is NOT edited and is handed back.
- Tests FIRST, run red, quoted. A demonstration `demos/ifc-whole.sh` (new) to the
  seven points in STATUS.md, and its row in `README.md`.

## Part 2, after the above is whole: call 199 (an assembly changed for another)
Read plumbline/DECISIONS.md "### Call 199" and plumbline/STATUS.md "An assembly
changed for another: a demonstration", and `crates/world/src/job.rs`
`change_assembly`. Build the five points; tests first, run red, quoted;
`demos/change-assembly.sh` (new) to the five steps; the reply of `apply --change`
in `crates/cli/src/` says what the call says it says. Before changing any
expectation, list every test and demonstration that changes one assembly for
another and what each reads before and after. `crates/app` is not touched: say in
the report which window code calls `change_assembly` and what it will now show.

## Rules
- Files you may touch: `crates/ifc/src/`, `crates/ifc/tests/`, `crates/world/src/`
  only where the reader needs a change a tool already makes opened to it (say
  which, and why; no meaning of any figure changes), `crates/cli/src/` (the two
  words of point 7; what `import ifc` prints), `demos/ifc-whole.sh`, existing tests
  and demonstrations only under the rule above, `README.md` (a row), `PORTMAP.md`,
  `NOTICE` (only if something is ported). Nothing in `crates/app/src`.
- The saved job's format does not change. No rounding rule, no status rule.
  IfcOpenShell is LGPL: run it, never link it. No number from a real job.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates, one crate's tests at a time. Watch
  `df -h /`: under 4 GB free, stop and delete your own old test programs (literal
  absolute paths). Do not run cargo before the time the starting message gives; do
  other work until then and do not hand back only to wait.
- Demonstrations with `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not have,
  stop and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
The report has the four "before any code" answers; the new tests pass (red runs
quoted); `demos/ifc-whole.sh`, `demos/ifc.sh` and `demos/export.sh` pass; every
command-line crate's tests and every command-line demonstration pass or are on the
list of expectations moved, each with its reason; the gates pass.
