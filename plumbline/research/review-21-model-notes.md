# Review 21 notes: the model and the money

Reviewer started 2026-11-16. Findings as they come.

## R21-1 (reproduced) A quantity is rounded to four places before it is priced: a cent wrong

value.rs:151 (`Derived::new` rounds every figure with `round4`), used by lines.rs:361
(quantity), :374 (with waste, from the rounded quantity), :393 (order, from the rounded
with-waste), then lines.rs:240 extends the rounded figure. Rounding is done at up to four
layers (quantity, waste, order, cents). The money is the product of a figure that has
already lost its tail; with a large unit price the lost tail is cents.

Repro (j1.json in RDIR):
    plumbline new j1.json "Made-up test"; level First 0 ft / 9 ft; type wall "Stud 6" 6 in
    wall j1.json level-0002 type-0003 --points "0,0 100,0" --height "9 ft"
    line j1.json wall-0004 "Trim run" "plan_length / 3" --cost-type material --division 6
    set j1.json wall-0004 unit_cost 300 --line "Trim run" --status Known --source made-up
    estimate j1.json  ->  Trim run 33.3333 LF $300.00/LF $9,999.99
By hand: 100 / 3 x 300 = 10,000.00. The program says 9,999.99. The xlsx (G5
=ROUND(D5*F5,2), D5 = 33.3333) and CSV agree with the program, so all three are a
cent off the formula the line shows. Serious: a wrong number on the money path. The
same happens through waste and an order factor that is not rounded up (each step
rounds again from the step before). Fix: carry the unrounded figure to the extension
and round only for display (or round the money once, from the exact product).

