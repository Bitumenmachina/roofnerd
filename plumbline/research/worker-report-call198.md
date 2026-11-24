# W-198 report (contract 198: an item's shape as data)

## Before any code

### 1. BIMCraft's Family
- Ported already (PORTMAP.md): `blocks` (the forms of ten library families, `family.rs:1975`), `Block`
  (`Form`, `family.rs:516`), connectors and fitting kinds. Not ported: parameters, types, formulas, connectors-as-data, symbols.
- What a form's figures are there: `Expr` is `Number(f64)` or `Formula(String)` (`family.rs:233`), a formula
  over the family's parameters in millimetres, Revit-style: `Width / 2`, `-(Depth) + 60`, and a form's
  `visible` is a formula too (the bed's two pillows: `Width > 1200`). The evaluator (`family.rs:~700-1010`) has
  `if`, `and`, `or`, `min`, `max`, comparisons. A base cabinet's figures there are fixed (plinth 100, top 40, toe 60, lip 20);
  the "no more than a share of the thing" is ours (`forms.rs`, `c(fixed, of, share)`), which is why `min` is wanted.
- This step takes: a block as six figures that are formulas, a material name per block, `min` and `max`.
  Leaves (call 198 point 7): profiles and revolves (a sloped or turned block), a shape's own parameters, the BIM body.
  **Not taken, and needed: `visible` and comparisons (see the held point below).**
- Recorded in PORTMAP.md; `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh` checked at the end.

### 2. The design beside the code (call 198, points 1 to 5)
Operations the thirteen pieces of code use that the language did not have:
| operation in code | how a row says it |
|---|---|
| lesser of a fixed size and a share (`c(100, h, 0.14)`) | `min(100 mm, 0.14 * height)` |
| lesser of two sizes (`w.min(d)`) | `min(width, depth)` |
| a share of a size (`h * 0.4`, `d * 180 / 650`) | `0.4 * height`, `depth * 180 / 650` (already in the language) |
| a size in millimetres (`60.0`) | `60 mm` (new: a number with its unit, worked in feet, shown `60 ÷ 304.8`) |
| a fixed size in inches or feet | `4 in`, `3 ft` (same); also `cm`, `m`, `LF`, `SF` (so `min(net_area, 100 SF)` reads) |
| BIMCraft's own sizes scaled (WC 400 x 650 x 800) | `width * 200 / 400` etc. (plain numbers) |
| the order of two ends (`put` sorts each pair) | the reader sorts each "from, to" pair, as `put` does |
| `if w > 1200` (bed: one pillow or two), `if w >= 762` (sink: a divider) | **not in the language, and not in the design: HELD, see below** |

- `min`/`max` added without changing how any formula is read: `min` and `max` are functions only with a
  bracket straight after (a name before a bracket was an error); the comma was an error; a number followed by a name
  was an error unless that name is a unit word now. `tests/shapes_formula.rs` (red first, green after: quoted
  at the end) and the whole `plumbline-world` suite (count at the end) hold it.
- Axes and unit: across from the middle (plus to the right of the front), back from the back (the wall side)
  toward the front, up from the foot; every figure a formula in feet over `width`, `depth`, `height` and
  worked as a length (dimension 1); a fixed size is written with its unit; a figure that comes out as a
  count or an area is refused, naming the block and the column.
- Everything else in points 1 to 5 agrees with the code. A shape carries no status (the rows are a drawing).

**Held for a ruling: the bed and the sink.** Each has a block that is there only when the thing is wide
enough: the bed's pillows (one across the head when the bed is 1200 mm or less wide, two when wider) and the sink's wall between two bowls
(there when 762 mm wide or more, 30 in). A row can say a figure but not "this block is here only when". BIMCraft says it
as `visible: "Width > 1200"`. It needs: a column "There when" on a row (BIMCraft's `visible`) with one comparison (`>`, `>=`)
in the language, or the rows written with the block's size made nothing and nothing-sized blocks left out (which
I did not do: it would make a block that "is there" and has no size). The code for these two stays; eleven shapes are rows.

### 3. The starter table `library/shapes.csv`
Columns: `Shape, Made of, Form, Across from, Across to, Back from, Back to, Up from, Up to`, one row a block, in the order the code gives
them. Form is `box` or `round`. Made of: timber, worktop, porcelain, fabric, steel, enamel. Base cabinet (see below for the rows as written).

### 4. The saved job
Gains a top-level `"shapes"` list, written only when the job holds a shape of its own or one an item names
that differs from the starter; `{ "name", "blocks": [ {"made","form","across":[from,to],"back":[..],"up":[..]} ] }`. Left out
when none, so a job that names none saves byte for byte as before. A job saved before has no `"shapes"`; items name starter shapes
that live in the program (as the starter assemblies do), so every thing set down has the blocks it had. A job that names a
shape it does not carry and the starter lacks is refused on the way in, naming the shape.

## Built, in order

1. **The language** (`formula.rs`): `min(a, b, ...)`, `max(...)`, and a number with its unit (`in ft mm cm m LF SF`).
   `tests/shapes_formula.rs`: run red against the unchanged `formula.rs` (4 of 4 failed: "',' is not something a formula can hold"),
   green after (4 passed).
2. **The shape** (`building/shape.rs`, new): Piece/Shape, compile and work with the same reader and worker as a cost line.
   `library/shapes.csv`: 11 shapes, 37 rows. Starter first run red: "0 comes out as a plain number" (see the point below).
3. **THE HOLDING TEST, run while both the code and the rows existed** (`cargo test -p plumbline-world --test shapes -- --nocapture`):
   `held: 11 shapes, 33 shape-and-size pairs, 111 blocks compared` (each figure within 1e-8 ft; sizes [0.4,0.3,0.5],
   [3.0,1.75,2.875] (the 36 in vanity), [6.5,3.0,2.6]); `test result: ok. 2 passed`. Gate proved red: one figure in the
   base cabinet's first row changed (0.14 to 0.15 of the depth) failed it at "BaseCabinet at [0.4, 0.3, 0.5], block 1: lo[1] -0.105 against -0.108"; put back.
4. Only then the code was taken out (`forms.rs`: eleven arms gone; the bed and the sink stay as code, held). Before it went the
   code's blocks were written once to `tests/shapes_golden.txt` (111 lines); the test now holds the rows to that file:
   `held: 11 shapes, 33 shape-and-size pairs, 111 blocks compared`, `2 passed`, after.

Point for the lead: **a plain `0`** is taken as no distance (a block starts at the foot, the back, the middle) although the
design says a fixed size is written with its unit; the alternative was `0 ft` in 30 cells. One-line rule in `shape.rs` `figure`.

5. **The chest as two tables**: `tables::shapes_out`, `shapes_in`, `shapes_from_rows`; `export shapes` and `import shapes` on the
   command line. Shapes brought in that are the starter's own carry nothing. Items refuse a shape that is nowhere
   (`items_in`, `editing::item`, `editing::item_drawn_as`, `file::load`), the message the old one was with the job's shapes added to
   "There are:" (so no old message moved). `model::check_item` no longer asks which shapes exist (it cannot see the job); it refuses a blank or padded word.
6. **The saved job**: `Job.shapes`, a top-level `"shapes"` list written only when there is one (SPEC section 3 and 5.12). A job saved before has none and
   reads and saves byte for byte as before (`tests/shapes_job.rs`, and the sample house in `demos/shapes.sh`). Shapes are re-checked on every load (so
   `all_or_nothing`'s put-back through the file keeps them).
7. **The verb**: `plumbline items <file> "<item>"` prints that item's shape's blocks at its sizes (the plain `items <file>` is unchanged; no new verb).
   The 36 in vanity reads: block 1 timber across -18 to 18 in, back 0 to 18.638, up 0 to 3.937; block 2 timber back 0 to 20.213, up 3.937 to 32.925;
   block 3 worktop back 0 to 21, up 32.925 to 34.5: the lead's figures to the thousandth.
8. **What the window calls**: `forms::blocks(form: Form, size: P3) -> Vec<Block>` as before (same signature, `Form`, `Made`, `Block`, `drawn_as`, `form_of`
   unchanged); it works the eleven from `shape::starter_blocks` (rows, cached once) and the bed and sink from the code held. `crates/app` was not
   edited and not built (not allowed): no item of `forms` the app names changed. **Not right**: an item drawn as a shape of the job's own is drawn as its box
   in the window until the window's half (call 198 point 6): `building.rs` makes `PlacedView.form` from `forms::drawn_as`, which knows the thirteen only;
   a job shape that has a starter name (a person's own "base cabinet") is drawn by the window as the starter's.
9. PORTMAP.md: 15 rows and the "Call 198" paragraph; `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh`: PASS (351 public items, 142 ported, 209 ours).
   (`Shape::blocks` was named `worked` so no ported name is reused; NOTICE needs no line.) `Made::named/ALL/word` added.

## Base cabinet's three rows as they read in `library/shapes.csv` (a leading apostrophe on `-width / 2` is the spreadsheet guard the table reader strips)
| Shape | Made of | Form | Across from | Across to | Back from | Back to | Up from | Up to |
|---|---|---|---|---|---|---|---|---|
| base cabinet | timber | box | -width / 2 | width / 2 | 0 | depth - min(60 mm, 0.14 * depth) | 0 | min(100 mm, 0.14 * height) |
| base cabinet | timber | box | -width / 2 | width / 2 | 0 | depth - min(20 mm, 0.05 * depth) | min(100 mm, 0.14 * height) | height - min(40 mm, 0.06 * height) |
| base cabinet | worktop | box | -width / 2 | width / 2 | 0 | depth | height - min(40 mm, 0.06 * height) | height |

Hand figures, 36 x 21 x 34.5 in (876.3 mm high, 533.4 deep): plinth min(100, 122.7)=100 mm = 3.937 in; toe min(60, 74.7)=60 mm = 2.362 in so back 21-2.362 = 18.638;
lip min(20, 26.7)=20 mm = 0.787 in so 20.213; top min(40, 52.6)=40 mm = 1.575 in so up to 34.5-1.575 = 32.925. The program gives exactly those (item 7 above).

## What was run
- `cargo test -p plumbline-world` (lib + 36 integration binaries, 37 in all): 555 passed, 0 failed, before the `blocks`->`worked` rename; after it,
  lib + `shapes` + `shapes_job` + `shapes_formula` again: 81 + 2 + 7 + 4 passed. That whole suite is every test that reads a formula (cost lines through
  `lines::read`, `view`, `vif`, `check`, `trim`): none failed and **no expectation was edited**.
- `plumbline` 22, `plumbline-ifc` 43, `plumbline-sheets` 6, `plumbline-plans` 172, `plumbline-roofing` 122 passed, 0 failed. (`plumbline-app` not built, as told.)
- `demos/shapes.sh` passes (all six points; with cargo, its step 2 runs the holding test: `held: 11 shapes, 33 shape-and-size pairs, 111 blocks compared`).
- Every command-line demonstration: **52** (every demos/*.sh without plumbline-app, open_window, xvfb or xdotool, except window.sh and field-job.sh),
  run with PLUMBLINE_BIN: 52 exit 0; `grep -E "FAILED|FAIL"` over all their output: nothing. Output in SCRATCH/w198/demos/.
- Gates: gate-language, gate-egress, gate-clientdata --all: PASS; tools/licenses.sh: exit 0; portmap: PASS.

## Held for a ruling: the bed and the sink (2 of 13)
Each has a block that exists only when the thing is wide enough: the bed's pillows (one across the head up to 1200 mm wide, two above) and the sink's wall
between two bowls (762 mm, 30 in, and wider). Rows say figures, not "there only when". BIMCraft's `visible` is a formula with comparisons
(`Width > 1200`, `family.rs`, the bed's `pillow`). The code for these two stays in `forms.rs` (`blocks_by_code`, private) and `starter_blocks` returns None for them.
The ruling wanted: a column "There when" on a row, holding a formula that is there when it is more than nothing, with `>`, `>=` (and `if`?) in the language;
or another way to say a block that is only sometimes there. Until then `export shapes` writes no rows for a bed or a sink (the item's "Drawn as" says them).

## Not right / said
- A plain `0` is accepted as a length (see above). A block outside the item's box is not refused (the design says nothing; clash tests the box).
- A shape named like one of the thirteen, in a job, overrides it for `plumbline items` and the tables but not in the window (see 8).
- Tests for the shapes were written after the code for most of them (only the formula tests were run red first, and the starter table / holding test
  failed red once on the plain `0` before passing); the holding gate was proved red by changing one figure.
- Disk was under 3 GB at the start (1.6 GB free): I deleted my own old test programs in wtB/plumbline/target/debug/deps by literal path (about 45 files) and it came back to 8 to 12 GB.

## Second round (the lead's rulings 1 to 3)
- **Ruling 1, "There when".** `Piece.when` (saved as `when`, table column "There when", empty = always there). The comparison is read in
  `shape.rs` (`read_cond`, `holds`) and nowhere else; `formula.rs` gained nothing, and `net_area > 100 SF` (also `>=`, `<`, `=`) is refused as before
  ("is not something a formula can hold"; `tests/shapes_formula.rs`). Sides must be both lengths or both plain numbers (a literal `0` stands against
  a length); anything else is refused with its row named. PORTMAP.md records it as BIMCraft's `visible`, narrowed.
  Bed (9 rows: base, mattress, headboard, pillows `width > 1200 mm` x2, `width <= 1200 mm`) and sink (6 rows, divider `width >= 30 in`) are rows.
  **Holding test with all thirteen against the code, before it went**: `held against the code: 13 shapes, 39 shape-and-size pairs, 141 blocks compared`,
  3 passed; bed blocks 4/4/5 and sink 5/6/6 at the small / typical (36 in) / large sizes (below and above each condition). Proved red once on the sink
  (divider `>= 30 in` changed to `>= 40 in`): "Sink at [3.0, 1.75, 2.875]: the number of blocks left: 5 right: 6"; put back. Then the code went:
  `forms::blocks(Form, size)` works all thirteen from the rows (`blocks_by_code` is gone). After, against the golden file regenerated from the
  code first: `held: 13 shapes, 39 shape-and-size pairs, 141 blocks compared`, passed. `export shapes` writes all thirteen (tested for the vanity and the closet).
- **Ruling 2.** `0` alone stands without a unit; a bare `4` where a length is wanted is refused with its row ("comes out as a plain number ... only 0 stands without one"); tested.
- **Ruling 3.** Every shape is worked at small, typical and large (0.4x0.3x0.5, 3x1.75x2.875, 6.5x3x2.6 ft) when brought in or loaded; a block that is there and
  reaches outside by more than 1e-8 ft is refused with its row and the size ("small size ... reaches ... ft outside the box ... across"). Tested; a block outside
  only where its condition fails is not tested there. **All thirteen starter shapes pass it; none excused.**
- **Run**: world 560 passed, 0 failed; plumbline 22, ifc 43, sheets 6, plans 172, roofing 122, 0 failed. 52 command-line demonstrations, all exit 0
  (one, shapes.sh, first stopped on its own old "11" and was corrected, then passed with and without PLUMBLINE_BIN); no FAIL or FAILED after that. Gates, licenses (exit 0),
  portmap (PASS, 353 items) pass. No existing expectation moved.
- **Not right (the window's half)**: an item drawn as a job's own shape draws as a box in the window; a job's shape named like a starter one does not take its place there.
