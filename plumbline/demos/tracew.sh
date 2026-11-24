#!/usr/bin/env bash
# A wall traced over a scan is set by the scan's face (DECISIONS.md call 190; the demonstration is
# written in STATUS.md, "A wall traced over a scan, set by its face", before the window was built).
# A made-up scan of a house (tools/made-up-scan.py --scene house) is brought in from the window; the
# Wall tool, over the scan's sheet, lands each click on the face the sheet shows and lays the wall by
# that face: its body on the side away from the room, its centre half the type's thickness beyond the
# line clicked; where the sheet shows both faces of a wall (a partition) the centre goes between them.
# The window says which side it took and offers "Other side". On a second level, the same clicks with
# Centerline chosen lay the centres on the faces, as every wall traced over a scan was laid before.
#
# Run from the repository root:  demos/tracew.sh [folder for the pictures]
#
# The house, as the scan shows it: 30 ft by 16 ft inside, 8 ft high, one face of each outside wall seen
# from indoors, a partition 4 1/2 in thick with both faces seen, 18 ft from the west wall. The scan is
# made up here with a little scatter and lean (as a phone's is), so the faces lie within about half an
# inch of the whole feet below: every figure is checked to a tenth of a foot, a room's floor to a
# square foot. The sheet puts the faces at about east -0.03 and 29.96 ft, north 0.00 and 16.00 ft; the
# partition's two at 17.77 and 18.15 ft, stopping 2 in short of the walls they meet.
#
# Worked by hand (wall types 6 in, partition 4 1/2 in):
#  - Brought in: the card says its sheet shows 6 straight lines, 2 ft or more (four outside faces, the
#    partition's two).
#  - Level ground, Wall tool, 6 in, "Face" lit without a click (over a scan it is Face to begin with),
#    four clicks 3 in inside the corners and the first again: each lands on the corner of the faces. The
#    centres are 3 in outside the faces: corners (-0.25, -0.25), (30.25, -0.25), (30.25, 16.25),
#    (-0.25, 16.25): 30.5 + 16.5 + 30.5 + 16.5 = 94.00 LF. The card says the body is outside the line.
#  - "Other side" once: the centres 3 in inside: (0.25, 0.25) .. (29.75, 15.75): 29.5 + 15.5 + 29.5 + 15.5
#    = 90.00 LF. Again: outside, 94.00 LF as before.
#  - The partition, 4 1/2 in type, a click on one face near the south wall and on the other near the
#    north: the centre between the faces (east 17.96), its ends on the wall faces: 16.00 LF; the card
#    says the scan shows 4 1/2 in between the faces and the type is 4 1/2 in. With the 6 in type (after
#    an Undo) it says the scan shows 4 1/2 in and the type is 6 in; it is laid, not refused.
#  - All walls: 94.00 + 16.00 = 110.00 LF. Floor and two rooms, inside their walls: 480 - 0.375 x 16 =
#    474.00 SF together (284.8 and 189.0 on this scan's faces).
#  - A click 3 ft inside the room, away from every line, stays where it was put and the card says the
#    scan shows no face here; a second 3 ft along: a wall of 3.00 LF laid on the line clicked, and the
#    card says so. (Undone after.)
#  - Level upper, the scan brought in again, Centerline chosen, the same five clicks: the clicks land on the faces and corners, and the centres on the
#    faces, (0, 0) .. (30, 16): 92.00 LF, the room inside 29.5 x 15.5 = 457.25 SF: 22.75 SF short of the
#    480 the faces enclose. That is what the choice is.
#  - Saved, closed, opened again with the scan's own file taken away: the sheet's lines are read again
#    from its picture, and a wall clicked along the south face lands by the face as before.
#  - Every wall by a face is Assumed, "traced by its face over the scan house"; none is Known.
#
# It drives the real program with a real pointer and keyboard on a virtual display, which needs Xvfb,
# xdotool and a Vulkan driver (Mesa's software one will do), and measures the pictures with python and
# PIL. It accepts the sizes the tool offers once (the partition's second drawing) and types over them
# (the first Wall tool, the partition).
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/tracew.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL, numpy" 2>/dev/null || { echo "demos/tracew.sh needs python's PIL and numpy (the made-up scan and the pictures)"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/tracew.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/tracew}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap '[ -z "${TRACEW_KEEP:-}" ] || cp -r "$T" "$TRACEW_KEEP"; rm -rf "$T"' EXIT
J="$T/house.json"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: $3: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }

