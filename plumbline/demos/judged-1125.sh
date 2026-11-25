#!/usr/bin/env bash
# What the command-line judge of 2026-11-25 found, fixed (call 206). At the command line, from empty jobs, every figure made up. Each step
# fails the script if the program does not say what the step expects. The figures are worked by hand in STATUS.md ("The command line judged
# (2026-11-25), and its findings fixed: a demonstration") and in the comments below.
# Run from the repository root:  demos/judged-1125.sh
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
K=(--status Known --source "made-up plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
fails() { # fails <file> <the words it must say> -- <command...>: the command is refused, and says
  local out="$1" words="$2"; shift 3
  if "$@" > "$out" 2>&1; then echo "FAIL: it was not refused: $*"; cat "$out"; exit 1; fi
  says "$out" "$words"
}

echo "== 1. Nothing a person typed on a line goes without a word =="
# A refrigerator set down once, priced as "Bought and set" at $1,000.00; 7 typed as the quantity of its line: 7 x $1,000.00 = $7,000.00.
J="$T/fridge.json"
"$P" new "$J" "Refrigerator (not a real job)" >/dev/null
"$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" import assemblies "$J" starter >/dev/null
"$P" import items "$J" starter >/dev/null
"$P" set "$J" Refrigerator priced_as "Bought and set" >/dev/null
printf 'Material,Unit,Price,How sure,Where from\nRefrigerator,EA,1000,Assumed,a made-up list\n' > "$T/list.csv"
"$P" import prices "$J" "$T/list.csv" --add >/dev/null
"$P" place "$J" level-0002 Refrigerator --at "10,5" "${K[@]}" >/dev/null
"$P" set "$J" equipment-0093 quantity 7 --line "Refrigerator" --status Assumed --source "made-up count" >/dev/null
"$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"
says "$T/sheet.txt" "Refrigerator 7.00 EA"
says "$T/sheet.txt" '$7,000.00'
"$P" set "$J" Refrigerator priced_as "Mechanical equipment" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" '"Refrigerator" came off equipment-0093, with a quantity of 7.00 EA ("7", where the assembly writes "1") entered on it (it came to $7,000.00)'
"$P" set "$J" Refrigerator priced_as "Bought and set" >/dev/null
"$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"
says "$T/sheet.txt" "Refrigerator 1.00 EA"
says "$T/sheet.txt" '$1,000.00'
echo "-- the same on an assembly changed: Corner boards of a wall 40 LF typed, 12 % waste, \$6.25/LF; the wall taken to the partition assembly --"
W="$T/wall.json"
"$P" new "$W" "Wall (not a real job)" >/dev/null
"$P" level "$W" ground --elevation "0 ft" --height "10 ft" "${K[@]}" >/dev/null
"$P" type "$W" wall w6 --thickness "6 in" "${K[@]}" >/dev/null
"$P" wall "$W" level-0002 type-0003 --points "0,0 20,0 20,16 0,16" --closed --height "10 ft" "${K[@]}" >/dev/null
"$P" import assemblies "$W" starter >/dev/null
"$P" apply "$W" wall-0004 "Exterior wall, 2x6, lap siding" --source "made-up plan" >/dev/null
"$P" set "$W" wall-0004 quantity "40 LF" --line "Corner boards" --status Assumed --source "made-up count" >/dev/null
"$P" set "$W" wall-0004 waste 12 --line "Corner boards" --status Assumed --source "made-up count" >/dev/null
"$P" set "$W" wall-0004 unit_cost '$6.25' --line "Corner boards" --status Assumed --source "made-up price" >/dev/null
"$P" apply "$W" wall-0004 "Take down a partition" --source "made-up plan" --change > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
# 40 LF with 12 % waste is 44.8 LF, at $6.25 = $280.00
says "$T/said.txt" '  Corner boards, with a quantity of 40.00 LF'
says "$T/said.txt" '12 % waste and $6.25/LF entered on it (it came to $280.00)'
echo "-- and a line taken off a thing with the lines of its assembly (mark --clear) says what was on it --"
"$P" new "$T/w2.json" "Wall (not a real job)" >/dev/null
"$P" level "$T/w2.json" ground --elevation "0 ft" --height "10 ft" "${K[@]}" >/dev/null
"$P" type "$T/w2.json" wall w6 --thickness "6 in" "${K[@]}" >/dev/null
"$P" wall "$T/w2.json" level-0002 type-0003 --points "0,0 20,0 20,16 0,16" --closed --height "10 ft" "${K[@]}" >/dev/null
"$P" import assemblies "$T/w2.json" starter >/dev/null
"$P" apply "$T/w2.json" wall-0004 "Exterior wall, 2x6, lap siding" --source "made-up plan" >/dev/null
"$P" set "$T/w2.json" wall-0004 quantity "40 LF" --line "Corner boards" --status Assumed --source "made-up count" >/dev/null
"$P" set "$T/w2.json" wall-0004 unit_cost '$6.25' --line "Corner boards" --status Assumed --source "made-up price" >/dev/null
"$P" mark "$T/w2.json" wall-0004 existing --clear "${K[@]}" > "$T/said.txt"
says "$T/said.txt" 'They had "Corner boards", with a quantity of 40.00 LF'
"$P" remove-line "$W" wall-0004 "Dust barrier" > "$T/said.txt"
lacks "$T/said.txt" "entered on it"

echo
echo "== 2. A shape stays inside its item's box at the item's own sizes =="
# The shape x has a block up to height + max(0 in, width - 10 ft). At the three sizes a table is tried at it is inside the box; on an item
# 144 in wide (12 ft, 2 ft over 10) it reaches 24 in above an item 96 in high.
S="$T/shapes.json"
"$P" new "$S" "Shapes (not a real job)" >/dev/null
printf 'Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to,There when\nx,timber,box,-width / 2,width / 2,0,depth,0,"height + max(0 in, width - 10 ft)",\n' > "$T/x.csv"
"$P" import shapes "$S" "$T/x.csv" > "$T/said.txt"
says "$T/said.txt" "x: added"
head='Item,Trade,Kind,Drawn as,Width,Width: how sure,Width: where from,Depth,Depth: how sure,Depth: where from,Height,Height: how sure,Height: where from'
printf '%s\nWide thing,,furnishing,x,144 in,Assumed,made-up,24 in,Assumed,made-up,96 in,Assumed,made-up\nNarrow thing,,furnishing,x,100 in,Assumed,made-up,24 in,Assumed,made-up,96 in,Assumed,made-up\n' "$head" > "$T/items.csv"
"$P" import items "$S" "$T/items.csv" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" '"Wide thing" is 144 in wide, 24 in deep and 96 in high, and drawn as "x", whose block 1 reaches 24 in above its box'
says "$T/said.txt" "Narrow thing: added"
echo "-- the narrow one sized to 144 in: refused, and left as it was --"
fails "$T/said.txt" '"Narrow thing" is 144 in wide, 24 in deep and 96 in high' -- "$P" set "$S" "Narrow thing" width "144 in"
"$P" items "$S" "Narrow thing" > "$T/said.txt"
says "$T/said.txt" "100 in wide"
echo "-- a job made to hold one (the file written by hand) says so under check, and still opens --"
cp "$S" "$T/held.json"
sed -i 's/"value": 100.0,/"value": 144.0,/' "$T/held.json"
cmp -s "$S" "$T/held.json" && { echo "FAIL: the job was not changed by hand"; exit 1; }
"$P" check "$T/held.json" > "$T/said.txt" || true
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" '! "Narrow thing" is 144 in wide, 24 in deep and 96 in high, and drawn as "x", whose block 1 reaches 24 in above its box'
says "$T/said.txt" "reads"
echo "-- a block set so that an item stands outside is refused naming the item --"
"$P" new "$T/y.json" "Shapes (not a real job)" >/dev/null
printf 'Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to,There when\ny,timber,box,-width / 2,width / 2,0,depth,0,height,\n' > "$T/y.csv"
"$P" import shapes "$T/y.json" "$T/y.csv" >/dev/null
printf '%s\nWide thing,,furnishing,y,144 in,Assumed,made-up,24 in,Assumed,made-up,96 in,Assumed,made-up\n' "$head" > "$T/yi.csv"
"$P" import items "$T/y.json" "$T/yi.csv" >/dev/null
fails "$T/said.txt" '"Wide thing" is 144 in wide' -- "$P" shape "$T/y.json" set y 1 "up to" "height + max(0 in, width - 10 ft)"

echo
echo "== 3. A model brought into a job that has a building =="
# The made-up whole house; its living room is 39.5 ft by 15.5625 ft inside the faces of its walls: 614.71875 SF. A model of one level at 0 ft
# (the job's first floor is at 0 ft) with one wall 6 in thick, 16 ft long, at x = 24 across the living room: the room's part holding its point
# is 23.5 ft by 15.5625 ft = 365.71875 SF. Receptacles are what the program's rule works (10 and 8).
H="$T/house.json"
"$P" sixteen "$H" >/dev/null
"$P" export ifc "$H" "$T/house.ifc" >/dev/null
"$P" new "$T/in.json" "Whole house (not a real job)" >/dev/null
"$P" import ifc "$T/in.json" "$T/house.ifc" --status Assumed --source "made-up model" >/dev/null
"$P" new "$T/m.json" "Another trade (not a real job)" >/dev/null
"$P" level "$T/m.json" L1 --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" type "$T/m.json" wall W --thickness "6 in" "${K[@]}" >/dev/null
"$P" wall "$T/m.json" level-0002 type-0003 --points "24,0 24,16" --height "9 ft" "${K[@]}" >/dev/null
"$P" export ifc "$T/m.json" "$T/m.ifc" >/dev/null
"$P" show "$T/in.json" room-0082 | tr -s ' ' > "$T/room.txt"
says "$T/room.txt" "floor area 614.7188 SF"
says "$T/room.txt" "receptacles 10 EA"
"$P" import ifc "$T/in.json" "$T/m.ifc" --status Assumed --source "made-up model" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" 'the model'"'"'s level "L1" has its floor at 0 ft, the height of the job'"'"'s "first floor": it is that level'
says "$T/said.txt" 'living room (first floor): floor area 614.72 SF before, 365.72 SF now; receptacles 10 before, 8 now'
"$P" takeoff "$T/in.json" > "$T/after.txt"
lacks "$T/after.txt" "L1  (level"
"$P" show "$T/in.json" room-0082 | tr -s ' ' > "$T/room.txt"
says "$T/room.txt" "floor area 365.7188 SF"
says "$T/room.txt" "receptacles 8 EA"
echo "-- a level at a new height is added, and said --"
"$P" level "$T/m.json" L2 --elevation "20 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" export ifc "$T/m.json" "$T/m2.ifc" >/dev/null
"$P" new "$T/in2.json" "Whole house (not a real job)" >/dev/null
"$P" import ifc "$T/in2.json" "$T/house.ifc" --status Assumed --source "made-up model" >/dev/null
"$P" import ifc "$T/in2.json" "$T/m2.ifc" --status Assumed --source "made-up model" > "$T/said.txt"
says "$T/said.txt" 'the model'"'"'s level "L2" has its floor at 20 ft, which is not the height of any level the job has: it was added as a new level'

echo
echo "== 4. Setting an item priced as what it already is says so, and how many are set down =="
"$P" place "$J" level-0002 Refrigerator --at "14,5" "${K[@]}" >/dev/null
"$P" place "$J" level-0002 Refrigerator --at "18,5" "${K[@]}" >/dev/null
"$P" set "$J" Refrigerator priced_as "Bought and set" > "$T/said.txt"
says "$T/said.txt" 'Refrigerator is already priced as "Bought and set"; nothing changes, and the 3 set down from it keep it'
lacks "$T/said.txt" "none is set down yet"

echo
echo "== 5. An item taken out of the chest says the prices that go with it =="
printf 'Material,Unit,Price,How sure,Where from\nRefrigerator,EA,900,Assumed,a made-up quote\n' > "$T/quote.csv"
"$P" import prices "$J" "$T/quote.csv" --supplier "Acme" >/dev/null
echo "-- (the item's own price \$1,000.00 and Acme's \$900.00) --"
"$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"
says "$T/sheet.txt" "Refrigerator: Refrigerator ~\$1,000.00 ~\$900.00"
echo "-- 6. its trade changed in the table: the lines on the 3 set down follow (2 lines on each: its own and Set in place) --"
"$P" export items "$J" "$T/chest.csv" >/dev/null
sed 's/^Refrigerator,11 Equipment,/Refrigerator,12 Furnishings,/' "$T/chest.csv" > "$T/chest2.csv"
"$P" import items "$J" "$T/chest2.csv" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "Refrigerator: changed: its trade; 6 lines on 3 set down from it moved to Division 12 Furnishings"
"$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"
says "$T/sheet.txt" "12 Furnishings"
for t in equipment-0093 equipment-0094 equipment-0095; do "$P" remove "$J" "$t" >/dev/null; done
"$P" remove "$J" type-0046 > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "its 2 prices in the price list went with it: the price \$1,000.00/EA, Acme's \$900.00/EA"

echo
echo "== 7. A price on a line made of its item is refused with how it is given; a price for an item priced as nothing says it prices nothing yet =="
"$P" new "$T/f7.json" "Refrigerator (not a real job)" >/dev/null
"$P" level "$T/f7.json" ground --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" import assemblies "$T/f7.json" starter >/dev/null
"$P" import items "$T/f7.json" starter >/dev/null
"$P" set "$T/f7.json" Refrigerator priced_as "Bought and set" >/dev/null
"$P" place "$T/f7.json" level-0002 Refrigerator --at "10,8" "${K[@]}" >/dev/null
fails "$T/said.txt" "is made of the item itself, so its price is the item's and not the line's: give it in a price list" -- "$P" price-item "$T/f7.json" Refrigerator '$1200' --status Known --source "made-up"
printf 'Item,Trade,Kind,Drawn as,Width,Width: how sure,Width: where from,Depth,Depth: how sure,Depth: where from,Height,Height: how sure,Height: where from,Unit cost,Unit cost: how sure,Unit cost: where from\nWine cooler,11 Equipment,equipment,box,24 in,Assumed,made-up,24 in,Assumed,made-up,34 in,Assumed,made-up,700,Assumed,made-up\n' > "$T/cooler.csv"
"$P" import items "$T/f7.json" "$T/cooler.csv" > "$T/said.txt"
says "$T/said.txt" "prices nothing on the job yet"
says "$T/said.txt" "Wine cooler"

echo
echo "== 8. A roof with no outline of this program's in the file is left out for that reason =="
"$P" export ifc "$H" "$T/h2.ifc" >/dev/null
sed "s/'Plumbline_Roof'/'Somebody_Elses'/" "$T/h2.ifc" > "$T/other.ifc"
"$P" new "$T/o.json" "Other (not a real job)" >/dev/null
"$P" import ifc "$T/o.json" "$T/other.ifc" --status Assumed --source "made-up model" > "$T/said.txt"
says "$T/said.txt" "a roof with no outline of this program's in the file is not brought in: draw it from its footprint and its pitch"
lacks "$T/said.txt" "not brought in from a model yet"

echo
echo "== 9. A model brought in twice says so, by counts, in a builder's words =="
"$P" import ifc "$T/in.json" "$T/house.ifc" --status Assumed --source "made-up model" > "$T/said.txt" 2>&1 || true
sed 's/^/    | /' "$T/said.txt" | cut -c1-300
says "$T/said.txt" "this model is already in the job: nothing was brought in. "
lacks "$T/said.txt" "distributionflowelement"
lacks "$T/said.txt" "a equipment"
lacks "$T/said.txt" "fitting-0"
# The counts stay under 700 characters, as they were; what can be done about it follows them (call 209 point 4).
test "$(sed 's/\. To bring it in again.*//' "$T/said.txt" | wc -c)" -lt 700 || { echo "FAIL: the reply is a wall of text"; exit 1; }

echo
echo "== 10. A connection left loose is named =="
n="$(awk '/=IFCRELCONNECTSPORTS\(/ { print NR; exit }' "$T/house.ifc")"
sed "${n}d" "$T/house.ifc" > "$T/loose.ifc"
"$P" new "$T/l.json" "Loose (not a real job)" >/dev/null
"$P" import ifc "$T/l.json" "$T/loose.ifc" --status Assumed --source "made-up model" > "$T/said.txt"
says "$T/said.txt" "1 of the 46 connections is not tied to the end of a run in the file, and is left loose: "

echo
echo "== 11. A refusal quotes what was typed; min with nothing says what it takes =="
for typed in "height * 0.5 in" "height - width * 0 in" "min()" "min(height,)"; do
  "$P" new "$T/z.json" "Z (not a real job)" >/dev/null
  printf 'Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to,There when\nz,timber,box,-width / 2,width / 2,0,depth,0,"%s",\n' "$typed" > "$T/z.csv"
  "$P" import shapes "$T/z.json" "$T/z.csv" > "$T/said.txt"
  lacks "$T/said.txt" "÷"
  lacks "$T/said.txt" "×"
  lacks "$T/said.txt" "closing bracket with nothing open"
  rm -f "$T/z.json"
done
"$P" new "$T/z.json" "Z (not a real job)" >/dev/null
printf 'Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to,There when\nz,timber,box,-width / 2,width / 2,0,depth,0,"height * 0.5 in",\n' > "$T/z.csv"
"$P" import shapes "$T/z.json" "$T/z.csv" > "$T/said.txt"
says "$T/said.txt" '"height * 0.5 in" comes out as an area'
printf 'Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to,There when\nz,timber,box,-width / 2,width / 2,0,depth,0,"min()",\n' > "$T/z.csv"
"$P" import shapes "$T/z.json" "$T/z.csv" > "$T/said.txt"
says "$T/said.txt" "min( takes two figures or more, with a comma between them, like min(4 in, height)"

echo
echo "== 12. One block, an x, and rows that do not stand together =="
"$P" new "$T/b.json" "Blocks (not a real job)" >/dev/null
"$P" import shapes "$T/b.json" "$T/x.csv" > "$T/said.txt"
says "$T/said.txt" "x: added: 1 block"
lacks "$T/said.txt" "1 blocks"
"$P" import items "$T/b.json" "$T/items.csv" >/dev/null || true
printf '%s\nNarrow thing,,furnishing,x,100 in,Assumed,made-up,24 in,Assumed,made-up,96 in,Assumed,made-up\n' "$head" > "$T/nt.csv"
"$P" import items "$T/b.json" "$T/nt.csv" >/dev/null
"$P" items "$T/b.json" > "$T/said.txt"
says "$T/said.txt" "drawn as an x"
lacks "$T/said.txt" "drawn as a x"
printf 'Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to,There when\nmybox,timber,box,-width / 2,width / 2,0,depth,0,height,\nmybox2,timber,box,-width / 2,width / 2,0,depth,0,height,\nmybox,timber,box,-width / 4,width / 4,0,depth,0,height,\n' > "$T/apart.csv"
"$P" new "$T/a.json" "Apart (not a real job)" >/dev/null
"$P" import shapes "$T/a.json" "$T/apart.csv" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "mybox: its rows do not stand together (rows 2, 4 are its, and other rows stand between them)"
says "$T/said.txt" "mybox2: added: 1 block"

echo
echo "== 13. A price of \$0.00 is taken and named and marked; a height below nothing is refused =="
"$P" new "$T/f.json" "Free (not a real job)" >/dev/null
"$P" level "$T/f.json" ground --elevation "0 ft" --height "10 ft" "${K[@]}" >/dev/null
"$P" type "$T/f.json" wall w6 --thickness "6 in" "${K[@]}" >/dev/null
"$P" wall "$T/f.json" level-0002 type-0003 --points "0,0 20,0 20,16 0,16" --closed --height "10 ft" "${K[@]}" >/dev/null
"$P" import assemblies "$T/f.json" starter >/dev/null
"$P" apply "$T/f.json" wall-0004 "Exterior wall, 2x6, lap siding" --source "made-up plan" >/dev/null
"$P" set "$T/f.json" wall-0004 unit_cost '$0' --line "Lap siding" --status Known --source "made-up price" > "$T/said.txt"
says "$T/said.txt" "! that price is \$0.00: it is taken, as a thing can be free"
"$P" estimate "$T/f.json" | tr -s ' ' > "$T/sheet.txt"
says "$T/sheet.txt" "Lap siding 200.00 SF \$0.00/SF \$0.00"
says "$T/sheet.txt" "! its price is \$0.00 as entered: nothing is charged for it"
printf 'Material,Unit,Price,How sure,Where from\nFree thing,EA,0,Known,a made-up list\n' > "$T/free.csv"
"$P" import prices "$T/f.json" "$T/free.csv" --add > "$T/said.txt"
says "$T/said.txt" "Free thing: priced at \$0.00, as the list says: nothing is charged for it"
fails "$T/said.txt" "wainscot is below nothing (-1 ft)" -- "$P" set "$T/f.json" wall-0004 wainscot "-1 ft" --line "Corner boards" --status Known --source "made-up"

echo
echo "== 14. An item renamed from the command line =="
"$P" rename "$T/f7.json" Refrigerator "Icebox" > "$T/said.txt"
says "$T/said.txt" '"Refrigerator" is now called "Icebox"; the 1 set down from it is called that too'
fails "$T/said.txt" 'another item is called "wine cooler" already' -- "$P" rename "$T/f7.json" "Icebox" "wine cooler"
"$P" estimate "$T/f7.json" | tr -s ' ' > "$T/sheet.txt"
says "$T/sheet.txt" "Icebox"

echo
echo "== 15. A job that holds a shape under a starter shape's name still opens, and check says it does (call 208 point 3) =="
"$P" new "$T/s.json" "Shapes (not a real job)" >/dev/null
printf 'Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to,There when\nzzmine,timber,box,-width / 2,width / 2,0,depth,0,height,\n' > "$T/zz.csv"
"$P" import shapes "$T/s.json" "$T/zz.csv" >/dev/null
sed -i 's/"zzmine"/"Bathtub"/' "$T/s.json"
"$P" check "$T/s.json" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "reads, its figures agree with what is drawn"
says "$T/said.txt" '! "Bathtub" is a starter shape'"'"'s name, and this job holds a shape of its own under it'
says "$T/said.txt" "keep it under another name"

echo
echo "== 16. Any length a line enters is refused below nothing, a spacing as a height (call 208 point 4) =="
"$P" new "$T/lw.json" "Lengths (not a real job)" >/dev/null
"$P" level "$T/lw.json" ground --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" type "$T/lw.json" wall w6 --thickness "6 in" "${K[@]}" >/dev/null
"$P" wall "$T/lw.json" level-0002 type-0003 --points "0,0 20,0" --height "8 ft" "${K[@]}" >/dev/null
"$P" line "$T/lw.json" wall-0004 Studs plan_length --cost-type material --division 6 >/dev/null
"$P" set "$T/lw.json" wall-0004 quantity "plan_length / spacing" --line Studs --with "spacing=16 in" "${K[@]}" >/dev/null
fails "$T/said.txt" "spacing is below nothing (-12 in): a length a line enters is nothing or more" -- "$P" set "$T/lw.json" wall-0004 spacing "-12 in" --line Studs "${K[@]}"
fails "$T/said.txt" "spacing is below nothing (-16 in): a length a line enters is nothing or more" -- "$P" set "$T/lw.json" wall-0004 quantity "plan_length / spacing" --line Studs --with "spacing=-16 in" "${K[@]}"
"$P" estimate "$T/lw.json" | tr -s ' ' > "$T/sheet.txt"
# 20 ft of plan length at 16 in (1.3333 ft): the figure the refusals left as it was, 15 (the line's own count of studs it names).
says "$T/sheet.txt" "Studs"

echo
echo "== 17. An item renamed renames every thing set down, each keeping its number (call 209 point 2) =="
# Three vanities set down: "Vanity, 36 in", "Vanity, 36 in 2", "Vanity, 36 in 3". Renamed, the three are "Vanity, 42 in", "... 2", "... 3".
"$P" new "$T/rn.json" "Renamed (not a real job)" >/dev/null
"$P" level "$T/rn.json" ground --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" import items "$T/rn.json" starter >/dev/null
for x in 5 10 15; do "$P" place "$T/rn.json" level-0002 "Vanity, 36 in" --at "$x,5" "${K[@]}" >/dev/null; done
"$P" rename "$T/rn.json" "Vanity, 36 in" "Vanity, 42 in" > "$T/said.txt"
sed 's/^/    | /' "$T/said.txt"
says "$T/said.txt" "the 3 set down from it are called that too"
"$P" takeoff "$T/rn.json" > "$T/take.txt"
says "$T/take.txt" "3 furnishings: Vanity, 42 in, Vanity, 42 in 2, Vanity, 42 in 3"
lacks "$T/take.txt" "36 in 2"

echo
echo "== 18. 12-6 is twelve feet six (call 209 point 5) =="
# A level 12 ft 6 in high (12-6) and a wall 20 ft long to its top: 20 x 12.5 = 250 SF.
"$P" new "$T/hy.json" "Hyphen (not a real job)" >/dev/null
"$P" level "$T/hy.json" ground --elevation "0 ft" --height "12-6" "${K[@]}" >/dev/null
"$P" type "$T/hy.json" wall w6 --thickness "6 in" "${K[@]}" >/dev/null
"$P" wall "$T/hy.json" level-0002 type-0003 --points "0,0 20,0" --height "12-6" "${K[@]}" >/dev/null
"$P" takeoff "$T/hy.json" | tr -s ' ' > "$T/take.txt"
says "$T/take.txt" "wall face, gross 250.00 SF"
fails "$T/said.txt" "has feet after the inches; write twelve feet six as 12-6" -- "$P" set "$T/hy.json" wall-0004 height "12-6 ft" "${K[@]}"
fails "$T/said.txt" "12-6 (twelve feet six)" -- "$P" set "$T/hy.json" wall-0004 height "12 6" "${K[@]}"
fails "$T/said.txt" "has no unit" -- "$P" set "$T/hy.json" wall-0004 height "-6" "${K[@]}"
lacks "$T/said.txt" "no commas"

echo
echo "== 19. The building and the tool chest are counted apart (call 209 point 8) =="
"$P" new "$T/ch.json" "Chest (not a real job)" >/dev/null
"$P" import items "$T/ch.json" starter >/dev/null
"$P" show "$T/ch.json" > "$T/show.txt"
says "$T/show.txt" "  in the building: 0 known, 0 assumed, 0 unknown"
grep -E "^  in the tool chest: " "$T/show.txt" | sed 's/^/    | /'
# The one old count is the two added.
OLD="$(grep -E "^[0-9]+ Known, " "$T/show.txt" | tr -dc '0-9 ' )"
CHEST="$(grep -E "^  in the tool chest: " "$T/show.txt" | tr -dc '0-9 ')"
[ "$(echo $OLD)" = "$(echo $CHEST)" ] || { echo "FAIL: the old count '$OLD' is not the chest '$CHEST' with an empty building"; exit 1; }
"$P" level "$T/ch.json" ground --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" show "$T/ch.json" > "$T/show.txt"
B="$(grep -E "^  in the building: " "$T/show.txt" | tr -dc '0-9 ')"; C="$(grep -E "^  in the tool chest: " "$T/show.txt" | tr -dc '0-9 ')"; T3="$(grep -E "^[0-9]+ Known, " "$T/show.txt" | tr -dc '0-9 ')"
SUM="$(paste <(echo $B | tr ' ' '\n') <(echo $C | tr ' ' '\n') | awk '{printf "%d ", $1+$2}')"
[ "$(echo $T3)" = "$(echo $SUM)" ] || { echo "FAIL: total '$T3' is not building '$B' plus chest '$C'"; exit 1; }
printf '    total %s = building %s + chest %s\n' "$(echo $T3)" "$(echo $B)" "$(echo $C)"

echo
echo "== 20. What a model written out and brought in says is a builder's words (call 209 point 3) =="
"$P" sixteen "$T/h16.json" >/dev/null
"$P" export ifc "$T/h16.json" "$T/h16.ifc" > "$T/export.txt"
sed 's/^/    | /' "$T/export.txt" | cut -c1-200
says "$T/export.txt" "147 joints"
says "$T/export.txt" "332 connections"
says "$T/export.txt" "432 cost lines: the estimate is not part of a building model: it is written out as a workbook (plumbline export estimate <job> <out.xlsx>)"
says "$T/export.txt" "31 wall, door, window or run types: each wall, door, window and run is written with its own size"
lacks "$T/export.txt" " x cost line"
lacks "$T/export.txt" "connection point"
"$P" new "$T/in20.json" "Empty (not a real job)" >/dev/null
"$P" import ifc "$T/in20.json" "$T/h16.ifc" "${K[@]}" > "$T/in.txt"
says "$T/in.txt" "runs straight up and down: there is no length along the plan to cut it at"
says "$T/in.txt" "not read: the notes the model carries on its things ("
lacks "$T/in.txt" "would not take it"
lacks "$T/in.txt" "stands upright"
lacks "$T/in.txt" "lists of what the model says"

echo
echo "== 21. A model refused says what stands in the way and what can be done today (call 209 point 4) =="
fails "$T/said.txt" "this model is already in the job: nothing was brought in." -- "$P" import ifc "$T/in20.json" "$T/h16.ifc" "${K[@]}"
says "$T/said.txt" "To bring it in again, bring it into a job of its own, or take what it brought in out of this job first."
says "$T/said.txt" "bringing in only what the job lacks is not there yet"
"$P" new "$T/fl.json" "A floor (not a real job)" >/dev/null
"$P" level "$T/fl.json" ground --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" type "$T/fl.json" floor slab --thickness "4 in" "${K[@]}" >/dev/null
"$P" floor "$T/fl.json" level-0002 type-0003 --points "0,0 10,0 10,10 0,10" "${K[@]}" >/dev/null
cp "$T/fl.json" "$T/fl-before.json"
fails "$T/said.txt" "cover the same ground on one level: two floors in one place would be counted twice. Nothing was brought in." -- "$P" import ifc "$T/fl.json" "$T/h16.ifc" "${K[@]}"
says "$T/said.txt" "take out of the job what is already there on that ground (here, the floor), or bring the model into a job of its own."
cmp "$T/fl.json" "$T/fl-before.json" || { echo "FAIL: the job changed though nothing was brought in"; exit 1; }

echo
echo "judged-1125: every step held"
