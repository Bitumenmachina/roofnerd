# Port map: the building model against BIMCraft

Every public item of the building model (`crates/world/src/building*.rs` and anything
under `crates/world/src/building/`) and of what draws and edits it
(`crates/world/src/editing*.rs` and anything under `crates/world/src/editing/`) is listed
here once, as **ported** from BIMCraft or as
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
| `run_goes_on` | ours |  |  | which classifications of system a kind of run goes on; `grep -rn 'fn run_goes_on' crates/core/src`: nothing (BIMCraft's system is a name on a run, read `model.rs` 1588 to 1612) |
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
| `system_classification` | ours |  |  | which of this program's system classifications carries a pipe connector's domain; `grep -rn system_classification crates/core/src`: nothing (BIMCraft's systems are names on runs) |
| `of_system` | ours |  |  | the reverse of `system_classification`; `grep -rn 'fn of_system' crates/core/src`: nothing |
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
| `UNDO_LIMIT` | ported | `UNDO_LIMIT` | `crates/core/src/engine.rs:103` | the last 200 steps are kept |
| `History` | ported | `undo` | `crates/core/src/engine.rs:160` | an undo stack and a redo stack, each step with the model before it; here the model kept is the saved job's text (header) |
| `before` | ported | `before` | `crates/core/src/engine.rs:386` | the model as it stands before a command runs |
| `record` | ported | `undo` | `crates/core/src/engine.rs:499` | a step is recorded only when the model changed; the oldest goes past the limit; the redo stack is emptied (`engine.rs:497` to `503`) |
| `undo` | ported | `step_back` | `crates/core/src/engine.rs:728` | pop one stack, push what stands now onto the other, name the step |
| `redo` | ported | `step_back` | `crates/core/src/engine.rs:728` | as `undo`, the other way |
| `labels` | ported | `history` | `crates/core/src/engine.rs:272` | the steps' labels, oldest first |
| `Entered` | ours |  |  | how sure a thing being drawn is and who says so; BIMCraft's commands carry no status: `grep -c Assumed` in each of commands.rs engine.rs: 0 |
| `Top` | ours |  |  | a wall's top as a height or a storey, said every time; BIMCraft's `wall` takes `height` or `top_level` and gives a default when neither is said (`commands.rs:7355` to `7362`); `grep -rn 'enum Top' crates/core/src`: nothing |
| `OnWall` | ours |  |  | an opening's place as an offset or a point; BIMCraft's `opening_of` takes `at` or `offset` as JSON fields (`commands.rs:7502`); `grep -rn 'enum OnWall' crates/core/src`: nothing |
| `all_or_nothing` | ported | `transaction` | `crates/core/src/engine.rs:648` | all of a change or none of it; here the job is put back from its saved text |
| `storey` | ported | `level` | `crates/core/src/commands.rs:6422` | a level by name and elevation; a storey height and the `Val` rules are ours |
| `wall_type` | ported | `WallType` | `crates/core/src/commands.rs:11188` | a wall type by its thickness; layers are not taken by this verb |
| `opening_type` | ported | `OpeningType` | `crates/core/src/commands.rs:11206` | a door or window type by width and height; its sill and operation are not taken |
| `segment_type` | ported | `RunType` | `crates/core/src/commands.rs:11243` | a run type round, or wide by deep; its material is not taken |
| `system` | ours |  |  | a system as an element is this program's (HANDOFF_REBASE 7); BIMCraft's is a name on a run (`commands.rs:8799`); `grep -rn 'fn system(' crates/core/src`: nothing |
| `walls` | ported | `wall` | `crates/core/src/commands.rs:7341` | a wall for each pair of points that follow one another, `closed` back to the first. Not taken: a default height or top level |
| `opening` | ported | `opening_of` | `crates/core/src/commands.rs:7473` | a door or window on a wall, at a point taken along it (`along`, `commands.rs:2426`). Not taken: the middle of the wall when no place is said; moving a point past an end to the end |
| `runs` | ported | `runs_of` | `crates/core/src/commands.rs:8776` | a run for each pair of points that follow one another, at a height, on a system. Not taken: a default height of 0 |
| `place` | ported | `stood_of` | `crates/core/src/commands.rs:8662` | a thing stood at a point on a level, so far above it; what it is as text is ours |
| `move_by` | ported | `move_elements` | `crates/core/src/commands.rs:12419` | move by a distance east and north; a wall's openings go with it. Not taken: moving one end; several at once |
| `remove` | ported | `delete` | `crates/core/src/commands.rs:13011` | take an element out, and a wall's doors and windows with it (`commands.rs:13040`) |
| `room` | ported | `room` | `crates/core/src/commands.rs:7909` | a room placed by a point on a level, with a name. Not taken: a number given when none is said |
| `STEP_FT` | ported | `STEP` | `crates/core/src/snap.rs:19` | what a point with nothing to snap to rounds to; an inch here, 10 mm there (header) |
| `SnapKind` | ported | `Kind` | `crates/core/src/snap.rs:34` | what a point snapped to; curves', scans' and annotation's kinds are not taken |
| `name` | ported | `name` | `crates/core/src/snap.rs:59` | what a snap is called |
| `Snaps` | ported | `Snaps` | `crates/core/src/snap.rs:79` | the snaps in force |
| `none` | ported | `none` | `crates/core/src/snap.rs:104` | no snap at all |
| `Reach` | ported | `Reach` | `crates/core/src/snap.rs:164` | how far a snap reaches, to a point and to a line |
| `Guide` | ported | `Guide` | `crates/core/src/snap.rs:171` | an angle guide through the last point |
| `Found` | ported | `Found` | `crates/core/src/snap.rs:180` | where a pick landed, what it snapped to and which element it came from |
| `Scene` | ported | `Scene` | `crates/core/src/snap.rs:211` | what the plan has to snap to; here it can be written out by hand (header) |
| `segment` | ported | `segment` | `crates/core/src/snap.rs:226` | a straight piece gives its ends, its middle and its line |
| `scene` | ported | `of` | `crates/core/src/snap.rs:267` | the scene of one level's plan; here the walls and runs entered on the storey (header) |
| `corner` | ported | `corner` | `crates/core/src/snap.rs:633` | the angle a new line makes with the one before |
| `snap` | ported | `solve` | `crates/core/src/snap.rs:736` | the solver: a point of the plan, the foot of the perpendicular, an angle guide, a wall's line, else the round step |
| `wall_type_for` | ported | `type_for_size` | `crates/core/src/text.rs:282` | BIMCraft's rule for a text type: the one of that size and nothing else, or a new one named for it. Here the same rule for wall type of this thickness, or a new one named for it |
| `opening_type_for` | ported | `type_for_size` | `crates/core/src/text.rs:282` | BIMCraft's rule for a text type: the one of that size and nothing else, or a new one named for it. Here the same rule for door or window type of this width and height, or a new one named for them |
| `segment_type_for` | ported | `type_for_size` | `crates/core/src/text.rs:282` | BIMCraft's rule for a text type: the one of that size and nothing else, or a new one named for it. Here the same rule for round run type of this size, or a new one named for it |
| `system_of_class` | ported | `type_for_size` | `crates/core/src/text.rs:282` | BIMCraft's rule for a text type: the one of that size and nothing else, or a new one named for it. Here the same rule for job's system of this classification, or a new one named for it |
| `toward` | ported | `toward` | `crates/core/src/draw.rs:304` | a typed length goes from the last point toward the pointer, east when the pointer is on the point |
| `split` | ported | `split` | `crates/core/src/commands.rs:12534` | BIMCraft's is of a wall: cut between the ends, the first piece keeps its identity, the second is new, what hangs on it goes with its piece. Here the same rule for a run; a run with a typed-quantity cost line is refused |
| `Fitted` | ours |  |  | what `fitting` did; `grep -rn 'struct Fitted' crates/core/src`: nothing |
| `fitting` | ours |  |  | a fitting worked out from the runs that meet; `grep -rn 'fn fitting' crates/core/src`: nothing. BIMCraft places a pipe fitting by hand as a component of a family (read `fixtures.rs` 203 to 250 and `family.rs` 132) |
| `PassedOver` | ours |  |  | a joint `fit_all` left; `grep -rn PassedOver crates/core/src`: nothing |
| `fit_all` | ours |  |  | as `fitting`, at every joint; `grep -rn 'fn fit_all' crates/core/src`: nothing |
| `connect` | ours |  |  | a connector made where a run ends at a thing; `grep -rn 'fn connect(' crates/core/src`: nothing. BIMCraft's connectors are drawn in the family (`family.rs`, `Connector`) |
| `connect_at` | ours |  |  | `connect`, for a point picked on the plan; `grep -rn 'fn connect_at' crates/core/src`: nothing |
| `MAX_CLEARANCE_FT` | ours |  |  | clash detection is this program's; `grep -rni clash crates/core/src`: six lines, two comments (`inquiry.rs:3`, `link.rs:72`) and a variable in `import.rs` (3273 to 3364) for parameter names that collide on import; nothing tests one solid against another. |
| `NEVER_TESTED` | ours |  |  | clash detection is this program's; `grep -rni clash crates/core/src`: six lines, two comments (`inquiry.rs:3`, `link.rs:72`) and a variable in `import.rs` (3273 to 3364) for parameter names that collide on import; nothing tests one solid against another. |
| `Reading` | ours |  |  | a measurement taken on site put on a value is this program's: BIMCraft keeps no certainty on a value. `grep -rniwE 'measured|verified|assumed|certain' crates/core/src/{commands,model,engine,constraints}.rs`: every hit is a dimension line, a room's area or a laser scan's point cloud (`commands.rs:177`, `11550`), none a value marked as measured. What a run's heights are held to is its `justify` (`commands.rs:8794`), which the model here already carries |
| `Measured` | ours |  |  | a measurement taken on site put on a value is this program's: BIMCraft keeps no certainty on a value. `grep -rniwE 'measured|verified|assumed|certain' crates/core/src/{commands,model,engine,constraints}.rs`: every hit is a dimension line, a room's area or a laser scan's point cloud (`commands.rs:177`, `11550`), none a value marked as measured. What a run's heights are held to is its `justify` (`commands.rs:8794`), which the model here already carries |
| `measure` | ours |  |  | a measurement taken on site put on a value is this program's: BIMCraft keeps no certainty on a value. `grep -rniwE 'measured|verified|assumed|certain' crates/core/src/{commands,model,engine,constraints}.rs`: every hit is a dimension line, a room's area or a laser scan's point cloud (`commands.rs:177`, `11550`), none a value marked as measured. What a run's heights are held to is its `justify` (`commands.rs:8794`), which the model here already carries |
| `takes` | ours |  |  | a measurement taken on site put on a value is this program's: BIMCraft keeps no certainty on a value. `grep -rniwE 'measured|verified|assumed|certain' crates/core/src/{commands,model,engine,constraints}.rs`: every hit is a dimension line, a room's area or a laser scan's point cloud (`commands.rs:177`, `11550`), none a value marked as measured. What a run's heights are held to is its `justify` (`commands.rs:8794`), which the model here already carries |
| `AGREE_FT` | ours |  |  | how nearly two Known values that say one thing must agree (an eighth of an inch): BIMCraft's level has an elevation and no height of its own (`crates/core/src/model.rs:131`, `pub struct Level`) and keeps no certainty on a value, so it never has two values to reconcile. `grep -rniE 'floor.to.floor|storey_height|level_height' crates/core/src/{commands,model}.rs`: nothing |
| `ClashKind` | ours |  |  | clash detection is this program's; `grep -rni clash crates/core/src`: six lines, two comments (`inquiry.rs:3`, `link.rs:72`) and a variable in `import.rs` (3273 to 3364) for parameter names that collide on import; nothing tests one solid against another. Hard and clearance are the trade's words (DECISIONS.md call 40). |
| `Clash` | ours |  |  | clash detection is this program's; `grep -rni clash crates/core/src`: six lines, two comments (`inquiry.rs:3`, `link.rs:72`) and a variable in `import.rs` (3273 to 3364) for parameter names that collide on import; nothing tests one solid against another. |
| `Clashes` | ours |  |  | clash detection is this program's; `grep -rni clash crates/core/src`: six lines, two comments (`inquiry.rs:3`, `link.rs:72`) and a variable in `import.rs` (3273 to 3364) for parameter names that collide on import; nothing tests one solid against another. |
| `clashes` | ours |  |  | clash detection is this program's; `grep -rni clash crates/core/src`: six lines, two comments (`inquiry.rs:3`, `link.rs:72`) and a variable in `import.rs` (3273 to 3364) for parameter names that collide on import; nothing tests one solid against another. The solid geometry is `parry3d-f64`'s, not written here. |

## Ported outside the building model

`tools/portmap.sh` covers the building model and its editing only. What else is ported
from BIMCraft at the same commit is listed here by hand, and is checked by its own tests.

- `crates/ifc/src/step.rs`, all of it, from `crates/core/src/step.rs`: the values `V`
  (line 9), `Entity` (24), `File` (30), `parse` (39), the `Parser` (107) and `decode`
  (314), with its three tests (354). Changed: two `if let ... && let ...` chains in
  `decode` are written with `and_then`, because this workspace is on the 2021 edition.
- `crates/ifc/src/lib.rs`, the importer over it, is ours (DECISIONS.md call 47):
  BIMCraft's `import.rs` builds BIMCraft's model and was read, not ported.
| `floor_type` | ported | `FloorType` | `crates/core/src/commands.rs:11195` | a floor type by its thickness; layers are not taken by this verb |
| `floor_type_for` | ported | `type_for_size` | `crates/core/src/text.rs:282` | the same rule as `wall_type_for`: the floor type of this thickness and nothing else, or a new one named for it |
| `floor` | ported | `floor` | `crates/core/src/commands.rs:7556` | a floor of a type on a level, by a boundary that is given or, with none, by the outline its walls enclose. Not taken: slope, the span arrow, "structural", the sketch; and a courtyard is refused, not cut out (DECISIONS.md call 53) |
| `FloorBy` | ours |  |  | how `floor` is told its boundary: by the walls or by points; BIMCraft reads an optional `points` out of its command's JSON (`commands.rs` 7556 to 7600); `grep -rn 'enum FloorBy' crates/core/src`: nothing |
| `Floored` | ours |  |  | what `floor` did, with what has to be said about it; `grep -rn 'struct Floored' crates/core/src`: nothing |
| `floor_outline` | ported | `floor_outline` | `crates/core/src/commands.rs:2467` | along the centre lines of the walls, the outer outline of the largest piece they make, and the loops standing free inside it. Added: the other pieces the walls make, and how sure the outline is |
| `FloorOutline` | ours |  |  | what `floor_outline` found, with its status; BIMCraft returns a pair (`commands.rs:2467`); `grep -rn 'struct FloorOutline' crates/core/src`: nothing |
| `corners` | ported | `corners` | `crates/core/src/geom.rs:136` | a ring's corners: its points less those on a straight run between their neighbours |
| `room_outlines` | ours |  |  | each room's outline at the faces of its walls, for writing a model out; BIMCraft keeps a room's boundary on the room (`rooms.rs`, read 40 to 120) and this program works it out with the room's figures; `grep -rn 'fn room_outlines' crates/core/src`: nothing |
