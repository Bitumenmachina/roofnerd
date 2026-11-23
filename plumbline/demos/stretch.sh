#!/usr/bin/env bash
# A wall moved, and the walls that meet it follow (call 170). From an empty job, a made-up
# house: the east wall moved 4 ft east takes the south and north walls' ends and the window
# keeps its place; the partition slid sideways keeps its ends on the walls it meets; pushed
# along its own length it is refused; the south wall moved takes the west and east walls'
# ends and the partition's; a move that would leave a window outside its wall is refused,
# naming it, and leaves the file as it was; `--alone` moves nothing but the wall; what stands
# against a moved wall is told and not moved; what is moved is no surer than the move, and a
# wall marked existing that follows is named.
# Run from the repository root:  demos/stretch.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by hand
# in STATUS.md ("A wall moved, and the walls that meet it follow").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
unchanged() { cmp -s "$1" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }; }
# A thing's block of `show`, in a file; and the total plan length of every wall.
# (from its own line to the next line indented no deeper: a wall's block holds its door and window)
block() { "$P" show "$1" | awk -v id="$2" 'inb { match($0, /^ */); if (RLENGTH <= ind) exit } !inb && $1 == id { inb = 1; match($0, /^ */); ind = RLENGTH } inb { print }' > "$T/block.txt"; }
walls_lf() { "$P" show "$1" | awk '/plan length/ { s += $(NF-1) } END { print s + 0 }'; }

echo "== 1. The house: 24 by 16 ft, 6 in walls 9 ft tall, a partition, a door, a window, two rooms =="
"$P" new "$J" "Stretch (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
# The door spans east 18 to 21 in the south wall (center 19.5 from its first end); the window
# spans east 18 down to 15 in the north wall, whose first end is at 24,16 (center 7.5 from it).
"$P" door "$J" "$SOUTH" "$DT" --offset "19.5 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" window "$J" "$NORTH" "$NT" --offset "7.5 ft" --sill "3 ft" "${K[@]}" >/dev/null
WEST_ROOM="$(made "$("$P" room "$J" "$S" West --at "6,8" "${K[@]}")")"
EAST_ROOM="$(made "$("$P" room "$J" "$S" East --at "18,8" "${K[@]}")")"
cp "$J" "$T/start.json"
"$P" show "$J" > "$T/show.txt"
block "$J" "$WEST_ROOM"; says "$T/block.txt" "floor area       178.25 SF"
block "$J" "$EAST_ROOM"; says "$T/block.txt" "floor area       178.25 SF"
[ "$(walls_lf "$J")" = 96 ] || { echo "FAIL: the walls are not 96 LF"; exit 1; }
echo "    (walls 96 LF)"

echo
echo "== 2. The east wall moved 4 ft east: the south and north walls follow at their ends =="
"$P" move "$J" "$EAST" --by "4,0" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "wall 1 followed: 4 ft longer"
says "$T/out.txt" "wall 3 followed: 4 ft longer"
lacks "$T/out.txt" "wall 4 followed"
lacks "$T/out.txt" "wall 5 followed"
block "$J" "$SOUTH"; says "$T/block.txt" "wall line        (0, 0) to (28, 0) ft"; says "$T/block.txt" "plan length      28 LF"
block "$J" "$NORTH"; says "$T/block.txt" "wall line        (28, 16) to (0, 16) ft"; says "$T/block.txt" "plan length      28 LF"
# The window stays at east 18 to 15: its center is 16.5, now 11.5 along from the first end at 28 (10 ft to its near edge).
says "$T/block.txt" "center, along the wall 11.5 ft"
block "$J" "$EAST"; says "$T/block.txt" "wall line        (28, 0) to (28, 16) ft"
block "$J" "$WEST"; says "$T/block.txt" "wall line        (0, 16) to (0, 0) ft"
block "$J" "$PART"; says "$T/block.txt" "wall line        (12, 0) to (12, 16) ft"
block "$J" "$SOUTH"; says "$T/block.txt" "center, along the wall 19.5 ft"
block "$J" "$WEST_ROOM"; says "$T/block.txt" "floor area       178.25 SF"
block "$J" "$EAST_ROOM"; says "$T/block.txt" "floor area       240.25 SF"
[ "$(walls_lf "$J")" = 104 ] || { echo "FAIL: the walls are not 104 LF"; exit 1; }
echo "    (walls 104 LF)"

