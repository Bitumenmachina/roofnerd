#!/usr/bin/env bash
# The light (call 184, step 2): the same views of the same made-up whole house, to be taken
# on a build before a piece of the light is switched on and on a build after it, and set
# side by side. Pictures: `01` the whole house from the street, `02` close on a window, `03`
# close on an eave, `04` the plan; with SUN=1 (a build that has the Sun control) `05` the sun
# in the morning, `06` in the late afternoon and `07` a winter noon, `08` the sun low in the morning, the same street view.
# Run from the repository root:  [PLUMBLINE_BIN=<folder>] [SUN=1] demos/lightw.sh [folder for the pictures]
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "demos/lightw.sh needs $tool, which is not installed"; exit 2; }
done
ICD="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
[ -e "$ICD" ] || { echo "demos/lightw.sh needs a Vulkan driver; none at $ICD (set VK_ICD_FILENAMES)"; exit 2; }
if [ -n "${PLUMBLINE_BIN:-}" ]; then echo "using the program in $PLUMBLINE_BIN, not building"; else cargo build --quiet; fi
P="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline"; APP="${PLUMBLINE_BIN:-$PWD/target/debug}/plumbline-app"
OUT="${1:-$PWD/demos/captured/light}"; mkdir -p "$OUT"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
J="$T/job.json"
"$P" sixteen "$J" >/dev/null
touch "$T/start"
cat > "$T/session.sh" <<SESSION
#!/bin/bash
export APP="$APP" T="$T" OUT="$OUT" ICD="$ICD"
. "$PWD/demos/window.sh"
wheel() { local i; for i in \$(seq 1 "\$2"); do xdotool click "\$1"; drawn_for 2 2; done; sleep 6; }
open_window "$J"
on "Show panel Job"; on "Show panel Details"; on "Show panel Tools"
park
shot                                    # 01 the whole house from the street
xdotool mousemove 775 640; drawn_for 2 2; wheel 4 5
xdotool mousemove 1000 880; drawn_for 2 2
shot                                    # 02 close on a window
xdotool key f; drawn_for 3 8
xdotool mousemove 700 458; drawn_for 2 2; wheel 4 5
xdotool mousemove 1000 880; drawn_for 2 2
shot                                    # 03 close on an eave
xdotool key f; drawn_for 3 8
xdotool key t; drawn_for 3 8
park
shot                                    # 04 the plan
xdotool key t; drawn_for 3 8
if [ -n "${SUN:-}" ]; then
  on "Sun"
  park
  shot                                  # 05 the sun in the morning (the card open: a mid-morning in spring)
  for i in 1 2 3 4 5 6 7; do on "Later"; done   # 5 in the afternoon
  park
  shot                                  # 06 late afternoon
  for i in 1 2 3 4 5 6 7 8; do on "Month after"; done   # December
  for i in 1 2 3 4 5; do on "Earlier"; done      # noon
  park
  shot                                  # 07 a winter noon
  for i in 1 2 3 4 5 6 7 8; do on "Month before"; done   # April again
  for i in 1 2 3 4 5 6; do on "Earlier"; done            # 6 in the morning: the sun low in the east, in front of the eye
  park
  shot                                  # 08 the sun low, where the glow is
  # The hour stops at 6 in the morning and 6 at night, and says so (call 193, H).
  on "Earlier"
  park
  shot                                  # 09 6:00 am, Earlier pressed: it says it goes no earlier
  for i in 1 2 3 4 5 6 7 8 9 10 11 12; do on "Later"; done   # 6 at night
  on "Later"
  park
  shot                                  # 10 6:00 pm, Later pressed: it says it goes no later
fi
close_window
SESSION
chmod +x "$T/session.sh"
timeout 1500 xvfb-run -a -s "-screen 0 2500x980x24" "$T/session.sh" || true
if [ -s "$T/missing.txt" ]; then echo "FAIL: the session stopped: a button it was to click was not on screen"; cat "$T/missing.txt"; exit 1; fi
if grep -q "panicked" "$T/app.log"; then echo "FAIL: the program panicked"; grep -A3 panicked "$T/app.log" | head; exit 1; fi
for n in 01 02 03 04; do [ "$OUT/$n-model.png" -nt "$T/start" ] || { echo "FAIL: no picture $n from this run"; exit 1; }; done
if [ -n "${SUN:-}" ]; then
  for n in 05 06 07 08 09 10; do [ "$OUT/$n-model.png" -nt "$T/start" ] || { echo "FAIL: no picture $n from this run"; exit 1; }; done
  grep -qF "The sun card goes no earlier than 6:00 am." "$T/map.txt.said" || { echo "FAIL: Earlier at 6:00 am said nothing"; cat "$T/map.txt.said"; exit 1; }
  grep -qF "The sun card goes no later than 6:00 pm." "$T/map.txt.said" || { echo "FAIL: Later at 6:00 pm said nothing"; cat "$T/map.txt.said"; exit 1; }
  echo "    the Sun's hour stops at 6:00 am and 6:00 pm, and says so"
fi
echo "  pictures saved to $OUT"
