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

# ---- call 179, part A: what a judge broke, each step with its figures worked by hand ----
# Scans for these are made by tools/made-up-scan.py --scene <name> (needs numpy): a house built
# from faces at so many points to the square foot, 1.5 cm of scatter, strays and half a degree of
# lean, the same every time. `--scene list` names each and says the right answer.
scene() { local out="$1"; shift; python3 tools/made-up-scan.py "$out" --binary --scene "$@"; }
job_for() { local j="$1"; new_job "$j"; }

echo
echo "== Step 6 (call 179, A7). The wrong measure, or the wrong way up: said first, and nothing is laid on its word =="
scene "$T/g1.ply" house --density 60
JY="$T/y.json"; LY="$(new_job "$JY")"; cp "$JY" "$T/y.before"
# By hand: the house is 30 ft by 16 ft and 8 ft high, written z up. Read with y up (--up y), what stands from "floor"
# to "ceiling" is the room's 16 ft depth, and the walls it reads are the house on its side.
FY="$("$P" find-walls "$JY" "$T/g1.ply" --level "$LY" --up y)"
echo "$FY" | sed -n '1,4p' | sed 's/^/    | /'
says "said first" "this is not likely the house: check the measure and which way is up" "$FY"
python3 - "$T" <<PY6
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FY"""
check("up read as the other axis: floor to ceiling", inches(re.search(r"floor to ceiling reads about " + LEN, said)[1]), 16 * 12, 2)
check("said before the first wall", said.index("not likely the house") < said.index("walls found"), 1, 0, "")
done()
PY6
WY="$(made "$("$P" type "$JY" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
cp "$JY" "$T/y.typed"
refused "not laid on its word" "nothing is laid: this is not likely the house" "$P" find-walls "$JY" "$T/g1.ply" --level "$LY" --up y --lay --outside "$WY"
refused "--anyway alone" "--anyway says to lay what is found though it is not likely a house: it goes with --lay" "$P" find-walls "$JY" "$T/g1.ply" --level "$LY" --up y --anyway
[ "$("$P" show "$JY" | grep -c '\[wall\]' || true)" = 0 ] || { echo "FAIL: a wall was laid from a scan that is not likely the house"; exit 1; }
cmp -s "$JY" "$T/y.typed" || { echo "FAIL: a refused laying changed the job"; exit 1; }
FYL="$("$P" find-walls "$JY" "$T/g1.ply" --level "$LY" --up y --lay --anyway --outside "$WY")"
says "laid on the person's word" "laid on" "$FYL"
# By hand: the same house read as feet where it is in meters is a house 0.3048 times the size: 8 ft is 2' 5".
JF="$T/f.json"; LF="$(new_job "$JF")"
FF="$("$P" find-walls "$JF" "$T/g1.ply" --level "$LF" --units ft)"
says "too small to hold a ceiling: said so" "this is not likely the house: check the measure and which way is up (the whole scan is only " "$FF"
python3 - "$T" <<PY6B
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FF"""
check("a house read as feet that is in meters: how high the whole scan is", inches(re.search(r"the whole scan is only " + LEN + " high", said)[1]), 8 * 12 * 0.3048, 2)
done()
PY6B

