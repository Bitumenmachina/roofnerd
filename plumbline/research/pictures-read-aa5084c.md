# Pictures read, new run (readpics2)

## bring (12 read: 6 model, 6 sheet)
- bring/02-model: file picker panel says "WHERE THIS JOB IS KEPT  < Back to tmp" — "tmp" is a computer folder name shown to a builder; type label "BIM model". BRING IN heading below is left empty (its buttons gone while picking). Sure.
- bring/03-sheet (also 04, 05): Open items line 'made-up-house.ifc, brought in on screen: made-up-house.ifc, "Ground", "buildingstorey" (as the model calls it) #23 in the file' — "buildingstorey" and "#23 in the file" are file/software words; "brought in on screen: made-up-house.ifc" repeats the file name. Sure.
- bring/03-model, 03-sheet: one number two ways: "Ground, elevation: 0 ft" in open items vs level "Ground 0' 0"" / "floor at 0' 0"". Also "Floor 600 SF" (panel) vs "Gross area ... 600.00 SF" (open item) on one screen. Sure.
- bring/03-model: "Roof, level height: unknown" (red) while Levels list shows "Roof 8' 0"" and sheet "Roof floor at 8' 0"" — reads as a contradiction unless you know "height" is floor-to-floor. Medium.
- bring/04-model: message "Brought in tinners-yard.xlsx: 5 rows read, 6 set on the job" (6 set from 5 rows?); "Copper, 16 oz from Tinners Yard: $9.05/LB, which Tinners Yard had not quoted; the job had no price for it and takes Tinners Yard's" — says both from Tinners Yard and not quoted by Tinners Yard; "Not read: 1 cell holds a formula: what it last worked out to was read, not the formula" — "Not read" heading over something that was read. Message box covers the top half of the house. Sure.
- bring/04-model..06: top bar still "PRICE nothing priced yet" after a price list was set (no assemblies, so likely true, but a person who just brought prices in is told nothing is priced without why). Medium.
- bring/05-model: "prior-estimate.csv: 4 rows read, 1 set on the job" then "it is no part of this job's price" — "set on the job" contradicts; the prior estimate is not visible anywhere on the sheet (no tab) . Medium.
- bring/06: "Saved" mark before counts, not cut off; Next steps Save ticked. OK.

## build (8 read: 6 model, 2 sheets differing; sheets 04, 06 differ only on the foot line)
- build/02-model: floor added (panel "Floor 1 Assumed 600 SF", open item "Floor 1, boundary: 4 corners") but on the plan nothing visibly changed from 01 (pixel diff tiny); the floor is not seen. Fairly sure.
- build/03-model (and 04, 05): only "East room 341 SF" is labelled on the plan; the West room (just named "West room" per sheet) has no label on the plan. Sure. One number two ways: plan label "341 SF" vs panel/sheet "341.25 SF". Sure.
- build/05-model: the ridge (dashed) runs straight through the "East room 341 SF" label. Sure, minor.
- build/04-model, 05: eave height field "8 ft" and message "Added roof: 4 planes, 6:12 on every side, eaves at 8 ft." — feet written "8 ft" while everything else is 8' 0". Sure, minor.
- build/06-model: the 3D house is plain: flat grey-blue roof with no roofing on it, bare walls, no eave overhang; reads as a massing block, not "dressed as built". Sure (may be by design for an unpriced job).
- build/01..05 sheet: in the walls table the row "floor 1  floor 4 in ... 600.00 SF" sits above "Walls, total", so the floor reads as part of the walls total. Medium.
- build/03, 05 model: open items now say "its outline counts on wall 1, wall 3, wall 4 and wall 5, whose heights are assumed" — the fixed wording. OK.
- The rest show what they claim.

## carry (6 read: 5 model, 1 sheet; other sheets checked by pixel diff, foot line / list only)
- carry/02..05-model: "Height above its level  Assumed  0 ft" (and sheet "height above its level: 0 ft") — written "0 ft", not 0' 0", beside positions written E 7' 0". Sure.
- carry/04-model: the comment claims "let go a foot and a half east"; the screen says "Moved Harbor Vanity 36 2' 0 11/16" east, 0' 1 7/8" north" (position E 9' 0 11/16", N 1' 0 3/8"). The message is consistent with the panel; only the script comment is off. Sure.
- carry/04-model: once clear, the vanity is drawn as a pale, see-through tan box inside the blue selection frame, not the solid wood of 01 — "its own colour" is only faintly there. Also the "Clashes" switch row disappears from SHOW when the count goes to 0 (present in 01-03, gone in 04, back in 05): a switch that vanishes. Fairly sure.
- carry/03-model: the Clashes switch turns itself on while the vanity is in the hand (off in 01/02). Possibly intended. Low.
- The rest show what they claim (01 vanity into WC; 02 red when selected; 03 red message "in the same space as Water closet (1 11/16 in into each other)"; 05 undone, back and red).

## clash (9 read: 7 model, 2 sheets; sheets 02,06,07 differ only on foot line, 03/04 not opened)
- clash/01-sheet, 05-sheet: "IN EACH OTHER BY  12.00 in", "3.50 in" — inches with two decimals where a trade writes 12" / 3 1/2". (The model window writes "1 11/16 in" for the same kind of figure in carry/03.) Same figure written two ways across the program. Sure.
- clash/01..06-model: with nothing selected the right panel shows "LEVEL ground ... Elevation 0 ft ... Level height 9 ft" — feet as "9 ft", "0 ft", not 9' 0" (while the left Levels list says "ground 0' 0""). Sure.
- clash/05-sheet: open items "Air handler, connection 1: Supply Air (duct): E 0' 0", N 0' 0", H 2' 0"" and "6 in round elbow, connection 1 (duct): E 0' 0", N 0' 0", H 0' 3"" — read as places at the job's corner, though the handler and elbow are not there (they look like offsets from the thing itself). "connection 1" is a software-ish label. Fairly sure it confuses.
- clash/03-model, 04-model: the elbow and tee are small light-blue blobs at the duct corners; hard to see they were added (comment "an elbow", "a tee, and the duct cut there"; the cut is not visible). Medium.
- clash/04, 05: after fittings, the clashes change from "2 findings" (red) to "2 to verify in the field" (amber) without any message saying why (the fittings are Assumed). Medium.
- clash/06, 07-model: pipe panel "Type  1 in" (a type that is just a size), "Height above its level 8 ft". The 1 in pipe in 3D (07) is a thin line, not a pipe. Medium.
- The rest show what they claim (01 two clashes marked; 02 fitting in hand; 05 one clash left; 06 pipe selected; 07 in 3D).

