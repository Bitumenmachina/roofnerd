#!/usr/bin/env bash
# A plan in the second window (DECISIONS.md call 222, part C, and call 224; STATUS.md "A plan in the second window:
# a demonstration"; contract handoff/contracts/contract-222WC.md). The job is the made-up whole house (three levels:
# foundation at -4 ft, first floor at 0, second floor at 10 ft; price $468,050.22), with the made-up plan
# (demos/plans/made-up-plan.pdf, 30 ft by 20 ft) added to a copy in step 6.
#
# The program writes what the second window shows to the map (PLUMBLINE_MAP): `second<TAB>plan`, for each plan
# `half<TAB>n<TAB>level First floor<TAB>east<TAB>north<TAB>dots a foot<TAB>x<TAB>y<TAB>w<TAB>h` (where its middle is on the
# ground, and where it stands on the screen), `marked<TAB>n<TAB>path` (what is marked in it; `marked<TAB>model<TAB>path` is what
# the model window has selected), `shown<TAB>n<TAB>count`, `catch<TAB>n<TAB>words`, `tabs<TAB>how many<TAB>lines it should stand
# on`; and each frame a plan's view moves, a line to `<map>.halfview`. The checks read those, the display's own account of the
# windows (xdotool), and the saved files; the pictures are looked at by a person.
#
# Sessions, each on its own virtual display and under ten minutes:
#   a  one plan: Estimate and Plan; its own wheel, drag and level; the tab and the scroll left on the estimate     (steps 1, 2, 3, 4, 12)
#   b  two plans: side by side; each its own wheel; layers; the catches; looking, not drawing                    (steps 5, 8, 9, 10)
#   c  selected in one, marked in the other; a double-click flies both; the price                                 (steps 7, 11)
#   g  the architect's sheet beside the tracing; the job as saved with and without the second window                (steps 6, 11)
#   d  kept: two plans, closed, opened again                                                                       (step 13)
#   e  the tabs of the estimate at 1,366 and at each step of lettering; the price bar at 720; the first two seconds (step 14 a, b, c)
# Step 14 d (under a window manager) is session h of demos/screensw.sh.
# Run from the repository root:  [PLUMBLINE_BIN=<folder>] [ONLY="a b c d e g"] [STATE=<folder>] demos/twoplansw.sh [pictures]
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/twoplansw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/twoplansw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/twoplansw}"; mkdir -p "$OUT"
ONLY="${ONLY:-a b c g d e}"
if [ -n "${STATE:-}" ]; then T="$STATE"; mkdir -p "$T"; else T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT; fi
wants() { case " $ONLY " in *" $1 "*) return 0;; *) return 1;; esac; }
J="$T/house.json"; [ -s "$J" ] || "$P" sixteen "$J" >/dev/null
cp demos/plans/made-up-plan.pdf "$T/"

