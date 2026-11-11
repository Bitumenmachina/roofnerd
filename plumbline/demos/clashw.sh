#!/usr/bin/env bash
# Fittings, connections and clashes in the application. A small job is made at the command
# line: four walls, a duct with a corner and a branch, an air handler over the duct's end,
# and a pipe drawn through the duct. Then, in the window, on the plan: the sheet's Clashes
# tab lists what is in the same space as what and the model marks where; a fitting is put
# on the corner and on the branch with one click each; the duct is joined to the air
# handler with one click, which takes that clash off the list; a clash picked on the sheet
# selects the run on the model. Then the saved job is read back.
# Run from the repository root:  demos/clashw.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job is made up.
# Every step fails the script if the program does not do what it says.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/clashw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/clashw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/clash}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
K=(--status Known --source "made-up plan")
made() { sed -E 's/^([a-z]+-[0-9]+).*/\1/' <<<"$1"; }

echo "== The job, made at the command line =="
"$P" new "$J" "Fitted and checked in the window (not a real job)"
S="$(made "$("$P" storey "$J" ground --elevation "0 ft" --height "9 ft" "${K[@]}")")"
WT="$(made "$("$P" type "$J" wall "wall 6 in" --thickness "6 in" "${K[@]}")")"
DT="$(made "$("$P" type "$J" duct "6 in round" --round "6 in" "${K[@]}")")"
PT="$(made "$("$P" type "$J" pipe "1 in" --round "1 in" "${K[@]}")")"
AIR="$(made "$("$P" system "$J" "Supply air" "Supply Air")")"
COLD="$(made "$("$P" system "$J" "Cold water" "Domestic Cold Water")")"
"$P" wall "$J" "$S" "$WT" --points "0,0 24,0 24,16 0,16" --closed --height "9 ft" "${K[@]}" >/dev/null
"$P" run "$J" "$S" duct "$DT" --points "2,2 10,2 10,14" --height "8 ft" --system "$AIR" "${K[@]}" >/dev/null
"$P" run "$J" "$S" duct "$DT" --points "6,2 6,10" --height "8 ft" --system "$AIR" "${K[@]}" >/dev/null
"$P" place "$J" "$S" equipment "Air handler" --at "2,2" --offset "6 ft" --size "2,2,4" --system "$AIR" "${K[@]}" >/dev/null
"$P" run "$J" "$S" pipe "$PT" --points "4,1 4,15" --height "8 ft" --system "$COLD" "${K[@]}" >/dev/null
# The duct's first 12 in are in the air handler, and the 1 in pipe crosses the 6 in duct at
# one height: 3 in + 1/2 in.
"$P" clashes "$J" | tee "$T/before.txt" | head -3 | sed 's/^/    /'
grep -q "^In the same space: 2" "$T/before.txt" || { echo "FAIL: wanted two clashes to begin with"; exit 1; }
cp "$J" "$T/before.json"

