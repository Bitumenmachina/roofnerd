#!/usr/bin/env bash
# A cabinet by rule (call 215), at the command line: the letters of a cabinet's code and the number after them make the box, from a
# table the person owns. In the eight steps of STATUS.md ("A cabinet by rule: a demonstration"), then the table as a tab of the
# database (call 223, as built, point 6). The figures are made up or are the starter rows': the chest has 27 box codes (B x 9, SB x 3,
# DB x 2, BLB, LS, W x 8, T x 2, F), eight rules (B, SB, DB, BLB, LS, W, T, F) make them.
# Run from the repository root:  demos/cabinet-rules.sh
# The jobs are made up (no real job data) and live in a temporary folder. Every step ends the script when the program does not say
# what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
K=(--status Known --source "made-up survey")
N=(--status Assumed --source "made-up addition plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
fail() { echo "FAIL: $*"; exit 1; }

# A 24 ft by 16 ft house with a Kitchen, its east wall's face clear from 0.25 ft to 15.75 ft along it; the starter items are brought
# in if $2 is "chest". Sets $J, $S (the level), $EAST, $KITCHEN.
kitchen() {
  J="$1"
  rm -f "$J"
  "$P" new "$J" "A kitchen (not a real job)" >/dev/null
  S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
  local WT; WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
  read -r _ EAST _ _ <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
  KITCHEN="$(made "$("$P" room "$J" "$S" "Kitchen" --at "12,8" "${K[@]}")")"
  if [ "${2:-}" = "chest" ]; then "$P" import items "$J" starter >/dev/null; fi
}
# The number of items the job has, by its table (one heading row).
items_in() { "$P" export items "$1" "$T/count.csv" >/dev/null; echo $(( $(wc -l < "$T/count.csv") - 1 )); }
# One item's row of the items table.
row_of() { "$P" export items "$1" "$T/rows.csv" >/dev/null; grep -E "^\"?$2[\",]" "$T/rows.csv" | head -1; }

echo "== 1. Every starter cabinet code is what its rule makes: 27 of 27 =="
# A job with an EMPTY chest: every one of the 27 codes is asked for, so each is made by its rule; the items table that results must
# say, for each code, the starter row's kind, what it is drawn as and its three sizes (the starter chest's file is the other side).
CODES="$(awk -F, '$2=="12 Furnishings" && $1 ~ /^[A-Z]+[0-9]+$/ {print $1}' library/items.csv)"
[ "$(wc -w <<<"$CODES")" = 27 ] || fail "the starter chest does not have 27 cabinet codes: $(wc -w <<<"$CODES")"
BASES="$(grep -vE '^W' <<<"$CODES" | tr '\n' ' ')"
WALLS="$(grep -E '^W' <<<"$CODES" | tr '\n' ' ')"
"$P" new "$T/long.json" "A long kitchen (not a real job)" >/dev/null
LS="$(made "$("$P" level "$T/long.json" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
LWT="$(made "$("$P" type "$T/long.json" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r LSOUTH LEAST LNORTH _ <<<"$("$P" wall "$T/long.json" "$LS" "$LWT" --points "0,0 60,0 60,20 0,20" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
LROOM="$(made "$("$P" room "$T/long.json" "$LS" "Hall" --at "30,10" "${K[@]}")")"
# The bases, the tall boxes and the filler: 474 in, 39.5 ft, along the south wall (60 ft); the wall boxes, 201 in, along the north wall.
"$P" cabinets "$T/long.json" "$LS" "$LSOUTH" --in "$LROOM" --from "1 ft" --boxes "$BASES" "${N[@]}" > "$T/all-bases.txt"
"$P" cabinets "$T/long.json" "$LS" "$LNORTH" --in "$LROOM" --from "1 ft" --boxes "$WALLS" "${N[@]}" > "$T/all-walls.txt"
"$P" export items "$T/long.json" "$T/long-items.csv" >/dev/null
n=0
for code in $CODES; do
  want="$(grep -E "^$code," library/items.csv | cut -d, -f1-5,8,11)"
  got="$(grep -E "^$code," "$T/long-items.csv" | cut -d, -f1-5,8,11)"
  [ -n "$want" ] && [ "$want" = "$got" ] || { echo "FAIL: $code: the starter row says ($want) and the rule makes ($got)"; exit 1; }
  grep -E "^$code," "$T/long-items.csv" | cut -d, -f3 | grep -q "^furnishing$" || fail "$code is not a furnishing"
  n=$((n+1))
done
echo "    $n of 27: kind, what it is drawn as, width, depth and height are the starter row's"
echo "    made by their rules: $(cat "$T/all-bases.txt" "$T/all-walls.txt" | grep -c "was made by the rule for")"

echo
echo "== 2. A run of B15, B30 and B42 along the east wall: B42 is made by the rule for B =="
kitchen "$T/job.json" chest
BEFORE="$(items_in "$T/job.json")"
"$P" cabinets "$T/job.json" "$S" "$EAST" --in "$KITCHEN" --from "1 ft" --boxes "B15 B30 B42" "${N[@]}" | tee "$T/run.txt" | sed 's/^/    | /'
says "$T/run.txt" "set 3 boxes along $EAST in $KITCHEN: a run of 7.25 ft, from 1 ft to 8.25 ft along the wall"
says "$T/run.txt" "B42 was made by the rule for B: 42 in wide, 24 in deep and 34 1/2 in high; Assumed; 42 in is not a stocked width of B"
lacks "$T/run.txt" "B15 was made"
[ "$(items_in "$T/job.json")" = $((BEFORE+1)) ] || fail "the chest has not one more item"
echo "    the chest has one more item: $BEFORE then $((BEFORE+1))"
row_of "$T/job.json" B42 | grep -q "Assumed" || fail "B42 is not Assumed"
row_of "$T/job.json" B42 | grep -q "the cabinet rule for B" || fail "B42 does not say its rule"
echo "    B42 is Assumed and its row says the rule and how the number was read"

echo
echo "== 3. W3342 hangs where wall boxes hang, T2490 stands on the floor =="
kitchen "$T/w.json" chest
"$P" cabinets "$T/w.json" "$S" "$EAST" --in "$KITCHEN" --from "1 ft" --boxes "W3342" "${N[@]}" | tee "$T/w.txt" | sed 's/^/    | /'
says "$T/w.txt" "W3342, 2' 9\" wide, middle at 23.25, 2.375"
says "$T/w.txt" "hung 4' 6\" up"
says "$T/w.txt" "W3342 was made by the rule for W: 33 in wide, 12 in deep and 42 in high"
kitchen "$T/t.json" chest
"$P" cabinets "$T/t.json" "$S" "$EAST" --in "$KITCHEN" --from "1 ft" --boxes "T2490" "${N[@]}" | tee "$T/t.txt" | sed 's/^/    | /'
says "$T/t.txt" "T2490 was made by the rule for T: 24 in wide, 24 in deep and 90 in high"
lacks "$T/t.txt" "hung"
lacks "$T/t.txt" "not a stocked width"

echo
echo "== 4. SB42 is made by the rule for SB, not the rule for B =="
kitchen "$T/s.json" chest
"$P" cabinets "$T/s.json" "$S" "$EAST" --in "$KITCHEN" --from "1 ft" --boxes "SB42" "${N[@]}" | tee "$T/s.txt" | sed 's/^/    | /'
says "$T/s.txt" "SB42 was made by the rule for SB: 42 in wide, 24 in deep and 34 1/2 in high"
row_of "$T/s.json" SB42 | grep -q "the cabinet rule for SB" || fail "SB42 does not say the rule for SB"

echo
echo "== 5. Refused, each saying how to write it, the job unchanged =="
cp "$T/job.json" "$T/before.json"
refused "$P" cabinets "$T/job.json" "$S" "$EAST" --in "$KITCHEN" --from "9 ft" --boxes "W33" "${N[@]}"
says "$T/out.txt" "W reads four digits, the width then the height: W3342 is 33 in wide and 42 in high"
refused "$P" cabinets "$T/job.json" "$S" "$EAST" --in "$KITCHEN" --from "9 ft" --boxes "B" "${N[@]}"
says "$T/out.txt" "B has no number after its letters: B reads the width in inches: B42 is 42 in wide"
refused "$P" cabinets "$T/job.json" "$S" "$EAST" --in "$KITCHEN" --from "9 ft" --boxes "X42" "${N[@]}"
says "$T/out.txt" "X42 is not in the tool chest, and no cabinet rule reads X"
says "$T/out.txt" "The letters the rules read are B, SB, DB, BLB, LS, W, T, F"
cmp -s "$T/job.json" "$T/before.json" || fail "a refusal changed the job"
echo "    the job is as it was"

echo
echo "== 6. What was made is listed, written out and brought back, row for row =="
"$P" items "$T/job.json" > "$T/items.txt"
says "$T/items.txt" "B42"
"$P" export items "$T/job.json" "$T/items-out.csv" >/dev/null
"$P" new "$T/fresh.json" "Another (not a real job)" >/dev/null
"$P" import items "$T/fresh.json" "$T/items-out.csv" >/dev/null
"$P" export items "$T/fresh.json" "$T/items-back.csv" >/dev/null
cmp -s "$T/items-out.csv" "$T/items-back.csv" || { echo "FAIL: the items did not come back the same"; diff "$T/items-out.csv" "$T/items-back.csv" | head; exit 1; }
echo "    B42 is in the table and the table comes back byte for byte"

echo
echo "== 7. The rules as a table, the rule for B changed to 21 in deep, brought back =="
"$P" export cabinet-rules "$T/job.json" "$T/rules.csv" | tee "$T/rules-said.txt" | sed 's/^/    | /'
says "$T/rules-said.txt" "8 rules under its headings"
# call 226, point 5: the table gains "Stocked heights", last, so a table of eleven columns still reads (the lead's edit)
[ "$(head -1 "$T/rules.csv")" = "Code,What it is,Number reads,Width,Depth,Height,Stands,Drawn as,Stocked widths,How sure,Where from,Stocked heights" ] || fail "the rules' headings are not what they should be"
grep -q '^B,base,the width,from the number,24 in,34 1/2 in,on the floor,base cabinet,' "$T/rules.csv" || fail "the rule for B is not written as it should be"
# The same table brought back: no change at all, and the job carries none of the rules.
cp "$T/job.json" "$T/job-same.json"
"$P" import cabinet-rules "$T/job.json" "$T/rules.csv" > "$T/rules-in.txt"
cmp -s "$T/job.json" "$T/job-same.json" || fail "the same table brought back changed the job"
grep -q "cabinetRules" "$T/job.json" && fail "a job that changed no rule carries rules"
echo "    the same table brought back changes nothing, and the job carries no rules of its own"
# B changed to 21 in deep.
sed 's/^B,base,the width,from the number,24 in,/B,base,the width,from the number,21 in,/' "$T/rules.csv" > "$T/rules-21.csv"
"$P" import cabinet-rules "$T/job.json" "$T/rules-21.csv" | tee "$T/rules-21.txt" | sed 's/^/    | /'
says "$T/rules-21.txt" "B: changed: its depth"
grep -q "cabinetRules" "$T/job.json" || fail "the changed rule is not in the job"
"$P" cabinets "$T/job.json" "$S" "$EAST" --in "$KITCHEN" --from "9 ft" --boxes "B45" "${N[@]}" | tee "$T/b45.txt" | sed 's/^/    | /'
says "$T/b45.txt" "B45 was made by the rule for B: 45 in wide, 21 in deep and 34 1/2 in high"
row_of "$T/job.json" B45 | grep -q ",21 in," || fail "B45 is not 21 in deep"
row_of "$T/job.json" B42 | grep -q ",24 in," || fail "B42, made before, is not still 24 in deep"
echo "    B42, made before the change, is still 24 in deep"
# The changed rules written out and brought into a new job: the same rows.
"$P" export cabinet-rules "$T/job.json" "$T/rules-back.csv" >/dev/null
"$P" new "$T/fresh2.json" "Another (not a real job)" >/dev/null
"$P" import cabinet-rules "$T/fresh2.json" "$T/rules-back.csv" >/dev/null
"$P" export cabinet-rules "$T/fresh2.json" "$T/rules-back2.csv" >/dev/null
cmp -s "$T/rules-back.csv" "$T/rules-back2.csv" || fail "the rules did not come back the same"
grep -q '^B,base,the width,from the number,21 in,' "$T/rules-back2.csv" || fail "the changed rule did not come back"
echo "    the rules written out and brought into a new job are the same rows"
# One bad row and no rule at all comes in, the row and why named.
cp "$T/fresh2.json" "$T/fresh2-before.json"
sed 's/^F,filler,/F,a cupboard,/' "$T/rules.csv" > "$T/rules-bad.csv"
sed 's/^T,tall box,/T,wall box,/' "$T/rules-bad.csv" > "$T/rules-bad2.csv"
"$P" import cabinet-rules "$T/fresh2.json" "$T/rules-bad2.csv" > "$T/bad-said.txt" 2>&1 || true
says "$T/bad-said.txt" "a base, a wall box, a tall box or a filler"
says "$T/bad-said.txt" "a wall box hangs"

echo
echo "== 8. The saved job of a run of starter codes only is the job the program wrote before this work =="
kitchen "$T/base.json" chest
# The same kitchen the baseline was taken from: a 24 ft by 16 ft closed wall, the Kitchen, the starter items, B15 B30 SB36 and a countertop
# along the east wall from 1 ft. (The baseline was taken with the program as it was; the header's date is today's, so the line is set to "-".)
"$P" cabinets "$T/base.json" "$S" "$EAST" --in "$KITCHEN" --from "1 ft" --boxes "B15 B30 SB36" --countertop "${N[@]}" > "$T/base.txt"
lacks "$T/base.txt" "was made by the rule"
grep -q "cabinetRules" "$T/base.json" && fail "a job that never touched the rules carries them"
SUM="$(sed 's/"timestamp": ".*"/"timestamp": "-"/' "$T/base.json" | sha256sum | cut -c1-64)"
# call 233: the starter chest grew (the job carries the chest). Call 235: six of its sizes say "an assumption: no published
# size was found"; with those six put back to the typical text the job is 66e6a894...17cd, the figure call 233's worker took.
BASELINE="2a869247608b0104239f05784a5083a2677bab439e3c27c4dbd376839e7c36a6"
[ "$SUM" = "$BASELINE" ] || fail "the saved job is not the one the program wrote before: $SUM, not $BASELINE"
echo "    sha256 $SUM, the job the program wrote before"

echo
echo "== 9. The rules are a tab of the database (call 223, point 6), after Items =="
"$P" export database "$T/job.json" "$T/my.xlsx" | tee "$T/db-said.txt" | sed 's/^/    | /'
says "$T/db-said.txt" "Cabinet codes 8, Connections"
cat > "$T/tabs.py" <<'PY'
import sys, zipfile
import xml.etree.ElementTree as ET
ns = {"m": "http://schemas.openxmlformats.org/spreadsheetml/2006/main"}
z = zipfile.ZipFile(sys.argv[1])
print(",".join(s.get("name") for s in ET.fromstring(z.read("xl/workbook.xml")).find("m:sheets", ns)))
PY
TABS="$(python3 -I "$T/tabs.py" "$T/my.xlsx")"
[ "$TABS" = "Shapes,Items,Cabinet codes,Connections,Assemblies,Prices" ] || fail "the workbook's tabs are: $TABS"
echo "    the tabs, in order: $TABS"
"$P" export database "$T/job.json" "$T/db-folder" >/dev/null
[ -f "$T/db-folder/cabinet codes.csv" ] || fail "no file in the folder for the rules"
cmp -s "$T/db-folder/cabinet codes.csv" "$T/rules-back.csv" || fail "the folder's file is not what the table's own verb writes"
echo "    the folder has a file for it, byte for byte what export cabinet-rules writes"
# Into an empty job and out again: the same, and the rules come in too.
"$P" new "$T/empty.json" "Empty (not a real job)" >/dev/null
"$P" import database "$T/empty.json" "$T/my.xlsx" | tee "$T/db-in.txt" | sed 's/^/    | /'
says "$T/db-in.txt" "the tab Cabinet codes: 8 rows under its headings"
"$P" export database "$T/empty.json" "$T/db-folder-2" >/dev/null
for f in "$T/db-folder"/*.csv; do
  cmp -s "$f" "$T/db-folder-2/$(basename "$f")" || { echo "FAIL: $(basename "$f") did not come back the same"; exit 1; }
done
echo "    out, in to an empty job and out again: every table the same, byte for byte"
# One bad row in the rules tab and nothing comes in at all.
mkdir "$T/db-bad" && cp "$T/db-folder"/*.csv "$T/db-bad/"
sed -i 's/^F,filler,/F,a cupboard,/' "$T/db-bad/cabinet codes.csv"
"$P" new "$T/empty2.json" "Empty (not a real job)" >/dev/null
cp "$T/empty2.json" "$T/empty2-before.json"
refused "$P" import database "$T/empty2.json" "$T/db-bad"
says "$T/out.txt" "tab Cabinet codes, row"
says "$T/out.txt" "the job is as it was"
cmp -s "$T/empty2.json" "$T/empty2-before.json" || fail "a refused database changed the job"
echo "    one bad row in the rules tab and nothing came in"
echo
echo "cabinet-rules: all steps passed"
