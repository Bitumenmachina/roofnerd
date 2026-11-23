#!/usr/bin/env bash
# Layouts, made and compared in the window (DECISIONS.md calls 156 and 161; the command line's half
# is demos/options.sh and its figures are worked by hand in STATUS.md, "Two layouts side by
# side"). Three jobs are made with the command line in one folder (A, "Addition 12 ft", B,
# "Addition 16 ft", and a third, left empty), and a file that is no job (notes.json) lies beside
# them. A is opened in the window. On the sheet's Layouts tab "Keep this as another layout" is
# pressed: the title offered is the job's own with "B", Enter keeps it, and the window writes
# the copy beside A (no file is copied by this script) and sets it beside A. It is kept again
# under A's own title, so that two layouts have one title and each column is headed by its file's
# name too. Both are taken away; "Set beside another layout" lists the jobs in the folder by
# their titles (notes.json is not one); Addition 16 ft is picked and set beside A; taken away.
# "Open another layout" lists the jobs; the kept copy is picked and opened in this window; with
# a change made and not saved, "Open another layout" says "Save first (Ctrl+S)" and opens
# nothing; saved, it opens A again. The window's tables are read from what it writes of the
# screen and set against `plumbline options` row by row; A's file and the two others never
# change, byte for byte; the kept copy is A's, with its title.
# Run from the repository root:  demos/layoutsw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard on a virtual display, which
# needs Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The jobs are made up
# (no real job data). Every step fails the script if the program does not do what it says.
#
# Worked by hand (STATUS.md): the Kitchen is 182.125 SF in A and 244.125 SF in B, +62 SF (B's
# addition is 4 ft deeper: 4 x 15.5); the job cost $5,777.00 and $7,137.00, +$1,360.00. The
# kept copy is A with another title: every difference in its table is "same".
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
echo '{"notes": "a file that is no job"}' > "$T/jobs/notes.json"
for f in addition-12 addition-16 nothing-yet; do cp "$T/jobs/$f.json" "$T/$f.before"; done
"$P" options "$T/jobs/addition-12.json" "$T/jobs/addition-16.json" | tr -s ' ' | sed 's/^ //' > "$T/cli.txt"
says "$T/cli.txt" "Kitchen ~182.125 SF ~244.125 SF +62 SF"
says "$T/cli.txt" "job cost ~\$5,777.00 ~\$7,137.00 +\$1,360.00"
cp "$T/jobs/addition-12.json" "$T/before.json"

echo
echo "== The session: A open; Keep this as another layout, twice; take them away; Set beside another layout (Addition 16 ft); Open another layout =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$T/jobs/addition-12.json"
sheet "Layouts"; park
shot                                     # 01 the Layouts tab, nothing set beside this job
sheet "Keep this as another layout"; sleep 3; park
cp "$T/map.txt" "$T/map-ask.txt"
shot                                     # 02 the title asked for: this job's, and B
xdotool key Return; sleep 10; park       # Enter keeps what is offered
cp "$T/map.txt" "$T/table-kept.txt"
cp "$T/jobs/addition-12-ft-b.json" "$T/b.kept"
shot                                     # 03 the copy kept and set beside this job
sheet "Keep this as another layout"; sleep 3
xdotool type --delay 250 "Addition 12 ft"; sleep 1   # A's own title: two layouts with one title
xdotool key Return; sleep 10; park
cp "$T/map.txt" "$T/table-alike.txt"
shot                                     # 04 three layouts, two with one title: each column headed by its file as well
sheet "Take it away"; sleep 5
sheet "Take it away"; sleep 6; park      # both taken away
sheet "Set beside another layout"; sleep 3; park
cp "$T/map.txt" "$T/map-beside-list.txt"
shot                                     # 05 the jobs in this job's folder, by their titles
sheet "Layout addition-16"; sleep 6; park
cp "$T/map.txt" "$T/table.txt"
shot                                     # 06 A and Addition 16 ft side by side
sheet "Take it away"; sleep 6; park
cp "$T/map.txt" "$T/away.txt"
sheet "Open another layout"; sleep 3; park
cp "$T/map.txt" "$T/map-open-list.txt"
shot                                     # 07 the jobs again, to open one in this window
sheet "Layout addition-12-ft-b"; sleep 14; park
shot                                     # 08 the kept copy is open in this window
on "Starter tool chest"; sleep 6              # a change not saved
sheet "Open another layout"; sleep 3; park
cp "$T/map.txt" "$T/map-grey.txt"
sheet "Layout addition-12"; sleep 5; park
shot                                     # 09 refused: Save first (Ctrl+S); nothing opened; the list grey
xdotool key F2; sleep 4                  # saved: the kept copy, which is the program's own to write
sheet "Layout addition-12"; sleep 14; park
cp "$T/map.txt" "$T/map-reopened.txt"
shot                                     # 10 A open again
close_window
SESSION
chmod +x "$T/session.sh"
timeout 2400 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a control it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04 05 06 07 08 09 10; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
[ -s "$T/map-reopened.txt" ] || { echo "FAIL: the session did not run to its end"; exit 1; }
echo "  20 pictures saved to $OUT (model and sheet, after each of 10 steps)"

