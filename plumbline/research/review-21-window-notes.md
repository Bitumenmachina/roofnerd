# Review 21, window crate: notes as I go

Reviewer: independent, by reading. Files read whole so far: main.rs, model.rs, tools.rs.

## R21-1 Closing: one "warned" flag for two windows; work can be lost without a word
main.rs:281-297 (`closing`). `warned` is a single Local<bool>, reset only when the job is
saved. Bevy's WindowPlugin default exit condition is "all windows closed", so closing
one window closes only that window.
Sequence: edit; close the sheet window -> "Not closed: ... close again to leave without
saving"; close the sheet again -> the sheet goes. Keep working (more edits); close the
model window -> `warned` is still true, so it closes at once, and with the sheet already
gone the program ends: every edit since the first warning is lost with no word.
Also: the warning says "close again to leave without saving them", but closing again
closes only that one window, not the program; the other stays and the work is still not
saved. Untrue message. The sheet, once closed, cannot be opened again (nothing
respawns it). Serious (a lost value). Suspected (read, not driven).

## R21-2 Every keystroke in a field reworks the whole job, clash detection included
main.rs:549-551 calls doc.touch() on every changed character; model.rs:499-517
`refresh` then runs view::elements, pricing, estimate, clash::clashes, vif, dress,
placements, takeoff, storey_totals... on every revision. `touch` is also used for
messages, hovers etc. On a whole house this is per character typed (and per laser
burst character). Not a wrong number; it makes typing lag on a large job and the laser
burst detection (BURST_GAP 0.12 s) is measured against frame times that this work
lengthens. Fragile. Suspected (performance not measured).

## R21-3 Window works out the length of a wall it reports (one arithmetic)
tools.rs:387: "Added wall 3, 12' 4" long." is the window's own hypot of the two picked
points, not the model's length of the wall it made. If the model measures differently
(snapped/rounded ends: editing::toward rounds to 1e-4 ft; a wall joined/trimmed at a
corner) the message and the sheet can disagree. Breaks "one arithmetic for one number".
Minor wrong figure in a message. Suspected.

## R21-4 Roof tool: a refused closing loses every corner and puts the tool down
tools.rs:445-448: the corners are taken (`mem::take`) and the tool set to None before
`roof_over` runs; if solve::new_roof refuses the outline (self-crossing, too small) the
estimator has lost all the corners clicked and the tool, with only the refusal said.
Compare walls (tools.rs:386), which keep the start on refusal. Lost work. Suspected.

## R21-5 Delete does not do what undo does for the level on show and the roof on show
tools.rs:712-734 (`delete`) clears only `selected` and `hover`. `step` (684-709) also
clears `show.storey` when the level is gone (and puts the tool down) and `roof` when the
roof is gone. If what is deleted is the level on show (or the roof on show), Doc keeps a
path to nothing: the tool stays in hand on a level that is not there (picks refused by
the model with its own words), and the sheet's roof tab shows a roof with no name
(model.rs:555-597 runs with a stale path). `refresh` (model.rs:538-543) also only
re-checks `selected` and `placing`, never `roof` or `show.storey`. State kept in two
places that can disagree. Suspected (depends on whether a level/roof can be selected
and removed; ed::remove decides).

## R21-6 A panel edit followed by Z in the next frame records the edit as an empty step
Update chain order (main.rs:189): keys ... tools::record ... panels::rebuild, sheet::rebuild.
An edit made by a panel/sheet button (doc.edit in panels/sheet) is recorded only on the
NEXT frame's `record`; `keys` runs before it. If an undo happens in that next frame,
history.undo restores the state before the last *recorded* step (dropping the panel
edit with it), and then `record` stores a step labelled with the panel edit whose
before and after are the same. Needs a key within one frame of a button (a script, a
laser, or the slow software renderer at 1 frame/s makes it likely). Fragile. Suspected.

## R21-7 Pressing a button while a field is open: the button acts, other presses are dropped
main.rs:455-461: while an entry is open, doc.pressed is cleared every frame. Doc::press
(model.rs:379-387) closes the entry before pushing the key, so normally this is fine;
but any press pushed by code that does not go through `press` (or a press made the
same frame an entry is opened) is silently thrown away. Fragile, untidy.
(Check on panels: whether anything pushes to doc.pressed directly.)

## R21-8 Clicking on the plan while a length is being typed does nothing and says nothing
tools.rs:824: with doc.entry Some (Length, Room, Distance, Measure...) a left click on the
model returns without a word. Only the Settings case (816-823) is answered. J18-2's rule
"never a click that does nothing" is kept for one entry kind only. Wrong/missing word.
Suspected (scene::pressed may also see the click; to check).

