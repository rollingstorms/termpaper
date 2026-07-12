# Raspberry Pi Development Notes

Termpaper keeps machine-specific execution details outside the public project
tree. Create a sibling local folder:

```text
../altoids
```

Then create:

```text
../altoids/termpaper.env
```

with values for:

```sh
PI_HOST="user@hostname"
PI_DIR="/path/to/remote/termpaper"
```

Use SSH public-key authentication:

```sh
ssh-copy-id "$PI_HOST"
```

Routine commands should connect with:

```sh
ssh "$PI_HOST"
```

or an equivalent SSH host alias.

Do not commit the sibling `altoids` folder to this repository. It is for local
execution details and any setup notes that should not be published with
Termpaper.

## Bootstrap

Audit the Pi without installing anything:

```sh
scripts/bootstrap-pi.sh
```

If ordinary Rust build dependencies are missing and you want the script to
install them with apt:

```sh
scripts/bootstrap-pi.sh --install-system-deps
```

That install mode may prompt for sudo on the Pi.

## Hardware Audit

Inspect non-destructive hardware readiness details:

```sh
scripts/hardware-info-pi.sh
```

Run a direct full-refresh test pattern:

```sh
scripts/hardware-test-pi.sh text
```

Run noninteractive command smoke tests through Termpaper:

```sh
scripts/command-smoke-pi.sh --display mock
```

Use `--display waveshare` only when you want slow full-panel refreshes. Use
`--font compact` when comparing against the denser 66 columns x 16 rows grid.

When running interactively, `Ctrl-]` stops Termpaper locally and terminates the
child shell. `Ctrl-C` and `Ctrl-D` are forwarded to the shell.

For a Waveshare SPI e-paper HAT, the expected baseline is:

- the user belongs to the `spi` and `gpio` groups
- `/dev/spidev0.0` exists
- one or more `/dev/gpiochip*` devices exist
- boot config includes `dtparam=spi=on`

The Waveshare backend uses a local SPI/GPIO driver for the current target panel.
It was ported from Waveshare's `epd3in0g` sample code because `epd-waveshare
0.6.0` does not expose a module for this exact 3-inch G / 400 x 168 four-color
panel.

Current target panel:

- Waveshare SKU 22506, `3inch e-Paper (G)`
- 400 x 168 pixels
- SPI interface
- red/yellow/black/white display colors
- 12 second full refresh
- no partial refresh listed in Waveshare's selection guide

Termpaper defaults to the standard 8 x 12 font for
`--display waveshare --panel waveshare3-in-g`, which gives a 50 columns x 14
rows grid. Use `--font compact` for the denser 6 x 10 font and 66 columns x 16
rows grid.

The first frame after display initialization performs an extra white clear. This
is intentional; it helps remove stale red/yellow pigment before drawing
monochrome terminal content.

## Sync Exclusions

`scripts/sync-pi.sh` excludes:

- `.git/`
- `target/`
- `.env`
- `.env.*`
- `frames/`
- `logs/`
- editor metadata
- common secret file names

The script uses `--delete`, but only against the dedicated remote directory
configured by `PI_DIR`.

## sudo

Service scripts call `sudo systemctl ...` and do not assume passwordless sudo.

If desired, configure a minimal sudoers rule only for Termpaper service
operations, rather than unrestricted root access. For example, adapt this with
`visudo`:

Use `visudo` and substitute the local Raspberry Pi user. Limit the rule to
`systemctl` operations for `termpaper.service`.

Do not put passwords in scripts, source code, config files, command-line
arguments, or logs.
