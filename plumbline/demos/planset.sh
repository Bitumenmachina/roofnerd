#!/usr/bin/env bash
# A plan set on paper (call 168). From an empty job, the house of the layouts demonstration:
# 6 in walls 9 ft tall round 0,0 to 24,16, Known; the south, north and west walls existing,
# the east wall to demolish; an addition of three new walls 24,0 36,0 36,16 24,16; a room
# line where the east wall stood; rooms Living (6,8) and Kitchen (30,8). The plan set is
# written and read back, to the figures worked by hand in STATUS.md ("A plan set on paper").
# Run from the repository root:  demos/planset.sh
# The job is made up (no real job data) and lives in a temporary folder.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
# One page of a --list, by its number.
page() { awk -v n="$2" '/^page [0-9]+ of /{on=($2==n)} on' "$1" > "$T/page$2.txt"; }

"$P" new "$J" "Plan set (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" mark "$J" "$EAST" demolish "${K[@]}" >/dev/null
"$P" mark "$J" "$SOUTH" "$NORTH" "$WEST" existing "${K[@]}" >/dev/null
"$P" wall "$J" "$S" "$WT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${K[@]}" >/dev/null
"$P" room-line "$J" "$S" --points "24,0 24,16" "${K[@]}" >/dev/null
"$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}" >/dev/null
"$P" room "$J" "$S" "Kitchen" --at "30,8" "${K[@]}" >/dev/null
cp "$J" "$T/before.json"

echo "== 1. Three pages, each ARCH A, at 1/4\" = 1'-0\" =="
"$P" plan-set "$J" "$S" "$T/set.pdf" --date 2026-11-22 --list > "$T/list.txt"
says "$T/list.txt" "3 pages, ARCH A paper (12 by 9 in), 1/4\" = 1'-0\", 2026-11-22"
for n in 1 2 3; do page "$T/list.txt" "$n"; done

