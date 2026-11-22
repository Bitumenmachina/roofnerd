# The sweep: a residential architect (renovations and additions)

Basis: about 30 pictures read; the frozen command line (`bin22`) run on a made-up house (`sixteen`) and a small
24 x 16 renovation I built (`architect/r.json`, `h.json`, `h2.json`, IFC files). No window run, no cargo.
Pictures read: trades/first-floor-alone, trades/2-framing, trades/1-foundation, trades/7-waterproofing;
renow/01, 03, 05, 07, 09, 10 (model) and 10 (sheet); house/01, 05, 08; layers/04; layoutsw/10 (model, sheet), 05 (sheet);
roofw/05; scanw/05, 10; foundw/08; clash/03, 05; plan/08; first/06; glass/05; d2/08 (plan, copper roof); cabinetsw/11; bring/03.
Already said (not repeated): `research/trades-use-design.md` (no sheets, no room separation, option = copy file, IFC
mark is text). I checked what is still so and what has since moved. NOTE: call 164/165 (beam, post, pad, joists,
rafters), call 168 (plan-set PDF) and the wall-follow move (call 167) are **held as patches and are not in the
program I ran**; "not seen" for all of them.

## 1. Who I am and why I open this
1. Is what was built in 3D what I drew: wall lengths, heights, openings (size, sill, head), roof pitch and eaves, room names and areas.
2. Existing / to demolish / new, on a renovation, in plan and in 3D, and what that does to areas, rooms and price.
3. How it reads from the street and in section: storey heights, roof over wall, overhang, floor thickness.
4. Whether my model (IFC, PDF) came in whole, and whether what goes back out a builder or consultant can open and still read my phases.
5. Where cost moves when I move a wall or add 4 ft.

## 2. What I saw
- **house/01, layers/04** (dressed house): readable as a building in five seconds: stone base, lap siding, bronze
  double-hung windows with divided lites, hip roof. Good. What I cannot find: no soffit, fascia, gutter, porch, step,
  chimney, no roof overhang (roof edge sits on the wall line; the eave "6 in" is a typed option, default none). Reads as a box with a hat.
- **trades/first-floor-alone, 2-framing**: a level can be isolated; framing shows studs, headers, openings framed. Useful. No
  floor joists or rafters drawn (held, call 165), no ceiling.
- **house/08** (walk): rooms read, a window and a door, a duct at the ceiling. No stair, no ceiling finish, no trim. Eye height fine.
- **renow/01 vs 03 vs 05 vs 10**: renovation is real: Existing / To demolish / New work buttons on the wall panel;
  Demolition switch ("shown red"), Existing "shown pale". In plan a demolished wall is a red-pink bar; in 3D (renow/10) it is
  a barely-tinted pink patch on a pale wall, hard to see from the street angle. The message "Wall 5: to demolish. Living is
  now 364.25 SF (was 178.25)" is exactly a designer's question answered. The refusal message (renow/05) asking "Take its lines off, or Keep its lines priced" is a builder's
  question, correct, but blocks the flow.
- **renow/09 plan**: plan reads as a drawing only partly: black outline, light poche, room name and area in a label, **no
  dimension strings, no door swings, no north arrow, no scale bar**. The demolished wall is solid pink here (dashed in d2/08 sample plan, where it is a dashed line with blue ticks): two conventions in two pictures.
- **layoutsw/05, 10** (layouts): "Keep this as another layout / Set beside another layout" is the nearest thing to design options; matched by room name; shows area per room and cost per division, "same / not worked". Good idea, but it is files in a folder, not options inside one model; a layout cannot share the unchanged part.
- **roofw/05, d2/08**: roof planes carry area, pitch, edge lengths (14' 0", 19' 2"), eave/hip/ridge LF in the panel; a mixed-pitch roof (9:12 beside 6:12) is possible. Plan labels "1 284 SF 9:12" are readable. Good. No valley/tie-in to an existing roof as one action (not seen), no dormer, no shed roof (not seen), gable end triangle not drawn or priced (PROPOSALS 29).
- **foundw/08**: foundation plan with footing outline grey around wall, wall panel with "Concrete, as a line priced on it says". Fine. Level at -4' 0" appears in level list.
- **plan/08, first/06**: a public PDF plan traced: 3D walls rise over the PDF; PDF is faded under. Good feel, but only 4 walls, 0 doors, so I cannot judge fidelity of a trace; scale "1/8" = 1'-0"" is shown. Nothing said about drawing sheet size or how far off.
- **scanw/10, 05**: scan cut at 4 ft becomes a plan sheet to trace; message says "about 24' 0" by 10' 0"", "floor to ceiling reads about 9' 0"" — what an architect wants from a scan. Plan view shows only a lone wall: scan points themselves not drawn in the picture (nothing to trace over visible), so I cannot tell it worked.
- **bring/03** (IFC in): honest report "Brought in ... 4 walls, 1 door, 1 window, 1 room, 2 levels... Not brought in: 1 beam, 1 furniture, 1 stair: nothing here to bring it in as" plus unit note. The model's level "Roof" at 8' 0" arrives as a level with "height unknown". The loss is reported (right), but a stair and a beam being dropped means my addition's stair is gone.
- **Words** that are not an architect's: "Brought in", "Open items", "Detail comes to", "the line", "cost line", "Set"; "Show: Finishes / Architecture" (Architecture is a layer name, ok); "Known, Assumed, Unknown" (estimator's); "buildingstorey" appears in IFC props. "Rooms, inside their walls" is good. "Wall 1, work: existing to remain, and 1 more for the same reason" is Assumed-noise on every renovation wall.

