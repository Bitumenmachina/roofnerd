# Contract 222WA: any screen, and work never lost (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole ("The main thing to
remember"; the glass rule), then plumbline/DECISIONS.md "### Call 222" (part A,
points 1 to 6, is yours; B and C are later jobs: build nothing of them, and close
no door on them), "### Call 220" and "### Call 217" (why), plumbline/STATUS.md "Any
screen, and work never lost: a demonstration", `research/review-1125-code-audit.md`
(sections 3 and 5: what ties the window to one size, file and line), and in
`crates/app/src`: `main.rs` (how the two windows are made: search `resolution`,
`WindowPosition`; `keys`; how a job is opened and saved), `scene.rs` (the panel
widths at its top; what is positioned by the window's size), `glass.rs` (the
frosted picture and when it is made), `panels.rs` (`panels_on`, `status_line`, the
panel switches, how a panel scrolls), `theme.rs` (sizes of lettering), `map.rs`.
SCRATCH is named in the message that starts you.
Notes as you go to SCRATCH/review/W-222WA-report.md, after every run. Commit nothing,
push nothing.

## Before any code, in your report
1. Every size and place in the window's code that is a fixed number of dots (the
   two windows, the panels, the bars, anything placed by an absolute figure), each
   with file and line, and for each whether it follows the window today.
2. What happens today when the model window is resized by hand (try it: `xdotool
   windowsize` on the virtual display, and look at the picture): what breaks.
3. How the program learns the size of the screen it is on, and how the virtual
   display is given other sizes for the demonstration (`xvfb-run -s "-screen 0
   1366x768x24"`). The smallest size at which every control can be reached, found
   by trying.
4. Where the settings file lives (the operating system's own place for a
   program's settings, through a crate already linked if one gives it; no new
   crate without its licence passing `tools/licenses.sh`), its form (plain text a
   person can read), and how the demonstrations run without it (the map's
   environment variable is set for every demonstration: when it is set the file
   is neither read nor written, unless the demonstration names one). No setting
   is kept in the job.
5. The recovery copy: its name and place beside the job, when it is written, how
   it is known to be newer, and the proof that it is byte for byte what Save
   would write. What the window says and the two controls' names. What happens to
   it on Save, on a clean close, and when the job itself is brought in fresh.
6. The keys: how Ctrl is read today and how Command will be read beside it.

## What is built (call 222, part A, points 1 to 6, every clause)
Tests FIRST where a thing can be tested without a screen (where two windows go on
a screen of a given size; which panels fold at which width; the settings read and
written; the recovery copy's bytes; larger lettering's arithmetic), run red,
quoted. `demos/screensw.sh` (new) to the eight steps in STATUS.md, in sessions,
its pictures in `demos/captured/screensw/` (one of each window at each of the
three screens, and the strip of the resize); its row in `plumbline/README.md`.

## Rules
- Files you may touch: `crates/app/src/*.rs`, `crates/app/Cargo.toml` (only for a
  crate whose licence you have shown passes), `demos/screensw.sh`,
  `demos/captured/screensw/`, `demos/window.sh` (only to add a helper, changing
  none), `plumbline/README.md` (a row). Nothing in any other crate.
- No number of the job, no meaning, no file format changes; the job is saved
  exactly as it was. **No existing expectation is edited**: every window
  demonstration opens the two windows at the sizes it always did when the map's
  variable is set and no settings file is named, so their pictures and figures
  do not move. If one moves, hand it back with what was seen.
- Every word on screen is a builder's ("Take the recovered work", "Larger
  lettering"); `tools/gate-language.sh` passes. Every control keeps its name on
  the map; new ones are on it by name.
- TIMED WINDOW RUNS: before every window session wait until `pgrep -x rustc` finds
  nothing, `touch SCRATCH/window-run.lock`, run, and remove the lock by its
  literal absolute path the moment the session ends, pass or fail. Sessions in
  the FOREGROUND, at most ten minutes each; never hand back while one runs; leave
  no timers. Freeze a copy of the two programs before window runs
  (`PLUMBLINE_BIN`); delete your own old frozen copies by literal path, keeping
  one. No cargo beside a timed run. A failure is run once more, alone, before it
  is believed. Watch the disk (`df -h /`; under 5 GB free delete your own old test
  programs by literal path).
- Look at every picture yourself, as a person opening the program for the first
  time on a small screen. Say plainly what is crowded, what is cut off, what is
  not right.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop
  and say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it.

## Done when
`cargo test --workspace` passes (count); `screensw`, `glassw`, `moneyw`, `benchw`
and `pricew` pass on your frozen build, each run alone, or stand recorded with
what was seen; the three root gates and `tools/licenses.sh` pass; the report has
the six "before any code" answers, says what each picture shows and what is not
right.
