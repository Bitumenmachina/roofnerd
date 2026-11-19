#!/usr/bin/env bash
# Foundation work taken off from the starter library, line by line as a concrete sub's
# estimator writes it: a ring of strip footings and the poured walls on them, each
# given its assembly, every figure worked in the program and checked here against the
# figure worked by hand; dampproofing with no figure until the height to grade is given;
# a foundation drain only where it is put; and the IFC model carrying the same figures
# as each footing's and wall's base quantities.
# Run from the repository root:  demos/foundation.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures are worked by
# hand in STATUS.md ("Foundation work") and in the comments here; the code rules behind
# the assemblies are in DECISIONS.md call 132.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up foundation plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
# One element's block of the job as `show` lists it, from its path to the next element.
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }

echo "== A foundation 40 ft by 28 ft on the walls' center lines: a level 4 ft deep, footings 20 in by 10 in, walls 8 in by 4 ft =="
"$P" new "$J" "Foundation (not a real job)" >/dev/null
L="$(made "$("$P" level "$J" foundation --elevation "-4 ft" --height "4 ft" "${K[@]}")")"
FT="$(made "$("$P" type "$J" footing "footing 20 x 10" --width "20 in" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "foundation wall, 8 in" --thickness "8 in" "${K[@]}")")"
read -r F_S F_E F_N F_W <<<"$("$P" footing "$J" "$L" "$FT" --points "0,0 40,0 40,28 0,28" --closed --depth "10 in" "${K[@]}" | grep -oE 'footing-[0-9]+' | tr '\n' ' ')"
read -r W_S W_E W_N W_W <<<"$("$P" wall "$J" "$L" "$WT" --points "0,0 40,0 40,28 0,28" --closed --height "4 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
"$P" check "$J"

echo
echo "== The starter library comes in, and each footing and wall is given its assembly =="
"$P" import assemblies "$J" starter > "$T/in.txt"
says "$T/in.txt" "set: 35"
says "$T/in.txt" "not used: 0"
for f in "$F_S" "$F_E" "$F_N" "$F_W"; do "$P" apply "$J" "$f" "Strip footing, formed, two #4 bars" --source "made-up foundation plan" >/dev/null; done
for w in "$W_S" "$W_E" "$W_N" "$W_W"; do "$P" apply "$J" "$w" "Foundation wall, poured concrete" --source "made-up foundation plan" >/dev/null; done
"$P" check "$J"

echo
echo "== The south footing (40 ft) and its lines, against the figures worked by hand =="
"$P" show "$J" > "$T/show.txt"
block "$F_S" > "$T/fs.txt"
says "$T/fs.txt" "quantity: Trench for the footing 40 LF"
# 2 x 40 x 10/12 = 66.6667 SF (call 133: worked from 10 in, not from 0.8333 ft).
says "$T/fs.txt" "quantity: Footing forms, both sides 66.6667 SF"
# 40 x 20/12 x 10/12 / 27 = 2.0576 CY; 5 % over: 2.1605 CY.
says "$T/fs.txt" "quantity: Footing concrete 2.0576 CY"
says "$T/fs.txt" "with waste: Footing concrete 2.1605 CY"
# Two bars: 80 LF; 12.5 % for laps: 90 LF; at 0.668 lb/LF: 60.12 lb.
says "$T/fs.txt" "quantity: Footing bars, #4, continuous 80 LF"
says "$T/fs.txt" "with waste: Footing bars, #4, continuous 90 LF"
says "$T/fs.txt" "to order: Footing bars, #4, continuous 60.12 lb"
# What a code rule set is said to be Assumed, and where it came from.
says "$T/fs.txt" "~ bars: Footing bars, #4, continuous 2"
says "$T/fs.txt" "~ waste: Footing concrete 5 %"

echo
echo "== The east footing (28 ft) =="
block "$F_E" > "$T/fe.txt"
# 2 x 28 x 10/12 = 46.6667; 28 x 20/12 x 10/12 / 27 = 1.4403, + 5 % = 1.5123;
# 56 LF of bar, 63 LF with laps, 42.084 lb.
says "$T/fe.txt" "quantity: Footing forms, both sides 46.6667 SF"
says "$T/fe.txt" "quantity: Footing concrete 1.4403 CY"
says "$T/fe.txt" "with waste: Footing concrete 1.5123 CY"
says "$T/fe.txt" "to order: Footing bars, #4, continuous 42.084 lb"

