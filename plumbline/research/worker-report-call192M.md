# W-192M notes (written as I go)

## Before any code (read only; no cargo before 05:35 UTC)

### 1. BIMCraft search
`grep -rli gable crates apps` and `grep -rli "top_profile|attach_top|..."` over /home/user/ipedrax/bimcraft, whole lists read where they bear on walls and roofs.
- BIMCraft has a wall attached to a roof: `Wall.attach_top: Option<Guid>` (`crates/core/src/model.rs:277`), a setting a person makes on the wall; its top then follows the roof's underside.
  `walls.rs`: `Bound::of` (:526) builds the roof's planes in convex pieces; `Bound::over` (:558) takes of two planes over a point the LOWER;
  `top_at` (:667), `heights` (:673, sampled 32 times along the wall), `prisms` (:720, the wall cut where the attached roof changes plane, e.g. under a gable "the parts either side of the ridge", test at :949).
- It has no rule that finds a gable by itself: the attachment is typed per wall. Ours finds the roof over a wall by plan (call 180/185) and works the figure; nothing is saved.
- Ported: nothing as code. The one rule taken from it (said in PORTMAP.md): where two roof planes cover a point, the lower is the roof's surface there (`Bound::over`).
  The geometry (convex pieces, mm planes, `Plane::at`) is a different model from this program's `SolidPlane::height_at` (feet, eave height + rise over the eave line), which already is the roof's height over a point.

### 2. Roof edge kinds; how an edge is found over a wall; the shed's high edge
- Kinds (components.rs `Kind`): Eave, Rake, Ridge, Hip, Valley, Headwall, PitchBreak, Parapet. A solved roof (`solve.rs`) gets its lines from `roof::lines_of(planes)`: a line one plane has: Eave if on the plane's eave line (or level and not the high side: a stepped-back eave), Rake if it climbs, Ridge if level and the plane falls away from it (a shed's top, with ONE plane). Two planes: Ridge/Hip/Valley/PitchBreak. An entered Headwall is accepted where the found line is a one-plane Ridge (`roof::compare`).
- Each such line is a `Run` entity (`ends`, `planes`: paths). A wall under an edge is found by the plan alone: `carries::edge_on(wall, edge, others, ring)` + trim's filter (`across <= thick` or `<= 4 ft and outward`); `trim::overhangs` takes the wall that shares most length (tie: the highest).
- So the lead's rule CAN be told from what the model holds: an edge over a wall whose kind is Rake, or Ridge/Headwall with exactly one plane (the high edge of a shed), is not an eave. An Eave over it (or nothing) is none. Hip/valley/pitch-break lines have two planes and are never a footprint edge.

### 3. Roof's height over each wall's line, the demonstration's house
(Worked from `SolidPlane::height_at` = eave_height + distance from the eave line x pitch/12; the roof drawn 1 ft outside the walls' lines, eave height 9 ft.)
- Gable roof, 6 in 12: over the west/east wall lines (1 ft in from the rake): y = 0 and y = 16 are 1 ft from the eaves at y = -1 and y = 17: 9 + 1 x 0.5 = **9.5**; ridge y = 8: 9 + 9 x 0.5 = **13.5**. Lead's hand figures agree. Area 40.00 SF each.
- Shed, eave on the south (y = -1): at y = 0: 9.5; at y = 16: 9 + 17 x 0.5 = **17.5**. Agrees. (East wall: 9.5 at the south end to 17.5 at the north end: 16 x (0.5 + 8.5)/2 = 72.00.)
- Note: the east wall's line runs south to north (first end at (24,0)) and the west wall's north to south (closed ring from (0,0)): the tops as "feet along the wall's line" run the other way on the west wall in the shed (17.5 first). The lead's "(0, 9.5), (16, 17.5)" is the east wall's.

(Further sections appended below as the work goes.)

## Revision 2: none under an eave takes the weakest status of wall line/top, eave run, plane; no-roof stays Known. Test none_under_an_eave_is_as_sure_as_the_roof_it_rests_on red then green; nothing existing moved.
