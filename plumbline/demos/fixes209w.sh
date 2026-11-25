#!/usr/bin/env bash
# The third window judge's points that are the window's (call 209, contract 205W part 2), each shown on the frozen program
# first (BEFORE=1: the same steps, the pictures kept as they are and nothing checked) and then on the program as built:
#   1  the estimate window's list of what is set down reads the model's count: a thing with no size is counted and said not to
#      be drawn; a fitting is not a thing set down
#   3  what the window says when the building is written out: where the estimate is written out (Estimate workbook, under
#      Write out), and no command's name
#   6  the red dashed line on the sample house's north wall with Clashes off: a pipe with no size, drawn as its centerline; it is
#      quiet ink under the Plumbing switch, red only under "Known, Assumed, Unknown", and listed "size unknown" on the sheet
#   7  the lone square at the lower left of the model window: gone when nothing is said there
#   9  "Priced as" stands once on an item's panel
#   10 the item's Price row says where its price comes from and how it is changed
#   (8, the top bar counting the building apart from the tool chest, needs the model's function of contract 209M: not built here.)
#
# Three sessions, each its own run of the program on its own made-up job:
#   a  an empty job with a level, the starter tool chest and a stool whose size the maker's sheet does not give: a vanity and the stool
#      set down; the sheet's Building list and Estimate                                           (point 1)
#   b  the made-up whole house: the BIM model written out; the Plan; the certainty layer; Plumbing off (points 3, 6, 7, and 1 for fittings)
#   c  the starter tool chest, the water heater given a price: its panel                          (points 9 and 10)
#
# Run from the repository root:  [PLUMBLINE_BIN=<folder>] [BEFORE=1] [ONLY="a b c"] demos/fixes209w.sh [folder for the pictures]
# It drives the real program with a real pointer and keyboard on a virtual display. The jobs and every figure are made up.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/fixes209w.sh needs $tool, which is not installed"; exit 2; }
done
python3 -c "import PIL, numpy" 2>/dev/null || { echo "demos/fixes209w.sh needs python's PIL and numpy, to read the pictures"; exit 2; }
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/fixes209w.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
BEFORE="${BEFORE:-}"
OUT="${1:-$PWD/demos/captured/fixes209w}"; mkdir -p "$OUT"
ONLY="${ONLY:-a b c}"
if [ -n "${STATE:-}" ]; then T="$STATE"; mkdir -p "$T"; else T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT; fi
K=(--status Known --source "made-up survey")
wants() { case " $ONLY " in *" $1 "*) return 0;; *) return 1;; esac; }
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; tail -20 "$1" | cut -c1-300; exit 1; }; printf '    %s\n' "$2"; }
lacks() { if grep -qF -- "$2" "$1"; then echo "FAIL: it says what it should not: $2"; grep -F -- "$2" "$1" | head -3 | cut -c1-300; exit 1; fi; printf '    (not said: %s)\n' "$2"; }
# $1 session name, $2 job; the steps are in $T/steps-$1.sh; the pictures go in $OUT/$1
session() {
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
  [ -s "$T/said-$1-end.txt" ] || { echo "FAIL: session $1 did not reach its end (a run cut short at ten minutes leaves no end)"; exit 1; }
}
# Words on the screen, from the map: `text<TAB>window<TAB>words<TAB>x<TAB>y`.
words() { awk -F'\t' -v w="$2" '$1=="text" && $2==w {print $3}' "$T/map-$1.txt"; }
saw() { words "$1" "$2" | grep -qxF -- "$3"; }
sawlike() { words "$1" "$2" | grep -qF -- "$3"; }
shown_text() { sawlike "$1" "$2" "$3" || { echo "FAIL: not on the screen at step $1 ($2 window): $3"; words "$1" "$2" | head -60 | cut -c1-120; exit 1; }; printf '    on the screen: %s\n' "$3"; }
no_text() { if sawlike "$1" "$2" "$3"; then echo "FAIL: on the screen at step $1 ($2 window), and not to be: $3"; exit 1; fi; printf '    (not on the screen: %s)\n' "$3"; }
# How many dots of a picture along the longer pieces of a run that has no size (30 dots or more on screen, less six dots at each end, where
# something else may cross) are the red of a mark of what is Unknown: the program writes where each piece is on screen, as `unsized` lines of its map.
red_dots() { python3 -I - "$1" "$2" "${3:-red}" <<'PY'
import sys, math
from PIL import Image
import numpy as np
a = np.asarray(Image.open(sys.argv[1]).convert("RGB")).astype(int)
total, pieces = 0, 0
for line in open(sys.argv[2]):
    f = line.rstrip("\n").split("\t")
    if f[0] != "unsized":
        continue
    x1, y1, x2, y2 = map(float, f[2:6])
    n = int(math.hypot(x2 - x1, y2 - y1))
    if n < 30:
        continue
    pieces += 1
    seen = set()
    for i in range(6, n - 5):
        x, y = x1 + (x2 - x1) * i / n, y1 + (y2 - y1) * i / n
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                seen.add((int(round(x)) + dx, int(round(y)) + dy))
    for (x, y) in seen:
        if 0 <= y < a.shape[0] and 0 <= x < a.shape[1]:
            r, g, b = a[y, x]
            total += int((r > 150 and g < 100 and b < 100) if sys.argv[3] == "red" else (r < 125 and g < 130 and b < 140))
print(total, pieces)
PY
}
# How different the lower left corner of the model window is from the ground beside it: a card there is not the ground.
corner_card() { python3 -I - "$1" <<'PY'
import sys
from PIL import Image
import numpy as np
a = np.asarray(Image.open(sys.argv[1]).convert("L")).astype(float)
h, w = a.shape
card = a[h - 38:h - 12, 10:52]     # where the little square's card stood
beside = a[h - 38:h - 12, 70:112]  # the same size of what is beside it
print(f"{abs(card.mean() - beside.mean()):.1f} {card.std():.1f}")
PY
}

