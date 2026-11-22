# How the GC, demolition, sitework, concrete and structure trades would use BuildOS on this renovation

Written 2026-11-22 by a reviewer acting as those trades (MasterFormat 1995 divisions 1, 2, 3, 5, 6).
Method: the frozen command line (`bin17`), a made-up job in `scratchpad/review/tu1/` (r.json): a 24 x 16 ft house
as it stands (four 6 in walls, a partition, a door, strip footings, a slab floor, a room), all marked existing
(`mark ... existing --clear`); the partition and the east wall marked to demolish, the east footing too; a 12 x 16 ft
addition (three strip footings, three 4 ft foundation walls, three 9 ft walls, a slab floor, a hip roof of four planes);
`import assemblies starter`; assemblies applied; made-up prices set line by line; then `takeoff`, `estimate`,
`estimate by-item`, `price`, `vif`, `clashes`, `export supplier|subcontractor|pm|vif|estimate|ifc`.
I also read the earlier review (E2) and the renovation research and looked at `demos/captured/renow` and `foundw`.
No cargo, no build, nothing in the repository edited except this file.

## What has changed since the earlier review (E2, a remodeler and a framer)

Fixed since: existing / demolish / new now exists (calls 148 to 150); a wall's studs, plates, headers and sills are
counted (call 137: "Studs 37 EA, Plates 10 piece" on my 40 LF of new wall); gable roofs and a slab by its own thickness
exist; scan import exists on the command line (`import scan`, PLY only).
Still true from E2: no change orders, no photos on a field question, no sub bids as objects, no stair, no
joists or rafters, no allowance in the contract sense, labor lines carry no rate unless the estimator types one,
the sheet does not follow what is clicked in the model.

---

## 1. The general contractor / remodeler running the job (division 1 and the whole)

### Works today
- One job, three states of a thing. `mark ... existing --clear` took the lines off what stands; the refusal text
  ("what stands is not bought again ... --clear or --keep") is exactly what a remodeler needs to hear. Checked:
  after the marks the estimate shows no line on the existing walls.
- A taking-down priced by face area: "Demolish partition, haul off" 144 SF at $2.50/SF = $360.00 (16 x 9 = 144; checked).
  "Demolish exterior wall" 144 SF at $3.50 = $504.00. Both appear in Division 2 and in the subcontract list.
- The room grows when the wall goes: Living 364.25 SF with the partition down (23.5 x 15.5 inside the faces),
  550.25 SF with the east wall down and the addition drawn (35.5 x 15.5). Checked by hand from reno.sh's figures.
- "Left out" is honest. `price` ends: "the sixteen divisions ~$6,461.50 leaves out 54 lines, named above". Dampproofing with no
  below-grade height shows "no figure", never zero.
- `export estimate` writes a workbook with 150 live formulas (3 sheets: Estimate, Recap, Divisions); `export pm` gives
  lines, a budget by cost type and by division.
- `check`: "its figures agree with what is drawn, and saving it back writes the same file".
- IFC out works (`export ifc`).

### Half there
- **Division 1 is four lines on a job** ("Building permit", "Temporary power and toilets", "Dumpsters", "Final cleaning",
  by 1 each, no prices). Shoring, protection, dust containment, site supervision by the week, porta-john rental,
  lead-safe (RRP) set-up for a pre-1978 house, engineer's fees, special inspections: I had to invent
  "Temporary shoring" as a line on the east wall, and as a Labor line it was refused ("is Labor, which is in-house work, and is not in hours")
  until I changed it to Subcontract. A GC sees that refusal as the program not knowing shoring is a job cost.
- **The takeoff mixes existing, demolished and new.** `takeoff` says "walls 8, 136.00 LF, wall face net 1,203.00 SF ...
  of the walls, 2 to demolish: counted above as they stand", "footings 7, 120.00 LF, footing concrete 6.09 CY", "floors 2, 576 SF".
  New footing concrete is 2.12 CY (priced by-item). The takeoff page says 6.09. Nobody can quote off that page; the estimate is right and the takeoff is not
  the quantity of the work.
- **Subcontract list is not by trade or phase and the "Where" is ambiguous.** `export subcontractor` writes 18 rows in one
  table: "wall 2, Demolish exterior wall" and "wall 2, Wall forms, both faces" are two different walls on two levels with
  the same name, no level and no path in the row. Demo, shoring, trench, forms, dampproofing, slab finishing sit together.
  Division is not in the file (it is in the PM file only).
