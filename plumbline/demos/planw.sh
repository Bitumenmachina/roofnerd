#!/usr/bin/env bash
# Plans under the tracing, in the application. Two sessions, each from an empty job.
#
# The first is a real sheet: USDA Plan No. 7213, Two-Bedroom Farmhouse (1976; public
# domain; a scan, so there is nothing on it to snap to). Its floor-plan sheet is laid under
# a level; a wall is refused while the sheet has no scale; the scale is set from the
# sheet's 41'-3" overall dimension, picked at its two ends by eye; and the outside walls
# are traced, the first corner picked on the sheet and each side typed as the sheet
# dimensions it: 41'-3" by 23'-11". The 3D house stands on the sheet.
#
# The second is the made-up vector plan (demos/plans/made-up-plan.pdf: 30 ft by 20 ft at
# 1/4 in to the foot). Its scale is set from its 30 ft dimension line, the pointer snapping
# to the line's own ends, and its outline is traced by four clicks near its corners, each
# snapping to the PDF's own lines, with no length typed.
#
# Then both saved jobs are read back and set against figures worked by hand.
# Run from the repository root:  demos/planw.sh [folder for the pictures]
# The real sheet is fetched once by tools/fetch-plans.sh into plans/, which git ignores.
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do).
# Every step fails the script if the program does not do what it says.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/planw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/planw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
REAL="$PWD/plans/usda-7213-two-bedroom-farmhouse.pdf"
[ -f "$REAL" ] || { echo "demos/planw.sh traces USDA Plan No. 7213, which is not here: run tools/fetch-plans.sh first"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/plan}"; mkdir -p "$OUT/made-up"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/farmhouse.json"; V="$T/made-up.json"
cp "$REAL" "$T/usda-7213.pdf"
cp demos/plans/made-up-plan.pdf "$T/"

# What both sessions share: the program started on a job, and how a key, a click and a
# picture are made. A click is held longer than a frame takes to draw.
preamble() { cat <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$3" ICD="$ICD" LOG="$2"
. "$PWD/demos/window.sh"
open_window "$1"
# Two steps of the wheel over the floor plan: the wheel closes in on what the pointer is on.
closer() { xdotool mousemove 760 420; sleep 2; xdotool click 4; sleep 2; xdotool click 4; sleep 6; }
xdotool key t; sleep 4
on "Add level"; sleep 2; typed "Ground, 0 ft, 8 ft"    # Add level
SESSION
}

echo "== A real sheet: USDA Plan No. 7213, its floor plan, from an empty job =="
"$P" new "$J" "Traced from USDA Plan No. 7213 (a public plan, not a real job)"
cp "$J" "$T/before.json"
# Where things are on the 1600 by 960 model window: the left panel's rows; and, two steps
# of the wheel in on the floor plan, the ends of the 41'-3" dimension under the house and
# the house's southwest outside corner.
{ preamble "$J" "$T/app1.log" "$OUT"; cat <<SESSION
on "Add drawing"; sleep 2; typed "$T/usda-7213.pdf, 4" # Add drawing: the file, and its page
sleep 30                                # a scanned sheet takes a while to read
on "Drawing usda-7213, page 4"          # the drawing, selected: what was not read from it
park
shot                                    # 01 the sheet under the plan: no scale yet
esc
closer
tool "Wall" "6 in, 8 ft"                # Wall
click 600 450
shot                                    # 02 refused: nothing is traced over a sheet with no scale
esc
on "Set scale"                          # Set scale, from one dimension
click 480 696                           # the west end of the 41'-3" dimension
click 1099 699                          # its east end
typed "41' 3\"" yes                     # 03 the dimension, typed as the sheet gives it
closer                                  # the sheet is to scale now, and was framed again
park
shot                                    # 04 to scale
tool "Wall"                              # Wall
click 478 614                           # the southwest outside corner, by eye
len 1000 616 "41' 3\"" yes              # 05 east, the overall dimension, being typed
len 1102 400 "23' 11\""                 # north
len 700 252 "41' 3\""                   # west
xdotool mousemove 480 612; sleep 3
shot                                    # 06 about to close on the first corner
xdotool mousedown 1; sleep 0.7; xdotool mouseup 1; sleep 4
esc; esc
park
shot                                    # 07 the outside walls, traced over the sheet
xdotool key t; sleep 8
shot                                    # 08 the house in 3D, standing on its sheet
xdotool key F2; sleep 3                 # save
close_window
SESSION
} > "$T/session1.sh"
chmod +x "$T/session1.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session1.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app1.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app1.log" | head; exit 1; fi

