use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use font8x8::UnicodeFonts;
use image::{ImageBuffer, Luma};

use crate::config::DisplayMode;
use crate::refresh::{DirtyRect, RenderSnapshot};

const CELL_WIDTH: u32 = 8;
const CELL_HEIGHT: u32 = 12;

pub trait DisplayBackend {
    fn render(&mut self, snapshot: &RenderSnapshot, dirty: &[DirtyRect]) -> Result<()>;
}

pub fn create_backend(mode: DisplayMode, frame_dir: PathBuf) -> Result<Box<dyn DisplayBackend>> {
    match mode {
        DisplayMode::Mock => Ok(Box::new(MockDisplay::new(frame_dir)?)),
        DisplayMode::Debug => Ok(Box::new(DebugDisplay)),
        DisplayMode::Waveshare => Ok(Box::new(WaveshareDisplay)),
    }
}

#[derive(Debug)]
pub struct MockDisplay {
    frame_dir: PathBuf,
    frame_index: u64,
}

impl MockDisplay {
    pub fn new(frame_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&frame_dir)
            .with_context(|| format!("creating frame directory {}", frame_dir.display()))?;
        Ok(Self {
            frame_dir,
            frame_index: 0,
        })
    }
}

impl DisplayBackend for MockDisplay {
    fn render(&mut self, snapshot: &RenderSnapshot, dirty: &[DirtyRect]) -> Result<()> {
        self.frame_index += 1;
        let image_path = self
            .frame_dir
            .join(format!("frame-{index:06}.png", index = self.frame_index));
        let metadata_path = self
            .frame_dir
            .join(format!("frame-{index:06}.txt", index = self.frame_index));

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
            .save(&image_path)
            .with_context(|| format!("writing mock frame {}", image_path.display()))?;

        let metadata = format!(
            "rows={} columns={} cursor=({}, {}) visible={} alternate_screen={}\n",
            snapshot.rows,
            snapshot.columns,
            snapshot.cursor_row,
            snapshot.cursor_column,
            snapshot.cursor_visible,
            snapshot.alternate_screen
        ) + &format!("dirty={dirty:?}\nimage={}\n", image_path.display());

        fs::write(&metadata_path, metadata)
            .with_context(|| format!("writing mock frame metadata {}", metadata_path.display()))
    }
}

fn draw_cell(
    image: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    snapshot: &RenderSnapshot,
    row: u16,
    column: u16,
    idx: usize,
) {
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

fn fill_rect(
    image: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    value: u8,
) {
    for py in y..(y + height) {
        for px in x..(x + width) {
            put_pixel_checked(image, px, py, value);
        }
    }
}

fn put_pixel_checked(image: &mut ImageBuffer<Luma<u8>, Vec<u8>>, x: u32, y: u32, value: u8) {
    if x < image.width() && y < image.height() {
        image.put_pixel(x, y, Luma([value]));
    }
}

#[derive(Debug)]
pub struct DebugDisplay;

impl DisplayBackend for DebugDisplay {
    fn render(&mut self, snapshot: &RenderSnapshot, dirty: &[DirtyRect]) -> Result<()> {
        eprintln!(
            "debug display: {}x{}, dirty rects={:?}, cursor=({}, {})",
            snapshot.columns, snapshot.rows, dirty, snapshot.cursor_row, snapshot.cursor_column
        );
        Ok(())
    }
}

#[derive(Debug)]
pub struct WaveshareDisplay;

impl DisplayBackend for WaveshareDisplay {
    fn render(&mut self, _snapshot: &RenderSnapshot, _dirty: &[DirtyRect]) -> Result<()> {
        #[cfg(target_os = "linux")]
        bail!("waveshare display backend is not wired to SPI/GPIO yet; use --display mock or --display debug");

        #[cfg(not(target_os = "linux"))]
        bail!("waveshare display backend is only supported on Linux/Raspberry Pi targets");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refresh::RenderCell;

    #[test]
    fn mock_display_writes_png_frame() {
        let dir = tempfile::tempdir().unwrap();
        let mut display = MockDisplay::new(dir.path().to_path_buf()).unwrap();
        let snapshot = RenderSnapshot {
            rows: 1,
            columns: 2,
            cells: vec![
                RenderCell {
                    text: "O".to_string(),
                    bold: true,
                    underline: false,
                    inverse: false,
                },
                RenderCell {
                    text: "K".to_string(),
                    bold: false,
                    underline: true,
                    inverse: false,
                },
            ],
            cursor_row: 0,
            cursor_column: 1,
            cursor_visible: true,
            alternate_screen: false,
        };

        display
            .render(
                &snapshot,
                &[DirtyRect::full(snapshot.columns, snapshot.rows)],
            )
            .unwrap();

        assert!(dir.path().join("frame-000001.png").exists());
        assert!(dir.path().join("frame-000001.txt").exists());
    }
}
