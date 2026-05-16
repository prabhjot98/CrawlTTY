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
                KeyCode::Backspace => return Ok('\u{8}'),
                KeyCode::Esc => return Ok('\u{1b}'),
                KeyCode::Enter => return Ok('\n'),
                KeyCode::Tab => return Ok('\t'),
                KeyCode::Up if navigation => return Ok('w'),
                KeyCode::Down if navigation => return Ok('s'),
                KeyCode::Left if navigation => return Ok('a'),
                KeyCode::Right if navigation => return Ok('d'),
                _ => {}
            }
        }
    }
}
