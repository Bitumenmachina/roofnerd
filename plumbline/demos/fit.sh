#!/usr/bin/env bash
# Fittings and connections, at the command line: a duct with a corner and a branch, a
# pipe that changes size, an air handler and a lavatory. A fitting is put at each joint
# and worked out from what meets there; each run's end is joined to the thing it serves;
# and the takeoff says, before and after, what is joined and what is still open.
# Run from the repository root:  demos/fit.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures expected are
# worked by hand in the comments.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")

made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1"; }   # the path a verb reports first
takeoff() { "$P" takeoff "$J" | tr -s ' ' > "$T/takeoff.txt"; }
says() { grep -qF -- "$2" "$T/takeoff.txt" || { echo "FAIL: $1: the takeoff does not say: $2"; cat "$T/takeoff.txt"; exit 1; }; printf '    %s\n' "$2"; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | head -1 | cut -c1-220
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
}

echo "== A storey, the types and two systems =="
"$P" new "$J" "Fitted runs (not a real job)"
S="$(made "$("$P" storey "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
DUCT="$(made "$("$P" type "$J" duct "6 in round" --round "6 in" "${K[@]}")")"
BIG="$(made "$("$P" type "$J" pipe "3/4 in" --round "3/4 in" "${K[@]}")")"
SMALL="$(made "$("$P" type "$J" pipe "1/2 in" --round "1/2 in" "${K[@]}")")"
AIR="$(made "$("$P" system "$J" "Supply air" "Supply Air")")"
COLD="$(made "$("$P" system "$J" "Cold water" "Domestic Cold Water")")"

echo
echo "== A duct with a corner, and a branch off the middle of its first piece =="
OUT="$("$P" run "$J" "$S" duct "$DUCT" --points "2,2 10,2 10,14" --height "8 ft" --system "$AIR" "${K[@]}")"; echo "    $OUT"
read -r D1 D2 <<<"$(sed -E 's/^[^:]*: //; s/,//g' <<<"$OUT")"
OUT="$("$P" run "$J" "$S" duct "$DUCT" --points "6,2 6,10" --height "8 ft" --system "$AIR" "${K[@]}")"; echo "    $OUT"
BRANCH="$(sed -E 's/^[^:]*: //' <<<"$OUT")"
takeoff
# 8 + 12 + 8 = 28 LF. The pieces meet at the corner; the branch meets the first piece
# along its length, which is a place with no fitting; three ends meet nothing.
says "the duct" "28.00 LF"
says "before" "joined: 1 network; 3 open ends; 1 place where a run meets another along its length with no fitting"

echo
echo "== A fitting at every joint of the supply air: what meets says what it is =="
"$P" fitting "$J" --all --system "$AIR" "${K[@]}" | tee "$T/fitted.txt"
grep -q "^2 fittings" "$T/fitted.txt" || { echo "FAIL: wanted 2 fittings"; exit 1; }
grep -q "a bend" "$T/fitted.txt" && grep -q "a tee; $D1 was cut there" "$T/fitted.txt" || { echo "FAIL: wanted a bend, and a tee that cut $D1"; exit 1; }
"$P" check "$J"
takeoff
# The first piece was cut at the branch, 8 LF into 4 and 4: the duct is still 28 LF.
says "the duct, cut" "28.00 LF"
says "the elbow" "Bend 6 in round elbow: 1 EA"
says "the tee" "Tee 6 in round tee: 1 EA"
says "after" "joined: 1 network; 3 open ends"

echo
echo "== The duct's first end joined to an air handler =="
# 2 ft square and 4 ft tall with its base 6 ft up: the duct's end, 8 ft up, is inside it.
AHU="$(made "$("$P" place "$J" "$S" equipment "Air handler" --at "2,2" --offset "6 ft" --size "2,2,4" --system "$AIR" "${K[@]}")")"
"$P" connect "$J" "$AHU" "$D1" "${K[@]}"
takeoff
says "one end fewer open" "joined: 1 network; 2 open ends"

echo
echo "== A pipe that changes size, and a lavatory at its end =="
OUT="$("$P" run "$J" "$S" pipe "$BIG" --points "13,1 13,8" --height "1 ft" --system "$COLD" "${K[@]}")"; echo "    $OUT"
OUT="$("$P" run "$J" "$S" pipe "$SMALL" --points "13,8 13,15" --height "1 ft" --system "$COLD" "${K[@]}")"; echo "    $OUT"
LAST="$(sed -E 's/^[^:]*: //' <<<"$OUT")"
"$P" fitting "$J" "$S" --at "13,8" "${K[@]}" | tee "$T/one.txt"
grep -q "a transition at (13, 8)" "$T/one.txt" || { echo "FAIL: wanted a transition"; exit 1; }
LAV="$(made "$("$P" place "$J" "$S" fixture "Lavatory" --at "13,15" --offset "0 ft" --size "2,1.5,3" --system "$COLD" "${K[@]}")")"
"$P" connect "$J" "$LAV" "$LAST" "${K[@]}"
"$P" check "$J"
takeoff
# 7 + 7 = 14 LF of pipe; one transition; open only where the 3/4 in starts.
says "the pipe" "14.00 LF"
says "the transition" "Transition 3/4 in to 1/2 in transition: 1 EA"
says "the cold water" "joined: 1 network; 1 open end"

echo
echo "== What is refused, and leaves the file as it was =="
cp "$J" "$T/before.json"
refused "a fitting joins two, three or four runs' ends" "$P" fitting "$J" "$S" --at "10,14" "${K[@]}"
refused "is already the fitting" "$P" fitting "$J" "$S" --at "10,2" "${K[@]}"
refused "nothing meets at (20, 20)" "$P" fitting "$J" "$S" --at "20,20" "${K[@]}"
refused "is not a kind of fitting" "$P" fitting "$J" "$S" --at "10,2" --kind elbow "${K[@]}"
refused "neither end of $BRANCH is at $AHU" "$P" connect "$J" "$AHU" "$BRANCH" "${K[@]}"
refused "already has a connector where" "$P" connect "$J" "$AHU" "$D1" "${K[@]}"
refused "is not on $D2" "$P" split "$J" "$D2" --at "11,6" "${K[@]}"
refused "--status is required" "$P" fitting "$J" "$S" --at "10,14"
cmp -s "$J" "$T/before.json" || { echo "FAIL: a refused verb changed the file"; exit 1; }
echo "    the file is byte for byte what it was before the refusals"

echo
echo "PASS: fittings worked out from what meets, runs joined to what they serve, and the takeoff says what is joined and what is open"
