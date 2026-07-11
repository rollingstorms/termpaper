# Local Target Configuration

The Termpaper repo should stay open and shareable. Device-specific execution
details belong in a sibling folder next to the repo:

```text
../altoids/
```

The Pi scripts automatically source:

```text
../altoids/termpaper.env
```

Expected variables:

```sh
PI_HOST="user@hostname"
PI_DIR="/remote/project/path"
```

You can also provide those variables directly in the shell environment.

Do not store passwords in this file. Use SSH public-key authentication or a
local SSH config alias.
