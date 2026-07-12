#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"

usage() {
  cat <<'USAGE'
Usage: scripts/command-smoke-pi.sh [--display mock|debug|waveshare] [--font compact|standard]

Runs noninteractive terminal smoke commands on the Pi through Termpaper.
The waveshare display performs full panel refreshes and is intentionally slow.
USAGE
}

DISPLAY="mock"
FONT="compact"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --display)
      DISPLAY="${2:-}"
      shift 2
      ;;
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

case "${DISPLAY}" in
  mock|debug|waveshare) ;;
  *) usage; exit 2 ;;
esac
case "${FONT}" in
  compact|standard) ;;
  *) usage; exit 2 ;;
esac

require_pi_config

run_smoke() {
  local command="$1"
  ssh -tt "${PI_HOST}" "cd '${PI_DIR}' && cargo run -- --display '${DISPLAY}' --panel waveshare3-in-g --font '${FONT}' --command '${command}'"
}

run_smoke 'printf "termpaper smoke\n"; pwd; ls -1 | sed -n "1,8p"'
run_smoke 'python3 -c "print(\"python smoke\"); print(2 + 2)"'
