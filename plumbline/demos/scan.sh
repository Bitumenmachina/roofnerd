#!/usr/bin/env bash
# A scan of a room (a PLY point cloud) brought in as a sheet to trace over: to scale, in
# the scan's own place, Assumed (DECISIONS.md call 128; Patrick, 2026-11-17: "It's
# approved for ply import"). Run from the repository root:  demos/scan.sh
#
# The scan is made up by tools/made-up-scan.py (no real scan is in the repository): a
# room 20 ft by 12 ft inside and 8 ft high, a point every 0.05 m on its walls, floor and
# ceiling, in metres with z up, its southwest inside corner at the scan's zero. The
# figures are worked by hand in STATUS.md ("A scan brought in as a sheet to trace").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/room.json"
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -30; exit 1; }; printf '    %s\n' "$2"; }
refused() { local out; if out="$("${@:3}" 2>&1)"; then echo "FAIL: $1: taken, and should have been refused"; exit 1; fi; says "$1" "$2" "$out"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }

echo "== The made-up scan, as text and as bytes, and the same room in feet with y up =="
python3 tools/made-up-scan.py "$T/room.ply"
python3 tools/made-up-scan.py "$T/bytes.ply" --binary
python3 tools/made-up-scan.py "$T/feet.ply" --feet-y-up
"$P" new "$J" "A scanned room (not a real job)" >/dev/null
L="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "8 ft" --status Assumed --source "to be checked on site")")"

echo
echo "== Brought in: what it read, what it took for granted, and the sheet it made =="
IN="$("$P" import scan "$J" "$T/room.ply" --level "$L")"
D="$(made "$IN")"
says "the points, and what was taken for granted" "36,924 points, taken as meters with z up (--units ft or --up y says otherwise)" "$IN"
says "6.096 m by 3.6576 m" "on plan, where it is cut, it is about 20' 0\" by 12' 0\"" "$IN"
says "2.4384 m" "floor to ceiling reads about 8' 0\" on the scan" "$IN"
says "the sheet: (20 + 4) x 24 by (12 + 4) x 24 dots" "576 by 384 dots at 24 to the foot" "$IN"
says "what is not kept" "not kept: each point's red" "$IN"
[ -s "$T/room cut at 4 ft 0 in.png" ] || { echo "FAIL: no picture of the cut beside the job"; exit 1; }
"$P" check "$J"
DR="$("$P" drawings "$J")"
says "to scale: 576 dots are 24 ft" "24 ft between (0, 0) and (576, 0) on its page: 24 dots to the foot" "$DR"
says "its source, with what was taken for granted" "scan room.ply, cut 4' 0\" above its floor (taken as meters with z up)" "$DR"
says "Assumed" "~ room [$D]" "$DR"

echo
echo "== Traced over it at the room's four inside corners: 20 + 12 + 20 + 12 = 64.00 LF =="
WT="$(made "$("$P" type "$J" wall "wall 4 in" --thickness "4 in" --status Assumed --source "to be checked on site")")"
"$P" wall "$J" "$L" "$WT" --points "48,48 528,48 528,336 48,336" --closed --height "8 ft" --on "$D" >/dev/null
"$P" check "$J"
says "the walls" "walls 4            64.00 LF" "$("$P" takeoff "$J")"
python3 - "$J" "$D" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))["data"]
walls = [e["attributes"]["pl::wall"] for e in d if "pl::wall" in e["attributes"]]
ends = sorted({tuple(round(c, 6) for c in p) for w in walls for p in w["line"]["value"]})
assert ends == [(0.0, 0.0), (0.0, 12.0), (20.0, 0.0), (20.0, 12.0)], ends
assert all(w["line"]["status"] == "Assumed" and sys.argv[2] in w["line"]["source"] or "room" in w["line"]["source"] for w in walls), [w["line"]["source"] for w in walls]
print("    their corners on the plan: (0, 0), (20, 0), (20, 12), (0, 12); each Assumed, traced from the drawing")
PY

