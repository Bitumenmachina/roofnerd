# The design side on this job: what BuildOS gives, from four chairs (2026-11-22)

Reviewer: a design-side reader (architect, kitchen designer, structural handoff, permit-set assembler).
Judged from: pictures in `demos/captured/` (renow 05 and 10, plan 05, draw 12, carry 04, gablew 04), and the frozen
command line run in `scratchpad/review/tu4/` (`run2.sh` builds a 28 x 20 ft house, marks it existing; `A.json` is a
12 ft addition on the east, `B.json` one on the north with a kitchen along a wall). No cargo, no window run by me.

## What I did, and what the program said

- Existing house: `wall --closed`, `floor --walls`, `room`, `mark ... existing --clear`. Works; the marks read back in `show`.
- Option A (east addition 12 x 20): `mark wall-0007 demolish`, three new walls, a floor, a Kitchen room at 34,10.
  Result: Living grew to 770.25 SF (the opened-up space) and Kitchen read "floor area unknown (Living was placed first
  and has the space this room's point is in)". Honest, but there is no way to say "the kitchen is this half of an open space".
  `PROPOSALS.md` line 29 lists "room separation lines" as not built.
- Option B (north addition 28 x 12, partition at x=14): same result, Kitchen and Family both unknown.
- Keeping two options: there is NO option or alternate in the model. The only way is `cp A.json B.json`. `compare` is
  only against a *prior estimate* brought in with `import prior`. Nothing sets A beside B: areas (A 800 SF of floor,
  B 896 SF), walls (A 140 LF, B 160 LF), price. I read those off two `takeoff` runs by eye.
- Price: after `import assemblies starter` nothing carries a price, so "price of A vs B" is "nothing priced yet"; both
  show "not priced at all: 8 things" until a price list is brought in.
