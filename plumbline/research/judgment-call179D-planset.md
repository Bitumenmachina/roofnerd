# planset-2: judge's notes (plan-set). Binary: review/bin26 was deleted mid-run (another agent); I used review/bin27/plumbline, copied to judges/planset-2/pl. Its output was byte-identical to bin26's on the one job I had written with bin26 (L3, same date), so I take it for the same plan-set code. Working folder: judges/planset-2/ (lib.sh builds jobs; chk.py checks every string against `show`). mutool is not installed; pdfinfo and pdftoppm and pdftotext are.

## Findings so far (written as I go)

### PS2-1 (misleading, builder must phone): the "?" area is printed on the sheet with no word why
Empty job; L house closed walls 0,0 40,0 40,16 20,16 20,30 0,30, rooms "Bed" at 10,22 and "Living" at 30,8 (no wall between): sheet prints `BED 885 SF` and `LIVING ? SF`. Nothing on the sheet or in the notes says the room has no area because it shares its space with Bed. The existing/demolition sheets DO have a note for rooms they leave off ("NOT LETTERED, NOTHING THAT STANDS CLOSES ROUND THEIR POINTS: HALL, FAMILY"); the proposed sheet has none for "?" rooms. Same on L reno proposed (wall 7 demolished): `HALL ? SF`, `KITCHEN ? SF` side by side with `FAMILY 182 SF`. And the one that does print (BED 885) is the whole L, silently counting the space of the room that has no area (and Living's name floats in the same space). Right: a note "HALL AND KITCHEN SHARE ONE SPACE: NO SEPARATE AREA" or the shared area printed once. How bad: misleading/missing.

### PS2-2 (misleading): "DIMENSIONS ARE TO THE WALLS' LINES AS DRAWN" does not say centre line or face
6 in walls drawn 0,0..40,0; strings read 40'-0". The wall line is the centre line (outside face to outside face is 40'-6"). A framer reads 40'-0" as the slab/outside size. The note should say CENTRE LINES OF WALLS. How bad: a builder's first phone call; wrong by wall thickness if read as faces.

### PS2-3 (missing): no door or window is dimensioned or located
Notes admit it ("OPENINGS ARE NOT DIMENSIONED"). Offset in the model is the opening's CENTRE (door 5 ft along a wall is drawn 3.5 to 6.5 ft), the sheet gives no way to read it. A builder cannot place a rough opening from this sheet. How bad: the next worst after a wrong dimension.

### PS2-4 (untidy, can mislead): overall string's end extension line hangs in the air on an L
L (and L reno): the overall 40'-0" across the top has its right extension line stop at the top of the bounding box, 290 px (about 12 ft at 1/4 in) above the corner it dimensions (the notch corner at 40,16 / 52,16), with a gap then a short horizontal stub; the right-hand 30'-0" overall likewise starts in the air at the top. At the notch corner (20,16) three ticks, two strings and their extension lines pile up over the wall outline (zoom 300 dpi: extension lines run through the wall outline; ticks touch each other).

### PS2-5 (marks): marking a wall marks its doors and windows the same, silently placed on "existing" sheets
`mark wall existing` says "door-0018, in it: existing to remain" (it says), but a door placed before and left unmarked in that wall becomes existing, not new: a new door cut in an existing wall must be marked `new` by hand afterwards (and then says "it carries no cost lines: give it its assembly..."). The plan-set follows the mark correctly (door-0026 heavy on proposed only after `mark door-0026 new`). Held.


### PS2-6 (WRONG LETTERING, every sheet it touches): an ampersand or an accented letter is printed as "?"
Empty job: `new J "The Henderson & Sons \"Garden View\" Résidence ..."`; a level named `Première étage ...`; a room `Café & "Bar" Ünderstairs ...`. `plan-set` writes without a word. The sheet reads `THE HENDERSON ? SONS "GARDEN VIEW" R?SIDENCE`, level `PREMI?RE...`, room schedule `CAF? ? "BAR" ?NDERSTAIRS...`. The quote survives; & and every accent become "?". The program's own mark for "unknown" is "?" (as in `? SF`), so a reader takes it as the unknown. A client's name, a street, a room called "Bath & Laundry" or "Café" is misprinted on every sheet. Right: fold accents (É to E) and spell & as AND, or refuse with words. Wrong/misleading text on a sheet: bad.

### PS2-7 (misleading): a long title or level name is cut, and the cut is not said
126-letter title: title box shows two lines, `THE HENDERSON ? SONS "GARDEN / VIEW" R?SIDENCE RENOVATION AND...` (the rest of the job name is lost on every sheet, the "..." tells the reader something is cut but nothing on the sheet says where the full name is). Level name (48 letters) `LEVEL: PREMI?RE...`: the one thing that tells a reader which floor this is is cut to 7 letters, and two levels with the same first letters would read the same. Right: a title may be cut, but the level must be told in full somewhere (a second line, or the sheet's own number and a list).

### PS2-8 (misleading; the program's own check sees it and the writer ignores it): lettering over lettering is written without a word
Kitchen 16x12 ft closed room, an island placed at the room's point (`place ... furnishing "Island" --at "8,6"`): the room is given a numbered mark ("1") and the mark lands on the island's name: the sheet reads `ISL1AND`. `plan-set ... --list` says `lettering faults: 1 / "ISLAND" is over "1"`. Plain `plan-set` (no --list) says only `wrote C2.pdf: 1 page ...`, exit 0. So the program knows and prints the sheet anyway. Right: say it when writing (or move the mark).

### PS2-9 (misleading; the "lettering faults: 0" is blind to lines): dimension strings and things are drawn through room names, areas and dimension figures
The checker only tests lettering against lettering. By eye (and at 300 dpi crops) on the 10-room reno (B.json, proposed page):
 - the 36'-0" string of the south wall is hung INSIDE the house (on the proposed page; on the demolition page of the same job it is outside), 3 ft above the wall, and its line runs through `PANTRY ? SF` ("? SF" struck through) and across the whole Living room; the west wall's 28'-0" string likewise runs inside, through `LINEN 9 SF` and across Bath and Living.
 - `HALL ? SF`: the two 24'-0" strings of the hall's walls lie one over the other and strike the label; on the demolition sheet the string runs along the top of `109 SF`.
 - `3'-6"` figures of the 3 ft closets land on `? SF` of CLOSET and on LINEN; the "3'-6"" vertical string cuts the E of LINEN.
 - a 30'-0" figure sits across the 6 in partition wall (N.json: `30'-0"` text over the wall fill, both ends).
 - the vanity's outline (a 2x1.6 ft fixture) crosses `78 SF` of BATH; the island's top edge cuts `? SF` of KITCHEN.
 Right: a check of strokes against strokes, not only of words. How bad: unreadable on exactly the small rooms a renovation is about.

### PS2-10 (MISSING, the worst missing): no dimension locates anything; every string is a wall's own length
Each string is one wall, end to end. No string says where a partition stands (the hall: two 24'-0" strings, but nothing says the hall is 3'-0" clear, nor how far from the south wall; the closets: "3'-6"" is the wall, not the 3'-0" clear). No opening is dimensioned (notes admit it) and the model's offset is the opening's CENTRE (door at offset 5 ft is drawn 3.5 to 6.5 ft along the wall): a reader cannot learn that from the sheet. No cabinet, island or fixture is dimensioned. A framer cannot stake one partition from this sheet; he has to scale it. This is not a bug of arithmetic; it means the sheet is a drawing with wall lengths, not a plan to build from.

### PS2-11 (misleading): on an angled wing the strings do not add to the overall and nothing says so; no angle is given
Angled house (rect 30x20 and a wing from (30,0) at 30 degrees, 20 ft along it): the top reads `30'-0"` + `11'-3 15/16"` under an overall `47'-3 7/8"`; 30 + 11'-3 15/16" = 41'-3 15/16", short by 6'-0" (the wing's wall is a slope and is lettered by its own length; the overall is the extent). The wing's 20'-0" and 12'-0" are along the slope, parallel to the line, and no angle (30 degrees) or bearing is lettered anywhere. A builder cannot lay out the wing from the sheet. Every figure is right to the model (20, 12, 11'-3 15/16": checked by hand: dx 11.3205, dy 0.3923, 11.3273 ft = 11'-3 15/16"); what is wrong is that no figure is the one he needs. Overall: 47.3205 ft = 47'-3 7/8" and 20.3923 ft = 20'-4 11/16", checked.

### PS2-12 (the builder's L and T question: what a person is told)
L (40x16 plus 20x14; no step shown on one chain): the figures are right and do add, but not as chains. Top: `20'-0"` (north wall) over an overall `40'-0"`; the other 20 ft is the notch wall, lettered 14 ft lower down at the notch (`20'-0"` at y=16). Right: `16'-0"` beside the east wall and `14'-0"` 20 ft away at the notch wall; overall `30'-0"` on a third line. 20+20=40 and 16+14=30 hold, but no string says "20 + 20" or shows where the two meet; the reader has to find the two strings and add. The sheet says nothing ("no chain"). T house (60x46: bar 60 x 16, stem 20 x 30): 20+20+20=60 across the bottom of the bar and stem; 16+30=46 on the left and right; also `60'-0"` printed twice at the top (wall string and overall), the same at the bottom. All 8 strings checked against `show` plan lengths by script (chk.py): 0 mismatches on L, T, angled, big reno, kitchen. So: nothing wrong, but the overall's end extension lines on the L, T and reno stop in the air (a gap, then a stub) far from the corner they dimension.
Every plain box also gets TWO identical strings per side (8x8 ft box: eight strings, each `8'-0"`; 80x24 ft box: eight strings): the wall string and the overall say the same. Untidy; noise on the small sheet.

### PS2-13 (misleading): on the L and reno plans a room with no area is `? SF` but the notes say nothing; and the existing plan names a bounded space for two room points
(see PS2-1.) Existing sheet of B.json (`HALL 109 SF`): by hand the Hall space as it stands (the 3 ft hall 23.75 ft + the 3 ft strip past its open east end, 11.75 x 3.25 up to the room line) is 71.25 + 38.19 = 109.4: the sheet letters that whole space as HALL, while the Nook's point (30,14) lies in that same space and the sheet says of Nook "NOT LETTERED, NOTHING THAT STANDS CLOSES ROUND THEIR POINTS" (false: walls enclose it, with the hall). The proposed sheet gives the same situation `?`. Two rules for one case.

### PS2-14 (misleading): a room line is not drawn, and the area depends on it
L reno proposed: `FAMILY 182 SF` (12 x 16 ft addition, bounded on its west by a room line where the east wall stood). By hand 11.75 x 15.5 = 182.1 (right). No line, no note, nothing on the sheet shows where the 182 SF ends; to the eye the Family room is open to Kitchen. Same for B.json (Nook/Kitchen). The brief's "a room line" is invisible on the paper.

### PS2-15 (a key that shows what the sheet does not use)
Upper level of B.json (nothing marked demolish): EXISTING PLAN key reads `EXISTING, heavy; TO DEMOLISH, heavy`, `--list` says `key: EXISTING, heavy; TO DEMOLISH, heavy`; no wall on that sheet is to demolish. (Ground and basement, which do have a demolished wall, read the same, so the key is fixed text on the existing sheet, not worked from the sheet.) And on that sheet the two keyed swatches are identical, so existing and to-demolish cannot be told apart in black and white on page 1 at all (by design, as the call says; a builder has to turn to page 2).

### PS2-16 (title block): sheet numbers repeat across levels
Same job, three levels: ground A-101..A-103, upper A-101..A-102, basement A-101..A-103. Three sheets are called A-101 and two A-102, so `A-101` does not name one sheet; "SHEET 1 OF 3" is per level. Dates: `--date yesterday` and `--date 2026-13-45` and a short phrase are lettered unchecked on every sheet (only a long one is refused: "a few characters"). Without --date it is the day written (2026-11-23).

### PS2-17 (words/untidy): names leak the model's own suffix; unnamed things
Kitchen cabinets: the second B24 is lettered `B24 2` on the plan (the model's disambiguating counter printed as if it were a size). DB18, the countertop, W2430 and W3030 (hung) are "drawn with no name in it" and the note says `4 THINGS ARE DRAWN WITH NO NAME IN IT: TOO SMALL, OR HUNG OVER THE FLOOR`: it does not say which four. Bath: toilet and vanity are two unnamed rectangles; only TUB is named. A sheet with a run of cabinets in which the DB18 is a blank box. Cabinets and island are outlines at their own sizes with no dimension (PS2-10). Notes on the existing/demolition sheets: `4 CABINETS AND FIXTURES SET IN PLACE ARE NOT SHOWN ON THIS SHEET` when the four are NEW things: reads as though existing things were omitted.

### PS2-18 (the file)
`pdfinfo`/`pdftoppm` open every PDF; page sizes right (ARCH A 864x648, B 1296x864, C 1728x1296, D by scale: 80 ft house chose C at 1/4, B at 1/8 and 3/16, D at 3/8; 130 ft chose D at 1/4; 300 ft at 1/8 is refused in words: "fits no paper this program writes (ARCH D is the largest) at any scale"; an 80 ft house forced to ARCH A at 1/8 refused with "give a larger paper"). Same bytes twice with the same --date (cmp: identical). No fonts in the file and `pdftotext` finds 3 characters: all lettering is strokes, so no sheet can be searched, copied or read aloud; no Title/Author in `pdfinfo`. No graphic scale bar anywhere: a sheet printed "fit to page" lies about its 1/4 in and nothing on it says "print at 100%". Name rules held: `x.txt`, `x`, `x.pdf.bak` refused with words and the existing content kept; `x.PDF` accepted and written over with "wrote over the plan set that was there"; a directory named .pdf: "cannot write dd.pdf: Is a directory"; missing folder: "cannot write nope/sub/a.pdf: No such file or directory" (a software error word: "No such file or directory" is the operating system's, not a builder's). A level named by its name ("ground") works; a wrong name lists the levels. `upper` of the first L job refused: "upper has no wall on its plan to draw".

### PS2-19 (lettering height, measured, held)
600 dpi crop of the notes: cap height 62 px = 0.103 in (3/32 = 0.094). Smallest lettering anywhere reported 6.75 pt on every page of every job. Lettering is the same size at 1/8 as at 1/4 (80 ft house at 1/8: readable). The 8 ft box at 1/4: figures 7.2 pt and fine.

## What held
- Every string equals the model's plan length (chk.py on 6 jobs, all levels I wrote, 0 mismatches), including 11.9999 ft -> 12'-0", 8.0418 ft -> 8'-0 1/2", 20.0417 -> 20'-0 1/2", and the angled 11'-3 15/16".
- Overall extents right (47'-3 7/8", 20'-4 11/16", 52'-0", 60 x 46).
- Room areas against hand: Main 575.25 (575), Wing ~218 (by hand 217-218), Bed in L 885.25 (stated 885, 920 gross), Kitchen 204.19, Bath 77.75 (78: 90 less the 12.25 linen), Linen 9.0, Family 182.1, Cell 56.25 (56), the 283 SF big room 282.75.
- Existing / demolition / proposed mark logic: a wall's mark carries to its openings (the mark verb says so); an opening marked by hand overrides (door-0026 new only after `mark door new`); demolished things dashed on page 2, absent on 3; "3 WALLS COME DOWN, 52 LF; 2 DOORS; 2 WINDOWS" counted right on B (12+12+28).
- Existing/new/demolished tell apart in black and white on the demolition and proposed pages (light unfilled / dashed / grey fill); key matches those pages.
- VIF: an Assumed 8'-0 1/2" wall carries VIF and the note counts it; the overall carries VIF on both axes even when the vertical one rests only on Known walls (over-flagged, harmless).
- Same bytes on a second run; refusals above.

## As the architect: what stops me putting my name beside it
1. No location dimensions: nothing positions a partition, a door, a window, a cabinet run (PS2-10). I cannot stamp a plan that cannot be staked.
2. "?" for an ampersand or an accent in the job's name, the level and the rooms (PS2-6); my client's name misprinted on every sheet.
3. Dimension strings through the lettering and rooms on the small-room renovation sheet (PS2-9); the program says "lettering faults: 0".
4. Wall lengths on a wing at an angle with no angle, and strings that do not chain to the overall (PS2-11, 12).
5. No wall tags, room numbers, door/window schedule, swings, sections, graphic scale; sheet numbers repeat across levels (PS2-16); dimensions to wall centre lines with a note that does not say so (PS2-2).
6. Not searchable, no document title (PS2-18).

## As the builder: what I would phone and ask
- Are these centre lines or faces? (The note says "walls' lines as drawn".)
- Where exactly is the hall, and is it 3'-0" clear? Where do the closet walls stand? Where is the door in the hall wall: centre or edge, and which way does it swing?
- Which cabinet is the DB18 and what is the blank box in the bath: toilet or vanity?
- What is the Living room's area ("? SF"), and why is HALL 109 SF as it stands when the Nook is in the same space?
- Why does "FAMILY 182 SF" end where there is no wall?
- What is the angle of the wing?
- Which sheet is A-101: the ground or the upper?

## Worst first (for the hand-back)
PS2-10 (nothing locates anything), PS2-6 (& and accents printed as "?"), PS2-9 (strings and things over lettering, checker blind), PS2-11/12 (angled and L chains), PS2-8 (known fault written silently), PS2-1/13/14 (? SF unexplained, a bounded space for two rooms, the room line invisible), PS2-15/16 (key shows what is not drawn, repeated sheet numbers).
