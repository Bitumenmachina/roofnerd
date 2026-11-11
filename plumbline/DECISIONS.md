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