echo
echo "== 3. The partition moved 2 ft west: its ends stay on the south and north walls =="
"$P" move "$J" "$PART" --by "-2,0" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "wall 5 stayed on wall 1 and wall 3"
lacks "$T/out.txt" "followed"
block "$J" "$PART"; says "$T/block.txt" "wall line        (10, 0) to (10, 16) ft"; says "$T/block.txt" "plan length      16 LF"
block "$J" "$WEST_ROOM"; says "$T/block.txt" "floor area       147.25 SF"
block "$J" "$EAST_ROOM"; says "$T/block.txt" "floor area       271.25 SF"
block "$J" "$SOUTH"; says "$T/block.txt" "wall line        (0, 0) to (28, 0) ft"
block "$J" "$NORTH"; says "$T/block.txt" "wall line        (28, 16) to (0, 16) ft"
block "$J" "$EAST"; says "$T/block.txt" "wall line        (28, 0) to (28, 16) ft"
block "$J" "$WEST"; says "$T/block.txt" "wall line        (0, 16) to (0, 0) ft"

echo
echo "== 4. The partition moved 3 ft north, along itself: refused, both its ends are held =="
cp "$J" "$T/before.json"
refused "$P" move "$J" "$PART" --by "0,3" "${K[@]}"
says "$T/out.txt" "wall 5 does not move: both its ends are held by the walls they meet (wall 1 and wall 3)"
unchanged "$J"

echo
echo "== 5. The south wall moved 2 ft south: the west and east walls' ends follow, and the partition's square =="
"$P" move "$J" "$SOUTH" --by "0,-2" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "wall 2 followed: 2 ft longer"
says "$T/out.txt" "wall 4 followed: 2 ft longer"
says "$T/out.txt" "wall 5 followed: 2 ft longer"
block "$J" "$WEST"; says "$T/block.txt" "wall line        (0, 16) to (0, -2) ft"; says "$T/block.txt" "plan length      18 LF"
block "$J" "$EAST"; says "$T/block.txt" "wall line        (28, -2) to (28, 16) ft"; says "$T/block.txt" "plan length      18 LF"
block "$J" "$PART"; says "$T/block.txt" "wall line        (10, -2) to (10, 16) ft"; says "$T/block.txt" "plan length      18 LF"
block "$J" "$SOUTH"; says "$T/block.txt" "wall line        (0, -2) to (28, -2) ft"; says "$T/block.txt" "plan length      28 LF"
says "$T/block.txt" "center, along the wall 19.5 ft"
block "$J" "$WEST_ROOM"; says "$T/block.txt" "floor area       166.25 SF"
block "$J" "$EAST_ROOM"; says "$T/block.txt" "floor area       306.25 SF"
[ "$(walls_lf "$J")" = 110 ] || { echo "FAIL: the walls are not 110 LF"; exit 1; }
echo "    (walls 28 + 18 + 28 + 18 + 18 = 110 LF)"

echo
echo "== 6. The east wall moved 12 ft west: the window would lie outside the north wall; refused, naming it =="
cp "$J" "$T/before.json"
refused "$P" move "$J" "$EAST" --by "-12,0" "${K[@]}"
says "$T/out.txt" "window 1 [$(grep -oE 'window-[0-9]+' "$J" | head -1)] would no longer lie within wall 3"
says "$T/out.txt" "door 1 ["
unchanged "$J"
echo "    (the door in the south wall, 16 ft long after that move, is named with it)"