- **Open questions (`vif`) fire only on what I drew as Assumed.** 15 questions on the addition (depth, height, type). Nothing asks
  the questions a remodeler cares about on the existing side: is that wall bearing, how deep is the old footing,
  what is in the wall (wiring, pipe), is there lead or asbestos, where does the old roof meet. The tool cannot hold those:
  there is no "bearing" or "hidden condition" field on a wall (grep: "bearing" appears only in the snap code and the framing notes).
  `vif` also says "Nothing priced rides on them" at the top while 84 lines ride; it means "no dollars yet" and is fine, but
  the first line reads strange.

### Missing
- Allowance (owner selection), contingency (hidden conditions on an old house: 10 to 20 percent), alternates, change orders.
  There is one flat Overhead and one flat Profit.
- Phasing and a schedule: demo, then shoring, then excavation, then foundation, then framing. Nothing orders the work.
- Permit set views (existing, demo, proposed) as sheets; the three views are named "not yet" in call 148.
- Photos on a field question.
- Costs that exist because the house is old (dumpster loads, floor protection, dust walls, patch and match).

### Wrong or misleading
- `price` and `estimate` print "Labor none" style gaps because the starter labor lines are in hours with no rate:
  every Labor line (Place the footing concrete, Form and place the wall, Frame the floor ...) reads "no figure" until the
  estimator types a rate for it on each element, one element at a time. For 7 new walls and footings that is 7 typing jobs per line.
- Rounding per element: wall sheathing is rounded up per wall (4 + 5 + 4 = 13 sheets for 360 SF, which is 11.25 sheets, 12 bought at the yard).
  The supplier list repeats the 13.

---

## 2. Selective demolition

### Works today
- Mark a wall or footing `demolish`: it is drawn as a red ghost, leaves the clash test ("wall-0010: it is to demolish, and in nothing's way"),
  and frees the room it bounded.
- A line on it priced by face area (`face_area`, `plan_length`, `opening_area` all work as quantities).
- A new opening cut into an existing wall: the wall's net area drops (216 SF face, 160 SF net, 56 SF opening for an 8 x 7 ft opening),
  and a line on the door by `opening_area` gives 56 SF in the subcontract list.

### Half there
- Demolition is a hand-made line every time: there is no demolition assembly in the 36 starter assemblies (listing of `export assemblies`).
  I typed "Demolish partition, haul off" and "Demolish exterior wall, shore and open" myself. They have no priced scope: strip finish,
  remove wall, cap and patch, haul, dump fee, protection, dust, floor cover.
- The demolished footing has no lines at all (nothing prices breaking out an old footing) and the program does not ask whether it
  should be demolished or kept as the tie-in for the addition's footing: the usual answer is to keep it and dowel to it.
  I marked it demolish and the new footing was drawn beside it with no conflict and no tie-in lines.
- Haul-off is inside the unit price. A dump load, a container, a tonnage cannot be stated or exported on its own.

### Missing
- Selective demolition inside a stays-wall: a new opening does not price demolition of the opening, shoring while it is open, a header, or the patch.
  The door line "Cut opening in existing wall, shore" I typed by hand is the only way, and a header line failed: `"width + 1"` was refused
  ("a length and 1 is a plain number"), so even typing `width + 1 ft` is not obvious.
- Debris quantity (CY, tons) from what is marked demolish. The model knows the volume of a wall and the thickness; it never turns it into a load.
- Finishes on existing walls to be removed or kept (plaster, tile, flooring) and the hazard flags (lead, asbestos, pre-1978).
- A demolition phase or sheet: PROPOSALS 33 lists three views; they are still "not yet".

### Wrong
- The takeoff counts a demolished wall as it stands (see above): a demolition sub reading `takeoff` reads existing walls as work.

---

## 3. Sitework and excavation

### Works today
- Per footing: "Trench for the footing" by LF (40 LF; I priced $6/LF = $240.00 on the three new footings).
- Slab: "Excavation and grading" 192 SF, "Base course, gravel" 2.37 CY (192 SF x 4 in = 64 CF = 2.37 CY, checked), vapor retarder 211.2 SF
  (192 + 10 percent), "Slab edge forms" 18.67 SF.
- Foundation drain assembly exists for the basement wall (by LF, with the IRC R405.1 note).

