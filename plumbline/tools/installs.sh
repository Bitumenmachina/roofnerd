#!/bin/bash
# Does the README's install list bring everything the window opens when it runs?
#
# The window is linked against almost nothing: it opens the X11 and Vulkan libraries by
# name when it starts. So a build that succeeds says nothing about whether it will run
# on a clean machine. This starts the built window on a virtual display, reads which
# libraries it has open, finds the package each belongs to, and checks that package is
# one the README's `apt-get install` line brings (itself, or as something it depends on).
#
#   tools/installs.sh            # needs the window built (cargo build), xvfb, apt-cache
set -euo pipefail
cd "$(dirname "$0")/.."
APP=target/debug/plumbline-app
[ -x "$APP" ] || { echo "installs: build the window first (cargo build)"; exit 2; }
# The packages of step 1 of "Install on Linux".
LISTED="$(sed -n '/^## Install on Linux/,/^\*\*2\./p' README.md | grep -m1 'apt-get install' | sed 's/.*apt-get install -y//')"
[ -n "$LISTED" ] || { echo "installs: FAIL. No apt-get install line under \"Install on Linux\" in README.md"; exit 1; }
# Everything those bring: themselves and what they depend on, all the way down.
BROUGHT="$(apt-cache depends --recurse --no-recommends --no-suggests --no-conflicts --no-breaks --no-replaces --no-enhances $LISTED | grep -v '^ ' | grep -v '^<' | sort -u)"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
cat > "$T/run.sh" <<RUN
#!/bin/bash
"$PWD/$APP" "$T/job.json" >/dev/null 2>"$T/err.txt" &
PID=\$!
sleep 25
grep -oE "/[^ ]+\.so[^ ]*" /proc/\$PID/maps | sort -u > "$T/open.txt" || true
kill \$PID; wait \$PID 2>/dev/null || true
RUN
chmod +x "$T/run.sh"
timeout 120 xvfb-run -a -s "-screen 0 2500x980x24" "$T/run.sh" || true
[ -s "$T/open.txt" ] || { echo "installs: FAIL. The window did not start:"; head -5 "$T/err.txt"; exit 1; }
missing=0; n=0
while read -r lib; do
  pkg="$(dpkg -S "$lib" 2>/dev/null | head -1 | cut -d: -f1)"
  [ -n "$pkg" ] || continue
  n=$((n + 1))
  if ! grep -qx "$pkg" <<<"$BROUGHT"; then
    echo "  not brought by the README's list: $pkg  (the window opens $lib)"
    missing=$((missing + 1))
  fi
done < "$T/open.txt"
if [ "$missing" -gt 0 ]; then
  echo "installs: FAIL. The window opens libraries from packages the README's install line does not bring."
  exit 1
fi
echo "installs: PASS. The window opens $n system libraries; every one is in a package the README's install line brings ($(wc -w <<<"$LISTED") listed)."
