# Getting the existing house in from a scan: what is already there to use (2026-11-21)

Question: a builder starts a renovation with an addition. He has LiDAR scans and plans of
the existing house. What is the least work that gets the existing house into BuildOS so he
can design on it? Search order was the repo, BIMCraft's source (now on this machine), the
earlier surveys, then the web. Nothing here is built; no code was edited.

Earlier work this builds on and does not repeat: `research/lidar-renovation.md` (the format
and accuracy survey behind call 128 and PROPOSALS 32 and 33), and the two reviews
`review/F-research-pointcloud.md` and `review/G-research-capture.md` in the session
scratchpad (a ranked table of 32 point-cloud projects with licences; RoomPlan; Stray Scanner).

## 1. What BuildOS does with a scan today, and where it stops

Files: `crates/plans/src/scan.rs` (the reader and the cut), `demos/scan.sh`, DECISIONS call 128.

- Reads **PLY only** (text, little and big endian), with its own reader. Keeps x, y, z and drops
  colour and normals, and says so. Units and which way is up are said, not guessed
  (`--units m|ft`, `--up z|y`). Refuses past 60,000,000 points.
- Finds the floor and ceiling as the two heights where most points lie, says "floor to ceiling
  reads about 8' 0"", and offers that as the level's height. It does not set it.
- Cuts the cloud at 4 ft above the floor (a band of 2 in) and draws it from overhead as a PNG
  at 24 dots to the foot, added under the level as a drawing already to scale and in the scan's
  own place. Everything traced from it is Assumed, sourced to the scan.
- It is a **command-line verb only** (`plumbline import scan`). The window's Bring in list does
  not have it (STATUS.md ~6890). The scan is not shown in 3D.
- **It finds no walls.** A picture of points has no drawn lines, so the Wall tool cannot snap to
  it; the builder places every wall by eye on the picture.
- It has been run on two made-up scans and on **no real scan** (STATUS.md ~6887). No judge has
  tried it.
