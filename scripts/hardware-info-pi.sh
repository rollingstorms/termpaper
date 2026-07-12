#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"

usage() {
  cat <<'USAGE'
Usage: scripts/hardware-info-pi.sh

Prints non-destructive Raspberry Pi hardware readiness details relevant to the
Waveshare e-paper backend: SPI devices, GPIO chips, user groups, boot overlays,
and common GPIO/SPI tools.
USAGE
}

if [[ "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

require_pi_host

ssh "${PI_HOST}" '
set -eu

echo "== system =="
uname -a
if [ -r /etc/os-release ]; then
  . /etc/os-release
  printf "os=%s\n" "$PRETTY_NAME"
fi
printf "arch=%s\n" "$(uname -m)"

echo
echo "== user groups =="
id -nG

echo
echo "== spi devices =="
ls -l /dev/spidev* 2>/dev/null || echo "no /dev/spidev* devices"

echo
echo "== gpio chips =="
ls -l /dev/gpiochip* 2>/dev/null || echo "no /dev/gpiochip* devices"

echo
echo "== boot overlays =="
for f in /boot/firmware/config.txt /boot/config.txt; do
  if [ -r "$f" ]; then
    echo "-- $f --"
    grep -E "^(dtparam|dtoverlay|enable_uart|gpio|spi)" "$f" || true
  fi
done

echo
echo "== tools =="
for tool in raspi-config gpioinfo gpiodetect pinctrl dtoverlay vcgencmd gh; do
  if command -v "$tool" >/dev/null 2>&1; then
    printf "%s=%s\n" "$tool" "$(command -v "$tool")"
  else
    printf "%s=missing\n" "$tool"
  fi
done

echo
echo "== gpio summary =="
if command -v gpioinfo >/dev/null 2>&1; then
  gpioinfo | sed -n "1,160p"
else
  echo "gpioinfo missing"
fi
'
