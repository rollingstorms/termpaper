use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::config::DisplayMode;
use crate::refresh::{DirtyRect, RenderSnapshot};

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
        let path = self
            .frame_dir
            .join(format!("frame-{index:06}.txt", index = self.frame_index));
        let mut output = String::new();
        output.push_str(&format!(
            "rows={} columns={} cursor=({}, {}) visible={} alternate_screen={}\n",
            snapshot.rows,
            snapshot.columns,
            snapshot.cursor_row,
            snapshot.cursor_column,
            snapshot.cursor_visible,
            snapshot.alternate_screen
        ));
        output.push_str(&format!("dirty={dirty:?}\n\n"));
        for row in 0..snapshot.rows {
            for column in 0..snapshot.columns {
                let idx = (row as usize * snapshot.columns as usize) + column as usize;
                output.push_str(&snapshot.cells[idx].text);
            }
            output.push('\n');
        }
        fs::write(&path, output).with_context(|| format!("writing mock frame {}", path.display()))
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
