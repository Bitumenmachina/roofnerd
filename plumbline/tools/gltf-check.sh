#!/usr/bin/env bash
# A glTF file checked by the Khronos glTF Validator, which is not part of this program.
#
#   tools/gltf-check.sh <house.glb>
#
# The validator (npm package `gltf-validator`, Apache-2.0, Khronos's own) is an outside checker: run
# here, never linked, never committed. Where it is installed (the folder named by GLTF_VALIDATOR_DIR,
# else ~/.cache/buildos-gltf-validator) it checks the file against the format and exits 1 on any error
# or warning, printing each. Where it is not installed it says so, says how to install it, and exits 3
# (the demonstration reports that as "not checked", never as passed).
#
#   npm install --prefix ~/.cache/buildos-gltf-validator gltf-validator@2.0.0-dev.3.10
set -euo pipefail
[ $# -eq 1 ] || { echo "usage: tools/gltf-check.sh <house.glb>"; exit 2; }
DIR="${GLTF_VALIDATOR_DIR:-$HOME/.cache/buildos-gltf-validator}"
if ! command -v node >/dev/null 2>&1 || [ ! -f "$DIR/node_modules/gltf-validator/index.js" ]; then
  echo "gltf-check: the Khronos glTF Validator is not installed here, so the file was not checked by it."
  echo "  to install it: npm install --prefix \"$DIR\" gltf-validator@2.0.0-dev.3.10"
  exit 3
fi
exec node - "$DIR/node_modules/gltf-validator" "$1" <<'JS'
const fs = require('fs');
const [lib, file] = process.argv.slice(2);
const validator = require(lib);
validator.validateBytes(new Uint8Array(fs.readFileSync(file)), { uri: 'house.glb' }).then((r) => {
  const i = r.issues;
  console.log(`glTF Validator ${validator.version()}: ${i.numErrors} errors, ${i.numWarnings} warnings, ${i.numInfos} infos, ${i.numHints} hints`);
  for (const m of i.messages) if (m.severity <= 1) console.log(`  ${m.severity === 0 ? 'error' : 'warning'} ${m.code} at ${m.pointer}: ${m.message}`);
  process.exit(i.numErrors + i.numWarnings > 0 ? 1 : 0);
}).catch((e) => { console.log('glTF Validator could not read the file: ' + e); process.exit(1); });
JS
