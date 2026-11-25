#!/usr/bin/env bash
# Another trade's model into a job that has a building (call 210, contract 210M). At the command line. The job is the made-up whole
# house (`plumbline sixteen`). The designer's model is made in this script by the program itself, so its figures are known: a job of
# its own with
#   - one level, "Level 1", its floor at 0 ft (the sample's first floor is at 0 ft), 10 ft high;
#   - four walls on the sample's outside line, 40 ft by 28 ft, 10 ft high, and one floor of the same outline: they lie over the
#     sample's own;
#   - one duct, 10 ft long, 14 in by 8 in, from (20, 20) to (30, 20), 8 ft above the floor, on a supply-air system: it goes through
#     the sample's partition at 24 (which runs (24, 16) to (24, 28), 9 ft high, with no opening in it: the duct, from 7 ft 8 in to
#     8 ft 4 in high, meets it at 24 ft along);
#   - one air handler, 2 ft by 2.5 ft by 5 ft with its middle at (30, 8), clear of every wall (the sample's walls on that floor are
#     the outside line and the three partitions north of y = 16, and the wet wall at y = 22);
# written out as a BIM model. The design as the lead first wrote it said a duct 12 ft long; from (20, 20) a 12 ft duct runs into the
# sample's own "branch to office" at x = 32 and makes a second clash, so the duct here is 10 ft, which makes the one clash the
# demonstration is about. (An air handler clear of every wall and every duct is at (30, 8).)
# By hand:
#  1. Brought in whole: not refused (a refusal comes only where nothing but levels could come in): the duct and the air handler come
#     in; the 4 walls and the floor are left out for lying over the job's own; the reply says so and says that only its runs and
#     the equipment they serve can be chosen.
#  2. Brought in as only its runs: 1 duct and 1 piece of equipment come in; 4 walls and 1 floor are counted as not asked for; the
#     reply says the model's level is the job's first floor; no level is added (the job has 3 levels before and after).
#  3. The sample's own figures do not move: its 8 + 8 + 4 walls, 15 rooms, 2 floors and its roof, and the price, $468,050.22; the
#     duct and the air handler carry no cost line, and the estimate says two more things are priced on nothing.
#  4. The two things make 1 clash with the house: a hole to cut, the duct through the partition at 24, 14.00 in by 8.00 in, at
#     (24, 20, 8). Before: 11 in the same space, 10 set against a wall, 47 holes to cut; after: 11, 10, 48; the clash list differs by
#     that one line and the two more things tested (193 to 195).
#  5. The same file again: refused, and the job is the same file byte for byte.
#  6. Everything brought in reads Assumed, and says the model and the thing in it it came from.
#  7. A level at a height the job does not have: a model with a "Mezzanine" with its floor at 4 ft and a duct 3 ft above it is put on
#     the job's first floor, 4 + 3 = 7 ft above it, and the reply says so; no level is added.
#  8. A model of walls alone, as only its runs: nothing comes in, and it says so.
# Run from the repository root:  demos/ifc-runs.sh
# Every figure is made up. Every step fails the script if the program does not say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
A=(--status Assumed --source "made-up designer")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
# The path a thing was made at, from what the program said.
made() { grep -oE "$2" "$1" | head -1; }

echo "== The designer's model, made here with the program and written out =="
D="$T/designer.json"
"$P" new "$D" "Designer (not a real job)" >/dev/null
L1="$(made <("$P" level "$D" "Level 1" --elevation "0 ft" --height "10 ft" "${A[@]}") 'level-[0-9]+')"
W6="$(made <("$P" type "$D" wall "Outside wall, 6 in" --thickness "6 in" "${A[@]}") 'type-[0-9]+')"
SL="$(made <("$P" type "$D" floor "Slab" --thickness "4 in" "${A[@]}") 'type-[0-9]+')"
"$P" wall "$D" "$L1" "$W6" --points "0,0 40,0 40,28 0,28" --closed --height "10 ft" "${A[@]}" >/dev/null
"$P" floor "$D" "$L1" "$SL" --points "0,0 40,0 40,28 0,28" "${A[@]}" >/dev/null
DT="$(made <("$P" type "$D" duct "Duct 14 by 8" --width "14 in" --depth "8 in" "${A[@]}") 'type-[0-9]+')"
SY="$(made <("$P" system "$D" "Supply air, designer" "Supply Air") 'system-[0-9]+')"
"$P" run "$D" "$L1" duct "$DT" --points "20,20 30,20" --height "8 ft" --system "$SY" "${A[@]}" >/dev/null
"$P" place "$D" "$L1" equipment "air handler" --at "30,8" --offset "0 ft" --size "2,2.5,5" "${A[@]}" >/dev/null
"$P" export ifc "$D" "$T/designer.ifc" > "$T/export.txt"
says "$T/export.txt" "1 duct, 1 floor, 1 level, 1 piece of equipment, 1 system, 4 walls"

