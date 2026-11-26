# Contract 222WC: a plan in the second window, and what part A turned up (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole ("The main thing to
remember"; the glass rule; "Speak construction"), then plumbline/DECISIONS.md "### Call
222" (part C, points 11 to 13, is yours; part B is a later job: build nothing of it
and close no door on it), "### Call 222, part A, as built", "### Call 224" (every
numbered point), "### Call 217" (Patrick's words: why this is built), "### Call 218"
(how a view moves: the second window's plans move by the same rules and the same
code), plumbline/STATUS.md "A plan in the second window: a demonstration" (your
demonstration, with its figures), and in `crates/app/src`: `main.rs` (how the second
window and its camera are made: search `SheetWindow`, `RenderTarget::Window`,
`RenderLayers`), `sheet.rs` (the estimate window: its top, its row of tabs, how it
scrolls), `scene.rs` (the plan view of the first window: how it looks straight down,
how edges are drawn by line type, how a level is looked into, how a drawing sheet is
shown, how a click picks a thing, how the pointer catches), `nav.rs` (the wheel, the
drag, a flight), `screen.rs` (what is kept, how a window is placed, the folding),
`panels.rs` (the Show switches), `map.rs` (what the program writes for the
demonstrations to read), `demos/window.sh` and `demos/screensw.sh` (how a
demonstration is cut into sessions and reads the map).
SCRATCH is named in the message that starts you.
Notes as you go to SCRATCH/review/W-222WC-report.md, after every run. Commit nothing,
push nothing.

## Before any code, in your report
1. How the second window is drawn today (its camera, its layer, what is spawned on
   it), and how a view of the model from straight above will be drawn in it, in one
   half or in two: which cameras, which part of the window each draws to, and how a
   thing is shown to one view and not to another (a level looked into in the left
   half and another in the right; Plumbing off in the second window and on in the
   first). Say what it costs: how many more times the house is drawn, and the frame
   time measured before and after with two plans open (the map's view lines carry
   the time).
2. How the first window's plan draws edges by line type and shows a drawing sheet,
   file and line, and so what is shared. **One way of drawing a plan**: the second
   window's plan is drawn by what draws the first window's; nothing is drawn twice
   by two pieces of code.
3. How a wheel notch, a drag and a flight are worked in `nav.rs`, and how the
   second window's plans use the same functions with their own state. No second
   arithmetic for a view.
4. How a click in the second window finds the thing under it, and how the pointer's
   catches are found there, by the first window's own functions.
5. What you will write to the map (new line kinds only; nothing already on the map
   changes): which of Estimate and Plan the second window shows; for each plan what
   it shows (a level by name, or a drawing by its sheet name), the place on the
   ground at its middle, dots to the foot, and what is marked in it; the count of
   things shown in each view.
6. BIMCraft at /home/user/ipedrax/bimcraft: `grep -rln -i
   "split_view\|second_view\|viewport\|multi_window\|side_by_side" crates` and read
   the whole list; say what is there to learn from (ideas, not code to copy, unless
   it is listed in `plumbline/PORTMAP.md` as ported).
7. The estimate's row of tabs today: why it is cut, and how it will wrap (call 224,
   point 1). How the two windows will not be seen before they are placed (point 3).

## What is built
- Call 222, part C, points 11 to 13, every clause, with call 224's points 8 and 9
  (the words; what is kept). The second window's title says what it shows.
- Call 224, points 1 to 5.
- Tests FIRST where a thing can be tested without a screen (which half a pointer
  is in; a half's fit of a level at a given size; what is kept and read back; the
  tabs' lines at a given width and lettering; the price bar's giving way; the word
  on a key by operating system), run red, quoted.
- `demos/twoplansw.sh` (new) to the fourteen steps in STATUS.md, in sessions of at
  most ten minutes; its pictures in `demos/captured/twoplansw/`: Plan with one
  level; two plans (first floor beside second); the sheet beside the tracing; a
  wall marked in both windows; the estimate's tabs on two lines at 1,366 and at
  the largest lettering; a window under the window manager with its title bar. Its
  row in `plumbline/README.md`. Where a figure in STATUS.md is wrong, say which is
  right and why; do not bend the program to it.

## Rules
- Files you may touch: `crates/app/src/*.rs`, `demos/twoplansw.sh`,
  `demos/captured/twoplansw/`, `demos/screensw.sh` and `demos/captured/screensw/`
  (only to add the session under a window manager and its pictures; no existing
  step changed), `demos/window.sh` (only to add a helper, changing none),
  `plumbline/README.md` (rows). Nothing in any other crate.
- No number of the job, no meaning, no file format changes. Looking at plans in
  the second window never changes the job: the saved job is byte for byte what it
  was. What the second window shows is the program's setting, never the job's.
- **No existing expectation is edited.** With the map's variable set and no
  settings file named the second window opens on the estimate exactly as it did:
  every earlier demonstration's pictures and figures do not move. The estimate's
  tabs wrapping changes where things below the row stand when the row needs two
  lines: at 880 wide on the sample house it will. If a demonstration clicks by a
  place and not by a name and so moves, hand it back with what was seen; do not
  edit it.
- The window's crate builds with no warning (two stand today: an unused `way` and
  an unused `BUILD_H`; put them right).
- Every word on screen is a builder's (call 224, point 8); `tools/gate-language.sh`
  passes. Every control keeps its name on the map; new ones are on it by name.
- **Sharing the machine.** Run every cargo command as
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 chrt -i 0 nice -n 19 cargo ... -j2`.
  Before every window session: wait until `SCRATCH/lead-run.lock` and
  `SCRATCH/window-run.lock` are both gone and `pgrep -x rustc` finds nothing; then
  `touch SCRATCH/builder-run.lock SCRATCH/window-run.lock`, run, and remove both by
  their literal absolute paths the moment the session ends, pass or fail. **A guard
  that prints and goes on is not a guard**: the wait is a loop that does not fall
  through. Sessions in the FOREGROUND, at most ten minutes each; never hand back
  while one runs; leave no timers and no background jobs. Freeze a copy of the two
  programs before window runs (`PLUMBLINE_BIN`); delete your own old frozen copies
  by literal absolute path, keeping one. A failure is run once more, alone, before
  it is believed.
- Disk: `df -h /` before every build; under 5 GB free, delete your own old test
  programs (`target/debug/deps`, files over 20 MB that are not libraries, by a
  literal path) before going on. After every `cargo test`, delete the test
  programs it made.
- The window manager (call 224, point 5) is installed with the system's package
  tool and only ever run; nothing of it enters the repository and nothing links
  it. If it cannot be installed, say what was tried and what each said.
- Look at every picture yourself, as a person opening the program for the first
  time with a set of plans beside him. Say plainly: can two plans be read at 440
  dots each; is the line between them clear; is anything under a panel; what is
  not right.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop
  and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
`cargo test -p plumbline-app` and `-p plumbline-world` pass (counts); `twoplansw`
and `screensw` pass on your frozen build, each session alone; `glassw`, `moneyw`
and `pricew` (they read the estimate window) pass on it or stand recorded with
what was seen; the three root gates and `tools/licenses.sh` pass; the report has
the seven "before any code" answers, says what each picture shows and what is not
right.
