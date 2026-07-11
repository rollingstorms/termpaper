#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"
DRY_RUN=()

usage() {
  cat <<'USAGE'
Usage: scripts/sync-pi.sh [--dry-run]

Synchronizes this working tree to the configured Raspberry Pi project directory.
Uses --delete only inside that dedicated remote project directory.
USAGE
}

case "${1:-}" in
  --dry-run) DRY_RUN=(--dry-run --itemize-changes) ;;
  --help) usage; exit 0 ;;
  "") ;;
  *) usage; exit 2 ;;
esac

require_pi_config

ssh "${PI_HOST}" "mkdir -p '${PI_DIR}'"

rsync -az --delete "${DRY_RUN[@]}" \
  --exclude '.git/' \
  --exclude 'target/' \
  --exclude '.env' \
  --exclude '.env.*' \
  --exclude 'frames/' \
  --exclude 'logs/' \
  --exclude '.DS_Store' \
  --exclude '.vscode/' \
  --exclude '.idea/' \
  --exclude '*secret*' \
  --exclude '*password*' \
  ./ "${PI_HOST}:${PI_DIR}/"
