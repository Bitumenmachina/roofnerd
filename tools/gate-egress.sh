#!/usr/bin/env bash
# ── The egress detector ────────────────────────────────────────────────────
# This program makes no network calls. What actually ENFORCES that is the
# content security policy in src-tauri/tauri.conf.json — `connect-src 'self'`
# means the window cannot reach out even if some code tries. This script is the
# detector that sits in front of it, so a call gets caught in review rather than
# at runtime.
#
# It deliberately does NOT grep for URLs. A URL in a string is not a network
# call: an SVG carries http://www.w3.org/2000/svg in its own namespace, and the
# trace surface arriving at Gate 1 is SVG. A gate that fires on legitimate code
# is a gate people learn to skip.
#
# It also refuses prompt, alert and confirm. This webview does not implement
# them: they return nothing, the caller gives up, and the feature silently does
# not exist. That is how the program shipped for a while unable to open a job at
# all. Anything the program needs to ask, it asks in a field in the window.
set -uo pipefail
cd "$(dirname "$0")/.."

WHERE=(packages/engine/src packages/engine/bin packages/app/src packages/app/index.html)
SKIP=(--exclude-dir=node_modules --exclude-dir=target --exclude-dir=dist)
fail=0

echo "egress gate"

# The calls themselves, not the strings they might carry.
if hits=$(grep -rnE "${SKIP[@]}" \
     -e '\bfetch\(' \
     -e '\bXMLHttpRequest\b' \
     -e '\bWebSocket\b' \
     -e '\bEventSource\b' \
     -e '\bnavigator\.sendBeacon\b' \
     -e '<script[^>]+src="(https?:)?//' \
     -e '<link[^>]+href="(https?:)?//' \
     "${WHERE[@]}"); then
  echo "  network calls in the product:"
  echo "$hits" | sed 's/^/    /'
  fail=1
else
  echo "  no network calls"
fi

# Dialogs the webview does not have. A call to one of these is a feature that
# silently does nothing.
if hits=$(grep -rnE "${SKIP[@]}" \
     -e '(^|[^.[:alnum:]_])(window\.)?(prompt|alert|confirm)[[:space:]]*\(' \
     "${WHERE[@]}"); then
  echo "  a browser dialog the webview does not implement:"
  echo "$hits" | sed 's/^/    /'
  fail=1
else
  echo "  no browser dialogs"
fi

# The policy has to actually be there, and connect-src has to actually be closed.
CONF=src-tauri/tauri.conf.json
if grep -q "connect-src 'self'" "$CONF"; then
  echo "  connect-src is closed in $CONF"
else
  echo "  connect-src is NOT closed in $CONF"
  fail=1
fi

# An HTTP plugin would hand the program a way around the policy entirely.
if grep -rqE '"http:|tauri-plugin-http' src-tauri/capabilities src-tauri/Cargo.toml; then
  echo "  an HTTP plugin is present — the policy is not the only door any more"
  fail=1
else
  echo "  no HTTP plugin"
fi

[ $fail -eq 0 ] && echo "PASS" || echo "FAIL"
exit $fail
