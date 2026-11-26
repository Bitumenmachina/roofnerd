#!/usr/bin/env bash
# A CAD drawing (DXF) in and out, at the command line (DECISIONS.md call 221). The drawings are made up and kept
# beside this script, in demos/dxf: a room 20 ft by 12 ft drawn in inches (plan-inches.dxf), the same with no unit said
# (plan-no-units.dxf), and the same with things a sheet cannot carry (plan-with-extras.dxf). Run from the repository root:
#   demos/dxf.sh
# Every step fails the script if the program does not do what the step says.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and nothing is built.
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
D="$PWD/demos/dxf"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made up")
READ=(--status Assumed --source "the drawing")
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3"; exit 1; }; printf '    %s\n' "$2"; }
is() { [ "$2" = "$3" ] || { echo "FAIL: $1: got $2, wanted $3"; exit 1; }; printf '    %-60s %s\n' "$1" "$2"; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $(head -1 <<<"$out" | cut -c1-200)"
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; echo "$out"; exit 1; }
}
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
drawing_fact() { # $1 drawing path, $2 python expression of d
  python3 -I - "$J" "$1" "$2" <<'PY'
import sys, json
e = next(x for x in json.load(open(sys.argv[1]))["data"] if x["path"] == sys.argv[2])
d = e["attributes"]["pl::drawing"]
print(eval(sys.argv[3]))
PY
}
dxf_count() { # $1 file, $2 layer, $3 entity: how many of that entity on that layer, in the text form
  python3 -I - "$1" "$2" "$3" <<'PY'
import sys
t = open(sys.argv[1]).read().split("\n")
pairs = [(t[i].strip(), t[i + 1].strip()) for i in range(0, len(t) - 1, 2)]
n = 0; ent = None
for c, v in pairs:
    if c == "0":
        ent = v; layer = None
    elif c == "8" and ent == sys.argv[3] and v == sys.argv[2]:
        n += 1
print(n)
PY
}

echo "== 1. A drawing in inches comes in at true size, with nothing to calibrate =="
"$P" new "$J" "A made-up room" >/dev/null
LEVEL="$(made "$("$P" level "$J" "First Floor" --elevation "0 ft" --height "9 ft" "${K[@]}")")"
"$P" type "$J" wall "Wall 6" --thickness "6 in" "${K[@]}" >/dev/null
WT=type-0003
cp "$D/plan-inches.dxf" "$T/plan-inches.dxf"
OUT="$("$P" drawing "$J" "$LEVEL" "Kitchen plan" --file "$T/plan-inches.dxf" "${READ[@]}")"
echo "$OUT" | sed 's/^/    | /'
SHEET="$(made "$OUT")"
says "true size" "at true size, nothing to calibrate: it covers 20' 0\" by 12' 0\"" "$OUT"
says "scale is Assumed, from the file" "its scale is Assumed, from the file's own unit (inches" "$OUT"
is "the scale's status in the job" "$(drawing_fact "$SHEET" 'd["scale"]["distance"]["status"]')" "Assumed"
case "$(drawing_fact "$SHEET" 'd["scale"]["distance"]["source"]')" in *"the file's own unit: inches"*) printf '    %-60s %s\n' "the scale's source" "names the file's own unit";; *) echo "FAIL: scale source"; exit 1;; esac
WALL="$("$P" wall "$J" "$LEVEL" "$WT" --points "0,0 240,0" --height "9 ft" --on "$SHEET")"
echo "    | $WALL"
TAKEOFF="$("$P" takeoff "$J")"
says "a wall traced corner to corner along the bottom line is 20 ft" "walls 1            20.00 LF" "$TAKEOFF"
"$P" remove "$J" "$(made "$(grep -o 'wall-[0-9]*' <<<"$WALL" | head -1)")" >/dev/null

echo "== 1b. The same drawing in the binary form is the same sheet =="
python3 -I - "$D/plan-inches.dxf" "$T/plan-binary.dxf" <<'PY'
import sys, struct
t = open(sys.argv[1]).read().split("\n")
pairs = [(int(t[i].strip()), t[i + 1].rstrip("\r")) for i in range(0, len(t) - 1, 2) if t[i].strip() != ""]
out = bytearray(b"AutoCAD Binary DXF\r\n\x1a\0")
for c, v in pairs:
    out += struct.pack("<H", c)
    if 10 <= c <= 59 or 110 <= c <= 149 or 210 <= c <= 239: out += struct.pack("<d", float(v))
    elif 60 <= c <= 79 or 170 <= c <= 179 or 270 <= c <= 289: out += struct.pack("<h", int(v))
    elif 90 <= c <= 99: out += struct.pack("<i", int(v))
    else: out += v.strip().encode() + b"\0" if c != 1 else v.encode() + b"\0"
