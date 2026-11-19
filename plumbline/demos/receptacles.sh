#!/usr/bin/env bash
# The fewest receptacles a room's walls need (call 147, NEC 210.52(A)): from an empty job,
# rooms of a small house, their floor lines cut at their doors into runs of wall, each
# run one outlet for every 12 ft or part, to the figures worked by hand; a second door
# cuts the run again; a room with no door is one run all the way round.
# Run from the repository root:  demos/receptacles.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by
# hand in STATUS.md ("The fewest receptacles a room's walls need").
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

echo "== A house 24 ft by 16 ft, 6 in walls, a partition with a 3 ft door, a window in the south wall =="
"$P" new "$J" "Receptacles (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${K[@]}")")"
read -r SOUTH _ NORTH _ <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" door "$J" "$PART" "$DT" --offset "8 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "3 ft" "${K[@]}" >/dev/null
WEST="$(made "$("$P" room "$J" "$S" "West room" --at "6,8" "${K[@]}")")"
EAST="$(made "$("$P" room "$J" "$S" "East room" --at "18,8" "${K[@]}")")"
"$P" show "$J" > "$T/show.txt"
block "$WEST" > "$T/west.txt"; block "$EAST" > "$T/east.txt"
# 54 LF round at the faces, less the 3 ft door: one run of 51 ft; 51 / 12 = 4.25: 5.
says "$T/west.txt" "perimeter        54 LF"
says "$T/west.txt" "~ receptacles      5 EA"
# The window does not cut the floor line: the same 5.
says "$T/east.txt" "~ receptacles      5 EA"
# Assumed, and why: what is not drawn, and the rooms the rule is for.
says "$T/show.txt" "fireplaces, stationary appliances and fixed cabinets also break wall space and are not drawn"

echo
echo "== A second door, in the north wall centred at x = 4.5: the west room's floor line is cut twice =="
"$P" door "$J" "$NORTH" "$DT" --offset "19.5 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" show "$J" > "$T/show.txt"; block "$WEST" > "$T/west.txt"
# Its north face is 3 to 6. Runs: from the partition door's top up the east face 6.25,
# along the north face 11.75 to 6, 5.75: 12 ft, 1; from 3 on round, 2.75 + 15.5 + 11.5
# + 6.25 = 36 ft, 3. 4 in all, where the floor line uncut by this door would need 5.
says "$T/west.txt" "~ receptacles      4 EA"

echo
echo "== A room 10 ft by 12 ft with no door: one run all the way round =="
read -r _ <<<"$("$P" wall "$J" "$S" "$WT" --points "40,0 50,0 50,12 40,12" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
SHUT="$(made "$("$P" room "$J" "$S" "Closed room" --at "45,6" "${K[@]}")")"
"$P" show "$J" > "$T/show.txt"; block "$SHUT" > "$T/shut.txt"
# 9.5 by 11.5 at the faces: 42 LF; 42 / 12 = 3.5: 4.
says "$T/shut.txt" "perimeter        42 LF"
says "$T/shut.txt" "~ receptacles      4 EA"
echo
echo "  PASS: each room's floor line was cut at its doors into runs, each run given one receptacle for every 12 ft or part, to the figures worked by hand, and each count says what it leaves out"
