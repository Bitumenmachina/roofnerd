# W-194 report: the sample whole house prices its outside trim (call 194)

Working copy: SCRATCH/wtB (commit 470292a). Nothing committed or pushed.

## Before any code

### 1. BIMCraft search
`grep -rli "wainscot\|corner_board\|cornerboard\|water.table\|water_table"` over all of /home/user/ipedrax/bimcraft (rs, ts, md, toml; target and .git left out): no file.
`grep -rli "corner.board\|fascia\|soffit"` over the whole tree: only two translation files (`translations/ui/it.po`, `translations/project/it.po`: words in
an Italian translation, no code). BIMCraft has no corner board, no trim that stops at a wainscot or a base course, and no figure for either.
Nothing to port. Recorded in PORTMAP.md as an "ours" row (see below); NOTICE untouched.

### 2. The program as it is, against the lead's hand table (frozen `SCRATCH/f5/plumbline`, `plumbline sixteen`, `estimate`, `trim`, `trim --pieces`)
Every figure of the hand table is what the program says. No difference; nothing to rule on.

| What | Hand table | The program (before any code) |
|---|---|---|
| Wall heights | 10 ft ground, 9 ft above | face area 400 SF on a 40 ft wall = 10 ft; `trim` corner board 20 LF a ground wall (2 x 10), 18 LF an upper wall (2 x 9); foundation walls 8 LF |
| Ground-level windows | 8, casing 20+20+18+14+16+15+14+18 = 135 | window-0021, 22, 24, 26, 28, 30, 31, 32: 20, 20, 18, 14, 16, 15, 14, 18 = 135 |
| Upper-level windows | 9, 19+20+20+18+14+16+15+14+18 = 154 | window-0054 to 62: 19, 20, 20, 18, 14, 16, 15, 14, 18 = 154 |
| Windows in all | 17 | 17 (8 + 9) |
| Eaves | 42, 30, 42, 30 = 144 LF | eave-0077 to 0080: 42, 30, 42, 30 LF |
| Overhang | 0.75 ft | 0.75 ft at each; soffit corners "4 corners, 2.25 SF in all" |
| Front door casing | none by length | "Casing and trim" 1 EA at $120; `trim --pieces` says its quantity "is not its casing length" |
| Recap before | Material $199,666.53 on 205; job cost $390,260.26 on 399; division 6 $51,074.22 on 41; overhead $39,026.03; profit $34,342.90; price $463,629.19 | the same, to the cent and the line |
| Hand additions | 450.00 + 350.00 + 729.00 + 831.60 + 691.20 + 669.60 = $3,721.40 on 33 lines | worked again: the same; new totals Material $203,387.93 / 238, job cost $393,981.66 / 432, division 6 $54,795.62 / 74; overhead 10 % = 39,398.166 -> $39,398.17; profit 8 % of 433,379.83 = 34,670.3864 -> $34,670.39; price $468,050.22 (adds to the cent) |

(`plumbline trim <file> --pieces` takes the flag after the file; `trim --pieces <file>` is refused: "trim takes one file and nothing else".)

