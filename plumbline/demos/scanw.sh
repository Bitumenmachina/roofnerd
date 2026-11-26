#!/usr/bin/env bash
# A scan of a house as it stands, brought in from the window and traced over (DECISIONS.md
# call 128, the window's half of it; the command line's is demos/scan.sh). A job with one
# level is opened; under Bring in, Scan file is pressed and the scan is brought in by a click
# on its row in the list (call 161: not Enter; nobody types where a file lies): it lies under
# the plan as a sheet, to scale and in the scan's own place. Undo takes it back and Redo lays
# it again. Then one wall is drawn over it with the Wall tool and the job is saved. A second
# made-up scan, written on its side (its height and its depth so near alike that the program
# cannot tell which way is up), is brought in the same way: the program says so and offers
# "It is lying on its side: stand it up", which is pressed once and brings it in turned.
# What was said and what the map held at each step has no "tmp", no ".ply", no "y up" or
# "z up" (the map's rows named "File <name>" are the handles a script clicks, not what a
# person reads, and are left out of that check).
# Run from the repository root:  demos/scanw.sh [folder for the pictures]
#
# The scan is made up here (no real scan is in the repository), in the shape a phone app
# writes: bytes, each point's place as three real numbers and its colour as four whole
# ones, the triangles of a mesh after the points, in metres with y up (the way a phone
# keeps its own axes: the app's own pages do not say which it uses, so the window reads up
# from which way the house is shortest, and says so). A room 24 ft by 16 ft inside and 9 ft
# high, a point every 0.05 m on its walls, floor and ceiling; its southwest inside corner
# is 4 ft east and 4 ft north of the scan's zero.
#
# Worked by hand. The sheet is cut 4 ft above the floor: the room's four walls, 24 ft by
# 16 ft, with 2 ft of paper all round, at 24 dots to the foot: (24 + 4) x 24 = 672 dots
# wide by (16 + 4) x 24 = 480 high, so 672 dots are 28 ft. Its lower left corner is 2 ft
# short of the room's: (2, 2) ft. The walls lie on the plan at east 4 ft and 28 ft, north
# 4 ft and 20 ft. The wall drawn over it, from the southwest corner east for 24 ft, is 24 LF;
# its centre is on the room's south face (north 4 ft) when it is laid by its centre, and 3 in
# outside it (north 3.75 ft) when it is laid by the face, which over a scan it is (call 190).
#
# It drives the real program with a real pointer and keyboard on a virtual display, which
# needs Xvfb, xdotool and a Vulkan driver (Mesa's software one will do), and measures the
# pictures with python and PIL. Every step fails the script if the program does not do what
# it says.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/scanw.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL" 2>/dev/null || { echo "demos/scanw.sh needs python's PIL, to measure the pictures"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/scanw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/scanw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/room.json"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: $3: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }

echo "== The made-up scan, in the shape a phone writes, and a job with one level =="
python3 - "$T/made-up-room.ply" "$T/points.txt" 16 <<'PY'
import struct, sys
out, count, deep = sys.argv[1], sys.argv[2], float(sys.argv[3])
FT = 0.3048
x0, x1 = 4 * FT, 28 * FT              # the room's inside, east: 24 ft
n0, n1 = 4 * FT, (4 + deep) * FT      # north: 16 ft (in the file, z is minus north)
h = 9 * FT                            # 9 ft high; y is up
step = 0.05
def span(a, b):
    k = int(round((b - a) / step)); return [a + (b - a) * i / k for i in range(k + 1)]
pts = []
for x in span(x0, x1):
    for n in span(n0, n1):
        pts.append((x, 0.0, -n)); pts.append((x, h, -n))        # floor and ceiling
for y in span(0.0, h):
    for x in span(x0, x1):
        pts.append((x, y, -n0)); pts.append((x, y, -n1))        # south and north walls
    for n in span(n0, n1):
        pts.append((x0, y, -n)); pts.append((x1, y, -n))        # west and east walls
faces = 200
head = ("ply\nformat binary_little_endian 1.0\ncomment made up for a demonstration\nelement vertex %d\n"
        "property float x\nproperty float y\nproperty float z\n"
        "property uchar red\nproperty uchar green\nproperty uchar blue\nproperty uchar alpha\n"
        "element face %d\nproperty list uchar int vertex_indices\nend_header\n" % (len(pts), faces))
