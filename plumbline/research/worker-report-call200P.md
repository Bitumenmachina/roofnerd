# W-200P report (contract 200P; part 1 = call 200 points 1 to 3, part 2 = an item's size as typed)

Working copy: SCRATCH/wtB at 39fbe9c, nothing committed. Notes written as I went.

## Before any code

### 1. BIMCraft searched (all of /home/user/ipedrax/bimcraft)
`grep -rlwi "ports\?\|connectors\?\|RelConnects[A-Za-z]*\|RelNests\|IfcDistributionPort" crates apps --include=*.rs` gives 36 files
(ui-egui, update, assistant, rvt, core, apps). The ones that matter, read:
- **Reading**: `crates/core/src/import.rs:52,63` lists `IFCDISTRIBUTIONPORT` among what never comes in as a thing ("ports"); the
  only occurrence of the word in the tree: `grep -rn "IFCDISTRIBUTIONPORT\|IFCRELNESTS\|IFCRELCONNECTS\|DistributionPort\|RelNests" crates apps`
  finds that one line. **No port and no relation between ports is read anywhere.**
- **Writing**: `crates/core/src/ifc.rs:1587-1615` writes a placed device's electrical connector as a property set
  (`Pset_ElectricalDeviceCommon`, `BIMCraft_Connector`, line 88) and says in a comment "a pipe's waits for IFC's ports": ports are
  not written. `family.rs` holds connectors on families (BIMCraft's own, `family.rs:132`, 698), `network.rs` and `water.rs` work
  networks from them (already ported as `building/network.rs`).
- `crates/rvt/src/pipes.rs` reads a Revit pipe's two connector origins as its line (not an IFC port).
Nothing to port as code. Recorded in `PORTMAP.md` (outside the building model, call 200 bullet).

### 2. IFC4: what I could read
buildingSMART's own pages (standards.buildingsmart.org, IFC4 ADD2 TC1 `ifcdistributionport.htm`, `ifcrelconnectsports.htm`) **could not be
reached**: the proxy answered 403 Forbidden to both. Source named instead: **IfcOpenShell 0.9.0.post1's own schema** (run:
`ifcopenshell.schema_by_name("IFC4")` and `("IFC2X3")`: attributes, enumerations, inverse attributes), its bundled copy of the IFC4 ADD2 TC1
entity texts (`ifcopenshell/util/schema/ifc4_entities.json`, which carries buildingSMART's own descriptions and spec urls) and its
bundled where-rules (`ifcopenshell/express/rules/IFC4.py`). Nothing from memory.
- `IfcDistributionPort` (supertype IfcPort < IfcProduct) attributes in order: GlobalId, OwnerHistory, Name, Description, ObjectType,
  ObjectPlacement, Representation, **FlowDirection** (IfcFlowDirectionEnum: SOURCE, SINK, SOURCEANDSINK, NOTDEFINED), **PredefinedType**
  (IfcDistributionPortTypeEnum: CABLE, CABLECARRIER, DUCT, PIPE, USERDEFINED, NOTDEFINED), **SystemType** (IfcDistributionSystemEnum, 43
  literals, among them AIRCONDITIONING, VENTILATION, EXHAUST, DOMESTICCOLDWATER, DOMESTICHOTWATER, SEWAGE, DRAINAGE, STORMWATER, GAS, VENT,
  REFRIGERATION, ELECTRICAL, LIGHTING, USERDEFINED, NOTDEFINED). The text: SystemType "if a system type is defined, the port may only be
  connected to other ports having the same system type"; FlowDirection "Sink (inlet), a Source (outlet) or both a SinkAndSource".
  IFC2X3's `IfcDistributionPort` has FlowDirection only (no PredefinedType, no SystemType).
- **How a port is put on an element**: IFC4's `IfcRelNests` (RelatingObject = the whole, RelatedObjects = the nested parts, "an order among
  the parts"); its text says it "can also be applied to ... elements having ports". `IfcRelConnectsPortToElement` (RelatingPort,
  RelatedElement: IfcDistributionElement) is in IFC4's schema too and is what IFC2X3 has (IfcOpenShell's IFC2X3 schema has both entities;
  `IfcDistributionElement.HasPorts` is the inverse in IFC4). Written: IfcRelNests (IFC4). Read: both.
- **How two ports are tied**: `IfcRelConnectsPorts` (RelatingPort, RelatedPort, optional RealizingElement); where-rule NoSelfReference
  (the two ports differ); `IfcPort.ConnectedTo` and `ConnectedFrom` are each `[0:1]`, so **one port is tied once as the relating and once
  as the related port, at most**. IfcRelNests has NoSelfReference too.
