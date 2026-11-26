#!/usr/bin/env bash
# The field's measurements brought back by file (call 234): the list to verify written as a sheet to fill on site, and what was
# found brought back, each row setting its value Known the way a typed measurement does. The job is the one demos/field.sh
# measures by typing (made by demos/field-job.sh, made up); the figures are that demonstration's own, worked by hand in STATUS.md
# ("Field entry"): the south wall 216 SF, $518.40, price $615.86; the ground level at 8 ft 10 1/2 in makes it 213 SF, $511.20,
# price $607.31; the duct's underside at 7 ft 10 3/4 in clears pipe A and leaves pipe B in the duct by 2 1/4 in, a finding.
# Run from the repository root:  demos/field-sheet.sh
# Every step fails the script if the program does not say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
not_said() { ! grep -qF -- "$2" "$1" || { echo "FAIL: it says: $2"; cat "$1"; exit 1; }; }
same_file() { cmp -s "$1" "$2" || { echo "FAIL: $3"; exit 1; }; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  printf '    refused: %s\n' "$(head -1 <<<"$out" | cut -c1-200)"
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\": $out"; exit 1; }
}
# fill <csv> <reference> <as found> [by] [on]: what a person does with a pencil, or in a workbook, on a row of the sheet
fill() { python3 -I - "$@" <<'PY'
import csv, sys
path, ref, found = sys.argv[1:4]
by = sys.argv[4] if len(sys.argv) > 4 else "JM"
on = sys.argv[5] if len(sys.argv) > 5 else "2026-11-26"
rows = list(csv.reader(open(path, newline="", encoding="utf-8")))
hit = 0
for r in rows[1:]:
    r += [""] * (10 - len(r))
    if r[9] == ref:
        r[6], r[7], r[8] = found, by, on
        hit += 1
assert hit == 1, "no row " + ref
csv.writer(open(path, "w", newline="", encoding="utf-8"), lineterminator="\n").writerows(rows)
PY
}
# reorder <csv> <out>: the rows reversed, and every other row that is not filled deleted
reorder() { python3 -I - "$1" "$2" <<'PY'
import csv, sys
rows = list(csv.reader(open(sys.argv[1], newline="", encoding="utf-8")))
head, body = rows[0], rows[1:]
body.reverse()
kept = [r for i, r in enumerate(body) if (len(r) > 6 and r[6]) or i % 2 == 0]
csv.writer(open(sys.argv[2], "w", newline="", encoding="utf-8"), lineterminator="\n").writerows([head] + kept)
PY
}
fill_field() { # the measurements demos/field.sh types, into a sheet
  fill "$1" "$GROUND elevation" "same"
  fill "$1" "$GROUND level height" "8'10\"1/2"
  fill "$1" "$DUCT line on the plan" "same"
  fill "$1" "$DUCT type" "same"
  fill "$1" "$DUCT height above its level" "bottom 7'10\"3/4"
}
fresh() { cp "$T/start.json" "$1"; }

echo "== 1. The sheet to fill: a row for each value, by dollars, each with its reference; nothing filled =="
eval "$(demos/field-job.sh "$J")"
cp "$J" "$T/start.json"
"$P" export field-sheet "$J" "$T/sheet.csv" > "$T/out.txt"
says "$T/out.txt" "a sheet to fill on site"
says "$T/out.txt" "2 of the 5 questions on the list to verify are not rows"
python3 -I - "$T/sheet.csv" <<'PY'
import csv, sys
rows = list(csv.reader(open(sys.argv[1], newline="", encoding="utf-8")))
assert rows[0][6:10] == ["As found", "Measured by", "On", "Reference"], rows[0]
refs = [r[9] for r in rows[1:] if len(r) > 9 and r[9]]
assert len(refs) == len(set(refs)) and len(refs) >= 6, refs
assert all(not r[6] for r in rows[1:] if len(r) > 6), "something is filled"
print("    %d rows, %d references, none the same, none filled" % (len(rows) - 1, len(refs)))
PY
"$P" export field-sheet "$J" "$T/sheet.xlsx" | head -1 | sed 's/^/    /'
echo "   brought straight back:"
"$P" import field-sheet "$J" "$T/sheet.csv" > "$T/out.txt"
says "$T/out.txt" "no value was measured"
same_file "$J" "$T/start.json" "a sheet with nothing filled changed the job"
echo "    the job is byte for byte what it was"

echo
echo "== 2. The measurements field.sh types, filled into the sheet and brought back =="
cp "$T/sheet.csv" "$T/one.csv"
fill_field "$T/one.csv"
"$P" import field-sheet "$J" "$T/one.csv" | tee "$T/out.txt" | sed 's/^/    /' | cut -c1-200
says "$T/out.txt" "the price was \$615.86 and is \$607.31"
says "$T/out.txt" "upper's floor is now at 8' 10 1/2\", Known; it was at 9' 0\""
says "$T/out.txt" "clash cleared"
says "$T/out.txt" "clash confirmed, now a finding"
says "$T/out.txt" "in each other by 2 1/4 in"
says "$T/out.txt" "5 values were measured"
cp "$J" "$T/by-sheet.json"

