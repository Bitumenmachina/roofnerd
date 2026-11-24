#!/usr/bin/env bash
# A model brought in whole (call 197, points 1 to 7; contract 197I). At the command line, from an empty job: the made-up whole
# house (`plumbline sixteen`) is written out as an IFC4 model and brought into an empty job; the second job is set beside the first.
# What a first job has and a second must have: the fixtures and pieces of equipment, set down where they stood; the fittings, on
# their runs; the four strip footings; the roof (a hip, and a gable made on two edges); the names (a door, a window, a system); what
# is left out says what follows; the two small words. The model passes IfcOpenShell (an outside checker, never linked).
# Run from the repository root:  demos/ifc-whole.sh
# The job and every figure are made up (no real job data) and live in a temporary folder. Every step fails the script if the program
# does not say what the step expects. The figures are worked in STATUS.md ("a model brought in whole: a demonstration") and by hand
# in the comments here. NOT BUILT, held for a ruling (said by this script, not asserted): the first job's 39 written fittings come
# in as 37 (two refused by the change that puts a fitting where runs meet), and the open ends it had at fixtures are not
# reproduced: the model holds no port, so a run's end off a fixture's box reads open.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
A=(--status Assumed --source "made-up tracing")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
# A takeoff as words and figures only: no flags, one space between.
plain() { sed 's/^ *[~?] */  /; s/^ *//; s/  */ /g' "$1"; }

echo "== The first job: the made-up whole house, written out as a model, brought into an empty job =="
"$P" sixteen "$T/first.json" >/dev/null
"$P" export ifc "$T/first.json" "$T/first.ifc" > "$T/export.txt"
says "$T/export.txt" "22 fixtures"
says "$T/export.txt" "39 fittings"
"$P" new "$T/second.json" "Second (not a real job)" >/dev/null
"$P" import ifc "$T/second.json" "$T/first.ifc" --status Known --source "made-up model" > "$T/in.txt"
# 1. By the first job's own figures: 22 fixtures and 5 pieces of equipment, each where it stood.
echo "-- Point 1: fixtures and equipment come in --"
says "$T/in.txt" "22 fixtures"
says "$T/in.txt" "5 pieces of equipment"
# 2. Fittings: 41 in the first job, 39 written (2 have no size), 37 brought in (held: two refused where a third end meets).
echo "-- Point 2: fittings come in on their runs (HELD for a ruling: 37 of the 39, and the open ends at fixtures) --"
says "$T/in.txt" "37 fittings"
says "$T/in.txt" "1 pipe fitting: this program would not take it: "
says "$T/in.txt" "leave (16.3, 22.7) the same way, one over the other"
says "$T/in.txt" "22 runs ending at a fixture or a piece of equipment were joined to it, where it stood"
# 3. Four strip footings, 136 LF (the four outside walls' 136 LF), 136 x 2 ft x 1 ft / 27 = 10.07 CY of concrete.
echo "-- Point 3: footings --"
says "$T/in.txt" "4 footings"
"$P" takeoff "$T/first.json" > "$T/first.takeoff"
"$P" takeoff "$T/second.json" > "$T/second.takeoff"
plain "$T/second.takeoff" > "$T/s.txt"
plain "$T/first.takeoff" > "$T/f.txt"
says "$T/s.txt" "footings 4 136.00 LF"
says "$T/s.txt" "footing concrete 10.07 CY"
# 4. The roof: four planes, 1,408.72 SF (14.09 SQ); eaves 144 LF; 4 hips 90 LF; 1 ridge 12 LF; the eaves of the trim, 42, 30, 42, 30 LF at 0.75 ft.
echo "-- Point 4: the roof --"
says "$T/in.txt" "1 roof"
says "$T/in.txt" "4 roof planes"
says "$T/s.txt" "roof planes 4 1,408.72 SF, 14.09 SQ"
says "$T/s.txt" "predominant pitch 6:12"
says "$T/s.txt" "eave 4 144.00 LF"
says "$T/s.txt" "hip 4 90.00 LF"
says "$T/s.txt" "ridge 1 12.00 LF"
"$P" trim "$T/second.json" > "$T/trim.txt"
for L in 42 30; do grep -qE "eave [1-4] \[eave-[0-9]+\]: ~$L LF long; overhang ~0.75 ft" "$T/trim.txt" || { echo "FAIL: the trim does not give an eave of $L LF at 0.75 ft"; cat "$T/trim.txt"; exit 1; }; done
printf '    eaves of 42 and 30 LF at 0.75 ft, as the first job gives\n'
# 5. Names kept: the front door; the nine windows of the upper level; "Supply air" with its capitals.
echo "-- Point 5: names --"
"$P" show "$T/second.json" > "$T/show.txt"
says "$T/show.txt" "front door"
says "$T/show.txt" "west window"
says "$T/show.txt" "Supply air"
lacks "$T/show.txt" "Supply Air  ["
lacks "$T/show.txt" "door 1"
# 6. The takeoff of the two jobs differs in no figure (the flags differ: everything brought in is Assumed; and the fittings,
#    the hot water's 3 pieces of no size which are not in a model, and the two Tees are the differences said above).
echo "-- Point 6: the two takeoffs, set side by side --"
for line in "walls 8 210.00 LF" "wall face, gross 2,026.00 SF" "wall face, net 1,793.02 SF" "floors 1 1,120.00 SF" "rooms 5 1,057.95 SF (the floor inside the walls)" "doors 5, windows 8" "doors 4, windows 9" "walls 4 136.00 LF"; do
    grep -qF -- "$line" "$T/f.txt" && grep -qF -- "$line" "$T/s.txt" || { echo "FAIL: a figure differs: $line"; diff "$T/f.txt" "$T/s.txt" | head -20; exit 1; }
    printf '    both say: %s\n' "$line"
