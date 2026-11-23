# W-179C report (worker, Sonnet 5.5). Working copy: scratchpad/wtA. Written as I go.

## C1. BIM file read back moves every wall: FIXED
Cause: the writer (`crates/ifc/src/write.rs`, `carried`) writes the first of two walls meeting square at a corner
running through to the outside corner and the second stopping at its face, so a body is up to half a neighbour's
thickness longer or shorter than the drawn line; the reader (`crates/ifc/src/lib.rs`, walls) took the body's
long side as the wall's line and length.
Fix: the writer now also writes the wall's own line as IFC's standard "Axis" representation (a two-point polyline,
own subcontext 'Axis'); the reader takes that line when it lies along the body (same direction, its middle inside
the body, within half the thickness across), else the body as before (foreign files unchanged).
Red: new test `a_wall_this_program_wrote_is_read_back_on_its_own_line_to_its_own_length` (crates/ifc/tests/write.rs),
run against the old lib.rs/write.rs: "a wall drawn [0,0] to [20,0] came back [-0.1667,0] to [20.25,0]". Green with fix.
Demo: `demos/sweep-money.sh` step 7b (ring 20x12, last wall 4 in, door at 5 ft, window at 8 ft): four lines as drawn,
20/12/20/12 LF, each wall's line, plan length, face/opening/net area, door and window identical to the first job's. Passes.
Other inputs: a 45-degree four-wall chain with an acute/obtuse joint, plus an open 2-wall L: all six lines come back exact.
Outside checker passes the ring file. Entity numbering of written files changes (new context + polylines); no test depended on it.

## C3. A joist mark on a notch's edge counted twice: FIXED
Cause: `members_laid` (crates/world/src/building/members.rs) dropped a mark's piece only when it had the SAME length as
an edge member beside it; a mark falling on a notch's edge runs longer than the edge member, so both were counted.
Fix: one rule for both: when a mark's piece and an edge member lie one along the other (within a member's thickness across),
the longer is the joist (equal: the edge member) and the shorter is dropped. Edges are now emitted after the marks are cut.
Red: two new unit tests (mirrored L, U) run against the old function: 20 where 19, 26 where 25. Green with fix. Existing
L/rectangle/hip/gable tests unchanged and pass.
Hand: mirrored L 19 / 250 LF / rim 48; U 25 / 408 LF (workings in the test comments). joists.sh gained both (printed: 19 EA, 250 LF, 25 EA, 408 LF).
Other inputs: the mirrored L turned a quarter (joists east): 19 / 250 (unit test); mirrored L at 12 in: hand 25 / 328 LF, printed 25 EA / 328 LF.
Not handled / not right: a mark's piece that only PARTLY overlaps an edge member (neither contains the other) keeps both; I found no ordinary outline that does this.

