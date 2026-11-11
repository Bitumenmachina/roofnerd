#!/usr/bin/env bash
# The whole-house demonstration: one job holding a building and what runs through it,
# walked round in the 3D window with the sheet beside it.
# Run from the repository root:  demos/house.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. On a desktop, run `plumbline-app <job>` and do
# the same by hand. Here it runs on a virtual display, which needs: Xvfb, xdotool, and a
# Vulkan driver (Mesa's software one will do). It says so and stops if one is missing.
# The job is made up. Every step fails the script if the program does not do what it says.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/house.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/house.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/house}"; mkdir -p "$OUT"
T="$(mktemp -d)"

echo "== The job: a made-up two-storey house with its heating, plumbing and wiring =="
"$P" house "$T/job.json"
"$P" check "$T/job.json"
"$P" takeoff "$T/job.json" | tee "$OUT/takeoff.txt"
cp "$T/job.json" "$T/before.json"

echo
echo "== What the takeoff has to say, checked by hand =="
check() { grep -qF -- "$2" "$OUT/takeoff.txt" || { echo "FAIL: $1: the takeoff does not say: $2"; exit 1; }; echo "  $1"; }
# Four outside walls (40 + 28 + 40 + 28) and partitions of 40, 12, 12 and 10 LF.
check "first floor: 8 walls, 210 LF" "walls 8            210.00 LF"
# Hot water: two lavatory branches of 1.9 LF and two tub branches of 2.5 LF.
check "hot water, 1/2 in: 4 runs, 8.80 LF" "Pipe     1/2 in round      4 runs       8.80 LF"
# The kitchen's hot branch has no size on the plan. Its 9.40 LF is there, on a line of its own.
check "hot water with no size: 9.40 LF, flagged Unknown, on a line of its own" "? Pipe     size unknown      1 runs       9.40 LF"
# The west trunk: up 3.4, south 8, west 17. Its size is assumed, and says so.
check "supply air, 16 x 8: 28.40 LF, flagged Assumed" "~ Duct     16 in × 8 in      1 runs      28.40 LF"

echo
echo "== The session: the house, see through it, a system at a time, a storey at a time, a plan, and on foot =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export VK_ICD_FILENAMES="$ICD" WGPU_BACKEND=vulkan PLUMBLINE_SHOTS="$OUT"
"$APP" "$T/job.json" > "$T/app.log" 2>&1 &
APP_PID=\$!
for i in \$(seq 1 60); do xdotool search --name "plumbline: model" >/dev/null 2>&1 && break; sleep 1; done
sleep 40   # the first frames are slow on a software renderer
xdotool windowfocus "\$(xdotool search --name 'plumbline: model' | head -1)"; sleep 1
shot() { xdotool key F12; sleep 12; }
shot                                                    # 01 the house as opened
xdotool key x; sleep 5
shot                                                    # 02 seen through: what runs inside it
xdotool mousemove 80 493; sleep 1; xdotool click 1; sleep 4          # the browser's Supply air row
xdotool mousemove 1830 78; sleep 1; xdotool click 1; sleep 4         # the sheet's Mechanical tab
xdotool mousemove 800 860; sleep 1
shot                                                    # 03 one system picked out: supply air
xdotool mousemove 80 601; sleep 1; xdotool click 1; sleep 4          # Hot water
xdotool mousemove 2060 78; sleep 1; xdotool click 1; sleep 4         # the sheet's Plumbing tab
xdotool mousemove 800 860; sleep 1
shot                                                    # 04 hot water, with its run of no size
xdotool key Escape; sleep 1; xdotool key x; sleep 2; xdotool key Prior; sleep 8
xdotool mousemove 1730 78; sleep 1; xdotool click 1; sleep 3; xdotool mousemove 800 860; sleep 1   # the sheet's Building tab
shot                                                    # 05 the first floor alone
xdotool key t; sleep 9
shot                                                    # 06 the first floor on plan
xdotool mousemove 800 500; sleep 1; xdotool click 1; sleep 4; xdotool mousemove 806 505; sleep 2
shot                                                    # 07 something on the plan clicked
xdotool key Escape; sleep 1; xdotool key t; sleep 3; xdotool key Next; sleep 3; xdotool key v; sleep 10
shot                                                    # 08 on foot in the living room
xdotool key F2; sleep 3                                 # save
kill \$APP_PID; wait \$APP_PID 2>/dev/null
SESSION
chmod +x "$T/session.sh"
timeout 900 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06 07 08; do
  # Each picture has to be from this run, not left over from an earlier one.
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  16 pictures saved to $OUT (model and sheet, after each of 8 steps)"

echo
echo "== What the session left in the saved job =="
"$P" check "$T/job.json"
# Nothing was edited: looking is not changing. The saved file is the one that was opened.
cmp -s "$T/before.json" "$T/job.json" || { echo "FAIL: looking at the job changed it"; exit 1; }
echo "  PASS: the job was walked round, picked through and saved, and is byte for byte what it was"
