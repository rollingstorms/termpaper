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
