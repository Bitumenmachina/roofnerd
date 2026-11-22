# Judgment of the window against TRY-IT.md section 3 (call 161), written cold

Frozen program bin16, virtual display, software renderer. Pictures and scripts: /tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/judge161/ (p1..p15c, t*/map.txt.said).

| # | Thing | Verdict |
|---|---|---|
| 1 | Bring your scan in | PARTLY: it works, in the right unit and the wrong way up; a wrong unit gives "nothing to draw" with no hint of the unit, or a huge house with no cue (J161-1, J161-4) |
| 2 | Trace the house as it stands | PARTLY: walls, doors, windows, rooms work; "select the walls and press Existing" cannot be done: one wall at a time only, and the page and the screen do not say so (J161-5) |
| 3 | Take down and add on | YES: demolish, undo, addition, Room line, areas follow, Demolition switch all done from the screen; rough edges J161-6, 7, 8 |
| 4 | Set a kitchen | YES: refusals and key list good; the page's corner sentence does not match what the screen does (J161-10); label overlap J161-9 |
| 5 | Try it two ways | PARTLY: keep, open, set beside, Save-first all work with plain words; "make the addition longer" was not tried (no way found by me in this run to lengthen a wall by a typed length; untested) |

Findings follow.
, appended after each session.

## Session A: Thing 1 (scan), empty job, wrong order, wrong units

All pictures in /tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/judge161/ (p1, p2, p3 ...). My made-up scans: a 24 ft x 16 ft x 9 ft room, in meters and in feet.

- Held up: a job file that does not exist opens as "job" with Next steps; Scan file with no level: "Can't bring in the scan file: this job has no level for it to be a scan of. Add a level first." (p2/01). Enter on the empty "or type its name" field: "Can't bring in the scan file: no file was named. Pick the scan from the list, or drop the file on this window." The list shows scans as "side-room / scan" without the file ending; the two Measured choices sit at the head, meters lit (p1/01-model.png).
- Held up: feet file with Measured in feet, meters file with Measured in meters: "56,658 points, measured in feet, Assumed", "about 24' 0" by 16' 0"", "Floor to ceiling reads about 9' 0"" (p3/01-model.png).

### J161-1 (confusing) Wrong unit chosen: the refusal does not name the unit
Did: Add level, Scan file, "Measured in feet", clicked the scan written in meters. Result: red "Can't bring in the scan file: study-meters: no point of it is at that height: nothing to draw" (p2/03-model.png). A builder is not told the cause may be the unit choice, nor to try Measured in meters. Retried twice, same words. (Also happened for my lattice "feet" scan read as meters, p3/02; my lattice may be the cause there, see session B with dense random points.)

### J161-2 (confusing, wording) Bringing the same scan in twice
Clicked the same scan row again: "Can't bring in the scan file: there is already something called "study-meters" under ground: two things in one place cannot go by one name" (p3/05-model.png). Says nothing of what to do (Undo it first, or it is already there); "two things in one place cannot go by one name" reads as program talk.

### J161-3 (cosmetic) The scan message card sits over the plan
After a good bring-in the 6-line card (p3/01-model.png) covers the plan's north wall and the top of the scan until something else is clicked; the status bar repeats its first line. The scan itself shows as a plain grey rectangle with no label saying it is the scan.

## Session B: Thing 1 (more), Thing 2 (trace)

### J161-4 (confusing / wrong for the page) Wrong unit the other way: the message gives the size but not the cue
Feet-written scan, "Measured in meters" chosen (dense made-up points): the window DID bring it in, "56,658/60,000 points, measured in meters ... about 78' 9" by 52' 6" ... Floor to ceiling reads about 29' 6"" (p4/01-model.png, said in t4/said-01.txt). A builder would catch it only if he knows his house is not 79 ft long; nothing says "if that is not the size of your house, choose the other unit and bring it in again" (TRY-IT says it; the screen does not). Meters file chosen as feet: red "nothing to draw" refusal (J161-1), again with no cue about the unit.

### Held up: lying scan
24x10x9 room written on its side (p4/03-model.png): message "Which way is up is not clear from the scan. If the house looks to be lying on its side, press the button below." with the button "It is lying on its side: stand it up" inside the card, readable and clickable. Pressed: "...about 24' 0" by 10' 0". Floor to ceiling reads about 9' 0"" and the button is gone. Undo after stand-up said "Undone: Scan." and took the scan out (both steps in one, fine). Confirms the builder's note: after standing it up nothing says it was turned; only the new size says so (said-04).