- Nothing in the job says a wall is Existing, Demolish or New (PROPOSALS 33, waiting for
  Patrick's word). Without that, a scan is only a faster first tracing: the builder cannot
  design on the existing house, because new work and old work are the same thing.
- IFC in already works (`crates/ifc/src/lib.rs`: IFCWALL, IFCDOOR, IFCWINDOW, IFCOPENINGELEMENT,
  IFCSLAB, IFCSPACE, storeys become levels; IFC2x3 and IFC4). It does not read the IFC4 `Status`
  property (New, Existing, Demolish, Temporary); everything comes in as new work.
- No DXF reader (`grep -rli dxf crates` finds nothing).

So today a scan gives the builder **one sheet to trace by eye**. It stops before any wall,
door or window is found, before it can be seen in 3D, and before existing and new work can be
told apart.

## 2. What BIMCraft already has (checked against its source, which is MIT OR Apache-2.0)

Source: `/home/user/ipedrax/bimcraft`; licence files `LICENSE-MIT` and `LICENSE-APACHE`
there (copyright 2026 IPedrax and the BIMCraft contributors). This is the first candidate to
port, and `PORTMAP.md` / `tools/portmap.sh` is how a port is recorded. The grep the
coordinator gave matches every `.last()` in the tree; the real hits are these.

| What | Where | Size | What it does |
|---|---|---|---|
| Point cloud reader | `crates/core/src/cloud.rs` (module note line 1; `Format` line 36; `read` line 523; PLY line 797; LAS line 928) | 1,370 lines | PTS, XYZ, PLY (text and binary), ASPRS LAS 1.2 to 1.4 **uncompressed**, E57. Units to mm, thinned to at most 5,000,000 points (`MAX_CLOUD_POINTS`), held as f32 about the cloud's centre, at most 16 clouds. `merge` (line 128) joins a scanner's several stations into one. Colour kept (`colours`, line 331). |
| E57 reader | `crates/core/src/cloud/e57.rs` | 961 lines | Own reader from ASTM E2807: XML section, bit-packed packets, every scan placed by its pose. Page checksums not checked. |
| LAZ, RCP, RCS | `cloud.rs` line 54 to 56 | | **Refused with a message** (LAZ: decompress with laszip or PDAL; ReCap formats closed). |
| A point cloud placed in the job | `model.rs` 1270 (`PointCloud` element: cloud, level, at, offset, angle), `commands.rs` 174 and 6447 (`point_cloud` command) | | A scan is held once and placed by an element; moved or turned, never mirrored. |
| Seen in 3D as dots | `view3d.rs` 1257, 1314, 1943 | | The ghost under the model that BuildOS still owes. |
| Snap to scan points on a plan | `snap.rs` 30 (`CUT_BAND` 100 mm), 497 (`cloud_point`), `cloud.nearest` at `cloud.rs` 346 | | The cursor snaps to the nearest scan point within 100 mm of the plan's cut plane. **This is the missing piece for tracing**: the Wall tool lands on the scan's wall instead of by eye. |
| Insert dialog | `ui-egui/src/clouds.rs` | | Pick files, then units, spacing and positioning. |
| Phases (existing, demolished, new, temporary) | `crates/core/src/phasing.rs` (1,347 lines; header line 1; `Status` line 40; `CREATED`, `DEMOLISHED` lines 23 to 24); `commands.rs` 1694, 1842 to 1859 | | Each element has a phase created and a phase demolished; a view has a phase filter that draws each of the four statuses as it chooses. `ifc.rs` 684 and 1617 write the IFC `Status` out; ROADMAP.md line 159 says so. This is PROPOSALS 33, already built in the program BuildOS came from. |
| DXF reader | `crates/core/src/dxf.rs` | 2,341 lines | Layers, entities, blocks, units from `$INSUNITS`, curves flattened. Reads what Polycam and magicplan write as a floor plan. |
| Tests | `crates/core/src/cloud_tests.rs` | 565 lines | |

What BIMCraft **lacks**: any finding of planes, walls, openings or rooms in a cloud (a grep of
`cloud.rs` for plane, ransac, detect, extract finds nothing); LAZ; a cloud used to give a
level its height or a length between two points. Its ROADMAP (line 164) calls the point-cloud
work "L" and says "public specifications only". What it has is **reading, placing, seeing and
snapping**, which is the whole of what a builder needs to trace by hand quickly.

Cost of the port, as a builder's estimate and not tried: the reader is 2,300 lines that stand
alone (they take bytes, give points) and BuildOS's own PLY reader would be replaced by it, so
LAS and E57 come with it. The element, the command and the 3D drawing touch BIMCraft's model
and would be written again against BuildOS's. The snap needs one function over the points of
a cut.

## 3. The web: what closes the rest

Licences below were read at crates.io's own API on 2026-11-21 unless said. "Not verified"
says what I could not reach.

### (a) Readers (Rust, permissive), if BIMCraft's own are not ported

| Crate | Licence | Latest, updated | Notes |
|---|---|---|---|
| `e57` https://crates.io/crates/e57 (repo https://github.com/cry-inc/e57) | MIT | 0.11.13, 2026-06 | Maintained. |
| `las` https://crates.io/crates/las | MIT | 0.11.2, 2026-10 | Maintained. |
| `laz` https://crates.io/crates/laz | Apache-2.0 | 0.14.0, 2026-11 | The only route to compressed LAS. Pure Rust. BIMCraft refuses LAZ. |
| `ply-rs-bw` https://crates.io/crates/ply-rs-bw | MIT | 4.0.1, 2026-07 | Maintained fork; the original `ply-rs` (MIT) stopped in 2020. BuildOS has its own PLY reader, so not needed. |
| `tobj` https://crates.io/crates/tobj | MIT | 4.0.5, 2026-08 | OBJ meshes (Canvas and Polycam write OBJ). |
| `gltf` https://crates.io/crates/gltf | MIT OR Apache-2.0 | 1.4.1, 2024-05 | GLB, which Scaniverse writes. Old but stable. |
| `dxf` https://crates.io/crates/dxf (https://github.com/ixmilia/dxf-rs) | MIT | 0.6.2, 2026-10 | The alternative to porting BIMCraft's `dxf.rs`. |
| `openusd` https://crates.io/crates/openusd | MIT | 0.9.0, 2026-11-20 | USDZ, where RoomPlan's own export lands. F says it needs Rust 1.96; the machine has 1.97.1. Not tried. |
| `kiddo` https://crates.io/crates/kiddo | MIT OR Apache-2.0 | 6.3.1, 2026-11-18 | Nearest-point search, if the snap or a plane fit needs it. |
| `threecrate` https://crates.io/crates/threecrate | MIT OR Apache-2.0 | 0.12.1, 2026-10 | RANSAC plane segmentation. 2,220 downloads: young; benchmark before depending. |

No crate for RoomPlan or Matterport files exists (crates.io lookups for `roomplan` and
`usd-rs` returned nothing). RoomPlan's data is Apple's `Codable` JSON, which `serde_json`
reads; its fields are in F section 3.3 and are not re-verified here.

### (b) Wall and room finding in a cloud

- **Cloud2BIM** https://github.com/VaclavNezerka/Cloud2BIM : Python, **MIT** (LICENSE read at
  `raw.githubusercontent.com/.../master/LICENSE`, "Copyright (c) 2025 Václav Nežerka"; note the
  default branch is `master`). Slices the cloud into horizontal density images, finds walls by
  morphology, openings by gaps, floor and ceiling slabs by a height histogram, and writes IFC.
  Paper: Autom. in Construction 177:106303 (2025). The README says this is the "foundational
  research version" and that a commercial AI version exists from a university spin-off. It is
  the same idea as BuildOS's 4 ft cut, taken further, and permissively licensed, so it can be
  ported with attribution or run beside the program. Known failure, from F: a closed door reads
  as a wall. Not run; wall accuracy on a house not verified.
