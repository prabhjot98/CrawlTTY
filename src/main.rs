use anyhow::{Context, Result};
use crossterm::terminal::size as terminal_size;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
};

mod input;

use input::{read_key_char, read_key_char_nav};

include!("model.rs");
include!("items.rs");
include!("ui.rs");
include!("save.rs");
include!("town.rs");
include!("skills.rs");
include!("dungeon_gen.rs");
include!("dungeon.rs");
include!("inventory.rs");

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
    save_character(&character)?;
    let mut town_message = String::new();

    loop {
        if character.active_dungeon.is_some() {
            dungeon_loop(&mut character)?;
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
        match read_key_char() {
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
