# Handoff: re-base the building model on BIMCraft

Written 2026-11-11 by the session that built `building.rs`, for the session that takes it
over. The owner asked for this handoff in these words: *"do the re-basing on BIMCraft now
as a handoff to new fable ... do not allow it to drift ensure it will search."*

Read this whole file before anything else. It is the task, its fence, and the reason for
both. Where it and an older document disagree, this file is the newer ruling; where it
and the owner disagree, the owner wins.

## 1. Why this handoff exists

The session before you wrote a building model from nothing while a program that already
had most of it sat on the same machine, already rated "borrow" in this project's own
decisions. It also wrote claims about other products and about IFC from memory. The owner
asked whether the work leaned on him instead of searching. It did. `DECISIONS.md` call 28
is the account. Your job is the correction, and the correction is only real if you do not
repeat the fault: **you port from source you have open, and you do not write from
memory.**

Two mechanisms hold you to that. Neither is optional.

1. **`tools/portmap.sh`** (section 6). It is red now, on purpose. The task is done when it
   is green, and it cannot be made green without BIMCraft's source in front of you.
2. **The search record** (section 5). No design decision is written down without what
   was read to reach it.

## 2. The task, in one sentence

Replace the shapes in `crates/world/src/building.rs` with BIMCraft's shapes for the same
things, ported with attribution, and keep on top of them everything this project has that
BIMCraft does not.

## 3. The fence

**In.**
* Level, wall, hosted opening, floor: the fields BIMCraft gives them.
* Run and run type: BIMCraft's `Run` and `RunType`, for duct, pipe, conduit, cable.
* Fittings and connectors, and the connectivity graph (`network.rs`): so a system can say
  whether its runs join, and its fittings can be counted. This replaces "bends".
* Rooms found from their bounding walls, with area.
* Whatever the window, the sheet and the command line need changed so they still show the
  same job truthfully.

**Out. Do not build these. If you want one, write it in `PROPOSALS.md` and move on.**
* Any graphics, lighting, camera or panel-design work. The window's look is not the task.
* Drawing or editing the building in the window. That is the stretch after this one.
* Sizing, loads, code checks. BIMCraft's are Brazilian standards (NBR 5626, 8160, 5410,
  10844). None of it is ported.
* IFC export or import (deliverable D6). Read BIMCraft's writer to learn its names; do not
  port it now.
* Structure, stairs, rebar, drawings, sheets, families as a system, DXF, PDF, RVT, DWG.
* The roof (`solve.rs`, `skeleton.rs`, `roof.rs`) and the cost lines (`lines.rs`,
  `formula.rs`). They are judged and they are not yours to touch here.
* New kinds of element beyond Room and whatever fittings need.
* Renaming things for taste.

A feature nobody asked for is the named defect this whole project exists to escape.

## 4. What is true, and how it is known

**Checked by hand against the source** (BIMCraft at
`f202abd85d1cf165325e2f7b250da0b5e51f5b71`, https://github.com/IPedrax/bimcraft, MIT OR
Apache-2.0, files `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE` at its root; paths below are
under `crates/core/src/`):