echo
echo "== 3. The job is the typed job =="
fresh "$T/typed.json"
"$P" measure "$T/typed.json" "$GROUND" elevation same --source "JM on site" >/dev/null
"$P" measure "$T/typed.json" "$GROUND" "level height" "8'10\"1/2" --source "JM with the laser" >/dev/null
"$P" measure "$T/typed.json" "$DUCT" "line on the plan" same --source "JM on site" >/dev/null
"$P" measure "$T/typed.json" "$DUCT" type same --source "JM on site" >/dev/null
"$P" measure "$T/typed.json" "$DUCT" "height above its level" "7'10\"3/4" --to bottom --source "JM with the laser" >/dev/null
for verb in estimate clashes vif; do
  "$P" "$verb" "$J" > "$T/a.txt"; "$P" "$verb" "$T/typed.json" > "$T/b.txt"
  same_file "$T/a.txt" "$T/b.txt" "$verb differs from the typed job"
  echo "    $verb: text for text what typing the same measurements leaves"
done
"$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"
says "$T/sheet.txt" "Gypsum board, hung and finished 213.00 SF \$2.40/SF \$511.20"
says "$T/sheet.txt" "PRICE \$607.31"
"$P" clashes "$J" > "$T/clashes.txt"
says "$T/clashes.txt" "In the same space: 1 (1 finding)"
says "$J" 'measured in the field by JM on 2026-11-26: field sheet \"one.csv\", row '

echo
echo "== 4. The rows in another order and half of them deleted =="
fresh "$J"
cp "$T/sheet.csv" "$T/two.csv"; fill_field "$T/two.csv"
reorder "$T/two.csv" "$T/cut.csv"
"$P" import field-sheet "$J" "$T/cut.csv" > "$T/out.txt"
says "$T/out.txt" "5 values were measured"
"$P" estimate "$J" > "$T/a.txt"; "$P" estimate "$T/typed.json" > "$T/b.txt"
same_file "$T/a.txt" "$T/b.txt" "a cut sheet came to another estimate"

echo
echo "== 5. One row with \"about nine feet\" brings nothing in =="
fresh "$J"
cp "$T/sheet.csv" "$T/bad.csv"
fill "$T/bad.csv" "$GROUND elevation" "same"
fill "$T/bad.csv" "$GROUND level height" "about nine feet"
refused "level height" "$P" import field-sheet "$J" "$T/bad.csv"
"$P" import field-sheet "$J" "$T/bad.csv" > "$T/out.txt" 2>&1 || true
says "$T/out.txt" "nothing was brought in"
same_file "$J" "$T/start.json" "a refused sheet changed the job"
echo "    the job is byte for byte what it was"

echo
echo "== 6. A value changed after the sheet went out =="
"$P" measure "$J" "$GROUND" "level height" "8'11\"" --source "KT with the tape" >/dev/null
cp "$J" "$T/changed.json"
cp "$T/sheet.csv" "$T/late.csv"; fill "$T/late.csv" "$GROUND level height" "8'10\"1/2"
refused "the sheet had 9 ft, Assumed and the job has 8' 11\", Known (KT with the tape)" "$P" import field-sheet "$J" "$T/late.csv"
same_file "$J" "$T/changed.json" "a stale row changed the job"
"$P" import field-sheet "$J" "$T/late.csv" --anyway > "$T/out.txt"
says "$T/out.txt" "1 value was measured"
"$P" show "$J" > "$T/show.txt"
says "$T/show.txt" "level height     8' 10 1/2\""

echo
echo "== 7. A row that agrees with what the job has =="
fresh "$J"
cp "$T/sheet.csv" "$T/agree.csv"; fill "$T/agree.csv" "$GROUND level height" "9 ft"
"$P" import field-sheet "$J" "$T/agree.csv" > "$T/out.txt"
says "$T/out.txt" "the price is \$615.86, as it was"
"$P" show "$J" > "$T/show.txt"
says "$T/show.txt" "level height     9 ft"
not_said "$T/show.txt" "~9 ft"

echo
echo "== 8. A second sheet that measures a Known value differently =="
cp "$T/by-sheet.json" "$J"
cp "$T/sheet.csv" "$T/second.csv"; fill "$T/second.csv" "$GROUND level height" "8'11\""
cp "$J" "$T/kept.json"
refused "8' 10 1/2\", Known" "$P" import field-sheet "$J" "$T/second.csv"
same_file "$J" "$T/kept.json" "a second sheet changed a Known value"
"$P" show "$J" > "$T/show.txt"
says "$T/show.txt" "level height     8' 10 1/2\""
echo "    the value is Known still, and was left"

echo
echo "== 9. Nothing of the sample whole house moves =="
"$P" sixteen "$T/house.json" >/dev/null
SUM="$(sha256sum "$T/house.json" | cut -c1-16)"
[ "$SUM" = "880d9fd3c364b5f7" ] || { echo "FAIL: the sample whole house is $SUM"; exit 1; }
echo "    plumbline sixteen: sha256 $SUM"
"$P" export field-sheet "$T/house.json" "$T/house-sheet.csv" > "$T/out.txt"
head -2 "$T/out.txt" | sed 's/^/    /' | cut -c1-200
"$P" import field-sheet "$T/house.json" "$T/house-sheet.csv" >/dev/null
[ "$(sha256sum "$T/house.json" | cut -c1-16)" = "$SUM" ] || { echo "FAIL: a blank sheet moved the house"; exit 1; }
"$P" export vif "$T/house.json" "$T/house-vif-1.csv" >/dev/null
"$P" export vif "$T/house.json" "$T/house-vif-2.csv" >/dev/null
same_file "$T/house-vif-1.csv" "$T/house-vif-2.csv" "export vif is not steady"
echo "    export vif of the sample house: $(sha256sum "$T/house-vif-1.csv" | cut -c1-16)"
echo
echo "field sheet: PASS"