### Held up: wall tool refusals
Wall with thickness empty then Enter: "Wall: Thickness is still to be typed. Then Enter, and click on the plan." Clicking a corner anyway: red "Nothing was placed: the wall's sizes are still open. Still to be typed: Thickness. Press Enter first, then click." (p5/04-model.png). Clear. But Height has "9 ft" filled and Thickness only a grey "like 6 in" hint, so "accept or type" in TRY-IT is half true: the first wall needs thickness typed.
Typing "6 in" Enter, Enter, then clicking the four corners in turn then the first again: "Added wall 1, 24' 0" long." ... four walls, "Let go. Click a new start, or Esc again to put the tool down." Walls close. Room tool: click inside, type "Study", Enter: "Added room "Study": 364.25 SF inside its walls. If that is more than the room, a wall round it does not close: draw it to meet." (p6/07-model.png).

### Door tool
Door needs Width typed (no default), and Window needs Width, Height, Sill height: refusals read "Nothing was placed: the door's sizes are still open. Still to be typed: Width. Press Enter first, then click." Fine. After typing 3 ft Enter, 6 ft 8 in Enter the strip shows "3 ft / 6' 8" / 0 ft" and "Click the place on a wall" but a red message appears: "A reading came in with no value picked to take it, and was not used. Click the value first (in the panel on the right), then shoot or type." (p7/01-model.png). That is the laser/keyboard-entry message appearing after plain typing a door's sizes: confusing (a builder has not been told of any reading). Door then placed OK: "Added door 1 in wall 1." (Not retried.)

### J161-5 (wrong for the page / confusing) "select the walls and press Existing": there is no selecting several
Selected a wall, shift-held clicks on others: each click replaces the selection ("Selected door 1. Selected window 1. Selected wall 3. Selected wall 4."; panel shows only "wall 4", p7/05-model.png). Existing then marks only that wall: "Wall 4: existing to remain" (p7/06-model.png). Nothing on screen says walls must be marked one at a time, or offers all the walls of the level at once. A builder following TRY-IT 2 ("select the walls") would mark one wall and think it done; the other walls stay New work with no word. Also clicking at a window position selected the window, not the wall (expected).

## Session C: Thing 3 (take down, add on, Room line, Demolition switch)
(pictures in .../judge161/p8 and p9)