open(sys.argv[2], "wb").write(out)
PY
BIN="$("$P" drawing "$J" "$LEVEL" "Same, as binary" --file "$T/plan-binary.dxf" "${READ[@]}")"
says "binary: the same sheet" "at true size, nothing to calibrate: it covers 20' 0\" by 12' 0\"" "$BIN"
says "binary: the same lines" "layer WALLS: 4 lines" "$BIN"
"$P" remove "$J" "$(made "$BIN")" >/dev/null

echo "== 2. Its lines, by layer, and its words =="
for want in "layer WALLS: 4 lines" "layer FURNITURE: 4 lines, 1 circle" "layer NOTES: 1 line of words" 'words: "KITCHEN"' 'block "CHAIR" is drawn where it is set, 2 times' "holds 8 lines, 1 circle"; do says "counts" "$want" "$OUT"; done
says "the curve is said" "its 1 curve is drawn as 36 straight pieces of at most 10 degrees" "$OUT"

echo "== 3. A layer switched off is not among the lines the sheet snaps to =="
LAY="$("$P" layer "$J" "$SHEET" furniture off)"
echo "    | $LAY"
says "the lines now" "layer FURNITURE is off; 4 straight pieces to snap to now (44 before)" "$LAY"
says "layers lists it off" "off FURNITURE: 4 lines, 1 circle" "$("$P" layers "$J" "$SHEET")"
says "the drawings list says it" "layers off: FURNITURE" "$("$P" drawings "$J")"
is "the job holds which layers are off" "$(drawing_fact "$SHEET" 'd["layers_off"]')" "['FURNITURE']"
refused "the drawing has no layer" "$P" layer "$J" "$SHEET" "NOT A LAYER" off
"$P" layer "$J" "$SHEET" FURNITURE on >/dev/null

echo "== 4. With no unit said, no scale is guessed; one dimension gives it =="
cp "$D/plan-no-units.dxf" "$T/plan-no-units.dxf"
NU="$("$P" drawing "$J" "$LEVEL" "No unit" --file "$T/plan-no-units.dxf" "${READ[@]}")"
echo "$NU" | sed 's/^/    | /'
NSHEET="$(made "$NU")"
says "no scale" "no scale yet: the drawing does not say what it is drawn in" "$NU"
refused "scale" "$P" wall "$J" "$LEVEL" "$WT" --points "0,0 240,0" --height "9 ft" --on "$NSHEET"
CAL="$("$P" calibrate "$J" "$NSHEET" --from "0,0" --to "240,0" --distance "20 ft" "${READ[@]}")"
echo "    | $CAL"
SAME="$("$P" drawings "$J" | grep -A1 "No unit" | tail -1 | sed -E 's/.*: (a CAD drawing)/\1/; s/  \(.*//')"
SAME1="$("$P" drawings "$J" | grep -A1 "Kitchen plan" | tail -1 | sed -E 's/.*: (a CAD drawing)/\1/; s/  \(.*//')"
is "the same scale as step 1" "$SAME" "$SAME1"

echo "== 5. The walls finder runs on it, and says what it offers =="
FW="$("$P" find-walls "$J" "$SHEET")"
echo "$FW" | sed 's/^/    | /'
says "one line to a wall: told so, nothing made" "a house or room drawn with one line for each wall, from which a thickness cannot be read" "$FW"
says "its lines are read (the circle's 36 pieces are among them)" "sheet \"Kitchen plan\": 44 straight lines read" "$FW"
says "the circle is a tag, not a wall" "1 tag (a window's or a door's mark" "$FW"
says "nothing is made" "no wall found" "$FW"

echo "== 6. A closed format is refused by name, and nothing is brought in =="
BEFORE="$(cksum < "$J")"
printf 'AC1018\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0' > "$T/house.dwg"
cp "$T/house.dwg" "$T/house.bin"
printf 'something that is not even that' > "$T/other.dwg"
printf '\xff\xfe\xff\x0eS\0k\0e\0t\0c\0h\0U\0p\0 \0M\0o\0d\0e\0l\0' > "$T/model.skp"
printf '\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1B\0a\0s\0i\0c\0F\0i\0l\0e\0I\0n\0f\0o\0' > "$T/house.rvt"
refused "AutoCAD drawing (DWG, AutoCAD 2004). DWG is a closed format this program cannot read. Open it in the CAD program and save it as a DXF" "$P" drawing "$J" "$LEVEL" "A DWG" --file "$T/house.dwg" "${READ[@]}"
refused "DWG is a closed format" "$P" drawing "$J" "$LEVEL" "A DWG by another name" --file "$T/house.bin" "${READ[@]}"
refused "DWG is a closed format" "$P" drawing "$J" "$LEVEL" "Named DWG" --file "$T/other.dwg" "${READ[@]}"
refused "is a Revit model" "$P" drawing "$J" "$LEVEL" "Revit" --file "$T/house.rvt" "${READ[@]}"
refused "is a SketchUp model" "$P" drawing "$J" "$LEVEL" "SketchUp" --file "$T/model.skp" "${READ[@]}"
refused "AutoCAD drawing (DWG" "$P" page "$T/house.bin"
is "the job is as it was" "$(cksum < "$J")" "$BEFORE"

