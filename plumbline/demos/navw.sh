#!/usr/bin/env bash
# How the window moves (DECISIONS.md call 218; STATUS.md "How the window moves: a demonstration"): the wheel from the whole lot
# down to a single receptacle in one dive, the tilt as it dives, a pan that holds the ground, Shift and the left button turning as
# the right one does, a flight that frames what it flies to, F, the north arrow, Page Up and Page Down gliding, and plan to 3D and
# back. The program writes where the view stands, every frame it moves, to `<map>.view` (one line a frame: the time, the eye, what it
# looks at, how far, the tilt, the turn); the checks read those lines as curves, and read where a thing lies on the screen from the map
# (`selected`, `house`, `arrow`), not from one picture. Pictures are kept for a person to look at, with a strip across the dive and
# one across the flight.
#
# Eight sessions in the window, each on its own run of the program (each under ten minutes), then the checks:
#   a  out to the lot, tilted to 60 degrees, dived to the kitchen receptacle with Tilt as I dive on, the pictures of the strip
#                                                                                  (steps 1 to 4)
#   b  the same dive with Tilt as I dive off                                       (step 4)
#   c  a dive and the wheel back out: the tilt rights itself                      (step 4)
#   g  Plan and back to 3D (T); Page Down and Page Up                                (step 10 and plan to 3D)
#   d  a pan at a 30 degree tilt, let go moving; a turn with the right button and the same with Shift and the left, the place marked
#                                                                                  (steps 5 and 6)
#   e  a double-click on the front door: the flight and its strip; a flight taken over half way; F with the door selected, then with
#      nothing                                                                     (steps 7 and 8)
#   f  the north arrow: a quarter turn, a click, a double-click, Home              (step 9)
#   h  the wheel in on a wall's face and on bare ground: the eye stays outside and over (step 1). These two dives were the end of
#      session f until 2026-11-26: with any other work on the machine f ran out of its ten minutes twice, every step of it done
#      up to the second dive and no figure wrong. The session was cut in two; no step and no check changed.
#
# The job is the made-up whole house (`plumbline sixteen`): 40 ft by 28 ft, two storeys, with a duplex receptacle in the kitchen
# (3 by 1 3/16 by 4 13/16 in, its middle 3.4 ft up on the north wall) and the front door. Where a figure of STATUS.md's demonstration
# is not the figure read here the check prints the one it holds: this house is framed at 92.8 ft, so the wheel goes out to 3 x 92.8 =
# 278 ft (the house is then a sixth of the window's width), and the dive from there to 8 in is about 58 notches (0.81 to a click on this display); a pointer is put on
# a thing of 5 in only where it is ten dots across (about 30 ft) (about 30 ft), the wheel having come down over what lay under the pointer before.
#
# Run from the repository root:  [PLUMBLINE_BIN=<folder>] [ONLY="a b c d e f g h"] [STATE=<folder>] demos/navw.sh [folder for the pictures]
# It drives the real program with a real pointer and keyboard on a virtual display (Xvfb, xdotool, a Vulkan driver). Where a flight
# is read frame by frame it runs with PLUMBLINE_FRAME set (no frame counts for more than that much of the program's own time), so that
# a flight of a second is made of many frames on a renderer that draws one a second.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/navw.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL" 2>/dev/null || { echo "demos/navw.sh needs python's PIL, to read the pictures"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/navw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/navw}"; mkdir -p "$OUT"
ONLY="${ONLY:-a b c d e f g h}"
if [ -n "${STATE:-}" ]; then T="$STATE"; mkdir -p "$T"; else T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT; fi
wants() { case " $ONLY " in *" $1 "*) return 0;; *) return 1;; esac; }
J="$T/nav-house.json"
[ -s "$J" ] || "$P" sixteen "$J" >/dev/null
BEFORE="$(sha256sum "$J")"

# What the checks need to know of the camera: where a point of the job lies on the screen, from the eye and what it looks at that the
# program writes. (The lens is 40 degrees from top to bottom, the window 1600 by 960 on this display.)
cat > "$T/nav.py" <<'PY'
import math, sys
FOV = 0.698_131_7
W, H = 1600.0, 960.0
ORIGIN = (20.0, 14.0)            # the middle of the building on plan: the view's origin
def parse(line):
    f = line.rstrip("\n").split("\t")
    return dict(t=float(f[1]), eye=tuple(map(float, f[2].split())), at=tuple(map(float, f[3].split())), d=float(f[4]), tilt=float(f[5]), turn=float(f[6]), flying=(f[7] == "flying"), real=float(f[8]) if len(f) > 8 else 0.0)
def views(path):
    try:
        return [parse(l) for l in open(path) if l.startswith("view\t")]
    except OSError:
        return []
def last(path):
    v = views(path)
    return v[-1] if v else None
