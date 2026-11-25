#!/usr/bin/env bash
# A beam, its posts and their pad footings (call 164). From an empty job, the house of
# demos/hidden.sh with its partition to demolish and said bearing: a beam in the partition's
# line, a post under each end and a pad under each post, each of a type whose size is
# typed and Assumed (the program sizes nothing); each counted, priced, listed for the
# engineer, in the clash test and in the BIM file; the three starter assemblies on them;
# saved and read back the same.
# Run from the repository root:  demos/structure.sh
# The job and every price are made up (no real job data) and live in a temporary folder.
# Every step fails the script if the program does not say what the step expects. The
# figures are worked by hand in STATUS.md ("A beam, its posts and their pad footings").
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
A=(--status Assumed --source "drawn size, not an engineer's")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
unchanged() { cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }; }
shown() { "$P" show "$J" | tr -s ' ' > "$T/show.txt"; }
est() { "$P" estimate "$J" > "$T/est.txt"; }

echo "== The house: 24 ft by 16 ft, 6 in walls 9 ft tall, the walls existing, the partition to demolish and bearing =="
"$P" new "$J" "Structure (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" mark "$J" "$SOUTH" "$EAST" "$NORTH" "$WEST" "$PART" existing --keep "${K[@]}" >/dev/null
"$P" mark "$J" "$PART" demolish --keep "${K[@]}" >/dev/null
"$P" set "$J" "$PART" bearing bearing --status Assumed --source "joists lap over it" >/dev/null
"$P" set "$J" "$PART" "in the wall" nothing --status Known --source "opened at the base" >/dev/null

echo
echo "== 1. A beam, LVL 3 1/2 x 11 7/8, in the partition's line, its top 9 ft up: 16 LF, \$152.00 at \$9.50/LF; no weight =="
LVL="$(made "$("$P" type "$J" beam "LVL 3 1/2 x 11 7/8" --width "3.5 in" --depth "11.875 in" "${A[@]}")")"
BEAM="$(made "$("$P" beam "$J" "$S" "$LVL" --points "12,0 12,16" --top "9 ft" "${K[@]}")")"
shown
sed -n "/^ *$BEAM /,/^ *[a-z]*-[0-9]* /p" "$T/show.txt" > "$T/beam.txt"
says "$T/beam.txt" "= length 16 LF"
says "$T/beam.txt" "height is to its top"
# The weight is not worked out where the type gives none, and the figure says why.
says "$T/show.txt" "$BEAM beam 1 weight: unknown (its type gives no weight per foot"
"$P" line "$J" "$BEAM" "Beam" length --cost-type material --division 6 >/dev/null
"$P" set "$J" "$BEAM" unit_cost '$9.50/LF' --line "Beam" "${K[@]}" >/dev/null
"$P" line "$J" "$BEAM" "Weight of the steel" weight --cost-type material --division 6 >/dev/null
est
says "$T/est.txt" "16.00 LF         \$9.50/LF       \$152.00"
shown
says "$T/show.txt" "weight: unknown (its type gives no weight per foot"
"$P" remove-line "$J" "$BEAM" "Weight of the steel" >/dev/null

echo
echo "== 2. A second type, W8x18, 18 lb/ft: the same beam changed to it weighs 16 x 18 = 288 LB, \$316.80 at \$1.10/LB; changed back =="
W8="$(made "$("$P" type "$J" beam "W8x18" --width "5.25 in" --depth "8.14 in" --weight "18 lb/ft" "${A[@]}")")"
refused "$P" type "$J" beam "W8x19" --width "5.25 in" --depth "8.14 in" --weight "19" "${A[@]}"
says "$T/out.txt" "a weight per foot with no unit"
refused "$P" type "$J" post "Post with weight" --width "5.5 in" --depth "5.5 in" --weight "18 lb/ft" "${A[@]}"
says "$T/out.txt" "a weight per foot is a beam's"
"$P" set "$J" "$BEAM" type "$W8" "${K[@]}" >/dev/null
"$P" line "$J" "$BEAM" "Steel, by the pound" weight --cost-type material --division 6 >/dev/null
"$P" set "$J" "$BEAM" unit_cost '$1.10/LB' --line "Steel, by the pound" "${K[@]}" >/dev/null
shown
sed -n "/^ *$BEAM /,/^ *[a-z]*-[0-9]* /p" "$T/show.txt" > "$T/beam.txt"
says "$T/beam.txt" "weight 288 LB"
est
says "$T/est.txt" "\$316.80"
# A beam is changed to a type of a beam, and to no other.
refused "$P" set "$J" "$BEAM" type "$WT" "${K[@]}"
says "$T/out.txt" "is not a type of beam"
"$P" set "$J" "$BEAM" type "$LVL" "${K[@]}" >/dev/null
"$P" remove-line "$J" "$BEAM" "Steel, by the pound" >/dev/null
shown
sed -n "/^ *$BEAM /,/^ *[a-z]*-[0-9]* /p" "$T/show.txt" > "$T/beam.txt"
says "$T/beam.txt" "type LVL 3 1/2 x 11 7/8 ($LVL)"
says "$T/beam.txt" "length 16 LF"

