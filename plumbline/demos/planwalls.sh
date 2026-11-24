#!/usr/bin/env bash
# Walls, doors and windows found on a drawn plan (a PDF with lines of its own), said in full and
# laid only on the person's word (DECISIONS.md call 181). Run from the repository root:
#   demos/planwalls.sh
#
# Written step for step from STATUS.md, "Walls found on a drawn plan: a demonstration, written
# before it is built", with each figure worked by hand there. The drawing is one the program
# writes itself (`plan-set`), so every figure of it is known: no real plan is in the repository.
# Job A (never opened here; it only makes the sheet): a ring 36 by 16 ft of 6 in walls, 9 ft
# high, the west 24 ft existing and the east 12 ft new, a door 3 ft wide in the new east wall
# (6 ft 6 in to 9 ft 6 in north) and a window 5 ft wide in the south wall (8 to 13 ft east), two
# rooms. Its proposed plan (sheet A-103, page 3) is brought into an empty job B. Each check prints
# what the program said beside the hand figure and fails where it is further off than the stated
# nearness (a wall's line within 1/2 in, a thickness within 1/8 in, an opening's sides within 1 in).
# Python 3 with its standard library only (it reads what the program said back as numbers, and
# writes the extra lines of steps 6 and 7 into a sheet with a small PDF writer of its own).
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
K=(--status Known --source "made-up survey")
A=(--status Assumed --source "made-up sheet")
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -60; exit 1; }; printf '    said: %s\n' "$2"; }
lacks() { if grep -qF -- "$2" <<<"$3"; then echo "FAIL: $1: said, and should not be: $2"; echo "$3" | head -60; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { local out; if out="$("${@:3}" 2>&1)"; then echo "FAIL: $1: taken, and should have been refused"; exit 1; fi; says "$1" "$2" "$out"; }

# What a hand figure is checked with: the sentences the program said, read back as numbers (inches).
cat > "$T/said.py" <<'PY'
import re, sys

def inches(text):
    """A length as the program writes it, 18' 0", -0' 2", 0' 4 1/2", to inches."""
    m = re.fullmatch(r"(-?)(\d+)' (\d+)(?: (\d+)/(\d+))?\"", text.strip())
    n = int(m[2]) * 12 + int(m[3]) + (int(m[4]) / int(m[5]) if m[4] else 0)
    return -n if m[1] else n

LEN = r"(-?\d+' \d+(?: \d+/\d+)?\")"

def walls_said(text):
    """Each wall found: {n, from, to, long, thick, doors, windows, doubt}, read from what was said."""
    out = []
    for line in text.splitlines():
        m = re.match(r"\s+wall (\d+): from " + LEN + " east, " + LEN + " north to " + LEN + " east, " + LEN + " north, " + LEN + " long, " + LEN + " thick", line)
        if m:
            g = m.groups()
            out.append({"n": int(g[0]), "from": (inches(g[1]), inches(g[2])), "to": (inches(g[3]), inches(g[4])), "long": inches(g[5]), "thick": inches(g[6]), "doors": [], "windows": [], "doubt": False})
            continue
        m = re.match(r"\s+door or open doorway: " + LEN + " to " + LEN + r" from its \w+ end, " + LEN + " wide", line)
        if m:
            g = m.groups(); out[-1]["doors"].append((inches(g[0]), inches(g[1])))
        m = re.match(r"\s+window: " + LEN + " to " + LEN + r" from its \w+ end, " + LEN + " wide", line)
        if m:
            g = m.groups(); out[-1]["windows"].append((inches(g[0]), inches(g[1])))
        if re.match(r"\s+in doubt, at ", line):
            out[-1]["doubt"] = True
    return out

fails = []
def check(what, program, hand, near, unit="in"):
    ok = abs(program - hand) <= near
    print("    %-58s program %9.3f %s   hand %9.3f   (within %s %s) %s" % (what, program, unit, hand, near, unit, "ok" if ok else "FAIL"))
    if not ok:
        fails.append(what)

def done():
    if fails:
        print("FAIL:", "; ".join(fails)); sys.exit(1)
PY

# A made-up sheet is written by this small PDF writer: the lines of a page of the plan set as the program
# reads them (`plumbline page ... --lines`) and any lines added to them. Nothing here is a real plan.
cat > "$T/mkpdf.py" <<'PY'
import re, subprocess, sys
# usage: mkpdf.py <program> <in.pdf> <page> <out.pdf> [extra lines "x0,y0,x1,y1" ...]
prog, src, page, out = sys.argv[1:5]
extra = [tuple(float(v) for v in a.split(",")) for a in sys.argv[5:] if not a.startswith("d:")]
dashed = [tuple(float(v) for v in a[2:].split(",")) for a in sys.argv[5:] if a.startswith("d:")]
text = subprocess.run([prog, "page", src, "--page", page, "--lines"], capture_output=True, text=True, check=True).stdout
size = re.search(r"(\d+) by (\d+) points", text)
w, h = int(size[1]), int(size[2])
lines = [tuple(map(float, re.findall(r"-?\d+\.\d+", l)[:4])) for l in text.splitlines() if l.startswith("line ")]
lines += extra
body = "0.5 w\n" + "".join("%.4f %.4f m %.4f %.4f l S\n" % l for l in lines)
if dashed:
    body += "[6 3] 0 d\n" + "".join("%.4f %.4f m %.4f %.4f l S\n" % l for l in dashed) + "[] 0 d\n"
objs = ["<< /Type /Catalog /Pages 2 0 R >>", "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 %d %d] /Contents 4 0 R /Resources << >> >>" % (w, h),
        "<< /Length %d >>\nstream\n%sendstream" % (len(body), body)]
data = b"%PDF-1.4\n"; offs = []
for i, o in enumerate(objs, 1):
    offs.append(len(data)); data += ("%d 0 obj\n%s\nendobj\n" % (i, o)).encode()
x = len(data)
data += ("xref\n0 %d\n0000000000 65535 f \n" % (len(objs) + 1)).encode() + b"".join(("%010d 00000 n \n" % o).encode() for o in offs)
data += ("trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objs) + 1, x)).encode()
open(out, "wb").write(data)
PY

echo "== Job A makes the sheet: a house 36 by 16 ft, the west 24 ft existing, the east 12 ft new, a door and a window =="
JA="$T/a.json"
"$P" new "$JA" "A house to draw (not a real job)" >/dev/null
SA="$(made "$("$P" level "$JA" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$JA" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$JA" door "door 3068" --width "3 ft" --height "6 ft 8 in" "${K[@]}")")"
NT="$(made "$("$P" type "$JA" window "window 5036" --width "5 ft" --height "3 ft 8 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$JA" "$SA" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" mark "$JA" "$EAST" demolish "${K[@]}" >/dev/null
"$P" mark "$JA" "$SOUTH" "$NORTH" "$WEST" existing "${K[@]}" >/dev/null
read -r ADDS ADDE ADDN <<<"$("$P" wall "$JA" "$SA" "$WT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" room-line "$JA" "$SA" --points "24,0 24,16" "${K[@]}" >/dev/null
"$P" door "$JA" "$ADDE" "$DT" --offset "8 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" window "$JA" "$SOUTH" "$NT" --offset "10.5 ft" --sill "3 ft" "${K[@]}" >/dev/null
"$P" room "$JA" "$SA" "Living" --at "6,8" "${K[@]}" >/dev/null
"$P" room "$JA" "$SA" "Kitchen" --at "30,8" "${K[@]}" >/dev/null
"$P" plan-set "$JA" "$SA" "$T/set.pdf" --date 2026-11-22 >/dev/null
cp "$T/set.pdf" "$T/sheet.pdf"
echo "    sheet A-103 (the proposed plan, page 3) written at 1/4 in to the foot"

new_job() { # $1 job file, $2 sheet file; prints the drawing's mark
  "$P" new "$1" "A drawn plan (not a real job)" >/dev/null
  local s; s="$(made "$("$P" level "$1" ground --elevation "0 ft" --height "9 ft" "${A[@]}")")"
  made "$("$P" drawing "$1" "$s" "A-103" --file "$2" --page "${3:-3}" "${A[@]}")"
}
# The overall dimension along the north side: the highest of the two long level lines 648 points (36 ft) long.
overall() { "$P" page "$1" --page "${2:-3}" --lines | python3 -c '
import re, sys
L = [tuple(map(float, re.findall(r"-?\d+\.\d+", l)[:4])) for l in sys.stdin if l.startswith("line ")]
H = [l for l in L if abs(l[1] - l[3]) < 1e-6 and abs(abs(l[2] - l[0]) - 648) < 0.01]
n = max(H, key=lambda l: l[1]); print("%s,%s %s,%s" % (n[0], n[1], n[2], n[3]))'; }
# The house's southwest corner on the page, worked from the sheet's own lines, not from where the plan sits on the page:
# x is the west end of the overall dimension, y the middle of the lowest pair of long level lines 6 in apart (9 points at
# 1/4 in to the foot: the two faces of the south wall, whose walls are 6 in thick).
corner() { "$P" page "$1" --page "${2:-3}" --lines | python3 -c '
import re, sys
L = [tuple(map(float, re.findall(r"-?\d+\.\d+", l)[:4])) for l in sys.stdin if l.startswith("line ")]
H = [l for l in L if abs(l[1] - l[3]) < 1e-6 and abs(l[2] - l[0]) >= 200]
top = max([l for l in H if abs(abs(l[2] - l[0]) - 648) < 0.01], key=lambda l: l[1])
x = min(top[0], top[2])
ys = sorted(set(round((a[1] + b[1]) / 2, 3) for a in H for b in H if abs(abs(a[1] - b[1]) - 9) < 0.01 and a[1] < b[1] and min(a[0], a[2]) < max(b[0], b[2]) and min(b[0], b[2]) < max(a[0], a[2])))
print("%s,%s" % (x, ys[0]))'; }
scale_it() { # $1 job, $2 drawing, $3 sheet file
  read -r FROM TO <<<"$(overall "$3" "${4:-3}")"
  "$P" calibrate "$1" "$2" --from "$FROM" --to "$TO" --distance "36 ft" --status Assumed --source "read off the drawing" >/dev/null
  "$P" lineup "$1" "$2" --page-point "$(corner "$3" "${4:-3}")" --plan-point "0,0" --status Assumed --source "the house's southwest corner" >/dev/null
}

echo
echo "== 1. Not scaled: nothing is found, and it says so =="
JB="$T/b.json"
DB="$(new_job "$JB" "$T/sheet.pdf")"
cp "$JB" "$T/b-before.json"
refused "an unscaled sheet" "has no scale yet: nothing is found on a sheet that is not scaled" "$P" find-walls "$JB" "$DB"

echo
echo "== 2. Scaled from one dimension: the overall dimension along the north side, 36 ft =="
read -r FROM TO <<<"$(overall "$T/sheet.pdf")"
OUT="$("$P" calibrate "$JB" "$DB" --from "$FROM" --to "$TO" --distance "36 ft" --status Assumed --source "read off the drawing")"
says "the scale" "36 ft is 648 on its page: 1/4\" = 1'-0\"" "$OUT"
"$P" lineup "$JB" "$DB" --page-point "$(corner "$T/sheet.pdf")" --plan-point "0,0" --status Assumed --source "the house's southwest corner" >/dev/null
cp "$JB" "$T/b-scaled.json"

echo
echo "== 3. Found, nothing laid: 6 walls, 104.00 LF, 6 in, they close, a door and a window, nothing in doubt =="
FOUND="$("$P" find-walls "$JB" "$DB")"
echo "$FOUND" | sed 's/^/    | /' | head -40
cmp -s "$JB" "$T/b-scaled.json" || { echo "FAIL: finding changed the job's file"; exit 1; }
printf '%s\n' "$FOUND" > "$T/found3.txt"
python3 - "$T" <<'PY'
import sys; sys.path.insert(0, sys.argv[1])
from said import *
text = open(sys.argv[1] + "/found3.txt").read()
w = walls_said(text)
check("walls found", len(w), 6, 0, "walls")
# By hand: (from, to) in inches, the lines of job A's walls as 24, 16, 24 existing and 12, 16, 12 new.
hand = [((0, 0), (288, 0)), ((288, 0), (432, 0)), ((0, 0), (0, 192)), ((432, 0), (432, 192)), ((0, 192), (288, 192)), ((288, 192), (432, 192))]
found = sorted(((tuple(round(v) for v in x["from"]), tuple(round(v) for v in x["to"])) for x in w))
for (hf, ht) in hand:
    best = min(w, key=lambda x: abs(x["from"][0] - hf[0]) + abs(x["from"][1] - hf[1]) + abs(x["to"][0] - ht[0]) + abs(x["to"][1] - ht[1]))
    for name, a, b in (("from, east", best["from"][0], hf[0]), ("from, north", best["from"][1], hf[1]), ("to, east", best["to"][0], ht[0]), ("to, north", best["to"][1], ht[1])):
        check("wall %s to %s: %s" % (hf, ht, name), a, b, 0.5)
    check("wall %s to %s: length (24, 16, 24, 12, 16, 12 ft)" % (hf, ht), best["long"], ((ht[0] - hf[0]) ** 2 + (ht[1] - hf[1]) ** 2) ** 0.5, 1.0)
    check("wall %s to %s: thickness" % (hf, ht), best["thick"], 6.0, 0.125)
check("the walls' lengths together (104.00 LF)", sum(x["long"] for x in w) / 12.0, 104.0, 0.5 / 12 * 6, "ft")
doors = [(x["n"], d) for x in w for d in x["doors"]]
windows = [(x["n"], d) for x in w for d in x["windows"]]
check("openings found (a door, a window)", len(doors) + len(windows), 2, 0, "openings")
check("a door or doorway: its south side, from the south end (6 ft 6 in)", doors[0][1][0], 78.0, 1.0)
check("a door or doorway: its north side (9 ft 6 in)", doors[0][1][1], 114.0, 1.0)
check("a window: its west side, from the west end (8 ft)", windows[0][1][0], 96.0, 1.0)
check("a window: its east side (13 ft)", windows[0][1][1], 156.0, 1.0)
the_door = next(x for x in w if x["doors"]); the_window = next(x for x in w if x["windows"])
check("the door is in the east wall (its east, 36 ft)", the_door["from"][0], 432.0, 0.5)
check("the window is in the south wall (its north, 0 ft)", the_window["from"][1], 0.0, 0.5)
check("walls in doubt", sum(1 for x in w if x["doubt"]), 0, 0, "walls")
done()
PY
says "the walls close" "the walls found close: every end meets another wall" "$FOUND"
says "nothing in doubt" "walls found: 6 (everything found is Assumed" "$FOUND"
lacks "no wall in doubt" "in doubt" "$FOUND"
says "the dimension lines left out" "(dimension lines, with their ticks and extension lines)" "$FOUND"
says "the border left out" "(the sheet's border)" "$FOUND"
says "the title block and key left out" "inside 2 boxes (a title block, a key, a schedule" "$FOUND"
says "the lettering left out" "too short to be a wall's (lettering, hatching, ticks" "$FOUND"
says "heights are the person's" "a plan gives no head or sill: 2 openings were found" "$FOUND"
says "nothing laid" "nothing is laid: say --lay to lay these walls on ground" "$FOUND"
# The room line is no wall (one line, dashed on a sheet that draws it): this sheet does not draw it at all, and no wall is found at 24 ft but the two that end there.
python3 - "$T" <<'PY'
import sys; sys.path.insert(0, sys.argv[1])
from said import *
w = walls_said(open(sys.argv[1] + "/found3.txt").read())
check("walls with a line at east 24 ft that is no wall's end (the room line)", sum(1 for x in w if x["from"][0] == x["to"][0] and abs(x["from"][0] - 288) < 1), 0, 0, "walls")
done()
PY
"$P" check "$JB" >/dev/null && ! grep -q '"pl::wall"' "$JB" && echo "    the job holds no wall after this step"

echo
echo "== 4. Laid with the openings' heights given: 6 walls, 1 door, 1 window, all Assumed and sourced to the sheet =="
cp "$T/b-scaled.json" "$JB"
LAID="$("$P" find-walls "$JB" "$DB" --lay --doors "6 ft 8 in" --windows "3 ft" "6 ft 8 in")"
echo "$LAID" | sed -n '/laid on/,$p' | sed 's/^/    | /'
says "laid" "laid on ground: 6 walls, 1 door, 1 window, each Assumed, source \"found on sheet A-103\"" "$LAID"
says "the heights are said" "\"said when the walls were found\"" "$LAID"
# A type the program makes is named the size first ("6 in wall": call 178, item 10; it was "wall 6 in").
says "a type made, Assumed" "made for them, Assumed: the type \"6 in wall\"" "$LAID"
"$P" check "$JB" >/dev/null && echo "    check passes"
"$P" room "$JB" "$(made "$(grep -oE 'level-[0-9]+' "$JB" | head -1)")" "Inside" --at "6,8" "${A[@]}" >/dev/null
TAKE="$("$P" takeoff "$JB")"
printf '%s\n' "$TAKE" > "$T/take4.txt"
python3 - "$T" "$JB" <<'PY'
import sys, json, re
sys.path.insert(0, sys.argv[1])
from said import *
t = open(sys.argv[1] + "/take4.txt").read()
def num(label):
    m = re.search(label + r"\s+([\d,]+\.\d+) (?:LF|SF)", t)
    return float(m[1].replace(",", ""))
check("walls (104.00 LF)", num(r"walls \d+"), 104.0, 0.01, "LF")
check("wall face, gross (104 x 9 = 936 SF)", num(r"wall face, gross"), 936.0, 1.0, "SF")
check("wall face, net (936 - 20.00 - 18.33 = 897.67 SF)", num(r"wall face, net"), 897.67, 1.0, "SF")
check("the room inside the walls (35.5 x 15.5 = 550.25 SF)", num(r"rooms \d+"), 550.25, 1.0, "SF")
data = json.load(open(sys.argv[2]))["data"]
walls = [e for e in data if "pl::wall" in e["attributes"]]
bad = [e["path"] for e in walls if e["attributes"]["pl::wall"]["line"]["status"] != "Assumed" or e["attributes"]["pl::wall"]["line"]["source"] != "found on sheet A-103"]
check("walls laid not Assumed and sourced to the sheet", len(bad), 0, 0, "walls")
check("walls laid", len(walls), 6, 0, "walls")
anyknown = [e["path"] for e in data if "pl::wall" in e["attributes"] and '"Known"' in json.dumps(e["attributes"]["pl::wall"])]
check("walls holding anything Known", len(anyknown), 0, 0, "walls")
done()
PY
cp "$JB" "$T/b-laid.json"

echo
echo "== 5. Laid with no heights given (a fresh job): 6 walls, no door, no window, and it says they are solid across 2 openings =="
JC="$T/c.json"
DC="$(new_job "$JC" "$T/sheet.pdf")"; scale_it "$JC" "$DC" "$T/sheet.pdf"
LAID="$("$P" find-walls "$JC" "$DC" --lay)"
says "laid" "laid on ground: 6 walls, 0 doors, 0 windows, each Assumed" "$LAID"
says "solid across the openings" "the walls stand across 2 openings and are priced as if solid until the openings are set" "$LAID"
"$P" check "$JC" >/dev/null && echo "    check passes"
TAKE="$("$P" takeoff "$JC")"
grep -E "wall face, net" <<<"$TAKE" | grep -qE "936\.00 SF" && echo "    net face 936.00 SF (nothing taken out for an opening not laid)" || { echo "FAIL: the net face is not 936.00 SF"; echo "$TAKE"; exit 1; }

echo
echo "== 6. Refusals and what is not a wall =="
cp "$T/b-laid.json" "$T/before-second.json"
refused "a second laying" "already has 6 walls found on sheet A-103" "$P" find-walls "$JB" "$DB" --lay --doors "6 ft 8 in" --windows "3 ft" "6 ft 8 in"
cmp -s "$JB" "$T/before-second.json" || { echo "FAIL: a refused laying changed the job's file"; exit 1; }
echo "    the job's file is byte for byte the same"
refused "a window with its head under its sill" "the head is above the sill" "$P" find-walls "$JB" "$DB" --lay --windows "6 ft" "3 ft"
refused "heights without laying" "they go with --lay" "$P" find-walls "$JB" "$DB" --doors "6 ft 8 in"
# A stair: twelve treads 10 in apart (13 lines), 3 ft long, inside the house. 10 in is 15 points at 1/4 in.
# Placed from the house's southwest corner on the page (CX, CY), so that it does not depend on where the plan sits: the stair 16 ft 3 in
# east of it and 2 ft 4 in north, the beam 10 ft 8 in east and 5 ft 1 in north.
IFS=, read -r CX CY <<<"$(corner "$T/sheet.pdf")"
STAIR=(); for i in $(seq 0 12); do STAIR+=("$(python3 -c "print('%s,%s,%s,%s' % ($CX + 292, $CY + 42 + $i * 15, $CX + 346, $CY + 42 + $i * 15))")"); done
python3 "$T/mkpdf.py" "$P" "$T/sheet.pdf" 3 "$T/stair.pdf" "${STAIR[@]}"
JS="$T/s.json"; DS="$(new_job "$JS" "$T/stair.pdf" 1)"; scale_it "$JS" "$DS" "$T/stair.pdf" 1
OUT="$("$P" find-walls "$JS" "$DS")"
says "still 6 walls" "walls found: 6 (everything found is Assumed" "$OUT"
says "the stair is a set" "1 set of 13 evenly spaced lines, 0' 10\" apart" "$OUT"
lacks "no wall in doubt from the stair" "in doubt" "$OUT"
# A beam shown, or a counter's two edges: two free lines 8 ft long and 4 in apart in the middle of a room.
python3 "$T/mkpdf.py" "$P" "$T/sheet.pdf" 3 "$T/beam.pdf" "$(python3 -c "print('%s,%s,%s,%s' % ($CX + 192, $CY + 92, $CX + 336, $CY + 92))")" "$(python3 -c "print('%s,%s,%s,%s' % ($CX + 192, $CY + 98, $CX + 336, $CY + 98))")"
JM="$T/m.json"; DM="$(new_job "$JM" "$T/beam.pdf" 1)"; scale_it "$JM" "$DM" "$T/beam.pdf" 1
OUT="$("$P" find-walls "$JM" "$DM")"
says "one in doubt" "walls found: 7, 1 of them in doubt" "$OUT"
says "with where" "meet no other wall at either end" "$OUT"
says "its place" "in doubt, at 14' 8\" east, 5' 3\" north" "$OUT"
N="$(grep -oE '\(--take [0-9]+\)' <<<"$OUT" | grep -oE '[0-9]+')"
cp "$JM" "$T/m-before.json"
LAID="$("$P" find-walls "$JM" "$DM" --lay)"
says "not laid by laying alone" "laid on ground: 6 walls" "$LAID"
says "said it was not laid" "not laid: wall $N of what was found, in doubt: say --take $N with --lay if it is a wall" "$LAID"
cp "$T/m-before.json" "$JM"
LAID="$("$P" find-walls "$JM" "$DM" --lay --take "$N")"
says "laid when taken by its number" "laid on ground: 7 walls" "$LAID"
"$P" check "$JM" >/dev/null && echo "    check passes"
refused "a wall number that was not found" "--take 9: 7 walls were found, numbered 1 to 7" "$P" find-walls "$JM" "$DM" --lay --take 9

echo
echo "== 7. A real sheet, with no hand figure: nobody has worked it by hand, and what it prints is to be looked at =="
REAL="$PWD/plans/usda-7213-two-bedroom-farmhouse.pdf"
if [ -f "$REAL" ]; then
  cp "$REAL" "$T/farm.pdf"
  JF="$T/f.json"; DF="$(new_job "$JF" "$T/farm.pdf" 4)"
  # The dimension is not picked from the sheet here (it is a scan): any two places, to have a scale to find walls by.
  "$P" calibrate "$JF" "$DF" --from "126,90" --to "666,90" --distance "41 ft 3 in" --status Assumed --source "read off the drawing" >/dev/null
  OUT="$("$P" find-walls "$JF" "$DF")"
  echo "$OUT" | sed 's/^/    | /' | head -12
  lacks "a scan is not read as lines" "walls found:" "$OUT"
  says "a scan has no lines to find walls in" "it has no drawn lines of its own, so there are none to find walls in" "$OUT"
  "$P" find-walls "$JF" "$DF" --lay >/dev/null && ! grep -q '"pl::wall"' "$JF" && echo "    nothing laid, nothing Known"
  echo "    NOBODY HAS WORKED THAT SHEET BY HAND; it is a scan of paper, with no lines of its own, and said so"
else
  echo "    not here: tools/fetch-plans.sh fetches it (23 MB). Skipped."
fi


# ---- Call 186: what the first judge broke, each step from its own made-up sheet, with the figures worked by hand ----
# The sheets are written by a second small PDF writer (standard library only): a house is lines in feet from its corner,
# 1/4 in to the foot, the corner 126 points in from the page's corner. A dimension is typed off the sheet: the house's
# outside width, from face to face.
cat > "$T/hl.py" <<'PY'
import math, sys
K, O = 18.0, 126.0

class S:
    def __init__(s): s.l = []; s.t = []
    def line(s, a, b, dash=False): s.l.append((tuple(a), tuple(b), dash))
    def rect(s, x0, y0, x1, y1):
        for a, b in (((x0, y0), (x1, y0)), ((x1, y0), (x1, y1)), ((x1, y1), (x0, y1)), ((x0, y1), (x0, y0))): s.line(a, b)
    def ring(s, x0, y0, x1, y1, t):
        for h in (t / 2, -t / 2): s.rect(x0 - h, y0 - h, x1 + h, y1 + h)
    def cut(s, a, b, lo, hi):
        L = math.hypot(b[0] - a[0], b[1] - a[1]); u = ((b[0] - a[0]) / L, (b[1] - a[1]) / L)
        on = lambda p: abs((p[0] - a[0]) * -u[1] + (p[1] - a[1]) * u[0]) < 1e-6
        t = lambda p: (p[0] - a[0]) * u[0] + (p[1] - a[1]) * u[1]
        at = lambda x: (a[0] + u[0] * x, a[1] + u[1] * x)
        out = []
        for p, q, d in s.l:
            if not (on(p) and on(q)): out.append((p, q, d)); continue
            s0, s1 = min(t(p), t(q)), max(t(p), t(q))
            if s1 <= lo or s0 >= hi: out.append((p, q, d)); continue
            if s0 < lo: out.append((at(s0), at(lo), d))
            if s1 > hi: out.append((at(hi), at(s1), d))
        s.l = out
    def opening(s, a, b, t, lo, hi, kind="door", over=0.0):
        """Both faces broken from lo+over to hi-over, jambs across at lo and hi, a door's leaf line or a window's two lines of glass."""
        L = math.hypot(b[0] - a[0], b[1] - a[1]); u = ((b[0] - a[0]) / L, (b[1] - a[1]) / L); n = (-u[1], u[0])
        pt = lambda x, o: (a[0] + u[0] * x + n[0] * o, a[1] + u[1] * x + n[1] * o)
        for h in (t / 2, -t / 2): s.cut(pt(0, h), pt(L, h), lo + over, hi - over)
        for x in (lo, hi): s.line(pt(x, -t / 2), pt(x, t / 2))
        for o in ((0.0,) if kind == "door" else (t / 6, -t / 6)): s.line(pt(lo, o), pt(hi, o))
    def tick_dim(s, a, b, off, text, side=1):
        """A dimension line on the page's points from a to b, `off` points outside, a slash tick across each end, extension lines, and
        `text` written beside it (level, or reading upward when the line is upright)."""
        L = math.hypot(b[0] - a[0], b[1] - a[1]); u = ((b[0] - a[0]) / L, (b[1] - a[1]) / L); n = (-u[1], u[0])
        pa = (a[0] + n[0] * off, a[1] + n[1] * off); pb = (b[0] + n[0] * off, b[1] + n[1] * off)
        s.pts.append((pa, pb))
        for e in (pa, pb):
            s.pts.append(((e[0] - (u[0] + n[0]) * 4, e[1] - (u[1] + n[1]) * 4), (e[0] + (u[0] + n[0]) * 4, e[1] + (u[1] + n[1]) * 4)))
        for e, f in ((a, pa), (b, pb)):
            s.pts.append(((e[0] + n[0] * 8, e[1] + n[1] * 8), (f[0] + n[0] * 8, f[1] + n[1] * 8)))
        mid = ((pa[0] + pb[0]) / 2, (pa[1] + pb[1]) / 2)
        c = (mid[0] + n[0] * 4 * side, mid[1] + n[1] * 4 * side)
        s.t.append((text, 9, c[0] - u[0] * 14, c[1] - u[1] * 14, u))
    pts = None
    def save(s, fn, clip=None, image=None, page=(1224, 792)):
        body = ("%s re W n\n" % clip if clip else "") + "0.5 w\n"
        for dash in (False, True):
            if dash: body += "[6 3] 0 d\n"
            for p, q, d in s.l:
                if d == dash: body += "%.4f %.4f m %.4f %.4f l S\n" % (O + p[0] * K, O + p[1] * K, O + q[0] * K, O + q[1] * K)
        body += "[] 0 d\n"
        for p, q in (s.pts or []): body += "%.4f %.4f m %.4f %.4f l S\n" % (p[0], p[1], q[0], q[1])
        if s.t:
            body += "BT\n/F1 9 Tf\n" + "".join("%.4f %.4f %.4f %.4f %.4f %.4f Tm (%s) Tj\n" % (u[0], u[1], -u[1], u[0], x, y, t.replace("(", "\\(").replace(")", "\\)")) for t, sz, x, y, u in s.t) + "ET\n"
        if image: body += "q %.1f 0 0 %.1f %.1f %.1f cm /Im1 Do Q\n" % tuple(image)
        res = b"/XObject << /Im1 5 0 R >>" if image else b""
        if s.t: res += b" /Font << /F1 6 0 R >>"
        objs = [b"<< /Type /Catalog /Pages 2 0 R >>", b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
                b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 %d %d] /Contents 4 0 R /Resources << %s >> >>" % (page[0], page[1], res),
                b"<< /Length %d >>\nstream\n%s\nendstream" % (len(body), body.encode())]
        if image:
            hexed = "".join("C8" if (i // 8 + i % 8) % 2 == 0 else "78" for i in range(64)) + ">"
            objs.append(b"<< /Type /XObject /Subtype /Image /Width 8 /Height 8 /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /ASCIIHexDecode /Length %d >>\nstream\n%s\nendstream" % (len(hexed), hexed.encode()))
        elif s.t:
            objs.append(b"<< /Type /XObject /Subtype /Form /BBox [0 0 1 1] /Length 0 >>\nstream\n\nendstream")
        if s.t:
            objs.append(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>")
        out = b"%PDF-1.4\n"; offs = []
        for i, o in enumerate(objs, 1):
            offs.append(len(out)); out += b"%d 0 obj\n" % i + o + b"\nendobj\n"
        x = len(out)
        out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objs) + 1) + b"".join(b"%010d 00000 n \n" % o for o in offs)
        out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objs) + 1, x)
        open(fn, "wb").write(out)

name, out = sys.argv[1:3]
s = S(); s.pts = []; clip = None; image = None; page = (1224, 792)
if name == "piers":   # 30 x 16 ft, 6 in walls: south wall a door 10-13 ft and a window 14 ft 6 in to 19 ft 6 in (a pier 1 ft 6 in between); a partition 4 1/2 in at 6 ft with a door 9 in from the south wall
    s.ring(0, 0, 30, 16, 0.5)
    s.opening((0, 0), (30, 0), 0.5, 10, 13, "door"); s.opening((0, 0), (30, 0), 0.5, 14.5, 19.5, "window")
    s.cut((0, 0.25), (30, 0.25), 6 - 0.1875, 6 + 0.1875); s.cut((0, 15.75), (30, 15.75), 6 - 0.1875, 6 + 0.1875)
    s.line((6 - 0.1875, 0.25), (6 - 0.1875, 15.75)); s.line((6 + 0.1875, 0.25), (6 + 0.1875, 15.75))
    s.opening((6, 0.25), (6, 15.75), 0.375, 0.75, 3.75, "door")
elif name == "layers":   # 30 x 20 ft, each wall six lines: 8 in over all, lines 1/2 in and 1 in in from each face
    for t in (8, 7, 6): s.ring(0, 0, 30, 20, t / 12)
elif name == "counter":   # 30 x 20 ft, 6 in walls, a counter's edge 4 1/2 in off the north wall's inner face, 24 ft of its 30
    s.ring(0, 0, 30, 20, 0.5); s.line((3, 19.75 - 4.5 / 12), (27, 19.75 - 4.5 / 12))
elif name == "single":   # an L-shaped house drawn with one line for each wall, and a partition as one line
    p = [(0, 0), (30, 0), (30, 12), (18, 12), (18, 24), (0, 24)]
    for i in range(6): s.line(p[i], p[(i + 1) % 6])
    s.line((9, 0), (9, 24))
elif name == "overshoot":   # 30 x 16 ft, a door 3 ft wide at 10 to 13 ft in the south wall, the faces run 1 in past each jamb into the opening
    s.ring(0, 0, 30, 16, 0.5); s.opening((0, 0), (30, 0), 0.5, 10, 13, "door", over=1 / 12)
elif name == "cut":   # 30 x 16 ft, 6 in walls, a cover over the page that shows only the west 20 ft
    s.ring(0, 0, 30, 16, 0.5); clip = "100 100 %.1f 600" % (O + 20 * K - 100)
elif name == "section":   # 30 x 16 ft, 6 in walls, and beside it a wall section drawn 1.5 times larger: a wall 9 in thick, 12 ft high, a slab 6 in thick meeting it
    s.ring(0, 0, 30, 16, 0.5)
    s.line((50, 3.75), (50, 16)); s.line((50.75, 3.75), (50.75, 16)); s.line((44, 4), (50.375, 4)); s.line((44, 3.5), (50.375, 3.5))
elif name == "shelf":   # 30 x 16 ft, 6 in walls, a 4 1/2 in partition at 18 ft, and a closet shelf 12 in deep, 17 ft 6 in long, from the west wall to the partition
    s.ring(0, 0, 30, 16, 0.5)
    s.cut((0, 0.25), (30, 0.25), 17.8, 18.2); s.cut((0, 15.75), (30, 15.75), 17.8, 18.2)
    s.line((18 - 0.1875, 0.25), (18 - 0.1875, 15.75)); s.line((18 + 0.1875, 0.25), (18 + 0.1875, 15.75))
    s.line((0.25, 8), (17.8125, 8)); s.line((0.25, 9), (17.8125, 9))
elif name == "twoplans":   # the house 30 x 20 ft, 6 in walls, a partition 4 1/2 in at 12 ft; 50 ft east of it a roof plan: an eave 32 x 22 ft (centre lines) with a second line 6 in inside it all round
    page = (2600, 792)
    s.ring(0, 0, 30, 20, 0.5)
    s.cut((0, 0.25), (30, 0.25), 11.81, 12.19); s.cut((0, 19.75), (30, 19.75), 11.81, 12.19)
    s.line((12 - 0.1875, 0.25), (12 - 0.1875, 19.75)); s.line((12 + 0.1875, 0.25), (12 + 0.1875, 19.75))
    s.ring(80, 0, 112, 22, 0.5)
elif name == "doubled":   # 30 x 20 ft; each wall drawn with four lines as the real sheets draw one: the outer faces 7, 6 1/2, 7 1/2 and 8 in apart, a second line 1 in inside one face and 0.6 in inside the other
    def doubled(a, b, span):
        L = math.hypot(b[0] - a[0], b[1] - a[1]); u = ((b[0] - a[0]) / L, (b[1] - a[1]) / L); n = (-u[1], u[0])
        for o in (-span / 2, -span / 2 + 1 / 12, span / 2 - 0.05, span / 2):
            s.line((a[0] + n[0] * o, a[1] + n[1] * o), (b[0] + n[0] * o, b[1] + n[1] * o))
    doubled((-0.2, 0), (30.2, 0), 7 / 12); doubled((30, -0.2), (30, 20.2), 6.5 / 12); doubled((30.2, 20), (-0.2, 20), 7.5 / 12); doubled((0, 20.2), (0, -0.2), 8 / 12)
elif name == "glass":   # 30 x 20 ft, 6 in walls, a window 5 ft wide in the south wall (8 to 13 ft), a door 3 ft wide in the east wall (6 ft 6 in to 9 ft 6 in), a partition 4 1/2 in at 12 ft with a shower's glass 3 ft long in it (6 to 9 ft: both faces run on, a line across at each end, two lines of glass inside), and two tags
    s.ring(0, 0, 30, 20, 0.5)
    s.cut((0, 0.25), (30, 0.25), 11.81, 12.19); s.cut((0, 19.75), (30, 19.75), 11.81, 12.19)
    s.line((12 - 0.1875, 0.25), (12 - 0.1875, 19.75)); s.line((12 + 0.1875, 0.25), (12 + 0.1875, 19.75))
    s.opening((0, 0), (30, 0), 0.5, 8, 13, "window"); s.opening((30, 0), (30, 20), 0.5, 6.5, 9.5, "door")
    for y in (6, 9): s.line((12 - 0.1875, y), (12 + 0.1875, y))
    for x in (12 - 0.0625, 12 + 0.0625): s.line((x, 6), (x, 9))
    for c, r in (((30.9, 8.0), 0.35), ((30.0, 15.0), 0.2)):
        pts = [(c[0] + r * math.cos(math.radians(60 * i)), c[1] + r * math.sin(math.radians(60 * i))) for i in range(6)]
        for i in range(6): s.line(pts[i], pts[(i + 1) % 6])
elif name == "dims":   # the house 30 x 20 ft with its own dimension strings: "30'-0\"" by a line 540 points long, "20'-0\"" by one 360 points long, and "12'-0\"" by one 180 points long (10 ft at the scale the others give)
    s.ring(0, 0, 30, 20, 0.5)
    s.tick_dim((O, O), (O + 540, O), -50, "30'-0\"")
    s.tick_dim((O, O), (O, O + 360), 50, "20'-0\"")
    s.tick_dim((O, O + 360), (O + 180, O + 360), 50, "12'-0\"")
elif name == "doubtful":   # four separate pairs of lines for a house (24, 24, 14 and 14 ft, 6 in thick, the corners open 3 ft: each meets nothing, so each is in doubt), and 50 ft east a clean roof plan, an eave 32 x 22 ft (centre lines) with a second line 6 in inside it
    page = (2600, 792)
    s.line((3, -0.25), (27, -0.25)); s.line((3, 0.25), (27, 0.25)); s.line((3, 19.75), (27, 19.75)); s.line((3, 20.25), (27, 20.25))
    s.line((-0.25, 3), (-0.25, 17)); s.line((0.25, 3), (0.25, 17)); s.line((29.75, 3), (29.75, 17)); s.line((30.25, 3), (30.25, 17))
    s.ring(80, 0, 112, 22, 0.5)
elif name == "uhouse":   # call 191: three walls of 80 ft (west, south, east of an 80 ft square, open to the north: 240 LF) and inside it, 25 ft clear of it all round, a house 30 x 30 ft (120 LF)
    page = (1800, 1700)
    def wall(a, b, t=0.5):
        L = math.hypot(b[0] - a[0], b[1] - a[1]); n = (-(b[1] - a[1]) / L, (b[0] - a[0]) / L)
        for h in (t / 2, -t / 2): s.line((a[0] + n[0] * h, a[1] + n[1] * h), (b[0] + n[0] * h, b[1] + n[1] * h))
    wall((0, 80), (0, 0)); wall((0, 0), (80, 0)); wall((80, 0), (80, 80))
    s.ring(25, 25, 55, 55, 0.5)
elif name == "inline":   # call 191: two houses in line, their south walls on one line with 15 ft of bare paper between: 30 x 20 ft (100 LF) and 28 x 18 ft (92 LF)
    page = (3600, 792)
    s.ring(0, 0, 30, 20, 0.5); s.ring(45, 0, 73, 18, 0.5)
elif name == "garage":   # call 191: 50 x 20 ft, 6 in walls, a garage door 16 ft wide in the south wall (17 to 33 ft) with its leaf drawn across it
    s.ring(0, 0, 50, 20, 0.5); s.opening((0, 0), (50, 0), 0.5, 17, 33, "door")
elif name in ("loose", "oneloose"):   # call 191: a clean roof plan, an eave 32 x 22 ft (108 LF), and beside it groups of wall that meet nothing: 30, 30 and 20 ft ("loose") or 20 ft alone ("oneloose"), each more than 8 ft from the others
    page = (2600, 792)
    s.ring(0, 0, 32, 22, 0.5)
    for x, ln in ((60, 30), (80, 30), (100, 20)) if name == "loose" else ((60, 20),):
        for h in (0.25, -0.25): s.line((x + h, 0), (x + h, ln))
elif name == "wrongone":   # call 191: the house 30 x 20 ft with ONE dimension string, "30'-0\"" drawn by a line 270 points long (15 ft at 1/4 in to the foot: the string would make the sheet 1/8 in to the foot)
    s.ring(0, 0, 30, 20, 0.5)
    s.tick_dim((O, O), (O + 270, O), -50, "30'-0\"")
elif name == "badstrings":   # call 191: the house with one good string, "30'-0\"", and four written as dimensions that cannot be used: a length of nothing, inches alone, metres and millimetres
    s.ring(0, 0, 30, 20, 0.5)
    s.tick_dim((O, O), (O + 540, O), -50, "30'-0\"")
    s.tick_dim((O, O + 100), (O + 200, O + 100), -50, "0'-0\"")
    for i, t in enumerate(("240 in", "9.144 m", "2744 mm")): s.t.append((t, 9, O + 700, O + 40 * i, (1.0, 0.0)))
elif name == "picture":   # the west 15 ft as lines (south, north, west, and a partition at 15 ft), the east half a picture
    for y in (0.25, -0.25): s.line((-0.25, y), (15.1875, y))
    for y in (15.75, 16.25): s.line((-0.25, y), (15.1875, y))
    for x in (0.25, -0.25): s.line((x, -0.25), (x, 16.25))
    for x in (14.8125, 15.1875): s.line((x, 0.25), (x, 15.75))
    image = (306.0, 306.0, O + 15.5 * K, O - 9.0)
s.save(out, clip=clip, image=image, page=page)
PY
sheet_of() { python3 "$T/hl.py" "$1" "$T/$1.pdf"; }
# A sheet's scale and place from its one dimension: the outside width of the house from face to face, and its corner.
scale_wide() { # job drawing west-face-x east-face-x distance (the page's points)
  "$P" calibrate "$1" "$2" --from "$3,300" --to "$4,300" --distance "$5" --status Assumed --source "read off the drawing" > "$T/calibrated.txt"
  "$P" lineup "$1" "$2" --page-point "126,126" --plan-point "0,0" --status Assumed --source "the house's corner" >/dev/null
}

echo
echo "== 8. What the first judge broke in the walls of a drawn plan (call 186), each from a sheet of its own =="

echo
echo "-- 8.1 A beam shown dashed, wall to wall, is no wall, not even in doubt: it is said as what it may be --"
# By hand: the sheet of step 3 and two dashed lines 4 in apart (6 points) and 8 ft long (144 points), at 14 ft 8 in east and 5 ft 3 in north.
python3 "$T/mkpdf.py" "$P" "$T/sheet.pdf" 3 "$T/dashed.pdf" "d:$(python3 -c "print('%s,%s,%s,%s' % ($CX + 192, $CY + 92, $CX + 336, $CY + 92))")" "d:$(python3 -c "print('%s,%s,%s,%s' % ($CX + 192, $CY + 98, $CX + 336, $CY + 98))")"
JD="$T/d.json"; DD="$(new_job "$JD" "$T/dashed.pdf" 1)"; scale_it "$JD" "$DD" "$T/dashed.pdf" 1
OUT="$("$P" find-walls "$JD" "$DD")"
echo "$OUT" | sed 's/^/    | /' | head -30
says "still the 6 walls, none in doubt" "walls found: 6 (everything found is Assumed" "$OUT"
lacks "no wall in doubt from a dashed pair" "in doubt" "$OUT"
says "the dashed pair is said, with where" "2 dashed lines 0' 4\" apart and 8' 0\" long, at 14' 8\" east, 5' 3\" north" "$OUT"
says "and what it may be" "a wall's lines are solid, so it is not taken as a wall and not laid" "$OUT"
LAID="$("$P" find-walls "$JD" "$DD" --lay)"
says "laid: the 6 walls" "laid on ground: 6 walls" "$LAID"
TAKE="$("$P" takeoff "$JD")"
grep -E "walls +[0-9]+ +104\.00 LF|walls [0-9]+ +104\.00" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 104.00 LF (nothing for the beam)" || { echo "FAIL: the walls are not 104.00 LF"; echo "$TAKE" | head; exit 1; }

echo
echo "-- 8.2 A pier between two openings is part of its wall, however short: both openings are found --"
# By hand: south wall 30 ft, west and east 16, north 30, partition 16 = 108 LF in 5 walls; in the south wall a door 3 ft wide at 10 ft and a window
# 5 ft wide at 14 ft 6 in (a pier of 1 ft 6 in between them); in the partition a door 3 ft wide from 1 ft 0 in (a pier 9 in from the wall's face).
sheet_of piers
JP="$T/p.json"; DP="$(new_job "$JP" "$T/piers.pdf" 1)"; scale_wide "$JP" "$DP" 121.5 670.5 "30 ft 6 in"
OUT="$("$P" find-walls "$JP" "$DP")"
echo "$OUT" | sed 's/^/    | /' | head -24
says "5 walls, none in doubt" "walls found: 5 (everything found is Assumed" "$OUT"
lacks "no doubt" "in doubt" "$OUT"
says "the door" "door or open doorway: 10' 0\" to 13' 0\" from its west end, 3' 0\" wide" "$OUT"
says "the window past the pier" "window: 14' 6\" to 19' 6\" from its west end, 5' 0\" wide" "$OUT"
says "the partition's door by the wall" "door or open doorway: 1' 0\" to 4' 0\" from its south end, 3' 0\" wide" "$OUT"
says "3 openings" "a plan gives no head or sill: 3 openings were found" "$OUT"

echo
echo "-- 8.3 A wall drawn in its layers is one wall, said once; a line beside a wall's face is no second wall --"
# By hand: a 30 by 20 ft house, each wall six lines 8 in over all: 4 walls, 100 LF, 8 in, each drawn with 6 lines, none in doubt, they close.
sheet_of layers
JL="$T/l.json"; DL="$(new_job "$JL" "$T/layers.pdf" 1)"; scale_wide "$JL" "$DL" 120 672 "30 ft 8 in"
OUT="$("$P" find-walls "$JL" "$DL")"
echo "$OUT" | sed 's/^/    | /' | head -14
says "4 walls" "walls found: 4 (everything found is Assumed" "$OUT"
lacks "none in doubt" "in doubt" "$OUT"
[ "$(grep -c ', drawn with 6 lines' <<<"$OUT")" = 4 ] && echo "    each wall says it is drawn with 6 lines" || { echo "FAIL: not 4 walls drawn with 6 lines"; exit 1; }
says "said once, as a sheet" "4 walls are drawn with several lines, the lines of their layers: each is taken as one wall" "$OUT"
says "8 in over all" "0' 8\" thick" "$OUT"
says "they close" "the walls found close: every end meets another wall" "$OUT"
# A counter's edge 4 1/2 in off the north wall's inner face: the north wall is the 6 in pair three other walls match; the edge is said, not laid.
sheet_of counter
JK="$T/k.json"; DK="$(new_job "$JK" "$T/counter.pdf" 1)"; scale_wide "$JK" "$DK" 121.5 670.5 "30 ft 6 in"
OUT="$("$P" find-walls "$JK" "$DK")"
echo "$OUT" | sed 's/^/    | /' | head -14
says "5 found, 1 in doubt" "walls found: 5, 1 of them in doubt" "$OUT"
says "its line is a wall's face already" "one of its lines is already the face of a wall 6\" thick" "$OUT"
LAID="$("$P" find-walls "$JK" "$DK" --lay)"
says "only the 4 walls are laid" "laid on ground: 4 walls" "$LAID"
TAKE="$("$P" takeoff "$JK")"
grep -E "walls +[0-9]+ +100\.00 LF|walls [0-9]+ +100\.00" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 100.00 LF" || { echo "FAIL: the walls are not 100.00 LF"; echo "$TAKE" | head; exit 1; }

echo
echo "-- 8.4 A house drawn with one line for each wall is told so --"
sheet_of single
JG="$T/g.json"; DG="$(new_job "$JG" "$T/single.pdf" 1)"
"$P" calibrate "$JG" "$DG" --from "126,300" --to "666,300" --distance "30 ft" --status Assumed --source "read off the drawing" >/dev/null
OUT="$("$P" find-walls "$JG" "$DG")"
echo "$OUT" | sed 's/^/    | /' | head -12
says "single lines" "walls here are drawn as single lines: a thickness cannot be read from one line" "$OUT"
lacks "no wall made of it" "walls found:" "$OUT"

echo
echo "-- 8.5 Faces that run past a jamb do not end the wall at the jamb --"
sheet_of overshoot
JO="$T/o.json"; DO="$(new_job "$JO" "$T/overshoot.pdf" 1)"; scale_wide "$JO" "$DO" 121.5 670.5 "30 ft 6 in"
OUT="$("$P" find-walls "$JO" "$DO")"
echo "$OUT" | sed 's/^/    | /' | head -12
says "4 walls" "walls found: 4 (everything found is Assumed" "$OUT"
says "the door, jamb to jamb" "door or open doorway: 10' 0\" to 13' 0\" from its west end, 3' 0\" wide" "$OUT"
lacks "no break in the walls" "do not close" "$OUT"

echo
echo "-- 8.6 What the page cuts off, and a picture on the page, are said --"
sheet_of cut
JX="$T/x.json"; DX="$(new_job "$JX" "$T/cut.pdf" 1)"; scale_wide "$JX" "$DX" 121.5 670.5 "30 ft 6 in"
OUT="$("$P" find-walls "$JX" "$DX")"
echo "$OUT" | sed 's/^/    | /' | head -16
says "3 walls" "walls found: 3 (everything found is Assumed" "$OUT"
says "not said to close" "the walls found cannot be said to close: 2 ends run to the edge of what the page shows" "$OUT"
lacks "never said to close" "the walls found close" "$OUT"
says "where, south" "its east end, cut off at 20' 0\" east, 0' 0\" north" "$OUT"
says "where, north" "cut off at 20' 0\" east, 16' 0\" north" "$OUT"
sheet_of picture
JI="$T/i.json"; DI="$(new_job "$JI" "$T/picture.pdf" 1)"
"$P" calibrate "$JI" "$DI" --from "121.5,300" --to "399.375,300" --distance "15.4375 ft" --status Assumed --source "read off the drawing" >/dev/null
OUT="$("$P" find-walls "$JI" "$DI")"
echo "$OUT" | sed 's/^/    | /' | head -16
says "the picture is said, and not read" "not read: 1 picture on the page: nothing was read from it for walls" "$OUT"
lacks "not said that nothing else was left out" "nothing else on the sheet was left out" "$OUT"

echo
echo "-- 8.7 A head is above the floor, and above its sill, or it is refused --"
cp "$JB" "$T/before-heads.json"
refused "a door's head of nothing" "--doors: a door's head is above the floor, and 0' 0\" is not" "$P" find-walls "$JB" "$DB" --lay --doors "0 ft"
refused "a door's head below the floor" "--doors: a door's head is above the floor" "$P" find-walls "$JB" "$DB" --lay --doors "-3 ft"
refused "a window's sill below the floor" "--windows: a window's sill is at the floor or above it" "$P" find-walls "$JB" "$DB" --lay --windows "-1 ft" "4 ft"
refused "a window's head at the floor" "--windows: a window's head is above the floor" "$P" find-walls "$JB" "$DB" --lay --windows "0 ft" "0 ft"
cmp -s "$JB" "$T/before-heads.json" || { echo "FAIL: a refused head changed the job's file"; exit 1; }

echo
echo "-- 8.8 A sheet not lined up with the job says where it lays the walls; a name that is no drawing lists the job's --"
JN="$T/n.json"; DN="$(new_job "$JN" "$T/sheet.pdf")"
read -r FROM TO <<<"$(overall "$T/sheet.pdf")"
"$P" calibrate "$JN" "$DN" --from "$FROM" --to "$TO" --distance "36 ft" --status Assumed --source "read off the drawing" >/dev/null
OUT="$("$P" find-walls "$JN" "$DN")"
says "not lined up, and why" "the sheet has not been lined up with the job: its lower left corner is taken as 0, 0" "$OUT"
LAID="$("$P" find-walls "$JN" "$DN" --lay)"
says "and where it laid them" "laid at the places said above, because the sheet has not been lined up with the job" "$LAID"
refused "a name that is no drawing and no scan" "\"A-104\" is no drawing of this job and no scan is on hand by that name; the job's drawings are: \"A-103\" ($DN)" "$P" find-walls "$JN" "A-104"
refused "a mistyped name, with a flag after it" "is no drawing of this job and no scan is on hand by that name" "$P" find-walls "$JN" "A-1O3" --lay
lacks "a lined-up sheet says nothing of it" "has not been lined up" "$FOUND"

echo
echo "-- 8.9 A scale no drawing is made at is cautioned, when it is set and when walls are found, with the nearest scales --"
# By hand: 648 points set as 39 ft is 16.6 points to the foot: 0.231 in of paper to the foot. The scales either side: 1:50 (0.240 in) and 3/16 in.
JW="$T/w.json"; DW="$(new_job "$JW" "$T/sheet.pdf")"
OUT="$("$P" calibrate "$JW" "$DW" --from "$FROM" --to "$TO" --distance "39 ft" --status Assumed --source "read off the drawing")"
says "cautioned when set" "caution: " "$OUT"
says "and why" "is not a scale drawings are made at (the nearest are 1:50 and 3/16\" = 1'-0\")" "$OUT"
says "it is no refusal" "$DW: 39 ft is 648 on its page" "$OUT"
"$P" lineup "$JW" "$DW" --page-point "$(corner "$T/sheet.pdf")" --plan-point "0,0" --status Assumed --source "the house's southwest corner" >/dev/null
OUT="$("$P" find-walls "$JW" "$DW")"
says "cautioned again when walls are found" "caution: " "$OUT"
OUT="$("$P" calibrate "$JW" "$DW" --from "$FROM" --to "$TO" --distance "36 ft" --status Assumed --source "read off the drawing")"
lacks "a scale drawings are made at is not cautioned" "caution: " "$OUT"

echo
echo "-- 8.10 A section drawn beside the plan, far smaller than it, is apart from the plan and in doubt --"
# By hand: the house is 4 walls, 30 + 30 + 16 + 16 = 92 LF; the section's two walls (the 9 in wall 12 ft 3 in, the slab 6 ft 4 in) are a group of 18 ft 7 in,
# under half of 92 ft: both in doubt, said apart from the plan, and only the 4 walls are laid.
sheet_of section
JU="$T/u.json"; DU="$(new_job "$JU" "$T/section.pdf" 1)"; scale_wide "$JU" "$DU" 121.5 670.5 "30 ft 6 in"
OUT="$("$P" find-walls "$JU" "$DU")"
echo "$OUT" | sed 's/^/    | /' | head -14
says "4 walls and the 2 of the section in doubt" "walls found: 6, 2 of them in doubt" "$OUT"
says "apart from the plan" "apart from the plan: it meets no wall of the largest group of walls on the sheet (92' 0\" of wall)" "$OUT"
LAID="$("$P" find-walls "$JU" "$DU" --lay)"
says "only the house is laid" "laid on ground: 4 walls" "$LAID"

echo
echo "-- 8.11 A pair thicker than every other wall, and alone at that thickness, is in doubt; a thin partition is not --"
# By hand: 5 walls found (92 + the 4 1/2 in partition 16 = 108 LF), the 12 in shelf the sixth, in doubt: 12 in is thicker than 6 in and 4 1/2 in, and nothing else is.
sheet_of shelf
JH="$T/h.json"; DH="$(new_job "$JH" "$T/shelf.pdf" 1)"; scale_wide "$JH" "$DH" 121.5 670.5 "30 ft 6 in"
OUT="$("$P" find-walls "$JH" "$DH")"
echo "$OUT" | sed 's/^/    | /' | head -14
says "6 found, 1 in doubt" "walls found: 6, 1 of them in doubt" "$OUT"
says "why" "thicker than every other wall found on the sheet (6\" at most) and the only one that thick (a closet shelf, a bench, a ledge)" "$OUT"
LAID="$("$P" find-walls "$JH" "$DH" --lay)"
says "the 5 walls laid, the shelf not" "laid on ground: 5 walls" "$LAID"
TAKE="$("$P" takeoff "$JH")"
grep -E "walls [0-9]+ +108\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 108.00 LF (the 4 1/2 in partition found, the shelf not)" || { echo "FAIL: not 108.00 LF"; echo "$TAKE" | head; exit 1; }

echo
echo "== 9. What real city sheets showed wrong in the finder (call 188 items 6 to 9): each made-up sheet is drawn with the cause found on a real one =="
# The causes, found on three public city sheets (kept out of this repository): a wall face is drawn twice (a second line 0.6 in to
# 1 in inside it), so each wall's lines pair four ways and it was found two to four times; the lines of a thin leaf drawn inside a
# partition whose faces run on, with a cap line at each end of it, read as a window with glass; a plan and a roof plan stand on one
# sheet; and a sheet's dimension figures are its own words. The figures below are worked by hand in STATUS.md, "The drawn plan after
# real sheets".

echo
echo "-- 9.1 More than one drawing on a sheet: said, none laid until the person says which --"
# By hand: the house is 30 + 30 + 20 + 20 + the partition 20 = 120 LF in 5 walls; the roof plan's eave is 32 + 22 + 32 + 22 = 108 LF in 4
# pieces. 108 is over half of 120: of like size. Said as 2 drawings, about 120 ft and about 108 ft; nothing laid. --plan 1: 5 walls, 120.00 LF.
sheet_of twoplans
J2="$T/t2.json"; D2="$(new_job "$J2" "$T/twoplans.pdf" 1)"; scale_wide "$J2" "$D2" 121.5 670.5 "30 ft 6 in"
OUT="$("$P" find-walls "$J2" "$D2")"
echo "$OUT" | sed 's/^/    | /' | grep -E "drawing|walls found" | head -8
says "9 walls found" "walls found: 9 (everything found is Assumed" "$OUT"
says "2 drawings and their sizes" "2 drawings on this sheet: about 120 ft and about 108 ft of wall" "$OUT"
says "and what to say" "nothing is laid until you say which is the plan (--plan <1 to 2>)" "$OUT"
says "the roof plan's lines are not walls" "a roof plan's lines are not walls, and this is how you say so" "$OUT"
says "drawing 1" "drawing 1: 5 walls, about 120 ft of wall, from 0' 0\" east, 0' 0\" north" "$OUT"
says "drawing 2" "drawing 2: 4 walls, about 108 ft of wall" "$OUT"
cp "$J2" "$T/t2-before.json"
refused "laying with no word on which" "nothing is laid: 2 drawings on this sheet" "$P" find-walls "$J2" "$D2" --lay
cmp -s "$J2" "$T/t2-before.json" || { echo "FAIL: a refused laying changed the job's file"; exit 1; }
OUT="$("$P" find-walls "$J2" "$D2" --plan 1)"
says "plan 1: 5 walls" "walls found: 5 (everything found is Assumed" "$OUT"
says "taken, and what was left out" "taking drawing 1 of the sheet's 2" "$OUT"
lacks "no wall of the roof plan" "wall 6:" "$OUT"
lacks "no word that it is still waiting for a choice" "nothing is laid until you say which" "$OUT"
OUT="$("$P" find-walls "$J2" "$D2" --plan 2)"
says "plan 2: 4 walls" "walls found: 4 (everything found is Assumed" "$OUT"
refused "a drawing that is not there" "--plan 3: this sheet has 2 drawings, numbered 1 to 2" "$P" find-walls "$J2" "$D2" --plan 3
refused "drawing 0" "--plan 0: this sheet has 2 drawings" "$P" find-walls "$J2" "$D2" --plan 0
OUT="$("$P" find-walls "$J2" "$D2" --within "-1,-1" "31,21")"
says "bounded by two corners: 5 walls" "walls found: 5 (everything found is Assumed" "$OUT"
says "and said" "looked at inside the box from " "$OUT"
lacks "one drawing in the box" "drawings on this sheet" "$OUT"
LAID="$("$P" find-walls "$J2" "$D2" --within "-1,-1" "31,21" --lay)"
says "laid with the box" "laid on ground: 5 walls" "$LAID"
TAKE="$("$P" takeoff "$J2")"
grep -E "walls +[0-9]+ +120\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 120.00 LF" || { echo "FAIL: the walls are not 120.00 LF"; echo "$TAKE" | head; exit 1; }
cp "$T/t2-before.json" "$J2"
LAID="$("$P" find-walls "$J2" "$D2" --plan 1 --lay)"
says "laid with --plan 1" "laid on ground: 5 walls" "$LAID"
TAKE="$("$P" takeoff "$J2")"
grep -E "walls +[0-9]+ +120\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 120.00 LF" || { echo "FAIL: the walls are not 120.00 LF"; echo "$TAKE" | head; exit 1; }

echo
echo "-- 9.2 One wall is found once, and says how many pairs of lines were joined --"
# By hand: 30 by 20 ft, each wall four lines; four walls, 30 + 20 + 30 + 20 = 100 LF, none in doubt, each made of 4 pairs, the outer faces 7, 6 1/2, 7 1/2, 8 in apart.
sheet_of doubled
JQ="$T/q.json"; DQ="$(new_job "$JQ" "$T/doubled.pdf" 1)"; scale_wide "$JQ" "$DQ" 120.75 671.25 "30 ft 7 in"
OUT="$("$P" find-walls "$JQ" "$DQ")"
echo "$OUT" | sed 's/^/    | /' | head -16
says "4 walls" "walls found: 4 (everything found is Assumed" "$OUT"
lacks "none in doubt" "in doubt" "$OUT"
[ "$(grep -c 'found once: 4 pairs of lines lay over this one wall' <<<"$OUT")" = 4 ] && echo "    each wall says it joined 4 pairs" || { echo "FAIL: not 4 walls each joining 4 pairs"; exit 1; }
says "said once, as a sheet" "4 walls were found once: 16 pairs of lines that lay over them" "$OUT"
LAID="$("$P" find-walls "$JQ" "$DQ" --lay)"
says "laid" "laid on ground: 4 walls" "$LAID"
TAKE="$("$P" takeoff "$JQ")"
grep -E "walls +[0-9]+ +100\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 100.00 LF" || { echo "FAIL: the walls are not 100.00 LF"; echo "$TAKE" | head; exit 1; }

echo
echo "-- 9.3 A shower's glass inside a room is no window, and a tag is no opening --"
# By hand: a window 5 ft in the south wall and a door 3 ft in the east wall are the 2 openings; the shower's glass is 3 ft long in a partition
# whose faces run on, with a room both sides: not a window. Two tags (a hexagon beside the door and one on a wall) are no opening.
sheet_of glass
JG2="$T/g2.json"; DG2="$(new_job "$JG2" "$T/glass.pdf" 1)"; scale_wide "$JG2" "$DG2" 121.5 670.5 "30 ft 6 in"
OUT="$("$P" find-walls "$JG2" "$DG2")"
echo "$OUT" | sed 's/^/    | /' | grep -E "walls found|window|door|tag|glass|openings" | cut -c1-200 | head -10
says "found, none in doubt" "walls found: 5 (everything found is Assumed" "$OUT"
lacks "none in doubt" "in doubt" "$OUT"
says "2 openings, a door and a window" "a plan gives no head or sill: 2 openings were found" "$OUT"
[ "$(grep -c '      window:' <<<"$OUT")" = 1 ] && [ "$(grep -c 'door or open doorway:' <<<"$OUT")" = 1 ] && echo "    1 window, 1 door" || { echo "FAIL: not 1 window and 1 door"; exit 1; }
says "the glass is said, with where" "wide of a wall at 12' 0\" east, 7' 6\" north, and lines inside it, are not read as a window" "$OUT"
says "the tags are said" "2 tags (a window's or a door's mark" "$OUT"

echo
echo "-- 9.4 The sheet's own dimension strings are read, listed, and offered to scale by; ones that disagree are said --"
# By hand: "30'-0\"" by a line 540 points long and "20'-0\"" by one 360 points long are each 18 points to the foot (1/4 in to the foot); "12'-0\""
# by one 180 points long is 15 points to the foot: at 18 it is 10 ft. The first two agree; the third does not, and is never used unasked.
sheet_of dims
JD2="$T/d2.json"; DD2="$(new_job "$JD2" "$T/dims.pdf" 1)"
cp "$JD2" "$T/d2-before.json"
OUT="$("$P" find-walls "$JD2" "$DD2" --dimensions)"
echo "$OUT" | sed 's/^/    | /' | cut -c1-220
says "3 strings read" "the sheet's own dimension strings, 3 read from its words" "$OUT"
says "string 1" "string 1: 30'-0\" at " "$OUT"
says "string 2" "string 2: 20'-0\" at " "$OUT"
says "string 3" "string 3: 12'-0\" at " "$OUT"
says "2 of 3 agree, 1/4 in" "2 of the 3 agree: 1/4\" = 1'-0\"" "$OUT"
says "the one that does not, with what it measures" "string 3, 12'-0\", does not agree with the others: its line measures 10' 0\" at the scale they give" "$OUT"
says "7 1/2 in of paper" "its line 7 1/2 in of paper long (540 points)" "$OUT"
cmp -s "$JD2" "$T/d2-before.json" || { echo "FAIL: listing the strings changed the job's file"; exit 1; }
echo "    the job's file is byte for byte the same (nothing is scaled unasked)"
refused "no scale yet, without --scale-by" "has no scale yet: nothing is found on a sheet that is not scaled" "$P" find-walls "$JD2" "$DD2"
refused "a string that is not there" "--scale-by 4: 3 dimension strings were read on this sheet, numbered 1 to 3" "$P" find-walls "$JD2" "$DD2" --scale-by 4
OUT="$("$P" find-walls "$JD2" "$DD2" --scale-by 1)"
echo "$OUT" | sed 's/^/    | /' | grep -E "scaled by|walls found" | cut -c1-200
says "scaled by string 1" "scaled by string 1, 30'-0\": 1/4\" = 1'-0\", Assumed, its source the string" "$OUT"
says "4 walls" "walls found: 4 (everything found is Assumed" "$OUT"
grep -q "the sheet's own dimension 30'-0" "$JD2" && echo "    the job holds the scale with its source, the string" || { echo "FAIL: the scale's source is not the string"; exit 1; }
python3 - "$JD2" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))["data"]
sc = [e["attributes"]["pl::drawing"]["scale"]["distance"] for e in d if "pl::drawing" in e["attributes"]][0]
assert sc["status"] == "Assumed", sc
print("    the scale is Assumed, source: " + sc["source"])
PY
LAID="$("$P" find-walls "$JD2" "$DD2" --lay)"
says "laid" "laid on ground: 4 walls" "$LAID"
TAKE="$("$P" takeoff "$JD2")"
grep -E "walls +[0-9]+ +100\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 100.00 LF (the house 30 by 20 ft)" || { echo "FAIL: the walls are not 100.00 LF"; echo "$TAKE" | head; exit 1; }
cp "$T/d2-before.json" "$JD2"
OUT="$("$P" find-walls "$JD2" "$DD2" --scale-by 3)"
says "asked for, a string that disagrees is cautioned" "caution: string 3 does not agree with the others" "$OUT"

echo
echo "-- 9.5 A plan all in doubt beside a clean roof plan: two drawings, told from every wall found, and none laid until the person says which --"
# By hand: the house is four pairs of lines, 24 + 24 + 14 + 14 = 76 LF, each meeting nothing, so all 76 ft in doubt; the roof plan is 108 LF, clean.
# 76 is over half of 108, so two drawings, counted over every wall found, in doubt or not: 1, the roof, about 108 ft; 2, the house, about 76 ft, all of it in doubt.
# --lay alone lays nothing. --plan 1 (the roof) lays its 4 pieces on the word: 108.00 LF. --plan 2 (the house) lays none of the doubtful and says so.
sheet_of doubtful
J5="$T/t5.json"; D5="$(new_job "$J5" "$T/doubtful.pdf" 1)"; scale_wide "$J5" "$D5" 121.5 670.5 "30 ft 6 in"
OUT="$("$P" find-walls "$J5" "$D5")"
echo "$OUT" | sed 's/^/    | /' | grep -E "drawing" | cut -c1-200 || true
says "8 walls found, 4 in doubt" "walls found: 8, 4 of them in doubt" "$OUT"
says "2 drawings, the doubt said" "2 drawings on this sheet: about 108 ft and about 76 ft (76 ft of it in doubt) of wall" "$OUT"
cp "$J5" "$T/t5-before.json"
refused "laying with no word on which" "nothing is laid: 2 drawings on this sheet" "$P" find-walls "$J5" "$D5" --lay
cmp -s "$J5" "$T/t5-before.json" || { echo "FAIL: a refused laying changed the job's file"; exit 1; }
LAID="$("$P" find-walls "$J5" "$D5" --plan 1 --lay)"
says "the roof's pieces laid on the word" "laid on ground: 4 walls" "$LAID"
TAKE="$("$P" takeoff "$J5")"
grep -E "walls +[0-9]+ +108\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 108.00 LF" || { echo "FAIL: the walls are not 108.00 LF"; echo "$TAKE" | head; exit 1; }
cp "$T/t5-before.json" "$J5"
LAID="$("$P" find-walls "$J5" "$D5" --plan 2 --lay)"
says "none of the doubtful laid" "nothing is laid" "$LAID"
says "and it says so" "not laid: wall 1 of what was found, in doubt" "$LAID"
lacks "no wall laid" "laid on ground" "$LAID"
cmp -s "$J5" "$T/t5-before.json" || { echo "FAIL: laying the doubtful drawing changed the job"; exit 1; }
echo
echo "== 10. What the judge of call 191 found: one drawing is its own walls, a bare gap is two walls, a lot in doubt lays nothing, a Known scale is kept =="

echo
echo "-- 10.1 --plan takes the walls of that drawing and no others (A1) --"
# By hand: a U of three walls, 80 + 80 + 80 = 240 LF, and a house 30 x 30 ft (120 LF) standing inside the box round it. Two drawings, 240 ft and 120 ft.
# --plan 1 --lay: the U's three walls, takeoff 240.00 LF (it laid 360). --plan 2 --lay on a fresh job: the house's four walls, 120.00 LF.
sheet_of uhouse
JU="$T/c191-u.json"; DU="$(new_job "$JU" "$T/uhouse.pdf" 1)"; scale_wide "$JU" "$DU" 121.5 1570.5 "80 ft 6 in"
OUT="$("$P" find-walls "$JU" "$DU")"
says "2 drawings, 240 ft and 120 ft" "2 drawings on this sheet: about 240 ft and about 120 ft of wall" "$OUT"
cp "$JU" "$T/c191-u-before.json"
OUT="$("$P" find-walls "$JU" "$DU" --plan 1)"
echo "$OUT" | sed 's/^/    | /' | grep -E "taking|walls found" | cut -c1-200
says "the U's 3 walls" "walls found: 3 (everything found is Assumed" "$OUT"
says "and the other drawing's walls said to be left out" "taking drawing 1 of the sheet's 2: 4 walls of the sheet's other drawings (120 ft of wall) are left out" "$OUT"
LAID="$("$P" find-walls "$JU" "$DU" --plan 1 --lay)"
says "laid: 3 walls" "laid on ground: 3 walls" "$LAID"
TAKE="$("$P" takeoff "$JU")"
grep -E "walls +[0-9]+ +240\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 240.00 LF (by hand 80 + 80 + 80; it laid 360)" || { echo "FAIL: the walls are not 240.00 LF"; echo "$TAKE" | head; exit 1; }
cp "$T/c191-u-before.json" "$JU"
LAID="$("$P" find-walls "$JU" "$DU" --plan 2 --lay)"
says "laid: the house's 4 walls" "laid on ground: 4 walls" "$LAID"
TAKE="$("$P" takeoff "$JU")"
grep -E "walls +[0-9]+ +120\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 120.00 LF" || { echo "FAIL: the walls are not 120.00 LF"; echo "$TAKE" | head; exit 1; }
echo "  (A5) laying a second drawing onto the level that holds the first says what to do:"
cp "$T/c191-u-before.json" "$JU"
"$P" find-walls "$JU" "$DU" --plan 1 --lay >/dev/null
refused "a second drawing onto the same level" "already has 3 walls found on sheet A-103: laying it again would make two of each" "$P" find-walls "$JU" "$DU" --plan 2 --lay
refused "and what to do" "lay it on a level of its own (plumbline level)" "$P" find-walls "$JU" "$DU" --plan 2 --lay

echo
echo "-- 10.2 A gap wider than 8 ft is an opening only where something is drawn in it; the drawings are told at 8 ft (A2, A3) --"
# By hand: two houses in line, a south wall of 30 ft and one of 28 ft on one line with 15 ft of bare paper (14 ft 6 in between the faces): two walls, 30 + 28 = 58 LF
# on that line, no door, the gap said with where; two drawings, 100 ft and 92 ft. A garage door 16 ft wide with its leaf across it is one wall with a 16 ft opening.
sheet_of inline
JI="$T/c191-i.json"; DI="$(new_job "$JI" "$T/inline.pdf" 1)"; scale_wide "$JI" "$DI" 121.5 670.5 "30 ft 6 in"
OUT="$("$P" find-walls "$JI" "$DI")"
echo "$OUT" | sed 's/^/    | /' | grep -E "drawing|long|gap|stop for" | cut -c1-230 | head -14
says "2 drawings, 100 ft and 92 ft" "2 drawings on this sheet: about 100 ft and about 92 ft of wall" "$OUT"
lacks "no door of 14 ft 6 in" "door or open doorway" "$OUT"
says "a wall of 30 ft" "30' 0\" long" "$OUT"
says "a wall of 28 ft" "28' 0\" long" "$OUT"
lacks "no wall of 73 ft" "73' 0\" long" "$OUT"
says "the gap is said with where" "both lines of a wall stop for 14' 6\" at" "$OUT"
says "and why" "wider than 8 ft with nothing drawn in it (no leaf, no swing, no line across)" "$OUT"
LAID="$("$P" find-walls "$JI" "$DI" --plan 1 --lay)"
TAKE="$("$P" takeoff "$JI")"
grep -E "walls +[0-9]+ +100\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 100.00 LF (drawing 1 alone)" || { echo "FAIL: the walls are not 100.00 LF"; echo "$TAKE" | head; exit 1; }
sheet_of garage
JK="$T/c191-k.json"; DK="$(new_job "$JK" "$T/garage.pdf" 1)"; scale_wide "$JK" "$DK" 121.5 1030.5 "50 ft 6 in"
OUT="$("$P" find-walls "$JK" "$DK")"
says "4 walls" "walls found: 4 (everything found is Assumed" "$OUT"
says "a 16 ft opening, a door" "door or open doorway: 17' 0\" to 33' 0\" from its west end, 16' 0\" wide" "$OUT"
lacks "not split" "stop for" "$OUT"
echo "  (A5) a box that cuts walls says which:"
OUT="$("$P" find-walls "$JI" "$DI" --within "-1,-1" "15,21")"
echo "$OUT" | sed 's/^/    | /' | grep -E "the box cuts" | cut -c1-300
says "the box cuts 2 walls" "the box cuts 2 walls: " "$OUT"
says "the wall named, with where" "the wall from 0' 0\" east, 0' 0\" north to 30' 0\" east, 0' 0\" north" "$OUT"

echo
echo "-- 10.3 Where the wall found outside the largest drawing is half of it or more, nothing is laid unasked (A4) --"
# By hand: a clean roof plan of 108 LF and three groups of wall that meet nothing, 30, 30 and 20 LF (each more than 8 ft from the others): 80 ft outside it,
# over half of 108 (54): nothing laid, and 80 ft said to be in doubt. One group of 20 LF alone (under half): the 108 LF is laid, the 20 set apart, as before.
sheet_of loose
JO="$T/c191-o.json"; DO="$(new_job "$JO" "$T/loose.pdf" 1)"; scale_wide "$JO" "$DO" 121.5 706.5 "32 ft 6 in"
OUT="$("$P" find-walls "$JO" "$DO")"
echo "$OUT" | sed 's/^/    | /' | grep -E "much of|drawings on|nothing is laid" | cut -c1-260
says "80 ft in doubt" "much of this sheet is in doubt: 80 ft of wall lies outside its largest drawing (108 ft), at least half of it" "$OUT"
says "and what to say" "nothing is laid until you say which is the plan (--plan <1 to 4>)" "$OUT"
cp "$JO" "$T/c191-o-before.json"
refused "laying with no word on which" "nothing is laid: 4 drawings on this sheet" "$P" find-walls "$JO" "$DO" --lay
cmp -s "$JO" "$T/c191-o-before.json" || { echo "FAIL: a refused laying changed the job's file"; exit 1; }
LAID="$("$P" find-walls "$JO" "$DO" --plan 1 --lay)"
says "the roof plan laid on the word" "laid on ground: 4 walls" "$LAID"
TAKE="$("$P" takeoff "$JO")"
grep -E "walls +[0-9]+ +108\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 108.00 LF" || { echo "FAIL: the walls are not 108.00 LF"; echo "$TAKE" | head; exit 1; }
sheet_of oneloose
J1="$T/c191-o1.json"; D1="$(new_job "$J1" "$T/oneloose.pdf" 1)"; scale_wide "$J1" "$D1" 121.5 706.5 "32 ft 6 in"
OUT="$("$P" find-walls "$J1" "$D1")"
lacks "one small group is not in doubt as a sheet" "much of this sheet is in doubt" "$OUT"
LAID="$("$P" find-walls "$J1" "$D1" --lay)"
says "the 108 LF laid, the 20 set apart" "laid on ground: 4 walls" "$LAID"
says "and the 20 said" "not laid: wall " "$LAID"
TAKE="$("$P" takeoff "$J1")"
grep -E "walls +[0-9]+ +108\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 108.00 LF" || { echo "FAIL: the walls are not 108.00 LF"; echo "$TAKE" | head; exit 1; }

echo
echo "-- 10.4 The sheet's scale: a Known one is kept, a lone string is said to be alone, laid walls are named when it changes, strings that cannot be read are counted (B6 to B9) --"
# By hand: the house 30 x 20 ft with one string "30'-0\"" drawn 270 points long (15 ft at 1/4 in to the foot): by the string the sheet is 1/8 in to the foot, and the
# house reads 60 x 40 ft. The scale is set by hand and marked Known at 1/4 in to the foot (540 points are 30 ft).
sheet_of wrongone
JW="$T/c191-w.json"; DW="$(new_job "$JW" "$T/wrongone.pdf" 1)"
"$P" calibrate "$JW" "$DW" --from "126,126" --to "666,126" --distance "30 ft" --status Known --source "tape, measured" >/dev/null
"$P" lineup "$JW" "$DW" --page-point "126,126" --plan-point "0,0" --status Assumed --source "the house's corner" >/dev/null
OUT="$("$P" find-walls "$JW" "$DW" --dimensions)"
says "a lone string is said to be alone" "1 dimension string: nothing else on the sheet to check it against" "$OUT"
cp "$JW" "$T/c191-w-before.json"
refused "a Known scale is kept" "A Known scale is not replaced by a string read off the sheet unless you say so" "$P" find-walls "$JW" "$DW" --scale-by 1
refused "and it says what it was and what it would be" "is 1/4\" = 1'-0\", Known, from tape, measured: string 1, 30'-0\", would make it 1/8\" = 1'-0\"" "$P" find-walls "$JW" "$DW" --scale-by 1
cmp -s "$JW" "$T/c191-w-before.json" || { echo "FAIL: a refused --scale-by changed the job's file"; exit 1; }
echo "    the job's file is byte for byte the same"
OUT="$("$P" find-walls "$JW" "$DW" --scale-by 1 --anyway)"
echo "$OUT" | sed 's/^/    | /' | grep -E "scaled by|replaces|caution" | cut -c1-240
says "taken, Assumed" "scaled by string 1, 30'-0\": 1/8\" = 1'-0\", Assumed, its source the string" "$OUT"
says "what it was and what it is now" "it replaces the scale the sheet had, 1/4\" = 1'-0\" (Known, from tape, measured): the scale now is 1/8\" = 1'-0\", Assumed" "$OUT"
says "the lone string carries its caution" "caution: 1 dimension string: nothing else on the sheet to check it against" "$OUT"
python3 - "$JW" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))["data"]
sc = [e["attributes"]["pl::drawing"]["scale"]["distance"] for e in d if "pl::drawing" in e["attributes"]][0]
assert sc["status"] == "Assumed", sc
print("    the scale in the job is Assumed, source: " + sc["source"])
PY
echo "  (B8) walls laid from the sheet are named when its scale changes: the 30 x 20 ft house laid at 1/4 in (4 walls, 100.00 LF), then the scale changed:"
JV="$T/c191-v.json"; DV="$(new_job "$JV" "$T/wrongone.pdf" 1)"; scale_wide "$JV" "$DV" 121.5 670.5 "30 ft 6 in"
"$P" find-walls "$JV" "$DV" --lay >/dev/null
TAKE="$("$P" takeoff "$JV")"
grep -E "walls +[0-9]+ +100\.00 LF" <<<"$TAKE" >/dev/null && echo "    takeoff: walls 100.00 LF" || { echo "FAIL: the walls are not 100.00 LF"; echo "$TAKE" | head; exit 1; }
OUT="$("$P" find-walls "$JV" "$DV" --scale-by 1)"
echo "$OUT" | sed 's/^/    | /' | grep -E "NOT MOVED" | cut -c1-300
says "the 4 laid walls named" "NOT MOVED: 4 things were traced over it or laid from its sheet at the scale it had before: " "$OUT"
SHOWN="$("$P" show "$JV" "$DV")"
says "and the drawing says so" "traced before it was rescaled or moved" "$SHOWN"
echo "  (call 191 item 14) walls laid from the sheet ride on its scale in the verify-in-field list. By hand: the house laid whole, 100 LF of wall 9 ft high, gypsum board on each wall's face at a made-up \$3.00/SF: 900 SF, \$2,700.00 riding on the scale of its sheet (the list had none of it):"
JR="$T/c191-r.json"; DR="$(new_job "$JR" "$T/wrongone.pdf" 1)"; scale_wide "$JR" "$DR" 121.5 670.5 "30 ft 6 in"
"$P" find-walls "$JR" "$DR" --lay >/dev/null
"$P" vif "$JR" > "$T/c191-vif0.txt" || true
for w in $(grep -oE ' wall-[0-9]+' <<<"$("$P" show "$JR")" | sort -u); do
  "$P" line "$JR" "$w" "Gypsum board" face_area --division 9 >/dev/null
  "$P" set "$JR" "$w" unit_cost '$3.00/SF' --line "Gypsum board" --status Known --source "made-up price" >/dev/null
