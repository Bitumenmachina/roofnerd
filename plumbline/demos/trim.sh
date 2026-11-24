#!/usr/bin/env bash
# The outside trim priced (call 185). From an empty job: a box with a gable roof drawn a foot outside its walls, a window
# and a door in the south wall. Each eave's and rake's length and how far it overhangs the wall under it, each wall's
# outside corners and the corner board they take, each opening's casing: said by the program and set beside the figure
# worked by hand. The starter lines (fascia, soffit, gutter, downspouts, rake fascia, rake soffit, corner boards, exterior
# casing) brought in and given to the eaves, rakes, walls, window and door, none with a price and "Downspouts" with no
# quantity until a person types how many; prices typed and each line's dollars set beside the hand figure; an L-shaped house
# (five outside corners and one inside); a roof drawn alone (no wall under its edge: the overhang is not known, and says why);
# and the sample houses' estimates the same, byte for byte, as before this call.
# Run from the repository root:  demos/trim.sh
# The job and every price are made up (no real job data) and live in a temporary folder.
# Every step fails the script if the program does not say what the step expects. The figures are worked by hand in
# STATUS.md ("The outside trim priced: a demonstration, written before it is built"); each is printed beside what the program said.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
A=(--status Assumed --source "made-up tracing")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
# A hand figure beside the line the program said it in: hand "<file>" "<what>" "<by hand>" "<text the line holds>"
hand() {
    local line
    line="$(grep -F -- "$4" "$1" | head -1 | sed 's/^ *//')" || true
    [ -n "$line" ] || { echo "FAIL: nothing says: $4 ($2)"; cat "$1"; exit 1; }
    printf '    %-30s by hand %-12s said: %s\n' "$2" "$3" "$line"
}
# The same, where the line's columns are spaced by the width of the sheet: the text is a pattern.
handre() {
    local line
    line="$(grep -E -- "$4" "$1" | head -1 | sed 's/^ *//')" || true
    [ -n "$line" ] || { echo "FAIL: nothing says: $4 ($2)"; cat "$1"; exit 1; }
    printf '    %-30s by hand %-12s said: %s\n' "$2" "$3" "$line"
}
# The estimate as a table (a line's quantity and what it comes to, to the cent), and a sum of a line over every thing it is on:
# sum "<line>" "<by hand: quantity>" "<by hand: dollars, or none>"
estimate() { "$P" export estimate "$J" "$T/est.csv" >/dev/null; "$P" estimate "$J" > "$T/est.txt"; }
sum() {
    python3 - "$T/est.csv" "$1" "$2" "$3" <<'PY'
import csv, sys
path, line, want_q, want_d = sys.argv[1:5]
rows = [r for r in csv.DictReader(open(path)) if r["Line"] == line]
assert rows, "no line %r in the estimate" % line
q = sum(float(r["Quantity"]) for r in rows if r["Quantity"] != "")
d = sum(float(r["Amount"]) for r in rows if r["Amount"] != "")
priced = [r for r in rows if r["Amount"] != ""]
got_q = "%.2f" % q
got_d = "%.2f" % d if priced else "none"
print("    %-20s %d on %-30s by hand %8s LF/SF  %-10s said %8s  %s" % (line, len(rows), ", ".join(r["Element"] for r in rows)[:30], want_q, want_d, got_q, got_d))
if got_q != want_q or got_d != want_d:
    print("FAIL: %s: by hand %s and %s, the program said %s and %s" % (line, want_q, want_d, got_q, got_d)); sys.exit(1)
PY
}
trimmed() { "$P" trim "$J" > "$T/trim.txt"; }

