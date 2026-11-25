#!/usr/bin/env bash
# A price set where it is seen (call 207, points 4 and 5: the model's half): the price list as rows, and one change that sets or clears the
# estimator's price or a supplier's, for a material or an item by its name. At the command line, from an empty job: the starter assemblies and
# the starter tool chest, "Water heater, 50 gal" set down twice and priced as "Bought and set". The figures are worked by hand below and in
# STATUS.md ("A price typed where it is seen: a demonstration").
# Run from the repository root:  demos/pricelist.sh
# The job and every price are made up (no real job data, no real supplier) and live in a temporary folder. Every step fails the script if the
# program does not say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/heaters.json"
K=(--status Known --source "made-up plan")
A=(--status Assumed --source "made-up quote")
HEATER="Water heater, 50 gal"
SHEATH="Sheathing, 7/16 in (made up)"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { # refused <what it must say> <command...>: the command must fail, say that, and leave the job as it was
  local must="$1"; shift
  local before; before="$(cksum < "$J")"
  if "$@" > "$T/refused.txt" 2>&1; then echo "FAIL: it took what it should have refused: $*"; cat "$T/refused.txt"; exit 1; fi
  says "$T/refused.txt" "$must"
  [ "$(cksum < "$J")" = "$before" ] || { echo "FAIL: a refused change altered the job"; exit 1; }
}
sheet() { "$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"; }
list() { "$P" price "$J" list | tr -s ' ' > "$T/list.txt"; }

build() { # build <file>: an empty job, the starter assemblies and chest, two heaters priced as "Bought and set"
  local f="$1"
  "$P" new "$f" "Heaters (not a real job)" >/dev/null
  "$P" level "$f" ground --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
  "$P" import assemblies "$f" starter >/dev/null
  "$P" import items "$f" starter >/dev/null
  "$P" place "$f" level-0002 "Water heater, 50 gal" --at "10,5" "${K[@]}" >/dev/null
  "$P" place "$f" level-0002 "Water heater, 50 gal" --at "14,5" "${K[@]}" >/dev/null
  "$P" set "$f" "Water heater, 50 gal" priced_as "Bought and set" >/dev/null
}

echo "== 1. Two heaters set down, priced as \"Bought and set\": the heater heads the price list, no price, 2 lines waiting on 2 things, no dollars =="
build "$J"
list
sed 's/^/    | /' "$T/list.txt"
says "$T/list.txt" "$HEATER [Item, Mechanical]"
says "$T/list.txt" "estimator ? no price (no price entered)"
says "$T/list.txt" "lines 2 lines on 2 things; 2 with no figure"
lacks "$T/list.txt" "riding"
lacks "$T/list.txt" '$0.00'

echo
echo "== 2. \$1,850.00: 2 lines on 2 things priced; the job cost from no figure to 2 x 1,850.00 = \$3,700.00 (the labor lines still wait) =="
"$P" set "$J" "$HEATER" price '$1,850.00' "${A[@]}" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "prices 2 lines on 2 things"
says "$T/said.txt" 'the job cost was no figure and is now $3,700.00 (2 lines left out of it)'
list
says "$T/list.txt" '~ $1,850.00/EA (made-up quote) ~$3,700.00 riding'
lacks "$T/list.txt" "no price"

echo
echo "== 3. A supplier new to the job at \$1,790.00: said to be new; side by side its column is 2 x 1,790.00 = \$3,580.00 =="
"$P" set "$J" "$HEATER" price '$1,790.00' --supplier "Northside Supply" "${A[@]}" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "Northside Supply is new to this job and is added as a supplier"
says "$T/said.txt" 'at Northside Supply'"'"'s price those lines come to $3,580.00'
list
says "$T/list.txt" '~ $1,790.00/EA (made-up quote) ~$3,580.00 riding'
says "$T/list.txt" '~ $1,850.00/EA (made-up quote) ~$3,700.00 riding'
sheet
says "$T/sheet.txt" 'total ~$3,700.00 ~$3,580.00'

echo
echo "== 3b. The same price again: nothing changed, and the file is the same =="
before="$(cksum < "$J")"
"$P" set "$J" "$HEATER" price '$1,850.00' "${A[@]}" > "$T/said.txt"
says "$T/said.txt" "nothing changed"
[ "$(cksum < "$J")" = "$before" ] || { echo "FAIL: the same price again changed the job"; exit 1; }

