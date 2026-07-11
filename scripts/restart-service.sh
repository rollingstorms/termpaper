#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"

usage() {
  cat <<'USAGE'
Usage: scripts/restart-service.sh

Restarts termpaper.service on the Raspberry Pi. This may prompt for sudo.
USAGE
}

if [[ "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

require_pi_host

ssh -t "${PI_HOST}" "sudo systemctl restart termpaper.service && sudo systemctl status --no-pager termpaper.service"
