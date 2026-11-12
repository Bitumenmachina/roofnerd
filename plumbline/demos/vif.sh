#!/usr/bin/env bash
# The verify-in-field list, at the command line: each question the job still asks of
# the building, what to measure or check to settle it, and the dollars riding on it,
# largest first. Then the measurements of the field demonstration, and the list after.
# Run from the repository root:  demos/vif.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures expected are
# worked by hand in STATUS.md ("The verify-in-field list") and in the comments here.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
not_said() { ! grep -qF -- "$2" "$1" || { echo "FAIL: it says: $2"; cat "$1"; exit 1; }; }
row() { grep -nF -- "$2" "$1" | head -1 | cut -d: -f1; }   # which line of the list a row is on
name() { "$P" show "$J" | sed -n "s/^ *$1  \(.*\)  \[.*/\1/p" | head -1; }   # what the job calls a thing

echo "== The job, as a tracing leaves it =="
eval "$(demos/field-job.sh "$J")"
"$P" vif "$J" > "$T/vif.txt"
# The south wall is 24 ft by 9 ft, 216 SF at $2.40: $518.40, flagged. Its height is
# worked from the ground level's floor and the upper level's floor: $518.40 rides on
# each. Nothing is priced on the duct or the pipes. In all, one line, counted once.
says "$T/vif.txt" "Verify in field: 5 questions. ~\$518.40 rides on them (1 line of the estimate, each counted once)."
says "$T/vif.txt" "  ground [$GROUND] (level): ~\$518.40, 1 line"
says "$T/vif.txt" "  upper [$UPPER] (level): ~\$518.40, 1 line"
says "$T/vif.txt" "      measure: ~elevation, ~level height"
says "$T/vif.txt" "  $(name "$DUCT") [$DUCT] and $(name "$PIPE_A") [$PIPE_A] in the same space by 2.00 in, on ground"
says "$T/vif.txt" "  $(name "$DUCT") [$DUCT] and $(name "$PIPE_B") [$PIPE_B] in the same space by 5.00 in, on ground"
says "$T/vif.txt" "  $(name "$DUCT") [$DUCT] (duct), on ground: no dollars riding on it"
says "$T/vif.txt" "      check: ~type, ~line on the plan; measure: ~height above its level"
# Largest first; ties in the job's order; no dollars after, clashes before things.
a="$(row "$T/vif.txt" "  ground [$GROUND] (level)")"; b="$(row "$T/vif.txt" "  upper [$UPPER] (level)")"
c="$(row "$T/vif.txt" "[$PIPE_A] in the same space")"; d="$(row "$T/vif.txt" "[$PIPE_B] in the same space")"
e="$(row "$T/vif.txt" "[$DUCT] (duct)")"
[ "$a" -lt "$b" ] && [ "$b" -lt "$c" ] && [ "$c" -lt "$d" ] && [ "$d" -lt "$e" ] || { echo "FAIL: the list is not in order: $a $b $c $d $e"; cat "$T/vif.txt"; exit 1; }
echo "    in order: the two levels, the two clashes, the duct"
# What is Known asks nothing.
not_said "$T/vif.txt" "[$WALL] (wall)"
not_said "$T/vif.txt" "[$PIPE_A] (pipe)"

echo
echo "== The ground level measured: its floor, and floor to floor =="
"$P" measure "$J" "$GROUND" elevation same --source "JM on site" >/dev/null
"$P" measure "$J" "$GROUND" "level height" "8'10\"1/2" --source "JM with the laser" >/dev/null
"$P" vif "$J" > "$T/vif.txt"
# The ground level is off the list. The wall is 213 SF, $511.20, and Known: the upper
# level still has its own floor-to-floor to verify, and no dollars ride on that.
says "$T/vif.txt" "Verify in field: 4 questions. Nothing priced rides on them."
not_said "$T/vif.txt" "[$GROUND] (level)"
says "$T/vif.txt" "  upper [$UPPER] (level): no dollars riding on it"
says "$T/vif.txt" "      measure: ~level height"

echo
echo "== The duct measured: its line, its type, its underside =="
"$P" measure "$J" "$DUCT" "line on the plan" same --source "JM on site" >/dev/null
"$P" measure "$J" "$DUCT" type same --source "JM on site" >/dev/null
"$P" measure "$J" "$DUCT" "height above its level" "7'10\"3/4" --to bottom --source "JM with the laser" >/dev/null
"$P" vif "$J" > "$T/vif.txt"; "$P" clashes "$J" > "$T/clashes.txt"
# Pipe A is clear. Pipe B in the duct is a finding: it is on the clash list, and a
# finding is not a question.
says "$T/vif.txt" "Verify in field: 1 question. Nothing priced rides on it."
says "$T/vif.txt" "  upper [$UPPER] (level): no dollars riding on it"
not_said "$T/vif.txt" "in the same space"
says "$T/clashes.txt" "In the same space: 1 (1 finding)"

echo
echo "== The upper level's floor-to-floor looked at: nothing left =="
"$P" measure "$J" "$UPPER" "level height" same --source "JM on site" >/dev/null
"$P" vif "$J" > "$T/vif.txt"
says "$T/vif.txt" "Verify in field: nothing. Every value that can be measured is Known, and no clash is a question."

echo
echo "PASS: the verify-in-field list"
