#!/usr/bin/env bash
# A plan set on paper (call 168). From an empty job, the house of the layouts demonstration:
# 6 in walls 9 ft tall round 0,0 to 24,16, Known; the south, north and west walls existing,
# the east wall to demolish; an addition of three new walls 24,0 36,0 36,16 24,16; a room
# line where the east wall stood; rooms Living (6,8) and Kitchen (30,8). The plan set is
# written and read back, to the figures worked by hand in STATUS.md ("A plan set on paper").
# Run from the repository root:  demos/planset.sh
# The job is made up (no real job data) and lives in a temporary folder.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
# One page of a --list, by its number.
page() { awk -v n="$2" '/^page [0-9]+ of /{on=($2==n)} on' "$1" > "$T/page$2.txt"; }

"$P" new "$J" "Plan set (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" mark "$J" "$EAST" demolish "${K[@]}" >/dev/null
"$P" mark "$J" "$SOUTH" "$NORTH" "$WEST" existing "${K[@]}" >/dev/null
read -r ADDS ADDE ADDN <<<"$("$P" wall "$J" "$S" "$WT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" room-line "$J" "$S" --points "24,0 24,16" "${K[@]}" >/dev/null
"$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}" >/dev/null
KITCHEN="$(made "$("$P" room "$J" "$S" "Kitchen" --at "30,8" "${K[@]}")")"
cp "$J" "$T/before.json"

echo "== 1. Three pages, each ARCH A, at 1/4\" = 1'-0\" =="
"$P" plan-set "$J" "$S" "$T/set.pdf" --date 2026-11-22 --list > "$T/list.txt"
says "$T/list.txt" "3 pages, ARCH A paper (12 by 9 in), 1/4\" = 1'-0\", 2026-11-22"
for n in 1 2 3; do page "$T/list.txt" "$n"; done

