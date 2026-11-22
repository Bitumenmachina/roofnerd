# Judgment: call 157 (command line) and the window pictures (calls 149, 153, 156)

Judge: cold, did not build it. Program: frozen bin19. Jobs made up; work in review/jc/. Figures by hand (24 x 16 ft house, 6 in walls, inside faces 0.25 ft from the lines, so a 23.5 x 15.5 interior).

## Part 1 notes, written as I went

### Earlier findings re-run (judgment-roomline-layouts-cabinets.md)
- 1 base on a room line: FIXED. Room A beside a line at x=12: perimeter 54.5, wall run 39 (54.5 less 15.5), base 39 LF x $2 = $78 as by hand. Perimeter unchanged.
- 2 line that reaches nothing: FIXED (said at once and again in `check`). Extended with a chain of lines: parts correctly (L 180.125 + R 88.125 SF).
- 3 stub: perimeter still counts a stub both sides (said, not fixed; wall run and wall area right). 4 duplicate / overlapping / along-a-wall lines: STILL TAKEN, no word. 5 room point on a line: STILL given to one side, no word. 6 vif "Check its line." / "check: ~line": STILL.
- 7 options heads: FIXED (title plus file name). 8 left-out row: FIXED. 9 no price (overhead, profit) row: STILL (job cost only). 10 wrong count: FIXED (one sentence).
- 11, 12 runs over runs / corner: FIXED (refused, naming what and by how much). 13 windows: FIXED at placing (sill 34 in refused, 34.5 in taken; wall box refused 4.5 ft sill, taken at 7 ft). See new finding: a box moved or window added later is not tested.
- 14 wall removed or demolished with boxes: FIXED (names 8 things standing against it, nothing moved). 15 reason: FIXED. 16 countertop after a box is removed: STILL (declared not in the call), and nothing is said when the box is removed.
- 17 price by code: FIXED (`price-item`); countertop cannot be priced that way. 19 supplier "How sure": FIXED (Assumed). 18 front edge LF not on a thing: STILL.
- Every refusal I tried (19 of them) left the file byte for byte (md5 equal).

### What held by hand (new)
- wall_run: one 3 ft door in a partition 54 -> 51 (both rooms); two overlapping doors counted once (union): 50.75; a door whose edge reaches into the corner counts only the part inside the face (door 0.1..3.1 ft: 54-2.85 = 51.15); door straddling a room line: 1.5 off each room (39 -> 37.5); a door at the line's end counts for one room only; a door to demolish is no doorway (48.15 -> 51.15) and an existing one is (back to 48.15); door across a corner into a partition 45.4 (2.75 off); island of four walls: floor 178.25 -> 167, perimeter 54 -> 68, wall run 48.15 -> 62.15 (+14), wall area +126; L-shaped room 78 perimeter, 78 run, 72 with two doors; L with line 44 and 28 (each less its door and the 7.5 LF line); partition marked to demolish: rooms merge, run 78, B Unknown with reason; both partition demolished and a line standing in its place: 39 each. Windows leave wall run alone (sill 3 ft).
- cabinets: runs touching end to end taken ("7.5 ft to 7.5 ft") and 0.01 ft overlap refused; BLB36 corner: other run must end 24 in off (2.25 ft accepted, 2.2 refused by 0.6 in); LS36: 36 in off (0.6 in refused); appliance first, middle, last in a run all stand right (range 26 in deep back to wall); countertop over a dishwasher 4.25 SF = 2 x 2.125; fridge first then 4 boxes: top 11.5 x 2.125 = 24.4375 SF; run split round a range as two runs: 9.5625 and 7.4375 SF, no clash; wall boxes over a range taken at 54 in, refused over a fridge or tall box.
- price-item: $180 on 6 B18 and says "6"; bad prices refused (negative, free, wrong unit, letter O for zero); lower case code taken; Unknown refused. Estimate by item and the supplier's list agree; division amounts carry "~" and a left-out line.

## Part 1 table: what I found past the last judge (all in review/jc/, bin19; hand figures in brackets)

