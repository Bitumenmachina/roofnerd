# Contract 205W: a shape seen and changed on the bench (window builder)

You are the one window builder, in /home/user/roofnerd, on the commit the lead names
when it starts you (the tree is clean). Read CLAUDE.md whole, then
plumbline/DECISIONS.md "### Call 205" (point 6 is yours) with "Call 205, the model's
half, as built", "### Call 203" and "### Call 195" before it; plumbline/STATUS.md "A
shape on the bench: a demonstration"; plumbline/research/worker-report-call205M.md
(the four changes, what each takes and gives back, and the call the window makes);
`crates/world/src/building/shape.rs`, `crates/world/src/editing/shapes.rs`; and in
`crates/app/src`: `bench.rs`, the item's panel in `panels.rs` (search `Drawn as`),
how a thing set down is drawn from its blocks in `scene.rs`.
SCRATCH is named in the message that starts you.
Notes as you go to SCRATCH/review/W-205W-report.md. Commit nothing, push nothing.

## Before any code, in your report
1. How the window gets an item's blocks today (`forms::blocks(Form, size)`: a fixed
   thirteen) and what it will call so that a job's own shape is drawn from its rows
   (the model's function: name it; do not work a block out in the window).
2. Where "Drawn as" lists its thirteen today, and how it will list the starter
   shapes and the job's own.
3. How a row of the panel is typed as a value today (a size, a name), so that a
   block's figure is typed the same way and refused the same way.

## What is built (call 205, point 6)
1. **An item drawn as a job's own shape is drawn from its rows**, set down in the
   house and on the bench. A job's shape named like a starter's cannot arise (call
   205 point 3); do not handle it.
2. **"Drawn as"** on an item's panel lists the starter shapes and the job's own,
   the job's own first and marked as the job's.
3. **On the bench, "Shape"**, under the item's sizes: the blocks of what the item
   is drawn as, each a row by its material and its form, numbered. A click on a
   row picks the block: it is outlined on the bench and its six figures (across
   from and to, back from and to, up from and to), its form, what it is made of
   and when it is there are rows under it, each read as the table writes it.
4. **A starter shape**: its figures are shown and not typed; "Add block" and "Take
   block away" are not offered; **"Keep as a new shape"** is, with a line saying a
   starter shape is kept as the job's own before it is changed. It asks a name;
   the model's refusal (a name taken) is said as the model says it.
5. **The job's own shape**: a figure is typed like any value; the model's change is
   called and its words are said (what was done, how many items are drawn as it,
   how many set down follow); a refusal is said as the model says it and the block
   is as it was. "Add block" adds one and picks it; "Take block away" takes the
   picked one. Each is one step for Undo.
6. The item on the bench follows each change as it is entered.
7. Nothing is worked out in the window: a block's place is what the model's
   function gives. The saved job is what the command line's `shape` verb would
   have made by the same steps, byte for byte.
8. Every word is a builder's (`tools/gate-language.sh`); every new control is on
   the map by name; every existing control keeps its name.
9. **With it, one point of call 206 that is the window's**: when an assembly is
   changed in the window and a line comes off that carried something a person
   typed, the message names it, in the model's words (`change_assembly_said().took`
   and its `tail()`; `research/worker-report-call206.md`, "Window"). One step of the
   demonstration: the judge's case, 7 typed on a line, named with its $7,000.00.
10. `demos/shapebenchw.sh` (new) to the steps in STATUS.md, its pictures in
   `demos/captured/shapebenchw/`, its row in `plumbline/README.md`.

## Rules
- Files you may touch: `crates/app/src/*.rs`, `demos/shapebenchw.sh`,
  `demos/captured/shapebenchw/`, `demos/window.sh` (only to add a helper, changing
  none), `plumbline/README.md` (a row). Nothing in `crates/world`: if the model
  does not give something the bench needs, say what and stop on that point.
- No number, meaning or file format changes; no existing expectation is edited.
  Tests first where a thing can be tested without a screen, run red, quoted.
- Every window run in the FOREGROUND with the longest timeout, a session at a
  time; before each, `pgrep -x rustc` and wait if busy; a failure is run once
  more before it is believed. Freeze your build (`PLUMBLINE_BIN`); delete your own
  old frozen builds by literal absolute path. Check `df -h /` before each build.
- Look at every picture yourself, as a person opening the program for the first
  time. Say plainly whether a person could find how to change a shape without
  being told, and what is not right.
- Subagents on Sonnet or Opus. If a tool asks a permission you do not have, stop and
  say so in the report; do not ask the lead to do it. No web search with a
  person's or the project's name in it. No timers left running when you hand back.

## Done when
`cargo test` passes for every crate, one at a time (count); `shapebenchw`, `benchw`,
`itemlibw`, `itemneww`, `itemsw` and `cabinetsw` pass on your frozen build, or stand
recorded with what was seen; their output holds no FAIL; the three root gates and
`tools/licenses.sh` pass; the report has the three "before any code" answers, says
what each picture shows and what is not right.
