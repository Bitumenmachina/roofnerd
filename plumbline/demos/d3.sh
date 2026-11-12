#!/usr/bin/env bash
# D3 demonstration: derived conditions and stretch-out.
# Run from the repository root:  demos/d3.sh
# It uses only the sample job (made up; no real job data) and a temporary folder.
# Every step fails the script if the program does not do what the step says.
# The figures expected are worked by hand in the comments.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT

# One figure on one element, as `show` prints it. $2 element, $3 the row's label.
figure() { "$P" show "$1" | awk -v el="$2" '$1==el{on=1;next} on&&/^ *[a-z]+-[0-9]+ /{on=0} on' | grep -F -- " $3 " | head -1 | sed -E "s/.*$(printf '%s' "$3" | sed 's/[][\.*^$/]/\\&/g') +//"; }
expect() { # $1 file, $2 element, $3 label, $4 expected text
  local got; got="$(figure "$1" "$2" "$3")"
  printf '    %-34s %s\n' "$3" "$got"
  [ "$got" = "$4" ] || { echo "FAIL: expected \"$4\""; exit 1; }
}
# Which lines' figures differ between two saves of one element, and which other elements changed.
line_changes() {
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys, json
before, after, element, expected = sys.argv[1:]
a = {e["path"]: e for e in json.load(open(before))["data"]}
b = {e["path"]: e for e in json.load(open(after))["data"]}
others = [p for p in a if p != element and a[p] != b.get(p)] + [p for p in b if p not in a]
qa, qb = a[element]["attributes"].get("pl::line_quantities", {}), b[element]["attributes"].get("pl::line_quantities", {})
lines = [k for k in sorted(set(qa) | set(qb)) if qa.get(k) != qb.get(k)]
print("    elements changed besides", element + ":", ", ".join(others) or "none")
print("    lines on it whose figures changed:", ", ".join(lines) or "none")
if others or lines != [l for l in expected.split("|") if l]:
    print("FAIL: expected only", expected); sys.exit(1)
PY
}
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | cut -c1-230
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
}
PARAPET=parapet-0024
K=(--status Known --source "sample detail 3/A-501")

"$P" sample "$T/job.json" >/dev/null
echo "== 0. The condition, as it sits in the job =="
"$P" show "$T/job.json" | grep -A5 "Parapet with copper coping  \[assembly\]" | sed 's/^/  /'

echo
echo "== 1. One parapet, one condition, four lines; nothing assumed for them =="
"$P" apply "$T/job.json" $PARAPET "Parapet with copper coping" --source "sample finish schedule" | sed 's/^/  /'
"$P" check "$T/job.json" >/dev/null
# The parapet is 20 LF. Its height is not on the sample plan and no stretch-out, spacing,
# waste or sheet size has been entered, so three lines have no number. The cleat needs
# only the length and the condition's own rule, a cleat each side: 20 x 2 = 40 LF.
expect "$T/job.json" $PARAPET "quantity: Copper wall flashing" "unknown"
expect "$T/job.json" $PARAPET "quantity: Copper coping cap" "unknown"
expect "$T/job.json" $PARAPET "to order: Copper coping cap" "unknown"
expect "$T/job.json" $PARAPET "quantity: Continuous cleat" "40 LF"
expect "$T/job.json" $PARAPET "quantity: Cleat fasteners" "unknown"
if "$P" show "$T/job.json" | grep -E "^ +~ $PARAPET .*(stretch-out|spacing|waste|order factor|quantity|with waste|to order)"; then echo "FAIL: something on the new lines is Assumed"; exit 1; fi
echo "  PASS: four lines, each with its formula; three wait on what the detail gives"

