#!/usr/bin/env bash
# Work on part of a thing, typed as a measure, and a partition taken down as a scope
# (call 159). From an empty job, a house as it stands (the house of demos/reno.sh), its
# walls marked existing and its partition to demolish: a patch of siding typed as "60 SF"
# with how sure it is and where it came from, refused without them, changed, measured;
# a measure worked in a formula; the starter assembly "Take down a partition" given to the
# partition, its eight lines to the figures worked by hand.
# Run from the repository root:  demos/partwork.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by
# hand in STATUS.md ("Work on part of a thing, and a partition taken down as a scope").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
est() { "$P" estimate "$J" | tr -s ' ' > "$T/est.txt"; }
unchanged() { cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }; }

echo "== 1. The house as it stands: its walls existing, the partition to demolish =="
"$P" new "$J" "Partwork (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" mark "$J" "$SOUTH" "$EAST" "$NORTH" "$WEST" "$PART" existing "${K[@]}" >/dev/null
"$P" mark "$J" "$PART" demolish "${K[@]}" >/dev/null

echo
echo "== 2. 60 SF of siding patched, typed as a measure: it says how sure and where from =="
cp "$J" "$T/before.json"
refused "$P" line "$J" "$SOUTH" "Patch and match siding" "60 SF" --division 7
says "$T/out.txt" "is a measure typed on the line, so --status is required"
unchanged
refused "$P" line "$J" "$SOUTH" "Patch and match siding" "60 SF" --status Assumed --division 7
says "$T/out.txt" "so --source is required"
unchanged
refused "$P" line "$J" "$SOUTH" "Patch and match siding" "60 SF" --status Unknown --source "walked it" --division 7
says "$T/out.txt" "Known or Assumed"
unchanged
refused "$P" line "$J" "$SOUTH" "Patch and match siding" "60 CF" --status Assumed --source "walked it"
unchanged
"$P" line "$J" "$SOUTH" "Patch and match siding" "60 SF" --cost-type subcontract --status Assumed --source "walked it" --division 7 > "$T/out.txt"
says "$T/out.txt" "60 SF (Assumed, walked it)"
"$P" set "$J" "$SOUTH" unit_cost '$14/SF' --line "Patch and match siding" --status Known --source "made-up price" >/dev/null
"$P" line "$J" "$SOUTH" "Paint the wall" net_area --cost-type subcontract --division 9 >/dev/null
"$P" set "$J" "$SOUTH" unit_cost '$1.20/SF' --line "Paint the wall" --status Known --source "made-up price" >/dev/null
est
# 60 x 14 = $840.00; the wall is 24 x 9 = 216 SF, painted at $1.20: $259.20.
says "$T/est.txt" "Patch and match siding [Subcontract] ~60.00 SF \$14.00/SF ~\$840.00"
says "$T/est.txt" "Paint the wall [Subcontract] 216.00 SF \$1.20/SF \$259.20"
"$P" set "$J" "$SOUTH" measured "72 SF" --line "Patch and match siding" --status Assumed --source "walked it again" > "$T/out.txt"
says "$T/out.txt" "set $SOUTH measured = 72 SF (Assumed)"
est
# 72 x 14 = $1,008.00.
says "$T/est.txt" "Patch and match siding [Subcontract] ~72.00 SF \$14.00/SF ~\$1,008.00"
"$P" measure "$J" "$SOUTH" measured same --line "Patch and match siding" --source "made-up tape" > "$T/out.txt"
says "$T/out.txt" "is 72 SF, found as drawn"
est
says "$T/est.txt" "Patch and match siding [Subcontract] 72.00 SF \$14.00/SF \$1,008.00"
"$P" show "$J" > "$T/show.txt"
says "$T/show.txt" "            measured: Patch and match siding 72 SF"

echo
echo "== 3. A measure in a formula is what its unit says: measured * 2 is in SF =="
"$P" line "$J" "$SOUTH" "Second patch" 1 --cost-type subcontract >/dev/null
"$P" set "$J" "$SOUTH" quantity "measured * 2" --with "measured=72 SF" --line "Second patch" --status Assumed --source "walked it" > "$T/out.txt"
says "$T/out.txt" "with measured = 72 SF (Assumed)"
says "$T/out.txt" "the line now comes out in SF; it came out in EA"
est
# 72 x 2 = 144 SF, and no price on it yet.
says "$T/est.txt" "Second patch [Subcontract] ~144.00 SF no price no figure"

