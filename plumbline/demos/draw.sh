#!/usr/bin/env bash
# Drawing demonstration, at the command line: a small house modelled from an empty job,
# one verb at a time, with no file edited by hand. Its walls, openings, rooms, a duct, a
# pipe, a fixture, equipment and a roof; then a wall moved and a window removed.
# Run from the repository root:  demos/draw.sh
# The house is made up (no real job data) and lives in a temporary folder. Every step
# fails the script if the program does not do what the step says. The figures expected
# are worked by hand in the comments.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")

made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1"; }   # the path a verb reports first
figure() { # $1 path, $2 measure: its value in the saved job, or "none"
  python3 - "$J" "$1" "$2" <<'PY'
import sys, json
e = next(x for x in json.load(open(sys.argv[1]))["data"] if x["path"] == sys.argv[2])
d = e["attributes"].get("pl::measure", {}).get(sys.argv[3])
print("none" if d is None or d["value"] is None else ("%s %s" % (d["value"], d["status"])))
PY
}
is() { # $1 what, $2 got, $3 wanted
  [ "$2" = "$3" ] || { echo "FAIL: $1: got $2, wanted $3"; exit 1; }
  printf '    %-58s %s\n' "$1" "$2"
}
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | head -1 | cut -c1-200
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
}

echo "== An empty job, a storey and the types =="
"$P" new "$J" "Drawn house (not a real job)"
S="$(made "$("$P" storey "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 by 7" --width "3 ft" --height "7 ft" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 4 by 3" --width "4 ft" --height "3 ft" "${K[@]}")")"
echo "    storey $S, wall type $WT, door type $DT, window type $NT"
"$P" check "$J"

echo
echo "== Walls: 24 ft by 16 ft outside, closed, 9 ft tall; and a partition down the middle =="
OUT="$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}")"; echo "    $OUT"
read -r SOUTH EAST NORTH WEST <<<"$(sed -E 's/^[^:]*: //; s/,//g' <<<"$OUT")"
OUT="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}")"; echo "    $OUT"
MID="$(sed -E 's/^[^:]*: //' <<<"$OUT")"
# South wall: 24 LF by 9 ft is 216 SF. East: 16 LF, 144 SF.
is "south wall, length" "$(figure "$SOUTH" plan_length)" "24.0 Known"
is "south wall, face" "$(figure "$SOUTH" face_area)" "216.0 Known"
is "east wall, face" "$(figure "$EAST" face_area)" "144.0 Known"

echo
echo "== Doors and windows =="
"$P" door "$J" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${K[@]}"
# A point on the plan: (12, 8) is 8 ft along the partition from its first end at (12, 0).
"$P" door "$J" "$MID" "$DT" --at "12,8" --sill "0 ft" "${K[@]}"
"$P" window "$J" "$EAST" "$NT" --offset "8 ft" --sill "3 ft" "${K[@]}"
"$P" window "$J" "$NORTH" "$NT" --offset "6 ft" --sill "3 ft" "${K[@]}"
W2="$(made "$("$P" window "$J" "$NORTH" "$NT" --offset "18 ft" --sill "3 ft" "${K[@]}")")"
# South 216 - 3x7 = 195. Partition 144 - 21 = 123. East 144 - 4x3 = 132. North 216 - 24 = 192.
is "south wall, net of its door" "$(figure "$SOUTH" net_area)" "195.0 Known"
is "partition, net of its door" "$(figure "$MID" net_area)" "123.0 Known"
is "east wall, net of its window" "$(figure "$EAST" net_area)" "132.0 Known"
is "north wall, net of two windows" "$(figure "$NORTH" net_area)" "192.0 Known"

echo
echo "== Rooms: a point in each; the walls round it make it =="
R1="$(made "$("$P" room "$J" "$S" "west room" --at "6,8" "${K[@]}")")"
R2="$(made "$("$P" room "$J" "$S" "east room" --at "18,8" "${K[@]}")")"
# Inside the 6 in walls: west from x = 0.25 to 11.75, 11.5 ft; y from 0.25 to 15.75, 15.5 ft.
# 11.5 x 15.5 = 178.25 SF, and 2 x (11.5 + 15.5) = 54 LF round. The east room is the same.
is "west room, floor" "$(figure "$R1" floor_area)" "178.25 Known"
is "west room, round" "$(figure "$R1" perimeter)" "54.0 Known"
is "east room, floor" "$(figure "$R2" floor_area)" "178.25 Known"

