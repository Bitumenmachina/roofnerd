#!/usr/bin/env bash
# D4 demonstration in the application: the job priced both ways on the sheet's Money tab,
# and prices typed in the window on what is selected, with the sheet following.
# Run from the repository root:  demos/d4w.sh [folder for the pictures]
#
# It drives the real program with a real pointer and keyboard and has the program save a
# picture of both windows after each step. Here it runs on a virtual display, which needs
# Xvfb, xdotool and a Vulkan driver (Mesa's software one will do). The job and every price
# are made up. Every step fails the script if the program does not do what it says. The
# figures expected are worked by hand in the comments, and in demos/d4.sh.
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/d4w.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/d4w.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
cargo build --quiet
P="$PWD/target/debug/plumbline"; APP="$PWD/target/debug/plumbline-app"
OUT="${1:-$PWD/demos/captured/d4}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"

echo "== The job: the sample roof, priced at the command line as demos/d4.sh prices it =="
"$P" sample "$J" >/dev/null
for plane in plane-0003 plane-0004 plane-0017; do
  "$P" set "$J" "$plane" allowance '$28.00/SF' --status Known --source "made-up budget rate" >/dev/null
done
"$P" set "$J" headwall-0021 allowance 40 --status Known --source "made-up budget rate" >/dev/null
"$P" set "$J" plane-0003 unit_cost 31.50 --status Known --source "made-up quote" >/dev/null
"$P" set "$J" plane-0017 unit_cost 31.50 --status Known --source "made-up quote" >/dev/null
"$P" set "$J" headwall-0021 unit_cost 18.25 --status Known --source "made-up quote" >/dev/null
"$P" check "$J"
"$P" price "$J" | tail -9
cp "$J" "$T/before.json"

echo
echo "== The session: the Money tab; the east plane picked on it and its line priced; the north plane given an allowance; save =="
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
open_window "$J"
shot                                                    # 01 as opened
sheet "Money"                                           # the sheet's Money tab
park
shot                                                    # 02 the job priced both ways, and reconciled
sheet "east"                                            # the east plane's row
park
shot                                                    # 03 the east plane picked from the sheet
on "Unit cost"; sleep 2; xdotool type --delay 300 "31.50"; sleep 2
shot                                                    # 04 its line's unit cost being typed
xdotool key Return; sleep 4
shot                                                    # 05 the price set: its detail is whole, and the totals moved
sheet "north"                                           # the north plane's row
park
on "Allowance, \$/SF"; sleep 2; xdotool type --delay 300 "28"; sleep 2; xdotool key Return; sleep 4
shot                                                    # 06 the north plane given an allowance
on "Unit cost"; sleep 2; xdotool type --delay 300 "nonsense"; sleep 2; xdotool key Return; sleep 4
shot                                                    # 07 a price that is not a price is refused, and says why
xdotool key F2; sleep 3                                 # save
close_window
SESSION
chmod +x "$T/session.sh"
timeout 900 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
ls "$OUT" | tr '\n' ' '; echo
for n in 01 02 03 04 05 06 07; do
  [ "$OUT/$n-model.png" -nt "$T/before.json" ] && [ "$OUT/$n-sheet.png" -nt "$T/before.json" ] || { echo "FAIL: no picture $n of both windows from this run"; exit 1; }
done
echo "  14 pictures saved to $OUT (model and sheet, after each of 7 steps)"

echo
echo "== What the session left in the saved job =="
"$P" check "$J"
"$P" price "$J" > "$T/price.txt"
says() { grep -qF -- "$2" "$T/price.txt" || { echo "FAIL: $1: the saved job's price does not say: $2"; cat "$T/price.txt"; exit 1; }; printf '    %s\n' "$2"; }
# The east plane, priced in the window: 251.5576 SF x 31.50 = 7,924.0644 -> $7,924.06.
# It is marked Assumed: a figure typed in the window has no source but the window.
# Against its allowance, 7,043.61: 880.45 over, 12.50 %.
says "the east plane's detail, typed in the window" "detail      ~\$7,924.06, which is ~\$31.50 per SF"
says "" "the detail is ~\$880.45 over the allowance (~12.50 %)"
# The north plane's allowance, typed in the window: 419.2627 SF x 28 = 11,739.36, Assumed.
says "the north plane's allowance, typed in the window" "allowance   ~\$11,739.36 at ~\$28.00 per SF"
# Allowances now: 22,033.70 + 11,739.36 = 33,773.06 over five elements.
says "the allowance total" "allowances   ~\$33,773.06 over 5 elements"
# Detail now: 16,761.85 + 7,924.06 = 24,685.91 over four elements.
says "the detail total" "detail       ~\$24,685.91 over 4 elements"
# Like for like, over the four with both: allowances 14,990.09 + 7,043.61 = 22,033.70;
# detail 24,685.91; 2,652.21 over; 2,652.21 / 22,033.70 = 12.04 %.
says "like for like" "like for like, over the 4 elements that have both: allowances ~\$22,033.70, detail ~\$24,685.91: the detail is ~\$2,652.21 over (~12.04 %)"
# The price that was not a price changed nothing: the north plane's line still has none.
python3 - "$J" <<'PY'
import sys, json
d = {e["path"]: e for e in json.load(open(sys.argv[1]))["data"]}
east = d["plane-0004"]["attributes"]["pl::cost"]["lines"][0]["unit_cost"]
north = d["plane-0005"]["attributes"]
ok = east["value"] == 31.5 and east["status"] == "Assumed" and "set in the window" in east["source"]
ok = ok and north["pl::allowance"]["rate"]["status"] == "Assumed" and north["pl::cost"]["lines"][0]["unit_cost"]["value"] is None
print("    in the file: the east plane's unit cost is", east["value"], east["status"], "(" + east["source"] + ")")
print("    in the file: the north plane's line still has no price:", north["pl::cost"]["lines"][0]["unit_cost"]["value"])
sys.exit(0 if ok else 1)
PY
echo "  PASS: the job was priced both ways on the sheet, two prices were typed in the window, and the saved job has them, flagged"