## R21-9 Comma in a field of a form is always "next field"
main.rs:505-513: in a form (tool sizes, a new level) a comma moves to the next field and
on the last field is silently dropped. A fixture named "Sink, double bowl" (Thing's
"What it is" is the FIRST field) is split: "Sink" and " double bowl" goes into the size
field, which is then refused as "not a size". A level named "Ground, east" likewise. The
comma on the last field vanishes without a word. Minor lost character.

(Read whole since: panels.rs, sheet.rs, map.rs, theme.rs.)

## R21-10 The budget tab works out the like-for-like difference and its percent itself
sheet.rs:1077-1084 (`money`): `cents(d - a)` and `diff / a * 100.0`, then rounded to 2
places by `view::figure`. The model gives each element its difference and percent
(`e.difference`, `e.difference_percent`, used at 1034) but the job's like-for-like
difference and percent are worked out in the window. Breaks "one arithmetic for one
number": the CLI and the workbook must work the same figure separately, and nothing holds
them to agree. Also: the percent has no status of its own; it is drawn in `status` of the
pair. Wrong-layer arithmetic. Reproduced by reading (the arithmetic is there).

## R21-11 A division with lines and no figure reads "none", in the colour of Known
sheet.rs:800-803: the heading of a division's block in "The sheet" says `"none"` with
`Status::Known` when `d.total.value` is None. But the block is only drawn when the
division HAS lines (`its` non-empty), so None here means "work in it, none of it priced"
which the "By division" table just above (778) calls "no figure" in red. Same division,
two words, and the sheet's one shows an unpriced division as settled. Wrong word and a
status surer than what it rests on. Same at 809: "No division" heading with no figure is
an empty string (unwrap_or_default), silently blank. Reproduced by reading.

## R21-12 Negative dollars written two ways; money formatted in a dozen places
`sheet::dollars` (sheet.rs:641-651) writes a negative as "-$12.00". Every other money
figure is `format!("${}", view::figure(x, 2))`: sheet.rs:686, 696, 777, 789, 801, 809,
944, 980, 1062, 1084; panels.rs:89 (`amount`), 312; main.rs:588; and it gives "$-12.00".
A credit line or a deduct shows "$-12.00" in the recap's class rows and "-$12.00" on the
Price row beside it. Money formatting is duplicated about fifteen times instead of one
helper. Wrong word, untidy. Suspected (needs a negative line; the model may refuse a
negative unit cost; a negative difference is shown with "under" so is fine).

## R21-13 The "not priced at all" words are built twice
sheet.rs:1089-1092 rebuilds the list "5 eaves, 2 doors" with its own "+s" plural;
`view::not_priced_said` (world view.rs:518-525) builds the same list the same way, and
is what the Estimate tab uses (sheet.rs:733). Two copies of one sentence. Both add "s"
to `Kind::said()`, which gives "2 equipments" for equipment (components.rs:130-136 says
"equipment"). Duplicate logic; wrong word (the plural belongs to the world reviewer).