# What every session can do beside the kit of window.sh.
cat > "$T/kit.sh" <<'KIT'
# A map row's field: kind, key, column (column 3 is the first after the key).
mrow() { awk -F'\t' -v a="$1" -v b="$2" '$1==a && $2==b' "$PLUMBLINE_MAP" | head -1; }
mcol() { mrow "$1" "$2" | cut -f"$3"; }
# A plan's view: east north dots-a-foot x y w h (the map's own numbers).
half() { mrow half "$1" | cut -f4-10 | tr '\t' ' '; }
# Where a place on the ground is on the screen in plan n, from the view as the map says it now: x y.
px() { local e n d x y w h E N; read -r E N d x y w h <<<"$(half "$1")"; awk -v e="$2" -v nn="$3" -v E="$E" -v N="$N" -v d="$d" -v x="$x" -v y="$y" -v w="$w" -v h="$h" 'BEGIN { printf "%d %d", x + w / 2 + (e - E) * d + 0.5, y + h / 2 - (nn - N) * d + 0.5 }'; }
# The view is at rest: two reads of it three seconds apart agree.
rest() { local a b i; for i in 1 2 3 4 5 6 7 8; do a="$(half "$1")"; sleep 3; b="$(half "$1")"; [ "$a" = "$b" ] && return 0; done; echo "plan $1 did not come to rest" >> "$T/missing.txt"; close_window; exit 1; }
# A press, a drag by dx dots to the right and held still, and let go.
drag_right() { xdotool mousemove "$1" "$2"; sleep 1.5; xdotool mousedown 1; sleep 1.5; local i; for i in 1 2 3 4; do xdotool mousemove_relative -- "$(( $3 / 4 ))" 0; sleep 2; done; sleep 4; xdotool mouseup 1; sleep 4; }
# Notches of the wheel over a place: $3 notches, up (nearer) or down.
wheel_at() { xdotool mousemove "$1" "$2"; sleep 2; local i; for i in $(seq 1 "$3"); do xdotool click "${4:-4}"; sleep 3; done; sleep 4; }
# A double-click as a hand makes one: the two presses 0.4 s apart (inside the half second every double-click is told by, at a fast frame as at a slow one).
double_click() { xdotool mousemove "$1" "$2"; sleep 2; xdotool mousedown 1; sleep 0.3; xdotool mouseup 1; sleep 0.1; xdotool mousedown 1; sleep 0.3; xdotool mouseup 1; sleep 8; }
note() { echo "$2" > "$T/n-$NAME-$1.txt"; }
keep_map() { cp "$PLUMBLINE_MAP" "$T/m-$NAME-$1.txt"; [ -e "$PLUMBLINE_MAP.view" ] && cp "$PLUMBLINE_MAP.view" "$T/v-$NAME-$1.txt" || : > "$T/v-$NAME-$1.txt"; }
settle() { sleep "${1:-8}"; }
# What a window is called by the display.
wname() { xdotool getwindowname "$(xdotool search --name "BuildOS: $1" | head -1)"; }
KIT

