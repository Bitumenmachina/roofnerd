#!/usr/bin/env bash
# A thing picked up and moved in the model, with its clash live. A vanity made from a
# maker's sheet stands 3 in into a water closet. In the window it is selected, picked
# up with the left button and carried east until it is clear: red while it is in the
# water closet, its own colour when it is clear. The job is saved and read back at the
# command line, then the carry is undone, in one step.
# Run from the repository root:  demos/carryw.sh [folder for the pictures]
#
# It drives the real program with a real pointer on a virtual display (Xvfb, xdotool
# and a Vulkan driver; Mesa's software one will do). The sheet, the sizes and the job
# are made up. The figures are worked by hand in the comments and in STATUS.md ("A
# thing picked up and moved, with its clash live").
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/carryw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/carryw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/carry}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }

echo "== The job: a slab 12 ft by 8 ft, a water closet, and a vanity from its maker's sheet, 3 in into it =="
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
"$P" new "$J" "Carried (not a real job)" >/dev/null
L="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
FT="$(made "$("$P" type "$J" floor "slab 4 in" --thickness "4 in" "${K[@]}")")"
"$P" floor "$J" "$L" "$FT" --points "0,0 12,0 12,8 0,8" "${K[@]}" >/dev/null
ITEM="$(made "$("$P" import spec "$J" "$T/vanity.pdf")")"
# The water closet reaches from 4.25 to 5.75 ft east; the vanity, its middle at 7 ft and
# 3 ft wide, from 5.5 to 8.5: in each other by 0.25 ft, 3.00 in.
"$P" place "$J" "$L" fixture "Water closet" --at "5,1.125" --offset "0 ft" --size "1.5,2.25,2.5" "${K[@]}" >/dev/null
V="$(made "$("$P" place "$J" "$L" "$ITEM" --at "7,0.875" --status Assumed --source "set by eye")")"
OUT1="$("$P" clashes "$J")"; grep -F "in each other" <<<"$OUT1" | sed 's/^/  /'
grep -qF "in each other by 3.00 in" <<<"$OUT1" || { echo "FAIL: the vanity is not 3.00 in into the water closet to begin with"; exit 1; }

echo
echo "== The session: select the vanity; pick it up; carry it east; let go; save; undo; save =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
park; shot                              # 01 as opened: the vanity (wood) stands into the water closet (white)
click 1010 440; park; shot              # 02 the vanity selected: it is in the water closet, and is drawn red
xdotool mousemove 1010 440; sleep 1.5; xdotool mousedown 1; sleep 1.5
xdotool mousemove 1012 440; sleep 1; xdotool mousemove 1015 439; sleep 4
shot                                    # 03 in the hand, a tenth of a foot on: still in it, still red
for i in 1 2 3 4 5 6 7 8; do xdotool mousemove \$(( 1015 + 75 * i / 8 )) \$(( 439 - 19 * i / 8 )); sleep 0.7; done
sleep 4; xdotool mouseup 1; sleep 5
shot                                    # 04 let go a foot and a half east: clear, and its own colour
xdotool key F2; sleep 5
cp "$J" "$T/moved.json"
on "Undo"; sleep 4; park; shot          # 05 undone in one step: back where it was, and in the water closet
xdotool key F2; sleep 5
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
[ ! -e "$T/missing.txt" ] || { echo "FAIL: a control was not on screen"; cat "$T/missing.txt"; exit 1; }
for n in 01 02 03 04 05; do [ -s "$OUT/$n-model.png" ] || { echo "FAIL: no picture $n"; exit 1; }; done
[ -s "$T/moved.json" ] || { echo "FAIL: the job was not saved after the move"; exit 1; }

echo
echo "== What the pictures show =="
python3 - "$OUT" <<'PY'
import sys
from PIL import Image
out = sys.argv[1]
pic = lambda n: Image.open(f"{out}/{n:02d}-model.png").convert("RGB")
red = lambda c: c[0] > c[1] + 45 and c[0] > c[2] + 45          # drawn in the clash's red
wood = lambda c: c[0] > c[1] > c[2] and c[0] - c[2] > 40 and not red(c)
def most(im, box, test):                                        # how much of a patch passes a test
    x0, y0, x1, y1 = box
    dots = [im.getpixel((x, y)) for x in range(x0, x1, 3) for y in range(y0, y1, 3)]
    return sum(1 for d in dots if test(d)) / len(dots)
def fail(why): print("FAIL:", why); sys.exit(1)
face = (975, 410, 1045, 470)        # the vanity's south face, as opened
if most(pic(1), face, wood) < 0.6: fail(f"01: the vanity is not drawn in its wood: {pic(1).getpixel((1010, 440))}")
if most(pic(2), face, red) < 0.6: fail(f"02: selected and in the water closet, the vanity is not drawn red: {pic(2).getpixel((1010, 440))}")
if most(pic(3), face, red) < 0.6: fail(f"03: in the hand and still in the water closet, the vanity is not red: {pic(3).getpixel((1010, 440))}")
moved = (1055, 392, 1120, 450)      # the same face, carried 75 dots right and 19 up
if most(pic(4), moved, red) > 0.1: fail("04: clear of the water closet, the vanity is still drawn red")
if most(pic(5), face, red) < 0.6: fail("05: undone, the vanity is not back in the water closet and red")
print("    01 wood; 02 selected, red; 03 in the hand and still in it, red; 04 let go clear, not red; 05 undone, red again")
PY

echo
echo "== What was saved after the move, read at the command line =="
OUT2="$("$P" clashes "$T/moved.json")"; sed -n 1,2p <<<"$OUT2" | sed 's/^/  /'
grep -qF "In the same space: 0" <<<"$OUT2" || { echo "FAIL: the saved job still has the vanity in the water closet"; exit 1; }
python3 - "$T/moved.json" "$J" "$V" <<'PY'
import json, sys
def placed(path, who):
    e = next(e for e in json.load(open(path))["data"] if e["path"] == who)
    return e["attributes"]["pl::placed"]
after, undone = placed(sys.argv[1], sys.argv[3]), placed(sys.argv[2], sys.argv[3])
(x, y), at = after["at"]["value"], after["at"]
print(f"    the vanity's middle is at {x:.2f} ft east, {y:.2f} ft north ({at['status']}; {at['source']})")
# It was in by 0.25 ft, so it has gone more than that; the hand went east, hardly north.
assert x > 7.25 and x < 9.5, x
assert abs(y - 0.875) < 0.45, y
assert at["status"] == "Assumed" and at["source"] == "set by eye; moved (by hand in the model)", at
assert after["size"]["value"] == [3.0, 1.75, 2.875], after["size"]
print("    its size is still its sheet's: 3 by 1.75 by 2.875 ft")
# Undone in one step: exactly where it was set.
assert undone["at"]["value"] == [7.0, 0.875] and undone["at"]["source"] == "set by eye", undone["at"]
print("    after one Undo the saved job has it at (7, 0.875) again, set by eye")
PY
OUT3="$("$P" clashes "$J")"
grep -qF "in each other by 3.00 in" <<<"$OUT3" || { echo "FAIL: after Undo the clash is not back at 3.00 in"; exit 1; }
echo "    and the clash is back: in each other by 3.00 in"
echo
echo "PASS: a vanity was picked up in the model and carried clear of a water closet, red while it was in it; the move was saved as Assumed; and one Undo put it back"
