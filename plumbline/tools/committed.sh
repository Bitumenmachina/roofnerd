#!/usr/bin/env bash
# Is what is committed enough to build? Takes the last commit as git holds it (nothing
# from the working folder), lays it out in a scratch folder, and builds and tests the
# workspace there. A file the build needs that is in the folder and not in git, because an
# ignore line caught it or it was never added, fails here and nowhere else.
# Run from anywhere in the repository:  plumbline/tools/committed.sh [--quick]
#   --quick  stop after cargo has found every crate of the workspace and the files the
#            demonstrations read; do not build
# It reuses this folder's build of what the workspace is built on, so it compiles only
# the workspace's own crates.
set -euo pipefail
ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
HERE="$ROOT/plumbline"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
git -C "$ROOT" archive HEAD plumbline | tar -x -C "$T"
cd "$T/plumbline"
cargo metadata --offline --no-deps --format-version 1 >/dev/null || { echo "committed: FAIL. The last commit is not a workspace cargo can read: a crate is missing from it."; exit 1; }
missing=0
# Every file a demonstration or a test names under demos/ has to be in the commit.
for f in $(grep -rhoE 'demos/(plans|tables)/[A-Za-z0-9._-]+\.[a-z]+' demos/*.sh crates/*/tests/*.rs 2>/dev/null | sort -u); do
  [ -e "$f" ] || { echo "committed: $f is read by a demonstration or a test and is not in the commit"; missing=1; }
done
[ "$missing" = 0 ] || { echo "committed: FAIL."; exit 1; }
if [ "${1:-}" = "--quick" ]; then echo "committed: PASS (quick). Every crate and every file the demonstrations read is in the last commit."; exit 0; fi
# The same build folder: what the workspace is built on is found built; the workspace's own
# crates are built again from the archive, since they are at another place.
export CARGO_TARGET_DIR="$HERE/target"
cargo test --workspace --offline 2>&1 | grep -E "^test result|^error|FAILED" > "$T/out.txt" || true
if grep -qE "^error|FAILED" "$T/out.txt" || ! grep -q "^test result" "$T/out.txt"; then cat "$T/out.txt"; echo "committed: FAIL. The last commit does not build and pass its tests from what is in it."; exit 1; fi
echo "committed: PASS. The last commit builds and passes its tests from what is in it ($(grep -c '^test result: ok' "$T/out.txt") test runs)."
