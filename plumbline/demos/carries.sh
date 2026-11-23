#!/usr/bin/env bash
# What a wall carries and what it stands on (call 176). From an empty job: a two-storey box
# with a partition on each level, strip footings under the first floor's outside walls, and a
# gable roof; every wall is asked what is drawn standing on it and what it stands on (worked
# from the levels above and below and the roof, and said to be only that); the joists of the
# floor above said and not said; the partition to demolish with a wall standing on it (a
# caution, a question on the field list with the partition's dollars riding); a beam under the
# wall above answering it; a wall said not bearing with a wall standing on it; nothing else
# moving (the estimate, the clashes and the takeoff the same to the byte, the job file the
# same); and, past the hand figures of STATUS.md, a header's depth said to be a rule of thumb
# and its size asked for each opening cut in a wall that is bearing or not said (call 175, as
# built).
# Run from the repository root:  demos/carries.sh
# The job and every price are made up (no real job data) and live in a temporary folder.
# Every step fails the script if the program does not say what the step expects. The figures
# are worked by hand in STATUS.md ("What a wall carries: a demonstration, written before it is
# built"); each is printed beside what the program said.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
A=(--status Assumed --source "drawn size, not an engineer's")
SAID="worked from what is drawn above and below this line; it is not an engineer's finding"
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
# A hand figure beside the line the program said it in: hand "<file>" "<what>" "<by hand>" "<text the line holds>"
hand() {
    local line
    line="$(grep -F -- "$4" "$1" | head -1 | sed 's/^ *//')" || true
    [ -n "$line" ] || { echo "FAIL: nothing says: $4 ($2)"; cat "$1"; exit 1; }
    printf '    %-34s by hand %-14s said: %s\n' "$2" "$3" "$line"
}
# What the program says of one wall: its two figures.
wall() { "$P" carries "$J" "$1" > "$T/$2.txt"; }
vif() { "$P" vif "$J" > "$T/vif.txt"; }
checked() { "$P" check "$J" > "$T/check.txt"; }

