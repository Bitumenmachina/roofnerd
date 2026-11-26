#!/usr/bin/env bash
# What a cost line is priced by, changed on the line in the window (DECISIONS.md call
# 125, from the twenty-seventh judgment: "I could not find how to change what a
# quantity is made of"). Run from the repository root:  demos/quantityw.sh
#
# It drives the real program with a real pointer and keyboard. The job is made up (no
# real job data). Every step fails the script if the program does not do what the step
# says. The figures, worked by hand: one wall 20 ft long and 9 ft high with no openings
# has a face of 180 SF and the same net. "Lap siding" priced by the net area at a
# made-up $4.50/SF is 180.00 SF, $810.00. Priced by "face area x 1.10" (the face and a
# tenth for waste) it is 198.00 SF, $891.00.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/quantityw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/quantityw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/quantityw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/priced.json"
K=(--status Known --source "made-up plan")
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -40; exit 1; }; printf '    %s\n' "$2"; }

echo "== The job: one wall 20 ft by 9 ft, lap siding on it by its net area at \$4.50/SF =="
"$P" new "$J" "Priced by (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
W="$(made "$("$P" wall "$J" "$S" "$WT" --points "0,0 20,0" --height "9 ft" "${K[@]}")")"
"$P" line "$J" "$W" "Lap siding" net_area --cost-type material --division 7 >/dev/null
"$P" set "$J" "$W" unit_cost 4.50 --line "Lap siding" --status Assumed --source "made-up price" >/dev/null
"$P" check "$J"
BEFORE="$("$P" estimate "$J")"
says "the line as it starts" "180.00 SF" "$(grep "Lap siding" <<<"$BEFORE")"
says "the job cost as it starts" "\$810.00" "$(grep "Job cost" <<<"$BEFORE")"
cp "$J" "$T/before.json"

echo
echo "== The session: the line's quantity clicked on the sheet, typed over, then typed wrong =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
sleep 8; park
sheet "Estimate"; sleep 5
sheet_at "Quantity of Lap siding on wall 1"; sleep 5
park
shot                                     # 01 the field open, holding what the line is priced by now: net area
typed "face area x 1.10"; sleep 6
park
shot                                     # 02 priced by the face and a tenth: 198.00 SF, \$891.00
sheet_at "Quantity of Lap siding on wall 1"; sleep 4
typed "roof area"; sleep 6
park
shot                                     # 03 refused: a wall has no such figure, and the figures it has are named
esc
xdotool key F2; drawn_for 3 3                  # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  6 pictures saved to $OUT (model and sheet, after each of 3 steps)"

echo
echo "== What the program said as it went =="
LOG="$(cat "$T/map.txt.said")"
says "set" "Wall 1: quantity of \"Lap siding\" set to face area x 1.10." "$LOG"
says "refused, with the figures a wall has" "Can't price it by that: wall 1 cost line \"Lap siding\": its quantity \"roof area\" uses \"roof area\", which is not a figure of this thing (a wall has plan length, height, face area, opening area, net area, thickness, studs, plate length, header length, sill length, net length)" "$LOG"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
AFTER="$("$P" estimate "$J")"
says "the line: 180 x 1.10 = 198.00 SF at \$4.50" "198.00 SF" "$(grep "Lap siding" <<<"$AFTER")"
says "its amount" "\$891.00" "$(grep "Lap siding" <<<"$AFTER")"
says "the job cost" "\$891.00" "$(grep "Job cost" <<<"$AFTER")"
echo "  PASS: a line's quantity was changed on the line in the window to the figure worked by hand, and a figure a wall does not have was refused with the ones it has"
