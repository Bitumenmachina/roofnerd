#!/usr/bin/env bash
# An assembly changed for another writes the quantities of the one it becomes (call 199). At the command line, from an empty job: the
# first wall of a ring 20 ft by 16 ft (10 ft high, no opening, two outside corners), the starter assemblies. The plain lap-siding wall is
# changed for the wall over a stone wainscot and back; a quantity typed by hand stays; the reply names each line with what it was and
# what it is. The figures are worked by hand below and in STATUS.md ("An assembly changed for another: a demonstration").
# Run from the repository root:  demos/change-assembly.sh
# The job and the price are made up (no real job data) and live in a temporary folder. Every step fails the script if the program does not
# say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/wall.json"
K=(--status Known --source "made-up plan")
PLAIN="Exterior wall, 2x6, lap siding"
STONE="Exterior wall, 2x6, lap siding over a stone wainscot"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
sheet() { "$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"; }

echo "== The wall =="
"$P" new "$J" "Change (not a real job)" >/dev/null
"$P" level "$J" ground --elevation "0 ft" --height "10 ft" "${K[@]}" >/dev/null
"$P" type "$J" wall w6 --thickness "6 in" "${K[@]}" >/dev/null
"$P" wall "$J" level-0002 type-0003 --points "0,0 20,0 20,16 0,16" --closed --height "10 ft" "${K[@]}" >/dev/null
"$P" import assemblies "$J" starter >/dev/null
"$P" apply "$J" wall-0004 "$PLAIN" --source "made-up plan" >/dev/null
"$P" set "$J" wall-0004 unit_cost '$4.00' --line "Lap siding" "${K[@]}" >/dev/null
sheet
echo "-- 1. The plain wall: Lap siding 20 x 10 = 200.00 SF at \$4.00 = \$800.00; Corner boards 2 x 10 = 20.00 LF --"
says "$T/sheet.txt" "Lap siding 200.00 SF \$4.00/SF \$800.00"
says "$T/sheet.txt" "Corner boards 20.00 LF"

echo
echo "== 2. Changed for the wall over a stone wainscot =="
"$P" apply "$J" wall-0004 "$STONE" --source "made-up plan" --change > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
# Lap siding 200 - 20 x 3 = 140.00 SF, still at $4.00: $560.00. Hang the siding the same 140.00 SF. Corner boards 2 x (10 - 3) = 14.00 LF.
says "$T/said.txt" "has gained 1 cost line"
says "$T/said.txt" "Stone veneer wainscot, 3 ft"
says "$T/said.txt" "3 cost lines it already had now read as the new assembly's quantity"
says "$T/said.txt" "Lap siding: 200.00 SF to 140.00 SF"
says "$T/said.txt" "Hang the siding: 200.00 SF to 140.00 SF"
says "$T/said.txt" "Corner boards: 20.00 LF to 14.00 LF"
lacks "$T/said.txt" "left as entered"
sheet
# (Marked ~: the 3 ft the new assembly enters on those lines is Assumed, and what rests on it is flagged so.)
says "$T/sheet.txt" "Lap siding ~140.00 SF \$4.00/SF ~\$560.00"
says "$T/sheet.txt" "Corner boards ~14.00 LF"
"$P" show "$J" wall-0004 | tr -s ' ' > "$T/show.txt"
says "$T/show.txt" "quantity: Hang the siding 140 SF"

echo
echo "== 3. Changed back =="
"$P" apply "$J" wall-0004 "$PLAIN" --source "made-up plan" --change > "$T/back.txt"
sed 's/^/    | /' "$T/back.txt"
says "$T/back.txt" "Lap siding: 140.00 SF to 200.00 SF"
says "$T/back.txt" "Corner boards: 14.00 LF to 20.00 LF"
says "$T/back.txt" "came off"
says "$T/back.txt" "Stone veneer wainscot, 3 ft"
sheet
says "$T/sheet.txt" "Lap siding 200.00 SF \$4.00/SF \$800.00"
says "$T/sheet.txt" "Corner boards 20.00 LF"

echo
echo "== 4. A quantity typed by hand stays: net_area * 0.9 = 180.00 SF =="
"$P" set "$J" wall-0004 quantity "net_area * 0.9" --line "Lap siding" "${K[@]}" >/dev/null
sheet
says "$T/sheet.txt" "Lap siding 180.00 SF \$4.00/SF \$720.00"
"$P" apply "$J" wall-0004 "$STONE" --source "made-up plan" --change > "$T/entered.txt"
sed 's/^/    | /' "$T/entered.txt"
says "$T/entered.txt" "Lap siding: left as entered, 180.00 SF (net_area * 0.9); the new assembly writes net_area - net_area_below_wainscot + gable_area, wainscot = 3 ft, which comes to 140.00 SF"
says "$T/entered.txt" "Corner boards: 20.00 LF to 14.00 LF"
sheet
says "$T/sheet.txt" "Lap siding 180.00 SF \$4.00/SF \$720.00"
says "$T/sheet.txt" "Corner boards ~14.00 LF"

echo
echo "== 5. Nothing else about changing an assembly moves =="
# A line the new assembly does not write differently is not named; the price, the waste and the order stay (the tests, crates/world/tests/call199.rs).
lacks "$T/said.txt" "Studs"
lacks "$T/said.txt" "Weather barrier"
"$P" check "$J" >/dev/null
printf '    the job checks\n'
echo
echo "change-assembly: all of it holds."
