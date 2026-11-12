#!/usr/bin/env bash
# What the house is dressed in, at the command line (DECISIONS.md call 58): a small
# made-up house is read bare; then lines are priced on it the way an estimator prices
# them, and each wall, the roof, a window and the door are read as finished with what
# their own lines name, each with the line that says so.
# Run from the repository root:  demos/dress.sh
# The job and every price are made up (no real job data) and live in a temporary folder.
# Every step fails the script if the program does not say what the step expects.
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
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$OUT" || { echo "FAIL: $1: not said: $2"; echo "$OUT"; exit 1; }; printf '    %s\n' "$2"; }
line() { "$P" line "$J" "$1" "$2" 1 --cost-type material --division "$3" >/dev/null; "$P" set "$J" "$1" unit_cost "$4" --line "$2" --status Known --source "made-up price" >/dev/null; }

echo "== The house: four walls round 24 ft by 16 ft, a partition, a door, two windows, a hip roof =="
"$P" new "$J" "Dressed (not a real job)"
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${K[@]}")")"
# Drawn counter-clockwise from the south-west corner: south, east, north, west.
WALLS="$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}")"
read -r SOUTH EAST NORTH WEST <<<"$(grep -oE 'wall-[0-9]+' <<<"$WALLS" | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
DOOR="$(made "$("$P" door "$J" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${K[@]}")")"
WIN1="$(made "$("$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "2.5 ft" "${K[@]}")")"
WIN2="$(made "$("$P" window "$J" "$EAST" "$NT" --offset "8 ft" --sill "2.5 ft" "${K[@]}")")"
"$P" roof "$J" "Main roof" --points "0,0 24,0 24,16 0,16" --pitch 8:12 --eave-height "9 ft" "${K[@]}" >/dev/null
"$P" check "$J"
PLANES="$("$P" show "$J" | grep -oE 'plane-[0-9]+' | sort -u | tr '\n' ' ')"
read -r P1 P2 _ <<<"$PLANES"

echo
echo "== Nothing is priced: everything is bare, and says why =="
OUT="$("$P" finishes "$J")"; echo "$OUT" | sed 's/^/  /' | cut -c1-150
# The outside face of each wall is the one outside the outline: the south wall's is south.
says "the south wall" "($SOUTH): bare, on its south face: nothing priced on it names a cladding"
says "the east wall" "($EAST): bare, on its east face"
says "the north wall" "($NORTH): bare, on its north face"
says "the west wall" "($WEST): bare, on its west face"
says "the partition" "($PART): a partition: both its faces are inside"
says "a window" "($WIN1): a frame, one sash and one light: nothing priced on it says how it operates"
says "the door" "($DOOR): a frame and a plain leaf: nothing priced on it says how it is built"
says "a roof plane" "($P1): bare: nothing priced on it names a roofing"

echo
echo "== Lines priced on it, as an estimator writes them =="
line "$SOUTH" "Wall framing, 2x6 at 16 in" 6 '$4.20'
line "$SOUTH" "Brick veneer, red" 4 '$24'
line "$EAST" "Fiber cement lap siding, white" 7 '$9.50'
line "$NORTH" "Stone veneer wainscot, 3 ft" 4 '$95'
line "$NORTH" "Cedar shingle siding" 7 '$14'
line "$WEST" "Stucco, three coat" 9 '$11'
line "$PART" "Brick veneer" 4 '$24'
for p in $PLANES; do line "$p" "Standing seam roofing, charcoal" 7 '$14'; done
line "$WIN1" "Window unit, clad wood, double-hung, 6 over 6, black" 8 '$1,150'
line "$DOOR" "Entry door, six panel, prehung" 8 '$2,400'
"$P" check "$J"
OUT="$("$P" finishes "$J")"; echo "$OUT" | sed 's/^/  /' | cut -c1-190
says "brick, and the line that says so" "($SOUTH): brick veneer, red, on its south face (the line \"Brick veneer, red\")"
says "lap siding" "($EAST): lap siding, white, on its east face (the line \"Fiber cement lap siding, white\")"
says "shingles over a stone wainscot" "($NORTH): shingles, cedar, on its north face (the line \"Cedar shingle siding\"); stone veneer wainscot to 3' 0\" (the line \"Stone veneer wainscot, 3 ft\")"
says "stucco" "($WEST): stucco, on its west face (the line \"Stucco, three coat\")"
# Brick priced on a partition is in the estimate and is not drawn: it has no outside face.
says "the partition" "($PART): a partition: both its faces are inside, so no cladding is drawn on it; priced on it and not drawn: brick veneer"
says "a roof plane" "($P1): standing seam, charcoal (the line \"Standing seam roofing, charcoal\")"
# 6 over 6: six lites in each sash, three across and two up.
says "the priced window" "($WIN1): double-hung, divided lites, 6 over 6, black"
says "the window nothing is priced on" "($WIN2): a frame, one sash and one light"
says "the door" "($DOOR): a panel door (the line \"Entry door, six panel, prehung\")"

echo
echo "== A second roofing priced on one plane: the first is drawn, and the second is said, not dropped =="
"$P" line "$J" "$P2" "Slate roofing" 1 --cost-type material --division 7 >/dev/null
OUT="$("$P" finishes "$J")"
says "the first is drawn, the second said" "($P2): standing seam, charcoal (the line \"Standing seam roofing, charcoal\"); also priced on it and not drawn: slate (the line \"Slate roofing\")"
cp "$J" "$T/kept.json"

echo
echo "== The reading is the job's: saved and read back, it is the same =="
BEFORE="$("$P" finishes "$J")"
"$P" check "$J"
[ "$BEFORE" = "$("$P" finishes "$J")" ] || { echo "FAIL: the reading changed"; exit 1; }
# And it is in the estimate: the brick is a Division 4 line and the siding a Division 7.
OUT="$("$P" estimate "$J" | tr -s ' ')"
says "the brick, priced" "Brick veneer, red"
says "the estimate by division" " 4 Masonry"
echo "  PASS: bare with nothing priced; then each wall, the roof, a window and the door finished with what their own lines name, each saying which line"
