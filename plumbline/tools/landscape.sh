#!/usr/bin/env bash
# Re-runs the D0 landscape checks and prints the evidence: for each crate, what the
# crates.io registry says; for each repository, its last commit, its license files, and
# whether its own CI runs on Linux. Needs network access to crates.io and github.com.
#
#   tools/landscape.sh > demos/captured/d0-landscape.txt
#
# Clones go to a temporary folder and nothing from them is executed.
set -uo pipefail
CRATES="bevy bevy_ecs bevy_egui egui eframe hecs z3 monstertruck truck-modeling opencascade cadrum fj straight-skeleton ironcalc rust_xlsxwriter calamine hayro pdfium-render ifc-lite-core bimifc-parser ifc_rs i_overlay earcutr boolmesh serde serde_json"
REPOS="bevyengine/bevy holg/bimifc IPedrax/BIMCraft louistrue/ifc-lite LTplus-AG/ifc-lite IfcOpenShell/IfcOpenShell buildingSMART/IFC5-development prove-rs/z3.rs virtualritz/monstertruck ricosjp/truck bschwind/opencascade-rs lzpel/cadrum hannobraun/fornjot lizelive/straight-skeleton ironcalc/IronCalc LaurenzV/hayro ajrcarey/pdfium-render Kentucky-ai/OpenTakeoff Driven-Waterproofing/take-off ArthurBrussee/brush OpenDroneMap/ODM datadrivenconstruction/OpenConstructionERP OpenAEC-Foundation/open-pdf-studio jmcnamara/rust_xlsxwriter"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
echo "D0 landscape evidence, gathered $(date -u +%Y-%m-%d) by tools/landscape.sh"
echo
echo "== Registry (crates.io API) =="
echo "crate | newest version | minimum rustc | declared license | last published | repository"
for c in $CRATES; do
  curl -sS -m 30 -H "User-Agent: plumbline-landscape-check" "https://crates.io/api/v1/crates/$c" -o "$T/c.json" || { echo "$c | REGISTRY NOT REACHED"; continue; }
  python3 -I - "$T/c.json" "$c" <<'PY'
import json, sys
try:
    d = json.load(open(sys.argv[1]))
    c, v = d["crate"], d["versions"][0]
    print(f'{sys.argv[2]} | {c["max_version"]} | {v.get("rust_version") or "not stated"} | {v.get("license")} | {c["updated_at"][:10]} | {c.get("repository")}')
except Exception as e:
    print(f"{sys.argv[2]} | NOT FOUND IN REGISTRY")
PY
done
echo
echo "== Repositories (shallow clone, nothing executed) =="
echo "repository | last commit | license files (first line of each) | CI workflows that run on Linux"
for r in $REPOS; do
  d="$T/$(echo "$r" | tr '/' '_')"
  if ! GIT_TERMINAL_PROMPT=0 timeout 120 git clone -q --depth 1 --filter=blob:none --no-checkout "https://github.com/$r" "$d" 2>/dev/null; then
    echo "$r | NOT REACHABLE (private, renamed, or gone)"; continue
  fi
  last="$(git -C "$d" log -1 --format=%cs)"
  lic=""
  for f in $(git -C "$d" ls-tree --name-only HEAD | grep -iE '^(licen[sc]e|copying|unlicense)' || true); do
    if [ "$(git -C "$d" cat-file -t "HEAD:$f" 2>/dev/null)" = blob ]; then
      first="$(git -C "$d" show "HEAD:$f" 2>/dev/null | grep -v '^[[:space:]]*$' | head -1 | sed -E 's/^[[:space:]#]+//' | cut -c1-60)"
      lic="$lic[$f: $first] "
    fi
  done
  [ -z "$lic" ] && lic="none at the repository root"
  ci="$(for w in $(git -C "$d" ls-tree -r --name-only HEAD -- .github/workflows 2>/dev/null | grep -E '\.ya?ml$' || true); do git -C "$d" show "HEAD:$w" 2>/dev/null | grep -qiE 'ubuntu|linux' && echo yes; done | wc -l)"
  total="$(git -C "$d" ls-tree -r --name-only HEAD -- .github/workflows 2>/dev/null | grep -cE '\.ya?ml$' || true)"
  echo "$r | $last | $lic| $ci of $total"
done
