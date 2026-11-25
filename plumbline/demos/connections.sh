#!/usr/bin/env bash
# Where an item connects (call 211, the bench's step D, the model's side). At the command line, from an empty job. The figures are worked by hand in
# STATUS.md ("Where an item connects: a demonstration") and corrected where the program's own way differs (said in the contract's report: joined is
# the network's 0.01 ft; a connection is held to the box at the item's own size and not at the three check sizes).
# The item "Test lavatory" is 24 in wide, 18 in deep, 34 in high; three connections: Cold water (across -4 in, back 0, up height - 12 in), Hot water
# (across 4 in) and Waste (across 0, up height - 16 in), each facing back. Every figure is made up.
#   1. the item shows its three connections at its sizes, in inches
#   2. set down with its middle at (10, 20): 3 connections, Assumed, naming the item; takeoff names all three open
#   3. a pipe ending at the cold connection joins it: takeoff no longer names it; not a clash
#   4. a pipe ending 3 in short of the hot connection, inside the lavatory: a clash that says 3 in; connect is refused with the same 3 in
#   5. the item's height typed 36 in: the thing follows, 1 run stands off its connection; typed back to 34 in: joined again
#   6. the thing turned a quarter turn: its connections turn with it
#   7. three refusals, in inches, naming the item, the connection and the figure
#   8. the rows written out and brought into a new job with the same item: the two tables alike to the byte
#   9. the building written out as a BIM model and read back: three connection points in the file, three when read
#  10. the sample whole house: plumbline sixteen byte for byte as before
# Run from the repository root:  PLUMBLINE_BIN=<a folder with plumbline> demos/connections.sh   (without it the script builds the command-line program)
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/lav.json"
K=(--status Known --source "made-up plan")
A=(--status Assumed --source "a typical rough-in; the product is not chosen")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }

"$P" new "$J" "Connections (not a real job)" >/dev/null
LV=$("$P" level "$J" Ground --elevation "0 ft" --height "9 ft" "${K[@]}" | sed -n 's/^\(level-[0-9]*\):.*/\1/p')
printf 'Item,Trade,Kind,Width,Depth,Height,How sure,Where from\nTest lavatory,15 Mechanical,fixture,24 in,18 in,34 in,Assumed,a made-up size\n' > "$T/lav.csv"
"$P" import items "$J" "$T/lav.csv" >/dev/null
add() { "$P" connection "$J" "Test lavatory" add "$1" --takes pipe --carries "$2" --goes "$3" --across "$4" --back 0 --up "$5" --faces back --size "$6" "${A[@]}" >> "$T/added.txt"; }
add "Cold water" cold in "-4 in" "height - 12 in" "1/2 in"
add "Hot water" hot in "4 in" "height - 12 in" "1/2 in"
add "Waste" waste out "0" "height - 16 in" "1 1/2 in"
"$P" system "$J" "Cold" "Domestic Cold Water" >/dev/null
"$P" type "$J" pipe "pipe 1/2 in" --round "1/2 in" "${K[@]}" >/dev/null
PT=$("$P" type "$J" pipe "pipe 1/2 in b" --round "1/2 in" "${K[@]}" | sed -n 's/^\(type-[0-9]*\):.*/\1/p')
[ -n "$LV" ] && [ -n "$PT" ] || { echo "FAIL: the level or the pipe type was not made"; exit 1; }

echo "== 1. The item shows its three connections at its sizes, in inches =="
"$P" items "$J" "Test lavatory" > "$T/item.txt"
says "$T/item.txt" "connection 1, Cold water: pipe, cold, in; across -4 in, back 0 in, up 22 in; faces back; size 1/2 in"
says "$T/item.txt" "connection 2, Hot water: pipe, hot, in; across 4 in, back 0 in, up 22 in; faces back; size 1/2 in"
says "$T/item.txt" "connection 3, Waste: pipe, waste, out; across 0 in, back 0 in, up 18 in; faces back; size 1 1/2 in"
"$P" items "$J" > "$T/items.txt"
says "$T/items.txt" "3 connections"

echo
echo "== 2. Set down with its middle at (10, 20): 3 connections, Assumed, naming the item and the connection; takeoff names all three open =="
echo "      cold (9.6667, 20.75, 1.8333), hot (10.3333, 20.75, 1.8333), waste (10, 20.75, 1.5): the back is 9 in from the middle"
"$P" place "$J" "$LV" "Test lavatory" --at "10,20" "${K[@]}" > "$T/placed.txt"
THING=$(sed -n 's/^\([a-z]*-[0-9]*\):.*/\1/p' "$T/placed.txt")
"$P" show "$J" "$THING" > "$T/thing.txt"
says "$T/thing.txt" "connection 1: Cold water (pipe, cold, in, faces back)"
says "$J" "the item Test lavatory, its Cold water connection: a typical rough-in; the product is not chosen"
"$P" takeoff "$J" > "$T/take.txt"
for c in "Cold water" "Hot water" "Waste"; do says "$T/take.txt" "its $c connection"; done