- Cabinets: `import items starter`, then `place <job> <level> <type-id> --at x,y`. Place by NAME fails ("Base cabinet, 36 in is
  not in the job"); it must be the type id (`type-0060`), which the person must look up with `items`. Position is the
  item's CENTRE; there is no "along this wall", "butt to the last one" or "fill to the corner".
  I placed three 36 in bases, a 30 in range, a 24 in dishwasher and a fridge on the north wall of B.
- Clearances by hand against `clashes`: it is correct to the fraction. Cabinet centred at x 1.5, 36 wide, against the west
  wall centreline at x 0 (inner face 0.25): "in each other by 3.00 in" = 36/2 - 1.5 + 0.25 ... checks (it spans 0 to 3, wall
  face at 0.25 = 0.25 ft = 3 in). Range at y 30.7, 26 in deep, wall inner face 31.75: 0.40 in, hand figure 0.033 ft = 0.4 in.
  Dishwasher 1.80 in, hand figure the same. Two fridges overlapping: 18 in, correct. So the geometry is trustworthy.
  But `--clearance "42 in"` returned 26 findings, mostly cabinets "3.00 in apart" from the wall they stand against and a
  fridge "0.00 in apart": it measures nearest distance between any two bodies, not aisles. It cannot say "42 in between
  the run and the island", "15 in beside a range", "36 in clear walking path", "door swing", or work triangle.
- Counts only: the takeoff lists "Furnishing Base cabinet: 3 EA, Range 1 EA". No countertop LF/SF from the run, no wall
  or tall cabinets (the starter library has base only; wardrobe and desk are furniture).
- IFC (`export ifc B.json B.ifc`): 8 IFCWALL, 2 IFCSLAB, 1 IFCSPACE (only Living, since the others had no area), 3 IFCFURNITURE for the
  base cabinets, 4 IFCDISTRIBUTIONFLOWELEMENT for the range, dishwasher and two fridges (not appliances), no IFCBUILDINGELEMENTPROXY, no doors or windows because I drew none.
  Existing/demolish: the mark is a custom text property `work = "existing to remain (Known; survey)"` in a property set
  named `Plumbline_Values`. It is NOT IFC's own Status (`Pset_WallCommon.Status`/`PEnum_ElementStatus`). Revit, ArchiCAD or
  BlenderBIM will not read it as a phase; the demolished wall is written as an ordinary wall, solid, in the model. Another designer would
  see the old east wall standing in the addition.
- Sheets: `plumbline drawings` is only the underlay (PDF) list. No sheet, no print, no title block, no dimensions that are
  the model's own, no section, no elevation. In the window, the plan shows a length label per wall and room tags; the
  only dimension string in `plan/05-model.png` is the traced PDF's own.

## Per role

### 1. The designer laying out the addition and rooms

Works today:
- A closed wall loop plus Floor and Room gives a measured existing house in minutes (`draw/12-model.png`: Plan mode,
  typed lengths "24 ft", an inspector that shows Assumed/Known and the source). Typed dimensions and the sources on
  every value suit a builder who measures, and `measure` turns Assumed to Known.
- Existing, To demolish, New work: three buttons, pale/red-ghost drawing (`renow/10-model.png`: the red partition ghost;
  `renow/05-model.png`: the message "what stands is not bought again. Say which: Take its lines off, or Keep ..."). This is the single
  most renovation-shaped thing in the program, and it moves areas (rooms the work leaves) as it should.
- A gable can be made on a roof edge (`gablew/04-model.png`: lap/brick, shingle roof, eaves/hips/ridges in LF).

Half there:
- Rooms follow walls only. Open plan (kitchen into family) cannot be modelled without a fake wall. Room separation lines are
  PROPOSALS, unbuilt. For an addition that opens into the old house this is the first wall the designer hits.
- Roofs: a hip footprint, edges turned to gable. I did not find a shed roof, or a roof that meets an existing roof
  (valley, tie-in) as one action; a hip over each of existing and addition does not describe the tie-in.
- Moving things: Move works for placed items and shows clashes live (`carry/04-model.png`, "It is clear of everything"), but a
  wall moved does not drag its openings or the rooms' names; I did not test it.

Missing: stairs; room separation lines; a way to see existing and proposed superimposed (option A over B);
an "option" (a named alternative) of any kind; a level of ceiling heights per room; site/lot; door swing and handing.

Wrong or misleading:
- Plan reads as a drawing only with the PDF underneath. Alone (`draw/12`), walls are black bars, the openings are small blue
  ticks, no poche, dimension labels sit on the wall ends and collide ("16' 0"" on the door).
- 3D items are boxes: the vanity in `carry/04-model.png` is a translucent orange box next to blocks for the toilet. A kitchen
  designer cannot judge a layout from this.

### 2. The kitchen and cabinet designer

Works: exact sizes with a source, a maker's PDF made into an item (`items/`, `itemnew/`, Assumed, sourced to the line),
placed and moved by the pointer, quarter turn (`turnw`), and the overlap and nearness figures are right to a fraction of an inch (above).

Half there: cabinets are placed one at a time by centre point; the library has one base cabinet size (36 in), a vanity, a
wardrobe; I could edit a CSV (`export items`) to add 12, 15, 18, 21, 24, 30, 33, 42 in sizes, but each is its own item.
Appliance fixed sizes are there (range 30, dishwasher 24, fridge 36).

Missing: wall cabinets and tall cabinets (research survey says so too); a RUN (start at a wall, fill left to right, end
filler, corner); toe kick, countertop SF/LF and edge, backsplash, island with overhang; sink-to-cabinet cutout; clearances by
rule (aisle 42 in, work aisle 48 in, 15 in landing beside range, 36 in primary path, dishwasher to sink, fridge door swing);
elevation of the kitchen wall with cabinet heights and uppers; a cabinet cut list or a cabinet-shop export (a list of box sizes by
run with end panels). The item list prints 3 EA cabinets and no price: items arrive unpriced.

Wrong: Place by name refuses and the id is not shown anywhere a person would look unless `items` is run first. The clearance
mode is noisy: 26 findings for a plain run on one wall.

### 3. The structural engineer, handed what to size

Works: sizes of the openings the model knows (wall length, height, opening width), a way to mark a wall existing and a wall demolished (a wall taken
out is recorded as a thing the engineer has to deal with), a verify-in-field list with dollars (`vif`, `export vif`), and Assumed
vs Known on everything. The IFC carries the wall with its work word.

Half there: the research file `renovation-structure.md` is right that nothing here holds joists, rafters, beams, headers or posts. A demolished bearing wall
is a red ghost, with no "bearing" flag, no load line above, no "this wall carries the floor above" fact.

Missing: bearing/non-bearing on a wall; beam, header, post, footing pad as things with spans and loads the engineer fills;
floor joist direction; roof framing; a "what to size" list (every opening cut in a wall, every wall removed that is marked bearing, every
point load) as a sheet to hand across; stairs and shafts. Do not put span tables in (licensing, per the survey); a list of
questions to hand the engineer is what is wanted, and the Verify-in-field machinery is the right shape for it.

Wrong: the demolished wall goes into IFC as a normal wall; an engineer's tool would analyse the wall that is leaving.

### 4. The person assembling the permit set (existing, demolition, proposed)

Works: the three states exist in the model (New / Existing / Demolish / Temporary) and a view toggle called Demolition.
The existing-house sheet can come from a scan: `import scan` (PLY only, cut at a height into a sheet to trace over, to scale, Assumed).

Half there: three views are described in `how-a-renovation-is-done.md` Part 1 as "not built yet". One window with a switch is not three sheets.

Missing: all of it as an output. No sheet, no title block, no scale bar, no north arrow, no printed or PDF plan, no DXF/DWG,
no model dimension strings (chained, overall, to openings), no door/window schedule, no room finish schedule, no section or
elevation, no demolition notes/keynotes, no legend of the demo linetype. `export` produces estimate, CSV, xlsx, VIF and IFC
and none of them is a drawing. A permit set cannot be made from this program today; the plans have to be drawn in a CAD tool
and the program fed from them (the PDF underlay path, which works).

Wrong: IFC status as above; a demolished thing is drawn in the window as red ghost, but there is no existing/proposed
line convention (light/dashed/heavy) on a plan.

## The ten things the design side needs most for THIS job

1. **Options (A/B/C) of the same job, side by side**: areas, wall LF, price. Missing. Today it is `cp` and eyes. Without it the
   "try two layouts of the addition and the kitchen" the builder said he wants cannot be done in the program.
2. **Room separation lines / open-plan rooms**: Missing. The kitchen of an addition opened to the old house reads "unknown area".
3. **Printable plan with the model's own dimensions, as existing / demolition / proposed sheets**: Missing. A permit set.
4. **Cabinet runs, with wall and tall cabinets, countertop and toe kick, and a kitchen elevation**: Missing (single base
   cabinets exist, placed by centre point).
5. **Clearances by rule (aisle, landing, path, door swing)** instead of nearest-distance: Partly (the figures are right,
   the rule isn't there; 26 findings for one wall run).
6. **Existing/demolish carried correctly in IFC** (`Pset_*Common.Status`, PEnum_ElementStatus, or a phase), and appliances
   and cabinets as proper classes: Partly (a custom text `work`; appliances out as DistributionFlowElement).
7. **Structural handoff**: bearing flag on a wall, beams/headers/posts as placed things, and a "what to size" list for the engineer
   built from the demolished walls, new openings and point loads: Missing.
8. **Existing house from the scan in the window as a measured set of walls** (scan reader: PLY only, no walls found; walls from
   the scan app through IFC import are the researched first step): Partly.
9. **Roof tie-in and shed roofs for an addition, joists and rafters**: Partly (hip, gable edge; no shed, no tie-in, no framing members).
10. **Pictures a designer would judge a layout from**: items as what they are (a cabinet with doors and a sink as a sink; the vanity is a box),
    a plan with poche and door swings, ceiling heights per room: Partly.

**Single biggest gain for a builder who must be designing soon: options.** Ten minutes after he is in the program he has
an existing house and an addition, and the first thing he does is draw a second layout. The program has no way to hold or compare
two, so it has no way to be used for the decision it is bought for. Room separation lines (2) is the nearest second, because the
very first addition he opens into the old house gives him "unknown".
