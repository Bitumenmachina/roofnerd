#!/usr/bin/env bash
# The outside trim drawn, each piece casting its shadow (call 184, step 1, the window's half). The demonstration is
# written in STATUS.md, "The outside trim drawn: a demonstration, written before it is built", with its figures worked by hand.
# The house of demos/trim.sh (24 by 16 ft, 9 ft walls, a gable roof 1 ft out all round, a window and a door in the south wall)
# is made at the command line with its trim lines given, and opened in the window. Every figure is the model's: the window
# draws the pieces `plumbline trim --pieces` lists, and works nothing out. Where a piece lies on screen is read from the
# program's own map (`trim<TAB>kind<TAB>thing<TAB>x1<TAB>y1<TAB>x2<TAB>y2<TAB>width`), and each picture is measured against it.
#
# Run from the repository root:  [PLUMBLINE_BIN=<folder>] demos/trimw.sh [folder for the pictures]
# Pictures (the model window; the sheet beside each; the program numbers a picture by the order it was taken in):
#   01 the Show list with its Trim switch and the count           02 the house from the street corner, with trim
#   03 the same view, Trim off                                    04 the eave and the nearest corner close, with trim
#   05 the same, Trim off                                         06 the window with its casing, with trim
#   07 the same, Trim off                                         08 the sun at noon in June, the eave and corner close, with trim
#   09 the same, Trim off                                         10 the street view with no trim line priced (as before)
#
# Call 193, point A (STATUS.md, "The window judged (2026-11-24), and its findings fixed"): the house is dressed in what the
# estimate prices, trim too. Three houses, each 24 by 16 ft with lap siding, a window and a door, in three more programs:
#  - (1) no trim line priced: no corner board and no outside casing is drawn, the window and the door are their units alone,
#    the roof has its own edge, and the Show list has no Trim switch (pictures 11);
#  - (2) the window's "Casing and trim at the window" priced by the each (the starter's line; its "Exterior casing" taken off): the
#    window is cased with the window's own standard casing, the door (no such line) is not (12);
#  - (3) casing and corner boards priced by their lengths: the model's 8 + 7 = 15 pieces, and the switch reads 15 (13).
# Measured: each picture against the first, round the pieces (the places are the third house's map).
#
# Worked by hand (STATUS.md):
#  - Opened: the switch "Trim" reads 27 (2 fascia, 4 rake fascia, 2 soffits, 4 rake soffits, 8 corner boards, 7 pieces of casing:
#    the window's four, the door's three), on. The window's map lists 27 pieces.
#  - From the street corner the picture differs from the same view with Trim off at the eaves, the corners and round the window
#    and the door: of the dots that differ, nine in ten lie within a hand's breadth of a piece (its strip, and where it casts
#    its shadow on the wall just under it); the wall's face between a corner board and the casing is the same dot for dot.
#  - A piece casts a shadow: the sun at noon in June, the south wall just under the soffit is darker than the same wall 3 ft
#    lower and darker than it is there with Trim off.
#  - Only what is priced: the Trim switch off is the house as it was drawn before: the same picture, dot for dot, as the job with
#    no trim line priced (the siding, the gutter and the roof as they were). A fascia line taken off the north eave and the job
#    opened again: the switch reads 26, and the north eave has no fascia where the south has.
#  - A click on a corner board selects its wall; on the window's casing, the window; on a fascia, the eave.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/trimw.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL, numpy" 2>/dev/null || { echo "demos/trimw.sh needs python's PIL and numpy (the pictures are measured)"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/trimw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/trimw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap '[ -z "${TRIMW_KEEP:-}" ] || cp -r "$T" "$TRIMW_KEEP"; rm -rf "$T"' EXIT
A=(--status Assumed --source "made-up tracing")
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }

