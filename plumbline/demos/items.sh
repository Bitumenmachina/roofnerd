#!/usr/bin/env bash
# The item library (DECISIONS.md call 105): the things that can be placed, kept by trade
# in a table a person can open, change and bring back. The library the program ships
# with is brought into an empty job, listed trade by trade, written out the same to the
# byte, changed in the file and brought back; an item is placed and follows its item.
# Run from the repository root:  demos/items.sh
#
# The job and every size are made up (no real job data). Every step fails the script if
# the program does not do what the step says.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -40; exit 1; }; printf '    %s\n' "$2"; }

echo "== The library the program ships with, into an empty job =="
"$P" new "$J" "An item library (not a real job)" >/dev/null
L="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" --status Known --source "made-up plan")")"
IN="$("$P" import items "$J" starter)"
says "all of it is set" "  set: 21" "$IN"
says "nothing is left out" "  not used: 0" "$IN"
LIST="$("$P" items "$J")"
says "by trade" "ITEMS: the things that can be placed, by trade (21)" "$LIST"
for trade in "Division 11 Equipment" "Division 12 Furnishings" "Division 15 Mechanical" "Division 16 Electrical"; do says "a trade" "  $trade" "$LIST"; done
# Every size in it is a typical one, flagged, and says so.
says "a size is flagged" "Vanity, 36 in [" "$LIST"
says "what it is drawn as" "furnishing, drawn as a base cabinet; ~36 in wide, ~21 in deep, ~34 1/2 in high; 0 placed" "$LIST"

echo
echo "== Written out, it is the file as it is kept; brought back, nothing changes =="
"$P" export items "$J" "$T/out.csv" >/dev/null
cmp "$T/out.csv" library/items.csv || { echo "FAIL: the library written out is not the library as it is kept"; exit 1; }
echo "    written out: the same as library/items.csv, to the byte"
AGAIN="$("$P" import items "$J" "$T/out.csv")"
says "brought back" "  the same as the job already has: 21" "$AGAIN"

echo
echo "== A row changed in the file, a row added, a row that cannot be read =="
# The vanity made 48 in wide; a bench of our own; and a thing drawn as nothing there is.
sed -e 's/^"Vanity, 36 in",12 Furnishings,furnishing,base cabinet,36 in/"Vanity, 36 in",12 Furnishings,furnishing,base cabinet,48 in/' "$T/out.csv" > "$T/edited.csv"
echo '"Mudroom bench",12 Furnishings,furnishing,box,60 in,Assumed,a made-up sketch,18 in,Assumed,a made-up sketch,18 in,Assumed,a made-up sketch,,Unknown,not chosen,,Unknown,not chosen,,,' >> "$T/edited.csv"
echo '"Gazebo",12 Furnishings,furnishing,gazebo,10 ft,Assumed,a made-up sketch,10 ft,Assumed,a made-up sketch,9 ft,Assumed,a made-up sketch,,Unknown,not chosen,,Unknown,not chosen,,,' >> "$T/edited.csv"
CH="$("$P" import items "$J" "$T/edited.csv")"
says "the changed row" "    Vanity, 36 in: changed: its width" "$CH"
says "the added row" "    Mudroom bench: added" "$CH"
says "the rest" "  the same as the job already has: 20" "$CH"
says "the row that cannot be read" "Gazebo: drawn as: \"gazebo\" is not something a thing can be drawn as. There are: box, base cabinet" "$CH"

echo
echo "== One placed: it is as big as its item, and counted =="
VANITY="$("$P" items "$J" | sed -nE 's/^ +Vanity, 36 in \[([a-z]+-[0-9]+)\].*/\1/p')"
"$P" place "$J" "$L" "$VANITY" --at "6,4" --offset "0 ft" --status Assumed --source "set by eye" >/dev/null
LIST="$("$P" items "$J")"
says "the vanity" "Vanity, 36 in [$VANITY]: furnishing, drawn as a base cabinet; ~48 in wide, ~21 in deep, ~34 1/2 in high; 1 placed" "$LIST"
says "the bench, in its trade" "Mudroom bench [" "$LIST"
"$P" check "$J"
echo "  PASS: the item library came into an empty job trade by trade, went out as it is kept, took a changed row and an added one and refused one it could not read, and a placed item is as big as its item says"
