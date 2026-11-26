#!/usr/bin/env bash
# An item carries its cost (call 196, the bench's step B). At the command line, from an empty job: the starter assemblies and the starter tool
# chest, "Water heater, 50 gal" set down twice. The item is "priced as" an assembly, the things set down follow, the price list prices it from
# two suppliers, and the chest written out is a priced table that comes back to the byte. The figures are worked by hand below and in STATUS.md
# ("The bench, step B: an item carries its cost. A demonstration").
# Run from the repository root:  demos/itemcost.sh
# The job and every price are made up (no real job data) and live in a temporary folder. Every step fails the script if the program does not
# say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/heaters.json"
K=(--status Known --source "made-up plan")
HEATER="Water heater, 50 gal"
BOUGHT="Bought and set"
# An item is its own entry in the price list (call 196): no material is made for it, and its price is kept under its own name.
LISTED="$HEATER"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
sheet() { "$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"; }
cost() { "$P" price "$J" | tr -s ' ' > "$T/price.txt"; }

echo "== An empty job, the starter assemblies and the starter tool chest =="
"$P" new "$J" "Heaters (not a real job)" >/dev/null
"$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" import assemblies "$J" starter >/dev/null
"$P" import items "$J" starter >/dev/null

echo
echo "== 1. Set down twice, nothing more done: no line, and said to be set down and not priced =="
"$P" place "$J" level-0002 "$HEATER" --at "10,5" "${K[@]}" > "$T/placed.txt"
"$P" place "$J" level-0002 "$HEATER" --at "14,5" "${K[@]}" >> "$T/placed.txt"
lacks "$T/placed.txt" "priced as"
sheet
says "$T/sheet.txt" "Nothing in this job has a cost line."
says "$T/sheet.txt" "nothing is priced on 2 things that could be (2 pieces of equipment)"
lacks "$T/sheet.txt" '$0.00'
cost
says "$T/price.txt" "not priced at all: 2 things that could be (2 pieces of equipment)"

echo
echo "== 2. Priced as \"$BOUGHT\": the 2 set down follow; four lines have no figure =="
"$P" set "$J" "$HEATER" priced_as "$BOUGHT" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "2 set down from it follow"
sheet
# Each has the item's line (Material, 1 EA, no price) and "Set in place" (Labor), both in the item's trade, Division 15.
says "$T/sheet.txt" "Water heater, 50 gal 1.00 EA no price no figure"
says "$T/sheet.txt" "Set in place no figure no price"
says "$T/sheet.txt" "the sixteen divisions none leaves out 4 lines, named above"
says "$T/sheet.txt" "it is priced from the price list under \"$LISTED\", and the price list has no price for that item yet"
"$P" items "$J" | grep -F "$HEATER" > "$T/items.txt"
says "$T/items.txt" "2 placed; priced as \"$BOUGHT\""
"$P" place "$J" level-0002 "$HEATER" --at "18,5" "${K[@]}" | tr -s ' ' > "$T/third.txt"
says "$T/third.txt" "priced as \"$BOUGHT\""
"$P" remove "$J" "$(cut -d: -f1 "$T/third.txt")" >/dev/null

echo
echo "-- (first, a price for an item that is priced as nothing is set, and the program says it prices nothing yet) --"
"$P" new "$T/other.json" "Other (not a real job)" >/dev/null
"$P" import items "$T/other.json" starter >/dev/null
printf 'Material,Unit,Price,How sure,Where from\n"%s",EA,1850,Assumed,a made-up list\n' "$HEATER" > "$T/list0.csv"
"$P" import prices "$T/other.json" "$T/list0.csv" > "$T/said0.txt"
says "$T/said0.txt" "prices nothing on the job yet"
echo "== 3. The price list: \$1,850.00, and a second supplier's \$1,790.00: each heater \$1,850.00, side by side \$3,700.00 and \$3,580.00 =="
printf 'Material,Unit,Price,How sure,Where from\n"%s",EA,1850,Assumed,a made-up list\n' "$LISTED" > "$T/list.csv"
printf 'Material,Unit,Price,How sure,Where from\n"%s",EA,1790,Assumed,a made-up quote\n' "$LISTED" > "$T/quote.csv"
"$P" import prices "$J" "$T/list.csv" --add > "$T/said.txt"
says "$T/said.txt" "$LISTED: \$1,850.00/EA"
lacks "$T/said.txt" "prices nothing on the job yet"
"$P" import prices "$J" "$T/quote.csv" --supplier "Second Supply" > "$T/said.txt"
says "$T/said.txt" "\$1,790.00/EA"
sheet
says "$T/sheet.txt" "Water heater, 50 gal 1.00 EA ~\$1,850.00/EA ~\$1,850.00"
says "$T/sheet.txt" "total ~\$3,700.00 ~\$3,580.00"

echo
echo "== 4. Set in place: 4 hours each at \$85.00/hr = \$340.00 each; job cost 2 x (1,850.00 + 340.00) = \$4,380.00 =="
# call 233: the starter chest grew
for t in equipment-0102 equipment-0103; do
  "$P" set "$J" "$t" order "4 hr/EA" --line "Set in place" --status Assumed --source "made-up crew" >/dev/null
  "$P" set "$J" "$t" unit_cost '$85.00/hr' --line "Set in place" --status Assumed --source "made-up crew" >/dev/null
