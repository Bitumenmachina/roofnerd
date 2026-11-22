# Judgment: room line (call 152), layouts side by side (154), kitchen from the item library (155)

Judge: cold, did not build it. Program: frozen bin18. Jobs are made up; work in review/jk/. Figures worked by hand (24 x 16 ft house, 6 in walls, inside faces 0.25 in from the lines).

## What held (checked by hand)
- Room line at x=12 across a 23.5 x 15.5 interior: 182.125 + 182.125; two lines: 120.125 + 124 + 120.125 = 364.25 (the whole). Wall area excludes the line. Receptacles per room by its own wall LF.
- L-shaped open plan split by a line: 112.125 + 283.125 = 395.25 (the whole). Line on level 1 does not touch level 2. Move of a line re-parts correctly (77.625 + 317.625). Remove restores the whole. check passes after each. IFC says "room line: not written". Line is no wall in takeoff.
- A room line with priced rooms: vif puts the dollars on the line (~$1,976.26, 2 lines).
- options: figures equal each job's own takeoff and estimate; allowance row matches; last-less-first works for three.
- Cabinets: run on S, N, W, E walls, partition from both sides, a diagonal wall, window run, door refusal, lower-case codes, typo (lists the codes), mixed wall and base refused, countertop 6 x 2.125 = 12.75 SF, top at 36 in, all to the hand figures. Refusals leave the file byte for byte. Item library out and back unchanged; an item's width changed on re-import moves the placed boxes ("the 2 set down are now that size") and clashes then show the 1.5 in overlaps.

## Findings