echo
echo "== 7. --alone: the east wall moved 1 ft east by itself; nothing follows =="
"$P" move "$J" "$EAST" --by "1,0" --alone "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "moved alone: nothing followed it"
block "$J" "$EAST"; says "$T/block.txt" "wall line        (29, -2) to (29, 16) ft"
block "$J" "$SOUTH"; says "$T/block.txt" "wall line        (0, -2) to (28, -2) ft"; says "$T/block.txt" "plan length      28 LF"
block "$J" "$NORTH"; says "$T/block.txt" "wall line        (28, 16) to (0, 16) ft"; says "$T/block.txt" "plan length      28 LF"

echo
echo "== 8. A box set against the east wall's inside face before the move: told, and not moved =="
cp "$T/start.json" "$T/box.json"
"$P" place "$T/box.json" "$S" furnishing "Box" --at "22.75,8" --offset "0 ft" --size "2,2,3" "${K[@]}" >/dev/null
"$P" move "$T/box.json" "$EAST" --by "4,0" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "1 thing stands against wall 2 and was not moved"
"$P" show "$T/box.json" | grep -q "(22.75, 8)" || { echo "FAIL: the box moved"; exit 1; }
echo "    (the box is still at 22.75, 8)"

echo
echo "== 9. Moved with Assumed, design: the walls moved say so; the rest stay Known; an existing wall that follows is named =="
cp "$T/start.json" "$T/design.json"
"$P" mark "$T/design.json" "$SOUTH" existing --keep "${K[@]}" >/dev/null
"$P" move "$T/design.json" "$EAST" --by "4,0" --status Assumed --source "design" > "$T/out.txt"
says "$T/out.txt" "wall 1 is marked existing: as moved, it is not as it stands."
for w in "$EAST" "$SOUTH" "$NORTH"; do
    block "$T/design.json" "$w"; says "$T/block.txt" "~ wall line"
done
grep -q '"source": "made-up survey; moved (design)"' "$T/design.json" || { echo "FAIL: the moved wall's line does not say it was moved"; exit 1; }
grep -q '"source": "made-up survey; moved with wall 2 (design)"' "$T/design.json" || { echo "FAIL: a wall that followed does not say it did"; exit 1; }
echo "    (the moved wall's line says: moved (design); each that followed: moved with wall 2 (design))"
for w in "$WEST" "$PART"; do
    block "$T/design.json" "$w"; lacks "$T/block.txt" "~ wall line"
done

echo
echo "== 10. Saved and read back the same =="
"$P" check "$J" >/dev/null
cp "$J" "$T/saved.json"
"$P" show "$J" > "$T/a.txt"; "$P" show "$T/saved.json" > "$T/b.txt"
cmp -s "$T/a.txt" "$T/b.txt" || { echo "FAIL: read back differently"; exit 1; }
echo "    (the job is read back as it was)"