## R21-14 Clicking a row of the sheet while a field is open does nothing and says nothing
sheet.rs:128, 629, 1120: `doc.entry.is_none()` guards every row click. With a value
half typed (or a laser reading's field still open), a click on a sheet row is thrown
away with no word, where `Doc::press` and `Doc::open` let the field go and say so
(model.rs:322-346). Same in tools.rs:824 (R21-8). Missing word. Reproduced by reading.

## R21-15 Same name on two controls in the map: "Pitch" button and "Pitch" Measure row
panels.rs:1339 makes a button named "Pitch" (theme::button -> named(label)); panels.rs:1298
names a measurable property row `theme::spoken(label)`; for a roof plane whose pitch is
in `snap.takes` the row is "Pitch" too. map.rs:87-99 then numbers them "Pitch" and
"Pitch #2" by height on screen, so which one a script clicks depends on the layout.
Likewise sheet rows are named by `spoken(path)` (sheet.rs:126): two roofs each have a
"plane 1", and two systems may share a name ("System Supply Air", panels.rs:753). The
map is what every window demonstration drives; a name that changes meaning with the
layout is the failure the map was written to remove. Fragile. Suspected (depends on
`takes` for a plane).

## R21-16 Picking an item sets plan/walk directly, bypassing set_view
panels.rs:874-876: `doc.plan = false; doc.walk = false;` instead of `doc.go` / `set_view`.
On foot, the view is not put back (`orbit.go_home` is not called; "Back outside" not
said), so the orbit keeps the eye-height target of the walk and the camera eases from
the walk to a 3D view about a point inside the house. A tool in hand stays in hand but
`clicks` ignores it off plan. State changed in two ways for one thing. Untidy, possible
jump. Suspected.

## R21-17 Stale and misplaced comments
theme.rs:4-7 says "The look is an instrument: ... no shadows and nothing rounded off":
call 62 replaced that and the same file now defines PANEL_RADIUS, panel_shadow etc.
sheet.rs:493 "A figure of money as the sheet says it" sits on `clashes()`; `dollars` is
at 641. sheet.rs:888-890: a comment about hours over `let places = 2;`, a constant
standing for nothing. main.rs:272-273: "/// Save the job to its file." sits on the
`Shutting` struct, not on `save`. panels.rs:210 `docked(_edge)` takes an argument it
ignores at every call. Untidy.

## R21-18 `spoken` lowers a kind's capital inside what a person typed
theme.rs:285-304 lowers "a Wall", "A Roof", "a Door" ... over the WHOLE text, quotes and
all, although 219-232 promise that what is inside quotation marks is left as typed.
"Added room \"A Roof Deck\"." is drawn "Added room \"A roof Deck\"."; a room named
"A Door Shop" is drawn on its plan pill (panels.rs:1191 -> quiet -> text -> spoken) as
"A door Shop". `american` likewise turns a name word "grey" or "centre" outside quotes into
"gray"/"center" (a material called "grey slate" is shown "gray slate"). A name a person
gave is changed on screen. Wrong word (minor). Reproduced by reading the code path.

(Read whole since: scene.rs, plans.rs (app), bring.rs, glass.rs, crates/plans/src/lib.rs.
Skimmed only: dress.rs, skylight.rs (no input from a person reaches them).)

## R21-19 A maker's sheet: the next column is read into the model number (REPRODUCED)
crates/plans/src/lib.rs:577-588 (`words`). A letter several heights past the one before
is marked a column apart with three spaces (580), then line 588 splits the line on ' ',
drops the empty pieces and joins with one space: every column gap is gone. The world's
`spec::after_label` (world spec.rs:305) ends a value "at the end of its column" by
`split("   ")`, which can now never match. So the model, the maker (and any label read
the same way) run on into whatever stands beside them.
Reproduction (made-up sheet, gen/mkpdf.py, a Helvetica page with "Model: MV-3621" at
x=72 and "Finish: White Oak" at x=320 on one baseline):
    python3 -I gen/mkpdf.py vanity.pdf
    plumbline new t6.json "Test house" (+ a level)  ;  plumbline import spec t6.json vanity.pdf --as furnishing
  got:      ~ model   MV-3621 Finish: White Oak  "Model: MV-3621 Finish: White Oak"
  expected: ~ model   MV-3621                    "Model: MV-3621   Finish: White Oak"
The quoted line of the sheet that is the value's source is also not what the sheet says
(the gap is gone). A value made up from the next column, Assumed and sourced to the
sheet. The window's Bring in > Maker's sheet goes through the same `words`
(bring.rs:98). Serious (a value made up). Reproduced.

## R21-20 A drawing whose reader panics stays "reading its page" for ever
app plans.rs:171: `try_recv().map_err(|_| ())` treats Disconnected like Empty. If
`plumbline_plans::read` panics on a damaged or unusual file (hayro, image, a bad
allocation), the thread dies, the sender is dropped, and the drawing is listed as
"reading its page" for the rest of the run (panels.rs:779-780), with no word. It is
never put in `faults` and never retried. A silent hang of one feature. Suspected
(needs a file that makes the reader panic; the code path is plain).

## R21-21 Bringing a price list in, adding a drawing, or setting a scale makes the view jump
bring.rs:159, plans.rs:296, tools.rs:674 set `doc.framed = false`. model.rs:643 then moves
`doc.origin` to the middle of the new extent and scene.rs:403-414 sets `orbit.target`,
`orbit.home` and `orbit.now` outright (`now` is where the camera IS; setting it is a cut,
not an ease). Bringing in a price list changes nothing that is drawn, yet the view cuts
back to the home framing from wherever the estimator was looking. Breaks the fixed rule
"no view ever jumps". Reproduced by reading (bring.rs:159 runs for every Bring kind).

## R21-22 Deleting the level on show: the whole model disappears, the tool stays in hand
CLI check: an empty level can be removed (plumbline remove t4.json level-0020 ->
"removed level-0020"); a level with things on it cannot. In the window: Add level
"Upper" (tools.rs:318 puts it on show), click its row (select), Delete. tools::delete
(712-734) does not clear `doc.show.storey` (step() does, 695-698), so `show.storey`
names a level that is gone: scene.rs:533 then filters out EVERY element (none is on that
level), the levels list shows every switch off (panels.rs:601), and a tool in hand picks
on a level that is not there. Nothing says why the house vanished. This is R21-5 made
concrete. Serious (looks like lost work). Reproduced in part (CLI), the rest by reading.

## R21-23 The ground grid's line count grows with the square of the job's reach
scene.rs:1632-1650 draws (2*steps+1)*(2*steps)*2 gradient lines every frame, steps =
reach/10, reach = 2.4 x the diagonal of everything (scene.rs:414), drawings included
(model.rs:633-637). A scanned sheet of 7200 x 10800 dots, before it has a scale, is shown
at 0.1 ft a dot (plans.rs:27): 720 x 1080 ft, diagonal ~1300 ft, reach ~3100 ft, steps
310, about 770,000 lines a frame, on a renderer that already runs at about a frame a
second. Most are faded to nothing (`fade` <= 0) but are still looped and many drawn.
A likely hang-like slowdown from bringing in a plan. Suspected (worked by hand).

## R21-24 Carrying a thing turns the clash marks on and can leave them on
scene.rs:1335-1341 switches `show.clashes` on at the press on any standing thing, and
1408-1411 switches it off at release only if the thing is clear. A plain click (no move)
on a vanity that already clashes leaves the Clashes switch on, though the person never
touched it. Minor; a switch that does not read what the person set.

## R21-25 Setting an item down: placed on the lowest level, whatever was clicked
scene.rs:1291-1299: with every level on show the item goes on the lowest level at the
plan point of whatever was hit (an upper floor, a roof), at "0 ft" above it, and the
message does not say which level. Clicking the upper floor of a two-level house puts the
vanity in the basement under it. Wrong place, said as "Set one down." Suspected.

## R21-26 Base drawn in the window by its own geometry
scene.rs:476-528 works out where the base board runs and where each door cuts it
(door ends projected onto the room's outline, within half the wall plus 0.1 ft). The
estimate prices base from the core. Nothing holds the drawn base to the priced LF: a door
in a wall that meets the room's outline at an angle, or within 0.1 ft tolerance, is cut
in one and not the other. The rule "what it is dressed in is what the estimate prices"
and "no window recomputes a quantity". Suspected.

## R21-27 Supplier name with a full stop: the price list is "not there"
bring.rs:71-74: the supplier after the comma is taken only if it has no '.'. "lists/
ridge.xlsx, J.R. Lumber" or "..., Ridge Supply Co." is taken whole as the file's name and
refused as "lists/ridge.xlsx, Ridge Supply Co. is not there". Wrong message. Reproduced
by reading. (Also: a dropped table is silently forgotten when Model or Maker's sheet is
clicked next, panels.rs:848-849.)

## R21-28 The same "lowest wall base" worked in two places
main.rs:361 and scene.rs:1024 each fold the minimum wall base out of `snap.drawn`. Two
copies; the view's offset under the ground and the ground's own height can drift apart
if one is changed. Untidy.

## R21-29 Stale module comment in scene.rs
scene.rs:1-3: "No sky, no sun, no grass" and "a neutral ground with a grid and no
horizon"; the file makes a sky dome (190-197), a sun (207) and fog to a horizon. Untidy.

(Read since: crates/ifc/src/step.rs whole; crates/ifc/src/lib.rs lines 480-670 (levels,
walls); write.rs not read.)

## R21-30 IFC: two levels at one height; the lower is said to have "no level above"
crates/ifc/src/lib.rs:545-555: a storey whose next storey is within 0.01 ft of its own
elevation falls into the "top storey" arm: its height is made Unknown with the note "the
model has no level above this one to say how tall it is", which is untrue (there is one,
at the same height: a mezzanine entered twice, or two buildings). Wrong message. Suspected.

## R21-31 An assembly clicked while a field is open is put on later, by surprise
main.rs:921-939 handles `doc.give` only after the entry branch (450-668) has returned.
C opens the list, B opens the allowance field and the list stays (choosing is not
cleared by `Doc::open`); a click on an assembly sets `doc.give` and nothing happens.
When the allowance is typed and Enter pressed, the NEXT frame puts the assembly on, and
its message replaces the allowance's. An edit the person did not see happen when they
clicked. Suspected (by reading; needs the window).

## Corrections to earlier notes
- R21-6: panel and sheet clicks are observers and run in PreUpdate, before `keys`; the
  failure needs a key and a click in the SAME frame (undo then drops the unrecorded click
  edit and `record` stores an empty step under its name). Still possible on a frame a
  second; lower than first written.
- R21-12: the model refuses a negative price ("-250 is below zero; a price is zero or
  more", CLI checked), so "$-12.00" needs a negative quantity or total; the finding is
  mainly the money formatting copied ~15 times. Kept as untidy.
- R21-11: CLI confirms the condition: a division with one unpriced line comes out
  "no figure" with "left out" in `plumbline estimate` (t7.json, Division 6); the sheet
  window's block heading for that same division says "none" in Known ink (sheet.rs:800-803).
- Aside for the model reviewer: `plumbline line t7.json wall-0004 "..." 1` (no unit)
  is priced as "1.00 EA": a unit nobody entered.
