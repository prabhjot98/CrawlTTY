use anyhow::{Context, Result, anyhow};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
};

static RATATUI_OWNS_RAW_MODE: AtomicBool = AtomicBool::new(false);

pub(crate) fn set_ratatui_owns_raw_mode(owns_raw_mode: bool) {
    RATATUI_OWNS_RAW_MODE.store(owns_raw_mode, Ordering::Relaxed);
}

#[cfg(test)]
pub(crate) fn ratatui_owns_raw_mode_for_test() -> bool {
    RATATUI_OWNS_RAW_MODE.load(Ordering::Relaxed)
}

pub(crate) struct LegacyScreenTerminalMode {
    restored: bool,
}

impl LegacyScreenTerminalMode {
    pub(crate) fn enter() -> Result<Self> {
        set_terminal_raw_mode(false).context("failed to release raw mode for legacy screen")?;
        set_ratatui_owns_raw_mode(false);
        Ok(Self { restored: false })
    }

    pub(crate) fn restore_ratatui(&mut self) -> Result<()> {
        if self.restored {
            return Ok(());
        }
        set_terminal_raw_mode(true).context("failed to restore raw mode after legacy screen")?;
        set_ratatui_owns_raw_mode(true);
        self.restored = true;
        Ok(())
    }
}

impl Drop for LegacyScreenTerminalMode {
    fn drop(&mut self) {
        if !self.restored {
            if set_terminal_raw_mode(true).is_ok() {
                set_ratatui_owns_raw_mode(true);
            }
        }
    }
}

#[cfg(not(test))]
fn set_terminal_raw_mode(enabled: bool) -> io::Result<()> {
    if enabled {
        enable_raw_mode()
    } else {
        disable_raw_mode()
    }
}

#[cfg(test)]
fn set_terminal_raw_mode(_enabled: bool) -> io::Result<()> {
    Ok(())
}

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

pub(crate) fn read_key_char_nav() -> Result<char> {
    read_key_char_with_navigation(true)
}

pub(crate) fn read_key_char() -> Result<char> {
    read_key_char_with_navigation(false)
}

pub(crate) fn read_key_char_nav_or_message(message: &mut String) -> Option<char> {
    match read_key_char_nav() {
        Ok(key) => {
            message.clear();
            Some(key)
        }
        Err(err) => {
            *message = format!("Input error: {err:#}");
            None
        }
    }
}

pub(crate) fn read_key_char_or_message(message: &mut String) -> Option<char> {
    match read_key_char() {
        Ok(key) => {
            message.clear();
            Some(key)
        }
        Err(err) => {
            *message = format!("Input error: {err:#}");
            None
        }
    }
}

fn read_key_char_with_navigation(navigation: bool) -> Result<char> {
    let _raw_mode = if RATATUI_OWNS_RAW_MODE.load(Ordering::Relaxed) {
        None
    } else {
        Some(RawModeGuard::new().context("failed to enable raw mode")?)
    };
    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read().context("failed to read terminal event")?
        {
            if modifiers.contains(KeyModifiers::CONTROL) && matches!(code, KeyCode::Char('c')) {
                return Err(anyhow!("input interrupted"));
            }
            match code {
                KeyCode::Char(c) => return Ok(c),
                KeyCode::Esc => return Ok('\u{1b}'),
                KeyCode::Enter => return Ok('\n'),
                KeyCode::Tab if navigation => return Ok('\t'),
                KeyCode::Up if navigation => return Ok('w'),
                KeyCode::Down if navigation => return Ok('s'),
                _ => {}
            }
        }
    }
}
