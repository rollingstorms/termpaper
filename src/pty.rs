use std::env;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

use crate::display::DisplayBackend;
use crate::input::{encode_event, InputEvent, Key, KeyEvent, Modifiers};
use crate::refresh::{coalesce_dirty_rects, RefreshPriority, RefreshScheduler};
use crate::terminal::TerminalState;

pub fn run_interactive(
    mut rows: u16,
    mut columns: u16,
    command: Option<String>,
    display: &mut dyn DisplayBackend,
) -> Result<()> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols: columns,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("opening PTY")?;

    let mut cmd = command_builder(command);
    cmd.env("TERM", "vt100");
    cmd.env("TERMPAPER", "1");

    let mut child = pair.slave.spawn_command(cmd).context("spawning shell")?;
    let mut reader = pair
        .master
        .try_clone_reader()
        .context("cloning PTY reader")?;
    let mut writer = pair.master.take_writer().context("taking PTY writer")?;
    let (pty_tx, pty_rx) = mpsc::channel::<std::io::Result<Vec<u8>>>();
    thread::spawn(move || {
        let mut read_buf = [0_u8; 8192];
        loop {
            match reader.read(&mut read_buf) {
                Ok(0) => break,
                Ok(count) => {
                    if pty_tx.send(Ok(read_buf[..count].to_vec())).is_err() {
                        break;
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(err) => {
                    let _ = pty_tx.send(Err(err));
                    break;
                }
            }
        }
    });

    enable_raw_mode().context("enabling raw keyboard mode")?;
    let raw_mode = RawModeGuard;
    let mut terminal = TerminalState::new(rows, columns);
    let capabilities = display.capabilities();
    let mut scheduler = RefreshScheduler::with_policy(
        capabilities.min_refresh_interval,
        capabilities.high_priority_bypasses_rate_limit,
    );
    let mut pending_dirty = Vec::new();
    let mut pending_input_feedback = false;

    loop {
        if event::poll(Duration::from_millis(1)).context("polling keyboard input")? {
            match event::read().context("reading terminal event")? {
                Event::Key(key) => {
                    let Some(input) = map_crossterm_key(key) else {
                        continue;
                    };
                    if input == InputEvent::Shutdown {
                        child.kill().context("terminating child process")?;
                        break;
                    }
                    let bytes = encode_event(&input, terminal.input_mode());
                    if !bytes.is_empty() {
                        writer.write_all(&bytes).context("writing input to PTY")?;
                        writer.flush().context("flushing PTY input")?;
                        pending_input_feedback = true;
                    }
                }
                Event::Resize(new_columns, new_rows) => {
                    if !capabilities.honors_host_resize {
                        continue;
                    }
                    columns = new_columns;
                    rows = new_rows;
                    pair.master
                        .resize(PtySize {
                            rows,
                            cols: columns,
                            pixel_width: 0,
                            pixel_height: 0,
                        })
                        .context("resizing PTY")?;
                    terminal.resize(rows, columns);
                    pending_dirty.clear();
                    pending_dirty.push(crate::refresh::DirtyRect::full(columns, rows));
                    scheduler.record_mutation(RefreshPriority::High);
                }
                _ => {}
            }
        }

        loop {
            match pty_rx.try_recv() {
                Ok(Ok(bytes)) => {
                    let dirty = terminal.process_output(&bytes);
                    if !dirty.is_empty() {
                        let priority = if pending_input_feedback {
                            pending_input_feedback = false;
                            RefreshPriority::High
                        } else {
                            RefreshPriority::Normal
                        };
                        scheduler.record_mutation(priority);
                        pending_dirty.extend(dirty);
                    }
                }
                Ok(Err(err)) => return Err(err).context("reading PTY output"),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }

        if !pending_dirty.is_empty() && scheduler.should_refresh(Instant::now()) {
            coalesce_dirty_rects(&mut pending_dirty);
            display.render(&terminal.snapshot(), &pending_dirty)?;
            pending_dirty.clear();
        }

        if child
            .try_wait()
            .context("checking child process")?
            .is_some()
        {
            if !pending_dirty.is_empty() {
                coalesce_dirty_rects(&mut pending_dirty);
                display.render(&terminal.snapshot(), &pending_dirty)?;
            }
            break;
        }

        thread::sleep(Duration::from_millis(2));
    }

    drop(raw_mode);
    Ok(())
}

fn command_builder(command: Option<String>) -> CommandBuilder {
    match command {
        Some(command) if command_contains_shell_syntax(&command) => {
            let mut builder = CommandBuilder::new("/bin/sh");
            builder.args(["-lc", command.as_str()]);
            builder
        }
        Some(command) => CommandBuilder::new(command),
        None => CommandBuilder::new(env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())),
    }
}

fn command_contains_shell_syntax(command: &str) -> bool {
    command.chars().any(char::is_whitespace)
        || command.chars().any(|ch| {
            matches!(
                ch,
                '|' | '&' | ';' | '<' | '>' | '*' | '?' | '$' | '\'' | '"'
            )
        })
}

fn map_crossterm_key(key: crossterm::event::KeyEvent) -> Option<InputEvent> {
    let modifiers = Modifiers {
        control: key.modifiers.contains(KeyModifiers::CONTROL),
        alt: key.modifiers.contains(KeyModifiers::ALT),
        shift: key.modifiers.contains(KeyModifiers::SHIFT),
    };

    if modifiers.control && key.code == KeyCode::Char(']') {
        return Some(InputEvent::Shutdown);
    }

    let key = match key.code {
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Tab | KeyCode::BackTab => Key::Tab,
        KeyCode::Delete => Key::Delete,
        KeyCode::Insert => Key::Insert,
        KeyCode::F(n) => Key::Function(n),
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Esc => Key::Escape,
        _ => return None,
    };

    Some(InputEvent::Key(KeyEvent { key, modifiers }))
}

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_shell_command_strings() {
        assert!(!command_contains_shell_syntax("/bin/ls"));
        assert!(command_contains_shell_syntax("ls -la"));
        assert!(command_contains_shell_syntax("printf hello; true"));
        assert!(command_contains_shell_syntax("echo $SHELL"));
    }

    #[test]
    fn maps_control_bracket_to_local_shutdown() {
        let input = map_crossterm_key(crossterm::event::KeyEvent::new(
            KeyCode::Char(']'),
            KeyModifiers::CONTROL,
        ));

        assert_eq!(input, Some(InputEvent::Shutdown));
    }
}
