# W-196 report (contract 196: an item carries its cost, step B)

Working copy: scratchpad/wtB (87a2c41, call 199 in the tree). Notes written as I go.

## Before any code

### 1. BIMCraft, searched whole (`grep -rln -i "cost\|price\|schedule" crates apps`, ~60 files, and a second search for dollar/currency/money/unit_cost/unit price)
Nothing carries money. Every "cost" hit is compute cost ("what the cutting may still
cost", "costs nothing") and every "schedule" hit is its general table
(`schedule.rs`: rows grouped by a field, a `Field` may be a formula and may be
summed). A family (`family.rs:555`) carries `parameters` (kinds Length, Number, Angle,
YesNo, Material: a parameter may name a material, never a price) and `types`
(values by parameter name). So a price could be a user number on a family type, but
nothing in BIMCraft types it, prices it, or ties it to a supplier. Nothing to port.
Recorded in PORTMAP.md as "ours" rows for each new public item, with the greps.

### 2. The design beside the code, point by point
1. **One home for a price.** CAN, with a correction to how it is read. The price list is
   the job's Material elements (`Supply`: `price` and `quotes` by supplier). It finds a
   price by a *material's name* only (`prices_in` matches `Kind::Material` by
   `check::plain_name`). A price by a *line's name* is a different road: `price_item`
   (call 157) writes `unit_cost` on each thing's own line named for the item; it is
   what a cabinet box uses and it is not the price list. An item's line uses the
   material road (the only one with three suppliers side by side).
