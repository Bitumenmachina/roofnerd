#!/usr/bin/env bash
# A room line drawn in the window (DECISIONS.md call 153; the command line's half is
# demos/roomline.sh). The house of roomline.sh step 1: a 24 ft by 16 ft house, its east
# wall to demolish, an addition of three new walls beyond it, and one room, "Living", at
# 6,8 that takes the whole inside (550.25 SF) and no room line. Opened in the window on its
# plan: the Room line tool is taken in hand and its two ends clicked at 24,0 and 24,16
# (where the east wall stood), and then a second room, "Kitchen", is placed at 30,8 with the
# Room tool. The job is saved. What the window said, the saved job and the pictures are each
# checked.
# Run from the repository root:  demos/roomlinew.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display, which
# needs Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job is made
# up (no real job data). Every step fails the script if the program does not do what it
# says. Worked by hand (roomline.sh step 2): inside the faces the house is 35.5 ft by
# 15.5 ft = 550.25 SF; parted at x = 24, the west room is 23.75 by 15.5 = 368.125 SF and the
# east room 11.75 by 15.5 = 182.125 SF; the two add to 550.25.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/roomlinew.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL" 2>/dev/null || { echo "demos/roomlinew.sh needs python's PIL, to measure the pictures"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/roomlinew.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"; APP="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/roomlinew}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
N=(--status Assumed --source "made-up addition plan")
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; head -60 "$1"; exit 1; }; printf '    %s\n' "$2"; }
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }

echo "== The house after its addition: one room, \"Living\", the whole inside; no room line (roomline.sh step 1) =="
"$P" new "$J" "Open plan in the window (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" mark "$J" "$EAST" demolish "${K[@]}" >/dev/null
"$P" wall "$J" "$S" "$WT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${N[@]}" >/dev/null
LIVING="$(made "$("$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}")")"
"$P" show "$J" > "$T/show.txt"; block "$LIVING" > "$T/living.txt"
says "$T/living.txt" "floor area       550.25 SF"
if grep -q "room line" "$T/show.txt"; then echo "FAIL: the job begins with a room line"; exit 1; fi
cp "$J" "$T/before.json"

echo
echo "== The session: on the plan, Room line in hand, its two ends clicked; then Kitchen with the Room tool; save =="
# At each shot the pointer is parked, so that what is measured is the line and not the
# pointer.
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
on "Plan"; sleep 8; park
plan_line > "$T/plan.txt"
shot                                     # 01 the plan as opened: one room, no line
on "Room line"; sleep 3                  # Room line in hand
at 24 0; sleep 2                         # one end: the south wall's end
at 24 16; sleep 4                        # the other: the north wall's end
esc; esc; park
shot                                     # 02 the room line drawn, dashed, where the east wall stood
on "Room"; sleep 2                       # Room
at 30 8; sleep 2; typed "Kitchen"        # a second room, in the addition
esc; park
shot                                     # 03 two rooms, each with its floor area
xdotool key F2; sleep 4                  # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  6 pictures saved to $OUT (model and sheet, after each of 3 steps)"

echo
echo "== What the window said =="
cp "$T/map.txt.said" "$T/said.txt"
says "$T/said.txt" "Room line: from here. Click the other end."
says "$T/said.txt" "It parts two rooms where no wall stands."
says "$T/said.txt" "Added room \"Kitchen\": "
if grep -qiE 'roomline|room_line' "$T/said.txt"; then echo "FAIL: the window said a program word"; grep -iE 'roomline|room_line' "$T/said.txt"; exit 1; fi

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
"$P" show "$J" > "$T/show.txt"
KITCHEN="$(grep -E '^ *[a-z]+-[0-9]+ +Kitchen ' "$T/show.txt" | grep -oE '[a-z]+-[0-9]+' | head -1)"
[ -n "$KITCHEN" ] || { echo "FAIL: no room called Kitchen in the saved job"; exit 1; }
block "$LIVING" > "$T/living.txt"; block "$KITCHEN" > "$T/kitchen.txt"
says "$T/living.txt" "floor area       368.125 SF"
says "$T/kitchen.txt" "floor area       182.125 SF"
N_LINES="$(grep -c "\[room line\]" "$T/show.txt")"
[ "$N_LINES" = 1 ] || { echo "FAIL: wanted one room line, the job has $N_LINES"; exit 1; }
printf '    one room line, as the command line gives it\n'
python3 -I - "$J" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))["data"]
lines = [e["attributes"]["pl::room_line"]["line"] for e in d if "pl::room_line" in e["attributes"]]
(a, b) = lines[0]["value"]
print("    the room line runs from (%.2f, %.2f) to (%.2f, %.2f): %s, %s" % (a[0], a[1], b[0], b[1], lines[0]["status"], lines[0]["source"]))
if max(abs(a[0] - 24), abs(b[0] - 24), abs(a[1] - 0), abs(b[1] - 16)) > 0.01:
    print("FAIL: wanted a line from (24, 0) to (24, 16): the ends snapped to the wall ends"); sys.exit(1)
