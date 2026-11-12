#!/usr/bin/env bash
# Items from a manufacturer's sheet, and furnishings. A cut sheet (a PDF with words on
# it) is read into an item: its name, maker, model, width, depth, height and weight,
# each Assumed and sourced to the line it was read from; what the sheet does not say is
# Unknown and asked for. Items are placed, one is moved along a wall until it clears a
# water closet, and the lot goes out as an IFC4 model.
# Run from the repository root:  demos/spec.sh
# Every sheet, maker, model number and size is made up in this script (no real product
# data) and lives in a temporary folder. Every step fails the script if the program
# does not say what the step expects. The figures are worked by hand in the comments
# and in STATUS.md ("The rebuild after call 62", part A).
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --quiet
P="$PWD/target/debug/plumbline"
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
A=(--status Assumed --source "set by eye")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$OUT" || { echo "FAIL: $1: not said: $2"; echo "$OUT"; exit 1; }; printf '    %s\n' "$2"; }
refused() { local want="$1"; shift; if OUT="$("$@" 2>&1)"; then echo "FAIL: not refused: $*"; echo "$OUT"; exit 1; fi; grep -qF -- "$want" <<<"$OUT" || { echo "FAIL: refused, but not for: $want"; echo "$OUT"; exit 1; }; printf '    refused: %s\n' "$(head -1 <<<"$OUT" | cut -c1-200)"; }

# A one-page sheet of words, written as a PDF by hand: each line as "size|text".
sheet() { # $1 out.pdf, then lines
  local out="$1"; shift
  python3 - "$out" "$@" <<'PY'
import sys
out, lines = sys.argv[1], [l.split("|", 1) for l in sys.argv[2:]]
content, y = "BT\n", 740
for size, text in lines:
    esc = text.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)")
    content += f"/F1 {size} Tf\n1 0 0 1 72 {y} Tm\n({esc}) Tj\n"
    y -= int(size) + 8
content += "ET"
objs = ["<< /Type /Catalog /Pages 2 0 R >>", "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>",
        f"<< /Length {len(content)} >>\nstream\n{content}\nendstream",
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"]
pdf, at = b"%PDF-1.4\n", []
for i, body in enumerate(objs):
    at.append(len(pdf)); pdf += f"{i+1} 0 obj\n{body}\nendobj\n".encode("latin-1")
xref = len(pdf)
pdf += f"xref\n0 {len(objs)+1}\n0000000000 65535 f \n".encode()
for o in at: pdf += f"{o:010} 00000 n \n".encode()
pdf += f"trailer\n<< /Size {len(objs)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
open(out, "wb").write(pdf)
PY
}

"$P" new "$J" "Items from sheets (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"

echo "== 1. A vanity: its sizes on one line of its sheet =="
# 36 in wide, 21 in deep, 34 1/2 in high: 3 ft by 1 ft 9 in by 2 ft 10 1/2 in.
sheet "$T/vanity.pdf" "24|Harbor Vanity 36" "12|Manufacturer: Northfield Millwork" "11|Model: NV-3621" '11|Overall dimensions: 36" W x 21" D x 34-1/2" H' "11|Shipping weight: 96 lb"
OUT="$("$P" import spec "$J" "$T/vanity.pdf")"; sed 's/^/  /' <<<"$OUT"
VANITY="$(made "$OUT")"
says "the vanity" "Harbor Vanity 36, a furnishing (its name says “vanity”; --as says otherwise), from vanity.pdf, page 1"
says "its maker" "~ maker   Northfield Millwork"
says "its model" "~ model   NV-3621"
says "its width" '~ width   36 in'
says "its depth" '~ depth   21 in'
says "its height" '~ height  34 1/2 in'
says "its weight" "~ weight  96 lb"
says "the line" '“Overall dimensions: 36" W x 21" D x 34-1/2" H”'
OUT="$("$P" show "$J")"
says "a source" 'vanity.pdf, page 1: “Overall dimensions: 36" W x 21" D x 34-1/2" H”'

