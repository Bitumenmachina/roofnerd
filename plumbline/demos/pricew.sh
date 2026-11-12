#!/usr/bin/env bash
# Pricing from the window with the starter assemblies: a made-up house with nothing
# priced is opened; the starter assemblies are brought in with one click; a wall is given
# an assembly from the list of those that go on a wall, then changed to another; a price
# is typed on the sheet; and the saved job is read back.
# Run from the repository root:  demos/pricew.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job and the price
# are made up. Every step fails the script if the program does not do what it says. The
# figures are worked by hand in the comments and in STATUS.md ("Pricing from the window").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/pricew.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/pricew.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/price}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }

echo "== The house, with nothing priced and no assembly in the job =="
"$P" new "$J" "Priced in the window (not a real job)"
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" door "$J" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "2.5 ft" "${K[@]}" >/dev/null
"$P" roof "$J" "Main roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 8:12 --eave-height "9 ft" "${K[@]}" >/dev/null
"$P" check "$J"
cp "$J" "$T/before.json"

echo
echo "== The session: Starter assemblies; the south wall; Assembly; lap siding, then brick; a price on the sheet; save =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
shot                                    # 01 as opened: a bare house
on "Starter assemblies"; park
shot                                    # 02 the starter assemblies brought in
click 920 600                           # the south wall, between its door and its window
on "Assembly"; park
shot                                    # 03 the assemblies that go on a wall, listed
on "Assembly Exterior wall, 2x6, lap siding"; park
shot                                    # 04 lap siding: seven lines, and the wall drawn in it
on "Assembly"
on "Assembly Exterior wall, 2x6, brick veneer"; park
shot                                    # 05 changed to brick: what came off is said, and the wall is brick
sheet_at "Unit cost of Brick veneer on wall 1"; sleep 2
xdotool type --delay 300 "24"; sleep 2
shot                                    # 06 the brick's price being typed on its line of the sheet
xdotool key Return; sleep 4
focus
shot                                    # 07 priced: the sheet by division
xdotool key F2; sleep 3                 # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06 07; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  14 pictures saved to $OUT (model and sheet, after each of 7 steps)"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
OUT_TEXT="$("$P" estimate "$J" | tr -s ' ')"
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -60; exit 1; }; printf '    %s\n' "$2"; }
# The south wall: 24 by 9 less a 21 SF door and a 15 SF window is 180 SF; at $24.00,
# $4,320.00. Typed in the window, so Assumed.
says "the brick, priced" "Brick veneer [Subcontract] 180.00 SF ~\$24.00/SF ~\$4,320.00" "$OUT_TEXT"
says "in Division 4" " 4 Masonry ~\$4,320.00 1 line" "$OUT_TEXT"
if grep -qF "Lap siding" <<<"$OUT_TEXT"; then echo "FAIL: the lap siding line is still on the wall"; exit 1; fi
echo "    no Lap siding line is left on it"
python3 - "$J" "$SOUTH" <<'PY'
import sys, json
d = {e["path"]: e for e in json.load(open(sys.argv[1]))["data"]}
wall = d[sys.argv[2]]["attributes"]
lines = [l["description"] for l in wall["pl::cost"]["lines"]]
assemblies = sum(1 for e in d.values() if e["attributes"].get("pl::class") == "Condition")
is_now = wall["pl::condition"]
print("    in the file: %d assemblies; the south wall is %r (%s) with %d lines" % (assemblies, is_now["value"], is_now["status"], len(lines)))
ok = assemblies == 30 and is_now["value"] == "Exterior wall, 2x6, brick veneer" and is_now["status"] == "Assumed"
# Framing, sheathing, barrier, insulation; brick and its ties; gypsum board and paint.
ok = ok and lines == ["Wall framing, 2x6 at 16 in", "Wall sheathing", "Weather barrier", "Batt insulation", "Gypsum board, hung and finished", "Paint, two coats", "Brick veneer", "Brick ties and flashing"]
if not ok:
    print("FAIL: wanted the brick assembly's eight lines, the shared ones first:", lines); sys.exit(1)
PY
says "the house is dressed in it" "($SOUTH): brick veneer, on its south face (the line \"Brick veneer\")" "$("$P" finishes "$J")"

echo
echo "== What the pictures hold, measured =="
python3 - "$OUT" <<'PY'
import sys
from PIL import Image
out = sys.argv[1]
def mean(name, box):
    px = list(Image.open(f"{out}/{name}-model.png").convert("RGB").crop(box).getdata())
    return tuple(round(sum(p[i] for p in px) / len(px)) for i in range(3))
ok = True
for name, what, test, want in [
    ("01", "bare", lambda c: abs(c[0] - c[2]) < 30 and c[0] > 150, "pale: nothing on it"),
    ("04", "lap siding", lambda c: abs(c[0] - c[2]) < 45 and c[0] > 150, "pale: lap siding in its own color"),
    ("05", "brick veneer", lambda c: c[0] - c[1] > 30 and c[0] - c[2] > 30, "red well over green and blue"),
]:
    c = mean(name, (880, 560, 980, 640))
    good = test(c); ok = ok and good
    print("    picture %s, the south wall (%s): %s  %s" % (name, what, c, "as wanted" if good else "FAIL: wanted " + want))
sys.exit(0 if ok else 1)
PY
echo "  PASS: the starter assemblies were brought in with one click, a wall was given one from the list and changed to another, a price was typed on the sheet, and the saved job has the brick assembly's lines, the price and its division"
