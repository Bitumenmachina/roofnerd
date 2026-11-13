#!/usr/bin/env bash
# The layers of the house a trade at a time, in the window: the finish taken off to the
# weather barrier and the flashing, the walls drawn as their frames, the ground taken
# away from the foundation, and one level looked down into. The made-up whole house,
# each layer switched with one click under Show or Levels, and each picture measured.
# Run from the repository root:  demos/tradesw.sh [folder for the pictures]
#
# It drives the real program with a real pointer on a virtual display (Xvfb, xdotool and
# a Vulkan driver; Mesa's software one will do). The job is made up. What each picture
# must show, and the measurement that fails on the picture before the switch, is in
# STATUS.md ("The layers, switched in the window").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/tradesw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/tradesw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/layers}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/house.json"

echo "== The made-up whole house, priced in all sixteen divisions =="
"$P" sixteen "$J" >/dev/null
"$P" estimate "$J" | grep -F "PRICE" > "$T/price-before.txt"
sed 's/^/    /' "$T/price-before.txt"

echo
echo "== The session: each layer with one click =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
park
shot                                    # 01 as it opens: dressed in what is priced
on "Show Finishes"; sleep 6; park
shot                                    # 02 the finish off: the weather barrier, and flashing at the openings
on "Show Finishes"; on "Show Framing"; sleep 6; park
shot                                    # 03 the walls as their frames
on "Show Framing"; on "Show Ground"; sleep 6; park
shot                                    # 04 the ground away: the foundation under the house
on "Show Ground"; on "Level first floor"; sleep 10; park
shot                                    # 05 the first floor alone, looked down into
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
[ ! -e "$T/missing.txt" ] || { echo "FAIL: a switch was not on screen"; cat "$T/missing.txt"; exit 1; }
for n in 01 02 03 04 05; do [ -s "$OUT/$n-model.png" ] || { echo "FAIL: no picture $n"; exit 1; }; done
echo "  5 pictures of the model saved under $OUT (and the sheet beside each)"

echo
echo "== What the pictures hold, measured =="
python3 - "$OUT" <<'PY'
import sys
from PIL import Image
out = sys.argv[1]
def pixels(n, box):
    im = Image.open(f"{out}/{n:02d}-model.png").convert("RGB").crop(box)
    return [im.getpixel((x, y)) for x in range(im.width) for y in range(im.height)]
def mean(n, box):
    px = pixels(n, box)
    return tuple(round(sum(p[i] for p in px) / len(px)) for i in range(3))
ok = True
def check(what, got, good, want):
    global ok
    ok = ok and good
    print("    %-66s %s  %s" % (what, got, "as wanted" if good else "FAIL: wanted " + want))
WALL, ROOF, SHADOW, FRAME = (860, 540, 900, 580), (880, 290, 920, 310), (1140, 550, 1170, 570), (700, 480, 1100, 600)
cedar = lambda c: c[0] - c[2] > 24                      # warm: the siding
wrapped = lambda c: c[2] >= c[0] and c[0] > 130         # pale and cool: the weather barrier
lumber = lambda p: p[0] > 150 and p[0] - p[2] > 35 and p[0] > p[1] > p[2]
framed = lambda n: sum(1 for p in pixels(n, FRAME) if lumber(p)) / len(pixels(n, FRAME))
# 01, as it opens: cedar on the wall, the roof on, the house's shadow on the ground.
w1, r1, s1, f1 = mean(1, WALL), mean(1, ROOF), mean(1, SHADOW), framed(1)
check("01 as opened: the south wall is its siding", w1, cedar(w1), "red well over blue (cedar)")
# 02, Finishes off: the same place is the weather barrier; the test fails on 01.
w2 = mean(2, WALL)
check("02 the finish off: the same place is the weather barrier", w2, wrapped(w2), "pale and cool")
check("   the barrier test on 01 (the siding) fails", w1, not wrapped(w1), "the siding not to pass for a barrier")
check("   the siding test on 02 fails", w2, not cedar(w2), "the barrier not to pass for siding")
# 03, Framing on: studs with the house seen between them: between a third and
# nine-tenths of the wall is lumber. On 01 none of it is.
f3 = framed(3)
check("03 the frame: lumber over the walls, with gaps between", round(f3, 2), 0.30 < f3 < 0.90, "between 0.30 and 0.90 lumber")
check("   on 01 none of the wall is lumber", round(f1, 2), f1 < 0.10, "under 0.10")
# 04, Ground off: the house's shadow has nothing to fall on.
s4 = mean(4, SHADOW)
check("04 the ground away: no shadow where the shadow fell", s4, s4[0] > 170, "the backdrop, lighter than 170")
check("   on 01 the shadow is there", s1, s1[0] < 160, "the shadow, darker than 160")
# 05, the first floor alone: where the roof was there is none.
r5 = mean(5, ROOF)
check("05 the first floor alone: no roof where the roof was", r5, r5[0] > 150, "lighter than 150")
check("   on 01 the roof is there", r1, r1[0] < 120, "the roofing, darker than 120")
sys.exit(0 if ok else 1)
PY

echo
echo "== A switch changes what is drawn and no figure =="
"$P" estimate "$J" | grep -F "PRICE" > "$T/price-after.txt"
cmp -s "$T/price-before.txt" "$T/price-after.txt" || { echo "FAIL: the price changed"; exit 1; }
sed 's/^/    /' "$T/price-after.txt"

echo
echo "PASS: with one click each, the finish came off to the weather barrier, the walls were drawn as their frames, the ground was taken from the foundation, and the first floor was looked down into; each measured, and each measurement fails on the picture before its switch"
echo "The pictures are the evidence: read them."
