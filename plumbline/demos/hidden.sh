#!/usr/bin/env bash
# What a wall the work opens may be hiding, asked in the field (call 160). From an empty
# job, the house of demos/reno.sh with a drain pipe through the partition: before any mark
# nothing is asked; with the partition to demolish its bearing and what is in it are asked
# of it (everything priced on it riding), and so is the pipe in its way; said, they leave the
# list; a bearing wall that comes down says what that needs and that it is not in the price;
# a new window in an old wall asks the same of that wall; saved and read back the same.
# Run from the repository root:  demos/hidden.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by hand
# in STATUS.md ("What a wall the work opens may be hiding").
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
unchanged() { cmp -s "$J" "$T/before.json" || { echo "FAIL: the refusal changed the job"; exit 1; }; }
vif() { "$P" vif "$J" > "$T/vif.txt"; }
shown() { "$P" show "$J" > "$T/show.txt"; }

echo "== 1. The house, a drain pipe through the partition, prices made up; nothing marked =="
"$P" new "$J" "Hidden (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
PT="$(made "$("$P" type "$J" pipe "2 in drain" --round "2 in" "${K[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
PIPE="$("$P" run "$J" "$S" pipe "$PT" --points "6,8 18,8" --height "4 ft" "${K[@]}" | grep -oE 'pipe-[0-9]+' | head -1)"
"$P" line "$J" "$PART" "Take down the partition" face_area --cost-type subcontract >/dev/null
"$P" set "$J" "$PART" unit_cost '$2.50/SF' --line "Take down the partition" "${K[@]}" >/dev/null
"$P" line "$J" "$PIPE" "Drain pipe" true_length --cost-type subcontract >/dev/null
"$P" set "$J" "$PIPE" unit_cost '$10/LF' --line "Drain pipe" "${K[@]}" >/dev/null
shown
lacks "$T/show.txt" "bearing"
lacks "$T/show.txt" "in the wall"
vif
says "$T/vif.txt" "Verify in field: nothing."
"$P" clashes "$J" > "$T/clashes.txt"
lacks "$T/clashes.txt" "In the way of demolition"
"$P" check "$J" > "$T/check.txt"
lacks "$T/check.txt" "is bearing"

echo
echo "== 2. All five walls existing, the partition to demolish: it is asked, and so is what is in its way =="
"$P" mark "$J" "$SOUTH" "$EAST" "$NORTH" "$WEST" "$PART" existing --keep "${K[@]}" >/dev/null
shown
lacks "$T/show.txt" "bearing"
"$P" mark "$J" "$PART" demolish --keep "${K[@]}" > "$T/out.txt"
says "$T/out.txt" "$PART: to demolish"
tail -n 1 "$T/out.txt" | grep -qxF "Is it bearing, and what is in it? Both are on the field list." || { echo "FAIL: what mark says does not end with the question"; cat "$T/out.txt"; exit 1; }
echo "    (what mark says ends: Is it bearing, and what is in it? Both are on the field list.)"
shown
sed -n "/^ *$PART  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" > "$T/part.txt"
says "$T/part.txt" "? bearing          unknown"
says "$T/part.txt" "? in the wall      unknown"
says "$T/show.txt" "wall 5 bearing: unknown (not seen until it is looked at: which way the joists above run and what bears on it)"
says "$T/show.txt" "wall 5 in the wall: unknown (pipes, ducts and wiring in an old wall are not seen until it is opened)"
# The four walls that stay have neither.
[ "$(grep -c "bearing  *unknown" "$T/show.txt")" = 1 ] || { echo "FAIL: a wall that stays was asked"; exit 1; }
echo "    (the four walls that stay are asked nothing)"
vif
# The partition: 16 x 9 = 144 SF at $2.50 = $360.00; the pipe 12 LF at $10 = $120.00; together $480.00.
says "$T/vif.txt" "Verify in field: 2 questions. \$480.00 rides on them (2 lines of the estimate, each counted once)."
says "$T/vif.txt" "wall 5 [$PART] (wall), on ground: \$360.00, 1 line"
says "$T/vif.txt" "check: ?bearing, ?in the wall"
says "$T/vif.txt" "pipe 1 [$PIPE] (pipe), on ground: \$120.00, 1 line"
says "$T/vif.txt" "Pipe 1 is in wall 5, which comes down: cap it, reroute it or take it out."
"$P" clashes "$J" > "$T/clashes.txt"
says "$T/clashes.txt" "In the way of demolition: 1"
says "$T/clashes.txt" "pipe 1 [$PIPE] is in wall 5 [$PART], which comes down"
says "$T/clashes.txt" "In the same space: 0"
lacks "$T/clashes.txt" "wall 5 [$PART] and pipe 1"

echo
echo "== 3. Said: not bearing, one 2 in drain: the partition leaves the list, the pipe stays =="
cp "$J" "$T/before.json"
refused "$P" set "$J" "$PART" bearing "load bearing" "${K[@]}"
says "$T/out.txt" "is not a word for it: a wall is \"bearing\" or \"not bearing\""
unchanged
refused "$P" set "$J" "$PIPE" bearing "bearing" "${K[@]}"
unchanged
"$P" set "$J" "$PART" bearing "not bearing" --status Known --source "joists run with it, seen from the attic" > "$T/out.txt"
says "$T/out.txt" "set $PART bearing = not bearing (Known)"
"$P" set "$J" "$PART" "in the wall" "one 2 in drain" --status Known --source "opened at the base" > "$T/out.txt"
says "$T/out.txt" "set $PART in the wall = one 2 in drain (Known)"
vif
says "$T/vif.txt" "Verify in field: 1 question. \$120.00 rides on them (1 line of the estimate, each counted once)."
lacks "$T/vif.txt" "wall 5 [$PART]"
says "$T/vif.txt" "Pipe 1 is in wall 5, which comes down"
shown
sed -n "/^ *$PART  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" > "$T/part.txt"
says "$T/part.txt" "  bearing          not bearing"
says "$T/part.txt" "  in the wall      one 2 in drain"
lacks "$T/part.txt" "is bearing and comes down"

echo
echo "== 4. Said bearing, Assumed: asked again, and what it needs is in show, check and the estimate =="
"$P" set "$J" "$PART" bearing bearing --status Assumed --source "joists lap over it" >/dev/null
vif
says "$T/vif.txt" "Verify in field: 2 questions. \$480.00 rides on them"
says "$T/vif.txt" "wall 5 [$PART] (wall), on ground: \$360.00, 1 line"
says "$T/vif.txt" "check: ~bearing"
CAUTION="is bearing and comes down: what it carries needs a beam, its posts and their footings, sized by an engineer, and shoring while it is out. None of that is in this price unless a line of this job says so."
shown; says "$T/show.txt" "$CAUTION"
"$P" check "$J" > "$T/check.txt"; says "$T/check.txt" "$CAUTION"
"$P" estimate "$J" > "$T/est.txt"; says "$T/est.txt" "$CAUTION"
# The program sizes nothing and prices nothing by it: the price is what it was.
"$P" estimate "$J" | tr -s ' ' > "$T/est.txt"; says "$T/est.txt" "Job cost \$480.00 2 lines"

echo
echo "== 5. The pipe to demolish too: its question goes, and it is gone from clashes =="
"$P" mark "$J" "$PIPE" demolish --keep "${K[@]}" >/dev/null
vif
lacks "$T/vif.txt" "Pipe 1 is in wall 5"
says "$T/vif.txt" "wall 5 [$PART] (wall), on ground: \$360.00, 1 line"
"$P" clashes "$J" > "$T/clashes.txt"
lacks "$T/clashes.txt" "In the way of demolition"

echo
echo "== 6. A window set in the south wall, which stays: the wall is asked, and nothing rides =="
WIN="$(made "$("$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "2.5 ft" "${K[@]}")")"
shown
sed -n "/^ *$SOUTH  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" > "$T/south.txt"
says "$T/south.txt" "? bearing          unknown"
says "$T/south.txt" "? in the wall      unknown"
vif
says "$T/vif.txt" "wall 1 [$SOUTH] (wall), on ground: no dollars riding on it"
says "$T/vif.txt" "check: ?bearing, ?in the wall"
lacks "$T/vif.txt" "\$0.00"
"$P" set "$J" "$SOUTH" bearing bearing --status Known --source "joists run across it, seen from the attic" >/dev/null
CUT="is bearing and is cut for Window 1: the opening needs a header sized by an engineer, and shoring while it is cut. None of that is in this price unless a line of this job says so."
shown; says "$T/show.txt" "Wall 1 $CUT"
"$P" check "$J" > "$T/check.txt"; says "$T/check.txt" "$CUT"

# A door in the north wall, which stays, itself marked to demolish (taken out and filled
# in): that cuts nothing, and the north wall is asked nothing (call 160, as built).
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "7 ft" "${K[@]}")")"
OLD="$(made "$("$P" door "$J" "$NORTH" "$DT" --offset "4 ft" --sill "0 ft" "${K[@]}")")"
"$P" mark "$J" "$OLD" demolish --status Known --source "made-up plan" >/dev/null
shown
sed -n "/^ *$NORTH  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" > "$T/north.txt"
lacks "$T/north.txt" "bearing"
vif
lacks "$T/vif.txt" "[$NORTH]"
echo "    the north wall, with a door in it taken out and filled in, is asked nothing"

