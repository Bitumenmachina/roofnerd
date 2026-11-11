# Status

Each claim gives its scope, its evidence, and what remains. Evidence is output from a
command anyone can run again from the repository root.

Workspace this was last run in: Linux, rustc 1.97.1 (installed for the 3D window; the
world crate and the command line also build on 1.97.0), no display: the window was run on
a virtual one with a software renderer. On the owner's go-ahead
(QUESTIONS.md Q2) the repository is pushed, as a `plumbline/` folder with its history, to
a working branch of the public repository of his earlier program. It arrived here as a
git bundle; on arrival `cargo test` (27
passed), `demos/d1.sh`, `tools/licenses.sh` and `tools/licenses.sh --self-test` were each
run again and each gave what this file said they gave.

| # | Deliverable | State |
|---|---|---|
| D0 | Landscape decision | Demonstrated, with limits below. Judged twice. |
| D1 | World data model and SPEC.md | Demonstrated at the command line; no window exists yet. Judged five times. Every judgment has found a new way to a wrong figure marked Known, each next to the last fix. All found are fixed. **The fifth round's fixes have not been judged.** |
| D1 amendment | Units (owner ruling) | Demonstrated at the command line. Judged twice: the five demonstrations held both times; two entry forms misread in the first are fixed and held in the second. |
| D2 | 3D roof slice | **A working 3D prototype, not the whole deliverable.** In a window: a roof solved from its footprint, a plane picked with the mouse, its pitch changed and its eave moved with the lines and totals following, a surface given its condition, and the sheet in a second window updating live. The windows were drawn again on 2026-11-11 after the owner said the first looked cheap and tacky (DECISIONS.md call 25). Not built: tracing a calibrated plan, a dormer, a parapet and tapered crickets in 3D, dragging with the mouse, editing in the sheet. Judged once: the solver's areas and lengths held on every ordinary roof; it found stale planes left Known, an eave moved through the far wall, and an estimator's entries moving to another slope, among others. All fixed. **Those fixes are not judged.** See "D2. What was run". |
| — | The whole building (owner, 2026-11-11) | **A working model of a whole house on BIMCraft's shapes, read and walked, not yet drawn in the window.** The owner widened the brief from the envelope to the building and its services (DECISIONS.md call 26). The model was first written from nothing, then, the same day, re-based on BIMCraft's (call 29; `HANDOFF_REBASE.md`): storeys, walls of a type with a top held to a storey or a height, doors and windows of a type placed by their centre, floors, rooms found from their walls, straight runs of a type joined by fittings, equipment and fixtures with connectors, systems, and a network that says what is joined and what is open. Each value flagged; a takeoff by system. In the window: the whole house under a sky, seen through, a system or a storey at a time, on plan, and on foot inside. Judged twice: see "The whole building" and "The re-base on BIMCraft". |
| — | Drawing and editing (the owner's loop, stretch b) | **Demonstrated at the command line and in the window (`demos/draw.sh`, `demos/draww.sh`). Not judged.** From an empty job: a storey, walls, doors, windows, ducts, pipes, conduit, cable, fixtures and equipment drawn with verbs at the command line or with the pointer and typed lengths on the window's plan, snapped; moved and removed; undone and redone. Everything drawn is flagged. Not built: drawing a floor anywhere; drawing a room or a roof in the window (both are at the command line); calling a drawn value Known in the window. See "Drawing and editing. What was run". |
| D3 | Derived conditions and stretch-out | Demonstrated at the command line, except the reproduction of the owner's copper estimate, which is **pending**: no estimate is in `refs/` (Q5). Judged once: every stated figure held, and it found a negative count saved as zero, among others. All found are fixed. **Those fixes have not been judged.** |
| D4 | Macro to micro | **Demonstrated at the command line and in the window, and judged (tenth judgment, below): thirteen findings, twelve fixed with tests, one minor one left and said. The fixes have not themselves been judged.** An element takes an allowance in $/SF or $/LF; a cost line's quantity is extended by its unit cost; an element's detail is rolled back up to the same $/SF or $/LF; the two are reconciled by element and for the job, each total saying what it leaves out and how much of it rests on an assumption. Prices are entered at the command line or typed in the window. See "D4. What was run". |
| D5 | Estimate sheet and money | **Demonstrated at the command line and in the window (`demos/d5.sh`, `demos/d5w.sh`), and judged (eleventh judgment, below): fourteen findings, each fixed with a test. The fixes have not themselves been judged.** A cost line has a class and lands in one column and one row of the recap; in-house work is a production rate into hours; a labor line can be switched off; a material is an element with a generic price and suppliers' quotes, and changing what an element is made of reprices the lines made of it; overhead and profit are entered and have no default. See "D5. What was run". Reproducing the owner's whole-job estimate is pending: there is no estimate in `refs/` (Q5). See "D5. How its demonstration will be run". |
| — | Fittings, connections and clashes (the owner's loop, stretch c) | **Demonstrated at the command line and in the window (`demos/fit.sh`, `demos/clash.sh`, `demos/clashw.sh`). Not judged.** A fitting is worked out from the runs that meet (elbow, tee, cross, transition, coupling) and a branch on a run's side cuts the run; a run is joined to a fixture or to equipment; the takeoff says what is joined and what is open. Clash detection finds what is in the same space with how far and how sure, lists runs through walls and floors apart, and tests a clearance only when one is asked for. See "Fittings, connections and clashes. What was run". |
| D6 | Interchange and reports | Not started. |
| D7 | Capture | Waiting on the owner's ruling. |

---

## D0. Landscape decision

**Demonstration as the brief states it:** DECISIONS.md and LICENSES.md are complete. No
dependency is linked before its license is confirmed.

**What was run**

1. `tools/landscape.sh > demos/captured/d0-landscape.txt`: for 26 crates, the crates.io registry
   (newest version, minimum compiler, declared license, last publish); for 24 repository
   addresses, a shallow clone (last commit, license files, and how many of the project's
   own CI workflows name a Linux runner). 23 were reachable.
2. For the candidates neither route reached, a web search.
3. `tools/licenses.sh`: every crate the workspace links, with its declared license,
   checked against a permissive list. `tools/licenses.sh --self-test`: the checker's logic
   against 24 expressions.

**What it showed**

* Every appendix candidate is rated for maintenance, license fit and Linux, and has a
  call, in one table in DECISIONS.md.
* Thirteen entries in the starting landscape were wrong, stale or unconfirmed; they are
  tabled at the top of DECISIONS.md. The three that change the plan:
  * OpenTakeoff is now AGPL-3.0-only, not Apache-2.0.
  * A permissive weighted straight skeleton exists (BIMCraft, MIT OR Apache-2.0), which
    closes the gap the brief named.
  * Bevy is at 0.20.1, and the full engine needs a newer compiler than this workspace has.
* `tools/licenses.sh` exits 0: 61 linked crates (63 rows; two are linked in two versions),
  all MIT, Apache-2.0, Zlib, Unicode-3.0 or Unlicense-or-MIT. The self-test passes 24 of
  24, including `GPL-3.0 AND (MIT OR Apache-2.0)`, which fails as it must.

**What remains**

* **"Works on Linux" is proven for one candidate only**: `bevy_ecs`, which builds here.
  For the rest the table reports what their own CI runs on. None of them was built here.
* ProTakeoff and ConstructCAD could not be located, so they have no verified license.
  Neither is used.
* The 3D window call (Bevy 0.20 with bevy_egui) is a decision, not a proof. It has not
  been compiled here. DECISIONS.md call 2 states the proof that opens D2 and what
  reverses the call.
* PDFium's own license, and the licenses of OpenAEC repositories other than
  open-pdf-studio, were not checked. Neither is used.
* The spreadsheet formula engine (IronCalc or our own evaluator) is left to the start of
  D3, with the test that decides it named in DECISIONS.md call 9.
* License data is what each crate declares in the registry. No license text inside the
  linked crates was read.
* "No dependency linked before its license is confirmed" rests on this record: the three
  direct dependencies were looked up in the registry before `Cargo.toml` named them. The
  history has no separate commit that shows the order.

---

## D1. World data model and SPEC.md

**Demonstration as the brief states it:** A saved job is readable JSON with one entity per
element. Changing one pitch changes, in a file diff, only that entity and the components
derived from it. Assumed and Unknown values display as flagged.

**How it is run:** `demos/d1.sh`. It uses the built-in sample job, which is made up.
`cargo test` runs 89 checks on the same model.

The figures below are as rerun after the units amendment and D3. Those added to the
sample job one sheet-metal cost line on the porch headwall and three conditions as recipes,
and every cost line now shows its worked quantity, which moved the count of entries (24
when D1 was first run) and the counts of Known, Assumed and Unknown values (86, 13, 13
before the pitch change and 94, 5, 13 after, when D1 was first run). The pitch change
still touches the same five entries and no others.

**What was run and what it showed** (output of `demos/d1.sh`, exit 0)

1. *One entity per element, readable JSON.* The sample job saves as 27 entries with 27
   distinct paths. `plumbline check` confirms the file loads, its measures match its
   geometry, and saving it back writes the same bytes.
2. *One pitch changed.* `plumbline set after.json plane-0017 pitch 4 --status Known
   --source "field measured"` prints "set plane-0017 pitch = 4 in/12 (Known)" and reports
   measures re-derived on plane-0017, eave-0018, rake-0019, rake-0020, headwall-0021.
3. *The file diff.* Changes fall in exactly those five entries out of 27. On the plane:
   the pitch (3.0 Assumed to 4.0 Known, with its source) and its measure (slope factor
   1.0308 to 1.0541, surface 98.9545 to 101.1929 SF, squares 0.9895 to 1.0119). On the two
   rakes: true length 8.2462 to 8.4327 LF. On the eave and headwall: status only, because
   they are level; their length does not move but it no longer rests on an assumption.
   The script fails if any other entry changes.
4. *Flags.* Before the change `show` reports "91 Known, 16 Assumed, 14 Unknown" and lists
   each flagged value with its reason: the assumed porch pitch and the seven figures that
   follow from it, the unknown parapet height and the face area that cannot be given
   without it, the unpriced cost lines, the unbuilt schedule codes, and the cost codes
   that wait on Q1. After the change: "101 Known, 6 Assumed, 14 Unknown".
5. *A contradiction is flagged, not resolved by guess.* Steepening one plane of the hip
   roof to 8:12 leaves its two hips and the ridge with no length and the note "the planes
   disagree about the height of the line's end: plane-0003 puts it at 20.00 ft; plane-0006
   puts it at 17.50 ft".

**Hand checks** (in `crates/world/tests/world.rs`, `hand_worked_figures`)

* South plane: trapezoid (40 + 10) ÷ 2 × 15 = 375 SF on plan; 6:12 factor √1.25 = 1.1180;
  surface 419.2627 SF.
* Hip: 15 over and 15 up on plan = 21.2132 LF; climbs 7.5 ft; √(450 + 56.25) = 22.5 LF.
* Porch rake: 8 ft run at 3:12 climbs 2 ft; √68 = 8.2462 LF.

**`cargo test`: 89 passed, 0 failed** (79 on the model, 8 on the ported skeleton, which
are BIMCraft's own, and 2 on the command line).

**What remains**

* **The demonstration runs at the command line, not in a window.** There is no desktop
  window yet; that arrives with D2. "Display as flagged" is shown as `~` and `?` marks in
  `plumbline show`.
* **The roof is not re-solved.** Hips, valleys and ridges are entered lines. A pitch change
  they cannot follow is flagged (item 5), which is honest and is not the D2 behaviour the
  brief wants (move with the pitch). SPEC.md section 7.
* **The sample job is the only job this has been run on.** No real job, no reference
  estimate.
* Only pitch, height, eave height, a line's stretch-out, condition and schedule code can
  be edited from the command line. Outlines, eave lines, line ends and cost lines are
  edited in the file.
* The limits listed in SPEC.md section 7, in particular the 0.01 ft fit tolerance (Q9).
* **Rule 4 has been broken by a judge five times**, each time after the previous round's
  fixes. Each judgment found entries that produced a wrong figure marked Known. Treat
  "nothing wrong is shown as Known" as the model's intent backed by 59 tests and five
  attacks, not as proven. Every attack so far has found a case next to the last fix, the
  fifth round's fixes have not been attacked, and the wrong-eave case in particular has
  now been reopened three times, each time by a path that returned before the evidence
  was looked at. The cases are getting more contrived but have not stopped.
* Planes are checked one at a time and each line against the planes it names. Nothing
  checks planes against each other (overlaps, gaps).
* **Process note:** the brief asks that each demonstration be written down before the
  deliverable is built. For D0 and D1 the demonstration was written alongside the work,
  not ahead of it. D2's is written below, ahead.

---

## D1 amendment. Units

**The ruling** (owner, in the handoff after D1; it overrules DECISIONS.md call 15, first
bullet): "Feet only, nothing converted" is rejected. US measurements stay, and the program
takes more than feet. An estimator enters what the drawing or the tape says.

**Demonstration as the ruling states it**, in the application:

1. A parapet height entered as `30 in` gives the same face area as `2.5 ft`.
2. `25' 6"`, `25.5 ft`, `306 in` and `25' 6 1/2"` are each accepted and each mean what
   they say, to the sixteenth.
3. A value is shown and saved in the unit it was entered in, with its status and source,
   and is converted only for arithmetic. The file diff for a unit change shows that value
   and what derives from it, nothing else.
4. A unit the program does not know is still refused by name. It is never read as feet.
5. A stretch-out in inches against a length in feet (the D3 case) works without the
   estimator converting anything.

**How it is run:** `demos/units.sh`, on the sample job. The script exits non-zero if any
step does not do what it says. Its output is kept in `demos/captured/units-demo.txt`.

**What was run and what it showed** (output of `demos/units.sh`, exit 0)

1. `set parapet-0024 height "30 in"` and, on a second copy, `"2.5 ft"`. Both give
   face_area 50 SF on the 20 LF parapet. Read as feet, 30 would have given 600.
2. `25' 6"`, `25.5 ft` and `306 in` each give 510 SF. `25' 6 1/2"` gives 510.8333 SF
   (20 × 306.5 ÷ 12). `25' 6 1/16"` gives 510.1042 SF: one sixteenth of an inch is
   0.1042 SF on 20 LF, and it shows. Each is echoed back as typed and each file passes
   `check`.
3. After `30 in` the file holds `"value": 30.0, "unit": "in"` with its status and source,
   and `show` prints `height 30 in`. Re-entering it as `2.5 ft` changes, in the file diff,
   the parapet's height (value and unit) and the formula of its face area
   (`plan_length × height ÷ 12` becomes `plan_length × height`); the face area's number
   does not move and no other element changes. Re-entering it as `2' 7 1/2"`, Assumed,
   changes the height (`[2.0, 7.5]`, `ft-in`, status, source) and the face area (50 to
   52.5 SF, status, formula) and nothing else.
4. Refused by name at the keyboard: `30` (no unit; "never read as feet"), `762 mm` and
   `0.76 m` (called metric), `3 cubits`, `1 yd`, and a pitch of `14 deg`. After all six
   the job file is byte-for-byte unchanged. Typed straight into the file: a height in
   `mm`, in `furlong`, and with its unit removed are each refused on load.
5. The sample porch headwall is 12 LF and carries a copper flashing line with a 24 in
   stretch-out. Its sheet area is 24 SF, saved with the formula
   `true_length × stretch_out ÷ 12`. The brief's stretch-outs on the same 12 LF: 12 in
   gives 12 SF, 8 in gives 8, 36 in gives 36, 24 in gives 24; `19 1/2"` gives 19.5; `2 ft`
   gives 24. Changing 24 in to 36 in changes, in the file diff, that line's stretch-out
   and its area on the headwall, and nothing else.

**Hand checks** (in `crates/world/tests/world.rs`): every sixteenth from `25' 6"` to
`25' 6 15/16"` is exactly its own number of inches ÷ 12, equals the same length typed in
inches, and is shown back as typed; 28 other spellings (`25'-6"`, `25 ft 6 in`, `6-1/2"`,
pasted ′ ″ ½) each give the stated number of feet; 23 malformed entries are each refused
with the stated reason.

**What remains**

* **Judged once** (third judgment, below). The five demonstrations held. Two entry forms
  were read as something other than what an estimator means (a pitch written as a bare
  fraction; a minus on the inches) and are fixed.
* A pitch is shown as a decimal rise (`1/4:12` shows as `0.25 in/12`). Same unit, not the
  same spelling.
* Sheets, pounds, waste and the conditions a parapet produces came with D3, below.
* **Outlines and line ends in inches are loaded from the file only.** There is no command
  that enters them. They are covered by a test, not by the demonstration script.
* Metric, and pitch in degrees or percent, are refused, not converted (PROPOSALS.md 5, 6).
* The reference copper estimate is not in `refs/` (Q5). The four stretch-outs are the
  brief's; the lengths they are applied to here are the sample job's.

---

## Independent judgment

A separate agent (Opus) that built none of this judges each deliverable: it reruns every
command above, then uses the program as an estimator would and tries to break it.

### First judgment: D1's narrow demonstration held; the model around it did not

Every figure in the demonstration checked out against the judge's own arithmetic, and
every license claim it sampled was correct. Outside the demonstration script it found
twelve defects. The serious ones broke rule 4: ordinary hand edits produced wrong numbers
marked Known. All twelve, and what was done:

| # | What the judge found | What was done | Regression test |
|---|---|---|---|
| 1 | Re-ordering an outline's points silently turned the eave into a rake; wrong lengths marked Known | The eave is now its own line on the plane. (Only partly closed by this: see the second judgment) | `reordering_an_outline_does_not_move_the_eave`, `an_eave_that_is_not_level_on_its_plane_is_flagged` |
| 2 | Units were decorative: 30 in read as 30 ft, metres read as feet | Each field takes one unit; anything else is refused. No conversion. (Since overruled by the owner: see "D1 amendment. Units". What is kept is that an unknown unit is refused.) | `a_unit_the_program_does_not_know_is_refused_by_name_never_read_as_feet` (the original test, `a_value_in_the_wrong_unit_is_refused_not_read_as_feet`, was rewritten with the ruling) |
| 3 | A loop in `children` passed loading and hung `show` | The tree is checked on load: one job, first, and everything leads back to it | `a_loop_of_children_is_refused` |
| 4 | Load and save silently dropped unknown fields, header keys, `inherits`; altered some floats | Unknown keys are kept everywhere except inside our own components, where they are refused by name. Numbers round-trip exactly | `nothing_is_dropped_quietly`, `a_coordinate_comes_back_as_the_same_number` |
| 5 | Deleting a component made its Unknowns vanish from the list | Every quantity is always listed; a missing input makes it Unknown with the reason | `a_missing_component_shows_as_unknown_instead_of_vanishing` |
| 6 | The license checker passed `GPL-3.0 AND (MIT OR Apache-2.0)` | Replaced with a real expression parser and a self-test | `tools/licenses.sh --self-test` |
| 7 | A pitch of 1e300 gave a Known infinity | Overflow gives Unknown; pitch above 240 in 12 and coordinates beyond 1,000,000 ft are refused | `numbers_too_large_to_mean_anything_are_refused` |
| 8 | Bowtie and collinear outlines, a ridge on a plane it does not touch, a hip naming one plane, a hip used as a plane: all accepted | Refused on load or flagged with no length | `outlines_and_lines_that_make_no_sense_are_refused_or_flagged` |
| 9 | SPEC claimed checks the code did not make; some values carry no status | SPEC rewritten against the code; the job-element checks added; what counts as a value is now stated (SPEC 2), with references and labels checked instead | `a_loop_of_children_is_refused` |
| 10 | Cost lines accepted anything | A line needs a description, a measure its element has, that measure's unit, and a non-negative price. Tying line status to quantity status is D5 | `cost_lines_have_to_make_sense` |
| 11 | `show \| head` panicked; repeated `--status` took the last; `set` hid stale measures; two siblings could share a name; `-0` stored | Each fixed. The tolerance point is now Q9. The judge's note about a BIMCraft tolerance was copied into DECISIONS.md unchecked and was wrong (second judgment, item 6) | `two_children_of_one_parent_cannot_share_a_name` and others |
| 12 | D0 evidence could not be re-run; no Linux rating per candidate | `tools/landscape.sh` and `demos/captured/d0-landscape.txt`; a rating table in DECISIONS.md | re-run the script |

Not changed, and why: a line's `planes` list still carries no status of its own (SPEC 2
says so and says why); the fit tolerance stays at 0.01 ft pending Q9.

### Second judgment: the twelve are closed or partly closed; five new defects, three of them rule 4 again

A fresh agent reran every command (all figures matched; the landscape script reproduced
the evidence file byte for byte), repeated each first-round attack, and attacked the fixes.
Nine of the twelve were closed outright. Three were partly closed, and it found new ones.
No hang, no panic, and no status better than its worst input were found.

| # | What the second judge found | What was done | Regression test |
|---|---|---|---|
| 1 | An eave entered on a stepped-back or inside edge folded the plane: a rake read 8.9443 LF, Known, where 11.3137 is right | The eave has to be the plane's low edge: an outline with points on both sides of it is refused | `the_eave_has_to_be_the_low_edge_of_its_plane` |
| 2 | A 0.028 ft eave entry at 45° tilted the whole plane; lengths Known and wrong | The slope is measured from the outline edge the eave lies along, not from the eave line as typed | `a_crooked_sliver_of_an_eave_cannot_tilt_the_plane` |
| 1c | An eave entered on the rake edge flagged the eave but left the rakes at 8 LF Known | A line's kind now says how it slopes: eave, ridge and headwall are level; rake, hip and valley climb. All four porch lines are flagged in that case | `an_eave_that_is_not_level_on_its_plane_is_flagged` |
| 3 | The same key twice inside an element's attributes: the last one won silently and the other was deleted on the next save | The file is parsed strictly; a repeated key anywhere is refused | `the_same_key_twice_is_refused_anywhere` |
| 4 | A line with both ends on the outline but running outside the plane got a Known length | A line has to run along the outline: ends and three points between are tested | `a_line_has_to_run_along_its_plane_not_just_touch_it` |
| 5 | A lowered `next_id` made `Job::add` mint a path already in use | Refused on load; `add` takes the next number past every one in use | `the_path_counter_cannot_hand_out_a_path_twice` |
| 6 | DECISIONS.md call 3 misstated a BIMCraft tolerance | Corrected, with the history of the error. Checked against the source this time | |
| 7, 8 | Two rating-table cells disagreed with the evidence file; "62 crates" was 61 crates in 63 rows; lists of scripts and of accepted licenses were stale | Corrected | |
| 9 | SPEC said a parapet names no plane but the code allowed it; several checks the code makes were not in SPEC | A parapet naming a plane is refused; SPEC's list of refusals rewritten from the code | `a_parapet_names_no_plane` |
| 10 | `MIT WITH GPL-3.0` passed the license checker; a parapet height over the limit got the wrong message; a unit cost of 1e308 was accepted; `set` did not say what unit it applied | Each fixed | `tools/licenses.sh --self-test` (24 cases) |

Left as the judge found it, and recorded in SPEC.md section 7: an eave entered on the edge
parallel to the true one passes (every length is still right, but the plane's heights run
the wrong way); two identical cost lines on one element are accepted.

These second-round fixes were judged in the third judgment, below.

### Third judgment: the claims held, the second round is closed but for one, and rule 4 broke four more ways

A fresh agent (Opus), working in its own clone at the commit that added units, reran
every command, checked every figure in this file against its own arithmetic, repeated the
second judgment's attacks, attacked those fixes, and attacked the units change with about
ninety ways of writing a length. It was not asked to judge the lines-from-planes
groundwork, which did not exist when it started.

**What held.** Every figure stated above for D1 and for the units amendment. All five of
the ruling's demonstrations, rerun with the judge's own values. Second-judgment items 2,
3, 5, 7/8, 9 and 10, each attacked again. A sample of the first judgment's twelve; none
reopened. Outlines, eaves and line ends in inches against feet in every mix. Limits
applied after conversion. Every malformed feet-and-inches pair it tried.

**What it found, and what was done.** Each fix was checked by rerunning the judge's own
reproduction as well as by the test named.

| # | What the third judge found | What was done | Regression test |
|---|---|---|---|
| J3-1 | **Wrong figure, Known.** A plane with its eave entered on the wrong edge still gave its rake a length (11.4564 LF where 12.2474 is right) when the rake climbs under both readings. The eave line on the same plane was flagged; the flag did not reach the rake. Second-judgment item 1c was only partly closed | One line whose kind contradicts a plane's eave now takes the length off every line on that plane | `a_line_that_shows_the_eave_is_on_the_wrong_edge_takes_the_length_off_every_line_on_that_plane` |
| J3-2 | **Wrong figure, Known.** A line drawn across a 5 ft notch got its full 30 LF: the five points tested all landed on edges. SPEC had listed this as a limit; the judge turned it into a number | The outline's edges have to cover the line end to end. No sampling | `a_line_across_a_notch_does_not_run_along_the_outline_however_the_gap_falls` |
| J3-3 | **Misread, Known.** A pitch typed `3/4` was stored as 0.75 in 12 while `6/12` was stored as 6 in 12. Read as a ratio, 3/4 is 9:12 | A bare fraction is refused as a pitch and told to carry its run (`3/4:12`). Only `a/12` is read without more | `a_bare_fraction_is_not_read_as_a_pitch_and_a_minus_on_the_inches_is_not_dropped` |
| J3-4 | **Misread, Known.** `0' -6"` was stored as plus 6 in: the minus was taken for the dash in `25'-6"` | A dash is a separator only hard against the feet mark or spaced both sides; otherwise refused | same |
| J3-5 | **Panic.** An argument that is not valid text crashed the command line. The file was not touched | Refused with a message | `an_argument_that_is_not_text_is_refused_not_a_panic` |
| J3-6 | DECISIONS call 16 and SPEC said a reader multiplying the figures on the page gets the figure on the page. Not so: figures are worked from unrounded inputs (375 × 1.118 is 419.25; the page shows 419.2627) | Both documents corrected. The arithmetic is unchanged | |
| J3-7 | Cost lines differing only in capitals or a trailing space were accepted as different lines, so one flashing could be priced twice | Descriptions are compared ignoring capitals and spacing; a description may not start or end with a space | `nothing_is_silently_zero_or_silently_unread` |
| J3-8 | This file cited a test that had been renamed; SPEC said `FT` is refused, which is true in the file and not at the keyboard | Both corrected | |
| J3-9 | An eave spanning an edge traced with a point part-way along it was refused | Points on the straight line between their neighbours are passed over when finding the eave's edge | `an_edge_traced_with_a_point_part_way_along_it_is_still_one_edge` |
| J3-10 | A minus zero in the file showed as `-0`; a number too small to hold was stored as `0 ft` Known; a refusal printed a 0.0108 ft miss as "0.01 ft" | Each fixed. Left: a pitch typed `1/4:12` is shown as `0.25 in/12` | `nothing_is_silently_zero_or_silently_unread` |
| J3-11 | A pitch under the mis-cased key `pl::Pitch` was kept as a foreign attribute and the plane said it had no pitch | Any key under `pl::` that is not one of this program's components is refused, naming the one it resembles | same |
| suspected, not run by the judge | A very small stretch-out would round to `0 SF`, Known | Confirmed and fixed: a derived figure that is not zero but would be written 0.0000 is Unknown with the reason | same |

**What the third judge did not do:** rerun the D0 landscape script; check second-judgment
item 6 (the BIMCraft tolerance note) against its source; check DECISIONS' rating-table
cells; large jobs; a valley with both of its planes mis-entered.

**Left as found:** a plane whose eave is on the wrong edge is caught only when some line
entered on it contradicts it. With only a rake entered, nothing does (SPEC.md section 7).

### Fourth judgment: J3-1 reopened two ways; `lines` said things that were not so

A fresh agent (Opus), in its own clone at the commit that fixed the third round, attacked
those fixes and the lines-from-planes groundwork, which no judge had seen. It did not see
D3, which was not built when it started.

**What held.** Every spot-checked claim in this file. J3-2 in substance, J3-3 (66 pitch
spellings), J3-4 (60 length spellings), J3-5, J3-8, J3-10, and the rounds-to-zero rule
(real zeros stay Known). `lines` on a hip roof with unequal pitches, an L-shaped hip roof
with a valley, mixed inches and feet, and three planes on one line. No panic or hang.

**What it found, and what was done.** Each fix was rerun against the judge's own files.

| # | What the fourth judge found | What was done | Regression test |
|---|---|---|---|
| J4-1 | **Wrong figure, Known.** J3-1's rule lost its evidence when the contradicting line sat on two planes that also disagreed about its height: the line was refused for the disagreement before its slope was looked at, so a rake on the wrong-eave plane kept 16.0078 LF where 17.5 is right | A line's slope is tested on every plane it names before the planes are compared | `evidence_of_a_wrong_eave_is_not_lost_behind_another_fault_or_another_plane` |
| J4-2 | **Wrong figure, Known.** With both planes of a ridge wrong, only the plane the ridge named first was caught | Every plane a line shows wrong is recorded | same |
| J4-8 | **Wrong figure, Known** (contrived). A hundred notches each 0.009 ft wide under a 10 ft rake passed: each gap was inside the tolerance | The uncovered length is added up and the total held to the tolerance | `gaps_under_a_line_add_up_and_bends_in_an_eave_do_not` |
| J4-7 | An eave was accepted across an edge with a 1 ft sag in it, traced as 400 points: J3-9's fix dropped points one at a time and the bends added up. No wrong length came of it | A run of edges is one edge only if every corner in it is within the tolerance of the one line from its start to its end | same |
| J4-9 | J3-11's neighbours: `pI::pitch`, `pl;;pitch`, ` pl::pitch` and the like were still kept unread | A key a slip away from one of this program's is refused | `a_mistyped_key_of_ours_and_a_description_with_an_unseen_character_are_refused` |
| J4-10 | J3-7's neighbours: a description with a zero-width space, a Cyrillic о or an ﬂ ligature was a different line | Unseen characters are refused. **Look-alike letters are not caught** (SPEC.md section 7) | same |
| J4-11 | Common pitch spellings refused (`1/4"/ft`, `6 in./12`); `6''` refused with a misleading reason | Read | `more_ways_a_drawing_writes_a_pitch_and_an_inch` |
| J4-12 | `-0` shown for a minus-zero eave height; a 5 ft gap reported as "0.0100 ft off"; `25' 0 1/2"` shown as `25' 1/2"`; extra arguments ignored | Each fixed. Left: a file starting with a byte-order mark is refused as unreadable; a control character is accepted in a source; finding an eave's edge is slow on outlines of thousands of points (1.2 s at 4,000) | same |
| J4-3 | **`lines` said what was not so.** With a shared edge traced 0.009 ft out of line, a valley was reported as a rake, or twice, depending on which plane came first in the file | Two edges share a stretch between corners, whichever is asked; each shared stretch is said once | `what_lines_says_does_not_depend_on_which_plane_comes_first` |
| J4-4 | The same plane entered twice gave a "valley" along its eave | Planes on the same side of a shared edge are reported as an overlap and give no line | `lines_says_what_is_so_about_overlaps_steps_and_lines_entered_twice` |
| J4-5 | A correct entered line was reported as "no edge of any plane runs there" when a wrong one was entered before it on the same edge; so were a duplicate and a part-length line | Exact matches are taken first; the rest are told what they are (wrong kind, entered twice, part of a line) | same |
| J4-6 | A stepped roof was reported as planes that do not fit; one plane traced in two pieces, and two flat areas, as "a change of pitch" | Said as what they are; the heading no longer calls them misfits | same |

**What the fourth judge did not do:** a dormer with side walls; large jobs beyond a timing
check; corner snapping to an unrelated plane's corner. Suspected and not reproduced: a
wrong merge of two found lines.

These fourth-round fixes were judged in the fifth judgment, below.

### Fifth judgment: D3's figures held; a negative count was a silent zero; J4-1 reopened again

A fresh agent (Opus), in its own clone, judged D3 and the fourth round's fixes. For D3 it
reran the demonstration, then ran it again on a parapet and three recipes of its own: a
diagonal parapet 32.5959 LF long entered in inches, 3' 4 1/2" high, with a coping at
1' 7 1/2" and screws by `(plan_length - 2 * end_gap) * sides / spacing`; a rake recipe;
a roof-plane recipe ordering shingles by the bundle and underlayment by the roll.

**What held.** Every figure stated for D3 above, and every step of the judge's own lines
worked by hand (120.8765 SF, 52.9683 SF, 56.9409 with waste, 3 sheets, 65.1918 LF, 49
screws, and the rest). Twenty-five formulas chosen to trip the bracketing of the worked
formula: each saved text gives its figure. Dimension refusals. Unknown anywhere leaves no
number after it, never a zero. Ten sequences of edits trying to leave a price in one unit
on a quantity in another: none got through. Forged or missing line figures in the file.
J4-2, J4-7 and J4-8, each attacked again.

**What it found, and what was done.** Each fix was rerun against the judge's own files.

| # | What the fifth judge found | What was done | Regression test |
|---|---|---|---|
| J5-1 | **A silent zero, Known.** A count formula that comes out below zero (a spacing typed `-12 in`, or a formula that takes off more than there is) was rounded "up" to `0 EA` and shown Known. The same formula in LF was refused | Below zero is refused before rounding. A count above zero but under a billionth is refused as too small to tell from zero. A count with waste is rounded up too | `a_count_below_zero_is_refused_not_rounded_up_to_nothing` |
| J5-2 | **Wrong figure, Known. J4-1 reopened.** The evidence of a wrong eave was still lost when the contradicting line was refused early for its other plane (no pitch, not in the job, or traced 0.05 ft off) | Nothing about a line is decided until every plane it names has been looked at | `a_line_refused_for_one_plane_is_still_evidence_about_another` |
| J5-3 | **Lost without a word.** Applying a second recipe wrote over the element's condition name and its source | Refused: an element is one condition | `an_element_is_one_condition_and_a_recipe_carries_no_stretch_out` |
| J5-4 | SPEC and DECISIONS said a recipe leaves stretch-out, waste and prices blank. Nothing enforced it, and `show` hid a recipe's stated values | A stretch-out in a recipe is refused. Other stated values are allowed, copied with their status and source, and shown. Both documents corrected | same |
| J5-5 | **Crash.** A formula of about 2,500 terms ran every command out of stack | A formula holds at most 200 tokens | `a_formula_too_long_to_read_is_refused_and_a_factor_is_not_a_count` |
| J5-6 | `--with spacing=12` (no unit) made the spacing a plain number and the fastener line a length, with no message | Cannot be refused, since a new name is whatever is typed. A change in what a line comes out in, or is priced in, is now said | |
| J5-7 | J4-9's fix refused other programs' keys (`ex::cost`, `my::job`) and called them this program's | Narrowed to real mistypings of this program's keys; the message says what is so | same as J5-5 |
| smaller | The minus in a saved formula could not be typed back; `slope_factor * 1` gave `2 EA`; `--with` twice for one name; a count with waste showed `53.9 EA`; an order unit with a space in it; no way to say a divide-form order is not rounded; `lines` said "no edge of any plane" for a line on a plane it had left out; this file said none of an applied recipe's values is Assumed when its cost code is | Each fixed | same, and `a_count_below_zero…` |
| found while rerunning | **Panic.** `set` and `apply` crashed when their output was piped to a reader that stopped (`| head -1`), which the first judgment had found on `show` | Every command tolerates it | `a_reader_that_stops_listening_is_not_a_panic` |

**Left as found:** a line's figure is worked from the measures as shown, so it can differ
from the same measure in its last place (`face_area` 110.0111 SF beside a
`plan_length * height` line of 110.0112 SF). SPEC.md section 5.4 says so. A new name
cannot be brought into a formula as Unknown.

**What the fifth judge did not do:** whether a recipe edited after being applied partly
reaches the element; a Condition element carrying cost lines of its own; a dormer.

**These fifth-round fixes have not been judged.**

---

## D2. How its demonstration will be run (written before any D2 work)

Opening proof (DECISIONS.md call 2), on a machine with a display and rustc 1.97.1 or newer:
a window showing the sample hip roof; a plane picked with the mouse; a second OS window on
the same world updating live; a coordinate readout holding 1/16 in at the far corner of a
real footprint. If any of the three fails, the window is rebuilt on eframe and wgpu and
that is logged.

Demonstration, in the application, recorded as a screen capture plus the saved-job diff
for each step:

1. Load a plan image, calibrate it from one known dimension, trace a footprint.
2. Set pitches per edge; the skeleton produces planes, hips, valleys and a ridge.
3. Change one pitch in the 3D view: the ridge moves, the lines and the total update.
4. Drag an eave: same.
5. Add a dormer: new planes and valleys appear with their lines.
6. Edit a line's dimension in the sheet: the 3D model follows.
7. Click a surface and assign its condition.
8. Trace a parapet and a low-slope area with tapered crickets.
9. Pop the sheet out to a second window; repeat step 3 and watch it update.

The judge runs steps 3 through 7 on a footprint of its own choosing.

## D2. What was run and what it showed

**The owner's instruction for this stretch:** make the decisions and get to a working 3D
prototype. The three things that had blocked D2 were his to decide and he handed them
over: the compiler, the system library the window needs, and reading BIMCraft's source.
What was decided is DECISIONS.md call 23.

**How it is run:** `demos/d2.sh`. It drives the real program with a real pointer and
keyboard (`xdotool`) on a virtual display (`Xvfb`) with Mesa's software Vulkan driver, has
the program save a picture of both windows after each step, and then reads the saved job
back with the command line. It exits non-zero if a step fails. The 16 pictures are in
`demos/captured/d2/`. On a desktop the same thing is `plumbline-app <job file>` and a
mouse.

**The windows were drawn again** (2026-11-11). The owner looked at the first pictures and
said the prototype looked cheap and tacky. He was right: it was a dark screen with a block
of monospaced text over it. What was looked at and what was decided is DECISIONS.md call
25. In short: the roof is drawn the way the trade's own measurement reports draw one
(facets labelled with area and pitch, lines coloured by kind with a legend, lengths
written on the lines, a plan view), it sits on a building and a ground with a shadow, the
values on the selection are listed beside it with their flags and sources, and the sheet
is a set of tables. The steps and the figures below are the same session as before, run
again in the new windows.

**What was run and what it showed** (output of `demos/d2.sh`, exit 0; every one of the 16
pictures was opened and read at full size)

1. *A roof from its footprint.* `plumbline roof` on an L-shaped footprint (40 × 20 with a
   20 × 20 wing, 6:12 all round, eaves at 10 ft) writes 6 planes and 14 lines: 6 eaves, 5
   hips, 2 ridges and the valley at the inside corner. Nobody entered a hip or a ridge.
   The file passes `check`.
2. *The windows* (`01-model.png`, `01-sheet.png`). The roof in 3D on its walls, with eaves
   green, hips orange, ridges magenta and the valley blue; the panel beside it lists each
   kind with its colour, its count and its length. Each facet carries its number, area and pitch. Beside the
   view: 1,341.64 SF, 13.42 SQ, predominant pitch 6:12, 6 facets (1,200 SF on plan ×
   1.118034); 160 LF of eave, 75 of hip, 40 of ridge, 15 of valley. The title bar counts
   the job's values by flag. A second operating-system window holds the sheet: the same
   summary, then each plane's pitch and surface and each line's length as rows.
3. *A plane picked with the mouse* (`02`). A click on the south plane outlines it in blue
   and marks its row in the sheet. The inspector lists its four corners from the model to
   the sixteenth (east 100' 0" to 140' 0", north 0' 0" and 10' 0", height 10' 0" and
   15' 0"), every value entered on it with its source, and every figure worked out from
   them with its formula. The status bar reads the pointer's position on the roof (east
   116' 11 5/16", north 2' 1 15/16", height 11' 1": 10 ft plus half of 2.16, to the
   sixteenth).
4. *Its pitch changed in the 3D view* (`03`). Two presses raise it from 6 to 8 in 12. The
   roof is solved again: the ridge moves, a short hip appears where the main ridge now
   meets the wing's at a different height, and the sheet shows 15 lines and a new total
   (1,362.17 SF). Everything that rests on the new pitch turns amber and carries an
   "Assumed" chip, in both windows, and the sheet lists the edge under "To settle" with
   its source.
5. *Its eave moved* (`04`). One press moves the south eave out 6 in; the eave line reads
   (100, -0.5) to (140, -0.5) and the eave is 40' 0" still, the eaves either side 20' 6".
6. *A dimension typed* (`05`, `06`). `P` opens a field over the view; `9:12` typed and
   Enter: the pitch is 9 in 12 and the roof is solved again (283.93 SF on the plane,
   1,393.98 SF in all).
7. *A surface given its condition* (`07`). `C` gives the selected plane "Standing seam
   copper roof". The plane is drawn in copper, its row in the sheet names the condition,
   and the inspector lists the condition and its cost code as Assumed.
8. *The same roof on plan* (`08`). `T` looks straight down: the L, every line with its
   length written on it, every facet with its area and pitch. The sheet, scrolled down,
   lists the plane's two cost lines worked out step by step (283.93 SF each, Assumed; the
   panels' waste, with-waste figure and both prices are red and say unknown, because none
   is entered).
9. *What was left in the saved job.* It passes `check`. Besides the roof and what is
   solved from it, only the job's path counter changed (by 1, for the new hip).
   Footprint edge 1 is 9 in 12, Assumed, source "set in the 3D view; not yet confirmed
   against a drawing". The south edge is at y = -0.5 ft. The plane kept its path through
   four solves and carries the condition and its two lines.

**The window was rebuilt once more the same day**, for the whole building (next section
but one). The eight steps above were run again in it and every picture opened again; the
figures are the same. What reads differently in the pictures: the panels float over the
model under a sky; the switch between 3D and plan is in the browser at the left, beside
"on foot"; the walls under the roof are still the for-the-eye ones, because this job has
no walls of its own.

Seen by hand in the window before that rebuild, and not repeated since: the sample's
hand-entered hip roof, drawn in copper, with an edit key on it refused in a red card; an
edge made a gable on a plane that carried a condition, with an amber card naming what was
lost; the wheel scrolling both panels; the window made smaller.

**Hand checks**
* After the session the south plane is a trapezoid 40 ft at the eave and 15.4 ft at the
  ridge, 8.2 ft deep on plan (the 9:12 slope from y = -0.5 meets the 6:12 slope from
  y = 20 at y = 7.7): (40 + 15.4) ÷ 2 × 8.2 = 227.14 SF, × 1.25 = 283.925 SF. The program
  saved 283.925.
* In `crates/world/tests/world.rs`: a 40 × 30 hip roof solved from its footprint gives
  the planes, hips and ridge of the sample roof that was entered by hand (419.2627 SF,
  22.5 LF, 10 LF); a 60 × 30 roof with one slope at 12:12 puts its ridge at y = 10, 20 LF
  long, with planes of 565.6854 and 894.4272 SF; an L-shaped roof gets one valley of
  15 LF; a gable roof gets a 40 LF ridge and four 11.1803 LF rakes; a footprint with a
  3 in jog, a mile from the origin, still solves and its planes cover it.
* The ported skeleton passes BIMCraft's own eight tests here, including 2,000 random
  footprints.

**The opening proof** (DECISIONS.md call 2): a window showing a roof, a plane picked with
the mouse, a second OS window on the same world updating live. All three are in the
pictures. The fourth, "a coordinate readout holding 1/16 in at the far corner of a real
footprint", is met in part: the corners the inspector lists come from the 64-bit model and
read to the sixteenth; the pointer's own position is as fine as the picture is, and no
real job has been opened.

**What is not built, of the demonstration as the brief states it**
* **Trace a calibrated plan or image.** The footprint is typed as coordinates.
* **A dormer.**
* **A parapet and a low-slope area with tapered crickets**, in the 3D view or from a
  solve. The window draws only the planes and lines of one roof.
* **Dragging.** The pitch and the eave are changed with keys and typed values on the
  selected plane, not by dragging in the view.
* **Editing in the sheet.** The sheet shows; it does not take input. "Editing a line's
  dimension updates the 3D model" is done by typing in the 3D window.
* **Choosing a condition.** `C` gives the first condition in the job that fits the
  element's kind.
* **One roof at a time.** The window shows the roof named on the command line, or the
  first one solved from a footprint. The job's other roofs are not in the scene.
* **A real desktop.** Everything was run on a virtual display with a software renderer.
  It has not been run on a graphics card, on Wayland, or across two monitors.
* **Courtyards.** The skeleton handles them; the solver refuses a slope that closes
  round a hole.

### Sixth judgment: the solver's figures held; what it leaves behind did not

A fresh agent (Opus), in its own clone, solved some forty roofs of its own with every
figure worked by hand first, attacked the edits, and drove the window with a pointer and
keyboard, reading the pictures.

**What held.** Every area and length on rectangles with four different pitches, a
pyramid, a 200 × 2 strip, a triangle, a parallelogram, a trapezoid, a pentagon, a hexagon,
L, T, U, X, Z, H and stepped footprints, sheds and gables, clockwise and anticlockwise, in
inches and in feet, a mile from the origin. Every figure stated in this section. A
re-solve with no change wrote the same bytes on 12 roofs. Every refused edit left the file
byte-identical. In the window: the sheet matched the command line row for row; no crash or
hang from any key sequence; a pitch below zero, above 240, `3/4`, `abc`, and a move with
no unit were each refused with a message.

**What it found, and what was done.** Each fix was rerun on the judge's own cases.

| # | What the sixth judge found | What was done | Regression test |
|---|---|---|---|
| J6-1 | **Wrong figures, Known.** A footprint edited in the file (an edge's pitch struck out) left the earlier solve's planes Known at their old areas, and no command could clear them | What is solved is solved again on every load; a roof that cannot be solved has the geometry taken off its planes and lines and shows `not solved` | `a_footprint_changed_in_the_file_does_not_leave_the_old_roof_standing` |
| J6-2 | **Wrong figures, Known.** An eave moved past the opposite wall was accepted and the roof turned inside out (40 SF, Known) | Refused: the moved edge and the two beside it have to keep pointing the way they did | `an_eave_cannot_be_moved_through_the_far_wall_and_a_footprint_has_to_be_clean` |
| J6-3 | **An invented line, Known.** A corner 1/16 in off straight got a 16.77 LF hip of its own | A footprint corner up to 0.01 ft off straight is refused, with what to do | same |
| J6-4 | **Lost without a word.** A rake's flashing and stretch-out jumped to the other rake when a second gable was made | A line on one plane is named for the footprint edge it stands over | `what_was_entered_on_a_slope_or_a_rake_stays_on_that_slope_or_rake` |
| J6-5 | **Lost without a word.** A point added to the footprint renumbered the edges, and a plane's condition moved to another slope | A plane keeps the edge its eave lies along; the solve says when its number changed | same |
| J6-6 | The window did not say that a removed plane carried cost lines; long messages ran off the window | It says so; messages wrap | |
| J6-7 | Mixed-pitch roofs had creases with no line; a gable beside an inside corner left a wall with no line; the note about either was printed once and kept nowhere | A change of pitch is a line (`PitchBreak`); planes at different heights each keep their edge; notes stay on the roof and `show` and the sheet list them | `every_edge_of_a_solved_roof_has_a_line_or_a_note_that_stays_with_the_roof` |
| J6-8 | A 1/16 in jog failed with "a fault in the program"; a 1/8 in jog made slivers | A footprint edge under 0.02 ft is refused as a slip of the trace | as J6-2 |
| J6-9 | A condition given in the window was saved Known, against what this file said | Assumed | `a_condition_can_be_applied_as_assumed…` |
| J6-10 | A hand-entered plane under a solved roof was not checked; a name clash was not explained | Noted on the roof; the clash says what is in the way. **Overlap is still not checked** | same |
| J6-11 | A pitch of 0.001 dropped 8 of 9 lines | A footprint pitch under 0.01 in 12 is refused as flat | as J6-2 |
| smaller | The pointer readout went stale after an edit; outline faults counted edges from 0; a move replaced the footprint's source; an eave height of 126 in was stored on planes as 10.5 ft | Each fixed | |
| found while fixing | A slope pinched by three gables comes back to touch itself, which the plane model cannot hold | Refused, saying so (as courtyards are) | `random_house_footprints_solve_whole_with_every_edge_lined` |

That last test is new: 300 random stepped footprints with mixed pitches and gables. Every
one that solves has to cover its footprint, give every line a length, leave no plane edge
without a line, and load back as it was saved.

**Left as found:** the sheet's total is the sum of the figures as shown, so it can differ
from the exact figure in its last place (1,341.6406 against 1,341.6408).

**What the sixth judge did not do:** two solved roofs in the window at once; two monitors;
a graphics card.

**What remains**
* **The sixth round's fixes are not judged.**
* **It has only been run on a virtual display with a software renderer**, at a few frames
  a second. It has not been run on a desktop, with a graphics card, or on two monitors.
  The second window is a real second window; "pops out to another monitor" is untested.
* **Wayland is not built in** (it needs a system development package); the window is X11.
* **A value set in the view is Assumed**, and a roof's solved outlines rest on every
  pitch of its footprint, so one edit in the view flags the whole roof Assumed until the
  value is confirmed from the command line.
* **Undo does not exist.** F2 saves over the file.
* The view redraws the whole roof on every change, which is fine for one house and
  untried on anything large.
* `plumbline lines` and the solver agree by construction on solved roofs; hand-entered
  roofs still go through D1's checks only.

---

### Seventh judgment: the redrawn windows were honest about most things and not about twelve

A fresh agent (Opus), working only in scratch copies, drove the redrawn windows in nine
sessions on jobs of its own (no roof, a roof path that names nothing, a hand-entered roof
with Unknown values, eaves at height 0, a sliver plane, edges gabled until one plane was
left), read every picture, and probed the new `view` functions from a crate of its own.

**What held.** No panic in any session. A refused edit left the "Saved" chip right. Every
edit key with nothing selected, on a hand-entered plane, and on the last sloping edge was
refused with a reason. Values set in the window went in as Assumed with the required
source and were listed under "To settle". The walls, fascia and ground could not be
selected and were in no total. Apart from what follows, the two windows agreed with each
other and with the command line on every surface and length compared.

**What it found, and what was done.**

| # | What the seventh judge found | What was done | Regression test |
|---|---|---|---|
| J7-1 | **A Known zero.** A roof with no planes, or a roof path naming nothing, showed "Surface 0.00 SF" in Known ink | A sum over nothing has no figure and is Unknown; the windows say "none". A roof named on the command line has to be a roof in the job | `a_roof_summary_does_not_say_more_than_it_knows` |
| J7-2 | **Known, and wrong.** The predominant pitch was named, Known, while a plane with no pitch was left out of it | No predominant pitch is named when a plane has no pitch or no surface, or when two pitches cover the same surface; the windows say why | same |
| J7-3 | A parapet's length showed as unknown in the sheet (the command line had 20 LF), and the low-slope roof looked clean in the main window with an Unknown height on it | A parapet is one of the roof's lines by its plan length; "To settle" lists every flagged value in the job, grouped by what it rests on | same |
| J7-4 | A failed save was shown with the green dot of success | How an action went is carried as what happened (done, warned, refused), not read back out of its words; a failed save is red | |
| J7-5 | **Rounded away.** A pitch of 0.004 in 12 showed as "0:12" on the roof and "0.004:12" in the inspector; 6.125 showed as 6.12 | A pitch is said by the model to the places it keeps (`view::pitch_said`), everywhere | `a_roof_summary_counts_facets...` |
| J7-6 | **Rounded up.** 0.0005 SF showed as "0.001 SF" | A figure that would print as zero is shown as the model has it, to four places, not to the first place that is not zero | same |
| J7-7 | A long message pushed the sheet wider than its window | The message is clipped to its bar | |
| J7-8 | The red of a line with no length and the red of a ridge could not be told apart | Red is Unknown and nothing else: a ridge is magenta, and a line with no length is drawn broken as well as red | |
| J7-9 | A selected facet's label lost its flag | It keeps a dot on the blue | |
| J7-10 | Line labels on the roof rounded to the whole inch, so a very short line read 0' 0" | Labels are to the sixteenth, like the sheet | |
| J7-11 | The inspector's cost-line cards dropped each value's flag, source and note | Each step carries its chip and, under it, what it rests on | |
| J7-12 | A surface under 0.005 SF was 0 SQ, Known; `figure(-inf)` lost its sign; a job with no roof opened to a blank window that never said so | Too small to say in squares is no figure, not zero; a figure that is not a number says so; a job with nothing in it says so | `a_roof_summary_does_not_say_more_than_it_knows`, `a_roof_summary_counts_facets...` |

**These fixes have not been judged.** The windows they were made in were then rebuilt for
the whole building (next section), and that rebuild was judged next (the eighth judgment, under "The whole building").

## The whole building. What was run and what it showed

**The owner's instruction** (2026-11-11, on seeing the redrawn roof window): think of this
for a whole house with its MEP; see what is out there; build your own; it should feel like
a video game and have the power of BIM. What was looked at and decided is DECISIONS.md
call 26. This is the first stretch of it. It is outside the brief's numbered deliverables
and is not described as one of them.

**How it is run:** `demos/house.sh`. It writes the sample house (`plumbline house`), checks
it, prints its takeoff (`plumbline takeoff`) and checks four of its figures against hand
arithmetic, then drives the real program with a real pointer and keyboard on a virtual
display and has it save a picture of both windows after each of eight steps. It ends by
saving the job from the window and comparing it byte for byte with what was opened. It
exits non-zero if any step fails. The 16 pictures and the takeoff are in
`demos/captured/house/`. It was run again after the eighth judgment's fixes and every
picture opened again.

**The job** is made up: two storeys, 40 ft by 28 ft, under a hip roof solved from its
footprint. 16 walls with 9 doors and 17 windows, 2 floors, 9 systems, 51 runs (15 ducts,
31 pipes, a conduit and 4 cables), 5 pieces of equipment and 22 fixtures. Sizes a designer
would work out are entered Assumed, with the source "sized by rule of thumb; no load
calculation has been done". One hot-water branch has no size at all.

**What was run and what it showed** (output of `demos/house.sh`, exit 0; every picture was
opened and read at full size)

1. *The model.* The file passes `check`: it loads, its measures match its geometry, and
   saving it writes the same bytes.
2. *The takeoff at the command line.* First floor: 8 walls, 210.00 LF, 2,026.00 SF of face
   and 1,793.02 SF less openings, 1,120.00 SF of floor. Supply air: 13 runs, 195.00 LF, in
   four sizes, every size flagged Assumed. Hot water: 8 runs, 50.00 LF; its run with no
   size is on a line of its own, flagged Unknown, with its 9.40 LF.
3. *The house* (`01-model.png`). The whole building under a sky, lit by a sun, with a
   shadow on the ground: walls with their windows and front door cut in them, the roof with
   its lines and facet labels. On the left a browser of storeys, what is on show, and the
   nine systems, each with its colour and its length. On the right what the job comes to,
   storey by storey, then the roof, then what is not yet settled.
4. *Seen through* (`02`). `X` draws the building as glass with its edges kept. The ducts,
   pipes, cables, equipment and fixtures inside are drawn at their own size in their
   system's colour.
5. *One system picked out* (`03`, `04`). A click on "Supply air" in the browser: its ducts
   keep their colour, everything on other systems goes grey, and the inspector gives its
   takeoff. The sheet, on its Mechanical tab, lists the same lines. Then "Hot water", with
   the sheet on Plumbing: the line with no size is red and says Unknown in both windows.
6. *One storey alone* (`05`). The first floor with everything above it lifted off: rooms,
   doors, ducts in the soffit, fixtures. The sheet, on its Building tab, lists each wall's
   length, face, openings and net.
7. *On plan* (`06`, `07`). `T`: a true plan of that storey with what runs in it. A click on
   the plan selects what is under the pointer and the inspector lists its values, each
   with its flag and source, and the status bar reads the pointer's position.
8. *On foot* (`08`). `V`: standing in the living room at eye height, with the sun coming
   through the windows onto the floor and a supply duct overhead.
9. *What was left in the saved job.* It was saved from the window with F2 and is byte for
   byte the file that was opened. Looking changed nothing.

**Hand checks** (each is a test in `crates/world/tests/world.rs`)
* The first floor's south wall: 40 ft by 10 ft is 400 SF. A 3 ft by 6.67 ft door and two
  5 ft by 5 ft windows are 70.01 SF. Net 329.99 SF. A wall with no openings has an opening
  area of 0, Known, and nets its face.
* An opening that runs past the end of its wall, or lies over another, leaves the wall
  with no net area and the reason; one with no width leaves it with no opening area
  either. The gross face stands in each case.
* The first-floor trunk, west: up 3.4 ft, south 8 ft, west 17 ft is 28.4 LF and two bends.
  At 16 in by 8 in it is 4 ft round: 113.6 SF of duct, Assumed because its size is.
  A 6 in round branch 11 ft long is π × 0.5 × 11 = 17.2788 SF.
* Hot water at 1/2 in: two branches of 1.9 LF and two of 2.5 LF, 8.8 LF.
* Every run and every placed thing in the job is on some line of some system's takeoff;
  every wall, floor, run and placed thing is either drawn or listed as not drawn, with
  why. Take a storey's elevation away and its 8 walls and its floor are listed, not drawn.
* The sample house saves and loads to the same bytes with nothing stale. A file that names
  a system that is not in the job does not load.

**What should have been read first, and was not.** BIMCraft, already on this machine and
already rated "borrow", has walls, hosted openings, levels, rooms, runs, fittings with
connectors, a connectivity graph, schedules and IFC4 export. `building.rs` was written
without opening it. DECISIONS.md call 28 sets out what it has, what it lacks (flags and
sources, net wall area, systems as elements, a takeoff by system), and the decision: the
building model is re-based on its shapes before anything more is built on `building.rs`.

**Re-based.** The next session did it: see "The re-base on BIMCraft" below. What follows in
this section, written before it, describes the model as it was first built; where the two
differ the later section is what is true now.

**What is not built**
* **Nothing of the building can be drawn or edited in the window.** It is entered in a
  file or in code, and read, walked and taken off in the window. The roof's edits still
  work.
* **Fittings and connections.** A bend is counted; it is not an element, and nothing
  checks that one run meets the next or that a fixture is actually supplied.
* **Clashes.** A pipe can pass through a duct or a wall and nothing says so.
* **Rooms, structure, stairs, a foundation.**
* **Cost.** No recipe exists for a pipe or a duct. A cost line on one can use its
  `true_length`, `bends` and `surface_area`, and none has been written.
* **IFC in or out** (D6).
* **A real desktop and a real graphics card.** Everything ran on a virtual display with
  a software renderer, at a frame every second or so. Whether it is smooth on real
  hardware has not been seen. The code is written to be (the camera eases; nothing is
  rebuilt on a hover), and that is a claim about the code, not a measurement.

### Eighth judgment: the building's figures held; what could be nested, unsure or uncut did not

A fresh agent (Opus), working only in scratch copies, wrote job files by hand from the
sample house and ran `check`, `show` and `takeoff` on them, and drove the window through
odd jobs and odd key orders, reading the pictures. (Its first run was lost when the
machine restarted; this is the second.)

**What held.** A wall's net area: an Unknown `along` or `sill` makes it Unknown, never
Known; an Assumed width makes it Assumed; an overlap, an opening past the end and one
above the top are each refused with the reason; an opening in inches on a wall in feet
converts. Metric, unitless and not-a-number lengths, a run of one point and a diameter of
zero are refused on load. Stale measures are reported and recomputed. `house` then `check`
gives the same bytes. A sum over nothing reads "none" in the command line and both
windows. The four hand checks in `demos/house.sh` are right. No panic from any key order
or any odd job; each refusal was red. The windows do no arithmetic but for placing things
on the screen.

**What it found, and what was done.**

| # | What the eighth judge found | What was done | Regression test |
|---|---|---|---|
| J8-1 | **Doubled, and Known.** A storey moved under another storey loaded, and the first floor showed 16 walls, 420 LF and two floors, Known | A storey has to be directly under the job; and each wall and floor counts on the nearest storey above it and no other | `a_storey_stands_under_the_job_and_counts_only_what_is_nearest_it` |
| J8-2 | **A flag dropped.** A run only assumed to be on a system was totalled under it as Known | A run's length on a line is no surer than that the run is on the system | `a_takeoff_is_no_surer_than_what_is_on_the_system_and_one_size_is_one_line` |
| J8-3 | **Gone from the windows.** A run whose system is Unknown was listed as "on no system", unflagged, and both windows left that group out altogether | Not knowing the system is not having none: such a run is in a group of its own, "System not known", flagged Unknown, with its length. Both groups are listed in the browser and on every discipline's tab | same |
| J8-4 | A long opening message pushed every figure column off the sheet | A bar holds one line cut to fit; the whole message is in the card over the model. Checked by eye on a file with 40 stale measures | |
| J8-5 | A figure refused for a reason of its own (two openings lying over each other) showed a bare red "unknown"; the reason could be read only by selecting the wall | Every refusal reason is listed under "To settle" | |
| J8-6 | **Gone from the picture.** An opening that could not be cut in its wall was not drawn and was in no list | `placements` lists each one with why, and the windows show the list | `an_opening_that_cannot_be_cut_is_said_and_a_listing_keeps_a_door_under_its_wall` |
| J8-7 | One size was three takeoff lines when typed as `6 in`, `0.5 ft` and `0' 6"`; lines sorted as text, 12 in before 6 in | Runs are grouped by their size as a size and said in inches; lines are in order of size, with no size last | `a_takeoff_is_no_surer_than...` |
| J8-8 | `show` listed every door and window after the last wall, indented as if under it | A flat reading is in tree order: each element, then what is under it | `an_opening_that_cannot_be_cut_is_said...` |

Not confirmed by the judge and not changed: a very short segment in a run adds a bend
(it is a bend; whether so short a one is a slip of the hand is for when runs are drawn in
the window, as it was for traced roofs). On foot with a storey of unknown elevation the eye
stands at the datum.

**These fixes have not been judged.** They were seen in the window once, by hand, on a
job made for it (a pipe on a system marked Unknown, a door moved past the end of its
wall, every measure stripped from the file): the browser and the Mechanical tab list
"System not known, 34.50 LF" with an Unknown chip; "To settle" lists "Net area of south
wall has no figure: door-0012 ends 42.00 ft along a wall 40.00 ft long" and "door-0012 is
not drawn: it runs past the end of wall-0004"; and the sheet keeps its columns under a
message naming over a hundred elements. That picture is not in `demos/captured/`.

## The re-base on BIMCraft. What was run and what it showed

**The task** was the previous session's handoff, `HANDOFF_REBASE.md`: replace the shapes in
`building.rs` with BIMCraft's shapes for the same things, ported with attribution, and
keep on top of them everything this project has that BIMCraft does not. It was done in
the order the handoff gives, with a commit after each step. What each thing became, and
the reading behind it, is DECISIONS.md call 29.

**How it is run:** `tools/portmap.sh`; `cargo test --workspace`; `demos/house.sh`; and the
other four demos, which it must not have moved.

**What was run and what it showed**

1. *The reading came first, and the gate held me to it.* BIMCraft's `network.rs`,
   `runs.rs` and `rooms.rs` were read whole, and the parts of `model.rs`, `walls.rs`,
   `geom.rs`, `family.rs` and `schedule.rs` that call 29 lists by line. The map and the
   decision were committed before any model code (`aa46576`). Over three runs
   `tools/portmap.sh` caught five lines of mine that did not hold the symbol I had named
   (a commit message of that day says four); each was corrected by opening the source.
2. *The port map is green.* `tools/portmap.sh`: "PASS. 136 public items: 73 ported from
   BIMCraft at f202abd85d1c, 63 ours, each with its search."
3. *The three things the handoff listed as not known are settled* (call 29): the ten
   system classifications are **not** IFC's (`IfcDistributionSystemEnum` has 44 values,
   read from the schema's generated source and buildingSMART's own definitions; six of
   ours have a value that says the same thing); BIMCraft joins runs end to end with no
   fitting and warns only where one ends on another's length; a door's and a window's
   size is on `OpeningType`. Whether the ten are Revit's was only partly checked, and
   call 26's sentence that they are is withdrawn for eight of them.
4. *The figures that must not move did not* (handoff section 8). The sample house's
   takeoff before and after, line by line:
   * each storey's walls (8, 210.00 LF), gross face (2,026.00 and 1,816.00 SF), net face
     (1,793.02 and 1,580.53 SF) and floor (1,120.00 SF): unchanged;
   * each system's length: unchanged (supply air 195.00, return air 22.20, refrigerant
     34.50, cold water 69.62, hot water 50.00, sanitary 54.80, vent 12.10, power 98.80,
     lighting 84.00 LF);
   * the count of runs became the count of straight pieces (supply air 13 to 20, return
     air 2 to 3, refrigerant 1 to 4, cold water 12 to 22, hot water 8 to 17, sanitary 9 to
     19, vent 1 to 1, power 3 to 10, lighting 2 to 8), and the windows and the command
     line now say "straight pieces" where they count them;
   * "bends" is gone. Every corner of a duct, a pipe or a conduit is a bend fitting, an
     element, counted on its system by size: the same number as the bends each such line
     had. A cable turns a corner with no fitting, so the twelve bends the two cable lines
     counted became no element (call 29, "as built").