- Everything else with a mature result is GPL (3DBAG roofer, City3D, PolyFit, CGAL's shape
  detection; the original Efficient RANSAC is "research purposes only"), the learned floor-plan
  models are PyTorch with non-commercial weights, and F lists them with licences. None is a fit.
  F's finding stands: for walls from a raw cloud the algorithm is small and published; the
  code to link does not exist in Rust.
- Open3D (MIT) and PCL (BSD-3) have plane segmentation (RANSAC); read for parameters, not linked.

### (c) Apps that already output walls, doors and windows as data

These beat any point-cloud algorithm for the first pass because the walls are already
walls. Every one is a closed paid app run outside BuildOS; BuildOS only reads their file.

| App | Output | Verified from |
|---|---|---|
| **Metaroom** (Graphisoft, Archicad's maker) | Generates walls, doors, windows and rooms from a LiDAR scan; exports IFC, DXF, GLB, OBJ, USD, PDF and 25 more. "Standalone app, Paid". IFC versions not stated; whether walls and openings come out as separate IFC elements is not stated. | https://www.graphisoft.com/en-ag/partner-solutions/metaroom/ |
| **BIM Scanner** (Solebo, App Store id6740094405) | Recognises walls, doors, windows, openings; exports **IFC 4.0 and 4.3** with typed doors and windows, and DXF. Free to download; full export needs a subscription ($9.99 a week, $29.99 a month, $249 a year per the listing). iPhone 12 Pro or newer, iOS 18. | https://apps.apple.com/app/id6740094405 |
| **Canvas** | CAD and BIM output (Revit, SketchUp, Archicad and others), IFC reported by a forum post and a 2021 magazine, OBJ free; the conversion is priced per square foot ($0.15 a square foot per the store listing). One review says it was renamed Twindo in 2026; unconfirmed. | https://apps.apple.com/us/app/canvas-lidar-3d-measurements/id1169235377 and https://community.graphisoft.com/t5/Collaboration-with-other/3D-scanning-of-as-built-with-Canvas-Structure-Sensor/td-p/255977 |
| **Polycam** floor plan | A **layered DXF** (walls, openings, fixtures, room labels on separate layers), PDF, PNG, SVG, CSV. Business tier. No IFC found. | https://poly.cam/floor-plans and https://learn.poly.cam/hc/en-us/articles/27756102599572 |
| **magicplan** | Detects walls, doors, windows and outlets with LiDAR; exports DXF, ESX, FML, PDF, PNG. **No IFC** on its product page. | https://magicplan.app/product/sketch |
| **Scaniverse** (what Patrick uses) | Meshes as FBX, OBJ, GLB, USDZ, STL; point clouds as PLY and LAS. No floor plan, DXF or walls found; the post is undated. | https://community.scaniverse.com/t/which-formats-can-scaniverse-export-to/50 |
| Room Scanner (open source) | Source only, no store release; writes PDF, DXF, USDZ and keeps the raw RoomPlan scan. **GPL-3.0-or-later**: reference only, never linked. | https://github.com/PatrickFanella/room-scanner |

Apple's RoomPlan itself (iOS 16 and later) gives walls, doors, windows and openings as sized
things with a transform and dimensions, and a confidence. No published accuracy; a forum anecdote
says about 15 cm, and Polycam's own review found doors and windows missed (G section 3.2). An
app that stores its JSON is the shortest path to walls, but only Room Scanner (GPL, not built
for the store) was found to keep it; the free and paid apps above export IFC or DXF instead.

### (d) IFC from scan tools

IFC is the one door that is already open in BuildOS. BIM Scanner (IFC 4 and 4.3) and Metaroom
(IFC) write it; Canvas reportedly does. Cloud2BIM writes IFC from a cloud. IfcOpenShell is
LGPL and runs only as an outside checker (`tools/ifc-check.py`); it is not linked. What
BuildOS's reader does with a scan app's IFC is **not known**: those files were not available
and no judge has tried one. IFC4 carries `Pset_WallCommon.Status`; IFC2x3 does not.

## 4. Ranked recommendation

**1. Put the walls from a scan app in through IFC, and carry Existing in with them.**
Biggest gain, least code: no point-cloud work at all.
- Adopt: BIM Scanner (https://apps.apple.com/app/id6740094405, paid app, IFC 4 and 4.3) or
  Metaroom (https://www.graphisoft.com/en-ag/partner-solutions/metaroom/, paid); either is
  run outside BuildOS. For the status of existing work, port BIMCraft's `phasing.rs`
  (MIT OR Apache-2.0) as PROPOSALS 33 and read the IFC `Status` property both ways.
- Still to write in BuildOS: Existing, Demolish and New on every thing (the saved job changes,
  so Patrick's word and a decision first); the IFC reader reading `Status` and calling
  everything from a scan app Assumed with the scan app as source; drawing and pricing by status.
- What can go wrong: no IFC from either app has been seen by BuildOS; they may give one box per
  room and no true wall thickness, or a wall cut by every door. Phone accuracy drifts (F and
  G: good locally, 10 to 35 cm off over a house walked carelessly); so everything arrives
  Assumed and the verify-in-field list sends a tape to the long walls. Subscription cost.
  Try one real room before anything is built.

**2. Port BIMCraft's cloud reader and its snap, and put Scan in the window's Bring in list.**
This is for the scans he already has, whatever app made them.
- Adopt: `crates/core/src/cloud.rs`, `cloud/e57.rs`, and `snap.rs` `cloud_point` from BIMCraft
  (MIT OR Apache-2.0, `LICENSE-MIT` read on the machine). Optionally `laz` (Apache-2.0) for LAZ.
  Record it in `PORTMAP.md`.
- Still to write: the placed scan in BuildOS's job and file, the ghost in 3D, Scan under Bring
  in, the Wall tool snapping to scan points at the cut, the floor and ceiling heights offered.
- What can go wrong: 5,000,000 points thinned from a house scan loses detail; BIMCraft's reader
  has not been run on a real phone scan here, and neither has BuildOS's; E57 and LAS are
  rare from a phone and common only from a tripod scanner. A scan is placed by hand, so its
  north and its alignment to the plans are a person's job.

**3. Suggest walls from the cut, ported from Cloud2BIM, and read DXF.**
- Adopt: Cloud2BIM (https://github.com/VaclavNezerka/Cloud2BIM, MIT) as the reference for a
  density image, morphology and line extraction on the cut BuildOS already makes. For
  Polycam or magicplan floor plans, port BIMCraft's `dxf.rs` (2,341 lines) or use `dxf`
  (MIT).
- Still to write: line finding, offered as lines to accept (never placed on their own);
  closed doors that read as walls; openings from gaps.
- What can go wrong: weeks and not days; no measure of how good it is on a house until tried;
  a veteran with a tape may be faster than correcting it. Do it only after 1 and 2 are used
  on a real house, as PROPOSALS 32 already says of the same step.

## 5. What I could not verify

- Any scan app's IFC or DXF through BuildOS's reader; no file was at hand.
- What Metaroom, Canvas and BIM Scanner write for each wall (separate elements, thickness,
  openings); store listings and marketing pages only.
- Prices beyond the listings, and that Canvas still goes by that name.
- Whether any app keeps RoomPlan's JSON for the user (only the GPL source app was found to).
- Cloud2BIM's results on a house and its running time; BIMCraft's reader on a real file.
- GitHub's API was refused for Cloud2BIM, so its last commit and stars are unknown; the
  licence was read from the raw file. Rust crate dates are crates.io's.
- The RoomPlan field names (taken from the earlier survey, not re-read at Apple's page).
