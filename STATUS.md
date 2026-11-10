# Status

Each claim gives its scope, its evidence, and what remains. Evidence is output from a
command anyone can run again from the repository root.

Workspace this was last run in: Linux, rustc 1.97.0, no display. The repository has no
remote (QUESTIONS.md Q2). It arrived here as a git bundle; on arrival `cargo test` (27
passed), `demos/d1.sh`, `tools/licenses.sh` and `tools/licenses.sh --self-test` were each
run again and each gave what this file said they gave.

| # | Deliverable | State |
|---|---|---|
| D0 | Landscape decision | Demonstrated, with limits below. Judged twice. |
| D1 | World data model and SPEC.md | Demonstrated at the command line; no window exists yet. Judged twice; the fixes from the second judgment have not themselves been judged. |
| D1 amendment | Units (owner ruling) | Demonstrated at the command line. **Not yet judged.** |
| D2 | 3D roof slice | **Not demonstrated; nothing of its demonstration runs.** One piece of groundwork is built and tested (lines read off planes). The roof solver waits on the BIMCraft source (Q10) and the window on the toolchain and a display (Q4). |
| D3 | Derived conditions and stretch-out | Not started. Reference estimate missing (Q5). |
| D4 | Macro to micro | Not started. |
| D5 | Estimate sheet and money | Not started. Reference estimate missing (Q5). |
| D6 | Interchange and reports | Not started. |
| D7 | Capture | Waiting on the owner's ruling. |

---

## D0. Landscape decision

**Demonstration as the brief states it:** DECISIONS.md and LICENSES.md are complete. No
dependency is linked before its license is confirmed.

**What was run**

1. `tools/landscape.sh > evidence/d0-landscape.txt`: for 26 crates, the crates.io registry
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
`cargo test` runs 33 checks on the same model.

The figures below are as rerun after the units amendment. The amendment added one
sheet-metal cost line to the sample job's porch headwall, which moved the counts of Known,
Assumed and Unknown values (they were 86, 13, 13 before the change and 94, 5, 13 after).
Nothing else in this demonstration moved.

**What was run and what it showed** (output of `demos/d1.sh`, exit 0)

1. *One entity per element, readable JSON.* The sample job saves as 24 entries with 24
   distinct paths. `plumbline check` confirms the file loads, its measures match its
   geometry, and saving it back writes the same bytes.
2. *One pitch changed.* `plumbline set after.json plane-0017 pitch 4 --status Known
   --source "field measured"` prints "set plane-0017 pitch = 4 in/12 (Known)" and reports
   measures re-derived on plane-0017, eave-0018, rake-0019, rake-0020, headwall-0021.
3. *The file diff.* Changes fall in exactly those five entries out of 24. On the plane:
   the pitch (3.0 Assumed to 4.0 Known, with its source) and its measure (slope factor
   1.0308 to 1.0541, surface 98.9545 to 101.1929 SF, squares 0.9895 to 1.0119). On the two
   rakes: true length 8.2462 to 8.4327 LF. On the eave and headwall: status only, because
   they are level; their length does not move but it no longer rests on an assumption.
   The script fails if any other entry changes.
4. *Flags.* Before the change `show` reports "87 Known, 15 Assumed, 14 Unknown" and lists
   each flagged value with its reason: the assumed porch pitch and the seven figures that
   follow from it, the unknown parapet height and the face area that cannot be given
   without it, the unpriced cost lines, the unbuilt schedule codes, and the cost codes
   that wait on Q1. After the change: "96 Known, 6 Assumed, 14 Unknown".
5. *A contradiction is flagged, not resolved by guess.* Steepening one plane of the hip
   roof to 8:12 leaves its two hips and the ridge with no length and the note "the planes
   disagree about the height of the line's end: plane-0003 puts it at 20.00 ft; plane-0006
   puts it at 17.50 ft".

**Hand checks** (in `crates/world/tests/world.rs`, `hand_worked_figures`)

* South plane: trapezoid (40 + 10) ÷ 2 × 15 = 375 SF on plan; 6:12 factor √1.25 = 1.1180;
  surface 419.2627 SF.
* Hip: 15 over and 15 up on plan = 21.2132 LF; climbs 7.5 ft; √(450 + 56.25) = 22.5 LF.
* Porch rake: 8 ft run at 3:12 climbs 2 ft; √68 = 8.2462 LF.

**`cargo test`: 33 passed, 0 failed.**

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
* **Rule 4 was broken twice by a judge after being reported sound.** Each judgment found
  ordinary hand edits that produced a wrong length marked Known. The fixes for the second
  set are covered by regression tests and by rerunning the judge's own cases, and have
  not been attacked by a separate agent. Treat "nothing wrong is shown as Known" as the
  model's intent backed by 33 tests, not as proven.
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
step does not do what it says. Its output is kept in `evidence/units-demo.txt`.

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

* **Not judged.** No separate agent has attacked the units change.
* **Stretch-out is the first step of D3 only.** A line's length becomes an area. Sheets,
  pounds, waste, and the conditions a parapet produces are not built.