| # | Finding | Commands | Expected by hand | What the program did | Severity |
|---|---|---|---|---|---|
| A | A job applied before call 157 keeps base on `perimeter` for ever, silently; `check` on it fails with "figures ... do not agree with what is drawn" and no reason | open jk/bb.json (made by bin18), `estimate`; `check`; `apply ... --change` | Base 31 LF ($62) | Base stays 46.5 LF ($93); `check` rc=1; `apply --change` says "already ...". Only `set <room> quantity wall_run --line "Base and trim"` fixes it (then 31 LF) | Money, medium (+50 %) for every old job |
| B | A window with sill 0 (glass to the floor) does not shorten wall run | whole-house kitchen, `window ... type 3x4 --sill "0 ft"`, `show` | 78 - 3 = 75 LF | 78 LF | Money, low |
| C | Kitchen assembly: base is laid behind cabinets, and the assembly's own lump lines "Cabinets", "Countertops", "Kitchen appliances" (1 EA each) sit beside the per-box lines | `import assemblies starter`, `apply room "Kitchen"` on a room with a 14.5 ft + 7 ft run | Base less the 21.5 ft behind boxes; one way to price the cabinets | Base 78 LF; three extra 1 EA lines, no word they overlap the boxes | Money, medium (double count if both priced) |
| D | Corner box LS36 (36 deep) under a 25 1/2 in top: top neither covers the box nor turns the corner | `cabinets S wall --from "0.25 ft" --boxes "LS36 SB36" --countertop`; same on the W wall from 9.75 ft | 3x3 + 3x2.125 = 15.375 SF | 12.75 SF, two separate tops; BLB36 and LS36 clear distances (24, 36 in) are right | Money low-medium |
| E | Range, refrigerator or a tall box in the middle of a run with `--countertop`: whole run refused, "set the countertop by hand" (nothing in the program does that) | `--boxes "B18 SB36 [Range, 30 in] B24" --countertop` | Two tops, or one with a gap | Refused, file unchanged. Two runs round the range work: 9.5625 + 7.4375 SF, no clash. Not one top, not a gap: the answer to "two or one" is "neither, refused" | Layout, low |
| F | Box moved, or a window added or moved, after a run: no word. `clashes` tests no window | run B27 at 0.25..2.5 ft; `window` sill 0 at 4 ft; `move furnishing --by "1,0"`; or `window` over placed boxes | Flag or refuse as at placing | Taken; `clashes` "0"; `vif` says nothing of the window | Layout, medium |
| G | Countertop cannot be priced by `price-item` ("not an item"); it has no cost line; edge LF only in the message | `price-item p.json Countertop '$45' ...`; then `line ... plan_area` and `set ... unit_cost '$45/SF'` | One price once | Works by hand: 24.4375 SF x $45 = $1,099.69 (right); one command per top, not in by-item or supplier list until a line is made (the estimate does say "nothing is priced on 2 furnishings") | Money, low-medium |
| H | Remove or move a box: the countertop stays full length and priced | `remove furnishing-<B24 mid-run>`; `remove` end box | Shorter top or a note | Top 24.4375 SF, $1,099.69, unchanged; remove names the box's own $210 and says nothing of the top | Money, medium (declared "not in this call") |
| I | Room line: a wall marked to demolish after a line met it leaves a stub with no word; stub counted on both sides in perimeter; duplicate lines, a line on a wall, a room point on a line still silent; message says "meets no wall" when it meets a wall to demolish | `room-line "0,8 12,8"` on a house with partition; `mark partition demolish` | Say it parts nothing now; perimeter 78 | Perimeter 101.5 (+23.5), wall run 78, no message; `vif` "check: ~line" | Layout low-medium |
| J | Words: refusal text for any overlap ends with the inside-corner sentence (wall box over tall box has no corner); typo list ("not in the tool chest") leaves out appliances; "tool chest"; "it has no \"header\""; options "not worked" | various | Builder's words | as left | Words, low |
| K | Boxes accepted on a temporary wall; an 84 in tall box in a 6 ft level; `mark` demolish on a wall says its existing door "stays existing to remain" | `cabinets` on `mark ... temporary`; `measure level height 6'` | A word | none | Low |
| L | options still shows job cost only (no overhead or profit), and job cost reads "no figure" where divisions read "none" | `options p2.json p3.json k.json` | A price row or "price left out" | as left (left-out row and heads from call 157 do work: $5,090.19 vs $5,390.19, difference $300 = SB36 priced) | Money low |

Re-run OK: all 19 refusals byte for byte; supplier list "Assumed" for boxes placed Known with Assumed sizes; `export estimate` csv Base 37.5 LF x $2 = $75 and xlsx formulas written.

## Part 2: the window, picture by picture
(Pictures are stale: renow/ was taken 2026-11-21 23:39, scanw/ 01:49, roomlinew/ and cabinetsw/ and layoutsw/ 03:29-03:37, all before call 157 (04:14). The window does not yet have call 157: `--boxes` splits on spaces so no appliance in brackets, the codes shown stop at F3 (no BLB36, LS36), the Layouts table has no "left out of the job cost" row and heads are titles only.)

