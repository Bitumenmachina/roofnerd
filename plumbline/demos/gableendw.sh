#!/usr/bin/env bash
# The gable drawn (call 192, the window's half). The demonstration is written in STATUS.md, "The gable drawn: a demonstration,
# written before it is built", with its figures worked by hand. Where a gable end is a rake the model gives each such wall its
# top as a line of points (`plumbline gable`); the window draws the wall, and what it is dressed in on its outside, up to that
# line and works nothing out: no height and no area. A wall under an eave is drawn as it was.
#
# The house of demos/trimw.sh (24 by 16 ft, 9 ft walls, a gable roof 1 ft out all round, ridge east-west, 6 in 12, the starter
# lap siding on every wall, a window and a door in the south wall, its trim lines given) is made at the command line and
# opened. Where the gable's middle lies on screen is read from the program's own map (`gable<TAB>wall<TAB>x<TAB>y<TAB>apex x<TAB>
# apex y<TAB>feet along<TAB>feet up<TAB>wall x<TAB>wall y`: a place halfway up between the wall's top and the roof over the middle
# of the wall, and a place on the wall's own face halfway up), and what the wall's panel says of its gable is read from
# `gable_panel<TAB>wall<TAB>figure<TAB>status`.
#
# Run from the repository root:
#   [PLUMBLINE_BIN=<folder of the program>] [GABLE_BEFORE_BIN=<folder of the program before this work>] demos/gableendw.sh [folder]
# With GABLE_BEFORE_BIN the same views are taken on the program before, beside these, and the hip-roofed house is compared with
# its picture there dot for dot. Without it those are said not to have been made, and nothing is compared.
# Pictures (the model window; the program numbers a picture by the order it was taken in, and they are named here):
#   01-home          the house from the street corner, as the window opens: the west gable end at the left, closed and sided
#   02-west          the same from the street at the west end, the gable end face on
#   03-west-selected the west wall picked by a click in its gable: its panel reads Gable 40.00 SF, Assumed
#   04-west-bare     the same with Finishes off: the weather barrier on the gable as on the wall under it
#   05-south         the south wall picked: its panel has no Gable
#   06-shed-home     the same walls under a shed roof, from the street corner
#   07-shed-north    the north wall, under the high edge, 17 ft 6 in to the roof along its whole length, picked: Gable 204.00 SF
#   08-hip-home      the same walls under a hip roof: the picture of the program before, dot for dot
#   before-01, before-02, before-08 (with GABLE_BEFORE_BIN): the views 01, 02 and 08 on the program before
#
# Worked by hand (STATUS.md):
#  - Gable roof: the west and the east wall each 16 ft; the roof over the wall's line is 9 ft 6 in at each end (9 ft and 6 in of
#    rise over the 1 ft overhang) and 13 ft 6 in at the middle (8 ft in, 4 ft of rise): top (0, 9.5) (8, 13.5) (16, 9.5), area
#    16 x 0.5 + 16 x 4 / 2 = 8 + 32 = 40.00 SF. The probe halfway up at the middle is (9 + 13.5) / 2 = 11 ft 3 in. The lap siding
#    on the west wall prices 144 + 40 = 184.00 SF (the wall's net face, and its gable).
#  - Shed roof (south eave, rising north): the north wall's top is level at 9 + 17 x 0.5 = 17.5 ft over its whole 24 ft, 8.5 ft
#    above its own top: 24 x 8.5 = 204.00 SF. The east and west walls: 16 x (0.5 + 8.5) / 2 = 72.00 SF.
#  - Measured: a patch about the probe reads the colour of the wall's own siding (within a few levels), and on the program before
#    reads the sky or the roof; the hip house is the program before's picture dot for dot; the south wall has no gable line.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/gableendw.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL, numpy" 2>/dev/null || { echo "demos/gableendw.sh needs python's PIL and numpy (the pictures are measured)"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/gableendw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
BEFORE_APP=""; [ -n "${GABLE_BEFORE_BIN:-}" ] && BEFORE_APP="$GABLE_BEFORE_BIN/plumbline-app"
OUT="${1:-$PWD/demos/captured/gableendw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap '[ -z "${GABLEEND_KEEP:-}" ] || cp -r "$T" "$GABLEEND_KEEP"; rm -rf "$T"' EXIT
A=(--status Assumed --source "made-up tracing")
L="the starter library"
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
says() { grep -qF -- "$2" <<<"$3" || { echo "FAIL: $1: not said: $2"; echo "$3" | head -40; exit 1; }; printf '    %s\n' "$2"; }

