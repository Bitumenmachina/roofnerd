# W-188 report: call 188 items 1 to 5 (the scan's walls on real scans), command line only

Working copy: .../scratchpad/wtN. Nothing committed or pushed. No window built or run. Written as I go.
My scratch (frozen "before" program, scans, tables, measures): wtN/.scratch (outside `plumbline/`, so not in the patch).

## A. What the three real scans are, measured before anything was changed

The real scans are not "a closed room with four clear walls". Read as pictures (rotated square, 3 colour bands of height) they are:
- small (a47331606, 196,676 vertices, 3.6 x 4.8 m box, floor 8 ft 1 in below ceiling in part, a lowered 6 ft 11 in ceiling over part): a cramped room
  full of furniture, notched, about 8 wall pieces of 2 to 9 ft, only the S wall (about 7 ft) and a W wall (about 6 ft) clean; the bounding box includes
  a doorway's view out and a diagonal ghost streak (SW). There is NO closed four-wall room in it.
- long (a41069021, 1.0 million vertices): a long room about 13 x 24 ft (W wall 12.9 ft, E wall 11.5 ft), an alcove at its south end, a bumpy north
  wall (bumps 6 in deep), about 8 wall pieces plus the ghosts of a second room beyond a doorway.
- large (a41048190, 818 thousand vertices): a notched room about 17 x 21 ft: W side 11.4 ft (but its points stop 1.9 m up, 2.4 m is the
  ceiling: a wardrobe, not a wall?), a 10 ft wall, steps, about 10 to 12 wall pieces, and far beyond a doorway the faint outline of another room.

Measures (python/numpy on the PLY, my own tool, read only; walls picked by hand from the pictures and measured in 0.5 m windows along them):

| measure | small | long | large | what the finder's constants assume |
|---|---|---|---|---|
| points to the sq ft of floor (the finder's own) | 420 | 625 | 500 | made-up 40 to 150; REFERENCE 80 |
| floor spread about its plane, 1.4826 x MAD (within 8 cm of it) | 1.0 cm | 2.5 cm | 2.65 cm | made-up 1.5 cm |
| ceiling spread, the same | 1.0 cm | 2.7 cm | 1.8 cm | 1.5 cm |
| a wall's spread about its own plane in 0.5 m windows (MAD sigma / rms) | 1.7 / 2.4-3.4 cm | 3.1 / 3.2-4.8 cm | 2.0-2.6 / 2.4-2.7 cm | 1.5 cm gaussian, BAND 0.15 ft = 4.6 cm |
| lean of a wall (offset change over its height; per 2.4 m) | 2.5-6 cm (0.6-1.4 deg) | 2-6 cm | 9 cm (2.2 deg) | half a degree; the old plan band ignored lean |
| bow of a wall over its length (middle against chord) | 0.9-14 cm over 2-3 m | 1.3-15 cm over 3-4 m; the S end wall 28 cm | 2.4 cm over 3.5 m | none |
| layers (passes) a wall shows in a 0.5 m window | 1.5-2.3, to 4 | 1.4-1.8 clean, 4.5-5 rough, to 11 | 1.8, to 4 | 1 |
| spacing of the layers | 1.4-2.0 cm median | 1.6-4.4 cm | 1.6 cm | none |
| points per sq ft of a wall face, against the floor's | 0.9 to 1.3 x (finder's face count: 370 to 4750) | 0.9 to 1.6 x | 0.9 x (finder: 93 to 6800) | one density everywhere; a face under 1/4 of the median face's is "not a face" |

### Which of the finder's tests a real wall fails, and by how much (the cause)
Run with PL_DEBUG instrumentation on the small scan (before any change): of 3 faces fitted, none passed. Constant by constant:
1. **The plan band** (BAND 0.15 ft, 4.6 cm, fixed, vertical): a real wall leans up to 2.2 degrees (9 cm over its height), so the band, centred at
   mid height, cuts the top and the foot off: the ceiling reach of a real face was 0.49 to 0.60 against the 0.80 a wall needs, the floor reach 0.08 to 0.15
   against 0.50. Failed by up to 4 times on the floor reach.
