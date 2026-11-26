#!/usr/bin/env bash
# Any screen, and work never lost (DECISIONS.md call 222, part A; STATUS.md "Any screen, and work
# never lost: a demonstration"). Three screens, a window drawn in and out, what a person set kept,
# larger lettering, and work that survives a program stopped dead. The program writes each window's
# place and size, the screen, which panels are in and why, the lines the tools are on, the size of
# every control and the glass picture's size to the map; the checks read those, and the window
# geometry is also asked of the display itself (xdotool), which does not take the program's word.
#
# Sessions, each on its own virtual display and under ten minutes:
#   a  screen 2560x1440: side by side; then the model drawn in from 1600 to 1000 and out again (step 4)
#   b  screen 1920x1080                                                                    (step 2)
#   c  screen 1366x768: every control inside its window                                    (step 3)
#   d  kept: moved, resized, a panel away, the lettering a step larger; closed, opened again, opened
#      on another job, the file removed                                                    (step 5)
#   e  larger lettering: the Wall button taller by the step, nothing cut off              (step 6)
#   g  "Leave it": the job as last saved, the recovered work left beside it until Save           (step 7)
#   f  work never lost: a wall and a door, not saved; stopped dead; opened again; taken; left; saved
#                                                                                         (step 7)
#   h  the same three screens under a window manager (openbox, with tint2 for the desktop's bar: outside programs, run and never linked,
#      as the IFC checker is): each window with its title bar, measured: what the title bar and the desktop's bar take, beside the
#      72 dots allowed for them (call 224, point 5)
# Run from the repository root:  [PLUMBLINE_BIN=<folder>] [ONLY="a b c d e f"] [STATE=<folder>] demos/screensw.sh [pictures]
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/screensw.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL" 2>/dev/null || { echo "demos/screensw.sh needs python's PIL, to join the pictures"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/screensw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/screensw}"; mkdir -p "$OUT"
ONLY="${ONLY:-a b c d e f g h}"
if [ -n "${STATE:-}" ]; then T="$STATE"; mkdir -p "$T"; else T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT; fi
wants() { case " $ONLY " in *" $1 "*) return 0;; *) return 1;; esac; }
J="$T/house.json"
[ -s "$J" ] || "$P" sixteen "$J" >/dev/null
K="$T/other.json"
[ -s "$K" ] || "$P" sixteen "$K" >/dev/null

# What every session can do beside the kit of window.sh.
cat > "$T/kit.sh" <<'KIT'
mval() { awk -F'\t' -v a="$1" -v b="$2" '$1==a && $2==b {print $3, $4, $5, $6}' "$PLUMBLINE_MAP" | head -1; }
# A window's place and size as the display itself has it: x y w h.
geom() {
  local id; id="$(xdotool search --name "BuildOS: $1" | head -1)"
  eval "$(xdotool getwindowgeometry --shell "$id")"; echo "$X $Y $WIDTH $HEIGHT"
}
settle() { sleep "${1:-8}"; }
keep_map() { cp "$PLUMBLINE_MAP" "$T/m-$NAME-$1.txt"; geom model > "$T/gm-$NAME-$1.txt"; geom sheet > "$T/gs-$NAME-$1.txt"; }
# The model window drawn to a size, from its corner, the way a hand drags its edge.
resize_model() { xdotool windowsize "$(xdotool search --name 'BuildOS: model' | head -1)" "$1" "$2"; settle 10; }
KIT

