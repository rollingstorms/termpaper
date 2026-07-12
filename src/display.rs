use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use crate::config::DisplayMode;
use crate::panel::PanelModel;
use crate::refresh::{DirtyRect, RenderSnapshot};
#[cfg(target_os = "linux")]
use crate::render::render_snapshot_to_mono_canvas;
use crate::render::{render_snapshot_to_mono_with_metrics, CellMetrics};
#[cfg(target_os = "linux")]
use crate::waveshare3ing::pack_landscape_mono_frame;

pub trait DisplayBackend {
    fn render(&mut self, snapshot: &RenderSnapshot, dirty: &[DirtyRect]) -> Result<()>;

    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities::interactive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayCapabilities {
    pub partial_refresh: bool,
    pub full_refresh_latency: Duration,
    pub min_refresh_interval: Duration,
    pub high_priority_bypasses_rate_limit: bool,
}

impl DisplayCapabilities {
    pub fn interactive() -> Self {
        Self {
            partial_refresh: true,
            full_refresh_latency: Duration::from_millis(0),
            min_refresh_interval: Duration::from_millis(250),
            high_priority_bypasses_rate_limit: true,
        }
    }

    pub fn full_refresh_only(
        full_refresh_latency: Duration,
        min_refresh_interval: Duration,
    ) -> Self {
        Self {
            partial_refresh: false,
            full_refresh_latency,
            min_refresh_interval,
            high_priority_bypasses_rate_limit: false,
        }
    }
}

pub fn create_backend(
    mode: DisplayMode,
    frame_dir: PathBuf,
    panel: PanelModel,
    metrics: CellMetrics,
) -> Result<Box<dyn DisplayBackend>> {
    match mode {
        DisplayMode::Mock => Ok(Box::new(MockDisplay::new(frame_dir, metrics)?)),
        DisplayMode::Debug => Ok(Box::new(DebugDisplay)),
        DisplayMode::Waveshare => Ok(Box::new(WaveshareDisplay::new(panel, metrics))),
    }
}

#[derive(Debug)]
pub struct MockDisplay {
    frame_dir: PathBuf,
    frame_index: u64,
    metrics: CellMetrics,
}

impl MockDisplay {
    pub fn new(frame_dir: PathBuf, metrics: CellMetrics) -> Result<Self> {
        fs::create_dir_all(&frame_dir)
            .with_context(|| format!("creating frame directory {}", frame_dir.display()))?;
        Ok(Self {
            frame_dir,
            frame_index: 0,
            metrics,
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

        let image = render_snapshot_to_mono_with_metrics(snapshot, self.metrics);

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
pub struct WaveshareDisplay {
    panel: PanelModel,
    metrics: CellMetrics,
    #[cfg(target_os = "linux")]
    device: Option<crate::waveshare3ing::Epd3in0gDevice>,
}

impl WaveshareDisplay {
    pub fn new(panel: PanelModel, metrics: CellMetrics) -> Self {
        Self {
            panel,
            metrics,
            #[cfg(target_os = "linux")]
            device: None,
        }
    }
}

impl DisplayBackend for WaveshareDisplay {
    fn capabilities(&self) -> DisplayCapabilities {
        DisplayCapabilities::full_refresh_only(Duration::from_secs(12), Duration::from_secs(12))
    }

    fn render(&mut self, _snapshot: &RenderSnapshot, _dirty: &[DirtyRect]) -> Result<()> {
        let profile = self.panel.profile();
        let _metrics = self.metrics;

        #[cfg(target_os = "linux")]
        {
            let expected_columns = profile.terminal_columns(_metrics.width as u16);
            let expected_rows = profile.terminal_rows(_metrics.height as u16);
            if _snapshot.columns != expected_columns || _snapshot.rows != expected_rows {
                bail!(
                    "{} expects a {}x{} terminal grid with the selected font, got {}x{}",
                    profile.name,
                    expected_columns,
                    expected_rows,
                    _snapshot.columns,
                    _snapshot.rows
                );
            }

            let frame = render_snapshot_to_mono_canvas(
                _snapshot,
                _metrics,
                profile.width_px as u32,
                profile.height_px as u32,
            );
            let packed = pack_landscape_mono_frame(&frame)?;
            let device = match self.device.as_mut() {
                Some(device) => device,
                None => {
                    let mut device = crate::waveshare3ing::Epd3in0gDevice::open_default()?;
                    device.init()?;
                    self.device.insert(device)
                }
            };
            device.display_packed(&packed)
        }

        #[cfg(not(target_os = "linux"))]
        bail!(
            "{} backend is only supported on Linux/Raspberry Pi targets",
            profile.name
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refresh::{RenderCell, TerminalColor};

    #[test]
    fn mock_display_writes_png_frame() {
        let dir = tempfile::tempdir().unwrap();
        let mut display =
            MockDisplay::new(dir.path().to_path_buf(), CellMetrics::STANDARD).unwrap();
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
