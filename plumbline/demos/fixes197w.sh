#!/usr/bin/env bash
# The window's six points of call 197, and what came with them (contract 197W): things set down listed in the
# sheet window (point 10), the Cabinets card (11), a corner refused in a run (12), a mark asked for and left (13),
# the view swung round the house (14), a maker's sheet's trade (15), an assembly changed for another saying what it
# rewrote (16, call 200 point 5), the window calling the model's `item_size` (17) and "Priced as" on an item (18,
# call 196). The steps and their figures are in STATUS.md, "The window's six points of call 197: a demonstration".
#
# Six sessions, each its own run of the program on its own made-up job:
#   a  an empty job with a level: the starter assemblies and tool chest, "Vanity, 36 in" twice and the starter water
#      heater twice; the sheet's Building list and Estimate; the heater priced as "Bought and set" from its panel; a
#      price typed on one of its lines
#   b  the same job, the same from the bench; and a size typed on the vanity (the model's `item_size`)
#   c  a room 14 ft by 10 ft: the Cabinets card; boxes typed before a run is started; the run; a corner refused; a mark
#      asked for and left; a maker's sheet made into an item
#   d  the wall of demos/change-assembly.sh changed for the wall over a stone wainscot, in the window
#   d2 the same with the siding's quantity typed by hand first
#   e  the trim demonstration's house from the home view: a swing begun on the ground, and one begun on a wall's corner
#
# Run from the repository root:  [PLUMBLINE_BIN=<folder>] [ONLY="a b c"] demos/fixes197w.sh [folder for the pictures]
# It drives the real program with a real pointer and keyboard on a virtual display (xvfb-run, xdotool, a Vulkan driver).
# The jobs and every figure are made up (no real job data). Every step fails the script if the program does not do what the
# step says. A picture is looked at by a person: what each shows is said in STATUS.md.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/fixes197w.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL" 2>/dev/null || { echo "demos/fixes197w.sh needs python's PIL, to read the pictures' sizes"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/fixes197w.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/fixes197w}"; mkdir -p "$OUT"
ONLY="${ONLY:-a b c d d2 e}"
T="$(mktemp -d)"; trap '[ -z "${FIXES_KEEP:-}" ] || cp -r "$T" "$FIXES_KEEP"; rm -rf "$T"' EXIT
K=(--status Known --source "made-up survey")
A=(--status Assumed --source "made-up tracing")
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; tail -40 "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; grep -F -- "$2" "$1" | head -5; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
wants() { case " $ONLY " in *" $1 "*) return 0;; *) return 1;; esac; }

# One session: its steps (a file of shell, using the helpers of demos/window.sh) run in the window on a job.
# $1 name, $2 job; the steps are in $T/steps-$1.sh; the pictures go in $OUT/$1.
session() {
  mkdir -p "$OUT/$1"
  rm -f "$T/map-$1.txt.said"
  cat > "$T/session-$1.sh" <<WRAP
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT/$1" ICD="$ICD" JOB="$2" NAME="$1"
. "$PWD/demos/window.sh"
# The map as it stands, kept under a name; the window's words so far, kept the same way.
keep_map() { sleep 3; cp "\$T/map.txt" "\$T/map-\$NAME-\$1.txt"; }
keep_said() { cp "\$T/map.txt.said" "\$T/said-\$NAME-\$1.txt"; }
. "$T/steps-$1.sh"
WRAP
  chmod +x "$T/session-$1.sh"
  rm -f "$T/missing.txt"
  timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session-$1.sh" || true
  if [ -s "$T/missing.txt" ]; then echo "FAIL: session $1 stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
  if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
  cp "$T/map.txt.said" "$T/said-$1.txt" 2>/dev/null || true
  rm -f "$T/map.txt" "$T/map.txt.said"
}

# ---------------------------------------------------------------------------------------------------------------------
if wants a || wants b; then
  echo "== The job: an empty job with a level =="
  mkdir -p "$T/a" "$T/b"
  "$P" new "$T/a/set.json" "Set down (not a real job)" >/dev/null
  printf 'Material,Unit,Price,How sure,Where from\n"Water heater, 50 gal",EA,1850,Assumed,a made-up list\n' > "$T/a/list.csv"
  "$P" level "$T/a/set.json" ground --elevation "0 ft" --height "9 ft" "${K[@]}" >/dev/null
