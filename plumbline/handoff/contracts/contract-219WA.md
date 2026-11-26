# Contract 219WA: shown before the click (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole, then
plumbline/DECISIONS.md "### Call 219" (points 1 to 4, 8 and 9 are yours; 5 to 7 are
the next two jobs: build nothing of them and close no door on them), "### Call 219,
as built" if it is there (what the model's side gives you and what a try costs),
plumbline/STATUS.md "How tools feel, in the window, the first job" (your
demonstration, with its figures), `research/review-1125-window-behaviours.md` (points
1 to 14: what the window does today, file and line; check a line before relying on
it), `research/worker-report-call219M.md`, and in the code:
`crates/world/src/editing/trying.rs` (`Change`, `try_change`, `Tried`, `Tried::said`,
`apply`), `crates/cli/src/feel.rs` (how the command line calls them), and in
`crates/app/src`: `tools.rs` and the entry state in `main.rs` (every tool, its
fields, what a click and typing do), `scene.rs` (how the real thing is drawn; the
rubber line a wall has today; how the pointer catches and names what it caught;
the ink of a clash), `panels.rs` (a thing's panel; the Show switches), `map.rs`.
SCRATCH is named in the message that starts you.
Notes as you go to SCRATCH/review/W-219WA-report.md, after every run. Commit nothing,
push nothing.

## Before any code, in your report
1. Every tool the window has, and for each: what is drawn today between picking
   the tool and the click, what the click passes to the model, and which `Change`
   that is. A tool with no `Change` is named (the model's side is then asked for
   one: say exactly what is missing; do not build round it in the window).
2. How the thing seen through will be drawn **by what draws the real thing** (the
   same mesh, the same sizes from the model), so the picture before and after the
   click is the same thing: where the shapes come from for a thing that is not in
   the job yet. No second way of drawing a wall, a door, an item or a run.
3. When a try is made: how "the pointer rests" is known (how long, how far it may
   drift), what a try costs in this build on the sample whole house (measure it:
   the model's side measured about a third of a second in a build that is not
   made for speed), and how a frame is never held up by one more than once per
   rest. If a try must be made off the frame, say how the job is kept safe.
4. Where the lettering at the pointer stands so that it is never under the
   pointer or off the window, and how a length and an angle are written (as the
   estimate writes a length; whole degrees).
5. The keys: the one that turns a thing today, the one that will let go of the
   catches while held, none clashing with a key in use (`grep -n "KeyCode::"
   crates/app/src/*.rs`), each shown on its control.
6. Every kind of catch the pointer has today, by name, and the switch each gets.
7. BIMCraft at /home/user/ipedrax/bimcraft: `grep -rln -i
   "ghost\|preview\|dry_run\|transient\|rubber" crates` and read the whole list;
   say what is there to learn from.

## What is built (call 219, points 1 to 4, 8 and 9, every clause)
Tests FIRST where a thing can be tested without a screen (which `Change` a tool's
state makes; the rest rule; the lettering's place; an angle's words; a catch
switched off), run red, quoted. `demos/shownw.sh` (new) to the ten steps in
STATUS.md, in sessions of at most ten minutes, comparing every sentence about
fitting with what `plumbline try` prints for the same change; its pictures in
`demos/captured/shownw/` (a wall seen through; the duct in clash ink with its
words; the refusal at the pointer; a door sliding with its two distances; a
lavatory turned in the hand); its row in `plumbline/README.md`. Where a figure in
STATUS.md is wrong, say which is right and why; do not bend the program to it.

## Rules
- Files you may touch: `crates/app/src/*.rs`, `demos/shownw.sh`,
  `demos/captured/shownw/`, `demos/window.sh` (only to add a helper, changing
  none), `plumbline/README.md` (a row). Nothing in any other crate: **the window
  works nothing out about fitting**; it shows what the model's try said.
- A try never touches the saved job, the undo list or what the next thing would
  be called; a thing seen through is never in the job, the estimate, the counts
  or the map's list of things.
- No number of the job, no meaning, no file format changes. **No existing
  expectation is edited**: a tool does on a click and on typing exactly what it
  did; if a demonstration moves, hand it back with what was seen.
- Call 219, point 9: the program never picks the result, never snaps to a grid,
  never shows trouble as a colour without the words and the size.
- Every word on screen is a builder's; `tools/gate-language.sh` passes. Every
  control keeps its name on the map; new ones are on it by name. The window's
  crate builds with no warning.
- **Sharing the machine.** Run every cargo command as
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 chrt -i 0 nice -n 19 cargo ... -j2`.
  Before every window session: `while [ -e SCRATCH/lead-run.lock ] || [ -e
  SCRATCH/window-run.lock ]; do sleep 10; done`; then `touch
  SCRATCH/builder-run.lock SCRATCH/window-run.lock`, run, and remove both by their
  literal absolute paths the moment the session ends, pass or fail. Sessions in
  the FOREGROUND, at most ten minutes each; never hand back while one runs; leave
  no timers and no background jobs. Freeze a copy of the two programs before
  window runs (`PLUMBLINE_BIN`); delete your own old frozen copies by literal
  absolute path, keeping one. A failure is run once more, alone, before it is
  believed.
- Disk: `df -h /` before every build; under 5 GB free, delete your own old test
  programs by a literal path before going on; after every `cargo test`, delete the
  test programs it made. One crate's tests at a time.
- Look at every picture yourself, as a person drawing his first wall. Say
  plainly: can the thing seen through be told from a thing that is there; can the
  words at the pointer be read over the model; is anything under a panel; what is
  not right.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop
  and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
`cargo test -p plumbline-app` and `-p plumbline-world` pass (counts); `shownw`,
`planw`, `movew` and `moneyw` pass on your frozen build, each session alone, or
stand recorded with what was seen; the three root gates and `tools/licenses.sh`
pass; the report has the seven "before any code" answers, says what each picture
shows and what is not right.
