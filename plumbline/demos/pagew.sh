#!/usr/bin/env bash
# A plan and its page picked from a list (DECISIONS.md calls 94 and 98): in a new job,
# Add drawing lists what is where the job is kept; USDA Plan No. 7213 (public domain; a
# scan of four pages) is clicked, its pages are listed, and page 4 is clicked. Nothing
# is typed but the level. The words of the list are a builder's: a name without its
# ending, what it is in a word, no path.
# Run from the repository root:  demos/pagew.sh [folder for the pictures]
# The sheet is fetched once by tools/fetch-plans.sh into plans/, which git ignores.
#
# It drives the real program with a real pointer and keyboard. Every step fails the
# script if the program does not do what the step says.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/pagew.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/pagew.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
REAL="$PWD/plans/usda-7213-two-bedroom-farmhouse.pdf"
[ -f "$REAL" ] || { echo "demos/pagew.sh adds USDA Plan No. 7213, which is not here: run tools/fetch-plans.sh first"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/pagew}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job/farmhouse.json"
# Where the job is kept: the plan beside it, and a folder with a price list in it,
# so that the list has one of each kind of row.
mkdir -p "$T/job/price lists"
cp "$REAL" "$T/job/usda-7213.pdf"
printf 'item,unit,price\nmade-up item,EA,1.00\n' > "$T/job/price lists/made-up.csv"
touch "$T/started"
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -60; exit 1; }; printf '    %s\n' "$2"; }

echo "== The session: a new job; a level; Add drawing; the plan clicked; page 4 clicked; save =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
sleep 8; park
on "Add level"; sleep 2; typed "Ground, 0 ft, 8 ft"
on "Add drawing"; sleep 4
shot                                    # 01 the list: the plan by its name, a folder, the way back out
on "File usda-7213.pdf"; sleep 5        # the plan, clicked
shot                                    # 02 its four pages, at the head of the list
on "Page 4"; sleep 30                   # the page, clicked; a scanned sheet takes a while to read
park
shot                                    # 03 page 4 laid under the level
xdotool key F2; drawn_for 3 3                 # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 900 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03; do
  [ "$OUT/$n-model.png" -nt "$T/started" ] && [ "$OUT/$n-sheet.png" -nt "$T/started" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  6 pictures saved to $OUT (model and sheet, after each of 3 steps)"

echo
echo "== What the program said as it went =="
LOG="$(cat "$T/map.txt.said")"
says "the plan picked" "Picked usda-7213: it has 4 pages. Pick the page to trace over from the list." "$LOG"
says "the page laid under" "The drawing usda-7213, page 4" "$LOG"
# Nothing said has the file's ending or where it lies in it.
if grep -qE "usda-7213\.pdf|$T" <<<"$LOG"; then echo "FAIL: a message says the file's ending or where it lies"; echo "$LOG"; exit 1; fi
echo "    no message has the file's ending or where it lies"

echo
echo "== What the session left in the saved job =="
[ -f "$J" ] || { echo "FAIL: the job was not saved"; exit 1; }
"$P" check "$J"
SHOWN="$("$P" show "$J")"
says "the drawing, by its page" "usda-7213, page 4" "$SHOWN"
python3 - "$J" <<'PY'
import sys, json
d = json.load(open(sys.argv[1]))["data"]
found = [a for e in d for k, a in e.get("attributes", {}).items() if k == "pl::drawing"]
lies = found[0]["file"]["value"] if found else ""
ok = len(found) == 1 and found[0].get("page") == 4 and lies == "usda-7213.pdf"
print("    in the file: one drawing, page 4 of usda-7213.pdf, named as it lies beside the job" if ok else "    in the file: not one drawing of page 4 beside the job: %s" % found)
sys.exit(0 if ok else 1)
PY
echo "  PASS: a plan and its page were picked from the list with two clicks and nothing typed, the window said what was picked without the file's ending or its place, and the saved job has page 4 under the level"