echo "== The house of demos/trim.sh, made at the command line, its trim lines given =="
J="$T/job.json"
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
read -r EAVE_S EAVE_N <<<"$EAVES"
[ "$(wc -w <<<"$EAVES")" = 2 ] && [ "$(wc -w <<<"$RAKES")" = 4 ] || { echo "FAIL: expected 2 eaves and 4 rakes: $EAVES / $RAKES"; exit 1; }
"$P" import assemblies "$J" starter >/dev/null
L="the starter library"
for e in $EAVES; do "$P" apply "$J" "$e" "Gutter" --source "$L" >/dev/null; done
for r in $RAKES; do "$P" apply "$J" "$r" "Rake trim" --source "$L" >/dev/null; done
for w in $SOUTH $EAST $NORTH $WEST; do "$P" apply "$J" "$w" "Exterior wall, 2x6, lap siding" --source "$L" >/dev/null; done
"$P" apply "$J" "$WIN" "Window" --source "$L" >/dev/null
"$P" apply "$J" "$DOOR" "Entry door" --source "$L" >/dev/null
"$P" trim "$J" --pieces > "$T/pieces.txt"
grep -E "^ *(fascia|rake fascia|soffit|rake soffit|corner boards|casing)" "$T/pieces.txt" | head -12 | sed 's/^/    /' || true

# The same house with no trim line priced: every trim line taken off (the siding, the gutter, the window and the door stay).
J0="$T/job-no-trim.json"; cp "$J" "$J0"
for e in $EAVES; do "$P" remove-line "$J0" "$e" "Fascia" >/dev/null; "$P" remove-line "$J0" "$e" "Soffit" >/dev/null; done
for r in $RAKES; do "$P" remove-line "$J0" "$r" "Rake fascia" >/dev/null; "$P" remove-line "$J0" "$r" "Rake soffit" >/dev/null; done
for w in $SOUTH $EAST $NORTH $WEST; do "$P" remove-line "$J0" "$w" "Corner boards" >/dev/null; done
for o in $WIN $DOOR; do "$P" remove-line "$J0" "$o" "Exterior casing" >/dev/null; done
[ -z "$("$P" trim "$J0" --pieces | grep -E '^ *(fascia|soffit|corner boards|casing)' || true)" ] || echo "    (note: pieces remain in the job with its trim lines taken off)"
# And with the north eave's fascia line taken off alone.
J1="$T/job-north-bare.json"; cp "$J" "$J1"
"$P" remove-line "$J1" "$EAVE_N" "Fascia" >/dev/null

