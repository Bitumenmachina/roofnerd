#!/usr/bin/env bash
# Loose ends at the command line (DECISIONS.md call 226; contract 226M; the six steps are in STATUS.md, "Loose ends at the command line: a
# demonstration"). Every figure is made up. Run from the repository root:
#   demos/loose-ends.sh
# The cost of a try (step 2) is the program a person is given, a release build: PLUMBLINE_BIN names a folder holding the program and, where it is
# a release build (its folder is called release), a try over a tenth of a second ends the script. Where it is not (the build the tests use is
# four times slower), the figure is said and the bound is not applied, and the script says so. Every step ends the script when the program does
# not say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
BIN="${PLUMBLINE_BIN:-$PWD/target/debug}"
P="$BIN/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
K=(--status Known --source "made up")
A=(--status Assumed --source "made up")
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3"; exit 1; }; printf '    %s\n' "$2"; }
not_said() { ! grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: said: $2"; echo "$3"; exit 1; }; printf '    (not said: %s)\n' "$2"; }
is() { [ "$2" = "$3" ] || { echo "FAIL: $1: got $2, wanted $3"; exit 1; }; printf '    %-62s %s\n' "$1" "$2"; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; echo "$out"; exit 1; }
  printf '    refused: %s\n' "$(head -1 <<<"$out" | cut -c1-200)"
}
word() { grep -oE "$2-[0-9]+" <<<"$1" | head -1; }

echo "== 1. A length in a sentence is said as a builder says it; a quantity on a line is not changed =="
J="$T/job.json"
"$P" new "$J" "Loose ends (made up)" >/dev/null
LEVEL="$(word "$("$P" level "$J" Ground --elevation "0 ft" --height "9 ft" "${K[@]}")" level)"
WT="$(word "$("$P" type "$J" wall "Wall 6" --thickness "6 in" "${K[@]}")" type)"
DT="$(word "$("$P" type "$J" door "Door 3068" --width "3 ft" --height "6 ft 8 in" "${K[@]}")" type)"
WALLS="$("$P" wall "$J" "$LEVEL" "$WT" --points "0,0 20,0 20,12 0,12" --closed --height "9 ft" "${K[@]}")"
W1="$(word "$WALLS" wall)"
"$P" door "$J" "$W1" "$DT" --offset "4 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" import items "$J" starter >/dev/null
"$P" takeoff "$J" > "$T/takeoff-before.txt"
BEFORE="$(sha256sum "$J" | cut -d' ' -f1)"
OUT="$("$P" try "$J" wall "$LEVEL" "$WT" --points "0,0 10,0" --height "9 ft" "${K[@]}" 2>&1 || true)"
says "a wall over the south wall" "lies along wall 1 for 10 ft" "$OUT"
not_said "a wall over the south wall" "10.00" "$OUT"
DUCT="$(word "$("$P" type "$J" duct "Duct 14 x 8" --width "14 in" --depth "8 in" "${K[@]}")" type)"
BEFORE="$(sha256sum "$J" | cut -d' ' -f1)"
OUT="$("$P" try "$J" run "$LEVEL" duct "$DUCT" --points "-2,6 22,6" --height "8 ft" "${A[@]}")"
is "a duct through two walls says its hole twice as a builder does" "$(grep -c '14 in by 8 in' <<<"$OUT")" "2"
not_said "the holes" "14.00" "$OUT"
is "the job is byte for byte what it was" "$(sha256sum "$J" | cut -d' ' -f1)" "$BEFORE"
"$P" takeoff "$J" > "$T/takeoff-after.txt"
cmp -s "$T/takeoff-before.txt" "$T/takeoff-after.txt" || { echo "FAIL: the takeoff is not what it was"; diff "$T/takeoff-before.txt" "$T/takeoff-after.txt"; exit 1; }
says "the takeoff writes a quantity as it did" "wall face, net     556.00 SF" "$(cat "$T/takeoff-after.txt")"

