#!/usr/bin/env bash
# A shape seen and changed on the bench (DECISIONS.md call 205, point 6, with the one point of call 206 that is the window's):
# an item's shape listed block by block on the bench; a starter shape shown and not changed; "Keep as a new shape"; a block
# added, its figures typed, one refused; a block taken away and Undo; the things set down drawn from the job's own shape;
# the saved job the same, to the byte, as the command line's `shape` verb makes by the same steps; and an assembly changed
# for another saying what was typed on a line that came off, with its dollars. STATUS.md ("A shape on the bench: a
# demonstration") has the figures worked by hand.
#
# Four sessions in the window, each on its own run of the program and each from the job the one before left (so that each
# can be run in ten minutes), then the command line:
#   a  the made-up whole house and the starter tool chest, "Vanity, 36 in" set down twice; the bench; its three blocks; a click on
#      the third; a figure typed on a starter shape (refused); "Keep as a new shape"                     (steps 1 to 3)
#   b  a block added, its six figures typed; the panel on the vanity's front measured in the picture; a figure above the box
#      refused                                                                                           (steps 4 and 5)
#   c  Take block away and Undo; back to the house, both vanities close up; "Base cabinet, 36 in" on the bench, still 3 blocks
#                                                                                                         (steps 6 and 7)
#   d  a wall's assembly changed for another, with 175 SF typed on its siding line                       (step 9, call 206 point 1)
#   e  at the command line: the same steps with `plumbline shape` on the job as it began, the same file as the window's (step 8)
#
# Run from the repository root:  [PLUMBLINE_BIN=<folder>] [ONLY="a b c d e"] [STATE=<folder>] demos/shapebenchw.sh [folder for the pictures]
# STATE keeps the jobs between runs: to run only "c", give the STATE of a run that has made "a" and "b".
# It drives the real program with a real pointer and keyboard on a virtual display. The jobs and every figure are made up.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/shapebenchw.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL, numpy" 2>/dev/null || { echo "demos/shapebenchw.sh needs python's PIL and numpy, to read the pictures"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/shapebenchw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/shapebenchw}"; mkdir -p "$OUT"
ONLY="${ONLY:-a b c d e}"
if [ -n "${STATE:-}" ]; then T="$STATE"; mkdir -p "$T"; else T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT; fi
wants() { case " $ONLY " in *" $1 "*) return 0;; *) return 1;; esac; }
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; tail -30 "$1"; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; grep -F -- "$2" "$1" | head -5; exit 1; fi; printf '    (not said: %s)\n' "$2"; }

# One session: its steps (a file of shell, using the helpers of demos/window.sh) run in the window on a job.
session() {   # $1 name, $2 job; the steps are in $T/steps-$1.sh; the pictures go in $OUT/$1
  # CHECK_ONLY=1: read what an earlier run left in STATE and in the pictures' folder, and run nothing in the window.
  [ -z "${CHECK_ONLY:-}" ] || return 0
  mkdir -p "$OUT/$1"; touch "$T/stamp-$1"
  rm -f "$T/map.txt.said" "$T/missing.txt"
  cat > "$T/session-$1.sh" <<WRAP
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT/$1" ICD="$ICD" JOB="$2" NAME="$1"
. "$PWD/demos/window.sh"
keep_map() { sleep 3; cp "\$T/map.txt" "\$T/map-\$NAME-\$1.txt"; }
keep_said() { cp "\$T/map.txt.said" "\$T/said-\$NAME-\$1.txt"; }
. "$T/steps-$1.sh"
WRAP
  chmod +x "$T/session-$1.sh"
  timeout 580 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session-$1.sh" || true
  if [ -s "$T/missing.txt" ]; then echo "FAIL: session $1 stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
  if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
  [ -s "$T/said-$1-end.txt" ] || { echo "FAIL: session $1 did not reach its end (a run that is cut short at ten minutes leaves no end)"; exit 1; }
  cp "$T/map.txt.said" "$T/said-$1.txt" 2>/dev/null || true
  rm -f "$T/map.txt" "$T/map.txt.said"
}
# The names on the map of a step, and one line of it.
names() { awk -F'\t' '($1=="model" || $1=="away") {print $2}' "$T/map-$1.txt"; }
has() { names "$1" | grep -qxF -- "$2"; }
shown() { has "$1" "$2" || { echo "FAIL: not on the map at step $1: $2"; names "$1" | head -60; exit 1; }; printf '    on the map: %s\n' "$2"; }
absent() { if has "$1" "$2"; then echo "FAIL: on the map at step $1, and not to be: $2"; exit 1; fi; printf '    (not offered: %s)\n' "$2"; }
# What `start` does: the tool chest in, the vanity set down twice on the ground in front of the house, saved.
COMMON="$T/common.sh"
cat > "$COMMON" <<'COMMON'
start() {
  open_window "$1"
  sleep 8; park
  on "Starter tool chest"; sleep 5
  on_at "Item Vanity, 36 in"; sleep 4
  click 560 760; sleep 5
  on "Item Vanity, 36 in"; sleep 4
  click 900 760; sleep 5
  esc; park
  xdotool key F2; sleep 3
}
open_item() { on_at "Change item $1" "${2:-5}"; sleep 5; park; }
COMMON

