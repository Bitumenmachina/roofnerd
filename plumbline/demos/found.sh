#!/usr/bin/env bash
# The foundation, drawn: strip footings and the foundation walls that stand on them, on
# a level of their own under the first floor, from an empty job at the command line.
# What each is, how long, and the concrete in the footings by the cubic yard; the
# level's takeoff with footings and walls counted apart; a footing priced by the CY;
# the saved job read back; and the BIM file with its footings as footings.
# Run from the repository root:  demos/found.sh
# The job is made up (no real job data) and lives in a temporary folder. Every step fails
# the script if the program does not do what the step says. The figures expected are
# worked by hand in STATUS.md ("The foundation drawn") and in the comments here.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up foundation plan")
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; cat "$1"; exit 1; }; printf '    %s\n' "$2"; }
not_said() { ! grep -qF -- "$2" "$1" || { echo "FAIL: it says: $2"; cat "$1"; exit 1; }; }
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }

echo "== A house 40 ft by 28 ft: a level for its foundation, 4 ft under the first floor =="
"$P" new "$J" "Foundation (not a real job)" >/dev/null
L="$(made "$("$P" level "$J" foundation --elevation "-4 ft" --height "4 ft" "${K[@]}")")"
FT="$(made "$("$P" type "$J" footing "footing 24 x 12" --width "24 in" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "foundation wall, 8 in concrete" --thickness "8 in" "${K[@]}")")"

echo "== Four strip footings on the walls' centre lines, 24 in wide and 12 in deep =="
"$P" footing "$J" "$L" "$FT" --points "0,0 40,0 40,28 0,28" --closed --depth "12 in" "${K[@]}" > "$T/made.txt"
says "$T/made.txt" "4 footings: "
F1="$(sed -E 's/^4 footings: ([a-z]+-[0-9]+),.*/\1/' "$T/made.txt")"

echo "== The four foundation walls on the same lines, 4 ft high: each stands on its footing =="
"$P" wall "$J" "$L" "$WT" --points "0,0 40,0 40,28 0,28" --closed --height "4 ft" "${K[@]}" > "$T/walls.txt"
says "$T/walls.txt" "4 walls: "

echo "== One footing, as the job lists it =="
"$P" show "$J" > "$T/show.txt"
sed -n "/^ *$F1  /,/^ *footing-[0-9]*  footing 2/p" "$T/show.txt" | sed '$d' > "$T/one.txt"
says "$T/one.txt" "[footing]"
says "$T/one.txt" "footing line     (0, 0) to (40, 0) ft"
says "$T/one.txt" "depth            12 in"
# Its width is its type's; its volume is 40 x 2 x 1 / 27 = 2.963 CY.
says "$T/one.txt" "=   plan length      40 LF"
says "$T/one.txt" "=   width            2 ft"
says "$T/one.txt" "=   depth            1 ft"
says "$T/one.txt" "=   volume           2.963 CY"
# A footing is listed in a footing's words, never a wall's.
not_said "$T/one.txt" "wall line"
not_said "$T/one.txt" "height"
not_said "$T/one.txt" "thickness"
not_said "$T/one.txt" "bounds rooms"

echo "== The level's takeoff: walls and footings apart =="
"$P" takeoff "$J" > "$T/takeoff.txt"
# Walls: 40 + 28 + 40 + 28 = 136 LF, 136 x 4 = 544 SF. The footings are not in them.
says "$T/takeoff.txt" "walls 4            136.00 LF"
says "$T/takeoff.txt" "wall face, gross   544.00 SF"
# Footings: 136 LF; 136 x 2 x 1 = 272 CF, over 27: 10.07 CY.
says "$T/takeoff.txt" "footings 4         136.00 LF"
says "$T/takeoff.txt" "footing concrete   10.07 CY"

echo "== Nothing built is tested against itself: no clash between a wall and its footing, or at the corners =="
"$P" clashes "$J" > "$T/clashes.txt"
says "$T/clashes.txt" "In the same space: 0"
says "$T/clashes.txt" "a footing is tested against neither, nor against another footing"

echo "== A pipe through a footing is a clash, not a hole to cut =="
# A 2 in pipe run level through the south footing, 6 in below its top (-4 ft 6 in).
PT="$(made "$("$P" type "$J" pipe "2 in pipe" --round "2 in" "${K[@]}")")"
"$P" run "$J" "$L" pipe "$PT" --points "10,-3 10,3" --height "-6 in" "${K[@]}" >/dev/null
"$P" clashes "$J" > "$T/pipe.txt"
says "$T/pipe.txt" "In the same space: 1"
says "$T/pipe.txt" "Through a wall or a floor (a hole to cut): 0"
PIPE="$("$P" show "$J" | sed -n 's/^ *\(pipe-[0-9]*\) .*/\1/p' | head -1)"
"$P" remove "$J" "$PIPE" >/dev/null

echo "== One footing drawn over another is refused, in a footing's words =="
if "$P" footing "$J" "$L" "$FT" --points "5,0 20,0" --depth "12 in" "${K[@]}" > "$T/over.txt" 2>&1; then echo "FAIL: a footing over a footing was taken"; exit 1; fi
says "$T/over.txt" "lies along $F1 for 15.00 ft: one footing drawn over another would be counted twice"

