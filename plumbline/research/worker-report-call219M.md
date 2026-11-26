# W-219M report (contract 219M, how tools feel, the model's side). Notes as I go.

Working copy: SCRATCH/wtB (detached at ef8cd2d). Nothing committed, nothing pushed.

## Before any code: the six answers

### 1. BIMCraft (/home/user/ipedrax/bimcraft)
- **Trying a change.** `Session::script_dry_run` (`crates/core/src/engine.rs:602`, tests `dry_run_tests.rs`): keeps the project (`Kept::of`),
  sets `dry_run`, runs the script all or nothing (`script_atomic`), puts back what was kept whatever it did, and answers made / changed /
  removed (by id and category, from `changed_since`) and warnings. A script that replaces the project, saves, undoes or names a file is refused as it comes
  (`dry_run_refuses`, 757). TAKEN: keep, run by the real function, answer, put back. OURS: one change and not a script (`Change`); the job is
  kept as its saved text (the same text the undo history keeps, `editing/history.rs`) and put back by loading it; the reply is the builder's words
  (`Tried::said`) with the clash test's answer for what the change touched (BIMCraft has no clash test).
- **Moving an opening.** `move` (`commands.rs:1211`, `move_elements` 12419, `relocate`): "doors and windows slide along their walls". An opening
  is held by host wall guid and offset (`walls/index.rs:38,107`: `hosts`). Nothing there moves one to another wall. OURS: `move_opening`
  (along, and to another wall by re-parenting, the opening keeping its path, type, lines and work mark).
- **Copy and array.** `copy` (`commands.rs:1219`, `duplicate` 12198): copies elements by a `by` offset, "with the doors and windows of copied walls";
  a copied opening alone is not made (it goes with its host). TAKEN: that rule. There is NO array/row command in BIMCraft
  (`grep -rn '"array"' crates/core/src/commands.rs`: nothing; the greps for array hit JSON arrays). The row is OURS: the one copy repeated a step farther,
  all or nothing, naming which copy a refusal is about.
- **Find.** `crates/ui-egui/src/find.rs`: Ctrl+K, every tool/command (`shortcuts::ACTIONS`) and view, matched by every word in any capitals
  (`matches`), `MOST` = 12 rows. TAKEN: the every-word rule, as the last rank. OURS: what is searched (the job's names and the estimate's lines), the order (exact, begins,
  words), the words each result is said in. The window's commands are the window's.
- PORTMAP.md rows for every new public item in `editing*` ("ours" or "BIMCraft's idea, written here"), `NOTICE` untouched (no BIMCraft code is copied: the shape of
  `script_dry_run` and the rule of `duplicate` are re-written here, not ported text). `BIMCRAFT=... tools/portmap.sh` to pass: see the run below.

### 2. Trying a change
`editing::try_change(job: &mut Job, change: &Change, how: Entered) -> Tried` (also `editing::apply(job, &Change, how) -> Result<Applied, String>`: the same
change made for real, by the same functions a verb goes by). `Change` is one of Walls, Opening, Runs, Item, Place, Move, Turn, MoveOpening, Copy, Row.
`Tried` has: `refused: Option<String>` (the model's refusal word for word, said by the names of things), `made` (path, name, kind, figures the model works out
(length, face less openings), `said`), `notes`, `same_space`, `set_against`, `holes`, `laid_in` (each a `Trouble { said, status, paths }`), `faces`, `fits()`,
`said() -> Vec<String>`. It keeps the saved text, makes the change by the real function (which, being all-or-nothing, puts the job back itself where it
refuses), asks the clash test about what was made or moved, and loads the kept text back. Proof of "as it was": the tests compare the saved text, the
job's next path number (`JobInfo.next_id`, the thing `add` gives out) and the undo history's labels before and after each kind of try, on the small job and on the sample whole house.
Cost on the sample whole house: see "What a try costs" below (measured).