echo
echo "== The session: the Clashes tab, a fitting on the corner and on the branch, the duct joined to the air handler, a clash picked =="
# On the 1600 by 960 model window the plan is 32 pixels to the foot, and the plan's (0, 0),
# the outside corner of the walls' lines, is at (416, 736). The sheet is to its right.
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export VK_ICD_FILENAMES="$ICD" WGPU_BACKEND=vulkan PLUMBLINE_SHOTS="$OUT"
"$APP" "$J" > "$T/app.log" 2>&1 &
APP_PID=\$!
for i in \$(seq 1 60); do xdotool search --name "plumbline: model" >/dev/null 2>&1 && break; sleep 1; done
sleep 40   # the first frames are slow on a software renderer
xdotool windowfocus "\$(xdotool search --name 'plumbline: model' | head -1)"; sleep 1
shot() { xdotool key F12; sleep 12; }
# A click held for longer than a frame takes to draw, so the press and the release are
# seen in different frames however slow the renderer is.
click() { xdotool mousemove \$1 \$2; sleep 1.5; xdotool mousedown 1; sleep 0.7; xdotool mouseup 1; sleep 3; }
esc() { xdotool key Escape; sleep 2; }
click 53 189                            # the storey, Ground
xdotool key t; sleep 6                  # the plan
click 1970 73                           # the sheet's Clashes tab
xdotool mousemove 1000 850; sleep 1
shot                                    # 01 two clashes listed, and marked on the plan
click 101 665                           # Fitting where runs meet
xdotool mousemove 736 672; sleep 2      # the duct's corner, (10, 2)
shot                                    # 02 about to put a fitting on the corner
xdotool mousedown 1; sleep 0.7; xdotool mouseup 1; sleep 4
xdotool mousemove 1000 850; sleep 1
shot                                    # 03 an elbow
click 608 672                           # where the branch meets the duct's side, (6, 2)
xdotool mousemove 1000 850; sleep 1
shot                                    # 04 a tee, and the duct cut there
esc
click 113 692                           # Join a run to what it ends at
click 480 672                           # the duct's end in the air handler, (2, 2)
esc
xdotool mousemove 1000 850; sleep 1
shot                                    # 05 joined: one clash left
click 2000 205                          # that clash, on the sheet
xdotool mousemove 1000 850; sleep 1
shot                                    # 06 the pipe selected on the model
xdotool key t; sleep 6
shot                                    # 07 the same in 3D
xdotool key F2; sleep 3                 # save
kill \$APP_PID; wait \$APP_PID 2>/dev/null
SESSION
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06 07; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  14 pictures saved to $OUT (model and sheet, after each of 7 steps)"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
"$P" takeoff "$J" | tr -s ' ' > "$T/takeoff.txt"
"$P" clashes "$J" > "$T/after.txt"
says() { grep -qF -- "$2" "$3" || { echo "FAIL: $1: not said: $2"; cat "$3"; exit 1; }; printf '    %s\n' "$2"; }
# The duct is 8 + 12 + 8 = 28 LF still, in four pieces now that the tee has cut one. What
# was put in the window is Assumed, so the tee, the elbow and the cut duct are flagged.
says "the duct" "28.00 LF" "$T/takeoff.txt"
says "the elbow" "Bend 6 in round elbow: 1 EA" "$T/takeoff.txt"
says "the tee" "Tee 6 in round tee: 1 EA" "$T/takeoff.txt"
# Joined to the air handler, the supply air is open at the corner piece's far end and at
# the branch's far end, and nowhere does a run meet another with no fitting.
says "what is joined" "joined: 1 network; 2 open ends; 0 places" "$T/takeoff.txt"
# One clash left: the pipe through the duct, 3 in + 1/2 in. It rests on the duct, whose
# line was cut in the window, so it is flagged.
says "one clash left" "In the same space: 1 (1 to verify in the field)" "$T/after.txt"
says "which" "in each other by 3.50 in, at (4, 2, 8)" "$T/after.txt"
python3 - "$J" <<'PY'
import sys, json
data = json.load(open(sys.argv[1]))["data"]
kind = lambda e: e["attributes"].get("pl::class")
fittings = [e for e in data if kind(e) == "Fitting"]
for f in fittings:
    what = f["attributes"]["pl::placed"]["what"]
    if what["status"] != "Assumed" or "3D view" not in what["source"] or "is bought is not selected" not in what["source"]:
        print("FAIL: a fitting put in the window is not flagged as put there:", what); sys.exit(1)
ahu = next(e for e in data if kind(e) == "Equipment")
c = ahu["attributes"]["pl::connectors"]
if len(c) != 1 or c[0]["at"]["value"] != [0.0, 0.0, 24.0] or c[0]["at"]["status"] != "Assumed":
    print("FAIL: the air handler's connector is not where the duct ends, flagged:", c); sys.exit(1)
print("    2 fittings, each Assumed and saying which runs it joins; the air handler's connector at (0, 0, 24) in, Assumed")
PY
echo "  PASS: fittings and a connection were made in the window with one click each, the sheet listed the clashes and the model marked them, and the saved job says the same"
