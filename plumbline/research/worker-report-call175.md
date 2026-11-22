# W-175 report: call 175 at the command line

Working copy: scratchpad/wt175 (plumbline/). Nothing committed or pushed. Patch: review/call175.patch
(`git add -N . && git diff --binary`, 18 files). Demonstration: plumbline/demos/sweep-money.sh (passes; prints each figure).
Tests: 499 passed, 0 failed (the six crates). Gates: language, egress, client-data, licenses, portmap all PASS. The outside
checker (IfcOpenShell 0.9.0.post1, run by the demonstration on the item-7 file): "no fault against the schema and its rules".
All 69 command-line demonstrations that only run `plumbline` pass (field-job.sh is a helper that needs an argument). Not run: any
window demonstration; `cargo check -p plumbline-app` took over 115 s and was stopped (nothing in crates/app builds a Clash,
Clashes or Between, and no signature the app calls changed; not proved).

## Status by item

| item | state |
|---|---|
| 1 siding over stone | DONE |
| 2 base in the sample | DONE |
| 5 two on one line | NOT BUILT, premise is false (see below) |
| 3 set against | DONE |
| 4 a hole once | DONE |
| 6 header depth | STOPPED by the contract's rule (see below) |
| 7 BIM file Status, LoadBearing | WRITTEN and READ BACK for marks, bearing, type names; the ROOF IS NOT READ BACK |

## Hand figures against what was printed

| step | hand | printed |
|---|---|---|
| 1 face, openings, net_area | 400, 45, 355 SF | 400, 45, 355 |
| 1 net_length | 37.00 LF | 37 LF |
| 1 stone | 37.00 LF | 37 LF |
| 1 lap siding, hang the siding | 244.00 SF | 244 SF |
| 1 barrier, batt, sheathing | 355, 355, 400 unchanged | 355, 355, 400 |
| 1 "same wall with no door" | net_length 40, stone 40, siding 380-120 = 260 | DOES NOT COME OUT: with the window kept net_area is 375 (400-25), siding 375-120 = 255. (380 is 400 less a door's 20: the door is the thing taken away.) Printed 375 / 40 / 40 / 255. Demo checks 255. Not bent. |
| 1 a wall with no wainscot line | nothing new | no "net length" line in its show; saved job unchanged byte for byte (test) |
| 2 kitchen base | 45.08 LF (was 50.25) | 45.08 (perimeter 50.25) |
| 2 living room base | 99.28 (was 110.125) | 99.285 (the hand figure is it cut to a hundredth) |
| 2 sum of the lines | sum of the base runs | 493.540 LF both ways; perimeters were 550.500; every one of 10 rooms equal |
| 2 sample job cost | falls by the difference | was $394,677.48, now $390,174.76: falls $4,502.72. By hand: base 56.96 LF x $7.50 = 427.20; stone 3 LF x $95 = 285.00; siding 399 SF x $9.50 = 3,790.50; total 4,502.70 (2 cents is each line rounded to the cent) |
| 1 sample south wall (it has the front door) | stone 40, siding 329.99-120 = 209.99 | The lead's "if it has no door" is the NORTH wall: stone 40, siding 370-120 = 250 (printed). The south wall HAS the 3 ft door: stone 37, siding 329.99-111 = 218.99 (printed) |
| 3 duct + pipe, tub 0.45 in in | same space 1, set against 1 | 1 and 1 (tub listed under "Set against", "set 0.45 in into it") |
| 3 tub 6 in in | same space 2 | 2, set against 0 |
| 4 riser of 3 pieces, one floor | 1 hole (was 3) | 1 hole, "4.00 in round, through 3 pieces of the run", floor named. (Was 3: the three pieces are each in the slab; the Rust test asserts 3 clashes, 1 hole.) |
| 4 carried through a second floor | 2 | 2, one line for each floor |
| 7 literals | NEW/EXISTING/DEMOLISH | EXISTING x1, DEMOLISH x2 (wall + door), NEW x2 (wall + roof); Pset_WallCommon x3, Pset_DoorCommon x1, Pset_RoofCommon x1; LoadBearing x1 true; Plumbline_Values stays |
| 7 read back | 1 existing, 1 demolish, 1 new wall, door to demolish, types by name, roof, takeoff same | all but the roof: marks (each Assumed, sourced to the file), bearing, types "exterior wall 2x6" on 3 walls and "entry door 3 ft", takeoff identical in walls/to build/existing/to demolish. ROOF: "4 roof slabs: a roof is not brought in from a model yet" |
| the sample's clashes | | In the same space 11 (was 21 with the mountings), Set against 10, Through 60 holes from 71 pieces |

## What each item changed

1. `net_length` wall figure (building.rs; list in components.rs). Starter library: the 3 lines of the stone assembly (CSV). Sample: ground outside walls.
2. Sample "Base and trim" on `base_run`.
3. clash.rs: `Clash.set_against`, `SET_AGAINST_FT` (2 in). Set against = Hard, not through, depth <= 2 in, and either (a) a Furnishing/Equipment/Fixture
   and a Wall where the wall's own `things_against` list holds the thing, or (b) the two are in the job's network as joined (`joined`, from `network`)
   AND the clash lies within 3 in (0.25 ft) of the joint. I used: `things_against` (building.rs) and the `joined` map already built in `find`. The 3 in reach is my
   choice: at 2 ft the existing demos/clash.sh (a hot branch drawn a foot back up inside its main, joined at the end) turned from a clash into set-against; that is a
   drawing fault, so the reach is a termination's inches. CLI `clashes` prints "Set against (...): N" with its list; `vif` and the workbook say "Set against the other,
   not in its way" for such a question.
4. clash.rs: `Hole`, `Clashes.holes`, `Clash.hole`. Pieces of a run (and a fitting on it) through one wall/floor that are joined end to end AND pass within their widths + 3 in
   of each other are one hole. (Without the nearness test a stack and the tub drain joined to it merged into one hole.) CLI lists one line a hole: "1 hole, size, through N pieces".
   The VIF asks one question a hole. New CLI flag `run ... --points "6,5" --height "8 ft" --rises-to "8.4 ft"` (and `editing::riser`) because no riser could be drawn at the
   command line.
7. write.rs: `Status` and `LoadBearing` in Pset_{Wall,Door,Window,Slab,Roof,Footing}Common; only where a mark / a said bearing exists. lib.rs (reader): reads them
   back; types by the `Plumbline_Values` "type" row via new `editing::wall_type_named`, `opening_type_named`. Literals read from IfcOpenShell's
   site-packages/ifcopenshell/util/schema/Pset_IFC4_ADD2.ifc (PEnum_ElementStatus = NEW, EXISTING, DEMOLISH, TEMPORARY, OTHER, NOTKNOWN, UNSET; LoadBearing IfcBoolean).
   LoadBearing: said not bearing is written false.

## Item 5: not built, and why

The program ALREADY refuses it, everywhere: `check_job` (building.rs ~466-491, `bodies_overlap`) fails "footing-0006: lies along footing-0005 for 16.00 ft: one footing
drawn over another would be counted twice" when a second footing or wall is drawn on the first, moved onto it, or loaded from a file edited so (I tried all three,
including a different level and "existing"). So `check` cannot be asked to caution on it; the demonstration's step (draw the 16 ft footing, then `check`) fails at the draw. The
decision's "a caution, not a refusal" would mean relaxing that refusal, which changes tests/behaviour call 175 does not list. I wrote a caution and removed it again. Decision needed:
relax the refusal for pairs of different work status? (it already allows one to demolish). The sweep reviewer's file must have reached it by a path I could not find.

