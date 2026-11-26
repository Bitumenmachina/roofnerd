#!/usr/bin/env bash
# A renovation marked in the window (DECISIONS.md call 149; the command line's half is
# demos/reno.sh). A house as it stands (24 ft by 16 ft, a partition, one room "Living"),
# opened in the window on its plan: the partition picked and "To demolish" pressed, an
# outside wall picked and "Existing" pressed, the switch "Demolition" off and on, the job
# saved. What the window said, the saved job and the pictures are each checked.
# Run from the repository root:  demos/renow.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display, which
# needs Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job is made
# up (no real job data). Every step fails the script if the program does not do what it
# says. Worked by hand (as reno.sh step 4): 23.5 ft by 15.5 ft inside the faces is
# 364.25 SF, where the room was 178.25 SF with the partition standing.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/renow.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/renow.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"; APP="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/renow}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up survey")
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; head -60 "$1"; exit 1; }; printf '    %s\n' "$2"; }
block() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show.txt" | sed '$d'; }

echo "== The house as it stands: 24 ft by 16 ft, a partition, one room (reno.sh step 1) =="
"$P" new "$J" "Renovation in the window (not a real job)" >/dev/null
S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART="$("$P" wall "$J" "$S" "$WT" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
R="$(made "$("$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}")")"
# The south wall is brick, so that it carries a cost line (what the window then asks) and
# has a colour of its own to be washed pale when it is marked existing.
"$P" line "$J" "$SOUTH" "Brick veneer" net_area --cost-type material --division 4 >/dev/null
"$P" show "$J" > "$T/show.txt"; block "$R" > "$T/room.txt"
says "$T/room.txt" "floor area       178.25 SF"
cp "$J" "$T/before.json"

echo
echo "== The session: on the plan, the partition to demolish, the south wall existing, Demolition off and on =="
# At each shot the pointer is parked and nothing is selected (a click on bare ground lets
# go), so that what is measured is the wall and not its highlight.
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
sleep 8; park
shot                                     # 01 the house in 3D as opened: nothing marked
on "Plan"; sleep 8; park
echo "\$(plan_px 12 4) \$(plan_px 6 0) \$(plan_px 3 0) \$(plan_px 9 0) \$(plan_px 12 3) \$(plan_px 12 13) \$(plan_line) \$(plan_px 6 16)" > "$T/px.txt"
shot                                     # 02 the plan as opened: nothing marked
at 12 4; sleep 3                         # the partition
on "To demolish"; sleep 4; park
shot                                     # 03 the partition picked, its WORK row: "To demolish" lit
click 1000 800; sleep 3
shot                                     # 04 nothing picked: the partition a faint red ghost
at 6 0; sleep 3                          # the south wall
on "Existing"; sleep 4; park
shot                                     # 05 refused, the wall carries a line: two buttons in the row's place, the sentence on the card
on "Keep its lines"; sleep 4; park
shot                                     # 06 "Keep its lines" pressed: "Existing" lit, the line kept
click 1000 800; sleep 3
cp "$T/map.txt" "$T/map-keys.txt"
shot                                     # 07 nothing picked: the south wall washed pale, the ghost still there; the two key rows under Show
on "Show Demolition"; sleep 4
click 1000 800; sleep 3
shot                                     # 08 Demolition off: the house as the work leaves it
on "Show Demolition"; sleep 4
click 1000 800; sleep 3
shot                                     # 09 Demolition on again: the ghost is back
on "3D"; sleep 8; park
shot                                     # 10 the same house in 3D: the south wall in halftone, the ghost
on "Show Existing, shown pale"; sleep 4; park
shot                                     # 11 its switch off: existing drawn as new work is
on "Show Existing, shown pale"; sleep 4; park
xdotool key F2; drawn_for 3 3                  # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 07 08 09 10 11; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  22 pictures saved to $OUT (model and sheet, after each of 11 steps)"

