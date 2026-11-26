#!/usr/bin/env bash
# The item library in the window (DECISIONS.md call 105): the starter items are brought
# in and listed trade by trade; one is set down; an item is opened with Change and what
# it is drawn as is changed with a click, and the one standing on the floor follows;
# the library is written out beside the job, and is read back at the command line.
# Run from the repository root:  demos/itemlibw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard. The job and every size
# are made up (no real job data). Every step fails the script if the program does not
# do what the step says.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/itemlibw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/itemlibw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/itemlib}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
mkdir "$T/job"; J="$T/job/bench.json"
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -60; exit 1; }; printf '    %s\n' "$2"; }

echo "== The job: a level, and nothing else =="
"$P" new "$J" "Items in the window (not a real job)" >/dev/null
"$P" level "$J" ground --elevation "0 ft" --height "9 ft" --status Known --source "made-up plan" >/dev/null
"$P" check "$J"
cp "$J" "$T/before.json"

echo
echo "== The session: starter items; a vanity set down; opened, and drawn as its box; the library written out; save =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
sleep 8; park
on "Starter tool chest"; sleep 5
shot                                     # 01 the items, trade by trade, each with Change
on "Item Vanity, 36 in"; sleep 4         # in hand
click 700 500; sleep 5                   # set down on the floor
esc; park
shot                                     # 02 a vanity on the floor: a cabinet under a worktop
on "Change item Vanity, 36 in"; sleep 5  # the item itself, opened on the right
park
shot                                     # 03 its sizes, its trade, what it is drawn as
on "Drawn as Box"; sleep 5
park
shot                                     # 04 drawn as its box, and so is the one on the floor
on "Tool chest, to keep"; sleep 6         # written out beside the job
park
shot                                     # 05 said, with where it is and what to do with it
xdotool key F2; drawn_for 3 3                  # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  10 pictures saved to $OUT (model and sheet, after each of 5 steps)"

echo
echo "== What the program said as it went =="
LOG="$(cat "$T/map.txt.said")"
says "the starter" "Brought in 48 starter items, trade by trade, at typical sizes: each size is Assumed until the product is chosen." "$LOG"
says "one set down" "Set one down on Ground." "$LOG"
says "what it is drawn as" "Vanity, 36 in is drawn as a box, wherever one stands." "$LOG"
says "the library written out" "Wrote “bench items”, beside the job: the job's tool chest, 48 items under their headings." "$LOG"

echo
echo "== What the session left: the saved job, and the library beside it =="
[ -f "$T/job/bench items.csv" ] || { echo "FAIL: the item library was not written beside the job"; ls "$T/job"; exit 1; }
"$P" check "$J"
LIST="$("$P" items "$J")"
says "the vanity, in the job" "furnishing, drawn as a box; ~36 in wide, ~21 in deep, ~34 1/2 in high; 1 placed" "$LIST"
# The library as written, into a job with nothing in it: the same 48 items, the vanity
# drawn as its box there too.
"$P" new "$T/other.json" "Another job (not a real job)" >/dev/null
IN="$("$P" import items "$T/other.json" "$T/job/bench items.csv")"
says "read back" "  set: 48" "$IN"
says "all of it" "  not used: 0" "$IN"
says "the change went with it" "furnishing, drawn as a box; ~36 in wide, ~21 in deep, ~34 1/2 in high; 0 placed" "$("$P" items "$T/other.json")"
echo "  PASS: the starter items came into the job in the window trade by trade, one was set down, an item was opened and what it is drawn as changed with a click, the library was written out beside the job, and another job read the same items from it"
