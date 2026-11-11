#!/usr/bin/env bash
# D4 demonstration: macro to micro. A job priced from allowances ($/SF, $/LF), the same
# elements priced from their cost lines, the detail rolled back up to $/SF and $/LF, and
# the two reconciled.
# Run from the repository root:  demos/d4.sh
# It uses only the sample job (made up; no real job data) and a temporary folder. Every
# price in it is made up and is stated here. Every step fails the script if the program
# does not do what the step says. The figures expected are worked by hand in the comments.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"

says() { # $1 what is being shown, $2 the text the price printout has to hold
  grep -qF -- "$2" "$T/price.txt" || { echo "FAIL: $1: the printout does not say: $2"; echo "---"; cat "$T/price.txt"; exit 1; }
  printf '    %s\n' "$2"
}
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | cut -c1-230
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
}
price() { "$P" price "$J" > "$T/price.txt"; }

echo "== The job: the sample house's roof. Nothing in it has a price yet =="
"$P" sample "$J"
"$P" check "$J"
price
says "no allowance, no detail, and neither is \$0" "allowances   none: no element has an allowance that comes to a figure"
says "" "detail       none: no element has a detail that is whole"

echo
echo "== 1. Design phase: priced from allowances alone =="
# Made-up budget rates: $28.00 a SF of roof, $40.00 a LF of headwall flashing.
# South plane: 419.2627 SF x 28.00 = 11,739.3556            -> $11,739.36
# East plane:  251.5576 SF x 28.00 =  7,043.6128            ->  $7,043.61
# Shed:         98.9545 SF x 28.00 =  2,770.726  (Assumed)  ->  $2,770.73
# Headwall:     12 LF      x 40.00 =    480.00   (Assumed: it lies on the shed)
for plane in plane-0003 plane-0004 plane-0017; do
  "$P" set "$J" "$plane" allowance '$28.00/SF' --status Known --source "made-up budget rate" >/dev/null
done
"$P" set "$J" headwall-0021 allowance 40 --status Known --source "made-up budget rate" >/dev/null
"$P" check "$J"
price
says "the south plane" "allowance   \$11,739.36 at \$28.00 per SF"
says "the east plane" "allowance   \$7,043.61 at \$28.00 per SF"
# 11,739.36 + 7,043.61 + 2,770.73 + 480.00 = 22,033.70 over four elements.
says "the total" "allowances   ~\$22,033.70 over 4 elements"
# The north and west planes have no allowance. Each has a cost line, so each is listed,
# with "none entered"; the 17 elements with neither are counted by kind.
says "what has no allowance is said" "allowance   none entered"
says "what is not priced at all is counted" "not priced at all:"

echo
echo "== 2. An assumed input shows in the money =="
# The shed's pitch is Assumed (3:12, not dimensioned), so its area is, and its price is.
# The headwall lies on the shed, so its length is no surer. 2,770.73 + 480.00 = 3,250.73.
says "the shed's allowance is flagged" "allowance   ~\$2,770.73 at \$28.00 per SF"
says "the total says how much of itself is assumed" "of which \$3,250.73 rests on something assumed"

echo
echo "== 3. Detail the same entities: their cost lines priced =="
# Made-up quotes: $31.50 a SF for the standing seam; $18.25 a SF for the headwall flashing.
# South plane: 419.2627 SF x 31.50 = 13,206.77505 -> $13,206.78
# Shed:         98.9545 SF x 31.50 =  3,117.06675 ->  $3,117.07 (Assumed)
# Headwall: 12 LF x 24 in / 12 = 24 SF x 18.25   ->    $438.00 (Assumed)
"$P" set "$J" plane-0003 unit_cost 31.50 --status Known --source "made-up quote" >/dev/null
"$P" set "$J" plane-0017 unit_cost '$31.50/SF' --status Known --source "made-up quote" >/dev/null
"$P" set "$J" headwall-0021 unit_cost 18.25 --status Known --source "made-up quote" >/dev/null
"$P" check "$J"
price

echo
echo "== 4. Rolled back up to \$/SF and \$/LF =="
# South plane: 13,206.78 / 419.2627 SF = $31.50 a SF.
# Headwall: its line is priced by the SF; over the 12 LF its allowance is in,
# 438.00 / 12 = $36.50 a LF.
says "the south plane's detail and what it comes to a SF" "detail      \$13,206.78, which is \$31.50 per SF"
says "the headwall's detail, a SF line, as a rate per LF" "detail      ~\$438.00, which is ~\$36.50 per LF"