echo "== A footing with no depth is refused; nothing is given a depth nobody entered =="
if "$P" footing "$J" "$L" "$FT" --points "50,0 60,0" "${K[@]}" > "$T/nodepth.txt" 2>&1; then echo "FAIL: a footing with no depth was taken"; exit 1; fi
says "$T/nodepth.txt" "--depth"

echo "== A footing that stops on the run of another (a T) does not count that concrete twice =="
# A second job: a run of 40 ft and a stem of 20 ft standing on its middle, both 24 x 12.
# By hand: the run 40 x 2 x 1 = 80 CF. The stem's first foot lies inside the run (half
# the run's width), so the stem is 19 x 2 x 1 = 38 CF. 118 CF = 4.37 CY, not 120.
TJ="$T/tee.json"
"$P" new "$TJ" "A T of footings (not a real job)" >/dev/null
TL="$(made "$("$P" level "$TJ" foundation --elevation "-4 ft" --height "4 ft" "${K[@]}")")"
TT="$(made "$("$P" type "$TJ" footing "footing 24 x 12" --width "24 in" "${K[@]}")")"
"$P" footing "$TJ" "$TL" "$TT" --points "0,0 40,0" --depth "12 in" "${K[@]}" >/dev/null
"$P" footing "$TJ" "$TL" "$TT" --points "20,0 20,20" --depth "12 in" "${K[@]}" >/dev/null
"$P" takeoff "$TJ" > "$T/tee.txt"
says "$T/tee.txt" "footings 2         60.00 LF"
says "$T/tee.txt" "footing concrete   4.37 CY"
"$P" show "$TJ" > "$T/teeshow.txt"
# The stem: 19 x 2 / 27 = 1.4074 CY; its length is still the 20 ft that was drawn.
says "$T/teeshow.txt" "=   volume           1.4074 CY"
says "$T/teeshow.txt" "=   plan length      20 LF"

echo "== A footing is spoken of as a footing: its type, its depth measured in the field, what is refused =="
"$P" show "$TJ" > "$T/words.txt"
says "$T/words.txt" "footing width    24 in"
not_said "$T/words.txt" "wall thickness"
TF="$(sed -n 's/^ *\(footing-[0-9]*\) .*/\1/p' "$T/words.txt" | head -1)"
# The run, dug 18 in deep where the plan said 12: 40 x 2 x 1.5 / 27 = 4.4444 CY.
"$P" measure "$TJ" "$TF" depth "18 in" --source "made-up dig" > "$T/dug.txt"
says "$T/dug.txt" "depth is 18 in, measured"
"$P" show "$TJ" > "$T/words.txt"
says "$T/words.txt" "=   volume           4.4444 CY"
if "$P" footing "$TJ" "$TL" "$TT" --points "5,5 5,5" --depth "12 in" "${K[@]}" > "$T/point.txt" 2>&1; then echo "FAIL: a footing with no length was taken"; exit 1; fi
says "$T/point.txt" "footing line: the two ends are the same point"
if "$P" type "$TJ" footing "no width" --width "0 in" "${K[@]}" > "$T/nowidth.txt" 2>&1; then echo "FAIL: a footing type with no width was taken"; exit 1; fi
says "$T/nowidth.txt" "footing type width: must be more than zero"

echo "== Priced by the cubic yard =="
"$P" estimate "$J" > "$T/before.txt" || true
says "$T/before.txt" "Nothing in this job has a cost line."
"$P" line "$J" "$F1" "Concrete footing, in place" volume --cost-type subcontract --division 3 > "$T/line.txt"
says "$T/line.txt" "= volume, with no price yet; Division 3, Concrete"
"$P" set "$J" "$F1" unit_cost '$185.00/CY' --line "Concrete footing, in place" --status Known --source "made-up price" >/dev/null
"$P" estimate "$J" > "$T/after.txt" || true
# 2.9630 CY at $185.00: $548.155, to the cent $548.16.
says "$T/after.txt" "2.9630 CY       \$185.00/CY"
says "$T/after.txt" "\$548.16"
says "$T/after.txt" "nothing is priced on 7 things that could be (3 footings, 4 walls)"

echo "== The saved job reads back the same =="
"$P" check "$J" > "$T/check.txt"
says "$T/check.txt" "reads, its figures agree with what is drawn, and saving it back writes the same file"

echo "== The BIM file: four footings as footings, four walls as walls =="
"$P" export ifc "$J" "$T/f.ifc" > "$T/ifc.txt"
says "$T/ifc.txt" "written: 4 footings, 1 level, 4 walls"
[ "$(grep -c 'IFCFOOTING(' "$T/f.ifc")" = 4 ] || { echo "FAIL: not four IfcFooting"; exit 1; }
[ "$(grep -c 'STRIP_FOOTING' "$T/f.ifc")" = 4 ] || { echo "FAIL: not four strip footings"; exit 1; }
if python3 -c 'import ifcopenshell' 2>/dev/null; then
  python3 tools/ifc-check.py "$T/f.ifc" > "$T/valid.txt"
  says "$T/valid.txt" "no fault against the schema and its rules"
else
  echo "    (IfcOpenShell is not on this machine: the file was not validated; set PYTHONPATH to where it is)"
fi

echo
echo "PASS: footings and foundation walls were drawn on a level of their own; 136 LF and 10.07 CY of footing, 544 SF of wall, counted apart; a footing priced by the CY; no clash among what is built; the job reads back; the BIM file has four strip footings"
