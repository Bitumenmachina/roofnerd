#!/usr/bin/env bash
# ── The egress gate ────────────────────────────────────────────────────────
# The fixed rule: the program itself never opens a connection. Two checks, both
# of the thing itself and not of strings that look like addresses:
#   1. the program's own source names no socket and nothing of std::net;
#   2. nothing it LINKS is a crate whose job is the network (an HTTP client, a
#      TLS stack, a websocket, a message bus). The list is of crates that exist
#      for that; a general event loop (async-io, which the engine's asset reader
#      pulls in) is not on it, because linking one opens nothing: check 1 is
#      what would catch this program using it for a socket.
# The tree read is the one for the machine the gate runs on. A build for another
# system links other crates (the lock file carries some this build never
# compiles) and must be run through this gate on that system before it ships.
#
# Until 2026-11-26 this file also read the earlier program (roofnerd), which
# stood at the root of this branch. That program left the branch that day
# (plumbline/DECISIONS.md call 237; it is on the branch `main`), and its checks
# left with the code they read. The two checks below are as they were.
#
#   tools/gate-egress.sh --self-test    check 2 must fire on a list that holds reqwest
set -uo pipefail
cd "$(dirname "$0")/.."
fail=0

echo "egress gate"

BUILDOS=plumbline/crates
NETWORK_CRATES='reqwest|hyper|hyper-util|ureq|attohttpc|minreq|isahc|surf|curl|curl-sys|tokio|tokio-util|socket2|mio|async-net|async-std|rustls|native-tls|openssl|openssl-sys|tungstenite|tokio-tungstenite|ws|websocket|quinn|h2|h3|tonic|tiny_http|actix-web|warp|axum|zbus|dbus|libdbus-sys|trust-dns-resolver|hickory-resolver|libp2p|ssh2|lettre|sentry|opentelemetry'
[ -d "$BUILDOS" ] || { echo "  no $BUILDOS here: nothing was read"; echo "FAIL"; exit 1; }
if true; then
  if hits=$(grep -rnE --include='*.rs' \
       -e '\bstd::net\b' -e '\bstd::os::unix::net\b' \
       -e '\b(TcpStream|TcpListener|UdpSocket|UnixStream|UnixListener|UnixDatagram|ToSocketAddrs|SocketAddr)\b' \
       "$BUILDOS" | grep -vE ':[0-9]+:[[:space:]]*//'); then
    echo "  BuildOS names a socket in its own source:"
    echo "$hits" | sed 's/^/    /'
    fail=1
  else
    echo "  BuildOS names no socket in its own source"
  fi
  if [ "${1:-}" = "--self-test" ]; then
    tree=$(printf 'serde\nreqwest\nbevy_ecs\n')
  elif command -v cargo >/dev/null 2>&1; then
    tree=$(cd plumbline && cargo tree --offline --workspace -e normal --prefix none 2>/dev/null | sed 's/ v[0-9].*//' | sort -u)
  else
    tree=""
  fi
  if [ -z "$tree" ]; then
    echo "  BuildOS: what it links could not be read (no cargo here, or the tree did not resolve): NOT checked"
    fail=1
  elif linked=$(echo "$tree" | grep -xE "($NETWORK_CRATES)"); then
    echo "  BuildOS links a crate whose job is the network:"
    echo "$linked" | sed 's/^/    /'
    fail=1
  else
    echo "  BuildOS links no network crate on this machine ($(echo "$tree" | wc -l | tr -d ' ') crates read)"
  fi
fi

[ $fail -eq 0 ] && echo "PASS" || echo "FAIL"
exit $fail