echo
echo "== 2. An air handler: its sizes on three lines, the word first =="
# 21 in, 24 in and 49 in: 1 ft 9 in by 2 ft by 4 ft 1 in.
sheet "$T/ah.pdf" "20|AH-24 Air Handler" "10|Model No. AH-24-B" "10|Width 21 in" "10|Depth 24 in" "10|Height 49 in" "10|Weight 135 lbs" "10|Electrical: 120 V, 15 A"
OUT="$("$P" import spec "$J" "$T/ah.pdf")"; sed 's/^/  /' <<<"$OUT"
AH="$(made "$OUT")"
says "the air handler" "AH-24 Air Handler, a piece of equipment (its name says “air handler”"
says "its model" "~ model   AH-24-B"
says "its width" "~ width   21 in"
says "its height" "~ height  49 in"
says "its weight" "~ weight  135 lb"
# No maker is written on it: Unknown, and asked for, not guessed.
says "no maker" "? maker"

echo
echo "== 3. A sheet with no height: the item is made, and what is missing is said =="
sheet "$T/bookcase.pdf" "18|Bookcase BK-30" "10|30 in wide, 12 in deep"
OUT="$("$P" import spec "$J" "$T/bookcase.pdf")"; sed 's/^/  /' <<<"$OUT"
BK="$(made "$OUT")"
says "no height" "? height"
says "what that means" "Without all three sizes it has no shape"
OUT="$("$P" place "$J" "$S" "$BK" --at "20,2" "${A[@]}")"; sed 's/^/  /' <<<"$OUT"
BK1="$(made "$OUT")"
says "placed with no shape" "no size yet, so it has no shape and is not tested for clashes"
says "why" "its height is not known"
# A tape on the real thing: 72 in. The item's height is Known, and the one placed follows.
OUT="$("$P" measure "$J" "$BK" height "72 in" --source "tape, on the unit as delivered")"; sed 's/^/  /' <<<"$OUT"
says "what followed" "$BK1 is one of it and is now that size"
OUT="$("$P" show "$J")"
[ "$("$P" clashes "$J" | grep -c "$BK1.*no shape" || true)" = 0 ] || { echo "FAIL: the bookcase still has no shape"; exit 1; }
echo "    the bookcase placed now has a shape: 2 ft 6 in by 1 ft by 6 ft"

echo
echo "== 4. Three sizes with no letters: read in the order written, and it says so =="
sheet "$T/cube.pdf" "16|Storage Cabinet C-24" "10|24 x 18 x 30 in"
OUT="$("$P" import spec "$J" "$T/cube.pdf")"; sed 's/^/  /' <<<"$OUT" | cut -c1-230
says "the order" "read as width by depth by height, in the order written"
says "its width" "~ width   24 in"
says "its depth" "~ depth   18 in"
says "its height" "~ height  30 in"

echo
echo "== 5. Metric only: turned to inches, and the millimetres kept =="
# 600 / 25.4 = 23.622 in; 450 / 25.4 = 17.7165 in; 850 / 25.4 = 33.4646 in.
sheet "$T/base.pdf" "16|Base Cabinet B-600" "10|600 x 450 x 850 mm"
OUT="$("$P" import spec "$J" "$T/base.pdf")"; sed 's/^/  /' <<<"$OUT" | cut -c1-230
BASE="$(made "$OUT")"
says "the millimetres" "600 mm on the sheet"
python3 - "$J" "$BASE" <<'PY'
import json, sys
item = next(e for e in json.load(open(sys.argv[1]))["data"] if e["path"] == sys.argv[2])["attributes"]["pl::item"]
got = [round(item[k]["value"], 4) for k in ("width", "depth", "height")]
assert got == [23.622, 17.7165, 33.4646], got
assert all(item[k]["status"] == "Assumed" and item[k]["unit"] == "in" for k in ("width", "depth", "height"))
print("    in the job: %s in, each Assumed" % " by ".join(str(g) for g in got))
PY

