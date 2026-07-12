# Waveshare 3inch e-Paper (G)

Target product:

- Waveshare SKU 22506
- Part number: `3inch e-Paper (G)`
- Product page: <https://www.waveshare.com/3inch-e-paper-g.htm>
- Wiki: <https://www.waveshare.com/wiki/3inch_e-Paper_Module_(G)>

Public product specs:

- Resolution: 400 x 168 pixels
- Interface: SPI
- Colors: red, yellow, black, white
- Grey scale: 2
- Full refresh time: 12 seconds
- Display size: 70.4 x 29.568 mm
- No partial refresh support is listed in Waveshare's selection guide.

Termpaper profile:

```sh
--panel waveshare3-in-g
```

The Waveshare backend defaults to the standard 8 x 12 pixel font, so the native
terminal grid is:

```text
50 columns x 14 rows
```

The denser compact 6 x 10 pixel font can still be selected explicitly:

```sh
cargo run -- --display waveshare --panel waveshare3-in-g --font compact
```

That mode uses a 66 columns x 16 rows grid. The compact grid leaves a narrow
right and bottom margin because 66 x 16 terminal cells cover 396 x 160 pixels
inside the 400 x 168 panel.

The driver sends a white clear before the first frame after initialization. That
extra refresh helps remove stale red/yellow pigment before monochrome terminal
content is drawn.

Implementation note: `epd-waveshare 0.6.0` does not currently expose a driver
module for this exact 3-inch G / 400 x 168 four-color panel. Termpaper includes
a local SPI/GPIO backend ported from Waveshare's `epd3in0g` sample code.

## Hardware Test Mode

Run one direct full-panel test refresh:

```sh
scripts/hardware-test-pi.sh text
```

Available patterns:

- `white`
- `black`
- `border`
- `checkerboard`
- `text`

Equivalent direct command on the Pi:

```sh
cargo run -- --display waveshare --panel waveshare3-in-g --hardware-test text
```
