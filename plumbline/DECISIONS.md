# Decisions

D0: the landscape, checked, and a call per component.

**How each entry was checked (2026-11-10).** Versions, licenses and last-publish dates
come from the crates.io registry API. Last-commit dates and license files were read from a
shallow clone of each source repository. Where neither was reachable the entry says so.
`tools/landscape.sh` re-runs all of it; its output is kept in `demos/captured/d0-landscape.txt`.

A call is one of: **build on** (a linked dependency), **borrow** (port code or a pattern,
with attribution where code moves), **write** (ours), **reference only** (read, never
linked, no code taken), or **not used**.

## What the starting landscape had wrong or missing

| Brief said | What the check found |
|---|---|
| Bevy 0.19 | Bevy is at **0.20.1** (2026-10-28). 0.20 upstreamed a pan-orbit camera with CAD-style navigation. |
| OpenTakeoff: Apache-2.0 (verify) | Now **AGPL-3.0-only**. Its README says code through tag `apache-2.0-final` stays Apache-2.0; everything after is AGPL. |
| Gap: no permissive weighted skeleton | **Found one.** BIMCraft `crates/core/src/skeleton.rs` (MIT OR Apache-2.0): each edge moves in at its own speed, holes and gables included. Its tests build a roof with edges at 30° and 45°. |
| ifc-lite (louistrue) | The crate's registered repository is **LTplus-AG/ifc-lite**; the louistrue address serves the same content. Crate `ifc-lite-core` 28.2.0, MPL-2.0. |
| IronCalc v0.8 | **0.9.1** (2026-10-21). |
| Fornjot: archived | Its README says the project **has been shut down**. License 0BSD. Last crate release 2024-03. |
| IfcOpenShell: LGPL (verify) | **LGPL-3.0** (`COPYING.LESSER` is version 3; `COPYING` is GPL-3.0). |
| OCCT via opencascade-rs: LGPL-2.1 (verify) | opencascade-rs is **LGPL-2.1**. cadrum is an MIT wrapper around the same LGPL kernel. |
| pdfium-render: verify | The wrapper is **MIT OR Apache-2.0**. The PDFium library's own license was not checked here. |
| OpenAEC: LGPL-3.0 (verify) | `OpenAEC-Foundation/open-pdf-studio` is **LGPL-3.0**. Other repositories in the organisation were not located by name. Its published roadmap names no estimating or takeoff tool. |
| ProTakeoff (Driven-Waterproofing/take-off) | **Not reachable** at that address or three near spellings. A listing site calls it MIT. Nothing could be verified. |
| ConstructCAD | **Not located** by search or by guessed addresses. Nothing could be verified. |
| bimifc: v0.3 on Bevy 0.18 | Confirmed, and it has not moved: last commit 2026-05-20, still on Bevy 0.18. `Cargo.toml` says MPL-2.0; the repository root has no license file. |

## Every candidate, rated

**Maintained** is the last commit in the source repository. **Linux** says only what was
actually seen: "builds here" means compiled in this Linux workspace; "own CI on Linux"
means the project's own workflows name a Linux runner (count from
`demos/captured/d0-landscape.txt`); neither is a claim that it was run here.

| Candidate | Maintained (last commit) | License fit | Linux | Call |
|---|---|---|---|---|
| bevy_ecs 0.20.1 | 2026-11-10 | yes: MIT OR Apache-2.0 | **builds here** (rustc 1.95+) | build on (call 1) |
| Bevy 0.20.1 (engine) | 2026-11-10 | yes: MIT OR Apache-2.0 | own CI on Linux (14 of 14); **not built here**, needs rustc 1.97.1 | build on, unproven (call 2) |
| bimifc 0.3 | 2026-05-20; still on Bevy 0.18 | MPL-2.0 (Q3); no license file at root | own CI on Linux (3 of 3); not built here | not used (call 6) |
| BIMCraft 0.11.0 | 2026-11-09 | yes: MIT OR Apache-2.0 | own CI on Linux (2 of 2); not built here | borrow (calls 3, 4) |
| ConstructCAD | not located | GPL-3.0 per the brief; unverified | not checked | not used |
| ifc-lite (core 28.2.0) | 2026-11-10 | MPL-2.0 (Q3) | own CI on Linux (47 of 49); not built here | not used (call 6) |
| IfcOpenShell | 2026-11-10 | no: LGPL-3.0 | own CI on Linux (43 of 44); not installed here | separate tool at D6, never linked |
| IFC5-development | 2026-09-08 | no license file at root | not applicable (a format) | pattern only (call 5) |
| z3 crate 0.22.0 | 2026-11-10 | yes: MIT | own CI on Linux (3 of 3); not built here; C++ library | not used (call 8) |
| monstertruck 0.4.1 | 2026-10-07 | yes: Apache-2.0 | own CI on Linux (1 of 1); not built here | not used now (call 7) |
| truck (upstream) | 2026-11-02; last crate release 2024-09 | yes: Apache-2.0 | no CI workflows in the repository; not built here | not used now (call 7) |
| opencascade-rs 0.3.0 | 2026-11-03 | no: LGPL-2.1 | own CI on Linux (4 of 4); not built here; C++ kernel | not used (call 7) |
| cadrum 0.9.1 | 2026-11-07 | MIT wrapper over an LGPL-2.1 kernel | own CI on Linux (3 of 3); not built here | not used (call 7) |
| Fornjot | shut down (README); last commit 2026-06-19; last crate release 2024-03 | 0BSD | own CI on Linux (2 of 2); not built here | not used |
| straight-skeleton 0.2.1 | 2026-07-17 | no: GPL-2.0-or-later | own CI on Linux (2 of 2) | reference only (call 3) |
| IronCalc 0.9.1 | 2026-11-10 | yes: MIT OR Apache-2.0 | own CI on Linux (6 of 6); not built here | candidate at D3 (call 9) |
| hayro 0.9.0 | 2026-11-09 | yes: Apache-2.0 OR MIT | own CI on Linux (2 of 3); not built here | build on at D2 (call 10) |
| pdfium-render 0.9.5 | 2026-11-08 | wrapper yes; PDFium itself not checked | own CI on Linux (1 of 1); needs a native library | fallback (call 10) |
| OpenTakeoff | 2026-11-07 | no: AGPL-3.0-only | own CI on Linux (2 of 2); a web application, not run here | reference only (call 11) |
| ProTakeoff | not reachable | unverified | not checked | not used |
| OpenConstructionERP (`datadrivenconstruction/OpenConstructionERP`) | 2026-11-10 | no: AGPL-3.0 | own CI on Linux (24 of 26); not run here | reference only (call 11) |
| OpenAEC (`open-pdf-studio` checked) | 2026-11-10 | no: LGPL-3.0 | own CI on Linux (4 of 8); not run here | not used (call 11) |
| Brush | 2026-10-03 | yes: Apache-2.0 | own CI on Linux (3 of 3); not built here | no call until D7 (call 12) |
| OpenDroneMap | 2026-10-28 | no: AGPL-3.0 | own CI on Linux (4 of 5); not run here | separate tool only (call 12) |
| *newer:* rust_xlsxwriter 0.102.0 | 2026-11-08 | yes: MIT OR Apache-2.0 | own CI on Linux (18 of 18); not built here | build on at D6 (call 9) |
| *newer:* egui / eframe 0.37.0 | published 2026-10-15 | yes: MIT OR Apache-2.0 | not built here | fallback window (call 2) |
| *newer:* bevy_egui 0.43.1 | published 2026-10-10 | yes: MIT | not built here | build on at D2 (call 2) |
| *newer:* i_overlay 9.1.0, earcutr 0.5.0 | published 2026-10-14, 2025-05-29 | yes: MIT OR Apache-2.0, ISC | not built here | candidates (call 7) |

Only one candidate has been compiled in this workspace. "Works on Linux" for the rest is
their own CI's word, not this project's, until each is built at the deliverable that
needs it.

## Calls

### 1. The world: build on `bevy_ecs` 0.20.1, on its own
MIT OR Apache-2.0. Needs rustc 1.95. Builds in this workspace in about half a minute.
Entities, components, queries and change detection without the engine. Authoritative
geometry is our own 64-bit components in feet; nothing in the model depends on a render
transform. `hecs` (MIT OR Apache-2.0) is the lighter alternative and was passed over
because call 2 wants the same world the renderer reads.
**Proven:** D1 runs on it. A 1/16 in difference a mile from the origin survives a save and
a load (test `a_sixteenth_of_an_inch_survives_a_mile_from_the_origin`).

### 2. The 3D window: build on Bevy 0.20. Proven on a virtual display (see call 23).
Bevy 0.20.1 (MIT OR Apache-2.0) supplies mesh picking, the pan-orbit camera and more than
one OS window; its feature list names `mesh_picking`, `pan_orbit_camera` and `bevy_gizmos`.
The brief credits 0.19 with a transform gizmo; that was not confirmed in the 0.20 feature
list and is checked in the D2 proof. `bevy_egui` 0.43.1 (MIT) puts an egui estimate sheet
in a window. egui 0.37 (MIT OR Apache-2.0) is what BIMCraft uses for its schedules.
**Risks, stated plainly:**
* Bevy's full engine needs rustc 1.97.1; this workspace has 1.97.0 (QUESTIONS.md Q4). So
  this call has not been compiled here, let alone run.
* Render transforms are 32-bit. The answer is call 1: the model is 64-bit and the view is
  drawn relative to a job origin. That has to be shown on a real job, as the brief says.
* Bevy made three breaking releases in 2026 (0.18, 0.19, 0.20). The version is pinned and
  only the app crate depends on the engine; the world crate depends on `bevy_ecs` alone.
* The default feature set pulls in audio and gamepad support, which need system
  development packages. Those features are left off.
**The proof that opens D2:** a window with a plane picked by the mouse, a second OS window
showing the same world updating live, and a coordinate readout that holds 1/16 in at the
far corner of a real footprint.
**What reverses it:** if any of those three fails, the window is written on `eframe`/egui
with our own `wgpu` viewport. BIMCraft shows that route works on Linux.

### 3. Roof from a footprint: borrow BIMCraft's weighted straight skeleton
`IPedrax/BIMCraft`, `crates/core/src/skeleton.rs`, 709 lines, MIT OR Apache-2.0, last
commit 2026-11-09. It depends only on that crate's own `geom` module. It is ported, not
linked, because BIMCraft's core crate brings its whole document model. The port carries a
NOTICE entry and a header naming the source file and commit. The `straight-skeleton` crate
(GPL-2.0-or-later, confirmed) stays reference only.
**Not yet done:** nothing has been ported. The port is D2 work.
**Done (2026-11-10): ported**, at commit `f202abd`, as `crates/world/src/skeleton.rs`,
with the tolerance restated as a share of the footprint's size and a test on a 3 in edge a
mile from the origin. See call 23. What follows is the history.
**Was not done earlier the same day:** the session that picked the work up ran
in a workspace whose repository access is limited to one repository, and a request to add
BIMCraft was declined. The source was not fetched another way. The port waits on the owner
(QUESTIONS.md Q10). Nothing was written in its place: the call is to port a tested
skeleton, not to write a second one.
**To watch in the port:** BIMCraft works in millimetres, and its skeleton's closeness
tolerance (`EPS` of 1e-6) is written for that. Every tolerance in the port has to be
restated in feet and tested on a short edge. (An earlier version of this note said the
skeleton rejects edges shorter than half a unit. That was wrong: the test at line 343 is
on a unit direction vector and rejects only zero-length edges. The first judge reported
it, it was copied here unchecked, and the second judge caught it.)

### 4. BIMCraft as a whole: borrow from, do not build on
v0.11.0, MIT OR Apache-2.0, active. It is the closest permissive match and it is a
Revit-style document with a command engine. This project's premise is different: one ECS
world where cost, status and provenance sit on the same entity as the geometry. Building
on BIMCraft would make its document model the model. Pieces worth borrowing: the skeleton
(call 3), its roof faces and dormer handling, its schedule formula fields as a pattern,
and its IFC export approach. Not borrowed: its Revit reader (Apache-2.0 only, ported from
rvt-rs) and its DWG bridge (over an MPL-2.0 crate).

### 5. Saved job: write. IFC5 is the pattern only.
`buildingSMART/IFC5-development` (last commit 2026-09-08) has no license file at its root;
its README says the viewer is MIT and the examples are for education and testing, not
production. The layout (`header`, `imports`, `schemas`, `data`; `path`, `children`,
`attributes`) is followed. No IFC5 code or schema file is used. **Done in D1.**

### 6. IFC parsing and IFC4 export: write the export; link no parser
`ifc-lite` and `bimifc` are both MPL-2.0 (QUESTIONS.md Q3). `ifc_rs` (MIT) stopped at an
alpha in 2024. The brief needs IFC4 as an export only, so the export is written for the
entities this project has, and IfcOpenShell (LGPL-3.0) checks it as a separate tool that
is never linked (QUESTIONS.md Q7). No IFC import is in any deliverable.

### 7. Solid geometry kernel: none
Roofs, parapets and crickets are flat faces. Plane equations, polygon outlines and their
intersections cover D2 through D5. If 2D boolean operations are needed, `i_overlay` 9.1
(MIT OR Apache-2.0); for triangulating faces to draw them, `earcutr` 0.5 (ISC).
`monstertruck` 0.4.1 (Apache-2.0, last commit 2026-10-07) is the candidate if curved
surfaces ever arrive; its upstream `truck` is active again (last commit 2026-11-02), so
which of the two to follow would be re-checked then. OCCT is not used: a C++ kernel under
LGPL-2.1. Fornjot is shut down.

### 8. Constraint solver: not used
`z3` 0.22.0 is MIT and maintained (last commit 2026-11-10), and it builds a C++ library
or needs one installed. Derivation by direct recomputation covers every demonstration in
the brief. See PROPOSALS.md item 1.

