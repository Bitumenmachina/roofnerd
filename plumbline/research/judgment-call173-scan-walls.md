# scan-walls: judge's notes on find-walls (call 173), frozen program 8627a83, command line only

Everything below was run from an empty job (`$B new`, `$B level ... --status Assumed`, `$B type ... wall`), on scans I
generated myself with `/tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/judges/scan-walls/gen.py`
(PLY, ascii, meters, z up; walls as true two-sided or one-sided faces, 1.5 cm scatter, 2 % strays; every scene `tN.py`
in that folder, every scan `*.ply`, every job `lay-*.json`). `run.sh <name> <ply> [flags]` runs find-walls on a new job.
Hand figures are the generator's own numbers (all in feet). Every result is repeatable (the finder is deterministic);
each was run once per scan unless said.

What the generator does not do (so the findings are a floor, not a ceiling): no lean, no occlusion shading except where I
cut a gap by hand, no real-scan irregularity.

## Findings, worst first

### 1. A wall missed, and the page says "furniture" (confident and false): sparse scans, and a long house
`gen('S..', house 30x16 + partition, dens=N)` with N = points per SF per surface.
- 10, 20, 30 per SF (19.5k, 39.8k, 59.7k points): `walls found: 0, 0 of them seen from one side only` and
  `stood short of the ceiling, and not walls: 2 things (cabinets, furniture); the tallest is 7' 10"`. The five walls of
  the house are called cabinets and furniture. Then `nothing is laid`. It never says "too few points to tell".