echo
echo "== The session: the Show list; the house from the street corner, with trim and without; the eave and corner; the window; a click on each; the sun at noon in June =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
st() { awk -F'\t' -v n="\$1" '\$1=="state" && \$2==n {print \$3}' "\$PLUMBLINE_MAP"; }
said_last() { tail -1 "\$PLUMBLINE_MAP.said"; }
fail() { echo "\$*" >> "\$T/fails.txt"; }
# The Trim switch is in the left panel: put it away for a picture, bring it back to press the switch, put it away again.
# Close on the house the pointer parked over it makes the foot of the window show where it is ("3D E 6' 8 3/4" ...), which
# shifts the three panel switches left; they move back when the pointer leaves the model for one of them. So go to the
# control, let the layout settle, and press it where it then is.
press() {
  local at again n=0
  at="\$(found model "\$1")" || exit 1
  while [ "\$n" -lt 5 ]; do
    xdotool mousemove \$at; drawn_for 2 3
    again="\$(where model "\$1")"
    [ "\$again" = "\$at" ] && break
    [ -n "\$again" ] && at="\$again"
    n=\$((n + 1))
  done
  click \$at
}
trim_switch() {
  local n=0
  press "Show panel Job"; sleep 6
  while [ -z "\$(where model 'Show Trim')" ] && [ -z "\$(away 'Show Trim')" ] && [ "\$n" -lt 20 ]; do sleep 3; n=\$((n + 1)); done
  on "Show Trim"; sleep 3
  press "Show panel Job"; sleep 5; park; sleep 4
}
# Where a piece lies on screen, from the map: kind, thing.
seg() { awk -F'\t' -v k="\$1" -v o="\$2" '\$1=="trim" && \$2==k && \$3==o {print \$4, \$5, \$6, \$7}' "\$PLUMBLINE_MAP"; }
mid() { awk '{printf "%d %d", (\$1 + \$3) / 2, (\$2 + \$4) / 2}'; }
wheel() { local i; for i in \$(seq 1 "\$2"); do xdotool click "\$1"; drawn_for 2 2; done; sleep 6; }
open_window "$J"
[ "\$(awk -F'\t' '\$1=="count" && \$2=="Trim" {print \$3}' "\$PLUMBLINE_MAP")" = 27 ] || fail "the Trim switch does not read 27: \$(grep -P '^count' "\$PLUMBLINE_MAP" || echo none)"
[ "\$(st Trim)" = on ] || fail "Trim does not read on as the job opens"
[ "\$(grep -c -P '^trim\t' "\$PLUMBLINE_MAP")" = 27 ] || fail "the map lists \$(grep -c -P '^trim\t' "\$PLUMBLINE_MAP") pieces, not 27"
# 00 the Show list, scrolled to its Trim switch
n=0
while [ -z "\$(where model 'Show Trim')" ] && [ "\$n" -lt 30 ]; do xdotool mousemove 120 600; drawn_for 2 0.5; xdotool click 5; drawn_for 2 2; n=\$((n + 1)); done
[ -n "\$(where model 'Show Trim')" ] || fail "the Show list has no Trim switch"
shot                                    # 01 the Show list, its Trim switch and the count
on "Show panel Job"; on "Show panel Details"; on "Show panel Tools"
park; sleep 6
shot                                    # 02 from the street corner, with trim
cp "\$PLUMBLINE_MAP" "\$T/map-01.txt"   # the map of the street view
trim_switch; [ "\$(st Trim)" = off ] || fail "Trim does not read off after the switch"
shot                                    # 03 the same, Trim off
trim_switch; [ "\$(st Trim)" = on ] || fail "Trim does not read on after the switch again"
# 03: the eave and the nearest corner, close: the foot of the nearest corner board lowest on screen, its top where the eave meets it
read -r cx cy <<<"\$(awk -F'\t' '\$1=="trim" && \$2=="corner boards" {y=(\$5>\$7)?\$5:\$7; if (y>best) {best=y; top=(\$5<\$7)?\$5:\$7; x=(\$5<\$7)?\$4:\$6}} END {printf "%d %d", x, top}' "\$T/map-01.txt")"
echo "close on the corner: \$cx \$cy" >> "\$T/log.txt"
xdotool mousemove "\$cx" "\$cy"; drawn_for 2 2; wheel 4 4
park
shot                                    # 04 the eave and corner close, with trim
trim_switch; shot                       # 05 the same, Trim off
trim_switch
xdotool key f; drawn_for 3 10
# 05: the window and its casing
read -r wx wy <<<"\$(awk -F'\t' -v o="$WIN" '\$1=="trim" && \$2=="casing" && \$3==o {sx+=\$4+\$6; sy+=\$5+\$7; n+=2} END {printf "%d %d", sx/n, sy/n}' "\$T/map-01.txt")"
echo "close on the window: \$wx \$wy" >> "\$T/log.txt"
xdotool mousemove "\$wx" "\$wy"; drawn_for 2 2; wheel 4 5
park
shot                                    # 06 the window with its casing
trim_switch; shot                       # 07 the same, Trim off
trim_switch
xdotool key f; drawn_for 3 10
# Clicks, from the street corner: each on the middle of a piece, the model's place for it on the map.
cp "\$PLUMBLINE_MAP" "\$T/map-home.txt"
click_on() { local at; at="\$(awk -F'\t' -v k="\$1" -v o="\$2" -v w="\$3" '\$1=="trim" && \$2==k && \$3==o {dx=\$6-\$4; dy=\$7-\$5; if (w=="tall" && (dy<0?-dy:dy) < (dx<0?-dx:dx)) next; printf "%d %d\n", (\$4+\$6)/2, (\$5+\$7)/2}' "\$T/map-home.txt" | head -1)"; echo "click on \$1 of \$2 at \$at" >> "\$T/log.txt"; click \$at; park; }
# the corner board nearest the eye (lowest foot on screen) and the wall it is on
read -r cwall <<<"\$(awk -F'\t' '\$1=="trim" && \$2=="corner boards" {y=(\$5>\$7)?\$5:\$7; if (y>best) {best=y; o=\$3}} END {print o}' "\$T/map-home.txt")"
click_on "corner boards" "\$cwall" tall
said_last > "\$T/said-corner.txt"; echo "corner board of \$cwall: \$(cat "\$T/said-corner.txt")" >> "\$T/log.txt"
click_on casing "$WIN" tall
said_last > "\$T/said-casing.txt"; echo "casing of $WIN: \$(cat "\$T/said-casing.txt")" >> "\$T/log.txt"
click_on fascia "$EAVE_S" wide
said_last > "\$T/said-fascia.txt"; echo "fascia of $EAVE_S: \$(cat "\$T/said-fascia.txt")" >> "\$T/log.txt"
esc; xdotool mousemove 1000 880; drawn_for 2 1
# 08: the sun at noon in June (the card: Later twice from ten, Month after twice from April), then closed; the eave and corner close again
on "Sun"; on "Later"; on "Later"; on "Month after"; on "Month after"; on "Sun"
xdotool key f; drawn_for 3 10
xdotool mousemove "\$cx" "\$cy"; drawn_for 2 2; wheel 4 4
park; sleep 6
shot                                    # 08 noon in June, the eave and corner close, with trim
cp "\$PLUMBLINE_MAP" "\$T/map-08.txt"
trim_switch; shot                       # 09 the same, Trim off
trim_switch
close_window
SESSION
chmod +x "$T/session.sh"
touch "$T/start"
timeout 2400 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 07 08 09; do [ "$OUT/$n-model.png" -nt "$T/start" ] || { echo "FAIL: no picture $n from this run"; exit 1; }; done
[ ! -s "$T/fails.txt" ] || { echo "FAIL: in the session:"; cat "$T/fails.txt"; exit 1; }
sed 's/^/    /' "$T/log.txt"

