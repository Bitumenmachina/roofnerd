#!/usr/bin/env bash
# Drawing demonstration in the application: from an empty job, a small house drawn in
# the plan view with the pointer and typed lengths. A storey, five walls, two doors, three
# windows, a duct, a pipe, a fixture and a piece of equipment; a step undone and done
# again; a wall moved; a wall and a window removed. Then the saved job is read back and its figures
# set against the ones worked by hand for the same house in demos/draw.sh.
# Run from the repository root:  demos/draww.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The house is made up.
# Every step fails the script if the program does not do what it says.
#
# Exact figures come from typed lengths and the angle guides, not from where the pointer
# happens to be: the first corner is wherever the first click lands, and no length or
# area depends on it.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/draww.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/draww.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/draw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"

echo "== The job: empty =="
"$P" new "$J" "Drawn in the window (not a real job)"
cp "$J" "$T/before.json"

echo
echo "== The session: a storey, walls, doors, windows, a duct, a pipe, a fixture, equipment; undo; move; remove; save =="
# Where things are on the 1600 by 960 model window: the left panel's rows, and the plan,
# on which ten feet are about 133 pixels and the middle of the window is the origin.
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export VK_ICD_FILENAMES="$ICD" WGPU_BACKEND=vulkan PLUMBLINE_SHOTS="$OUT"
"$APP" "$J" > "$T/app.log" 2>&1 &
APP_PID=\$!
for i in \$(seq 1 60); do xdotool search --name "plumbline: model" >/dev/null 2>&1 && break; sleep 1; done
sleep 40   # the first frames are slow on a software renderer
xdotool windowfocus "\$(xdotool search --name 'plumbline: model' | head -1)"; sleep 1
shot() { xdotool key F12; sleep 12; }
click() { xdotool mousemove \$1 \$2; sleep 1; xdotool click 1; sleep 3; }
esc() { xdotool key Escape; sleep 2; }
# A tool's row, then what it draws with, typed once the field is up.
tool() { click \$1 \$2; sleep 2; xdotool type --delay 250 "\$3"; sleep 1; if [ -n "\$4" ]; then shot; fi; xdotool key Return; sleep 3; }
# A length: the pointer shows the way, the first digit opens the field, and the rest is
# typed once the field is up (on a software renderer the frame that draws a new field is
# slow, and keys typed during it are lost).
len() { xdotool mousemove \$1 \$2; sleep 2; xdotool type "\${3:0:1}"; sleep 3; xdotool type --delay 250 "\${3:1}"; sleep 1; if [ -n "\$4" ]; then shot; fi; xdotool key Return; sleep 4; }
xdotool key t; sleep 4
shot                                    # 01 the empty job on plan: nothing to draw on yet
tool 64 184 "Ground, 0 ft, 9 ft"        # New storey
tool 44 257 "6 in, 9 ft" yes            # 02 Wall: what it draws with, being typed
click 600 600                           # the first corner
len 900 605 "24 ft" yes                 # 03 east: a length being typed
len 922 400 "16 ft"                     # north
len 600 385 "24 ft"                     # west
xdotool mousemove 603 597; sleep 2
shot                                    # 04 about to close on the first corner
xdotool click 1; sleep 4
esc
click 760 602                           # the middle of the south wall
len 760 400 "16 ft"                     # a partition north from it
esc; esc
tool 45 297 "3 ft, 7 ft, 0 ft"          # Door
click 680 600                           # in the south wall
click 760 500                           # in the partition
esc
tool 55 337 "4 ft, 3 ft, 3 ft"          # Window
click 919 500                           # in the east wall
click 680 387                           # two in the north wall
click 860 387
esc
xdotool mousemove 1000 800; sleep 1
shot                                    # 05 walls, doors and windows
tool 45 377 "6 in, 8 ft, Supply Air"    # Duct
click 630 570
len 760 572 "8 ft"
len 737 400 "12 ft"
esc; esc
tool 44 417 "3/4 in, 1 ft, Domestic Cold Water"   # Pipe
click 780 585
len 780 400 "14 ft"
esc; esc
tool 51 537 "Lavatory, 2 x 1.5 x 3, 0 ft"         # Fixture
xdotool mousemove 781 398; sleep 2
shot                                    # 06 about to stand it on the pipe's end
xdotool click 1; sleep 3
esc
tool 62 577 "Air handler, 2 x 2 x 4, 0 ft"        # Equipment
click 631 571                           # on the duct's start
esc
xdotool mousemove 1000 800; sleep 1
shot                                    # 07 everything drawn
xdotool key z; sleep 4
shot                                    # 08 the last step undone: the equipment is gone
xdotool key y; sleep 4                  # and done again
click 760 450                           # the partition, selected
click 97 611                            # Move what is selected
click 760 493                           # from its middle
len 900 494 "1 ft"                      # a foot east
xdotool mousemove 1000 800; sleep 1
shot                                    # 09 the partition a foot east, its door with it
click 44 257                            # Wall again: a stray one, to be removed
click 1000 700
len 1100 702 "5 ft"
esc; esc
click 1030 700                          # the stray wall, selected
shot                                    # 10 the stray wall selected
xdotool key Delete; sleep 4
xdotool mousemove 1000 800; sleep 1
shot                                    # 11 removed
click 860 387                           # the second window in the north wall: on plan, a click on it selects it
shot                                    # 12 the window selected, not its wall
xdotool key Delete; sleep 4
xdotool mousemove 1000 800; sleep 1
shot                                    # 13 the window taken out
tool 54 457 "3/4 in, 8 ft, Steam"       # Conduit, on a system no conduit goes on
xdotool mousemove 1000 800; sleep 1
shot                                    # 14 refused where it was typed, not at the second pick
xdotool key t; sleep 6
shot                                    # 15 the house in 3D
xdotool key F2; sleep 3                 # save
kill \$APP_PID; wait \$APP_PID 2>/dev/null
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  30 pictures saved to $OUT (model and sheet, after each of 15 steps)"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
# The figures of demos/draw.sh's house after its two edits, the partition a foot east and a
# window out of the north wall: walls 24 + 16 + 24 + 16 + 16 = 96 LF; net faces 195 (south,
# a door), 132 (east, a window), 204 (north, 216 less the one window left), 144 (west), 123
# (partition, a door); duct 8 + 12 = 20 LF; pipe 14 LF. Every one of them Assumed: it was
# drawn in the window.
python3 - "$J" <<'PY'
import sys, json
data = json.load(open(sys.argv[1]))["data"]
kind = lambda e: e["attributes"].get("pl::class")
figure = lambda e, name: e["attributes"]["pl::measure"][name]
walls = [e for e in data if kind(e) == "Wall"]
count = lambda k: sum(1 for e in data if kind(e) == k)
got = {
    "walls": len(walls),
    "wall length": round(sum(figure(w, "plan_length")["value"] for w in walls), 4),
    "net faces": sorted(figure(w, "net_area")["value"] for w in walls),
    "doors": count("Door"), "windows": count("Window"),
    "duct": round(sum(figure(e, "true_length")["value"] for e in data if kind(e) == "Duct"), 4),
    "pipe": round(sum(figure(e, "true_length")["value"] for e in data if kind(e) == "Pipe"), 4),
    "fixtures": count("Fixture"), "equipment": count("Equipment"),
}
want = {"walls": 5, "wall length": 96.0, "net faces": [123.0, 132.0, 144.0, 195.0, 204.0], "doors": 2, "windows": 2, "duct": 20.0, "pipe": 14.0, "fixtures": 1, "equipment": 1}
for k in want:
    print("    %-12s %s" % (k, got[k]))