echo
echo "== The same scan as bytes: the same sheet, dot for dot; and in feet with y up: the same room =="
cp "$J" "$T/again.json"; "$P" new "$T/b.json" "bytes (not a real job)" >/dev/null
LB="$(made "$("$P" level "$T/b.json" ground --elevation "0 ft" --height "8 ft" --status Assumed --source "x")")"
"$P" import scan "$T/b.json" "$T/bytes.ply" --level "$LB" >/dev/null
python3 - "$T/room cut at 4 ft 0 in.png" "$T/bytes cut at 4 ft 0 in.png" <<'PY'
import sys
from PIL import Image
a, b = (Image.open(p).convert("L") for p in sys.argv[1:3])
assert a.size == b.size == (576, 384) and a.tobytes() == b.tobytes(), "the two sheets differ"
dark = sum(1 for v in a.tobytes() if v < 128)
# The cut: four walls' points at one height. On the sheet they lie on the room's inside
# line, 48 dots in from each edge: dark dots there and none in the middle of the room.
assert a.getpixel((288, 384 - 1 - 48)) < 128 and a.getpixel((48, 192)) < 128 and a.getpixel((288, 192)) > 128, "the walls are not where the room's are"
print(f"    the same sheet from text and from bytes, 576 by 384, {dark} dark dots, the walls on the room's line and the room clear")
PY
FT="$("$P" import scan "$T/b.json" "$T/feet.ply" --level "$LB" --units ft --up y)"
says "the same room, told it is in feet with y up" "taken as feet with y up" "$FT"
says "the same size" "it is about 20' 0\" by 12' 0\"" "$FT"
says "the same height" "floor to ceiling reads about 8' 0\" on the scan" "$FT"

echo
echo "== A rough scan, as a phone makes one: scattered points each a little out, furniture, strays outside, the scan leaning =="
# tools/made-up-scan.py --rough: two rooms, 30 ft by 20 ft and 8 ft high; a partition 18 ft
# from the west wall with a doorway from 9 ft to 12 ft north; a window in the south wall
# from 8 ft to 13 ft east; cabinets 7 ft tall against the north wall from 20 ft to 29 ft
# east, 2 ft deep; a table and a sofa under 4 ft; each point out by about 1.5 cm; one
# point in fifty a stray up to 6 m outside; the whole scan leaning half a degree, which is
# tan(0.5 deg) x 120 = 1.05 in in 10 ft. What a person must be told, and how near:
python3 tools/made-up-scan.py "$T/house.ply" --rough --binary
"$P" new "$T/h.json" "A rough scan (not a real job)" >/dev/null
LH="$(made "$("$P" level "$T/h.json" ground --elevation "0 ft" --height "8 ft" --status Assumed --source "to be checked on site")")"
RO="$("$P" import scan "$T/h.json" "$T/house.ply" --level "$LH")"
echo "$RO" | sed 's/^/    /' | head -7
python3 - "$T/h.json" "$T/house cut at 4 ft 0 in.png" <<PY2
import json, re, sys
from PIL import Image
said = """$RO"""
def inches(text):
    m = re.fullmatch(r"(\d+)' (\d+)(?: (\d+)/(\d+))?\"", text)
    return int(m[1]) * 12 + int(m[2]) + (int(m[3]) / int(m[4]) if m[3] else 0)
