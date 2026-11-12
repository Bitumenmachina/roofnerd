#!/usr/bin/env bash
# D5 demonstration: the estimate sheet and money. Material, labor and general columns;
# in-house labor by a production rate; a labor line switched off; a subcontract line;
# fabrication and supervision as lines of their own; general conditions typed by hand;
# overhead and profit; the recap; three suppliers side by side; and an element's material
# changed.
# Run from the repository root:  demos/d5.sh
# It uses only the made-up priced job (`plumbline priced`; no real job data) and a
# temporary folder. Every price, rate and supplier in it is made up. Every step fails the
# script if the program does not do what the step says. The figures expected are worked by
# hand in the comments, each step from the figure the step before shows.
#
# The job, from the sample roof: south plane 419.2627 SF, east and west 251.5576 SF each,
# shed 98.9545 SF (Assumed: its pitch is), headwall 12 LF (Assumed: it lies on the shed).
#   south  panels     419.2627 x 1.10 waste = 461.1890 SF x 1 lb/SF = 461.1890 lb x 9.00 = 4,150.70
#          underlay   419.2627 x 0.90                                                   =   377.34
#          fabricate  419.2627 / 100 SF/hr = 4.1926 hr x 60                             =   251.56
#          install    419.2627 / 25 SF/hr  = 16.7705 hr x 72                            = 1,207.48
#   east   panels     251.5576 x 1.10 = 276.7134 lb x 9.00                              = 2,490.42
#          underlay   251.5576 x 0.90                                                   =   226.40
#          fabricate  251.5576 / 100 = 2.5156 hr x 60                                   =   150.94
#          install    251.5576 / 25  = 10.0623 hr x 72                                  =   724.49
#   west   by a sub   251.5576 x 41.00                                                  = 10,313.86
#   shed   panels     98.9545 x 1.25 lb/SF = 123.6931 lb x 9.00                         = 1,113.24
#   headwall flashing 12 LF x 24 in = 24 SF x 1.25 = 30 lb x 9.00                       =   270.00
#          install    12 x 0.25 hr/LF = 3 hr x 72                                       =   216.00
#   job    foreman    5 days x 8 hr = 40 hr x 95                                        = 3,800.00
#   north  the sample's line, with no price and no class.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"

sheet() { "$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"; }
says() { # $1 what is being shown, $2 the text the sheet has to hold (spaces run together)
  grep -qF -- "$2" "$T/sheet.txt" || { echo "FAIL: $1: the sheet does not say: $2"; echo "---"; cat "$T/sheet.txt"; exit 1; }
  printf '    %s\n' "$2"
}
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | cut -c1-240
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
}
K='--status Known --source'

echo "== 1. The sheet: every line under its element, its amount in one of three columns =="
"$P" priced "$J"
"$P" check "$J"
sheet
says "a material line, in pounds from the material" "Standing seam panels 461.1890 lb \$9.00/lb \$4,150.70"
says "what it is made of" 'made of "Copper, 16 oz"'
says "a material line with its own price" "Underlayment 419.2627 SF \$0.90/SF \$377.34"
# Material: 4,150.70 + 377.34 + 2,490.42 + 226.40 + 1,113.24 + 270.00 = 8,628.10
# Labor column: labor 2,147.97 + fabrication 402.50 + supervision 3,800.00 = 6,350.47
# General column: the subcontract, 10,313.86
says "each column is the sum of the amounts shown in it" "column totals ~\$8,628.10 ~\$6,350.47 \$10,313.86"

echo
echo "== 2. In-house labor is a production rate =="
# 419.2627 SF at 25 SF to the hour: 16.7705 hours, shown as 16.77; x 72.00 = 1,207.476.
says "hours from the rate, then dollars from the hours" "Install panels 16.7705 hr \$72.00/hr \$1,207.48"
# The other way up: 12 LF at a quarter of an hour each is 3 hours. Assumed, like the headwall.
says "a rate in hours to the LF" "Install headwall flashing ~3.0000 hr \$72.00/hr ~\$216.00"
# Labor: 1,207.48 + 724.49 + 216.00 = 2,147.97; 16.7705 + 10.0623 + 3 = 29.8328 hours.
says "the labor row, with its hours" "Labor ~\$2,147.97 3 lines ~29.8328 hr"
says "how much of it rests on an assumption" "\$216.00 of it rests on something assumed"

