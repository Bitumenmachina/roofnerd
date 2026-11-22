#!/usr/bin/env bash
# What the sweep found wrong in dollars, and what call 175 decided (STATUS.md, "Call 175's
# fixes: a demonstration, written before it is built"): from an empty job, at the command
# line, each step against the figure worked by hand. A step fails the script if the program
# does not say the figure, and each prints what was said beside what was worked out.
# Run from the repository root:  demos/sweep-money.sh
# The job is made up (no real job data) and lives in a temporary folder.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    said: %s\n' "$2"; }
saysnt() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says: $2"; cat "$1"; exit 1; fi; printf '    does not say: %s\n' "$2"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
# What the program says of one thing: its block of `show`.
block() { awk -v p="$2" '$1 == p && NF > 1 {on=1; print; next} on && /^    [a-z]+-[0-9]+  / {exit} on {print}' "$1"; }

echo "== 1. Siding is not bought over the stone =="
echo "   one outside wall, 40 ft long and 10 ft high, a door 3 ft by 6 ft 8 in with its sill at the floor, a window 5 ft by 5 ft with its sill at 3 ft"
"$P" new "$J" "Sweep (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "10 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "6 ft 8 in" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 5 ft" --width "5 ft" --height "5 ft" "${K[@]}")")"
W="$("$P" wall "$J" "$S" "$WT" --points "0,0 40,0" --height "10 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" door "$J" "$W" "$DT" --offset "6 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" window "$J" "$W" "$NT" --offset "24 ft" --sill "3 ft" "${K[@]}" >/dev/null
# A wall with no assembly on it says nothing new.
"$P" show "$J" > "$T/show.txt"; block "$T/show.txt" "$W" > "$T/wall.txt"
saysnt "$T/wall.txt" "net length"
"$P" import assemblies "$J" starter >/dev/null
"$P" apply "$J" "$W" "Exterior wall, 2x6, lap siding over a stone wainscot" --source "made-up finish schedule" >/dev/null
"$P" show "$J" > "$T/show.txt"; block "$T/show.txt" "$W" > "$T/wall.txt"
# face 40 x 10 = 400; door 3 x 6.6667 = 20; window 25; net 400 - 45 = 355.
says "$T/wall.txt" "face area        400 SF"
says "$T/wall.txt" "opening area     45 SF"
says "$T/wall.txt" "net area         355 SF"
# net_length = 40 - 3 = 37 (the door comes to the floor; the window does not).
says "$T/wall.txt" "net length       37 LF"
says "$T/wall.txt" "quantity: Stone veneer wainscot, 3 ft 37 LF"
# siding and hanging it: 355 - 3 x 37 = 244.
says "$T/wall.txt" "quantity: Lap siding 244 SF"
says "$T/wall.txt" "quantity: Hang the siding 244 SF"
# what runs behind the stone stays on the whole wall.
says "$T/wall.txt" "quantity: Weather barrier 355 SF"
says "$T/wall.txt" "quantity: Batt insulation 355 SF"
says "$T/wall.txt" "quantity: Wall sheathing 400 SF"
# The 3 ft is the name of the stone line, and it is Assumed: the siding rests on it.
says "$T/wall.txt" "~ quantity: Lap siding 244 SF"

echo "   the same wall with no door"
W2="$("$P" wall "$J" "$S" "$WT" --points "0,20 40,20" --height "10 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" window "$J" "$W2" "$NT" --offset "24 ft" --sill "3 ft" "${K[@]}" >/dev/null
"$P" apply "$J" "$W2" "Exterior wall, 2x6, lap siding over a stone wainscot" --source "made-up finish schedule" >/dev/null
"$P" show "$J" > "$T/show.txt"; block "$T/show.txt" "$W2" > "$T/wall2.txt"
# The window alone: net 400 - 25 = 375; net_length 40 (a window above the floor takes nothing); stone 40;
# siding 375 - 3 x 40 = 255. (STATUS.md works this as 380 - 120 = 260; 380 is 400 less a door's 20, and the door is gone.)
says "$T/wall2.txt" "net area         375 SF"
says "$T/wall2.txt" "net length       40 LF"
says "$T/wall2.txt" "quantity: Stone veneer wainscot, 3 ft 40 LF"
says "$T/wall2.txt" "quantity: Lap siding 255 SF"

