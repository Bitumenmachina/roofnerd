# Judgment 22B notes (the look and the words)

Program: bin18 (commit 17ab7a6). Jobs: JDIR/house.json (sixteen), JDIR/empty.json, JDIR/drw/traced.json + usda-7213.pdf.
Session 1 (o1): opening 3D, Plan (t), Walk (v), f in Walk (no change), back to 3D.

## Findings

J22B-1 (A4 fail) The Overhead and Profit "Set" buttons on the Job panel are square-cornered, drawn as a hard 1px blue hairline box. o1/01-model.png, crop crops/o1_right.png. Also the Next steps done marks are square filled green blocks and the open one a square hairline box (crops/o1_left.png).

J22B-2 (C2 / feel) The "sky" behind the house is a flat grey haze (RGB ~182,186,190 at top, ~195,192,185 low): no blue at all, it reads as fog or a grey studio backdrop, not a sky over a place (o1/01-model.png, crops/o1_sky.png). The ground is the same grey, with a faint tile grid. Nothing says outdoors. Fails "Google Earth feel", "light that reads as a place".

J22B-3 (A1 fail) Top bar runs the full width of the window (10px inset) and the status line runs full width at the bottom: they are strips along the edges; the model is seen only in a 10px gap. Weaker: the left and right panels run nearly the full height. The model occupies the middle ~980x740 of a 1600x960 window: panels cover ~45% of it. "The model fills the window" reads as not true; it reads as a docked application with a viewport.

J22B-4 (A2 fail as seen) Panels read near opaque: the left and right panels are a flat light grey; nothing behind them shows through in any picture (the house never passes behind one at this view, and the grey ground behind them is the same colour as the panel). Frosting cannot be seen. Not proven translucent from what a person sees.

J22B-5 (D8/E) Walk view (o1/03-model.png): the supply duct is a saturated flat blue plastic tube hung under the ceiling with a white disc; reads as a diagram, not sheet metal. Windows inside have no casing (known). Pressing f (Frame) in Walk did nothing and said nothing (o1/04 identical to 03) - A9 "nothing is met with nothing".

J22B-6 (E1 fail) Plan of the whole house (o1/02-model.png): the roof is drawn in solid orange/magenta/green lines over the plan, though it is overhead (E1: overhead in broken lines). Walls are light grey thin double lines, not ink; interior walls (8 per floor) are not visible at all under All levels; rooms not named on plan. It reads as a roof plan diagram, not a floor plan drawing.

Session 2 (o2): 01 Frame button; 02.. switches one at a time: Ground, Finishes, Framing, Roof, Architecture, Mechanical, Electrical, Plumbing, See through, Known Assumed Unknown, Clashes (o2/order.txt).

J22B-7 (A9 fail) No switch, no view button, says anything: the status line reads "Opened." through all twelve pictures of o2 (Frame, every Show switch). Frame (o2/01) changed nothing visible and said nothing. In Walk, pressing F does nothing and says nothing (o1/04).

J22B-8 (D7 / D8) Framing on (o2/04, crops/o2_frame.png): the stone foundation band disappears and studs run down to the ground; the foundation walls (concrete per tradesw) are not seen, so the frame stands on the grade with no sill on a foundation. No joists, no rafters: the roof floats over the top plates (known). Otherwise the frame reads well - the best picture in the run.

J22B-9 (A6 fail) Known, Assumed, Unknown on (o2/11, crops/o2_kau.png): the panel says "116 drawn from what is known, 81 drawn from what is assumed, 3 drawn from what is unknown", but from the opening view the whole house is plain white: no amber and no red anywhere a person can see (pixel count: the only amber is the hip lines). The hips stay their roof-line orange, the same family as Assumed amber, so the one amber on screen means something else. The Known dot in the legend is white on near-white (invisible). The top bar meanwhile says "3,213 known, 102 assumed, 255 unknown": two sets of counts for "how sure", unexplained.

