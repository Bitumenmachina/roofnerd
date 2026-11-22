#!/usr/bin/env bash
# An open plan: a room line (call 152). From an empty job, the house of demos/reno.sh after
# its addition (walls 6 in thick and 9 ft tall round 0,0 to 24,16; the east wall to
# demolish; three new walls 24,0 36,0 36,16 24,16; no partition); one room is the whole
# inside; a room line parts it in two where no wall stands; each room to the figures worked
# by hand; the line is no wall, takes no price and no mark; removed, the rooms are as they
# were; saved and read back, the same.
# Run from the repository root:  demos/roomline.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by
# hand in STATUS.md ("An open plan: a room line").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
N=(--status Assumed --source "made-up addition plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }
shown() { "$P" show "$J" > "$T/show.txt"; block "$1" > "$T/$2.txt"; }

echo "== 1. The house after its addition: one room is the whole inside; a second is refused =="
"$P" new "$J" "Open plan (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" mark "$J" "$EAST" demolish "${K[@]}" >/dev/null
"$P" wall "$J" "$S" "$WT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${N[@]}" >/dev/null
LIVING="$(made "$("$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}")")"
KITCHEN="$(made "$("$P" room "$J" "$S" "Kitchen" --at "30,8" "${K[@]}")")"
shown "$LIVING" living; shown "$KITCHEN" kitchen
# 35.5 by 15.5 inside the faces.
says "$T/living.txt" "floor area       550.25 SF"
says "$T/kitchen.txt" "floor area       unknown"
"$P" show "$J" | grep -F "Kitchen floor area" > "$T/why.txt"
says "$T/why.txt" "Living was placed first and has the space this room's point is in"

echo
echo "== 2. A room line from 24,0 to 24,16: two rooms, to the figures worked by hand =="
cp "$J" "$T/before.json"
refused "$P" room-line "$J" "$WT" --points "24,0 24,16" "${N[@]}"
says "$T/out.txt" "not a level"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
RL="$(made "$("$P" room-line "$J" "$S" --points "24,0 24,16" "${N[@]}")")"
shown "$LIVING" living; shown "$KITCHEN" kitchen
# Living: 23.75 by 15.5; round 2 x (23.75 + 15.5); its walls 23.75 + 15.5 + 23.75 = 63 LF at 9 ft;
# its floor line is one run of 63 ft: 63 / 12 = 5.25, 6 outlets.
says "$T/living.txt" "floor area       368.125 SF"
says "$T/living.txt" "perimeter        78.5 LF"
says "$T/living.txt" "wall area        567 SF"
says "$T/living.txt" "receptacles      6 EA"
# Its run of wall, for base: the same 63 LF (call 157; the line takes none, no door cuts it).
says "$T/living.txt" "wall run         63 LF"
# Kitchen: 11.75 by 15.5; walls 11.75 + 15.5 + 11.75 = 39 LF: 351 SF; 39 / 12 = 3.25, 4 outlets.
says "$T/kitchen.txt" "floor area       182.125 SF"
says "$T/kitchen.txt" "perimeter        54.5 LF"
says "$T/kitchen.txt" "wall area        351 SF"
says "$T/kitchen.txt" "receptacles      4 EA"
says "$T/kitchen.txt" "wall run         39 LF"
# The two floors add to 550.25; a room measured to the line is no surer than it (Assumed).
"$P" show "$J" > "$T/show.txt"
says "$T/show.txt" "$RL  room line 1  [room line]"
grep -F "Living floor area" "$T/show.txt" > "$T/flag.txt"
says "$T/flag.txt" "~ $LIVING Living floor area: 368.125 SF"
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "its figures agree with what is drawn, and saving it back writes the same file"

echo
echo "== 3. It is no wall, and no thing that is bought or built =="
"$P" takeoff "$J" > "$T/take.txt"
says "$T/take.txt" "walls 7            120.00 LF"
"$P" import assemblies "$J" starter >/dev/null
cp "$J" "$T/before.json"
refused "$P" line "$J" "$RL" "Partition" 1 --cost-type material --division 9
says "$T/out.txt" "a room line cannot carry cost lines"
refused "$P" apply "$J" "$RL" "Exterior wall, 2x6, lap siding" --source "made-up"
says "$T/out.txt" "is a room line"
refused "$P" set "$J" "$RL" allowance '$2.00/SF' --status Known --source "made-up"
says "$T/out.txt" "a room line cannot carry an allowance"
refused "$P" mark "$J" "$RL" existing "${K[@]}"
says "$T/out.txt" "is a room line, which is not a thing that is built"
"$P" estimate "$J" > "$T/est.txt"
lacks "$T/est.txt" "room line"
"$P" clashes "$J" > "$T/clash.txt"
says "$T/clash.txt" "Tested: 6 elements"
lacks "$T/clash.txt" "$RL"
"$P" export ifc "$J" "$T/o.ifc" > "$T/ifc.txt"
says "$T/ifc.txt" "1 x room line:"
says "$T/ifc.txt" "7 walls"
# Base is priced by the run of wall, and not by the perimeter, which takes in the line:
# Living's base is 63 LF and not 78.5; the Kitchen's 39 LF and not 54.5 (call 157).
"$P" apply "$J" "$LIVING" "Room, hardwood floor" --source "made-up" >/dev/null
"$P" apply "$J" "$KITCHEN" "Kitchen" --source "made-up" >/dev/null
"$P" estimate "$J" > "$T/est.txt"
grep -E "^  Base and trim +~63.00 LF" "$T/est.txt" > /dev/null || { echo "FAIL: Living's base is not 63 LF"; cat "$T/est.txt"; exit 1; }
grep -E "^  Base and trim +~39.00 LF" "$T/est.txt" > /dev/null || { echo "FAIL: the Kitchen's base is not 39 LF"; cat "$T/est.txt"; exit 1; }
echo "    Base and trim: 63.00 LF in Living, 39.00 LF in the Kitchen (the perimeters, 78.5 and 54.5, take the line in)"

echo
echo "== 4. Saved and read back the same; removed, the rooms are as in step 1 =="
cp "$J" "$T/saved.json"
"$P" check "$T/saved.json" > "$T/check.txt"
says "$T/check.txt" "saving it back writes the same file"
shown "$LIVING" living
says "$T/living.txt" "floor area       368.125 SF"
"$P" remove "$J" "$RL" >/dev/null
shown "$LIVING" living; shown "$KITCHEN" kitchen
says "$T/living.txt" "floor area       550.25 SF"
says "$T/kitchen.txt" "floor area       unknown"
echo
echo "== 5. A room line that reaches nothing is taken, and said (call 157) =="
# A stub from 12,-5 to 12,5: one end outside the house, the other inside the room, neither at a
# wall or another room line. A line from 12,0 to 12,16 meets the south and north walls and says nothing.
"$P" room-line "$J" "$S" --points "12,-5 12,5" "${N[@]}" > "$T/out.txt"
says "$T/out.txt" "1 room line: "
says "$T/out.txt" "its end at 12,-5 meets no wall and no room line: it parts nothing until it does"
says "$T/out.txt" "its end at 12,5 meets no wall and no room line: it parts nothing until it does"
"$P" room-line "$J" "$S" --points "12,0 12,16" "${N[@]}" > "$T/out.txt"
says "$T/out.txt" "1 room line: "
lacks "$T/out.txt" "meets no wall"
echo
echo "  PASS: a room line parted one room into two where no wall stands, to the figures worked by hand (368.125 and 182.125 SF, adding to 550.25), was no wall and took no price or mark, was saved and read back, and removing it gave the one room back"