echo
echo "== Step 7 (call 179, A3). A bank of cabinets to the ceiling: said, and never laid on the scan's word alone =="
scene "$T/tall.ply" tall --density 100
# By hand: a room 20 ft by 14 ft, 8 ft high; against its north wall a bank of cabinets 10 ft long (east 2 to 12 ft) and 2 ft
# deep, 7 ft 9 in high, its back and the wall behind it below its top not seen. The bank's face is a face that reaches the ceiling
# (7' 9" is within 6 in of 8 ft), at 12 ft north, 10 ft long; its two ends meet no wall; the north wall's face stands 2 ft behind
# it and goes on past both ends. It is not a wall the scan can vouch for: found in doubt, said, and laid only on the person's word.
JB="$T/b.json"; LB="$(new_job "$JB")"
WB="$(made "$("$P" type "$JB" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
FB="$("$P" find-walls "$JB" "$T/tall.ply" --level "$LB")"
echo "$FB" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
says "the bank is said to be in doubt" "a wall or something tall standing at the wall" "$FB"
python3 - "$T" <<PY7
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FB"""
w = walls_said(said)
bank = [x for x in w if abs(x["from"][1] - 12 * 12) < 3 and abs(x["to"][1] - 12 * 12) < 3 and 8 * 12 < x["long"] < 11 * 12]
assert len(bank) == 1, w
check("the bank's face, north", bank[0]["from"][1], 12 * 12, 1); check("the bank's length", bank[0]["long"], 10 * 12, 5)
check("walls found (the room's four, the north one whole, and the bank)", len(w), 5, 0, "walls")
north = [x for x in w if abs(x["from"][1] - 14 * 12) < 3 and abs(x["to"][1] - 14 * 12) < 3]
assert len(north) == 1 and north[0]["hidden"], w
check("the north wall, whole", north[0]["long"], 20 * 12, 4); check("behind the bank, not seen", north[0]["hidden"][0]["wide"], 10 * 12, 8)
check("in doubt", int(re.search(r"(\d+) of them in doubt", said)[1]), 1, 0, "walls")
done()
PY7
FBL="$("$P" find-walls "$JB" "$T/tall.ply" --level "$LB" --lay --outside "$WB")"
echo "$FBL" | sed -n '/laid on/,$p' | sed 's/^/    | /'
says "four laid, not the bank" "laid on $LB: 4 walls" "$FBL"
says "the bank named, and what to say" "not laid: wall" "$FBL"
says "the person's word" "say --take" "$FBL"
JB2="$T/b2.json"; LB2="$(new_job "$JB2")"; WB2="$(made "$("$P" type "$JB2" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
BIDX="$(python3 - "$T" <<PYI
import sys
sys.path.insert(0, sys.argv[1])
from said import *
w = walls_said("""$FB""")
print([n + 1 for n, x in enumerate(w) if abs(x["from"][1] - 144) < 3 and abs(x["to"][1] - 144) < 3 and 96 < x["long"] < 132][0])
PYI
)"
FBT="$("$P" find-walls "$JB2" "$T/tall.ply" --level "$LB2" --lay --outside "$WB2" --take "$BIDX")"
says "said to be a wall, it is laid" "laid on $LB2: 5 walls" "$FBT"
refused "--take without --lay" "--take says a wall found is a wall, to be laid with the rest: it goes with --lay" "$P" find-walls "$JB" "$T/tall.ply" --level "$LB" --take 4

echo
echo "== Step 8 (call 179, A2). A wall hidden behind cabinets is found, said not seen, and whether the walls close is said =="
scene "$T/hid.ply" hidden --density 100 --set pct=60
scene "$T/pack.ply" pack2 --density 100
scene "$T/nonorth.ply" nonorth --density 100
# By hand, 1: a room 30 ft by 16 ft, 8 ft high, cabinets 2 ft deep and 7 ft high stand before the north wall for 60 in 100 of it
# from the east end: 18 ft (east 12 to 30). The wall behind them is not seen below 7 ft, only above them; it is still one wall 30 ft
# long, the four walls close, and 18 ft of it is said not seen and taken as wall. The cabinets are 1 thing, not a wall.
JH="$T/hid.json"; LH="$(new_job "$JH")"
FH="$("$P" find-walls "$JH" "$T/hid.ply" --level "$LH")"
echo "$FH" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
python3 - "$T" <<PY8
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FH"""
w = walls_said(said)
check("walls found (four, the north one whole)", len(w), 4, 0, "walls")
north = [x for x in w if abs(x["from"][1] - 192) < 3 and abs(x["to"][1] - 192) < 3]
assert len(north) == 1, w
check("the north wall's length", north[0]["long"], 30 * 12, 4)
assert len(north[0]["hidden"]) == 1, north[0]
h = north[0]["hidden"][0]
check("not seen behind the cabinets: how long", h["wide"], 18 * 12, 6); check("not seen: from its west end", h["from"], 12 * 12, 6); check("not seen: to", h["to"], 30 * 12, 6)
assert "the walls found close: every end meets another wall" in said, said
check("stood short (the cabinets): things", int(re.search(r"not walls: (\d+) things?", said)[1]), 1, 0, "things")
done()
PY8
# By hand, 2: a room 20 ft by 14 ft, 8 ft high, with cabinets 2 ft deep and 6 ft 6 in high along the south, north (east 1 to 19 ft)
# and west (north 2 to 12 ft) walls, the walls behind them not seen below 6 ft 6 in: four walls, three of them seen in part only;
# not seen for 18 ft, 18 ft and 10 ft; the walls close.
JP="$T/pack.json"; LP="$(new_job "$JP")"
FP="$("$P" find-walls "$JP" "$T/pack.ply" --level "$LP")"
echo "$FP" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
python3 - "$T" <<PY8B
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FP"""
w = walls_said(said)
check("walls found", len(w), 4, 0, "walls")
hid = sorted(h["wide"] for x in w for h in x["hidden"])
check("walls with a stretch not seen", len(hid), 3, 0, "walls")
for got, hand in zip(hid, (10 * 12, 18 * 12, 18 * 12)):
    check("a stretch not seen behind the cabinets", got, hand, 6)
assert "the walls found close: every end meets another wall" in said, said
done()
PY8B
# By hand, 3: a room with no north wall: the west and east walls' north ends, at 16 ft north, meet nothing. Said, with where.
JN="$T/nn.json"; LN="$(new_job "$JN")"
FN="$("$P" find-walls "$JN" "$T/nonorth.ply" --level "$LN")"
echo "$FN" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
says "the walls do not close, and how many ends" "the walls found do not close: 2 ends meet nothing" "$FN"
python3 - "$T" <<PY8C
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FN"""
ends = re.findall(r"its (west|east|south|north) end, at " + LEN + " east, " + LEN + " north", said)
check("ends that meet nothing", len(ends), 2, 0, "ends")
for e in ends:
    check("an open end, north", inches(e[2]), 16 * 12, 3)
check("the open ends are at 0 and 30 ft east", sum(inches(e[1]) for e in ends), 30 * 12, 6)
done()
PY8C

echo
echo "== Step 9 (call 179, A1). A thin scan is read by its own thickness; one too thin to tell a wall says so =="
scene "$T/thin15.ply" house --density 15
scene "$T/thin30t.ply" house --density 30 --turned 33
scene "$T/thin4.ply" house --density 4
# By hand: the house of step 1 (30 ft by 16 ft, a partition 4 1/2 in thick at 18 ft) scanned at 15 points to the square foot
# of every surface, and at 30 turned 33 degrees: five walls, the partition 4 1/2 in thick, nothing stood short, the walls close,
# the scan's thickness said to about a fifth, and what thin does to what is said. At 4 to the square foot: too thin, nothing found.
for TH in thin15:15 thin30t:30; do
  N="${TH%%:*}"; D="${TH##*:}"
  JT2="$T/$N.json"; LT2="$(new_job "$JT2")"
  FTH="$("$P" find-walls "$JT2" "$T/$N.ply" --level "$LT2")"
  echo "$FTH" | sed -n '1,4p' | sed 's/^/    | /'
  python3 - "$T" "$D" <<PY9
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FTH"""; d = float(sys.argv[2])
w = walls_said(said)
check("thin, " + sys.argv[2] + " to the square foot: walls found", len(w), 5, 0, "walls")
check("the scan's thickness said", float(re.search(r"holds about (\d+) points", said)[1]), d, 0.2 * d, "points")
check("the partition's thickness", next(x for x in w if x["thick"] is not None)["thick"], 4.5, 1)
check("things stood short", 0 if "nothing stood short" in said else 1, 0, 0, "")
assert "the walls found close" in said and "this scan is thinner than the measures here were made for" in said, said
done()
PY9
done
JT4="$T/thin4.json"; LT4="$(new_job "$JT4")"; cp "$JT4" "$T/thin4.before"
FT4="$("$P" find-walls "$JT4" "$T/thin4.ply" --level "$LT4" --lay)"
echo "$FT4" | sed 's/^/    | /'
says "too thin is said" "nothing found: the scan is too thin to tell a wall from furniture" "$FT4"
says "and how thin" "to the square foot, and it takes 8 points or more" "$FT4"
[ "$(grep -c 'furniture' <<<"$FT4")" = 1 ] || { echo "FAIL: a thin scan's walls are called furniture again"; exit 1; }
cmp -s "$JT4" "$T/thin4.before" || { echo "FAIL: a scan too thin changed the job"; exit 1; }
echo "    nothing was laid, and the job is as it was"

echo
echo "== Step 10 (call 179, A4). A wall is judged against the ceiling over it: two heights, and a slope, said =="
scene "$T/two.ply" twoceil --density 100
scene "$T/slope.ply" slope --density 100
# By hand, 1: two rooms 15 ft by 16 ft side by side, the west 8 ft high and the east 10 ft; a partition 4 1/2 in thick between them
# reaching the low room's ceiling (8 ft), the outside walls to the ceiling over them. Five walls (the south and north run under both),
# no furniture; the west wall 8 ft high, the east wall 10 ft, the partition 8 ft. The ceiling is said to be two heights, 8 ft and 10 ft,
# and what is not done about it (the step in a wall that runs under both). Laid, each wall is as high as the ceiling over it.
J2="$T/two.json"; L2="$(new_job "$J2")"
W2="$(made "$("$P" type "$J2" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
F2H="$("$P" find-walls "$J2" "$T/two.ply" --level "$L2" --lay --outside "$W2")"
echo "$F2H" | sed -n '/the ceiling is not/,/laid on/p' | cut -c1-230 | sed 's/^/    | /'
says "two heights said" "the ceiling is not one height: 8' 0\" and 10' 0\"" "$F2H"
says "and what is not done" "the step in its top is not drawn" "$F2H"
echo "$F2H" > "$T/f2h.txt"
python3 - "$T" "$J2" <<PY10
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = open(sys.argv[1] + "/f2h.txt").read()
w = walls_said(said)
check("walls found (not furniture)", len(w), 5, 0, "walls")
assert "nothing stood short of the ceiling" in said, said
high = {}
for line in said.splitlines():
    m = re.match(r"\s+wall (\d+): from " + LEN + " east, " + LEN + " north to " + LEN + " east, " + LEN + r" north, .* (\d+)' (\d+)\" high", line)
    if m:
        high[int(m[1])] = int(m[6]) * 12 + int(m[7])
west = [i for i, x in enumerate(w, 1) if abs(x["from"][0]) < 3 and abs(x["to"][0]) < 3]
east = [i for i, x in enumerate(w, 1) if abs(x["from"][0] - 360) < 3 and abs(x["to"][0] - 360) < 3]
part = [i for i, x in enumerate(w, 1) if x["thick"] is not None]
check("the west wall's height", high[west[0]], 8 * 12, 3); check("the east wall's height", high[east[0]], 10 * 12, 3); check("the partition's height", high[part[0]], 8 * 12, 3)
walls = wall_lines(sys.argv[2])
got = sorted(w["height"]["value"] for w in walls.values())
check("laid: the lowest wall", got[0], 96, 3); check("laid: the highest wall", got[-1], 120, 3)
assert sum(1 for g in got if abs(g - 96) <= 3) == 2 and sum(1 for g in got if abs(g - 120) <= 3) == 3, got
print("    laid: two walls 96 in high (west wall, partition) and three 120 in (east, and the two that run under both)")
done()
PY10
# By hand, 2: a room 30 ft by 16 ft whose ceiling rises from 8 ft at the south wall to 12 ft at the north. A ceiling shows (it is not
# level: "no ceiling" was said once); four walls; the south wall under 8 ft, the north under 12; the slope said, and what is not drawn.
JS="$T/slope.json"; LS="$(new_job "$JS")"
FS="$("$P" find-walls "$JS" "$T/slope.ply" --level "$LS")"
echo "$FS" | sed -n '/the ceiling slopes/,/squared/p' | cut -c1-230 | sed 's/^/    | /'
says "a slope said" "the ceiling slopes, from 8' " "$FS"
says "and what is not drawn" "its sloped top is not drawn" "$FS"
python3 - "$T" <<PY10B
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FS"""
w = walls_said(said)
check("walls found", len(w), 4, 0, "walls")
assert "no ceiling shows" not in said, said
high = {}
for line in said.splitlines():
    m = re.match(r"\s+wall (\d+): from " + LEN + " east, " + LEN + " north to " + LEN + " east, " + LEN + r" north, .* (\d+)' (\d+)\" high", line)
    if m:
        high[int(m[1])] = int(m[6]) * 12 + int(m[7])
south = [i for i, x in enumerate(w, 1) if abs(x["from"][1]) < 3 and abs(x["to"][1]) < 3]; north = [i for i, x in enumerate(w, 1) if abs(x["from"][1] - 192) < 3 and abs(x["to"][1] - 192) < 3]
check("the south wall under the ceiling", high[south[0]], 8 * 12, 6); check("the north wall under the ceiling", high[north[0]], 12 * 12, 6)
done()
PY10B

echo
echo "== Step 11 (call 179, A5). A thin wall beside a thick one: found whole, nothing leaning; and one wall that cannot be laid is named, the rest laid =="
scene "$T/thickthin.ply" thickthin --density 100
scene "$T/closet.ply" closet --density 100
# By hand, 1: a room 30 ft by 16 ft, 8 ft high, a wall 12 in thick at 10 ft east and a wall 3 1/2 in thick at 20 ft east, each across the
# room and scanned on both faces. Six walls: the four outside walls (one face each), the 12 in wall and the 3 1/2 in wall. None leans: each
# wall found lies along the house to within an inch over its whole length (it was three walls at 20 ft, two of them leaning across one another).
JK="$T/tt.json"; LK="$(new_job "$JK")"
FK="$("$P" find-walls "$JK" "$T/thickthin.ply" --level "$LK")"
echo "$FK" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
echo "$FK" > "$T/fk.txt"
python3 - "$T" <<PY11
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = open(sys.argv[1] + "/fk.txt").read()
w = walls_said(said)
check("walls found (not phantoms)", len(w), 6, 0, "walls")
thick = sorted(x["thick"] for x in w if x["thick"] is not None)
check("the thin wall's thickness", thick[0], 3.5, 0.5); check("the thick wall's thickness", thick[1], 12, 0.5)
for x in w:
    over = min(abs(x["to"][0] - x["from"][0]), abs(x["to"][1] - x["from"][1]))
    check("a wall's lean over its whole length, wall at " + str(round(x["from"][0] / 12)) + " ft east", over, 0, 1)
done()
PY11
# By hand, 2: the closet: a room 20 ft by 12 ft, a closet of two walls 3 1/2 in thick, 2 ft apart (6 and 8 ft north, from the west wall to 6 ft east)
# and a 2 ft cross wall at 6 ft east. The two long closet walls are found, each as one wall of two faces 3 1/2 in thick, neither leaning;
# the 2 ft cross wall is exactly the least a wall is and is not found: said ("under 2 ft long"), and the walls found do not close there.
JC="$T/closet.json"; LC="$(new_job "$JC")"
FC="$("$P" find-walls "$JC" "$T/closet.ply" --level "$LC")"
echo "$FC" > "$T/fc.txt"
python3 - "$T" <<PY11B
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = open(sys.argv[1] + "/fc.txt").read()
w = walls_said(said)
thin = [x for x in w if x["thick"] is not None]
check("closet walls found, each of two faces", len(thin), 2, 0, "walls")
for x in thin:
    check("a closet wall's thickness", x["thick"], 3.5, 0.5); check("its length", x["long"], 6 * 12, 4)
    check("its lean over its whole length", min(abs(x["to"][0] - x["from"][0]), abs(x["to"][1] - x["from"][1])), 0, 1)
assert "not taken as walls" in said and "under 2 ft long" in said, said
done()
PY11B
says "and the closet is not shut" "the walls found do not close" "$FC"
# By hand, 3: laying that cannot lay one wall. A wall is drawn by hand along the whole south wall (30 ft). Laying what was found: the south wall
# lies along it for 30 ft and is not laid, and says which wall it is; the other five are laid.
JO="$T/ov.json"; LO="$(new_job "$JO")"
WO="$(made "$("$P" type "$JO" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
HAND="$(made "$("$P" wall "$JO" "$LO" "$WO" --points "0,0 30,0" --height "8 ft" --status Assumed --source "drawn by hand")")"
cp "$JO" "$T/ov.before"
FO="$("$P" find-walls "$JO" "$T/thickthin.ply" --level "$LO" --lay --outside "$WO")"
echo "$FO" | sed -n '/laid on/,$p' | sed 's/^/    | /'
says "five of six laid" "laid on $LO: 5 walls" "$FO"
says "the one left is named, and what it lies along" "of what was found: lies along $HAND for 30 ft" "$FO"  # call 226: a length in a sentence is said as a builder says it
[ "$("$P" show "$JO" | grep -c '\[wall\]' || true)" = 6 ] || { echo "FAIL: the hand-drawn wall and five found walls are six in the job"; exit 1; }
"$P" check "$JO" >/dev/null && echo "    check passes: the hand-drawn wall, and five laid beside it"

echo
echo "== Step 12 (call 179, A8 and A9). A level's height that the scan disagrees with is said; names, not marks; a scan far from zero =="
scene "$T/h12.ply" house --density 60
scene "$T/far.ply" house --density 60 --far
# By hand, 1 (A8): the level is made 9 ft high, Known, "tape"; the scan reads 8 ft from floor to ceiling. That is said, with both and
# which is laid: the walls are laid 8 ft (96 in), Assumed, from the scan; the level's 9 ft Known is not changed by it.
J12="$T/h12.json"; "$P" new "$J12" "A scanned house (not a real job)" >/dev/null
L12="$(made "$("$P" level "$J12" ground --elevation "0 ft" --height "9 ft" --status Known --source "tape")")"
W12="$(made "$("$P" type "$J12" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
# (A9: the level and the wall type are given by their names; the words say "read as", not "taken as ... z up")
F12="$("$P" find-walls "$J12" "$T/h12.ply" --level ground --lay --outside "Existing outside wall")"
echo "$F12" | sed -n '1,6p' | cut -c1-260 | sed 's/^/    | /'
says "the disagreement is said" "the level ground is 9' 0\" high (Known, tape) and the scan reads 8' 0\" from floor to ceiling, 1' 0\" less" "$F12"
says "and what is laid" "the walls are laid as high as the scan reads, Assumed, from the scan, and the level's height is not changed" "$F12"
python3 - "$T" "$J12" <<PY12
import sys
sys.path.insert(0, sys.argv[1])
from said import *
walls = wall_lines(sys.argv[2])
assert len(walls) == 5, list(walls)
for p, w in walls.items():
    check("a wall's height, laid", w["height"]["value"], 96, 1)
    assert w["height"]["status"] == "Assumed" and w["height"]["source"].startswith("found in scan"), w["height"]
lv = [e for e in job(sys.argv[2]) if "pl::level" in e["attributes"]][0]["attributes"]["pl::level"]["height"]
assert lv["status"] == "Known" and lv["value"] in (9, 9.0, 108, 108.0) and lv["source"] == "tape", lv
print("    the level is still 9 ft, Known, tape; the five walls are 96 in, Assumed, found in the scan")
done()
PY12
echo "$F12" > "$T/f12.txt"
! grep -qE "z up|y up|PLY| with z" "$T/f12.txt" || { echo "FAIL: a software word is said"; exit 1; }
says "read as, in a builder's words" "points, read as meters" "$F12"
# A type the program makes is named for what it is, the size first ("4.5 in wall": call 178, item 10;
# it was "wall 4.5 in").
says "a type by its name" "made for them, Assumed, from the scan: the type \"4.5 in wall\"" "$F12"
# By hand, 2 (A9): the same house scanned far from zero (1,234 ft east, 567 ft north in this made-up georeference). With no sheet of the scan
# on the level it is laid at its own southwest corner and that is said; with its sheet it is laid where the sheet lies, and that is said.
JF2="$T/far2.json"; LF2="$(new_job "$JF2")"
WF2="$(made "$("$P" type "$JF2" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
FFA="$("$P" find-walls "$JF2" "$T/far.ply" --level "$LF2" --lay --outside "$WF2")"
says "far from zero, no sheet: said" "has no sheet on ground to lay it by: it is laid with its own southwest corner at zero" "$FFA"
python3 - "$T" "$JF2" <<PY12B
import sys
sys.path.insert(0, sys.argv[1])
from said import *
walls = wall_lines(sys.argv[2])
xs = [c[0] for w in walls.values() for c in w["line"]["value"]]; ys = [c[1] for w in walls.values() for c in w["line"]["value"]]
check("laid at its own southwest corner: the least east", min(xs) * 12, -3, 2); check("the least north", min(ys) * 12, -3, 2)
check("the house over the outside, east to west", (max(xs) - min(xs) + 0.5) * 12, 31 * 12, 2)
done()
PY12B
JF3="$T/far3.json"; LF3="$(new_job "$JF3")"
WF3="$(made "$("$P" type "$JF3" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
"$P" import scan "$JF3" "$T/far.ply" --level "$LF3" >/dev/null
FFB="$("$P" find-walls "$JF3" "$T/far.ply" --level "$LF3" --lay --outside "$WF3")"
says "far from zero, with its sheet: laid where the sheet lies" "it is laid where its sheet on ground lies" "$FFB"
python3 - "$T" "$JF3" <<PY12C
import sys
sys.path.insert(0, sys.argv[1])
from said import *
walls = wall_lines(sys.argv[2])
xs = [c[0] for w in walls.values() for c in w["line"]["value"]]
check("laid where its sheet lies: the least east", min(xs), 1234.5, 1, "ft")
done()
PY12C

# ---- call 183: what the second judge found, each step with its figures worked by hand ----

echo
echo "== Step 13 (call 183, 1). A face belongs to one wall: a face recorded twice beside a partition is one wall and a doubt, never two laid =="
scene "$T/drift.ply" drift --set drift=0.6 --density 100 --seed 1
# By hand: a room 20 ft by 12 ft, 8 ft high, a partition 4 1/2 in thick at 10 ft east (faces at 9' 9 3/4" and 10' 2 1/4"), and its east face
# recorded again 7 1/5 in east of itself (10' 9 1/2") from 3 to 8 ft north (a scan that does not register its passes). Five walls
# are the house (the four outside, the partition 4 1/2 in, 12 ft); the face again is a sixth, 5 ft long, which lies where the
# partition already has its two faces: in doubt, said with the wall it lies by, and not laid unless the person says it is a wall.
J13="$T/d13.json"; L13="$(new_job "$J13")"
W13="$(made "$("$P" type "$J13" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
F13="$("$P" find-walls "$J13" "$T/drift.ply" --level "$L13")"
echo "$F13" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
says "the face again is said to be in doubt" "where it already has its faces: the same face seen again, or something against it" "$F13"
python3 - "$T" <<PY13
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F13"""
w = walls_said(said)
check("walls found (five, and the face again)", len(w), 6, 0, "walls")
check("in doubt", int(re.search(r"(\d+) of them in doubt", said)[1]), 1, 0, "walls")
again = [x for x in w if x["long"] < 6 * 12 and 10 * 12 < x["from"][0] < 11 * 12]
assert len(again) == 1, w
check("the face again, east", again[0]["from"][0], 10 * 12 + 9.6, 3); check("its length", again[0]["long"], 5 * 12, 6)
part = [x for x in w if x["thick"] is not None]
assert len(part) == 1, w
check("the partition's thickness", part[0]["thick"], 4.5, 0.5); check("the partition's length", part[0]["long"], 12 * 12, 4)
done()
PY13
F13L="$("$P" find-walls "$J13" "$T/drift.ply" --level "$L13" --lay --outside "$W13")"
echo "$F13L" | sed -n '/laid on/,$p' | sed 's/^/    | /'
says "five laid, not the face again" "laid on $L13: 5 walls" "$F13L"
says "the one not laid is named, and what to say" "say --take" "$F13L"
python3 - "$T" "$J13" <<PY13B
import sys
sys.path.insert(0, sys.argv[1])
from said import *
walls = wall_lines(sys.argv[2])
check("walls in the saved job", len(walls), 5, 0, "walls")
xs = sorted(round((w["line"]["value"][0][0]) * 12) for w in walls.values() if abs(w["line"]["value"][0][0] - w["line"]["value"][1][0]) < 0.01)
print("    north-south walls laid, east of the west corner, in inches:", xs)
assert not any(10 * 12 + 4 < x < 11 * 12 for x in xs), "a wall is laid beside the partition"
done()
PY13B

echo
echo "== Step 14 (call 183, 2). What is found beyond a wall with no way to it is in doubt and said: a mirror's room, a neighbour's wall through glass =="
scene "$T/mirror.ply" mirror --density 100 --seed 1
# By hand: a room 20 ft by 12 ft, 8 ft high, with a mirror on its east wall: the whole scan is again beyond that wall (east 20 to 40 ft). The room walked
# has four walls, 20 + 12 + 20 + 12 = 64 LF. The mirror's south and north walls run on to 40 ft (they are cut at the east wall: 20 ft each beyond
# it) and its west wall stands at 40 ft, 12 ft long. Four walls are the house; three are beyond the east wall with no way to it: in doubt, said,
# not laid. Which side of the wall is the house is not in the scan (as much floor each side): the side the scan begins on, and that is said.
J14="$T/m14.json"; L14="$(new_job "$J14")"
W14="$(made "$("$P" type "$J14" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
F14="$("$P" find-walls "$J14" "$T/mirror.ply" --level "$L14")"
echo "$F14" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
says "beyond a wall with no way to it" "in doubt: beyond wall" "$F14"
says "no way to it" "with no way to it" "$F14"
says "which side the house is on, said" "it is taken as the side the scan begins on" "$F14"
python3 - "$T" <<PY14
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F14"""
w = walls_said(said)
check("walls found (four, and three beyond)", len(w), 7, 0, "walls")
check("in doubt", int(re.search(r"(\d+) of them in doubt", said)[1]), 3, 0, "walls")
beyond = [x for x in w if min(x["from"][0], x["to"][0]) >= 20 * 12 - 3 and (x["long"] > 19 * 12 or abs(x["from"][0] - 40 * 12) < 3)]
check("walls east of the east wall (their ends at 20 ft or more)", len(beyond), 3, 0, "walls")
check("the longest of them, 20 ft", max(x["long"] for x in beyond), 20 * 12, 6)
done()
PY14
F14L="$("$P" find-walls "$J14" "$T/mirror.ply" --level "$L14" --lay --outside "$W14")"
echo "$F14L" | sed -n '/laid on/,$p' | sed 's/^/    | /'
says "four laid, not the room beyond" "laid on $L14: 4 walls" "$F14L"
python3 - "$T" "$J14" <<PY14B
import math, sys
sys.path.insert(0, sys.argv[1])
from said import *
walls = wall_lines(sys.argv[2])
lf = sum(math.hypot(w["line"]["value"][1][0] - w["line"]["value"][0][0], w["line"]["value"][1][1] - w["line"]["value"][0][1]) for w in walls.values())
check("laid, the centre lines of the room's four walls (hand: faces 64 LF, each wall's centre 3 in outside its face: 2 x 20.5 + 2 x 12.5 = 66)", lf, 66.0, 2.0, "ft")
done()
PY14B

scene "$T/neighbour.ply" neighbor --density 100 --seed 1
# By hand: a room 20 ft by 12 ft, 8 ft high, a window 5 ft wide (sill 3 ft, head 7 ft) in the east wall and, 10 ft outside the glass, a neighbour's wall seen
# through it: 5 ft of it, 3 to 7 ft up. Four walls and the window are the house. The neighbour's wall is beyond the east wall, which has a window and no
# doorway: said ("beyond wall ... with no way to it"), and not among what stood short in the room.
J14N="$T/n14.json"; L14N="$(new_job "$J14N")"
F14N="$("$P" find-walls "$J14N" "$T/neighbour.ply" --level "$L14N")"
echo "$F14N" | sed -n '/walls found/,$p' | sed 's/^/    | /'
says "the neighbour's wall is said to be beyond" "is beyond wall" "$F14N"
says "and not counted among what stood short" "nothing stood short of the ceiling" "$F14N"
python3 - "$T" <<PY14C
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F14N"""
w = walls_said(said)
check("walls found", len(w), 4, 0, "walls")
win = [x for ww in w for x in ww["windows"]]
check("windows", len(win), 1, 0, "windows"); check("the window's width", win[0]["wide"], 60, 3); check("its sill", win[0]["sill"], 36, 2); check("its head", win[0]["head"], 84, 2)
m = re.search(r"something (\d+)' (\d+)\" high, at " + LEN + " east", said)
assert m, said
check("the neighbour's wall, how high", int(m[1]) * 12 + int(m[2]), 7 * 12, 4); check("how far east", inches(m[3]), 30 * 12, 6)
done()
PY14C

echo
echo "== Step 15 (call 183, 4). A short wall with a door in it is found; every stretch left out is said with where it is, and where it leaves the room open =="
scene "$T/stub.ply" stub --density 100
# By hand: a room 20 ft by 12 ft, 8 ft high; across its east end a partition 4 1/2 in thick, 4 ft long (east 16 to 20 ft, 6 ft north), with a doorway
# 2 ft 6 in wide 1 ft from its west end, head 6 ft 8 in. Its pieces are a jamb of 1 ft, a head across, and a jamb of 6 in: a wall with a door in it. Five
# walls: the four outside and the partition (4 ft long, 4 1/2 in thick, one doorway 2 ft 6 in wide from 1 ft along it, head 6 ft 8 in). Its west end is free:
# the walls found do not close there.
J15="$T/s15.json"; L15="$(new_job "$J15")"
F15="$("$P" find-walls "$J15" "$T/stub.ply" --level "$L15")"
echo "$F15" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
python3 - "$T" <<PY15
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F15"""
w = walls_said(said)
check("walls found (the four outside and the short wall)", len(w), 5, 0, "walls")
stub = [x for x in w if x["thick"] is not None]
assert len(stub) == 1, w
check("the short wall's length", stub[0]["long"], 4 * 12, 4); check("its thickness", stub[0]["thick"], 4.5, 0.5)
assert len(stub[0]["doors"]) == 1, stub[0]
d = stub[0]["doors"][0]
check("its doorway's width", d["wide"], 30, 3); check("where the doorway begins", d["from"], 12, 3); check("its head", d["head"], 80, 2)
done()
PY15

scene "$T/stubx.ply" stubx --density 100
# By hand: a room 20 ft by 12 ft, 8 ft high; a partition 4 1/2 in thick, 4 ft long, going north from the south wall at 10 ft east; at its free end a face 1 ft 6 in
# long across it (east 10 to 11 ft 6 in, 4 ft north), floor to ceiling. Five walls found. The face is under 2 ft: left out, and said with where it is (from about
# 10 ft east, 4 ft north to 11 ft 6 in east) and how long (1 ft 6 in, to 3 in). The partition's north end meets nothing: said, and said that the stretch left out lies there.
J15B="$T/s15b.json"; L15B="$(new_job "$J15B")"
F15B="$("$P" find-walls "$J15B" "$T/stubx.ply" --level "$L15B")"
echo "$F15B" | sed -n '1,/walls found/p;/do not close/,/squared/p' | sed 's/^/    | /'
says "a stretch left out is said with where it is" "a stretch of upright points" "$F15B"
says "and why" "not taken as walls: under 2 ft long (a wall is at least 2 ft long)" "$F15B"
says "where it leaves the room open, said" "leaving it out leaves the rooms either side open to one another" "$F15B"
python3 - "$T" <<PY15B
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F15B"""
check("walls found (the four outside and the partition)", len(walls_said(said)), 5, 0, "walls")
m = re.search(r"a stretch of upright points " + LEN + " long, from " + LEN + " east, " + LEN + " north to " + LEN + " east, " + LEN + " north, not taken as walls: under 2 ft", said)
assert m, said
check("how long", inches(m[1]), 18, 4); check("where it begins, east", min(inches(m[2]), inches(m[4])), 10 * 12, 4); check("where it ends, east", max(inches(m[2]), inches(m[4])), 11.5 * 12, 4); check("north", inches(m[3]), 4 * 12, 3)
done()
PY15B

echo
echo "== Step 16 (call 183, 3). A soffit over cabinets, and a face that meets no wall at either end: in doubt, said, never laid on the scan's word =="
scene "$T/soffit.ply" soffit2 --density 100
# By hand: a room 20 ft by 12 ft, 8 ft high; along the whole north wall base cabinets 3 ft high and 2 ft deep with a soffit over them from 7 to 8 ft in the plane of
# their front (10 ft north), wall to wall. The wall behind (12 ft north) is seen only between 3 and 7 ft up. The face at 10 ft north is 20 ft long, reaches the
# floor (the cabinets) and the ceiling (the soffit), with nothing between 3 and 7 ft and a wall behind that: the top of what stands, not a wall: in doubt, laid
# only on the person's word. The south, west and east walls are laid (the west and east stop at the face, 11 ft 10 in): 3 walls; the north ends meet nothing: said.
J16="$T/f16.json"; L16="$(new_job "$J16")"
W16="$(made "$("$P" type "$J16" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
F16="$("$P" find-walls "$J16" "$T/soffit.ply" --level "$L16")"
echo "$F16" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
says "the face is in doubt, as base cabinets and a soffit" "such as base cabinets and a soffit, and not a wall" "$F16"
says "the hole said, 3 ft 3 in to 7 ft up" "(3' 3\" to 7' 0\" up)" "$F16"
python3 - "$T" <<PY16
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F16"""
w = walls_said(said)
check("walls found", len(w), 4, 0, "walls"); check("in doubt", int(re.search(r"(\d+) of them in doubt", said)[1]), 1, 0, "walls")
face = [x for x in w if abs(x["from"][1] - 120) < 3 and abs(x["to"][1] - 120) < 3]
assert len(face) == 1, w
check("the face, 10 ft north", face[0]["from"][1], 120, 2); check("its length", face[0]["long"], 240, 6)
check("the open ends said", int(re.search(r"do not close: (\d+) ends", said)[1]), 2, 0, "ends")
done()
PY16
F16L="$("$P" find-walls "$J16" "$T/soffit.ply" --level "$L16" --lay --outside "$W16")"
says "three laid, not the face" "laid on $L16: 3 walls" "$F16L"
says "the word the person must give" "say --take" "$F16L"

scene "$T/free.ply" freestand --density 100
# By hand: a room 20 ft by 12 ft, 8 ft high with, in the middle of it, a face of one side 6 ft long (east 7 to 13, 6 ft north), floor to ceiling, nearest wall 6 ft away. Four walls
# are the house. The face meets no wall at either end, with nothing standing behind it: in doubt, said, and not laid.
J16B="$T/g16.json"; L16B="$(new_job "$J16B")"
W16B="$(made "$("$P" type "$J16B" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
F16B="$("$P" find-walls "$J16B" "$T/free.ply" --level "$L16B")"
echo "$F16B" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
says "a face that meets no wall is in doubt" "meets no wall at either end: a wall seen from one side that stops in the room, or the front of something tall standing in it" "$F16B"
python3 - "$T" <<PY16B
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F16B"""
w = walls_said(said)
check("walls found", len(w), 5, 0, "walls"); check("in doubt", int(re.search(r"(\d+) of them in doubt", said)[1]), 1, 0, "walls")
face = [x for x in w if abs(x["from"][1] - 72) < 3 and abs(x["to"][1] - 72) < 3 and x["long"] < 8 * 12]
assert len(face) == 1, w
check("the face's length", face[0]["long"], 72, 6)
done()
PY16B
F16BL="$("$P" find-walls "$J16B" "$T/free.ply" --level "$L16B" --lay --outside "$W16B")"
says "four laid, not the face" "laid on $L16B: 4 walls" "$F16BL"

echo
echo "== Step 17 (call 183, 5). A wall that is mostly opening is a wall with an opening; a curved face is said to be curved, with its length, and is not laid =="
scene "$T/garage.ply" garage --density 100
# By hand: a garage 24 ft by 22 ft, 9 ft high; in its south wall a door 16 ft wide (east 4 to 20 ft, head 7 ft), in its east wall one 8 ft wide (north 6 to 14 ft, head 7 ft).
# The south wall stands 8 ft of its 24 (a jamb of 4 ft each side) with a head across: a wall with an opening, not "something hung from the ceiling". Four walls, two openings
# 16 ft and 8 ft wide, head 7 ft, and the walls close.
J17="$T/g17.json"; L17="$(new_job "$J17")"
F17="$("$P" find-walls "$J17" "$T/garage.ply" --level "$L17")"
echo "$F17" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
python3 - "$T" <<PY17
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F17"""
w = walls_said(said)
check("walls found", len(w), 4, 0, "walls")
doors = sorted((d for x in w for d in x["doors"]), key=lambda d: d["wide"])
check("openings", len(doors), 2, 0, "openings")
check("the narrower, 8 ft", doors[0]["wide"], 96, 4); check("the wider, 16 ft", doors[1]["wide"], 192, 5)
check("their head", doors[1]["head"], 84, 2); check("their head", doors[0]["head"], 84, 2)
assert "the walls found close: every end meets another wall" in said, said
assert "reaches the ceiling and not the floor" not in said, said
done()
PY17

scene "$T/curved.ply" curved --density 100 --seed 1
# By hand: a room 20 ft by 12 ft, 8 ft high with a curved wall across its northwest corner: a quarter circle of 8 ft radius about (0, 12 ft north), from the west wall (at 4 ft north) to
# the north wall (at 8 ft east), 4 1/2 in thick. Its centre line is 8 x pi / 2 = 12.57 ft long (12 ft 7 in). The four outside walls are walls; the curve is not found as walls: it is said to
# be curved (radius about 8 ft, length about 12 ft 7 in) and is not laid; no chord of it is laid.
J17B="$T/c17.json"; L17B="$(new_job "$J17B")"
W17B="$(made "$("$P" type "$J17B" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
F17B="$("$P" find-walls "$J17B" "$T/curved.ply" --level "$L17B")"
echo "$F17B" | sed -n '1,/walls found/p' | sed 's/^/    | /'
says "a curved face is said to be curved" "a curved face of radius" "$F17B"
says "and not found as walls" "curved walls are not found, so it is not laid" "$F17B"
python3 - "$T" <<PY17B
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F17B"""
m = re.search(r"a curved face of radius " + LEN + ", about " + LEN + " long", said)
assert m, said
check("the curve's radius", inches(m[1]), 8 * 12, 14); check("its length", inches(m[2]), 12.57 * 12, 20)
done()
PY17B
F17BL="$("$P" find-walls "$J17B" "$T/curved.ply" --level "$L17B" --lay --outside "$W17B")"
says "only the four outside walls are laid" "laid on $L17B: 4 walls" "$F17BL"

echo
echo "== Step 18 (call 183, 8). Two levels in one scan are said, and found one at a time: the level asked for, by its floor =="
scene "$T/storey.ply" storey --density 100 --seed 1
# By hand: a room 24 ft by 12 ft; the lower floor at 0 and its ceiling at 8 ft, the upper floor at 9 ft 3 in and its ceiling at 17 ft 3 in. In the south wall a door below
# (east 10 to 13 ft, head 6 ft 8 in) and a door above (east 4 to 7 ft, head 6 ft 8 in over the upper floor). The scan holds two levels: floors at 0' 0" and 9' 3", said. Each level
# is found alone: four walls each; the lower has its door 3 ft wide; the upper, asked for by its level (elevation 9 ft 3 in), has its own door, 3 ft wide from 4 ft, head 6 ft 8 in,
# and no window with its sill at 9 ft 3 in. Nothing is said to be not likely the house (it reads 8 ft from floor to ceiling).
J18="$T/s18.json"; "$P" new "$J18" "A scanned house (not a real job)" >/dev/null
L18G="$(made "$("$P" level "$J18" ground --elevation "0 ft" --height "8 ft" --status Assumed --source "to be checked on site")")"
L18U="$(made "$("$P" level "$J18" upper --elevation "9 ft 3 in" --height "8 ft" --status Assumed --source "to be checked on site")")"
F18G="$("$P" find-walls "$J18" "$T/storey.ply" --level "$L18G")"
F18U="$("$P" find-walls "$J18" "$T/storey.ply" --level "$L18U")"
echo "$F18U" | sed -n '1,/walls found/p' | sed 's/^/    | /'
says "the two levels are said" "the scan holds 2 levels: floors at 0' 0\" and 9' 3\"" "$F18G"
! grep -q "not likely the house" <<<"$F18G$F18U" || { echo "FAIL: two levels read as a house that is not likely"; exit 1; }
python3 - "$T" <<PY18
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
for name, said, door_from in (("lower", """$F18G""", 10 * 12), ("upper", """$F18U""", 4 * 12)):
    w = walls_said(said)
    check(name + ": walls found", len(w), 4, 0, "walls")
    doors = [d for x in w for d in x["doors"]]; wins = [d for x in w for d in x["windows"]]
    check(name + ": doorways", len(doors), 1, 0, "doorways"); check(name + ": windows", len(wins), 0, 0, "windows")
    check(name + ": the doorway's width", doors[0]["wide"], 36, 3); check(name + ": where it begins", doors[0]["from"], door_from, 4); check(name + ": its head", doors[0]["head"], 80, 3)
    assert re.search(r"the level found is the one whose floor is at " + ("0' 0\"" if name == "lower" else "9' 3\""), said), said
done()
PY18

echo
echo "== Step 19 (call 183, 6). A flat ceiling is not said to slope or to be two heights; a small closet's walls are as high as its ceiling =="
for SEED in 1 3 4; do
  scene "$T/closet$SEED.ply" closet6 --density 100 --seed "$SEED"
  # By hand: a closet 6 ft by 3 ft, 8 ft high, a flat ceiling. Four walls, each 8 ft (96 in) high. No word of a slope, of two heights, or of the ceiling not being one height.
  JC="$T/cl$SEED.json"; LC="$(new_job "$JC")"
  WC="$(made "$("$P" type "$JC" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
  FC="$("$P" find-walls "$JC" "$T/closet$SEED.ply" --level "$LC" --lay --outside "$WC")"
  ! grep -qE "slopes|not one height" <<<"$FC" || { echo "FAIL: seed $SEED: said of a flat ceiling:"; echo "$FC" | grep -E "slopes|not one height"; exit 1; }
  says "seed $SEED: four walls laid" "laid on $LC: 4 walls" "$FC"
  python3 - "$T" "$JC" "$SEED" <<PY19
import sys
sys.path.insert(0, sys.argv[1])
from said import *
walls = wall_lines(sys.argv[2])
assert len(walls) == 4, walls
for p, w in walls.items():
    check("seed " + sys.argv[3] + ": a wall's height", w["height"]["value"], 96, 1)
done()
PY19
done

echo
echo "== Step 20 (call 183, 7). How thick the scan is, place by place: a part ten times thinner is said by where, and read by its own thickness =="
scene "$T/dens.ply" dens --density 100 --seed 1
# By hand: a house 40 ft by 14 ft, 8 ft high; a partition 4 1/2 in thick at 20 ft east; the west 20 ft scanned at 100 points to the square foot and the east 20 ft at 10.
# Five walls: the south and the north each one wall 40 ft long, the east and west walls 14 ft, the partition 4 1/2 in. The walls close. The scan is said to be about 100 to the
# square foot in one part and about 10 in the other, with where: the floor is read in blocks of 8 ft, so the thin part, which begins at 20 ft east, is said to begin at 16 or at
# 24 ft east, and to run to 40 ft east.
JD="$T/d20.json"; LD="$(new_job "$JD")"
FD="$("$P" find-walls "$JD" "$T/dens.ply" --level "$LD")"
echo "$FD" | sed -n '/not as thick everywhere/p;/walls found/,/squared/p' | sed 's/^/    | /'
says "the scan is said to be uneven, with where" "the scan is not as thick everywhere: about " "$FD"
python3 - "$T" <<PY20A
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FD"""
m = re.search(r"and about \d+ points to the square foot from (\d+)' 0\" east, 0' 0\" north to 40' 0\" east, 16' 0\" north", said)
assert m, said
check("where the thinner part begins, east (hand 20 ft, in blocks of 8 ft)", int(m[1]) * 12, 20 * 12, 4 * 12)
done()
PY20A
says "read by parts, and joined" "each part is read by its own thickness and what each finds is joined" "$FD"
python3 - "$T" <<PY20
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$FD"""
w = walls_said(said)
check("walls found", len(w), 5, 0, "walls")
check("walls of 40 ft (the south and the north, each whole)", len([x for x in w if abs(x["long"] - 480) < 12]), 2, 0, "walls")
part = [x for x in w if x["thick"] is not None]
assert len(part) == 1, w
check("the partition's thickness", part[0]["thick"], 4.5, 0.5)
assert "the walls found close: every end meets another wall" in said, said
m = re.findall(r"about (\d+) points to the square foot from", said)
assert len(m) == 2, said
check("how thick the thicker part is", int(m[0]), 100, 20); check("how thick the thinner part is", int(m[1]), 10, 4)
done()
PY20

# ---- call 188: what three real phone scans showed, each step with its figures worked by hand ----
# The measures of the scene `phone` (rough, bowed, leaning, uneven, in passes) are those of three real phone scans, written beside it in tools/made-up-scan.py;
# no real scan is in the repository and none is read here.

echo
echo "== Step 21 (call 188, 1, 2 and 5). A phone's scan: rough, bowed, off plumb, uneven, scanned in passes: four walls, the bow said, a wall scanned twice found once =="
scene "$T/phone.ply" phone --density 120
# By hand: a room 12 ft by 14 ft, 8 ft high. The south wall (12 ft) bows 1 1/2 in, its middle that far off its ends; the west wall is scanned twice, the passes 10 in apart,
# each at half the density; the north and east walls are scanned 8 times as thickly as the south and west; every wall is 2.5 cm rough and leans a degree.
# Four walls, none in doubt: the south and the north 12 ft (144 in), the east and the west 14 ft (168 in), each seen from one side, none given a thickness (the two passes
# of the west wall are not the two faces of a 10 in wall). The south wall is said to bow 1 1/2 in over 12 ft (to an inch and a foot). The west wall is one wall, and it
# is said that 2 faces were joined into it. The walls close.
J21="$T/d21.json"; L21="$(new_job "$J21")"
F21="$("$P" find-walls "$J21" "$T/phone.ply" --level "$L21")"
echo "$F21" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
says "the south wall's bow is said" "bowed 1 1/2 in over 12 ft" "$F21"
says "a wall scanned twice is one wall, and how many were joined is said" "2 faces of this wall were joined into one" "$F21"
says "the walls close" "the walls found close: every end meets another wall" "$F21"
python3 - "$T" <<PY21
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F21"""
w = walls_said(said)
check("walls found", len(w), 4, 0, "walls")
assert all(x["thick"] is None for x in w), ("a thickness was told that the scan does not give", w)
assert "in doubt" not in said.split("walls found")[1].split("\n")[0], said
check("walls 12 ft long", len([x for x in w if abs(x["long"] - 144) <= 12]), 2, 0, "walls")
check("walls 14 ft long", len([x for x in w if abs(x["long"] - 168) <= 12]), 2, 0, "walls")
m = re.search(r"bowed (\d+)(?: (\d+)/(\d+))? in over (\d+) ft(?: (\d+) in)?", said)
assert m, said
check("how deep the bow is", int(m[1]) + (int(m[2]) / int(m[3]) if m[2] else 0), 1.5, 1.0)
check("how long it runs", int(m[4]) * 12 + (int(m[5]) if m[5] else 0), 144, 12)
assert len(re.findall(r"bowed", said)) == 1, "only the south wall bows"
done()
PY21
FS21="$("$P" find-walls "$J21" "$T/phone.ply" --level "$L21" --lay --outside "$(made "$("$P" type "$J21" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")")"
says "all four laid, Assumed" "laid on $L21: 4 walls" "$FS21"

echo
echo "== Step 22 (call 188, 3). Which way is up, told by shape: a house on its side is said, before anything else, and nothing is laid on its word =="
scene "$T/g22.ply" house --density 60 --set d=14
# By hand: a house 30 ft by 14 ft and 8 ft high, written z up. Read with y up (--up y) what stands from "floor" to "ceiling" is the room's 14 ft depth, a height a room
# can have, so the reading of the height alone says nothing is wrong. The two largest flat level things in a room are its floor and its ceiling (each 30 by 14 ft, 420 sq ft);
# across the direction taken as up the two largest are its south and north walls (30 by 8 ft, 240 sq ft each), 8 ft apart: the other way up fits the shape better, and
# the scan says so, with the 8 ft between the two and what each pair covers, before any wall.
J22="$T/d22.json"; L22="$(new_job "$J22")"
F22="$("$P" find-walls "$J22" "$T/g22.ply" --level "$L22" --up y)"
echo "$F22" | sed -n '1,4p' | sed 's/^/    | /'
says "the other way up is said first" "this is not likely the house: check the measure and which way is up (the two largest flat level things in a room, its floor and its ceiling, lie " "$F22"
says "and across what" "across, facing north and south, and cover " "$F22"
says "and what to try" "say --up z or --up y, whichever was not taken" "$F22"
python3 - "$T" <<PY22
import sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F22"""
check("said before the first wall", said.index("not likely the house") < said.index("walls found"), 1, 0, "")
import re
m = re.search(r"lie " + LEN + r" across, facing north and south, and cover (\d+) sq ft; across the direction taken as up the two largest cover (\d+) sq ft", said)
assert m, said
check("how far apart the two flat level things are (hand 8 ft, the bins 1 in)", inches(m[1]), 96, 4)
check("what they cover (hand 2 x 30 x 14 = 840 sq ft, in squares of a foot)", int(m[2]), 840, 160, "sq ft")
check("what the two largest across the direction taken as up cover (hand 2 x 30 x 8 = 480)", int(m[3]), 480, 130, "sq ft")
done()
PY22
W22="$(made "$("$P" type "$J22" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
refused "not laid on its word" "nothing is laid: this is not likely the house" "$P" find-walls "$J22" "$T/g22.ply" --level "$L22" --up y --lay --outside "$W22"
[ "$("$P" show "$J22" | grep -c '\[wall\]' || true)" = 0 ] || { echo "FAIL: a wall was laid from a house on its side"; exit 1; }
G22="$("$P" find-walls "$J22" "$T/g22.ply" --level "$L22")"
! grep -q "which way is up" <<<"$G22" || { echo "FAIL: the house the right way up was said to be on its side"; exit 1; }
echo "    the same house the right way up is not said to be on its side"

echo
echo "== Step 23 (call 188, 4). A large scan says how it is going and is thinned first, and that is said =="
scene "$T/big23.ply" house --density 250
# By hand: the house of step 1 (30 ft by 16 ft, a partition 4 1/2 in thick at 18 ft) at 250 points to the square foot: a floor and a ceiling of 480 sq ft, walls of 736 sq ft,
# the partition's two faces 256 sq ft, a little over 1,900 sq sf of surface in all: about 480,000 points, over the 300,000 that are read whole. It is thinned to under 300,000,
# one point in 2, the same points each time; the line saying so gives both counts. Progress is said as it goes, a line at a time. The five walls are found all the same, the
# partition 4 1/2 in thick (to 1/2 in), and the walls close.
J23="$T/d23.json"; L23="$(new_job "$J23")"
F23="$("$P" find-walls "$J23" "$T/big23.ply" --level "$L23" 2>&1)"
echo "$F23" | grep -E '^  \.\.\.|thinned' | sed 's/^/    | /'
says "the thinning is said" "it was thinned first, one point in 2 kept" "$F23"
says "progress is said as it goes" "... reading " "$F23"
says "progress, line by line" "... looking for walls: line 1 of at most" "$F23"
says "progress, after the lines" "faces found; telling which are walls" "$F23"
python3 - "$T" <<PY23
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F23"""
m = re.search(r"the scan holds ([\d,]+) points, more than can be read in good time: it was thinned first, one point in 2 kept \(([\d,]+) points\)", said)
assert m, said
before, kept = int(m[1].replace(",", "")), int(m[2].replace(",", ""))
assert before > 300000 and kept <= 300000 and abs(kept * 2 - before) < before * 0.02, (before, kept)
w = walls_said(said)
check("walls found", len(w), 5, 0, "walls")
part = [x for x in w if x["thick"] is not None]
assert len(part) == 1, w
check("the partition's thickness", part[0]["thick"], 4.5, 0.5)
assert "the walls found close: every end meets another wall" in said, said
done()
PY23
F23B="$("$P" find-walls "$J23" "$T/big23.ply" --level "$L23" 2>&1)"
[ "$(grep -v '^real\|^user\|^sys' <<<"$F23")" = "$(grep -v '^real\|^user\|^sys' <<<"$F23B")" ] || { echo "FAIL: the thinned scan did not read the same twice"; exit 1; }
echo "    read twice, it said the same"

echo
echo "== Step 24 (call 188, 1). What is left out and is 6 ft or longer is offered by its number as likely a wall, and laid only on the person's word =="
scene "$T/hung.ply" hung --density 100
# By hand: a room 14 ft by 12 ft, 8 ft high, with a face 8 ft long hanging from the ceiling, from 3 ft up (a beam, a soffit), 5 ft in from the south wall. It reaches the ceiling and
# not the floor, with nothing standing before it, so it is no wall the finder can tell; it is 8 ft long, so it is offered by its number as likely a wall, and not laid by --lay. The
# four outside walls are laid; with --take for the offer it is laid too.
J24="$T/d24.json"; L24="$(new_job "$J24")"
W24="$(made "$("$P" type "$J24" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
F24="$("$P" find-walls "$J24" "$T/hung.ply" --level "$L24")"
echo "$F24" | grep -E 'wall [0-9]+:|likely' | sed 's/^/    | /'
says "a long face left out is offered" "likely a wall: a face " "$F24"
says "by its number" "it is not laid unless you say it is a wall (--take " "$F24"
python3 - "$T" <<PY24
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F24"""
offers = re.findall(r"likely a wall: a face (" + LEN + r") long that reaches the ceiling, its foot not seen over (" + LEN + ")", said)
assert len(offers) == 1, said
check("the offered face's length", inches(offers[0][0]), 96, 12)
check("how much of its foot was not seen", inches(offers[0][2]), 96, 12)
done()
PY24
FL24="$("$P" find-walls "$J24" "$T/hung.ply" --level "$L24" --lay --outside "$W24")"
says "the four outside walls are laid, the offer is not" "laid on $L24: 4 walls" "$FL24"
J24B="$T/d24b.json"; L24B="$(new_job "$J24B")"; W24B="$(made "$("$P" type "$J24B" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
N24="$(grep -E 'likely a wall' -B1 <<<"$F24" | grep -oE 'wall [0-9]+:' | head -1 | grep -oE '[0-9]+')"
FT24="$("$P" find-walls "$J24B" "$T/hung.ply" --level "$L24B" --lay --take "$N24" --outside "$W24B")"
says "one taken on the person's word is laid" "laid on $L24B: 5 walls" "$FT24"

echo
echo "== Step 25 (call 189). The same phone's scan found the way the published method finds it, by the points near the ceiling (--by-ceiling) =="
# By hand: the room of step 21, 12 ft by 14 ft and 8 ft high, every wall leaning a degree and 2.5 cm rough, the south wall bowed 1 1/2 in, the west wall scanned twice 10 in apart.
# The points in the top tenth of the height (7 ft 2 in up) and not within 3 in of the ceiling are laid on the plan as how dense they are; furniture does not reach them. Their
# outlines are made straight (to the scan's own roughness: a bow of 1 in 96 is not a corner), the two sides of each thin outline are one face, and a face is a wall seen from one
# side: four walls, none given a thickness (the two passes of the west wall are joined into one face: said), none in doubt. The south and the north are 12 ft (144 in), the east
# 14 ft (168 in); the west is 14 ft too, but its top is thin in this scan and it is found over 11 ft 6 in of it (138 in, to a foot): the walls then do not close at its north end,
# and that is said with where.
# The south wall's bow (1 1/2 in by hand) is said where the face has points enough to show it (the unit test `a_bowed_wall_is_one_wall_and_its_bow_is_said` has it at 300 to the square
# foot); this south wall is scanned at an eighth of the density of the north and east, and its bow is under the inch said, so here it is said at most once, and of no other wall.
# The two finders are not asked to agree to the inch.
J25="$T/d25.json"; L25="$(new_job "$J25")"
F25="$("$P" find-walls "$J25" "$T/phone.ply" --level "$L25" --by-ceiling)"
echo "$F25" | sed -n '/walls found/,/squared/p' | sed 's/^/    | /'
says "the walls were found by their tops" "walls found by their tops" "$("$P" find-walls "$J25" "$T/phone.ply" --level "$L25" --by-ceiling 2>&1)"
says "a wall scanned twice is one wall, and how many were joined is said" "faces of this wall were joined into one" "$F25"
python3 - "$T" <<PY25
import re, sys
sys.path.insert(0, sys.argv[1])
from said import *
said = """$F25"""
w = walls_said(said)
check("walls found", len(w), 4, 0, "walls")
assert all(x["thick"] is None for x in w), ("a thickness was told that the scan does not give", w)
assert "in doubt" not in said.split("walls found")[1].split("\n")[0], said
check("walls about 12 ft long (the south, the north, the west found over 11 ft 6 in of its 14)", len([x for x in w if abs(x["long"] - 144) <= 12]), 3, 0, "walls")
check("the east wall 14 ft", len([x for x in w if abs(x["long"] - 168) <= 12]), 1, 0, "walls")
assert "the walls found do not close" in said, "the west wall's top is thin in this scan and it stops short of the north-west corner: that is said, and not hidden"
assert len(re.findall(r"bowed", said)) <= 1, "only the south wall bows"
assert "everything found is Assumed" in said, said
done()
PY25
W25="$(made "$("$P" type "$J25" wall "Existing outside wall" --thickness "6 in" --status Assumed --source "to be checked on site")")"
FS25="$("$P" find-walls "$J25" "$T/phone.ply" --level "$L25" --by-ceiling --lay --outside "$W25")"
says "all four laid, Assumed" "laid on $L25: 4 walls" "$FS25"
refused "a second laying is refused" "already has 4 walls found in scan" "$P" find-walls "$J25" "$T/phone.ply" --level "$L25" --by-ceiling --lay --outside "$W25"

echo
echo "PASS: walls, doors, windows and what stands short found in a scan, said in full and laid on the word, to the figures worked by hand"
