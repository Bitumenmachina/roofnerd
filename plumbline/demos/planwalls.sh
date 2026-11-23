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
extra = [tuple(float(v) for v in a.split(",")) for a in sys.argv[5:]]
text = subprocess.run([prog, "page", src, "--page", page, "--lines"], capture_output=True, text=True, check=True).stdout
size = re.search(r"(\d+) by (\d+) points", text)
w, h = int(size[1]), int(size[2])
lines = [tuple(map(float, re.findall(r"-?\d+\.\d+", l)[:4])) for l in text.splitlines() if l.startswith("line ")]
lines += extra
body = "0.5 w\n" + "".join("%.4f %.4f m %.4f %.4f l S\n" % l for l in lines)
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

echo
echo "PASS: the walls, doors and windows of a drawn plan were found as worked by hand, said in full, laid only on the word, all Assumed and sourced to the sheet, and what is not a wall was left out and counted"
