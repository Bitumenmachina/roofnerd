# Porting roofnerd's low-slope roofing into BuildOS (Division 7's depth)

Written 2026-11-18 by the second builder, after Patrick's ruling (DECISIONS.md call 129):
"BuildOS wins in engine however learned specifics can be ported from roof nerd." This is
the plan, in the order it will be done; each piece is a call of its own, a demonstration
written first with its figures worked by hand, and a judge. What is ported is listed the
way `PORTMAP.md` lists BIMCraft's, by file and function.

## What roofnerd has (src-tauri/core, about 8,800 lines, each rule sourced)

| File | Lines | What it solves | Its source |
|---|---|---|---|
| `taper.rs` | 1,075 | The tapered package: one region per drain (perpendicular bisectors), four planes to an interior drain at one slope, valleys at 45°, the build-up capped by the flashing height | NRCA *Membrane Roof Systems* 2019, ch. 4 pp. 159–168 |
| `cricket.rs` | 522 | Crickets between drains and behind curbs, at twice the field's slope | NRCA pp. 79, 165 |
| `sump.rs` | 118 | A sump at each drain: its side from the bowl, its panels | NRCA |
| `boards.rs` | 239 | Boards laid over the facets: letters, courses, fill, cut and waste, the board bill | the maker's profile (`data/profiles/`) |
| `walls.rs` | 423 | Walls and curbs: the finished roof height along each, the flashing height (8 in above it) | NRCA pp. 159, 251–252 |
| `coping.rs` | 320 | Coping by face height: gauge, stretch-out | SMACNA / NRCA |
| `wind.rs` | 527 | Wind zones (field, perimeter, corner by dimension `a`) and the uplift each asks for | NRCA App. 1 p. 553; ASCE 7 |
| `fastening.rs` | 749 | Fasteners and plates per board by zone, adhesive over the base layer | the maker's table (`data/fastening/dr-05-2024-07.json`) |
| `profile.rs`, `geom.rs`, `scope.rs` | 1,471 | Board profiles, plane geometry and offsets (cavalier_contours), what a condition covers | |

roofnerd keeps its job as *conditions traced on pages* (a field area, a drain, a wall, a
curb, a cricket, a sump). BuildOS keeps one world: a low-slope area (`Kind::LowSlopeArea`)
with its outline, parapets and headwalls as runs. What BuildOS lacks to hold the same
job: a **drain** (and a scupper) as a placed thing on a low-slope area, a **curb** as a
placed thing with a size, and the **roof system** (deck, insulation profile, cover board,
membrane) as the low-slope area's own values.

## The order

**Step 0, done 2026-11-19 (call 138):** the core is in this workspace whole, as
`crates/roofing`, its 122 tests passing. Nothing in the building model uses it yet.

**Step 1, done 2026-11-19 (call 139):** a low-slope area holds its drains and the
thickness at them (`pl::taper`); the solver lays the taper out and the area has six
figures from it (`drains`, `thinnest`, `thickest`, `average_thickness`, `board_feet`,
`valley_length`); `plumbline low-slope` and `plumbline drains` draw them from an empty
job; `demos/taper.sh`. Not yet: the cap at the flashing height (step 4), drawing a drain
in the window, and the drain as a plumbing fixture its storm piping runs from.

1. **Drains and the taper** (`taper.rs`, `profile.rs`, the part of `geom.rs` it needs).
   A drain placed on a low-slope area; the area's tapered package solved in the core:
   regions, planes, valleys, the depth at any point, the thickest and thinnest, the
   average thickness and the board feet; the figures a line can use (`taper_area`,
   `average_inches`, `max_inches`, `valley_length`, `drains`). Demonstration: the NRCA
   figure's own square roof with one drain, worked by hand.
2. **The board bill** (`boards.rs`) over the solved taper: boards by letter and course,
   cut and waste, in the maker's panel sizes; priced by the board.
3. **Crickets and sumps** (`cricket.rs`, `sump.rs`).
4. **Walls, curbs and coping** (`walls.rs`, `coping.rs`): the flashing height along a
   parapet from the finished roof beside it; base flashing in SF; coping's stretch-out
   from the wall's face height (BuildOS has a parapet's stretch-out on the line today).
5. **Wind and fastening** (`wind.rs`, `fastening.rs`): the zones drawn on the area, the
   rating each asks for, fasteners and plates by zone from the maker's table.

Each piece moves no existing number except where its call says so, carries every rule on
its line as Assumed with its source, and is judged before the next starts. The geometry
is ported, not rewritten: roofnerd's tests (`src-tauri/core/tests`) come across with it,
as the oracle the port is checked against.
