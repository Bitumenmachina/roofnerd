# Report W-197W (window builder, call 197 points 10 to 18)

Started on eeca2b7, clean tree. Frozen baseline SCRATCH/f11. First build 46 s.

## Built so far (code; not yet run in the window)
- 11: Cabinets card: left column 96 -> 140 px (so "Marks on the plan" no longer runs into the next column); card carries a name ("Cabinets card") and the map writes `box<TAB>Cabinets card<TAB>l<TAB>t<TAB>r<TAB>b`; a new system `scene::room_for_card` eases the plan up/out when the tool is taken in hand on the plan so the building stands in the band above the card; boxes typed with the tool in hand and no run started say "No run is started yet ... Click the wall's face where the run starts ...".
- 12: a wall/run corner refused says "The run of walls still starts at (x, y): click the next corner, or press Esc to let it go."
- 13: `Doc::left_unmarked`: selecting something else (select, shift-select, select all on a level) with a mark question pending says "<name> was not marked <status>: its question was left unanswered, and nothing was changed."
- 15: maker's sheet with no trade: "Its trade is not said yet: pick it under Trade, on the right, with the item selected."
- 16: both `change_assembly` calls in main.rs now `change_assembly_said`; the window's message appends the rewritten/left lines in the words `plumbline apply --change` uses.
- 10: sheet window: each level's Building table lists set-down things by kind with their names ("Set down on this level"), and the Estimate says "N furnishings are set down and not priced." (empty estimate and under the Price).
- 17: `tools::item_size` now calls `editing::item_size`; the window's own set-back-to-Assumed and mending of followers are gone; ITEM_TYPED and TYPED are `ed::ITEM_SIZE_TYPED`.

## Second interim (after the lead's ruling)
- Point 10 ruling carried out: a plain comment at `sheet.rs set_down` says the count is owed to the model. NOT RIGHT: the per-level count is the window grouping the model's placements; a set-down count by level and kind in the storey's totals is owed.
- f12 deleted; f13 frozen from the tree (SCRATCH/f13). Session c of fixes197w.sh started on f13 (log SCRATCH/w197W/rep13-c.log, temp kept in SCRATCH/w197W/keep13) when the lead asked for a handback.
- Run so far: f11 session c reproduced point 11 (south wall under the card, "MARKS ON THE PLAN" running into "DW"; SCRATCH/w197W/rep11/c/02-model.png). Nothing else run. Steps 2 to 7 of the lead's list are not done.
- session c on f13: PASS (card clear of all four walls, boxes-before-click, run 6.5 ft/13.81 SF, corner refused, mark left, maker's sheet trade). Pictures SCRATCH/w197W/rep13/c
- session a (first two tries) stopped: the sheet has no field for the price of a line made of a material (price is the price list's); changed the demonstration's price step to Bring in, Price list with a one-row list. To report: the contract's 'price typed on one of them' has no window path today.
- RULING case: the window has NO way to type a material's price (only Bring in, Price list; the Material list only picks a material for a line). So: no price field built; the demonstration prices the heater by a one-row price list brought in. NOT RIGHT (plain): a price for an item cannot be typed in the window and comes only from a price list brought in.
- session a on f13: PASS (Building list 'Set down on Ground: 2 furnishings'; Estimate says 2 furnishings are set down and not priced; heater Priced as list; 'is now priced as Bought and set; 2 set down from it follow'; four lines; $3,700.00 after a one-row price list). Pictures SCRATCH/w197W/rep13/a (04 the panel, 02-sheet the Building list; both read plainly).
- session b on f13: PASS (bench: priced as Bought and set, 2 follow; vanity width 48 in said in the model's item_size words, name still reads 36 in). Picture b/02-model: heater alone on its floor, panel Priced as row twice (the model's 'priced as' row and the new section: slightly redundant).
- session d on f13: PASS (200.00 to 140.00 SF x2, corner boards 20.00 to 14.00 LF, $560.00)
- session d2 on f13: PASS (180.00 SF left as entered, said in the window's words: 'net area * 0.9'; corner boards 20.00 to 14.00). d: PASS.
- session e on f13: PASS (quarter turn from the ground: house middle stays (800,454) of 1600x960; east gable end in view, picture e/02-model; corner board stays 2.5 dots; the first try used a pointer place over the tool bar and turned nothing: caught, moved to (1150,700), and the script now fails if the foot of the house moves under 150 dots).
- Google Earth: could not establish from its help pages (support.google.com/earth/answer/148186): they give the mouse gestures (left-drag, Shift+left-drag to tilt) and do not say what point the view turns about. Not written from memory.
- benchw on f13: PASS first run (its right-drag on the bench included).
- dressw on f13: PASS first run. Point 14: no existing check failed; none edited. (Only benchw and dressw right-drag.)
- cabinetsw on f13: FAILED twice (same six pixel checks: codes at each box's middle, DW inside its box, dimension gone from over the run). Cause: the script reads the plan's zero and scale ONCE (plan_line, before Cabinets is taken in hand) and measures the pictures against it; my room_for_card eases the plan (out and up) when the tool is taken in hand, so every place it samples has moved. Everything else it checks passed (the run, 13 ft, 27.625 SF, the same as the CLI, B99 refusal). NOT EDITED. It wanted: the plan where it was at open (23.95 dots to the foot); it gets: the plan eased so the 36 ft building clears the card (the picture 05 is right and reads well: boxes carry their codes). Ruling needed: let the script read plan_line again after the tool is in hand (one line), or the view not move for a building that is wide.
- movew on f13: PASS first run.
- tracew on f13: PASS first run.
- cabinetsw on f13 after the lead's ruling (plan zero and scale read again after the ease, per picture; bounds unchanged; one comment beside it): PASS, 0 FAIL, first run after the edit. plan zero (369,672) as opened -> (369,584) with the tool in hand, same scale.
- Esc after Cabinets (fixes197w c): the plan does NOT ease back; zero (451,729) as opened -> (453,640) with the tool in hand -> unchanged after Esc: the view is left moved (NOT RIGHT, small; nothing built).
- glassw on f13: PASS first run (including 'a turn is about the pointer').
- house on f13: PASS first run.
- tradesw on f13: PASS first run.
- itemlibw on f13: PASS first run.
- pricew on f13: PASS first run.
- tests: app 44, world 542, plans 172, ifc 43, sheets 6, roofing 122, cli 22 = 951 passed, 0 failed. gates language/egress/clientdata PASS; licenses.sh exit 0.

## Final
- Trades pictures: no script writes the named set demos/captured/trades/1-foundation.png ... 7-waterproofing.png (git log -S finds none in demos). demos/tradesw.sh writes numbered pictures 01 to 17 (the trades' buttons) into its argument folder; run on f13 into SCRATCH/w197W/trades13 (PASS). I did not copy or invent the named set. 12-model.png there (the Outside button) reads as the sample house with lap siding over stone, trim and windows: current.
- Pictures of fixes197w kept in plumbline/demos/captured/fixes197w/{a,b,c,d,d2,e}.
