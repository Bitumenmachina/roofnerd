#!/usr/bin/env bash
# A run of cabinet boxes set in the window (DECISIONS.md call 156; the command line's half is
# demos/cabinets.sh and its figures are worked by hand in STATUS.md, "A kitchen from the tool
# chest"). The house of cabinets.sh step 1 is made with the command line, the starter items
# brought in: a 24 ft by 16 ft house, its east wall to demolish, a 12 ft addition beyond it,
# a room line at x = 24, the rooms Living and Kitchen. The Kitchen's east wall runs
# 36,0 to 36,16 and its face on the Kitchen's side is at x = 35.75, clear from y = 0.25 to
# 15.75. Opened in the window on its plan: the Cabinets tool is taken in hand, the face is
# clicked 1 ft along the wall from its first end, the boxes B18 [Dishwasher] SB36 B24 DB18 B36
# are typed (the appliance by its name in square brackets, as the command line takes it) and
# Enter is pressed: a run of 13 ft, 1.75 ft left to the corner, and the countertop over it.
# The list under the field (the boxes, the corner boxes, the appliances, one line of key), each
# box's code written on it on the plan, the wall's dimension taken from over the run, and the
# box lines on the sheet's estimate are each checked. A second run with a code the job's items
# have not (Q99; it was B99 until call 215 made a base of any width by rule: call 225) is refused in words, the job as it was. The job is saved, the view turned to
# 3D, the run taken back whole with Undo.
# Run from the repository root:  demos/cabinetsw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display, which
# needs Xvfb, xdotool and a Vulkan driver (Mesa's software one will do), and measures the
# pictures with python and PIL. The job is made up (no real job data). Every step fails the
# script if the program does not do what the step says.
#
# Worked by hand (STATUS.md): widths 18 + 24 (the dishwasher) + 36 + 24 + 18 + 36 = 156 in, a
# 13 ft run from y = 1 to y = 14, 1.75 ft left to the corner at y = 15.75. Each is 2 ft deep, so
# its middle is 1 ft off the face (x = 35.75): 34.75. B18 at (34.75, 1.75), the dishwasher at
# (34.75, 3.5), SB36 at (34.75, 6), B24 at (34.75, 8.5), DB18 at (34.75, 10.25), B36 at
# (34.75, 12.5). The dishwasher stands 34 in tall, no taller than a base box (34 1/2 in), so the
# countertop goes over it: 13 ft by 25 1/2 in (2.125 ft): 27.625 SF, its middle at
# (34.6875, 7.5), covering x = 33.625 to 35.75 and y = 1 to 14.
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
grep -qF "set: 48" <<<"$IN" || { echo "FAIL: the starter items are not 48"; echo "$IN"; exit 1; }
echo "    the east wall is $KEAST, the Kitchen is $KITCHEN, the starter items are 48"
cp "$J" "$T/before.json"
# What the command line makes of the same run, on a copy: what the window's run is set against.
cp "$J" "$T/cli.json"
"$P" cabinets "$T/cli.json" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "B18 [Dishwasher] SB36 B24 DB18 B36" --countertop "${N[@]}" >/dev/null