echo "== The made-up scan, and a job with two levels =="
python3 tools/made-up-scan.py "$T/house.ply" --scene house
"$P" new "$J" "A traced house (not a real job)" >/dev/null
"$P" level "$J" ground --elevation "0 ft" --height "9 ft" --status Assumed --source "to be checked on site" >/dev/null
"$P" level "$J" upper --elevation "9 ft" --height "9 ft" --status Assumed --source "to be checked on site" >/dev/null
touch "$T/started"

echo
echo "== The session: scan in; four walls and the first again, by the face; Other side twice; the partition; rooms; no face; Centerline on the upper level; closed and opened again =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
on "Plan"; sleep 5
park
shot                                    # 01 the level on its plan, nothing under it
on "Scan file"; sleep 3
on "File house.ply"; sleep 40           # the scan comes in; its sheet's faces are read on a thread
park; sleep 6
cp "\$T/map.txt" "\$T/map-brought.txt"
plan_line > "$T/plan.txt"
shot                                    # 02 the sheet under the plan, the card with its lines
tool "Wall" "6 in, 9 ft"                # Wall: the sizes typed over those offered
cp "\$T/map.txt" "\$T/map-wall.txt"
park
shot                                    # 03 the Wall tool over the scan: Face lit, Centerline not
click \$(plan_px 0.25 0.25)             # 3 in inside the southwest corner
click \$(plan_px 29.75 0.25)
cp "\$T/map.txt" "\$T/map-first.txt"
click \$(plan_px 29.75 15.75)
click \$(plan_px 0.25 15.75)
click \$(plan_px 0.25 0.25)             # and the first again
cp "\$T/map.txt" "\$T/map-ring.txt"
park
shot                                    # 04 the four walls, laid by the face
xdotool key F2; sleep 4
cp "$J" "$T/outside.json"
on "Other side"; sleep 4
park
xdotool key F2; sleep 4
cp "$J" "$T/inside.json"
shot                                    # 05 the same walls, the other side of the line clicked
on "Other side"; sleep 4
park
xdotool key F2; sleep 4
cp "$J" "$T/outside2.json"
esc; esc
tool "Wall"                             # Wall again: the sizes offered (6 in, 9 ft) are accepted as they are
click \$(plan_px 9 3)                   # 3 ft inside the room, away from every line
cp "\$T/map.txt" "\$T/map-nofirst.txt"
click \$(plan_px 12 3)
park
shot                                    # 06 a wall where the hand put it, and the card says why
xdotool key F2; sleep 4
cp "$J" "$T/noface.json"
on "Undo"; sleep 4
esc; esc
# The upper level: the scan again, Centerline chosen, the same clicks.
on "Level upper"; sleep 5
on "Scan file"; sleep 3
on "File house.ply"; sleep 40
park; sleep 6
cp "\$T/map.txt" "\$T/map-upper.txt"
tool "Wall"
on "Centerline"; sleep 3
cp "\$T/map.txt" "\$T/map-centre.txt"
park
shot                                    # 07 Centerline lit, Face not
click \$(plan_px 0.25 0.25)
click \$(plan_px 29.75 0.25)
click \$(plan_px 29.75 15.75)
click \$(plan_px 0.25 15.75)
click \$(plan_px 0.25 0.25)
esc; esc
tool "Floor" "4 in"
click \$(plan_px 9 8)
esc
on "Room"
click \$(plan_px 15 8); sleep 2; typed "Hall"
esc
park
shot                                    # 08 the same clicks, centres on the faces
xdotool key F2; sleep 4
cp "$J" "$T/upper.json"
# Closed and opened again with the scan's own file taken away: the partition, whose two faces the sheet's
# picture has to give again, and the rooms.
close_window
rm -f "$T/house.ply"
mkdir -p "$OUT/again"; export PLUMBLINE_SHOTS="$OUT/again"
open_window "$J"
on "Plan"; sleep 5
park; sleep 8
plan_line > "$T/plan-again.txt"
tool "Wall" "6 in, 9 ft"                # the 6 in type for the partition first: the scan shows 4 1/2 in between its faces
click \$(plan_px 17.8 0.4)              # on one face of the partition near the south wall
click \$(plan_px 18.15 15.6)            # and on the other near the north wall
cp "\$T/map.txt" "\$T/map-six.txt"
park
shot                                    # again/01 laid between the faces with the 6 in type: said, not refused
on "Undo"; sleep 4
esc; esc
tool "Wall" "4 1/2 in, 9 ft"            # the type typed over what was offered
click \$(plan_px 17.8 0.4)
click \$(plan_px 18.15 15.6)
esc; esc
park
shot                                    # again/02 the partition, 4 1/2 in between its faces
tool "Floor" "4 in"                     # Floor
click \$(plan_px 9 8)
esc
on "Room"                               # Room
click \$(plan_px 9 8); sleep 2; typed "West room"
click \$(plan_px 24 8); sleep 2; typed "East room"
esc
park
xdotool key F2; sleep 4
cp "$J" "$T/final.json"
shot                                    # again/03 two rooms inside the walls
close_window
SESSION
chmod +x "$T/session.sh"
timeout 3000 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 07 08; do
  [ "$OUT/$n-model.png" -nt "$T/started" ] || { echo "FAIL: no picture $n from this run"; exit 1; }
