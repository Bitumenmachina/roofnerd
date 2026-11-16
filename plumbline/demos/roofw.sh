#!/usr/bin/env bash
# A roof is priced once (DECISIONS.md call 100): an assembly given to one plane of a hip
# roof goes on its other planes too; a price typed on one plane's line is taken by the
# same line on the others; a production rate is said in squares on a line that comes
# out in SF. Each is said in the window as it happens.
# Run from the repository root:  demos/roofw.sh
#
# It drives the real program with a real pointer and keyboard. The job and every price
# are made up (no real job data). Every step fails the script if the program does not
# do what the step says. The figures are worked by hand in the comments and in
# STATUS.md ("A roof priced once, in the window").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/roofw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/roofw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/roofw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -60; exit 1; }; printf '    %s\n' "$2"; }

echo "== The job: a hip roof 40 ft by 26 ft at 5:12, scaled off a plan; nothing priced =="
"$P" new "$J" "A roof priced once (not a real job)" >/dev/null
"$P" level "$J" ground --elevation "0 ft" --height "9 ft" --status Known --source "made-up plan" >/dev/null
"$P" roof "$J" "Main roof" --points "0,0 40,0 40,26 0,26" --pitch 5:12 --eave-height "9 ft" --status Assumed --source "scaled off a made-up plan" >/dev/null
"$P" check "$J"
cp "$J" "$T/before.json"
# Four planes. On plan the two on the 40 ft eaves are (40 + 14) / 2 x 13 = 351 SF and
# the two on the 26 ft eaves 26 x 13 / 2 = 169 SF: 1,040 SF, the footprint. At 5:12
# the slope factor is 13/12: 380.25 SF and 183.0833 SF, 1,126.67 SF in all.
SHOWN="$("$P" show "$J")"
[ "$(grep -cE "= ~ surface area +380.25 SF" <<<"$SHOWN")" = 2 ] && [ "$(grep -cE "= ~ surface area +183.0833 SF" <<<"$SHOWN")" = 2 ] || { echo "FAIL: the four planes are not 380.25 SF and 183.0833 SF, two of each"; exit 1; }
echo "    four planes: 380.25 SF on the two long eaves, 183.0833 SF on the two short"

echo
echo "== The session: starter assemblies; a plane clicked; shingles; a price and a rate typed once =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
sleep 8; park
on "Starter assemblies"; sleep 5         # thirty assemblies, no prices
click 760 470; sleep 4                   # a plane of the roof, on the model
on "Assembly"; sleep 4
shot                                     # 01 the assemblies that go on a roof plane
on "Assembly Asphalt shingle roof"; sleep 6
shot                                     # 02 on the plane clicked, and on the other three: said
sheet_at "Unit cost of Asphalt shingles on plane 1"; sleep 3
xdotool type --delay 250 -- "105"; sleep 1; xdotool key Return; sleep 5
focus
shot                                     # 03 \$105.00/SQ on plane 1's shingles, and on the same line of the others
sheet_at "Production rate of Lay underlayment and shingles on plane 1"; sleep 3
shot                                     # 04 the field open: it says what could be typed
xdotool type --delay 250 -- "2 hr/SQ"; sleep 1; xdotool key Return; sleep 5
focus
shot                                     # 05 hours on every plane, from a rate said in squares
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
# Every message the window gave, in the words on the screen (crates/app/src/map.rs).
LOG="$(cat "$T/map.txt.said")"
# Whichever plane the click landed on, the other three are named.
grep -qE 'is now "Asphalt shingle roof": 5 cost lines added\. So are plane [1-4], plane [1-4] and plane [1-4]: the same lines on each\.' <<<"$LOG" || { echo "FAIL: the assembly was not said to go on the other three planes"; echo "$LOG"; exit 1; }
echo "    the assembly: on the plane clicked, 5 cost lines; so are the other three, the same lines on each"
says "the price" "Plane 1: unit cost of \"Asphalt shingles\" set to \$105.00. The same line on plane 2, plane 3 and plane 4 had none: they take it too." "$LOG"
says "the rate" "Plane 1: production rate of \"Lay underlayment and shingles\" set to 2 hr/SQ. The same line on plane 2, plane 3 and plane 4 had none: they take it too." "$LOG"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
AFTER="$("$P" estimate "$J" | tr -s ' ')"
# Shingles by the square: 380.25 SF is 3.8025 SQ, at \$105.00 \$399.26 (399.2625);
# 183.0833 SF is 1.8308 SQ, \$192.23 (192.234). Two of each: \$1,182.98. Laying at 2 hr
# a square is 0.02 hr/SF: 380.25 x 0.02 = 7.6050 hr, and 183.0833 x 0.02 = 3.6617 hr.
# Everything rests on a roof scaled off a plan, so every figure is flagged.
[ "$(grep -cF "Asphalt shingles ~3.8025 SQ ~\$105.00/SQ ~\$399.26" <<<"$AFTER")" = 2 ] || { echo "FAIL: not two planes of 3.8025 SQ at \$105.00"; echo "$AFTER" | head -50; exit 1; }
[ "$(grep -cF "Asphalt shingles ~1.8308 SQ ~\$105.00/SQ ~\$192.23" <<<"$AFTER")" = 2 ] || { echo "FAIL: not two planes of 1.8308 SQ at \$105.00"; echo "$AFTER" | head -50; exit 1; }
echo "    shingles: 3.8025 SQ and \$399.26 on two planes, 1.8308 SQ and \$192.23 on two"
[ "$(grep -cF "Lay underlayment and shingles ~7.6050 hr" <<<"$AFTER")" = 2 ] && [ "$(grep -cF "Lay underlayment and shingles ~3.6617 hr" <<<"$AFTER")" = 2 ] || { echo "FAIL: the hours are not 7.6050 and 3.6617, two planes each"; echo "$AFTER" | head -50; exit 1; }
echo "    laying: 7.6050 hr on two planes, 3.6617 hr on two"
python3 - "$J" <<'PY'
import sys, json
d = [e for e in json.load(open(sys.argv[1]))["data"] if e["path"].startswith("plane")]
ok = len(d) == 4
for e in d:
    lines = {l["description"]: l for l in e["attributes"]["pl::cost"]["lines"]}
    price, rate = lines["Asphalt shingles"]["unit_cost"], lines["Lay underlayment and shingles"]["order"]["factor"]
    ok = ok and len(lines) == 5 and price["value"] == 105.0 and price["status"] == "Assumed" and "entered on screen" in price["source"]
    ok = ok and abs(rate["value"] - 0.02) < 1e-12 and rate["unit"] == "hr/SF" and rate["status"] == "Assumed"
print("    in the file: 4 planes, 5 lines each; shingles $105.00 Assumed, entered on screen; laying 0.02 hr/SF Assumed" if ok else "    in the file: not as typed")
sys.exit(0 if ok else 1)
PY
echo "  PASS: an assembly given to one plane went on the whole roof and said so, a price typed once priced the shingles of all four planes, a rate said in squares gave each plane its hours, and the saved job says the same"
