# W-204 report (worker, contract 204) -- working copy wtB, base 273f8c1

## Part 1, before any code

(a) What the program said on the three houses of demos/gable.sh, with a Corner boards line (corner_board_length, $3.50/LF,
made-up) on each wall, BEFORE any change (script: review/w204-houses.sh; output review/w204-before/):

| house | gable top lines, said by `plumbline gable` | board per wall S/E/N/W, said (trim, estimate) | lead's hand figure | pieces (`trim --pieces`) |
|---|---|---|---|---|
| gable roof (PGPG) | E and W: (0, 9.5) (8, 13.5) (16, 9.5); S, N: none, an eave | 18 / 18 / 18 / 18 = 72.00 LF, $252.00 | 18 / 19 / 18 / 19 = 74.00 | 8 pieces, 72.00 LF, every one 9 ft |
| shed roof (PGGG) | E (0, 9.5) (16, 17.5); N (0, 17.5) (24, 17.5); W (0, 17.5) (16, 9.5); S none | 72.00 LF | 18 / 27 / 35 / 27 = 107.00 | 8 pieces, 72.00 LF |
| hip roof (PPPP) | none on any | 72.00 LF | 72.00 | 8 pieces, 72.00 LF |

The model's top line at every corner IS the lead's figure: the wall's line runs corner to corner, so the line's two ends are
the corners (east wall of the gable house: 9.5 at both; of the shed: 9.5 south, 17.5 north; the north wall 17.5 both). No point
on which the program and the hand figures differ; nothing to hand back.

(b) Every test and demonstration that prices or prints a corner board (grep "corner" over demos/*.sh, crates/*/tests,
crates/app/src; then read each), and what it holds today and will hold:

Moves (the house has a gable roof, the starter or typed corner-board line is on the walls east and west under the rakes):
1. crates/world/tests/call184t.rs `corner_boards_stand_two_at_each_outside_corner_nine_feet_on_their_own_faces`: every piece 9 ft
   (len3 == 9, to z == 9). Will hold: the south and north walls' four pieces 9.00 ft, the east and west walls' four 9.50 ft (to z 9.5).
   Arithmetic: top of the gable's line at the wall's ends, 9.5 (STATUS point 1). Sum of 8 pieces 74.00 LF (was 72.00).
2. crates/world/tests/call184t.rs `a_piece_is_known_only_when_everything_it_rests_on_is`: with Known walls and an Assumed roof "the
   walls' boards stay Known". Will hold: the boards on the east and west walls (which rise to the roof) are Assumed; the south and
   north walls' (under an eave) stay Known. Follows from point 2 of the call (a board under an Assumed roof is Assumed).
3. crates/world/tests/call185.rs `each_wall_of_the_box_has_two_outside_corners_and_eighteen_feet_of_corner_board`: 18.00 LF each,
   total 72.00. Will hold: 18 / 19 / 18 / 19, total 74.00 (STATUS point 1). Its house has a gable roof (edges 2 and 4).
4. demos/trim.sh step 1 (the line "the walls: two outside corners each, a board up each, 9 ft (18 LF a wall)"; hand lines
   `corners and board` "2, 18 LF" for each of the four walls): east and west will be 2, 19 LF (hand: 2 x 9.5); south and north
   stay 18.
5. demos/trim.sh step 2 (comment: corner boards 4 x 18 = 72; its sum check, if the sums are on the line) and step 3 (made-up price
   $3.50: "corner boards 72 x 3.50 = 252.00"): will hold 74 LF and 74 x 3.50 = 259.00.
6. demos/trim.sh step 6 (pieces): want "Corner boards": (8, 72.0) becomes (8, 74.0); "each corner board stands 9 ft from the
   foot" becomes: south and north boards 9 ft, east and west boards 9.5 ft, from the foot (z 0); the sentence "corner boards:
   8 pieces come to 72.00 LF; the lines price 72.00 LF" becomes 74.00 and 74.00.
7. The same house's other sums in trim.sh that add the 72 (steps 8's C-checks do not use corner boards): to be found by running.
Does not move (read, and will be run):
- demos/trim.sh step 4 (the L-shaped house, no roof): 90.00 LF, 10 boards. Step 5, 8: roof alone / no corner boards.
- demos/trim.sh step 7 and demos/sixteen-trim.sh: the sample whole house, a hip roof: 128 LF, $468,050.22, the four sample
  houses' sha256 (demos/trim-before.sha256).
