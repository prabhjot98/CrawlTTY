pub(crate) use anyhow::{Context, Result};
pub(crate) use crossterm::terminal::size as terminal_size;
pub(crate) use rand::Rng;
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use std::{
    env, fs,
    io::{self, Write},
    path::Path,
};

mod classes;
mod dungeon;
mod dungeon_gen;
mod input;
mod inventory;
mod items;
mod model;
mod rogue;
mod save;
mod skills;
mod town;
mod town_projects;
mod ui;

pub(crate) use classes::*;
pub(crate) use dungeon::*;
pub(crate) use dungeon_gen::*;
#[cfg(test)]
pub(crate) use input::terminal_event_to_input;
pub(crate) use input::{UiInput, read_ui_input, read_ui_input_nav, set_ratatui_owns_raw_mode};
pub(crate) use inventory::*;
pub(crate) use items::*;
pub(crate) use model::*;
#[allow(unused_imports)]
pub(crate) use rogue::*;
pub(crate) use save::*;
pub(crate) use skills::*;
pub(crate) use town::*;
pub(crate) use town_projects::*;
pub(crate) use ui::*;

fn main() -> Result<()> {
    fs::create_dir_all("saves").context("failed to create saves directory")?;

    if env::args().any(|arg| arg == "reset-save") {
        match fs::remove_file(SAVE_PATH) {
            Ok(()) => println!("Deleted {SAVE_PATH}."),
            Err(err) if err.kind() == io::ErrorKind::NotFound => println!("No save file found."),
            Err(err) => return Err(err).context("failed to delete save file"),
        }
        return Ok(());
    }

    {
        let mut terminal_session = TerminalSession::start()?;
        let mut character = load_or_create_character(&mut terminal_session.terminal)?;
        let mut town_message = take_startup_town_message(&mut character);
        save_character(&character)?;
        run_game(
            &mut terminal_session.terminal,
            &mut character,
            &mut town_message,
        )?;
    }

    println!("Saved. Goodbye.");
    Ok(())
}

fn take_startup_town_message(character: &mut Character) -> String {
    if character.active_dungeon.is_some() {
        String::new()
    } else {
        std::mem::take(&mut character.pending_town_message)
    }
}

fn run_game(
    terminal: &mut ratatui::DefaultTerminal,
    character: &mut Character,
    town_message: &mut String,
) -> Result<()> {
    loop {
        if character.active_dungeon.is_some() {
            dungeon_loop(character, terminal)?;
            if !character.pending_town_message.is_empty() {
                *town_message = std::mem::take(&mut character.pending_town_message);
            } else {
                town_message.clear();
            }
            save_character(character)?;
            continue;
        }

        terminal
            .draw(|frame| render_town(frame, character, town_message))
            .context("failed to draw town")?;
        let key = match read_ui_input() {
            Ok(UiInput::Key(key)) => key,
            Ok(UiInput::Redraw) => continue,
            Err(err) => {
                save_character(character)?;
                return Err(err);
            }
        };
        match key {
            'm' | 'M' => {
                merchant(character, terminal)?;
                terminal.clear()?;
                town_message.clear();
            }
            'b' | 'B' => {
                blacksmith(character, terminal)?;
                terminal.clear()?;
                town_message.clear();
            }
            's' | 'S' => {
                stash_menu(character, terminal)?;
                terminal.clear()?;
                town_message.clear();
            }
            'p' | 'P' => {
                town_projects_menu(character, terminal)?;
                terminal.clear()?;
                town_message.clear();
            }
            't' | 'T' => *town_message = quest_giver(character),
            'd' | 'D' => *town_message = enter_dungeon(character),
            'i' | 'I' => {
                inventory_screen(character, terminal)?;
                terminal.clear()?;
            }
            'a' | 'A' => {
                spend_attributes(character, terminal)?;
                terminal.clear()?;
            }
            'k' | 'K' => {
                skill_tree_menu(character, terminal)?;
                terminal.clear()?;
            }
            'q' | 'Q' => {
                save_character(character)?;
                break;
            }
            _ => {}
        }
        save_character(character)?;
    }
    Ok(())
}

struct TerminalSession {
    terminal: ratatui::DefaultTerminal,
}

impl TerminalSession {
    fn start() -> Result<Self> {
        let terminal = ratatui::try_init().context("failed to initialize terminal")?;
        set_ratatui_owns_raw_mode(true);
        Ok(Self { terminal })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        set_ratatui_owns_raw_mode(false);
        ratatui::restore();
    }
}

#[cfg(test)]
mod tests;