echo
echo "== 3. Labor toggles per line =="
"$P" labor "$J" plane-0003 off --line "Install panels" >/dev/null
"$P" check "$J"
sheet
# Labor 2,147.97 - 1,207.48 = 940.49; hours 29.8328 - 16.7705 = 13.0623.
says "the labor row without it" "Labor ~\$940.49 2 lines ~13.0623 hr"
# Job cost 25,292.43 - 1,207.48 = 24,084.95.
says "the job cost without it" "Job cost ~\$24,084.95 12 lines"
says "the line is still on the sheet, marked off, with what it would come to" "Install panels 16.7705 hr \$72.00/hr off (\$1,207.48)"
says "and the recap says so" 'switched off, on the sheet and out of every total: plane-0003: "Install panels" ($1,207.48)'
"$P" labor "$J" plane-0003 on --line "Install panels" >/dev/null
sheet
says "on again, to the cent" "Labor ~\$2,147.97 3 lines ~29.8328 hr"
says "" "Job cost ~\$25,292.43 13 lines"

echo
echo "== 4. Subcontract is a priced line =="
says "a quantity and a price, in the general column" "Standing seam roofing, by subcontractor [Subcontract] 251.5576 SF \$41.00/SF \$10,313.86"
says "its own row of the recap" "Subcontract \$10,313.86 1 line"
refused "is Subcontract and carries a production rate" "$P" class "$J" plane-0003 subcontract --line "Install panels"

echo
echo "== 5. Fabrication and supervision are lines of their own =="
# Fabrication: 4.1926 hr x 60 = 251.56 and 2.5156 hr x 60 = 150.94: 402.50, 6.7082 hours.
says "shop work" "Fabricate panels [Fabrication] 4.1926 hr \$60.00/hr \$251.56"
says "its row" "Fabrication \$402.50 2 lines 6.7082 hr"
says "the foreman, on the job itself" "Foreman, days on site [Supervision] 40.0000 hr \$95.00/hr \$3,800.00"
says "its row" "Supervision \$3,800.00 1 line 40.0000 hr"

echo
echo "== 6. General conditions, typed by hand =="
says "nothing in general conditions is not \$0.00" "General conditions none"
"$P" line "$J" job-0001 "Dumpster" 2 --class general
"$P" check "$J"
sheet
says "typed and not yet priced: named, not counted as nothing" '? job-0001: "Dumpster": the line has no price (no price entered)'
"$P" set "$J" job-0001 unit_cost 650 --line "Dumpster" $K "made-up price" >/dev/null
"$P" line "$J" job-0001 "Permit" 1 --class general >/dev/null
"$P" set "$J" job-0001 unit_cost '$1,850' --line "Permit" $K "made-up price" >/dev/null
"$P" check "$J"
sheet
# 2 x 650.00 = 1,300.00; 1 x 1,850.00; together 3,150.00.
says "the typed lines" "Dumpster [General conditions] 2.00 EA \$650.00/EA \$1,300.00"
says "their row" "General conditions \$3,150.00 2 lines"
# General column: 10,313.86 + 3,150.00 = 13,463.86.
says "the general column" "column totals ~\$8,628.10 ~\$6,350.47 \$13,463.86"

echo
echo "== 7. The recap =="
# 8,628.10 + 2,147.97 + 402.50 + 3,800.00 + 10,313.86 + 3,150.00 = 28,442.43
says "the job cost is the six rows added up" "Job cost ~\$28,442.43 15 lines"
says "a line in no class is named and left out" '? plane-0005: "Copper standing seam, installed": the line has no cost type, so it is in no row of the recap'
says "with no overhead entered there is no price, and it says why" "PRICE no figure ? overhead has not been entered; enter a percentage, 0 if there is none"

echo
echo "== 8. O&P is editable =="
"$P" set "$J" job-0001 overhead 10 $K "made-up markup" >/dev/null
sheet
# 28,442.43 x 10 % = 2,844.243 -> 2,844.24. Profit is still not entered.
says "overhead on the job cost" "Overhead at 10.00 % ~\$2,844.24"
says "still no price: no profit" "PRICE no figure ? profit is not known (not entered)"
"$P" set "$J" job-0001 profit 8 $K "made-up markup" >/dev/null
sheet
# (28,442.43 + 2,844.24) x 8 % = 31,286.67 x 0.08 = 2,502.9336 -> 2,502.93.
# Price: 28,442.43 + 2,844.24 + 2,502.93 = 33,789.60.
says "profit on the job cost and the overhead together" "Profit at 8.00 % ~\$2,502.93"
says "the price" "PRICE ~\$33,789.60"
"$P" set "$J" job-0001 overhead 12 $K "made-up markup" >/dev/null
"$P" check "$J"
sheet
# 12 %: 3,413.0916 -> 3,413.09. Profit: 31,855.52 x 0.08 = 2,548.4416 -> 2,548.44.
# Price: 28,442.43 + 3,413.09 + 2,548.44 = 34,403.96.
says "changed" "Overhead at 12.00 % ~\$3,413.09"
says "" "Profit at 8.00 % ~\$2,548.44"
says "" "PRICE ~\$34,403.96"

