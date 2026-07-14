# Waveshare 3inch G Driver Port

This document is the hardware contract for Termpaper's local driver for the
Waveshare 3inch e-Paper (G), SKU 22506.

## Source Of Truth

Termpaper ports behavior from the Waveshare demo tree installed on the Pi:

- Python driver: `/home/pi/e-Paper/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epd3in0g.py`
- Python hardware config: `/home/pi/e-Paper/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epdconfig.py`
- C driver: `/home/pi/e-Paper/RaspberryPi_JetsonNano/c/lib/e-Paper/EPD_3in0g.c`

The Python driver is the reference path because it has been verified to render a
clean reference frame on the local panel.

## Hardware Contract

| Signal | GPIO / SPI | Source |
| --- | --- | --- |
| RESET | GPIO17 | `epdconfig.RST_PIN` |
| DC | GPIO25 | `epdconfig.DC_PIN` |
| BUSY | GPIO24 | `epdconfig.BUSY_PIN` |
| PWR | GPIO18 | `epdconfig.PWR_PIN` |
| CS | SPI0 CE0 / GPIO8 | `epdconfig.CS_PIN`, `SpiDev.open(0, 0)` |
| MOSI | GPIO10 | SPI0 MOSI |
| SCLK | GPIO11 | SPI0 SCLK |

SPI settings:

- Bus: SPI0
- Device: CE0
- Clock: 4 MHz
- Mode: 0

Panel geometry:

- Native controller buffer: 168 x 400 portrait
- Termpaper landscape frame: 400 x 168
- Packed buffer size: 16800 bytes
- Pixel encoding: 2 bits per pixel, 4 pixels per byte

Color codes:

| Color | Code |
| --- | --- |
| Black | `0b00` |
| White | `0b01` |
| Yellow | `0b10` |
| Red | `0b11` |

## Port Mapping

| Waveshare behavior | Termpaper implementation |
| --- | --- |
| `module_init`: power on GPIO18 | `Epd3in0gDevice::open_default`, `init` |
| Reset high 200 ms, low 2 ms, high 200 ms | `Epd3in0gDevice::reset` |
| Init command sequence | `Epd3in0gDevice::init` |
| Landscape image `rotate(90, expand=True)` | `pack_landscape_*_frame` |
| Pack 4 color indices per byte | `pack_2bpp_pixels` |
| `0x04`, wait busy high, write `0x10` buffer | `display_packed`, `write_solid_color` |
| Refresh with `0x12`, `0x01` | `turn_on_display` |
| Power off `0x02`, deep sleep `0x07`, `0xA5` | `sleep` |

## Bring-Up Tests

Run these after changing low-level driver behavior:

```sh
scripts/sync-pi.sh
scripts/build-pi.sh
scripts/test-pi.sh
scripts/hardware-test-pi.sh white
scripts/hardware-test-pi.sh black
scripts/hardware-test-pi.sh red
scripts/hardware-test-pi.sh yellow
scripts/hardware-test-pi.sh color-quadrants
scripts/hardware-test-pi.sh text
```

Use the Python reference when Rust output is suspect:

```sh
scripts/python-reference-display-pi.sh
```

Interpretation:

- Python reference clean, Rust dirty: Rust transport, power, timing, refresh, or packing bug.
- Python reference dirty too: panel state, cable/HAT, power, or physical panel issue.
- Monochrome tests clean but color tests dirty: color encoding or pigment clear behavior.
