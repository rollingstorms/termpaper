#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"

usage() {
  cat <<'USAGE'
Usage: scripts/bootstrap-pi.sh

Verifies SSH connectivity, creates the configured remote project directory,
inspects the remote system, and reports missing dependencies. It does not
install packages or make destructive changes.

Environment:
  PI_HOST   SSH target, for example user@host
  PI_DIR    Remote project directory
USAGE
}

if [[ "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

require_pi_config

echo "Checking SSH connectivity to ${PI_HOST}..."
ssh -o BatchMode=yes -o ConnectTimeout=5 "${PI_HOST}" 'true'

echo "Creating ${PI_DIR} if missing..."
ssh "${PI_HOST}" "mkdir -p '${PI_DIR}'"

echo "Remote system:"
ssh "${PI_HOST}" 'uname -a; printf "arch="; uname -m; if [ -r /etc/os-release ]; then . /etc/os-release; printf "os=%s\n" "$PRETTY_NAME"; fi'

echo "Checking Rust toolchain..."
ssh "${PI_HOST}" '
missing=0
command -v rustc >/dev/null 2>&1 || { echo "missing: rustc"; missing=1; }
command -v cargo >/dev/null 2>&1 || { echo "missing: cargo"; missing=1; }
command -v rsync >/dev/null 2>&1 || { echo "missing: rsync"; missing=1; }
command -v pkg-config >/dev/null 2>&1 || echo "optional missing: pkg-config"
exit "$missing"
'
