#!/usr/bin/env bash
# Joists and rafters counted across a floor and a roof plane (call 165). From an empty job:
# a floor 24 ft by 16 ft with a line that names its joists, counted the short way (taken,
# Assumed, and said) and the long way (said); an L-shaped floor; a gable roof and a hip roof
# with a line that names the rafters, plane by plane and whole; a floor and a roof with no
# such line have none of these figures; the starter "Framed floor" counted; saved and read
# back the same.
# Run from the repository root:  demos/joists.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails the
# script if the program does not say what the step expects. The figures are worked by hand in
# STATUS.md ("Joists and rafters counted").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
# One thing's lines of `show`, spaces squeezed.
of() { "$P" show "$J" | tr -s ' ' | sed -n "/^ *$1 /,/^ *[a-z]*-[0-9]* /p" > "$T/of.txt"; }

echo "== A floor 24 ft by 16 ft, 'Floor joists, 2x10 at 16 in' on it: the short way is taken, Assumed, and said =="
"$P" new "$J" "Joists (not a real job)" >/dev/null
GROUND="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
UPPER="$(made "$("$P" level "$J" upper --elevation "9 ft" --height "9 ft" "${K[@]}")")"
FT="$(made "$("$P" type "$J" floor "floor 10 in" --thickness "10 in" "${K[@]}")")"
FLOOR="$(made "$("$P" floor "$J" "$GROUND" "$FT" --points "0,0 24,0 24,16 0,16" "${K[@]}")")"
BARE="$(made "$("$P" floor "$J" "$UPPER" "$FT" --points "0,0 24,0 24,16 0,16" "${K[@]}")")"
"$P" line "$J" "$FLOOR" "Floor joists, 2x10 at 16 in" joists --cost-type material >/dev/null
"$P" line "$J" "$FLOOR" "Rim joist" rim_length --cost-type material >/dev/null
of "$FLOOR"
# 24 / 1.3333 + 1 = 19 joists of 16 ft: 304 LF; the rim 2 x 24 = 48 LF.
says "$T/of.txt" "= ~ joists 19 EA"
says "$T/of.txt" "= ~ joist length 304 LF"
says "$T/of.txt" "= ~ joist longest 16 ft"
says "$T/of.txt" "= ~ rim length 48 LF"
says "$T/of.txt" "= ~ quantity: Floor joists, 2x10 at 16 in 19 EA"
"$P" show "$J" | tr -s ' ' > "$T/show.txt"
says "$T/show.txt" "floor 1 joists: 19 EA (joists usually span the short way: say which way these run; blocking, hangers and doubled members are not in this count)"
# A floor with no line that names its joists has none of these figures.
of "$BARE"
lacks "$T/of.txt" "joists"
lacks "$T/of.txt" "rim length"

echo
echo "== Said the long way: 11 marks along the 16 ft and two edge joists: 13 joists of 24 ft, 312 LF; the rim 2 x 16 = 32 LF =="
refused "$P" set "$J" "$FLOOR" joists_span "sideways" "${K[@]}"
says "$T/out.txt" "is not a way for joists to run"
"$P" set "$J" "$FLOOR" joists_span "the long way" --status Known --source "the plan's framing sheet, made up" > "$T/out.txt"
says "$T/out.txt" "set $FLOOR joists span = the long way (Known)"
of "$FLOOR"
says "$T/of.txt" "= joists 13 EA"
says "$T/of.txt" "= joist length 312 LF"
says "$T/of.txt" "= joist longest 24 ft"
says "$T/of.txt" "= rim length 32 LF"
"$P" show "$J" | tr -s ' ' > "$T/show.txt"
lacks "$T/show.txt" "joists usually span the short way"
says "$T/show.txt" "joists span the long way"
"$P" set "$J" "$FLOOR" joists_span "short way" --status Known --source "the plan's framing sheet, made up" >/dev/null
of "$FLOOR"
says "$T/of.txt" "= joists 19 EA"

