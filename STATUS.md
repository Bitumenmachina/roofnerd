# Status

Each claim gives its scope, its evidence, and what remains. Evidence is output from a
command anyone can run again from the repository root.

Workspace this was run in: Linux, rustc 1.97.0, 2 cores, no display. The repository has no
remote (QUESTIONS.md Q2).

| # | Deliverable | State |
|---|---|---|
| D0 | Landscape decision | Demonstrated. Judged: see below. |
| D1 | World data model and SPEC.md | Demonstrated at the command line. No window exists yet. Judged: see below. |
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

1. For 65 crates, the crates.io registry API (`/api/v1/crates/<name>`): newest
   version, declared license, last publish date, registered repository.
2. For 22 repository addresses (20 reachable), `git clone --depth 1 --filter=blob:none --no-checkout`,
   then the last commit date and the license files read with `git show`.
3. For the three candidates neither route reached, a web search.
4. `tools/licenses.sh`: every crate the workspace links, with its declared license, checked
   against a permissive list.

**What it showed**

* Every appendix candidate has a call in DECISIONS.md, with the evidence beside it.
* Thirteen entries in the starting landscape were wrong, stale or unconfirmed; they are
  tabled at the top of DECISIONS.md. The three that change the plan:
  * OpenTakeoff is now AGPL-3.0-only, not Apache-2.0.
  * A permissive weighted straight skeleton exists (BIMCraft, MIT OR Apache-2.0), which
    closes the gap the brief named.
  * Bevy is at 0.20.1, and the full engine needs a newer compiler than this workspace has.
* `tools/licenses.sh` exits 0: 62 linked crates, all MIT, Apache-2.0, Zlib, Unicode-3.0 or
  Unlicense-or-MIT. The three direct dependencies were confirmed before being added.

**What remains**

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

---

## D1. World data model and SPEC.md

**Demonstration as the brief states it:** A saved job is readable JSON with one entity per
element. Changing one pitch changes, in a file diff, only that entity and the components
derived from it. Assumed and Unknown values display as flagged.

**How it is run:** `demos/d1.sh` (about a minute on a first build). It uses the built-in
sample job, which is made up. `cargo test` runs ten checks on the same model.

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
4. *Flags.* Before the change `show` lists 13 Assumed and 13 Unknown values, each with its
   reason: the assumed porch pitch and the seven figures that follow from it, the unknown
   parapet height and the face area that cannot be given without it, the unpriced cost
   lines, the unbuilt schedule codes, and the cost codes that wait on Q1.
   After the change: 5 Assumed, 13 Unknown.
5. *A contradiction is flagged, not resolved by guess.* Steepening one plane of the hip
   roof to 8:12 leaves its two hips and the ridge with no length and the note "the planes
   disagree about the height of the line's end: plane-0003 puts it at 20.00 ft; plane-0006
   puts it at 17.50 ft".

**Hand checks** (in `crates/world/tests/world.rs`, `hand_worked_figures`)

* South plane: trapezoid (40 + 10) ÷ 2 × 15 = 375 SF on plan; 6:12 factor √1.25 = 1.1180;
  surface 419.2627 SF.
* Hip: 15 over and 15 up on plan = 21.2132 LF; climbs 7.5 ft; √(450 + 56.25) = 22.5 LF.
* Porch rake: 8 ft run at 3:12 climbs 2 ft; √68 = 8.2462 LF.

**`cargo test`: 10 passed, 0 failed.** Beyond the above: a value with no source, an
Unknown carrying a number, a Known carrying none, a negative pitch, a repeated path, a
dangling child, and an element with no kind are each refused on load; a measure edited in
the file is caught and the recomputed figure used; an attribute the program does not know
survives a round trip; 1/16 in survives 5,280 ft from the origin.

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
  command line. Outlines and line ends are edited in the file.
* Not checked on load: overlapping or clockwise outlines, a line that does not touch the
  plane it names, a cost line naming a measure the element lacks.
* **Process note:** the brief asks that each demonstration be written down before the
  deliverable is built. For D0 and D1 the demonstration was written alongside the work,
  not ahead of it. D2's is written below, ahead.

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