2. **"Not straight"** (rms across the line over 1.5 x the best face's): the faces' rms are clipped by the band (so of the 3 faces found, 0.036 to 0.084 ft:
   the same line leaning or bowed reads 2 times worse than a clean one), and the best face (clean, 0.036) sets the bar: 21 stretches of the small scan are "not straight"
   at 0.07 to 0.09 ft, 2 times the bar. Some are real walls bowed or in passes; most are ghost streaks (a wall seen through a doorway, a mirrored smear): there is no way to tell them apart by this test.
3. **The sparse-face test** (a face under a quarter of the *median* face's points to the sq ft is "not a face"): real faces differ in density 10 to 50 times (a wall scanned close
   and square on 4750 to the sq ft, the same room's other wall 370, a quarter of the median is 520): the S wall of the small scan, 9 ft long, clean, was thrown out by it.
   The floor's own density (420) is the measure of what a surface shows, and the wall had 0.9 of it.
4. **One band for everything**: a scan with a floor spread of 2.5 to 2.65 cm (long, large) has walls 3 to 4.8 cm rough: 1.3 to 2 times the made-up scatter.
5. **Reaches ceiling and not floor / stops short**: a real wall's foot is often not seen (low furniture that no face was found for stands before it) and a wall's top may
   be a wardrobe's: those are taken as things and never offered. They were said only as "reaches the ceiling and not the floor"; nothing offered them by number.
6. **Wall found 2 to 4 times** (11 for 4 in the long room): faces of one wall scanned in passes 1 to 4 cm apart are each a face, and each pairs or stands alone.
7. **Time**: each stretch gathered and re-filtered all 0.8 to 1.0 million points; 40 to 75 s.
8. **Which way is up**: not checked at all except by the floor to ceiling reading 6 to 15 ft.

## What was built, per item (call 188, items 1 to 5). Cause in two sentences, red and green, the hand figure against what is said

How the checks were run: the three real scans were run read only from `.../scratchpad/real/ply/ark/` (never copied into the working copy; no test or demonstration reads them).
The finder before my work is frozen at `wtN/.scratch/before/plumbline`; the table of 65 made-up houses is `wtN/.scratch/tab` (the earlier builders' scans and rows, `table.py`, rerun).
HONEST NOTE ON "TESTS FIRST": I wrote the finder changes while learning what the real scans did, then wrote the unit tests (10 new, `found_188_tests.rs`) and the four demonstration
steps (21 to 24). The demonstration steps were proven red against the frozen before program (below). The unit tests were not run red against old code: they read fields that
did not exist (`Wall::bow`, `joined`, `Found::thinned`, `scatter`), so against the old code they do not compile, which is red of a kind and no more.

### Item 1. A real wall is not flat (the tolerance, the bow, what is left out and 6 ft or longer)
Cause (measured in part A): a real wall leans up to 2.2 degrees, bows 1 to 15 cm, is 1.7 to 3.1 cm rough by the median deviation, and is seen at 0.9 to 11 times the floor's density;
the finder cut each face with a plumb band 4.6 cm wide (so its top and foot fell out of the band: ceiling reach 0.49 to 0.60 against 0.80, floor reach 0.08 to 0.15 against 0.50),
judged it straight against the best face's clipped rms, and dropped any face with under a quarter of the median face's points (a clean 9 ft wall of the small room, at 0.9 of the floor's
own density, was dropped by that).
Built (all in `crates/plans/src/found_188.rs`, wired in `found.rs`):
- `fit_face`: each stretch is fitted as a plane that leans (offset against height, only where the face runs 4 ft or more up: LEAN_NEEDS; a patch seen through a window and a stray
  near the floor otherwise tilt it: found by a demonstration step failing), the points within the band of it, nearer each time; the direction turns to the points only on a face of 6 ft or more
  (FREE_FROM: a 3 ft stub is told to a degree at best, and a closet wall's pairing broke at 130.4 degrees against 130.0).
- A bow: fitted as a quadratic along the face; taken where it is 1/2 in or more, no deeper than 1 in 30 of the face's length (BOW_RATIO: a curve of 5 ft radius is a curve and not a bowed wall: it
  was taken as one at first and made 4 chords of it, found by the table), the face 6 ft or longer (BOW_LEAST_LEN), and where taking it in takes off more than chance does (an F ratio of 60: BOW_F);
  said at 1 in or more (BOW_SAY): "bowed 1 1/2 in over 12 ft", Assumed, and the wall is laid straight.
- The tolerance from the scan's own roughness: `scatter` = the larger of the floor's spread about its plane (1.4826 x median deviation of the points within 3 in of it) and the cleanest of the six
  strongest wall lines' spread; the band is that many times the made-up scans' band (1.5 cm = 0.0492 ft, 3 sigma = BAND 0.15 ft), from 1 (under 1.3 times: NOTICE) to 3 times. It is used as a fallback: a face is first fitted
  in the old band (so that two faces a thin wall apart stay two: at noise of 3 cm the 4 1/2 in partition was lost when the band was widened everywhere, found by the table) and only a face that is not
  straight is fitted again in the wide band, held to as many times the bar. "the walls of this scan are rough to about 1 1/4 in" is said only where the wide band was used.
- The sparse-face test (a face under a quarter of the usual points to the sq ft is not a face): the usual is now never more than what the floor shows to the sq ft (`usual.min(rho)`).
- What is left out and is 6 ft or longer (LIKELY_LEAST) is offered by its number: a face that reaches the ceiling over some of its length (30 to 80 in 100) and not over the rest, and a face that reaches
  the ceiling and whose foot is not seen: "likely a wall: ... it is not laid unless you say it is a wall (--take N)", a doubt like the others (never laid by --lay alone). A face that stands short of the
  ceiling over all its length is a cabinet and is NOT offered (offering it broke steps 1 and 7 of the existing demonstration, whose cabinets are 7 ft 1 in and 7 ft 9 in high: call 173's rule).
Red/green: demonstration step 21 on the before program: 4 walls but the west wall "0' 9 3/4" thick, seen from both sides" (two passes read as a wall's two faces: a thickness the scan does not give), no bow said:
red. After: 4 walls, none given a thickness, "bowed 1 1/2 in over 12 ft 3 in" (hand 1 1/2 in over 12 ft, to an inch and a foot), the walls close: green. Step 24 on before: the 8 ft hanging face is "left out ... not taken
as a wall"; after: offered as wall 4, "--take 4", laid only with it (5 walls laid with --take, 4 without). Unit: `a_bowed_wall_of_one_and_a_half_inches_over_12_ft_is_found_and_its_bow_said` (south wall bows
1 1/2 in over 12 ft, 1 degree of lean, 2 cm more scatter: found, depth within an inch, length within a foot, only that wall bows), `a_wall_that_leans_a_degree...`, `a_clean_wall_says_no_bow...`,
two for the offer and one that a cabinet bank is not offered.

### Item 2. One wall is found once
Cause: a wall scanned in passes shows each as a face; in the long room 11 walls were found for about 8 (one 3 times), and where two passes 10 in apart were paired the program invented a wall 9 3/4 in thick.
Built: after the side each face was seen from is told, faces parallel within 1 degree, within 16 in across (PASS_APART = PAIR_MOST), alongside for half the shorter's length (PASS_ALONGSIDE), and seen from the SAME one side
(only where the side was told by floor in front of it, not guessed: `seen_sure`; a closet wall's faces and the outer faces of a thick wall with no floor outside them were joined when this was not asked), are one face: the points
of both are the wall's, its extent the union, it is judged again. `Wall::joined` and a line say how many were joined: "2 faces of this wall were joined into one: ...".
Result on the real long room: 3 joins said (north wall, the long west wall, the south wall). NOT joined (and so found twice): the west wall of the long room as walls 4 and 5 (21 ft 6 in and 22 ft 11 in,
about 2 degrees apart in direction), and the small room's east wall as walls 2 and 3 (3 to 4 degrees apart): the rule is 1 degree, the same as pairing.
Red/green: step 21 (the west wall scanned twice 10 in apart is one wall, "2 faces ... joined"); unit tests `two_passes_of_one_wall_three_quarters_of_an_inch_apart_are_one_wall`,
`two_faces_a_wall_scanned_in_passes_ten_inches_apart_are_joined_and_it_says_how_many` (west wall found once, joined = 1).

### Item 3. Which way is up, told by shape
Cause: nothing asked; only the reading of the floor to the ceiling (6 to 15 ft) was looked at, so a house 14 ft deep read with y up reads a plausible 14 ft ceiling.
Built (`which_way_up`): along each of the scan's three directions the points are piled in bins of 1/10 ft; a pile's AREA is the 1 ft squares of the other two directions holding 3 points or more (NOT its count of points:
the first version counted points and said a house was on its side when two walls were scanned 8 times as thickly as the floor: my own phone scene tripped it; a real wall is seen at up to 11 times the floor's density). The two
largest piles 6 to 15 ft apart: where another direction's two cover 1.5 times (UP_BETTER) what the direction taken as up's two cover, it says first, with how far apart they are and both areas, and nothing is laid without --anyway.
A reading under 6 ft (the measure is wrong, not the way up) is left to the measure's own sentence; where both speak the two are said, the shape's first.
Hand figure: the 30 x 14 x 8 ft house read with y up: floor and ceiling 2 x 30 x 14 = 840 sq ft, the walls across the direction taken as up 2 x 30 x 8 = 480; said "lie 8' 0" across, facing north and south, and cover 980 sq ft;
across the direction taken as up ... cover 572" (the figures are in squares of a foot and the faces' ends: within the demonstration's 160 and 130 sq ft). Not said of the right way up (checked on every made-up house of the table, the phone
scene, and the three real scans). On the real small scan read with --up y it says it (15 ft ceiling and the shape: 502 against 141 sq ft); the old reading "plausible ceiling and went on" is the case the call named.
Red/green: step 22 on the before program: nothing said (5 walls found, "close"); after: said first, --lay refused, the same house right way up not said. Unit test `a_scan_turned_on_its_side_is_said_first...`.

### Item 4. Progress, and thinning
Built: a scan of 150,000 points or more says how it is going, a line at a time, to the error channel ("... looking for walls: line 11 of at most 120, 10 faces so far", "... 23 faces found; telling which are
walls", "... looking for openings in the walls"); a scan of more than 300,000 points (MOST_POINTS: the three real scans read 15,000 to 20,000 points a second, so about 20 s) is thinned first to one point in k by a fixed
scramble of the point's place in the file (never every k-th: a scan made in rows of one length would lose whole walls), the same points each time, and that is said: "the scan holds 1,008,964 points, more than can be read in
good time: it was thinned first, one point in 5 kept (251,737 points) ...". Measured: the large room 40 s (79 s under load) before, 7.5 s after (1 in 3); the long room 75 s before (117 s under load), 7.8 s after (1 in 5).
The small room (196,676 points) is not thinned and now takes 10 s alone (it took 1.7 s: the fit looks at more) : under the budget, and said as it goes.
Red/green: step 23 (a made-up house at 250 points to the sq ft, about 480,000 points): before: no thinning line, no progress lines; after: both, 5 walls, the partition 4 1/2 in, the walls close, read twice it says the same.
Unit test `a_large_scan_is_thinned_first_and_that_is_said` (400,000 points, 4 walls found all the same, a 60 per sq ft house not thinned).
Not done: the made-up houses do not take long (a 480,000 point house took 2 s before), so the demonstration shows the saying and not the saving; the saving is the real scans'.

### Item 5. The made-up scans learn from the real ones
`tools/made-up-scan.py --scene phone` (new; `face()` now returns its index and a face may be warped: new flags only; the byte-for-byte check: 64 of the 65 houses' scans regenerate identical, the 65th,
J-ew1, differs from the file the table holds but is identical to what the generator at HEAD makes, so the table's file is older than HEAD's generator, not changed by me). The scene: a 12 x 14 ft room, 8 ft high; south wall bowed 1 1/2 in; every wall leaning a degree and 2 cm rougher than the scan's own 1.5 cm; north and east walls 8 times as dense as the south and west (the real faces' density ranged 10 times: 370 to 4,750 to the sq ft); the west wall scanned twice, 10 in apart. The measures
and where they came from (part A, my own tool on the real files, no real point copied) are written beside it in a comment. `--scene hung` is the 8 ft hanging face of step 24.
`demos/findwalls.sh` steps 21 to 24, each with its figures by hand and `check` against what is said. Existing steps 1 to 20 are untouched and pass as written.

## The test that matters: the three real scans, before and after
By the picture (my own count, from the top-down images: `wtN/.scratch/small3.png`, `long3.png`, `large3.png`, and the overlays of what was found on them `ov2_*.png`). The lead's reading of the small room as "four clear walls" does not match
what the file holds: it is a cramped furnished room of about 8 wall pieces, only the south wall long (about 7 ft); the 3.6 x 4.8 m box includes a doorway's view and a ghost streak.

| scan | by the picture | before | after |
|---|---|---|---|
| small, 197k points | about 8 wall pieces: south 7 ft, west 6, north 5 (5 ft 8 in), east 4 1/2, four stubs of 2 to 3 ft | 0 found, 21 stretches "not straight" (4 s) | 3 offered as likely, none laid by itself (10 s): wall 1 the south wall (9 ft 8 in against about 7 ft: it runs 2 1/2 ft on into the ghost), bowed 2 1/4 in, said; walls 2 and 3 the east wall twice (8 ft 3 in and 9 ft 1 in against 4 1/2 ft: a strand and a ghost). West and north walls NOT found: said only as "left out ... not straight" and the north as 5 ft 8 in "reaches the ceiling and not the floor" (under 6 ft: not offered) |
| long, 1.0 million | about 8 to 9: north 14 ft (bumps 6 in deep), west 13 ft and the alcove's west 9 ft, east 12 to 14 ft, south end 9 ft, the alcove's two sides 6 and 5 ft | 11 found (one 3 times, two 10 in apart), 2 ends meet nothing, 75 s with no word, end walls missing | 12 found in 7.8 s (thinned 1 in 5, said), 11 not in doubt: north 14 ft 1 in (2 passes joined), west 13 ft 8 in, east 14 ft 6 in bowed 3 3/4 in, south 9 ft 9 in (joined), the alcove's pieces; the west line twice (21 ft 6 in and 22 ft 11 in, the second bowed 2 1/4 in over 23 ft: the west wall and the alcove's west joined across their jog), 6 ends meet nothing, said with where |
| large, 818k | about 10 to 12 wall pieces of 3 to 11 ft around a notched room; another room's wall far beyond a doorway | 5 short pieces (3 ft 6 in to 9 ft 7 in) in 40 s, 6 ends meet nothing | 8 in 7.5 s (thinned 1 in 3, said): west 11 ft 8 in, the 10 ft 7 in wall, the 6 ft 1 in notch wall, two short ones, none in doubt; and the north wall's two pieces joined across the notch into one 14 ft 8 in face, offered twice as likely (false), one beyond another wall (in doubt, said). The east wall (7 ft) is not found |

## Nothing that works gets worse (the table of 65 made-up houses, rerun; `wtN/.scratch/tab/before.txt` and `after8.txt`, `cmp.py`, `cmp2.py`)
60 of 65 rows identical; 52 identical in notes too. The 8 that differ in notes only move a "left out" stretch by an inch or two. The 5 rows that differ:
- N1cabin: 5 one-sided walls, open, to 5 with the 3 1/2 in closet wall found, closes. Better.
- J-curved: 5 walls with a doubt to 4 walls (the answer: 4). Better. N-storey2: 6 walls to 5 (the answer: 4 and a door). Better.
- J-ew1: 7 walls to 9 (2 in doubt, offered as likely: the cabinet bank, 12 ft 4 in, and a 6 ft 5 in face) against the answer's 9 each once: the two offers are the call's rule, not walls laid.
- N10cabin: closes to "1 end meets nothing": the 2 ft 4 in closet wall's south end is at (4' 10", 16' 0") and not (5' 1", 15' 9"): a short wall's end moves 3 in and no longer meets the cross wall (which is "under 2 ft", left out, said with where). WORSE, by a closing said, not by a wall lost or laid. Not fixed.
`demos/findwalls.sh` steps 1 to 24, `demos/scan.sh` and `demos/planwalls.sh` pass (PLUMBLINE_BIN= a frozen copy). The eight-angle test (step 5) passes as it was.

## Existing expectations I changed (each by call 188's own words; the lead decides)
1. `found_183_tests::a_beam_on_posts_or_a_soffit_is_not_a_wall_with_an_opening`: the 20 ft beam hung from the ceiling is now one wall in doubt, "likely a wall ... hangs from the ceiling" (call 188, 1: left out and 6 ft or longer is offered), the walls not in doubt still the room's four; it no longer says "reaches the ceiling and not the floor" in `left_out` for a face of 6 ft or more.
2. `found_179::a_wall_of_two_faces_ends_at_the_near_face_of_the_wall_it_meets`: its scene had the outer faces of its walls scanned with no floor outside them, so they were "seen" from the room, through the wall, as the inner faces are, and (call 188, 2) faces seen from the same side within 16 in are one face scanned in passes. I gave the scene a floor and a ceiling outside the walls, as a scan that saw the outer faces would have: the assertions are as they were. A scan that sees a wall's outer face from the room only is read as one wall scanned twice.

## Every measure chosen (constant, value, why)
BAND 0.15 ft kept (3 sigma of 1.5 cm); SCATTER_MADE_UP 0.0492 ft; NOTICE 1.3; MOST_WIDER 3 (the roughest real wall 4.8 cm = 3.2 times); PILOT_LINES 6, PILOT_WINDOW 0.3 ft, FLOOR_REACH 0.25 ft; TILT_MOST 0.06 (3 1/2 degrees; real 2.2);
LEAN_NEEDS 4 ft; FREE_FROM 6 ft; BOW_FROM 1/2 in, BOW_SAY 1 in, BOW_MOST 6 in, BOW_RATIO 1/30, BOW_LEAST_LEN 6 ft, BOW_F 60; LIKELY_LEAST 6 ft; PASS_APART 16 in, PASS_ALONGSIDE 0.5;
MOST_POINTS 300,000, PROGRESS_FROM 150,000; UP_BIN 0.1 ft, UP_CELL 1 ft, UP_POINTS 3, UP_APART 6 to 15 ft, UP_BETTER 1.5. Each is a named constant with a comment in `found_188.rs`/`found.rs`.
Why they would hold on a fourth scan: each is a ratio to something the scan itself gives (its floor's and walls' own scatter, its floor's density, the shape of its piles) or a physical size a room has (6 ft, 16 in, 3 1/2 degrees). None is a coordinate or a count from one file.
The one I am least sure of: PASS_APART at 16 in (a 12 in wall scanned from the same side only would be joined: it needs floor on both sides to be seen from both; a wall scanned from one side only is one face, so none is lost).

## Where it still fails on a real scan (blunt)
1. The small room is NOT four walls found: of about 8 wall pieces it offers 3 as likely (the south wall, the east wall twice) and finds none by itself; the west and north walls are said as left out, with where, and not offered. The wall bases are hidden by low furniture no face was found for.
2. The large room's north wall is joined across its notch into one false 14 ft 8 in face (two false offers); the east wall is missed. The long room's west wall is found twice.
3. Faces of one wall 2 to 4 degrees apart (strands) are not joined: the rule is 1 degree.
4. 21 stretches "not straight" on the small scan are listed and not offered: most are ghost streaks and no test I found told them from a bowed wall. A wall that is "not straight" and 6 ft or longer is NOT offered by its number (only a face that was found and then judged not a wall is).
5. The small room takes 10 s (was 1.7 s) because the fit looks at the stretch's wide window several times.
6. The "likely a wall" offers add noise on made-up scans with long beams and banks (J-ew1: 2 offers): by the call's rule.
7. Not reproduced: the judge's "ten times thinner" case (it passes the old table unchanged).
