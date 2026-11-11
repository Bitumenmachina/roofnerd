#!/usr/bin/env bash
# Is what is committed enough to build? Takes the last commit as git holds it (nothing
# from the working folder), lays it out in a scratch folder, and builds and tests the
# workspace there. A file the build needs that is in the folder and not in git, because an
# ignore line caught it or it was never added, fails here and nowhere else.
# Run from anywhere in the repository:  plumbline/tools/committed.sh [--quick]
# (COMMIT=<commit> checks that one instead of the last.)
#   --quick  stop after cargo has found every crate of the workspace and the files the
#            demonstrations read; do not build
# It links in this folder's build of what the workspace is built on, and compiles the
# workspace's own crates afresh in a build folder of its own, which it removes.
set -euo pipefail
ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
HERE="$ROOT/plumbline"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
git -C "$ROOT" archive "${COMMIT:-HEAD}" plumbline | tar -x -C "$T"
cd "$T/plumbline"
cargo metadata --offline --no-deps --format-version 1 >/dev/null || { echo "committed: FAIL. The last commit is not a workspace cargo can read: a crate is missing from it."; exit 1; }
missing=0
# Every file a demonstration or a test names under demos/ has to be in the commit.
for f in $(grep -rhoE 'demos/(plans|tables)/[A-Za-z0-9._-]+\.[a-z]+' demos/*.sh crates/*/tests/*.rs 2>/dev/null | sort -u); do
  [ -e "$f" ] || { echo "committed: $f is read by a demonstration or a test and is not in the commit"; missing=1; }
done
[ "$missing" = 0 ] || { echo "committed: FAIL."; exit 1; }
if [ "${1:-}" = "--quick" ]; then echo "committed: PASS (quick). Every crate and every file the demonstrations read is in the last commit."; exit 0; fi
# A build folder of its own, so nothing built from the archive is ever taken for a build of
# the working folder or the other way round. What the workspace is built on is linked in
# from this folder's build (hard links: no copying, no rebuilding); the workspace's own
# crates are always built afresh from the archive.
SCRATCH="$HERE/target/committed"
rm -rf "$SCRATCH"; mkdir -p "$SCRATCH/debug"
for part in deps build .fingerprint; do
  [ -d "$HERE/target/debug/$part" ] && cp -al "$HERE/target/debug/$part" "$SCRATCH/debug/$part"
done
rm -rf "$SCRATCH"/debug/.fingerprint/plumbline* "$SCRATCH"/debug/build/plumbline*
find "$SCRATCH/debug/deps" -maxdepth 1 \( -name 'plumbline*' -o -name 'libplumbline*' \) -delete
trap 'rm -rf "$T" "$SCRATCH"' EXIT
export CARGO_TARGET_DIR="$SCRATCH"
cargo test --workspace --offline 2>&1 | grep -E "^test result|^error|FAILED" > "$T/out.txt" || true
if grep -qE "^error|FAILED" "$T/out.txt" || ! grep -q "^test result" "$T/out.txt"; then cat "$T/out.txt"; echo "committed: FAIL. The last commit does not build and pass its tests from what is in it."; exit 1; fi
echo "committed: PASS. The last commit builds and passes its tests from what is in it ($(grep -c '^test result: ok' "$T/out.txt") test runs)."