fi

if wants a; then
echo
echo "== Session a: the vanity twice and the heater twice; the sheet's Building list and Estimate; the heater priced as Bought and set =="
cat > "$T/steps-a.sh" <<'STEPS'
open_window "$JOB"
sleep 8; park
on "Starter assemblies"; sleep 5
on "Starter tool chest"; sleep 5
on_at "Item Vanity, 36 in"; sleep 4
click 560 760; sleep 5
on "Item Vanity, 36 in"; sleep 4
click 900 760; sleep 5
esc; park
xdotool key F2; sleep 3
shot; keep_map 01                           # 01 two vanities set down: the top bar says 2 not priced
cp "$JOB" "$T/a-vanities.json"
sheet "Building"; sleep 4; park
shot; keep_map 02                           # 02 the sheet's Building list: 2 furnishings on the level
sheet "Estimate"; sleep 4; park
shot; keep_map 03                           # 03 the sheet's Estimate: 2 furnishings are set down and not priced
on_at "Item Water heater, 50 gal"; sleep 4
click 560 640; sleep 5
on "Item Water heater, 50 gal"; sleep 4
click 900 640; sleep 5
esc; park
xdotool key F2; sleep 3
cp "$JOB" "$T/a-heaters.json"
on_at "Change item Water heater, 50 gal"; sleep 5; park
on_right "Priced as" 5; sleep 3; park
shot; keep_map 04                           # 04 the heater's panel: Priced as says it is not priced; the list of what goes on equipment
on_right "Priced as Bought and set"; sleep 6; park
keep_said 05
shot; keep_map 05                           # 05 Bought and set picked: what the message says; the row reads it
sheet "Estimate"; sleep 5; park
shot; keep_map 06                           # 06 the sheet's Estimate: the heater's four lines
n=0
while [ -z "$(awk -F'\t' '$1=="sheet" && $2 ~ /^Quantity of Water heater, 50 gal on Water heater/ {print $2; exit}' "$PLUMBLINE_MAP")" ] && [ "$n" -lt 40 ]; do xdotool mousemove 2000 600; sleep 0.5; xdotool click 5; sleep 1.5; n=$((n + 1)); done
sleep 2; shot; keep_map 06b                 # 07 the heater's lines in view
# There is no field on the sheet for the price of a line made of a material: its price is the price list's. The window's way to a
# price for the heater is Bring in, Price list: a list of one row, the heater under the name the price list knows it by.
on "Price list"; typed "list.csv"; park
keep_said 08
shot; keep_map 08                           # 08 the price list brought in
sheet "Estimate"; sleep 5; park
shot; keep_map 09                           # 09 the sheet's Estimate: the price on both heaters
xdotool key F2; sleep 4
cp "$JOB" "$T/a-priced.json"
close_window
STEPS
session a "$T/a/set.json"
for n in 01 02 03 04 05 06 07 08 09; do
  [ "$OUT/a/$n-model.png" -nt "$T/steps-a.sh" ] && [ "$OUT/a/$n-sheet.png" -nt "$T/steps-a.sh" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  18 pictures saved to $OUT/a"
# 1 (point 10). By hand: two vanities, both on the first level, not priced: the sheet lists them there by kind and count, and
# the estimate says so; the top bar's count and `plumbline estimate` (the model's) give the same 2.
"$P" estimate "$T/a-vanities.json" > "$T/a-est.txt"
says "$T/a-est.txt" "nothing is priced on 2 things that could be (2 furnishings)"
python3 -I - "$T/map-a-02.txt" "$T/map-a-03.txt" <<'PY'
import sys
b = [l.split("\t") for l in open(sys.argv[1]) if l.startswith("sheet\t")]
e = [l.split("\t") for l in open(sys.argv[2]) if l.startswith("sheet\t")]
want = "Set down on Ground: 2 furnishings"
if not any(r[1] == want for r in b):
    sys.exit("FAIL: the sheet's Building list has no row %r: %s" % (want, [r[1] for r in b][:20]))
print("    the Building list reads: %s" % want)
want = "2 furnishings are set down and not priced."
if not any(r[1] == want for r in e):
    sys.exit("FAIL: the sheet's Estimate does not say %r: %s" % (want, [r[1] for r in e][:20]))
print("    the Estimate reads: %s" % want)
PY
# 10 (point 18). The heater, once priced: the message in the model's words, four lines, and one price on both.
says "$T/said-a-05.txt" "Water heater, 50 gal is now priced as \"Bought and set\"; 2 set down from it follow"
python3 -I - "$T/map-a-04.txt" "$T/map-a-05.txt" "$T/map-a-06b.txt" <<'PY'
import sys
def names(p, w): return [l.split("\t")[1] for l in open(p) if l.startswith(w + "\t")]
m4 = names(sys.argv[1], "model")
if "Priced as" not in m4 or "Priced as row" not in m4:
    sys.exit("FAIL: the item's panel has no Priced as row and button: %s" % [n for n in m4 if "Priced" in n])
if "Priced as Bought and set" not in m4:
    sys.exit("FAIL: the list of what goes on equipment does not hold Bought and set: %s" % [n for n in m4 if "Priced" in n])
print("    on the heater's panel: Priced as (not priced yet); the click lists what goes on equipment, Bought and set among it")
s6 = [l.split("\t")[1] for l in open(sys.argv[3]) if l.startswith("sheet\t")]
heater = [n for n in s6 if n.startswith("Quantity of Water heater")]
if len(heater) < 1:
    sys.exit("FAIL: the estimate does not show the heater's own line: %s" % heater)
print("    the estimate shows the heater's lines by their own name (the sheet lists the rows in view; the saved job is checked for all four below): %s" % ", ".join(heater))
PY
"$P" estimate "$T/a-heaters.json" > "$T/a-heaters.txt"
"$P" estimate "$T/a-priced.json" | tr -s ' ' > "$T/a-after.txt"
grep -c "Water heater, 50 gal 1.00 EA" "$T/a-after.txt" | grep -qx 2 || { echo "FAIL: the heater is not two lines of 1.00 EA"; cat "$T/a-after.txt"; exit 1; }
grep -c "Set in place no figure no price" "$T/a-after.txt" | grep -qx 2 || { echo "FAIL: the heater is not two lines of Set in place"; cat "$T/a-after.txt"; exit 1; }
printf '    four lines: the heater by its own name twice, Set in place twice\n'
says "$T/a-after.txt" 'Water heater, 50 gal 1.00 EA ~$1,850.00/EA ~$1,850.00'
[ "$(grep -c 'Water heater, 50 gal 1.00 EA ~\$1,850.00/EA ~\$1,850.00' "$T/a-after.txt")" = 2 ] || { echo "FAIL: the price typed on one line is not on both"; cat "$T/a-after.txt"; exit 1; }
says "$T/a-after.txt" 'Job cost ~$3,700.00 2 lines'
fi

# ---------------------------------------------------------------------------------------------------------------------
if wants b; then
echo
echo "== Session b: the heater priced as Bought and set from the bench; a size typed on the vanity =="
# The job of session b: the first job with the starter assemblies and chest, two vanities and two heaters (made at the command line
# here: it is session a's path, walked there through the buttons).
J="$T/b/set.json"
"$P" new "$J" "Set down (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
"$P" import assemblies "$J" starter >/dev/null
"$P" import items "$J" starter >/dev/null
for at in "5,5" "9,5"; do "$P" place "$J" "$S" "Vanity, 36 in" --at "$at" "${A[@]}" >/dev/null; done
for at in "5,9" "9,9"; do "$P" place "$J" "$S" "Water heater, 50 gal" --at "$at" "${A[@]}" >/dev/null; done
cat > "$T/steps-b.sh" <<'STEPS'
open_window "$JOB"
sleep 8; park
on_at "Change item Water heater, 50 gal"; sleep 5; park
on_right "Bench"; sleep 12; park
shot; keep_map 01                           # 01 the heater alone on the bench; its panel on the right
on_right "Priced as" 5; sleep 3; park
on_right "Priced as Bought and set"; sleep 6; park
keep_said 02
shot; keep_map 02                           # 02 priced as Bought and set from the bench
esc; sleep 12; park
on_at "Change item Vanity, 36 in"; sleep 5; park
on_right "Width"; sleep 3
typed "48 in"; sleep 8; park
keep_said 03
shot; keep_map 03                           # 03 the vanity 48 in wide (the model's item_size): said, and how many follow
xdotool key F2; sleep 4
cp "$JOB" "$T/b-after.json"
close_window
STEPS
session b "$J"
for n in 01 02 03; do
  [ "$OUT/b/$n-model.png" -nt "$T/steps-b.sh" ] || { echo "FAIL: no picture $n from this run"; exit 1; }
done
says "$T/said-b-02.txt" "Water heater, 50 gal is now priced as \"Bought and set\"; 2 set down from it follow"
# 9 (point 17): a size typed is the item changed: Assumed, in the words New item uses; two vanities follow; the name still reads 36 in.
says "$T/said-b-03.txt" "Vanity, 36 in: width is now 48 in (it was 36 in). Assumed: typed in the window; not yet checked against its maker's sheet. 2 things set down from it follow."
says "$T/said-b-03.txt" "Its name still reads 36 in: Rename item changes it."
"$P" check "$T/b-after.json" >/dev/null
"$P" estimate "$T/b-after.json" | tr -s ' ' > "$T/b-est.txt"
[ "$(grep -c 'Water heater, 50 gal 1.00 EA' "$T/b-est.txt")" = 2 ] || { echo "FAIL: the heater is not on the estimate twice after Bought and set from the bench"; cat "$T/b-est.txt"; exit 1; }
printf '    the same two heaters and four lines as from the panel; the saved job checks\n'
fi

# ---------------------------------------------------------------------------------------------------------------------
if wants c; then
echo
echo "== The room: 14 ft by 10 ft, walls 6 in, a Kitchen, a door in the south wall carrying the starter entry door's lines =="
mkdir -p "$T/c"
J="$T/c/room.json"
"$P" new "$J" "A room (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "6 ft 8 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 14,0 14,10 0,10" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
DOOR="$(made "$("$P" door "$J" "$SOUTH" "$DT" --offset "11 ft" --sill "0 ft" "${K[@]}")")"
"$P" room "$J" "$S" "Kitchen" --at "7,5" "${K[@]}" >/dev/null
"$P" import assemblies "$J" starter >/dev/null
"$P" import items "$J" starter >/dev/null
"$P" apply "$J" "$DOOR" "Entry door" --source "made-up" >/dev/null
"$P" check "$J" >/dev/null
echo "    the south wall is $SOUTH, the door $DOOR (with the entry door's lines), the room the Kitchen"
python3 - "$T/c/vanity.pdf" <<'PY'
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
cat > "$T/steps-c.sh" <<'STEPS'
open_window "$JOB"
on "Plan"; sleep 8; park
shot; keep_map 01                           # 01 the plan as opened
on "Cabinets"; sleep 10; park
shot; keep_map 02                           # 02 Cabinets in hand: the card, and the room above it
typed "B18" ; sleep 2                       # boxes typed before a run is started
keep_said 03
shot                                        # 03 what a stranger is told
at 1 0.25; sleep 3                          # the south wall's face, 1 ft along
keep_said 04
keep_map 04
shot                                        # 04 the run started
typed "B18 SB36 B24" yes                    # 05 the boxes typed (picture before Enter), then Enter
park; sleep 6
keep_said 06
shot                                        # 06 the run set
esc; esc; park; sleep 8
keep_map 06e                                # the tool put down: where the plan lies now
# A corner refused: a new wall laid along the north wall.
on "Wall"; sleep 3; xdotool key Return; sleep 3
at 4 10; sleep 3
at 10 10; sleep 3
keep_said 07
park; shot                                  # 07 refused, and where the run still starts
esc; esc; park
# A mark asked for, and left: the door carries lines, so To demolish asks.
at 11 0; sleep 3
on_right "To demolish"; sleep 3
keep_said 08
park; shot                                  # 08 the question: take its lines off, or keep them priced
at 3 10; sleep 3                            # something else clicked: the north wall
keep_said 09
park; shot                                  # 09 the door was not marked
xdotool key F2; sleep 4
cp "$JOB" "$T/c-after.json"
# A maker's sheet with no trade in it made into an item.
on "Maker's sheet"; typed "vanity.pdf"; park
keep_said 10
shot                                        # 10 the item, and that its trade is not said yet
close_window
STEPS
session c "$J"
for n in 01 02 03 04 05 06 07 08 09 10; do
  [ "$OUT/c/$n-model.png" -nt "$T/steps-c.sh" ] || { echo "FAIL: no picture $n from this run"; exit 1; }
done
echo "  pictures saved to $OUT/c"
# 2 (point 11).
python3 -I - "$T/map-c-02.txt" "$OUT/c/02-model.png" <<'PY'
import sys
from PIL import Image
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1])]
box = [r for r in rows if r[0] == "box" and r[1] == "Cabinets card"]
plan = [r for r in rows if r[0] == "plan"]
if not box or not plan:
    sys.exit("FAIL: the map has no Cabinets card or no plan: %s" % [r[:2] for r in rows if r[0] in ("box", "plan")])
