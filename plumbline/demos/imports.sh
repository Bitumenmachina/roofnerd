#!/usr/bin/env bash
# Tables brought in, at the command line: the job's price list out as CSV and back; a
# supplier's own list as CSV and as a workbook; an assembly library out and back; an
# estimate made before, set against the job line by line; and what is refused.
# Run from the repository root:  demos/imports.sh
# The job is the priced sample (plumbline priced): copper bought by the pound from three
# made-up suppliers. The tables are in demos/tables (tools/made-up-tables.py writes them).
# Every supplier, price and estimate here is made up. Every step fails the script if the
# program does not do what the step says.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet -p plumbline
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3"; exit 1; }; printf '    %s\n' "$2"; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | head -1 | cut -c1-230
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
}
amount() { # the side-by-side's figure for a line at the last supplier's column
  awk -v line="$1" 'index($0, line) { print $NF }' "$2"
}

echo "== The job: the priced sample =="
"$P" priced "$J" >/dev/null
"$P" estimate "$J" > "$T/before.txt"
sed -n '/SUPPLIERS, SIDE BY SIDE/,$p' "$T/before.txt" | sed 's/^/    /'

echo
echo "== 1. Its price list out as CSV, into an empty job, and out again =="
"$P" export prices "$J" "$T/prices.csv"
sed 's/^/    /' "$T/prices.csv"
"$P" new "$T/empty.json" "Prices brought in (not a real job)" >/dev/null
OUT="$("$P" import prices "$T/empty.json" "$T/prices.csv" --add)"
# Seven rows: two materials added and priced, and five suppliers' prices. Each row of a
# material adds it and sets its price, so the two of them are said twice.
says "all seven rows used" "not used: 0" "$OUT"
"$P" export prices "$T/empty.json" "$T/prices-again.csv" >/dev/null
cmp -s "$T/prices.csv" "$T/prices-again.csv" || { echo "FAIL: the price list did not come back as it went"; diff "$T/prices.csv" "$T/prices-again.csv"; exit 1; }
echo "    the price list written from the job it was brought into is byte for byte the first"
"$P" check "$T/empty.json"
OUT="$("$P" import prices "$J" "$T/prices.csv")"
says "back into the job it came from" "the same as the job already has: 7" "$OUT"
says "and nothing set" "set: 0" "$OUT"

echo
echo "== 2. A supplier's own list (CSV): its own headings, two rows above them, five rows =="
sed 's/^/    /' demos/tables/tinners-yard.csv
cp "$J" "$T/before.json"
refused "say how sure these prices are and where they came from" "$P" import prices "$J" demos/tables/tinners-yard.csv --supplier "Tinners Yard"
cmp -s "$J" "$T/before.json" || { echo "FAIL: a refused list changed the job"; exit 1; }
OUT="$("$P" import prices "$J" demos/tables/tinners-yard.csv --supplier "tinners yard" --status Known --source "Tinners Yard price list (made up)")"
echo "$OUT" | sed 's/^/    /'
says "a price changed" "Copper, 16 oz from Tinners Yard: \$9.10/lb becomes \$9.05/lb" "$OUT"
says "a price that supplier had not given" "Copper, 20 oz from Tinners Yard: \$9.40/lb, which Tinners Yard had not quoted" "$OUT"
says "a material the job does not have" "row 6: the job has no material called \"Zinc, 0.7 mm\"" "$OUT"
says "the wrong unit" "row 7: Copper, 16 oz: priced by the sheet; the job buys Copper, 16 oz by the lb" "$OUT"
says "a price that is a word" "row 8: Lead-coated copper: its price, \"call\", is not a figure" "$OUT"
says "the rows above the headings" "not read: 2 rows above the headings" "$OUT"
says "a column with no use" "not read: the column \"Lead time\"" "$OUT"
"$P" check "$J"
"$P" estimate "$J" > "$T/after.txt"
sed -n '/SUPPLIERS, SIDE BY SIDE/,$p' "$T/after.txt" | sed 's/^/    /'
# Each copper line at Tinners Yard is its pounds times the new price. The pounds are the
# generic amount over the generic $9.00/lb: 4,150.70 / 9 = 461.1889; 2,490.42 / 9 =
# 276.7133; 1,113.24 / 9 = 123.6933; 270.00 / 9 = 30.
#   south   461.1889 x 9.05 = 4,173.76      east     276.7133 x 9.05 = 2,504.26
#   shed    123.6933 x 9.40 = 1,162.72      headwall  30      x 9.40 =   282.00
#   total   4,173.76 + 2,504.26 + 1,162.72 + 282.00 = 8,122.74, and no line left out
is() { [ "$2" = "$3" ] || { echo "FAIL: $1: got $2, wanted $3"; exit 1; }; printf '    %-44s %s\n' "$1" "$2"; }
is "south at Tinners Yard" "$(amount "south: Standing seam panels" "$T/after.txt")" "\$4,173.76"
is "east at Tinners Yard" "$(amount "east: Standing seam panels" "$T/after.txt")" "\$2,504.26"
is "shed at Tinners Yard" "$(amount "shed: Standing seam panels" "$T/after.txt")" "~\$1,162.72"
is "headwall at Tinners Yard" "$(amount "headwall: Headwall flashing" "$T/after.txt")" "~\$282.00"
is "all of them" "$(awk '$1 == "total" { print $NF }' "$T/after.txt")" "~\$8,122.74"
says "none left out" "4 of 4         4 of 4         4 of 4         4 of 4" "$(cat "$T/after.txt")"