## d2 (11 read: 8 model, 3 sheets; 02,05,06 sheets only foot line or none; 03/04 sheets small changes not opened)
- d2/01..08-model: top bar ends "... 2 to verify | Saved" (later "Not saved") — the known/assumed/unknown counts are NOT on the model window's top bar at all in this set (the sheet shows "189 known 16 assumed 32 unknown"). Not cut off mid-word; just absent. Sure (crop d2top.png).
- d2/04, 05, 06, 07-model: CORNERS table "NORTH  -0' 6"" for corners 1 and 2 — a minus sign for south, while the Eave line just below on the same panel reads "E 100' 0", S 0' 6" to E 140' 0", S 0' 6"". One place, two ways, one with a minus. Sure.
- d2/02..07-model: "Eave height 10 ft" vs corner HEIGHT "10' 0"" on the same panel. Sure.
- d2/08-model: COST LINES "Quantity  Assumed  283.9250 SF" (four decimals) for both lines; the same figure is "283.93 SF" two lines up (Surface area) and on the sheet. Sure.
- d2/08-model: "Detail, all in  Unknown   detail ÷ surface area" — where a figure belongs the formula is printed in red. Sure.
- d2/08-sheet vs model: plane 1 "Copper standing seam panels" shows quantity "unknown" on the sheet while the model panel shows Quantity "283.9250 SF" for the same line (with waste unknown). The column on the sheet has no visible heading in this scrolled view. Fairly sure it reads as a disagreement.
- d2/01..08: top bar "34 not priced  6 lines unpriced" — two near-identical phrases side by side; also recap "Job cost ... none" (red) and "Price no figure" — two words for the same nothing. Medium.
- d2/01-model: right panel heading "BY DIVISION" with nothing under it, then MARKUP. Sure.
- d2/01-model: open item "South, schedule code: unknown ... schedule not built" — "schedule code" unclear to a builder. Medium.
- d2/08-model: the plan is cut by the bottom tool panel: plane 1 (the one being worked) lies half under it. Low.
- d2/08-model: "Detail comes to" wording is present (the fix). OK.
- The rest show what they claim (02 plane picked; 03 8:12; 04 eave out 6 in with amber "The roof's footprint was Known ... it is Assumed"; 05 typing 9:12; 06 set; 07 copper roof drawn; 08 sheet at cost lines "no price").

## d4 (12 read: 7 model, 5 sheets; 04/07 sheets same as before but foot)
- d4/01-sheet (Estimate) vs d4/02-sheet (Budget): Estimate says "Price  no figure ... there is no job cost: no line with a cost type and no allowance comes to a figure", while Budget says "Allowances over 4 items ... $22,033.70" and "Detail over 3 items $16,761.85". The job has allowances that come to figures; the recap says none does. Sure. The model's top bar "PRICE no figure" stays through 01-07 though the Budget grows to $33,773.06 (06-sheet). Sure.
- d4/01-model: nothing selected; right panel "BY DIVISION" heading with nothing under it — no "Allowances" row although the job has $22,033.70 of allowances. Sure. d4/01-sheet By division "0 of 16 with work in them", rows 1-7 "none"; rows below 7 not visible in the picture, so an Allowances row there could not be seen.
- d4/01..07-model top bar: no known/assumed/unknown counts at all (bar: "PRICE no figure  14 not priced  6 lines unpriced  2 to verify, $3,555.07 riding | Saved"). Sure.
- d4/05-model: after the east line's unit cost is set ($7,924.06 on the plane), top bar still "6 lines unpriced" (unchanged from 04). Fairly sure it should have moved or is a different count; confusing.
- d4/03, 06-model: the selected plane (east, north) faces away from the camera; its label is not on screen, only its blue outline; the visible labels are "west" and "south". Medium.
- d4/03..07-model: "Schedule code  Unknown  schedule not built" in red — "schedule code"/"schedule not built" unclear to a builder. Medium.
- d4/06-model: message "North: allowance set to $28.00." — no "/SF" (the panel says "$28.00/SF"). Low.
- d4/02..06-sheet: headwall "$42.00 under  -8.75 %" — "under" and a minus together. Low.
- d4/02-sheet: dollar figures and "rests on something assumed" fine; Budget reconciles.
- d4/07: refusal 'Can't set that price: "nonsense" is not a price; write it as a number of dollars, like 4.25 or $4.25/SF' in red. Shows what it claims.