### 9. Spreadsheet: build on `rust_xlsxwriter` for XLSX out; IronCalc is the formula-engine candidate
`rust_xlsxwriter` 0.102 (MIT OR Apache-2.0, 6.4M downloads) writes XLSX with live
formulas, which is D6's demonstration. It was not in the starting landscape. IronCalc
0.9.1 (MIT OR Apache-2.0) is the candidate for evaluating line formulas in D5; whether a
spreadsheet engine or a small expression evaluator of our own serves "the formula is
visible and editable on the line" better is decided when D3 opens, with both tried on the
stretch-out formula. `calamine` 0.38 (MIT) reads XLSX if reference estimates need reading.
**Decided when D3 opened (2026-11-10): write a small evaluator of our own for line
formulas.** Both were tried on the stretch-out formula. IronCalc 0.9.1, in a scratch
program outside this repository, with 12 in A1 and 24 in B1: `=A1*B1/12` gave 24. With the
stretch-out cell left empty it gave **0**, where this project has to give no number at
all. A name in place of a cell address (`=A1*STRETCH/12`) gave `#NAME?`. So the formula on
the line would read in cell addresses, units and statuses would have to be carried beside
it by hand, and every Unknown would have to be kept out of it before it was asked. Its
dependency tree is about 70 crates. An evaluator that knows names, feet and inches, and
Known/Assumed/Unknown is a few hundred lines (`crates/world/src/formula.rs`). IronCalc is
not linked. XLSX with live formulas (D6) is still `rust_xlsxwriter`'s job.

### 10. Plans and images to trace: build on `hayro` for PDF, `image` for images
`hayro` 0.9.0 (MIT OR Apache-2.0, pure Rust, last commit 2026-11-09). `pdfium-render`
0.9.5 is the fallback; it needs a native library, which needs the owner's go-ahead.
MuPDF is AGPL and is not used.

### 11. Takeoff and estimating projects: reference only or not used
* OpenTakeoff: AGPL-3.0-only now. Reference only. Its idea that an agent's measurement
  stays pending until a person approves it is already this project's status model: a
  value is Assumed until someone makes it Known.
* OpenConstructionERP: AGPL-3.0 confirmed. Reference only.
* ProTakeoff, ConstructCAD: could not be verified. Not used.
* OpenAEC: LGPL-3.0 where checked; no estimating tools. Not used.

### 12. Capture: no call until the owner rules on D7
Brush is Apache-2.0 (last commit 2026-10-03). OpenDroneMap is AGPL-3.0 confirmed and would
only ever be a separate tool.

### 13. This project's own license: MIT OR Apache-2.0
The pairing most of the Rust ecosystem uses. Both disclaim warranty and liability; Apache
adds a patent grant. Code borrowed under either can be carried with its notice.

### 14. Platform exceptions to "Rust by default"
None in the application. Outside it: five shell scripts (`demos/d1.sh`, `demos/units.sh`,
`demos/d3.sh`, `tools/licenses.sh`, `tools/landscape.sh`) and one Python file (`tools/license_expr.py`,
the license-expression checker). The demonstration and landscape scripts also call Python
inline to label a diff and read registry replies. None of these is part of the program.

### 15. Model calls made in D1 (ours to make; recorded so they can be overruled)
* ~~**Feet only, no conversion.** A value in any other unit is refused with a message to
  convert it.~~ **Overruled by the owner; see call 16.** What survives of it: a unit the
  program does not know is still refused, and nothing is ever read as feet by default.
* **The eave is its own line on the plane**, not "whichever edge was traced first".
  Re-tracing an outline cannot change which edge the pitch rises from. The eave has to
  lie along one outline edge and be the plane's low edge (no part of the outline on the
  other side of it). The slope is measured from that edge's line, not from the eave line
  as typed.
* **A line's kind says how it slopes.** An eave, a ridge and a headwall are level. A rake,
  a hip and a valley climb. A line that breaks this gets no length. It is the main guard
  against a plane whose eave was entered on the wrong edge.
* **A line runs along the outline of each plane it names.** Touching the outline at the
  ends is not enough. (First tested at five points; the third judgment ran a line across
  a notch between them, so the outline's edges now have to cover the line end to end.)
* **The same key twice in one JSON object is refused**, anywhere in the file.
* **A pitch above 240 in 12 is refused** as a wall. A coordinate beyond 1,000,000 ft is
  refused. Both are sanity limits against typing errors, not building rules.
* **A line has to fit its planes to 0.01 ft** (about 1/8 in) or it gets no length. That
  suits lines the program places itself (D2). It is tighter than a hand trace on a scanned
  plan can hold, so hand-entered lines will be flagged often until D2 snaps them.
* **References and labels carry no status.** The planes a line names, a cost line's
  description and the measure it is priced by are structure, not quantities. They are
  checked (a line off its plane is flagged; a cost line priced by a measure the element
  lacks is refused) but they do not carry Known/Assumed/Unknown.
* **An unknown field inside one of our own components is refused**, not kept. Anything
  else unknown in the file is kept and written back.

### 16. Units: held as entered, converted only for arithmetic (owner ruling, 2026-11-10)
**The ruling** (the handoff that followed D1; it overrules call 15's first bullet): "Feet
only, nothing converted" is rejected. US measurements stay, and the program takes more
than feet. An estimator enters what the drawing or the tape says. How units are held
internally was left to this project; metric was not asked for.

**The calls made under it:**
* **Three length units: `ft`, `in`, `ft-in`.** They are the three ways a US drawing or a
  tape gives a length. Yards and anything else are refused by name.
* **A value is stored exactly as entered: its number and its unit.** `30 in` is the number
  30 and the unit `in`. There is no second, normalised copy in the file or in the world,
  so the two cannot disagree. Conversion to feet happens inside `derive` and the checks,
  nowhere else (`Len::feet`, `Outline::points_ft`, `Run::ends_ft`).
* **Feet-and-inches is the pair `[feet, inches]`**, not text and not a count of inches.
  A pair is numbers a person can read and edit in the file, and inch fractions down to
  1/64 are exact in a 64-bit number, so nothing is rounded on the way in or out. It is
  converted through inches, (feet × 12 + inches) ÷ 12, so `25' 6 1/2"` and `306.5 in` are
  bit-for-bit the same number of feet.
* **A formula shows its conversion.** `plan_length × height ÷ 12` when the height is in
  inches, so nobody reading the page has to guess whether 30 was feet. The cost: changing
  only the unit a value is written in also changes the formula text of what derives from
  it. That is the ruling's "that value and what derives from it". (Figures are still
  worked from unrounded inputs and shown to four decimals, as in D1, so multiplying the
  rounded figures on the page can differ in the last place. A sheet-metal area is the
  exception: it is worked from the length as shown.)
* **A bare number is refused for a length** (`30`), at the keyboard and in the file. It is
  the one input that could silently become feet. A pitch typed bare is accepted, because a
  pitch has only one unit.
* **Plan coordinates take `ft` or `in`, not `ft-in`.** Coordinates are plain numbers; a
  traced outline is in the plan's unit and a shop detail is in inches.
* **Fractions are halves to sixty-fourths.** The ruling says "to the sixteenth"; a tape
  reads to 1/32 and a rule to 1/64, both are exact, and refusing them would be an
  arbitrary limit. `1/3` is refused: it cannot be held exactly.
* **Stretch-out lives on the cost line, and is brought forward from D3** because the
  ruling's fifth demonstration needs it: a stretch-out in inches against a length in feet.
  The brief says the stretch-out is editable on the line; the same material has a
  different stretch-out at every detail, so it cannot live on the material. Only the first
  step of D3's chain is built (LF × stretch-out ÷ 12 = SF); sheets, pounds and waste are
  D3. The derived area is kept in `pl::line_quantities`, by the line's description, so
  **two lines on one element can no longer share a description** (a gap SPEC section 7
  used to list).
* **`set` will not put a stretch-out on a line that already has a per-LF price.** The line
  would change from $/LF to $/SF under a price entered for the other. It is refused with
  the reason; the line is re-priced in the file.
* **The job's `length_unit` stays `ft`** and now means the unit derived lengths are worked
  out in. LF, SF and SQ follow from it.
* **Metric is refused and called metric** (PROPOSALS.md item 5). Degrees and percent for a
  pitch are refused too (item 6): the ruling was about lengths, and nobody asked.

### 17. A roof's lines are read off its planes: write (D2 groundwork)
`crates/world/src/roof.rs`. Whatever produces planes (the skeleton of call 3, or an
estimator entering them) hands them to one place that says where the ridges, hips,
valleys, eaves and rakes are. A shared edge is classed by its slope and by which way each
plane falls from it; an unshared edge by whether it is on the eave line, climbs, or is a
level high edge. It needs no skeleton and no window, so it was built while both were
blocked. Calls made in it:
* **A level top edge on one plane is a Ridge to the planes and may be a Headwall to the
  estimator.** The planes cannot know a wall stands there. The comparison treats the two
  as the same line.
* **A fold where one plane falls toward the line and the other away (a change of pitch)
  is reported, not classed.** There is no kind of line for it yet.
* **Line ends are snapped to traced corners** within the 0.01 ft tolerance, so a found
  line's end is the corner's own coordinates.
* **It changes nothing in the job.** `plumbline lines` reports implied lines against
  entered ones. Replacing entered lines with solved ones is the solver's step, because
  that is where the question of keeping an element's path, cost lines and condition
  through a re-solve has to be answered.

### 18. Low-slope solvers already exist in the owner's earlier program: borrow candidate
Found on the machine this stretch ran on: `roofnerd`, the same owner's earlier estimating
program, MIT (`LICENSE`; `src-tauri/core/Cargo.toml` says `license = "MIT"`), last change
to its core 2026-09-13. Its Rust crate `roofnerd-core` holds solvers with no window, disk
or network in them. Read this stretch, file headers and function lists only:
* `taper.rs` (1,075 lines): the tapered insulation package laid out per drain, citing NRCA
  *Membrane Roof Systems* 2019 ch. 4.
* `cricket.rs` (522 lines): a cricket behind a curb and a saddle between two drains, cut
  at twice the field's slope, held to a length-to-width table, sitting on the solved
  surface.
* `coping.rs` (320 lines): cap width, gauge by face height, sections, corners and cleat,
  citing NRCA *Architectural Metal Flashing* 2022; the stretch-out stays typed against the
  detail.
* `walls.rs` (423 lines): a parapet or curb as a run with a height and a thickness, and
  how high the flashing runs up it.
It depends on `geo` 0.29 and `cavalier_contours` 0.4 (both MIT OR Apache-2.0 per its own
`DEPENDENCIES.md`; not re-checked in the registry here, and neither is linked by this
project yet).
**Call:** borrow, by porting with a NOTICE entry, when D2 reaches "a low-slope area with
tapered crickets" and D3 reaches the parapet's coping and flashing. It was not in the
brief's appendix. **Not done:** nothing has been ported, none of it was built or run
here, and its model (a document of traced conditions with typed properties) is not this
project's (one entity per element), so the port is a re-fit, not a copy. Whether this
project is meant to take over that program's ground is the owner's to say (QUESTIONS.md
Q11).

### 19. Calls made fixing what the third judgment found
* **One line that contradicts a plane's eave takes the length off every line on that
  plane.** An eave that climbs, or a rake that runs level, is evidence about the plane,
  not only about that line. Without this a rake that climbs under both readings kept a
  wrong length marked Known. The cost: one mis-entered line flags its neighbours, which is
  the safe direction.
* **A bare fraction is refused as a pitch.** `3/4` was read as a rise of 0.75 in 12 while
  `6/12` was read as 6 in 12: one notation, two meanings, both stored Known. `a/12` keeps
  its reading; every other fraction has to carry its run (`3/4:12`).
* **A dash before the inches is a separator only hard against the feet mark or with a
  space both sides.** `0' -6"` reads as a minus on the inches and is refused.
* **A derived figure that is not zero but rounds to 0.0000 is Unknown.** Nothing is
  silently zero.
* **Any attribute under `pl::` that is not one of this program's components is refused.**
  It was kept as someone else's attribute, so a pitch under `pl::Pitch` sat in the file
  unread and the plane said it had none.
* **Cost lines are told apart ignoring capitals and spacing**, and a description may not
  start or end with a space.
* **An eave may span an edge traced in pieces**, when every corner inside the run is
  within the tolerance of the one line from the run's start to its end. (First done by
  dropping points one at a time, each tested against its neighbours; the fourth judgment
  showed 400 such points adding up to a 1 ft sag.)

### 20. Calls made fixing what the fourth judgment found
* **A line is asked about each plane it names before the planes are compared, and every
  plane it shows wrong is recorded.** Call 19's rule lost its evidence when the line was
  refused first for the planes disagreeing, and recorded only the first plane.
* **Gaps under a line add up.** The total uncovered length has to stay inside the
  tolerance, not each gap.
* **A key that is a mistyping of one of ours is refused.** First done as "within two
  edits of one of ours", which the fifth judgment showed refusing `ex::cost` and `my::job`
  and calling them ours. Now: under our prefix however it is spelled, or a prefix one
  letter off with one of our names exactly.
* **A description may not hold a character nobody can see.** Look-alike letters from
  other alphabets are not caught; that would take a table of them (SPEC.md section 7).
* **In `lines`, two edges share a stretch only between corners**: an end of one that sits
  on the other. An edge that crosses or touches at a corner shares nothing, and two edges
  a hair out of line share the same stretch whichever is asked first.
* **In `lines`, two planes level along one plan line at two heights are a step, or a
  misfit; the planes cannot say which.** It is said as that, and each plane's own edge is
  listed. Two planes on the same side of a shared edge are an overlap and give no line.

### 21. Model calls made in D3 (ours to make; recorded so they can be overruled)
The pattern is the owner's earlier program's: a line is worked formula, quantity, waste,
order units, each step shown (call 18). Its code was read for the pattern; none of it was
copied. The calls:
* **A line's quantity is a formula, and the D1 form is the simplest one.** A line priced
  by `surface_area` is a formula of one name. There is one field, `quantity`, not a
  measure name and a formula beside it.
* **Units are carried through the formula, so nobody writes `/ 12`.** The earlier program
  wrote `COPING_LF * (STRETCHOUT / 12)` with the stretch-out a bare number of inches. Under
  the units ruling the stretch-out carries its unit and the program converts; the worked
  formula saved beside the figure shows the `÷ 12`. A formula is also checked for sense by
  its units: a length added to an area is refused, and the unit a line is priced in
  follows from its formula and is not typed separately.
