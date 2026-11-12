#!/usr/bin/env bash
# Moving round the model, and the switches: what call 62 asked the window to feel like,
# shown by measurement. A made-up house is opened and its door, which is the one brown
# thing in the picture, is followed from picture to picture:
#   the roof is switched off and on;
#   the wheel closes on the door and the place under the pointer stays under it;
#   the wheel over a panel scrolls the panel and does not move the model;
#   a turn with the right button is about the door, which stays where it is on screen;
#   a drag with the left button slides the ground, and the door goes with the hand;
#   a double-click flies to the door;
#   a flick carries on after it is let go.
# Run from the repository root:  demos/glassw.sh [folder for the pictures]
#
# It drives the real program with a real pointer on a virtual display (Xvfb, xdotool
# and a Vulkan driver; Mesa's software one will do) and reads the pictures the program
# saves. The house is made up. The figures are worked by hand in the comments and in
# STATUS.md ("The window rebuilt, step 3").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/glassw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/glassw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/glass}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }

echo "== The house: 24 ft by 16 ft, a door and a window in its south wall, a hip roof =="
"$P" new "$J" "Moving round it (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${K[@]}")")"
read -r SOUTH _ <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" door "$J" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "2.5 ft" "${K[@]}" >/dev/null
"$P" roof "$J" "Main roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 8:12 --eave-height "9 ft" "${K[@]}" >/dev/null
"$P" check "$J" >/dev/null
BEFORE="$(sha256sum "$J")"

echo
echo "== The session =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
# A wheel notch, held as long as a frame; and a drag with a button, in steps a frame
# can see, held still at its end for \$6 seconds before it is let go (0: let go moving).
wheel() { local i; for i in \$(seq 1 "\$2"); do xdotool click "\$1"; sleep 2; done; sleep 4; }
drag() {
  xdotool mousemove "\$2" "\$3"; sleep 1.5; xdotool mousedown "\$1"; sleep 1.5
  local i
  for i in 1 2 3 4 5 6 7 8; do xdotool mousemove \$(( \$2 + (\$4 - \$2) * i / 8 )) \$(( \$3 + (\$5 - \$3) * i / 8 )); sleep 0.6; done
  if [ "\$6" != 0 ]; then sleep "\$6"; fi
  xdotool mouseup "\$1"; sleep 4
}
# A flick: the button goes down, the pointer goes \$3 dots to the left in a fifth of a
# second, and the button is let go while it is still going.
flick() {
  xdotool mousemove "\$1" "\$2"; sleep 1.5; xdotool mousedown 1; sleep 1.5
  local i
  for i in 1 2 3 4 5; do xdotool mousemove \$(( \$1 - \$3 * i / 5 )) "\$2"; sleep 0.04; done
  xdotool mouseup 1; sleep 6
}
open_window "$J"
park; shot                              # 01 as opened
on "Show Roof"; park; shot              # 02 the roof switched off
on "Show Roof"; park; shot              # 03 and on again
xdotool mousemove 805 640; sleep 1.5
wheel 4 3; park; shot                   # 04 three notches in, at the door
xdotool mousemove 1400 500; sleep 1.5
wheel 5 3; park; shot                   # 05 the wheel over the right panel: the model has not moved
drag 3 805 640 655 600 2; park; shot    # 06 a turn about the door
drag 1 805 640 1005 560 3; park; shot   # 07 the ground slid: the door went with the hand
xdotool mousemove 1005 560; sleep 1.5
xdotool mousedown 1; sleep 0.12; xdotool mouseup 1; sleep 0.12; xdotool mousedown 1; sleep 0.12; xdotool mouseup 1; sleep 6
park; shot                              # 08 a double-click on the door: flown to it
flick 800 480 100; park; shot           # 09 a flick to the left, let go moving
xdotool key F2; sleep 4
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
[ ! -e "$T/missing.txt" ] || { echo "FAIL: a control was not on screen"; cat "$T/missing.txt"; exit 1; }
for n in 01 02 03 04 05 06 07 08 09; do [ -s "$OUT/$n-model.png" ] || { echo "FAIL: no picture $n"; exit 1; }; done

echo
echo "== What the pictures show =="
python3 - "$OUT" <<'PY'
import sys
from PIL import Image
out = sys.argv[1]
pic = lambda n: Image.open(f"{out}/{n:02d}-model.png").convert("RGB")
def door(c):                     # the door's brown, (104, 80, 58) as lit; selected it is (116, 97, 100)
    r, g, b = c
    return (r > g > b and 85 < r < 125 and 35 <= r - b <= 62 and b < 76) or (105 < r < 128 and 88 < g < 106 and 90 < b < 110 and r > b)
def roof(c):                     # the bare roof's blue grey: (140, 150, 163)
    r, g, b = c
    return b > g > r and 125 < r < 155 and 150 < b < 178
def run(im, x, y):               # the stretch of door on a row that holds x: its two ends
    if not door(im.getpixel((x, y))): return None
    a = b = x
    while a > 0 and door(im.getpixel((a - 1, y))): a -= 1
    while b < im.width - 1 and door(im.getpixel((b + 1, y))): b += 1
    return a, b
