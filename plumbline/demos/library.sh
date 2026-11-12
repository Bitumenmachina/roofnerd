#!/usr/bin/env bash
# The starter library of assemblies (library/sixteen-divisions.csv): 30 assemblies with a
# line in every one of the sixteen divisions and no price in it. It is brought into an
# empty job, its assemblies are applied to a small made-up house, the takeoff comes out
# by division with no price on anything, and three prices are typed.
# Run from the repository root:  demos/library.sh
# The job and every price are made up (no real job data) and live in a temporary folder.
# Every step fails the script if the program does not say what the step expects. The
# figures are worked by hand in the comments and in STATUS.md ("A starter library").
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet
P="$PWD/target/debug/plumbline"
LIB="$PWD/library/sixteen-divisions.csv"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$OUT" || { echo "FAIL: $1: not said: $2"; echo "$OUT"; exit 1; }; printf '    %s\n' "$2"; }

echo "== The library has no price in it =="
python3 - "$LIB" <<'PY'
import csv, re, sys
rows = list(csv.reader(open(sys.argv[1])))
head, lines = rows[0], rows[1:]
cost, division, ctype = head.index("Unit cost"), head.index("Division"), head.index("Cost type")
assert all(re.fullmatch(r"\$\?/[A-Za-z]+", r[cost]) for r in lines), "a unit cost that is not $?/unit"
assert not any(re.search(r"\$\s*\d", cell) for r in lines for cell in r), "a dollar figure in the library"
assert all(r[ctype] for r in lines) and all(r[division] for r in lines), "a line with no cost type or no division"
print("    %d assemblies, %d lines, a line in each of divisions %s; every unit cost is $?, and no dollar figure is in the file"
      % (len({r[0] for r in lines}), len(lines), ", ".join(str(d) for d in sorted({int(r[division]) for r in lines}))))
assert sorted({int(r[division]) for r in lines}) == list(range(1, 17)) and len({r[0] for r in lines}) == 30 and len(lines) == 93
PY