# ---------------------------------------------------------------------------------------------------------------------
if wants a; then
echo "== The job: the made-up whole house, priced in all sixteen divisions =="
mkdir -p "$T/a"; [ -n "${CHECK_ONLY:-}" ] || "$P" sixteen "$T/a/bench.json" >/dev/null
"$P" check "$T/a/bench.json"
echo
echo "== Session a: the vanity on the bench: 3 blocks; the third picked; a starter shape not typed on; kept as a new shape =="
cat > "$T/steps-a.sh" <<'STEPS'
. "$T/common.sh"
start "$JOB"
cp "$JOB" "$T/saved-0.json"
open_item "Vanity, 36 in"
on_right "Bench"; sleep 12; park
shot; keep_map 01                           # 01 the vanity on the bench: Shape, three blocks; no Add block; Keep as a new shape
on_right "Shape block 3"; sleep 5; park
shot; keep_map 02                           # 02 the third block (the top) outlined; its rows
on_right "Up to"; sleep 3
typed "height + 1 in"; sleep 5; park
keep_said 02t
shot; keep_map 03                           # 03 what a starter shape says when typed on
esc
on_right "Keep as a new shape"; sleep 4
typed "vanity with a panel"; sleep 6; park
keep_said 03
keep_map 03k                                # the picked block stays picked on the copy
on_right "Shape block 3"; sleep 5; park     # (a click on the picked row lets go of it: the picture is the vanity as it was)
shot; keep_map 04                           # 04 kept as the job's own: Add block and Take block away offered
xdotool key F2; sleep 3
cp "$JOB" "$T/saved-a.json"
keep_said end
close_window
STEPS
session a "$T/a/bench.json"
for n in 01 02 03 04; do
  [ "$OUT/a/$n-model.png" -nt "$T/stamp-a" ] && [ "$OUT/a/$n-sheet.png" -nt "$T/stamp-a" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  8 pictures saved to $OUT/a"
echo
echo "== 1. The vanity on the bench: Shape lists its 3 blocks; no Add block; Keep as a new shape, with its line of why =="
for b in 1 2 3; do shown a-01 "Shape block $b"; done
shown a-01 "Keep as a new shape"; absent a-01 "Add block"; absent a-01 "Take block away"
echo
echo "== 2. A click on the third row picks the top: outlined on the bench; its rows read as the table writes them; typing on a starter shape is refused in words =="
shown a-02 "Up to"; shown a-02 "Across from"; shown a-02 "Across to"
python3 -I - "$T/map-a-02.txt" <<'PY'
import sys
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1])]
shape = [r for r in rows if r[0] == "shape"]
assert len(shape) == 1, "no picked block on the map"
r = shape[0]
assert r[2:4] == ["3", "worktop"] and r[4:6] == ["box", "always"], r
want = ["-width / 2", "width / 2", "0", "depth", "height - min(40 mm, 0.06 * height)", "height"]
assert r[6:12] == want, f"the picked block's rows read {r[6:12]}, to read {want}"
assert any(x[0] == "edge" and x[1] == "block" for x in rows), "the block is not outlined on the bench"
print("    block 3 is the worktop, a box, there always: across -width / 2 to width / 2, back 0 to depth, up from height - min(40 mm, 0.06 * height) to height")
PY
says "$T/said-a-02t.txt" "is a starter shape, and a starter shape is not changed"
says "$T/said-a-02t.txt" "keep it as a new shape first, then change that"
echo
echo "== 3. Kept as a new shape: one item drawn as it, two set down follow; the item's Drawn as lists it; the picture is the vanity as it was =="
says "$T/said-a-03.txt" "Vanity with a panel: kept as a new shape from base cabinet: its 3 blocks, word for word. 1 item is drawn as it, and 2 things set down from it follow."
shown a-04 "Drawn as Vanity with a panel"; shown a-04 "Add block"; absent a-04 "Keep as a new shape"
shown a-03k "Take block away"; absent a-03k "Keep as a new shape"; absent a-04 "Take block away"
python3 -I - "$OUT/a" <<'PY'
import sys
from PIL import Image, ImageChops
import numpy as np
d = sys.argv[1]
a = Image.open(f"{d}/01-model.png").convert("RGB"); b = Image.open(f"{d}/04-model.png").convert("RGB")
w, h = a.size
box = (int(w * 0.30), int(h * 0.38), int(w * 0.67), int(h * 0.78))
diff = np.asarray(ImageChops.difference(a.crop(box), b.crop(box))).astype(float)
print(f"    the vanity before and after being kept, over the middle of the model window: mean difference {diff.mean():.2f} of 255, {100.0 * (diff.max(axis=2) > 24).mean():.2f} % of dots differ by more than 24")
assert diff.mean() < 2.0, "the picture is not the vanity as it was"
PY
"$P" check "$T/saved-a.json"
"$P" shape "$T/saved-a.json" > "$T/a-shapes.txt"
says "$T/a-shapes.txt" "vanity with a panel: 3 blocks; 1 item drawn as it, 2 things set down from them"
fi