echo
echo "== The same house with no trim line priced (as it was drawn before), and with the north eave's fascia line taken off =="
OUT0="$T/p0"; mkdir -p "$OUT0"
cat > "$T/session0.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT0" ICD="$ICD" LOG="$T/app0.log"
. "$PWD/demos/window.sh"
open_window "$J0"
[ -z "\$(grep -P '^count\tTrim' "\$PLUMBLINE_MAP")" ] || echo "a job with no trim line priced reads a Trim count" >> "\$T/fails0.txt"
[ -z "\$(where model 'Show Trim')" ] || echo "a job with no trim line priced offers a Trim switch" >> "\$T/fails0.txt"
on "Show panel Job"; on "Show panel Details"; on "Show panel Tools"
park; sleep 6
shot                                    # the street view, no trim priced (picture 10 of this demonstration)
close_window
SESSION
chmod +x "$T/session0.sh"
touch "$T/start0"
timeout 900 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session0.sh" || true
cat > "$T/session1.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT0" ICD="$ICD" LOG="$T/app1.log"
. "$PWD/demos/window.sh"
open_window "$J1"
cp "\$PLUMBLINE_MAP" "\$T/map-north.txt"
close_window
SESSION
chmod +x "$T/session1.sh"
timeout 600 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session1.sh" || true
[ ! -s "$T/fails0.txt" ] || { echo "FAIL:"; cat "$T/fails0.txt"; exit 1; }
cp "$OUT0/01-model.png" "$OUT/10-model.png" 2>/dev/null || { echo "FAIL: no picture 10"; ls "$OUT0"; exit 1; }
cp "$OUT0/01-sheet.png" "$OUT/10-sheet.png" 2>/dev/null || true
[ "$(awk -F'\t' '$1=="count" && $2=="Trim" {print $3}' "$T/map-north.txt")" = 26 ] || { echo "FAIL: the switch does not read 26 with the north eave's fascia line taken off"; grep -P '^count' "$T/map-north.txt" || echo "no count line"; exit 1; }
echo "    the switch reads 26 with the north eave's fascia line taken off"
S_N="$(awk -F'\t' -v o="$EAVE_S" '$1=="trim" && $2=="fascia" && $3==o' "$T/map-north.txt" | wc -l)"
N_N="$(awk -F'\t' -v o="$EAVE_N" '$1=="trim" && $2=="fascia" && $3==o' "$T/map-north.txt" | wc -l)"
echo "    fascia on the south eave: $S_N; on the north eave: $N_N"
[ "$S_N" = 1 ] && [ "$N_N" = 0 ] || { echo "FAIL: the south eave should have its fascia and the north none"; exit 1; }

