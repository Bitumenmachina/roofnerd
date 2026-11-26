# Contract 240A: quiet, with room, part A (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read `CLAUDE.md` whole ("Quiet, with room";
"The window is glass over the model"; "Speak construction"), then
`plumbline/DECISIONS.md` "### Call 239" and "### Call 240" (every numbered point),
`plumbline/VISUAL.md` section G, **`demos/quietw.md` (your demonstration, written
first)**, and `research/judgment-design-1126.md` whole: its section 1 says what is
wrong and where, sections 3, 4 and 6 are the sizes, the type and the colours you build
to. Then in `crates/app/src`: `theme.rs` (every colour and size), `panels.rs` and
`glass.rs` (what is on the model window), `tools.rs` (the tools bar), `sheet.rs` (the
estimate window: not yours in this part but for one thing below), `map.rs`.
SCRATCH is named in the message that starts you. Notes as you go to
`SCRATCH/review/W-240A-report.md`, after every run. Commit nothing, push nothing.

## Before any code, in your report

1. `demos/quietw.sh` written and **run on the window as it is, red**: each line of
   the table in `demos/quietw.md` with the figure the script gets beside the figure
   the judge measured. Where they differ by more than the table allows, say which is
   right and why, and put the script right before going on. What you added to the map
   for it (new lines only).
2. The test in `theme.rs` for lines 8 and 9, written and run red (quote it).
3. Every place lettering is sized in the window's crate (`grep -n "font_size\|px("`),
   counted by size, and the four sizes each will take.
4. Which demonstrations measure something this part moves (a panel's box or share, a
   count of coloured dots, a region of the screen, a control's place): list each with
   its line and what it expects. **Do not edit them.** The lead rules on each.

## What is built (call 240, point 6, part A; no control moves into a menu in this part)

1. **No panel for a thing when nothing is selected.** With nothing selected the right
   panel is not there (today it repeats the estimate). It comes with a selection and
   goes with it. The job's price stays in the top bar, once.
2. **The tools**: each at least 56 by 52 with its word at 12 px, whole, 6 px clear of
   each edge; no outline and no fill on a tool at rest (a wash under the pointer; the
   accent wash and a 2 px bar when in hand); 6 px between tools, 16 between groups;
   the line of help text only on a new, empty job. The eighteen stay in this part
   (the second level is part B): if they do not fit one row at 1366 wide at these
   sizes, they wrap to a second row inside the dock, never shrink and never clip.
3. **No key labels on the glass at rest.** A key is said when the pointer rests on its
   control (a small label by the control), and stays in the map under the control's
   name so that every script still finds it.
4. **Four sizes of lettering**: 11, 13, 15, 20, and 12 for a tool's word. Headings in
   sentence case at 13, not capitals at 11.
5. **How sure, as a mark**: none for Known, a half-filled amber circle for Assumed, an
   open red circle for Unknown, before the figure; the figure in ink. Assumed's
   lettering colour becomes `#8A5508`. No more than two red words in a panel or card:
   a block with several things not yet priced says so once.
6. **Trade colours off amber and red**, as the judgment's section 6 gives them, and
   only in the model and beside a layer's name.
7. **One place for a message**: centred above the tools, never over what is selected;
   it fades; the last messages can be read again from the status line.
8. **The two top bars made small**: the job's name, Undo and Redo, Saved as a word; the
   price once, and "N to verify". No box round each.
9. **Nothing cut** at 1366 by 768, 1600 by 960, 1920 by 1080, 2560 by 1440, at every
   step of lettering, for everything this part touches. The estimate window's row of
   tabs must not cut a tab at 880 wide (it may still take two lines until part B).

## What must not change

- **Every control keeps its name on the map**, and is still reached the way it is
  today. No number of the job, no meaning, no file format, no word a builder would not
  say. The saved job is byte for byte what it was.
- **No expectation of an existing demonstration is edited by you.** Where one fails
  because this part moved what it measures, stop on that one, write in your report
  what it expects, what it now gets and a picture's name, and go on to the next.
- Files you may touch: `crates/app/src/*.rs`, `demos/quietw.sh`, `demos/window.sh`
  (only to add a helper), `README.md` (a row for `quietw`). Nothing else.

## How it is run

- Every cargo command through `tools/quiet.sh` with `CARGO_INCREMENTAL=0
  CARGO_PROFILE_DEV_DEBUG=0 -j2` and the `CARGO_TARGET_DIR` the lead names. Every
  window session through `tools/window-run.sh`. The lead is running all forty-five
  demonstrations in parts and another worker is building: waits of an hour are
  normal and are not a hang. Long commands with `run_in_background`; wait for the
  notice; leave nothing running when you hand back.
- **Look at every picture as Patrick will**: is it quiet; can each tool be read; does
  the house have the window. Say plainly what is still busy and still cramped. Put
  one picture at rest and one at work, at 1600 by 960 and 1366 by 768, in
  `SCRATCH/review/240A/` for the lead.
- If a tool asks a permission you do not have, stop and say so in the report; do not
  ask the lead to do it.

## Done when

`cargo test -p plumbline-app` passes (the count); `quietw` passes the lines part A owes
(`demos/quietw.md`, "In two parts") and prints the others as not yet owed, with their
figures; these pass by `tools/window-run.sh` on your last frozen build, or stand in
your report with what moved: `glassw`, `screensw`, `navw`, `tradesw`, `moneyw`,
`pricew`, `planw`, `movew`, `clashw`, `fixes209w`, `trimw`, `benchw`; the three gates
pass; the report has the four "before any code" answers, the results, the four
pictures and what is not right.
