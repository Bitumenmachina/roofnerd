# Handoff: draw and edit the building in the window

Written 2026-11-11 by the session that re-based the building model on BIMCraft and built
D4 and D5, for whichever session takes this stretch (it may be the same one after its
context turns over). The owner's order of work, set the same day, is: (a) re-base on
BIMCraft, done and judged; **(b) draw and edit walls, openings, runs and fixtures in the
window, this handoff;** (c) fittings, connections and clash detection; (d) money for the
whole building; (e) IFC4 export and import starting from BIMCraft's writer, and xlsx out;
(f) publishing.

Read this whole file, then `HANDOFF_REBASE.md` sections 5, 11 and 12 (how you search,
the standing rules, what cost time), which still bind and are not repeated here. Where
this file and the owner disagree, the owner wins.

## 1. Why this handoff exists

Today a building gets into plumbline one way: `sample::house()` builds it in Rust. The
window shows it, walks it, selects in it, and changes a roof's pitch and eave, a
condition, a price, a material. Nobody can put a wall in. A program a builder cannot
draw a house in is a viewer. The owner's stop condition says: "from a clean clone and
the README alone a person can ... model a house with its roof and MEP in the window".

BIMCraft, on this machine, has the whole of this already worked out for a plan canvas:
what a click snaps to, what a wall command takes, how a typed length steers the next
point, how an action is undone. **You port from that source, open in front of you. You
do not write a snapping rule or a drawing tool from memory.**

## 2. The task, in one sentence

In the plan view of plumbline's window a person draws walls (snapped, with typed
lengths), places doors and windows in them, draws ducts, pipes, conduit and cable,
places fixtures and equipment, moves and deletes what is selected, and undoes any of it,
and every element so made is the same element the file, the takeoff and the estimate
already know, with every value it was given marked Assumed and sourced to the window.

## 3. The fence

**In.**
* Snapping in plan, ported from BIMCraft `crates/core/src/snap.rs`: endpoint, midpoint,
  intersection, perpendicular, nearest on a wall's line, and the square (axis-locked)
  guide. Restated in feet with this program's one tolerance.
* Drawing a straight wall by two clicks, chained; a typed length for the leg being
  drawn (BIMCraft `draw.rs`, `reading` and `typed`). The wall's type, height or top
  level come from a current setting shown on screen, never from a hidden default.
* A door or a window placed on a wall by a click along it (BIMCraft `commands.rs`,
  `opening_of`): its type from the current setting, its offset the clicked place.
* A run (duct, pipe, conduit, cable) drawn as straight pieces by clicks, at a stated
  height, on a stated system (BIMCraft `commands.rs`, `runs_of`).
* A fixture or a piece of equipment placed by a click (BIMCraft `stood_of`).
* Move (by two clicks or a typed distance) and delete of what is selected.
* Undo and redo of every one of the above (BIMCraft `engine.rs`: `Session`, `Step`,
  `transaction`), and of the edits the window already makes.
* A new storey, and choosing the storey being drawn on.
* The command line gains the same verbs (`wall`, `door`, `window`, `run`, `place`,
  `move`, `remove`), so a demonstration can check the window against it.

**Out. If you want one, write it in `PROPOSALS.md` and move on.**
Curved walls, arcs, splines, rectangles and polygons as wall shapes; rooms placed by
click (a room is still a point: add it at the command line); fittings put in
automatically where runs meet (that is stretch (c)); families; dimensions and
annotation; copy, array, mirror, rotate, align, trim and extend; drawing in the 3D
view (plan only); multi-select; a properties panel that edits every field; snapping to
a traced image or a point cloud.

## 4. What is true, and how it is known

**Checked by hand against the source** (BIMCraft at `/home/user/ipedrax/bimcraft`,
commit `f202abd85d1cf165325e2f7b250da0b5e51f5b71`, MIT OR Apache-2.0):
* `crates/core/src/snap.rs` is 1,682 lines. `Kind` (line 34) is Endpoint, Midpoint,
  Intersection, Centre, Perpendicular, Quadrant, Tangent, Nearest, Point, Anchor, Angle,
  Free ("Nothing to snap to: the nearest 10 mm"). `snap(m, level, p, reach, from,
  square, o)` is at line 664 and returns a `Found`.
* `crates/core/src/draw.rs`: `Shape` (line 23) has sixteen shapes; `Line` is "A line
  between two points; chained, the next starts where it ended". `reading` (279) gives
  the numbers a shape shows while it is drawn; `typed` (300) turns typed numbers into
  the next point.
* `crates/core/src/engine.rs` is 864 lines: `Session` (152), `execute` (334),
  `history` (272), `steps(redo)` (277), `transaction` (648), `merge_steps` (682).
* `crates/core/src/commands.rs` is 14,220 lines: "The command set: everything the CLI,
  scripts and MCP can do. Parameters are JSON, lengths are millimetres." `opening_of`
  (7473), `stood_of` (8662), `runs_of` (8776).
* The plan canvas that drives them is `crates/ui-egui/src/canvas.rs`, 4,251 lines.
* BIMCraft has an xlsx writer (`crates/core/src/xlsx.rs`, 250 lines) and an IFC writer
  (`ifc.rs`, 2,774 lines) and importer (`import.rs`, 4,895 lines): for stretch (e).