echo "== The job: the made-up whole house, and what it was before =="
"$P" sixteen "$T/house.json" >/dev/null
cp "$T/house.json" "$T/whole.json"
cp "$T/house.json" "$T/runs.json"
"$P" estimate "$T/house.json" > "$T/est0.txt"
"$P" takeoff "$T/house.json" > "$T/take0.txt"
"$P" clashes "$T/house.json" > "$T/clash0.txt"
says "$T/est0.txt" '$468,050.22'
says "$T/clash0.txt" "In the same space: 11 ("
says "$T/clash0.txt" "Set against (stands against it, or is joined to it, by no more than 2.00 in: not a clash): 10 ("
says "$T/clash0.txt" "Through a wall or a floor (a hole to cut): 47 ("
says "$T/clash0.txt" "Tested: 193 elements"
[ "$("$P" show "$T/house.json" | grep -c '\[level\]')" = 3 ] || { echo "FAIL: the house does not have 3 levels"; exit 1; }
printf '    3 levels\n'

echo "== 1. Brought in whole: not refused; what lies over the job's own is left out, and the reply says what can be chosen =="
"$P" import ifc "$T/whole.json" "$T/designer.ifc" --status Assumed --source "designer, made up" > "$T/in-whole.txt"
says "$T/in-whole.txt" "1 duct"
says "$T/in-whole.txt" "1 piece of equipment"
lacks "$T/in-whole.txt" "1 pieces of equipment"
says "$T/in-whole.txt" "not brought in: 5 things"
[ "$(grep -c "one wall drawn over another would be counted twice" "$T/in-whole.txt")" = 4 ] || { echo "FAIL: not four walls lying over"; cat "$T/in-whole.txt"; exit 1; }
printf '    4 walls left out, each for lying over one of the job'"'"'s own\n'
says "$T/in-whole.txt" "1 floor slab: a floor with this very outline is already on this level: a second would be counted twice"
says "$T/in-whole.txt" "note: 4 walls and 1 floor of the model lie over what the job has and were left out, so as not to be counted twice; to bring a model like this in without being asked for them, choose only its runs and the equipment they serve (--only runs)"

echo "== 2. Brought in as only its runs and what they serve =="
"$P" import ifc "$T/runs.json" "$T/designer.ifc" --status Assumed --source "designer, made up" --only runs > "$T/in-runs.txt"
says "$T/in-runs.txt" "brought in, only its runs and what they serve"
says "$T/in-runs.txt" "    1 duct"
says "$T/in-runs.txt" "    1 piece of equipment"
says "$T/in-runs.txt" "not brought in: 0 things"
says "$T/in-runs.txt" "not asked for: 1 floor and 4 walls"
says "$T/in-runs.txt" "note: the model's level \"Level 1\" has its floor at 0 ft, the height of the job's \"first floor\": it is that level"
lacks "$T/in-runs.txt" "added as a new level"
[ "$("$P" show "$T/runs.json" | grep -c '\[level\]')" = 3 ] || { echo "FAIL: a level was added"; exit 1; }
printf '    still 3 levels: none added\n'

echo "== 3. The house's own figures do not move =="
"$P" estimate "$T/runs.json" > "$T/est1.txt"
"$P" takeoff "$T/runs.json" > "$T/take1.txt"
says "$T/est1.txt" '$468,050.22'
says "$T/est1.txt" "nothing is priced on 2 things that could be (1 duct, 1 piece of equipment): they are in no line and no total above"
# Every wall, floor, room and roof line of the takeoff is what it was.
for w in "walls " "wall face, gross" "wall face, net" "floors " "rooms " "roof planes"; do
  grep -E "^ *$w" "$T/take0.txt" > "$T/a.txt" || true
  grep -E "^ *$w" "$T/take1.txt" > "$T/b.txt" || true
  cmp -s "$T/a.txt" "$T/b.txt" || { echo "FAIL: '$w' lines moved"; diff "$T/a.txt" "$T/b.txt"; exit 1; }
