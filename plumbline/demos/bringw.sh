#!/usr/bin/env bash
# What people already have, brought into a job from the window: a model from another
# trade (IFC), a supplier's price list (XLSX) and an estimate made before (CSV), each by
# its button under "Bring in". The job is new: its file is not there when the program
# starts, and is written when it is saved.
# Then the saved job is set against the same three brought in at the command line.
# Run from the repository root:  demos/bringw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The model, the price
# list and the estimate are made up (demos/ifc, demos/tables). Every step fails the
# script if the program does not do what it says. The figures are those of
# demos/imports.sh, where they are worked by hand.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/bringw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/bringw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/bring}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/bring.json"
says() { grep -qF -- "$2" "$3" || { echo "FAIL: $1: not said: $2"; cat "$3"; exit 1; }; printf '    %s\n' "$2"; }
# The files lie beside the job, as a person keeps them, and are named from there.
cp demos/ifc/made-up-house.ifc demos/tables/tinners-yard.xlsx demos/tables/prior-estimate.csv "$T/"
touch "$T/started"

echo "== The session: a new job; a model, a price list and an estimate made before brought in; save =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
shot                                    # 01 a new job: nothing in it, and Next steps says where to start
on "Model (IFC)"; sleep 2               # Bring in: a model
typed "made-up-house.ifc" yes           # 02 its file's name, typed where it is asked for
sleep 6
park
shot                                    # 03 the model's walls, floor and runs; what was not brought in is said over the model
on "Price list"; sleep 2                # Bring in: a supplier's price list
typed "tinners-yard.xlsx, Tinners Yard"
sleep 4
shot                                    # 04 the prices set, and the two rows not used, each with why
on "Prior estimate"; sleep 2            # Bring in: an estimate made before
typed "prior-estimate.csv"
sleep 4
shot                                    # 05 the estimate, kept beside the job
on "Save"; sleep 3                      # the job's file is written now
shot                                    # 06 saved
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06; do
  [ "$OUT/$n-model.png" -nt "$T/started" ] && [ "$OUT/$n-sheet.png" -nt "$T/started" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  12 pictures saved to $OUT (model and sheet, after each of 6 steps)"

echo
echo "== What the session left in the saved job =="
[ -s "$J" ] || { echo "FAIL: the new job was not written when it was saved"; exit 1; }
"$P" check "$J"
"$P" takeoff "$J" > "$T/takeoff.txt"; "$P" show "$J" > "$T/show.txt"; "$P" compare "$J" > "$T/compare.txt"
# The made-up model: four walls round 30 ft by 20 ft, 100 LF; a floor of 600 SF; a room.
says "the model's walls" "walls 4            100.00 LF" "$T/takeoff.txt"
says "its floor" "floors 1           600.00 SF" "$T/takeoff.txt"
# The price list: three materials added, each with Tinners Yard's price for it.
says "a material added" "Copper, 16 oz" "$T/show.txt"
says "whose price" "Tinners Yard" "$T/show.txt"
says "the estimate made before" "prior-estimate" "$T/compare.txt"
# Everything brought in in the window is Assumed, and says the file it came from.
says "where it came from" "made-up-house.ifc, brought in in the window" "$T/show.txt"

echo
echo "== The same three brought in at the command line give the same job, byte for byte =="
C="$T/twin.json"
"$P" new "$C" "bring" >/dev/null
"$P" import ifc "$C" "$T/made-up-house.ifc" --status Assumed --source "made-up-house.ifc, brought in in the window" >/dev/null
"$P" import prices "$C" "$T/tinners-yard.xlsx" --supplier "Tinners Yard" --add --status Assumed --source "tinners-yard.xlsx, brought in in the window" >/dev/null
"$P" import prior "$C" "$T/prior-estimate.csv" --name "prior-estimate" --status Assumed --source "prior-estimate.csv, brought in in the window" >/dev/null
cmp "$J" "$C" || { echo "FAIL: the job saved from the window is not the job made at the command line"; diff <(python3 -m json.tool "$J") <(python3 -m json.tool "$C") | head -40; exit 1; }
echo "  PASS: a model, a price list and an estimate made before were brought into a new job in the window, and the saved job is the command line's, byte for byte"