5. *What is new in the model, each with hand-worked figures in a test.*
   * **Types.** A size is entered once. Make the sample's 5 ft window type 6 ft wide and
     both south windows are 30 SF, the wall's openings 80.01 SF and its net 319.99 SF.
   * **A wall held to the storey above.** The first floor's outside walls are as tall as
     the storeys are apart: 10 ft. Raise the second floor a foot and the south wall is
     11 ft and 440 SF with nothing typed on the wall. An assumed elevation makes its area
     Assumed; an unknown one leaves no area and says which elevation.
   * **Rooms.** The living room is 614.7188 SF inside its walls, 640 SF to their centre
     lines and 110.125 LF round; a floor's five rooms come to its 1,120 SF at centre
     lines and 1,057.9532 SF inside the walls. These were worked by hand before the
     program was run and matched on the first run, as did a room with a free-standing box
     of walls in it (360 SF, 384 SF, 96 LF) and a square drawn on its walls' outside
     faces (81 SF, 90.25 SF, 36 LF).
   * **The network.** Supply air is two networks with four open trunk ends and ten places
     a branch meets a trunk with no fitting; cold water is one network, open only where
     the service comes in; the drain and its vent are joined and that is said. None of it
     is made up: the sample's plans are read as drawn, and no tee was added to them.