done
sheet
says "$T/sheet.txt" "Set in place ~4.0000 hr ~\$85.00/hr ~\$340.00"
says "$T/sheet.txt" "Job cost ~\$4,380.00 4 lines"

echo
echo "== 5. Priced as \"Mechanical equipment\": 2 follow, the lines are gone and one is there; the hours are named when they go =="
"$P" set "$J" "$HEATER" priced_as "Mechanical equipment" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "2 set down from it follow"
# call 233: the starter chest grew
says "$T/said.txt" "\"Set in place\" came off equipment-0102, with 4 hr/EA and \$85.00/hr entered on it"
sheet
says "$T/sheet.txt" "Equipment, set and connected"
says "$T/sheet.txt" "leaves out 2 lines, named above"
echo "-- Changed back: the price list still has the heater, so each material line is priced again; the hours are gone --"
"$P" set "$J" "$HEATER" priced_as "$BOUGHT" > "$T/said.txt"
says "$T/said.txt" "2 set down from it follow"
sheet
says "$T/sheet.txt" "Water heater, 50 gal 1.00 EA ~\$1,850.00/EA ~\$1,850.00"
says "$T/sheet.txt" "Set in place no figure no price"
echo "-- A wrong name is refused, and the job is as it was --"
if "$P" set "$J" "$HEATER" priced_as "No such assembly" > "$T/said.txt" 2>&1; then echo "FAIL: it took an assembly that is not there"; exit 1; fi
says "$T/said.txt" "no assembly \"No such assembly\" in this job"
if "$P" set "$J" "$HEATER" priced_as "Exterior wall, 2x6, lap siding" > "$T/said.txt" 2>&1; then echo "FAIL: it took a wall's assembly for equipment"; exit 1; fi
says "$T/said.txt" "goes on a wall"

echo
echo "== 6. The chest written out is priced; brought into a second empty job it comes back to the byte =="
"$P" export items "$J" "$T/chest.csv" >/dev/null
head -1 "$T/chest.csv" | tr ',' '\n' | tail -4 | paste -sd, > "$T/heads.txt"
says "$T/heads.txt" "Priced as,Unit cost,Unit cost: how sure,Unit cost: where from"
grep -F "\"$HEATER\"" "$T/chest.csv" | tail -c 120 > "$T/row.txt"
says "$T/row.txt" "$BOUGHT,\"\$1,850.00/EA\",Assumed,a made-up list"
J2="$T/second.json"
"$P" new "$J2" "Second (not a real job)" >/dev/null
"$P" import assemblies "$J2" starter >/dev/null
"$P" import items "$J2" "$T/chest.csv" > "$T/said.txt"
says "$T/said.txt" "1 price was set in the price list"
"$P" items "$J2" | grep -F "$HEATER" | tr -s ' ' > "$T/items.txt"
says "$T/items.txt" "priced as \"$BOUGHT\""
"$P" export prices "$J2" "$T/prices2.csv" >/dev/null
says "$T/prices2.csv" "\"$LISTED\",EA,1850"
"$P" export items "$J2" "$T/chest2.csv" >/dev/null
cmp "$T/chest.csv" "$T/chest2.csv" && echo "    the table written out again from the second job is the first, to the byte"

echo
echo "== 6b. A chest with none of the new columns is read as it was: nothing priced is moved =="
python3 -I - "$T/chest.csv" "$T/old.csv" <<'PY'
import csv, sys
rows = list(csv.reader(open(sys.argv[1], newline="")))
w = csv.writer(open(sys.argv[2], "w", newline=""))
for r in rows:
    w.writerow(r[:22])
PY
"$P" import items "$J" "$T/old.csv" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt" | head -5
# call 233: the starter chest grew
says "$T/said.txt" "the same as the job already has: 57"
"$P" items "$J" | grep -F "$HEATER" | tr -s ' ' > "$T/items.txt"
says "$T/items.txt" "priced as \"$BOUGHT\""
sheet
says "$T/sheet.txt" "Job cost ~\$3,700.00 2 lines"

echo
echo "== 7. A job saved before has no such entry, and opens as it was =="
"$P" set "$J" "$HEATER" priced_as unknown > "$T/said.txt"
says "$T/said.txt" "priced as nothing"
says "$T/said.txt" "came off"
lacks "$J" "priced_as"
"$P" check "$J" > "$T/check.txt" 2>&1 || { cat "$T/check.txt"; exit 1; }
sheet
says "$T/sheet.txt" "nothing is priced on 2 things that could be (2 pieces of equipment)"
"$P" set "$J" "$HEATER" priced_as "$BOUGHT" >/dev/null
grep -c '"priced_as": "Bought and set"' "$J" | grep -qx 1 && echo "    the saved job has one entry on one item: \"priced_as\": \"$BOUGHT\""
echo
echo "itemcost: every step said what it should."
