#!/usr/bin/env bash
# The sample whole house prices its outside trim (call 194). `plumbline sixteen` makes the made-up whole house (40 ft by 28 ft, two
# levels, walls 10 ft and 9 ft, a hip roof whose four eaves are 42, 30, 42 and 30 LF with 0.75 ft of overhang, 17 windows, a 3 ft stone
# wainscot round the ground level). Its outside walls now price corner boards (from the top of the stone on the ground level, by the
# new figure corner_board_length_above_wainscot), each window its casing, each eave its fascia and soffit. The six points of
# STATUS.md ("The sample whole house prices its outside trim: a demonstration"), each said by the program and set beside the figure
# worked by hand.
# Run from the repository root:  demos/sixteen-trim.sh
# The house and every price are made up (no real job data) and live in a temporary folder.
# Every step fails the script if the program does not say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
# A line of the file read as a pattern (the columns are spaced by the width of the sheet): said "<file>" "<what>" "<by hand>" "<pattern>"
handre() {
    local line
    line="$(grep -E -- "$4" "$1" | head -1 | sed 's/^ *//')" || true
    [ -n "$line" ] || { echo "FAIL: nothing says: $4 ($2)"; cat "$1"; exit 1; }
    printf '    %-34s by hand %-14s said: %s\n' "$2" "$3" "$line"
}
# What a line of the estimate adds up to, over every thing it is on, from the estimate as a table:
# total "<line starts with>" "<how many>" "<by hand: quantity>" "<by hand: dollars>"
total() {
    python3 - "$T/est.csv" "$1" "$2" "$3" "$4" <<'PY'
import csv, sys
path, line, want_n, want_q, want_d = sys.argv[1:6]
rows = [r for r in csv.DictReader(open(path)) if r["Line"].startswith(line)]
n, q, d = len(rows), sum(float(r["Quantity"]) for r in rows), sum(float(r["Amount"]) for r in rows)
got = ("%d" % n, "%.2f" % q, "%.2f" % d)
print("    %-34s by hand %3s on it %9s  %10s   said %3s %9s  %10s" % (line, want_n, want_q, want_d, *got))
if got != (want_n, want_q, want_d):
    print("FAIL: %s" % line); sys.exit(1)
PY
}

echo "== The house: plumbline sixteen =="
"$P" sixteen "$J" >/dev/null
"$P" estimate "$J" > "$T/est.txt"
"$P" export estimate "$J" "$T/est.csv" >/dev/null
"$P" trim "$J" > "$T/trim.txt"
"$P" trim "$J" --pieces > "$T/pieces.txt"

echo
echo "== 1. The estimate: what was added, line by line, and what it does to the recap =="
echo "   by hand: corner boards 72 + 56 = 128 LF at \$6.25 = \$800.00; casing 135 + 154 = 289 LF at \$5.40 = \$1,560.60; fascia 144 LF at \$4.80 = \$691.20; soffit 144 x 0.75 = 108 SF at \$6.20 = \$669.60; 8 + 17 + 4 + 4 = 33 lines, \$3,721.40"
total "Corner boards, 5/4 x 6 cedar" 8 128.00 800.00
total "Exterior casing, 5/4 x 4 cedar" 17 289.00 1560.60
total "Fascia, 1x8 cedar" 4 144.00 691.20
total "Soffit, cedar plywood, vented" 4 108.00 669.60
echo "   the recap: Material was \$199,666.53 on 205 lines, job cost \$390,260.26 on 399, division 6 \$51,074.22 on 41; each moves by \$3,721.40 and 33 lines"
handre "$T/est.txt" "Material" '$203,387.93 238' '^ *Material +\$203,387\.93 +238 lines'
handre "$T/est.txt" "Job cost" '$393,981.66 432' '^ *Job cost +\$393,981\.66 +432 lines'
handre "$T/est.txt" "Division 6" '$54,795.62 74' '^ *6 Wood and Plastics +\$54,795\.62 +74 lines'
echo "   nothing else moves"
handre "$T/est.txt" "Supervision" 'unchanged' '^ *Supervision +\$40,800\.00 +1 line'
handre "$T/est.txt" "Subcontract" 'unchanged' '^ *Subcontract +\$142,543\.73 +190 lines'
handre "$T/est.txt" "General conditions" 'unchanged' '^ *General conditions +\$7,250\.00 +3 lines'
handre "$T/est.txt" "Division 7" 'unchanged' '^ *7 Thermal and Moisture Protection +\$51,488\.19 +54 lines'
handre "$T/est.txt" "Division 8" 'unchanged' '^ *8 Doors and Windows +\$28,735\.00 +35 lines'
handre "$T/est.txt" "Division 9" 'unchanged' '^ *9 Finishes +\$48,955\.12 +52 lines'
echo "   overhead 10 % of 393,981.66 = 39,398.166; profit 8 % of 433,379.83 = 34,670.3864; the price 433,379.83 + 34,670.39"
handre "$T/est.txt" "Overhead at 10 %" '$39,398.17' '^ *Overhead at 10\.00 % +\$39,398\.17'
handre "$T/est.txt" "Profit at 8 %" '$34,670.39' '^ *Profit at 8\.00 % +\$34,670\.39'
handre "$T/est.txt" "PRICE" '$468,050.22' '^ *PRICE +\$468,050\.22'

