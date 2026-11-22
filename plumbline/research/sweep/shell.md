# The shell: a concrete and foundation contractor, and a framing carpenter

Method: 25 pictures read (listed in each section), the frozen command line (made-up whole house `shell/h.json`;
a made-up 24 x 16 house with a 12 x 16 addition `shell/a.json`), DECISIONS.md calls 164, 165, 160, divisions/README.md,
the earlier research files. Window not run. Beams, posts, pads, joists and rafters are NOT in the frozen copy (no `beam`,
`structure` verb), so those I review as written in calls 164 and 165 only ("not seen").

Pictures read: trades/1-foundation, 2-framing, first-floor-alone, 7-waterproofing; foundw/01, 03, 05, 08 (model), 08 (sheet);
roofw/03, 05; renow/03, 06, 10; house/03, 05; layers/03, 05; glass/04; d2/04; clash/04; scanw/05; plan/05; field/04;
price/04; also read: trades/ (the rest by name only, not opened).

---------------------------------------------------------------------------
## A. The concrete and foundation contractor

### 1. Who I am and why I open this
1. Where the footing runs, how wide, how deep, what it sits under; the stem wall on it, its height and thickness.
2. Yards of concrete and pounds of bar, by the pour (footing, then wall, then slab), and forms in SF.
3. On an addition: where my new footing meets the old one, how deep the old one is, where I dowel, and what I must
   excavate or underpin. Will my new footing sit above or below the old bottom?
4. Steps: where grade changes, where the footing steps, how many steps, what each step's run and rise are.
5. What is in my way: a pipe or duct through my footing or stem (sleeves), anchor-bolt and hold-down locations.

### 2. What I saw
- `trades/1-foundation.png`: with only Architecture on, a clear grey ring, wall inside, a thin plinth around it (the footing
  sticking out past the wall). Readable as a foundation in five seconds. But: no number anywhere on the model, no
  footing width, no wall height, no depth below grade, no ground line (Ground switch off), no label. The right panel is the
  whole job ($468,876.85, sixteen divisions): nothing is about my trade until I click something. "See through" is off.
  I cannot find rebar, anchor bolts, sleeves, a slab edge, or the slab: the first floor's slab is absent when the first
  floor is off.
- `foundw/05-model.png` (Ground off): a clean concrete box, wall and footing in two greys. Left panel says "foundation -4' 0"".
  Right panel: Walls 4, 136 LF, 544 SF face, Footings 4, 136 LF, Footing concrete 7 CY. Good, these are my numbers.
- `foundw/08-model.png` (plan, footing selected): footing 1 drawn as double lines with a blue select; panel: type "footing
  20 x 10", depth 10 in, plan length 40.00 LF, volume 2.06 CY ("plan length x width x depth / 27"), width 1' 8". Each with
  "made-up foundation plan", Assumed, and Check/Measure buttons. This is good: every number says where from.
  Wall 1 (`foundw/08`... wall picture): net face 160 SF, 4 ft high, 8 in, "Foundation wall, poured concrete" Assumed.
- `foundw/08-sheet.png`: "Nothing is priced on 6 things that could be (3 footings, 3 walls)", footing lines "Trench for the
  footing", "Footing forms, both sides", "Footing concrete" each "no price entered". Honest, but it is a flag list, not a pour list.
- `house/03` and `layers/03`: the foundation level is simply not in sight with the house; in `layers/05` (first floor alone)
  the stone-faced base stands on nothing: no grade line, no footing under it (foundation off).
- `renow/03,06,10` (renovation): existing wall in white, to-demolish in red, brick on the new side. The renovation vocabulary
  (Existing / To demolish / New work) is right and fast. But it is walls only: nothing shows the old footing under an old wall or my
  new footing meeting it.
- `plan/05`: tracing a USDA plan over the PDF works; that PDF shows a "FOUNDATION PLAN" sheet lower on the page but only one
  plan is traced at a time.
- Words not my trade's: "Known, Assumed, Unknown" (fine to an estimator, not a mason), "Fitting", "Conduit", "Cable" in the
  toolbar I never use, "Wall" for what I call a stem or foundation wall (the type name carries it). "Plan length" for LF is fine.
  Footing "type footing 20 x 10" says width x depth reversed from how I write it (width first = 20 in wide x 10 in deep, OK).

