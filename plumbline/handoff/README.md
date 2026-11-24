# Handoff: how this branch is worked, for whoever picks it up

Written by the lead on 2026-11-24 because Patrick asked that another agent, or Opus
5.5, be able to carry on when this session's context runs out. Read `../../CLAUDE.md`
first (the rules, who holds what, where each thing is changed), then the last section
of `../STATUS.md`, "Start here" (where the work stands and what is owed, in order).
This folder is the working method and its papers, which until now lived only in a
session's scratch area and were lost with it.

## The seats, as they were run
- **The lead** (one session): writes each decision in `DECISIONS.md` and each
  demonstration, with figures worked by hand, in `STATUS.md` BEFORE anything is
  built; hands work out under a written contract; accepts it by reading the diff
  and the pictures and by its own runs; rules on every expectation that moves;
  commits, merges and pushes. It does not build what a worker can.
- **Workers and builders** (Sonnet agents): one contract each. `contracts/` holds
  five real ones as patterns: a command-line worker's (`contract-192M.md`,
  `contract-190P.md`), a fix job from a judge's findings (`contract-191.md`), and a
  window builder's (`contract-192W.md`, `contract-193W.md`). Every one says: the
  files it may touch; tests first, run red, quoted; no existing expectation edited
  (stop and report, the lead rules); notes written as it goes; commit nothing.
- **Judges** (Sonnet agents, never the mind that built it): `judge-brief-command-line.md`
  and `judge-brief-window.md`. A judge gets a frozen copy of the program and a
  folder of its own, starts from an empty job, works figures by hand, and writes as
  it goes. Then a second judge goes round the fixes. Their notes are kept in
  `../research/judgment-*.md`.

## The mechanics that cost time to learn
- **One builder at a time works on the window**, in the main checkout, and runs the
  timed window demonstrations on a frozen copy of the two programs
  (`PLUMBLINE_BIN=<folder>`). No build runs beside a timed window run: it costs
  clicks and gives false failures (it did, three times).
- **Command-line work goes beside it** in a second working copy:
  `git worktree add --detach <scratch>/wtX <commit>`, with links to the main
  checkout's `node_modules`, `packages/app/node_modules` and
  `packages/engine/node_modules` (the commit hook runs the root build). The worker
  writes a patch (`git add -N . && git diff --binary -- plumbline > call.patch`).
- **The lead verifies in the worker's own copy** (its build is there: saves disk):
  the six command-line crates' tests one crate at a time, deleting test programs
  between crates; every command-line demonstration with `PLUMBLINE_BIN`; the three
  root gates, `tools/licenses.sh`, `BIMCRAFT=<where it is> tools/portmap.sh`. Then
  `git apply --3way` on an integration copy, checks the code files are byte for
  byte the verified ones, commits there and pushes that commit to the branch.
- **When the window builder is done**: commit locally, merge the pushed head,
  build, every crate's tests, then EVERY window demonstration whose house uses
  something the merge touched (one was missed for three merges: see DECISIONS.md,
  "Call 193, as built"), `tools/committed.sh`, push.
- **The disk is about 39 GB.** `tools/committed.sh` needs 8 GB free and fails with
  "No space left": `cargo clean -p` each workspace crate first, keeping a frozen
  copy of the two programs elsewhere. A worker's `target` is deleted the moment its
  patch is verified.
- **A restart stops every agent and keeps the disk.** Resume the agent with a
  message; do not start a new one. A check-in every 40 minutes catches a lost
  done-signal. Tell every agent to write its notes after every run.
- **Hand figures are sometimes wrong, and that is what they are for.** Four of the
  lead's were caught this week by a worker or a judge who was told not to bend the
  program to them. Keep telling them that.

## What is in the program that the last days' notes do not dwell on
`../README.md` lists every verb and demonstration. In short:
- **Design in the window**: walls, doors, windows, floors, rooms and room lines,
  roofs, cabinets (runs of boxes by their catalog codes), ducts, pipes, conduit,
  cable, fittings, equipment and fixtures, each drawn with a tool, moved, undone;
  levels; layouts side by side; existing, to demolish and new work; each trade's
  view; see-through.
- **The tool chest**: an item library by trade (`library/items.csv`, 48 starter
  items; `plumbline import items`, `export items`; New item, Rename item, Delete
  item in the window). A maker's sheet (PDF) is read into an item whose sizes are
  Assumed and sourced to the line of the sheet (`import spec`). A placed thing
  follows its item when the item changes, and can be turned.
  **What it is not yet, and Patrick has asked for three times: a bench.** No item
  can be looked at alone; it carries no price, unit or labor; its shape is one of
  thirteen built in; it has no places to connect to; a maker's IFC object cannot
  be made into one. `../DECISIONS.md` call 195 takes it up in steps A to F and
  `../STATUS.md` has step A's demonstration. It is at the head of what is owed.
- **Assemblies**: a starter library of 41 assemblies, 200 lines
  (`library/sixteen-divisions.csv`), brought in, applied to a wall, room, roof or
  opening, changed for another, written out and brought back.
- **Bring in**: PDF and picture plans; price lists and prior estimates (CSV, XLSX);
  assembly and item libraries; a maker's sheet; a designer's or another trade's
  model (IFC2x3 or IFC4); a scan (PLY).
- **Write out**: the estimate as a workbook with live formulas or CSV; lists for a
  supplier, a subcontractor and the PM; the verify-in-field list; the item
  library; the plan set (PDF); the building as IFC4, validated by IfcOpenShell
  (run only as an outside checker) and read back by the program.
- **Where IFC stands.** Out: walls, slabs, roofs, doors, windows, footings, spaces,
  columns and beams by level, with base quantity sets (`Qto_*`), the common
  property sets (`Pset_*Common`: existing, to demolish or new; load-bearing), and
  IDs that hold from one writing to the next. In: walls, openings, slabs, roofs
  and spaces; what cannot be placed is listed as left out. **Not yet**: a wall's
  layers as material layer sets; types; a MasterFormat classification on each
  element; a designer's ducts, pipes and equipment arriving as things in the
  model; an item of the tool chest tied to a maker's IFC object. `crates/ifc` is
  the reader and writer; `demos/export.sh` and `demos/ifc.sh` are the checks.

## Where real files are, and are not
Real job, client and supplier data never enters the repository. The real phone
scans and city plan sheets the finders were tried on were public but not openly
licensed: they lived in the session's scratch area, are not here, and no test or
demonstration reads one. `../research/real-files-first-run.md` says what they were
and where they came from, so they can be fetched again.