## d5 (21 read: 9 model (01,06-12), 10 sheets; models 02-05 identical to 01 in the centre; sheet 02 identical to 01; 09/10 sheets foot only)
- d5/03, 07-sheet ("The estimate, by item"): quantities with four decimals: "Standing seam panels 3 places 861.5955 LB", "Fabricate panels 6.7082 hr", "Install panels 26.8328 hr", "Standing seam roofing, by subcontractor 251.5576 SF". Sure.
- d5/07, 08, 11-sheet ("The sheet"): line quantity boxes "461.1890 LB", "419.2627 SF", "276.7134 LB", "251.5576 SF", "123.6931 LB", "576.4863 LB" — four decimals on the line a person edits; other lines on the same sheet show "419.26 SF", "4.19 hr". Same kind of figure two ways on one screen. Sure.
- d5/01..12-model: the roof priced as copper standing seam is drawn dark grey; the panel admits it twice: "Also priced, not drawn: standing seam, copper  From its material, Copper, 16 oz" and again "Also priced, not drawn: standing seam, copper  From its assembly, Standing seam copper, 16 oz" (07-12). Two identical lines, and the roof does not look copper. Sure.
- d5/10-model: after the material is set to "Copper, 20 oz", the panel shows "From its material, Copper, 20 oz" and "From its assembly, Standing seam copper, 16 oz", and "Assembly Standing seam copper, 16 oz" — the plane now says two weights, with no word that they disagree. Medium.
- d5/01..12-model top bar: no known/assumed/unknown counts; "Saved"/"Not saved" last. Sure.
- d5/08-model: headwall selected (panel shows it) but nothing is highlighted on the 3D roof; the headwall is not visible. Fairly sure.
- d5/12: refusal "Can't set the overhead: "ten" is not a number" — says what is wrong but not what to type (the price refusal in d4/07 gives "like 4.25"). Low.
- d5/01, 06-model: BY DIVISION on the right panel lists "1 General Requirements $3,800.00 / 7 Thermal and Moisture Protection Assumed $21,492.43 / In no division $3,150.00" — no Allowances row; could not tell whether this job carries an allowance in its price (Budget tab not pictured).
- d5/01, 04-sheet: open item "The job, cost code: unknown / not coded" with a RED mark (model 01 open items: red "Unknown The job, cost code: unknown  not coded"). Sure.
- d5/04-sheet: Suppliers side by side: "Tinners Yard  Total $6,714.91 ... 2 of 4" with the two left out named — clear. OK.
- d5/06: price $34,403.96 agrees between the sheet recap and the model's top bar and right panel. d5/12: $34,305.57 agrees in both windows. OK.

## draw (16 read: 15 model, 1 sheet (05); other sheets small or foot-line changes)
- draw/10-model: "wall 6 45.00 SF" label sits on the plan at lower right where the stray wall is, but no wall line is visible there: the label covers the 5 ft wall entirely. The person cannot see the thing they selected. Sure.
- draw/09, 10-model: one panel, height two ways: AS DRAWN "Height Assumed 9 ft" and TAKEOFF "Height Assumed 9' 0"". Sure. draw/12: window "Center, along the wall  4.4167 ft" (decimal feet, four places) and "Sill 3 ft" beside "Height 3' 0"", "Width 4' 0"". Sure.
- draw/09-model: message "Moved wall 5 1' 0" east, 0' 0" north." — "0' 0" north" is noise. Low. The partition's door: the door in the partition is not visible on plan as moved (brown mark at 770,500 present — OK).
- draw/09, 11, 13, 14: open item "Wall 5, wall line: ... entered on screen; not yet checked against a drawing; moved (entered on screen; not yet checked against a drawing)" — the source repeated, nested in brackets. Sure.
- draw/04-model: the east wall's "16' 0"" length label is cut by the wall line (the wall draws through the label's right half). Sure (crop dr4.png). Same at the partition labels in 05-14 ("16' 0"" half-hidden behind the partition at 740-770,491) and "24' 0"" labels overlap the pipe and fixture at 745-775,400 and 582. Sure.
- draw/07-model: the lavatory (fixture) on plan is a small grey square at the top of the partition (780,400) overlapping the "24' 0"" label; hard to read as a lavatory. Medium.
- draw/05-sheet: the sheet window has only "Verify in field 14" and "Building 5" tabs; no Estimate tab while nothing is priced. Possibly by design; a first-time user may look for the estimate. Low.
- draw/15-model: the 3D house is small in the middle of the view; the pipe and lavatory and the air handler are not visible; no windows visible on the two visible faces (the remaining windows are on the north and east walls, east face appears blank). Clashes switch "Clashes 3" is off, so the 3 clashes are not shown anywhere on screen. Medium.
- draw/01..15 top bar: "Saved"/"Not saved" stands before "0 known" / "7 known 97 assumed", not cut off. OK (fix seen).
- draw/09..14: places written "W 2' 1", S 9' 0"" with W and S, not minus (fix seen).
- draw/14: refusal 'Can't use that system: "Steam" is not a system a conduit goes on. There are: Power, Lighting' shown at the field, red. OK.
- The rest show what they claim.

## field (12 read: 7 model, 5 sheets; 03 sheet identical to 02)
- field/04-model and field/06-model (and 07): after ONE laser reading, the message says "A second reading came straight after that one and was not used. One reading to a value: click the next value, then shoot." The script sends one reading ending with Tab (04) or Return (06), as a DISTO in keyboard mode does. A person who shot once is told a second shot was thrown away. Fairly sure this is a false message; high trip.
- field/04-model: one message, one height two ways: "level height is 8' 10 1/2", measured (it was 9 ft, ...)" then "upper's floor is now at 8' 10 1/2", Known; it was at 9' 0"". Same at 06: "7' 10 3/4", measured (it was 8 ft, ...)". Sure.
- field/01..07-model: level panel "Elevation 0 ft", "Level height 9 ft" (before measuring); the duct "Height above its level 8 ft". Not feet-and-inches. After measuring, "8' 10 1/2"" and "7' 10 3/4"". Sure.
- field/04-model: Elevation confirmed "same" now reads "scaled off a made-up plan; found as drawn (checked on site)" with no Known mark shown, while Level height loses its mark too; nothing on the panel says Known. Medium.
- field/05-sheet, 06-sheet: clash depths "2.00 in", "5.00 in", "2.25 in" — inches with decimals (a trade writes 2", 2 1/4"). Sure.
- field/07-model: the Clashes switch is off (on in 06) yet the red clash mark is still drawn across the duct at the second pipe. Fairly sure.
- field/07-sheet: "Upper, level height: 8 ft" while the Levels list shows "upper 8' 10 1/2"" (its floor) — fine in meaning, but "8 ft" vs feet-inches. Low.
- field/01-model: "SYSTEMS  On no system  44.00 LF" — reads oddly; the runs have no system named. Low.
- The amber announcement of a measured reading: SEEN (04, 06: amber bar and amber text, not red).
- The rest show what they claim (01 five questions, $518.40 on each level; 04 price $615.86 -> $607.31 in both windows; 06 one finding, red; 07 one question with no dollars).

