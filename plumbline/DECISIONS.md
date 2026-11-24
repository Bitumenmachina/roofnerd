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

### Call 48. The language pass: one place where every word on screen is said (2026-11-11)

Step 3's language pass (ruling 4: every word on screen is a builder's or estimator's
word; no software term anywhere a user sees).

* **Every string either window draws goes through one function, `theme::spoken`.** It
  is the only way text reaches the screen (`theme::text`), so a new message cannot
  miss it. It says a thing's path in the file (`wall-0008`) as the thing's name ("wall
  8"), says a name followed by its own path once, says a figure's name in a formula
  (`plan_length`) as its words ("plan length"), and leaves out the machine's number for
  a failure ("(os error 2)"). The field a person is typing in shows what was typed.
* **A path is never shown.** It was under each title, beside each row of the sheet and
  under each clash. A clash's second line says what the two things are (duct and
  equipment).
* **A formula can be typed as its words.** Two names never stand side by side in a
  formula, so words in a row with only spaces between them are one name: `plan length *
  stretch out` is `plan_length * stretch_out`. Both are read; the screen shows words.
* **The places the pointer catches on are said as at a drawing board:** on an end, on a
  middle, where two cross, on a centre, square to it, on a line, on the angle.
* **"plane(s)" and "opening(s)" are written out** (1 plane, 2 planes; the one opening,
  each of the 3 openings). One test's expected words change with this.
* **A model's source names the thing as the model does, less its "IFC"**, and its
  number in the file: `"Front door", door 113 in the file`.
* **The command line is not a screen in this sense.** It is how the demonstrations and
  the tests drive the program; its words are plain but it shows paths, because a path is
  what its commands take. The windows are what a user sees.
* **Not yet settled by this pass:** kinds of element are still said as the model names
  them where a message is built in the core ("a Wall", "Kind"); a judge is to list every
  word on every screen (release-candidate item 9) and what it finds is fixed then.

### Call 49. Field entry: what a laser types, and what a measurement does to the job (2026-11-11)

Order step 4 (ruling 10: any field that takes a length accepts a laser in keyboard
mode; marking a value Known works in the window). Written before it is built.

**What a laser in keyboard mode types**, from Leica's own Bluetooth guides for the DISTO
D810 touch and S910 (read 2026-11-11), not from memory:

* The laser joins the computer as a keyboard and types each reading where the cursor is.
* In its *text* setting it types the reading as its display shows it, unit included and
  with no space: `7.123m` on a laser set to metres, `2'2"13/32` on one set to feet and
  inches. The fraction comes after the inch mark.
* In its *spreadsheet* setting it types the figure alone, `7.123`, with no unit.
* After each reading it sends Tab, Enter, or nothing, as it is set.
* The decimal mark is a point, or a comma if it is set so.

What follows from that:

* **A fraction after the inch mark is read:** `2'2"13/32`, `26"13/32`, with or without
  a space between its parts (`2' 2" 13/32`; not inside a figure or a fraction: `1 / 2`
  is not read). It was refused as "something after the inches". The inches before the mark are
  then a whole number: `2' 6 1/2" 1/4` is refused.
* **Tab ends an entry as Enter does.** Where an entry takes more than one thing (a
  storey's name, floor and height), Tab goes on to the next, and ends the entry after
  the last.
* **A figure with no unit is still refused** (rule 4; SPEC 5.2: a number with no unit is
  never read as feet). A laser in its spreadsheet setting sends no unit, so the refusal
  now says to set the laser to send its unit. This is the ruling's "with units".
* **A comma is still refused, and now says why:** `1,234 ft` is a thousand feet to one
  reader and a foot and a bit to another. The laser's decimal mark is to be set to a
  point.
* **Metres are still refused by name.** Metric is proposal 5 and Patrick's to rule on
  (Q19); a laser bought here can be set to feet and inches. Converting would be exact,
  but a job that shows feet with sources that say metres is a decision about the product.

**What a measurement does.** One verb in the core, `editing::measure`, used by the
command line (`plumbline measure`) and by the window:

* It takes an element, the name of one of its entered values as the screen shows it
  ("storey height", "sill", "height above its storey"), and either a length or the word
  `same`.
* **A length replaces the value and the value is Known**, with the source given. Any
  entered value that is one length can be measured: a storey's floor and height; a
  type's thickness, width, height, diameter or depth; a wall's height and offsets; an
  opening's place along its wall and its sill; a run's heights; a placed thing's height.
* **`same` keeps the value and marks it Known**: the person looked and it is as drawn.
  This works on any entered value, a line on the plan or a position included. A value
  that is Unknown has nothing to be the same as, and is refused.
* **A line or a position is not measured in one length** (it is two or three figures).
  It is moved with the move verb, as before, and can then be marked `same`.
* **A storey's height is the floor-to-floor a person shoots.** The floors above are
  where that puts them: the next storey up sits at this floor plus the measurement, and
  is as sure as the less sure of the two; storeys above that move by the same amount and
  stay as sure as they were, with the move said in their source. If the next storey up
  is already Known somewhere else, the measurement is refused and says which three
  values disagree: one of them is wrong, and the program does not pick. Before this a
  storey's height was drawn nowhere: walls stand to the next storey's floor.
* **A run's height can be measured to its underside or its top.** A laser on the floor
  hits the bottom of a duct, not its centre line. `bottom`, `top` or `centre` before the
  length says which; the run then holds its heights to that line (BIMCraft's
  `justify`), and the centre is worked from its size, as sure as the size is. On a run
  whose two ends differ, the measurement has to be to the line it already holds, or the
  other end would be moved by a guess; it is refused otherwise.
* **The source.** At the command line, `--source`, as everywhere. In the window:
  "measured on site" for a reading; for `same`, the source keeps what it said and adds
  "; found as drawn (checked on site)"; and whatever the person types after a comma
  goes with it (`8' 10 1/2", JM with the laser`). (Corrected after the sixteenth
  judgment: this said the source for `same` was "checked on site, as drawn".) The program does not write the
  day: it would be the machine's clock stated as a fact (Q19).
* **Nothing new is worked out in the window.** The clash list and the sheet are read
  from the job after the verb, as after any edit.

**Not done here:** a wall's or a run's length measured as one length (it moves an end,
and what is joined to it); a laser shot with no entry open (the characters go to
whatever the keys do); Bosch GLM over Bluetooth (proposal 17).

### Call 50. The fifteenth judgment, part A: what was decided in fixing it (2026-11-11)

Twenty-five findings on tables and a model brought in (STATUS.md, "Fifteenth judgment,
part A"). What each fix decided:

* **A comma in a figure is a thousands mark or the figure is not read** (J15-1).
  `9,05` and `1.204,50` are how half the world writes nine and a bit and twelve hundred;
  dropping the comma made the first 905. A comma is read only where it can part
  thousands (one to three figures, then threes); any other is refused by row, and the
  refusal says to write a point. A list parted by semicolons says, where it says what
  was read, that such a list often has decimal commas, because `9,050` in one is nine
  and a bit and still reads as nine thousand: that one cannot be told apart, and is
  said, not guessed.
* **What the person bringing a list in says of it stands over what the list says of
  itself** (J15-2). `--status` and `--source` with a list that has its own "How sure"
  and "Where from" columns: the person's word is used for every row and the result
  says the columns were not read. With no word from the person the list's own columns
  are read, as before: that is how the program's own price list comes back unchanged.
* **A row is called by its number in the file** (J15-10), blank lines and a table that
  starts below a sheet's first row included. The reader of a table gives each row's
  number with it.
* **In a workbook, the first sheet on show is read**, not a hidden one before it; a
  hidden sheet is read only when asked for by name, and is said to be hidden (J15-9).
  **A cell shown as a percentage is read as shown** (`9.05%`), and **a date is read as
  the words "a date or a time"**: neither can then be taken for a price, and the row is
  refused for it by number (J15-8).
* **"Item" is a part number when there is a "Description" beside it** (J15-20). Of two
  columns that could both name the thing, the one headed with the plainer word for
  what it is wins and the other is listed as not read.
* **One thing priced twice in one list at two prices: the first is taken, the second is
  not used and says which row it disagrees with** (J15-25). The program does not pick
  the later because it is later.
* **A supplier's row cannot say how a material is ordered** (J15-13): that is the
  job's to say. The row's price is read and a note says the rest was not.
* **`lbs` and `lb.` are the pound.** A full stop after a unit, and the plural of the
  pound, are the list's way of writing and not another unit. Nothing wider: a general
  table of unit spellings is its own piece of work.
* **Names have no space before or after them**, wherever a thing is added: two names
  that differ only there read as one, and did not survive a table out and back.
* **A cell a spreadsheet would run as a formula goes out with an apostrophe before it**
  (`'=SUM(...)`), the usual guard, and the apostrophe comes off when the program reads
  the table back. The one loss this makes: a cell that really began with an apostrophe
  followed by `=`, `+`, `-` or `@` comes back without it.
* **A prior estimate's total rows are not lines** (J15-5): a row called total,
  subtotal or grand total, or beginning with one of those words and giving no quantity
  and no unit cost. "Total station hire, 2 days" is a line. **An amount that is not
  its quantity times its unit cost is kept as written and said**: the program does not
  know which of the three was written wrong.
* **Lines of a prior estimate that go by one name are added together before they are
  set against the job** (J15-6), since the job's lines of that name are added together
  too. **A difference is as sure as the less sure of the two it is between** (J15-7).
* **Two assemblies cannot have one name** (J15-14): rows of one name that disagree on
  what the assembly goes on, or on its cost code, are refused, all of them.
* **No spelling of time is what a material is ordered by** (J15-23, J14-11 again). The
  list of spellings is given up for a rule: a word that is a measure of time, or ends
  in one after a word of three letters or more (workhour, manweek, crew-day), or is one
  of the short forms estimates use (MH, LH).
* **A storey says how high its floor is, and is also placed; the two can differ**
  (J15-3). The floor's height in the job is what the storey says. What stands on the
  storey is measured from where the storey is placed. A note says when they differ.
* **Another trade's model stands on the job's own storeys** (J15-21). A storey in the
  model with the name and the floor height of one the job has is that storey, and
  nothing is made twice; with the name and another height, the model is refused, and
  the refusal gives both heights. Two storeys of one name inside one model: the later
  is named with its height.
* **A body of more than one piece is not read** (J15-11), and is listed with that as
  why; reading the first piece and dropping the rest was a silent loss.
* **What the model names and gives no place is listed too** (J15-12).
* **What a model calls a kind of thing is said in words** (`IFCBEAM` is "beam"), from a
  short table of the commoner ones; any other is its name less the "IFC", marked "as
  the model calls it". One test's expected words change with this.
* **A room's point is inside the room** (J15-16): the middle of the outline's corners
  where that is inside, else the middle of the widest stretch of the room across.
* **A floor with an opening cut in it comes in whole, flagged Assumed, and says in its
  source that it is more than the floor**; a floor whose own outline has a hole is not
  brought in. This program has no floor with a hole in it yet.
* **Upright and level are within an eighth of an inch in eight feet.** The model reader
  took a wall leaning two and a half degrees as upright.
* **The model's name for a wall, a floor or a run is its name here**, with its number
  in the file only where two would otherwise go by one name.
* **A number used twice in a model's file is said** (J15-22): the reader keeps the
  later of each.
* **A name already taken is refused in plain words**: "there is already something
  called "ridge" under roof: two things in one place cannot go by one name", where it
  said "roof-0002 already has a child named "ridge"". One test's expected words change.
* **One rule for a comma in a figure.** The rule was already in `units::parse_price`
  (the tenth judgment); the table reader had its own, looser reading beside it, and
  that was the fault. Both now ask `units::commas_part_thousands`.
* **A sheet lined up again after things were traced over it** (J15-4) is treated as a
  sheet whose scale was changed: nothing traced is moved, what was traced is what the
  verb gives back, the drawing keeps it as traced where the sheet lay before, and it
  is said by `drawings`, by `show` (a flagged row on the drawing), by `takeoff` and in
  the window. A drawing put back at the scale and in the place it had stops saying it
  (J15-15): what was traced then is where the sheet shows it again.

### Call 51. The sixteenth judgment: what was decided in fixing it (2026-11-12)

Ten findings on field entry (STATUS.md, "Sixteenth judgment"). What each fix decided:

* **A floor, its storey's height and the floor above say one thing three ways, and are
  kept agreeing whichever is measured** (J16-1). Call 49 made the floors above follow a
  storey height given as a length, and checked nothing on any other path. Now:
  a storey height, measured or checked (`same`), puts the next floor at this floor
  plus the height; a floor, measured or checked, makes the storey height below it the
  way up to it, and its own storey height the way up to the next floor (or, where its
  own height is Known, puts the next floor where that says). Each value so worked is
  as sure as the less sure of the two it is worked from, and its source says which two.
  A value Known in its own right is never overwritten: where it disagrees, the
  measurement is refused and names the three.
* **Agreement is to an eighth of an inch** (J16-5). A millionth of a foot refused a
  laser that read a thirty-second off a tape. Two Known values within an eighth are
  both left as they are, and that is said.
* **A second shot corrects the first** (J16-5). A floor that is Known only because the
  storey height below it was measured (its source says exactly that, and nothing has
  been said of it since) follows that height when it is measured again. Once the floor
  itself has been measured or checked, it is Known in its own right and is not moved.
* **A plane solved from its roof's footprint does not take a measured eave height**
  (J16-2): the roof is solved again from the footprint and the measurement would be
  thrown away. It is refused, and not offered in the window. The roof's own eave height
  is set with `set`, which solves it again, as before.
* **A run or a placed thing pulled off its joints by a measurement is said** (J16-3):
  what it met before and no longer meets is named, with what that will do to the
  takeoff and the clash list. Nothing joined to it is moved: the program does not
  know that the elbow moved with the duct, and a person on site does.
* **In the window a height says which line it is to**: a run's to its centre (or the
  bottom or top it is held to), a placed thing's to its underside. One label had meant
  both.
* **Walls that stand to their own height through a floor that has moved are said**
  (J16-4), each with by how much. Where the job's own check then refuses the
  measurement as one wall over another, the refusal says what is wrong (the walls'
  height) and what to do first.
* **A roof's eave height does not follow a floor** (J16-9); when a floor moves, that is
  said, so the roof is checked against the walls under it.
* **A floor that is moved is written in the unit it was written in** (J16-10).
* **`same` with the line a run is already held to is taken** (J16-10).
* **A reading typed with nothing picked to take it does nothing** (J16-6). Call 49 let
  it fall on the keys, where a `-` lowered a roof's pitch. A character key now acts
  only when nothing follows it within an eighth of a second: a hand does not type two
  keys that fast and a laser types nothing slower. A burst with no entry open is not
  used, and says so once; Escape and the key that saves still act. The cost is that a
  single key acts an eighth of a second after it is pressed.
* **The key that ends an entry ends it** (J16-7). What comes after it in the same
  breath is a second reading: it is not added to the first, and the message says a
  second reading came and was not used, under what the first did.
* **Who measured comes after a comma and a space** (J16-8): `8'10"1/2, 2nd shot by JM`
  is a reading and a note. A comma hard against a figure is still part of the figure,
  and refused there.
* **The program's own quoting does not reach the screen** (J16-8, J15-B3): a backslash
  before a quote mark is taken out where every word on screen is said.

**Not done:** a storey height of half an inch is still taken (a step between two
levels can be small, and no figure is plainly too small); a run measured Known on a
storey whose floor is Assumed keeps its clash "to verify" with no word of why on the
clash itself (the storey's row says it is Assumed).


### Call 52. The fifteenth judgment, part B: the words on screen (2026-11-12)

Thirteen findings (STATUS.md, "Fifteenth judgment, part B"). The language pass of call
48 put one function between every word and the screen and then left holes in it. What
was decided in closing them:

* **A thing is called what it was called after it is gone.** The names `spoken` knows
  are added to, never replaced: the message that says a duct was removed is drawn after
  the duct has left the job. A path is never given out twice, so an old name cannot be
  said of a new thing.
* **A path that owns something, and a path given a capital, are still paths**
  ("roof-0072's footprint", "Plane-0073: ..."), and take the thing's name.
* **The job is "the job".** Its element is named by its own path in the file; on screen
  that path is said as "the job".
* **The program's own quoting, its names for kinds of thing, and its way of writing
  money are put right in that one place**: `\"` is `"`; `RoofPlane` is "roof plane"
  and "a Wall" is "a wall"; `28.00 $/SF` is `$28.00/SF`; `lb` standing alone is `LB`,
  as the owner's list of units has it. The job's file and the command line keep what
  they had: these are how the screen writes, not what is stored.
* **The window does not send a person to the command line.** Three messages named a
  command to type. Each now says what the key does and that the rest "is not in this
  window yet", which is true and is a builder's sentence. (Call 48 held that the
  command line is not a screen; the judge's point stands that a window that points at
  it has made it one.)
* **Why a file could not be opened is said in words**: it is not there; it is there
  and may not be opened; it could not be opened. The machine's own sentence, and the
  whole path of a folder on this machine, are not shown: a file is called by its name.
  A plan that cannot be read says what to do next (put it back beside the job, or lay
  it again; print it to a new PDF; save the picture again).
* **A plan is named as it lies from the job's folder** (J15-B10), which is what the
  hint under the entry shows. It was looked for from wherever the program was started.
* **What is typed in the window is sourced "set in the window; not yet checked against
  a drawing"** (J15-B11). It said "set in the 3D view" of things typed on the plan.
  Jobs saved before keep the words they were saved with.
* **One name, once** (J15-B8). The sheet drew a thing's name and under it its path,
  which `spoken` then said as the name again. The path line is gone. STATUS.md's claim
  that it already was is corrected there.
* **Words changed:** "element(s)" on the Money tab is "item(s)", or the sentence is
  recast ("nothing has an allowance..."); "networks" is "joined lengths"; "under this
  wall" is "in this wall"; "pan" and "zoom" are "slide" and "come closer or back off";
  a part or a product "not selected" is "not specified", the word a specification
  uses, and no longer the window's word for what is picked; "select" in the hint is
  "pick"; a made-up job's cost code source no longer names one of this project's own
  files; "mask" is "cover".
* **Hours are shown to two places**, as a time sheet has them. They are held and added
  to four.
* **Only the keys that apply are offered**: no "Next roof plane" on a job with no roof,
  no condition, price or material for a drawing.
* **A size is read as it is said** ("2 by 2 by 3"), and one that is not a size says so,
  not that it "is not a plain number".

**Not settled here, and why:**

* (Settled since, below: a thing drawn with no name.)
* **"class"** for what a line's cost is (material, labor, subcontract). Estimating
  programs say "cost type" or "category"; which the trade's word is was not checked
  against a source, and the word is the model's and the command line's too (Q20).
* **"storey", "centre"**: British spelling in a program in US units (Q20).
* **Two words for one thing** (elbow and bend; storey, level and floor; join and
  connect): each is a change to the model's own words and to what demonstrations
  expect; listed in STATUS.md as open.
* **Places as bare decimal pairs, clash depths in decimal inches**: open.

**Added the same day: what a thing drawn with no name is called** (J15-B1's last part).

* **A wall, an opening and a run are their kind and a number, counted from 1 on their
  storey**: "wall 1" to "wall 4", "duct 1". They were numbered by their place in the
  job's file ("wall 9" to "wall 12" on a job whose first eight things are types), which
  is how the file tells things apart and not how anyone on a job does.
* **A fixture, a piece of equipment and a fitting are called what they are**:
  "Lavatory", and the next one on the storey "Lavatory 2"; "6 in round elbow". They
  were "fixture 18" and "fitting 99" with what they are on a second line.
* **A named run that is cut**: the second piece is "<name>, piece 2" (then 3). It was
  "<name>, cut at <its number in the file>". A run with only its kind and number gets
  the next number.
* **Where a thing is said by what it is and by its name** (a clash), a name that is
  what it is, or that with a number after, is said once: "Boiler", not "Boiler
  (Boiler)".
* Jobs saved before keep the names they have. Every demonstration and test that named
  such a thing has its expected name changed with this.

### Call 53. Floors, rooms and roofs drawn in the window: what is ported and what each tool does (2026-11-12)

Order step 5. Written before it is built.

* **The floor verb is BIMCraft's** (`floor`, `crates/core/src/commands.rs:7556`): a
  floor of a type on a storey, by a boundary that is given or, with none, by **the
  outline its walls enclose** (`floor_outline`, `commands.rs:2467`: along the centre
  lines of the walls, the outer outline of the largest piece they make). Ported with
  the two changes every ported verb has: lengths with their units and feet on the
  plan, and every value with a status and a source. What is not taken: BIMCraft cuts
  the courtyards its walls enclose out of the floor; this program's floor has no hole
  yet, so where the walls enclose a courtyard the verb refuses and says so, and the
  floor is given by its boundary. Slope, the span arrow, "structural" and the sketch
  are not taken.
* **A floor by its walls is as sure as the least sure of those walls' lines**, and its
  source says it is the outline of the walls. A floor by a boundary is as sure as
  whoever gives it says.
* **A second floor on an outline a floor of that storey already has is refused**: it
  would be counted twice (the model reader already holds this for a model brought in).
* **In the window: Floor is one click.** With the tool in hand and the thickness said,
  a click on the storey puts the floor its walls enclose. A floor by picked corners
  stays at the command line: the walls are what is traced, and the floor follows them.
* **Room is a click and a name.** The verb is there (`room`); the tool picks the point
  and asks for the name.
* **Roof is its footprint's corners and "pitch, eave height".** The verb is there
  (`solve::new_roof`, a roof solved from its footprint); the tool picks the corners,
  closes on the first, and asks for one pitch for every side and the height of the
  eaves above the job's zero. Every side at one pitch is a hip roof; a gable is made
  on a plane afterwards with the key that already does it.
* **What is drawn over a drawing is traced**, as walls are: Assumed, its source the
  drawing.


### Call 54. The verify-in-field list: what a question is and what rides on it (2026-11-12, before it is built)

Order step 6 (the loop's last sentence: "the program produces a verify-in-field list:
each question, where it is, and the dollars riding on it, sorted by dollars"). Thought
through while step 5 was building, and written down so it is not lost; the
demonstration with its hand figures goes into STATUS.md before the list is built.

* **A question is one of two things.** A clash that rests on anything not Known (the
  ruling's own words: a clash between Assumed elements is a field question). And a
  thing with a value that is Assumed or Unknown on which a priced line rests: the
  storey whose height was scaled off a plan and whose walls are priced by their face.
* **Where it is**: a clash's place, east, north and height, and its storey; a thing's
  storey and what it is called.
* **What to do about it is said with it**: which of its values are not Known, each as
  the panel names it, so the person on site knows what to measure or check.
* **The dollars riding on a thing** are what the estimate's lines that rest on it come
  to: every live line whose quantity is worked from a figure that is worked from that
  thing's values. The core already records what each worked-out figure rests on (each
  `Derived` names its inputs), so this is read from the job, not kept beside it. The
  record is by a thing's group of values (a storey's level: its floor and its height
  together), not by each value; the list says which values of the group are not Known.
* **The dollars riding on a clash** are what the lines on the two things in it come to:
  what is priced on what may have to move. The cost of the fix is not known to the
  program and is not guessed.
* **No dollars is said, not zero**: a question nothing priced rests on is listed after
  those with dollars and says nothing priced rests on it yet.
* **Sorted by dollars, largest first**; a figure that is itself flagged shows its flag.
* **One list, three places**: the command `vif`, a tab of the sheet window, and each
  row picks its thing in the model window, where its values say Measure or Check. A
  measurement that makes its values Known takes the question off the list.

What "riding on it" means is the owner's to say; this is the working assumption (Q21).

### Call 55. The screen, redrawn: what building games get right, and what is taken from them (2026-11-12)

Patrick, 2026-11-12, after seeing the screens: "I do not see how the interface is
showing a more manageable and Construcion oriented screen and using English storey and
the way things are worded does not seem to fit with good UI and software design
protocols." Then: "Use online search for best UI for video game world building
interfaces and you make the call proceed with previous plan to deliverable."

**What was read** (searches of 2026-11-12; player guides, developer notes and player
complaints, not formal studies, and taken as such):

* The Sims 4's build mode: tools in one bar with hotkeys; the catalog steps back when
  a thing is being placed so more of the build shows; its makers' own rule was
  "Flexible. Fast. Fun. Forgiving." (primagames.com build-mode guide; simscommunity.info).
  What players dislike: clutter, and filters that reset.
* Cities: Skylines II: a tool bar by category; an info view (an overlay on the world)
  opens by itself when the tool it bears on is taken; an advisor holds the teaching by
  subject; the 2026 rework by its new studio made icons "more expressive" and
  streamlined the first run (paradoxinteractive.com developer diary; colossalorder.fi).
* Satisfactory and Factorio: a build menu in tabs, and a hotbar of number keys for the
  few things used most, as a shortcut and never a step in the way; a search key
  (satisfactory.wiki.gg, Hotbar and Construction Tool).
* City builders at large: categories by use, kept shallow; side panels that fold away;
  a list of problems where a click takes the view there, whose rows do not jump about;
  "data, not a recommendation" (Civilization VII's developers).
* Planet Coaster and Paralives: snapping that can be let go of; things resized in the
  world; and the standing complaint that freedom without a grid makes plain work slow.
* Teaching: small steps that can each be seen to be done, shown one at a time as they
  are needed, never a wall of text, and skippable (uxdesign.cc on onboarding).

**The call.** The way a building game is worked, on the look this program's rules ask
for (an instrument: line types, matte, dense, drawn like a drawing; no sky, sun, grass).

1. **A build bar along the bottom**, in the middle: the tools as buttons with a drawn
   mark and a word, grouped by trade (Structure, HVAC, Plumbing, Electrical, Edit),
   each with a number key. It replaces the list of some twenty rows down the left.
2. **What the tool in hand draws with is a strip over the bar**, each value a field
   that is typed into where it stands. It replaces "not set yet" under each row and
   the typed line of commas.
3. **What is picked shows on the right as fields and buttons.** A value is changed by
   clicking it; what can be done to the thing is a button on it (Measure, Check,
   Delete, Move). The "Keys" list of letters goes; a key is shown on its button.
4. **The price is on the top bar at all times**, with how much of it is not yet sure.
5. **Next steps**: a short list, top left, of what this job needs next, read from the
   job itself (a level; a drawing and its scale; walls; floor, rooms, roof; prices;
   what to verify), each ticked when it is so, the first not done said in one line.
   It can be folded away. This is the in-app help the release asks for.
6. **Views laid over the building**: by trade (as now), by how sure (Known, Assumed,
   Unknown), and clashes. The clash view comes up with the Clashes list.
7. **The verify-in-field list is the list of problems**: sorted by dollars, a click
   picks the thing and takes the view to it (call 54).
8. **The look**: flat panels with hairlines, no shadows, no pills; ink on paper for
   the plan; figures in columns; the 3D view on a neutral ground with no horizon.
9. **Forgiving**: Undo and Redo are buttons on the top bar; Esc backs out one step;
   snapping can be let go of with a key.

**The words.**

* **US English.** Level (not storey), center, color, meters.
* **The trade's terms**: Level; Elevation; Floor-to-floor; Centerline; Rough opening
  where that is what is meant; Verify in field (VIF); Takeoff; Markup, Overhead and
  Profit.
* **The plain verbs every program uses are used**: Save, Undo, Redo, Delete, Cancel,
  Open. Avoiding them was a misreading of "no software term": that rule is against
  the program's own insides showing (paths, type names, error codes), not against the
  words on every toolbar a builder already uses.
* **A message is short**: what happened, in a few words; when it could not be done,
  "Can't <do it>: <why>. <What to do>." No "refused:".
* **A label is one to three words.** Where a thing came from stays a sentence, since
  it is the person's own.
* The model, the job file and the command line keep their words (storey, centre): the
  screen says them in US English through its glossary, in the one place every word on
  screen passes. The documents a user reads (README) follow.

**How the demonstrations find their buttons.** Every window demonstration clicks by
pixel, so every change to the layout broke them. With `PLUMBLINE_MAP=<file>` the
program writes where each named button is on the screen, and where the plan's zero is
and how many pixels are a foot; a script then moves the real pointer there and
presses the real button. The path is still the one a person uses; only the place to
click is asked of the program in place of being copied into the script by hand.

**Order.** The screen first, since floors, rooms, the roof, the verify list and the
next steps all go into it; then the order of work as it stood.

**Call 55 as built (2026-11-12).** What differs from the list above, and why:

* **The build bar's groups are Building; Mechanical, electrical, plumbing; and Edit**,
  not one group a trade. A fixture is a plumber's lavatory and an electrician's
  luminaire, and a piece of equipment is anybody's: a group a trade would have put
  each of them under one trade or under all three. Every tool shows at once, in one
  row; nothing is behind a tab.
* **The number keys 1 to 9 and 0 take the first ten tools in hand.** They used to put
  each trade on show or off; that is now a row to click under Show, and has no key.
  (Named in STATUS.md as a changed behaviour.)
* **A tool goes to the plan and to a level by itself.** Clicked in 3D it turns the view
  to the plan; with the whole building on show it draws on the lowest level and says
  so by showing it; with no level in the job it opens the form for the first one. It
  used to refuse each of these and say what to do first.
* **A form that will not do stays open**, with the caret in the field at fault, in
  place of being thrown away with a refusal.
* **Move and Delete are on the build bar, once.** They are not repeated on what is
  selected.
* **Each mark is drawn the way a plan draws the thing** (a wall a heavy line, a window
  a line with its glass, a duct two lines, a roof a gable end), out of the same flat
  strokes the rest of the screen is made of. No picture font is linked, so nothing is
  added to the licences.
* **The 3D view has a key light and a fill, a pale ground with its grid and no
  horizon.** The sky, the sun's disc, the bloom and the grass are gone in both
  graphics modes, which is the fixed rule ("No sky, sun or grass") as much as this call.
* **On the plan a floor is paper**, so what is drawn on it reads as on a drawing.
* **The price on the top bar is the estimate's price** with its flag; beside it, how
  many clashes are still questions. The dollars riding on what is not yet sure come
  with the verify-in-field list (call 54), which this bar is the place for.

### Call 56. Bringing in from the window, and a new job from the window (2026-11-12)

The fixed rule: "Imports matter as much as exports. People bring in what they already
have." Until now a plan could be added in the window and everything else (a price
list, an assembly library, an estimate made before, a model) only at the command line,
where an estimator is not.

* **Under "Bring in", on the left, a button for each**: Price list, Assemblies, Prior
  estimate, Model (IFC). Each asks for its file where the button stood; the file can be
  typed (found from the job's own folder first) or dropped on the window. Each is the
  verb the command line calls, all of it or none of it.
* **A file dropped with nothing asked for**: a plan is added as a drawing; an `.ifc`
  is brought in as a model; a CSV or a workbook could be three things, so it is kept
  and the screen asks which, by the same three buttons. The program does not guess
  what a table is from its headings and file it as that.
* **What is brought in in the window is Assumed, and its source is the file's name**
  ("ridge.xlsx, brought in in the window"). A price list that says for itself how sure
  each row is (the program's own, written out) keeps what it says. A window has no
  place yet to say "Known, from their list of 3 March"; the command line does.
* **A price list brought in in the window adds the materials it names** that the job
  does not have: on a new job that is how its materials get there. At the command
  line that is still `--add`, off unless asked for.
* **What was not used and what was not read is said in full over the model**, as the
  command line prints it: a row with no price, a column with no use, a beam this
  program has no element for.
* **A job file that is not there yet is a new job.** `plumbline-app house.json` on a
  file that does not exist opens an empty job called "house", written when it is first
  saved; with no file named it is `job.json` where the program was started. It used to
  stop with "cannot read". The top bar says "Not saved" until it is.

### Call 57. Patrick, 2026-11-12: the words are mine to choose; the focus is all sixteen divisions; the house is dressed

His words, after seeing the redrawn screen and the work on clashes and the verify list:

> "Why are you asking me on language when I said US construction it's a known language
> and you are the driver of this act like it. Showcase all your capabilities and really
> try to see how a human would experience it"
>
> "I do want to ensure wile clash detection is important here the focus is a full 16
> division building platform seems like that focus has been lost. Also I still see the
> 3d modeling but where is the texturing of an actual house or windows think high end
> residential construction"

Folded into `CLAUDE.md`. What each changes:

**1. Language.** No wording question goes to him again. The two that were open are
decided here:

* **A floor of the building is a level, everywhere a person reads**: the screen, the
  command line's output and its verbs, the documents. The job file keeps its key
  (`Storey`, IFC's `IfcBuildingStorey`): a file's keys are not language, and jobs
  already saved still load.
* **A cost line's "class" is its cost type** (material, labor, subcontract, equipment,
  other): the word US estimating uses. "Class" goes from every screen and from the
  command line.
* Q20 is closed. So is the habit: a word is chosen from the trade's own usage, checked
  against a published source where there is doubt, and written down.

**2. The focus: a full 16-division building platform.** The sixteen divisions are those
of CSI MasterFormat as published through its 1995 edition, checked against two
published lists on 2026-11-12 (en.wikipedia.org/wiki/16_Divisions;
archtoolbox.com/masterformat-1995-divisions), not written from memory:

| | | | |
|---|---|---|---|
| 1 General Requirements | 2 Site Construction | 3 Concrete | 4 Masonry |
| 5 Metals | 6 Wood and Plastics | 7 Thermal and Moisture Protection | 8 Doors and Windows |
| 9 Finishes | 10 Specialties | 11 Equipment | 12 Furnishings |
| 13 Special Construction | 14 Conveying Systems | 15 Mechanical | 16 Electrical |

(CSI replaced these with fifty divisions in 2004. He asked for sixteen, residential
builders still estimate by them, and sixteen it is. A line can carry a later code as
well.) This answers Q1: **the primary cost code is the division.**

* **Every cost line carries its division.** A line with none is said to have none, in
  the way a line with no cost type is: it is in no division's total, and the estimate
  says what it left out.
* **The estimate reads division by division**: the sheet groups its lines under the
  sixteen, the recap totals each, and the workbook, the CSV and the PM's lens carry the
  division. Material, labor and subcontract stay as the columns across.
* **Each division has assemblies behind it**, so a traced house prices across all of
  them: an exterior wall is framing (6), insulation, wrap and siding (7), gypsum board
  and paint (9); a floor is a slab (3) or framing (6) and its finish (9); a door and a
  window are 8; the site, the footings, the masonry, the cabinets, the appliances, the
  plumbing, the heating and the wiring each have theirs. The library ships with
  quantities and formulas and **no prices**: prices come from the price lists people
  bring in. The made-up house carries made-up prices so the whole sheet can be shown.
* **What the model does not yet have an element for is said, division by division**
  (earthwork, footings, stairs, cabinets, appliances): a division with nothing in it
  shows as having nothing, not as $0.00.
* Clash detection and the verify-in-field list stay as built. They are tools of the
  platform. The order of work goes back to the building: divisions first.

**3. The house is dressed.** The 3D view shows the house as it will be built, to the
standard of high-end residential work, inside the look rule (matte, studio light, a
neutral ground, no sky, sun or grass):

* Walls in their finish: lap siding, shingles, board and batten, brick, stone, stucco.
  Roofs in theirs: standing seam, shingles, slate. Floors in theirs.
* A window is a frame, sashes, muntins and glass, with casing and a sill; a door is a
  frame, a paneled leaf and casing. Fascia, soffit, gutters, corner boards.
* The finishes are drawn by the program (patterns made in code): no picture file of
  unknown licence is linked.
* **What a thing is dressed in is what its condition or material says.** A wall with
  no finish said is drawn bare and says so; the picture does not promise a finish the
  estimate does not price.

**4. Seen as a person sees it.** Every screen is walked cold and every picture read,
by its builder and then by judges briefed as a veteran estimator who has never seen
it. A demonstration that passes on a screen a builder would stumble over is a defect
in the demonstration.

### Call 58. The house is dressed in what the estimate prices (2026-11-12)

Patrick: "where is the texturing of an actual house or windows think high end
residential construction". Call 57 made it a rule; this is how.

**Where a finish comes from.** A wall, a roof plane, a window or a door is drawn in
what its own cost lines, its material or its assembly say it is finished with. The
words are the trade's: brick, stone, stucco, lap siding (clapboard, bevel, fiber
cement), board and batten, shingles; asphalt shingles, standing seam, slate, tile,
shakes, membrane; double-hung, casement, fixed, divided lites; panel, glazed. A color
word in the same text (white, charcoal, cedar, copper, ...) colors it. The reading is
done once, in the model (`plumbline_world::finish`), and the window draws what it
says; `plumbline finishes <file>` prints the same reading, with the line that said
it. **A thing with nothing priced on it that names a finish is drawn bare**, and its
panel says so. The picture then shows what the estimate has left out, the way a total
does: a bare wall on a dressed house is a wall nobody has priced the siding for.
Rejected: a finish field of its own on each element, set by hand. It would let the
picture and the estimate disagree, which is the one thing this rule exists to stop.

**Which face is outside.** The side of a wall that lies outside the outline its
level's walls enclose. A wall with both sides inside is a partition and takes no
cladding. Where the walls enclose nothing, which side is out is not known and a
cladding priced on the wall is drawn on both faces. (The thickness test this
replaces, "6 in or more is an outside wall", was a guess.)

**A window and a door are built, not painted on.** Frame, sash, glass and muntins;
stiles, rails and panels; casing outside and in; a sill; a knob. What nothing says
(how a window operates, whether its lites are divided) is not drawn: an unpriced
window is a frame, one sash and one light.

**Within the look rule.** Finishes are matte, drawn from patterns made in the program
(no photographs, no files brought in, nothing licensed): courses, joints and seams at
their real sizes, so a brick is 8 in by 2 2/3 in on the wall and a lap is its
exposure. Studio light, neutral ground, no sky. Edges and roof lines keep their line
types. With "Known, Assumed, Unknown" showing, everything goes back to flat color:
that view is about certainty, not finish. The plan is still a drawing.

**Addendum, the same day: BIMCraft's textures, read after this was built, which is
the wrong order.** `crates/core/src/texture.rs` in BIMCraft (at the pinned commit)
draws a material's 3D texture in code: ten kinds (brick, block, boards, concrete,
plaster, tile, roof, fabric, metal, insulation), 256 px, with repeating noise and a
chain of smaller sizes, laid at a size in millimetres the user sets on the material.
The method here is the same one (a pattern worked out in code, repeating, with every
smaller size, laid at real size), arrived at separately; no code was taken, so
`crates/app/src/dress.rs` is ours and is outside the port map (which covers the
building model and its editing). What BIMCraft has that this does not: a picture of
the user's own as a texture, and a texture turned and moved by hand. What this has
that it does not: the US claddings and roofings by name at their trade sizes (lap
exposure, modular brick, coursed ashlar, 16 in seams), relief, and the finish read
from the estimate instead of set on a material by hand, which is the point of this
call. The rule broken: search what is already surveyed before building. Nothing had
to be redone because of it; it is written here so the next survey is made first.

### Call 59. What the seventeenth judgment changed, and why (2026-11-12)

Each of these changes a behaviour or an expectation; each follows a finding in
STATUS.md ("Seventeenth judgment", parts A and B).

**A line names a finish only when the finish is what the line is** (J17B-4, 5, 7).
Call 58 took a finish from any priced line that held a finish's word, so "Stone sill
at windows" made a wall stone and felt underlayment made a roof asphalt shingles.
The reading now goes by a line's head: the words before its first comma and before
"with", "over", "on", "at". A head that holds a word for something else (sill,
flashing, nails, removal, paint, trim, underlayment, gutter, tear-off, hardware,
opener, screen) is not a finish. A head that names a finish is that finish, the one
named first. A head that names only the kind of thing ("Siding", "Roofing", "Window
unit") is refined by what follows its commas, the way estimators write: "Roofing,
standing seam, charcoal". Rejected: reading by division (a cladding only from
Division 4, 7 or 9). It would be cleaner and it would leave every line with no
division undressed, which on a first takeoff is every line.

**Outside is outside every enclosed piece, and inside a courtyard** (J17B-1, 2).
`building::enclosed` keeps every piece a level's walls make and every courtyard in
them, where the floor's outline keeps only the largest. A wall with both faces
outdoors (a garden wall) is dressed on both and says why.

**The words are level, cost type, center** (call 57, carried to the command line and
into every sentence the model writes). The verb is `plumbline level`; `storey` is
still taken. A new level's path is `level-0002`; a job saved before keeps the paths
it has. The job file's own keys and kind names (`pl::class`, `"Storey"`) do not
change: they are the file, not the screen. "Condition" stays: it is the estimator's
word for what is applied to an area, and Patrick's own in the deliverable.

**What is priced on a thing whose identity is in question rides on that question**
(J17A-14). The verify-in-field list counted only lines whose quantity is worked
from the thing. Where what the thing is, its type or its size is itself to be
checked, everything priced on it now rides on it: a price for a 50 gal heater is a
price for nothing if the heater is not that. The line's own status in the estimate
is still what its quantity and price rest on. Not done, and open: whether the
estimate's line should itself read Assumed when its thing's identity is. It would
turn most of a traced job's Known lines to Assumed, which may be the honest answer;
it is in QUESTIONS.md with this as the working assumption.

**Lines with no price yet are counted where they ride** (J17A-7): each question
says how many lines ride on it with no price, and the list's head says how many in
all, so "no dollars riding" no longer covers "what rides has no price".

**The workbook is formulas all the way down** (J17A-4, 6). A line with no price has
its amount formula and it comes to nothing until a price is typed. Every cost type
and every division is a formula whether or not it has a line yet, and comes to
"none" until one does, since nothing is not $0.00. "In no division" is the job cost
less the sixteen, so the two always come to the job cost. This replaces "a formula
only where it comes to the job's figure as written": the rule now is that every
formula comes to what is written with it, a figure, the word "none", or nothing.
`tools/workbook.py` works IF, OR and COUNTIFS to check exactly that.

**A division names what it leaves out, and the sixteen carry their status** (J17A-2,
3, 10): on the command line, in the workbook and in the PM's lens. A line given a
division and no cost type is named under its division as left out.

**A quantity is shown to four places when two would not multiply to its amount**
(J17A-9). 461.19 lb at $9.00 is not $4,150.70 on a calculator; 461.1890 lb is.

**A price is read one way** (J17A-17): an assembly library's unit cost goes through
the same reader as every other price, so "$1,23/LF" is refused, not read as $123.

**A made-up job is not written over a file** (J17A-16), as a new job is not.

**The estimate tab opens on the recap**, then the cost by division, then every line,
then the suppliers: what the job comes to is what the sheet is opened for. (Seen
when `demos/d5w.sh` could no longer find the recap under sixteen divisions and
every line.) **In 3D the roof is drawn when the top level alone is on show**: a job
with one level had no "All levels" to choose, and its roof was never drawn.

### Call 60. Assemblies: a starter library, picked from a list, and one word for it (2026-11-12)

**A starter library ships with the program** (`library/sixteen-divisions.csv`, and the
same file inside the program). Patrick's focus is sixteen divisions with assemblies
behind them; a new job had none, and a person had nothing to price with until they
found a library of their own. It is 30 whole assemblies (an outside wall in lap
siding, in brick, in stucco, over a stone wainscot; a partition; a slab; a framed
floor; four kinds of room; two roofs; ridge, hip and eave; a window; two doors;
runs, fixtures and equipment), 93 lines, a line in every division. **It has no
price in it, and never will**: prices are the estimator's, and the rule that no real
price enters the repository is kept by there being none at all. The figures in it
are arithmetic about how a thing is bought (32 SF to a sheet, 100 SF to the square,
81 SF to the yard at 4 in). Rejected: putting the library into every new job. It
would number thirty things ahead of the first level in every job file, and a person
who has a library of their own does not want ours under it; one click brings it in.

**An assembly is everything priced on one thing, whatever divisions its lines fall
in**, and a thing carries one. That was already how the model worked; the first
draft of the library was by division (a "Lap siding" assembly, a "Gypsum board"
assembly) and a wall could take only one of them. An estimator's wall assembly is
the whole wall.

**In the window an assembly is picked from a list** of those that go on what is
selected. The Condition button put on the first one that fitted, with no choice;
with five wall assemblies in the job that is no use. **And it can be changed**:
picking another takes off the lines the old one brought that the new one does not
have (and says which, since their prices go with them), keeps the lines the two
share exactly as they are, and adds the rest. Lines put on by hand are not touched.
The command line has the same as `apply ... --change`; without it a second assembly
is still refused, because at the command line nothing is undone with a key.

**One word: assembly.** Call 59 kept "condition" for what is applied to a thing, as
the takeoff programs say it, beside "assembly" for the library. On screen that was
two words for one thing a click apart (Bring in: Assemblies; the button: Condition;
the row: Condition). It is "assembly" everywhere a person reads, on screen and at the
command line; `set <path> assembly` is taken as well as `condition`. The job file's
key is still `pl::condition`: it is the file, not the screen.

### Call 61. A figure is named by its words; a run cut in two is one length (2026-11-12)

**A figure's name is its words.** The job file and a formula's stored text join the
words of a name with an underscore (`net_area`, `plan_length`, `stretch_out`,
`unit_cost`). That spelling is the file's. Nobody on a job writes it, and the
seventeenth judgment (J17B-21) found it in `show`, in `price` ("priced by its
net_area"), in the refusals that list what a thing has, and in a line's worked
formula. Everywhere a person reads, on the command line as in the window, a name is
its words: "net area", "plan length × stretch out ÷ 12". Either spelling can be
typed: a formula already took `plan length * stretch out`; `set <path> "unit cost"`
and `--with "turn up=8 in"` now do too. The job file is unchanged, so every job
written before reads as it did. A kind of thing is said by its words in the same
places ("roof plane", "Duct", not the program's own spelling). "The file's measures
did not match its geometry … recomputed" is "the file's figures did not agree with
what is drawn … worked out again".

**A run cut in two is one length of duct.** `split` is bookkeeping: it moves nothing.
The clash check measured how deep a pipe was in each piece on its own, so a pipe
through the middle of a duct read 5 in before the cut and 1 in after it (the inch it
reaches into the end of each piece), and a field hand was told it was nearly clear
(J17A-20). Where a run carries on, within half a degree of one line and at one size,
into a run joined to its end, depth is measured against the whole length, and the
place reported is where it is deepest in that length. The pipe is still in two
things the job holds, so it is still two clashes, one with each piece; both now say
5 in at the crossing, and the verify-in-field head total already counts each line
once. Runs joined at a corner are not carried on round it.

### Call 62. Patrick, 2026-11-12 (later): the window is to feel like Google Earth and look like glass with depth

His words, as given:

> When you are back online I still do not see how the UI functions are better please
> ensure you are deep diving into scroll mechanics , toggle on and off , glassy
> overlays. Think google earth feel , need to be able to export to BIM files and create
> items by spec sheet uploads IE mechanicals , furniture etc whole point of clash is
> also moving a bathroom vanity around please dive back into any open repos you feel
> may give you a better insight to polished clean look feel and function. Think of it
> more like an Apple product with windows depth. Editing software for construction that
> feels like playing a hyper realistic video game-- i want you to self review what you
> have created so far and also make sure the main md file has instructiosn to opus 5.5
> on how to use and adjust it- look at othjer gits, online interfaces etc as needed -
> infer intent

**What it overrides.** The fixed rule "the look is an instrument" (flat panels, dense
layouts, no shadow, no gradient: call 55 and VISUAL.md section A) is his rule and he
has replaced it. The screen he was shown under that rule read to him as no better. The
new standard is his: translucent panels floating over the model with real depth, the
model filling the window, navigation that feels like Google Earth, a house that looks
real. What does not change, because he did not change it and it is what makes the
program an estimating program: flag, never fabricate; speak construction; one
arithmetic for one number; no real job data; permissive licences only; every number
shows what it is made of.

**How I read each clause** (infer intent; the readings are mine and are open to him):

1. *"I still do not see how the UI functions are better."* The work since call 55 went
   into the model, the money and the words. A person opening the window sees docked
   white panels and a small model. The feel of the window is now the work.
2. *"Scroll mechanics."* The wheel over the model zooms toward what is under the
   pointer, with momentum that eases out; the wheel over a panel scrolls that panel
   and nothing else, smoothly, with a bar that shows where you are and can be dragged.
3. *"Toggle on and off."* Every layer of the job (each level, each system, the roof,
   the finishes, the drawing underneath, the clashes, each panel of the screen itself)
   has a switch that reads on or off at a glance.
4. *"Glassy overlays ... an Apple product with windows depth."* The model is the whole
   window. Panels are sheets of frosted glass over it: translucent, rounded, with a
   soft shadow and a light edge, layered so that what is nearer reads nearer.
5. *"Google Earth feel."* Drag to pan the ground under the pointer, wheel to dive at
   the pointer, tilt and turn about what you are looking at, everything with inertia;
   double-click flies to a thing; the view eases between plan and 3D and never jumps.
6. *"Export to BIM files."* The IFC4 export exists at the command line and is
   validated; it is not reachable from the window. It goes on the screen, with the
   workbook and the lenses.
7. *"Create items by spec sheet uploads, i.e. mechanicals, furniture."* A
   manufacturer's cut sheet (PDF) is brought in and becomes a thing that can be
   placed: its name, model number, width, depth and height read from the sheet, each
   value Assumed and sourced to the sheet and its page, anything not found Unknown and
   asked for. Furniture and casework join equipment and fixtures as things placed.
8. *"The whole point of clash is also moving a bathroom vanity around."* A placed
   thing is picked up and dragged in the model; what it clashes with lights as it
   moves and clears as it clears; it snaps to the wall it backs onto.
9. *"A hyper realistic video game."* Physically based materials, soft contact shadows,
   ambient occlusion, reflections in glass, anti-aliased edges, light that reads as a
   real place.
10. *"Self review."* Written cold in STATUS.md before anything is built.
11. *"The main md file has instructions to Opus 5.5 on how to use and adjust it."*
    CLAUDE.md gains a section for whoever builds next: how to run it, drive it, see
    it, and where each thing is changed.

**Open, with a working assumption** (QUESTIONS.md Q22): the older rule "no sky, sun or
grass" was not named. A hyper-real game and Google Earth both have a sky. Working
assumption: the model sits in a soft-lit place with a horizon and a sky gradient that
reads as real light, and still no grass, trees or scenery, which would be decoration
the estimate does not price.

### Call 63. What the window is rebuilt from: two surveys, and the numbers taken (2026-11-12)

Two agents surveyed, one the web and one the source on this machine. What is taken
from each, with where it comes from. A figure marked *mine* has no source and is a
starting value to tune by eye.

**How other tools move** (source read: CesiumJS `ScreenSpaceCameraController.js` and
`CameraFlightPath.js`, Apache-2.0; three.js `OrbitControls.js`, MIT;
yomotsu/camera-controls, MIT; `bevy_panorbit_camera`, MIT or Apache-2.0).

| What | Taken | From |
|---|---|---|
| wheel units | a line of wheel is 16 px, a page 100; one notch is 100 px | three.js |
| zoom per notch | distance times 0.90 (three.js uses 0.95; 0.90 *mine*) | three.js |
| zoom about the pointer | find P under the pointer (the model, else the ground); camera and target both move toward P by the same factor, so P stays under the pointer | derived; three.js `zoomToCursor` and camera-controls `dollyToCursor` do the same by other arithmetic |
| pan | the ground point under the pointer at the press stays under the pointer: ray to the ground plane at press and at each move, target moves by the difference | derived; Cesium pans the globe this way |
| turn | about the point under the pointer at the press; a full window height of drag is a full turn | three.js rotate rule; Cesium keeps the tilt centre picked at the press |
| momentum | on release, carry on and die away as exp(-t / 0.4 s) for a pan or a turn, exp(-t / 0.2 s) for a zoom; only if the press lasted under 0.4 s; stop under half a pixel | Cesium `inertiaSpin` 0.9, `inertiaTranslate` 0.9, `inertiaZoom` 0.8 with its decay(t) = exp(-(1-c) 25 t) |
| flying to a thing | 0.4 to 1.2 s by how far (*mine*; Cesium's 2 to 3 s is for a globe), eased quintic in and out | Cesium's easing for most flights |
| plan to 3D | 0.8 s, quintic in and out (*mine*) | |
| limits | never under the ground; 2 ft to 400 ft from what is looked at (*mine*) | |

Google Earth's own figures are not published; its help page documents wheel to zoom
and drag to move. Blender (GPL) was not read.

**What glass is** (source read: Microsoft's acrylic and timing pages; MDN
`backdrop-filter`; Apple's Materials page through a mirror, which gives rules and no
numbers). Both makers keep glass for what floats over content and keep dense text
opaque or thick: so the panels over the model are glass and the estimate sheet is
not. Microsoft: do not put accent-coloured text on glass. Motion: 83, 167 and 250 ms;
entering eases cubic-bezier(0, 0, 0, 1), leaving cubic-bezier(1, 0, 1, 1) (Fluent,
sourced). The rest is *mine*: blur about 20 px, tint 60 to 80% (thicker where text is
dense), a 1 px light edge, corners 18 px on a panel, 10 on a button, 6 on a small
control; shadow near (0 2 6, 8%) and far (0 8 24, 14%).

**What the engine gives** (Bevy 0.20.1, read in the cargo registry with file and line).
The 3D camera already fills the window; the panels are absolute nodes that happen to
be opaque. `BorderRadius`, `BoxShadow` (several at once), `BackgroundGradient` and
alpha all exist. There is **no backdrop blur**: a frosted panel is a custom
`UiMaterial` that samples a picture of the scene, which means rendering the scene to
a picture as well. The whole panel tree is thrown away and rebuilt on every change,
so anything that animates keeps its state outside the tree. `Interaction` is
deprecated for `Hovered`/`Pressed`. There is a transform gizmo (`bevy_gizmos::
transform_gizmo`) and mesh picking with drag events; no outline for a selected mesh.
`bevy_ui_widgets` (scroll area, slider, checkbox) is not in the lock file and would be
a new download; a switch is a few lines without it. HDR, bloom, AgX tonemapping,
ambient occlusion, contact shadows and a procedural atmosphere exist; the atmosphere
and the generated environment light need compute shaders. BIMCraft is not a Bevy
program: its camera arithmetic (`view3d.rs` `zoom_about`, `orbit_about`) and its
floating-card theme port as ideas, not code.

**Decided.** Keep this program's own camera (it has plan and walk, which the engine's
controller has not) and give it the mechanics above. Glass first as tint, edge and
shadow, which needs no new pass; real blur behind it as a second step from a small
second picture of the scene. Switches and smooth scrolling written here, no new
crate. Dragging by picking's drag events and a ray to the floor, written back
through `ed::move_by`.

### Call 64. A tool picked shows the sizes it will place, every time (2026-11-12)

The eighteenth judgment's worst findings (J18-1, J18-2) were one fault seen twice: a
person typed a size and the program placed a different one without a word. A tool
picked a second time came back in hand with its old sizes, no field open, and every
keystroke was thrown away; and Enter on a form with one field still empty moved to
that field in silence, so the next click on the wall did nothing.

Decided:

* **Picking a tool that has sizes always opens them**, filled with what it will place
  (the last sizes typed for it), the first field ready. Typing in a filled field
  replaces what is there, as it does in any program. Enter with nothing changed takes
  the tool in hand as it was: one more key for the person who wants the same again,
  and no way left to place a size that is not on screen.
* **Enter that cannot finish says why**: which field is still empty.
* **A click on the model while the sizes are open says so**: nothing was placed, and
  what to do. It is never a click that does nothing.
* **A door starts at 6' 8" high with its sill at the floor** (J18-7: a residential
  door is 6'-8", not 7 ft), shown in its fields to be typed over. Its width is typed.
  Nothing else is filled in for anyone: a window's sizes and sill, a wall's thickness,
  what a fixture is, are the person's to say.

What this changes for the scripts: a demonstration that picked a tool a second time
and expected it in hand now gets its sizes open and presses Enter (the kit's `tool`
does it).

### Call 65. A fixture or a piece of furniture is written to the BIM file as the sort IFC names (2026-11-12)

**What was there.** Every fixture went out as `IfcFlowTerminal` and every furnishing as
`IfcFurniture` with no sort given. Judge 19 (J19-39) opened the file as a designer's
program would and found a water closet indistinguishable from a floor register.

**The call.** IFC4 already has the words (checked against the schema IfcOpenShell
0.9 validates with, not from memory: the file validates with no fault):

| what the thing is drawn as | written as |
|---|---|
| water closet | `IfcSanitaryTerminal`, `TOILETPAN` |
| lavatory, washbasin | `IfcSanitaryTerminal`, `WASHHANDBASIN` |
| bathtub | `IfcSanitaryTerminal`, `BATH` |
| table, chair, bed, sofa, desk | `IfcFurniture` with that sort |
| any other fixture | `IfcFlowTerminal`, as before |
| any other furnishing | `IfcFurniture`, `NOTDEFINED`, as before |

**One rule for what a thing is.** The sort is taken from the same reading of the
thing's name that decides how it is drawn (`building::forms::form_of`), so the picture
and the file cannot disagree about what a thing is.

**What changes that a person sees.** Reading the program's own BIM file back, the
count of what it has no element for is now said in two lines where it was one: "16
fixture or outlet" and "6 plumbing fixtures" on the made-up house, where it said "22
fixture or outlet". `demos/export.sh` expected the old line and now expects these two.
Nothing is lost that was kept before: neither class is read back into the job, and
both say so.

**Not done.** Equipment is still `IfcBuildingElementProxy`/its present class: an air
handler as `IfcUnitaryEquipment`, a water heater as `IfcBoiler`/`IfcTank` needs a
reading of equipment names that does not exist yet (there are no equipment forms).

### Call 66. From judge 19's notes: how sure a clash is shows on its mark; one count; places and prompts reworded (2026-11-12)

Each of these changes something a person sees. None changes a number.

1. **A clash mark's colour says how sure it is** (J19-20: all 21 marks on the made-up
   house were one red, though 13 are findings and 8 are questions for the field; the
   release candidate's item 4 is "clashes show their certainty"). Between things that
   are Known the mark is red; where either rests on something not yet measured it is
   amber, the colour an Assumed figure already has. Under the Clashes switch, while
   the marks are on, a line says the two counts beside a spot of each colour.
   `VISUAL.md` A6 says so.
2. **One count of clashes.** The estimate window's Clashes tab counted everything it
   lists (87 on the made-up house: 21 in the same space and 66 holes to cut through
   walls and floors) while the model window's switch counted 21. Both now count what
   is in the same space. The tab still lists the holes, under their own heading with
   their own count.
3. **A place is written as the read-out under the pointer writes it**: "E 0' 0", N 28'
   0"", not "(0, 28) ft" (J19-16, J19-27). In the window only: the command line takes
   a place as `--at 13,8` and says it back the way it was typed. One function
   (`panels::places`) does it for every row the window shows.
4. **What is to verify is said thing first**: "Hot to the kitchen sink, elbow 1, what
   it is: unknown", and a group as "227 for the same reason. The first: ..." (J19-19:
   "What it is of hot to the kitchen sink, elbow 1: unknown").
5. **Enter is described one way**: "Enter, then click on the plan." (J19-44: one
   line said "Enter takes the tool in hand", another "Enter to draw".)
6. **What is written out is named in a sentence as it is said**: "Can't write out the
   BIM model (IFC): ...", "the workbook for suppliers" (J19-40: the button's name
   was put in lower case). A file written from a job with no lines says it has its
   headings and nothing under them.
7. **The wheel stops going out** where the building is about a sixth of the size it
   is framed at (J19-15: it shrank to nothing).

**Not changed, and why.** J19-14 (on plan with every level on show the roof is drawn
solid over the house): that view is the roof plan, the drawing the roof is taken off
from, and its lines are coloured by type there (VISUAL.md E2); the roof is its subject,
not something overhead. Pick a level and the roof over it is broken lines; switch Roof
off and the walls of every level show. J19-9 (a drag slid the house less than the
hand moved): the place grabbed is on the ground, and the house stands further off than
the ground under the pointer, so it moves less across the screen, as it does in Google
Earth; `demos/glassw.sh` measures the grabbed place itself and it stays under the
pointer.

### Call 67. After judge 19's report: nothing is lost or left out without a word, and what the first pictures showed (2026-11-12)

The judge's two "no"s were the look and silence. These are the calls made on silence,
and on what the first pictures taken after the report showed. Each changes something a
person sees; none changes a figure.

1. **Save is never put off.** While a field had the keyboard, F2, Ctrl+S and the Save
   button were dropped without a word (J19-46, 47); since call 64 a tool picked always
   has its sizes open, so that was the ordinary state to press Save in. Save now saves
   the job as it stands and the line says "What is being typed is not in it yet: Enter
   sets it." Ctrl with a letter is a command, never a letter of the value. The line
   names the file, not the folders it lies in.
2. **A window with changes not saved does not close at the first asking.** It says
   "Not closed: the job has changes that are not saved. Save (Ctrl+S), or close again
   to leave without saving them." The second asking closes it. (Not asked for by the
   judge; found reading the same code. A question in a box was the other way; the
   program has no boxes that stop the work, and the line is where everything else is
   said.)
3. **What is built and not priced is said where the price is read** (J19-48): under
   Price on the recap, in the sentence the command line already used, and as a red
   "3 not priced" beside the price in the model window's bar. `view::not_priced_said`
   is the one wording. The price itself is unchanged: it is still the price of what is
   priced, which is what it says it is.
4. **A double-click flies only to a place where the job is** (within four times the
   framed distance of the job's middle); on the sky or out at the horizon it does
   nothing (J19-50: it flew to an empty view). **The wheel with nothing near under
   the pointer** goes the way the pointer points, over the ground (J19-49).
5. **The message over the model is a card of the same frosted glass as the panels**,
   round, with a deeper shadow, a bar of the tone's colour down its left edge, under
   the view buttons. (The first picture after the report showed the old flat red bar
   cut off where it ran under those buttons; VISUAL.md A2 named it as a known fail.)
6. **The right panel is as tall as what is in it** (VISUAL.md A5), up to the height of
   the window, as the left one already was. The wheel scrolls a panel only when the
   pointer is on the panel itself.
7. **The Show switches come straight after the levels** in the left panel (J19-2: on
   the whole house they opened below the fold); what is brought in and written out is
   below them. The program's list of what is on screen now says which way a
   scrolled-away control of the left panel lies, and the demonstrations' kit turns
   the wheel that way before clicking, as a person does.

### Call 68. The house is lit by a sun and by its sky (2026-11-12)

**What was there.** No tone mapping, two lamps, a flat light from everywhere and a
painted sky that lit nothing. Judge 19: "a tidy massing model, not a game-grade one."

**The call.** The plan in STATUS.md ("The light and the materials"), step by step.
What is built so far, and the figures chosen by eye on the made-up whole house:

1. **The camera takes in a range of light and folds it to the screen** (`Hdr`, the
   engine's default fold, "TonyMcMapface", chosen over AgX because it leaves a
   mid-tone colour closer to the colour it was given; dither on). **On plan nothing
   is folded**: a plan is a drawing, and its paper and inks are the colours they are
   given, so the fold is switched with the view.
2. **The sky lights the house** (`crates/app/src/skylight.rs`): the two pictures the
   engine lights a scene from are worked out at start from the sky's own three
   colours (horizon, overhead, ground). They are worked out in plain arithmetic, not
   on the graphics card: the engine's own filter needs a compute pass every frame;
   this is the same on every machine, costs nothing per frame, and can be tested
   (five tests, the figures by hand). **What falls on a surface has most of the
   sky's blue taken out** (it keeps 0.35 of its colour): with all of it the ground
   and every white wall were lit blue, which was seen in the first picture and is
   not how a house under a real sky looks, where haze and cloud fill as much as blue
   does. **What is reflected is the sky as drawn.**
3. **One sun**, a little warm (`SUN_COLOUR`), 3,600 lux against the sky's 1,150 cd/m2
   in the engine's measures at its standing exposure. (The sky was first set at 800:
   a white wall out of the sun then read as a mid grey, (139, 141, 142) where the
   colour checks wanted "pale", and cedar siding in shade (121, 93, 68). At 1,150 the
   shaded siding is (135, 102, 73) against (172, 131, 93) in the sun: plainly two
   walls, and neither dim.) The second lamp and the flat
   light are gone.
4. **The small picture behind the frosted panels** is lit and folded the same way,
   so a panel's blur matches what is round it.

**Not built yet of the plan:** the sun's shadow softened with distance (the engine's
feature for it is marked experimental and is costly; the shadow's edge as it is
reads clean at the window's size, so it waits), a look at each material's roughness
under the new light, and bloom.

**What changes that a person sees:** every colour in 3D. Nothing on plan, nothing in
the estimate window, no figure.

### Call 69. Patrick, 2026-11-12 (night): one snapshot of each of seven trades

**His words:** "I want to see one snapshot of each of the following trades -
foundation , framing , electrical , plumbing , HVAC , interior wood work and
waterproofing". Then: "Please confirm you got my last message".

**What was done.** Confirmed. The six demonstrations then running were stopped (they
are his program's checks, not his request). One snapshot per trade was taken on the
made-up whole house, both windows side by side, on the program as it is
(`demos/captured/trades/`, seven pictures), and sent with what they do and do not
show said plainly.

**What they show, which is the finding.** Three of the seven trades are in the
model: electrical, plumbing and HVAC are drawn in 3D, each with a switch, and
taken off on a tab of their own. **Four are not drawn at all:** foundation, framing,
interior woodwork and waterproofing are lines on the estimate (footings and
foundation walls as one subcontract figure; wall framing 2x6 at 16 in by the SF of
wall; casing, base and cabinets; sheathing and weather barrier, flashing and
sealant) with nothing in the model behind them. For those four the snapshot is the
estimate's lines beside a picture of the finished house, and that was said.

**How this is read** (QUESTIONS.md Q23 has it as a question with the working
assumption): he asked to *see* each trade, in a program whose mission is the whole
house, every trade, in 3D. A trade that is only a line of text is not seen. The
working assumption is that each of the four is to be drawn in the model with a
switch of its own, from what the model already holds (footings and stem walls under
the exterior walls; studs, plates and headers in each wall from its framing line;
base along each room and casing at each opening; the weather barrier as the layer
under the siding), and that the quantities on those estimate lines come from what is
drawn. It is logged in PROPOSALS.md because product scope is his.

### Call 70. A strip footing is a kind of its own, held as a wall is (2026-11-12)

**Why now.** Q23's working assumption, step 1: the foundation is to be drawn.

**The call.** A new kind, **Footing**: a strip footing along a line on plan. It is
ported from BIMCraft's `strip_footing` (PORTMAP.md has the row), which makes one for
each segment of its points. It is held the way a wall is held (the same line, a type,
and one length) as a door and a window are both held as an opening, because that is
what it is on the foundation plan: a line with a width and a depth. What that gives
without new machinery: it reads and saves, it is drawn, it can be moved, measured and
removed, and its type is a type like any other.

**What is a footing's and never a wall's.**

* Its words: "footing line", "depth", "width". Never "wall line", "height",
  "thickness", "bounds rooms".
* Its quantities: length (LF), width, depth, and **volume in CY** (length x width x
  depth / 27), which is what it is priced by. At a corner two footings that end at the
  corner point overlap in a square and leave the same square open outside it, so the
  sum is the volume of the ring exactly.
* Where it lies: its top is at its level's floor and it goes down by its depth. A
  wall on the same line on the same level stands on it. Its place is worked from its
  depth each time, so a depth measured later moves its underside.
* It is never counted as a wall: a level's wall count, length and face do not change
  when footings are drawn on it. The takeoff lists a level's footings and their
  concrete on two lines of their own, only where the level has any.
* In the BIM file it is `IfcFooting`, `STRIP_FOOTING`.

**A foundation wall is a wall**, on a level of its own under the first floor, of a
type named for what it is. No new kind.

**Changed behaviour, named.** The list of what clashes never test now ends "; a
footing is tested against neither, nor against another footing" (walls, floors and
footings meet by design). One footing drawn over another is refused as one wall over
another is, in a footing's words. Nothing else changes for a job with no footing.

**Not yet:** a footing's type says "thickness" where it should say "width" when the
type itself is listed (a type does not know what it is used for); isolated (pad)
footings; a footing stepped down a slope; the concrete of a foundation wall as a
quantity of its own (today it is a line on the wall worked from its face).

### Call 71. A wall whose framing is priced can be drawn as its frame (2026-11-12)

**Why now.** Q23's working assumption, step 2: framing is to be drawn.

**The call.** A wall that has a framing line priced on it ("Wall framing, 2x6 at 16
in") can be drawn as its frame, with a switch of its own under Show (**Framing**,
offered where any wall has such a line, with how many do). Dressed in what is priced,
bare where nothing is, as the finishes are (call 58): a wall with no framing line is
not framed on a guess, and is drawn as it was.

**The layout is a port.** `building::framing::frame` is BIMCraft's timber layout
(`crates/core/src/timber.rs`): a bottom plate and two top plates, a stud at each end,
kings and jacks beside each opening, a header and a sill, studs on stations at
multiples of the spacing from the wall's first end, and cripples on the same stations
above a header and below a sill. **I first wrote in the module that BIMCraft had no
stud framing.** That was a claim of "missing" made on a list of file names cut short,
which is the defect this repository's own rules name; the port map's gate then caught
a second wrong search claim on the same change. Both were corrected before anything
was committed, and the module is headed and credited as the port it is. What is
changed from BIMCraft's: any number of openings, and doors; a station stud that a king
or a jack would cut into is left out, not clipped; the header's depth is taken from
its span.

**What the lumber is.** Its depth through the wall is the dressed size of what the
line names (a 2x6 is 5 1/2 in: the American Softwood Lumber Standard's sizes), and
its spacing is what the line says. Words that name no lumber frame nothing.

**Two rules of thumb, said where they are used:** a header's depth by its span (2x6
to 4 ft, 2x8 to 6 ft, 2x10 to 8 ft, 2x12 beyond), where the code sizes it from the
load; and the bottom plate laid whole under a door.

**What it is not yet.** A picture of the frame, and a count of it that nothing uses:
the estimate still prices framing by the SF of wall, as its line says. Stud counts
and plate lengths are not offered to a cost line; when they are they will be Assumed,
sourced to this layout and its two rules of thumb. No joists, no rafters, no corner
or partition backing, no blocking. On plan a wall is still a wall.

### Call 72. The finish can be taken off the walls, and what keeps the water out is seen (2026-11-12)

**Why now.** Q23's working assumption, step 3: waterproofing is to be drawn.

**The call.** A switch under Show, **Finishes** (offered where any wall has a finish
or a weather barrier priced on it). On, the house is as it was: dressed in what is
priced. Off, the cladding, the wainscot and the casing round each opening are taken
off, and what is under them is drawn where it is priced and nowhere else:

* a wall with a weather barrier priced on it ("Sheathing and weather barrier",
  "house wrap", "building paper", "WRB") is drawn in its barrier, a cool white
  sheet; a wall with none is bare, as any wall with nothing priced is;
* an opening with flashing priced on it ("Flashing and sealant") is drawn with its
  flashing: a strip 4 in wide at its head, down each side and under its sill, on
  the outside face, as it goes on before the siding.

It is the same rule as the finishes (call 58) and the framing (call 71): the model is
dressed in what the estimate prices, layer by layer, and each layer has its switch.
Finishes off and Framing on are two different layers of the same wall.

**The 4 in strip is a drawing size, not a quantity.** Nothing is counted from it; the
estimate prices flashing by the opening, as its line says.

**Not yet:** roof underlayment and ice barrier under the roofing; dampproofing on the
foundation wall; flashing at roof lines (step, counter, valley), which the roof's
lines could carry; the barrier's laps and tape.

### Call 73. The base in a room is drawn where it is priced (2026-11-12)

**Why now.** Q23's working assumption, step 4: interior woodwork is to be drawn. It
was the last of Patrick's seven trades with nothing in the model.

**The call.** A room with base priced on it ("Base and trim", "baseboard", "wood
base"; never a base cabinet, a gravel base or a base coat) is drawn with its base: a
board 5 1/4 in high and 5/8 in thick at the foot of its walls, along the room's own
outline (the faces of its walls, the outline its floor area is the area of), left out
across each door in a wall that runs along it. It is finish: it goes off with
Finishes and with its level. It is drawn as wood so that it reads against a wall;
what it is made of is not read from the line yet.

**The board's size is a drawing size.** The estimate prices base by the LF of the
room's perimeter, as its line says, and nothing is counted from what is drawn. A
door's width is not yet taken out of that LF either: the drawing leaves the base out
at a door and the line still prices the whole perimeter, which is how base is
usually bought (the offcut is waste) and is said here so that nobody takes the two
to disagree by accident.

**Not yet:** casing round interior doors and the inside of windows; the kitchen's
cabinets, which are one priced figure on the room with no layout to draw (a cabinet
is drawn where one is placed as a thing: call 62's forms); stairs, shelving, crown.

### Call 74. The made-up whole house has its foundation drawn, and priced by what is drawn (2026-11-12)

**What was there.** In the priced whole house (`plumbline sixteen`) the foundation was
one line on the job, "Footings and foundation walls, 1 at $24,600", with nothing in
the model behind it. It was the snapshot Patrick was sent for "foundation".

**The call.** That line is gone. The house has a level "foundation" 4 ft under the
first floor (source "sample plan S-101"), and under each of its four outside walls a
strip footing 24 in wide and 12 in deep and an 8 in concrete wall 4 ft high. Each
footing carries "Concrete footing, in place" by its volume at a made-up $450 the CY;
each wall "Foundation wall, 8 in concrete, formed and placed" by its face at a
made-up $38 the SF. All in Division 3.

**Every figure this changes, by hand.**

* Footings: the two of 40 ft are 2.9630 CY each, $1,333.35; the two of 28 ft are
  2.0741 CY each, $933.35 (2.0741 x 450 = 933.345). Four: **$4,533.40**.
* Foundation walls: 544 SF at $38: **$20,672.00**.
* Division 3, Concrete: the slab's $7,280.00 with those: **$32,485.40** in 9 lines,
  where it was $31,880.00 in 2 lines.
* Job cost: up by $605.40 to **$394,677.48** in 399 lines (was $394,072.08 in 392).
* Price: at 10 % and 8 %, up by 605.40 x 1.188 = $719.22 to **$468,876.85** (was
  $468,157.63).

The plain house (`plumbline house`, no prices) is not changed: it still has two
levels, and every demonstration that counts its walls, levels and model classes
stands as it was. No test and no demonstration held any of the old figures.

### Call 75. Patrick, 2026-11-13: each of the sixteen divisions will have its own AI harness

**His words:** "I will be running this locally eventually and each of the 16 trade
divisions will have its own AI harness the intent here is all trades are able to have
their own knowledge base, pricing and tweaking."

**How it is read.** Three statements, each with a consequence for this program.

1. **"Running this locally eventually."** The program already sends nothing anywhere
   (`tools/gate-egress.sh`), and stays that way. What a harness beside it may send is
   not the program's to decide and is asked in QUESTIONS.md Q24.
2. **"Each of the 16 trade divisions will have its own AI harness."** A division has to
   be something that can be handed to one worker whole and taken back from it,
   without that worker reading or writing the other fifteen. Today it is not: a
   division is a number on a cost line. The assemblies and the prices are two tables
   for the whole job, and nothing is kept by division but that number.
3. **"Their own knowledge base, pricing and tweaking."** Per division: what it knows
   (its assemblies, its rules of thumb and where each comes from, its units), what
   things cost (its prices, its labor), and what its owner has adjusted (waste,
   productivity, a formula on a line). Each of the three has to be readable and
   writable from outside the program, by division.

**What this does not change.** The fixed rules. Above all two of them: *real job,
client or supplier data never enters the repository* (a division's real prices and its
real knowledge live on the estimator's machine, never here; what is in the repository
is made up), and *flag, never fabricate* (what a harness writes arrives with who wrote
it as its source and is never Known on a harness's say-so: a person or a field
measurement makes a value Known).

**What is not decided here.** How a harness is built, what model it runs on, and
where. Those are his. What is proposed is the seam on this program's side
(PROPOSALS.md 21), so that when the harnesses come there is one way in and one way
out for each division and nothing has to be torn up.

### Call 76. Patrick, 2026-11-13: the two rulings on harnesses stand, and the repository is to be left clean for the next run

**His words:** "I agree with your rulings we are running low on limits please ensure
everything is cleaned up as when limits reset will make a copy of repo to local
machine for next fable run to have previous research we've done for each division".

**What is now ruled** (QUESTIONS.md Q24, answered as its working assumptions):

1. A harness is a thing beside the program. The program itself sends nothing anywhere
   and never opens a connection; where a harness's model runs is a rule for the
   harnesses. The seam gives a harness its own division and nothing else.
2. Nothing a harness writes is Known. It arrives Assumed, with the harness named as
   its source, and becomes Known only when a person or a field measurement says so.
3. A harness prices and tunes. It does not draw.

**What was done for the copy.** What sat outside the repository and would not have
travelled with it is now in it: the two surveys and both judges' briefs and notes
(`research/`), and an index of what the program has for each of the sixteen divisions
(`divisions/README.md`). The twentieth judgment was stopped part way to save limits;
its nine findings are recorded and none is fixed. STATUS.md ends with one section,
"Start here", for whoever opens the copy.

### Call 77. Walls whose heights are equally sure are said in one sentence (2026-11-13)

A wording call, made on reading the first job's last picture as a person would. A
room's figures rest on whether each wall round it bounds it, which rests on that
wall's heights; the reason was written once for each wall, so a room of four traced
walls read "whether wall 1 bounds this room rests on its heights, which are assumed;
whether wall 2 ..." four times over, three figures deep: a block of grey text nobody
reads. Walls whose heights are equally sure are now named in one sentence: "whether
wall 1 (wall-0005), wall 2 (wall-0006), wall 3 (wall-0007) and wall 4 (wall-0008)
bound this room rests on their heights, which are assumed". One wall alone is said
exactly as before. Nothing is left out: every wall is still named, with how sure.

### Call 78. Where a run's height is taken to, and what the sixteen leave out, said plainly (2026-11-13)

Two wording calls, made on reading the field-measurement pictures as a person would.

1. A duct's or a pipe's row read "Its heights are those of its: bottom", with the note
   "which line of its section the heights were given to". Nobody on a site says that.
   A height on a mechanical drawing is to the bottom of the duct, its centerline or its
   top, and it is asked as "is that to the bottom?". The row now reads "Height is to
   its: bottom", with the note "the bottom, the center or the top of its section:
   where its heights are taken to". The refusal that named it reads the same way ("its
   size is not known and its height is to its top or its bottom, so its center line
   cannot be placed"). The value, what it does and what is saved are unchanged; the
   two field demonstrations expect the new words.
2. The estimate's "By division" head read "0 of 16 with work in them" over sixteen
   rows of "none", under a recap with a job cost in it. The row that squares the two
   ("In no division") is the seventeenth and was below the bottom of the window. The
   head now says it: "0 of 16 with work in them; 1 line of the job cost is in no
   division". With nothing in no division the head reads as before.

### Call 79. A detail of one line that cannot be extended says "its one line" (2026-11-13)

A wording call, made on reading the Budget tab as a person would. An item whose
detail is a single line with no price read "the detail is not whole: 1 of its 1 lines
cannot be extended". It now reads "its one line cannot be extended"; with more than
one line it reads as before ("2 of its 8 lines cannot be extended"). `demos/d4.sh`
expects the new words. On the same tab, what is not priced at all is counted as the
Estimate tab counts it ("5 eaves, 4 hips", where it read "5 eave, 4 hip"). No figure
changes.

### Call 80. A drawing's scale is always said as a scale (2026-11-13)

A wording call, made on reading the first job's last picture. Under the drawing's
name the panel read "sheet 77' 8" x 60' 4"": the scale had been set from a dimension
clicked a hair off 1/8" = 1'-0", so it was not one of the architect's scales, and
the program fell back to the size of the sheet on the ground, which nobody reads as
a scale. A PDF sheet whose scale is not one of the architect's is now said the way
an engineer's scale is written, an inch of paper to so many feet: `1" = 8' 0 1/4"`.
A picture (PNG, JPEG) has no paper size, so it still says what its sheet covers, in
those words: "sheet covers 77' 8" x 60' 4"". The architect's scales read as before.
Nothing saved changes.

### Call 81. On plan a room carries its name and its floor area (2026-11-13)

Read in the plan pictures as a person would: two rooms had been named and the plan
showed two empty rectangles. A drawing writes each room's name in it. On plan, each
room on a level that is on show now carries its name and, under how sure it is, its
floor area to the SF, written where the room was named (a point that is always
inside it, which the middle of an L-shaped room is not). It is the model's own
figure, the one the Building tab shows; nothing is worked out in the window. The
names give way to the roof plan: with the roof on show on plan it is the roof's
lines and planes that are read. This is within the fixed rule that a plan reads
like a drawing, so it is built and not put to PROPOSALS.md. A door's swing on plan
is not built.

### Call 82. Hours on a labor line with no price are said beside the recap's hours (2026-11-16)

From the code review of the model (M21-3). The recap's hours for a cost type are the
hours of the lines in its dollars, so that the two can be read against each other;
that stays. But a labor line with its hours taken off and no price yet was in neither,
and nothing said so: two lines of 18 hr, one unpriced, read "18 hr" with no mark, and
a PM would man the job at half its hours. The hours of such lines, and how many
lines, are now held beside each cost type's total and said wherever the hours are
shown: "18.00 hr more on 1 line with no price". They are never added into the hours
beside the dollars. A cost type with no line in its dollars shows no hours of its own
(they read "0 hr, Known" inside the model before, though nothing showed it).

### Call 83. Dollars ride on a measurement only where the quantity is in question (2026-11-16)

From the code review of the model (M21-2). The verify-in-field list counted a line
against every unverified value of the thing it is priced on whenever the line's
dollars were not Known. A line of a Known length at an Assumed price was so listed
against the wall's Assumed height: "$200.00, measure: height", for a base trim line
that does not use the height and that no tape makes surer. The list is sorted by
those dollars, so it sent the crew to measure what moves no money. A line now rides
on a measurement only if its own quantity is not Known. (The rule of call 59 stands:
where what a thing is, its type or its size, is itself to be checked, everything
priced on it rides on that check.) A line whose only doubt is its price is still
Assumed on the estimate and still on the sheet's To verify; it is not a field
question. Figures that change because of this are named in STATUS.md.

### Call 84. A coverage on a price list buys whole ones by the same rule as one typed (2026-11-16)

From the code review of the model (M21-6). Call 21 says a package is bought whole:
"32 SF/sheet" typed on a material buys 4 sheets for 100 SF. The same words on a
supplier's price list bought 3.125 sheets, because the list was read as "whole ones
only" only where a column of that name said yes, and a supplier's list has no such
column. $37.50 where the keyboard gave $48.00, with no word. A list is now read by
the typed rule: where its "Whole ones only" column says yes or no, that is taken;
where there is no such column, or the cell is blank, a factor that is so much to the
unit the material is bought by (SF to the sheet) buys whole ones, and one that is so
much of it to something else (lb to the SF) does not. What was taken is said in what
the import reports: ", whole ones only" or ", to the exact amount". The program's own
price list writes yes or no in every row, so it reads back as it was written.

### Call 85. One value entered on a line, standing alone, is what was entered (2026-11-16)

From the code review of the model (M21-4). A line in EA is rounded up to a whole
number where its quantity is worked out: 26.67 fasteners is 27, and 5 anchors at one
to every 2 is 3 (the eleventh judgment, J11-3). A number typed as the whole quantity
is what was typed: 2.5 months of a dumpster. But the same 2.5 entered as a value on
the line and named as the quantity ("months", with months = 2.5) was rounded up to 3
with the note "a count is whole": $1,950.00 against $1,625.00 by how it was entered.
One value entered on the line, standing alone as the quantity, is now what was
entered, as a typed number is. Anything worked out in EA is still whole (so "2.5 * 1"
is 3: it is arithmetic, and says what it was rounded from). (A first form of this
call made whole only what was worked from the takeoff; it made 2.5 anchors of
"n / 2", which J11-3's test caught, and was withdrawn before it was committed.)

### Call 86. The first wall of a new job: what is offered is real, an example says it is one, and a number typed is never a tool (2026-11-16)

From the twenty-first judgment (J21-12, J21-13), which walked the path a stranger
walks: a new job, a level, Wall. Four things met him in his first five seconds, and
each is changed:

1. **An example looked like a value.** Each size field showed in grey what a value
   looks like ("9 ft"), and Enter did not take it. An example now says it is one:
   "like 9 ft". And where the program has a real figure to offer it offers it as a
   value to accept: a wall's height is offered as the floor-to-floor of the level it
   is drawn on, which the person typed a moment before. It is still theirs to type
   over, and what is set is recorded as set in the window, as before.
2. **The refusal named the wrong field.** A click with sizes still open said "Still to
   be typed: Height" and left out the empty field the cursor stood in. It names every
   field still empty.
3. **A number typed was taken for a tool.** The number keys take a tool in hand
   (1 Wall, 2 Door). With a wall in hand and no start clicked, a length typed began
   with a digit, the digit took another tool in hand, and the rest of the length went
   into that tool's sizes: a wall type of "2 ft" and a door 0' 6" wide that nobody
   typed. With a tool in hand a number key is now never a tool: it is the start of a
   length where a line has its start, and otherwise it is answered with what to do
   ("Wall is in hand: click where it starts first..."). To change tools, click the
   tool, or Esc and then its number.
4. **A click or an Esc that did nothing was not said.** A click on the plan with a
   length typed and not yet entered now says the length is not set; Esc on a typed
   length says the start is kept.

### Call 87. A wall type and a footing type are what they were made as (2026-11-16)

From the twenty-first judgment (J21-2, J21-5, J21-14) and the model's review (M21-10).
A strip footing is held as a wall is, on a type that gives its width. Which of the two
a type was for was read from what was first drawn on it: a type made with `type
footing` and not yet used was listed as a "wall thickness", a wall could be drawn on
it (a 36 in wall), and a type made with `type wall` took a footing and from then on
refused walls. A type now holds what it was made as (`footing` on its wall type in
the job file, left out for a wall's type, so files written before read as they did):
a footing is drawn only on a footing's type and a wall only on a wall's, from the
first, and each refusal says how to make the other kind and lists the types of the
right kind the job has. A type from a file written before this, already under
footings, is still a footing's type. Removing a type that is in use is refused in
words that are true: whose type it is.

### Call 88. The command line says a drawing's scale as the window does (2026-11-16)

From the twenty-first judgment (J21-16). Call 80 made the window say a scale as a
scale; the command line still said "30 ft is 540 on its page: 18 to the foot" (18
what? points). The scale's words are now worked once, in the model
(`Drawing::scale_said`), and both say them: "30 ft is 540 on its page: 1/4" = 1'-0"",
and for a scale that is not one of the architect's, `1" = 4' 1 5/8"` (to the eighth
of an inch; the window showed it to the quarter). `demos/plan.sh` expects the new
words.

### Call 89. Words and figures on the estimate window, from the look judge (2026-11-16)

From the twenty-second judgment, part B. Wording is the builder's to decide.

- **A three-way size is in inches while every side is under 8 ft**: `16 × 16 × 8 in`,
  `24 × 30 × 60 in`, to the sixteenth; feet and inches when a side is 8 ft or more.
  A sheet-metal man, a plumber and a maker's sheet write a fitting, a unit and a
  fixture in inches; `1' 4" × 1' 4" × 0' 8"` is how nobody writes a duct elbow. One
  place works it (`units::size_said`). A position, and a length along the building,
  stay in feet and inches as a tape reads.
- **A worked-out rate is shown to the cent in the window** ("$33.31 / SF"). The
  figure is not changed: the command line and the workbook keep four places, which
  is what the detail divided by the area comes to and what reproduces it.
- **The Budget tab's first column is "What"**, as Verify in field's is. ("Element"
  stays the workbook's column heading: it is the word of ASTM E1557, "Standard
  Classification for Building Elements and Related Sitework-UNIFORMAT II" (the title
  as ASTM's store lists it, read 2026-11-16), and a workbook is read beside one.)
- **The Roof tab's last column is "Roofing"** and says what is priced on the plane
  where the plane carries no condition of its own.
- **Several open items for one reason** read "the first of them, and N more for the
  same reason".
- **A finding is marked red in the Clashes list**, as on the model. A value that is
  Known still carries no mark anywhere else: there a mark means "look at this", and a
  finding is the one Known thing a person must look at.

### Call 90. A cost line is added in the window, and in-house work says how fast it goes (2026-11-16)

From the twenty-second judgment, part A (J22A-13): a veteran estimator looked for a
way to put labor on a wall and found none. The window could switch a labor line off
and could not make one.

- **"Add cost line"** is on the panel of anything that is built, beside Unit cost. It
  asks three things in an estimator's order: what it is, what it is priced by, its
  cost type. What it is priced by is one of the thing's own takeoff figures, typed
  as it reads on the panel ("net area") or clicked from the list of them under the
  field with what each comes to; a formula on them is taken as the sheet takes one.
- **In-house work is by the hour, so its cost type carries how fast it goes**: "labor
  at 40 SF/hr" (or "0.025 hr/SF"). A fourth field for the rate was tried in the
  head and dropped: it would stand empty on every material and subcontract line,
  and Enter goes to the first empty field. Material, subcontract and general take
  no rate and say so if given one.
- **All of it or none.** The model's verb for it (`Job::add_line_in_hours`) adds the
  line, gives it its rate and then its cost type, and puts the thing back as it was
  if any of the three is refused. The rate is Assumed, set in the window.
- The line has no price and says where one is typed. It has no division until it is
  given one, and the estimate says so, as for any line.

Not decided here: what labor lines the starter assemblies should carry. That is the
library's content, moves three demonstrations' figures, and is the next thing owed
on this finding.

### Call 91. A roof drawn from its footprint is measured on site, at the roof or at any plane of it (2026-11-16)

From the twenty-second judgment, part A (J22A-18, one of the two deciding fails of
item 12): on a traced house the roof carried the most dollars and was not on the
verify-in-field list, and nothing on it could be made Known on site. The list asks of
a thing only what a measurement can be put on, and a roof solved from its footprint
took none.

- **This replaces the J16-2 ruling of call 49** ("a plane solved from its roof's
  footprint does not take a measured eave height ... refused, and not offered"). The
  reason for it stands: what is put on a solved plane is thrown away the next time the
  roof is solved. So the measurement is not put on the plane. **It is put on the roof,
  and the roof is solved again**, and every plane of it has it. That is true to the
  building: a roof drawn this way has one eave height.
- **What a roof takes.** Its eave height: one length, shot. Its footprint, and how
  each edge rises (a pitch or a gable): looked at and found as drawn ("same"). A
  footprint that is not as drawn is moved first, as a wall line is.
- **What a plane of it takes.** Its eave height, which is the roof's, and its pitch,
  which is how its edge of the footprint rises. Both are said of the plane, where the
  person is standing, and then whose they are. Its outline and its eave line are the
  roof's footprint solved, and are not asked of at the plane.
- **On the list.** What is priced on a plane rests on its roof as well: the roof is a
  question with those dollars riding, until its footprint, its eave height and every
  edge's rise are Known. One pitch still Assumed leaves every plane's outline Assumed
  (a hip moves when the pitch next to it does), and the list says which edge.
- **Anything else priced on values in question that takes no measurement is listed
  all the same**, with what on it is to be checked, and its dollars ride. Left off,
  nobody is sent to it. (Nothing in the demonstrations is such a thing today.)
- **Words** (J22A-16, J22A-17, J22A-19; wording is the builder's call). A roof's
  eaves, rakes, hips, valleys and ridges are never called "lines": on the estimate a
  line is a cost line. The Roof tab's table is headed by what is in it ("Eaves, hips
  and ridges"), and a change to a roof says "its eaves, hips, valleys and ridges
  follow". Deleting something says what was priced on it and what that takes out of
  the job cost; the command line's `remove` says the same sentence. A percentage out
  of range is refused in a whole sentence ("has to be a percentage from 0 to 1000").
  Overhead or profit over 100 % is taken, and answered as a warning with the tenth of
  it offered ("If 25.00 % was meant, type it again"): it is a slip far more often
  than a markup, and the program does not know which.
- **A roof drawn from its footprint is deleted with everything solved from it**: its
  planes and their eaves, hips, valleys and ridges go with it, and what was priced on
  them is said. None of those can be taken out alone (the roof would make them again),
  so the roof could not be deleted at all. A thing that still has other things under
  it is refused as "has 5 things under it: delete those first" (it said "elements").
- **On the estimate sheet a division's name and its total have a row of their own**,
  and the column names are on the row under it. Beside the column names the two were
  cut short ("THERMAL AND MOISTURE PROT", the total gone).

### Call 92. The starter assemblies carry labor, with no production rate and no wage in them (2026-11-16)

The last open part of J22A-13 (a deciding fail of item 10): a veteran estimator
priced a wall from the starter library and found material lines only. Call 90 let a
cost line be added by hand; an assembly is what an estimator reaches for first, and
his assembly has the labor in it.

- **Eighteen labor lines**, on the assemblies whose work a builder's own crew
  usually does: framing and sheathing a wall (all four outside walls), hanging lap
  siding (the two walls that have it), framing a partition, framing a floor and
  laying its subfloor, base and trim in the four rooms that have it, sheathing a
  roof (both roofs), laying underlayment and shingles, setting a window, hanging a
  door (both). What is bought installed (brick, stucco, stone, standing seam, the
  slab's placing, gypsum board hung and finished, paint, every run and fixture)
  stays a material or a subcontract line as it was: which of those a builder does
  in-house is his to change with Add cost line.
- **A labor line in the library has no production rate and no wage**, as its
  material lines have no price (call 60: "it has no price in it, and never will").
  How fast a crew frames a wall is the estimator's own figure as much as what he
  pays for a stud, and a round number shipped in a library would be priced by
  somebody as if it were his. The line says "no production rate" in the colour of
  something not known, comes to no hours and no dollars, and is counted among the
  lines with no price.
- **The rate is typed on the line, on the estimate sheet**, beside its division, as
  a stretch-out is: "40 SF/hr" or "0.5 hr/EA". Until now a rate could be given only
  when a line was added by hand, and never changed in the window.
- The library is 30 assemblies and 111 lines. The file is what the program writes
  out from a job the library was brought into, to the byte, as before.

Rejected: a labor line under every material line. It doubles every assembly, and
most of a wall's finish trades are subcontracted on a house. Rejected: typical
production rates marked Assumed. An Assumed figure is priced; an Unknown one is not,
and says so.

### Call 93. The bars are cards with the model between them, and a panel has a shadow on plan (2026-11-16)

From the twenty-second judgment, part B (J22B-3, J22B-38): the window failed A1 and A2
of VISUAL.md. The top bar and the status line ran the whole width of the window, so
the model read as a view docked under a bar; and on plan no panel had a shadow.

- **Along the top there are two cards**, each as wide as what is on it: the job and
  Save, Undo, Redo at the left; the price and what is not yet sure at the right. The
  model is seen between them. **Along the bottom, two more**: what the last action
  did, as long as its words; where the pointer is and the view, at the right. No
  control was renamed or moved out of its card.
- **On plan the paper is white as given, and not lit.** It was a lit surface, brighter
  than the screen can show and folded back to white; a shadow laid over it was folded
  back to the same white (measured: 255 beside every panel's edge, where 3D showed a
  shadow). Unlit, it is the same white to look at and takes a shadow. Floors on plan
  are still lit and still fold to white: a panel over a floor has less shadow than one
  over the paper beside it.
- **The shadow under a floating panel is a little deeper** (near 0.10 to 0.16, far
  0.16 to 0.24): in 3D it darkened the ground under a panel's edge by about 4 %, and
  the panels read as flat grey cards.

Not done: the left and right panels still stand nearly the height of the window
(the judge's "weaker" half of A1). The pointer over the gap between two cards is
still taken as over a bar.

### Call 94. Picking a plan: a file, a folder and a page are a builder's words; the page is picked, not typed (2026-11-16)

From the twenty-second judgment, part B (J22B-35, the deciding fail of item 9): the
Add drawing list "is a file picker in software words". Wording is the builder of
this program's call (Patrick's rule), so it is decided here.

- **"File" and "folder" stay.** They are Patrick's own words in his rule ("a file
  is picked from a list; nobody types where a file lies"), and they are what a set
  of plans is to anyone who has been sent one: a PDF file, in the job's folder. What
  a builder does not say is "directory", "path", "browse" or "import", and none of
  those is on the screen.
- **A PDF's page is a page, not a sheet.** A sheet has its own number on its title
  block (A-201); "sheet 4" for the fourth page of a file would be taken for that
  number. A drawing is still named for its file and its page ("usda-7213, page 4")
  until it is given its sheet number by hand.
- **The page is picked from a list.** Picking a plan of more than one page lists its
  pages under the field, each a row; a click adds that page. It was typed after a
  comma in the field ("change the 1 for another page"), which is a thing a person
  has to be told. Typing "plans/a-1.pdf, 4" and Enter still works: the
  demonstrations and anyone quick with a keyboard use it.
- **The words on the list**: the field is "Plan" (it was "File, page"); "WHERE THIS
  JOB IS KEPT" over the list (it was "IN THE JOB'S FOLDER"); "Up one folder" (it
  named the folder above); "Picked usda-7213.pdf: it has 12 pages. Pick the page to
  trace over from the list." (it was "Usda-7213.pdf: Enter brings it in.", the
  file's name wrongly given a capital); "Nothing here that can be brought in (PDF,
  PNG, JPEG). Look in a folder, or go up."

### Call 95. What the twenty-third judgment found on the money path, and what was decided (2026-11-16)

A judge that did not build it tried the command line's money path after calls 91 and
92 (`research/judgment-23-notes.md`). Every total it worked by hand held. Eight
findings; these are the decisions in them.

- **A pitch or an eave height set on a plane solved from a roof's footprint is put on
  the roof** (J23-1). `set <plane> pitch 7:12` was taken, said to be Known, and thrown
  away at the next solve, leaving a file that failed its own check: the one thing
  call 91 fixed for `measure` and left open for `set`. It is now the roof's edge that
  changes, and the roof is solved again; the file checks.
- **How an edge of a roof rises can be read on site** (J23-2): `measure <roof> "edge
  1" 7:12`, or the same at the plane as its pitch. It answered that a pitch "is not
  one length ... move it to where it is". "same" still marks it found as drawn. The
  command line's usage names `edge:<n>` and `move:<n>`.
- **Whatever takes priced lines out of the job says the dollars** (J23-3): a roof
  edge made a gable (its plane goes), and an assembly changed for another (the lines
  the new one lacks come off). One sentence in the model says it for a delete, a
  dropped plane and lines that came off (`view::went_with_said`, `came_off_said`),
  at the command line and in the window.
- **An eave height at or below the job's zero is taken, and remarked on** (J23-4). A
  canopy over a basement door is there; far more often it is a reading typed wrong,
  and the program does not know which. A value that was already measured and is
  checked again is "found the same again", not "found as drawn".
- **A row of the recap with lines that come to no figure says "no figure" and how
  many are to price**, never "none" (J23-5): Labor read "none" over four labor lines
  with no production rate.
- **In the workbook the recap's hours are headed "Hours, as written"**, and a line of
  in-house work says in its note what its hours are worked from ("its hours are
  240.00 SF at 40 SF/hr") (J23-6). The hours are still not a formula: the sheet
  has the hours as its quantity, not the area and the rate, and making them live
  means two more columns on every line. Not done; said.
- **Words** (J23-7): a rate that cannot be read "is not a production rate; write how
  fast the work goes, like 40 SF/hr or 0.025 hr/SF" (it offered "30 SF/sheet"); "its
  production rate has to be more than nothing (and no more than a billion)" (it said
  "order factor ... 1e9"); `show` says "production rate" where it said "ordered by
  the hr"; `lines` no longer calls eaves and hips lines. **Left as they are**: the
  names in a library file's Quantity column ("face_area", "net_area"). They are what
  the program reads the file back by, and the file round-trips to the byte; a
  builder's words there need a second reading of the column. PROPOSALS.md 24.
- **Overhead or profit over 100 % is answered at the command line as in the window**
  (J23-8), by one sentence in the model.
- **Not done**: a production rate that is absurdly slow (0.0001 SF/hr) is taken
  without remark. There is no line a program can draw between slow and wrong that a
  mason and a finish carpenter would both accept; the hours are on the line to read.

### Call 96. Over a dressed roof the takeoff's colours wait until they are asked for (2026-11-16)

From the twenty-second judgment, part B (J22B-19; VISUAL.md D8): on the dressed house
in 3D the eaves, hips and ridges were bright green, orange and magenta lines laid
over the shingles, and each plane's tag was a square white box. "They make the house
read as a takeoff diagram, not a house."

- **In 3D, a roof that is dressed in what is priced on it is drawn as the roof.** Its
  eaves, rakes, hips, valleys and ridges are drawn in their takeoff colours on plan
  (where a drawing's edges are drawn by line type: Patrick's rule keeps that), on a
  roof not yet dressed (it is a takeoff then, and the colours are what there is to
  see), and while the roof, a plane or one of those edges is selected. A line with
  no length is a fault and is always drawn, broken and red.
- **A tag on the model is round and has a little shadow**, as everything that floats
  over the model is. Each plane keeps its tag (its number, area and pitch) in 3D: it
  is how a plane is told from the next, and it is one small thing where the lines
  were nine.

- **A roof's edge says what its figure is**: "Edge 1  5:12 pitch" on the roof's
  panel (it read "Edge 1  5:12").

Not done: the same for the coloured centre lines of ducts and pipes, which are what a
run is drawn as and have no other look yet (D6).

### Call 97. What the twenty-fourth judge's look and words changed in the window (2026-11-16)

The twenty-fourth judgment, part B, failed the look and the words
(`research/judgment-24b-notes.md`; STATUS.md has a row for each finding). These are
the decisions made in answering it. Each changes something a person sees.

- **"More below" is a strip of its own under what scrolls**, inside the panel's card,
  and a click on it goes a screenful down. It was a pill laid over the foot of the
  panel and it lay on words on nearly every screen (J24B-2). It takes 22 dots of the
  panel while there is more below, and none at the foot.
- **A panel opens at its head.** A thing just selected, and a tab of the sheet just
  picked, start at the top; the place a panel was scrolled to is kept only while what
  it shows stays the same (J24B-23, -30).
- **The glass is thinner** (0.44 of its tint where it was 0.60), and what is behind it
  is made a little richer in colour before the tint goes over: it was judged a grey
  sheet (J24B-1). Over a plan it is as it was: thin glass over white paper is no
  panel at all.
- **A form refused keeps what was typed, and the next key adds to it.** Told "type
  the unit with it", a person typed " ft" after the 9 and lost the 9 (J24B-45). A
  field that the next key will replace (one opened holding a value) is drawn with its
  words selected, as any program draws them, and Ctrl+A takes the whole of a field.
- **Backspace in an empty field does nothing.** It went back into the field before
  and rubbed that out (J24B-46). A comma or Tab still goes on; a click goes back.
- **The wall tool refuses a thickness over 4 ft**, saying a length is typed after the
  first corner is clicked: a length typed where the thickness goes drew a wall 20 ft
  thick without a word (J24B-48). Nothing in a house is 4 ft thick; a wall that is can
  still be made at the command line, which takes what it is given.
- **"All levels" brings back every level**, a level whose own switch was off among
  them (J24B-35).
- **A system's switch reads on only while its trade is on**, and switched on with its
  trade off it brings the trade back (J24B-37).
- **Supply air is a pale sky blue** (0x6FA8DC), well off the accent, so a selected
  duct can be told on plan (J24B-27). The Mechanical swatch follows it.
- **In Open items and under a line of the sheet, the mark says how sure and the words
  are in ink** (VISUAL.md A6: "a panel is not a column of red sentences"). Under an
  estimate line nothing repeats what a field on the line already says ("no price",
  "no production rate"). The recap names the first three lines the job cost leaves
  out and then says how many more, "each marked on its line below": every one is
  still marked on its own line, so nothing is left out silently.
- **The estimate's columns are Quantity, Unit cost, Material, Labor, and Sub,
  general** (they read Priced, At, Material, Labor, General). The column totals say
  what each holds: "Labor: labor, fabrication and supervision. Sub, general:
  subcontract and general conditions." The recap's "Labor none" stood over a LABOR
  total of supervision's money (J24B-32).
- **A labor line's switch reads "Labor on" or "Labor off"** (it read "On: switch
  off": a state and a command at once).
- **A field on the sheet opened empty shows what could be typed**: "like 40 SF/hr"
  for a production rate (J24A-4, J24B-54), "like 2.10" for a unit cost.
- **A roof plane's tag waits with the roof's coloured lines** (call 96 kept the tags;
  the judge of the copy before call 96 named "number labels on the roof" with the
  lines, and with the lines gone the tags were the diagram that was left). On plan,
  on a roof not dressed, and with the roof or anything of it selected, they are
  there.
- **The view's name comes first on the card at the lower right**, then where the
  pointer is: with the pointer off the model the card is one word and not a rule with
  nothing beside it (J24B-3). The line of what was done holds 170 letters (120).

Left as they are, with the reason: the refusal of a number with no unit still says a
laser has to be set to send its unit (the fixed rules name the laser, and that
sentence is for the person holding one); a system's class is written as its standard
writes it ("Supply Air", "Domestic Cold Water") where the job's own name for the
system is in a builder's hand ("Supply air").

### Call 98. Every panel has a switch; the lists a file is picked from, in a builder's words (2026-11-16)

Two more answers to the twenty-fourth judgment, part B.

- **Each of the three panels has a switch**, on a card at the lower right: Job (the
  lists down the left), Details (what is selected, down the right), Tools (the bar
  along the foot). The fixed rule is that every panel has a switch that reads on or
  off at a glance; only Next steps could be put away (J24B-40). A panel put away
  comes back by itself while something in it is being typed or a tool is in hand, so
  that nothing is typed out of sight, and its switch reads on while it is there.
  Switched off then, what was in hand is let go first.
- **The list comes first and nothing in it is a path.** A file is shown by its name
  without its ending, with what it is in a word at the right: PDF, picture, table,
  workbook, BIM model. A folder is shown by a drawn folder and its name; the way out
  is "Back to" and the name of the folder outside. The field to type a name in is
  under the list, headed "Or type its name" (it stood first, with "lists/ridge.xlsx"
  as its example). Call 94 kept "file" and "folder" as words; VISUAL.md A8 says "no
  folder path; no file name", the judge failed the list on it twice, and A8 is the
  standard: this replaces that part of call 94. "Folder" is still said in one
  sentence ("Look in a folder"): it is the office's word before it was a
  computer's.
- **Kept as they are:** PDF, IFC and CSV on a button or in a hint. They are what the
  people who send those files call them ("send me the PDF", "the IFC"), and the fixed
  rules name them. A file written out is still said by its whole name ("Wrote house
  estimate.xlsx, beside the job"): it has to be found afterwards, by that name.

### Call 99. A plan is not lit; a floor's underside is a ceiling; no grid with the ground off (2026-11-16)

- **On plan everything is drawn in its colour as given, with no light on it.** A plan
  is a drawing. Lit, the roof on plan was pale on the planes that faced the sun and
  dark on the rest (J24B-15, VISUAL.md E1 "no shadow").
- **A floor's underside is drawn as a ceiling**, in the colour of a wall's inside
  face, until something priced on it says otherwise. In the floor's own tan it read
  from the room below as raw wood (J24B-16).
- **With the ground switched off the grid is not drawn**: it is the ground's ruling,
  and it lay across the face of the foundation (J24B-25).
- **A selected footing is outlined** as a selected wall is (J24B-26).
- **No door swing is drawn** (J24B-14): the model does not hold which way a door
  swings, and one is not made up. When a door has a hand and a swing in the model,
  the plan draws them.

### Call 100. A roof is priced once; a rate can be said in squares; "entered on screen" (2026-11-16)

From the notes the twenty-fourth judge, part A, wrote while still at work (J24A-6 to
-8), and one word from part B.

- **An assembly given to one plane of a roof goes on every other plane of that roof
  that has nothing priced on it**, and the window says which: "Plane 1 is now
  "Asphalt shingle roof": 5 cost lines added. So are plane 2, plane 3 and plane 4:
  the same lines on each." A plane with lines of its own is left as it was and
  named. An estimator prices "the roof"; one plane of four was priced and nothing
  said the other three were bare (J24A-6). A plane that is to be different is
  changed afterwards, as any assembly is changed.
- **A unit cost or a production rate typed on a plane's line is taken by the same
  line on the roof's other planes where that line has none yet**, and it is said. One
  that has its own is left. What is carried is Assumed, as what was typed is. Four
  planes alike took twenty entries (J24A-8). A stretch-out is not carried: it is a
  measure of the piece on that line.
- **A production rate said in squares is taken on a line that comes out in SF, and
  the other way**: a square is 100 SF. "1 SQ/hr" is kept as 100 SF/hr, in the line's
  own unit, so that hours stay one division (J24A-7). A rate in a unit that is
  neither is refused as before.
- **A value entered in the program says "entered on screen; not yet checked against
  a drawing"** as its source. It said "set in the window", on a house full of
  windows (J24B-51). Four demonstrations read that source in the saved job and now
  read the new words.

### Call 101. More of the twenty-fourth judge's marked words (2026-11-16)

- **A fixture's or a piece of equipment's "connector" is its "connection"**, wherever
  a person reads it: "Connection 1", "connection 1 size", "its Waste connection",
  "already has a connection where the run ends". A maker's sheet lists a unit's
  connections; nobody on a job says connector (J24B-28). What is measured is named
  "connection 1 size" at the command line too. The job file's key is unchanged.
- **A plane's values say "worked out from the footprint of" its roof**, and Open
  items says a roof's planes "could not be worked out" (they said "solved": J24B-20).
- **A labor line's rows are "production rate" and "hours"** (call 100's commit after
  it; they read "order factor" and "to order", a material's words: J24B-56).
- **The sample house's footing type is "footing 24 in x 12 in"** (it had no units).

Left, with the reason:
- **Quantities to four places** on a line ("2.9630 CY", "3.8025 SQ"). The extension
  is that quantity times the unit cost, to the cent; shown to two places, a line
  would not come to what a person multiplying it out gets. Totals and stats are to
  two.
- **A slope factor's formula** on a roof plane's panel. Every number can show what
  it is made of, and that is what a slope factor is made of.
- **"Extension"** and, for a material, **"to order"**: an estimator's own words.
- **A height shown as "8.4 ft"**: a value is shown as it was entered, and the
  made-up house was entered in decimal feet. What is typed in feet and inches shows
  in feet and inches.

### Call 102. A sink, a water heater and an outdoor unit are drawn as what they are (2026-11-16)

VISUAL.md D6 ("a thing drawn as what it is") failed twice on boxes: the kitchen sink
a box hung on a wall, the water heater and the heat pump pale boxes (J24B-8, -28,
-29). Ten forms were ported from BIMCraft's library; it has no family for these
three (searched; PORTMAP.md's `blocks` row has the search). They are ours, in the
same function and under the same rule, tested: every block inside the thing's box,
and together reaching each of its sides.

- **A sink** is a bowl in stainless: a bottom and four sides. 30 in wide or more it
  is a double bowl, with a wall between the two.
- **A water heater or a tank** is an upright round, in painted sheet metal.
- **A heat pump's or a condenser's outdoor unit** is a cabinet with the round guard
  of its fan on top.

A thing is drawn so when its name says so ("sink", "water heater", "tank", "heat
pump", "condenser", "condensing unit", "outdoor unit"), as the ten are. Still boxes,
because a box is close to what they are and nothing better is claimed: an air
handler, a furnace, a panelboard, a meter socket, a register. Not built: a base
cabinet and a counter under a kitchen sink (cabinets are not in the made-up house or
in any assembly; STATUS.md, the seven trades).

### Call 103. A roof change that drops a priced plane says the job cost before and after (2026-11-16)

Found by working a demonstration by hand before running it (`demos/gablew.sh`). A hip
end made a gable drops a plane, and calls 91 and 95 have the window and the command
line say what went with it: "5 cost lines went with it: $71.55 is out of the job
cost". That is true of the lines that went and misleading as a whole: the two planes
beside the gable grow to cover what the hip plane covered, a gable and a hip of one
pitch cover the same roof, and the job cost moves by a cent of rounding. A person
reading "$71.55 is out of the job cost" would take the roof to be $71.55 cheaper.

- After that sentence the window and the command line both say, from one function
  in the model's crate: **"The planes that stay changed with it: the job cost was
  $429.32 and is $429.33."** It is said only when a plane that carried lines went,
  and only when the job cost has a figure before and after.
- The first sentence is kept as it is: it is what went, to the cent, and a person
  has to be told that lines they priced are gone.

### Call 104. A price list prices the cost lines named as its items, where they have no price (2026-11-16)

The twenty-fourth judge, part A (J24A-18, serious; J22A-15 before it): a supplier's
list was brought in, the window said "8 rows read, 16 set on the job", and no line of
the estimate moved. The list's prices went on the job's materials, and a cost line
takes a material's price only if it is "made of" its element's one material. The
lines the starter assemblies give are named as the list's items ("Lap siding",
"Asphalt shingles") and are made of nothing. The note written for J22A-15 said to
"select what it is on and give it that material"; the judge did, and the line still
kept its price: the advice was wrong.

- **A cost line that is named as a material on the job, is not made of one, and has
  no price, takes that material's price when a list is brought in**, where the two
  are priced by the same unit. It carries the list's status and its source, as the
  material does. Each is said: "Lap siding on wall 1: priced at $4.25/SF, the
  list's".
- **A line that already has a price keeps it**, and is named with both figures:
  "keeps $4.50/SF (the list says $4.25/SF)". A price a person typed is not written
  over by a file. To take the list's, it is typed on the line.
- A line priced by another unit than the list's item is named and left.
- Only the job's own price for a material does this, not one supplier's quote: three
  suppliers side by side are still a material's, and a line that is to be compared
  across suppliers has to be made of one. A line of its own that names the material
  it is made of is PROPOSALS.md 27.
- The wrong advice is gone from the note.

### Call 105. Patrick, 2026-11-16: a workbench for items, by trade, that can be modified like a database (and how it was read)

His words, in full: "just want to ensure there is a workbench for items for trade
that can be modified like a database of items / library like or similar to -
https://github.com/rgeometry/rgeometry https://github.com/olson-sean-k/plexus
https://github.com/hannobraun/fornjot".

**What there was when he asked.** A thing could be made into an item from a maker's
sheet, listed under Items, placed, moved and written to IFC. Thirteen shapes were
drawn by a word in a thing's name. There was no library of items, nothing kept them
by trade, an item could not be written out or brought back, and what it was drawn as
could not be chosen. So: not yet.

**How each clause was read.**
- *"items for trade"*: an item says which of the sixteen divisions' item it is, and
  items are listed, written out and read trade by trade.
- *"a database of items / library"*: an item library is a table, one row an item,
  kept in a file a person opens in any spreadsheet, changes and brings back. It
  round-trips to the byte, every value with how sure it is and where it came from.
  The program ships one (`library/items.csv`: 21 items in four trades, at typical
  sizes, each size Assumed and saying "a typical size; the product is not chosen").
- *"that can be modified"*: a row changed in the file changes its item when it is
  brought back, and says what changed. In the window an item is opened from the Items
  list (Change) and changed there: its sizes on their rows, its trade and what it is
  drawn as each with a click. A change is the item's, so every one of it already
  placed follows.
- *"a workbench"*: today that is the item's panel, with the building still in view.
  **A bench where an item stands alone, turned and sized by hand, is not built**; it
  is PROPOSALS.md 28.
- *"like or similar to" the three libraries*: each was read from its own pages
  (`research/item-libraries-survey.md`, by an agent on Sonnet, with what it could not
  verify). All three are permissively licensed (rgeometry Unlicense, plexus MIT,
  fornjot 0BSD). rgeometry is 2D computational geometry; plexus is polygon-mesh
  processing whose only release on crates.io is from 2019; fornjot is a b-rep CAD
  kernel whose README begins "This project has been shut down". **None is linked.**
  What an item needs here is a few sizes, a shape inside the box of those sizes, and
  a clash test on the box; `building::forms` does that, ported from BIMCraft's
  family library, which is the nearer model of "a library of parametric things":
  BIMCraft's `Family` (`crates/core/src/family.rs`) has parameters, types, formulas,
  forms, symbols and connectors. Porting that, so that an item's own shape is data in
  the library and not one of thirteen built in, is the real next step and is
  PROPOSALS.md 28. Whether he wants one of the three used for it is QUESTIONS.md
  Q27.

**What was built** (tests in `tests/tables.rs` and `tests/build.rs`):
`pl::item` gained `division` and `form`; `tables::items_out` and `items_in`;
`plumbline import items <file> <table|starter>`, `export items`, and `items` (the
job's items trade by trade); in the window, Starter items and Item library under
Bring in, Item library (CSV) under Write out, the Items list under each trade's name
with Change on every row, and on an item's panel "Drawn as" and "Trade" as rows to
click. A placed thing is drawn as its item says, else as its name says, else as its
box.

### Call 106. A step taken back says what the step was; "doors and windows" wants one of each (2026-11-16)

Two of the twenty-fourth judge's open rows, each a change to what a person reads.

- **J24A-12.** Undo said "Undone: Unit cost." and not which line or to what. A step
  is now remembered by the first line of what the window said when it was made, so
  taking it back reads "Undone: Plane 1: unit cost of "Asphalt shingles" set to
  $105.00", and doing it again "Redone: ..." the same. Where a step said nothing, it
  is still remembered by the short name of what was done ("Wall", "Delete"). A
  refusal is not a step and is never what one is remembered by. It does not say what
  the value went back to: the line shows it, and the history holds the job before the
  step, not a list of what the step changed.
- **J24A-2.** Next steps ticked "Add doors and windows" after one door. It is ticked
  with at least one door and one window; with only one of the two it says which is in
  and which is still to come. (A house with no window at all leaves the step
  unticked: the list is a guide, and no later step waits on it.)

Built, tests pass (no test holds either: both are words in the window). **Not seen
in the window yet**: two judges were driving windows on the machine when it was
built.

### Call 107. What the twenty-fifth judge found on the item library, as it wrote it (2026-11-16, night)

Part A of the twenty-fifth judgment is a veteran on the item library Patrick asked
for. These were fixed from its notes while it was still at work on the money path;
its report is recorded in STATUS.md when it arrives.

- **J25A-6 (silent, the serious one). A size changed in the library did not reach the
  thing already set down**, while the screen said "changed: its width": the range in
  the room kept its 30 in, and the clash test ran on the old size. Measure on the
  item's panel did carry a new size to what stood. Decided: **one rule, whichever way
  an item changes: what is set down from it is as big as it says.** Bringing a
  library in now carries a changed size to every one set down and says how many
  ("Range, 30 in: changed: its width; the one set down is now that size"). One
  function does it for both roads (`editing::placed_follow`).
- **J25A-3 (stuck). A thing set down could not be turned.** A panelboard on a wall
  that runs north stood with its width through the wall. Now: Turn a quarter on its
  panel, or Q (on foot Q is still the way down): 90 degrees counter-clockwise seen
  from above, about its own point; its size is its own and does not change; how it is
  turned is Assumed, "turned by hand in the model". It says which way its width now
  runs and what it is in the same space as, or that it is clear. Ported from
  BIMCraft's `rotate` (PORTMAP.md): the turn about the thing's own point is taken;
  about another point, walls and runs, several at once and a copy are not. A turn by
  a typed angle is not in the window.
- **J25A-4 (silent). Setting a thing down into a wall said nothing**; only a move
  did. A thing set down now says what it is in the same space as, in the tone a
  refusal has, worked out from the job as it stands.
- **J25A-5 (words).** "in the same space as Bathtub, 5 ft by 1' 4"" read as a size.
  How far two things are into each other goes in brackets after the name: "Bathtub,
  5 ft (1' 4" into each other)".
- **J25A-1 (look). Plumbing fixtures on plan were white on white paper**, with no
  edge. Every block of a placed thing is outlined in ink on plan, as a plan draws a
  fixture. They are still not named on plan.
- **J25A-2 (look). Taking an item in hand on plan threw the view to 3D.** A thing is
  set down on the plan or in 3D, whichever is on show; only a walk is left.
- **J25A-7 (silent, minor). An item with no weight was written as three blanks.** It
  is written Unknown, "not said", as a maker not chosen is written, and read back as
  the same item. `library/items.csv` says so on every row; it still goes out and
  comes back the same to the byte.
- **The message log** wrote a thing's kept name where the screen wrote its own
  ("fixture-0030 is one of it"): the log is now written line by line as the screen
  is.
- From part B's notes, **J25B-15**: in the view of how sure each thing is, a roof's
  hips stayed orange and read as Assumed. There each line of a roof is drawn in the
  colour of how sure it is.

Tests: `a_size_changed_in_the_library_reaches_what_is_already_set_down_and_says_so`,
`a_thing_set_down_is_turned_a_quarter_at_a_time_and_a_fitting_is_not`. 317 pass.
**None of it has been seen in the window**: both judges were driving windows on the
machine. Not taken up from the notes so far: the "Drawn as" list has no range,
refrigerator, washer or panel (they are boxes: PROPOSALS.md 28); a panelboard stands
on the floor, nothing hangs it on a wall at a height (its height above the level can
be measured on its panel); the switches under Show are not kept with the job.

### Call 108. A supplier's price list prices the job where the job has no price (2026-11-16, night)

**J25A-12, and the judge stopped on it.** The twenty-fourth judge found that a price
list priced no line (J24A-18) and call 104 answered it: a cost line named as one of
the list's items takes the material's job price. The twenty-fifth judge brought a
list in the way the window invites, with the supplier's name after the file's
("supplier-copy.csv, Supplier A"), and nothing was priced: with a supplier named, a
row sets that supplier's quote, the job's own price stayed unknown, and call 104
prices a line from the job's price. The test that held call 104 brought its list in
with no supplier named, which no one in the window does.

Decided:

- **A supplier's price is the job's too, where the job has none.** The material's own
  price takes it, as sure as the list was said to be, and says whose it is ("Supplier
  A's price list: ..."). The row says so: "Lap siding from Supplier A: $4.25/SF, which
  Supplier A had not quoted; the job had no price for it and takes Supplier A's".
- **A price the job has is not written over by a list.** A second supplier's list
  sets that supplier's quote beside the first; the job's price and every line stay,
  and suppliers side by side shows both.
- **A line is set only against what the list brought in names.** (Any list brought in
  noted again every line that differed from any material's price.) A line with a
  price that differs from this list's keeps it and is named with both figures. A
  line with no price takes the job's price; where the job already had one and this
  list says another, both are said.
- **The lines priced are said first**, ahead of the materials added: they are what
  the list did to the estimate, and were behind "and 16 more".
- **What the list names that prices nothing is said**: an item no cost line is named
  as and nothing is made of ("1 of this list's items prices nothing on the job yet
  ...: Fiberglass batts R-21"). A name spelled another way on the list was added as
  a material like any other row, with nothing to say it matched no line.

Test: `a_named_suppliers_list_prices_what_the_job_has_no_price_for_and_writes_over_nothing`.
318 tests pass; the twenty-one command-line demonstrations pass and none of their
figures moved (each brings its suppliers' lists onto a job that has its own
prices). `demos/bringw.sh` brings a named supplier's list onto a job whose materials
have no price: its figures are expected to move, and it is run and read before
anything is said of it. **Not seen in the window.**

Also from part A's notes, and left as they are with the reason:
- J25A-10: a rate typed "2 hr/SQ" is shown on the line as "at 0.02 hr/SF". It is kept
  and shown in the unit the line's quantity is in (call 100) and is exact (2.5 hr/SQ
  is shown 0.025, not 0.03); it is not shown in the unit it was typed in. Open.
- J25A-11: roof sheathing is rounded up to whole sheets plane by plane: 12 sheets on
  four planes of a 313 SF roof where 10 cover it. A line rounds up what it orders,
  and each plane has its own line. Rounding once for the roof is a rule of the model
  (what a line orders against what the job orders) and is not changed tonight; it is
  said here and in STATUS.md so that nobody takes the sheet count of a hip roof for
  an order.
- J25A-9: Unit cost on a plane's panel opens the price of its first line only; the
  others are priced on the sheet. Open.
- J25A-13: Steeper took the price up $6.10 and the message spoke only of the pitch.
  **A roof change that drops nothing says what the job cost was and is, when it
  moved** ("What is priced on the roof changed with it: the job cost was $71.55 and is
  $61.22."), as one that drops a plane has since call 103. Test:
  `a_pitch_made_steeper_says_what_the_job_cost_was_and_is`, its figure worked by hand.
  A value measured from the Verify in field list re-solves the roof and moves the
  price too, and does not yet say by how much: open.

### Call 109. What failed the look and the words a third time: a plan is of one level; a door in feet and inches; no "(IFC)" on a button (2026-11-16, night)

The twenty-fifth judgment, part B, failed item 11 on one thing above the rest and item
9 on two (STATUS.md has its report). Each is a decision about what a person sees.

- **A plan is of one level (J25B-3, VISUAL.md E1).** Plan pressed on a finished house
  with every level on show drew the roof, solid, over everything: "a builder pressing
  Plan on a finished house sees nothing." It was so by an earlier decision (with every
  level on show the roof itself is drawn; on the plan of one level it is its lines,
  broken), and a person who opens a job and presses Plan has every level on show.
  Now: **asked for with every level on show, on a job with a roof or more than one
  level, the plan puts one level on show**: the lowest with a door or a window in it
  (on the sample house the first floor, not the foundation), else the lowest. It says
  so: "On plan: First floor, north is up. A plan is of one level: Page Up and Page
  Down go to the others, and All levels shows the roof from above." Leaving the plan
  puts every level back, unless another level was chosen meanwhile. All levels on plan
  is still the roof seen from above, which is a roof plan and is asked for by name.
  A job with one level and no roof, or with no level, is as it was.
- **A door's and a window's sizes are offered in feet and inches (J25B-28).** The Door
  tool offered the last door's sizes as they are kept: "2.5 ft", "6.67 ft". They are
  offered as a tape reads them, to the eighth: 2' 6", 6' 8". (The sample house keeps
  its doors at 6.67 ft, which is 6 ft 8 1/32 in: a door placed from the offer is a new
  type at 6 ft 8 in, and its panel says so. The sample is not changed tonight: every
  figure of its south wall is worked from it.) "Sill height" on a door is still
  there: open.
- **No "(IFC)" or "(CSV)" on a button (J25B-29, -39; part A's J25A-19 says the same).**
  Bring in: "Designer's model" for "Model (IFC)". Write out: "BIM model", "Estimate
  list" for "Estimate (CSV)", and "Item library, to keep" for "Item library (CSV)"
  (Bring in has "Item library", and two buttons do not share a name). In a sentence:
  "the estimate as a list". The demonstrations that click them were given the new
  names. "BIM" stays: it is the word Patrick used and the word on a designer's
  transmittal.
- Placed things on plan are outlined in a lighter line than a wall's.

**Seen in the window by hand** (not by a judge): Plan on the sample house shows the
first floor with its rooms, walls in ink and fixtures outlined; the sentence above;
an item taken in hand on plan stays on plan, is set down, turned a quarter ("Turned
... 90 deg, Assumed, turned by hand in the model" on its panel); T again and the whole
house is back in 3D.

Also from part A's report (its last findings):
- **J25A-18 (silent, serious). Starter items pressed a second time wrote over a
  person's own changes** to three items (a lavatory measured at 24 in went back to 20
  in, Assumed) and called it "Brought in 3 starter items". The library the program
  ships with now adds what the job has not got and leaves every item the job has as
  it is, and says how many were left. A library a person brings in themselves still
  changes the items it names: there a changed row is meant. Test:
  `the_starter_items_brought_in_again_write_over_nothing_the_job_has`. (Starter
  assemblies pressed twice has not been tried for the same fault: open.)
- **J25A-16. A refused price closed its field**, and the "0" typed next to put "-12"
  right took the Cable tool in hand. The field stays open on the same line with what
  was typed; typing replaces it.
- J25A-17 (a pitch of 40:12 and an eave moved 40 ft taken without a word): open. A
  roof change now says what the job cost was and is (call 108), so the 40 ft eave
  says its $1,937.66; nothing questions the figure itself.

### Call 110. From the twenty-sixth judge's notes: a typed length to the cent, a comma in a line's name, a markup's source (2026-11-16, late in the night)

Fixed from the judge's notes while it was still at work; its report is recorded in
STATUS.md when it arrives.

- **J26A-3 (wrong money, by a cent). A typed length was kept to four places of a
  foot.** 28 ft 4 in became 28.3333, a floor 41 ft 3 in by 28 ft 4 in was shown as
  1,168.75 SF and held as 1,168.7486, and at $6.00/SF came to $7,012.49 on the top bar
  and in the saved job where a calculator says $7,012.50. "An estimator checking a
  line with a calculator gets a different cent from the screen." **The end of a typed
  length is kept to a hundred-millionth of a foot** (`editing::toward`): fine enough
  that what is typed multiplies to the cent, and still tidy enough that 0.1 + 0.2 is
  0.3. A length that is a round figure in feet or a binary fraction of one (41 ft 3
  in) is held exactly as before; a length in thirds of a foot (4 in, 8 in) is held to
  eight places where it was held to four, so **a job traced tonight and one traced
  yesterday can differ in the fifth place of such a corner**, and in the cent that
  follows. A job already saved is not changed. Test:
  `a_typed_length_is_kept_finely_enough_to_multiply_to_the_cent`.
- **J26A-5 (breaks). A comma could not be typed in what a cost line is called**: in a
  set of fields a comma goes on to the next, as Tab does, so "Exterior door, prehung"
  was cut in two and the line refused. In that one field a comma is a comma; the
  program's own lines are written with them.
- **J26A-1 (words).** Overhead and profit read "entered on screen; not yet checked
  against a drawing". A price, a rate, a wage and a markup typed in the window say
  "entered on screen": none of them is on a drawing. What is drawn or sized on screen
  still says it has not been checked against one.
- J26A-2 (sheathing rounded up plane by plane) is J25A-11 again, from a second judge:
  open, and said in STATUS.md. J26A-4 (an allowance is not in the price) is Q26, from
  a fourth.

**None of the three is seen in the window**, and the window demonstrations have not
been run on a build with them: the judge was driving a window on the machine.

### Call 111. The item library is kept by hand in the window: an item added, renamed, taken out (2026-11-16, late in the night)

Patrick asked again, in the same words, whether there is "a workbench for items for
trade that can be modified like a database of items / library". The twenty-fifth
judge's answer had been "not yet a library I can keep", and one of its reasons was
that in the window an item could be changed and not added, renamed or deleted: a new
one came only from a written list or a maker's sheet. A table that is kept like a
database has rows added, renamed and removed where it is used.

- **New item**, at the head of the Items list: what it is called, its kind (equipment,
  fixture or furnishing), and its width, depth and height. Its sizes are **Assumed,
  "typed in the window; not yet checked against its maker's sheet"**; its maker and
  model are Unknown, not chosen; its trade is not said until it is picked on its
  panel, and the window says so. A comma in its name is a comma.
- **Rename item**, on the item's panel: two items cannot share a name however it is
  typed; each thing set down from it is called what the item is, and how many were
  is said. Ported from BIMCraft's `rename_type` (PORTMAP.md), which
  `tools/portmap.sh` made me find: I had written that BIMCraft has none.
- **Delete item**, on the item's panel: **an item is not taken from under what is set
  down from it** ("one is set down from Wall oven, 27 in: take it out of the job
  first"); one nothing stands from goes, and the window says Undo brings it back.
- `demos/window.sh` gained `on_right`, the wheel over the right panel until a control
  is on screen, as `on_at` does on the left.

Tests: `an_item_is_renamed_with_what_stands_and_is_not_deleted_from_under_it`; 322
pass. `demos/itemneww.sh` walks it in the window and passed on its second run (the
first found my own script reaching for a control on the right panel with the left
panel's wheel, and then expecting words the command line does not use for an item
drawn by its name). **Not right**: the New item form opens at the foot of the left
panel and its lower fields are under "more below" until the wheel is turned. **Not
built**: a price, a unit or labor on an item; an item's own shape; a bench where it
stands alone (PROPOSALS.md 28). Not judged.

### Call 112. A wall typed north is north: what the finer place showed (2026-11-17)

Call 110 kept the end of a typed length to a hundred-millionth of a foot. The first
run of the demonstrations that type lengths on a build with it moved one figure: in
`demos/planw.sh` the farmhouse's outside walls came to 1,042.667 SF of face where the
script wanted 1,042.6672.

Neither was right. The old expectation was worked from a wall 23 ft 11 in long held as
23.9167 ft, 0.0004 in too long, which is the fault call 110 mended. The new figure was
off the other way: **the length is laid off toward the pointer, and a pointer held "on
the angle" is on it to a dot of the screen and no better.** Kept to four places, that
lean was rounded away with everything else; kept finely, a wall typed north leaned a
few hundred-thousandths of a foot east and its neighbours' faces moved in the fourth
place.

Decided: **a direction that is east, west, north or south to within a hundredth of a
percent is that direction exactly** (`editing::toward`). A wall typed 23 ft 11 in north
from 41 ft 3 in east ends at 41.25, 23.91666667. The demonstration's figure is worked
again by hand in its comments: each wall's face is kept to four places, 8 x
23.91666667 = 191.3333 SF, and 2 x 330 + 2 x 191.3333 = 1,042.6666 SF, which is what
the program gives. The expectation was changed after this was written, and the
script's working says what the old figure was and why it was wrong. A direction off
the square (a wall at 45 degrees) is still as good as the pointer's dot.

Test: the same in `a_typed_length_is_kept_finely_enough_to_multiply_to_the_cent`.

### Call 113. Patrick, 2026-11-17: what comes in and goes out, and other agents bringing a trade's knowledge, prices and specs (and how it was read)

His message, as he wrote it: "I need to confirm what files I can import and export into
the program it would be beneficial for me to have other AI agents in the future to
upload trade knowledge , pricing , CAD similar specs like an auto converter from
something simple like an xls to something more powerful or complex like a machine shop
spec or countertop slabs". He also said three earlier questions had not been answered
(two restarts of the machine had replayed them; the answers were in a message he may
not have been shown).

How each clause was read:
- *"what files I can import and export"*: answered from the program's own list of
  verbs, not from memory. In: plans as PDF (drawn or scanned), PNG, JPEG; price lists,
  assembly libraries, item libraries and prior estimates as CSV or XLSX; a designer's
  model as IFC2x3 or IFC4; a maker's sheet as PDF; the program's own job file. Out: the
  estimate as an XLSX workbook of live formulas and as CSV; the supplier, subcontractor
  and PM lenses as XLSX or CSV; prices and assemblies as CSV or XLSX; the item library
  as CSV; the building as IFC4. **Not in or out**: DWG, DXF, STEP, STL, Revit files,
  any shop or machine format.
- *"other AI agents ... upload trade knowledge, pricing"*: this is the harness for each
  division of calls 75 and 76, which is not built. What an agent can do today is what
  a person can: write a table (prices, assemblies, items) and bring it in. What it
  brings arrives Assumed with its source named, and is never Known on its say-so; that
  rule is already in the tables.
- *"CAD similar specs like an auto converter from ... an xls to ... a machine shop
  spec or countertop slabs"*: **new scope, not built, and nothing in the program does
  any of it.** It is two things: a table of parts turned into shapes (an item's own
  shape as data is PROPOSALS.md 28), and shapes written out in a form a shop cuts
  from. It is PROPOSALS.md 30, and what a shop wants to receive is QUESTIONS.md Q28.

Nothing was built on this message. It asks for confirmation and states a direction;
product scope is his.

### Call 114. Patrick, 2026-11-17: "Allowances are allowed and noted as such" (QUESTIONS.md Q26, ruled)

His words: "Allowances are allowed and noted as such". Q26 asked whether an allowance
on a thing with no cost lines is in the price; four judges had read "price" and
expected it in, and the builder's working assumption had kept it out and said so.
**He ruled it in.** How it was read:

- **An allowance on a thing with no cost line is carried in the job cost and so in the
  price**, with overhead and profit on it as on any cost. It stops being carried the
  moment the thing has a cost line that is on: then the cost lines are the price and
  the allowance is what they are held against on the Budget tab, as before. A thing
  with both is never counted twice.
- **"Noted as such"**: it is never mixed into a cost type. It has a row of its own in
  the recap, **Allowances**, between the cost types and the job cost; the job cost row
  says "and allowances"; the sentence under the price says how many things and how
  many dollars are carried as allowances; the top bar says "$X in allowances"; the
  command line prints the same row; the workbook has the row, its figure as written
  and its note naming each thing, and the job cost's formula adds it. It is in none
  of the sixteen divisions: it is counted with what the sixteen leave out, so the
  sixteen and that row still come to the job cost.
- An allowance with no figure (its area not known) is left out of the job cost and
  named with what is left out, like a line with no figure.
- It is as sure as what it rests on: Assumed where the rate or the quantity is.
- Not changed: the three columns of the sheet (material, labor, sub and general) are
  over cost lines and an allowance is in none of them; the verify-in-field list's
  dollars ride on cost lines and do not yet count an allowance's.

**Every job with an allowance on a bare thing now has a larger job cost and price.**
The expectations that move with it are changed after this was written.

### Call 115. Patrick, 2026-11-17: "rebrand to BuildOS" (and how far it was taken today)

"plumbline" was a working name (the README said so in its first line). The product
is **BuildOS**.

Done today, everywhere a person reads the name: the wordmark at the head of both
windows; the two windows' titles ("BuildOS: model", "BuildOS: sheet"); the README's
title and first lines; this repository's instructions; the sentence an IFC model
brought back in says about who wrote it. The demonstration kit finds the window by
its new title.

**Not done today, and still called plumbline**: the folder `plumbline/`; the crates
(`plumbline-world` and the rest); the two commands a person types (`plumbline`,
`plumbline-app`) and so every usage line the command line prints; the environment
names the demonstrations use (`PLUMBLINE_BIN`, `PLUMBLINE_MAP`). Renaming those is one
mechanical change across about fifty scripts, and it is owed as a commit of its own
made on a head whose every demonstration has just passed: the machine was restarted
twice the night before under long runs, and the head has not had a full run since.
Doing it now would put two risks in one commit.

**Never to be renamed without a ruling**: `pl::` at the head of every attribute in a
saved job, and `Plumbline_Values` in a written IFC model. They are the file format;
jobs and models already written have to keep reading.

**Not checked**: whether the name BuildOS is free to use (a trademark, a company, a
web address). That is a search outside this machine with the product's name in it,
and it is Patrick's to ask for. The pictures in `demos/captured/` taken before today
show the old wordmark and are stale in that one respect until the next full run.

### Call 116. Patrick, 2026-11-17: what the lead holds and what is handed off (and a correction)

His words: "I will be out of fable 5.2 context what do you feel you need to control vs
what can be delegated. Think big picture framework. Do you not think I've been
looking at all the screenshots ? It's not polished but if what you were spending
millions of tokens on and 5-6 days with two resets I would have stopped this a long
time ago".

**A correction to what the builder had said.** It had told him nobody real had seen
the program and that it lacked his eyes. He has read the pictures all along. What is
true is narrower: nobody has driven it on a machine with a real graphics card and a
screen, and it has not been set against a real estimate. The first is a matter of
where it is run; the second waits on a sanitized estimate in `refs/`.

**The framework** is in `CLAUDE.md`, "Who holds what": the owner, the lead, workers
and judges, with the test for whether a piece of work can be handed off (bounded
files, a check that exists first, no change to a number, a meaning, a format or an
expectation, and a result that can be accepted from a diff and its pictures). It is
there and not here because it is how the work is done, and that file is read first
every session.

### Call 117. Looked at as someone who has to work it, at a desk and on site; the estimate read by item (2026-11-17)

Patrick offered one more pass: "truly view it and see if you can see what I would
change as if you had to work this in the field or office what's sticking out the
most". Six screens of the priced sample house were read at full size, one at a time,
not on contact sheets. What stuck out, most first:

1. **The estimate was a list of the model, not an estimate.** 399 lines: "footing 1,
   Concrete footing, in place, 2.9630 CY", then footing 2, 3 and 4, then four walls.
   An estimator writes one line: footings, 10.07 CY at $450.00, $4,533.40. **Built
   today**: "The estimate, by item" above the sheet: 52 items for that house, each
   with how many places it is made of, its quantity, its price where every place is
   at one price ("prices vary" where not) and what it comes to, under its division
   with the division's total. It is the same lines and the same amounts added up in
   the model (`Estimate::by_item`); the line-by-line sheet below is still where a
   price is typed. Test: `the_estimate_reads_item_by_item_and_comes_to_the_same_money`.
2. **Money is not where the eye goes.** Click a wall and its panel leads with its area
   and length; what the wall costs is under "more below". **Built today for a thing
   selected**: under its two figures, Cost (its cost lines added up) and All in (that
   for each SF or LF of it), or Allowance where it has no cost line; the sample
   house's south wall reads $10,990.84 and $33.31/SF (10,990.84 / 329.99 = 33.307).
   Seen by hand. Still open: the price of the job is in small type on the top bar, and
   the Job panel lists lengths and areas by level and not one dollar by division.
3. **The verify-in-field list cannot go to the field.** It says "first floor" and not
   which room; "Check what it is, size" and "piece 4 of 4" are the program's words;
   inches are decimals (0.50 in); thirty-eight-dollar elbows sit in the same list as
   a $7,800 heat pump; nothing can be ticked off; and it cannot be printed or written
   out from the window at all. **Built today**: each question knows the room it is in
   (the room whose outline holds where the thing stands, or where two things meet):
   "first floor, utility"; inches are written as a tape reads them ("by 6 3/8 in");
   a run is named and not its piece; what to do is a sentence ("Check what it is and
   its size."); one wording, the model's, for the window and the sheet; and **Field
   list under Write out** writes `<job> field list.xlsx` (the command line: `export
   vif`), one row a question, the most dollars first, with a box to tick, where,
   what, what to do and a blank for what was found. Test:
   `the_list_goes_to_the_field_with_its_room_its_fractions_and_a_box_to_tick`. Seen
   by hand on the sample house: 93 rows. **Still open**: the trifles are in the same
   list as the large items (thirty elbows at $38); the command line's own `vif` verb
   still words its rows itself; what is ticked on paper is typed back in one value at
   a time; nothing here is a phone.
4. **Figures are written to more places than the trade writes.** 210.00 LF, 1,120.00
   SF, 2.9630 CY, a height as 10.00 ft beside 10' 0". (In the new table a count of
   whole things is written whole.) Open elsewhere.
5. **A plan with no dimensions on it.** Rooms carry their names and areas; no wall
   says how long it is, and a door has no swing. An estimator checks a takeoff
   against dimension strings. Open.
6. **Noise that is always there.** A finished job still gives the top of the left
   panel to a checklist with nine of ten ticked; "Cost code: Unknown, not coded" is in
   red on every thing selected; a wall's line is given as coordinates. Open.

Not looked at in this pass: drawing a job from nothing, and anything in motion.

Later the same day, the smaller three, each in part:
- **4, figures.** On the Job panel a total is written without zeros it does not need:
  210 LF, 1,120 SF; 1,793.02 SF stays as it is. Nothing is rounded. The takeoff tables
  keep two places so a column lines up, and a priced line keeps the places it needs
  to multiply to its amount: both still open as to how the trade would write them.
- **5, the plan.** On the plan of a level each wall carries its length in feet and
  inches along its line, in the colour of how sure the wall is; returns under 3 ft are
  left bare. Seen by hand: 40' 0", 28' 0", 12' 0", 10' 0" on the sample's first floor.
  A tag can still sit over a duct or a piece of equipment; there are no dimension
  strings to openings; a door still has no swing.
- **6, noise.** A job that already has cost lines opens with the checklist folded to
  its one next step. The red "not coded" on every thing selected is as it was.

### Call 118. The job's money on the Job panel; "not coded" said quietly; a door's swing is not made up (2026-11-17; the rest of call 117's second, fifth and sixth points)

1. **The Job panel says what the job comes to, first.** With nothing selected the
   right panel was the job's name, its markup, then lengths and areas level by level:
   the one figure an estimator opens the job for was 16 px in the top bar and nowhere
   on the panel headed "Job", and no dollar figure by division was in the model
   window at all. Now, under the job's name, where anything is priced: the price,
   large, in the colour of how sure it is; under it the job cost, the overhead and the
   profit; then **By division**, a row for each division with work in it (its number,
   its name, what it comes to), and a row for what is in no division where anything
   is. Every figure is the estimate's own, the same as the estimate window's recap and
   its By division table: nothing is worked out in the window. A division with lines
   and no figure says "no figure", never $0.00. Sixteen rows with twelve of them empty
   would be the noise call 117 complained of, so the empty ones are left to the
   estimate window, which lists all sixteen.
2. **"Not coded" is not an alarm.** A thing with no cost code showed a red "Unknown"
   mark and red words on every panel of every thing, on a job where nothing is coded.
   A cost code is how a line is filed in a builder's books; it is not a fact about the
   building that anyone verifies. The row now reads "Cost code  not coded" in quiet
   ink with no mark. Its status is unchanged (it is still Unknown in the job, in the
   tally and under Open items); only the panel stops shouting it. A cost code that is
   set is shown as before.
3. **A door's swing is not drawn**, because the job does not hold a door's hand and a
   swing drawn without one is made up (the fourth fixed rule). It is PROPOSALS.md 31.

### Call 119. A dimension is written in feet and inches; a quantity keeps its decimals (2026-11-17; call 117's fourth point, and the words judges' "lengths written four ways in one panel")

One panel wrote a wall's height as "10.00 ft", its line as "E 0' 0" to E 40' 0"" and
its wainscot "to 3' 0"". The trade writes two things two ways and no others:

- **A dimension** (a height, a width, a thickness, an elevation, a length read off a
  plan: anything whose unit is ft) is written as a tape reads it, feet and inches to
  the sixteenth: 10' 0", 6' 8", 23' 11". It was decimal feet to two places.
- **A quantity** that is priced or added up (LF, SF, SQ, CY, EA) is written as a
  decimal, as an estimate sheet has it: 40.00 LF, 329.99 SF.

Nothing is rounded in the job; this is how a worked-out figure is written on a panel
and in the Open items list. A value a person typed is shown as it was typed, as
before. The estimate window's tables are not changed by this.

The same day, the words on a thing's cost rows, which two words judges marked
("Comes to, detail", "Comes to, detail rate"): on a panel they read **Detail comes
to**, **Detail, all in** (its rate for each SF or LF), **Allowance comes to**,
**Detail over allowance** and **Detail over allowance, %**. "Detail" is the owner's
word for a thing priced line by line. The job and the command line keep the older
names; only the window's wording changed.

### Call 120. The estimate by item at the command line; an item's quantity written to the places it needs (2026-11-17)

`plumbline estimate <file> by-item` prints what the estimate window's "The estimate,
by item" shows: each item once, with its cost type, its places, its quantity, its
unit cost and its amount, under its division and the division's total, and the job
cost at the foot. It is `Estimate::by_item`, the same as the window reads; nothing is
worked out at the command line.

An item's quantity follows the rule a line of the sheet already follows (J17A-9): two
places, unless at two it would not multiply by its price to its amount on a
calculator, and then the four it is held to. The sample's footings read 10.07 CY at
$450.00 beside $4,533.40, and 10.07 × 450 is $4,531.50; they now read 10.0742 CY
(which gives $4,533.39: the cent is four lines each rounded to the cent and added,
and the amount is that sum, as the sheet under it shows). One writer,
`ItemTotal::quantity_said`, for the window and the command line.

### Call 121. What the pictures of the full run showed, read by a worker; a drag begins where the button goes down (2026-11-17)

All twenty-five window demonstrations passed on 5be55e3 and a worker (not the mind
that built it) then read every picture against what its step says it shows. Its
notes are `research/pictures-read-5be55e3.md`; where each of its thirty findings
stands is in STATUS.md. Two decisions come out of it at once.

1. **A drag begins where the button goes down.** On the frame a button is pressed,
   the pointer's movement in that frame is not part of the drag. On a slow frame the
   pointer's whole journey to the place arrived with the press, so a right-drag that
   was to turn the view half way round also tilted and slid it: two pictures of
   `dressw` claimed the north wall and showed the inside of the roof, and passed. On
   a fast machine the difference is a dot or two. This changes what a drag does on
   the slow renderer; no expectation of any demonstration is changed by it. And the
   place a view turns about, or the wheel closes on, is the place under the pointer
   **where the pointer is now**: what it was last found to be over is used only if
   it lies on the line of sight from there, and otherwise the ground under the
   pointer is. (On a slow frame the view turned about the place the pointer had
   left, and the house swung off the screen.)
2. **A demonstration that claims a view measures it.** `dressw` now turns from the
   whole job in view, and measures the colours of the north and east walls in the
   picture that claims them, as it already did for the south and west.

### Call 122. Words and small things from the pictures read (2026-11-17; `research/pictures-read-5be55e3.md` 2, 10, 12, 17, 20, 23, 29)

Each is a wording or a showing; no figure changes.

- **West and south.** A place west or south of the plan's corner read "E -2' 1"" and
  "N -0' 6"". It reads **W 2' 1"** and **S 0' 6"**, in the panels and under the
  pointer. Height keeps its sign (a footing is below the datum).
- **A place given in inches is inches.** A connection's place on a fitting, kept in
  inches, was written as if feet ("H 3' 0" in"). It reads H 0' 3".
- **A second reading refused is a caution, not a refusal of the first.** "Ground:
  level height is 8' 10 1/2", measured ... A second reading came straight after that
  one and was not used" was all in the red of a refusal though the first reading was
  taken. It is in amber.
- **"No price" is not "prices vary".** On the estimate by item, a row whose lines
  have no price says "no price"; one with some priced says "2 of 4 priced"; "prices
  vary" is only for a row priced throughout at more than one price.
- **A roof's place is "Roof"** on the verify-in-field list (it was blank: a roof is
  on no level).
- **A room's outline.** "whether wall 1, wall 3 and wall 5 bound this room rests on
  their heights, which are assumed" did not read. It is **"its outline counts on
  wall 1, wall 3 and wall 5, whose heights are assumed"** (one wall: "counts on wall
  1, whose height is assumed").
- **What a designer's model holds that this program does not.** "this program has no
  element for it" used a software word. It is **"nothing here to bring it in as"**
  ("the model gives it no place, and there is nothing here to bring it in as").

The expectations of three tests of the model, one of the window's place words, two
of the BIM reader and two demonstrations (`export.sh`, `model.sh`) change with the last two, word for word.

### Call 123. An allowance is on its own row wherever the job cost is read by division, and its dollars ride on the field list (2026-11-17; the twenty-seventh judge's notes, J27A-5, -10, -11, -15, read while it was still at work)

Call 114 carried an allowance in the job cost and, to keep "the sixteen and what is
in none come to the job cost", folded it into the figure for "in no division". The
judge met the result four times: a section headed "No division $6,862.67" whose only
line was $450.00; a Job panel row "In no division $6,412.67" with no word that it
was an allowance; a recap by division that did not read as footing; and a
verify-in-field list that left the largest number in the job out of every dollar
figure, though the floor it is on was traced and Assumed.

- **Three things come to the job cost: the sixteen divisions, the lines in no
  division, and the allowances.** "In no division" is lines only, everywhere, so a
  section's total is the total of its lines. **Allowances** has its own row beside
  it: on the estimate window's By division table, on the Job panel, at the command
  line, in the workbook's By division sheet (where "in no division" is now the job
  cost less the sixteen less the allowances) and in the PM's lens. It is named an
  allowance each time (Patrick: "allowed and noted as such").
- **What is carried as an allowance rides on the field list.** An allowance is an
  area or a length times a rate; where the thing it is on is not Known, a tape can
  move it, so its dollars ride on that thing's question as a line's do, and are in
  the list's total.

No amount, job cost or price changes. The figure called "in no division" changes
wherever an allowance had been folded into it, and the verify-in-field dollars rise
by the allowances that rest on something not Known.

### Call 124. Seven stops and silences from the veteran judge's notes (2026-11-17; J27A-9, -12, -13, -17, -19, -20, read while it was still at work)

- **"x" is times in a quantity** ("face area x 0.10"), as every takeoff sheet writes
  it and as the program itself writes "quantity × unit cost". And two figures side
  by side now say what goes between them: "put +, -, x or / between them".
- **"each" is one.** A door is priced each: "each" or "EA" typed as a quantity is 1.
- **A markup from 50 % up is asked about.** 100 typed for 10 went in without a word
  while 1,000 was questioned. From 50 % to 100 %: "That is a large markup. If 10.00 %
  was meant, enter it again." Over 100 % the words are as they were.
- **A plane's panel has Whole roof**, which picks the roof the plane is part of; the
  refusal to delete a plane says so. (Nothing on the model picked the roof itself.)
- **The command line's takeoff has the roof in it**: each roof's planes, surface,
  squares, predominant pitch and its eaves, hips, ridges and valleys by length.
- **On the field list** a room reads "Check where it is" (it read "Check its a point
  inside it").

### Call 125. What a line is priced by is changed on the line (2026-11-17; J27A-16)

The rule is that a formula is visible and editable on the line. In the window it was
visible ("Quantity 298.00 SF, net area") and could be set only when the line was
added; after that only the command line could change it, and the veteran judge
stopped there ("I could not find how to change what a quantity is made of").

On the estimate sheet a line's quantity is now a field, as its unit cost is: a click
opens it with what the line is priced by now, in words ("net area"), and what is
typed replaces it ("face area x 1.10", "12", "each"). It is the same setting the
command line makes (`set ... quantity ... --line`), through the same check: a name
that is not a figure of the thing is refused with the figures it has. Entered in the
window it is Assumed, "entered on screen", as a price typed there is. A line made of
a material by a coverage keeps its coverage; this changes the quantity it covers.

### Call 126. An allowance that stops being carried is said, to the cent; Undo brings the roof back to the window; an item nobody has set down is not on the field list (2026-11-17; the twenty-seventh judgment, J27A-28, -22, -27)

- **J27A-28 decided item 12 against the program.** A floor carried a $6,412.67
  allowance. Add cost line on it, the line left unpriced: the allowance stopped
  being carried (a thing with cost lines is priced by them, call 114), the price
  fell by it and by its markup, and the screen said only "Added ... with no price
  yet". The rule stands; the silence does not. Whatever the change (a cost line
  added, an assembly given), where a thing's allowance stops being carried the
  message now ends: **"Floor 1's allowance of $6,412.67 is no longer carried in the
  price: it is priced by its cost lines now."** In amber. One place does it for
  every change made in the window.
- **J27A-22.** A roof deleted and brought back with Undo was in the job and priced
  and was not drawn: the window lets go of a roof that is gone and did not take it
  up again. After Undo or Redo it looks for the job's roof again.
- **J27A-27.** The field list asked a carpenter to measure a sofa, a chair and a bed
  that are in the item library and were never set down. An item is a thing that can
  be placed, not a thing in the building: it is on the list only through what is
  set down from it.

### Call 127. The far ground fades into the sky without a line (2026-11-17; J27B-1, VISUAL.md C2)

Every 3D view had a hard edge across it about an eighth of the way down: pale sky
above, a band of deeper blue below, fading down to the ground. The judge of the look
named it second of the ten things that would change the owner's mind, and it is the
first thing seen. Measured on the screen: sky at the horizon (184, 196, 211); the far
ground's haze (171, 196, 222). The haze had been given the colour the sky's dome is
given at its foot, and the two do not come to the screen alike (why is not found:
moving the sky's lift from the dome's corner colours to its material changed
nothing, and was put back).

The haze's colour is now set by measuring the screen: tried at five colours, and set
between the two nearest (`HAZE` in `scene.rs`). With it the haze reads (188, 195, 207)
under a sky of (184, 196, 211), and there is no line. `PLUMBLINE_FOG="r,g,b"` tries
another, for whoever sets it by eye on a real screen. It is a matched pair of
numbers, not a cause understood: if the sky's colours are changed, the haze is
measured again.

### Call 128. Patrick, 2026-11-17: "It's approved for ply import" (PROPOSALS.md 32, the first piece)

His words, after the survey of `research/lidar-renovation.md`: "It's approved for ply
import". Read as: the point cloud of a scan, as a PLY file, brought in as something
to trace over at true size. Not read as approving what he did not name: existing,
demolish and new (PROPOSALS.md 33), walls found without tracing, other formats, or a
reading off a scan making a value Known.

How it is built, and why:

- **Our own reader.** PLY is a short public format (a header in words, then the
  points as text or as bytes). Reading it ourselves adds no code under anyone
  else's licence. Text, little-endian and big-endian files are read; of each point
  its x, y and z, whatever else it carries (colour, normals) is not kept, and that
  loss is said.
- **Units and which way is up are not in the file, so they are said, not guessed.**
  A phone scan is in metres. The file is taken as metres with z up unless told
  otherwise (`--units ft`, `--up y`), and what was taken is said in the message and
  kept in the drawing's source. Everything from a scan is **Assumed**.
- **The scan becomes a sheet, through the path a sheet already takes.** The floor
  and the ceiling are found as the two heights where most points lie; the points
  within 2 in of 4 ft above the floor (`--cut` says another height) are drawn from
  overhead, north up, at 24 dots to the foot, as a picture written beside the job;
  that picture is added under the level as a drawing, already to scale and in the
  scan's own place. So tracing, snapping where there are lines, the Assumed mark and
  its source all work as they do for a plan, in the window and at the command line,
  with nothing new to learn. A picture of points has no drawn lines, so the Wall
  tool is placed by eye on it.
- **The level's height is said, not set.** "Floor to ceiling reads 8' 0" on the
  scan": the person gives it to the level with Measure if they take it.

Owed after this first piece, in order: the scan seen in 3D as a ghost under the
model; two taped lengths checking the scan; a length picked off the scan; bringing
it in from the window's Bring in list (today: the command line makes the sheet, and
the window traces it).

The same night, after Patrick asked whether the piping could be tried without a scan
of his ("I'm in the field anyway"): a **rough made-up scan** (`tools/made-up-scan.py
--rough`: two rooms, scattered points each about 1.5 cm out, furniture, one point in
fifty a stray outside, the whole scan leaning half a degree) was put through the
same pipe, and broke it in two places a phone's scan would. Three things changed:

- **The scan is levelled by its own floor**, and how far it leaned is said. A cut
  through a leaning scan is not a plan, and a leaning floor and ceiling read the
  room 3 in too tall.
- **The floor and the ceiling are the fullest inch of their heights**, not the
  lowest and highest inch that is full.
- **Strays are left off the sheet and counted**: a point alone in its square foot of
  plan at the cut is not the building. The sheet is fitted to what is left, and the
  size said is of that, "about", to the inch. (With strays in, a house 30 ft by 20
  ft was said to be 69 ft by 59 ft.)

It is a made-up scan built to be awkward; it is not a phone's scan, and stands in
for one only until one is tried.

**Call 127, corrected (2026-11-18).** "There is no line" was the builder's reading of
one crop. A worker reading the next full run's pictures measured twelve 3D views and
found the edge still there in each, fainter: about 4 parts in 255 across two rows
where it had been 13, and visible as a line when a picture is enlarged. So: **much
reduced, not gone.** Two nearer colours were then tried on the same frozen build
with `PLUMBLINE_FOG`: (0.9025, 1.033, 1.26) gives haze (188, 196, 208) and (0.895,
1.045, 1.29) gives (187, 196, 209) under a sky of (184, 196, 211). The constant was
not changed again: every picture in `demos/captured/` would be of another build
than the program, and the gain is two parts in 255. The next change to it goes with
a full run; and the cause (why the haze and the sky's foot, given one colour, come
to the screen as two) is still not found, which is the real repair.

### Call 129. Patrick, 2026-11-18: the program stays in this repository; the scope is every trade and the design professions; a second builder, Opus 5.5, is approved for named work

After a review of this branch (published to Patrick as "BuildOS Branch Review"), his words:

> "This needs to be a software where all 16 trades can use it along with engineer,
> designer, architect and designer. BuildOS wins in engine however learned specifics
> can be ported from roof nerd."

> "Make a note in existing repository we stay here for now you are approved for the
> following research and execution of the program database trade catalogs and
> assemblies, fixes from this review, IFC quantity and property sets, the
> implementation tweaks so program is useable and anything else that's prudent while
> knowing you co own codebase but fable will override those changes if made without
> deep thought. Start with one trade like foundation work to show me you understand"

How each clause is read:

- **"We stay here for now."** BuildOS stays in this repository, in `plumbline/`, on
  branches of it. A separate repository was discussed and is not made now. (The
  review's suggestion to move it out without the 2.3 GB of pictures in history waits
  on a later word from him.)
- **"All 16 trades … along with engineer, designer, architect."** The sixteen
  divisions stay the frame (call 57), and the people who use the job now include the
  design side: an architect or designer hands work in and takes it back (IFC both
  ways), and an engineer's numbers (loads, sizes) are carried as figures with a
  source. A calculation the program shows is for estimating and checking; it is never
  a stamped design, and where the program shows one it says so.
- **"BuildOS wins in engine; learned specifics can be ported from roofnerd."** This
  engine is the one built on. What roofnerd learned (its tapered layout, crickets,
  coping stretch-out, walls and curbs with their flashing heights, fastening by wind
  zone, and the rules in its DECISIONS.md) is ported in as Division 7's depth, from
  its source in `src-tauri/core`, with the port listed the way `PORTMAP.md` lists
  BIMCraft's. Q11's "share one core crate" is answered by this: one engine, this one.
- **Who builds.** A second builder, Opus 5.5, co-owns the code with the builder of
  record (Fable 5.2) and is approved for: research; the item and assembly catalogs of
  every trade (the program's library); the fixes the review found; IFC quantity and
  property sets; the changes that make the program usable by a person; and other
  prudent work. "Fable will override those changes if made without deep thought" is
  read as a bar, not a threat: every change by the second builder is made by this
  file's rules (the demonstration written first with figures worked by hand, the
  decision written before an expectation moves, judges on another model, pictures
  read), and says in this file why, so that the other builder can weigh it rather
  than redo it. A change that one builder makes to the other's call is written here
  as a new call that names the old one.
- **Where it is pushed.** The second builder's session may push only to its own
  branch, `claude/hopeful-cannon-kng8eo`, which starts at this branch's head
  (289ff27) and adds commits on top, so `claude/nifty-tesla-qmn44n` can be brought
  up to it by a fast-forward. Neither builder rewrites the other's history.
- **"Start with one trade, like foundation work."** The first trade taken to its
  finish line by the second builder is foundations: footings, foundation walls and
  the slab on grade, Divisions 2, 3 and 7 where they meet the foundation. Its finish
  line, its sources and its demonstration are written in STATUS.md before it is
  built.

### Call 130. A formula can come out in cubic yards; a wall and a floor say how thick they are, and a floor its perimeter (2026-11-18; the second builder, for foundation work, call 129)

Three changes to what a number can be made of. Each is made because foundation work
cannot be taken off without it, and each is checked against a figure worked by hand.

- **A formula whose lengths multiply to a volume comes out in CY.** Until now a line's
  formula that came out as feet × feet × feet was refused ("a line is a count, a length
  or an area"). Concrete, gravel, fill and excavation are bought by the cubic yard, and
  the only way to get one was a hidden order factor (`81 SF/CY` on the slab, which is
  right only while the slab is 4 in). Now the formula is worked in feet as before, and a
  result in cubic feet is divided by 27 and said so in the worked formula
  (`face_area × thickness ÷ 27`). Every other dimension is refused as before. A figure
  that is already in CY (a footing's `volume`) still stands alone on its line, with its
  waste on the line; using it inside a formula is refused, as it was, and the refusal
  now says a formula can also come out as a volume. A volume formula that divides by
  27 itself is refused and told to take the ÷ 27 out: typed out of an estimator's habit
  it would divide twice (`plan_length * width * depth / 27` on a 2.0576 CY footing read
  0.0762 CY; found by the judge of 2026-11-18).
  **Expectation moved:** the formula test's `plan_length * height * height` on a
  parapet, refused before, now comes out in CY (`crates/world/tests/world.rs`, the
  formula test). Nothing else in the suite used a volume.
- **A wall has a `thickness`**, its type's thickness (ft), as sure as the type and the
  wall's choice of it. A foundation wall's concrete is `net_area × thickness`, in CY,
  worked in the core like any other figure, not typed beside it.
- **A floor has a `thickness`** (its type's) **and a `perimeter`** (the length round
  its boundary on the plan, LF): a slab's edge form is its perimeter, its concrete
  `plan_area × thickness`.

The new figures are added at the end of each kind's list, so every figure that was there
keeps its place. Where a message lists what a wall or a floor has, the list grows by
these names; demonstrations that quote the list whole are moved to the new list in the
same commit (`demos/quantityw.sh`).

### Call 131. Each thing in the IFC model carries its base quantities, from the figures the estimate is priced on (2026-11-18; the second builder; the review's IFC finding)

A designer's or another trade's program opening our IFC file found no quantities in it:
0 `Qto_*` sets, every value as text in one set. IFC4 has a standard set of base
quantities for each kind of thing (the names are IFC4's own, read from the property and
quantity set templates IfcOpenShell ships), and a program that reads IFC reads those.

- **What is written:** for each footing, wall, floor, roof plane, door, window and room
  that goes into the model, an `IfcElementQuantity` named for its IFC4 set
  (`Qto_FootingBaseQuantities` and the rest), holding the figures this program has
  already worked out for it, the same ones its cost lines are priced from: a footing's
  length, width, depth and volume; a wall's length, thickness, height, face area and
  net area; a floor's thickness, perimeter and area; a roof plane's area on its slope;
  a door's or a window's width, height and area; a room's floor area, gross area and
  perimeter.
- **Nothing is worked out in the writer.** A figure the program does not keep (a
  wall's volume, a door's perimeter) is not computed there to fill the set: one
  arithmetic for one number. A figure with no value is left out, never written as 0.
- **How sure each one is goes with it.** Each quantity's description is Known, Assumed
  or Unknown, and its formula is the formula the program worked it by, so a designer
  sees that a wall's height was Assumed before relying on it.
- **Units.** The model's units are feet, square feet and cubic feet (as before); a
  footing's volume, kept in cubic yards, is written in cubic feet.
- **Reading our own file back** still reads the shapes and the values it read before;
  the quantity sets are counted with the property sets as not read, as an element
  quantity was before this.

### Call 132. The starter library takes off a foundation: a strip footing, a poured foundation wall, and a foundation drain (2026-11-18; the second builder; call 129's first trade)

The starter library had no assembly that goes on a footing or on a foundation wall:
the made-up whole house priced its footings by one typed line, and a person drawing a
foundation had nothing to give it. Three assemblies are added, at the end of the file,
with no price in them, every rule that sets a figure written on the line as Assumed
with its source (the jurisdiction's own code decides, not this program):

- **Strip footing, formed, two #4 bars** (goes on a footing): trench for the footing
  (LF); footing forms, both sides (`2 × plan_length × depth`, SF of contact); footing
  concrete (`volume`, CY, 5 % over, "NRMCA CIP 31: order 4% to 10% over the plan
  quantity"); footing bars, #4, continuous (`bars × plan_length` with bars = 2, 12.5 %
  for 30 in laps on 20 ft bars, IRC Table R608.5.4(1); ordered by the lb at 0.668 lb/LF,
  ASTM A615); placing labor (hr, no rate).
- **Foundation wall, poured concrete** (goes on a wall): forms, both faces (`2 ×
  face_area`); concrete (`net_area × thickness`, CY, call 130, 5 % over); horizontal
  bars, #4 (`rows × plan_length`, rows = 2 by IRC Table R404.1.2(1) for a wall 8 ft or
  less, 12.5 % laps); vertical bars, #4 (`(plan_length ÷ spacing + 1) × height`,
  spacing = 48 in, the most Tables R404.1.2(2) to (8) allow); anchor bolts, 1/2 in
  (`(plan_length − 2 × end) ÷ spacing + 1`, end = 12 in, spacing = 6 ft, IRC 2021
  R403.1.6; a count, rounded up); dampproofing, outside face (`plan_length ×
  below_grade`, with below_grade **Unknown**: the height from the top of the footing to
  finished grade, IRC R406.1, is not on a plan, so the line has no figure until it is
  given, and is never a guess); forming and placing labor (hr, no rate).
- **Basement wall, poured concrete, with a drain** (goes on a wall): the foundation
  wall's lines, and a foundation drain, perforated pipe in gravel, by the LF. The code
  asks for a drain only where the wall retains earth and encloses habitable or usable
  space below grade (IRC R405.1), which is the trade's line between a basement wall and
  a crawl space's: so it is a wall of its own, not a line carried on every foundation
  wall. (It was first a one-line assembly to add beside the wall's; a thing carries one
  assembly, and a line can be switched off only if it is labor, so neither way held.)
  Changing a wall from one to the other with `--change` keeps what the two share, the
  height to grade and any price included.

**Expectations moved, with this call written first:** the starter library is 33
assemblies and 131 lines, 21 of them labor (was 30, 111 and 18): `crates/cli/tests/cli.rs` (the starter
library test, which now reads a line's division from its 27th cell, since figures
entered on a line follow it), `demos/library.sh`, and `demos/pricew.sh` (the assemblies
in the saved job). The file's heading gains the
"Figure 1" and "Figure 2" columns the program writes when a line carries figures of
its own, so the file still goes out byte for byte as it came in.

What a foundation still lacks is listed in STATUS.md ("Foundation work"): pad footings,
stepped footings, corner bars, laps counted splice by splice, excavation by the CY
(the model has no grade), the slab's assembly by its own thickness.

### Call 133. A formula works from a figure as it was worked out, not as it is written (2026-11-18; the second builder; found taking off the foundation)

Every figure the program works out is written to four decimals (value.rs, `round4`),
and a formula on a line took its figures as written. A footing 10 in deep is written
0.8333 ft, so its forms, `2 × plan_length × depth` on a 40 ft footing, came to 66.664
SF; by hand, and by the shape, they are 66.6667. The error grows with every figure a
formula multiplies, and an estimator checking the line against the footing's 10 in sees
it wrong.

A derived figure now keeps the value it was worked to, in memory only, beside the one
it is written as (`Derived::exact`), and a formula works from that. Nothing about what
is written changes: the saved job, every screen and every file still carry four
decimals, and the written figure is what two figures are compared by. A job read from
its file works its figures out again before any formula is worked, as it always has.
How a quantity is rounded before it is priced is untouched (Q29 is Patrick's).

The same was so inside the program's own arithmetic in one place found: a footing's
volume was worked from its width and depth as written, so a 40 ft footing 20 in by 10 in
was 40 × 1.6667 × 0.8333 = 55.5544 CF where it is 55.5556 (seen in the IFC file, which
writes the volume in cubic feet; in CY both round to 2.0576). It is worked from the
figures as worked now. Other figures worked from other written figures may be found the
same way; each is fixed where it is found, and said here.

No expectation in the suite moved with it: every figure a test or demonstration checks
was already the same to four places either way, apart from the new foundation's.

### Call 134. A roof has figures of its own, worked from its planes and lines, so a roof is priced as a roofer prices it: whole (2026-11-18; the second builder; roofing, call 129)

A roofer takes off a shingle roof once, for the whole roof: its squares with a waste
for how cut up it is, then its eaves, rakes, hips, ridges and valleys in LF. This program
could only price a roof plane by plane, so every plane rounded its own sheets and
squares (ten 20 LF copings bought 20 sheets where 12 would do; four planes bought their
own sheathing), and the edges had no line to go on but each line's own. The roof's
panel already showed the roof's surface, squares and lengths (`view::roof_totals`), but
no cost line could use them.

- **A roof has eight figures**, worked in the derive pass from what is under it:
  `surface_area` (SF) and `squares` (SQ), the surface of its planes added up; and
  `eave_length`, `rake_length`, `hip_length`, `ridge_length`, `valley_length` and
  `headwall_length` (LF), the true lengths of its lines of each kind added up. A
  plane's surface is worked by the one function the plane's own figure comes from
  (`plane_measures`), and a line's length is the length the line itself has: one
  arithmetic for each number.
- **A sum is only as good as its members.** A roof with a plane that has no surface has
  no surface, and says which plane; a line with no length leaves its kind's sum with
  none, and says which line. A roof with no line of a kind (no valleys on a hip roof)
  has 0 of that length, as sure as its planes, its formula saying "the roof has no
  valley line": a quantity of 0 is said to come to nothing, never left blank.
- **The figures are the roof's own and show where its figures are shown** (`show`, the
  panel's takeoff); nothing about a plane's figures changes, and a roof is still not a
  thing bought "by" one figure (its allowance stays on its planes).
- **A change to a plane or a line now touches its roof too**, since the roof's figures
  are worked from them: the two tests that list what one change touches
  (`changing_one_pitch_touches_only_that_plane_and_what_is_derived_from_it`,
  `planes_that_disagree_about_a_hip_give_no_length`) and the two demonstrations that
  check it (`demos/d1.sh`, `demos/d4.sh`) list the roof too. The roof's
  planes and lines are taken in the order of their paths, so a job read back from its
  file works the same roof figures, inputs and all (taken in the world's own order
  first, three tests found the reloaded roof stale). The panel's roof totals are worked
  from the same exact figures as the roof's own (they were the sum of the planes'
  written figures, 1,341.6406 SF on a 40 by 30 hip roof that is 1,341.6408).

### Call 135. The starter library prices a shingle roof whole: an assembly that goes on the roof (2026-11-18; the second builder; the roofers' judgment of the review)

Both roofers on the review's panel stopped at the same lines: shingles in 3.8025 SQ,
plane by plane, no waste, no bundles; no starter, drip edge, ice barrier or cap though the
eaves, hips and ridges were measured. The plane assembly stays as it was (a job built on
it keeps it), and **"Asphalt shingle roof, whole roof"** is added, going on a roof
(call 134). Every figure that is a rule or a product's packing is on its line as
Assumed, with its source (read from the 2021 IRC as reproduced by UpCodes, and from the
makers' published product data; the jurisdiction's code and the product bought decide):

- Roof sheathing, `surface_area`, 10 % for cuts (the estimator's allowance; no standard
  sets it), in whole 4 by 8 sheets.
- Underlayment, synthetic, `squares`, 10 % for its 2 in side and 4 in end laps (IRC
  R905.1.1, one layer at 4:12 and steeper; two below 4:12, which this line does not
  double), in whole 10 SQ rolls.
- Ice barrier, at the eaves and in the valleys, `eave_length × eave_width +
  valley_length × valley_width`, each width 3 ft (one 36 in course; R905.1.2 asks it to
  reach at least 24 in inside the exterior wall line where ice damming is known: check
  it against the overhang), in whole 200 SF rolls. One line, so one roll count for both.
- Field shingles, `squares`, 15 % waste (the trade's published ranges: about 10 % on a
  gable, 12 to 15 % on a hip, 15 to 20 % cut up; no standard sets it), in whole bundles
  at 3 bundles to the square (two makers' architectural shingles).
- Starter strip, `eave_length + rake_length`, 5 %, in whole bundles of 105 LF (one
  maker's; another's is 120).
- Hip and ridge cap, `hip_length + ridge_length`, 5 %, in whole bundles of 20 LF (the low
  end of 20 to 35 by product: the roofer buys over, not under).
- Drip edge, `eave_length + rake_length` (R905.2.8.5: at eaves and rakes, lapped 2 in), in
  whole 10 ft pieces less the 2 in lap: 9.8333 LF to the piece (the 10 ft piece is the
  trade's; not read in a standard).
- Ridge vent, `ridge_length`; open valley flashing, `valley_length` (R905.2.8.2, at least
  24 in wide); headwall flashing, `headwall_length`.
- Pipe boots, `boots`, **Unknown**: penetrations are not drawn, so the line has no
  figure until the count is given.
- Labor: sheathe the roof (`surface_area`) and lay the roof (`squares`), in hours with no
  production rate, as every starter labor line.

**Expectations moved, this call written first:** the starter library is 34 assemblies
and 144 lines, 23 of them labor (was 33, 131 and 21): `crates/cli/tests/cli.rs`,
`demos/library.sh`, `demos/pricew.sh`, `demos/foundation.sh`.

**Not yet:** a second layer of underlayment below 4:12; step flashing at sidewalls (the
model has headwalls, not sidewalls); tear-off and disposal (a reroofing assembly);
standing seam priced whole; nails.

### Call 136. A roof priced whole and plane by plane is said to be counted twice (2026-11-18; the second builder; the judge of the whole roof)

A judge put the plane-by-plane "Asphalt shingle roof" on a plane and "Asphalt shingle
roof, whole roof" on its roof: the estimate carried the plane's sheathing and shingles
and the roof's, without a word. The two assemblies name the same shingles differently
("Asphalt shingles", "Field shingles"), so a check by the line's name would miss most of
it. What is said is the pricing both ways: wherever a roof carries a cost line that is
switched on and one of its planes does too, the estimate says, under its price, at the
command line and on the sheet's Price row, in red: "counted twice: Main roof is priced
whole, and plane 1 is priced on its own as well: what both carry (its sheathing, its
shingles) is in the price twice; take one of the two off". It is said, not refused: a
roof can carry a line of its own beside its planes' (a tear-off by the roof, shingles
by the plane), and only the estimator knows which; the sentence says how to clear it.
(`view::Pricing::twice`, `view::twice_said`; checked in `demos/shingle.sh`.)

### Call 137. A wall's frame is counted, and the starter walls buy their lumber by the piece and the foot (2026-11-18; the second builder; the framer's judgment of the review)

The framer on the review's panel: "studs, headers and jacks are drawn, not counted;
framing is priced by the SF". The frame was laid out only in the window, to draw it
(`scene.rs`, from `building::framing`, BIMCraft's layout). It is now laid out in the
core too, by the same function from the same line, and counted.

- **A wall whose priced line names its framing** (the first line, material or assembly
  that names a lumber size and a spacing and is in the work: "Studs, 2x6 at 16 in", as
  its dress has read it since call 71; `finish::framing_of`, now the one rule both read)
  **has four more figures**: `studs` (EA: every stud, king, jack and cripple, one stud
  each), `plate_length` (LF: the bottom plate and two top plates), `header_length` (LF:
  a header over each opening, jack to jack) and `sill_length` (LF: a rough sill under
  each window). A wall nothing names a framing for has none of the four: a wall framed
  by nobody is not a wall with no studs. A wall whose openings are not placed has them
  refused, and says which opening.
- **A count of pieces is a count**: a line priced by `studs` with a waste takes it in
  whole studs (a figure in EA standing alone was taken as it stood).
- **The starter library's five framed walls** (four exterior 2x6 walls and the 2x4
  partition) carry, in place of the one material line priced by the SF ("Wall framing,
  2x6 at 16 in", `face_area`): "Studs, 2x6 at 16 in" (`studs`, 10 % for culls, corners
  and blocking), "Plates, 2x6" (`plate_length`, 10 %, in whole 16 ft pieces), "Headers,
  two plies" (`2 * header_length`, LF) and "Rough sills, 2x6" (`sill_length`, LF); each
  allowance Assumed, said to be the estimator's. A header's size from its span is a rule
  of thumb (`framing::header_depth`: a 2x6 to 4 ft, a 2x8 to 6 ft, a 2x10 to 8 ft, a 2x12
  beyond); the code sizes it from its load (IRC R602.7). The labor stays by the SF, as a
  framing sub quotes it.

**Expectations moved, with this call written first:** the starter library is 34
assemblies and 159 lines (was 144): `crates/cli/tests/cli.rs` (the starter library; an
assembly changed for another, which priced the framing line by the SF; a partition's labor, whose assembly now gains seven lines, not four), `demos/library.sh`,
`demos/pricew.sh` (the brick wall's lines). The made-up whole house (`sample.rs`) keeps
its "Wall framing, 2x6 at 16 in" by the SF: it is a job built before this, and a job
keeps what it was priced with.

**Judged the same day** (an agent on another model; STATUS.md has its table): an
opening that does not fit its wall (past an end, over another) was framed as if cut to
fit, and a window whose head is in the top plates got a header of 0 LF, both without a
word. Now the four figures are refused for such a wall, with the reason the wall's net
area already gave ("window-0007 starts 9.00 ft before the start of its wall"; "the head
of window-0007 is in the top plates: there is no room for its header"). And the stud
count is **Assumed** even on a Known wall: it rests on the spacing the line names and
leaves a corner's and an intersection's extra studs to the allowance, and its formula
says so.

**Not yet:** corners and channels counted as studs of their own (a corner takes three);
blocking and fire blocking; double top plates lapped at the corners (here two plates
the wall's length); headers sized from the load; floor joists, rafters and trusses.

**Missed when this was made, found by the window run of 2026-11-19:** a wall's list of
figures has the four framing figures in it, so a refusal that names what a wall has
names them too ("a wall has plan length, height, face area, opening area, net area,
thickness, studs, plate length, header length, sill length"). `demos/quantityw.sh`
checks that sentence word for word and was not moved with this call; it failed on the
frozen copy of this build. The program is right; the demonstration's sentence moves.

### Call 138. roofnerd's core comes across whole, as a crate, its tests with it (2026-11-19; the second builder; call 129's "learned specifics … ported from roof nerd", step 1 of `research/roofnerd-port.md`)

roofnerd's solvers (the tapered package laid out the NRCA way, crickets, sumps, the board
bill, walls and curbs with their flashing heights, coping, wind zones and fastening by
the maker's table) are about 8,800 lines and 122 tests, each rule sourced. They are not
rewritten: the core is brought into this workspace whole as `crates/roofing`
(`plumbline-roofing`), with the maker's profile and fastening tables and the
demonstration job its tests read, and its 122 tests pass here unchanged but for the
crate's name and the paths to what they read. Two tests still read the earlier
program's TypeScript (`packages/engine/src/roof.ts`, `model.ts`) to hold the two copies
of its conventions together; they read it where it stands in this repository, and if
BuildOS moves to a repository of its own they move with it or are retired with a call.

Nothing in the building model uses it yet. The next steps (`research/roofnerd-port.md`):
a drain placed on a low-slope area and the area's tapered package solved by this crate
from the area's outline and drains, giving figures a line can price (the taper's area,
its average and greatest thickness, the boards by letter); then crickets and sumps, walls
and coping, wind and fastening. Its licence (MIT) is in `NOTICE`; `LICENSES.md` lists it
under borrowed code; its two crates (`cavalier_contours`, `geo`, both MIT OR Apache-2.0)
pass `tools/licenses.sh`.

### Call 139. A low-slope area holds its drains, and its tapered insulation is laid out by roofnerd's solver into figures a line can price (2026-11-19; the second builder; step 1 of `research/roofnerd-port.md`)

**What is held.** A low-slope area gains one record, `pl::taper`: `drains`, the places
of its roof drains on the plan (ft), and `at_drain`, how thick the tapered insulation is
at a drain (in), each with its status and source. Nothing else is typed. The slope the
insulation makes is the area's own pitch: with drains on it, the deck is taken as level
and the pitch is the finished slope the taper builds (a quarter inch per foot is
`1/4:12`), so one number is not held twice. A job without the record is as it was; the
record is refused on anything but a low-slope area, and a drain that is not on the
area's outline is refused with where it is.

**What is worked out**, by roofnerd's `taper::solve` (call 138) and nothing else, on the
layout NRCA gives (*Membrane Roof Systems* 2019, ch. 4, pp. 163–168: one region to each
drain, cut by the line halfway between drains; four planes to an interior drain at the
one slope, laid square to the area's longest side; the valleys where two planes meet, at
45°). Six figures on the area, each with its formula and its inputs (the outline, the
pitch, the taper record), at the worst status of those:

| Figure | Unit | What it is |
|---|---|---|
| `drains` | EA | the drains on the area |
| `thinnest` | in | the thickness at the drain, where the water leaves |
| `thickest` | in | at the furthest run from a drain, measured square along the boards (not the diagonal) |
| `average_thickness` | in | the insulation's volume over the area's plan area |
| `board_feet` | BF | the insulation's volume, a board foot being a square foot an inch thick: each face of the layout's area times its thickness at its middle (a face is one plane, so that is exact), added up |
| `valley_length` | LF | the 45° lines the boards are mitred along |

The solver cuts its regions with `geo`'s boolean operations, which work on a fixed grid
and leave a corner up to half a millionth of a foot from where it belongs: two drains on
a 48 ft roof first came to 8,640.0001 BF, not the 8,640 worked by hand. The faces and
valleys are read to a hundred-thousandth of a foot before they are added up; nothing a
roofer cuts is finer, and the figure is then the layout's own.

Tapered insulation is quoted by the board foot; a line priced by `board_feet` alone is
in BF. The thicknesses are in inches and are not used in a formula (a formula works in
lengths, areas and counts and says so); a line uses `board_feet` for the insulation and
`drains` for the drains.

**Where it is not yet whole, said on the figure, not left silent.** Nothing caps the
build-up: roofnerd caps it at the flashing height along a parapet (8 in under the top of
the base flashing, NRCA pp. 159, 251–252) or at a number of layers, and neither is joined
to the building model yet (step 4 of the port). So `thickest` is the build-up as the
slope makes it, and the formula says nothing caps it. Crickets, sumps, the board bill by
letter, and drawing a drain in the window are the later steps. A drain on a low-slope
area is not a plumbing fixture here: the storm piping it feeds is not drawn from it.

**Two command-line verbs**, so a demonstration starts from an empty job: `low-slope`
draws a low-slope area under a roof of its own (its outline, its pitch, how high its
deck is), and `drains` places the drains on one and says how thick the taper is at them.

**A starter assembly**, "Low-slope roof, tapered insulation, single-ply membrane" (goes on
a low-slope area): the tapered polyiso by `board_feet` (5 % for the cuts, Assumed, until
the boards are laid out by letter); a 1/2 in cover board by `surface_area` in 4 ft by 8 ft
sheets (5 %); a 60 mil TPO membrane by `surface_area` in 1,000 SF rolls (10 % for laps
and turn-ups; the roll size as GAF's, IKO's and Johns Manville's 60 mil TPO data sheets
give it, 10 ft by 100 ft); the drains set, by `drains`, Division 15, subcontract; and the
labor by `squares`, with no production rate. Each rule is Assumed with its reason; no
prices. The library is then 35 assemblies and 164 lines, 24 of them labor (was 34, 159 and 23), and the
expectations that count it move with it: `crates/cli/tests/cli.rs` (the starter library
set; the same library twice), `demos/library.sh`, `demos/foundation.sh`,
`demos/framing.sh`, `demos/shingle.sh`.

**Found when the commit was checked:** `tools/committed.sh` lays out only the
`plumbline` folder, and three of roofnerd's ported tests read `packages/engine/src/roof.ts`
and `model.ts` beside it (call 138), so the call 138 commit already failed it. The check
now lays those two files out from the commit too, where they stand; it checks nothing
less, and a test that reads anything else outside the folder still fails it.

**What moved.** No existing number but the library's size above. Every saved job gains one line in its list of record
kinds (`schemas`), as each record before it did. A low-slope area's list of figures has
six more; on an area with no drains each is refused with "no drains placed on it".

### Call 140. A taper's layout does not depend on the order its drains are typed in; what it rests on is said where a person reads it (2026-11-19; the second builder; the judge of call 139)

The judge of call 139 found the tapered insulation's board feet changing with the order
the drains were typed: eight drains on a 140 by 80 field came to 50,587.5, 49,469.5 or
48,682 BF by order, against 50,700 worked by hand, with nothing flagged. The cause was
in the ported solver's geometry: it cuts each drain's region, and each region's faces,
with `geo`'s boolean intersection, which works on a fixed grid. An earlier cut left a
region's corner as two corners a millionth of a foot apart, and the next cut, along the
45° line through that corner, gave a face its point three feet from its drain; on a grid
of drains a whole face (225 of a region's 900 SF) went. Which drain was cut first decided
whether it happened.

**What changed, in `crates/roofing/src/geom.rs`:** a convex shape is now cut by a half
plane exactly, side by side (every region of a convex roof, every face); `geo` is kept
only for a shape that is not convex, which it alone splits into pieces; and every shape a
`geo` cut gives back has its corners read to a millionth of a foot, corners that then
coincide made one. roofnerd's 122 tests pass unchanged.

**And in `crates/world/src/taper.rs`:** the faces are added up against their region and
the regions against the area; ground unaccounted for (more than 0.01 SF) refuses all six
figures with "the taper's layout lost ground: …", never a short price. Board feet are
given to a hundredth of a board foot, and the formula says so: the faces are cut in
floating point, and two orders of thirty-six drains came 0.005 BF apart (half a cent at
a dollar a board foot); written to four places, the figure would claim what the cut does
not give.

**What a person now reads.** `thickest`, `average_thickness` and `board_feet` are at least
Assumed, with the note "nothing caps the build-up yet: it rises at the slope to the far
edge, and the flashing height along a parapet or curb, which caps it on the roof, is not
joined to it", because that is what they rest on (the judge: a 100 ft strip showed 13.5 in
with no word). A drain typed twice (two within an inch) is refused: it had been counted,
priced and laid out twice. `drains` given again says what it replaced ("2 drains in place
of the 1 drain it had") and echoes the thickness as read, not as typed ("-0 in" was echoed).

**Measured, and left:** the solve's time grows with the area, because the solver lays its
board grid out on 4 ft squares (the bill is the next step): 250,000 SF 0.3 s, 1,000,000
SF 1.1 s, 4,000,000 SF 3.6 s on this machine. The judge's 100,000,000 SF hung; no roof
area is that size.

**Expectations moved:** none of call 139's figures. The new test
`a_taper_does_not_hang_on_the_order_its_drains_are_typed_in` holds the eight drains in six
orders and a 6 by 6 grid in four to one answer.

### Call 141. A roof priced whole dresses its planes (2026-11-19; the second builder; the pictures of the shingle roof's window run)

The window run of call 135's demonstration was looked at: the roof was priced whole as
asphalt shingles, 13 lines on the roof, and its four planes were drawn bare gray. The
house is dressed in what the estimate prices (Patrick's fixed rules), and a plane read
only its own lines. Now a plane, or a low-slope area, with no roofing of its own is
dressed in what its roof is priced whole with, and `finishes` says so: "asphalt
shingles (the roof's line \"Field shingles\")". A plane that has roofing priced on its
own keeps it; a plane with neither stays bare and says so. No number moves.

### Call 142. What a taper rests on is said where its money is; the field list asks about the drains; a roof is laid out the same however it is traced (2026-11-19; the second builder; the second judge of the taper)

The second judge, about 400 jobs against its own exact cut, found the order fix of call
140 holding and these, each now changed:

- **A caution travels with its figure.** A figure that has a value and a note (the
  taper's "nothing caps the build-up yet") passes the note to every line priced by it,
  and the line's note says it: "its quantity: nothing caps …". It is read in `estimate`,
  the estimate's CSV and workbook, the PM lens's note, and the supplier and
  subcontractor lenses' notes. It had been in `show` and `vif` only.
- **The field list asks about the drains.** The taper record's two values are listed on
  the area ("drains", "thickness at the drain") and `measure` takes them: the drains are
  marked "same" once seen where they are, the thickness is measured as a length. The
  field list then asks "check: drains; measure: thickness at the drain" with the
  insulation's dollars on them; before, it sent a person to measure the eave height,
  which moves no board foot. A drain moved is placed again with `drains`.
- **A drain on the area's very edge is refused on every side**, with "a roof drain
  stands inside the area; one in the edge is a scupper, which is not drawn yet". It had
  been taken on the west and south edges and refused on the east and north.
- **Drains exactly an inch apart are two drains** (the refusal said "closer than 1 in"
  and refused exactly 1 in).
- **Laid out the same wherever and however the roof is traced.** The area is moved to
  the plan's origin before it is laid out (250,000 ft out it drifted up to 0.25 %), and
  its outline is started at its lowest corner and run counter-clockwise (where two sides
  tie for longest, the corner the outline began at chose the side the boards are laid
  to: 7,029.52 or 7,025.86 BF).

**Left, and said in STATUS.md:** the taper's figures stay Assumed until the build-up is
capped (port step 4); a U-shaped roof with one drain is laid out by the straight run
across its open middle (NRCA puts a cricket or an area divider there: port step 3);
`drains` replaces the whole list.

**No number moves** on any job of calls 139 to 141.

### Call 143. A partition's board and paint are bought for both its faces, and gypsum board carries its waste (2026-11-19; the second builder; Division 9, from the review)

The starter library's "Interior partition, 2x4" had "Gypsum board, both sides" and
"Paint, both sides" by `net_area`, which is one face of the wall: a partition priced
from it bought board and paint for half of it, in SF, with no sheets. Now:

- "Gypsum board, both sides": `2 * net_area`, 10 % waste, in 4 ft by 8 ft sheets
  (32 SF/sheet, whole sheets), as the exterior walls' board already was.
- "Paint, both sides": `2 * net_area`.
- Every "Gypsum board, hung and finished" line (the four exterior walls' inside face):
  10 % waste, Assumed, "cuts at openings, corners and ends: the estimator's allowance;
  no standard sets it". The sheet size stays as it was.

What moves: every job priced from these assemblies (a partition's board and paint
double; an exterior wall's board is ordered with its waste). No job built before this
changes: a job keeps the lines it was priced with. The made-up whole house
(`sample.rs`) keeps its own lines. Tests and demonstrations that count these lines'
quantities are named in STATUS.md as they move; the library keeps 35 assemblies and 164
lines.

**Not yet:** a room's own wall and ceiling board (the room's walls at their faces,
less its openings, and its ceiling), which is how a drywall estimator takes off a
house; joint compound and tape by their own measure; 4 ft by 12 ft sheets.

### Call 144. A room has the area of its walls' board, as a drywall estimator takes it off (2026-11-19; the second builder; Division 9)

A drywall estimator takes a house off room by room: round each room at its walls' faces,
times the height, less the doors and windows, and the ceiling. A room had its floor
area and its perimeter, and the board was priced on walls, by each wall's own net area
on its line, both faces or one.

A room now has `wall_area` (SF): each side of its outline at its walls' faces, as long as
it is, times the height of the wall it lies against, less the part of each door and
window in that wall that lies along the side. A door between two rooms comes off both,
since both faces are cut. Its status is the worst of the walls it is measured to (as the
floor area's is) and of the openings it took off; its inputs cite them. It is refused,
with the reason, where the room has no outline, where a side lies along no wall's face,
or where an opening's place in its wall is not known. The ceiling is the floor area,
which rooms already have, so there is no second figure for it.

This is a figure, not an assembly: the wall assemblies already put board on each wall,
and a room priced by `wall_area` as well would buy the same board twice. Which way a job
is priced is the estimator's; the library is not changed by this call. No number moves.

### Call 145. A slab on grade is bought by its own thickness (2026-11-19; the second builder; Division 3)

The starter library's "Slab on grade, 4 in" buys its concrete and its base at 81 SF to
the CY, which is 4 in, whatever the floor's type says: a 6 in slab drawn as 6 in was
bought as 4 in, a third short, without a word. That assembly stays as it is (jobs were
priced with it, and its name says 4 in); a new one, "Slab on grade, by its thickness"
(goes on a floor), works from the floor's own figures (`plan_area`, `perimeter`,
`thickness`, call 130):

| Line | Quantity | Rule, Assumed, with its source |
|---|---|---|
| Excavation and grading | `plan_area`, subcontract, Division 2 | |
| Base course, gravel | `plan_area * base`, CY, Division 2 | `base` = 4 in: 2021 IRC R506.2.2 (a 4 in base course of clean graded sand, gravel or crushed stone; not required on well-drained soils: check it) |
| Vapor retarder, 10 mil | `plan_area`, 10 % waste, Division 7 | 2021 IRC R506.2.3: 10 mil, ASTM E1745 Class A, joints lapped at least 6 in (the 2018 code's 6 mil is not this); the waste is the laps and turn-ups, the estimator's allowance |
| Welded wire reinforcing | `plan_area`, Division 3 | |
| Slab edge forms | `perimeter * thickness`, SF, Division 3 | |
| Slab concrete | `plan_area * thickness`, CY, 5 % waste, Division 3 | NRMCA CIP 31: order 4 % to 10 % over the plan quantity (as the footing's and the wall's, call 132) |
| Place and finish the slab | `plan_area`, subcontract, Division 3 | |

The IRC text was read through a copy of Virginia's adoption of the 2021 IRC and a code
forum quoting R506.2.3, not the ICC's own page (which refuses an automated reader); the
vapor retarder's 10 mil is in both. Each rule is Assumed and says so. The library is
then 36 assemblies and 171 lines, 24 of them labor; what counts it moves with it:
`crates/cli/tests/cli.rs`, `demos/library.sh`, `foundation.sh`, `framing.sh`, `shingle.sh`.
The demonstration's 14.5186 CY (not the 14.5185 worked exactly) is QUESTIONS.md Q29's:
a quantity is written to four places before its waste is added, and that is Patrick's.

### Call 146. A room's wall area takes in what stands inside it; it is refused where an opening does not fit; board priced by the room and by its walls is said to be counted twice (2026-11-19; the second builder; the judge of calls 143 to 145)

The judge found, by hand:

- **A room with a room inside it** (a closet or a chase standing free) counted the inner
  room's faces in its perimeter and left them out of its wall area: 702 SF where 930 SF
  is board, 228 SF short, Known. The wall area now goes round what stands free inside
  the room as well as round the room.
- **An opening that does not fit its wall** (hung off its end) left the wall's net area
  Unknown and the room's wall area Known, clipped to the room's sides. The room's wall
  area is now refused, naming the opening, as the wall's own net area is.
- **Board priced twice.** A room priced by `wall_area` whose walls also carry board lines
  bought the board twice, and nothing said so. Under the price, in `estimate` and in the
  sheet, it is now said, as a roof priced whole and plane by plane is (call 136):
  "counted twice: Kitchen's walls are priced by its wall area, and wall 1 carries board of
  its own: the board on those walls is in the price twice; take one of the two off". A
  wall carries board when a line on it that is switched on names gypsum, drywall,
  sheetrock, wallboard or plasterboard.

**Left, and said in STATUS.md:** a slab-on-grade assembly is taken on a framed upper floor
(its depth then read as concrete); the ceiling's board is still one line with its paint,
in SF; board on several walls is rounded up to whole sheets wall by wall; a slab's edge
forms are the whole perimeter. No number moves but those named above.

### Call 147. A room has the fewest receptacles its walls need, by NEC 210.52(A) (2026-11-19; the second builder; Division 16)

NEC 210.52(A)(1), as *Understanding the 2023 NEC* (Mike Holt) gives it: "The wall space
receptacle outlet must be installed so no point along the floor line of any wall is
more than 6 ft, measured horizontally along the floor line, from a receptacle outlet."
(A)(2): wall space is "any wall space 2 ft or more in width that is unbroken along the
floor line by doorways, fireplaces, stationary appliances, and fixed cabinets without
countertops or work surfaces", measured round corners. (A) covers "every kitchen, family
room, dining room, living room, sunroom, parlor, library, den, bedroom, recreation room,
and similar room or area" of a dwelling unit. The NFPA's own text was not read (its
viewer refuses an automated reader); the line on the figure says where it came from.

A room now has `receptacles` (EA): round the room at its walls' faces, the floor line is
cut at each door into runs of wall; a run of 2 ft or more needs one outlet for every 12
ft of it or part (an outlet serves 6 ft either way), and a shorter run none; a room with
no door is one run all the way round. Islands standing in the room are runs of their
own. It is **Assumed** however sure the walls are, and says why: the fireplaces,
stationary appliances and fixed cabinets that also break wall space are not drawn; it
is for the rooms 210.52(A) covers; a bathroom, a hallway, a foyer and a kitchen's
counters have rules of their own (210.52(D), (H), (I), (C)), not worked here; and
which room a room is, is the estimator's. It is refused where the room's wall area
is.

No number moves; a room lists one more figure.

### Call 148. Existing to remain, to demolish, new: a renovation job (2026-11-21; Patrick's go-ahead on PROPOSALS.md 33: "you own codebase", asked for his own renovation)

**What is held.** Any thing that is built (a wall, a door, a window, a floor, a footing,
a roof, a run, a thing set in place) may carry `pl::work`: `status`, one of the BIM
standard's four words for an element's Status (IFC `PEnum_ElementStatus`): New,
Existing, Demolish, Temporary, with how sure it is and where that came from. A thing
with no mark is new, so a job made before this reads as it did. The four words and
what they mean are BIMCraft's `Status` (`crates/core/src/phasing.rs:40`); its ordered
list of phases, its phase filters and its design options are not taken: one job, one
piece of work, one mark on each thing.

**What the mark does.**
- **Rooms are the rooms the work leaves.** A wall to demolish bounds no room: take a
  partition down and the room is the two it was; take an outside wall down and draw the
  addition, and the room runs into it.
- **The walls it met do not end on it** (found by the demonstration: the room past a
  demolished outside wall read 550.375 SF for 550.25, a quarter-foot pocket left at each
  end where the walls had been trimmed to it).
- **A new wall may stand where a demolished one stood.** "One wall drawn over another"
  is refused as before between two walls that both stand or both come down.
- **What comes down is in nothing's way**: it is left out of the clash test.
- **Existing work is not bought.** A thing marked existing or to demolish that carries
  cost lines is refused until the estimator says which: `--clear` takes the lines off it
  (what stands is not bought again, what comes down is not built) or `--keep` leaves
  them priced, as work done to it. Lines added after are priced as any line is: paint on
  an existing wall, the taking down of a partition by its face area. An existing thing
  with nothing priced on it is not listed as unpriced. (First built by switching the
  lines off; the saved job was then refused on reading, because only a Labor line can be
  switched off, call 23's reading of "labor toggles per line". That rule stands; the
  design changed.)
- **A wall's doors and windows are marked with it.**
- It is listed with the thing ("work: existing to remain"), `measure <thing> work same`
  makes it Known once it has been looked at, and it is saved and read back.

**One verb:** `plumbline mark <file> <path> [<path> ...] <new|existing|demolish|temporary>`.

**Not yet, each owed:** drawn in the window (existing light, demolition dashed) and a
way to mark there; the Status written to and read from the BIM file; a door filled in
(an opening to demolish in a wall that stays still comes off that wall's net area);
demolition assemblies in
the starter library (rates are a sub's); the plan before, the demolition and after as
three views.

**What moved.** No number on a job with no mark. Every saved job's list of record
kinds gains `pl::work`.

### Call 149. The renovation mark in the window: seen, and made with three buttons (2026-11-21; Patrick: "run until we have a working prototype"; call 148's "not yet")

Patrick's scans are Scaniverse's, his desktop is Linux, and the prototype he is owed is:
open the window, lay his scan under the plan, trace the house as it stands, say what
stays and what comes down, draw the addition, and watch the rooms and the price follow.
This call is the part of that the window lacks for call 148.

- **Seen.** Existing work is drawn light (its own colours washed toward pale gray): it
  reads as the house that is there. Work to demolish is drawn as a faint red ghost, in
  3D and on the plan, so what comes down can be seen through. New and temporary work
  is drawn as work is drawn today.
- **A switch under Show, "Demolition":** on, the ghosts are drawn; off, they are not,
  and the window shows the house as the work leaves it.
- **Made.** The panel of a selected thing that is built has a row, WORK, with three
  buttons: "Existing", "To demolish", "New work"; the one that holds is lit. Pressed on
  a thing that carries cost lines (for existing or to demolish), two buttons take the
  row's place: "Take its lines off" and "Keep its lines", with the sentence call 148
  refuses with; either finishes the mark. What the window says after is what
  `editing::mark` says. Marking in the window is Assumed, "entered on screen", as
  everything entered there is, until measured.
- The words are a builder's: existing, to demolish, new work, demolition.

No number moves. `demos/renow.sh` shows it, its pictures measured.

### Call 150. A door or window to demolish is filled in; the mark's edges, from its judge (2026-11-21; the judge of calls 147 and 148)

- **A door or window to demolish in a wall that stays is filled in**, and is no hole in
  that wall when the work is done: the wall's net area, the room's wall area and the
  room's floor line (its receptacles) take it back in. A doorway widened (the old door to
  demolish, a wider one hung where it was) is then the wider door's hole alone: the
  judge's partition reads 144 − 35 = 109 SF, where it had no net area and no reason. The
  filling in itself is priced by a line on the opening that comes out.
- **A wall's doors and windows follow its mark only where they were what the wall was.**
  One with a mark of its own keeps it, and that is said ("door-0011, in it: stays to
  demolish"). A new door hung in a wall before the wall is marked existing cannot be told
  from one that was always there: it follows, and is said, and is marked new again by hand.
- **No new door or window is set in a wall that is to demolish**: refused, in words.
- **Marked new again with nothing priced on it**, a thing that was existing or coming
  down says it carries no cost lines.
- **`apply` on a thing that is existing or coming down** says, where the lines go on,
  that an assembly's lines are for building a thing new.
- **`takeoff` names the walls to demolish** under a level's walls (they are counted as
  they stand); **`clashes` names what it left out** as to demolish.
- **The mark's refusal** names the choice in words ("take its lines off, or keep them
  priced as work done to it"); the command line adds its two flags; the window its two
  buttons. What `mark` says names a thing once, by its path, as other edits do.
- `mark` with its word missing says what it takes, not the whole usage.

Still owed from that judgment: the Status in the BIM file (J-N8); a second room left
Unknown by a new wall on part of a demolished wall's line says why in `check` only
(J-N11).

### Call 151. The tool chest (2026-11-22; Patrick's word)

Patrick: "we have all the trades and tool chest; I imagine that tool chest is a more apt
way to have single items like cabinets, countertops and design selections." No part of
the program carried that name. It is read as the item library (calls 105 to 111: items
by trade, kept as a table, brought in, written out, placed, moved, clash-tested, each
size Assumed and sourced to its maker's sheet), and that reading is said to him, to be
set right if he meant something else. What follows from it: single things a job picks
(cabinet boxes, countertops, fixtures, appliances, finish selections) live in the tool
chest by trade and are not given tools of their own; the renovation plan's step 4 is
built that way. Whether the screen's words "Item library" and "Starter items" become
"Tool chest" is a wording call, made when step 4 is built.

### Call 152. A room line parts two rooms where no wall stands (2026-11-22; plan step A2; the design and MEP reviews: an open kitchen read "floor area unknown")

An open plan has rooms with no wall between them. The program found rooms only among
walls, so a kitchen opened into an addition had no area. BIMCraft has the BIM tools'
answer, a room separation line (`room_separator`, `crates/core/src/commands.rs:466`;
`RoomSeparator` in its model), and it is ported, not invented.

- **What is held.** A new kind of thing, a **room line** (`Kind::RoomLine`, said "room
  line"), under a level, with `pl::room_line`: `line`, its two ends on the plan (ft or
  in), with how sure and where from. Nothing else: no type, no height, no thickness.
- **What it does.** It bounds the rooms of its level as a wall of no thickness would, at
  whatever height the rooms are found. A room on each side is measured to the line.
- **What it is not.** It is not built: it has no figures, takes no cost line, no
  assembly and no work mark, is never counted as a wall (takeoff, lists), is not
  clash-tested, joins and trims no wall, and is not written to the BIM file (the list
  of what was not written says so).
- **In a room's figures:** `floor_area` and `gross_area` run to the line. `perimeter`
  is the distance round the room, the line's side with it (it is the room's size round,
  as it already takes in doorways). `wall_area` gets nothing from a side along a room
  line: no wall is there. `receptacles`: a side along a room line is not wall space and
  cuts the floor line as a doorway does (NEC 210.52(A)(2) counts wall space, and an
  opening is none).
- **One verb:** `plumbline room-line <file> <level> --points "24,0 24,16"` (one line
  for each pair of points that follow one another). In the window it is a tool beside
  Room, built after the model.

No number moves on a job without one. Every saved job's list of record kinds gains
`pl::room_line`.

### Call 153. The room line drawn in the window; the scan sheet made fit to trace (2026-11-22; plan steps A1 and A2, their window halves)

- **A tool, "Room line", beside Room** in the window's BUILDING tools: on the plan, a
  click at each end sets one room line on the level in view (Assumed, entered on
  screen); it snaps to wall ends and faces as the Wall tool does; Esc puts the tool
  down. A room line is drawn on the plan as a thin dashed line in the colour rooms are
  named in, is not drawn in 3D, can be picked, moved and deleted, and its panel says
  what it is in one sentence: "Parts two rooms where no wall stands. It is not built
  and not priced." It has a switch with the rooms' layer, not one of its own.
- **The scan's sheet is darkened** so its dots can be traced over (they were 222 of
  255; walls on a sheet should read at about half that), and **the view moves to the
  sheet** when a scan file or a drawing is brought in, eased, as every move of the view
  is.
- **"meters"**, the US spelling, in the window and at the command line alike; and the
  bringing in of a scan is one function in the model's crates that both call.

No number moves. `demos/roomlinew.sh` shows the tool; `demos/scanw.sh` keeps passing
with its darker sheet (its measure's threshold may move with the sheet's darkness, and
says so).

### Call 154. Two layouts side by side (2026-11-22; plan step A3; the design review's "biggest single gain")

A builder designing an addition tries it two ways and wants the two set beside each
other. The first road is the plain one: **each layout is a job file, and the program
sets two or three of them side by side.** (BIMCraft keeps options inside one model,
`phasing.rs`, `OptionSet`; that is the later road, when a layout has to share one
drawing with another. A copy of the job is how a builder keeps a second scheme today.)

`plumbline options <a.json> <b.json> [<c.json>]` prints one table, each layout under
its job's title, and a last column, the last layout less the first:

- **Rooms**, matched by name: each room's floor area; a room one layout lacks reads
  "none"; then all rooms together.
- **Walls** by what the work does to them: new (and temporary), existing to remain, to
  demolish, each in LF.
- **Job cost**, and under it each division that has work in either layout.
- Every figure carries its flag (Assumed, Unknown) as it does in its own job; a figure
  with no value in a layout reads "no figure" and its difference is not worked.
- Nothing is written to any of the files.

One arithmetic: each figure is the one its own job's `takeoff` and `estimate` give; the
table adds nothing but the differences. In the window it is a view of the sheet,
built after this.

### Call 155. A kitchen set from the tool chest: cabinet boxes by code, in a run along a wall, with its countertop (2026-11-22; plan step A4; call 151; the design and finishes reviews)

**The tool chest gains cabinet boxes by the trade's codes.** In `library/items.csv`,
Division 12, each an item to place like any other, its sizes Assumed and sourced "the
trade's stock sizes; the maker's catalog decides": base boxes 34 1/2 in high and 24 in
deep (B12, B15, B18, B21, B24, B27, B30, B33, B36; sink bases SB30, SB33, SB36; drawer
bases DB18, DB24); wall boxes 12 in deep, named width then height (W1230, W1530, W1830,
W2430, W3030, W3630, W3018, W3612); tall boxes 24 in deep and 84 in high (T1884, T2484);
a 3 in filler (F3). The code's number is the width in inches, as the trade reads it.
The one "Base cabinet, 36 in" stays for the jobs that have it.

**A run.** `plumbline cabinets <file> <level> <wall> --in <room> --from "1 ft" --boxes
"B18 SB36 B24"`: sets the boxes side by side along the wall's face on the room's side,
backs to the wall, starting so far along the wall's line from its first end, each
turned to the wall. Base and tall boxes stand on the floor; wall boxes hang with their
bottoms 54 in up (18 in over a 36 in counter), Assumed, said. Each box is one of its
item, as sure as the item; each carries one cost line, its item's name, 1 EA, Division
12, with no price, so the estimate by item is the box schedule a cabinet maker quotes
from and the supplier list has every box by its code. It says the run's length and
what is left to the room's corner. Refused, in words, the job unchanged: a code the
chest does not have (naming those it has); a run that passes the room's corner; a run
across a door.

**Its countertop.** With `--countertop`, one countertop is set over the base boxes of
the run: as long as they are, 25 1/2 in deep (a 1 1/2 in overhang at the front),
1 1/2 in thick, at 34 1/2 in up, Assumed. **A thing set in place has figures of its
size** so it can be priced by them: `width` and `depth` (ft), `height` (ft) and
`plan_area` (SF). A countertop is priced by `plan_area`; its front edge by `width`.

**Not yet:** corner boxes; a filler worked out from the wall's length; door styles and
finishes as selections; the run set and dragged in the window; the Kitchen assembly's
three "1 EA" lines (cabinets, countertops, appliances) still go out as Known, which is
wrong and is next.

No number moves on a job made before this; the tool chest's count of items does, and
what counts it moves with it.

**Call 155, as built (2026-11-22).** One expectation moved by this decision and not by
the worker, who left it failing and said so: `crates/world/tests/tables.rs` held every
starter item's width to the one sentence "a typical size; the product is not chosen".
What that test guards is that every starter size is Assumed and says where it came from;
the cabinet boxes say "the trade's stock sizes; the maker's catalog decides", which is
truer of a stock box than "typical". The test now takes either sentence, each exactly,
and nothing else. The worker's own choices, taken: tall boxes are drawn as a wardrobe
and the filler as a box; the filler is 3 in by 24 in by 34 1/2 in; a run that mixes wall
boxes with others is refused; a box's line is Material; a door whose place is not known
is not checked for a run across it (owed: it should refuse). The counts of the starter
items moved from 21 to 46 in `tables.rs`, `demos/items.sh`, `demos/itemlibw.sh` and
`demos/turnw.sh`.

### Call 156. The kitchen run and the layouts in the window (2026-11-22; plan steps A3 and A4, their window halves)

- **A tool, "Cabinets", in the BUILDING row after Room line.** With it in hand, a click
  on a wall's face on the plan picks that wall and the room on that side, and where
  along the wall the click fell is where the run starts (snapped to the room's corner
  when within 6 in of it). A field then takes the boxes as the command line does, their
  codes with spaces between ("B18 SB36 B24"); Enter sets the run through
  `editing::cabinets`, with its countertop when the switch "Countertop" beside the field
  is on (on to begin with). What the window says is what the command line says; a
  refusal is said and nothing is set. The codes the job's items have are listed under
  the field as they are typed, so nobody has to remember them; a job with no boxes says
  to bring the starter items in. Undo takes a run back whole.
- **Layouts, a tab of the sheet window beside Estimate and Budget.** A button "Set
  beside another layout" lists the job files in the job's folder (a file is picked from
  a list); the picked one is set beside this job as `view::options` works it, in the
  rows and words of `plumbline options`; "Take it away" clears it. Nothing is written
  to either file. Up to two others.

No number moves. `demos/cabinetsw.sh` and `demos/layoutsw.sh` show them, their saved
jobs and what the window said checked against the command line's demonstrations'
figures.

### Call 157. A room's run of wall; the kitchen and layouts set right, from their judge (2026-11-22; the judge of calls 152, 154, 155)

**1. A room has `wall_run` (LF), and base is priced by it.** Call 152 let a room's
perimeter take in a room line, and the starter library prices base by perimeter: on the
judge's room 31 LF of wall read 46.5 LF. Perimeter stays what it is, the distance round
the room. The new figure is the run of the room's walls along its floor line: each
side of the room that lies along a wall, less the doorways in it (a door to demolish is
filled in and is no doorway), nothing for a side along a room line; islands standing
in the room with it. It is the floor line `receptacles` already walks. Its status and
inputs are `wall_area`'s. In `library/sixteen-divisions.csv` every "Base and trim" and
"Install base and trim" line is priced by `wall_run` and no longer by `perimeter`.
**What moves:** base on any room priced from the starter library, down by its doorways
and room lines (the drywall demonstration's rooms: 54 LF round, one 3 ft door, 51 LF).
Each test and demonstration that checks such a quantity moves with it and is named in
STATUS.md with its old and new figure.

**2. `options`**: under the job cost, a row "left out of the job cost" with each
layout's count of lines the estimate leaves out (none reads "none"), so two job costs
are not set side by side as if both were whole; where two layouts have one title, each
column is headed by its title and its file's name; a wrong count of files is refused in
a sentence, not with the whole usage.

**3. `cabinets`**:
- A box that would stand in another thing set in place (a box of another run, an
  appliance) is refused, naming it and by how much: a second run over the first, and a
  run into the corner another run fills, are no longer taken. A run is started off the
  corner by what the other run fills (24 in for base boxes), by hand.
- A wall box or a tall box that would stand across a window is refused, naming the
  window; a base box under a window is taken where the sill is at or above the box's
  top, and refused where it is below.
- `--boxes` takes any item of the job by its name as well as a code; a name with a
  space or a comma in it goes in square brackets: `"B18 [Dishwasher] SB36 [Range, 30
  in]"`. An appliance in a run stands on the floor, back to the wall, as a base box
  does; the countertop is not laid over a range or a refrigerator (anything taller
  than 34 1/2 in), and is laid over a dishwasher.
- Two corner boxes in the starter items: BLB36 (a blind corner base, 36 in wide, 24 in
  deep) and LS36 (a turning corner base, 36 in by 36 in), sizes Assumed as the others.
- A run on a wall that is to demolish is refused for that reason, in words.
- The supplier's list says how sure a box is by the worst of where it stands and its
  size (a box set Assumed reads Assumed, not Known).

**4. A wall removed or marked to demolish** says what is set against its faces and still
stands ("3 things set against it still stand: ...; move or remove them"). Nothing is
moved or removed for the estimator.

**5. A price by its code.** `plumbline price-item <file> "<item>" '$180' --status ...
--source ...` puts that unit cost on the line named for the item on every one of it set
in the job, and says how many; one not yet set takes nothing, and that is said.

**6. A room line that reaches no wall and no other room line at an end** is taken and
said: "its end at x,y meets no wall and no room line: it parts nothing until it does".

Not in this call: an island (a run off a wall); end panels and toe kick; a countertop
that follows a box taken out of its run; more sizes.

### Call 158. Every count and list says what is built, what stays and what comes down, and whose it is (2026-11-22; plan step B5; the GC, MEP and finishes reviews)

The takeoff said 6.09 CY of footing concrete on a job where 2.12 CY is new; an existing
air handler went out to a sub as "set and connected"; plumbing and HVAC could not be
handed their own lists. Call 148's mark is what tells them apart, and it is used.

- **`takeoff`.** Where a level (or a roof, or a system's runs) has anything that is not
  new work, each count and quantity is given three ways under its line: **to build**
  (new and temporary work), **existing to remain**, **to demolish**; the line itself
  stays the whole, as it stands, and says "as it stands". A job with no mark on anything
  prints as it did.
- **`export supplier` and `export subcontractor`** gain a column, **Work** (new work,
  existing to remain, to demolish, temporary): what the thing the line is on is. A line
  on an existing thing is work done to it and says so; nothing is dropped.
- **`export subcontractor`** gains **Division** (its number and title) and **System**
  (the system a run, a fitting, a fixture or a piece of equipment is on, blank
  otherwise), and takes `--division <1 to 16>` and `--system "<name>"` to write one
  trade's or one system's scope alone; what the filter left out is counted in a last
  line ("12 other lines, in other divisions, are not in this list").
- **`export pm`** gains the Work column.
- One arithmetic: the three ways add up to the line, to the digit; each is the sum of
  the same figures the line sums.

No number moves; lists gain columns and the takeoff gains lines.

### Call 159. Work on part of a thing, typed as a measure; taking a partition down as a scope (2026-11-22; plan step B6; the finishes and GC reviews: "a typed patch of 60 SF is refused"; "this is the whole renovation job")

**1. A line's quantity may be a measure typed outright.** A renovation is full of work on
part of a thing that no figure of the model gives: 60 SF of siding patched, 24 LF of
base matched, 2 CY hauled. `plumbline line <thing> "Patch and match siding" "60 SF"
--status Assumed --source "walked it"` is taken: a number and one of LF, SF, SQ, CY or
EA. It is held as a value entered on the line (named `measured`), with how sure it is and
where it came from, as every entered value is; `--status` and `--source` are required
with it, and it is refused without them, in words. A plain number is a count, as before.
In a formula such a value is what its unit says (an area, a length, a volume, a
count), so `measured * 2` is in SF. `set <thing> measured "72 SF" --line "<line>"
--status ... --source ...` changes it, and `measure` makes it Known. A value entered on a
line may carry SF, SQ, CY or EA as well as a length, by `--with "name=60 SF"`.

**2. A starter assembly, "Take down a partition" (goes on a wall)**, Division 2 unless
said, no prices, each rule Assumed with its reason; a sub prices demolition his own way
and the units here are the estimator's to change:
- "Protect the floors, both sides": `2 * plan_length * strip`, SF, `strip` = 4 ft
  ("a 4 ft run of floor protection each side: the estimator's allowance"), Division 1.
- "Dust barrier": `barrier * height`, SF, `barrier` Unknown ("where the work is closed
  off is the job's to say: give its length"), Division 1.
- "Take down the partition": `face_area`, SF, subcontract.
- "Haul off and dump": `net_area * thickness * swell`, CY, `swell` = 1.5 ("broken board
  and framing lie loose at about half again their volume in place: the estimator's
  allowance; the hauler's box is what is paid for").
- "Patch the floor where it stood" and "Patch the ceiling where it stood": each
  `plan_length * thickness`, SF, Division 9.
- "Patch the walls it met": `ends`, EA, `ends` = 2 ("one patch at each end; a wall that
  ends free has one"), Division 9.
- "Shore the floor above": Labor by `plan_length`, in hours with no production rate,
  **switched off**: on only where the wall carries load (call 160 says how that is asked).

The library is then 37 assemblies and 179 lines, 25 of them labor; what counts it moves.

No other number moves.

### Call 161. The five new things done from the screen alone (2026-11-22; the judge of the window's pieces: a scan file "no", a second layout "no", cabinets and a room line "partly")

A builder opening the window cold has to be able to do each of the five without a manual
and without a software word. What changes, each in the window's crate:

1. **Bring in lists.** In every list under Bring in, a click on a row brings that file
   in (Enter still does). A row shows the file's name without its ending and what it is
   in a builder's word (scan, plan, price list, job). The folder is named "this job's
   folder", never by its path. **Scan file:** the list's head has two choices, lit when
   chosen, "Measured in meters" and "Measured in feet" (meters to begin with). The
   message after says how big the house came in and "which way is up was read from the
   scan"; if it came in on its side, one button under the message, "It is lying on its
   side: stand it up", brings it in again turned. Nobody types an axis.
2. **Layouts, made in the window.** On the Layouts tab: "Keep this as another layout"
   takes a name in a field (the job's title and "B" offered), saves a copy of the job
   beside it under that title, and sets it beside this one. "Open another layout" lists
   the jobs in this job's folder by their titles (only files that are jobs) and opens
   the picked one in this window; with changes not saved it says "Save first (Ctrl+S)"
   and opens nothing. The table carries call 157's "left out of the job cost" row and
   file names under like titles.
3. **Cabinets.** The field takes what the command line takes, appliances by name in
   square brackets with them; the list under it shows the corner boxes and the
   appliances the job has, and one line of key: "B base · SB sink base · DB drawer base
   · W wall (width, then height) · T tall · F filler · BLB blind corner · LS turning
   corner · the number is the width in inches". On the plan each box carries its code,
   and a wall's dimension is not written over a run. The sheet's estimate lists each
   box's line.
4. **What is existing and what comes down, keyed.** Under Show, with anything marked:
   "Existing, shown pale" and "Demolition, shown red", each with its swatch. Existing
   work keeps its pattern, washed pale (brick still reads as brick). When a mark changes
   a room, the message says so: "Living is now 364.25 SF (was 178.25)".
5. **A room line** is drawn in its own ink (dark blue, dashed), over whatever it lies
   on, and the tool's hint says what it is for: "Parts two rooms where no wall stands:
   click each end."
6. **Open items** do not list an item's unknown maker, model or weight until one of it
   is set in the job. A refusal is said on the card and in the message line, not in the
   panel as well. A thing is named once.
7. **Every picture of the five demonstrations is retaken** on the build that has this,
   and each demonstration is changed to do its step the new way (a click on the scan's
   row; the second layout kept from the window; an appliance in the run) and to check it.

No number moves.

### Call 163. The scan's message in the window, in the words a builder reads (2026-11-22; the lead's reading of the scan pictures retaken for call 161)

Call 161's worker read its own pictures and said the scan's message is long and that
"each point's red, green, blue, alpha" is not a builder's word. It is right, and so is
"672 by 480 dots at 24 to the foot". In the window's message only (the command line's
`import scan` prints what it printed: whoever types the verb is asking for the detail):

- "Cut 4' 0" above the floor: N points drawn on the sheet, W by H dots at 24 to the
  foot." becomes "**It is cut 4' 0" above the floor, as a plan is.**" The sheet's size
  in feet is already under Drawings.
- Each "Not kept: ..." line (the file's own column names, its count of faces) becomes
  one sentence, said whenever anything in the file was not kept: "**Only where its
  points are is kept: not its colors, and not the surface drawn over them.**" It still
  turns the message to a warning: loss is said, never silent.

`demos/scanw.sh` checks the new sentences in place of the old three. No number moves.

Call 161 as accepted, and one word of the worker's that stands for now: in the window a
refusal that the model words with "the tool chest" reads "this job's items", because the
window's own list is headed Items. Patrick's word is tool chest (call 151). Which of the
two the window's list is headed is owed as one wording call with the next window work,
made once and carried to every control's name in one pass.

### Call 160. What a wall the work opens may be hiding, asked in the field (2026-11-22; plan step B7; the GC, MEP and structure reviews: "the remodel's money is in what is behind the wall"; "a run through a demolished wall is silent everywhere")

**1. Two things can be said of a wall**, each a value with how sure it is and where that
came from, as every value is (held on the wall as `pl::hidden`; left out, nothing was
said):
- **bearing**: `bearing` or `not bearing`, the trade's two words for whether it carries
  the floor or roof above;
- **in the wall**: in words, what runs inside it ("nothing", "a 2 in drain and two
  cables").

`plumbline set <file> <wall> bearing "not bearing" --status Known --source "joists run
with it, seen from the attic"`; `set <file> <wall> "in the wall" "one 2 in drain"
--status ... --source ...`; `unknown` puts either back to not known. Any other word for
bearing is refused, with the two it takes.

**2. They are asked of a wall the work opens, and of no other.** The work opens a wall
when it is marked to demolish, or when it is marked existing and a door or window not
itself marked existing is in it (a new opening cut in an old wall). On such a wall each
of the two not yet said reads **Unknown**, with its reason ("not seen until it is
looked at": for bearing, "which way the joists above run and what bears on it"; for in
the wall, "pipes, ducts and wiring in an old wall are not seen until it is opened"), in
`show`, in the window's open items, and on the field list as things to **check**, not
to measure. Everything priced on that wall rides on them, as it does on what a thing is
(call 59): a price to take a wall down is a price for something else if the wall is
bearing. Said Assumed they are still asked; said Known they leave the list. A wall the
work does not open is asked nothing; a word said of it anyway is kept and shown.

**3. A bearing wall the work opens says what that needs, and that it is not in the
price.** Where bearing is said (Assumed or Known) of a wall that comes down or is cut:
a caution on the wall, in `show`, in `check` and under the estimate's totals: "Wall 5
is bearing and comes down: what it carries needs a beam, its posts and their footings,
sized by an engineer, and shoring while it is out. None of that is in this price unless
a line of this job says so." (For a new opening: "is bearing and is cut for Window 3:
the opening needs a header sized by an engineer, and shoring while it is cut.") The
program sizes nothing and prices nothing by it: a flag, never a fabrication; no span
table is in the program (their terms do not allow it; `research/renovation-structure.md`).
The beam itself is plan step C8.

**4. What is in the way of a wall that comes down is asked too.** A run (pipe, duct,
conduit, cable), a fitting or a placed thing that the clash test's own shapes find in a
wall marked to demolish, and that is not itself marked to demolish, is a question on
the field list: "Pipe 1 is in Wall 5, which comes down: cap it, reroute it or take it
out", where it is, with what is priced on that run riding on it. `clashes` says it in a
count of its own, "in the way of demolition", apart from the clashes (it is not one:
call 148 stands, a demolished wall is in nothing's way once the work is done). Marked to
demolish itself, or moved clear, it goes.

**5. Marking says so.** When a wall is marked to demolish, what `mark` says ends: "Is
it bearing, and what is in it? Both are on the field list."

The saved job gains `pl::hidden` on a wall something was said of; a job from before
opens as it was, and a job with no mark has no new row, no new question and no number
moved.

### Call 162. A kitchen's base, its lump lines and its countertop set right (2026-11-22; the judge of call 157: J-C-C, J-C-D, J-C-G)

**1. Base does not run behind what stands on the floor against the wall.** A room gains
`base_run`: its wall run (call 157) less the length of its walls' faces that things set
in place cover at the floor. A thing covers a face where its plan lies against the face
(within the fit tolerance the clash test uses) and it stands on the floor (it is not
hung: a wall box 54 in up covers nothing); what it covers is the length of the face its
plan lies along, so a box in a corner covers its width on one wall and its depth on the
other, and two things covering the same length count it once. A thing marked to
demolish covers nothing. `base_run` is Assumed where anything it rests on is, and says
what it took off ("less 14 ft behind 5 things set against the walls"). In the starter
library every base line that was by `wall_run` (Room, Bath, Kitchen, Laundry or utility
room: material and labor, 8 lines) is by `base_run`. With nothing set against a wall it
is the wall run, to the digit: no number moves on such a room. `wall_run` stays and
means what it meant; a job priced before this keeps the figure its lines name.

**2. A lump line and the things it stands for, both in the price, is said.** The
starter Kitchen keeps "Kitchen appliances", "Cabinets" and "Countertops" at 1 EA: they
are the design-phase allowance, before anything is set. Once things are set in the room
that carry lines of their own in the same division, the estimate (under its totals),
`check`, and `cabinets` when it sets a run each say, once for each such line: "Kitchen's
line "Cabinets" (1 EA, 12 Furnishings) is in the price beside the lines of 7 things set
in Kitchen in that division: if it stands for them, take the line off." The rule is
general, not the Kitchen's: a line on a room whose quantity is a plain number, and
things set in that room (by where they stand) that carry cost lines in the line's
division. Nothing is taken off or switched off by the program.

**3. A countertop carries its line and is priced like an item.** The countertop
`cabinets` lays carries a line "Countertop" by `plan_area`, in SF, Division 12, with no
price. `price-item <file> Countertop "$62/SF"` prices that line on every countertop in
the job (a countertop is cut to its run and is no item of the tool chest: it is found by
what it is). A price per LF is by `width`, the front edge.

**4. A countertop over a turning corner box says what it leaves out.** Over LS36 (36 in
on each wall) a top 25 1/2 in deep is taken as 36 in by 25 1/2 in; the leg that turns
the corner, (36 − 25 1/2) by 25 1/2 in = 267.75 sq in = 1.859375 SF, is not in it. The
countertop's `plan_area` carries that as a caution, which goes with it to its line
(call 142) and is in what `cabinets` says: "the leg that turns the corner over LS36,
10 1/2 in by 25 1/2 in (1.86 SF), is not in this area: add it where the top is priced
by area." The area itself does not move: a shape the model does not hold is said, not
guessed. (The top as an L is owed with the countertop drawn to its true outline.)

After call 159 the library is 37 assemblies and 179 lines; this call changes the
formula of 8 of them and adds none.

### Call 164. A beam, its posts and their pad footings (2026-11-22; plan step C8, its first half; `research/structure-port-survey.md`; `research/renovation-structure.md`)

Where a bearing wall comes out its load goes to a beam, the beam's ends to posts, and
the posts to footings. The three are things of the model, drawn, counted, priced, in
the clash test and in the BIM file. **Every size is typed, with how sure it is and where
it came from; the program sizes nothing** (no span table is in it, and none may be).

**1. A beam** (`Kind::Beam`, said "beam") is held as BIMCraft holds one: a run, by its
two ends on plan, the height of its **top** above its level's floor, and a type that
gives its width and depth (ported: `Run`, `RunType`, justify Top as BIMCraft's beams
are, the command `beam`). A beam type may also give a **weight per foot**, typed (a
steel shape's name says it; no table of shapes is in the program). `plumbline type
<file> beam "<name>" --width .. --depth .. [--weight "18 lb/ft"]`; `plumbline beam
<file> <level> <type> --points "x,y x,y" --top "<height>"`. Refused, in BIMCraft's
three ways: too short; ends one over the other ("a beam runs along, and what stands is
a post"); sloped more than 75°. Its figures: `length` (LF, along the beam), `width`,
`depth`, and `weight` (LB = length × its type's weight per foot; where the type gives
none there is no such figure, and a line by it says why). It is bought by `length`.

**2. A post** (`Kind::Post`, said "post"; a steel column is a post of a round or a
square type) is held as BIMCraft's structural column: a point on plan, a height, the
height of its base above its level's floor, a type that gives its width and depth or
its diameter. `plumbline type <file> post "<name>" --width .. --depth ..` (or `--round
..`); `plumbline post <file> <level> <type> --at "x,y" --height ".." [--base ".."]`.
With no height given it stands to its level's height, Assumed, and says so (BIMCraft
stands a column to the level above). Its top must be above its base. Its figures:
`height` (LF), `width`, `depth`. It is bought by the piece.

**3. A pad footing** (`Kind::PadFooting`, said "pad footing") is held as BIMCraft's: a
post laid flat, its height its thickness, the height of its **top** given (below the
floor is a minus height). `plumbline type <file> pad "<name>" --width .. --depth ..`;
`plumbline pad <file> <level> <type> --at "x,y" --thickness ".." --top ".."`. The
thickness is required and above nothing (BIMCraft's default of 400 mm is not taken: a
thickness nobody gave is not a thickness). Its figures: `width`, `length`, `thickness`,
`plan_area` (SF), `form_area` (SF: its four sides, 2 × (width + length) × thickness)
and `volume` (CY). It is bought by `volume`, as a strip footing is.

**4. In the clash test.** A beam, a post and a pad footing are tested against every run
and every thing set in place, as a wall is: a duct through a beam is a clash. They are
not tested against walls, floors, footings or each other: framing bears on framing (a
post under a beam, a beam in a wall's line, a pad under a post), and `clashes` says so
once among what it leaves out and why.

**5. For the engineer.** `plumbline structure <file>` lists, level by level: each wall
the work opens with what was said of its bearing (call 160); each beam with its type,
length and the height of its top; each post; each pad footing; each size with how sure
it is and its source. It ends with the count of members whose size is not Known: "to be
sized by an engineer before it is built or priced firm". Nothing in the list is the
program's opinion of a size.

**6. In the BIM file**: a beam is `IFCBEAM .BEAM.` with `Qto_BeamBaseQuantities`
(Length, CrossSectionArea, GrossVolume: BIMCraft's `member_quantities`), a post is
`IFCCOLUMN .COLUMN.` with `Qto_ColumnBaseQuantities`, a pad footing is `IFCFOOTING
.PAD_FOOTING.` with `Qto_FootingBaseQuantities`; each carries its values and their
status as every written thing does. They are not read back in from another program's
file yet, and `import` says so by name.

**7. Three starter assemblies**, no prices, each rule Assumed with its reason: "Beam,
engineered lumber" (on a beam: the beam by `length`, LF; "Hangers and bearing hardware"
2 EA; labor "Set the beam" by `length`), Division 6; "Post" (on a post: the post, 1
EA; "Post cap and base" 2 EA; labor "Set the post", 1 EA), Division 6; "Pad footing,
formed" (on a pad footing: concrete by `volume`, CY; forms by `form_area`, SF; labor
"Form, place and strip" by `volume`), Division 3. Three assemblies and nine lines,
three of them labor, are added to what the library holds.

In the window they are drawn (a beam and a post as their timber, a pad as concrete,
with the Framing and Ground switches) and can be selected and read; tools to draw them
there are the next window work. The saved job gains the three kinds; a job from before
opens as it was. No number of any job without them moves. Joists and rafters counted
across a floor and a roof plane are call 165.

### Call 165. Joists and rafters counted (2026-11-22; plan step C8, its second half; JLC's Lumber field guide: "joists = floor width / spacing + 1"; `research/renovation-structure.md`)

A floor's joists and a roof plane's rafters are counted as a wall's studs are (call
137): **where a priced line names the framing**, by the rule walls use
(`finish::framing_of`: a lumber size and a spacing, "Floor joists, 2x10 at 16 in",
"Rafters, 2x8 at 24 in"), and nowhere else. They are figures of the floor and of the
plane, not things of their own; nothing is sized (the size is the line's words, typed).
BIMCraft lays out nothing across a floor or a roof (`research/structure-port-survey.md`):
the layout is ours, by the framer's own rule, and its stations are laid the way
BIMCraft's `timber.rs` lays a wall's studs, on multiples of the spacing from one end.

**The layout, one for both** (`members_across`: an outline on plan, the way the members
run, a spacing):
- A member is 1 1/2 in thick. Its middle is 3/4 in past its layout mark.
- **An edge member** lies 3/4 in inside every edge of the outline that runs with the
  members (a rim or end joist; the rafter at a rake), as long as that edge.
- **Marks** are at each multiple of the spacing from the outline's first side, square to
  the members (its west or south end first); the member at a mark is the line there cut
  to the outline: one piece for each length of it that lies inside.
- A piece that lies within a member's thickness of an edge member beside it, over the
  same length, is that edge member and is not counted twice.
- A piece under 6 in long is not a piece: it is left out, and the figure says how many.

**A floor**, where a line names its joists: `joists` (EA, the pieces), `joist_length`
(LF, their lengths added), `joist_longest` (ft: the stock they are bought in is that or
the next length up), `rim_length` (LF: the band the joist ends meet, the outline's
edges measured square to the joists). Which way they run is a value on the floor,
**"joists span"**: `the short way` (square to the floor's longest edge) or `the long
way`. Not said, it is taken as the short way, Assumed, "joists usually span the short
way: say which way these run", and every joist figure rests on it.

**A roof plane**, where a line of its roof names its rafters: `rafters` (EA),
`rafter_length` (LF, along the slope: each piece's length on plan times the plane's
slope factor), `rafter_longest` (ft). Rafters run up the plane square to its eave; the
marks are along the eave. The plane is as drawn, its overhang in it, so a rafter's tail
is in its length. A plane with no eave (or no pitch) has no rafter figures and says why.

Each figure is Assumed where the outline, the pitch or the way they run is; each says
what it was worked from. Hangers, blocking, bridging, doubled joists under a partition
and the headers round a stair are not counted: the figures' note says "blocking,
hangers and doubled members are not in this count". The starter library's "Framed
floor" gains "Floor joists, 2x10 at 16 in" by `joists` (EA) and "Rim joist" by
`rim_length` (LF); a starter roof assembly gains "Rafters, 2x8 at 24 in" by `rafters`.
No number moves on a floor or a roof priced before this: the figures exist only where
a line names the framing, and the lines a job already has are the job's.

### Call 160, as built: an opening that is taken out and filled in does not open the wall (2026-11-22; the lead, reading the worker's diff)

The call said a wall is opened when "a door or window not itself marked existing is in
it". The worker read that to the letter, so a door marked **to demolish** in an existing
wall asked the wall whether it is bearing. That is the wrong question: a door taken out
and filled in (call 150) cuts nothing; its header stays and the wall carries what it
carried. A wall is cut by an opening that is **new or temporary**: not marked, marked
new, or marked temporary. An opening marked existing or to demolish does not open the
wall. `demos/hidden.sh` gains the case: the north wall, existing, with a door in it
marked to demolish, is asked nothing.

### Call 166. One cost line taken off a thing (2026-11-22; found accepting call 162: the estimate says "take the line off" and nothing could)

A line put on a thing, by hand or by an assembly, could be changed, priced, switched off
(labor only) or cleared with every other line of the thing (call 148), and never taken
off by itself. `plumbline remove-line <file> <thing> "<line>"` takes that one line off,
with whatever was entered on it, and says what went: its quantity as it stood and its
price, or that it had none. A thing that has no line of that name is refused, with the
lines it has. The thing's other lines, its assembly's name and its code stay as they
are; what the estimate comes to drops by what the line came to, and by nothing else. In
the window it is a button on the line (the next window work). `demos/kitchen.sh` takes
the Kitchen's "Cabinets" line off with it, in place of editing a copy of the file.

### Call 167. What the judge of calls 159 to 166 found, set right (2026-11-22; `research/judgment-calls-159-166.md`)

1. **A typed measure that is not Known is a field question** (J166-2). A measure typed
   on a line ("60 SF", Assumed, "walked it") is a measurement of the building, and the
   loop is for it: the field list asks it under its thing, to measure, by the line's
   name and what was typed, with that line's dollars riding; `measure <thing> measured
   same --line` (or a new figure) makes it Known and it leaves. Only values in a
   measure's units (LF, SF, SQ, CY, EA) are asked: an assembly's rule values (a strip's
   width, a swell) are the estimator's allowances and are not site questions, as now.
   The list's sentence "Every value that can be measured is Known" is then true when it
   is said.
2. **A line bought by a length reads in LF** (J166-13). Where a line's quantity works
   out to a length, its unit is LF and its price is per LF, whatever unit the figure it
   is taken from is shown in (a thing's `width` is a size, shown in ft). `price-item
   Countertop "$80/LF"` puts the countertop's line on `width`, its front edge, at that
   price; `"$62/SF"` puts it on `plan_area`. A price per any other unit is refused with
   the two it takes.
3. **A bearing wall the work opens says so under the estimate whether or not anything is
   priced on it** (J166-8).
4. **The place given for a thing in the way of a wall that comes down is on that thing,
   inside the wall** (J166-9: with three pipes meeting it was a foot off).
5. **An assembly less a line says so, and can be made whole** (J166-19). Where a thing
   carries an assembly and lacks lines that assembly has (taken off with `remove-line`),
   `show` reads the assembly's name "without its line "Cabinets"". Putting the same
   assembly on again adds the lines of it the thing lacks, says which, and touches no
   other line; with none lacking it says the thing has every line of it already.
6. **Measures and words as estimators write them** (J166-1, 7, 12). A typed measure
   takes thousands with a comma ("1,200 SF") and the units' common spellings: sf, sq ft,
   sqft; lf, lin ft, ft (a length typed on a line is LF); sq, square, squares; cy, cu
   yd; ea, each. One below zero is refused as below zero; what is not a measure is
   refused with the units a measure takes. `bearing` takes "non-bearing", "nonbearing",
   "load bearing" and "load-bearing" as well as its two words, in any capitals, and
   prints its two. "in the wall" is taken in any capitals, and the list of what can be
   said of a wall names both.
7. **A lump line is one whose quantity names no figure and is more than nothing**
   (J166-15): a number, numbers worked together (`1 + 1`), or a count typed as a measure
   ("4 EA"). A quantity of 0 is not one.
8. **`place` takes an item by its name** (J166-16), in any capitals, as well as by its
   number; two items of one name are refused with both.
9. **`mark` asks "Is it bearing, and what is in it?" only of what has not been said**
   (J166-10).

Left open, each low: a typed measure's source in the workbook; "0 SF" taken without a
word; the odd words when an assembly would double a line; `check` without the
corner-leg caution; how near a wall and the floor a box must be to cover base (it
over-buys base, never under); `remove-line` minding capitals. J166-20 is as meant: a
blind corner base is 24 in deep and the top covers it.

### Call 168. A plan set on paper: the plan as it stands, the demolition plan and the proposed plan (2026-11-22; plan step D10; `research/sheets-port-survey.md`; the design and GC reviews)

A renovation is built from three plans of each level. The model already holds what each
shows (the work mark, call 148); nothing was written out. `plumbline plan-set <file>
<level> <out.pdf>` writes them, one page each, as line work a printer takes; "plan set"
is the builder's word (a "sheet" on this program's screens is already the estimate's, and
a "drawing" is one brought in).

**What each page shows.**
- **EXISTING PLAN**: what is marked existing or to demolish, as it stands, in full
  weight; new work is not on it.
- **DEMOLITION PLAN**: what stays, drawn light; what comes down, drawn dashed; new work is
  not on it. A line under the title counts what comes down ("1 wall, 16 LF").
- **PROPOSED PLAN**: what stays, drawn light and unfilled; new work heavy, its walls
  filled; what comes down is not on it.
A level with no mark on anything gets one page, **FLOOR PLAN**, everything heavy. A page
with nothing to show is not written, and what was written is said.

**How it is drawn** (ported from BIMCraft's plan pass, sheet and PDF writer: the display
list of fills and lines, `cut_unions`, `cut_edges`, `ink_cut` and `turn_corners` for walls
that read as one where they join, the door's gap and jambs, the window's three lines, the
pens by scale, the paper sizes, the lettering of strokes, the PDF written by hand with
`flate2`, already in the lockfile; no font and no new dependency):
- Walls cut at the level's own height for it (4 ft), by their thickness, joined.
- A door is its gap, its jambs and its leaf drawn shut across the gap: **which way a door
  swings is not in the model**, so no swing is drawn, and the page says so in its notes
  ("door swings are not set"). (A door's hand is its own call.)
- **Rooms**: each room's name and its area in whole SF at the room's point. On the
  proposed plan they are the rooms the work leaves. On the existing and demolition plans
  they are the rooms as they stand: each room's point bounded by the walls that stand
  today (existing and to demolish, no new wall, no room line that stands for a wall taken
  down); a room whose point no standing wall encloses is not lettered there.
- **Dimensions**: one string along each wall on the page, on its outer side, end to end
  of the wall's line as drawn, with a tick at each end and its length in feet and inches
  as plans write them (`24'-0"`). **A length that is not Known is lettered with VIF after
  it** ("verify in field", the trade's own mark), and one that is Unknown is lettered
  `?`: the paper carries what the model knows about each number. Openings are not
  dimensioned yet, and the notes say so.
- **Scale and paper**: `1/4" = 1'-0"` unless `--scale` says another (1/8, 3/16, 1/4, 3/8,
  1/2 in to the foot); the smallest of ARCH A (12 by 9 in), B (18 by 12), C (24 by 18)
  and D (36 by 24) that the level fits on with its dimensions and a 1/2 in margin, unless
  `--paper` names one; if it fits on none at that scale it is refused with the scale that
  does fit.
- **Title box**, lower right: the job's title, the page's name, the level, the scale, the
  date (`--date`, else the day it is written), and "page 1 of 3". **Notes**, beside it:
  what VIF means and how many lengths carry it; that door swings are not set; that
  dimensions are to the walls' lines as drawn.
- Plan north is up the page. No north arrow: true north is not in the model.

`plan-set ... --list` prints, page by page, what is on it (each wall with its weight,
each dimension as lettered, each room label, the notes), so a figure on paper can be
checked without reading the picture. The same pages can be written from the window under
Write out (the next window work). The saved job does not change; no number moves.

### Call 169. The window, from what its second judge met (2026-11-22; `research/judgment-call161-window.md`: three of the five things done from the screen, two partly) and the window's half of calls 159 to 167

**From the judge.**
1. **Several things marked at once** (J161-5: "select the walls and press Existing"
   could not be done). Shift and a click adds a thing to what is selected, or takes it
   out; the panel is the last one picked, headed "and 3 more". The panel of a wall,
   door, window, floor or footing has **"Select all on this level"**. The WORK row and
   Delete act on all that is selected, in one step that one Undo takes back; every
   other control acts on the one whose panel it is, and says so when more are selected.
2. **A scan in the wrong measure says so** (J161-1, 4). Every scan's message ends: "If
   that is not the size of the house, choose the other measure at the head of the list
   and bring it in again." A scan refused because no point of it is at the height it is
   cut at adds the same. A house over 150 ft long, or a floor to ceiling under 6 ft or
   over 15 ft, turns the message to a warning with that sentence first. The same scan
   brought in twice: "It is already in, under the plan. Undo takes it out." (J161-2).
   Stood up, the message begins "Stood up."
3. **A room with no area says what to do** (J161-6): "Kitchen has no area yet: its point
   is in the space Study has. Part the two with a wall or a room line, and each gets its
   own." (the model's sentence, so the command line says it too).
4. **A room line that parts nothing says so** (J161-7): "Added room line 3. It does not
   run from wall to wall, so it parts nothing yet." A line that does part two rooms says
   what it said.
5. **Where a run of boxes starts** (J161-10): at the click, to the nearest foot along
   the wall; the hint and the message say "the run starts where you click; type another
   start in Starts at". `TRY-IT.md`'s sentence about 24 in is for a corner another run
   fills, and says only that.
6. **The laser's message only for a laser** (the door note): "A reading came in with no
   value picked to take it" is not said after a tool's own sizes are typed.
7. **A message card can be put away** (J161-3; the fixed rule: every panel can be): a
   mark at its corner puts it away, and so does Esc with no tool in hand.
8. **Appliances on plan by their plan marks** (J161-9): DW, REF, RNG, WASH, DRY; any
   other label too wide for its box is lettered smaller, never over its neighbour. The
   key card says the marks.
9. **Words** (J161-11, 12, and the builder's own list): with changes not saved the list
   of jobs to open is grey; a job is named by its title, and by its file's name as well
   only where two titles are alike; "Existing, shown pale" shows only where something is
   so marked; a mark that changes a room's receptacles says so with its area.

**The window's half of the command line's calls.**
10. **A line's quantity takes a typed measure** ("60 SF": call 159); typed in the
    window it is Assumed, "entered on screen", as every typed value is.
11. **A wall the work opens** (call 160): its panel has Bearing with "Bearing" and "Not
    bearing", and "In the wall" with a field; each reads unknown with its reason until
    said; what a bearing wall needs is a card in the panel; the field list and the open
    items carry both questions and what is in the way of a wall that comes down.
12. **"Take line off"** on a cost line (call 166), one Undo.
13. **The tool chest** (call 151; Patrick's word): the list headed Items is headed
    **Tool chest**; "Starter items" is "Starter tool chest", "Item library" is "Tool
    chest file", "Item library, to keep" is "Tool chest, to keep"; a thing in it is
    still an item ("New item"). The window says "the tool chest" wherever the model does
    (call 163's stand-in, "this job's items", goes). Every script that clicks one of the
    three renamed controls is changed with it, in the same pass.

Owed after this, each its own call: a wall moved with the walls that meet it following
(the judge found no way to make the addition longer but to take walls down and draw
them again); the counts three ways on the Building tab; tools to draw a beam, a post
and a pad footing; the plan set written from the window.

### Call 170. A wall moved, and the walls that meet it follow (2026-11-22; the window's second judge: "no way to make the addition longer but to draw it again"; Patrick's job: rooms whose footprints change)

Design is pushing walls about. Today `move` shifts a wall alone and leaves its
neighbours where they were, a gap at each end. BIMCraft's `move` (`commands.rs:12419`,
`relocate`, and its `end` for one end of a wall) is the source to port from; what it
does at a joint is read there before anything is written here.

**A wall moved takes with it the ends that meet it**, unless it is moved `--alone`:
1. **At its ends**: every other wall's end that is at one of its ends (within the
   tolerance walls join by) moves as that end moves. The neighbour grows, shrinks, or
   turns if the move is not along it.
2. **Along it**: every other wall's end that lies on it between its ends (a partition
   that butts into it) moves by the part of the move that is square to the moved wall:
   it stays butted, and keeps its place along.
3. **Where one of its own ends butts into a wall that is not moving**: that end stays on
   that wall. It goes where the moved wall's new line meets that wall's line. (So a
   partition slid sideways between two walls keeps its ends on them, and one pushed
   along its own length does not move at all: that is refused, in words.)
4. **Doors and windows stay where they are in the house.** One in the moved wall goes
   with it (it is placed along it). One in a wall whose first end moved along the wall's
   own line keeps its place by having its offset changed by as much; one that would no
   longer lie within its wall refuses the whole move, naming it.
5. **What stands against the moved wall does not follow**, and the reply says how many
   and which ("1 thing stands against wall 2 and was not moved").
6. Everything whose line changed is no surer than the move, as now: the moved wall and
   each wall that followed. A wall marked existing that is moved or made to follow is
   said by name: "wall 3 is marked existing: as moved, it is not as it stands."
7. The whole move is one step: all of it is done or none of it, and one Undo takes it
   back in the window.

The reply names each wall that followed and by how much it grew or shrank. In the
window the Move tool does the same; with a wall in hand, a length typed and Enter moves
it that far toward the pointer, square to the wall where walls meet its ends. A run, a
thing set in place, a fitting and a room's point move as they did. No saved job
changes; a wall that no wall meets moves exactly as before.

### Call 171. The window's half of calls 164, 168 and 170, and the counts three ways (2026-11-22)

What the command line gained apart from the window, brought into it; nothing here works
a figure out, each is the model's.

1. **Move** (call 170). A wall moved in the window takes with it the walls that meet it,
   as the command line does, in one step that one Undo takes back; the message says what
   followed and by how much, what stands against it and was not moved, and any existing
   wall that moved, in the model's own sentences. With a wall in hand, a length typed and
   Enter moves it that far toward the pointer, square to the wall where walls meet its
   ends. The tool's strip has a switch, **Alone**, off to begin with. A refusal (a door
   left outside its wall; a wall held at both ends pushed along itself) is the model's
   sentence, and nothing moves.
2. **Beam, Post, Pad footing** (call 164): three tools with the building's. Beam: its
   width, depth and the height of its top, then two clicks. Post: its width, depth and
   height, then a click. Pad footing: its width, length, thickness and the height of its
   top, then a click. A size typed is Assumed, "entered on screen", as every typed size
   is, and each tool's hint ends "an engineer sizes it: this is what was typed". A beam
   and a post show with the Framing switch, a pad with the Ground switch; each is drawn
   to its size (timber for a beam and a post, concrete for a pad), can be selected, and
   its panel shows its figures, its type's size with how sure it is, and Assembly.
3. **For the engineer** (call 164): on the sheet window's Building tab, a card "What
   carries the building" with the model's `structure` list and its last line, the count
   of sizes not Known.
4. **Plan set** (call 168): under Write out, "Plan set" writes the level on show to a
   PDF in the job's folder, named for the job and the level (never over a file that is a
   job), and the message says which pages were written, on what paper and at what
   scale, in the model's sentence.
5. **Counts three ways** (call 158): on the Building tab, wherever anything it counts
   carries a mark, each count reads to build, existing and to demolish under its line,
   as `takeoff` prints them.
6. **What the clash test leaves out** names framing where the job has any (call 164).

Each is proved by a window demonstration that does it the way a stranger would, checked
in what the window said and, where it is something seen, on the picture.

### Call 172. What the program is for, in Patrick's words, and the order of work turned to it (2026-11-22)

Patrick, 2026-11-22, asked what was done, why it had stopped, and whether the builder
was "building what you think is easy what is known vs making the first 3d first
renovation oriented platform that allows pdf, model or lidar integestion that every
person can use for free so knowledge is no longer lost or mid translated"; and whether
what is known in building and in technology had been used and refined "to work smarter,
more in tune with field users".

**The purpose, as ruled.** The first 3D-first, renovation-first platform. It takes in
what a person already has (a PDF, a model, a LiDAR scan) and gives back the house in
3D, priced, with what must be checked in the field. Free, for every person. What a
tradesman knows is kept in it and reaches the next person without being lost or said
wrong on the way.

**What the answer to his question was, written down so the next builder does not repeat
it.** The model underneath is right for that purpose and is the hard part done: every
value Known, Assumed or Unknown with its source; existing, to demolish, new; what is
hidden in a wall asked before it is opened; clashes as field questions sorted by
dollars; sixteen divisions priced. But three things were built the easy way:

1. **Taking in.** A scan is cut into a picture and traced by hand. A PDF is calibrated
   and traced by hand, snapping to its lines. PROPOSALS.md 32 put "walls and rooms found
   in the scan without tracing" last, as research, "not to be started before 1 and 2
   are used on a real house", and recorded Patrick as content to trace. That was the
   builder choosing what a machine with no screen could prove. Finding flat faces in
   points is known work, and the program's own rule (it arrives Assumed, sourced, and is
   checked one by one) is exactly what makes a rough find safe to use.
2. **Proof.** Most judging went to the command line, because it is quick to check here.
   The window a person uses has trailed the arithmetic since the eighteenth judgment.
3. **Reach.** "Every person, for free" has an install line for Linux and nothing else,
   and the window has never run on a real graphics card.

**The order of work from here** (this goes ahead of the smaller open items; calls 169
and 171 and the held patch of 164, 165, 168 and 170 are finished and landed first, as
they are already built or in hand):

1. **Walls, openings and rooms found in a scan** (call 173), at the command line, then
   in the window as the first thing a scan offers when it is brought in.
2. **Walls found on a drawn PDF** the same way: the lines are already read to snap to;
   pairs of long parallel lines a wall's thickness apart are offered as walls, Assumed,
   sourced to the sheet (a call of its own, written when 1 is in hand).
3. **The scan seen in 3D with the house built inside it**, so a person sees what was
   found against what was scanned, and a thing that was missed.
4. **A mesh from the phone** (GLB or OBJ) read as a scan is, since that is what most
   people will export first.
5. **Reach**: a program a person downloads and opens on Windows, macOS and Linux with
   no build; the first job walked on a machine with a graphics card.
6. **What a tradesman knows, kept**: the item and assembly tables as files a person
   reads, changes and hands on (PROPOSALS 24, 30), and each division's harness (calls
   75, 76).

Nothing in the fixed rules moves. What is found by the program is Assumed, never Known.

### Call 173. Walls, openings and rooms found in a scan (2026-11-22; call 172's first)

A scan already brought in as a sheet (call 128) can be read for its walls. What is found
is offered, said in full, and laid only on the person's word. The finding is geometry in
`crates/plans`; laying uses the wall, door and window the model already has. One
arithmetic: the floor, the ceiling and the levelling are the ones call 128 already
works out.

**What a wall is, in a scan.** A flat upright face at least 2 ft long whose points run
from near the floor to near the ceiling (within 6 in of each, over its length, but for
its openings). An upright face that stops short of the ceiling is something standing in
the room (a cabinet, a sofa) and is **not** a wall: it is counted and said, with the
height of the tallest, never dropped without a word. With no ceiling in the scan a wall
cannot be told from a tall cabinet: nothing is found, and that is said.

**Two faces make a wall.** Two faces parallel within 1 degree, 2 in to 16 in apart,
alongside one another, are the two sides of one wall: its centre lies between them and
its thickness is the distance between them, Assumed, source the scan. A face with no
partner is a wall **seen from one side** (an outside wall scanned from indoors): the
scan does not give its thickness. It is laid only where the person names the wall it is
(`--outside <a wall type>`), with the seen face on the scan's face and the type's
thickness; where no type is named it is said and not laid.

**Square.** Faces within 1 degree of the scan's two main directions are taken as square
to them, and how many is said. The house is laid as the scan lies on its sheet; it is
not turned to the page.

**Corners.** A found wall is carried to meet another where its end comes within 1 ft of
the other's line, so that rooms close.

**Openings.** A stretch of a found wall at least 1 ft 6 in long with no points between
two heights is an opening. Clear from the floor up to a head: a doorway (the scan cannot
tell a door from an open doorway, and says so); it is laid as a door of that width and
head. Clear from a sill up to a head: a window, with its width, sill and head. Each
size is Assumed, source the scan, to the nearest inch. **Not seen is not open:** where
something stands in front of the stretch (within 3 ft, on the scanned side) the wall
behind is hidden, not opened: it is laid as wall and the stretch is said ("not seen for
9' 0\" behind what stands at it: taken as wall, to be checked").

**What is said.** Each wall found (where, how long, how thick or "seen from one side");
each opening; what was squared; what stood short and was not a wall; what was hidden
and taken as wall; and what was laid or not laid, and why. Everything laid is Assumed,
its source "found in scan <name>". Nothing is Known. Laying a second time from the same
scan onto the same level is refused.

**How near** (the demonstration's test, on a made-up scan with 1.5 cm of scatter, strays
and half a degree of lean): a face within 1 in; a thickness within 1/2 in; an opening's
sides, sill and head within 2 in.

The window's half (the offer on the card when a scan is brought in; the found walls
shown before they are laid) is a call of its own once this is judged.

### Call 174. Patrick, 2026-11-22: the builder runs on its own; a sweep by everyone who looks at the building (and what is no longer said to him)

Patrick: "Please stop saying it hasn't been judged on a real machine this sandbox and
images are sufficient for now." "I would like to see a deep sweep of each trade viewing
this including home owner, builder and architect/ engineer"; the program is to be "as
close to perfect clean and at the cutting edge of software as possible before I ever put
it in the desktop"; the builder is to "infer intent and research / review / advise on
your own while providing updates but nothing needed from me", and to run continually.
The same day: one program has shown him three names (BuildOS, plumbline, roomline), and
"a 3d view of a building is needed for everyone" with "the toggle on MEPs as a see
through".

**Read as:**
1. **This machine and its pictures are the proof, for now.** The window is judged on
   pictures taken here. That it has not run on a graphics card is not said to Patrick
   again and is not offered as a reason for anything. (CLAUDE.md's note on where the
   cost went stays as a note to builders.)
2. **Nothing is asked of Patrick.** Scope he has already given is inferred from; what
   is his alone (QUESTIONS.md) keeps its working answer and the build goes on. Updates
   go to him as pictures and a few plain lines.
3. **The sweep.** Each person who opens the building is a reviewer of its own, briefed
   as that person and looking at the same house: a homeowner; a builder (general
   contractor, estimator, project manager); an architect; a structural engineer; an
   electrician; a plumber and an HVAC mechanic; the shell (foundation, framing); the
   envelope (roofing, siding, windows, waterproofing); the interior (drywall, flooring,
   tile, paint, cabinets and kitchens). Each says what it opens the program to see,
   what it sees, what is missing or wrong, and what the best tools of its trade show
   today (from their own pages). A tenth reads what is at the edge of the craft for the
   3D view itself. The first pass reads the pictures on hand and the command line; the
   second walks the window once calls 169 and 171 and the held work are in it. Notes go
   to `research/sweep/`. The lead reads them all and writes one list, in order, in
   STATUS.md; what the list changes is built, judged and shown in pictures.
4. **One name.** The program is BuildOS everywhere a person can see it (it is, on
   screen). The folder, the crates and the two commands are renamed in one change of
   their own once the work in hand has landed (call 115 left them); demonstrations'
   file names are not names of anything and are left.
5. **The centre is the 3D building, seen by each trade.** See through and the switches
   for each trade are the first thing the sweep looks at.

### Call 175. What the sweep found wrong in dollars, and what is decided (2026-11-22)

The sweep (call 174; `research/sweep/`) found figures that are wrong or said twice. The
lead checked each against the code before writing this. Each moves an expectation, so
each is decided here first.

1. **Siding is not bought over the stone.** In the sample whole house and in the starter
   assembly "Exterior wall, 2x6, lap siding over a stone wainscot", lap siding is priced
   on `net_area` while a 3 ft stone wainscot is priced on the same wall's `plan_length`:
   on a 40 ft wall, 120 SF of siding bought behind stone (checked: 329.99 SF of siding
   and 40.00 LF of stone on the south wall). **A wall gains a figure `net_length`: its
   plan length less the widths of its openings that come down to its floor** (a door; a
   window with its sill at the floor). The stone is priced on `net_length` (no stone
   across a doorway); the siding and "Hang the siding" on `net_area - 3 * net_length`.
   The weather barrier, sheathing and insulation stay on the whole wall: they run behind
   the stone. By hand on the sample's south wall if it has no door: stone 40.00 LF,
   siding 329.99 - 120 = 209.99 SF.
2. **Base is not bought across a doorway.** The sample house prices "Base and trim" on a
   room's `perimeter`; the library's base lines went to `base_run` at call 162 and the
   sample was left behind (kitchen 50.25 LF priced, 45.08 LF drawn). The sample's line
   goes to `base_run`.
3. **A fixture set against its own wall is not a clash.** A thing that stands against or
   hangs on a wall (a tub, a water closet, a cabinet, a receptacle in the wall it is set
   in), a cable where it ends at its device, and a branch where it joins its own trunk
   are **set against**, counted and listed apart, and are not in "in the same space".
   The clash test's shapes do not change; what a finding is called does.
4. **A hole is counted once.** A run through a floor or a wall is one hole to cut for
   each floor or wall it passes, not one for each piece of the run: a riser of three
   pieces through one floor is one hole. The list says how many holes, of what size,
   through what.
5. **Two footings, or two walls, drawn on one line are said.** `check` and the takeoff
   name it ("footing 5 lies on footing 1 for 16 LF: counted twice"); it is a caution,
   not a refusal, since an old footing and a new one may truly lie side by side.
6. **A header's depth is not the program's to give.** The frame draws a header and the
   takeoff counts its length (call 137); its depth today comes from a rule of thumb
   carried in the port. The program sizes nothing (call 164): the depth is Unknown
   until typed, the header is drawn at the depth typed or, with none, as an outline,
   and the field list asks it for every opening in a wall said bearing or not said.
7. **The BIM file says existing, demolished and new in its own word.** A wall, door,
   window, floor, roof and footing is written with its common property set's `Status`
   (the standard's own literals, read from the schema page before a line is written),
   and a wall with `LoadBearing` where bearing is said and left out where it is not;
   the program's own text property stays. Reading the program's own file back gives
   the same marks, type names and roof: a round trip that loses them is a defect
   (found: it loses all three). Reading another's file takes `Status` in as the mark,
   Assumed, sourced to the file.
8. **Not a defect, checked:** footing concrete on a closed ring is plan length on the
   centre line times width times depth, which is exact at mitred corners (a 40 by 28
   ring, 24 in wide: outer 42 x 30 less inner 38 x 26 = 272 SF = 136 LF x 2 ft). The
   reviewer's suspected 0.3 CY over is not there.

Not decided here, on the list in STATUS.md: the receptacle count in a bath and a
kitchen (the 210.52(A) count of call 147 is for wall space; (C) counters and (D) the
basin are not built and the count must say so where a room is a kitchen or a bath).

### Call 176. What a wall carries and what it stands on (2026-11-22; the sweep's worst finding)

The engineer, the builder and the framer each found it: a first-floor wall marked to
demolish with a second-floor wall standing on its line, and the program silent. Call 160
asks whether an opened wall is bearing; nothing looks at what is drawn above it.

**Worked from what is drawn, and said to be only that.** For any wall the program works
out, from the plan lines of what is on the level above and the level below:

- **what stands on it**: each wall of the level above whose line lies over this wall's
  (within this wall's thickness, and within 1 degree of its direction), with the length
  they share; the roof, where an eave of a roof plane lies over the wall (a rake does
  not bear on the wall under it, and is not listed); the floor above, by its joists: if
  the joists' direction is said (call 165) and they cross this wall, "the joists of the
  floor above cross it"; if they run alongside it, nothing; if their direction is not
  said, "which way the joists of the floor above run is not said".
- **what it stands on**: a wall of the level below under its line, a footing under it,
  a beam under it (call 164), each with the shared length; and for the length where
  none is found, "nothing found under it for N LF" (on the lowest level: no footing; on
  an upper level: it stands on the floor alone, which is ordinary for a partition and
  is said plainly, not as an alarm).

These are figures of the wall (`carries`, `stands on`), worked in `crates/world`, each
resting on the positions it was worked from (Assumed if they are). Every place they are
shown carries the sentence: "worked from what is drawn above and below this line; it is
not an engineer's finding". The program still sizes nothing.

**Where it speaks up.**
1. A wall marked to demolish, or opened (call 160's rule for opened), with anything
   standing on it: a caution on the wall, in `check`, and a question on the field list
   with the wall's dollars riding: "wall B stands on it for 12 LF: once it is down,
   nothing is found under that wall". A beam drawn under the upper wall's line (new
   work) answers it: the upper wall then stands on the beam.
2. A wall said not bearing with a wall or an eave standing on it: a caution ("said not
   bearing, and wall B stands on it for 12 LF").
3. A wall whose bearing is not said with something standing on it is counted, for the
   window's tint (bearing, not bearing, not said) and its top-bar mark.

Nothing is refused: the drawing may be wrong, or the engineer may have it in hand.
The clash test, the price and every existing figure are unchanged.

Built after the held work of calls 164 and 165 is in the tree (it needs the beam and
the joists' direction).

### Call 177. A trade's view of the building: one switch, the frame drawn, sizes on the work (2026-11-22; the sweep, asked by seven of nine; Patrick: "a 3d view of a building is needed for everyone" and "the toggle on MEPs as a see through")

The window can already show one trade: See through, and a switch each for Mechanical,
Electrical, Plumbing, Framing, Finishes, Ground and Roof. The sweep found what is
between that and a trade opening it every day: the framing picture takes seven
switches; a pipe's size is read only by clicking it; pipes are hairlines in the glass
house; and no floor or roof frame is drawn at all.

**1. Trades, one click each.** At the head of the Show list, a row: **All, Foundation,
Framing, Outside, Plumbing, HVAC, Electrical, Inside**. A trade is a setting of the
switches that are already there, not a second state: pressing one moves the switches
below it, where they can be read and changed, and "All" puts them as they were before
the first trade was pressed.

| Trade | What it sets |
|---|---|
| Foundation | Ground off; footings, pad footings and foundation walls solid; everything above them glass; Sizes on |
| Framing | Framing on, Finishes off, Roof's covering off so the rafters show; walls as their frames; the rest glass; Sizes on |
| Outside | Finishes on, Roof on, See through off; the house as it will be built |
| Plumbing | See through on; Plumbing on, Mechanical and Electrical off; Sizes on |
| HVAC | See through on; Mechanical on, the other two off; Sizes on |
| Electrical | See through on; Electrical on, the other two off; Sizes on |
| Inside | See through off; Roof off; the level looked into, Finishes on |

The status line says what was set, in a sentence ("Plumbing: the rest of the house
shown as glass, sizes on"). The names of the existing switches do not change
("Mechanical" stays the switch; "HVAC" is the trade's button).

**2. Sizes: the work wears its size.** A switch "Sizes" in the Show list. With it on,
each run of a trade that is shown wears a small tag at its middle, facing the viewer:
its size and what it carries, in the words the model already has for it (the run
type's name and the system: "3 in waste", "3/4 in cold water", "16 x 8 supply"); a
footing, a foundation wall, a beam and a post wear their size; a thing set in place
wears its name. The words are the model's (one wording, as one arithmetic): the window
makes none up, and where the model has no trade name for a type it shows the type's
own name. Tags thin out, not pile up: of the pieces of one run end to end only the
longest wears the tag, and a tag that would cover a nearer tag is not drawn. A tag is
on the map by its words, so a demonstration can find it.

**3. The frame drawn.** Under Framing, with the walls' frames: joists and rafters as
the model lays them out (call 165's `members_across`: the window draws the members it
is given and works out none), their rim, beams and posts (call 164); pad footings
under Ground. Drawn only where a line prices them, as every trade's layer is. A floor
whose joists' direction is not said is drawn with no joists and wears the tag "joists:
direction not said".

**4. A run is never a hairline.** In a trade's view a run is drawn no thinner than 2
dots on the screen, however far off: this is how it is drawn and changes no figure;
the tag gives the true size. A vertical drain 3 in and over (a stack) is drawn at its
true size and wears "stack" with its size.

**5. A clash says where.** A clash's row and its mark in the 3D view name the room it
is in (the room whose outline holds the point; "outside any room" where none does),
before east, north and up.

What a clash says to do about it, a structure class of clash ("through a joist") and
the bearing tint (call 176's window half) are calls of their own after this one and
after call 175's "set against" is in.

Built after call 169 and the held work are in the tree. The demonstration
(`demos/tradesw.sh`, which already walks four layers, gains these steps, each the way
a stranger would do it): each trade's button pressed from a house opened cold, the
switches read off the map after it, the status line's sentence, and on the picture by
measurement: with Plumbing, pipe colour present and wire colour and duct colour absent;
the tags "3 in waste" (or the sample's own sizes) found on the map and inside the
window; with Framing, lumber-coloured ink across the floor where no joist was drawn
before (proved red on the picture as committed today); "All" puts every switch back.

### Call 178. What the pictures of call 169 showed: the window made clean (2026-11-22; a reader's notes on every retaken picture, `research/pictures-read-call169.md`, and the lead's own reading)

Call 169's thirteen items are built and their demonstrations pass. A reader then read
every retaken picture cold, and the lead read the ones that mattered. The demonstrations
pass and the window is still not clean: these are decided here, all of them the
builder's to decide (words and look), and go to the window's next pass.

1. **No file word on screen, anywhere.** Found: ".json" in the Layouts column heads;
   "items.csv" and "48 rows read" in a card; "file:" as a label in Open items ("Made-up-
   room, file: made-up-room cut at ..."); "scan file:" before a name. A job is named by
   its title; a table brought in is "the tool chest file" or "the price list" and its
   count is of items or prices, not rows; an open item about a scan says "Scan
   made-up-room, cut 4 ft above the floor: Assumed". The language gate gains these
   patterns for the window's strings (proved red on today's source first).
2. **Existing reads as existing.** Found: a brick wall marked existing is as red as one
   that is not (the wash of call 161 is 0.15 and cannot be seen; its check asks only
   "lighter by 10" and passes). Drawn as drafting draws it: **new work in full;
   existing in halftone** (its own colour mixed half-way to a pale gray, its pattern
   kept); **what comes down dashed in red**. On the plan: new walls filled solid,
   existing walls filled in light gray with their outline kept, demolished walls a red
   dashed outline with no fill. "Existing, shown pale" becomes a switch like every
   other row (off: existing is drawn as new work is). The check: over the wall's patch,
   the colour's strength (the largest channel less the smallest) falls by over a third
   from unmarked to existing, and the spread of brightness stays over 10 (the pattern
   is kept); proved red on today's picture.
3. **The view goes to what was brought in.** A scan, a drawing or a model brought in is
   framed with the job, easing there; it never leaves the person looking at an empty
   sheet (found: a 197 ft scan brought in and nothing on screen but a dash).
4. **A message put away is put away.** The mark on a card, or Esc, clears it from the
   status line of both windows as well. While a card is up the status line does not
   repeat it. A sentence in the status line is never cut inside a word: where it does
   not fit, it ends at the last whole word and an ellipsis, and the card has the rest.
5. **Colour says what a message is.** Done: green. A question the program needs
   answered, or the next thing to type: blue, the window's own accent. A caution:
   amber. Refused, or wrong: red, and only that. (Found: "Say which: take its lines off
   or keep them", "its cost type is still to be typed" and "save first" in red; a scan
   brought in rightly, in amber.)
6. **A card is short and does not sit on what it is about.** Four lines at most, the
   first being what happened; "More" opens the rest. A button in a card looks like a
   button. A scan's card says what was brought in, its size and what to do if the size
   is wrong; the rest is under More. A sentence is said once (found: one sentence twice
   in one card).
7. **The same thing said many times is said once with a count.** Open items with the
   same reason on one thing are one row ("Den: 6 figures wait on its area"); "and N
   more for the same reason" only where the reason is the same.
8. **Nothing under 11 points, nothing fainter than can be read.** The smallest type is
   11; the faintest ink for words a person must read is the panel's secondary ink, not
   the hint ink. A label does not wrap into a stray digit ("Rooms, inside their walls"
   is shortened to "Rooms" with its note under it).
9. **One way to write a figure.** An area is written to two decimals wherever it is
   shown (found: 27.625 SF on a card and 27.62 SF on the sheet beside it); the model's
   sentences use the same writer as the sheet.
10. **A name reads as a name.** A wall type made by the program is named for what it
    is ("6 in wall", not "wall 6 in"); "along wall 6 in Kitchen" is "along wall 6, in
    Kitchen"; an undo names what a person did ("Undone: marked existing"), not the
    program's word for it ("Mark work").
11. **What matters is at the top of a thing's panel.** Its name, its WORK mark, then
    its cost lines each with "Take off" at the end of its row, then as drawn and
    takeoff. (Found: "Take line off" is on no picture; it sits below "more below".)
12. **Left over from call 169, by its builder's own account:** the laser message after
    a tool's sizes is unproved by any step; the smaller lettering for a label too wide
    for its box is unproved; a typed measure works on a new line and not on an existing
    line's quantity; bearing and what is in a wall can be said from the window only as
    Assumed (to make them Known needs the field's measure box to take words: the same
    box, with "looked at in the field" as its source).

The 3D view reading as "a box in fog" on small made-up jobs is the look's own matter
(calls 62, 177 and the trade views) and is not decided again here.

### Call 175, as built (2026-11-22)

The builder finished items 1 to 4 and 7 and stopped, rightly, on 5 and 6. The lead ran
`demos/sweep-money.sh` on its build and read what it changed.

- **Item 5 is withdrawn.** Two footings, or two walls, on one line are already refused
  when drawn, when moved and when a file is read (`bodies_overlap`): they cannot be
  counted twice because they cannot be there. The reviewer's doubled figure came from
  an old footing and a new one drawn beside each other, which is two footings. Nothing
  is changed.
- **Item 6 is changed.** The frame's layout uses a header's depth to decide whether a
  cripple fits above it, and cripples are counted in `studs`, which is priced; making
  the depth Unknown would move a count. So the layout keeps its depth, and the program
  stops presenting it as a size: wherever the header's depth is shown it reads
  "Assumed: a rule of thumb used to lay out the cripples, not a size: an engineer or a
  span table sizes a header", and the field list asks the header's size for each
  opening in a wall said bearing or not said. No count moves. (Owed: built with call
  176, which already walks every opened wall.)
- **Item 7, the roof.** The BIM file writes a roof as its slabs and reads back walls,
  doors, marks, bearing and type names; it does not read a roof back, because the file
  holds the roof's surfaces and not what the program draws a roof from (its footprint,
  pitch and eave height). Writing those into the file as the program's own values and
  reading them back is owed, and is said by the demonstration's last line until it is.
- **A hand figure of the lead's was wrong.** "The same wall with no door: siding 380 -
  120 = 260": with no door the net is 400 - 25 = 375 and the siding 375 - 120 =
  **255.00 SF**. The builder did not bend the program to it; the demonstration checks
  255.
- **Chosen by the builder and kept:** a thing is set against a wall when the wall's own
  list of things against it holds it and they are in each other by no more than 2 in;
  a run and what it is joined to are "joined" within 3 in of the joint (at more, a
  branch drawn a foot back up inside its main would stop being a clash, and it is a
  drawing fault); pieces of one run are one hole where they are joined end to end and
  pass within their widths plus 3 in. A riser can now be drawn at the command line
  (`run --rises-to`).
- **The sample house:** in the same space 11 (was 21), set against 10; 60 holes (were
  71 pieces); job cost $390,174.76 (was $394,677.48: the siding behind the stone, the
  stone across the door, and the base across doorways, $4,502.72).
- **Owed in the window:** it still counts clashes its old way (per piece, mountings
  in); it takes the model's new counts with call 177's pass.

### Call 179. What three judges found in calls 164 to 175, and what is decided (2026-11-22)

Three judges (Sonnet, cold, on the program as committed at 8627a83) tried to break the
scan walls, the money fixes and structure, and the plan set and the wall move. Their
notes are `research/judgment-call173-scan-walls.md`,
`research/judgment-calls-175-164-165.md`, `research/judgment-calls-168-170.md`. Each
finding below was reproduced by its judge from an empty job; the lead read each and
decides what is so. None is waved away.

**A. The scan's walls (call 173).** The finder holds on dense scans of ordinary houses
(an L, three rooms, turned, walls 3 1/2 to 14 in). It fails silently elsewhere, and
silence is the defect.
1. *Too few points.* Under about 40 points a square foot it finds no walls, calls them
   furniture, and does not say why. **It says how thick the scan is and, where that is
   too thin to tell a wall, says so and finds nothing**; its counts that depend on how
   thick the scan is are worked from the scan's own thickness, not fixed.
2. *A wall half hidden vanishes, and the room is left open without a word.* **A face
   hidden by what stands in front of it is not "stood short"**: where things stand
   along a wall, the wall is found from what shows above and between them. **Whatever
   is found, it says whether the walls close**: "the walls found do not close: 2 ends
   meet nothing", with where.
3. *A cabinet bank within 6 in of the ceiling is laid as a wall across the room.* **A
   face that reaches the ceiling but whose two ends meet no wall, with a wall face
   behind it within 3 ft, is "a wall or something tall standing at the wall": it is
   said, and laid only on the person's word** (it is never laid by `--lay` alone).
4. *A sloped ceiling reads as "no ceiling"; with an 8 ft and a 10 ft room the low
   room's walls are furniture.* **A wall is judged against the ceiling over it**, not
   one height for the house; it says "the ceiling is not one height: 8 ft 0 in and 10
   ft 0 in" or "the ceiling slopes", and what it could not do there.
5. *A closet and a thin wall beside a thick one give leaning and phantom walls, and
   laying then refuses all of it with a sentence that names no wall.* **A wall found is
   never laid leaning** (a face is fitted along its whole length or not found); **what
   cannot be laid is named as the find named it ("wall 3 of what was found") and the
   rest is laid**, with what was not said.
6. *An 18 in wall is two 6 in walls.* **Two faces up to 24 in apart with no floor seen
   between them are one wall.**
7. *The wrong measure or the wrong way up gives a 15 ft 6 in ceiling and six walls laid
   at that height.* **A floor to ceiling under 6 ft or over 15 ft, or a house over 150
   ft, is said first as "this is not likely the house: check the measure and which way
   is up", and nothing is laid without `--lay --anyway`.**
8. *A level 9 ft high, Known, gets walls 8 ft high from the scan with no word.* **The
   disagreement is said** and the walls take the scan's height, Assumed.
9. *Small:* doors an inch apart in the noise become two door types (**openings within
   1 in of one another in width and head are one type**); a wall's end printed "-0' 4"
   past the face (**ends are trimmed to the faces they meet**); "PLY", "z up", raw
   level and type marks in what is said (**builder's words; a level and a type by
   name**); a scan far from zero laid thousands of feet off (**it is laid where its
   sheet lies; with no sheet, at its own southwest corner, and it says so**).
Not a defect: a person who types "found in scan" as the source of a field measure has
measured, on their word; the program does not police a person's source.

**B. Moving a wall (call 170), and what may be drawn.**
1. *The floor, the footing, the roof and the wall above are left behind, and nothing
   says so; `check` passes with a room bigger than its floor.* **A moved outside wall
   takes with it: the floor's edge that lay on it, the footing under it, the wall
   above it on the next level (and so on up), and the roof's edge over it.** Each is
   said, as the walls that follow are. What cannot follow is said as left behind, and
   `check` cautions wherever a room reaches past its floor or a wall stands off its
   footing. `--alone` moves the wall only and says what it left.
2. *A room line ending on the moved wall stays put.* **It follows, as a wall that butts
   does.**
3. *A move across another wall is accepted; the house can be turned inside out.* **A
   move that would carry a wall across a wall it does not meet, or past its own far
   side, is refused**, saying which wall is in the way and how far it can go.
4. *`--by` takes feet only; a move west is "moved -4 ft east".* **Every length is typed
   any way the program reads a length elsewhere; a direction is said as a direction
   ("4 ft west"); a wall is named with its level.**
5. *A door or window may be hung past the end of its wall, and two may overlap.*
   **Refused when drawn or moved**, as two walls on one line are: an opening lies
   within its wall, clear of every other opening in it.
6. *A floor drawn inside another floor is accepted and its area added; a beam or a post
   drawn twice in one place is counted twice.* **Refused, as two walls on one line
   are** (two floors of one level may not overlap; two beams, two posts, two pad
   footings may not lie in one place).

**C. Figures.**
1. *The BIM file read back moves every wall* (20 ft comes back 20.4167, 12 ft comes
   back 11.5). **Wrong quantity, unsaid: the worst finding of the three.** A wall
   written by the program is read back on the line it was drawn on, to the same length:
   `takeoff` of the job read back gives the same wall lengths, face areas and counts,
   and the demonstration checks the lengths, not only the counts. What is truly not
   brought back (footings, the roof) stays said.
2. *A window with its sill under 3 ft is taken off the siding twice* (255 SF where 265
   is right). **The siding is the wall's net area less the net area of the wall below
   the top of the stone**: the face up to 3 ft less the parts of its openings below 3
   ft. The stone's length stays `net_length`.
3. *A joist's mark that falls on a notch's edge is counted twice* (an L mirrored: 20
   joists where 19; a U: 26 where 25). **A mark is one joist.** The mirrored L and the
   U join the unit tests with the count by hand.
4. *After a pipe is split, the hole's place is given 1.4 ft off the wall.* **A hole is
   where the run passes through the wall or floor**, and the field list sends a person
   there.
5. *A pipe lying inside a slab is "1 hole to cut".* **A run that lies in a floor or a
   wall along its length is "laid in" it, said and counted apart from a hole.**
6. *A door in a wall to demolish is lost on read-back, with advice the program then
   contradicts.* Brought back with its wall, or said as lost with what to do that
   works.
7. *No question is asked of a beam's ends* (what it bears on) *or of the size the
   starter lines name ("2x10 at 16 in" reads as drawn).* **A beam asks what carries
   each end; a line's size that came from the starter library is Assumed and says "the
   starter library's size, not this job's"**; joists and rafters are named in what the
   clash test leaves out.
8. *Words:* "18 lb/LF" refused as having no unit (**read**); "property sets" and
   "LoadBearing" in what is said of a model brought in (**"its bearing mark", "what the
   model says of each thing"**); a raw figure name in a note (**its words**).

**D. The plan set (call 168).** No dimension or area on any sheet was wrong: every one
was checked by hand against the model. What is wrong is the drawing.
1. *A new door in an existing wall is drawn on the existing and the demolition plans.*
   **An opening is drawn on the sheets its own mark puts it on** (unmarked is new).
2. *A long title runs through its box and the sheet's edge, lettered under 3/32 in;
   small rooms' names, areas and dimensions print over one another ("64-6F").*
   **Lettering is never under 3/32 in at the printed scale and never over other
   lettering or outside its box**: a title that does not fit goes to two lines, then is
   cut at a word with an ellipsis; a room too small for its name and area takes a mark
   and a schedule at the side.
3. *Cabinets, fixtures and pipes are not drawn, and the notes do not say so.* **The
   notes say what the sheet does not show.** Cabinets and fixtures set in place are
   drawn on the proposed plan as their outlines.
4. *No sheet numbers, north arrow, key to the line types, or overall dimension.* **Each
   sheet has its number ("A-1 of 3"), a north arrow, a key (existing, to demolish,
   new), and an overall dimension outside the strings on each side.**
5. *A window hanging past its wall's end is drawn.* Gone with B5.
6. *A file named ".txt" is written as a PDF; an existing file is written over without
   a word.* **The plan set's name ends as a PDF's does or is refused; writing over a
   file is said ("wrote over the plan set that was there").**
Lettering as strokes (not text a reader can search) is left: it is what keeps the
program free of a font's licence and the sheets the same on every machine; said in the
notes of SPEC.md.

### Call 180. What the second judge found in the load path, the moved wall and the figures, and what is decided (2026-11-23)

A judge (Sonnet, cold, on the program as committed at e7f2efa) took what a wall carries
(call 176, never judged), the moved wall (calls 170 and 179 B) and the fixed figures
(call 179 C) to houses nobody had tried. Notes: `research/judgment-calls-176-179BC.md`.
What held: every wall moved out and in on a box, an L and a T matched the hand
figures, as did two-storey chains, partitions, the BIM round trip of angled and marked
walls, and the siding over stone.

**A. What a wall carries (call 176): all or nothing was wrong.**
1. *Walls whose lines are up to a thickness apart read as standing on one another for
   the whole length, even when they only touch; a wall at 2 degrees reads as standing
   on nothing though 14 LF of it lies over the wall below.* **What stands on a wall is
   worked from the two walls' bodies on the plan, not from a gate on angle and offset:
   the upper wall stands on the lower for the length over which at least half the upper
   wall's thickness lies over the lower wall's body.** Where less than half lies over
   it, for that length it is said "beside it, not on it". Any angle.
2. *A beam 18 in below a wall reads "ordinary for a partition".* A beam under a wall is
   under the floor the wall stands on: **a beam in the wall's line whose top is within
   2 ft below the wall's base is under it.** The same for an eave over a wall: **a roof
   is over a wall by the plan alone** (the 2 ft test on height is dropped, here and in
   a move), so a raised eave is not lost.
3. *A wall two levels up, with nothing drawn between, raises no caution when the wall
   under it comes down.* **It is said as what it is: "wall X, two levels up, lies over
   this line with nothing drawn between them"**, on the wall and in the caution.
4. *An interior wall under a ridge, or the high wall of a shed roof, reads "nothing
   found standing on it".* The program works eaves only. **Where a roof plane lies over
   a wall that is not under its eave, it says "a roof is over this wall: whether it
   bears here is not worked (only eaves are)"**, never "nothing".
5. *A wall said bearing that stands on nothing raises nothing.* **A caution and a field
   question: "said bearing, and nothing is found under it for N LF".**
6. *The dollars riding on a wall that comes down leave out the window in it.* **What
   rides is everything priced on the wall and on its openings.** Two walls named alike
   in one sentence are named with their levels.

**B. The moved wall.**
1. *A roof drawn with an overhang, or a slab whose edge is outside the wall, stays put
   and nothing is said; the roof's price does not move ($4,185.92 where $4,829.92 by
   hand).* **A roof's edge goes with the wall it is over (by A2: the plan alone, the
   edge parallel to the wall and outside it by no more than 4 ft with no other wall
   between); a floor's edge goes with the wall it lies along within 2 ft.** And the
   rule that ends the silence: **whenever a moved wall has a floor, a footing or a roof
   on its level or over it that did not go with it, the move says so by name ("roof 1
   was left where it was"), and `check` cautions on a roof or floor that no longer
   reaches, or reaches past, the walls under it.**
2. *A follower that turns can cross a third wall, unseen.* **Every wall the move
   changes is tested, not only the one moved; the move is refused if any would cross a
   wall it does not meet.**
3. *`--alone` leaves the corners open and says only that a footing has no wall.*
   **`--alone` says each corner it opens**, and the room's own figure already says its
   walls do not close.
4. *"4 ft to the east" is refused.* **Read**, with "toward", "to the" and the compass
   words spelled out.

**C. Figures.**
1. *A floor 0.0001 ft out of square gives 18 joists where 19; a house turned and
   written to four places the same.* **A joist's mark that falls within 1/16 in of an
   edge is on the edge**: the count does not turn on the last decimal. The square floor
   nudged and turned joins the tests.
2. *The rim along a 45 degree chamfer is 48 LF where 52.14.* **The rim is the true
   length of every edge the joists' ends meet**, not its length measured square to the
   joists. (For square floors nothing moves.)
3. *The BIM file read back gives a level 9 ft 1 1/8 in high as 9.625 ft, Known.*
   **Wrong and Known: a level's height is written and read back to what it was, and
   everything read from a model is Assumed** (call 175). *A 4 in wall stub can come
   back turned, and a clash of type names drops a 20 ft wall*: a wall is never dropped
   for its type's name (the type is told apart by a number after its name, and said).
4. *Beams lying partly on one another, and pad footings overlapping by half, are
   accepted and counted twice.* **Refused as walls on one line are: for the length two
   beams share a line, and where two pads' areas overlap.**
5. *A door taller than its wall leaves the wall's net area Unknown with no reason.*
   **Refused when drawn**, saying the wall's height.

### Call 181. Walls found on a drawn plan (2026-11-23; call 172's second)

A plan that is a drawing (a PDF with lines of its own, not a scan of paper) is already
read for its lines so that a traced wall can snap to them (order of work, step 2). The
walls are in those lines. As with a scan (call 173), what is found is offered, said in
full, and laid only on the person's word; the finding is geometry in `crates/plans`;
laying uses the wall, door and window the model already has; and silence is the defect
(call 179 A): whatever it cannot tell, it says, with where.

**It needs the sheet's scale.** Nothing is found on a sheet that has not been scaled
(calibrated from one dimension, as a traced sheet is): it says so.

**What a wall is, on a drawing.** Two lines side by side, parallel, a wall's thickness
apart (3 in to 14 in at the sheet's scale), each at least 2 ft long at scale, alongside
one another for that length. Its centre lies between them and its thickness is the
distance between them: Assumed, its source the sheet, since a drawn thickness is the
drafter's, not the wall's.
- **A line across the two lines ends a wall there** (the drawing's own end of one wall
  and start of the next, as where new work meets existing).
- **Three or more parallel lines evenly spaced are not walls** (stair treads, siding,
  hatching, a deck's boards): they are counted and said.
- **A pair that meets no other wall at either end is "in doubt"** (a counter's edge, a
  dimension line and its overall, a beam shown dashed): said, and laid only when taken
  by its number, as a scan's doubtful face is.
- Lettering, dimension lines, the sheet's border and title block are not walls; what
  was left out is counted by kind and said.

**Corners and closing.** Walls are carried to meet where their ends come within a
thickness of one another's line. It says whether the walls found close, and where they
do not.

**Openings.** A break in both lines of a wall, with the wall going on in line beyond
it, is an opening, with its width and where it is. Thin lines across the break inside
the wall's thickness mark a window; none, a door or an open doorway. **A plan does not
give a head or a sill**: openings are laid only when the person gives them
(`--doors "6 ft 8 in"`, `--windows "3 ft" "6 ft 8 in"` for sill and head), each
Assumed, its source "said when the walls were found"; where they are not given, the
openings are said and the walls are laid whole across them, and it says that the
walls then stand across N openings and are priced as if solid until the openings are
set.

**Heights.** A plan gives none: walls are laid to the level's height.

**What is said, and laid.** As call 173: each wall (where, how long, how thick), each
opening, what was in doubt, what was not taken as walls and why, whether the walls
close, and what was laid or not. Everything laid is Assumed, its source "found on
sheet <name>". Nothing is Known. Laying twice from the same sheet onto the same level
is refused.

**How near.** A drawing's lines are exact: on the made-up sheet of the demonstration a
wall's line within 1/2 in, its thickness within 1/8 in, an opening's sides within 1 in.
On a real sheet the program says what it found and a person looks: the demonstration
runs the public farmhouse plan and prints what it found without a hand figure to hold
it to, and says so.

The window's half (the offer on the card when a drawn plan is scaled) is a call of its
own once this is judged.

### Call 182. What the second judge found on the plan set, and what is decided (2026-11-23)

A judge (Sonnet, cold, as an architect and then as a builder) wrote plan sets for an L,
a T, a wing at 30 degrees, three levels, ten rooms and a kitchen, and read every page
as a picture: `research/judgment-call179D-planset.md`. **Every string it checked
matches the model on six jobs, and the room areas agree by hand.** What is wrong is
what a drawing owes a builder.

1. **Nothing is located.** Every string is one wall's own length; nothing says where a
   partition, a door, a window or a run of cabinets is. **Each side of the plan carries
   three strings, as a drafter lays them out: nearest the house, the openings in that
   outside wall, to their centres, from the wall's ends; next, the partitions that meet
   that wall, to their lines; outermost, the overall.** A partition that meets no
   outside wall is located by a string inside the plan from the nearest wall parallel
   to it. A run of cabinets is located by its start from the corner, lettered along its
   wall. The notes say what a string is taken to ("openings to their centers, walls to
   their lines as drawn"). Every string is a chain: its parts add to the string
   outside it, to the sixteenth, and the demonstration checks each by hand.
2. **An ampersand or an accented letter is lettered "?".** **The lettering gains the
   ampersand and the accented Latin letters (a base letter with its mark)**; a letter
   it still cannot draw is lettered as its plain letter where it has one, and both
   `plan-set` and its list say how many could not be lettered and which.
3. **Strings and outlines run through names, areas and figures**, and "lettering
   faults: 0" is said because only lettering against lettering is tested. **Lettering
   is tested against every line on the sheet**; a name moves within its room to clear
   them, and a string that would cross a room's lettering is drawn outside the house.
   What cannot be cleared is a fault and is said.
4. **A fault the program knows of is written without a word.** **`plan-set` says every
   fault it found, in plain words, whether or not the list was asked for.**
5. **On a wing at an angle the strings do not add to the overall under them, and no
   angle is lettered.** **A string runs along its own wall. An overall is drawn on a
   side only where the strings under it are parallel to it and add to it; a wall that
   meets another at other than a right angle has its angle lettered at the corner.**
   Extension lines run to the corners they measure from.
6. **"? SF" is never explained; one sheet says a room is enclosed by nothing while
   lettering an area for it; room lines are not drawn.** **A room with no area is
   lettered "AREA NOT KNOWN" and the notes say why in the model's own words; a room
   line is drawn (a thin long-dashed line) and is in the key; a note never contradicts
   what is lettered.**
7. **The key lists line types the sheet does not use; sheet numbers start again on
   every level; any text is taken as a date.** **The key shows exactly what the sheet
   draws. Sheets are numbered through the whole set (A-101 to A-10n, the levels from
   the lowest up). A date is a date or is refused.**
8. **No scale bar, no title in the file.** **A graphic scale under the plan's scale; the
   file's own title is the job's title.** Lettering stays strokes (call 179 D).

### Call 183. What the second judge found in the scan's walls, and what is decided (2026-11-23)

A second judge (Sonnet, cold) took the fixed finder to houses neither the first judge
nor the builder had tried: `research/judgment-call179A-scan-walls.md`. Held: a
seven-room house's walls, doors, windows, thicknesses and areas; every angle; rooms 120
ft and 6 ft; a missing wall said; a tray ceiling, a beam, a neighbour's wall through a
window; nothing Known. What it broke, worst first, and what is decided. The rule over
all of them is call 179's: silence is the defect, and nothing uncertain is laid without
the person's word.

1. **A face is used twice: a partition's face is paired into one wall and laid again
   as a second wall 2.4 in off it; `check` passes and 4.2 LF is counted twice.** **A
   face belongs to one wall.** After the find, two walls whose bodies share a length
   are one wall or one of them is in doubt: never both laid.
2. **A mirror doubles a 20 by 12 room to 40 by 12, with a phantom wall, "the walls
   found close", and 118 LF laid where 64 is true.** **The house is what can be reached
   from the floor that was walked: what is found beyond a wall with no opening leading
   to it (a mirror's room, a neighbour's wall through glass, a room not walked) is "in
   doubt: beyond wall N with no way to it", said with where, and laid only when taken.**
3. **A soffit over cabinets is laid as a 14 ft 6 in wall; a fireplace's front is laid
   as a wall; neither is in doubt.** **A face seen only above something standing, in
   the plane of that thing's own front (within 3 in), is that thing's top (a soffit, a
   cabinet to the ceiling), not a wall behind it: in doubt. Any face whose two ends
   meet no wall is in doubt, whatever is behind it** (this widens call 179 A3; a wing
   wall is then asked about, which costs a word and never lays a phantom).
4. **In the seven-room house a 10 ft partition with a door, and a closet's door wall,
   are dropped every time; the only word is "7 not straight along the whole length",
   with no place; two rooms become one.** **A wall with a door in it is a wall: a
   short wall is found from its pieces either side of the opening and the head over
   it.** And **every stretch left out is said with where it is (from, to) and how
   long; where leaving it out leaves two rooms open to one another, that is said.**
5. **A garage wall with a 16 ft door is "something hung from the ceiling"; a curved
   wall is laid as three chords and called furniture.** **A wall that is mostly opening
   is a wall with an opening: its head across and its ends make it.** **A curved face
   is said to be curved, with its length, and is not laid: curved walls are not found.**
6. **A flat-ceilinged closet's walls are laid 15 in short without a word, and "the
   ceiling slopes" is said of flat ceilings.** **The ceiling over a wall is read only
   where there are enough points to read it; where there are not, the wall takes the
   house's ceiling and says so. A slope is said only where the ceiling's points fit a
   slope better than a flat by more than their scatter.**
7. **Where the scan is ten times thinner at one end, 2 of 5 walls are kept.** **How
   thick the scan is, is worked place by place; where part of it is too thin to tell a
   wall, it says which part, by where, and finds what the rest allows.**
8. **A scan of two storeys gives duplicate walls and an upper door laid as a window
   with its sill at 9 ft 3 in.** **More than one floor in a scan is said ("the scan
   holds two levels: floors at 0 and 9 ft 3 in") and one is found at a time: the level
   asked for, by its floor.**
9. **Words**: a raw mark where a name belongs in what is said of a find; marks of
   certainty with no legend in the field list.

### Call 184. The plan for "hyper realistic" (2026-11-23; Patrick: "What is your plan for hyper realistic ?")

Call 62 has his words: "editing software for construction that feels like playing a
hyper realistic video game". Where it stands: the engine's light is already on (a sun
with soft cascaded shadows, light from a sky, ambient shadow in corners, a filmic fold
of the range, smoothing over time, relief on brick, siding and shingles). The house
reads as a house. It does not read as real, and the reason is not the engine: it is
**what is drawn**. Walls are flat slabs with a pattern on them; roofs stop at the wall
with no overhang; windows sit flush; inside, floors are bare gray and there are no
ceilings; cabinets are tan blocks; the house stands on a gray plane. A game looks real
because of small things that catch light and cast shadow.

**The rule that keeps it honest (the fixed rule, unchanged):** what the house is
dressed in is what the estimate prices. Every piece of detail below is drawn from a
priced line or a thing set in place, and each new kind of detail arrives with the line
that prices it (the envelope reviewer asked for exactly these lines: fascia, soffit,
gutters, corner boards, casing). Nothing is scenery.

**In this order**, each step shown to Patrick as a before and an after of the same
view, and judged by a visual judge against VISUAL.md as well as by the lead's eye:

1. **Detail that casts shadow, outside.** Eaves that overhang, with fascia, soffit,
   gutters and downspouts; windows and doors set back in the wall with casing and sills
   that stand proud; corner boards; ridge and hip caps and a drip edge; the foundation
   showing below the siding and a line where the ground meets it; lap siding and
   shingles with real thickness up close.
2. **The light finished.** The engine pieces not yet switched on: true reflections of
   the sky in glass and metal, softer sun shadows that sharpen where things touch,
   contact shadows under trim, a glow on bright sky, the eye adjusting when walking
   from outside to in; and a sky with the sun where it would be for the site's north
   and a time of day, so a sun study is one slider.
3. **Materials that do not repeat.** Each finish as a full set (colour, roughness,
   relief) with variation across the wall so brick and siding stop tiling; glass that
   reflects; metal that looks like metal. Made by the program; a public-domain texture
   set may be shipped where making one is worse, recorded in LICENSES.md.
4. **Inside.** Floors in their finish (plank, tile with its grid), ceilings, base and
   casing, cabinets with doors, drawers, toe kick and a counter that overhangs,
   fixtures in their real forms, light through the windows across the floor. Walking
   through a room reads as a room.
5. **The scan as the real house.** The scan's own colored points shown in 3D with the
   model built inside them: existing as it truly is, proposed as modeled, and one
   slider between them. Nothing drawn is more real than the house itself.
6. **The place.** The ground as the site that is priced (lawn, drive, walks) to a
   horizon under the sky. Trees and neighbors stay out (QUESTIONS.md Q22's working
   answer) unless Patrick says otherwise.

Movement stays as call 62 has it (the wheel dives at the pointer, the ground slides
with the hand, nothing jumps), and gains walking with doors that open.

Built by one window builder at a time, after call 178's clean-up is merged; steps 1
and 2 first, since they change every picture.

### Call 185. The outside trim priced: fascia, soffit, gutters, corner boards, casing (2026-11-23; call 184 step 1's model half; the envelope reviewer of the sweep: "drawn but not priced")

Call 184 draws detail only from what is priced. These are the figures and the starter
lines that price the trim an eave, a rake, a corner and an opening have; the window
draws each from its line (a call of its own for the window's half).

**Figures, each worked in the model from what is drawn, Assumed where its positions
are:**
- **An eave's and a rake's `overhang`**: the distance on the plan from the roof's edge
  to the outside face of the wall under it (what a carpenter measures a soffit by).
  The wall under an edge is found as call 180 finds a roof over a wall (by the plan).
  Unknown, with the reason, where no wall is found under the edge.
- **A rake**: the sloped edge of a roof plane at a gable. Its `true_length` is its
  length along the slope. Where the model does not yet hold a rake as a thing of its
  own (it holds eaves, hips and ridges), it is made one, as an eave is.
- **A wall's `outside_corners`**: how many of its two ends are outside corners of the
  ring of walls it belongs to (a corner that points out of the house; an inside corner
  is not one), and **`corner_board_length`**: its height for each of them. A corner is
  counted on both its walls: a corner has a board on each face.
- **An opening's `casing_length`**: a window, twice its width and its height (head,
  two legs and the sill's apron); a door, its width and twice its height.

**Starter lines** (each with no price entered, Division 6 unless said; the library
grows and its counts move by this call):
- on an eave: "Fascia", by `true_length`; "Soffit", `true_length * overhang` in SF;
  the gutter line that is there already; "Downspouts", EA, **with no quantity until a
  person types it** (how many is a judgment the program does not make);
- on a rake: "Rake fascia", by `true_length`; "Rake soffit", `true_length * overhang`;
- on an outside wall's assemblies with siding: "Corner boards", by
  `corner_board_length`;
- on a window and a door: "Exterior casing", by `casing_length`.

The sample house's own lines do not change (its "Casing and trim" by the each stays a
person's choice); nothing existing moves but the library's counts.

### Call 183, as built (2026-11-23)

The builder fixed items 1 to 8 (4 and 7 in part) and left one reading to the lead.

- **Item 2, the reading kept.** Read to the letter, "what is found beyond a wall with no
  opening leading to it is in doubt" would put the room behind every partition with no
  doorway found in doubt, and every house the finder was built on has one. **A wall
  found with both its faces is not a wall in the way: both its sides were scanned, so
  someone stood on each.** What bars the way is a wall of one face (a mirror, glass,
  an outside wall) and a window. What this leaves, and is said in STATUS: a room behind
  a two-faced partition whose door was missed is not said to be beyond.
- **Item 9 is not done**, rightly: the level and the type by their marks ("laid on
  level-0002") are in what existing steps of the demonstrations assert as written. They
  move to names with the window's half of the finders, in one change with those steps.
- **Item 1** is fixed at the rule (after the find, two walls whose bodies share a
  length: one is put in doubt); the judge's own scan that laid a wall twice could not
  be rebuilt in 72 runs.

### Call 186. What the first judge found in the walls of a drawn plan, and what is decided (2026-11-23)

A judge (Sonnet, cold, as an estimator handed a designer's PDF) wrote about sixty sheets
of its own, drawn as architects draw, and ran them: `research/judgment-call181-drawn-plan.md`.
Held: poché, walls as filled shapes, hatching, one giant path, white lines, dimension
strings, grid lines, lettering, furniture, turned sheets, four scales; a 60 by 40 house's
434.67 LF, face and net face and rooms by hand; everything laid Assumed and sourced to
the sheet. What it broke, worst first. The rule over all of it is call 179's.

1. **A beam shown dashed, wall to wall, is laid as a 15 ft wall, not in doubt.** **A
   wall's lines are solid. Two dashed lines are something overhead or hidden (a beam, a
   soffit, cabinets above): said as that, with where, and never a wall.**
2. **A ledge and a closet shelf are laid as walls.** **A line is the face of one wall
   only** (as a scan's face is: call 183): a line already a wall's face cannot pair
   again with a line beside it. **And a pair whose thickness no other wall on the sheet
   has is in doubt** ("12 in where the walls found are 4 1/2 in and 6 in").
3. **A wall section drawn beside the plan at another scale is laid as a wall.** **The
   plan is the largest group of walls that meet one another; walls apart from it are
   "apart from the plan: another plan, a detail or a section", in doubt, said with
   where.** A framed box left out is called what it may be ("a title block, a key, a
   schedule or a detail").
4. **A door beside a window with a pier under 2 ft between them vanishes; in a 60 by 40
   house the front door was lost.** **The least length of 2 ft is of a wall's whole
   line, not of a piece between two openings: a pier is part of its wall however
   short, and both openings are found.**
5. **"Nothing else on the sheet was left out" is printed over a sheet half of which is
   a picture; walls cut off by the page's clip are said to close.** **Where part of a
   sheet is a picture it says so and that nothing was read from it; a wall that runs
   to the edge of what the page shows is said to be cut off there, and the walls are
   not said to close.**
6. **A scale worked from a wrong dimension (1 in = 4 ft 4 3/4 in) draws no word.**
   **A scale that is not one drawings are made at (the architect's and the engineer's
   scales and the metric ones, within 1 percent) is said when the sheet is scaled and
   again when its walls are found, with the nearest ones: a caution, not a refusal** (a
   sheet printed small is at no scale and still true to its dimension).
7. **A door's head of 0 ft, or less, is taken and lays a door 1 in high.** **A head is
   above the floor and above its sill, or it is refused.**
8. **With the sheet not lined up with the job the walls are laid at the page's own
   offsets, unsaid; a sheet's name misspelled gets the scan's refusal.** **It says
   where it lays them and why ("the sheet has not been lined up with the job: its lower
   left corner is taken as 0, 0"); a name that is no drawing of the job and no scan on
   hand says so and lists the job's drawings.**
9. **A wall drawn with several lines a side gives 24 to 36 walls in doubt; a wall drawn
   as one centre line is told "no two lines 3 in to 14 in apart"; lines that overshoot
   every jamb shatter the plan into 14 walls with no openings.** **Lines lying between
   a wall's two outer lines, alongside them, are its layers: one wall, "drawn with N
   lines", said once. A sheet whose walls are single lines is told so ("walls here are
   drawn as single lines: a thickness cannot be read"). A line across a wall that runs
   past its faces by less than the wall's thickness is still a line across.**

### Call 185, as built (2026-11-23)

Built as decided; every hand figure came out. Two choices the builder made and the lead
keeps: the new lines sit inside the assemblies a person already gives a thing (fascia,
soffit and downspouts in the eave's "Gutter" assembly; corner boards in the two
lap-siding wall assemblies; exterior casing in "Window" and "Entry door"; a new "Rake
trim" on a rake), so that giving a wall its siding brings its corner boards with it;
and downspouts are Division 7, beside the gutter. A rake was already a thing. A figure
is given only where a line names it (as `net_length` is), so a job with none of these
lines holds nothing new; `plumbline trim <job>` lists them for a house not yet priced.

### Call 186, as built, and two of its rules narrowed (2026-11-23)

The builder fixed items 1 and 4 to 9 and the first half of item 2, and stopped, rightly,
on two rules of the lead's that existing tests show were wrong as written.

- **Item 2, second sentence, is withdrawn and narrowed.** "A pair whose thickness no
  other wall on the sheet has is in doubt" would put in doubt the one partition of an
  ordinary house (a 4 1/2 in partition in a ring of 6 in walls), which the finder's own
  tests rightly assert is found. **Narrowed: a pair thicker than every other wall found
  on the sheet, and alone at that thickness, is in doubt** (a 12 in closet shelf among
  4 1/2 in and 6 in walls; one thin partition is not). Geometry cannot tell a 6 in
  ledge between two walls from a 6 in partition; that case stays wrong and is said in
  STATUS. Not built yet.
- **Item 3, "apart from the plan", is given its measure.** To the letter it puts the
  second of two houses on one sheet in doubt. **A group of walls is apart from the plan
  only where it is under half the plan's length of wall**: two plans of like size are
  both plans; a section or a detail beside a plan is in doubt. The builder offered this
  as a patch; it is taken, and goes in with the narrowed item 2.
- **Dashed lines on a renovation plan** (a wall to demolish drawn dashed): item 1
  stands. They are said ("something overhead or hidden, or on a renovation a wall to be
  demolished"), counted, and never laid: a person draws or marks a demolished wall
  themselves.

### Call 187. The window first: what the lead changes after Patrick asked whether there is progress (2026-11-23)

Patrick: "i am still concerened that we are not making progess what can you do to
reassure me and confirm there has not been drift or issues with bad looping".

**The audit, from the branch itself** (48f3bdf to 0677db0, 2026-11-22 20:15 to 11-23
14:37 UTC): 33 commits; 23,020 lines added under `crates/` in 78 files, **1,357 of them
in the window's crate (6 percent)**; 6,311 lines added to the decisions, the status
record and the research notes; about half the commits are decisions and records.
Tests 510 to 760 in the command-line crates.

**What is so.**
- No drift from the purpose (call 172): each piece built that day is one of scan in,
  drawn plan in, the 3D building seen by each trade, the renovation, drawings out.
- **But the work is lopsided, the same way it was two days before**: the window, which
  is what a person uses, got 6 percent of the new code. Found walls in a scan and on a
  plan, what a wall carries, beams and posts, the plan set and the priced trim exist
  only at the command line. One builder at a time can work on the window, a window
  pass takes hours on this machine, and command-line work beside it is cheap: so it
  piled up.
- **One loop with shrinking returns**: build, judge, fix, judge on the two wall-finders,
  two rounds each, all on made-up files. Each round found real defects; a third round
  on made-up files would be looping.
- Lost time that was not work: about nine hours in two stalls (a worker restart that
  lost done signals; a usage limit that stopped the window's builder), and a steady
  overhead of check-ins and records.

**Changed from here.**
1. **The window first.** No new command-line feature is started until the window has
   what the command line has, in this order: the window's work in hand (contract-184)
   merged; the priced trim drawn; walls found in a scan and on a plan offered when one
   is brought in; the bearing tint and what a wall carries; the beam, post and pad
   tools; the plan set under Write out. Command-line agents fix only what real files
   show wrong.
2. **No third judgment on made-up files.** The finders are next judged by real files
   (the run in hand) and by a person; a judge is sent only to something no judge has
   seen (the window; the trim).
3. **An update to Patrick is a picture of something he can now do in the window, or a
   figure from a real file.** Not a count of calls.
4. **At most two agents at a time, the window's builder first**, so that the shared
   usage limit does not stop it again; a check-in resumes a stopped agent.
5. **Records are kept short**: a call is written where a meaning or an expectation
   moves, a section of STATUS where something was run; neither for its own sake.

### Call 188. What real files showed, and what is fixed first (2026-11-23)

The first run of real files through the two wall-finders is
`research/real-files-first-run.md`. By call 187, command-line work now fixes only what
real files show wrong. In this order:

**The scan first**, because it is what Patrick brings (a phone's LiDAR), and it failed
worst: no wall found in a room with four clear walls.
1. **A real wall is not flat.** A phone's scan bows and is rough: a wall face is fitted
   along its whole length with a tolerance worked from the scan's own roughness (the
   spread of its floor's points about a flat floor is the measure of it), not a fixed
   one from made-up scans. A stretch that is a wall but for a bow is found, and **its
   bow is said** ("bowed 1 1/2 in over 12 ft"), Assumed like the rest. What is still
   left out and is 6 ft or longer is offered by its number as "likely a wall", to be
   taken or not.
2. **One wall is found once.** Faces within a wall's thickness of one another, alongside
   and facing the same way, are one face (a rough wall scanned in passes); it says how
   many were joined.
3. **Which way is up is checked by shape**: the floor and the ceiling are the two
   largest flat level things; where the other way up fits better, it says so first and
   lays nothing without the person's word, as with the wrong measure.
4. **It says how it is going and stops in good time**: a line of progress on a large
   scan; where a scan would take over a minute it is thinned first, and that is said.
5. **The made-up scans learn from the real ones**: `tools/made-up-scan.py` gains a
   scene made as a phone's mesh is (rough, bowed, uneven, scanned in passes), its
   measures taken from real scans and written beside it; the demonstration walks it.
   No real scan enters the repository.

**Then the plan** (it is nearer right):
6. **More than one drawing on a sheet.** Where the walls found fall into more than one
   group of like size, it says so ("2 drawings on this sheet: about 125 ft and 150 ft
   of wall") and lays none until the person says which (`--plan <n>`), and a person can
   bound the plan (`--within` two corners on the sheet). A roof plan's lines are not
   walls: this is how a person says so.
7. **One wall is found once**: pairs whose bodies overlap are one wall; how many were
   joined is said.
8. **Openings**: glass and shower lines inside a room are not windows (a window is in
   a wall that has a face to the outside or is cut clean through); a tag is not an
   opening.
9. **The sheet's own dimensions**: where the page holds its words as words, the
   dimension strings are read and offered to scale the sheet by, each with where it is.

### Call 188, as built, and call 189: the scan's finder goes to the published method (2026-11-23)

**Call 188 items 1 to 5, as built.** Built and merged: a face fitted as a plane that
may lean and bow, its bow said; passes of one wall joined; which way is up checked by
the area of the two largest level things; progress said and a large scan thinned (40 s
and 75 s became 7 s); a made-up scene made as a phone's mesh is, from measures taken on
three real scans. The builder measured the real scans first
(`research/worker-report-call188.md`): walls rough by 1.7 to 3.1 cm where the made-up
ones had 1.5; leaning up to 2.2 degrees; bowed 1 to 15 cm; scanned in 1.5 to 5 passes
1.4 to 4.4 cm apart. **The lead's measure in item 1 (the floor's spread) was wrong for
the very scan it was aimed at** (floor 1.0 cm, walls 1.7 to 3.4): the builder used the
larger of the floor's spread and the cleanest strong wall's. Two expectations it moved
are kept: a 20 ft hanging beam is now offered as "likely a wall", in doubt (it is never
laid by itself); a test's scene was given a floor and a ceiling (its assertions are as
they were). The first run's "a room with four clear walls" was the first agent's
misreading: it is a cramped furnished room of about eight wall pieces.

**And it is still not right on a real scan.** The small room: nothing laid; 3 offered,
one wall twice, two walls not found. The long room: every side there, one wall twice,
6 ends meeting nothing. The large room: 8 found, one wall wrongly joined across a
notch, one missed.

**Call 189. Why, and what changes.** The finder was the lead's own design (lines voted
from points the whole height of the room), built before anyone had looked at a real
scan. CLAUDE.md: "Do not invent what already has a name." The published method for
exactly this (read for the sweep, `research/sweep/edge.md`, second pass, section 1:
its paper and its permissively licensed code) does it another way, and for the reason
the real scans showed: **furniture does not reach the ceiling.** It keeps only the
points in the top tenth of the room's height, lays them on the plan as a picture of
where points are dense, takes the outlines of the dense parts, makes them straight
lines, and pairs opposite faces into walls; openings are then read in each wall's own
face by where points are missing, with "not seen" told from "open" by what the
scanner could see.

1. **The scan's finder is rebuilt on that method**, ported from the published
   description and the code read open in front of the builder (its licence recorded in
   `plumbline/NOTICE` and `LICENSES.md`; nothing linked that is not permissive; its
   stated measures used and named, and each one the program changes said with why).
2. **What the first finder got right is kept and laid over it**: everything Assumed and
   sourced to the scan; a wall seen from one side has no thickness made up; in doubt,
   and laid only on the person's word; "not seen is not open"; what is left out said
   with where; whether the walls close; the wrong measure and the wrong way up said
   first; nothing Known.
3. **It is judged on real scans first.** The three on hand, by what a person sees in
   each one's top-down picture, and as many more real scans as can be had under terms
   that allow it. The made-up houses are run after, as a check that nothing a person
   would rely on is lost, not as the test.
4. **Until it is right on real scans, the window offers tracing over the cut scan
   first** (which works today) and the finder second, as an offer to be looked at.

**Call 189, as built (2026-11-23).** The published method is ported
(`crates/plans/src/found_189.rs`; Cloud2BIM, MIT, method ported and no code copied;
`NOTICE`, `LICENSES.md`) and reached with `plumbline find-walls ... --by-ceiling`.
**It does not replace the first finder**, because the condition written above (better
on all three real scans) is not met. On the three real phone scans, against what a
person reads in each one's pictures, written down before any code:

| scan | first finder lays | by ceiling lays |
|---|---|---|
| small room | 1 wall, 5 ft too long; 1 missed | the 2 sure walls, right size and place; the other two sides in 2 ft pieces |
| long room | 10 walls, 3 of them not there | 3 walls, all there; the alcove's walls in pieces or missed |
| large room | 5 walls, 2 not there, 1 run across a notch | 2 walls, both there |

By ceiling never laid a wall that is not there and takes a quarter of the time; it is
incomplete, and on the made-up houses it is the same on 28 of 65 and worse on the
rest (a wall's top broken into pieces; it depends on the angle the house lies at). No
expectation moved: the default is the code it was. Its own demonstration step is
`demos/findwalls.sh` step 25.

**Call 190. The scan in the window: the person traces, the scan helps (2026-11-23).**
Three rounds of finding walls in a scan with nobody looking (173, 183 and 188 on the
lead's design; 189 on the published one) and none is right on a real phone scan.
A fourth round is not started. What the real scans did show is which half of the job
a machine does well and which a person does: a person sees in one look which lines
are walls and which are a wardrobe; the scan knows to the half inch where the face
is. So:

1. **Tracing over the cut scan is the path**, and it is the first thing offered when
   a scan is brought in. It works today.
2. **A wall drawn over a scan is set by the scan's face.** Two things, and the second
   is a fault in what tracing a scan does today:
   - *The click lands on the scan.* The lines the scan's points make where it is cut
     are read from the sheet's own picture (so they are there again when the job is
     opened without the scan's file) and are places to pick, exactly as a drawn
     plan's lines are. Only a line at least 2 ft long is one.
   - *The line clicked is the wall's face, not its centre.* A scan shows the face of a
     wall. Today a wall traced over it is laid with its centre on that face, so a
     room 12 ft by 14 ft inside comes out 11 ft 6 in by 13 ft 6 in with 6 in walls:
     155.25 SF where there are 168. Over a scan the Wall tool draws by the face: the
     wall's body lies on the side away from the room (the side with fewer of the
     scan's points), its centre half its thickness beyond the line, corners carried
     out both ways. The window says which side it took and offers "Other side".
     Where the scan shows both faces of a wall the centre goes between them and the
     distance between them is said beside the type's thickness. The saved wall is
     what it always was (a centre line and a type): no file changes. It arrives
     Assumed, "traced by its face over the scan <name>".
   The same choice ("Centerline" or "Face") is on the Wall tool over a drawn plan,
   where it stays on Centerline unless the person changes it: nothing drawn today
   over a plan moves. A click with no scan line near stays where the hand put it and
   says "the scan shows no face here".
3. **The walls the by-ceiling finder is sure of are shown as pale lines to take**, one
   click each, never laid unasked in the window. Only those it would lay by itself
   (two ends met, or both faces seen): on the real scans that set held no wall that
   was not there. A taken wall is Assumed, sourced to the scan, as today.
4. **The verb keeps both finders**; the first is the default; `--lay` is as it was.
5. **What is said of it**: the README and the help say that a scan is traced over
   with the scan's help, and that the finder is an offer. Nothing says a scan is
   turned into walls by itself.

One arithmetic: the line the scan's points make near a place, and the thickness
between two faces, are worked out in `crates/plans` (the finders' own code) and the
window only asks; a line of clicks carried out by half a thickness is worked out in
`crates/world`. The 2 ft in point 2 is the lead's and Assumed: a named constant that
moves only by a written call. Point 3 is built after point 2 has been tried on the
real scans.

**Call 188 items 6 to 9, as built (2026-11-23).** Built by a worker that read the
three real city sheets first and wrote down why each fault happens before any code.

- *Why a wall was found two to four times*: the sheet draws each face of a wall as
  two lines about an inch apart, so one wall is four pairs. A stack whose lines all
  lie within 1 1/2 in of its two outermost is one wall, the outermost pair; it says
  how many pairs were joined. A stack nothing claims is one wall in doubt.
- *The "shower glass" was not one.* It is a door leaf drawn inside a partition whose
  faces run on. A window whose wall is not cut through and has a room on both sides
  is not a window: it is said, with where. **It is not yet found as a door**: owed.
- *More than one drawing on a sheet*: walls within 15 ft of one another are one
  drawing, and **a drawing's size is all the wall found in it, in doubt or not** (the
  lead's ruling when the first build laid a roof plan's lines because the floor
  plan beside it was all in doubt). Drawings of at least half the largest are
  listed, each with how much of it is in doubt, and none is laid until the person
  says which (`--plan <n>`) or bounds it (`--within`). Call 186's rule for a group
  under half is as it was.
- *The sheet's own dimension strings*: read where the page holds them as words,
  listed with where each is and which agree (`--dimensions`); `--scale-by <n>` sets
  the scale by one, Assumed, its source the string; one that disagrees is said with
  what its line measures. "Dimension string" is the trade's own word and is kept.
- *A tag* (a closed shape of short straight pieces) is left out and counted. No tag
  made an opening on any of the three real sheets: the rule is checked on made-up
  tags only.

Changed behaviour, named: on a sheet whose long lines are all parallel, each pair of
neighbouring faces was listed twice and is now listed once (found by a test of this
work; no existing expectation moved). Two of the lead's hand figures were set right
by the worker: the roof plan's 108 LF is its centre line 32 by 22 ft; the sheet with
glass has 5 walls.

On the three real sheets (they are not in the repository): the clean single-plan
sheet lost its 2 false windows and nothing else moved; the second is unchanged; the
sheet with a floor plan and a roof plan now says there are 2 drawings, about 150 ft
(all of it in doubt) and about 129 ft, and lays nothing unasked, where it laid 181 LF
of the wrong lines. **Its floor plan is still not found as walls** (hatching and
cabinet lines out to a foot from each wall): on a sheet drawn that way the program
says so and the person traces.
**Call 184 step 2 (the light), as built, and call 171 item 1 in the window (2026-11-23).**
Built by the window builder, each piece measured against a before picture and kept
only where it was better.

- **Kept: the sun is where the sun would be.** A "Sun" button: time of day and month,
  for a site 40 degrees north (the job holds no site yet, and the card says so). It
  opens at ten in the morning in April. It changes the light and nothing in the job.
  **This moves the light in every picture** (the shadow that lay to the right of the
  house lies to its left), so the colours three window checks looked for moved with
  it: `glassw.sh` (the door's and the roof's colour, both ends of each bound moved by
  as much as the colour moved, so the old light no longer passes) and `tradesw.sh`
  (the shadow's place; and its test for bare lumber gained a condition, because the
  siding in the new light passed the old one). The reason is beside each in the script.
- **Tried and left off, with the measure**: soft shadow edges (no different at a
  small size, smeared at a large one), contact shadows (nothing to see at a window
  head), glow (darkened every surface by a twentieth, or touched only the roof's
  edge against the sky). Their code is not in the tree.
- **Not built**: the eye adjusting between outdoors and in (a room is lit by the same
  sky as the yard, so there is nothing to adjust to until rooms are darker inside).
- **The lead's reading of the pairs**: a little more real, not a transformation. It
  still reads as a model. Call 184 said the light alone would not do it; step 1 (the
  outside trim drawn, casting its own shadow) is where the look changes.
- **A wall moved in the window says what followed it** (call 171 item 1): the Move
  tool uses the model's own move, and its card reads the model's sentences. The saved
  job is byte for byte the command line's.
- **The sheet's clash tables** each say their count by name, and "Laid in a floor or a
  wall" is a table of its own; the window's counts are the model's.

### Call 191. What the judge of the sheet's scale, two drawings and the trim pieces found, and what is decided (2026-11-23)

A cold judge (Sonnet, on the program as committed at d8d74cb, from empty jobs and
sheets it made up; `research/judgment-scale-plan-trim.md`). The lead read every
finding. Each that is so moves what the program does or says, so each is decided
here first.

**A. Laying from a sheet with more than one drawing.**
1. *`--plan <n>` took a box, not the drawing.* A plan shaped like a U (240 LF) with
   another house (120 LF) inside the box round it: `--plan 1 --lay` laid both, 360 LF.
   **`--plan <n>` takes the walls of that drawing and no others**: the ones the
   listing counted. What it lays is what it listed, wall for wall.
2. *Two houses in line, 15 ft apart, became one wall 73 LF long with a "14 ft 6 in
   door".* **A gap wider than 8 ft is an opening only where something is drawn in
   it** (a leaf, a swing, a line across: a garage door is drawn). A bare gap wider
   than that is two walls, and the gap is said with where. No wall is laid across
   white paper.
3. *What makes a drawing.* Walls that meet are one drawing; walls that do not meet
   are of one drawing when they are within 8 ft of one another (the same 8 ft: what
   could be an opening), not 15.
4. **Nothing is laid unasked while much of the sheet is in doubt.** Where the wall
   found outside the largest drawing (set apart, or in other drawings) is together
   at least half of it, nothing is laid until the person says which (`--plan`,
   `--within`), and it says how much is in doubt. This is what keeps a roof plan
   from being laid when the floor plan beside it falls into pieces. A small thing
   beside a plan (a section, under half) is set apart and the plan is laid, as
   call 186 has it.
5. A `--within` box that cuts a wall says which walls it cut. Laying a second
   drawing onto a level that holds the first says what to do about it.

**B. The sheet's scale.**
6. **A scale that is Known is not replaced by a string read off the sheet** without
   the person's word (`--anyway`); when it is replaced the message says what it was
   and what it is now, and it is Assumed.
7. **One string alone is said to be alone**: "1 dimension string: nothing else on
   the sheet to check it against". A scale taken from it carries that caution.
8. **Walls laid from a sheet are said when its scale changes**, as traced walls
   are: each is where the old scale put it and is named.
9. A string of no length ("0'-0"") is not a dimension. Dimension-like strings that
   could not be read (inches alone, metric) are counted and said beside the ones
   that were.

**C. The trim pieces.**
10. **A priced line whose piece is not drawn is always said.** A fascia line priced
    on another figure (a plan length, a typed number) was dropped without a word and
    the totals were said to agree at 62 LF where the estimate prices 88. Every line
    that prices a kind of trim is either drawn or in "Left out" with why, and each
    kind's total is set against all that the estimate prices for it.
11. **No figure is not zero.** A soffit whose overhang is Unknown reads "no figure",
    never "0 pieces come to 0 SF; the lines price 0 SF".
12. **Soffit at a corner where two eaves meet is in both runs, and that is how
    soffit is taken off**: total eave length times the overhang's width, with waste
    added on top (checked 2026-11-23 against published takeoff guides: BYU-Idaho's
    construction estimating text, books.byui.edu/construction_estimat/exterior_finishes_es;
    Angi's and Hunker's soffit guides: none takes the corners out). The judge's hand
    figure took them out (30.25 SF on a hip roof with a 2 ft 9 in overhang; 2.25 SF
    at 9 in). The formula stays. **The eave's figure says it**: how much of the
    soffit is corner counted in two runs.
13. The list prints a quantity as the estimate prints it (2 places), so two totals
    that are the same to the cent do not read as different in the fourth place.

Not fixed here, and said in STATUS.md: a door's casing at a wall's end overlaps the
corner board (a drawing matter, for the window's half).

**Call 191, two points added on reading the builder's report (2026-11-23).**

14. **A wall laid from a sheet rides on that sheet's scale**, as a wall traced over it
    does. The verify-in-field list asks for the sheet's scale to be checked and puts
    dollars on the question; it counted only what was traced by hand, so a house
    laid whole from a sheet by `find-walls --lay` had nothing riding on a scale that
    every one of its lengths rests on. Laid walls, doors and windows are counted.
    This moves a figure on jobs that have walls laid from a sheet (the dollars riding
    on the sheet's scale go up; no price changes). By hand: the 30 by 20 ft house
    laid from its sheet, 100 LF of wall 9 ft high priced at a made-up $3.00 a square
    foot of face: 900 SF, **$2,700.00 riding** on the scale of its sheet, where the
    list had none of it.
15. `plumbline calibrate` names walls laid from the sheet in the same words the
    other verb now uses ("traced over it or laid from its sheet").

And one ruling: the eight lines of `demos/trim.sh` step 6 that read a total as "52
LF" now read "52.00 LF" (point 13: a quantity is printed as the estimate prints it).
The expectation moves by this call; the reason is written beside the lines.
**Call 190 point 2, as built (2026-11-24).** In the window, over a scan's sheet, the
Wall tool is set by the scan's face. Each click lands on a face read from the sheet's
own picture; the wall is laid with its body on the side away from the room and its
centre half its type's thickness off the line clicked; the card says so and "Other
side" lays it on the other. A choice "Centerline | Face" is on the tool (Face over a
scan, Centerline everywhere else: nothing drawn over a plan moved). Where the scan
shows both faces the centre goes between them and the card says how far apart they
are beside the type's thickness. A click with no face near stays where the hand put
it and says the scan shows no face there. The builder's choices, kept: a wall's free
end within about 7 in of a face that crosses it is carried to that face (a scanned
partition stops a couple of inches short of the wall it meets); a wall whose room
side the scan cannot tell is laid to the right of the way it was clicked, and says
so. One expectation moved by this call, the reason beside it: the wall of
`demos/scanw.sh` is 3 in outside the room's face (north 3.75 ft) where it was on it
(north 4 ft), and its check is tighter than it was.

**Call 191, the fixes judged (2026-11-24).** A second judge (Sonnet, cold, on the
program at b207e7b; `research/judgment-fixes-191.md`) ran the first judge's cases
again and went round each fix: fourteen of the fifteen hold, none is unfixed, one is
flawed. What is decided:

16. **Doors and windows laid from a sheet ride on its scale too.** Point 14 said
    "walls, doors and windows" and the build counted the walls only. By hand, on the
    judge's house (40 by 20 ft, an 8 ft opening laid as a door): walls 1,024 SF of
    net face at a made-up $3.00, $3,072.00; the door $500.00: **$3,572.00 riding on
    the sheet's scale, 5 lines**, where it read $3,072.00 and 4. A door priced alone
    on a sheet: $500.00 riding, where it read "no dollars riding on it".
17. **The soffit's corner sentence says what it counts.** On an L-shaped roof the
    five outside corners are in two runs each and the one inside corner is in
    neither; the sentence gave the outside corners and was silent on the inside one.
    It says both, and what they come to: by hand at a 9 in overhang, 5 x 0.5625 =
    2.81 SF counted twice, 0.56 SF at the inside corner counted in neither, so
    length times overhang gives 2.25 SF more than the roof has (four corners' worth,
    as on any roof whose eaves turn square all the way round). **The lead first
    wrote this with six outside corners and 3.38 SF: an L has six corners, five
    outside and one inside. The builder said so and did not bend the program; the
    figures here are the right ones.** (STATUS.md had the inside corner as a thing
    not right; it is now said.)
18. **A figure printed to two places is rounded the way the estimate rounds it**:
    1.125 reads 1.13, not 1.12.
19. A gap with only its jambs drawn is said to have nothing drawn in it "from jamb to
    jamb", not "no line across" (the jambs are lines across its ends).

Not changed, and said: the trim list totals a kind from its lines' unrounded
lengths (four rakes, 53.67 LF) where a person adding the estimate's four printed
lines gets 53.68. How a quantity is kept and rounded on a line is as it was.
**Call 184 step 1, the window's half, as built (2026-11-24).** The window draws every
piece of outside trim the model hands it (`trim::lay`), as a board of the piece's own
width and thickness, casting and taking shadow; a "Trim" switch in the Show list
reads the model's count and belongs to the Outside trade's set; a click on a piece
selects what it trims. The window works nothing out.

The builder found, and the lead rules: **the window was already drawing a fascia
strip, a soffit, corner boards on a sided wall and a casing round every opening,
whatever the estimate priced.** Where the estimate prices a piece and Trim is on,
the model's board is drawn and the old drawing of that thing is not (an eave's dark
strip and plain soffit, a wall's corner boards, an opening's outside casing with its
head cap and sill nose). With Trim off, or where nothing is priced, the old drawing
is as it was, so no existing picture moved. **That last part is not the rule**
(CLAUDE.md: what the house is dressed in is what the estimate prices): a house with
no trim priced still shows trim. It is left for a call of its own, because taking it
away changes every picture of every house and the checks that measure them; it is
written in STATUS.md as not right.

For the look only, and in no quantity: where two corner boards meet, one is carried
3/4 in across the corner so no notch shows; an eave's fascia is carried a board's
thickness past the rake.

### Call 192. A wall under a rake rises to the roof: the gable (2026-11-24)

The pictures of the trim drawn (call 184 step 1) show what now most keeps the house
from reading as one: a wall is a rectangle to its top, and at a gable end the
triangle between the wall's top and the rake is empty. It is also a quantity nobody
has taken off: the siding, sheathing and weather barrier of a gable end are bought
and the estimate has none of them. Decided:

1. **Where a roof's edge along a wall is not an eave, the wall rises to the roof.**
   Under an eave a wall stops at its top and the soffit closes it (as drawn today).
   Under a rake (a gable end; the sloping sides of a shed roof) and under the high
   edge of a shed roof, the wall is carried up to the roof's surface over the wall's
   line. A hip roof has eaves all round and no wall of it changes.
2. **Two figures on such a wall, worked out and never typed**: `gable_area`, the
   area between the wall's top and the roof over its line, from end to end of the
   wall's line; and its top as a line of points (how far along the wall, how high),
   which is what the window draws. Each is Assumed where anything it rests on is,
   and says what it rests on; a wall with no roof over it has no gable and says so;
   a wall under an eave says "none: an eave is over it".
3. **A wall's `net_area` is as it was.** The gable is a figure of its own, and a line
   that buys it says so in its formula (`net_area + gable_area`). The starter
   outside-wall assemblies' lines for what covers the outside of the gable (the
   weather barrier, the sheathing where the assembly has it, the siding and hanging
   it, the outside paint) are priced on `net_area + gable_area`; what is inside the
   house (batts, gypsum board, inside paint) stays on `net_area`: the ceiling is
   level and the gable is above it. **This moves the estimate of every job that has
   a gable end and those starter lines.** The builder lists each existing
   expectation that moves, with the figure before and after, and the lead rules on
   each before it is touched.
4. **The frame of a gable** (its studs cut to the rake) is not counted by this call:
   said in the wall's framing figures as not counted.
5. The window's half (the wall drawn to its top line, dressed as the wall is) is the
   window builder's, after this.

By hand, on the house of "The outside trim priced" (24 by 16 ft on the walls' lines,
walls 9 ft, a gable roof 1 ft outside the lines all round, ridge east-west, 6 in
12; the roof's surface is 9 ft at its edge, so 9 ft 6 in over the north and south
walls' lines and 13 ft 6 in at the ridge, 8 ft in from either):
- The west and the east wall, each 16 ft: the roof over the wall's line runs 9.5 ft
  at each end to 13.5 ft at the middle. `gable_area` = 16 x 0.5 (the strip from the
  wall's top to the roof at its ends) + 16 x 4 / 2 (the triangle) = 8 + 32 =
  **40.00 SF**. Its top: (0, 9.5), (8, 13.5), (16, 9.5). Both ends: **80.00 SF**.
- The north and the south wall: an eave is over each: none.
- The same house under a shed roof (one plane, its eave on the south, rising north
  at 6 in 12, 1 ft out all round): the east and west walls' tops run from 9.5 ft at
  the south end to 17.5 ft at the north: `gable_area` = 16 x (0.5 + 8.5) / 2 =
  **72.00 SF** each; the north wall, under the high edge, rises 8.5 ft along its
  whole 24 ft: **204.00 SF**; the south, under the eave: none.
- Under a hip roof: none on any wall, and the estimate is byte for byte what it was.

### Call 193. What the judge who drove the window found, and what is decided (2026-11-24)

A cold judge (Sonnet) drove the window on the program as committed at 51cf498, alone
on the machine, thirteen runs of its own making (`research/judgment-window-1124.md`):
a wall traced over a scan, a wall moved, the outside trim, the sun. **It found no
wrong money, quantity or place**: every figure it worked by hand matched (a traced
ring 93.99 LF against 94.00; rooms to a tenth of a square foot through a turned
house and a rough scan with cabinets; a moved wall's saved job byte for byte the
command line's; the trim count the model's on three houses; the sun's height at
five dates and hours). What it found is in what the window draws and says. Decided:

A. **The house is dressed in what the estimate prices: trim too.** The window still
   draws corner boards on a sided wall and a casing, head cap and sill round every
   opening where no line prices them (the judge's house with one soffit line read
   "Trim 1" and showed trim on every edge). From this call:
   - a wall's corner boards are drawn only where a line prices them (the model's
     pieces);
   - an opening's outside casing is drawn where a line prices it: by its length,
     the model's pieces; by the each (the starter "Casing and trim at the window"),
     the window's own standard casing, which is a drawing and in no quantity;
     priced by neither, the opening is drawn as the unit alone, its frame, sash and
     glass;
   - a roof keeps an edge of its own thickness (a roof is not a sheet of paper):
     that is the roof, drawn as the roof, and is not fascia. A fascia board and a
     soffit are drawn only where priced.
   This changes the picture of every house that has openings or siding and no trim
   priced. The builder lists each demonstration whose check moves, with what it
   measured before and after, and the lead rules on each before it is touched.
B. **A run of walls is laid one way.** Changing "Centerline | Face" in the middle of
   a run changes the whole run (its walls are laid again, as "Other side" does) and
   says how many moved and how far; corners of one run always meet.
C. **A length typed while tracing by the face is the length along the face**, and
   the card says both where they differ: "10' 0" along the scan's face; the wall is
   10' 3" to its corner".
D. **"Other side" stays with the run it turns**: on the card of the run last laid,
   after the tool is put down, until those walls are changed or another run is laid;
   an Undo of a turn leaves it able to turn again; where it cannot, it says what to
   do instead.
E. **A wall can be moved by pressing on it and dragging**, as well as by a click, a
   glide and a click (the judge's drag moved nothing and said nothing of why).
F. **Words**: the floor's line in the Job panel says which line a floor goes to
   beside a figure that differs from the rooms'; a wall laid across a stretch where
   the scan shows no face says how long that stretch is ("the scan shows no face
   along 3' 0" of it: a doorway, or what the scanner did not see"); a wall's card
   that gave its length before the next wall joined its corner is followed by the
   length it then has; "the new wall: lies along" loses its stray colon; "no face
   here" is not said of a face that a laid wall already stands on.
G. **The scan under the tracing can be seen through**: its cut is drawn lighter than
   a traced wall, and the faces a click will land on are drawn as fine lines while
   the Wall tool is up.
H. The Sun's hour stops at six in the morning and six at night and says so when
   pressed further.

Left as they are, and said in STATUS.md: the sky the same at every hour; the plan
running under the right panel after a move; the roof's edges in their line colours
in the 3D view; the open gable (call 192).

**Call 193, as built (2026-11-24).** Each point was shown on the build before it,
fixed, and shown again in a demonstration step.

- **A.** Corner boards are drawn only as the model's pieces. An opening's outside
  casing is the model's pieces where it is priced by its length, the window's own
  standard casing where a line about casing is priced by the each, and otherwise
  nothing: the opening is its unit alone. The old plain soffit is gone. A roof keeps
  an edge of its own (it is the roof, and is not fascia). **No existing check failed
  because trim is no longer drawn where it sampled**; the sample whole house now
  shows its windows as units alone and no corner boards, because nothing in it
  prices them by a length (its doors' casing is priced by the each and is drawn).
- **B.** "Centerline | Face" pressed in the middle of a run lays the whole run again
  and says how many walls moved and how far.
- **C.** A length typed along the face says both lengths where they differ ("10' 0"
  was typed along the scan's face; the wall is 10' 3" to its corner"). A first wall
  of a run is exactly the length typed; the difference comes when it joins a corner.
- **D.** "Other side" stays on the card of the walls last laid while they stand as
  laid; an Undo of a turn leaves it able to turn again; the refusal says what to do.
- **E.** A wall moves by a press and a drag as by a click, a glide and a click.
- **F.** The sentences: a first click on a face a laid wall stands on says so; a wall
  that grew when the next joined its corner is said at its new length; the floor's
  line says "out to the walls' centerlines"; a wall across a stretch with no face
  says how long that stretch is. The model's own sentence "...: lies along ..." is
  left as it is (a command-line demonstration reads it); the window drops the colon
  where it quotes it.
- **G.** The scan's cut is drawn light, a wall over a scan is filled, and the faces
  are fine lines while the Wall tool is up.
- **H.** The Sun card says when it goes no earlier or no later.

**One expectation moved, by the lead's ruling, and why it was late.** `demos/gablew.sh`
expected "$864.00 is out of the job cost (1 of them had no price yet)"; it is 2. Call
185 added an unpriced "Corner boards" line to the starter lap siding, so two unpriced
lines come off with that assembly; the dollars are as they were. The demonstration
failed the same way on the build before this work: it had not been run on any merge
since call 185. That was the lead's omission in choosing which window demonstrations
to run after a merge. From here every merge runs every window demonstration whose
house uses a starter assembly the merge touched.