J22B-10 (A5) Left panel: with Clashes on, the "SYSTEMS" heading is cut in half by the panel's lower edge (o2/12, crops/o2_cl_legend.png). In every opening picture the left panel ends on "Supply air 195.00 LF" with the rest of the systems hidden below, nothing but a thin scroll bar says there is more (8 systems "away").

J22B-11 (D8 / C3) Clashes on (o2/12): each clash is a small red or amber 4-point cross floating on the siding; 13 red + 8 amber; they read as sprite marks, not as the two things in the same space. Ducts with Architecture off (o2/06) and See through (o2/10) are flat saturated blue plastic tubes; equipment pale boxes (known D6 fail). Reads as a schematic.

J22B-12 (D7 pass-ish, look) Finishes off (o2/03): walls go plain pale grey; flashing at the openings not distinguishable at this distance. Ground off (o2/02): house stands on a grey slab-like footing over an empty grey void; the foundation reads as concrete. Pass.

Session 3 (o3): sheet tabs, top and scrolled to end (o3/order.txt).

J22B-13 (F3/F1 fail) Estimate tab, scrolled (o3/01-sheet, 02-sheet): the column headings are gone once scrolled (no heading stays), and figures on a line sit in different columns with nothing to say which: Div 5's $504.00 at the third column, Div 6 and 7 at the third, Div 3/4 subcontract at the far right; the foot reads "Column totals $203,798.75 $40,800.00 $150,078.73" with no column names in sight. A person cannot tell material from labor from subcontract by reading the row. Every line also carries a "Div 3" chip under a heading that already says DIVISION 3.

J22B-14 (A6 / F2 fail) "Open items 11" at the foot of Estimate and Budget (o3/03-sheet, o3/05-sheet): a column of eleven red and amber sentences, the A6 "column of red sentences". Wording is machine-made: "235 for the same reason. The first: The job, cost code: unknown"; "2 for the same reason. The first: Air handler, what it is: Air handler, vertical"; "39 for the same reason. The first: First-floor trunk, west, elbow 1, size: 1' 4" × 1' 4" × 0' 8"" (a duct is written 16x16x8, not in feet-inches); "Cable 0.4 in round, assumed, diameter: 0.4 in". Some start lower case ("hot to the kitchen sink, elbow 1 is not drawn"), some upper.

J22B-15 (A5 / F fail) Budget tab (o3/04-sheet): the grey second name beside each item is cut off mid-word with no ellipsis: "hall partitio", "partition a", "west windo", "east windo", "front windc", "back windc". Rates to four places: "$33.3066 / SF", "$10.6268 / SF", "$16.7436 / SF" - no estimator writes four places. The DIFFERENCE column is empty all the way down. Column heading "ELEMENT" is a software word.

J22B-16 (A5 fail) Clashes tab, scrolled to its end (o3/09-sheet): the line "walls and floors are not tested against each other, nor wall against wall, nor floor against floor; a footing is tested against neither, nor against" is cut off at the card's right edge mid-sentence. Same tab: "its size is not known: it is a line, not a solid" ("solid" is a CAD word).

J22B-17 (A6) Clashes tab (o3/08-sheet): the 13 findings (both Known, red per A6) carry no mark at all in the list; only the amber ones have a dot. In the model the same findings are red crosses. Colour and mark do not agree between the windows.

J22B-18 (D1 fail) Roof tab (o3/12-sheet): every plane's CONDITION reads "none yet", while the estimate prices Standing seam roofing on each plane at $14.00/SF (estimate.txt lines 262-271) and the model draws standing seam. The panel does not agree with the picture or the price.

J22B-19 (A4 / D8) The roof-plane labels on the 3D house ("4 252 SF 6:12") are square-cornered white boxes with a grey hairline (crops/o1_eave.png); the eave, hip and ridge are drawn as bright green, orange and magenta lines laid over the dressed roof in 3D. On the dressed view, they make the house read as a takeoff diagram, not a house.

J22B-20 (F words) Every tab ends on the same "Open items 11" card (o3/03,05,07,09,11-sheet), so each tab's end is the same eleven red/amber sentences. Building tab: "ROUND" for perimeter; "of them, Outside wall, 6 in". Mechanical: "Supply air  Supply Air" (name twice, case differs). Roof: "FACETS 4 roof planes".