- 40 per SF, 60 ft house (LONG.ply, 135k points) and 40 ft house at 40/SF (L40s.ply, 90k): same: 0 walls, "3 things".
- 60 per SF: all five found, right. So the cliff is between 40 and 60 per SF (about 430 to 650 points per m2 of
  surface). Dense scans of 100+ per SF hold (the builder's one).
Right answer: either the walls, or "too few points to tell a wall from furniture: nothing found". Wrong: the sentence
that says the walls are cabinets. How bad: wrong quantity (house vanishes) with a misleading explanation. Note the
cliff is sharp: from 5 walls to 0 with no middle.

### 2. A wall hidden behind tall furniture silently disappears; the room is not said to be open
`t9.py`: a 30x16 room, tall cabinets (7 ft) hiding N % of the north wall (HID30..HID90.ply).
- 45 % hidden and up: `walls found: 3`, north wall absent, `2 things (cabinets, furniture)`. Nothing says the
  four walls do not close, or that something stands where a wall must be. (30 % hidden: the wall is found, but see 3.)
- PACK2.ply (20x14 room, cabinets against the south, north and west walls, wall behind them unseen to 6 ft 6 in):
  walls found: 2 (south, east). North and west walls lost without a word; south wall found only from its top foot and
  laid 3 in off (`from 0' 4" east, 0' 0" north to 19' 9" east, -0' 1" north`; truth 0' 3" north). Beyond the 1 in
  the demonstration promises, and said nothing about.
How bad: wrong quantity (walls missing from the takeoff), and nothing in the output tells the estimator.

### 3. A message gives a wrong figure
HID30.ply: the north wall is 9 ft hidden by cabinets. The page says: `left out: a gap 15 in wide in a wall runs from the
floor to the ceiling: it is not taken as an opening, and the wall is laid across it`. There is no 15 in gap; 9 ft is
hidden, and the "not seen for 9' 0"" sentence the program has for this does not appear. Also the north wall is 2 in out
of square (`15' 9" north ... 15' 11" north`). How bad: misleading.

### 4. Tall furniture 7 ft 9 in is laid as a wall across the room (the builder admitted it; reproduced and laid)
TALL.ply: 10 ft long cabinet bank 2 ft deep, top 7.8 ft, ceiling 8 ft. Output wall 4: `from 1' 10" east, 12' 0" north to
12' 2" east, 12' 0" north, 10' 4" long, seen from one side`. `--lay --outside <type>` lays it (wall-0007, 6 in thick,
at y 12.25) with no remark. Real: no wall. How bad: wrong money (10 LF of wall, 83 SF of face), a wall where none is,
nothing marks it as the doubtful one. It is the same word ("seen from one side") as every true outside wall.

### 5. Sloped ceiling: "no ceiling shows" is false, and the answer is nothing
SLOPE.ply (ceiling 8 ft at the south wall rising to 12 ft at the north). `left out: no ceiling shows in the scan ... nothing
found`. A ceiling shows; it is not level. Result is honest (nothing found, said so) but the reason is wrong, and a
builder with a vaulted ceiling is told his scan has no ceiling.

### 6. Two ceiling heights: the low room's walls are called furniture
TWOCEIL.ply (west room 8 ft, east room 10 ft, partition 8 ft). `floor to ceiling reads about 10' 0"` (the higher),
`walls found: 1` (the east wall only), `2 things (cabinets, furniture); the tallest is 8' 5"`. Three walls of the house,
including the whole west room, are "furniture". Said, in the sense that "left out" lists things; not said that the
ceiling read is one of two.

### 7. A real closet, and a 3 1/2 in wall next to a 12 in wall, give phantom walls
C.ply (20x12 room, closet of two 3 1/2 in walls 2 ft apart plus a 2 ft cross wall, all scanned both faces): found
`wall 3 ... 3 1/2 in thick, seen from both sides` (the y=6 wall: right), then the y=8 wall as TWO one-sided walls, one of them
leaning (`from 0' 3" east, 8' 0" north to 8' 3" east, 7' 9" north`, 3 in off over 8 ft; `squared: 7 of 8` says
one face was not square), and the 2 ft cross wall not found at all (a face under 2 ft is dropped, and not counted; the
builder said so, but the page does not). D.ply (30x16, 12 in wall at 10 ft, 3 1/2 in wall at 20 ft): the 12 in wall is right
(`1' 0" thick`); the 3 1/2 in wall comes out as three one-sided walls at 19' 10"/20' 2" (leaning 0.8 deg) plus a
7' 4" phantom (`from 19' 10" east, 4' 6" north to 20' 3" east, 11' 10" north`) which is no wall.
The same 3 1/2 in wall alone (D1.ply) is found right, and so is 4 1/2 in beside the 12 in (D2.ply): the failure needs both
close in one scan; it is a fragile fit. Result of laying: see 8.

### 8. `--lay` is all or nothing, and the refusal does not say which walls
C.ply and D.ply with `--lay --outside <6 in type>`: `plumbline: the new wall: lies along the new wall for 8.02 ft: one wall
drawn over another would be counted twice` (D: 7.33 ft). Nothing laid (the job is untouched: good). But the sentence names
no wall (it says "the new wall" twice), does not say it is two found walls that overlap, which they are, and does not say what
to do. The 4 good walls of the same scan cannot be laid at all. Same shape against a hand-drawn wall already on the level:
`the new wall: lies along wall-0006 for 36.00 ft` (names only the old one).
How bad: misleading, and a person is stuck.

### 9. A thick wall (18 in) is laid as two walls, doubling its length
T18.ply: 18 in wall scanned both faces. Found as two walls "seen from one side" at 14' 3" and 15' 9", each 15' 6" long. Laid
with `--outside` they become two 6 in walls 6 in apart: 31 LF, where the house has one 15.5 LF wall. 12 in is paired
properly (and 14 in `1' 2" thick`); above 16 in not. It says "seen from one side" for both although the two faces face each
other and are 18 in apart; nothing suggests they may be one wall.

### 10. Wrong unit or axis: a confident, nonsense house
- `--up y` on a z-up scan: `floor to ceiling reads about 15' 6"`, 6 walls found "-8' 0" north" and so on, nothing
  said. Laid, they would be 15' 6" high.
- `--units ft` on a meter scan (house 3.3 times too small): `no ceiling shows in the scan ... nothing found`: no hint that
  the scan is too small to be a house.
No plausibility check on the floor-to-ceiling (15 ft 6 in, 2 ft 5 in). How bad: misleading (the first would be priced).

### 11. A Known level height disagrees with the scan, and nothing says so
Level made `--height "9 ft" --status Known --source tape`; the scan reads 8 ft. `find-walls ... --lay` lays all walls 8 ft
(96 in, Assumed) on a level the job says is 9 ft Known; wall face areas are on 8 ft. The only line is `floor to ceiling reads
about 8' 0"`. (import scan does say "it is not given to the level".) No check says a 1 ft disagreement with a measured value.

### 12. `measure ... same` turns a scan's Assumed into Known with no measuring
`$B measure lay-A2.json wall-0004 "wall line" same --source "found in scan A"` => wall line becomes `Known`, source
`found in scan A; found as drawn (found in scan A)`; the same for `height` with source `scan`. So the scan can
promote itself to Known, with the scan named as the source of the measurement, and the program accepts it. The brief asks
whether anything found can become Known without a person measuring it: yes, by naming the scan as who measured it. The
source string is only required to be non-empty. (It does stop an empty source.) How bad: a Known on a harness's say-so,
CLAUDE.md "Flag, never fabricate".

### 13. Smaller
- Each door's type is made from its own head to the inch: the same 3 ft doors read 6' 7" and 6' 8" and give two types
  (`door 36 in by 79 in`, `door 36 in by 80 in`). Tidy-up, and prices of a "type" split by an inch of noise.
- Doors count `doors 2, windows 0` on the takeoff with no `~`: the row has no Assumed mark although both doors are
  Assumed from a scan (walls and rooms right above it carry `~`). Check how this row reads to a person: Known.
- Wall ends carried past the face: `wall 1: from -0' 4" east ... to 30' 4" east` (about 4 in beyond a 30' 0" face
  to face wall at the corners: it reaches the centre line; a negative feet-and-inches `-0' 4"` is a word no builder writes).
- Used `home` language: `0' 6" thick`, `0' 4 1/2" thick` (a builder says 6 in, 4 1/2 in).
- `the 1 wall seen from one side are laid only as a wall type...` (grammar, when there is one).
- `nothing is laid` / `walls found: 0` with no why when 0 found.
- Raw ids where a name is wanted: `type-0003`, `wall-0008`; `no element ground ... a thing is given by its path` (path is
  a software word) when `--level ground` is typed (the level's name, the natural thing to type, is refused).
- "PLY", "z up", "taken as meters with z up", "points" are software words on a screen a builder reads.
- A scan taken away from the origin is laid thousands of feet from it (`from 4050' 5" east, -1847' 1" north`), with no
  remark. (It says it is laid in the scan's own place.)
- Found stretches of wall 1 of the closet (C) and PACK2 are off the face by 3 in; the demonstration's promise is 1 in.

## What held (tried, and right)
- L-shaped house (6 faces): 6 walls, faces and lengths within 1 in; room area 501.26 SF against my hand figure 501.25 SF
  (offset of the 528 SF polygon by 3 in); turned 22 deg also 501.26.
- Three rooms, two partitions 4 1/2 in (A.ply): 6 walls, partitions `0' 4 1/2" thick, seen from both sides`, doorways
  4' 9"-7' 9" and 7' 9"-10' 9" (truth 5-8 ft, 8-11 ft from the wall's end), 3' 0" wide. Laid: 538.63 SF rooms against my
  hand figure 538.6 (179.2 + 180.2 + 179.2). `check` said "reads, its figures agree". Turned 37 deg: same.
- 12 in masonry, 3 1/2, 4, 5 1/2, 6 1/2, 8, 10 and 14 in partitions each found and measured to the half inch.
- Two doors and two windows in one wall (E.ply): door 2' 4" wide read `2' 4"`, 3' 0" door, window sill 1' 0" head 5' 0"
  width 5' 1", narrow tall window `2' 1" wide, sill 2' 0", head 7' 3"` (truth 2 ft, sill 2, head 7): all within 3 in.
- Noise 0.5 cm and 3 cm; ceiling at 9 ft; a scan in feet (`--units ft`); a 60 ft house at 60 to 150 per SF; an empty
  room; a room with furniture; a stairwell with 12 treads (walls right, treads ignored): all right.
- Packed room with furniture 6.5 ft tall in front of intact walls (PACK.ply): four walls found.
- Laying: every wall, door, type is Assumed with source `found in scan <name>`; none Known (checked in the saved job JSON).
  Second laying onto the same level refused, a different level accepted. Area figures, `check`, `takeoff` and an
  assembly (`Exterior wall, 2x6, lap siding` on walls 1, 2, 3, 6): 288.01 SF face on 36.0015 LF x 8 ft, 31 studs, 8 plates,
  `?` unpriced lines flagged; `estimate` shows "no price entered" everywhere (honest: nothing silently zero).
- Wrong flags: `--units feet`, `--units in`, no `--level`, a non-PLY file, a missing file, `--outside` with a bad type, with a
  nonexistent type: each refused with a sentence that says what is wrong.