echo
echo "== What the window said =="
cp "$T/map.txt.said" "$T/said.txt"
says "$T/said.txt" "Keep this as another layout: type its title, then Enter. Esc lets go."
says "$T/said.txt" "Kept a copy of this job as Addition 12 ft B, beside it, and set it beside this one: the table is on the sheet, under Layouts."
says "$T/said.txt" "Kept a copy of this job as Addition 12 ft, beside it, and set it beside this one"
says "$T/said.txt" "Set Addition 16 ft beside this job: the table is on the sheet, under Layouts. Nothing is written to either file."
says "$T/said.txt" "Took Addition 16 ft away from beside this job. Its file is as it was."
says "$T/said.txt" "Opened Addition 12 ft B. The other layouts are as they were; set one beside this to compare."
says "$T/said.txt" "Save first (Ctrl+S)."
says "$T/said.txt" "Opened Addition 12 ft. The other layouts are as they were; set one beside this to compare."
if grep -qiE 'tmp|\.json|beside::|Entry|layout_name' "$T/said.txt"; then echo "FAIL: the window said a file's ending or a program word"; grep -iE 'tmp|\.json|Entry' "$T/said.txt"; exit 1; fi

echo
echo "== The copy the window kept (no file was copied by this script): A with another title =="
[ -s "$T/b.kept" ] || { echo "FAIL: no copy was kept beside A"; ls "$T/jobs"; exit 1; }
"$P" check "$T/b.kept"
ls "$T/jobs" | tr '\n' ' '; echo
[ -s "$T/jobs/addition-12-ft.json" ] || { echo "FAIL: the second copy, kept under A's own title, is not beside A"; exit 1; }
python3 -I - "$T/addition-12.before" "$T/b.kept" <<'PY'
import sys
a, b = open(sys.argv[1]).read(), open(sys.argv[2]).read()
# The title is the one thing that differs: it is in the job's own record, once.
na, nb = a.count("Addition 12 ft"), b.count("Addition 12 ft B")
print("    A names its title %d times, the copy names its own %d times" % (na, nb))
if na < 1 or na != nb:
    sys.exit("FAIL: the copy does not name its title where A does")
if a.replace("Addition 12 ft", "Addition 12 ft B") != b:
    sys.exit("FAIL: the copy differs from A in more than its title")
print("    the copy is A word for word but for its title")
PY
cp "$T/jobs/addition-12-ft-b.json" "$T/b.after"
"$P" options "$T/jobs/addition-12.json" "$T/b.kept" > "$T/cli-kept.txt" 2>&1 || true
"$P" options "$T/jobs/addition-12.json" "$T/jobs/addition-12-ft-b.json" "$T/jobs/addition-12-ft.json" | sed 's/^/    /' > "$T/cli-alike.txt"

echo
echo "== The lists the window showed, from what it writes of the screen =="
python3 -I - "$T" <<'PY'
import sys
t = sys.argv[1]
def names(f):
    out = []
    for line in open(f):
        p = line.rstrip("\n").split("\t")
        if len(p) >= 4 and p[0] == "sheet" and p[1].startswith("Layout "):
            out.append(p[1][len("Layout "):])
    return sorted(out)
