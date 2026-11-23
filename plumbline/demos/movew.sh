#!/usr/bin/env bash
# A wall moved in the window, and what went with it (calls 170, 171 item 1, 179 B, 180 B).
# A made-up box house opened cold: 24 by 16 ft, 6 in walls 9 ft tall, a strip footing under
# it, a floor and a hip... a 6:12 roof over it, a room, and prices on the floor, the footings
# and the walls (the figures are those of demos/stretch.sh, worked by hand there: price
# $7,600.00; the east wall 4 ft east takes the floor's edge, the footings and the roof's edge
# and the price becomes $8,616.00).
#   1. the east wall picked on the plan, Move, a point on it, "4 ft" typed with the pointer
#      east of it: the card says what went with it, in the model's own sentences;
#   2. what the window saved is, byte for byte, what the command line's `move` makes of the
#      same job with the same words (the program's own source on it);
#   3. one Undo takes all of it back: the job is as it was opened;
#   4. a move across another wall (30 ft west) is refused in the model's words and nothing
#      changes.
# Run from the repository root:  demos/movew.sh [folder for the pictures]
# It drives the real program on a virtual display with a real pointer and keyboard.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/movew.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/movew.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/movew}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -60; exit 1; }; printf '    %s\n' "$2"; }
total() { "$P" estimate "$1" | grep -E 'column totals' | grep -oE '\$[0-9,]+\.[0-9]+' | tr -d '$,' | awk '{ s += $1 } END { printf "%.2f", s }'; }