with open(out, "wb") as f:
    f.write(head.encode())
    for i, (x, y, z) in enumerate(pts):
        f.write(struct.pack("<fffBBBB", x, y, z, (i * 7) % 256, 120, 200, 255))
    for t in range(faces):
        f.write(struct.pack("<Biii", 3, t, t + 1, t + 2))
open(count, "w").write("{:,}".format(len(pts)))
print("   ", len(pts), "points,", faces, "faces")
PY
# The second scan, on its side as far as the program can tell: 24 ft by 10 ft by 9 ft high,
# so that its height (9 ft) and its depth (10 ft) are within a fifth of each other.
python3 - "$T/lying-room.ply" "$T/points2.txt" 10 <<'PY'
import struct, sys
out, count, deep = sys.argv[1], sys.argv[2], float(sys.argv[3])
FT = 0.3048
x0, x1 = 4 * FT, 28 * FT
n0, n1 = 4 * FT, (4 + deep) * FT
h = 9 * FT
step = 0.05
def span(a, b):
    k = int(round((b - a) / step)); return [a + (b - a) * i / k for i in range(k + 1)]
pts = []
for x in span(x0, x1):
    for n in span(n0, n1):
        pts.append((x, 0.0, -n)); pts.append((x, h, -n))
for y in span(0.0, h):
    for x in span(x0, x1):
        pts.append((x, y, -n0)); pts.append((x, y, -n1))
    for n in span(n0, n1):
        pts.append((x0, y, -n)); pts.append((x1, y, -n))
head = ("ply\nformat binary_little_endian 1.0\ncomment made up for a demonstration\nelement vertex %d\n"
        "property float x\nproperty float y\nproperty float z\n"
        "property uchar red\nproperty uchar green\nproperty uchar blue\nproperty uchar alpha\nend_header\n" % len(pts))
with open(out, "wb") as f:
    f.write(head.encode())
    for i, (x, y, z) in enumerate(pts):
        f.write(struct.pack("<fffBBBB", x, y, z, (i * 7) % 256, 120, 200, 255))
open(count, "w").write("{:,}".format(len(pts)))
print("   ", len(pts), "points: the second scan, 24 ft by 10 ft by 9 ft high")
PY
# The third scan: a house 60 ft by 40 ft and 9 ft high, written in feet (a point every half
# foot along the plan, every tenth of a foot up). Brought in as meters it is 197 ft long, which
# is no house: that is what the window is to warn of (call 169).
python3 - "$T/big-house.ply" <<'PY'
import struct, sys
out = sys.argv[1]
def span(a, b, step):
    k = int(round((b - a) / step)); return [a + (b - a) * i / k for i in range(k + 1)]
pts = []
for x in span(0, 60, 0.5):
    for n in span(0, 40, 0.5):
        pts.append((x, 0.0, -n)); pts.append((x, 9.0, -n))
for y in span(0, 9, 0.1):
    for x in span(0, 60, 0.5):
        pts.append((x, y, 0.0)); pts.append((x, y, -40.0))
    for n in span(0, 40, 0.5):
        pts.append((0.0, y, -n)); pts.append((60.0, y, -n))
head = ("ply\nformat binary_little_endian 1.0\ncomment made up for a demonstration\nelement vertex %d\n"
        "property float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nproperty uchar alpha\nend_header\n" % len(pts))
with open(out, "wb") as f:
    f.write(head.encode())
    for i, (x, y, z) in enumerate(pts):
        f.write(struct.pack("<fffBBBB", x, y, z, (i * 7) % 256, 120, 200, 255))
print("   ", len(pts), "points: the third scan, 60 ft by 40 ft by 9 ft high, written in feet")
PY
"$P" new "$J" "A scanned room (not a real job)" >/dev/null
"$P" level "$J" ground --elevation "0 ft" --height "9 ft" --status Assumed --source "to be checked on site" >/dev/null
touch "$T/started"

