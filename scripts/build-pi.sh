#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"
BUILD_ARGS=()

usage() {
  cat <<'USAGE'
Usage: scripts/build-pi.sh [--release]
USAGE
}

case "${1:-}" in
  --release) BUILD_ARGS=(--release) ;;
  --help) usage; exit 0 ;;
  "") ;;
  *) usage; exit 2 ;;
esac

require_pi_config

if [[ "${#BUILD_ARGS[@]}" -gt 0 ]]; then
  ssh "${PI_HOST}" "cd '${PI_DIR}' && cargo build ${BUILD_ARGS[*]}"
else
  ssh "${PI_HOST}" "cd '${PI_DIR}' && cargo build"
fi
