#!/usr/bin/env bash
# A run of cabinet boxes set in the window (DECISIONS.md call 156; the command line's half is
# demos/cabinets.sh and its figures are worked by hand in STATUS.md, "A kitchen from the tool
# chest"). The house of cabinets.sh step 1 is made with the command line, the starter items
# brought in: a 24 ft by 16 ft house, its east wall to demolish, a 12 ft addition beyond it,
# a room line at x = 24, the rooms Living and Kitchen. The Kitchen's east wall runs
# 36,0 to 36,16 and its face on the Kitchen's side is at x = 35.75, clear from y = 0.25 to
# 15.75. Opened in the window on its plan: the Cabinets tool is taken in hand, the face is
# clicked 1 ft along the wall from its first end, the boxes B18 SB36 B24 DB18 B36 are typed and
# Enter is pressed: a run of 11 ft, 3.75 ft left to the corner, and the countertop over it.
# A second run with a code the tool chest has not (B99) is refused in words, the job as it
# was. The job is saved, the view turned to 3D, the run taken back whole with Undo.
# Run from the repository root:  demos/cabinetsw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display, which
# needs Xvfb, xdotool and a Vulkan driver (Mesa's software one will do), and measures the
# pictures with python and PIL. The job is made up (no real job data). Every step fails the
# script if the program does not do what the step says.
#
# Worked by hand (STATUS.md): widths 18 + 36 + 24 + 18 + 36 = 132 in, an 11 ft run from
# y = 1 to y = 12, 3.75 ft left to the corner at y = 15.75. Each box is 2 ft deep, so its
# middle is 1 ft off the face (x = 35.75): 34.75. B18 at (34.75, 1.75), SB36 at (34.75, 4),
# B24 at (34.75, 6.5), DB18 at (34.75, 8.25), B36 at (34.75, 10.5). The countertop is 11 ft
# by 25 1/2 in (2.125 ft): 23.375 SF, its middle at (34.6875, 6.5), covering x = 33.625 to
# 35.75 and y = 1 to 12.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/cabinetsw.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL" 2>/dev/null || { echo "demos/cabinetsw.sh needs python's PIL, to measure the pictures"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/cabinetsw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"; APP="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/cabinetsw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
N=(--status Assumed --source "made-up addition plan")
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; head -60 "$1"; exit 1; }; printf '    %s\n' "$2"; }