| Thing | Where | What was seen |
|---|---|---|
| Level | `model.rs:131` | `name`, `elevation`, view fields, `computation_height` |
| Wall | `model.rs:249` | `type_name`, `level`, `start`, `end`, `top_level`, `height`, `base_offset`, `top_offset`, `arc`, `location`, `attach_top`, `attach_base`, `room_bounding`, `join`, `params` |
| Opening | `model.rs:827` | `type_name`, `family`, `host`, `offset`, `sill`, `mark`, `flip_facing`, `flip_hand`. Width and height are not on it: they are on its type |
| Room | `model.rs:852` | struct exists |
| Run | `model.rs:1588` | `type_name`, `level`, `start`, `end` (plan points), `offset`, `end_offset`, `justify`, `join`, `system` (a string), `params`. **A run is one straight segment.** |
| RunType | `model.rs:2467` | `width`, `depth`, `round`, `material` |
| Connector | `family.rs:430`; kinds at `family.rs:350` | structs exist |
| Network | `network.rs:91`; `pub fn network(m, system, level, tolerance)` at `network.rs:163` | exists |
| IFC classes for runs | `runs.rs:25-28` | pipe IFCPIPESEGMENT, duct IFCDUCTSEGMENT, conduit and cable tray IFCCABLECARRIERSEGMENT |
| Units | `units.rs:2` | **BIMCraft holds every length as a plain number of millimetres.** |
| Size | | `model.rs` 8,156 lines, `walls.rs` 955, `rooms.rs` 252, `runs.rs` 719, `network.rs` 428, `family.rs` 2,841; 1,648 `#[test]` in the crate |
| No status on values | grep for Known/Assumed/Unknown | nothing |

**An agent's reading, not checked by hand. Verify before you rely on any of it:**
wall corner joins in `walls.rs:298` and `walls/compound.rs`; layered wall types at
`model.rs:421`; the past-the-top opening rule in `openings_past_top_tests.rs`; how
`rooms.rs:29` finds a boundary; that fittings are `Component`s of families with typed
connectors; that joints in `network.rs` can be a run end, a run side or a connector; that
the schedule's wall area is length by height (`schedule.rs:955`); that IFC export writes
IfcSystem and not IfcDistributionSystem (`ifc.rs` near 1541); that `ifc/classes.rs` is
generated from the IFC4 ADD2 TC1 schema.

**Not known. Find out; do not assume:**
* Whether the ten system classifications in `building.rs` (`CLASSIFICATIONS`) are the
  values of IFC's `IfcDistributionSystemEnum`, or of Revit's list. They were written from
  memory. Check `ifc/classes.rs` in BIMCraft first, then buildingSMART's schema page.
* What BIMCraft does where a run meets another with no fitting.
* How BIMCraft's door and window types carry width and height.

## 5. How you search, and the record of it

The parent repository's `CLAUDE.md` gives the order. It binds you here:

1. **The machine first.** BIMCraft's source. This project's own `DECISIONS.md` (its table
   of rated candidates, and calls 3, 4, 16, 26, 27, 28). The earlier program's solver core
   at `src-tauri/core/src/` in this repository (`walls.rs` there is parapets and flashing
   heights). Where a tree has an index, a README or a docs folder, that is the first read:
   BIMCraft has `ROADMAP.md`, `docs/disciplines.md`, `docs/areas/`.
2. **The web second**, for what the machine cannot settle (the IFC schema, a trade term).
   A search that comes back thin is followed up or reported as thin. It is never filled
   in from memory.
3. **The owner last**, with what was searched and what came back.

**The rules that come from last time:**
* **If BIMCraft cannot be read, stop.** Get it with `add_repo` (owner `IPedrax`, repo
  `bimcraft`, read access), clone it beside this repository, and check out the pinned
  commit. If that is refused, tell the owner and wait. Nothing is ported from recall.
* **Read the file you are porting from, whole, before you write its counterpart.** Not a
  grep hit; the file. `model.rs` is long: read the struct, its `impl`, its checks and its
  tests.
* **A sentence about another product, a standard or a schema carries its source** (a
  file and line, or a URL you fetched), or it is not written.
* **Bevy 0.20 is newer than your training.** Its API is read from the crate's own
  examples under `~/.cargo/registry/src/*/bevy-0.20.1/examples`, not remembered.
* **Every decision goes in `DECISIONS.md`** as a new call (the next number is 29) with
  what was read to make it.

## 6. The gate

`tools/portmap.sh` checks `PORTMAP.md`. Every public item of the building model is listed
there once, as **ported** (with the BIMCraft file and line that holds the named symbol) or
**ours** (with the search that shows BIMCraft does not have it). It fails on an unlisted
item, a line that does not hold the symbol, an "ours" with no search, ported code whose
file header and `NOTICE` do not say so, or BIMCraft not being there.