done
for n in 01 02 03; do
  [ "$OUT/again/$n-model.png" -nt "$T/started" ] || { echo "FAIL: no picture $n of the job opened again"; exit 1; }
done
echo "  11 pictures saved to $OUT (the last three in again/: the program opened again numbers its pictures from 01)"

echo
echo "== What the window said =="
SAID="$T/map.txt.said"
says "$SAID" "Its sheet shows 6 straight lines 2 ft long or more, for the Wall tool to land on." "the scan's lines"
says "$SAID" "Wall: from here, on the scan's face." "the first click"
says "$SAID" "Laid by the scan's face: the wall's body is on the side away from the room, so its center is 3 in off the line clicked (half the type's 6 in). If that is the wrong side, press Other side." "the first wall"
says "$SAID" "Laid by the scan's face: the wall's body is outside the line clicked, so its center is 3 in off the line clicked (half the type's 6 in)." "the ring closed"
says "$SAID" "Laid on the other side: 4 walls of this run now have their bodies inside the line clicked, so their centers are 3 in off the line clicked (half the type's 6 in)." "Other side"
says "$SAID" "Laid on the other side: 4 walls of this run now have their bodies outside the line clicked, so their centers are 3 in off the line clicked (half the type's 6 in)." "Other side again"
says "$SAID" "The scan shows both faces of this wall: 4 1/2 in between the faces, and the type is 6 in. The centerline is laid between the faces." "the partition with the 6 in type"
says "$SAID" "The scan shows both faces of this wall: 4 1/2 in between the faces, and the type is 4 1/2 in. The centerline is laid between the faces." "the partition"
says "$SAID" "Wall: from here. The scan shows no face here: this point stays where you clicked." "a click away from every line"
says "$SAID" "The scan shows no face here: the wall is laid on the line clicked." "a wall away from every line"
if grep -qiE 'tmp|\.ply|hough|ransac|vertex|mesh|point cloud|y up|z up' "$SAID"; then echo "FAIL: what the window said has a software word in it"; grep -iE 'tmp|\.ply|hough|ransac|vertex|mesh|point cloud|y up|z up' "$SAID"; exit 1; fi
echo "    nothing said has a software word in it ($(wc -l < "$SAID") lines said)"
for m in map-brought map-wall map-ring map-centre; do
  [ -s "$T/$m.txt" ] || { echo "FAIL: the session kept no $m"; exit 1; }
done
grep -q $'^model\tCenterline\t' "$T/map-wall.txt" && grep -q $'^model\tFace\t' "$T/map-wall.txt" || { echo "FAIL: the Wall tool's two choices are not on the map"; exit 1; }
grep -q $'^model\tOther side\t' "$T/map-ring.txt" || { echo "FAIL: after the walls were laid by a face, Other side is not on the map"; exit 1; }
! grep -q $'^model\tOther side\t' "$T/map-wall.txt" || { echo "FAIL: Other side is there before any wall was laid by a face"; exit 1; }
echo "    the Wall tool has Centerline and Face; Other side is there once walls are laid by a face, and not before"

