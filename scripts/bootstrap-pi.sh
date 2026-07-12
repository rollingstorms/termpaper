#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"

INSTALL_SYSTEM_DEPS=0

usage() {
  cat <<'USAGE'
Usage: scripts/bootstrap-pi.sh [--install-system-deps]

Verifies SSH connectivity, creates the configured remote project directory,
inspects the remote system, and reports missing dependencies.

By default, this script does not install packages or make destructive changes.
Pass --install-system-deps to install the ordinary build dependencies with apt.
That mode may prompt for sudo on the Raspberry Pi.

Environment:
  PI_HOST   SSH target, for example user@host
  PI_DIR    Remote project directory
USAGE
}

case "${1:-}" in
  --install-system-deps) INSTALL_SYSTEM_DEPS=1 ;;
  --help) usage; exit 0 ;;
  "") ;;
  *) usage; exit 2 ;;
esac

require_pi_config

echo "Checking SSH connectivity to ${PI_HOST}..."
ssh -o BatchMode=yes -o ConnectTimeout=5 "${PI_HOST}" 'true'

echo "Creating ${PI_DIR} if missing..."
ssh "${PI_HOST}" "mkdir -p '${PI_DIR}'"

echo "Remote system:"
ssh "${PI_HOST}" 'uname -a; printf "arch="; uname -m; if [ -r /etc/os-release ]; then . /etc/os-release; printf "os=%s\n" "$PRETTY_NAME"; fi'

if [[ "${INSTALL_SYSTEM_DEPS}" == "1" ]]; then
  echo "Installing system dependencies on ${PI_HOST}..."
  ssh -t "${PI_HOST}" 'sudo apt-get update && sudo apt-get install -y cargo rustc rsync pkg-config build-essential libssl-dev'
fi

echo "Checking Rust toolchain..."
ssh "${PI_HOST}" '
missing=0
command -v rustc >/dev/null 2>&1 || { echo "missing: rustc"; missing=1; }
command -v cargo >/dev/null 2>&1 || { echo "missing: cargo"; missing=1; }
command -v rsync >/dev/null 2>&1 || { echo "missing: rsync"; missing=1; }
command -v pkg-config >/dev/null 2>&1 || echo "optional missing: pkg-config"
exit "$missing"
'