echo
echo "== What the window said =="
cp "$T/map.txt.said" "$T/said.txt"
says "$T/said.txt" ": to demolish"
says "$T/said.txt" "Say which: Take its lines off, or Keep its lines"
says "$T/said.txt" ": existing to remain; its 1 cost line kept and priced, as work done to it"
# The partition marked to demolish opens the room it parted: the window says so, with the figures
# before and after (178.25 SF with the partition standing; 23.5 ft by 15.5 ft = 364.25 SF without).
says "$T/said.txt" "Living is now 364.25 SF (was 178.25)"
says "$T/said.txt" "Demolition: switched off."
says "$T/said.txt" "Demolition: switched on."
if grep -qE -- '--clear|--keep' "$T/said.txt"; then echo "FAIL: the window gave the command line's words"; exit 1; fi
# A thing is named once: "Wall 5 (wall 5)" was the name and then the name again (call 161).
if grep -qE '([Ww]all [0-9]+) \(([Ww]all [0-9]+)\)' "$T/said.txt"; then echo "FAIL: a wall is named twice in what the window said"; grep -E '([Ww]all [0-9]+) \(([Ww]all [0-9]+)\)' "$T/said.txt"; exit 1; fi
# The two key rows under Show, once anything is marked (call 161), from the map at picture 07:
# "Existing, shown pale" and, with its switch, "Demolition, shown red".
grep -q $'^model\tShow Existing, shown pale\t' "$T/map-keys.txt" && grep -q $'^model\tShow Demolition\t' "$T/map-keys.txt" || { echo "FAIL: the two key rows (Existing, Demolition) are not under Show"; grep -E 'Show|Key' "$T/map-keys.txt"; exit 1; }
echo "    under Show: Existing, shown pale and Demolition, shown red, each with its switch (call 178)"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
"$P" show "$J" > "$T/show.txt"
block "$PART" > "$T/part.txt"; block "$SOUTH" > "$T/south.txt"; block "$R" > "$T/room.txt"
says "$T/part.txt" "work             to demolish"
says "$T/south.txt" "work             existing to remain"
says "$T/south.txt" "Brick veneer"
says "$T/room.txt" "floor area       364.25 SF"