Decision found: DECISIONS.md line ~410 (call 21) "Each step is worked from the step
before as shown (four decimals), so the page can be followed by hand." So this is a
decision, and I think it is wrong: the line shows its formula (plan_length / 3, then x
unit cost), and that formula worked by hand gives $10,000.00; only the four-place
figure gives $9,999.99. DECISIONS.md ~258 says the opposite ("Figures are still worked
from unrounded inputs"). The two calls disagree; the code follows the later one.
Kept as a finding, said as a decision I would reverse (or the sheet should say that
the amount is the quantity as shown times the price).

## R21-2 (reproduced) The field list puts dollars on a measurement they do not ride on

vif.rs:117-121: for each measure a line's formula names, the line is said to rest on the
element itself (`rests.insert(sheet.path.clone())`) as well as on the measure's inputs;
vif.rs:203 then counts it against every unverified value of that element whenever the
line's amount is not Known. The line's amount can be not Known for a reason that has
nothing to do with the element (its own unit cost is Assumed), and its quantity may not
use the unverified value at all.

Repro (RDIR/j2b.json): a wall 40 ft long, line and type measured (Known), height 9 ft
Assumed; line "Base trim" = plan_length (40 LF, Known), unit cost $5 Assumed.
    plumbline vif j2b.json
    -> wall 1 [wall-0004]: ~$200.00, 1 line;  measure: ~height
Expected: measuring the height changes nothing on that line (40 LF does not use the
height; the $200 is Assumed only because the price is). Nothing should ride on the
height: no dollars, or the line listed as riding on its price, not on a field check.
Serious: the list is sorted by these dollars, so it sends the crew to measure what
does not move money, and the total "rides on them" counts it.
Fix: rest a line on the inputs of the measures its formula names (the element's own
entered value that feeds that measure: wall#run, not the whole wall), and count a line
against a value only if the line's quantity status comes from it.

## R21-3 (reproduced) The recap's hours leave out unpriced lines and still read as Known

view.rs:742-746: the hours of a class are summed only over lines whose amount has a
figure (`l.amount.value.is_some()`), and the status is the worst of those counted, with
`unwrap_or(Status::Known)` when none is. Hours that are known but not yet priced are
dropped from the hours, with no flag; the dollars beside them are flagged.

Repro (RDIR/j3.json): wall 40 ft x 9 ft, Known. Two Labor lines on net_area (360 SF) at
20 SF/hr: "Hang board" priced $60/hr, "Tape board" no price.
    plumbline estimate j3.json
    -> Labor  ~$1,080.00   1 line   18.0000 hr
By hand: the labor is 18 + 18 = 36 hours; the 18.0000 hr is printed with no mark (Known)
and nothing beside it says 18 hours are left out (the dollars do say so). A foreman
reading crew-days off the recap is short by half. Serious-ish: a lost value.
Also: a class with no counted line gets hours (0, Known); the CLI prints nothing
for it, but the struct says a Known zero (view.rs:745) - a trap for any view.
Fix: sum the hours of every line with an hour figure, whether priced or not, and say
what is left out; never default a status to Known.

## R21-4 (reproduced) The same 2.5 is priced two ways: typed, 2.5; through a value on the line, 3

lines.rs:350-354: a line in EA is rounded up to a whole number unless its formula is
one measure or a bare number. A plain value entered on the line (months, crew days,
sides) is a plain number too, but a formula that is just that value is rounded up.

Repro (RDIR/j3.json, job-0001):
    plumbline line j3.json job-0001 "Dumpster" 2.5 --cost-type general --division 1
      -> quantity 2.5 EA (kept as typed, per the comment at lines.rs:349)
    plumbline set j3.json job-0001 quantity "months" --line Dumpster --with "months=2.5" ...
    plumbline set j3.json job-0001 unit_cost 650 --line Dumpster ...
    estimate -> Dumpster 3.00 EA $650.00/EA $1,950.00  "2.5 rounded up to 3: a count is whole"
By hand: 2.5 months x $650 = $1,625.00 (what the line held a step before). Same figure,
$325 apart, by how it was entered. It is noted on the line, so it is not silent, but a
month is not a count and "a count is whole" is not true of it. Also `2.5 * 1` rounds up
while `2.5` does not. Moderate: a wrong number by the program's own rule.
Fix: round up only what is a count of pieces; let a line say it (an "exact" flag, as
the order factor has "whole/exact"), and treat a formula of plain values like a typed number.

## R21-5 (reproduced) The workbook writes counts of lines as dollars

sheets/src/lib.rs:381-386: every `Cell::Formula` is written with the dollars format
`$#,##0.00`. export.rs:220, :231, :287 put the count of lines of each cost type, of the
job cost and of each division in as `Out::Formula` (=COUNTIFS..., =SUM(C..)).
Repro: RDIR/e1.xlsx (from j1.json, `plumbline export estimate j1.json e1.xlsx`):
Recap!C4 =COUNTIFS(...) number format "$#,##0.00", cached value 1 -> shows "$1.00"
in the Lines column; Recap!C10 and Divisions!C9 the same. Expected "1".
A wrong word on a sheet that goes to people (a count of lines read as a dollar
figure). Moderate. Fix: carry the format with the formula (dollars or count).
Also suspected (not rendered here): quantities use "#,##0.####", which Excel shows
as "2." for a whole 2 (a trailing point on every whole quantity, e.g. "40." LF).

## R21-6 (reproduced) A coverage brought in from a price list is not bought in whole sheets; typed, it is

tables.rs:441: an order read from a list is `round_up: whole`, where `whole` is true
only if a "Whole ones only" column says yes. A supplier's list has no such column, so
"32 SF/sheet" comes in as exact. Typed at the keyboard (job.rs:272) the same factor is
whole by default (`bottom == bought`: a package is bought whole, call 21). And once the
import has made it exact, a later typed `set ... order "32 SF/sheet"` keeps exact
(job.rs:272 `said.or(old round_up)`).
Repro: wall 10 ft x 10 ft (face_area 100 SF), material "Gyp board" bought by the sheet,
line "Board" = face_area, made of it.
  a) RDIR/list.csv  "Material,Unit,Price,Coverage / Gyp board,sheet,12.00,32 SF/sheet"
     import prices j4.json list.csv --status Known --source made-up
     -> Board 3.1250 sheet $12.00/sheet $37.50
  b) (j4c.json) set material-0005 price 12; set material-0005 order "32 SF/sheet"
     -> Board 4.00 sheet $12.00/sheet $48.00
By hand, by the program's own rule (a package is bought whole): 100 / 32 = 3.125 ->
4 sheets x $12 = $48.00. The import gives $37.50: $10.50 short on one wall, silently
(the import report says "ordered at 32 SF/sheet" and nothing about whole ones).
Serious: a wrong number from an import, with no word.
Fix: one default for both roads (whole when the factor is per the bought unit), and
the import report says which it took.

## R21-7 (reproduced) A prior estimate adds to two different totals

