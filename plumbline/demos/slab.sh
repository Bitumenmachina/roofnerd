#!/usr/bin/env bash
# A slab on grade bought by its own thickness (call 145): from an empty job, a floor 40 ft
# by 28 ft of a 4 in slab type, the starter library's "Slab on grade, by its thickness",
# its concrete, base, vapor retarder and edge forms to the figures worked by hand; then
# the slab measured at 6 in, and what moves, against the old 4 in assembly that does not.
# Run from the repository root:  demos/slab.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by
# hand in STATUS.md ("A slab on grade bought by its own thickness").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }

echo "== A floor 40 ft by 28 ft, a 4 in slab, priced by its own thickness =="
"$P" new "$J" "Slab (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
FT="$(made "$("$P" type "$J" floor "slab" --thickness "4 in" "${K[@]}")")"
F="$(made "$("$P" floor "$J" "$S" "$FT" --points "0,0 40,0 40,28 0,28" "${K[@]}")")"
"$P" import assemblies "$J" starter >/dev/null
cp "$J" "$T/old.json"
"$P" apply "$J" "$F" "Slab on grade, by its thickness" --source "made-up plan" >/dev/null
"$P" show "$J" > "$T/show.txt"; block "$F" > "$T/floor.txt"
says "$T/floor.txt" "plan area        1120 SF"
says "$T/floor.txt" "perimeter        136 LF"
# 1,120 x 4/12 = 373.333 CF = 13.8272 CY; with 5 %, 13.8272 x 1.05 = 14.5186 (the
# quantity is written to four places before its waste is added: QUESTIONS.md Q29).
says "$T/floor.txt" "quantity: Slab concrete 13.8272 CY"
says "$T/floor.txt" "with waste: Slab concrete 14.5186 CY"
says "$T/floor.txt" "quantity: Base course, gravel 13.8272 CY"
# 1,120 with 10 % = 1,232 SF; 136 x 4/12 = 45.3333 SF.
says "$T/floor.txt" "with waste: Vapor retarder, 10 mil 1232 SF"
says "$T/floor.txt" "quantity: Slab edge forms 45.3333 SF"
says "$T/floor.txt" "~ base: Base course, gravel 4 in"

echo
echo "== The slab measured at 6 in: what is bought by its thickness follows =="
"$P" measure "$J" "$FT" "floor thickness" "6 in" --source "made-up: the structural drawing" >/dev/null
"$P" show "$J" > "$T/show.txt"; block "$F" > "$T/floor.txt"
# 1,120 x 6/12 = 560 CF = 20.7407 CY; with 5 %, 20.7407 x 1.05 = 21.7777 (Q29).
# Forms 136 x 1/2 = 68 SF.
says "$T/floor.txt" "quantity: Slab concrete 20.7407 CY"
says "$T/floor.txt" "with waste: Slab concrete 21.7777 CY"
says "$T/floor.txt" "quantity: Slab edge forms 68 SF"
says "$T/floor.txt" "quantity: Base course, gravel 13.8272 CY"

echo
echo "== The old 4 in assembly on the same 6 in slab: it does not follow, which is why the new one is =="
"$P" apply "$T/old.json" "$F" "Slab on grade, 4 in" --source "made-up plan" >/dev/null
"$P" measure "$T/old.json" "$FT" "floor thickness" "6 in" --source "made-up: the structural drawing" >/dev/null
"$P" show "$T/old.json" > "$T/show.txt"; block "$F" > "$T/old.txt"
# 1,120 / 81 = 13.8272 CY whatever the thickness.
says "$T/old.txt" "to order: Concrete, 4 in slab 14 CY"
echo
echo "  PASS: a slab was bought by its own thickness, its concrete, base, vapor retarder and edge forms to the figures worked by hand, and followed the slab when it was measured at 6 in"