* **A cost line cannot be added from the command line.** The demonstration uses the line
  the sample job carries. `set` changes its stretch-out.
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
| 2 | Units were decorative: 30 in read as 30 ft, metres read as feet | Each field takes one unit; anything else is refused. No conversion | `a_value_in_the_wrong_unit_is_refused_not_read_as_feet` |
| 3 | A loop in `children` passed loading and hung `show` | The tree is checked on load: one job, first, and everything leads back to it | `a_loop_of_children_is_refused` |
| 4 | Load and save silently dropped unknown fields, header keys, `inherits`; altered some floats | Unknown keys are kept everywhere except inside our own components, where they are refused by name. Numbers round-trip exactly | `nothing_is_dropped_quietly`, `a_coordinate_comes_back_as_the_same_number` |
| 5 | Deleting a component made its Unknowns vanish from the list | Every quantity is always listed; a missing input makes it Unknown with the reason | `a_missing_component_shows_as_unknown_instead_of_vanishing` |
| 6 | The license checker passed `GPL-3.0 AND (MIT OR Apache-2.0)` | Replaced with a real expression parser and a self-test | `tools/licenses.sh --self-test` |
| 7 | A pitch of 1e300 gave a Known infinity | Overflow gives Unknown; pitch above 240 in 12 and coordinates beyond 1,000,000 ft are refused | `numbers_too_large_to_mean_anything_are_refused` |
| 8 | Bowtie and collinear outlines, a ridge on a plane it does not touch, a hip naming one plane, a hip used as a plane: all accepted | Refused on load or flagged with no length | `outlines_and_lines_that_make_no_sense_are_refused_or_flagged` |
| 9 | SPEC claimed checks the code did not make; some values carry no status | SPEC rewritten against the code; the job-element checks added; what counts as a value is now stated (SPEC 2), with references and labels checked instead | `a_loop_of_children_is_refused` |
| 10 | Cost lines accepted anything | A line needs a description, a measure its element has, that measure's unit, and a non-negative price. Tying line status to quantity status is D5 | `cost_lines_have_to_make_sense` |
| 11 | `show \| head` panicked; repeated `--status` took the last; `set` hid stale measures; two siblings could share a name; `-0` stored | Each fixed. The tolerance point is now Q9. The judge's note about a BIMCraft tolerance was copied into DECISIONS.md unchecked and was wrong (second judgment, item 6) | `two_children_of_one_parent_cannot_share_a_name` and others |
| 12 | D0 evidence could not be re-run; no Linux rating per candidate | `tools/landscape.sh` and `evidence/d0-landscape.txt`; a rating table in DECISIONS.md | re-run the script |

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

**These second-round fixes have not been judged by a separate agent.** They were checked
by the tests above and by rerunning the judge's own reproductions.

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

### What exists toward D2, and what blocks the rest

None of the nine steps above runs. This is groundwork, not a demonstration.

**Built: a roof's lines read off its planes** (`crates/world/src/roof.rs`, DECISIONS.md
call 17). Given planes, it says where the ridges, hips, valleys, eaves and rakes are, and
reports planes that share an edge and disagree about its height. `plumbline lines <file>`
prints it and sets it against the lines entered. It reads the job and changes nothing.

*What was run and what it showed*
* `plumbline lines` on the sample job: 6 planes placed, 17 lines implied; all 13 roof
  lines entered in the sample agree with the planes (the porch headwall is, to the
  planes, a one-plane ridge); the 4 not entered are the deck's own edges.
* The same after `set plane-0003 pitch 8:12`: the ridge and the two hips on that plane are
  reported as running along edges their planes do not agree about, with each plane's
  heights (20.00 ft against 17.50 ft).
* Test `a_wing_meeting_a_main_roof_makes_two_valleys`: a 40 × 20 gable roof with a 10 ft
  gabled wing, entered as four planes, gives two valleys from the inside corners to
  (20, 5), two ridges, five eaves and six rakes, and no hips. Worked by hand in the test.
* `cargo test`: 36 passed.

*What remains of it*
* **Not judged.**
* It has been run on three roofs, all made up, all with planes entered by hand.
* It does not replace entered lines with solved ones. That is the solver's step.
* A change of pitch along a shared edge (a gambrel break) is reported, not classed. A
  level valley (a trough) would be classed a valley and then flagged by D1's rule that a
  valley climbs.
* An edge shared for only part of its length is split at the other plane's corners; this
  is exercised by the wing test only.

**Blocked: the roof solver** (steps 2 to 6). The call is to port BIMCraft's weighted
straight skeleton (DECISIONS.md call 3). Its source could not be read from the workspace
this stretch ran in (QUESTIONS.md Q10). Nothing was written in its place.

**Blocked: the window** (the opening proof and every step that says "in the 3D view").
Bevy 0.20.1 declares rustc 1.97.1; this workspace has 1.97.0 and no display, and updating
the toolchain needs the owner's go-ahead (QUESTIONS.md Q4).

**Found, not used yet:** the owner's earlier program has low-slope solvers (tapered
package, crickets, coping, parapet flashing height) under MIT (DECISIONS.md call 18,
QUESTIONS.md Q11). They are the candidate for step 8.