echo
echo "== A vector plan: the made-up plan, its own lines snapped to =="
"$P" new "$V" "Traced from a made-up plan (not a real job)"
# On the whole sheet as it is framed, the plan's outline has its corners at (539, 653),
# (1061, 653), (1061, 305) and (539, 305), and its dimension line runs from (539, 688) to
# (1061, 688). Every click below is a few pixels off the place it is for.
{ preamble "$V" "$T/app2.log" "$OUT/made-up"; cat <<SESSION
on "Add drawing"; sleep 2; typed "$T/made-up-plan.pdf" # Add drawing
sleep 8
on "Set scale"                          # Set scale, from one dimension
xdotool mousemove 543 691; sleep 3
shot                                    # made-up/01 the pointer on the end of the PDF's own dimension line
click 543 691
click 1057 691
typed "30 ft"
park
tool "Wall" "6 in, 8 ft"                # Wall
click 542 650                           # near each corner of the outline, in turn
click 1058 656
click 1064 308
click 536 302
xdotool mousemove 542 650; sleep 3
shot                                    # made-up/02 about to close on the first corner
xdotool mousedown 1; sleep 0.7; xdotool mouseup 1; sleep 4
esc; esc
on "Drawing made-up-plan"               # the drawing, selected
park
shot                                    # made-up/03 traced; the drawing's scale as its title block writes it
xdotool key F2; sleep 3                 # save
close_window
SESSION
} > "$T/session2.sh"
chmod +x "$T/session2.sh"
timeout 900 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session2.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app2.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app2.log" | head; exit 1; fi
ls "$OUT" "$OUT/made-up" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06 07 08 made-up/01 made-up/02 made-up/03; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  22 pictures saved to $OUT (model and sheet: 8 steps on the real sheet, 3 in made-up/ on the vector plan)"

echo
echo "== What the first session left in the saved job: the farmhouse's outside walls =="
"$P" check "$J"
"$P" drawings "$J" | sed 's/^/    /' | head -6
# 41'-3" is 41.25 ft. 23'-11" is 23.91666... ft, and a place on the plan is kept to a
# ten-thousandth of a foot, so the wall is 23.9167 ft long (0.0004 in over). Round the
# house 2 x (41.25 + 23.9167) = 130.3334 LF, shown as 130.33; at the 8'-0" the wall
# section gives, 1,042.6672 SF of face, shown as 1,042.67. Each wall is
# Assumed and says which drawing it was traced from. The scale is the 41'-3" as read off
# the sheet, Assumed. Picked by eye on a scan, the two ends are good to a pixel or two in
# 619, so the sheet's own size is only checked to be about what that makes it. On the
# whole sheet as it is framed, 766 pixels wide, the dimension is 360 pixels long: the
# sheet is 41.25 x 766 / 360 = 87.8 ft wide on the ground, give or take a foot.
python3 - "$J" <<'PY'
import sys, json
data = json.load(open(sys.argv[1]))["data"]
kind = lambda e: e["attributes"].get("pl::class")
walls = [e for e in data if kind(e) == "Wall"]
m = lambda e, name: e["attributes"]["pl::measure"][name]
got = {
    "walls": len(walls),
    "lengths": sorted(round(m(w, "plan_length")["value"], 4) for w in walls),
    "round the house": round(sum(m(w, "plan_length")["value"] for w in walls), 4),
    "face": round(sum(m(w, "face_area")["value"] for w in walls), 4),
}
want = {"walls": 4, "lengths": [23.9167, 23.9167, 41.25, 41.25], "round the house": 130.3334, "face": 1042.6672}
for k in want:
    print("    %-16s %s" % (k, got[k]))