### 3. How a figure that takes a height from its line is handled today (`net_area_below_<entry>`), place by place
Named (`formula::BELOW`, `net_area_below_`), and each place:
1. **Naming**: `formula.rs` `BELOW` const; the entry is whatever follows the prefix.
2. **Names a formula lists**: `formula::names` also lists the entry (`wainscot`) after the figure, so "a value is given for ..., which the formula does not use" and "what the new formula no longer names comes off the line" (`job.rs` `set_line_value`) both see it.
3. **Working it**: `building.rs` `measures` (wall branch): per wall, only where a line names it, the figure is worked from the line's own entry; as sure as the entry and the openings; two lines of one wall giving two heights make it refused and said.
4. **The operand of the worked formula**: `lines.rs` `operand`: a `Wall` name starting with BELOW takes the wall's figure (an area, dim 2), refused if the line has no such entry ("uses ..., which takes a height the line enters as ..., and the line has none").
5. **The worked formula shown**: `Operand.shown` is the figure's name; `formula::work` builds the text; nothing special.
6. **Inputs a line rests on**: `lines.rs` `worked`: a BELOW name makes the line rest on the wall's measures (`#measure`) as well as on the line's own entries (`#cost`).
7. **The refusal when the entry is missing or is not a length**: missing: (4), refused when the formula is set (`check.rs`, `lines::read` with no figures) and, for a saved job, at work time; not a length: `building.rs`: `hv.feet()` is None -> refused "the height ... is not known".
8. **Verify-in-field list**: `vif.rs` `typed_unsure` (an entry typed on the line that is not Known is a question; it uses `formula::names`, so the entry is seen) and the `rests` loop (`measure.0.get(&name)`), which find the figure in the wall's measures by name.
9. **The workbook's live formula** (`export.rs`): the quantity is written as a value; only the amount (`=ROUND(D*F,2)`) and the totals are live. Nothing there knows the name; nothing is needed. Not handled, and not to be.
10. **The CSV of assemblies** (`tables.rs` `line_out` / `line_in`): the quantity text is written as it is, and the line's entry as a "name = value" figure with its status and source (the "Figure 1" columns); read back the same way. Nothing name-aware; the starter library's siding line already carries `wainscot = 3 ft` that way.
11. **A place that does not handle it, named**: `lines.rs` `lump_quantity` (a quantity that names a figure of the building is not a lump) refuses any such name by "is a figure of the thing"; correct for both.

The new figure, `corner_board_length_above_<entry>` (`formula::ABOVE`), is handled in 1 to 8 by one shared reader, `formula::height_entry` (so the three places cannot drift), with 3 worked in `building.rs` from the wall's outside corners and corner-board length. 9 and 10 are unchanged and covered by a test (the job file and the table of assemblies round-trip the line). `trim::names` and `trim/lay.rs` `priced_lines` / `other_lines` were exact-name matches on `corner_board_length`; a shared `trim::names_figure` now also takes `corner_board_length_above_<entry>` there, which is what makes it a corner-board line to `trim --pieces`, to `trim::lay` (the function the window calls to draw trim, `model.rs:961`) and to the wall's figures being given at all.

## Built
(see below, filled in as it is done)

(The "Built" heading above was a placeholder; this is the whole of it.)