echo "== 7. The plan of a level goes out as a CAD drawing, in inches at true size =="
"$P" new "$T/out.json" "Out" >/dev/null
OL="$(made "$("$P" level "$T/out.json" "First Floor" --elevation "0 ft" --height "9 ft" "${K[@]}")")"
"$P" type "$T/out.json" wall "Wall 6" --thickness "6 in" "${K[@]}" >/dev/null
"$P" type "$T/out.json" door "Door 3" --width "3 ft" --height "7 ft" "${K[@]}" >/dev/null
"$P" wall "$T/out.json" "$OL" type-0003 --points "0,0 20,0 20,12 0,12" --closed --height "9 ft" "${K[@]}" >/dev/null
"$P" door "$T/out.json" wall-0005 type-0004 --offset "8 ft" --sill "0 ft" "${K[@]}" >/dev/null
"$P" room "$T/out.json" "$OL" Kitchen --at "10,6" "${K[@]}" >/dev/null
EX="$("$P" export dxf "$T/out.json" "$T/out.dxf" --level "First Floor")"
echo "$EX" | sed 's/^/    | /'
says "walls layer" "layer A-WALL: 10 lines drawn" "$EX"
says "door layer" "layer A-DOOR: 3 lines drawn" "$EX"
says "room layer" "layer A-AREA-IDEN: 1 room written" "$EX"
is "wall lines on A-WALL" "$(dxf_count "$T/out.dxf" A-WALL LINE)" "10"
is "door lines on A-DOOR" "$(dxf_count "$T/out.dxf" A-DOOR LINE)" "3"
is "text on A-AREA-IDEN" "$(dxf_count "$T/out.dxf" A-AREA-IDEN TEXT)" "2"
AREA="$("$P" takeoff "$T/out.json" | sed -nE 's/^ *rooms 1 +([0-9.]+) SF.*/\1/p')"
is "the room's area as takeoff gives it" "$AREA" "224.25"
python3 -I - "$T/out.dxf" "$AREA" <<'PY'
import sys
t = open(sys.argv[1]).read().split("\n")
words = [t[i + 1].strip() for i in range(0, len(t) - 1, 2) if t[i].strip() == "1" and t[i + 1].strip() not in ("AC1009",)]
assert "Kitchen" in words, words
assert sys.argv[2] + " SF" in words or sys.argv[2] + " SF VIF" in words, words
print("    the text is Kitchen and %s SF" % sys.argv[2])
PY
BACK="$("$P" page "$T/out.dxf")"
says "at true size: the outer lines span 246 by 150 inches (20 ft 6 in by 12 ft 6 in: the walls' lines are their centers, 6 in thick)" "246 by 150 drawing units" "$BACK"

echo "== 8. Written out and brought in again, it is the same plan, and the finder finds the same walls =="
cp "$T/out.dxf" "$T/back.dxf"
RB="$("$P" drawing "$J" "$LEVEL" "Back" --file "$T/back.dxf" "${READ[@]}")"
BSHEET="$(made "$RB")"
says "true size again" "it covers 20' 6\" by 12' 6\"" "$RB"
W2="$("$P" wall "$J" "$LEVEL" "$WT" --points "0,0 246,0" --height "9 ft" --on "$BSHEET")"
TK="$("$P" takeoff "$J")"
says "tracing the outer line gives 20 ft 6 in" "20.50 LF" "$TK"
FB="$("$P" find-walls "$J" "$BSHEET")"
echo "$FB" | sed 's/^/    | /'
says "four walls" "walls found: 4" "$FB"
says "20 ft by 12 ft on the walls' lines, 6 in thick" "20' 0\" long, 0' 6\" thick" "$FB"
says "the door, 3 ft wide" "3' 0\" wide" "$FB"

echo "== 9. What cannot be carried is counted, by kind =="
cp "$D/plan-with-extras.dxf" "$T/plan-with-extras.dxf"
XR="$("$P" drawing "$J" "$LEVEL" "With extras" --file "$T/plan-with-extras.dxf" "${READ[@]}")"
echo "$XR" | sed 's/^/    | /'
says "hatches" "2 hatches (filled or patterned areas) are in the drawing and not brought in" "$XR"
says "the picture" "1 picture placed in the drawing is in the drawing and not brought in" "$XR"
says "the 3D solid" "1 3D solid is in the drawing and not brought in" "$XR"
says "the dimension" "1 dimension is in the drawing: the figure of each is brought in as words" "$XR"
says "the sheet layout" "1 thing is drawn on a sheet layout (paper space) and not brought in" "$XR"
says "the dimension's figure is a word" '"240 in"' "$XR"
"$P" check "$J" | tail -1
echo "ALL STEPS PASSED"
