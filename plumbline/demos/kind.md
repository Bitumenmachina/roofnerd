# What a file is: a demonstration, written before it is built

Call 242, point 1. `plumbline kind <file> [<file> ...]` says what each file is, one
line to a file, by the rule in call 242. The window's Bring in list shows the same
words from the same function (`crates/files`). The script is `demos/kind.sh`. Every
file is made up and is already in `demos/` or `library/`, or is made by a script
that is.

## The files and what must be said, worked by hand from the rule

| File | Why | Said |
|---|---|---|
| `demos/plans/made-up-plan.pdf` | a PDF that opens, no measured marks | a plan, or a maker's sheet |
| `demos/marks/marked-plan.pdf` | a PDF with measured marks: the count is the count `plumbline marks` gives for it | a plan, with N measured marks, or a maker's sheet |
| `demos/marks/unreadable.pdf` | does not open | not read, with the reader's own reason |
| `demos/plans/made-up-plan.png`, `.jpg` | a picture | a plan (a picture) |
| `demos/marks/two-pages.tif` | two pages | a plan (a picture of 2 pages) |
| `demos/dxf/plan-inches.dxf` | a CAD drawing | a plan (a CAD drawing) |
| a PLY, a LAS, a LAZ and an E57 made by `demos/scan-files/make.py` | by ending | a scan (each) |
| `demos/ifc/made-up-house.ifc` | a level with walls on it | a designer's model, or a maker's model |
| `demos/maker-model/nl-2018.ifc` | no level | a maker's model |
| `demos/tables/tinners-yard.csv` | headings Item, UOM, Price, Lead time (third row): the price list's reader finds a material and a price; the prior estimate's finds a description; no quantity, no amount | a price list, or a prior estimate |
| `demos/tables/tinners-yard.xlsx` | the same as a workbook | a price list, or a prior estimate |
| `demos/tables/prior-estimate.csv` | Description, Quantity, Unit, Unit cost, Amount: both readers find their headings; it has a quantity and an amount | a prior estimate, or a price list |
| `library/sixteen-divisions.csv` | Assembly, Goes on, ... | assemblies (first) |
| `library/items.csv` | Item, Trade, Kind, Drawn as, Width, Depth, Height, ... | items (first) |
| `library/shapes.csv` | Shape, Made of, Form, Across from, ... | shapes (first) |
| `library/connections.csv` | Item, Connection, Takes, ... | connections (first) |
| `library/cabinet-rules.csv` | Code, What it is, Number reads, ... | cabinet rules (first) |
| the workbook `plumbline export database` writes for the sample house | its tabs are the tables | my database |
| the sheet `plumbline export field-sheet` writes for the sample house, as `.xlsx` and as `.csv` | headings Reference and As found | a field sheet |
| the sample house's own saved job | a job of this program | a job |
| `demos/kind.md` (this file) | no kind | not a kind BuildOS reads |

"(first)" marks five rows where the first thing said is fixed here and what follows
"or" is not worked by hand: the builder runs each table's reader over the headings,
writes the whole line in its report, and the lead rules on it before the script
expects it.

## What else is checked

1. **One function.** For every file above that the command line brings in today,
   the job saved by `plumbline import ...` (or `drawing`) is byte for byte what it
   was before `crates/files` (the demonstrations `imports`, `dxf`, `scan-files`,
   `marks`, `maker-model`, `database`, `field-sheet`, `export`, `bcf`, `gltf` pass
   unchanged and none is edited).
2. **Nothing is opened that is not read.** `kind` changes no file and no job (the
   files' checksums before and after).
3. **A file with the wrong ending** (a CSV named `.pdf`): not read, with the
   reader's reason; never guessed from its inside.
