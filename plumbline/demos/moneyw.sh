#!/usr/bin/env bash
# What the window says when money is at stake (DECISIONS.md calls 90 and 91): a roof
# drawn from its footprint is on the verify-in-field list with its dollars and is
# measured off it; a labor line is added to a wall; the wall is deleted and what was
# priced on it is said; a markup typed ten times too large is answered.
# Run from the repository root:  xvfb-run -a -s "-screen 0 2500x980x24" demos/moneyw.sh
#
# It drives the real program with a real pointer and keyboard. The job and every price
# are made up (no real job data). Every step fails the script if the program does not
# do what the step says. The figures are worked by hand in the comments and in
# STATUS.md ("The money path in the window").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/moneyw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/moneyw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/money}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
A=(--status Assumed --source "scaled off a made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -60; exit 1; }; printf '    %s\n' "$2"; }

echo "== The job: 24 ft by 16 ft, 9 ft walls, a hip roof scaled off a plan; board on the south wall, underlayment on the south plane =="
"$P" new "$J" "The money path (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH _ <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" roof "$J" "Main roof" --points "0,0 24,0 24,16 0,16" --pitch 6:12 --eave-height "9 ft" "${A[@]}" >/dev/null
ALL="$("$P" show "$J")"
ROOF="$(awk '$2=="Main" && $3=="roof"{print $1; exit}' <<<"$ALL")"
PLANE="$(awk -v r="$ROOF" '$1==r{on=1} on&&$2=="plane"&&$3=="1"{print $1; exit}' <<<"$ALL")"
ROOT="$(grep -oE 'job-[0-9]+' <<<"$ALL" | head -1)"
"$P" line "$J" "$SOUTH" "Gypsum board, hung and finished" net_area --cost-type material --division 9 >/dev/null
"$P" set "$J" "$SOUTH" unit_cost '$2.00/SF' --line "Gypsum board, hung and finished" --status Known --source "made-up price" >/dev/null
"$P" line "$J" "$PLANE" "Underlayment" surface_area --cost-type material --division 7 >/dev/null
"$P" set "$J" "$PLANE" unit_cost '$0.50/SF' --line "Underlayment" --status Known --source "made-up price" >/dev/null
"$P" set "$J" "$ROOT" overhead 20 --status Known --source "made-up markup" >/dev/null
"$P" set "$J" "$ROOT" profit 10 --status Known --source "made-up markup" >/dev/null
"$P" check "$J"
cp "$J" "$T/before.json"
BEFORE="$("$P" estimate "$J" | tr -s ' ')"; VIF="$("$P" vif "$J")"
# The south wall is 24 by 9: 216 SF at $2.00 is $432.00. The south plane stands on the
# 24 ft eave: on plan (24 + 8) / 2 x 8 = 128 SF; at 6:12 the slope factor is the root
# of 1.25, 1.118034: 143.1084 SF at $0.50 is $71.55. Job cost 503.55; overhead at 20 %
# 100.71; profit at 10 % of 604.26, 60.43; price 664.69. The roof is Assumed, so the
# price is flagged.
says "the board" "Gypsum board, hung and finished 216.00 SF \$2.00/SF \$432.00" "$BEFORE"
says "the underlayment" "Underlayment ~143.1084 SF \$0.50/SF ~\$71.55" "$BEFORE"
says "the price, flagged" "PRICE ~\$664.69" "$BEFORE"
says "the roof is a question, with the plane's dollars" "Main roof [$ROOF] (roof): ~\$71.55, 1 line" "$VIF"

echo
echo "== The session: the roof from the list, measured and checked; a labor line; the wall deleted; a markup typed too large =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
sheet "Verify in field"                 # what there is to verify, by the dollars riding on each
sheet "Verify Main roof"                # the roof's row: the roof is selected in the model window
shot                                    # 01 the roof on the list with \$71.55 riding; its values say Measure and Check
on "Eave height"                        # Eave height: Measure
laser "9' 2\"" Return                   # shot at the eave
shot                                    # 02 the eave height Known; the roof solved again
on "Footprint"; laser "same" Return     # Footprint: Check
on "Edge 1"; laser "same" Return        # each edge's pitch: Check
on "Edge 2"; laser "same" Return
on "Edge 3"; laser "same" Return
on "Edge 4"; laser "same" Return
sheet "Estimate"
shot                                    # 03 nothing left to verify on the roof: the underlayment is Known
sheet_at "Unit cost of Gypsum board, hung and finished on wall 1"; sleep 2   # the wall, by its line on the sheet
esc
on "Add cost line"; sleep 3             # labor on the wall
xdotool type --delay 250 "Hang the board"; sleep 1
on "Priced by net area"; sleep 3
xdotool type --delay 250 "labor at 40 SF/hr"; sleep 1
xdotool key Return; sleep 5
focus
shot                                    # 04 the wall has a labor line: 5.40 hr, no price yet
on "Delete"; sleep 4                    # the wall deleted
shot                                    # 05 what went with it is said
sheet_at "Overhead percent" 4; sleep 2  # overhead, typed ten times too large
xdotool type --delay 300 "200"; sleep 2; xdotool key Return; sleep 4
focus
shot                                    # 06 taken, and answered
sheet_at "Overhead percent" 4; sleep 2
xdotool type --delay 300 "20"; sleep 2; xdotool key Return; sleep 4
focus
xdotool key F2; sleep 3                 # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  12 pictures saved to $OUT (model and sheet, after each of 6 steps)"

echo
echo "== What the program said as it went =="
# Every message the window gave, in the words on the screen (crates/app/src/map.rs).
LOG="$(cat "$T/map.txt.said")"
says "the eave, measured at the roof" "eave height is 9' 2\", measured (it was 9 ft, Assumed" "$LOG"
# The wall carried the board, \$432.00, and the labor line added a moment before, which
# had hours and no price.
says "the delete" "2 cost lines went with it: \$432.00 is out of the job cost (1 of them had no price yet)." "$LOG"
says "the markup" "overhead set to 200.00 %. That is more than what it is a percentage of. If 20.00 % was meant, enter it again." "$LOG"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
AFTER="$("$P" estimate "$J" | tr -s ' ')"; VIF="$("$P" vif "$J")"
if grep -qF "Gypsum board" <<<"$AFTER"; then echo "FAIL: the deleted wall's line is still in the estimate"; exit 1; fi
echo "    the wall's lines are gone from the estimate"
# The roof is Known now: 143.1084 SF at \$0.50 is \$71.55, no longer flagged. Overhead
# typed in the window at 20 % is Assumed: 14.31. Profit at 10 % of 85.86 is 8.59. Price 94.45.
says "the underlayment, Known" "Underlayment 143.1084 SF \$0.50/SF \$71.55" "$AFTER"
says "overhead, as typed last" "Overhead at ~20.00 % ~\$14.31" "$AFTER"
says "the price" "PRICE ~\$94.45" "$AFTER"
if grep -qF "Main roof" <<<"$VIF"; then echo "FAIL: the roof is still on the verify-in-field list"; echo "$VIF"; exit 1; fi
echo "    the roof is off the verify-in-field list"
python3 - "$J" "$ROOF" <<'PY'
import sys, json
d = {e["path"]: e for e in json.load(open(sys.argv[1]))["data"]}
fp = d[sys.argv[2]]["attributes"]["pl::footprint"]
eave = fp["eave_height"]
print("    in the file: eave height %s %s, %s; footprint %s; edges %s" % (eave["value"], eave["unit"], eave["status"], fp["points"]["status"], [e["status"] for e in fp["edges"]]))
ok = eave["value"] == [9.0, 2.0] and eave["status"] == "Known" and fp["points"]["status"] == "Known" and all(e["status"] == "Known" for e in fp["edges"])
sys.exit(0 if ok else 1)
PY
echo "  PASS: a roof was measured off the verify-in-field list from the window, a labor line was added to a wall, the wall's delete said what went with it to the cent, a markup ten times too large was answered, and the saved job says the same"
