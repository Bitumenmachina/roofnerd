#!/usr/bin/env bash
# Drains on a low-slope roof and its tapered insulation laid out to them (call 139): from
# an empty job, a 48 ft square low-slope area at 1/4:12, one drain in its middle (NRCA's
# figure 4-11), the insulation's thickest, average, board feet and valleys worked out by
# roofnerd's solver to the figures worked by hand; priced by the board foot and the
# drain; then two drains, and the figures and the price move; then what is refused.
# Run from the repository root:  demos/taper.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by
# hand in STATUS.md ("Drains and the tapered insulation on a low-slope area").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { local file="$1"; shift; if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; cmp -s "$J" "$file" || { echo "FAIL: a refusal changed the job: $*"; exit 1; }; }
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }

echo "== 1. An empty job, and a low-slope roof 48 ft square at 1/4:12, its deck 11 ft up =="
"$P" new "$J" "Flat roof (not a real job)" >/dev/null
"$P" low-slope "$J" "Main roof" --points "0,0 48,0 48,48 0,48" --pitch 1/4:12 --height "11 ft" --status Known --source "made-up roof plan" > "$T/out.txt"
says "$T/out.txt" "a low-slope area, \"Main roof\", at 1/4:12"
A="$(grep -oE 'lowslope-[0-9]+' "$T/out.txt" | head -1)"
"$P" show "$J" > "$T/show.txt"; block "$A" > "$T/area.txt"
# 48 x 48 = 2,304 SF. No drains: no taper, and nothing flagged for it.
says "$T/area.txt" "plan area        2304 SF"
lacks "$T/area.txt" "board feet"
"$P" line "$J" "$A" "Tapered polyiso insulation, 1/4 in per ft" board_feet --division 7 >/dev/null
"$P" line "$J" "$A" "Roof drain, cast iron, set" drains --division 15 >/dev/null
"$P" show "$J" > "$T/show.txt"
says "$T/show.txt" "the line's quantity uses board feet, and no drains are placed on this low-slope area"

echo
echo "== 2. One drain in the middle, the insulation 1/2 in thick at it (NRCA fig. 4-11) =="
"$P" drains "$J" "$A" --at "24,24" --at-drain "1/2 in" --status Assumed --source "made-up: the designer's minimum" > "$T/out.txt"
says "$T/out.txt" "1 drain, the insulation 1/2 in at a drain"
"$P" show "$J" > "$T/show.txt"; block "$A" > "$T/area.txt"
says "$T/area.txt" "~ drains           1 EA"
says "$T/area.txt" "~ thinnest         0.5 in"
# Every edge is 24 ft from the drain square along the boards: 0.5 + 0.25 x 24 = 6.5 in.
says "$T/area.txt" "~ thickest         6.5 in"
# Four triangles, each 4.5 in thick at its middle (16 ft out): 2,304 x 4.5 = 10,368 BF.
says "$T/area.txt" "~ average thickness 4.5 in"
says "$T/area.txt" "~ board feet       10368 BF"
# Four valleys, drain to corner: 4 x 24 x sqrt(2) = 135.7645 LF.
says "$T/area.txt" "~ valley length    135.7645 LF"
"$P" set "$J" "$A" unit_cost '$0.95/BF' --line "Tapered polyiso insulation, 1/4 in per ft" --status Known --source "made-up price" >/dev/null
"$P" set "$J" "$A" unit_cost '$640/EA' --line "Roof drain, cast iron, set" --status Known --source "made-up price" >/dev/null
"$P" estimate "$J" | tr -s ' ' > "$T/est.txt"
# 10,368 x $0.95 = $9,849.60; one drain $640.00.
says "$T/est.txt" "\$9,849.60"
says "$T/est.txt" "\$640.00"