echo
echo "== The session: Scan file pressed, the scan brought in by a click on its row; Undo and Redo; a wall traced over it; save; the scan on its side, stood up =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
on "Plan"; sleep 5
park
shot                                    # 01 the level on its plan, nothing under it
on "Scan file"; sleep 3                 # Bring in: Scan file
cp "$T/map.txt" "\$T/map-list.txt"
shot                                    # 02 the list of scans in this job's folder, the two choices at its head
on "Measured in feet"; sleep 2          # a choice is lit when chosen ...
shot                                    # 03 feet chosen
on "Measured in meters"; sleep 2        # ... and meters is the one to begin with
on "File made-up-room.ply"; sleep 14    # a click on the scan's row brings it in: no Enter
park; sleep 8
cp "$T/map.txt" "\$T/map-brought.txt"
plan_line > "$T/plan.txt"
shot                                    # 04 the scan laid under the plan, to scale
on "Undo"; sleep 4
park; sleep 3
shot                                    # 05 taken back
xdotool key F2; drawn_for 3 4                 # save: the job without the scan
cp "$J" "$T/undone.json"
on "Redo"; sleep 6
park; sleep 3
plan_line > "$T/plan2.txt"
shot                                    # 06 laid again
tool "Wall" "6 in, 9 ft"                # Wall
click \$(plan_px 4 4)                   # the southwest inside corner, by eye
len \$(plan_px 16 4) "24 ft" yes        # 07 east along the south wall, its length being typed
esc; esc
park
shot                                    # 08 the wall, traced over the scan
xdotool key F2; drawn_for 3 4                 # save
cp "$J" "$T/first.json"
on "Scan file"; sleep 3                 # a second scan, the one on its side
on "File lying-room.ply"; sleep 14      # brought in by its row, as the first was
park; sleep 6
cp "$T/map.txt" "\$T/map-side.txt"
shot                                    # 09 it came in on its side: the message and the button under it
on "It is lying on its side: stand it up"; sleep 14
park; sleep 6
cp "$T/map.txt" "\$T/map-stood.txt"
shot                                    # 10 stood up: brought in again, turned
xdotool key F2; drawn_for 3 4                 # save
cp "$J" "$T/last.json"
# A scan in the wrong measure (call 169). The first scan is written in meters: chosen as feet it
# is a house of 8 ft with a ceiling under 3 ft, and nothing of it is 4 ft above its floor.
on "Scan file"; sleep 3
on "Measured in feet"; sleep 2
on "File made-up-room.ply"; sleep 10
park; sleep 4
cp "$T/map.txt" "\$T/map-wrong.txt"
shot                                    # 11 refused, with the sentence about the other measure
on "Put the message away"; sleep 3      # the mark at the card's corner
cp "$T/map.txt" "\$T/map-away.txt"
shot                                    # 12 the card put away: the plan clear
# The third scan is written in feet: chosen as meters it is 197 ft long.
on "Scan file"; sleep 3
on "Measured in meters"; sleep 2
on "File big-house.ply"; sleep 20
park; sleep 6
cp "$T/map.txt" "\$T/map-big.txt"
shot                                    # 13 a warning, the sentence first
# The same scan again: it is already in, under the plan.
on "Scan file"; sleep 3
on "Measured in meters"; sleep 2
on "File big-house.ply"; sleep 14
park; sleep 4
cp "$T/map.txt" "\$T/map-twice.txt"
shot                                    # 14 already in
xdotool key Escape; drawn_for 3 3             # Esc, with no tool in hand, puts the card away
cp "$T/map.txt" "\$T/map-esc.txt"
shot                                    # 15 away again
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 07 08 09 10; do
  [ "$OUT/$n-model.png" -nt "$T/started" ] || { echo "FAIL: no picture $n from this run"; exit 1; }
done
echo "  10 pictures saved to $OUT"