echo "== The job: the box of demos/stretch.sh, opened cold =="
"$P" new "$J" "A wall moved (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
FT="$(made "$("$P" type "$J" footing "footing 24 in" --width "24 in" "${K[@]}")")"
FL="$(made "$("$P" type "$J" floor "floor 10 in" --thickness "10 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" footing "$J" "$S" "$FT" --points "0,0 24,0 24,16 0,16" --closed --depth "12 in" "${K[@]}" >/dev/null
FLOOR="$(made "$("$P" floor "$J" "$S" "$FL" --walls "${K[@]}")")"
"$P" roof "$J" "Main roof" --points "0,0 24,0 24,16 0,16" --pitch 6:12 --eave-height "9 ft" "${K[@]}" >/dev/null
ROOF="$(grep -oE 'roof-[0-9]+' "$J" | head -1)"
"$P" set "$J" "$ROOF" edge:2 gable "${K[@]}" >/dev/null
"$P" set "$J" "$ROOF" edge:4 gable "${K[@]}" >/dev/null
"$P" room "$J" "$S" Inside --at "12,8" "${K[@]}" >/dev/null
"$P" line "$J" "$FLOOR" "Floor" plan_area --cost-type material >/dev/null
"$P" set "$J" "$FLOOR" unit_cost '$10/SF' --line "Floor" "${K[@]}" >/dev/null
for f in $(grep -oE 'footing-[0-9]+' "$J" | sort -u); do
  "$P" line "$J" "$f" "Footing" plan_length --cost-type subcontract >/dev/null
  "$P" set "$J" "$f" unit_cost '$20/LF' --line "Footing" "${K[@]}" >/dev/null
done
for w in $SOUTH $EAST $NORTH $WEST; do
  "$P" line "$J" "$w" "Wall" face_area --cost-type subcontract >/dev/null
  "$P" set "$J" "$w" unit_cost '$3/SF' --line "Wall" "${K[@]}" >/dev/null
done
"$P" check "$J" >/dev/null
[ "$(total "$J")" = "7600.00" ] || { echo "FAIL: the price is $(total "$J"), not 7600.00"; exit 1; }
echo "    price \$7,600.00 (walls 2,160 + floor 3,840 + footings 1,600)"
cp "$J" "$T/start.json"
# What the command line makes of the same move, in the window's own words for who did it:
# a move in the window is Assumed, "entered on screen; not yet checked against a drawing".
cp "$J" "$T/cli.json"
"$P" move "$T/cli.json" "$EAST" --by "4 ft east" --status Assumed --source "entered on screen; not yet checked against a drawing" > "$T/cli.txt"
cat "$T/cli.txt" | sed 's/^/    cli: /'
[ "$(total "$T/cli.json")" = "8616.00" ] || { echo "FAIL: the command line's price is not 8616.00"; exit 1; }

echo
echo "== The session: pick the east wall, Move, 4 ft; Undo; a move across the house =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
xdotool key t; sleep 6                  # the plan
park
shot                                    # 01 the house on plan, before
at 24 8                                 # the east wall, selected
shot                                    # 02 the east wall selected
on "Move"                               # Move what is selected
at 24 8                                 # from a point on it
len \$(plan_px 31 8) "4 ft" yes         # 03 four feet typed, the pointer east of it
park
shot                                    # 04 what went with it is said; the sheet shows the price
xdotool key F2; sleep 4; cp "$J" "$T/window.json"
xdotool key z; sleep 5                  # one Undo
park
shot                                    # 05 all of it taken back
xdotool key F2; sleep 4; cp "$J" "$T/undone.json"
at 24 8                                 # the east wall again
on "Move"
at 24 8
len \$(plan_px 10 8) "30 ft" yes        # 06 thirty feet west, across the west wall
park
shot                                    # 07 refused, in the model's words
xdotool key F2; sleep 4; cp "$J" "$T/refused.json"
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05 07; do
  [ "$OUT/$n-model.png" -nt "$T/start.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/start.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done

echo
echo "== What the window said =="
LOG="$(cat "$T/map.txt.said")"
echo "$LOG" | sed 's/^/    | /'
says "the move" "Moved wall 2 4 ft east." "$LOG"
# Every line the model gives for this move is said in the window, in the model's own words.
while IFS= read -r line; do
  case "$line" in *" moved "*) continue ;; esac
  line="$(sed 's/^ *//' <<<"$line")"
  [ -n "$line" ] && says "what went with it" "$line" "$LOG"
done < "$T/cli.txt"
says "the floor" "floor 1 followed: its edge on wall 2 went with it, 384.00 SF now 448.00 SF" "$LOG"
says "the footing" "footing 1 followed: 4 ft longer" "$LOG"
says "the roof" "Main roof followed: its edge over wall 2 went with it, plan area 384.00 SF now 448.00 SF, surface 429.33 SF now 500.88 SF" "$LOG"
says "the refusal" "Can't" "$LOG"
says "the refusal, in the model's words" "wall 2 would run into wall 4, a wall it does not meet" "$LOG"

echo
echo "== What it left in the saved jobs =="
cmp -s "$T/window.json" "$T/cli.json" && echo "    the window's saved job is, byte for byte, what the command line's move makes of the same job" || { echo "FAIL: the window's job is not the command line's:"; diff <("$P" show "$T/window.json") <("$P" show "$T/cli.json") | head -30; diff "$T/window.json" "$T/cli.json" | head -20; exit 1; }
[ "$(total "$T/window.json")" = "8616.00" ] || { echo "FAIL: the window's price is not 8616.00"; exit 1; }
echo "    price \$7,600.00 to \$8,616.00 (floor +640, footings +160, walls +216)"
cmp -s "$T/undone.json" "$T/start.json" && echo "    one Undo: the job is as it was opened, byte for byte" || { echo "FAIL: one Undo did not take all of it back"; diff <("$P" show "$T/undone.json") <("$P" show "$T/start.json") | head -30; exit 1; }
cmp -s "$T/refused.json" "$T/start.json" && echo "    the refused move changed nothing" || { echo "FAIL: the refused move changed the job"; diff <("$P" show "$T/refused.json") <("$P" show "$T/start.json") | head -30; exit 1; }
"$P" check "$T/window.json" > "$T/check.txt"; if grep -q '^!' "$T/check.txt"; then echo "FAIL: check cautions"; cat "$T/check.txt"; exit 1; fi
echo "    (check: no caution)"
echo "PASS: a wall moved in the window took its floor, footing and roof with it and said so, one Undo took it all back, and a move across a wall was refused in the model's words"