beside, opening = names(t + "/map-beside-list.txt"), names(t + "/map-open-list.txt")
print("    Set beside another layout lists: %s" % beside)
print("    Open another layout lists:       %s" % opening)
if "notes" in beside or "notes" in opening:
    sys.exit("FAIL: notes.json is no job and is in a list")
if beside != ["addition-12-ft", "addition-12-ft-b", "addition-16", "nothing-yet"] and beside != ["addition-12-ft", "addition-12-ft-b", "addition-16", "nothing-yet"]:
    sys.exit("FAIL: the list is not the jobs in the folder but this one")
if opening != beside:
    sys.exit("FAIL: Open another layout does not list the same jobs")
print("    only files that are jobs, and not the one open")
PY

echo
echo "== The tables the sheet showed, read from the screen, against what \`plumbline options\` gives =="
python3 -I - "$T" <<'PY'
import sys, subprocess
t = sys.argv[1]
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
def cli_rows(text):
    return [" ".join(l.split()) for l in text.splitlines() if l.strip() and not l.startswith("flags:") and " less " not in l]
# 1. A and its kept copy: every row the same, and the row that says what the job cost leaves out.
kept = rows(t + "/table-kept.txt")
print("    the kept copy's heading: " + heading(t + "/table-kept.txt"))
if heading(t + "/table-kept.txt") != "Addition 12 ft | Addition 12 ft B | Addition 12 ft B less Addition 12 ft":
    sys.exit("FAIL: the heading is not the two titles and the last less the first")
if not any(r.startswith("left out of the job cost") for r in kept):
    sys.exit("FAIL: the table has no row 'left out of the job cost': %s" % kept)
print("    " + [r for r in kept if r.startswith("left out of the job cost")][0])
if any(not (r.endswith("same") or r.endswith("none none same") or r.endswith("not worked")) for r in kept):
    sys.exit("FAIL: the copy is A with another title: every row reads same\n  " + "\n  ".join(kept))
cli = cli_rows(open(t + "/cli-kept.txt").read())
if [c for c in cli if c not in kept] or [k for k in kept if k not in cli]:
    sys.exit("FAIL: the sheet and the command line differ\n  only on the command line: %s\n  only on the sheet: %s" % ([c for c in cli if c not in kept], [k for k in kept if k not in cli]))
print("    all %d rows are the command line's rows, word for word" % len(kept))
# 2. Two layouts with one title: each column headed by its title and its file's name.
head = heading(t + "/table-alike.txt")
print("    three layouts: " + head)
want = "Addition 12 ft (addition-12) | Addition 12 ft B (addition-12-ft-b) | Addition 12 ft (addition-12-ft) | Addition 12 ft (addition-12-ft) less Addition 12 ft (addition-12)"
if head != want:
    sys.exit("FAIL: the heading under like titles is not each title and its file's name: wanted\n  " + want)
print("    where two layouts have one title, each column carries its file's name")
# 3. A and Addition 16 ft: the figures worked by hand, and the command line's rows.
shown, away = rows(t + "/table.txt"), rows(t + "/away.txt")
head = heading(t + "/table.txt")
print("    the sheet's heading: " + head)
if head != "Addition 12 ft | Addition 16 ft | Addition 16 ft less Addition 12 ft":
    sys.exit("FAIL: the heading is not the two titles and the last less the first")
for want in ("Kitchen ~182.125 SF ~244.125 SF +62 SF", "all rooms ~550.25 SF ~612.25 SF +62 SF", "job cost ~$5,777.00 ~$7,137.00 +$1,360.00", "6 Wood and Plastics ~$4,320.00 ~$5,184.00 +$864.00", "9 Finishes ~$1,457.00 ~$1,953.00 +$496.00"):
    if want not in shown:
        sys.exit("FAIL: the sheet has no row reading: %s\n  it has: %s" % (want, "\n  ".join(shown)))
    print("    " + want)
