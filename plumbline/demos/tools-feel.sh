#!/usr/bin/env bash
# How tools feel, the model's side, at the command line (DECISIONS.md call 219; contract 219M; the steps are in STATUS.md, "How tools feel,
# the model's side: a demonstration", with the figures the program's own way corrects). Every figure is made up. Run from the repository root:
#   demos/tools-feel.sh
# A job: one level Ground (0 ft, 9 ft high); four walls 6 in thick round 20 ft by 12 ft (0,0 20,0 20,12 0,12, closed: wall 1 south from its west end,
# wall 3 north from its EAST end); a door 3 ft by 6 ft 8 in in wall 1, its MIDDLE 4 ft from the wall's first end; the starter items.
# Every step fails the script if the program does not do what the step says.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and nothing is built.
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made up")
A=(--status Assumed --source "made up")
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3"; exit 1; }; printf '    %s\n' "$2"; }
not_said() { ! grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: said: $2"; echo "$3"; exit 1; }; }
is() { [ "$2" = "$3" ] || { echo "FAIL: $1: got $2, wanted $3"; exit 1; }; printf '    %-60s %s\n' "$1" "$2"; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $(head -1 <<<"$out" | cut -c1-220)"
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; echo "$out"; exit 1; }
}
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
word() { grep -oE "$2-[0-9]+" <<<"$1" | head -1; }
sum() { sha256sum "$J" | cut -d' ' -f1; }

"$P" new "$J" "Tools (made up)" >/dev/null
LEVEL="$(word "$("$P" level "$J" Ground --elevation "0 ft" --height "9 ft" "${K[@]}")" level)"
WT="$(word "$("$P" type "$J" wall "Wall 6" --thickness "6 in" "${K[@]}")" type)"
DT="$(word "$("$P" type "$J" door "Door 3068" --width "3 ft" --height "6 ft 8 in" "${K[@]}")" type)"
WALLS="$("$P" wall "$J" "$LEVEL" "$WT" --points "0,0 20,0 20,12 0,12" --closed --height "9 ft" "${K[@]}")"
W1="$(word "$WALLS" wall)"; W3="$(grep -oE 'wall-[0-9]+' <<<"$WALLS" | sed -n 3p)"; W2="$(grep -oE 'wall-[0-9]+' <<<"$WALLS" | sed -n 2p)"
DOOR="$(word "$("$P" door "$J" "$W1" "$DT" --offset "4 ft" --sill "0 ft" "${K[@]}")" door)"
"$P" import items "$J" starter >/dev/null

echo "== 1. Tried and taken back =="
BEFORE="$(sum)"; TAKEOFF="$("$P" takeoff "$J")"
OUT="$("$P" try "$J" wall "$LEVEL" "$WT" --points "10,0 10,12" --height "9 ft" "${K[@]}")"
echo "$OUT" | sed 's/^/    | /'
says "what it would make" "it would make wall 5, 12 ft long" "$OUT"
says "that it fits" "it fits: in the same space as nothing, and no hole to cut" "$OUT"
is "the file is byte for byte what it was" "$(sum)" "$BEFORE"
is "takeoff is what it was" "$("$P" takeoff "$J" | sha256sum | cut -c1-16)" "$(sha256sum <<<"$TAKEOFF" | cut -c1-16)"

echo "== 2. Tried where it would clash: a duct through two walls =="
DUCT="$(word "$("$P" type "$J" duct "Duct 14 x 8" --width "14 in" --depth "8 in" "${K[@]}")" type)"
BEFORE="$(sum)"
OUT="$("$P" try "$J" run "$LEVEL" duct "$DUCT" --points "-2,6 22,6" --height "8 ft" "${A[@]}")"
echo "$OUT" | sed 's/^/    | /'
says "two holes" "2 holes to cut" "$OUT"
says "the east wall" "wall 2" "$OUT"
says "the west wall" "wall 4" "$OUT"
says "how long the duct is" "24 ft long" "$OUT"
is "the file is byte for byte what it was" "$(sum)" "$BEFORE"