tables.rs:979-984 (`rows`, what `show` prints) adds only the lines' written amounts and
leaves a line with a quantity and unit cost but no amount out of the total without a
word. tables.rs:1101 (`compare`) adds the same lines with quantity x unit cost worked
for such a line. One figure, two arithmetics.
Repro: RDIR/prior.csv  "Trim run,2,LF,10,(no amount)" and "Paint,,,,50":
    plumbline import prior j5.json prior.csv --status Known --source "made-up prior"
    plumbline show j5.json     -> lines of a prior estimate 2, adding to $50.00
    plumbline compare j5.json  -> the prior estimate's lines add to $70.00
By hand: 2 x $10 + $50 = $70.00. `show` says $50.00, Known, and does not say it left
a line out. Moderate: a total that leaves something out without saying so.
Fix: one function for what a prior line comes to (comes_to), used by both.

## R21-8 (reproduced) A door's casing cannot be taken off by its size; a width alone is priced "per ft"

components.rs:229-233 give a wall's height and a door's or window's width and height
the unit "ft" (not LF). lines.rs:63-69 refuses any measure in a unit other than LF, SF
or none inside a formula: "uses height, which is in ft; inside a formula use the
figure it comes from, or price the line by height alone". lines.rs:97-104 lets the
measure stand alone and keeps "ft" as the line's unit.
Repro (RDIR/j6.json): door type 30 in x 80 in in wall-0004.
    plumbline line j6.json door-0006 "Casing" "2 * height + width"
      -> refused: ... uses height, which is in ft; inside a formula use the figure it comes from ...
    plumbline line j6.json wall-0004 "Corner bead" "2 * height"   -> refused the same way
    plumbline line j6.json door-0006 "Casing" "width"  -> accepted: "2.50 ft", priced in $/ft
Expected: casing by the opening is 2 x 6.6667 + 2.5 = 15.8333 LF (the trade's
two legs and a head); a length on a sheet is LF. The refusal's advice is not true:
there is no "figure it comes from" a person can name in the formula. The made-up house
works round it by pricing casing at "1" EA an opening (sample.rs:710).
Moderate: a takeoff the trade does every day cannot be written, and a unit "ft" on a
priced line is not how the trade writes it (CLAUDE.md "Speak construction": LF).
Fix: give these measures the unit LF (they are lengths) so they enter formulas as
dim 1; or say in the refusal what can be used.

## R21-9 (reproduced) Bringing in a supplier's catalogue takes time that grows with the square of its rows

tables.rs:242-462 (`prices_in`) does, for every row: `materials.iter().find(same_name)`
over every material so far (same_name builds two new strings each time, check.rs:493),
`job.find(&path)` (job.rs:93: builds a map of every element, each call), and with --add
`job.add_material` (job.rs:319: a query over every element for the name, `add` ->
`find`, `children()`, and `index()` over every element again for the highest number).
Repro (made-up lists, RDIR/big1000.csv, big4000.csv: "Made-up item N,EA,price"):
    plumbline import prices jb1000.json big1000.csv --add --status Assumed --source made-up   5.7 s
    plumbline import prices jb4000.json big4000.csv --add ...                                  93.2 s
Four times the rows, sixteen times the time. The reader accepts 200,000 rows
(sheets MOST_ROWS); at this rate that is days. A real supplier catalogue of 10,000
lines is ~10 minutes with no word on screen. Serious for a person bringing in a list
(it reads as a hang). Fix: one index of the job's materials by plain name and one of
paths, built once per import; `Job::find` keeps an index instead of rebuilding it.

## R21-10 (reproduced, minor) A refusal names a wall that does not exist

`plumbline wall j1.json level-0002 wall-type-0003 --points "0,0 100,0" ...` (a type
path typed wrong) -> "wall-0004: its type is "wall-type-0003", which is not a wall type
in this job". The wall was rolled back; wall-0004 is not in the job (the next wall
drawn takes that path). building.rs:425 speaks from check_whole, after the add. The
refusal also does not offer the wall types the job has (type-0003 "Stud 6"), as the
CLI's `plainly` does for "no element". Minor: an untrue word in a refusal.

## R21-11 (suspected) More than 24 suppliers writes formulas that are not cells

export.rs:100-103 `col(i)` = b'A' + i: the 25th supplier column is '[' and the sum
formula becomes "=SUM([5:[9)". A workbook with such a formula is reported damaged
by Excel on opening. Untested here (24+ quotes needed). Minor; easy to fix (AA, AB...).

## R21-12 (code, minor) A material is found by its name two ways

