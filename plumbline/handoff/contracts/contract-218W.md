# Contract 218W: how the window moves (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole, then
plumbline/DECISIONS.md "### Call 218" (every numbered point is yours), "### Call 217"
(Patrick's words: what this is for), "### Call 62" and "### Call 63" (what the feel
was built from, with its numbers and their sources: keep those numbers unless call
218 changes one), plumbline/STATUS.md "How the window moves: a demonstration" (your
demonstration, with its figures), `research/review-1125-window-behaviours.md`
(points 15 and 16: what the camera does today, file and line),
`research/review-1125-games-and-editors-pass2.md` (Google Earth's own keys, Unity's
and Unreal's), `demos/glassw.sh` (how movement is measured today: your work must
leave every figure of it where it is), and in `crates/app/src`: `scene.rs`
(`Orbit`, `Feel`, `orbit_input`, `place_camera`, the limits, the near and far of
what is drawn, how a double-click flies, how a label is shown or dropped), `main.rs`
(`keys`: every key in use), `panels.rs` (the view bar: 3D, Plan, Walk, Frame, Sun),
`map.rs` (what the program writes for the demonstrations to read).
SCRATCH is named in the message that starts you.
Notes as you go to SCRATCH/review/W-218W-report.md, after every run. Commit nothing,
push nothing.

## Before any code, in your report
1. The camera as it is: every limit, every rate and every time in the code, each
   with file and line, set beside call 63's table. Say which figure of call 218
   needs which of them changed.
2. Every key the window reads today (`grep -n "KeyCode::" crates/app/src/*.rs`),
   with what it does and whether it is shown on a control. Then the keys and
   buttons this work adds, none clashing, each shown on its control.
3. How near and far things are drawn today (the two distances past which nothing
   is drawn), and how they will follow the view so that a receptacle at 8 in and
   the lot at 400 ft are both whole. How the view is kept from passing through
   the ground and through a wall's face, and what "see-through" does to that.
4. What decides today whether a label is shown, and the one rule by size on screen
   that will decide it (point 2 of the call), with the sizes.
5. What you will write to the map each frame the view moves (one line: the time,
   where the view stands, what it looks at, how far, tilt, turn), so that
   `demos/navw.sh` reads curves. Nothing already on the map changes.
6. BIMCraft at /home/user/ipedrax/bimcraft: `grep -rln -i "tilt\|compass\|north_arrow\|fly_to\|zoom_about\|orbit_about" crates`
   and read the whole list; say what is there to learn from (its camera arithmetic
   is ideas, not code to copy: call 63).

## What is built (call 218, points 1 to 9, every clause)
Each with a test where it can be tested without a screen (the limits, a notch's
arithmetic, the tilt against height, a flight's curve, the label rule), written
FIRST and run red, quoted.

## The demonstration
`demos/navw.sh` (new) to the eleven steps in STATUS.md, reading the view's lines
from the map; its pictures in `demos/captured/navw/`, among them a strip of at
least eight pictures across the dive of step 2 and one across the flight of step
7, for a person to look at; its row in `plumbline/README.md`. Where a figure in
STATUS.md is wrong, say which is right and why; do not bend the program to it.
The lead's figures are worked from a tenth a notch and a 400 ft limit: if you
change either, say so and rework the count of notches.

## Rules
- Files you may touch: `crates/app/src/*.rs`, `demos/navw.sh`,
  `demos/captured/navw/`, `demos/window.sh` (only to add a helper, changing none),
  `plumbline/README.md` (a row). Nothing in any other crate.
- No number of the job, no meaning, no file format changes. The saved job is byte
  for byte what it was after any amount of looking round. **No existing
  expectation is edited**: `glassw`, `movew`, `trimw`, `dressw`, `moneyw` measure
  things the camera shows; if one moves, hand it back with what was seen.
- Every word on screen is a builder's: a north arrow, not a compass rose or a
  gizmo; "Tilt as I dive". `tools/gate-language.sh` passes. Every existing control
  keeps its name on the map; new ones are on it by name.
- Window sessions run in the FOREGROUND (a timeout of at most ten minutes each; a
  long demonstration is cut into sessions); never hand back while one is running;
  leave no timers behind. Freeze a copy of the two programs before window runs
  (`PLUMBLINE_BIN`); delete old frozen copies by literal absolute path, keeping
  one; **no cargo beside a timed run** (it cost three false failures on
  2026-11-25); a failure is run once more, alone, before it is believed. Watch the
  disk (`df -h /`; under 5 GB free delete your own old test programs by literal
  path).
- Look at every picture yourself, as a person opening the program for the first
  time. Say plainly whether the dive reads as one move, whether the tilt is a
  pleasure or a nuisance, whether the north arrow can be read, whether anything
  is under a panel, and what is not right.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop
  and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
`cargo test --workspace` passes (count); `navw`, `glassw`, `movew` and `moneyw`
pass on your frozen build, each run alone, or stand recorded with what was seen;
the three root gates and `tools/licenses.sh` pass; the report has the six "before
any code" answers, says what each picture shows and what is not right.