echo
echo "== The saved jobs =="
"$P" check "$J"
python3 - "$T" <<'PY'
import json, sys, os
T = sys.argv[1]
def job(name):
    return json.load(open(os.path.join(T, name)))["data"]
def walls(name):
    out = []
    for e in job(name):
        a = e["attributes"]
        if a.get("pl::class") == "Wall":
            w = a["pl::wall"]
            (p, q) = w["line"]["value"]
            out.append({"line": (p, q), "status": w["line"]["status"], "source": w["line"]["source"], "len": a["pl::measure"]["plan_length"]["value"]})
    return out
def near(p, q, tol=0.1):
    return abs(p[0] - q[0]) <= tol and abs(p[1] - q[1]) <= tol
bad = []
def want(what, ok, got):
    print("    %-66s %s%s" % (what, "as wanted" if ok else "WRONG", "" if ok else "  got " + str(got)))
    if not ok:
        bad.append(what)
def ring(name, corners, total, label):
    ws = walls(name)
    ok = len(ws) >= 4
    ls = ws[:4]
    for i, w in enumerate(ls):
        ok = ok and near(w["line"][0], corners[i]) and near(w["line"][1], corners[(i + 1) % 4])
    got = [tuple(round(x, 3) for x in w["line"][0]) for w in ls]
    want("%s: the four corners" % label, ok, got)
    length = sum(w["len"] for w in ls)
    want("%s: %.2f LF" % (label, total), abs(length - total) <= 0.1, round(length, 3))
    return ls
out = ring("outside.json", [(-0.25, -0.25), (30.25, -0.25), (30.25, 16.25), (-0.25, 16.25)], 94.0, "by the face, body outside")
ring("inside.json", [(0.25, 0.25), (29.75, 0.25), (29.75, 15.75), (0.25, 15.75)], 90.0, "Other side, body inside")
ring("outside2.json", [(-0.25, -0.25), (30.25, -0.25), (30.25, 16.25), (-0.25, 16.25)], 94.0, "Other side again, outside")
for w in out:
    want("a wall by the face is Assumed and says which scan: %s" % w["source"], w["status"] == "Assumed" and w["source"] == "traced by its face over the scan house", (w["status"], w["source"]))
ws = walls("noface.json")
nf = ws[4] if len(ws) > 4 else None
want("a wall 3 ft inside the room is laid where the hand put it: 3.00 LF, not by a face", nf is not None and abs(nf["len"] - 3.0) < 0.15 and abs(nf["line"][0][1] - 3.0) < 0.12 and "face" not in nf["source"], nf)
def called_in(data):
    return {path: name for e in data for name, path in e.get("children", {}).items()}
data = job("upper.json")
called = called_in(data)
ws = walls("upper.json")
cl = ws[4:] if len(ws) > 4 else []
want("the upper level: four walls beside the ground's four (the 3 ft wall was taken back)", len(ws) == 8 and len(cl) == 4, len(ws))
if len(cl) == 4:
    corners = [(0, 0), (30, 0), (30, 16), (0, 16)]
    ok = all(near(cl[i]["line"][0], corners[i]) and near(cl[i]["line"][1], corners[(i + 1) % 4]) for i in range(4))
    want("Centerline: the centres are on the faces, corners and all", ok, [w["line"] for w in cl])
    want("Centerline: 92.00 LF", abs(sum(w["len"] for w in cl) - 92.0) < 0.1, sum(w["len"] for w in cl))
    want("Centerline walls are not said to be by a face", all("face" not in w["source"] for w in cl), [w["source"] for w in cl])
    want("Centerline walls are Assumed", all(w["status"] == "Assumed" for w in cl), [w["status"] for w in cl])