if got != want:
    print("FAIL: wanted", want); sys.exit(1)
# Everything drawn carries the window as its source, Assumed.
for w in walls:
    line = w["attributes"]["pl::wall"]["line"]
    if line["status"] != "Assumed" or "3D view" not in line["source"]:
        print("FAIL: a wall's line is not flagged as drawn in the window:", w["path"], line); sys.exit(1)
moved = [w["path"] for w in walls if "moved" in w["attributes"]["pl::wall"]["line"]["source"]]
print("    every wall's line is Assumed, drawn in the window; moved:", ", ".join(moved))
if len(moved) != 1:
    print("FAIL: wanted one wall moved"); sys.exit(1)
PY
# Rooms are still placed at the command line: a point in each half, from the walls' lines.
read -r STOREY WX WY EX EY <<<"$(python3 - "$J" <<'PY'
import sys, json
data = json.load(open(sys.argv[1]))["data"]
walls = [e for e in data if e["attributes"].get("pl::class") == "Wall"]
xs = sorted({p[0] for w in walls for p in w["attributes"]["pl::wall"]["line"]["value"]})
ys = sorted({p[1] for w in walls for p in w["attributes"]["pl::wall"]["line"]["value"]})
storey = next(e["path"] for e in data if e["attributes"].get("pl::class") == "Storey")
print(storey, (xs[0] + xs[1]) / 2, (ys[0] + ys[-1]) / 2, (xs[1] + xs[2]) / 2, (ys[0] + ys[-1]) / 2)
PY
)"
"$P" room "$J" "$STOREY" "west room" --at "$WX,$WY" --status Assumed --source "made-up plan" >/dev/null
"$P" room "$J" "$STOREY" "east room" --at "$EX,$EY" --status Assumed --source "made-up plan" >/dev/null
"$P" check "$J"
# With the partition a foot east: west 12.5 x 15.5 = 193.75 SF, east 10.5 x 15.5 = 162.75 SF.
python3 - "$J" <<'PY'
import sys, json
data = json.load(open(sys.argv[1]))["data"]
rooms = {e["name"] if "name" in e else e["path"]: e["attributes"]["pl::measure"]["floor_area"] for e in data if e["attributes"].get("pl::class") == "Room"}
areas = sorted(r["value"] for r in rooms.values())
print("    the rooms inside the drawn walls:", areas, [r["status"] for r in rooms.values()])
if areas != [162.75, 193.75] or any(r["status"] != "Assumed" for r in rooms.values()):
    print("FAIL: wanted 162.75 and 193.75 SF, Assumed"); sys.exit(1)
PY
echo "  PASS: a house was drawn in the window from an empty job, and the saved job has the figures worked by hand, each flagged as drawn"