echo
echo "== 4. The estimator's price cleared: no price, never \$0.00; the lines read no price; set again =="
"$P" set "$J" "$HEATER" price unknown --status Unknown --source "cleared by the estimator" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "becomes no price"
lacks "$T/said.txt" '$0.00 entered'
list
says "$T/list.txt" "estimator ? no price"
sheet
says "$T/sheet.txt" "Water heater, 50 gal 1.00 EA no price no figure"
lacks "$T/sheet.txt" '$0.00'
"$P" set "$J" "$HEATER" price '$1,850.00' "${A[@]}" > "$T/said.txt"
says "$T/said.txt" 'is now $3,700.00 (2 lines left out of it)'

echo
echo "== 5. \$0.00 typed: taken, marked on the reply, the row and the sheet =="
"$P" set "$J" "$HEATER" price '$0.00' "${A[@]}" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" 'that price is $0.00: it is taken, as a thing can be free'
list
says "$T/list.txt" '! $0.00 entered: nothing is charged for it'
sheet
grep -F "Water heater, 50 gal" "$T/sheet.txt" | head -3 | sed 's/^/    | /'
says "$T/sheet.txt" 'its price is $0.00 as entered: nothing is charged for it'
"$P" set "$J" "$HEATER" price '$1,850.00' "${A[@]}" >/dev/null

echo
echo "== 6. Refused, and the job as it was: a name that is neither, a price that is no figure, one below nothing, another unit =="
refused "neither a material nor an item of this job" "$P" set "$J" "Water heater, 40 gal" price 900 "${A[@]}"
says "$T/refused.txt" "$HEATER"
refused 'not a figure' "$P" set "$J" "$HEATER" price abc "${A[@]}"
refused 'less than nothing' "$P" set "$J" "$HEATER" price -5 "${A[@]}"
refused 'priced by the SF' "$P" set "$J" "$HEATER" price '900/SF' "${A[@]}"

echo
echo "== 7. A material a line of a wall is made of, by the same change: 20 LF x 9 ft = 180 SF, 6 sheets of 32 SF whole, at \$18.50 = \$111.00 =="
echo "   (the starter library makes no material, so the script makes one: 3,700.00 + 111.00 = \$3,811.00; at \$19.00, 6 x 19.00 = \$114.00, \$3,814.00)"
WT="$("$P" type "$J" wall w6 --thickness "6 in" "${K[@]}" | sed 's/^\(type-[0-9]*\):.*/\1/')"
WALL="$("$P" wall "$J" level-0002 "$WT" --points "0,0 20,0" --height "9 ft" "${K[@]}" | sed 's/^1 wall: //')"
MAT="$("$P" material "$J" "$SHEATH" sheet | sed 's/.*as \(material-[0-9]*\).*/\1/')"
"$P" set "$J" "$WALL" material "$SHEATH" "${A[@]}" >/dev/null
"$P" line "$J" "$WALL" "Sheathe the wall (sheets)" face_area --of-material >/dev/null
"$P" set "$J" "$MAT" order "32 SF/sheet" "${A[@]}" >/dev/null
list
says "$T/list.txt" "$SHEATH [Material]"
"$P" set "$J" "sheathing 7/16 in (made up)" price '$18.50' "${A[@]}" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "prices 1 line on 1 thing"
says "$T/said.txt" 'the job cost was $3,700.00 (3 lines left out of it) and is now $3,811.00 (2 lines left out of it)'
"$P" set "$J" "$SHEATH" price '$19.00' "${A[@]}" > "$T/said.txt"
says "$T/said.txt" '$18.50/sheet becomes $19.00/sheet'
says "$T/said.txt" 'is now $3,814.00'
list
says "$T/list.txt" '~ $19.00/sheet (made-up quote) ~$114.00 riding'

echo
echo "== 8. A price for a material nothing is made of is taken, and says it prices nothing yet =="
"$P" material "$J" "Zinc sheet (made up)" sheet >/dev/null
"$P" set "$J" "Zinc sheet (made up)" price '$41.00' "${A[@]}" > "$T/said.txt"
says "$T/said.txt" "prices nothing on the job yet: no line is made of it"
list
says "$T/list.txt" "Zinc sheet (made up) [Material]"
says "$T/list.txt" "lines no line of the job is made of it"

