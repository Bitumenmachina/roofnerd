#!/usr/bin/env bash
# Field entry, at the command line: a job as a tracing leaves it, two clashes that are
# questions and a wall whose price is flagged; then what a veteran measures on site, typed
# the way a laser in keyboard mode types it, and what each measurement does: the floor
# above moves and the price with it, one clash clears, the other becomes a finding.
# Run from the repository root:  demos/field.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures expected are
# worked by hand in the comments, and in STATUS.md ("Field entry").
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
not_said() { ! grep -qF -- "$2" "$1" || { echo "FAIL: it says: $2"; cat "$1"; exit 1; }; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | head -1 | cut -c1-230
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
  cmp -s "$J" "$T/kept.json" || { echo "FAIL: a refusal changed the file"; exit 1; }
}
clashes() { "$P" clashes "$J" > "$T/clashes.txt"; }
name() { "$P" show "$J" | sed -n "s/^ *$1  \(.*\)  \[.*/\1/p" | head -1; }   # what the job calls a thing
sheet() { "$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"; }

echo "== The job, as a tracing leaves it =="
eval "$(demos/field-job.sh "$J")"
clashes; sheet
# The duct is 8 in deep about 8 ft: 7 ft 8 in to 8 ft 4 in. Pipe A, 2 in about 7 ft 9 in,
# is 7 ft 8 in to 7 ft 10 in: in the duct by 2 in. Pipe B, about 8 ft, is wholly inside
# the duct's depth: 5 in out, up or down. Both rest on what was traced.
says "$T/clashes.txt" "In the same space: 2 (2 to verify in the field)"
says "$T/clashes.txt" "~ $(name "$DUCT") [$DUCT] and $(name "$PIPE_A") [$PIPE_A]: in each other by 2.00 in"
says "$T/clashes.txt" "~ $(name "$DUCT") [$DUCT] and $(name "$PIPE_B") [$PIPE_B]: in each other by 5.00 in"
# The south wall: 24 ft by 9 ft, 216 SF, at $2.40: $518.40, flagged. Overhead $51.84;
# profit 8% of $570.24, $45.62; price $615.86.
says "$T/sheet.txt" "Gypsum board, hung and finished ~216.00 SF \$2.40/SF ~\$518.40"
says "$T/sheet.txt" "PRICE ~\$615.86"

echo
echo "== What a laser types is a length =="
# A fraction after the inch mark, as the display shows it. Each is tried on a copy, on
# the upper level's height, which nothing else here rests on.
cp "$J" "$T/try.json"
for pair in "2'2\"13/32=2' 2 13/32\"" "26\"13/32=26 13/32 in" "23.46ft=23.46 ft" "0'10\"1/2=0' 10 1/2\""; do
  typed="${pair%%=*}"; want="${pair#*=}"
  OUT="$("$P" measure "$T/try.json" "$UPPER" "level height" "$typed" --source "tried")"
  grep -qF -- "level height is $want, measured" <<<"$OUT" || { echo "FAIL: $typed was not read as $want: $OUT"; exit 1; }
  printf '    %-12s is read as %s\n' "$typed" "$want"
done

echo
echo "== What is refused, and the file is byte for byte what it was after each =="
cp "$J" "$T/kept.json"
refused "has no unit" "$P" measure "$J" "$GROUND" "level height" "8.875" --source "the laser, set to send no unit"
refused "has a comma" "$P" measure "$J" "$GROUND" "level height" "8,875 ft" --source "the laser, set to a comma"
refused "metric" "$P" measure "$J" "$GROUND" "level height" "2.705m" --source "the laser, set to meters"
refused "one or the other" "$P" measure "$J" "$GROUND" "level height" "8' 10 1/2\" 1/4" --source x
refused "not one length" "$P" measure "$J" "$WALL" "wall line" "24 ft" --source x
refused "it has: type, wall line, top level" "$P" measure "$J" "$WALL" "length" "24 ft" --source x
refused "a measurement is Known" "$P" measure "$J" "$GROUND" "level height" "9 ft" --status Assumed --source x
refused "who measured" "$P" measure "$J" "$GROUND" "level height" "9 ft"
refused "not a run" "$P" measure "$J" "$WALL" "wall line" same --to top --source x
refused "leave the line out" "$P" measure "$J" "$DUCT" "height above its level" same --to bottom --source x

echo
echo "== The floor everything is measured from, and floor to floor =="
"$P" measure "$J" "$GROUND" elevation same --source "JM on site" | sed 's/^/    /'
# 8 ft 10 1/2 in is 8.875 ft. The upper floor is a Known floor plus a Known measurement.
"$P" measure "$J" "$GROUND" "level height" "8'10\"1/2" --source "JM with the laser" | tee "$T/out.txt" | sed 's/^/    /'
says "$T/out.txt" "upper's floor is now at 8' 10 1/2\", Known; it was at 9' 0\""
sheet
# 24 ft by 8.875 ft, 213 SF, $511.20, not flagged. Overhead $51.12; profit 8% of
# $562.32, $44.99; price $607.31: $8.55 less.
says "$T/sheet.txt" "Gypsum board, hung and finished 213.00 SF \$2.40/SF \$511.20"
says "$T/sheet.txt" "PRICE \$607.31"
# A second shot that contradicts a floor now Known above is refused, and says which three.
cp "$J" "$T/kept.json"
"$P" measure "$J" "$UPPER" elevation same --source "JM on site" >/dev/null
cp "$J" "$T/kept.json"
refused "One of the three is wrong" "$P" measure "$J" "$GROUND" "level height" "9 ft" --source "a second shot"

echo
echo "== The duct: where it is, what it is, and its underside shot from the floor =="
"$P" measure "$J" "$DUCT" "line on the plan" same --source "JM on site" | sed 's/^/    /'
"$P" measure "$J" "$DUCT" type same --source "JM on site" | sed 's/^/    /'
clashes
says "$T/clashes.txt" "In the same space: 2 (2 to verify in the field)"
"$P" measure "$J" "$DUCT" "height above its level" "7'10\"3/4" --to bottom --source "JM with the laser" | sed 's/^/    /'
clashes
# The duct is now 7 ft 10 3/4 in to 8 ft 6 3/4 in. Pipe A's top is at 7 ft 10 in: 3/4 in
# clear, and off the list. Pipe B, 7 ft 11 in to 8 ft 1 in, is still inside: 2 1/4 in
# down to the duct's underside. Everything it rests on is Known: a finding.
says "$T/clashes.txt" "In the same space: 1 (1 finding)"
says "$T/clashes.txt" "  $(name "$DUCT") [$DUCT] and $(name "$PIPE_B") [$PIPE_B]: in each other by 2.25 in"
not_said "$T/clashes.txt" "$PIPE_A"
"$P" clashes "$J" --clearance "2 in" > "$T/near.txt"
says "$T/near.txt" "Closer than 2.00 in: 1 (1 finding)"
says "$T/near.txt" "$(name "$DUCT") [$DUCT] and $(name "$PIPE_A") [$PIPE_A]: 0.75 in apart"

echo
echo "== Read back =="
"$P" check "$J" | tail -1 | sed 's/^/    /'
"$P" show "$J" > "$T/show.txt"
says "$T/show.txt" "level height     8' 10 1/2\""
says "$T/show.txt" "height above its level 7' 10 3/4\""
says "$T/show.txt" "its heights are those of its bottom"
[ -n "${KEEP:-}" ] && cp "$J" "$KEEP"
echo
echo "field entry: PASS"