echo
echo "== 2. Base is not bought across a doorway: the sample whole house =="
"$P" sixteen "$T/sixteen.json" >/dev/null
"$P" show "$T/sixteen.json" > "$T/s.txt"
# Each room's base and trim is its own base_run; the sum of the lines is the sum of the base runs.
awk '
  /^    room-[0-9]+  .*\[room\]/ { room=$2; for (i=3;i<NF;i++) room=room" "$i; next }
  /^    [a-z]+-[0-9]+  / { room="" }
  room != "" && $0 ~ /=   base run / { for (i=1;i<=NF;i++) if ($i=="run") br=$(i+1) }
  room != "" && $0 ~ /=   perimeter / { for (i=1;i<=NF;i++) if ($i=="perimeter") pm=$(i+1) }
  room != "" && $0 ~ /quantity: Base and trim / { q=$(NF-1); n++; sbr+=br; sq+=q; sp+=pm
      if (q != br) { print "FAIL: " room ": Base and trim " q " is not its base run " br; bad=1 }
      printf "    %-16s base run %9s LF   priced %9s LF   (was its perimeter, %s)\n", room, br, q, pm }
  END { printf "    %d rooms: base runs %.3f LF, priced %.3f LF, perimeters were %.3f LF\n", n, sbr, sq, sp
        printf "    the price falls by %.2f at $7.50/LF, before overhead and profit\n", (sp - sq) * 7.5
        if (bad || n == 0 || sbr != sq) exit 1 }' "$T/s.txt"
block "$T/s.txt" "$(grep -E '^    room-[0-9]+  kitchen  \[room\]' "$T/s.txt" | head -1 | awk '{print $1}')" > "$T/kitchen.txt"
# the kitchen: 45.08 LF (was 50.25), the living room 99.285 (the STATUS.md hand figure 99.28 is this, cut at the hundredth; was 110.125).
says "$T/kitchen.txt" "quantity: Base and trim 45.08 LF"
says "$T/kitchen.txt" "perimeter        50.25 LF"
block "$T/s.txt" "$(grep -E '^    room-[0-9]+  living room  \[room\]' "$T/s.txt" | head -1 | awk '{print $1}')" > "$T/living.txt"
says "$T/living.txt" "quantity: Base and trim 99.285 LF"
says "$T/living.txt" "perimeter        110.125 LF"

echo "   the sample's walls: stone by the length that is not a doorway, siding not bought behind it"
N="$(grep -E '^    wall-[0-9]+  north wall  \[wall\]' "$T/s.txt" | head -1 | awk '{print $1}')"
SO="$(grep -E '^    wall-[0-9]+  south wall  \[wall\]' "$T/s.txt" | head -1 | awk '{print $1}')"
block "$T/s.txt" "$N" > "$T/north.txt"; block "$T/s.txt" "$SO" > "$T/south.txt"
# North: no door, 370 SF net, 40 LF: stone 40.00 LF; siding 370 - 120 = 250.
says "$T/north.txt" "quantity: Stone veneer wainscot, 3 ft 40 LF"
says "$T/north.txt" "quantity: Cedar lap siding, installed 250 SF"
# South: the front door, 3 ft: net_length 37, net 329.99; the stone 37 LF (was 40) and siding 329.99 - 111 = 218.99 (was 329.99).
says "$T/south.txt" "net area         329.99 SF"
says "$T/south.txt" "quantity: Stone veneer wainscot, 3 ft 37 LF"
says "$T/south.txt" "quantity: Cedar lap siding, installed 218.99 SF"
# The totals. Before: the job cost $394,677.48. By hand the fall is the base (427.20), 3 LF less stone at
# $95 (285.00) and 399 SF less siding at $9.50 (3,790.50): 4,502.70, to the cent of each line's rounding.
"$P" estimate "$T/sixteen.json" > "$T/est.txt"
JC="$(grep -E '^  Job cost ' "$T/est.txt" | head -1 | grep -oE '[0-9,]+\.[0-9]+' | head -1 | tr -d ,)"
awk -v jc="$JC" 'BEGIN { was = 394677.48; fall = was - jc; printf "    job cost was $394,677.48, now $%.2f: it falls by $%.2f (by hand $4,502.70)\n", jc, fall; if (fall < 4502.65 || fall > 4502.75) exit 1 }' || { echo "FAIL: the job cost did not fall by what was worked by hand"; exit 1; }