# ---------------------------------------------------------------------------------------------------------------------
if wants a; then
echo "== The job: an empty job with a level, the starter tool chest, and a stool whose size the maker's sheet does not give =="
mkdir -p "$T/a"; J="$T/a/set.json"
if [ -z "${CHECK_ONLY:-}" ]; then
  rm -f "$J"
  "$P" new "$J" "Set down (not a real job)" >/dev/null
  "$P" level "$J" ground --elevation "0 ft" --height "9 ft" --status Assumed --source "made-up tracing" >/dev/null
  "$P" import items "$J" starter >/dev/null
  printf 'Item,Trade,Kind,Drawn as,Width,Depth,Height,How sure,Where from\nHarbor Stool,12 Furnishings,furnishing,,,,,Unknown,not on the maker'"'"'s sheet\n' > "$T/a/stool.csv"
  "$P" import items "$J" "$T/a/stool.csv" >/dev/null
fi
cat > "$T/steps-a.sh" <<'STEPS'
open_window "$JOB"
sleep 8; park
on_at "Item Vanity, 36 in"; sleep 4
click 560 760; sleep 5
on_at "Item Harbor Stool"; sleep 4
click 900 760; sleep 5
esc; park
xdotool key F2; sleep 3
shot; keep_map 01                           # 01 the vanity and the stool set down (the stool has no size to draw)
sheet "Building"; sleep 4; park
shot; keep_map 02                           # 02 the sheet's Building list: what is set down on the level
sheet "Estimate"; sleep 4; park
shot; keep_map 03                           # 03 the sheet's Estimate: the same two
keep_said end
close_window
STEPS
session a "$J"
if [ -z "$BEFORE" ]; then
for n in 01 02 03; do [ "$OUT/a/$n-model.png" -nt "$T/stamp-a" ] && [ "$OUT/a/$n-sheet.png" -nt "$T/stamp-a" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }; done
echo
echo "== 1. The sheet's list of what is set down reads the model's count: the stool is counted, and said not to be drawn =="
"$P" takeoff "$T/a/set.json" > "$T/a-takeoff.txt" || true
python3 -I - "$T/map-a-02.txt" <<'PY'
import sys
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1])]
names = [r[2] for r in rows if r[0] == "sheet"] + [r[1] for r in rows if r[0] == "sheet"]
want = "Set down on Ground: 2 furnishings"
assert any(r[0] == "sheet" and r[1] == want for r in rows), f"the Building list has no row {want!r}: {[r[1] for r in rows if r[0] == 'sheet'][:20]}"
print(f"    the Building list reads: {want}")
PY
shown_text a-02 sheet "1 furnishing cannot be drawn: its size is not known."
shown_text a-02 sheet "Vanity, 36 in, Harbor Stool"
shown_text a-03 sheet "2 furnishings are set down and not priced."
echo "    (the model's own, takeoff, says:)"
grep -F -A3 "Set down on this level" "$T/a-takeoff.txt" | sed 's/^/    | /'
fi
fi

