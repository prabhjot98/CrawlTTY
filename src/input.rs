use anyhow::{Context, Result, anyhow};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

static RATATUI_OWNS_RAW_MODE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiInput {
    Key(char),
    Redraw,
    Tick,
}

pub(crate) const KEY_ARROW_UP: char = '\u{10}';
pub(crate) const KEY_ARROW_DOWN: char = '\u{0e}';

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

#[allow(dead_code)]
pub(crate) fn read_ui_input_nav() -> Result<UiInput> {
    read_ui_input_with_options(true, false, None)
}

pub(crate) fn read_ui_input_nav_timed(timeout: Duration) -> Result<UiInput> {
    read_ui_input_with_options(true, false, Some(timeout))
}

pub(crate) fn read_ui_input() -> Result<UiInput> {
    read_ui_input_with_options(false, false, None)
}

pub(crate) fn read_ui_input_timed(timeout: Duration) -> Result<UiInput> {
    read_ui_input_with_options(false, false, Some(timeout))
}

#[allow(dead_code)]
pub(crate) fn read_ui_input_raw_arrows() -> Result<UiInput> {
    read_ui_input_with_options(false, true, None)
}

pub(crate) fn read_ui_input_raw_arrows_timed(timeout: Duration) -> Result<UiInput> {
    read_ui_input_with_options(false, true, Some(timeout))
}

fn read_ui_input_with_options(
    navigation: bool,
    raw_arrows: bool,
    timeout: Option<Duration>,
) -> Result<UiInput> {
    let _raw_mode = if RATATUI_OWNS_RAW_MODE.load(Ordering::Relaxed) {
        None
    } else {
        Some(RawModeGuard::new().context("failed to enable raw mode")?)
    };
    loop {
        if let Some(timeout) = timeout {
            if !event::poll(timeout).context("failed to poll terminal event")? {
                return Ok(UiInput::Tick);
            }
        }
        if let Some(input) = terminal_event_to_input_with_options(
            event::read().context("failed to read terminal event")?,
            navigation,
            raw_arrows,
        )? {
            return Ok(input);
        }
    }
}

#[cfg(test)]
pub(crate) fn terminal_event_to_input(event: Event, navigation: bool) -> Result<Option<UiInput>> {
    terminal_event_to_input_with_options(event, navigation, false)
}

#[cfg(test)]
pub(crate) fn terminal_event_to_input_raw_arrows(event: Event) -> Result<Option<UiInput>> {
    terminal_event_to_input_with_options(event, false, true)
}

#[cfg(test)]
pub(crate) fn terminal_event_timeout_to_input(
    event: Option<Event>,
    navigation: bool,
    raw_arrows: bool,
) -> Result<UiInput> {
    match event {
        Some(event) => Ok(
            terminal_event_to_input_with_options(event, navigation, raw_arrows)?
                .unwrap_or(UiInput::Tick),
        ),
        None => Ok(UiInput::Tick),
    }
}

fn terminal_event_to_input_with_options(
    event: Event,
    navigation: bool,
    raw_arrows: bool,
) -> Result<Option<UiInput>> {
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
                KeyCode::Up if raw_arrows => Some(KEY_ARROW_UP),
                KeyCode::Down if navigation => Some('s'),
                KeyCode::Down if raw_arrows => Some(KEY_ARROW_DOWN),
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
