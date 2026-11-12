#!/usr/bin/env bash
# D5 demonstration in the application: the sheet's Estimate tab (the recap, the cost by
# division, every cost line in its column, three suppliers side by side); on the sheet itself the job's overhead
# and profit typed on their rows, a unit cost and a stretch-out typed on their lines and a
# labor line switched off; and in the model window an element's material changed, with
# the sheet following.
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
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
wheel() { xdotool mousemove 2000 600; sleep 1; for i in \$(seq 1 \$1); do xdotool click \$2; sleep 0.3; done; sleep 3; xdotool mousemove 800 860; sleep 1; focus; }
shot                                                    # 01 as opened
sheet "Estimate"                                        # the sheet's Estimate tab
park
shot                                                    # 02 the recap first: no price, and why; the cost by division under it
wheel DOWN1 5
shot                                                    # 03 the sheet: every line in its column, under its division
wheel DOWN2 5
shot                                                    # 04 at the foot, the suppliers, side by side
sheet_at "Overhead percent" 4; sleep 2                  # back up to the recap at the top: overhead, typed where it stands
xdotool type --delay 300 "12"; sleep 2
shot                                                    # 05 the overhead being typed, on its row of the recap
xdotool key Return; sleep 4
sheet_at "Profit percent"; sleep 2; xdotool type --delay 300 "8"; sleep 2; xdotool key Return; sleep 4
shot                                                    # 06 overhead and profit entered: the price
sheet_at "Unit cost of Underlayment on south"; sleep 2  # down to the lines: a price, typed on its line
xdotool type --delay 300 "0.95"; sleep 2
shot                                                    # 07 a unit cost being typed on its line
xdotool key Return; sleep 4
sheet_at "Stretch-out of Headwall flashing on headwall"; sleep 2   # down to the headwall: its flashing's stretch-out
xdotool type --delay 300 "30 in"; sleep 2
shot                                                    # 08 a stretch-out being typed on its line
xdotool key Return; sleep 4
xdotool mousemove 900 470; sleep 1; xdotool click 1; sleep 4          # the south plane, in the model
park
on "Material"; sleep 2; xdotool type --delay 300 "Copper, 20 oz"; sleep 2
shot                                                    # 09 its material being typed
xdotool key Return; sleep 4
shot                                                    # 10 the material changed: its panels and the recap repriced
sheet_at "Labor of Install panels on south" 4           # its labor line, switched off on the sheet
park
shot                                                    # 11 its labor line switched off
sheet_at "Overhead percent" 4; sleep 2; xdotool type --delay 300 "ten"; sleep 2; xdotool key Return; sleep 4
shot                                                    # 12 an overhead that is not a number is refused, and says why
xdotool key F2; sleep 3                                 # save
close_window
SESSION
sed -i -e "s/DOWN1/${DOWN1:-9}/" -e "s/DOWN2/${DOWN2:-40}/g" "$T/session.sh"
chmod +x "$T/session.sh"
timeout 1200 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06 07 08 09 10 11 12; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  24 pictures saved to $OUT (model and sheet, after each of 12 steps)"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
"$P" estimate "$J" | tr -s ' ' > "$T/sheet.txt"
says() { grep -qF -- "$2" "$T/sheet.txt" || { echo "FAIL: $1: the saved job's estimate does not say: $2"; cat "$T/sheet.txt"; exit 1; }; printf '    %s\n' "$2"; }
# The south plane made 20 oz copper in the window: 461.1890 SF x 1.25 = 576.4863 lb x 9.00 =
# 5,188.38 (it was 4,150.70). Assumed: what is typed in the window has no source but the window.
says "the south plane's panels, repriced" "Standing seam panels ~576.4863 lb \$9.00/lb ~\$5,188.38"
# The south plane's underlayment priced on its line of the sheet, $0.90 to $0.95/SF:
# 419.2627 SF is 398.30, up 20.96.
says "a price typed on its line" "Underlayment 419.26 SF ~\$0.95/SF ~\$398.30"
# The headwall flashing's stretch-out changed on its line, 24 in to 30 in: 12 LF by 30 in
# is 30 SF, at 1.25 lb/SF 37.5 lb, at 9.00: 337.50, up 67.50.
says "a stretch-out typed on its line" "Headwall flashing ~37.50 lb \$9.00/lb ~\$337.50"
# Material: 8,628.10 + 1,037.68 + 20.96 + 67.50 = 9,754.24.
says "the material row" "Material ~\$9,754.24 6 lines"
# Its labor line switched off: Labor 2,147.97 - 1,207.48 = 940.49.
says "the labor row" "Labor ~\$940.49 2 lines ~13.0623 hr"
says "the line is still on the sheet" "Install panels 16.7705 hr \$72.00/hr off (\$1,207.48)"
# Job cost: 28,442.43 + 1,037.68 + 20.96 + 67.50 - 1,207.48 = 28,361.09.
says "the job cost" "Job cost ~\$28,361.09 14 lines"
# Overhead 12 %: 3,403.3308 -> 3,403.33. Profit 8 % of 31,764.42: 2,541.1536 -> 2,541.15.
# Price: 28,361.09 + 3,403.33 + 2,541.15 = 34,305.57.
says "overhead, typed on the sheet" "Overhead at ~12.00 % ~\$3,403.33"
says "profit, typed on the sheet" "Profit at ~8.00 % ~\$2,541.15"
says "the price" "PRICE ~\$34,305.57"
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
headwall = b["headwall-0021"]["attributes"]
flashing = next(l for l in headwall["pl::cost"]["lines"] if l["description"] == "Headwall flashing")
underlay = next(l for l in south["pl::cost"]["lines"] if l["description"] == "Underlayment")
ok = changed == ["headwall-0021", "job-0001", "plane-0003"]
ok = ok and underlay["unit_cost"]["value"] == 0.95 and underlay["unit_cost"]["status"] == "Assumed"
ok = ok and flashing["stretch_out"]["value"] == 30 and flashing["stretch_out"]["unit"] == "in" and flashing["stretch_out"]["status"] == "Assumed"
ok = ok and made["value"] == "Copper, 20 oz" and made["status"] == "Assumed" and "set in the window" in made["source"]
ok = ok and markup["overhead"]["value"] == 12 and markup["overhead"]["status"] == "Assumed" and markup["profit"]["value"] == 8
ok = ok and install.get("off") is True
print("    in the file: the south plane is made of", made["value"], made["status"], "(" + made["source"] + ")")
print("    in the file: overhead", markup["overhead"]["value"], markup["overhead"]["status"], "and profit", markup["profit"]["value"], markup["profit"]["status"])
print("    in the file: its labor line is off:", install.get("off"))
print("    in the file: the underlayment's price", underlay["unit_cost"]["value"], underlay["unit_cost"]["status"], "and the flashing's stretch-out", flashing["stretch_out"]["value"], flashing["stretch_out"]["unit"], flashing["stretch_out"]["status"])
sys.exit(0 if ok else 1)
PY
echo "  PASS: the estimate was read on the sheet; overhead, profit, a unit cost and a stretch-out were typed on their own lines of it; a labor line was switched off on it; a material was changed in the model window; and the saved job has them, flagged"