# ---------------------------------------------------------------------------------------------------------------------
if wants b; then
echo
echo "== The made-up whole house: the BIM model written out; the Plan; the certainty layer; Plumbing off =="
mkdir -p "$T/b"; J="$T/b/house.json"
[ -n "${CHECK_ONLY:-}" ] || { rm -f "$J"; "$P" sixteen "$J" >/dev/null; }
cat > "$T/steps-b.sh" <<'STEPS'
open_window "$JOB"
sleep 8; park
on_at "BIM model"; sleep 12; park
keep_said 01
shot; keep_map 01                           # 01 the building written out: what the window says is not in it, and where the estimate is; the lower left
on "Plan"; sleep 10; park
shot; keep_map 02                           # 02 the plan: the pipe with no size, as the window draws it
on_at "Show Known, Assumed, Unknown"; sleep 6; park
shot; keep_map 03                           # 03 the certainty layer on: the same pipe in red
on_at "Show Known, Assumed, Unknown"; sleep 4
on_at "Show Plumbing"; sleep 6; park
shot; keep_map 04                           # 04 the plumbing switched off: the pipe with no size goes with it
on_at "Show Plumbing"; sleep 4
sheet "Building"; sleep 4; park
shot; keep_map 05                           # 05 the sheet's Building list for the house
sheet "Plumbing"; sleep 4; park
shot; keep_map 06                           # 06 the sheet's Plumbing tab: the hot water to the kitchen sink, listed
keep_said end
close_window
STEPS
session b "$J"
if [ -z "$BEFORE" ]; then
for n in 01 02 03 04 05 06; do [ "$OUT/b/$n-model.png" -nt "$T/stamp-b" ] && [ "$OUT/b/$n-sheet.png" -nt "$T/stamp-b" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }; done
echo
echo "== 3. The building written out: where the estimate is written out, and no command's name =="
grep -F "Not in it" "$T/said-b-01.txt" | head -1 | cut -c1-400 | sed 's/^/    | /'
says "$T/said-b-01.txt" "Estimate workbook, under Write out"
lacks "$T/said-b-01.txt" "export estimate"
lacks "$T/said-b-01.txt" '`'
echo
echo "== 7. The lone square at the lower left of the model window: nothing is there when nothing is said there =="
read -r diff sd <<<"$(corner_card "$OUT/b/01-model.png")"
echo "    the corner at the lower left differs from the ground beside it by $diff of 255 (spread $sd): no card"
python3 -I -c "import sys; sys.exit(0 if float('$diff') < 12.0 and float('$sd') < 12.0 else 1)" || { echo "FAIL: something is still drawn at the lower left with nothing said there"; exit 1; }
echo
echo "== 6. The red dashed line: a pipe with no size. Not red until the certainty layer is on; under the Plumbing switch; listed so =="
"$P" takeoff "$T/b/house.json" | grep -F "size unknown" | head -2 | sed 's/^/    | the model lists: /'
read -r r2 n2 <<<"$(red_dots "$OUT/b/02-model.png" "$T/map-b-02.txt")"; read -r r3 n3 <<<"$(red_dots "$OUT/b/03-model.png" "$T/map-b-03.txt")"
echo "    the pipe's longer piece on the plan: $r2 red dots along it (layer off), $r3 (layer on)"
python3 -I -c "import sys; sys.exit(0 if $n2 >= 1 and $r2 == 0 and $r3 >= 12 else 1)" || { echo "FAIL: the unsized pipe is red with the layer off, or not red with it on"; exit 1; }
read -r d2 _ <<<"$(red_dots "$OUT/b/02-model.png" "$T/map-b-02.txt" dark)"; read -r d4 _ <<<"$(red_dots "$OUT/b/04-model.png" "$T/map-b-04.txt" dark)"
echo "    quiet ink along it: $d2 dots with Plumbing on, $d4 with Plumbing off: the trade's own switch has it"
python3 -I -c "import sys; sys.exit(0 if $d2 >= 20 and $d4 * 4 < $d2 else 1)" || { echo "FAIL: the unsized pipe is not drawn in quiet ink, or stays with Plumbing off"; exit 1; }
shown_text b-06 sheet "size unknown"
echo
echo "== 1 (fittings). What is set down on the house's levels counts no fitting =="
python3 -I - "$T/map-b-05.txt" <<'PY'
import sys
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1])]
names = [r[1] for r in rows if r[0] == "sheet" and r[1].startswith("Set down on")]
print("    " + "; ".join(names))
assert names and not any("fitting" in n for n in names), "a fitting is listed as set down"
PY
fi
fi