It was proven red four ways before being trusted: on the tree as it stands (59 unlisted
items); on a row citing the wrong line; on an "ours" row with no search; with BIMCraft
absent. It accepted a row citing the right line.

* It is red now. That is the starting state, not a fault.
* **Never loosen it.** If it fires on honest work, fix the work or narrow the pattern with
  a comment saying why.
* It does not prove a port is faithful, only that you had the source open and said where
  each thing came from. Faithfulness is what the tests and the judge are for.
* A ported file's header follows `crates/world/src/skeleton.rs`: what it is ported from,
  the commit, the copyright line, the licence used (MIT), and **a list of what was changed
  in the port and nothing else**.

## 7. What this project has that BIMCraft does not. Keep all of it.

These are why the port is not a copy. Each must survive, and each has a test today.

1. **Every entered value is a `Val<T>`**: a value, the unit it was typed in, Known /
   Assumed / Unknown, and a source. An Unknown carries no number. BIMCraft's fields are
   bare `f64`; here each becomes a `Val`.
2. **Lengths are held as entered** (`ft`, `in`, `ft-in`) and converted only for
   arithmetic (`DECISIONS.md` call 16, an owner ruling). **Do not adopt millimetres.**
   A length with no unit, or a metric one, is refused.
3. **Every figure is worked out in `plumbline-world`**, takes the worst status of its
   inputs, and is refused with the reason when its inputs do not fit. Nothing is ever
   silently zero. A sum over nothing has no figure.
4. **Net wall area** (face less openings), refused when an opening does not fit.
5. **A system is an element** with a discipline and a classification, not a string.
6. **The takeoff by system**: runs by kind and size, a run with no size on a line of its
   own, a line no surer than the membership of what is on it, "On no system" apart from
   "System not known".
