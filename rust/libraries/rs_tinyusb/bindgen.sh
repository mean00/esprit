#!/usr/bin/env bash
set -e
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "$DIR/c_interface/bindgen.py" "$@"