echo "== 3. Tried where it would be refused: a wall drawn over the south wall =="
refused "lies along wall 1 for 10 ft" "$P" try "$J" wall "$LEVEL" "$WT" --points "2,0 12,0" --height "9 ft" "${K[@]}"  # call 226: a length in a sentence is said as a builder says it
refused "one wall drawn over another would be counted twice" "$P" wall "$J" "$LEVEL" "$WT" --points "2,0 12,0" --height "9 ft" "${K[@]}"
is "the file is byte for byte what it was" "$(sum)" "$BEFORE"

echo "== 4. A door moved along its wall: its middle is what is held =="
OUT="$("$P" try "$J" move "$DOOR" --by "5 ft east" "${A[@]}")"
echo "$OUT" | sed 's/^/    | /'
says "9 ft from the west end" "9 ft from the west end" "$OUT"
says "clear of each end" "7 ft 6 in clear of the west end and 9 ft 6 in clear of the east end" "$OUT"
says "the face is what it was" "wall 1: face less openings stays 160.00 SF" "$OUT"
is "the file is byte for byte what it was" "$(sum)" "$BEFORE"
# 4 + 15 = 19 ft along; the door is 3 ft wide: its east edge at 20 ft 6 in, on a wall 20 ft long: 6 in past (the lead's 2 ft took 4 ft as the near edge).
refused "door 1 would run 6 in past the east end of wall 1" "$P" move "$J" "$DOOR" --by "15 ft east" "${A[@]}"
refused "the wall is 20 ft long" "$P" move "$J" "$DOOR" --by "15 ft east" "${A[@]}"
is "the file is byte for byte what it was" "$(sum)" "$BEFORE"
"$P" move "$J" "$DOOR" --by "5 ft east" "${A[@]}" | sed 's/^/    | /'
"$P" move "$J" "$DOOR" --by "5 ft west" "${A[@]}" >/dev/null
OUT="$("$P" show "$J" "$DOOR")"
says "it is flagged Assumed from the move" "~ center, along the wall" "$OUT"

echo "== 5. A door moved to another wall: each face follows =="
OUT="$("$P" move "$J" "$DOOR" --to "$W3" --offset "6 ft" "${A[@]}")"
echo "$OUT" | sed 's/^/    | /'
says "the south wall's face grows by 20.00 SF" "wall 1: face less openings was 160.00 SF, now 180.00 SF" "$OUT"
says "the north wall's face shrinks by 20.00 SF" "wall 3: face less openings was 180.00 SF, now 160.00 SF" "$OUT"
says "from the first end of the north wall, which is its east end" "6 ft from the east end" "$OUT"
"$P" check "$J" | grep -q agree || { echo "FAIL: the job does not open"; exit 1; }
"$P" move "$J" "$DOOR" --to "$W1" --offset "4 ft" "${A[@]}" >/dev/null

echo "== 6. A copy is Assumed =="
LAV="$(word "$("$P" place "$J" "$LEVEL" Lavatory --at "5,5" --offset "0 ft" "${K[@]}")" fixture)"
OUT="$("$P" copy "$J" "$LAV" --by "3 ft east")"
echo "$OUT" | sed 's/^/    | /'
says "a copy" "Lavatory 2 is a copy of Lavatory" "$OUT"
COPY="$(word "$OUT" fixture)"
SHOWN="$("$P" show "$J" "$COPY")"
says "its place is flagged Assumed" "~ position" "$SHOWN"
says "and so is its size" "~ size" "$SHOWN"
not_said "the original stays Known: unflagged" "~ position" "$("$P" show "$J" "$LAV")"
FOUND="$("$P" find "$J" lavatory)"
says "two lavatories" "Lavatory 2: fixture, on Ground" "$FOUND"
refused "a copy is Assumed" "$P" copy "$J" "$LAV" --status Known --by "3 ft east"

