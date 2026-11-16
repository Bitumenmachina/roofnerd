#!/usr/bin/env bash
# Priced work taken off in the window, and what the window says of the money (DECISIONS.md
# calls 91, 95 and 103): a wall's assembly changed for another, with a priced line coming
# off; then a hip end of a priced roof made a gable, which drops a plane and its lines
# and makes the two planes beside it bigger.
# Run from the repository root:  demos/gablew.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard. The job and every price
# are made up (no real job data). Every step fails the script if the program does not
# do what the step says. The figures are worked by hand in the comments and in
# STATUS.md ("Priced work taken off, in the window").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/gablew.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/gablew.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/gablew}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -60; exit 1; }; printf '    %s\n' "$2"; }

echo "== The job: 24 ft by 16 ft, 9 ft walls, a hip roof at 6:12; siding on the south wall and shingles on the roof, priced =="
"$P" new "$J" "Priced work taken off (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH _ <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" roof "$J" "Main roof" --points "0,0 24,0 24,16 0,16" --pitch 6:12 --eave-height "9 ft" --status Assumed --source "scaled off a made-up plan" >/dev/null
"$P" import assemblies "$J" starter >/dev/null
"$P" apply "$J" "$SOUTH" "Exterior wall, 2x6, lap siding" --source "the starter library" >/dev/null
"$P" set "$J" "$SOUTH" unit_cost '$4.00/SF' --line "Lap siding" --status Known --source "made-up price" >/dev/null
ALL="$("$P" show "$J")"
for n in 1 2 3 4; do
  PL="$(awk -v n="$n" '$2=="plane" && $3==n {print $1; exit}' <<<"$ALL")"
  "$P" apply "$J" "$PL" "Asphalt shingle roof" --source "the starter library" >/dev/null
  "$P" set "$J" "$PL" unit_cost '$100.00/SQ' --line "Asphalt shingles" --status Known --source "made-up price" >/dev/null
done
"$P" check "$J"
cp "$J" "$T/before.json"
BEFORE="$("$P" estimate "$J" | tr -s ' ')"
# The south wall is 24 by 9, 216 SF: siding at $4.00 is $864.00. The roof: on plan the
# planes on the 24 ft eaves are (24 + 8) / 2 x 8 = 128 SF and those on the 16 ft eaves
# 16 x 8 / 2 = 64 SF; at 6:12 the slope factor is 1.118034: 143.1084 SF and 71.5542
# SF, 1.4311 SQ and 0.7155 SQ, at $100.00 $143.11 and $71.55. Two of each: $429.32.
# Job cost 864.00 + 429.32 = 1,293.32. The roof is Assumed, so the total is flagged.
says "the siding" "Lap siding 216.00 SF \$4.00/SF \$864.00" "$BEFORE"
[ "$(grep -cF "Asphalt shingles ~1.4311 SQ \$100.00/SQ ~\$143.11" <<<"$BEFORE")" = 2 ] && [ "$(grep -cF "Asphalt shingles ~0.7155 SQ \$100.00/SQ ~\$71.55" <<<"$BEFORE")" = 2 ] || { echo "FAIL: the shingles are not \$143.11 on two planes and \$71.55 on two"; echo "$BEFORE" | head -60; exit 1; }
echo "    shingles: \$143.11 on two planes, \$71.55 on two"
says "the job cost" "~\$1,293.32" "$BEFORE"

echo
echo "== The session: the wall's assembly changed to brick; the east hip made a gable =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
sleep 8; park
sheet "Estimate"
sheet_at "Unit cost of Lap siding on wall 1"; sleep 2   # the wall, by its priced line
esc
shot                                     # 01 the wall with its siding priced
on "Assembly"; sleep 4
on "Assembly Exterior wall, 2x6, brick veneer"; sleep 6
shot                                     # 02 brick: the siding's lines came off, and their dollars are said
sheet_at "Unit cost of Asphalt shingles on plane 2"; sleep 2   # the east plane, by its priced line
esc
shot                                     # 03 the east hip plane selected
on "Gable"; sleep 8
shot                                     # 04 a gable: three planes; what went, and the job cost before and after
xdotool key F2; sleep 3                  # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  8 pictures saved to $OUT (model and sheet, after each of 4 steps)"

echo
echo "== What the program said as it went =="
# Every message the window gave, in the words on the screen (crates/app/src/map.rs).
LOG="$(cat "$T/map.txt.said")"
# Brick for lap siding: the two assemblies share the framing, the sheathing, the
# barrier, the insulation, the board, the paint and the framing labor. Lap siding and
# the labor of hanging it come off: the siding was $864.00 and its labor had no price.
# (A name in quotes is said as it is kept, "2x6": only words outside quotes are respelled.)
says "the assembly changed" "is now \"Exterior wall, 2x6, brick veneer\"" "$LOG"
says "what came off, in dollars" "\$864.00 is out of the job cost (1 of them had no price yet)." "$LOG"
# The east plane goes with its five lines, of which the shingles were priced: $71.55.
# The planes on the long eaves now run to the east wall: on plan (24 + 16) / 2 x 8 =
# 160 SF each, 178.8854 SF on the slope, 1.7889 SQ, $178.89. The west hip is as it was.
# Shingles 178.89 + 178.89 + 71.55 = 429.33: a gable and a hip of one pitch cover the
# same roof, so the job cost moves by a cent of rounding and no more.
says "what went with the plane" "5 cost lines went with it: \$71.55 is out of the job cost (4 of them had no price yet)." "$LOG"
says "and the whole of it" "The planes that stay changed with it: the job cost was \$429.32 and is \$429.33." "$LOG"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
AFTER="$("$P" estimate "$J" | tr -s ' ')"
if grep -qF "Lap siding" <<<"$AFTER"; then echo "FAIL: the siding's line is still in the estimate"; exit 1; fi
echo "    the siding's line is gone from the estimate"
[ "$(grep -cF "Asphalt shingles ~1.7889 SQ \$100.00/SQ ~\$178.89" <<<"$AFTER")" = 2 ] && [ "$(grep -cF "Asphalt shingles ~0.7155 SQ \$100.00/SQ ~\$71.55" <<<"$AFTER")" = 1 ] || { echo "FAIL: the shingles are not \$178.89 on two planes and \$71.55 on one"; echo "$AFTER" | head -60; exit 1; }
echo "    shingles: \$178.89 on the two long planes, \$71.55 on the west hip; three planes"
says "the job cost" "~\$429.33" "$AFTER"
echo "  PASS: a wall's assembly was changed and a roof's hip made a gable in the window, each said what came off to the cent, the roof change said the job cost before and after, and the saved job says the same"
