# money-structure: judge's notes (written as I go)
Program: frozen build at 8627a83. Folder: judges/money-structure/. Made-up numbers only.

## Piece 1: call 175 (money fixes)

### Findings so far

**F1. Siding is under-bought when a window's sill is below the top of the stone (wrong quantity, small money).**
Repro: new job, level 10 ft, wall 6 in, window 5x5, `import assemblies starter`, a 40 ft wall 10 ft high,
`apply ... "Exterior wall, 2x6, lap siding over a stone wainscot"`, `window --offset 20ft --sill "1 ft"`.
Said: net area 375 SF, net length 40 LF, stone 40 LF, "Lap siding" 255 SF (= 375 - 3 x 40).
By hand: the window spans 1 ft to 6 ft; the part of it from 1 to 3 ft (5 ft wide x 2 ft = 10 SF) lies inside the stone's 3 ft band
and is taken off twice (once in net area, once in the band, which is 3 x full length). Siding bought = 400 - 120 band - 15 (window above band) = 265 SF. Program: 255. 10 SF short.
Same at sill 0.5 ft (255). With sill 0 the window is taken out of net_length, so that case is right (335 - 3x29 = 248 hand-checked ok).
The call's rule ("openings that come down to its floor") is what is built; the case of a sill part-way up the stone is not covered
and nothing says so. Quiet, no flag. Siding is Assumed anyway (the 3 ft) so it is shown with ~, but the error is not the 3 ft.

**F2. A door hanging off the end of a wall is accepted at the command line, then the whole wall goes Unknown; `check` says nothing.**
Repro: wall 0,0 40,0; `door ... --offset "0 ft"` (or 40 ft, 50 ft). Command prints "door-0047: a door in wall-0046" with no word of warning.
`show`: net area, net length, opening area, lap siding, stone, hang all "?" unknown (honest). But `check` prints
"reads, its figures agree with what is drawn" and only the long Unknown list of `show` / `estimate` hold the reason ("door-0047 starts 1.50 ft before the start of its wall").
Same for two doors over each other (offset 20 and 20.5): accepted, all Unknown. A builder who types the centre at the end of the wall (the natural thing for "a door at the very end") loses the whole wall's figures and sees no refusal. Misleading, not wrong money (nothing is invented). Note "offset" is the door's centre, not its edge, and the usage line does not say so ("--offset 5 ft").

### Held
- Doors: 1 door at 10 ft -> net 380, net_length 37, stone 37, siding 269 = 380-111 ok. Two doors + sill-0 window (5 ft): net 335, net_length 29, siding 248 ok. Doors touching (10 ft and 13 ft centres): net_length 34, ok. Door flush at 1.5 ft and 38.5 ft: ok (37).
- A wall 2 ft high with the 3 ft wainscot: Lap siding Unknown, reason given by `check`: "(the formula comes out below zero)". Honest. (Stone still priced 40 LF on a 2 ft wall; not wrong.) Negative via a big window on a 3.5 ft wall: same, Unknown, not negative.
- Wainscot line on a second-floor wall: stone 20 LF and siding 180-60=120 are produced on an upper floor wall, same as a ground one; the program does not ask whether a stone wainscot belongs 10 ft up (arguably a drawing-fault, not a program fault).
- Starter assembly from the library applies cleanly (13 lines).
- Sample house base: each room's priced base == its own base_run (10 rooms, e.g. kitchen 45.08, living 99.285), as the call says.
- Totals: `estimate` of `sixteen`: Job cost $390,174.76 = sum of 399 line amounts = sum of 16 divisions = Material+Supervision+Subcontract+General in the recap; CSV export by division sums to the same.

**F3. A hole's place is wrong once the run is split (field list sends a person to the wrong spot).**
Repro: new job, level 9 ft, wall 6 in `wall --points "0,0 20,0"`, pipe 4 in, `run ... pipe --points "6,-3 6,3" --height "4 ft"` (Assumed).
`clashes`/`vif`: "1 hole, 4.00 in round, at (6, 0, 4)" (right). Then `split <pipe> --at "6,-1"`:
"wall 1 [wall-0009] and pipe 2 [pipe-0011] passes through it (a hole to cut...), on ground, at (5.9726, -1.375, 4.0438)".
Right: (6, 0, 4). The stated point is 1.4 ft out from the wall and, with the split at y=-1, is not even on pipe 2 (which runs -1 to 3 ... the point y=-1.375 is on the other piece). Same figure (5.9726, -1.375, 4.0438) for splits at y=0, 0.2, 1, -1: it is not following the split. Un-split, and in the riser case, the point is also a little off the pipe's axis (6.0533, 5.064 vs 6, 5), but that is an inch. Count of holes is right (1); where it is, is wrong. Misleading (the VIF line "where is still to verify, at (...)").

