use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::hardware_test::HardwareTestPattern;
use crate::panel::PanelModel;
use crate::render::{CELL_HEIGHT, CELL_WIDTH};

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

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub display: DisplayMode,
    pub columns: u16,
    pub rows: u16,
    pub panel: PanelModel,
    pub frame_dir: PathBuf,
    pub command: Option<String>,
    pub hardware_test: Option<HardwareTestPattern>,
}

impl From<Cli> for RuntimeConfig {
    fn from(cli: Cli) -> Self {
        let profile = cli.panel.profile();
        let default_columns = match cli.display {
            DisplayMode::Waveshare => profile.terminal_columns(CELL_WIDTH as u16),
            DisplayMode::Mock | DisplayMode::Debug => 80,
        };
        let default_rows = match cli.display {
            DisplayMode::Waveshare => profile.terminal_rows(CELL_HEIGHT as u16),
            DisplayMode::Mock | DisplayMode::Debug => 24,
        };

        Self {
            display: cli.display,
            columns: cli.columns.unwrap_or(default_columns),
            rows: cli.rows.unwrap_or(default_rows),
            panel: cli.panel,
            frame_dir: cli.frame_dir,
            command: cli.command,
            hardware_test: cli.hardware_test,
        }
    }
}