echo
echo "== 3. Refused, the file as it was: a beam stood on end, a beam 0.01 ft long, a post 0 ft tall, a pad with no thickness =="
POSTT="$(made "$("$P" type "$J" post "6x6 post" --width "5.5 in" --depth "5.5 in" "${A[@]}")")"
PAD="$(made "$("$P" type "$J" pad "Pad 36 x 36" --width "3 ft" --depth "3 ft" "${A[@]}")")"
cp "$J" "$T/before.json"
refused "$P" beam "$J" "$S" "$LVL" --points "12,0 12,0" --top "9 ft" "${K[@]}"
says "$T/out.txt" "a beam runs along, and what stands is a post"
unchanged
refused "$P" beam "$J" "$S" "$LVL" --points "12,0 12,0.01" --top "9 ft" "${K[@]}"
says "$T/out.txt" "is 0.01 ft long; a run goes from one point to another"
unchanged
refused "$P" post "$J" "$S" "$POSTT" --at "12,0" --height "0 ft" "${K[@]}"
says "$T/out.txt" "a post's top is above its base"
unchanged
refused "$P" pad "$J" "$S" "$PAD" --at "12,0" --top "-1 ft" "${K[@]}"
says "$T/out.txt" "--thickness is required"
unchanged
refused "$P" pad "$J" "$S" "$PAD" --at "12,0" --thickness "0 ft" --top "-1 ft" "${K[@]}"
says "$T/out.txt" "a pad footing has a thickness, and nobody gave one"
unchanged
refused "$P" pad "$J" "$S" "$LVL" --at "12,0" --thickness "18 in" --top "-1 ft" "${K[@]}"
says "$T/out.txt" "is a beam's"
unchanged
refused "$P" type "$J" pad "Round pad" --round "36 in" "${A[@]}"
says "$T/out.txt" "a pad footing is square or rectangular"

echo
echo "== 4. Posts, 5.5 in by 5.5 in: two, at 12,0 and 12,16, 8 ft tall: 8 LF each, \$64 each, \$128.00 =="
P1="$(made "$("$P" post "$J" "$S" "$POSTT" --at "12,0" --height "8 ft" "${K[@]}")")"
P2="$(made "$("$P" post "$J" "$S" "$POSTT" --at "12,16" --height "8 ft" "${K[@]}")")"
for p in "$P1" "$P2"; do
    "$P" line "$J" "$p" "6x6 post, each" 1 --cost-type material --division 6 >/dev/null
    "$P" set "$J" "$p" unit_cost '$64/EA' --line "6x6 post, each" "${K[@]}" >/dev/null
done
shown
sed -n "/^ *$P1 /,/^ *[a-z]*-[0-9]* /p" "$T/show.txt" > "$T/post.txt"
says "$T/post.txt" "= height 8 LF"
est
[ "$(grep -c '\$64.00' "$T/est.txt" || true)" -ge 2 ] || { echo "FAIL: two posts at \$64.00"; cat "$T/est.txt"; exit 1; }
echo "    two posts at \$64.00"
# A post with no height stands to its level's height, and says so: Assumed.
"$P" post "$J" "$S" "$POSTT" --at "6,8" "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "stands to its level's height, 9 ft (Assumed)"
P3="$(made "$(cat "$T/out.txt")")"
shown
says "$T/show.txt" "$P3 post 3 height: 9 LF (follows from a flagged input)"
"$P" remove "$J" "$P3" >/dev/null

echo
echo "== 5. A pad, 3 ft by 3 ft, under each post, 18 in thick, its top 1 ft below the floor: 9 SF, form 18 SF, 0.5 CY each, \$90.00 =="
D1="$(made "$("$P" pad "$J" "$S" "$PAD" --at "12,0" --thickness "18 in" --top "-1 ft" "${K[@]}")")"
D2="$(made "$("$P" pad "$J" "$S" "$PAD" --at "12,16" --thickness "18 in" --top "-1 ft" "${K[@]}")")"
for d in "$D1" "$D2"; do
    "$P" line "$J" "$d" "Concrete, by the yard" volume --cost-type material --division 3 >/dev/null
    "$P" set "$J" "$d" unit_cost '$180/CY' --line "Concrete, by the yard" "${K[@]}" >/dev/null