7. **What cannot be drawn is listed with why** (`placements`' second list).
8. **Elements are named by path** (`wall-0004`), not by Guid. A path never changes.
9. **The saved job** is IFC5-style JSON under `pl::` keys, written in the order things
   were added, loads back to the same bytes, refuses a field it does not know inside its
   own components, and trusts no derived figure in the file.

## 8. The figures that must not move

`demos/captured/house/takeoff.txt` is the sample house's takeoff today. After the re-base:

* Each storey's wall length, gross face, net face and floor area: **unchanged**.
* Each system's total run length in LF: **unchanged**. A run that is now several straight
  pieces still comes to the same length.
* The number of runs, and "bends", **will** change, because a run becomes straight pieces
  joined by fittings. Say what they became and why, in `STATUS.md`.
* Anything else that moves is either a fault or a decision. If a decision, it is a call in
  `DECISIONS.md` with the old figure, the new one, and the arithmetic.

The 89 tests are the same contract in code. A test is changed only when what it asserts
has been decided otherwise, and the decision is written down first. **Never delete,
skip or weaken a test to get green.**

## 9. Order of work. Stop and commit after each.

0. **Set up and prove the baseline.** Toolchain 1.97.1 (`rust-toolchain.toml`). System
   packages for the window: `libxkbcommon-x11-0`, `xdotool`, `mesa-vulkan-drivers`, `xvfb`
   (`DECISIONS.md` call 23). Then `cargo test --workspace` (89 pass), `demos/d1.sh`,
   `demos/units.sh`, `demos/d3.sh`, `tools/licenses.sh`, and from the parent repository
   `tools/gate-language.sh`, `tools/gate-egress.sh`, `tools/gate-clientdata.sh --all`.
   If any of these does not give what `STATUS.md` says, stop and say so.
1. **Read, and write nothing but the map.** Read BIMCraft's files for section 4's rows.
   Fill `PORTMAP.md` for the model **as it stands**: each of the 59 items as what it will
   become (ported from where) or ours (searched how). Settle section 4's "not known" list.
   Write call 29: the mapping you have decided, including what a run becomes. Commit.
   **No model code before this commit.**
2. **Level, wall, opening, floor.** Tests first where a figure is at stake.
3. **Run and run type.**
4. **Connectors and the network.** A system can now say what is not joined.
5. **Rooms.**
6. **The window, the sheet, the command line**: only what the new model makes necessary.
7. **Demonstrate and judge** (section 10).

If a step turns out larger than it looks, finish the step you are in, commit, and write
what remains in `STATUS.md`. A smaller thing done and judged is worth more than the whole
thing half-ported.

## 10. Done means

* `tools/portmap.sh` passes.
* `cargo test --workspace` passes, with new tests for what was ported and hand-worked
  figures in them.
* All five demos pass (`demos/d1.sh`, `units.sh`, `d3.sh`, `d2.sh`, `house.sh`), and every
  picture the last two save has been opened and read, as a person would, at full size.
* `tools/licenses.sh` and the three repository gates pass.
* **An independent judge has tried to break it.** A fresh agent, **Opus or Sonnet, never
  Fable**, told to find faults, not to approve. Give it the code, not your conclusions.
  Fix what it finds with a regression test for each, and record the judgment in
  `STATUS.md` the way the eight before it are recorded. A deliverable is not described as
  done until then, and the fixes are described as unjudged until they are.
* `STATUS.md`, `DECISIONS.md`, `SPEC.md`, `PORTMAP.md`, `NOTICE` and `LICENSES.md`
  ("Borrowed code") say what is now true. `QUESTIONS.md` Q13 is updated.
* Committed and pushed to the branch you were given. **No pull request** unless the owner
  asks. **Never force-push.**

## 11. Standing rules, restated because the documents that hold them are not in the clone

The owner's brief and first handoff are in `refs/`, which git ignores, so you will not
have them. What binds you from them:

* Permissive licences only (MIT, Apache-2.0, BSD, ISC, Zlib and the like). GPL, AGPL and
  LGPL code is read for reference and never linked or copied. `LICENSES.md` stays current.
* A job's data never leaves the machine. No network call in the product.
* **Flag, never fabricate.**
* Subagents run on Opus or Sonnet, never Fable.
* Nothing is described as done until a judge has tried to break it.
* A claim is checked against its source before it is written down.
* Work outside the task goes in `PROPOSALS.md`, not into the code.
* No client data in the public tree: every job, name, figure and price in tests, demos and
  documents is made up.
* Plain words. No invented vocabulary where the trade or the standard has a word.

## 12. Things that cost time last time

* **A background agent is lost if the machine restarts while you sit idle.** Keep working,
  or wait on it actively, while a judge runs.
* The window is slow on the software renderer. `PLUMBLINE_GRAPHICS=plain` starts and draws
  much faster; use it for everything but the final pictures. Allow 40 s after launch and
  12 s after each F12 with full graphics.
* The demo scripts click fixed pixels. If you move a panel, a tab or a row, the click
  misses without an error: open the picture and look.
* `rm` on a path built from a shell variable is blocked by the harness. Delete by literal
  path.
* `cargo build` of the application from cold takes many minutes.
* The sample house is written in code (`sample::house()`); the saved-file form of every
  component is in `SPEC.md` section 4.
* Element paths in the sample will change if the order things are added in changes, and
  `demos/house.sh` and the tests find things by name for that reason. Keep doing that.

## 13. Where things stand as this is written

* Branch `claude/nifty-tesla-qmn44n` of `bitumenmachina/roofnerd`, plumbline in the
  `plumbline/` folder. Last commits: `1951204` (call 28), `990ea18` (eighth judgment's
  fixes), `f040825` (the whole building).
* 89 tests pass. Five demos pass. The eighth judgment's fixes are not judged.
* `tools/portmap.sh` fails with 59 unlisted items. That is where you start.
