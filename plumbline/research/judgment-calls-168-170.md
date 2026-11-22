# planset-move: judge's notes (binary 8627a83, bin24)

Working folder: .../judges/planset-move/ (jobs box0.json, h0.json, l0.json, t.json; helper env.sh)

## PIECE 1: call 170, `move` (a wall moved and the walls that meet it follow)

Wall-to-wall mechanics held (see "Held" below). The findings are about everything that is
NOT a wall, and a few words.

### P1-1 (wrong money and quantity, silent): moving an outside wall leaves the floor, footing, roof and the level above where they were; the program says nothing and `check` says all is well
Reproduce (empty job): `new`; level ground (0 ft, 9 ft) and upper (9 ft, 9 ft); wall 6 in; floor slab;
footing strip 24 in; closed walls 0,0 24,0 24,16 0,16 on BOTH levels; `floor ... --walls` on both
levels; room Down at 6,8; `roof Main --points "0,0 24,0 24,16 0,16" --pitch 6:12 --eave-height "18 ft"`;
`footing` on the four edges; allowances on walls $20/SF, floors $10/SF, roof planes $8/SF,
footings $300/CY (price 41,692.40, which I checked by hand: 28,800 walls + 7,680 floors + 3,434.6
roof (429.3 SF) + 1,777.8 footing (160 CF = 5.93 CY)).
`move box wall-0007 --by "4,0"` (east wall, ground level) said only:
    wall-0007 moved 4 ft east and 0 ft north / wall 1 followed: 4 ft longer / wall 3 followed: 4 ft longer
After it, by `show`/`takeoff`:
 - ground walls 88 LF, but footings still 80 LF (24+16+24+16) and floor still 384 SF (24x16),
   while room Down is 426.25 SF (27.5x15.5 inside): a room bigger than the floor it stands on.
 - upper level walls still 24x16 (80 LF), roof still 429.33 SF: the roof is 4 ft short of the house
   and the upper walls stand over 4 ft of nothing.
 - price rose only by the walls: 43,132.40 = +1,440 (= +72 SF x $20). By hand a 4 ft addition also
   wants +64 SF floor, +4 LF footing x2... so the price is understated and says so nowhere. Moving IN
   by 4 ft (`--by "-4,0"`) drops the price by the walls only (40,252.40), the footing and roof still
   priced for the old house. `check` -> "its figures agree with what is drawn"; `clashes` -> 0 (walls/floors
   are never tested against each other); `vif` -> "nothing".
 - The reply lists "things that stand against" but names no footing, floor, roof or wall above. The
   call text (rule 5) only promises things against the wall, so this is by the letter, but the
   brief's question "does the program say what it left behind" is answered no.
Right: say, after any move, that the floor outline, footing and roof of the level were not moved
and by how much they now disagree with the walls (or move them / offer to).
How bad: wrong money, silent. This is the worst of the piece.

### P1-2 (wrong area, silent): a room line that ends on a moved wall stays behind; rooms flood
Reproduce: box 24x16 + partition 12,0-12,16, rooms West (6,8) East (18,8), `room-line --points "12,8 24,8"`
(East is then 89.125 SF), `move <east wall> --by "4,0"`.
East goes 89.125 -> 240.25 SF (west half: `room-line "0,8 12,8"` and a wall moved -3 ft west gives
West 224.75 SF, the same flood). The room line (12,8)-(24,8) now stops 4 ft short of the wall at 28.
Reply says nothing about it ("2 things stand against wall 2" only lists cabinets). Right: the
room line ending on the wall is part of the room's boundary and follows (or the reply says it
did not and that rooms changed). How bad: wrong quantity (and the room-priced lines and receptacle
count: East 3 EA -> 6 EA), silent.

