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
[ "$(grep -cF ": 648.00 points (9.0000 in)" "$T/p3lines.txt")" = 2 ] || { echo "FAIL: the overall is not drawn twice, 648.00 points"; cat "$T/p3lines.txt"; exit 1; }
echo "    (two lines of 648.00 points, the overall above and below the plan)"
"$P" page "$T/set.pdf" --page 3 --lines --longer-than 280 > "$T/p3up.txt"
[ "$(grep -cF ": 288.00 points (4.0000 in)" "$T/p3up.txt")" = 4 ] || { echo "FAIL: the strings and the overall up the page are not four lines of 288.00 points (16 ft)"; cat "$T/p3up.txt"; exit 1; }
echo "    (four lines of 288.00 points: a string and the overall on each side)"
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
echo "== 9. The file is read back by the program's own reader: 12 by 9 in; the south wall's string is 6 in =="
"$P" page "$T/set.pdf" --page 3 --lines --longer-than 400 > "$T/read.txt"
says "$T/read.txt" "page 3 of 3: PDF, 864 by 648 points (12 by 9 in)"
says "$T/read.txt" ": 432.00 points (6.0000 in)"
"$P" plan-set "$J" "$S" "$T/again.pdf" --date 2026-11-22 >/dev/null
cmp -s "$T/set.pdf" "$T/again.pdf" || { echo "FAIL: the same job, scale, paper and day wrote other bytes"; exit 1; }
echo "    (the same bytes the second time)"

echo
echo "== 10. The job file is what it was =="
cmp -s "$J" "$T/before.json" || { echo "FAIL: the job changed"; exit 1; }
echo "    byte for byte"
echo
echo "PASS"