echo
echo "== The south wall (40 ft by 4 ft, 8 in) =="
block "$W_S" > "$T/ws.txt"
says "$T/ws.txt" "thickness        0.6667 ft"
# Forms both faces: 2 x 160 = 320 SF.
says "$T/ws.txt" "quantity: Wall forms, both faces 320 SF"
# 160 x 8/12 / 27 = 3.9506 CY; 5 % over: 4.1481 CY.
says "$T/ws.txt" "quantity: Wall concrete 3.9506 CY"
says "$T/ws.txt" "with waste: Wall concrete 4.1481 CY"
# Two rows: 80 LF, 90 LF with laps, 60.12 lb. Verticals at 48 in: (10 + 1) x 4 = 44 LF, 29.392 lb.
says "$T/ws.txt" "to order: Horizontal bars, #4 60.12 lb"
says "$T/ws.txt" "quantity: Vertical bars, #4 44 LF"
says "$T/ws.txt" "to order: Vertical bars, #4 29.392 lb"
# Bolts: (40 - 2) / 6 + 1 = 7.33, a count: 8.
says "$T/ws.txt" "quantity: Anchor bolts, 1/2 in, with nut and washer 8 EA"
# Dampproofing waits for the height to grade: no figure, never a guess.
says "$T/ws.txt" "? quantity: Dampproofing, outside face unknown"

echo
echo "== The east wall (28 ft) =="
block "$W_E" > "$T/we.txt"
# 112 x 8/12 / 27 = 2.7654, + 5 % = 2.9037; verticals (7 + 1) x 4 = 32 LF; bolts 26 / 6 + 1 = 5.33: 6.
says "$T/we.txt" "quantity: Wall concrete 2.7654 CY"
says "$T/we.txt" "with waste: Wall concrete 2.9037 CY"
says "$T/we.txt" "quantity: Vertical bars, #4 32 LF"
says "$T/we.txt" "quantity: Anchor bolts, 1/2 in, with nut and washer 6 EA"

echo
echo "== The estimate by item: the ring of four, each item once =="
"$P" estimate "$J" by-item | tr -s ' ' > "$T/items.txt"
# 136 LF; 226.6667 SF of footing forms; 7.3457 CY of footing concrete (4 x the two above);
# 204.408 lb of footing bars; 1,088 SF of wall forms; 14.1037 CY of wall concrete;
# 101.536 lb of verticals; 28 bolts. With no price on them they are shown to two places.
says "$T/items.txt" "Trench for the footing Subcontract 4 136.00 LF"
says "$T/items.txt" "Footing forms, both sides Subcontract 4 226.67 SF"
says "$T/items.txt" "Footing concrete Material 4 ~7.35 CY"
says "$T/items.txt" "Footing bars, #4, continuous Material 4 ~204.41 lb"
says "$T/items.txt" "Wall forms, both faces Subcontract 4 1,088.00 SF"
says "$T/items.txt" "Wall concrete Material 4 ~14.10 CY"
says "$T/items.txt" "Vertical bars, #4 Material 4 ~101.54 lb"
says "$T/items.txt" "Anchor bolts, 1/2 in, with nut and washer Material 4 ~28 EA"
says "$T/items.txt" "Dampproofing, outside face Subcontract 4 no figure"

echo
echo "== The height to grade is given on the walls: 3 ft 4 in (the 4 ft wall less 8 in above grade) =="
for w in "$W_S" "$W_E" "$W_N" "$W_W"; do
  "$P" set "$J" "$w" below_grade "3 ft 4 in" --line "Dampproofing, outside face" --status Known --source "made-up grading plan" >/dev/null
done
"$P" show "$J" > "$T/show.txt"
block "$W_S" > "$T/ws.txt"
block "$W_E" > "$T/we.txt"
# 40 x 3 1/3 = 133.3333 SF; 28 x 3 1/3 = 93.3333 SF; 453.3333 SF round.
says "$T/ws.txt" "quantity: Dampproofing, outside face 133.3333 SF"
says "$T/we.txt" "quantity: Dampproofing, outside face 93.3333 SF"

