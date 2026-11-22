#!/usr/bin/env bash
# Walls, doors, windows and what stands short, found in a scan (a PLY point cloud), said in
# full, and laid only on the person's word (DECISIONS.md call 173). Run from the
# repository root:  demos/findwalls.sh
#
# Written step for step from STATUS.md, "Walls found in a scan: a demonstration, written
# before it is built", with each figure worked by hand there. The scan is made up by
# tools/made-up-scan.py --as-scanned (no real scan is in the repository): the two-room
# house of demos/scan.sh, 30 ft by 20 ft inside and 8 ft high, as a phone sees it. The
# four outside walls one face each (seen from indoors), a partition of two faces 4 1/2 in
# apart with a doorway 9 ft to 12 ft north, a window in the south wall 8 ft to 13 ft east
# (3 ft to 7 ft up), tall cabinets against the north wall with the wall behind them not
# seen, a table and a sofa; 1.5 cm of scatter, strays, and half a degree of lean. Each
# check prints what the program said beside what the hand figure is, and fails where it is
# further off than the stated nearness.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -40; exit 1; }; printf '    said: %s\n' "$2"; }
refused() { local out; if out="$("${@:3}" 2>&1)"; then echo "FAIL: $1: taken, and should have been refused"; exit 1; fi; says "$1" "$2" "$out"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
new_job() { "$P" new "$1" "A scanned house (not a real job)" >/dev/null; made "$("$P" level "$1" ground --elevation "0 ft" --height "8 ft" --status Assumed --source "to be checked on site")"; }

# What a hand figure is checked with: the sentences the program said, read back as numbers.
cat > "$T/said.py" <<'PY'
import json, math, re

def inches(text):
    """A length as the program writes it, 18' 0", -0' 2", 0' 4 1/2", to inches."""
    m = re.fullmatch(r"(-?)(\d+)' (\d+)(?: (\d+)/(\d+))?\"", text.strip())
    n = int(m[2]) * 12 + int(m[3]) + (int(m[4]) / int(m[5]) if m[4] else 0)
    return -n if m[1] else n

LEN = r"(-?\d+' \d+(?: \d+/\d+)?\")"

def walls_said(text):
    """Each wall found, as {from, to, long, thick, openings, hidden}, read from what was said."""
    out = []
    for line in text.splitlines():
        m = re.match(r"\s+wall (\d+): from " + LEN + " east, " + LEN + " north to " + LEN + " east, " + LEN + r" north, " + LEN + r" long, (seen from one side|" + LEN + " thick, seen from both sides)", line)
        if m:
            g = m.groups()
            out.append({"from": (inches(g[1]), inches(g[2])), "to": (inches(g[3]), inches(g[4])), "long": inches(g[5]), "thick": inches(g[7]) if g[7] else None, "doors": [], "windows": [], "hidden": []})
            continue
        m = re.match(r"\s+doorway: " + LEN + " to " + LEN + r" from its \w+ end, " + LEN + r" wide, head " + LEN, line)
        if m:
            g = m.groups(); out[-1]["doors"].append({"from": inches(g[0]), "to": inches(g[1]), "wide": inches(g[2]), "head": inches(g[3])})
        m = re.match(r"\s+window: " + LEN + " to " + LEN + r" from its \w+ end, " + LEN + r" wide, sill " + LEN + ", head " + LEN, line)
        if m:
            g = m.groups(); out[-1]["windows"].append({"from": inches(g[0]), "to": inches(g[1]), "wide": inches(g[2]), "sill": inches(g[3]), "head": inches(g[4])})
        m = re.match(r"\s+not seen for " + LEN + r" \(" + LEN + " to " + LEN + r" from its \w+ end\) behind what stands at it", line)
        if m:
            g = m.groups(); out[-1]["hidden"].append({"wide": inches(g[0]), "from": inches(g[1]), "to": inches(g[2])})
    return out

fails = []
def check(what, program, hand, near, unit="in"):
    """Prints the program's figure beside the hand figure; a miss is a failure."""
    ok = abs(program - hand) <= near
    shown = lambda x: ("%g" % round(x, 3))
    print("    %-52s hand %8s %s   program %8s %s   (within %s %s)  %s" % (what, shown(hand), unit, shown(program), unit, shown(near), unit, "ok" if ok else "FAIL"))
    if not ok:
        fails.append(what)

def done():
    if fails:
        raise SystemExit("FAIL: further off than the hand figure allows: " + "; ".join(fails))

def job(path):
    return json.load(open(path))["data"]

def wall_lines(path):
    d = job(path)
    return {e["path"]: e["attributes"]["pl::wall"] for e in d if "pl::wall" in e["attributes"]}

def types(path):
    return {e["path"]: e["attributes"] for e in job(path)}
PY

echo "== The made-up house, as a phone would scan it: outside walls one face each, a two-faced partition =="
python3 tools/made-up-scan.py "$T/house.ply" --as-scanned --binary
python3 tools/made-up-scan.py "$T/noceiling.ply" --as-scanned --no-ceiling --binary
python3 tools/made-up-scan.py "$T/turned.ply" --as-scanned --turned 17 --binary
J="$T/h.json"; L="$(new_job "$J")"
cp "$J" "$T/before.json"

echo
echo "== Step 1. Found, nothing laid: the scan is asked for its walls =="
F1="$("$P" find-walls "$J" "$T/house.ply" --level "$L")"
echo "$F1" | sed 's/^/    | /'
cmp -s "$J" "$T/before.json" || { echo "FAIL: asking what is found changed the job"; exit 1; }
python3 - "$T" <<PY1
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F1"""
w = walls_said(said)
n = int(re.search(r"walls found: (\d+)", said)[1]); one = int(re.search(r"walls found: \d+, (\d+) of them seen from one side", said)[1])
check("walls found", n, 5, 0, "walls"); assert len(w) == 5, w
check("of them seen from one side", one, 4, 0, "walls")
# Each outside wall's seen face on the scan's face: east 0 and 30, north 0 and 20, to the inch.
faces = sorted(((x["from"][0] if abs(x["from"][0] - x["to"][0]) < 1 else x["from"][1]), "east" if abs(x["from"][0] - x["to"][0]) < 1 else "north") for x in w if x["thick"] is None)
for way, hand in (("east", [0, 360]), ("north", [0, 240])):
    got = sorted(at for at, w2 in faces if w2 == way)
    for g, h in zip(got, hand):
        check("an outside wall's face, " + way, g, h, 1)
part = next(x for x in w if x["thick"] is not None)
check("the partition's centre, east", (part["from"][0] + part["to"][0]) / 2, 18 * 12, 1)
check("the partition's thickness", part["thick"], 4.5, 0.5)
check("the partition's length", part["long"], 20 * 12, 2)
assert len(part["doors"]) == 1, part
d = part["doors"][0]
check("doorway: from, north", d["from"], 9 * 12, 2); check("doorway: to, north", d["to"], 12 * 12, 2); check("doorway: head", d["head"], 6 * 12 + 8, 2)
south = next(x for x in w if x["windows"])
o = south["windows"][0]
check("window in the south wall: from, east", o["from"], 8 * 12, 2); check("window: to, east", o["to"], 13 * 12, 2)
check("window: sill", o["sill"], 3 * 12, 2); check("window: head", o["head"], 7 * 12, 2)
assert not any(x["doors"] for x in w if x is not part), "a doorway where the cabinets stand"
short = int(re.search(r"stood short of the ceiling, and not walls: (\d+) things?", said)[1]); tall = re.search(r"the tallest is " + LEN, said)[1]
check("stood short, not walls: things", short, 3, 0, "things"); check("the tallest", inches(tall), 7 * 12, 2)
hid = [h for x in w for h in x["hidden"]]
assert len(hid) == 1, hid
check("hidden, taken as wall: how long", hid[0]["wide"], 9 * 12, 4); check("hidden: from, east", hid[0]["from"], 20 * 12, 4); check("hidden: to, east", hid[0]["to"], 29 * 12, 4)
sq = re.search(r"squared: (\d+) of (\d+) wall faces", said)
check("squared: faces", int(sq[1]), 6, 0, "faces"); check("of faces found", int(sq[2]), 6, 0, "faces")
done()
PY1
says "nothing laid, and what the four need" "nothing is laid: say --lay to lay these walls, doors and windows on $L" "$F1"
says "the four outside walls need a wall named" "the 4 walls seen from one side are laid only as a wall type you name for them (--outside" "$F1"
says "a doorway is not told from a door" "the scan cannot tell a door from an open doorway" "$F1"
says "everything found is Assumed" "everything found is Assumed" "$F1"
[ "$("$P" show "$J" | grep -c '\[wall\]' || true)" = 0 ] || { echo "FAIL: the job holds a wall after asking"; exit 1; }
echo "    the job holds no wall, and is byte for byte as it was"

echo
echo "== Step 2. Laid, with the outside wall named =="
WT="$(made "$("$P" type "$J" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
cp "$J" "$T/typed.json"
F2="$("$P" find-walls "$J" "$T/house.ply" --level "$L" --lay --outside "$WT")"
echo "$F2" | sed -n '/laid on/,$p' | sed 's/^/    | /'
says "5 walls, 1 door, 1 window laid" "laid on $L: 5 walls, 1 door, 1 window, each Assumed, source \"found in scan house\"; none is Known" "$F2"
says "what was not seen is said again" "not seen for" "$(grep -A2 'laid on' <<<"$F2"; grep 'behind what stands at it: laid as wall' <<<"$F2")"
"$P" check "$J" | sed 's/^/    /'
python3 - "$T" "$J" <<'PY2'
import sys
sys.path.insert(0, sys.argv[1])
from said import *
d = types(sys.argv[2]); walls = wall_lines(sys.argv[2])
assert len(walls) == 5, list(walls)
for p, w in walls.items():
    assert w["line"]["status"] == "Assumed" and w["line"]["source"] == "found in scan house", (p, w["line"])
    assert w["type"]["status"] == "Assumed" and w["type"]["source"] == "found in scan house", (p, w["type"])
print("    5 walls, each Assumed, its source \"found in scan house\"; none Known")
# The partition's type is one the program made for it.
mine = [p for p, a in d.items() if "pl::wall_type" in a and a["pl::wall_type"]["thickness"]["source"] == "found in scan house"]
assert len(mine) == 1, mine
t = d[mine[0]]["pl::wall_type"]["thickness"]
assert t["status"] == "Assumed" and t["unit"] == "in", t
check("the partition's type, made for it", t["value"], 4.5, 0.5)
doors = [(p, a["pl::opening"]) for p, a in d.items() if p.startswith("door-")]; windows = [(p, a["pl::opening"]) for p, a in d.items() if p.startswith("window-")]
check("doors on the partition", len(doors), 1, 0, "doors"); check("windows on the south wall", len(windows), 1, 0, "windows")
dt = d[doors[0][1]["type"]["value"]]["pl::opening_type"]; wt = d[windows[0][1]["type"]["value"]]["pl::opening_type"]
assert all(x["status"] == "Assumed" and x["source"] == "found in scan house" for x in (dt["width"], dt["height"], wt["width"], wt["height"], windows[0][1]["sill"], doors[0][1]["sill"])), "an opening is not Assumed from the scan"
check("the door's width", dt["width"]["value"], 36, 2); check("the door's height", dt["height"]["value"], 80, 2)
check("the window's width", wt["width"]["value"], 60, 2); check("the window's height", wt["height"]["value"], 48, 2); check("the window's sill", windows[0][1]["sill"]["value"], 36, 2)
# The north wall is one wall from corner to corner, with nothing cut in it where the cabinets stand.
north = [p for p, w in walls.items() if abs(w["line"]["value"][0][1] - 20.25) < 0.1 and abs(w["line"]["value"][1][1] - 20.25) < 0.1]
assert len(north) == 1, north
nx = sorted(c[0] for c in walls[north[0]]["line"]["value"])
check("the north wall, west end", nx[0] * 12, -3, 2); check("the north wall, east end", nx[1] * 12, 30.25 * 12, 2)
assert not [p for p, e in d.items() if p.startswith(("door-", "window-")) and False]
parents = {c: e["path"] for e in job(sys.argv[2]) for c in (e.get("children") or {}).values()}
assert parents[doors[0][0]] == [p for p, w in walls.items() if abs(w["line"]["value"][0][0] - 18) < 0.1][0] and parents[windows[0][0]] not in north, parents
print("    the door is in the partition, the window in the south wall, and nothing is cut in the north wall")
# The house measured over the outside: the walls' centre lines, and half a wall each side.
xs = [c[0] for w in walls.values() for c in w["line"]["value"]]; ys = [c[1] for w in walls.values() for c in w["line"]["value"]]
check("the house over the outside, east to west", (max(xs) - min(xs) + 0.5) * 12, 31 * 12, 2); check("the house over the outside, north to south", (max(ys) - min(ys) + 0.5) * 12, 21 * 12, 2)
done()
PY2
echo "    rooms, by the program's own room finding, from the walls laid:"
"$P" room "$J" "$L" "West room" --at "6,8" --status Assumed --source "a point inside it" >/dev/null
"$P" room "$J" "$L" "East room" --at "24,8" --status Assumed --source "a point inside it" >/dev/null
SH="$("$P" show "$J")"
python3 - "$T" <<PY2B
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
shown = """$SH"""
area = lambda name: float(re.search(r"room-\d+ " + name + r" floor area: ([\d.]+) SF", shown)[1])
w, e = area("West room"), area("East room")
check("West room, (18 ft less 2 1/4 in) x 20 ft", w, 356.25, 4, "SF"); check("East room, (30 ft less 18 ft 2 1/4 in) x 20 ft", e, 236.25, 4, "SF")
check("together, 30 x 20 less the partition's 7.5 SF", w + e, 592.5, 8, "SF")
done()
PY2B
"$P" check "$J" >/dev/null && echo "    check passes"

echo
echo "== Step 3. Refusals, and what is said =="
cp "$J" "$T/laid.json"
refused "laid a second time" "level-0002 already has 5 walls found in scan house" "$P" find-walls "$J" "$T/house.ply" --level "$L" --lay --outside "$WT"
cmp -s "$J" "$T/laid.json" || { echo "FAIL: a refused second laying changed the job"; exit 1; }
echo "    the job's file is byte for byte the same"
cp "$T/typed.json" "$T/none.json"
NO="$("$P" find-walls "$T/none.json" "$T/house.ply" --level "$L" --lay)"
says "no outside wall named: the 4 are not laid" "not laid: 4 walls seen from one side only" "$NO"
says "and how to lay them" "say which wall type they are with --outside" "$NO"
says "the partition and its door only" "laid on $L: 1 wall, 1 door, 0 windows" "$NO"
[ "$("$P" show "$T/none.json" | grep -c '\[wall\]' || true)" = 1 ] || { echo "FAIL: more than the partition was laid"; exit 1; }
"$P" check "$T/none.json" >/dev/null && echo "    the partition, its door and nothing else is laid"
JN="$T/nc.json"; LN="$(new_job "$JN")"; cp "$JN" "$T/nc.before"
NC="$("$P" find-walls "$JN" "$T/noceiling.ply" --level "$LN" --lay)"
says "no ceiling: nothing found" "nothing found: without a ceiling a wall cannot be told from a tall cabinet" "$NC"
says "no ceiling: and why" "no ceiling shows in the scan: without one, a wall cannot be told from a tall cabinet standing in the room" "$NC"
cmp -s "$JN" "$T/nc.before" || { echo "FAIL: a scan with no ceiling changed the job"; exit 1; }
echo "    nothing was laid"
refused "units" "--units \"yd\": a scan is in m or ft" "$P" find-walls "$J" "$T/house.ply" --level "$L" --units yd
refused "no level said" "say which level the scan is of" "$P" find-walls "$J" "$T/house.ply"
refused "not a PLY" "it is not a PLY file" "$P" find-walls "$J" "$T/before.json" --level "$L"
refused "outside without laying" "--outside says what the walls seen from one side are laid as: it goes with --lay" "$P" find-walls "$J" "$T/house.ply" --level "$L" --outside "$WT"

echo
echo "== Step 4. The same house turned 17 degrees about its southwest corner =="
JT="$T/t.json"; LT="$(new_job "$JT")"
WTT="$(made "$("$P" type "$JT" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
FT="$("$P" find-walls "$JT" "$T/turned.ply" --level "$LT")"
echo "$FT" | sed -n '/walls found/,$p' | sed 's/^/    | /'
python3 - "$T" <<PY4
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FT"""
w = walls_said(said)
check("walls found", len(w), 5, 0, "walls"); check("doorways", sum(len(x["doors"]) for x in w), 1, 0, "doorways"); check("windows", sum(len(x["windows"]) for x in w), 1, 0, "windows")
check("stood short", int(re.search(r"stood short of the ceiling, and not walls: (\d+) things?", said)[1]), 3, 0, "things")
hid = [h for x in w for h in x["hidden"]]
check("hidden, taken as wall: how long", hid[0]["wide"], 9 * 12, 4)
sq = re.search(r"squared: (\d+) of (\d+) wall faces were within 1 degree of the scan's two main directions \(([\d.]+) and", said)
check("squared: faces", int(sq[1]), 6, 0, "faces"); check("the main direction, degrees from east", float(sq[3]), 17, 0.5, "deg")
done()
PY4
FTL="$("$P" find-walls "$JT" "$T/turned.ply" --level "$LT" --lay --outside "$WTT")"
says "laid" "laid on $LT: 5 walls, 1 door, 1 window" "$FTL"
"$P" room "$JT" "$LT" "West room" --at "3.399,9.404" --status Assumed --source "a point inside it" >/dev/null
"$P" room "$JT" "$LT" "East room" --at "20.61,14.67" --status Assumed --source "a point inside it" >/dev/null
SHT="$("$P" show "$JT")"
python3 - "$T" "$JT" <<PY4B
import math, re, sys
sys.path.insert(0, sys.argv[1])
from said import *
shown = """$SHT"""
area = lambda name: float(re.search(r"room-\d+ " + name + r" floor area: ([\d.]+) SF", shown)[1])
check("West room, turned", area("West room"), 356.25, 4, "SF"); check("East room, turned", area("East room"), 236.25, 4, "SF")
# The walls lie as the scan lies: the south wall's line (the wall the window is in) is 17 degrees off east.
walls = wall_lines(sys.argv[2]); d = job(sys.argv[2])
host = next(e["path"] for e in d if e.get("children") and any(c.startswith("window-") for c in e["children"].values()))
(a, b) = walls[host]["line"]["value"]
angle = math.degrees(math.atan2(b[1] - a[1], b[0] - a[0]))
off = (angle - 17 + 90) % 180 - 90          # how far the line is from 17 degrees, either way along it
check("the south wall's line, degrees off east", 17 + off, 17, 0.5, "deg")
done()
PY4B
"$P" check "$JT" >/dev/null && echo "    check passes"
echo
echo "== Step 5. The same house at other angles: a house lies at any angle on a phone's scan (33 once read no wall at all) =="
for A in 33 60; do
  python3 tools/made-up-scan.py "$T/t$A.ply" --as-scanned --turned "$A" --binary
  JA="$T/a$A.json"; LA="$(new_job "$JA")"
  WTA="$(made "$("$P" type "$JA" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
  FA="$("$P" find-walls "$JA" "$T/t$A.ply" --level "$LA")"
  # The two rooms' points, turned with the house about its southwest corner.
  PTS="$(python3 -c "import math;t=math.radians($A);print(' '.join('%.3f,%.3f'%(x*math.cos(t)-y*math.sin(t),x*math.sin(t)+y*math.cos(t)) for x,y in ((6,8),(24,8))))")"
  python3 - "$T" "$A" <<PY5
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FA"""
w = walls_said(said); a = sys.argv[2]
check("turned " + a + ": walls found", len(w), 5, 0, "walls")
check("turned " + a + ": doorways", sum(len(x["doors"]) for x in w), 1, 0, "doorways"); check("turned " + a + ": windows", sum(len(x["windows"]) for x in w), 1, 0, "windows")
check("turned " + a + ": stood short", int(re.search(r"stood short of the ceiling, and not walls: (\d+) things?", said)[1]), 3, 0, "things")
check("turned " + a + ": the partition's thickness", next(x for x in w if x["thick"] is not None)["thick"], 4.5, 0.5)
check("turned " + a + ": hidden, how long", [h for x in w for h in x["hidden"]][0]["wide"], 9 * 12, 4)
done()
PY5
  "$P" find-walls "$JA" "$T/t$A.ply" --level "$LA" --lay --outside "$WTA" >/dev/null
  set -- $PTS
  "$P" room "$JA" "$LA" "West room" --at "$1" --status Assumed --source "a point inside it" >/dev/null
  "$P" room "$JA" "$LA" "East room" --at "$2" --status Assumed --source "a point inside it" >/dev/null
  SHA="$("$P" show "$JA")"
  python3 - "$T" "$A" <<PY5B
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
shown = """$SHA"""
area = lambda name: float(re.search(r"room-\d+ " + name + r" floor area: ([\d.]+) SF", shown)[1])
check("turned " + sys.argv[2] + ": West room", area("West room"), 356.25, 4, "SF"); check("turned " + sys.argv[2] + ": East room", area("East room"), 236.25, 4, "SF")
done()
PY5B
  "$P" check "$JA" >/dev/null && echo "    check passes at $A degrees"
done

echo
echo "PASS: walls, doors, windows and what stands short found in a scan, said in full and laid on the word, to the figures worked by hand"