echo
echo "== 4. The partition given \"Take down a partition\": eight lines, 16 ft long, 9 ft tall, 0.5 ft thick =="
"$P" import assemblies "$J" starter > "$T/out.txt"
says "$T/out.txt" "Take down a partition: added, with 8 lines"
"$P" apply "$J" "$PART" "Take down a partition" --source "the starter library" > "$T/out.txt"
says "$T/out.txt" "$PART is now \"Take down a partition\" and has gained 8 cost lines:"
"$P" set "$J" "$PART" unit_cost '$2.50/SF' --line "Take down the partition" --status Known --source "made-up price" >/dev/null
"$P" set "$J" "$PART" unit_cost '$85/CY' --line "Haul off and dump" --status Known --source "made-up price" >/dev/null
est
# Protect the floors: 2 x 16 x 4 = 128 SF.
says "$T/est.txt" "Protect the floors, both sides [General conditions] ~128.00 SF no price"
# Dust barrier: no figure until its length is given.
says "$T/est.txt" "Dust barrier [General conditions] no figure no price"
"$P" set "$J" "$PART" barrier "20 ft" --line "Dust barrier" --status Assumed --source "made-up: where the work is closed off" >/dev/null
est
# 20 x 9 = 180 SF.
says "$T/est.txt" "Dust barrier [General conditions] ~180.00 SF no price"
# Take down: the face, 16 x 9 = 144 SF, at $2.50 = $360.00.
says "$T/est.txt" "Take down the partition [Subcontract] 144.00 SF \$2.50/SF \$360.00"
# Haul off: 144 x 0.5 = 72 CF = 2.6667 CY in place, x 1.5 = 4 CY, at $85 = $340.00.
says "$T/est.txt" "Haul off and dump [Subcontract] ~4.00 CY \$85.00/CY ~\$340.00"
# Patches: 16 x 0.5 = 8 SF, the floor and the ceiling; one at each end of the wall.
says "$T/est.txt" "Patch the floor where it stood [Subcontract] 8.00 SF no price"
says "$T/est.txt" "Patch the ceiling where it stood [Subcontract] 8.00 SF no price"
says "$T/est.txt" "Patch the walls it met [Subcontract] ~2.00 EA no price"
# Shoring: a Labor line, off: in no total, and where the wall carries load it is switched on.
says "$T/est.txt" "Shore the floor above no figure no price off (no figure)"
says "$T/est.txt" "switched off, on the sheet and out of every total: $PART: \"Shore the floor above\""
refused "$P" labor "$J" "$PART" off --line "Take down the partition"
says "$T/out.txt" "only a Labor line can be"

echo
echo "== 5. The estimate's job cost: the four priced lines, the rest left out and said =="
# 1,008.00 + 259.20 + 360.00 + 340.00 = 1,967.20.
says "$T/est.txt" "Job cost ~\$1,967.20 4 lines"
says "$T/est.txt" "left out of the job cost, and of everything taken on it: 6 lines"
says "$T/est.txt" "\"Protect the floors, both sides\": the line has no price"

echo
echo "== 6. Saved and read back the same; a library written out and read in again =="
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "its figures agree with what is drawn, and saving it back writes the same file"
"$P" export assemblies "$J" "$T/lib.csv" >/dev/null
"$P" new "$T/second.json" "Second (not a real job)" >/dev/null
"$P" import assemblies "$T/second.json" "$T/lib.csv" > "$T/out.txt"
says "$T/out.txt" "Take down a partition: added, with 8 lines"
"$P" export assemblies "$T/second.json" "$T/lib2.csv" >/dev/null
cmp -s "$T/lib.csv" "$T/lib2.csv" || { echo "FAIL: the assembly library did not go out and come in again the same"; exit 1; }
echo "    the assemblies went out and came in again without loss"
echo
echo "  PASS: a measure typed on a line was refused without its status and source, held as an entered value, changed and measured Known, worked as what its unit says in a formula; the starter assembly \"Take down a partition\" gave 128, 180, 144, 4 CY, 8, 8 and 2 and left shoring off; the job cost came to \$1,967.20, to the figures worked by hand"