echo "== The house: one level 9 ft high, a ring of walls on the lines of a 24 ft by 16 ft box, 6 in thick; a gable roof a foot outside the walls all round (26 ft by 18 ft), its ridge east-west, 6 in 12; a window 3 ft by 5 ft and a door 3 ft by 6 ft 8 in in the south wall =="
"$P" new "$J" "Trim (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${A[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${A[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${A[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "6 ft 8 in" "${A[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${A[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
WIN="$(made "$("$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "2.5 ft" "${A[@]}")")"
DOOR="$(made "$("$P" door "$J" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${A[@]}")")"
ROOF="$(made "$("$P" roof "$J" "Main roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 6:12 --eave-height "9 ft" "${A[@]}")")"
"$P" set "$J" "$ROOF" edge:2 gable "${A[@]}" >/dev/null
"$P" set "$J" "$ROOF" edge:4 gable "${A[@]}" >/dev/null
EAVES="$(grep -oE 'eave-[0-9]+' <<<"$("$P" show "$J")" | sort -u | tr '\n' ' ')"
RAKES="$(grep -oE 'rake-[0-9]+' <<<"$("$P" show "$J")" | sort -u | tr '\n' ' ')"
read -r EAVE_S EAVE_N <<<"$EAVES"
[ "$(wc -w <<<"$EAVES")" = 2 ] && [ "$(wc -w <<<"$RAKES")" = 4 ] || { echo "FAIL: expected 2 eaves and 4 rakes: $EAVES / $RAKES"; exit 1; }
cp "$J" "$T/job-before-lines.json"

echo
echo "== 1. As drawn: the figures, each worked from what is drawn and Assumed as the tracing is =="
trimmed
echo "  the eaves, 26 LF each, and the rakes, each the slope of a 9 ft run at 6 in 12 (9 x 1.118034 = 10.0623 LF):"
for e in $EAVES; do hand "$T/trim.txt" "eave $e, length and overhang" "26 LF, 9 in" "[$e]: ~26 LF long; overhang ~0.75 ft"; done
for r in $RAKES; do hand "$T/trim.txt" "rake $r, length and overhang" "10.0623 LF, 9 in" "[$r]: ~10.0623 LF long; overhang ~0.75 ft"; done
echo "  each overhang rests on a wall, and says which (the roof's edge is 1 ft out; the wall's outside face is 3 in out: 12 - 3 = 9 in):"
says "$T/trim.txt" "[$EAVE_S]: ~26 LF long; overhang ~0.75 ft, the outside face of $SOUTH"
says "$T/trim.txt" "[$EAVE_N]: ~26 LF long; overhang ~0.75 ft, the outside face of $NORTH"
echo "  the walls: two outside corners each, a board up each, 9 ft (18 LF a wall):"
for w in $SOUTH $EAST $NORTH $WEST; do hand "$T/trim.txt" "$w, corners and board" "2, 18 LF" "[$w]: outside corners ~2 EA; corner board ~18 LF"; done
echo "  the window, 2 x (3 + 5) = 16 LF; the door, 3 + 2 x 6.6667 = 16.33 LF:"
hand "$T/trim.txt" "$WIN, casing" "16 LF" "[$WIN]: casing ~16 LF"
hand "$T/trim.txt" "$DOOR, casing" "16.33 LF" "[$DOOR]: casing ~16.3333 LF"
echo "  a thing no line asks the figure of does not carry it (the job's file is as it was):"
"$P" show "$J" "$SOUTH" > "$T/show-south.txt"
lacks "$T/show-south.txt" "outside corners"
lacks "$T/show-south.txt" "corner board length"
"$P" show "$J" "$EAVE_S" > "$T/show-eave.txt"
lacks "$T/show-eave.txt" "overhang"

echo
echo "== 2. The starter lines come in and are given to the eaves, the rakes, the walls, the window and the door: no price on any =="
"$P" import assemblies "$J" starter > "$T/imp.txt"
says "$T/imp.txt" "set: 41"
says "$T/imp.txt" "not used: 0"
L="the starter library"
for e in $EAVES; do "$P" apply "$J" "$e" "Gutter" --source "$L" > /dev/null; done
for r in $RAKES; do "$P" apply "$J" "$r" "Rake trim" --source "$L" > /dev/null; done
for w in $SOUTH $EAST $NORTH $WEST; do "$P" apply "$J" "$w" "Exterior wall, 2x6, lap siding" --source "$L" > /dev/null; done
"$P" apply "$J" "$WIN" "Window" --source "$L" > /dev/null
"$P" apply "$J" "$DOOR" "Entry door" --source "$L" > /dev/null
estimate
echo "  what each line's quantity comes to, summed over the things it is on (by hand: fascia 2 x 26 = 52; rake fascia 4 x 10.0623 = 40.25;"
echo "  soffit 2 x 26 x 0.75 = 39; rake soffit 40.2492 x 0.75 = 30.19; gutter 52; corner boards 4 x 18 = 72; casing 16 + 16.33 = 32.33); no price, so no dollars:"
sum "Fascia" "52.00" none
sum "Rake fascia" "40.25" none
sum "Soffit" "39.00" none
sum "Rake soffit" "30.19" none
sum "Gutter" "52.00" none
sum "Corner boards" "72.00" none
sum "Exterior casing" "32.33" none
echo "  \"Downspouts\" has no quantity until a person says how many (a figure that is not known is no figure, never zero):"
handre "$T/est.txt" "Downspouts, as the sheet has it" "no figure" "^  Downspouts +no figure +no price"
"$P" show "$J" "$EAVE_S" > "$T/show-eave.txt"
says "$T/show-eave.txt" "? downspouts: Downspouts unknown"
"$P" vif "$J" > "$T/vif.txt"
says "$T/vif.txt" "\"Downspouts\" (no figure yet)"
"$P" set "$J" "$EAVE_S" downspouts "4 EA" --line "Downspouts" --status Assumed --source "made-up: one at each end of the run" >/dev/null
estimate
handre "$T/est.txt" "Downspouts, 4 typed, Assumed" "~4 EA" "^  Downspouts +~4.00 EA"
echo "  the figures a line names are held on the thing now (one a line names, and no other):"
"$P" show "$J" > "$T/show-all.txt"
lacks "$T/show-all.txt" "did not agree with what is drawn"
"$P" show "$J" "$SOUTH" > "$T/show-south.txt"
handre "$T/show-south.txt" "$SOUTH, its figures now" "2 EA, 18 LF" "= ~ outside corners +2 EA"
handre "$T/show-south.txt" "$SOUTH, its figures now" "2 EA, 18 LF" "= ~ corner board length +18 LF"

echo
echo "== 3. Prices typed (made up: fascia \$4.00, soffit \$6.00, corner boards \$3.50, casing \$3.00): each line's dollars, to the cent =="
for e in $EAVES; do
    "$P" set "$J" "$e" unit_cost '$4/LF' --line "Fascia" "${A[@]}" >/dev/null
    "$P" set "$J" "$e" unit_cost '$6/SF' --line "Soffit" "${A[@]}" >/dev/null
done
for r in $RAKES; do
    "$P" set "$J" "$r" unit_cost '$4/LF' --line "Rake fascia" "${A[@]}" >/dev/null
    "$P" set "$J" "$r" unit_cost '$6/SF' --line "Rake soffit" "${A[@]}" >/dev/null
done
for w in $SOUTH $EAST $NORTH $WEST; do "$P" set "$J" "$w" unit_cost '$3.50/LF' --line "Corner boards" "${A[@]}" >/dev/null; done
for o in $WIN $DOOR; do "$P" set "$J" "$o" unit_cost '$3/LF' --line "Exterior casing" "${A[@]}" >/dev/null; done
estimate
echo "  (by hand: fascia 52 x 4 = 208.00; rake fascia 4 x (10.0623 x 4 = 40.25) = 161.00; soffit 39 x 6 = 234.00; rake soffit 4 x (7.5467 x 6 = 45.28) = 181.12;"
echo "   corner boards 72 x 3.50 = 252.00; casing (16 + 16.3333) x 3 = 48.00 + 49.00 = 97.00)"
sum "Fascia" "52.00" "208.00"
sum "Rake fascia" "40.25" "161.00"
sum "Soffit" "39.00" "234.00"
sum "Rake soffit" "30.19" "181.12"
sum "Corner boards" "72.00" "252.00"
sum "Exterior casing" "32.33" "97.00"

echo
echo "== 4. An L-shaped house: the 24 ft by 16 ft box with a 12 ft by 10 ft wing off its north side: five outside corners and one inside =="
L_="$T/l.json"
"$P" new "$L_" "L (not a real job)" >/dev/null
LS="$(made "$("$P" level "$L_" ground --elevation "0 ft" --height "9 ft" "${A[@]}")")"
LT="$(made "$("$P" type "$L_" wall "wall 6 in" --thickness "6 in" "${A[@]}")")"
read -r L1 L2 L3 L4 L5 L6 <<<"$("$P" wall "$L_" "$LS" "$LT" --points "0,0 24,0 24,16 12,16 12,26 0,26" --closed --height "9 ft" "${A[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
for w in $L1 $L2 $L3 $L4 $L5 $L6; do "$P" line "$L_" "$w" "Corner boards" corner_board_length --division 6 >/dev/null; done
"$P" trim "$L_" > "$T/ltrim.txt"
hand "$T/ltrim.txt" "$L1 (0,0 to 24,0)" "2 corners" "[$L1]: outside corners ~2 EA; corner board ~18 LF"
hand "$T/ltrim.txt" "$L2 (24,0 to 24,16)" "2 corners" "[$L2]: outside corners ~2 EA; corner board ~18 LF"
echo "  the two walls that meet at the inside corner (12,16) have one outside corner each, 9 LF of board:"
hand "$T/ltrim.txt" "$L3 (24,16 to 12,16)" "1 corner" "[$L3]: outside corners ~1 EA; corner board ~9 LF"
hand "$T/ltrim.txt" "$L4 (12,16 to 12,26)" "1 corner" "[$L4]: outside corners ~1 EA; corner board ~9 LF"
hand "$T/ltrim.txt" "$L5 (12,26 to 0,26)" "2 corners" "[$L5]: outside corners ~2 EA; corner board ~18 LF"
hand "$T/ltrim.txt" "$L6 (0,26 to 0,0)" "2 corners" "[$L6]: outside corners ~2 EA; corner board ~18 LF"
"$P" estimate "$L_" > "$T/lest.txt"; "$P" export estimate "$L_" "$T/lest.csv" >/dev/null
python3 - "$T/lest.csv" <<'PY'
import csv, sys
rows = [r for r in csv.DictReader(open(sys.argv[1])) if r["Line"] == "Corner boards"]
total = sum(float(r["Quantity"]) for r in rows)
print("    corner boards, six walls: by hand 10 x 9 = 90.00 LF, said %.2f LF" % total)
assert len(rows) == 6 and "%.2f" % total == "90.00", rows
PY

echo
echo "== 5. A roof drawn alone: no wall under any edge: the overhang is not known, and says why; the soffit has no figure =="
R_="$T/roof.json"
"$P" new "$R_" "Roof alone (not a real job)" >/dev/null
RR="$(made "$("$P" roof "$R_" "Main roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 6:12 --eave-height "9 ft" "${A[@]}")")"
"$P" set "$R_" "$RR" edge:2 gable "${A[@]}" >/dev/null
"$P" set "$R_" "$RR" edge:4 gable "${A[@]}" >/dev/null
RE="$(grep -oE 'eave-[0-9]+' <<<"$("$P" show "$R_")" | sort -u | head -1)"
"$P" line "$R_" "$RE" "Soffit" "true_length * overhang" --division 6 >/dev/null
"$P" trim "$R_" > "$T/rtrim.txt"
says "$T/rtrim.txt" "[$RE]: ~26 LF long; overhang ? no figure (no wall is found under this edge: none lies along it within 4 ft with no other wall between)"
"$P" estimate "$R_" > "$T/rest.txt"
handre "$T/rest.txt" "its soffit" "no figure" "^  Soffit +no figure +no price"

echo
echo "== 6. Where each piece lies (call 184, step 1): \`trim --pieces\` on the house of step 1, its lines given, made-up prices on =="
"$P" trim "$J" > "$T/trim-plain.txt"
cmp -s "$T/trim.txt" "$T/trim-plain.txt" && echo "    (without --pieces, \`trim\` says what it said at step 1, byte for byte: the lines changed nothing)"
lacks "$T/trim-plain.txt" "piece"
"$P" trim "$J" --pieces > "$T/pieces.txt"
python3 - "$T/pieces.txt" "$EAVE_S" "$EAVE_N" "$SOUTH" "$WIN" "$DOOR" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
eave_s, eave_n, south, win, door = sys.argv[2:7]
pat = re.compile(r'^  (.+) \[([a-z]+-\d+)\], priced by "(.*)": ([~?]?)([\d.]+) LF from \(([^)]*)\) to \(([^)]*)\); looks ([^;]*); (.*) wide, (.*) thick \((.*)\)$')
kind, rows = None, []
for line in text.splitlines():
    if re.match(r'^[A-Z][a-z ]+:$', line):
        kind = line[:-1]
    m = pat.match(line)
    if m and kind:
        name, path, priced, flag, length, a, b, looks, width, thick, said = m.groups()
        rows.append(dict(kind=kind, path=path, line=priced, flag=flag, length=float(length), a=[float(x) for x in a.split(",")], b=[float(x) for x in b.split(",")], looks=looks, width=width, thick=thick, said=said))
def of(k): return [r for r in rows if r["kind"] == k]
def check(what, by_hand, got, ok):
    print("    %-44s by hand %-22s said %s" % (what, by_hand, got))
    if not ok: print("FAIL: %s" % what); print(text); sys.exit(1)
def close(x, y, e=0.0006): return abs(x - y) < e
want = {"Fascia": (2, 52.0), "Rake fascia": (4, 40.2492), "Soffit": (2, 26.0 * 2), "Rake soffit": (4, 40.2492), "Corner boards": (8, 72.0), "Casing": (7, 32.3333)}
for k, (n, total) in want.items():
    got = of(k)
    check("%s: pieces, and their length" % k, "%d, %.2f LF" % (n, total), "%d, %.2f LF" % (len(got), sum(r["length"] for r in got)), len(got) == n and close(sum(r["length"] for r in got), total, 0.001))
check("2+4+2+4+8+4+3: all pieces", "27", str(len(rows)), len(rows) == 27)
z = 9 - 5.5 / 12 / 2
f = {r["path"]: r for r in of("Fascia")}
check("south fascia, from and to (east, north, up)", "(-1,-1,%.4f)-(25,-1,%.4f)" % (z, z), "%s-%s" % (f[eave_s]["a"], f[eave_s]["b"]), f[eave_s]["a"] == [-1.0, -1.0, round(z, 4)] and f[eave_s]["b"] == [25.0, -1.0, round(z, 4)] and f[eave_s]["looks"] == "south")
check("north fascia, from and to", "(-1,17,%.4f)-(25,17,%.4f)" % (z, z), "%s-%s" % (f[eave_n]["a"], f[eave_n]["b"]), f[eave_n]["a"] == [-1.0, 17.0, round(z, 4)] and f[eave_n]["b"] == [25.0, 17.0, round(z, 4)] and f[eave_n]["looks"] == "north")
check("each rake fascia climbs 4.5 ft in 9 ft on the plan", "10.0623 LF, up 4.5", "%s" % sorted(set((r["length"], round(r["b"][2] - r["a"][2], 4)) for r in of("Rake fascia"))), all(close(r["length"], 10.0623) and close(r["b"][2] - r["a"][2], 4.5) and close(abs(r["b"][1] - r["a"][1]), 9.0) for r in of("Rake fascia")))
check("each soffit is as wide as the overhang, looking down", "9 in", "%s" % sorted(set(r["width"] for r in of("Soffit"))), all(r["width"] == "9 in" and r["looks"] == "down" for r in of("Soffit")))
check("each rake soffit: 9 in wide, 10.0623 LF", "9 in, 10.0623", "%s" % sorted(set((r["width"], r["length"]) for r in of("Rake soffit"))), all(r["width"] == "9 in" and close(r["length"], 10.0623) for r in of("Rake soffit")))
cb = of("Corner boards")
check("each corner board stands 9 ft from the foot", "9 LF", "%s" % sorted(set(r["length"] for r in cb)), all(close(r["length"], 9.0) and r["a"][2] == 0.0 and r["b"][2] == 9.0 for r in cb))
sw = [r for r in cb if r["path"] == south and close(r["a"][0], -0.25 + 3.5 / 24)]
check("south wall's board at the south-west corner", "(-0.1042,-0.25), looks south", "%s %s" % (sw[0]["a"][:2], sw[0]["looks"]) if sw else "none", len(sw) == 1 and sw[0]["a"][1] == -0.25 and sw[0]["looks"] == "south")
cs = of("Casing")
win_p, door_p = [r for r in cs if r["path"] == win], [r for r in cs if r["path"] == door]
check("window casing: head 3, sill 3, two sides 5", "4 pieces, 16 LF", "%d pieces, %.2f LF" % (len(win_p), sum(r["length"] for r in win_p)), len(win_p) == 4 and close(sum(r["length"] for r in win_p), 16.0, 0.001))
check("door casing: head 3, two sides 6.6667, none at its foot", "3 pieces, 16.33 LF", "%d pieces, %.2f LF" % (len(door_p), sum(r["length"] for r in door_p)), len(door_p) == 3 and close(sum(r["length"] for r in door_p), 16.3333, 0.001) and all(min(r["a"][2], r["b"][2]) >= 0 for r in door_p) and sum(1 for r in door_p if r["a"][2] == r["b"][2]) == 1)
check("casing lies on the south wall's outside face", "north -0.25, looks south", "%s" % sorted(set((r["a"][1], r["looks"]) for r in cs)), all(r["a"][1] == -0.25 and r["b"][1] == -0.25 and r["looks"] == "south" for r in cs))
check("every piece is Assumed (the tracing is)", "~ on all 27", "%d with ~" % sum(1 for r in rows if r["flag"] == "~"), all(r["flag"] == "~" for r in rows))
check("a size no line names is said to be drawn only", "'the line names no size'", "%d say so" % sum(1 for r in rows if "the line names no size" in r["said"]), all("the line names no size" in r["said"] for r in rows))
PY
echo "  each kind's pieces beside what its lines price, as the verb ends:"
# Call 191 point 13 (ruled 2026-11-23): a quantity is printed as the estimate prints it, to 2 places. These eight lines read "52 LF", "40.2492 LF", "39 SF",
# "30.1869 SF", "72 LF", "32.3333 LF", "26 LF" and "90 LF" before; the figures are the same, now written 52.00, 40.25, 39.00, 30.19, 72.00, 32.33, 26.00, 90.00.
says "$T/pieces.txt" "  fascia: 2 pieces come to 52.00 LF; the lines price 52.00 LF"
says "$T/pieces.txt" "  rake fascia: 4 pieces come to 40.25 LF; the lines price 40.25 LF"
says "$T/pieces.txt" "  soffit: 2 pieces come to 39.00 SF; the lines price 39.00 SF"
says "$T/pieces.txt" "  rake soffit: 4 pieces come to 30.19 SF; the lines price 30.19 SF"
says "$T/pieces.txt" "  corner boards: 8 pieces come to 72.00 LF; the lines price 72.00 LF"
says "$T/pieces.txt" "  casing: 7 pieces come to 32.33 LF; the lines price 32.33 LF"
echo "  an eave with no fascia line has no fascia: the job before its lines were given has no piece at all, and says so:"
"$P" trim "$T/job-before-lines.json" --pieces > "$T/pieces0.txt"
says "$T/pieces0.txt" "No cost line prices any outside trim here, so no piece is drawn."
echo "  only the south eave given a fascia line: one piece, on it:"
cp "$T/job-before-lines.json" "$T/one.json"
"$P" line "$T/one.json" "$EAVE_S" "Fascia" true_length --division 6 >/dev/null
"$P" trim "$T/one.json" --pieces > "$T/pieces1.txt"
says "$T/pieces1.txt" "  fascia: 1 piece comes to 26.00 LF; the lines price 26.00 LF"
lacks "$T/pieces1.txt" "[$EAVE_N]"
echo "  the L-shaped house of step 4: ten corner boards, none at the inside corner (12,16):"
"$P" trim "$L_" --pieces > "$T/lpieces.txt"
says "$T/lpieces.txt" "  corner boards: 10 pieces come to 90.00 LF; the lines price 90.00 LF"
python3 - "$T/lpieces.txt" <<'PY'
import re, sys
pts = [tuple(float(x) for x in m.group(1).split(",")[:2]) for m in re.finditer(r"LF from \(([^)]*)\) to", open(sys.argv[1]).read())]
assert len(pts) == 10, pts
near = [p for p in pts if ((p[0] - 12) ** 2 + (p[1] - 16) ** 2) ** 0.5 < 0.6]
print("    boards within 0.6 ft of the inside corner (12,16): by hand 0, said %d" % len(near))
assert not near, near
PY
echo "  the roof alone, a soffit line and no wall under the edge: no soffit piece, and the list says why:"
"$P" line "$R_" "$RE" "Fascia" true_length --division 6 >/dev/null
"$P" trim "$R_" --pieces > "$T/rpieces.txt"
says "$T/rpieces.txt" "Left out:"
says "$T/rpieces.txt" "no wall is found under this edge"
lacks "$T/rpieces.txt" "Soffit:"
says "$T/rpieces.txt" "  fascia: 1 piece comes to 26.00 LF; the lines price 26.00 LF"

echo
echo "== 7. The sample houses: nothing moves. Each job file and each estimate is the same, byte for byte, as before this call =="
for v in sample house sixteen priced; do
    "$P" "$v" "$T/$v.json" >/dev/null
    "$P" estimate "$T/$v.json" > "$T/$v.estimate.txt" 2>&1
done
( cd "$T" && sha256sum -c --quiet "$OLDPWD/demos/trim-before.sha256" ) && echo "    (the four sample houses' files and estimates: every checksum as it was before call 185)"

echo
echo "== 8. What the judge of call 191 found: every priced line is drawn or said, no figure is not nothing, the corners of the soffit are said =="
echo "  C10. The north eave's fascia priced on its plan length (26 LF) and not its true length: by hand the two eaves price 26 + 26 = 52 LF, one piece (the south eave's, 26 LF) is drawn, and 26 LF is priced and not drawn:"
cp "$J" "$T/c10.json"
"$P" set "$T/c10.json" "$EAVE_N" quantity plan_length --line "Fascia" "${A[@]}" >/dev/null
"$P" trim "$T/c10.json" --pieces > "$T/c10.txt"
says "$T/c10.txt" "  fascia: 1 piece comes to 26.00 LF; the lines price 52.00 LF (they differ: see what is left out): 26.00 LF of what the lines price is not drawn"
says "$T/c10.txt" "Left out:"
says "$T/c10.txt" "[$EAVE_N]: \"Fascia\" prices 26.00 LF by \"plan_length\", which is not its true length: no piece of its fascia is drawn from it, and 26.00 LF of what it prices is not drawn"
echo "  C11. The roof drawn alone, a soffit line on its eave and no wall under it: the overhang is not known, so the soffit's total is no figure, never 0 SF:"
"$P" trim "$R_" --pieces > "$T/c11.txt"
says "$T/c11.txt" "  soffit: 0 pieces come to 0.00 SF; the lines price no figure: 1 of its lines has no figure (see what is left out)"
lacks "$T/c11.txt" "the lines price 0 SF"
lacks "$T/c11.txt" "the lines price 0.00 SF"
echo "  C12. A hip roof 26 ft by 18 ft on plan over a box of 6 in walls (an overhang of 9 in), a soffit line on each eave: by hand 2 x (26 + 18) x 0.75 = 66.00 SF is priced, as it always was, and each of the 4 corners is a 0.75 by 0.75 ft square counted in two runs: 4 x 0.5625 = 2.25 SF:"
H="$T/hip.json"
"$P" new "$H" "Hip (not a real job)" >/dev/null
HS="$(made "$("$P" level "$H" ground --elevation "0 ft" --height "9 ft" "${A[@]}")")"
HW="$(made "$("$P" type "$H" wall "wall 6 in" --thickness "6 in" "${A[@]}")")"
"$P" wall "$H" "$HS" "$HW" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${A[@]}" >/dev/null
"$P" roof "$H" "Hip roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 6:12 --eave-height "9 ft" "${A[@]}" >/dev/null
for e in $(grep -oE 'eave-[0-9]+' <<<"$("$P" show "$H")" | sort -u); do "$P" line "$H" "$e" "Soffit" "true_length * overhang" --division 6 >/dev/null; done
"$P" trim "$H" --pieces > "$T/c12.txt"
says "$T/c12.txt" "  soffit: 4 pieces come to 66.00 SF; the lines price 66.00 SF"
says "$T/c12.txt" "2.25 SF is corner counted in two runs (4 corners)"
"$P" trim "$H" > "$T/c12t.txt"
says "$T/c12t.txt" "counts each corner where two runs meet in both: 4 corners, 2.25 SF in all"
says "$T/c12t.txt" "0.5625 SF is in both runs"
echo "  C13. A quantity is said as the estimate says it, to 2 places: the rake fascia of the gable house, by hand 4 x 10.0623 = 40.2492, said 40.25 and not 40.2492:"
lacks "$T/pieces.txt" "40.2492 LF; the lines price"

echo
echo "PASS"