| Picture | What is wrong | How bad |
|---|---|---|
| renow 01 (3D, brick house) | Brick wall and cream box read well. "BIM model" button; "Designer's model"; "Known, Assumed, Unknown" switch unexplained; "Walls 5 96 LF"; sheet shows Job cost "none" in red beside "no figure" | words low |
| renow 02 plan | One room "Living 178 SF" ok; partition drawn as a wall. Fine | none |
| renow 03, 04 (partition to demolish) | The partition is a pale pink bar; no key says pink means "to demolish". Living goes 178 -> 364 SF and receptacles 5 -> 7 with no word. "Wall 5 (wall 5): to demolish" says the name twice. Left panel still says "Walls 5 96 LF" (the whole as it stands; no built / existing / demolish split in the window) | confusing, medium |
| renow 05 | Refusal sentence is clear and the two buttons come up, but the same sentence is shown three times (card, status line cut off, panel) and "Wall 1 (wall 1)". "Keep its lines priced, as work done to it" is right trade language | low |
| renow 06, 07 | "Existing" lit; on the plan the south wall washed pale is almost the same as before (cannot be told apart from new at a glance); sheet recap shows nothing of existing or demolished | medium |
| renow 08, 09 | Demolition switch hides and returns the pink bar; legible. Switch is under Show with no word of what it does | low |
| renow 10 (3D) | South wall brick turns plain gray (the finish the estimate prices vanishes); partition is a faint pink triangle on the far wall, hard to read as "a wall coming down" | medium |
| scanw 01 | "Scan file" sits at the bottom of Bring in, under a list of eleven buttons; Next steps first reads "Add a drawing" without saying a scan can do it | medium |
| scanw 02, 03 | File list shows "made-up-room  scan" and "Back to tmp" ("tmp" is a software folder word). The only way to finish is Enter: no button says "Bring in". Fine print says "y up or z up". The text box shows the extension ".ply" | high for a first time user (no on-screen way to commit) |
| scanw 04 | Sheet is a white rectangle with one dark outline: the walls at 4 ft show as a line, no dots, no scale, no label. Banner "its page is read (it is a scan: there are no drawn lines on it to snap to); 1 thing not read from it". Panel names the sheet "made-up-room cut at 4 ft 0 in.png, page 1" (a file name and "png") | words and clarity, medium |
| scanw 05 | Undo says "56,658 points, taken as meters with y up" | words low |
| scanw 06 to 08 | Wall traced over the outline: 24 LF, label "24' 0"" correct. Open items repeat the file name and ".png, page 1" | words low |
| roomlinew 01 | Fine plan, one room 550 SF | none |
| roomlinew 02 | The room line is a thin red dash drawn on top of the pink demolished wall: there is no way to see it is a line and not a wall. Figures 24' 0", 12' 0", 16' 0" appear around the plan unexplained. Living 368 SF is right | confusing, medium |
| roomlinew 03 | Two rooms 368 SF and 182 SF (right by hand). Never shows the tool in hand nor the line's own panel ("Parts two rooms where no wall stands...") that the call says explains it. Sheet "wall 1 is cut to wall 5, whose line or thickness is assumed" | clarity medium, words low |
| cabinetsw 01, 02 | "Cabinets" button in the bar; hint "Click a wall's face, on the room's side..." is clear; Countertop switch. Open items list "Range, 30 in, maker: unknown, and 91 more" for starter items nobody placed | noise, medium |
| cabinetsw 03, 04 | Fields STARTS AT and BOXES, a list of codes (B12 ... F3) with no word of what B, SB, DB, W, T, F or the number mean; appliances cannot be typed; the status line says "wall 6" | medium for a non-cabinet person |
| cabinetsw 05 | Run drawn as a tan strip against the east wall, no box names, no countertop outline; the "16' 0"" label sits on top of the boxes. Message: "Set 5 boxes ... a run of 11 ft ... 3.75 ft left to the corner; Countertop 11 ft by 25 1/2 in ... 23.375 SF" (hand figure agrees). Sheet Building tab lists no boxes | medium |
| cabinetsw 06, 07 | Refusal "B99 is not in the tool chest. It has: ..." shown three times (card, hint, status); correct but "tool chest" is not a builder's phrase | low |
| cabinetsw 08 (3D) | The run is one tan slab with a top; no doors or drawers, the house is see-through wire. Reads as a bench, not as cabinets | medium |
| cabinetsw 09, 10 | Undo takes it all back in one step: right | none |
| layoutsw 01, 02 | "Try a job two ways ... each is a job file kept in the same folder": the window has no Save a copy or Open, so a builder cannot make the second layout from the window. List shows file names ("addition-16 job") and an empty file "nothing-yet" as a job | high |
| layoutsw 03 | Table is clear: rooms 182.125 -> 244.125 (+62 SF), job cost $5,777 -> $7,137 (+$1,360), divisions 6 and 9 (agree with the command line). Missing: "left out of the job cost" row, file names in heads, overhead and profit; the Flags sentence ("one with no value reads no figure, and its difference is not worked") is hard to read. Open items sit under it | medium |
| layoutsw 04 | Taken away; says the file is as it was | none |

### Could a builder do it from the screen alone?
- Bring in a scan: only with luck. The button is far down, the file list works, but the bring-in is Enter alone and the words are software ones. No.
- Mark existing and demolition: yes. Pick a wall, three buttons in plain words, the refusal explains itself. The colours have no key and existing barely shows. Yes, with a guess about pink.
- Room line: the tool is findable; what it is for is never said on a screen shown, and it is drawn invisibly over a demolished wall. Partly.
- Run of cabinets: yes for someone who already speaks cabinet codes; the corner boxes and appliances are not in the window yet. Partly.
- Layouts side by side: reading it, yes; making the second layout, no.
