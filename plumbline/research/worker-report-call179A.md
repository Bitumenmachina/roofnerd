# W-179A report: call 179 part A, the scan's walls a judge broke (command line only)

Working copy: .../scratchpad/wtD (only). Nothing committed or pushed. No window built or run. Written as I go.
My own scratch (scans, jobs, tables, the frozen "before" program): .../scratchpad/review/w179/.

How the houses are made: the judge's generator was not on disk, so I wrote one: `tools/made-up-scan.py --scene <name>`
(new flags only; the old flags' files are untouched), and the same builder in Rust for the tests
(`crates/plans/src/found_scenes.rs`). Faces and boxes, a floor and a ceiling, points uniform at --density to the
square foot of every surface, 1.5 cm of scatter, one stray in fifty, half a degree of lean. `--scene list` says what each
is and the right answer. The "before" program is the program at the start of this work (copy in w179/before).

## A7 (the wrong measure, or the wrong way up) - FIXED
Cause: nothing asked whether the floor-to-ceiling the scan reads, or the size of the scan, is a house at all.
Rule (call 179): floor to ceiling under 6 ft or over 15 ft, or a house over 150 ft: "this is not likely the house: check the
measure and which way is up", first, and nothing laid without `--lay --anyway`. Measures (named constants in found.rs):
LEAST_HIGH 6 ft, MOST_HIGH 15 ft, MOST_LONG 150 ft; the size of the scan is the middle 95 in 100 of its points so the
strays outside the house do not make it long. Where no ceiling shows and the whole scan is under 6 ft high (meters read as feet),
it says how small the whole scan is.
Red: 4 unit tests (found_179: over 15 ft, under 6 ft, scan too small, over 150 ft) failed; the demonstration step 6 failed on the
before program ("not said: this is not likely the house"). Green: the same pass.
Hand figure against what is said: --up y on the 30 by 16 by 8 ft house: floor to ceiling 16 ft (the depth), said 16' 0"; meters read
as feet: 8 ft x 0.3048 = 29.3 in, said 2' 6" (within 2 in).
Changed: `--anyway` (new flag; refused without --lay), a refused laying now exits with an error after printing what was found.

## A3 (a cabinet bank to the ceiling laid as a wall) - FIXED
Cause: a face that reaches the ceiling was a wall whatever its ends did; a bank within 6 in of the ceiling reaches it.
Rule: a face that reaches the ceiling, seen from one side, whose two ends meet no wall (no cross wall face within 1 ft of the
end), with a wall face behind it within 3 ft (BEHIND) on the side the scan did not see it from, is "a wall or something tall standing
at the wall": said, and laid only with `--take <n>` (never by --lay alone). Wall gets `doubt: Option<String>`; `found::lay_with(found, outside, take)`.
One refinement to the rule as written, and why: the wall face behind must go on past an end of the face (by more than 1 ft). Without it a
wall of two faces that stops in the room (each face has the other behind it within 3 ft) would be in doubt, against call 173's
"two faces make a wall". A doubtful face does not pair into a wall.
Red: unit test `a_bank_of_cabinets_to_the_ceiling...` failed (the bank was found, no doubt said, and `lay` laid it); the
demonstration step 7. Green: both pass, also turned 33 degrees; the free-ended two-faced wall is not in doubt; a bank of 6 ft 6 in
is a thing as before.
Hand figure: bank face at 12' 0" north, 10 ft long (east 2 to 12): said 12' 0" north, 10' 3" long (within 5 in); 6 walls found, 1 in doubt;
laid 5; with --take 4, 6.

## A2 (a wall half hidden vanishes; the room is left open unsaid) - FIXED
Cause, three parts: (1) a wall seen only above what stands before it had no evidence the finder looked at (an "upright" cell needed
points at three heights between 6 in above the floor and 6 in under the ceiling, so the 9 in of wall above a 7 ft cabinet voted for nothing);
(2) a face that reached the ceiling but not the floor over half its length was called "stood short" whatever stood in front of it (and a wall
seen only above has few points to its whole face, so the "a quarter of the usual points" rule dropped it too); (3) nothing said whether the walls
close, and a stretch of hidden wall that ran to an end of its wall was never said.
Rule (call 179): a face hidden by what stands in front is not "stood short"; the wall is found from what shows above and between; whatever is
found, it says whether the walls close, with where.
What I built (named constants in found.rs, each with its reason):
- TOP_BAND 1 ft 6 in, TOP_TIER 3 in, TOP_TIERS 3: a cell with points at 3 of 5 quarter-foot tiers in the top band is upright too.
- A face that reaches the ceiling (WALL_REACHES_CEILING 0.8, as before) is a candidate wall; its floor reach is judged column by column with
  what stands within 3 ft before it (IN_FRONT, the opening rule's own) excused; floor reach still needs 0.5 once excused. A face that reaches the
  ceiling and not the floor with nothing before it is now said instead of counted as furniture.
- SPARSE (a quarter of the usual points) keeps its old measure; a face that reaches the ceiling is also kept when what the scan shows of it is dense.
- Ends: each face's ends are measured again once all faces are known, with the points of crossing faces near the corner taken off (a corner's worth of
  the east wall had carried the north wall's end 3 ft past the corner once that wall had few points of its own).
- A hole in a wall's grid is joined only through squares with at least 3 empty neighbours (noise squares joined a hidden stretch to the wall's end).
- A hole that runs to a wall's end, or from floor to ceiling, is said "not seen ... taken as wall" when something stands before it; if nothing does,
  the old sentence, now with its true width and place (the judge's "a gap 15 in wide" was a sub-hole).
- Two parts of one wall (collinear to 0.2 ft, a gap between) with something standing before 90 % of the gap (STANDS_ALONG) are one wall; the gap is the
  "not seen" stretch. The wall behind a bank to the ceiling is this case.
- Found::open_ends: each wall end that comes within 1 ft (CORNER) of no other wall; said "the walls found do not close: N ends meet nothing" with
  wall number, end and place, or "the walls found close: every end meets another wall". Walls in doubt count neither way.
Red: the unit tests a_wall_hidden_behind_cabinets_* (30, 60, 90 per cent and turned 33), walls_hidden_behind_cabinets_on_three_sides_*,
the_walls_found_say_where_they_do_not_close, the_wall_behind_a_bank_of_cabinets_is_one_wall_* failed first; demonstration step 8. Green: all pass.
An existing demonstration figure moved while I worked (turned 60: stood short 3 became 4): the ends fix shifted the knife-edge "a quarter of the usual"
test for a sparse cabinet face. I did not move the expectation: the end measure is now taken after that test.
Hand against said: 60 % of 30 ft = 18 ft (east 12 to 30): "not seen for 18' 0" (12' 0" to 30' 0" from its west end)"; PACK2 stretches 10, 18, 18 ft:
10' 5", 17' 9", 17' 7"; no north wall: "2 ends meet nothing" at (0, 16' 1") and (30', 16').
Changed (existing behaviour): a wall hidden 50 % or more is no longer "stood short"; the old "left out: a gap N in wide ... floor to ceiling" now names where
it is and its true width; demonstration step 7 (my own, A3) moved from 6 walls found to 5 (the north wall behind the bank is one wall with the gap said).

## A1 (too few points: nothing found, "furniture", no word) - FIXED
Cause: every count in the finder (the 3 in squares a face is looked at in, the cells it is voted for in, the votes, the points a square or a
column needs, the 3 in strip that is "within 6 in of the ceiling") was tuned on scans of 100 or more points to the square foot; on a scan of 40 or
fewer the squares of a real wall held too few points to be "there", and every wall was called a cabinet.
Rule (call 179): say how thick the scan is; where too thin to tell a wall, say so and find nothing; counts that depend on thickness are worked from
the scan's own thickness.
Built: `density()` = the middle count of the 1 ft squares of floor holding two or more points within 3 in of the floor (REFERENCE_DENSITY 80, LEAST_DENSITY 8).
From it `k = max(1, sqrt(80 / density))`: the squares (3 in) and the cells (1 1/4 in) a face is looked for in, the strip that is "within 6 in of the ceiling"
and the least votes (20 / k, at least 8) are scaled by k, so each holds as many points as it did on a dense scan; at 80 and more every measure is exactly what it was.
Under 8 points to the square foot: "nothing found: the scan is too thin to tell a wall from furniture: about N points to the square foot, and it takes 8 or more"
and nothing is laid. Always said: "the scan holds about N points to the square foot of floor"; under 56 also "this scan is thinner than the measures
here were made for ... a wall is told to a few inches, an opening only to about N in, and one may be missed or split: check each".
How thin it works (made-up house, 3 seeds, 2 angles): 10, 12, 15, 20, 30, 40, 60 to the square foot: 5 walls, partition 4 1/2 in, closes, every time;
8 (estimated 7 to 8): 5 walls or too thin; 5, 6, 7: too thin. At 10 to 15 the openings are not reliable (a doorway counted twice, a window missed): that is said.
Red: the before program (table below: S10..S40 and LONG40 "0 walls, 2 things (cabinets, furniture)"); unit tests (found_179: thin scan found, too thin said, dense says
nothing of thin) and demonstration step 9. Green: pass.

## A4 (a sloped ceiling reads "no ceiling"; two heights make the low room's walls "furniture") - FIXED
Cause: one floor and one ceiling for the whole house (scan.rs's `floor_and_ceiling`, which I may not change): a slope spreads the ceiling over feet of height so none
"shows"; with two heights the higher is read and every wall that stops at the lower is "stood short".
Built: the ceiling is looked for over each cell of plan (1 ft, larger on a thin scan): the highest half foot of height that holds 0.7 of the points a flat surface would
(CEILING_SHARE), at least 5 ft up. Where the middle 90 in 100 lie within 3 in (CEIL_FLAT) it is one height and the scan's own ceiling is used exactly as before. Where
they lie in groups 5 in apart or more (CEIL_STEP) it says "the ceiling is not one height: 8' 0" and 10' 0""; otherwise "the ceiling slopes, from 8' 1" to 11' 11"". Each
face is then judged against the ceiling over it (the lower of what is 1 ft 6 in out each side), column by column, and so is the grid its openings are told from
(squares above a lower ceiling are not "empty"). Each wall found carries its ceiling height, and `lay` lays each wall that high (FoundWall.height, the level's height
is the default). What could not be done is said: a wall that runs under both heights is laid at the height over its middle (the step in its top is not drawn); under a slope
likewise (the sloped top is not drawn).
Red: the before program on the same scans: SLOPE "no ceiling shows in the scan ... nothing found", TWOCEIL 1 wall and 2 things; demonstration step 10. Green: pass.
Hand against said: slope: south wall 8 ft, north 12 ft: said 8' 2" and 11' 8" (the east and west walls 9' 5" and 10' 2" for 10: the ceiling over each side is looked
for at a cell near, up to a few inches of slope off); two heights: west wall 8' 0", east 10' 0", partition 8' 0", laid 96, 120, 96 in.
Not done (A4): a wall whose top is not level is laid level; a slope steeper than half a foot to the foot is not read (the half foot window).

## A5 (a closet and a thin wall beside a thick one: leaning and phantom walls; laying refuses all without naming a wall) - FIXED
Cause: (1) the vote for a line told its place to 0.3 ft with three bins added, so two faces nearer than that (a 3 1/2 in wall is 0.29 ft) were one peak; the line fitted
through both leaned across them and across the next (two crossing "walls" each leaning 1.4 degrees; a 7 ft phantom from the 12 in wall's points). (2) `lay` was one
all-or-nothing step for the whole find, and the model's refusal ("lies along the new wall") names nothing, since what it names was taken back.
Rule (call 179): a wall found is never laid leaning (a face is fitted along its whole length or not found); what cannot be laid is named as the find named it and the rest laid.
Built: (a) `seed_line`: a vote's line is told again from the points near it (its direction from all of them, its place from the fullest 3/4 in across it, FINE_BIN
0.025 ft): one face, the fuller; the other is the next line's. The cells a face removes from the vote reach SEED_BAND (0.2 ft), not BAND (0.15): cells at a corner are
drawn a little off their line by the wall that crosses it. (b) Straightness: a fitted face's points must keep to its line within STRAIGHT (1.5) times as closely as the best
face's keep to theirs (root mean square across); a line that does not is tried again square to a face already found (within LEAN_MOST 8 degrees), and if it does not
keep to that either it is not a face and is counted and said. (c) What it counted but did not take is said: "N stretches of upright points not taken as walls: n under 2 ft
long (a wall is at least 2 ft long), m not straight along the whole length". (d) `lay_found` lays one wall at a time; one the model will not take is named ("not laid: wall 2
of what was found: lies along wall-0004 for 30.00 ft: one wall drawn over another would be counted twice"), what was begun of it is taken back, and the rest are laid.
Red: before program, scans C and D: D 7 walls (two leaning, one 7 ft phantom), C 8 walls and 2 "things", D1's 3 1/2 in wall read 2 3/4 in; world unit test
a_wall_the_model_refuses_is_named_and_the_rest_are_laid (it cannot even compile before); demonstration step 11. Green: pass.
Hand against said: D: 6 walls (12 in and 3 1/2 in, each exactly along the house, lean 0 in over 16 ft); C: two 3 1/2 in closet walls, 6 ft (said 6' 3" and 6' 2"), neither leaning;
the 2 ft cross wall is exactly the least a wall is and is not found (said, and the walls "do not close" there). Not done: that cross wall.

## A6 (an 18 in wall is two 6 in walls) - FIXED
Cause: two faces were one wall only up to 16 in apart (PAIR_MOST).
Built: up to 24 in (PAIR_MOST_BARE) where no floor is seen between them (less than FLOOR_BETWEEN 0.1 of the floor the gap would show, not counting 3 in each side) and where
the two faces end together (within CORNER, 1 ft). The end rule is what keeps a bank of cabinets 2 ft deep before a wall (A3) from being one wall of 24 in with it: its
ends do not agree with the wall's. (A6 and A3 pull against each other on a 24 in bank if only "no floor seen" is used: both faces are 24 in apart with no floor seen.
Not stopped, because the end rule separates them on every house tried; said here so it is not hidden.)
Red: before: T18 and T22 (an 18 in and a 22 in wall): 6 walls, none paired; unit test an_18_in_wall_and_a_22_in_wall_are_one_wall_each (18 in, 22 in, 18 in turned 33)
and two_walls_20_in_apart_with_the_floor_seen_between_them_are_two_walls. Green: pass. Thickness said 1' 6" and 1' 10" within 3/4 in. Not in the demonstration (the unit tests
and the table carry it).

## A8 (a level 9 ft high, Known, gets walls 8 ft high from the scan with no word) - FIXED
Cause: nothing compared the level's height with what the scan reads.
Built: where the level's height and the scan's floor-to-ceiling differ by more than an inch (LEVEL_AGREES), it is said before anything is laid: "the level ground is 9' 0" high
(Known, tape) and the scan reads 8' 0" from floor to ceiling, 1' 0" less: the walls are laid as high as the scan reads, Assumed, from the scan, and the level's height is not
changed; measure it to tell which is right". Not said where the ceiling is not one height (the walls are laid at the ceiling over each).
Red/green: demonstration step 12 (the five walls are 96 in Assumed, the level is still 9 ft Known, source tape).

## A9 (the small ones)
- Doors an inch apart in the noise are one type (ONE_TYPE 1 in, width and head; a window, width and height): FIXED; unit test doors_an_inch_apart_in_the_noise_are_one_type
  (3 ft 0 in x 6 ft 7 in, 3 ft 0 in x 6 ft 8 in, 3 ft 1/2 in x 6 ft 7 1/2 in are one; a 2 ft 6 in door is its own). Not in the demonstration (the made-up scans read their doors alike).
- Ends trimmed to the faces they meet: a wall of two faces whose faces end differently by no more than a wall is thick ends at the near face (the shorter), not the far; unit test
  a_wall_of_two_faces_ends_at_the_near_face_of_the_wall_it_meets (a room of four 8 in walls, both faces scanned: the south wall ran from -8 in to 30 ft before). It also needed
  a stray point beyond a face's end not to be the end: ends are now taken from where points run on. FIXED for pairs; the negative feet-and-inches ("-0' 4"") is a place
  printed as it is: not changed (below).
- Builder's words: "taken as meters with z up" is now "read as meters" (and "with the file's other direction taken as up (--up y)" where it is); the grammar of one wall seen from
  one side ("is laid only as ... how thick it is"); "no wall found: no upright face of 2 ft or more runs from the floor to the ceiling" where none is; a level is given by its name
  (`--level ground`) and a wall type by its name (`--outside "Existing outside wall"`); "made for them" says the type's name, not its mark. NOT changed, because each is an existing
  demonstration's expectation I may not move: "laid on level-0002: 5 walls" (steps 2 to 4 say it), "level-0002 already has 5 walls found in scan house" (step 3), "it is not a PLY file"
  (step 3; the text is scan.rs's), "0' 4 1/2" thick" and "-0' 4"" (said.py reads them); the usage line's "a PLY point cloud". These pull against call 179's A9; the lead decides.
- A scan far from zero: with a sheet of the scan on the level (the sheet `import scan` lays) it is laid where the sheet lies, and says so; with none, a scan whose southwest corner is
  over 150 ft (FAR_FROM_ZERO) from zero is laid with that corner at zero and says so, with where it lies. FIXED; demonstration step 12 (1,234 ft east of zero: laid at -3 in, and with its sheet at 1,234 ft).


## The judge's houses and the five more, before and after (the "before" program is the program at the start; the same scans, rebuilt)
Every scan is `tools/made-up-scan.py --scene <name> [--density N] [--turned D] [--set k=v]`, 1.5 cm of scatter, one stray in fifty, half a degree of lean (scratch: review/w179/table.py
makes and runs each: `python3 table.py <program folder>`; the full rows are review/w179/before-final.txt, after-final.txt, compare.txt). Row format: walls found [thickness, or 1-sided],
doors, windows, things stood short, hidden stretches said, whether the walls close, floor to ceiling.

```
S10
  right: 5 walls (4 one-sided, partition 4 1/2 in); closes
  before: walls 0 [] doors 0 win 0 things 2 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
S20
  right: 5 walls (4 one-sided, partition 4 1/2 in), 3 doors? none; closes
  before: walls 0 [] doors 0 win 0 things 2 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
S30
  right: 5 walls (4 one-sided, partition 4 1/2 in), 3 doors? none; closes
  before: walls 0 [] doors 0 win 0 things 2 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
S40
  right: 5 walls (4 one-sided, partition 4 1/2 in), 3 doors? none; closes
  before: walls 0 [] doors 0 win 0 things 2 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
S50
  right: 5 walls (4 one-sided, partition 4 1/2 in), 3 doors? none; closes
  before: walls 4 [1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 1 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
S60
  right: 5 walls (4 one-sided, partition 4 1/2 in), 3 doors? none; closes
  before: walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
LONG40
  right: 5 walls, 60 ft house
  before: walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
L40s
  right: 5 walls, 40 ft house
  before: walls 0 [] doors 0 win 0 things 2 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
HID30
  right: 4 walls; north wall one wall, 30% of it (9.0 ft) not seen and said; closes
  before: walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 1 hid 0 - ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 1 hid 1 close ceil 8' 0"
          note: left out: 1 stretch of upright points not taken as walls: 1 under 2 ft long (a wall is at least 2 ft long)
HID45
  right: 4 walls; north wall one wall, 45% of it (13.5 ft) not seen and said; closes
  before: walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 1 hid 0 - ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 1 hid 1 close ceil 8' 0"
HID60
  right: 4 walls; north wall one wall, 60% of it (18.0 ft) not seen and said; closes
  before: walls 3 [1-sided, 1-sided, 1-sided] doors 0 win 0 things 2 hid 0 - ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 1 hid 1 close ceil 8' 0"
HID90
  right: 4 walls; north wall one wall, 90% of it (27.0 ft) not seen and said; closes
  before: walls 3 [1-sided, 1-sided, 1-sided] doors 0 win 0 things 1 hid 0 - ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 1 hid 1 close ceil 8' 0"
          note: left out: 1 stretch of upright points not taken as walls: 1 under 2 ft long (a wall is at least 2 ft long)
PACK2
  right: 4 walls (20x14); the three hidden walls found from above the cabinets or said to be unseen; closes or says where not
  before: walls 1 [1-sided] doors 0 win 0 things 4 hid 0 - ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 4 hid 3 close ceil 8' 0"
          note: left out: 2 stretches of upright points not taken as walls: 2 under 2 ft long (a wall is at least 2 ft long)
TALL
  right: 4 walls; the bank (10 ft) is not laid by --lay alone, said as a wall or something tall at the wall
  before: walls 6 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 1 close ceil 8' 0"
          note: left out: 2 stretches of upright points not taken as walls: 2 under 2 ft long (a wall is at least 2 ft long)
SLOPE
  right: 4 walls; ceiling said to slope (8 ft to 12 ft)
  before: walls 0 [] doors 0 win 0 things 0 hid 0 - ceil ?
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 9' 11"
          note: the ceiling slopes, from 8' 1" to 11' 11"; a wall under it is judged against the ceiling over it and laid at the height of the ceiling over its middle (its sloped top is 
TWOCEIL
  right: 5 walls; ceiling said to be two heights 8 and 10; each wall judged against the ceiling over it
  before: walls 1 [1-sided] doors 0 win 0 things 2 hid 0 - ceil 10' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 10' 0"
          note: the ceiling is not one height: 8' 0" and 10' 0"; each wall is judged against the ceiling over it (a wall between two rooms, against the lower), and a wall that runs under
C
  right: 7 walls: 4 outside + closet 3 (two 3 1/2 in, one cross wall 2 ft); no leaning
  before: walls 8 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 6 [1-sided, 0' 3 1/2", 0' 3 1/2", 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 OPEN ceil 8' 0"
          note: left out: 2 stretches of upright points not taken as walls: 2 under 2 ft long (a wall is at least 2 ft long)
D
  right: 6 walls: 12 in and 3 1/2 in partitions, no phantom
  before: walls 7 [1-sided, 1-sided, 1-sided, 1' 0", 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 6 [1-sided, 1-sided, 1-sided, 1' 0", 0' 3 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
D1
  right: 5 walls
  before: walls 5 [1-sided, 1-sided, 1-sided, 0' 2 3/4", 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 3 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
D2
  right: 6 walls
  before: walls 6 [1-sided, 1-sided, 1-sided, 1' 0", 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 6 [1-sided, 1-sided, 1-sided, 1' 0", 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
T18
  right: 5 walls, 1 of them 1 ft 6 in thick (one wall, 15 ft 6 in long)
  before: walls 6 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 1' 6", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
T22
  right: 5 walls, 1 of them 1 ft 10 in thick
  before: walls 6 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 1' 10", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
UPY
  right: not likely the house: check the measure and which way is up; nothing laid without --anyway
  before: walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 - ceil 16' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 16' 0"
          note: this is not likely the house: check the measure and which way is up (floor to ceiling reads 16' 0", over 15' 0")
UFT
  right: not likely the house (scan 2 ft 5 in high as feet): said; nothing found/laid
  before: walls 0 [] doors 0 win 0 things 0 hid 0 - ceil ?
  after:  walls 0 [] doors 0 win 0 things 0 hid 0 - ceil ?
          note: this is not likely the house: check the measure and which way is up (the whole scan is only 2' 6" high, and 9' 2" by 4' 11" across; a room is at least 6' 0" high)
          note: left out: no ceiling shows in the scan: without one, a wall cannot be told from a tall cabinet standing in the room, so no wall is taken
L
  right: 6 walls, all one-sided; closes
  before: walls 6 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 6 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
A3rooms
  right: 6 walls (4 outside + 2 partitions), 2 doorways
  before: walls 6 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 0' 4 1/2", 1-sided] doors 2 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 6 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 0' 4 1/2", 1-sided] doors 2 win 0 things 0 hid 0 close ceil 8' 0"
E
  right: 4 walls, 2 doors (2'4", 3'0"), 2 windows (5'0" sill 1 head 5, 2'0" sill 2 head 7)
  before: walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 2 win 2 things 0 hid 0 - ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 2 win 2 things 0 hid 0 close ceil 8' 0"
FURN
  right: 4 walls, chair 6.5 ft not a wall
  before: walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 4 hid 0 - ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 4 hid 0 close ceil 8' 0"
          note: left out: 2 stretches of upright points not taken as walls: 2 not straight along the whole length (furniture, or two walls close together)
EMPTY
  right: 4 walls
  before: walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
N05
  right: 5 walls
  before: walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
N3
  right: 5 walls
  before: walls 6 [1-sided, 1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
T33
  right: 5 walls
  before: walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
T60d
  right: 5 walls
  before: walls 5 [1-sided, 1-sided, 0' 4 1/2", 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 0' 4 1/2", 1-sided, 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
FAR
  right: 5 walls; laid where its sheet lies, or at its own southwest corner, said
  before: walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 - ceil 8' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided] doors 0 win 0 things 0 hid 0 close ceil 8' 0"
N1cabin
  right: 6 walls, 9 ft high; doorway 3 ft; window 3 ft sill 3 head 7; closet 3 1/2 in walls
  before: walls 5 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 1 win 0 things 1 hid 0 - ceil 9' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 1 win 1 things 1 hid 0 OPEN ceil 9' 0"
          note: left out: a face 3' 10" long at 9' 11" east, 15' 8" north reaches the ceiling and not the floor, and nothing stands before it: it is not taken as a wall (something hung f
          note: left out: a face 3' 11" long at 10' 0" east, 15' 6" north reaches the ceiling and not the floor, and nothing stands before it: it is not taken as a wall (something hung f
N2masonry
  right: 6 walls, 8 ft 6 in; 10 in and 4 1/2 in partitions; 2 doorways
  before: walls 2 [1-sided, 1-sided] doors 0 win 0 things 5 hid 0 - ceil 8' 6"
  after:  walls 6 [1-sided, 1-sided, 0' 10", 0' 4 1/2", 1-sided, 1-sided] doors 2 win 0 things 3 hid 0 close ceil 8' 6"
          note: left out: 5 stretches of upright points not taken as walls: 5 under 2 ft long (a wall is at least 2 ft long)
N3bank
  right: 5 walls, 8 ft 6 in; 1 doorway; the cabinet bank to 8 ft 3 in is not a wall
  before: walls 5 [1-sided, 1-sided, 0' 4 1/2", 1-sided, 1-sided] doors 1 win 0 things 2 hid 0 - ceil 8' 6"
  after:  walls 6 [1-sided, 1-sided, 0' 4 1/2", 1-sided, 1-sided, 1-sided] doors 1 win 0 things 1 hid 1 close ceil 8' 6"
          note: left out: 2 stretches of upright points not taken as walls: 2 under 2 ft long (a wall is at least 2 ft long)
N4rooms
  right: 6 walls (4 outside + 2 partitions), 2 doorways (at 25 per SF)
  before: walls 0 [] doors 0 win 0 things 3 hid 0 - ceil 8' 0"
  after:  walls 6 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 0' 4 1/2", 1-sided] doors 2 win 0 things 0 hid 0 close ceil 8' 0"
N5hid
  right: 4 walls; north wall 65% hidden said; closes
  before: walls 3 [1-sided, 1-sided, 1-sided] doors 0 win 0 things 2 hid 0 - ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 1 hid 1 close ceil 8' 0"
          note: left out: 1 stretch of upright points not taken as walls: 1 under 2 ft long (a wall is at least 2 ft long)
N6rooms
  right: 6 walls (4 outside + 2 partitions), 2 doorways, at 14 per SF (thin)
  before: walls 0 [] doors 0 win 0 things 3 hid 0 - ceil 8' 0"
  after:  walls 6 [1-sided, 1-sided, 0' 4 1/2", 0' 4 1/2", 1-sided, 1-sided] doors 2 win 0 things 0 hid 0 close ceil 8' 0"
N7masonry
  right: 6 walls, 8 ft 6 in; 10 in and 4 1/2 in partitions; 2 doorways
  before: walls 6 [1-sided, 1-sided, 1-sided, 0' 10", 0' 4 1/2", 1-sided] doors 2 win 0 things 2 hid 0 - ceil 8' 6"
  after:  walls 6 [1-sided, 1-sided, 1-sided, 0' 10", 0' 4 1/2", 1-sided] doors 2 win 0 things 2 hid 0 close ceil 8' 6"
          note: left out: 3 stretches of upright points not taken as walls: 2 under 2 ft long (a wall is at least 2 ft long), 1 not straight along the whole length (furniture, or two wal
N8hid
  right: 4 walls; north wall 75% hidden (22.5 ft) said; closes
  before: walls 3 [1-sided, 1-sided, 1-sided] doors 0 win 0 things 2 hid 0 - ceil 8' 0"
  after:  walls 4 [1-sided, 1-sided, 1-sided, 1-sided] doors 0 win 0 things 1 hid 1 close ceil 8' 0"
          note: left out: 1 stretch of upright points not taken as walls: 1 under 2 ft long (a wall is at least 2 ft long)
N9bank
  right: 5 walls + the bank in doubt (a wall behind it 8 ft not seen), 1 doorway
  before: walls 0 [] doors 0 win 0 things 5 hid 0 - ceil 8' 6"
  after:  walls 6 [1-sided, 1-sided, 1-sided, 0' 4 1/2", 1-sided, 1-sided] doors 1 win 0 things 2 hid 1 close ceil 8' 6"
          note: left out: 3 stretches of upright points not taken as walls: 3 under 2 ft long (a wall is at least 2 ft long)
N10cabin
  right: 6 walls, 9 ft high; doorway 3 ft; window 3 ft sill 3 head 7; closet 3 1/2 in walls
  before: walls 6 [1-sided, 1-sided, 1-sided, 1-sided, 1-sided, 1-sided] doors 1 win 1 things 1 hid 0 - ceil 9' 0"
  after:  walls 5 [1-sided, 1-sided, 1-sided, 0' 3 1/2", 1-sided] doors 1 win 1 things 0 hid 0 close ceil 9' 0"
          note: left out: 1 stretch of upright points not taken as walls: 1 not straight along the whole length (furniture, or two walls close together)
          note: left out: a face 4' 2" long at 6' 5" east, 17' 1" north reaches the ceiling and not the floor, and nothing stands before it: it is not taken as a wall (something hung fro
```

## Every measure I chose or changed (found.rs constants unless said; each has its reason in a comment beside it)
Added: LEAST_HIGH 6 ft, MOST_HIGH 15 ft, MOST_LONG 150 ft (A7); BEHIND 3 ft (A3); TOP_BAND 1.5 ft, TOP_TIER 0.25 ft, TOP_TIERS 3, SOME_REACHES_CEILING 0.3, STANDS_ALONG 0.9 (A2);
REFERENCE_DENSITY 80, LEAST_DENSITY 8, THIN_NOTE_BELOW 0.7, MOST_THIN_VOTES 8 (A1); CEIL_CELL 1 ft, LEAST_CEILING 5 ft, CEILING_SHARE 0.7, CEIL_FLAT 0.25 ft, CEIL_STEP 0.4 ft, CEIL_GROUP 0.05,
CEIL_OUT 1.5 ft (A4); FINE_BIN 0.025 ft, STRAIGHT 1.5, LEAN_MOST 8 degrees, SHORT_FACE 1 ft (A5); PAIR_MOST_BARE 24 in, FLOOR_BETWEEN 0.1, FLOOR_MARGIN 0.25 ft (A6); LEVEL_AGREES 1 in and
FAR_FROM_ZERO 150 ft (cli found.rs), ONE_TYPE 1 in (A9).
Changed in how existing ones are used (a dense scan of 80 or more to the square foot is exactly as before): the grid square, the vote cell, the top strip and the least votes scale with the scan's
thickness (A1); the face's removal of votes reaches SEED_BAND not BAND (A5); a face's ends are measured again after all faces are found with crossing faces' points taken off, and from where points run
on (A2); a hole is joined through squares with 3 empty neighbours (A2); SPARSE keeps its measure and adds one for faces that reach the ceiling (A2). Not changed: scan.rs, the gates, any other
demonstration or test. Existing tests edited mechanically only: two `Wall { .. }` literals in found.rs tests (`..Wall::default()`) and two `FoundWall` literals in world's found_tests (`height: None`).

## Result
- `demos/findwalls.sh` PASSES (steps 1 to 5 as written, then 6 to 12 new; run with PLUMBLINE_BIN, nothing built by it), `demos/scan.sh` PASSES, the eight-angle test passes as written.
- Tests: `cargo test -q -j2 -p plumbline-plans -p plumbline-world -p plumbline`: 468 passed, 0 failed (plans found: 37, of which 23 are new in found_179.rs; world: 1 new, in found_tests).
- Gates from the working copy root: tools/gate-language.sh, tools/gate-egress.sh, tools/gate-clientdata.sh --all: all PASS.
- Old flags of tools/made-up-scan.py: eight combinations byte for byte the same as the original (checked by sha256 against HEAD's script).
- Patch: .../scratchpad/review/call179A.patch (git add -N . && git diff --binary), 10 files, +2505 -123. Nothing committed or pushed; no window built or run.

## Where it still fails (blunt)
1. N1cabin / N10cabin: a closet wall 4 ft long with a 2 ft 8 in doorway in it is not found (its floor reach is under half; it reaches the ceiling by the header): said ("reaches the ceiling and
   not the floor, and nothing stands before it: not taken as a wall") and the walls do not close there. Right answer 6 walls, found 5.
2. The 2 ft closet cross wall is exactly the least a wall is: not found, said, the walls "do not close" at the closet.
3. Openings on a thin scan (10 to 25 to the square foot) are not reliable: a doorway counted twice or missed; said ("an opening only to about N in, and one may be missed or split: check each").
   Walls are right to the inch-ish down to 10 to the square foot.
4. A wall under a sloped ceiling: laid level at the ceiling's height over its middle, and the east and west walls of the sloped room read 9' 5" and 10' 2" for a true 10 ft (the ceiling over each
   side is read from the nearest cell, which can be a cell along the slope). A slope steeper than half a foot to the foot is not read.
5. A wall that runs under two ceiling heights is laid at one (the height over its middle, a tie falls either way): the step in its top is said not drawn.
6. LEAST_DENSITY 8 is where the thinnest scan tried (made-up, three seeds, two angles) still found every wall, and at an estimate of 8 it is flaky (one of six runs said "too thin"). Real scans
   are not uniform: the density is the floor's; a scan whose walls are sparser than its floor is read as if they were as thick.
7. `a_scan_of_a_few_hundred_thousand_points_is_quick` (existing, asserts under 20 s) takes 6.4 s alone in a debug build and 24 s when 30 other tests run at once on this loaded machine: it can
   fail under load; I did not touch it. It passed in the full run above.
8. Not done because each is an existing demonstration's expectation (see A9): the wording "laid on level-0002", "level-0002 already has ...", "it is not a PLY file", "0' 4 1/2" thick", "-0' 4"" and the usage
   line's "PLY point cloud". They pull against A9; the lead decides which moves.
9. Red proof: the before program is not available to run the unit tests on (no stash allowed), so each unit test's red is shown by the before program on the same scan (rows above) and by the
   demonstration steps run against it (step 6 first failed on the before program, "not said: this is not likely the house"); the tests of A3, A2, A5 (world) failed first when written.
10. Disk: this machine's disk filled three times while I worked (other builders); I deleted test binaries in my own build folder and rebuilt. The integration-test binaries of plumbline-world are
    gone from my build folder (rebuilt by the full test run).