2. **Setting down applies the assembly.** CAN. `place_item` (the public wrapper) calls
   `place_item_in`, which the cabinet run (`editing.rs:910`) also calls and then adds its
   own line named for the item; so the assembly must be applied in `place_item`, not
   in `place_item_in`, or a box in a run would be priced twice. The estimate already
   says what is set down and not priced: `view::not_priced_said` ("nothing is priced on
   2 things that could be (2 pieces of equipment): they are in no line and no total
   above"), printed by `price`. It is NOT printed by `estimate` when the job has no
   cost line at all ("Nothing in this job has a cost line." and nothing else: seen),
   which is demonstration step 1; I add it there.
3. **A line "made of the element's material" on a thing set down is made of its item.**
   What it does today: the line is not refused; it is worked (quantity) and left
   without a figure, "the line is made of the element's material, and the element does
   not say what it is made of" (`lines::Stock::NotSaid`; seen with a heater and a hand
   `line --of-material`). The road: `derive.rs` builds a `Stock` from the element's
   `MadeOf`; with none it is `NotSaid`. I add one rule: no `MadeOf` and the element has
   `Placed.item` -> its material is the item's. Only an element with `Placed.item`
   reaches it; walls, floors and roofs have none, so no wall, floor or roof line is
   touched (a test says so, and every existing test is run unedited). No stored
   `MadeOf` on the thing, so a renamed item and a cleared "Priced as" cannot leave one
   stale. `view::estimate` needs the same fallback for its material column and price.
   **DOES NOT HOLD AS WRITTEN: "the name it is priced by in the price list is the
   item's".** A Material named like the item cannot be added: `Job::add` refuses two
   children of one parent with one name, and the item (a Type) and a Material are both
   children of the job: "there is already something called "Water heater, 50 gal" in
   the job: two things in one place cannot go by one name" (seen, from `material`,
   and from `import prices --add`). **HELD FOR A RULING.** The smallest true
   alternative, built behind ONE function (`editing::item_material_name`): the item's
   price-list name is the item's name and ", bought" ("Water heater, 50 gal, bought").
   The line on the thing is still named for the item. If the ruling is to let a Type
   and a Material share a name (a change to `Job::add`, to the file's load check and
   to every lookup by name: the lead's), that one function returns the item's own
   name and nothing else moves. Everything else below is built on the alternative.
4. **Three starter assemblies, "Bought and set".** CANNOT as three: an assembly is found
   by name (`apply_as`: "names 3 assemblies; use the path"), so three alike are
   unusable. "Goes on" is a list (`tables.rs:1124`), so ONE assembly "Bought and set"
   goes on Equipment, Fixture and Furnishing: the smallest true alternative, built.
5. **Things set down follow.** CAN, by the same road as sizes: `Placed.item` finds them
   (`placed_follow` lists the placed whose `item` is the path). It is not called by
   anything that changes assembly today; lines follow by `change_assembly_said`
   (call 199), with one wrinkle: lines are matched by description and the item's line
   is named for the item on the thing but for the assembly's own word on the assembly,
   so before a change the thing's item line is put back to the old assembly's word and
   after it is named for the item again. A thing in a cabinet run (it carries its own
   line named for the item) is not touched and is named. A thing whose assembly a
   person chose another is not touched and is named.
6. **Chest's table: Priced as, Unit cost (how sure, where from).** CAN. The unit cost is
   the Material's `Supply.price` (named per point 3); brought in it goes through
   `prices_in` (one door), with the row's own how-sure and where-from. The material
   is added if the job has none.
7. **Saved job gains one entry.** CAN (below).
8. Not in B. Not built.

### 3. The saved job
One entry on an item (`pl::item`), after `form`, left out when there is none:

    "priced_as": "Bought and set"

A job saved before has no such entry, opens as it did, every item priced as nothing
(`#[serde(default, skip_serializing_if = "Option::is_none")]`). A job saved after and
opened by an older program is refused (`deny_unknown_fields`): not promised, as call
196 says. SPEC.md `pl::item` row gains `priced_as` and what it is checked for (names an
assembly the job has, that goes on the item's kind).

### 4. The chest's table
Four columns after "Weight: where from", in this order (so every old column keeps its
place): `Priced as`, `Unit cost`, `Unit cost: how sure`, `Unit cost: where from`
(22 headings become 26). A table without them is read as it was: an item already in
the job keeps its "Priced as" and its price list is not touched. A table WITH the
column and an empty cell clears "Priced as". The starter `library/items.csv` is not
changed (its items name no assembly).

## Built (as it goes)

### Red runs, quoted
First run of `crates/world/tests/call196.rs` (17 tests) before any code, `cargo test -p plumbline-world --test call196`:
`error[E0425]: cannot find function \`item_priced_as\` in module \`ed\`` (15 times),
`cannot find function \`place_item_said\` in module \`ed\``, `cannot find function \`item_material_name\` in module \`ed\``
(and no field `priced_as` on `Item`): red as a build, nothing ran. After the code, a first run
ran 17 and three failed, each a fault of the test or of one missing rule, none a hidden figure:
a run box (test asked "priced as" twice in a row; the second is "no change"), a rename (the
thing's line was not renamed with the item: fixed in `rename_item`), and the price-list note
("prices nothing on the job yet" said of the item's own entry: fixed in `tables.rs`, the entry
is now counted as made-of by the things set down). Then 17 of 17.
`demos/itemcost.sh` was red the same way until the figures it says were right (it first
looked for "Job cost" in `price`, which has none: it is in `estimate`).

### The code (all in crates/world/src, crates/cli/src, library, docs)
- `building/model.rs`: `Item.priced_as: Option<String>` (serde default, left out when none);
  `check_item` refuses an empty or space-edged name.
- `editing.rs`: `item_material_name` (the ONE function the held ruling flips),
  `place_item_said` (`place_item` is now a wrapper: the window's call is the same call),
  `item_priced_as` + `ItemFollowed` (the follow, by the same `Placed.item` road, through
  `change_assembly_said`), `assembly_named`, helpers; `rename_item` carries the item's
  price-list entry and the things' item lines with the name; `remove` refuses an assembly an
  item is priced as, and a price-list entry things are priced from.
- `derive.rs`, `view.rs`: a line made of the element's material, on an element with no
  material of its own that is one of an item set down, is made of the item. Only an element with
  `Placed.item` reaches the rule: walls, floors and roofs do not.
- `tables.rs`: the chest's four new columns, in and out; prices set through `prices_in`
  (one door), the price-list note counts the items' entries as used.
- `building.rs`: `show` of an item has a "priced as" row when it has one.
- `cli`: `set <item> priced_as "<assembly>"|unknown` (the verb items' fields... there is none:
  the only way items were changed was the table, so `set`, which every other field uses, takes
  it; no `--status`/`--source`: it is a name, and they are refused if given); `place` says
  "priced as ..." and what is not as the item says; `items` lists "priced as ..."; `estimate`
  (and `estimate by-item`) of a job with no cost line says what is set down and not priced
  (the sentence `price` already said).
- `library/sixteen-divisions.csv`: the new starter assembly (two rows, see decisions below);
  `library/items.csv`: the four new columns on every row (see "expectations").

## What was run, and what it showed (tree as in call196.patch; nothing committed)
- `cargo test` (one crate at a time, `-j2`, `--no-fail-fast`): world 535 passed 0 failed
  (33 test binaries, call196's 17 among them, after items.csv was given its four columns);
  plans 172, sheets 6, ifc 32, roofing 122: all pass. cli (`-p plumbline`): 1 failed, see
  "Expectations that moved". Logs: review/w196-*-tests.log.
- All 51 command-line demonstrations (every demos/*.sh without plumbline-app, open_window,
  xvfb or xdotool, except window.sh and field-job.sh; itemcost.sh is the 51st): 45 pass
  unedited, 6 stopped at an expectation that moved (below). review/demo196/*.log.
- The three root gates, `tools/licenses.sh` (exit 0), `BIMCRAFT=... tools/portmap.sh`
  (PASS, 330 public items): pass.
- An old job (written by the pre-change program in /home/user/roofnerd, which is behind this tree
  by call 194, so only the heaters job is compared): opens, `check` says its figures agree and
  saving it back writes the same file; `estimate` and `price` read the same, except that
  `estimate` of a job with nothing priced now also says what is set down and not priced.
  The sample whole house is $393,981.66, 432 lines, as call 194 left it (the "before" checksums
  of trim.sh were already stale after call 194; see below).

## Expectations that moved (named, NOT edited)
All because the starter library gained its 42nd assembly, "Bought and set":
1. `crates/cli/tests/cli.rs:170` `the_starter_library_comes_in_whole_...`: `set: 41` is now
   `set: 42`; and after that two more per-line assertions would fail on the item line
   (its unit cost is none of its own, so no `$?/` cell; and it has no division, so
   `cells(line)[26].parse::<u8>()` is empty). Only the first is reached (assert stops).
2. `demos/foundation.sh`, `framing.sh`, `joists.sh`, `shingle.sh`, `trim.sh`: stop at
   `says ... "set: 41"`. With only that figure changed to 42 in a scratch copy (not committed)
   each passes. `trim.sh` also prints `FAILED` for three sample-house checksums (sixteen.json,
   the two estimates) at its step 7 and exits 0: that is call 194's move of the sample house
   (job cost $390,260.26 to $393,981.66), not this work (the `( ... ) && echo` form hides it).
3. `demos/library.sh`: stops at `set: 41` (and "41 assemblies" in its heading), and then
   at its python check "a unit cost that is not $?/unit" for the item line.
4. Not run (window): `demos/pricew.sh` wants "41 starter assemblies".
The item line has no division in the library on purpose: it is put in the item's trade when
the assembly goes on (a heater in 15, a vanity in 12). If the lead wants every starter line to
carry a division, the item line would carry one fixed trade, wrong for most items.
`library/items.csv` carries the four new columns on every row (`,,,Unknown,not said`), because
`tests/tables.rs` (`an_item_library_goes_out_and_comes_back_the_same...`) asserts the library
written out equals the shipped file cell for cell; unedited, that test passes.
Mechanical, not an expectation: `crates/world/tests/build.rs` 4 `Item { ... }` literals gained
`priced_as: None` (a new field cannot be left out of a struct literal).

## Held for a ruling / not right
1. **Price-list name (point 3 and 6).** An item and a material cannot share a name (see 2.3).
   Built with ", bought" behind `editing::item_material_name`. A ruling to let them share
   changes `Job::add`, the file's check and every lookup by name; then that function returns the
   item's own name and the demonstration's `LISTED` variable loses its suffix.
2. **Three starter assemblies cannot be three of one name.** One assembly, "Bought and set",
   goes on Equipment, Fixture and Furnishing.
3. **crates/app/src/tools.rs:512 does not compile any more**: `plumbline_world::Item { ... form: None }`
   needs `priced_as: None`. One line; I may not touch the window, and did not build it.
4. Window code that sets a thing down from an item: `crates/app/src/scene.rs:1929` calls
   `editing::place_item(...)` (unchanged signature, now a wrapper). With the starter chest
   (no item names an assembly) it does what it did; an item that is priced as an assembly gives
   the thing that assembly's lines, flagged Known (the estimator's own pick, source "the item X:
   priced as Y"), and the window is not told the notes (`place_item_said` has them).
   The window's panel for an item (pick "Priced as"), its list of assemblies and its messages
   are not built; `item_priced_as(...).said()` is the sentences.
5. Item's lines take the item's trade (Division) where the assembly line has none; a later
   `item_trade` change does not move lines already on things set down (not built).
6. A thing set in a cabinet run is priced by the run's own line named for its item and never
   by "Priced as": named as "set in a run" when "Priced as" changes. (So a box of an item that
   is priced as an assembly is not double priced.)
7. A thing a person gave another assembly keeps it and is named; a line a person put on a
   thing stays and is named (both tested).
8. `estimate` / `estimate by-item` of a job with no cost line now ends with the existing
   "nothing is priced on N things that could be (...)" sentence (was said by `price` only);
   a job with bare walls and no lines also says it. No test or demonstration moved by it.
9. A second price for the same item in one table is refused by `prices_in`' own rule.

## Paths
- Report: SCRATCH/review/W-196-report.md
- Patch: SCRATCH/review/call196.patch

## Second round (the lead's rulings)

**Ruling 1, the price-list name.** The item is its own entry in the price list; no Material is made for it and the ", bought"
suffix is gone (`editing::item_material_name` now returns the item's own name: the one place). What an item holds for a price
is **the same component a Material holds, `Supply`** (price, quotes by supplier, order), put on the item's own element. The
only change to what the check allows: `check::supply` took only `Kind::Material`; it now takes `Kind::Material` or `Kind::Type`
(an item is a Type). A material means what it meant. One arithmetic: `derive.rs` and `view.rs` list every element with a
`Supply` of those two kinds as a priced entry, so a line made of a stud's material and one made of a heater go the same way,
three suppliers side by side included. `prices_out` lists the item as a row (it already listed any `Supply`); `prices_in`
finds a row's name among materials and items, and gives an item with no price its first one (so no `--add` is needed for an
item). A row naming an item priced as nothing sets the price on the item and says "prices nothing on the job yet" (the note
it gives for a material nothing is made of): test added. A new item's price in the chest's table goes through the same door.
Rename carries it (it is the item); the material-rename and material-remove code is gone. SPEC's `pl::supply` row says
"a Material, or an item". Demonstration steps 3 and 6 read with the item's own name.

**Ruling 2, the 42nd assembly.** Edited, "call 196" beside each: `crates/cli/tests/cli.rs` (`set: 42`; "the same as the job
already has: 202", which counts lines: 200 plus my two; the per-line checks excuse a line whose "Made of the element's
material" cell is set, in words, and assert that such a line has no unit cost and no division of its own),
`demos/foundation.sh`, `framing.sh`, `joists.sh`, `shingle.sh`, `trim.sh` (the count only; step 7 and the sha256 untouched),
`demos/library.sh` (heading, `set: 42`, 42 assemblies, 202 lines, 29 labor lines instead of 28, the same narrow excuse,
and the division list read from the lines not made of the material), `demos/pricew.sh` (42 starter assemblies; a window
demonstration, NOT RUN). One consequence the ruling did not name: the other new line, "Set in place" (Labor), is not made of
the material, so it is checked as before and now carries a fixed Division 11 in the library; to keep a heater's labor in the
heater's trade, `name_item_lines` now puts every line an assembly brings onto a thing set down from an item in the ITEM'S
trade (was: only where the line said none). This applies to any assembly an item is priced as, not only this one.

**When `estimate` says "nothing is priced on N thing(s) that could be (...): they are in no line and no total above"
("it is" for one).** It is `view::not_priced_said(&pricing.not_priced)`: things that could carry a price (walls, floors, rooms,
roof planes, eaves, hips, ridge, doors, windows, fittings, ducts, pipes, fixtures, furnishings, equipment... anything with a
figure to be bought by or bought by the each) and carry neither an allowance nor a cost line. (a) Nothing priced at all
(no cost line, no allowance): before my change `estimate` said only "Nothing in this job has a cost line."; it now adds the
sentence under it, in both `estimate` and `estimate by-item`. This is the widening you saw: it lists whatever could be priced,
not only things set down from items (239 things on the unpriced sample house). (b) Some priced, some not: the sentence was
already said, same words, at the end of the recap (call 17A-19; tests/call167 and cli.rs check it). (c) Everything that could
be priced is: not said. Things existing to remain are not counted. Gained the sentence (command-line demonstrations, from a
wrapper that logged each `estimate` whose output has both lines): `found.sh` (8 things: 4 footings, 4 walls), `hidden.sh`
(3: 1 wall, 1 pipe, 1 window), `itemcost.sh` (2 pieces of equipment, twice), `reno.sh` (1 room), `roomline.sh` (9: 7 walls,
2 rooms), `trim.sh` (239 things). None asserts its absence. Tests: `crates/cli/tests/call167.rs`
`a_bearing_wall_the_work_opens_says_so_under_the_estimate_with_nothing_priced` (a house with nothing priced; it asserts only
"Nothing in this job has a cost line." and the bearing-wall words, so it passes). I changed nothing about it.

**Run after the rulings.** Tests, one crate at a time, `-j2`: world 536, plans 172, sheets 6, ifc 32, roofing 122, cli
21: all pass, none failed. Command-line demonstrations: 51 run (50 listed plus itemcost.sh counted in them), all 51 pass
(library.sh needed its own division line fixed after the first run; every other passed first time). The three gates,
`portmap.sh` (PASS) and `licenses.sh` (exit 0) pass. Patch rewritten: the same path as before. Still not built: the window
(`crates/app/src/tools.rs:512` needs `priced_as: None`).