* **A formula that comes out as a count is rounded up to a whole number**, and says so.
  26.67 fasteners is 27. Order quantities are rounded only where the line says to.
* **The stretch-out is a field of its own on the line; other named values go in
  `inputs`.** The brief names the stretch-out; a spacing or a number of sides is whatever
  the formula calls it.
* **Every value on a line has to be used by its formula.** A stretch-out or a `sides: 2`
  sitting on a line whose formula ignores it reads as if it counted. Rewriting a formula
  takes off the values it no longer names and reports them; a new name comes in with its
  value in the same edit (`--with`).
* **A condition is an element holding a recipe, and applying it copies the lines.** Every
  element then carries its own lines, which is the brief's "editable on the line", and the
  recipe is in the job file where it can be read. The cost: a recipe changed later does
  not reach elements it was applied to before.
* **The sample recipes leave the stretch-out, the spacing, the waste and the sheet size
  blank.** They are the detail's and the supplier's. A typical value would be an invented
  number one keystroke from a price. The only value the parapet recipe states is
  `sides: 2`, which is the brief's "cleat on both sides". **A stretch-out in a recipe is
  refused outright**; other values a recipe states are allowed, are copied with their
  status and source, and are shown where the recipe is read (fifth judgment: they were
  allowed and hidden, against what this document said).
* **Waste is one percentage per line.** The earlier program keeps a material's waste and a
  condition's waste apart. There is no material library here yet to hold the first.
* **The order conversion carries its direction in its unit**: `SF/sheet` divides,
  `lb/SF` multiplies. Typed as `30 SF/sheet` or `1.25 lb/SF`. A package (the divide form)
  is bought whole unless the line says otherwise; a weight is not rounded.
* **Each step is worked from the step before as shown** (four decimals), so the page can
  be followed by hand.

### 22. Calls made fixing what the fifth judgment found
* **A count is tested for being below zero before it is rounded up.** Rounding first
  turned -40 fasteners into 0, Known.
* **A line's length is not decided until every plane it names has been looked at.** Call
  20's rule still lost its evidence when the line was refused early for another plane.
* **An element is one condition.** Applying a second recipe to an element that already
  carries another condition's name is refused; it used to write over the name.
* **A formula holds at most 200 tokens.** A 2,500-term one ran every command out of stack.
* **A change in the unit a line comes out in, or is priced in, is said when it happens.**
  `--with spacing=12` (no unit) makes the spacing a plain number and the line a length
  instead of a count. It cannot be refused, because a new name's kind is whatever is
  typed; it is reported.
* **Every command tolerates a reader that stops listening.** `set ... | head -1` was a
  panic.

### 23. Decisions made on the way to the 3D prototype (the owner handed them over)
On 2026-11-10 the owner said: make the decisions yourself until there is a working 3D
prototype. These had been his (QUESTIONS.md Q4, Q7-style approvals, Q10). Each is
recorded so he can overrule it.
* **Compiler: rustc 1.97.1, pinned in `rust-toolchain.toml`.** Bevy 0.20.1 asks for it.
  Building against 1.97.0 with cargo's version check overridden also compiled, but the
  supported compiler was one command away, so that is the one used.
* **System packages installed in the workspace: `libxkbcommon-x11-0`, `xdotool`,
  `mesa-vulkan-drivers`.** The first is what the window needs to start on X11. The other
  two are for running the demonstration with no desktop: a pointer and keyboard that the
  program cannot tell from real ones, and a software Vulkan driver. A desktop with a
  graphics card needs only the first, and usually has it.
* **BIMCraft's skeleton is ported, not rewritten** (call 3). Read access to its
  repository turned out to be available; the file was cloned, read in full, and ported
  with its tests.
* **The sheet is drawn with Bevy's own text, not `bevy_egui`** (call 2 said `bevy_egui`).
  The prototype's sheet only shows. Bevy's text needed no second library and no question
  of whether `bevy_egui` 0.43 fits Bevy 0.20. When the sheet takes input (cells, formulas
  edited in place), this is decided again with that need in hand.
* **The orbit camera is ours**: thirty lines, right-drag and scroll. Bevy 0.20 ships a
  pan-orbit camera; it wants an input plugin of its own, and the prototype did not need it.
* **Wayland is left out of the build.** It needs libwayland's development files at
  compile time. X11 runs on a Wayland desktop too.
* **A roof solved from a footprint is a new component (`pl::footprint`) beside D1's
  hand-entered planes, not a replacement for them.** Both are jobs the program opens.
  What the solver makes is marked `pl::solved`, and it replaces only what carries that
  mark.
* **A solve keeps identity by place, then by number** (first done by edge number alone;
  see call 24). A line is known by its kind and the edges of the planes it lies between
  (`hip 1-2`). What no longer exists is removed and reported, with a warning if it carried
  a condition or cost lines.
* **A solved plane's outline takes the worst status in the whole footprint**, because one
  pitch moves every plane's outline. So one assumed pitch flags the whole roof.
* **A value set in the 3D view is Assumed**, with its source saying where it was set. The
  estimator chose it and has not said where it comes from; rule 4 does not let that be
  Known.
* **Solved coordinates are written to a millionth of a foot**, so a re-solve that changes
  nothing writes the same bytes.
* **Demo output moved from `evidence/` to `demos/captured/`.** The repository this lives
  in ignores any folder named `evidence`, for its own good reasons; the captures here are
  of made-up jobs and are meant to be tracked.

### 24. Calls made fixing what the sixth judgment found
* **What is solved is derived, so it is solved again on every load.** A footprint edited
  in the file left the planes of the earlier solve standing as Known. Now the planes and
  lines are recomputed like measures, and a roof that cannot be solved has the geometry
  taken off its planes and lines; they are not left looking true.
* **A plane keeps the slope it is on, not the number of its edge.** Identity by edge
  number moved a plane's condition to another slope when a point was added to the
  footprint. A plane is now matched to the edge its eave lies along; by number only for an
  edge moved square to itself.
* **A line on one plane is named for the footprint edge it stands over** (`rake 1 at 4`).
  Two rakes on one plane were told apart by which the solver found first, and swapped.
* **`PitchBreak` is a kind of line.** Mixed-pitch roofs have creases where one slope falls
  to the line and the other on from it; they had no line and no kind. The trade calls it a
  change of pitch or a pitch break.
* **Planes that share an edge on plan at different heights each keep an edge there**, with
  a note on the roof. With a gable beside an inside corner that is a wall, and it was left
  with no line at all.
* **A footprint has to be clean to be solved**: no edge under 0.02 ft, no corner a hair
  off straight, no pitch under 0.01 in 12. Each made slivers, invented hips, or dropped
  eaves. A hand trace will have these and will be refused with what to do; when tracing is
  built it has to clean its own output (QUESTIONS.md Q9 is the same question).
* **An edit that fails puts the job back from a saved copy**, not by undoing its steps.
* **A condition given in the 3D view is Assumed**, like every other value set there.
* **Not done:** a slope pinched by gables into two pieces joined at a point is refused,
  as courtyards are. A plane entered by hand under a solved roof is noted, not checked for
  overlap.

### 25. The windows drawn again (owner, 2026-11-11: the prototype "looks cheap and tacky")
The first window was a dark screen with one block of monospaced text over it and a second
block of text for a sheet. The owner asked for the toolkit's real use to be looked at and
for work of a higher standard. What was looked at, and what was decided from it:
* **What a roof is shown as, in this trade.** The measurement reports estimators already
  read (EagleView, RoofSnap, GAF QuickMeasure) all show the same things: a plan diagram
  with each facet labelled with its area and pitch, each line's length written on it, the
  lines coloured by kind, totals by kind of line, the predominant pitch and the facet
  count. None of that is ours to invent, so the windows now show those. RoofSnap's own word
  for the line where one slope changes to another is "pitch change"; the windows use it
  for `PitchBreak`. No vendor's colours could be confirmed from a public source, so the
  colours here are ours and **every window carries a legend**.
