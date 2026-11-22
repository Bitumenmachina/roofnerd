# Plan sheets on paper: what BIMCraft has and what BuildOS has (2026-11-22)

A read-only survey by an agent on another model (BIMCraft at f202abd85d1c, MIT OR
Apache-2.0; all of `crates/core/src` searched), condensed by the lead. It is what the
decision on existing, demolition and proposed sheets (plan step D10) rests on.

## BIMCraft (`crates/core/src`)

- **A display list, then writers.** `render.rs`: `Prim` (:88: `Fill` rings, `Line` points
  with a width, a colour, a dash and closed or not, `Image`), `Part` (:62), `View` (:189),
  `Drawing` (:215) with `fill`, `line`, `stroke`, `dashed`, `text`, `weigh` (pens become
  widths last). Arcs are polylines. Writers: `render::svg` (:4432, plans only),
  `render::png` (:4485), `pdf::pdf(&[(&Drawing, P2)])` (`pdf.rs:68`: PDF 1.4 written by
  hand, Flate streams, **no fonts**: lettering goes out as strokes and outlines). No DXF or
  DWG writer.
- **The plan pass** (`render.rs:2607 draw_plan`): walls cut at the view's cut height
  (`cut_outlines` :568, outlines from `walls::outline`), filled as one per colour so no seam
  shows (`cut_unions` :779), inked only on the outer edge of the union (`cut_edges` :897,
  `ink_cut` :976, `turn_corners` :986): that is the join clean-up. Poché is solid grey
  (`POCHE`); layers hatch (`hatch` :3230).
- **Doors and windows** (`opening` :3490): the gap filled with paper, jambs, the door's
  leaf and a 24-piece swing arc (`arc` :3684) by the opening's `flip_hand` and
  `flip_facing`; a window is three lines.
- **Rooms**: name, number and area lettered at the room's own point (:2867-2913).
- **Dimensions are placed elements** (`Dimension`, `model.rs:1060`; drawn by
  `render/dimensions.rs`): nothing makes a string along a wall by itself.
- **Lengths**: `Units::length` (`units.rs:68`) writes `9'-10 1/8"`; `Units::scale(48)` writes
  `1/4" = 1'-0"` (:123).
- **Phases** (`phasing.rs`): `Status { New, Existing, Demolished, Temporary }` (:40),
  filters (`filters()` :125: Show All, Show Complete, Show New, Show Previous + Demolished,
  Show Previous + New), overrides (`overrides()` :146: existing halftone; demolished dashed,
  red), applied by `Look::with` and `Look::dress` (`graphics.rs:1102`, `:1231`). Its
  Renovation menu makes the three plans (existing, demolition, new) from one level. Wall
  joins do not look at phase.
- **Sheets**: `Sheet` and `Viewport` elements (`model.rs:1932`, `:2006`), `sheet::draw`
  (`sheet.rs:310`) at 5 dots to the mm, `model::paper(size)` (:1966: ARCH A to E, ANSI A to
  E and the ISO sizes), a built-in title box (`sheet.rs:167-198`) and project title blocks
  (`titleblock.rs`: fields PROJECT, SHEET_NAME, SHEET_NUMBER, SCALE, DATE).
- **Pens** (`pen.rs`): 17 widths in paper mm (:34), by scale (`LineWeights::at`, :154);
  cut is pen 10 (0.35 mm at 1:100), seen is pen 4 (0.18). Its "ANSI" weights are its own
  numbers, not a published table.
- **A lettering font of strokes** (`font.rs`: `width` :231, `strokes` :237), capitals only,
  needing no font file.

## BuildOS

- The window's Plan is the 3D scene seen from above: each wall its own rectangle (no clean
  join), no door swing ("which way a door swings is not in the model"), no room label, no
  dimension. None of it works without the engine.
- The model has BIMCraft's wall joins already (`building/walls.rs`: `outline`, `cap`,
  `centre_line`), but no public function gives a level's joined walls; room outlines and
  areas are public (`building::room_outlines`). An opening has no hand and no swing.
- No PDF or SVG is written anywhere. `flate2` is already in the lockfile (through the PDF
  reader), which is all BIMCraft's hand-written PDF needs.
- The work mark (`Work`, `WorkStatus`) is per thing; `view::work_place` gives build,
  existing, demolish. A wall to demolish bounds no room, so the rooms the model works out
  are the proposed ones; there are no as-it-stands room areas.
- Two ways of writing feet and inches (`roofing::drawing::feet_written`: `3'-6 1/2"`;
  `world::units::feet_as_ft_in`: `3' 6 1/2"`).

## So

Ported as is: the display list, the wall-cut pass (union, outer edge, mitred ink), the
door leaf and arc, the paper sizes, the pens, the stroke font, the PDF writer, the scale's
wording, the title box. In another shape: phases (BuildOS's one mark per thing gives the
three filters directly), the sheet (a function of a job, a level, a filter and a scale,
not an element). Ours to write: a dimension string made along a wall by itself, a door's
hand, the as-it-stands rooms, a dash measured on paper.