done
grep -E "^(Duct|Pipe|Cable|Conduit) " "$T/f.txt" | grep -v "size unknown" | sort > "$T/f.runs"
grep -E "^(Duct|Pipe|Cable|Conduit) " "$T/s.txt" | sort > "$T/s.runs"
grep -E "^(Fixture|Equipment) " "$T/f.txt" | sort > "$T/f.things"
grep -E "^(Fixture|Equipment) " "$T/s.txt" | sort > "$T/s.things"
cmp -s "$T/f.runs" "$T/s.runs" || { echo "FAIL: the runs' figures differ"; diff "$T/f.runs" "$T/s.runs"; exit 1; }
cmp -s "$T/f.things" "$T/s.things" || { echo "FAIL: the fixtures and equipment differ"; diff "$T/f.things" "$T/s.things"; exit 1; }
printf '    every run and every fixture and piece of equipment: the same lines in both\n'
"$P" check "$T/second.json" > "$T/check.txt" 2>&1 || { echo "FAIL: the second job does not check"; cat "$T/check.txt"; exit 1; }
printf '    the second job checks\n'
# What is still not brought in: no fixture, footing or roof in the list; and every value is Assumed, none Known on the file's say-so.
grep -E "^  not brought in" "$T/in.txt" | head -1 | grep -qF "2 things" || { echo "FAIL: not brought in is not just the two fittings"; cat "$T/in.txt"; exit 1; }
lacks "$T/in.txt" "fixture or outlet"
lacks "$T/in.txt" "footings:"
lacks "$T/in.txt" "roof slabs"
"$P" show "$T/second.json" > "$T/second.show"
lacks "$T/second.show" "front door [Known"

echo
echo "== A gable made on two edges comes back a gable =="
G="$T/gable.json"
"$P" new "$G" "Gable (not a real job)" >/dev/null
S="$(grep -oE 'level-[0-9]+' <("$P" level "$G" ground --elevation "0 ft" --height "9 ft" "${A[@]}") | head -1)"
WT="$(grep -oE 'type-[0-9]+' <("$P" type "$G" wall "wall 6 in" --thickness "6 in" "${A[@]}") | head -1)"
"$P" wall "$G" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${A[@]}" >/dev/null
R="$(grep -oE 'roof-[0-9]+' <("$P" roof "$G" "Main roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 6:12 --eave-height "9 ft" "${A[@]}") | head -1)"
"$P" set "$G" "$R" "edge:2" gable "${A[@]}" >/dev/null
"$P" set "$G" "$R" "edge:4" gable "${A[@]}" >/dev/null
"$P" export ifc "$G" "$T/gable.ifc" > "$T/gable-export.txt"
grep -qF "'Plumbline_Roof'" "$T/gable.ifc" || { echo "FAIL: the roof carries no list of what makes it again"; exit 1; }
for property in "'Outline'" "'EdgeRise'" "'EaveHeight'" "'Basis'" "IFCLABEL('gable')" "IFCLABEL('6:12')"; do
    grep -qF -- "$property" "$T/gable.ifc" || { echo "FAIL: the roof's list lacks $property"; exit 1; }
