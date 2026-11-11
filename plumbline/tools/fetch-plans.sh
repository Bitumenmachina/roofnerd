#!/usr/bin/env bash
# Fetches the public plan set the demonstrations trace over, into plans/ (which git
# ignores), and checks it is the file that was looked at.
#
#   USDA Plan No. 7213, Two-Bedroom Farmhouse (1976), two sheets, scanned.
#   Cooperative Extension Work in Agriculture and Home Economics, U.S. Department of
#   Agriculture cooperating. Held by the National Agricultural Library, which says of it:
#   "This item is authored by federal employees as part of their official duties and is
#   therefore non-copyrightable and/or published by the federal government and now in the
#   public domain."  https://www.nal.usda.gov/exhibits/ipd/ruralusa/items/show/51
#
# Needs the network and curl. Run from the repository root:  tools/fetch-plans.sh
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p plans
get() { # $1 file, $2 address, $3 SHA-256
  if [ -f "plans/$1" ] && echo "$3  plans/$1" | sha256sum -c --quiet 2>/dev/null; then
    echo "plans/$1: already here, and it is the file that was looked at"
    return
  fi
  curl -sS -L --fail -m 300 -o "plans/$1.part" "$2"
  echo "$3  plans/$1.part" | sha256sum -c --quiet || { echo "plans/$1: what came is not the file that was looked at; not kept"; rm -f "plans/$1.part"; exit 1; }
  mv "plans/$1.part" "plans/$1"
  echo "plans/$1: fetched"
}
get usda-7213-two-bedroom-farmhouse.pdf "https://www.nal.usda.gov/exhibits/ipd/ruralusa/files/original/e4df2761c9078e52c8e3b4fefa562560.pdf" 13788ecf41653481d2ee0884d5443c52ddec0281dc05e07283988db3a2bf2d78
