#!/usr/bin/env bash
# The made-up job the two field-entry demonstrations start from, written as a tracing
# would leave it. Used by demos/field.sh and demos/fieldw.sh:  demos/field-job.sh <job file>
# It prints the paths the demonstrations use, as shell assignments.
#
# A ground storey at 0 ft, 9 ft to the next, and an upper storey at 9 ft: Assumed, scaled
# off a plan. Four 6 in walls round 24 ft by 16 ft, Known, standing to the upper storey.
# A 12 in by 8 in duct along the middle, its centre 8 ft above the floor, traced. Two 2 in
# pipes across it, Known: pipe A's centre at 7 ft 9 in, pipe B's at 8 ft. The south wall
# carries gypsum board on its net face at $2.40/SF. Overhead 10, profit 8.
set -euo pipefail
J="$1"
P="$(cd "$(dirname "$0")/.." && pwd)/target/debug/plumbline"
K=(--status Known --source "made-up plan")
A=(--status Assumed --source "scaled off a made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1"; }
"$P" new "$J" "Measured in the field (not a real job)" >/dev/null
GROUND="$(made "$("$P" storey "$J" ground --elevation "0 ft" --height "9 ft" "${A[@]}")")"
UPPER="$(made "$("$P" storey "$J" upper --elevation "9 ft" --height "8 ft" "${A[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" duct "12 x 8" --width "12 in" --depth "8 in" "${K[@]}")")"
PT="$(made "$("$P" type "$J" pipe "2 in" --round "2 in" "${K[@]}")")"
WALL="$("$P" wall "$J" "$GROUND" "$WT" --points "0,0 24,0 24,16 0,16" --closed --top "$UPPER" "${K[@]}" | sed -E 's/^[^:]*: //; s/,.*//')"
DUCT="$("$P" run "$J" "$GROUND" duct "$DT" --points "2,8 22,8" --height "8 ft" "${A[@]}" | sed -E 's/^[^:]*: //')"
PIPE_A="$("$P" run "$J" "$GROUND" pipe "$PT" --points "12,2 12,14" --height "7 ft 9 in" "${K[@]}" | sed -E 's/^[^:]*: //')"
PIPE_B="$("$P" run "$J" "$GROUND" pipe "$PT" --points "18,2 18,14" --height "8 ft" "${K[@]}" | sed -E 's/^[^:]*: //')"
"$P" line "$J" "$WALL" "Gypsum board, hung and finished" net_area --class material >/dev/null
"$P" set "$J" "$WALL" unit_cost '$2.40/SF' --line "Gypsum board, hung and finished" "${K[@]}" >/dev/null
"$P" set "$J" job-0001 overhead 10 "${K[@]}" >/dev/null
"$P" set "$J" job-0001 profit 8 "${K[@]}" >/dev/null
echo "GROUND=$GROUND UPPER=$UPPER WALL=$WALL DUCT=$DUCT PIPE_A=$PIPE_A PIPE_B=$PIPE_B DT=$DT PT=$PT"
