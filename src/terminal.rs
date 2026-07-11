use vt100::Parser;

use crate::input::TerminalInputMode;
use crate::refresh::{DirtyRect, RenderSnapshot};

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
}
