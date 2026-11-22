#!/usr/bin/env bash
# A wall's frame counted as a framer counts it (call 137): studs by the piece, plates by
# the foot and in whole 16 ft pieces, headers in two plies, rough sills, from the frame
# the window draws, on two walls given the starter library's lap-siding wall; each figure
# against the one worked by hand in STATUS.md ("A wall's frame counted").
# Run from the repository root:  demos/framing.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }

echo "== Two 9 ft walls: a plain 12 ft one, and 24 ft with a door centred at 6 ft and a window at 15 ft 6 in =="
"$P" new "$J" "Framing (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 by 7" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 4 by 3" --width "4 ft" --height "3 ft" "${K[@]}")")"
PLAIN="$("$P" wall "$J" "$S" "$WT" --points "0,20 12,20" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
OPEN="$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" door "$J" "$OPEN" "$DT" --offset "6 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" window "$J" "$OPEN" "$NT" --offset "15 ft 6 in" --sill "3 ft" "${K[@]}" >/dev/null
"$P" check "$J"
"$P" show "$J" > "$T/show.txt"; block "$OPEN" > "$T/open.txt"
if grep -q 'studs ' "$T/open.txt"; then echo "FAIL: a wall nothing names a framing for has studs"; exit 1; fi
echo "    nothing priced names a framing yet: no studs are counted"

echo
echo "== Each given the starter library's lap-siding wall =="
"$P" import assemblies "$J" starter > "$T/in.txt"
says "$T/in.txt" "set: 37"
for w in "$PLAIN" "$OPEN"; do "$P" apply "$J" "$w" "Exterior wall, 2x6, lap siding" --source "made-up plan" >/dev/null; done
"$P" show "$J" > "$T/show.txt"; block "$PLAIN" > "$T/plain.txt"; block "$OPEN" > "$T/open.txt"

echo
echo "== The plain 12 ft wall =="
says "$T/plain.txt" "studs            10 EA"
says "$T/plain.txt" "with waste: Studs, 2x6 at 16 in 11 EA"
says "$T/plain.txt" "quantity: Plates, 2x6 36 LF"
says "$T/plain.txt" "to order: Plates, 2x6 3 piece"
says "$T/plain.txt" "quantity: Headers, two plies 0 LF"

echo
echo "== The 24 ft wall with its door and window =="
says "$T/open.txt" "studs            29 EA"
says "$T/open.txt" "with waste: Studs, 2x6 at 16 in 32 EA"
says "$T/open.txt" "quantity: Plates, 2x6 72 LF"
says "$T/open.txt" "to order: Plates, 2x6 5 piece"
says "$T/open.txt" "header length    7.5 LF"
says "$T/open.txt" "quantity: Headers, two plies 15 LF"
says "$T/open.txt" "quantity: Rough sills, 2x6 4 LF"
says "$T/open.txt" "~ waste: Studs, 2x6 at 16 in 10 %"

echo
echo "== By item: 43 studs and 8 plates for the two =="
"$P" estimate "$J" by-item | tr -s ' ' > "$T/items.txt"
says "$T/items.txt" "Studs, 2x6 at 16 in Material 2 ~43 EA"
echo
echo "  PASS: two walls were counted as a framer counts them, studs, plates, headers and sills, to the figures worked by hand"
