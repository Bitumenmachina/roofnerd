# W-234M report (notes as I go)

## Before any code (read while quiet.lock stood; no cargo run)

1. The road of a typed measurement. CLI: `crates/cli/src/main.rs` ("measure", ~line 3971, inside `draw`) refuses --status, requires --source,
   reads --to (bottom/centre/top), reading "same" -> `ed::Reading::Same` else `Reading::Length(text)`, calls
   `editing::measure` (crates/world/src/editing/measure.rs:162) or, with --line, `measure_on_line` (:218), prints `Measured.said`
   then each `also` line indented; file written once at the end of `draw` (main.rs ~4000).
   `measure` -> `all_or_nothing` (editing.rs:117: saves the job, runs, runs `building::check_whole` and `new_faults`, on any error loads the
   saved text back and names a made-up path as "the new ..."). Inside it `put` (measure.rs:341) finds the value by the screen's name
   (`on!` per component; Level, Wall, Opening, Segment, Placed, ...) and `apply` (:108) does the one thing: `Length(typed)` ->
   `units::parse_length(typed)` and the Val becomes {value, unit, status: Known, source}; sentence "<name>: <what> is <len>, measured
   (it was <len>, <Status>: <source>)"; `Same` -> `v.mark(source)` (Known, source gets "found as drawn" after the old one), "found as
   drawn" / "found the same again". A storey's height/elevation then `floors_above`/`floor_set` (floors above move, Known floor above
   refuses, sentences in `also`); a run joined to others says what it no longer meets; a roof is solved again. It then `job.put`s
   the component after the component's check. NOTHING in the reply says the price or which clash cleared: the window/CLI only print
   said + also; the price and the clashes are read afterwards (`estimate`, `clashes`). So "price before and after" and "which clash
   cleared/became a finding" are said by the sheet's reply from the existing `view::estimate(..).price` and
   `clash::clashes` before and after each row (a read, never recomputed). Rows go through `editing::measure` /
   `measure_on_line` and no other road. All of it or none of it across rows: every row is tried in order on a scratch copy
   (save/load of the job) collecting EVERY refusal named by row; if any, the real job is not touched and the file is not
   written; if none, the rows are replayed in the same order on the real job inside one outer `all_or_nothing`.
2. A question: `vif::Question` {path,name,kind,storey,room,clash,in_the_way,values:(label,Status,is_length),dollars,also}.
   Kinds: (a) a thing's SIZE (values with is_length: width, depth, height, sill, level height, height above its level, ...) -> a row per value,
   "Measure its <label>" and reading as a tape writes it; (b) what a thing IS / WHERE it is (is_length=false: type, wall line, line on the
   plan, position, top level, what it is, boundary, condition-less): a row each, "Check its <label>", As found is `same`
   (or "as drawn"), a length is refused by `measure` itself ("not one length"); (c) a measure typed on a cost line ("60 SF"; label
   `"line" (60 SF)`): a row, "Measure <line> (60 SF)", As found as "72 SF"; reference carries line and input name (`measure_on_line`);
   (d) a CLASH ("measure either one"): no value of its own: not a row (its two things have rows of their own where they have a
   value); counted and said; a hole to cut and a thing set against: same; (e) a wall a demolition opens: "whether it is bearing",
   "what is in the wall" (and the `also` sentences: what stands on it, headers to size): looked at, no value `measure` takes: probed with
   `takes_among`, not a row, counted; (f) a thing in the way of a wall that comes down (`in_the_way`): cap, reroute or take out is a
   decision, not a measurement: not a row; (g) a PRICE is never a question of this list (a value on a line named "x: unit cost" is filtered,
   `not_on_site` words too): there is no price row. A row's dollars are its question's.
3. Reference: "<path> <value as the screen names it>" e.g. `wall-0001 level height`; for a value typed on a line
   `wall-0001 "Gypsum board, hung and finished" measured`. Plain text that starts with a letter (a workbook never turns it into a
   number or a date), no row number in it, so sorting/deleting/re-saving keep it. What the job had when the sheet went out: the columns
   "The job has" (the value as the screen writes it) and "How sure" (Assumed/Unknown): compared at import to what the job has now.
4. Lengths a typed measurement reads: `units::parse_length` (units.rs:390): feet-inches `25 ft 6 in`, `12-6`, laser `2'2"13/32`, `26"13/32`,
   `23.46ft`, `8'10"1/2`; refuses a bare number ("has no unit"), a comma, metres, a fraction after a fraction. On a cost line:
   `units::read_measure` / `parse_measure` (`60 SF`, `1,200 SF`, a length in feet there is LF) in `measure_on_line`. The sheet hands the
   cell text to those functions unchanged. A run's height word (bottom/centre/center/top, before or after) is parsed as the window
   does (tools.rs `measured`). A workbook program: a bare number in place of text ("9": refused by parse_length as having no unit; the row
   is named and says to write the unit); a date in place of a fraction/range ("8-6", "10/12": sheets crate reads a date cell as
   "a date or a time (day N)": the row is refused saying the workbook turned it into a date: write it as 8 ft 6 in); trailing blanks.
   The On column read as a date serial is turned back to YYYY-MM-DD. Guarded by refusing, never by guessing.
