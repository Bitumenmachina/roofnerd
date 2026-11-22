#!/usr/bin/env bash
# Two layouts side by side, in the sheet window (DECISIONS.md call 156; the command line's half
# is demos/options.sh and its figures are worked by hand in STATUS.md, "Two layouts side by
# side"). The two jobs of options.sh are made with the command line in one folder: A, "Addition
# 12 ft", and B, "Addition 16 ft" (and a third, left empty, that is only another file in the
# list). A is opened in the window; the sheet's Layouts tab is opened; "Set beside another
# layout" is pressed and B is picked from the list; the sheet is photographed; then "Take it
# away" is pressed. The window's table is read from what it writes of the screen and is set
# against `plumbline options` row by row; neither file changes, byte for byte.
# Run from the repository root:  demos/layoutsw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display, which
# needs Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The jobs are made up
# (no real job data). Every step fails the script if the program does not do what it says.
#
# Worked by hand (STATUS.md): the Kitchen is 182.125 SF in A and 244.125 SF in B, +62 SF (B's
# addition is 4 ft deeper: 4 x 15.5); the job cost $5,777.00 and $7,137.00, +$1,360.00.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool python3; do
  command -v "$tool" >/dev/null || { echo "demos/layoutsw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/layoutsw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline"; APP="${PLUMBLINE_BIN:-${CARGO_TARGET_DIR:-$PWD/target}/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/layoutsw}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
K=(--status Known --source "made-up survey")
N=(--status Assumed --source "made-up addition plan")
made() { grep -oE '[a-z]+-[0-9]+' <<<"$1" | head -1; }
says() { grep -qF -- "$2" "$1" || { echo "FAIL: it does not say: $2"; head -60 "$1"; exit 1; }; printf '    %s\n' "$2"; }

# A layout (as demos/options.sh makes it): the house, its east wall to demolish, an addition
# $3 ft out from it, two rooms and a room line, the addition's walls priced at $12/SF and the
# Kitchen's floor at $8/SF.
layout() {
  local J="$1" title="$2" x="$3" S WT SOUTH EAST NORTH WEST NEW KITCHEN
  "$P" new "$J" "$title" >/dev/null
  S="$(made "$("$P" level "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
  WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
  read -r SOUTH EAST NORTH WEST <<<"$("$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
  "$P" mark "$J" "$SOUTH" "$NORTH" "$WEST" existing --keep "${K[@]}" >/dev/null
  "$P" mark "$J" "$EAST" demolish "${K[@]}" >/dev/null
  NEW="$("$P" wall "$J" "$S" "$WT" --points "24,0 $x,0 $x,16 24,16" --height "9 ft" "${N[@]}" | grep -oE 'wall-[0-9]+' | tr '\n' ' ')"
  "$P" room "$J" "$S" "Living" --at "6,8" "${K[@]}" >/dev/null
  KITCHEN="$(made "$("$P" room "$J" "$S" "Kitchen" --at "$((x - 6)),8" "${K[@]}")")"
  "$P" room-line "$J" "$S" --points "24,0 24,16" "${N[@]}" >/dev/null
  for w in $NEW; do
    "$P" line "$J" "$w" "Wall, framed and finished" face_area --cost-type material --division 6 >/dev/null
    "$P" set "$J" "$w" unit_cost '$12.00/SF' --line "Wall, framed and finished" --status Known --source "made-up price" >/dev/null
  done
  "$P" line "$J" "$KITCHEN" "Flooring" floor_area --cost-type material --division 9 >/dev/null
  "$P" set "$J" "$KITCHEN" unit_cost '$8.00/SF' --line "Flooring" --status Known --source "made-up price" >/dev/null
}

echo "== Three jobs made from empty with the command line, in one folder =="
mkdir -p "$T/jobs"
layout "$T/jobs/addition-12.json" "Addition 12 ft" 36
layout "$T/jobs/addition-16.json" "Addition 16 ft" 40
"$P" new "$T/jobs/nothing-yet.json" "Nothing drawn yet" >/dev/null
for f in addition-12 addition-16 nothing-yet; do cp "$T/jobs/$f.json" "$T/$f.before"; done
"$P" options "$T/jobs/addition-12.json" "$T/jobs/addition-16.json" | tr -s ' ' | sed 's/^ //' > "$T/cli.txt"
says "$T/cli.txt" "Kitchen ~182.125 SF ~244.125 SF +62 SF"
says "$T/cli.txt" "job cost ~\$5,777.00 ~\$7,137.00 +\$1,360.00"
cp "$T/jobs/addition-12.json" "$T/before.json"

echo
echo "== The session: A open; the sheet's Layouts tab; Set beside another layout; B picked from the list; then Take it away =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$T/jobs/addition-12.json"
sheet "Layouts"; park
shot                                     # 01 the Layouts tab, nothing set beside this job
sheet "Set beside another layout"; park
shot                                     # 02 the list of job files kept beside this one
sheet "Layout addition-16"; sleep 6; park
cp "$T/map.txt" "$T/table.txt"
shot                                     # 03 the layouts side by side
sheet "Take it away"; sleep 6; park
cp "$T/map.txt" "$T/away.txt"
shot                                     # 04 taken away
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
[ -s "$T/away.txt" ] || { echo "FAIL: the session did not run to its end"; exit 1; }
echo "  8 pictures saved to $OUT (model and sheet, after each of 4 steps)"

echo
echo "== What the window said =="
cp "$T/map.txt.said" "$T/said.txt"
says "$T/said.txt" "Set Addition 16 ft beside this job: the table is on the sheet, under Layouts. Nothing is written to either file."
says "$T/said.txt" "Took Addition 16 ft away from beside this job. Its file is as it was."

echo
echo "== The table the sheet showed, read from the screen, against what \`plumbline options\` gives =="
python3 -I - "$T/table.txt" "$T/cli.txt" "$T/away.txt" <<'PY'
import sys, re
def rows(path):
    out = []
    for line in open(path):
        parts = line.rstrip("\n").split("\t")
        if len(parts) >= 4 and parts[0] == "sheet" and parts[1].startswith("Layouts row | "):
            out.append(" ".join(parts[1][len("Layouts row | "):].replace(" | ", " ").split()))
    return out
def heading(path):
    for line in open(path):
        parts = line.rstrip("\n").split("\t")
        if len(parts) >= 4 and parts[0] == "sheet" and parts[1].startswith("Layouts heading | "):
            return parts[1][len("Layouts heading | "):]
shown, away = rows(sys.argv[1]), rows(sys.argv[3])
head = heading(sys.argv[1])
print("    the sheet's heading: " + head)
if head != "Addition 12 ft | Addition 16 ft | Addition 16 ft less Addition 12 ft":
    sys.exit("FAIL: the heading is not the two titles and the last less the first")
for want in ("Kitchen ~182.125 SF ~244.125 SF +62 SF", "all rooms ~550.25 SF ~612.25 SF +62 SF", "job cost ~$5,777.00 ~$7,137.00 +$1,360.00", "6 Wood and Plastics ~$4,320.00 ~$5,184.00 +$864.00", "9 Finishes ~$1,457.00 ~$1,953.00 +$496.00"):
    if want not in shown:
        sys.exit("FAIL: the sheet has no row reading: %s\n  it has: %s" % (want, "\n  ".join(shown)))
    print("    " + want)
# The command line's rows, each as one line (without its flags line and its heading).
cli = [" ".join(l.split()) for l in open(sys.argv[2]).read().splitlines() if l.strip() and not l.startswith("flags:") and " less " not in l]
missing = [c for c in cli if c not in shown]
extra = [s for s in shown if s not in cli]
if missing or extra:
    sys.exit("FAIL: the sheet and the command line differ\n  only on the command line: %s\n  only on the sheet: %s" % (missing, extra))
print("    all %d rows are the command line's rows, word for word" % len(shown))
if away:
    sys.exit("FAIL: after Take it away the table is still on the sheet: %s" % away)
print("    after Take it away: no row of the table is on the sheet")
PY

echo
echo "== Nothing was written: each file is byte for byte as it was =="
for f in addition-12 addition-16 nothing-yet; do cmp "$T/jobs/$f.json" "$T/$f.before"; done
echo "    addition-12.json, addition-16.json and nothing-yet.json are unchanged"
echo
echo "  PASS: in the window, the Layouts tab set Addition 16 ft beside Addition 12 ft, picked from a list of the job files in the folder: Kitchen 182.125 SF and 244.125 SF (+62 SF), job cost \$5,777.00 and \$7,137.00 (+\$1,360.00), every row the command line's; Take it away cleared it; no file changed"
