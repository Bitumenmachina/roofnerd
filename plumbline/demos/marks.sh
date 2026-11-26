#!/usr/bin/env bash
# The marks on a marked-up PDF read as quantities, and a TIFF taken in as a plan (DECISIONS.md call 227), at the command
# line. The files are made up and kept in demos/marks (demos/marks/make.py writes them, by hand, to the PDF standard's own
# notes: no file a marking program wrote was read). The plan is 30 ft by 20 ft at 1/4 in to the foot. Run from the
# repository root:  demos/marks.sh
# Every step fails the script if the program does not say what the step expects.
set -euo pipefail
cd "$(dirname "$0")/.."
# PLUMBLINE_BIN names a folder holding a program already built: it is used as it is and nothing is built.
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet -p plumbline; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"
D="$PWD/demos/marks"
PL="$PWD/demos/plans"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made up")
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3"; exit 1; }; printf '    %s\n' "$2"; }
saysnot() { ! grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: said: $2"; echo "$3"; exit 1; }; printf '    %-60s not said\n' "$2"; }
is() { [ "$2" = "$3" ] || { echo "FAIL: $1: got $2, wanted $3"; exit 1; }; printf '    %-60s %s\n' "$1" "$2"; }
refused() { local want="$1"; shift; local out
  if out="$("$@" 2>&1)"; then echo "FAIL: accepted: $*"; exit 1; fi
  echo "    refused: $(head -1 <<<"$out" | cut -c1-200)"
  grep -qF -- "$want" <<<"$out" || { echo "FAIL: the refusal does not say \"$want\""; echo "$out"; exit 1; }
}
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
line_of() { grep -F -- "$2" <<<"$1" | head -1; }
# A copy of a variant of the marked plan under the marked plan's own name: a mark is known by its name in the file and the file's name.
as_marked_plan() { mkdir -p "$T/copy"; cp "$D/$1" "$T/copy/marked-plan.pdf"; echo "$T/copy/marked-plan.pdf"; }

echo "== 1. The marks of the page are listed, in the page's order =="
LIST="$("$P" marks "$D/marked-plan.pdf")"
echo "$LIST" | sed 's/^/    | /'
says "seven entries" "7 marks (2 lengths, 1 run of lengths, 2 areas, 1 count, 1 note)" "$LIST"
says "South wall" "30 LF" "$(line_of "$LIST" "South wall  ")"
says "Outside walls" "100 LF" "$(line_of "$LIST" "Outside walls")"
says "Whole floor" "600 SF" "$(line_of "$LIST" "Whole floor")"
says "Bedroom" "120 SF" "$(line_of "$LIST" "Bedroom  ")"
says "Receptacle" "4 EA" "$(line_of "$LIST" "Receptacle")"
says "the note measures nothing" "a note: it measures nothing" "$(line_of "$LIST" "verify header")"
python3 -I - "$LIST" <<'PY'
import sys
t = sys.argv[1]
at = [t.index(w) for w in ["South wall  ", "Outside walls", "Whole floor", "Bedroom  ", "Receptacle", "verify header", "North wall  "]]
assert at == sorted(at), at
print("    in the page's own order: South wall, Outside walls, Whole floor, Bedroom, Receptacle, verify header, North wall")
PY

echo "== 2. North wall: both figures, flagged =="
says "flagged" "! length         North wall" "$LIST"
says "both figures" "it says 31 LF; by its ends at 1/4\" = 1'-0\" it works out to 30 LF: 1 LF apart. What the mark says is what comes in" "$LIST"
is "one mark flagged, and no other" "$(grep -c '^!' <<<"$LIST")" "1"
for bad in annotation Annot dictionary Vertices Subtype Contents viewport Stamp; do saysnot "the standard's own words: $bad" "$bad" "$LIST"; done

echo "== 3. Brought into a new job (without --counts: five lines; the count is left and said) =="
"$P" new "$J" "A made-up job" >/dev/null
says "a new job has nothing priced" "Nothing in this job has a cost line" "$("$P" estimate "$J")"
cp "$D/marked-plan.pdf" "$T/marked-plan.pdf"
OUT="$("$P" import marks "$J" "$T/marked-plan.pdf")"
echo "$OUT" | sed 's/^/    | /'
says "five marks" "5 marks measured" "$OUT"
says "five added" "5 added:" "$OUT"
for l in "South wall: 30 LF" "Outside walls: 100 LF" "Whole floor: 600 SF" "Bedroom: 120 SF" "North wall: 31 LF"; do says "line" "$l" "$OUT"; done
saysnot "the count is not a line" "Receptacle: 4 EA" "$OUT"
says "the count is left and said" "counts left: 1 label, 4 symbols (Receptacle): the file states no count" "$OUT"
says "not priced and no division" "5 lines are not priced" "$OUT"
says "the flag repeated" "! North wall: it says 31 LF; by its ends" "$OUT"
SHEET="$("$P" estimate "$J")"
says "under the file's name" "from marked-plan.pdf, page 1: 5 lines of its marks, each Assumed, not priced, no division set" "$SHEET"
says "assumed on the sheet" "~31.00 LF" "$(line_of "$SHEET" "North wall")"
BYITEM="$("$P" estimate "$J" by-item)"
says "its division is not set" "No division" "$BYITEM"
says "five items" "5 items from 5 lines" "$BYITEM"
says "the job's price is what it was: no figure, nothing priced" "PRICE                    no figure   ? there is no job cost" "$SHEET"
python3 -I - "$J" <<'PY'
import sys, json
j = json.load(open(sys.argv[1]))
lines = [l for e in j["data"] if e["attributes"].get("pl::class") == "Job" for l in e["attributes"]["pl::cost"]["lines"]]
assert len(lines) == 5, len(lines)
for l in lines:
    m = l["inputs"]["measured"]
    assert m["status"] == "Assumed", l
    assert m["source"].startswith('mark "') and "page 1 of marked-plan.pdf" in m["source"], m["source"]
    assert "unit_cost" not in l or l["unit_cost"].get("value") is None, l
    assert "division" not in l and "class" not in l, l
print("    five lines: each Assumed, each source names its mark, page 1 and the file; none priced; none has a division")
PY

echo "== 3b. With --counts the count of the symbols is a sixth line, saying what it counted =="
OUT="$("$P" import marks "$J" "$T/marked-plan.pdf" --counts)"
says "one added" "1 added:" "$OUT"
says "the count" "Receptacle: 4 EA" "$OUT"
says "the others are the same" "5 are the same as the job already has" "$OUT"
saysnot "no count left" "counts left" "$OUT"
python3 -I - "$J" <<'PY'
import sys, json
j = json.load(open(sys.argv[1]))
lines = [l for e in j["data"] if e["attributes"].get("pl::class") == "Job" for l in e["attributes"]["pl::cost"]["lines"]]
assert len(lines) == 6, len(lines)
c = next(l for l in lines if l["description"] == "Receptacle")["inputs"]["measured"]
assert c["status"] == "Assumed" and c["source"] == "4 symbols labelled Receptacle counted on page 1 of marked-plan.pdf", c
print("    six lines; the count is Assumed and says: 4 symbols labelled Receptacle counted on page 1 of marked-plan.pdf")
PY
says "on the sheet" "~4" "$(line_of "$("$P" estimate "$J")" "Receptacle")"
echo "    (steps 4 to 6 run with --counts, so the job has the six lines)"

echo "== 4. Brought in again: nothing is added =="
SUM1="$(sha256sum "$J" | cut -c1-16)"
OUT="$("$P" import marks "$J" "$T/marked-plan.pdf" --counts)"
says "six are the same" "6 are the same as the job already has" "$OUT"
saysnot "nothing added" "added:" "$OUT"
is "the job is as it was" "$(sha256sum "$J" | cut -c1-16)" "$SUM1"

echo "== 5. Whole floor says 650 SF in a copy: that line moves and says what it was =="
OUT="$("$P" import marks "$J" "$(as_marked_plan floor-650.pdf)" --counts)"
says "moved" "Whole floor: was 600 SF, now 650 SF; its line was moved" "$OUT"
says "five are the same" "5 are the same as the job already has" "$OUT"
says "the sheet" "~650.00 SF" "$(line_of "$("$P" estimate "$J")" "Whole floor")"

echo "== 6. Bedroom taken out of a copy: named, and its line stays =="
OUT="$("$P" import marks "$J" "$(as_marked_plan no-bedroom.pdf)" --counts)"
says "gone" "Bedroom: gone from the file; its line is still in the job" "$OUT"
says "still on the sheet" "~120.00 SF" "$(line_of "$("$P" estimate "$J")" "Bedroom")"

echo "== 7. A curved edge, no scale and a figure in metres: each named and left out; the others come in =="
"$P" new "$T/job7.json" "A second made-up job" >/dev/null
OUT="$("$P" import marks "$T/job7.json" "$D/unreadable.pdf")"
echo "$OUT" | sed 's/^/    | /'
says "the one that reads comes in" "South wall: 30 LF" "$OUT"
says "curved edge" "not read, left out of the estimate: Round bed: its edge is a curve" "$OUT"
says "no scale" "not read, left out of the estimate: Loose area: it has no scale" "$OUT"
says "metres" "not read, left out of the estimate: Metric run: it says \"9 m\", a figure in m, a unit this program does not hold" "$OUT"
S7="$("$P" estimate "$T/job7.json")"
for gone in "Round bed" "Loose area" "Metric run"; do saysnot "$gone is no line" "$gone" "$S7"; done
says "a scale for part of the sheet" "not read: its scale is set for a part of the sheet, which is not read" "$("$P" marks "$D/viewport.pdf")"

echo "== 8. A page with no marks: said, and the job is byte for byte what it was =="
SUM8="$(sha256sum "$J" | cut -c1-16)"
OUT="$("$P" import marks "$J" "$PL/made-up-plan.pdf")"
says "no marks" "no marks on page 1 of made-up-plan.pdf" "$OUT"
is "the job is as it was" "$(sha256sum "$J" | cut -c1-16)" "$SUM8"
says "marks says so too" "no marks on page 1 of made-up-plan.pdf" "$("$P" marks "$PL/made-up-plan.pdf")"
refused "only a PDF has marks" "$P" marks "$PL/made-up-plan.png"
refused "page 2 was asked for" "$P" marks "$PL/made-up-plan.pdf" --page 2

echo "== 8b. A marked PDF added as a drawing keeps its marks with the sheet =="
"$P" new "$T/job8.json" "A third made-up job" >/dev/null
LEVEL="$(made "$("$P" level "$T/job8.json" "First Floor" --elevation "0 ft" --height "9 ft" "${K[@]}")")"
cp "$D/marked-plan.pdf" "$T/job8-marked.pdf"
OUT="$("$P" drawing "$T/job8.json" "$LEVEL" "Marked plan" --file "$T/job8-marked.pdf" --status Assumed --source "the drawing")"
says "the marks are kept and said" "marks: 7 marks on the page kept with the drawing (6 measured)" "$OUT"
python3 -I - "$T/job8.json" <<'PY'
import sys, json
j = json.load(open(sys.argv[1]))
d = next(e["attributes"]["pl::drawing"] for e in j["data"] if "pl::drawing" in e["attributes"])
ms = d["marks"]
assert [m["label"] for m in ms] == ["South wall", "Outside walls", "Whole floor", "Bedroom", "Receptacle", "verify header", "North wall"], [m["label"] for m in ms]
assert all(m["says"]["status"] == "Assumed" for m in ms if "says" in m)
south = ms[0]
assert south["points"] == [[126.0, 126.0], [666.0, 126.0]], south["points"]
print("    seven marks kept with the drawing, in the page's order; each figure Assumed; the south wall's two ends at (126, 126) and (666, 126)")
PY
cp "$PL/made-up-plan.pdf" "$T/job8-plain.pdf"
OUT="$("$P" drawing "$T/job8.json" "$LEVEL" "Plain plan" --file "$T/job8-plain.pdf" --status Assumed --source "the drawing")"
saysnot "a plain page says nothing of marks" "marks" "$OUT"

echo "== 9. TIFF: a picture of a plan as a PNG is =="
"$P" new "$T/job9.json" "A fourth made-up job" >/dev/null
L9="$(made "$("$P" level "$T/job9.json" "First Floor" --elevation "0 ft" --height "9 ft" "${K[@]}")")"
"$P" type "$T/job9.json" wall "Wall 6" --thickness "6 in" "${K[@]}" >/dev/null
cp "$D/made-up-plan.tif" "$T/plan.tif"; cp "$PL/made-up-plan.png" "$T/plan.png"
TOUT="$("$P" drawing "$T/job9.json" "$L9" "As a TIFF" --file "$T/plan.tif" --status Assumed --source "the scan")"
POUT="$("$P" drawing "$T/job9.json" "$L9" "As a PNG" --file "$T/plan.png" --status Assumed --source "the scan")"
says "the TIFF's size in dots" "(TIFF picture), 1584 by 1224 dots" "$TOUT"
says "the PNG's size in dots" "(PNG picture), 1584 by 1224 dots" "$POUT"
TD="$(made "$TOUT")"; PD="$(made "$POUT")"
for d in "$TD" "$PD"; do "$P" calibrate "$T/job9.json" "$d" --from "252,252" --to "1332,252" --distance "30 ft" "${K[@]}" >/dev/null; done
"$P" wall "$T/job9.json" "$L9" type-0003 --points "252,252 1332,252" --height "9 ft" --on "$TD" >/dev/null
W1="$(line_of "$("$P" takeoff "$T/job9.json")" "walls 1")"
"$P" remove "$T/job9.json" wall-0006 >/dev/null
"$P" wall "$T/job9.json" "$L9" type-0003 --points "252,252 1332,252" --height "9 ft" --on "$PD" >/dev/null
W2="$(line_of "$("$P" takeoff "$T/job9.json")" "walls 1")"
says "a wall traced over the TIFF is 30 ft" "walls 1            30.00 LF" "$W1"
is "the same wall traced over the PNG, at the same scale" "$W2" "$W1"
cp "$D/two-pages.tif" "$T/two.tif"; cp "$D/old-jpeg.tif" "$T/old.tif"
OUT="$("$P" drawing "$T/job9.json" "$L9" "Sheet 2" --file "$T/two.tif" --page 2 --status Assumed --source "the scan")"
says "page 2 of a two-page TIFF" "page 2 of 2 of two.tif (TIFF picture), 800 by 600 dots" "$OUT"
refused "the file has 2 pages; page 3 was asked for" "$P" drawing "$T/job9.json" "$L9" "Sheet 3" --file "$T/two.tif" --page 3 --status Assumed --source "the scan"
refused "packed as old-style JPEG, which is not read" "$P" drawing "$T/job9.json" "$L9" "Old" --file "$T/old.tif" --status Assumed --source "the scan"

echo "== 10. The sample whole house is written as it was =="
"$P" sixteen "$T/h.json" >/dev/null
is "plumbline sixteen, sha256" "$(sha256sum "$T/h.json" | cut -c1-16)" "880d9fd3c364b5f7"

echo "all ten steps passed"