6. *The house in the window* (`demos/house.sh`, exit 0; 16 pictures in
   `demos/captured/house/`, each opened and read at full size on 2026-11-11 after the last
   change to what they show). The house as opened; seen through; supply air picked out,
   with its pieces, its fittings and what is joined in the inspector and the same lines
   on the sheet's Mechanical tab; hot water, with its unsized branch and its two unsized
   elbows red in both windows; the first floor alone; its plan; a cable picked on the
   plan; on foot in the living room. The sheet's Building tab lists each storey's rooms
   under its walls. The job saved from the window is byte for byte the one opened.
7. *Two faults were found by reading those pictures, and fixed.* (a) With a system picked
   out, each elbow was drawn over with all twelve of its edges and read as a blot at
   every corner; fittings are no longer drawn over. (b) **An older fault, there since the
   building was first drawn:** a rectangular duct was given a mirror, not a turn, for its
   direction (`scene.rs`), so the trunks lying east-west and every rectangular riser were
   not drawn where they are. It was in the committed pictures of the eighth judgment and
   nobody saw it, I think because a selected system's centre lines were drawn over the
   place the ducts should have been. A new elbow hanging in the air over a missing riser
   is what showed it. The trunks, the riser to the attic and the return riser are now
   drawn.
8. *The other demos.* `demos/d1.sh`, `demos/units.sh`, `demos/d3.sh` and `demos/d2.sh`
   each exit 0. The 16 pictures `demos/d2.sh` saved were compared with the ones committed
   before, pixel by pixel: in none does any pixel differ by more than a tenth of full
   scale. One was opened by eye; the others are the pictures already on record.