echo
echo "== 2. Enter what the detail gives, each as it is written =="
"$P" set "$T/job.json" $PARAPET height "30 in" "${K[@]}" | head -1 | sed 's/^/  /'
"$P" set "$T/job.json" $PARAPET stretch_out '19 1/2"' --line "Copper coping cap" "${K[@]}" | head -1 | sed 's/^/  /'
"$P" set "$T/job.json" $PARAPET spacing "12 in" --line "Cleat fasteners" "${K[@]}" | head -1 | sed 's/^/  /'
# Wall flashing, a height on its LF:  20 LF x 30 in / 12        = 50 SF
# Coping cap:                         20 LF x 19 1/2 in / 12    = 32.5 SF
# Cleat, both sides:                  20 LF x 2                 = 40 LF
# Fasteners at 12 in, both sides:     20 LF x 2 / (12 in / 12)  = 40
expect "$T/job.json" $PARAPET "quantity: Copper wall flashing" "50 SF"
expect "$T/job.json" $PARAPET "quantity: Copper coping cap" "32.5 SF"
expect "$T/job.json" $PARAPET "quantity: Continuous cleat" "40 LF"
expect "$T/job.json" $PARAPET "quantity: Cleat fasteners" "40 EA"
echo "  PASS"

echo
echo "== 3. The sheet-metal chain on the coping line, each step with its formula =="
expect "$T/job.json" $PARAPET "with waste: Copper coping cap" "unknown"   # no waste entered yet: no number, not the quantity again
"$P" set "$T/job.json" $PARAPET waste "10%" --line "Copper coping cap" --status Assumed --source "shop rule of thumb; confirm" | head -1 | sed 's/^/  /'
"$P" set "$T/job.json" $PARAPET order "30 SF/sheet" --line "Copper coping cap" --status Known --source "3 ft x 10 ft sheet" | head -1 | sed 's/^/  /'
# 32.5 SF + 10% = 35.75 SF.  35.75 / 30 SF to the sheet = 1.19, so 2 sheets.
expect "$T/job.json" $PARAPET "with waste: Copper coping cap" "35.75 SF"
expect "$T/job.json" $PARAPET "to order: Copper coping cap" "2 sheet"
python3 - "$T/job.json" <<'PY'
import sys, json
for e in json.load(open(sys.argv[1]))["data"]:
    if e["path"] == "parapet-0024":
        for step, d in e["attributes"]["pl::line_quantities"]["Copper coping cap"].items():
            # A step with no figure (the line's extension, while it has no price) says why.
            value = "none" if d["value"] is None else d["value"]
            why = f'   ({d["note"]})' if d.get("note") else ""
            print(f'    as saved:  {step:10} {value:>8} {d["unit"]:6} {d["status"]:8} = {d["formula"]}{why}')
PY
cp "$T/job.json" "$T/lb.json"
"$P" set "$T/lb.json" $PARAPET order "1.25 lb/SF" --line "Copper coping cap" --status Known --source "20 oz copper weighs 1.25 lb to the SF" | head -1 | sed 's/^/  /'
# Or by weight: 35.75 SF x 1.25 lb = 44.6875 lb. A weight is not rounded up.
expect "$T/lb.json" $PARAPET "to order: Copper coping cap" "44.6875 lb"
# The waste is Assumed, so everything after it is flagged Assumed; the quantity before it is not.
"$P" show "$T/job.json" | grep -E "^ +(=)? +. ?(quantity|with waste|to order): Copper coping cap" | sed 's/^ */    flags:  /'
"$P" show "$T/job.json" | grep -qE "^ += +quantity: Copper coping cap" || { echo "FAIL: the quantity should be Known"; exit 1; }
"$P" show "$T/job.json" | grep -qE "^ += ~ to order: Copper coping cap" || { echo "FAIL: the order figure should be Assumed"; exit 1; }
cp "$T/job.json" "$T/gone.json"
"$P" set "$T/gone.json" $PARAPET stretch_out unknown --line "Copper coping cap" --status Unknown --source "detail withdrawn" | head -1 | sed 's/^/  /'
expect "$T/gone.json" $PARAPET "quantity: Copper coping cap" "unknown"
expect "$T/gone.json" $PARAPET "with waste: Copper coping cap" "unknown"
expect "$T/gone.json" $PARAPET "to order: Copper coping cap" "unknown"
echo "  PASS: LF x stretch-out / 12 to SF, plus waste, to sheets or pounds; an Unknown leaves no number after it"

