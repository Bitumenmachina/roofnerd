# W-190P report (notes as I go)
- Read contract, call 190, STATUS demo, scan.rs, lib.rs, found_scenes.rs.
- BIMCraft search (grep -rln -i "offset|location.line|justif" crates apps: 162 files, all read by name; the ones that are an offset of a line of points):
  crates/core/src/draw.rs:357 `offset` / :379 `offset_poly` (a polyline of lines and arcs moved `by` to its left, corners where the moved sides meet, MITRE=50 offsets reach, None where a corner is sharper than ~1deg);
  crates/core/src/stairs.rs:182 `offset` (line of points, bisector mitre, no sharp handling);
  crates/ui-egui/src/draw.rs:124 `offset_line` (one line); crates/core/src/walls.rs:919 test `faces_and_layers_lie_where_the_location_line_puts_them` (a wall's location line), tests.rs:3677 (exterior to the left).
  => PORT offset_poly's line part (join of moved sides). Changed: ours cuts a sharp corner square instead of refusing; sharp = under 15 degrees.
- Part 2: red run (stub returns input): 'test result: FAILED. 0 passed; 5 failed; 76 filtered out' (all five carry tests). Green after build: 5 passed. File crates/world/src/editing/carry.rs (+ 'pub mod carry;' in editing.rs). Ported => needs NOTICE line (outside the contract's file list; added one line) and PORTMAP row.
- Part 1 red run (stub returning no faces): 'test result: FAILED. 13 passed; 4 failed' - the four faces tests (house at 4 turns; whole-foot rectangle; chest 1.5 ft + blank; phone) each 'assertion left == right failed ... []'.
- done; patch written; real scan pictures in w190P/faces_*.png
