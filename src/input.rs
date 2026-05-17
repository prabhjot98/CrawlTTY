use anyhow::{Context, Result, anyhow};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
};

static RATATUI_OWNS_RAW_MODE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiInput {
    Key(char),
    Redraw,
}

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

pub(crate) fn read_ui_input_nav() -> Result<UiInput> {
    read_ui_input_with_navigation(true)
}

pub(crate) fn read_ui_input() -> Result<UiInput> {
    read_ui_input_with_navigation(false)
}

fn read_ui_input_with_navigation(navigation: bool) -> Result<UiInput> {
    let _raw_mode = if RATATUI_OWNS_RAW_MODE.load(Ordering::Relaxed) {
        None
    } else {
        Some(RawModeGuard::new().context("failed to enable raw mode")?)
    };
    loop {
        if let Some(input) = terminal_event_to_input(
            event::read().context("failed to read terminal event")?,
            navigation,
        )? {
            return Ok(input);
        }
    }
}

pub(crate) fn terminal_event_to_input(event: Event, navigation: bool) -> Result<Option<UiInput>> {
    match event {
        Event::Resize(_, _) => Ok(Some(UiInput::Redraw)),
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) => {
            if modifiers.contains(KeyModifiers::CONTROL) && matches!(code, KeyCode::Char('c')) {
                return Err(anyhow!("input interrupted"));
            }
            let key = match code {
                KeyCode::Char(c) => Some(c),
                KeyCode::Backspace => Some('\u{8}'),
                KeyCode::Esc => Some('\u{1b}'),
                KeyCode::Enter => Some('\n'),
                KeyCode::Tab => Some('\t'),
                KeyCode::Up if navigation => Some('w'),
                KeyCode::Down if navigation => Some('s'),
                KeyCode::Left if navigation => Some('a'),
                KeyCode::Right if navigation => Some('d'),
                _ => None,
            };
            Ok(key.map(UiInput::Key))
        }
        Event::Key(_) => Ok(None),
        _ => Ok(None),
    }
}