echo
echo "== 2. A ground-level outside wall's corner boards: 14 LF, two corners each 10 ft less the 3 ft entered as wainscot =="
GROUND="$(grep -E '^south wall  \[wall\]' "$T/est.txt" | head -1 | grep -oE 'wall-[0-9]+')"
UPPER="$(grep -E '^south wall  \[wall\]' "$T/est.txt" | sed -n 2p | grep -oE 'wall-[0-9]+')"
[ -n "$GROUND" ] && [ -n "$UPPER" ] && [ "$GROUND" != "$UPPER" ] || { echo "FAIL: two south walls are expected"; exit 1; }
"$P" show "$J" "$GROUND" > "$T/ground.txt"
handre "$T/ground.txt" "its figure" '14 LF' 'corner board length above wainscot +14 LF'
handre "$T/ground.txt" "its entry" '3 ft' 'wainscot: Corner boards, 5/4 x 6 cedar 3 ft'
handre "$T/ground.txt" "its quantity" '14 LF' 'quantity: Corner boards, 5/4 x 6 cedar 14 LF'
"$P" show "$J" "$UPPER" > "$T/upper.txt"
handre "$T/upper.txt" "an upper wall's quantity" '18 LF' 'quantity: Corner boards, 5/4 x 6 cedar 18 LF'
echo "   the entry taken off the line: the line is refused, not priced to the ground and not nothing"
cp "$J" "$T/before-edit.json"
"$P" set "$J" "$GROUND" quantity "corner_board_length" --line "Corner boards, 5/4 x 6 cedar" --status Known --source "made-up" > "$T/edit1.txt"
says "$T/edit1.txt" "wainscot"
"$P" set "$J" "$GROUND" quantity "corner_board_length_above_wainscot" --line "Corner boards, 5/4 x 6 cedar" --status Known --source "made-up" > "$T/edit1b.txt" 2>&1 && { echo "FAIL: the figure was accepted with its entry taken off"; cat "$T/edit1b.txt"; exit 1; } || true
says "$T/edit1b.txt" "the line has none"
"$P" line "$J" "$GROUND" "Corner boards, bare" "corner_board_length_above_wainscot" --cost-type material > "$T/edit2.txt" 2>&1 && { echo "FAIL: a line priced by the figure with no entry was accepted"; cat "$T/edit2.txt"; exit 1; } || true
says "$T/edit2.txt" "the line has none"
echo "   (the same refusal for the figure beside it, net_area_below_wainscot)"
"$P" line "$J" "$GROUND" "Siding, bare" "net_area_below_wainscot" --cost-type material > "$T/edit3.txt" 2>&1 && { echo "FAIL: accepted"; exit 1; } || true
says "$T/edit3.txt" "the line has none"
cp "$T/before-edit.json" "$J"