echo
echo "== 9. Three suppliers, side by side =="
# Harbor Metals: 461.1890 x 8.80 = 4,058.46; 276.7134 x 8.80 = 2,435.08; 123.6931 x 8.90 =
# 1,100.87; 30 x 8.90 = 267.00: 7,861.41.   Ridge Supply: 4,266.00 + 2,559.60 + 1,150.35 +
# 279.00 = 8,254.95.   Tinners Yard quoted the 16 oz only: 4,196.82 + 2,518.09 = 6,714.91.
says "the columns" "generic Harbor Metals Ridge Supply Tinners Yard"
says "a line at all four" "south: Standing seam panels \$4,150.70 \$4,058.46 \$4,266.00 \$4,196.82"
says "a line one supplier has not quoted" "shed: Standing seam panels ~\$1,113.24 ~\$1,100.87 ~\$1,150.35 no quote"
# A total that leaves lines out is flagged for that alone: Tinners Yard's is over two of four.
says "the totals" "total ~\$8,024.36 ~\$7,861.41 ~\$8,254.95 ~\$6,714.91"
says "how many lines each total is over" "lines in the total 4 of 4 4 of 4 4 of 4 2 of 4"
says "and what the short one leaves out" '? Tinners Yard leaves out plane-0017: "Standing seam panels": Tinners Yard has not quoted "Copper, 20 oz"'

echo
echo "== 10. Changing an entity's material reprices every line made of it, and the recap =="
cp "$J" "$T/before.json"
"$P" set "$J" plane-0003 material "Copper, 20 oz" $K "made-up revision" >/dev/null
"$P" check "$J"
sheet
# South panels: 461.1890 SF x 1.25 lb/SF = 576.48625 -> 576.4863 lb x 9.00 = 5,188.3767 -> 5,188.38.
# It was 4,150.70: 1,037.68 more.
says "the panels, in more pounds" "Standing seam panels 576.4863 lb \$9.00/lb \$5,188.38"
says "now made of" 'made of "Copper, 20 oz"'
says "the lines not made of it have not moved" "Underlayment 419.2627 SF \$0.90/SF \$377.34"
says "" "Install panels 16.7705 hr \$72.00/hr \$1,207.48"
# Material 8,628.10 + 1,037.68 = 9,665.78. Job cost 28,442.43 + 1,037.68 = 29,480.11.
says "the material row" "Material ~\$9,665.78 6 lines"
says "the job cost" "Job cost ~\$29,480.11 15 lines"
# Overhead 12 %: 3,537.6132 -> 3,537.61. Profit: 33,017.72 x 0.08 = 2,641.4176 -> 2,641.42.
# Price: 29,480.11 + 3,537.61 + 2,641.42 = 35,659.14.
says "the price" "PRICE ~\$35,659.14"
# Its quotes are the new material's: Harbor 576.4863 x 8.90 = 5,130.73; Tinners has not quoted 20 oz.
says "and the suppliers" "south: Standing seam panels \$5,188.38 \$5,130.73 \$5,361.32 no quote"
python3 - "$T/before.json" "$J" <<'PY'
import sys, json
a = {e["path"]: e for e in json.load(open(sys.argv[1]))["data"]}
b = {e["path"]: e for e in json.load(open(sys.argv[2]))["data"]}
changed = sorted(p for p in a if a[p] != b[p])
print("    elements changed in the file:", ", ".join(changed))
if changed != ["plane-0003"]:
    print("FAIL: something other than the plane changed"); sys.exit(1)
PY

echo
echo "== What is refused, and what is flagged =="
MAT="$(python3 - "$J" <<'PY'
import sys, json
print(next(e for e in json.load(open(sys.argv[1]))["data"] if e["path"] == "job-0001")["children"]["Copper, 20 oz"])
PY
)"
refused "is not in hours" "$P" class "$J" plane-0003 labor --line "Underlayment"
refused "only a Labor line" "$P" labor "$J" plane-0003 off --line "Fabricate panels"
refused "a price does not carry from one unit to another" "$P" set "$J" "$MAT" quote '$9.40/SF' --supplier "Tinners Yard" $K "x"
refused "its price is the material's" "$P" set "$J" plane-0004 unit_cost 9 --line "Standing seam panels" $K "x"
refused "is not a number" "$P" set "$J" job-0001 overhead ten $K "x"
# A material the job does not have is not refused at the door (the name may be on its way);
# the lines made of it have no figure, say why, and are left out of every total by name.
"$P" set "$J" plane-0004 material "Zinc, 0.7 mm" $K "made-up revision" >/dev/null
sheet
says "a material the job does not have" '? the element is made of "Zinc, 0.7 mm", and the job has no material of that name'

echo
echo "PASS: the sheet, production rates, the labor switch, subcontract, fabrication, supervision, typed general conditions, O&P, the recap, three suppliers and a change of material"
