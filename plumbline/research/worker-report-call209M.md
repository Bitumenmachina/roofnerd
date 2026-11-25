# W-209M report (contract 209M), written as I go. Second working copy: wtB, base 7fffde6. Nothing committed.

## Findings on my own build (command line), before any change
- Point 2: job with starter items, a "Vanity, 36 in" set down three times, `plumbline rename J "Vanity, 36 in" "Vanity, 42 in"` said
  "the 3 set down from it are called that too" and `takeoff` read `3 furnishings: Vanity, 42 in, Vanity, 36 in 2, Vanity, 36 in 3`.
  Cause: `editing::rename_item` renamed a thing only when its name was exactly the item's old name.
- Point 8: `plumbline show` of an empty job with `import items starter` read `144 Known, 144 Assumed, 96 Unknown` (only the chest).
  The count is the model's (`view::tally`, crates/world/src/view.rs); the window calls it at crates/app/src/model.rs:845.
- Point 5: `parse_length("12-6")` was refused: "...write it like 25.5 ft, 306 in, or 25' 6 1/2\", with no commas" (there is no comma).

## POINT 5, THE LIST (made before any change)
What `units::parse_length` read BEFORE (probe run on the unchanged build; saved in w209/probe-before.txt) and what it reads AFTER:
| typed | before | after |
|---|---|---|
| 12-6 | refused ("...with no commas") | 12 ft 6 in (FtIn [12,6]) |
| 12'-6" | 12 ft 6 in | same |
| 12 6 | refused | refused (still; says to write 12-6, 12' 6" or 12 ft 6 in) |
| 12-6 1/2 | refused | 12 ft 6 1/2 in |
| 6-0 | refused | 6 ft 0 in |
| -6 | refused: "has no unit" | refused: "has no unit" (still below nothing, still no unit) |
| 12-6", 12-6 in | refused | 12 ft 6 in |
| 12-6-1/2 | refused | 12 ft 6 1/2 in |
| -12-6 | refused | minus 12 ft 6 in |
| 12-12, 12-13 | refused | refused: inches less than 12 |
| 12.5-6 | refused | refused: whole feet |
| 12-6 ft, 12 - 6, 12-, 12--6 | refused | refused (a range or a subtraction, not read) |
| 6-1/2 | refused, no unit | same; 6-1/2" = 6.5 in as before |
| 12 ft-6, 25' -6" etc. | as before | as before |
Every place a length is read goes through `parse_length`: world (job.rs 292 and 1156, editing.rs 64 and 1761, editing/join.rs 207,
editing/stretch.rs 184, editing/measure.rs 113 and 260, finish.rs 603, plans.rs 436, tables.rs 788, 1029 and 1048), ifc/lib.rs 1768, 1808,
1870, cli/main.rs 1411, 2036, cli/found.rs 467, and in the window (not touched) tools.rs 242, 256, 514, 1131, 1596, 1684, 1703, main.rs 912,
bench.rs 244. The roofing crate, crates/plans (the drawn dimension reader) and the maker's-sheet reader (spec.rs, own scan) do not call it.
Search of every test (crates/*/tests, src tests), every demonstration (demos/*), library/*.csv, tools and README for a digit, a hyphen and a digit:
 * a length typed with a hyphen BETWEEN BARE NUMBERS ("12-6" style) in any test or demonstration: NONE. Number of figures that move: 0.
 * with a hyphen and a fraction (read by `quantity`, which is untouched): crates/world/tests/world.rs:375 `6-1/2"` (6.5 in) and :379
   `25' 6-1/2"` (306.5 in): unchanged. The maker's sheet line `34-1/2" H` (crates/plans/tests/plans.rs:227, world/src/spec.rs:516/523,
   demos/spec.sh:59/69/71, demos/carryw.sh:33, demos/itemsw.sh:91, README.md:309): spec.rs has its own scanner; unchanged.
 * with the hyphen after a mark (read before and after alike): world.rs 360-361, 378, 1039-1047, `25'-6"` family; cli/tests/call168.rs, call182.rs
   and demos/planset.sh print "24'-0\"" (output), they do not type it.
 * not lengths: "12-2" cable names (demos/div.sh:54, lists.sh:161, library.sh:55, sample.rs:501), project number "24-017" (world.rs:644),
   "ridge 1-3", "hip 1-2" names, NRCA "Fig. 4-11", ids like wall-0004, dates.
 * two places a free-text line reaches `parse_length` and could meet a range: finish.rs height_in (a line of text, "Stone wainscot 3-4 high" could
   be a range): GUARDED so it reads as before (nothing); a test line added to the existing test. The window's tools.rs size fields split on x, "by",
   "/" (not on a hyphen); not touched.
 * no place reads "12-6" as a range or a subtraction today in a length field: formulas have their own reader (`12 ft - 6 in` has marks).
Nothing stops on a collision.

## POINT 3, THE LIST (made before any change): phrases a builder does not say, and what each becomes
The judge's: (1) "write it out with `export estimate`", (2) "connection point ties", (3) "31 x type", (4) "432 x cost line",
(5) "this program would not take it", (6) "stands upright: it has no length on the plan to be cut along". Others in the same replies:
(7) "332 connection points", (8) "n x what: why" as a format on both screens (the window builds it in crates/app/src/bring.rs 154-158 and 289),
(9) "n lists of what the model says of each thing, and 0 materials" (the model's property sets), (10) "come in with its size Unknown" (grammar),
(11) "in the very place the model has them", (12) "the new floor lies in floor on the same level", (13) the not-written words "condition" ("assembly" is the word),
"room line", "estimate made before", "drawing", "material" given as raw kind names with "x", (14) "each thing is written with its own size; the type it took it
from is named among its values".
| was | becomes |
|---|---|
| write it out with `export estimate` (model) | "the estimate is not part of a building model: it is written out as a workbook" (model); CLI adds ", with `export estimate`"; window adds "Estimate workbook, under Write out" |
| connection point ties / connection points (counts) | joints / connections |
| 31 x type: ... | "31 wall, door, window and run types: each wall, door, window and run is written with its own size, and its type's name stays among its values" |
| 432 x cost line: ... | "432 cost lines: the estimate is not part of a building model: ..." |
| n x what: why (format) | "n whats: why", one model function gives it (Written::not_written_said, Brought::left_out_said); the window calls it |
| this program would not take it: R | R (the reason stands alone) |
| X stands upright: it has no length on the plan to be cut along | X runs straight up and down: there is no length along the plan to cut it at |
| n lists of what the model says of each thing, and m materials | the notes the model carries on its things (n) [and its materials (m) only when m is more than 0] |
| each is drawn as a line alone, and come in with its size Unknown | each is drawn as a line alone and comes in with its size Unknown |
| in the very place the model has them | in the job already, in the same place |
| the new floor lies in floor on the same level | the new floor and floor cover the same ground on one level |
Left as they are, said: "connection points" inside the note that names the FILE's own term and the value sources ("the file's connection points",
crates/ifc/tests/ports.rs:178); coordinates in the notes about bends (judge F-B2; not asked).
Existing tests and demonstrations whose words change (listed first, as the contract asks): demos/ifc-whole.sh lines 34, 35, 47, 58, 232;
demos/export.sh 165, 186; crates/ifc/tests/write.rs:95; crates/ifc/tests/call206.rs:98; crates/ifc/tests/ifc.rs:257 (and the "in the very place" ones).

## Done (all after the lists above)
Red runs quoted: point 2/5 (call209m world): 5 of 5 failed on the old build (e.g. `left: ["Vanity, 36 in 2", "Vanity, 42 in"] right: ["Vanity, 42 in", "Vanity, 42 in 2"]`;
`"12-12": ... is not a measurement ... with no commas`); point 8: `cannot find function tallies in module view`; point 4 (ifc call209m, fix switched off):
"this model is already in the job: ... (1 wall)" with nothing after, and "the new floor lies in floor-0004 on the same level: ...". Point 3 red = compile (new `not_written_said`, `left_out_said`) plus the findings above.
Fixes: units.rs (12-6), editing.rs (Renamed, rename_item_counted, names_in_scope), view.rs (Tallies, tallies), cli main.rs (rename, show, export ifc, import ifc),
ifc lib.rs/write.rs (words, with_a_way_on, left_out_said, not_written_said), building.rs (floor refusal), join.rs (upright), finish.rs (guard), PORTMAP.md (2 rows).
Existing expectations moved, each for its point's words: ifc tests call206.rs (98: new words + advice; 108: the 600 limit now counts the counts only), ifc.rs:257, write.rs:95;
demos ifc-whole.sh (34, 35, 47, 58, 232), export.sh (165, 186), judged-1125.sh step 9 (700-character limit counts the counts only), roomline.sh:94 ("1 x room line:" -> "1 room line:"),
sweep-money.sh:346 ("of what the model says of each thing" -> "the notes the model carries on its things"). The last two I did not have on my first list (my grep missed "x room"): found by running them.
Tests: world 46 binaries ok (+ unit tests), ifc 9 binaries ok, cli ok, plans ok, sheets ok, roofing ok; new: world/tests/call209m.rs (6), ifc/tests/call209m.rs (4).
Demonstrations: 54 command-line demos run with PLUMBLINE_BIN; 54 ended rc=0 (two reruns after the two words above); no FAIL/FAILED in any output (grep -w).
Gates: three root gates PASS, licenses.sh ok, portmap PASS (383 items). Patch: SCRATCH/review/call209M.patch.