J22B-21 (words, trades) Electrical and Plumbing tabs (o3/16, o3/18-sheet): cable is named by its diameter "Cable 0.4 in round" and the Verify in field list asks the man in the field to "Measure diameter." of a cable - an electrician names 12-2 NM, never a diameter. Pipe is "1/2 in round"; a hot-water fitting is named "not said". A receptacle and a ceiling register are filed as "Fixture".

Session 4 (o4): selected 01 south wall, 02 a window, 03 roof plane 1, 04 (missed a duct: "Nothing selected."), 05 tub 1 (Architecture off), 06 footing 4 (Ground off). The Clashes switch came up ON in this session though s2 left it off - a view switch seems to be kept in the job.

J22B-23 (A4 fail, every selection panel) The "Check" and "Measure" buttons on every selected thing's panel are square-cornered hairline boxes (o4/01-06, crops/o4_*_panel.png), as are "Set". They are the buttons the whole Verify loop rests on.

J22B-24 (D6 fail) tub 1 selected (o4/05, crops/o4_tub.png): the bathtub is a plain rectangular box (red, in the same space), not a tub, though VISUAL.md D6 says a bathtub is drawn as one. Lavatory and water closet read as rough white blocks at this distance. Air handler, water heater, heat pump: pale boxes.

J22B-25 (C4 / C5 fail) Footing 4 selected with Ground off (o4/06, crops/o4_footing.png): the ground's grid lines are drawn across the face of the concrete foundation wall (the ground plane shows through the wall); the footing itself has no outline or tint when selected - only the floating "footing 4" tag says something is selected. The footing reads as a pale slab barely darker than the void.

J22B-26 (words/units in panels) Window: "Center, along the wall 7.5 ft", "Sill 2.5 ft", "Height 5.00 ft" - decimal feet where every other length is feet-inches. Footing: "Depth 12 in" under AS DRAWN and "Depth 1.00 ft" under TAKEOFF; "Quantity 2.0741 CY" (four places). Roof plane: "rises from edge 1 of roof", "Outline 4 points, in feet", "solved from the footprint of roof" ("solved" a software word). Tub: "Connector 1 / Connector 2 ... where the plan shows the run arriving". Every panel: "Cost code ■ Unknown  not coded  unknown" says unknown three times in red.

J22B-27 (pass, C5) Wall and window selections draw a thin blue outline and a blue name tag, and the thing keeps its colour; status line says "Selected south wall." Pass.

Session 5 (o5): Plan; Wall tool; click with sizes open; Door, Roof, Duct tools; Next steps Hide; Add drawing; Estimate workbook.

J22B-28 (A9 pass) The refusal (o5/02): a frosted card at the top of the model with a red rule, "Nothing was placed: the wall's sizes are still open. Still to be typed: Thickness. Press Enter first, then click.", repeated in the status line. Says why and what to do. Pass. But the Wall tool opened with THICKNESS empty and HEIGHT 4 ft on the foundation level (o5/01): the first field a person meets is blank with no hint, while Roof and Duct fields show "like 8 ft" / "like Supply Air".

J22B-29 (A9 fail) Add drawing (o5/07): status says "Add drawing: pick the plan from the list, or drop the file on this window." and no list appears anywhere on screen (no drawings lie beside this job; nothing says so).

J22B-30 (A5 fail) Estimate workbook (o5/08): the status line is cut with an ellipsis where the window has room: "Wrote house estimate.xlsx, beside the job: the estimate, its amounts, recap and price as formulas. 3 sheets: Estimate, Recap, Divisions; 450 of its f..." The full sentence is never shown. It also names a file ("house estimate.xlsx") and says "sheets" in the spreadsheet sense.

J22B-31 (E1 pass, foundation) Plan of the foundation level alone (o5/01): walls are ink double lines with closed ends, footings a grey band under them, white paper. This plan reads as a drawing. Pass for this picture (contrast J22B-6, the whole house plan).