# ---------------------------------------------------------------------------------------------------------------------
if wants b; then
echo
echo "== Session b: a block added, its six figures typed; the panel measured on the picture; a figure above the box refused =="
mkdir -p "$T/b"; [ -n "${CHECK_ONLY:-}" ] || cp "$T/saved-a.json" "$T/b/bench.json"
cat > "$T/steps-b.sh" <<'STEPS'
. "$T/common.sh"
open_window "$JOB"
sleep 8; park
open_item "Vanity, 36 in"
on_right "Bench"; sleep 12; park
on_right "Add block"; sleep 6; park
keep_said 01
shot; keep_map 01                           # 01 a fourth row, picked: a box the vanity's whole size
on_right "Across from"; sleep 3; typed "-width / 2 + 2 in"; sleep 3
on_right "Across to"; sleep 3; typed "width / 2 - 2 in"; sleep 3
on_right "Back from"; sleep 3; typed "depth - 1 in"; sleep 3
on_right "Back to"; sleep 3; typed "depth"; sleep 3
on_right "Up from"; sleep 3; typed "4 in"; sleep 3
on_right "Up to"; sleep 3; typed "height - 6 in"; sleep 5; park
keep_said 02
shot; keep_map 02                           # 02 the panel stands on the vanity's front, 2 in in from each side
xdotool key F2; sleep 3
cp "$JOB" "$T/saved-b.json"
on_right "Up to"; sleep 3; typed "height + 1 in"; sleep 5; park
keep_said 03
shot; keep_map 03                           # 03 refused: the row still reads height - 6 in
esc
xdotool key F2; sleep 3
cmp "$JOB" "$T/saved-b.json" > "$T/b-after-refusal.txt" 2>&1 && echo same > "$T/b-after-refusal.txt"
keep_said end
close_window
STEPS
session b "$T/b/bench.json"
for n in 01 02 03; do
  [ "$OUT/b/$n-model.png" -nt "$T/stamp-b" ] && [ "$OUT/b/$n-sheet.png" -nt "$T/stamp-b" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  6 pictures saved to $OUT/b"
echo
echo "== 4. Add block: a fourth row, picked; its figures typed; the panel on the vanity's front, 2 in in from each side: 32 / 36 = 0.889 of its width (0.86 to 0.92) =="
shown b-01 "Shape block 4"; shown b-01 "Take block away"
says "$T/said-b-01.txt" "block 4 added, the whole size of the item, made of timber until its figures are typed"
python3 -I - "$T/map-b-01.txt" "$T/map-b-02.txt" <<'PY'
import sys, math
def rows(p): return [l.rstrip("\n").split("\t") for l in open(p)]
a, b = rows(sys.argv[1]), rows(sys.argv[2])
sa = [r for r in a if r[0] == "shape"][0]
assert sa[2] == "4" and sa[6:12] == ["-width / 2", "width / 2", "0", "depth", "0", "height"], f"a block added is the item's whole box: {sa}"
sb = [r for r in b if r[0] == "shape"][0]
want = ["-width / 2 + 2 in", "width / 2 - 2 in", "depth - 1 in", "depth", "4 in", "height - 6 in"]
assert sb[2] == "4" and sb[6:12] == want, f"the fourth block reads {sb[6:12]}, to read {want}"
def length(rs, what):
    r = [x for x in rs if x[0] == "edge" and x[1] == what][0]
    return math.hypot(float(r[4]) - float(r[2]), float(r[5]) - float(r[3]))
whole = length(a, "block") / length(a, "item")
panel = length(b, "block") / length(b, "item")
print(f"    the block added is {whole:.3f} of the vanity's width on screen (1.000); typed, the panel is {panel:.3f} (32 / 36 = 0.889)")
assert abs(whole - 1.0) < 0.02, "a block added is not the item's whole width"
assert 0.86 <= panel <= 0.92, f"the panel's width is {panel:.3f} of the vanity's, to be 0.889"
print("    the fourth block's rows: across -width / 2 + 2 in to width / 2 - 2 in, back depth - 1 in to depth, up 4 in to height - 6 in")
PY
for f in "block 4, Across from, set to -width / 2 + 2 in" "block 4, Across to, set to width / 2 - 2 in" "block 4, Back from, set to depth - 1 in" "block 4, Back to, set to depth" "block 4, Up from, set to 4 in" "block 4, Up to, set to height - 6 in"; do
  says "$T/said-b-02.txt" "$f"
done
says "$T/said-b-02.txt" "Block 4 reads, at Vanity, 36 in's sizes (36 in x 21 in x 34.5 in): timber: across -16 in to 16 in; back 20 in to 21 in; up 4 in to 28.5 in."
says "$T/said-b-02.txt" "1 item is drawn as it, and 2 things set down from it follow."
echo
echo "== 5. A figure above the box is refused, in the model's words; the row is as it was; the job is as it was =="
# (the model's words since call 212: "above the top"; they were "above the box")
says "$T/said-b-03.txt" "1 in above the top"
python3 -I - "$T/map-b-03.txt" <<'PY'
import sys
r = [l.rstrip("\n").split("\t") for l in open(sys.argv[1]) if l.startswith("shape\t")][0]
assert r[11] == "height - 6 in", f"the row reads {r[11]}"
print("    the Up to row still reads height - 6 in")
PY
[ "$(cat "$T/b-after-refusal.txt")" = same ] || { echo "FAIL: the job changed with the refusal"; exit 1; }
echo "    the job saved after the refusal is the job saved before it, to the byte"
fi

# ---------------------------------------------------------------------------------------------------------------------
if wants c; then
echo
echo "== Session c: Take block away and Undo; back to the house; both vanities close up; Base cabinet, 36 in on the bench =="
mkdir -p "$T/c"; [ -n "${CHECK_ONLY:-}" ] || cp "$T/saved-b.json" "$T/c/bench.json"
cat > "$T/steps-c.sh" <<'STEPS'
. "$T/common.sh"
open_window "$JOB"
sleep 8; park
open_item "Vanity, 36 in"
on_right "Bench"; sleep 12; park
on_right "Shape block 4"; sleep 4
on_right "Take block away"; sleep 6; park
keep_said 01
shot; keep_map 01                           # 01 three rows, the picture as the kept shape began
on "Undo"; sleep 6; park
shot; keep_map 02                           # 02 Undo: four rows again
on_right "Back to the house"; sleep 12; park
xdotool mousemove 560 780; sleep 2
for i in 1 2 3; do xdotool click 4; sleep 2; done; sleep 6; park
shot; keep_map 03                           # 03 back in the house, one vanity close up: the panel on its front
xdotool key F2; sleep 3
cp "$JOB" "$T/saved-c.json"
esc; sleep 2; open_item "Base cabinet, 36 in"
on_right "Bench"; sleep 12; park
shot; keep_map 04                           # 04 Base cabinet, 36 in on the bench: still 3 blocks, no panel
keep_said end
close_window
STEPS
session c "$T/c/bench.json"
for n in 01 02 03 04; do
  [ "$OUT/c/$n-model.png" -nt "$T/stamp-c" ] && [ "$OUT/c/$n-sheet.png" -nt "$T/stamp-c" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  8 pictures saved to $OUT/c"
echo
echo "== 6. Take block away with the fourth picked: 3 rows, and the picture is step 3's; Undo: 4 rows again =="
shown c-01 "Shape block 3"; absent c-01 "Shape block 4"
says "$T/said-c-01.txt" "block 4 taken away; the blocks after it are numbered one less"
shown c-02 "Shape block 4"
python3 -I - "$OUT/a" "$OUT/c" <<'PY'
import sys
from PIL import Image, ImageChops
import numpy as np
a = Image.open(f"{sys.argv[1]}/04-model.png").convert("RGB"); b = Image.open(f"{sys.argv[2]}/01-model.png").convert("RGB")
w, h = a.size
box = (int(w * 0.30), int(h * 0.38), int(w * 0.67), int(h * 0.78))
diff = np.asarray(ImageChops.difference(a.crop(box), b.crop(box))).astype(float)
print(f"    the vanity with the block taken away, beside the vanity as it was kept: mean difference {diff.mean():.2f} of 255")
assert diff.mean() < 2.5, "the picture is not step 3's"
PY
echo
echo "== 7. Back to the house: both vanities show the panel (a close picture of one); Base cabinet, 36 in still has 3 blocks and no panel =="
shown c-04 "Shape block 3"; absent c-04 "Shape block 4"; shown c-04 "Keep as a new shape"; absent c-04 "Add block"
echo "    (the close picture is $OUT/c/03-model.png: it is looked at, and what it shows is said in STATUS.md)"
fi

# ---------------------------------------------------------------------------------------------------------------------
if wants e; then
echo
echo "== 8. The saved job and the command line: the same steps made with \`plumbline shape\` on a copy of the starting job give the same file, to the byte =="
[ -s "$T/saved-0.json" ] && [ -s "$T/saved-c.json" ] || { echo "FAIL: no saved jobs: run sessions a, b and c first (STATE=<folder>)"; exit 1; }
cp "$T/saved-0.json" "$T/cli.json"
"$P" shape "$T/cli.json" keep "Vanity, 36 in" "vanity with a panel" > "$T/e-1.txt"
"$P" shape "$T/cli.json" add "vanity with a panel" > "$T/e-2.txt"
n=0
for pair in "across from|-width / 2 + 2 in" "across to|width / 2 - 2 in" "back from|depth - 1 in" "back to|depth" "up from|4 in" "up to|height - 6 in"; do
  "$P" shape "$T/cli.json" set "vanity with a panel" 4 "${pair%%|*}" "${pair#*|}" > "$T/e-set-$n.txt"; n=$((n + 1))
done
"$P" check "$T/saved-c.json"
"$P" shape "$T/saved-c.json" "vanity with a panel" > "$T/e-list.txt"
sed 's/^/    | /' "$T/e-list.txt"
says "$T/e-list.txt" "vanity with a panel (the job's own): 4 blocks"
says "$T/e-list.txt" "block 4, timber: across -width / 2 + 2 in to width / 2 - 2 in; back depth - 1 in to depth; up 4 in to height - 6 in"
"$P" items "$T/saved-c.json" "Vanity, 36 in" > "$T/e-items.txt"
says "$T/e-items.txt" "block 4, timber: across -16 in to 16 in; back 20 in to 21 in; up 4 in to 28.5 in"
cmp "$T/cli.json" "$T/saved-c.json" && echo "    the job the command line made and the job the window saved are the same file, to the byte"
BEFORE="$("$P" price "$T/saved-0.json")"; AFTER="$("$P" price "$T/saved-c.json")"
[ "$BEFORE" = "$AFTER" ] || { echo "FAIL: the price changed with a shape"; exit 1; }
echo "    the price is the figure it was: a shape changes no money"
fi

# ---------------------------------------------------------------------------------------------------------------------
if wants d; then
echo
echo "== 9. An assembly changed in the window: a line comes off that had a quantity typed on it, and the message names it with its dollars (call 206 point 1) =="
mkdir -p "$T/d"; J="$T/d/wall.json"; K=(--status Known --source "made-up survey")
if [ -z "${CHECK_ONLY:-}" ]; then
"$P" new "$J" "Assembly changed (not a real job)" >/dev/null
"$P" level "$J" ground --elevation "0 ft" --height "10 ft" "${K[@]}" >/dev/null
"$P" type "$J" wall w6 --thickness "6 in" "${K[@]}" >/dev/null
"$P" wall "$J" level-0002 type-0003 --points "0,0 20,0 20,16 0,16" --closed --height "10 ft" "${K[@]}" >/dev/null
"$P" import assemblies "$J" starter >/dev/null
"$P" apply "$J" wall-0004 "Exterior wall, 2x6, lap siding" --source "made-up plan" >/dev/null
"$P" set "$J" wall-0004 unit_cost '$4.00' --line "Lap siding" "${K[@]}" >/dev/null
"$P" set "$J" wall-0004 quantity "175 SF" --line "Lap siding" "${K[@]}" >/dev/null
fi
cat > "$T/steps-d.sh" <<'STEPS'
open_window "$JOB"
on "Plan"; sleep 8; park
at 10 0; sleep 3                            # the first wall of the ring, 20 ft long: lap siding on it, 175 SF typed where the assembly writes 200
shot; keep_map 01                           # 01 the wall selected
on_right "Assembly"; sleep 3; park
on_right "Assembly Exterior wall, 2x6, brick veneer"; sleep 8; park
keep_said 02
shot; keep_map 02                           # 02 changed for brick veneer: the message names the siding with the 175 SF and its $700.00
keep_said end
close_window
STEPS
session d "$J"
for n in 01 02; do [ "$OUT/d/$n-model.png" -nt "$T/stamp-d" ] || { echo "FAIL: no picture $n from this run"; exit 1; }; done
# By hand: 20 ft x 10 ft = 200 SF where the assembly writes net area; 175 SF was typed; at $4.00/SF it came to $700.00.
says "$T/said-d-02.txt" "3 cost lines of the assembly it had came off: Lap siding, with a quantity of 175.00 SF"
says "$T/said-d-02.txt" "where the assembly writes"
says "$T/said-d-02.txt" "(it came to \$700.00)"
fi
echo
echo "  PASS (sessions run: $ONLY)"
