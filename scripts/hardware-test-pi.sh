#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"

PATTERN="${1:-text}"

usage() {
  cat <<'USAGE'
Usage: scripts/hardware-test-pi.sh [white|black|red|yellow|border|checkerboard|color-quadrants|text]

Runs a direct Waveshare 3inch e-Paper (G) hardware test pattern on the Pi.
This bypasses the PTY terminal loop and performs one full panel refresh.
USAGE
}

case "${PATTERN}" in
  white|black|red|yellow|border|checkerboard|color-quadrants|text) ;;
  --help|-h) usage; exit 0 ;;
  *) usage; exit 2 ;;
esac

require_pi_config

ssh -t "${PI_HOST}" "cd '${PI_DIR}' && cargo run -- --display waveshare --panel waveshare3-in-g --hardware-test '${PATTERN}'"
