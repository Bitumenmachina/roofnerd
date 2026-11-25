#!/usr/bin/env bash
# What the second command-line judge found (2026-11-25), fixed (call 213): the fourteen steps of STATUS.md ("The command line judged a second time"), at the
# command line, from empty jobs, every figure made up or a starter row's. Each step ends the script where the program does not say what the step expects.
#   1. an item renamed to the name of a material is refused, and the job is what it was
#   2. how far a run's end stands off its connection is measured, and a run brought onto it is said joined
#   3. a run that already stood off and is carried farther is said, and one brought back is said joined
#   4. a run through a thing over its waste connection: takeoff names the connection; the clash says joined, and goes on so far inside
#   5. connect, with an end 1/2 in outside the thing: refused, naming the connection and the distance
#   6. a size typed at the command line says so; the same size again changes nothing; Known with how sure and where from
#   7. a turn is said from 0 up to 360 degrees; a turn by nothing changes nothing
#   8. a length in a figure is written as lengths are written
#   9. a size typed on a connection says it was typed there
#  10. a designer's model at 25 ft brought into the sample house: 9 ft above the top of the second floor, said
#  11. a model with one fixture and no run, as only its runs: the fixture comes in
#  12. three shapes kept, an item drawn as the last: all three written out, and brought into a new job
#  13. a figure typed on the vanity: refused on the vanity itself, first; then on the smaller item a shape is also tried on
#  14. a block with no thickness, and one written backwards: refused
# Run from the repository root:  PLUMBLINE_BIN=<a folder with plumbline> demos/judged-1125b.sh   (without it the script builds the command-line program)
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
K=(--status Known --source "made-up plan")
A=(--status Assumed --source "made-up quote")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refuses() { # refuses <file> <command...>: the program refuses, and what it says is in <file>
  local out="$1"; shift
  if "$P" "$@" > "$out" 2>&1; then echo "FAIL: not refused: $*"; cat "$out"; exit 1; fi
}
path_of() { sed -n "s/^\\($1[0-9]*\\).*/\\1/p" | head -1; }
# The starter lavatory (20 in wide, 18 in deep, 34 in high) with its middle at (10, 20), in a job with a level Ground and a pipe type.
lav() {
  "$P" new "$1" "Judged (not a real job)" >/dev/null
  LV=$("$P" level "$1" Ground --elevation "0 ft" --height "9 ft" "${K[@]}" | path_of level-)
  "$P" import items "$1" starter >/dev/null
  "$P" import connections "$1" starter >/dev/null
  THING=$("$P" place "$1" "$LV" Lavatory --at "10,20" --offset "0 ft" "${K[@]}" | path_of fixture-)
  PT=$("$P" type "$1" pipe "pipe half" --round "1/2 in" "${K[@]}" | path_of type-)
}
pipe() { "$P" run "$1" "$LV" pipe "$PT" --points "$2" --height "${3:-21 in}" "${A[@]}" | sed -n 's/^1 straight piece: \(.*\)$/\1/p'; }

echo "== 1. Starter items and a price list with Plywood; the Refrigerator renamed Plywood is refused; the job is what it was and still opens =="
"$P" new "$T/q.json" "Names (not a real job)" >/dev/null
"$P" import items "$T/q.json" starter >/dev/null
printf 'Material,Bought by,Price\nPlywood,SHEET,40\n' > "$T/m.csv"
"$P" import prices "$T/q.json" "$T/m.csv" --add --status Assumed --source "made-up list" >/dev/null
cp "$T/q.json" "$T/q.before"
refuses "$T/no.txt" rename "$T/q.json" "Refrigerator" "Plywood"
says "$T/no.txt" 'there is already something called "Plywood"'
says "$T/no.txt" "two things in one place cannot go by one name"
cmp "$T/q.json" "$T/q.before" && echo "    the job is the same to the byte"
"$P" check "$T/q.json" > "$T/check.txt"
says "$T/check.txt" "agree"