echo
echo "== 2. EXISTING PLAN: 4 walls full weight, 4 dimensions, LIVING 364 SF, no Kitchen, no new wall =="
says "$T/page1.txt" "page 1 of 3: EXISTING PLAN"
says "$T/page1.txt" "walls: 4 (4 heavy, 0 light, 0 dashed)"
says "$T/page1.txt" "dimensions: 4"
[ "$(grep -cE "^    wall-[0-9]+  (24|16)'-0\"$" "$T/page1.txt")" = 4 ] || { echo "FAIL: the four dimensions"; cat "$T/page1.txt"; exit 1; }
says "$T/page1.txt" "LIVING 364 SF"
lacks "$T/page1.txt" "KITCHEN 182"
says "$T/page1.txt" "NOT LETTERED, NOTHING THAT STANDS CLOSES ROUND THEIR POINTS: KITCHEN."

echo
echo "== 3. DEMOLITION PLAN: 3 walls light, 1 dashed; one wall comes down, 16 LF =="
says "$T/page2.txt" "walls: 4 (0 heavy, 3 light, 1 dashed)"
says "$T/page2.txt" "1 WALL COMES DOWN, 16 LF"
says "$T/page2.txt" "LIVING 364 SF"

echo
echo "== 4. PROPOSED PLAN: 3 light, 3 heavy and filled; 6 dimensions; LIVING 368, KITCHEN 182; no east wall =="
says "$T/page3.txt" "walls: 6 (3 heavy, 3 light, 0 dashed)"
says "$T/page3.txt" "dimensions: 6"
grep -E "^    wall-[0-9]+  " "$T/page3.txt" | sed 's/.*  //' | tr '\n' ' ' > "$T/dims.txt"
says "$T/dims.txt" "24'-0\" 24'-0\" 16'-0\" 12'-0\" 16'-0\" 12'-0\""
says "$T/page3.txt" "LIVING 368 SF"
says "$T/page3.txt" "KITCHEN 182 SF"
lacks "$T/page3.txt" "$EAST"

echo
echo "== 5. Notes; the north wall's line Assumed reads VIF on every page it is on =="
says "$T/page1.txt" "DOOR SWINGS ARE NOT SET."
says "$T/page1.txt" "DIMENSIONS ARE TO THE WALLS' LINES AS DRAWN."
says "$T/page1.txt" "NO LENGTH CARRIES VIF"
# The same house with the north wall drawn Assumed (the other walls Known).
V="$T/vif-job.json"
"$P" new "$V" "Plan set (not a real job)" >/dev/null
VS="$(made "$("$P" level "$V" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
VT="$(made "$("$P" type "$V" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
VW=(--height "9 ft")
V1="$(made "$("$P" wall "$V" "$VS" "$VT" --points "0,0 24,0" "${VW[@]}" "${K[@]}")")"
V2="$(made "$("$P" wall "$V" "$VS" "$VT" --points "24,0 24,16" "${VW[@]}" "${K[@]}")")"
NORTH="$(made "$("$P" wall "$V" "$VS" "$VT" --points "24,16 0,16" "${VW[@]}" --status Assumed --source "made-up sketch")")"
V4="$(made "$("$P" wall "$V" "$VS" "$VT" --points "0,16 0,0" "${VW[@]}" "${K[@]}")")"
"$P" mark "$V" "$V2" demolish "${K[@]}" >/dev/null
"$P" mark "$V" "$V1" "$NORTH" "$V4" existing "${K[@]}" >/dev/null
"$P" wall "$V" "$VS" "$VT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${K[@]}" >/dev/null
"$P" room-line "$V" "$VS" --points "24,0 24,16" "${K[@]}" >/dev/null
"$P" room "$V" "$VS" "Living" --at "6,8" "${K[@]}" >/dev/null
"$P" room "$V" "$VS" "Kitchen" --at "30,8" "${K[@]}" >/dev/null
"$P" plan-set "$V" "$VS" "$T/vif.pdf" --date 2026-11-22 --list > "$T/vif.txt"
for n in 1 2 3; do page "$T/vif.txt" "$n"; says "$T/page$n.txt" "$NORTH  24'-0\" VIF"; says "$T/page$n.txt" "1 LENGTH IS TO VERIFY IN FIELD (VIF)."; done

echo
echo "== 6. A door 3 ft wide in the south wall: its gap on each page the wall is on, no swing =="
"$P" door "$J" "$SOUTH" "$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")" --offset "12 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" plan-set "$J" "$S" "$T/door.pdf" --date 2026-11-22 --list > "$T/door.txt"
for n in 1 2 3; do page "$T/door.txt" "$n"; says "$T/page$n.txt" "in $SOUTH: door, gap 3'-0\""; says "$T/page$n.txt" ", no swing"; done
cp "$T/before.json" "$J"

echo
echo "== 7. 1/2 in to the foot: ARCH C; ARCH A at that scale is refused with the scale that fits =="
"$P" plan-set "$J" "$S" "$T/big.pdf" --scale "1/2 in" --date 2026-11-22 --list > "$T/big.txt"
says "$T/big.txt" "ARCH C paper (24 by 18 in), 1/2\" = 1'-0\""
refused "$P" plan-set "$J" "$S" "$T/small.pdf" --scale "1/2 in" --paper "ARCH A"
says "$T/out.txt" "does not fit ARCH A at 1/2\" = 1'-0\""
says "$T/out.txt" "at 1/4\" = 1'-0\" it does"
[ ! -e "$T/small.pdf" ] || { echo "FAIL: a refused plan set wrote a file"; exit 1; }

echo
echo "== 8. A job with nothing marked: one page, FLOOR PLAN =="
"$P" new "$T/plain.json" "Plain (not a real job)" >/dev/null
PS="$(made "$("$P" level "$T/plain.json" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
PW="$(made "$("$P" type "$T/plain.json" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
"$P" wall "$T/plain.json" "$PS" "$PW" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" >/dev/null
"$P" plan-set "$T/plain.json" "$PS" "$T/plain.pdf" --date 2026-11-22 --list > "$T/plain.txt"
says "$T/plain.txt" "1 page, ARCH A paper"
says "$T/plain.txt" "page 1 of 1: FLOOR PLAN"
says "$T/plain.txt" "walls: 4 (4 heavy, 0 light, 0 dashed)"

echo
echo "== 9. The file is read back by the program's own reader: 12 by 9 in; the south wall's string is 6 in =="
"$P" page "$T/set.pdf" --page 3 --lines --longer-than 400 > "$T/read.txt"
says "$T/read.txt" "page 3 of 3: PDF, 864 by 648 points (12 by 9 in)"
says "$T/read.txt" ": 432.00 points (6.0000 in)"
"$P" plan-set "$J" "$S" "$T/again.pdf" --date 2026-11-22 >/dev/null
cmp -s "$T/set.pdf" "$T/again.pdf" || { echo "FAIL: the same job, scale, paper and day wrote other bytes"; exit 1; }
echo "    (the same bytes the second time)"

echo
echo "== 10. The job file is what it was =="
cmp -s "$J" "$T/before.json" || { echo "FAIL: the job changed"; exit 1; }
echo "    byte for byte"
echo
echo "PASS"