echo
echo "== The session: on the plan; Cabinets in hand; the east wall's face clicked 1 ft along; the boxes typed, a dishwasher among them; Enter =="
echo "   the sheet's Estimate; then a run of Q99 (refused); save; 3D; Undo; save =="
# At each shot the pointer is parked: what is measured is the run, not the pointer.
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
# Where the plan lies on screen, read again once it has come to rest: two readings four seconds apart agree.
# (Call 197 point 11, ruled 2026-11-25: taking the Cabinets tool in hand eases the plan so that the building stands clear of the
# tool's card, so each picture is measured against the plan as it lay when that picture was taken, and not against the plan as
# opened. No bound below moves.)
plan_rest() { local a b n=0; a="\$(plan_line)"; while [ "\$n" -lt 20 ]; do sleep 4; b="\$(plan_line)"; [ -n "\$b" ] && [ "\$a" = "\$b" ] && break; a="\$b"; n=\$((n + 1)); done; echo "\$b" > "\$1"; }
on "Plan"; sleep 8; park
plan_rest "$T/plan-01.txt"
found model "Delete" > /dev/null         # every tool is still on the bar, the last of them too
shot                                     # 01 the plan as opened: no boxes along the east wall
on "Cabinets"; sleep 3; park
shot                                     # 02 Cabinets in hand, its Countertop switch on
at 35.75 1; sleep 3                      # the Kitchen's east wall, its face, 1 ft along from the first end
shot                                     # 03 the run's start taken; the fields are open
cp "$T/map.txt" "$T/map-fields.txt"
plan_rest "$T/plan-04.txt"
typed "B18 [Dishwasher] SB36 B24 DB18 B36" shot       # 04 the boxes typed (picture before Enter), then Enter
park; sleep 6
plan_rest "$T/plan-05.txt"
shot                                     # 05 the run set, in plan, each box carrying its code
sheet "Estimate"; sleep 5; park
# The estimate's lines are below its recap: the wheel over the sheet, a step at a time, until
# the first box's line is on screen (the picture is taken there), and on to the end, keeping
# every row the sheet listed on the way.
n=0
while [ -z "\$(where sheet B18)" ] && [ "\$n" -lt 60 ]; do xdotool mousemove 2000 600; drawn_for 2 0.5; xdotool click 5; drawn_for 2 1.5; n=\$((n + 1)); done
sleep 2; cp "$T/map.txt" "$T/map-estimate.txt"; grep '^sheet' "$T/map.txt" > "$T/rows-seen.txt"
shot                                     # 06 the sheet's estimate: each box's line
for i in 1 2 3 4 5 6 7 8; do xdotool mousemove 2000 600; drawn_for 2 0.5; xdotool click 5; drawn_for 2 2; grep '^sheet' "$T/map.txt" >> "$T/rows-seen.txt"; done
for i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16; do xdotool mousemove 2000 600; drawn_for 2 0.5; xdotool click 4; drawn_for 2 2; grep '^sheet' "$T/map.txt" >> "$T/rows-seen.txt"; done
at 35.75 15; sleep 3                     # the same face, 15 ft along
typed "Q99" shot                         # 07 a code the job's items have not: typed, then Enter
park; sleep 3
shot                                     # 08 refused, in words; nothing set
esc; esc; park
xdotool key F2; drawn_for 3 4                  # save
cp "$J" "$T/after.json"
on "3D"; sleep 8
on "Frame"; sleep 8
on "Show See through"; sleep 6; park          # the walls as glass, so what stands inside them is seen
shot                                     # 09 the same, in 3D
on "Undo"; sleep 5; park
shot                                     # 10 undone in one step, in 3D
on "Plan"; sleep 8; park
plan_rest "$T/plan-11.txt"
shot                                     # 11 undone, in plan
xdotool key F2; drawn_for 3 4                  # save again
cp "$J" "$T/undone.json"
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1800 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 07 08 09 10 11; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
[ -s "$T/undone.json" ] || { echo "FAIL: the session did not run to its end"; exit 1; }
echo "  22 pictures saved to $OUT (model and sheet, after each of 11 steps)"

echo
echo "== What the window said =="
cp "$T/map.txt.said" "$T/said.txt"
# The run starts where the click was, and says so (call 169; the click here was 1 ft along).
says "$T/said.txt" "Cabinets: the run starts where you click, 1' 0\" along "
says "$T/said.txt" "type another start in Starts at."
# (The window says what a thing is called, not its number in the file: "wall 6", "Kitchen".)
says "$T/said.txt" "Set 6 boxes along wall "
says "$T/said.txt" " in Kitchen: a run of 13 ft, from 1 ft to 14 ft along the wall; 1.75 ft left to the corner."
says "$T/said.txt" "Undone: Set 6 boxes along wall "
says "$T/said.txt" "Countertop: 13 ft long by 25 1/2 in deep, 34 1/2 in up; 27.62 SF, front edge 13 LF."
# call 225: B99 is now a box made by the rule for B (call 215); a code no rule reads is Q99, and the model says so
says "$T/said.txt" "Can't set the run: Q99 is not in the tool chest, and no cabinet rule reads Q."
says "$T/said.txt" "The list above the field has the boxes and the appliances this job has."
# A refusal is said on the card and in the message line, once: not again in the panel, and the
# list of what there is is not said again in it either (call 161).
[ "$(grep -c "Q99 is not in" "$T/said.txt")" = 1 ] || { echo "FAIL: the refusal was said more than once"; grep "Q99" "$T/said.txt"; exit 1; }
# The window says "the tool chest" where the model does (call 169: Patrick's word, call 151); the
# stand-in "this job's items" (call 163) is gone.
if grep -qiE "this job's items" "$T/said.txt"; then echo "FAIL: the window said this job's items, not the tool chest"; exit 1; fi
if grep -qiE 'furnishing-|wall-0|room-0|Entered|Err\(' "$T/said.txt"; then echo "FAIL: the window said a program word"; grep -iE 'furnishing-|wall-0|room-0|Entered|Err\(' "$T/said.txt"; exit 1; fi

