#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"
FOLLOW=0

usage() {
  cat <<'USAGE'
Usage: scripts/logs-pi.sh [--follow]
USAGE
}

case "${1:-}" in
  --follow|-f) FOLLOW=1 ;;
  --help) usage; exit 0 ;;
  "") ;;
  *) usage; exit 2 ;;
esac

require_pi_host

if [[ "${FOLLOW}" == "1" ]]; then
  ssh -t "${PI_HOST}" "journalctl -u termpaper.service -n 100 --no-pager -f"
else
  ssh "${PI_HOST}" "journalctl -u termpaper.service -n 100 --no-pager"
fi
