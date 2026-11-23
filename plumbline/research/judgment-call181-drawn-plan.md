# drawn-plan-walls: judge's notes (find-walls on a drawn plan, call 181)

Program: review/bin30/plumbline (7a7ffa1). Everything is in judges/drawn-plan-walls/: pdfw.py/gen.py/h.py
(a small PDF writer and a house drawer, standard library only), the sheet scripts (s01.py, all*.py, d*.py, ...),
sc.json (each sheet's scale points), run.sh / runall.sh (new job -> level -> drawing -> calibrate -> lineup ->
find-walls), and for every sheet NAME.pdf, NAME.json (the job) and NAME.out (what the program said).
Base house for most sheets: 30 x 20 ft, 8 in outside walls, 4 1/2 in partitions at x=15 and y=10 (east half),
doors, 3/2/1-line windows, slider, pocket; hand answer 6 walls, 135.00 LF, plus a phantom "wall in doubt" for each
3-line window. About 60 sheets run. Metric (1:50) can only be scaled by typing feet (the program says US only): fine.

## Findings, worst first

### 1. A dashed line pair (a beam, soffit, anything shown hidden) is laid as a wall, not even "in doubt"  [wrong quantity]
Repro: python3 fl.py; ./runall.sh f_beamdash ; then `$B find-walls f_beamdash.json drawing-0003 --lay`; `$B takeoff`.
Sheet: the base house plus two DASHED lines 8 in apart, 15 ft long, wall face to wall face (a beam over the living room).
Said: "wall 3: from 0' 0" east, 5' 4" north to 15' 0" east, 5' 4" north, 15' 0" long, 0' 8" thick" with no doubt; "the walls found
close". Only "2 dashed lines are kept whole: the gaps in it are places to snap to" (a line about snapping, not walls).
Laid: 7 walls, takeoff 150.00 LF. Right: 135.00 LF (6 walls). 15 LF of 8 in wall, 135 SF of face, in a framing estimate that
was never asked for. Dashed = hidden/overhead in every architect's set; it should never make a wall unasked.

### 2. Two lines between walls (a shelf, bench, ledge, duct) are laid as a wall, not "in doubt"  [wrong quantity]
Repro: fl.py -> f_shelfbox, f_shelfopen, f_bench.
 - f_bench: two lines 6 in apart from the west wall to the x=15 partition (a ledge): "wall 7 ... 15' 0" long, 0' 6" thick",
   not in doubt, "the walls found close", laid by --lay (150.00 LF vs 135).
 - f_shelfbox / f_shelfopen: a closet shelf 12 in deep, 5 ft 8 in long, touching the west wall: found as a 6 ft wall 1 ft thick,
   not in doubt; the only word is "the walls found do not close: wall 7, its east end, at 6' 0" east, 14' 6" north" (reads as a room
   left open, not as a shelf that is no wall).
The rule "a pair that meets no wall at either end is in doubt" is defeated by anything that touches a wall once.

### 3. A wall section / detail at another scale on the same sheet becomes a wall, not in doubt  [wrong quantity]
Repro: sec.py -> secnoframe. A 3/8 in section (6 in wall, slab, rafter pair) to the right of the 1/4 in plan, no frame.
Said "walls found: 10, 2 in doubt": wall 4 "from 39' 3" east, 2' 6" north to 39' 3" east, 15' 4" north, 12' 9" long, 0' 9" thick" is
NOT in doubt (it meets the slab pair, which meets it back), and wall 9 (the rafters) is "7 1/2 in thick". Only a sentence that
wall 9's east end meets nothing. With a frame (secframe) the box is left out and counted as "a title block, a key, a schedule"
(wording: it is a detail). Real sets rarely frame a section.

### 4. A door beside a window (pier under 2 ft) vanishes with no word, and "the walls found close"  [dropped opening]
Repro: d2.py; ./runall.sh pier150 (and pier75, pier100). South wall, door 10-13 ft, window starts 13+pier. Piers 0.75, 1.0, 1.5 ft:
"a plan gives no head or sill: 1 opening was found" (two drawn: only the window); door gone. At 2.0 and 2.5 ft both found.
Also d_b (window 1 1/2 ft wide next to door: the window gone, door kept). In the 60 x 40 house (big.py) the 3 ft front door
24-27 next to a window at 28.5 (pier 1.5 ft) and a partition door 2 ft from a corner were lost: 23 openings "found" of 25 drawn.
Nothing says an opening was missed; "the walls found close" stands. (The pier itself is not counted under "stretches under 2 ft"
here, though it is in another sheet.) Piers of 12 to 18 in between a door and a sidelight/window are ordinary.