echo
echo "== The house: 24 ft by 16 ft, 9 ft walls, a partition, a door, a window, a floor, two rooms, a hip roof, a duct, a cable =="
"$P" new "$J" "From the library (not a real job)"
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
FT="$(made "$("$P" type "$J" floor "slab 4 in" --thickness "4 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${K[@]}")")"
UT="$(made "$("$P" type "$J" duct "8 in round" --round "8 in" "${K[@]}")")"
CT="$(made "$("$P" type "$J" cable "12-2" --round "0.5 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
DOOR="$(made "$("$P" door "$J" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${K[@]}")")"
WIN="$(made "$("$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "2.5 ft" "${K[@]}")")"
FLOOR="$(made "$("$P" floor "$J" "$S" "$FT" --walls "${K[@]}")")"
KITCHEN="$(made "$("$P" room "$J" "$S" "Kitchen" --at "6,8" "${K[@]}")")"
BATH="$(made "$("$P" room "$J" "$S" "Bath" --at "18,8" "${K[@]}")")"
"$P" roof "$J" "Main roof" --points "0,0 24,0 24,16 0,16" --pitch 8:12 --eave-height "9 ft" "${K[@]}" >/dev/null
AIR="$(made "$("$P" system "$J" "Supply air" "Supply Air")")"
POWER="$(made "$("$P" system "$J" "Power" "Power")")"
DUCT="$("$P" run "$J" "$S" duct "$UT" --points "2,2 10,2" --height "8 ft" --system "$AIR" "${K[@]}" | grep -oE 'duct-[0-9]+' | head -1)"
CABLE="$("$P" run "$J" "$S" cable "$CT" --points "14,2 22,2" --height "8.5 ft" --system "$POWER" "${K[@]}" | grep -oE 'cable-[0-9]+' | head -1)"
"$P" check "$J"
ALL="$("$P" show "$J")"
of() { grep -oE "$1-[0-9]+" <<<"$ALL" | sort -u; }

echo
echo "== The library comes in: 30 assemblies, none refused =="
OUT="$("$P" import assemblies "$J" "$LIB")"; echo "$OUT" | grep -E "rows|set:|same|not used" | sed 's/^/  /'
says "all of them" "set: 30"
says "none refused" "not used: 0"

echo
echo "== Each thing given its assembly =="
apply() { "$P" apply "$J" "$1" "$2" --source "the starter library" >/dev/null || { echo "FAIL: $2 was not taken by $1"; exit 1; }; }
for w in "$SOUTH" "$EAST" "$WEST"; do apply "$w" "Exterior wall, 2x6, lap siding"; done
apply "$NORTH" "Exterior wall, 2x6, lap siding over a stone wainscot"
apply "$PART" "Interior partition, 2x4"
apply "$FLOOR" "Slab on grade, 4 in"
apply "$KITCHEN" "Kitchen"; apply "$BATH" "Bath, tile floor"
for p in $(of plane); do apply "$p" "Asphalt shingle roof"; done
for r in $(of ridge); do apply "$r" "Ridge, steel beam and cap"; done
for h in $(of hip); do apply "$h" "Hip cap"; done
for e in $(of eave); do apply "$e" "Gutter"; done
apply "$WIN" "Window"; apply "$DOOR" "Entry door"; apply "$DUCT" "Duct run"; apply "$CABLE" "Cable run"
apply "$(grep -oE 'job-[0-9]+' <<<"$ALL" | head -1)" "General conditions"
"$P" check "$J"

echo
echo "== The takeoff, with no price on anything =="
OUT="$("$P" estimate "$J" | tr -s ' ')"
# The south wall: 24 by 9 is 216 SF; less the door, 21, and the window, 15: 180.
says "the south wall's siding" "Lap siding 180.00 SF no price"
# Its sheathing over the whole face: 216 at 32 SF to the sheet is 6.75, bought as 7.
says "its sheathing, in whole sheets" "Wall sheathing 7.00 sheet no price"
# The slab: 24 by 16 along the walls' center lines is 384 SF; at 81 SF to the cubic yard
# (27 CF at 4 in), 4.74, bought as 5.
says "the slab's concrete, in whole yards" "Concrete, 4 in slab 5.00 CY no price"
says "nothing comes to a figure" "Job cost none"
# Lines in every division but 13 and 14, which this house has nothing for.
for d in "1 General Requirements" "2 Site Construction" "3 Concrete" "4 Masonry" "5 Metals" "6 Wood and Plastics" "7 Thermal and Moisture Protection" "8 Doors and Windows" "9 Finishes" "10 Specialties" "11 Equipment" "12 Furnishings" "15 Mechanical" "16 Electrical"; do
  says "work in it and no price yet" " $d no figure"
done
says "no sauna" " 13 Special Construction none"
says "no elevator" " 14 Conveying Systems none"

echo
echo "== Three prices typed: the estimate reads by division =="
"$P" set "$J" "$SOUTH" unit_cost '$9.50/SF' --line "Lap siding" --status Known --source "made-up price" >/dev/null
"$P" set "$J" "$FLOOR" unit_cost '$165/CY' --line "Concrete, 4 in slab" --status Known --source "made-up price" >/dev/null
"$P" set "$J" "$(grep -oE 'job-[0-9]+' <<<"$ALL" | head -1)" unit_cost '$1,200' --line "Building permit" --status Known --source "made-up price" >/dev/null
OUT="$("$P" estimate "$J" | tr -s ' ')"
# 180 SF at $9.50 is $1,710.00; 5 CY at $165 is $825.00; the permit is $1,200.00.
says "the siding" "Lap siding 180.00 SF \$9.50/SF \$1,710.00"
says "the concrete" "Concrete, 4 in slab 5.00 CY \$165.00/CY \$825.00"
# Each division has its figure and is flagged (~): it still leaves lines out, and names them.
says "Division 1" " 1 General Requirements ~\$1,200.00 1 line"
says "what Division 1 leaves out" "? left out: job-0001: \"Dumpsters\": the line has no price (no price entered)"
says "Division 3" " 3 Concrete ~\$825.00 1 line"
says "Division 7" " 7 Thermal and Moisture Protection ~\$1,710.00 1 line"
# 1,200.00 + 825.00 + 1,710.00 = 3,735.00.
says "the job cost" "Job cost ~\$3,735.00 3 lines"

echo
echo "== The house is dressed from the same lines =="
OUT="$("$P" finishes "$J")"
says "the south wall" "($SOUTH): lap siding, on its south face (the line \"Lap siding\")"
says "the north wall" "($NORTH): lap siding, on its north face (the line \"Lap siding\"); stone veneer wainscot to 3' 0\" (the line \"Stone veneer wainscot, 3 ft\")"
says "the partition" "($PART): a partition: both its faces are inside"
says "a roof plane" "asphalt shingles (the line \"Asphalt shingles\")"
says "an eave" "a gutter (the line \"Gutter\")"
echo "  PASS: the starter library came into an empty job with no price in it, its assemblies gave a small house its takeoff in fourteen of the sixteen divisions, three typed prices came out by division to the figures worked by hand, and the house is dressed from the same lines"
