use font8x8::UnicodeFonts;
use image::{ImageBuffer, Luma};

use crate::refresh::RenderSnapshot;

pub const CELL_WIDTH: u32 = 8;
pub const CELL_HEIGHT: u32 = 12;

pub type MonoFrame = ImageBuffer<Luma<u8>, Vec<u8>>;

pub fn render_snapshot_to_mono(snapshot: &RenderSnapshot) -> MonoFrame {
    let width = snapshot.columns as u32 * CELL_WIDTH;
    let height = snapshot.rows as u32 * CELL_HEIGHT;
    let mut image = ImageBuffer::from_pixel(width, height, Luma([255_u8]));

    for row in 0..snapshot.rows {
        for column in 0..snapshot.columns {
            let idx = (row as usize * snapshot.columns as usize) + column as usize;
            draw_cell(&mut image, snapshot, row, column, idx);
        }
    }

    image
}

fn draw_cell(image: &mut MonoFrame, snapshot: &RenderSnapshot, row: u16, column: u16, idx: usize) {
    let cell = &snapshot.cells[idx];
    let cursor_here =
        snapshot.cursor_visible && snapshot.cursor_row == row && snapshot.cursor_column == column;
    let inverse = cell.inverse ^ cursor_here;
    let background = if inverse { 0 } else { 255 };
    let foreground = if inverse { 255 } else { 0 };
    let x0 = column as u32 * CELL_WIDTH;
    let y0 = row as u32 * CELL_HEIGHT;

    fill_rect(image, x0, y0, CELL_WIDTH, CELL_HEIGHT, background);

    let ch = cell.text.chars().next().unwrap_or(' ');
    if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
        for (glyph_y, bits) in glyph.iter().enumerate() {
            for glyph_x in 0..8 {
                if bits & (1 << glyph_x) != 0 {
                    let px = x0 + glyph_x;
                    let py = y0 + glyph_y as u32 + 2;
                    put_pixel_checked(image, px, py, foreground);
                    if cell.bold && glyph_x < 7 {
                        put_pixel_checked(image, px + 1, py, foreground);
                    }
                }
            }
        }
    }

    if cell.underline {
        fill_rect(image, x0, y0 + CELL_HEIGHT - 2, CELL_WIDTH, 1, foreground);
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
