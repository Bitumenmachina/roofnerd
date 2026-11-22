#!/usr/bin/env bash
# A renovation: existing to remain, to demolish, new (call 148). From an empty job, a
# house as it stands, its walls marked existing; a partition taken down and the room
# growing into the two it was; the taking down priced; an outside wall taken down and an
# addition drawn, the room running into it; to the figures worked by hand.
# Run from the repository root:  demos/reno.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by
# hand in STATUS.md ("A renovation: existing, to demolish, new").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
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
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }

echo "== 1. The house as it stands: 24 ft by 16 ft, a partition, one room =="
"$P" new "$J" "Renovation (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
R="$(made "$("$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}")")"
"$P" show "$J" > "$T/show.txt"; block "$R" > "$T/room.txt"
says "$T/room.txt" "floor area       178.25 SF"

echo
echo "== 2. Its walls marked existing: what stands is not bought again =="
"$P" import assemblies "$J" starter >/dev/null
"$P" apply "$J" "$SOUTH" "Exterior wall, 2x6, lap siding" --source "made-up survey" >/dev/null
cp "$J" "$T/before.json"
refused "$P" mark "$J" "$SOUTH" "$EAST" "$NORTH" "$WEST" "$PART" existing "${K[@]}"
says "$T/out.txt" "and what stands is not bought again. Say which: take its lines off, or keep them priced as work done to it (paint, a patch, taking it down): --clear takes them off, --keep leaves them priced"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
"$P" mark "$J" "$SOUTH" "$EAST" "$NORTH" "$WEST" "$PART" existing --clear "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "$SOUTH: existing to remain; its"
says "$T/out.txt" "taken off: what stands is not bought again"
"$P" show "$J" > "$T/show.txt"; block "$PART" > "$T/part.txt"
says "$T/part.txt" "work             existing to remain"
"$P" estimate "$J" > "$T/est.txt"
lacks "$T/est.txt" "walls"

echo
echo "== 3. A new wall along the partition while it stands: refused =="
cp "$J" "$T/over.json"
refused "$P" wall "$T/over.json" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${N[@]}"
says "$T/out.txt" "drawn over another"

echo
echo "== 4. The partition to demolish: the room is the two it was; its taking down priced =="
"$P" mark "$J" "$PART" demolish "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "$PART: to demolish"
"$P" show "$J" > "$T/show.txt"; block "$R" > "$T/room.txt"
# 23.5 by 15.5 inside the faces.
says "$T/room.txt" "floor area       364.25 SF"
"$P" line "$J" "$PART" "Demolish partition, haul off" face_area --cost-type subcontract --division 2 >/dev/null
"$P" set "$J" "$PART" unit_cost '$2.50/SF' --line "Demolish partition, haul off" --status Known --source "made-up price" >/dev/null
"$P" estimate "$J" | tr -s ' ' > "$T/est.txt"
# 16 x 9 = 144 SF at $2.50.
says "$T/est.txt" "\$360.00"
cp "$J" "$T/over.json"
"$P" wall "$T/over.json" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${N[@]}" > "$T/out.txt"
says "$T/out.txt" "1 wall"

echo
echo "== 5. The addition: the east wall to demolish, three new walls; the room runs into it =="
"$P" mark "$J" "$EAST" demolish "${K[@]}" >/dev/null
read -r _ AEAST _ <<<"$("$P" wall "$J" "$S" "$WT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${N[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" show "$J" > "$T/show.txt"; block "$R" > "$T/room.txt"
# 35.5 by 15.5 inside the faces; Assumed, since the addition's walls are.
says "$T/room.txt" "~ floor area       550.25 SF"

echo
echo "== 6. A room is not built; saved and read back the same =="
cp "$J" "$T/before.json"
refused "$P" mark "$J" "$R" existing "${K[@]}"
says "$T/out.txt" "is not a thing that is built"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "its figures agree with what is drawn, and saving it back writes the same file"
echo
echo "== 7. A wall to demolish with boxes set against it (call 157): it says what still stands, and moves nothing =="
"$P" import items "$J" starter >/dev/null
"$P" cabinets "$J" "$S" "$AEAST" --in "$R" --from "1 ft" --boxes "B18 B18" "${N[@]}" >/dev/null
cp "$J" "$T/boxed.json"
"$P" mark "$J" "$AEAST" demolish "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "$AEAST: to demolish"
says "$T/out.txt" "2 things set against it still stand: B18 (furnishing-"
says "$T/out.txt" "move or remove them"
"$P" show "$J" > "$T/show.txt"
[ "$(grep -cE "^ *furnishing-[0-9]+  B18" "$T/show.txt")" = 2 ] || { echo "FAIL: the boxes were moved or removed for the estimator"; exit 1; }
echo "    both boxes still stand in the job"
cp "$J" "$T/before.json"
refused "$P" cabinets "$J" "$S" "$AEAST" --in "$R" --from "6 ft" --boxes "B18" "${N[@]}"
says "$T/out.txt" "is to demolish: boxes set against a wall that is coming down"
lacks "$T/out.txt" "no face on that room's side"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
cp "$T/boxed.json" "$T/removed.json"
"$P" remove "$T/removed.json" "$AEAST" > "$T/out.txt"
says "$T/out.txt" "2 things set against it still stand: B18 (furnishing-"
says "$T/out.txt" "move or remove them"
echo
echo "  PASS: a house as it stands was marked existing and not bought again, a partition taken down made one room of two and was priced by its face, a new wall was refused over a standing wall and taken over a demolished one, and an addition drawn past a demolished outside wall took the room with it, to the figures worked by hand"