echo
echo "== 2. The cold pipe ends at (10.3333, 20.95), 0.2 ft = 2.4 in behind the cold connection (20.75): the depth typed 24 in brings the connection onto it; back to 18 in: it is 2 3/8 in off, not the 3 in the connection moved =="
lav "$T/a.json"
COLD=$(pipe "$T/a.json" "10.3333,26 10.3333,20.95")
"$P" set "$T/a.json" Lavatory depth "24 in" > "$T/d24.txt"
says "$T/d24.txt" "typed at the command line"
says "$T/d24.txt" "1 run is now joined to its connection"
says "$T/d24.txt" "$COLD"
"$P" set "$T/a.json" Lavatory depth "18 in" > "$T/d18.txt"
says "$T/d18.txt" "1 run now stands off its connection"
says "$T/d18.txt" "is 2 3/8 in behind Lavatory's Cold water connection"
lacks "$T/d18.txt" "is 3 in"

echo
echo "== 3. A second pipe ends exactly at the hot connection; the depth typed 12 in moves the back 3 in nearer: the second stands off 3 in; the first, 2.4 in + 3 in = 5 3/8 in; back to 18 in: the second is joined =="
HOT=$(pipe "$T/a.json" "9.6667,26 9.6667,20.75")
"$P" set "$T/a.json" Lavatory depth "12 in" > "$T/d12.txt"
says "$T/d12.txt" "1 run now stands off its connection"
says "$T/d12.txt" "$HOT"
says "$T/d12.txt" "is 3 in behind Lavatory's Hot water connection"
says "$T/d12.txt" "1 run still stands off its connection"
says "$T/d12.txt" "is 5 3/8 in behind Lavatory's Cold water connection (it was 2 3/8 in)"
"$P" set "$T/a.json" Lavatory depth "18 in" > "$T/d18b.txt"
says "$T/d18b.txt" "1 run is now joined to its connection"
says "$T/d18b.txt" "$HOT"
says "$T/d18b.txt" "is 2 3/8 in behind Lavatory's Cold water connection (it was 5 3/8 in)"

echo
echo "== 4. A pipe drawn through the lavatory from (10, 25) to (10, 18) at 19 in, over its waste connection at (10, 20.75): takeoff does not call the waste open and names it; the clash says joined, and goes on 18 in inside =="
lav "$T/b.json"
pipe "$T/b.json" "10,25 10,18" "19 in" >/dev/null
"$P" takeoff "$T/b.json" > "$T/take.txt"
lacks "$T/take.txt" "open: Lavatory: its Waste connection"
says "$T/take.txt" "passes over Lavatory's Waste connection"
lacks "$T/take.txt" "1 place where a run meets another along its length"
"$P" clashes "$T/b.json" > "$T/clash.txt"
says "$T/clash.txt" "joined to Lavatory's Waste connection and goes on 18 in inside it"

echo
echo "== 5. connect, with the cold pipe's end 1/2 in outside the lavatory: refused, naming the cold water connection and 1/2 in =="
lav "$T/c.json"
RUN=$(pipe "$T/c.json" "10.3333,25 10.3333,20.79")
refuses "$T/no.txt" connect "$T/c.json" "$THING" "$RUN" "${A[@]}"
says "$T/no.txt" "Cold water connection"
says "$T/no.txt" "its end is 1/2 in away"
lacks "$T/no.txt" "10.3333"

echo
echo "== 6. A size typed at the command line says so; the same size again changes nothing and is no check; Known with how sure and where from =="
lav "$T/d.json"
"$P" set "$T/d.json" Lavatory depth "24 in" > "$T/s1.txt"
says "$T/s1.txt" "Assumed: typed at the command line; not yet checked against its maker's sheet"
cp "$T/d.json" "$T/d.before"
"$P" set "$T/d.json" Lavatory depth "24 in" > "$T/s2.txt"
says "$T/s2.txt" "depth is 24 in already: nothing changed"
lacks "$T/s2.txt" "Known, checked on site"
cmp "$T/d.json" "$T/d.before" && echo "    the job is the same to the byte"
"$P" set "$T/d.json" Lavatory depth "24 in" --status Known --source "J. Smith, tape" > "$T/s3.txt"
says "$T/s3.txt" "depth 24 in is Known: J. Smith, tape"
"$P" items "$T/d.json" Lavatory > "$T/item.txt"
says "$T/item.txt" "depth: J. Smith, tape"

