# Contract 240B1: quiet, with room, part B1, the layout (window builder)

**Not issued until part A (contract 240A) is accepted; the lead names the commit.**
You are the one window builder, in /home/user/roofnerd. Read `CLAUDE.md` whole, then
`plumbline/DECISIONS.md` calls 239, 240 (with its "as built" for part A) and 242
(points 3, 4, 6 and 7), `plumbline/VISUAL.md` section G, `demos/quietw.md`, and
`research/judgment-design-1126.md` sections 2 to 6: section 3 gives each region its
place and size, section 4 the tools, section 5 where each thing goes. Where call 240
or 242 says otherwise than the judgment, the call is right. Notes as you go to
`SCRATCH/review/W-240B1-report.md`, after every run. Commit nothing, push nothing.

## Before any code, in your report

1. `quietw` run on the tree as part A left it: every line with its figure (the
   lines part B owes must be red).
2. **The way on the map, and the kit's `on` following it** (call 242, point 3),
   built and shown first on one control: with a menu closed, `on "<a control in
   it>"` presses the opener, waits for a drawn frame, presses the control. The rows
   you added to the map. A red proof: a control with no way and not in view still
   fails with the kit's present message.
3. Every existing demonstration that names a control you will move, by control
   (`grep` the scripts): the list. None is edited.
4. Every demonstration that measures a place, a share, a count in view or a region
   that this part moves, with its line and what it expects. **Not edited: the lead
   rules on each.**

## What is built

1. **The job's cluster** (top left, 40 high): the job's name, which opens the job's
   menu (Bring in, Write out, Save, Save a copy, Messages, Keys, Help with the
   first-job guide, Lettering); Undo and Redo; "Saved" or "Not saved" as a word.
2. **The price's cluster** (top right, 40 high): the price once, with its cents;
   "N to verify", which opens Verify in field in the estimate window; the counts of
   Known, Assumed and Unknown when the pointer rests on the price.
3. **The next step** on a job whose guide is not done: one line under the job's
   cluster ("Next: Add doors and windows · 3 of 10"); pressed, the ten steps; a mark
   puts it away; afterwards it is under Help.
4. **The view's cluster** (bottom right, 40 high): the level ("Level: first floor"),
   opening the levels top down with each one's switch and height, All levels first,
   Add level at the foot; Layers, saying what is off by name or by count, opening
   every layer's switch in three groups (Building; Systems; On the model) with the
   presets as one row at its head and the plans under the tracing as rows of their
   own (Set scale and Line up beside a plan's row); **the Layers sheet can be pinned
   open**; 3D and Plan; Walk; fit; the compass; View (Sun, Tilt as I dive).
5. **The tools in two levels**: Wall, Door, Window, Floor, Room, Roof by name; then
   "Cabinets and items" and "Systems", each opening upward and showing the tool
   last used from it; then Move and Delete. Room line is a choice in Room's fields
   and keeps its name and its key. A tool's fields are one strip directly above the
   tools, there with the tool and gone with it.
6. **The panel for a thing**: as tall as what it holds; its kind and name; Move,
   Copy and Delete by word; its key figures; **one main button** (Measure when its
   key figure is Assumed or Unknown; Price it when nothing prices it; none when it
   is Known and priced); then rows that open and are closed at first: Price, Where
   it came from, Work. What is there today is all still there, inside those rows.
7. **The items of the tool chest**: "All items" at the foot of "Cabinets and items"
   opens the list at the left, 320 wide, as it is today, and it closes.
8. **A new, empty job shows less and grows** (call 242, point 6).
9. **Bring in and Write out** each open a sheet whose rows are today's kinds.
10. **Tab** hides all glass and brings it back. The three panel switches and the
    standing Job and Details panels are gone.

## What must not change

- **Every control keeps its name on the map** and can be pressed by `on` through
  its way. No number of the job, no meaning, no file format, no word a builder
  would not say (a new word on screen is listed in your report for the lead). The
  saved job is byte for byte what it was. What is kept between runs
  (`screen.rs`) still reads an older file without a word.
- **No expectation of an existing demonstration is edited by you.** Where one
  fails because the layout moved what it measures, stop on that one, write what it
  expects, what it gets and a picture's name, and go on.
- Files you may touch: `crates/app/src/*.rs`, `demos/quietw.sh`, `demos/window.sh`
  (the way, and helpers), `README.md` rows. Nothing else.

## How it is run

As contract 240A: every cargo command through `tools/quiet.sh`, every window session
through `tools/window-run.sh`, long commands in the background, nothing left running.
**Look at every picture as Patrick will**: is it quiet; does the house have the
window; can a first-time user find Wall; can an estimator find a layer's switch in
two presses. Four pictures for the lead in `SCRATCH/review/240B1/`: at rest and at
work at 1600 by 960, at rest at 1366 by 768, a new job at 1600 by 960; and one of
each sheet open. If a tool asks a permission you do not have, stop and say so in
the report; do not ask the lead to do it.

## Done when

`cargo test -p plumbline-app` passes (the count); `quietw` passes every line but 16
(the estimate window: part C), at the figures in `demos/quietw.md`; every window
demonstration has been run by `tools/window-run.sh all` on your last frozen build
and passes or stands in your report with what moved; the three gates pass; the
report has the four "before any code" answers, the results, the pictures and what
is not right.
