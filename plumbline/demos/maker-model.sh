#!/usr/bin/env bash
# A maker's model brought in as an item (call 214, contract 214M). At the command line. Every maker, product and figure is made up; the
# files are kept in demos/maker-model/ and each was passed by IfcOpenShell run as an outside checker (tools/ifc-check.py) before the
# program read it. By hand:
#  1. A new job; file 1 (a basin, 508 x 457.2 x 203.2 mm, no building, no level) brought in: 1 item "Northgate NL-2018", a fixture, 20 in
#     wide, 18 in deep, 8 in high, Assumed and naming the file; maker Northgate Fixtures and model NL-2018, Assumed; nothing set down.
#  2. Its three connections: cold water 4 in across (355.6 less 254 is 101.6 mm), 0 at the back, 2 in up; hot water -4 in across; waste on
#     the middle, at the back, at the foot, facing down.
#  3. The reply says it is drawn as a box and the body's own shape was not kept.
#  4. A level made and the item set down: takeoff names its three connections open.
#  5. The same file again: refused by name, the job byte for byte what it was.
#  6. File 2, the same product as a proxy: left out and named, saying how to say what it is; with --as fixture, 1 item.
#  7. File 3, its shape shared by its type, two of them: 1 item, and the reply says 2 of that type.
#  8. File 4, a body turned about an axis (not read): the item comes in with its three sizes Unknown, saying why, and not drawn.
#  9. File 5, no manufacturer's set: maker and model Unknown, "not said in the model".
# 10. File 1 with a fourth point 100 mm (3 15/16 in) behind the body: three rows, the fourth named as outside and not written.
# Run from the repository root:  demos/maker-model.sh   Every step fails the script if the program does not say what it expects.
set -euo pipefail
cd "$(dirname "$0")/.."
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
F="$PWD/demos/maker-model"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
A=(--status Assumed --source "made-up job")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; cat "$1"; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
newjob() { "$P" new "$1" "A job (not a real one)" >/dev/null; }

echo "== 1. File 1 into a new job: one item of its sizes, its maker and model; no level, nothing set down =="
J="$T/j.json"; newjob "$J"
"$P" import maker-model "$J" "$F/nl-2018.ifc" > "$T/1.txt"
cat "$T/1.txt"
says "$T/1.txt" "Northgate NL-2018, a fixture"
says "$T/1.txt" "~ width   20 in"
says "$T/1.txt" "~ depth   18 in"
says "$T/1.txt" "~ height  8 in"
says "$T/1.txt" "nl-2018.ifc"
says "$T/1.txt" "the extent of its body"
says "$T/1.txt" "~ maker   Northgate Fixtures"
says "$T/1.txt" "~ model   NL-2018"
says "$T/1.txt" "Pset_ManufacturerTypeInformation, Manufacturer"
"$P" show "$J" > "$T/show.txt"
lacks "$T/show.txt" "[level]"
lacks "$T/show.txt" "[fixture]"

echo "== 2. Its three connections =="
"$P" items "$J" "Northgate NL-2018" > "$T/items.txt"
says "$T/items.txt" "Cold water: pipe, cold, in; across 4 in, back 0 in, up 2 in; faces back; Assumed"
says "$T/items.txt" "Hot water: pipe, hot, in; across -4 in, back 0 in, up 2 in; faces back; Assumed"
says "$T/items.txt" "Waste: pipe, waste, out; across 0 in, back 0 in, up 0 in; faces down; Assumed"
says "$T/items.txt" "nl-2018.ifc"

echo "== 3. It is drawn as a box, and the body's own shape was not kept =="
says "$T/1.txt" "drawn as a box of its sizes"
says "$T/1.txt" "the body's own shape was not kept"

echo "== 4. Set down on a level: takeoff names its three connections open =="
L="$("$P" level "$J" "Level 1" --elevation "0 ft" --height "9 ft" "${A[@]}" | grep -oE 'level-[0-9]+' | head -1)"
IT="$(grep -oE '^type-[0-9]+' "$T/1.txt" | head -1)"
"$P" place "$J" "$L" "$IT" --at "10,20" "${A[@]}" > "$T/place.txt"
"$P" takeoff "$J" > "$T/take.txt"
says "$T/take.txt" "Cold water"
says "$T/take.txt" "Hot water"
says "$T/take.txt" "Waste"

echo "== 5. The same file again: refused by name; the job is byte for byte what it was =="
cp "$J" "$T/before.json"
if "$P" import maker-model "$J" "$F/nl-2018.ifc" > "$T/5.txt" 2>&1; then echo "FAIL: it took the same file twice"; exit 1; fi
says "$T/5.txt" "refused: Northgate NL-2018"
says "$T/5.txt" "already"
cmp "$J" "$T/before.json" || { echo "FAIL: the job changed"; exit 1; }
printf '    the job is the same, byte for byte\n'

echo "== 6. A proxy: left out and named; with --as fixture, one item =="
J="$T/j6.json"; newjob "$J"; cp "$J" "$T/before6.json"
if "$P" import maker-model "$J" "$F/nl-2018-proxy.ifc" > "$T/6a.txt" 2>&1; then echo "FAIL: a proxy was taken without being told what it is"; exit 1; fi
says "$T/6a.txt" "1 thing of no stated kind"
says "$T/6a.txt" "--as fixture"
cmp "$J" "$T/before6.json" || { echo "FAIL: the job changed"; exit 1; }
"$P" import maker-model "$J" "$F/nl-2018-proxy.ifc" --as fixture > "$T/6b.txt"
says "$T/6b.txt" "Northgate NL-2018, a fixture (as you said)"
says "$T/6b.txt" "~ width   20 in"

echo "== 7. A shape shared by its type, two of the type: one item, and the reply says two =="
J="$T/j7.json"; newjob "$J"
"$P" import maker-model "$J" "$F/nl-2018-shared.ifc" > "$T/7.txt"
says "$T/7.txt" "2 of that type are in the file, and one item is made of them"
says "$T/7.txt" "~ width   20 in"
says "$T/7.txt" "~ height  8 in"
says "$T/7.txt" "1 item made"

echo "== 8. A body turned about an axis: sizes Unknown, saying why; not drawn =="
J="$T/j8.json"; newjob "$J"
"$P" import maker-model "$J" "$F/np-1-turned.ifc" > "$T/8.txt"
says "$T/8.txt" "? width"
says "$T/8.txt" "a solid turned about an axis"
says "$T/8.txt" "it is not drawn until it has sizes"
lacks "$T/8.txt" "drawn as a box of its sizes"

echo "== 9. No manufacturer's set: maker and model Unknown, not said in the model =="
J="$T/j9.json"; newjob "$J"
"$P" import maker-model "$J" "$F/nl-2018-no-maker.ifc" > "$T/9.txt"
says "$T/9.txt" "? maker   not said in the model"
says "$T/9.txt" "? model   not said in the model"

echo "== 10. A fourth connection point 100 mm behind the body: three rows; the fourth named, not written =="
J="$T/j10.json"; newjob "$J"
"$P" import maker-model "$J" "$F/nl-2018-outside.ifc" > "$T/10.txt"
says "$T/10.txt" "not written: Overflow: it stands outside the body, 3 15/16 in behind it"
says "$T/10.txt" "3 connections"
lacks "$T/10.txt" "Overflow: pipe"

echo "== a thing that is no model is refused =="
if "$P" import maker-model "$J" "$F/../../README.md" > "$T/bad.txt" 2>&1; then echo "FAIL: a README was taken for a model"; exit 1; fi
cat "$T/bad.txt"
echo "maker-model: all steps passed"
