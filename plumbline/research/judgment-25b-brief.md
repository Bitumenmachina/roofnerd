# Brief for the twenty-fifth judgment, part B: the look and the words (release candidate items 9 and 11)

You are judging software you did not build. Do not fix anything. Do not edit the
repository. **Do not read plumbline/STATUS.md, DECISIONS.md, QUESTIONS.md or anything
under plumbline/research/**: they say what earlier judges found and what the builder
did about it, and you are here cold. (VISUAL.md's last section, "How it was last
judged", names lines an earlier judge failed: judge each line yourself from what you
see, and do not take its word either way.) Write only inside your own folder, JDIR (given to you when you were
started). Write your notes to JDIR/NOTES.md **as you go**, one finding at a time
(J25B-1, J25B-2, ...): a run can be cut off, and notes written as you go survive. Do
not run cargo and do not build.

plumbline is a 3D estimating program for a whole house, for builders and estimators
(repository /home/user/roofnerd, folder plumbline/). Read /home/user/roofnerd/CLAUDE.md
first: "How it should feel" and the fixed rules "Speak construction", "The window is
glass over the model, with depth", "It looks real", "The 3D house looks like the
house". Then plumbline/VISUAL.md, the standard you judge by: every lettered line of it.

## The two things you are checking

**Item 11. The look.** Judge every screen you can reach against plumbline/VISUAL.md,
line by line (A1, A2, ... F-last): pass or fail, with the picture that shows it. The
owner's own test is his eyes: "editing software for construction that feels like
playing a hyper realistic video game", "more like an Apple product with windows
depth", "think Google Earth feel". Where a screen passes the written line and still
would not make a person want to open it, say so and say why. Report anything clipped,
overlapping, cut off, unreadable, misaligned, washed out, or that looks like a toy.

**Item 9. Every word on every screen.** List every word and phrase on both windows as
you go, by screen. Mark any that is a software term (file, path, element, object,
entity, node, null, error, parse, invalid, default, config, toggle, and the like) or
that a US builder or estimator would not say or would trip on, any British spelling,
and any unit not written the trade's way (SF, LF, EA, SQ, LS, LB, CY). The list
itself is a deliverable: save it as JDIR/WORDS.md.

## The screens to reach

Make the made-up whole house with `BIN/plumbline sixteen JDIR/house.json` and open it.
Take and read pictures of at least: as it opens (3D); each view (Plan, Walk, Frame);
each switch under Show, one at a time (Ground, Finishes, Framing, Roof, each trade,
See through, Known Assumed Unknown, Clashes); a level on show alone, in 3D and on
plan; a wall, a window, a roof plane, a duct, a fixture and a footing each selected
(the panel on the right); every tab of the estimate sheet, scrolled to its end; a
tool in hand with its fields open (Wall, Door, Roof, Duct); a refusal (a click with
sizes open); the Bring in and Write out lists; the Next steps list hidden and shown.
On that house also bring in the Starter items, read the Items list to its end, set down
a lavatory, a water heater, an outdoor unit, a sofa and an air handler, look at each in
3D and on plan, and open one with its Change button: read every word of that panel.
Then a new empty job (`BIN/plumbline new JDIR/empty.json "Empty (not a real job)"`):
what a person sees first, Add level, the first wall. Bring in the Starter assemblies,
give a wall one, and read the estimate sheet's Estimate tab to its end (its labor
lines, a production rate typed on one). Delete a priced wall and read what is said. Then a job with a drawing under
it: run `demos/planw.sh` is NOT allowed (do not run the demonstrations); instead add
the public plan given to you (PLAN, its floor plan is on page 4) with Add drawing,
**picking it and its page from the list on the screen as a person would, not by
typing its name**, and look at it on plan and in 3D. Read every word of that list.

## Your bench

- The program: BIN/plumbline-app (two windows: the model, 1600 by 960, and the
  estimate sheet to its right); BIN/plumbline is its command line (for making the two
  jobs above only).
- The machine has no screen. You drive the real program with a real pointer and
  keyboard on a virtual display, and see it by taking pictures and **reading them**
  (PNG files; look at each as a person would):

      xvfb-run -a -s "-screen 0 2500x980x24" <your session script>

  Your session script sets APP=$BIN/plumbline-app, T and OUT (folders of your own
  under JDIR) and ICD=/usr/share/vulkan/icd.d/lvp_icd.json, then sources
  /home/user/roofnerd/plumbline/demos/window.sh and uses its kit: open_window <job>,
  on "<the words on a control>", sheet "<tab>", typed "<text>", tool "<Wall|Door|...>",
  click X Y, shot, park, esc, close_window; keys with `xdotool key t` (Plan), `v`
  (Walk), `f` (Frame), `x` (See through), F12 is what `shot` presses. Read
  demos/window.sh and one short example (demos/tradesw.sh). Control names are in
  $T/map.txt once the window is up. Each start of the program numbers its pictures
  from 01 again: give each session its own OUT folder. To see small type, crop and
  enlarge a picture (python3 with PIL is on the machine).
- The renderer is software and slow (about a frame a second), and other work shares
  the machine: hold every click as the kit does and wait after each; if a step seems
  not to have happened, look at a picture and try once more before calling it a fault.
  A software renderer has no fine anti-aliasing: do not fail a line for that alone.

## How to report

Your final message back (under 100 lines; details stay in the files): for item 11, a
table of every VISUAL.md line, pass or fail, with the picture; then the ten things
that would most change the owner's mind about the look, most serious first. For item
9, how many words you listed, and every one you marked, with the screen it is on. A
verdict for each item: pass or fail, with the finding that decides it. No praise.
