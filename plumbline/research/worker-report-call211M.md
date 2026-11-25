# W-211M report (contract 211M: where an item connects, the model's side)

Working copy: SCRATCH/wtB (detached at 5b8a1af).
Baseline (made BEFORE any change, with the program built from 5b8a1af; copy of that program kept as SCRATCH/w211/plumbline-before):
- `plumbline sixteen` -> SCRATCH/w211/sixteen-before.json, sha256 880d9fd3c364b5f77ad2abc666dde290955efd6a971f748f65fab8e00292b82a
- a job with an item and nothing else (new job + one imported item row "Test lavatory", 24 x 18 x 34 in): SCRATCH/w211/item-before.json, sha256 80ea1eb067528a84e0bf4d6554bb17456e99411ff713af4e251ebfd7865cb18f
- the same with the starter chest brought in: SCRATCH/w211/starter-before.json, sha256 7ef06ed39b9c42c03d9b1f02e592c52ba0a438086794ead3b5189b6d022820d7

## BEFORE ANY CODE: the four answers

### 1. BIMCraft (read: family.rs 340-500 and 1353-1440, model.rs 3313-3331, fixtures.rs, connector_tests.rs, family.rs 2095-2200)

Ported (the ideas and the facts, not the code; BIMCraft's Connector is a formula-of-parameters in millimetres with a direction vector):
- Which connections each starter fixture has, which way each goes (`flow`), and which way it faces (`direction`, a vector there; ported as the six words a tradesman says):
  - WC (family.rs:2127): waste, out, 100 mm, [0,0,-1] = faces down, at (0,-300,0); cold, in, 20 mm, [0,1,0] = faces back (toward the wall), at (150,0,200).
  - Washbasin (2149): waste, out, 40 mm, faces back, at (0,0,height-300); cold, in, 20 mm, faces back, at (-100,0,height-200).
  - Bathtub (2171): waste, out, 40 mm, faces down, at (-length/2+150,-width/2,0); cold, in, 20 mm and hot, in, 20 mm, face back, at (-length/2+100 / +250, 0, height+200).
  - Kitchen sink (fixtures.rs:134): waste, out, 50 mm, faces back, at (tap_x,0,height-400); cold, in, 20 mm, faces back, at (tap_x-150,0,height-300). (Shower and floor drain are BIMCraft's too; the contract names four.)
- The rules that apply here (`check_connectors`, family.rs:1353): only a pipe carries a domain (already ported, `check_connectors` in model.rs); a pipe connection needs its place, its size and what it carries; size positive and bounded; a bound on how many (8 there, 64 here, call 211 point 3); a refused change leaves the stored thing alone (the pattern calls 205 and 203 took).
- The turn (`component_connectors`, model.rs:3313): origin = the thing's place plus its angle; `(x cos - y sin, x sin + y cos)`, z from the base; the direction is turned the same way and not moved. Ours is the same sum already in network.rs:243.
- The tests' ideas (connector_tests.rs): a connection stands where its thing does and faces with it (place, direction, size, turned and raised); an unsound connection is refused and says what to fix; the starter fixtures carry their waste and water; a waste through the floor is under the fixture and faces down for a turned fixture; a thing with pipe connections is no electrical device in IFC; saved and opened as it was.
What is ours (not in BIMCraft): rows on an ITEM (BIMCraft's belong to a family), the table of them and its in/out, the verbs, the set-down/follow/joined rules, the clash words, Assumed status on every place, no formula over parameters but width/depth/height only, a facing kept as six words and not a vector, the box rule. BIMCraft has no clash and never draws or joins a pipe to a connector (survey C3, confirmed on a fresh grep). The starter fixtures' PLACES are BIMCraft's mm figures (e.g. WC cold at 150 mm across, 200 mm up); they are NOT used as the starter rows' places: the contract asks for US rough-in sources instead (research/connection-places.md).
New public items go in PORTMAP.md (each with its search); `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` passes on 5b8a1af (383 items, 142 ported). NOTICE: a ported idea (the six directions from `direction`, the check rules) is credited in the existing BIMCraft notice; I add a line naming the connection facts and the file/line.

### 2. The demonstration's steps 2 and 6, by the program's own code

The code that turns a shape's block into a place on a thing set down:
- model: `building/shape.rs` `run_one`: `let y = |b: f64| size[1] / 2.0 - b;` (a figure "back" is from the back toward the front; the thing's own y is front less, back more, from the middle; across is x from the middle; up is z from the foot).
- the window (scene.rs:1232-1240): `at = centre + turn * Vec3(mid[0], mid[2]-size[2]/2, -mid[1])`, `turn = rotation_y(angle)`: east = x, view z = south = -y, counter-clockwise from east. In world terms that is `x_w = at.x + x*cos(a) - y*sin(a)`, `y_w = at.y + x*sin(a) + y*cos(a)`, z = floor + offset + up.
- a connection's place today: `network.rs:243`: `[origin[0] + q[0]*cos - q[1]*sin, origin[1] + q[0]*sin + q[1]*cos, origin[2] + q[2]]` with q in the thing's own axes ("x along its angle, y to its left, z up from its base"), and `join.rs` `connect` takes it back with the inverse turn. These are the SAME frame as the block's (the thing's +y is its back; at angle 0 the back is north). So the connection's local y is `size[1]/2 - back`, exactly the block's. I will make that one public function in shape.rs (`thing_axes`) and have `run_one` call it (no change to any figure; the 13 starter shapes' golden test is re-run).

Step 2 (middle at (10,20), not turned, floor 0, item 24 x 18 x 34 in): across -4 in = -1/3 ft; back 0 -> y = 9 in - 0 = 0.75 ft; up = 34-12 = 22 in = 1.8333 ft.
  cold (10 - 0.3333, 20 + 0.75, 1.8333) = (9.6667, 20.75, 1.8333); hot (10.3333, 20.75, 1.8333); waste up 34-16 = 18 in = 1.5 -> (10, 20.75, 1.5).  **The lead's step 2 figures are right.**
Step 6 (turned 90 degrees counter-clockwise, `turn()` is counter-clockwise from east; the lead's "quarter turn"): local (x, y) = (-1/3, 0.75): world x = 10 + (-1/3*0 - 0.75*1) = 9.25; y = 20 + (-1/3*1 + 0.75*0) = 19.6667; z unchanged.
  cold (9.25, 19.6667, 1.8333); hot (x,y)=(1/3,0.75): (10 - 0.75, 20 + 0.3333) = (9.25, 20.3333, 1.8333); waste (0,0.75): (9.25, 20.0, 1.5).  **The lead's cold figure is right**, and a quarter turn the other way (-90) puts cold at (10.75, 20.3333, 1.8333).
Where I differ from the lead (set out in the demonstration's own words; the program is built on its own way):
  (a) Joined is NOT "within 0.05 ft": the network's own tolerance, the one `takeoff`, the clash excuse and the IFC writer all use, is `FIT_TOLERANCE_FT` = 0.01 ft (about 1/8 in) (`building/network.rs:TOLERANCE_FT`, `check.rs:17`). 0.05 ft is only how far a model-brought-in's TIE may be from a run end (`ifc/src/lib.rs:2499`), and there the connection is then PLACED at the run's end, so the two are 0 apart. A second tolerance would be a second way of knowing. Built on 0.01 ft; the contract's figure is corrected here. The demonstration's own figures (3 in, 2 in) are unaffected.
  (b) "held at the sizes a shape is held at" (the three check sizes, 4.8 x 3.6 x 6 in ... 6.5 x 3 x 2.6 ft): the lead's own sample rows `Up = height - 12 in` and `height - 16 in` reach 6 in BELOW the foot of the smallest check size (height 6 in), so a table of the demonstration's own rows would be refused at the small size. A connection is held to the box at the item's OWN size (and whenever the item's size changes), which is the only size its place means. The three-size check is not applied to connections. (Ruling point: say if you want it.)
  (c) The lead's step 3 says the pipe and lavatory "are not a clash": true by the excuse (a joint is cut) - to be shown by the run, not assumed.
Everything else in steps 2 to 6 agrees with the program's own figures (see the demonstration's run below).

### 3. How a run is known to be joined to a connection today

By where its end stands; nothing is stored. `network.rs:network()`: every thing's connector is put in the list of points at `origin + turn(at)` (line 243), every run end is a point, and points of one kind within tolerance are one node:
```
if points[j].2 == *kind && dist(points[j].0, *p) <= tol { sets.join(i, j); }      // network.rs ~263
```
with `tol = tolerance.clamp(..)`, `TOLERANCE_FT = FIT_TOLERANCE_FT = 0.01`. A node that has a `Joint::Connector` and a `Joint::RunEnd` is a joint; a node with one joint is open (`open = ... nodes[n].joints.len() == 1`). `takeoff` names the open ones ("lavatory 1: its Waste connection", building.rs:3608-3625). The clash excuse builds `joined` pairs from the same nodes (`clash.rs:817-827`) and cuts the run back there (`away_from_joints`). `connect` (join.rs:484) does not store a link either: it gives the thing a connector WHERE THE RUN ENDS, so that the position joins them.
So under point 6 "joined" is the same node: a connection is joined when the network says the node it is in has a run end, i.e. the run's end stands at its point within 0.01 ft. I add no second test: a function `connection_state` asks `network::network` for the nodes and reads whether each of a thing's connectors is in a node that has a RunEnd. `connect`'s refusal, the follow's "stands off" count, the clash words and the tests all use it.
Nothing is stored on the run or the connector to say "joined".

### 4. Every place a Connector or an Item is written or read, and what each needs

CONNECTOR (model.rs `Connector`, `Connectors`):
- `file.rs` K_CONNECTORS put! (write) and read+`check_connectors` -> two new keys on a connector (`facing`, `of_item`) are skipped when absent; nothing else. SPEC.md row.
- `join.rs`: `connect` (524-545), `connect_port` (608-635), `fit_at` (380) build Connector literals -> add `facing: None, of_item: None`. `connect` also refuses (point 6) and names a connection already there from an item.
- `sample.rs` 417-418, 559-562: literals -> `None, None`; the sample house is not set down from items and does not move (held by test + step 10).
- `measure.rs:521`: edits "connection N" position/size by hand -> unchanged. (A measured place makes the connection Known by the existing rule; a follow later must not move a Known place the person measured: I leave a connection whose place status is Known alone and say so.)
- `network.rs` reads each (`at_ft`, kind, domain): unchanged; this is where joined is known.
- `building.rs:3074` (the show rows "connection N: name (kind, domain, flow)"), building.rs:3608-3625 (takeoff's open ends by name): names unchanged; the row text gains the facing only where there is one.
- `clash.rs`: through the network; the words are added by a model function the CLI calls (point 7).
- `ifc/write.rs` 660-740 writes each as an IfcDistributionPort (place only; the axis is fixed [0,0,1]); `ifc/lib.rs` 2450-2530 reads ports through `connect_port` (no facing). **A facing is NOT written to the BIM file and is not read from it**: I will not change the port's axis (a quiet change to what every existing file says); the export's own "not written" list gains one line, only where a connection has a facing, so the loss is reported (call 211 asks "loss is reported, never silent"). Say if you want the axis written instead.
- `tests/world.rs` 2898, 3120, 3129: three struct literals `Connector { name, kind, domain, flow, at, size }` do not compile with two new fields. This is a compile fix with no expectation touched; see "Stopped on" below.
- `takeoff`, the Connections panel (app, not touched), `vif.rs` (none: point 10 says open connections are NOT VIF lines).
ITEM (model.rs `Item`, check_item, size_ft):
- The row store is NOT a field of `Item`: eleven struct literals in tests and two in the window (`tools.rs:518`, `bench.rs:248`) build it, and the window may not be touched. Rows are a component of their own on the item's element (`pl::item_connections`), the same way a thing's connectors are.
- `file.rs`: K_ITEM_CONNECTIONS put!/read; KNOWN_KEYS (the schema entry only where some item has rows, as `pl::hidden`: so a job with none is byte for byte what it was).
- `tables.rs`: `items_out`/`items_in`/`items_in_missing` unchanged (a chest that lacks rows changes nothing); a new pair `connections_out`/`connections_in` (beside `shapes_out`/`shapes_in`), `library/connections.csv` (`STARTER_CONNECTIONS`), and the starter items' rows are brought in by `import connections <job> starter`.
- `editing.rs`: `place_item_in` (gives the thing its connections), `placed_follow` (works them again), `item_size`/`measure.rs:401` (the follow after a size; the box rule for a changed size), `rename_item_inner` (rows stay on the item, nothing to do), `item_drawn_as`, `item_trade`, `item_priced_as`, `price_item` (nothing to do), `cabinets` (sets items down with `place_item_in`: gets whatever rows its items have, none today), `remove` (an item removed: things set down keep their connections, their `of_item` then names a row nobody follows: said by `check`? not handled; I will note).
- `editing/shapes.rs` `item_fits`/`items_outside_their_boxes` pattern -> `connections_outside_their_boxes` for `check`.
- `spec.rs:497` makes an item from a maker's sheet (literal): no rows (a maker's sheet's connections are step E; not here).
- `cli`: `items_said`, `item_blocks_said`, `set ... priced_as`, `rename`, `price-item`, `place`, `connect`, `check` -> `items` counts rows; `items <item>` lists them; `connect` message; new verbs `connection`, `import/export connections`, and `turn` (the demonstration needs a thing turned from the command line; there is no verb for it today, only the window calls `editing::turn`).
- `vif.rs:317`, `view.rs:370,1393`, `pricelist.rs:197`, `derive.rs:252`, `building.rs:2942,3334`, `job.rs:1037`: read an Item for its price, trade or form: not affected.
- `app/*`: not touched (bench.rs, tools.rs, panels.rs, scene.rs read Item and Connectors); the window's half is a later job. A thing set down from the window's tools goes through `place_item_in` and gets its connections with no change there.
A place found and not handled: the window; an item REMOVED while things set down from it keep connections whose `of_item` names a row that is gone (they stay as they are, which is the rule "left alone"), `check` does not say so.

(Work notes follow below.)

## Work notes (in order)

### Tests first, run red
`cargo test -p plumbline-world --test call211m` before any code (file crates/world/tests/call211m.rs, 19 tests + fixture call211m_item_before.json = the job `plumbline new` + one `import items` row written by the program built from 5b8a1af; SCRATCH/w211/item-before.json, sha256 80ea1eb0...):
```
error[E0433]: cannot find type `Facing` in this scope            (11)
error[E0425]: cannot find function `connection_row` in module `ed`   (10)
error[E0425]: cannot find function `connections_standing` in module `ed`  (6)
error[E0425]: cannot find function `connection_add` in module `ed`    (6)
error[E0425]: cannot find function `connections_out` in module `tables`   (5)
...
error: could not compile `plumbline-world` (test "call211m") due to 60 previous errors
```
After the code, first green run: 13 passed, 6 failed. Of the six, four were mistakes in my test setups (not the program), found and said:
 - the one-arithmetic test used a fixed `-4 in` in a shape, which a shape refuses at its small check size (the check sizes again: a shape's check is not for a connection);
 - **a run drawn from the back of the lavatory THROUGH the hot connection to a point 3 in short of... no: past it** (a pipe from y=24 to y=20.5 passes over the hot connection at y=20.75). The network counts a run that passes over a connection's place as joined mid-run (a `RunSide` joint, "joined_mid_run" warning), so `takeoff` does not name that connection open. That is the network as it was (not changed, one way of knowing). The lead's step 4 does not say from which side the hot pipe is drawn; drawn from the FRONT (from y = 17 to y = 20.5) the 3 in short of the connection is as the lead says. I draw it from the front and say so. (A run passing over a connection is a node that splits it: not made worse or better here. Flagged: "joined" in the network is broader than "a run's end stands at the point"; my `connections_standing` reads run ENDS only, so for a connection a run passes over, `takeoff` says joined-mid-run and my `joined_to` says open. Point 6 says joined means the run's end stands there: I follow point 6 for connect's refusal and the stands-off count, and leave the network alone. Say if you want the two the same.)
 - `connect` of a run-made connection on a thing from an item is now refused unless the run ends at an item connection (point 6): the test now makes its own connection with a duct.
 - a table with an unreadable row and a box fault in different rows says the unreadable one first.
Left red until the research lands: the starter rows' test.

### After the code
- `cargo test -p plumbline-world --test call211m`: 19 passed. Whole crates, `--no-fail-fast`, one at a time: plumbline-world 651, plumbline 28, plumbline-plans 172, plumbline-sheets 6, plumbline-ifc 69, plumbline-roofing 122 = 1,048 passed, 0 failed (SCRATCH/w211/test-*.txt). No existing test edited; the only change to an existing test file is the three `Connector { .. }` literals in crates/world/tests/world.rs (lines 2898, 3120, 3129) which gained `facing: None, of_item: None` so that they compile (no assert touched).
- `PLUMBLINE_BIN=.../target/debug demos/connections.sh`: all ten steps held (SCRATCH/w211/demo-run.txt, no FAIL). `plumbline sixteen` sha256 880d9fd3... as before; takeoff, estimate, clashes, vif and items output of the sixteen house identical to the before-program's (cmp). `items` of a starter chest gains exactly one line ("Where they connect: ...").
- `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh`: PASS (418 public items, 144 ported, 274 ours).

### Demonstrations, gates
All command-line demonstrations: 59 scripts run (every demos/*.sh not ending in w.sh, plus draw.sh), 56 pass with no FAIL, 3 are not command-line: d2.sh and house.sh drive the window (need plumbline-app, never built here) and field-job.sh is a helper that takes an argument. connections.sh is among the 56. Gates: gate-language, gate-egress, gate-clientdata --all, plumbline/tools/licenses.sh all pass. portmap PASS.
Patch: SCRATCH/review/call211M.patch.

## Round 2

Expectations that hold the old check-size sentence (`grep -rn "small size\|outside the box" crates demos`), listed BEFORE any was changed:
1. crates/world/tests/shapes_job.rs:174  `why.contains("small size (0.4 x 0.3 x 0.5 ft)") && why.contains("outside the box") && why.contains("across")`
2. crates/world/tests/call205.rs:109     `why.contains("small size (0.4 x 0.3 x 0.5 ft)") && why.contains("outside the box")` (and `"1 in above"` on the next line, which stays true)
3. demos/shapes.sh:180-181 (and its echo at 175)  `"small size (0.4 x 0.3 x 0.5 ft)"` and `"1 in above the box"`
No other test, demonstration or screen expects it (crates/app has none; research/*.md and STATUS.md name it as history only). The item-own-size sentence (`Shape::check_at`, "... whose block N reaches 24 in above its box (up)") is already in inches and is not touched.
All three move by exactly the words of the sentence.

Round 2 results
1. Joined: one answer, the network's node. `connections_standing` and the follow's stands-off count count a run's end AND a run passing over (RunSide); connect's refusal and the clash words read `connections_standing`. Test `round_2_joined_is_one_answer_for_the_open_list_and_the_function` holds takeoff's open list (by name) and the function to the same answer for: end at the point, run passing over, end 3 in off, nothing. (Red first: "a run passing over it: takeoff's open list and connections_standing differ"; and connect refused a pipe naming the Hot connection it passed over.) Edge left as is: two things' connectors at one point with no run are open in the function and not open in takeoff (a node with two joints); ruled "any run joint", not changed.
2. Rename: rows are on the item entity and `of_item` holds the ROW name, so a rename never touched it; what went stale was the source text ("the item Test lavatory, its ...") on each thing's connection. `rename_item_inner` now calls `follow`, which rewrites them with the new name (positions unchanged). Test red first (source still said Test lavatory), green after; a size change after the rename and a row changed by the new name still move the connections.
3. Delete: `editing::remove` REFUSES an item while anything is set down from it ("one is set down from type-NNNN: take it out of the job first"), so no connection is ever left that is nobody's. Tested (refused, job byte-identical; after the thing and then the item are removed nothing carries an `of_item`, `check` finds nothing outside a box, file reloads). My round 1 "not right" about a dangling `of_item` after deleting an item was wrong.
4. STATUS.md: my paragraph removed (git diff of STATUS.md and DECISIONS.md is empty).
5. Words (call 212): the check-size refusal now reads `shape "vanity with a panel", block 4: on an item 4 13/16 in wide, 3 5/8 in deep and 6 in high it would stand 1 in above the top; a shape is held inside its item at every size, so write the figure from the item's sizes` . Made in ONE place, `Shape::piece` (shape.rs); new `units::inches_said` (the program's own sixteenths writer, split out of `size_said` unchanged). The lead's example said 4 3/4 in: the program's writer gives 4.8 in as 4 13/16 (sixteenths), I used the program's. Six ways: above the top, below the foot, past the left side, past the right side, in front of the front, behind the back. Expectations changed (listed before changing): shapes_job.rs:174, call205.rs:109, demos/shapes.sh:180-181 and its echo. No connection refusal speaks that way (they use the item's own size in inches).
Results: crate tests 655 + 28 + 172 + 6 + 69 + 122 = 1,052 passed, 0 failed; command-line demonstrations 56 run (d2, house, field-job skipped as before) all rc=0 no FAIL (SCRATCH/w211/demos2/summary.txt); gates, licenses, portmap pass. Patch rewritten.