## Built (files: see `call194.patch`; 13 files, +529 -18)
- `formula.rs`: `ABOVE = "corner_board_length_above_"` and `height_entry(name)`, the one reader of "a figure that takes a height from the line" (`BELOW` and `ABOVE`); `names()` lists the entry through it.
- `building.rs` (wall measures): `corner_board_length_above_<entry>` = for each outside corner, the board's length less the entry (held between nothing and the wall's height), added up; status = weakest of the boards' length and the entry; refused (never nothing, never the whole board) when the entry is missing, is not a length (`3 SF` leaves it "unknown", as `net_area_below_wainscot` does), the wall's corners are not worked, or two lines of one wall give two heights.
- `lines.rs`: `operand` and the `#measure` input handled for both prefixes (the new one is a length, dim 1).
- `trim.rs`: `names_figure`, so a line priced by the new figure "names" `corner_board_length` (the wall is given its corner figures; `trim::lay`, the function the window calls, and `trim --pieces` treat it as a corner-board line; no change to `crates/app`).
- `trim/lay.rs`: `priced_lines`/`other_lines` use it; pieces of such a line start at the entered height (clamped to the wall); a board of no length is not drawn.
- `sample.rs`: corner boards (8 walls; ground ones by the new figure with `wainscot = 3 ft` Known as the siding's), exterior casing (17 windows), fascia and soffit (4 eaves), to the names and prices of call 194.
- `library/sixteen-divisions.csv`: the wainscot wall's Corner boards line only, with `wainscot = 3 ft` Assumed.
- `crates/world/tests/call194.rs` (9 tests), `demos/sixteen-trim.sh`, README row, PORTMAP note (not a row: `ABOVE` is not an item of the building model, a row fails the gate "not a public item of the building model").

## Red runs (tests first)
With the source put back to commit 470292a and only call194.rs added: `9 failed; 0 passed`. Quoted: 7 failed on `a value is given for "wainscot", which the formula "corner_board_length_above_wainscot" does not use`; the starter test `left: (Some(20.0), Assumed) right: (Some(14.0), Assumed)`; the sample test `left: Some(390260.26) right: Some(393981.66)`.
Then with the code: `9 passed`. `demos/sixteen-trim.sh` on the frozen program: `FAIL: Corner boards, 5/4 x 6 cedar ... by hand 8 on it 128.00 800.00 said 0 0.00 0.00` (exit 1); on the new build exit 0.

## What was run (CARGO_INCREMENTAL=0, -j2, nice, one crate at a time, --no-fail-fast)
- plumbline-world: 514 passed, 0 failed (82 incl. call194's 9 in the same run).
- plumbline (cli): first run 12 passed, 1 failed (below); after the one edit, 13 passed (+ 2 + 3 + 3 + 0 in its other targets).
- plumbline-sheets 6, plumbline-ifc 20, plumbline-roofing 129, plumbline-plans 172: all pass, 0 failed.
- Command-line demonstrations (the 49 that need no window, with PLUMBLINE_BIN=<wtB>/plumbline/target/debug): 48 pass, `field-job.sh` is not a demonstration but a helper that takes a job-file argument ("$1: unbound variable" run bare, as it does on the old program; `field.sh` that calls it passes). `sweep-money.sh` failed once (below), passes after the one edit. This includes `trim.sh` (its four sample-house checksums still hold) and `sixteen-trim.sh`.
- Gates: `tools/gate-language.sh`, `gate-egress.sh`, `gate-clientdata.sh --all`: PASS. `tools/licenses.sh`: exit 0. `BIMCRAFT=/home/user/ipedrax/bimcraft tools/portmap.sh`: PASS, 326 public items (142 ported, 184 ours).

## Every expectation that moved (all three edited, each by a figure in the STATUS section)
1. `crates/cli/tests/cli.rs:382` `the_estimate_is_read_by_item_and_comes_to_the_same_job_cost`: "52 items from 399 lines" -> "56 items from 432 lines". 399 + 33 (8 corner-board + 17 casing + 4 fascia + 4 soffit) = 432. Items: four new descriptions, one class and division each: 52 + 4 = 56. Then (same test, lines 391-392) `Job cost $390,260.26` and the by-item foot `$390,260.26` -> `$393,981.66` = 390,260.26 + 3,721.40. Comments on the lines updated.
2. `demos/sweep-money.sh:132`: the job cost had fallen by $4,417.20 (394,677.48 -> 390,260.26, window 4417.15 to 4417.25); it now falls by 394,677.48 - 393,981.66 = $695.82 = 4,417.20 - 3,721.40 (window 695.75 to 695.85). Printed text and comment updated.
Nothing else moved: no existing test file in world, sheets, ifc, roofing or plans, and no other command-line demonstration; `crates/world/tests/divisions.rs`, `call177.rs`, `finish.rs`, `vif.rs` (which open `house_priced`) hold no total that moved.

## Window demonstrations (NOT RUN: cannot here; nothing edited)
`grep -ln "sixteen\|house_priced" demos/*.sh` gives dressw.sh, layoutsw.sh, lightw.sh, tradesw.sh (and command-line ones above). None holds a price, line count or trim count of the sample as an expectation: tradesw.sh compares the price before and after (relative), layoutsw.sh's job-cost figures are its own job. What can move, by picture and not by figure: dressw.sh and tradesw.sh measure the colour of fixed screen rectangles on the sample house (`mean("priced/01", (700, 460, 760, 520))`, `WALL, ROOF, SHADOW, FRAME` regions) and the sample now draws casing, corner boards, fascia and soffit, so a region near an opening, a corner or an eave may read differently; lightw.sh's pictures likewise. These are left unedited for the lead/window builder to run. The crates/app unit tests were not run (not a command-line crate).

## Not right / for the lead
- `plumbline apply <wall> <other assembly> --change` between the wainscot wall and the plain wall keeps the wall's existing "Corner boards" line (same description, "lines the two share keep their prices"), so a wall changed from the wainscot assembly keeps the above-the-stone figure. Existing behaviour, not touched.
- The `plumbline trim` figure is unchanged (the new figure is not listed by `trim`, which says only the plain figure; point 4 held).
- Two things I judged "follows by arithmetic" and edited under the rule: the 4-item count in cli.rs and the sweep-money window. If the lead reads either as a different movement, revert those two hunks of the patch.
- No stop-for-ruling case arose: the hand table and the program agreed on every line.
