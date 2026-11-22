# W-173 report: walls found in a scan (command line only)

Working copy: .../scratchpad/wt173 (only). Patch: .../scratchpad/review/call173.patch (git add -N . && git diff --binary).
Nothing committed or pushed. No window built or run.

## Result
- demos/findwalls.sh PASSES (NOT demos/found.sh: that name is an existing demonstration, "the foundation drawn". My first
  write overwrote it by mistake; I put it back byte for byte from `git show HEAD:...` and it passes untouched; mine is
  demos/findwalls.sh, listed in plumbline/README.md). The lead may rename.
- cargo tests (the contract's six packages): 511 passed, 0 failed (12 new in plans `found::tests`, 3 new in world `found_tests`).
- Command-line demonstrations (36 scripts that only run `plumbline`): all pass, including demos/scan.sh (untouched) and demos/found.sh;
  demos/field-job.sh "fails" with `$1: unbound variable` only because it is a helper that takes a job path, not a demonstration.
- Gates: gate-language, gate-egress, gate-clientdata --all PASS; tools/licenses.sh exit 0; BIMCRAFT=... tools/portmap.sh PASS
  (no new public item in building*/editing*, so no PORTMAP.md entry; SPEC.md unchanged: walls laid are ordinary walls).
- Made-up scan, old flags: plain, --binary, --rough, --rough --binary sha256 identical before and after (saved in
  scratchpad/w173/before.sha). Gate proven red: a copy of the demo with 2 hand figures altered fails on exactly those two.

## How the finder works (crates/plans/src/found.rs)
1. Points from 3 in above the floor to 3 in below the ceiling (scan.rs's own floor/ceiling, after its levelling).
2. 0.1 ft cells of plan with points at 3+ heights (6 in apart) are "upright". They vote for lines (Hough, 0.5 deg x 0.1 ft); strongest first.
   Each vote's line is cut into stretches (a gap of 1 ft 6 in parts them), each stretch is fitted by itself (so a line across a sofa
   and a cabinet does not lean across the room), points of faces already found that this line crosses are taken off it (corner pollution).
3. A stretch of 2 ft or more is a face; its place is measured at the floor from a fit of offset against height (a leaning scan does not move it).
4. Wall face = points within 6 in of the ceiling over 80 % of its length and within 6 in of the floor over 50 %. Else it stood short; its
   height is where its points per 0.1 ft fall to half the usual. Stood-short faces whose ends are within 1 ft are one thing (counted).
5. Faces within 1 deg of the main direction (length-weighted, quarter turns) are made square; counted. A wall face's end that comes within
   0.5 ft of another wall's face line is carried to it (so a partition is 20 ft face to face).
6. Which side the scan saw each face from = which side shows floor (1 ft cells, nearest foot ignored; a third as much as the other side).
   Two parallel wall faces 2-16 in apart, side by side 2 ft or more, are one wall (centre between, thickness = distance). A lone face is "seen from one side": no thickness.
7. Per wall, a grid of 3 in squares along and up; empty squares joined; a clean block 1 ft 6 in+ wide, not at the ends, is a hole; its edges are
   measured by counting (points between a place well inside the solid and the hollow, at the solid's own density) not by nearest point, so
   scatter does not shrink it. Reaches floor = doorway; else window. Something that stands short within 3 ft on the scanned side (both sides
   for a two-faced wall) makes it hidden, not open.
8. Laying (`found::lay`, then `world::plans::lay_found`): one-sided walls only with `--outside <wall type>`: centre half a thickness outside
   the seen face, drawn with the room on its right; two-faced wall = measured type (reused if the job has one of that thickness, else made
   "wall 4.5 in"); ends within 1 ft of another's centre line carried to it; doors/windows via editing::opening_type_for / opening (no second
   way to make a wall). All Assumed, source "found in scan <name>". A second laying on the same level from the same scan is refused (walls
   with that source already on the level), job untouched. Height = scan's floor-to-ceiling to the inch.

## The verb (the call left the name to me)
`plumbline find-walls <file> <scan.ply> --level <level> [--units m|ft] [--up z|y] [--lay [--outside "<wall type>"]]`
Without --lay it says everything and writes nothing. Says: each wall (from/to east, north; length; "N thick, seen from both sides" or "seen from
one side (its thickness is not in the scan)"), each doorway/window (along it from its west or south end, width, sill, head), each stretch "not seen for ...
behind what stands at it: taken as wall, to be checked", squared "N of M wall faces ...", things stood short and the tallest, what is laid/not laid and why,
and that a doorway cannot be told from a door. In usage text.

## Hand figures against the program (demos/findwalls.sh prints each)
| What | Hand | Program | Near |
|---|---|---|---|
| walls found / one-sided | 5 / 4 | 5 / 4 | exact |
| outside faces east 0, 30; north 0, 20 | | 0, 30, 0, 20 ft | 1 in |
| partition centre east | 18'0" | 18'0" | 1 in |
| partition thickness | 4 1/2" | 4 1/2" | 1/2 in |
| partition length | 20'0" | 20'0" | 2 in |
| doorway | 9'0"-12'0", head 6'8" | 9'0"-12'0", head 6'8" | 2 in |
| window south wall | 8'-13', sill 3', head 7' | 7'11"-12'11", sill 3'1", head 7'0" | 2 in |
| stood short / tallest | 3 / 7'0" | 3 / 7'1" | exact / 2 in |
| hidden | 9'0" (20-29 ft) | 8'10" (20'1"-28'10") | 4 in |
| squared | 6 of 6 | 6 of 6 | exact |
| laid: 5 walls Assumed, source scan, none Known; partition type 4 1/2" | | 5, yes, 4.5 in | 1/2 in |
| door 3'0" x 6'8"; window 5' x 4', sill 3' | | 37" x 80"; 60" x 47", sill 37" | 2 in |
| rooms West / East | 356.25 / 236.25 SF | 356.17 / 236.21 | 4 SF |
| house over the outside | 31'0" x 21'0" | 31.00 x 21.00 ft (371.98, 251.98 in) | 2 in |
| second laying refused, job byte for byte same | | yes | |
| no outside wall named: partition + door only, says 4 not laid | | yes | |
| no ceiling: nothing found, says why | | yes | |
| turned 17: 5 walls/1 door/1 window/3 things/9 ft hidden; rooms | 356.25/236.25 | same; 356.13/236.19; south wall 16.997 deg | 4 SF, 0.5 deg |
Rooms are placed by a point (`room` verb) in the demo, as other demos do: the lay verb does not make Room elements.

## Measures I chose (call 173 gave none) - all named constants in found.rs with a comment
CELL 0.1 ft; CELL_POINTS 4; UPRIGHT_HEIGHTS 3 (6 in tiers); VOTE_DEGREES 0.5; VOTE_FEET 0.1 (3 bins added); MIN_VOTES 20; MOST_LINES 120;
BAND 0.15 ft (3 sigma of 1.5 cm); SEED_BAND 0.2; SQUARE (grid) 0.25 ft; COLUMN_POINTS 3; SET_ASIDE 0.25 (3 in off floor and ceiling);
WALL_REACHES_CEILING 0.8; WALL_REACHES_FLOOR 0.5; a square is "there" at max(2, quarter of the median occupied square);
CROSSES sin 10 deg; SPARSE 4 (a face under a quarter of the median points per sq ft is not one); face must have 3 tiers of >=10 points;
SNAP 0.5 ft; NOT_PARALLEL sine 0.5; EDGE_BACK 0.4, EDGE_LOOK 0.5, NEAR_END 0.3 (edge counting); MIN_OPENING_SLACK 0.04 ft;
IN_FRONT_LEAST 0.2 ft; THING_JOIN 1 ft; FLOOR_CELL 1 ft, SEEN_FLOOR_CELLS 5, SEEN_SHARE 3, SEEN_REACH 6 ft; face height rule "half the usual
points per 0.1 ft, two tenths in a row".

## Where it would fail on a real scan (blunt)
- It has only ever seen made-up scans from one generator (mine and the lead's): density 100+ points/sq ft, 1.5 cm gaussian scatter, uniform strays.
  Thresholds that count points (CELL_POINTS 4, 3 per column, 20 votes, edge counting needs 5+ points in 0.2-0.5 ft) will be wrong for sparse or
  very non-uniform scans; the demonstration passing says nothing about a real phone scan. No real scan was available.
- Slanted or vaulted ceilings: floor_and_ceiling() picks one flat ceiling height; walls under a slope fail the 6 in test and are called "stood short".
  Walls with a soffit or beam at the top likewise. A tall cabinet within 6 in of the ceiling is a wall.
- Curved walls: no (straight lines only); they come out as short straight bits or nothing. Faces under 2 ft are dropped without being counted.
- A wall scanned in part: found as the part; a wall hidden over about half its length (floor reach under 50 %) is called stood short; a gap from
  floor to ceiling parts a wall in two (or is said and laid across if inside a face). Wall ends not meeting another wall are as seen.
- Mirrors and glass: glass does not scan so a window reads as a hole (good), but a mirror/reflection or points seen through a window put wall-like
  points outside (strays in the face band could fill a hole), and glass doors read as doorways. Windows whose sill is under 6 in read as doorways.
- Stairs: stair stringers/walls are upright faces of varying height; a stair wall to the ceiling is a wall, the rest "stood short". A second floor
  in the scan is not understood at all (only one floor and one ceiling).
- Multi-room scans with different ceiling heights or a lean the floor fit cannot take out (scan.rs levelling is shear by the floor).
- Two real walls 2-16 in apart side by side (a chase, a closet) are paired as one thick wall; a thick wall (> 16 in, masonry) is two one-sided walls.
- A one-sided wall is laid only where the person names a type; interior one-sided partitions (scanned from one side only) are indistinguishable
  from outside walls and get the --outside type too.
- Edge measuring is by counting from a stretch 0.4 ft inside; very narrow solid between two openings (< 0.8 ft) falls back to the grid edge (+/- 3 in).
- Hidden test is all-or-nothing for a stretch (anything short within 3 ft in front along the stretch hides all of it).

## What is not right / known loose ends
- Found stretch measures read about 1 in small on the window (lean moves points 0.5 in at mid-height), 2 in on the hidden stretch.
- The doorway width reads 3'1" (37 in door type) against 3'0": inside tolerance, not exact.
- No Room elements are laid (only walls, doors, windows): rooms come from the existing room finding when a person or demo places a point.
- `lay` leaves the wall type "type-0003" etc. as paths in `--outside` (the program's usual way: type paths, not names).
- FOUND_DEBUG prints were removed; no env switches remain.
- found.rs is long (about 1200 lines with tests); the finder's `find` function is one long function.
- plans tests: the speed test asserts < 20 s (debug build, 100k+ points; 273k points runs in about 0.7 s in the debug CLI).

## After the lead's 33 degree case
Cause (mine, in found.rs; not scan.rs, not the generator): classify() took the *last* row of the 3 in grid as "within 6 in of the ceiling".
How many rows fit under the ceiling depends on the inch the floor and ceiling are read to; the 33 degree scan (turn plus lean, levelled) reads
a hair over 8 ft, the grid got a 31st row above the points, which is empty, so no face reached the ceiling and every wall was "stood short"
(tallest 7'10"). The generator's --turned and scan.rs are fine (other angles unaffected).
Fix: Grid rows are whole squares under the ceiling (floor, not ceil); "reaches the ceiling" is any row whose top is above height - 6 in, not
"the last row". Nothing in scan.rs changed.
Eight angles, command line and unit test: 0, 5, 17, 33, 45, 60, 75, 88 all PASS (5 walls, 1 doorway, 1 window, 3 things stood short, partition
4 1/2 in within 1/2 in, outside faces within 1 in, partition 20 ft within 2 in). New test `the_same_house_is_found_the_same_at_any_angle`
(generates the house with scatter, strays, lean, levelled; fails with the old code even at 0). demos/findwalls.sh step 5: --turned 33 and 60
through the command line: counts and rooms 356.13/236.18 (33), 356.13/236.19 (60) SF vs 356.25/236.25 within 4. Six-package tests: 512 passed, 0 failed.
findwalls.sh, scan.sh, found.sh, the three gates pass. Patch rewritten. Note: a stray `cargo build` from another agent filled the disk in my target
folder mid-task; I cleaned bevy build output in my own target (cargo clean -p bevy_*, deleted target/debug/incremental).
