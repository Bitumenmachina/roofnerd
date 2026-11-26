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
# Call 193 (the window judged, 2026-11-24), points B to G, in a second job of three levels on the same scan (see STATUS.md,
# "The window judged (2026-11-24), and its findings fixed"):
#  - D, ground: the ring by the face (94.00 LF), Esc twice, "Other side" is still on the card: 90.00 LF; Undo: 94.00; "Other
#    side" again: 90.00 LF; a wall of the ring deleted and "Other side" is gone from the card.
#  - B, upper: two walls laid by the face (north 0 ft where they lie 3 in south of the faces), then Centerline pressed: both
#    walls lie on the faces' own lines, the card says 2 walls moved 3 in, and the first wall's end is the second's start to
#    the hundredth of a foot.
#  - C, loft: a wall down the west face to the southwest corner, then 10 ft typed going east by the face: the typed 10.00 ft
#    runs from the click 3 in inside the corner; the wall runs back to the corner of the outside faces (3 in out each way,
#    the faces being 0.03 ft out on the west) and 10.25 ft to the click's end: 10.00 + 0.28 + 0.25 = 10.53 LF. The card says
#    both. A wall drawn over it is refused without a stray colon. In D's ring, a click on the middle of the south wall
#    (3 in inside the face) says it stands on the scan's face, and not that the scan shows no face.
#  - F, in a fourth level on a scan of a house with partitions and doorways (made-up-scan.py --scene rooms3: partitions at 10 and 20 ft
#    east, the first with its doorway 5 to 8 ft north): a wall traced across the 3 ft doorway says the scan shows no face along 3 ft
#    of it; and the Job panel says the floor goes out to the walls' center lines beside rooms measured inside them.
#  - G: with the Wall tool up over the scan, the traced wall is darker than the cut beside it, the cut is no longer black,
#    and the scan's faces are drawn as fine lines.
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
xdotool key F2; drawn_for 3 4
cp "$J" "$T/outside.json"
on "Other side"; sleep 4
park
xdotool key F2; drawn_for 3 4
cp "$J" "$T/inside.json"
shot                                    # 05 the same walls, the other side of the line clicked
on "Other side"; sleep 4
park
xdotool key F2; drawn_for 3 4
cp "$J" "$T/outside2.json"
esc; esc
tool "Wall"                             # Wall again: the sizes offered (6 in, 9 ft) are accepted as they are
click \$(plan_px 9 3)                   # 3 ft inside the room, away from every line
cp "\$T/map.txt" "\$T/map-nofirst.txt"
click \$(plan_px 12 3)
park
shot                                    # 06 a wall where the hand put it, and the card says why
xdotool key F2; drawn_for 3 4
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
xdotool key F2; drawn_for 3 4
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
xdotool key F2; drawn_for 3 4
cp "$J" "$T/final.json"
cp "\$T/map.txt" "\$T/map-final.txt"
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
grep -q $'^note\tFloor\tout to the walls\' center lines' "$T/map-final.txt" || { echo "FAIL: the Job panel's Floor line does not say which line the floor goes to"; grep -P '^note' "$T/map-final.txt" || echo "no note"; exit 1; }
echo "    the Job panel's Floor line says: out to the walls' center lines (beside Rooms, measured inside their walls)"

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