echo
echo
echo "== 3. A fixture set against its own wall is not a clash =="
echo "   a room 12 ft by 10 ft with 6 in walls, a tub that stands 0.45 in into the wall behind it, and a duct and a pipe drawn through one another in the middle of the room"
J3="$T/three.json"
"$P" new "$J3" "Set against (not a real job)" >/dev/null
L3="$(made "$("$P" level "$J3" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
W3T="$(made "$("$P" type "$J3" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DUCT="$(made "$("$P" type "$J3" duct "duct 12 x 8" --width "12 in" --depth "8 in" "${K[@]}")")"
PIPE="$(made "$("$P" type "$J3" pipe "pipe 4 in" --round "4 in" "${K[@]}")")"
SOUTH3="$("$P" wall "$J3" "$L3" "$W3T" --points "0,0 12,0 12,10 0,10" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
# The south wall's inside face is at y = 0.25. A tub 2 1/2 ft deep with its south edge at 0.25 - 0.45/12 = 0.2125 is centred at y = 1.4625.
TUB="$(made "$("$P" place "$J3" "$L3" fixture "tub" --at "6,1.4625" --offset "0 ft" --size "5,2.5,1.5" "${K[@]}")")"
"$P" run "$J3" "$L3" duct "$DUCT" --points "3,5 9,5" --height "6 ft" "${K[@]}" >/dev/null
"$P" run "$J3" "$L3" pipe "$PIPE" --points "6,3 6,7" --height "6 ft" "${K[@]}" >/dev/null
"$P" clashes "$J3" > "$T/clash3.txt"
# The duct and the pipe are in the same space: 1. The tub and its wall are set against: 1, listed under its own heading.
says "$T/clash3.txt" "In the same space: 1 "
says "$T/clash3.txt" "Set against (stands against it, or is joined to it, by no more than 2.00 in: not a clash): 1 "
says "$T/clash3.txt" "set 0.45 in into it"
sed -n '/^In the same space/,/^Set against/p' "$T/clash3.txt" > "$T/same3.txt"
saysnt "$T/same3.txt" "tub"
echo "   the tub moved 6 in into the wall: past 2 in it is in the wall, not against it"
"$P" move "$J3" "$TUB" --by "0,-0.4625" "${K[@]}" >/dev/null
"$P" clashes "$J3" > "$T/clash3.txt"
says "$T/clash3.txt" "In the same space: 2 "
says "$T/clash3.txt" "Set against (stands against it, or is joined to it, by no more than 2.00 in: not a clash): 0"
says "$T/clash3.txt" "tub"

echo
echo "== 4. A hole is counted once =="
echo "   a pipe riser of three pieces, end to end, up through a floor 10 in thick (its top at the level 9 ft up)"
J4="$T/four.json"
"$P" new "$J4" "A hole counted once (not a real job)" >/dev/null
G4="$(made "$("$P" level "$J4" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
S4="$(made "$("$P" level "$J4" second --elevation "9 ft" --height "9 ft" "${K[@]}")")"
T4="$(made "$("$P" level "$J4" third --elevation "18 ft" --height "9 ft" "${K[@]}")")"
FT4="$(made "$("$P" type "$J4" floor "floor 10 in" --thickness "10 in" "${K[@]}")")"
PT4="$(made "$("$P" type "$J4" pipe "pipe 4 in" --round "4 in" "${K[@]}")")"
"$P" floor "$J4" "$S4" "$FT4" --points "0,0 12,0 12,10 0,10" "${K[@]}" >/dev/null
for h in "8 ft:8.4 ft" "8.4 ft:8.8 ft" "8.8 ft:9.5 ft"; do
  "$P" run "$J4" "$G4" pipe "$PT4" --points "6,5" --height "${h%%:*}" --rises-to "${h##*:}" "${K[@]}" >/dev/null
done
"$P" clashes "$J4" > "$T/clash4.txt"
# Each of the three pieces is in the floor's slab (8 ft 2 in to 9 ft): three pieces, one hole to cut.
says "$T/clash4.txt" "Through a wall or a floor (a hole to cut): 1 "
says "$T/clash4.txt" "1 hole, 4.00 in round, through 3 pieces of the run"
says "$T/clash4.txt" "floor 1 [floor-"
echo "   the same riser carried up through a second floor"
"$P" floor "$J4" "$T4" "$FT4" --points "0,0 12,0 12,10 0,10" "${K[@]}" >/dev/null
for h in "9.5 ft:17.3 ft" "17.3 ft:17.9 ft" "17.9 ft:18.6 ft"; do
  "$P" run "$J4" "$G4" pipe "$PT4" --points "6,5" --height "${h%%:*}" --rises-to "${h##*:}" "${K[@]}" >/dev/null
done
"$P" clashes "$J4" > "$T/clash4.txt"
says "$T/clash4.txt" "Through a wall or a floor (a hole to cut): 2 "
[ "$(grep -c " 1 hole, 4.00 in round" "$T/clash4.txt")" = 2 ] || { echo "FAIL: not one line for each floor"; cat "$T/clash4.txt"; exit 1; }
"$P" vif "$J4" > "$T/vif4.txt"
echo
echo "== 7. The BIM file says existing, demolished and new in its own word =="
echo "   one wall existing, one to demolish, one new (said bearing), a door in the existing wall to demolish, and a roof, written out and read back"
J7="$T/seven.json"
"$P" new "$J7" "A BIM file's own word (not a real job)" >/dev/null
L7="$(made "$("$P" level "$J7" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
W7T="$(made "$("$P" type "$J7" wall "exterior wall 2x6" --thickness "5 1/2 in" "${K[@]}")")"
D7T="$(made "$("$P" type "$J7" door "entry door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
read -r OLD GONE NEW7 <<<"$(for y in 0 10 20; do "$P" wall "$J7" "$L7" "$W7T" --points "0,$y 20,$y" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1; done | tr '\n' ' ')"
DOOR7="$(made "$("$P" door "$J7" "$OLD" "$D7T" --offset "5 ft" --sill "0 ft" "${K[@]}")")"
ROOF7="$(made "$("$P" roof "$J7" "main roof" --points "0,0 30,0 30,30 0,30" --pitch 6:12 --eave-height "9 ft" "${K[@]}")")"
"$P" mark "$J7" "$OLD" existing "${K[@]}" >/dev/null
"$P" mark "$J7" "$GONE" demolish "${K[@]}" >/dev/null
"$P" mark "$J7" "$NEW7" new "${K[@]}" >/dev/null
"$P" mark "$J7" "$DOOR7" demolish "${K[@]}" >/dev/null
"$P" mark "$J7" "$ROOF7" new "${K[@]}" >/dev/null
"$P" set "$J7" "$NEW7" bearing bearing --status Assumed --source "made-up: the engineer has not said" >/dev/null
"$P" export ifc "$J7" "$T/seven.ifc" > "$T/export7.txt"
# The literals are the standard's: PEnum_ElementStatus in the property set templates that come with IfcOpenShell
# (Pset_IFC4_ADD2.ifc): NEW, EXISTING, DEMOLISH, TEMPORARY; the sets Pset_WallCommon, Pset_DoorCommon, Pset_RoofCommon.
cnt() { grep -cF -- "$1" "$T/seven.ifc" || true; }
[ "$(cnt "IFCLABEL('EXISTING')")" = 1 ] && [ "$(cnt "IFCLABEL('DEMOLISH')")" = 2 ] && [ "$(cnt "IFCLABEL('NEW')")" = 2 ] || { echo "FAIL: Status is not 1 existing, 2 to demolish (a wall and the door), 2 new (a wall and the roof)"; grep -n "Status" "$T/seven.ifc"; exit 1; }
printf '    Status: EXISTING x%s, DEMOLISH x%s, NEW x%s\n' "$(cnt "IFCLABEL('EXISTING')")" "$(cnt "IFCLABEL('DEMOLISH')")" "$(cnt "IFCLABEL('NEW')")"
[ "$(cnt "'Pset_WallCommon'")" = 3 ] && [ "$(cnt "'Pset_DoorCommon'")" = 1 ] && [ "$(cnt "'Pset_RoofCommon'")" = 1 ] || { echo "FAIL: the common property sets are not three of walls, one of the door, one of the roof"; exit 1; }
echo "    Pset_WallCommon x3, Pset_DoorCommon x1, Pset_RoofCommon x1"
# LoadBearing true on the wall said bearing and absent on the others.
[ "$(cnt "'LoadBearing'")" = 1 ] && [ "$(cnt "IFCBOOLEAN(.T.)")" = 1 ] || { echo "FAIL: LoadBearing is not written once, true, on the one wall said bearing"; grep -n "LoadBearing" "$T/seven.ifc"; exit 1; }
echo "    LoadBearing x1, true (the wall said bearing); the others are left out"
[ "$(cnt "'Plumbline_Values'")" -ge 3 ] || { echo "FAIL: the program's own property set is gone"; exit 1; }
echo "    the program's own values stay (Plumbline_Values)"
if python3 -I -c "import ifcopenshell" 2>/dev/null; then
  python3 -I tools/ifc-check.py "$T/seven.ifc" | sed 's/^/    outside checker: /' | head -2
  python3 -I tools/ifc-check.py "$T/seven.ifc" > /dev/null || { echo "FAIL: the outside checker does not pass the file"; exit 1; }
else
  echo "    outside checker: NOT RUN here (IfcOpenShell is not installed)"
fi
echo "   read back into an empty job"
"$P" new "$T/back.json" "Read back (not a real job)" >/dev/null
"$P" import ifc "$T/back.json" "$T/seven.ifc" --status Known --source "its own file" > "$T/import7.txt"
says "$T/import7.txt" "3 walls"
says "$T/import7.txt" "1 door"
"$P" show "$T/back.json" > "$T/back-show.txt"
[ "$(grep -c "~ work             existing to remain" "$T/back-show.txt")" = 1 ] || { echo "FAIL: not one thing read back as existing"; exit 1; }
[ "$(grep -c "~ work             to demolish" "$T/back-show.txt")" = 2 ] || { echo "FAIL: not two things (a wall and the door) read back as to demolish"; exit 1; }
echo "    marks read back, each Assumed: 1 existing, 2 to demolish (a wall and the door); the wall said new is new"
grep -q "bearing          bearing" "$T/back-show.txt" || { echo "FAIL: the wall said bearing is not said bearing"; exit 1; }
echo "    the wall said bearing is said bearing"
[ "$(grep -c "type             exterior wall 2x6 (type-" "$T/back-show.txt")" = 3 ] && grep -q "type             entry door 3 ft (type-" "$T/back-show.txt" || { echo "FAIL: the types are not read back by their names"; grep -n "type  " "$T/back-show.txt"; exit 1; }
echo "    types by their names: exterior wall 2x6 on 3 walls, entry door 3 ft on the door"
"$P" takeoff "$J7" | sed -n '/^ground/,/^  .* floors/p' | sed -E 's/^ *[~?=] / /; s/^ +//' > "$T/take7-was.txt"
"$P" takeoff "$T/back.json" | sed -n '/^ground/,/^  .* floors/p' | sed -E 's/^ *[~?=] / /; s/^ +//' > "$T/take7-back.txt"
diff <(grep -E "^(walls|to build|existing|to demolish)" "$T/take7-was.txt") <(grep -E "^(walls|to build|existing|to demolish)" "$T/take7-back.txt") > /dev/null && echo "    the takeoff of the job read back says what the first job's did (walls, to build, existing, to demolish)" || { echo "FAIL: the takeoff of the job read back is not the first job's"; diff "$T/take7-was.txt" "$T/take7-back.txt"; exit 1; }
if grep -q 'roof slabs' "$T/import7.txt"; then echo "    NOT DONE (item 7): the roof is not read back: $(grep -m1 'roof slabs' "$T/import7.txt" | sed 's/^ *//')"; fi
echo
echo "  PASS (so far): items 1, 2, 3, 4 and 7 (the roof is not read back: see the line above)"
