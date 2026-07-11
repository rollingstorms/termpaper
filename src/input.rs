#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Key(KeyEvent),
    InsertText(String),
    VoiceButtonPressed,
    Resize { columns: u16, rows: u16 },
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    Up,
    Down,
    Right,
    Left,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    Function(u8),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalInputMode {
    pub application_cursor: bool,
    pub bracketed_paste: bool,
}

pub fn encode_event(event: &InputEvent, mode: TerminalInputMode) -> Vec<u8> {
    match event {
        InputEvent::Key(key) => encode_key(key, mode),
        InputEvent::InsertText(text) => encode_inserted_text(text, mode.bracketed_paste),
        InputEvent::VoiceButtonPressed => Vec::new(),
        InputEvent::Resize { .. } => Vec::new(),
        InputEvent::Shutdown => Vec::new(),
    }
}

pub fn encode_inserted_text(text: &str, bracketed_paste: bool) -> Vec<u8> {
    if bracketed_paste {
        let mut bytes = b"\x1b[200~".to_vec();
        bytes.extend_from_slice(text.as_bytes());
        bytes.extend_from_slice(b"\x1b[201~");
        bytes
    } else {
        text.as_bytes().to_vec()
    }
}

pub fn encode_key(event: &KeyEvent, mode: TerminalInputMode) -> Vec<u8> {
    let mut bytes = Vec::new();
    if event.modifiers.alt {
        bytes.push(0x1b);
    }

    match event.key {
        Key::Char(c) if event.modifiers.control => {
            if let Some(control) = control_byte(c) {
                bytes.push(control);
            }
        }
        Key::Char(c) => {
            let mut buf = [0; 4];
            bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
        Key::Enter => bytes.push(b'\r'),
        Key::Backspace => bytes.push(0x7f),
        Key::Tab => bytes.push(b'\t'),
        Key::Escape => bytes.push(0x1b),
        Key::Up => bytes.extend_from_slice(cursor_sequence(b'A', mode.application_cursor)),
        Key::Down => bytes.extend_from_slice(cursor_sequence(b'B', mode.application_cursor)),
        Key::Right => bytes.extend_from_slice(cursor_sequence(b'C', mode.application_cursor)),
        Key::Left => bytes.extend_from_slice(cursor_sequence(b'D', mode.application_cursor)),
        Key::Home => bytes.extend_from_slice(b"\x1b[H"),
        Key::End => bytes.extend_from_slice(b"\x1b[F"),
        Key::PageUp => bytes.extend_from_slice(b"\x1b[5~"),
        Key::PageDown => bytes.extend_from_slice(b"\x1b[6~"),
        Key::Insert => bytes.extend_from_slice(b"\x1b[2~"),
        Key::Delete => bytes.extend_from_slice(b"\x1b[3~"),
        Key::Function(n) => bytes.extend_from_slice(function_key_sequence(n)),
    }

    bytes
}

fn cursor_sequence(final_byte: u8, application_cursor: bool) -> &'static [u8] {
    match (final_byte, application_cursor) {
        (b'A', true) => b"\x1bOA",
        (b'B', true) => b"\x1bOB",
        (b'C', true) => b"\x1bOC",
        (b'D', true) => b"\x1bOD",
        (b'A', false) => b"\x1b[A",
        (b'B', false) => b"\x1b[B",
        (b'C', false) => b"\x1b[C",
        (b'D', false) => b"\x1b[D",
        _ => b"",
    }
}

fn function_key_sequence(n: u8) -> &'static [u8] {
    match n {
        1 => b"\x1bOP",
        2 => b"\x1bOQ",
        3 => b"\x1bOR",
        4 => b"\x1bOS",
        5 => b"\x1b[15~",
        6 => b"\x1b[17~",
        7 => b"\x1b[18~",
        8 => b"\x1b[19~",
        9 => b"\x1b[20~",
        10 => b"\x1b[21~",
        11 => b"\x1b[23~",
        12 => b"\x1b[24~",
        _ => b"",
    }
}

fn control_byte(c: char) -> Option<u8> {
    let upper = c.to_ascii_uppercase();
    if ('@'..='_').contains(&upper) {
        Some((upper as u8) - b'@')
    } else if c == '?' {
        Some(0x7f)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode() -> TerminalInputMode {
        TerminalInputMode {
            application_cursor: false,
            bracketed_paste: false,
        }
    }

    #[test]
    fn encodes_arrow_keys_for_normal_and_application_cursor_modes() {
        let key = KeyEvent {
            key: Key::Up,
            modifiers: Modifiers::default(),
        };
        assert_eq!(encode_key(&key, mode()), b"\x1b[A");
        assert_eq!(
            encode_key(
                &key,
                TerminalInputMode {
                    application_cursor: true,
                    bracketed_paste: false
                }
            ),
            b"\x1bOA"
        );
    }

    #[test]
    fn bracketed_paste_wraps_inserted_text_without_enter() {
        assert_eq!(
            encode_inserted_text("hello", true),
            b"\x1b[200~hello\x1b[201~"
        );
    }

    #[test]
    fn control_letters_map_to_control_bytes() {
        let key = KeyEvent {
            key: Key::Char('c'),
            modifiers: Modifiers {
                control: true,
                alt: false,
                shift: false,
            },
        };
        assert_eq!(encode_key(&key, mode()), vec![3]);
    }
}
