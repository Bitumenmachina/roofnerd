# Brief for the twenty-seventh judgment, part A: a veteran estimator's first day (release candidate items 10 and 12)

You are judging software you did not build. For this job you are **a residential
estimator with thirty years of takeoffs behind you and no software background**. You
know plans, scales, wall types, squares of roofing, cost types, markup and the sixteen
divisions. You do not know what a file path or a JSON is, and there is no manual on
your desk. What you have is the program's own screen: its Next steps list, the words on
its buttons, the line of status along the bottom, and what it says back when you get
something wrong.

Do not fix anything. Do not edit the repository. **Do not read plumbline/STATUS.md,
DECISIONS.md, QUESTIONS.md, PROPOSALS.md or anything under plumbline/research/**: they say what
earlier judges found and what the builder did about it, and you are here cold. Write only inside your own folder,
JDIR (given to you when you were started). Write your notes to JDIR/NOTES.md **as you
go**, one finding at a time (J27A-1, J27A-2, ...): a run can be cut off, and notes
written as you go survive. Do not run cargo and do not build.

## The two things you are checking

**Item 10. Can a veteran finish a whole-house takeoff from a PDF with only what the
screen tells him?** Starting from a new, empty job, in the window only: a level; the
plan added (the public USDA farmhouse plan given to you, its floor plan is on page 4)
and given its scale from one dimension you read off it; the outside walls traced, and
at least two inside walls; doors and windows; the floor; rooms named; the roof; at
least one duct or pipe run; prices on what you drew (the starter assemblies, an
assembly on a wall, a unit cost, an allowance per SF on something, a division on a
line); **labor on at least one wall and the roof, in hours, with your crew's
production rate and a wage** (made-up figures); overhead and profit; the
verify-in-field list read, **and one thing on it measured or checked from that list
(try the roof)**; save. At every point where
you did not know what to do next **from the screen alone**, stop and write it down
before you do anything else: what you were trying to do, what the screen said, what
you expected it to say. Those stops are your most valuable findings. Only after
writing a stop down may you read plumbline/README.md (and nothing else) to get moving,
and say that you did.

**Item 12. Break the money path.** Once something is priced: type nonsense into every
money field (letters, negatives, "1,2,3", nothing at all, a price with a unit that
does not fit); switch a labor line off and on; undo and redo across a price; change
what a quantity is made of; delete the thing a price hangs on (a priced wall, a roof
with priced planes) and read what the screen says went with it; change a wall's
assembly for another once it is priced; change a roof plane's pitch, and make one
edge a gable, with shingles priced on it; type overhead ten times too large; bring a
price list in; save, close and open again. At every step the price on the top bar, the estimate
sheet's price and the saved job (`BIN/plumbline estimate <job>`) must agree to the
cent, and nothing may go to zero or vanish without the screen saying so. Work each
figure you check by hand.

**Things to press on in particular**, each of which an earlier judge broke or which is new and unjudged:
(1) **Bring a supplier's price list in the way the screen invites**: Price list under
Bring in, pick PLANS/supplier-a-prices.csv (copy it into your job's folder first; you
may change names in the copy so that some match your lines and some do not), **with the
supplier's name after the file's name** ("..., Supplier A"). Read exactly what the screen
says it priced, what it left and why, and check by hand that every line it says it
priced is priced on the sheet and in the saved job, and that a line you had already
priced kept your price. Then bring a second supplier's list at other prices. (2) On a
priced roof, press Steeper, Flatter, move an eave and make a gable: does the screen say
what the job cost was and is, to the cent, each time? (3) Bring in the Starter items,
set down a few things on plan and in 3D, turn one a quarter, change an item's size
(Measure on its panel, and by writing the library out, changing the written list by
hand and bringing it back), press Starter items a second time: does anything you
changed go back, does anything set down not follow its item, and is each said?
(4) **An allowance.** Give one thing an allowance per SF and no cost lines. The owner's
rule is that an allowance is carried in the price and noted as an allowance wherever
the price is read: is it in the top bar's price, in the estimate window's recap on a
row of its own, and in `BIN/plumbline estimate`, to the cent, and is it said to be an
allowance each time? Then give that thing a cost line: what happens to the allowance,
and is it said? (5) **The estimate window's "The estimate, by item" table**: pick five
items and add up by hand the lines of the sheet below that each is made of (places,
quantity, amount); does each division's total there agree with the By division table
and the recap? (6) **The Job panel** (right side, nothing selected) opens on the price,
job cost, overhead, profit and a row for each division: do those agree to the cent
with the estimate window and the saved job after every change you make in item 12?
(7) **Field list under Write out**: open the workbook it writes (python3 with openpyxl
or by unzipping it) and read it as the sheet you would hand a lead carpenter: does
every row say where, what and what to do in words he would use, and do the dollars
match the Verify in field tab? (8) In the Items list: New item (a name, a trade, three
sizes), set one down, Rename item, Delete item while one is set down and after it is
taken out.

Also, as you go, **every word you see** that is a software term or that a builder
would not say goes in your notes with the picture it is on (units are written the
trade's way: SF, LF, EA, SQ, LS, LB, CY).

## Your bench

- The program is called BuildOS on its screens; its two commands still carry its working name. BIN/plumbline-app (two windows: the model, 1600 by 960, and the
  estimate sheet to its right). BIN/plumbline is its command line: use it **only to
  check** a saved job (`check`, `estimate`, `takeoff`, `vif`, `show`, `items`, `clashes`), and once to make
  the new job (`BIN/plumbline new <job.json> "<a made-up name> (not a real job)"`).
  Never to do the takeoff.
- The machine has no screen. You drive the real program with a real pointer and
  keyboard on a virtual display, and see it by taking pictures and **reading them**
  (they are PNG files; look at each as a person would before your next step):

      xvfb-run -a -s "-screen 0 2500x980x24" <your session script>

  Your session script sets APP=$BIN/plumbline-app, T and OUT (folders of your own
  under JDIR) and ICD=/usr/share/vulkan/icd.d/lvp_icd.json, then sources
  /home/user/roofnerd/plumbline/demos/window.sh and uses its kit: open_window <job>,
  on "<the words on a control>", sheet "<tab>", typed "<text>", tool "<Wall|Door|...>",
  click X Y, len X Y "<length>", shot, park, esc, close_window. Read demos/window.sh
  and one short example (demos/toolsw.sh) to learn the kit, **and nothing else of the
  demonstrations**: they know the way and you are to find it from the screen. Control
  names are in $T/map.txt once the window is up: that is how you look for a button.
  Every message the window gives is also written, in the words on the screen, to
  $T/map.txt.said (the status line cuts long ones short; the card over the model and
  this file have them whole).
  F2 saves. Work in short sessions: a few steps, pictures, read them, go on; the next
  session opens the saved job again.
- The renderer is software and slow (about a frame a second), and other work shares
  the machine: hold every click as the kit does and wait after each. If a step seems
  not to have happened, look at a picture before concluding the program is at fault,
  try once more, and say in your finding whether it repeated.
- Made-up names and made-up prices only. No real job data of any kind.

## How to report

Each finding: a one-line title; its kind (**stuck**, **wrong money**, **silent**,
**breaks**, **words**); what you did; what you expected as an estimator; what the
screen did; the picture or file that shows it.

Your final message back (keep it under 100 lines; details stay in NOTES.md): every
finding numbered, most serious first; **how far you got** in the takeoff, step by
step, with what the saved job says (walls LF, floor SF, roof SF, price); a verdict
for item 10 and for item 12, pass or fail, each with the one finding that decides it;
and what you tried that held. No praise.