echo
echo "== What the session left in the saved job: the six things, their lines, the countertop, as the command line sets them =="
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
hand = {"B18": (34.75, 1.75), "Dishwasher": (34.75, 3.5), "SB36": (34.75, 6.0), "B24": (34.75, 8.5), "DB18": (34.75, 10.25), "B36": (34.75, 12.5)}
boxes = [b for b in window if b["what"] != "Countertop"]
if [b["what"] for b in boxes] != list(hand):
    sys.exit("FAIL: the run is not B18 [Dishwasher] SB36 B24 DB18 B36: %s" % [b["what"] for b in boxes])
for b in boxes:
    if max(abs(b["at"][0] - hand[b["what"]][0]), abs(b["at"][1] - hand[b["what"]][1])) > 1e-6:
        sys.exit("FAIL: %s stands at %s, the hand figures say %s" % (b["what"], b["at"], hand[b["what"]]))
    division = 11 if b["what"] == "Dishwasher" else 12
    if b["lines"] != [(b["what"], "1", "Material", division)]:
        sys.exit("FAIL: %s has lines %s, wanted its one line, 1 EA, Division %d" % (b["what"], b["lines"], division))
    if b["status"] != "Assumed":
        sys.exit("FAIL: %s is %s, not Assumed" % (b["what"], b["status"]))
top = [b for b in window if b["what"] == "Countertop"]
if len(top) != 1 or abs(top[0]["plan_area"] - 27.625) > 1e-9 or max(abs(top[0]["at"][0] - 34.6875), abs(top[0]["at"][1] - 7.5)) > 1e-9:
    sys.exit("FAIL: the countertop is not 27.625 SF with its middle at (34.6875, 7.5): %s" % top)
print("    the five boxes and the dishwasher are at the hand figures, each with its one line (1 EA, Material; Division 12, the dishwasher's 11), Assumed; the countertop is %.3f SF" % top[0]["plan_area"])
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
for code in B18 Dishwasher SB36 B24 DB18 B36; do
  grep -E "^  $code +Material +1 +1 EA +no price" "$T/item.txt" > /dev/null || { echo "FAIL: the estimate by item does not list $code once, 1 EA, with no price"; cat "$T/item.txt"; exit 1; }
done
echo "    the estimate by item lists each box and the dishwasher once, 1 EA, no price"
"$P" check "$T/undone.json"

echo
echo "== What the window put on screen: the list under the field, the boxes on the sheet =="
# The fields and the list under them (the map from the moment the fields were open): the card of
# boxes is not a control, so what it holds is read from the picture by eye (picture 03); what is
# checked here is what the window wrote of its controls: the two fields, the Countertop switch.
grep -q $'^model\tStarts at field\t' "$T/map-fields.txt" && grep -q $'^model\tBoxes field\t' "$T/map-fields.txt" || { echo "FAIL: the two fields are not on the map"; cat "$T/map-fields.txt" | head -30; exit 1; }
echo "    the fields Starts at and Boxes are on screen"
# The sheet's estimate, after the run: each box's line is a row of it, by the box's name (text on
# the sheet's map is the evidence: a picture measure of a table's rows is not practical).
for code in B18 Dishwasher SB36 B24 DB18 B36; do
  grep -q $'^sheet\t'"$code"$'\t' "$T/rows-seen.txt" || { echo "FAIL: the sheet's estimate has no row for $code"; sort -u "$T/rows-seen.txt" | head -60; exit 1; }
done
echo "    the sheet's estimate lists a row for each of B18, Dishwasher, SB36, B24, DB18, B36"