# ---- Call 179, B: a wall moved takes what lies on its line --------------------------------
# The box: 24 ft (east) by 16 ft (north), 6 in walls, one level 9 ft, a floor on the walls'
# outline, strip footings under all four walls, a gable roof (6:12, eave 9 ft, ridge east-west,
# the east and west edges gables) on the outline, a room inside.  By hand:
#   floor 24 x 16 = 384 SF; footings 2 x 24 + 2 x 16 = 80 LF; room 23.5 x 15.5 = 364.25 SF;
#   roof plan 384 SF, surface 384 x sqrt(1 + (6/12)^2) = 384 x 1.118034 = 429.33 SF.
# East wall 4 ft east: floor 28 x 16 = 448; footings 2 x 28 + 2 x 16 = 88; room 27.5 x 15.5 = 426.25;
#   roof plan 448, surface 448 x 1.118034 = 500.88 SF (plan +64 SF).
# Priced at a made-up $10.00/SF floor, $20.00/LF footing, $3.00/SF wall face:
#   floor +64 x 10 = $640.00; footings +8 x 20 = $160.00; walls (north and south +4 LF x 9 ft = +72 SF) 72 x 3 = $216.00;
#   the price rises by 640 + 160 + 216 = $1,016.00 ($800.00 for floor and footings, as the call has it, and $216.00 of walls).
B="$T/boxjob.json"
box() {
    "$P" new "$B" "Box (not a real job)" >/dev/null
    BS="$(made "$("$P" level "$B" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
    BWT="$(made "$("$P" type "$B" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
    BFT="$(made "$("$P" type "$B" footing "footing 24 in" --width "24 in" "${K[@]}")")"
    BFL="$(made "$("$P" type "$B" floor "floor 10 in" --thickness "10 in" "${K[@]}")")"
    read -r BSOUTH BEAST BNORTH BWEST <<<"$("$P" wall "$B" "$BS" "$BWT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
    "$P" footing "$B" "$BS" "$BFT" --points "0,0 24,0 24,16 0,16" --closed --depth "12 in" "${K[@]}" >/dev/null
    BFLOOR="$(made "$("$P" floor "$B" "$BS" "$BFL" --walls "${K[@]}")")"
    "$P" roof "$B" "Main roof" --points "0,0 24,0 24,16 0,16" --pitch 6:12 --eave-height "9 ft" "${K[@]}" >/dev/null
    BROOF="$(grep -oE 'roof-[0-9]+' "$B" | head -1)"
    "$P" set "$B" "$BROOF" edge:2 gable "${K[@]}" >/dev/null
    "$P" set "$B" "$BROOF" edge:4 gable "${K[@]}" >/dev/null
    BROOM="$(made "$("$P" room "$B" "$BS" Inside --at "12,8" "${K[@]}")")"
}
footings_lf() { "$P" show "$1" | awk '/^ *footing-/ { inf = 1; next } /^ *[a-z]+-[0-9]+ / { inf = 0 } inf && /plan length/ { s += $(NF-1) } END { print s + 0 }'; }
no_caution() { "$P" check "$1" > "$T/check.txt"; if grep -q '^!' "$T/check.txt"; then echo "FAIL: check cautions"; cat "$T/check.txt"; exit 1; fi; echo "    (check: no caution)"; }
total() { "$P" estimate "$1" | grep -E 'column totals' | grep -oE '\$[0-9,]+\.[0-9]+' | tr -d '$,' | awk '{ s += $1 } END { printf "%.2f", s }'; }

echo
echo "== 11. The box: floor 384 SF, footings 80 LF, room 364.25 SF, roof 384 SF plan =="
box
block "$B" "$BFLOOR"; says "$T/block.txt" "plan area        384 SF"
[ "$(footings_lf "$B")" = 80 ] || { echo "FAIL: the footings are not 80 LF"; exit 1; }
echo "    (footings 80 LF)"
block "$B" "$BROOM"; says "$T/block.txt" "floor area       364.25 SF"
"$P" line "$B" "$BFLOOR" "Floor" plan_area --cost-type material >/dev/null
"$P" set "$B" "$BFLOOR" unit_cost '$10/SF' --line "Floor" "${K[@]}" >/dev/null
for f in $(grep -oE 'footing-[0-9]+' "$B" | sort -u); do
    "$P" line "$B" "$f" "Footing" plan_length --cost-type subcontract >/dev/null
    "$P" set "$B" "$f" unit_cost '$20/LF' --line "Footing" "${K[@]}" >/dev/null
done
for w in $BSOUTH $BEAST $BNORTH $BWEST; do
    "$P" line "$B" "$w" "Wall" face_area --cost-type subcontract >/dev/null
    "$P" set "$B" "$w" unit_cost '$3/SF' --line "Wall" "${K[@]}" >/dev/null
done
PRICE0="$(total "$B")"
# walls 60 + ... : 80 LF x 9 ft = 720 SF x 3 = 2160; floor 3840; footings 80 x 20 = 1600
[ "$PRICE0" = "7600.00" ] || { echo "FAIL: the price is $PRICE0, not 7600.00 (2,160 + 3,840 + 1,600)"; exit 1; }
echo "    (price \$7,600.00 = walls 2,160 + floor 3,840 + footings 1,600)"
no_caution "$B"
cp "$B" "$T/box-start.json"

echo
echo "== 12. The east wall 4 ft east takes the floor's edge, the footings, the roof's edge; the price rises =="
"$P" move "$B" "$BEAST" --by "4 ft east" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "moved 4 ft east"
lacks "$T/out.txt" "-4"
says "$T/out.txt" "wall 1 followed: 4 ft longer"
says "$T/out.txt" "wall 3 followed: 4 ft longer"
says "$T/out.txt" "floor 1 followed: its edge on wall 2 went with it, 384.00 SF now 448.00 SF"
says "$T/out.txt" "footing 1 followed: 4 ft longer"
says "$T/out.txt" "footing 2 followed: moved with it, the same length"
says "$T/out.txt" "footing 3 followed: 4 ft longer"
says "$T/out.txt" "Main roof followed: its edge over wall 2 went with it, plan area 384.00 SF now 448.00 SF, surface 429.33 SF now 500.88 SF"
block "$B" "$BFLOOR"; says "$T/block.txt" "plan area        448 SF"
[ "$(footings_lf "$B")" = 88 ] || { echo "FAIL: the footings are not 88 LF"; exit 1; }
echo "    (footings 88 LF)"
block "$B" "$BROOM"; says "$T/block.txt" "floor area       426.25 SF"
no_caution "$B"
PRICE1="$(total "$B")"
# floor 448 x 10 = 4480; footings 88 x 20 = 1760; walls 88 LF x 9 = 792 SF x 3 = 2376; total 8616 = 7600 + 640 + 160 + 216
[ "$PRICE1" = "8616.00" ] || { echo "FAIL: the price is $PRICE1, not 8616.00"; exit 1; }
echo "    (price \$7,600.00 to \$8,616.00: floor +\$640.00, footings +\$160.00, walls +\$216.00 = +\$1,016.00; it did not fall)"

echo
echo "== 13. Moved back 4 ft west: every figure is what it was =="
"$P" move "$B" "$BEAST" --by "4 ft west" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "moved 4 ft west"
says "$T/out.txt" "floor 1 followed: its edge on wall 2 went with it, 448.00 SF now 384.00 SF"
block "$B" "$BFLOOR"; says "$T/block.txt" "plan area        384 SF"
[ "$(footings_lf "$B")" = 80 ] || { echo "FAIL: the footings are not 80 LF"; exit 1; }
block "$B" "$BROOM"; says "$T/block.txt" "floor area       364.25 SF"
[ "$(total "$B")" = "7600.00" ] || { echo "FAIL: the price is not back to 7600.00"; exit 1; }
echo "    (floor 384, footings 80, room 364.25, price \$7,600.00)"

echo
echo "== 14. The same move with --alone: only the wall moves, and what it left is said; check cautions =="
cp "$T/box-start.json" "$B"
"$P" move "$B" "$BEAST" --by "4 ft east" --alone "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "moved alone: nothing followed it"
says "$T/out.txt" "it left where they were: wall 1, wall 3, footing 1, footing 2, footing 3, floor 1 and Main roof"
block "$B" "$BFLOOR"; says "$T/block.txt" "plan area        384 SF"
"$P" check "$B" > "$T/check.txt"
says "$T/check.txt" "wall 2 stands 4 ft off footing 2, which has no wall on it"
echo "    (the room's walls no longer close, so it has no outline to reach past the floor with: see the report)"

echo
echo "== 15. A second level over the first: the first level's east wall takes the second level's with it =="
B2="$T/box2.json"
cp "$T/start.json" "$T/ignore.json"
"$P" new "$B2" "Box two levels (not a real job)" >/dev/null
L1="$(made "$("$P" level "$B2" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
L2="$(made "$("$P" level "$B2" second --elevation "9 ft" --height "9 ft" "${K[@]}")")"
W2="$(made "$("$P" type "$B2" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r G1S G1E G1N G1W <<<"$("$P" wall "$B2" "$L1" "$W2" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
read -r G2S G2E G2N G2W <<<"$("$P" wall "$B2" "$L2" "$W2" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" move "$B2" "$G1E" --by "4 ft east" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "wall 2 on second went with it: it stands over wall 2 on ground"
says "$T/out.txt" "wall 1 on second followed: 4 ft longer"
says "$T/out.txt" "wall 3 on second followed: 4 ft longer"
block "$B2" "$G2E"; says "$T/block.txt" "wall line        (28, 0) to (28, 16) ft"
block "$B2" "$G1E"; says "$T/block.txt" "wall line        (28, 0) to (28, 16) ft"

echo
echo "== 16. Refused: 30 ft west (past the west wall); a partition at east 12 and 14 ft west =="
cp "$T/box-start.json" "$B"
cp "$B" "$T/before.json"
refused "$P" move "$B" "$BEAST" --by "30 ft west" "${K[@]}"
# By hand: the east wall's west face is at 24 - 0.25 = 23.75, the west wall's east face at 0 + 0.25: 23.5 ft = 23 ft 6 in.
says "$T/out.txt" "wall 2 would run into wall 4, a wall it does not meet"
says "$T/out.txt" "it can go 23 ft 6 in west"
unchanged "$B"
PART2="$("$P" wall "$B" "$BS" "$BWT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
cp "$B" "$T/before.json"
refused "$P" move "$B" "$BEAST" --by "14 ft west" "${K[@]}"
# By hand: the partition's east face is at 12.25, the east wall's west face at 23.75: 11.5 ft = 11 ft 6 in.
says "$T/out.txt" "wall 2 would run into wall 5, a wall it does not meet"
says "$T/out.txt" "it can go 11 ft 6 in west"
unchanged "$B"
"$P" move "$B" "$BEAST" --by "11 ft 6 in west" "${K[@]}" >/dev/null
echo "    (and 11 ft 6 in is taken)"

echo
echo "== 17. Lengths typed as lengths, directions as directions =="
cp "$T/box-start.json" "$B"
"$P" move "$B" "$BEAST" --by "3' 6\" east and 2 ft north" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "moved 3 ft 6 in east and 2 ft north"
cp "$T/box-start.json" "$B"
"$P" move "$B" "$BNORTH" --by "6 in south" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "moved 6 in south"
cp "$T/box-start.json" "$B"
"$P" move "$B" "$BNORTH" --by "0,-0.5" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "moved 6 in south"
refused "$P" move "$B" "$BNORTH" --by "4 ft east and 3 ft west" "${K[@]}"
refused "$P" move "$B" "$BNORTH" --by "4" "${K[@]}"
echo "    (a bare 4 is refused; \"0,-0.5\" is still feet)"

echo
echo "== 18. A room line that ends on the moved wall goes with it =="
cp "$T/box-start.json" "$B"
"$P" room-line "$B" "$BS" --points "8,8 24,8" "${K[@]}" >/dev/null
"$P" move "$B" "$BEAST" --by "4 ft east" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "room line 1 followed: its end on wall 2 went with it"
"$P" show "$B" | grep -q "(8, 8) to (28, 8) ft" || { echo "FAIL: the room line's end did not follow"; exit 1; }
echo
echo "PASS: a wall moved takes the ends that meet it, what lies on its line (floor, footing, roof, wall above, room line), holds its ends on the walls it butts, keeps doors and windows in place, refuses what it cannot, and --alone moves it alone"