## 3. What is missing or wrong for me (one line each)
1. Looked for: demolished wall in IFC as a demolished thing. Got: ordinary solid wall plus custom text `work = "to demolish (Assumed; walk-through)"` in `Plumbline_Values` (r.ifc). Standard is `Pset_WallCommon.Status` (PEnum_ElementStatus: new/existing/demolish/temporary). Revit/Archicad/BlenderBIM will draw the old wall standing in the addition; a consultant is misled.
2. Looked for: my own export read back. Got: IFC re-imported into a new job: demolish/existing marks lost, wall type names lost ("Ext 6" became "wall 6 in"), the roof "not brought in" (1 roof, 4 slabs), 24 property sets "not read". Reported, but it means a round trip through the program destroys the renovation phasing. Costs: a change order.
3. Looked for: IFC that others read as a building. Got: 0 IfcMaterial / layer sets, 0 IfcWallType, no `Pset_WallCommon` (IsExternal, LoadBearing, FireRating), no classification (Uniclass/MasterFormat) though the estimate is in MasterFormat; property set still named `Plumbline_Values` (old name). Every wall is the same thing to the next trade.
4. Looked for: section and elevation. Got: nothing. No section cut, no elevation view of any side, no storey-height dimension, no floor-to-floor, no ceiling height per room. The street view is a 3D orbit only.
5. Looked for: a plan I would put my name beside. Got: in the window the plan has no dimensions, no swings, no north/scale (the held plan-set PDF, call 168, has dimension strings, three phases, title box, but is not in the program; the lead's own look says no door/window pages seen, openings not dimensioned, no swings).
6. Looked for: design options inside one model. Got: layouts are separate job files (copy), compared by name. An addition scheme A/B means two files; one design change must be made twice.
7. Moved the east wall of the first floor 4 ft (`move wall-0011 --by "4,0"`): walls LF stays 210, floor stays 1,120 SF, the second-floor east wall, the roof and the footing do not follow; rooms go "2 of 5 missing", **price fell $16,000 (394,677 to 381,207) because 6 lines "left out"**, not because the house got cheaper. The "!" says it, but a falling price after enlarging is the wrong direction for a designer reading the top bar. (Wall-follow, call 167, held.)
8. Rooms: opening a partition leaves one room "Kitchen: floor area unknown, Living was placed first" (r.json). Honest, but a designer wants "kitchen is this half of the open space". Room lines exist in the window (Room line tool seen) but in my run `mark` of a demolished wall on `room` still gave unknown; not tried with room-line in the CLI.
9. Stairs, stairwell openings in a floor, fireplaces/chimneys, porches, decks, soffits, gutters, bay windows: none drawn (house reads as box with hat). An addition always has an eave; the default roof has no overhang.
10. Door swing and hand (PROPOSALS 31): doors are a brown slab; the 3D has no swing, plan has no arc. Clearances at doors cannot be judged.
11. Existing vs new in 3D: pale/red tint is faint in renow/10; no "existing + new" view that a builder reads (existing grey, new in colour, demolished dashed ghost) with a legend. Phase buttons per wall only; no "select all walls of this room and mark".
12. Wall heights: "Height 9 ft made-up survey" per wall; no way seen to set a ceiling height per room or a sloped/cathedral ceiling, nor attic floor (the roof sits on the walls).
13. Scale of numbers: four-place areas in Layouts (614.7188 SF) and 7-place in assumed lines; an architect writes 615 SF.
14. Clashes screen: 22 clashes on the sample, of which 13 are "wall and fixture in each other by 0.45 in" (tub against the partition): a designer's tub does sit against a wall; clearances (aisle, door swing) are not tested at all.

## 4. The best tools today, and where this could be ahead
- Revit: Phase created/demolished per element, phase filters set what a view shows (new, existing, demolished, temporary) with graphic overrides; demolition plan = filter, not a copy. Source: https://help.autodesk.com/cloudhelp/2024/ENU/Revit-GetStarted/files/GUID-FE4C8D4D-F51D-486F-81E9-53A530E833A2.htm and https://help.autodesk.com/cloudhelp/2023/ENU/Revit-DocumentPresent/files/GUID-92CF3963-780B-42FA-BAED-1265E3E57CA7.htm. Design options are a separate feature (not read here).
- Archicad: three Renovation Statuses (Existing, To Be Demolished, New) and Renovation Filters (Change Plan, Existing Plan, Demolition Plan, After Demolition, Planned Status) that decide how each status draws in a view. Source: https://help.graphisoft.com/START/27/INT/_AC27_Help/050_ViewsVB/050_ViewsVB-130.htm, https://help.graphisoft.com/START/27/INT/_AC27_Help/050_ViewsVB/050_ViewsVB-127.htm.
- IFC: the standard carries a renovation status on elements: `Status` in `Pset_WallCommon` (PEnum_ElementStatus, described as mainly for renovation and retrofit; values new/existing/demolish/temporary; case matters). Source: https://ifc43-docs.standards.buildingsmart.org/IFC/RELEASE/IFC4x3/HTML/lexical/Pset_WallCommon.htm and https://ifc43-docs.standards.buildingsmart.org/IFC/RELEASE/IFC4x3/HTML/property/Status.htm. (The values list I read through a translation; check the page.)
- What none of them do that this could: the renovation states already move the **price** and the **field list** (to demolish, Take its lines off / Keep priced; VIF dollars). Revit/Archicad show phases; they do not say "this demolished wall takes $X of finish off and $Y to patch, and is it bearing, a VIF". Ahead, not level: one switch "Existing / Demolition / Proposed" that also shows the dollars of each, and a demolition plan that lists the quantities of what comes down (LF, SF) on the sheet. The Assumed/Known/Unknown colouring of an existing-conditions model (every dimension from a survey is a guess until measured) is something no BIM tool shows an architect; it is exactly the existing-condition risk.

## 5. My ten
1. **Write the work as IFC's own status.** Walls/doors/windows/floors export `Pset_WallCommon.Status` (and the siblings) = NEW / EXISTING / DEMOLISH / TEMPORARY, plus IsExternal, LoadBearing (from the wall's bearing flag when it exists), ThermalTransmittance blank (Unknown) rather than missing. Keep the text `work` property. Done when `export ifc` of r.json shows Status DEMOLISH on wall-0011 in IfcOpenShell, and BlenderBIM filters it.
2. **Read it back.** `import ifc` of the program's own file returns demolish/existing marks, type names and the roof; IFC from others reads Pset Status into the mark (Assumed, sourced "IFC Status, file name"). Done when r.json -> r.ifc -> r2.json `takeoff` shows the same "to demolish 1 wall, 16 LF" and the roof.
3. **Section and elevation.** A cut line drawn on the plan gives a section view (floor-to-floor, wall heights, roof pitch, footing depth, ceiling); four elevations, one click; dimensioned storey heights from the model. Done when a click on the south wall shows its elevation with window heads and sills in feet-inches.
4. **Dimensioned plan in the window.** Chained and overall strings, door swing arcs, window symbols, room name/area, north arrow, scale bar, matching the held plan-set PDF; one dash style for demolition in both window and PDF. Done when the plan picture for renow looks like the d2 plan with strings.
5. **Phase view as one control.** Existing / Demolition / Proposed / Change views (Archicad's names are trade language) over the one model, with existing in grey, demolished dashed or hatched red, new in colour, legend, in 3D too. Done when renow/10 shows the old partition unmistakably from the street angle.
6. **Options inside the job.** "Scheme A / Scheme B" on the changed elements only; unchanged walls shared; the Layouts table reads off them. Done when changing the kitchen wall once updates both and the area and dollar difference shows in the top bar.
7. **Move a wall and its neighbours follow** (floor edge, footing, second-floor wall, roof edge, openings, rooms), and the price change shows with what it left out named: never a lower price because something became unpriced. Done when moving the east wall 4 ft east changes floor SF, footing LF, roof SF and price upward, and the 6 lines are not left out.
8. **Rooms by room line in the CLI/IFC too, and a room that is "half of an open space"** with its area. Done when Kitchen after opening the partition reads 182 SF, not unknown.
9. **The things an addition has**: eave overhang default and soffit/fascia/gutter drawn and priced, a stair and stairwell, porch/step, chimney, door swing; dressed so the street view reads as a house. Done when the house/01 picture shows an overhang and the estimate line for the gutter lands on a wall's panel.
10. **IFC materials and types.** Wall types with layers (stud, sheathing, siding, drywall) as IfcMaterialLayerSet, IfcWallType, classification codes (MasterFormat division) on elements, property set named BuildOS. Done when Revit shows the wall's layers and its division.

## 6. What must not change
- The three work buttons per element (Existing / To demolish / New work) and the sentence "Living is now 364.25 SF (was 178.25)" after each.
- Known / Assumed / Unknown on every dimension, with the source of a survey number: right for existing conditions.
- The IFC import report listing what was not brought in, by name, with the unit note; and the export's "not written" list.
- Rooms refusing a false area ("Living was placed first...") rather than inventing one.
- The scan note "about 24' 0" by 10' 0", floor to ceiling about 9' 0", Assumed".
- The dressed house (stone base, lap siding, divided-lite windows) and roof planes with area/pitch/edge LF; the see-through and level switches; walk mode.
- The VIF list with dollars (the structural question on a removed wall belongs in it).