echo
echo "== 3. The same list as a workbook: a second sheet, and one price that is a formula =="
cp "$T/before.json" "$T/x.json"
OUT="$("$P" import prices "$T/x.json" demos/tables/tinners-yard.xlsx --supplier "Tinners Yard" --status Known --source "Tinners Yard price list (made up)")"
echo "$OUT" | sed 's/^/    /'
says "the same two prices" "Copper, 20 oz from Tinners Yard: \$9.40/lb, which Tinners Yard had not quoted" "$OUT"
says "the other sheet" "not read: the sheet \"Prices\" was read; 1 other sheet was not: \"Terms\"" "$OUT"
says "the formula" "not read: 1 cell holds a formula: what it last worked out to was read, not the formula" "$OUT"
says "how cells are shown" "not read: how its cells are shown" "$OUT"
cmp -s "$T/x.json" "$J" || { echo "FAIL: the workbook and the CSV did not leave the same job"; exit 1; }
echo "    the job is byte for byte what the CSV left"

echo
echo "== 4. Its assemblies out as CSV, into an empty job, and out again =="
"$P" export assemblies "$J" "$T/assemblies.csv"
cut -c1-150 "$T/assemblies.csv" | sed 's/^/    /'
"$P" new "$T/empty2.json" "Assemblies brought in (not a real job)" >/dev/null
OUT="$("$P" import assemblies "$T/empty2.json" "$T/assemblies.csv")"
echo "$OUT" | sed 's/^/    /'
says "every row used" "not used: 0" "$OUT"
"$P" export assemblies "$T/empty2.json" "$T/assemblies-again.csv" >/dev/null
cmp -s "$T/assemblies.csv" "$T/assemblies-again.csv" || { echo "FAIL: the assemblies did not come back as they went"; exit 1; }
echo "    the assemblies written from the job they were brought into are byte for byte the first"
"$P" check "$T/empty2.json"