- crates/world/tests/call194.rs (no roof; the box house), call199.rs (no roof), call192.rs (no corner board priced).
- demos/change-assembly.sh (no roof): 20 and 14 LF.
- demos/gable.sh: prices siding, sheathing, barrier and the hanging of the siding only; no corner board is priced or printed
  (steps 1 to 5 hold their figures); extended by me to the seven steps of the call.
Window demonstrations, which I cannot run (no screen) and which are NOT run:
- demos/trimw.sh: house 1 of the call-193 part and the main house have a gable roof (edges 2 and 4). Its checks count pieces (8
  corner boards + 7 of casing = 15; the Trim switch reads 15 (13) and 27), read the SOUTH wall's boards only (under an eave: 9 ft,
  unchanged), and compare pictures by dots (corner-boards strip c >= 100 dots differ: grows, not shrinks; "(2) with no corner
  board priced" has none). Piece counts do not move. No expectation text moves; its pictures (demos/captured/trim) show the east
  and west boards half a foot short and are stale.
- demos/gablew.sh, demos/gableendw.sh: the starter lap siding carries an unpriced "Corner boards" line, so the east and west
  boards are drawn; they check money counts of lines ("2 of them had no price yet", unchanged) and the colour of the gable's middle
  and the hip house against the program before (hip unchanged). No expectation text moves; their pictures at a gable's corners
  change (a board half a foot taller) and are stale.

(c) BIMCraft: searched all of /home/user/ipedrax/bimcraft: `grep -rniE 'corner.?board|cornerboard'` finds no file, no casing,
fascia, baseboard or moulding either; so no corner board and no trim that follows a wall's profile. What it has is
`walls.rs:667 top_at(m, w, s)`, a wall's top at the point s along its centre line, read from the roof a wall is attached to
(`attach_top`). The idea (a wall's top at a place along it) is what the corner board needs; the model's own top line (call 192)
is read instead, by `gable::top_over`: ours, and PORTMAP says so.

### Part 1 built (patch: review/call204-trim.patch)
- Red first: crates/world/tests/call204.rs (7 tests) run against the unchanged src: 6 FAILED, 1 passed (the hip/no-roof case, which must not move):
  gable `[18.0, 18.0, 18.0, 18.0]` for the wanted 74; shed `[18.0, 18.0, 18.0, 18.0]` for 107; statuses `[Known, Known, Known, Known]` for
  `[Known, Assumed, Known, Assumed]`; wainscot `Some(12.0)` for 21; unplaceable roof `Some(18.0)` for None; the shed east wall's pieces both 9 ft.
  (My first red run had the wainscot test failing for a mistake in the test; I corrected it and ran red again with src stashed: the figures above.)
  Green on the changed src: 7 passed.
- `gable::top_over` (the wall's top at s along it, off the model's own top line) and `trim::height_at_end` (the wall's height at an end; refuses,
  with the gable's own reason, where the gable cannot be worked), `trim::corners` returns a `Corners{count, board, up}` (the board at each outside
  corner) and the figure is the sum of those; `building.rs` `corner_board_length_above_<entry>` takes the entry off each corner's own height;
  `trim/lay.rs` lays each piece to the same height (one arithmetic), start clamped per piece. Status: only a wall whose top line rises takes the
  gable's (roof's) status; a wall under an eave is as it was, Known stays Known.
- The saved job holds the figure's formula text. I changed it first for every wall and demos/trim.sh step 7 (sixteen.json sha256) went red, as the
  contract says it would if anything moved, so I put the OLD text back (`outside_corners × height: ...`) for every wall that stands at its own
  height at each corner, and a new text only where a top line carries a corner up. The step passes unchanged (sha256 file untouched).
  SPEC.md line 488 still says only the old formula; SPEC.md is not mine to touch: for the lead.
- Not so simple, decided, and handed up: where the gable cannot be worked (a plane of the roof with no pitch) the wall's corner boards have NO FIGURE
  and say why (the gable's note), never the wall's height as if no roof were there. Held for a ruling if the lead wants otherwise.
- demos/gable.sh: new step 6 (6.1 to 6.7 = STATUS points 1 to 7 of the section, the Known case 6.5 held by call204.rs since the command line enters
  nothing Known there). demos/trim.sh: steps 1, 2, 3 and 6 moved as listed below. Both pass.

### Expectations moved in Part 1 (each by the arithmetic of STATUS point 1 or 2)
1. call184t `corner_boards_stand_two_at_each_outside_corner_...`: 9.0 on every piece -> 9.0 south/north, 9.5 east/west; sum 72.00 -> 74.00.
2. call184t `a_piece_is_known_only_when_everything_it_rests_on_is`: boards of east and west under an Assumed roof are Assumed (point 2).
3. call185 `each_wall_of_the_box_has_...eighteen_feet_of_corner_board`: 18 each, 72.00 -> 18/19/18/19, 74.00.
4. demos/trim.sh: step 1 east, west `2, 18 LF` -> `2, 19 LF`; step 2 comment and `sum "Corner boards" 72.00` -> 74.00; step 3 `72.00 / 252.00`
   -> `74.00 / 259.00` (74 x 3.50); step 6 `Corner boards (8, 72.0)` -> `(8, 74.0)`, the 9-ft board check -> 9 ft (south, north) and 9.5 ft
   (east, west), from the foot; `8 pieces come to 72.00 LF; the lines price 72.00 LF` -> 74.00 and 74.00. Step 7 (the four sample houses'
   sha256) UNCHANGED and passing.
5. The window's, NOT run, none edited: demos/trimw.sh, gablew.sh, gableendw.sh. I read each: no expectation text moves (they read the south wall's boards,
   counts of pieces and of lines, and dots that differ only grow); their pictures at the gable ends are stale (a board half a foot taller).
All of: world 567 tests pass (21 test files and unit tests), cli 5 files pass, portmap PASS (BIMCRAFT=/home/user/ipedrax/bimcraft), licenses and the three gates PASS.

## Part 2, before any code

Time of `plumbline import ifc` of the sample house's own file (`plumbline sixteen` then `export ifc`, 560,153 bytes; then `import ifc <empty
job> first.ifc --status Known`), a DEBUG build (the program as the tests and demonstrations run it), three runs: 18.30 s, 18.91 s, 18.35 s.
(What the lead called twenty seconds.)
Where the time goes, two ways, said how:
- A sampling profile (gdb attached to the running program, 12 stack samples, the stacks read): every sample but one is in `Job::derive` ->
  `building::measures` -> `room_measures` -> `rooms_among` -> `rooms::Plan::of_walls` / `split` / `region_in` (the rooms' outlines worked out
  again); the one other is in `file::save` (`write_pretty`). The callers are `editing::all_or_nothing` inside `bring`: each thing brought in is
  made by the change a tool makes (`place`, `footings`, `connect_port`, ...), and each of those is its own all-or-nothing change.
- Timed stages (a temporary clock in `Job::derive` and in `all_or_nothing`'s snapshot, taken out again): total 20.9 s that run; parsing the file
  0.02 s; `derive`: 211 calls, 15.74 s (75%); the snapshot `file::save` before each change: 204 calls, 3.99 s (19%); the rest (reading, the
  checks `check_whole` and `new_faults`) about 1.2 s (6%).
So: each of ~205 changes re-derived the whole job, and every derive worked the same ten rooms out again from walls that had not moved.
What I changed (the measurement names it): `building::rooms_among` (a pure function of the walls, room lines, points and tolerance) is kept by
what it was worked out from (Debug text of every input, exact), in a per-run cache of at most 256 entries, so the same walls are not worked
out twice. Nothing else. Measured after (debug build, three runs of the same import): 6.46 s, 5.96 s, 6.38 s. derive is now 1.55 s of it; the
snapshot saves (204 x ~16 ms = 3.2 s) are now the larger part. I did not touch them: the snapshot is what lets a change that fails put the
job back and the import go on (it is used when one of the 205 is refused), and a shared snapshot is a change to how a change is kept, which the
measurement does not call for. No figure moved: the job file written by the import is byte for byte the one before (cmp), its report line for
line (diff), takeoff and estimate the same. HELD for a ruling if the lead wants it under 2 s on a debug build: nested changes sharing one snapshot.

What a beam and a column are in the files the program reads and writes today:
- WRITES: a beam as `IFCBEAM(...,.BEAM.)` and a post as `IFCCOLUMN(...,.COLUMN.)`, each a rectangle (or circle) pushed along the run's centre line
  (write.rs `Shape::Run`, "a beam is held to its top": centre = top less half the depth), with quantities (`Qto_BeamBaseQuantities`,
  `Qto_ColumnBaseQuantities`) and the `Plumbline_Values` property set. A pad footing is `IFCFOOTING .PAD_FOOTING.`.
- READS: neither. `IFCBEAM` and `IFCCOLUMN` are in the table of words for what a model calls things (lib.rs 517) and nothing brings them in: every
  one falls through to "nothing here to bring it in as" (lib.rs ~2038; "the model gives it no place, and there is nothing here to bring it
  in as" where it has no place). The program HAS both (`ed::beams`, `ed::post`, `ed::framing_type`, kinds Beam and Post), so it is untrue.
  (A beam in demos/ifc/made-up-house.ifc, the designer's file, is such a straight box: 0.1 m by 0.3 m, 9.144 m long, level.)

### Part 2: expectations that move (listed here BEFORE any is edited; each by the arithmetic after it)
1. crates/ifc/tests/ifc.rs `a_model_comes_in_as_storeys_walls_...`: the designer's file's ridge beam (a 0.1 m by 0.3 m rectangle pushed 9.144 m level)
   is a straight box, so it comes in: `made` gains ("beams", 1); the left-out list is the stair alone, not the beam and the stair. By hand
   0.1 m = 0.3281 ft = 3.937 in; 0.3 m = 0.9843 ft = 11.811 in; 9.144 m = 30 ft. (Figures to be read off the job in the same test.)
2. crates/ifc/tests/whole.rs `p2_fittings_come_in_on_their_runs`: 41 fittings in the first job, all 41 written now (39 + the 2 with no size);
   the 2 the tool refuses where a third end meets stay refused: 39 come back, not 37. `made["fittings"]` 37 -> 39. The Bends: the first
   job's Bends less those back = 7 before (2 not written, 2 refused, 3 become Tees); now the 2 with no size come back as Bends, so 7 - 2 = 5.
3. crates/ifc/tests/whole.rs `the_same_model_twice_counts_no_footing_and_no_roof_twice`: the same 37 -> 39 (fittings counted once).
4. crates/ifc/tests/write.rs `the_house_is_written_whole_...`: "its two elbows have no shape, and each is said": 2 -> 0 said, 41 fittings written.
5. crates/ifc/tests/ports.rs: comment only (the number, 4, holds: a fitting with no size has no connection points written).
6. Command line demonstrations that I find by running them all; each listed below with its arithmetic before it is edited (demos/ifc-whole.sh
   "39 fittings" -> 41, "37 fittings" -> 39; demos/export.sh "it has no shape: its size is not known"; demos/model.sh the beam).

### Part 2 built (so far)
- Point 5 (crates/ifc/src/lib.rs): `Reader::beam` (a rectangle pushed level, square to its length, one side plumb and one level; why not otherwise:
  "its section is round, not a rectangle", "it slopes: a beam here is level", "its section is turned about its length", "its body is made
  of more than one piece", "it is already on this level: ... counted twice") and a block after the footings that brings in IFCBEAM by
  `ed::beams` (its section a type of its kind found or made by `ed::framing_type`, top held as a beam here is: middle + half its depth, above
  the level's floor) and IFCCOLUMN by `ed::post` (section from the box, round where the body is round; "it is turned on the plan" where it is).
  Every value Assumed (never Known on the file's say-so), sourced to the file, the name and the number in it.
- Point 6 (crates/ifc/src/write.rs): a fitting with a place and no size is written as the fitting it is (`fitting_class`, the one table) with its place and
  no body, counted among "fittings", on its level and its system, and no longer said to have no shape; its connection points are not written (they
  are placed on a body). Reader: a fitting with no body at all is put where its place is (the joint of the runs there) by the same change; it
  comes back with its size Unknown, as the runs there are. NOT more than that: a fitting's own connection points are still not read.
- Point 7: see "before any code": `rooms_among` kept by its inputs (crates/world/src/building.rs).
- New tests (red first, green after): crates/ifc/tests/call204.rs, 4 tests. Red: all 4 FAILED with "nothing in the model but its levels could be
  brought in: 1 beam (nothing here to bring it in as); 1 column (...)" and "left: 39 right: 41" for the fittings; green after.
- A thing found, not asked: with the ridge beam of demos/ifc/made-up-house.ifc now in the job, the supply duct (6 in round, 7 ft up, along y = 10 ft
  for 20 ft) lies in it: a second clash "Supply duct and Ridge beam: in each other by 2.35 in, at (15, 10, 7.15)" (beam bottom 2.15 m = 7.0538 ft,
  duct top 7.25 ft: 0.1962 ft = 2.35 in). It is a real question for the field as the file is Assumed. The test and the demonstration that hold the
  one clash were moved to two (ifc.rs; model.sh if it holds it, found by running).

### Command-line demonstrations that moved in Part 2 (found by running all 52; listed before editing; arithmetic each)
(Run 1 on the Part 2 tree, 52 run: 48 passed, 4 failed: export.sh, ifc-whole.sh, model.sh, structure.sh. The other 48 are not moved.)
- demos/export.sh: "it has no shape: its size is not known" is no longer said: 41 fittings written, not 39 plus 2 said; "37 fittings" brought in -> 39.
  (41 written - the 2 the tool refuses where a third end meets = 39.)
- demos/ifc-whole.sh: "39 fittings" written -> 41; "37 fittings" brought in -> 39; "not brought in: 2 things" stays (the two refused).
  New section: a beam and a column come in (the made-up file: 1 beam 12 ft, 3 1/2 by 9 1/4 in; 1 post 8 ft, 3 1/2 in square) and the file passes IfcOpenShell.
- demos/model.sh (the designer's file with a ridge beam, a stair and a chair): "not brought in: 2 things" -> "1 thing"; "1 beam: nothing here to bring
  it in as" -> brought in "1 beam"; the stair stays. The beam is 30 ft (9.144 m) long, 3.937 in by 11.811 in, top 8.04 ft up.
- demos/structure.sh (a beam, two posts, two pad footings, a strip footing, written and read back): the beam and the two posts used to be "not read back
  in ... and import says so by name"; now "1 beam" and "2 posts" come in and "not brought in: 0 things".

## Final (both parts)
- Tests, one command-line crate at a time (CARGO_INCREMENTAL=0 ... -j2): world 567, ifc 47, cli 22, sheets 6, plans 172, roofing 122: 936 passed, 0 failed.
  (The window's crate was not built or run.)
- Every command-line demonstration: 52 run (every demos/*.sh without plumbline-app, open_window, xvfb or xdotool, but window.sh and field-job.sh), final
  run on the final binary: 52 of 52 exit 0; the words FAIL and FAILED searched in all 52 outputs: none. (First run on the Part 2 tree: 48 of 52; the
  four, export, ifc-whole, model, structure, are the moves listed above.) demos/trim.sh step 7 (the four sample houses' sha256) passes unchanged.
- IfcOpenShell (0.9.0, `tools/ifc-check.py`, run never linked): the sixteen-house file with 41 fittings, the beam-and-post file and the gable roof file:
  no fault against the schema and its rules (ifc-whole.sh runs it on first.ifc, gable.ifc, framing.ifc).
- Gates: gate-language, gate-egress, gate-clientdata --all: PASS. portmap.sh (BIMCRAFT=/home/user/ipedrax/bimcraft): PASS (353 public items). licenses.sh: no GPL.
- Patches: review/call204-trim.patch (Part 1 alone), review/call204.patch (both).
- Not touched (outside the contract's files): SPEC.md line 488 (the corner board's formula, now two wordings: the old text where the wall stands at its own height,
  a new text where a top line carries a corner), STATUS.md ("Not built yet" under the demonstration), the window's files.
- Held / not right: (1) a wall whose gable cannot be worked gives its corner boards no figure and says why (my decision, said above); (2) the 204 snapshot
  cost: with the cache an import is ~6 s on a debug build; the remaining 56% is the snapshot saves, a shared snapshot is not built; (3) a fitting with no size
  has no connection points written, and a fitting's own connection points are still not read back; (4) the window demonstrations trimw, gablew, gableendw were
  not run (no screen): no expectation text moves, their pictures are stale at gable ends; (5) the new clash of the designer's ridge beam with its duct is a
  consequence of reading the beam, not asked for.
