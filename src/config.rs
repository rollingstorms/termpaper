use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::hardware_test::HardwareTestPattern;
use crate::panel::PanelModel;
use crate::render::CellMetrics;

#[derive(Debug, Clone, Parser)]
#[command(name = "termpaper")]
#[command(about = "E-paper-native interactive terminal environment")]
pub struct Cli {
    #[arg(long, value_enum, default_value_t = DisplayMode::Mock)]
    pub display: DisplayMode,

    #[arg(long)]
    pub columns: Option<u16>,

    #[arg(long)]
    pub rows: Option<u16>,

    #[arg(long, value_enum, default_value_t = PanelModel::Waveshare3InG)]
    pub panel: PanelModel,

    #[arg(long, value_enum)]
    pub font: Option<TerminalFont>,

    #[arg(long, default_value = "frames")]
    pub frame_dir: PathBuf,

    #[arg(long)]
    pub command: Option<String>,

    #[arg(long, value_enum)]
    pub hardware_test: Option<HardwareTestPattern>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DisplayMode {
    Mock,
    Debug,
    Waveshare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TerminalFont {
    Standard,
    Compact,
}

impl TerminalFont {
    pub fn metrics(self) -> CellMetrics {
        match self {
            Self::Standard => CellMetrics::STANDARD,
            Self::Compact => CellMetrics::COMPACT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub display: DisplayMode,
    pub columns: u16,
    pub rows: u16,
    pub panel: PanelModel,
    pub font: TerminalFont,
    pub frame_dir: PathBuf,
    pub command: Option<String>,
    pub hardware_test: Option<HardwareTestPattern>,
}

impl From<Cli> for RuntimeConfig {
    fn from(cli: Cli) -> Self {
        let profile = cli.panel.profile();
        let font = cli.font.unwrap_or(match cli.display {
            DisplayMode::Waveshare => TerminalFont::Compact,
            DisplayMode::Mock | DisplayMode::Debug => TerminalFont::Standard,
        });
        let metrics = font.metrics();
        let default_columns = match cli.display {
            DisplayMode::Waveshare => profile.terminal_columns(metrics.width as u16),
            DisplayMode::Mock | DisplayMode::Debug => 80,
        };
        let default_rows = match cli.display {
            DisplayMode::Waveshare => profile.terminal_rows(metrics.height as u16),
            DisplayMode::Mock | DisplayMode::Debug => 24,
        };

        Self {
            display: cli.display,
            columns: cli.columns.unwrap_or(default_columns),
            rows: cli.rows.unwrap_or(default_rows),
            panel: cli.panel,
            font,
            frame_dir: cli.frame_dir,
            command: cli.command,
            hardware_test: cli.hardware_test,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{CELL_HEIGHT, CELL_WIDTH, COMPACT_CELL_HEIGHT, COMPACT_CELL_WIDTH};

    #[test]
    fn waveshare_defaults_to_compact_native_grid() {
        let config = RuntimeConfig::from(Cli {
            display: DisplayMode::Waveshare,
            columns: None,
            rows: None,
            panel: PanelModel::Waveshare3InG,
            font: None,
            frame_dir: "frames".into(),
            command: None,
            hardware_test: None,
        });

        assert_eq!(config.font, TerminalFont::Compact);
        assert_eq!(config.columns, (400 / COMPACT_CELL_WIDTH) as u16);
        assert_eq!(config.rows, (168 / COMPACT_CELL_HEIGHT) as u16);
    }

    #[test]
    fn standard_font_keeps_original_waveshare_grid() {
        let config = RuntimeConfig::from(Cli {
            display: DisplayMode::Waveshare,
            columns: None,
            rows: None,
            panel: PanelModel::Waveshare3InG,
            font: Some(TerminalFont::Standard),
            frame_dir: "frames".into(),
            command: None,
            hardware_test: None,
        });

        assert_eq!(config.columns, (400 / CELL_WIDTH) as u16);
        assert_eq!(config.rows, (168 / CELL_HEIGHT) as u16);
    }
}