- Held up: select east wall, To demolish: "Wall 2: to demolish"; Undo: "Undone: Mark work."; Redo: "Redone: Mark work." (p8). The wall shows red; "Demolition, shown red 1" under Show. Demolition switch off: "Demolition: switched off." and the red wall and its count go from the plan (p9/06-model.png).
- Held up: with the east wall to demolish and no addition, the Study's area goes honestly Unknown: "no loop of walls closes round its point" (p8/05-model.png); a Room line over the demolished wall brings it back: "Study 368 SF" (p8/07-model.png). Addition walls: "Added wall 5, 12' 0" long." etc. Room line: "Room line: from here. Click the other end." then "Added room line 1. It parts two rooms where no wall stands." Rooms' areas follow (Study 182 SF, Kitchen 182 SF after a second room line across the Study) (p9/05-model.png).
- My first attempt put clicks under the right-hand panel (the addition lies east of the house at the plan's first scale); I had to zoom out with the wheel first. Nothing on screen warned me that clicks landed on a panel (a click on a panel just did nothing). Not counted as a defect; a builder would find the wheel (the hint text says "The wheel closes on what is under the pointer").

### J161-6 (confusing) Kitchen drawn before the Room line gets no area and is not told what to do
Room tool, clicked in the addition, typed Kitchen, Enter: "Added room "Kitchen": It has no area yet: Study was placed first and has the space this room's point is in." (said, p9). It does not say "part the rooms with a Room line" (the next thing to do), and "Study was placed first" is about the program's order, not a builder's word. Once the Room line was drawn the Kitchen read 182 SF. (TRY-IT's order draws the room line second, so the builder may meet this.)

### J161-7 (confusing) A Room line that parts nothing is taken silently
A short Room line (34,8 to 34,12) floating inside the Kitchen: "Added room line 3. It parts two rooms where no wall stands." It parts nothing; the picture shows a stray dashed stub with an arrowhead (p9/05-model.png). The sentence is false for it. (A line from wall to wall across the Study at 16 ft was also accepted, correctly parting it.)

### J161-8 (cosmetic) Dimension labels collide
In the zoomed-out plan the "16' 0"" side label runs into "Kitchen 182 SF" and the wall's "16' 0"" label ("Kitchen 182 SF — 16' 0"") and a dashed room line passes through "24' 0"" labels (p9/05-model.png).

### Note: counts
Walls 7 120 LF in the Level panel still includes the wall to demolish, with Demolition off (p9/06). Matches the builder's list ("counts not split", TRY-IT says so).

## Session D: Thing 4 (kitchen cabinets) (pictures p10, p11)

Setup: job made with the command line (house, east wall to demolish, a 12 ft addition, Study and Kitchen, a Room line, a door in the Kitchen's east wall); everything else from the window: zoomed out with the wheel, Starter items clicked, Cabinets tool.

- Held up: Cabinets before Starter items: tool takes a click and shows "Cabinets: the run starts 1' 0" along wall 6, in Kitchen. Type the boxes, like B18 SB36 B24 (an appliance by its name in square brackets, like [Dishwasher]), then Enter." (the job has no items yet: only Enter would tell him). After "Starter items": "Brought in 48 starter items, trade by trade, at typical sizes: each size is Assumed until the product is chosen. They are under Items, on the left..." Clear.
- Held up: click on bare ground with the tool: "Can't set the run: click on a wall's face, on the room's side of it". Enter with the boxes field empty: "Cabinets: The boxes' codes, like B18 SB36 B24 is still to be typed. Enter sets the run once both are in." Code not in the job: "Can't set the run: B99 is not in this job's items. The list above the field has the boxes and the appliances this job has." (the list above the field is on screen: p10/06-model.png, with the one-line KEY "B base · SB sink base · DB drawer base · W wall ... the number is the width in inches" - readable). Run over the door: "Can't set the run: a run of 12' 0" from 1' 0" crosses door 1, which is in wall 6 from 8' 6" to 11' 6": a door is no place for boxes". Good words.
- Held up: `B18 dishwasher SB36` WITHOUT brackets was taken: "Set 3 boxes along wall 6 in Kitchen: a run of 6.5 ft, from 1 ft to 7.5 ft along the wall; 8.25 ft left to the corner. | Countertop: 6.5 ft long by 25 1/2 in deep, 34 1/2 in up; 13.8125 SF, front edge 6.5 LF." Each box shows its code on the plan (B18, Dishwasher, SB36) (p10/07-model.png).
- Second run: clicking the north wall's face near the east corner: "Cabinets: the run starts 2' 0" along wall 7, in Kitchen." then "Set 2 boxes along wall 7 ...from 2 ft to 6 ft". Undo: "Undone: Set 2 boxes along wall 7 in Kitchen: ..." one step.
- Price after the boxes: the big figure reads "no figure" in red with "11 not priced 3 lines unpriced"; open items "B18, cost code: unknown, and 2 more for the same reason". True to the rules (no prices in the starter) but the builder is not told where to put a price from the cabinet screen.

### J161-9 (cosmetic, confirms builder) Labels on the boxes
"Dishwasher" is wider than its box and runs over the neighbouring SB36 and B18 labels; at the plan's zoomed-out scale (needed to reach an addition) the "12' 0"" dimension label sits on top of the B24 box (p11/06-model.png). Countertop has no outline of its own.

### J161-10 (to check) corner rule
In p11 the first run was on the east wall's SOUTH end; the second run was on the north wall near the NORTH-east corner, where no run stands; the window still said "the run starts 2' 0" along wall 7". TRY-IT says 24 in only for a corner another run fills. See session E for the same click with no run in the job.

### J161-10 settled (confusing): start offset differs by wall and is unexplained
With NO run anywhere in the job (fresh session after Starter items), clicking the north wall's face near its east end said: "Cabinets: the run starts 2' 0" along wall 7, in Kitchen." while the east wall said 1' 0". TRY-IT says 24 in only for "a corner another run fills". Nothing on screen says why this wall starts at 2 ft (t10/said-a.txt of session s12). The field "starts at" can be typed over; not tried. Cosmetic/confusing, the figure is honest.

## Session E: Thing 5 (layouts), first half (pictures p13, p14)
- Held up: Layouts tab explains itself in a heading: "Try a job two ways and set the layouts side by side. Keep this job as another layout, change either one, and set them side by side: each is a job kept in this job's folder. Rooms are matched by name. Setting side by side writes nothing; keeping a layout writes only the new one." (p13/05-sheet.png). Esc at the title: "Cancelled." Title cleared then Enter: "Can't keep this as another layout: it needs a title: type one, then Enter". Enter with the offered title: "Kept a copy of this job as Kitchen job (not real) B, beside it, and set it beside this one: the table is on the sheet, under Layouts. Open another layout opens it to be changed; this job's own file is as it was." Keeping again under the same offered title: "Can't keep this as another layout: a job is kept here under that title already (kitchen-job-not-real-b). Give this layout another title". Table readable, "same" in the third column.
- Held up: with the copy open and a change made (Starter items), Open another layout shows the heading "PICK THE JOB TO OPEN, FROM THIS JOB'S FOLDER" and in orange "This job has changes that are not saved: Save first (Ctrl+S), then pick." and the status bar goes red "Save first (Ctrl+S)." (p14/04-sheet.png). A file that is no job (notes) is not listed.

### J161-11 (confusing) Unsaved: the rows look like buttons and are not
In that state the jobs are listed in the same dark type as a live list, with no greyed look (p14/04-sheet.png); my script could not click them (they are not on the map) - see session F for what a real click does.

### J161-12 (cosmetic/wording) File stems on screen
The layout list shows each job by title and, in grey, its file name without ending ("addition", "other-house", "kitchen-job-not-real-b" in the refusal and the beside-line) and a tag "job". These are file names, not a path or ending; a builder knows his file names, but "kitchen-job-not-real-b" in a refusal is program-made text.

## Session F: Thing 5 second half (p15c)
- Held up: with changes unsaved, a click on a job row in "Open another layout": status "Save first (Ctrl+S)."; after saving, picking the first layout: "Opened Kitchen job (not real). The other layouts are as they were; set one beside this to compare."; "Set beside another layout" then the copy: "Set Kitchen job (not real) B beside this job: the table is on the sheet, under Layouts. Nothing is written to either file." Table readable (p15c/05-sheet.png): rooms same, job cost "no figure ... not worked".
- Harness note: xdotool "ctrl+s" in one burst did NOT save at one frame a second (no "Saved." said, three tries); held for 2 s each key it saved ("Saved."). Retried; I count this as the slow renderer, not a defect, but a builder on a slow machine could see the same.
- Not tested: making the addition longer in the second layout (I did not find in this run a screen way to give a wall a new length; changed Starter items instead), so the money half of Thing 5 (a cost difference between layouts) was not seen: both columns read "no figure" since nothing is priced. Also not tested: two layouts with the same title (the window refused the same offered title).

## Words check
Scanned said-lines: no ".ply", "tmp", axis names or paths anywhere said or on screen. Seen program-ish words: "two things in one place cannot go by one name" (J161-2), "Study was placed first" (J161-6), "A reading came in with no value picked to take it" (after typing a door's sizes, J161 door note), file stems like kitchen-job-not-real-b (J161-12), "wall 1/wall 6" numbering that is the program's. "Existing, shown pale 0" counts shown as 0 when only a demolition is marked: seen in p9 ("Existing, shown pale 0").

## Builder's own "not right" list, confirmed or not
- Scan card still seven lines; after stand-up nothing says it was turned: CONFIRMED (p4/03, said-04).
- "Existing, shown pale 0" with count 0: CONFIRMED (p9/05).
- Receptacle count changes with a mark: CONFIRMED in numbers: Receptacles of Study 7 EA before, 6 EA after the east wall to demolish (p7 vs p8/07), and the sentence only gives the area.
- "Dishwasher" label wider than its box; countertop no outline; boxes plain tan: CONFIRMED (p10/07, p11/06; 3D not looked at closely).
- Layouts tab open items under the table and long: CONFIRMED (p13/05: 14 open items under the table, long sentences repeated).
- List of jobs shows file name beside title where titles differ: CONFIRMED (p14/04 shows "addition" and "job" beside every title).
- "tool chest" vs "this job's items": B99 refusal says "this job's items" (not "tool chest"): consistent with the window's heading Items.

## Findings by severity (summary)
| id | severity | what |
|---|---|---|
| J161-5 | cannot do it as written | no multi-select; "select the walls and press Existing" |
| J161-1 / J161-4 | confusing | wrong unit: refusal "nothing to draw" or a 79 ft house with no cue |
| J161-6 | confusing | Kitchen "no area yet... Study was placed first" with no next step |
| J161-7 | wrong result (message false) | a Room line that parts nothing is accepted and said to part two rooms |
| J161-10 | confusing | run starts 2 ft on one wall, 1 ft on another, with no run in the corner |
| door note | confusing | "A reading came in with no value picked" after typing door sizes |
| J161-2, 11, 12 | wording | program talk in refusals, rows that look live, file stems |
| J161-3, 8, 9 | cosmetic | card over plan, label collisions, Dishwasher label overflow |
