#!/usr/bin/env bash
# Another trade's or a designer's model brought in, at the command line: a made-up IFC4
# model of a small house (demos/ifc/made-up-house.ifc; tools/made-up-ifc.py writes it, in
# metres) into an empty job. What comes in is measured and checked for clashes like
# anything drawn here; what does not is listed.
# Run from the repository root:  demos/model.sh
# The model is made up. Every step fails the script if the program does not do what it says.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet -p plumbline
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3"; exit 1; }; printf '    %s\n' "$2"; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | head -1 | cut -c1-200
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
}

echo "== An empty job, and the model brought into it =="
"$P" new "$J" "Brought in from a model (not a real job)" >/dev/null
cp "$J" "$T/before.json"
refused "say how sure the model is and where it came from" "$P" import ifc "$J" demos/ifc/made-up-house.ifc
cmp -s "$J" "$T/before.json" || { echo "FAIL: a refused model changed the job"; exit 1; }
OUT="$("$P" import ifc "$J" demos/ifc/made-up-house.ifc --status Assumed --source "the designer's model (made up)")"
echo "$OUT" | sed 's/^/    /'
# The model holds two storeys, four walls, a door, a window, a slab, a space, a duct, a
# pipe, and a beam, a stair and a chair.
for made in "2 storeys" "4 walls" "1 doors" "1 windows" "1 floors" "1 rooms" "1 ducts" "1 pipes"; do says "brought in" "$made" "$OUT"; done
says "what was not" "not brought in: 3 things" "$OUT"
for left in "1 IFCBEAM: this program has no element for it" "1 IFCSTAIR: this program has no element for it" "1 IFCFURNITURE: this program has no element for it"; do says "listed" "$left" "$OUT"; done
says "its unit" "the model is in metres: every length was turned into feet" "$OUT"
"$P" check "$J"

echo
echo "== What it measures to =="
OUT="$("$P" takeoff "$J" | tr -s ' ')"
echo "$OUT" | sed 's/^/    /' | head -24
# In metres the house is 9.144 by 6.096, its walls 2.4384 tall: 30 ft by 20 ft by 8 ft.
#   walls 2 x (30 + 20) = 100 LF; gross 100 x 8 = 800 SF; less a 3 x 7 door and a 4 x 3
#   window, 800 - 21 - 12 = 767 SF. The slab 30 x 20 = 600 SF. The room inside 6 in walls
#   on their centre lines, 29.5 x 19.5 = 575.25 SF. The duct 6.096 m = 20 LF; the pipe
#   3.048 m = 10 LF. Every one Assumed: the model was brought in as that.
says "walls" "~ walls 4 100.00 LF" "$OUT"
says "gross" "~ wall face, gross 800.00 SF" "$OUT"
says "net" "~ wall face, net 767.00 SF" "$OUT"
says "floor" "~ floors 1 600.00 SF" "$OUT"
says "room" "~ rooms 1 575.25 SF" "$OUT"
says "duct" "~ Duct 6 in round 1 piece 20.00 LF" "$OUT"
says "pipe" "~ Pipe 1 in round 1 piece 10.00 LF" "$OUT"

echo
echo "== What the model hid: its duct and its pipe are in the same space =="
OUT="$("$P" clashes "$J")"
echo "$OUT" | sed 's/^/    /' | head -4
# Both 2.1336 m, 7 ft, up; they cross at (10, 10) ft. A 6 in duct and a 1 in pipe on one
# centre height: 3 in + 1/2 in. A question for the field: the model is Assumed.
says "the clash" "In the same space: 1 (1 to verify in the field)" "$OUT"
says "by how much, and where" "in each other by 3.50 in, at (10, 10, 7)" "$OUT"

echo
echo "== What is refused, and leaves the job as it was =="
cp "$J" "$T/before.json"
echo "not a model" > "$T/notes.ifc"
refused "it is not an IFC file this can read" "$P" import ifc "$J" "$T/notes.ifc" --status Assumed --source "x"
refused "does not take --add" "$P" import ifc "$J" demos/ifc/made-up-house.ifc --add --status Assumed --source "x"
cmp -s "$J" "$T/before.json" || { echo "FAIL: a refused model changed the job"; exit 1; }
echo "    the job is byte for byte what it was before the refusals"

echo
echo "PASS: a model in metres was brought into an empty job as storeys, walls, a door, a window, a floor, a room, a duct and a pipe, measured to the figures worked by hand, with what was not brought in listed and the clash it hid found"