echo
echo "== 3. Two drains, 12,24 and 36,24: two regions 24 ft by 48 ft =="
"$P" drains "$J" "$A" --at "12,24 36,24" --at-drain "1/2 in" --status Assumed --source "made-up: the designer's minimum" > "$T/out.txt"
says "$T/out.txt" "2 drains in place of the 1 drain it had"
"$P" show "$J" > "$T/show.txt"; block "$A" > "$T/area.txt"
says "$T/area.txt" "~ drains           2 EA"
# What the thickest rests on is said where a person reads it (call 140).
says "$T/show.txt" "thickest: 6.5 in (nothing caps the build-up yet"
says "$T/area.txt" "~ thickest         6.5 in"
# The average run over a 12 by 24 quarter region is 3,744 / 288 = 13 ft: 0.5 + 0.25 x 13.
says "$T/area.txt" "~ average thickness 3.75 in"
says "$T/area.txt" "~ board feet       8640 BF"
# Eight valleys, each 12 ft square of the drain: 8 x 12 x sqrt(2).
says "$T/area.txt" "~ valley length    135.7645 LF"
"$P" estimate "$J" | tr -s ' ' > "$T/est.txt"
# 8,640 x $0.95 = $8,208.00; two drains $1,280.00.
says "$T/est.txt" "\$8,208.00"
says "$T/est.txt" "\$1,280.00"

echo
echo "== 4. Refused, each in words, the job unchanged =="
cp "$J" "$T/before.json"
refused "$T/before.json" "$P" drains "$J" "$A" --at "60,24" --at-drain "1/2 in" --status Assumed --source "made-up"
says "$T/out.txt" "the drain at 60,24 is not on $A: a drain goes inside the area's outline"
refused "$T/before.json" "$P" drains "$J" "$A" --at "12,24 12,24" --at-drain "1/2 in" --status Assumed --source "made-up"
says "$T/out.txt" "the drain at 12,24 is given twice"
"$P" roof "$J" "Porch" --points "60,0 90,0 90,20 60,20" --pitch 6:12 --eave-height "9 ft" --status Known --source "made-up roof plan" >/dev/null
cp "$J" "$T/before.json"
PLANE="$("$P" show "$J" | grep -oE 'plane-[0-9]+' | head -1)"
refused "$T/before.json" "$P" drains "$J" "$PLANE" --at "70,10" --at-drain "1/2 in" --status Assumed --source "made-up"
says "$T/out.txt" "$PLANE is a roof plane; only a low-slope area takes roof drains"
refused "$T/before.json" "$P" line "$J" "$A" "Insulation by thickness" "thickest * plan_area"
says "$T/out.txt" "uses thickest, which is a thickness, in inches; a formula works in lengths, areas and counts"
refused "$T/before.json" "$P" low-slope "$J" "Steep" --points "0,60 10,60 10,70 0,70" --pitch 6:12 --height "9 ft" --status Known --source "made-up"
says "$T/out.txt" "a pitch of 6:12 is a steep roof, not a low-slope one"

echo
echo "== 5. The same roof priced from the starter library's low-slope assembly =="
cp "$J" "$T/lib.json"
"$P" drains "$T/lib.json" "$A" --at "24,24" --at-drain "1/2 in" --status Assumed --source "made-up: the designer's minimum" >/dev/null
"$P" import assemblies "$T/lib.json" starter >/dev/null
"$P" apply "$T/lib.json" "$A" "Low-slope roof, tapered insulation, single-ply membrane" --source "made-up roof plan" --change >/dev/null
"$P" show "$T/lib.json" > "$T/show.txt"; block "$A" > "$T/area.txt"
# 10,368 BF x 1.05 = 10,886.4 BF.
says "$T/area.txt" "with waste: Tapered polyiso insulation 10886.4 BF"
# 2,304 SF x sqrt(1 + (0.25/12)^2) = 2,304.4999 SF; x 1.05 = 2,419.7249; / 32 = 75.6: 76 sheets.
says "$T/area.txt" "to order: Cover board, 1/2 in 76 sheet"
# 2,304.4999 x 1.10 = 2,534.9499 SF; / 1,000: 3 rolls.
says "$T/area.txt" "to order: TPO membrane, 60 mil 3 roll"
says "$T/area.txt" "quantity: Roof drain, set 1 EA"
says "$T/area.txt" "quantity: Lay the insulation and membrane 23.045 SQ"

echo
echo "== 6. Saved and read back: the same figures, nothing stale =="
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "its figures agree with what is drawn, and saving it back writes the same file"
echo
echo "  PASS: from an empty job, a low-slope roof was given its drains and its tapered insulation laid out to them on NRCA's four-way layout, thickest, average, board feet and valleys to the figures worked by hand, priced by the board foot and the drain, and moved when a second drain went in; a drain off the roof, drains on a steep plane, a thickness in a formula and a steep low-slope roof were refused in words"