echo
echo "== An L: 0,0 24,0 24,10 12,10 12,16 0,16 (drawn 120 ft east of the first floor: a floor is not drawn in another, call 179 B6), joists north: 20 joists, 250 LF, longest 16 ft, rim 24 + 12 + 12 = 48 LF =="
EL="$(made "$("$P" floor "$J" "$UPPER" "$FT" --points "120,0 144,0 144,10 132,10 132,16 120,16" "${K[@]}")")"
"$P" line "$J" "$EL" "Floor joists, 2x10 at 16 in" joists --cost-type material >/dev/null
of "$EL"
says "$T/of.txt" "= ~ joists 20 EA"
says "$T/of.txt" "= ~ joist length 250 LF"
says "$T/of.txt" "= ~ joist longest 16 ft"
says "$T/of.txt" "= ~ rim length 48 LF"

echo
echo "== The same L with its notch on the other side (a mark falls on the notch's edge: it is one joist): 19 joists, 250 LF, rim 48 LF =="
# 17 marks (1.3333 k + 0.0625, k = 1 to 17) and 3 edge joists; mark 9 is at 12.0625, on the notch's edge: the 6 ft edge joist is a part of it.
# 8 x 10 + 9 x 16 + 10 + 16 = 250.
EM="$(made "$("$P" floor "$J" "$UPPER" "$FT" --points "40,0 64,0 64,16 52,16 52,10 40,10" "${K[@]}")")"
"$P" line "$J" "$EM" "Floor joists, 2x10 at 16 in" joists --cost-type material >/dev/null
of "$EM"
says "$T/of.txt" "= ~ joists 19 EA"
says "$T/of.txt" "= ~ joist length 250 LF"
says "$T/of.txt" "= ~ rim length 48 LF"

echo
echo "== A U: 0,0 30,0 30,20 20,20 20,8 10,8 10,20 0,20, joists north: 25 joists, 408 LF =="
# 22 marks and 4 edge joists less the one at east 20, which mark 15 (20.0625) runs the whole of: 25; 356 + 64 - 12 = 408.
EU="$(made "$("$P" floor "$J" "$UPPER" "$FT" --points "80,0 110,0 110,20 100,20 100,8 90,8 90,20 80,20" "${K[@]}")")"
"$P" line "$J" "$EU" "Floor joists, 2x10 at 16 in" joists --cost-type material >/dev/null
of "$EU"
says "$T/of.txt" "= ~ joists 25 EA"
says "$T/of.txt" "= ~ joist length 408 LF"

echo
echo "== A gable roof over 24 by 16, 6 in 12, ridge along the 24 ft: a line on the roof names its rafters: 13 a plane, 26 =="
"$P" roof "$J" "Main roof" --points "0,0 24,0 24,16 0,16" --pitch 6:12 --eave-height "9 ft" --status Assumed --source "scaled off the plan, made up" > "$T/roof.txt"
ROOF="$(made "$(cat "$T/roof.txt")")"
# Before a line names the rafters, no plane has any.
"$P" show "$J" | tr -s ' ' > "$T/show.txt"
lacks "$T/show.txt" "rafter"
"$P" line "$J" "$ROOF" "Rafters, 2x8 at 24 in" rafters --cost-type material >/dev/null
# The hip roof first, as drawn: its planes (an eave of 24 ft with its ridge 8 ft; and an eave of 16 ft, a triangle).
of "$ROOF"
# 4 planes: 2 x (11 + 7) = 36 rafters; 2 x (63.9375 + 31.9375) x 1.118034 = 214.383 LF.
says "$T/of.txt" "= ~ rafters 36 EA"
says "$T/of.txt" "= ~ rafter length 214.383 LF"
says "$T/of.txt" "= ~ rafter longest 8.9443 ft"
PLANES="$("$P" show "$J" | grep -oE 'plane-[0-9]+' | sort -u | tr '\n' ' ')"
longs=0; shorts=0
for pl in $PLANES; do
    of "$pl"
    if grep -qF "= ~ rafters 11 EA" "$T/of.txt"; then
        # 63.9375 ft on plan x 1.118034 = 71.4843 LF; one piece under 6 in left out and said.
        says "$T/of.txt" "= ~ rafter length 71.4843 LF"; longs=$((longs + 1))
        "$P" show "$J" | tr -s ' ' > "$T/show.txt"
        says "$T/show.txt" "rafters: 11 EA (1 piece under 6 in left out; blocking, hangers and doubled members are not in this count)"
    elif grep -qF "= ~ rafters 7 EA" "$T/of.txt"; then
        # 31.9375 x 1.118034 = 35.7072 LF.
        says "$T/of.txt" "= ~ rafter length 35.7072 LF"; shorts=$((shorts + 1))
    else
        echo "FAIL: a plane of the hip roof has neither 11 rafters nor 7"; cat "$T/of.txt"; exit 1
    fi
