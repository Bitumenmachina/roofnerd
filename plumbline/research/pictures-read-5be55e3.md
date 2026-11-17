# Notes: reading the window pictures (demos/captured), 2026-11-17

## On every picture (so far)
- Top bar title truncated: "Traced from USDA Plan No. 7213 (a public pl..." (plan/01-model, plan/01-sheet, plan/07-model). The same job name in the right panel wraps to two lines, so it fits there; the top bar cuts it.
- Right panel open item "corner on the plan: E 0' 0", N 0' / 0"" breaks the value across two lines (the closing 0" alone on the next line): plan/02-model, plan/04-model, plan/07-model. Sure.
- Sheet header shows known/assumed/unknown counts but no dollars; model top bar shows "PRICE nothing priced yet" next to "4 not priced" (two price statements side by side; reads oddly but not wrong).

## plan (17 read: 8 model + 8 sheet + made-up 3+3 = 22 incl. made-up)
- plan/05-model: length "41' 3"" typed in the Wall field, but the blue preview line on the plan stops short (ends ~x970 with a box) of the 41'-3" dimension's right end (~x1060). The picture shows where the pointer is, not what typing will place. Medium sure (it is a preview; 06 shows the wall placed at the right length).
- plan/06,07 model+sheet: the same length shown two ways on one screen: on the plan "41' 3"", "23' 11"" and in the right panel/sheet "106.42 LF", "130.33 LF", sheet "23.92 LF", "41.25 LF". An estimator reads 23.92 LF fine; flag only as two notations. Low.
- plan/07-model: right panel "Wall 1, type: wall 6 in, and 11 more for the same reason" while there are 4 walls; "33 assumed" in the top bar for 4 traced walls. Count is of values, not things; may confuse. Low.
- plan/08-model: 3D shows the four walls as a plain open box, no top plate/cap colour difference; reads as a box, acceptable for "outside walls". The sheet's floor plan is hidden under the box, so you cannot see the walls stand on the traced lines. Low.
- the rest show what they claim (01 no scale, 02 refusal said in red banner and status line, 04 "1/8" = 1'-0"" set, made-up/01-03).

## Method note
Consecutive sheet pictures were pixel-compared first. Where a sheet is pixel-identical to the one before it, or differs only in the one-line message at the foot (which repeats the model window's foot line), it is recorded as such and the model picture's foot line was read instead. Pixel-identical pairs: d2/04=05, d4/03=04, d5/01=02, dress/one-each/01=02=03, dress/priced/01=02, field/02=03, glass/03=04=05=06=07, glass/08=09 (sheets only; no two model pictures are identical).

## bring (12: 6 model + 6 sheet)
- bring/02-model: the file picker says "Back to tmp" (a folder name off the disk) and "made-up-house  BIM model"; under it "OR TYPE ITS NAME" with "made-up-house.ifc" typed. A folder called "tmp" is a computer word. Sure.
- bring/03,04,05-sheet: open items read `"Ground", "buildingstorey" (as the model calls it) #23 in the file` and `"South wall", wall #44 in the file` -- the IFC record name and record numbers are software words on screen. Sure.
- bring/03-model: "1 x beam: this program has no element for it" (also furniture, stair): "element" is a software/BIM word; "this program" talks about itself. Medium.
- bring/04-model: header "Brought in tinners-yard.xlsx: 5 rows read, 6 set on the job" -- more set than read, no word why. Lines end "...the job had no price for it and takes Tinners Yard's" -- the sentence stops on a possessive (reads cut off). Item names kept as typed in the file side by side: "Copper, 16 oz" and "copper,20 OZ". The message box covers the top half of the house. Medium.
- bring/05: claims "the estimate, kept beside the job": neither window shows it. The sheet has tabs Verify in field / Clashes / Building / Mechanical / Plumbing only; the prior estimate appears only as one line of the message ("kept to be set against this job; it is no part of this job's price"). Sure.
- bring/03-sheet: "Verify in field 19" and the model's "19 to verify", but "Open items 16" on the same sheet (18 in 04, 19 in 05). Two counts of open questions that disagree. Medium.
- bring/03-model..06: right panel rounds to "767 SF", "600 SF", "100 LF" while the sheet shows "767.00 SF", "600.00 SF"; plan set showed "851.33 SF" in the same panel. Low.
- bring/03..06-model: Next steps ticks "Add a drawing (walls drawn without one)" and "Run the ducts, pipes and wiring" as done; no wiring exists. Low.
- the rest show what they claim (01 empty job, 06 "Saved").

## build (12: 6 model + 6 sheet; sheet 04 = 03 and 06 = 05 except the foot line)
- build/02-model: claims "the floor: the outline the walls enclose". The plan looks the same as 01: no fill, hatch or outline marks the floor; only the right panel's "Floor 1 ... 600 SF" changed. Fairly sure.
- build/03-model: claims two rooms; only "East room 341 SF" is written on the plan. The west room has no label on the plan at all. Plan label "341 SF" vs right panel/sheet "341.25 SF". Sure.
- build/03..06 (model right panel and sheet open items): "whether wall 1, wall 3, wall 4 and wall 5 bound this room rests on their heights, which are assumed" -- the sentence does not parse; repeated six times per screen. Sure. (Also bring/03: "whether South wall, East wall, North wall and West wall bound this room rests...")
- build/02,03,05-sheet (and bring sheets): "floor 1  floor 4 in ... 600.00 SF" is listed inside the walls table, just above "Walls, total", in the "NET, ONE SIDE" column; the total (960.00 SF) leaves it out without saying so. Medium.
- build/03-sheet: room table "PERIMETER 62.00 LF" sits beside "TO WALL CENTERS 240.00 SF" -- the perimeter is inside the walls (11.5+19.5)x2, the area to centres; the headings do not say which the perimeter is. Low.
- build/06-model: 3D hip roof shown plain grey with coloured edge lines, eave flush with the wall faces (no overhang); acceptable for the step. Foot line at lower right reads "3D  E 7' 0"  N 7' 0"  H 0' 0"" in 3D. Low.
- the rest show what they claim (04 roof outline about to close, 05 roof on the plan with eave/hip/ridge).

## carry (10: 5 model + 5 sheet)
- carry/02-model: the vanity is selected and drawn red, but the right panel (Harbor Vanity 36: What it is, Position, Height, Size) says nothing about it standing in the water closet; the "Clashes 1" switch at left is off. A person sees red and no word why. Medium-sure.
- carry/04-model: claim "let go a foot and a half east"; the screen says "Moved Harbor Vanity 36 2' 0 11/16" east, 0' 1 7/8" north" and Position "E 9' 0 11/16", N 1' 0 3/8"". It went two feet and also drifted 1 7/8 in north. The words are consistent with each other; the comment's figure is not what happened. Sure of the words.
- carry/04-model: the moved vanity is drawn as a pale translucent box, not the wood it was in 01; reads as a ghost of a thing rather than the thing. Low-medium.
- carry/03-sheet: the foot line is cut: "...Keep going, or let go and i..." Sure (the sheet window's foot line truncates long messages; also seen elsewhere).
- carry/01..05-sheet: "Verify in field 3" (2 in 04) beside "Open items 6"/"7": two counts of what is open, different. Low (same as bring).
- carry/01: level named "ground" in lower case while every other job says "Ground". Low.
- the rest show what they claim (03 the clash said in red banner, 05 undone and red again).

## clash (14: 7 model + 7 sheet; sheet 02 = 01 and 07 = 06 except the foot line)
- clash/04..07-sheet: open items give places that cannot be where the things are drawn: "6 in round elbow, connection 1 (duct): E 0' 0", N 0' 0", H 3' 0" in" (the elbow is drawn at about E 10', N 2' on the plan) and "Air handler, connection 1: Supply Air (duct): E 0' 0", N 0' 0", H 24' 0" in" (on a level 9 ft high). They read as positions on the job but look like offsets from the thing. Also "H 3' 0" in" puts a unit after feet-and-inches. Sure of the words; the meaning is what a person would misread.
- clash/03..07-sheet: "6 in round elbow, what it is: 6 in round elbow" and "6 in round tee, what it is: 6 in round tee" -- says nothing. Status line clash/04: "Added 6 in round tee, a tee; duct 1 cut there." Low.
- clash/06-model: pipe 1 selected; the right panel (Type, Line on the plan, Height, System, True length) does not say it is in the duct, though an orange cross sits on it and the sheet highlights "duct 1 and pipe 1 3.50 in". Medium.
- clash/06-model: pipe "Type 1 in" -- the size is shown under the heading Type. Low.
- clash/07-model (3D): the 1 in pipe is drawn as a hair-thin line, not a pipe; the ducts are round tubes. The pipe reads as a guide line, not a thing; hard to see what it is clashing with. Medium.
- clash/01..07-model: the right panel shows the level "ground" (Elevation, Level height) for the whole run; the two clashes the 01 comment calls "listed" are listed only on the sheet and as "Clashes 2 / 2 findings" under Show. Low.
- clash/03-model: Next steps count fell from "5 of 10 done" to "4 of 10 done" because adding a fitting un-ticked Save. A count that goes backwards when you add work is odd. Low.
- the rest show what they claim (02 fitting in hand, 03 elbow, 04 tee, 05 one clash left).

## d2 (16: 8 model + 8 sheet; sheets 02=01, 05=04 pixel-identical or foot line only)
- d2/07,08 (sheet): the starter assembly "Standing seam copper roof" puts lines on every plane that read "no cost type" "no division" in red, each with "no cost type: it is in no column and no row of the recap"; the sample house's own lines ("Copper standing seam, installed", "Copper headwall flashing") are the same. "By division 0 of 16 with work in them". In a program whose estimate reads division by division, the shipped sample and starter assembly cannot reach any division. Sure of what is on screen.
- d2/08: the same line's quantity two ways: model right panel "Copper standing seam panels ... Quantity Assumed 283.9250 SF" (four decimals) and on the sheet the same line "Copper standing seam panels  unknown". Underlayment: "283.9250 SF" in the panel, "283.93 SF" on the sheet. Sure.
- d2/08-model: right panel "(the line has no price (no price entered))" -- nested brackets; "Comes to, detail ... unknown", "Comes to, detail rate ... detail ÷ surface area". Low.
- d2/08-model: on plan, the selected plane 1 ("1 284 SF 9:12") is at the bottom edge and the plan runs under the tool bar; the lower part of the selected plane and its eave are hidden behind the bar. Medium.
- d2/04..08-model: corners and eave line read "N -0' 6"" and "E 100' 0", N -0' 6" to E 140' 0", N -0' 6"" -- a minus sign in front of feet-and-inches. Medium-low.
- d2/01..08-model: Next steps collapsed to "Next: Add a level" and "1 of 10 done" on a house that already has walls and a 6-plane roof. Wrong advice on a job that is built. Medium.
- d2/01..08-model: top bar shows "34 not priced" and "6 lines unpriced" side by side (two different "not priced" counts). Low-medium.
- d2/03-model: one pitch raised 6:12 to 8:12 and the top bar went from "189 known 16 assumed" to "120 known 88 assumed": 69 Known values became Assumed from one click, unexplained on screen (the 04 banner explains it for the eave move, 03 has no such banner). Medium.
- d2/02-model: right panel cut off at the foot ("Slope factor 1.12", next line under "more below"); fine as a scroll.
- d2/05: sheet identical to 04 while a pitch is typed (expected). The rest show what they claim (02 plane picked, 03 8:12, 04 eave out 6 in with a clear banner, 06 9:12 set, 07 copper on the planes, 08 on plan).

## d4 (14: 7 model + 7 sheet; sheet 04 = 03)
- d4/02..07: the sheet's Budget tab totals "Detail over 3 items $16,761.85" (later "$24,685.91") and "Allowances ... $22,033.70" while the Estimate tab of the same job (d4/01-sheet) says "Job cost none", "Price no figure", "By division 0 of 16 with work in them", and the model's top bar says "PRICE no figure". The same job reads as $24,685.91 of detail on one tab and nothing at all on the other; the reason (lines have "no cost type") is only in small print. Sure.
- d4/05: hand check fails by cents on screen figures: east "251.56 SF" x "$31.50 / SF" = $7,924.14, the sheet and panel show "$7,924.06"; south 419.26 x 31.50 = $13,206.69, shown "$13,206.78"; east allowance 251.56 x $28.00 = $7,043.68, shown "$7,043.61". The money is worked on the unrounded area; the area shown is rounded, so an estimator checking with a calculator gets a different figure. Sure of the arithmetic.
- d4/05,06: top bar keeps "2 to verify, $3,555.07 riding" while the sheet says "Detail ... $11,479.13 of it rests on something assumed": two dollar figures for "resting on assumed values", not explained side by side. Medium.
- d4/03..07-model: the selected plane (east in 03-05, north in 06-07) faces away from the camera; it is shown only as a blue outline drawn through the near planes, with its labels "22' 6"", "30' 0"", "40' 0"" floating over the south and west planes. The view does not turn to the thing picked from the sheet. Medium-high (a person cannot see what they selected).
- d4/06-model: north given an allowance; its panel still says "COST no figure" while the sheet shows north's allowance $11,739.36. Medium.
- d4/01..07: the sheet lists "shed 98.95 SF", "headwall 12.00 LF", "Parapet-south"; the 3D house shows only a plain four-plane hip roof on a box; there is no shed, headwall or parapet to be seen. Medium.
- d4/02-sheet: "$42.00 under -8.75 %" -- "under" and a minus sign together. Low.
- d4 right panel: "Schedule code  Unknown  schedule not built" -- what a schedule code is is not said; reads as a software field. Low-medium.
- the rest show what they claim (04 typing 31.50, 05 the price set, 07 "Can't set that price: "nonsense" is not a price; write it as a number of dollars, like 4.25 or $4.25/SF").

## d5 (24: 12 model + 12 sheet; sheet 02 = 01; model 01-04 the same view, as expected for sheet steps)
Arithmetic checked by hand on 02, 04, 06, 12: recap rows add to job cost ($28,442.43); divisions + no division add to it; 12% overhead $3,413.09 and 8% profit $2,548.44 are right; supplier totals add; after steps 07-11 the job cost $28,361.09 matches the sum of the four changes. Good.
- d5/03,04,06,12-sheet: "Dumpster 2 EA $1,300.00" and "Permit 1 EA $1,850.00" sit under "NO DIVISION" / red "In no division: in the job cost, and in none of the sixteen ... $3,150.00", while "General Requirements" (Division 1) is right above with only the foreman. A dumpster and a permit are Division 1 work to any estimator; the starter sample marks them as errors in red. Sure of what is on screen.
- d5/07..12-sheet: quantities with four decimals beside the same quantity with two: "461.1890 LB", "576.4863 LB", "276.7134 LB", "123.6931 LB", "251.5576 SF" (west subcontract line) while the item list above shows "251.56 SF", "861.60 LB". Sure.
- d5/07..12-model: the roof's finish is copper standing seam (sheet: "made of Copper, 16 oz", assembly "Standing seam copper, 16 oz") but the roof is drawn dark grey; the panel says "Finish: Standing seam" then twice "Also priced, not drawn: standing seam, copper". In d4 the same sample roof is drawn copper-brown. Confusing; medium-high (the rule is the house looks like what is priced).
- d5/10,11,12-model: after "South: material set to Copper, 20 oz" the panel still lists "Assembly Standing seam copper, 16 oz" and "Also priced, not drawn ... From its assembly, Standing seam copper, 16 oz" next to "From its material, Copper, 20 oz". Two gauges for one plane. Medium.
- d5/03,07-sheet: "Foreman, days on site  40.00 hr  $95.00" and on the sheet "Supervision Div 1 at 8 hr/EA  40.00 hr  $95.00/hr": a line named in days, counted in hours, with a rate "8 hr/EA". Medium-low.
- d5/04-sheet: suppliers side by side: Tinners Yard total "$6,714.91" in the same Total row as the others' $7,861-$8,254, though it covers "2 of 4" lines (said in red underneath). The lowest-looking total is the incomplete one. Low-medium (it is flagged).
- d5/08-model: "headwall" selected; nothing on the 3D house is highlighted and no headwall is visible (the house is a plain hip roof). Medium.
- d5/06-sheet: under Price "it leaves out 1 line, named below: it is the price of the lines in the job cost, not of the job" -- hard to parse. Low.
- d5/12: refusal shows ("Can't set the overhead: "ten" is not a number") but the right panel shows the south roof plane, not the overhead the person typed in; no example of what to type (the price refusal in d4/07 gives one). Low.
- the rest show what they claim.

## draw (30: 15 model + 15 sheet)
- draw/12-model: window 3 selected: "Center, along the wall  4.4167 ft" -- a raw four-place decimal where every other length on screen is feet-and-inches. Sure.
- draw/10-model: "the stray wall selected": on the plan only the blue label "wall 6 45.00 SF" floats in empty space below the house; no wall line can be seen (enlarged: nothing but the label). If it is under the label, the label hides the very thing selected. Medium-high.
- draw/09..14 (model panel and sheet open items): "Wall 5, wall line: E -2' 1", N -9' 0" to E -2' 1", N 7' 0"  entered on screen; not yet checked against a drawing; moved (entered on screen; not yet checked against a drawing)" -- negative feet, and a nested, repeated source note. Medium-low.
- draw/02..15-model: right panel "Ground, elevation: 0 ft, and 36 / 50 / 58 more for the same reason" -- one open item standing for 59 values; reads as one thing to do while the top bar says "24 to verify". Low.
- draw/06,07-model: Next steps ticks "Run the ducts, pipes and wiring" as done after one duct and one pipe; "Add the floor and name the rooms" stays highlighted as next though the person has moved on. Low.
- draw/15-model (3D): the house is open boxes with no floor, the door shows, the duct floats as a blue bar at top-of-wall height; the windows face away. The comment "the house in 3D" holds. The left Next steps list over the 3D still says "Add the floor..." Low.
- draw/01-sheet: the empty sheet says only "This job has no levels: no walls or floors are entered." Fine.
- the rest show what they claim (02-04 walls typed and drawn, 05 openings, 08 "Undone: Added Air handler." and the box gone, 09 partition and door a foot east, 11 "Deleted wall 6.", 13 window gone (2 windows, net 798 SF), 14 refused in red at the field: "Can't use that system: "Steam" is not a system a conduit goes on. There are: Power, Lighting").

## dress (16: one-each 4+4, priced 3+3, bare 1+1; sheets one-each 01=02=03, priced 01=02)
- dress/one-each/03-model: claims "the north wall in shingles over a stone wainscot, the east in white lap siding". The picture is a full-screen close-up of the grey standing-seam roof; no wall is in it at all. The turn put the camera into the roof. Sure. (The shingles, stone wainscot and lap siding of this job are never seen in any picture of the set.)
- dress/one-each/04-model: claims "its Finish, in the panel" for the south wall picked at 560,560. The picture is again a close-up inside the roof; what is selected is "ROOF PLANE plane 1 ... Standing seam, charcoal". The wall was not picked; the panel shows the roof's finish, not the brick's. Sure.
- dress/one-each/04-model: plane 1 "COST $14.00  ALL IN $0.08 /SF" for 183.88 SF of standing seam; the recap gives "Thermal and Moisture Protection 10 lines $145.50", Masonry "$119.00" for a brick wall. Prices are invented but read absurd to an estimator. Low-medium.
- dress/one-each/01..04-model: top bar "PRICE no figure  ■ Unknown  6 not priced" -- a bare "Unknown" with nothing after it. Medium-low.
- dress/priced/03-model: claims "from above: the seams run up each plane"; the picture is the same three-quarter view as 01 (foot line "The whole job, in view."), not from above. The F key reframed instead. Sure.
- dress/priced/01-02: shows what it claims: lap siding over a stone wainscot, double-hung windows with frames, entry door. Good. Roof is grey standing seam (fine).
- dress/bare/01-model: shows a plain white house, glass with no frames, roof plane labels "4 252 SF 6:12", "1 453 SF 6:12": as claimed. The bare job has no foundation level while the priced copy of "the same house" has one with footings; low.
- dress/bare/01: open item "hot to the kitchen sink, elbow 1 is not drawn ... its size is not known" -- run named "hot" in lower case, reads as a fragment. Low.

## field (14: 7 model + 7 sheet; sheet 03 = 02)
- field/04,06,07 (model banner, model foot line, sheet foot line): a measurement taken successfully is announced in red, error styling: red bar, red text "Ground: level height is 8' 10 1/2", measured (it was 9 ft, ...)", red square at the foot. Every refusal elsewhere uses the same red. A person reads their good measurement as a failure. Sure.
- field/04 and 06 banner: "A second reading came straight after that one and was not used. One reading to a value: click the next value, then shoot." on both measurements. The person is told a reading was thrown away on the normal path. Sure of the words (whether the script sent two readings is not mine to say).
- field/04-model: after measuring only the level height, "Elevation 0 ft ... scaled off a made-up plan; found as drawn (checked on site)" turned Known (the Assumed marker is gone); top bar 37 known -> 56 known from one reading. field/06-model: after measuring only the duct's height, "Type 12 x 8 ... found as drawn (checked on site)" and "Line on the plan ... found as drawn (checked on site: JM)" turned Known too. Values the person did not measure show as checked on site. Sure of what is on screen; medium-high as a problem (it may be a deliberate rule, but nothing on screen says the person confirmed them).
- field/05,06-model: duct "Type 12 x 8" with no unit. Low.
- field/01..07-model: left panel "SYSTEMS  On no system 44.00 LF". Reads oddly. Low.
- field/02-model: identical to 01 except the orange clash crosses on the duct are gone; the comment "before: the level's values Assumed, the price flagged" holds (top bar "$615.86 Assumed"). Low.
- field/04-sheet: "By division 0 of 16 with work in them; 1 line of the job cost is in no division" -- the job's one priced line is in no division. Low (same as d5).
- the rest show what they claim (03 the Measured on site field, 05 the duct and two clashes 2.00 in / 5.00 in, 06 one finding 2.25 in, 07 one question "upper (level) ... no dollars").

## first (12: 6 model + 6 sheet)
- first/02..06-model: the scale of the drawing is shown as "usda-7213, page 4  1" = 7' 1"" (left panel). No trade writes a scale that way; the sheet itself is printed 1/8" = 1'-0" (its own title reads "SCALE 1/8"=1'-0"" under FOUNDATION PLAN). The made-up plan in plan/made-up/03 shows the title-block form; this one does not. Medium.
- first/02-model foot line: "usda-7213, page 4: 41' 3" between the two points; the drawing is 77' 7 3/4" wide" -- fine, but "the drawing" here is the whole sheet, not the house. Low.
- first/03..05 (panel and sheet open items): "traced from usda-7213, page 4 (usda-7213.pdf, page 4)" -- a file name with ".pdf" on screen, and the name said twice. Low-medium.
- first/04,05 (panel and sheet): "Floor area of House: 954.23 SF / whether wall 1, wall 2, wall 3 and wall 4 bound this room rests on their heights, which are assumed" -- the same unparseable sentence as build/03. Sure.
- first/04-sheet: the whole house is one room called "House"; the comment says "the floor and the room": as claimed. Room perimeter "128.33 LF" vs walls "130.33 LF": not explained (inside vs centre). Low.
- first/06-model (3D): the house stands on the sheet as claimed; the roof sits flush on the walls with no overhang; one door and one window visible. Acceptable for a first takeoff.
- the rest show what they claim.

## gablew (8: 4 model + 4 sheet)
- gablew/03,04-sheet: shingle quantities to four places: "1.4311 SQ", "0.7155 SQ", "1.7889 SQ" beside "143.11 SF"/"178.89 SF" underlayment. Roofers write 1.43 SQ (or round up to the bundle). Sure.
- gablew/04-model: claims "a gable: three planes". The camera is unchanged and looks at the south and west faces; the east end, where the gable is, faces away, so the gable end (and any wall under it) cannot be seen. The roof looks almost the same as 03. A person cannot see the change they made. Medium-high.
- gablew/04 banner: "the job cost was $429.32 and is $429.33" -- turning a hip into a gable at the same pitch adds one cent; the area of a roof at one pitch over one footprint does not change, so the cent is a rounding artefact shown as a change. Low-medium.
- gablew/01..04-model: top bar "12 not priced  24 lines unpriced" -- two "not priced" counts side by side. Low (also d2).
- gablew/02 banner: "2 cost lines of the assembly it had came off: Lap siding, Hang the siding. $864.00 is out of the job cost (1 of them had no price yet)" -- reads fine; foot line truncated "...$864..." (sheet foot "it had cam..."). Low.
- gablew/01-model: right panel "Framed in 2x6 at 16 in on center" with the Framing switch off: fine.
- the rest show what they claim (01 lap siding drawn and priced $4.00/SF $864.00, 02 brick drawn, 03 east plane 2 outlined with its corners).

## glass (23: 9 model + 9 sheet + 5 step pictures; sheets 01-09 all the Roof tab, unchanged but for the foot line)
- glass/step1, step2, step4, step5, step6 (stale pictures kept in the set): the program is named "plumbline" in the top bar, the job file name "job.json" is on screen, wall line "(0, 0) to (24, 0) ft" and position "(7, 0.875) ft" as coordinates in brackets with decimals, "Model (IFC)" / "BIM model (IFC)" / "Estimate (CSV)", the roof's planes called "FACETS", "Doors and windows 5 and 8", step4's whole house priced $468,157.63 against $468,876.85 in dress/priced/01 for the same sample, and step5 the vanity in plain red/white boxes. These do not match the current program; anyone reading the set will take them as the program. Sure they differ; they are not labelled stale.
- glass/04,05-model: claims "three notches in, at the door" then "the wheel over the right panel: the model has not moved": 05 is pixel-for-pixel the same view as 04 (as intended). But the foot line still says "Roof: switched on." in 04-07: the last message lingers through four camera moves. Low.
- glass/04..07-model: the camera moves forward into the house; in 04-05 the roof's eave/fascia cuts across the top half and the house fills the screen behind the panels; the left and right panels cover most of the house's two ends. Expected for a close view. Low.
- glass/06,07-model: the roof plane label "1 184 SF 8:12" is cut by the right panel at 07 ("1 184 SF"). Low.
- glass/08,09-model: door 1 selected: "Type door 3 ft" -- a door type named by width only; height 7.00 ft is shown in decimal feet "7.00 ft", "3.00 ft" where a carpenter says 3' 0" x 7' 0" (other panels use 6' 8"). Low-medium.
- glass/08,09-model: foot line position "N -0' 3"" -- minus sign on feet-inches. Low.
- glass/02: the roof off: shows walls only, as claimed. 03 roof on again, identical to 01: as claimed. 06 a turn about the door, 07 slid, 08 flown to the door, 09 the view carried on to the left after a flick: as claimed.

## house (16: 8 model + 8 sheet; takeoff.txt not read)
- house/03-model: claims "one system picked out: supply air". The 3D view is unchanged from 02: every system (orange power, purple refrigerant, pink return, cyan water) is drawn at the same strength; supply air is not singled out on the model. Only the right panel changed. Medium-high.
- house/03-model right panel, Supply air connections: "first-floor trunk, west, piece 3 of 3: its second end ... 2, 17, 8.4", "first-floor trunk, east: its second end  open end  38, 17, 8.4", "attic trunk: its first end open end 4, 14, 20" -- bare numbers with no unit, no E/N/H, decimals. A software coordinate on a builder's screen. Sure.
- house/03-model: "Supply air / Supply Air" title and subtitle repeat the name with different capitals; house/04 "Hot water / Domestic Hot Water". Low.
- house/04-model: claims "hot water, with its run of no size": the panel shows "Pipe, size unknown 9.40 LF Unknown" and fittings "not said  bend  Unknown 2 EA". The hot-water lines are drawn red on the model; whether the 3D view singles out hot water is no clearer than 03 (the ducts go grey, other systems remain). "not said" as an item name is odd. Low-medium.
- house/07-model: a cable selected: "Type  Cable 0.4 in round, assumed" -- an electrician names cable 12/2 or 14/2 NM-B, not a 0.4 in round; "Height above its level 9.5 ft" decimal feet. The sheet jumps to the Building tab, not Electrical. Medium-low.
- house/07,06 sheet: rooms named "living room", "kitchen", "bath" in lower case on the sheet while the plan writes "Living room", "Kitchen", "Bath". Walls "partition at 14", "partition at 24" (named by a coordinate). Low.
- house/06-model (plan): roof switched off for plan, rooms labelled with area, fine. A large square with a circle sits outside the east wall (the outdoor unit) with no label. Low.
- house/08-model: "on foot in the living room": shows a white room with two windows, a bare blue duct run below the ceiling and a door at right; reads as a room. Fine.
- the rest show what they claim (01 the house, 02 seen through with the runs, 05 the first floor alone in 3D).

## itemlib (10: 5 model + 5 sheet)
- itemlib/02,03,04,05-model: the vanity set down is a box about 35 px across in the middle of an empty ground, seen from far off. The comment "a cabinet under a worktop" (02) vs "drawn as its box" (04) differs only in a thin white top on a 35-px box (enlarged: 03 has a pale top, 04 none). At the size shown, a person cannot see the change they made. The camera was not brought to the thing. Medium-high.
- itemlib/05: "said, with where it is and what to do with it": message "Wrote bench items.csv, beside the job: the job's items, 21 rows under their headings. Open it, change a size or add a row, and bring it back in with Item library, under Bring in." -- names a file "bench items.csv" and tells the person to open a file outside the program, without saying where "beside the job" is on their computer. Low-medium. (It is said, as claimed.)
- itemlib/02,03 left panel: items listed by trade "11 EQUIPMENT", "12 FURNISHINGS", "15 MECHANICAL", "16 ELECTRICAL" -- a bathtub, lavatory, water closet listed under "15 MECHANICAL" next to an air handler; a 1995 MasterFormat reading, fine for an estimator but plumbing fixtures under "MECHANICAL" may surprise. Low.
- itemlib/02-sheet: "Vanity, 36 in, position: E -2' 6", N 12' 3 5/8"" -- minus on feet-inches. Low.
- itemlib/01-model: right panel open items "Range, 30 in, maker: unknown, and 41 more for the same reason" -- on an empty job with no range placed, the open items are about library items not in the job. A new user sees 42 unknowns and 63 assumed on a job with nothing in it. Medium.
- itemlib/03: "Trade: Division 12 Furnishings", sizes Assumed, "Drawn as base cabinet": as claimed.

## itemnew (10: 5 model + 5 sheet)
- itemnew/04,05 (model right panel and both sheets): after "The item is now called "Wall oven, single". The one set down is called that too.", the open items still read "Wall oven, 27 in, position: E -2' 6", N 12' 3 5/8"" and "Wall oven, 27 in, size: 27 x 24 x 29 in" -- the old name, next to "Wall oven, single, maker: unknown". The message says the one set down was renamed; the screen shows it was not. Sure.
- itemnew/01-model: claims "the form, four fields in: a comma in its name is a comma". Only the NAME field ("Wall oven, 27 in") and the label KIND with its field cut off at the panel's foot can be seen; the other three fields are below "more below". The form a person is filling is mostly out of sight. Medium.
- itemnew/05-model: "Took the item "Range hood" out of the job" -- no Range hood was ever in the list on screen (only Wall oven); the message is about an item the person never saw. Also the message is orange (warning colour) for a deliberate deletion. Medium-low.
- itemnew/03..05: the placed oven is a grey box ~30 px across in the middle of an empty ground; it is not selected or highlighted, so "the one set down" in 04 cannot be told apart. Low-medium.
- itemnew/02: "Drawn as: box, its name saying nothing else" -- awkward. Low.
- the rest show what they claim (02 sizes Assumed, "Trade not said", 03 "Can't delete the item: one is set down from Wall oven, 27 in: take it out of the job first").

## items (8: 4 model + 4 sheet)
- items/03-model: message "Wrote job.ifc, beside the job: the building as a BIM model (IFC4, lengths in feet). ... Not in it: 2 x type (each thing is written with its own size; the type it took it from is named among its values)." -- "IFC4", "job.ifc", "2 x type", "values" are software/format words; "Not in it: 2 x type" cannot be understood by an estimator. Sure.
- items/04-model: claims "the last of the six written". Message: "Wrote job for the PM.xlsx ... The job has no lines yet: the file has its headings and nothing under them." -- a file name with a space and ".xlsx" glued to "PM", and a file written empty. It is said, as claimed; reads oddly. Low-medium.
- items/02-model: the vanity from the maker's sheet is drawn as a pale translucent box (as "a cabinet under a worktop" it shows only a slightly lighter top band), under "NO TRADE SAID" in the Items list although the sheet named it "a furnishing" (Div 12). Low-medium.
- items/01-model: the right panel shows the item's maker, model, sizes and weight as open items while nothing has been placed; fine.
- items/01..04-model: Next steps says "Add a drawing and set its scale" as the next step all through; fine.
- the rest show what they claim.

## layers (12: 5 model + 5 sheet + plan-panels + estimate-window; sheets 01-05 the same recap, foot line only)
Recap arithmetic checked on layers/01-sheet: 10% of $394,677.48 = $39,467.75; 8% of $434,145.23 = $34,731.62; price $468,876.85. Right.
- layers/plan-panels.png and layers/estimate-window.png (stale pictures kept in the set): "plumbline" as the program's name, "FACETS", "Doors and windows 5 and 8", "0 and 0" for the foundation. They do not match the current window (layers/01 says "BuildOS", "ROOF PLANES", "5 doors, 8 windows"). Not labelled stale. Sure.
- layers/plan-panels: on plan, the roof's planes are drawn solid dark grey over the whole house; no wall, room or opening can be seen through it. Low-medium (a plan normally shows the floor with the roof dashed overhead, as first/05 does).
- layers/02-model: "the finish off: the weather barrier, and flashing at the openings": the walls go a flat pale blue-grey and each window gets a dark grey frame (enlarged). It shows a change; a person would not read the flat blue-grey as house wrap without being told (nothing on screen says "weather barrier"; the foot line says only "Finishes: switched off."). Medium-low.
- layers/04-model: ground off: the grey foundation wall below the stone shows, as claimed. A short blue line sticks out of the foundation at the left corner (x~455, y~597) in mid-air (a pipe end?), unlabelled. Low.
- layers/05-model: "the first floor alone, looked down into": as claimed; the first floor's right panel shows "Elevation 0 ft" and "Level height 10 ft" in decimal feet while the left panel says "0' 0"" and "10' 0"" for the same values. Low.
- layers/01 left: "foundation  -4' 0"" fine. Recap "Labor none" for a priced whole house with 480 hr supervision: everything is material or subcontract -- reads odd to an estimator but is not wrong on its face. Low.
- the rest show what they claim (03 the walls as frames).

## light (4: before-whole, after-whole, after-street, after-window; hand-taken, no shot comments)
- light/* (all four): an older program: "plumbline" in the top bar, "FACETS", "Doors and windows 5 and 8", "TO VERIFY ... 227 for the same reason. The first: The job cost code:" cut off at the panel's foot, price $468,157.63 (current sample: $468,876.85). Stale pictures in the captured set.
- light/before-whole vs after-whole: I can see almost no difference: the siding is a touch lighter in "after". A person shown the pair would not see what changed. Medium-low.
- light/after-window: the window's upper sash shows no meeting rail lock; the sashes look plain; reads as a double-hung window with frame and casing. Fine.

## money (12: 6 model + 6 sheet)
Arithmetic checked: 03 $503.55 x 1.20 x 1.10 = $664.69; 05 $71.55 x 1.2 x 1.1 = $94.45; 06 $71.55 x 3.00 x 1.10 = $236.12. Right.
- money/01,02-sheet: the Verify in field list has a WHERE column, and every one of its five rows is blank there. The list is meant to say where each question is. Sure.
- money/02..06-model: after the eave height is measured at 9' 2" (walls stay 9 ft), the roof is drawn sitting on a dark band above the wall tops (enlarged 01 vs 02: in 01 the roof edge meets the wall; in 02 a dark grey strip lies between). The walls do not rise to meet it and nothing on screen says the walls and the roof now disagree by 2 in. Medium.
- money/01-model: claims "its values say Measure and Check": they do. The Main roof's eave height "9 ft" vs "9' 2"" later: fine.
- money/04-sheet: the new labor line "Hang the board 5.40 hr" sits under "NO DIVISION" with "prices vary" in the unit cost column and "no figure" in red. "prices vary" for a line with no price at all reads wrong. Also "The estimate, by item" shows gypsum at "$2.00" with no unit while the sheet below writes "$0.50/SF". Medium-low.
- money/03,04,06-sheet: "Underlayment 143.1084 SF" on the sheet line vs "143.11 SF" in the item list above it, same screen. Sure (same as d5).
- money/06: an overhead of 200.00 % is accepted (marked Assumed, with "If 20.00 % was meant, enter it again") and the price goes to $236.12 on a $71.55 job cost. The comment says "taken, and answered" -- it was. Flag only that a price triples on a likely typo while the top bar shows it as a normal price in orange. Low-medium.
- money/05-model: deleting wall 1 leaves the south side open and the roof's south eave hanging over nothing; expected from a delete; message "Deleted wall 1. 2 cost lines went with it: $432.00 is out of the job cost (1 of them had no price yet)." Fine.
- money/01..06: Next steps "Next: Add doors and windows" for a job that is being priced and verified. Low.

## pagew (6: 3 model + 3 sheet)
- pagew/01,02-model: the file list's way back out reads "Back to tmp.PoWUBb1H4G" -- a scrambled temporary folder name off the computer. Sure. (bring/02 had "Back to tmp".)
- pagew/02-model: after picking a page from the list, the "OR TYPE ITS NAME" box shows "usda-7213.pdf, 1" -- a file name with ".pdf" and a page number in a computer-style form, while the four pages are listed above. Low-medium.
- pagew/03-model: the message "its page is read (it is a scan: there are no drawn lines on it to snap to); 2 things not read from it: select the drawing to see what" -- "2 things not read" without saying what; fine that it says where to look. Low.
- pagew/03-model: the plan sits on a white grid with the sheet in full; the right panel's open item "usda-7213, page 4, scale: none / not set: nothing can be traced over it" is clear. Fine.
- the rest show what they claim (01 the list by name, a folder, the way back; 02 four pages; 03 page 4 under the level).

## price (16: 8 model + 8 sheet)
Checked: 216 SF / 40 SF/hr = 5.40 hr; 180 SF x $24.00 = $4,320.00; gypsum 180/32 -> 6 sheets; sheathing 216/32 -> 7 sheets. Right.
- price/02-model: claims "the starter assemblies brought in". The house is unchanged and Next steps went from "6 of 10 done" to "5 of 10 done" (Save un-ticked). Only the message says anything happened. Low.
- price/04..08 (sheet "The estimate, by item"): every unpriced line says "prices vary" in the unit-cost column (e.g. "Wall framing, 2x6 at 16 in 216.00 SF  prices vary  no figure"). Nothing varies; there is no price at all. A person reads "prices vary" as several prices on file. Medium. (Also money/04.)
- price/04..08-model: the wall's own assembly, which the person picked from the list, shows "Assembly  Assumed  Exterior wall, 2x6, lap siding / entered on screen; not yet checked against a drawing". A choice the estimator made is called Assumed and awaiting a drawing. Low-medium.
- price/04-sheet: the recap's Price block says "there is no job cost: no line with a cost type and no allowance comes to a figure" and "Nothing is priced on 18 things that could be (3 walls, 1 door, 1 window, 4 roof planes, ...)" and "Left out of the job cost: wall 1: "Wall framing, 2x6 at 16 in": the line has no price (no price entered)" -- right, but long, repetitive, with "(no price entered)" after "has no price". Low.
- price/06: the unit-cost field is open in both windows at once: the model's right panel ("UNIT COST OF "BRICK VENEER"  24|") and the sheet line ("24|" in the Unit cost box). Two typing boxes for one keystroke on two screens. Medium-low (may be deliberate; a person will wonder which one they are typing in).
- price/07,08-model: after the brick's price is set ($4,320.00 on the sheet), the wall's panel still shows "COST no figure" in red and the top bar "PRICE no figure". It is right by the rules (8 lines unpriced) but the person who just priced the wall sees no number on the wall. Low-medium.
- price/03-model: the assembly list for a wall shows "9 lines", "10 lines" etc. but no prices, which fits "with no prices". Fine.
- the rest show what they claim (04 lap siding drawn on the south face, 05 brick drawn and the message "2 cost lines of the assembly it had came off: Lap siding, Hang the siding", 08 5.40 hr).

## roofw (10: 5 model + 5 sheet)
Checked: plane 1 (40+14)/2 x 13 x 1.0833 = 380.25 SF; 3.8025 SQ x $105.00 = $399.26; sheathing 380.25/32 -> 12 sheets; 380.25 x 0.02 = 7.61 hr. Right.
- roofw/05: the rate typed was "2 hr/SQ" (message: "production rate ... set to 2 hr/SQ") but the sheet line shows "at 0.02 hr/SF". The person's own figure comes back in another unit. Comment says "a rate said in squares": said in the message, not kept on the line. Medium.
- roofw/03,05-sheet: "Asphalt shingles 3.8025 SQ" -- four decimals on squares. Sure (same family as gablew, d5, money).
- roofw/01..05-model: the selected plane 1 ("1 380 SF 5:12") faces the camera and is outlined -- fine; but its panel shows "COST no figure" after $105.00/SQ is set (because other lines on it are unpriced); the top bar shows "5 to verify, $1,182.98 riding" while "PRICE no figure". Low.
- roofw/02-model: the shingle texture is drawn on all four planes as claimed; the eave/fascia band is plain grey. Fine.
- roofw/04-model: the field says "How fast your crew does it, like 40 SF/hr or 0.5 hr/EA" -- as claimed.
- roofw/01..05: Next steps "Add a drawing and set its scale" as next on a roof-pricing job. Low.
- the rest show what they claim.

## tools (12: 6 model + 6 sheet)
Checked: door 1 centred at 6 ft, door 2 at 18 ft, 2' 8" wide, measured off the plan (37.8 px/ft); openings 23.33 + 17.78 = 41.11 SF; net 174.89 SF. Right.
- tools/02..06-model: the 24 ft wall's east end runs under the right (Job) panel once that panel grows with open items; the wall's end and its corner cannot be seen. Low-medium.
- tools/05,06: the open item reads "Door 1, type: door 3' 6" by 6' 8", and 9 more for the same reason" after door 2 (2' 8") is added; door 2 is hidden inside "9 more". Low.
- tools/01-model: a door's height "6' 8"" beside a sill "0 ft" in the same row of fields -- two notations side by side. Low.
- tools/06: "Saved. What is being typed is not in it yet: Enter sets it." -- as claimed; fine.
- the rest show what they claim (01 the fields, 02 the door placed, 03 picked again showing 3' 6", 6' 8", 0 ft, 04 "Nothing was placed: the door's sizes are still open. Press Enter first, then click." in red, 05 a second door).

## trades (17 hand-taken pictures, no shot comments; 7 opened at full size, the other 10 read only on two reduced contact sheets: 2-framing-drawn, 3-electrical, 4-plumbing-now, 4-plumbing, 5-hvac, 6-interior-woodwork-drawn, 6-interior-woodwork, 7-waterproofing-drawn, 7-waterproofing, first-floor-alone; at that size their small words could not be read)
- trades/* (every one I could read): the old program, with "plumbline" in the title bar and "plumbline / Sheet" on the sheet, "FACETS", "Doors and windows 5 and 8" and "0 and 0", "Model (IFC)", "BIM model (IFC)", "Estimate (CSV)" under Write out (1-foundation-drawn), price $468,157.63 on most and $468,876.85 on some, so two prices for the same sample in one folder. Right panel "227 for the same reason. The first: The job, cost code:" cut off at the foot. Nothing marks these pictures as stale. Sure.
- trades/1-foundation-drawn: the foundation walls and footing are drawn as a plain box ring, as claimed; "Doors and windows 0 and 0". Fine apart from the stale words.
- trades/3-electrical-now: the house is see-through with only orange runs; there are no fixtures or panel symbols, so it reads as lines in a glass box. Low.
- trades/6-interior-woodwork-close: base at the foot of the wall and a panel door, as claimed. On the right, "Elevation 0 ft" and "Level height 10 ft" are given in decimal feet. Low.
- trades/7-waterproofing-window: claims flashing at the opening. It shows a dark grey band round the window on a pale blue-grey wall; nothing on screen names it flashing or weather barrier. Medium-low (same as layers/02).

## Ranked
1. dress/one-each/03 and 04: the camera sits inside the roof. 03 claims the north (shingles, stone) and east (lap siding) walls; 04 claims the south wall's Finish. Both show a full-screen grey roof, and 04 has "ROOF PLANE plane 1 ... Standing seam, charcoal" selected instead of the wall. The shingles, stone wainscot and lap siding of this job are never seen.
2. field/04, 06, 07: a measurement that was taken is announced in red, the colour every refusal uses ("Ground: level height is 8' 10 1/2", measured ..."), with "A second reading came straight after that one and was not used". Values the person did not touch turn Known: "found as drawn (checked on site)" on Elevation, duct Type and "Line on the plan (checked on site: JM)".
3. d4/02-07: the same job is "$24,685.91" of detail and "$22,033.70" of allowances on Budget, but "Job cost none / Price no figure / By division 0 of 16" on Estimate and "PRICE no figure" in the top bar.
4. d4/05: the money does not check against the figures on screen: 251.56 SF x $31.50 = $7,924.14, but $7,924.06 is shown; 419.26 x 31.50 = $13,206.69, but $13,206.78 is shown; 251.56 x $28.00 = $7,043.68, but $7,043.61 is shown.
5. d4/03-07, gablew/04: the selected plane faces away from the camera and is only a blue outline through the near planes. The gable made in gablew/04 is out of sight, and the roof looks as it did in 03.
6. d5/07-12: the roof is priced as copper standing seam but drawn dark grey, and the panel says "Also priced, not drawn: standing seam, copper". After "material set to Copper, 20 oz" the panel still lists "Standing seam copper, 16 oz".
7. d2/07-08 and d5: lines of the shipped sample and of the starter assembly have "no cost type", "no division" in red. "Dumpster" and "Permit" sit under "NO DIVISION / In no division", with General Requirements right above them.
8. itemnew/04-05: the message says "The one set down is called that too", but the open items still say "Wall oven, 27 in, position ..." and "Wall oven, 27 in, size ..." next to "Wall oven, single".
9. house/03: "one system picked out: supply air" looks the same in 3D as 02. The panel lists ends as "2, 17, 8.4", "38, 17, 8.4", "4, 14, 20": bare coordinates with no unit.
10. clash/04-07 sheet: places a person cannot use: "6 in round elbow, connection 1 (duct): E 0' 0", N 0' 0", H 3' 0" in" and "Air handler, connection 1: ... H 24' 0" in" on a 9 ft level. "6 in round elbow, what it is: 6 in round elbow".
11. Stale pictures kept in the captured sets with no label: glass/step1-6, layers/plan-panels, layers/estimate-window, light/*, trades/*. They show "plumbline", "FACETS", "job.json", "(0, 0) to (24, 0) ft", "5 and 8", and a price of $468,157.63 against the current $468,876.85.
12. bring/03-05 sheet, items/03: IFC words on screen: `"Ground", "buildingstorey" (as the model calls it) #23 in the file`, "wall #44 in the file", "Wrote job.ifc ... (IFC4, lengths in feet) ... Not in it: 2 x type". bring/03: "this program has no element for it".
13. pagew/01-02, bring/02: file-list folder names off the disk: "Back to tmp.PoWUBb1H4G", "Back to tmp". pagew/02 shows "usda-7213.pdf, 1"; first/03-05 show "(usda-7213.pdf, page 4)".
14. draw/10: "the stray wall selected" shows only the label "wall 6 45.00 SF" floating below the house, with no wall line visible.
15. Raw decimals where trades write feet-and-inches or two places: draw/12 "4.4167 ft"; d2/08 "283.9250 SF"; d5 "461.1890 LB", "251.5576 SF"; gablew "1.4311 SQ"; roofw "3.8025 SQ"; money "143.1084 SF" beside "143.11 SF" on the same screen.
16. roofw/05: the rate typed as "2 hr/SQ" is shown on the line as "at 0.02 hr/SF".
17. price/04-08, money/04: "prices vary" in the unit-cost column of every line that has no price.
18. dress/priced/03: claims "from above: the seams run up each plane". It is the same three-quarter view as 01 ("The whole job, in view.").
19. money/02-06: once the eave is measured at 9' 2", the roof sits on a dark band above walls still 9 ft, and nothing on screen says the walls and roof now disagree.
20. money/01-02 sheet: the Verify in field list has a WHERE column, and it is blank on all five rows.
21. itemlib/02-05, itemnew/03-05: the placed item is a 30-35 px box in the middle of an empty ground. The change from base cabinet to box (itemlib/03 to 04) cannot be seen.
22. build/02-03: the floor step changes nothing on the plan. Of the "two rooms", only "East room 341 SF" is labelled.
23. build/03-06, first/04-05: "whether wall 1, wall 3, wall 4 and wall 5 bound this room rests on their heights, which are assumed" is repeated up to six times on a screen and does not parse.
24. bring/05: "the estimate, kept beside the job" is said only in a message line. Neither window shows the prior estimate.
25. d2/01-08: Next steps reads "Next: Add a level", "1 of 10 done" on a built house with a roof. money reads "Next: Add doors and windows" while it is being priced, and roofw reads "Add a drawing and set its scale".
26. d2/03: one pitch change turns 69 Known values into Assumed (189 known/16 assumed becomes 120/88) with no word why.
27. first/02-06: the scale is shown as "1" = 7' 1"", while the sheet prints "1/8" = 1'-0"".
28. carry/02, clash/06: the thing is selected and drawn red, or crossed, but its panel says nothing about the clash. carry/04's message says "2' 0 11/16" east, 0' 1 7/8" north" where the step meant 1 1/2 ft east.
29. A minus sign in front of feet-and-inches: "N -0' 6"" (d2), "E -2' 1"" (draw), "E -2' 6"" (itemlib, itemnew), "N -0' 3"" (glass).
30. Counts that disagree side by side: "Verify in field 19" against "Open items 16" (bring, carry); "34 not priced" against "6 lines unpriced" (d2, gablew); "93 to verify, $17,060.62 riding" against "$11,479.13 ... rests on something assumed" (d4).

Could not read: the small words on the 10 trades pictures I saw only on reduced contact sheets. house/takeoff.txt was not opened (it is not a picture). Most sheet pictures that are pixel-identical to the one before them, or differ only in the foot line, were checked by pixel comparison rather than opened one by one.
