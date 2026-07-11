use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Parser)]
#[command(name = "termpaper")]
#[command(about = "E-paper-native interactive terminal environment")]
pub struct Cli {
    #[arg(long, value_enum, default_value_t = DisplayMode::Mock)]
    pub display: DisplayMode,

    #[arg(long, default_value_t = 80)]
    pub columns: u16,

    #[arg(long, default_value_t = 24)]
    pub rows: u16,

    #[arg(long, default_value = "frames")]
    pub frame_dir: PathBuf,

    #[arg(long)]
    pub command: Option<String>,
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
    pub frame_dir: PathBuf,
    pub command: Option<String>,
}

impl From<Cli> for RuntimeConfig {
    fn from(cli: Cli) -> Self {
        Self {
            display: cli.display,
            columns: cli.columns,
            rows: cli.rows,
            frame_dir: cli.frame_dir,
            command: cli.command,
        }
    }
}
