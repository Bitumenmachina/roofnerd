# Judgment 26 part A: notes, written as I go

Job: JDIR/job/farm.json, made with `plumbline new` (the only command-line step that changes it).

## Path so far (no stops yet)
- Add level: clear; fields NAME / ELEVATION / FLOOR-TO-FLOOR with "like 0 ft" hints. Added "First floor".
- Add drawing: file list with usda-7213 PDF; picked; "it has 4 pages" -> Page 4. Set scale -> clicked the two ends of the 41'-3" overall dimension, typed 41' 3". Screen: "41' 3" between the two points; the drawing is 88' 4 5/16" wide". (Checked by hand: the page is 10.96 in at 1/8" = 1'-0" -> about 87.7 ft. Agrees.)
- My own slip: first two sessions ended without F2, so the level / drawing were not kept. Not counted against the program (the top bar said "Not saved").

- Walls: Wall tool, thickness empty on first pick ("like 6 in"); Enter alone refused with "Thickness is still to be typed". Typed 6 in, clicked the SW corner, typed 41' 3" toward the east, clicked the other corners: 4 walls, 139.17 LF (hand: 2 x (41.25 + 28.333) = 139.17). Two inside walls at 4 in: 32' 6" and 18' 9". Total 190.42 LF. Agrees.
- Doors 3'0" and 2'8" (height offered 6'8"), windows 3x4 ft sill 3 ft (window fields empty, all three must be typed). Floor: one click fills inside the walls: 1,168.75 SF = 41.25 x 28.333 (to the wall centre lines, not the outside face; no word on screen which).
- Rooms: Living room 462.92 SF (hand: 25.25 x 18.333 = 462.9, inside faces). My third room fell in the same open space as the second; the screen said so plainly ("It has no area yet: Kitchen and dining was placed first...") and the Job panel says "no total, 1 of 3 missing". Held.
- Roof: corners clicked 1 ft outside the walls, 4:12, eaves 9 ft -> "Added roof: 4 planes". 1,379.08 SF, 13.79 SQ, eave 147 LF, hip 87.90 LF, ridge 13 LF. Hand: 43.25 x 30.25 x 1.0541 = 1,379.1 SF; ridge 43.25 - 30.25 = 13; hips 4 x 21.98 = 87.9. Agrees.

- Pricing: Starter assemblies -> "Brought in 30 starter assemblies ... Click one of those on the model, then Assembly." Wall 4 -> Assembly -> "Exterior wall, 2x6, lap siding": 9 lines. Prices typed on the sheet's Unit cost fields; production rates typed "40 SF/hr" and "0.02 hr/SF"; wages "48" and "52/hr" both read as $/hr. Saved job: material $2,301.35, labor $571.20 (11.475 hr), job cost $2,872.55. Hand: 255x2.10=535.50, 8x26=208, 255x.22=56.10, 255x.85=216.75, 255x4.25=1,083.75, 8x14=112, 255x.35=89.25 => 2,301.35; 255/40=6.375 hr x 48 = 306.00; 255x.02=5.1 hr x 52 = 265.20. Agrees to the cent.

- Overhead 10 %, profit 8 % typed on the sheet: overhead $287.26, profit $252.78, price $3,412.59 (hand: 2,872.55 x .10 = 287.255; (2,872.55 + 287.26) x .08 = 252.78; sum 3,412.59). Top bar shows the same.

### J26A-1 (words) Overhead and profit are said to be "not yet checked against a drawing"
Kind: words. The Job panel, under Overhead 10.00 % and Profit 8.00 %, reads "entered on screen; not yet checked against a drawing" (out/s29/01-model.png). A markup is never on a drawing; an estimator reads this as the program not knowing what overhead is.

- Roof: plane 1 -> Assembly -> Asphalt shingle roof: "5 cost lines added. So are plane 2, plane 3 and plane 4: the same lines on each." A price typed on plane 1 is offered to the same line on the other planes, and the screen says so. Labor: 60 SF/hr at $48, 1.5 hr/SQ at $50. Saved: material $4,885.51, labor $2,708.78 (55.146 hr), job cost $7,594.29, OH $759.43, profit $668.30, price $9,022.02. Hand-worked plane by plane: roof material 1,012 + 206.86 + 1,365.30 = 2,584.16, + walls 2,301.35 = 4,885.51. Agrees.

### J26A-2 (wrong money, small) Roof sheathing is rounded up to whole sheets plane by plane
Kind: wrong money. 1,379.08 SF of roof needs 43.1 -> 44 sheets; the sheet carries 15 + 8 + 15 + 8 = 46 (each plane rounded up on its own: 448.40/32 = 14.01 -> 15; 241.14/32 = 7.54 -> 8). Two sheets ($44 at my $22) too many, and nothing on the line says the rounding is per plane. The same would hit gypsum and wall sheathing wall by wall.

### J26A-3 (wrong money) 1,168.75 SF x $6.00 comes to $7,012.49 on screen and in the saved job
Kind: wrong money. Floor 1, Allowance, $/SF, typed 6. The floor shows 1,168.75 SF (41' 3" x 28' 4" = 1,168.75 exactly). The top bar says "allowances $7,012.49 not in price" (out/s37/02-model.png) and `plumbline estimate` says $7,012.49. By hand 1,168.75 x 6 = $7,012.50. The saved job holds the area as 1168.7486 SF (`plumbline show`): the wall corners are kept to four decimals (28' 4" stored as 28.3333), so a quantity shown as 1,168.75 multiplies as 1,168.7486. Same seed shows on wall 2 and wall 4: face area 254.9997 SF. An estimator checking a line with a calculator gets a different cent from the screen.

### J26A-4 (words / money) "Allowance, $/SF" is not carried in the price
Kind: words. The screen says plainly "Floor 1: allowance set to $6.00. It is the budget's figure and is not in the price: give this cost lines ... to carry it in", and the top bar shows "allowances $7,012.49 not in price". Said, so not silent. But to an estimator an allowance ($6/SF flooring allowance) is a figure carried in the bid; here it is a budget figure kept out of the price. The button's name invites the wrong reading; the message corrects it after the fact.

### J26A-5 (breaks) A comma cannot be typed in a cost line's name: it jumps to the next field
Kind: breaks. Door 1 -> Add cost line -> WHAT IT IS: typed "Exterior door, prehung", Tab, "EA", Tab, "material", Enter. The comma moved the cursor to PRICED BY, so the fields read "Exterior door" / "prehungmaterial" / "EA", and the screen refused: "Can't add the cost line: "EA" is not a cost type" (out/s40/01-model.png). The program's own lines are written with commas ("Wall framing, 2x6 at 16 in", "Gypsum board, hung and finished"); an estimator writes them the same way and cannot. Nothing on screen says a comma acts as Tab.

