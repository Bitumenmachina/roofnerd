# Judgment 21: notes as I go

Program: frozen copy bin15. CLI work in j21/cli/.

## J21-1. Footing concrete still counted twice at a cross and where three footings meet at one point (J20-1 fixed only for the plain T)
Did (cli/B.json, cli/G.json; all 24 in x 12 in at -4 ft):
- B, a cross: run 0,0-40,0 and a footing 20,-10 to 20,10 straight through it. By hand: 80 + 40 - 4 (the 2 x 2 x 1
  square both hold) = 116 CF = 4.296 CY. Program: takeoff "footing concrete 4.44 CY", the crossing footing 1.4815 CY
  (20 x 2 / 27, nothing taken off). 4 CF (0.148 CY) counted twice, nothing says so.
- G, three meet at a point: 0,0-20,0, 20,0-40,0 and a stem 20,0-20,20. By hand 80 + 38 = 118 CF = 4.370 CY (the same
  concrete as the T that was fixed). Program: 4.44 CY, the stem 1.4815. The fix reads "stops on the run of another"
  only when the point is in the middle of one footing, not where two footings butt there.
Held: the plain T (4.37), a stem starting at the run's edge 20,1 (4.37, 59 LF), a stem starting 6 in inside the run
(4.37), a 12 in stem on a 24 in run (99 CF = 3.67, agrees), a stem on each side at one point (156 CF = 5.78, agrees).

## J21-2. A footing type with no footing on it yet is still listed as "wall thickness" (J20-2 fixed only once used)
Did: new job, `type t.json footing "ftg 24x12" --width "24 in"`, then `show`. Got
"type-0003  ftg 24x12  [type] / wall thickness 24 in". After one footing is drawn on it the same type reads
"footing width 24 in". The type was made with the verb `type ... footing`, so the program knows what it is
from the first moment; the wall word is what a person sees right after making it. (cli/t.json; demos/found.sh only
checks after footings are drawn.)

## J21-3. A footing that stops part way across another, past its centre line, is still counted twice over part of it
Did: cli/mv.json, the T of cli/t.json with the stem moved to x=25 then 0.5 ft south: stem (25,-0.5) to (25,19.5),
run (0,0)-(40,0) 24 in wide (y -1 to 1). By hand: 1.5 ft of the stem lies in the run, so the stem's own concrete is
18.5 x 2 x 1 = 37 CF, total 117 CF = 4.333 CY. Program: stem 1.4444 CY (39 CF: 19.5 ft), takeoff 4.41 CY. It took
off only the 0.5 ft between the stem's end and the run's centre line. (A stem ending 0.5 ft short of the centre
line, cli/D.json, is right.) Moved 5 ft north, off the run, it gives 1.4815 and 4.44 (agrees).

## J21-4. A stem deeper than the run it stops on loses the concrete under the run
Did: cli/n.json: the T, stem measured 18 in deep, run 12 in. By hand: run 80 CF; stem 19 x 2 x 1.5 = 57 CF plus the
part of the stem under the run, 1 x 2 x 0.5 = 1 CF; 138 CF = 5.111 CY. Program: stem 2.1111 CY (57 CF), takeoff
5.07 CY (137 CF). 1 CF of concrete under the run is dropped, nothing says so. (A run deeper than the stem,
cli/m.json: 120 + 38 = 158 CF = 5.85 CY, agrees.)

## J21-5. Removing a footing type in use: the refusal is untrue
`remove rm.json type-0003` (the footing type, still under footing 2) -> "footing-0005: its type is "type-0003",
which is not a footing type in this job". It IS a footing type in this job (show lists it as "footing width 24 in").
The refusal is right; its reason is false. J20-5 said the same with "wall": the word changed, the untruth did not.
(cli/rm.json)

## J21-6. Smaller footing words still left
- `measure <footing> height "18 in"` is still taken and answers "height is 18 in, measured": a wall's word on a
  footing, as a hidden second name for depth (J20-4: depth now works; height was not taken away).
