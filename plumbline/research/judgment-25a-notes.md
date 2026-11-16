# J25A notes (judge 25, part A)

Session 1 (job jobs/lib.json, pictures out/s1). Level "Ground" and four walls 20 x 14 drawn in the window: held (said "Added wall 1, 20' 0" long" etc.).
Starter items: 21 items, listed by division: 11 Equipment (dishwasher, dryer, range, refrigerator, washer), 12 Furnishings (base cabinet, bed, chair, desk, sofa, table, vanity, wardrobe), 15 Mechanical (air handler, bathtub, heat pump, kitchen sink, lavatory, water closet, water heater), 16 Electrical (panelboard only). Lavatory, range, panelboard all findable (out/s1/07, 08).

Placed 7 things in the room from 4 divisions (range, lavatory, water closet, panelboard, sofa, bathtub, water heater): each "Set one down on Ground"; range's position read E -6'11 15/16", N 5'5 7/8" for a click at (-7, 5.5): held. Sizes on plan measured off the picture at 30.88 px/ft: range 2.5 ft wide, sofa 7 x 3 ft, tub 5 x 2.5 ft, water heater 22 in round: held. 3D (out/s1/15, See through on): sofa with arms and back, tub, round water heater, lav, WC, panel on the east wall: drawn as what they are. Range is a plain box.

## J25A-1 Plan view: plumbing fixtures are white on white with no outline or name; you cannot find them (look)
Did: set down lavatory, water closet, bathtub; looked at the plan (out/s1/11 at full room, out/s1/12 zoomed in).
Expected: a plan symbol drawn in line work like a drawing, or at least an outline and the name.
Screen: lavatory and water closet are faint off-white blobs on a white plan, the tub a near-white rectangle with no edge; no item is labelled on plan (only the selected one gets a blue tag). At room scale (out/s1/11) the lav and WC are practically invisible.

## J25A-2 Picking an item from the Items list throws the view from Plan to 3D without a word (look)
Did: on Plan, clicked "Range, 30 in" in Items (out/s1/09). The view went to 3D; status only said "Range, 30 in in hand". I had to press Plan again to set it down on plan. Repeated for every item afterwards (my script had to press Plan each time).
Change on Lavatory (out/s1/17-22): panel shows kind, Trade, Drawn as, Maker/Model Unknown, Width/Depth/Height Assumed with Measure. Drawn as -> Sink: "Lavatory is drawn as a sink, wherever one stands", 3D changed. Trade -> 10 Specialties: "Lavatory is an item of Division 10 Specialties". Width Measure 24 in: "width is 24 in, measured (it was 20 in, Assumed ...) Lavatory is one of it and is now that size"; known count 74 -> 75. Held.
- (minor, words) The "Drawn as" list has 15 forms (box, base cabinet, wardrobe, water closet, lavatory, bathtub, table, chair, bed, sofa, desk, sink, tank, outdoor unit, what its name says): no range/cooktop, refrigerator, panel, washer/dryer form; a range is a plain box.
- (minor) After Measure the width row shows "measured on site" but no Known tag, where the rows beside it carry an Assumed tag (out/s1/22).
- The message log ($T/map.txt.said) for that step reads "fixture-0030 is one of it" where the screen reads "Lavatory is one of it": the screen is right; the log is not word for word.

## J25A-3 A placed thing cannot be turned; a panel on the east wall stands with its 15 in width sticking through the wall (stuck)
Did: set down Panelboard, 200 A against the east wall; selected it (out/s1/25). Its 15 in runs east-west, into the wall (clash "in each other by 6.05 in"). Looked for a way to turn it a quarter: none on the panel, Details shows only Position, Height above its level, Size; "r" key does nothing. A vanity, a range or a panel set on a north-south wall cannot face the room. (I grepped README for "rotate/turn" while checking: nothing about turning a thing.)
Also: Height above its level is 0 ft for a panelboard (sits on the floor) — Assumed, fine, but nothing hangs it on the wall.

## J25A-4 Setting a thing down into a wall says nothing; only moving one does (silent, minor)
Did: set down lavatory and water closet against the north wall and the panel on the east wall. The message each time: "Set one down on Ground. Pick it up to move it". The plan shows orange crosses (out/s1/24) and `plumbline clashes` lists 3 clashes (wall 3 x Lavatory 0.22 in, wall 3 x Water closet 2.52 in, wall 2 x Panelboard 6.05 in). Moving the sofa into the tub DID say so: "Moved Sofa 4'11 13/16" west ... It is in the same space as Bathtub, 5 ft by 1' 4"." (out/s1/24, red).

## J25A-5 "in the same space as Bathtub, 5 ft by 1' 4"" reads as an overlap of 5 ft by 1'4" (words)
The item's name "Bathtub, 5 ft" runs into "by 1' 4"" (the 16 in they overlap). An estimator reads "5 ft by 1'4"" as a size. Name needs setting off (quotes) or the depth worded "16 in deep into each other".

Write out: "Item library (CSV)" said "Wrote lib items.csv, beside the job: the job's items, 21 rows under their headings. Open it, change a size or add a row, and bring it back in with Item library, under Bring in." Columns: Item, Trade, Kind, Drawn as, Width/Depth/Height/Maker/Model/Weight each with "how sure" and "where from". No price, unit, labor, supplier or cost column.
Edited by hand (python): Range width 30 -> 32 in; Vanity 36 trade -> 6 Wood and Plastics; added Vanity, 48 in; mistakes: Shower base width "36" no unit; Grab bar trade "17 Plumbing"; Range hood drawn as "hood"; Linen cabinet twice (18 in, 24 in); Wall oven width Known with no source. Brought back with Item library -> file list -> Enter (out/s1/30). Screen: "28 rows read, 4 set on the job | Range: changed: its width | Vanity 36: changed: its trade | Vanity 48: added | Linen cabinet: added | 19 rows the same | Not used, row 24 ... no unit ... | row 25 '17' is not a division | row 26 'hood' is not something a thing can be drawn as. There are: ... | row 28 Linen cabinet: row 27 is the same item, and was used | row 29 Wall oven: does not say where its width came from". Every mistake named by row. Held.

## J25A-6 A size changed in the library does not reach the range already set down, while the screen says "changed: its width" (silent)
Did: changed Range, 30 in width to 32 in in the written CSV and brought it back (out/s1/30). Items list and `plumbline items` show the item ~32 in wide. The range standing in the room is still 30 x 26 x 36 in: Details panel (out/s1/31), its plan box measures 2.49 ft, and the saved job holds size [2.5, 2.1667, 3.0] ft sourced "Range, 30 in: a typical size". Nothing on screen says the placed one kept the old size. By contrast, Measure on the item's panel DID carry the lavatory's new width to the one standing ("Lavatory is one of it and is now that size"). Two ways to change the same item, two results; the clash check runs on the stale size.
Expected: the placed range becomes 32 in (as with Measure), or the bring-in says "1 placed keeps 30 in".

## J25A-7 Weight columns written blank with a blank "how sure" (silent, minor)
The written CSV has Weight, Weight: how sure, Weight: where from all empty for every row, where Maker/Model say Unknown / not chosen. An empty how-sure is neither Known, Assumed nor Unknown.
Save, close, open again (out/s2/01-02): everything stood where it was (range through west wall, lavatory as sink 24 in, sofa over tub, water heater, panel); items list kept the 6 Wood and Plastics heading with both vanities. Held. (The Clashes switch was on before closing and off after; switches are not kept with the job.)

# Part 2: money path (job jobs/money.json)
Level, starter assemblies (30), wall 20 ft (wall 1, 180.00 SF), hip roof 20 x 14 at 6:12, eaves 9 ft drawn in the window. Roof panel: 313.05 SF, 3.13 SQ, eave 68, hip 42, ridge 6 LF. By hand: 280 x 1.1180 = 313.05; hip = sqrt(9.899^2+3.5^2)=10.50 x 4 = 42.0; ridge 20-14=6. Held. Wall given "Exterior wall, 2x6, lap siding": "9 cost lines added". Price on top bar "no figure", 9 lines unpriced: says so.

## J25A-8 On plan a roof plane cannot be picked: a click inside it, or on its eave, says "Nothing selected." (stuck)
Did: wanted to give the south roof plane its assembly. On Plan, clicked inside the south plane (0,-4.5) and on the north eave: "Nothing selected." both times, repeated once (out/m2/04). Only in 3D did a click on the plane select it ("Selected plane 1", out/m2/06). Nothing on screen says roof planes are picked in 3D only.
(a) Shingle assembly given to plane 1 (in 3D): "Plane 1 is now 'Asphalt shingle roof': 5 cost lines added. So are plane 2, plane 3 and plane 4: the same lines on each." Said plainly. Held.
Unit cost $96.00 typed on plane 1's Asphalt shingles (sheet): "...The same line on plane 2, plane 3 and plane 4 had none: they take it too." Plane 1 1.0174 SQ x 96 = 97.67 (hand 97.6715). Div 7 = $300.52: hand, per plane 97.67+97.67+52.59+52.59 = 300.52 (unrounded 313.0495/100*96 = 300.53; sheet rounds per line then adds: fine, as long as every view does the same).
Production rate "2 hr/SQ" on "Lay underlayment and shingles": 2.03 hr on plane 1 (hand 1.017411 x 2 = 2.0348). Wage $45.00/hr: $91.57 (hand 91.567). Div 7 = $582.28 (hand: material 300.52 + labor 91.57+91.57+49.31+49.31 = 281.76 -> 582.28). Held.

## J25A-9 The "Unit cost" button on a roof plane opens the price of its first line (roof sheathing) only; the shingle line cannot be priced from the model window (stuck, minor)
Did: plane 1 selected, pressed Unit cost wanting the shingles (out/m2/09): the field reads UNIT COST OF "ROOF SHEATHING", no way to pick another line. The cost-line cards under the panel (out/m2/10) are not clickable. I went to the sheet and found "Asphalt shingles" there.

## J25A-10 A production rate typed as "2 hr/SQ" is shown back as "at 0.02 hr/SF" (words)
Did: typed 2 hr/SQ on the labor line (out/m2/16). The chip shows "at 0.02 hr/SF": not the unit I gave, and rounded to two places it no longer says what I typed (0.02 x 101.74 = 2.03 happens to agree; at 2.5 hr/SQ it would read 0.03 hr/SF = 3.05 hr vs 2.54 real). The status line said "set to 2 hr/SQ" correctly.

## J25A-11 Roof sheathing rounded up to whole sheets plane by plane: 12 sheets for 313 SF where 10 cover it (wrong money, minor)
Sheet (out/m2/12): plane 1 4.00 sheet, plane 2 2.00, plane 3 4.00, plane 4 2.00 = 12 sheets. 313.05 / 32 = 9.78 -> 10 sheets (before waste). Per-plane round-up carries 2 extra sheets with no waste factor named. An estimator orders the roof, not each plane. (Shingles are not rounded at all: 1.0174 SQ, no bundle round-up; inconsistent with sheathing.)
Overhead 10%, profit 5% typed on the sheet. Price $672.54. Hand: 582.28 + 58.228(58.23) = 640.51; x 5% = 32.0255 (32.03); 582.28+58.23+32.03 = 672.54. Top bar $672.54, sheet $672.54, saved job `plumbline estimate` PRICE ~$672.54, "it leaves out 21 lines, named below" on sheet and CLI. Agree to the cent. Held.

## J25A-12 STOP / A supplier's price list whose names match my lines word for word prices none of them (stuck, wrong money)
Did (b): copy of supplier-a-prices.csv in the job's folder (Batt insulation renamed so it would not match; added Roof sheathing per SF, Underlayment with no labor figure, Drip edge LF). Price list -> picked it -> typed ", Supplier A" -> Enter (out/m2/20, 21).
Screen: "Brought in supplier-copy.csv: 11 rows read, 22 set on the job. | Lap siding: added, bought by the SF | Lap siding from Supplier A: $4.25/SF, which Supplier A had not quoted | Wall framing, 2x6 at 16 in: added ... | Wall sheathing ... | and 16 more | Not read: the column 'Labor per unit'".
Then: top bar still $672.54, 21 lines unpriced; sheet (out/m2/22) still reads wall 1 "Wall framing, 2x6 at 16 in": the line has no price — though the list has exactly "Wall framing, 2x6 at 16 in", SF, 2.10.
Expected: matching lines priced (framing 180 SF x 2.10 = 378.00, sheathing 6 sheet x 26 = 156.00, weather barrier 180 x 0.22 = 39.60, lap siding 180 x 4.25 = 765.00, gypsum 6 x 14 = 84.00, paint 180 x 0.35 = 63.00, shingles 3.1305 SQ at 99 vs my 96), and a list of what it did not match (Fiberglass batts R-21, Drip edge) and what it would not take (Roof sheathing quoted per SF on a line bought by the sheet).
Screen says "22 set on the job" but nothing a builder can see changed in the money; "and 16 more" hides the rest of what it did, including what happened to the shingles I had priced at $96. I did not know from the screen what to do next. Reading plumbline/README.md now (allowed after a stop).
After README (it says "The program finds the headings, sets the prices it can, and lists every row and column it did not use and why"): it set no price. The saved job shows each row became a new Material with a "quote" from Supplier A and price "no price entered" (material-0049..0059), linked to no line. The wall's Material button (out/m2/23-25) lists them and "Wall 1: material set to Wall framing, 2x6 at 16 in" — price unchanged, 21 lines still unpriced, and the panel then said "2 lines on it name a framing". Undone. Neither the Estimate nor Budget tab (out/m2/26, 27) shows the supplier's quotes anywhere, side by side or otherwise. The "Labor per unit" column was "Not read" (said). Roof sheathing quoted per SF went in as a new SF material beside the sheet-bought line without a word about the unit. Batts renamed did not match and was not reported as unmatched (it too was "added").
Verdict for (b): the list priced nothing, and the screen's "22 set on the job" reads as though it did. FAIL for (b).
(c) Verify in field tab (out/m2/28): roof $582.28 riding, plane 1 $189.24 (97.67+91.57), plane 2 $101.90 (52.59+49.31): hand agrees. "Verify plane 1" selects plane 1. Its Pitch Check: typed 7:12 measured (out/m2/31): "Plane 1: pitch is 7:12, measured (it was 6:12, Assumed...) | roof solved again". Hand: h(12/7+2)=14 -> h=3.7692 (corner height 12'9 1/4" shown), plane 1 run 6.4615 (corner N -0'6 7/16"), ridge 4.923, plan 80.52 x 1.15770 = 93.22 SF shown. Price $679.40; CLI job cost 588.23 (mat 303.60 + lab 284.63), OH 58.82, profit 32.35, PRICE 679.40. Agree.
Steeper (out/m2/32): "Edge 1 was 7:12, Known (measured on site). It is now 8:12, Assumed: what was measured is no longer what is drawn. Measure it again, or undo." Good. Hand at 8:12: h=4, corners (+-2, -1, 13 ft) shown, plane 1 plan 72 x 1.20185 = 86.53 SF shown. Price $685.50.

## J25A-13 Steeper/Flatter change the price and the screen says nothing about the dollars (silent)
Steeper took the price from $679.40 to $685.50 (+$6.10) and the message speaks only of the pitch and the measured value. The brief's question "what does it say went out of the job cost, to the cent?": nothing. The figure moves on the top bar and you have to have written down the old one.
Flatter brought plane 1 back to 7:12 (Assumed now; it does not go back to Known; fair). Eave height measured 9' 2" (out/m2/34): "every plane of that roof has the one eave height"; corners 9'2"; price unchanged $679.40 (areas do not depend on eave height). Right.
Gable on plane 2 (east) (out/m2/35): "gone: plane 2, eave 2, hip 1-2, hip 2-3 | Lost with them: the assembly or cost lines on plane 2 | 5 cost lines went with it: $109.74 is out of the job cost (3 of them had no price yet). | The planes that stay changed with it: the job cost was $588.23 and is $590.02." Hand: plane 2 at 6:12 was 56.64 + 53.10 = 109.74; 588.23 - 109.74 = 478.49; planes 1 and 3 now 225.83 + 254.45, plus plane 4 109.74 = 590.02. To the cent. Price $681.47 = 590.02 + 59.00 + 32.45 (hand 59.002, 649.02 x .05 = 32.451). Top bar, sheet and CLI agree. Eave height still Known 9'2" after gable. Held, and well said.
- (words) "eaves, hips, valleys and ridges follow" fine; "Rake 2, 15.91 LF" appears on the roof panel: right word.
Move eave on plane 3 (north) out 1' 6" (out/m2/37): "Eave moved 1' 6" on edge 3 of roof. The roof is now 3 planes; its eaves, hips, valleys and ridges follow". Price $681.47 -> $754.24 (+$72.77); not a word about dollars (same as J25A-13). Plane 3 147.69 SF: hand, footprint now 20 x 15.5, ridge y = 0.1538 (7(y+7)=6(8.5-y)), run 8.346, west hip end x = -1.654, (20+11.654)/2 x 8.346 x 1.11803 = 147.69. CLI job cost 653.02, price 754.24; top bar 754.24. Agree.
- The measured eave height (9' 2", Known, "measured on site") stays Known after the eave it was measured at moved 18 in out. Arguable: the edge measured is not where it was. Pitch got "what was measured is no longer what is drawn"; eave height did not.
(d) Priced wall 1: framing $2.10 (180 SF = 378.00), lap siding $4.25 (765.00): job cost 653.02 + 1143.00 = 1796.02, price $2,074.40 (hand 1796.02 + 179.60 = 1975.62; x .05 = 98.78; = 2074.40). Shown.
Changed assembly to brick veneer (out/m2/40): "2 cost lines added. 2 cost lines of the assembly it had came off: Lap siding, Hang the siding. $765.00 is out of the job cost (1 of them had no price yet)." Framing kept $2.10. Price $1,190.83 (hand 1031.02 + 103.10 = 1134.12 + 56.71 = 1190.83). Right to the cent.
Delete (out/m2/41): "Deleted wall 1. 9 cost lines went with it: $378.00 is out of the job cost (8 of them had no price yet)." Price $754.24. Undo: $1,190.83 back, framing still $2.10 (out/m2/42); Redo: deleted again; Undo again; saved; CLI agrees. Held.
- (minor, words) With the only wall deleted, Next steps put "Add a drawing and set its scale" back as the next thing, where before it read "Add a drawing (walls drawn without one)" done.
(e) Wall 2 drawn (14 ft, no cost lines). Allowance, $/SF 12.50 (out/m2/45): "Wall 2: allowance set to $12.50. It is the budget's figure and is not in the price: give this cost lines (Assembly, or Add cost line) to carry it in." Top bar: "allowances $1,575.00 not in price" (126 SF x 12.50 = 1575.00). Price stays $1,190.83. Said plainly, nothing silent.

## J25A-14 An allowance is kept out of the price (wrong money, by an estimator's reading)
In a bid an allowance is carried in the price ("siding allowance $12.50/SF") until the owner picks. Here a wall with only an allowance adds $0 to the price; the screen says so honestly, but the price a builder would send out is $1,575 + markup short for that wall. If kept, the top bar should show a price with allowances as well.

## J25A-15 The top bar runs off the right edge of the window (look)
With the allowance note added (out/m2/45) the top bar reads "... 138 assumed  65 un" and is cut off at the window edge.
(f) nonsense: Allowance "lots", "twelve": "Can't set that price: 'twelve' is not a price; write it as a number of dollars, like 4.25 or $4.25/SF" (out/m2/48); "-12": "is below zero; a price is zero or more". Allowance kept $12.50. Held.

## J25A-16 A refused entry closes the field; the next digit typed picks a tool (breaks, minor)
Did: Allowance "-12" refused; I typed again "0" as one would to correct it: the field had closed, so "0" took up the Cable tool (out/m2/47: "Cable: type each value..."). After a refusal the field should stay open with what was typed, so it can be corrected.
More (f): unit cost "abc" refused; "$96/LF" on a SQ line refused ("a price does not carry from one unit to another"); "9,600" taken as $9,600/SQ with no warning while the same line is $96 on the other planes (price to $15,579.64; undone). Production rate "fast" refused; "0 hr/SQ" refused. Overhead "ten" refused, "-5%" refused, "250%" taken with a warning ("If 25.00 % was meant, enter it again"), price $3,789.00 checked by hand (1031.02 + 2577.55 = 3608.57 + 180.43); set back to 10. Profit "5 dollars" refused. Pitch "steep" refused with a good explanation. Eave height "-3 ft" taken, Known, with a warning that it is at or below the job's zero.

## J25A-17 A pitch of 40:12 and an eave moved 40 ft are taken without a word (silent, minor)
Pitch "40:12" was taken as measured and Known (out/m2/51) — no roofer's gauge reads that; overhead 250% got a warning, this got none. Move eave "40 ft" on a 20 x 15.5 roof was taken: the roof became 3 planes with plane 4 at 9.84 SQ, price $1,317.24 -> $3,254.90, and the message said only "Eave moved 40 ft ... gone: ridge 1-3" (out/m2/52). No dollars, no "that is more than twice the roof's width".
Money check after all of it: top bar $3,254.90 and $2,818.09 riding; Verify tab $2,818.09; saved job job cost $2,818.09, PRICE $3,254.90 (2818.09 + 281.81 + 155.00 = 3254.90; 3099.90 x .05 = 154.995 -> 155.00). `check`: figures agree with what is drawn. Agree to the cent.

## J25A-18 Pressing Starter items a second time throws away my changes to three items, and calls it "Brought in 3 starter items" (silent, serious for a library)
Did: on the library job (already holding my changed items), pressed Starter items again (out/s3/01), saved, read `plumbline items`/`show` (copy: jobs/lib-after-second-starter.json).
Before: Lavatory 24 in wide Known "measured on site", drawn as a sink, Division 10; Range, 30 in 32 in wide; Vanity, 36 in Division 6.
After: Lavatory ~20 in Assumed "a typical size", drawn as lavatory, Division 15; Range ~30 in; Vanity, 36 in back in Division 12. A measured Known value turned back to an Assumed typical size. The screen said only "Brought in 3 starter items, trade by trade, at typical sizes" — nothing about whose changes it overwrote. The lavatory standing in the room kept 24 x 18 x 34 (so now the placed one and its item disagree, as in J25A-6). An estimator presses "Starter items" again to get back an item he deleted or to see what is new; he loses his own library.

## J25A-19 Words a builder would not say (words)
- "roof solved again", "the roof this plane is solved from (roof)" (out/m2/31, 34): "solved" is a computer's word; "the roof is worked out again" / "the roof follows".
- "the detail is not whole: 3 of its 5 lines cannot be extended" (roof plane panel, Budget tab, out/m2/10, 27): "not whole" is coined; "3 of 5 lines have no price".
- "Comes to, detail rate ... detail ÷ surface area" (out/m2/10): "cost per SF of the detail".
- "Like for like: nothing has both an allowance and a whole detail" (out/m2/27).
- "Item library (CSV)", "Estimate (CSV)" buttons; "table" beside each file in the file list (out/m2/20); "Back to j25a" names a folder; "Not read: the column ..." (out/m2/21). CSV is a file format; a builder says spreadsheet.
- "Brought in supplier-copy.csv: 11 rows read, 22 set on the job" (out/m2/21): "set on the job" says nothing a builder can check; "Lap siding from Supplier A: $4.25/SF, which Supplier A had not quoted" reads as a contradiction.
- "Drawn as: What its name says" (Change panel, out/s1/19).
- "worked out from the footprint of roof", "rises from edge 1 of roof" (missing "the").
- "its production rate has to be more than nothing (and no more than a billion)".
- "at 0.02 hr/SF" for a rate typed as 2 hr/SQ (J25A-10).
- Command line only: "drawn as a outdoor unit".
