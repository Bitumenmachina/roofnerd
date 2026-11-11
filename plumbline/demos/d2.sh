#!/usr/bin/env bash
# D2 demonstration, the part of it that runs: a roof solved from its footprint, in a 3D
# window, with the sheet in a second window that follows every edit.
# Run from the repository root:  demos/d2.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. On a desktop, run `plumbline-app <job>` and do
# the same by hand. Here it runs on a virtual display, which needs: Xvfb, xdotool, and a
# Vulkan driver (Mesa's software one will do). It says so and stops if one is missing.
# The job is made up. Every step fails the script if the program does not do what it says.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/d2.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/d2.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/d2}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT

echo "== The job: the sample, plus an L-shaped house solved from its footprint =="
"$P" sample "$T/job.json" >/dev/null
# 40 x 20 with a 20 x 20 wing on its west half, 6:12 all round, eaves at 10 ft.
"$P" roof "$T/job.json" house --points "100,0 140,0 140,20 120,20 120,40 100,40" --pitch 6:12 --eave-height "10 ft" --status Known --source "sample plan A-201"
"$P" check "$T/job.json"
cp "$T/job.json" "$T/before.json"
ROOF="$("$P" show "$T/job.json" | awk '$2=="house"{print $1}')"
PLANE1="$("$P" show "$T/job.json" | awk -v r="$ROOF" '$1==r{on=1} on&&$2=="plane"&&$3=="1"{print $1; exit}')"
surface() { "$P" show "$1" | awk -v p="$PLANE1" '$1==p{on=1} on&&/surface_area/{print $(NF-1); exit}'; }
echo "the roof is $ROOF; the plane on its south edge is $PLANE1, $(surface "$T/job.json") SF"

echo
echo "== The session: pick a plane, raise its pitch twice, move its eave, type a pitch, give it a condition, look at the plan, save =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export VK_ICD_FILENAMES="$ICD" WGPU_BACKEND=vulkan PLUMBLINE_SHOTS="$OUT"
"$APP" "$T/job.json" > "$T/app.log" 2>&1 &
APP_PID=\$!
for i in \$(seq 1 60); do xdotool search --name "plumbline: model" >/dev/null 2>&1 && break; sleep 1; done
sleep 40   # the first frames are slow on a software renderer
xdotool windowfocus "\$(xdotool search --name 'plumbline: model' | head -1)"; sleep 1
shot() { xdotool key F12; sleep 12; }
shot                                                    # 01 as opened
xdotool mousemove 940 470; sleep 1; xdotool click 1; sleep 3; xdotool mousemove 946 474; sleep 2
shot                                                    # 02 a plane picked with the mouse
xdotool key equal; sleep 3; xdotool key equal; sleep 3
shot                                                    # 03 its pitch raised twice: 6 to 8 in 12
xdotool key bracketright; sleep 3
shot                                                    # 04 its eave moved out 6 in
xdotool key p; sleep 2; xdotool type --delay 300 "9:12"; sleep 2
shot                                                    # 05 a pitch being typed
xdotool key Return; sleep 3
shot                                                    # 06 the typed pitch set
xdotool key c; sleep 3
shot                                                    # 07 a condition given to the surface
xdotool mousemove 2050 500; sleep 1; for i in \$(seq 1 16); do xdotool click 5; sleep 0.4; done; sleep 1
xdotool mousemove 1400 500; sleep 1; for i in \$(seq 1 9); do xdotool click 5; sleep 0.4; done; sleep 1
xdotool mousemove 946 474; sleep 1; xdotool key t; sleep 9
shot                                                    # 08 the same roof on plan; the sheet scrolled to its cost lines
xdotool key F2; sleep 3                                 # save
kill \$APP_PID; wait \$APP_PID 2>/dev/null
SESSION
chmod +x "$T/session.sh"
timeout 900 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06 07 08; do
  # Each picture has to be from this run, not left over from an earlier one.
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  16 pictures saved to $OUT (model and sheet, after each of 8 steps)"

echo
echo "== What the session left in the saved job =="
"$P" check "$T/job.json"
python3 - "$T/before.json" "$T/job.json" "$ROOF" "$PLANE1" <<'PY'
import sys, json
before, after, roof, plane1 = sys.argv[1:]
a = {e["path"]: e for e in json.load(open(before))["data"]}
b = {e["path"]: e for e in json.load(open(after))["data"]}
under = set(b[roof].get("children", {}).values()) | set(a[roof].get("children", {}).values())
changed = [p for p in list(a) + [p for p in b if p not in a] if a.get(p) != b.get(p)]
outside = [p for p in changed if p != roof and p not in under]
# A solve that makes a new line takes the job's next path number. That, and only that,
# may change on the job element.
job = "job-0001"
if job in outside:
    x, y = json.loads(json.dumps(a[job])), json.loads(json.dumps(b[job]))
    grew = y["attributes"]["pl::job"]["next_id"] - x["attributes"]["pl::job"]["next_id"]
    x["attributes"]["pl::job"]["next_id"] = y["attributes"]["pl::job"]["next_id"] = 0
    if x == y:
        outside.remove(job)
        print(f"  the job's path counter went up by {grew}: the solve made {grew} new line(s)")
print(f"  elements changed: {len(changed)}; outside the roof and what is solved from it: {', '.join(outside) or 'none'}")
if outside: print("FAIL: an edit in the 3D view changed something else"); sys.exit(1)
fp = b[roof]["attributes"]["pl::footprint"]
e1 = fp["edges"][0]
print(f"  footprint edge 1: {e1['value']} {e1['unit']}, {e1['status']}, source: {e1['source']}")
if e1["value"] != 9.0 or e1["status"] != "Assumed": print("FAIL: the typed pitch is not in the job as 9 in 12, Assumed"); sys.exit(1)
south = [p[1] for p in fp["points"]["value"][:2]]
print(f"  footprint south edge now at y = {south[0]} ft (it was 0: moved out 6 in)")
if south != [-0.5, -0.5]: print("FAIL: the eave did not move out 6 in"); sys.exit(1)
p1 = b[plane1]["attributes"]
cond = p1.get("pl::condition", {}).get("value")
lines = [l["description"] for l in p1.get("pl::cost", {}).get("lines", [])]
print(f"  {plane1}: condition {cond!r}, cost lines {lines}, pitch {p1['pl::pitch']['value']} in/12 {p1['pl::pitch']['status']}")
if cond != "Standing seam copper roof" or len(lines) != 2: print("FAIL: the condition and its lines are not on the plane"); sys.exit(1)
# The plane kept its path through four solves, and its area is the solved one:
# south edge at y = -0.5, 9:12 against 6:12 on the other edges.
sa = p1["pl::measure"]["surface_area"]
print(f"  {plane1}: surface {sa['value']} SF ({sa['status']}); it was {a[plane1]['attributes']['pl::measure']['surface_area']['value']} SF")
PY
echo "  PASS: every edit made in the window is in the saved job, Assumed, with its source"