job.rs:216-225 (`set material`) and tables.rs:348 match a material by `same_name`
(capitals, spacing, a full stop), but derive.rs:285 and view.rs:706 find it with `==`
on the name or path. A job file whose element says "copper, 16 oz" for a material
named "Copper, 16 oz" (a hand edit or another program) prices nothing: "the job has no
material of that name". Suspected; not reproduced (needs a hand-edited file).

## R21-13 (code) Arithmetic done in the windows, against "one arithmetic for one number"

- cli main.rs:643-648 and export.rs:291-297 each add the sixteen divisions themselves
  (view::Estimate has no such total) and each work out its status their own way.
- cli main.rs:444-450 works out the like-for-like difference and percentage
  (view::Pricing.both carries only the two sums).
- export.rs:390 adds a supplier's quantities and rounds them to four places.
- tables.rs:982 adds a prior's amounts (R21-7), unlike compare.
They agree today; two of them already disagree in status rules. Each belongs in view.rs.

## R21-14 (code) Smaller things that make it hard to keep

- export.rs:153-154: the same comment line twice.
- export.rs:276 `d.total.left_out.iter().any(|_| true)` is `!is_empty()`.
- view.rs:745 hours status `unwrap_or(Status::Known)` and vif.rs:132 `(None, Status::Known, ..)`:
  a status for nothing should not be Known (R21-3).
- `Job::find`/`index` rebuild a map of every element on each call (job.rs:88-95); it is
  called in loops (vif.rs:112, tables.rs:366, export lens code), the root of R21-9.
- lines.rs:173-188 `cents` push and round4 in value.rs: two roundings of money/figures with
  different rules; value.rs:162-166 rounds at 4 places with no push, so a figure at
  .00005 rounds by binary noise (unlike cents). Untidy; no wrong cent found from it.
- DECISIONS.md call 21 (~line 410) and the note at ~258 disagree on whether steps are
  worked from rounded figures (R21-1).
- clash.rs:737 overwrites the library's depth with the "least way out" and leaves None
  when no way out clears, so a hard clash with a wall of two pieces or more can lose a
  depth the library did give. Suspected, not reproduced.
- sheets lib.rs:163: a cell the user wrote beginning with an apostrophe then "-", "=",
  "+" or "@" (e.g. "'-ish") loses its apostrophe on the way back in (the writer does not
  escape a leading apostrophe). Suspected, minor.

## R21-2, addendum
vif.rs head comment (call 54): dollars riding on a thing are the lines "whose quantity
is worked from a figure that rests on that thing's values, and whose amount is itself
not Known". plan_length's input is `wall-0004#pl::wall`, the whole wall, so by the
letter the line rests on the wall. The two tests are each true and together wrong: the
amount is not Known because of the price, the quantity is Known. The rule should be
"whose quantity is not Known because of that thing's values".

---

# The reviewer's report

# Review 21: the model and the money (report)

Reviewer that did not build it, 2026-11-16. Nothing in the repository was changed.
Details, commands and outputs for each finding are in NOTES.md beside this file; the
made-up jobs used are in this folder (j1..j6, j2b, j4, j4c, j5, h.json, list.csv,
prior.csv, big*.csv, e1.xlsx).

## Findings, most serious first

1. R21-1 (reproduced) lines.rs:361/374/393 with value.rs:151: every step of a line is
   rounded to four places before the next (quantity, waste, order, then money). A line
   "plan_length / 3" on a 100 LF wall at $300/LF is $9,999.99; by its own formula,
   $10,000.00. The xlsx and CSV carry the same cent. A decision (DECISIONS call 21,
   "worked from the step before as shown") that another note (~258) contradicts; I
   would reverse it.
2. R21-6 (reproduced) tables.rs:441 vs job.rs:272: a coverage brought in from a price
   list (32 SF/sheet, no "whole ones" column) is ordered exact; typed, whole. 100 SF of
   board is 3.125 sheets $37.50 from the import, 4 sheets $48.00 typed. Silent. And a
   later typed factor keeps the import's "exact".
3. R21-2 (reproduced) vif.rs:117-121, 203: the field list puts a line's dollars on a
   measurement its quantity does not use, because its price is Assumed. Wall height
   Assumed, base trim on plan_length (Known) at an Assumed $5: "$200.00 rides on
   measuring the height". The list is sorted by these dollars.
4. R21-9 (reproduced) tables.rs:242-462, job.rs:93/319: importing a price list is
   quadratic: 1,000 rows 5.7 s, 4,000 rows 93 s; the reader allows 200,000.