echo
echo "== 7. A thing turned 450 degrees reads 90 everywhere; by 0 it says it changes nothing =="
"$P" turn "$T/d.json" "$THING" --by 450 "${A[@]}" > "$T/t1.txt"
says "$T/t1.txt" "turned 90 degrees from east"
"$P" turn "$T/d.json" "$THING" --by 180 "${A[@]}" > "$T/t2.txt"
says "$T/t2.txt" "turned 270 degrees from east"
"$P" show "$T/d.json" "$THING" > "$T/show.txt"
says "$T/show.txt" "270 deg"
cp "$T/d.json" "$T/d.before"
refuses "$T/no.txt" turn "$T/d.json" "$THING" --by 0 "${A[@]}"
says "$T/no.txt" "changes nothing"
cmp "$T/d.json" "$T/d.before" && echo "    the job is the same to the byte"

echo
echo "== 8. A connection's Up typed as 1/2 in, 5 1/2 in, 3 ft 6 in and 3'6\": each read; on an item 34 in high the 42 in is 8 in above it =="
lav "$T/e.json"
"$P" connection "$T/e.json" Lavatory set Waste up "1/2 in" > "$T/u1.txt"
"$P" items "$T/e.json" Lavatory | grep "Waste" > "$T/w1.txt"
says "$T/w1.txt" "up 1/2 in"
"$P" connection "$T/e.json" Lavatory set Waste up "5 1/2 in" >/dev/null
"$P" items "$T/e.json" Lavatory | grep "Waste" > "$T/w2.txt"
says "$T/w2.txt" "up 5 1/2 in"
refuses "$T/no.txt" connection "$T/e.json" Lavatory set Waste up "3 ft 6 in"
says "$T/no.txt" "8 in above the item"
refuses "$T/no.txt" connection "$T/e.json" Lavatory set Waste up "3'6\""
says "$T/no.txt" "8 in above the item"
lacks "$T/no.txt" "'\\''"
refuses "$T/no.txt" connection "$T/e.json" Lavatory set Waste up "3#"
says "$T/no.txt" "3 ft 6 in"
lacks "$T/no.txt" "'\\''"

echo
echo "== 9. A connection's size typed with nothing else is Assumed and typed at the command line; with how sure and where from, as said =="
"$P" connection "$T/e.json" Lavatory set "Hot water" size "1/2 in" > "$T/z1.txt"
says "$T/z1.txt" "size 1/2 in (Assumed: typed at the command line; not checked against the maker's sheet)"
"$P" connection "$T/e.json" Lavatory set "Cold water" size "3/4 in" --status Assumed --source "J. Smith, rough-in sheet" > "$T/z2.txt"
says "$T/z2.txt" "size 3/4 in (Assumed: J. Smith, rough-in sheet)"

echo
echo "== 10. A designer's model with a level at 25 ft and a duct 3 ft above it, into the sample whole house (first floor 0 ft, second 10 ft, 9 ft high: its top is 19 ft): the duct is 28 ft up, 9 ft above, and the reply says to check the heights =="
"$P" sixteen "$T/s.json" >/dev/null
"$P" new "$T/m.json" "Designer (not a real job)" >/dev/null
ML=$("$P" level "$T/m.json" "Mechanical" --elevation "25 ft" --height "5 ft" "${K[@]}" | path_of level-)
MT=$("$P" type "$T/m.json" duct "Duct round 12" --round "12 in" "${K[@]}" | path_of type-)
"$P" run "$T/m.json" "$ML" duct "$MT" --points "5,5 15,5" --height "3 ft" "${K[@]}" >/dev/null
"$P" export ifc "$T/m.json" "$T/m.ifc" >/dev/null
"$P" import ifc "$T/s.json" "$T/m.ifc" --status Assumed --source "made-up designer" --only runs > "$T/in.txt"
says "$T/in.txt" "each was put on the level it stands within"
says "$T/in.txt" "stands above the top of the job's highest level"
says "$T/in.txt" "9 ft above it"
says "$T/in.txt" "check the model's heights against the job's"

