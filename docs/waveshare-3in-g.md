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

The current renderer uses 8 x 12 pixel terminal cells, so the native panel grid
is:

```text
50 columns x 14 rows
```

Implementation note: `epd-waveshare 0.6.0` does not currently expose a driver
module for this exact 3-inch G / 400 x 168 four-color panel. The Waveshare
backend must either add a controller-specific implementation or use a driver
crate that explicitly supports this panel.

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
