# Judgment 20: notes as I go

Program: the frozen copy in scratchpad/bin6. CLI work in j20/cli/.

## J20-1. A footing's concrete is counted twice where one footing ends on the middle of another (T)
Did: footing 0,0 to 40,0 and footing 20,0 to 20,20, both 24 in x 12 in (j20/cli/t.json).
By hand: the long one 40 x 2 x 1 = 80 CF. The stem stands on the long one's centre line, so
its first 1 ft (y 0 to 1) is inside the long footing: real concrete of the stem 19 x 2 x 1 =
38 CF. Total 118 CF = 4.370 CY.
Program: takeoff "footings 2 60.00 LF, footing concrete 4.44 CY" (120 CF): 2 CF (0.074 CY)
counted twice, and nothing says so. The "length x section is exact" rule (call 70) only holds
for two runs that both END at a shared corner; at a T it overcounts by width/2 x width x depth.
Clashes never test footing against footing (by design), so nothing flags it.

## J20-2. A footing's type, and two refusals, speak of a wall
Did: `type footing "ftg 24x12" --width "24 in"`, then `show`.
Expected: the type's width.  Got: "type-0003  ftg 24x12  [type] / wall thickness 24 in".
Call 70 admits "thickness"; the screen says "wall thickness", a wall's word on a footing.
Refusals: `type footing --width "0 in"` -> "type-0007 wall type thickness: must be more than
zero" (and names a path, type-0007, for a type that was never made); a footing with both ends
at one point -> "footing-0007 wall line: the two ends are the same point". The brief's rule:
a footing is never called a wall. It is, three times. (j20/cli, commands in the log above.)

## J20-3. Footing quantity rounded before it is priced (a few cents per footing)
The saved job holds volume 2.963 CY for a 40 ft x 2 ft x 1 ft footing (exact 2.962963); the
estimate shows "2.9630 CY" (a fourth decimal 0 that is not the real figure) and prices the
rounded figure: demos/found.sh expects $548.16 at $185/CY where 2.962963 x 185 = $548.148 =
$548.15. Call 74's whole house does the same: 2.9630 x 450 = $1,333.35 and 2.0741 x 450 =
$933.35, four footings $4,533.40, where the concrete drawn (272 CF = 10.074074 CY) at $450 is
$4,533.33. 7 cents over on Division 3. Small, but a figure "worked to the cent" from a
rounded quantity, shown with a precision it does not have.

