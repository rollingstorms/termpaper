#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"
HARDWARE=0

usage() {
  cat <<'USAGE'
Usage: scripts/test-pi.sh [--hardware]

Runs normal Raspberry Pi-compatible tests remotely. Hardware tests are opt-in.
USAGE
}

case "${1:-}" in
  --hardware) HARDWARE=1 ;;
  --help) usage; exit 0 ;;
  "") ;;
  *) usage; exit 2 ;;
esac

require_pi_config

if [[ "${HARDWARE}" == "1" ]]; then
  ssh "${PI_HOST}" "cd '${PI_DIR}' && TERMPAPER_HARDWARE_TESTS=1 cargo test -- --ignored"
else
  ssh "${PI_HOST}" "cd '${PI_DIR}' && cargo test"
fi