9. *Licences and gates.* `tools/licenses.sh` exits 0. The repository's three gates
   (language, egress, client data) exit 0.

**What a reader should know that the figures do not say**
* The sample's riser to the attic is entered, attic piece and all, on the first floor. So
  the first floor shown alone shows a duct standing 20 ft up and running south at attic
  height, and its plan shows that piece across the bath. That is the job as entered,
  drawn truthfully; a plan cut by height is not built.
* A fitting sits on the joint and has no length of its own: runs are measured through
  fittings, centre to centre.
* Equipment does not join the runs that meet it (BIMCraft's rule), which is why the two
  supply trunks off one air handler are two networks (PROPOSALS 11).
* Fittings are listed in the order of their names, so "1 in elbow" comes before
  "1/2 in elbow" (PROPOSALS 9).
* A job file saved before the re-base does not load. None existed but the samples.

**What is not built** (each is in PROPOSALS.md 7 to 11, or was already listed above)
* What BIMCraft's model has that was read and not taken: curved walls, a wall's top
  following a roof, openings in a floor, room heights, room separation lines, families,
  an opening rising past its wall's top, project parameters.
* Nothing of the building can be drawn or edited in the window. No clash check. No cost
  recipe for a pipe, a fitting or a room. No IFC in or out. No sizing or code check.
* A real desktop and a real graphics card: everything still ran on a virtual display
  with a software renderer.

### Ninth judgment: the hand figures held; an assumed height could decide a room's area unflagged, and the map said things that were not so

A fresh agent (Opus), working only in a scratch copy, built some sixty job files by script
and ran `show` and `takeoff` on them, worked each expected figure by hand, read BIMCraft's
source beside each ported function, and ran the searches `PORTMAP.md` cites. It did not
read the window's code, `sample.rs` or `PROPOSALS.md`, and could not build BIMCraft
offline, so its fourth finding is worked from BIMCraft's code and not from running it.

**What held.** Wall heights every way (a top level that is the wall's own storey, a top
below the base, offsets in inches and in feet and inches, elevations Assumed and Unknown).
Openings off either end, overlapping, above the top, touching, stacked. Rooms against
hand figures: a T-junction, a crossing wall, a partition in two pieces, a triangle,
corners of 5, 15, 45 and 60 degrees, an island with a room inside it, `join` off,
duplicate and overlapping walls, all six `location` lines with layers, a wall from the
storey below rising into this one, two rooms in one space. Sloped runs, risers, duct
surface. One line for one size however typed. A turned fixture joining at its turned
connector; a pipe and a duct at one point not joining. Every refusal on load it tried.
Save, load, save gave the same bytes on every probe file, and a hand-edited figure was
never trusted. Nothing panicked. Every line `PORTMAP.md` cites holds its symbol, and
about 45 line citations in DECISIONS.md call 29 were right to within a few lines.

**What it found, and what was done.**

| # | What the ninth judge found | What was done | Regression test |
|---|---|---|---|
| J9-1 | **A wrong figure, Known.** A partition with an Assumed base offset of 5 ft was left out of a room (it starts above the 4 ft its storey's rooms find their walls at) and the room was 185.25 SF, Known. With an Assumed base of 1 ft it was 90.25 SF, Known. One assumed value decided between the two and neither was flagged. A wall's entered height also did not carry its base's status | Each wall whose base or top rests on anything less than Known is tried gone and standing full height; a room that comes out differently either way takes its status and names it. A wall's heights carry the status of its storey's elevation and its base offset too (DECISIONS.md call 31) | `a_room_is_no_surer_than_a_height_that_decides_whether_a_wall_bounds_it` |
| J9-2 | **A silent drop.** A Wall element with no `pl::wall` loaded, and the rooms round it ignored it and stayed Known | A wall with nothing entered on it is a wall that cannot be placed: its storey's rooms have no figure and name it | `a_wall_with_nothing_entered_and_a_point_in_a_wall_leave_no_figure_standing` |
| J9-3 | **A flag dropped.** The command line's takeoff printed a system's total length with no mark when a run on it was only assumed to be on it | The heading carries the flag of its total | `a_takeoffs_heading_carries_the_flag_of_its_total` (command line) |
| J9-4 | **A port difference that was not listed, and moved an area.** BIMCraft joins a bounding wall to every wall it meets, bounding or not. The port joined bounding walls only to each other, so a partition ending on a wall that does not bound rooms ran to that wall's centre line: 180.375 SF where BIMCraft's rule gives 180.5 | A bounding wall is joined to every wall of the job it meets, as BIMCraft joins it. The judge's case gives 180.5 SF, 200 SF and 77 LF, worked by hand | `a_partition_stops_at_the_face_of_a_wall_that_does_not_bound_rooms` |
| J9-5 | An open connector was listed under the fixture's own system: a water closet on cold water with nothing on its waste outlet was said under Cold water, and Sanitary said nothing | An open pipe connector is listed on the systems that carry what it carries; any other on the thing's own system | `an_open_connector_is_listed_where_its_run_would_be` |
| J9-6 | **False in `PORTMAP.md`.** `MAX_CONNECTORS` was listed as ours with a search said to find nothing. BIMCraft has `MAX_CONNECTORS` (`family.rs:698`, 8) | The row is `ported`, `family.rs:698`; the header of `building/model.rs` says it is 64 here and why | the gate, which now runs the search |
| J9-7 | **Understated.** `building.rs` said five small things in it were BIMCraft's. Its own code cites BIMCraft for a wall's base and top, the opening fit rule, the height at which walls bound rooms and a run's true length; `check_wall` and `check_segment_type` take rules of BIMCraft's and were listed as ours; one citation pointed at a blank line | `building.rs`'s header lists the four rules of BIMCraft's its figures are worked out by, each cited where it is used; `check_wall` and `check_segment_type` are `ported` rows; the citation is `model.rs:3932` |  |
| J9-8 | **Misleading.** The takeoff was listed as ours on a search for the word "takeoff". BIMCraft's schedules list pipes and conduit by system with length and diameter, grouped and totalled; and it has a named electrical distribution system | The rows say what BIMCraft's schedules do and what is ours on top; DECISIONS.md call 31 corrects calls 28 and 29 |  |
| J9-9 | False in `PORTMAP.md`: a search said to find nothing for `fn said` finds `sketch.rs:221`. The gate never ran an "ours" search | The row cites a search that does find nothing. **The gate now runs every search an "ours" row says came back empty**, and was seen to fail on this row and on J9-6's before they were corrected | the gate |
| J9-10 | A wall's `opening_area` was refused when a window's sill or centre was Unknown, though its area was known; and the reason given for the net was wrong | A wall's opening area is as sure as its openings' sizes; an opening with a size and no known place leaves the net refused, saying the fit cannot be checked | `an_openings_area_is_its_size_and_where_it_is_matters_to_the_net` |
| J9-11 | A room whose point is inside a wall had no floor area, and a Known gross area | It has no gross area either: which side of the wall was meant is not guessed. A change from BIMCraft, listed in the header | `a_wall_with_nothing_entered_and_a_point_in_a_wall_leave_no_figure_standing` |
| J9-12 | `check_wall` refused only a wall whose ends are the same point; BIMCraft refuses one shorter than its tolerance. BIMCraft also refuses on load a wall whose top is not above its base; here it loads and its figures are refused. Neither difference was listed | A wall no longer than 0.01 ft is refused. A wall whose top is not above its base still loads, with its figures refused; the difference is listed in the header with the reason | `walls_join_within_a_hundredth_of_a_foot_and_no_further` |
| J9-13 | DECISIONS.md call 29 says each restated tolerance has a test on it. The tolerance by which walls join had none | A test: 0.005 ft short of a corner joins, 0.02 ft does not | `walls_join_within_a_hundredth_of_a_foot_and_no_further` |
| J9-14 | Smaller things: a header said a sill is above the storey's floor (it is above the wall's base); a header called 0.01 ft BIMCraft's 1 mm; the largest section was listed as ported at a different value with no note; the layers' sum was checked to 1e-6 ft where BIMCraft's is 1e-6 mm; SPEC.md said "a drain narrower downstream" where only one that drops is checked; "1 pieces" | Each corrected. The layers' sum is held to a billionth of a foot, with the reason in the code | `a_takeoffs_heading_carries_the_flag_of_its_total` for "1 piece" |

Not changed: a drain that narrows on the level is not warned of (BIMCraft's rule looks only
where a drain drops; SPEC.md now says so).

**These fixes have not been judged.**

## D3. How its demonstration will be run (written before any D3 work)

**Demonstration as the brief states it:** One parapet produces wall flashing (a height on
its LF), a coping cap, cleat on both sides, and fasteners by spacing rule. Sheet metal
converts LF × stretch-out ÷ 12 to SF, then to sheets or pounds, plus waste. Stretch-out is
editable on the line, and the formula is visible on the line. The owner's copper
sheet-metal estimate is reproduced line by line (eave 12 in, step flashing 8 in, valley
36 in, headwall 24 in), with any difference explained.

**What already exists, from the units amendment:** a cost line can carry a stretch-out,
and its length × stretch-out is derived as SF with the formula saved beside it. Nothing
else of D3 is built.

**Where the pattern comes from:** the owner's earlier program works a line as formula,
then quantity, then waste, then order units, and its sample parapet condition carries
exactly the four lines the brief names (DECISIONS.md call 18). D3 follows that pattern on
this project's model. Nothing is copied until it is read in full and attributed.

**How it will be run:** a script, `demos/d3.sh`, at the command line until the window
exists, on the sample job, printing the saved-job diff at each step and failing if a step
does not do what it says.

1. *One parapet, one condition, four lines.* Assign the sample parapet a parapet
   condition. It gains a wall flashing line, a coping cap line, a cleat line counted on
   both sides, and a fastener line by spacing. Each shows its formula. With the height
   unknown and no stretch-out or spacing entered, each shows Unknown with its reason and
   no number, and nothing is assumed for them.
2. *Enter what the detail gives.* Height `30 in`, coping stretch-out `19 1/2 in`, cleat
   stretch-out and fastener spacing as typed. Each line gets its number. The figures are
   worked by hand in the script's comments from the 20 LF parapet.
3. *The sheet-metal chain, each step visible.* On the coping line: LF × stretch-out ÷ 12
   gives SF; waste is added as a stated percentage; the result converts to sheets of a
   stated size, rounded up, or to pounds at a stated weight per SF. Each step is a figure
   with its own formula and status. An Unknown anywhere leaves no number after it.
4. *Edit on the line.* Change the coping stretch-out on its line: that line's figures
   change in the file diff and nothing else does. Change the cleat line's formula from
   both sides to one: the same.
5. *The brief's four stretch-outs.* Eave 12 in, step flashing 8 in, valley 36 in and
   headwall 24 in, each on a line of the sample roof, each worked by hand.
6. *Units.* Every stretch-out and spacing is entered as the detail gives it and nobody
   converts anything.

**Pending from the start:** the line-by-line reproduction of the owner's copper estimate.
`refs/` holds no estimate (QUESTIONS.md Q5). Step 5 uses the brief's stretch-outs on the
sample job's lengths, which proves the arithmetic and not the reproduction.

**The judge** runs steps 1 to 4 on a parapet and a condition of its own, and tries to get
a wrong figure, or a figure where there should be none, onto a line.

## D3. What was run and what it showed

**How it is run:** `demos/d3.sh`, on the sample job (20 LF of parapet). It exits non-zero
if any step does not do what it says. Its output is kept in `demos/captured/d3-demo.txt`. The
plan above was written first; two things changed from it while building, both noted below.

**What was run and what it showed** (output of `demos/d3.sh`, exit 0)

1. *One parapet, one condition, four lines.* `apply parapet-0024 "Parapet with copper
   coping"` gives the parapet the condition's name and four lines: wall flashing
   (`plan_length * height`), coping cap (`plan_length * stretch_out`), cleat
   (`plan_length * sides`), fasteners (`plan_length * sides / spacing`). With the height
   not on the plan and nothing else entered, the flashing, the coping and the fasteners
   show `unknown`, and none of the new values is Assumed (the cost code the recipe
   brings is Assumed, because the scheme waits on Q1). The cleat shows 40 LF: it needs
   only the length and the recipe's own `sides: 2`, which is the brief's "both sides".
2. *Entered as the detail gives it.* Height `30 in`, coping stretch-out `19 1/2"`, spacing
   `12 in`. Flashing 50 SF (20 × 30 ÷ 12). Coping 32.5 SF (20 × 19.5 ÷ 12). Cleat 40 LF.
   Fasteners 40 (20 × 2 ÷ 1 ft).
3. *The sheet-metal chain.* Before any waste is entered the "with waste" step shows
   `unknown`, not the quantity again. Waste `10%`, Assumed: 35.75 SF, Assumed. Ordered at
   `30 SF/sheet`: 2 sheets (35.75 ÷ 30 = 1.19, rounded up), Assumed. Or at `1.25 lb/SF`:
   44.6875 lb, not rounded. Each step is saved with its formula
   (`plan_length × stretch_out ÷ 12`; `quantity × (1 + waste ÷ 100)`;
   `with_waste ÷ factor, rounded up to a whole sheet`). With the stretch-out marked
   unknown, all three steps show `unknown`.
4. *Edited on the line.* Stretch-out to `24 in`: 40 SF, 44 SF with waste; in the saved
   file only the parapet changed, and on it only the coping line's figures. Cleat formula
   to `plan_length`: 20 LF, `sides` taken off the line and reported; only the cleat line
   changed. Flashing formula to `plan_length * (height + turn_out)` with `turn_out=8 in`:
   63.3333 SF; only the flashing line changed. Refused, with the reason: a length added to
   an area; a misspelled name; a spacing with no unit.
5. *The brief's four stretch-outs.* Eave 12 in on 40 LF: 40 SF. Step flashing 8 in on the
   8.2462 LF porch rake: 5.4975 SF. Valley 36 in, worked on a 22.5 LF hip because the
   sample roof has no valley: 67.5 SF. Headwall 24 in on 12 LF: 24 SF.
6. *Units.* Every length above was typed as written; ten values are saved in inches.

**Hand checks** (in `crates/world/tests/world.rs`): the figures above; fasteners at 18 in
(26.67, so 27); 60 SF at 30 SF to the sheet is 2 sheets, not 3; thirty-two formulas, line
values and recipes that do not make sense, each refused with the stated reason.

**Changed from the plan written beforehand**
* The plan said a cleat stretch-out would be entered. The cleat is counted by the LF on
  both sides, as the brief words it, and has no stretch-out.
* The plan said the cleat's formula would be changed "from both sides to one". It is, by
  rewriting the formula to `plan_length`, which takes `sides` off the line.

**What remains**

* **Judged once** (fifth judgment, below); its fixes are not judged.
* **The owner's copper estimate is not reproduced.** There is no estimate in `refs/`
  (Q5). Step 5 proves the arithmetic on made-up lengths. "With any difference explained"
  cannot start until the estimate is there.
* **The valley figure is on a hip.** The sample roof has no valley.
* **At the command line, not in a window.**
* **A unit cost cannot be entered from the command line, and a line cannot be removed.**
  Both are done in the file. Nothing is priced or totalled (D5).
* **A recipe is written in the file.** There is no command that makes or edits one.
* **Sheets or pounds is entered per line.** Nothing knows that copper is sold by the
  pound; there is no material library.
* **Fasteners are a count by spacing.** There is no end fastener (+1), no box size unless
  an order conversion is entered, and no spacing rule beyond one spacing.
* The model calls behind this are DECISIONS.md call 21, each recorded so it can be
  overruled. The one most worth the owner's eye: a recipe carries no typical stretch-out,
  waste or sheet size, so a newly applied condition prices nothing until they are entered.

---

## D4. How its demonstration will be run (written before any D4 work)

**Demonstration as the brief states it:** A design-phase job is priced from $/SF and $/LF.
Detailing the same entities rolls back up to $/SF and $/LF, and the two reconcile.

**What exists that it stands on:** every element has its SF or LF worked out with a status
(D1); a cost line works out a quantity, with waste and order units (D3). **What does not
exist:** nothing multiplies a quantity by a price. There is no extension, no total, and no
command that enters a unit cost. D4 is the first deliverable on the money path.

**How it will be run:** a script, `demos/d4.sh`, at the command line until the window
exists, on the sample job, with made-up prices that are stated in the script. Every
figure is worked by hand in its comments.

1. *Design phase.* Each roof plane is given an allowance in $/SF of its surface, and each
   run of flashing an allowance in $/LF, each with its status and source. The job is
   priced from those alone: each element's quantity times its rate, and a total. An
   element with no allowance is listed as not priced, and the total says how many elements
   it leaves out. It is never a silent zero.
2. *An assumed input shows in the money.* The porch plane's pitch is Assumed, so its
   area, and so its price, is flagged Assumed, and the total says how much of itself rests
   on assumptions.
3. *Detail the same entities.* The same planes and lines get their cost lines priced
   (unit costs entered on the lines D3 built). Each element's lines are extended and
   summed.
4. *Roll back up.* Each detailed element's sum is divided by the same SF or LF the
   allowance was priced on, giving the $/SF or $/LF the detail comes to, shown beside the
   allowance.
5. *Reconcile.* Per element and for the job: the allowance figure, the detailed figure,
   and the difference in dollars and percent. An element whose detail is incomplete (a
   line with no price, or no quantity) is said to be incomplete and is not rolled up as if
   it were whole.
6. *The geometry still drives both.* Change one pitch: the allowance figure and the
   detailed figure for that plane both move, and the file diff shows that plane and what
   derives from it.

**What the brief's rule 4 demands of it:** no assumed or unknown value reaches a price
without the price carrying the flag; a total never leaves something out without saying
what. QUESTIONS.md Q1 (MasterFormat or UniFormat) bites here: design-phase rates read
naturally by element and detailed lines by work result. Each code carries its scheme, so
the demonstration can run before the ruling; no total will be grouped by code until then.

**The judge** prices a job of its own both ways and tries to make the two disagree
without the program saying so, or to get a flagged value into a total that looks clean.

## D4. What was run and what it showed

**How it is run:** `demos/d4.sh` at the command line, and `demos/d4w.sh` in the window. Both
use the sample roof with made-up prices that are stated in the scripts: a budget rate of
$28.00 a SF of roof and $40.00 a LF of headwall flashing; quotes of $31.50 a SF for the
standing seam and $18.25 a SF for the headwall flashing. Every figure is worked by hand
in the scripts' comments. What was decided on the way is DECISIONS.md call 30.

**What was run and what it showed** (`demos/d4.sh`, exit 0)

1. *Nothing priced.* The job's allowance total and its detail total each read "none".
   Neither is $0.
2. *Design phase.* Three planes and the headwall given allowances. South plane:
   419.2627 SF x 28.00 = $11,739.36. East plane: 251.5576 SF = $7,043.61. Shed:
   98.9545 SF = $2,770.73. Headwall: 12 LF x 40.00 = $480.00. Total $22,033.70 over four
   elements. The two planes with no allowance read "none entered", and the 14 elements
   that could carry a price and carry none are counted by kind.
3. *An assumed input shows in the money.* The shed's pitch is Assumed, so its area is, and
   its $2,770.73 is flagged. The headwall lies on the shed, so its length and its $480.00
   are flagged too. The total says $3,250.73 of itself rests on something assumed. (I had
   worked the headwall as Known by hand; the program carried the flag further than I did,
   and was right.)
4. *Detail.* Unit costs entered on the lines D3 built. South plane: 419.2627 SF x 31.50
   = $13,206.78. Shed: $3,117.07. Headwall: 12 LF x 24 in ÷ 12 = 24 SF x 18.25 = $438.00.
5. *Rolled back up.* South plane: $13,206.78 ÷ 419.2627 SF = $31.50 a SF. The headwall's
   line is priced by the SF and rolls up to the LF its allowance is in: $438.00 ÷ 12 LF
   = $36.50 a LF.
6. *Reconciled.* South plane: $1,467.42 over its allowance, 12.50 %. Shed: $346.34 over,
   12.50 %. Headwall: $42.00 under, -8.75 %. The east plane has an allowance and a line
   with no price: its detail "is not whole: 1 of its 1 lines cannot be extended", it has
   no detail figure, and it is not rolled up. Detail total $16,761.85 over three
   elements, $3,555.07 of it assumed, with the three planes it leaves out named. Like for
   like, over the three elements that have both: allowances $14,990.09 against detail
   $16,761.85, $1,771.76 over, 11.82 %.
7. *The geometry still drives both.* The south plane made 9:12: 375 SF on plan x 1.25
   = 468.75 SF. Allowance $13,125.00; detail $14,765.63; still $31.50 a SF. In the file,
   the plane and three lines that lie on it changed, and only the plane's price.
8. *Refused:* an allowance by the LF on a plane; an allowance on a roof; a unit cost by
   the LF on a line priced by the SF; a price below zero.

**In the window** (`demos/d4w.sh`; pictures in `demos/captured/d4/`). The script starts
the program on the job step 6 of the list above leaves (before the pitch change), opens
it the way a person does, and drives it with the pointer and the keyboard. All fourteen
pictures were opened and read at full size. What they show:

* *01, 02.* The job as opened, and the sheet's Money tab. South plane: 419.26 SF,
  allowance $11,739.36, detail $13,206.78, comes to $31.50 / SF, $1,467.42 over,
  12.50 %. East: allowance $7,043.61, detail "no figure", with the reason under it in
  red. North and west: allowance "none entered", detail "no figure". Shed and headwall
  amber (assumed). The job: allowances $22,033.70 over 4 elements, "$3,250.73 of it
  rests on something assumed"; detail $16,761.85 over 3, with the three planes left out
  named; like for like $1,771.76 over (11.82 %); "Not priced at all: 14 that could be",
  counted by kind.
* *03.* The east row clicked on the sheet: the model selects the east plane and its
  card shows its allowance, 28.00 $/SF, with its source.
* *04, 05.* `U` opens a field on the model window, "Unit cost of "Copper standing seam,
  installed" on east"; 31.50 typed and entered. The sheet's east row now reads detail
  $7,924.06, $31.50 / SF, $880.45 over, 12.50 %, in amber, because a price typed in the
  window is Assumed until it is confirmed. Detail total $24,685.91 over 4 elements,
  $11,479.13 of it assumed; like for like $2,652.21 over (12.04 %). The title bar says
  "Not saved".
* *06.* The north row clicked, `B`, 28 entered: north's allowance $11,739.36, amber, its
  card saying "set in the 3D view; not yet confirmed against a drawing". Allowances
  $33,773.06 over 5 elements, $14,990.09 of it assumed. Its detail is still "no figure".
* *07.* "nonsense" typed as a price: refused in red on both windows, `"nonsense" is not
  a price; write it as a number of dollars, like 4.25 or $4.25/SF`, and nothing changed.

The script then saves with the keyboard and reads the file: the east line's unit cost is
31.5, Assumed, with the source above; the north plane's line still has no price.

Seen in the pictures and left as it is: the model window's "To settle" list still leads
with the schedule codes, which nothing in D4 uses; the sheet's Money tab runs its own "To
settle" list past the bottom of the window; the wheel scrolls it, and nothing on screen
says so.

**Tests** (`crates/world/tests/world.rs`, three, with the figures above worked in their
comments): the two ways and their reconciliation; that money is never silently zero (no
price is no figure, $0 entered is $0.00, less than half a cent is refused, an allowance
of $0 gives no percentage); that a price is entered for the unit a thing is priced in.

**What remains**
* A price typed in the window is entered as Assumed, with the window as its source: the
  window has nowhere to say where a figure came from.
* The window prices a line's unit cost and an element's allowance. A line's waste, order
  conversion and stretch-out are still entered at the command line, so a line that needs
  one of those before it has a quantity cannot be completed in the window.
* Nothing is grouped by cost code (QUESTIONS.md Q1 is open).
* An allowance is a rate on one figure per kind of element. A lump sum is not built.
* D5's columns (material, labour, general), mark-ups and suppliers are D5.

## D5. How its demonstration will be run (written before any D5 work)

The brief: "Material, labor, and general columns. Labor toggles per line. Subcontractor
work is a priced line; in-house labor is a production rate. Fabrication and supervision
are separate lines. General conditions include hand-typed lines. O&P is editable. Recap
categories are honest. Generic and supplier-specific pricing are shown side by side for
three suppliers. Changing an entity's material reprices every dependent line and the
recap. The owner's whole-job estimate is reproduced within rounding."

`demos/d5.sh` (the command line) and `demos/d5w.sh` (the window) run on a made-up job;
every price, rate and supplier in them is made up and stated in the script, and every
figure they expect is worked by hand in the script's comments. Each step fails the
script if the program does not do what the step says.

1. *The sheet.* `plumbline estimate` prints every cost line under its element with its
   quantity, what it is priced in, its unit cost and its amount in one of three columns
   (material, labor, general), and each column's total is the sum of the amounts shown.
2. *In-house labor is a production rate.* A labor line says so many SF or LF to the
   hour; the line shows the hours and the hours times the hourly rate. The hours are
   worked by hand.
3. *Labor toggles per line.* A labor line is switched off: it stays on the sheet, marked
   off, and leaves the labor column, the element's detail and the recap by exactly its
   amount. Switched on again, everything is back to the cent.
4. *Subcontract is a priced line.* A subcontract line is a quantity times a price, lands
   in its own class, and is refused a production rate.
5. *Fabrication and supervision are separate lines*, each in its own class in the recap,
   with its hours.
6. *General conditions with hand-typed lines.* A line is typed onto the job itself
   (a description, a number, a price) and lands in general conditions.
7. *The recap.* One row for each class, each the sum of its lines as shown; the job cost
   their sum; overhead and profit by the percentages entered; the price. A line with no
   class, no price or no quantity is named and left out, never counted as $0.00. With
   overhead or profit not entered there is no price, and the recap says which is missing.
8. *O&P is editable.* Overhead and profit are changed; the recap moves by the amounts
   worked by hand.
9. *Three suppliers side by side.* A material carries a generic price and a price from
   each of three suppliers. The sheet shows each line made of it at all four, and each
   supplier's total with what it leaves out (a material that supplier did not quote).
10. *Changing an entity's material.* An element's material is changed. Every line on it
    made of that material reprices (its pounds too, where the new material weighs
    differently), the lines not made of it do not move, and the recap moves by the
    difference worked by hand. In the file, only that element changed.
11. *What is refused:* a production rate on a subcontract line; a labor line not in
    hours; a material the job does not have; a quote in the wrong unit; a percentage
    that is not a number.

In the window: the sheet's Estimate tab shows the same sheet, recap and suppliers; an
element's material is changed, a labor line switched off and the overhead typed with the
keyboard, and the sheet follows; the job is saved and the file holds what was typed,
flagged.

**Pending, and said so:** "The owner's whole-job estimate is reproduced within rounding."
`refs/` holds no estimate (QUESTIONS.md Q5). D5 is built against the brief's words, and
that reproduction stays pending until the spreadsheet is there.

The judge prices a job of its own, and tries to get a line into the wrong class, out of
the recap without being named, or counted twice; to make a column total differ from the
sum of what is shown; and to get an assumed price into a total that looks clean.


## D5. What was run and what it showed

Judged since this was written: see "Eleventh judgment". Where a figure below gives hours
to two places, the program now gives four (J11-4); the dollars are unchanged.

**At the command line** (`demos/d5.sh`, exit 0; every figure worked by hand in the
script before it was run, and each matched):

1. *The sheet.* `plumbline estimate` on the made-up priced job: sixteen lines under
   their elements, each amount in one of three columns. Column totals: material
   $8,628.10, labor $6,350.47, general $10,313.86 (then $13,463.86 with the typed lines).
2. *Production rates.* South plane, 419.2627 SF at 25 SF to the hour: 16.7705 hours, at
   $72.00: $1,207.48. Headwall, 12 LF at 0.25 hours each: 3 hours, $216.00, flagged.
   Labor row: $2,147.97, 29.83 hours.
3. *The labor switch.* The south plane's install line switched off: labor $940.49 and
   13.06 hours, job cost $24,084.95, the line still on the sheet as "off ($1,207.48)"
   and named under the recap. Switched on: back to $2,147.97 and $25,292.43.
4. *Subcontract.* West plane, 251.5576 SF at $41.00: $10,313.86 in its own row.
   Making a line in hours Subcontract is refused.
5. *Fabrication and supervision.* Fabrication $402.50, 6.71 hours. Supervision
   $3,800.00, 40 hours (a foreman, five days of eight hours, on the job itself).
6. *General conditions, typed.* "Dumpster", 2, typed onto the job: named as having no
   price, and general conditions still "none". Priced at $650.00, and a permit at
   $1,850.00: general conditions $3,150.00.
7. *The recap.* Job cost $28,442.43 over 15 lines; the north plane's line, which has no
   class and no price, named and left out; no price, "overhead has not been entered".
8. *O&P.* Overhead 10 %: $2,844.24, and still no price. Profit 8 %: $2,502.93, price
   $33,789.60. Overhead changed to 12 %: $3,413.09, profit $2,548.44, price $34,403.96.
9. *Three suppliers.* Generic $8,024.36; Harbor Metals $7,861.41; Ridge Supply $8,254.95,
   each over 4 of 4 lines; Tinners Yard $6,714.91 over 2 of 4, and the two it leaves out
   named with why (it has not quoted the 20 oz copper).
10. *A change of material.* South plane from 16 oz to 20 oz copper: its panels go from
    461.19 lb and $4,150.70 to 576.49 lb and $5,188.38; its underlayment, shop work and
    install do not move; material $9,665.78, job cost $29,480.11, price $35,659.14. In
    the file only that plane changed.
11. *Refused:* a non-hours line made Labor; the switch on a Fabrication line; a quote by
    the SF for a material sold by the pound; a unit cost on a line made of its material;
    an overhead of "ten". A material the job does not have is accepted and leaves its
    lines with no figure and the reason.

**In the window** (`demos/d5w.sh`, exit 0; pictures in `demos/captured/d5/`). The script
opens the made-up priced job with the two typed general-conditions lines, the way a
person does, and drives it with the pointer and keyboard. All twenty pictures were
opened and read at full size (the ten sheet pictures are byte for byte those of a trial
run read first; the model pictures were read from this run).

* *01.* As opened: the roof, and the sheet's Roof tab.
* *02, 03, 04.* The sheet's Estimate tab, top, middle and bottom: the sixteen lines in
  their three columns with their column totals; the recap with its six rows, job cost
  $28,442.43, and "no figure" for overhead, profit and price, each with why; the
  suppliers side by side, with "2 of 4" in red under Tinners Yard and what it leaves out.
* *05, 06.* `O` opens "Overhead on the job"; 12 entered; `R`, 8 entered. The recap reads
  overhead $3,413.09, profit $2,548.44, price $34,403.96, in amber: what is typed in the
  window is Assumed.
* *07, 08.* The south plane picked in the model; `N` opens "What south is made of";
  "Copper, 20 oz" entered. Its card shows the material, Assumed, with its source. The
  recap reads material $9,665.78, job cost $29,480.11, price $35,659.14.
* *09.* `L`: "Install panels" switched off. Labor $940.49 and 13.06 hours, job cost
  $28,272.63, price $34,198.58, and the line named under the recap as switched off.
* *10.* An overhead of "ten" is refused in red on both windows and nothing changes.

The script then saves with the keyboard and reads the file: the two changed elements are
the job and the south plane; the material, the overhead and the profit are Assumed with
the window as their source; the labor line is off.

**Found by looking at the first trial's pictures, and fixed before these were taken:**
the sheet was wider than its window on the Estimate tab, so the General column and the
right edge of every card were cut off. The window's layout let the sheet grow to its
longest line of text. The sheet is now held to its window, and a long line name wraps
with its class or material under it.

**Seen and left:** on the entry card for the overhead the hint wraps onto a second line;
the model window has no way to pick which Labor line to switch when an element has more
than one (it says so and gives the command).

**Tests** (`crates/world/tests/world.rs`, seven for D5, each figure worked by hand in its
comments): the classes, columns and job cost; overhead and profit with no default; the
labor switch; production rates and what a subcontract line is refused; a change of
material; suppliers side by side; lines typed onto the job.

**What remains for D5:** an independent judge has not yet tried to break it. The owner's
whole-job estimate is not reproduced (there is none in `refs/`, Q5). Whether the sheet's
columns and the labor switch are what the owner's sheet has is Q14. Tax, burden, bond,
crew-day rates and pricing at a chosen supplier are proposals 12 to 14, not built.

## Drawing and editing. How its demonstration will be run (written as the stretch began)

The task and its fence are `HANDOFF_DRAW.md`. Two demonstrations:

* `demos/draw.sh`, at the command line: from an empty job, with no file edited by hand,
  a storey, the types, four outside walls and a partition, two doors and three windows,
  two rooms, a duct and a pipe each on a system, a fixture and a piece of equipment, and
  a roof; then the partition moved and a window removed. Every figure (wall lengths and
  net faces, room areas, run lengths) is worked by hand in the script before it is run.
  Then what is refused, and that a refusal leaves the file byte for byte what it was.
* `demos/draww.sh`, in the window: the same house drawn in the plan view with the pointer
  and typed lengths, snapping shown, a step undone and redone, the partition moved, a
  window taken out, the job saved; and the saved file's figures set against the
  command-line house: the same figures.

The judge tries to draw what should be refused, to get an unflagged value into the file
through a verb or the window, to undo into a state that never existed, and to snap to
something that is not there.

## Drawing and editing. What was run and what it showed

**Demonstrated at the command line and in the window. Not judged.**

* *The gate was widened first.* `tools/portmap.sh` covers `crates/world/src/editing*.rs`.
  It was run red on the new file (seven unlisted items) before the map had its rows; it
  is green at 177 items, 114 ported and 63 ours.
* *Undo and redo* (`editing/history.rs`, ported from BIMCraft's `engine.rs`): a test takes
  three steps, steps back and forward, and checks the job's saved text at each point
  against the text it had; all the way back is the job it started as, to the byte.
* *The verbs* (`editing.rs`; each rule ported from BIMCraft's `commands.rs`, each value
  written with a status and a source, each verb all of its change or none):
  `demos/draw.sh` exits 0. Walls 96 LF; net faces 195, 123, 132, 192 SF by wall and 786 SF
  in all; rooms 178.25 SF each, 54 LF round; duct 20 LF; pipe 14 LF; a roof solved over
  it. The partition moved 1 ft: rooms 193.75 and 162.75 SF, its door still 8 ft along it.
  A window removed: the north wall's net 204 SF. Eight refusals, and the file unchanged
  by them.
* *Snapping* (`editing/snap.rs`, ported from BIMCraft's `snap.rs`): a test on plans
  written out by hand: an end, a middle, a crossing, the square, a 15 degree guide, the
  nearest point on a wall, and the inch when nothing is in reach.
* *Tests:* a house drawn from nothing with its figures; what cannot be drawn is refused
  and leaves the job as it was (a wall of no length in the middle of three takes the
  first with it); a wall moved takes its openings and a wall removed takes them too.
* *The window* (`crates/app/src/tools.rs`; DECISIONS.md call 38): `demos/draww.sh` exits
  0. From a job made by `plumbline new` with nothing in it, with a real pointer and
  keyboard on a virtual display: a storey; four outside walls by typed lengths (24, 16,
  24 ft and a click that closes on the first corner) and a partition; a door in the
  south wall and one in the partition; a window in the east wall and two in the north;
  a duct of two pieces and a pipe, each on a system the tool made; a lavatory on the
  pipe's end and an air handler on the duct's start; the last step undone and done
  again; the partition moved a foot east by a typed length; a stray wall drawn and
  removed; one north window selected on the plan and removed; saved. The saved job
  passes `check`, and its figures are the command-line house's after the same two edits:
  walls 96 LF; net faces 123, 132, 144, 195 and 204 SF; two doors, two windows; duct
  20 LF; pipe 14 LF. Every wall's line is Assumed and says it was drawn in the window.
  Rooms placed at the command line inside the drawn walls come to 193.75 and 162.75 SF.
  The twenty-eight pictures are in `demos/captured/draw/` and each was opened and read.

**What reading the pictures found, and what was done:**

* On plan the doors and windows could not be seen: looking straight down, the wall over
  an opening hides it, and the only sign of one was its shadow. A plan now draws each
  opening as its span through the wall (a window with its glass down the middle; a door
  with no swing, because which way it swings is not in the model).
* On plan the sun's shadows filled the rooms and hid what was on the floor. A plan has
  no shadows now. The three older pictures taken on plan were taken again with
  `demos/house.sh` and `demos/d2.sh` (both exit 0) and read: `house/06-model.png`,
  `house/07-model.png`, `d2/08-model.png`. The other pictures of those two
  demonstrations are as they were last read; what draws them has not changed.
* A click on a window on plan selected its wall, and Delete then took the wall and both
  its windows. On plan the wall over and under an opening now selects the opening.
* A selected window on plan was the blue of every other window. Windows are drawn dark
  now, and the selected opening in the accent.
* Two rows of the left panel were squeezed when the Systems list appeared, so a click
  meant for one tool landed on another. The panel scrolls and its rows keep their height.

**What is not right, and is said:**

* On the software renderer the demonstration runs on, keys typed in about the second
  after a field opens are lost. Not explained. The script types the first character,
  waits, and types the rest. Whether a real graphics card shows it is not known.
* The window has no way to call a drawn value Known, or to draw a room or a roof; those
  are at the command line. Nothing draws a floor yet, in the window or at the command
  line: a floor is in a job only when the file was made with one (the sample house).
* A door is drawn on plan with no swing: the model has no hinge side.
* A selected element's label sits over its middle, so on plan the label of a short wall
  covers the wall (picture 10).
* Conduit and cable are the duct and pipe tool with another kind. Neither demonstration
  draws one.

**What remains of this stretch:** the judge.

## Fittings, connections and clashes. What was run and what it showed

**Demonstrated at the command line and in the window. Not judged.** The task is
`HANDOFF_CLASH.md`; what was decided is DECISIONS.md calls 39 and 40; SPEC.md 5.9, 5.10.

* *The gate first.* `tools/portmap.sh` was run red on `editing/join.rs` (eight unlisted
  items) and again on `building/clash.rs` (six) before either had rows. It is green at
  192 items, 115 ported and 77 ours. The licence gate passes with the geometry library's
  twelve crates, all Apache-2.0, MIT or Zlib (LICENSES.md).
* *Fittings and connections at the command line:* `demos/fit.sh` exits 0. A duct with a
  corner and a branch, 28 LF: before, "1 network; 3 open ends; 1 place where a run meets
  another along its length with no fitting". `fitting --all` puts an elbow on the corner
  and a tee on the branch, cutting the 8 LF piece into 4 and 4; the duct is still 28 LF.
  Joined to an air handler: 2 open ends. A 3/4 in pipe that becomes 1/2 in: a
  transition, called "3/4 in to 1/2 in transition"; joined to a lavatory: 1 open end.
  Eight refusals, and the file unchanged by them.
* *Clashes at the command line:* `demos/clash.sh` exits 0. Two 2 ft tanks 1 ft 6 in apart:
  in each other by 6.00 in. Touching: no clash; with a 2 in clearance asked, 0.00 in
  apart and flagged for the Assumed move. A 6 in duct and a 1 in pipe crossing at one
  height: 3.50 in, at (5, 5, 8), flagged because the pipe was Assumed. The pipe 5 in
  higher: clear, 1.50 in apart for a 2 in clearance and nothing for a 1 in one. A pipe
  through a wall over a window's head: one hole to cut; through the window: nothing. A
  corner, its elbow and a joined air handler: no clash; the same handler not joined:
  12.00 in. A thing with no size: listed as not tested. On the sample house: 18 in the
  same space, 66 through a wall or a floor, 185 elements tested, 5 not; its trunk duct
  is drawn through its soil stack by 3.50 in.
* *In the window:* `demos/clashw.sh` exits 0. On a job made at the command line, with a
  real pointer on a virtual display: the sheet's Clashes tab lists two clashes and the
  plan marks each in red; one click with "Fitting where runs meet" puts an elbow on the
  duct's corner, another a tee on its branch (the duct cut there); one click with "Join
  a run to what it ends at" joins the duct to the air handler, and that clash leaves the
  list; the clash that is left, picked on the sheet, selects the pipe on the model. The
  saved job passes `check`: 28 LF, one elbow, one tee, "1 network; 2 open ends; 0
  places", one clash of 3.50 in at (4, 2, 8), both fittings Assumed and saying which
  runs they join. The fourteen pictures are in `demos/captured/clash/`, each read.
* *Tests* (`crates/world/tests/world.rs`, six new, figures worked by hand): a fitting
  worked out from what meets, with what is refused; every joint of a system fitted and
  what is passed over said; a run cut in two, on the level and climbing, and the cut
  refused where a typed quantity would be counted twice; two solids in one space and two
  only close; runs clashing by how far their surfaces are in each other, round and
  rectangular; a run through a wall, through a window, and what is joined.

**What building it found:**

* The geometry library's measure of how deep two solids are in each other answers zero
  when one is dead centre in the other: a duct and a pipe crossing at one height. The
  first clash test written found it (no clash reported where 3 3/8 in was worked by
  hand). The yes or no now comes from its overlap test, which does not fail, and the
  depth is asked for separately (call 40).
* In the window a click can be lost when the press and the release fall inside one slow
  frame of the software renderer. The demonstration holds each click for longer than a
  frame. This is the same fault as the keys lost after a field opens; it has not been
  seen, or looked for, on a real graphics card, because none was available.

**What is not right, and is said:**

* Joints are found a storey at a time: a riser that meets a run on the storey above is
  not joined to it by `fitting`.
* A clash's depth is not given for a run through a wall or a floor, and is "not worked
  out" where the library cannot measure it even off centre. No case of the second has
  been seen.
* The roof's planes are not tested for clashes. A door or a window is a hole: a pipe
  across a doorway is not reported.
* The window's Fitting tool cannot say which joint is meant when two lie over one
  another at different heights; the command line can (`--height`).
* The status bar shows the storey's floor as the height of a joint the pointer is on,
  not the joint's own height.

**What remains of this stretch:** the judge.

## Tenth judgment (2026-11-11): D4, and the fixes to the ninth

An independent judge (Opus; it did not build any of this) was given commit `1886693`, a
frozen copy of the source and a built command line, and the brief to break D4's money
path and the ninth judgment's fixes. It reported thirteen findings and no panic. Its
reproductions are kept outside the repository (they are scratch files). What it found,
as it reported it; what was done about each is in the table that follows this list once
the fix is in.

| # | Finding (the judge's words, shortened) | Severity it gave |
|---|---|---|
| J10-1 | An element's condition can be Assumed or Unknown and its detail, rolled-up rate and difference still show as Known. | wrong figure unflagged |
| J10-2 | A non-bounding wall's Assumed thickness or line decides a room's area (through the wall that ends on it), and the room stays Known. | wrong figure unflagged |
| J10-3 | A quantity of zero is priced to a clean $0.00 and rolled up to $0.00 per SF. | silent zero |
| J10-4 | The like-for-like amounts and every percentage print without the flag. | wrong figure unflagged (printout) |
| J10-5 | A comma anywhere in a price is deleted: `1,25` is read as 125. | wrong figure unflagged (entry) |
| J10-6 | A rate under a cent is shown as $0.00 in `price` and `show`. | silent zero (display) |
| J10-7 | Extensions are not rounded half-up to the cent: 1 x 1.015 gives $1.01. | wrong figure, a cent |
| J10-8 | "Not priced at all" leaves out every door, window, fitting, fixture and piece of equipment, though each can carry a priced line. | silent drop |
| J10-9 | STATUS.md and call 30 say the allowance total says how many elements it leaves out. It does not. | false claim in a document |
| J10-10 | An Assumed height on a second-floor wall flags a first-floor room; the note names a wall by a name two walls share. | false flag |
| J10-11 | Money beyond about $90 trillion is printed as if exact to the cent. | minor |
| J10-12 | Printout faults: an empty reason "no figure ()"; a refused roll-up rate printed nowhere; "$0.00 the same as"; a room point on a face corner read as inside a wall. | minor |
| J10-13 | PORTMAP.md: `StoreyTotals`/`storey_totals` claim net wall area as ours on a search that skips `ifc.rs`, where BIMCraft works it; five `check_*` rows say they wrap BIMCraft's rule and cite none. | false claim in a document |

What was done about each. Every fix has a test in `crates/world/tests/world.rs` named
for its finding (`j10_1_...` and so on), with the judge's reproduction in it and the
figures worked by hand. The six money tests that the pre-fix source can compile were
run against the judge's frozen copy of commit `1886693`, and all six failed there.
**These fixes have not themselves been judged.**

| # | What was done | Test |
|---|---|---|
| J10-1 | What an element's lines come to now rests on its condition: an Assumed condition makes each extension, the detail, the rate and the difference Assumed; a condition that is not known leaves them with no figure and the reason. The allowance does not rest on the condition and is untouched. | `j10_1_...` |
| J10-2 | A room is no surer than the line and thickness of every wall its own walls are cut to, bounding or not. It says which wall, by name and path. | `j10_2_...` (180.625 SF and 181 SF worked by hand) |
| J10-3 | A line whose quantity is 0 still comes to $0.00 (a wall with no openings has no opening trim, and that is an answer), and now says so: the extension carries the note "the line's quantity is 0, so it comes to nothing", the element's detail names such lines, and `price` and `estimate` print the note. It is not refused: refusing would leave every such element with no detail. | `j10_3_...` |
| J10-4 | The like-for-like amounts each carry their own flag, and the difference and every percentage carry theirs. | `demos/d4.sh` expects the flagged line |
| J10-5 | A comma in a price is a thousands mark in the right place or the price is refused. `1,25`, `,5` and `12,34.5` are refused. | `j10_5_...` |
| J10-6 | A rate is written to the cent, or to as many places (up to six) as it takes to show it: $0.004 is $0.004. In `price`, `show`, `estimate` and the sheet. | `j10_6_...` |
| J10-7 | Money is rounded to the cent as it would be on paper: 1.015 is $1.02. The first fix for this was wrong at large amounts (it moved $9.9 trillion by $9,900); the test for J10-11 caught it before it was committed, and the fix now moves a figure by a few units in its own last place and never by a thousandth of a cent. | `j10_7_...`, `j10_11_...` |
| J10-8 | "Not priced at all" counts doors, windows, fittings, fixtures and equipment too. | `j10_8_...` |
| J10-9 | The program now does what the documents said: under the allowance total, the elements with cost lines and no allowance are counted and named; under the detail total, the elements with an allowance and no cost line. | `demos/d4.sh` |
| J10-10 | When a wall's heights are unsure, it is tried standing as tall as it could: its top anywhere above, its base anywhere below only if its base is itself unsure, and it bounds a storey's rooms only if it then reaches them. A wall that starts at a Known height above a room no longer flags it. The note names the wall by path as well as name. | `j10_10_...` |
| J10-11 | An amount of more than $10 trillion is refused: it cannot be held to the cent. | `j10_11_...` |
| J10-12 | The empty reason now reads "a value it is worked out from is not known"; a detail with a figure and no rate prints why it has no rate; a difference of nothing reads "the detail is the same as the allowance". **Left as it is:** a room point exactly on the corner where two wall faces meet reads as inside a wall, where a point on one face reads as in the room. BIMCraft's region test, as ported, decides it, and changing that test is a difference from the port for a point nobody places a room at. A change to it was tried and taken back out. | none for the corner |
| J10-13 | `StoreyTotals` and `storey_totals` stay ours and now say what BIMCraft has (a wall's net side area, `ifc.rs:731`; schedules that group and total) and what it does not (a total with a status), on a search that includes `ifc.rs`. `check_level`, `check_room`, `check_opening_type`, `check_floor` and `check_placed` are now `ported`, each with the line of BIMCraft's rule it carries. The map is 81 ported, 55 ours. | `tools/portmap.sh` |

**What held under attack** (the judge's list): every hand figure for allowances,
extensions, waste, order rounding, sums and "of which assumed"; statuses carried into the
money from an assumed waste, order factor and pitch; a price not carrying across a change
of unit; negatives, `nan`, `inf`, `1e3` and anything past 1e9 refused; an amount under
half a cent refused; nothing priced printing "none"; a detail not whole refused with its
line named; a hand-edited derived figure in the file never trusted; save, load, save to
the same bytes; allowances on a wall, floor, room, duct and pipe; J9-10, J9-11, J9-12.

**The demonstrations after these fixes.** `demos/d1.sh`, `units.sh`, `d3.sh`, `d4.sh`,
`d4w.sh`, `house.sh` and `d2.sh` were run again and exit 0 (`d4.sh` and `d4w.sh` with their
expected wording changed for J10-4 and J10-9). Every sheet picture came out byte for byte
the picture already read. Every model picture differs from the one already read by fewer
than 100 of its 1.5 million pixels (the renderer is not exactly repeatable), which was
measured, not looked at again.

**Not reached by the judge:** J9-5 again; J9-4 across storeys; an uncertain storey
elevation or computation height run (read only); the window; the money path through
`apply` and through low-slope and parapet lines.


## Eleventh judgment (2026-11-11): D5, and the fixes to the tenth

An independent judge (Opus; it did not build any of this) was given commit `016ac79`, a
frozen copy of the source and a built command line, and the brief to break the estimate
sheet and the tenth judgment's fixes. Fourteen findings, no panic. As it reported them:

| # | Finding (the judge's words, shortened) | Severity it gave |
|---|---|---|
| J11-1 | The PRICE is shown Known and clean when the job cost leaves lines out; it can be below the cost `price` shows. A line with no class shows its amount in the General column, whose total leaves it out. | wrong figure unflagged |
| J11-2 | Money is pushed across a cent from 0.4999 of a cent at $300 million, well below where call 33 says rounding is exact. | wrong figure unflagged |
| J11-3 | A typed quantity that is not whole (2.5) is rounded up to 3 with no note on the sheet. | wrong figure unflagged |
| J11-4 | Hours print to two places: a line's hours do not multiply to its amount, and a class's hours are not the sum of what its lines show. | wrong figure unflagged (display) |
| J11-5 | `line` on a door writes a file `check` then refuses: the wall's inputs come back in another order. | minor |
| J11-6 | J10-6 is not closed: `show` writes a rate of 0.00004 as 0, and `show` and `estimate` print the same price differently. | silent zero (display) |
| J11-7 | The supplier columns give false reasons: "has not quoted" for a line left out because its condition or material is not known. | false claim in a printout |
| J11-8 | Cost lines on a Condition, a Material or a Type are accepted and counted in the job cost. | loads what it should refuse |
| J11-9 | J10-2's fix flags a room for a wall that cannot matter to it (met at the far end of one of its walls). | false flag |
| J11-10 | Switching off an element's only line makes `price` say "detail none entered" and "no cost line on 1 element with an allowance", both false. | false claim in a printout |
| J11-11 | Totals above $10 trillion are printed to the cent; only the price is refused. | minor |
| J11-12 | Suppliers and materials whose names differ only in capitals or spacing are kept apart. | minor |
| J11-13 | Hours can be rounded up to whole hours with `whole`. | minor |
| J11-14 | An overhead of 0.004 prints "at 0.00 %"; a material can be priced by the hour; a material named by path prints the path; with an Assumed condition the pounds and hours are flagged though the quantity "does not rest on it"; typed lines take their element's condition's status. | minor |

What was done about each. Every fix has a test in `crates/world/tests/world.rs` named
for its finding (`j11_1_...` and so on), or a line in a demonstration script, with the
figures worked by hand. **These fixes have not themselves been judged.**

| # | What was done | Test |
|---|---|---|
| J11-1 | A total of money that leaves something out is never shown as Known: the job cost, each class, each column, the D4 totals and each supplier's total are flagged for that alone, and the overhead, the profit and the price inherit it. The price also says, on its own line, how many lines it leaves out and that it is "the price of the lines in the job cost, not of the job". A line with no class no longer shows an amount in the General column: it stands in no column and says so under itself, with what it comes to. | `j11_1_...` (a job with nothing assumed: $30,047.40 flagged with one line out; $44,989.93 Known with none) |
| J11-2 | The push that restores a half cent is now three units in the figure's last place, which is what the arithmetic can have lost, where it was sixteen. $300,000,000.004999 is $300,000,000.00. | `j11_2_...` |
| J11-3 | A quantity typed as a number is that number: 2.5 months at $650.00 is $1,625.00. A count worked out by a formula is still rounded up, and its quantity now says what it was rounded from; the sheet prints that. | `j11_3_...` |
| J11-4 | Hours are printed to the four places they are held and added to, on the sheet and in the recap, at the command line and in the window. | `demos/d5.sh` (16.7705 hr, 29.8328 hr) |
| J11-5 | A wall's openings are read in the order of their paths, not the order the world happens to hold them in. | `j11_5_...` |
| J11-6 | A rate is written to the cent or to every place it was entered with, by one function, in `show`, `price`, `estimate` and the window. 0.00004 is 0.00004. | `j11_6_...` |
| J11-7 | A line that has no figure at any price says its own reason in every supplier's column. A line whose condition is not known is still measured and ordered (pounds stay pounds); it is what it comes to that has no figure. A material named by its path is shown by its name. | `j11_7_...` |
| J11-8 | Cost lines on a Condition, a Material or a Type are refused, at the command line and on load. | `j11_8_...` |
| J11-9 | A room takes the status of a wall its own wall is cut to only where that end is at the room: a corner of the room lies within the walls' thickness of it. | `j11_9_...`; `j10_2_...` still passes |
| J11-10 | An element whose every line is switched off reads "none: every cost line it has is switched off", and is not listed as having no cost line. | `j11_10_to_14_...` |
| J11-11 | A total of more than $10 trillion is not given, and says why. | the same test |
| J11-12 | A supplier is named one way in a job, and so is a material: a name that differs from one already there only in capitals or spacing is refused, with the name to use. | the same test |
| J11-13 | Hours rounded up to whole hours are refused. | the same test |
| J11-14 | A percentage prints with every place it has; a material priced by the hour is refused; with an Assumed condition what a line comes to is Assumed and how much of it there is (SF, pounds, hours) is not. **Left as decided (call 33):** a line typed by hand onto an element takes that element's condition's status like any other line on it. | the same test |

**What held under attack** (the judge's list): every hand figure it worked (overhead and
profit up to 1000 %, supplier extensions, a change of material, lines typed onto a wall
and a floor of the house); Assumed overhead, prices, quotes, material and condition each
flagging what rests on them; no price with overhead or profit missing; a long list of
refusals at the command line and in a hand-edited file; save, load, save to the same
bytes on every roof file; J10-3, J10-5, J10-10 and small half-cent amounts; the five
`check_*` rows in PORTMAP; no panics.

**Not reached by the judge:** the window and the app's sheet; J10-2 at T-junctions and
two joins away; `apply` with recipes that carry classes; a material renamed or removed
in the file by hand; quantity error from four-place rounding at large unit prices.

## Twelfth judgment (2026-11-11): drawing and editing, and the fixes to the eleventh

An independent judge (Opus; it did not build any of this) was given commit `e73eb7e`, a
frozen copy of the source, the built command line and the built window, and the brief to
break the drawing stretch and the eleventh judgment's fixes. Twenty-one findings, no
panic. Its reproductions are kept outside the repository (scratch files). As it reported
them; what was done about each goes in the table that follows this one once the fix is in.

| # | Finding (the judge's words, shortened) | Severity it gave |
|---|---|---|
| J12-1 | A wall drawn on top of another wall, or back over itself (two points `--closed`), is accepted and double-counted as Known: 90 LF where 60 exist. | wrong figure unflagged |
| J12-2 | `remove` of a solved roof plane or hip writes a file that `check` then refuses, and the plane comes back on load. | the verb leaves an inconsistent file |
| J12-3 | `remove` of a hand-entered roof plane leaves its lines pointing at it, and `check` passes. | loads what it should refuse |
| J12-4 | Any run can go on any system, whatever its discipline: a duct on Domestic Cold Water. | loads what it should refuse |
| J12-5 | An opening placed by a point any distance off its wall is accepted (500 ft off). | loads what it should refuse |
| J12-6 | A wall's opening area and its door and window counts show as Known when its openings do not fit; only the first misfit is named. | wrong figure unflagged |
| J12-7 | README says the pointer snaps to "the square"; the window never turns the square on. | false claim in a document |
| J12-8 | In the window the wall being moved can snap to itself: the snap scene is kept by drawing and storey, not by what is left out of it. | minor; seen in the window once |
| J12-9 | In the window, changing storey in the middle of a chain keeps the chain's start: a wall is made on one storey from a point picked on another. | minor; seen in the window |
| J12-10 | A move by a worse status loses the wall's first source; repeated moves grow the source without limit. | minor (provenance) |
| J12-11 | A move by nothing changes the job (the source gains "; moved"; an Assumed one downgrades the status). | minor |
| J12-12 | A wall whose top is held to its own storey, or to the one below, is accepted (flagged, but it can never have a height). | loads what it should refuse |
| J12-13 | Names are not one name consistently: storeys "g" and "G", wall types "w6" and "W6", materials "Copper,16 oz" and "Copper, 16 oz."; `set material` does not match a name the way `material` refuses one. | minor |
| J12-14 | `remove` of a material that lines are made of, or of a condition in use, is accepted (flagged afterwards); the code's comment says it is refused. | false claim in a comment |
| J12-15 | Four "ours" rows in PORTMAP (`wall_type_for`, `opening_type_for`, `segment_type_for`, `system_of_class`) rest on a search for this program's own names; BIMCraft's `text.rs:282` `type_for_size` is the same rule. | false claim in a document |
| J12-16 | In the window the click that ends a Move probably also selects what is under the pointer (read in the code; not seen). | minor; not confirmed |
| J12-17 | Cost lines are accepted on a System and on a Storey. | minor |
| J12-18 | A material can still be given an order factor by the hour (`set order "2 lb/hr"`). | minor |
| J12-19 | The half-cent push is three to six units in the last place, not three as call 37 says; at $1e12 a 0.4999 cent cannot be held at all. | minor |
| J12-20 | `--status` and `--source` are silently ignored by `system` and `remove`. | minor |
| J12-21 | The window accepts tool settings that can only be refused at the second pick ("0 in, 9 ft", a system "Steam"). | minor |

**What held under attack** (the judge's list): every refused verb left the file byte for
byte unchanged, dozens of cases; what `remove` refuses (a type in use, a system with runs,
a storey with elements, the job); Unknown, empty and whitespace sources, NaN and far
points, lengths with no unit, all refused by the library, the command line and on load; a
Known room going Assumed after an Assumed move, with its figures as worked by hand;
openings that do not fit giving an Unknown net area with the reason; hand figures for a
three-piece duct and for a drawn, priced, moved and removed job; the recap by hand; undo
and redo at every step, a failed verb recording nothing, a redo after a new edit refused,
the limit of 200; snapping (rank, reach, other storeys, the skipped wall), and `toward`
at five angles; rates printed one way (J11-6); typed and worked counts (J11-3); cost lines
refused on a condition, a material and a type (J11-8); rounding at the half cent on a
search of 80,000 products (J11-2); ten ported PORTMAP rows at their lines and four "ours"
searches.

**Not reached by the judge:** J11-9 (rooms at acute and thick corners); J11-7, 10, 11 and
13; `apply` with recipes; in the window: Esc in every state, Delete with a type or a
system selected, F2 then undo, typed lengths of 0, the sheet. Two of its three window
runs were partly spoiled by its own script, so J12-8 and J12-9 rest on one picture each
and J12-16 on reading the code.

**Where the fixes stand (2026-11-11, at the commit that records this).** Each of J12-1 to
J12-6, J12-10 to J12-14, J12-17 and J12-18 is fixed in the core with a test named for it
(`j12_...` in `crates/world/tests/world.rs`, six tests, figures worked by hand); J12-20 at
the command line with a line in `demos/draw.sh`; J12-15 in PORTMAP.md; J12-19 by
correcting the document (DECISIONS.md call 41; no code changed). J12-7, 8, 9, 16 and 21
are changed in the window's code (`crates/app/src/tools.rs`, `scene.rs`) and **have not
been run in the window**: `demos/draww.sh` and `demos/clashw.sh` have not been rerun
since, so the pictures in `demos/captured/draw/` and `demos/captured/clash/` are of the
build before those five changes. The fourteen pictures in `demos/captured/clash/` are a
second run whose only difference from the run that was read is the "Not tested" heading
on the sheet; its seven model pictures and seven sheets have not each been read again.
What was decided is call 41. **None of these fixes has been judged.** All 154 tests,
the port map gate (193 items), the licence gate, the repository's three gates and the
eight command-line demonstrations pass at this commit.