l, t, r_, b = (float(x) for x in box[0][2:6])
x0, y0, k = (float(x) for x in plan[0][1:4])
X = lambda e: x0 + e * k
Y = lambda n: y0 - n * k
walls = {"south": (-0.25, 14.25, -0.25, 0.25), "north": (-0.25, 14.25, 9.75, 10.25), "west": (-0.25, 0.25, -0.25, 10.25), "east": (13.75, 14.25, -0.25, 10.25)}
w, h = Image.open(sys.argv[2]).size
print("    the model window is %d by %d dots; the card is %d..%d across and %d..%d down; %.2f dots to the foot" % (w, h, l, r_, t, b, k))
for name, (ea, eb, na, nb) in walls.items():
    xa, xb, ya, yb = X(ea), X(eb), Y(nb), Y(na)
    over = not (xb < l or xa > r_ or yb < t or ya > b)
    print("      %-5s wall %d..%d across, %d..%d down: %s" % (name, xa, xb, ya, yb, "UNDER THE CARD" if over else "clear of the card"))
    if over:
        sys.exit("FAIL: the %s wall is under the Cabinets card" % name)
    if ya < 50 or xa < 0 or xb > w:
        sys.exit("FAIL: the %s wall is off the window" % name)
print("    no wall of the room is under the card, and the room is on the window")
PY
# Where the plan lies as opened, with the tool in hand, and with the tool put down again (Esc): said, not held to.
python3 -I - "$T/map-c-01.txt" "$T/map-c-02.txt" "$T/map-c-06e.txt" <<'PY'
import sys
for what, f in zip(("as opened", "tool in hand", "tool put down (Esc)"), sys.argv[1:]):
    r = [l.rstrip("\n").split("\t") for l in open(f) if l.startswith("plan\t")]
    print("    plan %-22s zero at (%s, %s), %s dots to the foot" % (what, r[0][1], r[0][2], r[0][3]))
