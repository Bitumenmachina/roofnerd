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
| — | Fittings, connections and clashes (the owner's loop, stretch c) | **Demonstrated at the command line and in the window (`demos/fit.sh`, `demos/clash.sh`, `demos/clashw.sh`). Judged at the command line (thirteenth judgment: 18 findings, among them a missed clash, false clashes and a doubled total). Sixteen are fixed with tests (call 44), two are not (J13-14, J13-15); none of the fixes is judged, and the window part is not judged.** A fitting is worked out from the runs that meet (elbow, tee, cross, transition, coupling) and a branch on a run's side cuts the run; a run is joined to a fixture or to equipment; the takeoff says what is joined and what is open. Clash detection finds what is in the same space with how far and how sure, lists runs through walls and floors apart, and tests a clearance only when one is asked for. See "Fittings, connections and clashes. What was run". |
| — | Plans under the tracing (order step 2) | **Demonstrated at the command line and in the window (`demos/plan.sh`, `demos/planw.sh`). Not judged.** A PDF (drawn or scanned), a PNG or a JPEG is laid under a storey, given its scale from one dimension and traced over; what is traced is Assumed and names the drawing; a drawn PDF's own lines are snapped to. A real public sheet, USDA Plan No. 7213, was traced in the window to its dimensions: 130.33 LF of outside wall. Only walls are traced so far. See "Plans under the tracing. What was run". |
| D6 | Interchange and reports | Not started. |
| D7 | Capture | Waiting on the owner's ruling. |

## The release candidate (Patrick's ruling 12, 2026-11-11)

What is owed now is these thirteen items (CLAUDE.md, "The deliverable"; DECISIONS.md
call 42). Each is done when this table points at its evidence. The D rows above are the
record of what was built and judged before the ruling. The order the work is done in is
CLAUDE.md, "Order of work"; step 1 of it is under way.

| # | Item | Where it stands |
|---|---|---|
| 1 | A clean clone installs on Linux from the README in one pass | **Not shown.** The README lists the commands; nobody has followed it on a clean machine. |
| 2 | Every import works as stated: PDF (vector and scanned), PNG, JPEG, aerial images, CSV and XLSX (price lists, prior estimates, assembly libraries), IFC, the job file | **Each import named by the rule is read at the command line; only plans are in the window.** The job file loads and saves to the same bytes. A PDF plan (drawn or scanned), a PNG and a JPEG are laid under a storey and traced over, with what each could not give listed (`demos/plan.sh`, `demos/planw.sh`; judged, fixes not judged). A CSV or XLSX price list, an assembly library and a prior estimate are brought in with every unused row and column said, and a price list and an assembly library go out as CSV and come back the same (`demos/imports.sh`; not judged). A plain IFC model comes in as storeys, walls, openings, floors, rooms and runs, with the rest listed (`demos/model.sh`; not judged; one made-up model only). Not built: tables and models in the window; an aerial picture shown as such; an estimate's own lines out and back. |
| 3 | From an empty job: open a real residential plan PDF, calibrate, trace walls, openings, floors, rooms and the roof; the 3D house builds from the tracing | **Part.** From an empty job the window opens USDA Plan No. 7213 (public domain; a scan), sets its scale from its 41 ft 3 in dimension, and traces its outside walls; the 3D house stands on the sheet (`demos/planw.sh`, pictures 01 to 08; not judged). Not built: openings, floors, rooms and the roof traced over it. |
| 4 | Clashes show their certainty; the VIF list is sorted by dollars | **Half.** A clash is Known or Assumed from its two shapes and says so (command line and the sheet's Clashes tab; not judged). There is no VIF list and no dollars on a clash. |
| 5 | A typed or lasered measurement makes a value Known; the clash confirms or clears; the price changes live in the second window | **Demonstrated, not judged** (`demos/field.sh`, `demos/fieldw.sh`; "Field entry. What was run"). A reading typed as a Leica DISTO in keyboard mode types it, ended by its Tab or Enter, makes a value Known in the window; one clash clears and one becomes a finding; the sheet's price goes from $615.86 to $607.31. No laser was in the loop: the keys were typed by `xdotool` in a burst. A wall's length is not measured as one length yet. |
| 6 | A design-phase price per SF or LF grows into line items for the same condition, and the detail rolls back up | **Demonstrated on the roof** (D4: `demos/d4.sh`, `demos/d4w.sh`), judged, fixes not judged. Not shown on a wall, a floor or a run. |
| 7 | The sheet: stretch-out editable on the line, labor switchable per line, editable overhead and profit, three suppliers side by side | **Three of four in the window** (D5: `demos/d5w.sh`): the labor switch, overhead and profit, three suppliers. Stretch-out is editable at the command line and not on the line in the window. |
| 8 | Exports: xlsx with live formulas, CSV, IFC4 validated by IfcOpenShell; supplier, subcontractor and PM lenses from one estimate | **Not started** (Q17 for what each lens holds). |
| 9 | A judge lists every word on every screen; none is a software term | **Tried once and failed** (fifteenth judgment, part B): paths, command lines, a backslash before every inch mark, and the machine's own error words reach the screen. Thirteen findings, none fixed yet. |
| 10 | A fresh judge, briefed as a veteran estimator with no software background, completes a whole-house takeoff from a PDF using only the in-app help | **Not possible yet**: no PDF, no in-app help. |
| 11 | The visual judge passes every screen against the instrument standard | **Not started.** The 3D view still has the sky, sun and grass that ruling 7 removes. |
| 12 | A fresh judge walks the whole path cold and cannot break the money path | **Not done.** Twelve judgments so far, each of one stretch; the last fixes of D1, D2, D3, D4, D5 and drawing are not themselves judged. |
| 13 | A hand estimate in `refs/` is reproduced, every difference explained line by line | **Pending:** `refs/` has no estimate (Q5). |

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
  The thirty pictures are in `demos/captured/draw/` and each was opened and read.

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

**Judged: the twelfth judgment, below.**

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

## Plans under the tracing. How its demonstration will be run (written before it is built)

Step 2 of the order of work; DECISIONS.md call 43. Two demonstrations.

* `demos/plan.sh`, at the command line. (1) The made-up vector plan
  (`demos/plans/made-up-plan.pdf`, written by `tools/made-up-plan.py`: a 30 ft by 20 ft
  house drawn at 1/4 in to the foot, so 30 ft is 7.5 in, 540 points, on the page) is
  added to an empty job as a drawing of its storey. Reading it reports the page's size,
  how many straight lines it holds, and what was lost. Calibrated from the two ends of
  its 30 ft dimension line, 540 points apart: 18 points to the foot. A wall traced
  between two page points 360 points apart is 20.00 LF, Assumed, and says which drawing
  it was traced from. (2) A PNG and a JPEG of the same plan are added the same way and
  report that a picture has no lines to snap to. (3) What is refused: a file that is not
  there, a page the file does not have, a scale from two points on top of each other, a
  tracing over a drawing with no scale; and a drawing whose file has changed since it
  was added is said to have changed. (4) If `tools/fetch-plans.sh` has fetched USDA Plan
  7213, its floor-plan sheet is added and read: a scan, one picture, no lines.
* `demos/planw.sh`, in the window, on USDA Plan 7213's floor plan from an empty job: the
  sheet is added, shown under the plan, and calibrated on its 41 ft 3 in overall
  dimension; the outside walls are traced over it, each side by a typed length off the
  drawing (41 ft 3 in by 23 ft 11 in: 130.33 LF round, 1,042.67 SF of face at the 8 ft
  0 in the wall section gives). The saved job has those figures, Assumed, traced from
  that drawing. Then the made-up vector plan: the pointer snaps to the PDF's own line
  ends, and the bar says so.

The judge tries to trace at the wrong scale without being told, to calibrate into a
Known figure, to lose a drawing or a tracing on save and load, and to get the program to
read a file it should refuse.

## Plans under the tracing. What was run and what it showed

Step 2 of the order of work, on commit `939e6aa` and the one after it, which holds the pictures. **Not judged.**
What was built: a Drawing (one page of a PDF, a PNG or a JPEG laid under the storey it
is a plan of); its scale from one dimension; lining it up; tracing over it with the
tools that were there, each thing traced flagged Assumed and naming the drawing; a
crate that reads plan files and says what a page could not give; in the window, the
sheet under the plan, the wheel closing in on what the pointer is on, the scale picked
on the sheet, and the pointer snapping to a drawn PDF's own lines. SPEC.md section 5.11;
DECISIONS.md call 43.

**`demos/plan.sh`** (command line), exit 0:

| Step | Worked by hand | The program |
|---|---|---|
| The made-up vector plan read | a letter sheet on its side, 792 by 612 points; 8 straight lines (outline 4, partition, dimension line, 2 ticks); 1 curve (the door swing) | 792 by 612 points, 8 straight lines to snap to; not read: 1 curved piece of line, the words and figures |
| A wall traced before a scale | refused | refused: "has no scale: calibrate it from one dimension"; file unchanged |
| Scale: 30 ft between two places 540 points apart | 18 points to the foot | 18 to the foot |
| The outline traced by four page places | 540 points is 30.0 ft, 360 points is 20.0 ft; south face 30 x 8 = 240 SF | 30.0, 20.0, 240.0, each Assumed, "traced from A-1 floor plan (made-up-plan.pdf, page 1)" |
| The same plan as a PNG and as a JPEG, two dots to the point | 1,584 by 1,224 dots; 1,080 dots for 30 ft is 36 to the foot | the same; "it is a picture: it has no lines of its own to snap to" |
| Refused, file unchanged | a missing file, page 2 of 1, a file that is no plan, two points that are one, a dimension with no unit, a status given with `--on`, `--on` a wall | all seven |
| The plan file changed after the drawing was made | said | "file: HAS CHANGED since this drawing was made from it" |
| USDA Plan No. 7213, sheet 4 | a scan | page 4 of 4, 789 by 613 points, a scan, nothing to snap to |

**`demos/planw.sh`** (the window, two sessions from empty jobs), exit 0, 22 pictures in
`demos/captured/plan` and `demos/captured/plan/made-up`, every one read:

| Picture | What it shows |
|---|---|
| 01 | USDA Plan No. 7213's floor-plan sheet under the Ground storey, whole. The left panel lists it with "no scale yet" in red; selected, the right panel says Scale: Unknown, none, and what was not read from it (a scan, nothing to snap to; words and figures not read). The title bar counts 1 unknown. |
| 02 | Two steps of the wheel in on the floor plan: the dimension figures on the scan are readable. A wall tried: refused in the banner and the bar, "has no scale: set it from one dimension before tracing over it". |
| 03 | The two ends of the 41 ft 3 in overall dimension picked, a line between them, and `41' 3"` being typed. |
| 04 | To scale: the panel says "sheet 88' 2" x 68' 6"" in the Assumed colour; the count of unknowns is gone. |
| 05, 06 | The first outside wall going east with `41' 3"` being typed; then three walls, the pointer on the first corner about to close: 106.42 LF, 851.33 SF. |
| 07 | Four walls in ink on the sheet's own outside wall lines, all four sides: Walls 4, 130.33 LF, 1,042.67 SF, Assumed. |
| 08 | The same in 3D: the four walls standing on the sheet. |
| made-up/01 | The vector plan under the storey; with the scale tool the pointer is on the west end of the PDF's dimension line and the bar says it is on an end. |
| made-up/02, 03 | The outline traced by four clicks near its corners: 100.00 LF, 800.00 SF. The drawing, selected, reads `1/4" = 1'-0"`, "8 straight lines to snap to", and what was not read. |

The saved jobs, read back by the script:

| Figure | Worked by hand | In the saved job |
|---|---|---|
| The farmhouse's outside walls | 41 ft 3 in is 41.25; 23 ft 11 in is 23.9167 to the ten-thousandth of a foot a place is kept to; 2 x (41.25 + 23.9167) = 130.3334 LF; x 8 ft = 1,042.6672 SF | 4 walls: 41.25, 23.9167, 41.25, 23.9167; 130.3334; 1,042.6672; each Assumed, "traced from usda-7213, page 4 (usda-7213.pdf, page 4)" |
| The farmhouse sheet's scale | 41 ft 3 in read off the sheet, Assumed; on the whole sheet (766 pixels wide) the dimension is 360 pixels, so the sheet is about 41.25 x 766 / 360 = 87.8 ft wide | 41' 3" over 369.2 points, Assumed, "read off the drawing"; the sheet 88.2 ft wide |
| The vector plan's scale | the dimension line's ends are (126, 90) and (666, 90): 540 points for 30 ft, 18 to the foot | from (126, 90) to (666, 90) exactly |
| The vector plan's outline | (126, 126) and (666, 486) over 18: (7, 7) to (37, 27); 30, 20, 30, 20 ft | walls from (7, 7), (37, 7), (37, 27), (7, 27): exact, with no length typed |

**What is not right, or not shown:**

* **Only walls are traced.** Openings, floors, rooms and the roof over a drawing are
  release-candidate item 3 and are not shown; floors, rooms and roofs cannot be drawn in
  the window at all yet (order step 5).
* **Lining a drawing up is in the window and no demonstration clicks it.** It is used
  at the command line in `demos/plan.sh`.
* **A PNG or a JPEG is read at the command line; none is laid in the window in a
  demonstration.** The window reads them by the same code.
* **The scan lies a quarter of a degree crooked on its page** and a drawing cannot be
  turned: the sheet drifts about 2 1/2 in under the traced walls from one end of the
  house to the other. PROPOSALS.md 19.
* **Two hand figures of mine were wrong and the program right**, both in the script's
  own checks and both corrected before this was written: the sheet's width (I had 52 to
  53 ft, from pixels of one zoom and points of another; it is 88 ft) and the total
  round the house (130.3333 against the 130.3334 that 23.9167 gives).
* **A typed length is kept to a ten-thousandth of a foot**, so 23 ft 11 in is 0.0004 in
  long. It shows as 23.92 and adds as 130.33.
* **The paper shows grey**, and the sky and grass are still behind the 3D view: the
  look is order step 7.
* **A page is read at up to 4,096 dots on its long side.** Close in on a big sheet it
  will blur; sharper pieces on zooming in are not built.
* **A PDF with more than 400,000 straight lines** keeps the first 400,000 and says so;
  how the pointer's snapping holds up near that many has not been tried.

**What changed that was so before:**

* The left panel on plan has a new section, "Drawings to trace", between the storeys
  and the drawing tools. The tools are 62 pixels lower; `demos/draww.sh` and
  `demos/clashw.sh` click the new places.
* On plan a wall is drawn with its two faces and ends in ink.
* On plan the wheel closes in on the place under the pointer, not on the middle of the
  window; a drag moves the plan by exactly what a pixel covers.
* A wall type, an opening type and a run type made by a drawing tool say "set in the
  3D view" as before; what is traced with them says which drawing.
* The dependencies are built optimised in a debug build (`[profile.dev.package."*"]`):
  reading the scanned sheet took 65 s without it.
* A placed thing's angle shows as "180 deg"; it showed "180° deg".

**What was run on this build:** `cargo test --workspace` (164 passing);
`tools/portmap.sh` (193 items, PASS); `tools/licenses.sh` (391 crates, exit 0; LICENSES.md
brought up to it, 33 more than before: hayro's and image's); the repository's three
gates (exit 0); `demos/d1.sh`, `units.sh`, `d3.sh`, `d4.sh`, `d5.sh`, `draw.sh`, `fit.sh`,
`clash.sh`, `plan.sh` (each exit 0); `demos/planw.sh`, `demos/clashw.sh`, `demos/draww.sh`
(each exit 0; 22, 14 and 30 pictures, every one read). `demos/draww.sh` was run one small
change before the last (a place on a page rounded to a millionth of a unit, an angle
shown with one unit, one line of the Clashes tab shortened), none of which it shows. **Not rerun on this build:** `demos/d2.sh`, `d4w.sh`, `d5w.sh` and
`house.sh`; their pictures under `demos/captured` are of commit `b566079` and the
window's drawing of walls has changed since.

## Tables brought in. How its demonstration will be run (written before it is built)

Step 3 of the order of work; DECISIONS.md call 45. `demos/imports.sh`, at the command
line, on the priced sample job (`plumbline priced`), whose copper is bought by the pound:
"Copper, 16 oz" at $9.00/lb with quotes from three made-up suppliers, and "Copper, 20 oz"
at $9.00/lb with quotes from two of them.

1. **A price list out and back.** The job's price list is written as CSV, read back
   into a copy of the job from before its prices were entered, and the two jobs' price
   lists are the same, value for value, status for status, source for source. Then the
   CSV written from the copy is byte for byte the first CSV.
2. **A supplier's own list.** A made-up list from one of the suppliers, with its own
   headings (Item, UOM, Price, Lead time) and five rows: the 16 oz copper at a new
   price; the 20 oz copper, which that supplier had not quoted, typed "copper,20 OZ" and
   "LB"; a metal the job does not have; the 16 oz copper again by the sheet; and a row
   whose price is the word "call". Brought in for that supplier: two prices set (one
   changed, one new), three rows listed as not used, each with why, and the column
   "Lead time" named as not used. The estimate's side-by-side then prices every copper
   line at that supplier, where two were left out before; each new figure is the line's
   pounds times the new price, worked by hand from the pounds the job shows.
3. **The same list as a workbook.** The same five rows as an XLSX file with a second
   sheet and one price that is a formula: the same two prices set, and the report also
   says the second sheet was not read and one cell's formula was read as its value.
4. **An assembly library out and back**: the job's conditions as CSV, into an empty
   job, and out again: the same bytes.
5. **An estimate's lines out and back** is moved to order step 8, with the estimate's
   own CSV export (decided before any of this was built: the estimate's table is the
   export's to define).
6. **A prior estimate set against the job.** A made-up prior estimate of four lines
   (description, quantity, unit, unit cost, amount), two of which match lines of the job
   by description and two of which do not. `compare` lists each prior line beside the
   job's line of that name with the difference in quantity, unit cost and amount worked
   by hand, lists the prior lines the job has nothing for and the job's lines the prior
   estimate has nothing for, and adds none of it to the job's price: the recap is what
   it was.
7. **Refused, and the job left as it was:** a file that is neither CSV nor XLSX; a list
   with no price column; a workbook sheet that does not exist; a price list brought in
   with no source.

IFC and an aerial picture are the second half of this step and get their own
demonstration when the first half is judged.

## Tables brought in. What was run and what it showed

The first half of step 3, on the commit that holds this. **Not judged.** Built: a crate
that reads a CSV file or a workbook's sheet into rows and says what it did not read; a
price list, an assembly library and a prior estimate as tables (SPEC.md 5.12;
DECISIONS.md call 45); `export`, `import` and `compare` at the command line. Nothing of
it is in the window yet.

**`demos/imports.sh`**, exit 0:

| Step | Worked by hand | The program |
|---|---|---|
| The job's price list out, into an empty job, and out again | two coppers, each its own price and order factor, three and two suppliers' prices: 7 rows | 7 rows; the second CSV is byte for byte the first; into the job it came from: "the same as the job already has: 7", nothing set |
| A supplier's list, CSV: 2 rows above its headings, 5 rows | 2 prices to set (one changed from $9.10 to $9.05, one that supplier had not given, typed "copper,20 OZ" by the "LB"); 3 rows not usable (a material the job has not, the wrong unit, the word "call" for a price); 1 column with no use | set 2, not used 3 with those reasons, "not read: 2 rows above the headings", "not read: the column \"Lead time\"" |
| The copper lines at that supplier | pounds = generic amount / $9.00: 461.1889, 276.7133, 123.6933, 30. x 9.05 or 9.40: $4,173.76, $2,504.26, $1,162.72, $282.00; total $8,122.74; no line left out | the same five figures; "4 of 4" under that supplier, where it was "2 of 4" |
| The same list as a workbook with a second sheet and a formula | the same two prices; the sheet and the formula said | the same job, byte for byte, as the CSV left; "1 other sheet was not: \"Terms\""; "1 cell holds a formula" |
| The assemblies out, into an empty job, and out again | 3 conditions | 7 rows; the second CSV is byte for byte the first |
| A prior estimate of 4 lines set against the job | panels: 700 lb, $5,950.00 against 861.60 lb, $7,754.36 on three lines (+161.60 lb, +$1,804.36), Assumed because the shed's is; underlayment: 650 SF, $585.00 against 670.82 SF, $603.74 (+20.82 SF, +$18.74); gutters, 120 LF x $14.00 = $1,680.00 worked, and a dumpster, $450.00, in no line of the job; 7 kinds of line in the job that the prior does not name; the prior adds to $8,665.00 | the same, figure for figure; the job's cost $25,292.43 before and after |
| Refused, job unchanged | a file that is no table; no price column; no unit column; a sheet that is not there; a status with no source; Unknown as a status; a prior estimate with no source; a supplier's list as an assembly library; an option the table does not take | all nine |

`cargo test --workspace`: 171 passing (4 on the tables, 3 on the reader, new).
`tools/licenses.sh`: 411 crates, exit 0 (20 more: csv's, calamine's). Portmap and the
three repository gates: exit 0.

**What is not right, or not built:**

* **None of this is in the window.** A list cannot be dropped on it or brought in from
  it. That is still owed for release-candidate item 2.
* **An assembly library is only read in this program's own form.** Somebody else's
  library has no agreed headings to find; none has been seen.
* **An estimate's own lines do not yet go out and come back** (moved to step 8).
* **`compare` matches lines by what they are called and nothing else.** A prior line
  called "SS panels" and the job's "Standing seam panels" are two things to it. Cost
  codes are read and not yet used to match.
* **A prior line's unit is compared as written**: "LF" and "lf" agree, "LF" and "ft" do
  not, and then the quantities are not compared, which is said.
* **A date in a workbook is read as the workbook's day number.**
* **IFC and an aerial picture, the second half of this step, are not started.** BIMCraft
  reads IFC2x3 and IFC4 (`import.rs`, 4,895 lines, over `step.rs`) and writes IFC4
  (`ifc.rs`); both are to be ported, item by item.
* **The language pass over every screen is not started.**

## A model brought in. What was run and what it showed

The second half of step 3's imports, on the commit that holds this. **Not judged.** Its
demonstration was not written in STATUS before it was built, as the practice asks; the
made-up model's own description (`tools/made-up-ifc.py`, written first) holds the hand
figures. Built: BIMCraft's STEP reader ported into a new crate with its three tests,
and an importer over it (SPEC.md 5.13; DECISIONS.md call 47); `import ifc` at the
command line. Nothing of it is in the window.

The made-up model (`demos/ifc/made-up-house.ifc`, 253 entities, IFC4, in metres) was
written by hand and checked once with IfcOpenShell's validator as a separate program: 0
messages.

**`demos/model.sh`**, exit 0:

| Step | Worked by hand | The program |
|---|---|---|
| The model into an empty job | two storeys, four walls, a door, a window, a slab, a space, a duct, a pipe; a beam, a stair and a chair with no element here | 2 storeys, 4 walls, 1 door, 1 window, 1 floor, 1 room, 1 duct, 1 pipe; "not brought in: 3 things", each named with why; "the model is in metres" |
| Walls | 9.144 m and 6.096 m are 30 and 20 ft; 2.4384 m is 8 ft: 100 LF, 800 SF gross, less a 3 x 7 door and a 4 x 3 window, 767 SF | 100.00 LF, 800.00 SF, 767.00 SF, Assumed |
| The door and the window | middles 3.048 m, 10 ft, along their walls; the window's sill 0.9144 m, 3 ft, up | 10 ft and 0; 10 ft and 3 ft (the test) |
| The slab and the room | 30 x 20 = 600 SF; inside 6 in walls on their centre lines 29.5 x 19.5 = 575.25 SF | 600.00 SF, 575.25 SF |
| The duct and the pipe | 6.096 m = 20 LF; 3.048 m = 10 LF | 20.00 LF on Supply Air, 10.00 LF on Domestic Cold Water |
| The clash the model hid | both 7 ft up, crossing at (10, 10): a 6 in duct and a 1 in pipe, 3 in + 1/2 in | "in each other by 3.50 in, at (10, 10, 7)", to verify in the field |
| Refused, job unchanged | no status and source; a file that is no model; an option it does not take | all three |

`cargo test --workspace`: 180 passing (2 on the importer, new).

**What is not right, or not built:**

* **It has met one model, and that one made up and plain.** No model written by a design
  program has been brought in: none with a clear licence was looked for yet. What such a
  model does with wall axes, layers, mapped shapes and types is not tried.
* **Only plain pushed shapes are read.** A wall cut to a roof, a curved wall, anything
  drawn as a mesh or a brep, a mapped item (a shape used many times) is listed, not
  brought in. BIMCraft reads those; that part is not ported.
* **Fixtures, equipment and fittings are not brought in at all** (flow terminals, air
  handlers, elbows): listed as having no element, though this program has elements for
  them. A model's clashes against equipment are therefore not found.
* **A wall is always taken on the middle of its body.** Its layers and which face it
  was drawn to are not read.
* **A space is a point**: the room's area is found from the walls that came in, not
  read from the model, and a space with no walls round it has no area.
* **Roofs are not brought in.**
* **Not in the window.**
* **An aerial picture** is a picture: it is laid under a storey, given its scale from
  one known length and traced, by the same code as a scanned plan
  (`demos/plan.sh` reads a JPEG and a PNG). No demonstration uses an aerial picture as
  such, and none is committed.

## The language pass. What was done

The last part of step 3; DECISIONS.md call 48. **Not judged; no screen has been looked at
since it was made** (the window demonstrations are being run again on this build, and
every picture is to be read before any of this is called shown).

* Every string either window draws now passes through one function (`theme::spoken`): a
  thing's path in the file is said as the thing's name, a figure's name is said as its
  words, and the machine's number for a failure is left out. Held by a test in the
  window's crate.
* Paths are gone from under each title, beside each row of the sheet and under each
  clash.
* A formula can be typed as its words (`plan length * stretch out`). Held by a test.
* The places the pointer catches on are said as at a drawing board.
* `cargo test --workspace`: 182 passing. The eleven command-line demonstrations exit 0.

**Not done:** release-candidate item 9 (a judge lists every word on every screen) is
what settles this, and has not been run. Known already: the core still says "a Wall",
"a Duct" in some refusals, as the model names kinds.

## Field entry. How its demonstration will be run (written before it is built)

Order step 4; DECISIONS.md call 49 has what a laser types and what a measurement does.
Two demonstrations, on one made-up job, every figure worked here first.

**The job** (made at the command line, as a tracing would leave it): a ground storey at
0 ft, 9 ft to the next, and an upper storey at 9 ft, both Assumed, "scaled off a made-up
plan". Four 6 in walls round 24 ft by 16 ft, Known, standing to the upper storey. A
12 in by 8 in duct along the middle from (2, 8) to (22, 8), its centre 8 ft above the
floor, line and height Assumed. Two 2 in pipes across it, Known: pipe A at (12, 2) to
(12, 14) with its centre at 7 ft 9 in, pipe B at (18, 2) to (18, 14) with its centre at
8 ft. The south wall carries one line, gypsum board on its net face at $2.40/SF, classed
material. Overhead 10, profit 8.

**Before any measurement, by hand:**

* The duct is 8 in deep about 8 ft: from 7 ft 8 in to 8 ft 4 in. Pipe A is 2 in about
  7 ft 9 in: from 7 ft 8 in to 7 ft 10 in. They are in each other by 2.00 in. Pipe B is
  from 7 ft 11 in to 8 ft 1 in, wholly inside the duct's depth: the least way out is
  5.00 in, up or down. **Two clashes, both to verify in the field** (the duct and the
  ground storey are Assumed).
* The south wall: 24 ft by 9 ft, 216 SF, at $2.40, **$518.40**, flagged Assumed (its
  top is the upper storey's Assumed floor). Overhead $51.84; profit 8% of $570.24,
  $45.62; **price $615.86**.

**The measurements,** each typed the way the laser types it:

1. The ground storey's floor: `same`. It is the floor everything is measured from.
   Known, value unchanged, 0 ft.
2. The ground storey's height, floor to floor: `8'10"1/2` and Tab. 8 ft 10 1/2 in is
   8.875 ft. The upper storey's floor is now at 8 ft 10 1/2 in, Known (a Known floor
   plus a Known measurement). The south wall is 24 by 8.875, **213 SF, $511.20**, no
   longer flagged. Overhead $51.12; profit 8% of $562.32, $44.99; **price $607.31**:
   $8.55 less.
3. The duct's line on the plan, and which type it is (a 12 by 8): `same`, each. Known.
   (The type was not in this plan as first written: a traced run's choice of type is
   Assumed too, and the clash rests on it. Found when the verb was first run.)
4. The duct's height, shot to its underside: `bottom 7'10"3/4` and Enter. The duct is
   now from 7 ft 10 3/4 in to 8 ft 6 3/4 in. Pipe A's top is at 7 ft 10 in: 3/4 in
   clear. **That clash clears.** Pipe B, 7 ft 11 in to 8 ft 1 in, is still inside the
   duct: down is now 2 1/4 in (its top to the duct's underside), up is 7 3/4 in. **It
   is in the duct by 2.25 in, and it is now a finding**: everything it rests on is
   Known. One clash, a finding, where there were two questions.

**`demos/field.sh`, at the command line:** the job; the figures above before; what a
laser types read as lengths (`2'2"13/32` is 2 ft 2 13/32 in, `26"13/32`, `23.46ft`);
what is refused and that the file is byte for byte what it was after each: a figure
with no unit, a comma, metres, a fraction after a fraction, `same` on a value that is
Unknown, a length for a line on the plan, a storey height that contradicts a Known
floor above, `bottom` on a sloping run held to its centre; then the measurements
with `plumbline measure` and the figures above after.

**`demos/fieldw.sh`, in the window:** the same job opened with both windows up, the
sheet on Clashes, then on the Estimate. The storey, then the duct, picked on the model;
each value picked in the panel on the right and the reading typed by `xdotool` as a
keyboard that types it in one burst, ended by Tab or Enter. A picture before, after the
storey height (the sheet's price), and after the duct's height (the Clashes tab and the
marks on the model). The saved job is then read at the command line and has to say
exactly what `demos/field.sh` ended with.

The judge tries to get a value marked Known that nobody measured, to have a measurement
change a figure it should not or fail to change one it should, to find a field that
takes a length and will not take what the laser types, and to break the sheet's price
with a measurement.

## Field entry. What was run and what it showed

Built as the plan above has it (DECISIONS.md call 49; SPEC.md 5.14). Not judged.

**What was built.**

* `units::parse_length` reads a fraction after the inch mark, the way a laser's display
  writes it (`2'2"13/32`, `26"13/32`). A figure with no unit, a comma and metres are
  refused as before; the first two refusals now say how a laser is to be set.
* `editing::measure` (`crates/world/src/editing/measure.rs`), one verb for the command
  line and the window: a length replaces an entered value and it is Known; `same` keeps
  the value and marks it Known; a storey's height moves the floors above; a run's
  height can be taken to its underside or its top. `editing::takes` says which values
  of a thing a measurement can be put on.
* `plumbline measure <file> <path> "<value>" <reading|same> [--to bottom|centre|top]
  --source "..."`.
* In the window: every value in the right-hand panel that can be measured says
  **Measure** (one length) or **Check** (anything else) and a click on it asks for the
  reading. A storey's row on the left now picks the storey as well as putting it on
  show, so its floor and its height are on the right. An entry is ended by Enter or by
  Tab; where an entry takes several things, Tab goes on to the next.

**`demos/field.sh`** (the command line): PASS. Every figure of the plan came out as
worked by hand: two clashes to verify, 2.00 in and 5.00 in; the wall 216.00 SF,
$518.40, price $615.86, flagged; after the storey height `8'10"1/2`, the upper floor at
8' 10 1/2", Known, the wall 213.00 SF, $511.20, price $607.31, no flag; after the
duct's underside `7'10"3/4`, one clash, a finding, 2.25 in, and pipe A 0.75 in clear.
Ten refusals, the file byte for byte the same after each. A second storey height that
contradicts the floor above once that floor is Known is refused and names the three
values.

**`demos/fieldw.sh`** (the window): PASS. Each reading typed by `xdotool` in one burst
(20 ms a key), ended by the key a laser sends: `same` and Enter; `8'10"1/2` and Tab;
`same, JM` and Enter; `bottom ` typed by hand and then `7'10"3/4` and Enter. The job the
window saved is **byte for byte** the job the command line makes from the same five
measurements. Pictures in `demos/captured/field`, each read:

| Picture | What it shows |
|---|---|
| 01 | The storey picked: Elevation 0 ft and Storey height 9 ft, both Assumed, each with Measure. The sheet's Estimate: 216.00 SF, $518.40, Price $615.86, every figure flagged, "$518.40 of it rests on something assumed". |
| 02 | The entry: "Measured on site: elevation of ground", and under it "As the tape or laser reads, or same". |
| 03 | Storey height 8' 10 1/2", "measured on site", no flag; Upper at 8' 10 1/2" in the list of storeys; the message says the upper floor moved and that it is Known. The sheet: 213.00 SF, $511.20, Price $607.31, no flag anywhere on it. |
| 04 | The duct picked on the plan: Type, Line on the plan (Check) and Height above its storey (Measure), all Assumed. Both crossings marked on the plan. The sheet's Clashes: 2, "2 to verify in the field", 2.00 in and 5.00 in. |
| 05 | The duct's three values Known with their sources; "Its heights are those of its bottom"; one crossing marked. The sheet's Clashes: 1, "1 finding", 2.25 in. |

**Tests.** `crates/world/tests/field.rs`, ten: what a laser types; the figures above; a
floor only as sure as the floor below it; a Known floor above refuses the measurement;
floors further up move by the same and a Known one stops the move; what a measurement
will not do, each leaving the job as it was; `same` on a value nobody knows; a sloping
run; a run with no size.

**What this changed that was so before:**

* `1,200 ft` is refused with "has a comma, and 1,234 is a thousand to one reader...";
  it was refused with "is not a measurement this program can read". One test's expected
  words changed (call 49).
* A length with no unit is refused in the same words, with a sentence about a laser
  added.
* `2' 6" 1/2` and `26" 1/2` are read; they were refused ("something after the inches").
* A click on a storey's row picks the storey (the bar says "selected ground" and the
  right-hand panel shows it); it only put the storey on show before. Delete with a
  storey picked tries to remove the storey, and is refused while anything is on it.
* Tab in an entry ends it, or goes on to its next part; it did nothing in an entry.

**What is not right, or not done:**

* A wall's or a run's length cannot be measured as one length: it would move an end and
  what is joined to it. A line is moved and then checked.
* A laser shot with no entry open types into whatever the keys do.
* The hint under the entry wraps to two lines.
* The pictures show two faults that were there before: a sheet row says its name twice
  ("wall 7 wall 7"), and Keys offers "Next roof plane" on a job with no roof. Both are
  in the fifteenth judgment, part B.
* Only `demos/fieldw.sh` has been run on this code. The other window pictures are still
  those of `719ff91` or before.

## Floors, rooms and roofs drawn in the window. How its demonstration will be run (written before it is built)

Order step 5; release item 3 wants walls, openings, floors, rooms and the roof traced
from a plan, and the 3D house built from the tracing. DECISIONS.md call 53 has what is
ported and what each tool does. One demonstration at the command line and one in the
window, on the made-up vector plan (`demos/plans/made-up-plan.pdf`: a house 30 ft by
20 ft drawn at 18 points to the foot, a partition 12 ft from its west side), every
figure worked here first.

**The house, traced:** four outside walls on the outline and the partition, 6 in thick,
8 ft tall, on their centre lines, as `demos/planw.sh` already traces them.

* **The floor**, by the outline its walls enclose: the walls' centre lines, 30 ft by
  20 ft: **600.00 SF**. Assumed, since the walls are traced; its source says it is the
  outline of the walls.
* **Two rooms**, each a point picked inside it and a name. Inside the walls' faces: the
  west room is from 3 in to 11 ft 9 in east and 3 in to 19 ft 9 in north, 11.5 ft by
  19.5 ft, **224.25 SF**; the east room is from 12 ft 3 in to 29 ft 9 in, 17.5 ft by
  19.5 ft, **341.25 SF**. Together 565.50 SF: the floor less what the walls stand on.
* **The roof**, by its footprint (the outline's four corners), a pitch of 6:12 on every
  side and its eaves 8 ft up: a hip roof. Its ridge is 30 less 20, **10 LF**, and
  stands 20 / 2 x 6/12 = 5 ft above the eaves, at 13 ft. Each hip runs 10 ft east, 10 ft
  north and 5 ft up: 15 ft; four of them, **60 LF**. Eaves **100 LF**. On plan the two
  long sides are (30 + 10) / 2 x 10 = 200 SF each and the two ends 20 x 10 / 2 = 100 SF
  each; on the slope each is that times the square root of 1.25 (1.118034): 223.61 SF
  and 111.80 SF. **670.82 SF** in all, 6.71 SQ.

**`demos/build.sh`, at the command line:** the plan laid under a storey and given its
scale; the walls traced with `--on`; `floor --walls`; two `room`s; `roof` by the
outline's corners turned from the page; the figures above; what is refused (a floor
where no walls enclose anything; a second floor on the same outline; a room's point
outside every wall; a roof whose footprint crosses itself) and the file byte for byte
the same after each.

**`demos/buildw.sh`, in the window:** from an empty job, the plan laid, its scale set
from its 30 ft dimension, the five walls traced with the pointer snapping to the plan's
own lines; then the Floor tool and one click inside the walls; the Room tool, a click
in each room and its name typed; the Roof tool, the four corners picked and "6:12, 8 ft"
typed. Pictures of the plan after each, and of the house in 3D with its roof on. The
saved job read at the command line has to give the figures above.

The judge tries a floor on walls that do not close, on two separate houses on one
storey, and on a house with a courtyard; a room on a wall, outside, and two in one
space; a roof with three corners, with a corner picked twice, drawn clockwise and
anticlockwise; and to get any of them Known or unflagged off a traced plan.

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

**Where the fixes stand (2026-11-11).** Each of J12-1 to J12-6, J12-10 to J12-14, J12-17
and J12-18 is fixed in the core with a test named for it (`j12_...` in
`crates/world/tests/world.rs`, six tests, figures worked by hand); J12-20 at the command
line with lines in `demos/draw.sh`; J12-15 in PORTMAP.md; J12-19 by correcting the
document (DECISIONS.md call 41; no code changed). J12-7, 8, 9, 16 and 21 are changed in
the window's code. After them `demos/draww.sh` and `demos/clashw.sh` were both run again
and both exit 0, and their pictures are of this build: every picture that differed from
the set read before was read again (the left panel's two new rows account for the model
pictures; the sheet gains a Clashes tab once the drawn house has a pipe ending in a
lavatory). **Of the five window changes only J12-21 is shown in the window**: picture
`draw/14` is a conduit refused, where its settings were typed, for a system no conduit
goes on. The square held by Shift (J12-7), the snap scene that leaves out what is being
moved (J12-8), a chain let go on a change of storey (J12-9) and the click that ends a
move (J12-16) are in the code and no demonstration exercises them. What was decided is
call 41. **None of these fixes has been judged.**

## Thirteenth judgment (2026-11-11): fittings, connections and clashes, and the fixes to the twelfth

An independent judge (Opus; it did not build any of this) was given commit `b566079`, a
frozen copy of the source and the built command line, and the brief to break fittings,
connections and clash detection and the twelfth judgment's fixes. Eighteen findings, no
panic. It did not build or drive the window. Its reproductions are kept outside the
repository (scratch files). As it reported them; what was done about each goes in the
table that follows this one once the fix is in.

| # | Finding (the judge's words, shortened) | Severity it gave |
|---|---|---|
| J13-1 | Two runs of one kind that touch at one end are never tested against each other, so whole-length overlaps are missed: a duplicate run, a run folded back over another, a pipe of one system lying in another's, a pipe through a tank once it is joined to it. The sample house has one (a hot branch drawn 1 ft back up inside the hot main) that the clash report does not list. A tee is made where two ends leave a joint the same way. | missed clash |
| J13-2 | Stacked openings (a window over a window, a transom over a door) fill each other with solid wall: a pipe through either window is reported through the wall, as a Known finding. The same where a door and a window lie over each other. | false clash |
| J13-3 | A clash between things on two storeys ignores an Assumed storey elevation, and a wall's Assumed base offset is ignored: field questions are reported as findings. Fittings and connectors leave out the storey elevation's status too. | wrong certainty unflagged |
| J13-4 | A door or window that could not be cut (taller than its wall, place unknown) leaves solid wall, and a pipe through it is a Known finding. | false clash |
| J13-5 | "In each other by" is short where a solid is in pieces: a box on a floor's diagonal seam reads 3.39 in where it is 10.8 in; a box across a wall's piece joint reads 0.20 ft where it is 0.35 ft. | wrong figure unflagged |
| J13-6 | A fitting's box is never turned: on a run at 45 degrees its box sticks out and clashes with a pipe that is clear of the duct. | false clash (flagged Assumed) |
| J13-7 | Cutting a run counts a typed number twice when it is written as a sum (`2 * 2`, `-(-4)`): material $40.00 becomes $80.00. Only a bare number is refused. | wrong figure unflagged |
| J13-8 | The J12-1 fix (a wall over a wall) misses a wall a quarter inch off another, shallow crossing walls, and one body drawn on two location lines (20 LF where 10 exist), all Known and never reported; and it refuses back-to-back walls on a shared face line and a wall standing on a knee wall. "Never tested" does not say walls are not tested against walls. | wrong figure unflagged; refuses what is right |
| J13-9 | The J12-5 fix measures an opening's point from the wall's drawn line, not its body: a point inside the body is refused and a point out in the room is accepted. | loads what it should refuse |
| J13-10 | The J12-18 fix refuses only the spelling "hr": `2 lb/hour`, `2 lb/h`, `2 lb/HR` are accepted. | loads what it should refuse |
| J13-11 | The J12-13 names rule makes different names one name: "Pipe 1 1/2 in" and "Pipe 11/2 in"; "Copper 1 6 oz" and "Copper 16 oz". "Grade No. 1" and "Grade No 1" are two names while "No 1." and "No 1" are one. Remove matches a condition exactly but a material by the rule. | minor |
| J13-12 | The J12-10 fix cuts a person's own source short: "A-101; cut (section 3) shows this wall at grid C, rev B", moved, becomes "A-101; moved (by eye)". | minor (provenance lost) |
| J13-13 | A supply-air duct can be joined to a fixture on a water system, with no warning. | loads what it should refuse |
| J13-14 | A pipe and a duct ending at one point leave no way to make either fitting: the refusal asks for a height and comes back when one is given. | minor |
| J13-15 | A tee can never be put on an upright run, and `fitting --all` passes over some joints without listing them. | minor |
| J13-16 | The place reported for a clash can be outside one of the two things (2 ft off a pipe that a duct crosses at an angle). | minor |
| J13-17 | Depths between round runs are slightly short: 3.483 in for 3.50, 0.995 in for 1.00. | minor |
| J13-18 | The second piece of a cut run loses the run's name ("pipe 294"), and the clash report then names it so. | minor |

**What held under attack** (the judge's list): a thin pipe across a duct at 0, 1, 15, 45,
89 and 90 degrees (4.06 in each), climbing (7.16 in) and upright (6.5 in), all worked by
hand; a run ending at a wall's face, 0.005 in and 0.011 in in it, and short of it; a pipe
at a window's edge and under its sill; a box in a box, corner to corner, and turned 45
degrees (flagged); equipment sunk in an L-shaped floor away from its seams; clearances
at 2.0004 in against 2 in and either side, and every bad clearance refused. On the sample
house every wall, floor, run and placed thing is tested or listed as not tested (185 and
5), and the five clashes reported are right by hand (1.80, 3.50, 6.38, 1.20 and 1.27 in).
Four thousand parallel pipes in 3.7 s; three thousand crossing in a grid gave exactly the
2,223,000 that cross. Fittings: bend, coupling and same-way at 0.5, 0.99, 1.0, 1.01, 179
and 180 degrees; a cut's lengths adding to the last digit and its height on a climbing
run; formula lines, allowances and priced totals cut without loss; a typed-number line
refused by every route with the file unchanged; `--all` twice adding nothing. Of the
twelfth judgment's fixes: J12-2, 3, 4, 6, 14, 17 and 20 held.

**Not reached by the judge:** things on two storeys joined; `--kind` given wrong on a
transition; save, load, save after every verb; rooms; J12-12 and J12-19; **the window,
so J12-7, 8, 9, 16 and 21 are still not judged**; runs not justified on their centre;
clearance between a run and a wall it passes through.

**Where this leaves the twelfth judgment's fixes.** Judged and held: J12-2, 3, 4, 6, 14,
17, 20. Judged and found wanting: J12-1 (J13-8), J12-5 (J13-9), J12-10 (J13-12), J12-13
(J13-11), J12-18 (J13-10). Not judged: J12-7, 8, 9, 11, 12, 15, 16, 19, 21.

**Where the fixes to the thirteenth stand (2026-11-11).** What was decided is DECISIONS.md
call 44. **None of these fixes has been judged.**

| # | What was done | Held by |
|---|---|---|
| J13-1 | A joint excuses the joint only: each run is left out a little way either side of it and the rest is tested. Two ends leaving a joint the same way make no fitting. | `j13_1_...`; `demos/clash.sh` (the house's hot branch inside its hot main, 5/8 in) |
| J13-2 | A wall's solid pieces are its face less every hole (`geom::face_less_holes`); the window draws from the same pieces. | `j13_2_4_...` |
| J13-3 | A shape is no surer than its storey's floor height; a wall no surer than its base offset; a fitting and a connector no surer than the floor height. | `j13_3_...` |
| J13-4 | A wall with an opening that could not be cut is Assumed at best. | `j13_2_4_...` |
| J13-5 | Against a wall or a floor, depth is the least way out that clears every piece. | `j13_5_...` (0.9 ft on the floor's seam, 0.35 ft across a wall's piece joint) |
| J13-6 | A fitting is turned to lie along the first run it joins. | `j13_6_7_...` |
| J13-7 | A quantity that names no measure is a typed number, however written. | `j13_6_7_...` |
| J13-8 | One wall over another is judged by their bodies and by where each stands. Back-to-back walls and a wall on a knee wall are accepted; a wall a quarter inch off another and one crossing it at 1 in 100 are refused. "Never tested" says wall against wall. **Not done:** the `wall` verb still has no base offset. | `j13_8_9_...` |
| J13-9 | An opening's point is measured from the wall's middle. | `j13_8_9_...` |
| J13-10 | The hour is refused however it is spelled. | `j13_10_11_12_13_18_...` |
| J13-11 | Names: capitals, runs of spaces, spaces beside a comma and a full stop that is not a decimal point are ignored; a space between figures is not. Removing a condition matches its name the same way. | the same |
| J13-12 | Only a note this program wrote, at the end of a source, is replaced. | the same |
| J13-13 | A run is not joined to a thing on a system it cannot go on. | the same |
| J13-14 | **Not fixed.** A pipe's and a duct's ends at one place still cannot be given either fitting. The refusal no longer asks for a height that would not help; it says the two cannot be told apart and to put each fitting on before the other run is drawn there. | |
| J13-15 | **Not fixed.** No tee on an upright run; `fitting --all` still passes over some joints without listing them. | |
| J13-16 | The place reported is inside both things: the middle of the boxes' overlap if it is, otherwise a place the measure of depth found. | not held by a test of its own |
| J13-17 | Two round runs are measured by plain geometry where it is exact. | `j13_1_...` (1 in pipes in each other by 0.0833 ft) |
| J13-18 | The second piece of a cut run keeps the run's name: "<name>, cut at <number>". | `j13_10_11_12_13_18_...` |

**What these changed that was so before** (each is a change to an existing behaviour or
number):

* The sample house reports 21 things in the same space where it reported 18. The three
  more are true and were hidden by "joined": a hot branch inside the hot main (5/8 in)
  and two pipes lying half in the water heater's face (1/2 in and 3/8 in).
* A job whose storey height is Assumed (every job drawn in the window) now has no clash
  that is a finding: each is to verify in the field until the height is Known.
* A wall on a storey whose height is Assumed, or with an opening that could not be cut,
  is drawn and reported as Assumed.
* A fitting put on a run that is not square to the plan is turned (its `angle` is
  saved); before, none had an angle.
* The refusal of an opening's point reads "... ft off the middle of <wall>, which is ...
  ft thick" where it read "... ft off <wall>'s line", and is measured from the middle.
* Names that differ only by spaces inside them ("wall 6 in", "wall6in") are two names
  now; they were one under call 41.
* A second piece cut off a named run is named after it, not numbered.
* `cargo test --workspace`: 164 tests (151 on the model, 8 on the skeleton, 2 on plan
  files, 3 on the command line), all passing; it was 154.

## Fourteenth judgment (2026-11-11): plans under the tracing, and the fixes to the thirteenth

An independent judge (Opus; it did not build any of this) was given commit `719ff91`, a
frozen copy of the source, the built command line and the built window, and the brief to
break plans under the tracing, the thirteenth judgment's fixes, and the window fixes of
the twelfth that nobody had judged. Fourteen findings, no panic. It used three window
runs of the eight it was allowed. Its reproductions are kept outside the repository
(scratch files). As it reported them; what was done about each goes in the table that
follows this one once the fix is in.

| # | Finding (the judge's words, shortened) | Severity it gave |
|---|---|---|
| J14-1 | **The commit cannot be built from a clean clone.** `.gitignore`'s line `plans/` ignores every folder of that name: `crates/plans/` and `demos/plans/` are in no commit. `cargo metadata` on an archive of `719ff91` fails; `demos/plan.sh` and `planw.sh` need files that are not committed. STATUS claims work that is not in the commit. | release-candidate item 1 fails; false claim |
| J14-2 | Setting a drawing's scale again after tracing leaves what was traced at the old scale, and nobody is told: two equal lines on one page come out 30 LF and 60 LF, both "traced from" the same drawing. In the window a wall stays 60.00 LF and runs off the sheet beside "Scale dimension 30 ft". | wrong figure, unflagged as stale |
| J14-3 | Setting the scale after lining a drawing up breaks the line-up, silently: the corner stays where the old scale put it, still Known. | wrong figure, false Known |
| J14-4 | A picture more than 16,384 dots across kills the window with nothing said, and the job cannot be opened in it again. PDF pages are capped at 4,096 dots; pictures are not. | loads what it should refuse |
| J14-5 | Walls are double-counted as Known: (a) two walls of a type with no thickness, 1 in apart for 20 ft, are both accepted; (b) a ground wall held up to the roof storey and an upper-storey wall on the same line are both counted (480 SF where 320 exist): walls are only compared on one storey. | wrong figure unflagged |
| J14-6 | The J13-8 fix refuses legitimate layouts: walls crossing at 25 and 20 degrees, a T and a V at 10 degrees. Every crossing under about 29 degrees is refused. | refuses what is right |
| J14-7 | The J13-1 fix reports a 30 degree lateral (two 4 in pipes) as a Known hard clash, 0.056 ft, with or without its fitting. | false finding |
| J14-8 | Hand-made PDFs lose things without saying so: walls drawn as filled bars give no lines and an empty "not read" list; 200 hatch ticks under 1 pt are dropped unsaid; a clipped line is reported at its unclipped length; a white line is a snap line with no ink under it; dashed lines become solid and the dashes are not mentioned; a page with no size is given A4's; a truncated file is refused as "0 pages". | loss not reported |
| J14-9 | A drawing under one storey is accepted for tracing on another (`--on`). In the window the source is taken from the last drawing on the storey, not the one under the pick (read in the code, not seen). | loss of provenance |
| J14-10 | A drawing whose page size is edited by hand loads, `check` passes, and `drawings` does not say the file's page differs. | loads what it should refuse |
| J14-11 | The J13-10 fix is not closed: "2 lb/MH", "lb/man-hour", "lb/manhour", "lb/hr.", lb/day, lb/min, lb/shift are accepted as a material's order. | loads what it should refuse |
| J14-12 | The J13-12 fix still cuts a person's own note when it ends in the same words: "A-101 rev B; moved (per RFI 12, architect)" becomes "A-101 rev B; moved (tape)". | minor |
| J14-13 | The J13-11 names rule keeps plainly one name as two ("Copper, 16 oz" and "Copper 16 oz"; "Pipe 1-1/2 in" and "Pipe 1 1/2 in"; "Cu 16oz" and "Cu 16 oz"; "No. 1" and "No.1"; "Type X" and "Type-X") and makes weak merges ("a.b" and "ab"; "Pipe 1,2" and "Pipe 1, 2"). | minor |
| J14-14 | Minor: a scale from points 0.001 apart is accepted ("0 to the foot"); scale points far off the page are accepted; a height and a type typed with `--on`, and a move in the window, are sourced "traced from"; deleting a drawing says nothing of what was traced from it; software terms on screen ("(os error 2)", "drawing-0003", "(type-0004)"); a JPEG turned by its EXIF tag is shown unturned, unsaid; one file can be laid twice with no notice. | minor |

**What held under attack** (the judge's list): no traced value came out Known by any
route; the arithmetic of a scale on a slant, a line-up to a negative place, and a door by
a point on a traced wall; a moved, changed or missing plan file each said, and the rest of
the job working without it; paths kept from the job's folder; save, load, save the same
bytes; pictures of every sort read by what is in them; rotated pages, a crop box, an
offset page, flipped and scaled coordinates, forms, a layer switched off, two pages, text
and a curve in hand-made PDFs. In the window: a wrong path, a text file and a page that
is not there refused; every tool and the line-up refused before a scale; Escape at each
point of the scale tool; undo and redo; a reopened job showing its drawing to scale; the
wheel keeping the place under the pointer. Of the twelfth judgment's window fixes: J12-7,
J12-9, J12-16 and J12-21 held. Of the thirteenth's: stacked openings (J13-2), status
(J13-3), a duct to a water fixture (J13-13), the fitting's turn (J13-6), and the sample
house's three new clashes, each right by hand.

**Not reached by the judge:** J12-8; the depth figures of J13-2, 4 and 5 by hand; J13-9;
J13-14 to 18; encrypted PDFs; a transparent PNG and two drawings on one storey in the
window; lining up in the window; the USDA scan traced in the window.

**Where this leaves earlier fixes.** Judged and held: J12-7, 9, 16, 21; J13-2, 3, 6, 13.
Judged and found wanting: J13-1 (J14-7), J13-8 (J14-5, J14-6), J13-10 (J14-11), J13-11
(J14-13), J13-12 (J14-12). Not judged: J12-8, 11, 12, 15, 19; J13-4, 5, 7, 9, 16, 17, 18.

**Where the fixes to the fourteenth stand (2026-11-11).** What was decided is DECISIONS.md
call 46. **None of these fixes has been judged.** Commits `939e6aa`, `719ff91` and
`f449a57` do not build from a clean clone, and what STATUS said of them was true only of
the working folder.

| # | What was done | Held by |
|---|---|---|
| J14-1 | The ignore line is anchored to the one folder it was meant for; the plans crate and the made-up plan files are committed. `tools/committed.sh` lays out the last commit as git holds it, in a build folder of its own, and builds and tests it; it is run before a push. | the check itself: it fails on each of the three commits named above (`COMMIT=719ff91 tools/committed.sh --quick`) |
| J14-2 | A scale set again moves nothing; what was traced at the earlier scale is listed when it is set, kept on the drawing, listed by `drawings` from then on, and said in the window in the tone of a refusal. | `j14_2_3_...` |
| J14-3 | A drawing remembers what it was lined up by; a scale set again leaves that place where it was put. | `j14_2_3_...` (the corner at -7, the lined-up place still at 0, 0) |
| J14-4 | A picture is shown at no more than 4,096 dots on its long side and keeps its own size; that it is shown smaller is said. One over 250 million dots is refused. | `a_picture_too_big_...` in the plans crate; the judge's 20,000 by 12,000 picture reads and says so. **Not shown in the window.** |
| J14-5 | A wall of unknown thickness counts as 3 in thick for the test; walls on different storeys are compared by where each stands. | `j14_5_6_7_...` |
| J14-6 | Walls at an angle (more than about three degrees) are never one over another. | `j14_5_6_7_...` (25, 20 and 10 degrees accepted; one degree refused) |
| J14-7 | A joint between two runs is excused for the sum of their half-widths over the sine of the angle. | `j14_5_6_7_...` (a 30 degree lateral clear; a 5 degree branch a clash) |
| J14-8 | Filled shapes' edges are read as lines; lines under a point long, lines in white, dashed lines and lines under a mask are counted and said, and a line is cut to what a square mask shows; a page with no size of its own is refused; a file with no pages says it may be cut short; a PDF that does not end as one says so. | `what_a_drawn_page_holds_...` in the plans crate; the judge's files `fills`, `short`, `clip`, `white`, `dashed`, `zero`, `truncated` each read as it now should |
| J14-9 | `--on` a drawing of another storey is refused. In the window what is traced takes its source from the drawing the pick is on, and a pick off every sheet is drawn in the window. | **The command line: not held by a test or a demonstration line yet. The window: not shown.** |
| J14-10 | `drawings` says when the file's page is not the size the drawing was made at; the window does not show such a sheet and says why. | **Not held by a test.** |
| J14-11 | No measure of time is what a material is ordered by. | `j13_10_11_12_13_18_...`, with the judge's spellings added |
| J14-12 | A source loses nothing: a note is added and never twice running. | `j12_10_...` and `j13_10_...`, both changed after call 46 |
| J14-13 | Names: a comma, a hyphen, an underscore and a full stop part words; a figure and the letters beside it are two words. | `j13_10_...`, with the judge's pairs added |
| J14-14 | A scale's points are on the page and at least a unit apart. A move, a fitting and a join in the window are sourced as done in the window. Removing a drawing says how many things were traced over it. Laying the same page twice is said. A JPEG is shown the way its own note says. **Not done:** a height and a type typed with `--on` still say "traced from"; the words on screen go to the language pass. | `j14_2_3_...` for the scale's points; the rest not held by tests |

**The window pictures are stale.** Every picture under `demos/captured` was taken on
commit `719ff91` or before. Commit `67b3e4e` changed the window's code (where a traced
thing's source comes from, what a second scale says, what removing a drawing says, a
picture's size); `demos/planw.sh`, `clashw.sh` and `draww.sh` have not been run on it.
They are to be run again, and every picture read again, after the language pass, which
changes the words on every screen.

**What these changed that was so before:**

* A scale set again on a lined-up drawing moves the sheet (its corner), where before the
  corner stayed and the lined-up place moved.
* Walls crossing or meeting at any angle over about three degrees are accepted; under
  call 44 those under about 29 degrees were refused.
* A second wall on a line where a wall of another storey already stands at the same
  height is refused; it was accepted.
* A wall's or a run's source after a move by a new hand reads "...; moved (first);
  moved (second)" where it read "...; moved (second)".
* "Copper, 16 oz" and "Copper 16 oz" are one name; "a.b" and "ab" are two.
* A PDF whose walls are filled bars gives their edges as lines to snap to; it gave none.
* The made-up vector plan still reads as 8 lines; the USDA scan as none.
* `cargo test --workspace`: 178 passing (153 on the model, 8 on the skeleton, 4 on plan
  files, 4 on tables, 3 on the spreadsheet reader, 3 on the STEP reader, 3 on the
  command line).

## Fifteenth judgment, part A (2026-11-11): tables and a model brought in, and the fixes to the fourteenth

An independent judge (Opus), on a frozen copy of commit `9629e27`, told to break it. A
second judge, on the same copy, is listing every word on every screen (part B; not back
yet). Twenty-five findings, worst first, as the judge gave them. None is fixed at the
time of writing; what is done about each goes in the table that follows this one.

| # | What the judge did | What is wrong |
|---|---|---|
| J15-1 | A CSV parted by semicolons with decimal commas (`9,05`) | **Read 100 times too high: $905.00/lb.** Every comma is dropped as a thousands mark. A prior estimate is read the same way. Wrong figure, unflagged. |
| J15-2 | `import prices --status Assumed --source ...` on a list with its own "How sure" and "Where from" columns | **The person's word is ignored without a word:** a row saying Known goes in Known. |
| J15-3 | A model whose building is placed at a height | **The walls stand that far above their storey** ("base offset 164.042 ft"): the storey is taken at its stated elevation, the wall at its full placement. Wrong figure, unflagged. |
| J15-4 | A drawing lined up again after things were traced over it | **The sheet moves out from under what was traced and nothing is said** (J14-2 by another route). Tracing the same printed line again makes a second wall. |
| J15-5 | A prior estimate with a "Total" row; an amount that is not quantity times unit cost | **The total row is taken as a line and the prior's total doubled.** A wrong extension is used without a word. |
| J15-6 | `compare`, one description twice in the prior | **The job's whole quantity is set against each of the two lines.** |
| J15-7 | `compare`, both sides Assumed | **The difference carries no flag.** |
| J15-8 | A workbook with a date, and a percent, in the price column | **The date becomes a price, and 9.05% becomes $0.09.** The note gives no row and the row is used. |
| J15-9 | A workbook whose first sheet is hidden | **The hidden sheet is read and not said to be hidden.** |
| J15-10 | Blank rows in a CSV; a sheet that does not start at A1 | **The row numbers in "not used" are wrong.** |
| J15-11 | A wall in a model made of two solids | **The first is kept and the second dropped; "not brought in: 0".** |
| J15-12 | A product with no placement | **In neither count:** what is not brought in does not add up to the file's products. |
| J15-13 | A supplier's row with its own "ordered at" | **Dropped, and the row counted as the same as the job has.** |
| J15-14 | An assembly library with two assemblies of one name | **Merged; the second's "goes on" and cost code lost unsaid.** |
| J15-15 | Walls left at an earlier scale | **Said only by `drawings`**; the takeoff and `show` say nothing. A scale set back to what a wall was traced at still calls the wall "traced at an earlier scale". |
| J15-16 | An L-shaped space in a model | **The room's point is outside the room** (the average of its corners). |
| J15-17 | `tools/committed.sh --quick` on a commit without the made-up model | **Passes.** The check looks only at `demos/plans` and `demos/tables`. The root `.gitignore` still has unanchored names (`fixtures/`, `work/`, `out/`, `evidence/`): the trap of J14-1, waiting. |
| J15-18 | A PDF line under a white filled box | **Kept to snap to**, and nothing says it is covered. |
| J15-19 | Two drawings overlapping on one storey in the window | **One is hidden, which one is chance, and a pick goes to the last laid** (the sourcing read from the code, not driven). |
| J15-20 | A list with a part-number column and a description column | **The part number is taken as the material's name.** |
| J15-21 | A model with two storeys of one name; the same model brought in twice | **The whole model refused, in the program's own words** ("already has a child named"). |
| J15-22 | A model with one number on two things | **Taken silently; the later wins.** |
| J15-23 | Order units "lb/workhour", "lb/crew-day", "lb/LH", "lb/manweek" | **Accepted** (J14-11 still open). The lines price to no figure, so no dollar is wrong. |
| J15-24 | A PDF line wholly off the page | **Kept to snap to**, unsaid. |
| J15-25 | Minor, together | A name beginning `=`, `+`, `-` or `@` goes out as a spreadsheet would run it; a name's leading and trailing spaces are lost; blank rows and conflicting duplicate prices pass unsaid; UTF-16 and empty files are refused for the wrong reason; "LBS" and "lb." are refused; a slab with a hole in a model is "may slope, or be a mesh" and is brought in whole; a missing reference is "curved"; a model's walls lose their names; a wall leaning 2 degrees is upright; "IFCBEAM", "system-0004", "type-0003" on screen. |

**What held, in the judge's words:** a price list back into the job it came from, byte
for byte, and into an empty job whole but for the spaces; BOM, CRLF, tabs and Latin-1
read and said; 200,001 rows cut and said; a price that is a word, a wrong unit and a
missing unit column each refused by row; the assembly library's refusals; a prior
estimate is never added to the job's cost; a model's walls in mm, m, ft, in and no unit,
straight and turned, against hand figures; a storey below zero, a slanted pipe, a duct
on its side; cut-short and broken files refused; the made-up house's clash at
(10, 10, 7), 3.50 in; of the fourteenth's fixes, J14-2's own message, J14-3, J14-4 at the
command line, J14-9, J14-10, the EXIF turn, a clip under a scale; J13-14 and J13-15's
`fitting --all`.

**What the judge did not reach:** the whole-workspace mode of `committed.sh`; merged
cells, error cells and formulas with no saved result; .ods, .xlsm and .xls; curved,
swept, mapped and mirrored walls in a model, placements three deep, a void with no door,
elbows; in the window, the stale-scale notice, lining up after tracing, and the
sourcing on overlapping drawings.

The judge's reproductions are kept outside the repository (the scratch folder
`judge15/findings/a`), each made up.

## Fifteenth judgment, part B (2026-11-11): every word on the two windows

The second judge (Opus), same frozen copy (`9629e27`), told to list every word on
every screen and find the ones a builder would not say. Nine window runs. **The
language pass does not hold.** Thirteen findings, worst first, as the judge gave them;
none is fixed at the time of writing. Its word lists (every literal with its file and
line) are in the scratch folder `judge15/findings/words`; they are not grouped by where
on screen each appears, which the brief asked for and release item 9 will need.

| # | What is wrong | Where the judge saw it |
|---|---|---|
| J15-B1 | **Paths are still on screen; `theme::spoken` has at least five holes.** A thing just removed ("removed duct-0015"; "gone: plane-0073, eave-0077, ..."); a path with 's after it ("roof-0072's footprint"); the message card capitalises before `spoken` runs ("Plane-0073: ...", "Drawing-0004: ..."); the job is named by its own path ("job-0001: overhead set to 12", a row "job-0001 job-0001"); and a thing drawn with no name is called by the file's counter ("duct 13", "fixture 18" for a lavatory, walls numbered from 9). | Delete, G on a roof, the card, the Money tab, every panel |
| J15-B2 | **The window tells a user to type command lines** ("say which with `plumbline labor <job> ...`", "edit it in the file", "join them by name (plumbline connect)"). | L on a plane with two labor lines; three more in the source |
| J15-B3 | **A backslash before every inch mark** in a line's description, from the program's own quoting (`"Gutter 6\" half round"`). | The entry's title, the Money tab, the Estimate |
| J15-B4 | **The machine's error words and full folder paths:** "the PDF could not be read (Invalid)", "unexpected end of file", "No such file or directory", a whole scratch path. None says what to do next. | Laying a broken, cut-short or missing file |
| J15-B5 | **Words a builder would not use:** element(s), network(s), solved and solver, class, values, "under this wall", pan and zoom, "not selected" for a product not chosen, "schedule code", "QUESTIONS.md Q1" as a source; kinds written as the code writes them ("a RoofPlane"), "connector", "mask". | Money tab, system panel, roof messages, sheet, Keys, To settle |
| J15-B6 | **Units not the trade's way, and mixed:** "28.00 $/SF" beside "$9.00/lb" beside "$15.68 / SF"; hours and a rate to four decimals; "lb" where the rule says LB; places as bare decimal pairs; open ends and a page's size with no unit; clash depths in decimal inches where fractions are used elsewhere; one height three ways on one panel; "1/2 in round" for a pipe. | Panels, sheet, system panel |
| J15-B7 | **Two words for one thing:** elbow and bend; storey, level and floor; facets and planes; "drawing" for the sheet and for the act; join, connect and connector; "selected" two ways. | Throughout |
| J15-B8 | **The same text written twice, and a false claim in this file:** "The language pass" says paths are gone from beside each row of the sheet; the code still draws the path and `spoken` turns it into the name: "plane 1 plane 1", "south south", "ROOF: ROOF". | Every tab of the sheet |
| J15-B9 | **Wrong reason, or no next step:** a fixture's size "2 by 2" refused as "not a plain number"; "the job has no condition that goes on a roof plane" with no way in the window to make one; "undone: remove" does not say what came back. | Tool settings, C, Undo |
| J15-B10 | **A plan named the way the hint shows is refused:** a name typed relative to the job's folder is read from the folder the program was started in. | Lay a drawing |
| J15-B11 | **A false source:** what is typed on the plan is sourced "set in the 3D view; not yet confirmed against a drawing". | Anything drawn on the plan |
| J15-B12 | Minor: British spellings in a US-unit program (storey, centre); "Architecture" for "Architectural"; Keys offers C, U, N, L for a drawing and "Next roof plane" on a job with no roof; "job.json" as the job's subtitle. | Keys, top bar |
| J15-B13 | **Call 48's and the test's claims are not so:** `theme::text` is not the only way text reaches the screen (the pointer's position, the entry field, the two windows' titles); the test asserts a path left on screen; removing "(os error 2)" leaves the rest. | `theme.rs`, `panels.rs`, `main.rs` |

**What held, in the judge's words:** the refusals for a length with no unit, a pitch in
degrees, a price in the wrong unit, a word for a number, a file that is not a plan, a
system that is not there, a door off a wall, a drawing with no scale, and what a door
takes: each plain, each saying what to do. The names of what the pointer catches on,
the tab names, the clash and not-tested wording. Pitch as 6:12; SF, LF, SQ, EA in the
tables; "On the tape". No panic in any run.

**What the judge did not reach:** a condition's panel; the move refusal; the panels of
a fitting, a room, a picture drawing and a storey; the metric refusal; panels compared
between plan and 3D.

**The claim this file made that was false** (J15-B8) is in "The language pass. What was
done": that a path is never shown beside a row of the sheet. It is shown, as the name
a second time. That section stands as written, with this correction.


## Where the fixes to the fifteenth judgment, part A, stand

Fixed in the commit that adds this section, each held by a test; **none of these fixes
has been judged.** DECISIONS.md call 50 says what each decided.

| # | What was done | Held by |
|---|---|---|
| J15-1 | A comma in a figure is read only where it parts thousands; `9,05` and `1.204,50` are refused by row. One rule, `units::commas_part_thousands`, for a typed price and a list. A semicolon list says it may hold decimal commas. | `j15_1_...` (tables) |
| J15-2 | The person's `--status` and `--source` stand over the list's own columns, and a note says the columns were not read. | `j15_2_13_20_...` |
| J15-3 | What stands on a storey is measured from where the storey is placed; the floor's height is what the storey says; a note when they differ. | `j15_3_...` (ifc): 50 m up, walls on their floor, same figures |
| J15-4 | A second line-up after tracing records what was traced, gives it back, and it is said by `lineup`, `drawings`, `show`, `takeoff` and the window. | `j15_4_15_...` (world) |
| J15-5 | Total rows of a prior estimate are not lines; a wrong extension is kept as written and noted. | `j15_5_6_7_...` |
| J15-6 | Prior lines of one name are added together before being set against the job. | the same |
| J15-7 | A difference carries the flag of the less sure of its two sides (command line). | **By reading only**: no test drives the printed comparison. |
| J15-8 | A percent cell is read as shown (`9.05%`), a date as words; neither is a figure, so the row is refused. | `j15_8_9_...` (sheets), on `demos/tables/hidden-and-shown.xlsx` |
| J15-9 | The first sheet on show is read; a hidden one only when asked for, and said to be hidden. | the same |
| J15-10 | Each row carries its number in its file: blank lines, cells with line breaks, a table below a sheet's first row. The CSV library's own line number was wrong past a blank line and is not used. | `j15_10_...` (sheets and tables) |
| J15-11 | A body of more than one piece is not read, and is listed. | `j15_11_12_22_...` (ifc) |
| J15-12 | A thing the model names and gives no place is listed. | the same |
| J15-13 | A supplier's row's "ordered at" is noted as not read. | `j15_2_13_20_...` |
| J15-14 | Rows of one assembly name that disagree are refused, all of them. | `j15_14_...` |
| J15-15 | The stale list is on the drawing's own values (flagged) and in `takeoff`; a drawing back at the scale and place it had stops listing. | `j15_4_15_...` |
| J15-16 | A room's point is inside its outline. | `j15_16_...` (ifc unit test): a square, an L, a U |
| J15-17 | `committed.sh` checks every file named under `demos/` and `tools/`, and fails on anything git ignores inside the program's folders. Proven red both ways (a `fixtures/` folder under a crate; a scratch copy committed without the made-up model). The repository root's `.gitignore` is the earlier program's and is left as it is. | run by hand; its output is in this session |
| J15-18 | A line wholly under a white square filled over it is not kept; a white shape that is not a square is said. | `j15_18_24_...` (plans), against the picture's own pixels |
| J15-19 | **Not fixed.** Two drawings overlapping on one storey in the window. | |
| J15-20 | Of "Item" and "Description", the description names the thing; "Item" is listed as not read. | `j15_2_13_20_...` |
| J15-21 | A model's storey with the name and height of one the job has is that storey; another height is refused with both heights; two of one name in a model are told apart by height. The same model brought in twice makes nothing twice and is refused, each thing with why. | `j15_21_...` (ifc) |
| J15-22 | Numbers used twice in a model's file are counted and said. | `j15_11_12_22_...` |
| J15-23 | A rule for what is a measure of time, in place of a list of spellings. | `j15_23_...`, with the judge's spellings and twenty-two units that are not time |
| J15-24 | A line wholly off the page is not kept; one that runs off is cut at the edge; both said. | `j15_18_24_...` |
| J15-25 | Done: the apostrophe guard on a cell a spreadsheet would run; names trimmed where a thing is added; a duplicate price at another figure not used; UTF-16 and empty files refused for what they are; `lbs` and `lb.`; a floor with a hole in its outline listed, one with an opening cut in it flagged Assumed and said; the reason for an unreadable body; the model's own names; upright to an eighth in eight feet; a model's class names in words. **Not done:** blank rows are still passed over without a word (they are not rows); the paths in "this program would not take it: wall-0022 lies along wall-0005" (part B). | `j15_25_...` (sheets, ifc); the rest by reading |

**What these changed that was so before:**

* `plumbline import prices` with `--status` on a list that has its own columns now uses
  the person's word; it used the list's.
* A figure such as `9,05` in a list is refused; it was read as 905.
* A prior estimate's "Total" row is not a line, and its total is the lines' own.
* `compare` shows two prior lines of one name as one, with how many were added.
* A workbook's hidden first sheet is no longer what is read by default.
* A model's class names are words ("beam", not "IFCBEAM") in what is listed as left out.
* A wall leaning more than about a twelfth of a degree is not brought in from a model;
  up to two and a half degrees was.
* Walls, floors and runs from a model are called what the model calls them, with the
  number in the file only where two would share a name; floors and runs always had the
  number, and walls were "wall N".
* A name given to anything loses the spaces before and after it.
* "there is already something called ... under ...: two things in one place cannot go
  by one name" replaces "... already has a child named ...".
* `plans::line_up` gives back what it left behind; the drawing's `earlier` entries
  carry the corner the sheet had (jobs saved before load as they were).
* A PDF's lines off the page, and under a white square, are no longer kept to snap to.
* `cargo test --workspace`: 208 passing, 16 of them added here (6 on tables, 3 on the
  table reader, 5 on the model reader, 1 on plan files, 1 on the model). The twelve
  command-line demonstrations pass; `demos/imports.sh` and `demos/model.sh` each had
  their expected words changed as call 50 says (a flagged difference; "beam" for
  "IFCBEAM"). The window demonstrations have not been run on this.

## Sixteenth judgment (2026-11-12): field entry

An independent judge (Opus), on a frozen copy of commit `b9bd935`, told to break it.
Command-line probes and four window runs. Ten findings, worst first, as the judge gave
them. **None is fixed at the time of writing.**

| # | What the judge did | What is wrong |
|---|---|---|
| J16-1 | `same` on the ground's floor; the upper floor measured at 8' 10 1/2"; `same` on the ground's storey height (9 ft) | **Three Known values that disagree by 1 1/2 in, all accepted.** The check against the floor above runs only when a storey height is given as a length. `same` on a storey height, and a length on an elevation, check nothing. A floor measured an inch off leaves its storey height as it was, unsaid. |
| J16-2 | `measure` the eave height of a roof plane solved from a footprint | **Accepted, reported, and thrown away the next time the job is opened** (the roof is solved again from its footprint); `check` then fails. The window offers Measure on that row. SPEC 5.14 says an outline's eave height takes a length. |
| J16-3 | A duct between two elbows measured to another height | **Its joints are broken and nobody is told.** The clash list gains five clashes between the duct and its own fittings and neighbours; the takeoff's networks go from 2 to 4. A fitting measured stays "joined" 5 in off its runs. "Height above its storey" is the centre for a run and the underside for a fitting, under one label. |
| J16-4 | A storey height measured below a wall that has its own height | **The wall stands through the floor above, Known, unsaid.** With a wall above on the same line the measurement is refused as "one wall drawn over another", which is not what is wrong and gives no way to record what was measured. |
| J16-5 | A mistyped storey height, then the right one | **The second shot is refused**: the floor above is Known from the first, and the refusal does not say so. The command line has no way back. A reading 1/32 in off a Known floor above is refused: the tolerance is a millionth of a foot. |
| J16-6 | A reading typed with no entry open (`12'4"1/2` Tab `-0.52ft` Tab) | **The job is changed**: layers hidden, the view switched, a roof plane selected and its pitch lowered from 6:12 to 5:12. STATUS said only that "the characters go to whatever the keys do". |
| J16-7 | Two readings sent into one entry | **Both lost, and the message is about a roof plane**: what follows the Tab that ended the entry, in the same frame, went on to the keys. |
| J16-8 | `8'10"1/2, 2nd shot by JM` | **Refused as "something after the inches"**: a note that starts with a figure is taken for part of the reading. The refusal shows a backslash before the inch mark. |
| J16-9 | A storey height that raises the floor under a roof | **The walls now top out above the eave, and nobody is told.** The roof's eave height is its own value. (The clash list does say roofs are not tested.) |
| J16-10 | Minor | Moved floors are rewritten in decimal feet whatever they were entered in; a storey height of 1/2 in is accepted; `same` with `--to` equal to the line held is refused; a run measured Known on an Assumed storey keeps its clash "to verify" with no word why; call 49 misquotes the window's source for `same` and says spaces are free where `8 ' 10 " 1 / 2` is refused. |

**What held, in the judge's words:** every hand figure ($615.86 before, $607.31 after;
2.00 in and 5.00 in, then 2.25 in a finding and 0.75 in clear); status carried through
(a floor no surer than the floor below; floors further up keep theirs; a Known third
floor stops the chain; a crossing refused; the file byte for byte the same after each
refusal); a window head above a new wall top makes the net area Unknown and the sheet
says the line is left out; bottom and top on a run, kept through a split and a move;
every form of length read or refused as documented; in the window, Esc, rows locked
while an entry is open, `bottom` after the reading, undo and redo of a measurement, a
new storey by three Tab-parted parts, the sheet following at once.

**What the judge did not reach:** things brought in from a model; things traced with
`--on`; F2 and reopen of a measured roof plane in the window; Tab in a wall tool's
settings; rooms after a storey moves; a real DISTO; Leica's guides (no network).

## Where the fixes to the sixteenth judgment stand

Fixed in the commit that adds this section; **none of these fixes has been judged.**
DECISIONS.md call 51 says what each decided; SPEC.md 5.14 is brought up to them.

| # | What was done | Held by |
|---|---|---|
| J16-1 | A floor, its storey's height and the floor above are kept agreeing whichever is measured, by a length or by `same`; a value Known in its own right is never overwritten, and a disagreement names the three. | `j16_1_...` (field): the judge's three in both orders, a floor an inch up, `same` on a storey height |
| J16-2 | A plane solved from a footprint refuses a measured eave height and does not offer it. | `j16_2_3_...`, on the made-up house |
| J16-3 | What a run or a placed thing no longer meets after a measurement is named, with what that does to the takeoff and the clash list. Nothing joined to it is moved. In the window a height's entry says which line it is to (a run's centre, bottom or top; a placed thing's underside). | `j16_2_3_...`: the house's trunk, 1 1/4 in up, comes off its elbows and says so; measured as it was, says nothing. The window's title **by reading only**. |
| J16-4 | Walls standing to their own height through a floor that moved are each said, with by how much. A refusal as "one wall drawn over another" now says what is wrong and what to do first. | `j16_4_9_10_...` for the note. The refusal's new words **by reading only**. |
| J16-5 | A floor Known only because the storey height below was measured follows a second shot; one since measured or checked itself does not. Agreement is to an eighth of an inch. | `j16_5_...`: a slip corrected; 1/32 in agrees and is left, 1/4 in is refused; a floor checked on site is its own |
| J16-6 | A burst of characters with no entry open does nothing to the job and says so once. A single key acts an eighth of a second after it is pressed. | **A scratch window run, pictures read, not committed**: the judge's burst on the made-up house leaves the job byte for byte what it was ("Saved", 6:12, every layer on) with the refusal on screen; `t` alone then switches to the plan. No test. |
| J16-7 | An entry ends at the key that ends it; what follows in the same breath is said as a second reading not used. | **A scratch window run**: `8'10"1/2` Tab `9'0"` Tab into the storey height leaves it at 8' 10 1/2". The note about the second reading was not shown in that run (both readings fell in one frame); fixed, and seen in a second scratch run: under what the first reading did, "a second reading came straight after that one and was not used". |
| J16-8 | Who measured follows a comma and a space, whatever it starts with. A backslash before a quote mark is taken off wherever words are drawn. | **A scratch window run**: `0'0"1/2, 2nd shot by JM` is taken, source "measured on site: 2nd shot by JM"; `"1'0"x" has something after the inches` shows no backslash. No test. |
| J16-9 | When floors move, a note says a roof's eave height is its own and did not move. | **By reading only.** |
| J16-10 | A moved floor keeps the unit it was written in; `same` with the line a run is held to is taken; call 49's two misstatements corrected. **Not done:** a storey height of half an inch is still taken; a clash on a storey whose floor is Assumed does not say that is why it is still to verify. | `j16_4_9_10_...` |

**What these changed that was so before:**

* `measure <storey> elevation ...` now changes storey heights (the one below, and this
  one's own) or the floor above, and says so; it changed only the elevation.
* `measure <storey> "storey height" same` now brings the floor above along, as a length
  does; it changed nothing else.
* A storey height that puts a Known floor above within an eighth of an inch of where it
  is is taken and the floor left; anything over a millionth of a foot was refused.
* A second storey height where the first made the floor above Known is taken; it was
  refused.
* A moved floor entered in inches or in feet and inches stays so; it became decimal feet.
* In the window, a character key acts an eighth of a second after it is pressed, and a
  run of them with no entry open does nothing. Keys typed into an entry after the key
  that ends it are not added to it.
* On screen, `\"` is `"`.

`cargo test --workspace`: 212 passing (4 added on the model, for these). The twelve
command-line demonstrations pass. `demos/fieldw.sh` was run again on this code and
passes, the saved job byte for byte the command line's; its ten pictures were retaken
and read again (as two contact sheets), and show what the table under "Field entry.
What was run" says.

**What is not right:** the window fixes rest on scratch runs, not on a committed
demonstration; `demos/fieldw.sh` does not try a stray burst or a second reading. The
other window demonstrations have still not been run since `719ff91`.

## Where the fixes to the fifteenth judgment, part B, stand

Part of it is fixed in the commit that adds this section, and **none of it has been
judged**. DECISIONS.md call 52 says what was decided and what was left, and why.

| # | What was done | Held by |
|---|---|---|
| J15-B1 | A thing removed keeps its name in what is said of it; a path that owns something ("roof-0072's") and a path given a capital take the thing's name; the job is "the job". A wall, an opening and a run drawn with no name are their kind and a number counted from 1 on their storey; a fixture, equipment and a fitting are called what they are ("Lavatory", then "Lavatory 2"); a named run cut in two has a "piece 2"; a clash says a thing once where its name is what it is. | `j15b_...` in `theme.rs`; the model's tests and the command-line demonstrations, whose expected names changed with it |
| J15-B2 | The three messages in the window that named a command to type say what the key does and that the rest is not in this window yet. The core's "join them by name (plumbline connect)" says "they are joined by naming the thing and the run". | By reading. |
| J15-B3 | A backslash before a quote mark is taken out where every word on screen is said. | `j15b_...`; seen in a scratch run (`"1'0"x" has something after the inches`) |
| J15-B4 | A plan or a picture that cannot be read says so in plain words and what to do next; a file that cannot be opened is called by its name, and why is one of three plain reasons. | By reading; **not seen on screen.** |
| J15-B5 | "element(s)" on the Money tab, "networks", "under this wall", "pan" and "zoom", "not selected" of a part, a kind written as the code writes it ("RoofPlane", "a Wall"), "mask", and the made-up job's source that named `QUESTIONS.md`: each changed. **Not done:** "solved" and "solver"; "class"; "values"; "schedule code"; "connector". | `j15b_...` for the kinds; the rest by reading |
| J15-B6 | `28.00 $/SF` is drawn `$28.00/SF`; `lb` is drawn `LB`; hours are shown to two places. **Not done:** "$15.68 / SF" on the Money tab; places as bare decimal pairs; open ends and a page's size with no unit; clash depths in decimal inches; one height three ways on one panel. | `j15b_...` |
| J15-B7 | **Not done.** Elbow and bend; storey, level and floor; facets and planes; "drawing" two ways; join, connect and connector. | |
| J15-B8 | The sheet no longer draws a thing's path under its name, so no name is drawn twice. ("Supply air Supply Air" and "ROOF: ROOF" were not looked at.) | By reading; **not seen on screen.** |
| J15-B9 | A size written "2 by 2 by 3" is read; one that is not a size says so. **Not done:** "the job has no condition that goes on a roof plane" still offers no way to make one; "undone: remove" still does not say what came back. | By reading. |
| J15-B10 | A plan's name is looked for from the job's folder first. | By reading; **not seen on screen.** |
| J15-B11 | What is typed in the window is sourced "set in the window; not yet checked against a drawing". | The four window demonstrations that check that source, once they are run. |
| J15-B12 | Keys offers "Next roof plane" only on a job with a roof, and no condition, price or material for a drawing. **Not done:** spelling (Q20); "Architecture". | Seen in the field pictures once retaken. |
| J15-B13 | The pointer's position passes through `spoken`. The test no longer rests on the claims the judge found false, and holds the holes found. **Not done:** the two windows' titles are still set directly. | `j15b_...` |

**What these changed that was so before** (besides the words): a wall drawn on a new
job is "wall 1", not "wall 7", and so for every thing drawn with no name; a fixture is
called what it is; a cut run's second piece is "<name>, piece 2", not "<name>, cut at
<number>" (jobs saved before keep their names); a source written by
the window reads "set in the window; ..." where it read "set in the 3D view; ..."; a
fitting's and a made-up thing's source reads "not specified" for "not selected"; the
made-up jobs' cost code source reads "which cost code scheme this job is to follow is
not settled"; a plan named in the window is found beside the job.

## Stopped for the owner's review (2026-11-12)

Patrick asked for a full account with screenshots before anything more is built, and
said the screen does not read as a construction tool, that "storey" is English spelling,
and that the wording does not fit good interface practice (his words are in
QUESTIONS.md under Q20). He is right on each. Nothing further is built until he rules.

**Where it stands at this commit:**

* **All eight window demonstrations were run on `f520014` and pass** (`fieldw`,
  `clashw`, `draww`, `planw`, `d4w`, `d5w`, `d2`, `house`), and their pictures under
  `demos/captured` are retaken from it. **Not every picture has been read**: the field
  set was read whole; of the rest, one picture pair each of the house, the drawn plan,
  the traced scan, the estimate sheet and the clashes was read, and sent to Patrick.
  The statement above that the window pictures are stale since `719ff91` no longer
  holds; that they have all been looked at does not hold yet either.
* **`demos/build.sh` and `demos/buildw.sh` are written ahead of what they demonstrate**
  (order step 5) and do not run: the floor verb and the Floor, Room and Roof tools are
  not on the branch. The floor verb and its outline are written and pass three tests in
  a scratch copy outside the repository.
* **Call 54 (the verify-in-field list) and Q21 are thought through and not built.**
* **Not judged:** the fixes to the fifteenth judgment (A and B) and to the sixteenth.
  The brief for that judge is written and the judge is not started.

**What is wrong with the screen, as its builder now sees it:** it is a web dashboard's
look (floating cards, pastel pills, soft shadows), not a drawing's or a sheet's; the
left panel is one flat list of some twenty rows with tools and actions at one weight;
actions sit behind letter keys listed under "Keys" where they should be on the thing
they change; the estimate cannot be edited on the line; "To settle" says "55 values,
the first: ..."; the messages are long sentences in a manner of my own ("found as
drawn", "refused: ...") where a short label and a plain message belong; and the
spelling is British. Order step 7 (the visual standard) is not started.

**Proposed, and waiting on him:** a sheet of every word on screen beside the US trade
term to replace it; a short rule for messages; and a layout for him to mark up before
any of it is coded.


## The screen redrawn (2026-11-12, DECISIONS.md call 55)

Patrick's word after the review: "Use online search for best UI for video game world
building interfaces and you make the call proceed with previous plan to deliverable."
The searches, what was taken from them and the rule for words are call 55. This is
what was built, what it changed, and what was run.

**What was built** (`crates/app`):

* **A new layout.** Along the top: the job, Save, Undo, Redo, the view (3D, Plan,
  Walk), Frame, the price in the colour of how sure it is, how many clashes are still
  questions, the tally of Known, Assumed and Unknown, the file and whether it is saved.
  Down the left: Next steps, Levels, Drawings, Show, Systems. Along the bottom: the
  build bar, sixteen tools in three groups, each a button with a mark drawn the way a
  plan draws the thing; over it the fields of the tool in hand and what to do with it.
  Down the right: what is selected, with its actions as buttons and its values as rows
  that say Measure or Check. One line of status at the foot.
* **Fields in place of a typed line of commas.** A tool's values (a wall's thickness
  and height) are each a field with its name over it and an example in it; Tab, a comma
  or a click goes to the next; Enter takes the tool in hand. A new level is three
  fields under Levels. What is read on site is typed under the value it is read for.
* **Next steps**: ten steps read from the job itself (a level; a drawing with its
  scale; walls; doors and windows; the floor and the rooms; the roof; ducts, pipes and
  wiring; prices; verify in field; save). Each is ticked when the job shows it; the
  first that is not says how it is done, in the words on the buttons. It folds away.
* **Three views laid over the building**, under Show: by trade (as before), by how
  sure (everything drawn in a pale neutral, amber or red for Known, Assumed, Unknown),
  and clashes marked on the model whatever tab the sheet is on.
* **The look**: flat panels set edge to edge and parted by hairlines; no shadows, no
  rounded cards, no pills; a tag is a square mark and a word. The 3D view has a key
  light, a fill, a pale ground with its grid and no horizon: no sky, sun or grass in
  either graphics mode. On the plan a floor is paper.
* **The words**: US English, and a floor of the building is a level, said in the one
  place every word on screen passes (`theme::spoken`), with a test. A name a person
  typed is not respelled. Save, Undo, Redo, Delete and Cancel are said as every program
  says them. A refusal reads "Can't <do it>: <why>." Headings on what is selected are
  As drawn, Takeoff and Cost lines; "To settle" is "To verify".
* **Floors, rooms and roofs drawn in the window** (order step 5, call 53): the floor
  verb, its outline and its tests are in the model and at the command line (`floor`,
  `type ... floor`); Floor, Room and Roof are tools on the build bar.
* **`PLUMBLINE_MAP`**: the program writes where each button is on screen, by the word
  on it, for the demonstrations (`crates/app/src/map.rs`, `demos/window.sh`). A
  demonstration still moves the real pointer and presses the real button; it now asks
  the program where "Wall" is, and stops, failing, when a button it was to click is
  not on screen.

**Changed behaviour, each named:**

* The number keys 1 to 5 no longer put a trade on show or off; 1 to 9 and 0 take a
  build tool in hand. The trades are rows under Show.
* A tool clicked in 3D turns to the plan; with the whole building on show it draws on
  the lowest level; with no level it opens the form for one. Each used to be refused.
* A tool's settings that will not do leave its form open at the field at fault. They
  used to be thrown away with the refusal.
* A click on a button while a value is being typed lets go of the value and does the
  button. It used to be ignored.
* Tab with no roof in the job does nothing. It used to say there was no roof plane.
* Ctrl+S, Ctrl+Z and Ctrl+Y save, undo and redo (F2, Z and Y still do). W A S D do not
  move the view while Ctrl is held.
* Adding a drawing with the whole building on show puts it under the lowest level and
  turns to the plan. It used to be refused until a level was chosen.
* Every message the window writes was reworded (shorter, a capital and a full stop,
  "Can't ...:" for a refusal). The model's own reasons are unchanged, and so is
  everything the command line writes.
* A level's row is named as typed; "Storey" as a kind of thing is "Level" on screen.
* The left panel lists levels from the top down, as a section reads. It listed them in
  the job's order.
* The picture's ground is a neutral grey-white in both graphics modes. In the full mode
  it was grass under a sky.

**Run so far:** `cargo test -p plumbline-app` (3 pass, one new: the words), the three
`build.rs` tests of the floor verb (pass), `tools/portmap.sh` (206 items: 124 ported,
82 ours), the language gate (pass). The window was opened on the made-up house and four
pictures were taken and read: the layout is as described.

**The nine window demonstrations, run on `1fe4d1c`** (each converted to click buttons
by the word on them; their pictures under `demos/captured` are from that commit):

| Demonstration | Result |
|---|---|
| `draww` (a house drawn from an empty job) | PASS, the same hand figures as before: 5 walls, 96 LF, net faces 123, 132, 144, 195, 204 SF; rooms 162.75 and 193.75 SF |
| `fieldw` (field entry, and the verify-in-field tab) | PASS; byte for byte the command line's twin; the Verify in field tab shows the five rows and "$518.40 riding" worked by hand, then one row |
| `clashw` (fittings, a connection, clashes) | PASS |
| `house` (the made-up whole house walked round) | PASS |
| `d2` (the roof: pitch, eave, gable, condition, by their buttons) | PASS |
| `d4w` (allowance and unit cost typed on what is selected) | PASS |
| `d5w` (the estimate sheet; overhead, profit, material, labor) | PASS |
| `planw` (a real scanned sheet and a vector plan traced) | PASS |
| `buildw` (floor, rooms and roof with their tools over a traced plan) | **The session ran and the window shows the hand figures** (walls 120.00 LF, floor 600.00 SF, rooms 565.50 SF, roof 670.82 SF, eaves 100, hips 60, ridge 10 LF: picture 05). **The script then failed in its own closing check**, which read a room's name from the wrong place in the job file. That check is corrected; the demonstration has not been run again since. |

**Pictures read so far, as a person:** draw 02 and 07; field 01 (model and sheet);
house 01; d2 05; d4 04; clash 04; build 05. Not yet read: the rest.

**Seen in those pictures and not right yet:** the sheet still has a table headed "To
settle"; a level offers Condition and Unit cost, which price nothing on a level; a
flagged figure that has a value is listed under To verify as having "no figure"; the
roof is not drawn on the plan of a level after the Roof tool makes it; a long job title
is cut mid-word on the top bar. Each is being fixed with the next change.

## The verify-in-field list. How its demonstration will be run (written before it is built)

Order step 6; DECISIONS.md call 54 has what a question is and what rides on it. One
rule is added to call 54 here, found working the figures below: **only a line whose
amount is itself not Known has dollars riding on anything.** A line that has come out
Known rests on nothing that is still in question, whatever group of values it names.

**The job** is the field job (`demos/field-job.sh`, described under "Field entry"): two
levels scaled off a plan, four Known walls standing from the ground level's floor to
the upper level's, one of them carrying gypsum board on its net face at $2.40/SF; a
traced duct; two Known pipes across it.

**Before any measurement, by hand.** What has a value that can be measured or checked
and is not Known: the ground level (its elevation and its floor-to-floor), the upper
level (the same two), and the duct (its type, its line on the plan, its height above
its level). The walls, the pipes and the types are Known. Two clashes rest on the
traced duct and are questions.

* The south wall's line is 24 ft by 9 ft, 216 SF at $2.40: **$518.40**, Assumed. Its
  height is worked from the ground level's floor and the upper level's floor, so that
  $518.40 rides on the ground level, and the same $518.40 rides on the upper level.
* Nothing is priced on the duct or on the pipes: the duct and the two clashes are
  questions with no dollars, and say so. No dollars is not $0.00.
* **The list, largest first, ties in the job's order, questions with no dollars after
  those with, clashes before things:**
  1. ground (level): **$518.40**, 1 line. Measure: elevation, floor-to-floor.
  2. upper (level): **$518.40**, 1 line. Measure: elevation, floor-to-floor.
  3. The duct and pipe A in the same space, by 2 in: no dollars.
  4. The duct and pipe B in the same space, by 5 in: no dollars.
  5. The duct: no dollars. Check: type, line on the plan. Measure: height above its level.
* **Riding in all**, each line counted once: **$518.40**, 1 line, of a job cost of
  $518.40 (price $615.86).

**After the four measurements of the field demonstration** (the ground floor `same`,
floor-to-floor 8 ft 10 1/2 in, the duct's line and type `same`, its underside at
7 ft 10 3/4 in):

* The ground level has nothing left to verify and is off the list. The duct is off the
  list. Pipe A is clear of the duct. Pipe B in the duct is a finding, not a question,
  and is not on this list (it is on the Clashes tab).
* The upper level's floor is Known now (it follows from the ground's), and its own
  floor-to-floor, 8 ft, is still as scaled. The south wall is 213 SF, $511.20, Known:
  nothing in question rides on it. **The list is one row: upper (level), no dollars,
  measure floor-to-floor.** Riding in all: nothing.

**`demos/vif.sh`** runs `plumbline vif` on the job before and after and holds each row
and each figure above. **In the window** (`demos/fieldw.sh`, two pictures added): the
sheet's Verify in field tab before, with its five rows and the top bar saying what
rides; a click on the ground level's row selects it in the model window, where its two
values say Measure; and the tab after, with its one row.

The judge tries to get a question off the list without a measurement, to get dollars
counted twice in the total, to find a thing that is not Known and priced and missing
from the list, and to make the list and the Clashes tab disagree about what is a
question.

## Exports and lenses. How their demonstration will be run (written before they are built)

Order step 8, release item 8: "xlsx with live formulas, CSV, and IFC4 validated by
IfcOpenShell. Supplier, subcontractor and PM lenses come out of one estimate." Q17 has
what is in each lens (the working assumption there stands).

**The job** is the made-up priced job (`plumbline priced`, the job of D5, every figure
in it worked by hand in `demos/d5.sh`), with overhead 10 and profit 8 set on it.

**The estimate, by hand** (from D5's lines): Material $8,628.10 (six lines: 4,150.70 +
377.34 + 2,490.42 + 226.40 + 1,113.24 + 270.00); Labor $2,147.97 (1,207.48 + 724.49 +
216.00); Fabrication $402.50; Supervision $3,800.00; Subcontract $10,313.86. **Job cost
$25,292.43**, thirteen lines. Overhead 10%: $2,529.24. Profit 8% of $27,821.67:
$2,225.73. **Price $30,047.40**, flagged, and leaving out one line (the north plane's,
which has no price and no class), which it names.

**`export estimate <job> <out.xlsx>`: a workbook whose figures are formulas.**

* Sheet "Estimate": a row a line, with its element, its description, its class, what
  is priced and its unit to every place the program holds, the price it is extended at,
  and its amount as a formula, the quantity times the price rounded to the cent. A line
  with no figure has no amount and says why; a line switched off says so and is in no
  sum. How sure each is, is a column.
* Sheet "Recap": each class a formula over the Estimate sheet's amounts; the job cost
  their sum; overhead and profit each a percentage in a cell of its own and a formula
  on it; the price their sum; and under it, in words, what the price leaves out.
* **What the demonstration holds.** The workbook's formulas, worked by a second hand
  (a short evaluator in the script that knows only ROUND, SUM, SUMIF, the four
  operations and cell references), come to the figures above to the cent. Then one
  price is changed **in the workbook** (the south plane's underlayment, $0.90 to
  $0.95/SF) and the formulas worked again: 419.2627 SF at $0.95 is $398.30, up $20.96;
  Material $8,649.06; job cost $25,313.39; overhead $2,531.34; profit 8% of $27,844.73,
  $2,227.58; **price $30,072.31**. The same price changed in the job gives the same
  figures from the program. The formulas are live, and they are the program's
  arithmetic.

**`export estimate <job> <out.csv>`**: the same rows as values. Brought back in as an
estimate made before (`import prior`) and set against the job it came from
(`compare`), every line agrees and nothing is left over: the CSV round-trips.

**The lenses**, each `export <lens> <job> <out.csv|out.xlsx>`, each from the one estimate:

* **Supplier** (what to quote; none of our prices): Copper, 16 oz: 461.1890 + 276.7134
  = **737.90 lb**, two lines. Copper, 20 oz: 123.6931 + 30 = **153.69 lb**, two lines,
  Assumed. Underlayment: 419.2627 + 251.5576 = **670.82 SF**, two lines. No price, no
  amount, no labor. It says the north plane's line is left out and why.
* **Subcontractor** (the scope of the lines classed Subcontract; none of our prices):
  the west plane, standing seam roofing, **251.56 SF**. One line.
* **PM** (every line, its hours, what is ordered, its class and its code; the budget by
  class; what is to verify in the field; no overhead, no profit, no price): the
  thirteen lines; hours 29.8328 labor, 6.7082 fabrication, 40.0000 supervision; the
  budget by class as above, **$25,292.43**; and the verify-in-field list.
* A lens that leaves a column out says at its head what it leaves out.

**`export ifc <job> <out.ifc>`: an IFC4 model of the building.** On the made-up whole
house (`plumbline house`): two levels; sixteen walls (eight a level, 210.00 LF a level)
with their doors (5 and 4) and windows (8 and 9) in openings; two floors of 1,120.00 SF;
ten rooms; the roof's four planes; the ducts, pipes, conduit and cable, their fittings,
the equipment and the fixtures, each on its system. Lengths in feet. Every value's
status and source go with it in a property set, so what is Assumed is still said to be
in the model another program reads.

* **Validated by IfcOpenShell**: `ifcopenshell.validate` on the file reports no error,
  and IfcOpenShell counts the same sixteen walls, nine doors, seventeen windows, two
  slabs that are floors, ten spaces.
* **Read back by this program** (`import ifc` into an empty job): the same two levels
  at their elevations, the same 210.00 LF of walls a level and 1,120.00 SF of floor, as
  the takeoff of the job it came from.
* **What an IFC file does not carry is listed, not dropped in silence**: the estimate's
  lines and prices, the drawings, what was measured on site beyond the status and source
  of each value. The export says so, by count.

The judge tries to make a formula in the workbook disagree with the program by a cent,
to find a line the workbook's sums count that the program leaves out (or the other
way), to find a price of ours in the supplier's or the subcontractor's lens, to get a
quantity in a lens that is not the estimate's, and to find something IfcOpenShell
rejects or this program reads back differently.

## The verify-in-field list. What was run and what it showed

Built as planned above (`crates/world/src/vif.rs`, `plumbline vif`, the sheet's Verify
in field tab, the top bar).

* **`crates/world/tests/vif.rs`, three tests, pass.** As traced: the five rows in
  order, $518.40 on each level, one line counted once in the total. After the ground
  level is measured it is off the list and the upper level, whose own height is still
  as scaled, has no dollars riding (the wall has come out Known). A line on two things
  in question is counted once in the total ($518.40 + $240.00 = $758.40) and on each
  of the three rows it rides on. **Proven red first**: with the rule "only a line that
  is not Known has anything riding" taken out, the second test fails with $511.20
  where none is expected.
* **`demos/vif.sh` passes**: the five rows and their order as traced; four after the
  ground level is measured, none with dollars; one after the duct is measured (and the
  clash list then holds one finding, which this list does not); none after the upper
  level's height is looked at.
* **In the window** (`demos/fieldw.sh`, pictures 01 and 07, run on `1fe4d1c`): the tab
  shows the five rows with "$518.40 riding · 1 line of the estimate, each counted
  once"; the top bar says "5 to verify, $518.40 riding"; the click on the ground
  level's row selects it in the model window, puts it on show, and its two values say
  Measure. Picture 01, model and sheet, was read. Picture 07 has not been read yet.
* **Not right, and known:** the list works out the estimate, the elements and the
  clashes again each time the window redraws, beside the window's own reading of the
  same three; on the made-up whole house the window is no slower to the eye on this
  machine, and nobody has timed it. A wall's dollars ride on every level its height
  is worked from, whichever of that level's two values is still in question (call 54
  says so; the one rule added keeps a Known line from riding on anything).
* **Not judged.**

## Exports and lenses. What was run and what it showed

Built as planned above: `crates/world/src/export.rs` (the estimate as sheets of cells,
the three lenses), `plumbline_sheets::xlsx` (on `rust_xlsxwriter`, MIT OR Apache-2.0,
passed by `tools/licenses.sh`), `crates/ifc/src/write.rs` (the IFC4 writer), and the
verbs `export estimate|supplier|subcontractor|pm|ifc`.

* **`crates/world/tests/export.rs`, two tests, and `crates/ifc/tests/write.rs`, three,
  pass.** Each amount is the formula on its own row and comes to the job's figure;
  the recap's figures are the hand figures ($8,628.10, $2,147.97, $402.50, $3,800.00,
  $10,313.86; job cost $25,292.43; overhead $2,529.24; profit $2,225.73; price
  $30,047.40); a class with nothing in it says "none"; what is left out is named
  once. The supplier's lens is 737.9024 lb, 670.8203 SF and 153.6931 lb (Assumed), the
  subcontractor's one line of 251.5576 SF, neither with a dollar in any cell; the PM's
  has the job cost and no markup. The house is written as 2 levels, 16 walls, 9 doors,
  17 windows, 2 floors, 10 rooms, 1 roof of 4 planes; written twice it is the same
  file to the byte; every identifier is 22 characters and no two are the same; read
  back, each level's walls (210.00 LF), wall faces, floor (1,120.00 SF), rooms, doors
  and windows are what was written, and come back Assumed.