echo
echo "== The pictures, measured =="
# The plan says where it lies on screen ($T/plan.txt: where its zero is and the pixels to a
# foot). The countertop covers x = 33.625 to 35.75 and y = 1 to 14 and hides the boxes under
# it from above. (1) A run is measured as its colour differing from the floor beside it: the mean
# colour of a patch inside it (x 34.0 to 35.4, y 1.4 to 13.6) against the same patch 6 ft west
# (x 28.0 to 29.4, in the Kitchen) and a patch past the run's end (y 14.4 to 15.4): a run is
# there when the first differs from both by more than 30 (the sum of the three colour channels'
# differences). (2) Each box carries its code on the plan: in a window 1.2 ft wide and 0.5 ft
# along the wall, at each box's middle, the dark pixels (luminance under 110, the code's ink) are
# counted: a label has at least 6, a bare box none. (3) The wall's dimension is not written over
# the run: the figure "16' 0"" of the east wall stood inside the wall, at (34.85, 8), before the
# run (picture 01); with the run (picture 05) it is taken from over the boxes (to the other side
# of the wall, which on this plan lies under the right panel): the same window counted in the
# picture before and the picture after.
# As measured on a run of these steps (plan 23.95 pixels to the foot):
#   codes (dark pixels at each box's middle, with the run / before / 6 ft west): B18 28/0/0, Dishwasher 20/0/0,
#   SB36 32/0/0, B24 21/0/0, DB18 38/0/0, B36 24/0/0
#   the wall's 16' 0" window: 70 non-paper pixels before the run (04), 0 light pixels over the run (05), 236 in picture 04 (control)
#   run patches: 01 gaps 0 and 0; 05 gaps 281 and 283; 11 gaps 1 and 2; 05 6 ft west gaps 6 and 5
# Each check is proved able to fail: the run check on the before picture (01) and on the patches
# moved 6 ft west of the run in picture 05; the code check on picture 01 and on windows 6 ft west
# in picture 05; the dimension check on the other place in each picture.
python3 -I - "$OUT" "$T" <<'PY'
import sys
from PIL import Image
out, tmp = sys.argv[1], sys.argv[2]
# Each picture is measured against the plan as it lay when the picture was taken (call 197 point 11; ruled 2026-11-25).
def plan(name):
    return map(float, open("%s/plan-%s.txt" % (tmp, name)).read().split())
x0, y0, k = plan("01")
def mean(name, e0, e1, n0, n1):
    x0, y0, k = plan(name)
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
def dark(name, e0, e1, n0, n1, lum=110):
    x0, y0, k = plan(name)
    im = Image.open("%s/%s-model.png" % (out, name)).convert("RGB")
    px = im.load()
    n = 0
    for x in range(int(round(x0 + e0 * k)), int(round(x0 + e1 * k)) + 1):
        for y in range(int(round(y0 - n1 * k)), int(round(y0 - n0 * k)) + 1):
            p = px[x, y]
            if (p[0] * 299 + p[1] * 587 + p[2] * 114) / 1000 < lum:
                n += 1
    return n
def gap(a, b):
    return sum(abs(p - q) for p, q in zip(a, b))
def run_drawn(name, shift=0.0):
    inside = mean(name, 34.0 + shift, 35.4 + shift, 1.4, 13.6)
    west = mean(name, 28.0 + shift, 29.4 + shift, 1.4, 13.6)
    past = mean(name, 34.0 + shift, 35.4 + shift, 14.4, 15.4)
    g1, g2 = gap(inside, west), gap(inside, past)
    return g1 > 30 and g2 > 30, inside, west, past, g1, g2
for pic in ("01", "04", "05", "11"):
    print("    the plan on screen in picture %s: zero at (%.0f, %.0f), %.2f pixels to the foot" % ((pic,) + tuple(plan(pic))))
ok = True
for what, name, shift, want in (
    ("01 before the run", "01", 0.0, False),
    ("05 after the run", "05", 0.0, True),
    ("11 after Undo", "11", 0.0, False),
    ("05 again, the patches 6 ft west of the run", "05", -6.0, False),
):
    drawn, inside, west, past, g1, g2 = run_drawn(name, shift)
    good = drawn == want
    ok = ok and good
    print("    %-44s inside %s, west %s, past %s; gaps %3.0f and %3.0f: %s: %s" % (what, [round(v) for v in inside], [round(v) for v in west], [round(v) for v in past], g1, g2, "a run" if drawn else "no run", "as wanted" if good else "FAIL"))
