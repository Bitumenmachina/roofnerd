#!/usr/bin/env bash
# A shingle roof priced whole, as a roofer takes it off: the roof's own figures from its
# planes and lines (call 134), and the starter library's "Asphalt shingle roof, whole
# roof" (call 135) giving sheathing, underlayment, ice barrier, shingles, starter, cap
# and drip edge in their packs, each to the figure worked by hand; the pipe boots waiting
# for a count; two prices typed.
# Run from the repository root:  demos/shingle.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by
# hand in STATUS.md ("A shingle roof priced whole") and in the comments here.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }

echo "== A hip roof over 42 ft by 30 ft, 6:12 all round =="
"$P" new "$J" "Shingle roof (not a real job)" >/dev/null
"$P" roof "$J" "Main roof" --points "-1,-1 41,-1 41,29 -1,29" --pitch 6:12 --eave-height "9 ft" --status Known --source "made-up roof plan" >/dev/null
R="$("$P" show "$J" | grep -oE 'roof-[0-9]+' | head -1)"
"$P" check "$J"

echo
echo "== The roof's own figures, from its planes and lines =="
"$P" show "$J" > "$T/show.txt"; block "$R" > "$T/roof.txt"
# 42 x 30 = 1,260 SF on plan; x sqrt(1.25) = 1,408.7228 SF; 14.0872 SQ.
says "$T/roof.txt" "surface area     1408.7228 SF"
says "$T/roof.txt" "squares          14.0872 SQ"
# Eaves 144; ridge 12; four hips of 22.5 ft: 90.
says "$T/roof.txt" "eave length      144 LF"
says "$T/roof.txt" "ridge length     12 LF"
says "$T/roof.txt" "hip length       90 LF"
says "$T/roof.txt" "valley length    0 LF"

echo
echo "== The whole roof given its assembly from the starter library =="
"$P" import assemblies "$J" starter > "$T/in.txt"
says "$T/in.txt" "set: 34"
"$P" apply "$J" "$R" "Asphalt shingle roof, whole roof" --source "made-up roof plan" >/dev/null
"$P" show "$J" > "$T/show.txt"; block "$R" > "$T/roof.txt"
says "$T/roof.txt" "with waste: Roof sheathing 1549.5951 SF"
says "$T/roof.txt" "to order: Roof sheathing 49 sheet"
says "$T/roof.txt" "with waste: Underlayment, synthetic 15.4959 SQ"
says "$T/roof.txt" "to order: Underlayment, synthetic 2 roll"
says "$T/roof.txt" "quantity: Ice barrier, eaves and valleys 432 SF"
says "$T/roof.txt" "to order: Ice barrier, eaves and valleys 3 roll"
says "$T/roof.txt" "with waste: Field shingles 16.2003 SQ"
says "$T/roof.txt" "to order: Field shingles 49 bundle"
says "$T/roof.txt" "with waste: Starter strip 151.2 LF"
says "$T/roof.txt" "to order: Starter strip 2 bundle"
says "$T/roof.txt" "quantity: Hip and ridge cap 102 LF"
says "$T/roof.txt" "to order: Hip and ridge cap 6 bundle"
says "$T/roof.txt" "to order: Drip edge 15 piece"
says "$T/roof.txt" "quantity: Ridge vent 12 LF"
says "$T/roof.txt" "quantity: Open valley flashing 0 LF"
says "$T/roof.txt" "? quantity: Pipe boots unknown"
# What a rule set is Assumed and says where it came from.
says "$T/roof.txt" "~ waste: Field shingles 15 %"
says "$T/roof.txt" "~ order factor: Field shingles 3 bundle/SQ"

echo
echo "== The boots counted, and two prices typed =="
"$P" set "$J" "$R" boots 3 --line "Pipe boots" --status Known --source "counted on the made-up plan" >/dev/null
"$P" set "$J" "$R" unit_cost '$38/bundle' --line "Field shingles" --status Known --source "made-up price" >/dev/null
"$P" set "$J" "$R" unit_cost '$24/sheet' --line "Roof sheathing" --status Known --source "made-up price" >/dev/null
"$P" estimate "$J" | tr -s ' ' > "$T/est.txt"
says "$T/est.txt" "Pipe boots 3.00 EA"
# 49 x $38 = $1,862.00; 49 x $24 = $1,176.00.
says "$T/est.txt" "\$1,862.00"
says "$T/est.txt" "\$1,176.00"
echo
echo "== A plane given the plane-by-plane assembly as well: counted twice, and said (call 136) =="
cp "$J" "$T/twice.json"
PLANE="$("$P" show "$T/twice.json" | grep -oE 'plane-[0-9]+' | head -1)"
"$P" apply "$T/twice.json" "$PLANE" "Asphalt shingle roof" --source "made-up: by mistake" >/dev/null
"$P" estimate "$T/twice.json" > "$T/twice.txt"
says "$T/twice.txt" "! counted twice: Main roof is priced whole, and plane 1 is priced on its own as well: what both carry (its sheathing, its shingles) is in the price twice; take one of the two off"
echo
echo "  PASS: a hip roof was priced whole from the starter library, its sheathing, underlayment, ice barrier, shingles, starter, cap and drip edge in whole packs to the figures worked by hand, its boots waiting for a count, and a roof priced both whole and by the plane said to be counted twice"