echo
echo "== The pictures, measured =="
python3 - "$OUT" "$T" "$SOUTH" <<'PY'
import sys, math
import numpy as np
from PIL import Image
out, T, south = sys.argv[1], sys.argv[2], sys.argv[3]
def load(n): return np.asarray(Image.open(f"{out}/{n}-model.png").convert("RGB")).astype(int)
def lum(a): return 0.2126 * a[..., 0] + 0.7152 * a[..., 1] + 0.0722 * a[..., 2]
def pieces(mapfile):
    rows = []
    for line in open(mapfile):
        f = line.rstrip("\n").split("\t")
        if f[0] == "trim": rows.append((f[1], f[2], *map(float, f[3:8])))
    return rows
ok = True
def check(what, good, said):
    global ok
    ok = ok and good
    print(("  ok   " if good else "  FAIL ") + what + ": " + said)
# 1. the picture with trim differs from the one without at the eaves, the corners and the openings, and nowhere else
a, b = load("02"), load("03")
BAR = 915  # the status line at the foot says what was just done ("Trim: switched off."): not the house
a, b = a[:BAR], b[:BAR]
diff = np.abs(a - b).max(axis=2) > 24
ys, xs = np.nonzero(diff)
check("02 and 03 differ", diff.sum() > 200, f"{diff.sum()} dots differ of {diff.size}")
segs = pieces(f"{T}/map-01.txt")
def near(x, y, margin):
    best = 1e9
    for kind, of, x1, y1, x2, y2, w in segs:
        dx, dy = x2 - x1, y2 - y1
        L2 = dx * dx + dy * dy
        t = 0.0 if L2 == 0 else max(0.0, min(1.0, ((x - x1) * dx + (y - y1) * dy) / L2))
        d = math.hypot(x - (x1 + t * dx), y - (y1 + t * dy)) - w / 2
        best = min(best, d)
    return best
for margin in (12, 40):
    within = sum(1 for x, y in zip(xs, ys) if near(x, y, 0) <= margin)
    share = within / max(1, len(xs))
    check(f"dots that differ lie near a piece (within {margin} px of its strip)", share >= 0.9 if margin == 40 else True, f"{share * 100:.1f} % of {len(xs)}")
# 2. each kind of piece shows: the picture differs along it (a strip round the middle of one piece of each kind)
kinds = {}
for kind, of, x1, y1, x2, y2, w in segs: kinds.setdefault(kind, []).append((x1, y1, x2, y2, w))
for kind, ss in sorted(kinds.items()):
    n = 0
    for x1, y1, x2, y2, w in ss:
        for t in np.linspace(0.1, 0.9, 9):
            x, y = int(x1 + t * (x2 - x1)), int(y1 + t * (y2 - y1))
            if 0 <= y < diff.shape[0] and 0 <= x < diff.shape[1] and diff[max(0, y - 6):y + 7, max(0, x - 6):x + 7].any(): n += 1
    total = len(ss) * 9
    check(f"{kind}: the picture differs along its {len(ss)} piece(s)", n > 0, f"{n} of {total} places along them")