- Refusals still name a path for a thing never made: "type-0006 footing type width: must be more than zero",
  "footing-0006 footing line: the two ends are the same point" (J20-2's second point).
- `measure <footing> width` refused, listing "type, footing line, depth": a footing dug wider than drawn cannot be
  recorded on the footing; only by changing the type for every footing on it.
- A footing 1000 ft deep under a 4 ft level is taken without a word (740.74 CY).

## J20-6 rechecked: not fixed (and said open in STATUS.md)
cli/r.ifc, a 40 x 28 ring of 24 in footings, read back with IfcOpenShell in world coordinates: footing 1 x 0..40 ft,
y -1..1; footing 2 x 39..41, y 0..28: the 1 ft square x 40..41, y -1..0 is in no footing (a notch at each outside
corner). The slab drawn on the level (-4.333 to -4 ft) lies inside the footings' top 4 in; no clash and not said.
IfcOpenShell: no schema fault.

## Held (foundation, CLI)
The plain T 4.37 CY; a stem from the run's edge; a 12 in stem on a 24 in run; a stem each side; 10" depth 0.6173 CY;
1/2 in depth 0.0309 CY; depth 0, -12 in, a missing width, width 0 and -6 in, a zero-length footing all refused;
measure depth 18 in -> 4.4444 CY, takeoff 5.85 (120 + 38 = 158 CF, agrees); remove a footing: 20 LF 1.48 CY; move 5 ft
east keeps the T figure; a pipe through a footing is "In the same space: 1 ... at (10, 0, -4.5)" and "a hole to cut:
0" (J20's point fixed); `floor --walls` on footings only refused in plain words (footings are not taken for walls);
IFC: 4 IfcFooting STRIP_FOOTING, 0 IfcWall, no fault.

## J21-7. Taking something out is read as putting it in: a wall is drawn in a weather barrier, or in concrete, because a line priced on it removes one
Did: cli/w.json and cli/w2.json, one 10 ft wall per line, then `finishes` (the word rules the window draws from).
- "Remove house wrap", "Demo existing house wrap", "Tear off felt paper" -> "a weather barrier under its finish".
- "Remove existing concrete wall" -> "concrete, as a line priced on it says".
A remodel estimate is full of these lines; the house is then drawn wrapped (or in concrete) where the work is to
strip it. (Framing held: "Remove existing 2x4 wall framing" is not framed, but only because it names no spacing.)
Files: cli/map.txt + cli/wf.txt, cli/map2.txt + cli/wf2.txt.

## J21-8. Weather-barrier and framing words: what is missed, and missed silently
- Read: Tyvek, felt paper, Housewrap, Typar, WRB, Water-resistive barrier, Sheathing and weather barrier. Not read:
  "Weather-resistive barrier" (the common trade name; only "water-" is), "Asphalt felt, 30 lb" (felt, but not
  "felt paper"), "Zip System sheathing, taped" (a sheathing with its barrier on it). "vapor barrier, 6 mil",
  "ice and water shield", "Moisture barrier" correctly not read. "Air barrier" IS read as a weather barrier (an
  air barrier is often the drywall inside; not the same thing). "Housewrap tape at seams" alone draws the whole wrap.
- Framing: "2x4 framing" (no spacing) is not framed and `finishes` says nothing about it: a framing line that is
  on the wall and not drawn is not said. Same for "Wall framing, 2x6 at 400 in" and "...at 0 in" (silently not
  framed, rightly refused, never said). "Wall framing, 2x6 at 16 in - NOT INCLUDED" IS framed.
- Held: "Wall framing, 2x6 at 19.2 in", "Exterior wall framing 2x6 24\" OC", "2x6 studs @ 16\" o.c.",
  "Framing 2x8 at 12 in", J20's "2 x 6 @ 24", "2X4 studs 16 OC", "wall framing 2x6 16 in o.c." all read; "Metal stud
  framing", "Blocking, 2x4 at 48 in", "Furring strips, 1x3 at 16 in", "Headers, 2x10" not framed (right).
- J20's k.json rerun (cli/k.json): every prediction of J20 now goes the right way (Tyvek, felt read; vapor barrier not;
  bath's vanity/shower base no base; anchor bolts not concrete; the two framing lines on wall 1 are now said:
  "2 lines on it name a framing: the first is the one drawn").