### 3. What is missing or wrong for me (one line each)
- Looked for: the footing's size and the wall's height written on the 3D model; got: nothing but the grey shape. Matters: I cannot
  check it against the structural sheet without clicking each piece.
- Looked for: a ground line / grade and the depth of the bottom below it (frost depth); got: "the model has no grade"
  (divisions README). A footing bottom must be at or below frost depth for the code; excavation CY cannot be computed; backfill, grading and
  a stepped footing are impossible. Failed inspection if the depth is wrong.
- Looked for: stepped footings; got: "Not yet". On any sloping lot every footing steps; each step is forms, bars, a pour.
- Looked for: rebar drawn and counted in the 3D (continuous bars, vertical dowels, corner bars): the sample `sixteen` house prices
  footing and wall as one line each ("Concrete footing, in place" at $450.00/CY, "Foundation wall ... formed and placed" at $38.00/SF),
  no bars, anchor bolts or dampproofing in the sample; only the starter library has them, and bars are not drawn.
  Matters: bar is 10 to 15 percent of the pour price and the inspector counts it.
- Looked for: the new footing tied to the old (dowels, bottom-of-old-footing depth): got: duplicate footing counted. In `a.json` I drew the
  addition's wall and footing on the old house's east line: the program took both, footing concrete 6.09 CY, 120 LF, said nothing about
  two footings and two walls on the same line ("a footing is tested against neither [wall nor footing]"; "walls ... not tested against each other").
  Already noted in trades-use-gc-structure: takeoff counts existing and new alike. For me it is double concrete ordered.
- Looked for: a slab in the foundation picture; got: the slab belongs to the first floor and disappears with it; thickened edge, slab on grade
  vapor barrier and gravel in the sample is a flat $6.50/SF, the "by thickness" assembly is a newer one.
- Looked for: a pour schedule (footings 7 CY, walls, slab, in what trucks, what min load); got: per-footing quantities
  (2.9630 CY and 2.0741 CY on the 40 ft and 28 ft sides, which are 40 x 24in x 12in / 27 = 2.963 and 28 x ... = 2.074, hand checked right).
  The takeoff shows footing concrete 10.07 CY for 136 LF at 24 x 12 in: 136 x 2 x 1 / 27 = 10.07, right. But the corner overlap is not taken
  off in the sample as it says it is for T and cross; at a corner footing 1 and footing 2 overlap (a 2 ft x 2 ft x 1 ft square). The README says the
  overlap on a T or cross is taken off; for a plain L corner here the four totals add exactly to 10.07 as if full length. I did not check the
  four corners separately; 2.963+2.074+2.963+2.074 = 10.074, i.e. NO overlap removal at the corners of a closed ring (about 0.30 CY over, 3 percent).
  (Suppose, not proven: one of the two edges at each corner should lose width/2.)
- Looked for: sleeves through the footing or stem for the sewer, water service and conduit: clashes say "a footing is tested against neither [runs]"
  nothing is tested against a footing; `clashes` on the whole house shows the water service and waste stack going through "floor [floor-0009]" at z -0.17
  ("a hole to cut") but the stem wall and footing are not in the test. A 4 in drain through a footing is a sleeve I must set before the pour.
- Looked for: the foundation wall's height vs the framing above: the foundation wall is "4 ft" and first floor at 0, foundation level at -4;
  nothing shows the sill plate, anchor bolts at 6 ft spacing, or the 7 in max edge distance; the starter library has anchor bolts "by spacing".
- Looked for: formed vs. poured against the earth, wall thickness change (a brick ledge), a pilaster, a corner, a beam pocket: not present.
- Looked for: the foundation tied in sheets: a foundation plan sheet printed with footing marks and dimensions (call 168 prints plan sets, not seen).

### 4. What the best tools show today
- Concrete takeoff tools (Bluebeam, PlanSwift, STACK) are 2D area/length tools. Not checked here from the makers' pages; I did not fetch them.
- Chief Architect generates foundation and framing from the model, with materials lists by Cut/Buy list:
  https://www.chiefarchitect.com/support/article/KB-00095/creating-a-cut-list-of-framing-materials.pdf and
  https://www.chiefarchitect.com/support/category/19/framing.html (their site says: "Build > Framing > Build Framing"; reports
  as Mixed, Total Lineal Length, Cut List, Buy List).
