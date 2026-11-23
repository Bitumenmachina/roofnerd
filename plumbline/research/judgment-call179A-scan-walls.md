# scan-walls-2: second judgment of walls, doors and windows found in a scan

Judge: scan-walls-2. Command line only. Program: the frozen copy at `review/bin27/plumbline` (the brief named
`bin26`, which was deleted from the machine mid-run; I re-ran the first house on `bin27` and the output was identical,
so every figure below is from bin27; I cannot say its commit. wtA HEAD moved to 76ffd81 meanwhile).
My generator: `judges/scan-walls-2/gen.py`, `scenes.py`, `big.py`, `run.py`, `fnd.sh` (all made-up; faces and boxes in feet,
100 points to the square foot unless said, 1.5 cm scatter, 1 stray in 50, half a degree of lean, as the builder's own).
Every house is worked by hand in the comment beside it in `scenes.py` / `big.py`.
A "finding" is reproduced by `python3 run.py <scene>` (or `python3 big.py`) then `./fnd.sh <scene>` in that folder.

## The big house (40 x 28 ft, 8 ft): 4 outside walls (one face each), hall 4 ft wide between two 4 1/2 in partitions, living/kitchen,
bed 1 with a closet (3 1/2 in walls), bath with a tub, bed 2. Hand answer: 10 walls (4 outside, N-S x=26, N-S x=30, E-W y=14,
E-W y=20, closet E-W y=9.5 with a door, closet N-S x=36), 7 doors (one 6 ft cased opening), 3 windows (the north one behind cabinets).

(The brief's frozen copy `bin26` was gone partway; all runs below are bin27. Seeds: scenes use seed 1 unless said; `big.py` takes a seed.)

Held on the big house, all three seeds, unturned and turned 33 and 45 degrees: the four outside walls (one face each) at the right places and
lengths (40, 28, 40, 28 ft to the inch), the 4 1/2 in partitions paired and 4 1/2 in thick, the hall 4 ft wide kept as two walls (14' 4" for 14' 4 1/2"),
the 6 ft cased opening found as 6' 0" (not split in two), the 3 ft doors found as 2' 11" to 3' 1" wide head 6' 8", the window behind the
cabinets said "not seen for 4' 0" ... taken as wall, to be checked" (not silently dropped, not laid as a window). Laid: 8 walls, 5 doors, 2 windows, every
one Assumed, sourced to the scan, none Known (I searched the saved job: the only Known values are mine, the level's, and six 0 SF "opening area"
formulas that are the model's own sums). `check` clean; rooms by hand: Living 25.81 x 28 = 722.75 SF (said 722.60), Hall 3.625 x 28 = 101.5 (said 101.49), Bed 2
9.81 x 7.81 = 76.66 (said 76.86): all within the nearness. `clashes`, `carries`, `vif` ran and read sensibly.

## Findings (worst first)

### 1. A wall found twice: a face is paired into a wall AND laid again alone, on top of it. Silent, adds quantity. (wrong quantity, unsaid)
Reproduce: `python3 -c "import ew1; ew1.big(1).save('ew1.ply')"` (the big house with its east wall left out and the tub against the partition at y=14;
`ew1.py` is in my folder) then `./fnd.sh ew1`. Seed 1 and seed 2 do it; seeds 3, 4, 5 do not (2 of 5).
Said: `wall 5: from 29' 10" east, 14' 0" north to 39' 11" east, 14' 0" north, 10' 1" long, 0' 4 1/2" thick, seen from both sides`
and `wall 8: from 34' 9" east, 14' 2" north to 39' 0" east, 14' 2" north, 4' 3" long, seen from one side (its thickness is not in the scan)`.
Right: one wall, 10 ft, 4 1/2 in. Wall 8 is the partition's north face again. `--lay --outside "<a 6 in type>"` lays both without a word
(wall-0015 at y 14.00, 0.375 ft thick; wall-0025 at y 14.43, 0.5 ft thick, lying 2.4 in off it and overlapping it for 4.2 ft). `check` says "reads, its figures agree";
`clashes` "walls are not tested against each other". The takeoff counts 4.23 LF and about 34 SF of face that is not there. The model refuses a wall exactly along another
("lies along") but not one 2.4 in beside and inside the first. Nothing said; "the walls found do not close" even says 4 ends, none of them this.
Same family: the 3 1/2 in closet wall seen as two one-faced walls 4 in apart (`./fnd.sh p2b` : wall 6 at 35' 10", wall 8 at 36' 2", each "seen from one side") and the program then
says **"the walls found close: every end meets another wall"** (wall 8's north end is 1 ft 6 in short of the partition it should meet; each of the two "meets" the other).

### 2. A 10 ft partition with a door, whole, is dropped, and the only word is "7 not straight along the whole length (furniture, or two walls close together)". (a wall dropped, said but not placed)
Reproduce: `python3 big.py big1.ply` (also seeds 2, 3 via `big.big(2).save(...)`), `./fnd.sh big1`. Hand answer 10 walls; found 8: the partition at y=14 (x 30 to 40, with a 3 ft
door at 31 to 34) and the closet's 4 ft wall with a door (two jambs of 1 ft and 6 in) are missing. All three seeds drop the partition; take the furniture away (or move it: `dv1.py`, `dv3.py`) and
it is found. Said: `left out: 10 stretches of upright points not taken as walls: 3 under 2 ft long ..., 7 not straight along the whole length (furniture, or two walls close together)`;
in seed 1 also `a face 6' 1" long at 32' 10" east, 14' 0" north reaches the ceiling and not the floor, and nothing stands before it: it is not taken as a wall (something hung from the ceiling, or a wall seen only above)`.
The person is not told WHERE the seven are, so bed 1 and bath become one room (`room ... Bath --at 35,17` then reads `unknown` on every figure, and `takeoff` says "1 of 5 missing", honest, but
nothing connects it to the lost partition). The walls "close" test does not catch it (the lost wall's ends meet other walls). The closet door wall's "3 under 2 ft long" is the same: unplaced.
This is the "dropped without a word" class: a word, but not one a builder can act on. The same sentence describes a curved wall (finding 5).

### 3. A wall laid where there is none, no doubt said: a soffit over cabinets. (wall laid where there is none)
`./fnd.sh soffit` (20 x 12 room, base cabinets 0 to 3 ft, wall seen between, a soffit 2 ft deep from 7 to 8 ft over 14 ft of them). Said: `wall 4: from 2' 9" east, 10' 0" north to 17' 3" east, 10' 0" north, 14' 6" long, seen from one side`
(true soffit 3 to 17 ft; it is not a wall at all) and `the walls found do not close: 2 ends meet nothing`. Call 179's rule (A3) puts "a wall or something tall standing at a wall" in doubt, and lays only with `--take`; this
one has its two ends free and a wall behind it and is NOT in doubt: `--lay --outside` laid it (wall-0007, 6 in thick, 14.5 ft, floating 2 ft from the north wall). Said only by the "do not close".
Same with a fireplace: `./fnd.sh fire` : the front face of a 5 ft by 2 ft bump to the ceiling is found as wall 4 (5' 3" long) and laid as a wall 6 in thick, floating 2 ft off the east wall;
its two 2 ft returns are "under 2 ft long". Said: "do not close" only.

### 4. A mirror (or glass) doubles the house, and nothing says so. (wall laid where there is none; "the walls close" while they do not)
`./fnd.sh mirror` (a 20 x 12 room; the whole point set repeated beyond the east wall, x' = 40 - x, as a mirror on one wall does to a scanner). Found: 5 walls: south and north each 40 ft, a wall at x=20
and one at x=40 (**is the 20 x 12 room**: true 4 walls, 64 LF; laid 118.5 LF, 948 SF of face). Said: `the walls found close: every end meets another wall`. Nothing about points beyond a wall that repeat the room. (The "neighbour's wall
10 ft outside seen through a window" was held: `./fnd.sh neighbor` finds 4 walls and the window, and does not lay the neighbour; but it never says it saw something beyond the window and left it out.)

### 5. A curved wall is laid as three straight walls, and the word for it is "furniture, or two walls close together". (misleading)
`./fnd.sh curved` (a quarter circle of 8 ft radius, 4 1/2 in thick, 12.6 ft long). Found `wall 4` 2' 8" long, `wall 5` 4' 4", `wall 6` 4' 5", each "seen from one side" (the outer face, r 8.19 ft, is not found at all), gaps of
about 6 in between and short of both ends; `--lay` lays three 6 in walls. Said: `left out: 5 stretches ... 5 not straight along the whole length (furniture, or two walls close together)`. Nothing says "a curve". A builder reading "3 walls"
would draw 3 walls. It does say "do not close" (1 end). Not a wrong figure on its own, but the call's rule is that a wall found is never laid leaning; here a curve is laid as chords and described as furniture.

### 6. A small room's walls laid up to 15 in short, no word; and "the ceiling slopes" said of a flat ceiling. (wrong height unsaid; a false statement)
`./fnd.sh closet6` (a 6 ft by 3 ft closet, 8 ft, flat). Seed 1: `the ceiling slopes, from 6' 8" to 8' 1"` and walls `8' 1" high`, `7' 3" high`, `8' 0" high`, `8' 1" high`.
Seeds 3 and 4 (`python3 run.py closet6 "dict(seed=3)"`): no word about a slope, walls `7' 3" high` and `6' 9" high`, with the same scan reading 8 ft floor to ceiling in its first line. A wall 15 in short of the ceiling it stands under is a quantity error
of about 17 SF on a 6 ft wall, and the sentence that would warn (call 179 A8: the level's height against the scan's) does not fire because the floor-to-ceiling is read right. Also `./fnd.sh dens`
(west half 100 points to the square foot, east half 10): says `the ceiling slopes, from 7' 3" to 8' 1"` of a flat ceiling. A person told "slopes" will go and look for a slope.

### 7. Density ten to one across the house: 2 of 5 walls found; the others, including the whole east wall and the 40 ft south and north walls, vanish. (walls dropped; said, thinly)
`./fnd.sh dens` (west 20 ft at 100 to the square foot, east 20 ft at 10, same scan). Found wall 1 (x=0) and wall 2 (x=20 the partition, but ONE of its walls, laid `seen from one side`, its other face and its 4 1/2 in lost); south, north and east walls lost. Said: "the scan holds about 17 points to the square foot", "thinner than the measures here were made for
... check each" and "do not close: 4 ends meet nothing". That is said, not silent, but the person is told "check each" when most are not there; and the wrong "ceiling slopes" (finding 6) is also said.

### 8. A garage wall with a 16 ft door is dropped, and the reason given is wrong. (a wall dropped, said wrongly)
`./fnd.sh garage` (24 x 22, 9 ft ceiling, south wall with a 16 ft door x 4 to 20, head 7 ft; east wall with an 8 ft door, found: `doorway: 6' 0" to 14' 0" ... 8' 0" wide, head 7' 0"`). The south wall is NOT found: 3 walls; said `a face 24' 0" long at 12' 0" east, 0' 0" north reaches the ceiling and not the floor, and nothing stands before it: it is not taken as a wall (something hung from the ceiling, or a wall seen only above)`. A wall with a door 16 ft wide is a wall. It is said, and the room "does not close" (2 ends), so not silent, but the reason is a builder's wrong word, and the 16 ft door (the one the estimate most needs) is lost. 
(The same sentence correctly describes a beam: `./fnd.sh basement`, the 12 in beam, found as two faces "hung from the ceiling". Held. The posts and the hung duct are not said at all; "nothing stood short of the ceiling" is false there.)

### 9. Two storeys: duplicate walls found, a door of the upper floor called a window. (misleading)
`./fnd.sh storey` (two floors of points, 9 ft apart): refused first, well: `this is not likely the house: check the measure and which way is up (floor to ceiling reads 17' 0", over 15' 0")`. But it does not say "two storeys"; after `--anyway`: 6 walls found, wall 2 and wall 4 are the same wall (both (0,0) to (24,0)), and the upper floor's door is
`window: 5' 0" to 8' 0" from its south end, 3' 0" wide, sill 9' 3", head 15' 8"` on the same wall as the ground floor's door. The duplicate is named and not laid ("lies along ... counted twice"); the "window" is laid. (Same duplicates in `stairfix`.)

### 10. A 2 in / 4 in furring, and a seam, are absorbed without a word; a free-standing stub is said to be "open". (untidy)
Furred 4 in over 10 ft of a 16 ft wall (`furred4`): the furring is found as `wall 4: 29' 8" east, 2' 1" to 13' 3" north, 11' 2" long` in doubt (good, said), but it is 14 in longer than the 10 ft furring. Furred 2 in (`furred2b`, `furred`): the wall is taken at the furring, 1 in short over its whole length, not said.
Seam (`seam`, the second half 2 in and half a degree out): north wall `15' 11" ... to 16' 2"` over 40 ft, partition `5 3/4 in` (true face to face 6.5 in), "the walls found close". Within "a few inches". The program lays one straight line through a jog and does not say how far the points lie from it.
A tub alcove's two 2 1/2 ft stubs (`tub`) are found 2' 4" and 2' 3" (3 in short at the free end) and "the walls found do not close: 2 ends meet nothing, so the room is open there" is said of their free ends, which are not open.
A bay with 45 degree walls, 6 faces 2 ft 10 in and 4 ft long (`bay`): all 9 found, all to the inch, "close". Held.

### 11. Words. (words)
- `a thing is given by its path, like wall-0004` and `"ground" is the name of level-0002: give level-0002` (`path`). `find-walls --level ground` takes the name but `floor`, `room`, `show` want level-0002: the refusals say so, but the reader meets "path".
- `plumbline floor <file> level-0002 "Slab 4 in" --walls` : `Slab 4 in is not a floor type`, for a floor type I just made by that name ("type-0026"); with type-0026 it works. Wrong refusal (a name and a mark).
- `vif`: `check: ~type, ~wall line; measure: ~height` and `measure: ~center, along the wall, ~sill`: tildes for Assumed with no legend on that page; `wall 1 [wall-0004]` raw identifiers in a sentence. "wall line", "center, along the wall" are not a builder's words.
- `Nope is not a wall type with a thickness`: does not list the types the job has.
- `find-walls` and its sentences otherwise read as a builder's: "doorway", "not seen for 4' 0" ... taken as wall, to be checked", "stood short of the ceiling". The `made for them, Assumed, from the scan: the type "wall 5.75 in"`: a type per odd thickness (a seam makes one).

## What I tried that held
- 40 x 28 house, 5 rooms, hall 4 ft wide, 3 1/2 and 4 1/2 in walls, a 6 ft cased opening, island, peninsula, upper cabinets, sofa, tub, bay and 45 degree house: walls, doors, windows within the stated nearness, nothing Known.
- Turned 33 and 45 degrees (the 45 degree house found 9 of 10 walls; the 33 degree 8: same drop as unturned).
- Walls 120 x 60 ft, 2.0 million points: 7 of 7 walls, 6 doors, 3.4 s. 6 x 6 ft room: 4 of 4. 4 ft clear hall: two walls, right. A missing whole east wall: "4 ends meet nothing" said, east wall not invented.
- Tray ceiling (8 ft with a 9 ft 6 in tray): "the ceiling is not one height: 8' 0" and 9' 6"", four walls, the three step faces named "reaches the ceiling and not the floor", not laid. Beam (basement) not laid, said. Pony walls and pass-through: the 3 ft 6 in pony walls are "stood short, tallest 3' 6"", the 4 ft pass-through is a window of 3' 11" x sill 3' 0".
- A scan read in feet that is meters: refused: "not likely the house ... the whole scan is only 2' 6" high". Floor-to-ceiling 17 ft: refused.
- Laid jobs: `check` clean, saving reads back the same; laying twice refused ("would make two of each").
