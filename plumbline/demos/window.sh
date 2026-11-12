# What every window demonstration does with the real program: start it, find a button
# by the word on it, move the real pointer there and press the real button, type, and
# have the program save a picture of both windows.
#
# Sourced by a session script that runs under xvfb-run, with these set: APP (the
# program), T (a scratch folder), OUT (where the pictures go), ICD (the Vulkan driver).
#
# Where a button is comes from the program itself (PLUMBLINE_MAP): it writes the place
# of everything on screen that can be clicked, by the word on it. A button that is not
# on screen stops the session and is named in $T/missing.txt, which fails the
# demonstration. A place on the plan or on the model is still given in pixels: the plan
# is drawn about the middle of the window, at the scale the script says.
export VK_ICD_FILENAMES="$ICD" WGPU_BACKEND=vulkan PLUMBLINE_SHOTS="$OUT" PLUMBLINE_MAP="$T/map.txt"

focus() { xdotool windowfocus "$(xdotool search --name 'plumbline: model' | head -1)"; sleep 1; }
open_window() {
  "$APP" "$@" > "${LOG:-$T/app.log}" 2>&1 &
  APP_PID=$!
  for i in $(seq 1 60); do xdotool search --name "plumbline: model" >/dev/null 2>&1 && break; sleep 1; done
  sleep 40   # the first frames are slow on a software renderer
  focus
}
close_window() { kill "$APP_PID"; wait "$APP_PID" 2>/dev/null; }
# A picture of both windows. The pointer stays where it is: what it is over is part of
# the picture. `park` puts it on the model, clear of every panel and of the building.
shot() { xdotool key F12; sleep 12; }
park() { xdotool mousemove 1000 800; sleep 1; }
# A click held for longer than a frame takes to draw.
click() { xdotool mousemove "$1" "$2"; sleep 1.5; xdotool mousedown 1; sleep 0.7; xdotool mouseup 1; sleep 3; }
where() { awk -F'\t' -v w="$1" -v n="$2" '$1==w && $2==n {print $3, $4}' "$PLUMBLINE_MAP" 2>/dev/null; }
found() {
  local at="" try
  for try in 1 2 3 4 5 6; do at="$(where "$1" "$2")"; [ -n "$at" ] && break; sleep 2; done
  if [ -z "$at" ]; then
    { echo "not on screen in the $1 window: $2"; echo "what is:"; awk -F'\t' -v w="$1" '$1==w {print "  " $2}' "$PLUMBLINE_MAP"; } >> "$T/missing.txt"
    close_window; exit 1
  fi
  echo "$at"
}
# A button, a row or a field of the model window, by the word on it.
on() { local at; at="$(found model "$1")" || exit 1; click $at; }
# The same, in the sheet window. The keyboard goes back to the model window after.
sheet() { local at; at="$(found sheet "$1")" || exit 1; click $at; focus; }
esc() { xdotool key Escape; sleep 2; }
# Typed, then Enter. With a second word, a picture is taken before the Enter.
typed() { xdotool type --delay 250 -- "$1"; sleep 1; if [ -n "${2:-}" ]; then shot; fi; xdotool key Return; sleep 3; }
# A tool taken in hand; with a second word, what it draws with, typed into its fields
# (a comma goes on to the next field, as Tab does).
tool() { on "$1"; sleep 2; if [ -n "${2:-}" ]; then typed "$2" "${3:-}"; fi; }
# A length: the pointer shows the way, the first digit opens the field, and the rest is
# typed once the field is up (on a software renderer the frame that draws a new field is
# slow, and keys typed during it are lost).
len() { xdotool mousemove "$1" "$2"; sleep 2; xdotool type "${3:0:1}"; sleep 3; xdotool type --delay 250 "${3:1}"; sleep 1; if [ -n "${4:-}" ]; then shot; fi; xdotool key Return; sleep 4; }
# A laser in keyboard mode: the reading typed in one burst, then the key it is set to send.
laser() { xdotool type --delay 20 -- "$1"; xdotool key "$2"; sleep 4; }
