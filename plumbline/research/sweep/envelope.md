# The envelope: four people open the outside of the building

Written from pictures and the command line, not the live window. "Not seen" means a thing may exist but no
picture shows it. Made-up numbers only (the program's own made-up whole house). I changed nothing in the
repository. Command-line work is in `/tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/review/sweep/envelope/`
(`h.json` = `sixteen`, plus `estimate.txt`, `takeoff.txt`, `finishes.txt`).

**Pictures read (29):** `trades/` 7-waterproofing, 2-framing, 1-foundation, first-floor-alone; `roofw/` 01, 03, 05;
`house/` 01, 05; `gablew/01`; `shinglew/01`; `renow/` 04, 08; `foundw/05`; `layers/` 01, 03; `dress/one-each/` 01, 03, 04;
`dress/priced/02`; `light/` after-street, after-window; `glass/06`; `scanw/05`; `d2/` 04, 06, 08; `clash/03`.
Commands run: `sixteen`, `estimate`, `takeoff`, `finishes`, `check`, `clashes`, `vif`, `show`, `lines`, `drawings`.
Read also: `library/sixteen-divisions.csv` (the starter assemblies), `crates/app/src/scene.rs` where the pictures leave a
question (what is drawn for the eye and what is priced), and `research/trades-use-envelope-finishes.md` so I do not repeat it.

Pictures that are older than the code are a risk: `layers/`, `house/`, `light/` still show "plumbline" or an older panel
in places (`light/after-*` says "plumbline"), so I read them as the look, not as proof of what the current build draws.

---

## 1. Who I am and why I open this

**A roofer (steep and low slope; a new roof cut into an old one).** I come to see the roof as a set of planes I can
walk: pitch on each, where the ridge, hips and valleys fall, every eave and rake in feet, where the new roof dies into
the old wall or the old roof (headwall, step flashing, valley, cricket), and what pokes through (vent stack, flue,
skylight, curb). I want squares, not square feet, and I want to know what comes off (tear-off layers) and what stays.

**A siding and exterior trim contractor.** I come to see wall area by finish (lap, shake, panel, brick, stone) with the
openings out, the corners counted in feet, the trim by the piece (corner boards, casing, frieze, rake, fascia, soffit),
the gable triangles, and what goes under it (housewrap, tape). I want to see my wall dressed as I will build it and know
the dollar on each face.

**A window and door installer.** I come for the schedule: every opening's mark, unit size, rough opening, sill height,
head height, type, swing, glass (egress, tempered), where each sits on its wall, and whether heads line up. I want to see
that the opening has a pan and a head flashing, and that the thing priced is the thing drawn.

**A waterproofing and flashing man.** I come to follow water from the roof edge to the footing: the weather barrier on
each wall, flashing at every opening and every roof-to-wall line, penetrations (pipe, duct, line set, vent) and what
seals each one, kick-out at the end of every step flashing run, and the ledge flashing over a brick or stone course.

---

## 2. What I saw

### 2a. As a roofer

- **`roofw/01` and `d2/04` (roof planes, See through off).** In five seconds I read: a hip roof on a box, four planes,
  each with a label in SF and pitch ("1 380 SF 5:12", "4 183 SF 5:12"), lengths on the edges (14' 0", 19' 2", 40' 0").
  The eaves are green, hips orange, ridge violet. I can click a plane and its panel gives surface, pitch, corners with
  heights, "Eave line, Assumed", and a row of tools (Pitch, Steeper, Flatter, Move eave, Eave out 6 in, Eave in 6 in,
  Gable, Whole roof). That is a roofer's vocabulary and it is good. I can set Existing / To demolish / New work on a
  plane (`d2/04` shows the WORK row), which is exactly what a tie-in job needs.
- **`d2/06` and `d2/08` (L-shaped roof, plan).** Six planes, valleys drawn, the valley length printed (15' 4 13/16"),
  and the plane panel shows the formula: plan area, slope factor 1.25 = √(1+(pitch÷12)²), surface, squares. A roofer can
  check that by hand. I like that. In `d2/08` the roof is drawn as brown standing-seam ribs on the plan; the plan shows
  pitch per plane and the ridge in violet.
- **`roofw/03` and `roofw/05`.** Shingles on the roof (a convincing granule pattern), "Asphalt shingles, from the line
  'Asphalt shingles'", and the status line tells me the production rate was set on plane 1 and also taken by planes 2 to 4.
  The price reads "no figure" until every line has a price: honest.
- **`dress/one-each/01`, `03`, `04`, `layers/03`, `house/01`.** The hip roof is standing seam with seams at a regular
  spacing, a white fascia board all round, and the eave is plain. The sixteen-division house is in standing seam at
  $14.00/SF; the other pictures are shingle. The metal looks right; the hip cap line is drawn as a ridge in the
  `Roof` takeoff colours.
- **`house/01` (the Job panel, Roof section).** SURFACE 1,408.72 SF, SQUARES 14.09 SQ, PREDOMINANT PITCH 6:12, ROOF
  PLANES 4, Eave 144 LF, Hip 90 LF, Ridge 12 LF. That is the roofer's summary block, in the roofer's units. Missing from
  it: Rake, Valley, Headwall (they show only when they exist, which is right) and anything about penetrations or
  tear-off. Not seen on any picture: a rake, a valley on a real tie-in, or a headwall drawn on a model.
- **`d2/04` message bar.** "Eave moved out 6 in on edge 1 of house. The roof is now 6 planes; its eaves, hips, valleys
  and ridges follow. The roof's footprint was Known (sample plan A-201). With an edge moved here it is Assumed...". This
  is the best thing I saw for a roofer: move an eave and the whole hip-and-valley layout recomputes, and the program
  says what it did to the certainty.
- **`trades/7-waterproofing`, `first-floor-alone`.** Not roof. But `trades/2-framing` shows the roof as a dark-grey cap
  over bare studs: no rafters, no trusses, no sheathing. A roofer cannot see the deck he nails to.
- **What I cannot find on any picture:** a vent stack or flue where it goes through the roof; a skylight, curb or
  chimney; a cricket; kick-out flashing; a second roof meeting an existing wall; tear-off layers; snow guards, a ridge
  vent where it sits. `clashes` says it plainly: "the roof's planes are not tested."

### 2b. As a siding and exterior trim contractor

- **`dress/one-each/03` (brick wall selected).** Red brick in running bond with a mortar line, a white-trimmed door and
  double-hung window with a head drip cap, and the wall panel gives NET FACE 180.00 SF, LENGTH 24 LF, FINISH "Brick
  veneer, red, on its south face. The line 'Brick veneer, red'". Take-off rows: face area (plan length x height), height,
  net area (face - openings), opening area, plan length. A veteran siding man reads this at once. The finish is read from
  what is priced on the wall, so what is drawn is what is priced.
- **`layers/03`, `house/05`, `light/after-street`, `glass/06`.** Cedar lap siding in a warm tone, a stone wainscot
  course at the foot, white casings with a head cap at each window, a mahogany door. The cladding looks like a house
  and I could believe it. At the corners I can see a corner board (`scene.rs` draws it for lap, not for masonry).
- **`gablew/01`.** Wall 1 selected, lap siding on its south face, "Framed in 2x6 at 16 in on center", "A weather barrier
  under its finish", assembly "Exterior wall, 2x6, lap siding". The siding boards are drawn at an exposure that reads as
  lap. Good.
- **`house/01`, `light/after-window` (a window close up, lap siding).** The casing is flat with a head cap, a sill, and the
  sash has divided lites in a bronze-brown. As a trim man I can see the trim is drawn: head casing wider than the jambs, a
  drip cap over it.
- **`dress/one-each/04`.** Eave: a white fascia and a flat soffit, drawn; I count that as drawn for the eye (`scene.rs`
  says "for the eye only").
- **Missing on every picture:** a gable end (there is no gable on any model I saw; `STATUS.md` J24A-17 says so), a
  frieze, a belly band, a water table, a rake board, a column wrap, shutters, any trim line I can click for its LF.

### 2c. As a window and door installer

- **`light/after-window` (a first floor window, big).** The unit has an upper and lower sash, a meeting rail, divided
  lites in a 4x3 grid, a bronze frame, a casing with a head cap, and a sill. When it is on screen it looks like a window.
- **`layers/03` and `house/05` (whole house).** Eight windows on the first floor and nine on the second (the Job panel
  says "Doors and windows 5 and 8", "4 and 9") in a repeated pattern. I can see the sills are at one height and heads
  at another from the door's head: the door head is visibly lower than the windows' heads. A builder wants those on one
  line (door 6'8" = 6.67 ft here, window sill 2.5 ft + 5 ft = 7.5 ft head). Nothing flags it.
- **`dress/one-each/03` (wall panel, Opening area 36.00 SF "of each of the 2 openings in this wall, added up").** The
  panel gives opening area. It does not give a count by size or a schedule.
- **What I cannot find on any picture:** a window or door schedule; a click on a window that shows mark, unit size, rough
  opening, head height, sill height, egress, tempered glass, swing; the rough opening framing (the framing picture
  `trades/2-framing` and `house/` do show kings, jacks, header, sill and cripples at openings - that is good - but no
  size label).

### 2d. As a waterproofing and flashing man

- **`trades/7-waterproofing` (selected window with Finishes off).** Cladding comes off, I read the wall's face as the
  weather barrier, and the flashing round the window is drawn: head strip, jamb strips, sill strip in a darker blue-grey
  band against the grey wall. Zoomed in I can see the flashing is a band round the opening, and at a glance it reads
  correctly: the head flashing over the jambs. This is the most useful picture for me, and it is built from what the
  estimate prices (a line named "Flashing and sealant").
- **`trades/2-framing`, `layers/` framing on.** The studs show where the wall's sheathing and barrier would be; no
  sheathing drawn on the frame itself, which is right for that view.
- **Roof-to-wall line:** no picture shows one. The headwall exists in the model (`Kind::Headwall`, a
  `headwall_length` figure, "Headwall flashing" in the shingle assembly) but the sixteen house has none.
- **Penetrations:** `clashes` finds "east wall [wall-0011] and refrigerant line set, piece 1 of 4: through a wall (a hole to
  cut)", a return riser through the north wall (an outside wall, at 5.45 ft), and the plumbing vent "vent through the
  roof [pipe-0254]" ending at z 23.6. Three outside-envelope penetrations, each found. Each says "nothing priced rides on
  the hole itself". For me the hole IS the work: a pipe boot, a line-set hood, a duct termination with a damper and a
  flashing. Not seen on any picture: a mark on the roof or the wall where the hole goes.

---

## 3. What is missing or wrong for me

Each: what I looked for -> what I got -> why it matters.

**Roof**
1. A tie-in of a new roof to an old wall or roof -> the roof is one solid of planes; the old hip is not cut back, valleys 0
   LF, headwall 0 LF (`trades-use-envelope-finishes.md` found it, and the sixteen house confirms it: `valley length 0 LF`,
   `rake length 0 LF`) -> an addition is the most common roofing job, and its valley, step and headwall flashing is the leak.
2. Penetrations drawn on the roof (stack, flue, skylight, curb) -> `Pipe boots` is in the whole-roof shingle assembly
   with `boots = unknown` ("the roof's pipes and vents are not drawn: count them"); the sixteen house has a "vent through
   the roof" 2 in pipe ending at 23.6 ft and `clashes` does not see the roof -> a boot per pipe and a curb flashing per
   skylight are small dollars that leak; and an Unknown count the program could answer itself from a pipe that ends
   above a roof plane.
3. Sixteen house roof lines vs. what a roofer prices -> in standing seam the plane carries only "Roof sheathing" and
   "Standing seam roofing"; no underlayment (the `Standing seam roof` assembly names Underlayment, the sixteen house did
   not apply it), no drip edge, no eave and rake trim, no ridge closure, no valley, no pipe boot, no snow guards -> the
   estimate reads as a complete roof ($13,800 on planes plus $2,970 gutters plus caps) and is short of ~5 lines a roofer
   always adds.
4. Ridge line named "Steel ridge beam" -> a structural member priced on a roof ridge line ($42.00/LF, 12 LF) beside "Ridge
   cap and vent" -> the structural line belongs to division 5 or 6 and a roofer looking at the Roof takeoff reads it as
   roofing; also in the picture the ridge is only a violet line, no cap drawn.
5. Eave height is the height of the eave line (the tail), 19 ft with a 1 ft overhang, so the roof plane at the wall plate
   is 6 in higher than a builder would say the plate is -> `d2/04` "eave height 10 ft". Nothing says "wall plate". The
   fascia drop is fixed at 0.7 ft "for the eye only" (`scene.rs FASCIA_FT`) -> heel height and plate height are what a
   framer and a roofer talk about, and the fascia depth changes the trim and gutter quantity.
6. Tear-off and layers (existing roof) -> the plane has Existing / To demolish / New work and that is good, but no layers
   count, no deck condition, no "re-deck allowance" (`trades-use-envelope-finishes.md` already says it) -> a re-roof is
   priced by tear-off squares and bad deck LF, and both are the swing in the bid.
7. Ice barrier width, shingle assembly: `eave_width = 3 ft` Assumed flat; with a 24 in overhang the 24 in inside the wall
   line is 3 ft horizontal, about 40 in along the slope at 6:12 (slope factor 1.118) -> a fixed 3 ft is short by ~4 in
   along the slope, and it is the same 3 ft whether the overhang is 6 in or 24 in. The number should come from overhang
   and pitch.
8. Rakes, gable ends: no gable in any picture; `STATUS.md` J24A-17 says the triangle is neither drawn nor priced -> short
   by about 72 SF of framing, sheathing, barrier and siding on a 24 ft gable at 6:12 (that is PROPOSALS 29, open).
9. Gutters: `eave-0077..0080` carry "Gutter" at $16.50/LF = $1,683 for 144 LF, drawn in code (`scene.rs` hangs a
   gutter on a fascia where one is priced) but I see no gutter in `layers/03`, `house/05` or `dress/one-each/04`; downspouts
   are not a line anywhere (`trades-use-envelope-finishes.md` says so too) -> "a gutter is priced and none is drawn"
   (STATUS owed list). Not seen in any picture after the code; may be fixed.

**Siding and trim**
10. Siding over a wainscot -> in the sixteen house the "Cedar lap siding, installed" line is 329.99 SF on the south wall,
    which is the whole net area (400 - 70 of openings), while "Stone veneer wainscot, 3 ft" is also priced on the same wall
    at 40.00 LF; the wainscot's 3 ft x 40 LF = 120 SF is stone in the picture (`layers/03`) and cedar in the estimate ->
    siding is paid twice over the stone: about 120 SF x $9.50 = $1,140 on each long wall and $3,876 on the first floor's
    408 SF of wainscot. The library does the same (`Exterior wall, 2x6, lap siding over a stone wainscot` prices Lap
    siding at `net_area` and the stone at `plan_length`). The stone is 40 LF with no deduction for the door's 3 ft either.
    (This is the one line I would call wrong, not just missing.)
11. Weather barrier and siding with no waste -> in the library the "Weather barrier" and "Lap siding" rows have a blank
    Waste % (the "Gypsum board" row has 10 %), so siding is bought at exactly the area, and housewrap goes by SF, not by
    the roll (the "Sheathing and weather barrier" line in the sixteen house, $2.10/SF) -> a siding man buys boards in
    12 ft lengths, with 10 to 12 % waste; the order comes up short.
12. Corner boards, J-channel, casing, frieze, fascia, soffit, rake, belly band by the LF -> corner boards, fascia and
    soffit are drawn (`scene.rs` 797-810, 1091-1130) and none is priced in the sixteen house; casing exists as 1 EA at
    $120 per door and none at all on the windows in the sixteen house, though every window picture shows a head-capped
    casing -> what is drawn is not what is priced: about 4 corners x 19 ft and 144 LF of fascia and soffit in
    the picture, nothing in the estimate. (Already said in `trades-use-envelope-finishes.md` item 7; I confirm it still
    holds and add that it holds on the one house the program ships.)
13. The wall's measures: `plan_length`, `height`, `face_area`, `opening_area`, `net_area`, `thickness`, `studs`,
    `plate_length`, `header_length`, `sill_length` -> no `corner_length` (outside corners LF), no `perimeter`, no
    `siding_area` (net less wainscot), no `top_length` -> a siding man cannot price corner trim from the wall.
14. Siding face by direction -> "lap siding, cedar, on its south face" is read per wall; the panel says which face; fine.
    But a wall that is brick on one side and siding on the other (`dress/one-each/01`: one wall side is brick, the
    adjacent face is plain stucco-look) reads right; no picture shows a mixed-cladding corner.

**Windows and doors**
15. A schedule -> none; an opening has width, height, opening_area only (`components.rs` measures for Door and Window),
    so I cannot price by a mark, a size group or a supplier list (trades-use already: "no window/door schedule").
16. Rough opening versus unit -> the type is "Window 5 ft x 5 ft"; no rough opening, no clearance, no header height to
    the unit -> the framer and the installer work to the RO, the supplier to the unit.
17. Heads that do not line up -> door head 6.67 ft, window head 7.5 ft on the same wall, no flag -> a flag costs nothing
    and catches a drawing error that a stucco or siding man sees as a jog in his courses.
18. Flashing per opening is a flat $65.00/EA "Flashing and sealant" for every window, 5 x 5 or not -> a flashing kit,
    tape and sealant are by the LF of opening (head + 2 jambs + sill); the program has the width and height and could
    work it; the door has no flashing line at all (`Entry door` in the library: door, hardware, casing, hang). Doors are
    the leakiest opening and the one with no pan.
19. `finishes` says each window is "flashed" because a line says so, but the estimate says nothing about a sill pan,
    jamb tape or a head flashing separately -> ASTM E2112's sill pan is its own work (below).
20. No egress flag, no tempered flag, no safety glazing near a tub or a stair; grep of the crates finds neither word
    outside a test -> inspectors fail a house on these.

**Flashing and penetrations**
21. Penetrations of outside walls (the line set at 25 ft up the east wall; the return riser at 5.45 ft up the north
    wall) -> found as clashes ("a hole to cut"), with "nothing priced rides on the hole itself" -> a line set cut needs
    a sleeve, a sealant and a hood, and a duct termination needs a damper and a flashed hood: the program finds the
    hole and prices nothing for it.
22. No ledger flashing, no kick-out, no step-flashing LF along a roof-to-wall line (there is no such line in the
    house); the headwall flashing exists in the shingle assembly and the sample (`sample.rs` copper headwall, 24 in
    stretch-out) -> fine as a quantity, nothing in the 3D view.
23. Brick: the library has "Brick ties and flashing" at `net_area`, one line -> through-wall flashing and weeps are by the
    LF of the course, not by net area; it is a wrong quantity base, small dollars, big callback.

---

## 4. What the best tools of my trade show today, and where this could be ahead

- **Aerial roof reports (EagleView).** A sample report lists ridges, hips and valleys each by length and count, rakes
  and eaves with a definition ("Rakes are defined as roof edges that are sloped (not level). Eaves ... not sloped and
  level"), drip edge as eaves + rakes ("Drip Edge (Eaves + Rakes) = 378 ft (38 Lengths)"), flashing and step flashing
  as two separate lines with lengths, and penetrations as a count, a perimeter and an area
  ("Total Penetrations = 3 ... Perimeter = 6 ft ... Area = 1 sq ft").
  Source: https://www.eagleview.com/wp-content/uploads/2025/07/Premium-Sample-Report-Residential-1.pdf (as quoted
  from search results; the PDF itself did not extract, so I did not read it end to end).
  What this program already matches: separate eave, rake, hip, ridge, valley lengths and drip edge = eave + rake
  (library row "Drip edge", `eave_length + rake_length`), with the same definition of eave and rake.
  What it lacks that the report has: step flashing and flashing as two lengths; penetrations as a drawn count.
- **Hover (phone-photo 3D model).** Its help centre lists the measurement panels "Walls", "Roof", "Siding", "Shutters",
  "Trim, fascia, and soffits", "Openings including windows, doors, and garages"
  (https://help.hover.to/en/articles/13367306-3d-measurements), and its siding page says it gives "siding areas,
  elevations, openings, gutters, downspouts" (https://hover.to/measurements/siding). A trade review describes trim,
  corners, roofline and openings in a siding chart (https://www.proremodeler.com/products/article/55184185/pros-pick-hover).
  So the competitor's report carries trim, fascia and soffit LF, downspouts, and a list of openings; this program
  draws fascia and soffit and prices none of them, and has no downspouts.
- **The standard for window and door flashing, ASTM E2112.** Summaries say it recommends a sill pan under all windows
  and doors in low-rise residential, in the order sill, then jambs, then head, with the head over the jambs, and sloped
  pans where the sill depth is over 6 in (https://iibec.org/publication-post/a-flash-in-the-pan-a-field-guide-for-windows-and-doors/,
  https://basc.pnnl.gov/resource-guides/windows-and-doors-are-fully-flashed, and NAHB's technote
  https://www.nahb.org/-/media/NAHB/advocacy/docs/top-priorities/codes/technotes/tn05-window-and-door-flashing-201705.pdf).
  I did not read the standard itself (it is paid); these are secondary summaries, and one source gives tape widths
  (4 in jamb, 6 in or wider sill) as practice not as standard text. This program's picture `trades/7-waterproofing`
  already draws the sequence as three strips at head, jambs and sill; it does not say which order or whether there is a pan.
- **Where this program could be ahead of all of them.** None of the aerial or photo tools builds the inside, so none
  can: (a) tell a roofer the vent pipe, flue and duct that go through a roof are the penetrations - this program
  has the pipe and the roof in one model, and the stack is 23.6 ft up against a roof plane; (b) tie a new roof to an old
  wall with the old wall's real framing; (c) show a window's head against the door's head, and the framing and
  the flashing and the siding in one clickable stack on one wall (the See through and Finishes switches already do
  nearly that); (d) say Known / Assumed / Unknown on a pitch, an eave and an overhang so a roofer sees which ones
  the field must check (it does: `d2/04`); (e) move an eave and have the valleys, hips and the dollars follow.

---

## 5. My ten, in order

Most valuable first. Each: what the screen shows, what it is called, what a click does; how one would know it is done.

1. **Price what is drawn: fascia, soffit, corner board, window and door casing by the LF.** A wall panel row "Corner trim,
   4 outside corners, 76 LF"; a plane or eave panel row "Fascia, 144 LF"; "Soffit, 144 LF x overhang"; "Casing, 5 openings,
   96 LF" with a line the user prices. Click the row, see the board lit in 3D. Done when the sixteen house's estimate has a
   line for each of the four things that its picture already shows, and `finishes` lists them.
2. **Fix the stone and siding overlap.** "Lap siding" quantity = net area less wainscot area (and the stone LF less door
   widths), shown on the wall as "Siding above the stone, 210 SF" and "Stone, 120 SF". Done when the sixteen house's
   south wall siding is 330 less 120 = 210 SF, and a test says the two cannot add to more than the wall.
3. **Penetrations as items, on the roof and the walls.** Where a pipe or duct passes through an outside wall or ends above a
   roof plane, a small marker "Pipe boot, 2 in" or "Line set sleeve and hood" on that surface, counted into the `boots`
   figure and into a priced line; click it and go to the pipe. Done when `boots` stops being Unknown on the sixteen
   house (1: the vent) and the line-set hole has a line.
4. **A window and door schedule, one click from any opening.** A list "Windows: W1 5 x 5, 17 of them; D1 3 x 6-8, 1" with
   mark, unit size, rough opening (unit + 1/2 in each way, Assumed, editable), sill height, head height, egress and
   tempered flags, and a button "Heads line up" that flags door 6.67 vs window 7.5 on a wall. Done when the sheet
   exports it (CSV and the supplier lens), and the flag fires on the sample.
5. **Flashing by the LF of opening, and one on doors.** Replace the flat $65/EA with `2 x height + width` (head + jambs) and
   a sill pan line of `width`; add the same to the door assembly. Show the sequence on the opening's panel: "Sill pan, jamb
   tape x 2, head flashing". Done when a 3 x 6.67 door has a flashing line and a larger window costs more than a smaller one.
6. **Roof tie-in as a command.** "Tie into wall" on a roof plane edge: cuts the new plane back to the old wall or old
   roof, creates the headwall (step flashing) or the valley and a cricket where needed, and says what it measured as Assumed.
   Done when an addition on an existing house gives non-zero valley and headwall LF in `takeoff` and draws them.
7. **The roof the sixteen house is priced as: underlayment, drip edge, eave/rake trim, ridge closure, valley, boots.** The
   "Standing seam roof" assembly exists with Underlayment; give the metal roof the same whole-roof treatment as the shingle
   one (drip edge = eave + rake, closure strips, valley, headwall, boots). Done when `estimate` on a standing seam roof
   has the same shape of lines as on shingle.
8. **Roof by the roofer's own units and conditions.** A Roof panel with "New 12.4 SQ, Existing 1.6 SQ, Tear-off 1.6 SQ x 2
   layers", Waste % per shape (gable 10, hip 12 to 15) shown on the line, and the ice barrier width worked from overhang
   and pitch (24 in inside the wall plus overhang, along the slope). Done when the ice barrier row changes when the
   overhang changes.
9. **Gable ends and rakes.** A wall that ends under a gable edge gets its triangle (drawn, priced as framing, sheathing,
   barrier, siding), and the rake gets rake trim and drip edge. Done when the 24 ft gable has 72 SF more, as PROPOSALS 29 says.
10. **Water-path view.** One switch "Water path" that lights, on one colour, the barrier, pan and head flashing at every
    opening, the headwalls, the kickouts and the penetrations, and counts them ("Openings 17, pans 17, head flashings
    17, penetrations 3, none missing / 2 missing"). Done when removing the flashing line from one window changes its
    count from 17 to 16 and lights that opening.

---

## 6. What must not change

- **Finish read from the price line, and the picture follows the price** ("Brick veneer, red, on its south face. The line
  'Brick veneer, red'"; the window as "double-hung, divided lites, bronze"). This is the right idea and the reason I can
  trust a picture: keep it, and extend it to the new trim lines instead of drawing them without a price.
- **Roof vocabulary and arithmetic on the panel**: pitch as 6:12, slope factor shown as a formula, SQ, eave, rake, hip,
  ridge, valley, headwall as kinds with the right definitions (eave level, rake sloped, ridge level between two planes).
- **Move an eave and the roof re-solves and says what it did to Known/Assumed** (`d2/04`).
- **Existing / To demolish / New work on a roof plane.**
- **Shingle whole-roof assembly with its sources** (underlayment laps, drip edge piece length, starter, ice barrier,
  valley, headwall, boots as an honest Unknown). It carries the right lines; do not flatten it.
- **Flashing drawn at an opening when Finishes is off** (`trades/7-waterproofing`) and the framing of openings (kings,
  jacks, header, sill, cripples) in the framing view.
- **Clash lines that say "a hole to cut" for pipe and duct through a wall, and "the roof's planes are not tested" stated
  openly.** Say the gap; do not hide it.
- **Door/window/wall opening area and net area shown with their formula** ("face area - opening area").
- **Honest "no figure" when a line has no price** instead of a zero.
