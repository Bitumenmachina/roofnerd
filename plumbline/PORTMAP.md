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
| `K_LEVEL` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_WALL` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_WALL_TYPE` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_OPENING` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_OPENING_TYPE` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_FLOOR` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_FLOOR_TYPE` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_SEGMENT` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_SEGMENT_TYPE` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_FITTING` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_CONNECTORS` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_ROOM` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_PLACED` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_SYSTEM` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `K_IN_SYSTEM` | ours |  |  | the saved job's `pl::` keys are this program's (handoff 7.9); `grep -rn pl:: crates/core/src`: nothing |
| `Discipline` | ours |  |  | `grep -rn 'enum Discipline' crates/core/src`: nothing; BIMCraft's disciplines are tabs of its window (read `docs/disciplines.md`) |
| `SystemInfo` | ours |  |  | a system as an element with a discipline and a classification; read `model.rs` 1588 to 1612 (a run carries its system's name) and `electrical.rs` 70 to 95 (a `DistributionSystem` is a row of electrical settings: phases, wires, voltages); `grep -n 'struct System' model.rs`: nothing |
| `CLASSIFICATIONS` | ours |  |  | `grep -rn DOMESTICCOLDWATER .` over BIMCraft: nothing; checked against IFC's enumeration in DECISIONS.md call 29 |
| `InSystem` | ported | `system` | `crates/core/src/model.rs:1605` | a run's system; here a component of its own pointing at a system element |
| `is_segment` | ported | `RUN_CATEGORIES` | `crates/core/src/model.rs:2477` |  |
| `is_opening` | ported | `Door` | `crates/core/src/model.rs:2131` |  |
| `is_placed` | ported | `Component` | `crates/core/src/model.rs:2162` |  |
| `discipline` | ours |  |  | as Discipline: `grep -rn 'enum Discipline' crates/core/src`: nothing |
| `check_system` | ours |  |  | as SystemInfo: `grep -n 'struct System' model.rs`: nothing |
| `check_in_system` | ours |  |  | as SystemInfo: a system here is a path to an element; read `model.rs` 1588 to 1612 |
| `check_whole` | ours |  |  | the tree of paths is ours (handoff 7.8); BIMCraft's references are Guids, read `model.rs` 2060 to 2120 |
| `measures` | ours |  |  | figures with a status and a refusal are ours; four of BIMCraft's rules are applied inside it and cited where they are used (the header of `building.rs` lists them); `grep -c Assumed` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
| `rows` | ours |  |  | a flat reading with flags; read `schedule.rs` 940 to 975: its rows carry no status |
| `Shape` | ours |  |  | what a Bevy window draws; read `runs.rs` 153 to 182: BIMCraft's shapes are prisms and tubes for its own egui view |
| `Placement` | ours |  |  | as Shape: read `runs.rs` 153 to 182 |
| `placements` | ours |  |  | the list of what is not drawn, with why; `grep -c -i 'not drawn'` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
| `TakeoffLine` | ours |  |  | read `schedule.rs` 368 to 398 and 1036 to 1100: BIMCraft's pipe and conduit schedules carry System, Length and Diameter and can be sorted, grouped and totalled, which is the substance of a takeoff by system and size. Ours is the status on every line, the run with no size kept on a line of its own, and membership of a system that is an element; `grep -c Assumed` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
| `SystemTakeoff` | ours |  |  | as TakeoffLine: read `schedule.rs` 368 to 398 and 1036 to 1100 |
| `OpenEnd` | ours |  |  | an open end said for a person, by system; read `network.rs` whole: its `open` is a list of node numbers |
| `Continuity` | ours |  |  | the network read by system; read `network.rs` whole: it has no system as an element to read by |
| `takeoff` | ours |  |  | as TakeoffLine: read `schedule.rs` 368 to 398 and 1036 to 1100; the code is new, the idea of a grouped schedule by system is BIMCraft's and everybody's |
| `storeys_of` | ours |  |  | the storey an element is under in the tree of paths; read `model.rs` 2060 to 2120: BIMCraft's is a `level` Guid on each element |
| `StoreyTotals` | ours |  |  | a storey's walls, floors and rooms added up, each total with a status and what it leaves out. BIMCraft works a wall's net side area itself (`ifc.rs:731`, gross less each opening cut to the wall's height) and its schedules can group and total a field (`schedule.rs` 1036 to 1100); what it has no word for is a total that is flagged: `grep -c Assumed` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs ifc.rs: 0 |
| `storey_totals` | ours |  |  | as StoreyTotals; the wall net area it adds up is worked in `measures`, which refuses an opening that does not fit where BIMCraft cuts it to the wall (`ifc.rs:731`): `grep -c Assumed` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs ifc.rs: 0 |
| `Level` | ported | `Level` | `crates/core/src/model.rs:131` |  |
| `COMPUTATION_HEIGHT_FT` | ported | `CUT_HEIGHT` | `crates/core/src/model.rs:43` | 1200 mm there, 4 ft here (DECISIONS.md call 29) |
| `Location` | ported | `Location` | `crates/core/src/model.rs:352` |  |
| `label` | ported | `label` | `crates/core/src/model.rs:387` | `Location::label` |
| `Function` | ported | `Function` | `crates/core/src/model.rs:408` |  |
| `Layer` | ported | `Layer` | `crates/core/src/model.rs:421` |  |
| `WallType` | ported | `WallType` | `crates/core/src/model.rs:2417` |  |
| `FloorType` | ported | `FloorType` | `crates/core/src/model.rs:2426` |  |
| `OpeningType` | ported | `OpeningType` | `crates/core/src/model.rs:2493` |  |
| `Wall` | ported | `Wall` | `crates/core/src/model.rs:249` |  |
| `tall` | ours |  |  | a wall built in code with its usual fields; `grep -n 'fn tall' model.rs`: nothing |
| `up_to` | ours |  |  | as tall: `grep -n 'fn up_to' model.rs`: nothing |
| `line_ft` | ours |  |  | a value held in the unit it was entered in is ours (call 16); read `units.rs` whole: BIMCraft holds millimetres only |
| `Opening` | ported | `Opening` | `crates/core/src/model.rs:827` |  |
| `Floor` | ported | `Floor` | `crates/core/src/model.rs:567` |  |
| `boundary_ft` | ours |  |  | a value held in the unit it was entered in is ours (call 16); read `units.rs` whole: BIMCraft holds millimetres only |
| `Room` | ported | `Room` | `crates/core/src/model.rs:852` |  |
| `point_ft` | ours |  |  | a value held in the unit it was entered in is ours (call 16); read `units.rs` whole: BIMCraft holds millimetres only |
| `MAX_SECTION_FT` | ported | `MAX_CONNECTOR_SIZE` | `crates/core/src/family.rs:396` | the largest size a section may be; 2000 mm there, 20 ft here for a duct |
| `Justify` | ported | `Justify` | `crates/core/src/model.rs:1551` |  |
| `SegmentType` | ported | `RunType` | `crates/core/src/model.rs:2467` |  |
| `round` | ported | `round` | `crates/core/src/model.rs:2471` |  |
| `rectangular` | ported | `depth` | `crates/core/src/model.rs:2469` |  |
| `status` | ours |  |  | `grep -c Assumed` and `grep -c 'enum Status'` in each of model.rs walls.rs runs.rs network.rs rooms.rs schedule.rs family.rs: 0 |
| `round_ft` | ours |  |  | a value held in the unit it was entered in is ours (call 16); read `units.rs` whole: BIMCraft holds millimetres only |
| `rect_ft` | ours |  |  | a value held in the unit it was entered in is ours (call 16); read `units.rs` whole: BIMCraft holds millimetres only |
| `perimeter_ft` | ours |  |  | the distance round a section, for a duct's surface; `grep -n -i perimeter runs.rs`: nothing; `geom.rs:119` is a ring's |
| `said` | ours |  |  | a size said as it was entered; `grep -n 'fn said(' model.rs runs.rs family.rs`: nothing (`sketch.rs:221` has a `said` that says a sketch's faults, another thing) |
| `Segment` | ported | `Run` | `crates/core/src/model.rs:1588` |  |
| `offsets_ft` | ours |  |  | a value held in the unit it was entered in is ours (call 16); read `units.rs` whole: BIMCraft holds millimetres only |
| `Placed` | ported | `Component` | `crates/core/src/model.rs:1616` |  |
| `at_ft` | ours |  |  | a value held in the unit it was entered in is ours (call 16); read `units.rs` whole: BIMCraft holds millimetres only |
| `size_ft` | ours |  |  | a value held in the unit it was entered in is ours (call 16); read `units.rs` whole: BIMCraft holds millimetres only |
| `angle_deg` | ours |  |  | an angle that may be Unknown; read `model.rs` 1616 to 1632: BIMCraft's `angle` is a bare number |
| `FittingKind` | ported | `bend` | `crates/core/src/family.rs:133` | the kinds of a pipe fitting, `family.rs` 132 to 138 |
| `title` | ours |  |  | a fitting's kind said in a takeoff; read `family.rs` 119 to 147: its kinds are words of its file format and their IFC types |
| `Fitting` | ported | `PipeFitting` | `crates/core/src/family.rs:37` | a component of the pipe-fitting category there; an element of its own kind here |
| `ConnectorKind` | ported | `ConnectorKind` | `crates/core/src/family.rs:350` |  |
| `joins` | ours |  |  | which kind of run meets a connector; read `network.rs` whole: its network is of pipes only |
| `for_run` | ours |  |  | as joins: read `network.rs` whole |
| `Domain` | ported | `Domain` | `crates/core/src/family.rs:369` |  |
| `Flow` | ported | `Flow` | `crates/core/src/family.rs:382` |  |
| `Connector` | ported | `Connector` | `crates/core/src/family.rs:430` |  |
| `Connectors` | ours |  |  | the connectors as a component on the placed thing; read `family.rs` 430 to 475: there they are a family's list |
| `MAX_CONNECTORS` | ported | `MAX_CONNECTORS` | `crates/core/src/family.rs:698` | 8 a family there, 64 a thing here; the change is in the header of `building/model.rs` |
| `check_level` | ported | `computation_height` | `crates/core/src/model.rs:6498` | a level's elevation and computation height are usable numbers (`model.rs:6497`, `6498`); a storey height above zero and the `Val` rules (a unit, a status, a source) are ours |
| `check_room` | ported | `point` | `crates/core/src/model.rs:6713` | a room's point is a usable point and its number is text (`model.rs:6713`, `6715`); the rest is the `Val` rules |
| `check_wall_type` | ported | `MAX_LAYERS` | `crates/core/src/model.rs:481` | and the layers adding up to the thickness, `model.rs:6245` |
| `check_floor_type` | ported | `MAX_LAYERS` | `crates/core/src/model.rs:481` | as check_wall_type |
| `check_opening_type` | ported | `size` | `crates/core/src/model.rs:6673` | an opening's width and height are above zero (`model.rs:6673`, `6674`; `size`, `7593`); the rest is the `Val` rules |
| `check_wall` | ported | `top_level` | `crates/core/src/model.rs:6585` | a top held to a level or a height tall; and too short to be a wall, `model.rs:7838`. Not taken: refusing a wall whose top is not above its base (header) |
| `check_opening` | ported | `sill` | `crates/core/src/model.rs:6691` | a sill on or over the wall's base |
| `check_floor` | ported | `boundary` | `crates/core/src/model.rs:6603` | a floor has a known type and a boundary of at least three usable points (`model.rs:6601`, `6603`, `7788`); that the boundary does not cross itself is this program's own outline rule (`geom::outline_fault`), and the rest is the `Val` rules |
| `check_segment_type` | ported | `round` | `crates/core/src/model.rs:6266` | a round section has one size; the rest is the `Val` rules |
| `check_segment` | ported | `check` | `crates/core/src/runs.rs:62` | too short to be a run |
| `check_placed` | ported | `point` | `crates/core/src/model.rs:7105` | a component's position, angle and offset are usable numbers (`model.rs:7105` to `7107`); what it is as free text (BIMCraft names a family) and the `Val` rules are ours |
| `check_fitting` | ours |  |  | a fitting's kind is on a fitting; read `family.rs` 28 to 160: there the category is the family's |
| `check_connectors` | ported | `domain` | `crates/core/src/family.rs:445` | only a pipe connector carries a domain |
| `TOLERANCE_FT` | ported | `TOLERANCE` | `crates/core/src/network.rs:23` |  |
| `MAX_TOLERANCE_FT` | ported | `MAX_TOLERANCE` | `crates/core/src/network.rs:25` |  |
| `MAX_POINTS` | ported | `MAX_POINTS` | `crates/core/src/network.rs:27` |  |
| `Joint` | ported | `Joint` | `crates/core/src/network.rs:38` |  |
| `element` | ported | `element` | `crates/core/src/network.rs:48` |  |
| `Node` | ported | `Node` | `crates/core/src/network.rs:57` |  |
| `Edge` | ported | `Edge` | `crates/core/src/network.rs:64` |  |
| `Group` | ported | `Group` | `crates/core/src/network.rs:74` |  |
| `Warning` | ported | `Warning` | `crates/core/src/network.rs:82` |  |
| `Network` | ported | `Network` | `crates/core/src/network.rs:91` |  |
| `network` | ported | `network` | `crates/core/src/network.rs:163` |  |
| `area` | ported | `area` | `crates/core/src/geom.rs:98` | and `Region::area`, `geom.rs:540` |
| `perimeter` | ported | `perimeter` | `crates/core/src/geom.rs:119` | and `Region::perimeter`, `geom.rs:543` |
| `contains` | ported | `contains` | `crates/core/src/geom.rs:124` |  |
| `faces` | ported | `faces` | `crates/core/src/geom.rs:448` |  |
| `Region` | ported | `Region` | `crates/core/src/geom.rs:534` |  |
| `region_in` | ported | `region_in` | `crates/core/src/geom.rs:556` |  |
| `region_holds` | ported | `region_holds` | `crates/core/src/model.rs:316` |  |
| `RoomArea` | ported | `RoomArea` | `crates/core/src/model.rs:2687` |  |
| `Unbound` | ported | `Unbound` | `crates/core/src/model.rs:303` |  |
| `Plan` | ported | `Plan` | `crates/core/src/rooms.rs:18` |  |
| `of_walls` | ported | `plan` | `crates/core/src/rooms.rs:50` |  |
| `bound` | ported | `bound` | `crates/core/src/rooms.rs:109` |  |
| `Geom` | ported | `Geom` | `crates/core/src/runs.rs:81` |  |
| `of` | ported | `of` | `crates/core/src/runs.rs:92` |  |
| `length` | ported | `length` | `crates/core/src/runs.rs:107` |  |
| `plan_length` | ported | `plan_length` | `crates/core/src/runs.rs:112` |  |
| `vertical` | ported | `vertical` | `crates/core/src/runs.rs:117` |  |
| `half` | ported | `half` | `crates/core/src/runs.rs:122` |  |
| `z_span` | ported | `z_span` | `crates/core/src/runs.rs:148` |  |
| `depth` | ported | `depth` | `crates/core/src/walls.rs:153` |  |
| `wall_faces` | ported | `faces` | `crates/core/src/walls.rs:191` |  |
| `centre` | ported | `centre` | `crates/core/src/walls.rs:198` |  |
| `PlanWall` | ours |  |  | a wall as a row of a list, in place of BIMCraft's model and an id; read `walls.rs` 1 to 420: its functions take `&Model` and an `Id` |
| `stacked` | ported | `stacked` | `crates/core/src/walls.rs:204` |  |
| `cap` | ported | `cap` | `crates/core/src/walls.rs:298` |  |
| `centre_line` | ported | `centre_line` | `crates/core/src/walls.rs:348` |  |
| `outline` | ported | `outline` | `crates/core/src/walls.rs:375` |  |