## Part 6: the whole house's price (cli/h.json, cli/est.txt) - held, apart from J20-3
By hand: Division 3 = slab 1,120 SF x 6.50 = 7,280.00; footings 2.9630 x 450 = 1,333.35 (x2) and 2.0741 x 450 =
933.345 -> 933.35 (x2) = 4,533.40; foundation walls 160 + 112 + 160 + 112 = 544 SF x 38 = 20,672.00; total
32,485.40 in 9 lines: agrees. Sixteen divisions summed: 394,677.48 = job cost; recap 203,798.75 + 40,800.00 +
142,828.73 + 7,250.00 = 394,677.48 agrees. Overhead 10 % = 39,467.748 -> 39,467.75; profit 8 % of 434,145.23 =
34,731.62; price 468,876.85: agrees to the cent. Every one of the 399 lines' amount = shown quantity x unit price
rounded half up: 0 differences (script over est.txt).
J20-3 not fixed (STATUS says held for Q25): the concrete as drawn, 272 CF = 10.074074 CY x 450 = 4,533.33; priced
4,533.40. Same quantity shown two ways on one room: office "Hardwood flooring 179.9414 SF" and "Ceiling ... 179.94 SF".

## J21-9. `finishes` calls a concrete foundation wall "bare" in the same sentence that says it is concrete
cli/hfin.txt: "wall 1 (wall-0289): bare, on its south face: nothing priced on it names a cladding; concrete, as a line
priced on it says". J20-7's addition is there (framed / weather barrier / concrete / flashed / base are now said: 16
framed walls, 8 barriers, 17 flashed openings, 10 rooms with base), but the sentence still starts "bare".

## Window, the whole house (wa/out, session wa/s.sh) - what held
- 01 as opened: price $468,876.85 = the command line; the foundation level's panel "Footings 4 136.00 LF, Footing
  concrete 10.07 CY", Walls 4 136.00 LF, 544.00 SF: agrees with `takeoff`. Switch counts: Finishes 8 (= 8 walls with a
  barrier in `finishes`), Framing 16 (= 16 "framed in"), and they agree with cli/hfin.txt.
- 02 Finishes off: walls pale (barrier); each window has a dark band round its frame (wa/z02.png); the front door has
  none, and no door line prices flashing (17 "Flashing and sealant", all on windows): only where priced. Price same.
- 03/04 Framing on: both storeys as stud walls with headers and sills; the foundation walls (no framing line) are
  not framed. Price same in every picture.
- 05 Ground off: the view eases down; grey concrete foundation walls under the stone, the footing's lip at the
  bottom, clear of the tool bar. (A short cyan stub floats in the air left of the house at about 455,597: a pipe
  running out under where the ground was; it reads as something loose.)
- 06 first floor picked: J20-8 FIXED: second floor, foundation and Roof switches all read off. The view tilts down
  into the floor. The near corner is under the tool bar (known).
- 07 Up x3 with the pointer parked at 1000,800: the view tilts (07 differs from 06): J20-9 FIXED. 08 two more with
  the pointer over the model: tilts further.
- The base is a thin brown line at the foot of the walls (wa/z06.png): visible only when you know to look.

## J21-10. With both levels on show, the plan writes no room's name or area at all (call 81)
Did: cli/b.json (an L-shaped "living" with a partition and a "closet" on ground; a "loft" on upper), the window on
plan. Ground alone (wa/out2/03-model.png): "Living 223 SF" inside the L, "Closet 52 SF" (CLI: 223.25, 52.25: agree).
Upper alone (04): "Loft 185 SF" (185.25, agrees). All levels (wa/out2/05-model.png): both levels drawn one over the
other and NOT ONE room name or area: Living, Closet and Loft all gone, although every level is on show. Call 81
says "each room on a level that is on show now carries its name". The panel on the right still reads "LEVEL upper"
after "All levels" was pressed. Also on that picture the ground floor's south door is hidden under the upper's
south wall: two storeys stacked on one plan with nothing saying which line is which floor.
(Note: wa/out2/01 and 02 were overwritten by the second window of that session: each start of the program numbers
its pictures from 01 again. Not a program fault; the framing pictures are retaken in wd/.)

## J21-11. The room base has no switch of its own on a job with no wall finish priced
wa/out2/02-model.png: b.json, ground alone in 3D. The base is drawn (a brown strip along the living room's walls, none
in the closet: right), but Show lists only Architecture, See through and Known/Assumed/Unknown. Call 73 says the
base goes off with Finishes; Finishes is offered only where a wall has a finish or barrier. So here the base layer
cannot be put away except with its whole level.

