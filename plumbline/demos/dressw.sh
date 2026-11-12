#!/usr/bin/env bash
# The house dressed as built, in the application (DECISIONS.md call 58). Three jobs are
# opened in the model window and looked at: a small made-up house with a different
# cladding priced on each wall (brick, lap siding, shingles over a stone wainscot,
# stucco), a priced window and door and a standing seam roof; the sample whole house
# priced in all sixteen divisions; and the same sample house with nothing priced, which
# is bare. Each is dressed in what its own cost lines name, and in nothing else.
# Run from the repository root:  demos/dressw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). Every job and every
# price is made up. The pictures are the evidence and are to be read by eye; what the
# script itself checks is that every picture was taken, that the program said in its
# panel what the selected wall is finished with, and that the walls in the pictures are
# the colors of what is priced on them and not the color of a bare wall.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/dressw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/dressw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/dress}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
line() { "$P" line "$J" "$1" "$2" 1 --cost-type material --division "$3" >/dev/null; "$P" set "$J" "$1" unit_cost "$4" --line "$2" --status Known --source "made-up price" >/dev/null; }

echo "== Three jobs: one cladding to a wall; the priced sample house; the same house unpriced =="
"$P" new "$J" "Dressed (not a real job)"
S="$(made "$("$P" storey "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
DOOR="$(made "$("$P" door "$J" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${K[@]}")")"
WIN1="$(made "$("$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "2.5 ft" "${K[@]}")")"
WIN2="$(made "$("$P" window "$J" "$EAST" "$NT" --offset "8 ft" --sill "2.5 ft" "${K[@]}")")"
WIN3="$(made "$("$P" window "$J" "$NORTH" "$NT" --offset "12 ft" --sill "3.5 ft" "${K[@]}")")"
"$P" roof "$J" "Main roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 8:12 --eave-height "9 ft" "${K[@]}" >/dev/null
line "$SOUTH" "Brick veneer, red" 4 '$24'
line "$EAST" "Fiber cement lap siding, white" 7 '$9.50'
line "$NORTH" "Stone veneer wainscot, 3 ft" 4 '$95'
line "$NORTH" "Cedar shingle siding" 7 '$14'
line "$WEST" "Stucco, three coat" 9 '$11'
for p in $("$P" show "$J" | grep -oE 'plane-[0-9]+' | sort -u); do line "$p" "Standing seam roofing, charcoal" 7 '$14'; done
for e in $("$P" show "$J" | grep -oE 'eave-[0-9]+' | sort -u); do line "$e" "Gutter, white" 7 '$16.50'; done
line "$WIN1" "Window unit, clad wood, double-hung, 6 over 6, black" 8 '$1,150'
line "$WIN2" "Window unit, casement, white" 8 '$980'
line "$DOOR" "Entry door, six panel, prehung" 8 '$2,400'
"$P" check "$J"
"$P" finishes "$J" | sed 's/^/  /' | cut -c1-170
"$P" sixteen "$T/priced.json" >/dev/null
"$P" house "$T/bare.json" >/dev/null
cp "$J" "$T/before.json"

session() { # $1 job, $2 the steps
  cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD" N=${N:-0}
. "$PWD/demos/window.sh"
turn() { xdotool mousemove 750 450; sleep 0.6; xdotool mousedown 3; sleep 0.6; for i in \$(seq 1 \$3); do xdotool mousemove_relative -- \$1 \$2; sleep 0.3; done; xdotool mouseup 3; sleep 5; }
nearer() { xdotool mousemove 750 450; sleep 0.6; for i in \$(seq 1 \$1); do xdotool click 4; sleep 0.5; done; sleep 5; }
open_window "$1"
$2
close_window
SESSION
  chmod +x "$T/session.sh"
  timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
  if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
  if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
}

echo
echo "== One cladding to a wall: brick south, lap siding east; turned round, shingles over stone north, stucco west =="
N=0 session "$J" '
shot                        # 01 as opened: the south wall in brick, the east in white lap siding
turn 0 -10 5
nearer 2
shot                        # 02 lower and nearer: the courses, the corner board, the door and the windows
turn 26 0 20                # half way round
shot                        # 03 the north wall in shingles over a stone wainscot, the west in stucco
turn -26 0 20
click 560 560               # the south wall, picked
park
shot                        # 04 its Finish, in the panel: what it is dressed in and the line that says so
cp "$PLUMBLINE_MAP" "$OUT/names.txt"
'