echo
echo "== 3. A pipe ending at the cold connection joins it: takeoff no longer names it; the pipe and the lavatory are not a clash =="
COLD=$("$P" run "$J" "$LV" pipe "$PT" --points "9.6667,24 9.6667,20.75" --height "22 in" "${A[@]}" | sed -n 's/^1 straight piece: \(.*\)$/\1/p')
"$P" takeoff "$J" > "$T/take.txt"
lacks "$T/take.txt" "$THING: its Cold water connection"
lacks "$T/take.txt" "its Cold water connection"
says "$T/take.txt" "its Hot water connection"
says "$T/take.txt" "its Waste connection"
[ -n "$COLD" ] || { echo "FAIL: the cold pipe was not drawn"; exit 1; }
"$P" clashes "$J" > "$T/clash.txt"
says "$T/clash.txt" "In the same space: 0"

echo
echo "== 4. A pipe ending inside the lavatory 3 in in front of its hot connection: a clash that says 3 in; connect is refused with the same 3 in; the hot connection stays open =="
HOT=$("$P" run "$J" "$LV" pipe "$PT" --points "10.3333,17 10.3333,20.5" --height "22 in" "${A[@]}" | sed -n 's/^1 straight piece: \(.*\)$/\1/p')
[ -n "$HOT" ] || { echo "FAIL: the hot pipe was not drawn"; exit 1; }
"$P" clashes "$J" > "$T/clash.txt"
says "$T/clash.txt" "In the same space: 1"
says "$T/clash.txt" "Test lavatory's Hot water connection is 3 in away"
if "$P" connect "$J" "$THING" "$HOT" "${A[@]}" > "$T/refused.txt" 2>&1; then echo "FAIL: connect was not refused"; cat "$T/refused.txt"; exit 1; fi
says "$T/refused.txt" "its end is 3 in away from the nearest one that is open, the Hot water connection (3 in in front of it)"
"$P" takeoff "$J" > "$T/take.txt"
says "$T/take.txt" "its Hot water connection"

echo
echo "== 5. The item's height typed 36 in: the thing follows, its cold connection is 24 in up (2 ft), 2 in above the cold pipe's end; 1 run stands off; typed back to 34 in it is joined again =="
"$P" set "$J" "Test lavatory" height "36 in" > "$T/h36.txt"
says "$T/h36.txt" "1 thing set down from it follows"
says "$T/h36.txt" "1 run now stands off its connection"
says "$T/h36.txt" "2 in below Test lavatory's Cold water connection"
"$P" takeoff "$J" > "$T/take.txt"
says "$T/take.txt" "its Cold water connection"
"$P" show "$J" "$THING" > "$T/thing.txt"
says "$T/thing.txt" "(-4, 9, 24)"
"$P" set "$J" "Test lavatory" height "34 in" > "$T/h34.txt"
# The cold pipe is joined again; the hot pipe, 3 in short of its connection in step 4, stood off all along and still does, and is said so (call 213 point 2).
says "$T/h34.txt" "1 run is now joined to its connection"
says "$T/h34.txt" "1 run still stands off its connection"
"$P" takeoff "$J" > "$T/take.txt"
lacks "$T/take.txt" "its Cold water connection"

echo
echo "== 6. The thing turned a quarter turn (90 degrees, counter-clockwise from east): its connections turn with it =="
echo "      cold goes from (9.6667, 20.75) to (9.25, 19.6667): x = 10 + (-1/3 * 0 - 0.75 * 1), y = 20 + (-1/3 * 1 + 0.75 * 0); it stands off its pipe now"
"$P" turn "$J" "$THING" --by 90 "${A[@]}" | sed 's/^/    | /'
"$P" takeoff "$J" > "$T/take.txt"
says "$T/take.txt" "its Cold water connection"
"$P" turn "$J" "$THING" --by -90 "${A[@]}" >/dev/null
"$P" takeoff "$J" > "$T/take.txt"
lacks "$T/take.txt" "its Cold water connection"