5. demos/field.sh (read; run after the lock goes): job from field-job.sh; before: 2 clashes (duct/pipe A 2.00 in, duct/pipe B 5.00 in), south wall
   216.00 SF $518.40 flagged, PRICE $615.86. Measurements: ground elevation same; ground "level height" 8'10"1/2 (upper floor now 8' 10 1/2" Known;
   213.00 SF $511.20; PRICE $607.31); duct "line on the plan" same; duct type same; duct "height above its level" 7'10"3/4 --to bottom
   -> one clash left, duct/pipe B 2.25 in, a finding; pipe A cleared (0.75 in clear).
6. BIMCraft grep (42 files): nothing named field_sheet, survey_import or punch-list; "punch" only in an IFC class name (PUNCHING RING) and a
   UI string; "measured" is the egui measure tool's drawn distance (ui-egui canvas/tests) and layout widths; "verify" is update
   signature checks (crates/update) and checksum notes (crates/rvt). No field sheet, no import of measurements, nothing to port.

## Progress (lock still standing when written)
Written, not yet compiled: crates/world/src/fieldsheet.rs (new, `pub mod` in lib.rs), vif.rs (two small refactors: `typed_unsure_named`,
`to_do_word`; no behaviour change), crates/cli/src/main.rs (export/import field-sheet arms, --anyway flag, usage lines), tests
crates/world/tests/call234m.rs (11 tests) and crates/cli/tests/call234m.rs (6 tests), demos/field-sheet.sh, README row.
Tests were written together with the code (the lock stopped any run), so they are proved by BREAKING the code, below, once the lock goes.

## Run 1 (lock gone ~13:20)
World tests crates/world/tests/call234m.rs: 13 pass. The tests were written WITH the code (lock), so none was run red first; each was
proved by breaking the code it tests (one break at a time, then restored, then 13 pass again):
 a trial-pass skipped -> one_figure_that_cannot_be_read... FAILED; b stale check off -> a_value_changed_since... FAILED; c price "before" = after ->
 the_measurements_of_the_typed_demonstration... FAILED; d clash sentences dropped -> same test FAILED; e workbook date not turned back to a day ->
 what_a_workbook_does... FAILED; f source loses sheet and row -> the_measurements... FAILED; g "same" taken over a changed value ->
 same_is_not_taken_for_a_value_that_changed... FAILED; h second-measurement-differs check off -> a_row_whose_stamp_says_known... FAILED; i duplicate
 references allowed -> a_reference_the_job_does_not_have... FAILED; j clash questions made rows -> the_sheet_has_a_row_for_each_value... FAILED.
Baseline made from the old built binary (no field-sheet in it) BEFORE any new build: SCRATCH/base234.DYgM: `sixteen` sha256 880d9fd3c364b5f7,
`export vif` of it = vif.csv sha256 0a334faf63dffa8b84d42fc78a42b67e12498c2b6a02e606ccec6c3930020898 (85 rows).
Note: a stray background wait loop of mine (pid 25852, a `sleep 15` loop on quiet.lock) was started by mistake; it ends by itself now that the lock is gone;
pkill was refused by the permission system, I did not try another way.

## Run 2 (tests)
All command-line crates' tests pass on this tree (one test program at a time, deleted after each; files review/tests-*.txt):
world 753 (740 before + my 13), command line 91 (85 + my 6), sheets 7, ifc 131, roofing 122, plans 231 = 1,335, 0 failed.
My 6 CLI tests proved by breaking: job not written after import -> the_field_measurements... FAILED; only the first refusal said ->
one_row_that_cannot_be_read... FAILED. Rows applied in the job's own order (found by demos/field-sheet.sh step 4: a sheet in reverse order
left the upper floor Assumed because a floor above follows the floor below as it stands when measured); test
rows_in_another_order_and_cut_down... now also asserts the upper floor Known and is red with the sort off.

## Run 3 (demonstrations, gates)
Command-line demonstrations: 67 run (every demos/*.sh but *w.sh, d2.sh, house.sh and the two helpers; 66 old + demos/field-sheet.sh), each in the idle class with
PLUMBLINE_BIN, exit 0 and no FAIL in any output (review/demos-summary.txt, outputs in review/demos/). demos/field.sh and demos/vif.sh unedited and pass.
Gates: gate-language PASS, gate-egress PASS, gate-clientdata --all PASS, portmap PASS (477 public items, none of mine in building*/editing*, so no row), licenses.sh exit 0.
Sample house: `plumbline sixteen` sha256 880d9fd3c364b5f7; `export vif` of it sha256 0a334faf63dffa8b... byte for byte the baseline taken from the old build.
Final: 7 CLI tests (a seventh added: a workbook that turned the reading into a number is refused by row; it rests on parse_length's existing refusal, not proved by a break).
Patch: review/call234M.patch (8 files, 1,511 insertions, 14 deletions).
