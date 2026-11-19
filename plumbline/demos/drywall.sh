#!/usr/bin/env bash
# Drywall bought for both faces of a partition, and gypsum board with its waste in whole
# sheets (call 143): from an empty job, a house 24 ft by 16 ft with a partition and a
# door in it, the starter library's partition and exterior wall assemblies, and the
# board and paint to the figures worked by hand.
# Run from the repository root:  demos/drywall.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by
# hand in STATUS.md ("Drywall bought for both faces of a partition").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }

echo "== A house 24 ft by 16 ft, 9 ft walls, a partition with a door, a window in the south wall =="
"$P" new "$J" "Drywall (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${K[@]}")")"
read -r SOUTH _ _ _ <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" door "$J" "$PART" "$DT" --offset "8 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "3 ft" "${K[@]}" >/dev/null
"$P" import assemblies "$J" starter >/dev/null
"$P" apply "$J" "$PART" "Interior partition, 2x4" --source "made-up plan" >/dev/null
"$P" apply "$J" "$SOUTH" "Exterior wall, 2x6, lap siding" --source "made-up plan" >/dev/null
"$P" show "$J" > "$T/show.txt"

echo
echo "== The partition: both its faces =="
block "$PART" > "$T/part.txt"
# 16 x 9 = 144 SF, less the door's 21: 123 SF.
says "$T/part.txt" "net area         123 SF"
# Both faces: 246 SF; with 10 % waste 270.6 SF; / 32 = 8.46: 9 sheets.
says "$T/part.txt" "quantity: Gypsum board, both sides 246 SF"
says "$T/part.txt" "with waste: Gypsum board, both sides 270.6 SF"
says "$T/part.txt" "to order: Gypsum board, both sides 9 sheet"
says "$T/part.txt" "quantity: Paint, both sides 246 SF"
says "$T/part.txt" "~ waste: Gypsum board, both sides 10 %"

echo
echo "== The south wall: its inside face =="
block "$SOUTH" > "$T/south.txt"
# 24 x 9 = 216 SF, less the window's 15: 201 SF; with waste 221.1 SF; / 32 = 6.91: 7 sheets.
says "$T/south.txt" "net area         201 SF"
says "$T/south.txt" "with waste: Gypsum board, hung and finished 221.1 SF"
says "$T/south.txt" "to order: Gypsum board, hung and finished 7 sheet"
says "$T/south.txt" "quantity: Paint, two coats 201 SF"
echo
echo "== Two rooms: each one's walls at their faces, less its doors and windows (call 144) =="
WEST="$(made "$("$P" room "$J" "$S" "West room" --at "6,8" "${K[@]}")")"
EAST="$(made "$("$P" room "$J" "$S" "East room" --at "18,8" "${K[@]}")")"
"$P" show "$J" > "$T/show.txt"
block "$WEST" > "$T/west.txt"; block "$EAST" > "$T/east.txt"
# Faces 0.25 ft in: 11.5 by 15.5 inside, 178.25 SF, 54 LF round.
says "$T/west.txt" "floor area       178.25 SF"
says "$T/west.txt" "perimeter        54 LF"
# 54 x 9 = 486, less the door's 21: 465 SF. The east room loses the window's 15 as well.
says "$T/west.txt" "wall area        465 SF"
says "$T/east.txt" "wall area        450 SF"
echo
echo "  PASS: a partition's board and paint were bought for both its faces, and gypsum board on it and on an outside wall ordered in whole sheets with its waste, and each room's walls taken off at their faces less its doors and windows, to the figures worked by hand"
