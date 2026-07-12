use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::config::DisplayMode;
use crate::refresh::{DirtyRect, RenderSnapshot};
use crate::render::render_snapshot_to_mono;

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

        let image = render_snapshot_to_mono(snapshot);

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
    use crate::refresh::{RenderCell, TerminalColor};

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
                    foreground: TerminalColor::Default,
                    background: TerminalColor::Default,
                },
                RenderCell {
                    text: "K".to_string(),
                    bold: false,
                    underline: true,
                    inverse: false,
                    foreground: TerminalColor::Default,
                    background: TerminalColor::Default,
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