### 3. How a door and a window are held
A door or a window is an element under its wall (`Node.parent` = the wall's path), with `Opening { type, offset, sill }`: `offset` is how far its MIDDLE is along
the wall's line from the line's FIRST end (the end the wall was drawn from), `sill` how high its bottom is above the wall's base; width and height are its type's
(`OpeningType`). So the lead's "4 ft from its west end" is the middle 4 ft along, not the near edge. Figures that depend on it, all worked from the model after the
change and none by hand: the wall's face less openings (`Measure` `net_area`, `opening_area`; derived from the children), the span in `building::drawn_faults` (refuses an
opening off its wall's ends, over another opening, or above its wall's top), the casing and trim (`trim::lay` reads `Opening` + `Measure`), the framing round it (`building/framing.rs`:
jack and king studs, header, sill), `carries` (what bears on a header), `planset`/`plan_set` (the plan's door gaps), the room's walls. A moved opening carries
all of them because they read the model; what a move to another wall has to carry itself is the parent, a name no other thing on that level has, and the wall's
work mark (a door cannot go into a wall to demolish). NOT handled (named): the door's swing is drawn only on the plan set (`plan_set.rs`, the arc) and is not a held figure;
a header the engineer sized by the old wall (`structure`) is not re-sized; framing is laid out from the model each time and has nothing to carry.

### 4. What a copy carries
Placed things (fixture, equipment, furnishing): type (`what`), sizes, item, offset, angle, assembly (`Condition`), cost lines, wbs, allowance, material, system,
connections made from the item (`of_item`). Not: its path (a new one, named "Lavatory 2" as `place` names), a connection a run or a model made (the copy is joined to
nothing: said), a work mark of existing/demolish (a copy is new work; temporary stays temporary: said). Runs: type, line, heights, justification, lines, system; joined to nothing
(said). Walls and footings: type, line, height/top, the openings (each with its own type, offset, sill, lines), lines, assembly; not what is known to be inside (`Hidden`: said).
Not copied alone: a door or a window (it goes with its wall), a fitting, a floor, a room. Marking: every value the copy holds that its original held Known is Assumed;
every value of its place and size says "copied from <name> (<who>)" in its source; the cost lines' entered inputs, stretch-out and waste are Assumed too; the line's
price is a price and is carried as it is.

### 5. What can be found by name
See `editing/find.rs` head: levels (Node name of Storey), rooms, things set down, what is built (walls, doors, ...), types, items (Type + Item), systems, materials, lines of the
estimate (description). Rule of order: exact, begins with, has every word; within each, level, room, thing set down, built, item, type, system, material, estimate line; then the job's order.
Words: "Lavatory 2: fixture, on Ground"; "Set lavatory: estimate line of Lavatory, on Ground, $12.00"; nothing found: `nothing in the job is called "south", and no line of the estimate has it`.

### 6. The ten steps by the code as it is, against the lead's figures
(filled in after the run: see below)

## Progress log
- 01:50 UTC: code written but NOT yet compiled (lead-run.lock and window-run.lock held since 01:03; I have not run cargo or any demonstration). Written so far:
  crates/world/src/editing/{slide,copies,trying,find}.rs (not yet declared in editing.rs, so the red run is a compile failure of tests/call219m.rs on missing items),
  crates/world/tests/call219m.rs, crates/cli/src/feel.rs (+ routes in main.rs, usage lines), crates/cli/tests/call219m.rs, demos/tools-feel.sh, README row, PORTMAP rows.
- Hand figure already found wrong in the lead's demonstration: a door's offset is its MIDDLE, so "4 ft + 15 ft + 3 ft = 22" (overrun 2 ft) is wrong: middle at 19 ft, east edge 20 ft 6 in, overrun 6 in.
  Also 5 ft east leaves 9 ft 6 in (not 8 ft 6 in) clear of the east end.
- 01:59 UTC one cargo build (compile errors only) started a moment before lead-run.lock re-appeared; all later cargo/demos go through review/gate.sh
- 02:31 UTC: RED runs (quoted): world test `cargo test -p plumbline-world --test call219m --no-run` before the modules were declared:
    error[E0432]: unresolved imports `plumbline_world::editing::AtOpening`, `...::Change`, `...::HowTall`, `...::OpeningTo`
    error[E0425]: cannot find function `move_opening` in module `ed` / `try_change` / `apply` (13 errors)
  CLI (old program built at ef8cd2d, not rebuilt): `plumbline try|copy|repeat|find <job> ...` each printed `plumbline: usage:` (no such verb); `plumbline move <job> door-0001 --by 1` said `--status is required` (no opening move).
- GREEN: crates/world/tests/call219m.rs 13 passed (the first green run had 4 failures that were test expectations, not the program: the existing refusal writes `10.00 ft` not `10 ft`; a duct's length is `true_length`; the estimate line was added after the copy in my test).
  Try cost on the sample whole house (debug build, nice 19, shared machine): 245-300 ms per try (wall 246, wall moved 288, door slid 296, wall copied 266, 3 things copied 265, thing moved 287).
- 03:00 UTC: crates/cli/tests/call219m.rs 7 passed; demos/tools-feel.sh passes every step (output in review/tools-feel.out, no FAIL).
  Fixes made after first runs (program, not expectations): the door move's reply said the faces twice (once as a note, once as faces); a move tried said "it would make nothing new"
  (now says nothing of making); a try now lists as made only what the change made (a wall then its doors), not roof planes that a wall move re-solves.
- NEXT: every command-line crate's tests (one at a time), every command-line demonstration, portmap, licenses, three root gates, patch.
- 04:10 UTC: from the lead's message (idle class), every cargo command and demo ran as `chrt -i 0 nice -n 19`, and I no longer waited on the locks.
- Final: tests world 686 (incl. 14 new), cli 58 (8 new), plans 187, sheets 6, ifc 111, roofing 122, 0 failed; 60 command-line demos rc=0 no FAIL (field-job.sh is a helper needing an argument, not a demo); portmap PASS, licenses ok, three gates PASS.
- Patch: review/call219M.patch. Try cost on the sample house (debug): ~300-375 ms (save 62 ms, load 111 ms, clash test 41 ms, the change's own save/load inside all_or_nothing the rest).
