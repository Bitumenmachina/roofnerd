# W-169 report (call 169, the window). Nothing committed or pushed.

## 1. The thirteen items: built, and the step that proves each
1. Several at once: built. renow.sh second session (Shift+click, "Select all on this level", Existing/Delete on all, one Undo each; east-wall blue pixels 0 vs 240).
2. Scan in wrong measure: built. scanw.sh steps 11-15 (wrong-measure refusal, 197 ft warning sentence first, same scan twice, "Stood up.", every scan ends with the sentence).
3. Room with no area: built (model sentence, world building.rs). roomlinew.sh step 05 (Den); roomline.sh; two world tests.
4. Room line parts nothing: built. roomlinew.sh step 06 (stub line), then Undo.
5. Run-start hint: built (wording only). cabinetsw.sh says-checks "the run starts where you click, 1' 0\" along" and "type another start in Starts at." The hint on the tool bar is seen on a picture, not measured.
6. Laser message after tool sizes: built, NOT proved by any demonstration step. I could not reproduce the judge's keystroke timing.
7. Card put away: built. scanw.sh: map has "Put the message away"; mark press and Esc each take it off the map.
8. Plan marks DW/REF/RNG/WASH/DRY: built. cabinetsw.sh DW check. The scaling of too-wide labels (Fit/place_labels) is built but NOT proved by any step: no label in any run is wider than its box.
9. Words: built. layoutsw.sh (grey list, stem only where titles alike, picture measure); renow.sh (Key Existing/Show Demolition from the map; receptacles said "needs 7 receptacles (was 5)").
10. Typed measure: built for "Add cost line" only ("60 SF" in Priced by; renow.sh checks said text and the saved job). NOT built for the quantity of an existing line in the sheet.
11. Wall the work opens: built. renow.sh (Bearing, In the wall typed, caution in saved `show`, ~ Assumed marks). Cannot be made Known from the window (needs a model change).
12. Take line off: built. renow.sh (said text, Undo, saved job).
13. Tool chest renames: built. itemlibw, turnw, layoutsw click the new names; cabinetsw says-check "not in the tool chest" and forbids "this job's items".

## 2. Builds
- Every final passing run used bin2 (w169/bin2). bin1 was used only for debugging runs and was deleted.
- bin2 was copied from target/debug right after the last source edit (main.rs in_the_wall fix) and its build; the tests ran on that same build. No source file was edited after bin2 was frozen: only demos/*.sh and demos/captured were changed afterwards. So the tree's source is what bin2 was built from.
- Final passing run per demo, all on bin2: renow (run4), scanw (run2), roomlinew (run3, after I moved two shots), cabinetsw, layoutsw, itemlibw, turnw, moneyw, bringw, firstw (the single batch), roomline.sh (the same batch).
- Caveat: itemlibw, turnw, moneyw, bringw, firstw, cabinetsw and layoutsw ran in that batch, before I fixed scanw's sentence-first check and renow's checks; those fixes did not touch those scripts. The scripts of the batch were not edited again.

## 3. Pictures in demos/captured/
- Copied from the final runs, after those runs ended: renow (including multi/, from run4, not from an earlier run), scanw (run2), roomlinew (run3), cabinetsw, layoutsw, itemlib, turnw, money, bring, first (batch).
- Not retaken: every other folder. Tool chest renames change the left panel in other pictures (items, tools, layers etc.); itemsw.sh and the rest were not run. Their pictures are stale in that panel.
- w169/cap-renow still holds the run4 pictures as well; the tree copy is the same.

## 4. Expectations moved
- cabinetsw.sh: "the window said tool chest" failure became a failure on "this job's items" (item 13 reverses call 163's stand-in); B99 refusal now "not in the tool chest."; run-start says-check changed (item 5).
- itemlibw.sh, turnw.sh: "the job's items" -> "the job's tool chest" in the written-out message; control names renamed. layoutsw.sh: "Starter tool chest".
- roomline.sh and world tests (two assertions): old "was placed first and has the space this room's point is in" -> the new item 3 sentence.
- roomlinew.sh: new steps added after step 04; picture numbering of 01-04 unchanged.
- No test or check was deleted, skipped or loosened.

## 5. New picture checks (red/green)
- renow.sh: blue pixels (blue-red over 80) in a 26x60 window on the east wall: unpicked picture 01 = 0 (red: check fails), picked picture 02 = 240 (green).
- cabinetsw.sh: light pixels (lum over 180) over the countertop edge strip 33.6-33.8 ft east, 3.2-3.8 north: new 05 = 0 (green); committed pre-change 05 = 47 (red). Also 11 dark pixels of DW inside the box (wanted at least 6).
- layoutsw.sh: title darkest pixel 25 (07, nothing unsaved) vs 133 (09, unsaved); stem ink 0 for the alone title vs 121 for alike titles. Pre-change picture: 25 and 161, both fail (red). (The red figures for layoutsw and cabinetsw were measured by hand on the committed pictures with approximate row/plan positions, not by the script.)

## 6. Choices the decision did not make
- A multi-mark or multi-delete is one Undo step; multi "Existing" with a costed wall asks once for all; Select all picks the same kind on that level.
- Said on screen, bearing and in-the-wall are Assumed, "entered on screen".
- Typed measure on a labor line is accepted with its rate.
- Esc order: with something selected Esc deselects; only with nothing selected and no tool does it put the card away.
- The card mark is the character x (U+00D7); named "Put the message away".
- Item 5 wording only; I did not change the rounding to "nearest foot".
- The per-button "acts on this one only" notice is a static line in the panel, not a message at press time.
- Wording I picked: "Marked N things: ...", "Selected all N walls on Ground...", "needs N receptacles (was M)", "Took ... off ...".

## 7. What is not right, bluntly
- Item 6 is unproved. Item 8's scaling is unproved and unseen. Item 10 on existing lines is not built. Bearing/in-the-wall cannot be made Known in the window.
- scanw/13: card is ten lines and says the sentence twice. scanw/15: plan not reframed after the large scan.
- roomlinew/05: Den's open items repeat the long sentence five times (the model's note, not changed).
- layoutsw/04: headings under alike titles show ".json" file endings, against the no-file-ending rule; I did not change them (call 157 expectation).
- Worked round: renow's first drafts failed for my own script mistakes (same wall shift-clicked twice, `on` instead of `on_right`, "Ground" capital, saved `show` has no source text). The scanw check had to read the said log as one " | "-joined line. cabinetsw's first DW window measured the countertop's own dark edge; I changed the window, then proved it red on the committed picture.
- Pictures of folders not rerun are stale in the left panel (tool chest names).
- The layoutsw/cabinetsw red numbers were computed on the committed pictures by hand, not by the demonstration.
- The handback tool refused (not in auto mode), so this file is the report.
