# Contract 250P: a place on the house is asked of the program (demonstration worker)

You work in your own working copy, `SCRATCH/wtP` (the pushed head), with
`CARGO_TARGET_DIR=SCRATCH/tgt-P`. Read `CLAUDE.md` whole, then
`plumbline/DECISIONS.md` call 250 (this job), calls 244 and 249 (the machine rule),
`demos/window.sh` (the kit) and `demos/navw.sh` lines 55 to 100 (`project`, `ray`,
`view_of`, and the `view` line it reads from `$PLUMBLINE_MAP.view`). Notes as you go
to `SCRATCH/review/W-250P-report.md`, after every run. Commit nothing, push nothing.
You change no program source: only demonstrations and the kit.

## Before any change, in your report

1. **The list.** Every fixed dot in every `demos/*w.sh` and in `demos/window.sh`:
   a click, a drag's two ends, a pointer set down, a dive, a pixel or a box read
   from a picture. For each: the script and line, the dot, the view it is used in
   (model or plan; which window), and what it stands for, one of: **a place on the
   house or the ground in the model view** (say which: "the front door, mid
   height"), **a place of the window** (a panel, a bare corner), **a place on the
   plan** (already through `at` or not). Find what a dot stands for from the
   script's own comment, its statement (`demos/<name>.md`) and the picture, not by
   guessing; where you cannot tell, say so and leave it.
2. For each dot of the first kind, **the place in the model**: feet east, north
   and up, worked from the sample house or the job the script makes (`plumbline
   show`, the script's own figures), with where each figure came from.

## What is built

1. In the kit, one helper that turns a place in the model (east, north, up, in
   feet) into a dot of the model window, from the last `view` line at rest. It is
   `navw`'s `project` and `view_of`, moved, not written again; `navw` calls the
   kit's. It refuses, with a sentence in `$T/missing.txt` and a stop (as `at`
   does), when the view has not come to rest or the place is behind the eye or
   off the window. A second helper gives a box (two places) for a picture read.
2. Each dot of the first kind becomes its place through the helper. Nothing else
   on its line changes.

## The proof (call 250, point 3)

- A table in your report, place by place: the dot the script held, the dot the
  helper gives on the build of the pushed head with the view as the script has
  it at that step, and how far apart. **Within 3 dots, or the place is wrong:**
  work the place again; do not widen the 3.
- Every changed demonstration run on that build and passing, with no other line
  changed. `git diff` of each script shows only places.
- Red first: one place deliberately a foot off, run, and the demonstration that
  uses it fails; put back.

## What must not change

No threshold, count, colour test, sentence, hand figure or statement; no step
added or taken out; no program source; no control's name. A demonstration that
fails on the pushed head before you touch it is reported, not mended. Files you
may touch: `demos/window.sh`, `demos/*w.sh`. The `tradesw` hue bands are the
lead's (call 250, point 4): leave them.

## How it is run

Every cargo command through `tools/quiet.sh`; every window session through
`tools/window-run.sh` with `PLUMBLINE_BIN` set to a frozen copy of your build,
**at most three demonstrations to a call**; long commands in the background, to a
log; nothing left running. Look at the pictures of each changed step: the
pointer is where the script says it is. If a tool asks a permission you do not
have, stop and say so in the report; do not ask the lead to do it.

## Done when

The list, the places, the table and the red run are in the report; every changed
demonstration passes on your frozen build (results file named); `tools/gate-language.sh`,
`tools/gate-egress.sh` and `tools/gate-clientdata.sh --all` pass from the
repository root; the report says what is not right.
