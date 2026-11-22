# How the envelope and finish trades would use BuildOS on this renovation (2026-11-22)

Review by trade, run on the frozen command line (bin17), not the window code. Made-up job: an existing
24 x 16 ft house (one partition, 3 windows, 2 doors, hip roof 6:12), east wall demolished, a 12 x 16 ft
addition (3 new walls, 2 windows, 1 door, its own gable roof 6:12), a kitchen across the old east room
and the addition, starter assemblies and starter items. Job: scratchpad `review/tu2/job.json`.
Trades: masonry (4), roofing, siding and trim, insulation and weather barrier (7), windows and doors (8),
drywall, paint, flooring and tile (9), specialties (10), appliances (11), cabinets and millwork (12).

## What was checked by hand, and is right

- Existing south wall 24 x 9 = 216 SF less one window 12 and one door 21 = 183 SF: siding, barrier, batt, paint all 183.
- New south wall 12 x 9 = 108 - 12 = 96 SF; east wall 16 x 9 = 144 - 21 = 123 SF. Match.
- Main hip roof: 24 x 16 = 384 SF plan x 1.118 = 429.3 SF; sheathing 429.3/32 x 1.10 = 14.8 -> 15 sheets; shingles 4.293 SQ x 3 x 1.15 = 14.8 -> 15 bundles. Match.
- Addition gable roof (gables on the 16 ft edges): two planes of 12 x 8.944 = 107.3, total 214.66 SF, 12 LF ridge, 4 rakes 35.78 LF, 24 LF eave. Match.
- Rooms: Living 178.25 SF (11.5 x 15.5); Kitchen 364.25 SF (23.5 x 15.5) once the east wall is demolished and the addition drawn. Match.
- Marking the old walls existing refuses until told `--clear` or `--keep`; `--keep` left a roof tear-off line priced. Good habit; stops a re-bought house.