echo "== 7. A row of copies =="
"$P" remove "$J" "$COPY" >/dev/null
is "the copy of step 6 is taken back: one lavatory" "$(grep -c 'fixture, on Ground' <<<"$("$P" find "$J" lavatory)")" "1"
cp "$J" "$T/one.json"
OUT="$("$P" repeat "$J" "$LAV" --times 4 --apart "3 ft east")"
echo "$OUT" | sed 's/^/    | /'
says "four copies" "4 copies, each 3 ft east farther than the last" "$OUT"
FOUND="$("$P" find "$J" lavatory)"
is "five lavatories set down" "$(grep -c 'fixture, on Ground' <<<"$FOUND")" "5"
for n in 2 3 4 5; do says "Lavatory $n" "Lavatory $n: fixture, on Ground" "$FOUND"; done
for n in 2 3 4 5; do P5="$("$P" show "$J" "$(grep -oE 'fixture-[0-9]+' <<<"$OUT" | sed -n "$((n-1))p")")"; says "Lavatory $n is at x = $((5 + 3 * (n - 1)))" "($((5 + 3 * (n - 1))), 5) ft" "$P5"; done
# Six, tried from the one lavatory: the sixth stands at (23, 5), outside the house, and is made; the fifth, at (20, 5), is on the east wall's line.
ONE="$(sha256sum <"$T/one.json" | cut -d' ' -f1)"
OUT="$("$P" try "$T/one.json" repeat "$LAV" --times 6 --apart "3 ft east")"
echo "$OUT" | sed 's/^/    | /'
says "six would be made" "Lavatory 7, 20 × 18 × 34 in" "$OUT"
says "the one on the east wall's line" "Lavatory 6 would be in the same space as wall 2" "$OUT"
not_said "the sixth, outside the house, is in nothing" "Lavatory 7 would be in the same space" "$OUT"
is "the file is byte for byte what it was" "$(sha256sum <"$T/one.json" | cut -d' ' -f1)" "$ONE"

echo "== 8. A wall copied with its door =="
refused "lies along wall 3 for 20 ft" "$P" copy "$J" "$W1" --by "12 ft north"  # call 226: a length in a sentence is said as a builder says it
OUT="$("$P" copy "$J" "$W1" --by "6 ft north")"
echo "$OUT" | sed 's/^/    | /'
says "the wall and its door" "with its 1 door or window" "$OUT"
"$P" check "$J" | grep -q agree || { echo "FAIL: the job does not open"; exit 1; }

echo "== 9. Find =="
FOUND="$("$P" find "$J" lav)"
echo "$FOUND" | sed 's/^/    | /'
says "the item and the things set down" "Lavatory: item in the tool chest" "$FOUND"
OUT="$("$P" find "$J" south)"
says "nothing, and says so" "nothing in the job is called \"south\"" "$OUT"
OUT="$("$P" find "$J" ground)"
says "a level" "Ground: level" "$OUT"

echo "== 10. Nothing of the sample whole house moves =="
"$P" sixteen "$T/a.json" >/dev/null
"$P" sixteen "$T/b.json" >/dev/null
is "the sample whole house is the same bytes twice" "$(sha256sum <"$T/a.json" | cut -d' ' -f1)" "$(sha256sum <"$T/b.json" | cut -d' ' -f1)"
A_SUM="$(sha256sum <"$T/a.json" | cut -d' ' -f1)"
SW="$(grep -oE '"wall-[0-9]+"' "$T/a.json" | head -1 | tr -d '"')"
SD="$(grep -oE '"door-[0-9]+"' "$T/a.json" | head -1 | tr -d '"')"
OUT="$("$P" try "$T/a.json" copy "$SW" --by "0,-40")"
says "the sample house, a wall copied, tried" "nothing was changed" "$OUT"
OUT="$("$P" try "$T/a.json" move "$SW" --by "1 ft east" "${A[@]}")"
says "the sample house, a wall moved, tried" "nothing was changed" "$OUT"
OUT="$("$P" try "$T/a.json" move "$SD" --by "0.5 ft east" "${A[@]}" 2>&1 || true)"
echo "$OUT" | head -3 | sed 's/^/    | /'
"$P" find "$T/a.json" wall >/dev/null
is "the sample whole house after a try and a find" "$(sha256sum <"$T/a.json" | cut -d' ' -f1)" "$A_SUM"
echo "tools-feel: every step passed"
