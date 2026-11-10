# Status

Each claim gives its scope, its evidence, and what remains. Evidence is output from a
command anyone can run again from the repository root.

Workspace this was run in: Linux, rustc 1.97.0, 2 cores, no display. The repository has no
remote (QUESTIONS.md Q2).

| # | Deliverable | State |
|---|---|---|
| D0 | Landscape decision | Demonstrated, with limits below. Judged twice. |
| D1 | World data model and SPEC.md | Demonstrated at the command line; no window exists yet. Judged twice. |
| D2 | 3D roof slice | Not started. Its opening proof is blocked in this workspace (QUESTIONS.md Q4). |
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
   against 19 expressions.

**What it showed**

* Every appendix candidate is rated for maintenance, license fit and Linux, and has a
  call, in one table in DECISIONS.md.
* Thirteen entries in the starting landscape were wrong, stale or unconfirmed; they are
  tabled at the top of DECISIONS.md. The three that change the plan:
  * OpenTakeoff is now AGPL-3.0-only, not Apache-2.0.
  * A permissive weighted straight skeleton exists (BIMCraft, MIT OR Apache-2.0), which
    closes the gap the brief named.
  * Bevy is at 0.20.1, and the full engine needs a newer compiler than this workspace has.
* `tools/licenses.sh` exits 0: 62 linked crates, all MIT, Apache-2.0, Zlib, Unicode-3.0 or
  Unlicense-or-MIT. The self-test passes 19 of 19, including
  `GPL-3.0 AND (MIT OR Apache-2.0)`, which fails as it must.

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
* License data is what each crate declares in the registry. No license text inside the 62
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
`cargo test` runs 21 checks on the same model.

**What was run and what it showed** (output of `demos/d1.sh`, exit 0)

1. *One entity per element, readable JSON.* The sample job saves as 24 entries with 24
   distinct paths. `plumbline check` confirms the file loads, its measures match its
   geometry, and saving it back writes the same bytes.
2. *One pitch changed.* `plumbline set after.json plane-0017 pitch 4 --status Known
   --source "field measured"` reports measures re-derived on plane-0017, eave-0018,
   rake-0019, rake-0020, headwall-0021.
3. *The file diff.* Changes fall in exactly those five entries out of 24. On the plane:
   the pitch (3.0 Assumed to 4.0 Known, with its source) and its measure (slope factor
   1.0308 to 1.0541, surface 98.9545 to 101.1929 SF, squares 0.9895 to 1.0119). On the two
   rakes: true length 8.2462 to 8.4327 LF. On the eave and headwall: status only, because
   they are level; their length does not move but it no longer rests on an assumption.
   The script fails if any other entry changes.
4. *Flags.* Before the change `show` reports "86 Known, 13 Assumed, 13 Unknown" and lists
   each flagged value with its reason: the assumed porch pitch and the seven figures that
   follow from it, the unknown parapet height and the face area that cannot be given
   without it, the unpriced cost lines, the unbuilt schedule codes, and the cost codes
   that wait on Q1. After the change: "94 Known, 5 Assumed, 13 Unknown".
5. *A contradiction is flagged, not resolved by guess.* Steepening one plane of the hip
   roof to 8:12 leaves its two hips and the ridge with no length and the note "the planes
   disagree about the height of the line's end: plane-0003 puts it at 20.00 ft; plane-0006
   puts it at 17.50 ft".

**Hand checks** (in `crates/world/tests/world.rs`, `hand_worked_figures`)

* South plane: trapezoid (40 + 10) ÷ 2 × 15 = 375 SF on plan; 6:12 factor √1.25 = 1.1180;
  surface 419.2627 SF.
* Hip: 15 over and 15 up on plan = 21.2132 LF; climbs 7.5 ft; √(450 + 56.25) = 22.5 LF.
* Porch rake: 8 ft run at 3:12 climbs 2 ft; √68 = 8.2462 LF.

**`cargo test`: 21 passed, 0 failed.**

**What remains**