session() {   # $1 name, $2 screen WxH; the steps are in $T/steps-$1.sh
  mkdir -p "$OUT"; rm -f "$T/map.txt" "$T/map.txt.said" "$T/missing.txt"
  cat > "$T/session-$1.sh" <<WRAP
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT/$1" ICD="$ICD" NAME="$1" JOB="$J" OTHER="$K" P="$P" REPO="$PWD"
mkdir -p "\$OUT"; rm -f "\$OUT"/*.png
. "$PWD/demos/window.sh"
. "$T/kit.sh"
. "$T/steps-$1.sh"
touch "$T/end-$1"
WRAP
  chmod +x "$T/session-$1.sh"; rm -f "$T/end-$1"
  timeout 580 xvfb-run -a -s "-screen 0 ${2}x24" "$T/session-$1.sh" || true
  if [ -s "$T/missing.txt" ]; then echo "FAIL: session $1 stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
  if grep -q "panicked" "$T/app.log" 2>/dev/null; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
  [ -e "$T/end-$1" ] || { echo "FAIL: session $1 did not reach its end"; exit 1; }
  echo "  session $1 done"
}

# --- a: the largest screen, then the strip of the resize
cat > "$T/steps-a.sh" <<'STEPS'
export PLUMBLINE_SETTINGS="$T/settings-a.txt"; rm -f "$PLUMBLINE_SETTINGS"
open_window "$JOB"
park; shot; keep_map open
for w in 1600 1500 1400 1300 1200 1100 1000 1100 1200 1300 1400 1500 1600; do
  resize_model "$w" 960; park; keep_map "w$w-$(date +%s)"; shot
done
close_window
STEPS
# --- b, c: the two smaller screens
for s in b c; do cat > "$T/steps-$s.sh" <<'STEPS'
export PLUMBLINE_SETTINGS="$T/settings-$NAME.txt"; rm -f "$PLUMBLINE_SETTINGS"
open_window "$JOB"
park; shot; keep_map open
close_window
STEPS
done
# --- d: kept
cat > "$T/steps-d.sh" <<'STEPS'
export PLUMBLINE_SETTINGS="$T/settings-d.txt"; rm -f "$PLUMBLINE_SETTINGS"
open_window "$JOB"
M="$(xdotool search --name 'BuildOS: model' | head -1)"; S="$(xdotool search --name 'BuildOS: sheet' | head -1)"
xdotool windowsize "$M" 1300 800; xdotool windowmove "$M" 60 40; xdotool windowmove "$S" 1500 100; xdotool windowsize "$S" 900 700; settle 8
on "Show panel Tools"; on "Larger lettering"; settle 8; park; shot; keep_map set
geom model > "$T/gm-d-set.txt"; geom sheet > "$T/gs-d-set.txt"
sleep 3; cp "$PLUMBLINE_SETTINGS" "$T/settings-d-written.txt"
close_window
open_window "$JOB"; park; shot; keep_map again
close_window
open_window "$OTHER"; park; shot; keep_map other
close_window
rm -f "$PLUMBLINE_SETTINGS"
open_window "$JOB"; park; shot; keep_map removed
close_window
STEPS
# --- e: lettering
cat > "$T/steps-e.sh" <<'STEPS'
export PLUMBLINE_SETTINGS="$T/settings-e.txt"; rm -f "$PLUMBLINE_SETTINGS"
open_window "$JOB"
park; keep_map step0; shot
on "Larger lettering"; settle 10; park; keep_map step1; shot
on "Larger lettering"; settle 10; park; keep_map step2; shot
close_window
STEPS
# --- f: work never lost
cat > "$T/steps-f.sh" <<'STEPS'
export PLUMBLINE_SETTINGS="$T/settings-f.txt"; rm -f "$PLUMBLINE_SETTINGS"
W="$T/lost.json"; rm -f "$W" "$W.recovery"; "$P" new "$W" "Work not lost (not a real job)" >/dev/null
draw() {
  open_window "$W"
  xdotool key t; drawn_for 3 4
  on "Add level"; sleep 2; typed "Ground, 0 ft, 9 ft"
  tool "Wall" "6 in, 9 ft"
  click 600 600; len 900 605 "24 ft"; esc; esc
  tool "Door" "3 ft, 7 ft, 0 ft"; click 680 600; esc
  park; sleep 3
}
draw
START=$(date +%s)
keep_map drawn; shot
# not saved: wait until a minute and ten seconds after the last change, then stop the program dead
while [ $(( $(date +%s) - START )) -lt 80 ]; do sleep 5; done
ls -l "$T"/lost.json* > "$T/f-ls-before-kill.txt" 2>&1 || true
kill -9 "$APP_PID"; wait "$APP_PID" 2>/dev/null || true
sleep 3
cp "$W.recovery" "$T/f-recovery-copy.json"; cp "$W" "$T/f-job-as-saved.json"
open_window "$W"; park; shot; keep_map offered
on "Take the recovered work"; settle 6; park; shot; keep_map taken
xdotool key F2; drawn_for 3 4; shot
cp "$W" "$T/f-taken-saved.json"; ls -l "$T"/lost.json* > "$T/f-ls-after-save.txt" 2>&1 || true
close_window
STEPS

# --- g: Leave it (needs session f's pictures of the job as saved and of the recovery copy)
cat > "$T/steps-g.sh" <<'STEPS'
export PLUMBLINE_SETTINGS="$T/settings-g.txt"; rm -f "$PLUMBLINE_SETTINGS"
W="$T/left.json"; cp "$T/f-job-as-saved.json" "$W"; sleep 2; cp "$T/f-recovery-copy.json" "$W.recovery"
open_window "$W"; park; shot; keep_map offered
on "Leave it"; settle 5; park; shot; keep_map left
xdotool key F2; drawn_for 3 4; shot
cp "$W" "$T/g-left-saved.json"; ls -l "$T"/left.json* > "$T/g-ls.txt" 2>&1 || true
close_window
STEPS

# --- h: under a window manager (call 224, point 5): each of the three screens, each window with its title bar and the desktop's bar
for s in 1 2 3; do cat > "$T/steps-h$s.sh" <<'STEPS'
export PLUMBLINE_SETTINGS="$T/settings-$NAME.txt"; rm -f "$PLUMBLINE_SETTINGS"
openbox > "$T/wm-$NAME.log" 2>&1 &
WM_PID=$!
sleep 4
tint2 > "$T/bar-$NAME.log" 2>&1 &
BAR_PID=$!
sleep 6
open_window "$JOB"
settle 10
# What the display has: each window's frame (the title bar and the edges the manager gave it), and the room the desktop leaves.
for w in model sheet; do
  id="$(xdotool search --name "BuildOS: $w" | head -1)"
  eval "$(xdotool getwindowgeometry --shell "$id")"
  xprop -id "$id" _NET_FRAME_EXTENTS > "$T/fx-$NAME-$w.txt" 2>&1 || true
  echo "$X $Y $WIDTH $HEIGHT" > "$T/gw-$NAME-$w.txt"
  # the frame itself, as the display has it (xdotool's own position for a framed window is not the frame's)
  xwininfo -frame -id "$id" | awk '/Absolute upper-left X/ {x=$4} /Absolute upper-left Y/ {y=$4} /Width:/ {w=$2} /Height:/ {h=$2} END {print x, y, w, h}' > "$T/ow-$NAME-$w.txt"
done
xprop -root _NET_WORKAREA > "$T/work-$NAME.txt" 2>&1 || true
keep_map open
park; shot
import -window root "$OUT/desktop.png" 2>/dev/null || true
close_window
kill "$BAR_PID" "$WM_PID" 2>/dev/null || true
STEPS
done

wants a && session a 2560x1440
wants b && session b 1920x1080
wants c && session c 1366x768
wants d && session d 2560x1440
wants e && session e 2560x1440
wants f && session f 2560x1440
wants g && session g 2560x1440
wants h && { session h1 2560x1440; session h2 1920x1080; session h3 1366x768; }

if wants f && [ -s "$T/f-taken-saved.json" ]; then
  # The job the command line makes by drawing the same wall and door, from the places the window's job holds.
  read -r PTS OFF < <(python3 -I - "$T/f-taken-saved.json" <<'PY2'
import json, sys
j = json.load(open(sys.argv[1])); pts = off = None
for d in j["data"]:
    a = d.get("attributes", {})
    if "pl::wall" in a: (p, q) = a["pl::wall"]["line"]["value"]; pts = "%s,%s %s,%s" % (p[0], p[1], q[0], q[1]); pts = pts.replace(" ", "@")
    if "pl::opening" in a: off = a["pl::opening"]["offset"]["value"]
print(pts, off)
PY2
)
  PTS="${PTS//@/ }"
  CW="$T/cli-made.json"; rm -f "$CW"; SRC="entered on screen; not yet checked against a drawing"
  mk() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1" | head -1; }
  "$P" new "$CW" "Work not lost (not a real job)" >/dev/null
  CL="$(mk "$("$P" level "$CW" Ground --elevation "0 ft" --height "9 ft" --status Assumed --source "$SRC")")"
  CT="$(mk "$("$P" type "$CW" wall "6 in wall" --thickness "6 in" --status Assumed --source "$SRC")")"
  CWL="$("$P" wall "$CW" "$CL" "$CT" --points "$PTS" --height "9 ft" --status Assumed --source "$SRC" | grep -oE 'wall-[0-9]+' | head -1)"
  CD="$(mk "$("$P" type "$CW" door "door 3 ft by 7 ft" --width "3 ft" --height "7 ft" --status Assumed --source "$SRC")")"
  "$P" door "$CW" "$CWL" "$CD" --offset "$OFF ft" --sill "0 ft" --status Assumed --source "$SRC" >/dev/null
fi
echo "== the checks =="
python3 -I - "$T" "$OUT" "$ONLY" <<'PY'
import sys, os, glob
T, OUT, ONLY = sys.argv[1], sys.argv[2], sys.argv[3].split()
bad = []
def ok(c, msg):
    print(("  ok    " if c else "  FAIL  ") + msg)
    if not c: bad.append(msg)
def m(path):
    d = {"window": {}, "panel": {}, "size": {}, "ctl": {}}
    try: lines = open(path).read().split("\n")
    except OSError: return d
    for l in lines:
        f = l.split("\t")
        if f[0] == "window" and len(f) >= 6: d["window"][f[1]] = tuple(int(float(x)) for x in f[2:6])
        elif f[0] == "panel": d["panel"][f[1]] = f[2]
        elif f[0] == "screen": d["screen"] = (int(f[1]), int(f[2]))
        elif f[0] == "tools": d["tools"] = (int(f[2]), int(f[3]))
        elif f[0] == "lettering": d["lettering"] = (int(f[1]), float(f[2]))
        elif f[0] == "glass": d["glass"] = tuple(int(x) for x in f[1:5])
        elif f[0] == "size" and len(f) >= 5: d["size"][(f[1], f[2])] = (int(f[3]), int(f[4]))
        elif f[0] in ("model", "sheet") and len(f) >= 4: d["ctl"][(f[0], f[1])] = (int(f[2]), int(f[3]))
    return d
def g(path):
    try: return tuple(int(x) for x in open(path).read().split())
    except (OSError, ValueError): return None
def inside(r, s): return r is not None and r[0] >= 0 and r[1] >= 0 and r[0] + r[2] <= s[0] and r[1] + r[3] <= s[1]
def over(a, b): return a[0] < b[0] + b[2] and b[0] < a[0] + a[2] and a[1] < b[1] + b[3] and b[1] < a[1] + a[3]
screens = {"a": (2560, 1440), "b": (1920, 1080), "c": (1366, 768)}
for s in "abc":
    if s not in ONLY: continue
    mm, gm, gs = m(f"{T}/m-{s}-open.txt"), g(f"{T}/gm-{s}-open.txt"), g(f"{T}/gs-{s}-open.txt")
    sc = screens[s]
    ok(mm.get("screen") == sc, f"{s}: the program reads the screen as {sc}: {mm.get('screen')}")
    ok(inside(gm, sc) and inside(gs, sc), f"{s}: both windows wholly on {sc} as the display has them: model {gm} estimate {gs}")
    if s == "a": ok(not over(gm, gs), f"{s}: side by side, not over one another")
    else: ok(gm[2] == sc[0], f"{s}: the model takes the screen's width ({gm[2]})")
    # every named control inside its window
    for win, r in (("model", mm["window"].get("model")), ("sheet", mm["window"].get("sheet"))):
        outside = [n for (w, n), (x, y) in mm["ctl"].items() if w == win and not (r[0] <= x <= r[0] + r[2] and r[1] <= y <= r[1] + r[3])]
        ok(not outside, f"{s}: every control of the {win} window is inside it ({len(outside)} outside {outside[:4]})")
if "a" in ONLY:
    steps = sorted(glob.glob(f"{T}/m-a-w*.txt"), key=os.path.getmtime)
    prev = None
    for p in steps:
        d = m(p); w = d["window"]["model"][2]
        job, det = d["panel"].get("Job"), d["panel"].get("Details")
        ok((det == "folded") == (w < 1500) and (job == "folded") == (w < 1100), f"resize {w}: Details {det}, Job {job}")
        ok("Tools" in d["panel"] and d["panel"]["Tools"] == "shown", f"resize {w}: the tools are on the bar ({d.get('tools')} lines, bar)")
        gl = d.get("glass")
        ok(gl is not None and abs(gl[0] * 6 - gl[2]) < 6 and abs(gl[1] * 6 - gl[3]) < 6, f"resize {w}: the glass picture is the window over six {gl}")
if "d" in ONLY:
    st = open(f"{T}/settings-d-written.txt").read() if os.path.exists(f"{T}/settings-d-written.txt") else ""
    print(st)
    a, b = g(f"{T}/gm-d-set.txt"), g(f"{T}/gm-d-again.txt")
    ok(a == b and a is not None, f"kept: the model window is as left {a} then {b}")
    a, b = g(f"{T}/gs-d-set.txt"), g(f"{T}/gs-d-again.txt")
    ok(a == b and a is not None, f"kept: the estimate window is as left {a} then {b}")
    md, mo, mr = m(f"{T}/m-d-again.txt"), m(f"{T}/m-d-other.txt"), m(f"{T}/m-d-removed.txt")
    ok(md["panel"].get("Tools") == "away" and md["lettering"][0] == 1, f"kept: Tools away and lettering 1 after opening again: {md['panel'].get('Tools')} {md.get('lettering')}")
    ok(mo["panel"].get("Tools") == "away" and mo["lettering"][0] == 1, "kept: the same on another job")
    ok(g(f"{T}/gm-d-other.txt") == g(f"{T}/gm-d-set.txt"), "kept: the window is where it was on another job")
    ok(mr["panel"].get("Tools") == "shown" and mr["lettering"][0] == 0 and g(f"{T}/gm-d-removed.txt")[2:] == (1600, 960), f"settings removed: as first opened {g(f'{T}/gm-d-removed.txt')}")
if "e" in ONLY:
    h = [m(f"{T}/m-e-step{i}.txt")["size"].get(("model", "Wall")) for i in range(3)]
    print("  the Wall button, width and height by step:", h)
    ok(all(x is not None for x in h), "the Wall button is there at each step")
    if all(h):
        r = h[1][1] / h[0][1]
        ok(1.1 <= r <= 1.25, f"one step larger: the Wall button is {r:.3f} times as tall")
    for i in range(3):
        d = m(f"{T}/m-e-step{i}.txt"); w = d["window"]["model"]
        out = [n for (wn, n), (x, y) in d["ctl"].items() if wn == "model" and not (0 <= x - w[0] <= w[2] and 0 <= y - w[1] <= w[3])]
        ok(not out, f"step {i}: nothing of the model window cut off ({out[:4]})")
if "f" in ONLY:
    rc = open(f"{T}/f-recovery-copy.json").read() if os.path.exists(f"{T}/f-recovery-copy.json") else ""
    saved = open(f"{T}/f-job-as-saved.json").read() if os.path.exists(f"{T}/f-job-as-saved.json") else ""
    taken = open(f"{T}/f-taken-saved.json").read() if os.path.exists(f"{T}/f-taken-saved.json") else ""
    ok(rc != "" and rc != saved, "a recovery copy stood beside the job when the program was stopped dead")
    off = m(f"{T}/m-f-offered.txt")
    ok(("model", "Take the recovered work") in off["ctl"] and ("model", "Leave it") in off["ctl"], "opened again: the two answers are on the screen")
    ok(taken == rc, "taken and saved: the job is the recovery copy, byte for byte")
    ok(not os.path.exists(f"{T}/lost.json.recovery"), "after Save there is no recovery copy")
    cli = open(f"{T}/cli-made.json").read() if os.path.exists(f"{T}/cli-made.json") else None
    ok(cli is not None and taken == cli, "the taken job is, byte for byte, the job the command line makes by drawing the same wall and door")
    ok('"wall-' in taken and '"door-' in taken and '"wall-' not in saved, "the wall and the door are in the taken job and not in the job as last saved")
if "g" in ONLY:
    lf = open(f"{T}/g-left-saved.json").read() if os.path.exists(f"{T}/g-left-saved.json") else None
    base = open(f"{T}/f-job-as-saved.json").read() if os.path.exists(f"{T}/f-job-as-saved.json") else None
    ok(lf is not None and lf == base, "Leave it: the job saved after is the job as it was last saved, byte for byte")
    ok(not os.path.exists(f"{T}/left.json.recovery"), "and after that Save there is no recovery copy")
    lm = m(f"{T}/m-g-left.txt"); ok(("model", "Leave it") not in lm["ctl"], "the offer is gone once it is answered")
if "h" in ONLY:
    import re as _re
    screens_h = {"h1": (2560, 1440), "h2": (1920, 1080), "h3": (1366, 768)}
    for s, sc in screens_h.items():
        def ext(w):
            try: n = [int(x) for x in _re.findall(r"\d+", open(f"{T}/fx-{s}-{w}.txt").read())]
            except OSError: return None
            return n if len(n) == 4 else None
        def geo(w): return g(f"{T}/gw-{s}-{w}.txt")
        try: wa = [int(x) for x in _re.findall(r"\d+", open(f"{T}/work-{s}.txt").read())][:4]
        except OSError: wa = None
        for w in ("model", "sheet"):
            e, r = ext(w), geo(w)
            if e is None or r is None or wa is None or len(wa) < 4:
                ok(False, f"{s}: the {w} window's frame and the desktop's room were measured ({e} {r} {wa})"); continue
            left, right, top, bottom = e
            outer = g(f"{T}/ow-{s}-{w}.txt") or (r[0] - left, r[1] - top, r[2] + left + right, r[3] + top + bottom)
            ok(outer[0] >= wa[0] and outer[1] >= wa[1] and outer[0] + outer[2] <= wa[0] + wa[2] and outer[1] + outer[3] <= wa[1] + wa[3], f"{s}: the {w} window with its title bar {outer} is whole on the desktop's room {tuple(wa)} of the screen {sc}")
            print(f"        {s} {w}: title bar {top} dots, edges {left} and {right}, foot {bottom}")
        print(f"        {s}: the desktop's bar takes {sc[1] - wa[3] - wa[1] if wa else '?'} dots at the foot or head of the screen {sc}; the program allows 72")
print("PASS" if not bad else "FAIL: " + "; ".join(bad))
sys.exit(1 if bad else 0)
PY
