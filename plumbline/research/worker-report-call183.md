# W-183 report: call 183, what the second judge found in the scan's walls (command line only)

Working copy: .../scratchpad/wtJ (only). Nothing committed or pushed. No window built or run.
Scratch: .../scratchpad/review/w183/ (table.py, one.sh, run.sh, dup.py, the frozen "before" program in before-bin/, scans/).
The judge's generator was not on disk (its folder judges/scan-walls-2 is gone): the judge's houses are rebuilt from its notes as
scenes in `tools/made-up-scan.py` (new scenes and new Scene methods only; the old flags' files are unchanged):
big, soffit, fire, mirror, neighbor, curved, closet6, dens, garage, storey, drift. `--scene list` says each and its right answer.
"Before" = the finder at the start of this work (wtJ HEAD 8c6a94c).

## Order: 1, 2, 4, 3, 5, 8, 6, 7, 9. Written as I go.

## Item 1: a face belongs to one wall (FIXED at the rule; the judge's own scan not reproducible)
Cause (as far as it can be told): nothing after the find asked whether two walls lay over one another; `twice` (a face found twice) only
looks at faces while they are found, and the pairing takes the best pair, leaving a third face (a copy of a face, 2 to 16 in off) a wall of its own that
`lay` then laid 2.4 in beside the first, where the model's own check ("lies along") does not reach.
I could not make my rebuilt big house do what the judge's did (72 runs: seeds, densities, turns, tub variants; `review/w183/sweep1.out`): the finder
absorbs a second copy of a face up to about 0.45 ft off. A second copy 0.6 ft (7 in) off (scene `drift`, --set drift=0.6) is found as a sixth wall 5 ft long.
Rule (call 183): `found_183::one_face_one_wall`, after the find, on the walls as numbered: a wall of one face on a face of a wall of two (within
BODY_SLACK 1 in) or outside it by no more than PAIR_MOST (16 in), alongside it more than SHARE_ALONG (1 ft); two walls of one face on the same side within
SAME_FACE (2 in); two of one face on opposite sides each seen from the side away from the other, 2 to 24 in apart (the two faces of one wall not paired);
two of two faces whose bodies overlap. The shorter (or the single one) is in doubt, saying which wall it lies by, how far, and along how much.
Red: unit test `a_second_copy_of_a_face_beside_a_partition_is_found_in_doubt_and_not_laid` (6 walls, none in doubt: left 6 right 5) and demo step 13 on the before
program ("not said: where it already has its faces"). Green: both pass (turned 0 and 33). Four tests on the rule itself on `Wall`s (a face on a face; a third
face; walls that only meet or lie apart are not in doubt; the two faces of one wall not paired).
Hand figure against said: 5 walls laid, 6th in doubt; its place 10' 9 1/2" (hand 129.6 in, said 129 in), 5' 0" long, 0' 7" from the partition (hand 7.2 in).
Changed: the sentence after "walls found: N ... in doubt" no longer names only the tall-at-a-wall kind (each doubt says its own reason).

## Item 2: what is found beyond a wall with no way to it (FIXED, with ONE PLACE WHERE THE CALL AND THE EXISTING STEPS PULL APART: read this)
Cause: the finder had no notion of which floor the scan was walked on; every face and thing the points showed was a house.
Rule built (`found_183::beyond`, measures named in the file): the scan's floor cells (1 ft; a cell is floor where it holds 0.3 of the middling floor cell's points, so the
3 in foot of a wall is not floor: FLOOR_LIKE) are joined into regions where no wall bars the straight way between neighbouring cells (a doorway lets it through; a window,
a not-seen stretch, a wall of one face do not). The house is the largest region; where two are alike in size (the smaller at least ALIKE 0.8 of the larger: the mirror) it is the one
whose floor lies nearest the scan's own zero, and that is said ("which side of the wall the house is on is not in the scan ... taken as the side the scan begins on").
Each wall is sampled every foot on the side it was seen from (both sides for a wall of two faces); a sample not beside the house's floor is "beyond"; where there is no floor beside it
(a neighbour's wall) it is beyond when the straight way to the nearest floor that was walked crosses a barring wall. A run of beyond samples of at least 2 ft is cut from the wall at the
wall in the way (the mirror's south and north walls, 40 ft, are cut at 20 ft); a whole wall beyond is in doubt: "in doubt: beyond wall N with no way to it: ..." with N the wall in the way
(numbered as the walls are). In doubt = not laid without `--take`. A thing beyond (the neighbour's wall) is taken out of what stood short and said ("something 7' 0" high, at 30' 0" east, 6' 0" north,
is beyond wall 4 with no way to it").
Red: the before program on the same scans (table rows J-mirror: 5 walls, 118 LF; J-neighbor: "things 1", nothing said of what is beyond); unit tests
`a_mirror_doubles_nothing_...` (turned 0 and 33), `a_neighbours_wall_seen_through_a_window_is_said_...`, `a_house_with_doorways_between_its_rooms_has_nothing_beyond`; demo step 14.
Green: all pass. Hand against said: room walked 64 LF (faces), laid centre lines 65.0 ft (hand 66 within 2: the model's corners carry the ends to meet); three walls beyond (20, 20, 12 ft) in doubt;
the neighbour's wall at 30' 0" east, 7' 0" high (hand 30 ft, 7 ft); the window 5' 2" wide (hand 5 ft), sill 3' 0", head 7' 0".
**THE PULL (do not bend either: I stopped at the literal rule and say why).** Read to the letter ("a wall with no opening leading to it"), a partition of two faces with no doorway in it
puts the whole room behind it in doubt. Every made-up house the finder was built on (the `house` scene of steps 9 to 12 and of the table's S10..S60, LONG40, T33, N05, N3, FAR, D, T18 ... about 20 rows)
has such a partition with no doorway: laid by the letter they would lose the room behind it, and steps 9 and 11 ("the walls close") would fail as written. I made `TWO_FACED_WALL_BARS_THE_WAY = false`:
a wall found with both faces (a wall with a body) is not a barrier between regions; a wall of one face (a mirror, glass, the outside seen from within) and a window are. The judge's two houses
(mirror, neighbour) are caught by this; the literal rule is one constant away. What the narrower reading leaves: a room behind a two-faced partition whose doorway was missed is NOT said to be beyond. The lead decides.
Also: in real scans a wall's far room is only seen through an opening, so floor beyond a two-faced wall usually means a missed opening, and the literal rule would flag every missed door.

## Item 4: a short wall with a door in it; every stretch left out said with where; where leaving it out opens the rooms (PARTLY FIXED)
Causes (four, found by printing the finder's own stages; the judge's "7 not straight" hid all of them):
 (a) a face that reaches the ceiling over its head but not the floor over half its length (a wall 4 ft long with a 2 ft 6 in door: its jambs are 1 ft and 6 in)
     was demoted as "reaches the ceiling and not the floor" (or, below 0.8 of the ceiling, as furniture);
 (b) the columns inside a crossing wall (a short wall's ends are all corner) counted against its reach;
 (c) **two faces of one wall found running opposite ways (theta 0.01 and 179.99) were laid over one another mirrored** (`shift` assumed the same way), so a doorway in a
     partition was filled by the other face's points and found short or not at all (wall y=20 of the big house: door 3 ft found as 1' 9"); the same mistake was in the
     "ends agree" and "overlap" tests of the pairing and in the A3 doubt;
 (d) `edge` carried a face's end feet past its last point where the solid beside the end (a jamb) is thicker than where the density was counted (a head): the 4 ft stub ran
     2 ft past the east wall.
Fixed: (a) `found.rs`, the demotion loop: a stretch of at least 1 ft 6 in with no floor reach that has a head reaching the ceiling over 0.8 of it, with floor reach at both ends
(END_PIECE 0.5 ft), counts as reached (it is an opening: the same rule makes a garage wall with a 16 ft door a wall: item 5, first half); (b) CORNER_BODY 0.35 ft columns are judged neither way;
(c) `along_span` (the faces' ends along the other's own way) replaces `shift` in the pairing and in `tall_at_a_wall`; points are moved by the exact transform; (d) `edge`: the count may not put
an edge past the farthest point counted where what is counted is more than EDGE_THICKER 1.5 times as thick (+EDGE_OVERSHOOT 0.1 ft).
(I tried capping every edge at the last point: it moved a 2 ft piece of wall below the 2 ft a wall is and broke two of call 179's tests; and clamping the line's drift in the fit: it made
the big house worse. Both taken out.)
Said: every stretch of upright points left out is said one by one, "a stretch of upright points 1' 9" long, from 3' 0" east, 26' 1" north to 2' 11" east, 27' 10" north, not taken as walls:
under 2 ft long (a wall is at least 2 ft long)" (or "not straight along the whole length ..."); shorter than 1 ft, in one sentence with their number. Where a wall's end meets nothing and a stretch left out
(or a face counted as standing short) lies within 2 ft (LEFT_NEAR) of it, the open end says so: "the stretch from A to B lies there (...): leaving it out leaves the rooms either side open to one another; if it is a wall, it is for you to say so".
Red: before program: scene `stub` found 4 walls and said nothing of the stub ("things 1"); demo step 15 failed ("walls found ... hand 5 program 4"); unit tests
a_short_wall_with_a_door_in_it_..., every_stretch_left_out_is_said_..., where_leaving_a_stretch_out_leaves_a_wall_open_... (and the compile of `Found::left`). Green: all pass.
Hand against said: stub 4' 0" long (hand 4 ft), 4 1/2" thick, doorway 2' 7" (hand 2' 6"), from 0' 11" (hand 1' 0"), head 6' 8"; the 1 ft 6 in face said as 1' 9" (3 in columns, hand 1' 6" within 4 in).
Big house: 9 walls of 10, 7 doors of 8 (was 8 walls, 6 doors).
**Still fails**: the closet's 4 ft wall with a door (big house, E-W at 9 ft 6 in) is found as a face tilted 4.8 degrees (the fit narrows its band round each turn, each taking the points of the wall that crosses it near one end)
and is dropped as standing short; what is said is the open end and the stretch that lies there ("the stretch from 41' 10" east ... lies there"), not a wall. The judge's other dropped wall (the 10 ft partition with a door) is found now (cause c).

## Item 3: a soffit; any face whose two ends meet no wall is in doubt (FIXED)
Cause: two. (a) Call 179's A3 asked only where a wall face stood behind the face and went on past an end of it, and it measured "behind" with the same orientation mistake as the pairing
(cause 4c above): the judge's soffit and fireplace were in doubt on the rebuilt houses once that was mended, but not where the wall behind was hidden or where the soffit ran from wall to wall (both ends meet a wall).
(b) A face made of base cabinets below and a soffit above, in one plane, is one line of points with a hole between: floor reached, ceiling reached, a wall by every test.
Rules built (found.rs): (1) `tall_at_a_wall` also puts in doubt any tall face with no other face beside it that it could pair with (parallel, 2 in to 2 ft, alongside 2 ft: so a wall of two
faces that stops in the room is not asked about) whose two ends meet no wall (that was seen from one side or from both: a face with floor each side and one face is the same): "a face that reaches the
ceiling and meets no wall at either end: ..."; (2) `holed`: a tall face with a stretch of height of HOLE_HIGH 2 ft or more, between points below and points above, over HOLE_SHARE 0.6 of its length,
with HOLE_BACKED 0.3 of the points a surface would show standing behind the hole within 3 ft (on the side the scan did not see it from): "the front of something at the wall with something over it, such as base cabinets
and a soffit, and not a wall" with where the hole is (3' 3" to 7' 0" up). Nothing behind the hole: a window along most of the wall, measured as a window (unit test).
Red: before program, scene `soffit2` (20 ft wall to wall): found as a 20 ft wall at 10 ft north, 3 walls "do not close"; demo step 16 (not said: "such as base cabinets and a soffit"); unit tests
(the soffit with free ends and wall to wall, the free-standing face). Green: pass; the window-along-the-wall test passes (not in doubt, measured as a window).
Hand against said: the face at 10 ft north, 20 ft long (hand 240 in, said 240), the hole 3' 3" to 7' 0" up (hand 3 ft to 7 ft: the cabinet top and the soffit's foot, to 3 in); laid 3 walls, not the face; the north ends said open (2).
The free-standing face: 6 ft 1 in (hand 6 ft), in doubt, 4 laid.
Not done: the wall behind the cabinets and soffit (12 ft north, seen only 3 to 7 ft up) is not found (a wall seen only between two things reaches neither the floor nor the ceiling): said by the open ends of the two walls that run to the face.

## Item 5: a wall that is mostly opening; a curved face (FIXED)
Mostly opening: the same opening rule as item 4 (a stretch of 1 ft 6 in or more with a head reaching the ceiling and a piece that reaches the floor at each end, END_PIECE 6 in): the garage wall with
a 16 ft door (24 ft wall, 8 ft standing) is a wall; the door is its opening (16' 0" wide, head 7' 0"; the 8 ft door 8' 1"). A beam hung across the room (nothing below it, no posts) is not (unit test).
Red: before: 3 walls, one opening, "reaches the ceiling and not the floor ... something hung from the ceiling" (the judge's wrong word), "do not close"; step 17. Green: pass (turned 0 and 33).
Curved: `curved`: the upright points that no wall not in doubt accounts for are gathered (cells of 0.3 ft) into pieces; the short walls (under 8 ft: CURVE_CHORD) that touch a piece are taken as part of it (an arc's chords are found
straight where short, and its bulge is what is left over); a piece at least 5 ft long, 0.2 ft or more off its straight line, whose circle fit is within 0.34 of the line's rms, radius 2 ft 6 in to 80 ft, arc 30 to 270 degrees, is a curved face.
Said: "a curved face of radius 7' 6", about 12' 1" long, from ... to ...: curved walls are not found, so it is not laid"; the walls found on it (ends and middle within 0.8 ft of the circle, within the arc +10 degrees) are in doubt, "part of a curved face".
Hand: radius 8 ft (said 7' 6"), length 12.57 ft (said 12' 1"). A bay of straight 45 degree walls is not a curve (unit test: nothing left over). Red: before: 3 chords found and laid (turned 33: 8 walls found, 4 of them chords),
the sentence "furniture, or two walls close together". Green: unit test (turned 0 and 33) and step 17.

## Item 8: two levels in one scan (FIXED)
Cause: one floor and one ceiling for the scan (the lowest floor and the highest top): a house of two storeys read 17 ft 3 in from floor to ceiling, "not likely the house", and with --anyway both storeys' faces were found together (a wall twice, an upper door a window with its sill at 9 ft 3 in).
Rule built: `found::levels` (the surfaces the scan has at one height, the scan's own measure: an inch holding over a third of the fullest; a level is a floor and the ceiling over it, 5 to 15 ft above; the next floor within 3 ft 6 in above that ceiling (MOST_PLATE) and its ceiling;
fewer than two complete levels is one: a two-height ceiling (8 and 10 ft) and a tray ceiling are not two levels, unit test). `find_level(scan, wanted)` cuts the points of the level asked for (halfway between one level's ceiling and the next's floor) and finds that alone; `find` is the lowest. Said: "the scan holds 2 levels:
floors at 0' 0" and 9' 3"; the level found is the one whose floor is at 9' 3", alone (find another by naming its level)". The CLI asks for the level by its floor: its elevation above the job's lowest level's. (The nearest floor is taken, and said where over 2 ft off.)
Red: before (J-storey: "not likely the house ... 17' 3"" and with --anyway duplicates); demo step 18 ("not said: the scan holds 2 levels"). Green: step 18 and unit tests `two_levels_in_one_scan_...`, `a_scan_of_one_level_is_not_said_to_hold_more_...`.
Hand against said: floors 0' 0" and 9' 3" (hand 0 and 9.25 ft); lower door 3' 1" from 10' 0" head 6' 9" (hand 3 ft from 10 ft head 6' 8"); upper door 3' 0" from 3' 11" head 6' 9", no window with a sill at 9' 3".

## Item 6: the ceiling over a wall; a slope said only where there is one (FIXED)
Cause: a cell of ceiling was the mean of the highest half foot of height that held enough points: at a closet's corners two walls' feet hold as many points to the half foot as a ceiling does (a 6 by 3 ft closet is all corner), and a mean pulled by wall points read the ceiling 5 ft 6 in
or 7 ft 3 in; the cells then spread by more than the 3 in a flat is allowed, and "the ceiling is not one height: 6' 6" and 8' 1"" (or "slopes") was said of a flat ceiling and the walls laid 15 in short.
Rule built: a cell of ceiling must be a surface: its points in the window must fill 0.8 of the nine parts of the cell (CEIL_FILL); its height is the median of the window, not the mean; a slope is said only where a plane through the cells' heights leaves under half (SLOPE_BETTER 0.5) of the scatter a flat leaves and rises
by more than 3 in; where cells vary and neither one height, two, nor a plane fit them: "the ceiling's heights vary more than ... the scan's own floor to ceiling is used for every wall". Where the ceiling is not one height and none was read within a cell of a wall: `Wall.ceiling_not_read`, laid at the middle of the house's ceiling and said, in the wall's line.
Red: before (seeds 1, 3, 4 of closet6: "the ceiling is not one height: 6' 6" and 8' 1"" / "7' 3" and 8' 0"" / "6' 1" and 6' 9" and 8' 0""); demo step 19 ("FAIL: seed 1: said of a flat ceiling"); unit test `a_small_closets_flat_ceiling_...` (seed 1). Green: all three seeds say nothing and lay 96 in.
Side effect, better: SLOPE house floor to ceiling reads 10' 1" (was 9' 11"; hand 10 ft). The sloped and two-height houses (steps 10) still say so.

## Item 7: how thick the scan is, place by place (PARTLY FIXED: axis-aligned parts only)
Cause: one thickness for the scan (the middle of the floor's squares) and measures worked from it: a scan ten times thinner in one half read as "about 17 to the square foot", the squares and counts were made for that, and only 2 of 5 walls were found (the thick half's partition, one wall).
Rule built (`found_183::zones`, `part_of`, `merge`, and `find_one`): the floor (squares of a foot with floor points and not more than 3 points standing over them: not the foot of a wall) is read in blocks of 8 ft; each block's thickness is its middle square's; where the thinnest block is under 0.4 of the thickest
at the widest step in order (ZONE_RATIO 2.5), the scan is two parts; each part is found by itself with its own measures, a point over a block with no floor of its own (a wall at the edge) going with the part it lies nearer (ZONE_EDGE 1.5 ft); what each finds is one find: a wall that runs from one part into the next (the same line within 3 in, ends within 2 ft) is one wall, with its openings;
the doubts name walls by their places in the whole. Said: "the scan is not as thick everywhere: about 99 points to the square foot from 0' 0" east, 0' 0" north to 24' 0" east, 16' 0" north, and about 11 points to the square foot from 24' 0" east ... ; each part is read by its own thickness and what each finds is joined"; a part under LEAST_DENSITY 8 is said to be too thin to tell a wall, with where, and nothing is found there.
Red: before program (J-dens: "walls 2", ceiling "slopes"); demo step 20 ("not said: the scan is not as thick everywhere"); unit test `a_scan_ten_times_thinner_in_one_part_is_read_by_parts_and_said_by_where`. Green: 5 walls, the south and the north each one wall of 40 ft, the partition 4 1/2 in, the walls close; the thinner part said to begin at 16 ft east (hand 20: blocks of 8 ft).
**Still fails**: (1) turned 33 degrees the parts' boundary (blocks of the scan's own axes) cuts the house in steps and the walls that cross it are found in several pieces, 105 ft of 122 ft of wall found (unit test asserts over 100 ft); (2) the part is said by the bounding box of its blocks, and boundary blocks of mixed thickness are put with the thinner (the thinner part is said to begin at 16 ft, the hand figure 20 ft).
Also found and fixed on the way (not asked): the zone first read the foot of every wall as thick floor and split a uniform house (the "few hundred thousand points" test doubled in time and the soffit house split at 16 ft): the floor is only a square with not more than 3 points standing over it.

## Item 9: words (NOT DONE where it pulls against the existing steps; two pieces outside what I may touch)
The raw marks in what is said of a find are asserted by existing steps as written (steps 1 to 4 and every step after: "laid on level-0002: 5 walls", "level-0002 already has 5 walls found in scan house" (that one is `crates/world/src/plans.rs`'s `lay_found`, which I may touch only in `lay_found`: it is its message), "nothing is laid: say --lay ... on level-0002"). The contract says every existing step must pass as written, and call 183 asks for names:
the two pull against each other and I did not bend either. The lead decides: if the marks move to names, the demonstration's steps 2, 3, 4 and all of mine (13 to 20) that say `laid on $L` move with them (they are one line each).
Not touched because outside my files: the tildes in the VIF list ("check: ~type, ~wall line; measure: ~height") have a legend in some verbs ("flags: ~ Assumed ? Unknown") and not in `vif` (`crates/cli/src/main.rs`, a verb I may not touch); `a thing is given by its path, like wall-0004` and "the name of level-0002: give level-0002" (main.rs:649..664).
Done in what I wrote: every new sentence was written in a builder's words (a wall, a stretch, a face, a level by its floor, "the scan holds 2 levels", "left out", "not read"); the language gate passes.

## The houses, before and after (the last builder's 46 houses and the judge's 11 rebuilt: 57 rows, and the eight new ones below)
"Before" is the program at the start of this work, "after" the program in the patch; every scan is `tools/made-up-scan.py --scene <name> [--density N] [--turned D] [--set k=v]` (1.5 cm of scatter, one stray in fifty, half a degree of lean);
`review/w183/table.py <program folder>` makes and runs each (rows in review/w183/before-table.txt, after-final-table.txt). Row: walls found [thickness or 1-sided], doors, windows, things stood short, hidden stretches said, whether the walls close, floor to ceiling, doubts.
**Every one of the last builder's 46 houses came out the same, but SLOPE** (floor to ceiling 9' 11" before, 10' 1" after: hand 10 ft; the slope is still said). No house got worse. The judge's houses that changed (11 rows in all with SLOPE), with the right answer by hand:

SLOPE
  right:  4 walls; ceiling said to slope (8 ft to 12 ft)
  before: walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 9' 11"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 10' 1"
J-big
  right:  10 walls, 8 doors, 3 windows; closes
  before: walls 8 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 0' 4 1/2", 0' 4 1/2", 0' 4 1/2", 1-sided] doors 6 win 2 things 5 hid 1 close ceil 8' 0"
  after:  walls 9 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 0' 4 1/2", 0' 4 1/2", 0' 4 1/2", 0' 3 1/2", 1-sided] doors 7 win 2 things 4 hid 1 close ceil 8' 0"
J-ew1
  right:  9 walls (no east wall), each once; the tub's wall not twice; 4 ends open at the east
  before: walls 7 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 0' 4 1/2", 0' 4 1/2", 0' 4 1/2"] doors 6 win 2 things 5 hid 1 close ceil 8' 0"
  after:  walls 7 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 0' 4 1/2", 0' 4 1/2", 0' 4 1/2"] doors 7 win 2 things 5 hid 1 close ceil 8' 0"
J-mirror
  right:  4 walls (64 LF); what is beyond the east wall in doubt, not laid
  before: walls 5 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
  after:  walls 7 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0" doubt 3
J-neighbor
  right:  4 walls, 1 window; the neighbour's wall said, beyond the east wall, not laid
  before: walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 1 things 1 hid 0 close ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 1 things 0 hid 0 close ceil 8' 0"
J-curved
  right:  4 walls; the curve said (12' 7" long), not laid
  before: walls 5 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0" doubt 1
J-closet6
  right:  4 walls 8' 0" high; no slope said
  before: walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 1"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
J-dens
  right:  5 walls (S and N 40 ft); the thin east half said by where
  before: walls 2 [1-sided, 0' 4 1/2"] doors 0 win 0 things 2 hid 0 OPEN ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
J-garage
  right:  4 walls; openings 16 ft and 8 ft, head 7 ft
  before: walls 3 [1-sided, 1-sided, 1-sided] doors 1 win 0 things 0 hid 0 OPEN ceil 9' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 2 win 0 things 0 hid 0 close ceil 9' 0"
J-storey
  right:  the scan holds two levels, floors at 0 and 9' 3"; one found at a time
  before: walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 1 win 0 things 0 hid 0 close ceil 17' 3"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 1 win 0 things 0 hid 0 close ceil 8' 0"
J-stub
  right:  5 walls, the stub 4 ft long 4 1/2 in with a doorway 2 ft 6 in
  before: walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 1 hid 0 close ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 1 win 0 things 0 hid 0 OPEN ceil 8' 0"

Unchanged: J-soffit (5 walls, 1 in doubt: already so on the rebuilt house) and J-fire (the same).

### The eight houses nobody had tried (made after the fixes; right answer by hand)
```
N-mirror2   26 x 18 ft room, a mirror on its NORTH wall (north of 18 ft). Right: 4 walls laid (88 LF); 3 beyond in doubt.
  before: walls 5 (the mirror's room found as walls), nothing said     after: walls 7, 3 in doubt (the south wall again at 36 ft, the east and west walls from 18 to 36 ft): RIGHT
N-soffit3   24 x 12 ft, two runs of base cabinets each with a soffit (8 ft fronts at 10 ft north), turned 17. Right: 4 laid, 2 fronts in doubt.
  before: walls 6, 2 in doubt                                           after: the same: RIGHT (the old rule held here)
N-hall4     36 x 24 ft, a hall 4 ft wide with four doors (3 ft), 70 per SF. Right: 6 walls, 4 doors, the walls close.
  before: walls 6, 4 doors, 1 end meets nothing                         after: the same: NOT RIGHT, not worse (one end open)
N-garage2   two-car garage 24 x 22 ft, two 8 ft doors 2 ft apart in the south wall, turned 40. Right: 4 walls, 2 openings 8 ft head 7 ft.
  before: walls 3, no doors, "reaches the ceiling and not the floor"   after: walls 4, 2 doors, the walls close: RIGHT
N-storey2   two storeys, a stairwell and a stair of ten treads. Right: two levels said (0 and 9' 3"); each found alone: 4 walls and its door.
  before: "not likely the house (17' 3")", 4 walls, no door             after: two levels said; the lower: 4 walls and its door, and the stair's top two treads (they reach the ceiling) found as two more walls, one in doubt, the walls do not close at them: NOT RIGHT (the stair)
N-dens25    40 x 14 ft, the east half at 25 per SF. Right: 5 walls (S and N 40 ft), the thin part said by where.
  before: 5 walls                                                       after: 5 walls, the parts said: RIGHT
N-stub41    4 ft partition with a 2 ft 6 in door, 60 per SF, turned 41. Right: 5 walls, the stub 4 ft 4 1/2 in, doorway 2 ft 6 in.
  before: 4 walls, nothing said                                         after: 5 walls, the door found, but the stub is found as ONE face (its other face is said to "reach the ceiling and not the floor") and one end is open: PARTLY (better, not right)
N-curved5   a quarter circle of 5 ft radius across a corner, 70 per SF, turned 15. Right: 4 walls laid; the curve said (7 ft 10 in long), not laid.
  before: walls 6 (two chords laid), "furniture, or two walls close together"    after: the same 6 walls: NOT RIGHT (the curve is not detected at 5 ft radius)
```

## Every measure I chose or changed (all named constants in `plumbline/crates/plans/src/found.rs` or `found_183.rs`, each with its reason in a comment beside it)
Item 1: SAME_FACE 2 in (= PAIR_LEAST), SHARE_ALONG 1 ft (= CORNER), BODY_SLACK 1 in.
Item 2: FLOOR_LIKE 0.3 (a floor cell holds a third of the middling cell's points), JOIN 2 cells, BESIDE 0.75 ft, NEAR_FLOOR 2 ft, ALIKE 0.8, CARVE_LEAST 2 ft (= MIN_FACE), TWO_FACED_WALL_BARS_THE_WAY false (the pull).
Item 3: HOLE_HIGH 2 ft, HOLE_SHARE 0.6, HOLE_BACKED 0.3, HOLE_TIER 0.25 ft.
Item 4/5: END_PIECE 0.5 ft, CORNER_BODY 0.35 ft, LEFT_NEAR 2 ft, EDGE_THICKER 1.5 and EDGE_OVERSHOOT 0.1 ft (in `edge`), CURVE_CELL 0.3 ft, CURVE_LEFTOVER 60, CURVE_CHORD 8 ft, CURVE_LEAST 5 ft, CURVE_OFF 0.2 ft, CURVE_FIT 0.34, CURVE_RADIUS 2 ft 6 in to 80 ft, CURVE_ARC 30 to 270 degrees, CURVE_ON 0.8 ft, CURVE_ARC_SLACK 10 degrees.
Item 6: CEIL_FILL 0.8 (of nine parts of a cell), SLOPE_BETTER 0.5. Item 7: ZONE 8 ft, ZONE_RATIO 2.5, ZONE_CELLS 24, ABOVE_MOST 3, ZONE_EDGE 1.5 ft, JOIN_OFF 3 in, JOIN_GAP 2 ft. Item 8: MOST_PLATE 3 ft 6 in, SURFACE_SHARE 3 (a third), LEVEL_NEAR 2 ft.
Changed in how existing code works: the pairing and the A3 doubt use `along_span` (the faces' ends along the other's own way) instead of `shift`; `edge` is capped where the solid beside the end is thicker; the demotion loop of faces that reach the ceiling judges columns inside crossing walls neither way and excuses openings; a cell of ceiling is a surface that fills its cell and its height is the median.
Existing tests edited: none changed in meaning (`Wall { .. }` literals in found.rs and found_179.rs got `..Wall::default()` long ago; `Wall` gained a field `ceiling_not_read`, and `Found` fields `left, stood, curves, levels, level_found, level_note, zone_note, beyond_things, beyond_note`).

## Gates, tests, demonstrations
- `cargo test -q -j2 -p plumbline-plans -p plumbline-world -p plumbline`: 564 passed, 0 failed (plans 93: 26 of them new in found_183_tests.rs; the eight-angle test passes as written; the 38 of call 179 and of the first finder pass unchanged).
  `a_scan_of_a_few_hundred_thousand_points_is_quick` (existing, asserts under 20 s) takes about 12 s alone in this debug build (6.4 s at the start of this work), 27 s in the full run on this loaded machine, and passed there; it can fail under load.
- `demos/findwalls.sh` PASSES (steps 1 to 12 untouched, 13 to 20 new, one per item; run with PLUMBLINE_BIN, nothing built), `demos/scan.sh` PASSES, `demos/planwalls.sh` PASSES untouched.
- tools/gate-language.sh, tools/gate-egress.sh, tools/gate-clientdata.sh --all: all PASS (from the working copy root).
- The old flags of tools/made-up-scan.py: eight combinations and twelve old scenes byte for byte the same as the original (cmp).
- Patch: `.../scratchpad/review/call183.patch` (`git add -N . && git diff --binary -- plumbline`), 8 files, +2471 -67. Nothing committed or pushed; no window built or run.

## Per item, in one line each
1. face belongs to one wall: FIXED at the rule; judge's own scan not reproducible (my houses did not do it): reproduced with a copy of a face 7 in off.
2. beyond a wall with no way to it: FIXED for the mirror and the neighbour; THE PULL: the literal rule would put the room behind every doorless two-faced partition in doubt and break steps 9 and 11 as written; I read "a wall with a body is not a wall with no way" (one constant to flip).
3. soffit; free-ended faces: FIXED (also fixed a bug in A3's orientation that was half the cause).
4. short wall with a door; stretches left out said; rooms open said: PARTLY (the 10 ft partition with a door and the 4 ft stub found; the closet's own 4 ft door wall in the big house is still dropped; every left-out stretch is said with where).
5. mostly opening (garage); curved said: FIXED for the garage and for a curve of 8 ft radius; NOT for a curve of 5 ft radius (N-curved5).
8. two levels: FIXED (a stair's top treads are walls in the lower level: N-storey2).
6. ceiling: FIXED. 7. thickness by place: PARTLY (axis-aligned parts; turned 33 degrees 105 of 122 ft). 9. words: NOT DONE (the pull with existing steps; two pieces outside my files).

## Where it still fails (blunt)
1. N-curved5, a curve of 5 ft radius: chords laid as walls, nothing said but "not straight": the curve is not detected there. 2. The closet's own door wall in the big house (E-W, 4 ft, door 2 ft 6 in) and the stub turned 41 degrees at 60 per SF: found as a face that leans 5 degrees (a short face's fit narrows its band round each turn and is pulled round) or as one of its two faces only, and said "reaches the ceiling and not the floor": a wrong word for it (I tried a cap on the fit's drift and a squaring of short faces: both made the big house worse and are out).
3. N-storey2: a stair's top treads, which reach the ceiling, are found as walls in the lower level (one in doubt, one a "1 ft thick wall"). 4. Item 7 turned: 105 of 122 ft; the thin part is said by 8 ft blocks. 5. Item 2's literal rule (the pull) and item 9's words (the pull) wait for the lead. 6. N-hall4: one wall end 1 ft 8 in short of the south wall (open end said, as before).
7. The judge's item-1 case (a face found twice with the wall paired) could not be reproduced on my rebuilt houses, so the cause there is not known; the rule catches it where it is found. 8. A west wall in the scene `drift` seed 4243 overruns its corner by 1 ft 2 in (a stray run: `edge` fallback to the last point), an old defect in `find_core`, not touched.
9. Debug prints were used while finding causes and are all taken out (grep PL_DEBUG finds nothing).