Consequence for the design: where three or more ends meet at one place, ties are pairs; the model joins the rest by being at the same
place (a node), so only a connection on a thing needs a tie to be read.

### 3. What the model holds, and what `takeoff` counts as an open end
- On a thing set down (fixture, equipment) and on a fitting: `Connectors`, each `Connector {name, kind (power/pipe/duct/conduit), domain,
  flow, at (in the thing's own axes: along, left, up from its base), size}`. On a run's end: nothing; a run is `Segment` (line, two heights).
  Between two runs: nothing: two ends within 0.01 ft are one node of `building::network` (and an end on another run's length is a node too).
- **Rule** (`building.rs` ~3577 and `network.rs`): a node is open where one end or one connector meets nothing else; a run end is open when
  it is alone at its node; a thing's connector is open when no run end is at it. An open end is said on the system of its run; a connector
  on the systems that carry what it says (a pipe connector with a domain) else the thing's system.
- First job (`sixteen`, `takeoff`; the figures the demonstration is held to):
  **supply air 4, return air 0, refrigerant 0, cold water 1, hot water 0, sanitary 1, vent 1, power 1, lighting 0.**
- Two different counts exist and differ for sanitary and vent: `takeoff` (what a person reads) gives 1 and 1; the waiting test's own
  count (`network(Some(system))`, the network the system's runs are in) gives **sanitary 2, vent 2**, because the sanitary and the vent
  run are one network and each is asked for the whole network's open ends (the same one end of each, counted in both). **`takeoff`'s is right
  and agrees with the lead's STATUS figure (sanitary 1)**; the waiting test is held to `takeoff`'s count. Second job today (the same
  counts by `network`): supply air 13, return air 2, refrigerant 1, cold water 5, hot water 4, lighting 1, power 1, sanitary 5, vent 5.

### 4. A fixture with two connections and the two runs that reach it, as the built program writes it (the house, `export ifc`)
The water heater (equipment) with its two connections, nested; each tied to the end of the run that reaches it:
```
#2262=IFCDISTRIBUTIONFLOWELEMENT('0NKTeGTgW8BNSp4Fvw1LwW',$,'water heater',$,'equipment',#2250,#2261,$);
#5027=IFCDISTRIBUTIONPORT('1gPlorL7VGmY_ZZ4Uw9mIB',$,'Cold water',$,$,#5026,$,.SINK.,.PIPE.,.DOMESTICCOLDWATER.);
#5033=IFCDISTRIBUTIONPORT('1gPlsrL7VQbI_ZYqUw9pyy',$,'Hot water',$,$,#5032,$,.SOURCE.,.PIPE.,.DOMESTICHOTWATER.);
#5034=IFCRELNESTS('2wXdNVneIT7xoN3YnsIU7A',$,$,$,#2262,(#5027,#5033));
   (#5026 = IFCLOCALPLACEMENT(#2250, axes at (-0.3, 0., 5.)) and #5032 at (0.3, 0., 5.): in the heater's own placement, feet)
#2636=IFCPIPESEGMENT('1c6can_e1_VtYI_99FAO8g',$,'cold to the water heater, piece 3 of 3',$,$,#2624,#2635,$,.RIGIDSEGMENT.);
#5349=IFCDISTRIBUTIONPORT('2ZQIQpaukX8pC_6XX9RTwi',$,'second end',$,$,#5348,$,.SOURCEANDSINK.,.PIPE.,.DOMESTICCOLDWATER.);
#5350=IFCRELNESTS('2fTnL5zKgt2CbxUTHLh$yk',$,$,$,#2636,(#5343,#5349));
#6619=IFCRELCONNECTSPORTS('2CxxGNc5SUv5mZc4D$BKfi',$,$,$,#5027,#5349,$);
#2704=IFCPIPESEGMENT('1c7N8n_hxidNXqWv9809x4',$,'hot main to the baths, piece 1 of 4',$,$,#2692,#2703,$,.RIGIDSEGMENT.);
#5395=IFCDISTRIBUTIONPORT('338pOsT47WKSs1Nn6mirFL',$,'first end',$,$,#5394,$,.SOURCEANDSINK.,.PIPE.,.DOMESTICHOTWATER.);
#5402=IFCRELNESTS('2m$aUzCLN7Qd7gIxNv$Aox',$,$,$,#2704,(#5395,#5401));
#6620=IFCRELCONNECTSPORTS('34H6QhgUXeyKPCn7sADcwh',$,$,$,#5033,#5395,$);
```
(IfcOpenShell: no fault against the schema and its rules; and an outside check in `demos/ifc-whole.sh` that each tie's two ports sit
at one place: 147 ties, 332 ports.) A run's end port points out of the run (the first end's z is -z of the run), a thing's up.

## What is built, part 1 (written as I went)
- **Written** (`crates/ifc/src/write.rs`): a connection point (IfcDistributionPort: name, FlowDirection SOURCE/SINK/SOURCEANDSINK, PredefinedType
  DUCT/PIPE/CABLECARRIER/CABLE, SystemType from what the connection carries or the system it is on) for each connection of a fixture, a
  piece of equipment and a fitting, placed in the thing's own placement at the connection's place; two for each run (first end, second
  end; the first points out of the run); each thing's in one IfcRelNests, in order; ties (IfcRelConnectsPorts) from the model's own
  network: at each place a connection and a run's end meet, the two are tied; two ends alone, the two are tied; a third end there is
  joined by being there. A run with no size: its placement and an "Axis" line (Curve3D polyline), no body. Made-counts say "connection
  points" and "connection point ties" (no software word on a screen). House: 332 connection points, 147 ties; IfcOpenShell no fault.
- **Read** (`crates/ifc/src/lib.rs`, `crates/world/src/editing/join.rs`): new change `connect_port` (opened from `connect`: it is told
  which end, or a place where there is no run; Assumed, sourced to the file's connection point and the run it is tied to). Each
  connection point of a fixture or piece of equipment comes back as its connection at its place (46 in the house, in order); where tied
  to the end of a run brought in and that end is within 0.05 ft of the point, the run is joined to it (46 of 46); a tie to an end that is
  not at the point joins nothing and is said ("tied in the file to the end of a run that is not at it, and not joined"). A file with
  NO connection points reads as it always did (an end inside a box is joined, today's note). Either way, run ends within a foot of a
  fixture or piece of equipment that read open are said with how far ("1 run end stands within a foot ... 0.50 ft from Water closet").
  Unsized runs: read from the Axis line, Segment type = Unknown, counted ("3 runs have no size: each is drawn as a line alone ...").
  Fitting words: the file's sort against what `fitting` makes; a difference names place and both words.
- Files touched in `crates/world/src`: `editing/join.rs` (`connect_port`, `PortSaid`) and `editing.rs` (the re-export; part 2's `item_size`). No
  figure's meaning changes; `connect` is untouched.

## Red runs (part 1, `cargo test -p plumbline-ifc --test ports`, before any change to the reader or writer)
`2 passed; 9 failed` (r3 and r5, which hold what the reader did, passed): w1 `ports: left: 0 right: 336`; p2 (the waiting test, held to
takeoff's count) `the second job: {"Domestic Cold Water": 5, "Domestic Hot Water": 4, "Lighting": 1, "Power": 1, "Refrigerant": 1,
"Return Air": 2, "Sanitary": 4, "Supply Air": 13, "Vent": 1}`; f1 `left: 0 right: 2`; r1 `left: 46 right: 22` (its first assertion
was moved); r2/r4 notes missing; w3 the three pipes missing from the file. Greens afterward: 11 of 11 (10 after p2 moved to whole.rs).
Part 2 red: compile errors, `crates/world/tests/item_size.rs`: "unresolved imports `plumbline_world::editing::ItemSizeDone`", "cannot
find function `item_size` in module `ed`", "cannot find value `ITEM_SIZE_TYPED`" (SCRATCH/w200/red-item-size.txt). The byte-for-byte test
was proved red by a mutation (the followers' status fix switched off): `the_one_change_makes_the_same_saved_job_as_the_windows_steps ... FAILED`.

## What is built, part 2 (an item's size as typed)
- `crates/world/src/editing.rs`: `item_size(job, item, what, typed, who) -> Result<ItemSizeDone, String>`, `ITEM_SIZE_TYPED` (the words, one copy),
  `HowSized {Changed, Checked}`, `ItemSizeDone {item, what, how, old, new, followed, name_carries_old, also}` with `said()`: the words both
  say. One change, all or none (`all_or_nothing`). Differs: `measure` with ITEM_SIZE_TYPED, the item's value set back to Assumed, each
  follower put right (status and source) as the window did, `derive`. Same (or "same"): `measure` Same, "checked on site" (": who"), Known. A
  size that is not a length, an unknown name or a dimension that is not one of the three: Err, nothing changed.
- The command line: `plumbline set <file> <item> width|depth|height "<size>" [--source "<who checked it>"]` with no `--status`
  (the verb that sets an item's value today; today it refuses without --status) says `done.said()`; with `--status` it is as it was.
  `measure` is unchanged (a measurement is Known). Test: `crates/cli/tests/cli.rs` `an_items_size_typed_is_said_as_the_window_says_it`.
- Tests (`crates/world/tests/item_size.rs`, 6): differs; the same again (and "same" with a name); nothing set down; two set down; not a
  length (and a dimension that is not one, and a name that is not an item: the job saved the same); a name that carries the old size and
  one that does not; and **the one change and the window's own three steps (copied from `tools.rs`) make the saved job byte for byte the
  same**, for width, depth and height, and after all three were checked on site and one typed over.
- **What the window's function becomes** (not edited; `crates/app/src/tools.rs` `item_size`, 1674-1746): keep its guard (is it an item, is
  `what` one of three, "same" or a length, else `return false`), then
  ```rust
  match ed::item_size(&mut doc.job, path, what, reading, who) {
      Ok(done) => { doc.edit(if done.how == ed::HowSized::Checked { "checked an item's size" } else { "typed an item's size" });
                    doc.say(if done.name_carries_old { Tone::Warned } else { Tone::Done }, done.said()); }
      Err(why) => cant(doc, "use that size", why),
  }
  true
  ```
  and `ITEM_TYPED` (tools.rs:1666) and `TYPED` (tools.rs:503, New item) become `ed::ITEM_SIZE_TYPED`. Its `said()` is word for word what the
  window says today (the window's test strings "2 things set down from it follow", "type 48 in again", "still reads 36 in", "checked on site").
  One difference for the window to know: its `Tone` for a changed size whose name carries the old size is `Warned` today; `done.name_carries_old`
  says so. `crates/app/src/bench.rs` tests need no change.

## Run
- `cargo test -p plumbline-ifc`: 43 passed (unit 4+4, call180 4... ifc 6, ports 10, whole 13, write 6), 0 failed, 0 ignored (the waiting test no longer ignored).
- `cargo test -p plumbline` (cli): 22 passed. `cargo test -p plumbline-world`: 542 passed (item_size 6 of them), 0 failed. (Not run: plumbline-app, plumbline-sheets.)
- `demos/ifc-whole.sh`, `export.sh`, `model.sh`: pass. There is no `demos/ifc.sh` (the folder `demos/ifc/` holds the made-up model `model.sh` and `bringw.sh` use).
- **Every command-line demonstration: 51 run (every demos/*.sh without plumbline-app, open_window, xvfb or xdotool, less window.sh and field-job.sh), 51 ended exit 0; the words FAILED and FAIL are in none of the 51 logs** (SCRATCH/w200/demos/*.log, summary.txt).
- Gates: `tools/gate-language.sh`, `gate-egress.sh`, `gate-clientdata.sh --all` PASS; `tools/licenses.sh` exit 0; `BIMCRAFT=... tools/portmap.sh` PASS (336 items; 194 ours).

## Expectations that moved (each listed before changed)
| where | wanted | gets | why |
|---|---|---|---|
| crates/ifc/tests/whole.rs `p2_each_system_reads_as_closed_as_it_was` | `#[ignore]`d; `network(Some(system))` figures with Supply Air 4, Cold 1, Power 1, Refrigerant 0 | runs; held to `takeoff`'s count, all nine systems: supply 4, return 0, refrigerant 0, cold 1, hot 0, sanitary 1, vent 1, power 1, lighting 0 | the figures of answer 3; the old count asked each system for its whole network (sanitary 2, vent 2) |
| crates/ifc/tests/whole.rs `p2_fittings_come_in_on_their_runs` | 37 fittings; Bends = second job's + 6; 2 Tees | 37 fittings (unchanged); Bends = + 7; **3 Tees** | the three hot-water runs with no size are now in the file, so the hot main's elbow at the water heater, (16.3, 26.5, 6), has three ends and is a tee: a third "bend that is a tee" |
| crates/ifc/tests/write.rs `the_house_is_written_whole...` | 5 items "size is not known" (3 pipes, 2 elbows) | 2 (the elbows) and the 3 pipes are in the file | call 200 point 3 |
| demos/export.sh:160 | says "its size is not known, so it has no shape to write" | says "it has no shape: its size is not known" (the elbows) and not the old words | same |
| demos/ifc-whole.sh | "22 runs ending at a fixture ... were joined"; run figures with "size unknown" lines left out of the comparison; held-for-a-ruling header | "46 connections on fixtures and equipment were brought in from the file's connection points...; 46 ... tied"; the 3 unsized runs compared too; open ends by system; three fitting notes; the near-fixture run | the figures of the contract |
| counts of what a written file holds (`export ifc` print) | no ports | + "332 connection points, 147 connection point ties" | ports are new things in it |
| the first import's notes | "22 runs ending at a fixture ... were joined to it, where it stood" | the "connection points" note above, for a file with them; the old note, for a file with none | the file says what is connected |