echo
echo "== 4. Edit on the line: that line changes and nothing else does =="
cp "$T/job.json" "$T/so.json"
"$P" set "$T/so.json" $PARAPET stretch_out "24 in" --line "Copper coping cap" --status Known --source "detail 3/A-501 rev B" | head -1 | sed 's/^/  /'
# 20 LF x 24 in / 12 = 40 SF; + 10% = 44 SF; / 30 = 1.47, so still 2 sheets.
expect "$T/so.json" $PARAPET "quantity: Copper coping cap" "40 SF"
expect "$T/so.json" $PARAPET "with waste: Copper coping cap" "44 SF"
line_changes "$T/job.json" "$T/so.json" $PARAPET "Copper coping cap"
cp "$T/job.json" "$T/one.json"
"$P" set "$T/one.json" $PARAPET quantity "plan_length" --line "Continuous cleat" --status Known --source "detail 3/A-501 rev B: cleat on the outside face only" | sed 's/^/  /' | head -2
expect "$T/one.json" $PARAPET "quantity: Continuous cleat" "20 LF"
line_changes "$T/job.json" "$T/one.json" $PARAPET "Continuous cleat"
cp "$T/job.json" "$T/turn.json"
"$P" set "$T/turn.json" $PARAPET quantity "plan_length * (height + turn_out)" --with "turn_out=8 in" --line "Copper wall flashing" "${K[@]}" | sed 's/^/  /' | head -2
# The flashing turned out 8 in onto the roof: 20 LF x (30 in + 8 in) / 12 = 63.3333 SF.
expect "$T/turn.json" $PARAPET "quantity: Copper wall flashing" "63.3333 SF"
line_changes "$T/job.json" "$T/turn.json" $PARAPET "Copper wall flashing"
echo "  a formula that does not make sense is refused, and says why:"
refused "adds what cannot be added" "$P" set "$T/turn.json" $PARAPET quantity "plan_length + face_area" --line "Copper wall flashing" "${K[@]}"
refused "is not a figure of this thing" "$P" set "$T/turn.json" $PARAPET quantity "plan_length * hieght" --line "Copper wall flashing" "${K[@]}"
refused "has no unit" "$P" set "$T/turn.json" $PARAPET spacing "12" --line "Cleat fasteners" "${K[@]}"
for f in so one turn; do "$P" check "$T/$f.json" >/dev/null; done
echo "  PASS"

echo
echo "== 5. The brief's four stretch-outs, each on a line of the sample roof =="
cp "$T/job.json" "$T/four.json"
# Eave 12 in on the south eave, 40 LF:            40 x 12 / 12      = 40 SF
# Step flashing 8 in up the porch rake, 8.2462 LF: 8.2462 x 8 / 12  = 5.4975 SF
# Valley 36 in. The sample roof has no valley; the same arithmetic on a hip, 22.5 LF:
#                                                  22.5 x 36 / 12   = 67.5 SF
# Headwall 24 in on the porch headwall, 12 LF:    12 x 24 / 12      = 24 SF
for row in "eave-0007|12 in|40 SF" "rake-0019|8 in|5.4975 SF" "hip-0012|36 in|67.5 SF"; do
  IFS='|' read -r el so want <<<"$row"
  "$P" apply "$T/four.json" "$el" "Copper flashing" --source "sample finish schedule" >/dev/null
  "$P" set "$T/four.json" "$el" stretch_out "$so" --status Known --source "the brief" >/dev/null
  printf '  %-10s stretch-out %-6s' "$el" "$so"; expect "$T/four.json" "$el" "quantity: Copper flashing" "$want"
done
printf '  %-10s stretch-out %-6s' "headwall-0021" "24 in"; expect "$T/four.json" headwall-0021 "quantity: Copper headwall flashing" "24 SF"
"$P" check "$T/four.json" >/dev/null
echo "  PASS (the arithmetic; the owner's estimate itself is not in refs/, so its reproduction is pending)"

echo
echo "== 6. Nobody converted anything =="
echo "  entered above as written: 30 in, 19 1/2\", 12 in, 24 in, 8 in, 36 in, 10%, 30 SF/sheet, 1.25 lb/SF"
grep -c '"unit": "in"' "$T/four.json" | sed 's/^/  values saved in inches: /'
echo "  PASS"