echo
echo "== 7. Refused, each naming the item, the connection and the figure in inches =="
refuse() { if "$P" "$@" > "$T/no.txt" 2>&1; then echo "FAIL: not refused: $*"; cat "$T/no.txt"; exit 1; fi; }
refuse connection "$J" "Test lavatory" add "Vent" --takes pipe --carries vent --goes out --across 0 --back 0 --up "height + 2 in" --faces up "${A[@]}"
says "$T/no.txt" "Test lavatory"
says "$T/no.txt" "the Vent connection's Up figure, height + 2 in, puts it 2 in above the item (24 in wide, 18 in deep, 34 in high)"
refuse connection "$J" "Test lavatory" add "cold water" --takes pipe --carries cold --goes in --across 0 --back 0 --up "10 in" --faces back "${A[@]}"
says "$T/no.txt" "has a connection called \"Cold water\" already"
refuse connection "$J" "Test lavatory" add "Cold duct" --takes duct --carries cold --goes in --across 0 --back 0 --up "10 in" --faces back "${A[@]}"
says "$T/no.txt" "only a pipe connection says what it carries"
refuse connection "$J" "Test lavatory" add "Vent" --takes pipe --carries vent --goes out --across 0 --back 0 --up "10 in" --faces up --status Known --source "a person's say-so"
says "$T/no.txt" "a place is Known only where somebody measures it on the thing"
for w in " ft" "coordinate" "small size" "port " "connector"; do lacks "$T/no.txt" "$w"; done

echo
echo "== 8. The rows written out and brought into a new job with the same item: the two tables alike, byte for byte =="
"$P" export connections "$J" "$T/out1.csv" | sed 's/^/    | /'
"$P" new "$T/second.json" "Second (not a real job)" >/dev/null
"$P" import items "$T/second.json" "$T/lav.csv" >/dev/null
"$P" import connections "$T/second.json" "$T/out1.csv" > "$T/in.txt"
says "$T/in.txt" "Test lavatory: added: 3 connections"
"$P" export connections "$T/second.json" "$T/out2.csv" >/dev/null
cmp "$T/out1.csv" "$T/out2.csv" && echo "    the two tables are the same, byte for byte"
"$P" import connections "$T/second.json" "$T/out1.csv" > "$T/again.txt"
says "$T/again.txt" "the same as the job already has: 1"
# The starter rows go to an item that has none, and leave an item that has some as it is.
"$P" new "$T/starter.json" "Starter (not a real job)" >/dev/null
"$P" import items "$T/starter.json" starter >/dev/null
"$P" import connections "$T/starter.json" starter > "$T/st.txt"
says "$T/st.txt" "Water closet: added: 2 connections"
says "$T/st.txt" "Lavatory: added: 3 connections"
"$P" import connections "$T/starter.json" starter > "$T/st2.txt"
says "$T/st2.txt" "items that have connections already, and keep them: 2"
"$P" items "$T/starter.json" > "$T/sti.txt"
says "$T/sti.txt" "have none"
# A chest brought in again, with no rows in it, changes nothing of the rows.
"$P" import items "$T/second.json" "$T/lav.csv" >/dev/null
"$P" export connections "$T/second.json" "$T/out3.csv" >/dev/null
cmp "$T/out1.csv" "$T/out3.csv" && echo "    an item table brought in again leaves the rows as they were"

echo
echo "== 9. The building written out as a BIM model and read back: three connection points in the file, three when read =="
"$P" export ifc "$J" "$T/lav.ifc" > "$T/ifc.txt"
cat "$T/ifc.txt" | sed 's/^/    | /'
n=$(grep -c "IFCDISTRIBUTIONPORT" "$T/lav.ifc" || true)
[ "$n" -ge 3 ] || { echo "FAIL: the file has $n connection points on the lavatory and the pipes' ends; wanted at least 3"; exit 1; }
echo "    the file holds $n connection points (3 on the lavatory, 2 on each of 2 pipes)"
says "$T/ifc.txt" "3 connections: which way it faces is not written"
"$P" new "$T/back.json" "Read back (not a real job)" >/dev/null
"$P" import ifc "$T/back.json" "$T/lav.ifc" --status Assumed --source "a made-up model" > "$T/read.txt" 2>&1 || { cat "$T/read.txt"; exit 1; }
says "$T/read.txt" "3 connections on fixtures and equipment were brought in from the file's connection points"
sed 's/^/    | /' "$T/read.txt" | head -12
"$P" show "$T/back.json" > "$T/shown.txt"
k=$(grep -c "^ *~ connection [0-9]*: " "$T/shown.txt" || true)
[ "$k" -eq 3 ] || { echo "FAIL: the lavatory read back has $k connections, not three"; head -60 "$T/shown.txt"; exit 1; }
echo "    read back, the lavatory has $k connections"

echo
echo "== 10. The sample whole house is as it was: plumbline sixteen byte for byte, and its price and open ends =="
"$P" sixteen "$T/sixteen.json" >/dev/null
echo "880d9fd3c364b5f77ad2abc666dde290955efd6a971f748f65fab8e00292b82a  $T/sixteen.json" | sha256sum -c --quiet || { echo "FAIL: the sample whole house is not the job it was (written before this work)"; exit 1; }
echo "    plumbline sixteen: the same job, byte for byte (sha256 880d9fd3...)"
"$P" takeoff "$T/sixteen.json" > "$T/sixteen-take.txt"
"$P" estimate "$T/sixteen.json" > "$T/sixteen-est.txt"
says "$T/sixteen-est.txt" "468,050.22"
echo
echo "all ten steps held"
