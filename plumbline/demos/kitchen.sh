#!/usr/bin/env bash
# A kitchen's base, its lump lines and its countertop, set right (call 162). From an empty
# job, one room with a door, the starter assemblies and items, the starter "Kitchen" on the
# room: base does not run behind what stands against the walls, and says what it took off;
# wall boxes hung over a run take none; the countertop carries its line and says what its
# area leaves out over a box that turns a corner; one price on every countertop; the lump
# lines "Cabinets" and "Countertops" are said to be in the price beside the things set in
# the room; a run taken down gives its base back.
# Run from the repository root:  demos/kitchen.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by hand
# in STATUS.md ("A kitchen's base, lump lines and countertop").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
N=(--status Assumed --source "made-up layout")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
est() { "$P" estimate "$J" > "$T/est.txt"; tr -s ' ' < "$T/est.txt" > "$T/est1.txt"; }
roomrow() { "$P" show "$J" | sed -n "/^ *$R  /,/^ *[a-z]*-[0-9]*  /p" > "$T/room.txt"; }

echo "== 1. One room, 11.5 by 15.5 ft inside, a 3 ft door; the starter Kitchen on it; nothing set =="
"$P" new "$J" "A kitchen (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 12,0 12,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" door "$J" "$WEST" "$DT" --offset "6.5 ft" --sill "0 ft" "${K[@]}" >/dev/null
R="$(made "$("$P" room "$J" "$S" "Kitchen" --at "6,8" "${K[@]}")")"
"$P" import assemblies "$J" starter >/dev/null
"$P" import items "$J" starter >/dev/null
"$P" apply "$J" "$R" "Kitchen" --source "the starter library" >/dev/null
"$P" set "$J" "$R" unit_cost '$4/LF' --line "Base and trim" --status Known --source "made-up price" >/dev/null
roomrow
# 54 LF round, less the 3 ft door: 51 LF of wall; nothing stands against it, so the base is as long.
says "$T/room.txt" "base run         51 LF"
says "$T/room.txt" "wall run         51 LF"
est
says "$T/est1.txt" "Base and trim 51.00 LF \$4.00/LF \$204.00"
lacks "$T/est.txt" "less "
lacks "$T/est.txt" "is in the price beside"

echo
echo "== 2. Run A in the southeast corner along the east wall, run B along the south wall: base runs behind neither =="
# A: LS36 B24 SB36 = 96 in, 8 ft from 3 in, with its countertop. B: B18 DB18, 3 ft from 5 ft 9 in, 36 in off the
# corner (what LS36 fills), with its countertop. Covered: the east wall 8 ft; the south wall 3 ft of LS36 and 3 ft
# of run B: 14 ft, behind 5 things. 51 - 14 = 37 LF.
"$P" cabinets "$J" "$S" "$EAST" --in "$R" --from "3 in" --boxes "LS36 B24 SB36" --countertop "${N[@]}" > "$T/runa.txt"
says "$T/runa.txt" "a run of 8 ft, from 0.25 ft to 8.25 ft along the wall"
"$P" cabinets "$J" "$S" "$SOUTH" --in "$R" --from "5' 9\"" --boxes "B18 DB18" --countertop "${N[@]}" > "$T/runb.txt"
says "$T/runb.txt" "a run of 3 ft, from 5.75 ft to 8.75 ft along the wall"
TOPA="$(grep -oE 'furnishing-[0-9]+: countertop' "$T/runa.txt" | sed 's/:.*//')"
TOPB="$(grep -oE 'furnishing-[0-9]+: countertop' "$T/runb.txt" | sed 's/:.*//')"
roomrow
says "$T/room.txt" "~ base run         37 LF"
says "$T/room.txt" "wall run         51 LF"
"$P" show "$J" > "$T/show.txt"
says "$T/show.txt" "Kitchen base run: 37 LF (less 14 ft behind 5 things set against the walls)"
est
# 37 x $4 = $148.00.
says "$T/est1.txt" "Base and trim ~37.00 LF \$4.00/LF ~\$148.00"
says "$T/est.txt" "its quantity: less 14 ft behind 5 things set against the walls"

echo
echo "== 3. Three wall boxes hung over run A (54 in up): the base is as it was =="
"$P" cabinets "$J" "$S" "$EAST" --in "$R" --from "3 in" --boxes "W3030 W2430 W3030" "${N[@]}" > "$T/wall.txt"
says "$T/wall.txt" "hung 4' 6\" up"
roomrow
says "$T/room.txt" "~ base run         37 LF"

echo
echo "== 4. The countertops: A over the corner box says what its area leaves out; B says nothing =="
# A: 8 ft by 25 1/2 in = 2,448 sq in = 17 SF. Over LS36 (36 in on each wall) the leg that turns the corner,
# (36 - 25 1/2) by 25 1/2 in = 267.75 sq in = 1.859375 SF, is not in it. B: 3 ft by 25 1/2 in = 6.375 SF.
says "$T/runa.txt" "17 SF, front edge 8 LF; one line, Countertop, by its area, Division 12, no price"
says "$T/runa.txt" "the leg that turns the corner over LS36, 10 1/2 in by 25 1/2 in (1.86 SF), is not in this area: add it where the top is priced by area."
says "$T/runb.txt" "6.375 SF, front edge 3 LF; one line, Countertop, by its area, Division 12, no price"
lacks "$T/runb.txt" "the leg that turns the corner"

