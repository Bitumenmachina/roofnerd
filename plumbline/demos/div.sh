#!/usr/bin/env bash
# Sixteen divisions: a small house estimated across every division, from General
# Requirements to Electrical. Every cost line carries its division; the estimate reads
# division by division; a line with no division is said to have none.
# Run from the repository root:  demos/div.sh
# The job is made up (no real job data) and lives in a temporary folder; every price in
# it is made up. Every step fails the script if the program does not do what the step
# says. The figures expected are worked by hand in STATUS.md ("Sixteen divisions") and in
# the comments here.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
PRICE=(--status Known --source "made-up price")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
sheet() { "$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"; }
says() { grep -qF -- "$2" "$T/sheet.txt" || { echo "FAIL: $1: the estimate does not say: $2"; cat "$T/sheet.txt"; exit 1; }; printf '    %s\n' "$2"; }
refused() { local want="$1"; shift; local out
  cp "$J" "$T/kept.json"
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | head -1 | cut -c1-200
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
  cmp -s "$J" "$T/kept.json" || { echo "FAIL: a refusal changed the file"; exit 1; }
}
# Gravel and concrete are bought by the whole cubic yard, as anything ordered by a unit is.
# A line on a thing, in a division, with its price: path, description, quantity, cost type, division, price.
line() { "$P" line "$J" "$1" "$2" "$3" --cost-type "$4" --division "$5" >/dev/null; "$P" set "$J" "$1" unit_cost "$6" --line "$2" "${PRICE[@]}" >/dev/null; }