def fail(why):
    print("FAIL:", why); sys.exit(1)

p = {n: pic(n) for n in range(1, 10)}
# The roof is there, is switched off, and is back.
at = (900, 330)
if not roof(p[1].getpixel(at)): fail(f"01: no roof at {at}: {p[1].getpixel(at)}")
if roof(p[2].getpixel(at)): fail("02: the roof is still drawn after its switch was put off")
if not roof(p[3].getpixel(at)): fail("03: the roof did not come back")
print("    the roof: there, switched off, and back")

# The wheel at the door. A line of the wheel is a tenth nearer. On this virtual display
# a notch arrives as two lines (PLUMBLINE_TRACE=1 prints what the wheel sent: "wheel:
# Line 2"), so three notches are six lines: 0.9 to the sixth is 0.531 of the way off,
# and the door is 1 / 0.531 = 1.88 times as wide. The place under the pointer,
# (805, 640), is as far along the door as it was.
r1, r4 = run(p[1], 805, 640), run(p[4], 805, 640)
if not r1: fail(f"01: the door is not at (805, 640): {p[1].getpixel((805, 640))}")
if not r4: fail(f"04: after the wheel the door is not under the pointer: {p[4].getpixel((805, 640))}")
w1, w4 = r1[1] - r1[0] + 1, r4[1] - r4[0] + 1
along1, along4 = (805 - r1[0]) / w1, (805 - r4[0]) / w4
print(f"    the wheel at the door: {w1} dots wide, then {w4} ({w4 / w1:.2f} times; 1.88 by hand); the pointer {along1:.2f} of the way along it, then {along4:.2f}")
if not 1.74 <= w4 / w1 <= 2.02: fail("the door did not grow by what six lines of the wheel give")
if abs(along4 - along1) > 0.08: fail("the place under the pointer did not stay under it")

# The wheel over a panel moved nothing on the model.
r5 = run(p[5], 805, 640)
# (An edge is steadied over several frames and can sit a dot either way.)
if not r5 or abs(r5[0] - r4[0]) > 2 or abs(r5[1] - r4[1]) > 2: fail(f"05: the model moved when the wheel was over a panel: {r4} then {r5}")
print(f"    the wheel over the right panel: the door is where it was ({r4} then {r5})")

# A turn about the door: the door is still under where the pointer went down.
# A full window height of drag is a full turn: 150 dots to the left of 960 is 56
# degrees, so the door is seen from well round to the other side and is narrower.
r6 = run(p[6], 805, 640)
if not r6: fail(f"06: after the turn the door is not where the turn began: {p[6].getpixel((805, 640))}")
w5, w6 = r5[1] - r5[0] + 1, r6[1] - r6[0] + 1
if abs(w6 - w5) < 0.12 * w5: fail(f"06: the turn turned nothing: the door is {w5} dots wide, then {w6}")
print(f"    a turn with the right button: the door is still at (805, 640), {w5} dots wide then {w6}")

# The ground slid with the hand: the place grabbed went from (805, 640) to (1005, 560).
if not door(p[7].getpixel((1005, 560))): fail(f"07: the door did not go with the hand to (1005, 560): {p[7].getpixel((1005, 560))}")
if door(p[7].getpixel((805, 640))): fail("07: the door is still where it was")
print("    a drag with the left button: the door went with the hand, (805, 640) to (1005, 560)")

# A double-click on it: it is flown to, and is in the middle of the window.
if not door(p[8].getpixel((800, 480))): fail(f"08: the door is not in the middle of the window after the double-click: {p[8].getpixel((800, 480))}")
r7, r8 = run(p[7], 1005, 560), run(p[8], 800, 480)
print(f"    a double-click on the door: it is selected, as a click selects, and in the middle of the window, {r8[1] - r8[0] + 1} dots wide where it was {r7[1] - r7[0] + 1}")
if (r8[1] - r8[0]) <= (r7[1] - r7[0]) * 1.3: fail("08: the view did not close on the door")

# A flick: the hand went 100 dots to the left and let go moving; the door went further.
# The place grabbed was (800, 480): where is it now? As far along the door as it was.
along8 = (800 - r8[0]) / (r8[1] - r8[0] + 1)
row = [x for x in range(p[9].width) if door(p[9].getpixel((x, 480)))]
if not row: fail("09: the door is not on the row it was flicked along")
a, b = row[0], row[-1]
now = a + along8 * (b - a + 1)
print(f"    a flick of 100 dots to the left: the place grabbed is now at {now:.0f}, {700 - now:.0f} dots past where the hand let go")
if now > 700 - 10: fail("09: the view stopped dead when it was let go")
PY
[ "$(sha256sum "$J")" = "$BEFORE" ] || { echo "FAIL: looking round the model changed the job"; exit 1; }
echo "    the job is unchanged: nothing was selected, moved or let go of by any of it"
echo
echo "PASS: the wheel closes on the pointer, a panel scrolls alone, a turn is about the pointer, the ground goes with the hand, a double-click flies, a flick carries on, and the roof has a switch"