## Item 6: stopped (the contract's rule)

The header's depth feeds a counted figure today: framing.rs `frame()` sets `top = head + header_depth(span)` and puts a Cripple above the header only if `high - top > t`, and
`building.rs` counts `studs` (Stud+King+Jack+Cripple) and `header_length` from that frame; `studs` is priced. For most walls the cripple is there whatever the depth, but where a
head is within (depth + 1 1/2 in) of the top plates the cripple (and `studs`) depends on it. Making the depth Unknown needs a decision on what the layout then does, plus a place to
type it (the saved job gains a field). Nothing of item 6 was changed.

## Expectations moved (file, line, old, new, item)

- crates/cli/tests/cli.rs:389 `Job cost $394,677.48` -> `$390,174.76` (items 1 and 2: the sample's job cost).
- crates/cli/tests/cli.rs:390 `ends_with("$394,677.48")` -> `$390,174.76` (same).
- demos/quantityw.sh:77 (a WINDOW demonstration, not run): the sentence listing a wall's figures gains ", net length" at the end (item 1: `net_length` is a figure a wall has).
No other test or demonstration changed. demos/clash.sh, framing.sh and every other command-line demo pass untouched.

## Choices the decision did not make

- The 3 ft is a figure on the siding line (`wainscot = 3 ft`): a formula cannot take a bare 3 as a length. Library: Assumed ("the 3 ft in the name of the stone line"). Sample: Known
  (made-up, like its prices); Assumed turned the whole sample job cost to "~" and moved every expectation that shows it.
- `net_length` exists only where a line of the wall names it (so unpriced walls print what they printed). Adding it to the wall's figure list lengthens the
  "a wall has ..." sentence.
- Set-against thresholds as above. A Fitting is not a "placed thing" for the wall rule; a thing in a wall by more than 2 in is a clash.
- Marks are written only where said; an unmarked thing gets no Status.
- Reading `Status`/`LoadBearing`: always Assumed, source = the file's entry, even from the program's own file.

## What is not right

- The roof is not read back (item 7). The reader brings no roof from any model; the plane rows in Plumbline_Values hold "4 points ft", not coordinates, and the footprint
  is not in the file. Needs the roof's footprint, pitch and eave height written as data.
- The window counts clashes with `!x.through` (panels.rs:1153,1161; sheet.rs:214,586; scene.rs:1506,2055) and `vif`'s window list: set-against things are still counted and
  drawn as clashes there. Not touched (crates/app is off limits). The window's hole counts are per piece.
- `Clash.hole`/`set_against` are not saved anywhere (computed each time), so no file format changed.
- The hole grouping is geometric (3 in beyond the widths), so a long run laid inside a slab is split where its pieces are far apart.
- `demos/sweep-money.sh` passes while stating item 7's roof gap on its last lines; it is not a full pass of the STATUS hand figures for item 7.
- Time of runs: full tests ~3 min; sweep-money.sh ~5 s.
