#!/usr/bin/env bash
# A kitchen from the tool chest (call 155). From an empty job, the house of demos/roomline.sh
# with its 12 ft addition and the rooms Living and Kitchen (a room line parts them at x = 24):
# the Kitchen's east wall runs 36,0 to 36,16 and its face on the Kitchen's side is at
# x = 35.75, clear from y = 0.25 to 15.75. A run of cabinet boxes is set along it by their
# codes: where each stands, the run's length and what is left to the corner, the countertop
# over the base boxes, wall boxes hung over them, one cost line on each box, no clash with
# the wall, and three refusals that leave the job as it was.
# Run from the repository root:  demos/cabinets.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by hand
# in STATUS.md ("A kitchen from the tool chest").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
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

echo "== 1. The house after its addition, two rooms, and the tool chest =="
"$P" new "$J" "A kitchen (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" mark "$J" "$EAST" demolish "${K[@]}" >/dev/null
read -r ADD1 KEAST ADD3 <<<"$("$P" wall "$J" "$S" "$WT" --points "24,0 36,0 36,16 24,16" --height "9 ft" "${N[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
LIVING="$(made "$("$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}")")"
KITCHEN="$(made "$("$P" room "$J" "$S" "Kitchen" --at "30,8" "${K[@]}")")"
"$P" room-line "$J" "$S" --points "24,0 24,16" "${N[@]}" >/dev/null
IN="$("$P" import items "$J" starter)"
grep -qF "set: 48" <<<"$IN" || { echo "FAIL: the starter tool chest is not 48 items"; echo "$IN"; exit 1; }
echo "    the tool chest: 48 items, the 27 boxes among them"

echo
echo "== 2. A run along the Kitchen's east wall, from 1 ft: B18 SB36 B24 DB18 B36, with its countertop =="
# 18 + 36 + 24 + 18 + 36 = 132 in: 11 ft, from y = 1 to y = 12; the face is clear to y = 15.75:
# 3.75 ft left. Each box is 2 ft deep, so its middle is 1 ft off the face (x = 35.75): 34.75.
"$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "B18 SB36 B24 DB18 B36" --countertop "${N[@]}" | tee "$T/run.txt" | sed 's/^/    | /'
says "$T/run.txt" "set 5 boxes along $KEAST in $KITCHEN: a run of 11 ft, from 1 ft to 12 ft along the wall; 3.75 ft left to the corner"
says "$T/run.txt" "B18, 1' 6\" wide, middle at 34.75, 1.75"
says "$T/run.txt" "SB36, 3' 0\" wide, middle at 34.75, 4"
says "$T/run.txt" "B24, 2' 0\" wide, middle at 34.75, 6.5"
says "$T/run.txt" "DB18, 1' 6\" wide, middle at 34.75, 8.25"
says "$T/run.txt" "B36, 3' 0\" wide, middle at 34.75, 10.5"
# The countertop: 11 ft by 25 1/2 in (2.125 ft): 23.375 SF; its middle 1.0625 ft off the face.
says "$T/run.txt" "countertop, 11 ft long by 25 1/2 in deep, 34 1/2 in up, middle at 34.6875, 6.5; 23.375 SF, front edge 11 LF"
TOP="$(grep -oE '[a-z]+-[0-9]+: countertop' "$T/run.txt" | head -1 | sed 's/:.*//')"
"$P" line "$J" "$TOP" "Countertop, quartz" plan_area --cost-type material --division 6 >/dev/null
"$P" set "$J" "$TOP" unit_cost '$62.00/SF' --status Assumed --source "made-up price" --line "Countertop, quartz" >/dev/null
"$P" estimate "$J" > "$T/est.txt"
grep -F "Countertop, quartz" "$T/est.txt" | head -3 | sed 's/^/    | /'
says "$T/est.txt" "\$1,449.25"
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "its figures agree with what is drawn, and saving it back writes the same file"

echo
echo "== 3. Wall boxes on the same wall from 1 ft: W3030 W3630 W1830 =="
# 30 + 36 + 18 = 84 in: 7 ft, 1 ft deep, middles 0.5 ft off the face (35.25), 54 in up.
"$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "W3030 W3630 W1830" "${N[@]}" | tee "$T/wall.txt" | sed 's/^/    | /'
says "$T/wall.txt" "set 3 boxes along $KEAST in $KITCHEN: a run of 7 ft, from 1 ft to 8 ft along the wall; 7.75 ft left to the corner"
says "$T/wall.txt" "W3030, 2' 6\" wide, middle at 35.25, 2.25, hung 4' 6\" up"

echo
echo "== 4. Eight boxes by their codes, no price, in the supplier's list; no clash with the wall they back onto =="
"$P" estimate "$J" by-item > "$T/item.txt"
for code in B18 SB36 B24 DB18 B36 W3030 W3630 W1830; do
  grep -E "^  $code +Material +1 +1 EA +no price" "$T/item.txt" > /dev/null || { echo "FAIL: the estimate by item does not list $code once, 1 EA, with no price"; cat "$T/item.txt"; exit 1; }
  printf '    %s: 1 EA, no price\n' "$code"
done
"$P" export supplier "$J" "$T/supplier.csv" >/dev/null
for code in B18 SB36 B24 DB18 B36 W3030 W3630 W1830; do says "$T/supplier.csv" "$code," ; done
"$P" clashes "$J" > "$T/clash.txt"
for thing in $(grep -oE '[a-z]+-[0-9]+' "$T/run.txt" | sort -u); do
  if grep -E "$thing.*$KEAST|$KEAST.*$thing" "$T/clash.txt" | grep -qi "clash"; then echo "FAIL: $thing clashes with the wall it backs onto"; cat "$T/clash.txt"; exit 1; fi
done
lacks "$T/clash.txt" "B18"
lacks "$T/clash.txt" "Countertop"
"$P" clashes "$J" | head -3 | sed 's/^/    | /'
# The test is live: a box pushed 0.3 ft into the wall it backs onto does clash with it.
cp "$J" "$T/pushed.json"
FIRST="$(grep -oE 'furnishing-[0-9]+' "$T/run.txt" | head -1)"
"$P" move "$T/pushed.json" "$FIRST" --by "0.3,0" "${N[@]}" >/dev/null
"$P" clashes "$T/pushed.json" > "$T/pushed.txt"
grep -q "In the same space: 1" "$T/pushed.txt" || { echo "FAIL: a box in the wall is not found"; cat "$T/pushed.txt"; exit 1; }
echo "    (a box pushed 0.3 ft into its wall is found: the check is live)"

echo
echo "== 5. Refused, in words, the job unchanged =="
"$P" type "$J" door "door 3 x 7" --width "3 ft" --height "7 ft" "${K[@]}" > "$T/dt.txt"
DT="$(made "$(cat "$T/dt.txt")")"
cp "$J" "$T/before.json"
# call 225: B99 is now a box made by the rule for B (call 215); a code no rule reads is Q99
refused "$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "Q99" "${N[@]}"
says "$T/out.txt" "Q99 is not in the tool chest, and no cabinet rule reads Q"
says "$T/out.txt" ". It has: "
says "$T/out.txt" "B18"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
refused "$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "B36 B36 B36 B36 B36 B36" "${N[@]}"
says "$T/out.txt" "past the corner of"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
cp "$J" "$T/no-door.json"
"$P" door "$J" "$KEAST" "$DT" --offset "5 ft" --sill "0 ft" "${K[@]}" >/dev/null
cp "$J" "$T/before.json"
refused "$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "B36 B36" "${N[@]}"
says "$T/out.txt" "a door is no place for boxes"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }

echo
echo "== 6. A box that would stand in another thing is refused, naming it and by how much (call 157) =="
# From here on, the job as it was before step 5 put a door in the east wall.
J="$T/no-door.json"
# The east run (B18 SB36 B24 DB18 B36, from 1 ft, 2 ft deep) fills x = 33.75 to 35.75, y = 1 to 12.
# A second run over the first, the same boxes from the same place: B18 stands in the B18 there.
cp "$J" "$T/before.json"
refused "$P" cabinets "$J" "$S" "$KEAST" --in "$KITCHEN" --from "1 ft" --boxes "B18 SB36 B24 DB18 B36" "${N[@]}"
says "$T/out.txt" "B18 would stand in B18 (furnishing-"
grep -qE "would stand in B18 \(furnishing-[0-9]+\) by [0-9.]+ in: set the run clear of it" "$T/out.txt" || { echo "FAIL: it does not say by how much"; cat "$T/out.txt"; exit 1; }
echo "    ... by so many inches: set the run clear of it"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
# Two runs meeting in an inside corner: the south wall's run to the corner, B36 from 8.75 ft
# (x = 32.75 to 35.75, y = 0.25 to 2.25), stands in the B18 of the east run at the corner.
refused "$P" cabinets "$J" "$S" "$ADD1" --in "$KITCHEN" --from "8.75 ft" --boxes "B36" "${N[@]}"
says "$T/out.txt" "B36 would stand in B18 (furnishing-"
says "$T/out.txt" "starts off the corner by what the other run fills (24 in for base boxes)"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
# Off the corner by what the east run fills, 24 in: the run ends at 9.75 ft (x = 33.75) and is taken.
"$P" cabinets "$J" "$S" "$ADD1" --in "$KITCHEN" --from "6.75 ft" --boxes "B36" "${N[@]}" > "$T/out.txt"
says "$T/out.txt" "set 1 box along $ADD1 in $KITCHEN: a run of 3 ft, from 6.75 ft to 9.75 ft along the wall; 2 ft left to the corner"

echo
echo "== 7. The corner boxes are in the tool chest; the supplier's list is no surer than the boxes =="
"$P" items "$J" > "$T/items.txt"
says "$T/items.txt" "BLB36"
says "$T/items.txt" "LS36"
"$P" export supplier "$J" "$T/supplier.csv" >/dev/null
# Every box was set Assumed, and its size is Assumed: the list does not say Known.
says "$T/supplier.csv" "B18,1,EA,1,Assumed,"
lacks "$T/supplier.csv" "B18,1,EA,1,Known"

echo
echo "== 8. A price by its code: one command prices every box of it, and says how many =="
"$P" price-item "$J" "B36" '$300' --status Assumed --source "made-up supplier quote" > "$T/out.txt"
# B36 is set twice: the east run's and the south run's.
says "$T/out.txt" "priced 2 B36 at \$300 (Assumed)"
"$P" price-item "$J" "B21" '$300' --status Assumed --source "made-up supplier quote" > "$T/out.txt"
says "$T/out.txt" "none of B21 is set in the job yet, so nothing was priced"
"$P" estimate "$J" by-item > "$T/item.txt"
grep -E "^  B36 +Material +2 +2 EA +.[$]300.00/EA" "$T/item.txt" > /dev/null || { echo "FAIL: both B36 are not priced at \$300"; cat "$T/item.txt"; exit 1; }
echo "    B36: 2 EA at \$300.00/EA"

cp "$J" "$T/boxed-before-windows.json"

echo
echo "== 9. Windows: a box across one is refused; a base box under a sill at or above its top is taken =="
# The north wall (36,16 to 24,16, 12 ft; its face is clear from 0.25 to 11.75 ft). Window 1: 3 ft by 4 ft,
# centred 3.5 ft along, so from 2 to 5 ft, its sill 3 ft up (36 in: at or above a base box's 34 1/2 in) and
# its head 7 ft. Window 2: centred 9.5 ft, from 8 to 11 ft, its sill 2 ft up (24 in: below a base box's top).
"$P" type "$J" window "window 3 x 4" --width "3 ft" --height "4 ft" "${K[@]}" > "$T/wt.txt"
WT4="$(made "$(cat "$T/wt.txt")")"
"$P" window "$J" "$ADD3" "$WT4" --offset "3.5 ft" --sill "3 ft" "${K[@]}" >/dev/null
"$P" window "$J" "$ADD3" "$WT4" --offset "9.5 ft" --sill "2 ft" "${K[@]}" >/dev/null
cp "$J" "$T/before.json"
# A wall box hangs from 54 in to 84 in: across window 1.
refused "$P" cabinets "$J" "$S" "$ADD3" --in "$KITCHEN" --from "2.5 ft" --boxes "W3030" "${N[@]}"
says "$T/out.txt" "W3030 hangs from 2' 6\" to 5' 0\" along"
says "$T/out.txt" "across window 1 ("
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
# A tall box, 84 in, stands across window 1 too.
refused "$P" cabinets "$J" "$S" "$ADD3" --in "$KITCHEN" --from "2.5 ft" --boxes "T2484" "${N[@]}"
says "$T/out.txt" "across window 1 ("
# A base box (top 34 1/2 in) under window 2, whose sill is 24 in: refused.
refused "$P" cabinets "$J" "$S" "$ADD3" --in "$KITCHEN" --from "8.5 ft" --boxes "B30" "${N[@]}"
says "$T/out.txt" "across window 2 ("
says "$T/out.txt" "a base box fits under a window only where the sill is at or above its top"
cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }
# A base box under window 1, whose sill, 36 in, is above its 34 1/2 in top: taken.
"$P" cabinets "$J" "$S" "$ADD3" --in "$KITCHEN" --from "2.25 ft" --boxes "B30" "${N[@]}" > "$T/out.txt"
says "$T/out.txt" "set 1 box along $ADD3 in $KITCHEN"

