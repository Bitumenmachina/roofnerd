#!/usr/bin/env bash
# Foundation work in the window: a ring of strip footings and the poured walls on them,
# made at the command line as a traced plan would make them; in the window, the starter
# assemblies brought in with one click, the ground taken away to see the foundation, a
# footing and a wall picked on the plan and each given its assembly from the list; the
# estimate read on the sheet; and the saved job's figures checked against those worked by
# hand in STATUS.md ("Foundation work"), the same as demos/foundation.sh gets.
# Run from the repository root:  demos/foundw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display, which
# needs Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job is made
# up. Every step fails the script if the program does not do what it says.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/foundw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/foundw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"; APP="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/foundw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up foundation plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }

echo "== The foundation, as a traced plan makes it: 40 ft by 28 ft, footings 20 in by 10 in, walls 8 in by 4 ft =="
"$P" new "$J" "Foundation in the window (not a real job)" >/dev/null
L="$(made "$("$P" level "$J" foundation --elevation "-4 ft" --height "4 ft" "${K[@]}")")"
FT="$(made "$("$P" type "$J" footing "footing 20 x 10" --width "20 in" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "foundation wall, 8 in" --thickness "8 in" "${K[@]}")")"
read -r F_S _ <<<"$("$P" footing "$J" "$L" "$FT" --points "0,0 40,0 40,28 0,28" --closed --depth "10 in" "${K[@]}" | grep -oE 'footing-[0-9]+' | tr '\n' ' ')"
read -r W_S _ <<<"$("$P" wall "$J" "$L" "$WT" --points "0,0 40,0 40,28 0,28" --closed --height "4 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" check "$J"
cp "$J" "$T/before.json"

echo
echo "== The session: Starter assemblies; the ground away; the south footing and the south wall each given its assembly; the estimate; save =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
shot                                    # 01 as opened: the foundation under the ground
on "Starter assemblies"; park
shot                                    # 02 the starter assemblies brought in
on "Show Ground"; park; sleep 4
shot                                    # 03 the ground away: the footings under the walls
on "Plan"; sleep 8
at 20 -0.6                              # the south footing, outside the wall's face and within the footing's
sleep 3
shot                                    # 04 the south footing picked: its figures
on "Assembly"; park
shot                                    # 05 the assemblies that go on a footing
on "Assembly Strip footing, formed, two #4 bars"; park
shot                                    # 06 the footing given its assembly: its lines and what they come to
at 20 0                                 # the south wall, on its line
sleep 3
on "Assembly"; park
on "Assembly Foundation wall, poured concrete"; park
shot                                    # 07 the wall given its assembly
sheet "Estimate"; sleep 4
shot                                    # 08 the sheet: the foundation by division
xdotool key F2; drawn_for 3 3                 # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 07 08; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  16 pictures saved to $OUT (model and sheet, after each of 8 steps)"

echo
echo "== What the session left in the saved job, against the figures worked by hand =="
"$P" check "$J"
"$P" show "$J" > "$T/show.txt"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1" | head -80; exit 1; }; printf '    %s\n' "$2"; }
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }
block "$F_S" > "$T/fs.txt"
block "$W_S" > "$T/ws.txt"
says "$T/fs.txt" "Strip footing, formed, two #4 bars"
says "$T/fs.txt" "quantity: Footing forms, both sides 66.6667 SF"
says "$T/fs.txt" "with waste: Footing concrete 2.1605 CY"
says "$T/fs.txt" "to order: Footing bars, #4, continuous 60.12 lb"
says "$T/ws.txt" "Foundation wall, poured concrete"
says "$T/ws.txt" "with waste: Wall concrete 4.1481 CY"
says "$T/ws.txt" "quantity: Anchor bolts, 1/2 in, with nut and washer 8 EA"
says "$T/ws.txt" "? quantity: Dampproofing, outside face unknown"
echo "  PASS: in the window, the starter assemblies came in with one click, the ground was taken away, a footing and a wall were picked on the plan and each given its assembly from the list, and the saved job has the figures worked by hand"