echo "== The house: two levels of 24 ft by 16 ft, 6 in walls 9 ft tall; strip footings under the first floor's outside walls; a gable roof =="
"$P" new "$J" "Carries (not a real job)" >/dev/null
S1="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
S2="$(made "$("$P" level "$J" second --elevation "9 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
FT="$(made "$("$P" type "$J" footing "footing 24 in" --width "24 in" "${K[@]}")")"
FLT="$(made "$("$P" type "$J" floor "floor 10 in" --thickness "10 in" "${K[@]}")")"
read -r SOUTH1 EAST1 NORTH1 WEST1 <<<"$("$P" wall "$J" "$S1" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
read -r SOUTH2 EAST2 NORTH2 WEST2 <<<"$("$P" wall "$J" "$S2" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
# P1: the first floor's partition on east 12, north 0 to 16 (16 LF). P2: the second floor's, over it, north 4 to 16 (12 LF).
P1="$("$P" wall "$J" "$S1" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
P2="$("$P" wall "$J" "$S2" "$WT" --points "12,4 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
FOOT_SOUTH="$("$P" footing "$J" "$S1" "$FT" --points "0,0 24,0 24,16 0,16" --closed --depth "12 in" "${K[@]}" | grep -oE 'footing-[0-9]+' | head -1)"
FLOOR="$(made "$("$P" floor "$J" "$S2" "$FLT" --walls "${K[@]}")")"
# The roof's ridge runs east-west: the east and west edges are gables (rakes), the south and north edges eaves.
ROOF="$(made "$("$P" roof "$J" "Main roof" --points "0,0 24,0 24,16 0,16" --pitch 6:12 --eave-height "18 ft" --status Known --source "made-up survey")")"
"$P" set "$J" "$ROOF" edge:2 gable "${K[@]}" >/dev/null
"$P" set "$J" "$ROOF" edge:4 gable "${K[@]}" >/dev/null
# The partition is priced as the starter partition is: a made-up $10.00/SF of face, 16 x 9 = 144 SF, $1,440.00.
"$P" line "$J" "$P1" "Partition" face_area --cost-type subcontract >/dev/null
"$P" set "$J" "$P1" unit_cost '$10/SF' --line "Partition" "${K[@]}" >/dev/null
# What the three of these say before any figure is asked for: the money and the model, to compare at the end.
"$P" estimate "$J" > "$T/est-before.txt"; "$P" clashes "$J" > "$T/clashes-before.txt"; "$P" takeoff "$J" > "$T/takeoff-before.txt"; cp "$J" "$T/job-before.json"

echo
echo "== 1. As drawn: each wall's two figures =="
"$P" carries "$J" > "$T/all.txt"
echo "  the first floor's partition, P1 (16 LF):"
wall "$P1" p1
hand "$T/p1.txt" "P1 carries P2" "12.00 LF" "[$P2] for 12.00 LF"
hand "$T/p1.txt" "P1 and the joists above" "not said" "which way the joists of the floor above run is not said"
hand "$T/p1.txt" "P1 stands on" "none, 16.00 LF" "nothing found under it for 16.00 LF (no footing)"
says "$T/p1.txt" "$SAID"
echo "  the first floor's south wall:"
wall "$SOUTH1" s1
hand "$T/s1.txt" "south, first floor, carries" "24.00 LF" "[$SOUTH2] for 24.00 LF"
hand "$T/s1.txt" "south, first floor, stands on" "footing, 24.00 LF" "[$FOOT_SOUTH] (footing) for 24.00 LF"
lacks "$T/s1.txt" "nothing found under it"
echo "  the second floor's south wall:"
wall "$SOUTH2" s2
hand "$T/s2.txt" "south, second floor, carries" "eave, 24.00 LF" "the roof's eave for 24.00 LF"
hand "$T/s2.txt" "south, second floor, stands on" "first floor, 24.00 LF" "[$SOUTH1] (wall) for 24.00 LF"
echo "  the second floor's east wall (a rake is over it, which does not bear on it):"
wall "$EAST2" e2
hand "$T/e2.txt" "east, second floor, carries" "nothing" "nothing found standing on it"
hand "$T/e2.txt" "east, second floor, stands on" "first floor, 16.00 LF" "[$EAST1] (wall) for 16.00 LF"
lacks "$T/e2.txt" "eave"
echo "  the second floor's north wall carries the other eave, and its partition, P2, carries nothing:"
wall "$NORTH2" n2
says "$T/n2.txt" "the roof's eave for 24.00 LF"
wall "$P2" p2
hand "$T/p2.txt" "P2 carries" "nothing" "nothing found standing on it"
hand "$T/p2.txt" "P2 stands on" "P1, 12.00 LF" "[$P1] (wall) for 12.00 LF"
lacks "$T/p2.txt" "nothing found under it"
# Every wall of the ten says what the figures are, and the same figures are in `show` of a wall.
[ "$(grep -cF -- "$SAID" "$T/all.txt")" = 10 ] || { echo "FAIL: not every wall says what its figures are"; cat "$T/all.txt"; exit 1; }
echo "    all ten walls say: $SAID"
"$P" show "$J" "$P1" > "$T/show.txt"
says "$T/show.txt" "[$P2] for 12.00 LF"
says "$T/show.txt" "nothing found under it for 16.00 LF (no footing)"
says "$T/show.txt" "$SAID"
echo "  nothing is opened, so nothing is a caution and nothing is asked:"
checked
[ "$(grep -c '^!' "$T/check.txt" || true)" = 0 ] || { echo "FAIL: a caution where nothing is opened"; cat "$T/check.txt"; exit 1; }
echo "    (check: no caution)"
vif
lacks "$T/vif.txt" "stands on it"
# A wall under a level is told by its level: the figures are of each wall on the level asked.
"$P" carries "$J" "$S2" > "$T/level2.txt"
[ "$(grep -c '^  wall ' "$T/level2.txt")" = 5 ] || { echo "FAIL: the second floor's five walls"; cat "$T/level2.txt"; exit 1; }
echo "    (carries for the second level: its five walls)"
# Bearing not said, with something standing on it: counted, by level and in the job.
says "$T/all.txt" "Walls whose bearing is not said, with something standing on them: 7 in the job"
says "$T/all.txt" "ground: 5"
says "$T/all.txt" "second: 2"

echo
echo "== 2. The joists of the floor above said: north-south (the short way, alongside P1), then east-west (across it) =="
"$P" set "$J" "$FLOOR" joists_span "the short way" "${K[@]}" >/dev/null
wall "$P1" p1
hand "$T/p1.txt" "P1, joists run north-south" "P1 and no joists" "[$P2] for 12.00 LF"
lacks "$T/p1.txt" "joists of the floor above"
"$P" set "$J" "$FLOOR" joists_span "the long way" "${K[@]}" >/dev/null
wall "$P1" p1
hand "$T/p1.txt" "P1, joists run east-west" "they cross it" "the joists of the floor above cross it"
lacks "$T/p1.txt" "is not said"
# The south wall is at the edge of the floor: whichever way the joists run it carries the floor's edge, and joists are not said of it.
wall "$SOUTH1" s1
lacks "$T/s1.txt" "joists"

echo
echo "== 3. P1 marked to demolish: a caution, and a question on the field list with P1's dollars riding =="
"$P" mark "$J" "$SOUTH1" "$EAST1" "$NORTH1" "$WEST1" "$SOUTH2" "$EAST2" "$NORTH2" "$WEST2" "$P2" "$P1" existing --keep "${K[@]}" >/dev/null
"$P" mark "$J" "$P1" demolish --keep "${K[@]}" >/dev/null
checked
says "$T/check.txt" "stands on it for 12.00 LF: once it is down, nothing is found under that wall for 12.00 LF ("
says "$T/check.txt" "the joists of the floor above cross it: once it is down, nothing is found under them for 16.00 LF ("
says "$T/check.txt" "$SAID"
vif
hand "$T/vif.txt" "the dollars riding on P1" "\$1,440.00" "[$P1] (wall), on ground: \$1,440.00, 1 line"
says "$T/vif.txt" "stands on it for 12.00 LF: once it is down, nothing is found under that wall"
says "$T/vif.txt" "The joists of the floor above cross it"
says "$T/vif.txt" "$SAID"
wall "$P2" p2
hand "$T/p2.txt" "P2 now stands on" "nothing, 12.00 LF" "nothing found under it for 12.00 LF"
says "$T/p2.txt" "[$P1] (wall) for 12.00 LF, which comes down: not counted"
# With the joists said the short way, the sentence of the wall stays and the joists go.
"$P" set "$J" "$FLOOR" joists_span "the short way" "${K[@]}" >/dev/null
checked
says "$T/check.txt" "stands on it for 12.00 LF: once it is down"
lacks "$T/check.txt" "the joists of the floor above cross it"
"$P" set "$J" "$FLOOR" joists_span "the long way" "${K[@]}" >/dev/null

echo
echo "== 4. A beam on the second floor's line, east 12 from north 0 to 16, its top at the floor, new work: P2 stands on it; the caution goes =="
BT="$(made "$("$P" type "$J" beam "LVL 3 1/2 x 11 7/8" --width "3.5 in" --depth "11.875 in" "${A[@]}")")"
BEAM="$(made "$("$P" beam "$J" "$S2" "$BT" --points "12,0 12,16" --top "0 ft" "${K[@]}")")"
wall "$P2" p2
hand "$T/p2.txt" "P2 stands on the beam" "12.00 LF" "[$BEAM] (beam) for 12.00 LF"
checked
lacks "$T/check.txt" "once it is down"
[ "$(grep -c '^!' "$T/check.txt" || true)" = 0 ] || { echo "FAIL: a caution is left after the beam"; cat "$T/check.txt"; exit 1; }
echo "    (check: no caution)"
vif
lacks "$T/vif.txt" "stands on it for"
lacks "$T/vif.txt" "cross it"
# What the beam asks stands: its size is typed and Assumed, and is to be sized by an engineer.
says "$T/vif.txt" "LVL 3 1/2 x 11 7/8 [$BT] (type)"
"$P" structure "$J" > "$T/structure.txt"
says "$T/structure.txt" "Members whose size is not Known: 1 of 1: to be sized by an engineer before it is built or priced firm."

echo
echo "== 5. The beam taken away; P1 marked existing and said not bearing: a caution =="
"$P" remove "$J" "$BEAM" >/dev/null
"$P" mark "$J" "$P1" existing --keep "${K[@]}" >/dev/null
"$P" set "$J" "$P1" bearing "not bearing" --status Known --source "joists run with it, seen from the attic" >/dev/null
checked
says "$T/check.txt" "is said not bearing, and"
says "$T/check.txt" "stands on it for 12.00 LF ("
lacks "$T/check.txt" "once it is down"
vif
lacks "$T/vif.txt" "stands on it for 12.00 LF"
"$P" carries "$J" > "$T/all.txt"
# Said, so no longer counted among the walls whose bearing is not said.
says "$T/all.txt" "Walls whose bearing is not said, with something standing on them: 6 in the job"

echo
echo "== 6. Nothing else moves: the estimate, the clashes and the takeoff the same to the byte; the job file the same =="
# Put back as it was: the beam, and P1 as it was marked and said, are not what is compared; the compared job is the one of step 1.
cp "$T/job-before.json" "$T/same.json"
"$P" carries "$T/same.json" > /dev/null
"$P" carries "$T/same.json" "$P1" > /dev/null
"$P" show "$T/same.json" "$P1" > /dev/null
"$P" check "$T/same.json" > /dev/null
"$P" vif "$T/same.json" > /dev/null
"$P" estimate "$T/same.json" | cmp -s - "$T/est-before.txt" || { echo "FAIL: the estimate moved"; exit 1; }
"$P" clashes "$T/same.json" | cmp -s - "$T/clashes-before.txt" || { echo "FAIL: the clashes moved"; exit 1; }
"$P" takeoff "$T/same.json" | cmp -s - "$T/takeoff-before.txt" || { echo "FAIL: the takeoff moved"; exit 1; }
cmp -s "$T/same.json" "$T/job-before.json" || { echo "FAIL: asking for the figures changed the job file"; exit 1; }
echo "    the estimate, the clashes and the takeoff are the same to the byte, and so is the job file"
# A job saved before opens and saves the same, and nothing is added to it.
"$P" sixteen "$T/old.json" >/dev/null
cp "$T/old.json" "$T/old-before.json"
"$P" check "$T/old.json" > "$T/check.txt"
says "$T/check.txt" "its figures agree with what is drawn, and saving it back writes the same file"
"$P" carries "$T/old.json" > /dev/null
cmp -s "$T/old.json" "$T/old-before.json" || { echo "FAIL: the figures changed a job saved before"; exit 1; }
grep -q "carries\|stands_on\|stands on" "$T/old.json" && { echo "FAIL: the figures are kept in the job"; exit 1; }
echo "    a made-up whole house opens and saves the same, and holds no figure of these"

echo
echo "== 7. (beyond STATUS.md, call 175 as built) A window cut in an old wall: its header's depth is a rule of thumb; its size is asked unless the wall is said not bearing =="
WINT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${K[@]}")")"
"$P" window "$J" "$SOUTH1" "$WINT" --offset "12 ft" --sill "2.5 ft" "${K[@]}" >/dev/null
vif
HEADER="What size is the header over window 1? It is drawn 5 1/2 in deep (Assumed: a rule of thumb used to lay out the cripples, not a size: an engineer or a span table sizes a header)."
hand "$T/vif.txt" "the header's depth, 3 ft opening" "2x6, 5 1/2 in" "It is drawn 5 1/2 in deep"
says "$T/vif.txt" "$HEADER"
checked
says "$T/check.txt" "is cut for window 1, and"
# Said bearing: still asked. Said not bearing: not asked.
"$P" set "$J" "$SOUTH1" bearing bearing --status Known --source "joists run across it, seen from the attic" >/dev/null
vif; says "$T/vif.txt" "What size is the header over window 1?"
"$P" set "$J" "$SOUTH1" bearing "not bearing" --status Known --source "joists run with it, seen from the attic" >/dev/null
vif; lacks "$T/vif.txt" "What size is the header over window 1?"
# No count of lumber moves: framing.sh is the proof, run by hand beside this one.

echo
echo "  PASS: every wall said what is drawn standing on it and what it stands on, worked from the levels above and below and the roof, to the figures worked by hand, and said it is no engineer's finding; the joists were said and not said; the partition to demolish was a caution and a question with \$1,440.00 riding, answered by a beam under the wall above and not by anything else; a wall said not bearing with a wall on it was a caution; the estimate, the clashes, the takeoff and the job file did not move; a header's depth was said to be a rule of thumb and its size asked where the wall is bearing or not said"