# 3. the wall between the corner and the casing is the same dot for dot: a patch of wall's face midway between the door and the window
cb = [s for s in segs if s[0] == "corner boards" and s[1] == south]
if len(cb) == 2:
    tops = [(s[2], s[3]) if s[3] < s[5] else (s[4], s[5]) for s in cb]
    feet = [(s[2], s[3]) if s[3] >= s[5] else (s[4], s[5]) for s in cb]
    def at(f, k):  # a place on the south wall's face: f along it from the west board's top to the east board's, k down from its top
        x = [tops[i][0] * (1 - f) + 0 for i in range(2)]
        return None
    T0, T1 = np.array(tops[0]), np.array(tops[1]); F0, F1 = np.array(feet[0]), np.array(feet[1])
    if T0[0] > T1[0]: T0, T1, F0, F1 = T1, T0, F1, F0
    def spot(f, k): top = T0 * (1 - f) + T1 * f; foot = F0 * (1 - f) + F1 * f; return top + (foot - top) * k
    p = spot(0.5, 0.55).astype(int)  # midway between the door and the window, lower on the wall, clear of the eave's shade
    patch = (slice(p[1] - 8, p[1] + 9), slice(p[0] - 8, p[0] + 9))
    d = np.abs(a[patch] - b[patch]).max()
    check("the wall's face away from every piece is the same dot for dot", d <= 24, f"largest difference in a 17 px patch at {tuple(p)}: {d} of 255")
    # 4. the shadow: the sun at noon in June, close on the eave and the nearest corner. Down the corner board that stands on
    # the south wall (it takes the sun), from its top to its foot.
    c, e = load("08")[:BAR], load("09")[:BAR]
    segs8 = pieces(f"{T}/map-08.txt")
    cb8 = [s for s in segs8 if s[0] == "corner boards" and s[1] == south]
    nearest = max(cb8, key=lambda s: max(s[3], s[5]))
    top, foot = ((nearest[2], nearest[3]), (nearest[4], nearest[5])) if nearest[3] < nearest[5] else ((nearest[4], nearest[5]), (nearest[2], nearest[3]))
    top, foot = np.array(top), np.array(foot)
    def down(img, u):
        q = (top + (foot - top) * u).astype(int)
        if not (3 <= q[1] < img.shape[0] - 3 and 3 <= q[0] < img.shape[1] - 3): return None
        return lum(img)[q[1] - 2:q[1] + 3, q[0] - 2:q[0] + 3].mean()
    rows = []
    for u in (0.03, 0.08, 0.14, 0.22, 0.30, 0.40, 0.55):
        on_, off_ = down(c, u), down(e, u)
        if on_ is None or off_ is None: continue
        rows.append((u, on_, off_))
        print(f"    the south corner board, {u * 9:.1f} ft below its top: luminance {on_:.1f} with trim, {off_:.1f} without")
    check("the picture reaches the south corner board", len(rows) >= 5, f"{len(rows)} places on it are on screen")
    if len(rows) >= 5:
        shaded = min(r[1] for r in rows[:3]); lit = max(r[1] for r in rows[3:])
        check("with trim, the corner board just under the eave is darker than it is lower down", shaded + 5 < lit, f"{shaded:.1f} under the eave against {lit:.1f} lower")
        # How far down the shade runs: the last place still darker than the lit value by half the step.
        def reach(col):
            lit_ = max(r[col] for r in rows[3:]); dark_ = min(r[col] for r in rows[:3])
            mid = (lit_ + dark_) / 2
            run = 0.0
            for r in rows:
                if r[col] < mid: run = r[0] * 9
            return run
        check("the shade reaches as far down with trim as without", reach(1) >= reach(2) - 1e-9, f"{reach(1):.1f} ft with trim, {reach(2):.1f} ft without (to the nearest of the places looked at)")
else:
    check("the south wall's two corner boards are on the map", False, f"{len(cb)} found")
# 5. Trim off is the house as it was drawn before: the same picture as the job with no trim line priced
n = load("10")[:BAR]
# The price bar at the top says how many lines are priced (the job with its trim lines taken off has fewer), and an edge is a
# dot or two different from one run to the next (the picture is smoothed over time): so what is compared is every place where
# a patch of 3 by 3 dots differs, below the bars.
m = (np.abs(b - n).max(axis=2) > 24)[110:]
core = m[1:-1, 1:-1].copy()
for dy in (-1, 0, 1):
    for dx in (-1, 0, 1):
        core &= m[1 + dy:m.shape[0] - 1 + dy, 1 + dx:m.shape[1] - 1 + dx]
check("Trim off is the job with no trim line priced: no patch of the house differs", core.sum() == 0, f"{int(core.sum())} patches of 3 by 3 dots differ ({int(m.sum())} single dots, on the edges)")
sys.exit(0 if ok else 1)
PY