wide, deep = re.search(r"it is about (\d+' [\d /]+\") by (\d+' [\d /]+\")", said).groups()
# The size: 30 ft by 20 ft, and no more than 3 in over (the fuzz of two walls). With
# the strays counted in, it read 69 ft by 59 ft.
assert 0 <= inches(wide) - 360 <= 3 and 0 <= inches(deep) - 240 <= 3, (wide, deep)
# The height: 8 ft to the inch, though the scan leans (read from the floor's and the
# ceiling's edges it was 8 ft 3 in).
assert "floor to ceiling reads about 8' 0\"" in said, said
lean = re.search(r"it leaned ([\d /]+) in in 10 ft and was levelled by its own floor", said)[1]
lean = sum(eval(p) for p in lean.split())
assert 0.85 <= lean <= 1.25, lean
strays = int(re.search(r"left off the sheet: ([\d,]+) stray points", said)[1].replace(",", ""))
assert strays >= 50, strays
print(f"    said: about {wide} by {deep}; 8' 0\" high; leaned {lean:.2f} in in 10 ft (1.05 by hand); {strays} strays left off")
# The sheet: about (30 + 4) x 24 = 816 by (20 + 4) x 24 = 576 dots, and on it, at the
# places worked from the house as made: wall where there is wall, none where there is not.
d = json.load(open(sys.argv[1]))["data"]
dr = next(e["attributes"]["pl::drawing"] for e in d if "pl::drawing" in e["attributes"])
cx, cy = dr["at"]["value"]
im = Image.open(sys.argv[2]).convert("L")
w, h = im.size
assert abs(w - 816) <= 30 and abs(h - 576) <= 45, (w, h)
def dark(e0, n0, e1, n1):  # how many dark dots in a box given in feet on the plan
    x0, x1 = sorted((round((e0 - cx) * 24), round((e1 - cx) * 24)))
    y0, y1 = sorted((h - 1 - round((n1 - cy) * 24), h - 1 - round((n0 - cy) * 24)))
    return sum(1 for v in im.crop((x0, y0, x1 + 1, y1 + 1)).tobytes() if v < 128)
checks = [
    ("the west wall", dark(-0.25, 4, 0.25, 16), 150, None),
    ("the south wall, east of the window", dark(14, -0.25, 17, 0.25), 40, None),
    ("the window in the south wall, 8 ft to 13 ft east: no points", dark(8.5, -0.25, 12.5, 0.25), None, 8),
    ("the partition, south of its doorway", dark(17.75, 2, 18.25, 8), 80, None),
    ("the doorway in the partition, 9 ft to 12 ft north: clear", dark(17.75, 9.4, 18.25, 11.6), None, 4),
    ("the front of the tall cabinets (they stand above the cut)", dark(21, 17.75, 28, 18.25), 20, None),
    ("the middle of the west room (a table and a sofa below the cut): clear", dark(3, 3, 15, 13), None, 12),
]
for what, n, least, most in checks:
    ok = (least is None or n >= least) and (most is None or n <= most)
    print("    %-72s %4d dark dots  %s" % (what, n, "as wanted" if ok else "FAIL"))
    assert ok, (what, n)
PY2
"$P" check "$T/h.json"

echo
echo "== Refused, each in a sentence, and the job left as it was =="
cp "$J" "$T/before.json"
printf 'solid not a scan\n' > "$T/not.ply"
printf 'ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nend_header\n1\n' > "$T/nox.ply"
head -c 4000 "$T/room.ply" > "$T/short.ply"
refused "not a PLY" "it is not a PLY file" "$P" import scan "$J" "$T/not.ply" --level "$L"
refused "no x, y, z" "its points have no x, y and z" "$P" import scan "$J" "$T/nox.ply" --level "$L"
refused "cut short" "it is cut short: it says 36924 points and holds" "$P" import scan "$J" "$T/short.ply" --level "$L"
refused "units" "--units \"yd\": a scan is in m or ft" "$P" import scan "$J" "$T/room.ply" --level "$L" --units yd
refused "up" "--up \"x\": up is z or y" "$P" import scan "$J" "$T/room.ply" --level "$L" --up x
refused "no level said" "say which level the scan is of" "$P" import scan "$J" "$T/room.ply"
cmp -s "$J" "$T/before.json" || { echo "FAIL: a refused scan changed the job"; exit 1; }
echo "    the job is as it was after each"
echo "PASS: a scan brought in as a sheet to trace, to scale and in its own place, Assumed, to the figures worked by hand"
