# W-229M report (notes as I go)

## Before any code
1. Format: research/gltf.md written from the Khronos Specification.adoc and schema files, fetched whole and quoted (metres; right-handed; +Y up, front +Z; GLB 12-byte header, JSON then BIN chunk, lengths multiples of 4; accessor min/max; baseColorFactor linear; alphaMode BLEND; mode 1/4 from the schema). Not confirmed from text: viewer behaviour for BLEND order, size limits.
2. Solids source: crates/ifc/src/write.rs writes bodies from building::placements (Shape::Wall/Floor/Run/Box), roof::solid_planes, building::room_outlines; the corner rule (J20-6) and run frame (`along`) were inline in write.rs: moved to world/src/solids.rs (corner_carry, along, level_height) and the writer now calls them (BIM file unchanged, diff = only FILE_NAME line). Not carried (named): fitting with no size (BIM file writes place and no body), run with no size is carried as a LINE (BIM file writes only its line), wall with no length, level without elevation, room without outline/height, roof planes that cannot be placed. A thing set down is its BOX (as BIM file), its form (tub, sink) not carried.
3. Axes: (x east, y north, z up) ft -> (x, z, -y) m. South-west low outside corner of first-floor south wall (-0.25,-0.25,0) ft = (-0.0762, 0, +0.0762) m.
4. Test reader: reads GLB by its own table of contents; BIM file box evaluator over plumbline_ifc::step. Checker: Khronos glTF Validator (npm gltf-validator 2.0.0-dev.3.10, Apache-2.0), run only by tools/gltf-check.sh (exit 3 and install line where not installed).
5. BIMCraft: crates/core/src/gltf.rs (514 lines, MIT) writes the 3D scene as glb (node per element, material per colour, (x,z,-y)); its layout is followed, rewritten for these parts; also referenced in commands.rs, lib.rs, ui-egui. 

## Progress
- solids.rs, gltf.rs, CLI `export gltf` written; ifc tests crates/ifc/tests/call229m.rs 12 pass; red runs in review/red.txt (mutations).