**F4. A pipe lying inside a floor slab is "1 hole, to cut".**
Repro: floor 10 in at level 2 (elevation 9 ft), `run ground pipe 1in --points "2,5 10,5" --height "8.6 ft"` (inside the slab, 8.17 to 9.0, both ends inside the outline).
Said: "Through a wall or a floor (a hole to cut): 1 ... pipe 1 and floor 1: 1 hole, 1.00 in round, at (6, 5, 8.6)". A conduit cast in a slab is no hole; the pipe never comes out. The call asked for "a run lying inside a slab" to be tried: it is counted as a hole to cut, i.e. a thing a plumber/estimator would price (sleeve/core) that is not there. Judgment call, but it is a number that goes on the list. Small.

**F5. Round trip of the program's own BIM file moves every wall (wrong quantity, unsaid).**
Repro: 4 walls 0,0 20,0 / 20,0 20,12 / 20,12 0,12 / 0,12 0,0 (6 in, one 4 in), `export ifc`, `new`, `import ifc ... --status Known --source x`.
Written: wall 1 (0,0)-(20,0) 20 LF. Read back: "(-0.25, 0) to (20.1667, 0)" 20.4167 LF; wall 2 12 LF -> (20,0.25)-(20,12.25) 12 LF; wall 3 20 -> 20.0833; wall 4 12 -> 11.5 LF. Another job (30 ft wall existing + 20 ft wall new + door/window/floor): takeoff before: "to build 20.00 LF, existing to remain 30.00 LF, wall face net existing 255 SF"; after: 19.75 / 30.25 LF, 257.50 SF. The sum holds (64 and 50) but each wall's line, length and the existing/new split of the quantities moves a quarter foot, and the import's notes ("not read: 18 property sets ... ") say nothing of it. Call 175 item 7 says "the same marks, type names and roof" and the demo's last step says "takeoff of the job read back says what the first job's did": by the program's own takeoff that is not so. CLAUDE.md: "For any other format, loss is reported, never silent." Everything read is marked Assumed (said), but this shift is not said.
Also lost and said: roof ("a roof is not brought in from a model yet", said, owed per call 175 as built), footings ("2 footings: nothing here to bring it in as": the program's own footings are lost on the way back, said but the 50 LF and 3.70 CY of footing concrete vanish from the takeoff).

**F6. A door in a wall marked to demolish is written but cannot be read back, and the refusal tells you to do something the program itself did not ask.**
Repro: wall 2 marked demolish, door in it marked demolish (as the program allowed), `export ifc` then `import ifc` into an empty job:
"1 door: this program would not take it: wall-0006 is to demolish: a door is not set in a wall that is coming down. Mark the wall existing or new first, or draw the new wall it goes in". The first job holds exactly that door (a demolished door in a demolished wall, which `mark` says "wall-0048: to demolish / door-0051, in it: to demolish"). So the program's own file loses a door it made. It is said, not silent, but the advice contradicts the program's own earlier statement. Untidy/loss.

**F7. Software words on the import's note.** "not read: 23 property sets and 0 materials in the model ... the Status it says ... and the LoadBearing it says" and "(IFC4)", "IFC". A builder does not say "property sets", "LoadBearing" (a raw identifier). Word finding. (The export's "every value entered on each goes with it" is fine.)

**F8. `connect` accepts a pipe whose end lies deep inside the fixture it is connected to, and says nothing (pre-existing, not call 175; untidy).**
Repro: tub 5x2.5x1.5 at (6,5) (face y=6.25), pipe 4 in ending at y=6.25, 6.1, 6.0, 5.9, 5.5 then `connect`: 0 clashes for every one, including an end 9 in inside the tub. Unconnected it is a clash (1.80, 3.00 ... in). "Set against" is not involved. At y=5.0 (centre) it is finally a clash (3.88 in). Not wrong by the written rule ("as deep as the joint is in it") but a pipe that ends 9 in inside the tub body reads as fine.