done
printf '    the roof carries Plumbline_Roof: Outline, EdgeRise (6:12, gable, 6:12, gable), EaveHeight, Basis\n'
"$P" new "$T/gable2.json" "Second gable (not a real job)" >/dev/null
"$P" import ifc "$T/gable2.json" "$T/gable.ifc" --status Assumed --source "made-up model" > "$T/gable-in.txt"
says "$T/gable-in.txt" "1 roof"
says "$T/gable-in.txt" "2 roof planes"
"$P" takeoff "$G" > "$T/g1.takeoff"; "$P" takeoff "$T/gable2.json" > "$T/g2.takeoff"
plain "$T/g1.takeoff" > "$T/g1.txt"; plain "$T/g2.takeoff" > "$T/g2.txt"
# Two planes, each 26 ft long and 9 ft on the plan to the ridge, 9 x sqrt(1.25) = 10.062 ft up the slope: 2 x 26 x 10.062 = 523.24 SF; eaves 2 x 26 = 52 LF; rakes 2 x 10.062 x 2.
for line in "roof planes 2 523.24 SF" "eave 2 52.00 LF"; do
    grep -qF -- "$line" "$T/g1.txt" && grep -qF -- "$line" "$T/g2.txt" || { echo "FAIL: the gable roof differs: $line"; diff "$T/g1.txt" "$T/g2.txt"; exit 1; }
    printf '    both say: %s\n' "$line"
done
"$P" gable "$T/gable2.json" > "$T/gable2.txt"
"$P" gable "$G" > "$T/gable1.txt"
grep -c "gable " "$T/gable1.txt" > "$T/n1"; grep -c "gable " "$T/gable2.txt" > "$T/n2"
cmp -s "$T/n1" "$T/n2" || { echo "FAIL: the gable ends differ"; cat "$T/gable1.txt" "$T/gable2.txt"; exit 1; }
printf '    the same gable ends in both\n'
# A roof slab from a file with no such list is left out and said, as before.
sed "s/'Plumbline_Roof'/'Somebody_Elses'/" "$T/gable.ifc" > "$T/other.ifc"
"$P" new "$T/other.json" "Other (not a real job)" >/dev/null
"$P" import ifc "$T/other.json" "$T/other.ifc" --status Assumed --source "made-up model" > "$T/other-in.txt"
says "$T/other-in.txt" "a roof is not brought in from a model yet"

echo
echo "== What is left out says what follows from it (point 6) =="
sed 's/=IFCSANITARYTERMINAL(/=IFCBUILDINGELEMENTPROXY(/; s/\.TOILETPAN\./.ELEMENT./; s/\.WASHHANDBASIN\./.ELEMENT./; s/\.BATH\./.ELEMENT./' "$T/first.ifc" > "$T/without.ifc"
"$P" new "$T/without.json" "Without (not a real job)" >/dev/null
"$P" import ifc "$T/without.json" "$T/without.ifc" --status Assumed --source "made-up model" > "$T/without.txt"
grep -qE "note: [0-9]+ run ends? read open here: .* stood on .*thing of no stated kind" "$T/without.txt" || { echo "FAIL: it does not say how many runs read open"; cat "$T/without.txt"; exit 1; }
printf '    %s\n' "$(grep -oE 'note: [0-9]+ run ends? read open here[^(]*' "$T/without.txt" | head -1)"

echo
echo "== The two small words (point 7) =="
"$P" gable "$T/second.json" > "$T/gable-said.txt"
says "$T/gable-said.txt" "~ none: an eave is over it"
"$P" import items "$T/second.json" starter >/dev/null
"$P" items "$T/second.json" > "$T/items.txt"
says "$T/items.txt" "drawn as an outdoor unit"
lacks "$T/items.txt" "a outdoor"

echo
echo "== The model passes IfcOpenShell (an outside checker, never linked) =="
if python3 -c "import ifcopenshell" 2>/dev/null; then
    python3 tools/ifc-check.py "$T/first.ifc" > "$T/checked.txt" || { echo "FAIL: IfcOpenShell finds a fault in the house"; cat "$T/checked.txt"; exit 1; }
    python3 tools/ifc-check.py "$T/gable.ifc" > "$T/checked-gable.txt" || { echo "FAIL: IfcOpenShell finds a fault in the gable roof"; cat "$T/checked-gable.txt"; exit 1; }
    sed 's/^/    /' "$T/checked.txt" | head -2
    sed 's/^/    /' "$T/checked-gable.txt" | head -1
else
    echo "demos/ifc-whole.sh needs IfcOpenShell to validate the model (python3 -c 'import ifcopenshell' fails; set PYTHONPATH to where it is)"; exit 2
fi
echo
echo "ifc-whole: all of it holds (points 1 to 7, but for what is held for a ruling and said above)."
