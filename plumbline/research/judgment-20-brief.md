# Brief for the judge of the four new trades and the typed-length fix

You are judging work you did not build. Your job is to try to break it and to say plainly
what a builder would see. Do not fix anything. Do not edit the repository. Write only
inside your own folder, JDIR:

    /tmp/claude-0/-home-user-roofnerd/ae04f28e-dd5a-5c73-a560-15cd8a86f760/scratchpad/j20

Write your notes to JDIR/NOTES.md **as you go**, one finding at a time (J20-1, J20-2, ...),
each with: what you did, what you expected, what happened, and the file or picture that
shows it. A run can be cut off; notes written as you go survive.

## What this is

plumbline is a 3D estimating program for a whole house (repository
/home/user/roofnerd, folder plumbline/). Read /home/user/roofnerd/CLAUDE.md first
(the fixed rules: flag never fabricate, speak construction, every value Known, Assumed
or Unknown with where it came from). Then the END of plumbline/STATUS.md, from "Patrick's
request for a snapshot of each trade" on, and plumbline/DECISIONS.md calls 69 to 74.

The owner, Patrick, asked to SEE each of seven trades: foundation, framing, electrical,
plumbing, HVAC, interior woodwork, waterproofing. Four were lines on the estimate with
nothing drawn. In one night the builder drew a first layer of each. Nothing of it has
been judged. That is your job.

## The program to judge (do NOT run cargo; do NOT build)

A built copy, frozen for you:

    BIN=/tmp/claude-0/-home-user-roofnerd/ae04f28e-dd5a-5c73-a560-15cd8a86f760/scratchpad/bin6
    $BIN/plumbline            # the command line; run it alone to list every verb
    $BIN/plumbline-app <job>  # the two windows

The window runs on a virtual display and is seen by taking pictures and READING them:

    xvfb-run -a -s "-screen 0 2500x980x24" <your session script>

Your session script sources /home/user/roofnerd/plumbline/demos/window.sh after setting
APP=$BIN/plumbline-app, T and OUT (your own folders under JDIR) and
ICD=/usr/share/vulkan/icd.d/lvp_icd.json. The kit gives you: open_window <job>,
on "<name of a control>" (clicks it; names are in $T/map.txt once the window is up),
sheet "<tab>", typed "<text>", tool "<Wall|Door|...>" ["sizes"], click X Y, len X Y "<length>",
shot (saves NN-model.png and NN-sheet.png to OUT), park, esc, close_window. The renderer
is slow: about a frame a second. Hold every click and wait after it, as the kit does.
plumbline/demos/toolsw.sh and carryw.sh are short examples to copy from. IfcOpenShell,
for checking a BIM file, is at PYTHONPATH=/tmp/claude-0/-home-user-roofnerd/ae04f28e-dd5a-5c73-a560-15cd8a86f760/scratchpad/pylib
with /home/user/roofnerd/plumbline/tools/ifc-check.py <file.ifc>.

Use made-up jobs only. No real job data of any kind.

## What to try, and to try to break

Work every figure by hand first, then compare. A figure you did not work by hand is not
checked.

1. **Foundation.** `plumbline type <job> footing ...`, `plumbline footing <job> <level> <type>
   --points ... --depth ...`, foundation walls as walls on a level under the first floor
   (demos/found.sh shows the path). Footing length, width, depth, volume in CY; the
   level's takeoff; pricing a footing by the CY; the BIM file. Try: an L-shaped or
   T-shaped run of footings (is the concrete right where three meet, or where one ends
   on the middle of another?), a footing in inches, a zero or negative depth, a type
   with no width, a footing measured deeper afterwards (`plumbline measure`), a footing
   moved, removed, its type removed, a pipe run through a footing (what does `clashes`
   say, and is it true?), a footing on a level with a floor. In the window: is it
   drawn, does the Ground switch show it, do the panel and the Building tab say what
   the command line says? Is a footing ever called a wall, or counted as one, anywhere?
2. **Framing.** `plumbline sixteen <job>` makes a made-up whole house whose walls have
   framing lines. In the window, the Framing switch. Take a wall, work its frame by hand
   (plates, studs on 16 in centres from its first end, kings, jacks, header, sill,
   cripples; plumbline/crates/world/src/building/framing.rs says the rule) and compare
   with the picture as closely as a picture allows. Then make small jobs of your own to
   break it: an opening close to a wall's end, two openings close together, an opening
   as tall as the wall, a wall 2 ft long, a framing line worded oddly ("2 x 6 @ 24",
   "2X4 studs 16 OC", "wall framing 2x6 16 in o.c."), a wall with no framing line (it
   must NOT be framed), a wall with two framing lines. Is anything framed that no line
   prices? Does the switch say how many walls it frames, and is that right?
3. **Waterproofing.** The Finishes switch. With it off: which walls show a weather
   barrier and which lines made them? Does a wall with no barrier line show one (it
   must not)? Flashing at openings: only where priced? Try your own lines: "Tyvek",
   "felt paper", "ice and water shield", "vapor barrier" (that one is NOT a weather
   barrier: is it taken for one?). Does switching Finishes off change any number
   anywhere (it must not)?
4. **Interior woodwork.** The base in rooms with base priced. Look inside the house
   (pick a level alone under Levels, switch Mechanical, Electrical and Plumbing off,
   tilt with the Up arrow key). Is the base on every wall of a priced room, cut at
   doors, absent in an unpriced room? An L-shaped room? Is it drawn outside the house
   anywhere, or through a door, or floating?
5. **The typed-length defect just fixed.** In the window, from an empty job: Add level,
   pick Wall, ACCEPT the sizes it shows with Enter (do not type over them), click a
   corner, type a length, Enter, and so round four sides. Read the saved job back with
   the command line: is every length what was typed? Do the same for Door and Window
   (accept, place; then type over one size, place). Try to make the program place a
   size that is not the one on screen, by any order of Tab, Enter, Esc and clicks a
   person might plausibly use.
6. **The whole house's price.** `plumbline sixteen`, then `plumbline estimate`. Work
   Division 3 (Concrete) by hand from what is drawn and what each line is priced at,
   and the job cost and the price from the divisions and the markup. Any cent of
   difference is a finding.
7. **Anything silently wrong**: a number on screen that the saved job does not hold, a
   thing drawn that is not in the job, a thing in the job that is not drawn and not
   said, a switch that changes a figure, a word a builder would not use.

## How to report

End with a report (also saved as JDIR/REPORT.md):

- every finding, numbered, most serious first: what a person would have met;
- for each of the four new trades: **would Patrick see it?** yes, partly or no, and why,
  in a sentence;
- what you tried that held;
- the ten things that would most change his mind.

Be exact. If you could not check something, say so and why. Do not round a failure into
a pass.
