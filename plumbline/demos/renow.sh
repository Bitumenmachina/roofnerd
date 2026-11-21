#!/usr/bin/env bash
# A renovation marked in the window (DECISIONS.md call 149; the command line's half is
# demos/reno.sh). A house as it stands (24 ft by 16 ft, a partition, one room "Living"),
# opened in the window on its plan: the partition picked and "To demolish" pressed, an
# outside wall picked and "Existing" pressed, the switch "Demolition" off and on, the job
# saved. What the window said, the saved job and the pictures are each checked.
# Run from the repository root:  demos/renow.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display, which
# needs Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job is made
# up (no real job data). Every step fails the script if the program does not do what it
# says. Worked by hand (as reno.sh step 4): 23.5 ft by 15.5 ft inside the faces is
# 364.25 SF, where the room was 178.25 SF with the partition standing.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/renow.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/renow.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"; APP="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/renow}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; head -60 "$1"; exit 1; }; printf '    %s\n' "$2"; }
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }

echo "== The house as it stands: 24 ft by 16 ft, a partition, one room (reno.sh step 1) =="
"$P" new "$J" "Renovation in the window (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
R="$(made "$("$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}")")"
# The south wall is brick, so that it carries a cost line (what the window then asks) and
# has a colour of its own to be washed pale when it is marked existing.
"$P" line "$J" "$SOUTH" "Brick veneer" net_area --cost-type material --division 4 >/dev/null
"$P" show "$J" > "$T/show.txt"; block "$R" > "$T/room.txt"
says "$T/room.txt" "floor area       178.25 SF"
cp "$J" "$T/before.json"

echo
echo "== The session: on the plan, the partition to demolish, the south wall existing, Demolition off and on =="
# At each shot the pointer is parked and nothing is selected (a click on bare ground lets
# go), so that what is measured is the wall and not its highlight.
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
sleep 8; park
shot                                     # 01 the house in 3D as opened: nothing marked
on "Plan"; sleep 8; park
echo "\$(plan_px 12 4) \$(plan_px 6 0) \$(plan_px 3 0) \$(plan_px 9 0) \$(plan_px 12 3) \$(plan_px 12 13) \$(plan_line)" > "$T/px.txt"
shot                                     # 02 the plan as opened: nothing marked
at 12 4; sleep 3                         # the partition
on "To demolish"; sleep 4; park
shot                                     # 03 the partition picked, its WORK row: "To demolish" lit
click 1000 800; sleep 3
shot                                     # 04 nothing picked: the partition a faint red ghost
at 6 0; sleep 3                          # the south wall
on "Existing"; sleep 4; park
shot                                     # 05 refused, the wall carries a line: two buttons in the row's place, the sentence on the card
on "Keep its lines"; sleep 4; park
shot                                     # 06 "Keep its lines" pressed: "Existing" lit, the line kept
click 1000 800; sleep 3
shot                                     # 07 nothing picked: the south wall washed pale, the ghost still there
on "Show Demolition"; sleep 4
click 1000 800; sleep 3
shot                                     # 08 Demolition off: the house as the work leaves it
on "Show Demolition"; sleep 4
click 1000 800; sleep 3
shot                                     # 09 Demolition on again: the ghost is back
on "3D"; sleep 8; park
shot                                     # 10 the same house in 3D: the south wall light, the ghost
xdotool key F2; sleep 3                  # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 07 08 09 10; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  20 pictures saved to $OUT (model and sheet, after each of 10 steps)"

echo
echo "== What the window said =="
cp "$T/map.txt.said" "$T/said.txt"
says "$T/said.txt" ": to demolish"
says "$T/said.txt" "Say which: Take its lines off, or Keep its lines"
says "$T/said.txt" ": existing to remain; its 1 cost line kept and priced, as work done to it"
says "$T/said.txt" "Demolition: switched off."
says "$T/said.txt" "Demolition: switched on."
if grep -qE -- '--clear|--keep' "$T/said.txt"; then echo "FAIL: the window gave the command line's words"; exit 1; fi

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
"$P" show "$J" > "$T/show.txt"
block "$PART" > "$T/part.txt"; block "$SOUTH" > "$T/south.txt"; block "$R" > "$T/room.txt"
says "$T/part.txt" "work             to demolish"
says "$T/south.txt" "work             existing to remain"
says "$T/south.txt" "Brick veneer"
says "$T/room.txt" "floor area       364.25 SF"

echo
echo "== The pictures, measured =="
# Boxes: the partition on the plan is worked from where the program says the plan lies on
# screen ($T/px.txt: the partition's two ends 3 and 13 ft up, and the plan's scale); the
# south wall in 3D is a patch of its brick face, the eye being where it was at the start.
# As measured on a run of these steps (mean colour of the box):
#   south wall in 3D: unmarked (01) (122, 85, 75), brick red; existing (10) (159, 162, 164)
#     (the brick washed toward pale gray, its pattern left off)
#   partition on the plan: as opened (02) (244, 242, 238), paper; ghost on (04) (234, 182, 182);
#     Demolition off (08) (248, 248, 248); on again (09) (233, 182, 183)
# Each test was run against a picture from before the step (01 for the wall's second
# figure; 02 for the partition's ghost) and each failed there.
python3 -I - "$OUT" "$T/px.txt" <<'PY'
import sys
from PIL import Image
out, pxfile = sys.argv[1], sys.argv[2]
v = [float(x) for x in open(pxfile).read().split()]
(qx0, qy0), (qx1, qy1), k = (v[8], v[9]), (v[10], v[11]), v[14]
def mean(name, box):
    im = Image.open(f"{out}/{name}-model.png").convert("RGB").crop(tuple(int(round(b)) for b in box))
    px = list(im.getdata())
    return tuple(round(sum(p[i] for p in px) / len(px)) for i in range(3))
# The partition between 3 ft and 13 ft up, on the plan.
part = (qx0 - k * 0.12, qy1, qx0 + k * 0.12, qy0)
# The south wall in 3D: a patch of its face as the house is first seen, the same patch
# once it is marked existing (the eye is where it was: the view comes home).
south3d = (800, 500, 1100, 650)
lum = lambda c: sum(c) / 3
ok = True
def report(what, a, b, test, want):
    global ok
    good = test(a, b)
    ok = ok and good
    print("    %-58s %s -> %s  %s" % (what, a, b, "as wanted" if good else "FAIL: wanted " + want))
report("south wall in 3D, unmarked (01), existing (10)", mean("01", south3d), mean("10", south3d), lambda a, b: lum(b) > lum(a) + 10, "lighter by over 10")
report("partition, ghost on (04) against Demolition off (08)", mean("04", part), mean("08", part), lambda a, b: sum(abs(x - y) for x, y in zip(a, b)) > 40, "a different place by over 40")
report("partition, ghost on again (09) as it was on (04)", mean("04", part), mean("09", part), lambda a, b: sum(abs(x - y) for x, y in zip(a, b)) < 15, "the same to within 15")
report("partition, as opened (02) against ghost (04): redder", mean("02", part), mean("04", part), lambda a, b: (b[0] - b[2]) > (a[0] - a[2]) + 5, "redder over blue by 5")
sys.exit(0 if ok else 1)
PY
echo
echo "  PASS: in the window, on the plan, the partition was marked to demolish and a south wall existing, each lit in its WORK row and said in words; Demolition off took the ghost away and on brought it back; the saved job has the partition to demolish and the room it leaves at 364.25 SF; the pictures show the existing wall lighter and the partition a red ghost"