| # | Finding | Commands to reproduce | Expected by hand | What the program did | Severity |
|---|---|---|---|---|---|
| 1 | Base and trim priced on a room's perimeter counts the room line as wall | whole 24x16 house, rooms A,B, `room-line ... "8,0 8,16"`, `import assemblies starter`, `apply room-0008 "Room, hardwood floor"`, `set ... unit_cost '$2.00/LF' --line "Base and trim"`, `estimate` | A has 31 LF of wall (7.75+15.5+7.75): base 31 LF = $62 | Base and trim 46.50 LF = $93.00 (15.5 LF across the open line). Wall area and receptacles use 31 LF; perimeter does not. Over-count 50 % | Money, high |
| 2 | A room line that does not reach a wall parts nothing and says nothing | rooms A (6,8) B (18,8); `room-line "12,0.5 12,15.5"` | A message that it parts nothing, or B placed | Accepted. A = 364.25, B stays Unknown with the reason "A was placed first", which never mentions the line. Reach tolerance about 0.05 ft past the face | Layout, medium |
| 3 | A line attached at one end only (stub), or lying in a doorway, adds its length to the rooms' perimeter, not to wall area | `room-line "12,-5 12,5"` on one room; or `"11,6.5 13,6.5"` across a partition door | No change to the rooms | Perimeter 78.5 to 87.5 (and +19 with a second stub); doorway line +1.5 LF on each room; wall area unchanged. Same root as 1 | Money, medium |
| 4 | Duplicate or exactly overlapping room lines, and a line lying along a wall, are accepted | same line twice; `"0,0 24,0"` | Refused or said | Taken; two lines listed, vif asks two questions with no dollars if no room is priced | Low |
| 5 | Room point lying on the room line is silently given to one side | `room-line "0,8 24,8"` with room at (12,8) | Said, or Unknown | Takes one half, no note | Low |
| 6 | VIF for a room line says only "Check its line." and, in the window list, "check: ~line" | `vif`, `export vif` | A builder's question (where the line stands, whether a wall is wanted) | Vague; "~line" is a code, not a word | Language, low |
| 7 | options: column heads are the job titles; two layouts of one job both read "t" (or collide with "less t") | `options i.json i2.json` | The file name or A, B, C | "t t t less t"; a long title swamps the table | Layout, medium |
| 8 | options "job cost" leaves out lines with no figure and does not say | i2.json: room B Unknown, its flooring line has no quantity; `options i.json i2.json` | A flag that 1 line is left out (the estimate says "leaves out 1 line") | "~$1,976.25 ... -$0.01" as a plain comparison | Money, medium |
| 9 | options shows no price (overhead and profit), only job cost | i4.json with overhead 10, profit 8: PRICE $2,347.80 | The difference in price shown or said to be left out | "same" | Money, low (by claim: job cost) |
| 10 | Wrong number of files dumps the whole usage text | `options a.json` | One line | 70 lines of usage | Low |
| 11 | A second run over an earlier run is accepted with no word | `cabinets ... --from "3 in" --boxes "B18 SB36"` twice | Refusal or note | Both set, stacked. Clashes finds them afterward (33 on my test) | Layout, medium |
| 12 | Two runs meeting at an inside corner overlap and the program does not say | S wall B36 from 3 in; W wall `--from "12.75 ft" --boxes B36` | A note, or the second run starts 2 ft 3 in off the first | Both set ("0 ft left to the corner"); only `clashes` shows "in each other by 24.00 in" | Layout, medium |
| 13 | Wall box set in front of a window; base or tall box in front of a low or floor-to-ceiling window | window 3x4 ft sill 3 ft on the north wall, `cabinets --from "8.5 ft" --boxes W3030`; or sill 2 ft with B36; or sill 0 with T2484 | Refused or flagged (a wall box hangs 54-84 in; window 36-84 in) | Taken, no word; clashes never tests a window | Layout, medium |
| 14 | Boxes keep standing and stay priced after their wall is removed or marked to demolish | run on a partition, `mark demolish`; or `remove wall-0006` | A word, a clash or a question | Boxes float, no clash, vif only asks about position; remove prints "removed window, wall" and says nothing of 6 boxes | Layout and money, medium |
| 15 | Refusal for a run on a demolished wall gives the wrong reason | `mark ... demolish`, then `cabinets` on it | "that wall is coming down" | "it has no face on that room's side" | Words, low |
| 16 | Removing a box leaves the countertop at full length over the gap; the countertop is not tied to its run | run of B18 SB36 B18 with `--countertop`; `remove` an end box | Shorter top, or a note | 12.75 SF stays | Money, medium |
| 17 | Boxes must be priced one at a time; a second B18 does not take the first's price; no way to price by code | `set furnishing-0059 unit_cost '$180/EA' --line B18`; `estimate by-item` | One price per code | "B18 ... 1 of 2 priced"; a 20-box kitchen is 20 commands; division amount shows ~$180.00 without a left-out note | Money, medium |
| 18 | Countertop front edge (LF) is in the message only; no figure on the thing to price; no price line made (estimate does say "1 furnishing priced on nothing") | `show furnishing-0062` | edge 6 LF available | width 6 ft only | Low |
| 19 | Supplier list says "How sure: Known" for boxes placed Assumed with sizes Assumed | `export supplier` on the kitchen | Assumed | Known | Honesty, low to medium |
| 20 | Run with `--from 0` is refused (face begins 3 in in); no way to start from the far end of a wall | `--from "0 ft"` | clear message | "clear from 0' 3\" to 23' 9\"" (fine), but the far end needs hand arithmetic from the wall's direction | Low |

## As a kitchen designer: first things I would miss
1. Corners: no blind-corner or lazy-susan base, no wall corner, no rule that the second run starts 24 in off the first (finding 12). First miss.
2. Fillers: only F3 (3 in); no 1 to 6 in fillers, no end panels, no toe-kick.
3. Appliances in the run: the range, dishwasher and refrigerator exist as items but cannot go in `--boxes`; a gap must be left by hand with a second `--from`. A sink base does not know the sink.
4. Island: a run needs a wall; an island is `place` at x,y one box at a time, with no countertop.
5. Range of boxes: no 39, 42 or 45 in bases, no drawer banks beyond DB18/24, no 36 or 42 in high walls, no 30 in or 36 in pantries or oven cabinets; wall boxes are fixed at 54 in up.
6. Windows and doors are not checked against boxes except a door under a run (finding 13); a sink base under a window is taken but a wall box over it is too.
7. Price once per code (finding 17).