echo
echo "== 9. Four suppliers: the model has no limit of three; the list and the sheet give all four =="
n=0
for who in "Eastside Supply" "Southside Supply" "Westside Supply"; do
  n=$((n + 1))
  "$P" set "$J" "$HEATER" price "\$$((1800 - 10 * n)).00" --supplier "$who" "${A[@]}" > "$T/said.txt"
  says "$T/said.txt" "$who is new to this job"
done
list
for who in "Northside Supply" "Eastside Supply" "Southside Supply" "Westside Supply"; do says "$T/list.txt" "$who ~ \$"; done
sheet
says "$T/sheet.txt" "SUPPLIERS, SIDE BY SIDE"
for who in Northside Eastside Southside Westside; do says "$T/sheet.txt" "$who Supply"; done

echo
echo "== 10. The item renamed: its price is still its, under the new name =="
"$P" rename "$J" "$HEATER" "Tank heater, 50 gal" >/dev/null
list
says "$T/list.txt" "Tank heater, 50 gal [Item, Mechanical]"
lacks "$T/list.txt" "Water heater, 50 gal [Item"
says "$T/list.txt" '~ $1,850.00/EA (made-up quote) ~$3,700.00 riding'
HEATER="Tank heater, 50 gal"

echo
echo "== 11. A price typed, then a price list with the same name: the list wins, and says what it changed =="
printf 'Material,Unit,Price,How sure,Where from\n"%s",EA,1900,Assumed,a made-up list\n' "$HEATER" > "$T/list.csv"
"$P" import prices "$J" "$T/list.csv" > "$T/said.txt"
says "$T/said.txt" "$HEATER: \$1,850.00/EA becomes \$1,900.00/EA"
"$P" set "$J" "$HEATER" price '$1,850.00' "${A[@]}" > "$T/said.txt"
says "$T/said.txt" '$1,900.00/EA becomes $1,850.00/EA'

echo
echo "== 12. The price list written out, brought into a second job with the same things: the same, row for row =="
"$P" export prices "$J" "$T/out.csv" >/dev/null
J2="$T/second.json"
build "$J2"
"$P" rename "$J2" "Water heater, 50 gal" "$HEATER" >/dev/null
WT2="$("$P" type "$J2" wall w6 --thickness "6 in" "${K[@]}" | sed 's/^\(type-[0-9]*\):.*/\1/')"
WALL2="$("$P" wall "$J2" level-0002 "$WT2" --points "0,0 20,0" --height "9 ft" "${K[@]}" | sed 's/^1 wall: //')"
"$P" material "$J2" "$SHEATH" sheet >/dev/null
"$P" set "$J2" "$WALL2" material "$SHEATH" "${A[@]}" >/dev/null
"$P" line "$J2" "$WALL2" "Sheathe the wall (sheets)" face_area --of-material >/dev/null
"$P" material "$J2" "Zinc sheet (made up)" sheet >/dev/null
"$P" import prices "$J2" "$T/out.csv" > "$T/said.txt"
says "$T/said.txt" "not used: 0"
"$P" export prices "$J2" "$T/out2.csv" >/dev/null
cmp "$T/out.csv" "$T/out2.csv" && echo "    the two jobs' price lists are the same file, byte for byte"
"$P" price "$J" list > "$T/l1.txt"
"$P" price "$J2" list > "$T/l2.txt"
cmp "$T/l1.txt" "$T/l2.txt" && echo "    and the price list as rows reads the same in both"
echo
echo "== 13. A material given by its path goes by the same change; a supplier's first price on a material with none of the job's own keeps the older road =="
"$P" set "$J" "$MAT" price '$20.00' "${A[@]}" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "prices 1 line on 1 thing"
says "$T/said.txt" 'is now $3,820.00'
LEAD="$("$P" material "$J" "Lead sheet (made up)" sheet | sed 's/.*as \(material-[0-9]*\).*/\1/')"
"$P" set "$J" "$LEAD" quote '$30.00' --supplier "Northside Supply" "${A[@]}" > "$T/said.txt"
says "$T/said.txt" "set $LEAD quote from Northside Supply = \$30.00 (Assumed)"
echo
echo "pricelist: every step said what it should."
