#!/usr/bin/env bash
# A wall under a rake rises to the roof (call 192, the model's half). From an empty job at the command line: the house of
# demos/trim.sh (a box 24 by 16 ft on the walls' lines, walls 6 in thick and 9 ft high, a roof drawn 1 ft outside the lines all
# round, 6 in 12). The gable ends of a gable roof, a shed roof and a hip roof, and a house with no roof: each wall's gable area
# and its top as points, said by the program and set beside the figure worked by hand; the starter outside wall priced on
# net_area + gable_area, the inside of it not; the frame of a gable said not to be counted.
# Call 204 (step 6): the corner boards of a wall under a rake or the high edge of a shed roof are as tall as the wall is at each corner
# (STATUS.md, "A corner board as tall as its wall at that corner: a demonstration", points 1 to 7).
# Run from the repository root:  demos/gable.sh
# The job and every price are made up (no real job data) and live in a temporary folder.
# Every step fails the script if the program does not say what the step expects. The figures are worked by hand in STATUS.md
# ("A wall under a rake rises to the roof: a demonstration, written before it is built"); each is printed beside what was said.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
A=(--status Assumed --source "made-up tracing")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
# The box with the roof of these edges: house <file> <title> <edges: pitch or gable for each of the four edges, south first, or none>
house() {
    local J="$1" edges="$2"
    "$P" new "$J" "Gable (not a real job)" >/dev/null
    local S WT
    S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${A[@]}")")"
    WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${A[@]}")")"
    read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${A[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
    [ "$edges" = none ] && return 0
    local R i=0 e
    R="$(made "$("$P" roof "$J" "Main roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 6:12 --eave-height "9 ft" "${A[@]}")")"
    for e in $edges; do
        i=$((i + 1))
        [ "$e" = G ] && "$P" set "$J" "$R" "edge:$i" gable "${A[@]}" >/dev/null
    done
    return 0
}
# A figure of the line the program said, set beside the hand figure: hand "<file>" "<what>" "<by hand>" "<text the line holds>"
hand() {
    local line
    line="$(grep -F -- "$4" "$1" | head -1 | sed 's/^ *//')" || true
    [ -n "$line" ] || { echo "FAIL: nothing says: $4 ($2)"; cat "$1"; exit 1; }
    printf '    %-34s by hand %-26s said: %s\n' "$2" "$3" "$line"
}
# The sum of a line's quantity and its dollars over every thing it is on: sum "<estimate csv>" "<line>" "<by hand: quantity>" "<by hand: dollars, or none>"
sum() {
    python3 - "$1" "$2" "$3" "$4" <<'PY'
import csv, sys
path, line, want_q, want_d = sys.argv[1:5]
rows = [r for r in csv.DictReader(open(path)) if r["Line"] == line]
assert rows, "no line %r in the estimate" % line
q = sum(float(r["Quantity"]) for r in rows if r["Quantity"] != "")
d = sum(float(r["Amount"]) for r in rows if r["Amount"] != "")
priced = [r for r in rows if r["Amount"] != ""]
got_q = "%.2f" % q
got_d = "%.2f" % d if priced else "none"
print("    %-22s %d on %-22s by hand %8s SF  %-8s said %8s  %s" % (line, len(rows), ", ".join(r["Element"] for r in rows)[:22], want_q, want_d, got_q, got_d))
if got_q != want_q or got_d != want_d:
    print("FAIL: %s: by hand %s and %s, the program said %s and %s" % (line, want_q, want_d, got_q, got_d)); sys.exit(1)
PY
}

echo "== 1. A gable roof (its ridge east-west): the east and the west wall rise to the ridge; the south and the north are under an eave =="
J="$T/gable.json"
house "$J" "P G P G"
"$P" gable "$J" > "$T/g1.txt"
echo "  the roof is 9 ft at its eave (1 ft out) and half a foot higher for each foot in: 9.5 ft over the lines of the south and north walls, 13.5 ft at the ridge, 8 ft in."
echo "  each end wall, 16 ft: 16 x 0.5 + 16 x 4 / 2 = 8 + 32 = 40.00 SF; its top (0, 9.5), (8, 13.5), (16, 9.5); assumed, as the tracing is:"
hand "$T/g1.txt" "$EAST" "40 SF, 9.5 13.5 9.5" "[$EAST]: gable ~40 SF; top (0, 9.5) (8, 13.5) (16, 9.5) ft"
hand "$T/g1.txt" "$WEST" "40 SF, 9.5 13.5 9.5" "[$WEST]: gable ~40 SF; top (0, 9.5) (8, 13.5) (16, 9.5) ft"
echo "  each says what it rests on (the wall, the rake over it, the plane of the roof):"
says "$T/g1.txt" "rests on: $EAST#pl::wall, rake-"
# Call 197, point 7: "none" is marked ~ where it is Assumed, as `show` marks it (call 192: none takes the weakest of the
# roof, the eave and the wall, and every house here is traced, so Assumed). The four checks of "none" below moved from
# "none: ..." to "~ none: ..." by that decision, and by nothing else.
echo "  the south and the north wall: an eave is over each, none, and it says so:"
says "$T/g1.txt" "[$SOUTH]: ~ none: an eave is over it"
says "$T/g1.txt" "[$NORTH]: ~ none: an eave is over it"
echo "  a thing no line asks the figure of does not carry it, and net_area is what it was (16 x 9 = 144 SF on the west wall):"
"$P" show "$J" "$WEST" > "$T/show-west.txt"
lacks "$T/show-west.txt" "gable area"
hand "$T/show-west.txt" "$WEST, net area" "144 SF" "net area"
echo "  the frame of a gable is not counted: said on the wall's framing figures where the wall has one (the starter outside wall names its studs):"
"$P" import assemblies "$J" starter > "$T/imp.txt"
says "$T/imp.txt" "not used: 0"
for w in $SOUTH $EAST $NORTH $WEST; do "$P" apply "$J" "$w" "Exterior wall, 2x6, lap siding" --source "the starter library" >/dev/null; done
"$P" show "$J" > "$T/show-all.txt"
says "$T/show-all.txt" "$WEST"
WEST_LINES="$(grep -F "$WEST" "$T/show-all.txt" || true)"
SOUTH_LINES="$(grep -F "$SOUTH" "$T/show-all.txt" || true)"
if grep -qF "the frame of the gable (its studs cut to the rake) is not counted" <<<"$WEST_LINES"; then echo "    $WEST: its studs say the frame of the gable is not counted"; else echo "FAIL: the west wall's studs do not say the frame of the gable is not counted"; grep -F "studs" "$T/show-all.txt" | head; exit 1; fi
if grep -qF "frame of the gable" <<<"$SOUTH_LINES"; then echo "FAIL: the south wall says it has a gable"; exit 1; fi
echo "    (the south wall, under an eave, says nothing of a gable)"

echo
echo "== 2. Priced: the starter outside wall, lap siding at a made-up \$4.00 a square foot: the siding is bought over the gable =="
for w in $SOUTH $EAST $NORTH $WEST; do "$P" set "$J" "$w" unit_cost '$4.00/SF' --line "Lap siding" --status Known --source "made-up price" >/dev/null; done
"$P" export estimate "$J" "$T/est1.csv" >/dev/null
echo "  by hand: the end walls 144 + 40 = 184 SF each, the others 24 x 9 = 216 SF each, no opening: 2 x 184 + 2 x 216 = 800.00 SF, \$3200.00 (it was 720.00 SF, \$2880.00):"
sum "$T/est1.csv" "Lap siding" "800.00" "3200.00"
echo "  the weather barrier is bought over it too (800.00 SF), and the sheathing (on face_area + gable_area, the same 144 + 40 and 216) in whole sheets of 32 SF:"
echo "  2 x 184 / 32 = 5.75, so 6 sheets each, and 2 x 216 / 32 = 6.75, so 7 each: 2 x 6 + 2 x 7 = 26 sheets (it was 2 x 5 + 2 x 7 = 24):"
sum "$T/est1.csv" "Weather barrier" "800.00" none
sum "$T/est1.csv" "Wall sheathing" "26.00" none
echo "  the hanging of the siding is hours, with no production rate yet, so its quantity is read on the west wall itself (144 + 40 = 184 SF):"
"$P" show "$J" "$WEST" > "$T/show-west2.txt"
HANG="$(grep -E "quantity: Hang the siding +184 SF" "$T/show-west2.txt" || true)"
[ -n "$HANG" ] || { echo "FAIL: the west wall's hanging of the siding is not 184 SF"; grep "Hang the siding" "$T/show-west2.txt"; exit 1; }
printf '    said: %s\n' "$(sed 's/^ *//' <<<"$HANG" | head -1)"
echo
echo "== 3. A shed roof (one plane, its eave on the south, rising north): the sides and the high edge rise =="
J3="$T/shed.json"
house "$J3" "P G G G"
"$P" gable "$J3" > "$T/g3.txt"
echo "  at the south end 9.5 ft, at the north end 9 + 17 x 0.5 = 17.5 ft over the walls' lines. East wall: 16 x (0.5 + 8.5) / 2 = 72.00 SF; west the same:"
hand "$T/g3.txt" "$EAST" "72 SF, 9.5 to 17.5" "[$EAST]: gable ~72 SF; top (0, 9.5) (16, 17.5) ft"
hand "$T/g3.txt" "$WEST (runs north to south)" "72 SF, 17.5 to 9.5" "[$WEST]: gable ~72 SF; top (0, 17.5) (16, 9.5) ft"
echo "  the north wall, under the high edge, rises 8.5 ft along all 24 ft: 204.00 SF, its top level at 17.5 ft; the south, under the eave: none:"
hand "$T/g3.txt" "$NORTH" "204 SF, level 17.5" "[$NORTH]: gable ~204 SF; top (0, 17.5) (24, 17.5) ft"
says "$T/g3.txt" "[$SOUTH]: ~ none: an eave is over it"

echo
echo "== 4. A hip roof: eaves all round, no wall has a gable, and the siding is what it was =="
J4="$T/hip.json"
house "$J4" "P P P P"
"$P" gable "$J4" > "$T/g4.txt"
for w in $SOUTH $EAST $NORTH $WEST; do says "$T/g4.txt" "[$w]: ~ none: an eave is over it"; done
"$P" import assemblies "$J4" starter >/dev/null
for w in $SOUTH $EAST $NORTH $WEST; do
    "$P" apply "$J4" "$w" "Exterior wall, 2x6, lap siding" --source "the starter library" >/dev/null
    "$P" set "$J4" "$w" unit_cost '$4.00/SF' --line "Lap siding" --status Known --source "made-up price" >/dev/null
done
"$P" export estimate "$J4" "$T/est4.csv" >/dev/null
echo "  the walls' net areas: 2 x 216 + 2 x 144 = 720.00 SF, \$2880.00:"
sum "$T/est4.csv" "Lap siding" "720.00" "2880.00"

echo
echo "== 5. No roof drawn: none, and it says there is no roof over it =="
J5="$T/noroof.json"
house "$J5" none
"$P" gable "$J5" > "$T/g5.txt"
for w in $SOUTH $EAST $NORTH $WEST; do says "$T/g5.txt" "[$w]: none: no roof is drawn over it"; done

echo
echo "== 6. A corner board is as tall as its wall is at that corner (call 204): the same three houses, a corner-board line on every wall, priced at a made-up \$3.50 a foot =="
# boards <file>: a corner-board line on each of the four walls of the house just made, priced
boards() {
    local J="$1" w
    for w in $SOUTH $EAST $NORTH $WEST; do
        "$P" line "$J" "$w" "Corner boards" corner_board_length --division 6 >/dev/null
        "$P" set "$J" "$w" unit_cost '$3.50/LF' --line "Corner boards" "${A[@]}" >/dev/null
    done
}
# board_of <trim said> <wall> <by hand: LF>: the wall's own corner-board figure, as the trim verb says it
board_of() { hand "$1" "$2, corner boards" "$3 LF" "[$2]: outside corners ~2 EA; corner board ~$3 LF"; }
echo "  6.1 The gable roof. South and north under the eaves: 2 x 9 = 18.00 LF each. East and west under the rake, their top at each end 9.5 ft: 2 x 9.5 = 19.00 LF each (it was 18.00)."
echo "      The house: 18 + 19 + 18 + 19 = 74.00 LF, 74 x 3.50 = \$259.00 (72.00 LF and \$252.00 before):"
K1="$T/boards-gable.json"; house "$K1" "P G P G"; boards "$K1"
"$P" trim "$K1" > "$T/t1.txt"
board_of "$T/t1.txt" "$SOUTH" 18; board_of "$T/t1.txt" "$EAST" 19; board_of "$T/t1.txt" "$NORTH" 18; board_of "$T/t1.txt" "$WEST" 19
"$P" export estimate "$K1" "$T/est6a.csv" >/dev/null
sum "$T/est6a.csv" "Corner boards" "74.00" "259.00"
echo "  6.2 The shed roof. South 2 x 9 = 18; east 9.5 + 17.5 = 27; north 2 x 17.5 = 35; west 17.5 + 9.5 = 27: 107.00 LF, \$374.50 (72.00 LF before):"
K2="$T/boards-shed.json"; house "$K2" "P G G G"; boards "$K2"
"$P" trim "$K2" > "$T/t3.txt"
board_of "$T/t3.txt" "$SOUTH" 18; board_of "$T/t3.txt" "$EAST" 27; board_of "$T/t3.txt" "$NORTH" 35; board_of "$T/t3.txt" "$WEST" 27
"$P" export estimate "$K2" "$T/est6b.csv" >/dev/null
sum "$T/est6b.csv" "Corner boards" "107.00" "374.50"
echo "  6.3 The hip roof, eaves all round: every wall 18.00 LF, the house 72.00 LF; nothing moves:"
K3="$T/boards-hip.json"; house "$K3" "P P P P"; boards "$K3"
"$P" trim "$K3" > "$T/t4.txt"
for w in $SOUTH $EAST $NORTH $WEST; do board_of "$T/t4.txt" "$w" 18; done
"$P" export estimate "$K3" "$T/est6c.csv" >/dev/null
sum "$T/est6c.csv" "Corner boards" "72.00" "252.00"
echo "  6.4 The pieces laid are those lengths (trim --pieces): on the shed, the east wall's board at its north corner is 17.5 ft, at its south corner 9.5 ft, each from the foot:"
"$P" trim "$K2" --pieces > "$T/p3.txt"
python3 - "$T/p3.txt" "$EAST" <<'PY'
import re, sys
text, east = open(sys.argv[1]).read(), sys.argv[2]
pat = re.compile(r'^  .* \[(wall-\d+)\], priced by "Corner boards": [~?]?([\d.]+) LF from \(([^)]*)\) to \(([^)]*)\);')
rows = []
for line in text.splitlines():
    m = pat.match(line)
    if m: rows.append((m.group(1), float(m.group(2)), [float(x) for x in m.group(3).split(",")], [float(x) for x in m.group(4).split(",")]))
total = sum(r[1] for r in rows)
print("    the shed's boards: by hand 8 pieces, 107.00 LF; said %d pieces, %.2f LF" % (len(rows), total))
mine = sorted((r for r in rows if r[0] == east), key=lambda r: r[2][1])
print("    the east wall's two boards, south then north: by hand 9.5 and 17.5 ft; said %s" % [(r[1], r[3][2]) for r in mine])
ok = len(rows) == 8 and abs(total - 107.0) < 0.01 and len(mine) == 2 and abs(mine[0][1] - 9.5) < 0.01 and abs(mine[1][1] - 17.5) < 0.01 and abs(mine[1][3][2] - 17.5) < 0.01 and mine[1][2][2] == 0.0
if not ok:
    print("FAIL: the pieces are not the figures' lengths"); print(text); sys.exit(1)
PY
says "$T/p3.txt" "corner boards: 8 pieces come to 107.00 LF; the lines price 107.00 LF"
echo "  6.5 The boards read ~ where the roof is traced (Assumed), as the gable's area does: every line above says \"corner board ~\" and every piece is flagged ~:"
grep -E '^  wall .*priced by "Corner boards"' "$T/p3.txt" | grep -vqE 'priced by "Corner boards": ~' && { echo "FAIL: a board is not marked ~"; exit 1; }
echo "      (all eight pieces of the shed read ~; the Known case, a Known wall under a Known roof, is held by crates/world/tests/call204.rs)"
echo "  6.6 The sample whole house, a hip roof: its corner boards are the 128 LF they were and its price the \$468,050.22 it was:"
"$P" sixteen "$T/sixteen.json" >/dev/null
"$P" export estimate "$T/sixteen.json" "$T/est6s.csv" >/dev/null
"$P" estimate "$T/sixteen.json" > "$T/est6s.txt"
python3 - "$T/est6s.csv" <<'PY'
import csv, sys
rows = [r for r in csv.DictReader(open(sys.argv[1])) if r["Line"].startswith("Corner boards, 5/4 x 6 cedar")]
q = sum(float(r["Quantity"]) for r in rows)
print("    by hand 128.00 LF; said %.2f LF on %d walls" % (q, len(rows)))
if abs(q - 128.0) > 0.005:
    print("FAIL: the sample's corner boards moved"); sys.exit(1)
PY
hand "$T/est6s.txt" "the sample's price" '$468,050.22' '$468,050.22'
echo "  6.7 The shed's east wall over a stone wainscot (the starter wall with its stone line, wainscot 3 ft): the boards start at the top of the stone, each at its own corner's height: (9.5 - 3) + (17.5 - 3) = 21.00 LF:"
K4="$T/boards-shed-stone.json"; house "$K4" "P G G G"
"$P" import assemblies "$K4" starter >/dev/null
"$P" apply "$K4" "$EAST" "Exterior wall, 2x6, lap siding over a stone wainscot" --source "the starter library" >/dev/null
"$P" show "$K4" "$EAST" > "$T/show-stone.txt"
hand "$T/show-stone.txt" "$EAST, its boards above the stone" "21 LF" "corner board length above wainscot"

echo
echo "== all of it as the hand figures say =="