## Masonry (div 4)
Works: brick veneer by net area, ties and flashing, stone wainscot by LF, as assemblies; drawn from what is priced.
Half: veneer is one subcontract line by SF; no brick count (units per SF, 6.75/SF with mortar and waste), no lintel/shelf angle at openings, no sill, no
control joints, no stone by the ton. Matching existing brick on an addition (the real mason's job: tooth-in, the cut, the match) is not modeled.
Missing: chimney/fireplace, pointing, repair by LF. Wrong: nothing found.

## Roofing (div 7, steep and low)
Works (command: `roof`, `apply ... "Asphalt shingle roof, whole roof"`, `set roof-0094 edge:2 gable`): planes, eaves, rakes, hips, ridge, valleys, whole sheets, rolls, bundles, drip edge,
ice barrier, ridge vent, with IRC sections on the figures. The earlier roofers' review said no membrane assembly; there is now one for low-slope (tapered insulation, drains).
`line ... "Tear off shingles" surface_area` works and went to `export subcontractor` (429.33 SF; no SQ because the line unit is SF; the price must be typed per SF, a $/SQ price was refused).
Half: the tie-in. The addition's roof is a separate roof. The old hip's east end is not cut back, nothing joins them, so valleys 0 LF and headwall flashing 0 LF; the tie-in
valley/cricket quantity a roofer prices is simply absent, and ice barrier/underlayment for the joint is not there. Re-roof to the tie-in: the only way is the whole Main roof kept priced, or a line by hand.
`clashes` says "the roof's planes are not tested", so the addition's roof overlapping the old hip is not found.
Missing: tear-off layers, roof-area-by-condition (new vs. existing), pipe boots count (no plumbing vents drawn: "no figure"), chimney/skylight/curb flashing, fascia, soffit, gutter downspouts
(a gutter assembly exists by eave LF only), gable-end siding triangle, rake trim, step flashing along the old wall. Unit: SQ is the roofer's unit and the program shows SF for tear-off.
Wrong: no price comes out unless typed; fine, but "Pipe boots: no figure" is the right flag.

## Siding and exterior trim (div 7/6)
Works: lap siding, stucco, brick, by net area (openings out), as lines on each wall; finishes drawn in 3D.
Half: patch and match. `line wall-0004 "Patch siding" "net_area * 0.2"` priced 36.6 SF. That is a fudge percentage; typing a patch of 60 SF failed twice:
`patch=60 SF` ("not a unit it knows": it takes only feet) and `patch=60` makes the line EA, which then refuses a $/SF price. So a patch by SF cannot be typed. Real fix: a value with SF.
Missing: the eave-to-peak gable triangle (the addition's east gable is 12 x 6 / 2 = 36 SF of siding that no line owns: walls stop at 9 ft), corner boards by LF, J-channel, window/door casing by LF
(casing is 1 EA per opening, not LF), soffit, fascia, frieze, belly band, siding by the box/square (lap siding is in SF and not by the 12 ft piece or the SQ). "Weather barrier" sits in SF, not rolls.
Wrong: nothing.

## Insulation and weather barrier (div 7)
Works: batt by wall SF (183 SF matches), barrier by SF; the barrier is drawn when the Finishes switch is off.
Half: no R-value, thickness, or type on the batt (batt vs. blown vs. spray foam vs. rigid); no ceiling/attic insulation line (roof has no attic floor in the model); the room's "Ceiling" line is drywall and paint only.
Missing: rim/band joist, air sealing, vapor retarder choice, ice and water at windows, insulation of the existing walls opened for the addition tie-in (a real renovation cost), a rolls count for house wrap.

## Windows and doors (div 8)
Works: windows and doors cut walls (net area), counted EA, with flashing/casing/hardware lines per opening; the starter library prices them; the IFC output carries 5 windows.
Half: every window line is "Window unit 1 EA": the type name carries the size (win 3x4), but the supplier list shows "Window unit 2 EA" with no sizes, so the supplier gets no schedule.
There is no window/door schedule (mark, size, type, rough opening, swing, hardware). Replacing an existing window (existing + demolish + new) has no recipe. Casing is EA, not LF.
Missing: rough opening vs. unit size, headers sized to the opening, lite/egress flags, garage door, sliding door, jamb extensions for thick walls.

## Drywall (div 9)
Works: board by whole sheets with waste (13 sheets for 3 walls; 183 SF x 1.1/32 = 6.3 -> 7 per the demo rule), both faces on a partition, taped and finished as a labor/material line; ceiling by room.
Half: patching. Same patch-quantity problem as siding. The existing walls were marked existing and cleared, which removed the drywall, so the "patch drywall at the new opening" is a hand line.
Missing: 4x12 vs. 4x8 sheets, 5/8 type X (garage, fire wall), moisture-resistant in bath, level of finish, corner bead and tape/mud/screw counts, the ceiling by room has to be by drywall sheets, not SF. The existing ceiling being opened up for beams is not modeled.

## Paint (div 9)
Works: "Paint, two coats" by wall SF (183), ceilings in the room line.
Half: SF only, no coverage, no gallons; both sides of a wall; paint existing rooms touched (a "refinish the existing room" line is possible by hand but not by the room's own walls as existing).
Missing: trim by LF, doors and windows by EA, primer vs. finish coat, stain, exterior vs. interior, ceilings by room as their own line.

## Flooring and tile (div 9)
Works: hardwood and tile by room SF (178.25 for Living), base by room perimeter (LF around, cut at doors).
Wrong/half: a room is one thing. The Kitchen is half existing house and half addition (364.25 SF), and it priced new hardwood across all of it. A room cannot be marked existing, and `mark room` is refused. The refinish line
`Refinish hardwood, sand and 3 coats = floor_area` works and went to the subcontract list (178.25 SF), but the new/existing split of a room is not available. A floor was marked existing --keep without trouble.
Missing: boxes/cartons with waste, tile by SF with setting materials, thinset/grout/backer, transitions, underlayment, tile layout/cuts, stairs, sill heights, feather floors for the step between old and new floor heights.

## Cabinets, countertops, specialties, appliances (div 10, 11, 12)
Works: starter items: Base cabinet 36 in, Range, Refrigerator, Dishwasher, Sink, Vanity; set them down at a point, they are 3-D boxes that clash with the wall (clashes found: the refrigerator was 9.00 in into the east wall;
base cabinets 0.60 in into the south wall; the range 1.60 in), and `vif` lists each with `check: position, size; measure: height above its level`.
Wrong: the Kitchen assembly "Cabinets 1 EA", "Countertops 1 EA", "Kitchen appliances 1 EA" come out as quantity 1, How sure "Known" in `export supplier`: a plain 1 for a kitchen not yet designed, shown Known.
That breaks the program's own rule; it should be Unknown. The six placed items carry no lines at all (nothing prices a placed cabinet; a furnishing cannot take "Plumbing fixture", and nothing says "Appliance").
Missing: cabinet runs (edge to edge, total LF), wall and tall cabinets, filler and end panel, corners, a cabinet schedule by box code (B36, W3030), countertop by SF with cutouts and edge by LF, backsplash, a mounting height, appliance clearances,
hardware count, door styles. Specialties (div 10): nothing (bath accessories are one lump in the bath; no mirrors, grab bars, closet shelving by LF, fireplace, shower doors). Millwork: base only; no crown, casing by LF, stair parts, built-ins.

## Handing it to the supplier, and seeing what to check
`export supplier`: good where the units are right (sheets, bundles, rolls, pieces), mixed with SF for flooring, siding, paint, barrier. Placeholders show Known (cabinets). A window list with no sizes.
`export subcontractor`: only two rows (tear-off, refinish); clean and honest. `vif`: ranks clashes and heights by dollars; with no prices typed, every line says "no dollars yet" and the list is 16 questions in no useful order. The kitchen clashes are visible and say what to measure.
`export pm`: 90 rows, fine. IFC: windows, slabs, roofs written; I did not find the cabinets and the status of existing/demolish in it (README says status in IFC is not built).

## What these trades would do on paper or in a spreadsheet instead
Patch quantities (a siding or drywall patch of a given area), the tie-in at the old roof, the window and door schedule, the cabinet list by box code with countertop LF and SF, the finish schedule by room with
new/existing, the paint takeoff with gallons, the insulation by R-value, trim by LF. Because the program gives one line per element and no way to say "this much of this existing thing".

## The ten, ranked for this job

1. Part-of-a-thing work on existing: a patch/area entered on a line in SF (siding, drywall, paint, roof tear-off), and a room that is part existing, part new. Partly: lines exist, the SF value cannot be typed, rooms cannot be split.
2. Kitchen cabinet runs and schedule: base, wall, tall, filler, corner; cabinet LF and a box list by code. Missing.
3. Countertop by SF and edge LF with cutouts and backsplash. Missing.
4. Roof tie-in of addition to old roof: cut-back of the old hip, valley LF, headwall/step flashing, a clash/overlap test of roof planes. Partly (headwall exists, 0 LF).
5. Window and door schedule (mark, size, rough opening) and supplier list by size; casing LF. Partly (counted EA, no sizes).
6. Placed items priced: a placed cabinet, range, refrigerator, sink with a unit price and the supplier list, and the placeholder 1 EA lines shown Unknown, not Known. Partly (wrong flag).
7. Trim and exterior by LF: fascia, soffit, rake, corner boards, casing, crown, gable-end siding triangle. Missing.
8. Units in the trade's packs: SQ for roofing, boxes for flooring and tile, gallons for paint, rolls for barrier, 4x12 sheets and tape/mud for drywall. Partly.
9. Insulation by R-value and type, plus attic/ceiling insulation and the opened existing walls at the tie-in. Partly.
10. Three views (before, demolition, after) and tear-off layers/dumpster, with status in the IFC for the other trades. Partly.
