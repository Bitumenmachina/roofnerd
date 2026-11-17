# A LiDAR scan as the start of a renovation job: what to bring in, and how it would pipe (2026-11-17)

Patrick, 2026-11-17: he scans with Scaniverse, is content to clean the scan in
another program and to trace over it, and wants the import that pipes well for
renovations, "and then being able to work off that". This is a survey from each
source's own pages (linked at the foot), with what was not confirmed said so.
Nothing here is built. PROPOSALS.md 32 and 33 are the proposals.

## What was found

- **What the phone app writes** (its community forum; not its manual): meshes as
  OBJ, FBX, GLB, USDZ, STL; point clouds as PLY and LAS.
- **How good a phone scan is.** One study of interior measurements off scans found
  them between 2.2 cm short and 0.3 cm long, repeating to 0.6 to 1.4 cm. Others
  found errors of 10 to 35 cm on scans made carelessly or over a large area, and say
  error grows with the area walked. So: good enough to take off SF and LF for a
  price; not good enough, unchecked, for anything cut to fit (cabinets, tops,
  stairs, a beam).
- **Cleaning outside the program.** CloudCompare cuts a cloud to the house, removes
  what is not building, and cuts a section at a height. Its drawing of a section's
  outline works on point clouds and not on meshes, and its own author says it is
  for simple outlines; for walls, drawing by hand over the section is what is
  advised. It saves sections as DXF. (CloudCompare is GPL: used beside this
  program, never linked, as IfcOpenShell is.)
- **Renovation in the BIM standard.** IFC4 gives every wall, door, slab and so on a
  Status in its common property set: New, Existing, Demolish, Temporary. It is
  there for renovation work. IFC2x3 has none; programs wrote their own.
- **Rooms that arrive as walls.** Apple's RoomPlan gives walls, doors and windows as
  sized things, not a mesh; kept as its JSON they stay so, exported as USDZ mesh
  they do not. Which scanning apps hand that JSON over was not confirmed.
- **Readers under permissive licences.** An E57 reader in Rust is MIT. PLY, LAS,
  OBJ and glTF readers exist; their licences were not confirmed here and
  `tools/licenses.sh` decides.

## What follows from it

**Bring in the point cloud, not the mesh, as the thing to work from.** A point
cloud (PLY first) is what the cleaning program handles best, is the simplest file
to read, is in real units, and cuts cleanly into a plan. The mesh (GLB or OBJ) is
for looking at in 3D and can come second. A splat is a picture and is out.

**Use what the program already has.** It already lays a sheet under a level,
snaps the Wall tool to lines on it and marks everything traced as Assumed with its
source. A scan cut at 4 ft above a floor and looked at from overhead is such a
sheet, already to scale. So the shortest pipe is: cloud in, cut, traced with the
tools that exist. Two things a scan gives that a sheet does not, both cheap: the
floor and ceiling heights (the two heights where most points lie), so levels come
in with their heights; and any length, picked between two points.

**The scan is Assumed; a tape makes it Known.** That is the loop the program is
built on. A renovation job would open with every traced value Assumed and sourced
to the scan, and the verify-in-field list would send someone to tape the few
lengths the money rides on. Two taped lengths typed against the scan (a long wall,
a diagonal) say how far off the scan is, and that is written on everything taken
from it.

**"Working off it" needs one thing the program does not have: what is existing,
what comes out, what is new.** Without it a scan is only a faster first tracing.
With it: existing work is drawn light and not priced; work to demolish is drawn
dashed and priced as demolition; new work is drawn heavy and priced as now; the
plan can show before, demolition, and after from one job; and the BIM file carries
the standard's Status both ways. This touches what the job holds and its saved
file, so it is the larger and more careful piece.

## The pipe, end to end

1. Scan a floor at a time in mesh (LiDAR) mode; write out the point cloud as PLY.
2. In CloudCompare: cut away what is outside, level it, turn it so north is up,
   take out furniture if wanted, save as PLY.
3. Bring the PLY in under a level. The program finds the floor and the ceiling,
   offers the level's height, shows the cloud as a ghost in 3D and its cut as a
   sheet on plan.
4. Type two taped lengths; the program says how far off the scan is.
5. Trace walls, doors and windows over the cut. All Existing, all Assumed.
6. Mark what comes out; draw what is new. Price demolition and new work.
7. The verify-in-field list, by dollars: tape those. Estimate, workbook, BIM file.

## How long (not prototyped; a builder's estimate)

| Piece | Days |
|---|---|
| PLY in; ghost in 3D; cut as a sheet to trace and snap to; levels from the scan | 3 to 4 |
| Existing, demolish, new: in the job, on the plan, in the estimate, in the BIM file both ways | 3 to 5 |
| Two taped lengths checking the scan; a length picked off the scan answering a field question | 1 to 2 |
| LAS, E57, a DXF section, a mesh to look at: each | about 1 |
| Walls found in the scan without tracing; a RoomPlan file read as walls | not estimated: research |

## Sources

- https://community.scaniverse.com/t/which-formats-can-scaniverse-export-to/50
- https://saemobilus.sae.org/articles/bias-repeatability-measurements-3d-scans-made-using-ios-based-lidar-2021-01-0891
- https://jm.um.ac.ir/article_45986_597060eb27e16a7834e2eb271332da6d.pdf
- https://www.cloudcompare.org/doc/wiki/index.php/Cross_Section
- https://www.cloudcompare.org/doc/qCC/CloudCompare_cross_section_tool_V2.pdf
- https://ifc43-docs.standards.buildingsmart.org/IFC/RELEASE/IFC4x3/HTML/lexical/Pset_WallCommon.htm
- https://forums.buildingsmart.org/t/phasing-in-ifc/3155
- https://developer.apple.com/documentation/roomplan/capturedroom/usdexportoptions.md
- https://docs.rs/crate/e57/latest