if got != want:
    print("FAIL: wanted", want); sys.exit(1)
source = "traced from usda-7213, page 4 (usda-7213.pdf, page 4)"
for w in walls:
    line = w["attributes"]["pl::wall"]["line"]
    if line["status"] != "Assumed" or line["source"] != source or m(w, "plan_length")["status"] != "Assumed":
        print("FAIL: a traced wall is not flagged as traced from the drawing:", w["path"], line); sys.exit(1)
print("    every wall is Assumed: \"%s\"" % source)
d = next(e for e in data if kind(e) == "Drawing")["attributes"]["pl::drawing"]
s = d["scale"]
if s["distance"]["status"] != "Assumed" or s["distance"]["source"] != "read off the drawing" or d["file"]["value"] != "usda-7213.pdf" or d["page"] != 4:
    print("FAIL: the drawing's scale is not the dimension read off it, flagged:", d); sys.exit(1)
on_page = ((s["to"][0] - s["from"][0]) ** 2 + (s["to"][1] - s["from"][1]) ** 2) ** 0.5
wide = d["size"][0] * 41.25 / on_page
print("    the scale: 41' 3\" over %.1f points of the page, Assumed, read off the drawing; the sheet is %.1f ft wide" % (on_page, wide))
if not 86.8 <= wide <= 88.8:
    print("FAIL: the sheet came out %.1f ft wide; the two ends of the dimension were not picked where it is" % wide); sys.exit(1)
PY

echo
echo "== What the second session left: the made-up plan's outline, snapped =="
"$P" check "$V"
# The PDF's dimension line is 540 points for 30 ft: 18 points to the foot, 1/4 in to the
# foot. Its outline is 540 by 360 points: 30 ft by 20 ft, exactly, with nothing typed.
python3 - "$V" <<'PY'
import sys, json
data = json.load(open(sys.argv[1]))["data"]
kind = lambda e: e["attributes"].get("pl::class")
d = next(e for e in data if kind(e) == "Drawing")["attributes"]["pl::drawing"]
s = d["scale"]
if [s["from"], s["to"]] != [[126.0, 90.0], [666.0, 90.0]]:
    print("FAIL: the scale's two points are not the ends of the PDF's dimension line:", s); sys.exit(1)
print("    the scale's two points are the dimension line's own ends: (126, 90) and (666, 90); 18 points to the foot")
walls = [e for e in data if kind(e) == "Wall"]
lines = [w["attributes"]["pl::wall"]["line"] for w in walls]
got = [l["value"] for l in lines]
want = [[[7.0, 7.0], [37.0, 7.0]], [[37.0, 7.0], [37.0, 27.0]], [[37.0, 27.0], [7.0, 27.0]], [[7.0, 27.0], [7.0, 7.0]]]
print("    the walls' lines:", got)
if got != want:
    print("FAIL: wanted the outline's corners, 126 and 666 points over 18, 126 and 486 over 18:", want); sys.exit(1)
if any(l["status"] != "Assumed" or l["source"] != "traced from made-up-plan (made-up-plan.pdf, page 1)" for l in lines):
    print("FAIL: a traced wall is not flagged as traced from the drawing"); sys.exit(1)
print("    30, 20, 30 and 20 ft, each Assumed, traced from made-up-plan (made-up-plan.pdf, page 1)")
PY
echo "  PASS: a real scanned sheet and a vector plan were each laid under a level, given a scale from one dimension and traced in the window, and the saved jobs have the figures worked by hand"
