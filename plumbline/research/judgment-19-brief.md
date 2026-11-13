# Brief for the judge of the rebuilt window (commit 14fc994)

You are judging software you did not build. Try to break it and write down exactly what
you broke and what it was like to use. You are not asked to fix anything or to be kind.

## What it is, and what its owner asked for

plumbline is a 3D estimating program for a whole house: trace the drawings, build the
house, price it by division, and find the clashes. Its owner, Patrick, looked at its
window and said (his words): "I still do not see how the UI functions are better please
ensure you are deep diving into scroll mechanics, toggle on and off, glassy overlays.
Think google earth feel, need to be able to export to BIM files and create items by
spec sheet uploads IE mechanicals, furniture etc whole point of clash is also moving a
bathroom vanity around ... Think of it more like an Apple product with windows depth.
Editing software for construction that feels like playing a hyper realistic video game".

The builder has since changed the window. **Your job is to say, as Patrick would, whether
he would now see it**, and to find what is broken. Judge these, each on its own:

1. **Moving round the model ("Google Earth feel").** The wheel should close on what is
   under the pointer; a left-drag on the ground should slide the ground with the hand;
   a right-drag should turn the view about what is under the pointer; a double-click
   should fly to a thing; letting go while moving should carry on and ease out; Plan
   and 3D should ease into each other. Try each, from several places, zoomed in and
   out, on the ground, on a roof, on the sky, near the horizon, with a panel in the
   way. Does anything jump, flip, run away, go under the ground, get lost with no way
   back (try Frame)?
2. **The look ("glassy overlays", "Apple product with depth", "hyper realistic").**
   Read every picture as a person. Do the panels read as glass floating over the
   model, with depth? Is anything clipped, overlapping, cramped, or dated? Does the
   house look real? Say plainly what still looks like an old program, and where.
3. **Switches ("toggle on and off") and scrolling.** Every row under Show. Is it clear
   what is on? Does the wheel over a panel scroll the panel and not the model? Is
   there anything a person would want to switch that has no switch (levels, systems,
   panels themselves)?
4. **A maker's sheet made into an item** ("spec sheet uploads"). In JDIR/sheets are
   four made-up sheets: vanity.pdf, air-handler.pdf, bookcase.pdf (no height on it)
   and mystery.pdf (its name does not say what it is). Bring each in with Bring in >
   Maker's sheet (type the file's name as it lies from the job's folder, e.g.
   ../sheets/vanity.pdf; a kind can follow a comma: fixture, equipment, furnishing).
   Is what was read right? Is what was not on the sheet said, or made up? Set items
   down from the Items list. What happens with the bookcase, which has no height?
5. **Picking a thing up and moving it** ("moving a bathroom vanity around"). Place two
   things so that they overlap (set one item down on another, or use the Fixture tool
   for a water closet). Select one, press on it and carry it. Is it red while it is
   in the other and its own colour when clear? Does the line of status say by how
   much? Does Undo put it back in one step? Carry it far, off the slab, into a wall,
   under a panel, while zoomed far out. Does the saved job agree with the screen
   (`JDIR/bin/plumbline clashes <job>` and `show <job>`)?
6. **Writing out** ("export to BIM files"). Each button under Write out. Are the six
   files beside the job? Open the .ifc as text: is the furniture in it? Does writing
   out twice, or with nothing in the job, behave?
7. **Typed sizes.** Door, Window, Wall: pick a tool, type sizes, place; pick it again
   and change one size; press Tab and Enter in every order you can think of; click the
   plan while the size fields are open. Does it ever place a size that was not on
   screen, or throw away what you typed, without saying so?
8. **Anything that goes silently wrong**: a number on screen that the saved job does
   not agree with, a click that does nothing and says nothing, a thing that vanishes.

## Your bench

* Your folder ("JDIR") is named in the message that gave you this brief. Work only
  inside it. **Never write anywhere under /home/user/roofnerd.** No git, no network, no
  cargo.
* `JDIR/bin/plumbline-app` is the program (the model window, 1600 by 960, and a sheet
  window to its right). `JDIR/bin/plumbline` is its command line: use it to make a
  starting job quickly (run it with no arguments for its verbs; `plumbline sixteen
  <file>` writes a made-up whole house priced in sixteen divisions; `plumbline house
  <file>` the same house unpriced) and to check a saved job (`check`, `show`,
  `clashes`, `estimate`).
* The machine has no screen. You drive the real program with a real pointer and
  keyboard on a virtual display through one script:

      bash JDIR/trial.sh JDIR <out folder> <job file> <steps file>

  **Read the comment at the top of `JDIR/trial.sh`**: it lists what a steps file can
  use (`on "Wall"` clicks the control with that name, `typed "..."`, `click X Y`,
  `shot`, `names`, `xdotool ...`). `names` writes the name of everything that can be
  clicked with where it is. `shot` saves a picture of both windows as PNG files:
  **read every picture you take** before deciding the next step.
* A drag: `xdotool mousemove X Y; sleep 1.5; xdotool mousedown 1; sleep 1.5;` then
  several `xdotool mousemove X2 Y2; sleep 0.6` steps, then `xdotool mouseup 1; sleep 4`
  (button 3 for the right button). A flick is the same with quick small steps (sleep
  0.04) and no pause before mouseup. A wheel notch: `xdotool click 4` (nearer) or `5`;
  on this display one notch is two lines of wheel (a tenth nearer per line).
* The renderer is software and slow (a frame can take half a second). A click or key
  that begins and ends inside one frame can be lost; the helpers hold each one. If a
  step seems not to have happened, look at the picture, try once more, and say in your
  finding whether it repeated. Keep sessions short (a few steps, pictures, read them).
  One session at a time. Each takes a few minutes.
* The job file is saved at the end of a session (F2) and the next session opens it.

## How to report

Keep notes as you go in `JDIR/NOTES.md` using shell redirection (`cat >> ... <<'X'`),
one finding at a time, so nothing is lost if you are cut off. Number findings J19-1,
J19-2, ... Each has: a one-line title; a kind (**wrong** a number or the saved job
disagrees with the screen; **silent** something dropped or ignored without a word;
**breaks** a crash, a freeze, a view that cannot be got back; **feel** it does not move
or respond as asked; **look** against "glass with depth" and "looks real"; **words** a
software term or wording a builder would trip on; **missing** something Patrick asked
for that is not there); what you did (the steps file); what you expected; what
happened (name the picture).

Your final message back must contain the whole report (do not rely on the file):
every finding; then **"Would Patrick see it now?"**: one paragraph each for the eight
headings above, as he would say it, with a plain yes, partly or no; then **"The ten
things that would most change his mind"**, most important first; then **"What I tried
that held"**. No praise beyond that.
