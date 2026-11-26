# Contract 229M: the house written out as glTF (command line only; no window)

You work in the second working copy named in the message that starts you, never in
/home/user/roofnerd. Read CLAUDE.md, then plumbline/DECISIONS.md "### Call 229"
(every numbered point) and "### Call 220" (the table), plumbline/STATUS.md "The
house written out as glTF: a demonstration", and in the code: `crates/ifc/src/write.rs`
(what the BIM file writes for each thing and where each solid comes from: this
job takes its solids from the same place), `crates/world/src/building/` (`forms.rs`,
`shape.rs`: a thing's shape as blocks; how a wall's openings are cut),
`crates/world/src/roof.rs` (`solid_planes`), `crates/world/src/geom.rs`
(`triangles`), `crates/world/src/finish.rs` (what finish a priced line names),
`crates/cli/src/main.rs` (`export ifc`), `tools/ifc-check.py`.
SCRATCH is named in the starting message.
Notes as you go to SCRATCH/review/W-229M-report.md, after every run. Commit nothing,
push nothing. When done, from the working copy's root:
`git add -N plumbline && git diff --binary -- plumbline > SCRATCH/review/call229M.patch`.

## Before any code, in your report
1. **The format, from its source**: search the web with plain words for the
   format's own published text (its measure, its axes and handedness, the one-file
   form and its two parts, how a part's points and faces are laid out and
   described, what a colour is). Write `research/gltf.md` with the address and
   the line quoted for each thing relied on; what you could not confirm is said
   and not relied on.
2. Every kind of thing the BIM file's writer writes and where its solid comes
   from, file and line; which of those can be handed out as faces without
   working a shape out a second way, and which cannot (named, and said in the
   reply as not carried).
3. The axes: how a place in the job becomes a place in the file, with one corner
   of the sample house worked by hand.
4. The test's own reader: what it reads and checks. The format's own checker:
   what it is, its licence, how it is installed here and only run
   (`tools/gltf-check.*`), and what the demonstration says where it is not
   installed.
5. BIMCraft at /home/user/ipedrax/bimcraft: `grep -rln -i "gltf\|glb\|mesh_export\|obj_export" crates`
   and read the whole list; say what is there. `PORTMAP.md` for every new public
   item the port map takes.

## What is built (call 229, points 1 to 8, every clause)
Tests FIRST, run red, quoted. `demos/gltf.sh` (new) to the eight steps in
STATUS.md; every step ends the script when the program does not say what the
step expects; its row in `README.md`. Where a figure in STATUS.md is wrong, say
which is right and why; do not bend the program to it.

## Rules
- Files you may touch: `crates/world/src/` and `crates/world/tests/`,
  `crates/ifc/src/` and `crates/ifc/tests/` (say where the writer lives and
  why), `crates/cli/src/` and `crates/cli/tests/`, `tools/gltf-check.*`,
  `demos/gltf.sh`, `README.md` (a row), `PORTMAP.md`, `research/gltf.md`. Nothing
  in `crates/app/src`.
- **No existing expectation is edited.** The BIM file is written exactly as it
  was (hold the sample house's to a copy made before your work).
- No person's name, no machine's name and no path is written into the file.
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
quoted); `demos/gltf.sh` passes; every command-line crate's tests and every
command-line demonstration pass with none edited; the gates pass; the report says
what is not carried, what could not be confirmed from a source, and what is not
right.