### P1-3 (accepted nonsense): a move that crosses another wall is not refused
Box with partition at x=12 and a wall 12,8-24,8 butting it: `move east-wall --by "-14,0"` gives
east wall at x=10, the partition now OUTSIDE the house at x=12, and the butting wall running
(12,8)->(10,8). Reply: "wall 6 followed: 10 ft shorter, and turned". `--by "-30,0"` gives an
inside-out house (east wall at x=-6 left of the west wall at 0, "wall 1 followed: 18 ft shorter,
and turned"). `check` clean, `clashes` clean. The call text says only "refused" for opening
and held-end cases; but "the program does not know it crossed" is a lie of omission.
How bad: misleading; the numbers are then geometry nobody drew.

### P1-4 (words): `--by` takes only feet as bare numbers; every spelling a builder types is refused with a confusing message
`--by "4 ft,0 ft"` -> `"4" is not a point; write it x,y in feet`. Same for `4' 0",0`, `48 in,0`,
`4 ft, 0`, `4 ,0` (a space), `1.2192 m,0`. `4'` -> `"4'" is not a plain number`; `4ft` the same.
The message blames "4" which is a perfectly good number the user typed. Everything else in the
program (offset, height) takes `4 ft`, `4' 0"`. Only `4,0` works. A builder typing 4' 6" cannot move a
wall 4' 6" without turning it to 4.5. How bad: untidy/usability, but it is the first thing the
verb does with a person.

### P1-5 (words): a move west is said "moved -4 ft east"; a move south "-2 ft north"
`move ... --by "-4,0"` -> `wall-0007 moved -4 ft east and 0 ft north`. A builder says "4 ft west".
Also a move of 0 on one axis says "and 0 ft north". Followers' words are right ("4 ft shorter").

### P1-6 (misleading): followers are named "wall 1", "wall 3" with no level, and the names repeat
In the two-level house, the reply says `wall 1 followed: 4 ft longer`; both levels have a "wall 1"
(and `show` lists ids wall-0006 and wall-0010 as "wall 1"). Nothing says which level or gives
the id. In the one-level house the ids are given only for things "standing against". Also
in refusals ids appear raw ("wall-0078: lies along wall-0008 for 12.00 ft").

### P1-7 (untidy, low): moving a room or a pipe says "no wall meets it: nothing followed it"
`move room-0012 --by "1,0"` -> the room's point moved 1 ft and says "no wall meets it". Reads as a
fault. Likewise for a pipe. Also a cabinet run left 4 ft from the moved wall is said (good) but a
pipe run 3 in off the face (x 23.5 beside wall at 24) and a fixture 4 ft from it are not.
(Vanity at 20,10 is not "against" the wall; fine.) Not a defect by the call; noting that the
reply covers only cabinets/things within a distance, and a pipe 0.25 ft off the face is not
listed while 2 cabinets are.

## Held (tried, behaved as the call says; figures checked by hand)
- East wall +4 / -4 in the one-level partitioned house: south/north 28 / 20 LF, window keeps its
  place (offset now 11.5 ft from its first end: 7.5 + 4 ✓), areas 240.25 and 178.25 SF ✓.
- Partition -2 ft: ends stay on south and north walls ("wall 5 stayed on wall 1 and wall 3") ✓;
  partition +3 along itself: refused, both ends held, file byte-for-byte unchanged (cmp) ✓.
- South-wall move: west/east/partition ends follow; door offset adjusted 19.5 -> 22.5 when the west
  wall moved -3 ft ✓. A door 2 ft in from the west end and the west wall moved +2 or +1 ft: refused,
  door named with how far it would stick out (1'6", 0'6") ✓ (exact).
- A wall butting the partition in the middle: partition -2 -> butting wall's end follows (10,8)
  ✓; butting wall slid 3 north stays on both ("stayed on") ✓; slid along itself refused ✓;
  east wall +4 drags the butting wall's end on it (12,8 -> 28,8) ✓; skew move 3.5,2 gives
  27.573 and the reply "3' 6 7/8" longer, and turned" ✓ (sqrt(27.5²+2²)-24 = 3.573 ft).
- L house: all six walls, along-wall and square moves; "turned" figures checked (0' 6 1/2",
  0' 3 3/8" ✓). A move onto another wall ("lies along wall-0008 for 12.00 ft") refused ✓.
- Move of 0: "moved by nothing", refused cleanly. A shrink to nothing is refused naming the wall.
- Move +4 then -4 (and 0.1+0.2-0.3): the geometry is identical to the original, every number
  (diff of the pretty-printed job, minus "source"): only the `source` text of each moved line
  differs ("made-up; moved with wall 2 (made-up)"), and does not stack on a second move. So
  the job is NOT byte for byte what it was after moving out and back: the history is kept, by
  design as far as I can tell; recording as a note, not a defect.
- Moved with `--status Assumed`: the moved wall's line and each follower's line read Assumed
  (`~ wall line`), room areas Assumed, vif lists them ("check: ~wall line"); others Known ✓.
- An existing-marked wall that follows or is moved is named ✓. `--alone` says "nothing followed it" ✓.
- Cabinet run left behind is told ("2 things stand against wall 2 and were not moved: B18, B24") ✓
  though they then float 4 ft off the wall with nothing saying so in `check` (`vif` says only
  "check: ~size").
- Unknown path, a door, missing --status: clear refusals.
- Not reproduced twice / not tried: window near the moved wall's far end, a thing moved after
  undo in the window (window not allowed).

## PIECE 2: call 168, `plan-set` and `page` (the plan set as a PDF)

Jobs and pictures: .../judges/planset-move/ps/ (ps.json, set.pdf, rd-1..3.png read at 150 dpi;
s2.json, s2-1.png, s8-1.png). I opened every picture.

The good: every number on the sheets is the model's own. By hand and against `show`/`takeoff`:
dimensions 24, 16, 24, 16, partition 10, new walls 12, 16, 12; EXISTING Living 364 (23.5 x 15.5 =
364.25); PROPOSED Living 365 (364.875 with a 4 in partition 9.75 ft inside: 23.75 x 15.5 =
368.125 less 3.25) and Kitchen 182 (182.125); closet 14 (14.42); 413 (413.21); the Assumed diagonal
wall "15'-7 7/16" VIF" (sqrt(10^2+12^2) = 15.620 ft = 15'-7 7/16") ✓; a door at the stated
place and width (3 ft gap from 3.5 to 6.5 ft: 349-464 px at 38.3 px/ft ✓). The dimension ticks
are at the wall centre lines, as the note says. Tidy to scale: wall 24 ft = 6.000 in on paper.
Nothing wrong was found in a figure. No wrong dimension found.

### P2-1 (misleading, a drawing a builder would take wrongly): a new door or window cut in an existing wall is drawn on the EXISTING plan and the DEMOLITION plan, and as old work on the proposed plan
Reproduce: `new`; level; wall 6 in 0,0 24,0 24,16 0,16 closed; `mark` the walls existing; THEN
`door ... wall-0009 ... --offset 5ft` (no mark: "a thing with no mark is new"). `plan-set ... --list`:
page 1 "EXISTING PLAN ... door-0017 in wall-0009: door, gap 3'-0", heavy"; page 2 the same, light;
page 3 the same, light (as existing weight). The existing page therefore shows a door nobody has
cut; the demolition plan says nothing about cutting a new opening; the proposed plan draws the
new opening as old work. (A door marked while its wall is marked comes out existing; only the order
differs.) `demos/planset.sh` step 6 enshrines this ("on each page the south wall is on").
Right: an unmarked opening is new work: not on the existing page; heavy on the proposed page, with
the hole in the existing wall. How bad: misleading on a drawing that is built from.

### P2-2 (overlap / cut off): a long project title runs out of the title box and across the sheet frame; and is set smaller than 3/32 in
Reproduce: `new f "A very long project title for an addition to the Smith Residence at 123 Main"`; any
level and walls; `plan-set`. On the picture (s2-1.png, s8-1.png) the title line crosses the right
edge of the title box and the sheet frame, ending ~8 px (150 dpi: 0.05 in) from the paper's edge. It
was shrunk to about 0.06 in cap height (measured ~7 px at 110 dpi): below the 3/32 in (0.094 in) a
reader needs. Elsewhere text measured right: notes, dimensions, room labels all 15-17 px at
150 dpi = 0.10 to 0.11 in (>= 3/32 ✓); title 0.19 in ✓. How bad: untidy / could be cut off by a plotter.

### P2-3 (overlap, unreadable): small rooms. A room's name is centred on its point and runs through walls; its area and the wall dimensions pile on each other
Reproduce: s2.json: a 6 x 3 ft closet named "WALK-IN CLOSET WITH SHELVING" (4 in partition walls
(6,0)-(6,3)-(0,3)); room label spans well beyond the closet, over the walls; "14 SF" is overprinted by
the 6'-0" and 3'-0" dimension strings; the closet's dimensions read as "64-6F". At 1/8 in the same.
"HALL 52 SF VIF" is overrun by the diagonal wall (H is cut off by the wall strip). No leader, no
shortening, no moving. How bad: untidy to unreadable on a normal plan (a closet or a bath is normal).

### P2-4 (a thing in the house is not on the drawing, and the drawing does not say so)
The cabinet run (B18, SB36, B24 and a countertop, set 1 ft from the wall in Kitchen), a fixture
(Vanity), a pipe: none is drawn on any page, and the notes list nothing as "not shown". `--list`
has no "furnishings" line. A builder reading the proposed plan sees an empty Kitchen. The model
knows them (`takeoff`: "Furnishing B18: 1 EA", "base run 29.5 LF (less 6.5 ft behind 3 things set
against the walls)"). Right: draw them, or say in the notes "NOT SHOWN: 3 cabinets, 1 fixture".
Also the room line is not drawn and not said (it is no wall; fine), the upper level has no ghost.

### P2-5 (completeness, as an architect or a builder reads it): not a drawing someone builds from yet
- No sheet number (A-1.1, A-1.2...): only "PAGE 1 OF 3". No north arrow (a sentence says "PLAN NORTH IS UP
  THE PAGE"), no graphic scale bar, no project address/owner/drawn by/revision in the title box, no
  seal. No key for the three line types (heavy filled / light / dashed): only the demolition
  page's line under the title ("1 WALL COMES DOWN, 16 LF") and no word anywhere that dashed means
  "to come down" or light means "stays". The existing and proposed pages carry no legend either.
- No door swings (said), no door or window mark/number, no schedule, no openings dimensioned (said),
  and no overall dimension: the proposed plan gives 24'-0" and 12'-0" along the top, never the
  36'-0" (the strings do add up: 24 + 12 = 36, 16 = 16). No dimension from a wall to an opening.
- Dimensions are to the wall's centre line, said "TO THE WALLS' LINES AS DRAWN": a builder does not
  say "wall line" and the sheet never says "centre line" or "face"; every layout dimension a
  contractor lays out from is to a face or stud. The inside clear dimension a room is built to is
  never given.
- Room names are uppercased and the area is whole SF; "VIF" after an area is said in notes.
- The existing plan is placed left of centre with a large empty field on the right (PAGE 1 and 2 of
  the 24 x 16 house: house at 215-1135 of 1800 px); the proposed page is near centred. Untidy.
- Level notes: "WALLS ARE CUT 4'-0" ABOVE THE FLOOR" is true and welcome.

### P2-6 (model, shows on the sheet): a window placed past the end of its wall is drawn hanging in the air
Reproduce: wall 0,0-40,0; `window ... --offset "39.5 ft"` (3 ft wide, so it ends 1 ft past the wall's
end): accepted ("window-0023: a window in wall-0007"), `check` and `clashes` clean, `show` flags
the rooms' wall figures Unknown ("window-0023 does not fit its wall"), but `plan-set` draws it
heavy at the corner out into space and `--list` says only "window, gap 3'-0", heavy". The
sheet and the notes do not warn that one opening does not fit its wall. The same job also accepts two
doors overlapping on one wall (offsets 10 and 11 ft, both 3 ft wide); the plan draws a stray tick in
the gap. Right: refuse at `door`/`window` (or at least `check` should say), and a sheet should not
draw what the model knows does not fit. How bad: wrong drawing + silent.

### P2-7 (the file, tidy)
- `plan-set ... o.txt` writes a PDF named .txt without a word; `plan-set ... set.pdf` over an
  existing PDF writes over it silently (it refuses to write over the job file itself, good).
- The text on the sheets is drawn with lines: `pdftotext` returns nothing, the file has no font
  or text, no title or other information, so a reader cannot search, select or read the
  sheet's text and a screen reader gets nothing. (Said in call 168: "the lettering of strokes, no
  font": by design; recording it as an effect.)
- `--date x` is accepted: "DATE: x" (it only refuses an empty date); `--date` is free text.
- Without `--date` the day is today's, so the same job written on two days writes different bytes.
- Readers: header `%PDF-1.4`; `pdfinfo` reads it (3 pages, 864 x 648 pts), `pdftoppm` renders every
  page without a warning, the program's own `page --png` renders 4096 x 3072. (mutool, qpdf, gs not on
  this machine.) File 31 kB for three pages. Plan not generated a second time with a different
  result: same job, scale, paper and day gives the same bytes (cmp, twice).
- A level with only an empty name `g` is refused as "g has no wall on its plan to draw": good, but
  names the level by its title and not the id (fine).

## What held in piece 2 (tried)
Refusals leave no file and the job file byte for byte unchanged (cmp): unknown level ("its levels
are ground (level-0002), upper (level-0003)"), a paper that does not exist ("ARCH A, B, C or D"), a
scale not offered ("1/8, 3/16, 1/4, 3/8 or 1/2"), 3/8 at ARCH A refused with the scale that fits,
`--date ""`, unwritable folder, the job file itself as the target. `--scale '1/4"'`, `1/4 in`,
`--paper "arch b"` all accepted. A level with no mark: one page, FLOOR PLAN; the upper level too.
A level (the ground of a two-level job) drawn without the upper level's walls. VIF lettered
after exactly the Assumed length and the two rooms whose outline rests on it, with counts in the
notes ("1 LENGTH IS TO VERIFY IN FIELD (VIF). 2 AREAS ARE TO VERIFY IN FIELD (VIF)."), on the
page; the demolished wall is dashed and the proposed page omits it ✓; rooms the existing plan
cannot enclose are named as not lettered (Kitchen) ✓.

## Where I stopped
Not tried: a job with an Unknown length (lettered `?`), two levels' worth of stairs, a hip
roof (not drawn, nor claimed), a sheet at ARCH D with a very large house. Did not start the window.
Nothing in the repository was changed; my files are in judges/planset-move/.