### 5. Half-picture sheet: "nothing else on the sheet was left out" while the picture is not read  [misleading]
Repro: img.py; ./runall.sh halfscan. West half vector, east half an image. Said: "not read: 1 picture on the page is shown and has no
lines to snap to" ... "nothing else on the sheet was left out". Found 3 walls, said they do not close (right). But a builder
reads "has no lines to snap to" as a tracing note and then "nothing left out": nowhere is he told that half the house (the picture)
was not read for walls.

### 6. A clip on the page: "the walls found close" when walls are cut off  [misleading]
Repro: all.py -> clip. A clip shows only x < 16.7 ft. Found 3 walls; the west wall (its outer line is clipped away) is not found at all;
wall 1 and 3 end at the clip edge. Said "not read: part of the page is covered over: 10 lines are cut ... 23 hidden lines are left
out" (good) but also "the walls found close: every end meets another wall" (false: the south and north walls stop at the clip,
at 16' 8", meeting nothing). halfvector: both found walls "in doubt", no closing sentence at all.

### 7. No warning when the scale is from the wrong dimension  [misleading]
Repro: ./run.sh wrongdim s02.pdf "100,150" "640,150" "33 ft" "100,150" (30 ft dimension typed as 33). calibrate says
"1" = 4' 4 3/4"" and the finder goes on: walls 33 ft, 22 ft, thickness 8 3/4 in, windows 5 ft 6 in; everything 10 % big.
Nothing says that is not a drawing scale (1/8, 3/16, 1/4, 3/8, 1/2) nor that 8 3/4 in is not a stock wall.

### 8. --doors "0 ft" / "-3 ft" is accepted and lays a door 1 inch high  [wrong quantity, silent]
Repro: `$B find-walls s02overshoot.json drawing-0003 --lay --doors "-3 ft"` then `$B show` the door: "type door 36 in by 1 in,
height 0.0833 ft, opening area 0.25 SF"; takeoff net face 1,213.42 SF instead of 1,055.33. 0 ft the same. No refusal, no word.
(--doors "10 ft" on a 9 ft wall is refused per wall with a good sentence; --windows head below sill is refused; "80" with no unit is
refused with a good sentence.)

### 9. Not lined up: walls laid at page offsets with no word  [misleading]
Repro: nolu.json steps (drawing, calibrate, no lineup, find-walls --lay). Walls come out "from 5' 7" east, 8' 4" north"
(wherever the page's corner puts them) and are laid there. Nothing says the sheet has not been lined up to the plan's
origin. (Probably meant; but a person reading the list cannot tell.)

### 10. A misspelled drawing name gets the scan's refusal, with a software term  [word]
`$B find-walls job.json nonsense` -> "say which level the scan is of: --level <level>"; with no argument:
"find-walls takes the job's file, a scan (a PLY point cloud) and --level <level> ..." A builder who mistyped a sheet's name is told
about a scan and a point cloud.

### 11. Layered walls: loud and one sentence is false  [noise / wording]
 - layers3 (3 lines a side), layers2, studsiding: "walls found: 36, 36 of them in doubt", 24 for studsiding (one per pair of lines),
   walls 29' 11" long 6 1/2" to 8" thick overlapping each other. Said, but unreadable. --take 1 --take 6 lays one and refuses the
   second "lies along wall-0005 ... counted twice" (good); a stray type "8 in wall" is still made for the one not laid.
 - wallmid (each wall drawn with a line down its middle): "no wall found: no two lines 3 in to 14 in apart run side by side for 2 ft or more"
   is false (the outer pair is 8 in apart); the true reason ("sets of 3 evenly spaced lines") comes below. All four walls lost, said.

### 12. Every 3-line window makes a phantom "wall in doubt"; several in one wall bridge into one  [noise]
Each window drawn with 3 lines gives "wall N ... 5' 0" long, 0' 4" thick, in doubt ... (a counter's edge ...)". In the 60 x 40 house
the window lines of two windows 16 ft apart joined into "wall 3 ... 23' 6" long ... in doubt" carrying "door or open doorway 16' 0" wide".
Not laid unless taken, but five doubts on a house that has none.