echo
echo "== 5. An estimate made before, set against the job =="
sed 's/^/    /' demos/tables/prior-estimate.csv
"$P" estimate "$J" > "$T/cost-before.txt"
OUT="$("$P" import prior "$J" demos/tables/prior-estimate.csv --name "Earlier estimate" --status Assumed --source "a made-up earlier estimate")"
echo "$OUT" | sed 's/^/    /'
"$P" check "$J"
OUT="$("$P" compare "$J")"
echo "$OUT" | sed 's/^/    /'
# Standing seam panels: prior 700 lb at $8.50, $5,950.00. The job has three lines called
# that: 461.1889 + 276.7133 + 123.6933 = 861.5955 lb (861.60), and 4,150.70 + 2,490.42 +
# 1,113.24 = $7,754.36; the shed's is Assumed, so the three together are.
#   this job less the prior: 861.60 - 700 = +161.60 lb; 7,754.36 - 5,950.00 = +$1,804.36
# Underlayment: prior 650 SF, $585.00. The job: 419.26 + 251.56 = 670.82 SF (the lines'
# own figures, 419.2627 + 251.5576 = 670.8204), and 377.34 + 226.40 = $603.74.
#   this job less the prior: +20.82 SF; +$18.74
# Gutters 120 LF at $14.00 = $1,680.00 and the dumpster, $450.00: the job has neither.
# The prior estimate adds to 5,950 + 585 + 1,680 + 450 = $8,665.00.
says "the panels, prior" "prior:     700.00 lb at \$8.50: \$5,950.00" "$OUT"
says "the panels, this job" "this job:  ~861.60 lb: ~\$7,754.36  (3 lines on south, east, shed)" "$OUT"
says "the panels, the difference" "this job less the prior: quantity +161.60 lb; amount +\$1,804.36" "$OUT"
says "the underlayment, this job" "this job:  670.82 SF: \$603.74  (2 lines on south, east)" "$OUT"
says "the underlayment, the difference" "this job less the prior: quantity +20.82 SF; amount +\$18.74" "$OUT"
says "the gutters, worked" "prior:     120.00 LF at \$14.00: \$1,680.00 (its quantity times its unit cost)" "$OUT"
says "what the job has not" "IN THE PRIOR ESTIMATE AND NOT IN THIS JOB: 2" "$OUT"
says "what the prior has not" "IN THIS JOB AND NOT IN THE PRIOR ESTIMATE: 7" "$OUT"
says "a line with no price is no figure" "Copper standing seam, installed: no figure" "$OUT"
says "the prior's total" "the prior estimate's lines add to \$8,665.00" "$OUT"
says "and none of it in the job" "this job's cost is ~\$25,292.43; nothing of the prior estimate is in it" "$OUT"
"$P" estimate "$J" > "$T/cost-after.txt"
[ "$(grep 'Job cost' "$T/cost-before.txt")" = "$(grep 'Job cost' "$T/cost-after.txt")" ] || { echo "FAIL: bringing a prior estimate in changed the job's cost"; exit 1; }
echo "    the recap's job cost is what it was before the prior estimate was brought in"

echo
echo "== 6. What is refused, and leaves the job as it was =="
cp "$J" "$T/before.json"
printf 'not a table\001\002\n' > "$T/bad.bin"
refused "it is not a CSV file or an XLSX workbook" "$P" import prices "$J" "$T/bad.bin" --status Known --source "x"
printf 'Item,UOM\n"Copper, 16 oz",lb\n' > "$T/noprice.csv"
refused "not a price list this program can read" "$P" import prices "$J" "$T/noprice.csv" --status Known --source "x"
printf 'Item,Price\n"Copper, 16 oz",9.05\n' > "$T/nounit.csv"
refused "no column for what each thing is bought by" "$P" import prices "$J" "$T/nounit.csv" --status Known --source "x"
refused "the workbook has no sheet \"Nope\"; it has: Prices, Terms" "$P" import prices "$J" demos/tables/tinners-yard.xlsx --sheet Nope --status Known --source "x"
refused "--status and --source go together" "$P" import prices "$J" demos/tables/tinners-yard.csv --status Known
refused "what is brought in is Known or Assumed" "$P" import prices "$J" demos/tables/tinners-yard.csv --status Unknown --source "x"
refused "say how sure the prior estimate is" "$P" import prior "$J" demos/tables/prior-estimate.csv
refused "not an assembly library this program wrote" "$P" import assemblies "$J" demos/tables/tinners-yard.csv
refused "does not take --add" "$P" import assemblies "$J" "$T/assemblies.csv" --add
cmp -s "$J" "$T/before.json" || { echo "FAIL: a refused table changed the job"; exit 1; }
echo "    the job is byte for byte what it was before the refusals"

echo
echo "PASS: a price list and an assembly library went out as CSV and came back the same; a supplier's list was brought in from CSV and from a workbook with every unused row and column said; an estimate made before was set against the job line by line and added nothing to it"