done
[ "$longs" = 2 ] && [ "$shorts" = 2 ] || { echo "FAIL: the hip roof's planes: $longs long, $shorts short"; exit 1; }
echo "    two planes of 11 and two of 7"
echo "    then the same roof as a gable: its two short edges are gables"
"$P" set "$J" "$ROOF" edge:2 gable "${K[@]}" >/dev/null
"$P" set "$J" "$ROOF" edge:4 gable "${K[@]}" >/dev/null
of "$ROOF"
# 24 / 2 + 1 = 13 rafters a plane, 8 x 1.118034 = 8.944272 ft each: 13 x 8.944272 = 116.2755 LF; the roof 26, 232.5511 LF.
says "$T/of.txt" "= ~ rafters 26 EA"
says "$T/of.txt" "= ~ rafter length 232.5511 LF"
says "$T/of.txt" "= ~ rafter longest 8.9443 ft"
PLANES="$("$P" show "$J" | grep -oE 'plane-[0-9]+' | sort -u | tr '\n' ' ')"
for pl in $PLANES; do
    of "$pl"
    says "$T/of.txt" "= ~ rafters 13 EA"
    says "$T/of.txt" "= ~ rafter length 116.2755 LF"
done

echo
echo "== The starter library's 'Framed floor' and 'Asphalt shingle roof, whole roof' now count their joists and rafters =="
"$P" import assemblies "$J" starter > "$T/in.txt"
says "$T/in.txt" "set: 42"
BASE="$(made "$("$P" floor "$J" "$GROUND" "$FT" --points "30,0 54,0 54,16 30,16" "${K[@]}")")"
"$P" apply "$J" "$BASE" "Framed floor" --source "the starter library" >/dev/null
"$P" show "$J" | tr -s ' ' > "$T/show.txt"
of "$BASE"
says "$T/of.txt" "= ~ quantity: Floor joists, 2x10 at 16 in 19 EA"
says "$T/of.txt" "= ~ quantity: Rim joist 48 LF"
refused "$P" line "$J" "$BASE" "Nothing" joists_all --cost-type material
says "$T/out.txt" "a floor has plan area, perimeter, thickness, joists, joist length, joist longest, rim length"

echo
echo "== Saved and opened again: what it says is what it said; a job with none of this is as it was =="
"$P" show "$J" > "$T/a.txt"
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "saving it back writes the same file"
"$P" show "$J" > "$T/b.txt"
cmp -s "$T/a.txt" "$T/b.txt" || { echo "FAIL: show differs after check"; exit 1; }
echo "    show is the same"
"$P" house "$T/house.json" >/dev/null
"$P" show "$T/house.json" > "$T/house-show.txt"
lacks "$T/house-show.txt" "rafter"
lacks "$T/house-show.txt" "joists"
echo
echo "demos/joists.sh: every step passed"