echo
echo "== 7. Saved and opened again: the same; a job from before opens as it was =="
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "its figures agree with what is drawn, and saving it back writes the same file"
shown; cp "$T/show.txt" "$T/show1.txt"; vif; cp "$T/vif.txt" "$T/vif1.txt"; "$P" clashes "$J" > "$T/clashes1.txt"
# Read, set to what it already says, and written again by the program: the same file.
cp "$J" "$T/copy.json"
"$P" set "$T/copy.json" "$SOUTH" bearing bearing --status Known --source "joists run across it, seen from the attic" >/dev/null
cmp -s "$J" "$T/copy.json" || { echo "FAIL: the job opened and saved again is not the same file"; exit 1; }
"$P" show "$T/copy.json" | cmp -s - "$T/show1.txt" || { echo "FAIL: show printed differently when opened again"; exit 1; }
"$P" vif "$T/copy.json" | cmp -s - "$T/vif1.txt" || { echo "FAIL: the field list printed differently when opened again"; exit 1; }
"$P" clashes "$T/copy.json" | cmp -s - "$T/clashes1.txt" || { echo "FAIL: clashes printed differently when opened again"; exit 1; }
echo "    show, the field list and clashes print what they printed"
echo
echo "  PASS: nothing was asked of a wall before a mark; the partition to demolish was asked its bearing and what is in it with \$360.00 riding and the pipe in its way with \$120.00, \$480.00 together; said, they left the list; a bearing wall that comes down said what that needs and that it is not in the price; a new window in an old wall asked the same of that wall with nothing riding; saved and read back the same, to the figures worked by hand"