echo "== The house after its addition, two rooms, the starter items brought in (cabinets.sh step 1) =="
"$P" new "$J" "A kitchen in the window (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" mark "$J" "$EAST" demolish "${K[@]}" >/dev/null
read -r ADD1 KEAST ADD3 <<<"$("$P" wall "$J" "$S" "$WT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${N[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}" >/dev/null
KITCHEN="$(made "$("$P" room "$J" "$S" "Kitchen" --at "30,8" "${K[@]}")")"
"$P" room-line "$J" "$S" --points "24,0 24,16" "${N[@]}" >/dev/null
IN="$("$P" import items "$J" starter)"
grep -qF "set: 46" <<<"$IN" || { echo "FAIL: the starter tool chest is not 46 items"; echo "$IN"; exit 1; }
echo "    the east wall is $KEAST, the Kitchen is $KITCHEN, the tool chest has 46 items"
cp "$J" "$T/before.json"
# What the command line makes of the same run, on a copy: what the window's run is set against.
cp "$J" "$T/cli.json"
"$P" cabinets "$T/cli.json" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "B18 SB36 B24 DB18 B36" --countertop "${N[@]}" >/dev/null

echo
echo "== The session: on the plan; Cabinets in hand; the east wall's face clicked 1 ft along; the boxes typed; Enter =="
echo "   then a run of B99 (refused); save; 3D; Undo; save =="
# At each shot the pointer is parked: what is measured is the run, not the pointer.
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
on "Plan"; sleep 8; park
plan_line > "$T/plan.txt"
found model "Delete" > /dev/null         # every tool is still on the bar, the last of them too
shot                                     # 01 the plan as opened: no boxes along the east wall
on "Cabinets"; sleep 3; park
shot                                     # 02 Cabinets in hand, its Countertop switch on
at 35.75 1; sleep 3                      # the Kitchen's east wall, its face, 1 ft along from the first end
shot                                     # 03 the run's start taken; the fields are open
typed "B18 SB36 B24 DB18 B36" shot       # 04 the boxes typed (picture before Enter), then Enter
park; sleep 6
shot                                     # 05 the run set, in plan
at 35.75 13; sleep 3                     # the same face, 13 ft along: clear
typed "B99" shot                         # 06 a code the tool chest has not: typed, then Enter
park; sleep 3
shot                                     # 07 refused, in words; nothing set
esc; esc; park
xdotool key F2; sleep 4                  # save
cp "$J" "$T/after.json"
on "3D"; sleep 8
on "Frame"; sleep 8
on "Show See through"; sleep 6; park          # the walls as glass, so what stands inside them is seen
shot                                     # 08 the same, in 3D
on "Undo"; sleep 5; park
shot                                     # 09 undone in one step, in 3D
on "Plan"; sleep 8; park
shot                                     # 10 undone, in plan
xdotool key F2; sleep 4                  # save again
cp "$J" "$T/undone.json"
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1800 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 07 08 09 10; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
[ -s "$T/undone.json" ] || { echo "FAIL: the session did not run to its end"; exit 1; }
echo "  20 pictures saved to $OUT (model and sheet, after each of 10 steps)"

echo
echo "== What the window said =="
cp "$T/map.txt.said" "$T/said.txt"
says "$T/said.txt" "Cabinets: the run starts 1' 0\" along "
# (The window says what a thing is called, not its number in the file: "wall 6", "Kitchen".)
says "$T/said.txt" "Set 5 boxes along wall "
says "$T/said.txt" " in Kitchen: a run of 11 ft, from 1 ft to 12 ft along the wall; 3.75 ft left to the corner."
says "$T/said.txt" "Undone: Set 5 boxes along wall "
says "$T/said.txt" "Countertop: 11 ft long by 25 1/2 in deep, 34 1/2 in up; 23.375 SF, front edge 11 LF."
says "$T/said.txt" "Can't set the run: B99 is not in the tool chest. It has: "
if grep -qiE 'furnishing-|wall-0|room-0|Entered|Err\(' "$T/said.txt"; then echo "FAIL: the window said a program word"; grep -iE 'furnishing-|wall-0|room-0|Entered|Err\(' "$T/said.txt"; exit 1; fi

echo
echo "== What the session left in the saved job: the five boxes, their lines, the countertop, as the command line sets them =="
"$P" check "$T/after.json"
python3 -I - "$T/after.json" "$T/cli.json" "$T/before.json" "$T/undone.json" <<'PY'
import json, sys
def placed(path):
    d = json.load(open(path))["data"]
    out = []
    for e in d:
        a = e["attributes"]
        if "pl::placed" not in a:
            continue
        p = a["pl::placed"]
        out.append({
            "what": p["what"]["value"],
            "at": p["at"]["value"], "offset": p["offset"]["value"],
            "angle": (p.get("angle") or {}).get("value"),
            "size": p["size"]["value"],
            "lines": [(l["description"], l["quantity"], l["class"], l["division"]) for l in a.get("pl::cost", {}).get("lines", [])],
            "plan_area": a.get("pl::measure", {}).get("plan_area", {}).get("value"),
            "status": p["at"]["status"],
        })
    return out
window, cli = placed(sys.argv[1]), placed(sys.argv[2])
print("    the window's run: " + "; ".join("%s at (%.4g, %.4g)" % (b["what"], b["at"][0], b["at"][1]) for b in window))
hand = {"B18": (34.75, 1.75), "SB36": (34.75, 4.0), "B24": (34.75, 6.5), "DB18": (34.75, 8.25), "B36": (34.75, 10.5)}
boxes = [b for b in window if b["what"] != "Countertop"]
if [b["what"] for b in boxes] != list(hand):
    sys.exit("FAIL: the boxes are not B18 SB36 B24 DB18 B36: %s" % [b["what"] for b in boxes])
for b in boxes:
    if max(abs(b["at"][0] - hand[b["what"]][0]), abs(b["at"][1] - hand[b["what"]][1])) > 1e-6:
        sys.exit("FAIL: %s stands at %s, the hand figures say %s" % (b["what"], b["at"], hand[b["what"]]))
    if b["lines"] != [(b["what"], "1", "Material", 12)]:
        sys.exit("FAIL: %s has lines %s, wanted its one line, 1 EA, Division 12" % (b["what"], b["lines"]))
    if b["status"] != "Assumed":
        sys.exit("FAIL: %s is %s, not Assumed" % (b["what"], b["status"]))
top = [b for b in window if b["what"] == "Countertop"]
if len(top) != 1 or abs(top[0]["plan_area"] - 23.375) > 1e-9 or max(abs(top[0]["at"][0] - 34.6875), abs(top[0]["at"][1] - 6.5)) > 1e-9:
    sys.exit("FAIL: the countertop is not 23.375 SF with its middle at (34.6875, 6.5): %s" % top)
print("    the five boxes are at the hand figures, each with its one line (1 EA, Division 12, Material), Assumed; the countertop is %.3f SF" % top[0]["plan_area"])
if window != cli:
    for w, c in zip(window, cli):
        if w != c:
            print("      window:", w); print("      command line:", c)
    sys.exit("FAIL: the window's run is not the command line's")
print("    and the same, place for place and line for line, as `plumbline cabinets` on the same job (%d things)" % len(window))
if len(placed(sys.argv[3])) != 0 or len(placed(sys.argv[4])) != 0:
    sys.exit("FAIL: before the run, or after Undo, a box is in the job")
print("    the job before the run and the job after Undo have none of them: the run went back whole")
PY
"$P" estimate "$T/after.json" by-item > "$T/item.txt"
for code in B18 SB36 B24 DB18 B36; do
  grep -E "^  $code +Material +1 +1 EA +no price" "$T/item.txt" > /dev/null || { echo "FAIL: the estimate by item does not list $code once, 1 EA, with no price"; cat "$T/item.txt"; exit 1; }
done
echo "    the estimate by item lists each box once, 1 EA, no price"
"$P" check "$T/undone.json"

echo
echo "== The pictures, measured =="
# The plan says where it lies on screen ($T/plan.txt: where its zero is and the pixels to a
# foot). The countertop covers x = 33.625 to 35.75 and y = 1 to 12 and hides the boxes under
# it from above. A run is measured as its colour differing from the floor beside it: the mean
# colour of a patch inside it (x 34.0 to 35.4, y 1.4 to 11.6) against the same patch 6 ft west
# (x 28.0 to 29.4, in the Kitchen) and the same size patch past the run's end (y 12.6 to
# 14.6): a run is there when the first differs from both by more than 30 (the sum of the
# three colour channels' differences). As measured on a run of these steps (plan 23.95 pixels
# to the foot), the mean colour of the patch inside the run, the patch west, the patch past:
#   01 before:    (253, 253, 253), (252, 253, 253), (253, 253, 253): gaps 1 and 0, no run
#   05 after:     (190, 153, 112), (252, 252, 252), (253, 253, 253): gaps 301 and 303, a run
#   10 undone:    (252, 252, 252), (252, 252, 252), (253, 253, 253): gaps 0 and 2, no run
#   05, patches 6 ft west of the run: gaps 6 and 7, no run
# The check is proved able to fail: the same check on the before picture (01) says no run, so
# "a run is drawn" fails on it; and the same patches moved 6 ft west in the picture with the
# run (05) say no run either, so it does not pass anywhere (a check that passes anywhere is
# no check).
python3 -I - "$OUT" "$T/plan.txt" <<'PY'
import sys
from PIL import Image
out, planfile = sys.argv[1], sys.argv[2]
x0, y0, k = map(float, open(planfile).read().split())
def mean(name, e0, e1, n0, n1):
    im = Image.open("%s/%s-model.png" % (out, name)).convert("RGB")
    px = im.load()
    xs = range(int(round(x0 + e0 * k)), int(round(x0 + e1 * k)) + 1)
    ys = range(int(round(y0 - n1 * k)), int(round(y0 - n0 * k)) + 1)
    n, acc = 0, [0, 0, 0]
    for x in xs:
        for y in ys:
            p = px[x, y]
            for i in range(3):
                acc[i] += p[i]
            n += 1
    return [a / n for a in acc]
def gap(a, b):
    return sum(abs(p - q) for p, q in zip(a, b))
def run_drawn(name, shift=0.0):
    inside = mean(name, 34.0 + shift, 35.4 + shift, 1.4, 11.6)
    west = mean(name, 28.0 + shift, 29.4 + shift, 1.4, 11.6)
    past = mean(name, 34.0 + shift, 35.4 + shift, 12.6, 14.6)
    g1, g2 = gap(inside, west), gap(inside, past)
    return g1 > 30 and g2 > 30, inside, west, past, g1, g2
print("    the plan on screen: zero at (%.0f, %.0f), %.2f pixels to the foot" % (x0, y0, k))
ok = True
for what, name, shift, want in (
    ("01 before the run", "01", 0.0, False),
    ("05 after the run", "05", 0.0, True),
    ("10 after Undo", "10", 0.0, False),
    ("05 again, the patches 6 ft west of the run", "05", -6.0, False),
):
    drawn, inside, west, past, g1, g2 = run_drawn(name, shift)
    good = drawn == want
    ok = ok and good
    print("    %-44s inside %s, west %s, past %s; gaps %3.0f and %3.0f: %s: %s" % (what, [round(v) for v in inside], [round(v) for v in west], [round(v) for v in past], g1, g2, "a run" if drawn else "no run", "as wanted" if good else "FAIL"))
print("    (the check \"a run is drawn\" fails on picture 01, as it must, and on the patches moved away from the run)")
sys.exit(0 if ok else 1)
PY
echo
echo "  PASS: in the window the Cabinets tool set B18 SB36 B24 DB18 B36 along the Kitchen's east wall from 1 ft (11 ft, 3.75 ft left, the countertop 23.375 SF), the same as the command line, drawn on the plan where the hand figures put it and not before; B99 was refused with the job as it was; Undo took the run back whole"