### Half there
- Excavation is priced by SF of floor, not by the cubic yard. The program has no grade, so no cut or fill, no bank-to-loose factor, no
  spoil haul, no backfill, no compaction. A sitework sub prices a footing trench and a foundation-wall dig by CY. I would take this off in a
  spreadsheet from the footing and wall depths.
- The new foundation walls sit on the level "foundation" (elevation -4 ft). That is the only depth the model knows about; a crawl or
  a full basement under the addition is one more level.

### Missing
- Site: utility locations, a driveway or walk to be broken, trees, silt fence, a temporary construction entrance, a site plan with the lot lines
  (README says "the ground (a plain plane with a switch). No grading, paving or utilities are drawn").
- Geotech (soil bearing, frost depth) as an Assumed or Known value that drives the footing width and depth.
- A dig or demo staging area in the plan.

### Wrong
- The made-up whole house prices "Site clearing and grading, by the EA" as division 2: an EA of site is not a sitework sub's number.

---

## 4. Concrete and foundations

### Works today (the strongest division)
Checked by hand on the three new footings and three new foundation walls (12 + 16 + 12 LF; 20 in x 10 in footing; 8 in x 4 ft wall):
- Footing concrete: the corner takes half the overlap off each leg. Footing 5: 12 LF is 16.67 CF = 0.617 CY straight; the program says 0.5959 CY (volume),
  0.6257 CY with 5 percent waste, shown 0.63 CY. Checked by working back the half-corner (a 20 x 20 x 10 in corner is 1.157 CF, half is 0.578 CF: 16.67 - 0.58 = 16.09 CF = 0.5959 CY).
- Wall concrete: 12 LF x 4 ft x 8 in = 32 CF = 1.185 CY, with 5 percent = 1.244 CY, matches "1.24 CY".
- Wall forms: both faces, 2 x 48 = 96 SF, matches.
- Bars by the lb, anchor bolts by whole EA ("2.6667 rounded up to 3: a count is whole", said on the line).
- Slab by its thickness: 192 SF x 4 in = 64 CF = 2.37 CY, x 1.05 = 2.4889 CY, matches the program's figure.
- Total new concrete for the order: 2.1156 + 4.1482 + 2.4889 = 8.75 CY.

### Half there
- The supplier file lists "Footing concrete", "Wall concrete" and "Slab concrete" as three lines. A ready-mix truck delivers one mix. The program does not say a
  concrete order, or let me group those three into 8.75 CY, 3,000 psi, at the pour dates.
