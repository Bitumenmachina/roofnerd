#!/usr/bin/env bash
# Appliances and small devices drawn as what they are (call 233, contract 233M). At the command line, from an empty job: the tool chest's range, refrigerator,
# dishwasher, washer, dryer, air handler and panelboard are drawn as shapes where they were plain boxes, and nine items were added that the chest lacked
# (a duplex receptacle, a switch, a ceiling light, a recessed light, a smoke alarm, a thermostat, a supply register, a return grille, a shower base).
# The figures are worked by hand below, in STATUS.md ("Starter shapes and items for appliances and small devices") and in research/device-sizes.md.
#   1. the chest is 57 items; the nine new ones are there, each Assumed and saying where its size is from
#   2. the seven that were boxes say what they are drawn as, and their width, depth and height are what they were
#   3. each new shape, set at its item's size and at half again as wide: every block inside the item's box and reaching each of its six faces
#   4. a duplex receptacle set down on a wall at 18 in to its middle: its size is the plate's, 2 3/4 by 1/2 by 4 1/2 in; set against the wall, not a clash
#   5. the shapes and the items written out and brought into an empty job: the same tables, byte for byte
#   6. the sample whole house: plumbline sixteen, its estimate, its clash list and its verify-in-field list are what they were
# Run from the repository root:  PLUMBLINE_BIN=<a folder with plumbline> demos/devices.sh   (without it the script builds the command-line program)
# The job and every figure are made up (no real job data) and live in a temporary folder. Every step ends the script if the program does not say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/devices.json"
K=(--status Known --source "made-up plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
is() { [ "$2" = "$3" ] || { echo "FAIL: $1: $2, not $3"; exit 1; }; printf '    %s: %s\n' "$1" "$2"; }

echo "== 1. The chest: 57 items, the nine new ones there, each Assumed and saying where its size is from =="
"$P" new "$J" "Devices (not a real job)" >/dev/null
"$P" import items "$J" starter > "$T/in.txt"
says "$T/in.txt" "  set: 57"
"$P" items "$J" > "$T/list.txt"
says "$T/list.txt" "the things that can be placed, by trade (57)"
for item in "Duplex receptacle" "Switch" "Ceiling light" "Recessed light, 6 in" "Smoke alarm" "Thermostat" "Supply register, 4 x 10" "Return grille, 20 x 20" "Shower base, 60 x 32"; do
  "$P" items "$J" "$item" > "$T/one.txt"
  says "$T/one.txt" "a typical size; the product is not chosen"
  lacks "$T/one.txt" "unknown"
done
echo "      call 235: six sizes for which no published figure was found say so, each by its own name (the depth of a plate, a register and a grille; the height of a light)"
while IFS='|' read -r item what; do
  "$P" items "$J" "$item" > "$T/one.txt"
  says "$T/one.txt" "$what: an assumption: no published size was found"
done <<'ROWS'
Duplex receptacle|depth
Switch|depth
Supply register, 4 x 10|depth
Return grille, 20 x 20|depth
Ceiling light|height
Recessed light, 6 in|height
ROWS
for item in "Smoke alarm" "Thermostat" "Shower base, 60 x 32"; do
  "$P" items "$J" "$item" > "$T/one.txt"
  lacks "$T/one.txt" "no published size was found"
done
echo "      the plate of a receptacle or a switch: 2 3/4 in wide, 4 1/2 in high (the wiring-device standard's single-gang plate), 1/2 in deep with its device face"
"$P" items "$J" "Duplex receptacle" > "$T/recept.txt"
says "$T/recept.txt" "sizes: 2.75 in wide, 0.5 in deep, 4.5 in high; ~width: a typical size; the product is not chosen; depth: an assumption: no published size was found; height: a typical size; the product is not chosen"
echo "      by hand: the plate is 0.25 in thick (half the depth), the two device faces 0.25 in more; each face 0.7 of the width (1.925 in) across, a third of the height up"
says "$T/recept.txt" "block 1, enamel: across -1.375 in to 1.375 in; back 0 in to 0.25 in; up 0 in to 4.5 in"
says "$T/recept.txt" "block 2, porcelain: across -0.962 in to 0.962 in; back 0.25 in to 0.5 in; up 2.475 in to 4.05 in"
says "$T/recept.txt" "block 3, porcelain: across -0.962 in to 0.962 in; back 0.25 in to 0.5 in; up 0.45 in to 2.025 in"

echo
echo "== 2. The seven that were boxes: drawn as their shapes, their sizes what they were (width, depth, height in inches) =="
while IFS='|' read -r item drawn size; do
  "$P" items "$J" "$item" > "$T/seven.txt"
  says "$T/seven.txt" "drawn as $drawn"
  says "$T/seven.txt" "sizes: $size; ~a typical size; the product is not chosen"
  lacks "$T/seven.txt" "drawn as a box"
done <<'ROWS'
Range, 30 in|a range|30 in wide, 26 in deep, 36 in high
Refrigerator|a refrigerator|36 in wide, 30 in deep, 70 in high
Dishwasher|a dishwasher|24 in wide, 24 in deep, 34 in high
Washer|a laundry machine|27 in wide, 30 in deep, 38 in high
Dryer|a laundry machine|27 in wide, 30 in deep, 38 in high
Air handler|an air handler|24 in wide, 22 in deep, 48 in high
Panelboard, 200 A|a panelboard|15 in wide, 4 in deep, 30 in high
ROWS
echo "      by hand, the range: body 0 to 26 - 1 = 25.016 in back (a 25 mm door), up to 36 - 2.362 = 33.638 in (a 60 mm cooktop); four burners round, 0.14 of the 26 in = 3.64 in each way from their middles"
"$P" items "$J" "Range, 30 in" > "$T/range.txt"
says "$T/range.txt" "its shape is 7 blocks"
says "$T/range.txt" "block 1, enamel: across -15 in to 15 in; back 0 in to 25.016 in; up 0 in to 33.638 in"
says "$T/range.txt" "block 4, steel, round: across -11.14 in to -3.86 in; back 4.16 in to 11.44 in; up 34.819 in to 36 in"

echo
echo "== 3. Each new shape at its item's size and at half again as wide: every block inside the item's box, together reaching each of its six faces =="
# The item's width, depth, height in inches, and the width half again as wide. Read from the program's own words and checked by python.
cat > "$T/boxes.py" <<'PY'
import re, subprocess, sys
P, J = sys.argv[1], sys.argv[2]
items = [("Range, 30 in", 30, 26, 36), ("Refrigerator", 36, 30, 70), ("Dishwasher", 24, 24, 34), ("Washer", 27, 30, 38), ("Dryer", 27, 30, 38),
         ("Air handler", 24, 22, 48), ("Panelboard, 200 A", 15, 4, 30), ("Duplex receptacle", 2.75, 0.5, 4.5), ("Switch", 2.75, 0.5, 4.5),
         ("Ceiling light", 6, 6, 2), ("Recessed light, 6 in", 7.5, 7.5, 6), ("Smoke alarm", 5.625, 5.625, 2), ("Thermostat", 3.3125, 1.5, 4.8125),
         ("Supply register, 4 x 10", 11.625, 0.75, 5.625), ("Return grille, 20 x 20", 21.75, 1, 21.75), ("Shower base, 60 x 32", 60, 32, 2.625)]
line = re.compile(r"block \d+, [a-z]+(?:, round)?: across (\S+) in to (\S+) in; back (\S+) in to (\S+) in; up (\S+) in to (\S+) in")
tried = 0
for name, w, d, h in items:
    out = subprocess.run([P, "items", J, name], capture_output=True, text=True, check=True).stdout
    sizes = re.search(r"sizes: (\S+) in wide, (\S+) in deep, (\S+) in high", out)
    assert all(abs(float(x) - y) < 0.002 for x, y in zip(sizes.groups(), (w, d, h))), (name, sizes.groups())
    blocks = [[float(x) for x in m.groups()] for m in map(line.search, out.splitlines()) if m]
    assert len(blocks) >= 3, (name, len(blocks))
    lo = [min(b[i] for b in blocks) for i in (0, 2, 4)]
    hi = [max(b[i + 1] for b in blocks) for i in (0, 2, 4)]
    # across is from the middle, back is from the back (0 to the depth), up is from the foot
    want_lo, want_hi = [-w / 2, 0, 0], [w / 2, d, h]
    for k in range(3):
        assert abs(lo[k] - want_lo[k]) < 0.002 and abs(hi[k] - want_hi[k]) < 0.002, (name, "does not fill its box", lo, hi)
    for b in blocks:
        assert b[0] >= -w / 2 - 0.002 and b[1] <= w / 2 + 0.002 and b[2] >= -0.002 and b[3] <= d + 0.002 and b[4] >= -0.002 and b[5] <= h + 0.002, (name, "outside its box", b)
    tried += 1
    print("    %-26s %d blocks, inside the box and reaching all six faces at %g x %g x %g in" % (name, len(blocks), w, d, h))
print("    %d items" % tried)
PY
python3 -I "$T/boxes.py" "$P" "$J" > "$T/boxes.txt" || { cat "$T/boxes.txt"; echo "FAIL: a shape is not true to its item's box"; exit 1; }
cat "$T/boxes.txt"
says "$T/boxes.txt" "16 items"
echo "-- half again as wide: the width of each typed one and a half times (a second job, the widths from its own table) --"
"$P" export items "$J" "$T/chest.csv" >/dev/null
python3 -I - "$T/chest.csv" "$T/wide.csv" <<'PY'
import csv, sys
rows = list(csv.reader(open(sys.argv[1])))
new = {"Range, 30 in": 45, "Refrigerator": 54, "Dishwasher": 36, "Washer": 40.5, "Dryer": 40.5, "Air handler": 36, "Panelboard, 200 A": 22.5, "Duplex receptacle": 4.125, "Switch": 4.125,
       "Ceiling light": 9, "Recessed light, 6 in": 11.25, "Smoke alarm": 8.4375, "Thermostat": 4.96875, "Supply register, 4 x 10": 17.4375, "Return grille, 20 x 20": 32.625, "Shower base, 60 x 32": 90}
for r in rows[1:]:
    if r[0] in new:
        r[4] = "%g in" % new[r[0]]
csv.writer(open(sys.argv[2], "w"), lineterminator="\n").writerows(rows)
PY
J2="$T/wide.json"
"$P" new "$J2" "Wide (not a real job)" >/dev/null
"$P" import items "$J2" "$T/wide.csv" > "$T/wide-in.txt"
says "$T/wide-in.txt" "not used: 0"
sed -e 's/("Range, 30 in", 30,/("Range, 30 in", 45,/' -e 's/("Refrigerator", 36,/("Refrigerator", 54,/' -e 's/("Dishwasher", 24,/("Dishwasher", 36,/' -e 's/("Washer", 27,/("Washer", 40.5,/' -e 's/("Dryer", 27,/("Dryer", 40.5,/' \
    -e 's/("Air handler", 24,/("Air handler", 36,/' -e 's/("Panelboard, 200 A", 15,/("Panelboard, 200 A", 22.5,/' -e 's/("Duplex receptacle", 2.75,/("Duplex receptacle", 4.125,/' -e 's/("Switch", 2.75,/("Switch", 4.125,/' \
    -e 's/("Ceiling light", 6,/("Ceiling light", 9,/' -e 's/("Recessed light, 6 in", 7.5,/("Recessed light, 6 in", 11.25,/' -e 's/("Smoke alarm", 5.625,/("Smoke alarm", 8.4375,/' -e 's/("Thermostat", 3.3125,/("Thermostat", 4.96875,/' \
    -e 's/("Supply register, 4 x 10", 11.625,/("Supply register, 4 x 10", 17.4375,/' -e 's/("Return grille, 20 x 20", 21.75,/("Return grille, 20 x 20", 32.625,/' -e 's/("Shower base, 60 x 32", 60,/("Shower base, 60 x 32", 90,/' \
    -e 's/, 21.75, 1, 21.75)/, 32.625, 1, 21.75)/' "$T/boxes.py" > "$T/boxes-wide.py"
python3 -I "$T/boxes-wide.py" "$P" "$J2" > "$T/boxes2.txt" || { cat "$T/boxes2.txt"; echo "FAIL: a shape is not true to its item's box at half again as wide"; exit 1; }
cat "$T/boxes2.txt"
says "$T/boxes2.txt" "16 items"

echo
echo "== 4. A duplex receptacle on a wall, 18 in to its middle: the plate's size, set against the wall, not a clash =="
echo "      a wall 6 in thick along y = 10 from x 0 to 20: its south face is y = 9.75. The plate 2 3/4 by 1/2 in, middle at (10, 9.75): its back 1/4 in into the face. Underside 18 - 2.25 = 15.75 in up."
LV=$("$P" level "$J" Ground --elevation "0 ft" --height "9 ft" "${K[@]}" | sed -n 's/^\(level-[0-9]*\):.*/\1/p')
WT=$("$P" type "$J" wall "Plain" --thickness "6 in" "${K[@]}" | sed -n 's/^\(type-[0-9]*\):.*/\1/p')
[ -n "$LV" ] && [ -n "$WT" ] || { echo "FAIL: the level or the wall type was not made"; exit 1; }
"$P" wall "$J" "$LV" "$WT" --points "0,10 20,10" --height "9 ft" "${K[@]}" >/dev/null
"$P" place "$J" "$LV" "Duplex receptacle" --at "10,9.75" --offset "15.75 in" "${K[@]}" > "$T/placed.txt"
says "$T/placed.txt" "one of Duplex receptacle, 0' 2 3/4\" wide, 0' 0 1/2\" deep, 0' 4 1/2\" high"
THING=$(sed -n 's/^\([a-z]*-[0-9]*\):.*/\1/p' "$T/placed.txt")
"$P" show "$J" "$THING" > "$T/thing.txt"
says "$T/thing.txt" "size             2 3/4 × 1/2 × 4 1/2 in"
says "$T/thing.txt" "height above its level 15 3/4 in"
"$P" clashes "$J" > "$T/clash.txt"
says "$T/clash.txt" "In the same space: 0"
says "$T/clash.txt" "Set against (stands against it, or is joined to it, by no more than 2.00 in: not a clash): 1"
says "$T/clash.txt" "set 0.25 in into it, at (10, 9.7604, 1.5)"

echo
echo "== 5. The shapes and the items written out and brought into an empty job: the same tables, byte for byte =="
"$P" export shapes "$J" "$T/shapes1.csv" > "$T/out.txt"
"$P" export items "$J" "$T/items1.csv" >/dev/null
J3="$T/third.json"
"$P" new "$J3" "Third (not a real job)" >/dev/null
"$P" import shapes "$J3" "$T/shapes1.csv" > "$T/third-shapes.txt"
says "$T/third-shapes.txt" "not used: 0"
"$P" import items "$J3" "$T/items1.csv" > "$T/third-items.txt"
says "$T/third-items.txt" "not used: 0"
says "$T/third-items.txt" "  set: 57"
"$P" export shapes "$J3" "$T/shapes2.csv" >/dev/null
"$P" export items "$J3" "$T/items2.csv" >/dev/null
cmp "$T/shapes1.csv" "$T/shapes2.csv" && echo "    the shapes table: the same to the byte"
cmp "$T/items1.csv" "$T/items2.csv" && echo "    the items table: the same to the byte"
cmp "$T/items1.csv" library/items.csv && echo "    the items written out are library/items.csv, to the byte"
[ "$(grep -c '^' "$T/items1.csv")" = 58 ] && echo "    58 lines: the headings and 57 items"
grep -qF '"shapes"' "$J3" && { echo "FAIL: the job carries a starter shape of its own"; exit 1; } || echo "    and the saved job carries no starter shape (they are in the program)"

echo
echo "== 6. The sample whole house is what it was (sums of copies made before call 233) =="
"$P" sixteen "$T/house.json" >/dev/null
is "plumbline sixteen (sha256 begins)" "$(sha256sum "$T/house.json" | cut -c1-16)" "880d9fd3c364b5f7"
is "its estimate (sha256 begins)" "$("$P" estimate "$T/house.json" | sha256sum | cut -c1-16)" "b07a6b07390ff523"
is "its clash list (sha256 begins)" "$("$P" clashes "$T/house.json" | sha256sum | cut -c1-16)" "0250881a220082d8"
is "its verify-in-field list (sha256 begins)" "$("$P" vif "$T/house.json" | sha256sum | cut -c1-16)" "14bd89dcaf5908e3"
echo
echo "PASS: the chest is 57 items; the seven that were boxes are drawn as a range, a refrigerator, a dishwasher, a laundry machine (washer and dryer), an air handler and a panelboard at the sizes they had; the nine new items are Assumed; each of the 14 new shapes fills its item's box at its size and at half again as wide; a receptacle on a wall at 18 in is its plate's size and set against the wall; the two tables come back to the byte; the sample whole house is what it was."