done
"$P" vif "$JR" > "$T/c191-vif1.txt"
sed 's/^/    | /' "$T/c191-vif1.txt" | grep -E "$DR|scale" | head -4
grep -F "$DR" "$T/c191-vif1.txt" | grep -qF '$2,700.00' && echo "    the sheet's scale: \$2,700.00 riding (by hand 900 SF x \$3.00)" || { echo "FAIL: \$2,700.00 is not riding on the scale"; cat "$T/c191-vif1.txt"; exit 1; }
echo "  (B9) strings that cannot be used are counted: \"0'-0\\\"\" has no length, \"240 in\" is inches alone, \"9.144 m\" and \"2744 mm\" are metric:"
sheet_of badstrings
JS="$T/c191-s.json"; DS="$(new_job "$JS" "$T/badstrings.pdf" 1)"
OUT="$("$P" find-walls "$JS" "$DS" --dimensions)"
echo "$OUT" | sed 's/^/    | /' | cut -c1-260 | head -8
says "1 string read" "the sheet's own dimension strings, 1 read from its words" "$OUT"
says "4 not read, each said" "4 strings are written as a dimension and not read: " "$OUT"
says "the zero string" "0'-0\" at " "$OUT"
says "inches alone" "240 in at " "$OUT"
says "metric" "9.144 m at " "$OUT"
says "and the millimetres" "2744 mm at " "$OUT"
lacks "no 'inf to the foot'" "inf to the foot" "$OUT"

echo
echo "PASS: the walls, doors and windows of a drawn plan were found as worked by hand, said in full, laid only on the word, all Assumed and sourced to the sheet, and what is not a wall was left out and counted"