echo
echo "== 6. What is not a sheet, or does not say what it is, is refused and nothing is changed =="
BEFORE="$(sha256sum "$J")"
sheet "$T/mystery.pdf" "16|Model 4400" "10|10 x 10 x 10 in"
refused "does not say what kind of thing it is" "$P" import spec "$J" "$T/mystery.pdf"
printf 'not a pdf' > "$T/words.txt"
refused "only a PDF has words" "$P" import spec "$J" "$T/words.txt" --as fixture
refused "the file has 1 page; page 3 was asked for" "$P" import spec "$J" "$T/vanity.pdf" --page 3
[ "$(sha256sum "$J")" = "$BEFORE" ] || { echo "FAIL: a refusal changed the job"; exit 1; }
echo "    the job is unchanged"

echo
echo "== 7. The vanity moved along a wall until it clears the water closet =="
# A water closet 1.5 ft wide, 2.25 ft deep, 2.5 ft high, its middle at 5 ft east: it
# reaches from 4.25 to 5.75. The vanity (3 ft wide), its middle at 7 ft, reaches from
# 5.5 to 8.5: in each other by 5.75 - 5.5 = 0.25 ft = 3.00 in.
WC="$(made "$("$P" place "$J" "$S" fixture "Water closet" --at "5,1.125" --offset "0 ft" --size "1.5,2.25,2.5" "${K[@]}")")"
OUT="$("$P" place "$J" "$S" "$VANITY" --at "7,0.875" "${A[@]}")"; sed 's/^/  /' <<<"$OUT"
V1="$(made "$OUT")"
says "its size" "3' 0\" wide, 1' 9\" deep, 2' 10 1/2\" high (as its sheet says; not yet checked)"
OUT="$("$P" clashes "$J")"; grep -F "$V1" <<<"$OUT" | sed 's/^/  /' | cut -c1-220
says "the clash" "in each other by 3.00 in"
# Moved 6 in east: it reaches from 6 to 9, clear of the water closet by 3 in.
"$P" move "$J" "$V1" --by "0.5,0" "${A[@]}" | sed 's/^/  /' | head -2
OUT="$("$P" clashes "$J")"
if grep -F "$V1" <<<"$OUT" | grep -q "in each other"; then echo "FAIL: still in the water closet after the move"; echo "$OUT"; exit 1; fi
echo "    no clash after the move"
OUT="$("$P" clashes "$J" --clearance "4 in")"; grep -F "$V1" <<<"$OUT" | sed 's/^/  /' | cut -c1-220
says "the clearance" "3.00 in"

echo
echo "== 8. Out to BIM: an IFC4 model with the furniture in it =="
"$P" place "$J" "$S" "$AH" --at "14,2" "${A[@]}" >/dev/null
OUT="$("$P" export ifc "$J" "$T/out.ifc")"; sed "s|$T/||g; s/^/  /" <<<"$OUT" | cut -c1-200 | head -5
says "the furnishings" "2 furnishings"
[ "$(grep -c '^#[0-9]*=IFCFURNITURE(' "$T/out.ifc")" = 2 ] || { echo "FAIL: the model does not hold two IfcFurniture"; exit 1; }
grep -q "IFCFURNITURE(.*'Harbor Vanity 36'" "$T/out.ifc" || { echo "FAIL: the vanity is not in the model by name"; exit 1; }
echo "    2 IfcFurniture in the model, the vanity by its name"
if python3 -c "import ifcopenshell" 2>/dev/null; then
  OUT="$(python3 tools/ifc-check.py "$T/out.ifc")" || { echo "FAIL: IfcOpenShell finds a fault in the model"; echo "$OUT"; exit 1; }
  says "validated" "IFC4: no fault"
else
  echo "demos/spec.sh needs IfcOpenShell to validate the model (python3 -c 'import ifcopenshell' fails; set PYTHONPATH to where it is)"; exit 2
fi
"$P" check "$J" >/dev/null || { echo "FAIL: the job does not check"; exit 1; }
echo
echo "PASS: items are made from sheets with every value sourced to its line, what a sheet does not say is asked for, a vanity is moved clear, and the model validates"