if lines[0]["status"] != "Assumed" or "on screen" not in lines[0]["source"]:
    print("FAIL: the line is not Assumed and entered on screen"); sys.exit(1)
PY

echo
echo "== The pictures, measured =="
# The plan says where it lies on screen ($T/plan.txt: where its zero is and the pixels to
# a foot). The room line runs along x = 24 from the south wall's face to the north wall's:
# 0.25 ft to 15.75 ft north. It is drawn dashed: along that column the picture is measured,
# row by row, for a dark pixel within 1 pixel of the column, and the dark rows are counted
# as runs. A dashed line is several runs with gaps between them; before the tool there is
# none. As measured on a run of these steps (column, rows 0.25 to 15.75 ft):
#   before the tool (01): 2 runs, 0.01 of the rows dark (the line is not there: the check "a dashed line" FAILS on it)
#   after the tool (02):  24 runs, 0.49 of the rows dark (the plan 23.95 pixels to the foot); the same with Kitchen (03)
# The same measure on picture 01 fails the check (it must), and so does the same line moved
# 6 ft west, where the room line is not (a check that passes anywhere is no check).
python3 -I - "$OUT" "$T/plan.txt" <<'PY'
import sys
from PIL import Image
out, planfile = sys.argv[1], sys.argv[2]
x0, y0, k = map(float, open(planfile).read().split())
def runs(name, east=24.0, north0=0.25, north1=15.75, lum=110):
    im = Image.open("%s/%s-model.png" % (out, name)).convert("RGB")
    px = im.load()
    x = int(round(x0 + east * k))
    rows = range(int(round(y0 - north1 * k)), int(round(y0 - north0 * k)) + 1)
    dark = []
    for y in rows:
        hit = False
        for dx in (-1, 0, 1):
            p = px[x + dx, y]
            if (p[0] * 299 + p[1] * 587 + p[2] * 114) / 1000 < lum:
                hit = True
        dark.append(hit)
    n_runs = sum(1 for i, d in enumerate(dark) if d and (i == 0 or not dark[i - 1]))
    return n_runs, sum(dark) / len(dark)
def dashed(r):
    n, share = r
    return n >= 4 and 0.25 <= share <= 0.8
print("    the plan on screen: zero at (%.0f, %.0f), %.2f pixels to the foot" % (x0, y0, k))
before, after, after3 = runs("01"), runs("02"), runs("03")
west = runs("02", east=18.0)
print("    along x = 24: before the tool (01) %d runs, %.2f dark; after (02) %d runs, %.2f dark; with Kitchen (03) %d runs, %.2f dark" % (before + after + after3))
print("    along x = 18 (no line there), picture 02: %d runs, %.2f dark" % west)
ok = True
for what, r, want in (("before the tool (01): no dashed line", before, False), ("after the tool (02): a dashed line", after, True), ("with Kitchen (03): the dashed line still", after3, True), ("x = 18 in picture 02: no dashed line", west, False)):
    good = dashed(r) == want
    ok = ok and good
    print("    %-44s %s" % (what, "as wanted" if good else "FAIL"))
sys.exit(0 if ok else 1)
PY
echo
echo "  PASS: in the window, on the plan, the Room line tool drew a dashed line from 24,0 to 24,16 (Assumed, entered on screen) that parted Living into 368.125 SF and, with Kitchen placed beyond it, 182.125 SF, saved as one room line; the pictures show the dashed line and none before"