### Set against / same space: tried, held
- Tub into its wall 1.9 in / 2.0 in -> "Set against 1", same space 0; 2.1 in and 2.5 in -> same space 1, set against 0. Boundary exact.
- Cabinet in a corner 1.5 in into two walls: two set-against lines (one per wall), 0 clashes; with a second cabinet 4 in into the east wall: same space 1 ("in each other by 4.00 in"), the corner pair stays set against.
- Branch into a trunk: a 2 in branch beginning at the trunk's centre line or at its skin: no clash; beginning 1.2 in past the centre line (y=4.9): a clash, 3.00 in; a 1 in branch begun at 4.95 down to 3.0 into a 4 in trunk: clash "2.50 in" every time. I could not make anything real hide as "set against" through the joint rule: the depth for round pipes is the sum of the radii, so the 2 in limit is never met by a branch that is really in the trunk. (Within 3 in of a joint and under 2 in deep is a corner that needs a very small overlap; did not find one.)
- Nothing real hid as "set against" that I could make by hand, except that a placed thing in the 2 in is always called "set against" whether or not it was ever set against that wall: the test is "thing lies in the band of the wall" and nothing else (`things_against` in building.rs). A vanity 1.9 in into a wall it was drawn ON the other side of would still pass; I did not find a case where that is wrong in a practical drawing.
- Hole counts: riser of 3 pieces through one floor = 1 hole ("through 3 pieces of the run"); carried up through a second floor = 2 (one per floor); a run through a wall twice (a U) = 2 holes at their own places; a straight run split at the wall = 1 hole (count right, place wrong: F3); two 1 in pipes 1.2 in apart through the same wall = 2 holes (conservative, no one hole).