echo "== 2. What a try costs, on the sample whole house: a partition tried five times =="
H="$T/house.json"
"$P" sixteen "$H" >/dev/null
SUM="$(sha256sum "$H" | cut -d' ' -f1)"
is "the sample whole house is what it was (sha256 begins)" "${SUM:0:16}" "880d9fd3c364b5f7"
PART="$(word "$("$P" show "$H" | grep -E '^ +type-[0-9]+ +Partition' | head -1)" type)"
[ -n "$PART" ] || { echo "FAIL: the sample has no partition type"; exit 1; }
LVL="$(word "$("$P" show "$H" | grep -E '^ +level-[0-9]+ +first floor' | head -1)" level)"
[ -n "$LVL" ] || { echo "FAIL: the sample has no first floor"; exit 1; }
OUT="$("$P" try "$H" wall "$LVL" "$PART" --points "10,5 10,15" --height "9 ft" "${A[@]}" --timed 5)"
echo "$OUT" | grep -E '^(try [0-9]|the least|writing)' | sed 's/^/    | /'
LEAST="$(grep -oE 'the least of 5: [0-9.]+' <<<"$OUT" | grep -oE '[0-9.]+$')"
if [[ "$BIN" == */release ]]; then
  awk -v l="$LEAST" 'BEGIN{exit !(l < 0.1)}' || { echo "FAIL: a try takes $LEAST s in the program a person is given: over a tenth of a second"; exit 1; }
  printf '    a try takes %s s in the release build: under a tenth of a second\n' "$LEAST"
else
  printf '    a try takes %s s in %s: NOT the build a person is given, so the tenth-of-a-second bound is not applied here (run with PLUMBLINE_BIN=<a release folder>)\n' "$LEAST" "$BIN"
fi
is "the job is byte for byte what it was" "$(sha256sum "$H" | cut -d' ' -f1)" "$SUM"
# The next name the job would give: a wall made after the tries is named what it would have been without them.
cp "$H" "$T/house2.json"
N1="$("$P" wall "$H" "$LVL" "$PART" --points "10,5 10,15" --height "9 ft" "${A[@]}")"
N2="$("$P" wall "$T/house2.json" "$LVL" "$PART" --points "10,5 10,15" --height "9 ft" "${A[@]}")"
is "the next wall is named as if nothing had been tried" "$N1" "$N2"
# (There is no undo list at the command line: each verb is one step. The model's tests hold it, on the same house: crates/world/tests/call219m.rs.)

echo "== 3. The database's reply: one line about how cells are shown, and a tab named as typed =="
"$P" new "$T/a.json" "Database (made up)" >/dev/null
for t in items connections assemblies; do "$P" import $t "$T/a.json" starter >/dev/null; done
"$P" export database "$T/a.json" "$T/my.xlsx" >/dev/null
python3 -I - "$T/my.xlsx" "$T/edited.xlsx" <<'PY'
import sys, zipfile, re
src, dst = sys.argv[1:3]
zin = zipfile.ZipFile(src)
zout = zipfile.ZipFile(dst, "w", zipfile.ZIP_DEFLATED)
names = zin.namelist()
n = len([x for x in names if re.match(r"xl/worksheets/sheet\d+\.xml$", x)]) + 1
sheet = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>Remember</t></is></c></row><row r="2"><c r="A2" t="inlineStr"><is><t>call the supplier</t></is></c></row></sheetData></worksheet>'
for name in names:
    data = zin.read(name)
    if name == "xl/workbook.xml":
        t = data.decode()
        t = t.replace("</sheets>", '<sheet name="Notes" sheetId="%d" r:id="rIdNotes"/></sheets>' % (n + 100))
        data = t.encode()
    elif name == "xl/_rels/workbook.xml.rels":
        t = data.decode()
        t = t.replace("</Relationships>", '<Relationship Id="rIdNotes" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet%d.xml"/></Relationships>' % n)
        data = t.encode()
    elif name == "[Content_Types].xml":
        t = data.decode()
        t = t.replace("</Types>", '<Override PartName="/xl/worksheets/sheet%d.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>' % n)
        data = t.encode()
    zout.writestr(name, data)
zout.writestr("xl/worksheets/sheet%d.xml" % n, sheet)
zout.close()
PY
"$P" new "$T/b.json" "Second (made up)" >/dev/null
OUT="$("$P" import database "$T/b.json" "$T/edited.xlsx")"
is "how cells are shown is said once" "$(grep -c 'how its cells are shown' <<<"$OUT")" "1"
says "the tab as it was typed" 'not read: the tab "Notes": this program has no table of that name' "$OUT"

