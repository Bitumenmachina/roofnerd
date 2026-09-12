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

# The two content delivery networks the 3D toolkit's own tutorials load from.
# Named rather than matched as URLs, for the reason above: a URL in a string is
# not a network call and a gate that fires on an SVG namespace gets switched
# off. These two are different — `IfcLoader.setup()` will fetch unpkg on its own
# unless `autoSetWasm` is turned off, and `FragmentsManager.getWorker()` fetches
# the worker from it — so naming either of them in this program's own source is
# a mistake worth catching in review. The copies inside the libraries are not
# this program's source and are not read here; what stops those paths at run
# time is passing the local paths and the policy below.
#
# Comment lines are out of the pass, the same way the plural rule leaves them
# out and for the same reason: the file that closes these two doors has to be
# able to say which doors it closed. A hostname in a comment is not a fetch.
if hits=$(grep -rnE "${SKIP[@]}" -e 'unpkg\.com|cdn\.jsdelivr\.net' "${WHERE[@]}" \
     | grep -vE ':[0-9]+:[[:space:]]*(//|/\*|\*)'); then
  echo "  a content delivery network in the product:"
  echo "$hits" | sed 's/^/    /'
  fail=1
else
  echo "  no content delivery network"
fi

# The multi-thread web-ifc module needs SharedArrayBuffer, which needs
# cross-origin isolation, which this webview is not given and should not be. The
# single-thread module is the one that ships; asking for the other one in code is
# a defect. Comment lines are out of the pass, as above.
if hits=$(grep -rnE "${SKIP[@]}" -e 'web-ifc-mt' "${WHERE[@]}" packages/app/vite.config.ts \
     | grep -vE ':[0-9]+:[[:space:]]*(//|/\*|\*)'); then
  echo "  the multi-thread web-ifc module is named in the product:"
  echo "$hits" | sed 's/^/    /'
  fail=1
else
  echo "  no multi-thread wasm"
fi

# The policy has to actually be there, and connect-src has to actually be closed.
CONF=src-tauri/tauri.conf.json
if grep -q "connect-src 'self'" "$CONF"; then
  echo "  connect-src is closed in $CONF"
else
  echo "  connect-src is NOT closed in $CONF"
  fail=1
fi

# And it has to still be a policy worth having. `script-src 'self'` gained
# `'wasm-unsafe-eval'` when the IFC reader arrived, because WebKit gates
# WebAssembly compilation on script-src and without it web-ifc cannot compile at
# all. That keyword permits compiling the module this program carries; it opens
# no door to the network and it is not the broad `'unsafe-eval'`. The broad one,
# and inline script or style, are what this pair refuses — the quotes are part
# of the pattern, so `'wasm-unsafe-eval'` does not match `'unsafe-eval'`.
wide=0
for opening in "'unsafe-eval'" "'unsafe-inline'"; do
  if grep -qF "$opening" "$CONF"; then
    echo "  $opening is in the policy in $CONF"
    wide=1
    fail=1
  fi
done
[ $wide -eq 0 ] && echo "  the policy carries no unsafe-eval and no unsafe-inline"

# An HTTP plugin would hand the program a way around the policy entirely.
if grep -rqE '"http:|tauri-plugin-http' src-tauri/capabilities src-tauri/Cargo.toml; then
  echo "  an HTTP plugin is present — the policy is not the only door any more"
  fail=1
else
  echo "  no HTTP plugin"
fi

[ $fail -eq 0 ] && echo "PASS" || echo "FAIL"
exit $fail