done
printf '    walls, wall faces, floors, rooms and roof planes: the same lines in the takeoff before and after\n'
# The price of the whole-bringing-in job is the same too: the duct and the air handler carry no cost line.
"$P" estimate "$T/whole.json" | grep -q 'PRICE  *\$468,050.22' || { echo "FAIL: the price moved when brought in whole"; exit 1; }
printf '    brought in whole, the price is the same\n'

echo "== 4. The clashes the two things make =="
says "$T/in-runs.txt" "  clashes:"
says "$T/in-runs.txt" "the things brought in make 1 hole to cut, 0 in the same space and 0 set against a wall with what the job has"
says "$T/in-runs.txt" "the job had 47 holes to cut and now has 48"
says "$T/in-runs.txt" "a hole to cut: duct 1 and partition at 24, 14.00 in by 8.00 in, at (24, 20, 8)"
says "$T/in-runs.txt" "they are listed with the job's own by \`plumbline clashes $T/runs.json\`"
"$P" clashes "$T/runs.json" > "$T/clash1.txt"
says "$T/clash1.txt" "In the same space: 11 ("
says "$T/clash1.txt" "Through a wall or a floor (a hole to cut): 48 ("
says "$T/clash1.txt" "Tested: 195 elements"
diff "$T/clash0.txt" "$T/clash1.txt" | grep '^[<>]' | sed -E 's/\[(duct|wall|floor|equipment)-[0-9]+\]//g; s/  */ /g; s/ :/:/g' > "$T/clash.diff" || true
# The list differs by the count lines, the tested count, and this one line.
[ "$(grep -c '^>' "$T/clash.diff")" = 3 ] && [ "$(grep -c '^<' "$T/clash.diff")" = 2 ] || { echo "FAIL: the clash list differs by more than one line, the hole count and the tested count"; cat "$T/clash.diff"; exit 1; }
grep -qF "> ~ duct 1 and partition at 24: 1 hole, 14.00 in by 8.00 in, at (24, 20, 8)" "$T/clash.diff" || { echo "FAIL: the one line is not the duct through the partition"; cat "$T/clash.diff"; exit 1; }
printf '    the clash list differs by exactly: the duct through the partition at 24, the hole count 47 to 48, the tested count 193 to 195\n'
"$P" vif "$T/runs.json" > "$T/vif.txt"
grep -qE "partition at 24 \[wall-[0-9]+\] and duct 1 \[duct-[0-9]+\] passes through it \(a hole to cut" "$T/vif.txt" || { echo "FAIL: the hole is not a question to verify in the field"; cat "$T/vif.txt"; exit 1; }
printf '    and it is on the list of what to verify in the field\n'

echo "== 5. The same file again =="
cp "$T/runs.json" "$T/runs.before"
if "$P" import ifc "$T/runs.json" "$T/designer.ifc" --status Assumed --source "designer, made up" --only runs > "$T/again.txt" 2>&1; then echo "FAIL: the second bringing in was not refused"; cat "$T/again.txt"; exit 1; fi
says "$T/again.txt" "this model is already in the job: nothing was brought in. 2 of its 2 things are in the job already, in the same place, so a second would be counted twice"
says "$T/again.txt" "It holds 1 floor and 4 walls, which were not asked for"
cmp -s "$T/runs.json" "$T/runs.before" || { echo "FAIL: the job changed"; exit 1; }
printf '    the job is the same file byte for byte\n'