5. R21-3 (reproduced) view.rs:742-746: the recap's hours leave out labor lines that have
   no price and still print as Known (18.0000 hr of 36), the PM lens's Lines sheet
   shows both lines' hours, its Budget the 18.
6. R21-4 (reproduced) lines.rs:350-354: 2.5 typed is 2.5 EA; the same 2.5 entered as a
   value on the line ("months") is rounded up to 3, $1,950 against $1,625. Noted on the
   line, but "a count is whole" is not true of months.
7. R21-7 (reproduced) tables.rs:979-984 vs :1101: a prior estimate "adds to $50.00" in
   `show` and "$70.00" in `compare`; `show` leaves out a qty x unit-cost line silently.
8. R21-8 (reproduced) components.rs:229-233, lines.rs:63-69: an opening's width and
   height and a wall's height are in "ft" and refused inside a formula, so casing
   "2 * height + width" cannot be written; "width" alone is priced "per ft", not LF.
   The refusal's advice ("use the figure it comes from") cannot be followed.
9. R21-5 (reproduced) sheets lib.rs:381-386, export.rs:220/231/287: in the workbook
   every formula cell is formatted as dollars, so line counts read "$1.00".
   Suspected beside it: "#,##0.####" shows whole quantities as "40.".
10. R21-10 (reproduced, minor) building.rs:425: a refusal names "wall-0004", a wall that
    was rolled back and does not exist.
11. R21-12 (suspected) derive.rs:285, view.rs:706 match a material with `==`; job.rs and
    tables.rs with same_name. A file naming it in other capitals prices nothing.
12. R21-11 (suspected) export.rs:100: more than 24 suppliers make formulas like
    "=SUM([5:[9)".
13. R21-13 (code) arithmetic in the windows: the sixteen divisions' sum (cli 643, export
    291), like-for-like difference and percent (cli 444-450), a prior's total (tables 982).
14. R21-14 (code) smaller: duplicate comment export.rs:153-154; `any(|_| true)`
    export.rs:276; statuses defaulting to Known for nothing (view.rs:745, vif.rs:132);
    clash.rs:737 may drop a known depth (suspected); CSV apostrophe edge (sheets :163).

Checked and found right: the sixteen-division house (`sixteen`) recap, overhead, profit
and price to the cent by hand; the sixteen divisions add to the job cost; d5's labor,
off-switch and O&P arithmetic; formula parser limits (depth, tokens, divide by zero);
negative counts refused before rounding; waste/order/markup ranges in check.rs; job
file duplicate keys and unknown keys.

## Files read whole
crates/world/src: lines.rs, formula.rs, derive.rs, view.rs, vif.rs, job.rs, export.rs,
tables.rs, units.rs, value.rs, file.rs, check.rs; building/clash.rs; crates/sheets/src/lib.rs.
Read in part: building.rs (491-930 measures; 1700-2088 takeoff and storey totals;
not 1-490 checks, 930-1700 rooms/placements/rows); components.rs (searched only);
crates/cli/src/main.rs (162-702 vif, clashes, takeoff, price, estimate; 1176-1190,
1575-1600; the rest searched for unwrap/indexing only). Not read: editing*, plans.rs,
spec.rs, solve.rs, roof.rs, network.rs, rooms.rs, runs.rs, walls.rs, crates/ifc, crates/plans.

## The ten changes I would make first
1. Carry unrounded figures through quantity, waste and order; round money once, at the
   extension (or say on the sheet that the amount is the quantity as shown x price).
2. One default for "bought whole" on every road in (typed, price list, library), and
   say in the import report which it took.
3. VIF: a line rides on a thing only if its quantity's status comes from that thing's
   entered values; never because of its price.
4. Index the job (path -> entity, plain name -> material) once per command; make
   prices_in linear.
5. Hours in the recap and both lens sheets: every line with an hour figure, priced or
   not, with what is left out said and a status that is not Known by default.
6. Let a line say a plain value is exact (months, days) and stop rounding a formula of
   plain values; round up only counts of pieces.
7. Give opening width/height and wall height the unit LF so formulas can use them; fix
   the refusal text.
8. One `comes_to` for a prior line, used by show and compare.
9. Carry a format with each workbook formula (dollars, count, quantity); column letters
   past Z.
10. Move every total a window adds (sixteen divisions, like-for-like, supplier
    quantities) into view.rs, and match materials by one rule everywhere.
