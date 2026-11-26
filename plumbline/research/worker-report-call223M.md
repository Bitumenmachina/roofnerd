# W-223M report (notes as I go)

## Before any code

### 1. Every table, today (run in a scratch dir, starter + a made-up 3-supplier price list + 1 shape + 1 item of its own)
| Table | Verbs | Headings (count) | Writes | Reads | Depends on |
|---|---|---|---|---|---|
| Shapes | export/import shapes | Shape, Made of, Form, Across from/to, Back from/to, Up from/to, There when (10) | `tables::shapes_out` (`shapes_out_counted`) | `tables::shapes_in` | nothing; the 13 starter shapes are in every job (reads "13 the same" into an empty job) |
| Items | export/import items (+ `starter`) | 26 (`ITEM_HEADINGS`; 3 columns per sized value: value, how sure, where from) | `items_out` | `items_in` (`items_in_missing` for starter) | the shape an item is "Drawn as" must exist (own shapes first; starter shapes always there) |
| Connections | export/import connections (+ `starter`) | 14 | `connections_out` | `connections_in` (`connections_in_missing` for starter) | the item the row names must be in the job |
| Assemblies | export/import assemblies (+ `starter`) | 27 | `assemblies_out` | `assemblies_in` | nothing in the tables above |
| Prices | export/import prices | 10 (`PRICE_HEADINGS`) | `prices_out` | `prices_in` | names a material or an item. Into a job that lacks the material, the single verb refuses every row ("the job has no material called ...") unless `--add`; with `--add` the material is made. A whole database must bring materials back, so the database reads prices with add on |

Out, in to an empty job, and out again, byte for byte, run today for each table, in the order shapes, items, connections, assemblies, prices (prices with --add): **CSV: all five the same, byte for byte. XLSX (one table per workbook): items, shapes, connections the same; assemblies and prices differ by bytes, and only in `docProps/core.xml`, the creation time stamp rust_xlsxwriter writes (checked: same job written twice, 2 s apart, differs the same way).** So bytes are not the test for a workbook; cell for cell is. No table "cannot be written out or brought back whole": no STOP. Not a table (no cabinet rules table exists in the code: `grep -ri "cabinet rule"` finds nothing in crates/*/src), so no tab for it. Prior estimates (`import prior`) are not in call 223's list and are not made a tab.

### 2. Order, and what "all or none" rests on
Shapes, Items, Connections, Assemblies, Prices. Items name shapes; connections name items; prices name materials (made by the price tab with add on) and items. Assemblies stand alone and come before prices as the call says. The tabs are brought in in that order wherever they lie in the file. All-or-none rests on `editing::all_or_nothing`, the model's own: it saves the job, runs the whole bringing-in, then checks the model whole and for new faults, and on any refusal loads the saved text back. The single readers refuse row by row (a row "not used" while the others go in); a database counts any row not used as a refusal, so the whole is wrapped and returns Err after all tabs have been tried (so every refusal is named, not only the first).

### 3. Workbooks today
`plumbline_sheets::xlsx(&[Tab])` already writes several tabs (name, rows of Cell, head row, widths); `read(bytes, Some(name))` reads one named sheet and adds a "N other sheets were not read" note to `lost`. Writing the same job twice does not give the same bytes (only the core.xml time stamp), so xlsx is compared cell for cell. Lacking: a way to list a workbook's tabs and read them all. I add `plumbline_sheets::read_tabs(bytes)` (reads every visible or hidden tab by its name, dropping the "other sheets" note which would be false) in `crates/sheets/src/lib.rs`. Nothing existing changes.

### 4. Words
Single verbs say, per table: `{file} ({from}): N rows under its headings`, `  set: N` + each `    added...`, `  the same as the job already has: N`, `  not used: N` + `    row R: why`, `  NOTE: ...`, `  not read: the column "X"`, `  not read: <lost>`. The database says the same lines under a heading for each tab ("the tab Items: 49 rows under its headings"), then `not read: the tab "Notes"`, `no tab for Prices (the job's own are left as they are)`, and a total. A refusal says `nothing was brought in ...` then `  tab Items, row 6: <the table's own reason>` per refusal. Words: tab, row, table, "my database"; never schema, record, field.

### 5. The seven steps' counts, as the program holds them today (22 Nov 2026 tree, 7e3d3d6)
Starter: **48 items, 13 shapes (49 rows, one row a block), 5 rows of where items connect (2 items: water closet 2, lavatory 3), 42 assemblies in 202 lines**. With the made-up list of three suppliers (2 materials x 3 = 6 rows + the 2 materials' own rows = **8 price rows**), one shape of its own and one item of its own: **49 items, 14 shapes in 50 rows, 5 connection rows, 42/202, 8 price rows**. (The lead's step-2 numbers of 5 connection rows and 42/202 are right; items and shapes are 48/13 before the job's own, 49/14 after.)

## Log
- Stub `crates/world/src/database.rs` + `crates/world/tests/call223m.rs` (14 tests) written first. Red run: `test result: FAILED. 0 passed; 14 failed` (every one "not built").
- red proofs: world 14 failed 0 passed (stub); sheets lib test did not compile (read_tabs missing); CLI step 6 FAILED with the refusal check removed (then restored)

## Done (see the hand-back message for the full account)
Patch: review/call223M.patch (9 files). Tests: world 700, plumbline (cli) 64, plans 187, sheets 7, ifc 111, roofing 122, none failed. 59 older command-line demos + database.sh pass with no FAIL (d2, house are window demos; field-job is a helper, not a demo). Gates, portmap (454 items), licenses pass. sixteen sha 880d9fd3c364b5f7.
