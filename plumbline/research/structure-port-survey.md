# Beams, posts, pad footings, joists and rafters: what BIMCraft has and what BuildOS has (2026-11-22)

A read-only survey by an agent on another model, all of BIMCraft's `crates/core/src` searched
(commit f202abd85d1c, MIT OR Apache-2.0), condensed by the lead. It is what call 164 rests on.

## BIMCraft

- **Beam**: `Element::Beam(Run)`, a run like a pipe (`model.rs:1586-1609`): type, level, start,
  end, offset (height at the start), end_offset, justify (Top, Centre, Bottom: `model.rs:1551`;
  a beam's default is Top, `runs.rs:57`). Its section is `RunType { width, depth, round,
  material }` (`model.rs:2467`). Made by the command `beam` (`commands.rs:577-586`, `runs_of`
  at `commands.rs:8776`): one beam for each segment of `points`.
- **Column**: `Column` (`model.rs:1526-1546`): type, level, at, angle, top_level, height,
  base_offset. Type `ColumnType { width, depth, round }` (`model.rs:2456`), no material. Two
  categories, `column` and `structuralcolumn`. Made by `stood_of` (`commands.rs:8662`); with no
  height and no top level, it stands to the level above (`top_param`, `commands.rs:8631`).
  There is no "post".
- **Pad footing**: `Element::Footing(Column)` (`model.rs:2148-2149`): "a column laid flat, its
  height its thickness"; `base_offset = top_offset - thickness` (`commands.rs:8674-8686`);
  thickness must be above zero; a default thickness of 400 mm.
- **Refusals** (`runs.rs:62-76`, `structure_tests.rs:60-76`): too short; "its ends are one over
  the other; a beam runs along, and what stands is a column"; "it slopes X°, more than the 75° a
  beam may"; a column's top must be above its base.
- **Quantities**: only in the IFC export: `member_quantities(width, depth, round, length)`
  (`ifc.rs:707-719`): Length, CrossSectionArea, OuterSurfaceArea, GrossVolume, for beams
  (`Qto_BeamBaseQuantities`, `ifc.rs:1105`) and columns (`Qto_ColumnBaseQuantities`,
  `ifc.rs:1061`). No weight of any member but rebar; no density table.
- **IFC**: beam `IFCBEAM .BEAM.` (`runs.rs:22-38`); column and structural column `IFCCOLUMN
  .COLUMN.`, pad footing `IFCFOOTING .PAD_FOOTING.` (`ifc.rs:1031-1064`); `LoadBearing` in the
  common property sets. The class table (`ifc/classes.rs`) has JOIST and LINTEL under IFCBEAM and
  POST, RAFTER, PURLIN, STUD under IFCMEMBER, as names only: BIMCraft never writes them.
- **Nothing**: no joist, rafter, purlin, lintel, header, post or truss as an element; no layout
  of members across a floor or a roof (only `timber.rs`, one wall, already ported as
  `framing::frame`); no section library (sizes are typed); no bearing, load path, span check or
  sizing for timber or steel (`reinforce.rs` is concrete, geometry only: "how many bars, how
  thick and how far apart is the engineer's").

## BuildOS before call 164

- `Segment`, `SegmentType` and `building/runs.rs` are already BIMCraft's `Run`, `RunType` and
  `runs::Geom` (PORTMAP rows 96, 104, 160-166); only "too short" of the run refusals was taken.
- A strip footing is held as a wall is (`Kind::Footing`, call 70); there is no pad footing.
- A wall's header is `header_length` LF from the wall's frame, sized for drawing only by
  `framing::header_depth`, a rule of thumb.
- The BIM file's writer (`crates/ifc/src/write.rs`) picks a class by a match on the shape and the
  kind; a run of an unknown kind falls to a cable segment. `IFCFOOTING .STRIP_FOOTING.` is the
  only structural class it writes; it reads none in.
- `Kind` is matched without a wildcard in two places (`components.rs` `Kind::stem`,
  `app/src/theme.rs` `kind_name`); everywhere else a new kind falls to the default unless given
  an arm (the survey lists each file).

## So

Ported as is: the beam as a run with its section and justify; the column's shape for a post and
for a pad footing laid flat; the three refusals; `member_quantities`; the IFC classes.
Ours to write: weight (from a weight per foot typed on the type), the list for the engineer, the
starter assemblies, and (call 165, later) joists and rafters counted across a floor or a roof
plane.
