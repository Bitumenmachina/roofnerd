#!/usr/bin/env bash
# An item's shape as data (call 198, the bench's step C). At the command line, from an empty job: the starter tool chest, the 36 in vanity's shape
# printed as the blocks its rows give; the thirteen starter shapes held to the code they replaced; a shape of the user's own brought in with the chest,
# an item drawn as it, its blocks worked by hand below; the two tables written out, brought into a second job and written again, the same to the byte,
# and an item naming a shape that is not there refused; a job saved before opens as it did; a cost line's quantity written min(net_area, 100 SF); and (call 205)
# a shape changed by hand: kept as a new shape, a block added and its figures set, a figure that leaves the box refused, a starter shape not changed, a
# starter's name refused on the way in, a block taken away, and the job saved, opened and saved again the same to the byte.
# The figures are worked by hand below and in STATUS.md ("The bench, step C: an item's shape as data. A demonstration").
# Run from the repository root:  demos/shapes.sh
# The job and every figure are made up (no real job data) and live in a temporary folder. Every step fails the script if the program does not say
# what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/shapes.json"
K=(--status Known --source "made-up plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }

echo "== An empty job and the starter tool chest =="
"$P" new "$J" "Shapes (not a real job)" >/dev/null
"$P" import items "$J" starter >/dev/null

echo
echo "== 1. The 36 in vanity: 36 wide, 21 deep, 34 1/2 high. Plinth 100 mm (14 % of the height would be 122.7), top 40 mm, toe 60 mm, lip 20 mm =="
echo "      plinth: back 0 to 21 - 2.362 = 18.638 in, up 0 to 3.937 in;  case: back 0 to 21 - 0.787 = 20.213 in, up 3.937 to 34.5 - 1.575 = 32.925 in;  top: the whole 21 in, up 32.925 to 34.5 in"
"$P" items "$J" "Vanity, 36 in" > "$T/vanity.txt"
sed 's/^/    | /' "$T/vanity.txt"
says "$T/vanity.txt" "its shape is 3 blocks"
says "$T/vanity.txt" "block 1, timber: across -18 in to 18 in; back 0 in to 18.638 in; up 0 in to 3.937 in"
says "$T/vanity.txt" "block 2, timber: across -18 in to 18 in; back 0 in to 20.213 in; up 3.937 in to 32.925 in"
says "$T/vanity.txt" "block 3, worktop: across -18 in to 18 in; back 0 in to 21 in; up 32.925 in to 34.5 in"
says "$T/vanity.txt" "a block is neither Known nor Assumed; the sizes above are"
says "$T/vanity.txt" "sizes: 36 in wide, 21 in deep, 34.5 in high; ~"

echo
echo "== 2. The starter shapes, held to the code they replaced, block for block, at three sizes (a hundred-millionth of a foot) =="
if [ -n "${PLUMBLINE_BIN:-}" ]; then
  echo "    (not run here: PLUMBLINE_BIN builds nothing; run: cargo test -p plumbline-world --test shapes)"
else
  cargo test --quiet -p plumbline-world --test shapes -- --nocapture 2>&1 | tee "$T/held.txt" | grep -E "^held|test result" | sed 's/^/    | /'
  says "$T/held.txt" "held: 13 shapes, 39 shape-and-size pairs, 141 blocks compared"
  says "$T/held.txt" "0 failed"
fi

echo
echo "== 3. A shape of the user's own: a wall-hung closet, 16 wide, 26 deep, 30 high =="
echo "      tank: across -8 to 8 in, back 0 to 6.5 in (a quarter of the depth), up 12 to 30 in;  bowl, round: across -7.2 to 7.2 in (0.45 of the width each way), back 7.8 to 26 in, up 4 to 12 in (clear of the floor)"
cat > "$T/closet-shape.csv" <<'ROWS'
Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to
wall-hung closet,porcelain,box,-width / 2,width / 2,0,0.25 * depth,0.4 * height,height
wall-hung closet,porcelain,round,-0.45 * width,0.45 * width,0.3 * depth,depth,4 in,0.4 * height
ROWS
cat > "$T/closet-item.csv" <<'ROWS'
Item,Trade,Kind,Drawn as,Width,Depth,Height,How sure,Where from
Wall-hung closet,15 Mechanical,fixture,wall-hung closet,16 in,26 in,30 in,Assumed,a made-up size
ROWS
echo "-- (the item first, before the shape it names: refused, naming it) --"
"$P" import items "$J" "$T/closet-item.csv" > "$T/refused.txt" || true
says "$T/refused.txt" "\"wall-hung closet\" is not something a thing can be drawn as"
lacks "$T/refused.txt" "Wall-hung closet: added"
"$P" import shapes "$J" "$T/closet-shape.csv" > "$T/shape-in.txt"
says "$T/shape-in.txt" "wall-hung closet: added: 2 blocks"
"$P" import items "$J" "$T/closet-item.csv" > "$T/item-in.txt"
says "$T/item-in.txt" "Wall-hung closet: added"
"$P" items "$J" "Wall-hung closet" > "$T/closet.txt"
sed 's/^/    | /' "$T/closet.txt"
says "$T/closet.txt" "drawn as a wall-hung closet"
says "$T/closet.txt" "its shape is 2 blocks"
says "$T/closet.txt" "block 1, porcelain: across -8 in to 8 in; back 0 in to 6.5 in; up 12 in to 30 in"
says "$T/closet.txt" "block 2, porcelain, round: across -7.2 in to 7.2 in; back 7.8 in to 26 in; up 4 in to 12 in"
"$P" items "$J" | grep -F "Wall-hung closet" > "$T/list.txt"
says "$T/list.txt" "drawn as a wall-hung closet"
echo "-- (a shape that cannot be worked is not used, and says which block and which figure) --"
cat > "$T/bad.csv" <<'ROWS'
Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to
bad shape,timber,box,-width / 2,width / 2,0,depth,0,height
bad shape,timber,box,-width / 2,width / 2,0,width * depth,0,height
ROWS
"$P" import shapes "$J" "$T/bad.csv" > "$T/bad.txt"
sed 's/^/    | /' "$T/bad.txt"
says "$T/bad.txt" "block 2, Back to"
says "$T/bad.txt" "comes out as an area"

echo
echo "== 4. The chest as two tables: written out, brought into a second job, written again: the same to the byte =="
"$P" export items "$J" "$T/items1.csv" >/dev/null
"$P" export shapes "$J" "$T/shapes1.csv" > "$T/out.txt"
sed 's/^/    | /' "$T/out.txt"
J2="$T/second.json"
"$P" new "$J2" "Second (not a real job)" >/dev/null
"$P" import shapes "$J2" "$T/shapes1.csv" >/dev/null
"$P" import items "$J2" "$T/items1.csv" > "$T/second-in.txt"
says "$T/second-in.txt" "not used: 0"
"$P" export items "$J2" "$T/items2.csv" >/dev/null
"$P" export shapes "$J2" "$T/shapes2.csv" >/dev/null
cmp "$T/items1.csv" "$T/items2.csv" && echo "    the items table: the same to the byte"
cmp "$T/shapes1.csv" "$T/shapes2.csv" && echo "    the shapes table: the same to the byte"
grep -c "^wall-hung closet," "$T/shapes2.csv" | grep -qx 2 && echo "    the closet's two blocks came back"
echo "-- (the items alone, into a third job with no shape of its own: refused, each naming the shape) --"
J3="$T/third.json"
"$P" new "$J3" "Third (not a real job)" >/dev/null
"$P" import items "$J3" "$T/items1.csv" > "$T/third-in.txt" || true
says "$T/third-in.txt" "\"wall-hung closet\" is not something a thing can be drawn as"
echo "-- (the job carries the shape it names, and only that one) --"
grep -c '"shapes"' "$J" | grep -qx 1 && echo "    the saved job has a shapes list"
grep -qF '"name": "wall-hung closet"' "$J" && echo "    carrying the wall-hung closet"
grep -qF 'base cabinet' <(sed -n '/"shapes"/,/"data"/p' "$J") && { echo "FAIL: the job carries a starter shape"; exit 1; } || echo "    and no starter shape (they are in the program)"

echo
echo "== 5. A job saved before: no shapes in it, it opens, the thing set down has the blocks it had, the estimate is what it was =="
"$P" house "$T/house.json" >/dev/null
if grep -q '"shapes"' "$T/house.json"; then echo "FAIL: a job that carries no shape of its own has a shapes list"; exit 1; fi
echo "    the sample house has no shapes list"
"$P" estimate "$T/house.json" > "$T/estimate-before.txt"
cp "$T/house.json" "$T/house-before.json"
"$P" import shapes "$T/house.json" "$T/shapes1.csv" >/dev/null
"$P" import shapes "$T/house.json" library/shapes.csv > "$T/starter-in.txt"
says "$T/starter-in.txt" "the same as the job already has: 13"
"$P" estimate "$T/house.json" > "$T/estimate-after.txt"
cmp "$T/estimate-before.txt" "$T/estimate-after.txt" && echo "    its estimate is the same to the byte after the shapes are brought in"
"$P" check "$T/house-before.json" >/dev/null && echo "    the saved job opens and checks"

echo
echo "== 6. A cost line's quantity written min(net_area, 100 SF): a wall 12 ft by 10 ft (120 SF net) reads 100.00 SF; a wall 8 ft by 10 ft (80 SF), 80.00 SF =="
J4="$T/walls.json"
"$P" new "$J4" "Walls (not a real job)" >/dev/null
"$P" level "$J4" ground --elevation "0 ft" --height "10 ft" "${K[@]}" >/dev/null
"$P" type "$J4" wall "Plain" --thickness "6 in" "${K[@]}" >/dev/null
"$P" wall "$J4" level-0002 type-0003 --points "0,0 12,0" --height "10 ft" "${K[@]}" >/dev/null
"$P" wall "$J4" level-0002 type-0003 --points "0,10 8,10" --height "10 ft" "${K[@]}" >/dev/null
"$P" line "$J4" wall-0004 "Siding cap" "min(net_area, 100 SF)" >/dev/null
"$P" line "$J4" wall-0005 "Siding cap" "min(net_area, 100 SF)" >/dev/null
"$P" estimate "$J4" | tr -s ' ' > "$T/walls.txt"
says "$T/walls.txt" "Siding cap 100.00 SF"
says "$T/walls.txt" "Siding cap 80.00 SF"
echo
echo "== 7. A shape changed by hand (call 205): the 36 in vanity (36 x 21 x 34 1/2 in), set down twice, beside the base cabinet (36 x 24 x 34 1/2 in) =="
J5="$T/hand.json"
"$P" new "$J5" "By hand (not a real job)" >/dev/null
"$P" level "$J5" ground --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
"$P" import items "$J5" starter >/dev/null
"$P" place "$J5" level-0002 "Vanity, 36 in" --at "5,5" "${K[@]}" >/dev/null
"$P" place "$J5" level-0002 "Vanity, 36 in" --at "9,5" "${K[@]}" >/dev/null
echo "-- 7.1 the vanity's shape kept as a new shape: the job has one of its own, three blocks, word for word the base cabinet's; the vanity is drawn as it, the base cabinet still as base cabinet; 1 item and 2 set down follow --"
"$P" shape "$J5" keep "Vanity, 36 in" "vanity with a panel" > "$T/keep.txt"
sed 's/^/    | /' "$T/keep.txt"
says "$T/keep.txt" "kept as a new shape from base cabinet"
says "$T/keep.txt" "1 item is drawn as it, and 2 things set down from it follow."
"$P" shape "$J5" > "$T/own.txt"
says "$T/own.txt" "vanity with a panel: 3 blocks; 1 item drawn as it, 2 things set down from them"
"$P" shape "$J5" "vanity with a panel" > "$T/panel.txt"
says "$T/panel.txt" "block 3, worktop: across -width / 2 to width / 2; back 0 to depth; up height - min(40 mm, 0.06 * height) to height"
"$P" items "$J5" "Vanity, 36 in" | grep -qF "drawn as a vanity with a panel" && echo "    the vanity is drawn as a vanity with a panel"
"$P" items "$J5" "Base cabinet, 36 in" | grep -qF "drawn as a base cabinet" && echo "    the base cabinet is still drawn as a base cabinet"
echo "-- 7.2 a block added and its six figures typed: across -width / 2 + 2 in to width / 2 - 2 in = -16 to 16 in; back depth - 1 in to depth = 20 to 21 in; up 4 in to height - 6 in = 28 1/2 in. Four blocks --"
"$P" shape "$J5" add "vanity with a panel" --made timber > "$T/add.txt"
says "$T/add.txt" "block 4 added, the whole size of the item"
says "$T/add.txt" "it has 4 blocks now"
"$P" shape "$J5" set "vanity with a panel" 4 "across from" "-width / 2 + 2 in" >/dev/null
"$P" shape "$J5" set "vanity with a panel" 4 "across to" "width / 2 - 2 in" >/dev/null
"$P" shape "$J5" set "vanity with a panel" 4 "back from" "depth - 1 in" >/dev/null
"$P" shape "$J5" set "vanity with a panel" 4 "back to" "depth" >/dev/null
"$P" shape "$J5" set "vanity with a panel" 4 "up from" "4 in" >/dev/null
"$P" shape "$J5" set "vanity with a panel" 4 "up to" "height - 6 in" > "$T/figures.txt"
sed 's/^/    | /' "$T/figures.txt"
says "$T/figures.txt" "Block 4 reads, at Vanity, 36 in's sizes (36 in x 21 in x 34.5 in): timber: across -16 in to 16 in; back 20 in to 21 in; up 4 in to 28.5 in."
"$P" items "$J5" "Vanity, 36 in" > "$T/vanity4.txt"
says "$T/vanity4.txt" "its shape is 4 blocks"
says "$T/vanity4.txt" "block 4, timber: across -16 in to 16 in; back 20 in to 21 in; up 4 in to 28.5 in"
cp "$J5" "$T/hand-two.json"
echo "-- 7.3 its Up to set as height + 1 in: refused, naming the block, the size it was tried at and that it reaches 1 in above the box; the job as it was --"
cp "$J5" "$T/hand-before.json"
"$P" shape "$J5" set "vanity with a panel" 4 "up to" "height + 1 in" > "$T/over.txt" 2>&1 || true
sed 's/^/    | /' "$T/over.txt"
says "$T/over.txt" "block 4"
says "$T/over.txt" "small size (0.4 x 0.3 x 0.5 ft)"
says "$T/over.txt" "1 in above the box"
cmp "$J5" "$T/hand-before.json" && echo "    the job is the same to the byte"
echo "-- 7.4 a block added to base cabinet: refused, a starter shape is not changed --"
"$P" shape "$J5" add "base cabinet" > "$T/starter.txt" 2>&1 || true
sed 's/^/    | /' "$T/starter.txt"
says "$T/starter.txt" "a starter shape is not changed"
says "$T/starter.txt" "keep it as a new shape first"
cmp "$J5" "$T/hand-before.json" && echo "    the job is the same to the byte"
echo "-- 7.5 a table of shapes with one called base cabinet: refused, naming it (the others still come in) --"
cat > "$T/starter-name.csv" <<'ROWS'
Shape,Made of,Form,Across from,Across to,Back from,Back to,Up from,Up to
base cabinet,timber,box,-width / 2,width / 2,0,depth,0,height
plain box shape,timber,box,-width / 2,width / 2,0,depth,0,height
ROWS
"$P" import shapes "$J5" "$T/starter-name.csv" > "$T/name-in.txt"
sed 's/^/    | /' "$T/name-in.txt"
says "$T/name-in.txt" "\"base cabinet\" is a starter shape's name"
says "$T/name-in.txt" "plain box shape: added: 1 block"
echo "-- 7.6 the fourth block taken away: 3 blocks, the figures of 7.1 --"
cp "$T/hand-two.json" "$J5"
"$P" shape "$J5" take "vanity with a panel" 4 > "$T/take.txt"
sed 's/^/    | /' "$T/take.txt"
says "$T/take.txt" "vanity with a panel: block 4 taken away"
says "$T/take.txt" "it has 3 blocks now"
"$P" items "$J5" "Vanity, 36 in" > "$T/vanity3.txt"
says "$T/vanity3.txt" "block 1, timber: across -18 in to 18 in; back 0 in to 18.638 in; up 0 in to 3.937 in"
says "$T/vanity3.txt" "block 3, worktop: across -18 in to 18 in; back 0 in to 21 in; up 32.925 in to 34.5 in"
lacks "$T/vanity3.txt" "block 4"
echo "-- 7.7 the job saved after 7.2, opened and saved again: the same to the byte; written out as a table the shape has its four rows --"
"$P" check "$T/hand-two.json" >/dev/null && echo "    the job with the fourth block opens and checks"
"$P" shape "$T/hand-two.json" "vanity with a panel" > "$T/hand-list.txt"
says "$T/hand-list.txt" "vanity with a panel (the job's own): 4 blocks"
"$P" export shapes "$T/hand-two.json" "$T/hand-shapes.csv" >/dev/null
test "$(grep -c '^vanity with a panel,' "$T/hand-shapes.csv")" = 4 && echo "    the shapes table has the four rows"
cp "$T/hand-two.json" "$T/hand-two-saved.json"
"$P" import shapes "$T/hand-two.json" "$T/hand-shapes.csv" > "$T/again.txt"
says "$T/again.txt" "the same as the job already has: 1"
cmp "$T/hand-two.json" "$T/hand-two-saved.json" && echo "    opened, the same table brought in, saved again: the job is the same to the byte"
"$P" export shapes "$T/hand-two.json" "$T/hand-shapes2.csv" >/dev/null
cmp "$T/hand-shapes.csv" "$T/hand-shapes2.csv" && echo "    written out again: the same to the byte"

echo
echo "shapes: every step passed"
