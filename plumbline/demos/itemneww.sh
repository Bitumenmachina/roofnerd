#!/usr/bin/env bash
# The job's item library kept by hand in the window (DECISIONS.md call 111): a new item
# typed in, given its trade, set down, renamed with what stands, refused deletion while
# one stands, and an item nothing is set down from taken out.
# Run from the repository root:  demos/itemneww.sh
#
# It drives the real program with a real pointer and keyboard. The job is made up (no
# real job data). Every step fails the script if the program does not do what the step
# says. STATUS.md ("An item added, renamed and taken out by hand") has what each step
# is to say.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/itemneww.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/itemneww.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/itemnew}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -60; exit 1; }; printf '    %s\n' "$2"; }

echo "== The job: a level, and no item in it =="
"$P" new "$J" "Items by hand (not a real job)" >/dev/null
"$P" level "$J" ground --elevation "0 ft" --height "9 ft" --status Known --source "made-up plan" >/dev/null
"$P" check "$J"
cp "$J" "$T/before.json"

echo
echo "== The session: a new item typed in; its trade; one set down; deleting refused; renamed; another added and taken out =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
field() { xdotool type --delay 250 -- "\$1"; sleep 1; xdotool key "\${2:-Tab}"; sleep 2; }
open_window "$J"
sleep 8; park
on_at "New item"; sleep 3
field "Wall oven, 27 in"; field "equipment"; field "27 in"; field "24 in"
shot                                     # 01 the form, four fields in: a comma in its name is a comma
field "29 in" Return; sleep 4
park
shot                                     # 02 added, and open on the right: sizes Assumed, no trade said
on_right "Trade 11"; sleep 5                # Division 11 Equipment
on_at "Item Wall oven, 27 in" 4; sleep 4 # in hand
click 700 500; sleep 6                   # set down on the floor
esc; park
on_at "Change item Wall oven, 27 in" 4; sleep 5
on_right "Delete item" 4; sleep 5        # refused: one is set down from it
park
shot                                     # 03 not deleted from under what stands, and why
on_right "Rename item" 4; sleep 3
xdotool type --delay 250 -- "Wall oven, single"; sleep 1; xdotool key Return; sleep 5
park
shot                                     # 04 renamed, and the one set down with it
on_at "New item" 4; sleep 3
field "Range hood"; field "fixture"; field "30 in"; field "20 in"; field "9 in" Return; sleep 4
on_right "Delete item" 4; sleep 5        # nothing is set down from it: it goes
park
shot                                     # 05 taken out, said, and no longer in the list
xdotool key F2; sleep 3                  # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
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
says "added" "Added the item \"Wall oven, 27 in\": its sizes are Assumed, as typed, until they are checked against its maker's sheet." "$LOG"
says "its trade" "Wall oven, 27 in is an item of Division 11 Equipment" "$LOG"
says "one set down" "Set one down on Ground." "$LOG"
says "not deleted from under it" "Can't delete the item: one is set down from Wall oven, 27 in: take it out of the job first" "$LOG"
says "renamed" "The item is now called \"Wall oven, single\". The one set down is called that too." "$LOG"
says "another added" "Added the item \"Range hood\"" "$LOG"
says "and taken out" "Took the item \"Range hood\" out of the job. Undo brings it back." "$LOG"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
LIST="$("$P" items "$J")"
says "the oven, under its trade" "11 Equipment" "$LIST"
says "the oven, as typed" "Wall oven, single" "$LIST"
# Nothing says what it is drawn as, and "wall oven" names none of the shapes the program
# draws, so it is drawn as its box and the list says why.
says "its sizes and what stands" "equipment, drawn as a box, its name saying nothing else; ~27 in wide, ~24 in deep, ~29 in high; 1 placed" "$LIST"
if grep -qF "Range hood" <<<"$LIST"; then echo "FAIL: the range hood is still in the job"; echo "$LIST"; exit 1; fi
echo "    no Range hood in the job"
python3 - "$J" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))["data"]
item = next(e for e in d if "pl::item" in e["attributes"])
it = item["attributes"]["pl::item"]
for what in ("width", "depth", "height"):
    v = it[what]
    assert v["status"] == "Assumed" and v["source"] == "typed in the window; not yet checked against its maker's sheet", v
assert it["maker"]["status"] == "Unknown" and it["model"]["status"] == "Unknown", it
placed = next(e["attributes"]["pl::placed"] for e in d if "pl::placed" in e["attributes"])
assert placed["what"]["value"] == "Wall oven, single", placed["what"]
print("    in the file: one item, its three sizes Assumed and typed in the window, maker and model Unknown; the one set down called what its item is")
PY
echo "  PASS: an item was typed into the job's library in the window with a comma in its name, given its trade, set down, kept from deletion while one stood, renamed with what stood, and a second was added and taken out"