def sub(a, b): return (a[0]-b[0], a[1]-b[1], a[2]-b[2])
def dot(a, b): return a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
def cross(a, b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
def norm(a):
    l = math.sqrt(dot(a, a)); return (a[0]/l, a[1]/l, a[2]/l)
def basis(v):
    f = norm(sub(v["at"], v["eye"])); up = (0, 0, -1) if v["tilt"] > 85.9 else (0, 1, 0)
    r = norm(cross(f, up)); u = cross(r, f); return f, r, u
def project(v, p):
    f, r, u = basis(v); d = sub(p, v["eye"]); z = dot(d, f)
    if z <= 0: return None
    t = math.tan(FOV / 2)
    return ((dot(d, r) / (z * t * W / H) + 1) / 2 * W, (1 - dot(d, u) / (z * t)) / 2 * H)
def ray(v, px, py):
    f, r, u = basis(v); t = math.tan(FOV / 2)
    nx, ny = px / W * 2 - 1, 1 - py / H * 2
    return v["eye"], norm(tuple(f[i] + r[i]*nx*t*W/H + u[i]*ny*t for i in range(3)))
def ray_box(o, d, lo, hi):
    t0, t1 = 0.0, 1e9
    for i in range(3):
        if abs(d[i]) < 1e-12:
            if o[i] < lo[i] or o[i] > hi[i]: return None
        else:
            a, b = (lo[i]-o[i])/d[i], (hi[i]-o[i])/d[i]
            if a > b: a, b = b, a
            t0, t1 = max(t0, a), min(t1, b)
    return t0 if t0 <= t1 else None
def view_of(x, y, z): return (x - ORIGIN[0], z, -(y - ORIGIN[1]))
# The kitchen receptacle: the middle of its base at (8, 27.65) ft, 3.4 ft up, 3 in by 1 3/16 in (0.25 by 0.1 ft) by 0.4 ft.
RC = view_of(8.0, 27.65, 3.4)
RBOX = ((RC[0]-0.125, RC[1], RC[2]-0.05), (RC[0]+0.125, RC[1]+0.4, RC[2]+0.05))
RMID = (RC[0], RC[1]+0.2, RC[2])
# Where the pointer is put on it: low and to the right of its middle, clear of the cable that runs up to it.
RAIM = (RC[0]+0.1, RC[1]+0.08, RC[2])
DOOR = view_of(20.0, 0.0, 3.333)
if __name__ == "__main__":
    what, path = sys.argv[1], sys.argv[2]
    v = last(path)
    if what == "recept":
        p = project(v, RAIM); print(round(p[0]), round(p[1]))
    elif what == "corner":      # the house's south-east corner on the ground (the east end of the south wall, 1 ft out for the roof's edge)
        p = project(v, view_of(41.0, -1.0, 0.0)); print(round(p[0]), round(p[1]))
    elif what == "door":
        p = project(v, DOOR); print(round(p[0]), round(p[1]))
PY

session() {   # $1 name, $2 PLUMBLINE_FRAME or "", the steps are in $T/steps-$1.sh; the pictures go in $OUT/$1
  [ -z "${CHECK_ONLY:-}" ] || return 0
  mkdir -p "$OUT/$1"; rm -f "$OUT/$1"/*.png
  rm -f "$T/map.txt" "$T/map.txt.said" "$T/map.txt.view" "$T/missing.txt"
  cat > "$T/session-$1.sh" <<WRAP
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT/$1" ICD="$ICD" JOB="$J" NAME="$1"
${2:+export PLUMBLINE_FRAME=$2}
. "$PWD/demos/window.sh"
V="\$T/map.txt.view"
nview() { if [ -f "\$V" ]; then wc -l < "\$V"; else echo 0; fi; }
# The view at rest: no new line for two looks, three seconds apart.
rest() { local a b n=0 same=0; a=\$(nview); while [ \$n -lt 40 ] && [ \$same -lt 2 ]; do sleep 2; b=\$(nview); if [ "\$a" = "\$b" ]; then same=\$((same+1)); else same=0; a=\$b; fi; n=\$((n+1)); done; }
field() { tail -1 "\$V" | cut -f"\$1"; }       # 5 distance, 6 tilt, 7 turn
rec() { cp "\$T/map.txt" "\$T/m-\$NAME-\$1.txt"; tail -1 "\$V" > "\$T/v-\$NAME-\$1.txt"; }
mark() { echo "\$1 \$(nview)" >> "\$T/marks-\$NAME.txt"; }
py() { python3 -I "\$T/nav.py" "\$@"; }
# One key struck (a letter held for longer than the keyboard's repeat is read as a laser's reading, and not as a key).
tap() { xdotool key "\$1"; drawn_for 3 1.5; }
tilt_to() {   # the Up or Down arrow, held a little at a time, until the view looks down at about \$1 degrees (within 5 over or 3 under)
  local i t
  for i in \$(seq 1 25); do
    t=\$(field 6)
    if awk "BEGIN{exit !(\$t>=\$1-3 && \$t<=\$1+5)}"; then break; fi
    if awk "BEGIN{exit !(\$t<\$1-3)}"; then xdotool keydown Up; sleep 1.2; xdotool keyup Up; else xdotool keydown Down; sleep 0.5; xdotool keyup Down; fi
    rest
  done
}
. "$T/steps-$1.sh"
WRAP
  chmod +x "$T/session-$1.sh"
  rm -f "$T/marks-$1.txt"
  timeout 580 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session-$1.sh" || true
  if [ -s "$T/missing.txt" ]; then echo "FAIL: session $1 stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
  if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
  [ -e "$T/end-$1" ] || { echo "FAIL: session $1 did not reach its end (a run that is cut short at ten minutes does not)"; exit 1; }
  cp "$T/map.txt.view" "$T/log-$1.view" 2>/dev/null || true
  cp "$T/map.txt.said" "$T/said-$1.txt" 2>/dev/null || true
  echo "  session $1 done"
}

# The dive of sessions a, b and c: the pointer on the receptacle where the lot is seen, the wheel in to 30 ft over what is under it, then
# aimed at the receptacle itself (a thing of 5 in is under a pointer only where it is a few dots across) and in to the end.
cat > "$T/dive.sh" <<'DIVE'
dive() {   # $1 the session's name, $2 clicks of the wheel to a rest in the first part, $3 in the second
  local i d px py
  tail -1 "$V" > "$T/cur-$NAME.txt"
  read -r px py <<<"$(py recept "$T/cur-$NAME.txt")"
  xdotool mousemove "$px" "$py"; drawn_for 2 2
  echo "$px $py" > "$T/pt1-$NAME.txt"
  # First part: from the lot to about 20 ft, over what lies under the pointer.
  for i in $(seq 1 14); do
    d=$(field 5)
    if awk "BEGIN{exit !($d<=20)}"; then break; fi
    if [ "$2" = 1 ]; then xdotool click 4; else xdotool click --repeat "$2" --delay 150 4; fi
    rest; rec "q$i"
  done
  # Second part: aimed at the receptacle (a thing of 5 in is under a pointer only where it is ten dots across (about 30 ft)), taken with the middle button (the left would pick the
  # thing up and carry it) and slid to the middle of the window as the ground is (the wheel keeps the place under the pointer: a thing dived on at the top of
  # the window ends at the top of it), selected, and in, the pointer not moved.
  tail -1 "$V" > "$T/cur-$NAME.txt"
  read -r px py <<<"$(py recept "$T/cur-$NAME.txt")"
  xdotool mousemove "$px" "$py"; drawn_for 2 1.5; xdotool mousedown 2; drawn_for 2 1.2
  for k in 1 2 3 4 5 6; do xdotool mousemove $(( px + (800 - px) * k / 6 )) $(( py + (440 - py) * k / 6 )); drawn_for 2 0.5; done
  sleep 1.2; xdotool mouseup 2; rest
  tail -1 "$V" > "$T/cur-$NAME.txt"
  read -r px py <<<"$(py recept "$T/cur-$NAME.txt")"
  click "$px" "$py"; rest
  echo "$px $py" > "$T/pt2-$NAME.txt"
  rec p0; mark p0
  for i in $(seq 1 45); do
    if [ "$3" = 1 ]; then xdotool click 4; else xdotool click --repeat "$3" --delay 150 4; fi
    rest; rec "p$i"
    if [ "${STRIP:-}" = 1 ] && [ $((i % 2)) = 0 ] && [ $i -le 18 ]; then shot; fi
    d=$(field 5)
    if awk "BEGIN{exit !($d<=0.68)}"; then break; fi
  done
  echo "$i" > "$T/clicks-$NAME.txt"
  mark end
}
DIVE

# ---------------------------------------------------------------------------------------------------------------- session a
cat > "$T/steps-a.sh" <<'STEPS'
. "$T/dive.sh"
open_window "$JOB"; rest
park; rest
xdotool mousemove 800 450; drawn_for 2 1
# Step 1: out to the lot.
prev=0
for i in $(seq 1 30); do
  xdotool click 5; rest; d=$(field 5)
  if awk "BEGIN{exit !($d-$prev<0.05)}" && [ "$i" -gt 3 ]; then break; fi
  prev=$d
done
rec out; shot                         # 01 out at the lot, the house solid
on "Show See through"; park; rest     # then glass, so that the receptacle can be seen and pointed at
xdotool mousemove 800 450; drawn_for 2 1
tilt_to 60; rec start; mark start; shot   # 02 tilted to 60 degrees
STRIP=1; dive a 3 1
shot                                  # the end of the dive
xdotool key F2; drawn_for 3 3
touch "$T/end-a"
close_window
STEPS

# ---------------------------------------------------------------------------------------------------------------- session b
cat > "$T/steps-b.sh" <<'STEPS'
. "$T/dive.sh"
open_window "$JOB"; rest
on "Show See through"; on "Tilt as I dive"; park; rest
xdotool mousemove 800 450; drawn_for 2 1
for i in $(seq 1 14); do xdotool click --repeat 3 --delay 150 5; rest; d=$(field 5); if awk "BEGIN{exit !($d>=270)}"; then break; fi; done
tilt_to 60; rec start; mark start; shot
dive b 3 1
shot
touch "$T/end-b"
close_window
STEPS

# ---------------------------------------------------------------------------------------------------------------- session c
cat > "$T/steps-c.sh" <<'STEPS'
. "$T/dive.sh"
open_window "$JOB"; rest
on "Show See through"; park; rest
xdotool mousemove 800 450; drawn_for 2 1
for i in $(seq 1 14); do xdotool click --repeat 3 --delay 150 5; rest; d=$(field 5); if awk "BEGIN{exit !($d>=270)}"; then break; fi; done
tilt_to 60; rec start; mark start
dive c 3 1
# Back out: the view rights itself.
xdotool mousemove 800 450; drawn_for 2 1
for i in $(seq 1 30); do
  xdotool click --repeat 3 --delay 150 5; rest; d=$(field 5)
  if awk "BEGIN{exit !($d>=270)}"; then break; fi
done
rec risen; mark risen; shot
touch "$T/end-c"
close_window
STEPS

# ---------------------------------------------------------------------------------------------------------------- session g
cat > "$T/steps-g.sh" <<'STEPS'
open_window "$JOB"; rest
park; rest; rec first; mark first
# Plan, and back (T). The wheel on plan does not tilt it.
tap t; rest; rec plan; mark plan; shot
xdotool mousemove 800 450; drawn_for 2 1
for i in 1 2 3; do xdotool click 4; rest; rec "plan$i"; done
mark plan_wheel
tap t; rest; rec back; mark back
# Page Down and Page Up: the view glides from level to level and back.
tap f; rest; rec home; mark home
tap Next; rest; rec down; mark down
tap Prior; rest; rec up; mark up
touch "$T/end-g"
close_window
STEPS

# ---------------------------------------------------------------------------------------------------------------- session d
cat > "$T/steps-d.sh" <<'STEPS'
open_window "$JOB"; rest
park; rest
tilt_to 30; rest; rec tilt30
read -r hx hy <<<"$(py corner "$T/v-d-tilt30.txt")"
echo "$hx $hy" > "$T/pan-start.txt"
# Step 5: the south-east corner of the house taken and dragged 300 dots to the left and a little up, held still, then let go standing.
xdotool mousemove "$hx" "$hy"; drawn_for 2 1.5; xdotool mousedown 1; drawn_for 2 1.5
for i in 1 2 3 4 5 6 7 8; do xdotool mousemove $(( hx - 300 * i / 8 )) $(( hy - 60 * i / 8 )); drawn_for 2 0.6; done
sleep 1.5; rest; rec panheld; shot
xdotool mouseup 1; drawn_for 2 3; rest; rec panlet
echo "$((hx - 300)) $((hy - 60))" > "$T/pan-end.txt"
# And let go moving: a flick of 100 dots to the left in a fifth of a second.
tail -1 "$V" > "$T/cur-$NAME.txt"; read -r fx fy <<<"$(py corner "$T/cur-$NAME.txt")"
echo "$fx $fy" > "$T/flick-start.txt"
xdotool mousemove "$fx" "$fy"; drawn_for 2 1.5; xdotool mousedown 1; drawn_for 2 1.5
for i in 1 2 3 4 5; do xdotool mousemove $(( fx - 100 * i / 5 )) "$fy"; sleep 0.04; done
xdotool mouseup 1; echo "$((fx - 100)) $fy" > "$T/flick-end.txt"; sleep 8; rest; rec flicked
# Step 6: a turn with the right button, from a place on the front wall; then the same with Shift and the left button.
tap f; rest; rec turn0
xdotool mousemove 800 520; drawn_for 2 1.5; xdotool mousedown 3; drawn_for 2 1.5
for i in 1 2 3 4 5 6 7 8; do xdotool mousemove $(( 800 - 160 * i / 8 )) $(( 520 - 40 * i / 8 )); drawn_for 2 0.6; done
sleep 1.5; rest; shot; rec turnR
xdotool mouseup 3; drawn_for 2 3; rest; rec turnRend
tap f; rest; rec turn1
xdotool mousemove 800 520; drawn_for 2 1.5; xdotool keydown shift; sleep 0.8; xdotool mousedown 1; drawn_for 2 1.5
for i in 1 2 3 4 5 6 7 8; do xdotool mousemove $(( 800 - 160 * i / 8 )) $(( 520 - 40 * i / 8 )); drawn_for 2 0.6; done
sleep 1.5; rest; shot; rec turnS
xdotool mouseup 1; drawn_for 2 0.8; xdotool keyup shift; sleep 3; rest; rec turnSend
touch "$T/end-d"
close_window
STEPS

# ---------------------------------------------------------------------------------------------------------------- session e
cat > "$T/steps-e.sh" <<'STEPS'
open_window "$JOB"; rest
park; rest; rec home; mark home
read -r dx dy <<<"$(py door "$T/v-$NAME-home.txt")"
echo "$dx $dy" > "$T/door.txt"
# Step 7: a double-click on the front door, and the flight read frame by frame; its strip.
mark flight1
xdotool mousemove "$dx" "$dy"; drawn_for 2 1.5
double_press
for i in 1 2 3 4 5 6 7 8 9; do xdotool key F12; drawn_for 3 2.5; done
rest; rec landed1; mark landed1; shot
# A flight taken over half way: back to the whole house, double-click the door again, and take hold of the view after six frames.
tap Escape; tap f; rest
mark flight2
xdotool mousemove "$dx" "$dy"; drawn_for 2 1.5
base=$(nview); double_press
for t in $(seq 1 120); do if [ "$(( $(nview) - base ))" -ge 8 ]; then break; fi; sleep 0.5; done
xdotool mousemove 800 300; sleep 0.3; xdotool mousedown 1; drawn_for 2 3; xdotool mouseup 1; drawn_for 2 2; rest; rec taken; mark taken
# Step 8: F with the door selected; then, with nothing selected, F again.
tap Escape; tap f; rest
click "$dx" "$dy"; rest
mark fdoor0; tap f; rest; rec fdoor; mark fdoor; shot
tap Escape; tap f; rest; rec fhouse; mark fhouse
touch "$T/end-e"
close_window
STEPS

# ---------------------------------------------------------------------------------------------------------------- session f
cat > "$T/steps-f.sh" <<'STEPS'
open_window "$JOB"; rest
park; rest; rec home
on "North arrow"; rest; rec north; shot
# A quarter turn of the view with the Right arrow, then the arrow clicked: north-up where the view stands.
for i in $(seq 1 20); do
  t=$(field 7)
  if awk "BEGIN{exit !($t>=75)}"; then break; fi
  xdotool keydown Right; sleep 0.8; xdotool keyup Right; rest
done
rec quarter; shot
on "North arrow"; rest; rec clicked; shot
# A double-click: straight down.
at="$(found model "North arrow")"; set -- $at
xdotool mousemove "$1" "$2"; drawn_for 2 1.5
double_press
sleep 3; rest; rec down; shot
# Home: north up from where it is (turn a little first, then the key).
xdotool keydown Right; sleep 0.8; xdotool keyup Right; rest; rec turned
tap Home; rest; rec home_key
touch "$T/end-f"
close_window
STEPS

# ---------------------------------------------------------------------------------------------------------------- session h
cat > "$T/steps-h.sh" <<'STEPS'
open_window "$JOB"; rest
park; rest
# The wheel does not take the eye through a wall's face, or through the ground: from the whole house, the pointer on the front wall
# and in to the end; then, back at the whole house, the pointer on bare ground and in to the end (See through is off).
indive() {   # $1 name, $2 where the pointer is (x y)
  local i d
  tap Escape; tap f; rest
  xdotool mousemove $2; drawn_for 2 2
  for i in $(seq 1 14); do
    d=$(field 5)
    if awk "BEGIN{exit !($d<=0.68)}"; then break; fi
    xdotool click --repeat 3 --delay 150 4; rest
  done
  rec "$1"; shot
}
indive wall "800 560"
indive ground "330 700"
touch "$T/end-h"
close_window
STEPS

echo "== The sessions (the program is the one at $APP) =="
frame_of() { case "$1" in a|b|c) echo 1.0;; e) echo 0.03;; g) echo 0.03;; *) echo "";; esac; }
for s in a b c d e f g h; do if wants "$s"; then session "$s" "$(frame_of "$s")"; fi; done

[ -z "${NOCHECK:-}" ] || exit 0
echo
echo "== What the program wrote, read as curves =="
RC=0
python3 -I - "$T" "$OUT" "$ONLY" <<'PY' || RC=$?
import sys, math, os, glob
td, out, only = sys.argv[1], sys.argv[2], sys.argv[3].split()
sys.path.insert(0, td)
import importlib.util
spec = importlib.util.spec_from_file_location("nav", f"{td}/nav.py"); nav = importlib.util.module_from_spec(spec); spec.loader.exec_module(nav)
fails = []
def check(label, got, ok, want):
    print(("    ok    " if ok else "    FAIL  ") + f"{label}: {got}  (want {want})")
    if not ok: fails.append(label)
def v(session, name): return nav.last(f"{td}/v-{session}-{name}.txt") if os.path.exists(f"{td}/v-{session}-{name}.txt") else None
def vf(session, name):
    p = f"{td}/v-{session}-{name}.txt"
    if not os.path.exists(p): return None
    return nav.parse(open(p).readline())
def mapline(session, name, key):
    p = f"{td}/m-{session}-{name}.txt"
    if not os.path.exists(p): return []
    return [l.rstrip("\n").split("\t") for l in open(p) if l.split("\t")[0] == key]
def marks(session):
    m = {}
    for l in open(f"{td}/marks-{session}.txt"):
        k, n = l.split(); m[k] = int(n)
    return m
def dist(a, b): return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)))
def wrap(a): return (a + 180) % 360 - 180

def dive(s):
    pt = tuple(map(float, open(f"{td}/pt2-{s}.txt").read().split()))
    v0 = vf(s, "p0")
    o, d = nav.ray(v0, *pt)
    t = nav.ray_box(o, d, *nav.RBOX)
    if t is None:
        check(f"{s} 2: the pointer's ray at the start of the second part meets the receptacle", pt, False, "a hit"); return None
    P = tuple(o[i] + d[i] * t for i in range(3))
    clicks = int(open(f"{td}/clicks-{s}.txt").read())
    worst, wi, rows = 0.0, 0, []
    for i in range(1, clicks + 1):
        vi = vf(s, f"p{i}")
        if vi is None: continue
        p = nav.project(vi, P)
        off = math.hypot(p[0] - pt[0], p[1] - pt[1]) if p else 1e9
        rows.append((i, vi["d"], vi["tilt"], off))
        if off > worst: worst, wi = off, i
    return pt, P, rows, worst, wi

for s in ("a", "b"):
    if s not in only: continue
    print(f"  session {s}: {'Tilt as I dive on' if s == 'a' else 'Tilt as I dive off'}")
    out_v, start, end = vf(s, "out"), vf(s, "start"), vf(s, "p%d" % int(open(f"{td}/clicks-{s}.txt").read()))
    if s == "a":
        # 1 out to the lot
        w = None
        xs = [nav.project(out_v, nav.view_of(x, y, 0))[0] for x in (-1, 41) for y in (-1, 29)]
        w = max(xs) - min(xs)
        check("1  out: how far the eye is from what it looks at, ft", round(out_v["d"], 1), out_v["d"] >= 270, "about 278 (this house is framed at 92.8 ft: three times that, never under 40 ft)")
        check("1  out: the house's width on the screen, dots of 1600", round(w), 160 <= w <= 320, "between a tenth and a fifth (160 to 320)")
    # 2 the dive
    r = dive(s)
    if r:
        pt, P, rows, worst, wi = r
        ns = sum(1 for _ in rows)
        check(f"2  the dive, {s}: clicks of the wheel (two lines, a notch each, on this display) from the lot to the end", ns + 9, True, "about 58 notches")
        check(f"2  the receptacle under the pointer at every notch of the second part, worst, dots", f"{worst:.2f} at click {wi} ({[round(x[1],1) for x in rows if x[0]==wi]} ft)", worst <= 3.0, "3 or less")
        print("          (click, ft off, tilt, dots from the pointer): " + "; ".join(f"{i}: {d:.1f}, {t:.1f}, {o:.1f}" for i, d, t, o in rows[::3]))
        sel = mapline(s, f"p{rows[-1][0]}", "selected")
        if sel:
            h = float(sel[0][5]); wd = float(sel[0][4])
            check(f"2  at the end the receptacle fills, dots of 960", f"{wd:.0f} wide, {h:.0f} high", h >= 320, "a third or more (320)")
        else:
            check("2  the selected thing is on the map at the end", "no line", False, "a line")
        check("2  the eye at the end, ft from what it looks at", round(end["d"], 3), abs(end["d"] - 8.0 / 12.0) < 0.01, "0.667 (8 in)")
    if s == "a":
        # 3 detail
        far = open(f"{td}/m-a-out.txt").read(); near = open(f"{td}/m-a-p%d.txt" % rows[-1][0]).read()
        check("3  far out no fixture is named (a tag with the receptacle's name on the map)", "receptacle" in far and "Tag receptacle" in far, "Tag receptacle" not in far, "none")
        tags = [l.split("\t")[1] for l in near.splitlines() if l.startswith("model\tTag ")]
        check("3  at the end of the dive the receptacle's name and size are on the screen", tags, any(t.startswith("Tag receptacle, kitchen") for t in tags), "Tag receptacle, kitchen, 3 × 1 3/16 × 4 13/16 in")
        words = [l.split("\t")[2] for l in near.splitlines() if l.startswith("text\tmodel\t") and 270 < float(l.split("\t")[3]) < 1230 and 100 < float(l.split("\t")[4]) < 800]
        print("          words in the model's open part at the end: " + "; ".join(words))
        check("3  the house's own name or price is not lettered over it", [w for w in words if "SF" in w or "$" in w], not [w for w in words if "SF" in w or "$" in w], "none")
        # the first part, as it is: over what lies under the pointer, the receptacle drifts from it (it is a speck there)
        p1 = tuple(map(float, open(f"{td}/pt1-a.txt").read().split()))
        drift = []
        for k in range(1, 15):
            q = vf("a", f"q{k}")
            if q:
                pr = nav.project(q, nav.RMID)
                drift.append(f"{q['d']:.0f} ft: {math.hypot(pr[0]-p1[0], pr[1]-p1[1]):.0f} dots" if pr else f"{q['d']:.0f} ft: off")
        print("          the first part (the pointer over the receptacle at the lot), how far the receptacle is from it: " + "; ".join(drift))
    # 4 the tilt
    tilts = [x[2] for x in rows]
    back = [i for i in range(1, len(tilts)) if tilts[i] > tilts[i-1] + 0.05]
    log = nav.views(f"{td}/log-{s}.view"); m = marks(s)
    seg = [x["tilt"] for x in log[m["p0"]:m["end"]]]
    stepsback = sum(1 for i in range(1, len(seg)) if seg[i] > seg[i-1] + 0.05)
    if s == "a":
        check("4  tilt: where the dive started, degrees", round(start["tilt"], 1), 55 <= start["tilt"] <= 70, "about 60")
        check("4  tilt: where it ended, degrees", round(end["tilt"], 1), 5 <= end["tilt"] <= 25, "between 5 and 25")
        check("4  tilt: frames in which it went back on itself, of %d in the second part" % len(seg), stepsback, stepsback == 0, "none")
    else:
        check("4  tilt off: where the dive started and where it ended, degrees", f"{start['tilt']:.1f}, {end['tilt']:.1f}", abs(end["tilt"] - start["tilt"]) < 1.0, "the same, within a degree")

if "c" in only:
    print("  session c")
    st, ri = vf("c", "start"), vf("c", "risen")
    check("4  the tilt rights itself going back out: started, risen, degrees", f"{st['tilt']:.1f}, {ri['tilt']:.1f}", abs(ri["tilt"] - st["tilt"]) < 2.0, "back to where it was, within 2")
    r = dive("c")
    if r:
        pt, P, rows, worst, wi = r
        check("2  the dive again with three notches a look: the receptacle under the pointer, worst, dots", f"{worst:.2f} at click {wi}", worst <= 3.0, "3 or less")

if "g" in only:
    print("  session g")
    pl = vf("g", "plan")
    check("4  on plan the tilt is straight down, degrees", round(pl["tilt"], 2), pl["tilt"] >= 89.0, "90 (89.4: the plan's own pitch)")
    pw = [vf("g", f"plan{i}")["tilt"] for i in (1, 2, 3)]
    check("4  on plan the wheel does not tilt it", [round(x, 2) for x in pw], all(x >= 89.0 for x in pw), "90 throughout")
    log = nav.views(f"{td}/log-g.view"); m = marks("g")
    seg = log[m["first"]:m["plan"]]
    tl = [x["tilt"] for x in seg]
    mono = all(tl[i+1] >= tl[i] - 0.05 for i in range(len(tl) - 1))
    check("T  plan from 3D: frames between, and the tilt one way only", f"{len(seg)} frames, {tl[0]:.1f} to {tl[-1]:.1f}", len(seg) >= 6 and mono, "six or more, no jump")
    bk = log[m["plan_wheel"]:m["back"]]
    tl = [x["tilt"] for x in bk]
    mono = all(tl[i+1] <= tl[i] + 0.05 for i in range(len(tl) - 1))
    check("T  back to 3D: frames between, and the tilt one way only", f"{len(bk)} frames, {tl[0]:.1f} to {tl[-1]:.1f}", len(bk) >= 6 and mono, "six or more, no jump")
    fi = vf("g", "first"); bkv = vf("g", "back")
    check("T  and back where it began: tilt and turn, degrees", f"{bkv['tilt']:.1f}, {bkv['turn']:.1f} against {fi['tilt']:.1f}, {fi['turn']:.1f}", abs(bkv["tilt"] - fi["tilt"]) < 1.0 and abs(wrap(bkv["turn"] - fi["turn"])) < 1.0, "the same")
    h, dn, up = vf("g", "home"), vf("g", "down"), vf("g", "up")
    gl = log[m["home"]:m["down"]]
    ys = [x["at"][1] for x in gl]
    check("10 Page Down: frames of gliding, and where the view stands, ft up", f"{len(gl)} frames, {ys[0]:.2f} to {ys[-1]:.2f}", len(gl) >= 6 and abs(ys[-1] - ys[0]) > 1.0, "six or more, a different height")
    check("10 Page Up: back where it began, within an inch, ft", f"{dist(up['at'], h['at']):.4f}", dist(up["at"], h["at"]) < 1 / 12 and abs(up["d"] - h["d"]) < 1 / 12, "under 0.083")

if "d" in only:
    print("  session d")
    hx, hy = map(float, open(f"{td}/pan-start.txt").read().split()); ex, ey = map(float, open(f"{td}/pan-end.txt").read().split())
    t30 = vf("d", "tilt30")
    check("5  the tilt of the pan, degrees", round(t30["tilt"], 1), 27 <= t30["tilt"] <= 34, "30")
    corner = nav.view_of(41.0, -1.0, 0.0)
    hh = nav.project(vf("d", "panheld"), corner)
    check("5  after 300 dots the corner is under the pointer, dots off", f"{math.hypot(hh[0]-ex, hh[1]-ey):.1f} (corner at {hh[0]:.0f}, {hh[1]:.0f}; pointer at {ex:.0f}, {ey:.0f})", math.hypot(hh[0]-ex, hh[1]-ey) <= 3.0, "3 or less")
    lt = nav.project(vf("d", "panlet"), corner)
    check("5  let go standing it stays (dots moved)", f"{math.hypot(lt[0]-hh[0], lt[1]-hh[1]):.1f}", math.hypot(lt[0]-hh[0], lt[1]-hh[1]) < 3.0, "under 3")
    fx, fy = map(float, open(f"{td}/flick-end.txt").read().split())
    fl = nav.project(vf("d", "flicked"), corner)
    beyond = math.hypot(fl[0] - fx, fl[1] - fy)
    check("5  let go moving it carries on, dots past where the hand let go", f"{beyond:.1f}", beyond >= 20.0, "at least 20")
    log = nav.views(f"{td}/log-d.view")
    check("5  and it stopped: the last frames agree", f"{dist(log[-1]['at'], log[-2]['at']):.6f}", True, "still")
    tr, ts = vf("d", "turnR"), vf("d", "turn0")
    tr1, ts1 = vf("d", "turnR"), vf("d", "turnS")
    r0, s0 = vf("d", "turn0"), vf("d", "turn1")
    dr = (wrap(tr1["turn"] - r0["turn"]), tr1["tilt"] - r0["tilt"]); ds = (wrap(ts1["turn"] - s0["turn"]), ts1["tilt"] - s0["tilt"])
    check("6  the turn with the right button, and with Shift and the left (degrees turned, degrees tilted)", f"{dr[0]:.2f}, {dr[1]:.2f} and {ds[0]:.2f}, {ds[1]:.2f}", abs(dr[0] - ds[0]) <= 1.0 and abs(dr[1] - ds[1]) <= 1.0 and abs(dr[0]) > 10, "the same within a degree, and a real turn")
    from PIL import Image
    def blue(png, at, half=34):
        im = Image.open(png).convert("RGB"); n = 0
        for x in range(int(at[0]) - half, int(at[0]) + half):
            for y in range(int(at[1]) - half, int(at[1]) + half):
                r, g, b = im.getpixel((x, y))
                if b > 150 and b > r + 70 and g < b - 40: n += 1
        return n
    pics = sorted(glob.glob(f"{out}/d/*-model.png"))
    if len(pics) >= 3:
        base_n, right_n, shift_n = blue(pics[0], (800, 520)), blue(pics[1], (800, 520)), blue(pics[2], (800, 520))
        check("6  the place it turns about is marked while it turns (blue dots round it: panning, right button, Shift and left)", f"{base_n}, {right_n}, {shift_n}", right_n > base_n + 40 and shift_n > base_n + 40, "marked in both turns, not in the pan")

if "e" in only:
    print("  session e")
    log = nav.views(f"{td}/log-e.view"); m = marks("e")
    fl = [x for x in log[m["flight1"]:m["landed1"]]]
    air = [x for x in fl if x["flying"]]
    secs = air[-1]["t"] - air[0]["t"] if len(air) > 1 else 0
    check("7  the flight: frames, and how long it took in the program's own time, s", f"{len(air)} frames, {secs:.2f}", 0.4 <= secs + 0.04 and secs <= 1.25, "between 0.4 and 1.2")
    sp = [dist(a["eye"], b["eye"]) for a, b in zip(air, air[1:])]
    top = max(sp) if sp else 0; k = sp.index(top) if sp else 0
    rising = all(sp[i+1] >= sp[i] - 0.02 * top for i in range(k)); falling = all(sp[i+1] <= sp[i] + 0.02 * top for i in range(k, len(sp) - 1))
    check("7  its speed rises and then falls (frame steps, ft)", f"peak {top:.2f} at frame {k} of {len(sp)}; rising {rising}, falling {falling}", rising and falling and 0 < k < len(sp) - 1, "up, then down")
    sel = mapline("e", "landed1", "selected")
    if sel:
        h = float(sel[0][5])
        check("7  at the end the door fills, of the window's 960 dots", f"{h:.0f} ({h/960:.2f})", 1/3 <= h / 960 <= 2/3, "between a third and two thirds")
    else:
        check("7  the selected door is on the map", "no line", False, "a line")
    # taken over
    fl2 = log[m["flight2"]:m["taken"]]
    a0 = fl2[0]; end1 = vf("e", "landed1")
    way = tuple(end1["at"][i] - a0["at"][i] for i in range(3)); n = math.sqrt(sum(x * x for x in way)) or 1
    prog = [sum((x["at"][i] - a0["at"][i]) * way[i] / n for i in range(3)) for x in fl2]
    back = [i for i in range(1, len(prog)) if prog[i] < prog[i-1] - 0.01]
    steps = [dist(a["eye"], b["eye"]) for a, b in zip(fl2, fl2[1:])]
    check("7  a flight taken over half way: frames in which the view moved back", f"{back} of {len(prog)} frames", not back, "none")
    flying = [x["flying"] for x in fl2]
    tk = vf("e", "taken"); stopped_at = max(i for i, f in enumerate(flying) if f) if any(flying) else 0
    check("7  taken over before it landed (frames in the air, of its frames)", f"{sum(flying)} of {len(flying)}; ends at {tk['d']:.1f} ft, the full flight at {end1['d']:.1f}", sum(flying) >= 3 and tk["d"] > end1["d"] * 1.05, "stopped short of the end")
    biggest = max(steps) if steps else 0; jump = steps[-1] if steps else 0
    # 8 F
    fd, fh = vf("e", "fdoor"), vf("e", "fhouse")
    sel = mapline("e", "fdoor", "selected")
    if sel:
        h = float(sel[0][5])
        check("8  F with the door selected: the door fills, of 960", f"{h:.0f} ({h/960:.2f})", 1/3 <= h / 960 <= 2/3, "between a third and two thirds")
    hm = vf("e", "home")
    check("8  F with nothing selected: the whole house, as it was framed (ft away; turn)", f"{fh['d']:.1f}, {fh['turn']:.1f} against {hm['d']:.1f}, {hm['turn']:.1f}", abs(fh["d"] - hm["d"]) < 1.0 and abs(wrap(fh["turn"] - hm["turn"])) < 1.0, "the same")

if "f" in only:
    print("  session f")
    hm, no, qu, cl, dn, tn, hk = (vf("f", n) for n in ("home", "north", "quarter", "clicked", "down", "turned", "home_key"))
    check("9  clicked once: north-up, degrees", f"{wrap(no['turn']):.2f}", abs(wrap(no["turn"])) < 1.0, "within a degree")
    ah = mapline("f", "home", "arrow")
    across = float(ah[0][3]) if ah else 0.0
    ax = [float(x.split("\t")[2]) for x in open(f"{td}/m-f-home.txt") if x.startswith("model\tNorth arrow\t")]
    check("9  the north arrow's width in dots, and its left edge against the left panel's right edge (262)", f"{across:.0f}; {ax[0] - across / 2:.0f}" if ax else "none", across >= 44 and bool(ax) and ax[0] - across / 2 >= 262, "44 or more, clear of the panel")
    arrow = mapline("f", "quarter", "arrow")
    needle = float(arrow[0][1]) if arrow else -1
    check("9  after a quarter turn of the view the needle is a quarter turn off, degrees (view turn, needle)", f"{qu['turn']:.1f}, {needle:.1f}", abs(abs(wrap(qu["turn"])) - 90) < 15 and abs(((needle - qu['turn']) + 180) % 360 - 180) < 3.0, "a quarter, and the same as the view's turn")
    check("9  a click and the view is north-up within a degree, standing where it stood (ft moved, ft nearer, tilt change)", f"{wrap(cl['turn']):.2f}; {dist(cl['at'], qu['at']):.3f}; {abs(cl['d'] - qu['d']):.3f}; {abs(cl['tilt'] - qu['tilt']):.2f}", abs(wrap(cl["turn"])) < 1.0 and dist(cl["at"], qu["at"]) < 0.1 and abs(cl["d"] - qu["d"]) < 0.1, "north-up, same place")
    check("9  a double-click and it looks straight down, degrees", f"{dn['tilt']:.1f}", dn["tilt"] >= 85.0, "85 or more")
    check("9  Home: north-up from where it stands, degrees", f"{wrap(hk['turn']):.2f}, from {tn['turn']:.1f}", abs(wrap(hk["turn"])) < 1.0 and abs(wrap(tn["turn"])) > 3.0, "north-up")
if "h" in only:
    print("  session h")
    wl, gr = vf("h", "wall"), vf("h", "ground")
    check("1  in on the front wall's face: how near the eye is to what it looks at, and which side of the wall it stays on (the wall's face lies at 14.25 ft south)", f"{wl['d']:.3f} ft; eye {wl['eye'][2]:.2f} ft south, {wl['eye'][1]:.2f} up", wl["d"] < 0.7 and wl["eye"][2] > 14.2 and wl["eye"][1] > 0.1, "outside the wall, over the ground, 8 in from the place")
    check("1  in on bare ground: how near, and how high the eye stays over it", f"{gr['d']:.3f} ft; eye {gr['eye'][1]:.2f} up", gr["d"] < 0.7 and gr["eye"][1] > 0.1, "over the ground")

print()
if fails:
    print("FAIL: " + "; ".join(fails)); sys.exit(1)
print("all the figures read are as written")
PY
[ "$(sha256sum "$J")" = "$BEFORE" ] || { echo "FAIL: looking round the model changed the job"; exit 1; }
echo "    the job is unchanged: nothing was selected, moved or let go of by any of it, and Save (F2) wrote nothing new"
[ "$RC" = 0 ] || exit 1

echo
echo "== The strips, for a person to look at =="
python3 -I - "$OUT" <<'PY'
import sys, glob, os
from PIL import Image, ImageDraw
out = sys.argv[1]
def strip(name, pics, cols=4, w=400):
    pics = [p for p in pics if os.path.exists(p)]
    if not pics: print("    no pictures for", name); return
    h = int(w * 0.6); rows = (len(pics) + cols - 1) // cols
    sheet = Image.new("RGB", (cols * w, rows * h), (20, 20, 24))
    for i, p in enumerate(pics):
        sheet.paste(Image.open(p).convert("RGB").resize((w, h)), ((i % cols) * w, (i // cols) * h))
    sheet.save(f"{out}/{name}"); print(f"    {name}: {len(pics)} pictures")
strip("strip-dive.png", sorted(glob.glob(f"{out}/a/*-model.png"))[1:])
strip("strip-dive-tilt-off.png", sorted(glob.glob(f"{out}/b/*-model.png")), cols=2)
strip("strip-flight.png", sorted(glob.glob(f"{out}/e/*-model.png"))[:9], cols=3)
PY
echo
echo "PASS: the wheel dives from the lot to a receptacle with the place under the pointer, the view leans as it comes down and rights itself, a pan holds the ground, Shift and the left button turns, a flight frames what it flies to, F frames, the north arrow turns and is clicked, Page Up and Page Down glide, and the saved job is as it was"