cli = cli_rows(open(t + "/cli.txt").read())
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
echo "== The list with changes not saved is grey; a job is named by its title, its file's name only where two titles are alike (call 169) =="
# The sheet window is 880 pixels wide at the right of the screen (the map's places are the
# screen's). The row of "Addition 16 ft" (its title alone) and of "Addition 12 ft" (two jobs have it)
# are read from picture 09, with changes not saved, and the first from picture 07, with none.
# Measured on a run of these steps: the title's darkest pixel is luminance 25 in 07 and 133 in 09; the
# stem's place (130 to 260 pixels across) has 0 pixels under 200 for Addition 16 ft and 121 for the
# two Addition 12 ft rows. Run on the picture from before this change (demos/captured/layoutsw/09-sheet.png
# at the commit before it: the title in full ink, a stem and the word job on every row) the first check
# fails (darkest 25, wanted over 100) and the second fails (161 pixels where 0 are wanted).
python3 -I - "$OUT" "$T/map-grey.txt" "$T/map-open-list.txt" <<'PY'
import sys
from PIL import Image
out, mapfile, mapfile7 = sys.argv[1], sys.argv[2], sys.argv[3]
def row_y(name, mapfile=mapfile):
    for line in open(mapfile):
        p = line.rstrip("\n").split("\t")
        if len(p) >= 4 and p[0] == "sheet" and p[1] == name:
            return int(float(p[3]))
sixteen, twelve, sixteen7 = row_y("Layout addition-16"), row_y("Layout addition-12"), row_y("Layout addition-16", mapfile7)
if sixteen is None or twelve is None or sixteen7 is None:
    sys.exit("FAIL: the rows are not on the map")
def darkest(name, y):
    im = Image.open("%s/%s-sheet.png" % (out, name)).convert("L")
    return min(im.getpixel((x, yy)) for x in range(40, 128) for yy in range(y - 8, y + 9))
def stem_ink(name, y):
    im = Image.open("%s/%s-sheet.png" % (out, name)).convert("L")
    return sum(1 for x in range(130, 260) for yy in range(y - 8, y + 9) if im.getpixel((x, yy)) < 200)
print("    the title's darkest pixel: no changes (07) %d, changes not saved (09) %d" % (darkest("07", sixteen7), darkest("09", sixteen)))
print("    a stem beside the title: Addition 16 ft %d pixels, Addition 12 ft %d pixels" % (stem_ink("09", sixteen), stem_ink("09", twelve)))
# Grey is told against the same title when it can be opened, not against a fixed number: call
# 178 (item 8: nothing fainter than can be read) darkened the faintest ink, and the grey title's
# darkest pixel went from 133 to 100. It must still be lighter than the open title by 60 or more
# (25 against 100 now; 25 against 25 where the list is not grey, which fails).
ok = darkest("07", sixteen7) < 90 and darkest("09", sixteen) >= darkest("07", sixteen7) + 60 and stem_ink("09", sixteen) == 0 and stem_ink("09", twelve) > 10
print("    grey when nothing can be opened, a file's name only beside a title that is not alone: %s" % ("as wanted" if ok else "FAIL"))
sys.exit(0 if ok else 1)
PY

echo
echo "== No file was written over: A and the two others are byte for byte as they were =="
for f in addition-12 addition-16 nothing-yet; do cmp "$T/jobs/$f.json" "$T/$f.before"; done
echo "    addition-12.json, addition-16.json and nothing-yet.json are unchanged (the kept copy was written by the window, and saved again with the starter items: its own to change)"
echo
echo
echo "== A thing to do first is blue (call 178) =="
python3 -W ignore -I "$PWD/demos/cardcolour.py" "$OUT/09-model.png" blue
echo "  PASS: in the window, the Layouts tab kept A as another layout under a title it offered (a copy of A with that title, written beside it by the window and set beside A), and again under A's own title (each column headed by its file's name too); Set beside another layout listed the jobs in the folder by their titles and not notes.json, and set Addition 16 ft beside A: Kitchen 182.125 SF and 244.125 SF (+62 SF), job cost \$5,777.00 and \$7,137.00 (+\$1,360.00), the row 'left out of the job cost' there, every row the command line's; Open another layout opened the kept copy in the window, said Save first (Ctrl+S) with a change not saved and opened A when saved; no job file but the copies changed"