- Connector and anchor layout matters to both trades: JLC says hold-down hardware in the wall has to line up with the anchor bolts in the concrete
  and warns the bolts end up in the wrong place from layout mistakes
  (https://www.jlconline.com/how-to/foundations/tying-down-the-house_o?o=3). Simpson publishes anchorage tables matching holdowns
  (https://seblog.strongtie.com/2015/05/holdown-anchorage-solutions/).
- Where this could be ahead: one model holds both the foundation and the wall above it: it could print the anchor-bolt and hold-down layout once,
  from the framing's king/jack positions, so bolt and hold-down agree (none of the above does this in a free tool, to my knowledge; not checked).
  And the old/new tie: no estimating tool I know of draws the old footing from a field depth and lists the dowels and the dig to its bottom.

### 5. My ten (concrete)
1. **Write the number on the thing.** With "Dimensions" on, footing and wall in 3D wear tags: "footing 20 x 10, 40 LF, 2.06 CY"; wall "8 in x 4 ft, 160 SF".
   Done when each foundation piece carries its tag, Assumed in amber like the roof planes in `roofw/03` (which already do this).
2. **A grade line.** A level has a "grade" (finished ground) height, Assumed from the drawing or measured; the 3D draws it as a thin ground edge cut through the wall;
   panel gives "below grade 3 ft 4 in, frost depth 36 in: footing bottom is X in below, OK/short". Excavation CY and backfill CY from it.
3. **Stepped footings.** A footing may have steps: each with its run and rise; panel lists the steps and the forms/concrete for each; code limit (rise <= ... ) shown as a flag only.
4. **New meets old.** Mark an old footing existing with a typed/measured depth (VIF). Drawing a new footing that touches it lists "dowel to the old footing: n bars, #4,
   18 in o.c." and "dig to old bottom: x CY" as lines Assumed, and the VIF list asks "old footing depth".
5. **Duplicate on the same line.** `check` or the takeoff says "footing 5 lies on footing 1 for 16 LF: counted twice" and, for an existing footing, the takeoff page separates
   Existing from New (the takeoff already mis-adds them).
6. **Rebar in the model.** Bars drawn as thin lines inside the footing and wall in See through (continuous, corner, dowel, vertical at spacing), counted in LB with lap and cover as Assumed rules.
7. **Sleeves and penetrations.** A pipe or duct crossing a footing or foundation wall shows up as a clash type "sleeve to set before the pour" with its location and size, in the VIF list.
8. **Pour schedule.** One report: footings CY, stem walls CY, slab CY, each with a pour order, forms SF, bar LB, plus 5 percent waste shown. Truck count only as a note.
9. **Anchor bolts and hold-downs from the framing.** Draw bolts at spacing, with a bolt at each hold-down position taken from the framing's end studs (JLC point above).
10. **Foundation plan sheet** (call 168 style): footing marks F1, F2 and a footing schedule, with a dimension string.
Each is done when a person can click the foundation, read the figure, and see it agree with the sheet.

### 6. What must not change (concrete)
- Every value says Assumed / Known and where it came from (the Check/Measure buttons in `foundw/08`).
- The "Nothing is priced on 6 things ... each marked on its line" honest total; the footing CY worked from "plan length x width x depth / 27" shown under the number.
- The Ground switch and the clean see-through foundation ring; the Existing / To demolish / New work buttons.

---------------------------------------------------------------------------
## B. The framing carpenter

### 1. Who I am and why I open this
1. The frame laid out: studs at 16 in, plates, kings, jacks, headers, cripples, sills, corners, so I can see that it works and order the lumber.
2. Lumber by the piece and length (studs by their length, plates, headers by size), not by the SF; sheathing sheets; a cut/buy list.
3. What bears on what: where a header goes, where the beam under the wall above is, what goes if a wall comes out; temporary shoring.
4. On an addition: the new floor level against the old, the ledger/tie, the roof tie-in (valley, flashing), new joists beside old ones.
5. What my pipe-and-duct neighbors cut: a hole through a plate or a stud, a joist notch.

### 2. What I saw
- `trades/2-framing.png` (Framing on, Finishes off): a real stud frame in 3D, two stories plus a hip roof on top: studs, plates, headers over the
  windows, sills. Readable as a house frame in five seconds. This is the strongest picture for my trade. But: the roof is a solid slab (no rafters),
  the floors are not visible, and no number anywhere on it: no stud count, no stud length, no spacing, no header size. The `layers/03`
  picture is the same with Mechanical etc. on, which is noise for me; in `layers/05` Framing is switched off and the walls are solid.
- `house/05` (first floor, trades on): ducts and pipes through the rooms: I can see where duct and plumbing might hit my wall.
  The right panel's "Roof" block lists eave 144 LF, hip 90 LF, ridge 12 LF, surface 1,408.72 SF, 14.09 SQ, 6:12, four planes. No rafter, no bird's mouth, no ceiling joist, no heel.
- `roofw/03,05`: roof plane 1 selected: surface 380.25 SF, 5:12, plan dimension labels, corners with east/north/height, "Eave out 6 in", "Gable". Good for roofers.
  For me: no rafter count/length (call 165 is for this, not in my copy), no stretch-out, no seat cut, no collar ties, no ridge board size.
- `renow/03,10`: wall 5 selected "To demolish", wall 144 SF; the room grows from 178.25 to 364.25 SF ("Living is now 364.25 SF (was 178.25)"). Good. But no
  "is it bearing?" question on the panel and no header/beam, no shoring on the screen (call 160 adds the asked-in-the-field part, in the VIF text; I did not see it on a picture).
- `d2/04`: roof eave moved out 6 in: "The roof is now 6 planes; ... With an edge moved here it is Assumed". Good wording.
- `field/04`: level height "8' 10 1/2", measured (it was 9 ft, Assumed)". This is precisely the framer's stud-length question: a stud is cut from this. Good.
- `price/04`: wall 1 selected: "Framed in 2x6 at 16 in on center", "A weather barrier under its finish", net 180 SF with openings 36 SF. Good. Wall framing price is by the SF.
- Words not mine: "Wall face, less openings", "Assembly", "Cost code", "Allowance, $/SF" are estimator words, fine; "Fixture", "Fitting", "Conduit" are other trades'; "Floor" tool places a slab/floor type without saying joists.

Command line, `h.json` south wall (40 ft, 10 ft high, 2 openings + door): `studs 51 EA` (note: "every stud, king, jack and cripple ... a corner's and an intersection's extra studs not counted"),
`plate_length 120 LF` (3 x 40: one bottom + two top, a 40 ft wall needs 5 pieces of 8 ft x ... shown as LF only), `header_length 13.75 LF`, `sill_length 10 LF`.
The price line is "Wall framing, 2x6 at 16 in, 400 SF at $4.20/SF". The stud, plate and header figures exist but the sample prices none of them: the price is by SF.
`header_length 13.75`: I count a door (3 ft) + two windows (5 ft)? it printed three openings in one wall but I could not reproduce 13.75 by hand: 3+1.5+1.5 plus 2 x (5+3)... = 22 ft. I did not resolve it, so I flag it "not checked: may be the sample's windows are not all in this wall; may be headers share" and say so (the sample wall has door 3 ft at 20 ft and windows at 7.5 and 32.5 ft per `show`).
Hmm: 3 + 5 + 5 = 13 plus jack to jack extras 0.75: so the header is "jack to jack" at about the opening width + 0.25 each: that doesn't include the 2 jacks (3 in) on each side. A real header is bearing on the jacks: opening + 3 in. 13.75 = 13 + 0.75 = 3 x 0.25. So the header is the opening plus 3 in, which is jack to jack as the note says. Plausible. Resolved: right.

### 3. What is missing or wrong for me
- Looked for: lumber by the piece and length: got: framing priced by SF face; figures studs/plates/headers exist but a stud's LENGTH, the stock length (92 5/8 precut, 104 5/8, 8/10/12 ft)
  and a count by length are not given. Matters: I order by length; a 10 ft wall with 9 ft studs is a callback or a waste.
- Looked for: corner and intersection studs counted: not counted (said so). Undercount of about 3 studs per corner and 2 per T: for a house 8 corners + 6 Ts = about 36 studs, a real order short.
- Looked for: headers sized: got a rule of thumb (2x6 to 4 ft ... 2x12 beyond) used for drawing only; 3D shows a 2x6 flat header over a 3 ft door: any carpenter or inspector will argue.
  The program says "nothing is sized" for beams but sizes headers by rule: two rules for the same thing. Flag: a header over an exterior bearing opening comes from the tables/engineer (IRC R602.7); the drawing should show it Unknown until typed.
- Looked for: floor joists and rafters on the screen: not in my copy. Call 165 counts them only where a priced line names the framing (the starter "Framed floor" gains it) and I could not see it; the frozen "Framed floor" is by SF only (`a.json`: "Floor framing 192 SF"), thickness 12 in carries no joist size, spacing or direction. Floor-to-floor: a 10 ft story with a 12 in floor does not show on the 3D as joists in `trades/2-framing`.
  What I suppose about call 165 (not seen): "joists span the short way, Assumed" is a sound default, rim figures present; no drawing of members is promised there (figures only: "not things of their own"). A carpenter wants to SEE the joists, the joist direction arrow, the doubled joist under a partition, blocking and hangers ("not counted").
- Looked for: what bears on what: nothing; there is no bearing flag on a wall (call 160 asks it in the field only for an opened wall). Matters: a wall taken out without a beam is the most expensive mistake on a remodel. 3D gives no load path: top plate, joists above, the wall below.
- Looked for: a wall taken out and its header/beam/shoring priced: renow shows the room growing but no header or shoring (trades-use-gc-structure says so too); call 164 gives the beam/post/pad as typed members, "tools to draw them in the window are the next work": so today in the window I cannot place a beam.
- Looked for: an addition's floor against the old, a ledger, the sill/band tie: not present. Roof tie-in (valley, saddle, flashing, rafter against the old roof): the roof is hip-from-footprint by edges; a shed roof tied to the old wall is not drawable except as a plane (not seen).
- Looked for: framing clashes: `clashes` lists every duct and pipe through a wall as "a hole to cut" (71 findings in the sample), with no word on whether the hole hits a stud, a plate or a header. Because studs are not placed as members in the clash test, I cannot tell a 3 in hole between studs from one through a king stud. Also the floors: "floor [floor-0044] and refrigerant line set" through at x,y: no joist, no notch/bore rule (1/3 depth, 2 in from the edge), no I-joist web hole.
  The framing exists as drawn objects in 3D, so the info is nearly there.
- Looked for: framing on the VIF list: a rough opening size: Door 3 ft x 6.67 ft: the "rough opening" (3 ft 2 in x 6 ft 10 in or the like) is not stated; the opening in the frame is jack to jack at the nominal size (I believe: the model drew header to span width + 3 in; I did not see a rough-opening number on screen).
- Looked for: a cut list: no. The takeoff prints the counts, no list "42 x 2x6x10 studs, 12 x 2x6x16 plates".
- Looked for: sheathing in sheets and the layout of the first sheet: `Sheathing and weather barrier 329.99 SF at 2.10/SF`: SF, no sheets (a "to order" figure exists for subfloor: "6 sheet").
- Looked for: the walls' height from a field measurement flowing to stud length: the level height measured to 8' 10 1/2" changes the level, and I assume the wall: not checked whether the wall's `studs` change.

### 4. What the best tools show today
- Chief Architect: builds wall, floor, ceiling and roof framing, headers sized from an Openings panel, and reports lumber as Mixed / Total Lineal Length / Cut List / Buy List
  (https://www.chiefarchitect.com/support/category/19/framing.html, https://www.chiefarchitect.com/support/article/KB-00095/creating-a-cut-list-of-framing-materials.pdf).
  Their own support text says compare with a manual takeoff before ordering; a forum user (not official) says it gives no joist/rafter span calculations.
- Home Designer (same maker) has the same cut/buy list (https://www.homedesignersoftware.com/support/article/KB-00182/creating-a-framing-cut-buy-list.pdf).
- Layout: JLC advises the layout person to mark key members (trimmers, kings, bearing posts) before framing and warns hold-downs and anchor bolts end up out of line
  (https://www.jlconline.com/how-to/foundations/tying-down-the-house_o?o=3).
- Not checked from makers' pages: Revit/Archicad framing, Metwood/StrucSoft, Weyerhaeuser Javelin, the truss makers' software: I did not read their pages.
- Where this could be ahead: it already calls what is Assumed/Known and lists the field questions; none of the framing tools above (as far as I read) mark a header, a joist direction or a stud length as "assumed: confirm in the field" with the dollars riding. A framing cut list where each line says "Assumed: the wall height, measure it" and is on the VIF list, and a "wall to remove: bearing? what does it hold up?" that walks up through the floors above would be ahead of them.

### 5. My ten (framing)
1. **A cut/buy list from the frame.** One report: studs by length, plates (pieces by stock length), headers by size and length, sills, cripples, kings/jacks, blocking none -> flagged: "not counted". Each line shows Assumed/Known and the wall height behind it. Done: the list's total length equals the model's, and changes when the level height is measured.
2. **Count the corners and Ts.** Corner posts (3-stud or 2-stud with backing), intersection backing; the note "not counted" goes away. Done when a 24 x 16 box shows 4 corners x n studs, hand-checked.
3. **Stud length on the line.** `stud length` and stock length on the wall's framing figures, from the level height less the plates; the wall height label in 3D.
4. **Header sizes Unknown until typed** (or from a typed table the user brings in), drawn as a dashed/amber box in the frame; list "header over the west window: size to confirm" on the VIF list. Remove the thumb rule from the drawing. Rough openings named and sized (jack to jack plus clearance) on each door/window panel.
5. **Bearing on a wall** as a value: Bearing / Not bearing / Not known, Assumed; "wall taken out" opens a question "what does it carry? n joists above, the wall above on level 2" with a highlight in 3D of what sits above (walls, joists, point loads) and a typed beam/posts/shoring set (call 164 has the members; add the window tools to draw them, now owed).
6. **Draw joists and rafters** in 3D under Framing (call 165 counts them; the picture must show them: direction arrow, spacing label, rim, doubled joist under a partition, header joists at a stair opening). Stretch-out and seat cut on rafters. Done when `trades/2-framing` shows floors and a roof frame.
7. **Framing-aware clashes.** A pipe/duct through a wall says "through plate / through stud / between studs"; through a floor "through joist: notch or bore; or between joists" (bore/notch limit as a typed rule). Sorted by dollars as today.
8. **Addition tie-in.** Marking a wall existing and drawing a new wall against it yields "ledger / tie / flash" lines Assumed, floor-height match as a VIF: "old floor at 8' 11 1/2, new at 9' 0: confirm". Roof: valley/saddle lines and flashing LF where a new plane meets an existing one.
9. **Sheets and nails by the piece.** Sheathing in sheets (4 x 8) with the waste shown, to order; hangers/straps/hold-downs listed from the frame (JLC/Simpson), each Assumed.
10. **A framing-only view.** One switch "Framing trade" turns off all finishes and other trades, turns See through on, and labels the members: stud spacing, header sizes, lumber, with a legend and a hide-the-roof-planes toggle so a person sees the floor frame. Today I need 7 switches (`trades/2-framing`: Ground, Framing, Architecture, Roof, Mechanical, Electrical, Plumbing are all on in that picture).

### 6. What must not change (framing)
- The frame is real: kings, jacks, headers, cripples, sills and plates at 16 in, in 3D (`trades/2-framing.png`).
- Wall framing is a line named with its size and spacing ("Framed in 2x6 at 16 in on center", `price/04`); the formula visible and editable on the line.
- Lumber sizes are the American Softwood Lumber Standard's; the layout rule is stated and its source named; "a corner's and an intersection's extra studs not counted" is said out loud. Keep saying what a figure leaves out.
- Measured heights become Known and move the number (`field/04`); eave and edge moves turn a Known into Assumed with a plain sentence (`d2/04`).
- Existing / To demolish / New work as three buttons on a wall, and the room figure changing as the wall comes out (`renow/10`).

---------------------------------------------------------------------------
## Shared things I could not settle
- Not seen: any picture of footing steps, beams, posts, pad footings, joists, rafters; any picture of Dimensions on the 3D; the window tools for a beam/post.
- Frozen copy lacks `structure`, `beam`, `post`, `pad` verbs; call 164/165 reviewed as written only.
- The sixteen sample has a 4 ft foundation wall 8 in thick drawn in `trades/1-foundation`; I did not test depth below grade because the model has no grade.