* BIMCraft has **no clash detection** of its own: `grep -rl 'clash\|interference'
  crates/core/src` finds only `inquiry.rs`, `link.rs` and `import.rs`, and `inquiry.rs`
  line 3 speaks of ids "pasted from ... a clash report". For stretch (c), search wider.

**In plumbline, checked:**
* `Job::add(parent, kind, name)`, `Job::put(path, component)` and `Job::remove(path)`
  exist (`crates/world/src/job.rs`). `put` does not run the entry checks (SPEC section
  7); a drawing tool has to call the `building::check_*` function for what it puts.
* The window has a plan view (`T`), picking by click, a typed entry card
  (`model::Entry`, `panels.rs`), and saves with `F2`. It has no drawing tool and no undo
  (SPEC section 7: "No undo").
* Everything typed in the window is entered Assumed with the source
  `model::SOURCE` ("set in the 3D view; not yet confirmed against a drawing").
* The window demonstrations run on a virtual display with `xdotool`
  (`demos/house.sh`, `demos/d5w.sh` are the patterns). The sheet window sits at screen
  x 1610; model pictures are 1600 by 960.

**Not known. Find out; do not assume:**
* How BIMCraft's undo stores a step (a whole-model snapshot, or a diff), and whether
  that suits a bevy_ecs world. Read `engine.rs` 100 to 200 and 334 to 560 first.
* What `Reach` and `Snaps` hold, and what the plan canvas does between a click and a
  command (`canvas.rs`): which part is BIMCraft's rule and which is egui plumbing.
* Whether a wall drawn onto another wall's line splits it, joins it or is refused in
  BIMCraft, and what plumbline's rooms do in each case.
* How Bevy 0.20 gives a pointer position on the plan (the window already reads one out
  in its status bar: start from that code, `panels::write_pointer`).

## 5. The gate

`tools/portmap.sh` accounts for every public item of `building*.rs` against BIMCraft.
**Widen it, before writing any drawing code**, to cover the new files this stretch adds
(snapping, drawing, the undo stack): the same rule, ported with a line that holds the
symbol, or ours with a search that is run. Prove it red on an unlisted item in a new
file, and say so in `STATUS.md`. Never loosen it.

## 6. The figures that must not move

The sample house and the sample roof are not edited by this stretch. After every step:
`demos/house.sh`, `demos/d1.sh`, `d3.sh`, `d4.sh`, `d5.sh` exit 0 with their hand-worked
figures unchanged (first-floor walls 210 LF, rooms 1,057.95 SF, every system's LF; D5's
job cost $25,292.43). `plumbline check` on both samples: same bytes.

## 7. Order of work. Stop and commit after each.

0. Read: `snap.rs` whole; `draw.rs` 1 to 360; `engine.rs` whole; `commands.rs` round
   `opening_of`, `stood_of`, `runs_of` and the wall command; `canvas.rs` for the wall
   tool's path from click to command. Write what you found in `DECISIONS.md` (what is
   taken, what is not, what was changed and why) before any code. Write "How its
   demonstration will be run" in `STATUS.md`.
1. Widen the gate; prove it red.
2. The verbs in the world crate and at the command line (`wall`, `door`, `window`,
   `run`, `place`, `move`, `remove`, `storey`), each through the entry checks, each with
   a test worked by hand. A demonstration script builds a small house from an empty job
   at the command line and checks its takeoff against figures worked by hand.
3. Undo and redo in the world crate, tested: any sequence of the verbs undone leaves
   the file byte for byte what it was.
4. Snapping in the world crate, ported, tested against cases worked by hand.
5. The tools in the window, one at a time (wall; door and window; run; place; move and
   delete; undo), each with the current settings on screen and the snap shown.
6. A window demonstration that draws a small house with a roof, a door, windows, a
   duct, a pipe and a fixture from an empty job, saves it, and checks the saved file's
   takeoff against the command-line house of step 2: the same figures.
7. An independent judge (Opus or Sonnet, never Fable): tries to draw what should be
   refused, to get an unflagged value into the file through the window, to undo into a
   state that never existed, and to snap to something that is not there.

## 8. Done means

* `tools/portmap.sh` green over the widened set; all tests, all demonstrations, the
  three repository gates and `tools/licenses.sh` pass.
* A person can start from an empty job and model the small house of step 6 in the
  window with no file editing and no command line, and the README says how.
* Every value a tool wrote is Assumed with the window as its source until someone
  confirms it; nothing a tool wrote skipped an entry check.
* Every picture of the window demonstration was opened and read at full size.
* A judge has tried to break it, each finding is fixed with a test or left and said,
  and `STATUS.md` says which fixes have not themselves been judged.
* `STATUS.md`, `SPEC.md`, `DECISIONS.md`, `PROPOSALS.md`, `README.md`, `NOTICE` and
  `PORTMAP.md` are true.

## 9. Where things stand as this is written

Branch `claude/nifty-tesla-qmn44n`, commit `016ac79`. D4 is judged (tenth judgment) and
its fixes are in; D5 is demonstrated at the command line and in the window and is with
its judge (the eleventh). The brief's D6 (interchange) is the owner's stretch (e) and
D7 (capture) waits on his ruling. Open questions for the owner: `QUESTIONS.md` Q1, Q3,
Q5, Q7, Q11, Q13, Q14; none blocks this stretch.
