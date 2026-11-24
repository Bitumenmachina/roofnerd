# W-197I report (worker, contract 197I; part 1 = call 197 points 1 to 7, part 2 = call 199)

Working copy: SCRATCH/wtB at 86501d4, nothing committed. Notes written as I went.

## Before any code

### 1. BIMCraft searched (all of /home/user/ipedrax/bimcraft)

`grep -rln -i "ifc" crates apps` gives 76 files (crates/core, crates/rvt, crates/ui-egui, apps/*). Read for how a
fitting, a terminal, a footing and a roof are read or written:

- **Reading**: only `crates/core/src/import.rs`. It builds BIMCraft's model out of the file. A fixture, a fitting,
  furniture is a "generic component": a family of its IFC class with the shape taken from the body
  (`NOT_GENERIC`, import.rs:44-80, lists what never comes in as one: openings, spatial elements, grids and
  **ports**: `IFCDISTRIBUTIONPORT`, `IFCPORT`). `grep -n "IFCRELCONNECTSPORT\|IFCDISTRIBUTIONPORT" ` over import.rs:
  only that exclusion; **no port and no relation between a fitting and a run is read anywhere**. Footings:
  import.rs:2424-2426 reads a footing whose sort is not `STRIP_FOOTING`/`FOOTING_BEAM` as an isolated (pad)
  footing, a column laid down; strips go with the walls (2525). Roofs: import.rs:2003-2007, an `IFCROOF` and
  the `.ROOF.` slabs under it are one roof, taken from the slabs' faces (a roof of BIMCraft's own kind), not
  from a list of how it was made.
- **Writing**: `crates/core/src/ifc.rs:1042` (`IFCFOOTING .PAD_FOOTING.` for an isolated footing),
  `:1305` (`IFCROOF`), `family.rs:94-97` (a family of a category is written as `IFCFURNITURE`,
  `IFCSANITARYTERMINAL`, `IFCPIPEFITTING`, with its connectors drawn in the family, `family.rs:132`), and
  `ifc/classes.rs` (the class table with each class's sorts).
- What is taken: the **strip/pad split by sort** (a rule, not code: written fresh here against the same
  enumeration), and the **class names** of what is a fixture, a fitting, furniture. Nothing is copied as code.
- What is not there to port: reading a fitting's run by port or relation (BIMCraft reads none); a roof made
  again from how it was made (BIMCraft rebuilds from the faces); a thing's size as a box from its body (BIMCraft
  measures every shape). Recorded in `PORTMAP.md` ("Ported outside the building model", call 197 bullet).
  `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh`: see "Gates" below.

### 2. The tree as it was (f5 binary, a0d8477; `sixteen` then `export ifc` then `import ifc` into an empty job)

First job (h1.json, 3 levels): 20 walls, 9 doors, 17 windows, 2 floors, 10 rooms, 4 footings (136.00 LF,
10.07 CY), roof 4 planes 1,408.72 SF, eave 144.00 LF, hip 90.00 LF, ridge 12.00 LF, 22 fixtures, 5 pieces of
equipment, 41 fittings (39 written: 2 of the 41 have no size and are not written, with the 3 hot-water pieces
they join), 60 pipes written (3 more with no size are not written), 23 ducts, 16 cables, 2 conduits, 9 systems.
Open ends (takeoff, the program's own count): supply air 4, return air 0, refrigerant 0, cold water 1, hot water
0, sanitary 1, vent 1, power 1.

Second job before this work: levels 3, walls 20, doors 9, windows 17, floors 2, rooms 10, ducts 23, pipes 60,
cables 16, conduits 2; no fixture, equipment, fitting, footing or roof; doors "door 1".. (names lost); systems
"Supply Air", "Domestic Cold Water" (capitals of the kind, not of the file); return air on no system; open ends
air 15 (the judge's figures).

**What differs from the lead's figures, and which is right** (the lead's figures are the first job's own; where
the second job cannot have them, it is because the file does not hold what they rest on):

a. **39 fittings: 37 come in, not 39.** The change that puts a fitting where runs meet (`fitting`) refuses two of
   the first job's elbows at (16.3, 22.7): "pipe-0151 and pipe-0160 leave (16.3, 22.7) the same way, one over the
   other" (and "pipe-0151 stands upright: no length on the plan to be cut along"). In the first job a third run's
   end (the hot riser) touches the hot main's elbow joint; the sample's hand-made connectors joined two of three.
   The tool is right to refuse it and the first job's joint is a hand-made oddity. Two more of the 37 come back as
   Tees, not Bends (the first-floor trunk at (19, 17) and the return riser at (21.2, 27): a third end meets
   there): the kind is worked out from the runs, not copied. HELD FOR A RULING (below).

b. **Open ends "air 4, cold 1, sanitary 1, power 1, refrigerant none" are not reproduced.** Two ends that meet
   each other are joined with no fitting, so a fitting is not what closes them: a missing fitting changes the
   takeoff's count of fittings, not the open ends. What closes a run's end at a fixture is the **connector** the
   first job's plan put on the fixture "where the plan shows the run arriving" (sample.rs:512-565): up to 0.9 ft
   off the fixture's box. The file holds no port and no relation: only each thing's place and body. The change
   that joins a run to a thing (`connect`) takes only an end inside the box. Brought in, 22 run ends inside
   a box are joined; the others read open. Second job per system, with the same network count run on both:
   supply air 13 (first 4), return air 2 (0), refrigerant 1 (0), cold water 5 (1), hot water 4 (0), sanitary 5
   (2; vent 5, 2), power 1 (1), lighting 1 (0). Every extra end is 0.13 to 0.90 ft from a fixture or piece of
   equipment's box (worked offline from the written file: `review/w197/openat.txt`); the 4 supply ends of the
   first job are 3.6 to 8 ft from any thing. HELD FOR A RULING: join an end to the nearest fixture or equipment
   within one foot, flagged Assumed (the figures then are the first job's; it is an inference nobody drew), or
   have the writer put ports in the file (a format change), or leave them open and say so (what is built). Built:
   the last, with the note "N runs ... could not be joined".

c. Hot water open ends: the 3 hot-water pipes with no size are not in the file, so the hot-water network of the
   second job has those places missing (not a figure the lead gave).

### 3. The roof's property set, proposed and built as proposed

Name: `Plumbline_Roof` (the existing set is `Plumbline_Values`; never `Pset_`). On the `IFCROOF`, by
`IFCRELDEFINESBYPROPERTIES`, only for a roof with a footprint (a roof of planes drawn by hand writes none).

| property | type | unit | holds |
|---|---|---|---|
| `Outline` | `IfcPropertyListValue` of `IfcLengthMeasure` | feet (the model's own) | the corners round the eaves, east then north, in order, anticlockwise: x1 y1 x2 y2 ... (the overhang is in it: the outline is at the eaves) |
| `EdgeRise` | `IfcPropertyListValue` of `IfcLabel` | in 12 | one word for each edge (from corner n to n+1): `6:12` or `gable` |
| `EaveHeight` | `IfcPropertySingleValue` `IfcLengthMeasure` | feet | how high the eaves are above the datum |
| `Basis` | `IfcPropertySingleValue` `IfcText` | | how sure the outline, each edge and the eave height are, and where each came from |

One roof as it reads in the file (a 26 ft by 18 ft eave outline, 6:12, a gable on the second edge, eaves at 9 ft;
written by the built program, `review/w197/g.ifc`; the 4th edge in it is 6:12 because only edge 2 was made a gable):

```
#169=IFCROOF('1zeMCYPAC6Kz4JUcZhRZxj',$,'Main roof',$,$,#168,$,$,.NOTDEFINED.);
#171=IFCPROPERTYLISTVALUE('Outline','the corners round the eaves, each as east then north, in feet, in order',(IFCLENGTHMEASURE(-1.),IFCLENGTHMEASURE(-1.),IFCLENGTHMEASURE(25.),IFCLENGTHMEASURE(-1.),IFCLENGTHMEASURE(25.),IFCLENGTHMEASURE(17.),IFCLENGTHMEASURE(-1.),IFCLENGTHMEASURE(17.)),$);
#172=IFCPROPERTYLISTVALUE('EdgeRise','what each edge does, the edge from the first corner to the second first: its rise in 12, or a gable',(IFCLABEL('6:12'),IFCLABEL('gable'),IFCLABEL('6:12'),IFCLABEL('6:12')),$);
#173=IFCPROPERTYSINGLEVALUE('EaveHeight','how high the eaves are above the datum, in feet',IFCLENGTHMEASURE(9.),$);
#174=IFCPROPERTYSINGLEVALUE('Basis','how sure each is, and where it came from',IFCTEXT('outline (Assumed; made-up plan); edges (1: Assumed, made-up plan; 2: ...); eave height (Assumed; made-up plan)'),$);
#175=IFCPROPERTYSET('2OZJbewzg$O9GgiiUerHNg',$,'Plumbline_Roof',$,(#171,#172,#173,#174));
(and the relation tying #175 to #169)
```

Enough to make the same roof: the roof in the model is a `Footprint` (points at the eaves, one `Rise` per edge,
`Pitch(n)` or `Gable`, an eave height). The Roof tool is `solve::new_roof` over such a footprint (its pitch on
every edge), and a gable made on an edge is `solve::edit(Change::Rise{edge, Gable})`, which only replaces that
edge's rise before it solves again. The footprint the reader builds from the set is the footprint those
changes leave, so it is made by the one change (`new_roof`) with every edge's rise given at once; the solve
is by the same arithmetic. Test: the house (hip, four 6:12 edges) and the gable (24 x 16, two gables) both come
back with the same outline, rises, eave height, and the same plane areas and line lengths (1,408.72 SF, 144.00
LF eave); the file's roof slabs' areas (computed from their faces) are set beside the planes' and a note says
so if they differ.

What is not in the set, and why: the overhang (it is already in the outline, which is drawn at the eaves; the
trim's overhang is worked from the walls under the eave, not entered); the planes (they follow from the above);
the roof's name and its work mark (the element's name, and `Pset_RoofCommon.Status`, which was already written).
Everything comes back Assumed, sourced to the file and the roof's number in it.

### 4. How a fitting names the runs it joins in the program's own file

**Only its place.** The writer (`write.rs`, `Shape::Box`) writes a fitting as `IFCPIPEFITTING` / `IFCDUCTFITTING` /
`IFCCABLECARRIERFITTING` with a body (a box the size of its section, centred where the runs meet) and its sort
(`BEND`, `JUNCTION`, `TRANSITION`, `CONNECTOR`); it writes no `IfcDistributionPort`, no `IfcRelConnectsPorts`,
no `IfcRelConnectsPortToElement`. What can be read from a file with only its place: where the runs' ends meet (the
box's middle) and its sort; so the reader asks the program's own change (`fitting`) to put a fitting there, which
finds the run ends at that point, works out the kind (bend, tee, cross, transition, coupling) from them, takes its
size from the runs, joins them with connectors, and refuses what it cannot (no run ends there, a third end leaving
the same way, a run passing through that cannot be cut). The file's sort is not used (it is worked out), so a
file that calls a tee a bend comes back as the tee.

## What is built (part 1)


Points 1 to 7 (crates/ifc/src/lib.rs, write.rs; crates/cli/src/main.rs):

1. Fixtures (sanitary terminal, flow terminal, outlet, air terminal, light fixture, stack and waste terminal),
   furnishings (furniture, furnishing element, system furniture) and equipment (the distribution elements and
   the energy, moving, storage and treatment devices, boards, boilers, pumps, fans, tanks...) are set down by
   `ed::place` (then `ed::turn` for an angle), each a box the size of its body (a rectangle as it is, a round body
   as its square box, an outline as the box round its corners), Assumed, its source the file, its name and its
   number there. Its kind is by what the file calls it. What it was ("what it is" in the file's values list)
   is Placed.what, so the takeoff groups it as the first job did. A body that is not read leaves the size
   Unknown with the reason, and the thing stays where it is placed. The same model twice sets nothing down twice.
2. Fittings by `ed::fitting` where runs' ends meet at the box's middle (kind worked out, sizes from the runs).
   Run ends inside a fixture's or equipment's box are joined by `ed::connect`. HELD: see answer 2 above.
3. Footings: `ed::footings` (strip: a straight box on its axis, top at its level's floor) and `ed::pad`
   (pad: a straight box, square to the plan); the type found or made (`footing_type`, `framing_type`); any
   other sort, a body not straight, a turned pad, a top off the floor: left out with that reason. Counted once
   if brought in twice.
4. The roof: written `Plumbline_Roof` (above); read and made by `solve::new_roof` from the footprint it
   gives (anticlockwise as the tool turns it), the work mark by the same `marks`; the planes' areas set beside the
   file's roof slabs (areas worked from their faces) and a note says if they differ. A roof with no such set is
   left out as before. Counted once if brought in twice.
5. Names: a door or window is called what the file calls it where nothing else in its wall goes by that (else with
   its number in the file); fixtures, fittings, footings, the roof likewise; a system keeps the file's name, and
   a system written by name (Return Air, which IFC has no word for) is read by that name.
6. After everything, the run ends that stand open where a thing not brought in stood are counted and said by
   what stood there ("N run ends read open here: each stood on things that were not brought in (n on plumbing
   fixture)"). A run's end meeting only another run's end is joined, not open: a left-out fitting changes no
   open end, only the count of fittings.
7. `plumbline gable` says "~ none: an eave is over it" where it is Assumed; "a outdoor unit" is "an outdoor unit"
   (`items`, drawn as). `with_article` in main.rs.

Files touched: crates/ifc/src/lib.rs, crates/ifc/src/write.rs, crates/ifc/tests/{whole.rs (new), ifc.rs},
crates/cli/src/main.rs, demos/{ifc-whole.sh (new), export.sh, model.sh}, README.md, PORTMAP.md. `crates/world/src`
is NOT touched by part 1.

## Expectations that moved (listed here before each was changed)

| where | wanted | gets | why |
|---|---|---|---|
| crates/ifc/tests/ifc.rs:31 (a_model_comes_in_as_storeys...) | made: no furnishings | `("furnishings", 1)` | the chair of the designer's model is set down as a furnishing (point 1) |
| crates/ifc/tests/ifc.rs:33-35 | left out: beam 1, furniture 1, stair 1 | beam 1, stair 1 | the same chair |
| demos/model.sh:30,33,34 | made list without "1 furnishing"; "not brought in: 3 things"; "1 furniture: nothing here to bring it in as" | "1 furnishing" in; "not brought in: 2 things"; the furniture line gone | the same chair |
| demos/export.sh:178-179 | "16 fixture or outlet: nothing here to bring it in as", "6 plumbing fixtures: nothing here..." | "22 fixtures", "5 pieces of equipment", "37 fittings", "1 pipe fitting: this program would not take it: " | the 22 fixtures, 5 pieces of equipment and 37 of the 39 fittings of the house written out come back (the two refused are named) |

Not moved: tests/write.rs and call180.rs (unchanged), `demos/found.sh`, `foundation.sh`, `spec.sh`, `structure.sh`,
`sweep-money.sh` (all pass as they were; sweep-money.sh:288 prints "NOT DONE (item 7): the roof is not read back" only
where roof slabs are left out, which now they are not for a roof this program wrote). Part 2 moved nothing.

## Red runs (tests first)

Part 1, `cargo test -p plumbline-ifc --test whole` before any change to `lib.rs`: `1 passed; 10 failed`
(`review/w197/red1.txt`), e.g. `p1_fixtures...: assertion left == right: left: 0 right: 22`,
`p3_footings_come_in_as_footings: left: 0 right: 4`, `p5 ...: left: ["bath door", ...] right: ["door 1", "door 1",
"door 2", ...]`, `p4_the_house_roof_comes_back...: the roof carries what is needed to make it again`,
`p3_a_pad_footing...: nothing in the model but its levels could be brought in: 1 footing (nothing here to bring it in
as)`. The one that passed was the existing behaviour kept (a roof without the program's own set is left out).
The test of the same model twice (`the_same_model_twice_counts...`) was written after the guards, not before.
Part 2, `cargo test -p plumbline-world --test call199` before `change_assembly_said` existed: compile errors
`no method named change_assembly_said found for struct Job` and `cannot find LeftAsIs in plumbline_world`
(`review/w197/red2.txt`); the figures were then driven red by hand on the way (Corner boards 0.00 LF, "no figure"
for the new figure, until the probe described below).

## What was run

- `cargo test -p plumbline-ifc --no-fail-fast`: 4+4+6+12 (1 ignored: the held open-end figures)+6, all pass.
- `cargo test -p plumbline --no-fail-fast`: all pass (2, 3, 3, 13). `cargo test -p plumbline-world --no-fail-fast`:
  every binary passes, incl. `call199` (4 new).
- `demos/ifc-whole.sh`, `demos/change-assembly.sh`, `demos/export.sh`, `demos/model.sh`, `found.sh`,
  `foundation.sh`, `spec.sh`, `structure.sh`, `sweep-money.sh`, `taper.sh`: all pass (IfcOpenShell 0.9.0.post1 from
  review/pylib: the house, and the gable roof with its set: "no fault against the schema and its rules").
- Gates: `tools/gate-language.sh`, `gate-egress.sh`, `gate-clientdata.sh --all`, `BIMCRAFT=/home/user/ipedrax/bimcraft
  tools/portmap.sh` (326 items), `tools/licenses.sh`: pass. The window crate was not built or run.
- `import ifc` of the house takes about 17 s in this debug build (each thing set down is one all-or-nothing change).

## Part 2 (call 199)

`Job::change_assembly_said` (crates/world/src/job.rs; `Changed`, `Rewritten`, `Left`, `LeftAsIs` exported), and
`change_assembly` is now `change_assembly_said` less what it says (same return as before, so the window compiles). Per
shared line: its price, waste, order, off, class and division are never touched. Its quantity (formula, entries,
stretch-out) becomes the new assembly's where it equals the old assembly's line; where it does not (a person's), or the
old assembly is not in the job, or the new quantity comes out in another unit, it stays; each is named. The new
figure is read by putting the new quantities beside the lines, working the thing's figures out again and
reading them (a figure like the wall's area below a wainscot exists only where a line names it), then putting back
only the lines. `apply --change` says: "N cost lines it already had now read as the new assembly's quantity ..." with
each "line: 200.00 SF to 140.00 SF (old formula to new formula, entries)", and "left as entered ... the new
assembly writes ..., which comes to ...". The figure stays Assumed where the entry the new assembly gives (wainscot
= 3 ft) is Assumed: the sheet reads `~140.00 SF ~$560.00`, the lead's figures flagged.

Tests and demonstrations that change one assembly for another, and what each read before and after:
`crates/world/tests/call167.rs:279-281` (Kitchen on a room, the same assembly: no change); `crates/cli/tests/cli.rs:208-243`
(a plain lap-siding wall to brick veneer: Studs $72.00 kept; Lap siding gone; no shared quantity differs: same);
`demos/foundation.sh:133` (a foundation wall to a basement wall: dampproofing 133.3333 SF before and after, same
formula); `demos/taper.sh:99` (a low-slope roof area from no assembly: first one given). All pass unchanged.
No expectation moved in part 2.

Window code that calls it: `crates/app/src/main.rs:1226` (an assembly picked from the list on what is selected) and
`:1245` (the same, on the other bare planes of a roof). Both take `(added, taken)` and say "{n} cost lines added" and what
came off. With this change a shared line's quantity is now rewritten by the same call, and the window says nothing of it
(the sheet shows the new figure). Owed to the window job: use `change_assembly_said` and say `rewritten` and `left`
(the words the command line uses), and `Status::Assumed` as the window passes it.

## Held for a ruling (nothing was bent to a figure)

1. 39 fittings: 37 come in; the two refused are a hand-made joint in the sample (a third run's end at an elbow).
2. Open ends at fixtures and equipment: the file holds no port, so a run's end off a box reads open (supply air 13,
   not 4; cold water 5, not 1 ...). Options: join to the nearest thing within one foot (Assumed, the evidence is
   in `review/w197/openat.txt`: every extra end is 0.13 to 0.90 ft off a box, the four real ones 3.6 ft or more), or
   write ports in the file (a change to the format), or leave it said. The ignored test
   `p2_each_system_reads_as_closed_as_it_was` holds the figures.
3. The roof's property set name and properties (`Plumbline_Roof`: Outline, EdgeRise, EaveHeight, Basis): the lead may ask
   for a name changed; nothing else depends on the name.

## Not right

- The model comes in slowly (17 s for the house).
- `Basis` is text; the reader does not read how sure each value was (all Assumed, as for every model).
- Furnishings and things with an unreadable body are placed at their placement point, size Unknown; they are not
  drawn until sized.
- A fitting's file sort (BEND, JUNCTION) is not used: its kind is worked out, so a file whose bend sits where three ends
  meet comes back a Tee (2 in the house).
- The window's message after Bring in lists what `import` says; it will now include the notes on joined runs and read-open ends;
  `demos/bringw.sh` pictures (made-up-house.ifc) show "not brought in" with 2 things, not 3 (not run: window).

Note on the patches: `review/call197I.patch` is the tree when part 1 was first whole; `review/call197I-199.patch` is
the final tree with both parts (it also holds the two later guards against the same model counted twice, the
`the_same_model_twice...` test, the README row for change-assembly, and part 2). Use the second.
