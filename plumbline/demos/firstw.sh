#!/usr/bin/env bash
# The first job, on a real plan: from a job whose file is not there yet, USDA Plan No.
# 7213 (Two-Bedroom Farmhouse, 1976; public domain; a scan) is added under a level, its
# scale is set from its 41'-3" overall dimension, and its outside walls, a door, two
# windows, the floor, a room and the roof are traced over it with the build bar's tools.
# The 3D house then stands on its sheet. Then the saved job is read back and set against
# figures worked by hand in STATUS.md ("The first job, on a real plan").
# Run from the repository root:  demos/firstw.sh [folder for the pictures]
# The sheet is fetched once by tools/fetch-plans.sh into plans/, which git ignores.
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do).
# Every step fails the script if the program does not do what it says.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/firstw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/firstw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
REAL="$PWD/plans/usda-7213-two-bedroom-farmhouse.pdf"
[ -f "$REAL" ] || { echo "demos/firstw.sh traces USDA Plan No. 7213, which is not here: run tools/fetch-plans.sh first"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/first}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/farmhouse.json"
cp "$REAL" "$T/usda-7213.pdf"
touch "$T/started"

echo "== The session: a new job; the plan, its scale; walls, a door, two windows, the floor, a room, the roof; 3D; save =="
# Where things are on the sheet, two steps of the wheel in: the ends of the 41'-3"
# dimension under the house, and the house's southwest outside corner, by eye (as
# demos/planw.sh has them). Everything after that corner is placed in feet from it: the
# script asks the program where on screen that is.
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
# Two steps of the wheel over the floor plan: the wheel closes in on what the pointer is on.
closer() { xdotool mousemove 760 420; sleep 2; xdotool click 4; sleep 2; xdotool click 4; sleep 6; }
# A place so many feet east and north of the first corner.
from_corner() { at "\$(awk -v a="\$E0" -v d="\$1" 'BEGIN { print a + d }')" "\$(awk -v a="\$N0" -v d="\$2" 'BEGIN { print a + d }')"; }
shot                                    # 01 a new job: Next steps says where to start
on "Plan"; sleep 4
on "Add level"; sleep 2; typed "Ground, 0 ft, 8 ft"
on "Add drawing"; sleep 2; typed "$T/usda-7213.pdf, 4"
sleep 30                                # a scanned sheet takes a while to read
closer
on "Set scale"
click 480 696                           # the west end of the 41'-3" dimension
click 1099 699                          # its east end
typed "41' 3\""
closer                                  # the sheet is to scale now, and was framed again
park
shot                                    # 02 the sheet, to scale
tool "Wall" "6 in, 8 ft"
click 478 614                           # the southwest outside corner, by eye
read -r E0 N0 <<<"\$(plan_ft 478 614)"  # where that is on the plan, in feet
len 1000 616 "41' 3\""                  # east, the overall dimension
len 1102 400 "23' 11\""                 # north
len 700 252 "41' 3\""                   # west
from_corner 0 0                         # and closed on the first corner
esc; esc
park
shot                                    # 03 the outside walls
tool "Door" "3 ft, 7 ft, 0 ft"
from_corner 10 0                        # 10 ft along the south wall
esc
tool "Window" "4 ft, 3 ft, 3 ft"
from_corner 30 0                        # 30 ft along the south wall
from_corner 20 23.9167                  # 20 ft along the north wall
esc
tool "Floor" "4 in"
from_corner 20 12                       # anywhere on the level
esc
on "Room"
from_corner 20 12; sleep 2; typed "House"
esc
park
shot                                    # 04 a door, two windows, the floor and the room
tool "Roof" "6:12, 8 ft"
from_corner 0 0                         # the four corners in turn
from_corner 41.25 0
from_corner 41.25 23.9167
from_corner 0 23.9167
from_corner 0 0                         # and the first again
sleep 4
park
shot                                    # 05 the roof over the level, as a plan draws what is overhead
on "3D"; sleep 8
shot                                    # 06 the house in 3D, standing on its sheet
on "Save"; sleep 3
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1800 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06; do
  [ "$OUT/$n-model.png" -nt "$T/started" ] && [ "$OUT/$n-sheet.png" -nt "$T/started" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  12 pictures saved to $OUT (model and sheet, after each of 6 steps)"

echo
echo "== What the session left in the saved job =="
[ -s "$J" ] || { echo "FAIL: the new job was not written when it was saved"; exit 1; }
"$P" check "$J"
python3 - "$J" <<'PY'
import sys, json
data = json.load(open(sys.argv[1]))["data"]
kind = lambda e: e["attributes"].get("pl::class")
m = lambda e, name: e["attributes"]["pl::measure"][name]
of = lambda k: [e for e in data if kind(e) == k]
total = lambda k, name, places=2: round(sum(m(e, name)["value"] for e in of(k)), places)
called = {path: name for e in data for name, path in e.get("children", {}).items()}
got = {
    "walls": (len(of("Wall")), total("Wall", "plan_length", 4)),
    "wall face, net": total("Wall", "net_area"),
    "doors, windows": (len(of("Door")), len(of("Window"))),
    "floor": (len(of("Floor")), total("Floor", "plan_area")),
    "rooms": {called[e["path"]]: round(m(e, "floor_area")["value"], 2) for e in of("Room")},
    "roof planes": (len(of("RoofPlane")), total("RoofPlane", "surface_area")),
    "ridge": total("Ridge", "true_length"),
    "hips": (len(of("Hip")), total("Hip", "true_length")),
    "eaves": total("Eave", "true_length"),
}
# 2 x (41.25 + 23.9167); 1,042.6672 less 21 + 12 + 12; 41.25 x 23.9167; 40.75 x 23.4167;
# 986.5639 x the square root of 1.25; 41.25 - 23.9167; 4 x 11.9584 x 1.5; the walls again.
want = {
    "walls": (4, 130.3334),
    "wall face, net": 997.67,
    "doors, windows": (1, 2),
    "floor": (1, 986.56),
    "rooms": {"House": 954.23},
    "roof planes": (4, 1103.01),
    "ridge": 17.33,
    "hips": (4, 71.75),
    "eaves": 130.33,
}
for k in want:
    print("    %-16s %s" % (k, got[k]))
if got != want:
    print("FAIL: wanted", want); sys.exit(1)
source = "traced from usda-7213, page 4 (usda-7213.pdf, page 4)"
for e in of("Wall"):
    line = e["attributes"]["pl::wall"]["line"]
    if line["status"] != "Assumed" or line["source"] != source:
        print("FAIL: a traced wall is not flagged as traced from the drawing:", e["path"], line); sys.exit(1)
for e in of("Wall") + of("Floor") + of("Room") + of("RoofPlane"):
    for name, d in e["attributes"]["pl::measure"].items():
        if d["value"] is not None and d["status"] == "Known" and name in ("plan_length", "plan_area", "floor_area", "face_area", "net_area", "surface_area"):
            print("FAIL: traced, and Known:", e["path"], name); sys.exit(1)
print("    every wall is Assumed, traced from the drawing; no traced figure is Known")
PY
echo "  PASS: from a new job, a real plan was added, scaled and traced into walls, a door, windows, a floor, a room and a roof in the window, and the saved job gives the figures worked by hand"