J22B-32 (state kept) Show switches and the level chosen carry over from one session to the next on the same job (Clashes came up on in s4; levels came up with only the foundation on in s5): the view is saved with the job without a person saving it ("Saved" in the top bar the whole time).

Session 6 (o6): the empty job. 01 as opened; 02 Add level; 03 Enter on an empty name; 04 Wall (refused: no level); 05-07 typing went into the level form, Esc.

J22B-33 (look, first sight) Empty job as opened (o6/01): a grey-beige haze over a faint grey grid, a grey-blue band at the very top: no horizon line, no sky, no sun, nothing to say what is up. It reads as an empty CAD viewport. The Job panel opens with two red "Unknown ... not set" rows (Overhead, Profit) - the first thing a new user meets on the right is red. The panel help ("The wheel closes on what is under the pointer") is pale grey 10px type.

J22B-34 (A9) Enter in the level form with an empty name (o6/03) does nothing and says nothing. Wall with no level (o6/04) is refused well: "Add a level to draw on first: its name, the elevation of its floor, and floor-to-floor." Pass for that one. In the level form, Enter moves to the next field (o6/06) though the form says "Enter adds it".

J22B-35 (A8 fail) The Add drawing list (o7/01-02) is a file picker in software words: "FILE, PAGE"; "IN THE JOB'S FOLDER"; "↑ Up, to j22b" (a folder's name on screen); the page is chosen by typing after a comma in the field ("Its page follows a comma: change the 1 for another page"); status "Usda-7213.pdf: Enter brings it in." (file name, capitalised wrongly). A8 says no folder path and no file name where the job's name belongs; this list is all file names and folders.

J22B-36 (A9 / A5 / words) Adding a level with Floor-to-floor typed "9" (the form's own hint is "like 9 ft") is refused (o8/02): "Can't add the level: "9" has no unit; say which it is: 25.5 ft, 306 in, or 25' 6". A number with no unit is never read as feet. A laser has to be set to send its unit with the reading" - the examples are not the number typed (25' 6" for a 9), the sentence ends without a stop, and the status line copy is cut with "...". A builder typing 9 for a floor-to-floor means 9 feet; the refusal reads as a lecture. Elevation "0" was taken without a unit, so the rule is not even the same for both fields. Add drawing then refused well: "Can't add the drawing: this job has no level for it to be a plan of. Add a level first." (o8/03).

J22B-37 (E plan, empty) Plan on a job with nothing in it (o8/04): white paper with a grey grid running under the panels to the window edges; fine.

J22B-38 (A2 / A3, measured) Panels do take a tint from what is behind (left panel 229,228,227 over the 3D ground; 238,241,244 over the plan paper; o1/01 vs o1/02) and the grid behind is blurred out completely (crops/o8_glass.png), so they are translucent and frosted in fact. But the frost is so heavy that they read as flat light-grey cards; and on plan no shadow can be found under any panel: the pixels beside the Job panel's edge and under it are pure white 255 (o8/04, crops/o8_edge.png). On plan the panels have no depth at all. The message card over the 3D view does cast a visible shadow (o8/03: 233 -> 171 under it). A2 "a soft shadow under it": fail on plan.

Session 8b (o8b): traced job; level added with units; Add drawing usda-7213.pdf page 4; plan; 3D; plan.

J22B-39 (E3 pass, with notes) The USDA plan page 4 laid under the plan (o8b/03, 05): flat, unlit, legible, the whole sheet with its title block and schedules. Pass. In 3D (o8b/04) the sheet lies on the ground washed to grey-on-grey, low contrast, larger than the grid, before any scale is set; it reads as a faded print on a floor, which is honest, though not inviting. The drawing is named by its file name "usda-7213, page 4" in the Drawings list, in the Job panel's Open items and in the status line ("Usda-7213, page 4: its page is read (nothing on it to snap to); 2 things not read from it: select the drawing to see what") - A8 "no file name where the job's name belongs" (it is the drawing's name here; a person would call it "Floor plan, sheet 1"). "snap" is a CAD word.

J22B-40 (A6) Job panel on the traced job (o8b/03): "OPEN ITEMS" again a column of red and amber sentences ("usda-7213, page 4, scale: none / not set: nothing can be traced over it"; "2 for the same reason. The first: First floor, elevation: 0 ft"; "usda-7213, page 4, corner on the plan: E 0' 0", N 0' 0"" broken over two lines after N 0'). Overhead and Profit still red "Unknown not set".

Session 9b (o9b): empty job; level "first floor", 0 ft, 9 ft added; Wall tool; Enter; click; "20" typed.

J22B-41 (A9 / first wall, serious) On a new job the Wall tool offers HEIGHT 9 ft and an EMPTY Thickness with no hint (o9b/02) - Roof and Duct fields carry "like 8 ft" hints, Thickness carries none. Pressing Enter to accept what is offered does nothing visible; the click on the plan is then refused ("Nothing was placed: the wall's sizes are still open. Still to be typed: Thickness."; o9b/04). The length the person then types ("20") lands in the Thickness field, and is refused again: "Can't use that thickness: "20" has no unit; say which it is: 25.5 ft, 306 in, or 25' 6". A number with no unit is never read as feet. A laser has to be set to send its unit with the reading" (o9b/05) - a thickness example of 25.5 ft. Three refusals before the first wall, none of them proposing a wall thickness. Status line copy cut with "..." (A5).

Session 11 (o11): house reopened; Level first floor; t; t; All levels; wheel x3 at 780,500; double-click at 780,500; For the PM.

J22B-42 (pass, the best screens) First floor alone in 3D (o11/02): looked down into, rooms with doors and windows, ducts and pipes under the cut; reads as a model of a house. First floor on plan (o11/03): ink walls with faces and ends, openings as gaps with blue window/orange door lines, rooms named with SF in labels - this reads as a drawing (E1 pass on this plan, except the runs: J22B-43). The level's panel (o11/02) is short and as tall as its content (A5 pass).

J22B-43 (E4 / E1) First floor plan (o11/03): runs are thin 1px coloured lines (orange, red dashed) indistinguishable in weight from the window marks; ducts that in 3D are 12x8 are drawn here as hairlines, not "at their own size". A dashed red run at the top (hot water) reads as a "line with no length" (E2's mark) though it is a pipe. Fail E4. Runs out past the walls into the panels' gap at left and right (grey stubs at x 430 and 1160-1210) - a grey box outside the house at right (the heat pump?) is unlabeled.

J22B-44 (B1 / B5, one run) Wheel x3 at the house's middle (o11/06 -> 07): the view went from the whole house to a close view of roof plane 1 filling the window, 3 notches; whatever was under the pointer stayed near the pointer (pass-ish). The double-click did not fly anywhere: it selected plane 1 (o11/07 = o11/08 view). A5: the right panel lets the house's colours through as streaks (crops/o11_7_right.png): here the frost is visible (A2 pass in this picture). On the close view, the roof label "3 453 SF 6:12" is cut by the top bar (o11/07 top: "3 453 SF 6 12" half hidden) - labels run under panels (A5 fail).

J22B-45 (A5) For the PM (o11/08): status "Wrote house for the PM.xlsx, beside the job: the lines, the budget and what is to verify, with no overhead, profit or price. 3 sheets: Lines, Budget,..." cut with an ellipsis; file name again.

Session 12 (o12): empty job, level with units, Wall, "6 in" typed, Enter, click, "20 ft", Enter, Esc, 3D.

J22B-46 (first wall, works when units are typed) With "6 in" typed into Thickness, Enter, a click, then "20 ft": the length field opens ("Toward the pointer, like 24 ft or 12' 6". Enter sets it."; o12/03) and the wall is placed: "Added wall 1, 20' 0" long." (o12/04). The first wall in 3D (o12/06) is a plain grey slab with a soft shadow on a grey ground - correct (nothing priced) but bleak.

J22B-47 (flag, never fabricate) After the first wall, Next steps ticks "Add a drawing and set its scale" as done (o12/04: "3 of 10 done", the drawing step green) though no drawing was ever added to this job. The checklist claims a step that did not happen.

J22B-48 (words) "Tool put down." (o12/06 status after Esc) [TRIP]; "On the angle" (o12/03 status bar right) [TRIP: means snapped to an angle]; "1 not priced" in red in the top bar for a single wall.

J22B-22 (F1 not in picture) No supplier side-by-side shows anywhere on the made-up whole house (no "supplier" in the estimate; no sheet tab for it). F1's last clause cannot be judged on this job; a person opening the sample never sees the three suppliers.

## VISUAL.md, line by line (verdicts)
A1 FAIL o1/01: top bar and status line are full-width strips; panels cover ~45% of the window (J22B-3)
A2 FAIL o8/04: tinted and frosted in fact (o11/07 shows it), but no shadow under any panel on plan; on 3D they read as flat grey cards (J22B-4, -38)
A3 PASS-ish o8/03, o4/02: message card shadows deeper than panels; tool in hand blue-washed and edged; selection blue
A4 FAIL o4/01 crop o4_1_panel: Check, Measure, Set are square hairline boxes; Next steps marks square; roof labels square (J22B-1, -19, -23)
A5 FAIL o3/04 (Budget names cut), o3/09 (Clashes note cut), o5/08, o11/08, o8/02 (status ellipses), o2/12 (SYSTEMS heading cut), o11/07 (label under top bar)
A6 FAIL o2/11 (KAU shows no amber/red on the house), o3/03 (Open items: column of red sentences), o3/08 (Known clashes unmarked in the list)
A7 PASS: one blue for selected, in hand, on; always with a frame or switch shape
A8 FAIL o7/01 (file picker: FILE, folder, file names), o3/04 (ELEMENT), o3/16 (cable by diameter), WORDS.md 69 marks
A9 FAIL o2/* (every switch and Frame: status still "Opened."), o5/07 (Add drawing with no list says nothing more), o6/03; refusals themselves PASS (o5/02, o6/04, o8/03)
B1 PASS-ish o11/06->07 (one run); B2-B4 not judged from stills (not in picture); B5 FAIL o11/07 (double-click selected, did not fly); B6 not in picture; B7 not judged; B8 PASS o1/01 (every layer, level, system has a switch; panels: only Next steps has Hide; the other panels have no switch -> FAIL for "every panel")
C1 PASS-ish o1/01 (one sun, soft-ish shadow, not black); C2 FAIL o1/01, o6/01 (grey haze, no blue sky, no horizon; reads as fog) (J22B-2, -33); C3 PASS-ish brick/stone/siding/standing seam read; ducts plastic (J22B-5); C4 FAIL o4/06 (ground grid drawn through the foundation wall face) (J22B-25); C5 PASS o4/02, FAIL o4/06 (footing selected shows no outline)
D1 FAIL o3/12 (Roof tab "none yet" while standing seam is priced and drawn) (J22B-18); D2 PASS o1/01; D3 PASS o1/01; D4 PASS-ish o4/02 (frame, sashes, lites, casing, sill; door with frame and leaf); D5 PASS-ish o1/01 (fascia, soffit; gutters priced, a thin line only); D6 FAIL o4/05 (bathtub a box) (J22B-24); D7 FAIL o2/04 (framing stands on grade with no foundation under it; no joists/rafters, known); D8 FAIL o2/12, o4/04 (diagram of boxes and coloured lines over the house)
E1 FAIL o1/02 (whole-house plan: roof in solid colours over the plan, interior walls missing) PASS o11/03, o5/01; E2 PASS-ish o1/02; E3 PASS o8b/05; E4 FAIL o11/03 (runs are hairlines) (J22B-43)
F1 FAIL-in-part o3/01: recap, sixteen divisions, lines under headings PASS; suppliers side by side not in picture on the sample (J22B-22); F2 FAIL o3/03 (Open items wording; "none"/"no figure" used right in the by-division list); F3 FAIL o3/01-02 (scrolled rows lose their column heads; figures in unnamed columns) (J22B-13); F4 PASS o1/01-sheet (cards, same type, same blue)
