#!/usr/bin/env bash
# The estimate written out for other people: a workbook whose figures are formulas, the
# same as rows of values, and three lenses on the one estimate (what a supplier is asked
# to quote, a subcontractor's scope, and the project manager's lines, budget and list to
# verify).
# Run from the repository root:  demos/export.sh
# The job is the made-up priced job (`plumbline priced`; no real job data), in a
# temporary folder. Every step fails the script if the program does not do what the step
# says. The figures expected are worked by hand in demos/d5.sh, in STATUS.md ("Exports
# and lenses") and in the comments here.
#
# The workbook's formulas are worked out here by a second hand: tools/workbook.py, which
# knows ROUND, SUM, SUMIF, the four operations and cell references and nothing of this
# program. It needs Python 3 with openpyxl.
set -euo pipefail
cd "$(dirname "$0")/.."
python3 -c "import openpyxl" 2>/dev/null || { echo "demos/export.sh needs Python 3 with openpyxl, which is not installed"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
not_said() { ! grep -qiF -- "$2" "$1" || { echo "FAIL: it says: $2"; cat "$1"; exit 1; }; }

echo "== The job: the made-up priced job, overhead 10, profit 8 =="
"$P" priced "$J" >/dev/null
"$P" set "$J" job-0001 overhead 10 --status Known --source "made up" >/dev/null
"$P" set "$J" job-0001 profit 8 --status Known --source "made up" >/dev/null
"$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"
# Material 8,628.10 + labor 2,147.97 + fabrication 402.50 + supervision 3,800.00 +
# subcontract 10,313.86 = 25,292.43. Overhead 10%: 2,529.24. Profit 8% of 27,821.67:
# 2,225.73. Price 30,047.40.
says "$T/sheet.txt" "Job cost ~\$25,292.43 13 lines"
says "$T/sheet.txt" "PRICE ~\$30,047.40"

echo
echo "== The workbook: every amount, the recap and the price are formulas =="
"$P" export estimate "$J" "$T/estimate.xlsx" | sed "s|$T/||g; s/^/    /"
python3 tools/workbook.py "$T/estimate.xlsx" > "$T/worked.txt"
# Each formula, worked out from the cells it names, comes to the figure the program wrote
# beside it; tools/workbook.py fails if any does not.
# Thirteen amounts, five cost types, the job cost, overhead, profit, the price, the sum of
# the divisions (this job's lines carry none: the sheet says so), and five supplier totals.
says "$T/worked.txt" "27 formulas worked out; each comes to the figure written with it"
says "$T/worked.txt" "Divisions!In no division"
says "$T/worked.txt" "Recap!Material = 8628.10"
says "$T/worked.txt" "Recap!Labor = 2147.97"
says "$T/worked.txt" "Recap!Fabrication = 402.50"
says "$T/worked.txt" "Recap!Supervision = 3800.00"
says "$T/worked.txt" "Recap!Subcontract = 10313.86"
says "$T/worked.txt" "Recap!Job cost = 25292.43"
says "$T/worked.txt" "Recap!Overhead = 2529.24"
says "$T/worked.txt" "Recap!Profit = 2225.73"
says "$T/worked.txt" "Recap!Price = 30047.40"
says "$T/worked.txt" "Recap!General conditions: none"
says "$T/worked.txt" "Left out of the job cost, and of everything taken on it: 1 line"

echo
echo "== A price changed in the workbook: the formulas follow, to the program's own figures =="
# The south plane's underlayment, $0.90 to $0.95/SF: 419.2627 SF is $398.30, up $20.96.
# Material 8,649.06; job cost 25,313.39; overhead 2,531.34; profit 8% of 27,844.73,
# 2,227.58; price 30,072.31.
python3 tools/workbook.py "$T/estimate.xlsx" --set "Estimate" "south" "Underlayment" 0.95 > "$T/changed.txt"
says "$T/changed.txt" "Estimate: south, Underlayment: the price 0.9 changed to 0.95; its amount 377.34 is now 398.30"
says "$T/changed.txt" "Recap!Material = 8649.06"
says "$T/changed.txt" "Recap!Job cost = 25313.39"
says "$T/changed.txt" "Recap!Overhead = 2531.34"
says "$T/changed.txt" "Recap!Profit = 2227.58"
says "$T/changed.txt" "Recap!Price = 30072.31"
# The same price changed in the job: the program comes to the same.
cp "$J" "$T/changed.json"
SOUTH="$("$P" show "$J" | sed -n 's/^ *\(plane-[0-9]*\)  south  .*/\1/p' | head -1)"
"$P" set "$T/changed.json" "$SOUTH" unit_cost '$0.95/SF' --line "Underlayment" --status Known --source "made up" >/dev/null
"$P" estimate "$T/changed.json" | tr -s ' ' > "$T/sheet2.txt"
says "$T/sheet2.txt" "Underlayment 419.26 SF \$0.95/SF \$398.30"
says "$T/sheet2.txt" "Job cost ~\$25,313.39 13 lines"
says "$T/sheet2.txt" "Overhead at 10.00 % ~\$2,531.34"
says "$T/sheet2.txt" "Profit at 8.00 % ~\$2,227.58"
says "$T/sheet2.txt" "PRICE ~\$30,072.31"

echo
echo "== The same as rows of values, and back in as an estimate made before =="
"$P" export estimate "$J" "$T/estimate.csv" | sed "s|$T/||g; s/^/    /"
says "$T/estimate.csv" "Element,Line,Cost type,Quantity,Unit,Unit cost,Amount,How sure,Note,Division"
says "$T/estimate.csv" "south,Underlayment,Material,419.2627,SF,0.9,377.34,Known,"
says "$T/estimate.csv" "the job,\"Foreman, days on site\",Supervision,40,hr,95,3800.00,Known,"
cp "$J" "$T/back.json"
"$P" import prior "$T/back.json" "$T/estimate.csv" --name "written out" --status Known --source "this job's own estimate, written out" > "$T/in.txt"
says "$T/in.txt" "a prior estimate of 14 lines"
# What a prior estimate has no place for is said, not dropped in silence.
says "$T/in.txt" 'not read: the column "Element"'
"$P" compare "$T/back.json" > "$T/compare.txt"
# Every line that comes to something agrees to the cent, and no quantity differs.
[ "$(grep -c 'amount +\$0.00\|amount ~+\$0.00' "$T/compare.txt")" = 8 ] || { echo "FAIL: not every line agrees"; cat "$T/compare.txt"; exit 1; }
! grep -E 'amount ~?[+-]\$[0-9,]*\.[0-9]{2}' "$T/compare.txt" | grep -vq '+\$0.00' || { echo "FAIL: a line differs"; cat "$T/compare.txt"; exit 1; }
! grep -E 'quantity [~?]?[+-][0-9.,]+ ' "$T/compare.txt" | grep -vq '+0.00 ' || { echo "FAIL: a quantity differs"; cat "$T/compare.txt"; exit 1; }
echo "    set against the job it came from: nine names, eight with an amount, each +\$0.00; the ninth has no price in either"

echo
echo "== The supplier's lens: what to quote, and none of our prices =="
"$P" export supplier "$J" "$T/supplier.csv" | sed "s|$T/||g; s/^/    /"
# 461.1890 + 276.7134 = 737.9024 lb; 419.2627 + 251.5576 = 670.8203 SF; 123.6931 + 30 =
# 153.6931 lb, Assumed (the shed's pitch is).
says "$T/supplier.csv" "None of our prices are in this list."
says "$T/supplier.csv" '"Copper, 16 oz",737.9024,lb,2,Known,'
says "$T/supplier.csv" "Underlayment,670.8203,SF,2,Known,"
says "$T/supplier.csv" '"Copper, 20 oz",153.6931,lb,2,Assumed,'
says "$T/supplier.csv" "Not in this list: 1 line with no class"
for ours in '$' "9.00" "0.90" "4150" "8628" "Install panels" "Fabricate" "Foreman" "Amount" "Unit cost"; do not_said "$T/supplier.csv" "$ours"; done
echo "    no price, no amount, no labor line in it"

echo
echo "== The subcontractor's lens: the scope classed Subcontract, and none of our prices =="
"$P" export subcontractor "$J" "$T/sub.csv" | sed "s|$T/||g; s/^/    /"
says "$T/sub.csv" "None of our prices are in this list."
says "$T/sub.csv" 'west,"Standing seam roofing, by subcontractor",251.5576,SF,Known,'
for ours in '$' "41" "10313" "Amount" "Unit cost" "Copper, 16" "Install panels"; do not_said "$T/sub.csv" "$ours"; done
echo "    one line; no price and no amount in it"

echo
echo "== The project manager's lens: every line, the budget by class, what is to verify; no markup =="
"$P" export pm "$J" "$T/pm.csv" | sed "s|$T/||g; s/^/    /"
says "$T/pm.csv" "No overhead, no profit and no price are in this list."
says "$T/pm.csv" "south,Install panels,Labor,,16.7705,hr,16.7705,1207.48,Known,"
says "$T/pm.csv" "Labor,2147.97,29.8328,3,Assumed"
says "$T/pm.csv" "Fabrication,402.50,6.7082,2,Known"
says "$T/pm.csv" "Supervision,3800.00,40,1,Known"
says "$T/pm.csv" "Job cost,25292.43,,13,Assumed"
says "$T/pm.csv" "to verify in the field"
for markup in "2529.24" "2225.73" "30047.40"; do not_said "$T/pm.csv" "$markup"; done
! grep -qE '^"?(Overhead|Profit|Price)' "$T/pm.csv" || { echo "FAIL: the PM's lens has a row of markup"; cat "$T/pm.csv"; exit 1; }
echo "    no overhead, no profit, no price: not as a row, and not as a figure"
"$P" export pm "$J" "$T/pm.xlsx" | sed "s|$T/||g; s/^/    /"
python3 tools/workbook.py "$T/pm.xlsx" > "$T/pm.txt"
says "$T/pm.txt" "3 sheets: Lines, Budget, Verify in field"

echo
echo "== The building as an IFC4 model: validated by IfcOpenShell, and read back by this program =="
# The made-up whole house: two levels; eight walls a level, 210.00 LF a level, with 5 and
# 4 doors and 8 and 9 windows; a floor of 1,120.00 SF a level; five rooms a level; a
# roof of four planes; and its ducts, pipes, conduit, cable, fittings, fixtures and
# equipment on their systems.
"$P" house "$T/house.json" >/dev/null
"$P" export ifc "$T/house.json" "$T/house.ifc" > "$T/ifc.txt"
sed "s|$T/||g; s/^/    /" "$T/ifc.txt" | cut -c1-200 | head -4
says "$T/ifc.txt" "2 levels"
says "$T/ifc.txt" "16 walls"
says "$T/ifc.txt" "9 doors"
says "$T/ifc.txt" "17 windows"
says "$T/ifc.txt" "2 floors"
says "$T/ifc.txt" "10 rooms"
says "$T/ifc.txt" "4 roof planes"
# What a model has no place for, or what has no shape, is counted and said.
says "$T/ifc.txt" "its size is not known, so it has no shape to write"
says "$T/ifc.txt" "x type: each thing is written with its own size"
if python3 -c "import ifcopenshell" 2>/dev/null; then
  python3 tools/ifc-check.py "$T/house.ifc" > "$T/checked.txt" || { echo "FAIL: IfcOpenShell finds a fault in the model"; cat "$T/checked.txt"; exit 1; }
  says "$T/checked.txt" "IFC4: no fault"
  says "$T/checked.txt" "2 IfcBuildingStorey, 16 IfcWall, 9 IfcDoor, 17 IfcWindow, 2 IfcSlab that are floors, 10 IfcSpace, 1 IfcRoof"
  says "$T/checked.txt" "lengths in foot"
else
  echo "demos/export.sh needs IfcOpenShell to validate the model (python3 -c 'import ifcopenshell' fails; set PYTHONPATH to where it is)"; exit 2
fi
# Read back into an empty job: the same levels, walls, floors and rooms, to the figure.
"$P" new "$T/readback.json" "Read back (not a real job)" >/dev/null
"$P" import ifc "$T/readback.json" "$T/house.ifc" --status Assumed --source "this program's own model, written out" > "$T/in.txt"
says "$T/in.txt" "16 walls"
says "$T/in.txt" "10 rooms"
# What this program has no element for it says, by kind and count; nothing goes missing.
says "$T/in.txt" "22 fixture or outlet: this program has no element for it"
level() { "$P" takeoff "$1" | sed -n "/^$2 /,/doors/p" | sed -E 's/^ +[~?]? ?//; s/ +/ /g; /^(first|second) floor/d'; }
for floor in "first floor" "second floor"; do
  level "$T/house.json" "$floor" > "$T/was.txt"; level "$T/readback.json" "$floor" > "$T/is.txt"
  [ -s "$T/was.txt" ] && cmp -s "$T/was.txt" "$T/is.txt" || { echo "FAIL: the $floor read back is not the $floor written"; diff "$T/was.txt" "$T/is.txt"; exit 1; }
  sed "s/^/    $floor: /" "$T/is.txt" | head -5
done
echo "    read back: each level's walls, wall faces, floor, rooms, doors and windows are what was written"

echo
echo "== What is refused =="
if out="$("$P" export estimate "$J" "$T/estimate.pdf" 2>&1)"; then echo "FAIL: accepted a file that is neither"; exit 1; fi
echo "    refused: $out" | head -1 | sed "s|$T/||g" | cut -c1-200
grep -qF "a workbook (.xlsx) or rows of values (.csv)" <<<"$out" || { echo "FAIL: the refusal does not say what can be written"; exit 1; }

echo
echo "PASS: the estimate as a workbook of formulas and as values, the three lenses, and the building as an IFC4 model, each to the figures worked by hand"