## C2. Window with a low sill taken off the siding twice: FIXED
Cause: the siding line was `net_area - wainscot * net_length`; net_area already lacks the window's part under 3 ft, and the
band `3 x net_length` takes it again (net_length only lifts openings that come to the floor).
Choice (and why): a new wall figure that takes its height from the line, `net_area_below_<entry>` (read aloud "net area below wainscot"):
the face up to that height less the parts of the openings below it, worked in building.rs beside net_length and only where
a line names it. The height is the line's own entry (`wainscot`, Assumed), so changing it on the line moves the figure and
its Assumed flag flows. Smaller/safer than function syntax (the formula grammar stays "four operations and names"); `formula::names`
returns the entry as used too so the unused-entry check, formula edits and dependencies work. Library (2 lines: siding, "Hang the siding")
and sample.rs now use `net_area - net_area_below_wainscot`; the stone line stays `net_length`. Band is h x plan_length, not clipped to a low
wall, so a 2 ft wall under a 3 ft stone is still Unknown for siding (as before).
Hand: 40x10 wall, door 20, window 25, net 355: sill 1 ft: below = 120-9-10 = 101, siding 254; sill 3 ft: 111, 244; sill 0.5: 98.5, 256.5. Judge's wall (no door, sill 1 ft): net 375, below 110, siding 265: the judge's 265 is RIGHT.
Printed (demos/sweep-money.sh step 1): 254, 244, 265, stone 37 LF.
Red: crates/world/tests/call179c.rs run with the old formula text: fails (the figure is not there; the judge's old figure was 255). Green: 4 tests.
Other inputs: a 5x2 window at the floor (old formula bought 285, right 280): 280; a window 5x5 sill 2 ft (top at 7): 260; height changed to 4 ft: 230; 2 ft wall: Unknown.
EXPECTATIONS MOVED (all C2): the made-up whole house's siding now takes the windows' sills at 2 1/2 ft (south 5 SF, east 2, west 2) into account:
 - crates/cli/tests/cli.rs:389-390 job cost $390,174.76 -> $390,260.26 (+$85.50 = 9 SF x $9.50).
 - demos/sweep-money.sh step 2: south wall siding 218.99 -> 223.99 SF; job-cost fall 4,502.70 -> 4,417.20 by hand (printed 4,417.22).
Not moved: step 1's 244 and 255 (the sill is at 3 ft / the window above).

## C4. A hole's place after a pipe is split: FIXED
Cause: for a run that carries on through joined pieces, the clash's place was taken as the deepest point of the whole length (kept for
runs that end inside things), which for a thin wall is off the wall: (5.97, -1.375, 4.04) after a split at y = -1.
Fix (crates/world/src/building/clash.rs, `cross_at`): the place of a through clash (hole) is now where the run's whole line, carried on through its
pieces, crosses the wall's middle (plan intersection of the lines; height from the run) or the floor's middle (mid-depth).
Red: `a_hole_is_where_the_run_passes_through_however_the_run_is_split` (tests/call179c.rs) with the fix switched off: [5.9726, -1.375, 4.0438] (the judge's figure). Green with it.
Hand: (6, 0, 4) before and after splits at y = -1, 0.1, -0.2; a diagonal wall (0,0)-(10,10) crossed by a pipe along y = 4 at 5 ft: (4, 4, 5), also after a split.
Demo: sweep-money.sh step 4b. Two more inputs: the split at (6, 0.1) across the wall and the diagonal wall, both exact.
Note: a through hole in a FLOOR is now placed at mid-depth of the slab (was: deepest-point/middle of the overlap); nothing asserted that figure.

## C5. A pipe lying in a slab is "1 hole": FIXED
Cause: any run meeting a wall or floor was a `through` clash and so a hole.
Fix: `Clash.laid_in` (new field): a through clash whose run (whole carried line) has both ends inside the floor's depth or inside the wall's thickness
is laid in: it has no hole; `Clashes::laid_in()`; the clash listing (cli) says "Laid in a floor or a wall (it runs inside it, along its length: no hole to cut): n" apart
from holes, with the Assumed/Known count; `hole_rows` and holes exclude it; vif.rs skips it (not a question).
Red: with `laid_in` forced false the test `a_run_lying_in_a_floor_or_a_wall_along_its_length_is_laid_in_and_no_hole` fails: 1 hole, 0 laid in. Green with the fix.
Demo: sweep-money.sh step 4c (pipe in the slab: 0 holes, 1 laid in; a riser out of the slab: 1 hole).
Not right: a run whose axis is outside the wall but whose skin touches it (y = 0.4 against a 6 in wall) is still counted as a hole, as before (not changed).
vif.rs edited (one line) as the contract allowed.

## B5 and B6. What may not be drawn: FIXED
Cause: nothing guarded them (only walls/footings on one line and identical floors were refused).
Fix: `building::drawn_faults` (building.rs) lists, as (key, sentence): an opening past its wall's end/start or over another live opening in it (an opening to
demolish is left out, as the wall's figures leave it out); two floors of one level with any area in common (edges crossing, a corner strictly inside,
or a piece of one strictly inside the other: touching along an edge is clear); two beams on one line (either way round, same top), two posts or two
pad footings at one point with heights in common, on one level. `editing::all_or_nothing` runs it after every edit and refuses an edit that brings in a
fault the job did not already have (the faults of the job as it was are found by loading the saved text of it): so a job saved before that holds one still
opens (check_whole, and so reading, is untouched), can be edited about anything else (even the faulty thing's other values), and says it:
`plumbline check` now prints "! <sentence>" for each. Moving an opening (measure ... "center, along the wall") onto another or past the end is refused the same way.
Tests (tests/call179c.rs): openings (ends, touching, door under window, moved), old job opens/says/edits, floors (inside, overlapping, enclosing, half, sharing an edge,
other level, an L with a floor in its notch and one 1 ft into it), beams/posts/pads. Red: with `new_faults` made to find nothing, 4 of the 10 tests then in the file failed
(floors, old-job, openings, members). Green with it. Demo: sweep-money.sh step 7d.
EXPECTATIONS MOVED (B5/B6):
 - crates/world/tests/world.rs ~5485 (j12_6_...): the test drew a second door over the first to show the wall's figures go Unknown and name both. That door is now refused when drawn; the test asserts the refusal and
   puts the door in by hand (as a saved job holds it); the Unknown figures and their notes are asserted unchanged.
 - demos/joists.sh: the L floor was drawn on the same level over the 24x16 floor; drawn 120 ft east (the figures are position-free: 20 EA, 250 LF, rim 48 unchanged).
Not right / not done: a beam drawn along part of another's line, or two posts a hair apart (> 0.01 ft), are not refused; a beam is "in one place" only for the same two ends and top.
Not handled: footings are not in B6 (walls/footings on a line were already refused).

## C6 and C8: FIXED
C6 cause: the reader put a wall's work mark on at once, then could not set its door in "a wall that is coming down": the door was dropped (said "1 door ... would not take it") and the advice contradicted the program.
Fix: wall marks are put on after the openings are in. Red: test `a_door_in_a_wall_to_demolish_is_read_back_in_it_and_what_is_said_is_in_words` with the old reader: doors (Some(1), None). Green: wall and door each to demolish. Demo 7c.
C8: (a) "18 lb/LF", "18 lbs/lf", "18 lb / LF", "#/lf" are read as 18 lb/ft (stored unit unchanged); the refusal now names both ways. (b) the import's note says
"N lists of what the model says of each thing", "its work mark ... and its bearing mark" (no "property sets", "Status", "LoadBearing"). (c) the BIM file's quantity notes name figures in words ("plan length × height",
"face area − opening area") and a volume's formula no longer reads "(x ÷ 27) × 27": the ÷ 27 is dropped ("section area × length, in cubic feet").
EXPECTATIONS MOVED (C8): crates/ifc/tests/write.rs:92 `n.contains("property set")` -> `n.contains("of what the model says of each thing")`.
Not changed: C7 (not mine). The words "IFC"/"IFC4" stay in the import's lines.
Not in C6/C8: the export's "not written" list is untouched.