hall = [e["attributes"]["pl::measure"]["floor_area"]["value"] for e in data if e["attributes"].get("pl::class") == "Room" and called.get(e["path"]) == "Hall"]
print("    the Hall, inside the walls laid on the faces:", [round(h, 2) for h in hall])
want("Centerline: the room is 457.25 SF, 22.75 SF short of the 480 SF the faces enclose", len(hall) == 1 and abs(hall[0] - 457.25) < 0.8, hall)
data = job("final.json")
called = called_in(data)
ws = walls("final.json")
faced = [w for w in ws if w["source"].startswith("traced by its face")]
part = [w for w in faced if abs(w["line"][0][0] - 17.96) < 0.1 and abs(w["line"][1][0] - 17.96) < 0.1]
want("opened again without the scan's file: five walls by a face, four of them the ring", len(faced) == 5 and len(part) == 1, len(faced))
if part:
    (p, q) = part[0]["line"]
    want("the partition's centre is between its faces (east 17.96), 16.00 LF", abs(p[0] - 17.96) < 0.08 and abs(q[0] - 17.96) < 0.08 and abs(part[0]["len"] - 16.0) < 0.1, (p, q, part[0]["len"]))
    want("the partition is Assumed, by the face", part[0]["status"] == "Assumed" and part[0]["source"] == "traced by its face over the scan house", (part[0]["status"], part[0]["source"]))
want("every wall by a face is Assumed, and none is Known", all(w["status"] == "Assumed" for w in ws), [w["status"] for w in ws])
total = sum(w["len"] for w in faced)
want("all the walls by a face: 94.00 + 16.00 = 110.00 LF", abs(total - 110.0) < 0.15, round(total, 3))
rooms = {called[e["path"]]: e["attributes"]["pl::measure"]["floor_area"]["value"] for e in data if e["attributes"].get("pl::class") == "Room"}
print("    rooms:", {k: round(v, 2) for k, v in rooms.items()})
two = {k: v for k, v in rooms.items() if k != "Hall"}
want("two rooms, 474 SF together inside their walls (480 less the partition)", set(two) == {"West room", "East room"} and abs(sum(two.values()) - 474.0) < 1.0, two)
if bad:
    print("FAIL:", "; ".join(bad)); sys.exit(1)
PY

echo
echo "== The pictures, measured =="
# The Wall tool's two choices are two buttons; the one chosen is lit in the window's blue. Their places come from
# the program's own map, and a pixel just inside each button's edge (clear of its word) is read: the lit one is
# bluer than it is red by more than the other. 03: Face lit over the scan; 07: Centerline lit after it was chosen.
python3 -W ignore -I - "$OUT" "$T" <<'PY'
import sys
from PIL import Image
out, T = sys.argv[1], sys.argv[2]
def place(map_file, name):
    for line in open(map_file):
        f = line.rstrip("\n").split("\t")
        if f[0] == "model" and f[1] == name:
            return float(f[2]), float(f[3])
    sys.exit("FAIL: %s is not on %s" % (name, map_file))
def blueness(img, at):
    x, y = at
    # four pixels in the button's top padding, either side of its middle: blue minus red
    px = img.load()
    v = [px[int(x + dx), int(y - 9)] for dx in (-6, -3, 3, 6)]
    return sum(p[2] - p[0] for p in v) / len(v)
bad = 0
for pic, lit, unlit, m in (("03", "Face", "Centerline", "map-wall.txt"), ("07", "Centerline", "Face", "map-centre.txt")):
    img = Image.open("%s/%s-model.png" % (out, pic)).convert("RGB")
    a, b = blueness(img, place("%s/%s" % (T, m), lit)), blueness(img, place("%s/%s" % (T, m), unlit))
    print("    picture %s: %s is %.0f blue over red, %s is %.0f" % (pic, lit, a, unlit, b))
    if not (a > b + 8):
        bad += 1
if bad:
    sys.exit("FAIL: the chosen way is not lit in the picture")
PY
echo "  PASS: in the window, a scan's sheet gave its 6 straight lines to the Wall tool, which was on Face over it without a click; four clicks 3 in inside the corners and the first again laid four walls whose centres lie 3 in outside the faces (94.00 LF), said which side the body took, and Other side laid them 3 in inside (90.00 LF) and back; a partition clicked on its two faces was laid between them (16.00 LF) with the scan's 4 1/2 in said beside the type's thickness, 4 1/2 in and 6 in; two rooms came to 474 SF inside the walls; a click away from every line stayed where it was put and said the scan showed no face; Centerline, on the upper level, laid the same clicks on the faces (92.00 LF, the room 457.25 SF, 22.75 SF short); opened again without the scan's file a wall was still laid by the face; every wall is Assumed"