echo
echo "== 5. Reconciled =="
# South plane: 13,206.78 - 11,739.36 = 1,467.42 over; 1,467.42 / 11,739.36 = 12.50 %.
# Shed:         3,117.07 -  2,770.73 =   346.34 over; 346.34 / 2,770.73 = 12.4999... %.
# Headwall:       438.00 -    480.00 =    42.00 under; 42 / 480 = 8.75 %.
says "south plane" "the detail is \$1,467.42 over the allowance (12.50 %)"
says "shed" "the detail is ~\$346.34 over the allowance (~12.50 %)"
says "headwall" "the detail is ~\$42.00 under the allowance (~-8.75 %)"
# Each total names what it does not cover at all.
says "the allowance total names the elements with lines and no allowance" "? no allowance entered on 2 elements with cost lines: plane-0005, plane-0006"
# The east plane has an allowance and a line with no price: its detail is not whole. It
# has no detail figure and is not rolled up as if it were whole.
says "an element whose detail is not whole says so" "the detail is not whole: 1 of its 1 lines cannot be extended. \"Copper standing seam, installed\": the line has no price (no price entered)"
# Detail total: 13,206.78 + 3,117.07 + 438.00 = 16,761.85 over three elements, of which
# 3,117.07 + 438.00 = 3,555.07 is assumed. Three planes are left out, each named.
says "the detail total" "detail       ~\$16,761.85 over 3 elements"
says "" "of which \$3,555.07 rests on something assumed"
says "what the detail total leaves out is named" "? left out: plane-0004: the detail is not whole"
# Like for like, over the three elements that have both: allowances 11,739.36 + 2,770.73
# + 480.00 = 14,990.09; detail 16,761.85; 1,771.76 over; 1,771.76 / 14,990.09 = 11.82 %.
says "like for like" "like for like, over the 3 elements that have both: allowances ~\$14,990.09, detail ~\$16,761.85: the detail is ~\$1,771.76 over (~11.82 %)"

echo
echo "== 6. The geometry still drives both =="
# Make the south plane 9:12. Its plan area is 375 SF and its slope factor 1.25: 468.75 SF.
# Allowance 468.75 x 28.00 = 13,125.00. Detail 468.75 x 31.50 = 14,765.625 -> 14,765.63.
# The rate it rolls up to is still $31.50 a SF.
cp "$J" "$T/before.json"
"$P" set "$J" plane-0003 pitch 9:12 --status Known --source "made-up revision" >/dev/null
"$P" check "$J"
price
says "the allowance moved" "allowance   \$13,125.00 at \$28.00 per SF"
says "the detail moved" "detail      \$14,765.63, which is \$31.50 per SF"
# In the file: the plane, and the lines that lie on it (their lengths follow its slope).
python3 - "$T/before.json" "$J" <<'PY'
import sys, json
a = {e["path"]: e for e in json.load(open(sys.argv[1]))["data"]}
b = {e["path"]: e for e in json.load(open(sys.argv[2]))["data"]}
changed = sorted(p for p in a if a[p] != b[p])
print("    elements changed in the file:", ", ".join(changed))
planes = lambda p: a[p]["attributes"].get("pl::run", {}).get("planes", [])
stray = [p for p in changed if p != "plane-0003" and "plane-0003" not in planes(p)]
if "plane-0003" not in changed or stray:
    print("FAIL: something changed that is not the plane or a line on it:", stray); sys.exit(1)
priced = [p for p in changed if a[p]["attributes"].get("pl::priced") != b[p]["attributes"].get("pl::priced")]
print("    of those, the ones whose price changed:", ", ".join(priced))
if priced != ["plane-0003"]:
    print("FAIL: expected only the plane's price to change"); sys.exit(1)
PY

echo
echo "== What is refused =="
refused "a price does not carry from one unit to another" "$P" set "$J" plane-0003 allowance '$28/LF' --status Known --source "x"
refused "cannot carry an allowance" "$P" set "$J" roof-0002 allowance 5 --status Known --source "x"
refused "a price does not carry from one unit to another" "$P" set "$J" headwall-0021 unit_cost '$18.25/LF' --status Known --source "x"
refused "is below zero" "$P" set "$J" plane-0003 unit_cost -3 --status Known --source "x"

echo
echo "PASS: priced from allowances, priced from its lines, rolled back up, reconciled, and moved by the geometry"