# The house: house <file> <edges: P (pitch) or G (gable) for each of the four edges of the roof, south first>
house() {
  local J="$1" edges="$2" i=0 e R
  "$P" new "$J" "Gable end (not a real job)" >/dev/null
  S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${A[@]}")")"
  WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${A[@]}")")"
  NT="$(made "$("$P" type "$J" window "window 3 ft" --width "3 ft" --height "5 ft" "${A[@]}")")"
  DT="$(made "$("$P" type "$J" door "door 3 ft" --width "3 ft" --height "6 ft 8 in" "${A[@]}")")"
  read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${A[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
  WIN="$(made "$("$P" window "$J" "$SOUTH" "$NT" --offset "18 ft" --sill "2.5 ft" "${A[@]}")")"
  DOOR="$(made "$("$P" door "$J" "$SOUTH" "$DT" --offset "6 ft" --sill "0 ft" "${A[@]}")")"
  R="$(made "$("$P" roof "$J" "Main roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 6:12 --eave-height "9 ft" "${A[@]}")")"
  for e in $edges; do
    i=$((i + 1))
    [ "$e" = G ] && "$P" set "$J" "$R" "edge:$i" gable "${A[@]}" >/dev/null
  done
  "$P" import assemblies "$J" starter >/dev/null
  local all; all="$("$P" show "$J")"
  for e in $(grep -oE 'eave-[0-9]+' <<<"$all" | sort -u); do "$P" apply "$J" "$e" "Gutter" --source "$L" >/dev/null; done
  for e in $(grep -oE 'rake-[0-9]+' <<<"$all" | sort -u); do "$P" apply "$J" "$e" "Rake trim" --source "$L" >/dev/null; done
  for e in $SOUTH $EAST $NORTH $WEST; do "$P" apply "$J" "$e" "Exterior wall, 2x6, lap siding" --source "$L" >/dev/null; done
  "$P" apply "$J" "$WIN" "Window" --source "$L" >/dev/null
  "$P" apply "$J" "$DOOR" "Entry door" --source "$L" >/dev/null
  return 0
}
echo "== The house of demos/trimw.sh with a gable roof, a shed roof and a hip roof, made at the command line =="
JG="$T/gable.json"; JS="$T/shed.json"; JH="$T/hip.json"
house "$JG" "P G P G"
house "$JS" "P G G G"
house "$JH" "P P P P"
G="$("$P" gable "$JG")"
says "the west wall's gable" "[$WEST]: gable ~40 SF; top (0, 9.5) (8, 13.5) (16, 9.5) ft" "$G"
# Call 197, point 7: "none" is marked ~ where it is Assumed (every house here is traced). The two checks of "none"
# below moved from "none: ..." to "~ none: ..." by that decision, and by nothing else.
says "the south wall, under an eave" "[$SOUTH]: ~ none: an eave is over it" "$G"
GS="$("$P" gable "$JS")"
says "the shed's north wall" "[$NORTH]: gable ~204 SF; top (0, 17.5) (24, 17.5) ft" "$GS"
GH="$("$P" gable "$JH")"
for w in $SOUTH $EAST $NORTH $WEST; do says "the hip's wall $w" "[$w]: ~ none: an eave is over it" "$GH"; done
"$P" export estimate "$JG" "$T/est.csv" >/dev/null
python3 - "$T/est.csv" <<'PY'
import csv, sys
# The walls are named in the order they were drawn: south, east, north, west.
rows = [r for r in csv.DictReader(open(sys.argv[1])) if r["Line"] == "Lap siding" and r["Element"] == "wall 4"]
assert rows, "no lap siding line on the west wall"
q = sum(float(r["Quantity"]) for r in rows if r["Quantity"] != "")
print("    lap siding on the west wall: by hand 144 + 40 = 184.00 SF, the estimate says %.2f" % q)
if "%.2f" % q != "184.00":
    print("FAIL: the west wall's lap siding is not 184.00 SF"); sys.exit(1)
PY
echo "$WEST $SOUTH $NORTH $EAST" > "$T/walls.txt"

# One session: open <job>, take the views named, write a map beside each.
# The keys that turn the view from the street corner to the street at the west end: the Left arrow held twice.
cat > "$T/lib.sh" <<'LIB'
st() { awk -F'\t' -v n="$1" '$1=="state" && $2==n {print $3}' "$PLUMBLINE_MAP"; }
gable_at() { awk -F'\t' -v w="$1" '$1=="gable" && $2==w {print $3, $4}' "$PLUMBLINE_MAP"; }
panel_said() { awk -F'\t' -v w="$1" '$1=="gable_panel" && $2==w {print $3 " " $4}' "$PLUMBLINE_MAP"; }
panels_away() { on "Show panel Job"; on "Show panel Details"; on "Show panel Tools"; }
turn_left() { focus; xdotool keydown Left; sleep 1.2; xdotool keyup Left; sleep 8; }
# Press a control where it settles (the foot of the window shifts the panel switches when the pointer is over the model).
press() {
  local at again n=0
  at="$(found model "$1")" || exit 1
  while [ "$n" -lt 5 ]; do
    xdotool mousemove $at; sleep 3
    again="$(where model "$1")"
    [ "$again" = "$at" ] && break
    [ -n "$again" ] && at="$again"
    n=$((n + 1))
  done
  click $at
}
finishes_switch() {
  local n=0
  press "Show panel Job"; sleep 6
  while [ -z "$(where model 'Show Finishes')" ] && [ -z "$(away 'Show Finishes')" ] && [ "$n" -lt 20 ]; do sleep 3; n=$((n + 1)); done
  on "Show Finishes"; sleep 3
  press "Show panel Job"; sleep 5; park; sleep 4
}
LIB

echo
echo "== The gable roof: the street corner, the street at the west end; the west wall picked in its gable; Finishes off; the south wall =="
cat > "$T/session-g.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$T/pg" ICD="$ICD" LOG="$T/app-g.log"
. "$PWD/demos/window.sh"
. "$T/lib.sh"
fail() { echo "\$*" >> "\$T/fails.txt"; }
read -r WEST SOUTH NORTH EAST < "$T/walls.txt"
mkdir -p "\$OUT"
open_window "$JG"
panels_away; park; sleep 6
shot                                                      # 01 the street corner
cp "\$PLUMBLINE_MAP" "\$T/map-01.txt"
[ -n "\$(gable_at \$WEST)" ] || fail "the map has no gable for the west wall"
[ -z "\$(gable_at \$SOUTH)" ] || fail "the map has a gable for the south wall"
turn_left; turn_left; turn_left; park; sleep 4
shot                                                      # 02 the street at the west end
cp "\$PLUMBLINE_MAP" "\$T/map-02.txt"
read -r gx gy <<<"\$(gable_at \$WEST)"
echo "pick the west wall in its gable at \$gx \$gy" >> "\$T/log.txt"
click \$gx \$gy; park
sleep 4
on "Show panel Details"; park; sleep 6
shot                                                      # 03 the west wall picked, its panel open
on "Show panel Details"; park; sleep 4
cp "\$PLUMBLINE_MAP" "\$T/map-03.txt"
echo "the west wall's panel says: \$(panel_said \$WEST)" >> "\$T/log.txt"
finishes_switch; [ "\$(st Finishes)" = off ] || fail "Finishes does not read off"
shot                                                      # 04 Finishes off
finishes_switch; [ "\$(st Finishes)" = on ] || fail "Finishes does not read on again"
close_window
SESSION
chmod +x "$T/session-g.sh"; touch "$T/start"
timeout 2400 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session-g.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app-g.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app-g.log" | head; exit 1; fi
for n in 01 02 03 04; do [ "$T/pg/$n-model.png" -nt "$T/start" ] || { echo "FAIL: no picture $n from this run"; exit 1; }; done
[ ! -s "$T/fails.txt" ] || { echo "FAIL: in the session:"; cat "$T/fails.txt"; exit 1; }
sed 's/^/    /' "$T/log.txt"
cp "$T/pg/01-model.png" "$OUT/01-home.png"; cp "$T/pg/02-model.png" "$OUT/02-west.png"
cp "$T/pg/03-model.png" "$OUT/03-west-selected.png"; cp "$T/pg/04-model.png" "$OUT/04-west-bare.png"
read -r WEST SOUTH NORTH EAST < "$T/walls.txt"
[ "$(awk -F'\t' -v w="$WEST" '$1=="gable_panel" && $2==w {print $3 " " $4}' "$T/map-03.txt")" = "40.00 SF Assumed" ] || { echo "FAIL: the west wall's panel does not say Gable 40.00 SF, Assumed: $(grep gable_panel "$T/map-03.txt" || echo 'no line')"; exit 1; }
echo "    the west wall's panel: Gable 40.00 SF, Assumed"

echo
echo "== The south wall, under an eave: picked on its siding, its panel has no Gable =="
cat > "$T/session-u.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$T/pu" ICD="$ICD" LOG="$T/app-u.log"
. "$PWD/demos/window.sh"
. "$T/lib.sh"
mkdir -p "\$OUT"
open_window "$JG"
panels_away; park; sleep 6
# On the siding between the door and the window, from the street corner.
click 900 620; park; sleep 4
on "Show panel Details"; park; sleep 6
shot                                                      # the south wall picked, its panel open
cp "\$PLUMBLINE_MAP" "\$T/map-u.txt"; tail -1 "\$PLUMBLINE_MAP.said" > "\$T/said-u.txt"
close_window
SESSION
chmod +x "$T/session-u.sh"; touch "$T/startu"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session-u.sh" || true
[ "$T/pu/01-model.png" -nt "$T/startu" ] || { echo "FAIL: no picture of the south wall picked"; exit 1; }
cp "$T/pu/01-model.png" "$OUT/05-south.png"
grep -qF "Selected wall 1" "$T/said-u.txt" || { echo "FAIL: the south wall was not picked: $(cat "$T/said-u.txt")"; exit 1; }
[ -z "$(grep -P '^gable_panel\t' "$T/map-u.txt" || true)" ] || { echo "FAIL: a panel says Gable with the south wall picked: $(grep gable_panel "$T/map-u.txt")"; exit 1; }
echo "    the south wall is picked and its panel has no Gable"

echo
echo "== The shed roof: the street corner, and the north wall, under the high edge, picked in its gable =="
cat > "$T/session-s.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$T/ps" ICD="$ICD" LOG="$T/app-s.log"
. "$PWD/demos/window.sh"
. "$T/lib.sh"
fail() { echo "\$*" >> "\$T/fails.txt"; }
read -r WEST SOUTH NORTH EAST < "$T/walls.txt"
mkdir -p "\$OUT"
open_window "$JS"
panels_away; park; sleep 6
shot                                                      # 01 the street corner
# From the north: the view turned half way round, by the Left arrow held six times.
for i in 1 2 3 4 5 6; do turn_left; done
park; sleep 4
cp "\$PLUMBLINE_MAP" "\$T/map-s2.txt"
read -r gx gy <<<"\$(awk -F'\t' -v w="\$NORTH" '\$1=="gable" && \$2==w {print \$3, \$4}' "\$T/map-s2.txt")"
shot                                                      # 02 from the north
echo "pick the north wall in its gable at \$gx \$gy" >> "\$T/log-s.txt"
click \$gx \$gy; park; sleep 4
on "Show panel Details"; park; sleep 6
shot                                                      # 03 the north wall picked, its panel open
cp "\$PLUMBLINE_MAP" "\$T/map-s3.txt"
close_window
SESSION
chmod +x "$T/session-s.sh"; touch "$T/starts"
timeout 1800 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session-s.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app-s.log"; then echo "FAIL: the program panicked"; exit 1; fi
for n in 01 02 03; do [ "$T/ps/$n-model.png" -nt "$T/starts" ] || { echo "FAIL: no shed picture $n from this run"; exit 1; }; done
[ ! -s "$T/fails.txt" ] || { echo "FAIL: in the session:"; cat "$T/fails.txt"; exit 1; }
cp "$T/ps/01-model.png" "$OUT/06-shed-home.png"; cp "$T/ps/02-model.png" "$OUT/07a-shed-north.png"; cp "$T/ps/03-model.png" "$OUT/07-shed-north-selected.png"
[ "$(awk -F'\t' -v w="$NORTH" '$1=="gable_panel" && $2==w {print $3 " " $4}' "$T/map-s3.txt")" = "204.00 SF Assumed" ] || { echo "FAIL: the north wall's panel does not say Gable 204.00 SF, Assumed: $(grep gable_panel "$T/map-s3.txt" || echo 'no line')"; exit 1; }
echo "    the north wall's panel: Gable 204.00 SF, Assumed"

echo
echo "== The hip roof: the street corner, as the window opens =="
view_only() {  # view_only <program> <job> <folder> <keys: 1 to take the street view too>
cat > "$T/session-h.sh" <<SESSION
#!/bin/bash
export APP="$1" T="$T" OUT="$3" ICD="$ICD" LOG="$T/app-h.log"
. "$PWD/demos/window.sh"
. "$T/lib.sh"
mkdir -p "\$OUT"
open_window "$2"
panels_away; park; sleep 6
shot
$([ "${4:-0}" = 1 ] && echo 'turn_left; turn_left; turn_left; park; sleep 4; shot')
close_window
SESSION
chmod +x "$T/session-h.sh"; touch "$T/starth"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session-h.sh" || true
[ "$3/01-model.png" -nt "$T/starth" ] || { echo "FAIL: no picture from the view of $2"; exit 1; }
if grep -q "panicked" "$T/app-h.log"; then echo "FAIL: the program panicked"; exit 1; fi
}
view_only "$APP" "$JH" "$T/ph" 0
cp "$T/ph/01-model.png" "$OUT/08-hip-home.png"
if [ -n "$BEFORE_APP" ]; then
  echo
  echo "== The program before this work, the same views =="
  view_only "$BEFORE_APP" "$JH" "$T/bh" 0
  cp "$T/bh/01-model.png" "$OUT/before-08.png"
  view_only "$BEFORE_APP" "$JG" "$T/bg" 1
  cp "$T/bg/01-model.png" "$OUT/before-01.png"; cp "$T/bg/02-model.png" "$OUT/before-02.png"
fi

echo
echo "== The pictures, measured =="
python3 - "$OUT" "$T" "${BEFORE_APP:+before}" "$WEST" <<'PY'
import sys
import numpy as np
from PIL import Image
out, T, before, west = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
def load(n): return np.asarray(Image.open(f"{out}/{n}.png").convert("RGB")).astype(float)
def gable(mapfile, wall):
    for line in open(mapfile):
        f = line.rstrip("\n").split("\t")
        if f[0] == "gable" and f[1] == wall: return [float(x) for x in f[2:]]
ok = True
def check(what, good, said):
    global ok
    ok = ok and good
    print(("  ok   " if good else "  FAIL ") + what + ": " + said)
def patch(img, x, y, r=5):
    return img[int(y) - r:int(y) + r + 1, int(x) - r:int(x) + r + 1].reshape(-1, 3)
# The map lines are in the window's own pixels, as the pictures are.
for pic, mapfile in (("01-home", "map-01.txt"), ("02-west", "map-02.txt")):
    g = gable(f"{T}/{mapfile}", west)
    img = load(pic)
    # the west wall's own face halfway up, where the siding is; the gable's middle, halfway between the wall's top and the roof
    wall, gab = patch(img, g[6], g[7]).mean(axis=0), patch(img, g[0], g[1]).mean(axis=0)
    d = float(np.abs(wall - gab).max())
    check(f"{pic}: the gable's middle is the siding's colour", d <= 22, f"the wall's {wall.round()} against the gable's {gab.round()}, the greatest difference {d:.0f} of 255 (22 allowed)")
    if before:
        b = load("before-01" if pic == "01-home" else "before-02")
        was = patch(b, g[0], g[1]).mean(axis=0)
        e = float(np.abs(wall - was).max())
        # On the program before the place is the sky, the ground or the far side of the roof: another colour, or a flat colour with
        # none of the siding's courses; here it has the courses.
        flat = float(patch(b, g[0], g[1], 8).mean(axis=1).std()) < 1.0 and float(patch(b, g[0], g[1], 8).std()) < 1.0
        courses = float(patch(img, g[0], g[1], 8).reshape(17, 17, 3).mean(axis=(1, 2)).std()) > 2.0
        check(f"{pic}: on the program before the same place is not the siding", e > 22 or flat, f"it read {was.round()} there, {e:.0f} from the siding{', a flat colour with no courses' if flat else ''}")
        check(f"{pic}: here the place has the siding's courses", courses, "the lap courses show across it (row to row variation over 2 levels)" if courses else "no courses show across it")
if before:
    a, b = load("08-hip-home"), load("before-08")
    BAR = 915  # the foot of the window says what was just done
    # The software renderer's own noise: two runs of the same program differ by a thousand dots of more than 6 levels and a few
    # dozen of more than 24 (measured); the picture of this program against the program before must be no further from it.
    d = np.abs(a[:BAR] - b[:BAR]).max(axis=2)
    few, big = int((d > 12).sum()), int(d.max())
    check("08-hip-home: the hip-roofed house is the picture of the program before", few <= 400 and big <= 48, f"{int((d > 6).sum())} dots differ by more than 6 levels, {few} by more than 12, the greatest difference {big} levels, of {a[:BAR].shape[0] * a.shape[1]} dots (the renderer alone moves a thousand by 6 and a few dozen by 24, the greatest 48 at most)")
else:
    print("  (no GABLE_BEFORE_BIN: the views on the program before were not made, and the hip house was not compared)")
sys.exit(0 if ok else 1)
PY
echo
echo "gableendw: every step passed; pictures in $OUT"
