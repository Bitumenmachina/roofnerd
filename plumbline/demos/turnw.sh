#!/usr/bin/env bash
# A thing from the item library set down against a wall, turned to face the room, and
# changed in the library (DECISIONS.md call 107, from the twenty-fifth judgment): taking
# an item in hand keeps the plan; setting it down into a wall says so; a quarter turn
# clears it; a width changed in the written library reaches the one already set down.
# Run from the repository root:  demos/turnw.sh
#
# It drives the real program with a real pointer and keyboard. The job is made up (no
# real job data). Every step fails the script if the program does not do what the step
# says. The figures are worked by hand in the comments and in STATUS.md ("A thing
# turned, and a library change reaching it").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/turnw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/turnw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/turnw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
mkdir -p "$T/job"; J="$T/job/turned.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -60; exit 1; }; printf '    %s\n' "$2"; }

echo "== The job: four 6 in walls round 24 ft by 16 ft, and the starter items =="
"$P" new "$J" "A thing turned (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
# The second wall drawn is the east one: its line from 24 ft east, 0 north to 24, 16.
# 6 in thick about its line, its faces are at 23.75 ft and 24.25 ft east.
"$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" >/dev/null
"$P" import items "$J" starter >/dev/null
"$P" check "$J"
says "the panelboard, as the library has it" "~15 in wide, ~4 in deep, ~30 in high; 0 placed" "$("$P" items "$J" | grep -A1 "Panelboard, 200 A")"
cp "$J" "$T/before.json"

echo
echo "== The session: on plan; a panelboard set down into the east wall; turned; its width changed in the library =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
sleep 8; park
xdotool key t; sleep 8                   # the plan, north up
on_at "Item Panelboard, 200 A"; sleep 5  # in hand: the plan stays on show
park
shot                                     # 01 on plan still, the panelboard in hand
at 23.5 8; sleep 7                       # its middle 6 in inside the room from the wall's line
park
shot                                     # 02 set down, and in the wall: said, and outlined in ink
xdotool key q; sleep 7                   # a quarter turn
park
shot                                     # 03 its 4 in now toward the wall: clear
on_at "Item library (CSV)"; sleep 6      # the library written out beside the job
python3 - "$T/job/turned items.csv" <<'PY'
import csv, sys
rows = list(csv.reader(open(sys.argv[1], newline="")))
at = next(i for i, r in enumerate(rows) if r[0] == "Panelboard, 200 A")
assert rows[at][4] == "15 in", rows[at]
rows[at][4] = "20 in"                    # a 42-circuit panel is wider
csv.writer(open(sys.argv[1], "w", newline=""), lineterminator="\n").writerows(rows)
PY
on_at "Item library" 4; sleep 3          # Bring in: the list as changed
typed "turned items.csv"; sleep 6
park
shot                                     # 04 changed, and the one set down is now that size
xdotool key F2; sleep 3                  # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen, or the plan was not"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  8 pictures saved to $OUT (model and sheet, after each of 4 steps)"

echo
echo "== What the program said as it went =="
LOG="$(cat "$T/map.txt.said")"
says "in hand" "Panelboard, 200 A in hand: click where one stands." "$LOG"
# Set down with its middle at 23.5 ft east (a pointer lands on a dot of the screen, so
# near it: the saved job says where). 15 in wide, it reaches 7 1/2 in each way: to
# 24.125 ft east, past the wall's near face at 23.75 ft by 0.375 ft, 4 1/2 in. That is
# the least the two would have to part by, so it is how far they are into each other.
grep -qE 'Set one down on Ground\. It is in the same space as [Ww]all 2 \([0-9 /]+ in into each other\)\.' <<<"$LOG" || { echo "FAIL: setting it down into the wall was not said"; echo "$LOG"; exit 1; }
SAID_IN="$(grep -oE 'Set one down on Ground\. It is in the same space as [Ww]all 2 \([0-9 /]+ in' <<<"$LOG" | head -1 | sed -E 's/.*\(//; s/ in$//')"
echo "    set down: in the same space as wall 2, $SAID_IN in into each other"
# A quarter turn: its 4 in is now east and west, 2 in each way of 23.5: to 23.667 ft,
# an inch short of the wall's face.
says "the turn" "Turned Panelboard, 200 A a quarter: its width runs north and south. It is clear of everything." "$LOG"
says "the library written out" "Wrote turned items.csv, beside the job: the job's items, 21 rows under their headings." "$LOG"
says "the change, and what stands" "Panelboard, 200 A: changed: its width; the one set down is now that size" "$LOG"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
says "nothing in the same space" "In the same space: 0" "$("$P" clashes "$J")"
says "the item, in the job" "~20 in wide, ~4 in deep, ~30 in high; 1 placed" "$("$P" items "$J" | grep -A1 "Panelboard, 200 A")"
python3 - "$J" "$SAID_IN" <<'PY'
import json, sys
from fractions import Fraction
d = json.load(open(sys.argv[1]))["data"]
p = next(e["attributes"]["pl::placed"] for e in d if "pl::placed" in e["attributes"] and e["attributes"]["pl::placed"].get("item"))
(east, north), size, angle = p["at"]["value"], p["size"]["value"], p["angle"]
# Where the click landed: within a dot or two of 23.5 ft east, 8 ft north.
assert abs(east - 23.5) < 0.03 and abs(north - 8.0) < 0.03, (east, north)
# What was said as it was set down, against where it stands: to the nearest 1/16 in.
deep = round((east + 0.625 - 23.75) * 12 * 16) / 16
said = float(sum(Fraction(part) for part in sys.argv[2].split()))
assert abs(said - deep) < 1e-9, (said, deep)
print(f"    it stands at {east:.3f} ft east, {north:.3f} ft north: {deep} in into the wall as set down, as was said")
# 20 in by 4 in by 30 in, in feet; turned 90 degrees, by hand, so Assumed.
assert all(abs(a - b) < 1e-9 for a, b in zip(size, [20 / 12, 4 / 12, 2.5])), size
assert angle["value"] == 90.0 and angle["status"] == "Assumed" and angle["source"] == "turned (turned by hand in the model)", angle
print("    in the file: 1.6667 by 0.3333 by 2.5 ft, turned 90 degrees, Assumed, turned by hand in the model")
PY
echo "  PASS: an item taken in hand on plan was set down on plan, said it was in the wall and by how much, was turned a quarter and cleared it, and took its new width from the library where it stood"
