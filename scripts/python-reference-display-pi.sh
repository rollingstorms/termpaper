#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/pi-env.sh"

usage() {
  cat <<'USAGE'
Usage: scripts/python-reference-display-pi.sh

Displays a simple Termpaper reference frame using Waveshare's stock Python
epd3in0g driver on the Pi. This bypasses Termpaper's Rust SPI driver and is
intended to isolate hardware/panel behavior from Rust transfer behavior.
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

require_pi_config

ssh -t "${PI_HOST}" "python3 - <<'PY'
import sys
import time
from PIL import Image, ImageDraw, ImageFont

sys.path.append('/home/pi/e-Paper/RaspberryPi_JetsonNano/python/lib')
from waveshare_epd import epd3in0g

epd = epd3in0g.EPD()
width, height = epd.height, epd.width

image = Image.new('RGB', (width, height), epd.WHITE)
draw = ImageDraw.Draw(image)
try:
    font = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf', 18)
except Exception:
    font = ImageFont.load_default()

lines = [
    'TERMPAPER PYTHON REFERENCE',
    'Waveshare stock epd3in0g driver',
    'If this is clean, Rust SPI is suspect.',
    time.strftime('%Y-%m-%d %H:%M:%S'),
]
for index, line in enumerate(lines):
    draw.text((8, 8 + index * 28), line, font=font, fill=epd.BLACK)

draw.rectangle((0, 0, width - 1, height - 1), outline=epd.BLACK)

print('python reference: init clear')
epd.init()
epd.Clear()
print('python reference: display')
epd.init()
epd.display(epd.getbuffer(image))
epd.sleep()
print('python reference: done')
PY"