echo
echo "== What the window said =="
SAID="$T/map.txt.said"
N="$(cat "$T/points.txt")"
says "$SAID" "Brought in the scan \"made-up-room\": $N points, measured in meters, Assumed." "Scan"
says "$SAID" "Which way is up was read from the scan." "Scan"
says "$SAID" "On plan, where it is cut, it is about 24' 0\" by 16' 0\"." "Scan"
says "$SAID" "Floor to ceiling reads about 9' 0\" on the scan." "Scan"
says "$SAID" "It is cut 4' 0\" above the floor, as a plan is." "Scan"
says "$SAID" "to scale and in the scan's own place: trace over it as over any plan." "Scan"
says "$SAID" "Only where its points are is kept: not its colors, and not the surface drawn over them." "Scan"
# The second scan: its way up is in doubt and the window says so; stood up, it is 24 ft by 10 ft
# and 9 ft high (before: 24 ft by 9 ft and 10 ft high, on its side).
says "$SAID" "On plan, where it is cut, it is about 24' 0\" by 9' 0\"." "the scan on its side"
says "$SAID" "Floor to ceiling reads about 10' 0\" on the scan." "the scan on its side"
says "$SAID" "Which way is up is not clear from the scan. If the house looks to be lying on its side, press the button below." "the scan on its side"
says "$SAID" "On plan, where it is cut, it is about 24' 0\" by 10' 0\"." "the scan stood up"
says "$SAID" "Floor to ceiling reads about 9' 0\" on the scan." "the scan stood up"
# Every scan's message ends with the sentence about the other measure; a scan refused because nothing
# of it is at the height it is cut at adds it; a house over 150 ft long is a warning with it first;
# the same scan twice says it is in; stood up begins "Stood up." (call 169).
SENT="If that is not the size of the house, choose the other measure at the head of the list and bring it in again."
[ "$(grep -cF "$SENT" "$SAID")" -ge 5 ] || { echo "FAIL: the sentence about the other measure was not said enough times"; grep -cF "$SENT" "$SAID"; exit 1; }
says "$SAID" "nothing to draw. $SENT" "the scan in the wrong measure"
says "$SAID" "Stood up. " "the scan stood up"
says "$SAID" "about 196' 10\" by 131' 3\"" "the 60 ft house read as meters"
says "$SAID" "It is already in, under the plan. Undo takes it out." "the same scan twice"
# (what the window said is one line to a message, its lines joined with " | ": the sentence begins the message)
BIG="$(grep -F 'Brought in the scan "big-house"' "$SAID" | head -1)"
case "$BIG" in "$SENT | Brought in the scan"*) ;; *) echo "FAIL: the sentence is not first in the warning for the 197 ft house"; echo "$BIG" | cut -c1-200; exit 1;; esac
echo "    the warning for a house over 150 ft puts the sentence first"
grep -q $'^model\tPut the message away\t' "$T/map-wrong.txt" || { echo "FAIL: the card has no mark to put it away"; exit 1; }
! grep -q $'^model\tPut the message away\t' "$T/map-away.txt" || { echo "FAIL: the card is still up after its mark was pressed"; exit 1; }
grep -q $'^model\tPut the message away\t' "$T/map-twice.txt" || { echo "FAIL: the card for the second bring-in is not up"; exit 1; }
! grep -q $'^model\tPut the message away\t' "$T/map-esc.txt" || { echo "FAIL: Esc did not put the card away"; exit 1; }
echo "    the message card has its mark; the mark puts it away, and so does Esc with no tool in hand"
# No software word is anywhere a person reads: in what was said, or on the map (the rows named
# "File <name>" are the handles a script clicks, and are left out). The folder is named
# "this job's folder" and never by its place.
if grep -qiE 'tmp|\.ply|y up|z up|vertex|binary|point cloud|mesh' "$SAID"; then echo "FAIL: what the window said has a software word or a path in it"; grep -iE 'tmp|\.ply|y up|z up|vertex|binary|point cloud|mesh' "$SAID"; exit 1; fi
for m in map-list map-brought map-side map-stood; do
  [ -s "$T/$m.txt" ] || { echo "FAIL: the session kept no $m"; exit 1; }
  if grep -v $'^model\tFile ' "$T/$m.txt" | grep -qiE 'tmp|\.ply|y up|z up'; then echo "FAIL: the map ($m) has a software word or a path in it"; grep -v $'^model\tFile ' "$T/$m.txt" | grep -iE 'tmp|\.ply|y up|z up'; exit 1; fi
done
echo "    nothing said and nothing on the map has tmp, .ply, y up or z up in it (4 maps, $(wc -l < "$SAID") lines said)"
grep -q $'^model\tMeasured in meters\t' "$T/map-list.txt" && grep -q $'^model\tMeasured in feet\t' "$T/map-list.txt" || { echo "FAIL: the scan list's two choices are not on the map"; exit 1; }
echo "    the list has its two choices at its head: Measured in meters, Measured in feet"
grep -q $'^model\tIt is lying on its side: stand it up\t' "$T/map-side.txt" || { echo "FAIL: the second scan came in on its side and the button is not on the map"; exit 1; }
if grep -q "It is lying on its side" "$T/map-brought.txt" "$T/map-stood.txt"; then echo "FAIL: the button is there for a scan that came in the right way up, or after it was stood up"; exit 1; fi
echo "    the button 'It is lying on its side: stand it up' is there for the second scan only, and gone once it is pressed"