echo
echo "== A second job on the same scan, three levels: Other side kept (D), a run laid one way (B), a typed length (C), the cut seen through (G) =="
J2="$T/more.json"
python3 tools/made-up-scan.py "$T/house.ply" --scene house
"$P" new "$J2" "A traced house, again (not a real job)" >/dev/null
"$P" level "$J2" ground --elevation "0 ft" --height "9 ft" --status Assumed --source "to be checked on site" >/dev/null
"$P" level "$J2" upper --elevation "9 ft" --height "9 ft" --status Assumed --source "to be checked on site" >/dev/null
"$P" level "$J2" loft --elevation "18 ft" --height "9 ft" --status Assumed --source "to be checked on site" >/dev/null
"$P" level "$J2" attic --elevation "27 ft" --height "9 ft" --status Assumed --source "to be checked on site" >/dev/null
python3 tools/made-up-scan.py "$T/rooms3.ply" --scene rooms3
cp "$T/map.txt.said" "$T/said-first.txt"
mkdir -p "$OUT/more"
touch "$T/started2"
cat > "$T/session2.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT/more" ICD="$ICD" LOG="$T/app2.log"
. "$PWD/demos/window.sh"
scan_in() { on "Scan file"; sleep 3; on "File house.ply"; sleep 40; park; sleep 6; }
open_window "$J2"
on "Plan"; sleep 5
park
scan_in
tool "Wall" "6 in, 9 ft"
# D: the ring by the face, the tool put down, "Other side" on the card
click \$(plan_px 0.25 0.25)
click \$(plan_px 29.75 0.25)
click \$(plan_px 29.75 15.75)
click \$(plan_px 0.25 15.75)
click \$(plan_px 0.25 0.25)
park
shot                                    # 01 the ring, the Wall tool up over the scan: the cut light, the walls heavier, the faces fine
xdotool key F2; drawn_for 3 4; cp "$J2" "$T/d-ring.json"
esc; esc
park
cp "\$T/map.txt" "\$T/map-down.txt"
shot                                    # 02 the tool put down: "Other side" is on the card of the walls just laid
on "Other side"; sleep 4
park
xdotool key F2; drawn_for 3 4; cp "$J2" "$T/d-turned.json"
shot                                    # 03 turned with the tool down
xdotool key z; drawn_for 3 5
park
xdotool key F2; drawn_for 3 4; cp "$J2" "$T/d-undone.json"
cp "\$T/map.txt" "\$T/map-undone.txt"
on "Other side"; sleep 4
park
xdotool key F2; drawn_for 3 4; cp "$J2" "$T/d-again.json"
shot                                    # 04 Undo, and Other side again
# a wall of the ring taken out: the walls are changed, and Other side goes from the card
at 15 15.75; sleep 2                    # the north wall, now 3 in inside the face
xdotool key Delete; drawn_for 3 4
park
cp "\$T/map.txt" "\$T/map-changed.txt"
shot                                    # 05 a wall deleted: no Other side
# a click on the middle of a wall that stands on the face (the south wall, 3 in inside it): not "no face here"
tool "Wall"
click \$(plan_px 5 0.25)
esc; esc
# B: the upper level: two walls by the face, then Centerline
on "Level upper"; sleep 5
scan_in
tool "Wall" "6 in, 9 ft"
click \$(plan_px 0.25 0.25)
click \$(plan_px 29.75 0.25)
click \$(plan_px 29.75 15.75)
park
xdotool key F2; drawn_for 3 4; cp "$J2" "$T/b-face.json"
on "Centerline"; sleep 4
park
xdotool key F2; drawn_for 3 4; cp "$J2" "$T/b-centre.json"
shot                                    # 06 the two walls laid again on the faces' own lines
# C: the loft: a length typed along the face
on "Level loft"; sleep 5
scan_in
tool "Wall" "6 in, 9 ft"
on "Face"; sleep 3
click \$(plan_px 0.25 15.75)            # the northwest corner, 3 in inside it
click \$(plan_px 0.25 0.25)             # down the west face to the southwest corner
len \$(plan_px 12 0.25) "10 ft" yes     # 07 ten feet typed, going east
park
xdotool key F2; drawn_for 3 4; cp "$J2" "$T/c-typed.json"
shot                                    # 08 the wall, and what the card says of it
esc
click \$(plan_px 5 0.25)                # on the wall just laid
click \$(plan_px 9 0.25)                # a wall drawn over it
park
shot                                    # 09 refused
esc; esc
# F: the attic, a scan of a house with two partitions, each with a 3 ft doorway; one traced from near its south end to near
# its north, across the doorway (5 to 8 ft north)
on "Level attic"; sleep 5
on "Scan file"; sleep 3; on "File rooms3.ply"; sleep 40; park; sleep 6
tool "Wall" "6 in, 9 ft"
click \$(plan_px 9.8 0.4)
click \$(plan_px 10.2 17.6)
park
shot                                    # 10 the partition across its doorway
esc; esc
close_window
SESSION
chmod +x "$T/session2.sh"
timeout 3000 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session2.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the second session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app2.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app2.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 07 08 09 10; do
  [ "$OUT/more/$n-model.png" -nt "$T/started2" ] || { echo "FAIL: no picture $n of the second job"; exit 1; }