PY
says "$T/said-c-03.txt" "No run is started yet, so the boxes typed were not set. Click the wall's face where the run starts, on the room's side, then type the boxes, like B18 SB36 B24."
lacks "$T/said-c-03.txt" "reading"
lacks "$T/said-c-03.txt" "shoot"
# 3 (point 11, as before). 18 + 36 + 24 = 78 in = 6.5 ft; the countertop 6.5 x 25 1/2 in = 6.5 x 2.125 = 13.81 SF.
says "$T/said-c-06.txt" "Set 3 boxes along wall "
says "$T/said-c-06.txt" "a run of 6.5 ft, from 1 ft to 7.5 ft along the wall"
says "$T/said-c-06.txt" "Countertop: 6.5 ft long by 25 1/2 in deep, 34 1/2 in up; 13.81 SF, front edge 6.5 LF."
# 4 (point 12). The start is a place the tool writes, to the inch.
grep -E "Can't draw the wall: .*The run of walls still starts at \([0-9]+' [0-9 /]+\"?, 10' 0\"\): click the next corner, or press Esc to let it go\." "$T/said-c-07.txt" > "$T/c-07.txt" || { echo "FAIL: the refused corner does not say where the run still starts"; cat "$T/said-c-07.txt"; exit 1; }
printf '    %s\n' "$(head -1 "$T/c-07.txt" | cut -c1-260)"
# 5 (point 13).
says "$T/said-c-08.txt" "what comes down is not built. Say which: Take its lines off, or Keep its lines priced"
says "$T/said-c-09.txt" "was not marked to demolish: its question was left unanswered, and nothing was changed."
python3 -I - "$T/c-after.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))["data"]
marks = [e for e in d if "pl::work" in e["attributes"]]
if marks:
    sys.exit("FAIL: the saved job has a mark: %s" % [e["attributes"]["pl::work"] for e in marks])