echo
echo "== 5. One price on every countertop: 17 x \$62 = \$1,054.00 and 6.375 x \$62 = \$395.25 =="
"$P" price-item "$J" Countertop '$62/SF' --status Assumed --source "made-up quote" > "$T/out.txt"
says "$T/out.txt" "priced 2 Countertop at \$62/SF (Assumed): $TOPA, $TOPB"
est
says "$T/est1.txt" "Countertop ~17.00 SF ~\$62.00/SF ~\$1,054.00"
says "$T/est1.txt" "Countertop ~6.3750 SF ~\$62.00/SF ~\$395.25"
# The leg is on the line of the top it is on, in the estimate.
says "$T/est.txt" "its quantity: the leg that turns the corner over LS36, 10 1/2 in by 25 1/2 in (1.86 SF)"
"$P" export estimate "$J" "$T/est.csv" >/dev/null
says "$T/est.csv" "the leg that turns the corner over LS36"
# $1,054.00 + $395.25 = $1,449.25 of countertop.
python3 - "$T/est.csv" <<'PY'
import csv, sys
rows = [r for r in csv.reader(open(sys.argv[1])) if any("Countertop" == c for c in r)]
print("    %d countertop lines in the workbook as values" % len(rows))
assert len(rows) == 2, rows
PY

echo
echo "== 6. The lump lines are in the price beside the things set in the room =="
CAB="Kitchen's line \"Cabinets\" (1 EA, 12 Furnishings) is in the price beside the lines of"
TOPS="Kitchen's line \"Countertops\" (1 EA, 12 Furnishings) is in the price beside the lines of"
est
# Five boxes, three wall boxes and two countertops: 10 things, each carrying a line in Division 12.
says "$T/est.txt" "$CAB 10 things set in Kitchen in that division: if it stands for them, take the line off."
says "$T/est.txt" "$TOPS 10 things set in Kitchen in that division: if it stands for them, take the line off."
lacks "$T/est.txt" "Kitchen's line \"Kitchen appliances\""
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "$CAB 10 things"
# `cabinets` says it too, as it sets a run: after the first run of boxes there were 4 things.
says "$T/runa.txt" "$CAB 4 things set in Kitchen"
# A dishwasher set in a third run (Division 11): it is said of "Kitchen appliances" too. Done on a copy.
cp "$J" "$T/third.json"
"$P" cabinets "$T/third.json" "$S" "$NORTH" --in "$R" --from "3 in" --boxes "[Dishwasher]" "${N[@]}" > "$T/third.txt"
"$P" estimate "$T/third.json" > "$T/third-est.txt"
says "$T/third-est.txt" "Kitchen's line \"Kitchen appliances\" (1 EA, 11 Equipment) is in the price beside the line of 1 thing set in Kitchen in that division: if it stands for it, take the line off."
says "$T/third.txt" "Kitchen's line \"Kitchen appliances\" (1 EA, 11 Equipment)"
# With the "Cabinets" line taken off a copy of the job (call 166: one line, by its verb), it is
# said of "Countertops" only. The line had no price: 1 EA went, and nothing else of the room's.
cp "$J" "$T/nocab.json"
refused "$P" remove-line "$T/nocab.json" "$R" "Cabinetry"
says "$T/out.txt" "has no cost line \"Cabinetry\"; it has"
cmp -s "$J" "$T/nocab.json" || { echo "FAIL: a refused remove-line changed the file"; exit 1; }
"$P" remove-line "$T/nocab.json" "$R" "Cabinets" > "$T/took.txt"
says "$T/took.txt" "took \"Cabinets\" off $R: 1 EA, with no price on it"
"$P" estimate "$T/nocab.json" > "$T/nocab-est.txt"
lacks "$T/nocab-est.txt" "Kitchen's line \"Cabinets\""
says "$T/nocab-est.txt" "$TOPS 10 things"

echo
echo "== 7. Run B marked to demolish: LS36 still covers 3 ft of the south wall; the base is 51 - 8 - 3 = 40 LF =="
"$P" mark "$J" "$(grep -oE 'furnishing-[0-9]+: B18' "$T/runb.txt" | sed 's/:.*//')" "$(grep -oE 'furnishing-[0-9]+: DB18' "$T/runb.txt" | sed 's/:.*//')" demolish --clear "${K[@]}" >/dev/null
roomrow
says "$T/room.txt" "~ base run         40 LF"
says "$T/room.txt" "wall run         51 LF"
"$P" show "$J" > "$T/show.txt"
says "$T/show.txt" "Kitchen base run: 40 LF (less 11 ft behind 3 things set against the walls)"
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "its figures agree with what is drawn, and saving it back writes the same file"

echo
echo "  PASS: base ran 51 LF with nothing set and 37 LF behind five things covering 14 ft, still 37 with wall boxes hung, 40 with run B taken down; the countertops came to 17 SF with the leg over the corner box said and 6.375 SF, priced at \$62 together \$1,449.25; the lump lines Cabinets and Countertops were said to be in the price beside the things set in the room, and Kitchen appliances only where a dishwasher is, to the figures worked by hand"
