# W-214M report (contract 214M) -- notes as I go

## Before any code

### 1. BIMCraft: how a product's body is read (import.rs 862-1050, 2603-2780; import/shapes.rs), kind of body by kind
BIMCraft's `Reader::faces` turns each representation item into polygons; `chosen` picks the items (representation identified
"Body"/"Facetation", else any but axis/footprint/annotation/plan/profile/reference/surface); `looked` handles mapped items and
colour maps; a product is then a family+type made of the triangles of those faces in its own axes (`component`, 2623).
Per kind of body (what I port = follow the structure and rule, written fresh over our own `step::V`; no code copied, so NOTICE unchanged):
- IFCEXTRUDEDAREASOLID (rect, circle, arbitrary closed polyline profile; also holed profiles): PORTED as the prism's corner points, pushed any way (BIMCraft's `profile` + `faces`). Left: tapered extrusion, rounded/hollow/I/L/U/C/T/Z profiles beyond their bounding rectangle (BIMCraft reads those as their bounding rectangle: ported for I/L/U/C/T/Z/rounded-rect/ellipse as the rectangle; indexed polycurve/arcs in arbitrary profiles left: needs curve sampling).
- IFCFACETEDBREP / FACETEDBREPWITH VOIDS / CSGSOLID / CLOSEDSHELL / OPENSHELL / CONNECTEDFACESET / SHELLBASEDSURFACEMODEL / FACEBASEDSURFACEMODEL / IFCFACE(+SURFACE) with poly loops: PORTED (points of the outer bound's loop; reversed bound sense is irrelevant for an extent).
- IFCTRIANGULATEDFACESET / POLYGONALFACESET (+ PnIndex): PORTED (coordinates the faces use).
- IFCMAPPEDITEM: PORTED, through the IfcRepresentationMap, the mapping origin and the transformation operator (3D and 3D non-uniform, scale), depth capped at 16 as BIMCraft does. This is "a shape shared by its type".
- IFCBOUNDINGBOX / IFCBLOCK: PORTED (corner + three sizes).
- IFCBOOLEANCLIPPINGRESULT / BOOLEANRESULT: PORTED as BIMCraft reads it (first operand; second too for UNION); a clipping does not change what the element is. Said so in the source.
- LEFT, with why (import/shapes.rs, 931 lines): revolved solids, swept solids along curves, swept disks, advanced B-reps (planes, cylinders, spheres, revolution, extrusion surfaces, B-splines). Each needs a curved-surface tessellator to find an exact extent; sampling would give a size nobody can vouch for, so the item comes in with its three sizes Unknown saying which kind it was. (File 4 of the demonstration is a revolved solid.)
- Also left: BIMCraft's materials/looks/colours per face, the 100000-triangle cap and families/types/mirror finding: this program draws an item as a box.
- Tessellated/brep extent is an extent of vertices; a mesh with a sunk point is still its extent.

### 2. Where the standard's names come from (IfcOpenShell 0.9.0.post1, run as an outside program; never linked)
Read by `python3 -I` with `ifcopenshell.ifcopenshell_wrapper.schema_by_name("IFC4"/"IFC2X3")`:
- Type of a product: `IfcRelDefinesByType` (RelatedObjects idx4, RelatingType idx5), both schemas. Occurrence attributes IFC4 (SanitaryTerminal, BuildingElementProxy, ElectricAppliance, Furniture): GlobalId, OwnerHistory, Name(2), Description, ObjectType(4), ObjectPlacement(5), Representation(6), Tag, PredefinedType(8). IFC2X3 has no IfcSanitaryTerminal/IfcElectricAppliance/IfcFurniture occurrence class (the occurrence is IfcFlowTerminal/IfcFurnishingElement; types exist); IFC2X3 proxy has CompositionType in place of PredefinedType.
- Shape shared by a type: `IfcElementType.RepresentationMaps` (idx6) -> IfcRepresentationMap (MappingOrigin, MappedRepresentation); the occurrence uses `IfcMappedItem(MappingSource, MappingTarget: IfcCartesianTransformationOperator3D(Axis1, Axis2, LocalOrigin, Scale, Axis3))`.
- Manufacturer's property set: IfcOpenShell's own copy of the standard's templates (util/schema/Pset_IFC4_ADD2.ifc): `Pset_ManufacturerTypeInformation` (applicable to IfcElement, type-driven): GlobalTradeItemNumber, ArticleNumber, ModelReference ("the model number or designator"), ModelLabel ("the descriptive model name"), Manufacturer, ProductionYear, AssemblyPlace; `Pset_ManufacturerOccurrence`: AcquisitionDate, BarCode, SerialNumber, BatchReference, AssemblyPlace (no maker/model). A type carries property sets in `IfcTypeObject.HasPropertySets` (idx5); a product through `IfcRelDefinesByProperties` (RelatingPropertyDefinition idx5). Properties are IfcPropertySingleValue(Name, Description, NominalValue, Unit). Rule I chose: maker = Manufacturer; model = ModelReference where there is one (the number), else ModelLabel; the source says which property.
- A connection point: `IfcDistributionPort` (IFC4: GlobalId, OwnerHistory, Name, Description, ObjectType, ObjectPlacement(5), Representation, FlowDirection(7), PredefinedType(8), SystemType(9); IFC2X3 has no PredefinedType/SystemType). FlowDirection enum: SOURCE, SINK, SOURCEANDSINK, NOTDEFINED. PredefinedType enum: CABLE, CABLECARRIER, DUCT, PIPE, USERDEFINED, NOTDEFINED. SystemType = IfcDistributionSystemEnum (DOMESTICCOLDWATER, DOMESTICHOTWATER, DRAINAGE, SEWAGE, WASTEWATER, VENT, ...). Put on the product by `IfcRelNests` (IFC4) or `IfcRelConnectsPortToElement` (RelatingPort, RelatedElement; both schemas) -- the same two as call 202 read.
- NOT readable from the machine: which way a port faces. The schema has no direction attribute; EXPRESS carries no prose. I take the z axis of the port's placement (the placement's Axis) as the way it faces, as the standard's placement of a port is generally used, and say so in each row's source ("the connection point's own axis"); it is only written where that axis lies along one of the six. FLAGGED in "what is not right".