print("    the saved job has no mark on anything")
PY
# 7 (point 15).
says "$T/said-c-10.txt" "Made an item from vanity.pdf: Harbor Vanity 36"
says "$T/said-c-10.txt" "Its trade is not said yet: pick it under Trade, on the right, with the item selected."
fi

# ---------------------------------------------------------------------------------------------------------------------
change_job() { # $1 folder name
  mkdir -p "$T/$1"; local J="$T/$1/wall.json"
  "$P" new "$J" "Change (not a real job)" >/dev/null
  "$P" level "$J" ground --elevation "0 ft" --height "10 ft" "${K[@]}" >/dev/null
  "$P" type "$J" wall w6 --thickness "6 in" "${K[@]}" >/dev/null
  "$P" wall "$J" level-0002 type-0003 --points "0,0 20,0 20,16 0,16" --closed --height "10 ft" "${K[@]}" >/dev/null
  "$P" import assemblies "$J" starter >/dev/null
  "$P" apply "$J" wall-0004 "Exterior wall, 2x6, lap siding" --source "made-up plan" >/dev/null
  "$P" set "$J" wall-0004 unit_cost '$4.00' --line "Lap siding" "${K[@]}" >/dev/null
}
change_steps() {
cat > "$T/steps-$1.sh" <<'STEPS'
open_window "$JOB"
on "Plan"; sleep 8; park
at 10 0; sleep 3                            # the first wall of the ring, 20 ft long
keep_said 01
shot                                        # 01 the wall selected: lap siding on it
on_right "Assembly"; sleep 3; park
shot                                        # 02 the assemblies that go on a wall
on_right "Assembly Exterior wall, 2x6, lap siding over a stone wainscot"; sleep 8; park
keep_said 03
shot                                        # 03 changed: the message names each line
xdotool key F2; sleep 4
cp "$JOB" "$T/$NAME-after.json"
close_window
STEPS
}
if wants d; then
echo
echo "== Session d: the wall of demos/change-assembly.sh, changed in the window for the wall over a stone wainscot =="
change_job d
change_steps d
session d "$T/d/wall.json"
for n in 01 02 03; do [ "$OUT/d/$n-model.png" -nt "$T/steps-d.sh" ] || { echo "FAIL: no picture $n from this run"; exit 1; }; done
# 8 (point 16). Lap siding 200 - 20 x 3 = 140.00 SF; Hang the siding the same; Corner boards 2 x (10 - 3) = 14.00 LF.
says "$T/said-d-03.txt" "3 cost lines it already had now read as the new assembly's quantity, and kept its price, its waste and how it is ordered:"
says "$T/said-d-03.txt" "Lap siding: 200.00 SF to 140.00 SF"
says "$T/said-d-03.txt" "Hang the siding: 200.00 SF to 140.00 SF"
says "$T/said-d-03.txt" "Corner boards: 20.00 LF to 14.00 LF"
lacks "$T/said-d-03.txt" "left as entered"
"$P" estimate "$T/d-after.json" | tr -s ' ' > "$T/d-est.txt"
says "$T/d-est.txt" 'Lap siding ~140.00 SF $4.00/SF ~$560.00'
fi
if wants d2; then
echo
echo "== Session d2: the same, with the siding's quantity typed first as net_area * 0.9 =="
change_job d2
"$P" set "$T/d2/wall.json" wall-0004 quantity "net_area * 0.9" --line "Lap siding" "${K[@]}" >/dev/null
change_steps d2
session d2 "$T/d2/wall.json"
for n in 01 02 03; do [ "$OUT/d2/$n-model.png" -nt "$T/steps-d2.sh" ] || { echo "FAIL: no picture $n from this run"; exit 1; }; done
# (The window says a formula in a builder's words: an underscore is a space.)
says "$T/said-d2-03.txt" "Lap siding: left as entered, 180.00 SF (net area * 0.9); the new assembly writes net area - net area below wainscot + gable area, wainscot = 3 ft, which comes to 140.00 SF"
says "$T/said-d2-03.txt" "Corner boards: 20.00 LF to 14.00 LF"
"$P" estimate "$T/d2-after.json" | tr -s ' ' > "$T/d2-est.txt"
# (The amount reads Assumed: the window gives the new assembly as Assumed. The quantity stays 180.00 SF, as typed.)
says "$T/d2-est.txt" 'Lap siding 180.00 SF $4.00/SF ~$720.00'
fi

