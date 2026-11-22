#!/usr/bin/env bash
# Two layouts side by side (call 154). From empty jobs, the house of demos/roomline.sh made
# twice: A, an addition 12 ft deep (new walls 24,0 36,0 36,16 24,16), and B, one 16 ft deep
# (24,0 40,0 40,16 24,16); a third job left empty. `plumbline options` sets them side by side
# to the figures worked by hand in STATUS.md ("Two layouts side by side"); each file is byte
# for byte as it was after; one file, or four, is refused.
# Run from the repository root:  demos/options.sh
# The jobs are made up (no real job data) and live in a temporary folder. Every step fails
# the script if the program does not do what the step says.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
K=(--status Known --source "made-up survey")
N=(--status Assumed --source "made-up addition plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
refused() { if "$@" > "$T/out.txt" 2>&1; then echo "FAIL: this was taken: $*"; cat "$T/out.txt"; exit 1; fi; }
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
# A row of the table: its label, then every figure on it, in one line (spaces squeezed).
row() { grep -F -- "$2" "$1" | head -1 | tr -s ' ' | sed 's/^ //'; }
has() { local r; r="$(row "$1" "$2")"; [ "$r" = "$3" ] || { echo "FAIL: the row \"$2\" reads: $r"; echo "       it should read: $3"; cat "$1"; exit 1; }; printf '    %s\n' "$3"; }

# A layout: the house, its east wall to demolish, an addition $3 ft out from it, two rooms
# and a room line, the addition's walls priced at $12/SF and the Kitchen's floor at $8/SF.
layout() {
  local J="$1" title="$2" x="$3" S WT SOUTH EAST NORTH WEST NEW KITCHEN
  "$P" new "$J" "$title" >/dev/null
  S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
  WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
  read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
  "$P" mark "$J" "$SOUTH" "$NORTH" "$WEST" existing --keep "${K[@]}" >/dev/null
  "$P" mark "$J" "$EAST" demolish "${K[@]}" >/dev/null
  NEW="$("$P" wall "$J" "$S" "$WT" --points "24,0 $x,0 $x,16 24,16" --height "9 ft" "${N[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
  "$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}" >/dev/null
  KITCHEN="$(made "$("$P" room "$J" "$S" "Kitchen" --at "$((x - 6)),8" "${K[@]}")")"
  "$P" room-line "$J" "$S" --points "24,0 24,16" "${N[@]}" >/dev/null
  for w in $NEW; do
    "$P" line "$J" "$w" "Wall, framed and finished" face_area --cost-type material --division 6 >/dev/null
    "$P" set "$J" "$w" unit_cost '$12.00/SF' --line "Wall, framed and finished" --status Known --source "made-up price" >/dev/null
  done
  "$P" line "$J" "$KITCHEN" "Flooring" floor_area --cost-type material --division 9 >/dev/null
  "$P" set "$J" "$KITCHEN" unit_cost '$8.00/SF' --line "Flooring" --status Known --source "made-up price" >/dev/null
}

echo "== 1. Three jobs made from empty with the command line =="
layout "$T/a.json" "Addition 12 ft" 36
layout "$T/b.json" "Addition 16 ft" 40
"$P" new "$T/c.json" "Nothing drawn yet" >/dev/null
cp "$T/a.json" "$T/a.before"; cp "$T/b.json" "$T/b.before"; cp "$T/c.json" "$T/c.before"
"$P" estimate "$T/a.json" | tr -s ' ' > "$T/ea.txt"
says "$T/ea.txt" "Job cost ~\$5,777.00"
"$P" estimate "$T/b.json" | tr -s ' ' > "$T/eb.txt"
says "$T/eb.txt" "Job cost ~\$7,137.00"

echo
echo "== 2. Two layouts side by side, to the figures worked by hand =="
"$P" options "$T/a.json" "$T/b.json" > "$T/two.txt"
cat "$T/two.txt"
says "$T/two.txt" "Addition 16 ft less Addition 12 ft"
has "$T/two.txt" "Living" "Living ~368.125 SF ~368.125 SF same"
has "$T/two.txt" "Kitchen" "Kitchen ~182.125 SF ~244.125 SF +62 SF"
has "$T/two.txt" "all rooms" "all rooms ~550.25 SF ~612.25 SF +62 SF"
has "$T/two.txt" "walls, new" "walls, new (with temporary) ~40 LF ~48 LF +8 LF"
has "$T/two.txt" "walls, existing" "walls, existing to remain 64 LF 64 LF same"
has "$T/two.txt" "walls, to demolish" "walls, to demolish 16 LF 16 LF same"
has "$T/two.txt" "job cost" "job cost ~\$5,777.00 ~\$7,137.00 +\$1,360.00"
has "$T/two.txt" "left out of the job cost" "left out of the job cost none none same"
has "$T/two.txt" " 6 Wood" "6 Wood and Plastics ~\$4,320.00 ~\$5,184.00 +\$864.00"
has "$T/two.txt" " 9 Finishes" "9 Finishes ~\$1,457.00 ~\$1,953.00 +\$496.00"

echo
echo "== 3. The other way round: a decrease reads with a minus =="
"$P" options "$T/b.json" "$T/a.json" > "$T/back.txt"
says "$T/back.txt" "Addition 12 ft less Addition 16 ft"
has "$T/back.txt" "Kitchen" "Kitchen ~244.125 SF ~182.125 SF -62 SF"
has "$T/back.txt" "walls, new" "walls, new (with temporary) ~48 LF ~40 LF -8 LF"
has "$T/back.txt" "job cost" "job cost ~\$7,137.00 ~\$5,777.00 -\$1,360.00"

echo
echo "== 4. A third job, empty: none, no figure, and no difference worked =="
"$P" options "$T/a.json" "$T/b.json" "$T/c.json" > "$T/three.txt"
cat "$T/three.txt"
says "$T/three.txt" "Nothing drawn yet less Addition 12 ft"
has "$T/three.txt" "Living" "Living ~368.125 SF ~368.125 SF none not worked"
has "$T/three.txt" "all rooms" "all rooms ~550.25 SF ~612.25 SF none not worked"
has "$T/three.txt" "walls, new" "walls, new (with temporary) ~40 LF ~48 LF none not worked"
has "$T/three.txt" "job cost" "job cost ~\$5,777.00 ~\$7,137.00 no figure not worked"
has "$T/three.txt" "left out of the job cost" "left out of the job cost none none none same"
has "$T/three.txt" " 6 Wood" "6 Wood and Plastics ~\$4,320.00 ~\$5,184.00 none not worked"

echo
echo "== 5. Two layouts of one title, one of them with a line the estimate leaves out (call 157) =="
# The same job drawn twice, the second with the Kitchen's flooring unpriced: the estimate
# leaves that line out. Each column is headed by its title and its file's name, and the job
# cost row is followed by what each leaves out: none, and 1 line. The job cost of the second
# is $4,320.00 (the walls), $1,457.00 (the floor, 182.125 SF at $8) less than the first.
layout "$T/d.json" "Addition 12 ft" 36
KD="$("$P" show "$T/d.json" | grep -oE 'room-[0-9]+  Kitchen' | cut -d' ' -f1)"
"$P" set "$T/d.json" "$KD" unit_cost unknown --line "Flooring" --status Unknown --source "made-up: not priced yet" >/dev/null
cp "$T/d.json" "$T/d.before"
"$P" options "$T/a.json" "$T/d.json" > "$T/same.txt"
cat "$T/same.txt"
says "$T/same.txt" "Addition 12 ft (a.json)"
says "$T/same.txt" "Addition 12 ft (d.json)"
says "$T/same.txt" "Addition 12 ft (d.json) less Addition 12 ft (a.json)"
has "$T/same.txt" "job cost" "job cost ~\$5,777.00 ~\$4,320.00 -\$1,457.00"
has "$T/same.txt" "left out of the job cost" "left out of the job cost none 1 line +1 line"
has "$T/same.txt" " 9 Finishes" "9 Finishes ~\$1,457.00 no figure not worked"

echo
echo "== 6. One file, or four, is refused in one sentence =="
refused "$P" options "$T/a.json"
says "$T/out.txt" "options sets two or three layouts side by side, each a job file; 1 was given"
[ "$(wc -l < "$T/out.txt")" = 1 ] || { echo "FAIL: the refusal is more than one line"; cat "$T/out.txt"; exit 1; }
refused "$P" options
says "$T/out.txt" "options sets two or three layouts side by side, each a job file; none was given"
[ "$(wc -l < "$T/out.txt")" = 1 ] || { echo "FAIL: the refusal is more than one line"; cat "$T/out.txt"; exit 1; }
refused "$P" options "$T/a.json" "$T/b.json" "$T/c.json" "$T/a.json"
says "$T/out.txt" "options sets two or three layouts side by side, each a job file; 4 were given"
[ "$(wc -l < "$T/out.txt")" = 1 ] || { echo "FAIL: the refusal is more than one line"; cat "$T/out.txt"; exit 1; }
refused "$P" options "$T/a.json" "$T/missing.json"
says "$T/out.txt" "cannot read"

echo
echo "== 7. Nothing was written: each file is byte for byte as it was =="
cmp "$T/a.json" "$T/a.before"; cmp "$T/b.json" "$T/b.before"; cmp "$T/c.json" "$T/c.before"; cmp "$T/d.json" "$T/d.before"
echo "    a.json, b.json, c.json and d.json are unchanged"
echo
echo "  PASS: two layouts of an addition were set side by side to the figures worked by hand (Kitchen +62 SF, new walls +8 LF, job cost +\$1,360.00), a third that is empty read none and no figure with no difference worked, one file or four was refused, and no file was changed"
