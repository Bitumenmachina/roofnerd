#!/usr/bin/env bash
# Floors, rooms and a roof drawn in the application, on a house traced from a plan. From
# an empty job: the made-up vector plan (demos/plans/made-up-plan.pdf: 30 ft by 20 ft, a
# partition 12 ft from its west side) is laid under a level and given its scale from its
# 30 ft dimension; its outline and its partition are traced, the pointer snapping to the
# plan's own lines; then the Floor tool and one click (the floor is the outline the walls
# enclose); the Room tool, a click inside each room and its name typed; the Roof tool, the
# footprint's four corners, the first again to close it, with "6:12, 8 ft" said first.
# Then the saved job is read back and set against figures worked by hand.
# Run from the repository root:  demos/buildw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job is made up.
# Every step fails the script if the program does not do what it says. The figures are
# worked by hand in STATUS.md ("Floors, rooms and roofs") and in demos/build.sh.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/buildw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/buildw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/build}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/house.json"
cp demos/plans/made-up-plan.pdf "$T/"
"$P" new "$J" "Built from a plan in the window (not a real job)"
cp "$J" "$T/before.json"

echo "== The session: the plan, its scale, the walls, the floor, two rooms and the roof =="
# On the whole sheet as it is framed in the 1600 by 960 model window, the plan's outline
# has its corners at (539, 653), (1061, 653), (1061, 305) and (539, 305): 522 pixels for
# 30 ft, 17.4 to the foot. Its partition runs from (748, 653) to (748, 305), 12 ft east.
# Its dimension line runs from (539, 688) to (1061, 688). Every click below is a few
# pixels off the place it is for. The left panel's rows, with a drawing laid: the scale
# at 318; Wall 413, Floor 533, Room 573, Roof 613.
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
xdotool key t; sleep 4
on "Add level"; sleep 2; typed "Ground, 0 ft, 8 ft"    # Add level
on "Add drawing"; sleep 2; typed "$T/made-up-plan.pdf" # Add drawing
sleep 8
on "Set scale"                          # Set scale, from one dimension
click 543 691                           # the ends of the PDF's own dimension line
click 1057 691
typed "30 ft"
tool "Wall" "6 in, 8 ft"                # Wall
click 542 650                           # near each corner of the outline, in turn
click 1058 656
click 1064 308
click 536 302
click 542 650                           # and the first again
esc; esc
on "Wall"                               # Wall again: the partition, end to end
click 751 656
click 745 302
esc; esc
shot                                    # 01 the walls traced over the plan
tool "Floor" "4 in"                     # Floor
click 900 420                           # anywhere on the level
esc
shot                                    # 02 the floor: the outline the walls enclose
on "Room"                               # Room
click 643 479; sleep 2; typed "West room"
click 904 479; sleep 2; typed "East room"
esc
shot                                    # 03 two rooms, each with its floor inside the walls
tool "Roof" "6:12, 8 ft"                # Roof
click 542 650                           # the footprint's corners, in turn
click 1058 656
click 1064 308
click 536 302
xdotool mousemove 542 650; sleep 3
shot                                    # 04 about to close the footprint on its first corner
xdotool mousedown 1; sleep 0.7; xdotool mouseup 1; sleep 6
shot                                    # 05 the roof on the plan
xdotool key t; sleep 8
shot                                    # 06 the house in 3D with its roof on
xdotool key F2; sleep 3                 # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  12 pictures saved to $OUT (model and sheet, after each of 6 steps)"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
# Walls: 30 + 20 + 30 + 20 round the house and the 20 ft partition, 120 LF. The floor, the
# walls' outline, 30 by 20: 600 SF. The rooms inside the walls' faces: 11.5 by 19.5 is
# 224.25 SF and 17.5 by 19.5 is 341.25 SF. The roof, 6:12 on every side of 30 by 20 with
# its eaves 8 ft up: a 10 LF ridge, four 15 LF hips, 100 LF of eaves, and 600 SF on plan
# times the square root of 1.25: 670.82 SF. All of it traced, so all of it Assumed.
python3 - "$J" <<'PY'
import sys, json
data = json.load(open(sys.argv[1]))["data"]
kind = lambda e: e["attributes"].get("pl::class")
m = lambda e, name: e["attributes"]["pl::measure"][name]
of = lambda k: [e for e in data if kind(e) == k]
total = lambda k, name: round(sum(m(e, name)["value"] for e in of(k)), 2)
# What a thing is called is held by what it is under: its name there, to its path.
called = {path: name for e in data for name, path in e.get("children", {}).items()}
rooms = {called[e["path"]]: round(m(e, "floor_area")["value"], 2) for e in of("Room")}
got = {
    "walls": (len(of("Wall")), total("Wall", "plan_length")),
    "floor": (len(of("Floor")), total("Floor", "plan_area")),
    "rooms": rooms,
    "roof planes": (len(of("RoofPlane")), total("RoofPlane", "surface_area")),
    "ridge": total("Ridge", "true_length"),
    "hips": (len(of("Hip")), total("Hip", "true_length")),
    "eaves": total("Eave", "true_length"),
}
want = {
    "walls": (5, 120.0),
    "floor": (1, 600.0),
    "rooms": {"West room": 224.25, "East room": 341.25},
    "roof planes": (4, 670.82),
    "ridge": 10.0,
    "hips": (4, 60.0),
    "eaves": 100.0,
}
for k in want:
    print("    %-12s %s" % (k, got[k]))
if got != want:
    print("FAIL: wanted", want); sys.exit(1)
for e in of("Wall") + of("Floor") + of("Room"):
    for name, d in e["attributes"]["pl::measure"].items():
        if d["value"] is not None and d["status"] == "Known" and name in ("plan_length", "plan_area", "floor_area", "face_area"):
            print("FAIL: traced, and Known:", e["path"], name); sys.exit(1)
floor = of("Floor")[0]["attributes"]["pl::floor"]["boundary"]
if "the outline of the walls on Ground" not in floor["source"]:
    print("FAIL: the floor does not say it is the outline of its walls:", floor["source"]); sys.exit(1)
print("    everything traced is Assumed; the floor says it is the outline of the walls on Ground")
PY
echo "  PASS: a floor, two rooms and a roof were drawn in the window on walls traced from a plan, and the saved job gives the figures worked by hand"
