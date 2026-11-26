# Design review of BuildOS: the prompt for an outside designer

Patrick pastes the block below into a design session and attaches the five pictures
named in it. What comes back is put in `research/` and ruled on in a call
(DECISIONS.md call 239). The lead keeps this file true to the window as it is: when
the layout changes, the inventory below is changed with it.

---

You are reviewing the interface of a desktop program and proposing a cleaner design
for it. I need a critique and a buildable layout specification, not a brand exercise.

## What the program is

BuildOS is a 3D estimating program for residential construction. A builder traces a
set of plans, the program builds the 3D house from the tracing, prices it by trade
(16 divisions), and shows where the trades clash. The users are estimators (who want
depth) and field people and first-time users (who must not be overwhelmed). It is free
and open source.

It has two windows that are open at the same time, usually on one wide screen or two
screens:

- the **model window** (the house in 3D or in plan, with the drawing tools);
- the **estimate window** (the price, by division; it can also show one or two plans).

## The owner's brief for the look, in his words

- "editing software for construction that feels like playing a hyper realistic video game"
- "more like an Apple product with windows depth"; "glassy overlays"
- "think Google Earth feel": the model fills the window and the interface floats over it
- "it has the depth of a full high end home custom build but without overwhelming
  field guys and new users, clean code clean screens"
- And his review of the screens attached: **"it still seems quite busy and the tools
  look a bit cramped"**. He is right. The builder of this program favours function and
  has let the look go. The goal is both.

## The attached pictures (all at 1x; the house and every figure are made up)

1. `1-model-window-3d.png` (1600 x 960): the model window, nothing selected.
2. `2-model-window-plan-with-a-tool.png` (1600 x 960): plan view, a tool in hand, a
   message shown.
3. `3-model-window-thing-selected.png` (1600 x 960): a wall selected; its values on the
   right.
4. `4-estimate-window.png` (880 x 960): the estimate window.
5. `5-second-window-two-plans.png` (880 x 960): the estimate window showing two plans.

## What is on the model window today (the full inventory)

**Top left bar:** the name BuildOS, the job's title, Save (Ctrl+S), Undo (Ctrl+Z), Redo
(Ctrl+Y).

**Top right bar:** PRICE and the figure; "92 to verify, $17,060.62 riding"; Saved;
three counts: known, assumed, unknown.

**View bar** (floating at the top right of the model): 3D, Plan (T), Walk (V), Frame
(F), Tilt as I dive, Sun. A north arrow card floats at the top left of the model.

**Job panel** (left, about 250 px wide, full height, scrolls; "more below" at its foot):
- NEXT STEPS: a ten-step guide for a first job, with a Show/Hide link.
- LEVELS: All levels; one row per level with its height and a switch; Add level.
- SHOW: eight buttons (All, Foundation, Framing, Outside, Plumbing, HVAC, Electrical,
  Inside) and below them twelve switches, each with a colour dot and a count (Ground,
  Finishes, Framing, Architecture, Roof, Mechanical, Electrical, Plumbing, See through,
  Sizes, Known/Assumed/Unknown, Clashes).
- DRAWINGS: Add drawing, Set scale, Line up, and a row per drawing.
- BRING IN: nine buttons (Starter assemblies, Starter tool chest, Price list,
  Assemblies, Prior estimate, Designer's model, Maker's sheet, Tool chest file, Scan
  file). Ten more kinds of file are about to be added here and under Write out.
- TOOL CHEST: New item, and the list of items.
- WRITE OUT: six buttons (BIM model (IFC), Estimate workbook, Estimate (CSV), For
  suppliers, For subcontractors, For the PM).

**Details panel** (right, about 345 px wide, scrolls):
- With nothing selected: the job's title, the price in large figures, job cost,
  overhead and profit, all sixteen divisions with their dollars, the markup, then the
  quantities of the level in view. (The estimate window shows most of this too.)
- With a thing selected: its name; its values, each with a mark for Known, Assumed or
  Unknown, a line saying where the value came from, and links (Check, Measure); the
  assembly it is priced by; its cost lines.

