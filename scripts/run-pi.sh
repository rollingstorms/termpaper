#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"

usage() {
  cat <<'USAGE'
Usage: scripts/run-pi.sh [--font compact|standard]

Runs Termpaper interactively on the Pi with the Waveshare display backend.
Stops termpaper.service first if it is active so the development process does
not conflict with the installed service.
USAGE
}

FONT="standard"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --font)
      FONT="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      usage
      exit 2
      ;;
  esac
done

case "${FONT}" in
  compact|standard) ;;
  *) usage; exit 2 ;;
esac

require_pi_config

ssh -t "${PI_HOST}" "if systemctl is-active --quiet termpaper.service; then sudo systemctl stop termpaper.service; fi; cd '${PI_DIR}' && cargo run -- --display waveshare --panel waveshare3-in-g --font '${FONT}'"
