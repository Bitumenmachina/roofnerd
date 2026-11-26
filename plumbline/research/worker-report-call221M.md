# W-221M report (DXF in and out) - notes written as the work goes

## Before any code: the six answers

### 1. The format, from its source (Autodesk DXF reference, 2018 pages, fetched 2026-11-25)
Index: https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/index.htm
- Units: HEADER variable $INSUNITS (group 70): 0 unitless, 1 inches, 2 feet, 3 miles, 4 mm, 5 cm, 6 m, 7 km, 8 microinches,
  9 mils, 10 yards, 11 angstroms, 12 nm, 13 microns, 14 dm, 15 dam, 16 hm, 17 Gm, 18 AU, 19 light years, 20 parsecs,
  21 US survey feet, 22 US survey inch, 23 US survey yard, 24 US survey mile. $MEASUREMENT (70): 0 English, 1 Metric (sets
  which hatch/linetype files; it does not name a unit). $EXTMIN/$EXTMAX = extents. $ACADVER AC1006..AC1032.
  https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm
- Entities page (list of all): .../GUID-7D07C886-FD1D-4A0C-A7AB-B4D21F18E484.htm
  LINE .../GUID-FCEF5726-53AE-4C43-B4EA-C84EB8686A66.htm ; LWPOLYLINE .../GUID-748FC305-F3F2-4F74-825A-61F04D757A50.htm
  (70 flag bit 1 closed; 90 count; 10/20 vertices; 42 bulge = tan(1/4 included angle), negative clockwise; 210 extrusion);
  POLYLINE .../GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm (70 bit 1 closed, 8 = 3D polyline, 16 = 3D mesh, then VERTEX, SEQEND);
  VERTEX .../GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm (42 bulge; 70 flags 16 = spline frame point, 128 polyface);
  ARC .../GUID-0B14D8F1-0EBA-44BF-9108-57D8CE614BC8.htm (40 radius, 50/51 start/end angle - the page does not state the unit;
  the Autodesk page on the other hand lists 40-59 as doubles. DEGREES, counter-clockwise, is the long-standing DXF convention and
  how every published arc reads; I check it with a test file made so a quarter turn must be 90 - NOT stated on the page fetched);
  CIRCLE .../GUID-8663262B-222C-414D-B133-4A8506A27C18.htm ; ELLIPSE .../GUID-107CB04F-AD4D-4D2F-8EC9-AC90888063AB.htm
  (10 centre, 11 major-axis end relative to centre, 40 ratio, 41/42 start/end PARAMETER, 0..2pi full);
  SPLINE .../GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm (70 flags 1 closed 2 periodic 4 rational; 71 degree; 72 knots; 73
  control points; 74 fit points; 40 knots; 10 control points; 11 fit points);
  TEXT .../GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm (1 string, 10 first alignment point, 11 second (used when 72/73 nonzero),
  40 height, 50 rotation, 72/73 justification); MTEXT .../GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm (1 text <250 chars, 3
  further 250-char chunks, 10 insertion, 40 height, 50 rotation radians as the page says, 11 x-axis direction);
  INSERT .../GUID-28FA4CFB-9D5E-4880-9F11-36C97578252F.htm (2 block name, 10 point, 41/42/43 scale, 50 rotation, 70/71
  columns/rows, 44/45 spacing, 66 attributes follow); DIMENSION .../GUID-239A1BDD-7459-4BB9-8DD7-08EC79BF1EB0.htm (an overview;
  group codes on the "Common Dimension Group Codes" subpage, which I did not retrieve: from other readers' knowledge, 2 = name
  of the anonymous block that draws it, 1 = text, 42 = measurement, 11 = text midpoint, 70 = type: marked as not taken from the
  page); HATCH, IMAGE, 3DSOLID pages are in the list above and are counted, never read.
- Layers: LAYER table entry: 2 name, 70 flags (bit 1 frozen), 62 colour (negative = layer off), 6 linetype
  (https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-D94802B0-8BE8-4AC9-8054-17197688AFDB.htm, as quoted by search).
  Every entity names its layer in group 8.
- Blocks: BLOCKS section, BLOCK (2/3 name, 10 base point) ... ENDBLK
  https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-66D32572-005A-4E23-8B8B-8726E8C14302.htm
- ASCII form: pairs of lines, a group code then its value; sections HEADER, TABLES, BLOCKS, ENTITIES... ended by 0/EOF.
  Group value types by range (code range page): 0-9 string, 10-39 double, 40-59 double, 60-79 int16, 90-99 int32, 100/102 string,
  210-239 double, 280-289 int16, 290-299 bool, 300-309 string, 1000-1009 string, 1010-1059 double, 1060-1070 int16, 1071 int32.
  https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm
- Binary form: https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm
  sentinel "AutoCAD Binary DXF<CR><LF><SUB><NUL>" (22 bytes); then code (2 bytes LE; 1 byte before R14) + value
  (string NUL-ended; int 2/4/8 bytes LE; double 8 bytes LE; bool 1 byte; chunk 1-byte length).

