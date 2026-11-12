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
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/dress}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
line() { "$P" line "$J" "$1" "$2" 1 --cost-type material --division "$3" >/dev/null; "$P" set "$J" "$1" unit_cost "$4" --line "$2" --status Known --source "made-up price" >/dev/null; }

echo "== Three jobs: one cladding to a wall; the priced sample house; the same house unpriced =="
"$P" new "$J" "Dressed (not a real job)"
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
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

session() { # $1 job, $2 the folder of $OUT its pictures go in, $3 the steps
  mkdir -p "$OUT/$2"
  cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT/$2" ICD="$ICD"
. "$PWD/demos/window.sh"
turn() { xdotool mousemove 750 450; sleep 0.6; xdotool mousedown 3; sleep 0.6; for i in \$(seq 1 \$3); do xdotool mousemove_relative -- \$1 \$2; sleep 0.3; done; xdotool mouseup 3; sleep 5; }
nearer() { xdotool mousemove 750 450; sleep 0.6; for i in \$(seq 1 \$1); do xdotool click 4; sleep 0.5; done; sleep 5; }
open_window "$1"
$3
close_window
SESSION
  chmod +x "$T/session.sh"
  timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
  if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
  if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
}

echo
echo "== One cladding to a wall: brick south, lap siding east; turned round, shingles over stone north, stucco west =="
session "$J" one-each '
shot                        # 01 as opened: the south wall in brick, the west in stucco
turn 0 -10 5
nearer 2
shot                        # 02 lower and nearer: the courses, the corner, the panel door, the windows
turn 26 0 20                # half way round
shot                        # 03 the north wall in shingles over a stone wainscot, the east in white lap siding
turn -26 0 20
click 560 560               # the south wall, picked
park
shot                        # 04 its Finish, in the panel: what it is dressed in and the line that says so
'

echo
echo "== The sample whole house, priced in all sixteen divisions: dressed in what is priced on it =="
session "$T/priced.json" priced '
shot                        # 01 as opened
turn 0 -10 5
nearer 2
shot                        # 02 lower and nearer: lap siding over a stone wainscot, double-hung windows, the entry door
turn 0 30 8
xdotool key f; sleep 5
shot                        # 03 from above: the seams run up each plane
'

echo
echo "== The same house with nothing priced: bare =="
session "$T/bare.json" bare '
shot                        # 01 as opened: no line names a finish, so none is drawn
'

for f in one-each/01 one-each/02 one-each/03 one-each/04 priced/01 priced/02 priced/03 bare/01; do
  [ "$OUT/$f-model.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $f from this run"; exit 1; }
done
echo "  8 pictures of the model saved under $OUT (and the sheet beside each)"

echo
echo "== What the pictures hold, measured: the walls are the colors of what is priced on them =="
python3 - "$OUT" <<'PY'
import sys
from PIL import Image
out = sys.argv[1]
def mean(name, box):
    im = Image.open(f"{out}/{name}-model.png").convert("RGB").crop(box)
    px = list(im.getdata())
    return tuple(round(sum(p[i] for p in px) / len(px)) for i in range(3))
ok = True
def check(what, got, test, want):
    global ok
    good = test(got)
    ok = ok and good
    print("    %-62s %s  %s" % (what, got, "as wanted" if good else "FAIL: wanted " + want))
# The made-up house as opened: its south wall, right of the door, is red brick; its
# west wall is stucco, a pale warm gray.
check("one-each: the south wall, right of its door", mean("one-each/01", (880, 560, 980, 640)), lambda c: c[0] - c[1] > 40 and c[0] - c[2] > 40, "red well over green and blue (brick)")
check("one-each: the west wall", mean("one-each/01", (480, 520, 600, 640)), lambda c: abs(c[0] - c[1]) < 25 and c[0] > 150, "pale and nearly gray (stucco)")
# The sample house: priced, its walls are cedar siding; unpriced, the same place is bare.
p, b = mean("priced/01", (700, 460, 760, 520)), mean("bare/01", (700, 460, 760, 520))
# This wall is out of the sun, lit by the sky alone since the house has had a sun and
# a sky (DECISIONS.md call 68): cedar there measures (132, 114, 98), red 34 over blue,
# and the bare wall (156, 158, 159), with no colour to it at all. Each test is set
# between the two and fails on the other's picture: the bare wall is not warm (red 3
# under blue, wanted more than 24 over) and the cedar is not colourless (34, wanted
# under 12). Under the lamps these were "more than 40" and "under 30, and brighter
# than 170"; a wall in shade is no longer that bright.
cedar = lambda c: c[0] - c[2] > 24
plain = lambda c: abs(c[0] - c[2]) < 12 and c[0] > 140
check("priced: a wall of the sample house", p, cedar, "red well over blue (cedar)")
check("bare: the same place with nothing priced", b, plain, "pale, no cladding")
# Proved against each other every run: a test that passed on the wrong picture would
# tell nothing.
check("the cedar test on the bare wall fails", b, lambda c: not cedar(c), "the bare wall not to pass for cedar")
check("the bare test on the cedar wall fails", p, lambda c: not plain(c), "the cedar wall not to pass for bare")
# Its roof: priced, dark standing seam; unpriced, the light gray of a bare plane.
p, b = mean("priced/01", (760, 290, 860, 330)), mean("bare/01", (760, 290, 860, 330))
check("priced: its roof", p, lambda c: c[0] < 120, "dark (standing seam)")
check("bare: its roof", b, lambda c: c[0] > 130, "the light gray of a bare plane")
sys.exit(0 if ok else 1)
PY
echo "  PASS: three jobs opened and pictured; the walls and roofs in the pictures are the colors of what is priced on them, and bare where nothing is"
echo "  The pictures are the evidence: read them."
