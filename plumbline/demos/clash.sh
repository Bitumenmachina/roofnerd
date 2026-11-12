#!/usr/bin/env bash
# Clash detection, at the command line: two tanks in each other, a duct and a pipe that
# cross at one height, a pipe through a wall and through a window, a pipe that only comes
# close, and what is joined, which is no clash. Then the sample house.
# Run from the repository root:  demos/clash.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures expected are
# worked by hand in the comments.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")

made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1"; }   # the path a verb reports first
last() { sed -E 's/^[^:]*: //' <<<"$1"; }                # the path of a one-piece run
clashes() { "$P" clashes "$J" "$@" > "$T/clashes.txt"; }
says() { grep -qF -- "$2" "$T/clashes.txt" || { echo "FAIL: $1: the report does not say: $2"; cat "$T/clashes.txt"; exit 1; }; printf '    %s\n' "$2"; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | head -1 | cut -c1-200
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
}

echo "== A level and the types =="
"$P" new "$J" "Clashes (not a real job)"
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 4 by 3" --width "4 ft" --height "3 ft" "${K[@]}")")"
DUCT="$(made "$("$P" type "$J" duct "6 in round" --round "6 in" "${K[@]}")")"
PIPE="$(made "$("$P" type "$J" pipe "1 in" --round "1 in" "${K[@]}")")"
AIR="$(made "$("$P" system "$J" "Supply air" "Supply Air")")"
clashes
says "an empty level" "In the same space: 0"

echo
echo "== Two tanks, 2 ft each way, their middles 1 ft 6 in apart =="
"$P" place "$J" "$S" equipment "Tank A" --at "30,30" --offset "0 ft" --size "2,2,2" "${K[@]}" >/dev/null
B="$(made "$("$P" place "$J" "$S" equipment "Tank B" --at "31.5,30" --offset "0 ft" --size "2,2,2" "${K[@]}")")"
clashes
# They overlap by 2 ft − 1 ft 6 in = 6 in: one moves 6 in to clear the other.
says "the tanks" "In the same space: 1"
says "by how much" "in each other by 6.00 in"
# Moved 6 in apart they touch: no clash. Asked for a clearance, they are closer than it.
"$P" move "$J" "$B" --by "0.5,0" --status Assumed --source "moved by eye" >/dev/null
clashes
says "touching" "In the same space: 0"
clashes --clearance "2 in"
says "touching, and a clearance asked for" "Closer than 2.00 in: 1 (1 to verify in the field)"
says "how far apart" "0.00 in apart"
"$P" move "$J" "$B" --by "20,0" "${K[@]}" >/dev/null

echo
echo "== A 6 in duct and a 1 in pipe crossing at one height =="
# The duct east along y = 5 and the pipe north along x = 5, both 8 ft up: their center
# lines cross. To clear, one moves by both radii: 3 in + 1/2 in.
D="$(last "$("$P" run "$J" "$S" duct "$DUCT" --points "0,5 10,5" --height "8 ft" "${K[@]}")")"
X="$(last "$("$P" run "$J" "$S" pipe "$PIPE" --points "5,0 5,10" --height "8 ft" --status Assumed --source "set by eye")")"
clashes
says "the crossing, flagged" "~ duct"
says "the crossing" "in each other by 3.50 in, at (5, 5, 8)"
says "one of one assumed" "In the same space: 1 (1 to verify in the field)"
"$P" remove "$J" "$X" >/dev/null
# The pipe 5 in higher: the center lines are 5 in apart and the surfaces 5 − 3.5 = 1.5 in.
X="$(last "$("$P" run "$J" "$S" pipe "$PIPE" --points "5,0 5,10" --height "8 ft 5 in" "${K[@]}")")"
clashes
says "clear of it" "In the same space: 0"
says "no clearance asked" "Closer than a clearance: not asked for"
clashes --clearance "2 in"
says "closer than 2 in" "Closer than 2.00 in: 1"
says "how far apart" "1.50 in apart"
clashes --clearance "1 in"
says "not closer than 1 in" "Closer than 1.00 in: 0"

