#!/usr/bin/env bash
# A shingle roof priced whole, in the window: a plane of a hip roof clicked on the model,
# its whole roof picked from the plane's panel, "Asphalt shingle roof, whole roof" picked
# from the list of assemblies that go on a roof, and the estimate read; the saved job's
# figures checked against those worked by hand in STATUS.md ("A shingle roof priced
# whole"), the same demos/shingle.sh gets at the command line.
# Run from the repository root:  demos/shinglew.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display, which
# needs Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job is made
# up. Every step fails the script if the program does not do what it says.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/shinglew.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/shinglew.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"; APP="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/shinglew}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"

echo "== The job: a hip roof over 42 ft by 30 ft at 6:12; nothing priced =="
"$P" new "$J" "A shingle roof priced whole (not a real job)" >/dev/null
"$P" level "$J" ground --elevation "0 ft" --height "9 ft" --status Known --source "made-up plan" >/dev/null
"$P" roof "$J" "Main roof" --points "-1,-1 41,-1 41,29 -1,29" --pitch 6:12 --eave-height "9 ft" --status Known --source "made-up roof plan" >/dev/null
R="$("$P" show "$J" | grep -oE 'roof-[0-9]+' | head -1)"
"$P" check "$J"
cp "$J" "$T/before.json"

echo
echo "== The session: starter assemblies; a plane clicked; its whole roof; the whole-roof assembly; the estimate =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
sleep 8; park
on "Starter assemblies"; sleep 5
shot                                    # 01 the roof as opened, the starter assemblies in
click 760 470; sleep 4                  # a plane of the roof, on the model
on "Whole roof"; sleep 4; park
shot                                    # 02 the whole roof picked: its own figures
on "Assembly"; sleep 4; park
shot                                    # 03 the assemblies that go on a roof
on "Assembly Asphalt shingle roof, whole roof"; sleep 6; park
shot                                    # 04 the roof given its assembly: what it comes to
sheet "Estimate"; sleep 4
shot                                    # 05 the sheet: the roof in sheets, rolls, bundles and pieces
xdotool key F2; drawn_for 3 3                 # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  10 pictures saved to $OUT (model and sheet, after each of 5 steps)"

echo
echo "== What the session left in the saved job, against the figures worked by hand =="
"$P" check "$J"
"$P" show "$J" > "$T/show.txt"
sed -n "/^ *$R  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d' > "$T/roof.txt"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; head -80 "$1"; exit 1; }; printf '    %s\n' "$2"; }
says "$T/roof.txt" "Asphalt shingle roof, whole roof"
says "$T/roof.txt" "to order: Roof sheathing 49 sheet"
says "$T/roof.txt" "to order: Underlayment, synthetic 2 roll"
says "$T/roof.txt" "to order: Ice barrier, eaves and valleys 3 roll"
says "$T/roof.txt" "to order: Field shingles 49 bundle"
says "$T/roof.txt" "to order: Starter strip 2 bundle"
says "$T/roof.txt" "to order: Hip and ridge cap 6 bundle"
says "$T/roof.txt" "to order: Drip edge 15 piece"
says "$T/roof.txt" "? quantity: Pipe boots unknown"
echo
echo "== The roof is dressed as it is priced (call 141): bare before, shingled after, measured =="
# The same patch of the south plane, before the assembly (picture 03) and after (04).
# As measured on a run of these steps: bare (138, 149, 157), blue over red; shingled
# (104, 106, 102), darker and with no blue to it. A roof drawn bare after it is priced
# fails the second test (call 141 found it so).
python3 -I - "$OUT" <<'PY'
import sys
from PIL import Image
out = sys.argv[1]
def mean(name):
    im = Image.open(f"{out}/{name}-model.png").convert("RGB").crop((760, 470, 960, 540))
    px = list(im.getdata())
    return tuple(round(sum(p[i] for p in px) / len(px)) for i in range(3))
bare, shingled = mean("03"), mean("04")
ok = bare[2] - bare[0] > 10
print("    before: %s  %s" % (bare, "as wanted (bare, blue over red)" if ok else "FAIL: wanted the bare roof's blue-gray"))
good = shingled[2] - shingled[0] < 5 and sum(shingled) / 3 < 125
print("    after:  %s  %s" % (shingled, "as wanted (shingles: darker, no blue to it)" if good else "FAIL: wanted asphalt shingles"))
sys.exit(0 if ok and good else 1)
PY
echo "  PASS: in the window, a plane was clicked, its whole roof picked, and the whole-roof assembly given from the list; the saved job has the roof in whole packs to the figures worked by hand, and the roof is drawn in shingles once it is priced in them"