echo "== 6. Everything brought in reads Assumed, and says what it came from =="
"$P" show "$T/runs.json" > "$T/runs.show"
DUCT="$(grep -E '^ *duct-[0-9]+  duct 1  \[duct\]' "$T/runs.show" | grep -oE 'duct-[0-9]+' | head -1)"
AIR="$(grep -E '^ *equipment-[0-9]+  air handler [0-9]+  \[equipment\]' "$T/runs.show" | grep -oE 'equipment-[0-9]+' | head -1)"
grep -qE "^  ~ $DUCT duct 1 height above its level: 8 ft \(designer, made up: designer.ifc, \"duct 1\", duct #[0-9]+ in the file\)" "$T/runs.show" || { echo "FAIL: the duct is not Assumed and sourced to the file"; grep "$DUCT" "$T/runs.show"; exit 1; }
grep -qE "^  ~ $AIR air handler [0-9]+ position: \(30, 8\) ft \(designer, made up: designer.ifc, \"air handler\"" "$T/runs.show" || { echo "FAIL: the air handler is not Assumed and sourced to the file"; grep "$AIR" "$T/runs.show"; exit 1; }
printf '    %s and %s: Assumed (~), each value saying the model and the thing in it\n' "$DUCT" "$AIR"
# A model said Known is not Known: said so.
cp "$T/house.json" "$T/known.json"
"$P" import ifc "$T/known.json" "$T/designer.ifc" --status Known --source "the designer, by phone" --only runs > "$T/known.txt"
says "$T/known.txt" "was brought in Assumed, not Known as was said of it"

echo "== 7. A level at a height the job does not have =="
M="$T/mezz.json"
"$P" new "$M" "Mezzanine (not a real job)" >/dev/null
ML="$(made <("$P" level "$M" "Mezzanine" --elevation "4 ft" --height "5 ft" "${A[@]}") 'level-[0-9]+')"
MT="$(made <("$P" type "$M" duct "Duct 12 round" --round "12 in" "${A[@]}") 'type-[0-9]+')"
"$P" run "$M" "$ML" duct "$MT" --points "5,5 15,5" --height "3 ft" "${A[@]}" >/dev/null
"$P" export ifc "$M" "$T/mezz.ifc" >/dev/null
cp "$T/house.json" "$T/mezz-job.json"
"$P" import ifc "$T/mezz-job.json" "$T/mezz.ifc" --status Assumed --source "designer, made up" --only runs > "$T/mezz-in.txt"
says "$T/mezz-in.txt" "note: the model's level \"Mezzanine\" has its floor at 4 ft, which is not the height of any level the job has: it holds 1 thing, and each was put on the level it stands within (the job's \"first floor\"); no level was added"
"$P" show "$T/mezz-job.json" > "$T/mezz.show"
MD="$(grep -E '^ *duct-[0-9]+  duct 1  \[duct\]' "$T/mezz.show" | grep -oE 'duct-[0-9]+' | head -1)"
"$P" show "$T/mezz-job.json" "$MD" > "$T/mezz-duct.txt"
says "$T/mezz-duct.txt" "height above its level 7 ft"
[ "$("$P" show "$T/mezz-job.json" | grep -c '\[level\]')" = 3 ] || { echo "FAIL: a level was added"; exit 1; }
printf '    3 levels: none added; the duct is 7 ft up on the first floor (4 ft and 3 ft)\n'

echo "== 8. A model of walls alone, as only its runs =="
WM="$T/walls.json"
"$P" new "$WM" "Walls (not a real job)" >/dev/null
WL="$(made <("$P" level "$WM" "L1" --elevation "0 ft" --height "9 ft" "${A[@]}") 'level-[0-9]+')"
WT="$(made <("$P" type "$WM" wall "W" --thickness "6 in" "${A[@]}") 'type-[0-9]+')"
"$P" wall "$WM" "$WL" "$WT" --points "0,40 12,40 12,52" --height "9 ft" "${A[@]}" >/dev/null
"$P" export ifc "$WM" "$T/walls.ifc" >/dev/null
cp "$T/house.json" "$T/walls-job.json"
if "$P" import ifc "$T/walls-job.json" "$T/walls.ifc" --status Assumed --source "designer, made up" --only runs > "$T/walls-in.txt" 2>&1; then echo "FAIL: walls alone were not refused as only runs"; exit 1; fi
says "$T/walls-in.txt" "this model has no runs and no equipment: nothing was brought in. It holds 2 walls, which were not asked for"
cmp -s "$T/walls-job.json" "$T/house.json" || { echo "FAIL: the job changed"; exit 1; }
if "$P" import ifc "$T/walls-job.json" "$T/walls.ifc" --status Assumed --source "x" --only walls >/dev/null 2>"$T/bad.txt"; then echo "FAIL: --only walls was taken"; exit 1; fi
says "$T/bad.txt" "the choice is runs"

echo
echo "ifc-runs: every step passed"