echo
echo "== Call 193 A: three houses, 24 by 16 ft, lap siding, a window and a door: no trim priced, casing by the each, casing and corner boards by their lengths =="
# house193 <file> <title> <window: none|each|length> <door: none|length> <corner boards: no|yes>
house193() {
  local H="$1" S WT NT DT SO EA NO WE WI DO
  "$P" new "$H" "$2" >/dev/null
  S="$(made "$("$P" level "$H" ground --elevation "0 ft" --height "9 ft" "${A[@]}")")"
  WT="$(made "$("$P" type "$H" wall "wall 6 in" --thickness "6 in" "${A[@]}")")"
  NT="$(made "$("$P" type "$H" window "window 3 ft" --width "3 ft" --height "5 ft" "${A[@]}")")"
  DT="$(made "$("$P" type "$H" door "door 3 ft" --width "3 ft" --height "6 ft 8 in" "${A[@]}")")"
  read -r SO EA NO WE <<<"$("$P" wall "$H" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${A[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
  WI="$(made "$("$P" window "$H" "$SO" "$NT" --offset "18 ft" --sill "2.5 ft" "${A[@]}")")"
  DO="$(made "$("$P" door "$H" "$SO" "$DT" --offset "6 ft" --sill "0 ft" "${A[@]}")")"
  "$P" roof "$H" "Main roof" --points "-1,-1 25,-1 25,17 -1,17" --pitch 6:12 --eave-height "9 ft" "${A[@]}" >/dev/null
  local R2; R2="$(grep -oE 'roof-[0-9]+' "$H" | head -1)"
  "$P" set "$H" "$R2" edge:2 gable "${A[@]}" >/dev/null
  "$P" set "$H" "$R2" edge:4 gable "${A[@]}" >/dev/null
  "$P" import assemblies "$H" starter >/dev/null
  for w in $SO $EA $NO $WE; do "$P" apply "$H" "$w" "Exterior wall, 2x6, lap siding" --source "$L" >/dev/null; [ "$5" = yes ] || "$P" remove-line "$H" "$w" "Corner boards" >/dev/null; done
  case "$3" in
    each) "$P" apply "$H" "$WI" "Window" --source "$L" >/dev/null; "$P" remove-line "$H" "$WI" "Exterior casing" >/dev/null ;;
    length) "$P" apply "$H" "$WI" "Window" --source "$L" >/dev/null ;;
  esac
  [ "$4" = length ] && "$P" apply "$H" "$DO" "Entry door" --source "$L" >/dev/null
  echo "$WI $DO"
}
H1="$T/h1.json"; H2="$T/h2.json"; H3="$T/h3.json"
read -r W1 D1 <<<"$(house193 "$H1" "No trim priced (not a real job)" none none no)"
read -r W2 D2 <<<"$(house193 "$H2" "Casing by the each (not a real job)" each none no)"
read -r W3 D3 <<<"$(house193 "$H3" "Casing and corner boards by length (not a real job)" length length yes)"
[ "$W1" = "$W3" ] && [ "$D1" = "$D3" ] && [ "$W2" = "$W3" ] || { echo "FAIL: the three houses were not made the same way ($W1 $W2 $W3 / $D1 $D2 $D3)"; exit 1; }
[ -z "$("$P" trim "$H1" --pieces | grep -E '^ *(fascia|soffit|corner boards|casing)' || true)" ] || { echo "FAIL: house 1 has trim priced"; exit 1; }
"$P" trim "$H3" --pieces | grep -E '^ *(corner boards|casing)' | sed 's/^/    /'
for n in 1 2 3; do
  OUTH="$T/h$n"; mkdir -p "$OUTH"
  cat > "$T/session-h$n.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUTH" ICD="$ICD" LOG="$T/app-h$n.log"
. "$PWD/demos/window.sh"
open_window "$T/h$n.json"
on "Show panel Job"; on "Show panel Details"; on "Show panel Tools"
park; sleep 6
cp "\$PLUMBLINE_MAP" "$T/map-h$n.txt"
shot                                    # the street view
close_window
SESSION
  chmod +x "$T/session-h$n.sh"
  timeout 900 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session-h$n.sh" || true
  cp "$OUTH/01-model.png" "$OUT/1$n-model.png" 2>/dev/null || { echo "FAIL: no picture 1$n"; exit 1; }
  cp "$OUTH/01-sheet.png" "$OUT/1$n-sheet.png" 2>/dev/null || true