done
echo "  ten more pictures saved to $OUT/more"
echo
echo "== What the window said, the second job =="
SAID2="$T/map.txt.said"
tail -n +$(( $(wc -l < "$T/said-first.txt") + 1 )) "$SAID2" > "$T/said2.txt"
SAID="$T/said2.txt"
sed 's/^/    | /' "$SAID"
says "$SAID" "Laid on the other side: 4 walls of this run now have their bodies inside the line clicked" "D: Other side, the tool put down"
says "$SAID" "Laid again on the centerline: 2 walls of this run moved up to 3 in, and the corners still meet." "B: Centerline in the middle of a run"
says "$SAID" "10' 0\" was typed along the scan's face; the wall is 10' 3\" to its corner." "C: a typed length"
grep -qF "which stands on the scan's face" "$SAID" || { echo "FAIL: a click on a wall already laid on the face was not said to be on it"; cat "$SAID"; exit 1; }
printf '    %s\n' "which stands on the scan's face"
grep -qE "The scan shows no face along (2' 1[01]|3' [01])[^ ]* of it: a doorway, or what the scanner did not see\." "$SAID" || { echo "FAIL: F: the partition traced across its doorway does not say how long the scan shows no face along it"; grep -F "both faces" "$SAID"; exit 1; }
printf '    %s\n' "The scan shows no face along 3 ft of it: a doorway, or what the scanner did not see."
if grep -qF "the new wall: lies" "$SAID"; then echo "FAIL: the stray colon is still there"; exit 1; fi
grep -qF "the new wall lies along" "$SAID" || { echo "FAIL: a wall drawn over another says nothing of it"; cat "$SAID"; exit 1; }
printf '    %s\n' "the new wall lies along"
# a wall's card that gave its length before the next wall joined its corner says what it has now
grep -qE "wall[- ][0-9]+ is now [0-9]+' [0-9 /]+\" long, where it meets this wall" "$SAID" || { echo "FAIL: the first wall's length after the next joined it is not said"; cat "$SAID"; exit 1; }
printf '    %s\n' "(the wall before it is told its length when the next wall joins its corner)"
[ "$(grep -c "which stands on the scan's face" "$SAID")" -ge 1 ] && ! grep -qF "The scan shows no face here" "$SAID" || { echo "FAIL: a face a laid wall stands on is said to be no face"; grep -F "no face here" "$SAID"; exit 1; }
printf '    %s\n' "(\"no face here\" is not said of a face a laid wall stands on)"
grep -q $'^model\tOther side\t' "$T/map-down.txt" || { echo "FAIL: with the tool put down, Other side is not on the card"; exit 1; }
grep -q $'^model\tOther side\t' "$T/map-undone.txt" || { echo "FAIL: after an Undo of the turn, Other side is not on the card"; exit 1; }
! grep -q $'^model\tOther side\t' "$T/map-changed.txt" || { echo "FAIL: Other side is on the card after one of the walls was taken out"; exit 1; }
echo "    Other side: on the card with the tool put down, still there after an Undo of the turn, gone once a wall of the run is taken out"
python3 - "$T" <<'PY'
import json, sys, os
T = sys.argv[1]
def walls(name):
    out = []
    for e in json.load(open(os.path.join(T, name)))["data"]:
        a = e["attributes"]
        if a.get("pl::class") == "Wall":
            (p, q) = a["pl::wall"]["line"]["value"]
            out.append({"p": p, "q": q, "len": a["pl::measure"]["plan_length"]["value"], "path": e["path"]})
    return out
bad = []
def want(what, ok, got):
    print("    %-78s %s%s" % (what, "as wanted" if ok else "WRONG", "" if ok else "  got " + str(got)))
    if not ok: bad.append(what)
def total(name, level=None):
    return sum(w["len"] for w in walls(name))
want("D: the ring by the face, 94.00 LF", abs(total("d-ring.json") - 94.0) < 0.1, total("d-ring.json"))
want("D: Other side with the tool put down, 90.00 LF", abs(total("d-turned.json") - 90.0) < 0.1, total("d-turned.json"))
want("D: Undo of the turn, 94.00 LF again", abs(total("d-undone.json") - 94.0) < 0.1, total("d-undone.json"))
want("D: Other side after the Undo, 90.00 LF", abs(total("d-again.json") - 90.0) < 0.1, total("d-again.json"))
fa = walls("b-face.json")[4:] if len(walls("b-face.json")) > 4 else []
ce = walls("b-centre.json")[4:] if len(walls("b-centre.json")) > 4 else []
# The ground's ring is four walls; the upper level's two follow.
fa = walls("b-face.json")[-2:]; ce = walls("b-centre.json")[-2:]
want("B: by the face the south wall lies 3 in south of the face (north -0.25 ft)", len(fa) == 2 and abs(fa[0]["p"][1] + 0.25) < 0.1 and abs(fa[0]["q"][1] + 0.25) < 0.1, fa)
want("B: then Centerline: the south wall lies on the face's own line (north 0 ft)", len(ce) == 2 and abs(ce[0]["p"][1]) < 0.1 and abs(ce[0]["q"][1]) < 0.1, ce)
want("B: and the east wall on its own (east 30 ft)", len(ce) == 2 and abs(ce[1]["p"][0] - 30.0) < 0.1 and abs(ce[1]["q"][0] - 30.0) < 0.1, ce)
want("B: the first wall's end is the second's start, to the hundredth of a foot", len(ce) == 2 and abs(ce[0]["q"][0] - ce[1]["p"][0]) < 0.005 and abs(ce[0]["q"][1] - ce[1]["p"][1]) < 0.005, ce)
c = walls("c-typed.json")[-1]
want("C: ten feet typed along the face: the wall is 10.25 LF (3 in more than typed)", abs(c["len"] - 10.25) < 0.05, (c["len"], c["p"], c["q"]))
if bad:
    print("FAIL:", "; ".join(bad)); sys.exit(1)
PY
echo
echo "== The cut seen through, the faces fine (G), measured on picture 04 of the first job: the Wall tool up over the scan, the ring laid =="
python3 -W ignore -I - "$OUT" "$T" <<'PY'
import sys
import numpy as np
from PIL import Image
out, T = sys.argv[1], sys.argv[2]
x0, y0, k = map(float, open(T + "/plan.txt").read().split())
img = np.asarray(Image.open(out + "/04-model.png").convert("RGB")).astype(float)
def px(e, n): return int(round(x0 + e * k)), int(round(y0 - n * k))
def lum(a): return 0.2126 * a[..., 0] + 0.7152 * a[..., 1] + 0.0722 * a[..., 2]
def patch(e, n, r=2):
    x, y = px(e, n); return lum(img[y - r:y + r + 1, x - r:x + r + 1]).mean()
wall = patch(5, -0.25)       # the traced south wall's own middle, over the scan's cut
cut = patch(17.96, 8)        # the partition's cut, not yet traced
print("    luminance: the traced wall %.0f, the cut beside it %.0f (white is 255, the old cut was black)" % (wall, cut))
bad = []
if not (wall + 25 < cut): bad.append("a place on the traced wall is not darker than the cut beside it")
if not (cut > 110): bad.append("the cut is still heavy: luminance %.0f" % cut)
# the faces are fine lines: some dot in a few dots' reach of the partition's west face is the accent blue
fx, fy = px(17.77, 8)
def bluest(cx): return (img[fy - 40:fy + 41, cx - 3:cx + 4, 2] - img[fy - 40:fy + 41, cx - 3:cx + 4, 0]).max()
blue, beside = bluest(fx), max(bluest(fx - 30), bluest(fx + 30))
print("    the most blue over red along the partition's west face: %.0f, and in a strip of the cut 30 dots to either side: %.0f" % (blue, beside))
if not (blue > beside + 30): bad.append("no fine line is drawn along the scan's face")
if bad:
    print("FAIL:", "; ".join(bad)); sys.exit(1)
PY

echo "  PASS: in the window, a scan's sheet gave its 6 straight lines to the Wall tool, which was on Face over it without a click; four clicks 3 in inside the corners and the first again laid four walls whose centres lie 3 in outside the faces (94.00 LF), said which side the body took, and Other side laid them 3 in inside (90.00 LF) and back; a partition clicked on its two faces was laid between them (16.00 LF) with the scan's 4 1/2 in said beside the type's thickness, 4 1/2 in and 6 in; two rooms came to 474 SF inside the walls; a click away from every line stayed where it was put and said the scan showed no face; Centerline, on the upper level, laid the same clicks on the faces (92.00 LF, the room 457.25 SF, 22.75 SF short); opened again without the scan's file a wall was still laid by the face; every wall is Assumed"