## first (7 read: 6 model, 1 sheet (04); other sheets small changes)
- first/02..06-model: the drawing's scale shown as "1" = 7' 1"" under "usda-7213, page 4" — no scale a builder knows (the sheet itself says 1/8" = 1'-0" on the foundation plan). It is what the calibration worked out, but reads as wrong. Medium.
- first/04..06-model: plan label "House 954 SF" vs panel "Rooms, inside their walls 954.23 SF" and open item "Floor area of House: 954.23 SF". Same number two ways on one screen. Sure (minor).
- first/04-sheet: wall lengths "41.25 LF", "23.92 LF" while the plan labels say "41' 3"", "23' 11"". Two ways across the windows. Medium.
- first/04-sheet: open item source "traced from usda-7213, page 4 (usda-7213.pdf, page 4)" — the drawing named twice, with its file ending ".pdf". Low.
- first/05-model: message "Added roof: 4 planes, 6:12 on every side, eaves at 8 ft." — "8 ft" not 8' 0". Low.
- first/06-model: the 3D house: grey roof with no roofing, bare walls, one small window with no frame; plain massing. The plan sheet lies flat under it and reads well. Medium (look).
- first/04, 05, 06: "its outline counts on wall 1, wall 2, wall 3 and wall 4, whose heights are assumed" — fix SEEN.
- first/01: top bar "nothing priced yet | Not saved | 0 known" — Saved mark before the counts. OK.
- The rest show what they claim (01 new job; 02 sheet to scale, "41' 3" between the two points"; 03 outside walls on the plan's lines; 04 door, windows, floor, room; 05 roof dashed over; 06 3D on its sheet).

## gablew (8 read: 4 model, 4 sheets)
- gablew/02-model: message 'Wall 1 is now "Exterior wall, 2x6, brick veneer": 2 cost lines added. 2 cost lines of the assembly it had came off: Lap siding, Hang the siding. $864.00 is out of the job cost (1 of them had no price yet).' — "the assembly it had came off" is broken English; which 2 lines were added is not said. Sure.
- gablew/04-sheet (also 01, 02 sheets): "Asphalt shingles 1.7889 SQ" — four decimals in the quantity box; the same plane's underlayment is "178.89 SF". Sure.
- gablew/01..04-model: wall "Height 9 ft" (as drawn) and "Height 9' 0"" (takeoff) on one panel; plane 2 "Eave height 9 ft" beside corner heights "9' 0"". Sure.
- gablew/04-model: after the gable, top bar "PRICE no figure" (red) while the right panel shows "JOB COST $429.33" — explained only by "OVERHEAD no figure / PROFIT no figure" in small type. Medium.
- gablew/01..04-model: only the south wall is dressed (lap siding, then brick); the west wall beside it is bare white, so the house looks half built. By the rule "drawn only where priced", but a first-time viewer would read it as a rendering fault. Medium.
- gablew/04-model: the gable end (east) is out of view, so the picture does not show the gable itself — only the ridge running further east; the claim "a gable: three planes" rests on the message and the panel ("ROOF PLANES 3", "Rake 2 17.89 LF"). Medium.
- gablew/01..03: "Cost code   not coded" in quiet grey with no red mark — fix SEEN.
- gablew/04-model: "4 Masonry Unknown no figure" etc. on BY DIVISION, no Allowances row (this job may carry none). 
- The rest show what they claim (02 lines off and dollars said; 03 east plane selected with its corners; 04 the job cost before and after, $429.32 -> $429.33, in both windows).

## glass (11 read: 01, 02 model at full size, 03-09 model on a contact sheet at half size; 01 sheet; other sheets identical or foot only)
- glass/04..07-model: through the frosted right panel the roof's orange hip and green eave lines show as coloured smears across "Overhead", "Profit" and the "Floor"/"Doors and windows" rows; at half size they read like highlights on those rows. Medium (seen at half size).
- glass/08, 09-model: door panel "Type door 3 ft", "Center, along the wall 6 ft", "Sill 0 ft" beside "Height 7' 0"", "Width 3' 0"" — one panel, two ways. Sure.
- glass/08, 09-model: after the double-click the door fills the whole view from top to bottom (cut at the top by the top edge and the bottom by the tool panel); no wall or ground round it to say where you are. Low.
- glass/01: roof drawn plain grey (no roofing priced), walls plain; window has frame and glass. Low (by the rule).
- glass/05: identical in the model to 04 — as claimed (the wheel over the panel did not move the model).
- The rest show what they claim (02 roof off, 03 on again, 04 in at the door, 06 turned, 07 door went right with the hand, 08 flown to the door and selected, 09 moved left after the flick).

## house (10 read: 8 model, 2 sheets; sheets 04, 05 not opened (tab changes), others foot only)
- house/03-model: "one system picked out: supply air" — on the model nothing is picked out: every system is drawn as in 02 (pixel change in the model area is 422 px, a riser greyed); supply air is not highlighted against the others. Also the panel heading "Supply air" with subtitle "Supply Air" (same words twice). Fairly sure.
- house/04-model: hot water is drawn red, but the other systems are also still drawn (greyed somewhat); red run visible. The "run of no size" reads "Pipe, size unknown 9.40 LF Unknown" and fittings "not said  bend  Unknown 2 EA" — "not said" as a fitting name is odd. Medium.
- house/03-model: CONNECTIONS rows "first-floor trunk, west, piece 3 of 3: its second end ... open end" — wraps into four lines; "E 2' 0", N 17' 0", H 8' 4 13/16"" fine. Low.
- house/07-model: cable panel "Type  Cable 0.4 in round, assumed" — a decimal-inch size with the word "assumed" inside the value (no Assumed mark), and "Height above its level 9.5 ft" — decimal feet. Sure.
- house/06, 07-model: plan labels crowd and overlap: "Utility 53 SF" over the pink return riser and the "40' 0"" dimension; "40' 0"" printed on interior lines at (785,264) and (785,427); "12' 0"" labels on the wall line; "Bath 54 SF" across the duct. Sure.
- house/01-model: no roofing on the roof (plain grey), plain walls: the whole-house sample does not look dressed. By the rule (nothing priced), but a first look reads unfinished. Medium.
- house/05: the levels list is scrolled so the first visible row is "first floor"; "All levels" and "second floor" are off the top with no sign they are there. Low.
- house/08-model: on foot: a duct runs through the room just under the ceiling (plausible), a window on the left wall reaches nearly to the floor; reads as a room. OK.
- The rest show what they claim.

## itemlib (5 read: 5 model; sheets differ only on the foot line or a small list change)
- itemlib/02, 03, 04-model: the vanity set on the floor is a tiny brown box in the middle of an empty ground, far from the camera (about 35 px). "A cabinet under a worktop" (02/03) and "drawn as its box" (04) look the same; the change cannot be seen. Sure.
- itemlib/02..05: the item list's sizes "furnishing, 36 x 24 x 34 1/2 in" wrap so "in" sits alone on a third line (Base cabinet, Vanity). Low.
- itemlib/03: "An item, placed as  a furnishing" and "Trade Division 12 Furnishings" — fine. "Drawn as base cabinet" lower-case vs list "Base cabinet". Low.
- itemlib/05: message "Wrote bench items.csv, beside the job" — the file's name and where it is; no way on screen to see where "beside the job" is. Low.
- itemlib/02: open item "Vanity, 36 in, position: W 2' 6", N 12' 3 5/8"" — W, not minus (fix SEEN).
- The rest show what they claim.

## itemnew (7 read: 5 model, 2 sheets (04, and 01 not needed); others small)
- itemnew/01-model: "the form, four fields in": only NAME "Wall oven, 27 in" shows at the foot of the left panel; KIND is cut at the panel's edge and the other fields are under "more below". The form a person is filling is mostly out of sight. Sure.
- itemnew/04-sheet and 05-model: after the rename every heading reads "Wall oven, single", but the size's source line still reads "Wall oven, 27 in: typed in the window; not yet checked against its maker's sheet" — the old name lives on under the new one. Sure. (Otherwise the set-down thing is called by the new name in the item list, the panel, the open items and the sheet: fix SEEN, with that one exception.)
- itemnew/02-model: "Drawn as  box, its name saying nothing else" — awkward wording. Low.
- itemnew/03-model: refusal "Can't delete the item: one is set down from Wall oven, 27 in: take it out of the job first" — two colons, "set down from" unclear. Medium.
- itemnew/05-model: "Took the item "Range hood" out of the job" — a Range hood never appears in any earlier picture of this set, so the person sees an item vanish that they were never shown. Low (the script made it off-picture).
- itemnew/02..05: the oven on the ground is a tiny grey box far from the camera. Low.
- The rest show what they claim.

## items (5 read: 4 model, 1 sheet (02); sheets 03, 04 foot only)
- items/02..04-model: the vanity made from the maker's sheet is drawn as a see-through pink box with a blue frame, not as a vanity (the item library's "Drawn as" choices include Base cabinet; here nothing on the panel says what it is drawn as). Medium.
- items/03-model: message "Wrote job.ifc ... the building as a BIM model (IFC4, lengths in feet) ... Not in it: 2 x type (each thing is written with its own size; the type it took it from is named among its values)." — "IFC4", "type", "values" are software words; the sentence is hard for a builder. Sure.
- items/02..04-model: "Height above its level 0 ft" (not 0' 0"). Sure.
- items/04-model: "Wrote job for the PM.xlsx ... 3 sheets: Lines, Budget, Verify in field. The job has no lines yet: the file has its headings and nothing under them." Clear. OK. The claim "the last of the six written" — only two writes are pictured (03, 04); could not see the other four. Low.
- items/01-model: the Items list entry is below "more below" (only "NO TRADE SAID" peeks out) although the message says "It is under Items, on the left". Medium.
- items/01: E 5' 2 7/8", N 3' 0 3/16" — fine.
- The rest show what they claim.

## layers (6 read: 5 model, 1 sheet; sheets 02-05 foot only)
- layers/05-model: level panel "Elevation 0 ft", "Level height 10 ft" while the Levels list beside it reads "first floor 0' 0"", "second floor 10' 0"". Sure.
- layers/01..04-model: Overhead "10.00 %" and Profit "8.00 %" each sourced "made-up price" — a percentage called a price. Low.
- layers/04-model: with the ground away, a short blue stub floats in the air outside the house at the left (about 455,597), not joined to anything visible. Medium.
- layers/01..04-model, 01-sheet: BY DIVISION lists 1-16 with figures; no Allowances row on the model's right panel. The sheet's By division table shows rows 1-13 only before the picture ends; could not see below. Could not tell whether this job carries an allowance.
- layers/01: the house reads well: lap siding, stone base, double-hung windows with trim, a standing-seam roof. layers/02 weather barrier and dark flashing at openings; layers/03 frames with headers at the openings; layers/04 foundation wall and footing under the house; layers/05 first floor looked into. Each shows what it claims.

## money (12 read: 6 model, 6 sheets)
- money/04-sheet, 05-sheet: "Underlayment 143.1084 SF" — four decimals, both in "The estimate, by item" and in the line's quantity box; the plan label says "143 SF". Sure.
- money/01-sheet: open item "Wall 1, cost code: unknown / not coded" with a RED dot on the sheet's Verify-in-field tab (the model panel shows "Cost code not coded" in quiet grey; the sheet still marks it red). Sure.
- money/04-model, 04-sheet: the labor line "Hang the board 5.40 hr no price no figure" sits under "NO DIVISION" though it belongs to the wall's drywall (Division 9). Sure. Meanwhile the top bar shows "PRICE $664.69" in plain black with no mark that a line in it has no price (only a separate "1 line unpriced"). Medium.
- money/03-model: the comment claims "the underlayment is Known"; the screen says "Main roof: edge 4 found as drawn" and the roof's values lose their Assumed marks; on 04-sheet the underlayment quantity is black (Known). OK in substance.
- money/01, 02: "Eave height 9 ft" then message "eave height is 9' 2", measured (it was 9 ft, ...)". Two ways in one message. Sure.
- money/03, 04-model: each value's source "scaled off a made-up plan; found as drawn (checked on site)" — long and repeated six times. Low.
- money/05, 06-model: Overhead/Profit sourced "made-up markup"; in layers the same rows say "made-up price". Two words for one thing across jobs. Low.
- money/06-model: "The job: overhead set to 200.00 %. That is more than what it is a percentage of. If 20.00 % was meant, enter it again." — accepted, priced $236.12 Assumed; the sentence "more than what it is a percentage of" is clumsy. Low-medium.
- money/05: wall deleted: house shown open on the south, "$432.00 is out of the job cost (1 of them had no price yet)"; price $94.45 agrees in both windows. OK.
- money/04-sheet: "no price" on the unpriced row (fix SEEN). Allowances: no row (this job has no allowance) — could not test.
- The rest show what they claim.

## pagew (4 read: 3 model, 1 sheet)
- pagew/01, 02-model: "WHERE THIS JOB IS KEPT  < Back to tmp.srbQRh4SpB" — a computer's scratch-folder name, gibberish to a builder. Sure.
- pagew/02-model: the page is picked by typing "usda-7213.pdf, 1" into "OR TYPE ITS NAME" (the script typed it), with the four pages listed just above — the picture shows typing as the way in, the very thing the rule says nobody should have to do. Medium.
- pagew/01..03: "Ground, elevation: 0 ft" open item vs "Ground 0' 0"" in levels. Sure (every set).
- pagew/03-model: message "its page is read (it is a scan: there are no drawn lines on it to snap to); 2 things not read from it: select the drawing to see what" — clear enough; "snap" is a software word. Low.
- pagew/03-model: the drawing is laid under the level, small in the middle with wide margins; fine.
- The rest show what they claim.

## plan (11 read: 8 model + made-up/01-03 model; sheets differ little, 04/06/07 sheets not opened)
- plan/04 vs first/02: the SAME sheet (usda-7213 page 4), the same "41' 3"" dimension: plan says the drawing is "87' 6 13/16" wide" and scale "1/8" = 1'-0""; first says "77' 7 3/4" wide" and "1" = 7' 1"". One of the two calibrations is about 12% off, and nothing on screen warns a person in `first` that "1" = 7' 1"" is not a scale any drawing uses. Fairly sure this matters (a person's whole takeoff rides on it).
- plan/01-model: right panel "Nothing on it to snap to." and "NOT READ FROM IT: it is a scan (1 picture and no drawn lines): there is nothing on it to snap to, and it is traced over by eye" — "snap" is a software word. Medium. (Also in made-up/03: "8 straight lines to snap to", "1 curved piece of line is known by its ends only".)
- plan/made-up/03-model: "Scale dimension Assumed 30 ft" beside "1/4" = 1'-0"" and the drawing's "30'-0"". Low.
- plan/06, 07-model: the "23' 11"" length labels sit on the wall lines and are half washed out against the drawing (semi-transparent label over the wall), hard to read. Medium.
- plan/08-model: the 3D walls are bare, no roof, standing over the sheet; fine for the step.
- The rest show what they claim (01 no scale; 02 refused, red, at the top and the foot; 03 scale typed "41' 3""; 05 a wall length typed; 06/07 the walls traced; made-up 01-03 the pointer on the dimension line's end, closing, and the title-block scale 1/4" = 1'-0").

## price (14 read: 8 model, 5 sheets (04, 06, 07, 08, and 05 by diff); 02/03 sheets foot only)
- price/05-model: message 'Wall 1 is now "Exterior wall, 2x6, brick veneer": 2 cost lines added. 2 cost lines of the assembly it had came off: Lap siding, Hang the siding.' — broken English ("the assembly it had came off"), same as gablew/02. Sure.
- price/04-sheet: recap "Job cost ... none" (red) and "Price ... no figure" (red) — two words for the same empty figure on adjacent rows; "none" is also used in grey for the rows that truly have nothing (Fabrication, Supervision). A red "none" reads as "zero". Medium.
- price/07, 08: the sheet shows Division 4 at "$4,320.00" while the model's top bar still reads "PRICE no figure" and the wall panel "COST no figure" — consistent with the rule (the price is incomplete) but nothing on the model says $4,320 of it is in. Low-medium.
- price/04..08-model: Wall "Height 9 ft" (AS DRAWN) beside "Height 9' 0"" (TAKEOFF). Sure.
- price/06, 07-sheet: "Frame and sheathe the wall  no quantity" in red until the rate is set; then "5.40 hr" (08). OK.
- price/04..08-model: only the south wall is dressed; the west wall stays bare white. Medium (look).
- "no price" on rows with no price: SEEN (06, 07, 08 sheets).
- The rest show what they claim.

## roofw (9 read: 5 model, 2 sheets (02, 04); 03/05 sheets read by diff)
- roofw/02..05-model: the top bar ends at "| Not saved" — the known/assumed/unknown counts that 01 shows ("2 known 65 assumed") are GONE once the price line grows ("no figure 9 not priced 20 lines unpriced 5 to verify, $1,182.98 riding"). The Saved mark stands first, as fixed, but the counts are pushed off the bar. Sure.
- roofw/04-sheet: "Asphalt shingles 3.8025 SQ" (four decimals) beside "380.25 SF". Sure.
- roofw/04-model + 04-sheet: the production-rate field is open in two places at once — a box on the model's right panel ("PRODUCTION RATE OF ... like 40 SF/hr or 0.5 hr/EA") and an open box on the sheet line ("like 40 SF/hr"). A person cannot tell which one the typing goes to. Medium.
- roofw/04-sheet: "Sheathe the roof  no production rate ... unknown ... no price ... no figure" — four red words on one line. Low.
- roofw/01..05-model: "Eave height 9 ft" beside corner heights "9' 0"". Sure.
- roofw/05-model: message "...set to 2 hr/SQ. The same line on plane 2, plane 3 and plane 4 had none: they take it too." clear. OK.
- The rest show what they claim.

## tools (6 read: 6 model; sheets differ only at the foot except 02/05, small)
- tools/02..06-model: once a door is placed the right panel grows (OPEN ITEMS) and covers the wall's east end; the wall now runs under the panel and its end is not seen (in 01 the whole wall shows). Medium.
- tools/02..06-model: doors on the plan are bare brown rectangles in the wall, with no swing and no label; a builder reads a door on a plan by its swing. Medium.
- tools/01..06: "SILL HEIGHT 0 ft" for a door, "0 ft" not 0' 0". Low.
- tools/01 vs 02: Next steps goes from "4 of 10 done" to "3 of 10 done" when the job becomes unsaved — a step un-ticks itself. Low.
- tools/06: "Saved. What is being typed is not in it yet: Enter sets it." — clear. OK.
- The rest show what they claim (01 fields with 6' 8" and 0 ft; 02 a 3' 6" door; 03 fields 3' 6"; 04 red "Nothing was placed: the door's sizes are still open. Press Enter first, then click."; 05 2' 8" door; 06 saved).

## turnw (4 read: 4 model; sheets small changes)
- turnw/02, 03, 04-model: the panelboard on the plan is hidden under its own blue name tag "Panelboard, 200 A"; in 02 nothing of it shows ("outlined in ink" is not visible), in 03/04 only a sliver above and below the tag. The step's subject cannot be seen. Sure.
- turnw/03, 04-model: "Turned Assumed 90 deg / turned (turned by hand in the model)" — word repeated; "deg" rather than a builder's "90°" (minor). Low.
- turnw/02-model: red message "It is in the same space as wall 2 (4 3/8 in into each other)" — clear. OK.
- turnw/04-model: "Brought in turned items.csv: 21 rows read, 1 set on the job. Panelboard, 200 A: changed: its width; the one set down is now that size. 20 rows are the same as the job already has." — clear; size 15 x 4 -> 20 x 4 x 30 in. OK.
- turnw/01: the left panel is scrolled to the item list with its top row cut ("equipment, 30 x 26 x 36 in" half off the top). Low.
- The rest show what they claim.

## Final

Pictures opened (model + sheet). The other sheets were checked by pixel difference, and only the foot line had changed: bring 12, build 8, carry 6, clash 9, d2 11, d4 12, d5 17, draw 16, field 12, first 7, gablew 8, glass 11 (03-09 at half size), house 10, itemlib 5, itemnew 6, items 5, layers 6, money 12, pagew 4, plan 11 (with made-up), price 12, roofw 7, tools 6, turnw 4.

New findings, worst first:
1. field/04, 06: one laser reading, ended by Tab or Return, is followed by "A second reading came straight after that one and was not used. One reading to a value: click the next value, then shoot."
2. first/02 vs plan/04: the same sheet and the same 41' 3" dimension. first says "the drawing is 77' 7 3/4" wide", "1" = 7' 1""; plan says "87' 6 13/16" wide", "1/8" = 1'-0"". One calibration is about 12% off, and nothing warns about it.
3. d4/01-02: Estimate shows "Price no figure ... no allowance comes to a figure" while Budget shows "Allowances $22,033.70". The top bar says "PRICE no figure" throughout. The model's BY DIVISION is empty and has no Allowances row.
4. roofw/02-05, d2, d4, d5: once the price line grows, the top bar ends at "Not saved" and the known/assumed/unknown counts are gone (roofw/01 still shows "2 known 65 assumed").
5. d4/05: "251.56 SF" x "$31.50/SF" is shown as "$7,924.06" (it should be $7,924.14). Budget: south $13,206.78, east allowance $7,043.61.
6. Four-decimal quantities on lines a person edits: d5/07 "461.1890 LB", d2/08 "283.9250 SF", gablew/04 "1.7889 SQ", roofw/04 "3.8025 SQ", money/04 "143.1084 SF", d5/03 "6.7082 hr".
7. roofw/05: "set to 2 hr/SQ" is shown on the line as "at 0.02 hr/SF".
8. d2/08 sheet: every line reads "no cost type  no division" in red, with "it is in no column and no row of the recap".
9. d5/07-12: the copper roof is drawn grey, and "Also priced, not drawn: standing seam, copper" appears twice. After Copper, 20 oz (d5/10) the panel still says "Standing seam copper, 16 oz".
10. turnw/02-04: the panelboard is hidden under its own name tag. draw/10: the "wall 6 45.00 SF" tag covers the selected stray wall.
11. Heights in two ways on one panel: "Height 9 ft" / "Height 9' 0"" (price/04, draw/09, gablew); level "Elevation 0 ft", "Level height 10 ft" next to "first floor 0' 0"" (layers/05); "Eave height 10 ft" next to corners "10' 0"" (d2, roofw).
12. d2/04-07: the corners table shows "NORTH -0' 6"" with a minus, beside "S 0' 6"" on the Eave line of the same panel.
13. gablew/02, price/05: "2 cost lines of the assembly it had came off: Lap siding, Hang the siding."
14. bring/03 sheet: '"buildingstorey" (as the model calls it) #23 in the file'. items/03: "IFC4 ... Not in it: 2 x type ... named among its values". plan/01: "snap to".
15. pagew/01-02: "Back to tmp.srbQRh4SpB"; bring/02: "Back to tmp". pagew/02 picks the page by typing "usda-7213.pdf, 1".
16. roofw/04: the production-rate field is open both on the model panel and on the sheet line at once.
17. clash/01, 05 and field/05 sheets: "12.00 in", "3.50 in", "2.25 in". clash/05: "Air handler, connection 1 ... E 0' 0", N 0' 0", H 2' 0"".
18. house/03: "one system picked out: supply air" looks the same as 02 in 3D.
19. house/07: "Cable 0.4 in round, assumed" and "Height above its level 9.5 ft".
20. itemnew/01: the New item form is mostly under "more below"; only NAME shows. items/01: "It is under Items, on the left" but the entry is off the panel.
21. itemnew/04-05: after the rename, the source line still reads "Wall oven, 27 in: typed in the window ..."
22. tools/02-06: the right panel covers the wall's east end, and doors on the plan are bare rectangles with no swing.
23. carry/04: when clear, the vanity is drawn pale and see-through, and the Clashes switch vanishes from SHOW at 0 clashes.
24. build/03: only "East room 341 SF" is labelled, and the plan label says 341 SF where the panel says 341.25 SF. build/02: the floor step changes nothing on the plan.
25. house/06-07: plan labels overlap ("Utility 53 SF" on a riser, "40' 0"" on interior lines).

Earlier thirty:
1 could not tell (dress skipped). 2 red gone (amber in field/04, 06); "second reading" message still there (field/04). 3 still there (d4/01-02 sheets). 4 still there (d4/05). 5 still there (d4/03, d4/06, gablew/04). 6 still there (d5/07, d5/10). 7 still there (d2/08 sheet). 8 mostly gone; one source line keeps the old name (itemnew/05). 9 coordinates gone (house/03 now "E 2' 0", N 17' 0""); the system still not picked out (house/03). 10 partly gone: "H 24' 0" in" gone, "E 0' 0", N 0' 0"" places remain (clash/05 sheet). 11 still kept on disk; the README now names them. Not opened, per the brief. 12 still there (bring/03 sheet, items/03). 13 still there (pagew/01, bring/02, first/04 sheet "(usda-7213.pdf, page 4)"). 14 still there (draw/10). 15 still there (d5/07, roofw/04, money/04; draw/12 "4.4167 ft"). 16 still there (roofw/05 sheet). 17 gone (price/06-08 "no price"). 18 could not tell (dress skipped). 19 still there (money/03, 05: dark band under the roof). 20 gone (money/01 sheet WHERE reads "Roof"). 21 still there (itemlib/02-04, itemnew/02-05). 22 still there (build/02, 03). 23 gone (build/03, first/04). 24 still there (bring/05). 25 still there (d2/01 "Next: Add a level, 1 of 10 done"; money/01 "Next: Add doors and windows"; roofw/01 "Add a drawing and set its scale"). 26 still there (d2/02 to 03 sheets: 189 known/16 assumed becomes 120/88). 27 still there (first/02 "1" = 7' 1""). 28 still there (carry/02 panel has no clash line; carry/04 "2' 0 11/16" east"). 29 partly: gone in draw and itemlib (W, S); still in d2/04-07 "-0' 6""; glass could not tell. 30 "34 not priced" vs "6 lines unpriced" still there (d2); bring and d4 counts could not tell.

Builder's fixes:
- W/S instead of minus: partly seen (draw/09 "W 2' 1", S 9' 0"", itemlib/02 "W 2' 6""); not in d2/04-07 corners "-0' 6"".
- Amber reading: seen (field/04, 06).
- Saved before counts, never cut off: the order is seen (draw, first, money, layers), but the counts are cut off in roofw/02-05, d2, d4 and d5.
- Allowances row: not seen (d4/01 model BY DIVISION is empty; d4 sheet rows below 7 are not in the picture; d5, money and layers have no such row).
- "no price": seen (price/06-08, money/04, d2/08, roofw/04).
- "its outline counts on ... whose heights are assumed": seen (build/03, 05; first/04-06).
- itemnew new name everywhere: seen, except one source line (itemnew/04 sheet, 05: "Wall oven, 27 in: typed in the window").
- Height in feet and inches: seen in TAKEOFF ("Height 9' 0"", price/04, money/04); not seen for AS DRAWN wall height ("9 ft") or a level ("Level height 10 ft", layers/05; "9 ft", clash, field/01).
- "Detail comes to": seen (d2/08, money/04).
- "not coded" in quiet grey: seen on the model panels (gablew/01, price/04, money/04); not seen on the sheet, where it still has a red mark (money/01 sheet, d5/01 open items).

Could not read: glass 03-09 were seen only at half size; d4's By division rows 8-16 and the Allowances row are not in any picture; dress, quantityw, light, trades and the stale files were skipped as instructed.
