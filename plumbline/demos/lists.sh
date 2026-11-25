#!/usr/bin/env bash
# Lists by what is built, kept and taken down, and by trade (call 158). From an empty
# job, a renovation: the takeoff gives each count three ways (to build, existing to
# remain, to demolish) and they add up to the line; the subcontractor's list says what
# each line's thing is to the work, its division and its system, and is cut to one trade
# or one system; a job with no mark takes off as it did before.
# Run from the repository root:  demos/lists.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by
# hand in STATUS.md ("Lists by what is built, kept and taken down, and by trade").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
N=(--status Assumed --source "made-up addition plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
# The rows of a list, one to a line, whatever the cells hold.
rows() { python3 -I -c 'import csv,sys; [print("|".join(r)) for r in csv.reader(open(sys.argv[1]))]' "$1"; }

echo "== 1. The house as it stands, marked, and an addition: the walls three ways =="
"$P" new "$J" "Lists (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" mark "$J" "$SOUTH" "$EAST" "$NORTH" "$WEST" "$PART" existing "${K[@]}" >/dev/null
"$P" mark "$J" "$PART" "$EAST" demolish "${K[@]}" >/dev/null
read -r NEW1 NEW2 NEW3 <<<"$("$P" wall "$J" "$S" "$WT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${N[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" takeoff "$J" > "$T/take.txt"
# 24 + 16 + 24 + 16 + 16 + 12 + 16 + 12 = 136 LF as it stands; new 12 + 16 + 12 = 40;
# existing to remain the south, north and west walls, 24 + 24 + 16 = 64; to demolish the
# east wall and the partition, 16 + 16 = 32; 40 + 64 + 32 = 136. Each at 9 ft: 1,224 SF,
# 360, 576, 288.
says "$T/take.txt" "~ walls 8            136.00 LF as it stands"
says "$T/take.txt" "      to build            3 walls  40.00 LF"
says "$T/take.txt" "      existing to remain  3 walls  64.00 LF"
says "$T/take.txt" "      to demolish         2 walls  32.00 LF"
says "$T/take.txt" "~ wall face, gross   1,224.00 SF as it stands"
says "$T/take.txt" "      to build            360.00 SF"
says "$T/take.txt" "      existing to remain  576.00 SF"
says "$T/take.txt" "      to demolish         288.00 SF"
lacks "$T/take.txt" "footings"

echo
echo "== 2. Footings: a strip under each new wall, and the one under the south wall as it stands =="
FT="$(made "$("$P" type "$J" footing "footing 24 in" --width "24 in" "${K[@]}")")"
"$P" footing "$J" "$S" "$FT" --points "24,0 36,0 36,16 24,16" --depth "12 in" "${N[@]}" >/dev/null
OLDF="$("$P" footing "$J" "$S" "$FT" --points "0,0 24,0" --depth "12 in" "${K[@]}" | grep -oE 'footing-[0-9]+' | head -1)"
"$P" mark "$J" "$OLDF" existing "${K[@]}" >/dev/null
"$P" takeoff "$J" > "$T/take.txt"
"$P" show "$J" > "$T/show.txt"
# 12 + 16 + 12 = 40 LF new, 24 LF existing, 64 as it stands. The concrete: 40 x 2 x 1 = 80 CF,
# 2.963 CY to build; 24 x 2 x 1 = 48 CF, 1.778 CY existing; 4.741 CY as it stands. The new
# footings' concrete is checked against the program's own `show`, one footing at a time.
says "$T/take.txt" "~ footings 4         64.00 LF as it stands"
says "$T/take.txt" "      to build            3 footings  40.00 LF"
says "$T/take.txt" "      existing to remain  1 footing  24.00 LF"
says "$T/take.txt" "      to demolish         none"
says "$T/take.txt" "~ footing concrete   4.74 CY as it stands"
says "$T/take.txt" "      to build            2.96 CY"
says "$T/take.txt" "      existing to remain  1.78 CY"
python3 -I - "$T/show.txt" "$T/take.txt" <<'PY'
import re, sys
show = open(sys.argv[1]).read()
blocks = re.split(r"\n(?=    footing-)", show)
vol = {}
for b in blocks:
    m = re.match(r"\s*(footing-\d+)", b)
    v = re.search(r"volume\s+([\d.]+) CY", b)
    if m and v:
        vol[m.group(1)] = (float(v.group(1)), "existing to remain" in b)
new = round(sum(v for v, old in vol.values() if not old), 4)
old = round(sum(v for v, old in vol.values() if old), 4)
assert abs(new - 2.963) < 0.00015 and abs(old - 1.7778) < 0.00015, (new, old)
print("    the program's own show: new %.4f CY, existing %.4f CY, together %.4f; the three ways add up to the line" % (new, old, new + old))
PY
cp "$T/take.txt" "$T/take-before-lines.txt"

echo
echo "== 3. Lines on the new walls, the partition and two runs; the subcontractor's list =="
for w in "$NEW1" "$NEW2" "$NEW3"; do "$P" line "$J" "$w" "Wall, framed" face_area --cost-type subcontract --division 6 >/dev/null; done
"$P" line "$J" "$PART" "Demolish partition, haul off" face_area --cost-type subcontract --division 2 >/dev/null
PLUMB="$(made "$("$P" system "$J" "Plumbing" "Domestic Cold Water")")"
AIR="$(made "$("$P" system "$J" "Supply air" "Supply Air")")"
PT="$(made "$("$P" type "$J" pipe "pipe 1 in" --round "1 in" "${K[@]}")")"
UT="$(made "$("$P" type "$J" duct "8 in round" --round "8 in" "${K[@]}")")"
PIPE="$("$P" run "$J" "$S" pipe "$PT" --points "2,2 10,2" --height "8 ft" --system "$PLUMB" "${N[@]}" | grep -oE 'pipe-[0-9]+' | head -1)"
DUCT="$("$P" run "$J" "$S" duct "$UT" --points "2,4 10,4" --height "8 ft" --system "$AIR" "${N[@]}" | grep -oE 'duct-[0-9]+' | head -1)"
"$P" line "$J" "$PIPE" "Piping, installed" true_length --cost-type subcontract --division 15 >/dev/null
"$P" line "$J" "$DUCT" "Ductwork, installed" true_length --cost-type subcontract --division 15 >/dev/null
"$P" check "$J" >/dev/null
"$P" export subcontractor "$J" "$T/sub.csv" >/dev/null
rows "$T/sub.csv" > "$T/sub.txt"
# The face of a wall is its length by 9 ft: 12 x 9 = 108, 16 x 9 = 144, 12 x 9 = 108; the
# partition 16 x 9 = 144 SF; the pipe and the duct 8 LF each.
says "$T/sub.txt" "Where|Scope|Quantity|Unit|How sure|Note|Work|Division|System"
[ "$(grep -c '|new work|6 Wood and Plastics|$' "$T/sub.txt")" = 3 ] || { echo "FAIL: three new walls' lines are not each new work, Division 6"; cat "$T/sub.txt"; exit 1; }
echo "    the three walls' lines: new work, 6 Wood and Plastics, on no system"
says "$T/sub.txt" "Demolish partition, haul off|144|SF|Known||to demolish|2 Site Construction|"
says "$T/sub.txt" "Piping, installed|8|LF|Assumed||new work|15 Mechanical|Plumbing"
says "$T/sub.txt" "Ductwork, installed|8|LF|Assumed||new work|15 Mechanical|Supply air"
lacks "$T/sub.txt" "other lines"
"$P" export subcontractor "$J" "$T/sub15.csv" --division 15 >/dev/null
rows "$T/sub15.csv" > "$T/sub15.txt"
says "$T/sub15.txt" "Piping, installed|8|LF|Assumed||new work|15 Mechanical|Plumbing"
says "$T/sub15.txt" "Ductwork, installed|8|LF|Assumed||new work|15 Mechanical|Supply air"
lacks "$T/sub15.txt" "Wall, framed"
lacks "$T/sub15.txt" "Demolish partition"
says "$T/sub15.txt" "4 other lines, in other divisions, are not in this list"
"$P" export subcontractor "$J" "$T/subp.csv" --system "Plumbing" >/dev/null
rows "$T/subp.csv" > "$T/subp.txt"
says "$T/subp.txt" "Piping, installed|8|LF|Assumed||new work|15 Mechanical|Plumbing"
lacks "$T/subp.txt" "Ductwork"
says "$T/subp.txt" "5 other lines, on other systems or on none, are not in this list"
refused "$P" export subcontractor "$J" "$T/x.csv" --division 17
says "$T/out.txt" "is not a division: there are sixteen, 1 to 16"
refused "$P" export subcontractor "$J" "$T/x.csv" --system "Gas"
says "$T/out.txt" "no system called \"Gas\": this job has \"Plumbing\", \"Supply air\""
[ ! -e "$T/x.csv" ] || { echo "FAIL: a refused list was written"; exit 1; }
# The workbook carries the same columns and the same rows.
"$P" export subcontractor "$J" "$T/sub15.xlsx" --division 15 >/dev/null
python3 -I - "$T/sub15.xlsx" <<'PY'
import sys, openpyxl
ws = openpyxl.load_workbook(sys.argv[1]).active
rows = [[("" if c is None else str(c)) for c in r] for r in ws.iter_rows(values_only=True)]
head = next(i for i, r in enumerate(rows) if r[:3] == ["Where", "Scope", "Quantity"])
assert rows[head][6:9] == ["Work", "Division", "System"], rows[head]
assert [r[1] for r in rows[head + 1:head + 3]] == ["Piping, installed", "Ductwork, installed"], rows
assert any("4 other lines, in other divisions, are not in this list" in c for r in rows for c in r)
print("    the workbook: the same columns, the same two lines and the same last line")
PY
"$P" export supplier "$J" "$T/supplier.csv" >/dev/null
says <(rows "$T/supplier.csv") "What to quote|Quantity|Unit|Lines|How sure|Note|Work"
"$P" export pm "$J" "$T/pm.csv" >/dev/null
rows "$T/pm.csv" > "$T/pm.txt"
says "$T/pm.txt" "Element|Line|Cost type|Code|Quantity|Unit|Hours|Amount|How sure|Note|Division|Work"
says "$T/pm.txt" "Demolish partition, haul off|Subcontract||144|SF|||Unknown|the line has no price (no price entered)|2 Site Construction|to demolish"
"$P" takeoff "$J" > "$T/take.txt"
says "$T/take.txt" "Plumbing  (Domestic Cold Water; 1 straight piece, ~8.00 LF)"
lacks "$T/take.txt" "Plumbing  (Domestic Cold Water; 1 straight piece, ~8.00 LF) as it stands"

echo
echo "== 4. A job with no mark takes off as it did before this call (but for the two lines call 205 adds under each level, saying nothing is set down): the house of demos/library.sh =="
H="$T/house.json"
(
KH=(--status Known --source "made-up plan")
"$P" new "$H" "From the library (not a real job)" >/dev/null
S="$(made "$("$P" level "$H" ground --elevation "0 ft" --height "9 ft" "${KH[@]}")")"
WT="$(made "$("$P" type "$H" wall "wall 6 in" --thickness "6 in" "${KH[@]}")")"
FT="$(made "$("$P" type "$H" floor "slab 4 in" --thickness "4 in" "${KH[@]}")")"
DT="$(made "$("$P" type "$H" door "door 3 ft" --width "3 ft" --height "7 ft" "${KH[@]}")")"
NT="$(made "$("$P" type "$H" window "window 3 ft" --width "3 ft" --height "5 ft" "${KH[@]}")")"
UT="$(made "$("$P" type "$H" duct "8 in round" --round "8 in" "${KH[@]}")")"
CT="$(made "$("$P" type "$H" cable "12-2" --round "0.5 in" "${KH[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$H" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${KH[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$H" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${KH[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
DOOR="$(made "$("$P" door "$H" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${KH[@]}")")"
WIN="$(made "$("$P" window "$H" "$SOUTH" "$NT" --offset "18 ft" --sill "2.5 ft" "${KH[@]}")")"
FLOOR="$(made "$("$P" floor "$H" "$S" "$FT" --walls "${KH[@]}")")"
KITCHEN="$(made "$("$P" room "$H" "$S" "Kitchen" --at "6,8" "${KH[@]}")")"
BATH="$(made "$("$P" room "$H" "$S" "Bath" --at "18,8" "${KH[@]}")")"
"$P" roof "$H" "Main roof" --points "0,0 24,0 24,16 0,16" --pitch 8:12 --eave-height "9 ft" "${KH[@]}" >/dev/null
AIR="$(made "$("$P" system "$H" "Supply air" "Supply Air")")"
POWER="$(made "$("$P" system "$H" "Power" "Power")")"
DUCT="$("$P" run "$H" "$S" duct "$UT" --points "2,2 10,2" --height "8 ft" --system "$AIR" "${KH[@]}" | grep -oE 'duct-[0-9]+' | head -1)"
CABLE="$("$P" run "$H" "$S" cable "$CT" --points "14,2 22,2" --height "8.5 ft" --system "$POWER" "${KH[@]}" | grep -oE 'cable-[0-9]+' | head -1)"
"$P" takeoff "$H" > "$T/house-takeoff.txt"
# The text the program before this call gave for this house, to the byte.
cat > "$T/house-before.txt" <<'TAKEOFF'
flags:  ~ Assumed    ? Unknown    (no mark) Known

ground  (level-0002)
    walls 5            96.00 LF
    wall face, gross   864.00 SF
    wall face, net     828.00 SF
    floors 1           384.00 SF
    rooms 2            356.50 SF  (the floor inside the walls)
    doors 1, windows 1
  Set down on this level:
    Nothing is set down on this level.

Main roof  (roof-0019)
    roof planes 4      461.51 SF, 4.62 SQ
    predominant pitch  8:12
    eave     4         80.00 LF
    hip      4         50.03 LF
    ridge    1         8.00 LF

Supply air  (Supply Air; 1 straight piece, 8.00 LF)
    Duct     8 in round        1 piece      8.00 LF
    joined: 1 network; 2 open ends; 0 places where a run meets another along its length with no fitting
      open: duct 1: its first end at (2, 2, 8)
      open: duct 1: its second end at (10, 2, 8)

Power  (Power; 1 straight piece, 8.00 LF)
    Cable    1/2 in round      1 piece      8.00 LF
    joined: 1 network; 2 open ends; 0 places where a run meets another along its length with no fitting
      open: cable 1: its first end at (14, 2, 8.5)
      open: cable 1: its second end at (22, 2, 8.5)
TAKEOFF
cmp -s "$T/house-takeoff.txt" "$T/house-before.txt" || { echo "FAIL: a job with no mark does not take off as it did"; diff "$T/house-before.txt" "$T/house-takeoff.txt"; exit 1; }
echo "    byte for byte the same: no split anywhere, and the line says nothing of what stands"
lacks "$T/house-takeoff.txt" "as it stands"
)
echo

echo
echo "== 5. Equipment that stays, a run taken down: a system's takeoff three ways =="
"$P" place "$J" "$S" equipment "Air handler" --at "14,10" --offset "0 ft" --size "2,2,4" --system "$AIR" "${N[@]}" >/dev/null
AH="$("$P" show "$J" | grep -oE 'equipment-[0-9]+' | head -1)"
"$P" mark "$J" "$AH" existing "${K[@]}" >/dev/null
"$P" mark "$J" "$DUCT" demolish --keep "${K[@]}" >/dev/null
"$P" takeoff "$J" > "$T/take.txt"
# One air handler, existing to remain; the duct comes down: its 8 LF are to demolish.
says "$T/take.txt" "Supply air  (Supply Air; 1 straight piece, ~8.00 LF as it stands)"
says "$T/take.txt" "      to build            none"
says "$T/take.txt" "      to demolish         1 piece  8.00 LF"
says "$T/take.txt" "as it stands"
says "$T/take.txt" "      to build none; existing to remain 1 EA; to demolish none"

echo
echo "  PASS: a renovation's takeoff gave its walls, footings and a system three ways that add up to the line, the subcontractor's list said what each line's thing is to the work, its division and its system and was cut to one trade or one system with the rest counted, and a job with no mark took off as it did before, to the figures worked by hand"