echo
echo "== The saved jobs =="
"$P" check "$J"
"$P" drawings "$T/undone.json" > "$T/undone.txt"
if grep -q "made-up-room" "$T/undone.txt"; then echo "FAIL: Undo did not take the scan back"; cat "$T/undone.txt"; exit 1; fi
echo "    Undo took the scan's sheet out of the saved job (and Redo put it back)"
"$P" drawings "$J" > "$T/drawings.txt"
says "$T/drawings.txt" "28 ft between (0, 0) and (672, 0) on its page" "the sheet's scale"
says "$T/drawings.txt" "cut 4' 0\" above its floor (taken as meters with y up)" "the sheet's source"
says "$T/drawings.txt" "~ made-up-room" "Assumed"
# The second scan stood up came in once, turned: its sheet is 24 ft by 10 ft of room and
# 2 ft of paper all round, at 24 dots to the foot: 672 by (10 + 4) x 24 = 336 dots (on its side
# it was 672 by (9 + 4) x 24 = 312).
says "$T/drawings.txt" "~ lying-room" "the stood-up scan"
[ "$(grep -c -- '~ lying-room' "$T/drawings.txt")" = 1 ] || { echo "FAIL: the second scan is in the job more than once"; cat "$T/drawings.txt"; exit 1; }
python3 -I - "$T" <<'PY'
import sys, glob
from PIL import Image
f = glob.glob(sys.argv[1] + "/lying-room cut at*.png")
if len(f) != 1:
    sys.exit("FAIL: wanted the one sheet of the second scan, found %s" % f)
w, h = Image.open(f[0]).size
print("    the second scan's sheet is %d by %d dots" % (w, h))
if (w, h) != (672, 336):
    sys.exit("FAIL: wanted 672 by 336 dots, the scan stood up")
PY
"$P" show "$J" > "$T/show.txt"
python3 - "$J" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))["data"]
walls = [e["attributes"]["pl::wall"] for e in d if "pl::wall" in e["attributes"]]
if len(walls) != 1:
    print("FAIL: wanted the one wall drawn, found", len(walls)); sys.exit(1)
(a, b), line = walls[0]["line"]["value"], walls[0]["line"]
length = ((b[0] - a[0]) ** 2 + (b[1] - a[1]) ** 2) ** 0.5
print("    the wall runs from (%.2f, %.2f) to (%.2f, %.2f): %.2f LF, %s, %s" % (a[0], a[1], b[0], b[1], length, line["status"], line["source"]))
# Call 190: a wall drawn over a scan is laid by the scan's face. The room's south face is at north 4 ft and
# the room lies north of it, so the 6 in wall's body is south of the face and its centre half the type's
# thickness (3 in) south of it: north 3.75 ft, where it was 4 ft when the centre went on the face. Its ends
# are where the clicks landed along the face, as before (east 4 ft, 24 ft long).
if abs(a[0] - 4) > 0.6 or abs(a[1] - 3.75) > 0.05 or abs(b[1] - a[1]) > 0.01 or abs(length - 24) > 0.01:
    print("FAIL: wanted a wall from about (4, 3.75) east for 24 ft, 3 in outside the room's south face"); sys.exit(1)
if line["status"] != "Assumed" or "made-up-room" not in line["source"]:
    print("FAIL: the wall is not Assumed and traced from the scan's sheet"); sys.exit(1)
PY

