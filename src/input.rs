use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;

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

pub(crate) fn read_key_char_nav() -> char {
    let _raw_mode = RawModeGuard::new().expect("failed to enable raw mode");
    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read().expect("failed to read terminal event")
        {
            if modifiers.contains(KeyModifiers::CONTROL) && matches!(code, KeyCode::Char('c')) {
                disable_raw_mode().ok();
                std::process::exit(0);
            }
            match code {
                KeyCode::Char(c) => break c,
                KeyCode::Esc => break '\u{1b}',
                KeyCode::Enter => break '\n',
                KeyCode::Tab => break '\t',
                KeyCode::Up => break 'w',
                KeyCode::Down => break 's',
                _ => {}
            }
        }
    }
}

pub(crate) fn read_key_char() -> char {
    let _raw_mode = RawModeGuard::new().expect("failed to enable raw mode");
    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read().expect("failed to read terminal event")
        {
            if modifiers.contains(KeyModifiers::CONTROL) && matches!(code, KeyCode::Char('c')) {
                disable_raw_mode().ok();
                std::process::exit(0);
            }
            match code {
                KeyCode::Char(c) => break c,
                KeyCode::Esc => break '\u{1b}',
                KeyCode::Enter => break '\n',
                _ => {}
            }
        }
    }
}