* **The demonstration runs at the command line, not in a window.** There is no desktop
  window yet; that arrives with D2. "Display as flagged" is shown as `~` and `?` marks in
  `plumbline show`.
* **The roof is not re-solved.** Hips, valleys and ridges are entered lines. A pitch change
  they cannot follow is flagged (item 5), which is honest and is not the D2 behaviour the
  brief wants (move with the pitch). SPEC.md section 7.
* **The sample job is the only job this has been run on.** No real job, no reference
  estimate.
* Only pitch, height, eave height, condition and schedule code can be edited from the
  command line. Outlines, eave lines and line ends are edited in the file.
* The limits listed in SPEC.md section 7, in particular the 0.01 ft fit tolerance (Q9).
* **Process note:** the brief asks that each demonstration be written down before the
  deliverable is built. For D0 and D1 the demonstration was written alongside the work,
  not ahead of it. D2's is written below, ahead.

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
| 1 | Re-ordering an outline's points silently turned the eave into a rake; wrong lengths marked Known | The eave is now its own line on the plane. A line has to sit on the outline of every plane it names, and an eave has to be level, or it gets no length | `reordering_an_outline_does_not_move_the_eave`, `an_eave_that_is_not_level_on_its_plane_is_flagged` |
| 2 | Units were decorative: 30 in read as 30 ft, metres read as feet | Each field takes one unit; anything else is refused. No conversion | `a_value_in_the_wrong_unit_is_refused_not_read_as_feet` |
| 3 | A loop in `children` passed loading and hung `show` | The tree is checked on load: one job, first, and everything leads back to it | `a_loop_of_children_is_refused` |
| 4 | Load and save silently dropped unknown fields, header keys, `inherits`; altered some floats | Unknown keys are kept everywhere except inside our own components, where they are refused by name. Numbers round-trip exactly | `nothing_is_dropped_quietly`, `a_coordinate_comes_back_as_the_same_number` |
| 5 | Deleting a component made its Unknowns vanish from the list | Every quantity is always listed; a missing input makes it Unknown with the reason | `a_missing_component_shows_as_unknown_instead_of_vanishing` |
| 6 | The license checker passed `GPL-3.0 AND (MIT OR Apache-2.0)` | Replaced with a real expression parser and a self-test | `tools/licenses.sh --self-test` |
| 7 | A pitch of 1e300 gave a Known infinity | Overflow gives Unknown; pitch above 240 in 12 and coordinates beyond 1,000,000 ft are refused | `numbers_too_large_to_mean_anything_are_refused` |
| 8 | Bowtie and collinear outlines, a ridge on a plane it does not touch, a hip naming one plane, a hip used as a plane: all accepted | Refused on load or flagged with no length | `outlines_and_lines_that_make_no_sense_are_refused_or_flagged` |
| 9 | SPEC claimed checks the code did not make; some values carry no status | SPEC rewritten against the code; the job-element checks added; what counts as a value is now stated (SPEC 2), with references and labels checked instead | `a_loop_of_children_is_refused` |
| 10 | Cost lines accepted anything | A line needs a description, a measure its element has, that measure's unit, and a non-negative price. Tying line status to quantity status is D5 | `cost_lines_have_to_make_sense` |
| 11 | `show \| head` panicked; repeated `--status` took the last; `set` hid stale measures; two siblings could share a name; `-0` stored | Each fixed. The tolerance point is now Q9; the BIMCraft millimetre tolerances are noted in DECISIONS.md call 3 | `two_children_of_one_parent_cannot_share_a_name` and others |
| 12 | D0 evidence could not be re-run; no Linux rating per candidate | `tools/landscape.sh` and `evidence/d0-landscape.txt`; a rating table in DECISIONS.md | re-run the script |

Not changed, and why: a line's `planes` list still carries no status of its own (SPEC 2
says so and says why); the fit tolerance stays at 0.01 ft pending Q9.

### Second judgment

Pending: a fresh agent is asked to confirm each fix by its own reproduction and to look
for what the fixes broke.

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
