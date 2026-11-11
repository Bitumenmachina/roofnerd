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
| — | The whole building (owner, 2026-11-11) | **A working model of a whole house, read and walked, not yet drawn in the window.** The owner widened the brief from the envelope to the building and its services (DECISIONS.md call 26). In the world: storeys, walls with doors and windows, floors, ducts, pipes, conduit, cable, equipment, fixtures and systems, each value flagged, with lengths, areas, bends and a takeoff by system. In the window: the whole house under a sky, seen through, a system or a storey at a time, on plan, and on foot inside. Nothing of the building can be edited in the window yet. Judged once: the wall, run and takeoff figures held and nothing crashed; it found a nested storey counted twice as Known and a run whose system was uncertain totalled without a flag, among eight. All fixed. **Those fixes are not judged.** See "The whole building". |
| D3 | Derived conditions and stretch-out | Demonstrated at the command line, except the reproduction of the owner's copper estimate, which is **pending**: no estimate is in `refs/` (Q5). Judged once: every stated figure held, and it found a negative count saved as zero, among others. All found are fixed. **Those fixes have not been judged.** |
| D4 | Macro to micro | Not started. How its demonstration will be run is written at the end of this file. |
| D5 | Estimate sheet and money | Not started. Reference estimate missing (Q5). |
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

**Handed over.** The re-base is the next session's task, set out in `HANDOFF_REBASE.md`.
`tools/portmap.sh` is the gate for it and is red until it is done: it fails today with 59
public items of the building model not yet accounted for against BIMCraft's source.

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