echo
echo "== 10. Appliances in a run, by their names; the countertop is not laid over a range =="
# A fresh run on the north wall of the job before its windows: B18, the dishwasher, SB36 and the 30 in range,
# from 1 ft: 18 + 24 + 36 + 30 = 108 in, 9 ft, to 10 ft along the 12 ft wall: 2 ft left. The dishwasher and the
# range are Division 11 items, and carry their line there.
# The countertop goes over what is no taller than 34 1/2 in: B18 (34 1/2), the dishwasher (34) and SB36
# (34 1/2): 18 + 24 + 36 = 78 in, 6.5 ft, and not over the range (36 in tall) at the end of the run.
# 6.5 ft x 25 1/2 in (2.125 ft) = 13.8125 SF; its front edge is 6.5 LF.
cp "$T/boxed-before-windows.json" "$T/appl.json"
"$P" cabinets "$T/appl.json" "$S" "$ADD3" --in "$KITCHEN" --from "1 ft" --boxes "B18 [Dishwasher] SB36 [Range, 30 in]" --countertop "${N[@]}" > "$T/out.txt"
cat "$T/out.txt" | sed 's/^/    | /'
says "$T/out.txt" "set 4 boxes along $ADD3 in $KITCHEN: a run of 9 ft, from 1 ft to 10 ft along the wall; 2 ft left to the corner"
says "$T/out.txt" "Dishwasher, 2' 0\" wide"
says "$T/out.txt" "Range, 30 in, 2' 6\" wide"
says "$T/out.txt" "countertop, 6.5 ft long by 25 1/2 in deep, 34 1/2 in up"
says "$T/out.txt" "13.8125 SF, front edge 6.5 LF"
"$P" export supplier "$T/appl.json" "$T/supplier2.csv" >/dev/null
says "$T/supplier2.csv" "Dishwasher,1,EA,1,Assumed,"
cp "$T/appl.json" "$T/before.json"
refused "$P" cabinets "$T/appl.json" "$S" "$ADD3" --in "$KITCHEN" --from "1 ft" --boxes "[Dishwasher" "${N[@]}"
says "$T/out.txt" "not closed"
cmp -s "$T/appl.json" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }

echo
echo "  PASS: a run of five boxes stood along the east wall to the hand figures (11 ft, 3.75 ft left), its countertop came to 23.375 SF and \$1,449.25, wall boxes hung at 54 in, eight boxes are on the estimate by item and the supplier's list with no price, none clashes with the wall, and three runs were refused with the job as it was; and (call 157) a run over a run, a run into an inside corner, a wall box or tall box across a window and a base box under a low sill were refused, naming what and by how much, with the job as it was; a base box under a sill at or above its top was taken; a run of B18 [Dishwasher] SB36 [Range, 30 in] took a countertop of 13.8125 SF over the first three and none over the range; BLB36 and LS36 are in the tool chest; the supplier's list reads Assumed for boxes set Assumed; and one command priced both B36"