echo
echo "== 11. A model with one fixture and no run, as only its runs: the fixture comes in; the usage line says fixtures and equipment come whether a run reaches them or not =="
"$P" new "$T/t.json" "Tub (not a real job)" >/dev/null
TL=$("$P" level "$T/t.json" "Ground" --elevation "0 ft" --height "9 ft" "${K[@]}" | path_of level-)
"$P" place "$T/t.json" "$TL" fixture "Tub" --at "5,5" --offset "0 ft" --size "2,5,2" "${K[@]}" >/dev/null
"$P" export ifc "$T/t.json" "$T/t.ifc" >/dev/null
"$P" new "$T/e2.json" "Empty (not a real job)" >/dev/null
"$P" level "$T/e2.json" "Ground" --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" import ifc "$T/e2.json" "$T/t.ifc" --status Assumed --source "made-up designer" --only runs > "$T/in2.txt"
says "$T/in2.txt" "1 fixture"
"$P" takeoff "$T/e2.json" > "$T/take2.txt"
lacks "$T/take2.txt" "?none"
"$P" 2> "$T/usage.txt" || true
says "$T/usage.txt" "whether a run reaches it or not"

echo
echo "== 12. A shape kept three times under three names, the item drawn as the last: export shapes writes all three and says two are drawn as by no item; brought into a new job, all three are there =="
lav "$T/f.json"
for n in "Lavatory a" "Lavatory b" "Lavatory c"; do "$P" shape "$T/f.json" keep Lavatory "$n" >/dev/null; done
"$P" export shapes "$T/f.json" "$T/shapes.csv" > "$T/x.txt"
says "$T/x.txt" "no item is drawn as 2 of them"
for n in "Lavatory a" "Lavatory b" "Lavatory c"; do grep -q "^$n," "$T/shapes.csv" || { echo "FAIL: $n is not in the table"; exit 1; }; done
"$P" new "$T/g.json" "New (not a real job)" >/dev/null
"$P" import shapes "$T/g.json" "$T/shapes.csv" > "$T/in3.txt"
"$P" shape "$T/g.json" > "$T/list.txt"
for n in "Lavatory a" "Lavatory b" "Lavatory c"; do says "$T/list.txt" "$n"; done

echo
echo "== 13. On the 36 in vanity (36 in wide, 21 in deep, 34 1/2 in high) a block's Up to: height + 1 ft is 12 in above the top; 36 in is 1 1/2 in; 30 in holds on the vanity and is refused on the smaller item a shape is also tried on, and says why =="
"$P" new "$T/v.json" "Vanity (not a real job)" >/dev/null
"$P" import items "$T/v.json" starter >/dev/null
"$P" shape "$T/v.json" keep "Vanity, 36 in" "Vanity panel" >/dev/null
refuses "$T/no.txt" shape "$T/v.json" set "Vanity panel" 1 "up to" "height + 1 ft"
says "$T/no.txt" "Vanity, 36 in"
says "$T/no.txt" "it would stand 12 in above the top"
lacks "$T/no.txt" "4 13/16"
refuses "$T/no.txt" shape "$T/v.json" set "Vanity panel" 1 "up to" "36 in"
says "$T/no.txt" "it would stand 1 1/2 in above the top"
refuses "$T/no.txt" shape "$T/v.json" set "Vanity panel" 1 "up to" "30 in"
says "$T/no.txt" "holds on Vanity, 36 in"
says "$T/no.txt" "a shape has to hold on a smaller item too"

echo
echo "== 14. Up from typed height with Up to height: no thickness; Across from width / 2 with Across to 0: backwards =="
cp "$T/v.json" "$T/v.before"
refuses "$T/no.txt" shape "$T/v.json" set "Vanity panel" 3 "up from" "height"
says "$T/no.txt" "has no thickness"
says "$T/no.txt" "34 1/2 in to 34 1/2 in"
"$P" shape "$T/v.json" set "Vanity panel" 3 "across to" "0" >/dev/null
refuses "$T/no.txt" shape "$T/v.json" set "Vanity panel" 3 "across from" "width / 2"
says "$T/no.txt" "is backwards"
says "$T/no.txt" "18 in to 0 in"

echo
echo "PASS: the fourteen steps"