## J21-12. From an empty job, Wall picked and its sizes "accepted" with Enter: what a person types next goes into the thickness, and the wall type saved is not what the field showed (wc/out, wc/bars.png, wc/status.png, wc/e.json)
Did (wc/s.sh): empty job, Plan, Add level "Ground, 0 ft, 9 ft", Wall. The tool bar shows THICKNESS empty with the
cursor in it and HEIGHT "9 ft" in grey (01). Enter, as the brief says, then a click for the first corner: "Nothing
was placed: the wall's sizes are still open. Still to be typed: Height. Press Enter first, then click." (02) It
names Height, which shows 9 ft, and not Thickness, which is empty and has the cursor. Then the first length,
20' 6", typed with the pointer east: it goes into THICKNESS, which shows "20' 6"" (02). Enter. The next length,
12 ft, goes into HEIGHT. The bar then reads THICKNESS "2 ft", HEIGHT "12 ft" (03). Saved job: type-0003 "wall 2 ft",
wall thickness 2 ft; two walls 12 ft high. 20' 6" was on screen in the thickness field; 2 ft was kept (the first digit
of what was typed; see J21-13). A grey "9 ft" that is not a value, a message that names the wrong field, and a
typed figure kept as its first digit: the first five seconds of a first wall.
Then: a length typed (8 ft) and a click where Enter belongs: nothing placed, no word, the 8 ft stays in the field (05).
A length typed then Tab: the wall is placed at the typed 6' 0" (06, "Added wall 1, 6' 0" long": fine). A length typed,
Esc ("Cancelled."), then a click: a wall of 18' 10" is placed to the click (09). "Cancelled" read as the wall
cancelled; the start point was kept.
Door, picked next (10): it OFFERS a width of 0' 6" ("Door: this is what it will place"). Nobody typed a door width;
0' 6" is the tail of the "20' 6"" typed earlier. A door width typed and a click with no Enter: "Nothing was placed:
the door's sizes are still open. Press Enter first, then click." (11: held). Window (12): WIDTH empty, HEIGHT and
SILL grey 3 ft.