**Tools bar** (bottom, about 965 px wide): a line of help text, then eighteen tools in
one row under three small headings. BUILDING: Wall (1), Door (2), Window (3), Floor (4),
Room (5), Room line, Cabinets, Roof (6). MECHANICAL, ELECTRICAL, PLUMBING: Duct (7),
Pipe (8), Conduit (9), Cable (0), Fitting, Equipment, Fixture, Connect. EDIT: Move,
Delete. Each tool is a button about 48 px wide with a drawn glyph and a label at about
10 px. With a tool in hand, a second row of fields (thickness, height, length)
appears in the same bar.

**Status line** (bottom left): the last message. **Bottom right:** three switches that
put away the Job panel, the Details panel and the Tools bar.

In picture 1 the three panels and the bars cover a little over half of the window; the
house has the rest.

## What is on the estimate window today

A row of tabs that wraps to two lines at this width (Estimate, Budget, Verify in field,
Clashes, Layouts, Building, Roof, Mechanical, Electrical, Plumbing, each with a count);
a card "The recap"; a card "By division"; then the lines of the sheet with editable
quantities, unit costs and a formula on each line. At its top, Plan and Two plans turn
the window into one or two plans, each with its own Layers menu.

## What I think is wrong (confirm, correct or add to this)

1. Everything is on screen at once. A first-time user sees about ninety controls.
2. The Job panel is one long stack of six unlike things (a guide, levels, layers, files
   in, a library, files out).
3. Show has two systems for one idea: eight buttons and twelve switches.
4. The tools are eighteen equal buttons with labels too small to read; "Room line",
   "Cabinets" and "Equipment" touch their neighbours.
5. The Details panel repeats the estimate window when nothing is selected.
6. Almost every control has its own outline, so nothing leads the eye. There are seven
   or more accent colours in view (trade colours, blue switches, amber, red, green).
7. The smallest type is about 10 to 11 px.
8. Floating panels with gaps round them cost model space (Figma's UI3 found the same
   and went back to fixed panels for people who work in the tool all day).

## What must stay true (the constraints)

- Two windows. The model fills its window edge to edge; the interface sits over it.
- Every layer of the job has a switch that reads on or off at a glance.
- Every panel can be put away.
- Depth is on demand: an estimator can reach every value, its source, its formula.
- Every word on screen is a builder's or estimator's word. No software terms. Units
  are written the way trades write them (SF, LF, EA).
- Every value shows whether it is Known, Assumed or Unknown. Today: Known is plain
  ink, Assumed is amber, Unknown is red.
- A file is picked from a list in the window; there is no operating-system file dialog.
- It must work from 1366 x 768 to 2560 x 1440, with four sizes of lettering.
- It is built with a game engine's own interface kit (Bevy UI: flexbox layout, rounded
  rectangles, borders, shadows, translucent and frosted panels, text in Inter, simple
  drawn glyphs). There is no web view. Plain, cheap effects only: the renderer may be
  software. An icon set can be added if its licence is permissive (MIT, Apache, OFL).
- Keyboard: tools have number keys; views have letter keys.

## What I want back

1. **A critique**, screen by screen, in order of harm: what makes it busy, what makes
   the tools cramped. Be specific and blunt.
2. **What to show first and what to bring on demand**: for a first-time user, for a
   field person verifying a measurement, and for an estimator. One layout that serves
   all three by disclosure, not three modes.
3. **A layout specification a programmer can build from**, at 1600 x 960 and at
   1366 x 768: each region, its size in px, what is in it, what collapses and how.
   Say plainly whether the panels should float with gaps or sit on the edges.
4. **The tools**: how to group eighteen tools (and room for more), the size of a tool,
   the size of its label, what is shown as an icon alone, where the fields of the tool
   in hand go.
5. **Show, Levels, Bring in, Write out**: where each belongs and in what form.
6. **A type scale, a spacing scale and colour tokens** (with hex values) for a light
   scene: sizes in px at 1x, the fewest that do the job; how Known, Assumed and Unknown
   and the trade colours live together without a rainbow.
7. **The estimate window**: the tabs, the recap, the sheet's rows.
8. **Three mockups** as HTML or SVG: the model window at 1600 x 960 with nothing
   selected; the same with a wall selected and a tool in hand; the estimate window at
   880 x 960.
9. **What to remove.** Then the changes ranked by effect against cost to build.
10. **Reference points**: which shipping products do each of these well (Google Earth,
    Apple's own apps, Shapr3D, Figma, SketchUp, Procreate, a well-made city-builder
    game), and what exactly to take from each.

Write the answer so that I can hand it to the programmer as it stands.