echo "== The house: one level, four walls, a door, two windows, a floor, a room, a roof, a duct, a pipe, a cable, a lavatory, a furnace =="
"$P" new "$J" "A house in sixteen divisions (not a real job)" >/dev/null
JOB="$("$P" show "$J" | sed -n 's/^ *\(job-[0-9]*\) .*/\1/p' | head -1)"
S="$(made "$("$P" level "$J" Ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 30,0 30,20 0,20" --closed --height "9 ft" "${K[@]}" | sed -E 's/^[^:]*: //; s/,//g')"
DT="$(made "$("$P" type "$J" door "door 3 x 7" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 4 x 4" --width "4 ft" --height "4 ft" "${K[@]}")")"
DOOR="$(made "$("$P" door "$J" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${K[@]}")")"
WIN1="$(made "$("$P" window "$J" "$SOUTH" "$NT" --offset "14 ft" --sill "3 ft" "${K[@]}")")"
WIN2="$(made "$("$P" window "$J" "$SOUTH" "$NT" --offset "24 ft" --sill "3 ft" "${K[@]}")")"
FT="$(made "$("$P" type "$J" floor "slab 4 in" --thickness "4 in" "${K[@]}")")"
FLOOR="$(made "$("$P" floor "$J" "$S" "$FT" --walls "${K[@]}")")"
ROOM="$(made "$("$P" room "$J" "$S" "Great room" --at "15,10" "${K[@]}")")"
"$P" roof "$J" "Main roof" --points "0,0 30,0 30,20 0,20" --pitch 6:12 --eave-height "9 ft" "${K[@]}" >/dev/null
PLANES="$("$P" show "$J" | sed -n 's/^ *\(plane-[0-9]*\) .*/\1/p' | tr '\n' ' ')"
RIDGE="$("$P" show "$J" | sed -n 's/^ *\(ridge-[0-9]*\) .*/\1/p' | head -1)"
UT="$(made "$("$P" type "$J" duct "6 in round" --round "6 in" "${K[@]}")")"
PT="$(made "$("$P" type "$J" pipe "3/4 in" --round "3/4 in" "${K[@]}")")"
CT="$(made "$("$P" type "$J" cable "12-2" --round "1/2 in" "${K[@]}")")"
DUCT="$("$P" run "$J" "$S" duct "$UT" --points "5,10 25,10" --height "8 ft" "${K[@]}" | sed -E 's/^[^:]*: //')"
PIPE="$("$P" run "$J" "$S" pipe "$PT" --points "8,3 8,17" --height "1 ft" "${K[@]}" | sed -E 's/^[^:]*: //')"
read -r CABLE1 CABLE2 <<<"$("$P" run "$J" "$S" cable "$CT" --points "1,19 29,19 29,7" --height "8 ft 6 in" "${K[@]}" | sed -E 's/^[^:]*: //; s/,//g')"
LAV="$(made "$("$P" place "$J" "$S" fixture "Lavatory" --at "8,17.5" --offset "0 ft" --size "2,1.5,3" "${K[@]}")")"
FURNACE="$(made "$("$P" place "$J" "$S" equipment "Furnace" --at "26,12" --offset "0 ft" --size "2,2,4" "${K[@]}")")"
"$P" check "$J"
"$P" takeoff "$J" | sed -n '/^Ground/,/doors/p' | sed 's/^/    /'

echo
echo "== One assembly, brought in as a library, puts lines in three divisions on each wall =="
# "Exterior wall, 2x6, sided": framing by the gross face (Division 6); insulation and
# siding by the net face (7); gypsum board and paint by the net face (9).
python3 - "$T/assemblies.csv" <<'PY'
import csv, sys
head = ["Assembly", "Goes on", "Cost code", "Code title", "Code scheme", "Code: how sure", "Code: where from", "Line", "Cost type", "Quantity",
        "Stretch-out", "Stretch-out: how sure", "Stretch-out: where from", "Waste %", "Waste: how sure", "Waste: where from",
        "Ordered by", "Ordered at", "Whole ones only", "Ordered at: how sure", "Ordered at: where from",
        "Unit cost", "Unit cost: how sure", "Unit cost: where from", "Made of the element's material", "Off", "Division"]
lines = [("Wall framing, 2x6 at 16 in", "face_area", "$4.20/SF", 6), ("Batt insulation", "net_area", "$1.60/SF", 7), ("Siding, installed", "net_area", "$9.50/SF", 7),
         ("Gypsum board, hung and finished", "net_area", "$2.40/SF", 9), ("Paint, two coats", "net_area", "$1.10/SF", 9)]
with open(sys.argv[1], "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(head)
    for name, quantity, price, division in lines:
        w.writerow(["Exterior wall, 2x6, sided", "Wall", "", "", "", "Unknown", "not coded", name, "Material", quantity] + [""] * 11 + [price, "Known", "made-up price", "", "", division])
PY
"$P" import assemblies "$J" "$T/assemblies.csv" | sed "s|$T/||g; s/^/    /" | head -4
for w in "$SOUTH" "$EAST" "$NORTH" "$WEST"; do "$P" apply "$J" "$w" "Exterior wall, 2x6, sided" --source "made-up finish schedule" >/dev/null; done
# Written out again and read into an empty job, the assembly keeps its divisions.
"$P" export assemblies "$J" "$T/again.csv" >/dev/null
"$P" new "$T/other.json" "another (not a real job)" >/dev/null
"$P" import assemblies "$T/other.json" "$T/again.csv" >/dev/null
"$P" export assemblies "$T/other.json" "$T/third.csv" >/dev/null
cmp "$T/again.csv" "$T/third.csv" || { echo "FAIL: the assembly library does not come back as it was written"; exit 1; }
grep -q ',6$' "$T/again.csv" && grep -q ',7$' "$T/again.csv" && grep -q ',9$' "$T/again.csv" || { echo "FAIL: the library does not carry each line's division"; cat "$T/again.csv"; exit 1; }
echo "    the library written out, read into another job and written again is the same file; each line carries its division"

echo
echo "== The other lines, each in its division =="
# In-house supervision is in hours: five days at eight hours, then its rate, then its cost type.
"$P" line "$J" "$JOB" "Superintendent, days on site" 5 --division 1 >/dev/null
"$P" set "$J" "$JOB" order "8 hr/EA" --line "Superintendent, days on site" "${K[@]}" >/dev/null
"$P" set "$J" "$JOB" unit_cost '$85/hr' --line "Superintendent, days on site" "${PRICE[@]}" >/dev/null
"$P" class "$J" "$JOB" supervision --line "Superintendent, days on site" >/dev/null
line "$JOB" "Building permit" 1 general 1 '$1,200'
# Bought by the cubic yard: 6 in deep is 54 SF to the CY; a 4 in slab is 81 SF to the CY.
"$P" line "$J" "$FLOOR" "Excavate and gravel base, 6 in" plan_area --cost-type subcontract --division 2 >/dev/null
"$P" set "$J" "$FLOOR" order "54 SF/CY" --line "Excavate and gravel base, 6 in" "${K[@]}" >/dev/null
"$P" set "$J" "$FLOOR" unit_cost '$48.60/CY' --line "Excavate and gravel base, 6 in" "${PRICE[@]}" >/dev/null
"$P" line "$J" "$FLOOR" "Concrete slab, 4 in" plan_area --cost-type material --division 3 >/dev/null
"$P" set "$J" "$FLOOR" order "81 SF/CY" --line "Concrete slab, 4 in" "${K[@]}" >/dev/null
"$P" set "$J" "$FLOOR" unit_cost '$165/CY' --line "Concrete slab, 4 in" "${PRICE[@]}" >/dev/null
"$P" line "$J" "$SOUTH" "Stone veneer, 3 ft band" plan_length --cost-type material --division 4 >/dev/null
"$P" set "$J" "$SOUTH" quantity "plan_length * band" --with "band=3 ft" --line "Stone veneer, 3 ft band" "${K[@]}" >/dev/null
"$P" set "$J" "$SOUTH" unit_cost '$38/SF' --line "Stone veneer, 3 ft band" "${PRICE[@]}" >/dev/null
line "$RIDGE" "Steel ridge beam" true_length material 5 '$42/LF'
for p in $PLANES; do line "$p" "Standing seam roofing" surface_area material 7 '$14/SF'; done
line "$DOOR" "Entry door, prehung" 1 material 8 '$2,400'
line "$WIN1" "Window unit" 1 material 8 '$1,150'
line "$WIN2" "Window unit" 1 material 8 '$1,150'
line "$ROOM" "Hardwood flooring" floor_area material 9 '$12/SF'
line "$ROOM" "Toilet and bath accessories" 1 material 10 '$450'
line "$ROOM" "Kitchen appliances" 1 material 11 '$8,500'
line "$ROOM" "Window blinds" 2 material 12 '$380'
line "$JOB" "Sauna, prefabricated" 1 subcontract 13 '$9,200'
line "$JOB" "Platform lift" 1 subcontract 14 '$14,500'
line "$DUCT" "Duct, installed" true_length subcontract 15 '$18/LF'
line "$PIPE" "Pipe, installed" true_length subcontract 15 '$9.50/LF'
line "$LAV" "Lavatory, set" 1 subcontract 15 '$640'
line "$FURNACE" "Furnace, set" 1 subcontract 15 '$4,300'
line "$CABLE1" "Cable, pulled" true_length subcontract 16 '$2.25/LF'
line "$CABLE2" "Cable, pulled" true_length subcontract 16 '$2.25/LF'
line "$ROOM" "Light fixtures" 6 subcontract 16 '$185'
"$P" set "$J" "$JOB" overhead 10 "${K[@]}" >/dev/null
"$P" set "$J" "$JOB" profit 8 "${K[@]}" >/dev/null
"$P" check "$J"

echo
echo "== The estimate, division by division =="
sheet
sed -n '/^BY DIVISION/,/the sixteen divisions/p' "$T/sheet.txt" | sed 's/^/    /'
# Each worked by hand in STATUS.md.
says "1" " 1 General Requirements \$4,600.00 2 lines"
says "2" " 2 Site Construction \$583.20 1 line"
says "3" " 3 Concrete \$1,320.00 1 line"
says "4" " 4 Masonry \$3,420.00 1 line"
says "5" " 5 Metals \$420.00 1 line"
says "6" " 6 Wood and Plastics \$3,780.00 4 lines"
says "7" " 7 Thermal and Moisture Protection \$18,793.20 12 lines"
says "8" " 8 Doors and Windows \$4,700.00 3 lines"
says "9" " 9 Finishes \$9,867.50 9 lines"
says "10" "10 Specialties \$450.00 1 line"
says "11" "11 Equipment \$8,500.00 1 line"
says "12" "12 Furnishings \$760.00 1 line"
says "13" "13 Special Construction \$9,200.00 1 line"
says "14" "14 Conveying Systems \$14,500.00 1 line"
says "15" "15 Mechanical \$5,433.00 4 lines"
says "16" "16 Electrical \$1,200.00 3 lines"
says "the sixteen" "the sixteen divisions \$87,526.90"
says "the job cost is their sum" "Job cost \$87,526.90 46 lines"
# Overhead 10%: 8,752.69. Profit 8% of 96,279.59: 7,702.37. Price 103,981.96.
says "overhead" "Overhead at 10.00 % \$8,752.69"
says "profit" "Profit at 8.00 % \$7,702.37"
says "the price" "PRICE \$103,981.96"

echo
echo "== A line with no division is in the job cost and in none of the sixteen, and the estimate says so =="
"$P" line "$J" "$JOB" "Contingency" 1 --cost-type general >/dev/null
"$P" set "$J" "$JOB" unit_cost '$2,000' --line "Contingency" "${PRICE[@]}" >/dev/null
sheet
says "the job cost has it" "Job cost \$89,526.90 47 lines"
says "the sixteen do not" "the sixteen divisions \$87,526.90"
says "and it is said" "in no division: 1 line, \$2,000.00: in the job cost, and in none of the sixteen"
"$P" division "$J" "$JOB" 1 --line "Contingency" | sed 's/^/    /'
sheet
says "given its division" " 1 General Requirements \$6,600.00 3 lines"
says "nothing is left out" "the sixteen divisions \$89,526.90"
! grep -qF "in no division" "$T/sheet.txt" || { echo "FAIL: it still says a line is in no division"; exit 1; }

echo
echo "== The workbook: each division a formula over the lines; the CSV carries each line's division =="
"$P" export estimate "$J" "$T/estimate.xlsx" | sed "s|$T/||g; s/^/    /"
python3 tools/workbook.py "$T/estimate.xlsx" > "$T/worked.txt"
grep -F "formulas worked out; each comes to what is written with it" "$T/worked.txt" | sed 's/^/    /'
for row in "Divisions!1 General Requirements = 6600.00" "Divisions!7 Thermal and Moisture Protection = 18793.20" "Divisions!9 Finishes = 9867.50" "Divisions!16 Electrical = 1200.00" "Divisions!The sixteen divisions = 89526.90"; do
  grep -qF -- "$row" "$T/worked.txt" || { echo "FAIL: the workbook does not come to: $row"; cat "$T/worked.txt"; exit 1; }; echo "    $row"
done
"$P" export estimate "$J" "$T/estimate.csv" >/dev/null
head -1 "$T/estimate.csv" | grep -qF "Division" || { echo "FAIL: the CSV has no column for the division"; head -2 "$T/estimate.csv"; exit 1; }
grep -F "Stone veneer, 3 ft band" "$T/estimate.csv" | sed 's/^/    /'
grep -F "Stone veneer, 3 ft band" "$T/estimate.csv" | grep -q ',4 Masonry$' || { echo "FAIL: the stone veneer's row does not say Division 4"; exit 1; }

echo
echo "== What is refused, and the file is byte for byte what it was after each =="
refused "there are sixteen, 1 to 16" "$P" division "$J" "$JOB" 17 --line "Contingency"
refused "there are sixteen, 1 to 16" "$P" division "$J" "$JOB" 0 --line "Contingency"
refused "is not a division" "$P" division "$J" "$JOB" "seven and a half" --line "Contingency"
refused "is not a division" "$P" line "$J" "$JOB" "Something" 1 --division Roofing

echo
echo "PASS: a house estimated in all sixteen divisions, each to the figure worked by hand"
