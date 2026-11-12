#!/usr/bin/env bash
# Plans under the tracing, at the command line: a made-up vector plan is added to an
# empty job as a drawing of its level, given its scale from one dimension, and traced
# over; the same plan as a PNG and as a JPEG; what is refused; a plan file that changes
# under its drawing; and, when it has been fetched, a real scanned sheet.
# Run from the repository root:  demos/plan.sh
# The made-up plan is demos/plans/made-up-plan.pdf (tools/made-up-plan.py writes it): a
# house 30 ft by 20 ft at 1/4 in to the foot, 18 points to the foot, its lower-left corner
# at (126, 126) on the page, with a 30 ft dimension line from (126, 90) to (666, 90).
# Every step fails the script if the program does not do what the step says.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and
# nothing is built (so that a long run is not changed under it by work in the tree).
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
READ=(--status Assumed --source "read off the drawing")

made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
figure() { # $1 path, $2 measure
  python3 - "$J" "$1" "$2" <<'PY'
import sys, json
e = next(x for x in json.load(open(sys.argv[1]))["data"] if x["path"] == sys.argv[2])
d = e["attributes"].get("pl::measure", {}).get(sys.argv[3])
print("none" if d is None or d["value"] is None else ("%s %s" % (d["value"], d["status"])))
PY
}
is() { [ "$2" = "$3" ] || { echo "FAIL: $1: got $2, wanted $3"; exit 1; }; printf '    %-52s %s\n' "$1" "$2"; }
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3"; exit 1; }; printf '    %s\n' "$2"; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $out" | head -1 | cut -c1-200
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; exit 1; }
}

echo "== An empty job, a level, a wall type; and the plan laid under the level =="
cp demos/plans/made-up-plan.pdf demos/plans/made-up-plan.png demos/plans/made-up-plan.jpg "$T/"
"$P" new "$J" "Traced from a plan (not a real job)"
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "8 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
OUT="$("$P" drawing "$J" "$S" "A-1 floor plan" --file "$T/made-up-plan.pdf" "${K[@]}")"
D="$(made "$OUT")"
says "the page" "page 1 of 1 of made-up-plan.pdf (PDF), 792 by 612 points" "$OUT"
says "its lines" "8 straight lines to snap to" "$OUT"
says "no scale" "no scale yet" "$OUT"
says "the door's swing" "not read: 1 curved piece of line is known by its ends only" "$OUT"
says "its words" "not read: the words and figures on the page are drawn and not read" "$OUT"

echo
echo "== Nothing is traced over a drawing with no scale =="
cp "$J" "$T/before.json"
refused "has no scale: calibrate it from one dimension" "$P" wall "$J" "$S" "$WT" --points "126,126 666,126" --height "8 ft" --on "$D"
cmp -s "$J" "$T/before.json" || { echo "FAIL: a refused tracing changed the file"; exit 1; }

echo
echo "== Its scale, from one dimension: 30 ft between two places 540 points apart =="
OUT="$("$P" calibrate "$J" "$D" --from "126,90" --to "666,90" --distance "30 ft" "${READ[@]}")"
says "the scale" "30 ft is 540 on its page: 18 to the foot" "$OUT"
"$P" lineup "$J" "$D" --page-point "126,126" --plan-point "0,0" --status Assumed --source "the house's southwest corner"

echo
echo "== Traced: the outline, by four places on the page =="
OUT="$("$P" wall "$J" "$S" "$WT" --points "126,126 666,126 666,486 126,486" --closed --height "8 ft" --on "$D")"; echo "    $OUT"
read -r SOUTH EAST NORTH WEST <<<"$(sed -E 's/^[^:]*: //; s/,//g' <<<"$OUT")"
# 540 points at 18 to the foot is 30 ft; 360 points is 20 ft. Traced, so Assumed.
is "south wall, 540 points" "$(figure "$SOUTH" plan_length)" "30.0 Assumed"
is "east wall, 360 points" "$(figure "$EAST" plan_length)" "20.0 Assumed"
is "south wall's face, 30 ft by 8 ft" "$(figure "$SOUTH" face_area)" "240.0 Assumed"
python3 - "$J" "$SOUTH" <<'PY'
import sys, json
e = next(x for x in json.load(open(sys.argv[1]))["data"] if x["path"] == sys.argv[2])
line = e["attributes"]["pl::wall"]["line"]
want = "traced from A-1 floor plan (made-up-plan.pdf, page 1)"
if line["value"] != [[0.0, 0.0], [30.0, 0.0]] or line["status"] != "Assumed" or line["source"] != want:
    print("FAIL: the traced wall is not on the plan where the drawing puts it, flagged and sourced:", line); sys.exit(1)
