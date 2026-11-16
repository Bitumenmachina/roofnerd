# rgeometry, plexus, fornjot: primary-source findings (checked 2026-11-16)

Method: raw.githubusercontent.com files (README, Cargo.toml, LICENSE), crates.io JSON API, docs.rs pages.
NOT possible: GitHub API / commit pages (the session allows only the roofnerd repo; commits page returned 403,
github MCP refused). So **latest-commit dates are unverified for all three**; release dates come from crates.io/docs.rs.

## Licences (first, as asked)

| Project | Licence | Linkable under the program's rule? |
|---|---|---|
| rgeometry 0.12.0 | Unlicense (public domain) | Yes (Unlicense is on the permitted list) |
| plexus 0.0.11 | MIT | Yes |
| fornjot (fj-* 0.49.0) | 0BSD | Yes, but the project is shut down (see below) |

### rgeometry
- Cargo.toml (https://raw.githubusercontent.com/rgeometry/rgeometry/main/Cargo.toml): `license = "Unlicense"`, `version = "0.12.0"`, `edition = "2024"`.
- LICENSE (https://raw.githubusercontent.com/rgeometry/rgeometry/main/LICENSE) begins: "This is free and unencumbered software released into the public domain." and ends "For more information, please refer to <http://unlicense.org/>". One file only (LICENSE.md, LICENSE-MIT absent: 404).
- crates.io API (https://crates.io/api/v1/crates/rgeometry): every version listed has license "Unlicense".
- Note: the Unlicense is a public-domain dedication; fine for the stated rule, but some corporate reviewers dislike it. Its dependency tree was not audited (see deps).

### plexus
- Workspace members: examples/sphere, examples/subdivide, examples/teapot, pictor, plexus. The crate manifest is https://raw.githubusercontent.com/olson-sean-k/plexus/master/plexus/Cargo.toml: `license = "MIT"`.
- LICENSE at repo root (https://raw.githubusercontent.com/olson-sean-k/plexus/master/LICENSE) begins "The MIT License (MIT)". Single file.
- crates.io and docs.rs (https://docs.rs/plexus/0.0.11/plexus/) both show MIT.

### fornjot
- Workspace Cargo.toml (https://raw.githubusercontent.com/hannobraun/fornjot/main/Cargo.toml): `license = "0BSD"`, `description = "Early-stage b-rep CAD kernel."`, `version = "0.49.0"`.
- LICENSE.md (https://raw.githubusercontent.com/hannobraun/fornjot/main/LICENSE.md) begins "# Zero-Clause BSD License". README: "licensed under the terms of the Zero Clause BSD License (0BSD, for short)."
- docs.rs for fj-export 0.49.0 shows "0BSD".
- Crate names on crates.io are `fj`, `fj-core`, `fj-math`, `fj-export`, `fj-interop`, `fj-viewer`, `fj-window` (there is no crate called `fornjot`: crates.io says "crate `fornjot` does not exist"). All at 0.49.0, published 2024-03-21 (search result of https://crates.io/api/v1/crates?q=fornjot). `fj-kernel` stopped at 0.46.0 (2023-05-08).

## What each is

### rgeometry: computational geometry algorithms (2D)
README (https://raw.githubusercontent.com/rgeometry/rgeometry/main/README.md): "RGeometry is a collection of data types such as points, polygons, lines, and segments, and a variety of algorithms for manipulating them. This crate will be of use to you if you've ever wondered if a point is inside a polygon or if a bank is adequately covered by surveillance cameras."
It is 2D plane geometry: convex hull, triangulation, polygon intersection, visibility, random polygons. docs.rs 0.11.0 (https://docs.rs/rgeometry/0.11.0/rgeometry/algorithms/index.html) modules: `convex_hull`, `intersection`, `kernel`, `polygonization`, `triangulation`, `visibility`, `zhash`; functions `convex_hull`, `segment_intersections`. Data (`rgeometry::data`) structs: `Point`, `Vector`, `Polygon`, `PolygonConvex`, `LineSegment`, `Line`, `Triangle`, `Transform`, `DirectedEdge`, `HalfLine`; trait `Polygon`; macro `polygon!`.
Does NOT do: 3D, meshes, solids, booleans, sweeps, parametric models, any file export. (Absence of 3D inferred from the module/struct lists above; I did not read the source.)

### plexus: polygonal mesh data structures and processing (3D mesh, no CAD)
README (https://raw.githubusercontent.com/olson-sean-k/plexus/master/README.md): "**Plexus** is a highly composable Rust library for polygonal mesh processing." Provides primitive generators (e.g. `UvSphere::new(16, 8).polygons::<Position<E3>>()...triangulate().collect()` into `MeshBuffer`), a half-edge graph `MeshGraph` ("supports arbitrary data in vertices, arcs (half-edges), edges, and faces"), and face operations such as `extrude_with_offset(1.0)`. Optional geometry features for cgmath, glam, mint, nalgebra, ultraviolet. Encoding: PLY **read only** ("`encoding-ply` ... Read Yes, Write No").
docs.rs modules: `buffer`, `geometry`, `graph`, `prelude`, `primitive`.
Does NOT do: parametric/constraint modelling, boolean (CSG) operations (none in README or module list; not verified in source), sweeps along a path, STL/OBJ/3MF/STEP/IFC writing, rendering.

### fornjot: b-rep CAD kernel, **shut down**
README top line (https://raw.githubusercontent.com/hannobraun/fornjot/main/README.md): "This project has been shut down. Its goals were never reached. The rest of this README has been left as it was, for posterity."
Then: "Fornjot is an early-stage and experimental CAD kernel, using boundary representation (b-rep)". Principles: "Focus on mechanical CAD applications, like 3D printing, machining, woodworking, over other use cases such as architecture or electronics." Status: "Currently, Fornjot lacks the features for anything more advanced, making it unsuited for real-world use cases."
Does NOT do: STEP, general booleans (unverified; none listed in the README), architecture/BIM.

## Maturity

| | crates.io latest | latest release date | latest commit | stated status | MSRV |
|---|---|---|---|---|---|
| rgeometry | 0.12.0 | 2026-01-06 (crates.io API); 0.11.0 on 2025-11-05; 21,403 downloads | not verified | none of experimental/unmaintained in README; "a long list of algorithms ... yet to be implemented" | "rust-1.90" (README); edition 2024 |
| plexus | 0.0.11 | **2019-03-01** (crates.io API `updated_at`); created 2017-08-04 | not verified | version 0.0.x; no status line in README | master manifest: `rust-version = "1.81.0"`, edition 2021 |
| fornjot | 0.49.0 (fj-*) | 2024-03-21 | not verified; README says mainline "has not been developed in over a year" and moved to `experiments/` | "shut down" | edition 2024 (workspace) |

Important plexus discrepancy: the crates.io 0.0.11 (2019) is an old API (docs.rs lists deps `arrayvec ^0.4.8`, `cgmath ^0.17.0`, `decorum ^0.1.1`, `derivative ^1.0.2`). The GitHub master manifest also says `version = "0.0.11"` but has 2025-era deps (`thiserror ^2.0.10`, `itertools ^0.14.0`, `glam ^0.29.0`, `nalgebra ^0.33.0`, `theon ^0.1.0`) and `rust-version = "1.81.0"`. So master is far ahead of the only published release; using it means a git dependency, or the 2019 crate. The README on master (not the 2019 crate) is what I quoted. I could not confirm when master was last committed.
Also rgeometry: docs.rs "failed to build rgeometry-0.12.0"; last good docs are 0.11.0 (https://docs.rs/crate/rgeometry/0.12.0). API names above are from 0.11.0.

## Dependencies, GPU or window

- rgeometry (Cargo.toml): `apfp 0.1.0`, `array-init`, `claims`, `num`, `num-bigint`, `num-rational`, `num-traits`, `rand 0.9` (small_rng only), optional `rug` (rug wraps GMP; GMP/rug is LGPL, so it must stay off; it is optional, default off, but check the feature set when depending). `claims` is a normal (non-dev) dependency. No GPU, no window. The repo workspace also has wasm and demo crates, not part of the library crate. Pure CPU, headless-safe. No dependency on Bevy or glam (it has its own `Point`/`Vector`).
- plexus (master manifest): `approx`, `ahash`, `arrayvec`, `derivative`, `itertools`, `num`, `smallvec`, `thiserror`, `typenum`, `decorum` (features approx, serde, std), `theon` (features `lapack`). Optional: cgmath, glam, mint, nalgebra, ultraviolet, ply-rs. No GPU, no window in the library (the `sphere` and `teapot` examples render; they are separate workspace members). The `theon` default-features=false with "lapack" feature: **not checked** what the lapack feature pulls (possibly nalgebra-lapack, a native BLAS/LAPACK build); this is a real build risk and unverified. `geometry-glam` exists for Bevy's glam, but note glam 0.29 vs Bevy's glam version would need to match (not checked).
- fornjot: fj-core, fj-math, fj-export, fj-interop are non-GUI; `fj-viewer` and `fj-window` need wgpu and a window and must be left out. I could not read fj-core's dependency list: its crate Cargo.toml at crates/fj-core/Cargo.toml fetched fine but has no `[dependencies]` section (the files at main may be stripped down after the shutdown); docs.rs dependency page not read. Treat as unverified.

## Use for a library of parametric items

### rgeometry
Not a modelling library. It could help with 2D footprint work (point in polygon, polygon intersection, convex hull of a plan outline), which is relevant to clash tests in plan, but it offers no solids, no 3D, no export. `Polygon`/`PolygonConvex`/`Transform` types (docs.rs 0.11.0). Booleans: none listed.

### plexus
Closest to "mesh from a description": `primitive` generators (sphere, cube-like generators exist; exact list not read), `MeshGraph` face `extrude_with_offset` (an extrusion of a face, from README), `triangulate()`, `MeshBuffer` (index + vertex buffer for drawing; fits a Bevy mesh). A vanity as a box with a cut-out, or a water heater as a cylinder, can be made by hand from these; there are no booleans (so no holes or sink cut-outs), no sweeps along a path (a duct fitting elbow would be built by hand), no parametric layer (the item's sizes would be your own code), and no writer for STL/OBJ/3MF/STEP. PLY read only. For an axis-aligned bounding box you would compute min/max over vertex positions yourself; plexus does not provide a box-clash test.

### fornjot
Real parametric-ish API: sketch, sweep (extrude) and validation (`fj::core::new::operations::{Sketch, Sweep}`, example in https://raw.githubusercontent.com/hannobraun/fornjot/main/models/cuboid/src/lib.rs: `Sketch::new().line_to(..).into_face(Plane{..}, &mut topology)` then `sweep.face_to_solid(...)`; older API `BuildSketch`, `SweepSketch`, `UpdateSketch`). Mesh export through `fj_export::export` (to a path), `export_3mf`, `export_obj`, `export_stl` (docs.rs fj-export 0.49.0). No STEP. README: "Currently, Fornjot lacks the features for anything more advanced". Two parallel APIs (`new::` and old) show the code was mid-rewrite when abandoned. Booleans: not in README or `fj_core` module list (modules: algorithms, geometry, layers, objects, operations, presentation, queries, storage, validate, validation); not verified by reading `operations`. "Robustness through explicitness" (docs.rs fj-core): touching shapes are rejected as errors, which is the opposite of what a clash tool wants.

## Comparison and risks

Target: a library of building items (fixtures, equipment, fittings, furniture) each described by a few sizes, drawn in 3D, edited in a workbench, clash-tested by bounding box.

- That need is small: a typed list of forms (box, cylinder, elbow, ...) from sizes, mesh generation, an axis-aligned box. None of the three is needed for it; the program already has `crates/world/src/building/forms.rs` (per CLAUDE.md) doing this. Bevy's own primitives (`Cuboid`, `Cylinder`, extrusion of a 2D shape) cover drawing.
- plexus (MIT) fits "mesh from a description" best, and `MeshBuffer` maps to a drawing buffer. Risks: only release is 2019 and its API differs from the README on master; git dependency required for the current code; `theon` lapack feature unverified; no booleans, no export, nothing for a bounding box; last commit unverified.
- rgeometry (Unlicense) fits only plan-view (2D) polygon work, e.g. footprints/room outlines, hull, polygon intersection. Not 3D items. Maintained recently (0.12.0, 2026-01-06); latest docs build failing on docs.rs; optional `rug` is LGPL-based and must stay off.
- fornjot (0BSD) is the only one with sketch+sweep and mesh export (STL/OBJ/3MF), but it is shut down, 0.49.0 is March 2024, the README calls it unsuited to real-world use and mechanical-CAD-focused; fj-core's dependency list could not be verified. Taking on an abandoned kernel for a few box-and-cylinder forms is poor value. Do not choose it.
- Recommendation from the evidence: none of the three should be linked now. If one is wanted for study or later use: plexus for mesh structure (verify the git master builds headlessly and what `theon`/lapack pulls in), rgeometry for plan-view polygon tests. All three licences pass the rule; the issue is fit and maintenance, not licence.
- Unverified: latest commit dates (all three); fornjot dependency tree; whether any has booleans (none documented in the pages read); whether plexus master compiles against Bevy 0.20's glam.
