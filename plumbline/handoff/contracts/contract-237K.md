# Contract 237K: a press waits until the window has drawn (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names.
Three files are already changed there and are yours to carry: the double-click repair
(`crates/app/src/nav.rs`, `crates/app/src/scene.rs`, `demos/twoplansw.sh`). Read
`CLAUDE.md` whole ("Use it" above all), then `plumbline/DECISIONS.md` "### Call 237",
`plumbline/STATUS.md` ("The window", "What is not right"), `demos/window.sh` (the kit
every window demonstration uses), `crates/app/src/map.rs` (what the program writes for
a script to read), `tools/window-run.sh` and `tools/quiet.sh`. SCRATCH is named in the
message that starts you. Notes as you go to `SCRATCH/review/W-237K-report.md`, after
every run. Commit nothing, push nothing.

## Why

The window demonstrations click by the clock: a press is held for a fixed time and the
script sleeps a fixed time after it. The renderer here is software, a frame can take a
second, and on a loaded machine longer; a press that begins and ends inside one frame
is lost, and a double-click whose presses fall more than half a second of the
program's own clock apart is two clicks. On 2026-11-26 three demonstrations failed for
this and a day's evidence went with them. The demonstrations are made to wait for the
window, not for the clock.

## Before any code, in your report

1. How the kit waits today: every helper in `demos/window.sh`, what it sends and how
   long it sleeps. Then the direct `xdotool` presses in the `demos/*w.sh` scripts that
   do not go through the kit: a count by script, and the kinds (a press, a key, a
   drag, a wheel notch, a double-click).
2. What the program can say so that a script knows a frame has been drawn since a
   press: what `map.rs` writes today and when, and the smallest addition (a count of
   frames drawn, in a file beside the map, written only when `PLUMBLINE_MAP` is set).
   What it costs each frame.
3. Every double-click in every demonstration (`grep` for two `mousedown` close
   together, `click --repeat`, `double`), with the times between its presses, set
   against `nav::again`. Which pass only because a frame is slow here.

## What is built

1. **The program says how many frames it has drawn**, beside the map, when
   `PLUMBLINE_MAP` is set and not otherwise. Nothing already written to the map
   changes. A test where it can be tested without a screen, written first, run red,
   quoted.
2. **The kit waits on frames.** A helper `drawn [n]` returns when the window has drawn
   `n` more frames (default 2), with a ceiling in seconds after which it says so
   plainly and fails the script (a window that has stopped drawing is a failure, not
   something to sleep through). `click`, `shift_click`, `typed`, `esc`, `len`, `laser`,
   `shot` and the rest hold a press until a frame has been drawn with it down, and
   wait for frames after it is let go, in place of their fixed sleeps. One kit helper
   `double_click X Y` whose two presses fall inside the half second at a fast frame
   and inside a frame or two at a slow one; `twoplansw.sh` and `navw.sh` use it.
3. **The scripts' own presses go through the kit** where a kit helper does the same
   thing. This is a change of pacing only.

## What must not change

- **No expected figure, no step, no check of any demonstration is edited, added or
  taken out.** Only how a press is paced and what it waits for. If a demonstration
  cannot pass without a change to what it expects, stop on that one, write what was
  seen, and go on to the next: the lead rules on expectations.
- No number of the job, no meaning, no file format. Every control keeps its name on
  the map. `tools/gate-language.sh` passes. The window's crate builds with no warning.
- Files you may touch: `crates/app/src/map.rs` (and `main.rs` only to register what it
  needs), `crates/app/src/nav.rs` and `scene.rs` (only what is already changed there),
  `demos/window.sh`, `demos/*w.sh` and the other scripts that drive the window
  (`grep -l xdotool demos/*.sh`). Nothing else.

## How it is run

- **Every cargo command goes through `tools/quiet.sh`**, with `CARGO_INCREMENTAL=0
  CARGO_PROFILE_DEV_DEBUG=0` and `-j2`. **Every window session goes through
  `tools/window-run.sh <names>`** (it freezes the two programs, waits until no cargo
  is running, holds the lock, runs a failure once more). Never start a window session
  any other way and never run cargo while one of yours is going. The lead's own run of
  three demonstrations may hold the lock when you start: your first build waits for
  it, an hour or more. That is not a hang; read and write your report's first part
  meanwhile.
- A demonstration can take forty-five minutes. Start `tools/window-run.sh` with
  `run_in_background` and wait for its notice; do not poll it with sleeps; leave no
  timers and no background jobs when you hand back.
- Disk: `df -h /` before every build; under 5 GB free, delete your own old test
  programs by a literal path.
- Look at the pictures of every demonstration you run, as a person opening the program
  for the first time, and say what is not right.
- If a tool asks a permission you do not have, stop and say so in the report; do not
  ask the lead to do it.

## Done when

- `cargo test -p plumbline-app` passes (the count; it is 109 today, plus yours).
- Each of these passes by `tools/window-run.sh`, on your last frozen build:
  `twoplansw`, `navw`, `tradesw`, `glassw`, `moneyw`, `pricew`, `screensw`, `movew`,
  `planw`, `clashw`. Give `target/window-run/results.txt` whole in your report. Where
  one fails twice, say at which step, with the figures, and whether it is the pacing,
  the program or the expectation.
- **One trial of what this was for**, stated as the single exception to the rule
  above: `movew` run once more by hand with a build of the command line going beside
  it (`touch crates/world/src/lib.rs`, then `cargo build -p plumbline -j2` in a loop
  until the demonstration ends). Say whether it passed and how long it took against
  its run alone. The rule itself does not change whatever this shows.
- The three gates pass. The report has the three "before any code" answers, the
  results, what each picture set shows and what is not right, and the list of scripts
  you did not convert and why. The other thirty-five window demonstrations are run by
  the lead after you hand back.
