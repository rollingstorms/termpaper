# Termpaper

Termpaper is an e-paper-native interactive terminal environment for small Linux computers.

The repository name, primary executable, and service identity are all `termpaper`.

## Current Milestone

This is the first practical scaffold for an interactive terminal MVP:

- PTY-backed child process execution through `portable-pty`.
- VT100/ANSI parsing through the maintained `vt100` crate.
- Raw keyboard input and terminal-style key encoding.
- Bracketed paste-aware text injection API.
- Dirty-cell diffing and refresh coalescing primitives.
- Local display modes for development.
- Raspberry Pi sync, build, run, test, service, and log scripts.

This milestone intentionally sets `TERM=vt100`. The implementation should not claim
`xterm-256color` until its compatibility contract supports the control sequences
applications expect from that terminfo entry.

## Local Development

Run locally with a mock display:

```sh
cargo run -- --display mock
```

Run with debug refresh logging:

```sh
cargo run -- --display debug
```

Attempt the Waveshare backend on Raspberry Pi hardware:

```sh
cargo run -- --display waveshare
```

The Waveshare backend currently returns a clear unsupported/not-wired error until
SPI/GPIO integration is implemented.

## Tests

```sh
cargo fmt --check
cargo test
```

Ordinary tests must stay hardware-independent. SPI, GPIO, and physical display
tests should be opt-in and should not run as part of plain `cargo test`.

## Raspberry Pi Development

Machine-specific Raspberry Pi details live outside this repository in a sibling
local folder:

```text
../altoids/termpaper.env
```

That file is intentionally not part of the Termpaper repo. It may contain local
execution details such as:

```sh
PI_HOST="user@hostname"
PI_DIR="/path/to/remote/termpaper"
```

Initial key setup:

```sh
ssh-copy-id "$PI_HOST"
```

After that, scripts use SSH public-key authentication. Passwords must not be
committed, stored in scripts or config files, printed in logs, passed as command
arguments, or embedded in source.

Recommended optional SSH config pattern:

```sshconfig
Host termpaper-pi
    HostName raspberry-pi-hostname.local
    Port 22
    User raspberry-pi-user
```

## Pi Workflow

```sh
scripts/bootstrap-pi.sh
scripts/sync-pi.sh --dry-run
scripts/sync-pi.sh
scripts/build-pi.sh
scripts/run-pi.sh
scripts/test-pi.sh
scripts/install-service.sh
scripts/restart-service.sh
scripts/logs-pi.sh --follow
```

GitHub is the authoritative source repository, but rapid hardware iteration can
use `rsync` without committing every test attempt. Commit and push meaningful
checkpoints once they compile, pass tests, or validate on hardware.

## Configuration Identity

- Default configuration directory: `/etc/termpaper`
- Development installation directory on Pi: configured by local `PI_DIR`
- systemd service: `termpaper.service`
