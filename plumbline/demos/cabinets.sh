#!/usr/bin/env bash
# A kitchen from the tool chest (call 155). From an empty job, the house of demos/roomline.sh
# with its 12 ft addition and the rooms Living and Kitchen (a room line parts them at x = 24):
# the Kitchen's east wall runs 36,0 to 36,16 and its face on the Kitchen's side is at
# x = 35.75, clear from y = 0.25 to 15.75. A run of cabinet boxes is set along it by their
# codes: where each stands, the run's length and what is left to the corner, the countertop
# over the base boxes, wall boxes hung over them, one cost line on each box, no clash with
# the wall, and three refusals that leave the job as it was.
# Run from the repository root:  demos/cabinets.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by hand
# in STATUS.md ("A kitchen from the tool chest").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
N=(--status Assumed --source "made-up addition plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }

echo "== 1. The house after its addition, two rooms, and the tool chest =="
"$P" new "$J" "A kitchen (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" mark "$J" "$EAST" demolish "${K[@]}" >/dev/null
read -r ADD1 KEAST ADD3 <<<"$("$P" wall "$J" "$S" "$WT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${N[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
LIVING="$(made "$("$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}")")"
KITCHEN="$(made "$("$P" room "$J" "$S" "Kitchen" --at "30,8" "${K[@]}")")"
"$P" room-line "$J" "$S" --points "24,0 24,16" "${N[@]}" >/dev/null
IN="$("$P" import items "$J" starter)"
grep -qF "set: 46" <<<"$IN" || { echo "FAIL: the starter tool chest is not 46 items"; echo "$IN"; exit 1; }
echo "    the tool chest: 46 items, the 25 boxes among them"

echo
echo "== 2. A run along the Kitchen's east wall, from 1 ft: B18 SB36 B24 DB18 B36, with its countertop =="
# 18 + 36 + 24 + 18 + 36 = 132 in: 11 ft, from y = 1 to y = 12; the face is clear to y = 15.75:
# 3.75 ft left. Each box is 2 ft deep, so its middle is 1 ft off the face (x = 35.75): 34.75.
"$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "B18 SB36 B24 DB18 B36" --countertop "${N[@]}" | tee "$T/run.txt" | sed 's/^/    | /'
says "$T/run.txt" "set 5 boxes along $KEAST in $KITCHEN: a run of 11 ft, from 1 ft to 12 ft along the wall; 3.75 ft left to the corner"
says "$T/run.txt" "B18, 1' 6\" wide, middle at 34.75, 1.75"
says "$T/run.txt" "SB36, 3' 0\" wide, middle at 34.75, 4"
says "$T/run.txt" "B24, 2' 0\" wide, middle at 34.75, 6.5"
says "$T/run.txt" "DB18, 1' 6\" wide, middle at 34.75, 8.25"
says "$T/run.txt" "B36, 3' 0\" wide, middle at 34.75, 10.5"
# The countertop: 11 ft by 25 1/2 in (2.125 ft): 23.375 SF; its middle 1.0625 ft off the face.
says "$T/run.txt" "countertop, 11 ft long by 25 1/2 in deep, 34 1/2 in up, middle at 34.6875, 6.5; 23.375 SF, front edge 11 LF"
TOP="$(grep -oE '[a-z]+-[0-9]+: countertop' "$T/run.txt" | head -1 | sed 's/:.*//')"
"$P" line "$J" "$TOP" "Countertop, quartz" plan_area --cost-type material --division 6 >/dev/null
"$P" set "$J" "$TOP" unit_cost '$62.00/SF' --status Assumed --source "made-up price" --line "Countertop, quartz" >/dev/null
"$P" estimate "$J" > "$T/est.txt"
grep -F "Countertop, quartz" "$T/est.txt" | head -3 | sed 's/^/    | /'
says "$T/est.txt" "\$1,449.25"
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "its figures agree with what is drawn, and saving it back writes the same file"

echo
echo "== 3. Wall boxes on the same wall from 1 ft: W3030 W3630 W1830 =="
# 30 + 36 + 18 = 84 in: 7 ft, 1 ft deep, middles 0.5 ft off the face (35.25), 54 in up.
"$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "W3030 W3630 W1830" "${N[@]}" | tee "$T/wall.txt" | sed 's/^/    | /'
says "$T/wall.txt" "set 3 boxes along $KEAST in $KITCHEN: a run of 7 ft, from 1 ft to 8 ft along the wall; 7.75 ft left to the corner"
says "$T/wall.txt" "W3030, 2' 6\" wide, middle at 35.25, 2.25, hung 4' 6\" up"

echo
echo "== 4. Eight boxes by their codes, no price, in the supplier's list; no clash with the wall they back onto =="
"$P" estimate "$J" by-item > "$T/item.txt"
for code in B18 SB36 B24 DB18 B36 W3030 W3630 W1830; do
  grep -E "^  $code +Material +1 +1 EA +no price" "$T/item.txt" > /dev/null || { echo "FAIL: the estimate by item does not list $code once, 1 EA, with no price"; cat "$T/item.txt"; exit 1; }
  printf '    %s: 1 EA, no price\n' "$code"
done
"$P" export supplier "$J" "$T/supplier.csv" >/dev/null
for code in B18 SB36 B24 DB18 B36 W3030 W3630 W1830; do says "$T/supplier.csv" "$code," ; done
"$P" clashes "$J" > "$T/clash.txt"
for thing in $(grep -oE '[a-z]+-[0-9]+' "$T/run.txt" | sort -u); do
  if grep -E "$thing.*$KEAST|$KEAST.*$thing" "$T/clash.txt" | grep -qi "clash"; then echo "FAIL: $thing clashes with the wall it backs onto"; cat "$T/clash.txt"; exit 1; fi
done
lacks "$T/clash.txt" "B18"
lacks "$T/clash.txt" "Countertop"
"$P" clashes "$J" | head -3 | sed 's/^/    | /'
# The test is live: a box pushed 0.3 ft into the wall it backs onto does clash with it.
cp "$J" "$T/pushed.json"
FIRST="$(grep -oE 'furnishing-[0-9]+' "$T/run.txt" | head -1)"
"$P" move "$T/pushed.json" "$FIRST" --by "0.3,0" "${N[@]}" >/dev/null
"$P" clashes "$T/pushed.json" > "$T/pushed.txt"
grep -q "In the same space: 1" "$T/pushed.txt" || { echo "FAIL: a box in the wall is not found"; cat "$T/pushed.txt"; exit 1; }
echo "    (a box pushed 0.3 ft into its wall is found: the check is live)"

echo
echo "== 5. Refused, in words, the job unchanged =="
"$P" type "$J" door "door 3 x 7" --width "3 ft" --height "7 ft" "${K[@]}" > "$T/dt.txt"
DT="$(made "$(cat "$T/dt.txt")")"
cp "$J" "$T/before.json"
refused "$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "B99" "${N[@]}"
says "$T/out.txt" "B99 is not in the tool chest. It has: "
says "$T/out.txt" "B18"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
refused "$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "B36 B36 B36 B36 B36 B36" "${N[@]}"
says "$T/out.txt" "past the corner of"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
"$P" door "$J" "$KEAST" "$DT" --offset "5 ft" --sill "0 ft" "${K[@]}" >/dev/null
cp "$J" "$T/before.json"
refused "$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "B36 B36" "${N[@]}"
says "$T/out.txt" "a door is no place for boxes"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }

echo
echo "  PASS: a run of five boxes stood along the east wall to the hand figures (11 ft, 3.75 ft left), its countertop came to 23.375 SF and \$1,449.25, wall boxes hung at 54 in, eight boxes are on the estimate by item and the supplier's list with no price, none clashes with the wall, and three runs were refused with the job as it was"
