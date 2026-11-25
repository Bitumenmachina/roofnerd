# Contract 221M: a CAD drawing in and out, DXF (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md (the fixed rules on licences and on imports;
"The main thing to remember"), then plumbline/DECISIONS.md "### Call 221" (every
numbered point) and "### Call 220" (why), "### Call 188" (the walls finder on a
drawn plan), plumbline/STATUS.md "A CAD drawing in and out: a demonstration", and
in the code: `crates/plans/src/lib.rs` (`read`, `Page`: how a PDF page's lines and
words are held), `crates/plans/src/drawn.rs` and `found*.rs` (the finders),
`crates/world/src/plans.rs` (how a sheet is put on a level, its scale and its
calibration, and what the saved job holds of it), `crates/cli/src/main.rs` (the
verbs that bring a drawing in: follow one whole, from the words typed to the saved
job), `demos/plan.sh` and `demos/imports.sh`.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-221M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call221M.patch`.

## Before any code, in your report
1. **The format, from its source.** Autodesk publishes the DXF reference
   (https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/index.htm and
   later years). Read what you need there (search the web by the format's own
   name only; never a person's or this project's name) and say, with the page
   for each: how units are given (`$INSUNITS`, `$MEASUREMENT`), the entities you
   will read (LINE, LWPOLYLINE, POLYLINE, ARC, CIRCLE, ELLIPSE, SPLINE, TEXT,
   MTEXT, INSERT with its BLOCK, DIMENSION, HATCH, IMAGE, 3DSOLID), layers, and
   the binary and ASCII forms. Nothing about the format is written from memory.
2. **The code to read and write it.** Search crates.io for DXF crates (the review
   found `dxf`, MIT: https://github.com/ixmilia/dxf-rs): for each, its licence,
   its last release, what it reads, how much it would add to what is linked
   (`cargo tree` after adding it in your copy), and whether
   `plumbline/tools/licenses.sh` and `tools/gate-egress.sh` pass with it. Choose:
   that crate, or a reader written here from the reference (the ASCII form is
   pairs of lines and is small). Say why. No GPL, AGPL or LGPL code is linked or
   copied (LibreDWG and its tools are GPL: not even read for this).
3. **How a DWG is known** whatever its name (its first bytes: find the published
   statement of them) and how a Revit and a SketchUp file are known, so each is
   refused by name.
4. **The layer names for the way out**: the AIA CAD Layer Guidelines as the US
   National CAD Standard carries them (A-WALL, A-DOOR, A-GLAZ and so on). Find a
   source you can read and quote for each layer you will write; a thing with no
   sourced name gets a plain one and is listed as such.
5. How a PDF page is held as a sheet today and what a DXF sheet needs that it
   lacks (layers as switches; a scale that is the file's own; curves), and what
   the saved job will hold for it. If a key is added to the saved job, say which,
   and show an old job reads as it did.
6. Work the demonstration's steps 1, 2 and 7 by hand from your hand-written file
   and set your figures beside the lead's; where they differ say which is right.

## What is built (call 221, points 1 to 8)
Tests FIRST, run red, quoted. `demos/dxf.sh` (new) to the nine steps in STATUS.md,
every step ending the script when the program does not say what the step expects;
its files under `demos/dxf/`; its row in `README.md`; `SPEC.md` for any new key.

## Rules
- Files you may touch: `crates/plans/src/` and its tests (or a new crate
  `crates/dxf` if you write the reader here: say so first), `crates/world/src/`
  (only `plans.rs` and what a sheet needs), `crates/cli/src/` and its tests,
  `Cargo.toml` files and `Cargo.lock` (only for a crate you have shown passes the
  licence gate), `demos/dxf.sh`, `demos/dxf/`, `README.md` (rows), `SPEC.md`,
  `PORTMAP.md`, `NOTICE` (a crate added is named there with its licence). Nothing
  in `crates/app/src`. No public function the window calls changes its arguments.
- **No existing expectation is edited.** A PDF and a picture come in exactly as
  they did; an existing test or demonstration that fails is handed back.
- Everything read from a file is Assumed and says the file. What cannot be
  carried is counted and said, never dropped in silence. Nothing from a real
  drawing: the demonstration's files are made up.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo ... -j2` from
  `plumbline/`, only the command-line crates, one crate's tests at a time. Before
  EVERY cargo command and every demonstration, wait while the file
  SCRATCH/window-run.lock exists (check every 15 seconds); never create or delete
  it. NEVER run a demonstration without
  `PLUMBLINE_BIN=<your copy>/plumbline/target/debug`. Watch `df -h /`: under 5 GB
  free, stop and delete your own old test programs (literal absolute paths).
- Search each demonstration's output for FAIL as well as its exit.
- The three root gates and `plumbline/tools/licenses.sh` pass.
- Every word a person sees is a builder's: a drawing, a layer, a sheet; "a CAD
  drawing (DXF)" where the format must be named.
- Work in the foreground; leave no timers or background jobs when you hand back.
  Subagents, if any, on Sonnet or Opus. If a tool asks a permission you do not
  have, stop and say so in the report; do not ask the lead to do it.

## Done when
The report has the six "before any code" answers; the new tests pass (red runs
quoted); `demos/dxf.sh` passes; every command-line crate's tests and every
command-line demonstration pass with none edited; the gates pass; the report says
what is not carried in each direction and what is not right.