### 13. Lines that run 1 in past every opening's jamb shatter the plan  [dropped openings, said badly]
Repro: all-piece overshoot (ovall.pdf, made by the lines at the end of gen_all.py; ./runall.sh ovall): 14 "walls" 4-12 ft long, zero openings,
"the walls found do not close: 11 ends meet nothing ... (a wall may be left off the sheet)". Said, but as open rooms not as doors.
Overshoot only at the outside corners (over1, over6) and corners that fall 3 in short (gap3, gap1) are all fine: 4 walls, 100 LF.

### 14. Stacked dimension strings drawn with solid arrowheads are not recognised  [noise]
Repro: dm2.py -> st_arrow, st_none (two strings 9 in apart, 24 ft long): "wall 1 ... 24' 0" long, 0' 9" thick, in doubt". With ticks (st_tick) fine.
Single strings 10 in off the wall: fine in all four styles.

### 15. Small untidiness
 - s01 (counter 2 ft deep 4 1/2 in off the north wall): the west wall is found 19' 10" long (2 in short, the 20 ft wall ends meeting a wall in doubt)
   with no word; repro s01.py; ./run.sh s01 s01.pdf "100,150" "640,150" "30 ft" "100,150". The north wall itself is "in doubt: another line lies beside it".
 - Walls laid are named in order ("wall 1" is found wall 2); the report says "found wall 2". `check` fine.
 - Second laying: "Take those away first if it is to be laid again": there is no verb that takes a wall away (usage lists only remove-line).
 - `carries`: "Assumed: it rests on positions that are not Known: wall 1" says nothing.
 - A window drawn with one line is laid as a door if --doors is given (the general sentence says so once; the list says "door or open doorway").
 - The furniture boxes (stair, bed, tub) are reported "inside 3 boxes (a title block, a key, a schedule ...)": wrong words for a builder.
 - `apply <wall> <assembly> --source` marks the wall's "condition" Known (the one Known in the saved job; the finder itself wrote none).

## What I checked by hand and held
- Base house, plain two lines, three, four and six... (layers see above), poché solid + line (pochecut), filled shapes only with and without
  gaps (filledonly, filledopen), 45 degree hatch in walls, hatching of thousands of lines over the sheet, one giant path, white lines
  (told "not ink"), thick exterior/thin interior, walls in 2 pieces (pieces), corners short 3 in / over 6 in, dimension strings with ticks,
  arrows, dots, none, grid lines (dashed and solid) along wall centres, bubbles, north arrow, title block as open lines, leaders,
  room names/areas lettered as strokes at three heights (240 strokes "too short"), a furniture layout (cabinet run, sink, table, bed, stair
  with break line, tub, toilet, fireplace, deck): no wall for any; bezier swings and a round column "known by their ends only";
  tile grid: nothing. 1/8, 3/16, 1/4, 1/2 in scale; 90 degrees (page turned and plan turned) and 15 degrees: right in all; two plans on a sheet at one
  scale: both found; metric by typing feet: right.
- The big house (60 x 40, 12 rooms, 22 drawn openings): walls 434.67 LF = my hand 434.67 minus a 5'4" stub lost beside a door. Lay with
  --doors "6 ft 8 in" --windows "3 ft" "6 ft 8 in": 11 walls, 15 doors, 8 windows. takeoff: 434.67 LF, face gross 3,912.00 (= 434.67 x 9), net
  3,476.50 (= 3,912 - 435.5 openings by my hand), 8 in 200 LF, 4 1/2 in 234.67 LF. (The front door and one partition door never laid: finding 4.)
  On the base house: 135.00 LF, 1,215 SF, net 1,055.33 SF by hand, door centres 6.5 and 9.8335 ft (15 degree sheet too), rooms 554.43 SF =
  279.93 + 137.25 + 137.25 by my hand. `check` passes, `estimate` with "Exterior wall, 2x6, lap siding" / "Interior partition, 2x4":
  paint both sides 960 SF on a 60 ft partition with 3 doors = 2 x (540 - 60). Floor `--walls` takes the centre-line outline (2,400 SF for 60 x 40).
- Saved job: 77 openings' heights "Assumed, said when the walls were found", 35 "Assumed, found on sheet big"; no Known from the finder.
  Rooms: "no total, 7 of 18 missing" said when walls did not enclose. Second laying refused, file unchanged. --take 99 refused with a sentence;
  --doors without --lay refused with a sentence; head above wall height refused per wall.
- Sheets off the page: "7 lines are drawn off the page" said and the walls then do not close (said).