# ---------------------------------------------------------------------------------------------------------------------
if wants c; then
echo
echo "== The starter tool chest, the water heater given a price; its panel =="
mkdir -p "$T/c"; J="$T/c/item.json"
if [ -z "${CHECK_ONLY:-}" ]; then
  rm -f "$J"
  "$P" new "$J" "Item (not a real job)" >/dev/null
  "$P" level "$J" ground --elevation "0 ft" --height "9 ft" --status Assumed --source "made-up tracing" >/dev/null
  "$P" import items "$J" starter >/dev/null
  "$P" set "$J" "Water heater, 50 gal" price '$1,850.00' --status Assumed --source "a made-up list" >/dev/null
fi
cat > "$T/steps-c.sh" <<'STEPS'
open_window "$JOB"
sleep 8; park
on_at "Change item Water heater, 50 gal"; sleep 6; park
shot; keep_map 01                           # 01 the item's panel: Priced as; the Price row and what it says
keep_said end
close_window
STEPS
session c "$J"
if [ -z "$BEFORE" ]; then
for n in 01; do [ "$OUT/c/$n-model.png" -nt "$T/stamp-c" ] && [ "$OUT/c/$n-sheet.png" -nt "$T/stamp-c" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }; done
echo
echo "== 9. \"Priced as\" stands once on an item's panel =="
n="$(awk -F'\t' '$1=="text" && $2=="model" && tolower($3)=="priced as" {print $3}' "$T/map-c-01.txt" | wc -l)"
echo "    the words \"Priced as\" alone, as a heading or a button, on the panel: $n"
[ "$n" = 1 ] || { echo "FAIL: \"Priced as\" stands $n times"; exit 1; }
echo
echo "== 10. The item's Price row says where its price comes from and how it is changed =="
shown_text c-01 model "Not typed here: it is what the price list says. To change it, bring in a price list with the new price (Bring in, Price list)."
shown_text c-01 model "a made-up list"
fi
fi
echo
echo "  PASS (sessions run: $ONLY${BEFORE:+; BEFORE: the pictures only, nothing checked})"
