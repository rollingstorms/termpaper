use anyhow::Result;
use clap::Parser;
use termpaper::config::{Cli, RuntimeConfig};
use termpaper::display::create_backend;
use termpaper::hardware_test::run_hardware_test;
use termpaper::pty::run_interactive;

fn main() -> Result<()> {
    let config = RuntimeConfig::from(Cli::parse());
    if let Some(pattern) = config.hardware_test {
        return run_hardware_test(pattern);
    }

    let mut display = create_backend(config.display, config.frame_dir.clone(), config.panel)?;
    run_interactive(
        config.rows,
        config.columns,
        config.command,
        display.as_mut(),
    )
}