# ---------------------------------------------------------------------------------------------------------------------
if wants e; then
echo
echo "== The house of demos/trimw.sh: 24 by 16 ft, a gable roof, a window and a door in the south wall, its trim lines given =="
mkdir -p "$T/e"
J="$T/e/house.json"
"$P" new "$J" "Outside trim (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${A[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${A[@]}")")"
NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${A[@]}")")"
DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "6 ft 8 in" "${A[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${A[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
WIN="$(made "$("$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "2.5 ft" "${A[@]}")")"
DOOR="$(made "$("$P" door "$J" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${A[@]}")")"
ROOF="$(made "$("$P" roof "$J" "Main roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 6:12 --eave-height "9 ft" "${A[@]}")")"
"$P" set "$J" "$ROOF" edge:2 gable "${A[@]}" >/dev/null
"$P" set "$J" "$ROOF" edge:4 gable "${A[@]}" >/dev/null
EAVES="$(grep -oE 'eave-[0-9]+' <<<"$("$P" show "$J")" | sort -u | tr '\n' ' ')"
RAKES="$(grep -oE 'rake-[0-9]+' <<<"$("$P" show "$J")" | sort -u | tr '\n' ' ')"
"$P" import assemblies "$J" starter >/dev/null
L="the starter library"
for e in $EAVES; do "$P" apply "$J" "$e" "Gutter" --source "$L" >/dev/null; done
for r in $RAKES; do "$P" apply "$J" "$r" "Rake trim" --source "$L" >/dev/null; done
for w in $SOUTH $EAST $NORTH $WEST; do "$P" apply "$J" "$w" "Exterior wall, 2x6, lap siding" --source "$L" >/dev/null; done
"$P" apply "$J" "$WIN" "Window" --source "$L" >/dev/null
"$P" apply "$J" "$DOOR" "Entry door" --source "$L" >/dev/null
"$P" check "$J" >/dev/null
cat > "$T/steps-e.sh" <<'STEPS'
# A swing: the pointer put somewhere, the right button down, then so many dots along, a step at a time.
swing() { # x y dx steps
  xdotool mousemove "$1" "$2"; sleep 2
  xdotool mousedown 3; sleep 1.2
  for i in $(seq 1 "$4"); do xdotool mousemove_relative -- "$3" 0; sleep 0.4; done
  sleep 1; xdotool mouseup 3; sleep 10
}
corner() { # the middle of the nearest corner board, from the map: its foot lowest on the screen
  awk -F'\t' '$1=="trim" && $2=="corner boards" {y=($5>$7)?$5:$7; if (y>best) {best=y; fx=($5>$7)?$4:$6; fy=($5>$7)?$5:$7; tx=($5>$7)?$6:$4; ty=($5>$7)?$7:$5}} END {printf "%d %d", (fx+tx)/2, (fy+ty)/2}' "$1"
}
open_window "$JOB"
sleep 6; park
keep_map 00
shot                                        # 01 the home view
read -r px py <<<"$(awk -F'\t' '$1=="middle" {print int($2), int($3)}' "$T/map-$NAME-00.txt")"
echo "$px $py" > "$T/e-middle.txt"
# A swing begun on the ground, clear of the house: a quarter turn (a window height of dots is a whole turn: 240 dots is a quarter of 960).
read -r gx gy <<<"$(cat "$T/e-ground.txt" 2>/dev/null || echo "1150 700")"
swing "$gx" "$gy" 40 6
park
keep_map 01
shot                                        # 02 a quarter turn from the ground: the house stays where it was, turned
# Back to the home view, then a swing begun on a wall's corner.
xdotool key f; sleep 12; park
keep_map 02
shot                                        # 03 the home view again
read -r cx cy <<<"$(corner "$T/map-$NAME-02.txt")"
echo "$cx $cy" > "$T/e-corner.txt"
xdotool mousemove "$cx" "$cy"; sleep 2
xdotool mousedown 3; sleep 1.2
for i in 1 2 3; do xdotool mousemove_relative -- 40 0; sleep 0.4; done
sleep 1.5; keep_map 03                      # held: mid-turn
for i in 4 5 6; do xdotool mousemove_relative -- 40 0; sleep 0.4; done
sleep 1.5; keep_map 04                      # held: the quarter turn done
xdotool mouseup 3; sleep 10; park
keep_map 05
shot                                        # 04 a quarter turn about the corner
close_window
STEPS
session e "$J"
for n in 01 02 03 04; do [ "$OUT/e/$n-model.png" -nt "$T/steps-e.sh" ] || { echo "FAIL: no picture $n from this run"; exit 1; }; done
python3 -I - "$T" "$OUT/e/01-model.png" <<'PY'
import sys
from PIL import Image
t, shot = sys.argv[1], sys.argv[2]
w, h = Image.open(shot).size
def line(name, tag):
    for l in open(f"{t}/map-e-{name}.txt"):
        f = l.rstrip("\n").split("\t")
        if f[0] == tag:
            return float(f[1]), float(f[2])
    sys.exit(f"FAIL: map {name} has no {tag}")
