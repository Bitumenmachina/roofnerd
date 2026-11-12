# The visual standard

What every screen of plumbline is held to, since Patrick's ruling of 2026-11-12
(DECISIONS.md call 62): "an Apple product with windows depth"; "glassy overlays";
"Google Earth feel"; "editing software for construction that feels like playing a
hyper realistic video game". It replaces the standard that came before it (flat, dense,
matte: "the look is an instrument"), which was shown to him and read as no better.

It is written so that a person, or a judge who has never seen the program, can look at
one picture, or one short run of pictures, and say pass or fail on each line. A picture
fails a line if any one place in it breaks that line. "Not in this picture" is a third
answer, not a pass. Where a line cannot be judged from a still picture it says what to
do instead.

Where the figures behind a line come from is in DECISIONS.md call 63; the figures
themselves are named at the top of `crates/app/src/theme.rs`.

## A. Glass over the model: the model window

1. **The model is the window.** The 3D view (or the plan) runs edge to edge behind
   everything. No panel is docked against an edge as an opaque strip; the model is
   seen in the gaps round every panel.
2. **Panels are glass.** Each panel is translucent: what is behind it tints it, and a
   change behind it shows through it. It has round corners, a light edge and a soft
   shadow under it. **The best glass is frosted:** what is behind is blurred, so text
   on the panel is never fought by detail behind it. (As of this writing the panels
   are frosted; the message shown over the model is not, and fails this line.)
3. **Depth.** What floats nearer reads nearer: a message over the model casts a
   deeper shadow than a panel; a control on a panel is a little whiter than the panel;
   a control in hand or selected is washed in the accent and edged with it.
4. **Round, never square.** Buttons, fields, keys, cards, switches, and the small
   action buttons in a row are round-cornered. Nothing on the glass has a square
   corner or a hard hairline box.
5. **Nothing cut, nothing cramped.** No text is clipped, written over other text, or
   ended in an ellipsis where room exists. No control runs under another panel. A
   panel is as tall as what is in it, not a tall sheet half empty.
6. **One way to say how sure.** Known is plain ink; Assumed is amber; Unknown is red;
   a figure's colour and its mark agree. Red is also what is in the same space as
   something else. A panel is not a column of red sentences: what is not yet priced
   or not yet known is said once, in a place, not on every row.
7. **One accent**, a blue, marks what is selected, in hand or switched on.
   Accent-coloured text is never the only thing that says so.
8. **A builder's words.** Every word is one a builder or an estimator says. Units as
   the trades write them. No software term; no folder path; no file name where the
   job's name belongs.
9. **It says what just happened and what to do next**, in a line a person can read
   whole. A refusal says why and what to do instead. Nothing a person does is met
   with nothing.

## B. How it moves (judged from a run of pictures, or by `demos/glassw.sh`)

1. **The wheel closes on what is under the pointer.** The place under the pointer
   stays under the pointer as the view goes nearer or further.
2. **A drag slides the ground with the hand.** The place grabbed stays under the
   pointer.
3. **A turn is about what is under the pointer**, which stays where it is on screen.
4. **It carries on.** Let go while moving and the view goes on and eases to a stop.
5. **A double-click flies to a thing.**
6. **No view jumps.** Plan and 3D ease into each other with no step in size at the
   moment one becomes the other. Nothing flips, and the eye never goes under the
   ground.
7. **The wheel over a panel scrolls the panel and nothing else**, smoothly, and the
   panel shows where in its length it is.
8. **Switches.** Every layer of the job (each level, each system, the roof, the
   finishes, the drawing under the tracing, the clashes) and every panel has a switch
   that reads on or off at a glance.

## C. The place the building stands in

1. **Real light.** One sun that casts soft shadows, light from the sky, and shade
   where surfaces meet. Shadows are never black and edges are never jagged.
2. **A place.** A ground the building stands on and a sky that is pale at the horizon
   and deeper overhead (QUESTIONS.md Q22). No grass, trees, neighbours or scenery:
   nothing is drawn that the estimate does not price.
3. **Materials read as what they are.** Brick as brick, glass as glass that reflects a
   little, wood as wood, metal as a satin. Nothing looks like painted plastic, and
   nothing has a hot white highlight.
4. **Still.** Nothing flickers or swims between two pictures of the same view; no two
   surfaces fight for the same place.
5. **What is selected** is plainly selected and keeps the colour of what it is made
   of; its outline is fine, not a heavy wire. What is in the same space as another
   thing is drawn red while it is selected or carried.

## D. The house, dressed

1. **Dressed in what is priced, bare where nothing is.** A wall, a roof plane, a
   window or a door with a finish priced on it is drawn in that finish; one with none
   is plain. The panel says which line says so, and agrees with the picture.
2. **At size and level.** Courses run level, seams and battens plumb; a brick, a lap
   and a seam are their real sizes against a door.
3. **Corners close.** No gap, notch or sliver where two walls meet.
4. **Openings are built.** A window has a frame, sashes and glass, casing and a sill;
   a door has a frame, a leaf and a knob. All of it sits in the wall.
5. **The roof sits on the house.** A fascia at its edge, a closed soffit where it
   overhangs, a gutter only where one is priced.
6. **A thing placed is drawn as what it is.** A vanity is a vanity and a water closet
   a water closet, not a box of its sizes. (As of this writing a thing whose name
   names one of ten forms is drawn as it: cabinet, wardrobe, WC, washbasin, bathtub,
   table, chair, bed, sofa, desk. Equipment, and anything else, is still a box, and
   fails this line.)
7. **It looks like a house a builder would build**, at the standard of good
   residential work. A picture that reads as a toy or a diagram of boxes fails.

## E. On plan

1. **It reads as a drawing.** The floor is paper. Walls are drawn in ink as their two
   faces and their ends. Openings read as openings. What is overhead is in broken
   lines. There is no sky and no shadow.
2. **Lines by type.** Each kind of roof line has its one colour everywhere it is
   drawn or named; a line with no length is broken and red.
3. **A drawing laid under the tracing** is shown as it is, flat and unlit, and the
   tracing reads against it.
4. **Runs** are in their system's colour at their own size and read apart from walls.

## F. The estimate window

1. **It reads as an estimate.** The recap first, then the cost by division, all
   sixteen in order, then every line under its division's heading, then the suppliers
   side by side.
2. **Every total says how sure it is and what it leaves out.** "none" means no work;
   "no figure" means work with no price yet.
3. **What can be typed is a field on its own line**, and looks like one.
4. **It belongs to the same program.** Round corners, the same type, the same accent,
   the same depth. Dense text sits on a surface thick enough to read on; the estimate
   is not glass, and it is not a grey web page either. (As of this writing its
   tables are round-cornered sheets lifted off the page and its fields are round;
   its heading, its tabs and its foot are still flat strips, and fail this line.)

## How a screen is judged

Sections A, C and D apply to a 3D picture of the model window; A and E to a plan; F
to the estimate window; B to a run of pictures or to the demonstration that measures
it. The judge writes, per picture, each line as pass, fail (with the place in the
picture and what is wrong), or not in this picture. A screen passes when no line fails
on any picture of it. The lines marked "as of this writing" are known to fail: a judge
who finds them passing has found the note out of date, which is also a finding.
