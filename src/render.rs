use font8x8::UnicodeFonts;
use image::{ImageBuffer, Luma};

use crate::refresh::RenderSnapshot;

pub const CELL_WIDTH: u32 = 8;
pub const CELL_HEIGHT: u32 = 12;
pub const COMPACT_CELL_WIDTH: u32 = 6;
pub const COMPACT_CELL_HEIGHT: u32 = 10;

pub type MonoFrame = ImageBuffer<Luma<u8>, Vec<u8>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellMetrics {
    pub width: u32,
    pub height: u32,
    pub glyph_width: u32,
    pub glyph_height: u32,
    pub y_offset: u32,
}

impl CellMetrics {
    pub const STANDARD: Self = Self {
        width: CELL_WIDTH,
        height: CELL_HEIGHT,
        glyph_width: 8,
        glyph_height: 8,
        y_offset: 2,
    };

    pub const COMPACT: Self = Self {
        width: COMPACT_CELL_WIDTH,
        height: COMPACT_CELL_HEIGHT,
        glyph_width: 6,
        glyph_height: 8,
        y_offset: 1,
    };
}

pub fn render_snapshot_to_mono(snapshot: &RenderSnapshot) -> MonoFrame {
    render_snapshot_to_mono_with_metrics(snapshot, CellMetrics::STANDARD)
}

pub fn render_snapshot_to_mono_with_metrics(
    snapshot: &RenderSnapshot,
    metrics: CellMetrics,
) -> MonoFrame {
    let width = snapshot.columns as u32 * metrics.width;
    let height = snapshot.rows as u32 * metrics.height;
    render_snapshot_to_mono_canvas(snapshot, metrics, width, height)
}

pub fn render_snapshot_to_mono_canvas(
    snapshot: &RenderSnapshot,
    metrics: CellMetrics,
    width: u32,
    height: u32,
) -> MonoFrame {
    let mut image = ImageBuffer::from_pixel(width, height, Luma([255_u8]));

    for row in 0..snapshot.rows {
        for column in 0..snapshot.columns {
            let idx = (row as usize * snapshot.columns as usize) + column as usize;
            draw_cell(&mut image, snapshot, row, column, idx, metrics);
        }
    }

    image
}

fn draw_cell(
    image: &mut MonoFrame,
    snapshot: &RenderSnapshot,
    row: u16,
    column: u16,
    idx: usize,
    metrics: CellMetrics,
) {
    let cell = &snapshot.cells[idx];
    let cursor_here =
        snapshot.cursor_visible && snapshot.cursor_row == row && snapshot.cursor_column == column;
    let inverse = cell.inverse ^ cursor_here;
    let background = if inverse { 0 } else { 255 };
    let foreground = if inverse { 255 } else { 0 };
    let x0 = column as u32 * metrics.width;
    let y0 = row as u32 * metrics.height;

    fill_rect(image, x0, y0, metrics.width, metrics.height, background);

    let ch = cell.text.chars().next().unwrap_or(' ');
    if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
        for (glyph_y, bits) in glyph.iter().enumerate().take(metrics.glyph_height as usize) {
            for glyph_x in 0..metrics.glyph_width {
                if bits & (1 << glyph_x) != 0 {
                    let px = x0 + glyph_x;
                    let py = y0 + glyph_y as u32 + metrics.y_offset;
                    put_pixel_checked(image, px, py, foreground);
                    if cell.bold && glyph_x + 1 < metrics.glyph_width {
                        put_pixel_checked(image, px + 1, py, foreground);
                    }
                }
            }
        }
    }

    if cell.underline {
        fill_rect(
            image,
            x0,
            y0 + metrics.height.saturating_sub(2),
            metrics.width,
            1,
            foreground,
        );
    }
}

fn fill_rect(image: &mut MonoFrame, x: u32, y: u32, width: u32, height: u32, value: u8) {
    for py in y..(y + height) {
        for px in x..(x + width) {
            put_pixel_checked(image, px, py, value);
        }
    }
}

fn put_pixel_checked(image: &mut MonoFrame, x: u32, y: u32, value: u8) {
    if x < image.width() && y < image.height() {
        image.put_pixel(x, y, Luma([value]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refresh::{RenderCell, TerminalColor};

    fn cell(text: &str) -> RenderCell {
        RenderCell {
            text: text.to_string(),
            bold: false,
            underline: false,
            inverse: false,
            foreground: TerminalColor::Default,
            background: TerminalColor::Default,
        }
    }

    #[test]
    fn renders_snapshot_at_stable_cell_dimensions() {
        let snapshot = RenderSnapshot {
            rows: 2,
            columns: 3,
            cells: vec![cell("A"); 6],
            cursor_row: 0,
            cursor_column: 0,
            cursor_visible: false,
            alternate_screen: false,
        };

        let frame = render_snapshot_to_mono(&snapshot);

        assert_eq!(frame.width(), 3 * CELL_WIDTH);
        assert_eq!(frame.height(), 2 * CELL_HEIGHT);
    }

    #[test]
    fn renders_compact_snapshot_at_stable_cell_dimensions() {
        let snapshot = RenderSnapshot {
            rows: 2,
            columns: 3,
            cells: vec![cell("A"); 6],
            cursor_row: 0,
            cursor_column: 0,
            cursor_visible: false,
            alternate_screen: false,
        };

        let frame = render_snapshot_to_mono_with_metrics(&snapshot, CellMetrics::COMPACT);

        assert_eq!(frame.width(), 3 * COMPACT_CELL_WIDTH);
        assert_eq!(frame.height(), 2 * COMPACT_CELL_HEIGHT);
    }

    #[test]
    fn renders_snapshot_into_larger_canvas() {
        let snapshot = RenderSnapshot {
            rows: 1,
            columns: 1,
            cells: vec![cell("A")],
            cursor_row: 0,
            cursor_column: 0,
            cursor_visible: false,
            alternate_screen: false,
        };

        let frame = render_snapshot_to_mono_canvas(&snapshot, CellMetrics::COMPACT, 10, 12);

        assert_eq!(frame.width(), 10);
        assert_eq!(frame.height(), 12);
        assert_eq!(frame.get_pixel(9, 11), &Luma([255]));
    }

    #[test]
    fn cursor_inverts_cell_background() {
        let snapshot = RenderSnapshot {
            rows: 1,
            columns: 1,
            cells: vec![cell(" ")],
            cursor_row: 0,
            cursor_column: 0,
            cursor_visible: true,
            alternate_screen: false,
        };

        let frame = render_snapshot_to_mono(&snapshot);

        assert_eq!(frame.get_pixel(0, 0), &Luma([0]));
    }
}