echo
echo "== 3. plumbline trim --pieces: the boards on a ground wall are two of 7 ft that start 3 ft above the wall's foot; on an upper wall two of 9 ft from its foot =="
"$P" trim "$J" --pieces > "$T/pieces.txt"
python3 - "$T/pieces.txt" "$GROUND" "$UPPER" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
ground, upper = sys.argv[2], sys.argv[3]
# a piece line: <name> [<wall>], priced by "<line>": <n> LF from (x, y, z) to (x, y, z); ...
pat = re.compile(r"\[(wall-\d+)\], priced by \"Corner boards[^\"]*\": ([\d.]+) LF from \(([-\d.]+), ([-\d.]+), ([-\d.]+)\) to \(([-\d.]+), ([-\d.]+), ([-\d.]+)\)")
found = [m.groups() for m in map(pat.search, text.splitlines()) if m]
for wall, foot, long_, up in ((ground, 0.0, 7.0, 3.0), (upper, 10.0, 9.0, 0.0)):
    mine = [g for g in found if g[0] == wall]
    got = [(float(g[1]), float(g[4]), float(g[7])) for g in mine]
    print("    %s: %d pieces, each %s LF, from %s ft to %s ft (the wall's foot is at %s ft); by hand 2 pieces of %s ft from %s ft" % (wall, len(mine), sorted({x[0] for x in got}), sorted({x[1] for x in got}), sorted({x[2] for x in got}), foot, long_, foot + up))
    if len(mine) != 2 or any(abs(l - long_) > 1e-6 or abs(z0 - (foot + up)) > 1e-6 for l, z0, z1 in got):
        print("FAIL: %s: the pieces are not two of %s ft from %s ft" % (wall, long_, foot + up)); sys.exit(1)
PY
echo "   the pieces of every corner-board line add to what the estimate prices: 128 LF; of every window's casing, 289 LF"
says "$T/pieces.txt" "corner boards: 16 pieces come to 128.00 LF; the lines price 128.00 LF"
says "$T/pieces.txt" "casing: 68 pieces come to 289.00 LF; the lines price 289.00 LF"
says "$T/pieces.txt" "fascia: 4 pieces come to 144.00 LF; the lines price 144.00 LF"
says "$T/pieces.txt" "soffit: 4 pieces come to 108.00 SF; the lines price 108.00 SF"

echo
echo "== 4. plumbline trim says what it said: the walls' figures are 20 and 18 LF, the soffit's 2.25 SF in two runs =="
says "$T/trim.txt" "corner board 20 LF"
says "$T/trim.txt" "corner board 18 LF"
says "$T/trim.txt" "4 corners, 2.25 SF in all"

echo
echo "== 5. The starter wainscot wall on a 10 ft wall with two outside corners and its 3 ft: 14 LF where it was 20 =="
K="$T/starter.json"
"$P" new "$K" "Box (not a real job)" >/dev/null
A=(--status Assumed --source "made-up tracing")
S="$(grep -oE 'level-[0-9]+' <<<"$("$P" level "$K" ground --elevation "0 ft" --height "10 ft" "${A[@]}")" | head -1)"
WT="$(grep -oE 'type-[0-9]+' <<<"$("$P" type "$K" wall "wall 6 in" --thickness "6 in" "${A[@]}")" | head -1)"
W="$("$P" wall "$K" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "10 ft" "${A[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" import assemblies "$K" starter >/dev/null
"$P" apply "$K" "$W" "Exterior wall, 2x6, lap siding over a stone wainscot" --source "the starter library" >/dev/null
"$P" estimate "$K" > "$T/starter.txt"
handre "$T/starter.txt" "Corner boards (wainscot wall)" '14.00 LF' '^ *Corner boards +~14\.00 LF'
W2="$("$P" wall "$K" "$S" "$WT" --points "30,0 54,0 54,16 30,16" --closed --height "10 ft" "${A[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" apply "$K" "$W2" "Exterior wall, 2x6, lap siding" --source "the starter library" >/dev/null
"$P" estimate "$K" > "$T/starter2.txt"
handre "$T/starter2.txt" "Corner boards (plain wall)" '20.00 LF' '^ *Corner boards +~20\.00 LF'

echo
echo "== 6. The front door has no casing line by length, and the foundation walls have no corner-board line =="
FRONT="$(grep -E '^front door  \[door\]' "$T/est.txt" | head -1 | grep -oE 'door-[0-9]+')"
"$P" show "$J" "$FRONT" > "$T/front.txt"
says "$T/front.txt" "Casing and trim"
lacks "$T/front.txt" "Exterior casing"
FOUND="$(grep -E '^wall 1  \[wall\]' "$T/est.txt" | head -1 | grep -oE 'wall-[0-9]+')"
"$P" show "$J" "$FOUND" > "$T/found.txt"
lacks "$T/found.txt" "Corner boards"
echo
echo "sixteen-trim: the six points hold"