## J20-4. A footing measured deeper in the field: the word the job shows is refused
Did: j20/cli/g2.json (found.sh's 40 x 28 ring). `show` lists footing 1 with "depth 12 in".
`plumbline measure g2.json footing-0007 depth "18 in" --source "made-up dig"` ->
"plumbline: footing 1 has no entered value called "depth"; it has: type".
So did "footing depth". Only `measure ... height "18 in"` works, and it answers
"footing 1: height is 18 in, measured". The error lists only "type" as measurable, so a
person told what to type is told wrong. (After "height": 40x2x1.5/27 = 4.4444 CY, ring
11.56 CY, and a pipe at -5.25 ft now clashes with the deeper footing and not with the 12 in
one: the underside does move. That part holds.) `set ... height` answers "the height of a
footing is not set this way ... Only a parapet carries a height" - wall words again.

## J20-5. Removing a footing's type is refused with a wall's words
`remove g4.json type-0003` (the footing type, in use) -> "footing-0007: its type is
"type-0003", which is not a wall type in this job". Refusing is right; the reason reads as
if the type were the wrong kind. Also: a footing can be drawn on a wall type (footing on
"fdn wall 8 in concrete" took width 0.6667 ft, 0.2469 CY) and a wall on a footing type (a
24 in "ftg 24x12" wall): types are not told apart, no warning (call 70 says so).

## J20-6. The BIM file's footings (and walls) have their outside corners cut out
`export ifc g2.json`, read back with IfcOpenShell in world coordinates (feet):
footing 1 x 0..40, y -1..1; footing 2 x 39..41, y 0..28; footing 3 x 0..40, y 27..29;
footing 4 x -1..1, y 0..28. The square x 40..41, y -1..0 (and its like at each corner) is
in no footing: the ring has four 1 ft x 1 ft notches at its outside corners, and four 1 ft
squares inside counted in two footings. The volume is right (call 70's argument); the shape
another trade receives is not a footing anyone would pour. The walls do the same with 8 in
notches. IfcOpenShell finds no schema fault (it would not). The level's floor slab
(-4.333 to -4) also sits inside the footings' top 4 in along the walls (not tested, by design).

## Held (foundation, CLI)
Depth 0 and -12 in refused ("more than nothing"); --depth missing refused; width 0 and
-6 in refused; zero-length footing refused; 10" depth read as 0.8333 ft, 10x2x0.8333/27 =
0.6173 CY (agrees); remove a footing: 108 LF, 108x2/27 = 8.00 CY (agrees); move: line
follows, clash location follows; pipe through a footing at -4.5 ft reported where it is
(a footing listed under "Through a wall or a floor (a hole to cut)": heading names only
wall/floor); `check` reads back; IfcOpenShell no fault; footing depth in IFC 1.5 ft after
measure.

## Division 3 and the price, by hand (held, apart from J20-3)
`plumbline sixteen` then `estimate` (j20/cli/est.txt). Slab 1,120 SF x $6.50 = 7,280.00;
footings 2.9630x450 = 1,333.35 (x2), 2.0741x450 = 933.35 (x2) = 4,533.40; foundation
walls 40x4=160 SF and 28x4=112 SF, 544 SF x $38 = 20,672.00. Division 3 = 32,485.40,
9 lines: agrees. Divisions summed by hand: 394,677.48 = job cost: agrees. Overhead
10% = 39,467.748 -> 39,467.75; profit 8% of (cost+overhead) 434,145.228 = 34,731.618 ->
34,731.62; price 468,876.85: agrees to the cent. Recap columns 203,798.75 + 40,800 +
150,078.73 = 394,677.48 agrees. (Concrete exactly as drawn would be $4,533.33: J20-3.)

## J20-7. `finishes` (the command line's own report of what each wall is dressed in) does not
mention framing, weather barrier, flashing, base or concrete, and calls the foundation
walls "bare"
j20/cli/fin.txt: "wall 1 (wall-0289): bare, on its south face: nothing priced on it names
a cladding" for each foundation wall, while STATUS.md says a wall priced "concrete" is drawn
in concrete. The windows' flashing, the walls' barrier and frame, the rooms' base: no line of
the report names any of them, so nothing outside the window says what those layers are
drawn from.

## J20-8. With one level picked alone, the other levels' switches still read ON
s2/out/04-model.png: "first floor" clicked under Levels; only the first floor is drawn;
"second floor" and "foundation" switches both still blue (on). The fixed rule: every layer
has a switch that reads on or off at a glance. Here two read on and are not drawn.

## J20-9. The Up arrow did not tilt the view (s2/out/06 identical to 05)
Three Up presses after picking the first floor alone, pointer parked at 1000,800 (just
above the tool bar). Nothing moved. STATUS.md says this is how the woodwork pictures were
got. Retried below with the pointer over the model.

## Test job k.json (j20/cli/k.json), made to break framing, barrier, flashing, base
Level ground, 9 ft; 6 in walls round 0,0-24,16 (wall 1 south 0007, 2 east 0008, 3 north
0009, 4 west 0010), partition wall 5 (0011) at x=12, a 2 ft wall 6 (0024) at 30,0-32,0.
Openings (offset is the CENTRE along the wall): window 1 (0018) 0.25-3.25 ft, window 2
(0019) 3.5-6.5 ft (3 in apart), both sill 3 head 7; door 3 (0022) 3 ft x 9 ft in the 9 ft
wall 1 at 7.5-10.5; window 4 (0017) east wall; door 1 (0015) 3x9 north wall; door 2 (0016)
in the partition. Rooms: living (west half), bath (east half).
Lines (no prices): wall 1 "Wall framing, 2x6 at 16 in" AND "Wall framing, 2x4 at 24 in",
"Sheathing and weather barrier"; wall 2 "Wall framing 2 x 6 @ 24", "Tyvek HomeWrap";
wall 3 "2X4 studs 16 OC", "Felt paper, 15 lb"; wall 4 "wall framing 2x6 16 in o.c.",
"Vapor barrier, 6 mil poly"; wall 5 "Anchor bolts into concrete", "Partition framing, 2x4
at 16 in"; wall 6 "Wall framing, 2x6 at 16 in"; window 1 and door 1 "Flashing and sealant";
living "Base and trim"; bath "Vanity base, 36 in", "Shower base, 32 x 48".

Expected from the code's word rules (framing.rs `named`, finish.rs `barrier_in`,
`base_in`, `concrete`), before looking:
- Framing switch "3" (walls 1, 5, 6). Walls 2, 3, 4 have framing priced in words a framer
  writes every day and are NOT framed, with no word.
- Wall 1: the first framing line wins (2x6 at 16); the 2x4 at 24 line is ignored: is it said?
- Barrier: only wall 1. "Tyvek HomeWrap" and "Felt paper" (both real weather barriers) are
  not read; "Vapor barrier" correctly not read.
- Base: drawn in living (right) and ALSO in bath, from "Vanity base"/"Shower base" (wrong).
- Wall 5, a framed partition, drawn in concrete (Finishes on and off) from "Anchor bolts
  into concrete".
- Windows 1 and 2 3 in apart: window 1's right king (3.375-3.5) is window 2's left jack;
  window 2's left king (3.25-3.375) is window 1's right jack; each king runs full height
  through the other's header: lumber drawn through lumber.
- Door 3, 9 ft in a 9 ft wall: no header (no room); the plates run across its head.