## Totals
Tests (`cargo test -q -j2 -p plumbline-world -p plumbline -p plumbline-ifc -p plumbline-sheets`): 455 passed, 0 failed (new: call179c.rs 11, ifc write.rs 2, members 2).
Demos that run only `plumbline`: all 44 pass with PLUMBLINE_BIN (sweep-money, structure, joists included). `demos/field-job.sh` takes a job file as $1 and was not run (not a result of these changes; it also stopped at "$1: unbound" before them).
Gates: gate-language, gate-egress, gate-clientdata --all PASS; tools/licenses.sh exit 0; BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh PASS (3 rows added to PORTMAP.md: laid_in, drawn_faults; hole_rows existed).
Outside checker (IfcOpenShell via tools/ifc-check.py, in sweep-money 7 and 7b): passes the files, incl. the ring with the Axis lines.
Patch: scratchpad/review/call179C.patch (git add -N . && git diff --binary).
Expectations moved, list: cli.rs 389-390 ($390,174.76 -> $390,260.26, C2); sweep-money step 2 (218.99 -> 223.99; fall 4,502.70 -> 4,417.20, C2); world.rs j12_6 (second door by hand, B5); joists.sh L floor offset (B6); ifc write.rs:92 wording (C8).
Not touched: C7, app, plans, DECISIONS/STATUS, gates. vif.rs: one line (skip laid-in clashes).
