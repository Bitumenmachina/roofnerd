#!/usr/bin/env bash
# Checks PORTMAP.md against the building model and against BIMCraft's source.
# See tools/portmap.py. Run from anywhere.
exec python3 "$(dirname "$0")/portmap.py" "$@"