* **What the engine offers.** Bevy 0.20 has real-time shadows, anti-aliasing, distance
  fog, a layout engine with rounded corners, borders and shadows, picking for panels as
  well as meshes, and OpenType text with tabular figures. The first window used none of
  them. It now uses all of them. `bevy_feathers` (Bevy's own widget set) was read and not
  used: it is styled for a dark editor and is marked experimental.
* **A light drafting look, not a dark game look.** A pale canvas, white panels with a
  hairline edge, ink-dark text. Colour is kept for meaning and means one thing each:
  amber is Assumed, red is Unknown, blue is the selection, and each kind of line has one
  colour in the drawing, the legend and the sheet.
* **A plane's colour is what it is covered with, not its status.** The first window
  painted a whole plane gold, red or tan by status, so one assumed pitch turned the roof
  into a warning sign and the copper could not be seen. Status is now on the figure it
  belongs to: an amber or red figure, a chip that says Assumed or Unknown, a dot on the
  facet's label. A plane with no area at all is still drawn in a red wash.
* **The roof sits on something.** Under each eave and rake a fascia and a wall are drawn
  down to the datum, on a ground with a ten-foot grid and a shadow. **They are for the eye
  only**: not elements, not saved, not measured, not selectable. The ground has no edge:
  it is one sheet in a colour that comes out as the canvas when lit.
* **Two views of the one roof: 3D, and plan.** `T` looks straight down without
  perspective, north up, with every line's length written on it: the diagram a roof report
  gives. It is the same scene and the same labels from another eye, not a second drawing.
* **An inspector beside the view.** With nothing selected it summarises the roof; with a
  plane or a line selected it lists every value on it with its flag and, under each, its
  source or the formula it is worked out by, and the plane's corners to the sixteenth.
* **The sheet is a sheet.** Summary figures, then a table of planes and a table of lines
  with figures in aligned columns, then the selected element's cost lines step by step.
  A click on a row selects it in the 3D view. It still takes no input (call 23).
* **Typeface: Inter, built into the program** (SIL Open Font License 1.1; QUESTIONS.md
  Q12). Three weights, 1.8 MB. Figures are set tabular so columns line up. The program
  must look the same on a machine with no fonts installed, so it is not left to the system.
* **A window may round a figure but not round it away.** Figures show to two places. One
  that is not zero and would print as zero is shown to as many places as it takes
  (`view::figure_shown`). The model still keeps four.
* **The windows still work nothing out.** What was added to `plumbline-world` for them:
  the facet count and predominant pitch on `view::roof_totals`, a surface said in squares
  (`view::in_squares`), and each derived row's figure as a number so a view can set its
  own decimal places. Where a label is placed on the screen is worked out in the window;
  no quantity is.
* **Added to the keys:** `T` plan or 3D, `F` frame the roof, `Esc` let go of the
  selection, middle-drag to pan, and a click on the ground lets go.
* **Not done:** the windows are still not judged on a real graphics card or a real
  desktop; everything above was run on the virtual display.

### 26. The whole building, not the roof alone (owner, 2026-11-11)
The owner looked at the redrawn roof window and said: better, but think of this for a
whole house with its MEP; you are limiting yourself; see what is out there now and feel
free to build your own; it should feel like a video game, with a game's smoothness and
graphics, and have the power of BIM and Revit. That widens the brief's "proven on the
building envelope first" to the building and its services, and it is his to widen.

**What was looked at.**
* *Where the industry is.* Revit holds the model and its data; the picture people enjoy
  comes from sending that model on to a game engine (Twinmotion, Unreal, Unity Reflect)
  to look at, with the authoring tool still the record. Modumate put authoring itself on a
  game engine. Newer tools (Motif, Qonic, Arcol and others) are rebuilding BIM in the
  browser. The gap the owner is pointing at is real: the data and the good picture are
  still two programs. This program has one world, and the window is already a game
  engine, so it does not have to choose.
* *What gives Revit's MEP its power*, by the accounts of the people who use it: runs are
  elements with sizes, runs and equipment belong to named **systems** with a
  classification, and schedules and takeoffs come out of the model, not out of a drawing.
  That, not the drafting, is what was built here first.
* *What the engine has.* Bevy 0.20 has a physical sky that lights the scene, ambient
  occlusion, bloom, temporal anti-aliasing and cascaded shadows. They were read in its own
  examples and used.

**What was decided.**
* **The building's elements are in the world, with the same rules as the roof**
  (`building.rs`): storeys, walls with doors and windows, floors, ducts, pipes, conduit,
  cable, equipment, fixtures, and systems. Every value flagged and sourced; every figure
  worked out in the world and refused, with the reason, when its inputs do not fit.
* **The names are IFC's and Revit's, not ours.** Storey (Revit says level), wall, opening,
  floor (IFC says slab), duct, pipe, conduit, cable, equipment, fixture, system, and
  Revit's system classifications (Supply Air, Domestic Hot Water, Sanitary, Power). IFC's
  own words for the last three kinds are longer (distribution flow element, terminal);
  the trade's are used.
* **A run is its centre line in space and a size.** Its length is measured along the
  centre line, so a riser counts. Each corner is a bend: a fitting to be counted. A duct's
  surface is its length by the distance round it. Fittings are not yet elements of their
  own, and nothing checks that runs actually join.
* **Heights for runs and placed things are above the job's datum, not above their storey.**
  A riser passes through storeys, and a point read against the wrong one is a wrong point.
  Walls, floors and openings stand on their storey.
* **A system is an element**, so it can be selected, read and totalled. A takeoff is the
  system's runs by kind and size, each line a sum that says what it could not include; a
  run with no size is not dropped and not guessed: it is on a line of its own, "size
  unknown", with its length.
* **A material is a condition**, as it is on the roof. "Type L copper" on a pipe is the
  same thing as "standing seam copper" on a plane: a name now, a recipe with cost lines
  when one is written. Cost lines on a run can already use its `true_length` and `bends`.
* **Colours of systems follow mechanical drawings**: supply air blue, return air pink, hot
  water red, cold water cyan, waste brown, vent green, power yellow. So red means hot
  water on a pipe and Unknown on a figure. A run with no size is drawn as a broken red
  line, which no system's solid colour is.
* **The window is one model view, not a roof view.** Everything in the job is drawn at
  its own size and can be clicked. A browser lists storeys, disciplines and systems; each
  can be shown alone. "See through" draws the building as glass with its edges kept, so
  what runs inside can be seen and clicked. A selected system keeps its colour and
  everything on other systems goes grey.
* **It moves like a game.** The camera eases toward where it is sent instead of cutting.
  W A S D move over the ground. `V` goes in on foot at eye height: the same world, from
  inside. `T` is a true plan, of the whole building or of one storey.
* **It is lit like a game.** A sky, a sun, shadows, ambient occlusion, and light through
  the windows onto the floor. `PLUMBLINE_GRAPHICS=plain` draws without them, for a machine
  with no graphics card. The panels float over the picture.
* **Sizes on the sample that a designer would work out are Assumed and say so** ("sized by
  rule of thumb; no load calculation has been done"). The sample house is drawn, not
  engineered, and the program does not pretend otherwise.
* **Not built, and next:** fittings and connections (so a system can be checked for
  continuity and its fittings counted by type); clash detection between runs and
  structure; rooms; structure itself; editing any of this in the window (it is read and
  walked, not yet drawn); IFC in and out (D6); cost recipes for MEP.

### 27. Calls made fixing what the eighth judgment found
* **A storey is a floor of the building, directly under the job.** Nothing else is a
  storey's parent. And a total counts each wall and floor on its nearest storey only, so
  a job built in code that breaks the rule still cannot count anything twice.
* **Not knowing which system is not the same as being on none.** A takeoff has two groups
  after the systems, each only when it has something in it: "On no system", which is a
  Known fact about what is in it, and "System not known", which is flagged Unknown. A run
  in either keeps its length. Neither is a system and neither can be selected as one.
* **A takeoff line is no surer than the membership of what is on it.** A run assumed to be
  on a system makes its line, and the system's total, Assumed.
* **A size is a size, however it was typed.** Runs are grouped by their size in feet and
  the line is said in inches, the way pipe and duct are sold. The value on the run itself
  is still held and shown in the unit it was entered in (call 16); only the takeoff's
  grouping converts.
* **An opening that is not cut is said.** It is in the not-drawn list with why. Its wall is
  still drawn, whole where the opening would have been.
* **A flat reading of the job is in tree order**, not the order things were added in. The
  saved file is still written in the order things were added, so a save changes nothing
  it did not have to.
* **A one-line bar shows one line.** A message is cut to fit it and shown whole in the
  card over the model. It is formatting; nothing is hidden that is not shown elsewhere in
  the same window.

### 28. What was already on the machine and was not read (owner's question, 2026-11-11)
The owner asked whether the whole-building work shows what this seat can do, or whether
it leans on him instead of searching. It leaned on him. This entry is the search that
should have come before call 26, done after it.

**What was skipped.**
* **BIMCraft was on this machine, already rated "borrow" (call 4), and was not opened
  before `building.rs` was written.** Only its skeleton file had ever been read. A survey
  of its model crate (an Opus agent's reading; the struct names, the `network` function,
  the IFC class table for runs and the test count below were then checked by hand) found:
  walls with location lines, layered types, corner joins, base and top levels and offsets
  (`model.rs:249`); doors and windows hosted in walls and cut in 3D, sections and IFC
  (`model.rs:827`), with the past-the-top case this project's eighth judge found handled
  and tested; levels (`model.rs:131`); rooms found from their bounding walls, with area
  and height (`rooms.rs`); one `Run` for pipe, duct, conduit, cable tray and cable, with
  its size on a type (`model.rs:1588`, `2467`); fittings as components with connectors,
  and a connectivity graph with open-end warnings (`network.rs:163`); schedules; IFC4
  export (IfcBuildingStorey, IfcWall, IfcSlab, IfcOpeningElement, IfcPipeSegment,
  IfcDuctSegment, IfcCableCarrierSegment, IfcCableSegment, IfcSystem); a section box;
  plan and section drawing; drawing and snapping tools. 1,648 tests in the crate.
* **Two things call 26 lists as "not built, and next" already exist there**: fittings and
  connections, and editing tools.
* **The web was searched four times per push, and thin results were not followed up.**
  Call 26 named Qonic and Arcol from memory; the search behind it had returned nothing on
  either. Both have since been looked up and the sentence stands (Qonic: browser BIM with
  clash detection, quantities and, from April 2026, drawings; Arcol: browser massing and
  early design, launched June 2025, not a documentation tool).
* **"The names are IFC's and Revit's" was written from memory.** Checked since, for IFC:
  storey is IfcBuildingStorey; floor is IfcSlab; duct, pipe and cable are IfcDuctSegment,
  IfcPipeSegment and IfcCableSegment; a conduit is an IfcCableCarrierSegment; a fixture is
  an IfcFlowTerminal; a system with a kind is IfcDistributionSystem with a PredefinedType.
  Not checked: that the ten classification names in `building.rs` are the values of
  IfcDistributionSystemEnum or of Revit's list. They are what was remembered of Revit's.
* **Both jumps in this window's quality came from the owner saying it was not good enough,
  not from looking first.**

**What BIMCraft does not have, and this project does.** A status and a source on every
value, carried through to every figure (it has none). Net wall area (its wall area is
length by height). Systems as elements with a discipline and a classification (its system
is a name on each run). A takeoff by system that says what it could not size. Clash
detection is in neither. Its sizing and code checks follow Brazilian standards (NBR 5626,
8160, 5410, 10844) and are not usable here; its ducts are thin.

**What is decided.**
* **`building.rs` is not extended further as it stands.** Before the next stretch (drawing
  in the window, fittings, connections), the building model is re-based on BIMCraft's
  shapes, ported with attribution as the skeleton was (call 3): the wall with its location
  line and levels, the hosted opening, the run with its type, the connector and the
  network, the room. This project's flags, sources, refusals, net area, systems and
  takeoff are kept on top of them. Call 4 stands: borrow from, do not build on, because
  its window is egui and its world is not an ECS.
* **Its IFC4 writer is the starting point for D6**, not a new one.
* **Its Brazilian sizing is not ported.** Sizing for this market is a separate question
  for the owner when it is reached.
* **The search order in the parent repository's instructions applies here**: what is on
  the machine first, indexes first; then the scored landscape (this file's own table);
  then the web, with a thin result followed up before anything is written from memory.


### 29. The building model re-based on BIMCraft: what each thing becomes (2026-11-11)
The mapping asked for by `HANDOFF_REBASE.md`, written before any model code. BIMCraft is
at `f202abd85d1c` beside this repository; paths are under its `crates/core/src/`.

**What was read, whole unless a range is given.** `network.rs` (428 lines); `runs.rs`
(719); `rooms.rs` (252); `walls.rs` 1 to 420 (the axis, the faces, the arms round a node,
the caps, the centre line, the outline); `geom.rs` 336 to 620 (line crossings, the split,
the faces of a planar graph, `Region`, `region_in`); `family.rs` 28 to 160 and 300 to 510
(categories and their kinds, connectors); `model.rs` 124 to 180 (`Level`), 240 to 300
(`Wall`), 340 to 440 (`Location`, `Function`, `Layer`), 560 to 606 (`Floor`), 806 to 905
(`Opening`, `Room`), 1540 to 1637 (`Justify`, `Run`, `Component`), 2110 to 2175
(`Element`), 2405 to 2505 (the type structs), 2590 to 2622 (`Types`), 2680 to 2722
(`RoomArea`), 3300 to 3345 (`component_connectors`), 3780 to 3945 (levels, a wall's base
and top, what bounds a room), 4168 to 4200 (`opening_dims`), 4990 to 5015 (`run_geom`),
5784 to 5947 (bounding lines), 5860 to 5880, 6660 to 6720 (the check of a door or window
in its host); `schedule.rs` 940 to 975; `openings_past_top_tests.rs` 1 to 60;
`ifc/classes.rs` 1 to 25 and its fitting rows; `docs/areas/structure-and-mep.md` for the
network and connectors.

**The three things the handoff listed as not known.**
* *Are the ten system classifications IFC's?* **No.** IFC4 ADD2 TC1's
  `IfcDistributionSystemEnum` has 44 values: AIRCONDITIONING, AUDIOVISUAL, CHEMICAL,
  CHILLEDWATER, COMMUNICATION, COMPRESSEDAIR, CONDENSERWATER, CONTROL, CONVEYING, DATA,
  DISPOSAL, DOMESTICCOLDWATER, DOMESTICHOTWATER, DRAINAGE, EARTHING, ELECTRICAL,
  ELECTROACOUSTIC, EXHAUST, FIREPROTECTION, FUEL, GAS, HAZARDOUS, HEATING, LIGHTING,
  LIGHTNINGPROTECTION, MUNICIPALSOLIDWASTE, OIL, OPERATIONAL, POWERGENERATION, RAINWATER,
  REFRIGERATION, SECURITY, SEWAGE, SIGNAL, STORMWATER, TELEPHONE, TV, VACUUM, VENT,
  VENTILATION, WASTEWATER, WATERSUPPLY, USERDEFINED, NOTDEFINED. Read from the
  enumeration's declaration in IfcOpenShell's `src/ifcparse/schemas/Ifc4-schema.cpp` at
  `75390dab` (its header: "generated from IFC4_ADD2_TC1.exp"; the file was read, nothing
  of it is copied or linked), with each value's definition from buildingSMART's own
  `IFC4.3.x-development` at `424cadb1`,
  `docs/schemas/shared/IfcSharedBldgServiceElements/Types/IfcDistributionSystemEnum.md`.
  buildingSMART's published pages refuse an automated fetch (HTTP 403); BIMCraft does not
  hold the enumeration (`grep -rn DOMESTICCOLDWATER` over its tree: nothing), and writes a
  plain `IFCSYSTEM` (`ifc.rs:1544`).
  This program's ten are names an estimator reads, and stay. Six have a value of IFC's
  that says the same thing: Exhaust Air is EXHAUST, Refrigerant is REFRIGERATION,
  Domestic Cold Water is DOMESTICCOLDWATER, Domestic Hot Water is DOMESTICHOTWATER, Vent
  is VENT, Lighting is LIGHTING. Power is ELECTRICAL ("a circuit for delivering electrical
  power"). IFC does not split supply air from return air: both are AIRCONDITIONING
  ("conditioned air distribution"), and the split is kept here as the name. Sanitary is
  SEWAGE ("sewage collection system"). The IFC value is what D6's export will write; it
  is not in the code until the writer is.
  *Are they Revit's?* **Partly checked.** Autodesk's API reference has an enumeration
  `MEPSystemClassification` whose members include `SupplyAir` ("a supply air duct system")
  and `ReturnAir`
  (https://help.autodesk.com/cloudhelp/2026/ENU/Revit-API-MainReference/files/html/43ec0d75-d6bb-2d08-a920-9715e83040e7.htm,
  as a search returned it; the page itself would not load here). The other eight were not
  seen there. Call 26's sentence that the names are "Revit's system classifications" is
  therefore withdrawn for those eight: they are the trade's words, and nothing more is
  claimed for them.
* *What does BIMCraft do where a run meets another with no fitting?* End to end, they are
  one node and nothing is said (`network.rs:206` to 237: points within the tolerance are
  one node; a node is open only when one thing is at it, `network.rs:416`). An end on
  another pipe's length is a node that splits that pipe into two edges without splitting
  the element, and a warning, `joined_mid_run` (`network.rs:238` to 281, 356 to 371). It
  never refuses a model.
* *How do door and window types carry width and height?* `OpeningType { width, height,
  sill, operation }` (`model.rs:2493`), held by name in `Types.door` and `Types.window`
  (`model.rs:2598`); an opening of a family takes them from the family's `Width` and
  `Height` instead (`opening_dims`, `model.rs:4174`).

**What each thing becomes.** BIMCraft holds types in maps by name and points at levels and
hosts by Guid. Here a thing is an element with a path, and its place in the tree says what
it stands on (kept: handoff section 7, item 8). So a BIMCraft `level` or `host` field is
the element's parent, and a BIMCraft `type` name is the path of a type element.

| BIMCraft | Here | What changes in the port |
|---|---|---|
| `Level` (`model.rs:131`) | `pl::level` on a storey: `elevation`, `computation_height`; `height` (the storey's, ours) stays | view range, crop, extent, graphics are drawing and are not taken |
| `WallType`, `Layer`, `Function` (`2417`, `421`, `408`) | a new kind of element, **Type**, carrying `pl::wall_type`: `thickness`, `layers` | a layer's id and Guid go (a path names things here); `variable` is a floor's and is not taken |
| `Wall` (`249`) | `pl::wall`: `type`, `line` (its `start` and `end`, one flagged value), `top_level`, `height`, `base_offset`, `top_offset`, `location`, `room_bounding`, `join` | `arc`, `attach_top`, `attach_base`, `params` are not taken (PROPOSALS 7); `level` is the parent storey; thickness moves to the type |
| `OpeningType` (`2493`) | `pl::opening_type` on a Type: `category` (door or window, the name of BIMCraft's map), `width`, `height` | `sill` (a default for drawing) and `operation` are not taken |
| `Opening` (`827`) | `pl::opening`: `type`, `offset` (to its **centre**, from the wall line's start, as BIMCraft has it), `sill` | `host` is the parent wall; width and height move to the type; `family`, `mark`, the two flips and `params` are not taken |
| `FloorType` (`2426`), `Floor` (`567`) | `pl::floor_type` on a Type; `pl::floor`: `type`, `boundary`, `offset` | openings in a floor, slope, shape and sketch are not taken (PROPOSALS 7) |
| `RunType` (`2467`) | `pl::run_type` on a Type: `category` (the key of BIMCraft's `runs` map), `width`, `depth`, `round` | `material` is not taken: a material here is a condition (call 26) |
| `Run` (`1588`), `Justify` (`1551`), `runs::Geom` (`runs.rs:81`) | `pl::run`: `type`, `line` (plan `start` and `end`), `offset`, `end_offset`, `justify`, `join`. **One straight piece.** Heights are above its storey, as BIMCraft's are above its level | `system` stays a component of its own pointing at a system element (kept: item 5); `params` not taken |
| `Component` (`1616`) | `pl::placed` on equipment, a fixture or a fitting: `what`, `at` (plan), `offset` above its storey, `angle`, `size` | `family` and `type` become `what` and `size`: families as a system are outside the fence; `mirrored` and `params` not taken |
| `Connector`, `ConnectorKind`, `Domain`, `Flow` (`family.rs:430`, `350`, `367`, `380`) | `pl::connectors` on the placed thing itself: `name`, `kind`, `domain`, `flow`, `at`, `size` | on the element, not on a family; `at` and `size` are values, not formulas; a kind is added for ducts and one for conduit, which BIMCraft's network does not have yet (its doc: "Conduits and the other electrical runs are not in it yet"); the electrical load fields are sizing and are not taken |
| pipe fitting kinds (`family.rs:132`) | a new kind of element, **Fitting**, carrying `pl::fitting`: `kind` of bend, tee, cross, transition, coupling | the same five for every kind of run; IFC4 has the same list for pipe and duct fittings (`ifc/classes.rs:140`, `244`) |
| `network` and its types (`network.rs`) | `network`: nodes, edges, groups, open ends, warnings, for the runs of every kind | lengths in feet; a tolerance restated in feet; a run's system is a path; each kind of run joins its own kind; nothing here is a refusal there, and nothing is here |
| `Room`, `RoomArea`, `Unbound` (`852`, `2687`, `303`), `rooms.rs`, `geom::faces`, `Region`, `region_in` | a new kind of element, **Room**, carrying `pl::room`: `point`, `number`. Its boundary is the smallest loop of bounding walls round its point, less islands; its area and perimeter are figures | found among every wall each time (BIMCraft's own tested twin of its indexed search); `frozen`, `kept`, `colour`, heights not taken; **both** measures are given as figures (inside the wall faces, and to the centre lines) where BIMCraft has a project setting |
| wall joins (`walls.rs:213` to 394) | the arms round a node, the cap, the centre line run on to its neighbours', the outline | straight walls only; no index; the compound butt corner (`walls/compound.rs`) is drawing and is not taken |

**Three kinds of element are new: Type, Fitting, Room.** The fence allows a room and what
fittings need. A type is the third, and is there because the fence's own "run type" has to
live somewhere, and in this world the only place anything lives is an element: it has a
path, each of its values is flagged and sourced, and it is saved like anything else.

**What is ours on top, and stays** (handoff section 7): every value a `Val`; lengths held
as entered, so BIMCraft's millimetres are not adopted and each of its tolerances is
restated in feet with a test on it; every figure worked out in the world with the worst
status of its inputs; net wall area; systems as elements; the takeoff by system; what is
not drawn listed with why; paths; the saved job.

**Where this program's rule differs from BIMCraft's, on purpose.**
* *An opening taller than its wall.* BIMCraft cuts the wall to its top and lets the
  opening keep its size (`model.rs:6688`, `openings_past_top_tests.rs`), as Revit does.
  Here the wall is refused a net area and the opening is listed as not cut, with the
  reason (eighth judgment). That stands: net area is this program's figure, not
  BIMCraft's, and a window taller than its wall in a takeoff is far more often a slip
  than a notch. The gross face still stands.
* *A run with no type.* BIMCraft has no geometry for it (`run_geom` is `None`,
  `model.rs:4998`). Here it still has its length, and is on a takeoff line of its own
  (kept: item 6).

**What a run becomes, and what happens to the figures.** A `pl::route` of n points becomes
n − 1 runs, each one straight piece, and a bend fitting at each corner. A fitting is
placed at the joint: pipe and duct are measured through their fittings, centre to centre,
so **each system's length does not move**. The count of runs goes up to the count of
straight pieces. "Bends", which was a count of corners in a centre line, goes: what it
counted is now the bend fittings, which are elements. A size is entered once, on a type,
and a takeoff line is the runs of one type's size.

**Tolerances restated.** BIMCraft's `TOL` is 1 mm (`model.rs:47`) and its network
tolerance 1 mm (`network.rs:23`). Here both are this program's fit tolerance,
`FIT_TOLERANCE_FT`, 0.01 ft (about 1/8 in), which is already what a line has to fit its
planes by (Q9). BIMCraft's cut height for finding a room's walls, 1200 mm
(`model.rs:43`), is 4 ft here: a decision, and a level can say otherwise.

**As built (the same day), where the mapping above was changed in the doing.**
* *BIMCraft's `Run` is `Segment` here, `pl::segment`, and its `RunType` is `SegmentType`,
  `pl::segment_type`.* The table above says `pl::run`. That key and the name `Run` were
  already a roof line's plan run in this program (`components.rs`), which was found when
  the code was opened. A segment is IFC's word for the same thing (BIMCraft `runs.rs:25`
  to 37 writes a pipe as IFCPIPESEGMENT, a duct as IFCDUCTSEGMENT, conduit as
  IFCCABLECARRIERSEGMENT, a cable as IFCCABLESEGMENT), so nothing was coined. In what a
  person reads, a run is still a run.
* *A cable turns a corner with no fitting.* The old "bends" counted a cable's corners
  with everything else's. A corner of a duct, a pipe or a conduit is a bend fitting; a
  cable's is not, so the twelve bends the sample's two cable lines counted became no
  element.
* *A run and a placed thing have to be under a storey*, since their heights are now above
  one. A job with a run directly under the job element does not load.
* *A run's `join` was not taken after all*: in BIMCraft it shapes the mitred plan outline
  of runs that meet, which is drawing. Whether runs are joined is the network's to say.
* *Room figures.* Three are given: the floor inside the wall faces, the area to the wall
  centre lines, and the distance round at the faces. *A wall that cannot be placed* is
  ours to rule on, BIMCraft having no such thing as a wall with an unknown height: under
  the room's own storey it leaves the storey's rooms with no figure; under another it
  makes them Assumed and the figure says what was assumed. Leaving such a wall out and
  saying nothing would let two rooms run together into one area marked Known.
* *A total is the sum of its lines as shown.* A storey's rooms come to 1,057.9532 SF,
  which is the five figures as each is written, to four decimals, added up; the unrounded
  areas add to 1,057.953125. This is how every total in the program already worked
  (`view::sum_of`); it is written down here because a hand check met it.
* *A wall that cannot be drawn stays in its storey's table in the sheet.* It used to be
  filed under no storey and so shown in none: a row dropped without a word. Found while
  adding rooms to the same table, and fixed.
* *Flags on the network's rows.* An open end, a join with no fitting and a warning are
  facts about the job as entered, so the windows show them unflagged. Only what could not
  be checked is flagged, Unknown.

### 30. Model calls made in D4, macro to micro (ours to make; recorded so they can be overruled)
Written before the code, 2026-11-11. The demonstration is in STATUS.md, "D4. How its
demonstration will be run". What was read: this program's `lines.rs`, `derive.rs`,
`check.rs` (`cost`) and `job.rs` (`set_on_line`), which is where a unit cost already sits
on every line with nothing multiplying by it. Nothing outside this repository is needed
or was consulted: an extension is a quantity times a price.

* **A line's extension is what it is priced in, times its unit cost.** A line is priced
  in its order unit when it has one, else in the unit its quantity comes out in
  (`lines::priced_in`, already there). So the extension is `order × unit_cost`, or
  `with_waste × unit_cost` when there is waste and no order unit, or `quantity ×
  unit_cost`. It is a fourth figure of the line, beside quantity, with_waste and order,
  worked out and never typed.
* **Money is in cents.** An extension is rounded to the cent. A total is the sum of the
  extensions as shown, so adding the lines up by hand gives the total. An amount that is
  more than nothing and less than half a cent is refused, not written as $0.00.
* **A line with no price has no extension, and says so.** It is not $0.
* **An allowance is a rate on the element** (`pl::allowance`): dollars per SF or per LF,
  with its status and source. What it multiplies is the one figure a thing of its kind is
  bought by: a roof plane's or a low-slope area's `surface_area`; a roof line's
  `true_length` (a parapet's `plan_length`); a wall's `net_area`; a floor's `plan_area`;
  a room's `floor_area`; a run's `true_length`. The rate's unit has to be the figure's
  (`$/SF` for an area, `$/LF` for a length), or it is refused.
* **What an element comes to, both ways, is on the element** (`pl::priced`, worked out,
  never typed): `allowance` ($, the figure times the rate); `detail` ($, its lines'
  extensions added up); `detail_rate` ($/SF or $/LF: the detail divided by the same
  figure the allowance is priced on); `difference` ($, detail less allowance) and
  `difference_percent` (of the allowance).
* **A detail that is not whole is not rolled up.** If any line of an element has no
  extension, the element has no `detail`, no `detail_rate` and no `difference`, and the
  reason names the line. A part-sum would look like a low price.
* **Every figure takes the worst status of what it was worked out from**, as everywhere:
  an assumed pitch makes an assumed area, an assumed allowance amount and an assumed
  difference.
* **The job's totals say what they leave out.** The allowance total is over the elements
  that have an allowance and says how many elements that could carry one have none. The
  detail total is over the elements whose detail is whole and names those whose detail is
  not. Each total says how many dollars of it rest on anything Assumed. The two are
  reconciled over the elements that have both figures, and only those.
* **Cost codes are not used to group anything yet** (QUESTIONS.md Q1 is still open).
* **Entering them.** `set <path> allowance <rate>` on an element and `set <path> unit_cost
  <price> --line "<line>"` on a line. The unit is the one the element or the line takes;
  typed (`$4.25/SF`), it has to be that one. `price <file>` prints the reconciliation.

### 31. Calls made fixing what the ninth judgment found (2026-11-11)
* **A room is measured again with each uncertain wall taken away and standing full
  height.** A wall bounds a storey's rooms when it stands at the height they are found
  at. When its base or its top rests on anything less than Known, that is no surer than
  they are. Folding every such wall's status into every room would flag rooms a garden
  wall could never bound. So for each such wall the storey's rooms are worked out twice
  more, with the wall gone and with it standing through every height, and a room whose
  area or perimeter comes out differently either way takes that wall's status and says
  which wall. A room that comes out the same both ways does not depend on it.
* **A wall's heights have a status of their own**: everything its base and its top rest
  on (its storey's elevation, its base offset, its height or its top level and offset).
  A wall with an entered height used to give only the height's.
* **A Wall element with nothing entered on it is a wall that cannot be placed**, and
  leaves its storey's rooms with no figure, like one with no height.
* **A bounding wall is joined to every wall it meets, bounding or not**, as BIMCraft joins
  it (`rooms.rs:62`, `walls.rs:238`). The port had joined bounding walls to each other
  only; the difference was not listed and moved an area by an eighth of a square foot.
* **A room whose point is inside a wall has no area to centre lines either.** BIMCraft
  gives it one (its centre measure does not look at wall bodies). Here which side of the
  wall was meant is not guessed. Listed as a change in `building/rooms.rs`'s header.
* **A wall's opening area is as sure as the openings' sizes; its net area is as sure as
  where they are too.** An opening with a size and no known place still counts in the
  opening area, and the net is refused because the fit cannot be checked.
* **An open connector is listed where its run would be.** A pipe connector says what it
  carries; one that meets nothing is listed on the systems that carry that. Any other is
  listed on the system the thing itself is on.
* **A wall no longer than the fit tolerance is refused**, as BIMCraft refuses one no
  longer than its own. **A wall whose top is not above its base still loads**, where
  BIMCraft refuses the model: its figures are refused with the reason. Whether a top is
  above a base depends on two storeys, and a job in which one is being changed has to
  stay open. Listed as a change in `building/model.rs`'s header.
* **`tools/portmap.sh` now runs every search an "ours" row says came back empty.** It had
  only checked that a search was named, and two rows named searches that find something.
  It was run red on those two rows before they were corrected. This tightens the gate.
* **Correction to calls 28 and 29.** They say BIMCraft has no takeoff by system. It has
  schedules of pipes and conduit with System, Length and Diameter that can be grouped and
  totalled (`schedule.rs` 368 to 398, 1036 to 1100), which is a takeoff's substance, and
  a named electrical distribution system (`electrical.rs:75`). What it does not have is a
  status on a line, a run with no size kept on a line of its own, or a system that is an
  element. And `MAX_CONNECTORS` is BIMCraft's (`family.rs:698`, 8), ported at 64.

### Call 32. D5: the estimate sheet, the recap, materials and suppliers (2026-11-11)

Searched first, on this machine: the earlier estimating program in this repository
(`packages/engine/src/model.ts`, `line.ts`, `recap.ts`) has this ground settled, with the
reasons written beside the code. What is taken from it, by name, and what is not:

* **In-house labor is a line whose order unit is hours.** Its production rate is the
  order factor (`SF/hr`: so many to the man-hour, or `hr/SF`), and its unit cost is
  dollars an hour. The earlier program does the same (`line.ts`: a production rate "IS
  the order-step conversion: the order unit is HOURS"). No new arithmetic: a quantity,
  an order factor, a unit cost. Crew-day rates (forty squares a day with a crew of six)
  are in the earlier program and not here: they need a crew size and the hours in a
  day, and are in PROPOSALS.md.
* **A line has one class**, and the recap has one row for each: Material, Labor,
  Fabrication, Supervision, Subcontract, General conditions. The earlier program's
  classes are Material, Labor, Sub, Equipment, Other, Supervision, with fabrication kept
  apart as a "basis"; the brief names the six used here. Equipment and Other are not
  built (nobody asked). A class is typed on the line and checked against what the line
  is: a Labor, Fabrication or Supervision line is in hours, and no other line is. A
  subcontract line is a quantity and a price and is refused a production rate.
* **The three columns** are Material; Labor (labor, fabrication, supervision); General
  (subcontract and general conditions). The recap keeps all six classes apart. Which
  columns the owner's own sheet has is not known (QUESTIONS.md Q14).
* **"Labor toggles per line"** is read as: a Labor line can be switched off. It stays on
  the sheet with what it would come to and is out of the element's detail, the columns
  and the recap. The other reading (one row with a material amount and a labor amount,
  and a switch on the labor) is a different sheet; Q14 asks.
* **Money is added the way D4 adds it:** each amount to the cent, a total the sum of the
  amounts as shown (the earlier program's rule, `recap.ts`), a line with no figure named
  and left out, never $0.00. Hours are shown to four places and added as shown.
* **Overhead is a percentage of the job cost; profit is a percentage of the job cost
  plus overhead** (`recap.ts`). Both are entered on the job. Neither has a default: with
  one not entered there is no price. The earlier program's class adders (sales tax,
  escalation, burden, general liability) and bond are not built: the brief asks for O&P.
  They are in PROPOSALS.md, because a real price needs them.
* **A material is an element** (`Kind::Material`) carrying how it is bought (an order
  unit and factor: pounds at so many to the SF) and what it costs (a generic price and
  a price by supplier). An element says what it is made of by the material's name, the
  way BIMCraft's layers and run types name a material of the project's library
  (`model.rs:430`, `2473`; its material is colour and pattern, and is not ported). A
  cost line marked as made of the element's material takes its order factor and its
  price from that material, and carries neither itself. Changing the element's material
  reprices those lines and no others. This replaces the note in `building/model.rs`
  that "a material here is a condition".
* **Suppliers are names on a material's prices.** A line made of a material is extended
  at the generic price and at each supplier's. A supplier's total is over the lines it
  quoted, and names the ones it did not. The recap is at the generic price; choosing a
  supplier to price the job by is not asked for and is in PROPOSALS.md.
* **General conditions are cost lines on the job element itself**, typed by hand: a
  description, a number, a price.

### Call 33. The tenth judgment: what was decided in fixing it (2026-11-11)

* **What an element's lines come to rests on its condition.** The lines are what the
  condition is made of. An Assumed condition makes assumed money; one that is not known
  leaves no figure. The quantity of a line (an area, a length) does not rest on it, and
  neither does an allowance.
* **A quantity of 0 is $0.00 with a note, not a refusal.** The judge offered either. A
  line priced on a figure that is truly nothing (the opening area of a wall with no
  openings) is right at $0.00, and refusing it would leave the element with no detail.
  It is never silent: the line, the element's detail and both printouts say so. A price
  of $0 that somebody entered is an answer and carries no note (call 30).
* **Money is rounded half up, as on paper.** A product such as 1 x 1.015 is held a hair
  under the half cent; it is pushed by a few units in its own last place, never by as
  much as a thousandth of a cent, before it is rounded. Above $2.8 billion the push is
  capped and a true half cent may round either way; above $10 trillion an amount is
  refused.
* **A comma in a price is a thousands mark or the price is refused.** `1,25` is not read
  as 125, and is not read as 1.25 either: which was meant is not guessed.
* **A room rests on the walls its walls are cut to.** This is wider than the judge's
  case: any wall met at an end of a wall the room is measured to counts, bounding or
  not, because the end is cut to it.
* **A wall with unsure heights is tried as tall as it could be, from where it is known
  to start.** Its base is moved only if its base is unsure.
* **A room point exactly on the corner of two faces is left reading as inside a wall.**
  The ported region test decides it. Said in STATUS.md.
* **Five `check_*` functions are ported, not ours**: they carry BIMCraft's rule for the
  same thing, and PORTMAP.md now cites the line.

### Call 34. Drawing and editing: the undo history (2026-11-11)

First piece of the stretch in `HANDOFF_DRAW.md`. Read first: BIMCraft
`crates/core/src/engine.rs` lines 100 to 200, 362 to 400, 494 to 506 and 722 to 749.

* **The gate was widened before any of it was written.** `tools/portmap.sh` now covers
  `crates/world/src/editing*.rs` and `editing/` as it covers the building model. It was
  run red on the new file (seven unlisted items) before the map had its rows.
* **Undo is BIMCraft's: a stack of steps, each with the whole model as it was before
  the step.** The last 200 are kept; a step that changed nothing is not one; a new step
  empties the redo stack; stepping back swaps the model with the one kept.
* **Changed in the port: the model kept is the saved job's text.** BIMCraft keeps a
  shared copy of its model in memory. A job here is an entity-component world, which is
  not copied, and its saved text loads back to the same bytes; so the text is kept, and
  a step undone is the file as it was. The cost is one save of the job for each step.
  Listed in the file's header.

### Call 35. Drawing and editing: the verbs (2026-11-11)

Read first: BIMCraft `crates/core/src/commands.rs`, `wall` (7341 to 7400), `opening_of`
and `build_opening` (7473 to 7520), `room` (7909 to 7923), `stood_of` (8662 to 8676),
`runs_of` (8776 to 8800), `move_elements` (12419 to 12435), `delete` (13011 to 13040),
`level` (6422 to 6436) and the type command (11186 to 11245); `engine.rs` `transaction`
(644 to 665).

* **Each verb follows BIMCraft's rule for the same thing** (a wall for each pair of
  points, an opening taken along its wall, a run for each pair of points, a thing stood
  at a point, a wall's openings moved and removed with it), and `PORTMAP.md` cites the
  line.
* **Nothing is given a value nobody entered.** BIMCraft gives a wall a default height or
  the level above, an opening the middle of its wall, a window type a 900 mm sill, a run
  the floor's height. Here a verb with one of those unsaid is refused and says what to
  say. That is rule 4 of the brief, and it is listed as a change in the file's header.
* **A point for an opening that falls past its wall's end is refused**, where BIMCraft
  takes it to the end: an opening centred on a wall's end does not fit, and would be
  written and then flagged.
* **Every verb is all of its change or none.** If any part fails, or leaves a reference
  that does not hold, the job is put back from its text as it was. Three walls of which
  the second has no length leave no wall.
* **A new element is named for its kind and its number** ("wall 31"). BIMCraft's
  elements have ids and no names; here a name is what the sheet and the notes say.
* **A move takes the worse of the thing's status and the mover's**, and adds the move to
  the source of a value that keeps its status.
* **`plumbline new`** stamps the job with the machine's date. Nothing else reads a clock.

### Call 36. Drawing and editing: snapping (2026-11-11)

Read first: BIMCraft `crates/core/src/snap.rs` lines 1 to 330 (the kinds, the switches,
the scene and how walls and components enter it), 586 to 662 (crossings, ray_hit, rays,
corner, rank) and 736 to 916 (the solver), and `geom.rs` 330 to 340.

* **The solver is BIMCraft's, in its order:** a point of the plan within reach (an end
  before a crossing before a middle where two lie as near); the foot of the
  perpendicular from the last point; a guide through the last point every 15 degrees
  from east and from the line before, within 4 degrees, ending where it crosses a wall;
  the nearest point of a wall's line; else the round step. A square step is pinned to
  east or north of the last point first.
* **The round step is an inch.** BIMCraft's is 10 mm. Neither is the other converted.
* **Runs are snapped to.** BIMCraft's plan scene has no runs in it (its lines are walls,
  grids and detail lines). A duct here has to start where another ends, so a run gives
  its ends, its middle and its line. Listed as added in the file's header.
* **The scene is a value of its own**, made from the job by `scene`, so the solver is
  tested on plans written out by hand.
* **The walls snapped to are those entered on the storey being drawn on.** BIMCraft takes
  the walls its plan cuts.

### Call 37. The eleventh judgment: what was decided in fixing it (2026-11-11)

* **A total of money that leaves something out is flagged for that alone.** The owner's
  rule is that such a total says what it left out, and it did, a few lines further down.
  A price reads on its own line, so it carries the flag and says there how many lines it
  leaves out. It is not refused: an estimate in progress still has a running price, and
  a refused one would hide it. This applies to every total: job cost, class, column,
  the D4 allowance and detail totals, and each supplier's.
* **A line with no class stands in no column.** It showed in the General column while
  that column's total left it out. It now says under itself what it comes to and that it
  is in no column and no row.
* **A number typed as a whole quantity is not a count to be rounded.** 2.5 is 2.5. A
  quantity that is a formula coming out as a count is still rounded up (26.67 fasteners
  is 27) and now says what it was rounded from.
* **Hours are shown to four places.** Call 32 said they were; the printouts showed two.
* **The half-cent push is three units in the last place** (correcting call 33, which
  described a larger one and a wrong bound): a quantity, a price, their product and the
  hundred are each good to half a unit, so under three in all. It is capped at a
  thousandth of a cent, which it reaches at about $15 billion; above $10 trillion an
  amount or a total is refused.
* **A rate is written as it was entered**, by the shortest decimal that is the number,
  and never to fewer than two places. One function writes it everywhere.
* **Cost lines go on what is built and on the job.** Not on a condition (its lines are
  its recipe), a material (its cost is its price) or a type.
* **A name is one name.** Suppliers and materials are matched ignoring capitals and
  spacing when one is entered, as cost lines on an element already are.
* **With a condition that is not known a line is still measured and ordered**; only what
  it comes to has no figure. With an Assumed one, the dollars are Assumed and the
  quantities are not.
* **A room and the walls its walls are cut to (narrowing call 33):** only where the cut
  end is at the room.

### Call 38. Drawing and editing: the tools in the window (2026-11-11)

Ours to make; recorded so they can be overruled. The design is `HANDOFF_DRAW.md`
section 10. What was decided in building it:

* **Drawing is done on the plan, with one storey chosen.** In the 3D view a tool is
  refused and says to go to the plan. With no storey in the job the panel offers a new
  one and nothing else.
* **A tool draws with what was typed for it, and with nothing until something is.** A
  wall's thickness and height, an opening's width, height and sill, a run's size, height
  above the floor and system, a thing's name, size and height: one line, typed once per
  tool, shown under the tool from then on. The first click on a tool that has none opens
  the field instead of drawing. There is no default wall.
* **What is drawn in the window is Assumed, and says it was drawn there.** A pointer on a
  screen is not a measurement. A typed length is no better than the corner it starts
  from, so it is Assumed too. The window has no way to call a drawn value Known; a
  Known wall is entered at the command line, where every verb takes a status and a
  source.
* **Types and systems are found by what was typed or made:** a wall type of that
  thickness, an opening type of that width and height, a system of that classification.
  A second wall type of the same thickness is never made by the tool.
* **A typed length goes from the last point toward where the pointer's snapped place
  is**, for walls, runs and a move (BIMCraft `draw.rs`, the `toward` rule). A digit
  typed with one point picked opens the field.
* **On plan, the wall over and under an opening is the opening.** A plan draws a door or
  a window as its span in the wall, and looking straight down the wall above it is all
  that can be clicked. So on plan a click there selects the opening; in the 3D view the
  same piece is still the wall.
* **Undo covers every edit the window makes**, the ones it already made (a value typed,
  a condition set) as well as the drawing ones: Z and Y, and rows in the panel.
* **Rooms are not drawn in the window.** The handoff's stretch is walls, openings, runs
  and fixtures. A room is placed at the command line (`plumbline room --at`), from the
  walls the window drew, and the demonstration does that.
* **The left panel scrolls and its rows keep their height.** When the Systems section
  grew, the rows above it were squeezed and a click meant for one tool landed on another.

A defect that is not explained and is said here rather than hidden: on the software
renderer the demonstrations run on, keys typed in about the second after a field opens
are lost. The frame that first draws the field is slow there; whether a real graphics
card shows it is not known, because none was available. The demonstration types the
first character, waits, then types the rest.

### Call 39. Fittings, connections and a run cut in two (2026-11-11)

Ours to make; recorded so they can be overruled. The task is `HANDOFF_CLASH.md`.
What was checked first: BIMCraft places a pipe fitting by hand, as a component of a
family with its connectors drawn in the family (`fixtures.rs:203`, `family.rs:132`). It
does not work out which fitting a joint takes and has no verb that joins a run to a
thing. Its `split` (`commands.rs:12534`) is of a wall. So the rule for cutting is
ported and the rest is ours (`crates/world/src/editing/join.rs`; PORTMAP.md).

* **A fitting's kind is worked out from what meets, not typed.** Two ends at an angle
  are a bend; two in line (within a degree) of one size a coupling, of two sizes a
  transition; three a tee; four a cross. A kind that is given is checked against that
  and refused if it disagrees. One end, five ends, a fixture's connector at the joint,
  runs of two systems, or cable: refused, each with its reason. An estimator counts
  fittings from the runs and has no family library to pick a part from.
* **Two runs in line with a size not known cannot be told apart as a coupling or a
  transition**, so the verb refuses unless it is told which.
* **A fitting is as sure as the runs it joins, never surer**, whatever status it was
  asked for with; its source says which runs it joins and that which part is bought is
  not selected. It is drawn at its largest run's size, and that size is Assumed.
* **A branch on a run's length becomes a tee by cutting the run there.** The network
  calls a run met along its length a place with no fitting whether or not something
  stands there (it is ported that way), so a tee has to be three ends. The first piece
  keeps the run's path, the second is new, and what the run carries (its system, its
  condition, its allowance, its material, its cost lines) is on both.
* **A run with a cost line whose quantity is a typed number is not cut.** Four hangers
  typed on a run would be four on each piece, and which piece they belong to cannot be
  told. A quantity that is a formula is worked again for each piece. The refusal names
  the line.
* **Fittings are not placed by themselves as runs are drawn.** `fitting` places one
  where it is asked; `fitting --all` places one at every joint that has none and lists
  what it passed over and why.
* **`--all` does not put a coupling between two runs of one size in line.** That is most
  often one run drawn in two picks, and a coupling at every pick would be a count nobody
  entered. A coupling goes in where it is asked for by name.
* **Joints are looked for a storey at a time.** A riser that meets a run of the storey
  above is not joined to it by these verbs. Said, not hidden.
* **A run is joined to a fixture or to equipment by giving the thing a connector where
  the run's end is**: of the run's kind and size, carrying what the run's system carries
  where a connector can say that (cold, hot, waste, vent). The end has to be at the
  thing: inside its box or within the fit tolerance of it. A thing whose size is not
  known cannot be joined to: whether the run reaches it cannot be told.
* **What a pipe connector carries and what a system is classed as are matched in one
  place** (`Domain::system_classification`), where the takeoff and `connect` both read it.

### Call 40. Clash detection (2026-11-11)

Ours to make. What was checked first: BIMCraft has none (`grep -rni clash
crates/core/src`: two comments, and a variable in `import.rs` for parameter names that
collide). The trade's words, from a web search on the day: a hard clash is two things in
the same space; a soft or clearance clash is two things closer than a distance asked
for. The solid geometry is `parry3d-f64` 0.31.1 (Apache-2.0; its twelve crates are in
LICENSES.md), read from its source before a line of it was called.

* **What is tested is what `building::placements` gives a shape, and nothing else.** The
  window draws from the same list, so what clashes is what is seen.
* **A clash is two solids in each other by more than the fit tolerance**, a hundredth of
  a foot: each is drawn in by half of that and what is left is tested for overlap.
  Touching is not a clash.
* **The yes or no and the how deep are asked separately, because the library's measure
  of depth fails where it matters most.** Asked how deep two solids are in each other,
  it answers zero when one is dead centre in the other: a pipe and a duct drawn at one
  height, crossing. Found by the first test written (0 clashes where one was worked by
  hand at 3 3/8 in). Its overlap test does not fail, and that is what decides. The depth
  is then asked three times, a billionth of a foot off centre, and a clash whose depth
  still cannot be had is reported with no depth. It is not dropped.
* **A door or a window is a hole.** A pipe through a window is through nothing.
* **Walls and floors are not tested against each other**; they are built into one
  another. Nor is what is joined: two runs at a joint, a run and its fitting, a run and
  the thing it is connected to. Two runs of one system that cross without being joined
  are tested.
* **A run or a fitting that meets a wall or a floor is listed apart, as through it**, with
  no depth: it is a hole to cut and sleeve, which is a count, not two trades in one
  place. This model's walls are solid, so a pipe drawn inside a wall is through it.
* **A clearance is tested only when one is asked for.** There is no default distance.
  More than ten feet is refused.
* **A clash is as sure as the two shapes**: Assumed when either is.
* **What has no shape is listed as not tested, with why**: a thing with no size, a run
  with no size (a line, not a solid). The roof's planes are not in `placements` and are
  not tested; every report says so.
* **Nothing is stored.** Clashes are worked out from the job each time, as the network
  is, so they cannot go stale.

### Call 41. The twelfth judgment: what was decided in fixing it (2026-11-11)

The findings are in STATUS.md, "Twelfth judgment". What had to be decided:

* **One wall drawn over another on a storey is refused**, by the verbs and on load:
  two walls that share a line for more than the fit tolerance are the same wall counted
  twice. Two that stand one over the other, each with its own height entered, are two
  walls. (J12-1)
* **What a roof's footprint solves is not removed**; it would come back the next time
  the roof is solved. The footprint is changed, or the roof removed. **A plane that
  lines lie on is not removed** while they name it. **Nor a material something is made
  of, nor a condition something has.** (J12-2, 3, 14)
* **A run goes on a system that carries what it carries**: a duct on Supply, Return or
  Exhaust Air; a pipe on Refrigerant, Domestic Cold or Hot Water, Sanitary or Vent; a
  conduit or a cable on Power or Lighting. Checked by the verbs and on load. A fixture,
  a piece of equipment and a fitting can be on any system. (J12-4)
* **An opening placed by a point is on the wall**: within half the wall's thickness of
  its line. (J12-5)
* **A wall's opening area is not given when an opening is known not to fit** (past an
  end, over the top, over another), and every opening that does not fit is named. An
  opening whose place is not known still has its size, so there the opening area stands
  and only the net is refused: that part of the earlier rule is kept, and its test with
  it. The counts of doors and windows are counts of what is entered and are left. (J12-6)
* **Shift holds the next point square to the last.** The square guide was in the
  handoff's fence and in the README and was never switched on. (J12-7)
* **A source keeps where the value first came from and what was done to it last**:
  "plan A; moved (by eye)". It does not grow with each move, and a move that makes the
  value less sure no longer wipes where it came from. The same for a cut. This changes
  two test expectations, which said the worse source replaced the first. (J12-10)
* **A move by nothing is refused.** (J12-11)
* **A wall's top cannot be held to its own storey or to one below**, when both floors'
  heights are known; the verb refuses it. A file that has one still loads, flagged,
  because a storey's height can be corrected afterwards. (J12-12)
* **A name that is looked up is one name whatever its capitals, its spacing or a full
  stop at its end**, and `set material` finds a material the way `material` refuses a
  second one. Storeys and types are found by path, not by name, and two whose names
  differ in capitals are left as two. (J12-13)
* **Cost lines are not accepted on a system or a storey** (they go on what is built and
  on the job, as call 37 said). **A material is not ordered by the hour.** (J12-17, 18)
* **`system` and `remove` refuse `--status` and `--source`** rather than ignore them. (J12-20)
* **The window checks a tool's settings when they are typed**: a size of nothing or less,
  and a system a run of that kind does not go on, are refused there and not at the
  second pick. (J12-21)
* **Correcting call 37 on the half-cent push (J12-19).** It is three times the spacing
  of numbers at the size of the figure, which is between three and six units in its last
  place, not three. And above about a trillion dollars a figure cannot hold half a cent
  at all: 1e12 + 0.004999 is stored as 1e12 + 0.005. Amounts are refused above ten
  trillion; between one and ten trillion a cent can be off. No code changed.
* **Left, and said:** four `*_for` rows in PORTMAP are now marked ported from BIMCraft's
  `type_for_size` (J12-15). The changes to the window (J12-8, 9, 16) are made from the
  judge's reading of the code and one picture each, and have no test that drives the
  window; they are covered only by `demos/draww.sh` still passing.

### Call 42. Patrick's rulings of 2026-11-11, as he gave them, and how they were folded in

He was shown a review of the web seat's brief and an account of what was built, and
ruled. "These override anything older that conflicts. Fold them into CLAUDE.md, one
place per rule, and delete what's stale." His words:

1. **Mission: the whole house, every trade.** Call 26 stands. Trace the drawings we're
   given, build the 3D house from the tracing, and show the clashes the drawings hid. A
   veteran with simple field tools then confirms or clears each one.
2. **The loop.** Traced values start as Assumed. A clash between Assumed elements is a
   field question; a clash between Known elements is a finding. Produce a
   verify-in-field (VIF) list: each question, where it is, and the dollars riding on it,
   sorted by dollars. A field measurement turns Assumed into Known.
3. **Imports matter as much as exports.** People must be able to bring in what they
   already have: PDF plans, both vector and scanned; PNG and JPEG plans, and aerial
   images; CSV and XLSX: supplier price lists, prior estimates, and assembly libraries;
   IFC models from other trades and designers; the program's own job file. The job file
   and CSV round-trip without loss. For other formats, any loss is reported, never silent.
4. **Speak construction.** Every word on screen is a builder's or estimator's word.
   Write units the way trades write them: SF, LF, EA, SQ, LS, LB, sheets, rolls, boxes,
   M. No software term appears anywhere a user sees.
5. **Teach by doing.** A new user completes a first takeoff from a PDF without a manual.
   Every number can show what it is made of and where it came from. Depth is on demand:
   macro to micro means zooming into the same job, never changing screens. How you teach
   is your design.
6. **Order of work.** The order is mine; how you do each step is yours. Finish fittings
   and clashes, and have them judged. PDF underlay: calibrate from one dimension, trace,
   and snap to vector lines where they exist. The remaining imports, plus a language
   pass over every screen. Field entry: any field that takes a length accepts a laser in
   keyboard mode; a Leica DISTO types the reading at the cursor, with units; marking a
   value Known works in the window; Bosch GLM Bluetooth goes to PROPOSALS.md. Floors,
   rooms and roofs drawn in the window. The VIF list and the teach-by-doing first job.
   The visual standard and a visual judge. Then IFC export, xlsx export and publishing.
7. **Look: the instrument.** Draw edges by line type, keep surfaces matte, keep layouts
   dense, and make the plan and the sheet read like drawings. 3D stays shaded, with
   studio light and a neutral ground. No sky, sun or grass. How you meet this standard
   is yours.
8. **Yes: put this at the top of CLAUDE.md:** "This codebase is yours. You own it, you
   will maintain it, and its design is your call. Product scope and the fixed rules
   belong to Patrick." Replace the inherited "no feature nobody asked for" line with "New
   features go to PROPOSALS.md."
9. **Repository: stay on the branch.** Code in the cloud is fine; local-only covers job
   data and costs when people use the program.
10. **Verified:** the model requires 30-day data retention, per its maker's What's new
    page. Sanitize anything placed in refs/.
11. **Run autonomously.** Search the web whenever it helps. These need no approval from
    me: anything in the repository or the cloud workspace; installing packages and
    toolchains there; pushing to the branch; starting judges, on Opus or Sonnet only. Ask
    me before rewriting branch history or touching any other branch. Log each call in
    DECISIONS.md. Questions go to QUESTIONS.md with your working assumption, and you
    keep building. Real job, client or supplier data never enters the repository.
12. **The deliverable: a release candidate.** Thirteen items, each with evidence, then
    tag the release candidate on the branch, stop and ask for the production ruling. The
    list is in CLAUDE.md, "The deliverable", in his words, and STATUS.md tracks each.

How they were folded in, which is mine to decide:

* **CLAUDE.md is rewritten around plumbline.** It was the earlier program's file. The
  rules that were that program's alone (its webview's missing dialogs, its probe
  harness, its screen vocabulary, its run commands) are kept in one short section at the
  end, because that program is still in the tree. Everything that was a history of how a
  rule was broken is cut to the practice it taught.
* **What the rulings replaced, and so was deleted:** "no feature nobody asked for"
  (ruling 8); "never force-push" and "write only inside this repository" (ruling 11
  says what needs asking); "no network call in the product" as a rule about the code
  (ruling 9: it is job data and costs that stay local; the program still sends none of
  either, and the egress gate still runs); the earlier program's word list as "the only
  vocabulary" (ruling 4).
* **The brief's D0 to D7 are no longer the list of what is owed.** The thirteen items of
  ruling 12 are. STATUS.md keeps the D rows as the record of what was built and judged
  under them, and gains a table for the thirteen.
* **The handoffs** (`HANDOFF_REBASE.md`, `HANDOFF_DRAW.md`, `HANDOFF_CLASH.md`) each
  carry rules that CLAUDE.md now holds. They are the record of three tasks, two of them
  finished. A line at the head of each says the rules live in CLAUDE.md and that file
  governs; their task sections stand as written.
* **The lit scene goes.** Calls 25 and 26 put a sky, a sun, shadows and grass in the 3D
  view on a reading of "game-grade" that ruling 7 corrects. It is step 7 of the order of
  work, so it is not changed today; every picture taken before then shows the old look.
* **Questions this closes:** Q2 (stay on the branch), Q7 (IfcOpenShell: installing a
  validation tool in the workspace needs no approval, and item 8 asks for it), Q11 (the
  whole house is the mission; the earlier program is not built on), Q13 (the order).
* **Questions it opens** are Q15 to Q18 in QUESTIONS.md, each with the assumption in use.

### Call 43. Plans under the tracing: what is built on, and the plan set (2026-11-11)

Step 2 of Patrick's order: a PDF under the tracing, calibrated from one dimension,
traced over, snapping to its own lines where it has them.

**Searched first.** On the machine: BIMCraft's `pdf.rs` writes PDFs of its sheets and
reads none (`grep -rn "underlay\|calibrat" crates/core/src`: nothing but a curve
digitised for a storm table). The earlier programs on this machine traced over plans in
a browser, which is no help to a Rust window. On the registry, 2026-11-11:

| Candidate | Licence | Verdict |
|---|---|---|
| `hayro` 0.9.0, with `hayro-interpret` and `hayro-syntax` | Apache-2.0 OR MIT | **Build on.** Pure Rust, no unsafe, draws a page to pixels on the processor, and hands each path it draws to a device of ours, which is how a vector plan's own lines are read out. Read in its source before use: `render`, `interpret_page`, the `Device` trait. It says of itself that it is not yet complete (no encrypted files, no blending groups). |
| `pdfium-render` 0.9.5 | MIT OR Apache-2.0 over Google's PDFium | Not used: it needs PDFium's compiled library fetched or shipped beside the program. |
| MuPDF, Poppler | AGPL, GPL | Reference only. `pdftoppm` was used once here to look at the plan set before anything was built, as a person would use a viewer. |

**The plan set (answers Q15).** The U.S. National Agricultural Library holds the
Cooperative Farm Building Plan Exchange's rural house plans, and says of each: "authored
by federal employees as part of their official duties and is therefore
non-copyrightable and/or published by the federal government and now in the public
domain". Plan No. 7213, Two-Bedroom Farmhouse (USDA, 1976), two sheets: a floor plan
with dimensions, a foundation plan, an electrical plan, door and window schedules, four
elevations, a wall section and a roof at 3 1/2 in 12. It is a scan, 23 MB. It is not
committed: `tools/fetch-plans.sh` fetches it from the library and checks its SHA-256,
into `plans/`, which git ignores. A vector plan is still needed to show snapping to a
PDF's own lines, and no public vector set with a certain licence was found in two
searches, so a small made-up plan is written as a vector PDF by `tools/made-up-plan.py`
and committed, marked as made up.

**What was decided.**

* **A plan under the tracing is a Drawing: an element under the storey it is a plan of.**
  It holds where its file is (beside the job, by a relative path), which page, the
  file's length and a checksum so a changed file is noticed, and its scale. The picture
  is not kept in the job: the job stays readable text, and a client's plans do not ride
  along inside it.
* **The scale is one dimension: two points on the page and the distance between them.**
  That is how a takeoff is calibrated everywhere. Until a drawing has one, nothing can be
  traced over it: a tracing with no scale is a number nobody entered.
* **What is traced is Assumed**, and its source names the drawing it was traced from
  (Patrick's ruling 2). The distance typed for the scale is a dimension read off the
  drawing and is Assumed too.
* **A new crate, `plumbline-plans`, reads plan files.** The world crate still reads no
  disk and draws nothing: it holds the Drawing and the arithmetic of its scale. The
  plans crate turns a page into a picture and, for a vector PDF, the straight lines on
  it. The command line and the window both use it.
* **Loss is reported** (ruling 3). Reading a page says what it could not give: curves
  (snapped to at their ends only), words (drawn, not read: a dimension's figure is typed
  by the person), pictures inside a vector page, and a scan (a picture: nothing on it to
  snap to).
* **A page is drawn at no more than 4,096 dots on its long side.** A 36 in sheet comes
  out at about 113 dots to the inch, which reads. Sharper tiles on zooming in are not
  built.

### Call 44. The thirteenth judgment: what was decided in fixing it (2026-11-11)

The findings are in STATUS.md, "Thirteenth judgment". Each fix that changed what the
program accepts, refuses or reports is a decision, and is here before the test that
holds it was written.

* **Being joined excuses the joint, not the two things (J13-1).** Before, any two
  elements that shared a joint were never tested against each other, so a run laid over
  another, folded back along it, or carried through the tank it was joined to was
  missed. Now each run is left out for a little way either side of each joint and what
  is left of the two is tested like anything else. The little way: between two runs,
  three times the half-width of the larger (which clears every joint down to a 45
  degree wye); into a fixture, equipment or a fitting, as deep as the joint is in it
  (to its nearest face) plus the run's half-width. A run joined to the middle of a tank
  and leaving by the nearest face is clear; one that goes on through is not. A stub
  shorter than the little way is not seen, and the file's head says so. No clearance is
  asked between things that are joined. Two fixtures or fittings joined to each other
  with no run are still not tested.
* **The sample house gains three clashes by this**, each checked by hand: the hot branch
  to lavatory 1 drawn a foot back up inside the hot main (1/4 + 3/8 = 5/8 in), and two
  pipes whose centre lines lie in the water heater's face, each in it by its own radius
  (1/2 in and 3/8 in). 18 becomes 21. The house is drawn, not engineered; they stay.
* **Two ends leaving a joint the same way make no fitting (J13-1)**, for three and four
  ends as for two. They are one over the other, which is a clash.
* **A wall's solid pieces are its face less its holes, whatever the holes do (J13-2).**
  `geom::face_less_holes` cuts every hole out of every strip. The clash test and the
  window draw from the same pieces.
* **A shape is no surer than its storey's floor height; a wall's also no surer than its
  base offset (J13-3).** `placements` now carries both into every shape's status, and a
  fitting's and a connector's status carries the floor height's. A job drawn in the
  window, where the storey's height is typed and Assumed, therefore has no findings,
  only questions for the field, until that height is made Known. That is ruling 2.
* **A wall with an opening that could not be cut is Assumed at best (J13-4).** It is
  drawn solid where the opening would be, so a clash with it there is a question.
* **Against a wall or a floor, depth is the least way out that clears all of it
  (J13-5).** A wall or a floor is several solids; each one's own way out counts only if
  the thing, moved that far, is clear of every piece; straight out through the face (a
  wall) or the top or bottom (a floor) always counts.
* **Two round runs are measured by plain geometry (J13-17)** where it is exact: side by
  side, one in the other, or crossing well inside both lengths. Sum of the radii less
  the distance between centre lines. The library's figure was half a percent short.
* **Where a clash is (J13-16):** the middle of the two boxes' overlap if that place is
  inside both things; otherwise a place the measure of depth found in both.
* **A fitting is turned to lie along the first run it joins (J13-6).** Its box is still
  the run's size and still not the part that is bought.
* **A quantity that names no measure is a typed number (J13-7)**, however written:
  `2 * 2`, `-(-4)`. A run carrying one is not cut.
* **One wall over another is judged by their bodies (J13-8).** Each wall is its centre
  line (from its drawn line and its `location`) and its thickness. Two bodies overlap
  where the centre lines are nearer than half of both thicknesses. Walls running the
  same way (within about three degrees) may not overlap at all; walls at an angle may be
  in each other for up to twice both thicknesses, which is a corner or a crossing. Walls
  back to back on a shared face are two walls. Where each stands is taken from its base
  and its top however the top is said, so a wall on a knee wall is two walls. Walls
  meeting at less than about ten degrees are refused by this; none has been drawn.
* **An opening's point is measured from the wall's middle (J13-9)**, not from the line
  it was drawn on.
* **The hour is the hour however it is spelled (J13-10):** hr, hrs, h, hour, hours, in
  any capitals.
* **What makes two names one name is narrower (J13-11).** Capitals, runs of spaces,
  spaces beside a comma, and a full stop that is not a decimal point. A space between
  two figures is kept: "1 1/2 in" is not "11/2 in". This replaces call 41's "all
  spacing". Removing a condition matches its name the same way a material's is matched.
* **Only a note this program wrote is replaced in a source (J13-12):** the last
  "; moved (...)" or "; cut (...)", and only at the end. The same words inside what a
  person wrote stay.
* **A run is not joined to a thing on a system it cannot go on (J13-13).**
* **The second piece of a cut run keeps the run's name (J13-18)** when somebody named
  it: "<name>, cut at <number>".
* **Not done, and why.** J13-14 (a pipe's and a duct's ends at one place: which fitting)
  and J13-15 (a tee on an upright run; joints `fitting --all` passes over without
  listing) need a way to say which run or a cut by height. They are next in the fixes,
  after these are judged.

### Call 45. Tables brought in: what is built on, and what each table is (2026-11-11)

Step 3 of Patrick's order: the remaining imports. Ruling 3 names CSV and XLSX price
lists, prior estimates and assembly libraries, IFC models, and aerial images; the job
file and CSV go out and come back without loss; for anything else, loss is reported.

**Searched first.** On the machine: BIMCraft writes one sheet of XLSX by hand
(`xlsx.rs`, 250 lines, "Nothing is read back"), reads IFC2x3 and IFC4 (`import.rs`,
4,895 lines, over its own STEP reader, `step.rs`), and writes IFC4 (`ifc.rs`). It has no
CSV and reads no spreadsheet. The earlier program at the repository root reads price
lists in its own JSON only. On the registry, 2026-11-11:

| Candidate | Licence | Verdict |
|---|---|---|
| `csv` 1.4.0 | MIT OR Unlicense | **Build on** for CSV in and out: RFC 4180's quoting is a solved thing. |
| `calamine` 0.38.0 | MIT | **Build on** for reading XLSX: cells, formulas (to count what was lost), merged cells, sheet names. Pure Rust. |
| `rust_xlsxwriter` 0.102.0 | MIT OR Apache-2.0 | For order step 8 (xlsx out with live formulas). Not linked yet. |
| BIMCraft `import.rs`, `step.rs` | MIT | **Port** for IFC in, item by item in PORTMAP.md, when that part is built. |

**What was decided.**

* **A new crate, `plumbline-sheets`, turns a file into rows of text** and lists what was
  in the file and is not in the rows. The world crate still reads no disk: its `tables`
  says what the rows mean. The command line and the window both use the two.
* **Three tables, each written and read by this program:** a price list (a material,
  what it is bought by, a price, whose price), an assembly library (conditions and the
  lines each puts on an element), and an estimate's lines. A table this program wrote
  comes back as the same values, each with how sure it is and where it came from: those
  are columns.
* **Somebody else's table is read by its headings.** A heading is matched to what it
  means from a short list a trade would use (Item or Material or Description; UOM or
  Unit; Price or Unit price or Cost). A column that is not used is named. A row that is
  not used is listed with why. A file with no column for something the table needs is
  refused whole.
* **A price brought in is as sure as whoever brought it says** (`--status`,
  `--source`); a table this program wrote carries its own.
* **A row for a material the job does not have is not added unasked.** It is listed.
  `--add` adds them.
* **A prior estimate is brought in to be set against the job, not added to it.** Its
  lines would otherwise be counted in the job's price. It is kept in the job as what it
  is, and `compare` sets it against the job's own lines, line by line.
* **A workbook's first sheet is read unless one is named.** The others are named as not
  read. A formula is read as what it last worked out to, and how many there were is
  said.

### Call 46. The fourteenth judgment: what was decided in fixing it (2026-11-11)

The findings are in STATUS.md, "Fourteenth judgment". Each fix that changed what the
program accepts, refuses or reports is a decision, and is here before the test that
holds it was written or changed.

* **What is committed has to build (J14-1).** The ignore line for fetched plan sets was
  `plans/`, which also caught `crates/plans` and `demos/plans`. It is `/plans/` now.
  `tools/committed.sh` lays the last commit out as git holds it and builds and tests it;
  it is run before a push. It fails on `939e6aa`, `719ff91` and `f449a57`.
* **A scale set again moves nothing that was traced, and says what it left (J14-2).** A
  length typed off a dimension is right whatever the sheet's scale was; a point picked
  on the sheet is not; the two cannot be told apart afterwards, so moving what was
  traced would spoil the first to mend the second. What was traced at the earlier scale
  is listed when the scale is set, kept on the drawing (`earlier`), listed by `drawings`
  from then on, and said in the window in the tone of a refusal.
* **A drawing remembers what it was lined up by (J14-3)**: the place on the page and the
  place on the plan. A scale set again puts the corner where that place stays put.
* **A scale's two points are on the page and at least one unit of it apart (J14-14).**
* **A picture is shown at no more than 4,096 dots on its long side, as a PDF page is
  (J14-4).** Its size in the job is still its own dots, so a scale picked on it is right.
  That it is shown smaller than it is, is said.
* **Walls over walls (J14-5, J14-6; this replaces part of call 44).** Walls at an angle
  to each other (more than about three degrees) are never one over another, however
  sharp the angle: they are in each other only where they meet. Walls running the same
  way may not overlap at all. A wall whose thickness is not known counts as three
  inches thick for this test. Walls on different storeys are compared too, by where each
  stands above the datum; where that is not known for either, walls of one storey are
  taken to stand together and walls of two storeys apart.
* **How much of a joint is excused depends on the angle (J14-7; this replaces part of
  call 44).** Between two runs: the sum of their half-widths over the sine of the angle
  between them, each way along each run; for anything under fifteen degrees, what
  fifteen degrees gives. A 30 degree lateral is a joint. Two runs along one another
  for more than about four times their combined half-widths are a clash.
* **What a PDF page could not give (J14-8).** The edges of filled shapes are read as
  lines (walls are often drawn as solid bars). Lines under a point long are counted and
  said. A line is cut to what a square clip shows of it. A line drawn in white is not
  ink and is not read, and how many is said. Dashed lines are read whole, and that is
  said. A page that gives no size of its own is refused. A file with no pages is said
  to be perhaps cut short.
* **A drawing is traced over on its own storey only (J14-9).** `--on` a drawing of
  another storey is refused. In the window what is traced takes its source from the
  drawing the pick is on; a pick off every sheet is drawn in the window, not traced.
* **A drawing whose stored page size is not its file's is said (J14-10)** by `drawings`
  and is not shown in the window.
* **No measure of time is what a material is bought by (J14-11)**: hours and man-hours
  however spelled, minutes, days, shifts, weeks, months.
* **A source loses nothing (J14-12; this replaces calls 41 and 44 on it).** A note of
  what was done is added after what the source says, and is not added twice running. It
  can grow, one note for each change of hand; it never drops what a person wrote. The
  twelfth judge's complaint was unbounded growth on repeated moves by one hand, which
  this still prevents.
* **What makes two names one name (J14-13; this replaces call 44 on it).** A comma, a
  hyphen, an underscore and a full stop part words and are not part of a name, except a
  point or a comma between two figures. A figure and the letters beside it are two
  words. So "Copper, 16 oz" and "copper 16oz" are one; "Type X" and "Type-X" are one;
  "a.b" and "ab" are two; "Pipe 1,2" and "Pipe 1, 2" are two.
* **A move in the window is sourced as a move in the window (J14-14)**, not as traced.
  Removing a drawing says how many things were traced from it; they stay.
* **Not done:** a JPEG turned by its EXIF tag is still shown unturned (said in STATUS);
  the words on screen ("drawing-0003", "(os error 2)") go to the language pass.

### Call 47. A model brought in: what is read, and what is listed instead (2026-11-11)

The second half of step 3: IFC models from other trades and designers.

**Searched first.** BIMCraft reads IFC2x3 and IFC4 (`import.rs`, 4,895 lines, with
`import/shapes.rs`, 931) over its own STEP reader (`step.rs`, 408). Its import builds
BIMCraft's own model and measures every shape it can (curved walls, breps, meshes,
splines). IfcOpenShell (LGPL) reads everything and is never linked; it is installed in
scratch as a separate program and was used once here, to check that the made-up model
written by hand is IFC4 (0 messages from its validator).

**What was decided.**

* **The STEP reader is ported whole** into a new crate, `plumbline-ifc`, with its three
  tests; two `if let` chains are rewritten for the 2021 edition, nothing else. PORTMAP.md
  and NOTICE say so.
* **The importer is ours, and reads the plain shapes.** A wall whose body is a rectangle
  pushed straight up; a door or a window by its width and height, filling a hole in such
  a wall; a slab whose body is an outline pushed straight up or down; a space, as a
  point inside it; a duct, a pipe, a tray or a cable whose body is a circle or a
  rectangle pushed along its run. That is how nearly everything in a house model is
  made. Porting BIMCraft's shape-measuring (some five thousand lines) is not done; what
  it would have read and this does not is listed, never dropped.
* **Everything not brought in is listed**, by what the file calls it, how many, and why:
  no element for it here (a beam, a stair, furniture), a body that is not one of the
  plain shapes, a wall that leans, a door in no wall that came in, a hole with nothing
  in it.
* **A model is as sure as whoever brings it in says** (`--status`, `--source`), and each
  value's source also names the thing in the file it was read off.
* **Lengths are turned into feet** from the unit the file says (metres, millimetres,
  feet or inches as a converted unit), kept to a ten-thousandth of a foot, and that is
  said. A file that names no length unit is in metres, as the standard has it, and that
  is said too.
* **The top storey's height is Unknown.** Each storey is as tall as it is to the next
  one up; nothing in the file says how tall the last is.
* **A system the file names is used where it is one this program has** (supply air,
  exhaust, cold and hot water, sanitary, vent, refrigerant, power, lighting). A run on
  any other sort of system comes in on no system, and how many is said.
* **A thing the model puts on no storey goes on the highest storey whose floor is not
  above it**, and how many were placed that way is said.