* **`demos/export.sh` passes**, with the formulas worked out by a second hand
  (`tools/workbook.py`, which knows ROUND, SUM, SUMIF and the four operations and
  nothing of this program): 26 formulas, each to the figure written with it. A price
  changed **in the workbook** ($0.90 to $0.95/SF on the south plane's underlayment)
  gives $398.30, $8,649.06, $25,313.39, $2,531.34, $2,227.58 and **$30,072.31**, and
  the same price changed in the job gives the same from the program. The estimate as
  CSV, brought back as an estimate made before and compared, agrees on every line
  (eight amounts at +$0.00; the ninth has no price in either), and says the three
  columns a prior estimate has no place for.
* **IfcOpenShell 0.9.0** (`tools/ifc-check.py`; a checker, not linked) reports **no
  fault** in the written house against the IFC4 schema and its rules, and counts 2
  IfcBuildingStorey, 16 IfcWall, 9 IfcDoor, 17 IfcWindow, 2 IfcSlab that are floors,
  10 IfcSpace, 1 IfcRoof.
* **A fault the first test seemed to find and did not:** "two things have one
  identifier" was the test's own parser taking a 22-letter value name for an
  identifier. The test was tightened; the identifiers were unique throughout.
* **What the export does not do, said so it is not taken for done:** a door's and a
  window's body in the model is the space it takes up in its wall, as thick as the
  wall (the job holds no thickness for a leaf or a sash). A piece of equipment is
  written as a distribution element of no stated kind, and a fixture as a flow
  terminal: the job does not say which kind of either it is. A system IFC has no word
  for (return air) is written by its name as a user-defined sort, and this program's
  own reader then brings what is on it back on no system and says so. A wall shorter
  than it is thick would be read back turned. Fittings, fixtures, equipment and the
  roof are written and are **not read back** by this program's reader, which has no
  element to put them in from a model; it says so, by kind and count.
* All fifteen command-line demonstrations pass on the tree that holds these (225
  tests). **Not judged.**

## The first job, on a real plan. How its demonstration will be run (written before it is run)

Release item 3: "Starting from an empty job, it opens a real residential plan PDF,
calibrates it, and traces walls, openings, floors, rooms and the roof. The 3D house
builds from the tracing." `demos/planw.sh` already traces the outside walls of USDA
Plan No. 7213 (a public plan, fetched by `tools/fetch-plan.sh`, not in the repository).
`demos/firstw.sh` goes the whole way on it, by the buttons a first user is pointed to
by Next steps, from a job whose file is not there yet.

**The session:** Add level (Ground, 0 ft, 8 ft). Add drawing (the PDF's page 4). Set
scale from the sheet's 41'-3" dimension. Wall (6 in, 8 ft): the southwest outside
corner by eye, then 41' 3" east, 23' 11" north, 41' 3" west, and closed. Door (3 ft,
7 ft, sill 0) 10 ft along the south wall. Window (4 ft, 3 ft, sill 3 ft) 30 ft along
the south wall and 20 ft along the north. Floor (4 in): one click. Room: one click
inside, named "House". Roof (6:12, eaves 8 ft): the four corners and the first again.
Then 3D, and Save.

**By hand.** 23'-11" is kept as 23.9167 ft (a place on the plan is kept to a
ten-thousandth of a foot).

* Walls: 2 x (41.25 + 23.9167) = **130.3334 LF**; at 8 ft, 1,042.6672 SF of face; less
  the door (21 SF) and two windows (12 SF each), **997.67 SF** net.
* Floor, the outline the walls enclose along their lines: 41.25 x 23.9167 = 986.5639,
  **986.56 SF**.
* The room inside the walls' faces (each wall 6 in, 3 in either side of its line):
  40.75 x 23.4167 = 954.2305, **954.23 SF**.
* The roof, 6:12 on every side of 41.25 by 23.9167: 986.5639 x the square root of 1.25
  = 1,103.0119, **1,103.01 SF**; a ridge of 41.25 - 23.9167 = **17.33 LF**; four hips,
  each half the width (11.9584) x 1.5 = 17.9375, **71.75 LF**; eaves **130.33 LF**.
* Everything traced is Assumed and says it was traced from the drawing; the scale is
  Assumed, read off the drawing.

The saved job has to give these, and the pictures have to show the house on its sheet.
Where a place on the plan is clicked after the first corner, the script asks the
program where that many feet from the first corner is on screen (the `plan` line of
`PLUMBLINE_MAP`), in place of a pixel copied in by hand.

## Sixteen divisions. How its demonstration will be run (written before it is built)

DECISIONS.md call 57: the focus is a full 16-division building platform. Every cost line
carries its division; the estimate and its recap read division by division; a line with
no division is said to have none.

**The job** (`demos/div.sh`, made at the command line; every price is made up): one
level, 9 ft floor to floor. Four 6 in walls round 30 ft by 20 ft, 9 ft high: 100 LF,
900 SF of face. In the south wall a 3 ft by 7 ft door and two 4 ft by 4 ft windows
(21 + 16 + 16 = 53 SF): the south wall's net face is 270 - 53 = 217 SF, the north's
270, the east's and west's 180 each; 847 SF in all. A 4 in floor, the outline of the
walls: 600 SF. One room inside the walls' faces, 29.5 by 19.5: 575.25 SF. A roof at
6:12 on every side, eaves at 9 ft: two planes of 111.8034 SF and two of 223.6068 SF,
a 10 LF ridge. A 20 LF duct, a 14 LF pipe, a 40 LF cable, a lavatory, a furnace.

**One assembly, "Exterior wall, 2x6, sided", is applied to each of the four walls** and
puts lines in three divisions on each: framing by the gross face (Division 6);
insulation and siding by the net face (7); gypsum board and paint by the net face (9).

**The lines, by division, each worked by hand:**

| Division | Line (on what) | Quantity and price | Amount |
|---|---|---|---|
| 1 General Requirements | Superintendent (the job): 5 days at 8 hr | 40 hr at $85.00 | 3,400.00 |
| | Building permit (the job) | 1 at $1,200.00 | 1,200.00 |
| 2 Site Construction | Excavate and gravel base, 6 in (the floor) | 600 SF at 54 SF to the CY: 11.11, bought as 12 CY at $48.60 | 583.20 |
| 3 Concrete | Concrete slab, 4 in (the floor) | 600 SF at 81 SF to the CY: 7.41, bought as 8 CY at $165.00 | 1,320.00 |
| 4 Masonry | Stone veneer, a 3 ft band (the south wall) | 30 LF by 3 ft: 90 SF at $38.00 | 3,420.00 |
| 5 Metals | Steel ridge beam (the ridge) | 10 LF at $42.00 | 420.00 |
| 6 Wood and Plastics | Wall framing (four walls) | 900 SF at $4.20 | 3,780.00 |
| 7 Thermal and Moisture Protection | Insulation (four walls) | 347.20 + 432.00 + 288.00 + 288.00 | 1,355.20 |
| | Siding (four walls) | 2,061.50 + 2,565.00 + 1,710.00 + 1,710.00 | 8,046.50 |
| | Standing seam roofing (four planes) at $14.00 | 1,565.25 + 1,565.25 + 3,130.50 + 3,130.50 | 9,391.50 |
| 8 Doors and Windows | Entry door, prehung | 1 at $2,400.00 | 2,400.00 |
| | Window unit (two windows) | 2 at $1,150.00 | 2,300.00 |
| 9 Finishes | Gypsum board (four walls) at $2.40 | 520.80 + 648.00 + 432.00 + 432.00 | 2,032.80 |
| | Paint (four walls) at $1.10 | 238.70 + 297.00 + 198.00 + 198.00 | 931.70 |
| | Hardwood flooring (the room) | 575.25 SF at $12.00 | 6,903.00 |
| 10 Specialties | Toilet and bath accessories (the room) | 1 at $450.00 | 450.00 |
| 11 Equipment | Kitchen appliances (the room) | 1 at $8,500.00 | 8,500.00 |
| 12 Furnishings | Window blinds (the room) | 2 at $380.00 | 760.00 |
| 13 Special Construction | Sauna, prefabricated (the job) | 1 at $9,200.00 | 9,200.00 |
| 14 Conveying Systems | Platform lift (the job) | 1 at $14,500.00 | 14,500.00 |
| 15 Mechanical | Duct 20 LF at $18.00; pipe 14 LF at $9.50; lavatory $640.00; furnace $4,300.00 | 360.00 + 133.00 + 640.00 + 4,300.00 | 5,433.00 |
| 16 Electrical | Cable 40 LF at $2.25; light fixtures 6 at $185.00 (the room) | 90.00 + 1,110.00 | 1,200.00 |

(**Corrected before the demonstration was first passed, 2026-11-12:** this table first
had gravel and concrete at 11.1111 and 7.4074 CY, $540.00 and $1,222.22. The program
buys what is ordered by a unit in whole ones, as it always has (a sheet, a roll, and
so a cubic yard), and the first run showed 12 and 8. That is the program's standing
rule and the right one for a truck of gravel or concrete, so the hand figures were
wrong and are corrected here; nothing in the program was changed to meet them.)

**By division:** 1: $4,600.00. 2: $583.20. 3: $1,320.00. 4: $3,420.00. 5: $420.00.
6: $3,780.00. 7: $18,793.20. 8: $4,700.00. 9: $9,867.50. 10: $450.00. 11: $8,500.00.
12: $760.00. 13: $9,200.00. 14: $14,500.00. 15: $5,433.00. 16: $1,200.00.
**Job cost $87,526.90**, the sum of the sixteen. Overhead 10%: $8,752.69. Profit 8% of
$96,279.59: $7,702.37. **Price $103,981.96.**

**Then a line with no division** is added to the job (a contingency of $2,000.00, cost
type general conditions). The job cost is $89,526.90. The divisions still come to
$87,526.90, and the recap says under them: 1 line, $2,000.00, is in no division. Given
Division 1, Division 1 is $6,600.00 and nothing is left out.

**What the demonstration holds:** each division's total and the job cost from
`plumbline estimate`; the same sixteen as formulas in the workbook
(`export estimate`), worked again by the second hand; the division on each line of the
CSV; the assembly written out and read back with its divisions (`export assemblies`,
`import assemblies`); a division that is not one of the sixteen refused (`0`, `17`,
`seven and a half`), the file byte for byte what it was. In the window, the Estimate
tab reads division by division, each with its total.

## Sixteen divisions. What was run and what it showed (the model and the command line)

* **Built:** `Division` (the sixteen, in the job file as its number); a division on
  every cost line; `Estimate.divisions` (all sixteen, each over the lines of the job
  cost in it) and `Estimate.no_division` (what is in the job cost and in none);
  `plumbline line ... --division`, `plumbline division`, and a BY DIVISION recap in
  `plumbline estimate`; a Divisions sheet of formulas in the workbook, a Division
  column in the CSV and in the PM's lens with the budget by division; each line's
  division kept in an assembly library, written and read.
* **Changed, and named:** an assembly could be put only on a roof plane or a roof
  line. It now goes on anything that is built (a wall, a door, a window, a floor, a
  room, a run, a fitting, a fixture, a piece of equipment, a roof) and on the job.
  That limit was the roofing program this came from, and it is what kept the rest of
  the building out of the estimate. The assembly library's heading "Class" is "Cost
  type" and it has a "Division" column; a library written before still reads. The
  workbook has a Divisions sheet and its Estimate sheet a Division column.
* **`crates/world/tests/divisions.rs`, four tests, pass**: the sixteen titles in order
  and every way a person writes one; totals by division, a division with nothing in
  it having no total, what is in none counted apart, and the sixteen and that coming
  to the job cost; a line with no cost type out of its division as it is out of the
  job cost; the division kept in the file as its number, and 0 and 17 refused on load.
* **`demos/div.sh` passes**: the house's 46 lines land in all sixteen divisions at the
  hand figures above ($4,600.00, $583.20, $1,320.00, $3,420.00, $420.00, $3,780.00,
  $18,793.20, $4,700.00, $9,867.50, $450.00, $8,500.00, $760.00, $9,200.00,
  $14,500.00, $5,433.00, $1,200.00; job cost $87,526.90; price $103,981.96); the line
  with no division is said to be in none and then is in Division 1; the workbook's 72
  formulas come to the figures written with them, the Divisions sheet among them; 0,
  17, "seven and a half" and "Roofing" are refused and the file is what it was.
* **Two of the hand figures were wrong and were corrected, not the program**: see the
  note in the plan above (gravel and concrete are bought by the whole cubic yard).
* 229 tests and all sixteen command-line demonstrations pass on this tree.
* **Not done yet:** the window does not show the divisions (next); there is no
  library of assemblies for each division shipped with the program; the model has no
  element for much of Divisions 2, 3, 5, 10 to 14 (footings, steel, stairs, cabinets,
  appliances), whose lines sit on the job, a floor or a room for now; "condition" is
  still the word on screen for an assembly, and "class" for a cost type in places.
  **Not judged.**

## Sixteen divisions on the sheet (2026-11-12)

* **Built:** the sheet window's Estimate tab reads division by division. A "By
  division" table lists all sixteen with their line counts and totals (and "In no
  division" when anything is in none); under it "The sheet" has every line under its
  division's heading (`DIVISION 7  THERMAL AND MOISTURE PROTECTION` and its total),
  and each line's division is a field on the line (`Div 7`): click it, type a number
  or a title, Enter. `plumbline sixteen <file>` writes the sample whole house with a
  made-up price on everything in it, 347 lines in all sixteen divisions (job cost
  $401,094.08, price $476,499.77, every price made up and saying so).
* **Changed, and named:** the sheet's tabs are in a new order, Estimate, Budget,
  Verify in field, Clashes, Building, Roof, Mechanical, Electrical, Plumbing; the
  estimate is what the job is for and is first. The tab "Money" is "Budget" (it is
  the allowances set against the detail). The sheet opens on Estimate when the job
  has any cost line; before that on what was taken off, as it did. The "At" column
  is wider (a price of $1,800.00/EA was cut). `demos/d4w.sh` clicks "Budget".
* **Seen, by opening `plumbline sixteen` in the window and reading both pictures:**
  the nine tabs sit on one line (they wrapped and cut "Verify in field"); the sixteen
  rows and their totals read as a recap does; prices are whole. In the model window
  the house is a plain box with flat-coloured walls and bare openings. That is task
  23, next. The top bar's right end is cut at 1600 wide ("hous…").
* **Tests:** a fifth in `crates/world/tests/divisions.rs`: the priced sample has work
  in every one of the sixteen, nothing in none, the sixteen add to the job cost to the
  cent, and every price says it is made up. 230 tests pass.
* **Not judged. Not yet demonstrated by a script through the window** (a division
  typed on a line of the sheet); `demos/d5w.sh` and `demos/d4w.sh` are to be rerun on
  the new tab order with the rest.

## The house dressed as built. How its demonstration will be run (written before it is built)

DECISIONS.md call 58. `demos/dress.sh` (command line) and `demos/dressw.sh` (window).

1. **The reading, at the command line.** On a made-up job: a level; four 6 in walls
   round 24 ft by 16 ft, 9 ft tall; a partition; a door and two windows; a hip roof.
   `plumbline finishes` first says every wall, the roof and each opening is bare,
   and why ("nothing priced on it names a finish"). Then lines are added the way an
   estimator adds them and the reading follows, each with the line that said it:
   * south wall, "Brick veneer, red" (Division 4): brick, red, outside face south;
   * east wall, "Fiber cement lap siding, white" (Division 7): lap siding, white;
   * north wall, "Stone veneer wainscot, 3 ft" and "Cedar shingle siding": shingles
     above a stone wainscot 3 ft high;
   * west wall, "Stucco, three coat": stucco;
   * the partition, "Brick veneer": said to be a partition, both sides inside, so
     the line is priced and no cladding is drawn, and the reading says that;
   * the roof planes, "Standing seam roofing, charcoal"; one changed to "Slate
     roofing" reads slate alone;
   * a window, "Window unit, clad wood, double-hung, divided lites, black": double-
     hung, divided, black; the other window, no line: a frame, one sash, one light;
   * the door, "Entry door, six panel, prehung": a panel door.
   A line switched off stops dressing the wall (it is out of the estimate, so it is
   off the house). (Written before the build; in the build only a labor line can be
   switched off, an existing rule, so this is shown with a labor line that names the
   siding, and a material line is taken off the house by renaming or removing it.) A wall's outside face is the one outside the outline: checked for
   all four against the plan by hand (south wall drawn west to east: outside is its
   right; east wall drawn south to north: its right; and so on round).
2. **The same in the window** (`dressw.sh`): `plumbline sixteen` opened and walked
   round, front, corner, a window close up, the entry door close up, the roof from
   above, inside looking at a partition; the unpriced `plumbline house` beside it,
   bare. Every picture read by eye against the standard: do the courses run level
   and at size, do corners close, do the window parts sit in the wall, is anything
   glossy, is anything swimming or flickering.
3. **Tests** (`crates/world/tests/finish.rs`): every word of the vocabulary to its
   finish; a color word; outside faces of a rectangle of walls and of an L; a
   partition; a line off; the reading unchanged by a save and a load.

## The house dressed as built. What was run and what it showed (2026-11-12)

* **Built** (DECISIONS.md call 58): `plumbline_world::finish` reads what each wall,
  roof plane, window, door and eave is finished with from its own live cost lines,
  its material and its assembly, and which face of a wall is outside (the one outside
  the outline its level's walls enclose). `plumbline finishes <file>` prints the
  reading with the line that says each thing. The model window draws it
  (`crates/app/src/dress.rs`): six claddings (lap siding, board and batten, shingles,
  brick, stone, stucco) and six roofings (asphalt shingles, standing seam, slate,
  tile, shakes, membrane) as patterns made in the program at trade sizes, with
  relief; a wainscot under a cladding; corner boards on siding; a window built of
  frame, sash, glass, muntins, casing and sill (double-hung, casement, slider or
  fixed; divided lites as "6 over 6" or left to the size); a door of frame, leaf
  (panel, glazed, overhead or plain), casing and knobs; a gutter on an eave that has
  one priced. The right panel has a Finish block on whatever is selected: what it is
  dressed in and the line that says so, or that it is bare and why.
* **Flagged, not silent:** a thing nothing names a finish for is drawn bare and says
  so. A second, different finish priced on the same thing is said ("also priced on
  it and not drawn"), not dropped. A cladding priced on a partition is said to be
  priced and not drawn. A wainscot with no height written is drawn 3 ft and says the
  height is not written. Walls that enclose nothing say which face is outside is not
  known and are dressed on both.
* **Changed, and named:**
  * An outside wall was told from a partition by its thickness (6 in or more). It is
    now told by the outline. Wall ends at a corner now run past to close the corner
    (they left a notch half a wall square).
  * A window was one pane of glass and a door one slab. Glass is darker and less
    clear (it was a pale wash).
  * A roof plane's color came from its assembly's name holding "copper". It now comes
    from the same reading as everything else; a plane with an assembly but no line,
    material or assembly naming a roofing is bare gray where it was blue-gray.
  * The priced sample (`plumbline sixteen`): every door but "bath door" was priced as
    an entry door at $2,400; only the front door is one now and the rest are interior
    doors at $640. Its windows and doors say what they are ("double-hung, divided
    lites, bronze"; "mahogany, six panel"). **Job cost $401,094.08 to $390,534.08;
    price $476,499.77 to $463,954.49.** Division 8 moved by the same $10,560.00.
* **Tests:** `crates/world/tests/finish.rs`, six: the vocabulary; a color is a whole
  word ("standing" holds no "tan"); outside faces of a rectangle drawn
  counter-clockwise (south, east, north, west) and a partition; walls that enclose
  nothing; what lines name and which line, a second cladding said, a labor line
  switched off undressing its wall; the reading the same after a save and a load,
  and the counts on the priced sample (8 walls sided, 4 with a wainscot, 8
  partitions, 4 planes, 4 gutters, 17 windows double-hung) against the unpriced one,
  bare. `crates/app/src/dress.rs`, three: every pattern made with its nine smaller
  sizes; brick laid 4 to 32 in across and 12 courses to 32 in; a double-hung window
  has two lights and the bars of 3 across and 2 up. **239 tests pass.**
* **`demos/dress.sh` passes:** the house bare, then each wall, the roof, a window and
  the door as the plan above lists them, word for word, and the second roofing said.
* **`demos/dressw.sh` passes** (pictures in `demos/captured/dress/`, taken on this
  tree): the measured colors are brick (124, 77, 67) and stucco (212, 204, 189) on
  the made-up house; cedar (144, 123, 103) against bare (183, 183, 179) at the same
  place of the sample house priced and unpriced; its roof (100, 103, 107) against
  (143, 154, 167). The same thresholds fail on the other job of each pair, which is
  the check proven red.
* **Pictures read, all eight of the model:**
  * `one-each/01`, `02`: brick in running bond, joints struck; a six-panel door with
    its knob in a brickmould; a black 6-over-6 double-hung; stucco round the corner;
    the corner closes clean (it flickered where two wall ends lay in each other's
    faces, found in the first run and fixed); standing seam with its seams, a white
    gutter on each eave.
  * `one-each/03`: cedar shingles in uneven widths over coursed ashlar to 3 ft, a
    corner board against white lap siding, an unpriced window as a frame and one
    light.
  * `one-each/04`: the south wall picked: "Brick veneer, red. On its south face.
    From the line "Brick veneer, red"".
  * `priced/01` to `03`: the sample house in cedar lap siding over a stone wainscot,
    corner boards, bronze double-hung windows with divided lites in white casings
    with drip caps and sills, a mahogany door, seams running up each plane.
  * `bare/01`: the same house with nothing priced: plain walls, a plain gray roof.
* **Not right, seen in the pictures:**
  * Where a roof overhangs its walls there is no soffit: under the eave the gap
    between the wall's top and the roof is open and reads dark.
  * A selected dressed wall takes a violet cast from the selection's glow.
  * First-floor casings overlap the top of the stone wainscot on the sample (its
    sills are at 2 ft 6 in and its wainscot is 3 ft): the sample's data, and what a
    drawing would show too.
  * No band between levels, no foundation, no shutters, chimney or porch: there is
    no element for them yet. The roof has no thickness.
  * The window demonstrations written before this (draww, fieldw, clashw, house, d2,
    d4w, d5w, planw, buildw, bringw) have not been rerun since; a click placed beside
    an opening may now land on its casing. They are rerun next.
* **Not judged.**

## Seventeenth judgment: launched (2026-11-12)

Two judges on Opus, on commit 8c6978f as pushed, each in a folder of its own with the
built command line and the source, briefed to break it and not to fix it:
**17A, the money path** (the sixteen divisions, the estimate and recap, the workbook,
CSV and lenses, the verify-in-field dollars, hostile input) and **17B, what the house
is dressed in, the job file, imports, the words at the command line, and the
documents**. Neither starts the window: the window demonstrations are being rerun at
the same time and are timed. A third judge, a veteran estimator with no software
background working only in the window, follows when they finish. Their findings are
recorded here the turn they arrive.

## The first job, on a real plan. What was run and what it showed (2026-11-12)

* **`demos/firstw.sh` passes, on its first run, on commit 8c6978f** (438 s; pictures
  in `demos/captured/first/`). From a new job, in the window, with the pointer and
  the keyboard only: a level added; the public-domain USDA Plan No. 7213 (page 4, a
  scanned sheet) added and given its scale from one dimension read off it, 41 ft 3
  in; the outside walls traced, a door and two windows set in them, the floor from
  the outline the walls enclose, a room named, a hip roof at 6:12; then 3D and save.
  The saved job gives the figures worked by hand before it was run: walls 130.3334
  LF, wall face less openings 997.67 SF, floor 986.56 SF, room 954.23 SF, roof
  1,103.01 SF in four planes, ridge 17.33 LF, hips 71.75 LF, eaves 130.33 LF; every
  one of them Assumed, traced off a drawing, and none Known. **This is release
  candidate item 3.**
* **Pictures read (01 to 06 of the model):** the empty job says where to start; the
  sheet comes in to scale; the walls trace over the plan's own walls in ink and read
  against the scan; the door and windows sit where the plan has them; the roof shows
  on the level's plan as broken lines.
* **Not right, seen in picture 06 and a real stumble for a person:** in 3D the house
  has no roof. The roof was added (the status line says so and the panel has its
  1,103.01 SF) and it is not drawn, because a roof is drawn in 3D only with "All
  levels" showing, and a job with one level has no "All levels" row to pick. Fix
  queued behind the demonstration rerun (the program is not rebuilt while timed
  demonstrations run): in 3D the roof is drawn whenever the level on show is the top
  one. `firstw.sh` is then rerun and gets a check that the roof is in the picture.
* **Also seen:** the house is bare, as it should be with nothing priced. The "To
  verify" notes under a room repeat one clause per wall ("whether wall 1 bounds this
  room rests on its heights, which are assumed; whether wall 2 ...") and should say
  it once: taken into the cold walk.

## Seventeenth judgment, part B (2026-11-12): what the house is dressed in, the job file, imports, the words, the documents

Judge on Opus, commit 8c6978f, command line and source only. 25 findings: 18 silent,
5 words, 2 doc, no wrong money, nothing that breaks. (The judge could not write its
file; its report came back as text and is set down here.) Every one reproduces from
the judge's commands on a 24 ft by 16 ft house.

| | kind | finding |
|---|---|---|
| J17B-1 | silent | Courtyard walls are read as partitions (the outside test knows only the outer outline), so cladding priced on them is not drawn. |
| J17B-2 | silent | A second building on a level, or a garden wall off a corner, is said to have "walls that enclose nothing" (only the largest piece is the outline) and is clad on both faces. |
| J17B-3 | silent | Trade words missed: Hardie plank, CMU, split face block, fiber cement panel all read "bare". |
| J17B-4 | silent | Lines that are not cladding dress the outside face: "Stone sill at windows", "Brick ledge flashing", "Siding nails", "Siding removal and haul off", "Shingle molding", "Paint, siding, white", "Granite countertop", "Kitchen backsplash, brick", "Fireplace surround, stacked stone, interior", "Interior wainscot, stone tile, 42 in". "Brick ties" ahead of "Lap siding, white" makes the wall brick. |
| J17B-5 | silent | A fixed order of words beats what the line says: "Lap siding with brick mold casing" is brick; "Lap siding, Cobble Stone" is stone; "Lap siding over 1x3 rainscreen battens" is board and batten; "Shingle siding, white, on a brick base" is brick; T1-11, vertical board and corrugated metal siding are drawn as horizontal laps. |
| J17B-6 | silent | Colors: "Western red cedar bevel siding" is red; "Red cedar shakes" red; a window takes "black" from "Window flashing tape, black" while quoting the line that says white; "Window trim, white", "Window screen, black", "Door casing, white", "Door threshold, bronze" color the unit; "Casement, black, 30x48" loses its color. |
| J17B-7 | silent | Underlayment and accessories are read as the roofing and the first wins: "#30 asphalt felt underlayment" then "Standing seam roofing, bronze" is drawn as asphalt shingles; "Ice and water membrane" then "Architectural shingles" is membrane; a copper gutter, copper flashing, drip edge, zinc strip, tile underlayment, felt, "Architectural shingles, Slate", "Metal shingles, stone coated", "Shingle tear-off" all misread. |
| J17B-8 | silent | Roofings missed: torch-down, 5V crimp, corrugated metal panels, rubber roof, a shingle by its brand name. |
| J17B-9 | silent | "Awning window" reads fixed; "Hopper" and "Bay" read "nothing says how it operates"; "6 over 1", "6/6" and "9 lite" read as not divided. |
| J17B-10 | silent | "Door from garage to house, 20 min fire rated" and "Garage door opener" read as an overhead door; "Door hardware, glass knob" as a glazed door; half lite, full view, Dutch, bifold read as a plain leaf. "a overhead door". |
| J17B-11 | silent | A wainscot height written in the line but not after a comma ("wainscot 36 in", "to 3'-0\"", "42 in high", "1 1/2 in thick, 42 in high") is read as "a height no line gives"; "0 in" and "1.5 m" the same. |
| J17B-12 | silent | A wainscot taller than its wall is accepted and the reading still claims the siding above it, which the picture cannot show. |
| J17B-13 | silent | A second wainscot, the same cladding in a second color, and a second window or door unit are dropped without "also priced, not drawn". |
| J17B-14 | silent | A wainscot priced on a partition vanishes from `finishes`; the panel lists it; the picture does not draw it. Three ways apart. |
| J17B-15 | silent | An eave with no gutter is not listed though SPEC 5.19 says each eave is read; "Gutter apron flashing, black" and "Remove existing gutters" read as a gutter; "eavestrough" does not. |
| J17B-16 | silent | An IFC written by this program with Assumed values comes back Known under `import ifc --status Known`: the per-value "(Assumed; ...)" it wrote is not read, and "not brought in: 0 things". The level's height comes back unknown and the report does not list it. |
| J17B-17 | silent | An IFC material and a property set are dropped with no word in the import's report. |
| J17B-18 | words | A roof slab, and this program's own roof planes written to IFC and read back, are reported as "floor slab". |
| J17B-19 | words | IFC report: "1 doors", "1 floors", "4 floor slab", "the model is in foot", "wall 44 in the file" (reads as 44 inches), "wallstandardcase". |
| J17B-20 | words | A hand-edited job file is refused in the program's internals ("unknown variant", "expected u8", "missing field", "pl::cost", "os error 2", "U+000A"); `check` says the bytes differ and not where. |
| J17B-21 | words | `plumbline wall w.json ground w6` says "no element ground" and not that a level is given by its number; `price` prints "priced by its net_area" and "4 RoofPlane, 4 Eave"; `show` prints plan_area, slope_factor; "measures re-derived on". |
| J17B-22 | words | Price lists: a row with a stray comma is blamed on its price; a date cell says "day 46031"; a formula with no saved figure says both "it has no price" and "what it last worked out to was read"; "#" is refused as pounds; "Hrs: the new material price". |
| J17B-23 | silent | A prior estimate whose own Total row ($17,000.00) disagrees with its lines ($17,404.00) is not said to disagree; "SQFT" is not taken as SF. |
| J17B-24 | doc | `plumbline roof` works and is in neither the usage nor the README; the usage says `export prices` writes `.csv` and `.xlsx` works too. |
| J17B-25 | doc | The estimate CSV does not come back into a job (the loss is reported when it is read as a prior estimate, but nothing reads it back as lines), against "the job file and CSV round-trip without loss". |

**What held** (the judge's list): outside faces on clockwise, L, U, a wall in two
pieces, an upper level; a switched-off labor line; "6 over 6"; EIFS, cultured stone,
dimensional shingles, EPDM; wainscot heights after a comma; every job made checks
byte for byte; every hostile hand edit of the job file is refused on load (division
0 and 99, a status outside the three, Unknown with a value, Known with no source,
duplicate path, low next id, a measure that does not match); price lists with a BOM,
CRLF, a header in row 3, duplicates, Latin-1, an unknown column (each said); an
assembly library out and back byte for byte with cost type and division; IFC2x3 and
millimetre models; what is not brought in is listed.

**The lesson in J17B-4, 5 and 7:** a finish was taken from any line that held a
finish's word. A line names a finish only when the finish is what the line is: its
first words, not something it mentions on the way to a sill, a flashing, nails or a
removal. That is the fix, with the outside test made to know every enclosed piece
and every courtyard. Fixes follow below as they are made.

## Seventeenth judgment, part A (2026-11-12): the money path

Judge on Opus, commit 8c6978f, command line and source only. 20 findings: 5 wrong
money, 8 silent, 5 words, 1 breaks, 1 doc. Each has commands that reproduce it from
an empty folder (the judge's file is kept beside the build, not in the repository;
its titles are set down here as written).

| | kind | finding |
|---|---|---|
| J17A-1 | words | a hand-edited division that is not a whole number is refused in software words |
| J17A-2 | silent | "the sixteen divisions" and "in no division" are shown Known, and say nothing of what they leave out, when the divisions under them are Assumed or leave lines out |
| J17A-3 | silent | a line with a division and no cost type vanishes from its division without a word |
| J17A-4 | wrong money | in the workbook, "In no division" is a typed figure, not a formula, so it stops agreeing with the lines the moment a price is changed |
| J17A-5 | words | the workbook's Recap says "Unknown: $0.00 of it rests on something assumed" and "-0" hours for every cost type with no lines |
| J17A-6 | wrong money | a line with no price yet gets no Amount formula in the workbook, and an empty cost type or division is the word "none", not a SUMIF; a price typed in later goes nowhere |
| J17A-7 | silent | the verify-in-field list drops a line that rides on the question but has no price yet, and its total does not say it left one out |
| J17A-8 | words | `set ... height` on a wall says "a Wall cannot carry a height" while the wall shows a height and `measure` takes one |
| J17A-9 | wrong money | on the estimate sheet, the quantity shown times the price shown is not the amount shown on 37 of the 347 lines of the sixteen-division sample |
| J17A-10 | silent | the project manager's lens gives each division a figure and says nothing of the lines it leaves out; a division with only unpriced work reads "none" |
| J17A-11 | wrong money | a price to a tenth of a cent is reported to the cent by `import prices` and `compare`, so the figures shown do not multiply |
| J17A-12 | words | software spellings on screen: escaped quotes, "1e9", "u8", an empty list, a garbled division by zero |
| J17A-13 | silent | an order factor that does not fit how a material is bought is taken without a word, and wipes out every line made of it |
| J17A-14 | silent | a line priced on a thing whose identity is Assumed or Unknown is Known, and the verify-in-field list says no dollars ride on the question |
| J17A-15 | doc | the usage's way to put a stretch-out on a cost line cannot be followed: each of the two steps refuses until the other is done |
| J17A-16 | breaks | `sixteen`, `sample`, `house` and `priced` write over an existing job without a word; `new` refuses the same |
| J17A-17 | wrong money | `import assemblies` reads a unit cost of "$1,23/LF" as $123/LF and "$1,2,3/SF" as $123/SF; every other way in refuses both |
| J17A-18 | words | when one row of an assembly is bad, every row of that assembly is refused with that one row's reason, under the wrong row numbers |
| J17A-19 | silent | the estimate reads Known and whole while parts of the building have nothing priced on them; only `price` says so. The sixteen-division sample prices the ridge cap and not the four hips |
| J17A-20 | silent | cutting a run where it crosses another makes the clash read 1.00 in instead of 5.00 in, and splits its dollars |

**What held** (the judge's list):

- Sixteen-division sample: the 16 division totals add to the job cost to the cent ($390,534.08); line counts add to 347; overhead 10% / profit 8% / price worked by hand agree.
- 45 randomised versions of the sample (divisions removed or moved, cost types removed, prices made Unknown, Assumed or changed to three decimals): every time, the sixteen + in no division = job cost on the command line, the CSV's classed amounts add to the job cost, and the workbook's formulas all worked out (tools/workbook.py) to the command line's job cost and sixteen.
- No line counted twice or dropped between estimate, CSV, workbook, supplier lens (201 material lines, quantities grouped by name and unit agree), subcontractor lens (142 lines, element/line/quantity/unit agree) and PM lens (347 lines, every field agrees).
- Every CSV row of the sample: quantity x unit cost, rounded half up, equals the amount.
- A switched-off labor line is out of the recap, its division and the workbook's SUMIFs, and shown "off ($4,000.00)".
- A line with no cost type is out of the job cost and named in the recap; the workbook marks its division "(in no total)".
- Hand edits of a division to 0, 17, -1, 7.5, "7", "Roofing", 1e3, 2^32+6 are refused at load; a class "Roofer", duplicate line names, a switched-off non-labor line, a negative price are refused at load; a stale unit cost in the file is re-derived on load.
- Division by the command line: 0, 17, "7.0", "09 Finishes", "seven and a half", no value: refused, file byte-for-byte unchanged. " 7", "07", "+7", "finishes", "Div 9", "Division 9" accepted as the division meant.
- Duplicate line names (exact, by capitals, by a trailing space), empty and blank names, a newline in a name, "1/0", "nan", "inf", unknown names in a formula, a missing element, a missing file: refused, file unchanged.
- Hand-worked small jobs: waste 10% + 32 SF/sheet whole sheets (6 sheets, $90.00); 81 SF/CY whole yards (3 CY, $495.00); 1.25 lb/SF; spacing count "plan_length / spacing + 1" (16 EA); stretch-out 19 3/4" (32.92 SF, $102.04); overhead 12.5% / profit 7.25% on the sum; half-cent lines (1.005 -> 1.01, 8.015 -> 8.02); a sub-half-cent line refused, not $0.00.
- Overhead and profit: -5 and "1e2" and "10.5.1" refused; 0, 100, 1000, 12.3456789, "10%" taken and worked; Unknown overhead gives no price and says why.
- Prices import: "$1,2,3", negative, 1e20, empty name, wrong unit, no status, no source, "€12", "twelve", "12%", "(12.00)" refused with the file unchanged; padded and differently capitalised names matched.
- Price list CSV round trip (export prices -> import prices --add -> export prices) is byte-identical, quotes and commas in names included; the estimate CSV quotes commas and quotes, guards leading = + - @ with an apostrophe, and `import prior` reads them back as written.
- Suppliers side by side: generic $1,050.00, two suppliers' quotes extended with waste ($991.20, ~$1,075.20) as worked by hand.
- VIF: a measured height turns the line Known, re-prices it (80 -> 85 SF, $200 -> $212.50) and drops "measure: height"; a measured duct height clears the clash and the list; "same" on all three of a duct's values turns the clash into a finding off the list; the head total counts each line once across a clash and a thing (180 + 180 + 95 = $455.00).
- Measure: negative, no unit, metric, "8 ft 13 in", "8.5.1 ft", no source, wrong value name, a wall line given one length: refused, file unchanged.
- Allowance $28.00/SF reconciles with a $4.20/SF detail ($4,480 vs $672, -85%); a $/LF allowance on a wall refused.
- `new` will not write over a file; export to a missing folder, to a .txt, or to the job itself is refused.

**Where the weight is.** J17A-17 (an assembly library's unit cost read past its stray
commas, where every other way in refuses it) and J17A-16 (a sample written over a
person's own job) are the two that can cost a builder a number or a file; J17A-2, 3,
7, 10, 14 and 19 are one family: a figure that is right and reads surer or more whole
than it is. Fixes follow below as they are made.

## Where the fixes to the seventeenth judgment stand (2026-11-12)

DECISIONS.md call 59 has the reason for each change of behaviour. 243 tests and all
seventeen command-line demonstrations pass on this tree. **None of these fixes has
been judged.**

| finding | state | what was done, or why not yet |
|---|---|---|
| J17A-2, 3 | fixed | a division names every line it leaves out, a line with a division and no cost type among them; "the sixteen divisions" carries its status and says how many it leaves out; "in no division" lists each line with its figure or why it has none |
| J17A-4, 6 | fixed | the workbook: unpriced lines keep their amount formula; every cost type and division is a formula that comes to "none" until it has a line; "In no division" is the job cost less the sixteen. Checked by `tools/workbook.py` (IF, OR, COUNTIFS added) and by typing a price into an unpriced line with it: Material, Division 9, the sixteen and the job cost all move |
| J17A-5 | fixed | an empty cost type has no note and no hours |
| J17A-7 | fixed | lines that ride on a question and have no price are counted on the question and at the head of the list |
| J17A-9 | fixed | a quantity is shown to four places when two would not multiply to its amount. `demos/d5.sh` and `d5w.sh` expectations moved: 461.19 lb to 461.1890, 576.49 to 576.4863, 419.26 SF to 419.2627, 251.56 SF to 251.5576; no amount moved |
| J17A-10 | fixed | the PM's lens: each division says what it leaves out and how sure it is; one with only unpriced work reads "no figure", not "none" |
| J17A-14 | fixed in part | what is priced on a thing whose identity is to be checked rides on that check. Whether the line itself should read Assumed is QUESTIONS.md Q21, with the working assumption |
| J17A-16 | fixed | `sample`, `house`, `sixteen`, `priced` refuse to write over a file. Not done: "(os error 2)" after a file that cannot be read |
| J17A-17 | fixed | an assembly's unit cost is read by the one price reader: "$1,23/LF" and "$1,2,3/SF" are refused |
| J17A-1, 8, 11, 12, 13, 15, 18, 19, 20 | open | software words in a hand-edited file's refusal; "a Wall cannot carry a height"; a price to a tenth of a cent shown to the cent on import; "1e9", escaped quotes; an order factor that does not fit how a material is bought; the usage's two steps for a stretch-out that each refuse; one bad row blamed on every row of its assembly; the estimate not saying which parts of the building have nothing priced; a run cut where it crosses another changing its clash |
| J17B-1, 2 | fixed | `building::enclosed`: every piece and every courtyard; a garden wall has both faces out and says so. Tested on a house with a courtyard, a shed and a garden wall on one level |
| J17B-3 to 10 | fixed | the reading goes by a line's head (call 59). Every line the judge wrote is a test case in `crates/world/src/finish.rs`. Added: vertical siding, panel siding, metal siding, concrete block; metal panels, metal shingles; awning, hopper; "6 over 1", "6/6", "9 lite"; "an overhead door". Not done: a shingle known only by its brand name still reads bare |
| J17B-11, 12, 13, 14 | fixed | a wainscot's height is read wherever it is written and not from a thickness; one as tall as its wall is the whole face and the siding above is said to be priced and not drawn; a second wainscot, a second color and a second unit are said; a wainscot on a partition is said to be priced and not drawn, by the report and the panel alike |
| J17B-15 | fixed in part | a gutter is read by its head ("Gutter apron flashing" and "Remove existing gutters" are not gutters; "eavestrough" is). An eave with no gutter is not listed: an eave needs none. SPEC 5.19 to be corrected |
| J17B-19 | fixed in part | "1 door", "2 levels". Not done: "the model is in foot", "wall 44 in the file", "wallstandardcase" |
| J17B-16, 17, 18, 20, 21, 22, 23, 24, 25 | open | an IFC's own per-value "Assumed" not read back; a material and a property set dropped without a word; roof slabs called floor slabs; internals in a hand-edited file's refusal; a level not taken by its name at the command line and measure names shown raw; price-list messages; a prior estimate's own total not checked against its lines; `plumbline roof` missing from the usage; the estimate CSV not read back as lines |

**Also changed in this stretch, and named:**

* **The words** (call 57, carried through): every sentence the model and the command
  line write says level, center, color, meters, cost type. The verb is `plumbline
  level` (`storey` still taken). A new level's path starts `level-`. The command-line
  demonstrations say the same; their figures did not move.
* **In 3D the roof is drawn when the top level alone is on show.** It was drawn only
  with every level showing, so the roof of a one-level job never appeared (seen in
  `demos/firstw.sh` picture 06).
* **The sheet's Estimate tab opens on the recap**, then By division, then the lines,
  then the suppliers. `demos/d5w.sh` was changed to match and passes (pictures in
  `demos/captured/d5/`, taken on this tree; 06 read: overhead and profit typed on
  their rows of the recap, the price under them, the sixteen divisions below).
* **Seen in that picture:** the roofing sample (`plumbline priced`) has every line in
  no division. It is the old roofing job and nobody gave its lines one; it should be
  Division 7 with its general conditions in Division 1.
* **The window demonstrations, rerun on 8c6978f before these fixes:** firstw, draww,
  buildw, fieldw, clashw, d4w, planw, bringw, d2 passed; d5w failed (the recap had
  moved under the divisions; fixed as above); house failed (the "Hot water" row of
  the left panel is off the bottom of the panel now that Bring in sits above it;
  the demo now wheels the panel to it, as a person would). **All of their pictures
  are from before these fixes and are stale until rerun.**
* **Disk:** the session's disk filled during a scratch build. Old judges' built
  programs, the scratch build and the build's incremental cache were deleted; no
  source and no evidence was.

## Seventeenth judgment, part C (2026-11-12): the fixes, judged

Judge on Opus, commit bbc72b0, command line and source only: every row marked fixed
was rerun from the first judges' commands and then attacked from the side. 19
findings: 1 breaks, 2 wrong money, 12 silent, 3 words, 1 doc. (Report returned as
text; set down here.)

**The claimed fixes:**

| finding | verdict | shown by |
|---|---|---|
| J17A-2, 3; J17A-5; J17A-7; J17A-9 (347 of 347 sample lines multiply) | holds | |
| J17A-4, 6 (the workbook) | holds in part | J17C-3, 4, 5, 6 |
| J17A-10 (the PM's lens) | holds in part | J17C-7, 8 |
| J17A-14 | holds as far as it claims; SPEC not brought up to date | J17C-17 |
| J17A-16 | holds for the four verbs; the export verbs beside them still destroy a job | J17C-1 |
| J17A-17 | holds; small differences in how a price is read remain | J17C-19 |
| J17B-1, 2 (outside faces) | holds in part | J17C-11, 12 |
| J17B-3 to 10 (the reading) | holds in part: every original line reads right; near variants do not | J17C-9, 10, 13, 14, 15, 16 |
| J17B-11 to 14 (wainscot, also priced) | holds in part | J17C-13, 14 |
| J17B-15; J17B-19 | holds in part | J17C-16; J17C-18 |
| the words | holds in part: an old job with storey paths loads, edits and exports; both verbs work | J17C-18 |

| | kind | finding |
|---|---|---|
| J17C-1 | breaks | `export prices` and `export assemblies` write over the job file itself when it is named as the output; `export estimate` writes over another job's file |
| J17C-2 | silent | a hole to cut that is still in question is on `clashes` as "to verify in the field" and is not on the verify-in-field list |
| J17C-3 | wrong money | in the workbook, typing in the cost type a line lacks leaves its money out of its division and in "In no division": the division cell reads "9 Finishes (in no total)" and the SUMIF looks for "9 Finishes" |
| J17C-4 | silent | a job with nothing priced yet shows Job cost $0.00 in the workbook; with overhead and profit entered and no price yet, their rows are the words "no figure" and the percentages are gone, so prices typed in never give a price |
| J17C-5 | silent | an unpriced line's amount formula is written with the figure 0, not nothing, and `tools/workbook.py` passes it because it takes 0 for nothing |
| J17C-6 | silent | after a price is typed into the workbook its line counts and "leaves out" notes, which are typed text, stay as they were; a line with a figure and no cost type is said to have "no figure" |
| J17C-7 | silent | a division with only unpriced work reads "none" on the estimate and in the workbook and "no figure" in the PM's lens: one word was fixed in one place |
| J17C-8 | wrong money | the PM lens's lines put a line that is in no total under its division with no mark, so the lens's own lines do not add to its own budget row |
| J17C-9 | silent | honest lines read bare under the head rule: "North wall, lap siding"; "Brick veneer and flashing"; "LABOR, LAP SIDING"; "Labor to install lap siding"; "Stones veneer"; "Trim-less lap siding"; "Galvalume roofing"; "Slates, Vermont gray"; "Reroof, asphalt shingles"; "Asphalt shingles and felt"; "Labor, casement window" |
| J17C-10 | silent | lines that are not a finish still dress: "Fire block, 2x4", "Siding mounting block", "Glass block" (concrete block); "Rebar lap splices" (lap siding); "Granite vanity top"; "Stacked stone veneer, interior fireplace wall"; "Picture frame molding" (a fixed window); "Starter shingles", "Hip and ridge shingles", "Tile battens", "Copper ridge cap" on a plane, the last ahead of "Slate roofing, gray" drawing a slate roof as copper |
| J17C-11 | silent | a closet or chase walled on four sides with no room placed in it is read as a courtyard and its walls face into it; a shed inside a courtyard reads as standing outside everything; a room placed in a courtyard to take off its paving makes the courtyard's walls partitions |
| J17C-12 | silent | outside is judged at a wall's midpoint: a wall that is the house's side for part of its length and a garden wall for the rest reads as one or the other over all of it |
| J17C-13 | silent | "Wainscot, stone, 36 in" reads bare (wainscot is not taken as a kind); "Lap siding above stone wainscot" becomes a lap siding wainscot under a bare wall |
| J17C-14 | words | "Stone wainscot 0 in" says no line gives a height; a wainscot taller than its wall leaves a fragment of a sentence and is not said to be taller; a second wainscot is not said to be a wainscot |
| J17C-15 | silent | a color is taken from what a line is laid over or set in: "Brick veneer with gray mortar" is gray brick; "Lap siding over black building paper" is black |
| J17C-16 | silent | "Gutter hangers", "Gutter end caps", "Gutter labor" read as a gutter |
| J17C-17 | doc | SPEC 5.16 still describes the dollars riding as before call 59 (5.19 was rewritten after this judge's commit) |
| J17C-18 | words | `show` prints "[Storey]"; the usage still offers the verb `class`; "4 floor slab" |
| J17C-19 | words | `import assemblies` refuses a bare "5" where `set ... unit_cost 5` takes it; "$/SF" says `"" is not a price`; "1.234,56" does not get the decimal-comma explanation |

**What held, beyond the table** (the judge's list): overhead and profit worked by hand
agree on the command line and in the workbook; the workbook's formulas use only IF, OR,
COUNTIFS, SUMIF, SUM and ROUND and Excel would work each; a price typed into an
unpriced line moves its cost type, its division, the sixteen, the job cost, overhead,
profit and price to the hand figures; an unpriced line riding on two clashes and its
own question is counted once; checking a thing's identity takes its dollars off the
list; the divisions agree on the command line, the CSV and the PM's lens for all
sixteen of the sample; outside faces are right on an L, a smaller upper level, two
courtyards, a party wall to an attached garage, a second building and a garden wall;
ten hostile assembly prices are each refused on their own row.

## A starter library of assemblies in all sixteen divisions. How its demonstration will be run (written before the script)

Task: the sixteen-division platform needs assemblies behind its divisions. An assembly
is everything priced on one thing, whatever divisions its lines fall in (a wall's
framing is Division 6, its siding 7, its gypsum board 9), and a thing carries one.
`library/sixteen-divisions.csv` is a library the program ships, in the form
`plumbline import assemblies` reads: 30 assemblies, 93 lines, some line in every one of
the sixteen divisions, a cost type and a division on every line, **and no price in
it**: every unit cost is "$?" and Unknown, "no price entered". The only figures in it
are how a thing is bought where that is arithmetic and not a price: 32 SF to a 4 ft by
8 ft sheet, 100 SF to the square, 81 SF to the cubic yard at 4 in thick (27 CF over
1/3 ft).

`demos/library.sh`, on a made-up house 24 ft by 16 ft with 9 ft walls, a partition at
12 ft, a 3 ft by 7 ft door and a 3 ft by 5 ft window in the south wall, a floor from
the walls' outline, a kitchen and a bath, a hip roof, a duct and a cable:

1. The library has no price in it (no "$" before a digit; every unit cost "$?/unit").
2. It comes into an empty job: 30 assemblies set, none refused.
3. Applied the way the window's Condition button applies them: the four outside walls
   "Exterior wall, 2x6, lap siding" (the north one "... over a stone wainscot"), the
   partition "Interior partition, 2x4", the floor "Slab on grade, 4 in", the rooms
   "Kitchen" and "Bath, tile floor", the planes "Asphalt shingle roof", the ridge, the
   hips, the eaves, the window, the door, the duct, the cable, and the job "General
   conditions".
4. The takeoff is there with no price on anything: south wall, 24 by 9 less a 21 SF
   door and a 15 SF window, lap siding 180.00 SF; its sheathing 216 SF at 32 to the
   sheet, 6.75, bought as 7 sheets; the slab's concrete 384 SF at 81 to the yard,
   4.74, bought as 5 CY. The job cost is "none", fourteen divisions say "no figure"
   (they have lines and no price) and Divisions 13 and 14 say "none" (no sauna, no
   elevator).
5. Three prices typed: siding $9.50/SF on the south wall, 180 × 9.50 = $1,710.00
   (Division 7); concrete $165/CY, 5 × 165 = $825.00 (Division 3); the permit $1,200.00
   (Division 1). Job cost $3,735.00, and each of the three divisions its figure, each
   still naming the lines it leaves out.
6. The house is dressed from the same lines: lap siding on the four outside walls,
   a stone wainscot on the north one, asphalt shingles on the planes, a gutter on
   each eave; the partition bare.

## Where the fixes to part C, and the rest of parts A and B, stand (2026-11-12)

250 tests and all eighteen command-line demonstrations pass on this tree
(`demos/library.sh` is new). **None of this has been judged.** The window
demonstrations have not been rerun on it: their pictures are stale.

| finding | state | what was done |
|---|---|---|
| J17C-1 | fixed | no export verb writes over a job: not the job it writes from, and not any file that loads as a job, whatever its name ends in |
| J17C-2 | fixed | a hole to cut that is still in question is on the verify-in-field list, with no dollars of its own (neither thing has to move), on the command line, the sheet and the PM's lens |
| J17C-3 | fixed | in the workbook a line with no cost type has an amount formula that comes to nothing until the cost type is typed in, and its division is written plainly: typing "Material" on such a line moves its $200.00 into Material, Division 9 and the sixteen together, and "In no division" stays what it was (replayed with the judge's job) |
| J17C-4 | fixed | Job cost and the sixteen read "none" until something has a figure; with both percentages entered, overhead, profit and price are formulas that read "no figure" until the job cost has one. The judge's job, Tile priced at $4.00 in the workbook: job cost $12.00, overhead $1.20, profit $0.66, price $13.86, as worked by hand |
| J17C-5 | fixed | a formula that comes to nothing is written with nothing (it was written with 0); `tools/workbook.py` no longer takes 0 for nothing, and works IF as a workbook does (only the side it takes) |
| J17C-6 | fixed in part | line counts are formulas; notes that were typed text now say "as written:" and a line with a figure and no cost type is said to have no cost type |
| J17C-7 | fixed | "none" is no work and "no figure" is work with no price yet, on the estimate, in the workbook and in the PM's lens alike |
| J17C-8 | fixed | the PM lens marks a line that is in no total beside its division, as the estimate CSV does |
| J17C-9, 10, 13, 15, 16 | fixed | the reading: a word for something else anywhere in what a line is and it is not a finish; a head that names none is refined by what follows its commas; "Labor to install", "Furnish and install" and the like are dropped from the front; "and flashing" is cut off; "block", "lap" and "picture" alone no longer name a finish; a color is read from the line's own words, not from what it is laid over. Every line in the judge's tables is a test case. Not done: "Trim-less lap siding" still reads bare |
| J17C-11 | fixed in part | a shed standing in a courtyard has outside faces of its own. Not changed, by decision: a loop of walls with no room in it is a courtyard, as the floor's outline takes it (one rule for the floor and the faces); a closet reads right once its room is named |
| J17C-12 | fixed | outside is judged in five places along a wall; one that bounds the house for part of its length and runs on outside says so, is dressed on both faces, and says to split it |
| J17C-14 | fixed in part | a wainscot taller than its wall is one whole sentence and says it is taller; a second wainscot is said to be a wainscot. Not done: "0 in" |
| J17C-17 | fixed | SPEC 5.16 and 5.19 describe what the program now does |
| J17C-18 | fixed | kinds are said in words everywhere at the command line ("[level]", "[roof plane]", "[assembly]"); the verb is `cost-type` (`class` still taken); "4 floor slabs" |
| J17C-19 | open | an assembly library's unit cost still wants its unit where `set` takes a bare figure |
| J17A-8, 13, 15, 16 (the rest), 19 | fixed | a wall's height refusal says how a height is given and measured; an order factor that would take the figure off a priced line is refused with the file unchanged; the stretch-out refusals and the usage say to give the quantity and its stretch-out together; a file that cannot be read is said without "(os error 2)"; the estimate says what is built and has nothing priced on it ("! nothing is priced on 1 thing that could be (1 wall)") |
| J17B-16, 17, 18, 19, 21 (part), 24 | fixed | a model this program wrote and marks Assumed comes in Assumed when it is said to be Known, and says so; property sets and materials not read are counted in the report; a roof's slab is a roof slab; "the model is in feet"; "wall #44 in the file"; a thing named by its name where its path is wanted is told its path ("ground" is the name of level-0002: give level-0002); `plumbline roof` is in the usage |
| J17A-1, 11, 12, 18, 20; J17B-20, 21 (raw measure names), 22, 23, 25 | open | as listed above |

**Changed in this stretch, and named:**

* **The priced sample house** (`plumbline sixteen`) prices its four hips ("Hip cap") and
  its 41 fittings ("Fitting, installed"), so nothing in it is left with no line: **392
  lines; job cost $390,534.08 to $394,072.08; price $463,954.49 to $468,157.63.**
* **The roofing sample** (`plumbline priced`) has its lines in Division 7 and its
  foreman in Division 1. It had none. No figure of it moved.
* **A starter library of assemblies** ships with the program
  (`library/sixteen-divisions.csv`, and inside the program: `plumbline import
  assemblies <job> starter`): 30 whole assemblies, 93 lines, a line in every one of the
  sixteen divisions, no price in it. `demos/library.sh` passes to the figures in the
  plan above, with one correction to the plan: a division's total that still leaves
  lines out is flagged ("~$1,200.00") and names them, as every such total is. A test
  holds that it comes into an empty job whole, goes back out byte for byte as the file
  in the repository, and has no dollar figure in it.
* The verify list gains every hole to cut that is still in question; `show` and the
  estimate say kinds in words; the estimate calls the job "the job".

## Pricing from the window with the starter assemblies. How its demonstration will be run (written before the script)

`demos/pricew.sh`. The made-up house of `demos/dressw.sh` (24 ft by 16 ft, 9 ft walls,
a door and a window in the south wall, a hip roof), with nothing priced and no
assembly in the job, opened in the window:

1. Under Bring in, **Starter assemblies** is clicked: the job has 30 assemblies.
2. The south wall is clicked on the model and **Assembly** pressed: the panel lists
   the five that go on a wall, by name, with their lines counted.
3. "Exterior wall, 2x6, lap siding" is clicked: the wall has 7 cost lines and is drawn
   in lap siding.
4. Assembly again, and "Exterior wall, 2x6, brick veneer" clicked: the lap siding line
   comes off (1 line), the two brick lines go on, the six lines the two assemblies
   share stay; the wall has 8 lines and is drawn in brick. The status line says what
   came off.
5. On the sheet, the brick veneer's unit cost is typed, 24: the south wall is 24 by 9
   less a 21 SF door and a 15 SF window, 180 SF; 180 × $24.00 = **$4,320.00**, in
   Division 4, Assumed (typed in the window).
6. Saved, and the saved job read back: the wall's assembly is the brick one; it has
   eight lines and none of them is "Lap siding"; "Brick veneer 180.00 SF $24.00/SF
   $4,320.00"; Division 4 Masonry $4,320.00; `plumbline finishes` says brick veneer on
   its south face.
7. Measured in the pictures: the south wall is the pale color of lap siding after
   step 3 and red after step 4.

## Pricing from the window with the starter assemblies. What was run and what it showed (2026-11-12)

* **Built** (DECISIONS.md call 60): `Job::change_assembly`; `plumbline apply ...
  --change`; in the window a **Starter assemblies** button under Bring in, and the
  **Assembly** button (it was "Condition") opens a list of the assemblies that go on
  what is selected, each with its line count, the one it has marked; a click puts it
  on or changes to it, the status line says what was added and what came off, and the
  sheet turns to its Estimate tab. The sheet's By division table says "no figure" and
  "2 lines to price" where a division has work with no price, and counts such a
  division as having work.
* **Changed, and named:** the Condition button applied the first assembly that fitted
  and is now a list; "condition" is "assembly" in every sentence (call 60); a selected
  wall with a pattern on it no longer takes the selection's blue glow (it turned brick
  violet); `apply` says a thing already has an assembly before it says anything else.
* **`demos/pricew.sh` passes** (pictures in `demos/captured/price/`, taken on this
  tree): 30 assemblies brought in with one click; the south wall given "Exterior wall,
  2x6, lap siding" (7 lines) and changed to "... brick veneer" (the status line: "2
  cost lines added. 1 cost line of the assembly it had came off: Lap siding."); 24
  typed on the brick's line of the sheet; the saved job has the eight lines in the
  order worked out (the six shared ones, then the two brick ones), "Brick veneer
  [Subcontract] 180.00 SF ~$24.00/SF ~$4,320.00", "4 Masonry ~$4,320.00 1 line" and
  the wall read as brick veneer on its south face. Measured in the pictures, the south
  wall is (198, 194, 186) bare, (190, 186, 177) in lap siding and (128, 85, 73) in
  brick.
* **Found by running it, and fixed:** the sheet stayed on its Building tab when the
  first lines arrived (it now turns to the Estimate); Ctrl+S sent by the test
  harness did not save (the demonstrations use F2, as the others do; the judges'
  harness was changed to F2 too, and whether Ctrl+S is lost only on this renderer is
  not known: **to check by hand**); the brick wall measured violet while selected
  (the glow, above).
* **Pictures read:** 03 (the list: five wall assemblies by name with 7, 8, 8, 7 and
  3 lines, under the buttons; the status line says where to pick); 05 (brick in
  running bond round the door and window, "Assembly: Exterior wall, 2×6, brick
  veneer" in the panel, what came off in the status line); 07 of the sheet (the recap,
  then the sixteen divisions with Masonry $4,320.00, then the lines under DIVISION 4
  MASONRY with the price in its field). **Seen and fixed after 07:** Divisions 6, 7
  and 9 read "none" beside Masonry though each had lines waiting for a price; they
  now read "no figure" and say how many lines are to price. Picture 07 is from
  before that fix and is stale.
* A test (`crates/cli/tests/cli.rs`): an assembly changed for another keeps the
  shared line's $720.00, loses the siding and its $1,710.00 and says so; refused
  without `--change` with the file unchanged; refused when it is already that, and
  when the assembly goes on something else. **251 tests and eighteen command-line
  demonstrations pass.**
* **Not judged.** The other window demonstrations are still to be rerun on this tree.

## Eighteenth judgment: launched (2026-11-12)

A judge on Opus, on commit 43acb60 as pushed, playing a residential estimator of
thirty years with no software background on a first day with the program: no manual,
no source, no demonstration scripts in its folder; only the two windows, driven
through the harness with a real pointer and keyboard, every picture read. Its task is
the release candidate's items 9 to 12: every word on every screen; a whole-house
takeoff from the USDA plan PDF, priced, using only what the screen says; the look
against the instrument standard and the dressed house; and breaking the money path.
Findings are recorded here the turn they arrive. No timed window demonstration is run
while it works.

## While the eighteenth judgment runs (2026-11-12)

* **J17A-18 fixed:** one bad row in an assembly is named for what is wrong with it;
  the other rows of that assembly say they are left out with it and which row is why
  (replayed with the judge's three rows: rows 2 and 3 "nothing wrong with this row",
  row 4 the comma in its price).
* **J17A-11 fixed:** a price is reported to the places it is held to, on a price list
  brought in and on a prior estimate compared: "$15.00/sheet becomes $12.345/sheet".
* **Built, not yet seen:** where an eave hangs past an outside wall a soffit closes it
  underneath, level from the foot of the fascia back to the wall's face (the gap stood
  open and dark in the dressed pictures). No picture of it has been taken or read: the
  window is not run while the judge is using the display. It is looked at in the next
  rerun of `demos/dressw.sh`, whose house has a 1 ft overhang.
* 220 tests of the model and the command line pass on this tree; the window's crate
  builds.
* **J17B-25 fixed:** an estimate written out as CSV and read back as a prior estimate
  loses nothing. What each line is on, its cost type, its division, how sure it is and
  its note each have a place in a prior line; no column of the program's own estimate
  is said to be "not read". **Changed behaviour:** a prior estimate's Notes column is
  now kept with each line, where it was named as a column not read
  (`crates/world/tests/tables.rs` and `demos/export.sh` expect the new behaviour).
* **J17B-23 fixed:** when a prior estimate ends in a total row and that total is not
  the sum of the lines read, it is said, with both figures and how far apart they are
  (replayed with made-up lines of $380.00 and $200.00 under a total of $600.00: "Total
  says $600.00, and the 2 lines read add to $580.00: $20.00 apart").
* `compare` reads SQFT and SF, LIN FT and LF, EACH and EA as one unit, so a prior line
  in another estimator's hand is compared and not reported as a different unit.
* 251 tests pass on this tree; `demos/export.sh` and `demos/imports.sh` pass.
* **J17A-20 fixed (call 61): a run cut where another crosses it clashes by as much as
  before the cut.** Worked by hand: an 8 in duct and a 2 in pipe through its middle
  are in each other by 4 in + 1 in = 5 in. Replayed with the judge's own commands on
  the judged program and on this one: before the cut both say 5.00 in at (10, 5, 8);
  after it the judged program says 1.00 in twice, at (9.9583, 5, 8) and (10.0417, 5,
  8), and this one says 5.00 in twice, both at (10, 5, 8). It is still two clashes,
  one with each piece of duct (the job holds two things); the head total counts each
  line once. A new test (`a_run_cut_where_another_crosses_it_still_clashes_by_as_much`)
  holds a round duct, a rectangular duct, a cut away from the crossing, and a corner,
  which is not carried on round. **Changed behaviour:** the depth and the place of a
  clash at a joint between two runs in one line.
* **J17B-21 fixed (call 61): a figure is named by its words.** `show`, `price`, the
  refusals that list what a thing has, a line's worked formula and the confirmation
  of `set` say "net area", "plan length", "unit cost"; kinds are said as words in the
  takeoff and in an assembly's "applies to". Either spelling is typed: `set <path>
  "unit cost"` and `--with "turn up=8 in"` are taken. Counted over `show`, `price` and
  `takeoff` of the two sample jobs: 503 joined names before, none after. **Changed
  behaviour:** every one of those lines of output; `check` now says "reads, its
  figures agree with what is drawn, and saving it back writes the same file".
  `demos/units.sh`, `demos/build.sh` and `demos/d2.sh` read the new words.
* **J17A-1 and J17B-20 fixed: a job file changed by hand is refused in a person's
  words.** "job-0001, its cost lines: it holds 7.5 where a whole number belongs"; "it
  says "Roofer", which is not one of: Material, Labor, ..."; "it has no
  "description", and needs one"; "the file stops short, as if part of it was cut off
  (line 240 of the file)". `check` on a file that reads but is not written the
  program's way says where: "They part at line 2: the file has ...; saving writes
  ...". A new test holds six hand changes and that none of the reader's own words
  comes through.
* 253 tests pass; all eighteen command-line demonstrations pass on this tree. The
  window's crate builds; its pictures are not retaken (the judge has the display).
* **J17C-14 closed:** a wainscot whose line gives no height to draw it to (none
  written, or "0 in") says so: "drawn to 3 ft, since its line gives no height to draw
  it to (one is written like 36 in or 3 ft)".
* **J17C-19 closed, by decision:** a price in an assembly library still says what it
  is for, because a library's line goes on things of more than one kind and its unit
  is not known until it does. What changed is what a refusal says: a bare "5" "does
  not say what it is for: a price in a library is dollars for a unit, like $4.25/SF";
  "$/SF" "has no figure: write the dollars, like $4.25/SF, or $?/SF where the price
  is not known yet"; "$4.25 per SF" is read as $4.25/SF.
* **Every kind of thing is said in words in the model's refusals** (47 of them said
  "a RoofPlane", "a Door", "a LowSlopeArea"): "plane-0003: a roof plane cannot carry
  a level", "is a door, but its type, type-0009, is a window's", "an eave", "a piece
  of equipment". **Changed behaviour:** those sentences; three tests expect the words.
* 253 tests pass; all eighteen command-line demonstrations pass.

## Eighteenth judgment: what it found (2026-11-12)

The judge (Opus, as a thirty-year estimator with no manual) ran sixteen sessions in the
window on 43acb60 and was stopped there: it was cut off once by a usage limit, and the
window it was judging is about to be rebuilt (call 62). It got as far as a level, the
USDA plan scaled from one dimension, ten walls, three doors and nine windows. Nothing
was priced. Its own figures: walls 130.33 LF against 130.34 by hand; scale read as
1/8" = 1'-0", matching the title block.

**Verdicts.** Item 9 (words): **fail**. Item 10 (whole-house takeoff from the screen
alone): **fail so far**. Item 11 (the look) and item 12 (break the money): **not
judged**; it did not get far enough.

| # | Kind | What it found |
|---|---|---|
| J18-1 | wrong money, silent | A tool picked a second time comes back already in hand with its old sizes and takes no typing: a 10 ft window, an 8 ft window and a 2'-8" door each went in at the size before. Openings on the front wall read 42.33 SF where 113.33 SF were drawn for; net wall face is about 69 SF over. Nothing said the typing was thrown away. 4 of 4 times. |
| J18-2 | stuck, silent | Door: width, Tab, height, Enter places nothing (Enter moves to Sill height and blanks it; the click on the wall does nothing). Three doors lost with no word why. All three values with commas, then Enter, works. |
| J18-3 | stuck, words | Add drawing wants a typed file name "as it lies from the job's folder" and there is no way to browse for the file. `usda-7213.pdf, 4` was refused; only `../plans/usda-7213.pdf, 4` worked. |
| J18-4 | words | Folder paths and a file name on screen: "Save writes it to /tmp/...", "house.json" in the top bar, "(../plans/usda-7213.pdf, page 4)" on every traced value in both windows. |
| J18-5 | words, stuck | "0" is refused as an elevation ("has no unit ... A laser has to be set to send its unit"). Zero is zero in any unit. |
| J18-6 | silent | A half-filled Add level form is dropped without a word when another button is clicked. |
| J18-7 | silent | A door has no kind (hollow core, solid core, bifold); its default height is 7 ft where residential doors are 6'-8"; "Sill height" on a door is a threshold. |
| J18-8 | silent, wrong money risk | The Building tab lists "wall 1" to "wall 10" with no type, thickness or place; 6 in exterior walls and 4 in partitions are in one total; Face is one side and does not say which. |
| J18-9 | look, stuck | Set scale throws away the zoom just made to pick the dimension's ends; a job saved in plan reopens in 3D, tilted over the scanned sheet. |
| J18-10 | words | "(0, 0) ft"; "Line up" does not say what it does; "nothing on it to snap to"; "Usda"; "Architecture 10"; "1 known" with no word of what; "30 values, the first: ..."; "Tool put down."; "Let go."; "WALL OR FLOOR". |

**What it was like to use, in the judge's words, shortened.** The Next steps list is
the only guide and only says which button to press; when something silently did
nothing there was no sign, and the only way to learn a 10 ft window was a 3 ft one was
to work the sheet's numbers by hand. The wheel zooms in fixed big steps; there is no
zoom box and Frame does not say what it frames. Right-drag, middle-drag and W A S D
"are video-game controls, not a plan-table habit". The size fields behave differently
depending on how they are reached; "a good program would always open the size fields
when a tool is picked, show the size it will place right on the pointer, and refuse to
place when the typing was thrown away". The right panel's To verify grows into piles.
Roughly 3 of 16 sessions went to looking only, one to finding the file, two to opening
sizes that did not take, one to a lost level.

**What held:** the scale from one dimension; the wall loop's length and face against
hand figures; openings subtracted correctly where the size took; the panel and the
sheet agreeing on every total seen; bad input got a red banner, never a crash.

None of these is fixed yet. They are folded into the rebuild planned below.

## Self-review, written cold (2026-11-12, call 62)

Patrick: "I still do not see how the UI functions are better." Looking at
`demos/captured/price/05-model.png` as a person who has never seen the program, he is
right, and the eighteenth judgment says the same from inside it.

**What is solid, and worth keeping exactly as it is.** The model and the money. One
job file; every value Known, Assumed or Unknown with where it came from; quantities
worked in one place; sixteen divisions on every line; a verify-in-field list sorted by
dollars; clash depth by real solid geometry; IFC4 out, validated by IfcOpenShell; a
workbook whose every amount is a live formula; 253 tests, 18 command-line
demonstrations, three independent judgments of the arithmetic that found no dollar
wrong after their fixes. The dressed house reads its finishes from what is priced.

**What is not good, plainly.**

1. **The window is a web form from fifteen years ago with a model in the middle.**
   Three docked white panels and two bars take 40% of the screen; the model gets a box
   about 1000 by 780. Twelve-pixel grey text, red sentences stacked down the right
   side, hard 3 px blue selection wire. Nothing has depth. I built it to a rule that
   said flat and dense, and judged it against that rule instead of against whether a
   person would want to use it.
2. **Moving round the model is crude.** The wheel steps the distance by a fixed
   factor toward the middle of the screen, with no momentum; orbit is right-drag about
   a fixed point; nothing eases; plan to 3D is a cut. The judge called them
   video-game controls and meant it unkindly: they are a bad game's.
3. **The tools throw typing away.** J18-1 and J18-2 are the worst things found in
   this program to date: a person typed a size, the program placed a different one,
   and said nothing. That is "silently wrong", the one thing the fixed rules exist to
   prevent, and it was in the path every takeoff goes through. My window
   demonstrations never caught it because each script types sizes the one way that
   works. A check that only walks the path its author knows is not a check.
4. **Adding a plan needs a typed path.** There is no file picker anywhere. For a
   program whose first step is "open the drawings", that is disqualifying.
5. **Layers barely exist.** "Show" has Architecture, Roof, See through and one status
   toggle, drawn as small squares. No level switch, no per-system switch for the
   ducts, pipes and wiring, no way to turn the finishes or the drawing off.
6. **A placed thing cannot be picked up and moved with the hand.** Move is a tool
   that takes typed distances or two clicks. Clash feedback is a tab on the other
   window. Patrick's vanity cannot be dragged along a wall while the clash lights and
   clears, which he says is the whole point.
7. **Nothing is exported from the window.** IFC, the workbook, CSV and the three
   lenses are command-line verbs. To someone in the window they do not exist.
8. **There are only fixtures and equipment, sized by typing three numbers.** No
   furniture or casework; nothing made from a manufacturer's sheet.
9. **The light is flat.** Matte everything, one soft shadow, grey void. It reads as a
   diagram of a house. The patterns on the walls are good; the place they stand in is
   not a place.
10. **The second window is a long grey page.** Its scrolling is the engine's raw wheel
    steps with no bar.
11. **Process.** Seventeen judgments went to the command line and the arithmetic, and
    one, cut short, to the window a person actually uses. The order of attention was
    wrong for a product a person has to want to open.

**What follows.** The plan below is the rebuild of the window: its feel, its look,
and the functions Patrick named. The model underneath does not change.

## The rebuild after call 62: what will be built and how each part will be shown (written before building)

In the order it is built. Each part has a demonstration that drives the real program;
the hand figures are here first.

**A. Items from a manufacturer's sheet, and furnishings** (the model; no window yet).
A cut sheet (PDF with words in it) is read into an *item*: a thing that can be placed
any number of times. What is read: its name, its maker, its model number, its width,
depth and height, its weight. Each value read is Assumed and its source is the sheet,
the page and the line as written; each value not found is Unknown and is asked for. A
third kind of placed thing joins fixtures and equipment: a *furnishing* (Division 12,
Furnishings: casework, furniture). `demos/spec.sh`, all sheets made up in the script:

1. *A vanity.* The sheet says: Harbor Vanity 36; Northfield Millwork; Model: NV-3621;
   Overall dimensions: 36" W x 21" D x 34-1/2" H; Shipping weight: 96 lb. The item is
   3 ft wide, 1.75 ft deep, 2.875 ft high (36/12, 21/12, 34.5/12), all Assumed.
2. *An air handler*, sizes on three lines (Width 21 in; Depth 24 in; Height 49 in):
   1.75 by 2 by 4.0833 ft, a piece of equipment.
3. *A sheet with no height* (30 in wide, 12 in deep): the item is made; its height is
   Unknown; placed, it has no shape, is not tested for clashes, and that is said. A
   height of 72 in typed from a tape makes it Known and it has a shape.
4. *Sizes with no letters* (24 x 18 x 30 in): read as width by depth by height in the
   order written, and the source says the order was assumed.
5. *Metric only* (600 x 450 x 850 mm): 23.622 by 17.7165 by 33.4646 in (each over
   25.4), and the source gives the millimetres as written.
6. *The vanity moved along a wall.* A water closet 1.5 ft wide stands with its middle
   at 5 ft east: it reaches from 4.25 to 5.75. The vanity, 3 ft wide, set with its
   middle at 7 ft, reaches from 5.5 to 8.5: the two are in each other by 5.75 - 5.5 =
   0.25 ft, **3.00 in**. Moved 6 in east it reaches from 6 to 9: clear by 3 in, no
   clash; asked for a 4 in clearance, they are 3.00 in apart.
7. *Out to BIM.* The IFC4 file carries the vanity as IfcFurniture and the air handler
   as it carries equipment now, and IfcOpenShell validates it.

**B. The window, rebuilt** (after the two surveys now running report: what Google
Earth, Cesium and the building tools do, with their numbers; and what the engine
gives). The demonstration is `demos/glassw.sh` and its pictures, read one by one:

1. The model fills the window. Panels are glass over it and can each be put away.
2. The wheel dives at what is under the pointer and eases out; a drag slides the
   ground with the hand; a double-click flies to a thing; Plan and 3D ease into each
   other. Checked by pictures taken mid-flight and by where the thing under the
   pointer is before and after a zoom (it does not move on screen).
3. Every level, system, the roof, the finishes, the drawing and the clashes has a
   switch; each is shown on and off.
4. A panel scrolls smoothly under the wheel, shows a bar, and the model does not move
   while it does.
5. The vanity of A.6 is picked up and dragged along its wall: red while it is in the
   water closet, clear when it is clear, and the estimate does not change (a vanity
   is priced each).
6. A drawing and a cut sheet are opened by picking the file from a list, not by
   typing where it lies (J18-3).
7. Export, in the window: BIM (IFC4), the workbook, CSV, the three lenses.
8. The tools: the size being placed is shown at the pointer; a tool picked again
   opens its fields again; typing is never thrown away (J18-1, J18-2).

**C. CLAUDE.md** folded to call 62, with a guide for whoever builds next.

## Items from a maker's sheet, and furnishings. What was run and what it showed (2026-11-12)

Part A of the rebuild. `demos/spec.sh` passes; every sheet in it is made up.

* **Read.** `plumbline_plans::words` reads the words of a PDF page line by line with
  the height of each line's letters (the PDF library already linked hands over each
  letter and where it is drawn; no new library). `plumbline_world::spec` reads those
  lines into what a sheet says and makes nothing up: a size is taken when the sheet
  writes it with a unit and says which way it runs, or writes three in a row.
* **The demonstration against the hand figures.** The vanity: 36 in, 21 in, 34 1/2 in,
  96 lb, maker and model, each with the line it was read from. The air handler from
  three lines: 21, 24, 49 in; no maker on the sheet, so maker is Unknown and asked
  for. The sheet with no height: the item is made, one is placed with no shape and
  says "its height is not known"; a tape reading of 72 in on the item gives the placed
  one its shape too. Three sizes with no letters say they were read in the order
  written. 600 x 450 x 850 mm is 23.622 by 17.7165 by 33.4646 in and the millimetres
  are kept in the source. A sheet whose name does not say what it is, a file that is
  not a PDF and a page that is not there are refused with the job unchanged. The
  vanity set at 7 ft is in the water closet by **3.00 in**; moved 6 in east it clears,
  and asked for a 4 in clearance they are **3.00 in apart**. The IFC4 file holds two
  IfcFurniture and IfcOpenShell finds no fault.
* **New in the model.** A kind of thing, *furnishing* (Division 12); an *item* (`pl::item`
  on a type): maker, model, width, depth, height, weight, each with its status and
  source; a placed thing remembers its item and follows it when the item is measured.
  Verbs: `import spec`, `place <level> <item>`, `place <level> furnishing`, `measure
  <item> width|depth|height`.
* **Changed behaviour:** a job file may now hold `pl::item` and a placed thing an
  `item`; files written before read as they did. The IFC export writes a furnishing as
  IfcFurniture.
* 262 tests pass; all nineteen command-line demonstrations pass; the port map accounts
  for the five new public items.
* **Not right.** A placed item is drawn as a box of its sizes. BIMCraft has
  parametric families (`family.rs`, with a library) that draw a thing as what it is;
  they are not ported. A scanned sheet has no words to read and is refused with that
  reason; nothing reads it by eye. Connections (a drain size, a voltage) are on many
  sheets and are not read. There is no window for any of this yet.

## The window rebuilt, step 1: panels float, and the view moves under the hand (2026-11-12)

Part B, first step. **Not finished and not judged.**

* **Moving round the model** (`scene.rs`, `orbit_input`; figures from call 63). The
  wheel closes on what is under the pointer, in 3D as it already did on plan: the eye
  and what it looks at both move toward that place by the same factor, so the place
  stays under the pointer (a notch is a tenth). A drag with the left button, when no
  tool is in hand, or with the middle button slides the ground: the place grabbed is
  put back under the pointer every frame by a ray to its level. A drag with the right
  button turns the view about the place under the pointer at the press, not about
  the middle of the screen. Let go while moving and the slide or the turn carries on
  and dies away as e to the minus t over 0.4 s. A double-click flies to the place
  clicked. A drag is told from a click by how far it went, so sliding the view does
  not select or let go of anything. "Is the pointer over the model" is now asked of
  what the pointer is over, not of fixed rectangles, so a panel can move or be put
  away.
* **The panels** (`theme.rs`, a block of named figures at the top; `panels.rs`
  `docked`). Every panel floats 10 px clear of the window's edge and of the next
  panel, with 16 px corners, a light edge, a thick white tint and two shadows, near
  and far. Buttons, fields, keys and cards are round-cornered and sit on the glass a
  little whiter than it.
* **Run:** `demos/pricew.sh` passes on this tree (controls are still found by name).
  One picture is kept, `demos/captured/glass/step1-floating-panels.png`.
* **What that picture shows that is not right.** Over a scene that is nearly white
  the panels read as white cards: there is nothing behind the glass for it to be
  glass over, and nothing is blurred. The tool bar's right end (Move, Delete) is cut
  off under the right panel. The left panel is a tall sheet with half of it empty.
  The Show list is still small squares, not switches. The selected wall is still a
  heavy blue wire. The right panel is still a column of red sentences. The momentum
  and the zoom about the pointer have been built and compiled, and **not yet seen**:
  no picture has been taken mid-flight and no before-and-after of the place under
  the pointer has been measured.
* **Not rerun:** every other window demonstration. Five of them click at fixed dots
  on the screen (`buildw`, `clashw`, `draww`, `d2`, `d5w`) and the left button's drag
  now slides the view where it used to do nothing: they are expected to need work.

## The window rebuilt, step 2: a place to stand, and switches (2026-11-12)

* **A place.** In 3D the building stands on a warm grey yard under a sky that is pale
  at the horizon and soft blue overhead (a dome round the eye that nothing lights;
  QUESTIONS.md Q22 has the question, `theme::sky_horizon` and `sky_overhead` are the
  two figures). On plan the ground is still paper and there is no sky. The light is
  unchanged, so the colours the demonstrations measure on the walls are unchanged
  (`demos/pricew.sh` measures the bare wall, the lap siding and the brick and finds
  each as before).
* **Switches.** Each row under Show ends in a switch, blue with its knob to the right
  when on, grey with its knob to the left when off, and starts with a round spot of
  the colour that layer is drawn in. The rows keep their names ("Show Roof"), so the
  scripts that click them still find them.
* **The panels.** The left panel is as tall as what is in it. The tool bar's tiles are
  narrower and round-cornered and all sixteen fit between the side panels.
* **Run:** `demos/pricew.sh` passes. Picture: `demos/captured/glass/step2-sky-switches.png`.
* **Read as a person, what is still not right:** there is no blur behind the glass;
  the selected wall is a heavy blue wire; the right panel is a column of red
  sentences and small square Check and Measure buttons; levels and systems have no
  switch of their own yet; no panel can be put away; the sheet window is untouched.

## The window rebuilt, step 3: how it moves, shown by measurement (2026-11-12)

`demos/glassw.sh` passes. It opens a made-up house, drives the real program with a
real pointer, and follows the door (the one brown thing in the picture) from picture
to picture. What it measured on the last run, against the hand figures:

| What | By hand | Measured |
|---|---|---|
| the roof's switch | off, then on | the roof's blue grey at (900, 330): there, gone, back |
| three notches of the wheel at the door | on this display a notch arrives as two lines (traced), so six lines: 1 / 0.9^6 = 1.88 times as wide | 55 dots, then 103: 1.87 times |
| the place under the pointer stays under it | as far along the door as before | 0.49 of the way along, then 0.50 |
| the wheel over the right panel | the model does not move | the door's ends at (753, 855), then (753, 856) |
| a turn with the right button, 150 dots | about the door: it stays at (805, 640) and is seen from round the side | still at (805, 640); 104 dots wide, then 136 |
| a drag with the left button | the place grabbed goes with the hand to (1005, 560) | the door is at (1005, 560) and no longer at (805, 640); nothing was selected |
| a double-click on the door | flown to: in the middle of the window, nearer | at (800, 480); 96 dots wide, then 242 |
| a flick of 100 dots, let go moving | carries on past where the hand let go | the place grabbed is 85 dots past it |
| looking round changes nothing | the job file is the same bytes | it is |

**What writing the demonstration found, each fixed:**

* A pointer that is *put* somewhere (the test rig here; a tablet or a remote desktop
  in life) sends no "moved by" at all. The first version read "moved by": the turn
  turned nothing and a slide was taken for a click and selected the door. Movement is
  now read from where the pointer is.
* On a slow frame the last movement before the button came up was dropped, so a
  flick never carried. The frame in which the button is let go still counts.
* A double-click was timed against the frame clock and missed when frames were slow.
  It is now half a second, or two presses seen in one frame.
* My own hand figure for the wheel was wrong (three notches, not six lines): the
  program was right and the trace said why.

**Not known:** what one notch of a real mouse sends on a real display (one line is
expected; the virtual display sends two). `PLUMBLINE_TRACE=1` prints what the wheel
sent, for whoever has a mouse. Easing between plan and 3D has a step in scale at the
moment the perspective is taken out (the plan shows 0.62 of the distance, the lens
0.73). **Not rerun:** the other window demonstrations.

## A thing picked up and moved, with its clash live. How it will be shown (written before building)

`demos/carryw.sh`. The job of `demos/spec.sh` step 7, made at the command line: a
water closet 1.5 ft wide with its middle at 5 ft east, and the vanity made from its
sheet (3 ft wide) set with its middle at 7 ft: in each other by 5.75 - 5.5 = 0.25 ft,
**3.00 in**. In the window:

1. Opened: the clash is marked on the model without asking for it (the Clashes
   switch is on when there is one to see), and selecting the vanity says what it is in.
2. The vanity is selected with a click, then picked up (the button goes down on it)
   and dragged to the right. While it is in the water closet it is drawn red and the
   line along the bottom says by how much; a picture is taken with the button still
   down.
3. Dragged on until it is clear and let go. It is drawn in its own colour, the line
   says it is clear, and the price has not changed (nothing is priced on where a
   vanity stands).
4. Saved, and read back at the command line: the vanity's middle is east of 7.25 ft
   (it has gone more than the 3 in it was in by), `plumbline clashes` finds nothing
   in the same space, where it stands is Assumed and its source says it was moved by
   hand in the model.
5. Undo, once: it is back at 7 ft and the clash is back at 3.00 in. The whole drag is
   one step.

## A thing picked up and moved, with its clash live. What was run and what it showed (2026-11-12)

`demos/carryw.sh` passes; its five pictures are in `demos/captured/carry` and each was
read.

* **In the window.** The vanity (wood) stands into the water closet (white). Clicked,
  it is selected and drawn red, because it is in something. The left button down on
  it picks it up: it goes with the hand over the floor, the place grabbed staying
  under the pointer, and the Clashes switch comes on. A tenth of a foot on, with the
  button still down (picture 03), it is still red, the clash is marked, and the line
  across the top and along the bottom says "Harbor Vanity 36: in the same space as
  Water closet by 1 11/16 in. Keep going, or let go and it stands here." (3.00 in
  less the 0.108 ft it had gone, 1.30 in, is 1.70 in: 1 11/16.) Carried on and let
  go, it is its own colour and the line says it is clear.
* **Saved and read back.** `plumbline clashes`: nothing in the same space. The
  vanity's middle is at 9.05 ft east and 1.03 ft north; where it stands is Assumed
  and its source reads "set by eye; moved (moved by hand in the model)"; its size is
  still its sheet's.
* **Undo, once:** it is at (7, 0.875) again, set by eye, and the clash is back at
  3.00 in. The whole carry is one step.
* **Changed from the plan above:** the clash is marked from the moment a thing is
  picked up, not from opening (the Clashes switch stays off until then, so the
  scripts that turn it on still turn it on).
* **Also changed:** a furnishing with nothing priced on it is drawn in a light oak, so
  it reads apart from a white fixture; a weight is shown once with its unit (it read
  "96 LB LB").
* **Not right.** A carried thing does not snap to the wall it backs onto, does not
  turn, and is carried in 3D only (on plan the Move tool still takes two clicks). It
  is a box. It can be carried through a wall without a word beyond the hole it would
  cut. Nothing here has been judged by anyone but me.

## A maker's sheet, an item set down, and everything written out, all in the window. How it will be shown (written before the script)

`demos/itemsw.sh`. An empty job with a level and a slab 12 ft by 8 ft; beside it, the
made-up vanity sheet of `demos/spec.sh` (36 in wide, 21 in deep, 34 1/2 in high, 96 lb).

1. Bring in, Maker's sheet, the sheet's name: the item is made and listed under
   Items with its kind and its size: furnishing, 3' 0" x 1' 9" x 2' 10 1/2".
2. Its row under Items, then a click on the slab: one is set down there and selected.
3. Write out: BIM model (IFC), Estimate workbook, Estimate (CSV), For suppliers, For
   subcontractors, For the PM. Six files, beside the job, each named for the job.
4. Saved. Read back: the job holds the item with its three sizes in inches, each
   Assumed and sourced to its line of the sheet; one furnishing stands on the slab,
   3 by 1.75 by 2.875 ft, where it stands Assumed and "set down by hand in the
   model"; the IFC file holds one IfcFurniture named Harbor Vanity 36 and IfcOpenShell
   finds no fault in it; the workbook opens and its sheets are the estimate's; the
   CSV has its headings; the three lenses are there.

## A maker's sheet, an item set down, and everything written out, in the window. What was run and what it showed (2026-11-12)

`demos/itemsw.sh` passes; its pictures are in `demos/captured/items` and were read.

* **Maker's sheet** (Bring in). The sheet's name is typed (or the file dropped); the
  item is made by the same reader the command line uses and the line across the
  screen says what was read and what was not on the sheet. It is listed under
  **Items** with its kind and size: "Harbor Vanity 36: furnishing, 3' 0" x 1' 9" x
  2' 10 1/2"". If a sheet's name does not say what it is, the kind is typed after a
  comma.
* **Set down.** A click on the item's row takes it in hand; a click on the slab sets
  one down there, selected. Read back from the saved job: at (5.237, 3.017) ft, 3 by
  1.75 by 2.875 ft, where it stands Assumed and "set down by hand in the model".
* **Write out** (a new section of the left panel): BIM model (IFC), Estimate
  workbook, Estimate (CSV), For suppliers, For subcontractors, For the PM. Each
  writes beside the job under the job's own name ("job.ifc", "job estimate.xlsx",
  ...) and says what it wrote: "Wrote job.ifc, beside the job: the building as a BIM
  model (IFC4, lengths in feet). In it: 1 floor, 1 furnishing, 1 level." The BIM
  file holds one IfcFurniture named for the vanity and IfcOpenShell finds no fault;
  the workbook opens with its Estimate, Recap and Divisions sheets; the CSV has its
  headings; the three lenses open. Nothing is written over a job.
* **One writer.** The workbook is made by `plumbline_sheets::workbook`, which the
  command line now calls too (it had its own copy); `demos/export.sh` passes as before.
* **Not right.** The file's name is still typed, not picked from a list (J18-3 stands);
  a sheet is read from its first page only; an item cannot be turned or renamed in the
  window; the Write out names say "job" because the job file is job.json, which is a
  file's name on screen (J18-4 stands).

## A tool shows the sizes it will place (J18-1, J18-2). How it will be shown (written before the script)

`demos/toolsw.sh` walks what the estimator walked. A made-up wall 24 ft long. In the
window, on plan:

1. Door: its fields open with a height of 6' 8" and a sill of 0 ft already in them;
   the width `3' 6"` is typed, then Enter (the keys that lost three doors in the
   eighteenth judgment). A click on the wall at 6 ft: a door 3.5 ft wide and
   6.6667 ft high.
2. Door again: its fields open showing 3' 6", 6' 8", 0 ft. Before anything is typed,
   a click on the wall at 12 ft: nothing is placed and the line says the door's sizes
   are still open.
3. `2' 8"` typed over the width, Enter, a click at 18 ft: a second door, 2.6667 ft
   wide (the one that went in at the size before, J18-1).
4. Window, `10 ft, 5 ft, 2 ft 6 in`, a click at 3 ft on the north wall... is not in
   this script: one wall keeps it short. Windows go through the same fields.
5. Saved and read back: two doors, 3.5 ft and 2.6667 ft wide, each 6.6667 ft high;
   the wall's openings are 3.5 x 6.6667 + 2.6667 x 6.6667 = 23.33 + 17.78 = 41.11 SF.

## A tool shows the sizes it will place (J18-1, J18-2). What was run and what it showed (2026-11-12)

`demos/toolsw.sh` passes; its five pictures are in `demos/captured/tools` and were read.
Call 64 has the decision.

* Door: its fields open with 6' 8" and 0 ft in them. `3' 6"` and Enter: in hand; a
  click at 6 ft places a door 3.5 ft by 6.6667 ft. (In the eighteenth judgment these
  keys lost three doors.)
* Door again: its fields show 3' 6", 6' 8", 0 ft. A click on the wall then places
  nothing, and the line across the top and along the bottom reads "Nothing was
  placed: the door's sizes are still open. Enter takes it in hand; then click."
* `2' 8"` typed over the width, Enter, a click at 18 ft: a door 2.6667 ft wide. (In
  the judgment this one went in at the size before.)
* Read back from the saved job: two doors, at 6 ft and 18 ft, each the size that was
  on screen; the wall's openings 41.11 SF, as worked by hand.
* **Changed behaviour:** picking a tool that has sizes opens them every time; a door's
  height starts at 6' 8" (it was a hint of 7 ft); typing in a filled field replaces
  what is there. The kit's `tool` presses Enter for a tool picked with nothing new to
  type.
* **Also fixed, seen in these pictures:** on plan the ground stayed the yard's grey
  when the view changed without anything being redrawn. It is paper on plan again.
* **Still open from the eighteenth judgment:** J18-3 (a file is named by typing),
  J18-4 (folder and file names on screen), J18-5 ("0" refused as an elevation), J18-6
  (a half-filled level form dropped in silence), J18-7 in part (a door still has no
  kind), J18-8 (the Building tab's walls have no type, thickness or place), J18-9,
  J18-10.

## Nineteenth judgment: launched (2026-11-12)

A judge on Opus, on commit 14fc994 as pushed, briefed with Patrick's own words from
call 62 and asked one question above the rest: would he now see it? It drives the
window with a real pointer on the heads the rebuild claims: moving round the model,
the look, switches and scrolling, a maker's sheet made into an item, a thing picked up
and carried, writing out, typed sizes, and anything that goes silently wrong. It keeps
notes as it goes. Findings are recorded here the turn they arrive. No window
demonstration of mine is run while it works.

## While the nineteenth judgment runs (2026-11-12)

* **VISUAL.md rewritten** to call 62's standard: glass over the model, how it moves,
  the place the building stands in, the house dressed, the plan, the estimate window.
  It names four lines the window is known to fail today: the panels are tinted and
  not blurred (A2); a placed thing is a box, not what it is (D6); the estimate window
  has not been touched (F4); and, under B, plan and 3D still step in size where one
  becomes the other, panels do not scroll smoothly or show where they are, and levels,
  systems and panels have no switch.
* **J18-5 fixed:** "0" is taken as a length with no unit (nothing is nothing in any
  unit). Any other number with no unit is still refused.
* **J18-4 in part:** a new job's first line no longer shows a folder path ("Save
  keeps it as house."). The job file's name in the top bar, and the path in each
  traced value's source, stand.
* 262 tests and all nineteen command-line demonstrations pass on this tree.

## Where the rebuild stands against Patrick's words (2026-11-12)

| He asked for | State | Shown by |
|---|---|---|
| "scroll mechanics" (the model) | built: the wheel closes on the pointer, measured | `demos/glassw.sh` |
| "scroll mechanics" (the panels) | **not built**: the wheel over a panel scrolls it alone (measured), but in the engine's raw steps, with no bar | |
| "toggle on and off" | part: a switch on every row under Show; none for a level, a system or a panel | `demos/glassw.sh` (the roof) |
| "glassy overlays" | part: panels float, round, translucent, shadowed; **nothing is blurred behind them** | `demos/captured/glass` |
| "Google Earth feel" | built and measured: grab the ground, turn about the pointer, momentum, double-click to fly; plan to 3D eases but steps in size | `demos/glassw.sh` |
| "export to BIM files" | built, in the window: IFC4, validated; with the workbook, CSV and three lenses | `demos/itemsw.sh` |
| "create items by spec sheet uploads" | built, command line and window: read from a PDF's words; a scan is refused; the file is named by typing, not picked | `demos/spec.sh`, `demos/itemsw.sh` |
| "mechanicals, furniture" | built: equipment, fixtures and a new kind, furnishing; each drawn as a box | |
| "moving a bathroom vanity around" | built: picked up and carried, red while it is in another, one step to undo; no snap to a wall, no turning | `demos/carryw.sh` |
| "an Apple product with windows depth" | part: see glass; the estimate window is untouched | |
| "a hyper realistic video game" | **mostly not built**: a yard and a sky; the light, the materials and the placed things are as they were | |
| "self review" | written | "Self-review, written cold", above |
| "instructions to Opus 5.5" | written | CLAUDE.md, "For whoever builds next" |

**Not rerun on this tree:** `firstw`, `dressw`, `pricew` (last passed at step 2),
`draww`, `buildw`, `fieldw`, `clashw`, `d4w`, `d5w`, `planw`, `bringw`, `house`, `d2`.
Their pictures in `demos/captured` are of the old window and are stale.

## The window rebuilt, step 4: the glass is frosted (2026-11-12)

* **Frosted glass** (`crates/app/src/glass.rs`, `frost.wesl`). The engine has no way to
  blur what is behind a panel, so the scene is drawn a second time, a sixth of the
  window's size each way, into a picture; a panel's fill is a material that looks at
  that picture where the panel is, at thirteen places round each dot, and lays the
  glass's tint over it (60% thick; `theme::FROST_THICK`). Every panel made by
  `docked` is frosted. With `PLUMBLINE_GRAPHICS=plain` there is no second picture and
  a panel is its tint, as before.
* **The views have a cluster of their own.** 3D, Plan, Walk and Frame float over the
  model's top right. They were in the top bar, which they crowded until its tally was
  cut off ("247 unkno"); the bar now holds the job, Save, Undo, Redo, the price, what
  is to verify, the tally and Saved, whole. The job file's name is no longer in the
  bar (J18-4). The four buttons keep their names for the scripts.
* **Seen:** one picture of the priced whole house, `demos/captured/glass/step4-frosted.png`,
  opened and read: the left panel carries the blurred ground, the top bar and the
  views cluster the blurred sky, the house reads through nothing. No error from the
  shader in the program's log.
* **Not measured:** what the second picture costs a frame. It is drawn with shadows,
  which it does not need. **Not done:** the picture is made once at the window's
  first size and does not follow a window that is resized. **Not rerun:** any
  demonstration; the judge has the display. VISUAL.md's note against line A2 ("tinted
  and not blurred") is now out of date for a panel and still true of the message
  shown over the model and of the estimate window.

## Built while the nineteenth judgment runs, and not yet seen (2026-11-12)

Each of these compiles and has not been looked at in a picture: the judge has the
display. They are looked at, and `demos/glassw.sh` rerun, when it is free.

* **Panels scroll smoothly and show where they are.** The wheel sets where a panel is
  going and the panel eases there (all the way in one frame where frames are slow,
  so a script that wheels and then clicks is not chasing a moving row). Each
  scrolling panel, and the estimate window, has a thumb at its right edge, as long
  against the panel as what is seen is against the whole, and absent when the whole
  is seen.
* **Plan and 3D meet with no step in size.** As the eye comes up to overhead it also
  comes in, to where the lens frames the ground exactly as the plan will; the
  perspective is taken out only when the eye is all but there. A turn in 3D stops
  short of where this begins. The ray under the pointer uses the same stand-off, so
  the wheel and the drag stay true.
* **What is selected is outlined finely** (2.5 dots; it was a 4.5 dot wire).
* **A drag is told from a click by the pointer's own account** (its drag events),
  which comes before the click that ends it even when press, drag and release land
  in one slow frame. From the judge's notes so far (J19-11): a quick flick that began
  on a wall slid the view and also selected the wall.
* **More built and not yet seen** (same reason): a switch on each level's row and on
  each system's row (what stands on a level, or runs on a system, is drawn or not;
  the row itself still selects); the Building tab says each wall's type under its
  name and heads its columns "Face, one side" and "Net, one side" (J18-8, in part);
  hours on the recap to two places (J19-4, from the judge's notes); dollars written
  with their sign first in the panels, "$4,732.40" not "4,732.40 $" (J19-16).
* **J18-8, the rest: a level's walls are added up by their type**, in the model
  (`StoreyTotals.walls_by_type`), so the command line and the estimate window say the
  same sums. A test with hand figures: outside walls 24 ft by 16 ft round at 9 ft are
  80 LF and 720 SF; one 16 ft partition is 16 LF and 144 SF; the level's 96 LF and
  864 SF are the two added. On the sample house `plumbline takeoff` now reads, under
  the first floor's 210.00 LF: "of them Outside wall, 6 in: 4 walls, 136.00 LF ...
  Partition, 4 1/2 in: 3 walls, 64.00 LF ... Wet wall, 6 in: 1 wall, 10.00 LF"
  (136 + 64 + 10 = 210). The estimate window's Building tab has the same rows under
  "Walls, total" (built, not yet seen). **Changed behaviour:** `takeoff` prints those
  lines where a level has more than one type of wall; `demos/export.sh` leaves them
  out of its read-back comparison, because a building model is written with each
  wall's thickness and not its type (which the export already says).
* 263 tests pass; all nineteen command-line demonstrations pass.

## Fixed from the nineteenth judge's notes, ahead of its report (2026-11-12)

The judge writes each finding down as it has it; these were fixed from the notes while
it worked on. The window ones are built and **not yet seen**.

| Note | What it found | What was done |
|---|---|---|
| J19-11 | a quick flick that began on a wall slid the view and also selected the wall | a drag is told from a click by the pointer's own drag events, which come before the click even inside one slow frame |
| J19-25, 26 | an item with no height, set down: nothing appeared, the line said "Nothing selected", and nothing said it was not tested for clashes | setting one down says so at that moment ("it cannot be drawn or tested for clashes yet: its height is not known ... measure what is missing; this one will follow"); the click that sets it down no longer lets go of the selection |
| J19-27 | a placed thing's size in decimal feet ("3 x 1.75 x 2.875 ft") where the Items list said 3' 0" x 1' 9" x 2' 10 1/2" | one way: as a tape reads it, in the panel and in `show` |
| J19-28 | the source of a carried thing read "set down by hand in the model; moved (moved by hand in the model)"; the clash marks stayed on over the whole house after a carry | the source reads "...; moved (by hand in the model)"; the marks go back to how they were when the thing is let go clear |
| J19-23 | a weight not on the sheet was absent from the saved item, where a size not on the sheet was Unknown | Unknown, with "weight is not on <sheet>, page 1" |
| J19-4 | hours to four places on the recap | two |
| J19-16 | "4,732.40 $" | "$4,732.40" |
| J19-1 | the top bar cut off at "247 unkno" | fixed in step 4 (seen) |
| J19-3, 8, 13 | panels read as flat grey cards, and the model's labels showed through them | frosted in step 4 (seen): a panel shows what is behind it only as blurred colour |

**Changed behaviour:** `show` writes a placed thing's size as feet and inches; a moved
thing's source ends "moved (by hand in the model)" (`demos/carryw.sh` expects it).

* **J18-3, built and not yet seen: a file is picked from a list.** Wherever the window
  asks for a file (Add drawing; each kind under Bring in), the files that can be
  picked are listed under the field: those in the job's folder to begin with, with a
  row to go up one folder and a row for each folder to go into. A file's row puts its
  name in the field as it lies from the job's folder (a drawing's with ", 1" for its
  page, to be changed for another page); Enter then does what it did. The name can
  still be typed and the file can still be dropped on the window.
* **J18-6, built and not yet seen:** a half-filled entry let go for something else is
  said with whatever is said next: "The level was not added: its fields were let go."
  (and the same for a drawing's file, a file being brought in, or any value half
  typed). A tool's sizes are not said to be lost: they are kept as last taken in hand.
* **From the judge's notes J19-29, 30, 32, built and not yet seen:** a thing can be
  picked up and carried on plan as well as in 3D (on plan a press on it slid the
  whole plan and said nothing); and in 3D, when the pointer is on something that
  stands between the eye and the floor (a wall's face), the thing goes to where the
  hand is, at the foot of what it is on, not to the floor far behind it (carried with
  the pointer on the west wall, the vanity had gone 21 ft east and 14 ft north of the
  hand, inside the house).

## A placed thing drawn as what it is: ported from BIMCraft (2026-11-12)

VISUAL.md line D6 ("a vanity is a vanity ... not a box of its sizes") failed, and the
nineteenth judge's notes say the same (J19-27: "a plain blue-outlined pale box - not a
vanity"). BIMCraft's library of families has the shapes: each is a short list of
blocks. Ten are ported, block for block and in the same order, in
`crates/world/src/building/forms.rs`: base cabinet (a plinth set back for toes, a
carcass, a worktop over the front), wardrobe, WC (cistern and bowl), washbasin on its
pedestal, bathtub, table, chair, bed, sofa, desk. PORTMAP.md has the five new rows
and NOTICE names the file.

* **Which thing gets which form** is read from its name, the earliest word that names
  one: "Harbor Vanity 36" is a base cabinet, "Water closet" a WC, "Bookcase BK-30" a
  wardrobe. A name that names none ("AH-24 Air Handler") is still the box it is.
* **What is drawn is inside what is tested.** A test holds, for every form at five
  sizes from a few inches to several feet, that every block is inside the thing's box
  and that together they reach all six of its sides. The box is still what is tested
  for clashes and what is outlined when the thing is selected.
* **Checked against the source:** the base cabinet at BIMCraft's own 1800 by 600 by
  900 mm gives BIMCraft's three blocks to the millimetre (plinth 100 high, 60 back;
  carcass 100 to 860, 20 back; worktop 40 thick over all).
* **Built, not yet seen** in a picture: the scene draws these (`scene.rs`), timber in
  oak, the worktop in pale stone, porcelain white, fabric grey-blue.
* **Not right, known before looking:** a form's back is the thing's north side unless
  it has been turned, and nothing turns a thing in the window yet, so a cistern can
  face the room. Parameters, types and connectors of BIMCraft's families are not
  ported. Equipment has no forms (BIMCraft's library has none for it either).
* 266 tests pass.

## Seen, in one picture of the vanity job (2026-11-12)

`demos/captured/glass/step5-things-as-what-they-are.png`, opened and read. Of what was
"built, not yet seen":

* **Seen and right:** the water closet is a cistern and a bowl with a round front; the
  vanity is a cabinet under a worktop (red in this picture: it is selected and stands
  3 in into the water closet); the selection's outline is fine; the level's row has
  its switch; the left panel has its thumb at its right edge; the size reads 3' 0" ×
  1' 9" × 2' 10 1/2"; the panels are frosted; the views are in their cluster.
* **Still not seen:** the panel easing under the wheel (a still picture cannot show
  it); plan meeting 3D with no step; a system's switch; the Building tab's walls by
  type; a flick that begins on a wall; a shapeless item set down; the file list; a
  half-filled form let go; carrying on plan and against a wall. These wait for the
  display.

* **The estimate window, seen** (`demos/captured/glass/step6-estimate-window.png`): its
  tables are white round-cornered sheets with a slight shadow, its fields are round,
  hours read "480.00 hr", and it has its thumb at the right edge. Its heading, its
  tabs and its foot are still flat strips (VISUAL.md F4 says so).

## What is next, and why it has not been started (2026-11-12)

**The light and the materials** ("a hyper realistic video game"; VISUAL.md C1, C3). The
window is still lit as it was before call 62: one sun, a fill, flat light from all
round, no reflections, colours written out as they are worked. That is the largest
thing Patrick asked for that is still not there. The plan, from the engine survey
(call 63):

1. Light from the sky: a generated environment light, so glass reflects the sky and
   shade is blue, not grey. It needs compute shaders; the ambient occlusion already
   in use needs them too and runs on the software renderer, so it should.
2. A wide-range picture and a film-like curve (HDR, then AgX or Tony McMapface), so
   a sunlit wall is bright and its shade is still a colour.
3. Soft shadows and contact shadows; the sun a little warm, the sky's light a little
   cool.
4. Glass that reflects; metal as a satin; a ground with some grain to it.

**Why not blind:** every one of these changes the colour of every surface, and six
demonstrations measure colours (`dressw`, `pricew`, `glassw`, `carryw`, and two that
read the plan's paper). Each measured colour would be re-derived and its check
re-proved red, one at a time, with the display. It is the next thing to do when the
display is free and the judge's findings are in.

**After that, in order:** rerun and repair the thirteen older window demonstrations on
the rebuilt window; a thing turned and snapped to the wall it backs onto; the message
over the model frosted; the estimate window's heading and tabs; the second picture
of the scene drawn without shadows and following a resized window; then the release
candidate's table.

* **From the judge's notes J19-34 and J19-37, built and not yet seen:** a press on any
  thing that stands where it is put picks it up, selected or not (a press on one that
  was not selected slid the whole view: "a person reaching for the vanity will move
  the world"); and the program's own list of what is on screen
  (`PLUMBLINE_MAP`) leaves out what a panel has scrolled out of sight or cut off at
  its edge, by the engine's own account of what is clipped (buttons below the left
  panel's bottom edge were listed; a script clicked them, nothing happened, and
  nothing said so: five of six Write out presses wrote nothing).

* **From the judge's notes J19-38 and J19-39:** the BIM file now carries, on each thing
  that is one of an item, what its sheet said of it: maker, model, weight and the
  sheet, each with how sure it is and the line it came from (`demos/spec.sh` checks
  for "'maker',$,IFCTEXT('Northfield Millwork (Assumed; vanity.pdf, page 1 ..." and
  passes; the judge's own search of the judged program's file found none). A file
  written out a second time says it took the place of the one that was there (built,
  not yet seen). **Done since from J19-39 (call 65):** a water closet, a lavatory and a
  tub go out as `IfcSanitaryTerminal` with their sort, and a table, chair, bed, sofa or
  desk as `IfcFurniture` with its sort; on the made-up house 6 of the 22 fixtures are
  now sanitary terminals, and IfcOpenShell finds no fault. **Changed behaviour:**
  reading that file back now says "16 fixture or outlet" and "6 plumbing fixtures"
  where it said "22 fixture or outlet"; `demos/export.sh` expects the two lines.
  **Still not done:** equipment by sort (an air handler as IfcUnitaryEquipment).

* **From the judge's notes J19-40 to J19-44 and the ones left open before them (call
  66). Built; the window pieces not yet seen in a picture (the judge still has the
  display):**

  | Note | What it said | What was done |
  |---|---|---|
  | J19-20 | every clash mark one red; the two windows count clashes differently (87 and 21) | a finding is red, a question for the field amber, and the Show panel says how many of each; both windows count what is in the same space |
  | J19-19 | "What it is of hot to the kitchen sink, elbow 1: unknown" | thing first: "Hot to the kitchen sink, elbow 1, what it is: unknown"; a group is "N for the same reason. The first: ..." |
  | J19-16, J19-27 | places as bracketed decimal feet | "E 0' 0", N 28' 0"", as the pointer's read-out writes it (`panels::places`, two tests) |
  | J19-44 | "Enter takes the tool in hand" / "Enter to draw" | "Enter, then click on the plan." everywhere |
  | J19-40 | "the bim model (ifc)"; an empty job writes files without a word | named as said; "The job has no lines yet: the file has its headings and nothing under them." |
  | J19-15 | forty notches out the house is gone | the wheel stops where the building is about a sixth of its framed size |
  | J19-41 | the saved mark is off the window on a real job; a file name in the bar | answered before this note arrived: the bar has no file name and the view buttons moved out of it (not yet seen) |
  | J19-42 | held: typed door sizes are placed | nothing to do |
  | J19-43 | "Still to be typed: Sill height" named the wrong field | the judged program is 14fc994; the message was rebuilt after it (which field is empty is now named from the fields themselves). To be seen on the current tree |
  | J19-14 | plan of all levels is the roof, solid | not changed: it is the roof plan (call 66 says why) |
  | J19-9 | a drag slid the house less than the hand | not changed: the ground grabbed stays under the hand; the house is further off (call 66) |
  | J19-2 | the Show switches open below the fold | open |
  | J19-22, J19-23 | an item's kind has no source; the model number in a title is not read; electrical data on a sheet is not mentioned | open (minor) |
  | J19-24 | the status line cuts a message that has room | open |

  **Changed behaviour, named:** the Clashes tab's count on the made-up house is 21
  where it was 87; the to-verify rows, the tool prompts and every place shown in the
  window read differently (the words above). No figure changed. 268 tests pass.

* **J19-46, the worst of the judge's notes so far: work lost without a word.** Three
  windows were drawn, the Wall tool was picked (its sizes open, the cursor in
  Thickness), F2 was pressed, the program was closed. The saved job had none of the
  three and nothing on screen said it was not saved. **The cause, read in the code:**
  while any field has the keyboard the key handler returns before it reaches Save;
  F2, Ctrl+S and the Save button were all dropped, silently. Since call 64 a tool
  picked always has its sizes open, so this is the ordinary state to press Save in.
  **Fixed:** Save works whatever has the keyboard; with a value half typed the line
  says "Saved ... What is being typed is not in it yet: Enter sets it." Ctrl with a
  letter is never a letter of the value. **Found beside it and fixed:** closing a
  window with changes not saved closed it and dropped them; the first close is now
  answered "Not closed: the job has changes that are not saved. Save (Ctrl+S), or
  close again to leave without saving them."

  **The demonstration, written before it is run** (`demos/toolsw.sh`, extended): the
  session's only Save is now pressed with the Door tool picked and its sizes open.
  The job read back must still have its two doors, 3' 6" by 6' 8" at 6 ft and 2' 8"
  by 6' 8" at 18 ft, openings 41.11 SF (3.5 x 6.6667 + 2.6667 x 6.6667 = 23.33 +
  17.78). On the program as it was the saved job would have no door at all: that is
  the red this check must be proved against (the judged binary from 14fc994 is in
  the scratch folder for exactly that). Picture 06 must show the saved line. **Not
  run yet**: the judge has the display. The close warning has no demonstration yet
  (the virtual display has no window frame to press the close button of; `xdotool
  windowquit` is to be tried).

  J19-45 held: typed window sizes are placed as typed when the order is plain.

## Nineteenth judgment: the report (arrived 2026-11-12, recorded the turn it arrived)

An Opus judge, on the program as committed at 14fc994, 25 sessions on the virtual
display with a real pointer and keyboard, every picture read, 51 notes (J19-1 to
J19-51; its running notes and pictures are in the scratch folder `j19/`). No session
panicked. It was asked one question: **would Patrick see it now?**

**Its verdicts, in its words, against Patrick's words:**

| What Patrick asked for | Verdict | Why |
|---|---|---|
| moving round the model ("Google Earth feel") | partly | wheel on the pointer, drag and coast, turn without flipping, fly to a thing and plan easing in all held; it runs off at the horizon (J19-49), a double-click on the sky blanks the view (J19-50), a drag begun on the house also selects it (J19-11) |
| the look ("glassy", "Apple", "hyper realistic") | **no** | panels read as grey cards with lines showing through and no frost (J19-3, 8, 31); plan labels and roof lines painted over panels and bars (J19-13); the top bar cut off (J19-1); the house "a tidy massing model, not a game-grade one" (J19-5); placed things plain boxes (J19-27) |
| switches and scrolling | partly | eight clear switches and panel scrolling held; systems, levels and panels had none (J19-18); the switches open below the fold (J19-2) |
| a maker's sheet made into an item | yes, with gaps | read right, said what it did not know, refused the sheet that says nothing; a thing with no height is set down unseen and untested with no word (J19-25, 26) |
| picking a thing up and moving it | partly | in 3D on open ground "the real thing": red while inside, the overlap in feet and inches, one-step undo, saved job agrees; needs a click first (J19-34), fails on plan (J19-30, 33), lands far from the hand when pointed at a wall (J19-32) |
| writing out ("export to BIM files") | partly | six files written, an empty building refused clearly; no furniture class, the bookcase dropped, no maker or model in the file (J19-39) |
| typed sizes | partly | doors and windows place what is on screen; the wrong field named (J19-43); **Save with a field open saves nothing and says nothing: three windows lost** (J19-46, 47) |
| anything silently wrong | **no** | new openings lower the price and the recap does not say why (J19-48); work lost through F2; buttons listed as on screen that are cut off (J19-37); the bookcase unseen and untested |

**Where each note stands.** "Built" means in the tree and passing its tests; "seen"
means looked at in a picture of the current program. Until the pictures are retaken
(next, now the display is free) nothing below that is a window change is claimed as
shown.

| Notes | State |
|---|---|
| J19-6, 7, 12, 17, 42, 45, 51 | held; nothing to do |
| J19-46, 47 (Save with a field open; the folder path in "Saved ...") | built: Save works whatever has the keyboard and says the typed value is not in it; the line names the file only; closing with work not saved warns first. `demos/toolsw.sh` extended to press its only Save with a field open. Not yet run |
| J19-48 (price drops, recap silent) | built: under the Price on the recap, in the words the command line uses ("Nothing is priced on 3 things that could be (2 doors, 1 window): they are in no line and no total above."), and a red "3 not priced" beside the price in the model window's bar. One sentence for every view (`view::not_priced_said`). Not yet seen |
| J19-20 (clash certainty; 21 against 87) | built (call 66): red finding, amber question for the field, counts of each under the switch; both windows count what is in the same space. Not yet seen |
| J19-29, 30, 32, 33, 34 (carrying: plan, first press, at a wall) | built before the report from the running notes. Not yet seen |
| J19-3, 8, 31 (no frost) | built before the report: the panels are frosted (a blurred picture of the model behind each); seen once in `demos/captured/glass`; to be seen again on plan |
| J19-13 (plan labels over the panels) | built before the report. Not yet seen |
| J19-1, 41 (bar cut off; file name in it) | built before the report: no file name in the bar, view buttons out of it. Not yet seen on the whole house |
| J19-4, 16, 19, 27, 36, 40, 44 (words) | built (call 66 and before): hours to two places, the dollar sign first, places as E and N in feet and inches, sizes in feet and inches, to-verify thing first, Enter said one way, names as said |
| J19-11 (a flick also selects) | built before the report. Not yet seen |
| J19-15 (the house zoomed away to nothing) | built (call 66) |
| J19-18 (systems and levels without switches) | built before the report: each level and each system has a switch. Panels still cannot be put away one by one: **open** |
| J19-23, 25, 26 (a thing with a size missing) | built before the report: weight not on the sheet is kept as Unknown; setting one down says it cannot be drawn or tested yet and what to measure. Not yet seen. Drawing its footprint where two sizes are known: **open** |
| J19-28 (clash marks left on after a carry) | built before the report: they go back to how they were |
| J19-37 (the map lists cut-off buttons) | built before the report: the map leaves out what the engine reports as clipped |
| J19-38, 39 (overwrite without a word; IFC classes, maker and model) | built (call 65 and before): the second writing says it replaced the first; sanitary terminals and furniture by sort; maker, model, weight in the file |
| J19-43 (wrong field named) | built after 14fc994. To be seen |
| J19-49, 50 (the wheel at the horizon; double-click on the sky) | **being fixed now** |
| J19-14 (plan of all levels is a solid roof) | not changed (call 66 says why); the judge ranks it eighth of ten, so it is looked at again with the pictures |
| J19-9 (the house moves less than the hand) | not changed (call 66) |
| J19-10 (a sky at the horizon) | as intended since call 62: QUESTIONS.md Q22, VISUAL.md C2 |
| J19-2 (switches below the fold) | **open** |
| J19-5 (a massing model, not hyper realistic) | **open**: the light-and-materials pass, the largest thing left of call 62 |
| J19-21 (shipping weight kept as weight; Items pushes Write out down), J19-22 (kind without a source; electrical data unread), J19-24 and J19-39 (a message cut in the one-line bar), J19-35 (the carried thing's tag hides what it hits; a second one is named "... 2") | **open**, minor |

**What the judgment says about the work.** Its two "no"s are the look and silence.
The look was already known and is the pass not yet started. Silence was not known:
Save dropped without a word and a price that fell without a reason were both found
by a stranger in an afternoon, in the window, on the path a person takes, after
seventeen judgments that never went there. Both are fixed in the tree and neither
is yet proven in a picture.

## The light and the materials: the plan, written before it is built

Judge 19's first "no" (J19-5: "a tidy massing model, not a game-grade one") and the
largest thing left of call 62 ("hyper realistic"). VISUAL.md C1 and C3 are the lines
it is built to. What the engine gives is in the engine survey (call 63); nothing here
is taken from memory.

**What the window does now** (`scene.rs`, `setup`): no tone mapping at all (a colour
goes to the screen as the surface's own colour times the light, cut off at white),
two lamps and one flat light from everywhere, a painted dome for a sky that lights
nothing, shadows with hard edges, glass that is see-through and reflects nothing.
That is why it reads as a diagram: nothing in it is lit by the place it stands in.

**What is to be built, in the order it will be tried, each seen in a picture before
the next:**

1. **The camera sees a range of light and folds it to the screen** as a camera does:
   `Hdr` with `Tonemapping::AgX` (its tables are already built in), dither on, and a
   little bloom so that sun on glass and metal glints rather than clips.
2. **The sky lights the house.** A small picture of the sky all round (the same pale
   horizon and blue overhead the dome is painted with, and the ground's grey below)
   is made at start and given to the engine as the light from everywhere
   (`GeneratedEnvironmentMapLight`): walls facing the sky are cooler and lighter than
   soffits facing the ground, and glass and metal reflect the sky. The flat light
   from everywhere goes. If the software renderer the demonstrations run on cannot
   make that picture (it needs a compute pass), the flat light stays on that
   renderer only and STATUS says so; the window on a real card still gets it.
3. **One sun.** The second lamp goes; the sun's shadow edge is softened with distance
   from what casts it (the engine's soft shadows, a build feature) and steadied over
   frames with the anti-aliasing already on.
4. **Materials say what they are.** Glass: smooth, reflecting the sky, a little
   see-through. Metal roofing and gutters: a satin. Brick, stone, shingles and wood:
   rough, with the relief they already have. Nothing with a white-hot highlight.
5. **The ground meets the house**: the shade where a wall meets the ground is kept
   (it is there now) and looked at again under the new light.

**What it will break, known before starting.** Every colour on screen changes, and
five demonstrations read colours out of pictures: `glassw` (the door's and the
roof's colour, to find them), `carryw` (the vanity's wood and the red of a clash),
`dressw` and `pricew` (a finish's colour where it is priced), and the plan's paper in
`planw` and `firstw` (the plan is unlit and should not change; that is a check on
the pass, not a cost of it). For each: the colour is measured again on the new
picture, the test for it is rewritten from the measurement, and the check is proved
red against the picture it must fail on (each already has one: the roof switched
off, the vanity not selected, the wall before its finish). No check is loosened: a
test that found "the roof" by its colour must still tell the roof from the sky and
the wall beside it.

**How it is judged.** By eye first: the made-up whole house from the street, from
above, and close on a window, before and after, side by side. Then a visual judge
against VISUAL.md C and D with those pictures. Patrick's own test is the one that
counts, and the pictures are what he will be shown.

## After judge 19: what was run and seen (2026-11-12, evening)

**On the program as committed at 99021d2** (call 67's changes), all five window
demonstrations pass and every picture was taken on that program:

| Demonstration | Result | What was read in its pictures |
|---|---|---|
| `toolsw` | PASS. **Proved red first** on the judged program (14fc994): the job read back had no door at all. Green: both doors saved by the session's only Save, pressed with the Door tool's sizes open | 06: "Saved job.json. What is being typed is not in it yet: Enter sets it." and a green Saved mark at the bar's end; 04: the refusal as a round frosted card under the view buttons, whole; the right panel as tall as what is in it; Show straight after Levels; "2 not priced" in red beside the price |
| `carryw` | PASS | 02: the vanity red while in the water closet, an amber clash mark, and under the Clashes switch "1 to verify in the field" beside an amber spot; its place reads "E 7' 0", N 0' 10 1/2"" and its size 3' 0" x 1' 9" x 2' 10 1/2" |
| `itemsw` | PASS (IfcOpenShell validates the model written from the window) | not read one by one yet |
| `pricew` | PASS | 03: "Wall line E 0' 0", N 0' 0" to E 24' 0", N 0' 0""; "19 not priced" beside the price |
| `glassw` | PASS, with the new rules for a double-click and for the wheel with nothing near under the pointer | 01 read |

`tools/committed.sh` passes on dd42088 (the code of 99021d2 and the pictures).

**A mistake in the record, set right.** The tools pictures were first committed
while that run was still going (commit aa17b92: five new pictures and one old, and a
message saying the run had passed before it had). The run did pass; e596a17 holds
its whole set and says so.

**Seen in those pictures and not right:**

* The job's name in the top bar is cut ("Sample whole house (not a...") with room
  beside it (VISUAL.md A5).
* On a job of one wall the plan runs under the right panel: the view is framed on
  the window, not on the part of it the panels leave free.
* Next steps marks "Add a drawing and set its scale" done on a job with no drawing.

**Not yet seen in any picture** (built, tests pass): carrying on plan, the first
press picking a thing up, a thing pointed at a wall landing by the wall, the camera
guards (a double-click on the sky; the wheel at the horizon), the close warning,
the whole house's switches now above the fold (seen: they are, in the light
pictures below), the Building tab's rows by wall type. Judge 19's notes on these
stand until a picture or a second judge says otherwise.

## The light: step one built and seen (call 68)

`demos/captured/light/`: the made-up whole house, **before** (`before-whole.png`, the
program as committed) and **after** (`after-whole.png`, `after-window.png` close on a
window, `after-street.png` low from the street), taken the same way on the same job.

What the after pictures show that the before does not: a cast shadow of the house on
the ground; the wall in the sun and the wall out of it plainly two walls; window
glass that reflects the sky, paler toward the horizon, with the muntins dark against
it; the sky pale at the horizon behind the house from the street; the frosted panels
blurring the siding behind them in the close picture. What they do not show: nothing
in them would be taken for a photograph. The siding, stone and roofing are the same
patterns as before under better light; the ground is a plain grey plane; there is no
soft edge to the shadow. It is a better-lit model of a house, not yet a "hyper
realistic" one, and the pictures were sent to Patrick as they are.

The first picture under the new light was wrong and is not kept: with the sky's full
blue as fill, the ground and the trim were lit blue. Call 68 has what was changed.

**Being run now:** the six demonstrations that read colours out of pictures
(`glassw`, `carryw`, `dressw`, `pricew`, `planw`, `firstw`), to find which checks the
new light breaks. Each broken check is re-derived from the new picture and proved
red before it is trusted; none is loosened. The light is not committed until they
have been through.

## Patrick's request for a snapshot of each trade (call 69)

Seven snapshots, `demos/captured/trades/`, the made-up whole house, both windows side
by side, on the program as committed at 58ab151 (the new light). Sent to him.

| Trade | In the model (3D) | On the estimate | What the snapshot is |
|---|---|---|---|
| Foundation | **not drawn** | Division 3: footings and foundation walls (one subcontract figure), slab on grade by the SF | the estimate at Division 3 beside the finished house |
| Framing | **not drawn** | Division 6: wall framing 2x6 at 16 in and partition framing 2x4, by the SF of each wall | the estimate at Division 6 beside the finished house |
| Electrical | drawn: conduit, cable, panel, meter, receptacles, lights | its tab: power and lighting, by run | the house seen through, electrical only |
| Plumbing | drawn: pipe, fittings, fixtures, water heater | its tab | the house seen through, plumbing only |
| HVAC | drawn: supply and return duct, air handler, registers | its tab | the house seen through, mechanical only |
| Interior woodwork | **not drawn** | Division 6: casing at each door, base by the LF of each room, cabinets as one figure | the estimate at those lines beside the finished house |
| Waterproofing | **not drawn** | Division 7: sheathing and weather barrier by the SF of each wall, flashing and sealant at each opening | the estimate at Division 7 beside the finished house |

**Seen in them and not right:** with the house seen through, electrical and plumbing
runs are thin and pale against the ground under the new light and are hard to read
(HVAC's ducts read well); the Electrical tab's "To verify" lists plumbing questions.

**What it means.** Four of the seven trades he asked to see cannot be seen. That is
now Q23 with the working assumption that they are to be drawn, and proposal 20.

**Stopped to do this, and to be run again:** the six colour-measuring demonstrations
on the new light (`glassw` was part way; none had finished). The light commit
(58ab151) has still only had the window crate's tests, and `tools/committed.sh` has
not been run on it.

## The foundation drawn: the demonstration, written before it is built (Q23, step 1)

**What is ported.** BIMCraft has a strip footing: a run under a wall with a
rectangular section, drawn between points like a beam (`crates/core/src/commands.rs`,
`strip_footing` and `runs_of`, at f202abd85d1c). Here it is a kind of its own,
**Footing**, held the way a wall is held (a line on plan, a type that gives its width,
and its depth), as a door and a window are both held as an opening. A foundation wall
is a wall, on a level of its own under the first floor, of a type that says what it
is.

**The demonstration** (`demos/found.sh`, from an empty job, at the command line; the
window after). A house 40 ft by 28 ft on the centre lines of its walls.

1. A level "foundation" at -4 ft, 4 ft high.
2. A footing type "footing 24 x 12": 24 in wide, 12 in deep. Four footings on the
   four centre lines, their tops at the level's floor (-4 ft).
3. A wall type "foundation wall, 8 in concrete": 8 in. Four walls on the same lines,
   4 ft high.

**The figures, by hand.**

* Footings: 40 + 28 + 40 + 28 = **136 LF**. Section 2 ft x 1 ft = 2 SF. Volume 136 x
  2 = 272 CF = **10.07 CY** (272 / 27 = 10.074). The corners: two strips that end at
  a corner point overlap in a 1 ft square and leave a 1 ft square notch outside it,
  so length times section is the volume of the mitred ring exactly.
* Foundation walls: 136 LF, face one side 136 x 4 = **544 SF**, volume 544 x 8/12 =
  362.67 CF = **13.43 CY**.
* Each footing says its own: the 40 ft ones 40 x 2 / 27 = **2.96 CY**, the 28 ft ones
  28 x 2 / 27 = **2.07 CY**; 2.963 + 2.074 + 2.963 + 2.074 = 10.074.

**What must be true, each checked:** `show` of a footing says Footing, its width and
depth (never "thickness" and "height"), its length and its volume in CY with what it
is made of; the level's takeoff counts walls and footings apart (a footing is not a
wall: no wall count, wall length or wall face on any level changes); a footing whose
type has no width has no volume and says why, not zero; the saved job reads back the
same; the BIM file has four `IfcFooting` (`STRIP_FOOTING`) and IfcOpenShell finds no
fault; a footing with nothing priced on it is counted as not priced.

**In the window:** a footing is drawn as concrete under its wall; the ground has a
switch under Show, and with the ground off and the upper levels off the foundation is
seen whole. That picture is the snapshot Patrick asked for and did not get.

**Not in this step:** the made-up whole house gets its foundation in a step of its
own, because it changes that house's Division 3 from one subcontract figure to lines
by the CY, and every changed figure is named when it is.

**The foundation, step 1: built and run at the command line (call 70).**
`demos/found.sh` passes: 4 footings, 136.00 LF, 10.07 CY; one of them 40 LF, 2 ft
wide, 1 ft deep, 2.963 CY, listed in a footing's words; 4 foundation walls, 136.00
LF, 544.00 SF, counted apart from the footings; no clash among them; one footing
over another refused; a footing with no depth refused; a footing priced at $185.00
the CY comes to $548.16 (2.9630 x 185) and the estimate says the other 3 footings
and 4 walls are not priced; the job reads back the same; the BIM file has four
`IfcFooting` and IfcOpenShell finds no fault. 273 tests pass; the port map has the
row. **Not done of the demonstration as written:** the foundation walls' own 13.43
CY (there is no volume figure on a wall yet; call 70 says so), and everything in the
window (the concrete, the ground's switch, the picture). The made-up whole house has
no foundation yet.

**The foundation in the window: built and seen** (`demos/captured/trades/1-foundation-drawn.png`,
both windows, the job of `demos/found.sh`; sent to Patrick). With the ground switched
off (a new switch under Show, offered where something is drawn below the ground): four
footings in concrete, closed at their corners and projecting evenly past the walls;
four foundation walls standing on them, meeting cleanly at the corners; the level's
panel reads Walls 4, 136.00 LF; Wall face 544.00 SF; **Footings 4, 136.00 LF; Footing
concrete 10.07 CY**; the Building tab has the same footings row under the walls'
total; "8 not priced" beside the price.

**Seen in the first picture and fixed before the second:** each wall was carried a
foot past its corner (the rule that closes wall corners took the footing under a wall
for a wall meeting it; it now takes walls only), and the level's panel did not list
its footings.

**Seen and not fixed:** the footings' concrete sits under a column headed "Net, one
side" on the Building tab; the foundation walls are bare, the off-white of any wall
with nothing priced on it, not concrete; on plan (not looked at closely yet) the
foundation is drawn like any level. There is no tool in the window to draw a footing
yet: it is drawn at the command line. No window demonstration script yet; this
picture was taken by hand with the kit.

## Framing drawn (Q23 step 2, call 71)

**The demonstration, worked by hand before the code** (it is the module's own test,
`a_window_takes_studs_out_and_puts_its_own_round_it`): a wall 12 ft by 9 ft, studs at
16 in, a window from 54 in to 90 in along it, sill at 36 in, head at 84 in.

* With no window: a stud at each end and one on each of the centres 16, 32 ... 128
  in: 10 studs, each 108 - 4.5 = 103.5 in. One bottom plate and two top plates: 36 LF.
* With the window: the studs on centres 64 and 80 fall in the opening: 8 studs. Kings
  at 51 and 91.5 in, jacks at 52.5 and 90 in (82.5 in long, plate to head). Header: a
  2x6 for a 3 ft span, 52.5 to 91.5 in (39 in), from 84 to 89.5 in up. Rough sill 54
  to 90 in, 34.5 to 36 in up. Four cripples on the two centres taken out: two of 33
  in under the sill, two of 15.5 in over the header.

**Built and run.** `crates/world/src/building/framing.rs`, five tests, all of the
figures above; they passed the first time they ran. 278 tests pass in all. The port
map has its seven rows (two ported from BIMCraft's `timber.rs`, five ours).

**Seen.** `demos/captured/trades/2-framing-drawn.png`: the made-up whole house with
Framing on and the roof off: both storeys as stud walls, 2x6 outside and 2x4
partitions, plates, the openings with their headers, the ducts seen through the
frame. Sent to Patrick. (Its right half, the estimate at the framing lines, is the
picture taken an hour before on the same job.)

**Not right, or not done:** the floors are still drawn as solid decks between the
framed walls; where two walls corner their frames run through each other (there is no
corner framing); no window demonstration script drives the switch yet; nothing has
judged it.

**`planw` under the new light: fails, and not because of the light.** It traced no
wall at all. Read in its script: it picks the Wall tool a second time with a plain
click and then clicks the plan, and since call 64 a tool picked always shows its
sizes and wants Enter before it places. That is one of the thirteen older window
demonstrations not yet repaired on the rebuilt window (listed above as stale); the
same plain pick is in `buildw` and `draww`. All three now pick the tool the way the
kit does since call 64 (`tool "Wall"`, which presses Enter when there is nothing new
to type). **Not rerun yet.** Their clicks are still at fixed places on the screen
that were right for the window before it was rebuilt, and may not be now.

## Waterproofing drawn (Q23 step 3, call 72)

**Built and seen.** With Finishes off, the made-up whole house stands in its weather
barrier (all eight outside walls have "Sheathing and weather barrier" priced on
them), the stone and the cedar gone, and each of its seventeen flashed openings has
its flashing round it: `demos/captured/trades/7-waterproofing-drawn.png` (the whole
house, with Division 7's lines beside it) and `7-waterproofing-window.png` (close on
one window: a dark band at the head, down each side and under the sill, on the pale
barrier). Sent to Patrick. A test reads the barrier's words and refuses near-misses.

**Seen in the first picture and fixed before the second:** the flashing was drawn and
could not be seen, because the white casing was still on over it. Casing is finish
and now comes off with the rest.

**Not right, or not done:** the barrier is a flat colour with no laps or tape; the
roof is still in its roofing with nothing under it to take off; no window
demonstration script drives the switch; nothing has judged it. **Interior woodwork
is the one trade of Patrick's seven still not drawn at all.**

**A commit pushed with a test that did not compile, set right in the next.** b55f51b
(the weather barrier) was committed and pushed after a test run whose count came back
"0 passed, 0 failed": nothing had run, because a test in `crates/app/src/dress.rs`
builds an opening's dressing by hand and did not have the new `flashed` field. The
program itself built; its tests did not. I read the count after pushing, not before.
The fix is one field in that test; 279 tests pass. `tools/committed.sh` exists to
catch exactly this and had not been run since dd42088: it is run on this commit.

## Interior woodwork drawn (Q23 step 4, call 73)

**Built and seen.** `demos/captured/trades/6-interior-woodwork-drawn.png`: the made-up
whole house's first floor alone, mechanical trades off, seen from above: its rooms,
partitions and interior doors, sun through the windows on the floors, and a wood line
at the foot of the walls of each of the five rooms with base priced on it.
`6-interior-woodwork-close.png`: close on a partition: the base along it, stopping at
the six-panel door's opening and going on the other side. Sent to Patrick. A test
reads the words (base, and not a base cabinet or a gravel base). 280 tests pass.

**How the pictures were got, which is a finding about the window.** Two tries before
these were useless: a wheel into the house ended inside the ductwork, and a turn
about a point on the ground swung the house off the screen. A person wanting to look
into a floor from above has no one step for it. Picking the level alone and tilting
with the arrow key worked. That is a navigation gap, not a woodwork one, and it is
open.

**Not right, or not done:** the base is hard to see until the view is close (it is
5 1/4 in tall); no casing inside; no cabinets; no demonstration script; not judged.

## All seven of Patrick's trades now have something in the model

| Trade | In the model | Snapshot in `demos/captured/trades/` | Still missing |
|---|---|---|---|
| Foundation | strip footings (a kind of their own) and foundation walls, with a switch for the ground | `1-foundation-drawn.png` | on a small job of its own: the made-up whole house has none; pad footings; a tool in the window |
| Framing | each wall with a framing line, as its frame (Framing switch) | `2-framing-drawn.png` | joists, rafters, corners; quantities from the frame |
| Electrical | conduit, cable, panel, meter, devices | `3-electrical.png` | hard to read seen through |
| Plumbing | pipe, fittings, fixtures | `4-plumbing.png` | hard to read seen through |
| HVAC | duct, air handler, registers | `5-hvac.png` | |
| Interior woodwork | base in each room with base priced | `6-interior-woodwork-drawn.png`, `-close.png` | casing, cabinets, stairs |
| Waterproofing | weather barrier and opening flashing with the finish off (Finishes switch) | `7-waterproofing-drawn.png`, `-window.png` | roof underlayment, dampproofing, roof flashing |

The first seven pictures (`1-foundation.png` to `7-waterproofing.png`) are the program
as it was when he asked: four of them show estimate lines beside a finished house.
None of the four new layers has a window demonstration script or has been judged.

## The light under the six colour demonstrations (call 68, sky at 1,150)

On the build of that commit: `glassw` PASS, `carryw` PASS, `pricew` PASS, `firstw`
PASS (its plan's paper and inks unchanged, as they must be: nothing is folded on
plan). `dressw`: four of six colour checks pass as they were written; two needed
re-deriving for a wall that is now out of the sun (cedar there (132, 114, 98), the
bare wall (156, 158, 159)). Each is now set between the two measurements and the
script proves each test fails on the other's picture, every run. `planw` fails for a
reason that is not the light (above). Reruns of `dressw` and `planw` on the current
build are going now.

**The made-up whole house has a foundation (call 74).** `plumbline sixteen` now draws
it and prices it by what is drawn: Division 3 is $32,485.40 in 9 lines (was
$31,880.00 in 2: the slab and one lump), the job cost $394,677.48 in 399 lines (was
$394,072.08 in 392), the price **$468,876.85 (was $468,157.63)**. Each figure is
worked by hand in call 74 and agrees with the program to the cent. 280 tests pass.
**Not yet seen in the window on that house** (the picture of a foundation sent to
Patrick is of the small job), and the pictures in `demos/captured/light/` and
`trades/` taken before this show the old price in the bar.

**Seen: the whole house on its foundation** (`demos/captured/trades/1-foundation-whole-house.png`,
ground off, on the build of call 74; sent to Patrick): the concrete walls under the
stone wainscot, the level "foundation -4' 0"" in the list with its switch, the panel's
"Level: foundation" with Walls 4, 136.00 LF; 544.00 SF; Footings 4, 136.00 LF;
Footing concrete 10.07 CY; the price $468,876.85. **Not right in it:** the footings
themselves are behind the tool bar at the bottom of the window (the view is framed
on the house as it was before it had anything under the ground), and the foundation
walls are the bare off-white of an unfinished wall, not concrete.

**A wall priced as concrete is drawn as concrete.** A wall with a line that says
concrete and no cladding (the whole house's foundation walls: "Foundation wall, 8 in
concrete, formed and placed") is drawn in concrete on every face, where it was the
off-white of a bare framed wall. Built; 280 tests pass. **Not yet seen in a
picture.**

## A typed length lost its first digit after a tool's sizes were accepted (found by `planw`, 2026-11-13)

**What a person would have met.** Pick Wall. Its sizes are shown, already filled
(call 64). Press Enter to accept them, click the first corner, type the first wall's
length, 41' 3". The wall placed is **1' 3"** long, and nothing says so.

**How it was found.** `demos/planw.sh`, repaired to pick the tool the current way,
traced four walls and two of their lengths were wrong: 1.25 ft where 41.25 was typed,
and a closing wall of 43.81 ft. It failed the same way run alone on a quiet machine,
so it was not a lost keystroke. 1.25 ft is 1' 3": the typed length with its first
digit gone.

**The cause, read in the code.** "Typing replaces what the field holds" is a flag set
when a field of a tool's sizes opens already filled (call 64). Accepting the sizes
with Enter left it set. The first digit typed for a length opens the length field
with that digit in it, and the next key, finding the flag still set, cleared the
field before going in. The flag came from call 64, written this week: the defect is
mine and is two days old. None of the demonstrations met it because each one typed a
tool's sizes (which clears the flag) rather than accepting them, which is the same
blind spot the eighteenth judgment named: the scripts walk the path that works.

**The fix.** A single field that is opened with something in it (a length's first
digit) never replaces: `Doc::open` clears the flag. 280 tests pass.

**Not yet proven.** `planw` has not been rerun on the fix: the session's machine was
restarted during that run. It is the red (1.25 and 43.81 where 41.25 and 23.92 were
typed, twice) and must be the green before this is called fixed. `buildw` and `draww`
pick a tool the same way and have not been rerun either.

**The typed-length fix, proven on the path it broke.** `demos/planw.sh`, run alone on
the build of ee3baec: the four outside walls of the farmhouse come out at exactly the
lengths typed, **[23.9167, 23.9167, 41.25, 41.25] ft, 130.3334 ft round the house**.
On the program before the fix the same check gave [1.25, 23.9167, 41.25, 43.8062],
twice. That is the red and the green.

**`planw` still does not pass, for a second reason that is the script's.** Its next
check fails: "the sheet came out 77.6 ft wide". Read in its picture 03: the two
clicks that set the scale land under the plan, on the foundation plan's title, not on
the ends of the 41'-3" dimension. They are at fixed places on the screen that were
right before the window was rebuilt; the wheel now closes in by a different amount
and on the pointer, so the sheet sits elsewhere. The walls came out right all the
same because each side's length is typed. Every fixed place in that script has to be
taken again from pictures of the rebuilt window. **Not done.** Its pictures in the
repository are the old ones and are stale. `buildw` and `draww` are in the same
state and have not been rerun.

## Twentieth judgment: launched (2026-11-13)

An Opus judge, on the program as committed at ee3baec (a frozen copy), asked to break
the four new trades (footings and foundation walls, wall frames, the weather barrier
and flashing, room base) and the typed-length path, at the command line and in the
window, with every figure worked by hand, and to answer for each trade: would Patrick
see it? Its brief is in the scratch folder (`judge20-brief.md`); it writes its notes
as it goes to `j20/NOTES.md`. **Its findings are recorded here the turn they
arrive.** Until then none of those four layers has been tried by anyone who did not
build it. `tools/committed.sh` passes on the commit it judges' successor (the last
code commit).

## Patrick, 2026-11-13: a harness for each of the sixteen divisions (call 75)

Recorded as he gave it. **What the program has for it today, exactly:** every cost
line carries its division; the estimate and its recap read by division; the workbook
has a Divisions sheet; the starter assemblies put a line in every division. **What it
does not have:** any way to take one division out of a job and put it back; prices or
assemblies kept by division (both are one table for the whole job); anything written
down, by division, of what the program knows about that trade. So a harness for
Division 9 could today only be given the whole job and the whole price list.

The seam is proposed (PROPOSALS.md 21: a division out, a division in, and what a
division knows), and the two things only Patrick can rule on are Q24 (where a
harness's model may run; whether anything a harness writes may be Known). **Nothing of
it is built.** It does not displace what is owed: the twentieth judgment's findings
when they arrive, and the older demonstrations to repair.

## Twentieth judgment: stopped part way, and what it had found (2026-11-13)

Stopped at Patrick's word that the session's limits were low. It had finished the
foundation at the command line and the whole house's price, and begun the window. Its
notes as written are in `research/judgment-20-notes.md`. **None of these is fixed.**

| Note | What it found | Kind |
|---|---|---|
| J20-1 | Where one footing ends on the middle of another (a T), the concrete is counted twice over a half-width: two footings of 40 ft and 20 ft, 24 x 12, are 118 CF by hand and 120 CF in the takeoff (4.44 CY against 4.37), and nothing says so. Call 70's "length times section is exact" holds only where two footings both end at a shared corner | **wrong figure, silent** |
| J20-3 | A quantity is rounded to four places before it is priced: a 40 ft footing is 2.962963 CY, shown and priced as 2.9630; the whole house's four footings come to $4,533.40 where the concrete drawn at $450 is $4,533.33. Seven cents on Division 3. Call 74's "by hand" figures were worked from the rounded quantity | wrong by cents; older than this work (every line does it) |
| J20-2, 4, 5 | A footing is called a wall in five places: its type lists "wall thickness"; refusals say "wall type thickness", "wall line", "not a wall type"; and its depth can be measured only under the word "height" (`measure ... depth` is refused and the refusal lists only "type") | words; one blocks the field path |
| J20-5 | A footing can be drawn on a wall type and a wall on a footing type: types are not told apart | missing check |
| J20-6 | In the BIM file each footing and wall is a plain box between its end points: the ring has a square notch at each outside corner and a square counted in two at each inside one. The volume is right; the shape another trade receives is not | export |
| J20-7 | `plumbline finishes` says nothing of framing, weather barrier, flashing, base or concrete, and calls the foundation walls "bare" | the command line does not say what the window draws from |
| J20-8 | With one level picked alone, the other levels' switches still read on though nothing of them is drawn | a switch that reads wrong |
| J20-9 | The Up arrow did not tilt the view with the pointer parked above the tool bar | to be tried again; it was how the woodwork pictures were got |
| held | Division 3 and the whole house's price to the cent; every refusal of a bad footing; move, remove, measure, the pipe through a footing, the saved job, the BIM file's schema | |

**Laid out by the judge and not yet looked at** (its test job `k.json`, read from the
code's word rules): walls with framing priced as "2 x 6 @ 24", "2X4 studs 16 OC" and
"wall framing 2x6 16 in o.c." are **not framed, with no word**; of two framing lines
on one wall the first wins and the second is not mentioned; "Tyvek HomeWrap" and
"Felt paper" are **not read as weather barriers**; a bathroom with "Vanity base" and
"Shower base" priced **gets baseboard drawn**; a framed partition with "Anchor bolts
into concrete" priced **is drawn in concrete**; two windows 3 in apart have their
kings and jacks drawn through each other; a door as tall as its wall has no header.
Each is a prediction from reading the code, by a judge who was right about the rest.
Each needs trying, and most will be true.

## The clean-up pass after judgment 20 (2026-11-13)

Patrick asked for one last code and window clean-up. What was done, each committed
on its own with the tests passing:

| From | What | Proof |
|---|---|---|
| J20-1 | A footing that stops on the run of another (a T) no longer counts that concrete twice: the part of the stem inside the run is taken off the stem, and the formula on the figure says so | `demos/found.sh`: a 20 ft stem on a 40 ft run, 24 by 12, is 4.37 CY (118 CF by hand; it was 120); the ring is still 10.07 CY |
| J20-2, 4, 5 | A footing is spoken of as a footing: its type lists "footing width"; its depth is measured under "depth"; a footing with no length, a type with no width, a type that is not a footing's are refused in a footing's words | `demos/found.sh`: measured 18 in deep, the 40 ft run is 4.4444 CY (40 x 2 x 1.5 / 27) |
| the judge's test job | Framing is read from "2 x 6 @ 24", "2X4 studs 16 OC", "2x6 16 in o.c."; a number not said to be a spacing is not taken for one. A weather barrier is read from Tyvek, Typar, felt or tar paper; not from a vapor barrier. Base is read only where the line means the base at the foot of a wall (not a vanity base or a shower base). A wall is concrete only where the line says the wall is (not "anchor bolts into concrete") | unit tests with the judge's own lines; 281 pass |
| J20-7 | `plumbline finishes` says of a wall that it is framed and in what, has a weather barrier, is concrete; of an opening that it is flashed; of a room that it has base | read on the made-up house |
| J20-8 | With one level on show alone, the other levels' switches, and the roof's, read off; pressing one brings every level back with it | seen for the levels in a picture of the first floor alone; **the roof's switch is built and not yet seen** |
| the judge's test job | A pipe through a footing is a clash to settle, not "a hole to cut" | `demos/found.sh`: in the same space 1, holes 0 |
| J20-5 | A type is a wall's or a footing's, never both: a wall on a footing's type, or a footing on a wall's, is refused | `demos/found.sh` |
| the judge's test job | Openings with less than four studs between them are framed as one, under one header with studs packed between; a test holds that no piece of any frame runs through another, over the judge's cases | unit tests; 283 pass |
| seen in pictures | The job's name in the top bar is whole ("Sample whole house (not a real job)"), cut at 44 letters, not 26 | seen |

**Still open from judgment 20:** J20-3 (a quantity is rounded to four places before
it is priced: every line does it, and changing it changes every total by cents, so it
wants a decision first); J20-6 (in the BIM file a footing or a wall is a plain box
between its ends, notched at outside corners); J20-9 (the Up arrow with the pointer parked low);
two framing lines on one wall, where the first wins without a word; and **the judgment itself,
which was stopped part way and has to be run again to the end**.

**Seen while cleaning up, and fixed (2026-11-13).** With the house seen through, the
new room base was drawn solid: thick wood bars along every wall that swamped the runs
inside (seen in a picture of the electrical alone). The base, the frame and the
flashing now go glass with the walls they are on. The centre lines of runs are three
dots wide where they were two: a cable is thinner than a dot at the size a house is
seen at. `demos/captured/trades/3-electrical-now.png` and `4-plumbing-now.png` are
the electrical and the plumbing alone on the current program (the first read: the
conduit and cable plain in orange through the glass house; the second not yet read).

**One step to look into a floor (2026-11-13).** Picking a level under Levels puts it
on show alone and, in 3D, tilts the view down into it: its rooms, doors, ducts, pipes
and wiring are seen over its walls. Seen: `demos/captured/trades/first-floor-alone.png`
(one click on "first floor" from the opening view; the roof's switch and the other
levels' read off). Not right in it: the near corner of the house sits behind the tool
bar. It was the navigation gap named under "Interior woodwork drawn".

## The layers, switched in the window (`demos/tradesw.sh`, 2026-11-13)

The first demonstration script for last night's layers. The made-up whole house; one
click for each layer; five pictures in `demos/captured/layers/`; and for each a
measurement that **fails on the picture before the switch**, run every time:

| Picture | The switch | What is measured | Before the switch |
|---|---|---|---|
| 02 | Finishes off | the south wall is pale and cool, (160, 168, 174): the weather barrier | (138, 106, 77), the cedar: the barrier test fails on it, and the cedar test fails on 02 |
| 03 | Framing on | 0.48 of the wall is lumber, with the house seen between the studs (wanted 0.30 to 0.90) | 0.00 |
| 04 | Ground off | where the house's shadow fell there is none, (183, 186, 189) | (143, 147, 149), the shadow |
| 05 | the first floor picked | where the roof was there is none, (193, 189, 191) | (90, 98, 107), the roofing |

And the price read before and after is the same, $468,876.85: a switch changes what
is drawn and no figure. **It passed on its first run. Read:** picture 04, the house
with the ground away: the foundation walls are concrete grey under the stone now (the
concrete change, seen for the first time); the footings are still behind the tool
bar. Pictures 02, 03 and 05 are the same views as the ones read last night and were
not read again.

**What it does not check:** flashing at the openings (seen by eye last night, not
measured); the base in the rooms (seen by eye); anything of the foundation but that
the ground went. And it is the builder's own script: no judge has run it.

## Start here (written 2026-11-13 for the next run, on a copy of this repository)

**Where the work is.** Branch `claude/nifty-tesla-qmn44n`. The tree is clean and
pushed. 283 tests pass; the gates pass; `tools/committed.sh` is run on the last
commit of the session. Read `CLAUDE.md` first, then this section, then
`divisions/README.md` (what the program has for each of the sixteen trades) and
`research/README.md` (the surveys and the two latest judgments).

**What Patrick has ruled most recently.** Each of the sixteen divisions will have its
own AI harness with its own knowledge, pricing and tweaking; he will run locally;
nothing a harness writes is Known; a harness prices and tunes and does not draw
(calls 75 and 76). He asked to see each of seven trades and has been sent a picture
of each (call 69). His question Q23 (should the four trades that were only estimate
lines be drawn at all) is still open on its working assumption.

**What is true of the program today, in one paragraph.** It traces a plan, builds the
house in 3D, prices it in sixteen divisions, finds clashes, lists what to verify in
the field by dollars, brings in and writes out price lists, assemblies, estimates and
BIM files, and makes items from makers' sheets. Since call 62 its window is glass
panels over the model with Google Earth navigation. Since call 68 the house is lit by
a sun and its sky. Since calls 70 to 74 all seven trades Patrick named have a first
layer drawn. None of that last night's work has been through a judge to the end.

**What is owed, in this order.**

1. **What judgment 20 found is mostly fixed** ("The clean-up pass after judgment 20",
   just above, has each with its proof). Left: a quantity rounded to four places
   before it is priced (QUESTIONS.md Q25: it stays as it is until Patrick says); a footing
   or a wall in the BIM file as a plain box notched at its outside corners; two
   framing lines on one wall. **None of the fixes has been seen by a judge.**
2. **Run judgment 20 again to the end** on the result, with the same brief
   (`research/judgment-20-brief.md`). It needs a fresh frozen copy of the program
   (`PLUMBLINE_BIN`, in CLAUDE.md) and the IFC checker's Python package, which was in
   the old scratch folder and is not in the repository (`pip install ifcopenshell
   --target <a folder>`, then `PYTHONPATH`).
3. **The new layers have one window demonstration** (`demos/tradesw.sh`: the Finishes,
   Framing and Ground switches and a level looked into, each measured). It does not
   measure the flashing, the base or the footings themselves. Extend it.
4. **The window demonstrations all pass on the rebuilt window** (sixteen of them, on
   the last build; the end of this file lists them). What they are owed is to be
   read, picture by picture, and judged. Their fixed click places predate the
   rebuild.
5. **The seam for a harness per division** (PROPOSALS.md 21): a division out, a
   division in, and `divisions/` grown from an index into what each trade knows.
   Nothing of it is built.
6. Then what was owed before: the rest of the light (soft shadow edges, materials),
   judge 19's open notes, the release candidate's table of thirteen items.

**What is known to be not right and is nobody's finding yet.** Electrical and
plumbing are hard to read with the house seen through under the new light. (Looking down into one floor is now one click; the job's name is whole.) On a
one-wall job the plan runs under the right panel. The four new layers have no
quantities of their own: framing is still priced by the SF of wall and base by the
room's whole perimeter.

**Three mistakes of the last night, so they are not made again** (each is recorded
where it happened): pictures committed while their run was still going; a commit
pushed on a test count of nothing, whose tests did not compile; and "BIMCraft has no
framing" written from a list of file names cut short. And one defect of my own making
found late: since call 64 a length typed after a tool's sizes were accepted lost its
first digit. `planw` found it the moment it walked the path a person walks.

**Which pictures in `demos/captured/` are of the program as it is, and which are
not.** Of the current program: `layers/` (the last run of the session), `trades/`
(taken by hand through the night; each says in this file which build), `light/`
(the before and after of call 68; the price in their top bar is the old one).
Taken on builds of 2026-11-12 after the new light and before the trade layers and the
clean-up: `glass/`, `carry/`, `price/`, `tools/`, `items/`, `dress/`. Their checks
passed then and nothing since should have changed what they show, but they have not
been rerun on the last commit. (`plan/` is of the last build since `planw` was repaired, below.)
**Stale, from before the window was rebuilt:**
`first/`, and every other folder there (`build`, `draw`, `field`, `clash`, `d4`, `d5`,
`bring`, `house`, `d2` and the rest): their demonstrations click at places that have
moved and have not passed on the rebuilt window.

**How to check all is well after copying:** in `plumbline/`, `cargo test --workspace`
(283 pass), `tools/portmap.sh`, `tools/licenses.sh`, and from the repository root the
three `tools/gate-*.sh`; then `demos/found.sh`, `demos/spec.sh`, `demos/export.sh`
(command line, a minute each; the last two want IfcOpenShell on `PYTHONPATH`), and
`demos/tradesw.sh` for the window (about four minutes on a software renderer).

**J20-9, a tapped arrow key.** Held, an arrow turns the view steadily; tapped, it now
turns it a step (0.08 of a radian). A tap that began and ended inside one slow frame
was no turn at all, which is what the judge met. Built, 283 tests pass, **not tried
in the window**.

**The ground away now brings the foundation into view.** Switching Ground off eases
the view down by a little more than what lies under the ground is deep, and back when
it is switched on. Seen: `demos/captured/trades/1-foundation-whole-house.png`, retaken:
the whole house on its concrete foundation walls with the footing at the bottom, all
of it clear of the tool bar (it was the one thing named as not right in that picture).
`demos/tradesw.sh` rerun on this build: all ten measurements as wanted, the price
unchanged; its pictures in `demos/captured/layers/` are of this build. 283 tests pass.

**The estimate window's heading and foot (2026-11-13).** They were flat strips, the
one line of VISUAL.md section F named as failing. They are round cards lifted off the
page like its tables; the tabs lie on the page; the heading carries the job's name
where it said "Sheet". Seen: `demos/captured/layers/estimate-window.png` (the made-up
whole house: the recap, then the sixteen divisions with Concrete at 9 lines and
$32,485.40). `demos/pricew.sh`, which finds and types in that window, passes on this
build; its pictures in `demos/captured/price/` are of this build. 283 tests pass.

**`demos/planw.sh` passes on the rebuilt window (2026-11-13).** It is the
demonstration of the release candidate's item 3: from an empty job a real scanned
sheet (USDA Plan No. 7213, a public plan) is laid under a level, given its scale from
its own 41'-3" dimension, and its outside walls traced; then a vector plan, snapped
to its own lines. Run alone on the last build: the scale is 41' 3" over 371.7 points
of the page, the sheet 87.6 ft wide (worked figure 87.8, give or take a foot); the
four walls [23.9167, 23.9167, 41.25, 41.25] ft, 130.3334 ft round the house; the
second session passes too. Its 22 pictures in `demos/captured/plan/` are of this
build. Read: 07, the four walls lying exactly on the house's outline on the sheet,
in the glass window, the panel reading Walls 4, 130.33 LF, 1,042.67 SF, Assumed.

**How its clicks were taken again, so the next ones can be:** the wheel now closes in
by 0.81 a step about the pointer where it was more, so every place the script clicks
after its two steps of the wheel moved toward the pointer's place (760, 420) by one
factor. The factor is the dimension's length on screen now over then: it is 370.7
points long on the page; a failing run printed how many points my first guess had
picked (359.2, for a guess of 360), which gave the scale of the screen exactly
(1.4767 dots to the point), so 547 dots now where it was 619: 0.8843. The first try
used 0.8588, from a wrong guess at the dimension's length, and failed by 3 ft of
sheet width. `buildw` and `draww` want the same treatment and have not had it.

**`demos/buildw.sh` passes on the rebuilt window too** (run alone on the last build):
a floor, two rooms and a roof drawn in the window on walls traced from a plan, and the
saved job gives the figures worked by hand. It needed only the tool picked the
current way (its places are worked from where the program says the plan is, not
fixed on the screen). Its pictures in `demos/captured/build/` are of this build; they
have not been read one by one.

**`demos/draww.sh` passes on the rebuilt window** (run alone on the last build): a
house drawn in the window from an empty job (a level, walls, doors, windows, a duct, a
pipe, a fixture, equipment; undo; move; remove; save), and the saved job has the
figures worked by hand, each flagged as drawn. Like `buildw` it needed only the tool
picked the current way. Its pictures in `demos/captured/draw/` are of this build and
have not been read one by one.

**So of the older window demonstrations, on the rebuilt window:** `planw`, `buildw`,
`draww` and `firstw` (on an earlier build of the new light) pass. Not rerun:
`fieldw`, `clashw`, `d4w`, `d5w`, `bringw`, `house`, `d2`.

**Five more of the older window demonstrations pass on the rebuilt window** (a frozen
copy of the last build, one after another): `fieldw` (values measured in the window
as a laser types them; the price and the clashes follow), `clashw` (fittings and a
connection with a click each; the clashes listed and marked), `d4w` (the job priced
both ways; two prices typed in the window), `bringw` (a model, a price list and a
prior estimate brought into a new job; the saved job is the command line's, byte for
byte), and `firstw` (the first job from a real plan: walls, a door, windows, a floor,
a room and a roof). Their pictures in `demos/captured/` are of this build and have
not been read one by one. `d5w` stopped: a row of the estimate it scrolls up to is
no longer above where it was looking. The kit's `sheet_at` now looks the other way
when a row is not the way it was told; `d5w` is being rerun.

**`d5w` passes too** (rerun alone with the kit looking both ways for a row): the
estimate read on the sheet; overhead, profit, a unit cost and a stretch-out typed on
their own lines; a labor line switched off; a material changed in the model. Its
pictures in `demos/captured/d5/` are of this build.

**Where the window demonstrations stand on the rebuilt window, all of them:** passing
on the last build or a frozen copy of it: `tradesw`, `pricew`, `planw`, `buildw`,
`draww`, `fieldw`, `clashw`, `d4w`, `d5w`, `bringw`, `firstw`. Passing on builds of
2026-11-12 after the new light and not rerun since: `glassw`, `carryw`, `toolsw`,
`itemsw`, `dressw`. Not window demonstrations, or not rerun: `house`, `d2`. None of
the picture sets of this last round has been read picture by picture: that reading,
and a judge, are what the next run owes them.

**Every window demonstration passes on the last build (2026-11-13).** `glassw`,
`carryw`, `toolsw`, `itemsw` and `dressw` were rerun on the same frozen copy and
pass, so with the eleven above all sixteen stand on it: `tradesw`, `pricew`, `planw`,
`buildw`, `draww`, `fieldw`, `clashw`, `d4w`, `d5w`, `bringw`, `firstw`, `glassw`,
`carryw`, `toolsw`, `itemsw`, `dressw`. Their pictures in `demos/captured/` are of
that build (the last commit that changed the program; what came after it changed
only scripts and this file). **Read picture by picture, by the builder:** only those
named as read earlier in this file. The rest passed their measurements and have not
been looked at as a person would: a check that passes while the window is wrong is
a defect in the check, and that looking is owed.

**Read as a person: the first job's last picture** (`demos/captured/first/06-model.png`,
of the last build): the farmhouse built from the real plan stands on its own sheet,
its hip roof's lines by type, a door and two windows, the walls bare because nothing
is priced, "22 not priced" and "14 to verify" in the bar. It reads as what it is.
**Not right in it, and fixed (call 77):** the To verify panel said "whether wall N
bounds this room rests on its heights, which are assumed" once for each of four
walls under each of three figures. Walls equally sure are now named in one sentence.
283 tests pass (one wall alone reads exactly as before, which two tests hold).
**The picture itself predates the fix**: `firstw` has not been rerun since.