echo
echo "== A pipe through a wall, and through a window in it =="
# A wall east along y = 20, 10 ft long and 9 ft tall, with a window 4 ft by 3 ft at its
# middle, its sill 3 ft up: the hole is from 3 ft to 7 ft along and 3 ft to 6 ft up.
W="$(last "$("$P" wall "$J" "$S" "$WT" --points "0,20 10,20" --height "9 ft" "${K[@]}")")"
"$P" window "$J" "$W" "$NT" --offset "5 ft" --sill "3 ft" "${K[@]}" >/dev/null
"$P" run "$J" "$S" pipe "$PIPE" --points "5,17 5,23" --height "4 ft" "${K[@]}" >/dev/null
clashes
says "through the window is through nothing" "Through a wall or a floor (a hole to cut): 0"
"$P" run "$J" "$S" pipe "$PIPE" --points "5,17 5,23" --height "7 ft" "${K[@]}" >/dev/null
clashes
says "over the window's head" "Through a wall or a floor (a hole to cut): 1"
says "where" "at (5, 20, 7)"
says "and not among the clashes" "In the same space: 0"

echo
echo "== What is joined is not a clash =="
OUT="$("$P" run "$J" "$S" duct "$DUCT" --points "20,2 28,2 28,14" --height "8 ft" --system "$AIR" "${K[@]}")"
read -r D1 D2 <<<"$(sed -E 's/^[^:]*: //; s/,//g' <<<"$OUT")"
"$P" fitting "$J" "$S" --at "28,2" "${K[@]}" >/dev/null
clashes
# The two pieces are in each other at the corner, and so is the elbow: all joined there.
says "a corner and its elbow" "In the same space: 0"
AHU="$(made "$("$P" place "$J" "$S" equipment "Air handler" --at "20,2" --offset "6 ft" --size "2,2,4" --system "$AIR" "${K[@]}")")"
clashes
# The handler is from 19 ft to 21 ft east; the duct starts on its middle and runs east:
# 1 ft of it is inside, and it clears by moving that 12 in.
says "a duct in an air handler it is not joined to" "in each other by 12.00 in"
"$P" connect "$J" "$AHU" "$D1" "${K[@]}" >/dev/null
clashes
says "joined to it" "In the same space: 0"

echo
echo "== What has no shape is not tested, and that is said =="
NOSIZE="$(made "$("$P" place "$J" "$S" fixture "Something" --at "5,5" --offset "8 ft" "${K[@]}")")"
clashes
says "the thing with no size" "$NOSIZE: it has no shape"
says "what is never tested" "walls and floors are not tested against each other"

echo
echo "== Asking changes nothing, and what cannot be asked is refused =="
cp "$J" "$T/before.json"
clashes --clearance "6 in"
refused "more than nothing" "$P" clashes "$J" --clearance "0 in"
refused "ask for less" "$P" clashes "$J" --clearance "11 ft"
refused "unit" "$P" clashes "$J" --clearance 2
cmp -s "$J" "$T/before.json" || { echo "FAIL: asking for clashes changed the file"; exit 1; }
echo "    the file is byte for byte what it was"

echo
echo "== The sample house =="
"$P" house "$T/house.json" >/dev/null
"$P" clashes "$T/house.json" > "$T/house.txt"
grep -E "^(In the same space|Through a wall|Closer than|Tested|Not tested)" "$T/house.txt" | sed 's/^/    /'
# The house is drawn, not engineered: its trunk duct is drawn through its soil stack.
grep -qF "first-floor trunk, west, piece 2 of 3 [duct-0100] and soil stack [pipe-0246]: in each other by 3.50 in" "$T/house.txt" || { echo "FAIL: the duct through the soil stack is not reported"; exit 1; }
echo "    first-floor trunk, west, piece 2 of 3 and soil stack: in each other by 3.50 in"
# A hot branch to a lavatory is drawn a foot back up inside the hot main it comes off: a
# 1/2 in pipe in a 3/4 in pipe, in each other by both radii, 1/4 + 3/8 = 5/8 in. The two
# are joined at the main's end. That excuses the joint, not the foot of pipe.
grep -qF "hot main to the baths, piece 3 of 4 [pipe-0185] and hot to lavatory 1, piece 1 of 2 [pipe-0202]: in each other by 0.62 in" "$T/house.txt" || { echo "FAIL: the hot branch drawn back up inside the hot main is not reported"; exit 1; }
echo "    hot main to the baths, piece 3 of 4 and hot to lavatory 1, piece 1 of 2: in each other by 0.62 in (joined at one end, and one inside the other for a foot)"

echo
echo "PASS: what is in the same space is found with how far and how sure; what passes through a wall is counted apart; a joint is not a clash, and what is joined is still tested away from its joint"