# (2) each box's code on the plan, at the box's middle
middles = {"B18": 1.75, "Dishwasher": 3.5, "SB36": 6.0, "B24": 8.5, "DB18": 10.25, "B36": 12.5}
counts = {}
for code, n in middles.items():
    after = dark("05", 34.15, 35.35, n - 0.25, n + 0.25)
    before = dark("01", 34.15, 35.35, n - 0.25, n + 0.25)
    west = dark("05", 28.15, 29.35, n - 0.25, n + 0.25)
    counts[code] = (after, before, west)
    good = after >= 6 and before == 0 and west == 0
    ok = ok and good
    print("    code %-10s dark pixels at its middle: with the run %3d, before it %3d, 6 ft west %3d: %s" % (code, after, before, west, "as wanted" if good else "FAIL: wanted at least 6, none, none"))
# (3) the east wall's dimension is not over the run. Its figure "16' 0"" stands at (34.85, 8.1)
# when the run is not yet there (picture 04, the boxes typed): a window 1.3 ft wide and 0.35 ft
# along the wall there has pixels that are not paper (luminance under 235, at least 6). Over the
# run (picture 05) the window is the tan of the boxes: a figure on its light pill would be light
# pixels (luminance over 215) in it, and there are none. The same window in picture 04 is paper:
# every pixel in it is light, so the second test fails there, as it must.
def count(name, e0, e1, n0, n1, test):
    x0, y0, k = plan(name)
    im = Image.open("%s/%s-model.png" % (out, name)).convert("RGB")
    px = im.load()
    n = 0
    for x in range(int(round(x0 + e0 * k)), int(round(x0 + e1 * k)) + 1):
        for y in range(int(round(y0 - n1 * k)), int(round(y0 - n0 * k)) + 1):
            p = px[x, y]
            n += test((p[0] * 299 + p[1] * 587 + p[2] * 114) / 1000)
    return n
win = (34.2, 35.5, 7.85, 8.2)
text_before = count("04", *win, lambda l: l < 235)
light_after = count("05", *win, lambda l: l > 215)
light_before = count("04", *win, lambda l: l > 215)
print("    the wall's 16' 0\" window (34.85, 8.0): before the run %d non-paper pixels; over the run %d light pixels (picture 04 as a control: %d)" % (text_before, light_after, light_before))
good = text_before >= 6 and light_after == 0 and light_before > 100
ok = ok and good
# (4) The dishwasher is marked DW on the plan, as a drawing marks it (call 169), and its label stays
# inside its box. The box stands 33.75 to 35.75 ft east, its middle 3.5 ft north, and the countertop's
# slate edge runs along 33.6 to 33.8 ft: a label wider than the box lies over that edge as a light
# pill. The light pixels (luminance over 180) in that strip at the dishwasher's row (3.2 to 3.8 ft
# north) are counted: 0 on picture 05 of this run, and 47 on the picture from before this change
# (demos/captured/cabinetsw/05-model.png at the commit before it, "Dishwasher" lettered wider than its
# box), which fails the check. And DW has ink inside the box (at least 6 dark pixels).
def light(name, e0, e1, n0, n1, lum=180):
    x0, y0, k = plan(name)
    im = Image.open("%s/%s-model.png" % (out, name)).convert("RGB")
    px = im.load()
    n = 0
    for x in range(int(round(x0 + e0 * k)), int(round(x0 + e1 * k)) + 1):
        for y in range(int(round(y0 - n1 * k)), int(round(y0 - n0 * k)) + 1):
            p = px[x, y]
            if (p[0] * 299 + p[1] * 587 + p[2] * 114) / 1000 > lum:
                n += 1
    return n
ink_out = light("05", 33.6, 33.8, 3.2, 3.8)
ink_in = dark("05", 34.0, 35.5, 3.2, 3.8)
print("    the dishwasher's label: %d light pixels over the countertop's edge beside its box, %d dark pixels of it inside" % (ink_out, ink_in))
good_dw = ink_out == 0 and ink_in >= 6
ok = ok and good_dw
print("    DW inside its own box, none of it over the edge beside: %s" % ("as wanted" if good_dw else "FAIL"))
print("    the dimension is gone from over the run: %s" % ("as wanted" if good else "FAIL: wanted the figure there before (>= 6), no light pixel over the run, and the control light"))
sys.exit(0 if ok else 1)
PY
echo
echo "  PASS: in the window the Cabinets tool set B18 [Dishwasher] SB36 B24 DB18 B36 along the Kitchen's east wall from 1 ft (13 ft, 1.75 ft left, the countertop 27.625 SF), the same as the command line, drawn on the plan where the hand figures put it and not before, each box carrying its code and the wall's dimension out from over the run, the list of boxes and its key under the field, each box a row of the sheet's estimate; B99 was refused with the job as it was; Undo took the run back whole"
