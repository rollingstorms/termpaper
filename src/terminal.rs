use vt100::Parser;

use crate::input::TerminalInputMode;
use crate::refresh::{DirtyRect, RenderSnapshot, TerminalColor};

pub struct TerminalState {
    parser: Parser,
    last_snapshot: Option<RenderSnapshot>,
}

impl TerminalState {
    pub fn new(rows: u16, columns: u16) -> Self {
        Self {
            parser: Parser::new(rows, columns, 1_000),
            last_snapshot: None,
        }
    }

    pub fn process_output(&mut self, bytes: &[u8]) -> Vec<DirtyRect> {
        self.parser.process(bytes);
        let next = self.snapshot();
        let dirty = match &self.last_snapshot {
            Some(previous) => previous.diff(&next),
            None => vec![DirtyRect::full(next.columns, next.rows)],
        };
        self.last_snapshot = Some(next);
        dirty
    }

    pub fn resize(&mut self, rows: u16, columns: u16) {
        self.parser.screen_mut().set_size(rows, columns);
        self.last_snapshot = None;
    }

    pub fn input_mode(&self) -> TerminalInputMode {
        let screen = self.parser.screen();
        TerminalInputMode {
            application_cursor: screen.application_cursor(),
            bracketed_paste: screen.bracketed_paste(),
        }
    }

    pub fn snapshot(&self) -> RenderSnapshot {
        let screen = self.parser.screen();
        let (rows, columns) = screen.size();
        let mut cells = Vec::with_capacity(rows as usize * columns as usize);

        for row in 0..rows {
            for column in 0..columns {
                let cell = screen.cell(row, column);
                cells.push(crate::refresh::RenderCell {
                    text: cell
                        .map(|cell| cell.contents().to_string())
                        .unwrap_or_else(|| " ".to_string()),
                    bold: cell.map(|cell| cell.bold()).unwrap_or(false),
                    underline: cell.map(|cell| cell.underline()).unwrap_or(false),
                    inverse: cell.map(|cell| cell.inverse()).unwrap_or(false),
                    foreground: cell
                        .map(|cell| cell.fgcolor())
                        .map(TerminalColor::from)
                        .unwrap_or_default(),
                    background: cell
                        .map(|cell| cell.bgcolor())
                        .map(TerminalColor::from)
                        .unwrap_or_default(),
                });
            }
        }

        let (cursor_row, cursor_column) = screen.cursor_position();
        RenderSnapshot {
            rows,
            columns,
            cells,
            cursor_row,
            cursor_column,
            cursor_visible: !screen.hide_cursor(),
            alternate_screen: screen.alternate_screen(),
        }
    }

    pub fn contents(&self) -> String {
        self.parser.screen().contents()
    }
}

impl From<vt100::Color> for TerminalColor {
    fn from(color: vt100::Color) -> Self {
        match color {
            vt100::Color::Default => Self::Default,
            vt100::Color::Idx(index) => Self::Indexed(index),
            vt100::Color::Rgb(red, green, blue) => Self::Rgb(red, green, blue),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_rewrites_existing_screen_content() {
        let mut terminal = TerminalState::new(2, 10);
        terminal.process_output(b"hello\rbye");
        assert!(terminal.contents().contains("byelo"));
    }

    #[test]
    fn tracks_alternate_screen_mode() {
        let mut terminal = TerminalState::new(2, 10);
        terminal.process_output(b"\x1b[?1049h");
        assert!(terminal.snapshot().alternate_screen);
        terminal.process_output(b"\x1b[?1049l");
        assert!(!terminal.snapshot().alternate_screen);
    }

    #[test]
    fn tracks_bracketed_paste_mode() {
        let mut terminal = TerminalState::new(2, 10);
        terminal.process_output(b"\x1b[?2004h");
        assert!(terminal.input_mode().bracketed_paste);
        terminal.process_output(b"\x1b[?2004l");
        assert!(!terminal.input_mode().bracketed_paste);
    }

    #[test]
    fn tracks_cursor_visibility() {
        let mut terminal = TerminalState::new(2, 10);
        terminal.process_output(b"\x1b[?25l");
        assert!(!terminal.snapshot().cursor_visible);
        terminal.process_output(b"\x1b[?25h");
        assert!(terminal.snapshot().cursor_visible);
    }

    #[test]
    fn tracks_text_attributes_and_colors() {
        let mut terminal = TerminalState::new(2, 10);
        terminal.process_output(b"\x1b[1;4;7;31;47mX");
        let snapshot = terminal.snapshot();
        let cell = &snapshot.cells[0];

        assert!(cell.bold);
        assert!(cell.underline);
        assert!(cell.inverse);
        assert_eq!(cell.foreground, TerminalColor::Indexed(1));
        assert_eq!(cell.background, TerminalColor::Indexed(7));
    }

    #[test]
    fn clears_screen_content() {
        let mut terminal = TerminalState::new(2, 5);
        terminal.process_output(b"hello\x1b[2J\x1b[H");
        assert_eq!(terminal.contents().trim(), "");
    }

    #[test]
    fn inserts_and_deletes_lines() {
        let mut terminal = TerminalState::new(4, 5);
        terminal.process_output(b"one\r\ntwo\r\nthree\x1b[2;1H\x1b[Lnew");
        assert!(terminal.contents().contains("new"));
        assert!(terminal.contents().contains("two"));

        terminal.process_output(b"\x1b[2;1H\x1b[M");
        assert!(!terminal.contents().contains("new"));
    }

    #[test]
    fn scroll_region_limits_scrolling() {
        let mut terminal = TerminalState::new(4, 5);
        terminal.process_output(b"top\r\none\r\ntwo\r\nbot");
        terminal.process_output(b"\x1b[2;3r\x1b[3;1H\n");
        let contents = terminal.contents();

        assert!(contents.contains("top"));
        assert!(contents.contains("bot"));
    }
}