echo
echo "== The pictures, measured =="
# The room's walls on the plan are at east 4 and 28 ft, north 4 and 20 ft. The program says
# where its plan lies on screen (its zero and the pixels to a foot), and the picture is
# measured along each wall's line: the share of places along it where a dark pixel lies
# within 3 pixels of the line. Then along a line inside the room, 6 ft off the south wall
# and 6 ft off the west wall, which the scan does not draw, as the control. The same
# measure on the picture from before the scan came in, at the same places, has to fail:
# a check that passes on a picture without the scan is no check.
# (Call 153: the sheet's dots were 2 across and read, at this size, 222 of 255 along a wall
# (233 at the wall's own line): too faint to trace over. They are 4 across and 100 of 255
# now: along a wall's own line the sheet reads 100. This measure's threshold, 238, is not
# moved: it is a "has a dot at all" test, and the sheet before the change and after it
# both pass it; what the change cures is read from the picture.)
python3 - "$OUT" "$T/plan2.txt" <<'PY'
import sys
from PIL import Image
out, planfile = sys.argv[1], sys.argv[2]
x0, y0, k = map(float, open(planfile).read().split())
def dark(img, x, y, r=3, threshold=238):
    px = img.load()
    for dx in range(-r, r + 1):
        for dy in range(-r, r + 1):
            X, Y = int(round(x)) + dx, int(round(y)) + dy
            if 0 <= X < img.width and 0 <= Y < img.height:
                p = px[X, Y]
                if (p[0] * 299 + p[1] * 587 + p[2] * 114) / 1000 < threshold:
                    return True
    return False
def along(img, a, b, shift=(0, 0)):
    # the share of places along a line from a to b (feet east, north) with a dark pixel at it
    (e0, n0), (e1, n1) = a, b
    steps = int(max(abs(e1 - e0), abs(n1 - n0)) * k)
    hits = 0
    for i in range(steps + 1):
        e, n = e0 + (e1 - e0) * i / steps + shift[0], n0 + (n1 - n0) * i / steps + shift[1]
        hits += dark(img, x0 + e * k, y0 - n * k)
    return hits / (steps + 1)
walls = {"south": ((4, 4), (28, 4)), "north": ((4, 20), (28, 20)), "west": ((4, 4), (4, 20)), "east": ((28, 4), (28, 20))}
controls = {"6 ft inside the south wall": ((4, 10), (28, 10)), "6 ft inside the west wall": ((10, 4), (10, 20))}
after, before, undone, drawn = (Image.open("%s/%s-model.png" % (out, n)).convert("RGB") for n in ("06", "01", "05", "08"))
print("    the plan on screen: zero at (%.0f, %.0f), %.2f pixels to the foot; the picture is %d by %d" % (x0, y0, k, after.width, after.height))
bad = 0
for name, (a, b) in walls.items():
    s, t, u, w = along(after, a, b), along(before, a, b), along(undone, a, b), along(drawn, a, b)
    print("    %-5s wall: with the scan %.2f; before it came in %.2f; after Undo %.2f; with the wall drawn %.2f" % (name, s, t, u, w))
    if s < 0.85 or t > 0.15 or u > 0.15:
        bad += 1
for name, (a, b) in controls.items():
    s = along(after, a, b)
    print("    %s: %.2f" % (name, s))
    if s > 0.15:
        bad += 1
# The check can fail: the same measure, with the lines moved 3 ft, finds the walls gone.
moved = [along(after, a, b, shift=(-3, -3)) for a, b in walls.values()]
print("    the same lines moved 3 ft west and 3 ft south: %s" % ", ".join("%.2f" % m for m in moved))
if max(moved) > 0.5:
    bad += 1
if bad:
    print("FAIL: the sheet's walls are not on the plan where the room's walls are (or the check passes where it should not)"); sys.exit(1)
PY
echo
echo "== Colour says what a message is, and a message put away is put away (call 178) =="
python3 -W ignore -I "$PWD/demos/cardcolour.py" "$OUT/04-model.png" green "$OUT/11-model.png" red "$OUT/13-model.png" amber "$OUT/14-model.png" amber
python3 -W ignore -I "$PWD/demos/barempty.py" "$OUT/11-model.png" "$OUT/12-model.png" "$OUT/15-model.png"
echo "  PASS: in the window, Scan file listed the scans in this job's folder with its two choices (meters, feet), a click on a row brought the scan in as a sheet under the plan, to scale and in its own place, said how big it came in, that which way is up was read from it and what was not kept, with no tmp, .ply, y up or z up said or on the map; Undo took it back and Redo laid it again; a wall traced over it is saved Assumed from it; a second scan, on its side as far as the program could tell, said so, and the button It is lying on its side: stand it up brought it in again turned (24 ft by 10 ft, 9 ft high), once"