session() {   # $1 name, $2 screen WxH; the steps are in $T/steps-$1.sh
  rm -f "$T/map.txt" "$T/map.txt.said" "$T/map.txt.view" "$T/map.txt.halfview" "$T/missing.txt"
  cat > "$T/session-$1.sh" <<WRAP
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT/$1" ICD="$ICD" NAME="$1" JOB="$J" P="$P" REPO="$PWD"
mkdir -p "\$OUT"; rm -f "\$OUT"/*.png
. "$PWD/demos/window.sh"
. "$T/kit.sh"
. "$T/steps-$1.sh"
touch "$T/end-$1"
WRAP
  chmod +x "$T/session-$1.sh"; rm -f "$T/end-$1"
  timeout 590 xvfb-run -a -s "-screen 0 ${2}x24" "$T/session-$1.sh" || true
  if [ -s "$T/missing.txt" ]; then echo "FAIL: session $1 stopped: a control it was to click, or a view it was to read, was not there"; cat "$T/missing.txt"; exit 1; fi
  if grep -q "panicked" "$T/app.log" 2>/dev/null; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
  [ -e "$T/end-$1" ] || { echo "FAIL: session $1 did not reach its end"; exit 1; }
  echo "  session $1 done"
}

# --- a: one plan (steps 1, 2, 3, 4, 12)
cat > "$T/steps-a.sh" <<'STEPS'
cp "$JOB" "$T/a-before.json"
open_window "$JOB"
note title-estimate "$(wname sheet)"
# the estimate, on its Roof tab, its list scrolled a little: to be found as it was left (step 12)
sheet "Roof"; sleep 4
xdotool mousemove 1900 600; sleep 2; for i in 1 2 3; do xdotool click 5; sleep 3; done; sleep 4
keep_map estimate-before; park; shot
cp "$PLUMBLINE_MAP.view" "$T/v-a-0.txt" 2>/dev/null || : > "$T/v-a-0.txt"
# 1. Plan
sheet "Window Plan"; sleep 10; rest 1; park
keep_map plan; shot
note title-plan "$(wname sheet)"
# 2. its own wheel: three clicks of the wheel (two notches each on this display, as navw.sh found: six notches) over a corner of the house (its
#    south-west corner is the ground's 0, 0)
read -r CX CY <<<"$(px 1 0 0)"; note corner "$CX $CY"
D0="$(half 1 | cut -d' ' -f3)"
V0="$(wc -c < "$PLUMBLINE_MAP.view" 2>/dev/null || echo 0)"
wheel_at "$CX" "$CY" 3; rest 1
D1="$(half 1 | cut -d' ' -f3)"; read -r AX AY <<<"$(px 1 0 0)"
note wheel "$D0 $D1 $CX $CY $AX $AY"
V1="$(wc -c < "$PLUMBLINE_MAP.view" 2>/dev/null || echo 0)"; note modelview "$V0 $V1"
keep_map wheel; shot
# 3. its own drag: 200 dots to the right
drag_right "$AX" "$AY" 200; rest 1
read -r BX BY <<<"$(px 1 0 0)"; note drag "$AX $AY $BX $BY"
keep_map drag; shot
# 4. its own level: second floor in this window; the model window's level is what it was
S0="$(grep -E '^state	Level' "$PLUMBLINE_MAP" | head -1 || true)"
sheet "Plan 1 shows"; sleep 3
at="$(found sheet "Plan 1 shows Second floor")" || exit 1; click $at; sleep 8; rest 1
note level "$(mcol half 1 3)|$S0|$(grep -E '^state	Level' "$PLUMBLINE_MAP" | head -1 || true)"
note title-level "$(wname sheet)"
keep_map level; park; shot
# 12. back to Estimate: the tab that was open is open, the list where it was scrolled to
sheet "Window Estimate"; sleep 12
keep_map estimate-after; park; shot
close_window
STEPS

# --- b: two plans (steps 5, 8, 9, 10)
cat > "$T/steps-b.sh" <<'STEPS'
cp "$JOB" "$T/b-before.json"
open_window "$JOB"
# what a frame of the model window costs with the second window on the estimate: the view's own lines (one a frame while it moves)
wheel_at 800 500 4 4; sleep 3; wheel_at 800 500 4 5; sleep 4
V0="$(wc -l < "$PLUMBLINE_MAP.view" 2>/dev/null || echo 0)"; note frames-base "$V0"
sheet "Window Plan"; sleep 10
sheet "Two plans"; sleep 10; rest 1; rest 2; park
# and with two plans open
wheel_at 800 500 4 4; sleep 3; wheel_at 800 500 4 5; sleep 4
cp "$PLUMBLINE_MAP.view" "$T/b-view.txt"; park
keep_map two; shot
# 5. each its own wheel
read -r LX LY <<<"$(px 1 0 0)"; read -r RX RY <<<"$(px 2 0 0)"
L0="$(half 1)"; R0="$(half 2)"
wheel_at "$LX" "$LY" 3; rest 1; rest 2
L1="$(half 1)"; R1="$(half 2)"
keep_map left-wheel; shot
wheel_at "$RX" "$RY" 3; rest 1; rest 2
L2="$(half 1)"; R2="$(half 2)"
note wheels "$L0|$R0|$L1|$R1|$L2|$R2"
keep_map right-wheel; shot
# 8. its own layers: Plumbing off in the right plan only
sheet "Plan 2 layers"; sleep 3; keep_map layers-open; shot
S2="$(mcol shown 2 3)"; S1="$(mcol shown 1 3)"; SM="$(mcol shown model 3)"
sheet "Plan 2 Plumbing"; sleep 10
sheet "Plan 2 layers"; sleep 3; park
note layers "$S1 $S2 $SM $(mcol shown 1 3) $(mcol shown 2 3) $(mcol shown model 3)|$(grep -cE '^text	model	94	' "$PLUMBLINE_MAP" || true)"
keep_map layers-off; shot
# 9. the catches: the pointer brought to the end of a wall in the left plan
read -r EX EY <<<"$(px 1 0 0)"
xdotool mousemove "$EX" "$EY"; sleep 6; xdotool mousemove_relative 1 0; sleep 6
note catch "$(mcol catch 1 3)"
keep_map catch; shot
# 10. looking, not drawing: the Wall tool in hand, a click in the right plan draws nothing and says where to draw
tool "Wall"; sleep 3
read -r WX WY <<<"$(px 2 10 14)"
click "$WX" "$WY"; sleep 4
xdotool key F2; sleep 4
note said "$(tail -3 "$PLUMBLINE_MAP.said" | tr '\n' '|')"
cp "$JOB" "$T/b-after.json"
keep_map tool; shot
esc
close_window
STEPS

# --- c: selected in one, marked in the other; the flight; the price (steps 7, 11)
cat > "$T/steps-c.sh" <<'STEPS'
open_window "$JOB"
sheet "Window Plan"; sleep 10
sheet "Two plans"; sleep 10; rest 1; rest 2
note price-model "$(grep -cE '^text	model	\$468,050.22	' "$PLUMBLINE_MAP" || true)"
# 7. a wall clicked in the model window (on its plan) is marked in the second window's plan of that level
on "Plan"; sleep 12
xdotool key Escape; sleep 2
at_plan() { local x0 y0 k; read -r x0 y0 k <<<"$(plan_line)"; [ -n "$x0" ] || { echo "the model window's plan is not at rest" >> "$T/missing.txt"; close_window; exit 1; }; click $(plan_px "$1" "$2"); }
at_plan 13 0; sleep 6; park
note model-click "$(mcol marked model 3)|$(mcol marked 1 3)|$(mcol marked 2 3)"
keep_map model-click; shot
esc; sleep 4
# ... and a wall clicked in the second window's plan is the model window's selection
read -r WX WY <<<"$(px 1 13 0)"; click "$WX" "$WY"; sleep 6; park
note second-click "$(mcol marked model 3)|$(mcol marked 1 3)|$(mcol marked 2 3)"
keep_map second-click; shot
# a double-click on a window in the second window's plan flies both to it
M0="$(grep -E '^selected	' "$PLUMBLINE_MAP" | head -1 || true)"
read -r WX WY <<<"$(px 1 7.5 0)"; double_click "$WX" "$WY"; rest 1; sleep 10; park
note flight "$(mcol marked model 3)|$(grep -E '^selected	' "$PLUMBLINE_MAP" | head -1 || true)|$(half 1)|$WX $WY|$(grep -E '^window	model' "$PLUMBLINE_MAP")"
keep_map flight; shot
# 11. the price, in both windows
note price-after "$(grep -cE '^text	model	\$468,050.22	' "$PLUMBLINE_MAP" || true) $(grep -cE '^text	sheet	\$468,050.22	' "$PLUMBLINE_MAP" || true)"
sheet "Window Estimate"; sleep 12
note price-estimate "$(grep -cE '^text	sheet	\$468,050.22	' "$PLUMBLINE_MAP" || true)"
keep_map estimate; shot
close_window
STEPS

# --- g: the architect's sheet beside the tracing (step 6), and the job as saved (step 11)
cat > "$T/steps-g.sh" <<'STEPS'
# the same drawing added with the second window never opened, and with it opened and used: the two jobs are the same, byte for byte
cp "$JOB" "$T/g1.json"
JOB="$T/g1.json"; open_window "$JOB"
on "Add drawing"; sleep 2; typed "$T/made-up-plan.pdf"; sleep 8
xdotool key F2; sleep 5; close_window
sleep 2
cp "$T/g1.json" "$T/g1-saved.json"
cp "$T/house.json" "$T/g2.json"
JOB="$T/g2.json"; open_window "$JOB"
on "Add drawing"; sleep 2; typed "$T/made-up-plan.pdf"; sleep 8
sheet "Window Plan"; sleep 10
sheet "Two plans"; sleep 10
sheet "Plan 1 shows"; sleep 3
at="$(found sheet "Plan 1 shows made-up-plan")" || exit 1; click $at; sleep 10; rest 1; rest 2; park
note sheet-beside "$(half 1)|$(half 2)"
note sheet-names "$(mcol half 1 3)|$(mcol half 2 3)"
keep_map sheet; shot
xdotool key F2; sleep 5
cp "$T/g2.json" "$T/g2-saved.json"
close_window
STEPS

# --- d: kept (step 13)
cat > "$T/steps-d.sh" <<'STEPS'
export PLUMBLINE_SETTINGS="$T/settings-d.txt"; rm -f "$PLUMBLINE_SETTINGS"
open_window "$JOB"
sheet "Window Plan"; sleep 10
sheet "Two plans"; sleep 10; rest 1; rest 2
sheet "Plan 2 shows"; sleep 3
at="$(found sheet "Plan 2 shows First floor")" || exit 1; click $at; sleep 8
sheet "Plan 2 layers"; sleep 3
sheet "Plan 2 Plumbing"; sleep 8
park; keep_map set; shot
sleep 4; cp "$PLUMBLINE_SETTINGS" "$T/settings-d-written.txt"
close_window
open_window "$JOB"; sleep 6; rest 1; rest 2; park
keep_map again; shot
close_window
STEPS

# --- e: the tabs, the price bar at 720, the first two seconds (step 14 a, b, c)
cat > "$T/steps-e.sh" <<'STEPS'
export PLUMBLINE_SETTINGS="$T/settings-e.txt"; rm -f "$PLUMBLINE_SETTINGS"
# c. the first two seconds on a screen 1,366 wide: whatever window is seen is no wider than the screen
"$APP" "$JOB" > "$T/app.log" 2>&1 &
APP_PID=$!
: > "$T/first-seconds.txt"
for i in $(seq 1 120); do
  for w in model sheet; do
    id="$(xdotool search --onlyvisible --name "BuildOS: $w" 2>/dev/null | head -1 || true)"
    if [ -n "$id" ]; then eval "$(xdotool getwindowgeometry --shell "$id")"; echo "$i $w $X $Y $WIDTH $HEIGHT" >> "$T/first-seconds.txt"; fi
  done
  sleep 0.25
done
sleep 20; focus
keep_map open; park; shot
mv "$OUT/01-sheet.png" "$OUT/open-sheet.png"; mv "$OUT/01-model.png" "$OUT/open-model.png"
close_window
# a. the tabs, at each step of lettering (the estimate stands over the model's lower right at 1,366, so the program is opened at each step as the settings keep it)
for step in 0 1 2 3; do
  printf 'lettering = %s\n' "$step" > "$PLUMBLINE_SETTINGS"
  open_window "$JOB"; sleep 6; park; keep_map "tabs$step"; if [ "$step" = 0 ] || [ "$step" = 3 ]; then shot; mv "$OUT/01-sheet.png" "$OUT/tabs$step-sheet.png"; mv "$OUT/01-model.png" "$OUT/tabs$step-model.png"; fi
  close_window
done
printf 'lettering = 0\n' > "$PLUMBLINE_SETTINGS"
open_window "$JOB"
# b. the model window 720 wide: the price bar
M="$(xdotool search --name 'BuildOS: model' | head -1)"
xdotool windowsize "$M" 720 700; sleep 14; park; keep_map narrow; shot; mv "$OUT/01-sheet.png" "$OUT/narrow-sheet.png"; mv "$OUT/01-model.png" "$OUT/narrow-model.png"
close_window
STEPS

wants a && session a 2560x1440
wants b && session b 2560x1440
wants c && session c 2560x1440
wants g && session g 2560x1440
wants d && session d 2560x1440
wants e && session e 1366x768

echo "== the checks =="
python3 -I - "$T" "$OUT" "$ONLY" <<'PY'
import sys, os, re, hashlib, glob
T, OUT, ONLY = sys.argv[1], sys.argv[2], sys.argv[3].split()
bad = []
def ok(c, msg):
    print(("  ok    " if c else "  FAIL  ") + msg)
    if not c: bad.append(msg)
def note(name, key):
    try: return open(f"{T}/n-{name}-{key}.txt").read().rstrip("\n")
    except OSError: return None
def nums(s): return [float(x) for x in re.findall(r"-?\d+(?:\.\d+)?", s or "")]
def halfline(s):
    f = (s or "").split()
    return (float(f[0]), float(f[1]), float(f[2]), float(f[3]), float(f[4]), float(f[5]), float(f[6])) if len(f) >= 7 else None
def rows(path, kind):
    try: return [l.split("\t") for l in open(path).read().split("\n") if l.startswith(kind + "\t")]
    except OSError: return []
def sha(p): return hashlib.sha256(open(p, "rb").read()).hexdigest()
NOTCH = 0.9
if "a" in ONLY:
    ok(note("a", "title-estimate") == "BuildOS: sheet", f"1: the second window opens on the estimate, titled as it always was: {note('a','title-estimate')}")
    p = rows(f"{T}/m-a-plan.txt", "half"); sec = rows(f"{T}/m-a-plan.txt", "second")
    ok(sec and sec[0][1] == "plan", f"1: the second window says it shows a plan: {sec}")
    ok(p and p[0][2] == "level First floor", f"1: it shows the first floor: {p[0][2] if p else None}")
    dpf = float(p[0][5]) if p else 0
    ok(15 <= dpf <= 20, f"1: the house (40 ft by 28 ft) is whole in 880 dots at {dpf} dots a foot (15 to 20)")
    ok("plan" in (note("a", "title-plan") or "") and "First floor" in (note("a", "title-plan") or ""), f"1: the window's title says what it shows: {note('a','title-plan')}")
    w = nums(note("a", "wheel"))
    if len(w) >= 6:
        d0, d1, cx, cy, ax, ay = w[:6]; want = (1 / NOTCH) ** 6
        ok(abs(d1 / d0 - want) < 0.03 * want, f"2: six notches bring the plan nearer by {d1/d0:.3f} (a notch takes a tenth off the distance, so the plan grows by 1 / 0.9 a notch: to the sixth {want:.3f}; the 1.77 written in STATUS.md is 1.1 to the sixth, which is not what a notch does)")
        ok(abs(ax - cx) <= 2 and abs(ay - cy) <= 2, f"2: the corner stays under the pointer: ({cx:.0f}, {cy:.0f}) then ({ax:.0f}, {ay:.0f})")
    v = nums(note("a", "modelview"))
    ok(len(v) == 2 and v[0] == v[1], f"2: the model window's view has not moved (its view file {v})")
    d = nums(note("a", "drag"))
    if len(d) >= 4: ok(abs((d[2] - d[0]) - 200) <= 2 and abs(d[3] - d[1]) <= 2, f"3: a drag of 200 dots carries the corner by ({d[2]-d[0]:.0f}, {d[3]-d[1]:.0f})")
    lv = (note("a", "level") or "").split("|")
    ok(len(lv) == 3 and lv[0] == "level Second floor", f"4: set to the second floor, the plan shows it: {lv[0] if lv else None}")
    ok(len(lv) == 3 and lv[1] == lv[2], f"4: the model window's level is what it was: {lv[1:] }")
    ok("Second floor" in (note("a", "title-level") or ""), f"4: the title follows: {note('a','title-level')}")
    # 12. the tab and the scroll
    b = {(r[2]): int(float(r[4])) for r in rows(f"{T}/m-a-estimate-before.txt", "text") if r[1] == "sheet"}
    a = {(r[2]): int(float(r[4])) for r in rows(f"{T}/m-a-estimate-after.txt", "text") if r[1] == "sheet"}
    same = [k for k in b if k in a and k not in ("Opened.",)]
    moved = [k for k in same if abs(a[k] - b[k]) > 2]
    ok(len(same) > 20 and not moved, f"12: back on the estimate, the open tab and the list stand as left ({len(same)} words compared, {len(moved)} moved: {moved[:3]})")
    ok(any(r[2].startswith("plane ") for r in rows(f"{T}/m-a-estimate-after.txt", "text")), "12: the Roof tab is the one open (its planes are listed)")
if "b" in ONLY:
    try:
        vl = [l.split("\t") for l in open(f"{T}/b-view.txt").read().split("\n") if l.startswith("view\t")]
        base = int(note("b", "frames-base") or 0)
        def dts(ls): return [float(b[8]) - float(a[8]) for a, b in zip(ls, ls[1:]) if 0 < float(b[8]) - float(a[8]) < 5]
        d0, d1 = dts(vl[:base]), dts(vl[base:])
        if d0 and d1:
            ok(True, f"frame time of the model window while its view moves: {sum(d0)/len(d0)*1000:.0f} ms a frame with the second window on the estimate ({len(d0)} frames), {sum(d1)/len(d1)*1000:.0f} ms with two plans open ({len(d1)} frames)")
        else:
            ok(False, f"the view's lines were read for frame times ({len(vl)} lines, base {base})")
    except OSError:
        ok(False, "the view file was kept")
    p = rows(f"{T}/m-b-two.txt", "half")
    ok(len(p) == 2 and p[0][2] == "level First floor" and p[1][2] == "level Second floor", f"5: two plans, the first floor beside the second: {[r[2] for r in p]}")
    if len(p) == 2:
        for r in p:
            dpf, w, h = float(r[5]), float(r[8]), float(r[9])
            ok(40 * dpf <= w and 28 * dpf <= h, f"5: plan {r[1]} is whole in its half ({w:.0f} by {h:.0f}) at {dpf} dots a foot")
        ok(float(p[1][6]) - (float(p[0][6]) + float(p[0][8])) >= 2, "5: a line stands between them")
    w = (note("b", "wheels") or "").split("|")
    if len(w) == 6:
        L0, R0, L1, R1, L2, R2 = [halfline(x) for x in w]
        ok(L1[2] > L0[2] * 1.2 and R1 == R0, f"5: the wheel over the left moves the left only: {L0[2]} -> {L1[2]}; the right {R0[2]} -> {R1[2]}")
        ok(R2[2] > R1[2] * 1.2 and L2 == L1, f"5: and the other way about: the right {R1[2]} -> {R2[2]}; the left {L1[2]} -> {L2[2]}")
    ly = (note("b", "layers") or "").split("|")
    if len(ly) == 2:
        n = [int(x) for x in ly[0].split()]
        ok(n[4] < n[1] and n[3] == n[0] and n[5] == n[2], f"8: Plumbing off in the second window: its count {n[1]} -> {n[4]}, the first's {n[0]} -> {n[3]}, the model's {n[2]} -> {n[5]}")
        ok(int(ly[1] or 0) >= 1, "8: the model window's Plumbing switch still counts its 94")
    c = note("b", "catch") or ""
    ok(any(w in c for w in ("On an end", "Where two cross")), f"9: the pointer at the end of a wall names what it caught: {c!r}")
    s = note("b", "said") or ""
    ok("Draw in the model window. This window is for looking." in s, f"10: with the Wall tool in hand, a click in the plan says where to draw: {s!r}")
    ok(sha(f"{T}/b-before.json") == sha(f"{T}/house.json") and os.path.exists(f"{T}/b-after.json") and sha(f"{T}/b-after.json") == sha(f"{T}/b-before.json"), "10: and nothing was drawn: the job is byte for byte as it was")
if "c" in ONLY:
    ok(int(note("c", "price-model") or 0) >= 1, f"11: the price reads $468,050.22 in the model window (the bar and the Job panel): {note('c','price-model')}")
    mc = (note("c", "model-click") or "").split("|")
    ok(len(mc) == 3 and mc[0] != "none" and mc[1] == mc[0], f"7: a wall clicked in the model window is marked in the first-floor plan: {mc}")
    ok(len(mc) == 3 and mc[2] == "none", f"7: and not in the second-floor plan (another wall there): {mc[2:]}")
    sc = (note("c", "second-click") or "").split("|")
    ok(len(sc) == 3 and sc[0] != "none" and sc[0] == sc[1], f"7: a wall clicked in the second window is the model window's selection and marked there: {sc}")
    fl = (note("c", "flight") or "").split("|")
    if len(fl) >= 5:
        ok(fl[0] not in ("none", ""), f"7: the double-click selected {fl[0]}")
        hl = halfline(fl[2]); click = nums(fl[3]); win = nums(fl[4])
        if hl:
            cx, cy = hl[3] + hl[5] / 2, hl[4] + hl[6] / 2
            # the window's middle is the plan's centre: its ground is (7.5, 0.0) on the south wall
            e, n = hl[0], hl[1]
            ok(abs(e - 7.5) * hl[2] <= 20 and abs(n - 0.0) * hl[2] <= 20, f"7: the plan ends with the window within 20 dots of the middle of its half: ground ({e:.2f}, {n:.2f}) is {abs(e-7.5)*hl[2]:.0f} east-west and {abs(n)*hl[2]:.0f} north-south dots off")
        sel = fl[1].split("\t")
        if len(sel) >= 6 and len(win) >= 4:
            wh = win[3]
            side = max(float(sel[4]), float(sel[5]))
            ok(wh / 3 <= side <= 2 * wh / 3 + 0.05 * wh, f"7: the model window ends with it between a third and two thirds of its view: {side:.0f} dots of {wh:.0f}")
    pa = nums(note("c", "price-after"))
    ok(pa and pa[0] >= 1, f"11: the price still reads $468,050.22 in the model window after all of it: {pa}")
    ok(note("c", "price-estimate") == "1", f"11: and in the second window on the estimate: {note('c','price-estimate')}")
if "g" in ONLY:
    names = (note("g", "sheet-names") or "").split("|")
    ok(len(names) == 2 and names[0].startswith("drawing ") and names[1] == "level First floor", f"6: the left half is the architect's sheet and the right the first floor: {names}")
    l, r = [halfline(x) for x in (note("g", "sheet-beside") or "|").split("|")][:2] if (note("g", "sheet-beside") or "").count("|") == 1 else (None, None)
    if l and r:
        ok(30 * l[2] <= l[5] and 20 * l[2] <= l[6], f"6: the sheet (30 ft by 20 ft) is whole in the left half at {l[2]} dots a foot")
        ok(40 * r[2] <= r[5] and 28 * r[2] <= r[6], f"6: and the floor whole in the right half at {r[2]} dots a foot")
    a, b = f"{T}/g1-saved.json", f"{T}/g2-saved.json"
    ok(os.path.exists(a) and os.path.exists(b) and sha(a) == sha(b), "11: the job saved after all of this is byte for byte the job saved with the drawing added and the second window never opened")
    ok(sha(f"{T}/house.json") != sha(a) if os.path.exists(a) else False, "11: (and the drawing is in it: it is not the job as it began)")
if "d" in ONLY:
    st = open(f"{T}/settings-d-written.txt").read() if os.path.exists(f"{T}/settings-d-written.txt") else ""
    print(st)
    ok("second window = two plans" in st and "plan 1 shows = level" in st and "plan 2 shows = level" in st, "13: the settings name two plans and what each shows")
    ok("plan 2 layers away = Plumbing" in st, "13: and the right plan's Plumbing put away")
    a, b = rows(f"{T}/m-d-set.txt", "half"), rows(f"{T}/m-d-again.txt", "half")
    ok(len(b) == 2 and [r[2] for r in a] == [r[2] for r in b], f"13: opened again it shows two plans, the same two: {[r[2] for r in b]}")
    ok(len(a) == 2 and len(b) == 2 and int(rows(f"{T}/m-d-again.txt", 'shown')[1][2]) == int(rows(f"{T}/m-d-set.txt", 'shown')[1][2]), "13: with the same layers")
if "e" in ONLY:
    fs = [l.split() for l in open(f"{T}/first-seconds.txt")] if os.path.exists(f"{T}/first-seconds.txt") else []
    wide = [x for x in fs if int(x[4]) >= 1600]
    ok(not wide, f"14c: no window seen in the first thirty seconds is 1,600 wide on a screen 1,366 wide ({len(fs)} looks; wide: {wide[:3]})")
    ok(len(fs) > 0, f"14c: and windows were seen (first look at {fs[0] if fs else None})")
    for step in range(4):
        path = f"{T}/m-e-tabs{step}.txt"
        win = [r for r in rows(path, "window") if r[1] == "sheet"]
        tabs = [r for r in rows(path, "sheet") if r[1] in ("Estimate", "Budget", "Verify in field", "Clashes", "Layouts", "Building", "Roof", "Mechanical", "Electrical", "Plumbing")]
        exp = rows(path, "tabs")
        if win and tabs:
            wx, wy, ww, wh = [float(x) for x in win[0][2:6]]
            inside = [r[1] for r in tabs if wx <= float(r[2]) <= wx + ww and wy <= float(r[3]) <= wy + wh]
            ys = sorted({round(float(r[3]) / 8) for r in tabs})
            ok(len(inside) == len(tabs), f"14a: lettering step {step}: every tab ({len(tabs)}) is inside the window")
            ok(len(ys) >= 2 and (not exp or len(ys) == int(exp[0][2])), f"14a: lettering step {step}: the tabs stand on {len(ys)} lines (the program expects {exp[0][2] if exp else '?'})")
        else:
            ok(False, f"14a: lettering step {step}: the map has the tabs")
    nr = f"{T}/m-e-narrow.txt"
    t = [r for r in rows(nr, "text") if r[1] == "model"]
    words = [r[2] for r in t]
    ok(any(w == "$468,050.22" for w in words), "14b: at 720 wide the price is on the bar")
    ok("Saved" in words or "Not saved" in words, "14b: and whether the job is saved")
    ok(any(re.match(r"\d+ to verify", w) for w in words), f"14b: and the count to verify: {[w for w in words if 'verify' in w]}")
    ok(not any("known" in w or "assumed" in w or "unknown" in w for w in words if w[:1].isdigit() or w[:1] == "3"), "14b: the counts of Known, Assumed and Unknown have given way")
print()
if bad:
    print(f"{len(bad)} check(s) failed")
    sys.exit(1)
print("all checks passed")
PY
