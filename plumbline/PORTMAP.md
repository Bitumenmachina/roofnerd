# Port map: the building model against BIMCraft

Every public item of the building model (`crates/world/src/building*.rs` and anything
under `crates/world/src/building/`) is listed here once, as **ported** from BIMCraft or as
**ours**. `tools/portmap.sh` checks it and cannot be passed from memory: a ported row's
line has to hold the symbol it names in BIMCraft's source at the pinned commit, and an
"ours" row has to say what was searched for and where.

BIMCraft: https://github.com/IPedrax/bimcraft at
`f202abd85d1cf165325e2f7b250da0b5e51f5b71` (MIT OR Apache-2.0; used here under MIT).

| item | how | BIMCraft symbol | file:line | searched |
|---|---|---|---|---|
| `P3` | ported | `P3` | `crates/core/src/view3d.rs:34` |  |
| `BEND_DEGREES` | ours |  |  | goes in the re-base (a corner becomes a bend fitting); `grep -n -i bend_deg model.rs runs.rs network.rs`: nothing |
| `MAX_SECTION_FT` | ported | `MAX_CONNECTOR_SIZE` | `crates/core/src/family.rs:396` |  |
| `K_LEVEL` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_WALL` | ours |  |  | as K_LEVEL: `grep -rn pl:: crates/core/src`: nothing |
| `K_OPENING` | ours |  |  | as K_LEVEL: `grep -rn pl:: crates/core/src`: nothing |
| `K_FLOOR` | ours |  |  | as K_LEVEL: `grep -rn pl:: crates/core/src`: nothing |
| `K_ROUTE` | ours |  |  | as K_LEVEL: `grep -rn pl:: crates/core/src`: nothing; becomes the key of a run |
| `K_SIZE` | ours |  |  | as K_LEVEL: `grep -rn pl:: crates/core/src`: nothing; goes, the size moves to a run type |
| `K_PLACED` | ours |  |  | as K_LEVEL: `grep -rn pl:: crates/core/src`: nothing |
| `K_SYSTEM` | ours |  |  | as K_LEVEL: `grep -rn pl:: crates/core/src`: nothing |
| `K_IN_SYSTEM` | ours |  |  | as K_LEVEL: `grep -rn pl:: crates/core/src`: nothing |
| `Level` | ported | `Level` | `crates/core/src/model.rs:131` |  |
| `Wall` | ported | `Wall` | `crates/core/src/model.rs:249` |  |
| `line_ft` | ours |  |  | a line held in the unit it was entered in is ours (call 16); read `units.rs` whole: BIMCraft holds millimetres only |
| `Opening` | ported | `Opening` | `crates/core/src/model.rs:827` |  |
| `Floor` | ported | `Floor` | `crates/core/src/model.rs:567` |  |
| `boundary_ft` | ours |  |  | as line_ft: read `units.rs` whole |
| `Route` | ported | `Run` | `crates/core/src/model.rs:1588` |  |
| `points_ft` | ours |  |  | as line_ft: read `units.rs` whole; goes with the route |
| `Size` | ported | `RunType` | `crates/core/src/model.rs:2467` |  |
| `round` | ported | `round` | `crates/core/src/model.rs:2471` |  |
| `rectangular` | ported | `depth` | `crates/core/src/model.rs:2469` |  |
| `status` | ours |  |  | `grep -c Assumed` and `grep -c 'enum Status'` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
| `round_ft` | ours |  |  | as line_ft: read `units.rs` whole |
| `rect_ft` | ours |  |  | as line_ft: read `units.rs` whole |
| `perimeter_ft` | ours |  |  | the distance round a section, for a duct's surface; `grep -n -i perimeter runs.rs`: nothing; `geom.rs:119` is a ring's |
| `said` | ours |  |  | a size said as it was entered; `grep -rn 'fn said' crates/core/src`: nothing |
| `Placed` | ported | `Component` | `crates/core/src/model.rs:1616` |  |
| `at_ft` | ours |  |  | as line_ft: read `units.rs` whole |
| `size_ft` | ours |  |  | as line_ft: read `units.rs` whole |
| `Discipline` | ours |  |  | `grep -rn 'enum Discipline' crates/core/src`: nothing; BIMCraft's disciplines are tabs of its window (read `docs/disciplines.md`) |
| `SystemInfo` | ours |  |  | a system as an element; BIMCraft's is a name on each run, `model.rs:1605`; `grep -n 'struct System' model.rs`: nothing |
| `CLASSIFICATIONS` | ours |  |  | `grep -rn DOMESTICCOLDWATER .` over BIMCraft: nothing; checked against IFC's enumeration in DECISIONS.md call 29 |
| `InSystem` | ported | `system` | `crates/core/src/model.rs:1605` |  |
| `is_route` | ported | `RUN_CATEGORIES` | `crates/core/src/model.rs:2477` |  |
| `is_opening` | ported | `Door` | `crates/core/src/model.rs:2131` |  |
| `is_placed` | ported | `Component` | `crates/core/src/model.rs:2162` |  |
| `discipline` | ours |  |  | as Discipline: `grep -rn 'enum Discipline' crates/core/src`: nothing |
| `check_level` | ported | `Level` | `crates/core/src/model.rs:131` |  |
| `check_wall` | ported | `Wall` | `crates/core/src/model.rs:249` |  |
| `check_opening` | ported | `Window` | `crates/core/src/model.rs:6666` |  |
| `check_floor` | ported | `Floor` | `crates/core/src/model.rs:567` |  |
| `check_route` | ported | `check` | `crates/core/src/runs.rs:62` |  |
| `check_size` | ported | `RunType` | `crates/core/src/model.rs:2467` |  |
| `check_placed` | ported | `Component` | `crates/core/src/model.rs:1616` |  |
| `check_system` | ours |  |  | as SystemInfo: `grep -n 'struct System' model.rs`: nothing |
| `check_in_system` | ours |  |  | as SystemInfo: a system here is a path to an element; read `model.rs` 1588 to 1612 |
| `check_whole` | ours |  |  | the tree of paths is ours (handoff 7.8); BIMCraft's references are Guids, read `model.rs` 2060 to 2120 |
| `measures` | ours |  |  | figures with a status and a refusal; `grep -c Assumed` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
| `rows` | ours |  |  | a flat reading with flags; read `schedule.rs` 940 to 975: its rows carry no status |
| `Shape` | ours |  |  | what a Bevy window draws; read `runs.rs` 153 to 182: BIMCraft's shapes are prisms and tubes for its own egui view |
| `Placement` | ours |  |  | as Shape: read `runs.rs` 153 to 182 |
| `placements` | ours |  |  | the list of what is not drawn, with why; `grep -c -i 'not drawn'` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
| `TakeoffLine` | ours |  |  | `grep -c -i takeoff` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
| `SystemTakeoff` | ours |  |  | `grep -c -i takeoff` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
| `takeoff` | ours |  |  | `grep -c -i takeoff` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
| `StoreyTotals` | ours |  |  | net wall area by storey; `grep -c -i 'net area'` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0; its wall area is length by height, `schedule.rs:955` |
| `storey_totals` | ours |  |  | as StoreyTotals: `grep -c -i 'net area'` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