done
shown
sed -n "/^ *$D1 /,/^ *[a-z]*-[0-9]* /p" "$T/show.txt" > "$T/pad.txt"
says "$T/pad.txt" "plan area 9 SF"
says "$T/pad.txt" "form area 18 SF"
says "$T/pad.txt" "volume 0.5 CY"
says "$T/pad.txt" "thickness 18 in"
est
[ "$(grep -c '\$90.00' "$T/est.txt" || true)" -ge 2 ] || { echo "FAIL: two pads at \$90.00"; cat "$T/est.txt"; exit 1; }
echo "    two pads at \$90.00"

echo
echo "== 6. The estimate: Division 6 \$280.00 (152 + 128), Division 3 \$180.00, job cost \$460.00 =="
sq() { tr -s ' ' < "$1"; }
sq "$T/est.txt" > "$T/est-sq.txt"
says "$T/est-sq.txt" " 3 Concrete ~\$180.00 2 lines"
says "$T/est-sq.txt" " 6 Wood and Plastics \$280.00 3 lines"
grep -E "^ *Job cost" "$T/est.txt" | grep -q '460.00' || { echo "FAIL: job cost is not 460.00"; grep -E "Job cost" "$T/est.txt"; exit 1; }
echo "    Job cost 460.00"

echo
echo "== 7. A round duct 12 in across from 6,8 to 18,8, its middle 8.5 ft up: it is in the beam; framing is tested against nothing of the building =="
DUCTT="$(made "$("$P" type "$J" duct "12 in round" --round "12 in" "${K[@]}")")"
DUCT="$("$P" run "$J" "$S" duct "$DUCTT" --points "6,8 18,8" --height "8.5 ft" "${K[@]}" | grep -oE 'duct-[0-9]+' | head -1)"
"$P" clashes "$J" > "$T/clashes.txt"
says "$T/clashes.txt" "In the same space: 1 (1 to verify in the field)"
says "$T/clashes.txt" "[$DUCT]"
says "$T/clashes.txt" "[$BEAM]"
lacks "$T/clashes.txt" "[$P1]"
lacks "$T/clashes.txt" "[$D1]"
lacks "$T/clashes.txt" "beam 1 [$BEAM] and wall 5"
lacks "$T/clashes.txt" "wall 5 [$PART] and beam 1"
says "$T/clashes.txt" "a beam, a post and a pad footing are not tested against walls, floors, footings or each other"
[ "$(grep -c "are not tested against walls, floors, footings or each other" "$T/clashes.txt")" = 1 ] || { echo "FAIL: said more than once"; exit 1; }
echo "    (said once)"
# A job with none of it says nothing of it.
"$P" house "$T/house.json" >/dev/null
"$P" clashes "$T/house.json" > "$T/house-clashes.txt"
lacks "$T/house-clashes.txt" "a beam, a post"

echo
echo "== 8. structure: what carries the building, for the engineer =="
"$P" structure "$J" > "$T/structure.txt"
cat "$T/structure.txt" | sed 's/^/    | /'
says "$T/structure.txt" "wall 5 [$PART] comes down; bearing: bearing (Assumed: joists lap over it)"
says "$T/structure.txt" "Beam 1 [$BEAM] (LVL 3 1/2 x 11 7/8): 16 LF, top 9 ft above the floor"
says "$T/structure.txt" "Post 1 [$P1] (6x6 post): 8 LF tall, its base on the floor"
says "$T/structure.txt" "Pad 2 [$D2] (Pad 36 x 36): 18 in thick, its top 1 ft below the floor"
says "$T/structure.txt" "Members whose size is not Known: 5 of 5: to be sized by an engineer before it is built or priced firm."
cp "$J" "$T/known.json"
for t in "$LVL" "$POSTT" "$PAD"; do
    for v in width depth; do
        what="width"; [ "$v" = depth ] && what="depth"
        "$P" measure "$T/known.json" "$t" "$what" same --source "engineer's letter, made up" >/dev/null
    done
done
"$P" structure "$T/known.json" > "$T/known.txt"
says "$T/known.txt" "Members whose size is not Known: none. All 5 sizes are Known."

