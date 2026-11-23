# Worker report: call 185, the outside trim priced (command line only)

Working copy: /tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/wtK (plumbline/)
(Written as I go; the final sections are filled at the end.)

## Built (in order)
1. Figures, `crates/world/src/trim.rs` (new module, one arithmetic; the `trim` verb of the CLI and a cost line's quantity both read it):
   overhang (eave and rake), outside_corners and corner_board_length (wall), casing_length (window, door).
   Hooks: `derive.rs` (eave/rake), `building.rs` (wall, opening), `components.rs` (`Kind::measures()` names).
   Given ONLY where a line of the thing names the figure (the way a wall's `net_length` is), so no existing job
   or sample house gains a figure, nothing is reported stale, and a job saved before opens and saves the same bytes.
2. Tests written first: `crates/world/tests/call185.rs` (10 tests). Red first (a skeleton `trim::figures` returning
   nothing, the library untouched): all 10 failed, e.g.
   - `its quantity "true_length * overhang" uses "overhang", which is not a figure of this thing (a eave has plan length, true length)`
   - `its quantity "corner_board_length" uses "corner board length", which is not a figure of this thing (a wall has plan length, height, ...)`
   - `"casing_length" ... not a figure of this thing (a window has width, height, opening area)`
   - `no assembly "Rake trim" in this job`
   - `no figure overhang of eave-...` (the helper's panic)
   Then the code; 9 green; the 10th (starter lines) green after the library rows.
3. Starter lines in `library/sixteen-divisions.csv` (9 rows, none priced).
4. `demos/trim.sh` (+ `demos/trim-before.sha256`), listed in README.

(The rest of this report is completed at the end.)

## Result
`demos/trim.sh` PASSES (run with PLUMBLINE_BIN=$PWD/target/debug; proved red by changing a hand figure: it said
`FAIL: Soffit: by hand 40.00 and none, the program said 39.00 and none`).

| hand figure (STATUS) | program said |
|---|---|
| eaves 26.00 LF each, rakes 10.0623 LF each | `26 LF`, `10.0623 LF` (x6) |
| overhang 9 in (0.75 ft) at every eave and rake | `overhang ~0.75 ft, the outside face of wall-...` (x6), Assumed `~` |
| fascia 52.00 LF; rake fascia 40.25 LF | 52.00; 40.25 (4 x 10.0623 = 40.2492) |
| soffit 39.00 SF; rake soffit 30.19 SF | 39.00; 30.19 |
| gutter 52.00 LF | 52.00 |
| corner boards 72.00 LF (2 corners x 9 ft x 4 walls) | `outside corners ~2 EA; corner board ~18 LF` x4; 72.00 |
| casing: window 16.00, door 16.33 LF | 16 (`~16 LF`), `~16.3333 LF`; 32.33 together |
| Downspouts: no figure until typed; 4 EA Assumed | `no figure` (sheet), `? downspouts: Downspouts unknown` (show), VIF `"Downspouts" (no figure yet)`; then `~4.00 EA` |
| $: fascia 208.00, rake fascia 161.00, soffit 234.00, rake soffit 181.12, corner boards 252.00, casing 97.00 | 208.00, 161.00, 234.00, 181.12, 252.00, 97.00 |
| L: 90.00 LF; inside-corner walls 1 corner each | 90.00; walls 3 and 4 `~1 EA`, `~9 LF`; the other four `~2 EA` |
| roof alone: overhang Unknown, says why; soffit no figure | `overhang ? no figure (no wall is found under this edge: none lies along it within 4 ft with no other wall between)`; Soffit `no figure` |
| sample houses unchanged | sha256 of the four sample job files and their four estimates: identical (demos/trim-before.sha256, taken with the program built at cfd5c32) |

No hand figure was wrong; none was bent.

## Tests
`cargo test -q -j2 --no-fail-fast -p plumbline-world -p plumbline -p plumbline-ifc -p plumbline-sheets`: 513 passed, 0 failed
(10 new in crates/world/tests/call185.rs). `cargo check -p plumbline-app`: clean (no arm needed anywhere in crates/app).

## Expectations moved (all from the library growing; file, line, old -> new)
- crates/cli/tests/cli.rs:170  `set: 40` -> `set: 41` (assemblies; Rake trim is new)
- crates/cli/tests/cli.rs:202  `the same as the job already has: 191` -> `200` (lines; 9 new)
- crates/cli/tests/cli.rs:229  `2 cost lines of the assembly it had came off` -> `3` (lap siding -> brick: "Corner boards" is now a third
  line of the lap-siding assembly that brick does not have: Lap siding, Corner boards, Hang the siding). THIS ONE IS A CONSEQUENCE OF PUTTING
  CORNER BOARDS INSIDE THE EXISTING SIDING ASSEMBLIES, not a bare count: see "choices".
- demos/library.sh header, 39 (`== 40` -> 41, `== 191` -> 200), 69, 71 (`set: 40` -> `set: 41`); labor lines stay 28
- demos/shingle.sh:42, demos/framing.sh:39, demos/joists.sh:148, demos/foundation.sh:40: `set: 40` -> `set: 41`
- demos/pricew.sh (a WINDOW demonstration, NOT RUN, edited by reading only): `assemblies == 40` -> 41 and its message.
  Window demonstrations were not run; any that lists the lap-siding assembly's lines may also show "Corner boards" (pricew.sh's
  brick-after-lap list of the shared lines is unchanged: corner boards are not shared with brick).
Nothing else moved. All 46 command-line demonstrations I could run pass (every demos/*.sh not ending in w.sh, except d2.sh and house.sh which open
the window, window.sh the kit, and field-job.sh which is a helper that needs an argument), after these edits.

## What the saved job gains (also in SPEC.md 5.6)
Nothing for any job saved before, and nothing for a job in which no cost line names `overhang`, `outside_corners`, `corner_board_length` or
`casing_length`: each figure is held in the element's `pl::measure` ONLY on an element a line names it on (the way `net_length` is). So the sample
houses and every old job open with nothing reported stale and save the same bytes (tested; and checksummed in the demonstration). No new key, no new
component. A job with such a line carries the figure in `pl::measure` like any other (value, unit, status, formula, inputs, note). The rake is
already a thing the job holds (`Kind::Rake`, a Run on one plane with a `true_length`, made when the roof is solved): no job gains a rake.

## Choices the decision did not make for me
1. Given only where a line names the figure (the alternative, always, would have added figures to every job's measures and made every old job "stale").
   Because of it, the demonstration's step 1 reads the figures through a new verb `plumbline trim <file>` (reads `trim::figures`, the same arithmetic;
   changes nothing; listed in the usage text).
2. Where the starter lines live: I put Fascia, Soffit and Downspouts INTO the existing assembly "Gutter" (Goes on Eave); a new assembly "Rake trim"
   (Goes on Rake) with Rake fascia and Rake soffit; "Corner boards" INTO both lap-siding wall assemblies (not brick, stone-only or stucco: masonry has no
   boards, as finish.rs says); "Exterior casing" INTO "Window" and "Entry door" (not "Interior door"). Existing "Casing and trim at the window/door"
   (by the each) is left as it was; a person applying Window or Entry door gets both lines, and may take either off. The alternative (separate new
   assemblies) would have moved no existing assembly's line count; the cost is that nothing then says "on an outside wall's assemblies with siding".
3. Downspouts is Division 7 (sheet metal, with the gutter beside it); the call said Division 6 unless said. Everything else is 6.
4. Downspouts' "no quantity until typed" is call 159's way: quantity `downspouts`, Figure 1 `downspouts = unknown EA`, Unknown, "how many is the job's to say:
   give the count"; typed with `set <eave> downspouts "4 EA" --line Downspouts --status Assumed --source ...`.
5. Wall under an edge: `carries::edge_on` (A2), over the walls of every level; of several, the most shared length, then the highest top, then the lowest
   path. An edge inside the wall's outside face has no overhang (it says by how much); exactly flush is 0.
6. Corner: two walls of one level ending within half their thicknesses added + 0.05 ft; straight (within 1 degree), tee and free end are no corner; a ring
   that is not closed, or 3 walls ending at one point, makes the figure Unknown with the reason; a ring wholly inside another (a closet) has 0.
7. Casing is the opening's own width and height (its type's), not net of anything.
8. Status: the worst of the edge/wall lines (and wall type) it rests on; for corners, of every wall in the ring and the wall's height.

## What is not right / not done
- An eave or rake of a roof with no Footprint (planes entered by hand, as in the sample house) has no overhang: "its roof has no footprint on the plan to
  find the wall under it by". I did not fall back to the plane's own outline.
- The corner rule uses wall END points (within the tolerance above); walls overshooting a corner or drawn a little short leave the ring "not closed":
  Unknown, said, never a guess. Not tried on a house drawn in the window.
- A figure given only where named means `show`/`takeoff` of an unpriced house do not list overhangs; `plumbline trim` does.
- `demos/trim-before.sha256` is a golden: any later change to the four sample houses' files or estimates breaks step 6 (that is its job, but it must be
  re-taken on purpose).
- Window demonstrations were not run; pricew.sh's expectation edit is unverified.
- Tests: the first red run used a skeleton `trim::figures` (returns nothing) so the tests could compile; every one failed for the reasons quoted above.
- PORTMAP.md untouched: no new public item in building* or editing* (trim.rs is top-level, ours). BIMCraft searched (all crates): `overhang` appears only as a roof
  parameter in `crates/core/src/commands.rs` (a roof's overhang past the walls, mm); nothing for fascia, soffit, corner board, casing, downspout, outside corner.

## Gates
tools/gate-language.sh, gate-egress.sh, gate-clientdata.sh --all: PASS. plumbline/tools/licenses.sh: exit 0. BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh: PASS
(324 public items). Patch: /tmp/claude-0/-home-user-roofnerd/ca6a484f-c0e5-523e-99ce-c9d574b09f03/scratchpad/review/call185.patch
No `cargo clean` was needed.