(Also piece 1, found late: **F9.** A roof marked to demolish writes Status DEMOLISH on the IfcRoof only; its four IfcSlab planes (which carry the shape) carry no Status, so another tool counting slabs sees new roof slabs. A beam or post marked "existing" writes no Status at all (beams, columns not in call 175's list; only the program's own "work" text carries it); the export's "not written" list does not say so. ifcopenshell (`validate`) passes all my files: 0 issues; Status and LoadBearing read back through `ifcopenshell.util.element.get_psets` as the call says (NEW/EXISTING/DEMOLISH/TEMPORARY; LoadBearing true/false; absent where bearing unknown). Untidy.)

## Piece 2: calls 164 and 165 (structure)

**F10. Two joists counted on one line when a mark falls on a notch edge (wrong quantity, small).**
Repro: new job, floor 10 in, `floor --points "0,0 24,0 24,16 12,16 12,10 0,10"` (the demo's L, mirrored: notch at the NW), `line <floor> "Floor joists, 2x10 at 16 in" joists`, `line ... "Rim" rim_length`.
Said: joists 20 EA, joist_length 256 LF. The demo's own L (0,0 24,0 24,10 12,10 12,16 0,16): 20 EA, 250 LF.
By hand (the same floor mirrored must give the same): mark 9 at 12 ft lands exactly on the notch's edge member at east 12.0625; the mark's joist runs the full 16 ft and the edge member (6 ft, y 10 to 16) lies on it. Right: 19 joists, 250 LF. Program: both counted: +1 EA, +6 LF. The merge rule only drops a piece "over the same length", and here the mark's piece is longer than the edge member. A U (0,0 30,0 30,20 20,20 20,8 10,8 10,20 0,20) does the same (26 EA, 420 LF; right 25, 408: the 12 ft edge member at east 20 is under a 20 ft joist). Plain rectangles: all hand-checked right (see Held).

**F11. A floor drawn inside another floor on the same level is accepted and its area counted twice.**
Repro: floor 0,0 24,0 24,16 0,16 on level 1, then floor 6,4 12,4 12,8 6,8 on the same level: accepted "floor-0010: a floor on level-0002"; `takeoff`: "floors 2   408.00 SF" (384 + 24); `check` says "reads, its figures agree with what is drawn". An identical outline is refused ("a second floor there would be counted twice") and footings/walls laid over one another are refused; a floor inside a floor is not. (This job has no floor with a hole: "This program has no floor with a hole in it yet", DECISIONS.md; so the only way a person draws a stair opening is a second floor, which then adds area.) Wrong quantity, silent.

**F12. A beam or a post drawn twice on the same line/spot is accepted and counted twice.**
Repro: `beam ... --points "0,0 10,0" --top "12 ft"` twice; `post ... --at "3,3"` twice: both accepted; `structure` lists Beam 1 and Beam 4 identical, 10 LF each, and "Members whose size is not Known: 6 of 6". `check`: clean. Duplicate footing/wall/floor are refused with "would be counted twice"; beams and posts are not guarded. (A pad twice is likewise open: not tried.) Wrong quantity if drawn by mistake; silent.

**F13. A beam with no post under either end, a beam top above the ceiling or below the floor, a pad top 5 ft above the floor: nothing is said.**
Repro (clean job, level 9 ft): `beam --top "12 ft"` accepted, listed "top 12 ft above the floor"; `--top "0.5 ft"` with an 11 7/8 in deep beam (its underside is 5 in below the floor) accepted; `--top "-2 ft"` accepted; `pad --top "5 ft"` accepted (listed as 5 ft above the floor); a post 12 ft tall on a 9 ft level, and a post with base 5 ft and height 9 ft (top at 14 ft): accepted. A beam drawn at the partition's line with no post at either end (the demo's beam, minus its posts): `structure` and `vif` say nothing of what it bears on at either end ("the beam's own questions (its size, what it bears on at each end: call 164)", call 176 as written; call 164's own text never promised such a question and none is built). So the bearing question for a beam end is not asked anywhere. Misleading/untidy; no money moves.

**F14. A weight per foot typed "18 lb/LF" is refused as having "no unit".**
Repro: `type beam b --width "5 in" --depth "8 in" --weight "18 lb/LF"` -> `"18 lb/LF" is a weight per foot with no unit: write it like "18 lb/ft"`. Same sentence for "0.018 kip/ft", "18 lb / ft", "26.8 kg/m", "18 lb/in", "18 lb", "18 pounds per foot". The program's own unit for length is LF everywhere else (a beam is "16 LF", the library "$9.50/LF"), so a builder who types the program's own word is told he left the unit off. A word/refusal finding. ("18 lb/ft", "18 lbs/ft", "18 plf" are taken and stored as "18 lb/ft".)

**F15. Starter assemblies put a lumber size and spacing in a line's name and nothing flags it Assumed or asks it.**
Repro: `import assemblies starter`; `apply <floor> "Framed floor"`: lines "Floor joists, 2x10 at 16 in" 19 EA and "Rim joist"; roof assembly "Rafters, 2x8 at 24 in"; wall "Studs, 2x6 at 16 in", "Headers, two plies". `vif`: "Verify in field: nothing. Every value that can be measured is Known"; `structure`: "No beam, post or pad footing is in the job". The "~" on the joist count is for the span direction only (the note says "joists usually span the short way"); the 2x10 and the 16 in are the library's, are not marked as the program's guess, and the spacing drives the count. CLAUDE.md/call 164: "the program sizes nothing". It is a table the estimator can edit, but a reader of the sheet sees a size on a line as if drawn. Also: a header's depth is nowhere asked (call 175 item 6's field question "owed", still not built): a bearing wall cut for a new window (wall existing, window new, said bearing) -> `vif`: 1 question "check: ~bearing" only; `structure`: "wall 1 is cut for window 1; bearing: bearing (Assumed: guess)" and no header size is asked or said to be unsized.

**F16. A pipe through a joist bay says nothing, and the list of what is never tested does not say joists, rafters or studs are not tested.**
Repro: second floor 10 in, pipe across it at 8.7 ft inside the floor's depth, with "Floor joists" on the floor: `clashes` -> "Through a wall or a floor: 1 hole"; with the pipe a little under the floor (no floor in its depth): nothing; `Never tested:` lists walls/floors, joints, doors, roof planes, beam/post/pad, but not the joists/rafters/studs counted from lines. A pipe across joists needs a bored hole in each joist; the program never has a joist to bore. Not wrong by the written rule (joists are "figures of the floor, not things"), but nothing says so. Beam: a 12 in duct through an 11 7/8 in LVL at east 12: "beam 1 and duct 1: in each other by 11.88 in" (right: beam depth), and it is on the field list.

**F17. Raw words.** Rafter line with no spacing in its name ("Rafters 2x8"): quantity "unknown (follows from a flagged input)" with no flagged input shown; it should say the line names no spacing. "Rafters, 2x8 at 1 in" and "at 200 in" and "at 0 in" are also Unknown with that same reason. Post type typed with both --round and --width: the refusal quotes `--width "3 1/2 in" --depth "11 7/8 in", or --round "6 in"` as a post's sizes (an LVL's size given as an example of a post's: the program offers a size to type). The beam refusal "is 0.001 ft long; a run goes from one point to another" says "a run" and does not say how short is too short. The BIM file's quantity notes carry raw names: "plan_length × height", "face_area − opening_area", "(section_area × its length ÷ 27) × 27, in cubic feet" (the last is also silly arithmetic: ÷27 then ×27).

### Piece 2: tried and held
- Beam: 16 LF for 12,0-12,16 (0,0 to 20,0 is 20 LF; 0,0-3,4-20,0 is 5 LF + 17.4642 LF, right: sqrt(17^2+4^2)). Refused: ends one over the other, 0.001 ft long. Weight: 16 x 18 = 288 LB (shown "~", Assumed with its type). Type size spellings taken and normalised: 3 1/2 in, 3.5 in, 3 1/2", 3-1/2 in, 3½ in, 3 1/2 inches, 3 1/2in, 3.5in, 0.2917 ft (kept as feet), 3'1/2 (3' 0 1/2"); refused with a reason: no unit ("3 1/2", "3.5"), metric ("89 mm"), zero, negative, "3 1/2 in 3", "3 in 1/2".
- Post: height 8 LF; no height -> "stands to its level's height, 9 ft (Assumed): say how tall it is with --height" and base 0 shown Assumed; height -2 refused; base+height top above base. Round and square types.
- Pad: 3 x 3 x 18 in: plan 9 SF, form 18 SF, 0.5 CY ✓; 3 x 5.5 x 18 in: 16.5 SF, form 2(3+5.5)1.5 = 25.5 SF ✓, 0.9167 CY ✓; thickness 0 or less refused; Division 3.
- Priced: beam 16 LF x $9.50 = 152.00; hardware 2 x $20 = 40.00 (~); post $64; pad 0.5 CY x $180 = 90.00 (~), forms 18 SF x $3.25 = 58.50 (~): job cost ~$404.50 = sum; by division 3 = 148.50, 6 = 256.00 ✓; "$130.00 of it rests on something assumed" = 40 + 90 ✓. The estimate says the wall that comes down is bearing and what it carries is not in the price.
- Joists, rectangles 24x16 at 12/16/19.2/24 in: 25/19/16/13 joists, 400/304/256/208 LF (hand: 24/s + 1 ✓); 25x16 @16: 20, 320 LF, rim 50; 30x14 @19.2: 20, 280; 24.1x16 @16: 19; 16x24: same as 24x16; 20x20: 16 x 20 ft; 10x10 @24: 6. All equal my hand figures. L (demo orientation) 20 / 250 LF / rim 48; long way: 14 EA, 264 LF (by hand 7x24 + 4x12 + 24+12+12 = 264 ✓), rim 32 ✓. Triangle and a skewed rectangle give figures (not checked by hand to the foot).
- Rafters: hip 24x16 @24: 36 (11, 7, 11, 7), 71.4843 + 35.7072 LF per pair ✓; gable 26 / 232.5511 LF ✓; shed (one pitched edge, three gable): 13 rafters of 17.8885 ft = 232.5511 LF ✓ (@16: 19 = 339.8823; @12: 25 = 447.2136; @19.2: 16 = 286.2167 ✓ all by hand). "at 16" with no unit is taken as inches.
- BIM file: IFCBEAM/IFCCOLUMN/IFCFOOTING PAD_FOOTING written; Qto: beam Length 16, area 0.2778 ft², volume 4.4444 (5 x 8 in type: 40 in² ✓), column 8 / 0.2101 / 1.6806 ✓, pad 3 x 3 / 1.5 / 13.5 ✓; each with its Known/Assumed. `import ifc` says "1 beam: nothing here to bring it in as, 2 columns, 2 footings" (said). ifcopenshell validate: 0 issues.
- Clashes: a duct through a beam is a clash and on the field list; beam/post/pad vs walls/floors/each other are said never to be tested.