print("    its line is (0, 0) to (30, 0) ft, Assumed, \"%s\"" % want)
PY
"$P" check "$J"
OUT="$("$P" drawings "$J")"
says "the drawing, listed" "scale: 30 ft between (126, 90) and (666, 90) on its page: 18 to the foot" "$OUT"
says "its file" "file: as it was when the drawing was made" "$OUT"

echo
echo "== The same plan as a picture: nothing on it to snap to =="
for kind in png jpg; do
  OUT="$("$P" drawing "$J" "$S" "the plan as a $kind" --file "$T/made-up-plan.$kind" "${K[@]}")"
  says "a $kind" "1584 by 1224 dots, no lines to snap to" "$OUT"
  says "a $kind" "not read: it is a picture: it has no lines of its own to snap to" "$OUT"
done
# The picture is two dots to the point: the dimension line is 1,080 dots for 30 ft.
PIC="$(made "$OUT")"
OUT="$("$P" calibrate "$J" "$PIC" --from "252,180" --to "1332,180" --distance "30 ft" "${READ[@]}")"
says "the picture's scale" "30 ft is 1080 on its page: 36 to the foot" "$OUT"

echo
echo "== What is refused, and leaves the file as it was =="
cp "$J" "$T/before.json"
refused "cannot read" "$P" drawing "$J" "$S" "missing" --file "$T/not-there.pdf" "${K[@]}"
refused "the file has 1 page; page 2 was asked for" "$P" drawing "$J" "$S" "page two" --file "$T/made-up-plan.pdf" --page 2 "${K[@]}"
echo "not a plan" > "$T/notes.txt"
refused "it is not a PDF, a PNG or a JPEG" "$P" drawing "$J" "$S" "notes" --file "$T/notes.txt" "${K[@]}"
refused "its two points are one point" "$P" calibrate "$J" "$D" --from "126,90" --to "126,90" --distance "30 ft" "${READ[@]}"
refused "has no unit" "$P" calibrate "$J" "$D" --from "126,90" --to "666,90" --distance "30" "${READ[@]}"
refused "--status and --source are not given" "$P" wall "$J" "$S" "$WT" --points "126,200 300,200" --height "8 ft" --on "$D" "${K[@]}"
refused "is not a drawing" "$P" wall "$J" "$S" "$WT" --points "126,200 300,200" --height "8 ft" --on "$SOUTH"
cmp -s "$J" "$T/before.json" || { echo "FAIL: a refused verb changed the file"; exit 1; }
echo "    the file is byte for byte what it was before the refusals"

echo
echo "== A plan file that changes under its drawing is said to have changed =="
printf '\n%% a note added to the file after the drawing was made from it\n' >> "$T/made-up-plan.pdf"
OUT="$("$P" drawings "$J")"
says "the changed file" "file: HAS CHANGED since this drawing was made from it" "$OUT"

echo
echo "== A real sheet: USDA Plan No. 7213, Two-Bedroom Farmhouse (public domain; a scan) =="
REAL="$PWD/plans/usda-7213-two-bedroom-farmhouse.pdf"
if [ -f "$REAL" ]; then
  OUT="$("$P" drawing "$J" "$S" "USDA 7213, sheet 1" --file "$REAL" --page 4 --status Known --source "USDA plan 7213, sheet 1 of 2")"
  says "the sheet" "page 4 of 4" "$OUT"
  says "its size" "789 by 613 points, no lines to snap to" "$OUT"
  says "a scan" "not read: it is a scan (1 picture and no drawn lines)" "$OUT"
else
  echo "    not here: tools/fetch-plans.sh fetches it (23 MB) from the National Agricultural Library. Skipped."
fi

echo
echo "PASS: a plan was laid under a level, given its scale from one dimension and traced over, and what it could not give was said"