- Concrete strength, mix, additives, cold-weather and pump are not fields anywhere.
- "Dampproofing, outside face" has no figure until the wall's below-grade height is entered (honest: "no figure", flagged Unknown).
- Tie-in to the existing footing: no dowel line (JLC: at least two #4 bars into the old footing, about 12 in, epoxy), no excavation to the old footing bottom,
  no continuous footing reinforcement counted as laps and corner bars (README: "corner bars, laps counted splice by splice" are Not yet).
- Pad footings, piers, stepped footings, grade beams, and the interior footings under a girder or post are not in the model (README Division 3).
- Stripping and curing, form rental, finishing the slab, control joints are subcontract lines with one unit each.

### Missing
- Existing foundation under an existing wall to be opened: the opening in a bearing wall may load an old footing that is too small. No question exists for it.
- Hand off to a concrete sub as a package: by phase (excavate, footing, wall, backfill, slab), with a drawing sheet.

### Wrong
- Footing "7 footings, 120 LF, 6.09 CY" in `takeoff` includes the existing ones (see above).

---

## 5. Structural steel and framing carpentry (divisions 5 and 6)

### Works today
- A wall's frame is counted: studs "~11 EA" on a 12 LF wall, plates "3 piece", headers and sills by opening, sheathing 4 sheets on 108 SF
  (108 / 32 = 3.375, rounded up). Walls priced "Frame and sheathe the wall" at "40 SF/hr" (the estimator enters the rate) = 9.00 hr for 360 SF (360 / 40, checked).
- Drawn in 3D behind the Framing switch.
- Roof sheathing by plane (10 sheets, 4 planes) and the roof's eaves, hips, ridge by LF.
- Wall height is a number, and the first-floor wall can run to the roof (gable).

### Half there
- **Framed floor is one line.** "Floor framing" is priced by SF (192 SF), "Subfloor" 6 sheets. There are no joists, no rim, no girder, no posts, no blocking,
  no hangers, no stair opening. A framer buys by the stick; none of it is derived.
- **Structure over an opening.** Where a bearing wall comes out, the load goes to a beam: LVL, a flush steel beam, or a header with posts and a footing.
  The model has no beam. `Kind` has Wall, Door, Window, Floor, Footing, Roof, Run, Item. A beam can only be a hand-written line on a door or a wall; the length is `width`,
  and adding a bearing allowance (`width + 1`) failed on units.
- **Division 5 is one line**: "Steel ridge beam, by the LF" on a ridge. There is no steel beam, lally column, steel post, connection plate, or lintel in the model or
  in the assemblies. There is no engineered-lumber line (LVL, PSL, I-joist, truss).
- Plates: the program says "piece" and does not say the length. Stud lengths (precut 92-5/8) are not stated in the supplier file; the supplier file says
  "Studs, 2x6 at 16 in, 37 EA". Headers and sills show "0.00 LF" lines for walls with no openings, and travel into the supplier list as rows of zero.
- A wall removed (not just opened) needs temporary shoring; the first line is mine.
- Roof: four planes and a hip. The addition tying into the existing roof is not modelled: the existing house has no roof, so there is no valley, no flashing line,
  and no "where the new roof meets the old wall". A hip is not what a framer would put on a 12 x 16 add-on (it is a gable or a shed); the default tool
  draws a hip.

### Missing
- Rafters, trusses, ceiling joists, ridge board, collar ties, hip and valley lengths, fascia, sub-fascia, blocking, fire blocking.
- Floor system: joists by span and spacing, rim, bridging, subfloor sheets with waste, stair.
- A beam or header schedule by opening and size; engineer's sizes as Known, with the engineer's letter as the source.
- Hold-downs, hangers, connector hardware by the EA.

### Wrong
- On the applied wall, the starter "Wall framing, 2x6 at 16 in" by face area is replaced by the counted lines (studs, plates, headers, sills): checked, no double line on wall-0063. But the README still lists the SF line as what the starter ships, so the two descriptions disagree.

---

## What I would do in a spreadsheet or on paper instead, and why

1. **The beam over the opened wall and its posts and footing**: paper and a call to the engineer. The program has no beam.
2. **Excavation by the CY and haul/backfill**: a spreadsheet (length x width x depth / 27, times a bank factor). No grade.
3. **A phased schedule and the order of the subs**: paper or Buildertrend. Nothing orders the work.
4. **Hidden-condition contingency and allowances**: a line in my own spreadsheet.
5. **The concrete order**: I add the three concrete lines myself and call the plant.
6. **Joist, rafter and stick counts for the roof and floor**: a lumber yard take-off.
7. **The demolition scope**: a short paragraph and a number per room, not per square foot.

## Ranked: what these trades most need for THIS job

(Exists, partly, missing; why it matters in the field or the money.)

1. **Takeoff by what is being built, what is coming down and what stays** - PARTLY (priced lines are right; `takeoff` and the supplier list count existing and demolished things). Every sub quotes off the takeoff; it overstates new concrete 6.09 vs 2.12 CY.
2. **A beam / header / post over an opened bearing wall, with its load path and footing** - MISSING. The single structural fact of this job; today only a typed line.
3. **A bearing / hidden-condition mark on existing things, as a field question with dollars on it** - MISSING (VIF asks only Assumed values on new work). The remodel's money is in what is behind the wall.
4. **Demolition as an assembly: unit scope, protection, dust, haul/dump, patch, shoring** - PARTLY (priced by face area if typed by hand). Demolition and protection are where remodel bids lose money.
5. **Joist, rafter, truss and header counting from the drawn floor and roof** - MISSING (framed floor and roof sheathing are SF lines). A framer buys sticks.
6. **Tie-in to the existing: footing dowels, old-footing depth, floor-height match, roof tie-in, valley and flashing** - MISSING. Where additions fail.
7. **Excavation, backfill and haul by the CY (grade, bank factor)** - MISSING (trench by LF; excavation by SF). The sitework sub's unit.
8. **Subcontract and supplier lists by trade and phase, with the level named and ready-mix grouped into one order** - PARTLY ("wall 2" appears on two levels; one flat list).
9. **Contingency, allowances, change orders and photos on a field item** - MISSING. A remodel bid carries hidden-conditions money and records what it found.
10. **Labor with a crew and a rate that sets once for the job, not per element** - PARTLY (rates can be typed; every labor line reads "no figure" until typed on each wall). Self-performed carpentry is the GC's own money.
