#!/usr/bin/env bash
# Run one command when no timed window demonstration is running, and not before.
#   tools/quiet.sh cargo test -p plumbline-world
# A window demonstration clicks by the clock; a build beside it costs it clicks (CLAUDE.md, "Use it").
# tools/window-run.sh holds the lock while it runs; every cargo command of a worker goes through here.
LOCK="${BUILDOS_LOCK:-/tmp/buildos-window.lock}"
# While it waits it says so ($LOCK.waiting), so that window runs called one after another let it in between two of them.
while [ -e "$LOCK" ]; do touch "$LOCK.waiting"; sleep 15; done
exec "$@"
