#!/usr/bin/env bash
# D5 demonstration in the application: the sheet's Estimate tab (every cost line in its
# column, the recap, the suppliers side by side), and in the model window the job's
# overhead and profit typed, an element's material changed and its labor line switched
# off, with the sheet following.
# Run from the repository root:  demos/d5w.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job, every price,
# every rate and every supplier are made up. Every step fails the script if the program
# does not do what it says. The figures expected are worked by hand in demos/d5.sh and in
# the comments here.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/d5w.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/d5w.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/d5}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"

echo "== The job: the made-up priced job, with two general-conditions lines typed at the command line =="
"$P" priced "$J" >/dev/null
"$P" line "$J" job-0001 "Dumpster" 2 --class general >/dev/null
"$P" set "$J" job-0001 unit_cost 650 --line "Dumpster" --status Known --source "made-up price" >/dev/null
"$P" line "$J" job-0001 "Permit" 1 --class general >/dev/null
"$P" set "$J" job-0001 unit_cost '$1,850' --line "Permit" --status Known --source "made-up price" >/dev/null
"$P" check "$J"
"$P" estimate "$J" | sed -n '/THE RECAP/,/PRICE/p'
cp "$J" "$T/before.json"

echo
echo "== The session: the Estimate tab; overhead and profit typed; the south plane's material changed; its labor switched off; save =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export VK_ICD_FILENAMES="$ICD" WGPU_BACKEND=vulkan PLUMBLINE_SHOTS="$OUT"
"$APP" "$J" > "$T/app.log" 2>&1 &
APP_PID=\$!
for i in \$(seq 1 60); do xdotool search --name "plumbline: model" >/dev/null 2>&1 && break; sleep 1; done
sleep 40   # the first frames are slow on a software renderer
focus() { xdotool windowfocus "\$(xdotool search --name 'plumbline: model' | head -1)"; sleep 1; }
wheel() { xdotool mousemove 2000 600; sleep 1; for i in \$(seq 1 \$1); do xdotool click \$2; sleep 0.3; done; sleep 3; xdotool mousemove 800 860; sleep 1; focus; }
focus
shot() { xdotool key F12; sleep 12; }
shot                                                    # 01 as opened
xdotool mousemove TABX 78; sleep 1; xdotool click 1; sleep 4          # the sheet's Estimate tab
xdotool mousemove 800 860; sleep 1; focus
shot                                                    # 02 the sheet: every line in its column
wheel DOWN1 5
shot                                                    # 03 the recap: no price, and why
wheel DOWN2 5
shot                                                    # 04 the suppliers, side by side
wheel DOWN2 4
xdotool key o; sleep 2; xdotool type --delay 300 "12"; sleep 2
shot                                                    # 05 the overhead being typed
xdotool key Return; sleep 4
xdotool key r; sleep 2; xdotool type --delay 300 "8"; sleep 2; xdotool key Return; sleep 4
shot                                                    # 06 overhead and profit entered: the price
xdotool mousemove 900 470; sleep 1; xdotool click 1; sleep 4          # the south plane, in the model
xdotool mousemove 800 860; sleep 1
xdotool key n; sleep 2; xdotool type --delay 300 "Copper, 20 oz"; sleep 2
shot                                                    # 07 its material being typed
xdotool key Return; sleep 4
shot                                                    # 08 the material changed: its panels and the recap repriced
xdotool key l; sleep 4
shot                                                    # 09 its labor line switched off
xdotool key o; sleep 2; xdotool type --delay 300 "ten"; sleep 2; xdotool key Return; sleep 4
shot                                                    # 10 an overhead that is not a number is refused, and says why
xdotool key F2; sleep 3                                 # save
kill \$APP_PID; wait \$APP_PID 2>/dev/null
SESSION
sed -i -e "s/TABX/${TABX:-1810}/" -e "s/DOWN1/${DOWN1:-8}/" -e "s/DOWN2/${DOWN2:-6}/g" "$T/session.sh"
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06 07 08 09 10; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  20 pictures saved to $OUT (model and sheet, after each of 10 steps)"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
"$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"
says() { grep -qF -- "$2" "$T/sheet.txt" || { echo "FAIL: $1: the saved job's estimate does not say: $2"; cat "$T/sheet.txt"; exit 1; }; printf '    %s\n' "$2"; }
# The south plane made 20 oz copper in the window: 461.1890 SF x 1.25 = 576.4863 lb x 9.00 =
# 5,188.38 (it was 4,150.70). Assumed: what is typed in the window has no source but the window.
says "the south plane's panels, repriced" "Standing seam panels ~576.49 lb \$9.00/lb ~\$5,188.38"
# Material: 8,628.10 + 1,037.68 = 9,665.78.
says "the material row" "Material ~\$9,665.78 6 lines"
# Its labor line switched off: Labor 2,147.97 - 1,207.48 = 940.49.
says "the labor row" "Labor ~\$940.49 2 lines ~13.0623 hr"
says "the line is still on the sheet" "Install panels 16.7705 hr \$72.00/hr off (\$1,207.48)"
# Job cost: 28,442.43 + 1,037.68 - 1,207.48 = 28,272.63.
says "the job cost" "Job cost ~\$28,272.63 14 lines"
# Overhead 12 %: 3,392.7156 -> 3,392.72. Profit 8 % of 31,665.35: 2,533.228 -> 2,533.23.
# Price: 28,272.63 + 3,392.72 + 2,533.23 = 34,198.58.
says "overhead, typed in the window" "Overhead at ~12.00 % ~\$3,392.72"
says "profit, typed in the window" "Profit at ~8.00 % ~\$2,533.23"
says "the price" "PRICE ~\$34,198.58"
python3 - "$T/before.json" "$J" <<'PY'
import sys, json
a = {e["path"]: e for e in json.load(open(sys.argv[1]))["data"]}
b = {e["path"]: e for e in json.load(open(sys.argv[2]))["data"]}
changed = sorted(p for p in a if a[p] != b[p])
print("    elements changed in the file:", ", ".join(changed))
south, job = b["plane-0003"]["attributes"], b["job-0001"]["attributes"]
made = south["pl::material"]
markup = job["pl::markup"]
install = next(l for l in south["pl::cost"]["lines"] if l["description"] == "Install panels")
ok = changed == ["job-0001", "plane-0003"]
ok = ok and made["value"] == "Copper, 20 oz" and made["status"] == "Assumed" and "3D view" in made["source"]
ok = ok and markup["overhead"]["value"] == 12 and markup["overhead"]["status"] == "Assumed" and markup["profit"]["value"] == 8
ok = ok and install.get("off") is True
print("    in the file: the south plane is made of", made["value"], made["status"], "(" + made["source"] + ")")
print("    in the file: overhead", markup["overhead"]["value"], markup["overhead"]["status"], "and profit", markup["profit"]["value"], markup["profit"]["status"])
print("    in the file: its labor line is off:", install.get("off"))
sys.exit(0 if ok else 1)
PY
echo "  PASS: the estimate was read on the sheet, overhead, profit and a material were typed in the window, a labor line was switched off, and the saved job has them, flagged"