## J21-13. Reproduced: the wall's thickness is replaced by the first digit of the next length typed, and the tail of that length becomes the next door's width (we/bars.png, we/status.png, we/e2.json)
Did (we/s.sh), empty job: Add level; Wall; typed thickness "6 in" (01: THICKNESS "6 in", HEIGHT a grey "9 ft"); Enter:
the cursor goes to HEIGHT and the grey 9 ft is gone: "Wall: Height is still to be typed" (02). Enter again: the same
(03): the 9 ft that was on screen cannot be accepted. A click: "Nothing was placed..." Then, as a person who thinks the
wall is in hand, the first length 20' 6" typed: it goes into HEIGHT, which shows 20' 6" while THICKNESS still shows
6 in (04). Enter: THICKNESS now reads "2 ft", HEIGHT "20' 6"", "Wall in hand" (05). Nobody typed 2 ft. The saved job:
type-0003 "wall 2 ft", thickness 2 ft. Wall picked again offers 2 ft / 20' 6" (06). Door picked: it offers WIDTH
"0' 6"" (08), exactly as in wc/: the "2" went to the wall thickness and the "0' 6"" to the door. Same result twice in
two sessions. (Typing as the kit's `len` types: the first digit, a pause, the rest.)
The planw fix (STATUS 2026-11-13) holds on the path it was proven on; this is the same family of fault on the path
just beside it: the first wall of a new job, where the sizes are not yet in hand.
Also: a grey "9 ft" (and the window's grey "3 ft" / "3 ft") is a hint that looks like a value; Enter does not take it.

## J21-14. A wall type with nothing on it yet is taken for a footing's type (J20-5 fixed only by first use)
Did: cli/ty.json: `type wall "fdn wall 8" --thickness "8 in"` (type-0006, made as a WALL type), then
`footing ... type-0006 --points "50,0 60,0" --depth "12 in"` -> "1 footing: footing-0007", taken. From then on the wall
type refuses walls: "type-0006 is the type of footing-0007, a footing: a wall needs a type of its own (make one with
`type wall`)" - it was made with `type wall`. The type is told apart by what was first drawn on it, not by what it was
made as (the reverse, a footing type with nothing on it taking a wall, is the same rule). demos/found.sh tests it only
with a type already in use. (Removing that footing type in use: "footing-0007: its type is "type-0006", which is not a
footing type in this job" - J21-5 again.)
Converse confirmed (cli/ty2.json): `type footing "ftg 36" --width "36 in"`, then `wall ... type-0006` -> "1 wall:
wall-0007", a 36 in wall on a type made as a footing's; the type now lists "wall thickness 36 in"; takeoff counts it as
a wall (walls 1, 10.00 LF, 40.00 SF). The program was told which kind each type is when it was made, and forgets it.

## Part 5 held: the brief's path, once the wall's sizes are in hand (wf/out1, wf/bars.png, wf/status.png, wf/e3.json)
Empty job; Add level; Wall: "6 in" Tab "9 ft" Enter ("Wall in hand"), Esc. Wall picked again: "this is what it will
place" 6 in / 9 ft (03). Enter (accepted, not typed over), a corner, then four lengths typed: 20' 6", 12 ft, 20' 6",
12 ft. Saved: wall 1 20.5 ft; wall 2 12.0 ft; wall 3 20.5 ft; wall 4 12.0 ft (each worked from the saved ends). No
first digit lost: the planw fix holds on this path. (Walls 2-4 are off square because my pointer was not square:
the length follows the typing, the direction the pointer; "Shift keeps it square" says so.) Door: first pick shows
WIDTH empty, HEIGHT 6' 8", SILL 0 ft; typed 3 ft, placed: 3 ft. Picked again: offers 3 ft / 6' 8" / 0 ft; Enter;
placed: 3 ft. Typed over 2' 8": placed 2.6667 ft. Window: offered 4 ft / 3 ft / 3 ft after typing; my clicks for the
windows fell off the skewed walls, so no window was placed: the window half of part 5 is not proven by me.

## J21-15. "To verify" on the job's panel lists office work as field questions
wf/out2/01-model.png (cli/f.json): under TO VERIFY: "2 for the same reason. The first: Living, cost code: unknown /
not coded" and "Closet, unit cost: Vanity base, 36 in: unknown / no price entered". A cost code and a price are not
verified in the field; they are entered at the desk. A veteran reads "to verify" as "go and measure". (The job had
no "N to verify" button in its top bar, though the panel lists three; on the whole house the bar says "93 to verify".)

## Part 8 held so far
By division head on a job whose only priced line has no division (wf/out2/02-sheet.png): "0 of 16 with work in them; 1
line of the job cost is in no division" (call 78): right. Building tab (03-sheet): the L-shaped room "living" 66.00
LF round, 240.00 SF to wall centres, 223.25 SF inside; loft 58 LF, 185.25 SF: each worked by hand and agrees. A
drawing whose file is not beside the job: a plain message saying what to do (01). Level with nothing drawn (attic):
the model panel lists Walls 0 none, Wall face none, Floor 0 none, Doors and windows 0 doors, 0 windows (not red).

## J21-16. The command line never says a drawing's scale as a scale (call 80 is the window only)
cli/d2.json runs: `calibrate ... --distance "30 ft"` over 540 points (7 1/2 in of paper) is 1/4" = 1'-0" exactly; the
program says "30 ft is 540 on its page: 18 to the foot" and `drawings` "scale: 30 ft between (126, 90) and (666, 90) on
its page: 18 to the foot". 18 what? (points). 31 ft: "17.4194 to the foot" where the window is to say 1" = 4' 1 5/8".
A builder reads neither.

## J21-17. A level with only footings, or nothing, has its zero walls flagged Unknown at the command line
`takeoff` (cli/r.json, cli/f.json attic): "? walls 0 none", "? wall face, gross none", "? floors 0 none". A level with no
wall has, Known, no wall. The window's panel shows the same rows without the flag (wf/out2/01). Two answers for one
figure. Small.

## J21-18. A footing's panel shows a quantity that does not multiply to its own extension
wg/o3/02-model.png (the whole house, footing 1 picked on the foundation plan): COST LINES "Concrete footing, in place:
Unit cost $450.00/CY, Quantity 2.96 CY, Extension $1,333.35". 2.96 x 450 = $1,332.00. The figure priced is 2.9630
(J20-3), the figure shown 2.96. A builder checking the line with a calculator is $1.35 out and nothing on the panel
says why. Also TAKEOFF "Depth 1.00 ft - its depth, as entered" where 12 in was entered.

## J21-19. On the foundation plan the roof's hips and outline are drawn with the Roof switch off
wg/o3/01-model.png: foundation picked alone on plan; Show: Roof 4 reads OFF; the plan carries the hip roof's dashed
outline, four dashed hips and the dashed ridge over the foundation walls and footings. Either the switch is wrong or
the lines are; a foundation plan with the roof's hips on it is not a drawing a builder would read.

## Window held (G)
- Scale said as a scale (wg/o2/02-model.png): a PDF calibrated at 31 ft over 540 points reads `1" = 4' 1 1/2"` under
  the drawing's name. By hand 31 / 7.5 = 4.1333 ft = 4' 1.6" (nearest 1/8: 4' 1 5/8"); the half-inch rounding is
  about 3/4 in over the 31 ft dimension. Acceptable; noted.
- With All levels on show on plan the room names are gone again (same picture): J21-10 confirmed on a second job.
- A typed price said back as money: "Closet: unit cost of "Vanity base, 36 in" set to $185.00." (wg/o2/03-sheet.png);
  job cost 495 + 185 = $680.00 agrees; "2 lines of the job cost are in no division" agrees.
- The footing's panel (wg/o3/02): FOOTING, Footing line, Depth, Width "the width its type gives", Volume "plan length
  x width x depth / 27" = 2.96 CY: in a footing's words throughout; no "wall".
- Base, close (wg/o4/02-model.png, cli/b.json): along the foot of each wall of the L-shaped living room, round its
  inside corner, stopping at the partition door; none in the closet ("Vanity base" is not base). Not floating, not
  outside. (The part of the partition south of the door is under the tool bar: not seen.) A faint dashed line runs
  across the living room floor in 3D with nothing to say what it is.
- Building tab's foundation rows: below the window's fold; not looked at.

## Framing in the window (wg/o1/01-model.png, wg/w1.png, wg/w2.png; cli/fr2.json)
Switch reads "Framing 5": walls 1-5 carry "Wall framing, 2x6 at 16 in", wall 6 only "Drywall": 5 is right, and wall 6
is drawn as a plain wall, not framed (held). Wall 1 (12 ft, 3 ft x 4 ft window 54-90 in, sill 36, head 84; call 71's own
case) worked by hand: whole studs at 0, 16, 32, 48 in (48 stands 2 1/4 in off the king), 96, 112, 128 and the end
stud at 142 1/2; a king and jack each side; header 2x6 39 in long; one rough sill; two cripples above (15 1/2 in) and
two below (33 in). The picture (w1.png, about 16 dots to the foot) shows: 3 + (stud, king, jack bunched) left, two
cripples over a deep header, two under a single sill, 3 studs + end right. As close as the picture allows: agrees.
Wall 2 (two 2 ft windows overlapping, 1 in and 0 in from the first end; the job flags their areas Unknown and says
why: held): at this size the pieces at the first end cannot be told apart (w2.png). By the code (framing.rs: an
opening is clamped to start 3 in from the wall's end), a window 1 in from the end has its jack and king drawn inside
the window's opening by 2 in. Not seen; a prediction from the code, like J20's.
Close-up attempts (wh/o, wh/m25.png): the wheel at a wall dived past it and ended among studs; pressing F (Frame)
after each did not bring back the whole job's view: four of five pictures are of lumber filling the window. Looking
closely at one wall's frame took me more than a person would give it.

## J21-20. A framed wall's own panel says "Bare, on both faces"
wi/o1/01-model.png: wall 3 of cli/fr2.json double-clicked with Framing on, drawn as studs and plates; its panel's
FINISH reads "Bare, on both faces ... nothing priced on it names a cladding". "Wall framing, 2x6 at 16 in" is on it and
is what is drawn; `finishes` at the command line says "framed in 2x6 at 16 in on centre" but the panel does not. The
same for a wall's weather barrier: the panel's FINISH speaks only of cladding.

## J21-21. Looking closely at a frame: a double-click between studs lands the camera inside the lumber
wi/o2/01-model.png: with Framing on, a double-click at a point of wall 3 that fell between two studs selected nothing
("Opened.") and flew the view into the frames: the window filled with studs, nothing legible. With the frames drawn,
most of a wall is gaps; the "double-click flies to a thing" rule then flies to the ground behind. (wi/o1: a
double-click that did hit a stud selected wall 3 and moved closer: fine.)
Not seen in any picture I could get: the 9 ft door in the 9 ft wall (wall 3) - by the code it has no header and its
top plates run across the top 3 in of the door's opening, and nothing says the door is as tall as its wall.