home = line("00", "middle")
turned = line("01", "middle")
print("    the window is %d by %d dots; the middle of the house at the home view: (%d, %d)" % (w, h, home[0], home[1]))
print("    after a quarter turn begun on the ground, clear of the house: (%d, %d)" % turned)
# The view really turned: the house's south-west foot on the ground (the map's `house`) moved a long way.
h0, h1 = line("00", "house"), line("01", "house")
moved = ((h0[0] - h1[0]) ** 2 + (h0[1] - h1[1]) ** 2) ** 0.5
print("    the house's south-west foot went from (%d, %d) to (%d, %d): %d dots" % (h0 + h1 + (moved,)))
if moved < 150:
    sys.exit("FAIL: the swing did not turn the view (the foot of the house moved %d dots)" % moved)
if not (w / 3 <= turned[0] <= 2 * w / 3 and h / 3 <= turned[1] <= 2 * h / 3):
    sys.exit("FAIL: the middle of the house left the middle third of the window (%d..%d across, %d..%d down)" % (w / 3, 2 * w / 3, h / 3, 2 * h / 3))
print("    it is in the middle third of the window each way (%d..%d across, %d..%d down)" % (w / 3, 2 * w / 3, h / 3, 2 * h / 3))
# The corner swing: the board's middle stays where the pointer was put.
cx, cy = (float(x) for x in open(f"{t}/e-corner.txt").read().split())
for name in ("03", "04", "05"):
    # The nearest board is read again from the map (the same board: the one whose foot is lowest).
    best, pos = -1e9, None
    for l in open(f"{t}/map-e-{name}.txt"):
        f = l.rstrip("\n").split("\t")
        if f[0] == "trim" and f[1] == "corner boards":
            x1, y1, x2, y2 = (float(v) for v in f[3:7])
            mid = ((x1 + x2) / 2, (y1 + y2) / 2)
            d = (mid[0] - cx) ** 2 + (mid[1] - cy) ** 2
            if pos is None or d < best:
                best, pos = d, mid
    off = best ** 0.5
    print("    the corner board begun on, after %s: %.1f dots from where the pointer was put" % ({"03": "half the turn", "04": "the quarter turn", "05": "the button let go"}[name], off))
    if off > 6.0:
        sys.exit("FAIL: the corner left the pointer by %.1f dots (6 allowed)" % off)
print("    the corner stays within 6 dots of the pointer through the turn")
PY
fi

echo
echo "== The words, searched =="
for f in "$T"/said-*.txt; do
  [ -e "$f" ] || continue
  if grep -qE 'furnishing-0|wall-0|door-0|Entered|Err\(|panicked' "$f"; then echo "FAIL: the window said a program word in $f"; grep -E 'furnishing-0|wall-0|door-0|Entered|Err\(' "$f" | head; exit 1; fi
done
echo "  none of what the window said holds a file's shorthand"
echo
echo "fixes197w: every step said what it should."
