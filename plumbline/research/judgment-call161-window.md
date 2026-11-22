# Judgment of the window against TRY-IT.md section 3 (call 161), written cold

| # | Thing | Verdict |
|---|---|---|
| 1 | Bring your scan in | (pending) |
| 2 | Trace the house as it stands | (pending) |
| 3 | Take down and add on | (pending) |
| 4 | Set a kitchen | (pending) |
| 5 | Try it two ways | (pending) |

Findings follow, appended after each session.

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
