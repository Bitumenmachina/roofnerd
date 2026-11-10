# Decisions

D0: the landscape, checked, and a call per component.

**How each entry was checked (2026-11-10).** Versions, licenses and last-publish dates
come from the crates.io registry API. Last-commit dates and license files were read from a
shallow clone of each source repository. Where neither was reachable the entry says so.
`tools/landscape.sh` re-runs all of it; its output is kept in `evidence/d0-landscape.txt`.

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
`evidence/d0-landscape.txt`); neither is a claim that it was run here.

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

### 2. The 3D window: build on Bevy 0.20 with `bevy_egui`. Not yet proven.
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
None in the application. Outside it: four shell scripts (`demos/d1.sh`, `demos/units.sh`,
`tools/licenses.sh`, `tools/landscape.sh`) and one Python file (`tools/license_expr.py`,
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
* **A line runs along the outline of each plane it names**, tested at its ends and three
  points between. Touching the outline at the ends is not enough.
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
  inches. A reader multiplying the figures on the page gets the figure on the page. The
  cost: changing only the unit a value is written in also changes the formula text of what
  derives from it. That is the ruling's "that value and what derives from it".
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
