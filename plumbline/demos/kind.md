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
| `demos/marks/marked-plan.pdf` | 7 marks: 2 lengths, 1 run of lengths, 2 areas, 1 count, and 1 note that measures nothing: 2 + 1 + 2 + 1 = 6 | a plan, with 6 measured marks, or a maker's sheet |
| `demos/marks/unreadable.pdf` | it opens; 4 marks: 1 length, and 3 that cannot be read | a plan, with 1 measured mark (3 more not read), or a maker's sheet |
| a PDF cut short, made by the script | does not open | not read, with the reader's own reason |
| `demos/plans/made-up-plan.png`, `.jpg` | a picture that opens | a plan (a picture) |
| `demos/marks/two-pages.tif` | two pages | a plan (a picture of 2 pages) |
| `demos/dxf/plan-inches.dxf` | a CAD drawing | a plan (a CAD drawing) |
| a PLY, a LAS, a LAZ and an E57 made by `demos/scan-files/make.py` | by ending | a scan (each) |
| `demos/ifc/made-up-house.ifc` | a level with walls on it | a designer's model, or a maker's model |
| `demos/maker-model/nl-2018.ifc` | no level | a maker's model |
| `demos/tables/tinners-yard.csv` | headings Item, UOM, Price, Lead time (third row): the price list's reader finds a material and a price; the prior estimate's finds a description; no quantity, no amount | a price list, or a prior estimate |
| `demos/tables/tinners-yard.xlsx` | the same as a workbook | a price list, or a prior estimate |
| `demos/tables/prior-estimate.csv` | Description, Quantity, Unit, Unit cost, Amount: both readers find their headings; it has a quantity and an amount | a prior estimate, or a price list |
| `library/sixteen-divisions.csv` | Assembly, Goes on, ... | assemblies |
| `library/items.csv` | Item, Trade, Kind, Drawn as, Width, Depth, Height, ... | items |
| `library/shapes.csv` | Shape, Made of, Form, Across from, ... | shapes |
| `library/connections.csv` | Item, Connection, Takes, ... | connections |
| `library/cabinet-rules.csv` | Code, What it is, Number reads, ... | cabinet rules |
| the workbook `plumbline export database` writes for the sample house | its tabs are the tables | my database |
| the sheet `plumbline export field-sheet` writes for the sample house, as `.xlsx` and as `.csv` | headings Reference and As found | a field sheet |
| the sample house's own saved job | a job of this program | a job |
| `demos/kind.md` (this file) | no kind | not a kind BuildOS reads |

The rule is call 242 point 1 as call 247 put it right: a particular table whose
reader finds every heading it needs is said alone; a price list and a prior estimate
are said only where no particular table is found; my database is a workbook with two
or more tabs named as the program's tables (`tinners-yard.xlsx` has one, Prices, and
a tab of terms: it is said by its table); a note is not a measured mark, and marks
not read are said.

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
4. **Saying otherwise** (call 247, point 2). For `tinners-yard.xlsx` the kinds whose
   readers would take it are a price list, a prior estimate and my database, in that
   order; for `library/sixteen-divisions.csv`, assemblies first and then whatever
   else the readers answer, printed by the script and not judged. `plumbline kind`
   prints only what the file is said to be.
5. **A picture's ending on something that is not a picture** (a CSV named `.png`):
   not read, with the reader's reason.