echo
echo "== A basement wall has a foundation drain; a crawl space's foundation wall does not =="
# The south wall is changed to a basement wall: every line the two share stays as it is,
# and the drain is added: 40 LF.
"$P" apply "$J" "$W_S" "Basement wall, poured concrete, with a drain" --change --source "made-up foundation plan: the south wall encloses a finished room" >/dev/null
"$P" estimate "$J" by-item | tr -s ' ' > "$T/items.txt"
says "$T/items.txt" "Foundation drain, perforated pipe in gravel Subcontract 1 40.00 LF"
"$P" show "$J" > "$T/show.txt"
block "$W_S" > "$T/ws.txt"
says "$T/ws.txt" "quantity: Dampproofing, outside face 133.3333 SF"

echo "== A price typed on the footing concrete: what it comes to, footing by footing and round the ring =="
for f in "$F_S" "$F_E" "$F_N" "$F_W"; do
  "$P" set "$J" "$f" unit_cost '$150/CY' --line "Footing concrete" --status Known --source "made-up price" >/dev/null
done
"$P" estimate "$J" | tr -s ' ' > "$T/est.txt"
# 2.1605 x 150 = 324.075 -> $324.08 on each 40 ft footing; 1.5123 x 150 = 226.845 -> $226.85
# on each 28 ft one: 2 x 324.08 + 2 x 226.85 = $1,101.86.
says "$T/est.txt" "\$324.08"
says "$T/est.txt" "\$226.85"
says "$T/est.txt" "~\$1,101.86"

echo
echo "== Out to IFC: each footing and wall carries the same figures as its base quantities =="
"$P" export ifc "$J" "$T/f.ifc" > "$T/ifc.txt"
python3 -I - "$T/f.ifc" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
sets = re.findall(r"IFCELEMENTQUANTITY\('[^']*',\$,'([^']+)'", text)
assert sets.count("Qto_FootingBaseQuantities") == 4, sets
assert sets.count("Qto_WallBaseQuantities") == 4, sets
q = dict()
for name, value in re.findall(r"IFCQUANTITY(?:LENGTH|AREA|VOLUME)\('(\w+)','(?:Known|Assumed)',\$,([0-9.]+),", text):
    q.setdefault(name, []).append(float(value))
# A 40 ft footing's volume is 40 x 20/12 x 10/12 = 55.5556 CF (2.0576 CY), written in cubic feet.
assert 55.5556 in q["NetVolume"], q["NetVolume"]
assert sorted(set(q["GrossSideArea"])) == [112.0, 160.0], q["GrossSideArea"]
assert 0.6667 in q["Width"] and 1.6667 in q["Width"], q["Width"]
print("    4 Qto_FootingBaseQuantities, 4 Qto_WallBaseQuantities; a 40 ft footing's NetVolume 55.5556 CF (2.0576 CY); wall side areas 160 and 112 SF; widths 0.6667 and 1.6667 ft")
PY
if command -v python3 >/dev/null && python3 -c 'import ifcopenshell' 2>/dev/null; then
  python3 -I - "$T/f.ifc" <<'PY'
import sys, ifcopenshell, ifcopenshell.validate
f = ifcopenshell.open(sys.argv[1])
log = ifcopenshell.validate.json_logger()
ifcopenshell.validate.validate(f, log, express_rules=True)
assert not log.statements, log.statements[:3]
import ifcopenshell.util.element as el
footing = f.by_type("IfcFooting")[0]
qto = el.get_psets(footing, qtos_only=True)
assert "Qto_FootingBaseQuantities" in qto, qto
print("    IfcOpenShell reads it: valid, with the express rules; a footing's quantities:", {k: v for k, v in qto["Qto_FootingBaseQuantities"].items() if k != "id"})
PY
else
  echo "    (IfcOpenShell is not installed here: the model was not validated by it)"
fi

echo
echo "  PASS: a foundation was taken off from the starter library to the figures worked by hand, line by line; dampproofing waited for the height to grade; a drain went only where it was put; and the IFC model carries the same figures"