echo
echo "== A duct, a pipe, a fixture and equipment, each on its system =="
DUCT="$(made "$("$P" type "$J" duct "6 in round" --round "6 in" "${K[@]}")")"
PIPE="$(made "$("$P" type "$J" pipe "3/4 in" --round "3/4 in" "${K[@]}")")"
AIR="$(made "$("$P" system "$J" "Supply air" "Supply Air")")"
COLD="$(made "$("$P" system "$J" "Cold water" "Domestic Cold Water")")"
"$P" run "$J" "$S" duct "$DUCT" --points "2,2 10,2 10,14" --height "8 ft" --system "$AIR" "${K[@]}"
"$P" run "$J" "$S" pipe "$PIPE" --points "13,1 13,15" --height "1 ft" --system "$COLD" "${K[@]}"
"$P" place "$J" "$S" fixture "Lavatory" --at "13,15" --offset "0 ft" --size "2,1.5,3" --system "$COLD" "${K[@]}"
"$P" place "$J" "$S" equipment "Air handler" --at "2,2" --offset "0 ft" --size "2,2,4" --system "$AIR" "${K[@]}"
"$P" check "$J"
"$P" takeoff "$J" | tr -s ' ' > "$T/takeoff.txt"
says() { grep -qF -- "$2" "$T/takeoff.txt" || { echo "FAIL: $1: the takeoff does not say: $2"; cat "$T/takeoff.txt"; exit 1; }; printf '    %s\n' "$2"; }
# Walls: 24 + 16 + 24 + 16 + 16 = 96 LF. Net: 195 + 132 + 192 + 144 (west) + 123 = 786 SF.
# Rooms: 178.25 + 178.25 = 356.5 SF. Duct: 8 + 12 = 20 LF. Pipe: 14 LF.
says "the walls" "96.00 LF"
says "their net face" "786.00 SF"
says "the rooms" "356.50 SF"
says "the duct" "20.00 LF"
says "the pipe" "14.00 LF"

echo
echo "== A roof over it, solved from its footprint =="
"$P" roof "$J" roof --points "0,0 24,0 24,16 0,16" --pitch 6:12 --eave-height "9 ft" "${K[@]}" | head -2
"$P" check "$J"

echo
echo "== The partition moved a foot east, and a window taken out =="
"$P" move "$J" "$MID" --by "1,0" "${K[@]}"
# West room now 0.25 to 12.75: 12.5 x 15.5 = 193.75. East: 13.25 to 23.75, 10.5 x 15.5 = 162.75.
is "west room, floor" "$(figure "$R1" floor_area)" "193.75 Known"
is "east room, floor" "$(figure "$R2" floor_area)" "162.75 Known"
# Its door went with it, 8 ft along: the partition's net is still 123.
is "partition, net of its door" "$(figure "$MID" net_area)" "123.0 Known"
"$P" remove "$J" "$W2"
# North wall with one window: 216 - 12 = 204.
is "north wall, net of one window" "$(figure "$NORTH" net_area)" "204.0 Known"
"$P" check "$J"

echo
echo "== What is refused, and leaves the file as it was =="
cp "$J" "$T/before.json"
refused "say how tall the walls are" "$P" wall "$J" "$S" "$WT" --points "0,20 5,20" "${K[@]}"
refused "it is not on the wall" "$P" door "$J" "$SOUTH" "$DT" --at "30,0" --sill "0 ft" "${K[@]}"
refused "--sill is required" "$P" window "$J" "$EAST" "$NT" --offset "3 ft" "${K[@]}"
refused "--height is required" "$P" run "$J" "$S" duct "$DUCT" --points "0,0 5,0" "${K[@]}"
refused "is not a kind of system" "$P" system "$J" "Steam" "Steam"
refused "--status is required" "$P" wall "$J" "$S" "$WT" --points "0,20 5,20" --height "9 ft"
refused "has elements under it" "$P" remove "$J" "$S"
refused "would be ignored" "$P" move "$J" "$MID" --by "1,0" --height "9 ft" "${K[@]}"
refused "they would be ignored" "$P" remove "$J" "$S" "${K[@]}"
refused "counted twice" "$P" wall "$J" "$S" "$WT" --points "0,0 24,0" --height "9 ft" "${K[@]}"
refused "a duct goes on" "$P" run "$J" "$S" duct "$DUCT" --points "3,3 5,3" --height "8 ft" --system "$COLD" "${K[@]}"
refused "it is not on the wall" "$P" door "$J" "$SOUTH" "$DT" --at "6,5" --sill "0 ft" "${K[@]}"
cmp -s "$J" "$T/before.json" || { echo "FAIL: a refused verb changed the file"; exit 1; }
echo "    the file is byte for byte what it was before the refusals"

echo
echo "PASS: a house drawn from an empty job at the command line, with its figures as worked by hand"
