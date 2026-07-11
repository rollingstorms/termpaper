# Terminal Compatibility Contract

Termpaper currently uses the `vt100` Rust crate for terminal parsing and screen
state. Escape-sequence parsing should remain delegated to a maintained library
unless a required behavior cannot be provided cleanly by that library.

## TERM Value

The MVP sets:

```text
TERM=vt100
```

This is deliberately conservative. Do not switch to `xterm-256color` until the
terminal layer supports the xterm behaviors and terminfo expectations needed by
interactive applications.

## Intended MVP Coverage

The terminal layer is intended to support:

- Printable UTF-8 text.
- Carriage return, line feed, backspace, and tab.
- Automatic wrapping.
- Absolute and relative cursor movement.
- Cursor save and restore.
- Cursor visibility.
- Clear line and clear display operations.
- Insert and delete character operations.
- Insert and delete line operations.
- Primary and alternate screen buffers.
- Application cursor-key mode where reported by the parser.
- Bracketed paste mode.
- Bold, underline, inverse, foreground color, and background color tracking.

The physical e-paper renderer may map colors and attributes to monochrome output,
but terminal state should preserve the attributes.

## Refresh Model

PTY reading, parsing, and in-memory screen mutation must not be blocked by
physical e-paper updates. The refresh scheduler should coalesce mutations and
render only the newest relevant state when updates arrive faster than the panel
can refresh.

High-priority changes include typed characters, editing keys, inserted voice or
clipboard text, prompts, and application states requiring immediate input.

Normal-priority changes include streaming output, Codex-generated text,
scrolling, status messages, and ordinary redraws.

Low-priority changes include cursor blinking, spinner frames, decorative
animation, and rapidly superseded progress updates. Cursor blinking is disabled
by default for the e-paper MVP.
