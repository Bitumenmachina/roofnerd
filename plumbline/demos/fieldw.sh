#!/usr/bin/env bash
# Field entry in the application. The job is the one demos/field.sh starts from, as a
# tracing leaves it: two clashes that are questions, and a wall whose price is flagged.
# Then, in the window, what a veteran measures on site is put on the job: the storey is
# picked on the left and its floor and its height on the right; the duct is picked on the
# plan and its place, its type and its height on the right. Each reading is typed in one
# burst, the way a laser in keyboard mode types it, and ended by the Tab or the Enter the
# laser sends. The sheet's price changes when the storey height is measured; the Clashes
# tab goes from two questions to one finding when the duct's underside is shot.
# Then the saved job is set against the same measurements made at the command line.
# Run from the repository root:  demos/fieldw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job is made up.
# Every step fails the script if the program does not do what it says. The figures are
# worked by hand in STATUS.md ("Field entry") and in demos/field.sh.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/fieldw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/fieldw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/field}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" "$3" || { echo "FAIL: $1: not said: $2"; cat "$3"; exit 1; }; printf '    %s\n' "$2"; }

echo "== The job, as a tracing leaves it =="
eval "$(demos/field-job.sh "$J")"
"$P" clashes "$J" > "$T/before.txt"; "$P" estimate "$J" | tr -s ' ' > "$T/sheet-before.txt"
says "two questions" "In the same space: 2 (2 to verify in the field)" "$T/before.txt"
says "the price, flagged" "PRICE ~\$615.86" "$T/sheet-before.txt"
cp "$J" "$T/before.json"

echo
echo "== The session: the floor, the storey height, the duct's place, its type and its underside =="
# The model window is 1600 by 960 with the sheet to its right. On the right-hand panel a
# value's row is clicked anywhere; the rows below one that has been marked sit a line
# lower, because its source has grown a note.
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export VK_ICD_FILENAMES="$ICD" WGPU_BACKEND=vulkan PLUMBLINE_SHOTS="$OUT"
"$APP" "$J" > "$T/app.log" 2>&1 &
APP_PID=\$!
for i in \$(seq 1 60); do xdotool search --name "plumbline: model" >/dev/null 2>&1 && break; sleep 1; done
sleep 40   # the first frames are slow on a software renderer
xdotool windowfocus "\$(xdotool search --name 'plumbline: model' | head -1)"; sleep 1
shot() { xdotool mousemove 1000 850; sleep 1; xdotool key F12; sleep 12; }
# A click held for longer than a frame takes to draw.
click() { xdotool mousemove \$1 \$2; sleep 1.5; xdotool mousedown 1; sleep 0.7; xdotool mouseup 1; sleep 3; }
# A laser in keyboard mode: the reading typed in one burst, then the key it is set to send.
laser() { xdotool type --delay 20 "\$1"; xdotool key "\$2"; sleep 4; }
click 53 189                            # the storey, Ground: on show, and picked
xdotool key t; sleep 6                  # the plan
click 2044 73                           # the sheet's Estimate tab
shot                                    # 01 before: the storey's values Assumed, the price flagged
click 1400 188                          # its floor (Elevation): Measure
shot                                    # 02 what is asked for
laser "same" Return                     # it is the floor everything is measured from
click 1400 240                          # Storey height: Measure
laser "8'10\"1/2" Tab                   # floor to floor, as the laser types it, ended by Tab
shot                                    # 03 the storey height Known, the floor above moved, the price changed
click 2130 73                           # the sheet's Clashes tab
click 600 480                           # the duct, on the plan
shot                                    # 04 the duct's values, and two clashes to verify
click 1400 242                          # Type: Check
laser "same" Return
click 1400 300                          # Line on the plan: Check
laser "same, JM" Return                 # and who looked
click 1400 362                          # Height above its storey: Measure
xdotool type --delay 120 "bottom "      # the person says where the shot is to
laser "7'10\"3/4" Return                # the laser, to the duct's underside
shot                                    # 05 one clash, a finding
xdotool key F2; sleep 3                 # save
kill \$APP_PID; wait \$APP_PID 2>/dev/null
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  10 pictures saved to $OUT (model and sheet, after each of 5 steps)"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
"$P" clashes "$J" --clearance "2 in" > "$T/after.txt"; "$P" estimate "$J" | tr -s ' ' > "$T/sheet-after.txt"; "$P" show "$J" > "$T/show.txt"
# 24 ft by 8 ft 10 1/2 in: 213 SF, $511.20; with overhead and profit, $607.31. No flag.
says "the wall" "Gypsum board, hung and finished 213.00 SF \$2.40/SF \$511.20" "$T/sheet-after.txt"
says "the price" "PRICE \$607.31" "$T/sheet-after.txt"
# Pipe A is 3/4 in clear of the duct's underside; pipe B is in the duct by 2 1/4 in, and
# everything that rests on is Known.
says "one finding" "In the same space: 1 (1 finding)" "$T/after.txt"
says "which" "in each other by 2.25 in" "$T/after.txt"
says "the other, clear" "0.75 in apart" "$T/after.txt"
says "the storey height" "storey height    8' 10 1/2\"" "$T/show.txt"
says "the duct's underside" "height above its storey 7' 10 3/4\"" "$T/show.txt"
says "held to its bottom" "its heights are those of its bottom" "$T/show.txt"
python3 - "$J" "$GROUND" "$UPPER" "$DUCT" <<'PY'
import sys, json
data = json.load(open(sys.argv[1]))["data"]
ground, upper, duct = (next(e for e in data if e["path"] == p)["attributes"] for p in sys.argv[2:5])
want = [
    ("the floor", ground["pl::level"]["elevation"], "Known", "scaled off a made-up plan; found as drawn (checked on site)"),
    ("the storey height", ground["pl::level"]["height"], "Known", "measured on site"),
    ("the floor above", upper["pl::level"]["elevation"], "Known", "ground's floor and its storey height (measured on site)"),
    ("the duct's type", duct["pl::segment"]["type"], "Known", "scaled off a made-up plan; found as drawn (checked on site)"),
    ("the duct's line", duct["pl::segment"]["line"], "Known", "scaled off a made-up plan; found as drawn (checked on site: JM)"),
    ("the duct's height", duct["pl::segment"]["offset"], "Known", "measured on site"),
]
for what, value, status, source in want:
    if (value["status"], value["source"]) != (status, source):
        print(f"FAIL: {what} is {value['status']}, {value['source']!r}; wanted {status}, {source!r}"); sys.exit(1)
if upper["pl::level"]["height"]["status"] != "Assumed":
    print("FAIL: the upper storey's own height was not measured and is no longer Assumed"); sys.exit(1)
print("    six values Known, each saying it was measured or checked on site; the upper storey's own height still Assumed")
PY

echo
echo "== The same measurements at the command line give the same figures =="
C="$T/twin.json"; cp "$T/before.json" "$C"
"$P" measure "$C" "$GROUND" elevation same --source "checked on site" >/dev/null
"$P" measure "$C" "$GROUND" "storey height" "8'10\"1/2" --source "measured on site" >/dev/null
"$P" measure "$C" "$DUCT" type same --source "checked on site" >/dev/null
"$P" measure "$C" "$DUCT" "line on the plan" same --source "checked on site: JM" >/dev/null
"$P" measure "$C" "$DUCT" "height above its storey" "7'10\"3/4" --to bottom --source "measured on site" >/dev/null
cmp "$C" "$J" && echo "    the job saved by the window is byte for byte the job the command line makes"
echo "  PASS: values were measured and checked in the window as a laser types them, the price and the clashes followed, and the saved job says the same"
