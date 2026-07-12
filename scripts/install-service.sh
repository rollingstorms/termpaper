#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"
SERVICE_FILE="/etc/systemd/system/termpaper.service"

usage() {
  cat <<'USAGE'
Usage: scripts/install-service.sh

Builds a release binary and installs termpaper.service. This may prompt for sudo
on the Raspberry Pi; the script does not assume passwordless sudo.
USAGE
}

if [[ "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

require_pi_config

ssh -t "${PI_HOST}" "cd '${PI_DIR}' && cargo build --release && sudo tee '${SERVICE_FILE}' >/dev/null <<'SERVICE'
[Unit]
Description=Termpaper e-paper terminal
After=network.target

[Service]
Type=simple
WorkingDirectory=${PI_DIR}
ExecStart=${PI_DIR}/target/release/termpaper --display waveshare --panel waveshare3-in-g
Restart=on-failure
RestartSec=2

[Install]
WantedBy=multi-user.target
SERVICE
sudo systemctl daemon-reload
sudo systemctl enable termpaper.service"