echo
echo "== Several things at once, a wall the work opens, a line taken off, a measure typed (call 169) =="
# The same house in a second job. Worked by hand: the partition down opens the Living room
# to 23.5 ft by 15.5 ft inside the faces, 364.25 SF where it was 178.25 SF with the partition
# standing. Receptacles (NEC 210.52(A): one for every 12 ft or part of a room with no door, all
# the way round): 11.5 by 15.5 ft inside is 54 ft round, 4.5, so 5; 23.5 by 15.5 is 78 ft round,
# 6.5, so 7.
J2="$T/job2.json"; M="$OUT/multi"; mkdir -p "$M"
"$P" new "$J2" "Renovation, several at once (not a real job)" >/dev/null
S2="$(made "$("$P" level "$J2" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT2="$(made "$("$P" type "$J2" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
read -r SOUTH2 EAST2 NORTH2 WEST2 <<<"$("$P" wall "$J2" "$S2" "$WT2" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
PART2="$("$P" wall "$J2" "$S2" "$WT2" --points "12,0 12,16" --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | head -1)"
"$P" room "$J2" "$S2" "Living" --at "6,8" "${K[@]}" >/dev/null
"$P" line "$J2" "$SOUTH2" "Brick veneer" net_area --cost-type material --division 4 >/dev/null
touch "$T/started2"
cat > "$T/session2.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$M" ICD="$ICD"
. "$PWD/demos/window.sh"
LOG="$T/app2.log"
open_window "$J2"
sleep 8; park
on "Plan"; sleep 8; park
echo "\$(plan_px 24 8) \$(plan_px 12 8) \$(plan_px 6 0) \$(plan_line)" > "$T/px2.txt"
cp "$T/map.txt" "$T/map-0.txt"
shot                                      # 01 nothing marked, nothing selected: no key rows under Show
at 6 0; sleep 3                           # the south wall
shift_at 24 8; shift_at 12 8              # Shift and a click: the east wall, the partition
shot                                      # 02 three picked: "and 2 more", the three outlined
on "Existing"; sleep 4; park              # the south wall carries a line: asked
on "Keep its lines"; sleep 4; park
shot                                      # 03 all three marked existing, in one step
click 1000 800; sleep 3
cp "$T/map.txt" "$T/map-1.txt"
on "Undo"; sleep 4; park                  # one Undo takes the whole mark back
cp "$T/map.txt" "$T/map-2.txt"
shot                                      # 04 none of the three is marked
xdotool key F2; drawn_for 3 3
cp "$J2" "$T/after-undo.json"
at 12 4; sleep 3                          # the partition alone
on "To demolish"; sleep 4; park
cp "$T/map.txt" "$T/map-3.txt"
shot                                      # 05 the partition to demolish: Living and its receptacles said
on "Bearing"; sleep 4; park               # the wall the work opens: asked, and said from the panel
shot                                      # 06 Behind the wall: bearing, and what a bearing wall needs
on "In the wall"; sleep 3
typed "a 2 in drain" yes; sleep 2         # 07 typed, then Enter
park
shot                                      # 08 both said
at 6 0; sleep 3
on "Select all on this level"; sleep 4; park
shot                                      # 09 every wall of the level picked from one button
on "Existing"; sleep 4; park
on "Keep its lines"; sleep 4; park
shot                                      # 10 five marked in one step
on "Undo"; sleep 4; park                  # one Undo takes all five back
click 1000 800; sleep 3
at 6 16; sleep 3
shift_at 0 8; sleep 2
on "Delete"; sleep 4; park                # Delete acts on both, in one step
shot                                      # 11 two walls gone
on "Undo"; sleep 4; park
click 1000 800; sleep 3
at 6 0; sleep 3
on_right "Take line off: Brick veneer"; sleep 4; park
shot                                      # 12 the line is off the south wall
on "Undo"; sleep 4; park                  # put back, by one Undo
on "Add cost line"; sleep 3
typed "Patch and match siding"; typed "60 SF"; typed "material" yes
sleep 3; park
shot                                      # 13 a measure typed as the quantity
xdotool key F2; drawn_for 3 4
close_window
SESSION
chmod +x "$T/session2.sh"
timeout 1800 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session2.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the second session stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app2.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app2.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 08 09 10 11 12 13; do
  [ "$M/$n-model.png" -nt "$T/started2" ] || { echo "FAIL: no picture $n of the second session"; exit 1; }
done
echo "  pictures saved to $M"
SAID2="$T/map.txt.said"
says "$SAID2" "Added wall "
says "$SAID2" " things are selected."
says "$SAID2" "Marked 3 things: existing to remain"
says "$SAID2" "Undone: marked existing"
says "$SAID2" "Living is now 364.25 SF (was 178.25), and needs 7 receptacles (was 5)"
says "$SAID2" " is bearing. Assumed: entered on screen"
says "$SAID2" ": a 2 in drain. Assumed: entered on screen"
says "$SAID2" "Selected all 5 walls on Ground"
says "$SAID2" "Marked 5 things: existing to remain"
says "$SAID2" "Deleted 2 things"
says "$SAID2" "Took \"Brick veneer\" off wall "
says "$SAID2" "Its quantity, 60 SF, is Assumed: entered on screen"
# "Existing, shown pale" is under Show only once something is marked existing; the Demolition
# switch only once something comes down.
key() { grep -q $'^model\t'"$2"$'\t' "$1"; }
! key "$T/map-0.txt" "Show Existing, shown pale" && ! key "$T/map-0.txt" "Show Demolition" || { echo "FAIL: a key row is there with nothing marked"; exit 1; }
key "$T/map-1.txt" "Show Existing, shown pale" || { echo "FAIL: no Existing key row with three marked existing"; exit 1; }
! key "$T/map-2.txt" "Show Existing, shown pale" || { echo "FAIL: the Existing key row is still there after Undo"; exit 1; }
key "$T/map-3.txt" "Show Demolition" && ! key "$T/map-3.txt" "Show Existing, shown pale" || { echo "FAIL: with only a demolition marked, the Demolition switch should show and Existing should not"; exit 1; }
echo "    under Show: no key rows with nothing marked; Existing once three are existing, gone with Undo; only Demolition with a demolition alone"
"$P" show "$T/after-undo.json" > "$T/show-undo.txt"
if grep -q "existing to remain" "$T/show-undo.txt"; then echo "FAIL: one Undo left a wall marked existing"; exit 1; fi
echo "    one Undo took all three marks back (the job saved after it has none)"
"$P" check "$J2"; "$P" show "$J2" > "$T/show2.txt"
block2() { sed -n "/^ *$1  /,/^ *[a-z]*-[0-9]*  /p" "$T/show2.txt" | sed '$d'; }
block2 "$PART2" > "$T/part2.txt"; block2 "$SOUTH2" > "$T/south2.txt"
says "$T/part2.txt" "to demolish"
# (`show` marks an Assumed value with ~)
says "$T/part2.txt" "~ bearing          bearing"
says "$T/part2.txt" "~ in the wall      a 2 in drain"
says "$T/south2.txt" "Patch and match siding"
says "$T/south2.txt" "Brick veneer"
grep -q "60 SF" "$T/south2.txt" || { echo "FAIL: the typed 60 SF is not on the line"; cat "$T/south2.txt"; exit 1; }
grep -q "is bearing and comes down" "$T/show2.txt" || { echo "FAIL: the saved job does not say what a bearing wall that comes down needs"; exit 1; }
echo "    saved: the partition to demolish, bearing and what is in it Assumed (~) from the screen, what a bearing wall needs said; the south wall with its brick and the typed 60 SF"
python3 -I - "$M" "$T/px2.txt" <<'PY'
import sys
from PIL import Image
m, v = sys.argv[1], [float(x) for x in open(sys.argv[2]).read().split()]
east = (v[0], v[1])
def blue(name, cx, cy, w=26, h=60):
    im = Image.open(f"{m}/{name}-model.png").convert("RGB").crop((int(cx - w / 2), int(cy - h / 2), int(cx + w / 2), int(cy + h / 2)))
    return sum(1 for p in im.getdata() if p[2] - p[0] > 80)
# The wall is 16 pixels across: a window 26 wide takes in both its edges, where the outline is drawn.
a, b = blue("01", *east), blue("02", *east)
print("    pixels of the picked blue along the east wall: unpicked (01) %d, picked with Shift (02) %d" % (a, b))
good = a < 5 and b > 40
print("    outlined blue once picked along with the others: %s" % ("as wanted" if good else "FAIL"))
sys.exit(0 if good else 1)
PY
echo
echo "== The pictures, measured =="
# Boxes: the partition on the plan is worked from where the program says the plan lies on
# screen ($T/px.txt: the partition's two ends 3 and 13 ft up, and the plan's scale); the
# south wall in 3D is a patch of its brick face, the eye being where it was at the start.
# As measured on a run of these steps (mean colour of the box):
#   south wall in 3D: unmarked (01) (122, 85, 75), brick red; existing (10) (159, 162, 164)
#     (the brick washed toward pale gray, its pattern left off)
#   partition on the plan: as opened (02) (244, 242, 238), paper; ghost on (04) (234, 182, 182);
#     Demolition off (08) (248, 248, 248); on again (09) (233, 182, 183)
# Each test was run against a picture from before the step (01 for the wall's second
# figure; 02 for the partition's ghost) and each failed there.
python3 -W ignore -I "$PWD/demos/renow-pictures.py" "$OUT" "$T/px.txt"
echo
echo
echo "== A question is blue (call 178) =="
python3 -W ignore -I "$PWD/demos/cardcolour.py" "$OUT/05-model.png" blue
echo "  PASS: in the window, on the plan, the partition was marked to demolish and a south wall existing, each lit in its WORK row and said in words; Demolition off took the ghost away and on brought it back; the saved job has the partition to demolish and the room it leaves at 364.25 SF; the pictures show the existing wall lighter and the partition a red ghost"
