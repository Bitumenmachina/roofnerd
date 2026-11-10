# Decisions

D0: the landscape, checked, and a call per component.

**How each entry was checked (2026-11-10).** Versions, licenses and last-publish dates
come from the crates.io registry API. Last-commit dates and license files were read from a
shallow clone of each source repository. Where neither was reachable the entry says so.
The commands are in STATUS.md under D0.

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
None in the application. Two helper scripts are shell (`demos/d1.sh`, `tools/licenses.sh`);
the demonstration script calls Python to label a diff. They are not part of the program.
