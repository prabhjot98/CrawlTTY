pub(crate) use anyhow::{Context, Result};
pub(crate) use crossterm::terminal::size as terminal_size;
pub(crate) use rand::Rng;
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use std::{
    env, fs,
    io::{self, Write},
    path::Path,
};

mod dungeon;
mod dungeon_gen;
mod input;
mod inventory;
mod items;
mod model;
mod save;
mod skills;
mod town;
mod ui;

pub(crate) use dungeon::*;
pub(crate) use dungeon_gen::*;
use input::{read_key_char, read_key_char_nav_or_message, read_key_char_or_message};
pub(crate) use inventory::*;
pub(crate) use items::*;
pub(crate) use model::*;
pub(crate) use save::*;
pub(crate) use skills::*;
pub(crate) use town::*;
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

    let mut character = load_or_create_character()?;
    let mut town_message = std::mem::take(&mut character.pending_town_message);
    save_character(&character)?;

    loop {
        if character.active_dungeon.is_some() {
            dungeon_loop(&mut character)?;
            if !character.pending_town_message.is_empty() {
                town_message = std::mem::take(&mut character.pending_town_message);
                save_character(&character)?;
            } else {
                town_message.clear();
            }
            continue;
        }

        clear_screen();
        print_town(&character);
        if !town_message.is_empty() {
            println!("{YELLOW}{town_message}{RESET}");
        }
        println!("\n{BOLD}Town services{RESET}");
        println!("Use the footer commands below to choose a service.");
        print_footer(&[
            &format!(
                "{BOLD}Town:{RESET} {GREEN}h{RESET}=healer  {GREEN}m{RESET}=merchant  {GREEN}b{RESET}=blacksmith  {GREEN}s{RESET}=stash  {GREEN}t{RESET}=quest  {GREEN}d{RESET}=dungeon"
            ),
            &format!(
                "{GREEN}i{RESET}=inventory  {GREEN}a{RESET}=attributes  {GREEN}k{RESET}=skill tree  {RED}q{RESET}=save+quit"
            ),
        ]);
        let key = match read_key_char() {
            Ok(key) => key,
            Err(err) => {
                save_character(&character)?;
                return Err(err);
            }
        };
        match key {
            'h' | 'H' => {
                healer(&mut character);
                town_message = "Healed to full HP and mana.".to_string();
            }
            'm' | 'M' => {
                merchant(&mut character);
                town_message.clear();
            }
            'b' | 'B' => {
                blacksmith(&mut character);
                town_message.clear();
            }
            's' | 'S' => {
                stash_menu(&mut character);
                town_message.clear();
            }
            't' | 'T' => town_message = quest_giver(&mut character),
            'd' | 'D' => town_message = enter_dungeon(&mut character),
            'i' | 'I' => {
                inventory_screen(&mut character);
            }
            'a' | 'A' => spend_attributes(&mut character),
            'k' | 'K' => skill_tree_menu(&mut character),
            'q' | 'Q' => {
                save_character(&character)?;
                println!("Saved. Goodbye.");
                break;
            }
            _ => {}
        }
        save_character(&character)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