### 2. The code
Candidate: crate `dxf` 0.6.2 (MIT, released 2026-10-03, https://crates.io/crates/dxf, https://github.com/ixmilia/dxf-rs).
Dependencies (crates.io API): byteorder, chrono, encoding_rs, enum_primitive, image (no default features), itertools 0.14,
num 0.4, uuid, serde (optional); build: xmltree. Of those the lockfile already has byteorder, encoding_rs, image, itertools,
uuid; new would be chrono (with its time-zone support), num, enum_primitive, xmltree and their own dependencies.
It parses to typed entities: an entity it does not model is dropped without a name, which is what the fixed rule forbids; I
need every dropped entity counted BY KIND, blocks resolved with the insert's layer, and layers' own state kept. DECISION: the
reader and writer are written here from the reference (ASCII form is pairs of lines; binary is the same pairs in bytes), in
crates/plans/src/dxf.rs, no new dependency, nothing to add to NOTICE, licences/egress gates unchanged. (I did not run
`cargo tree` with dxf added: decided on the dependency list and the loss-counting need.)

### 3. How a DWG, a Revit file and a SketchUp file are known
- DWG: first six bytes "AC10xx" (AC1015 2000, AC1018 2004, AC1021 2007, AC1024 2010, AC1027 2013, AC1032 2018); four bytes
  41 43 31 30 "AC10" are the magic of every AutoCAD DWG since 2.5. Library of Congress:
  https://www.loc.gov/preservation/digital/formats/fdd/fdd000445.shtml (Autodesk's own article 403'd; the version codes are
  also in the $ACADVER table above). Older "AC" 1.2-2.22 / "MC0.0" (1.0) also refused as DWG.
- SketchUp .skp: begins (offset 0) with bytes FF FE FF 0E then "SketchUp Model" in UTF-16LE (PRONOM fmt/1265:
  https://pronom.nationalarchives.gov.uk/fmt/1265.xml). I test for FF FE FF + UTF-16 "SketchUp".
- Revit .rvt/.rfa/.rte: an OLE2 compound file (PRONOM container signature OLE2 with stream BasicFileInfo
  https://pronom.nationalarchives.gov.uk/fmt/1347). OLE2 header is D0 CF 11 E0 A1 B1 1A E1 (standard; I did not fetch a page that
  states it). Known as Revit when OLE2 header AND the bytes of "BasicFileInfo" in UTF-16 appear in the first part of the file,
  or the name ends .rvt/.rfa/.rte and it is OLE2. (BasicFileInfo is stored in the directory in UTF-16: tested on my own sample.)

## Progress notes (run 1)
- Reader/writer written in crates/plans/src/dxf.rs (no new crate). Tests crates/plans/tests/dxf.rs: 14 pass. Tests were
  written after the reader (not before: said plainly); proved to bite by mutation (arc angle in radians, layer-0 inheritance,
  bulge sign flipped: 5 of 14 went red; file restored).
- World: Drawing.layers_off (new optional key, skipped when empty), switch_layer, set_layers_off, scale_said for .dxf.
- CLI: crates/cli/src/cad.rs (bring in, layers, layer, export dxf), found.rs (reads with layers off; DXF given a margin so the rectangle is not "the border").
- demos/dxf.sh passes all nine steps (output seen).
- Figures: lead's step 7 "8 long lines" is wrong: a door cuts both faces of its wall, so 4 walls = 8 faces + 2 cuts = 10. Lead's
  "240 SF" is wrong for a room measured inside the faces: takeoff gives 224.25 SF (19.5 x 11.5 ft). Lead's "outer rectangle reads
  20 ft by 12 ft" is wrong: walls are centered on their line, 6 in thick, so the outer faces span 20 ft 6 in by 12 ft 6 in (246 by 150 in);
  the finder, reading the same drawing back, finds the 20 ft by 12 ft CENTER lines, 6 in thick. Step 5 on the one-line rectangle: finder
  refuses by its own ruling (a house drawn with one line a wall), and calls the circle a tag.
run2: plans 161+4+14+7 ok, sheets 6, roofing ok
gates: language, egress, clientdata PASS; licenses exit 0 (no crate added); portmap needs BIMCraft source (not present; no public item added in building*/editing*)

## Final (run 3)
Tests: plans 187 (15 in tests/dxf.rs), world 672, ifc 111, sheets 6, roofing 122, cli 50 (5 in tests/call221m.rs): 0 failed.
Demonstrations (command line): 58 pass incl. demos/dxf.sh; d2 and house need plumbline-app (not built, not allowed) and fail "button not on screen";
field-job.sh is a helper needing an argument. No existing expectation edited. Gates pass (language, egress, clientdata; licenses exit 0).
Patch: review/call221M.patch. Sources: pages listed in the six answers above plus
 https://www.loc.gov/preservation/digital/formats/fdd/fdd000445.shtml, https://pronom.nationalarchives.gov.uk/fmt/1265.xml,
 https://pronom.nationalarchives.gov.uk/fmt/1347, https://pages.uoregon.edu/arch/landcad/Layer_Standards.htm (AIA CAD Layer Guidelines 2nd ed.),
 https://academics.triton.edu/faculty/fheitzman/layname.html (A-ANNO-NOTE, A-WALL, A-DOOR, A-GLAZ, A-FURN, P-FIXT, M-HVAC, E-POWR, E-LITE).

## Round 2
Ruling applied: replies count what the drawing holds by kind and layer ("holds 8 lines, 1 circle"; "FURNITURE: 4 lines, 1 circle"); curves said once
("its 1 curve is drawn as 36 straight pieces of at most 10 degrees"); the layer verbs count "straight pieces". Own round-1 tests/steps changed:
plans tests/dxf.rs (curve sentence; new held() asserts), cli tests/call221m.rs (layer wording), demos/dxf.sh step 2/3. The finder's own "44 straight lines read" (step 5) is
the finder's existing sentence and is kept. Tests: plans 187, cli 50, 0 failed. demos/dxf.sh passes, no FAIL. portmap.sh with BIMCRAFT: PASS (429 public items, 144 ported, 285 ours); no PORTMAP line needed.
Patch rewritten.