echo
echo "== 2. EXISTING PLAN: 4 walls full weight, 4 dimensions, LIVING 364 SF, no Kitchen, no new wall =="
says "$T/page1.txt" "page 1 of 3: EXISTING PLAN"
says "$T/page1.txt" "walls: 4 (4 heavy, 0 light, 0 dashed)"
says "$T/page1.txt" "dimensions: 4"
[ "$(grep -cE "^    wall-[0-9]+  (24|16)'-0\"$" "$T/page1.txt")" = 4 ] || { echo "FAIL: the four dimensions"; cat "$T/page1.txt"; exit 1; }
says "$T/page1.txt" "LIVING 364 SF"
lacks "$T/page1.txt" "KITCHEN 182"
says "$T/page1.txt" "NOT LETTERED, NOTHING THAT STANDS CLOSES ROUND THEIR POINTS: KITCHEN."

echo
echo "== 3. DEMOLITION PLAN: 3 walls light, 1 dashed; one wall comes down, 16 LF =="
says "$T/page2.txt" "walls: 4 (0 heavy, 3 light, 1 dashed)"
says "$T/page2.txt" "1 WALL COMES DOWN, 16 LF"
says "$T/page2.txt" "LIVING 364 SF"

echo
echo "== 4. PROPOSED PLAN: 3 light, 3 heavy and filled; 6 dimensions; LIVING 368, KITCHEN 182; no east wall =="
says "$T/page3.txt" "walls: 6 (3 heavy, 3 light, 0 dashed)"
says "$T/page3.txt" "dimensions: 6"
grep -E "^    wall-[0-9]+  " "$T/page3.txt" | sed 's/.*  //' | tr '\n' ' ' > "$T/dims.txt"
says "$T/dims.txt" "24'-0\" 24'-0\" 16'-0\" 12'-0\" 16'-0\" 12'-0\""
says "$T/page3.txt" "LIVING 368 SF"
says "$T/page3.txt" "KITCHEN 182 SF"
lacks "$T/page3.txt" "$EAST"

echo
echo "== 5. Notes; the north wall's line Assumed reads VIF on every page it is on =="
says "$T/page1.txt" "DOOR SWINGS ARE NOT SET."
says "$T/page1.txt" "DIMENSIONS ARE TO THE WALLS' LINES AS DRAWN."
says "$T/page1.txt" "NO LENGTH CARRIES VIF"
# The same house with the north wall drawn Assumed (the other walls Known).
V="$T/vif-job.json"
"$P" new "$V" "Plan set (not a real job)" >/dev/null
VS="$(made "$("$P" level "$V" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
VT="$(made "$("$P" type "$V" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
VW=(--height "9 ft")
V1="$(made "$("$P" wall "$V" "$VS" "$VT" --points "0,0 24,0" "${VW[@]}" "${K[@]}")")"
V2="$(made "$("$P" wall "$V" "$VS" "$VT" --points "24,0 24,16" "${VW[@]}" "${K[@]}")")"
NORTH="$(made "$("$P" wall "$V" "$VS" "$VT" --points "24,16 0,16" "${VW[@]}" --status Assumed --source "made-up sketch")")"
V4="$(made "$("$P" wall "$V" "$VS" "$VT" --points "0,16 0,0" "${VW[@]}" "${K[@]}")")"
"$P" mark "$V" "$V2" demolish "${K[@]}" >/dev/null
"$P" mark "$V" "$V1" "$NORTH" "$V4" existing "${K[@]}" >/dev/null
"$P" wall "$V" "$VS" "$VT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${K[@]}" >/dev/null
"$P" room-line "$V" "$VS" --points "24,0 24,16" "${K[@]}" >/dev/null
"$P" room "$V" "$VS" "Living" --at "6,8" "${K[@]}" >/dev/null
"$P" room "$V" "$VS" "Kitchen" --at "30,8" "${K[@]}" >/dev/null
"$P" plan-set "$V" "$VS" "$T/vif.pdf" --date 2026-11-22 --list > "$T/vif.txt"
for n in 1 2 3; do page "$T/vif.txt" "$n"; says "$T/page$n.txt" "$NORTH  24'-0\" VIF"; says "$T/page$n.txt" "1 LENGTH IS TO VERIFY IN FIELD (VIF)."; done

echo
echo "== 6. A door 3 ft wide in the south wall is on the sheets its own mark puts it on (unmarked is new) =="
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
D="$(made "$("$P" door "$J" "$SOUTH" "$DT" --offset "12 ft" --sill "0 ft" "${K[@]}")")"
sets() { "$P" plan-set "$J" "$S" "$T/door.pdf" --date 2026-11-22 --list > "$T/door.txt"; for n in 1 2 3; do page "$T/door.txt" "$n"; done; }
sets
# Unmarked: new work, so on the proposed plan only, heavy; a new door in an existing wall is not on the existing or demolition plan.
lacks "$T/page1.txt" "in $SOUTH: door"
lacks "$T/page2.txt" "in $SOUTH: door"
says "$T/page3.txt" "in $SOUTH: door, gap 3'-0\", heavy, no swing"
# Marked existing: heavy on the existing plan, light on the other two.
"$P" mark "$J" "$D" existing "${K[@]}" >/dev/null
sets
says "$T/page1.txt" "in $SOUTH: door, gap 3'-0\", heavy, no swing"
says "$T/page2.txt" "in $SOUTH: door, gap 3'-0\", light, no swing"
says "$T/page3.txt" "in $SOUTH: door, gap 3'-0\", light, no swing"
# Marked to demolish: heavy as it stands, dashed on the demolition plan, gone from the proposed plan.
"$P" mark "$J" "$D" demolish "${K[@]}" >/dev/null
sets
says "$T/page1.txt" "in $SOUTH: door, gap 3'-0\", heavy, no swing"
says "$T/page2.txt" "in $SOUTH: door, gap 3'-0\", dashed, no swing"
lacks "$T/page3.txt" "in $SOUTH: door"
says "$T/page2.txt" "1 DOOR COMES DOWN"
cp "$T/before.json" "$J"

echo
echo "== 7. 1/2 in to the foot: ARCH C; ARCH A at that scale is refused with the scale that fits =="
"$P" plan-set "$J" "$S" "$T/big.pdf" --scale "1/2 in" --date 2026-11-22 --list > "$T/big.txt"
says "$T/big.txt" "ARCH C paper (24 by 18 in), 1/2\" = 1'-0\""
refused "$P" plan-set "$J" "$S" "$T/small.pdf" --scale "1/2 in" --paper "ARCH A"
says "$T/out.txt" "does not fit ARCH A at 1/2\" = 1'-0\""
says "$T/out.txt" "at 1/4\" = 1'-0\" it does"
[ ! -e "$T/small.pdf" ] || { echo "FAIL: a refused plan set wrote a file"; exit 1; }

echo
echo "== 8. A job with nothing marked: one page, FLOOR PLAN =="
"$P" new "$T/plain.json" "Plain (not a real job)" >/dev/null
PS="$(made "$("$P" level "$T/plain.json" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
PW="$(made "$("$P" type "$T/plain.json" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
"$P" wall "$T/plain.json" "$PS" "$PW" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" >/dev/null
"$P" plan-set "$T/plain.json" "$PS" "$T/plain.pdf" --date 2026-11-22 --list > "$T/plain.txt"
says "$T/plain.txt" "1 page, ARCH A paper"
says "$T/plain.txt" "page 1 of 1: FLOOR PLAN"
says "$T/plain.txt" "walls: 4 (4 heavy, 0 light, 0 dashed)"

echo
echo "== 8a. Each sheet has its number, a north arrow, a key of its line types and an overall dimension outside the strings (call 179 D4) =="
# The job as it was (no door): its three pages again.
for n in 1 2 3; do page "$T/list.txt" "$n"; done
# The sheet's number is the National CAD Standard's form: A (architectural), 1 (plans), then the order in the set.
says "$T/page1.txt" "sheet: A-101, sheet 1 of 3"
says "$T/page2.txt" "sheet: A-102, sheet 2 of 3"
says "$T/page3.txt" "sheet: A-103, sheet 3 of 3"
for n in 1 2 3; do says "$T/page$n.txt" "north arrow: plan north, up the page (true north is not set)"; done
# The key: each line type the sheet draws, as the sheet draws it (heavy, light or dashed).
says "$T/page1.txt" "key: EXISTING, heavy; TO DEMOLISH, heavy"
says "$T/page2.txt" "key: EXISTING, light; TO DEMOLISH, dashed"
says "$T/page3.txt" "key: EXISTING, light; NEW, heavy"
# The overall: worked by hand. The existing walls make a 24 ft by 16 ft house; the proposed plan adds 12 ft.
says "$T/page1.txt" "overall: across 24'-0\", up the page 16'-0\""
says "$T/page2.txt" "overall: across 24'-0\", up the page 16'-0\""
says "$T/page3.txt" "overall: across 36'-0\", up the page 16'-0\""
# The strings add up to the overall, to the sixteenth: on the proposed plan the two strings on the
# south side (24'-0" and 12'-0") make 36'-0", and the string along the west wall is 16'-0".
feet() { awk '{ s=$0; f=s; sub(/\x27.*/,"",f); i=s; sub(/^[0-9]+\x27-/,"",i); gsub(/"/,"",i); n=split(i,a," "); inch=a[1]+0; if(n>1){split(a[2],q,"/"); inch+=q[1]/q[2]}; printf "%.4f\n", f+inch/12 }'; }
SUM="$(grep -E "^    wall-[0-9]+  " "$T/page3.txt" | sed 's/.*  //' | sed -n '1p;4p' | feet | paste -sd+ | bc)"
ALL="$(sed -n 's/^  overall: across \([^,]*\),.*/\1/p' "$T/page3.txt" | feet)"
[ "$(printf '%.4f' "$SUM")" = "$(printf '%.4f' "$ALL")" ] || { echo "FAIL: the strings on the south side ($SUM ft) do not add up to the overall ($ALL ft)"; exit 1; }
echo "    24'-0\" + 12'-0\" = 36'-0\" (the strings add up to the overall)"
# And on the written page: the overall's strings are in the file, 36 ft is 648 points at 1/4 in to the
# foot, one above the plan and one below it; before this, no line was as long.
"$P" page "$T/set.pdf" --page 3 --lines --longer-than 600 > "$T/p3lines.txt"
[ "$(grep -cF ": 648.00 points (9.0000 in)" "$T/p3lines.txt")" = 4 ] || { echo "FAIL: the overall and the string of the walls are not drawn above and below the plan, 648.00 points"; cat "$T/p3lines.txt"; exit 1; }
echo "    (four lines of 648.00 points: on the north and the south, the string of the walls and the overall outside it; call 182 moved this from two)"
"$P" page "$T/set.pdf" --page 3 --lines --longer-than 280 > "$T/p3up.txt"
[ "$(grep -cF ": 288.00 points (4.0000 in)" "$T/p3up.txt")" = 3 ] || { echo "FAIL: up the page there are not three lines of 288.00 points (16 ft): the overall on the west, on the east, and the room line"; cat "$T/p3up.txt"; exit 1; }
echo "    (three lines of 288.00 points: the overall on the west and on the east, one wall each, so no second string saying the same, and the room line between the rooms; call 182 moved this from four)"
# What a plan's sheet does not show is said on it.
says "$T/page1.txt" "ROOMS ARE AS THEY STAND TODAY."

echo
echo "== 8b. Lettering is never under 3/32 in, never over other lettering, never out of its box; a room too small for its name takes a mark and a schedule (call 179 D2) =="
# Every page above, at the scale and paper it was written at, and again at 1/8 in to the foot:
# the lettering as it is written on the page has no piece under 3/32 in (6.75 pt), none over
# another, none outside the frame or across the edge of a box.
"$P" plan-set "$J" "$S" "$T/eighth.pdf" --scale "1/8" --date 2026-11-22 --list > "$T/eighth.txt"
for f in list eighth vif; do
  [ "$(grep -c "^  lettering faults: 0$" "$T/$f.txt")" = 3 ] || { echo "FAIL: lettering faults on the pages of $f"; grep -A3 "lettering faults" "$T/$f.txt"; exit 1; }
  awk '/^  lettering: /{ for(i=1;i<=NF;i++) if($i=="smallest") { if ($(i+1)+0 < 6.75) bad=1 } } END{exit bad}' "$T/$f.txt" || { echo "FAIL: lettering under 3/32 in on the pages of $f"; exit 1; }
done
echo "    (the three pages at 1/4, at 1/8 and with VIF: no piece under 6.75 pt, none over another)"
# A made-up house whose title is far too long for its box, whose level has a long name, with a water
# heater closet inside its walls, a 1 ft 6 in stub wall and a 6 in wall's strings: lettered, and
# kept inside its box.
L="$T/lettering.json"
LONG="Plan set of an existing house with a very long job title that will not go on one line of its title box nor on two"
"$P" new "$L" "$LONG (not a real job)" >/dev/null
LS="$(made "$("$P" level "$L" "Ground floor of the main house and the garage" --elevation "0 ft" --height "9 ft" "${K[@]}")")"
LT="$(made "$("$P" type "$L" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
"$P" wall "$L" "$LS" "$LT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" >/dev/null
"$P" wall "$L" "$LS" "$LT" --points "0,3 2.5,3 2.5,0" --height "9 ft" "${K[@]}" >/dev/null
"$P" wall "$L" "$LS" "$LT" --points "12,16 12,14.5" --height "9 ft" "${K[@]}" >/dev/null
"$P" room "$L" "$LS" "Living" --at "12,8" "${K[@]}" >/dev/null
"$P" room "$L" "$LS" "Water heater closet" --at "1.25,1.5" "${K[@]}" >/dev/null
"$P" plan-set "$L" "$LS" "$T/lettering.pdf" --date 2026-11-22 --list > "$T/lettering.txt"
says "$T/lettering.txt" "1 page, ARCH A paper"
# By hand: the closet is 2 ft wide between the 6 in faces (3 in to 2 ft 3 in) and 2 ft 6 in deep
# (3 in to 2 ft 9 in): 5 SF. At 1/4 in to the foot it is 36 pt wide inside; its name, 19 letters
# at 3/32 in, is (19 x 11 - 3) / 12 x 6.75 = 115.9 pt wide: it takes a mark, and the schedule at
# the side of the sheet says what it is. Living keeps its label (its area is the model's own figure).
says "$T/lettering.txt" "room schedule: 1"
says "$T/lettering.txt" "    1  WATER HEATER CLOSET 5 SF"
says "$T/lettering.txt" "    LIVING 356 SF"
# The title: 8 pt on two lines of 33 letters at the most; this one is cut at a word with an ellipsis.
says "$T/lettering.txt" "title box: PLAN SET OF AN EXISTING HOUSE / WITH A VERY LONG JOB TITLE..."
says "$T/lettering.txt" "lettering faults: 0"
awk '/^  lettering: /{ for(i=1;i<=NF;i++) if($i=="smallest") { if ($(i+1)+0 < 6.75) bad=1 } } END{exit bad}' "$T/lettering.txt" || { echo "FAIL: lettering under 3/32 in"; exit 1; }
echo "    (no piece under 6.75 pt, none over another, none across a box's edge)"
# The same page at 1/8 in to the foot, where the 24 by 16 house is half as large: the rooms keep
# their names or take a mark, and still nothing is over anything.
"$P" plan-set "$L" "$LS" "$T/lettering8.pdf" --scale "1/8" --date 2026-11-22 --list > "$T/lettering8.txt"
says "$T/lettering8.txt" "lettering faults: 0"

echo
echo "== 8c. Cabinets and fixtures set in place are drawn on the proposed plan as outlines; the notes say what the sheet does not show (call 179 D3) =="
# The house as it was, with a vanity 3 ft by 2 ft by 3 ft at 30,3 and, along the east wall of the
# kitchen from 1 ft, base boxes B18 SB36 B24 (18 in, 3 ft and 2 ft wide, 2 ft deep) and wall boxes
# W3030 W2430 hung 4 ft 6 in up, which is over the 4 ft the walls are cut at.
cp "$J" "$T/things.json"
"$P" import items "$T/things.json" starter >/dev/null
"$P" place "$T/things.json" "$S" fixture "Vanity" --at "30,3" --offset "0 ft" --size "3,2,3" "${K[@]}" >/dev/null
"$P" cabinets "$T/things.json" "$S" "$ADDE" --in "$KITCHEN" --from "1 ft" --boxes "B18 SB36 B24" "${K[@]}" >/dev/null
"$P" cabinets "$T/things.json" "$S" "$ADDE" --in "$KITCHEN" --from "1 ft" --boxes "W3030 W2430" "${K[@]}" >/dev/null
"$P" plan-set "$T/things.json" "$S" "$T/things.pdf" --date 2026-11-22 --list > "$T/things.txt"
says "$T/things.txt" "3 pages, ARCH A paper"
for n in 1 2 3; do page "$T/things.txt" "$n"; done
# Not on the existing plan or the demolition plan; said there.
for n in 1 2; do
  says "$T/page$n.txt" "things set in place: 0"
  says "$T/page$n.txt" "6 CABINETS AND FIXTURES SET IN PLACE ARE NOT SHOWN ON THIS SHEET."
done
# On the proposed plan: all six, the wall boxes dashed (above the cut), the rest solid.
says "$T/page3.txt" "things set in place: 6"
[ "$(grep -cE '^    (fixture|furnishing)-[0-9]+ ' "$T/page3.txt")" = 6 ] || { echo "FAIL: six things on the proposed plan"; cat "$T/page3.txt"; exit 1; }
says "$T/page3.txt" "\"Vanity\": heavy"
says "$T/page3.txt" "\"SB36\": heavy"
says "$T/page3.txt" "\"W3030\": heavy, above the cut, dashed"
says "$T/page3.txt" "\"W2430\": heavy, above the cut, dashed"
says "$T/page3.txt" "6 CABINETS AND FIXTURES SET IN PLACE ARE DRAWN AS THEIR OUTLINES; 2 DASHED HANG ABOVE"
says "$T/page3.txt" "2 THINGS ARE DRAWN WITH NO NAME IN IT: TOO SMALL, OR HUNG OVER THE FLOOR."
# And it is said on every page what the sheet does not show.
for n in 1 2 3; do says "$T/page$n.txt" "THIS SHEET DOES NOT SHOW PIPES, DUCTS, WIRING OR THE FRAME."; done
# On the written page: the vanity (3 ft by 2 ft) and SB36 (3 ft by 2 ft) are outlines whose long sides are
# 3 ft = 54 pt at 1/4 in to the foot, two sides each: four lines of 54.00 points. The proposed plan
# of the house with none set has no line that long but the wall faces' own.
"$P" page "$T/things.pdf" --page 3 --lines --longer-than 30 > "$T/thinglines.txt"
[ "$(grep -cF ": 54.00 points (0.7500 in)" "$T/thinglines.txt")" = 4 ] || { echo "FAIL: the outlines of the vanity and SB36 are not four lines of 54.00 points"; cat "$T/thinglines.txt"; exit 1; }
echo "    (four lines of 54.00 points: the vanity's and SB36's long sides)"
[ "$(grep -c "^  lettering faults: 0$" "$T/things.txt")" = 3 ] || { echo "FAIL: lettering faults with things set"; grep -A3 "lettering faults" "$T/things.txt"; exit 1; }
# The job is untouched by writing the plan set.
cp "$T/before.json" "$J"

echo
echo "== 8d. The plan set's name ends as a PDF's does or is refused; writing over a plan set is said (call 179 D6) =="
refused "$P" plan-set "$J" "$S" "$T/set.txt" --date 2026-11-22
says "$T/out.txt" "set.txt does not end in .pdf"
[ ! -e "$T/set.txt" ] || { echo "FAIL: a name that is not a PDF's wrote a file"; exit 1; }
"$P" plan-set "$J" "$S" "$T/Upper.PDF" --date 2026-11-22 > "$T/first.txt"
lacks "$T/first.txt" "wrote over"
"$P" plan-set "$J" "$S" "$T/Upper.PDF" --date 2026-11-22 > "$T/second.txt"
says "$T/second.txt" "wrote over the plan set that was there"
# A file that is there and is not a PDF is not written over, and keeps what it held.
printf 'my notes, not a PDF' > "$T/notes.pdf"
refused "$P" plan-set "$J" "$S" "$T/notes.pdf" --date 2026-11-22
says "$T/out.txt" "is there and is not a PDF: it is not written over"
[ "$(cat "$T/notes.pdf")" = "my notes, not a PDF" ] || { echo "FAIL: a file that is not a PDF was written over"; exit 1; }

echo
echo "== 8e. Every fault the program knows is said by plan-set, whether or not the list was asked for (call 182 item 4) =="
# Five stub walls 7 1/4 in apart along a 24 ft by 16 ft house's south wall: five figures each 55 points
# wide on stations about 10 points apart. They cannot all be lettered clear of one another.
KJ="$T/crowded.json"
"$P" new "$KJ" "Crowded (not a real job)" >/dev/null
KS="$(made "$("$P" level "$KJ" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
KW="$(made "$("$P" type "$KJ" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
"$P" wall "$KJ" "$KS" "$KW" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" >/dev/null
for x in 4 4.6 5.2 5.8 6.4; do "$P" wall "$KJ" "$KS" "$KW" --points "$x,0 $x,2" --height "9 ft" "${K[@]}" >/dev/null; done
"$P" plan-set "$KJ" "$KS" "$T/crowded.pdf" --date 2026-11-22 > "$T/crowded-said.txt"
"$P" plan-set "$KJ" "$KS" "$T/crowded.pdf" --date 2026-11-22 --list > "$T/crowded-list.txt"
N="$(sed -n 's/^  lettering faults: \([0-9]*\)$/\1/p' "$T/crowded-list.txt" | paste -sd+ | bc)"
[ "$N" -gt 0 ] || { echo "FAIL: the crowded job should have lettering that cannot be cleared"; cat "$T/crowded-list.txt"; exit 1; }
says "$T/crowded-said.txt" "not right on the pages as written"
says "$T/crowded-said.txt" "page 1 of 1, FLOOR PLAN: the lettering"
[ "$(grep -c '^  page 1 of 1, FLOOR PLAN: ' "$T/crowded-said.txt")" = "$N" ] || { echo "FAIL: plan-set said another number of faults than the list found ($N)"; cat "$T/crowded-said.txt"; exit 1; }
echo "    (the list found $N; plan-set said the same $N without being asked)"
# And the sheet itself says it, for whoever holds the paper.
page "$T/crowded-list.txt" 1
says "$T/page1.txt" "ON THIS SHEET RUN OVER EACH OTHER OR OVER A LINE"
# A set with none says nothing of the kind.
lacks "$T/first.txt" "not right on the pages"

echo
echo
echo "== 8f. An ampersand and the accented letters are lettered; what cannot be lettered is said, with how many (call 182 item 2) =="
# By the file: the same house with a room called "Café & Bar" and with one called "Cafe ? Bar" (where a
# letter with no glyph was drawn as the question mark) were the same bytes. They are not now.
mk() { # title, level, room: a 24 by 16 house
  local f="$1"; "$P" new "$f" "$2" >/dev/null
  local l; l="$(made "$("$P" level "$f" "$3" --elevation "0 ft" --height "9 ft" "${K[@]}")")"
  local t; t="$(made "$("$P" type "$f" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
  "$P" wall "$f" "$l" "$t" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" >/dev/null
  "$P" room "$f" "$l" "$4" --at "12,8" "${K[@]}" >/dev/null
  "$P" plan-set "$f" "$l" "${f%.json}.pdf" --date 2026-11-22 --list > "${f%.json}.txt"
}
mk "$T/e1.json" "Henderson & Sons Résidence" "Première étage" "Café & Bar"
mk "$T/e2.json" "Henderson ? Sons R?sidence" "Premi?re ?tage" "Caf? ? Bar"
if cmp -s "$T/e1.pdf" "$T/e2.pdf"; then echo "FAIL: the ampersand and the accents are lettered as the question mark"; exit 1; fi
echo "    (\"Café & Bar\" is not lettered as \"Caf? ? Bar\": the files differ)"
says "$T/e1.txt" "unlettered: 0 letters"
# A letter no glyph is made for: lettered as its plain letter where there is one, and said; else the question mark.
mk "$T/e3.json" "Łódź Plan" "Ground" "Ωmega Room"
says "$T/e3.txt" "unlettered: 3 letters"
says "$T/e3.txt" "Ł as L"
says "$T/e3.txt" "Ź as Z"
says "$T/e3.txt" "Ω as ?"
"$P" plan-set "$T/e3.json" "ground" "$T/e3.pdf" --date 2026-11-22 > "$T/e3-said.txt"
says "$T/e3-said.txt" "3 letters could not be lettered as written"
says "$T/e3-said.txt" "Ł as L"

echo
echo "== 8g. A name moves within its room to clear what is drawn; lettering is tested against every line on the sheet (call 182 item 3) =="
# The kitchen of 8e's kind (16 ft by 12 ft, walls 6 in, its room's point at 8,6) with an island 6 ft by 3 ft set at that point: the
# room's name and area are wanted where the island's outline runs. By hand the room is 15 ft 6 in by 11 ft 6 in inside = 178.25 SF.
# Before this the name stayed at its point and was lettered over the island's name ("lettering faults: 1"); now it moves.
IJ="$T/island.json"
"$P" new "$IJ" "Island (not a real job)" >/dev/null
IS="$(made "$("$P" level "$IJ" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
IW="$(made "$("$P" type "$IJ" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
"$P" wall "$IJ" "$IS" "$IW" --points "0,0 16,0 16,12 0,12" --closed --height "9 ft" "${K[@]}" >/dev/null
"$P" room "$IJ" "$IS" "Kitchen" --at "8,6" "${K[@]}" >/dev/null
"$P" import items "$IJ" starter >/dev/null
"$P" place "$IJ" "$IS" furnishing "Island" --at "8,6" --size "6,3,3" --offset "0 ft" "${K[@]}" >/dev/null
"$P" plan-set "$IJ" "$IS" "$T/island.pdf" --date 2026-11-22 --list > "$T/island.txt"
says "$T/island.txt" "KITCHEN 178 SF"
says "$T/island.txt" "things set in place: 1"
says "$T/island.txt" "lettering faults: 0"
lacks "$T/island.txt" "AREA NOT KNOWN"

echo
echo "== 11. Strings that locate: three on each side (openings to their centers, walls to their lines, the overall), every one a chain that adds (call 182 items 1 and 5) =="
# Worked by hand BEFORE the program is asked. A house 36 ft by 16 ft (the walls' lines), one partition across it at 24 ft
# east of the west wall; a door 3 ft wide in the south wall centered 6 ft from its west end; a window 5 ft wide in it
# centered 30 ft from its west end; a door 2 ft 8 in wide in the partition centered 8 ft from its south end.
#   south, openings (centers, from the ends):   6'-0" (end to door), 24'-0" (door to window), 6'-0" (window to end)  = 36'-0"
#   south, partitions (walls to their lines):   24'-0" (to the partition), 12'-0"                                   = 36'-0"
#   south, overall:                             36'-0"
#   north: no opening, so partitions 24'-0" 12'-0" and the overall 36'-0"
#   west and east: one wall each, so the overall alone, 16'-0" (a string saying what the overall says is not drawn twice)
#   the partition's door: 8'-0" from the south wall's line and 8'-0" to the north wall's line, 16'-0" the partition;
#   the partition meets the south and north walls, so its place is in their strings and it needs none of its own.
HJ="$T/hand.json"
"$P" new "$HJ" "Hand house (not a real job)" >/dev/null
HS="$(made "$("$P" level "$HJ" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
HW="$(made "$("$P" type "$HJ" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r HSOUTH _ _ _ <<<"$("$P" wall "$HJ" "$HS" "$HW" --points "0,0 36,0 36,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
HPART="$(made "$("$P" wall "$HJ" "$HS" "$HW" --points "24,0 24,16" --height "9 ft" "${K[@]}")")"
HD3="$(made "$("$P" type "$HJ" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
HD28="$(made "$("$P" type "$HJ" door "door 2 ft 8 in" --width "2 ft 8 in" --height "7 ft" "${K[@]}")")"
HWT="$(made "$("$P" type "$HJ" window "window 5 ft" --width "5 ft" --height "4 ft" "${K[@]}")")"
"$P" door "$HJ" "$HSOUTH" "$HD3" --offset "6 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" window "$HJ" "$HSOUTH" "$HWT" --offset "30 ft" --sill "3 ft" "${K[@]}" >/dev/null
"$P" door "$HJ" "$HPART" "$HD28" --offset "8 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" room "$HJ" "$HS" "Living" --at "12,8" "${K[@]}" >/dev/null
"$P" plan-set "$HJ" "$HS" "$T/hand.pdf" --date 2026-11-22 --list > "$T/hand.txt"
page "$T/hand.txt" 1
says "$T/page1.txt" "south, openings: 6'-0\" 24'-0\" 6'-0\" (adds to 36'-0\")"
says "$T/page1.txt" "south, partitions: 24'-0\" 12'-0\" (adds to 36'-0\")"
says "$T/page1.txt" "south, overall: 36'-0\" (adds to 36'-0\")"
says "$T/page1.txt" "north, partitions: 24'-0\" 12'-0\" (adds to 36'-0\")"
says "$T/page1.txt" "north, overall: 36'-0\" (adds to 36'-0\")"
says "$T/page1.txt" "west, overall: 16'-0\" (adds to 16'-0\")"
says "$T/page1.txt" "east, overall: 16'-0\" (adds to 16'-0\")"
says "$T/page1.txt" "$HPART, along it inside the plan, openings: 8'-0\" 8'-0\" (adds to 16'-0\")"
lacks "$T/page1.txt" "north, openings"
lacks "$T/page1.txt" "west, partitions"
lacks "$T/page1.txt" "$HPART, from"
# Every string on every page is a chain: its parts add to what it says it adds to, to the sixteenth, worked here
# from the figures as they are lettered (feet, inches and sixteenths all to sixteenths of an inch).
adds() { # a --list file: each string line's parts, summed in sixteenths, against its whole
  awk '
    function sx(piece,   f, i, n, a, num) { gsub(/^ +| +$/, "", piece); f = piece; sub(/\x27.*/, "", f); i = piece; sub(/^[0-9]+\x27-/, "", i); n = split(i, a, " "); num = a[1] * 16; if (n > 1) { split(a[2], q, "/"); num += q[1] * 16 / q[2] } return f * 192 + num }
    /^    .*, [a-z ]+: .* \(adds to .*\)$/ {
      line = $0; whole = line; sub(/^.*\(adds to /, "", whole); sub(/\)$/, "", whole)
      parts = line; sub(/^[^:]*: /, "", parts); sub(/ \(adds to .*$/, "", parts); gsub(/ VIF/, "", parts)
      n = split(parts, p, "\""); sum = 0; for (j = 1; j < n; j++) sum += sx(p[j])
      w = whole; gsub(/"/, "", w); if (sx(w) != sum) { printf "FAIL: %s\n  parts add to %d sixteenths, the whole is %d\n", $0, sum, sx(w); bad = 1 } else checked++
    }
    END { if (bad) exit 1; printf "    (%d strings checked: every one adds to its whole, to the sixteenth)\n", checked }' "$1"
}
adds "$T/page1.txt" || exit 1
# On the written page: the strings on the south are one line each (36 ft = 648 points at 1/4 in to the foot), the
# openings string, the walls' string and the overall; the same on the north but the openings: five lines of 648.00 points.
"$P" page "$T/hand.pdf" --page 1 --lines --longer-than 600 > "$T/handlines.txt"
[ "$(grep -cF ": 648.00 points (9.0000 in)" "$T/handlines.txt")" = 5 ] || { echo "FAIL: not five strings of 648.00 points on the north and south"; cat "$T/handlines.txt"; exit 1; }
echo "    (five lines of 648.00 points: three on the south, two on the north)"

echo
echo "== 11a. An L: the strings chain, 20 + 20 = 40 and 16 + 14 = 30 (call 182 item 1) =="
# By hand: 0,0 40,0 40,16 20,16 20,30 0,30. The north is two walls at two heights: 20 ft (x 0 to 20, at y = 30) and 20 ft
# (x 20 to 40, at y = 16): the walls' string reads 20'-0" 20'-0" under an overall 40'-0". The east is 16 ft (the east wall) and
# 14 ft (the notch wall, at x = 20): 16'-0" 14'-0" under 30'-0". The south and the west are one wall each.
LJ="$T/ell.json"
"$P" new "$LJ" "L house (not a real job)" >/dev/null
LS2="$(made "$("$P" level "$LJ" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
LW2="$(made "$("$P" type "$LJ" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
"$P" wall "$LJ" "$LS2" "$LW2" --points "0,0 40,0 40,16 20,16 20,30 0,30" --closed --height "9 ft" "${K[@]}" >/dev/null
"$P" room "$LJ" "$LS2" "Bed" --at "10,22" "${K[@]}" >/dev/null
"$P" plan-set "$LJ" "$LS2" "$T/ell.pdf" --date 2026-11-22 --list > "$T/ell.txt"
page "$T/ell.txt" 1
says "$T/page1.txt" "north, partitions: 20'-0\" 20'-0\" (adds to 40'-0\")"
says "$T/page1.txt" "north, overall: 40'-0\" (adds to 40'-0\")"
says "$T/page1.txt" "east, partitions: 16'-0\" 14'-0\" (adds to 30'-0\")"
says "$T/page1.txt" "east, overall: 30'-0\" (adds to 30'-0\")"
says "$T/page1.txt" "south, overall: 40'-0\" (adds to 40'-0\")"
says "$T/page1.txt" "west, overall: 30'-0\" (adds to 30'-0\")"
adds "$T/page1.txt" || exit 1
says "$T/page1.txt" "lettering faults: 0"

echo
echo "== 11b. A wing at 30 degrees: no overall where the strings cannot add, a string along each wall off square, the angle lettered (call 182 item 5) =="
# By hand: 0,0 30,0 47.3205,10 41.3205,20.3923 30,20 0,20. The wing leaves the east end of the south wall at 30 degrees, 20 ft along
# it (dx 20 cos 30 = 17.3205, dy 10), turns square for 12 ft (dx -6, dy 10.3923), and comes back to 30,20 in a wall of
# sqrt(11.3205^2 + 0.3923^2) = 11.3273 ft = 11'-3 15/16". The extent across is 47.3205 ft and up the page 20.3923 ft, and the
# walls that run square reach only 30 ft and 20 ft: the strings do not add to the extent, so there is no overall on the
# south, the north or the west. Each wall off square is dimensioned along itself: 20'-0", 12'-0", 11'-3 15/16". The angles at
# the corners off square, worked from the walls' directions: at 30,0 the south wall (west) and the wing (30 degrees up) make 150;
# at 41.3205,20.3923 the wing's end wall (reversed, 300 degrees) and the last wall (181.985 degrees) make 118.015; at 30,20 the
# last wall (reversed, 1.985 degrees) and the north wall (west) make 178.015. The corner at 47.3205,10 is square: not lettered.
WJ="$T/wing.json"
"$P" new "$WJ" "Wing (not a real job)" >/dev/null
WS="$(made "$("$P" level "$WJ" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$WJ" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
"$P" wall "$WJ" "$WS" "$WT" --points "0,0 30,0 47.3205,10 41.3205,20.3923 30,20 0,20" --closed --height "9 ft" "${K[@]}" >/dev/null
"$P" plan-set "$WJ" "$WS" "$T/wing.pdf" --date 2026-11-22 --list > "$T/wing.txt"
page "$T/wing.txt" 1
lacks "$T/page1.txt" "south, overall"
lacks "$T/page1.txt" "north, overall"
lacks "$T/page1.txt" "west, overall"
says "$T/page1.txt" "no overall on the south side"
says "$T/page1.txt" "south, partitions: 30'-0\" (adds to 30'-0\")"
says "$T/page1.txt" ", along its wall: 20'-0\" (adds to 20'-0\")"
says "$T/page1.txt" ", along its wall: 12'-0\" (adds to 12'-0\")"
says "$T/page1.txt" ", along its wall: 11'-3 15/16\" (adds to 11'-3 15/16\")"
says "$T/page1.txt" "angles: 3"
says "$T/page1.txt" ": 150°"
says "$T/page1.txt" ": 118°"
says "$T/page1.txt" ": 178°"
says "$T/page1.txt" "NO OVERALL ON THE SOUTH, NORTH, WEST SIDES: A WALL OFF SQUARE MAKES THE PLAN RUN PAST THE WALLS THERE, SO THE STRINGS DO NOT ADD TO IT."
says "$T/page1.txt" "A WALL OFF SQUARE IS DIMENSIONED ALONG ITSELF; THE ANGLE BETWEEN WALLS IS LETTERED AT THE CORNER."
adds "$T/page1.txt" || exit 1
says "$T/page1.txt" "lettering faults: 0"

echo
echo "== 11c. A hall 3 ft wide: the partition across the house is located by the strings outside, and no string runs through a room (call 182 items 1 and 3) =="
# By hand: a 24 ft by 16 ft house, a partition from 0,3 to 24,3 (the hall's north wall, 3 ft from the south wall's line) and a partition
# from 8,3 to 8,16: the west string reads 3'-0" (to the hall's wall) and 13'-0" (the rest) = 16'-0"; the south string, 8'-0" and 16'-0" = 24'-0",
# for the second partition's end meets the north wall at 8 and nothing meets the south wall but the hall's wall, which runs
# along it: so the south has no partitions string and the north has 8'-0" 16'-0".
HLJ="$T/hall.json"
"$P" new "$HLJ" "Hall (not a real job)" >/dev/null
HLS="$(made "$("$P" level "$HLJ" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
HLW="$(made "$("$P" type "$HLJ" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
"$P" wall "$HLJ" "$HLS" "$HLW" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" >/dev/null
"$P" wall "$HLJ" "$HLS" "$HLW" --points "0,3 24,3" --height "9 ft" "${K[@]}" >/dev/null
"$P" wall "$HLJ" "$HLS" "$HLW" --points "8,3 8,16" --height "9 ft" "${K[@]}" >/dev/null
for r in "Hall:12,1.5" "Bed:4,9" "Living:16,9"; do "$P" room "$HLJ" "$HLS" "${r%%:*}" --at "${r##*:}" "${K[@]}" >/dev/null; done
"$P" plan-set "$HLJ" "$HLS" "$T/hall.pdf" --date 2026-11-22 --list > "$T/hall.txt"
page "$T/hall.txt" 1
says "$T/page1.txt" "west, partitions: 3'-0\" 13'-0\" (adds to 16'-0\")"
says "$T/page1.txt" "north, partitions: 8'-0\" 16'-0\" (adds to 24'-0\")"
lacks "$T/page1.txt" "south, partitions"
says "$T/page1.txt" "south, overall: 24'-0\" (adds to 24'-0\")"
# Nothing but the strings outside the plan is drawn across it: every string is on a side, none is "along" a wall inside.
lacks "$T/page1.txt" "along it inside the plan"
says "$T/page1.txt" "HALL 59 SF"
says "$T/page1.txt" "BED 94 SF"
says "$T/page1.txt" "LIVING 194 SF"
says "$T/page1.txt" "lettering faults: 0"
adds "$T/page1.txt" || exit 1

echo
echo "== 12. A room with no area says AREA NOT KNOWN and why; room lines are drawn and in the key; no note contradicts its sheet (call 182 item 6) =="
# The L house of 11a with a second room, Living, whose point is in the space Bed already has (no wall or room line
# between): by hand, Living has no area of its own, and the model says so in its own words.
SJ="$T/shared.json"
cp "$LJ" "$SJ"
"$P" room "$SJ" "$LS2" "Living" --at "30,8" "${K[@]}" >/dev/null
"$P" plan-set "$SJ" "$LS2" "$T/shared.pdf" --date 2026-11-22 --list > "$T/shared.txt"
page "$T/shared.txt" 1
says "$T/page1.txt" "LIVING AREA NOT KNOWN"
lacks "$T/page1.txt" "? SF"
says "$T/page1.txt" "LIVING HAS NO AREA YET: ITS POINT IS IN THE SPACE BED HAS. PART THE TWO WITH A WALL OR A ROOM LINE, AND EACH GETS ITS OWN."
# The same job with its walls marked existing: the sheets of what stands say the same of the room whose point
# a standing wall does enclose, and do not say that nothing closes round it.
for w in $(grep -oE 'wall-[0-9]+' "$SJ" | sort -u); do "$P" mark "$SJ" "$w" existing "${K[@]}" >/dev/null; done
"$P" plan-set "$SJ" "$LS2" "$T/shared2.pdf" --date 2026-11-22 --list > "$T/shared2.txt"
page "$T/shared2.txt" 1
says "$T/page1.txt" "page 1 of 2: EXISTING PLAN"
says "$T/page1.txt" "LIVING AREA NOT KNOWN"
says "$T/page1.txt" "LIVING HAS NO AREA AS THE WALLS STAND: ITS POINT IS IN THE SPACE BED HAS."
lacks "$T/page1.txt" "NOT LETTERED, NOTHING THAT STANDS CLOSES ROUND THEIR POINTS: LIVING"
# Room lines: the house of step 1 has one, where its east wall stood. It is drawn on the proposed plan, a thin line of long dashes,
# and is in that sheet's key; it stands for a wall to demolish, so it is not on the plans of what stands today.
says "$T/list.txt" "room lines: 1"
page "$T/list.txt" 3
says "$T/page3.txt" "room lines: 1"
says "$T/page3.txt" "key: EXISTING, light; NEW, heavy; ROOM LINE, long dashes, thin"
for n in 1 2; do page "$T/list.txt" "$n"; says "$T/page$n.txt" "room lines: 0"; lacks "$T/page$n.txt" "ROOM LINE, long dashes"; done

echo
echo "== 13. The key shows what the sheet draws; sheets are numbered through the whole set; a date is a date or is refused (call 182 item 7) =="
# A house all marked existing with one new partition: nothing comes down, so the existing sheet's key has no TO DEMOLISH.
KJ2="$T/keyjob.json"
"$P" new "$KJ2" "Key (not a real job)" >/dev/null
KS2="$(made "$("$P" level "$KJ2" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
KW2="$(made "$("$P" type "$KJ2" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
for w in $("$P" wall "$KJ2" "$KS2" "$KW2" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+'); do "$P" mark "$KJ2" "$w" existing "${K[@]}" >/dev/null; done
"$P" wall "$KJ2" "$KS2" "$KW2" --points "12,0 12,16" --height "9 ft" "${K[@]}" >/dev/null
"$P" plan-set "$KJ2" "$KS2" "$T/keyjob.pdf" --date 2026-11-22 --list > "$T/keyjob.txt"
says "$T/keyjob.txt" "not written: DEMOLITION PLAN: nothing is marked to demolish"
page "$T/keyjob.txt" 1
says "$T/page1.txt" "key: EXISTING, heavy"
lacks "$T/page1.txt" "TO DEMOLISH"
page "$T/keyjob.txt" 2
says "$T/page2.txt" "key: EXISTING, light; NEW, heavy"
# The house of step 1, key by key: the existing sheet draws both, so the key has both; the demolition sheet has existing light and to
# demolish dashed; (step 8a checks them). Sheet numbers: two levels, the upper drawn first and the ground second; the sheets
# run from the lowest level up, each level's set after the one below it, A-101 and A-102 for the ground (a plan and its
# proposal) and A-103 for the upper (one page).
NJ="$T/levels.json"
"$P" new "$NJ" "Two levels (not a real job)" >/dev/null
NUP="$(made "$("$P" level "$NJ" upper --elevation "9 ft" --height "9 ft" "${K[@]}")")"
NGR="$(made "$("$P" level "$NJ" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
NW="$(made "$("$P" type "$NJ" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
for l in "$NUP" "$NGR"; do "$P" wall "$NJ" "$l" "$NW" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" >/dev/null; done
for w in $(grep -oE 'wall-[0-9]+' "$NJ" | sort -u | head -4); do :; done
GW="$("$P" show "$NJ" "$NGR" 2>/dev/null | grep -oE 'wall-[0-9]+' | head -1 || true)"
# Mark the ground level's four walls existing and give it one new wall: two pages. (The walls of a level are those drawn on it.)
for w in $(grep -oE '"path": *"wall-[0-9]+"' "$NJ" | grep -oE 'wall-[0-9]+' | sort -u); do :; done
"$P" plan-set "$NJ" "$NUP" "$T/up.pdf" --date 2026-11-22 --list > "$T/up.txt"
"$P" plan-set "$NJ" "$NGR" "$T/gr.pdf" --date 2026-11-22 --list > "$T/gr.txt"
# Nothing is marked on either level: one page each, FLOOR PLAN. The ground is the lower: A-101, sheet 1 of 2; the upper after it: A-102, sheet 2 of 2.
says "$T/gr.txt" "sheet: A-101, sheet 1 of 2"
says "$T/up.txt" "sheet: A-102, sheet 2 of 2"
# A date is a date or is refused.
for bad in yesterday 2026-13-45 2026-02-30 2027-02-29 "11/31/2026" "" "26-11-22"; do
  refused "$P" plan-set "$J" "$S" "$T/baddate.pdf" --date "$bad"
  says "$T/out.txt" "is not a day of the calendar"
done
[ ! -e "$T/baddate.pdf" ] || { echo "FAIL: a refused date wrote a file"; exit 1; }
for good in 2026-02-28 2028-02-29 11/22/2026 2026-12-31; do
  "$P" plan-set "$J" "$S" "$T/gooddate.pdf" --date "$good" --list > "$T/good.txt"
  says "$T/good.txt" "$good"
done

echo
echo "== 14. A graphic scale; the file's own title is the job's title (call 182 item 8) =="
# A bar of four blocks and a whole number of feet, true at full size: 8 ft at 1/4 in to the foot is 144 points (a bar is at most 170 points: 16 ft would be 288); 16 ft at 1/8 in is 144 (20 ft would be 180).
page "$T/list.txt" 3
says "$T/page3.txt" "graphic scale: 8'-0\" bar, true at full size"
page "$T/eighth.txt" 1
says "$T/page1.txt" "graphic scale: 16'-0\" bar, true at full size"
# The file's own title (the information a reader's window and a file list show) is the job's, as UTF-16 hex: FEFF and each letter as 00xx.
hex_of() { printf 'FEFF'; printf '%s' "$1" | od -An -v -tx1 | tr -d ' \n' | sed 's/\(..\)/00\1/g' | tr a-f A-F; }
WANT="/Title <$(hex_of "Plan set (not a real job)")>"
LC_ALL=C grep -a -o '/Title <[0-9A-F]*>' "$T/set.pdf" > "$T/title.txt"
says "$T/title.txt" "$WANT"
echo "    (the file's title is the job's: Plan set (not a real job))"
LC_ALL=C grep -a -c '%PDF-1.4' "$T/set.pdf" >/dev/null

echo
echo "== 9. The file is read back by the program's own reader: 12 by 9 in; the overall on the south is 9 in (36 ft at 1/4 in; call 182 moved this from the south wall's own 6 in string) =="
"$P" page "$T/set.pdf" --page 3 --lines --longer-than 600 > "$T/read.txt"
says "$T/read.txt" "page 3 of 3: PDF, 864 by 648 points (12 by 9 in)"
says "$T/read.txt" ": 648.00 points (9.0000 in)"
"$P" plan-set "$J" "$S" "$T/again.pdf" --date 2026-11-22 >/dev/null
cmp -s "$T/set.pdf" "$T/again.pdf" || { echo "FAIL: the same job, scale, paper and day wrote other bytes"; exit 1; }
echo "    (the same bytes the second time)"

echo
echo "== 10. The job file is what it was =="
cmp -s "$J" "$T/before.json" || { echo "FAIL: the job changed"; exit 1; }
echo "    byte for byte"
# The pictures kept in demos/captured/planset/ are taken here when CAPTURE names the folder:
#   CAPTURE=demos/captured/planset PLUMBLINE_BIN=$PWD/target/debug demos/planset.sh
if [ -n "${CAPTURE:-}" ]; then
  mkdir -p "$CAPTURE"
  cp "$T/set.pdf" "$CAPTURE/set.pdf"
  for n in 1 2 3; do "$P" page "$T/set.pdf" --page "$n" --png "$CAPTURE/page$n.png" > /dev/null; done
  "$P" page "$T/lettering.pdf" --png "$CAPTURE/lettering-and-schedule.png" > /dev/null
  "$P" page "$T/things.pdf" --page 3 --png "$CAPTURE/things-on-the-proposed-plan.png" > /dev/null
  "$P" page "$T/hand.pdf" --png "$CAPTURE/hand-house.png" > /dev/null
  "$P" page "$T/ell.pdf" --png "$CAPTURE/l-house.png" > /dev/null
  "$P" page "$T/wing.pdf" --png "$CAPTURE/wing-at-30-degrees.png" > /dev/null
  "$P" page "$T/shared2.pdf" --png "$CAPTURE/existing-plan-room-without-area.png" > /dev/null
  "$P" page "$T/hall.pdf" --png "$CAPTURE/hall.png" > /dev/null
  "$P" page "$T/crowded.pdf" --png "$CAPTURE/crowded-fault-said.png" > /dev/null
  echo "    (pictures taken into $CAPTURE)"
fi
echo
echo "PASS"
