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

echo
echo "PASS: a wall moved takes the ends that meet it, holds its ends on the walls it butts, keeps doors and windows in place, refuses what it cannot, and --alone moves it alone"