echo "== 4. A rule taken away =="
K2="$T/k.json"
"$P" new "$K2" "A kitchen (made up)" >/dev/null
S="$(word "$("$P" level "$K2" ground --elevation "0 ft" --height "9 ft" "${K[@]}")" level)"
W6="$(word "$("$P" type "$K2" wall "wall 6 in" --thickness "6 in" "${K[@]}")" type)"
read -r _ EAST _ _ <<<"$("$P" wall "$K2" "$S" "$W6" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
ROOM="$(word "$("$P" room "$K2" "$S" Kitchen --at "12,8" "${K[@]}")" room)"
"$P" import items "$K2" starter >/dev/null
HEAD="Code,What it is,Number reads,Width,Depth,Height,Stands,Drawn as,Stocked widths,How sure,Where from"
printf '%s\nX,base,the width,from the number,24 in,30 in,on the floor,base cabinet,,Assumed,a made-up rule\n' "$HEAD" > "$T/x.csv"
"$P" import cabinet-rules "$K2" "$T/x.csv" >/dev/null
OUT="$("$P" cabinets "$K2" "$S" "$EAST" --in "$ROOM" --from "1 ft" --boxes "X30" "${A[@]}")"
says "X30 is made by the rule" "X30 was made by the rule for X: 30 in wide, 24 in deep and 30 in high" "$OUT"
OUT="$("$P" remove-cabinet-rule "$K2" X)"
echo "$OUT" | sed 's/^/    | /'
says "the rule taken away" "the rule for X was taken away: no rule reads X now" "$OUT"
refused "no cabinet rule reads X" "$P" cabinets "$K2" "$S" "$EAST" --in "$ROOM" --from "6 ft" --boxes "X33" "${A[@]}"
"$P" export items "$K2" "$T/items.csv" >/dev/null
grep -E '^X30,' "$T/items.csv" | grep -qF "the cabinet rule for X" || { echo "FAIL: X30 does not say the rule for X made it"; exit 1; }
echo "    X30 is still in the chest and still says the rule for X made it"
"$P" export cabinet-rules "$K2" "$T/rules.csv" >/dev/null
sed 's/^B,base,the width,from the number,24 in,/B,base,the width,from the number,21 in,/' "$T/rules.csv" > "$T/rules21.csv"
cmp -s "$T/rules.csv" "$T/rules21.csv" && { echo "FAIL: B was not changed"; exit 1; }
"$P" import cabinet-rules "$K2" "$T/rules21.csv" >/dev/null
OUT="$("$P" remove-cabinet-rule "$K2" B)"
says "B goes back" "B goes back to the program's own" "$OUT"
OUT="$("$P" cabinets "$K2" "$S" "$EAST" --in "$ROOM" --from "6 ft" --boxes "B45" "${A[@]}")"
says "B45 is made 24 in deep" "B45 was made by the rule for B: 45 in wide, 24 in deep" "$OUT"
BEFORE="$(sha256sum "$K2" | cut -d' ' -f1)"
OUT="$("$P" remove-cabinet-rule "$K2" SB)"
says "a rule the job does not carry" "SB is the program's own rule and the job carries nothing of it: nothing was changed" "$OUT"
is "the job is byte for byte what it was" "$(sha256sum "$K2" | cut -d' ' -f1)" "$BEFORE"

echo "== 5. A height the rule does not stock is flagged as a width is =="
K3="$T/k3.json"
"$P" new "$K3" "A kitchen (made up)" >/dev/null
S="$(word "$("$P" level "$K3" ground --elevation "0 ft" --height "9 ft" "${K[@]}")" level)"
W6="$(word "$("$P" type "$K3" wall "wall 6 in" --thickness "6 in" "${K[@]}")" type)"
read -r _ EAST _ _ <<<"$("$P" wall "$K3" "$S" "$W6" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
ROOM="$(word "$("$P" room "$K3" "$S" Kitchen --at "12,8" "${K[@]}")" room)"
"$P" import items "$K3" starter >/dev/null
OUT="$("$P" cabinets "$K3" "$S" "$EAST" --in "$ROOM" --from "1 ft" --boxes "W3342" "${A[@]}")"
says "W3342 is made" "W3342 was made by the rule for W: 33 in wide, 12 in deep and 42 in high" "$OUT"
says "the width is flagged" "33 in is not a stocked width of W" "$OUT"
says "the height is flagged" "42 in is not a stocked height of W" "$OUT"
OUT="$("$P" cabinets "$K3" "$S" "$EAST" --in "$ROOM" --from "6 ft" --boxes "W3030" "${A[@]}")"
not_said "W3030 is a stocked box" "not a stocked" "$OUT"
"$P" export cabinet-rules "$K3" "$T/r3.csv" >/dev/null
is "the table's last heading" "$(head -1 "$T/r3.csv" | awk -F, '{print $NF}')" "Stocked heights"

echo "== 6. The sample whole house is what it was =="
"$P" sixteen "$T/again.json" >/dev/null
is "plumbline sixteen (sha256 begins)" "$(sha256sum "$T/again.json" | cut -c1-16)" "880d9fd3c364b5f7"
echo "ok"
