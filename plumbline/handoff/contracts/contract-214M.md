# Contract 214M: a maker's model brought in as an item (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 214"
(every numbered point), "### Call 195" (step E), "### Call 211" with its "as built"
(the rows a connection is written as), "### Call 200" and "### Call 202" (how a
model's connection points are read today), plumbline/STATUS.md "A maker's model
brought in as an item: a demonstration", and
`research/survey-maker-model-and-cabinets.md` (where each thing is today in both
programs, file and line: check a line before relying on it).
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-214M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call214M.patch`.

## Before any code, in your report
1. BIMCraft at /home/user/ipedrax/bimcraft: read how a product's body is read
   (`crates/core/src/import.rs` about 862 to 905 and 2603 to 2780,
   `crates/core/src/import/shapes.rs`) and say, kind of body by kind, what is
   ported and what is left (say why). Everything ported is in
   `plumbline/PORTMAP.md` and `NOTICE`;
   `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` must pass.
2. Where the class names, the attribute places and the property set's names come
   from: IfcOpenShell's own copy of the IFC4 and IFC2x3 schemas, read by running
   it as an outside program (it is never linked), as call 202 did. Nothing about
   the standard is written from memory. Say what you read for: the type of a
   product, a shape shared by a type, the manufacturer's property set, a
   connection point's placement and direction.
3. How the five files of the demonstration are written (by hand, small, made up),
   and that each passes `tools/ifc-check.py` (or says exactly what the checker
   says and why that is acceptable). Work the demonstration's steps 1 and 2 by
   the code and set your figures beside the lead's; where they differ say which
   is right and why.
4. What `import spec` returns and how its reply reads, so a maker's model reads
   the same way to a person (the same order: what was made, each value with how
   sure and where from, what is Unknown, what was left out).

## What is built (call 214, every numbered point)
- The reader and the verb (`import maker-model <job> <file> [--as ...]`), with its
  usage line in a builder's words.
- Tests FIRST, run red, quoted: each kind of body; millimetres, metres and feet; a
  type shared by several; a proxy with and without `--as`; an appliance; the
  manufacturer's property set on the type, on the product, absent; each
  connection kind; a point outside the body; the same file twice (byte for byte);
  a file with a building in it (it still makes items and sets nothing down; say
  what the reply says of the building).
- `demos/maker-model.sh` (new) to the ten steps in STATUS.md, every step ending
  the script when the program does not say what the step expects; its files under
  `demos/maker-model/`; its row in `README.md`.

## Rules
- Files you may touch: `crates/ifc/src/` and `crates/ifc/tests/`, `crates/cli/src/`,
  `crates/world/src/` (only what making an item and its rows from this needs: say
  what), `demos/maker-model.sh`, `demos/maker-model/`, `README.md` (rows),
  `PORTMAP.md`, `NOTICE`. Nothing in `crates/app/src`. No public function the
  window calls changes its arguments (`grep -rn` it in `crates/app/src` first).
- **No existing expectation is edited.** An existing test or demonstration that
  fails is handed back with what was seen. `import ifc` reads exactly as it did:
  the test that holds the sample house out and back is unchanged and passes.
- The saved job's format does not change. Nothing is ever Known on a file's
  say-so. Nothing from a real maker: the maker, the product and the figures are
  invented.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates (`-p plumbline-world -p plumbline -p
  plumbline-plans -p plumbline-sheets -p plumbline-ifc -p plumbline-roofing`), one
  crate's tests at a time. NEVER run a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. Watch `df -h /`: under 4 GB
  free, stop and delete your own old test programs (literal absolute paths).
- Search each demonstration's output for FAIL as well as its exit.
- The three root gates and `plumbline/tools/licenses.sh` pass. No new crate is
  linked without its licence passing `tools/licenses.sh` (none should be needed).
- Every word a person sees is a builder's: a connection, a maker's model; never a
  port, a proxy's class name alone, or a file ending as a noun.
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the four "before any code" answers; the new tests pass (red runs
quoted); `demos/maker-model.sh` passes; every command-line crate's tests and every
command-line demonstration pass with none edited; the gates pass; the report says
what is not right.