done
[ -z "$(grep -P '^count\tTrim' "$T/map-h1.txt")" ] && [ -z "$(grep -P '^trim\t' "$T/map-h1.txt")" ] || { echo "FAIL: house 1, with no trim priced, lists trim"; exit 1; }
! awk -F'\t' '$2=="Show Trim"' "$T/map-h1.txt" | grep -q . || { echo "FAIL: house 1 offers a Trim switch"; exit 1; }
echo "    house 1, no trim priced: no Trim count, no pieces, no Trim switch"
[ "$(awk -F'\t' '$1=="count" && $2=="Trim" {print $3}' "$T/map-h3.txt")" = 15 ] || { echo "FAIL: the switch of house 3 does not read 15"; grep -P '^count' "$T/map-h3.txt" || echo none; exit 1; }
[ "$(grep -c -P '^trim\t' "$T/map-h3.txt")" = 15 ] || { echo "FAIL: house 3's map does not list 15 pieces"; exit 1; }
echo "    house 3, casing and corner boards by their lengths: the switch reads 15, the map lists 15 pieces (8 corner boards, 7 of casing)"
python3 -W ignore -I - "$OUT" "$T" "$W3" "$D3" <<'PY'
import sys, math
import numpy as np
from PIL import Image
out, T, win, door = sys.argv[1:5]
def load(n): return np.asarray(Image.open(f"{out}/{n}-model.png").convert("RGB")).astype(int)
p1, p2, p3 = load("11")[:915], load("12")[:915], load("13")[:915]
rows = []
for line in open(f"{T}/map-h3.txt"):
    f = line.rstrip("\n").split("\t")
    if f[0] == "trim": rows.append((f[1], f[2], *map(float, f[3:8])))
def strip_mask(kind, of, margin=14):
    m = np.zeros(p1.shape[:2], bool)
    for k, o, x1, y1, x2, y2, w in rows:
        if k != kind or (of and o != of): continue
        n = max(2, int(math.hypot(x2 - x1, y2 - y1) / 2))
        for t in np.linspace(0, 1, n):
            x, y = int(x1 + t * (x2 - x1)), int(y1 + t * (y2 - y1))
            m[max(0, y - margin):y + margin + 1, max(0, x - margin):x + margin + 1] = True
    return m
def differs(a, b, mask): return int((np.abs(a - b).max(axis=2) > 24)[mask].sum())
ok = True
def check(what, good, said):
    global ok
    ok = ok and good
    print(("  ok   " if good else "  FAIL ") + what + ": " + said)
mw, md, mc = strip_mask("casing", win), strip_mask("casing", door), strip_mask("corner boards", "")
a = differs(p1, p2, mw); b = differs(p1, p2, md); c = differs(p1, p2, mc)
check("(2) the window is cased where (1) is not", a >= 100, f"{a} dots differ round the window's casing")
check("(2) the door, with no casing line, is as in (1)", b <= 80, f"{b} dots differ round the door's casing")
check("(2) the corners, with no corner board priced, are as in (1)", c <= 80, f"{c} dots differ along the corner boards")
a = differs(p1, p3, mw); b = differs(p1, p3, md); c = differs(p1, p3, mc)
check("(3) the window is cased by the model's casing", a >= 100, f"{a} dots differ round the window's casing")
check("(3) the door is cased by the model's casing", b >= 100, f"{b} dots differ round the door's casing")
check("(3) the corner boards are there", c >= 100, f"{c} dots differ along the corner boards")
# nothing else of the house differs between (1) and (2): the picture, but for round the window
m = (np.abs(p1 - p2).max(axis=2) > 24)[110:]
core = m[1:-1, 1:-1].copy()
for dy in (-1, 0, 1):
    for dx in (-1, 0, 1):
        core &= m[1 + dy:m.shape[0] - 1 + dy, 1 + dx:m.shape[1] - 1 + dx]
outside = core & ~mw[110:][1:-1, 1:-1]
check("(1) and (2) are one house but for the window's casing", outside.sum() == 0, f"{int(outside.sum())} patches of 3 by 3 dots differ away from the window's casing")
sys.exit(0 if ok else 1)
PY
echo
echo "== The words said on a click =="
for w in corner casing fascia; do printf '    %-7s %s\n' "$w" "$(cat "$T/said-$w.txt")"; done
grep -q "^Selected $(sed 's/-/ /;s/^./\U&/' <<<"$SOUTH")\.\?" "$T/said-corner.txt" || echo "    (check by eye: which wall the corner board of $SOUTH's side selected)"
echo "  pictures saved to $OUT"
