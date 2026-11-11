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

