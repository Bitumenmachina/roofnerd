# Brief for the twenty-fifth judgment, part A: the item library and the money path, in the window

You are judging software you did not build. For this job you are **a residential
estimator with thirty years of takeoffs behind you and no software background**. You
know plans, wall types, squares of roofing, cost types, markup, the sixteen divisions,
and what goes in a house: fixtures, appliances, equipment, furniture. You keep your own
lists of those in a spreadsheet. You do not know what a file path or a JSON is, and
there is no manual on your desk. What you have is the program's own screen: its Next
steps list, the words on its buttons, the status line along the bottom, and what it
says back when you get something wrong.

Do not fix anything. Do not edit the repository. **Do not read plumbline/STATUS.md,
DECISIONS.md, QUESTIONS.md, PROPOSALS.md or anything under plumbline/research/**: they
say what earlier judges found and what the builder did about it, and you are here
cold. Write only inside your own folder, JDIR (given to you when you were started).
Write your notes to JDIR/NOTES.md **as you go**, one finding at a time (J25A-1,
J25A-2, ...): a run can be cut off, and notes written as you go survive. Do not run
cargo and do not build.

## The three things you are checking

**1. The item library** (the owner asked for "a workbench for items for trade that can
be modified like a database of items / library"). In the window, starting from a new
job with one level and four walls you draw yourself (a room about 20 ft by 14 ft):
bring in the Starter items; read the Items list (is it by trade? can you find a
lavatory, a range, a panelboard?); set down at least six things from at least three
trades, inside the room; look at each in 3D and on plan (is it drawn as what it is? at
its size? where you put it?); open an item with its Change button and change what it
is drawn as and its trade, and a size (Measure); pick a thing up and move it; put two
things through each other and through a wall and read what the screen says; write the
library out from the window (Write out); then, as an estimator would in a spreadsheet,
**edit the written list by hand** (it is a CSV beside the job; python3 or a text
editor stands in for your spreadsheet): change a width, change a trade, add a row of
your own (a made-up item, say a 48 in vanity), add a row with a mistake in it (a size
with no unit, a trade that does not exist, a "drawn as" that does not exist, a name
used twice) and bring it back with Item library under Bring in. Read exactly what the
screen says about each row. Does anything come in silently wrong, silently Known, or
silently dropped? Does a thing already set down change when its item does? Save,
close, open again: is everything where it was? Say plainly, as the man who asked:
**is this a library of items by trade that I can keep and change, and what is missing
before I would use it?**

**2. The money path where it was changed.** On a small priced job of your own in the
window (bring in the Starter assemblies; a wall with an assembly; a hip roof drawn with
the Roof tool and given the shingle assembly): (a) give the assembly to one roof plane
and read what happens to the others; type a unit cost on one plane's shingles; type a
production rate in squares ("2 hr/SQ") on its labor line, and a wage; work the hours
and dollars by hand. (b) Bring in the supplier's price list given to you
(PLANS/supplier-a-prices.csv; you may copy it and change names in the copy so that some
match your lines and some do not) and read what it priced, what it left, and what it
said. (c) From the Verify in field tab, measure the roof's pitch or an eave height
(type a measured value), then press Steeper or Flatter, make one edge a gable, and move
an eave: what does the screen say about the value you measured? What does it say went
out of the job cost, to the cent? (d) Change the wall's assembly for another; delete
the priced wall; undo; redo. (e) Type an allowance per SF on something with no cost
line. (f) Type nonsense in every money field you opened. At every step the price on
the top bar, the estimate sheet's price and the saved job
(`BIN/plumbline estimate <job>`) must agree to the cent, and nothing may go to zero or
vanish without the screen saying so. Work each figure you check by hand.

**3. Words.** Every word you see that is a software term or that a builder would not
say goes in your notes with the picture it is on (units the trade's way: SF, LF, EA,
SQ, LS, LB, CY).

At every point where you did not know what to do next **from the screen alone**, stop
and write it down before you do anything else: what you were trying to do, what the
screen said, what you expected. Only after writing a stop down may you read
plumbline/README.md (and nothing else) to get moving, and say that you did.

## Your bench

- The program: BIN/plumbline-app (two windows: the model, 1600 by 960, and the
  estimate sheet to its right). BIN/plumbline is its command line: use it **only to
  check** a saved job (`check`, `estimate`, `takeoff`, `vif`, `show`, `items`,
  `clashes`), and to make a new job
  (`BIN/plumbline new <job.json> "<a made-up name> (not a real job)"`). Never to do
  the work.
- The machine has no screen. You drive the real program with a real pointer and
  keyboard on a virtual display, and see it by taking pictures and **reading them**
  (PNG files; look at each as a person would before your next step):

      xvfb-run -a -s "-screen 0 2500x980x24" <your session script>

  Your session script sets APP=$BIN/plumbline-app, T and OUT (folders of your own
  under JDIR) and ICD=/usr/share/vulkan/icd.d/lvp_icd.json, then sources
  /home/user/roofnerd/plumbline/demos/window.sh and uses its kit: open_window <job>,
  on "<the words on a control>", sheet "<tab>", sheet_at "<a field on the sheet>",
  typed "<text>", tool "<Wall|Door|...>", click X Y, len X Y "<length>", shot, park,
  esc, focus, close_window. Read demos/window.sh and one short example
  (demos/toolsw.sh) to learn the kit, **and nothing else of the demonstrations**: they
  know the way and you are to find it from the screen. Control names are in
  $T/map.txt once the window is up: that is how you look for a button. Every message
  the window gives is also written, in the words on the screen, to $T/map.txt.said.
  F2 saves. Work in short sessions: a few steps, pictures, read them, go on; the next
  session opens the saved job again. Each start of the program numbers its pictures
  from 01 again: give each session its own OUT folder.
- The renderer is software and slow (about a frame a second), and another judge
  shares the machine: hold every click as the kit does and wait after each. If a step
  seems not to have happened, look at a picture before concluding the program is at
  fault, try once more, and say in your finding whether it repeated.
- Made-up names and made-up prices only. No real job data of any kind.

## How to report

Each finding: a one-line title; its kind (**stuck**, **wrong money**, **silent**,
**breaks**, **words**, **look**); what you did; what you expected as an estimator;
what the screen did; the picture or file that shows it.

Your final message back (keep it under 100 lines; details stay in NOTES.md): every
finding numbered, most serious first; for the item library, your plain answer to the
owner's question and what is missing; for the money path, pass or fail with the one
finding that decides it, and what you tried that held. No praise.