### 3. The five files, by hand, small, made up
(see demos/maker-model/; all invented: maker "Northgate Fixtures", product "NL-2018"). Written with a small Python generator? NO: written as text by hand with the Write tool, each passed through tools/ifc-check.py. (filled in below)

### 4. What `import spec` returns (to be matched)
Line 1: `<path>: <name>, <a kind> (<why>), from <file>, page N`; then `  read off the sheet, each Assumed until it is checked against the thing:` and `    ~ <what> <value> "<line>" (note)`; then `  not on the sheet, and Unknown until entered:` and `    ? <what>`; then for missing sizes a line saying what no shape means. maker-model says the same, per item: what was made (path, name, kind and why), each value read with how sure and where from (`~`), what is Unknown (`?`), what was left out. Header line first says what the file is.

## Tests first, run red (stub `maker_model` returns Err("not built yet"))
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 cargo test -p plumbline-ifc --test call214m -j2`:
`test result: FAILED. 1 passed; 33 failed; 0 ignored` -- e.g. `called Result::unwrap() on an Err value: "not built yet"` at the_points_are_worked_in_the_things_own_axes_when_it_is_turned,
the_maker_and_model_on_the_type_on_the_product_and_nowhere, things_that_are_not_items_are_counted_and_left_out ... (the one that passed is "something that is no model is refused", which the stub trivially satisfies).
File: plumbline/crates/ifc/tests/call214m.rs (34 tests; builder of STEP text inside it; the five demo files are read from demos/maker-model/ and do not yet exist).

### 3 (filled). Five files + the outside variant written by hand (heredocs) in plumbline/demos/maker-model/:
nl-2018.ifc (file 1), nl-2018-proxy.ifc (2), nl-2018-shared.ifc (3: shape on the type's map, two of the type), np-1-turned.ifc (4: IFCREVOLVEDAREASOLID, chosen because BIMCraft's shapes.rs revolved solids need a curved tessellator), nl-2018-no-maker.ifc (5), nl-2018-outside.ifc (file 1 + a 4th point 100 mm behind the back).
All six: `tools/ifc-check.py` -> "IFC4: no fault against the schema and its rules" (np-1 first had 1 fault, a 3D point in a 2D placement; fixed). Checker prints "lengths in metre" for every file: its own quirk (reads the first unit), the file is MILLI metre.
Hand figures (steps 1,2): 508/457.2/203.2 mm = 20/18/8 in; cold: 355.6-254 = 101.6 mm = 4 in across, back 457.2-457.2 = 0, up 50.8 mm = 2 in; hot -4 in; waste 0/0/0 facing down. No difference from the lead's so far.
Resumed after a pause; maker.rs fully written (not yet compiled at this note); first compile running.

## Built; runs
- crates/ifc/src/maker.rs (new), 2 lines in lib.rs (`mod maker; pub use`), CLI verb `import maker-model <job> <file> [--as ...]` + usage line in crates/cli/src/main.rs. Nothing changed in crates/world or crates/app.
- call214m: first green run 29 of 34 (5 red: a stray `)` in my test text; the building sentence lacked "not brought in"; count and not-drawn notes absent; my expected 3.93701 in was wrong -- sizes are held to 1/10000 in so the place reads 3.937 in; the test was corrected, not the program); then `34 passed; 0 failed`.
- demos/maker-model.sh with PLUMBLINE_BIN: exit 0, "maker-model: all steps passed", FAIL searched: none (review/mm-run.txt). First run stopped at step 4 (script bug: it took the file name for the item path; fixed).
- README row and PORTMAP paragraph added (no code taken; NOTICE unchanged).

## Final
Tests (one crate at a time): world 667, cli 45, plans 172, sheets 6, ifc 107 (of which call214m 34), roofing 122: 0 failed. Demos: 56 command-line demonstrations rc=0 with no FAIL (maker-model among them); d2 and house need the window (plumbline-app, not built here) and field-job is a helper: not run as demonstrations. Gates (language, egress, clientdata), licenses.sh, BIMCRAFT portmap.sh: pass. Patch: review/call214M.patch.
Not right: see summary.

## Round 2
Test first: a_placement_with_no_axis_says_nothing_of_which_way_it_faces; red with the check off: `34 passed; 1 failed`. Fix: a point's facing is read only where its placement (IfcAxis2Placement3D) gives an Axis; otherwise it is not written (ItemConnection.faces is required in the saved format, which may not change, so "no facing" means no row) and the reply says "N connection points say nothing of which way they face". Green: ifc 108 passed, cli 45 passed, maker-model.sh rc 0, FAIL none. Our writer (write.rs `placed(.., [0,0,1], [1,0,0])`) always gives an explicit Axis (0,0,1): the "axis given" case, so a program-written file still reads "up" for every connection; unchanged by this fix.

## Round 3
Expectations moved (listed before changing; all mine or the two rulings'):
1. demos/connections.sh:160 held `says "3 connections: which way it faces is not written"`; now holds that the reply does NOT say it, and (after the read back) that the lavatory shows "faces back". Moved by ruling 2 (the writer now writes the axis from the facing).
2. crates/ifc/tests/call214m.rs: `.faces` comparisons now `Some(..)` (5 lines, the row's facing is optional); `the_way_it_faces_only_where_the_file_says_one_of_the_six` held "an axis along none of the six is not written, one row fewer": now written with no facing and the notes say so; round 2's `a_placement_with_no_axis...` held 1 row and a not-written line: now 4 rows, the three with no facing, a note "3 connection points say nothing of which way they face". Moved by ruling 1.
3. No other test in world, cli or ifc held a required facing or the "not written" line (grep of crates/*/tests and demos/*.sh); existing tests call `connection_row(.., Facing, ..)`, which is kept as it was (it wraps `Some`); `connection_row_facing` is added beside it.
Built: ItemConnection.faces is Option<Facing> (key absent where none), table Faces cell empty both ways, `connection add` may leave `--faces` out (or `none`), `connection set .. faces none` clears (and the things set down follow), things set down get `facing: None`, `PortSaid` gains `facing`, writer: a thing's connection point's axis is its facing vector in the thing's own axes (as its place is), and `IFCAXIS2PLACEMENT3D(#p,$,$)` where it has none; run-end ports keep their axis (it is the run's own direction, which they have); the "which way it faces is not written" line is gone; `import ifc` reads the facing onto the thing's connection, `import maker-model` onto the row; a point whose axis is along none of the six is written with no facing and counted (own wording in the note). SPEC.md updated.
Honest: the world and ifc code was written before this round's new tests (call214r3 in world and ifc), so those were not run red first; they were then run green (world 5, ifc 3 new). Old job fixture: crates/world/tests/data/job-with-connections-before-round-3.json was written by the tree I started from (new job, starter items, starter connections, nl-2018.ifc maker model) and loads and saves back byte for byte.
Round 3 runs: world 672 passed, cli 45, ifc 111, none failed; 57 command-line demonstrations rc 0 with no FAIL (maker-model, connections, ifc, ifc-whole, ifc-runs, export, judged-1125b among them; d2, house need the window); portmap, three gates, licenses pass (connection_row_facing added to PORTMAP.md). `plumbline sixteen` file sha256 880d9fd3c364b5f77ad2abc6... unchanged (a saved job holds no file axis). ifc-check on the sample's exported IFC and the six demo files: "IFC4: no fault against the schema and its rules". Patch rewritten.
