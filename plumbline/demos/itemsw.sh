#!/usr/bin/env bash
# In the window: a maker's sheet brought in and made into an item; one of it set down
# on a slab with a click; and everything that goes out of a job written out with a
# click each: the BIM model (IFC4), the estimate as a workbook and as CSV, and the
# three lenses. Then the job and the six files are read back.
# Run from the repository root:  demos/itemsw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display
# (Xvfb, xdotool and a Vulkan driver; Mesa's software one will do). The sheet, the
# maker and the job are made up. The figures are in STATUS.md ("A maker's sheet, an
# item set down, and everything written out").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/itemsw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/itemsw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/items}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }

echo "== The job: a level and a slab 12 ft by 8 ft; and beside it, a maker's sheet for a vanity =="
python3 - "$T/vanity.pdf" <<'PY'
import sys
lines = [("24", "Harbor Vanity 36"), ("12", "Manufacturer: Northfield Millwork"), ("11", "Model: NV-3621"), ("11", 'Overall dimensions: 36" W x 21" D x 34-1/2" H'), ("11", "Shipping weight: 96 lb")]
content, y = "BT\n", 740
for size, text in lines:
    esc = text.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)")
    content += f"/F1 {size} Tf\n1 0 0 1 72 {y} Tm\n({esc}) Tj\n"; y -= int(size) + 8
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
open(sys.argv[1], "wb").write(pdf)
PY
"$P" new "$J" "From a sheet, in the window (not a real job)" >/dev/null
L="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
FT="$(made "$("$P" type "$J" floor "slab 4 in" --thickness "4 in" "${K[@]}")")"
"$P" floor "$J" "$L" "$FT" --points "0,0 12,0 12,8 0,8" "${K[@]}" >/dev/null
"$P" check "$J" >/dev/null

echo
echo "== The session: Maker's sheet; the item, then a click on the slab; each thing written out; save =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
on "Maker's sheet"; typed "vanity.pdf"; park
shot                                    # 01 the item made from the sheet, and listed under Items
on "Item Harbor Vanity 36"; sleep 2
click 800 500; park
shot                                    # 02 one set down on the slab, and selected
on "BIM model (IFC)"; park
shot                                    # 03 the BIM model written, and what is in it said
on "Estimate workbook"; on "Estimate (CSV)"; on "For suppliers"; on "For subcontractors"; on "For the PM"; park
shot                                    # 04 the last of the six written
xdotool key F2; sleep 5
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
[ ! -e "$T/missing.txt" ] || { echo "FAIL: a control was not on screen"; cat "$T/missing.txt"; exit 1; }
for n in 01 02 03 04; do [ -s "$OUT/$n-model.png" ] || { echo "FAIL: no picture $n"; exit 1; }; done

echo
echo "== The job, read back =="
python3 - "$J" <<'PY'
import json, sys
data = json.load(open(sys.argv[1]))["data"]
items = [e for e in data if "pl::item" in e.get("attributes", {})]
assert len(items) == 1, f"{len(items)} items"
item = items[0]["attributes"]["pl::item"]
sizes = [item[k]["value"] for k in ("width", "depth", "height")]
assert sizes == [36.0, 21.0, 34.5] and all(item[k]["status"] == "Assumed" and item[k]["unit"] == "in" for k in ("width", "depth", "height")), item
assert "vanity.pdf, page 1" in item["width"]["source"] and 'Overall dimensions: 36" W x 21" D x 34-1/2" H' in item["width"]["source"], item["width"]["source"]
assert item["category"] == "Furnishing" and item["maker"]["value"] == "Northfield Millwork" and item["model"]["value"] == "NV-3621", item
print("    the item: a furnishing, 36 by 21 by 34.5 in, each Assumed and sourced to its line of vanity.pdf; maker and model read")
placed = [e for e in data if e.get("attributes", {}).get("pl::placed", {}).get("item") == items[0]["path"]]
assert len(placed) == 1, f"{len(placed)} set down"
p = placed[0]["attributes"]["pl::placed"]
(x, y) = p["at"]["value"]
assert p["size"]["value"] == [3.0, 1.75, 2.875], p["size"]
assert 0 <= x <= 12 and 0 <= y <= 8, (x, y)
assert p["at"]["status"] == "Assumed" and p["at"]["source"] == "set down by hand in the model", p["at"]
print(f"    one of it stands on the slab at ({x:g}, {y:g}) ft, 3 by 1.75 by 2.875 ft, Assumed, set down by hand in the model")
PY
"$P" check "$J" >/dev/null || { echo "FAIL: the saved job does not check"; exit 1; }

echo
echo "== What was written out, beside the job =="
for f in "job.ifc" "job estimate.xlsx" "job estimate.csv" "job for suppliers.xlsx" "job for subcontractors.xlsx" "job for the PM.xlsx"; do
  [ -s "$T/$f" ] || { echo "FAIL: not written: $f"; ls "$T"; exit 1; }
  printf '    %s (%s bytes)\n' "$f" "$(stat -c %s "$T/$f")"
done
[ "$(grep -c '^#[0-9]*=IFCFURNITURE(' "$T/job.ifc")" = 1 ] || { echo "FAIL: the model does not hold one IfcFurniture"; exit 1; }
grep -q "IFCFURNITURE(.*'Harbor Vanity 36'" "$T/job.ifc" || { echo "FAIL: the vanity is not in the model by name"; exit 1; }
echo "    the BIM model holds one IfcFurniture, Harbor Vanity 36"
if python3 -c "import ifcopenshell" 2>/dev/null; then
  CHECKED="$(python3 tools/ifc-check.py "$T/job.ifc")" || { echo "FAIL: IfcOpenShell finds a fault in the model"; echo "$CHECKED"; exit 1; }
  grep -qF "IFC4: no fault" <<<"$CHECKED" || { echo "FAIL: not validated"; echo "$CHECKED"; exit 1; }
  echo "    IfcOpenShell: IFC4, no fault"
else
  echo "demos/itemsw.sh needs IfcOpenShell to validate the model (python3 -c 'import ifcopenshell' fails; set PYTHONPATH to where it is)"; exit 2
fi
python3 - "$T" <<'PY'
import sys, csv, openpyxl
t = sys.argv[1]
names = openpyxl.load_workbook(f"{t}/job estimate.xlsx").sheetnames
assert "Estimate" in names and "Recap" in names, names
print("    the workbook opens; its sheets:", ", ".join(names))
head = next(csv.reader(open(f"{t}/job estimate.csv")))
assert "Division" in head and "Amount" in head, head
print("    the CSV's headings:", ", ".join(head[:6]), "...")
for lens in ("for suppliers", "for subcontractors", "for the PM"):
    assert openpyxl.load_workbook(f"{t}/job {lens}.xlsx").sheetnames, lens
print("    the three lenses open")
PY
echo
echo "PASS: in the window, a maker's sheet became an item, one was set down with a click, and the BIM model, the workbook, the CSV and the three lenses were each written with a click"