echo
echo "== 9. The BIM file: 1 IFCBEAM, 2 IFCCOLUMN, 2 IFCFOOTING (.PAD_FOOTING.); a strip footing is .STRIP_FOOTING. as before =="
STT="$(made "$("$P" type "$J" footing "Strip 24 in" --width "24 in" "${K[@]}")")"
"$P" footing "$J" "$S" "$STT" --points "0,0 24,0" --depth "12 in" "${K[@]}" >/dev/null
"$P" export ifc "$J" "$T/out.ifc" > "$T/export.txt"
cat "$T/export.txt" | sed 's/^/    | /'
[ "$(grep -c '=IFCBEAM(' "$T/out.ifc")" = 1 ] || { echo "FAIL: IFCBEAM"; exit 1; }
[ "$(grep -c '=IFCCOLUMN(' "$T/out.ifc")" = 2 ] || { echo "FAIL: IFCCOLUMN"; exit 1; }
[ "$(grep -c 'PAD_FOOTING' "$T/out.ifc")" = 2 ] || { echo "FAIL: PAD_FOOTING"; exit 1; }
[ "$(grep -c 'STRIP_FOOTING' "$T/out.ifc")" = 1 ] || { echo "FAIL: STRIP_FOOTING"; exit 1; }
echo "    1 IFCBEAM, 2 IFCCOLUMN, 2 pad footings, 1 strip footing"
grep -q "Qto_BeamBaseQuantities" "$T/out.ifc" && grep -q "Qto_ColumnBaseQuantities" "$T/out.ifc" || { echo "FAIL: quantity sets"; exit 1; }
# The beam's Length is 16 ft (4.8768 m), written in the model's unit, the foot, with how sure.
grep -qE "IFCQUANTITYLENGTH\('Length','Known',\\\$,16\.,'length of its line on the plan'\)" "$T/out.ifc" || { echo "FAIL: the beam's Length is not 16 ft"; grep IFCQUANTITYLENGTH "$T/out.ifc" | head; exit 1; }
echo "    the beam's Length: 16 ft (4.8768 m), Known (its line is; its section is Assumed)"
# Read back in (call 204, point 5): the beam and the two posts are straight boxes, so they come in as a beam and two posts, sized from their
# bodies; nothing is left out. (Before call 204 import said "beam" and "column" by name as not read.)
"$P" new "$T/brought.json" "Brought in (not a real job)" >/dev/null
"$P" import ifc "$T/brought.json" "$T/out.ifc" --status Assumed --source "made-up model" > "$T/import.txt" 2>&1 || true
says "$T/import.txt" "1 beam"
says "$T/import.txt" "2 posts"
says "$T/import.txt" "not brought in: 0 things"
if python3 -c "import ifcopenshell" 2>/dev/null; then
    python3 tools/ifc-check.py "$T/out.ifc" > "$T/check-ifc.txt" 2>&1 || { echo "FAIL: the outside checker did not pass the BIM file"; cat "$T/check-ifc.txt"; exit 1; }
    sed 's/^/    | /' "$T/check-ifc.txt" | head -12
else
    echo "    (IfcOpenShell is not on this machine: the outside checker was not run)"
fi

echo
echo "== 10. The three starter assemblies on the beam, a post and a pad =="
"$P" import assemblies "$J" starter >/dev/null
"$P" apply "$J" "$BEAM" "Beam, engineered lumber" --source "the starter library" >/dev/null
"$P" apply "$J" "$P1" "Post" --source "the starter library" >/dev/null
"$P" apply "$J" "$D1" "Pad footing, formed" --source "the starter library" >/dev/null
shown
sed -n "/^ *$BEAM /,/^ *[a-z]*-[0-9]* /p" "$T/show.txt" > "$T/beam.txt"
says "$T/beam.txt" "quantity: Beam, engineered lumber 16 LF"
says "$T/beam.txt" "quantity: Hangers and bearing hardware 2 EA"
says "$T/beam.txt" "quantity: Set the beam 16 LF"
sed -n "/^ *$P1 /,/^ *[a-z]*-[0-9]* /p" "$T/show.txt" > "$T/post.txt"
says "$T/post.txt" "quantity: Post 1 EA"
says "$T/post.txt" "quantity: Post cap and base 2 EA"
says "$T/post.txt" "quantity: Set the post 1 EA"
sed -n "/^ *$D1 /,/^ *[a-z]*-[0-9]* /p" "$T/show.txt" > "$T/pad.txt"
says "$T/pad.txt" "quantity: Pad concrete 0.5 CY"
says "$T/pad.txt" "quantity: Pad forms 18 SF"
says "$T/pad.txt" "quantity: Form, place and strip 0.5 CY"

echo
echo "== 11. Saved and opened again: show, estimate, clashes and structure print what they printed =="
"$P" show "$J" > "$T/a-show.txt"; "$P" estimate "$J" > "$T/a-est.txt"; "$P" clashes "$J" > "$T/a-clash.txt"; "$P" structure "$J" > "$T/a-struct.txt"
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "saving it back writes the same file"
"$P" show "$J" > "$T/b-show.txt"; "$P" estimate "$J" > "$T/b-est.txt"; "$P" clashes "$J" > "$T/b-clash.txt"; "$P" structure "$J" > "$T/b-struct.txt"
for w in show est clash struct; do cmp -s "$T/a-$w.txt" "$T/b-$w.txt" || { echo "FAIL: $w differs after check"; exit 1; }; done
echo "    the four are the same"
echo
echo "demos/structure.sh: every step passed"
