use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, size as terminal_size},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
};

const SAVE_PATH: &str = "saves/save.json";
const MAP_W: i32 = 40;
const MAP_H: i32 = 16;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";
const BRIGHT_BLACK: &str = "\x1b[90m";

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DeathMode {
    Softcore,
    Hardcore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Item {
    name: String,
    kind: ItemKind,
    value: u32,
    #[serde(default)]
    damage_min: i32,
    #[serde(default)]
    damage_max: i32,
    #[serde(default)]
    armor: i32,
    #[serde(default)]
    dodge: i32,
    #[serde(default)]
    speed: i32,
    #[serde(default)]
    rarity: Rarity,
    #[serde(default = "default_item_level")]
    item_level: u32,
    #[serde(default)]
    required_strength: u32,
    #[serde(default)]
    required_dexterity: u32,
    #[serde(default)]
    required_intelligence: u32,
    #[serde(default)]
    upgrade_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
enum Rarity {
    #[default]
    Common,
    Magic,
    Rare,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum ItemKind {
    HealthPotion,
    ManaPotion,
    Weapon,
    Armor,
    Shield,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum EliteModifier {
    Armored,
    Swift,
    Vampiric,
    Burning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Enemy {
    name: String,
    glyph: char,
    x: i32,
    y: i32,
    hp: i32,
    max_hp: i32,
    damage_min: i32,
    damage_max: i32,
    armor: i32,
    speed: i32,
    xp: u32,
    gold_min: u32,
    gold_max: u32,
    is_boss: bool,
    #[serde(default)]
    stunned_turns: u32,
    #[serde(default)]
    bleed_turns: u32,
    #[serde(default)]
    bleed_damage: i32,
    #[serde(default)]
    guarding: bool,
    #[serde(default)]
    elite_modifier: Option<EliteModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Chest {
    x: i32,
    y: i32,
    opened: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Dungeon {
    floor: u32,
    player_x: i32,
    player_y: i32,
    stairs_x: i32,
    stairs_y: i32,
    enemies: Vec<Enemy>,
    chests: Vec<Chest>,
    log: Vec<String>,
    #[serde(default)]
    tiles: Vec<char>,
    #[serde(default)]
    bell_wave_tiles: Vec<(i32, i32)>,
    #[serde(default)]
    boss_turn_counter: u32,
    #[serde(default)]
    log_turn: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Character {
    name: String,
    class_name: String,
    death_mode: DeathMode,
    level: u32,
    xp: u32,
    gold: u32,
    strength: u32,
    dexterity: u32,
    intelligence: u32,
    #[serde(default)]
    unspent_attributes: u32,
    #[serde(default)]
    unspent_skills: u32,
    #[serde(default = "default_skill_rank")]
    cleave_rank: u32,
    #[serde(default = "default_skill_rank")]
    shield_bash_rank: u32,
    #[serde(default = "default_skill_rank")]
    battle_cry_rank: u32,
    #[serde(default = "default_skill_rank")]
    deep_cut_rank: u32,
    #[serde(default = "default_skill_rank")]
    iron_guard_rank: u32,
    #[serde(default = "default_skill_rank")]
    second_wind_rank: u32,
    hp: u32,
    mana: u32,
    inventory: Vec<Item>,
    stash: Vec<Item>,
    equipped_weapon: Item,
    equipped_armor: Item,
    equipped_shield: Item,
    bellkeeper_defeated: bool,
    #[serde(default)]
    act1_completed: bool,
    #[serde(default)]
    cleave_cooldown: u32,
    #[serde(default)]
    shield_bash_cooldown: u32,
    #[serde(default)]
    battle_cry_cooldown: u32,
    #[serde(default)]
    battle_cry_turns: u32,
    #[serde(default)]
    active_dungeon: Option<Dungeon>,
    #[serde(default)]
    weapon_shards: u32,
    #[serde(default)]
    armor_shards: u32,
    #[serde(default)]
    shield_shards: u32,
}

fn default_skill_rank() -> u32 {
    1
}

fn default_item_level() -> u32 {
    1
}

impl Character {
    fn new(name: String, death_mode: DeathMode) -> Self {
        let strength = 6;
        let dexterity = 3;
        let intelligence = 1;
        let max_hp = 10 + strength * 5;
        let max_mana = 10 + intelligence * 5;
        Self {
            name,
            class_name: "Ironbound".to_string(),
            death_mode,
            level: 1,
            xp: 0,
            gold: 50,
            strength,
            dexterity,
            intelligence,
            unspent_attributes: 0,
            unspent_skills: 0,
            cleave_rank: 1,
            shield_bash_rank: 1,
            battle_cry_rank: 1,
            deep_cut_rank: 1,
            iron_guard_rank: 1,
            second_wind_rank: 1,
            hp: max_hp,
            mana: max_mana,
            inventory: vec![health_potion(), health_potion(), mana_potion()],
            stash: Vec::new(),
            equipped_weapon: rusted_sword(),
            equipped_armor: cloth_tunic(),
            equipped_shield: worn_shield(),
            bellkeeper_defeated: false,
            act1_completed: false,
            cleave_cooldown: 0,
            shield_bash_cooldown: 0,
            battle_cry_cooldown: 0,
            battle_cry_turns: 0,
            active_dungeon: None,
            weapon_shards: 0,
            armor_shards: 0,
            shield_shards: 0,
        }
    }

    fn max_hp(&self) -> u32 {
        10 + self.strength * 5
    }
    fn max_mana(&self) -> u32 {
        10 + self.intelligence * 5
    }
    fn hit_rating(&self) -> u32 {
        10 + self.dexterity * 5
    }
    fn dodge_rating(&self) -> u32 {
        (10 + self.dexterity as i32 * 3 + self.equipped_shield.dodge + self.equipped_armor.dodge)
            .max(0) as u32
    }
    fn speed(&self) -> u32 {
        (10 + self.dexterity as i32 * 5
            + self.equipped_weapon.speed
            + self.equipped_armor.speed
            + self.equipped_shield.speed)
            .max(1) as u32
    }
    fn armor(&self) -> i32 {
        self.equipped_armor.armor + self.equipped_shield.armor + iron_guard_armor_bonus(self)
    }
    fn weapon_damage(&self) -> (i32, i32) {
        (
            self.equipped_weapon.damage_min + (self.strength as i32 / 4),
            self.equipped_weapon.damage_max + (self.strength as i32 / 3),
        )
    }
}

fn item(
    name: &str,
    kind: ItemKind,
    value: u32,
    damage_min: i32,
    damage_max: i32,
    armor: i32,
    dodge: i32,
    speed: i32,
) -> Item {
    let required_strength = match kind {
        ItemKind::Weapon => damage_max.max(0) as u32,
        ItemKind::Armor | ItemKind::Shield => (armor + 3).max(0) as u32,
        ItemKind::HealthPotion | ItemKind::ManaPotion => 0,
    };
    let required_dexterity = if kind == ItemKind::Weapon && name.contains("Sword") {
        2
    } else {
        0
    };
    Item {
        name: name.to_string(),
        kind,
        value,
        damage_min,
        damage_max,
        armor,
        dodge,
        speed,
        rarity: Rarity::Common,
        item_level: 1,
        required_strength,
        required_dexterity,
        required_intelligence: 0,
        upgrade_level: 0,
    }
}

fn item_with_rarity(
    name: &str,
    kind: ItemKind,
    value: u32,
    damage_min: i32,
    damage_max: i32,
    armor: i32,
    dodge: i32,
    speed: i32,
    rarity: Rarity,
    item_level: u32,
    required_strength: u32,
    required_dexterity: u32,
    required_intelligence: u32,
) -> Item {
    Item {
        name: name.to_string(),
        kind,
        value,
        damage_min,
        damage_max,
        armor,
        dodge,
        speed,
        rarity,
        item_level,
        required_strength,
        required_dexterity,
        required_intelligence,
        upgrade_level: 0,
    }
}
fn health_potion() -> Item {
    item(
        "Health Potion (restores 25% HP)",
        ItemKind::HealthPotion,
        15,
        0,
        0,
        0,
        0,
        0,
    )
}
fn mana_potion() -> Item {
    item(
        "Mana Potion (restores 25% mana)",
        ItemKind::ManaPotion,
        15,
        0,
        0,
        0,
        0,
        0,
    )
}
fn rusted_sword() -> Item {
    item(
        "Rusted Sword (3-5 dmg, STR F, DEX F)",
        ItemKind::Weapon,
        20,
        3,
        5,
        0,
        0,
        0,
    )
}
fn crude_axe() -> Item {
    item(
        "Crude Axe (4-6 dmg, STR F)",
        ItemKind::Weapon,
        60,
        4,
        6,
        0,
        0,
        -1,
    )
}
fn cloth_tunic() -> Item {
    item("Cloth Tunic (+1 armor)", ItemKind::Armor, 12, 0, 0, 1, 0, 0)
}
fn battered_mail() -> Item {
    item(
        "Battered Mail (+2 armor, -5 speed)",
        ItemKind::Armor,
        55,
        0,
        0,
        2,
        0,
        -5,
    )
}
fn worn_shield() -> Item {
    item(
        "Worn Shield (+1 armor, +2 dodge)",
        ItemKind::Shield,
        40,
        0,
        0,
        1,
        2,
        0,
    )
}

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

    loop {
        if character.active_dungeon.is_some() {
            dungeon_loop(&mut character)?;
            continue;
        }

        clear_screen();
        print_town(&character);
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
            'h' | 'H' => healer(&mut character),
            'm' | 'M' => merchant(&mut character),
            'b' | 'B' => blacksmith(&mut character),
            's' | 'S' => stash_menu(&mut character),
            't' | 'T' => quest_giver(&mut character),
            'd' | 'D' => enter_dungeon(&mut character),
            'i' | 'I' => inventory_screen(&mut character),
            'a' | 'A' => spend_attributes(&mut character),
            'k' | 'K' => skill_tree_menu(&mut character),
            'q' | 'Q' => {
                save_character(&character)?;
                println!("Saved. Goodbye.");
                break;
            }
            _ => pause("Unknown action."),
        }
        save_character(&character)?;
    }
    Ok(())
}

fn load_or_create_character() -> Result<Character> {
    if Path::new(SAVE_PATH).exists() {
        let data = fs::read_to_string(SAVE_PATH).context("failed to read save file")?;
        return serde_json::from_str(&data).context("failed to parse save file");
    }
    println!("ASHEN DEPTHS");
    println!("ASCII terminal action RPG prototype");
    let name = prompt("Character name: ");
    println!("{BOLD}Choose death mode:{RESET}");
    println!("{GREEN}Softcore{RESET}: death returns you to town.");
    println!("{RED}Hardcore{RESET}: death permanently ends the character.");
    print_footer(&[&format!(
        "{BOLD}Character creation:{RESET} {GREEN}1{RESET}=Softcore  {RED}2{RESET}=Hardcore"
    )]);
    let death_mode = loop {
        match read_key_char() {
            '1' => break DeathMode::Softcore,
            '2' => break DeathMode::Hardcore,
            _ => println!("Choose 1 or 2."),
        }
    };
    Ok(Character::new(name.trim().to_string(), death_mode))
}

fn save_character(character: &Character) -> Result<()> {
    let data = serde_json::to_string_pretty(character).context("failed to serialize save")?;
    fs::write(SAVE_PATH, data).context("failed to write save")
}

fn print_town(c: &Character) {
    println!("{BOLD}{CYAN}+--------------------------------------------------+{RESET}");
    println!("{BOLD}{CYAN}| Town: Hollow's Rest                              |{RESET}");
    println!("{BOLD}{CYAN}+--------------------------------------------------+{RESET}");
    println!(
        "{BOLD}{}{RESET} the {GREEN}{}{RESET}  {}  {}  {}",
        c.name,
        c.class_name,
        level_text(c.level),
        xp_text(c.xp),
        gold_text(c.gold)
    );
    println!(
        "{}  {}",
        hp_text(c.hp, c.max_hp()),
        mana_text(c.mana, c.max_mana())
    );
    println!(
        "{}  {}  {}  {}  {}  {}",
        strength_text(c.strength),
        dexterity_text(c.dexterity),
        intelligence_text(c.intelligence),
        hit_text(c.hit_rating()),
        dodge_text(c.dodge_rating()),
        speed_text(c.speed())
    );
    println!(
        "{}  {}",
        unspent_attributes_text(c.unspent_attributes),
        unspent_skills_text(c.unspent_skills)
    );
    println!(
        "{BOLD}Weapon:{RESET} {}",
        colored_item_name(&c.equipped_weapon)
    );
    println!(
        "{BOLD}Armor :{RESET} {}",
        colored_item_name(&c.equipped_armor)
    );
    println!(
        "{BOLD}Shield:{RESET} {}",
        colored_item_name(&c.equipped_shield)
    );
    if c.act1_completed {
        println!(
            "{GREEN}Act I complete:{RESET} The Hollow Marches are safe for now. Act II is unlocked as a placeholder."
        );
    } else if c.bellkeeper_defeated {
        println!(
            "{YELLOW}Quest ready to turn in:{RESET} Speak with Warden Mara ({GREEN}t{RESET}) about the Bellkeeper."
        );
    } else {
        println!(
            "Quest: Kill the Bellkeeper below the crypt. Speak with Warden Mara ({GREEN}t{RESET}) for details."
        );
    }
}

fn colored_stat(label: &str, value: impl std::fmt::Display, color: &str) -> String {
    format!("{color}{label} {value}{RESET}")
}

fn strength_text(value: u32) -> String {
    colored_stat("STR", value, RED)
}

fn dexterity_text(value: u32) -> String {
    colored_stat("DEX", value, GREEN)
}

fn intelligence_text(value: u32) -> String {
    colored_stat("INT", value, BLUE)
}

fn hp_text(current: u32, max: u32) -> String {
    format!("{RED}HP {current}/{max}{RESET}")
}

fn mana_text(current: u32, max: u32) -> String {
    format!("{BLUE}Mana {current}/{max}{RESET}")
}

fn gold_text(value: u32) -> String {
    format!("{YELLOW}Gold {value}{RESET}")
}

fn xp_text(value: u32) -> String {
    format!("{MAGENTA}XP {value}{RESET}")
}

fn level_text(value: u32) -> String {
    format!("{CYAN}Level {value}{RESET}")
}

fn hit_text(value: u32) -> String {
    colored_stat("Hit", value, CYAN)
}

fn dodge_text(value: u32) -> String {
    colored_stat("Dodge", value, GREEN)
}

fn speed_text(value: u32) -> String {
    colored_stat("Speed", value, YELLOW)
}

fn armor_text(value: i32) -> String {
    colored_stat("Armor", value, WHITE)
}

fn unspent_attributes_text(value: u32) -> String {
    format!("{CYAN}Unspent attributes: {value}{RESET}")
}

fn unspent_skills_text(value: u32) -> String {
    format!("{MAGENTA}Unspent skills: {value}{RESET}")
}

fn shard_text(label: &str, value: u32) -> String {
    format!("{WHITE}{label} {value}{RESET}")
}

fn damage_text(value: impl std::fmt::Display) -> String {
    format!("{RED}{value} damage{RESET}")
}

fn xp_reward_text(value: u32) -> String {
    format!("{MAGENTA}{value} XP{RESET}")
}

fn gold_reward_text(value: u32) -> String {
    format!("{YELLOW}{value} gold{RESET}")
}

fn heal_amount_text(value: u32) -> String {
    format!("{GREEN}{value} HP{RESET}")
}

fn push_level_up_logs(log: &mut Vec<String>, levels_gained: &[u32]) {
    for level in levels_gained {
        log.push(format!(
            "{BOLD}{MAGENTA}*** LEVEL UP! Reached level {level}. +3 attributes, +1 skill point. ***{RESET}"
        ));
        log.push(format!(
            "{CYAN}Town reminder:{RESET} press {GREEN}a{RESET} for attributes and {GREEN}k{RESET} for skills."
        ));
    }
}

fn quest_giver(c: &mut Character) {
    clear_screen();
    println!("{BOLD}{CYAN}Warden Mara{RESET}");
    if c.act1_completed {
        println!("Mara stands at the northern road, watching ash drift over the marsh.");
        println!(
            "\"You broke the bell's curse. Beyond this road lie the Glass Wastes... but that journey is not yet playable.\""
        );
        pause("Act II placeholder: The Glass Wastes will open in a later milestone.");
    } else if c.bellkeeper_defeated {
        println!("\"The bells are silent. Hollow's Rest owes you its next dawn.\"");
        println!("Quest complete: Silence the Bellkeeper");
        println!(
            "Reward: {YELLOW}100 gold{RESET}, {MAGENTA}+1 skill point{RESET}, {GREEN}full heal{RESET}, {CYAN}Act II placeholder unlocked{RESET}."
        );
        c.gold += 100;
        c.unspent_skills += 1;
        c.hp = c.max_hp();
        c.mana = c.max_mana();
        c.act1_completed = true;
        pause("Act I complete. The road to the Glass Wastes is now visible.");
    } else {
        println!(
            "\"A cursed bell tolls beneath the crypt. Each ring wakes more dead. Descend, find the Bellkeeper, and end it.\""
        );
        println!("Objective: defeat the Bellkeeper on floor 3 of the Hollow Crypts.");
        pause("Quest accepted: Silence the Bellkeeper.");
    }
}

fn healer(c: &mut Character) {
    c.hp = c.max_hp();
    c.mana = c.max_mana();
    pause("The healer restores your health and mana.");
}

fn merchant(c: &mut Character) {
    let mut selected = 0usize;
    let mut message = String::new();
    let options = [
        "Buy Health Potion - 15 gold",
        "Buy Mana Potion - 15 gold",
        "Sell items",
    ];
    loop {
        clamp_selection(&mut selected, options.len());
        clear_screen();
        println!("{BOLD}{YELLOW}Merchant{RESET} - {}", gold_text(c.gold));
        println!("Selling gives 25% of item value.");
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();
        println!("{BOLD}Services{RESET}");
        for (i, option) in options.iter().enumerate() {
            let marker = if i == selected {
                format!("{GREEN}>{RESET}")
            } else {
                " ".to_string()
            };
            println!("{marker} {option}");
        }
        println!();
        print_inventory_preview(c, inventory_visible_rows(12));
        print_footer(&[&format!(
            "{BOLD}Merchant:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=choose  {RED}Esc{RESET}=back"
        )]);
        match read_key_char_nav() {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < options.len() {
                    selected += 1;
                }
            }
            '\n' => match selected {
                0 => message = buy_item_message(c, health_potion()),
                1 => message = buy_item_message(c, mana_potion()),
                2 => sell_item_screen(c),
                _ => {}
            },
            _ => message = "Unknown merchant command.".to_string(),
        }
    }
}

fn blacksmith(c: &mut Character) {
    let mut selected = 0usize;
    let mut message = String::new();
    let options = [
        "Buy Crude Axe - 60 gold",
        "Buy Battered Mail - 55 gold",
        "Buy Worn Shield - 40 gold",
        "Salvage carried gear for shards",
        "Sharpen equipped weapon",
        "Reinforce equipped armor",
        "Brace equipped shield",
    ];
    loop {
        clamp_selection(&mut selected, options.len());
        clear_screen();
        println!("{BOLD}{WHITE}Blacksmith{RESET} - {}", gold_text(c.gold));
        println!(
            "{BOLD}Shards:{RESET} {}  {}  {}",
            shard_text("weapon", c.weapon_shards),
            shard_text("armor", c.armor_shards),
            shard_text("shield", c.shield_shards)
        );
        println!(
            "No durability or repairs. Salvage gear into type shards, then spend shards + gold to upgrade equipped gear."
        );
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();
        println!("{BOLD}Services{RESET}");
        for (i, option) in options.iter().enumerate() {
            let marker = if i == selected {
                format!("{GREEN}>{RESET}")
            } else {
                " ".to_string()
            };
            println!("{marker} {option}");
        }
        println!();
        println!("{BOLD}Equipped{RESET}");
        println!("Weapon: {}", item_summary(&c.equipped_weapon));
        println!("Armor : {}", item_summary(&c.equipped_armor));
        println!("Shield: {}", item_summary(&c.equipped_shield));
        print_footer(&[&format!(
            "{BOLD}Blacksmith:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=choose  {RED}Esc{RESET}=back"
        )]);
        match read_key_char_nav() {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < options.len() {
                    selected += 1;
                }
            }
            '\n' => match selected {
                0 => message = buy_item_message(c, crude_axe()),
                1 => message = buy_item_message(c, battered_mail()),
                2 => message = buy_item_message(c, worn_shield()),
                3 => salvage_screen(c),
                4 => message = upgrade_equipped_message(c, UpgradeSlot::Weapon),
                5 => message = upgrade_equipped_message(c, UpgradeSlot::Armor),
                6 => message = upgrade_equipped_message(c, UpgradeSlot::Shield),
                _ => {}
            },
            _ => message = "Unknown blacksmith command.".to_string(),
        }
    }
}

fn buy_item_message(c: &mut Character, item: Item) -> String {
    if c.gold < item.value {
        return "Not enough gold.".to_string();
    }
    c.gold -= item.value;
    let message = format!("Bought {}.", item.name);
    c.inventory.push(item);
    message
}

#[derive(Clone, Copy)]
enum UpgradeSlot {
    Weapon,
    Armor,
    Shield,
}

fn salvage_screen(c: &mut Character) {
    let mut selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_selection(&mut selected, c.inventory.len());
        clear_screen();
        println!("{BOLD}{WHITE}Salvage Gear{RESET}");
        println!(
            "{BOLD}Shards:{RESET} {}  {}  {}",
            shard_text("weapon", c.weapon_shards),
            shard_text("armor", c.armor_shards),
            shard_text("shield", c.shield_shards)
        );
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        if c.inventory.is_empty() {
            println!("Inventory is empty.");
        } else {
            print_inventory_list(c, selected, inventory_visible_rows(8));
            println!();
            println!("Selected: {}", item_summary(&c.inventory[selected]));
            if let Some(kind) = shard_kind(&c.inventory[selected]) {
                println!(
                    "Salvage yield: {} {} shard(s)",
                    salvage_shard_yield(&c.inventory[selected]),
                    shard_name(kind)
                );
            } else {
                println!("Consumables cannot be salvaged.");
            }
        }
        print_footer(&[&format!(
            "{BOLD}Salvage:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=salvage  {RED}Esc{RESET}=back"
        )]);
        match read_key_char_nav() {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < c.inventory.len() {
                    selected += 1;
                }
            }
            '\n' => message = salvage_inventory_item(c, selected),
            _ => message = "Unknown salvage command.".to_string(),
        }
    }
}

fn salvage_inventory_item(c: &mut Character, index: usize) -> String {
    if index >= c.inventory.len() {
        return "No item selected.".to_string();
    }
    let Some(kind) = shard_kind(&c.inventory[index]) else {
        return "Only weapons, armor, and shields can be salvaged.".to_string();
    };
    let item = c.inventory.remove(index);
    let amount = salvage_shard_yield(&item);
    add_shards(c, kind, amount);
    format!(
        "Salvaged {} into {} {} shard(s).",
        item.name,
        amount,
        shard_name(kind)
    )
}

fn salvage_shard_yield(item: &Item) -> u32 {
    let rarity_bonus = match item.rarity {
        Rarity::Common => 1,
        Rarity::Magic => 2,
        Rarity::Rare => 3,
    };
    rarity_bonus + item.upgrade_level
}

fn upgrade_equipped_message(c: &mut Character, slot: UpgradeSlot) -> String {
    let (cost_shards, cost_gold, kind, item_name) = {
        let item = equipped_item(c, slot);
        let kind = shard_kind(item).expect("equipped gear has shard kind");
        let (cost_shards, cost_gold) = upgrade_cost(item);
        (cost_shards, cost_gold, kind, item.name.clone())
    };
    if shard_count(c, kind) < cost_shards {
        return format!(
            "Need {} {} shard(s) to upgrade {}.",
            cost_shards,
            shard_name(kind),
            item_name
        );
    }
    if c.gold < cost_gold {
        return format!("Need {cost_gold} gold to upgrade {item_name}.");
    }
    spend_shards(c, kind, cost_shards);
    c.gold -= cost_gold;
    let item = equipped_item_mut(c, slot);
    upgrade_item(item);
    format!("Upgraded {} to +{}.", item.name, item.upgrade_level)
}

fn upgrade_cost(item: &Item) -> (u32, u32) {
    let next = item.upgrade_level + 1;
    (next * 2, next * 25)
}

fn upgrade_item(item: &mut Item) {
    item.upgrade_level += 1;
    item.value += 10 * item.upgrade_level;
    match item.kind {
        ItemKind::Weapon => {
            item.damage_min += 1;
            item.damage_max += 1;
        }
        ItemKind::Armor => item.armor += 1,
        ItemKind::Shield => item.armor += 1,
        ItemKind::HealthPotion | ItemKind::ManaPotion => {}
    }
}

fn equipped_item(c: &Character, slot: UpgradeSlot) -> &Item {
    match slot {
        UpgradeSlot::Weapon => &c.equipped_weapon,
        UpgradeSlot::Armor => &c.equipped_armor,
        UpgradeSlot::Shield => &c.equipped_shield,
    }
}

fn equipped_item_mut(c: &mut Character, slot: UpgradeSlot) -> &mut Item {
    match slot {
        UpgradeSlot::Weapon => &mut c.equipped_weapon,
        UpgradeSlot::Armor => &mut c.equipped_armor,
        UpgradeSlot::Shield => &mut c.equipped_shield,
    }
}

fn shard_kind(item: &Item) -> Option<ItemKind> {
    match item.kind {
        ItemKind::Weapon => Some(ItemKind::Weapon),
        ItemKind::Armor => Some(ItemKind::Armor),
        ItemKind::Shield => Some(ItemKind::Shield),
        ItemKind::HealthPotion | ItemKind::ManaPotion => None,
    }
}

fn shard_name(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Weapon => "weapon",
        ItemKind::Armor => "armor",
        ItemKind::Shield => "shield",
        ItemKind::HealthPotion | ItemKind::ManaPotion => "unknown",
    }
}

fn shard_count(c: &Character, kind: ItemKind) -> u32 {
    match kind {
        ItemKind::Weapon => c.weapon_shards,
        ItemKind::Armor => c.armor_shards,
        ItemKind::Shield => c.shield_shards,
        ItemKind::HealthPotion | ItemKind::ManaPotion => 0,
    }
}

fn add_shards(c: &mut Character, kind: ItemKind, amount: u32) {
    match kind {
        ItemKind::Weapon => c.weapon_shards += amount,
        ItemKind::Armor => c.armor_shards += amount,
        ItemKind::Shield => c.shield_shards += amount,
        ItemKind::HealthPotion | ItemKind::ManaPotion => {}
    }
}

fn spend_shards(c: &mut Character, kind: ItemKind, amount: u32) {
    match kind {
        ItemKind::Weapon => c.weapon_shards = c.weapon_shards.saturating_sub(amount),
        ItemKind::Armor => c.armor_shards = c.armor_shards.saturating_sub(amount),
        ItemKind::Shield => c.shield_shards = c.shield_shards.saturating_sub(amount),
        ItemKind::HealthPotion | ItemKind::ManaPotion => {}
    }
}

fn sell_item_screen(c: &mut Character) {
    let mut selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_selection(&mut selected, c.inventory.len());
        clear_screen();
        println!("{BOLD}{YELLOW}Sell Items{RESET} - {}", gold_text(c.gold));
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        if c.inventory.is_empty() {
            println!("Inventory is empty.");
        } else {
            print_inventory_list(c, selected, inventory_visible_rows(8));
            let item = &c.inventory[selected];
            println!();
            println!("Selected: {}", item_summary(item));
            println!("Sell value: {YELLOW}{} gold{RESET}", item.value / 4);
        }
        print_footer(&[&format!(
            "{BOLD}Sell:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=sell  {RED}Esc{RESET}=back"
        )]);
        match read_key_char_nav() {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < c.inventory.len() {
                    selected += 1;
                }
            }
            '\n' => {
                if c.inventory.is_empty() {
                    message = "Inventory is empty.".to_string();
                    continue;
                }
                let item = c.inventory.remove(selected);
                let sell_value = item.value / 4;
                c.gold += sell_value;
                message = format!("Sold {} for {} gold.", item.name, sell_value);
            }
            _ => pause("Unknown sell command."),
        }
    }
}

fn stash_menu(c: &mut Character) {
    let mut side = StashSide::Inventory;
    let mut inv_selected = 0usize;
    let mut stash_selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_selection(&mut inv_selected, c.inventory.len());
        clamp_selection(&mut stash_selected, c.stash.len());
        clear_screen();
        println!("{BOLD}{MAGENTA}Stash{RESET}");
        println!(
            "{CYAN}Inventory items: {}{RESET}   {MAGENTA}Stash items: {}{RESET}",
            c.inventory.len(),
            c.stash.len()
        );
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();
        let visible_rows = (inventory_visible_rows(12) / 2).max(4);
        print_stash_column(
            "Inventory",
            &c.inventory,
            inv_selected,
            side == StashSide::Inventory,
            visible_rows,
        );
        println!();
        print_stash_column(
            "Stash",
            &c.stash,
            stash_selected,
            side == StashSide::Stash,
            visible_rows,
        );
        print_footer(&[&format!(
            "{BOLD}Stash:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {CYAN}Tab{RESET}=switch list  {YELLOW}Enter{RESET}=move selected  {RED}Esc{RESET}=back"
        )]);
        match read_key_char_nav() {
            '\u{1b}' => break,
            '\t' => side = side.other(),
            'w' | 'W' => match side {
                StashSide::Inventory => inv_selected = inv_selected.saturating_sub(1),
                StashSide::Stash => stash_selected = stash_selected.saturating_sub(1),
            },
            's' | 'S' => match side {
                StashSide::Inventory => {
                    if inv_selected + 1 < c.inventory.len() {
                        inv_selected += 1;
                    }
                }
                StashSide::Stash => {
                    if stash_selected + 1 < c.stash.len() {
                        stash_selected += 1;
                    }
                }
            },
            '\n' => {
                message = match side {
                    StashSide::Inventory => {
                        move_selected(&mut c.inventory, &mut c.stash, inv_selected, "Stored")
                    }
                    StashSide::Stash => {
                        move_selected(&mut c.stash, &mut c.inventory, stash_selected, "Retrieved")
                    }
                };
            }
            _ => pause("Unknown stash command."),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StashSide {
    Inventory,
    Stash,
}

impl StashSide {
    fn other(self) -> Self {
        match self {
            StashSide::Inventory => StashSide::Stash,
            StashSide::Stash => StashSide::Inventory,
        }
    }
}

fn move_selected(from: &mut Vec<Item>, to: &mut Vec<Item>, index: usize, verb: &str) -> String {
    if from.is_empty() {
        "Nothing to move.".to_string()
    } else if index >= from.len() {
        "No item selected.".to_string()
    } else {
        let item = from.remove(index);
        let msg = format!("{} {}.", verb, item.name);
        to.push(item);
        msg
    }
}

fn spend_attributes(c: &mut Character) {
    while c.unspent_attributes > 0 {
        clear_screen();
        println!(
            "{BOLD}{CYAN}Spend attributes{RESET} ({CYAN}{} left{RESET})",
            c.unspent_attributes
        );
        println!(
            "{GREEN}1){RESET} {RED}Strength {} -> {}{RESET} ({RED}+5 max HP{RESET})",
            c.strength,
            c.strength + 1
        );
        println!(
            "{GREEN}2){RESET} {GREEN}Dexterity {} -> {}{RESET} ({CYAN}+5 hit{RESET}, {YELLOW}+5 speed{RESET})",
            c.dexterity,
            c.dexterity + 1
        );
        println!(
            "{GREEN}3){RESET} {BLUE}Intelligence {} -> {}{RESET} ({BLUE}+5 max mana{RESET})",
            c.intelligence,
            c.intelligence + 1
        );
        print_footer(&[&format!(
            "{BOLD}Attributes:{RESET} {GREEN}1{RESET}={RED}Strength{RESET}  {GREEN}2{RESET}={GREEN}Dexterity{RESET}  {GREEN}3{RESET}={BLUE}Intelligence{RESET}  {RED}Esc{RESET}=back"
        )]);
        match read_key_char() {
            '1' => {
                c.strength += 1;
                c.unspent_attributes -= 1;
                c.hp += 5;
            }
            '2' => {
                c.dexterity += 1;
                c.unspent_attributes -= 1;
            }
            '3' => {
                c.intelligence += 1;
                c.unspent_attributes -= 1;
                c.mana += 5;
            }
            '\u{1b}' => break,
            _ => pause("Unknown action."),
        }
    }
    if c.unspent_attributes == 0 {
        pause("No unspent attribute points.");
    }
}

fn skill_tree_menu(c: &mut Character) {
    loop {
        clear_screen();
        println!("{BOLD}{CYAN}Ironbound Skill Tree{RESET}");
        println!("{}", unspent_skills_text(c.unspent_skills));
        println!();
        println!("{BOLD}Weapons Branch{RESET}");
        print_skill_upgrade_preview(
            '1',
            "Cleave",
            c.cleave_rank,
            "cost 5 mana, cd 1, hits up to 3 adjacent enemies",
            cleave_percent_for_rank(c.cleave_rank),
            cleave_percent_for_rank(next_skill_rank(c.cleave_rank)),
            "% weapon damage",
        );
        print_skill_upgrade_preview(
            '4',
            "Deep Cut",
            c.deep_cut_rank,
            "passive melee bleed chance and damage; requires Cleave rank 2 for upgrades",
            deep_cut_chance_for_rank(c.deep_cut_rank),
            deep_cut_chance_for_rank(next_skill_rank(c.deep_cut_rank)),
            "% bleed chance",
        );
        println!(
            "   Bleed damage: {} now, {} next.",
            deep_cut_damage_for_rank(c.deep_cut_rank),
            deep_cut_damage_for_rank(next_skill_rank(c.deep_cut_rank))
        );
        println!("{BOLD}Defense Branch{RESET}");
        print_skill_upgrade_preview(
            '2',
            "Shield Bash",
            c.shield_bash_rank,
            "cost 6 mana, cd 3, hits 1 enemy and staggers",
            shield_bash_percent_for_rank(c.shield_bash_rank),
            shield_bash_percent_for_rank(next_skill_rank(c.shield_bash_rank)),
            "% weapon damage",
        );
        print_skill_upgrade_preview(
            '5',
            "Iron Guard",
            c.iron_guard_rank,
            "passive armor while using a shield; requires Shield Bash rank 2 for upgrades",
            iron_guard_armor_bonus_for_rank(c.iron_guard_rank) as u32,
            iron_guard_armor_bonus_for_rank(next_skill_rank(c.iron_guard_rank)) as u32,
            " armor",
        );
        println!("{BOLD}Warcry Branch{RESET}");
        print_skill_upgrade_preview(
            '3',
            "Battle Cry",
            c.battle_cry_rank,
            "cost 8 mana, cd 6, lasts 5 turns",
            battle_cry_bonus_percent_for_rank(c.battle_cry_rank),
            battle_cry_bonus_percent_for_rank(next_skill_rank(c.battle_cry_rank)),
            "% bonus damage",
        );
        print_skill_upgrade_preview(
            '6',
            "Second Wind",
            c.second_wind_rank,
            "passive heal on kill while Battle Cry is active; requires Battle Cry rank 2 for upgrades",
            second_wind_heal_percent_for_rank(c.second_wind_rank),
            second_wind_heal_percent_for_rank(next_skill_rank(c.second_wind_rank)),
            "% max HP heal",
        );
        println!();
        println!(
            "Each upgrade costs 1 skill point. Max rank is 5. Passive upgrades require rank 2 in their branch starter."
        );
        print_footer(&[&format!(
            "{BOLD}Skill Tree:{RESET} {GREEN}1{RESET}=Cleave {GREEN}2{RESET}=Bash {GREEN}3{RESET}=Cry {GREEN}4{RESET}=Deep Cut {GREEN}5{RESET}=Iron Guard {GREEN}6{RESET}=Second Wind {RED}Esc{RESET}=back"
        )]);
        match read_key_char() {
            '1' => upgrade_skill(c, "Cleave"),
            '2' => upgrade_skill(c, "Shield Bash"),
            '3' => upgrade_skill(c, "Battle Cry"),
            '4' => upgrade_skill(c, "Deep Cut"),
            '5' => upgrade_skill(c, "Iron Guard"),
            '6' => upgrade_skill(c, "Second Wind"),
            '\u{1b}' => break,
            _ => pause("Unknown action."),
        }
    }
}

fn print_skill_upgrade_preview(
    key: char,
    name: &str,
    rank: u32,
    details: &str,
    current_value: u32,
    next_value: u32,
    value_label: &str,
) {
    println!("{GREEN}{key}) {name}{RESET} rank {rank}/5");
    println!("   Current: {CYAN}{current_value}{value_label}{RESET}; {details}");
    if rank >= 5 {
        println!("   Next: {YELLOW}MAX RANK{RESET}");
    } else {
        println!(
            "   Next rank {}: {GREEN}{next_value}{value_label}{RESET}; {details}",
            rank + 1
        );
    }
}

fn upgrade_skill(c: &mut Character, skill: &str) {
    if c.unspent_skills == 0 {
        pause("No unspent skill points.");
        return;
    }
    if skill_rank(c, skill) >= 5 {
        pause("That skill is already at max rank.");
        return;
    }
    if let Some(requirement) = unmet_skill_prerequisite(c, skill) {
        pause(&requirement);
        return;
    }
    match skill {
        "Cleave" => c.cleave_rank += 1,
        "Shield Bash" => c.shield_bash_rank += 1,
        "Battle Cry" => c.battle_cry_rank += 1,
        "Deep Cut" => c.deep_cut_rank += 1,
        "Iron Guard" => c.iron_guard_rank += 1,
        "Second Wind" => c.second_wind_rank += 1,
        _ => return,
    }
    c.unspent_skills -= 1;
    // Do not pause here: immediately return to the skill tree loop so the
    // upgraded rank and next-rank preview redraw right away.
}

fn skill_rank(c: &Character, skill: &str) -> u32 {
    match skill {
        "Cleave" => c.cleave_rank,
        "Shield Bash" => c.shield_bash_rank,
        "Battle Cry" => c.battle_cry_rank,
        "Deep Cut" => c.deep_cut_rank,
        "Iron Guard" => c.iron_guard_rank,
        "Second Wind" => c.second_wind_rank,
        _ => 5,
    }
}

fn unmet_skill_prerequisite(c: &Character, skill: &str) -> Option<String> {
    match skill {
        "Deep Cut" if c.cleave_rank < 2 => {
            Some("Deep Cut upgrades require Cleave rank 2.".to_string())
        }
        "Iron Guard" if c.shield_bash_rank < 2 => {
            Some("Iron Guard upgrades require Shield Bash rank 2.".to_string())
        }
        "Second Wind" if c.battle_cry_rank < 2 => {
            Some("Second Wind upgrades require Battle Cry rank 2.".to_string())
        }
        _ => None,
    }
}

fn next_skill_rank(rank: u32) -> u32 {
    (rank + 1).min(5)
}
fn cleave_multiplier(c: &Character) -> f32 {
    cleave_multiplier_for_rank(c.cleave_rank)
}
fn cleave_multiplier_for_rank(rank: u32) -> f32 {
    0.8 + (rank.saturating_sub(1) as f32 * 0.10)
}
fn shield_bash_multiplier(c: &Character) -> f32 {
    shield_bash_multiplier_for_rank(c.shield_bash_rank)
}
fn shield_bash_multiplier_for_rank(rank: u32) -> f32 {
    0.7 + (rank.saturating_sub(1) as f32 * 0.10)
}
fn battle_cry_multiplier(c: &Character) -> f32 {
    battle_cry_multiplier_for_rank(c.battle_cry_rank)
}
fn battle_cry_multiplier_for_rank(rank: u32) -> f32 {
    1.20 + (rank.saturating_sub(1) as f32 * 0.05)
}
fn cleave_percent_for_rank(rank: u32) -> u32 {
    (cleave_multiplier_for_rank(rank) * 100.0).round() as u32
}
fn shield_bash_percent_for_rank(rank: u32) -> u32 {
    (shield_bash_multiplier_for_rank(rank) * 100.0).round() as u32
}
fn battle_cry_bonus_percent_for_rank(rank: u32) -> u32 {
    ((battle_cry_multiplier_for_rank(rank) - 1.0) * 100.0).round() as u32
}
fn cleave_percent(c: &Character) -> u32 {
    cleave_percent_for_rank(c.cleave_rank)
}
fn shield_bash_percent(c: &Character) -> u32 {
    shield_bash_percent_for_rank(c.shield_bash_rank)
}
fn battle_cry_bonus_percent(c: &Character) -> u32 {
    battle_cry_bonus_percent_for_rank(c.battle_cry_rank)
}
fn deep_cut_chance_for_rank(rank: u32) -> u32 {
    10 + rank.min(5) * 5
}
fn deep_cut_damage_for_rank(rank: u32) -> i32 {
    1 + rank.min(5).div_ceil(2) as i32
}
fn iron_guard_armor_bonus(c: &Character) -> i32 {
    iron_guard_armor_bonus_for_rank(c.iron_guard_rank)
}
fn iron_guard_armor_bonus_for_rank(rank: u32) -> i32 {
    1 + rank.min(5) as i32
}
fn second_wind_heal_percent_for_rank(rank: u32) -> u32 {
    5 + rank.min(5) * 5
}
fn second_wind_heal_amount(c: &Character) -> u32 {
    ((c.max_hp() * second_wind_heal_percent_for_rank(c.second_wind_rank)) / 100).max(1)
}

fn enter_dungeon(c: &mut Character) {
    if c.act1_completed {
        pause("Act II placeholder: The road to the Glass Wastes is visible, but not playable yet.");
        return;
    }
    if c.bellkeeper_defeated {
        pause("The Bellkeeper is dead. Return to Warden Mara (t) to complete Act I.");
        return;
    }
    c.active_dungeon = Some(generate_dungeon(1));
}

#[derive(Clone)]
struct Room {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Room {
    fn center(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    fn intersects(&self, other: &Room) -> bool {
        self.x <= other.x + other.w + 1
            && self.x + self.w + 1 >= other.x
            && self.y <= other.y + other.h + 1
            && self.y + self.h + 1 >= other.y
    }
}

fn generate_dungeon(floor: u32) -> Dungeon {
    let mut rng = rand::thread_rng();
    let mut tiles = vec!['#'; (MAP_W * MAP_H) as usize];
    let target_rooms = match floor {
        1 => rng.gen_range(6..=8),
        2 => rng.gen_range(8..=10),
        _ => rng.gen_range(5..=7),
    };
    let mut rooms: Vec<Room> = Vec::new();

    for _ in 0..120 {
        if rooms.len() >= target_rooms {
            break;
        }
        let room = Room {
            w: rng.gen_range(5..=11),
            h: rng.gen_range(4..=7),
            x: rng.gen_range(1..MAP_W - 12),
            y: rng.gen_range(1..MAP_H - 8),
        };
        if rooms.iter().any(|existing| room.intersects(existing)) {
            continue;
        }
        carve_room(&mut tiles, &room);
        if let Some(previous) = rooms.last() {
            carve_corridor(&mut tiles, previous.center(), room.center());
        }
        rooms.push(room);
    }

    if rooms.is_empty() {
        let fallback = Room {
            x: 2,
            y: 2,
            w: 10,
            h: 6,
        };
        carve_room(&mut tiles, &fallback);
        rooms.push(fallback);
    }

    let start = rooms.first().unwrap().center();
    let stairs = farthest_room_center(&rooms, start);
    let mut enemies = Vec::new();
    let enemy_count = match floor {
        1 => 5,
        2 => 7,
        _ => 4,
    };
    for _ in 0..enemy_count {
        let (x, y) = random_room_floor(&rooms, &mut rng, start, stairs);
        let e = match floor {
            1 => {
                if rng.gen_bool(0.55) {
                    rat(x, y)
                } else {
                    skeleton(x, y)
                }
            }
            2 => {
                if rng.gen_bool(0.45) {
                    skeleton(x, y)
                } else {
                    cultist(x, y)
                }
            }
            _ => {
                if rng.gen_bool(0.45) {
                    cultist(x, y)
                } else {
                    boneguard(x, y)
                }
            }
        };
        enemies.push(e);
    }
    if floor == 2 {
        let (x, y) = farthest_room_center(&rooms, start);
        enemies.push(elite_skeleton(x, y));
    }
    if floor == 3 {
        enemies.push(bellkeeper(stairs.0, stairs.1));
    }

    let chest_count = rng.gen_range(1..=3);
    let mut chests = Vec::new();
    for _ in 0..chest_count {
        let (x, y) = random_room_floor(&rooms, &mut rng, start, stairs);
        chests.push(Chest {
            x,
            y,
            opened: false,
        });
    }

    Dungeon {
        floor,
        player_x: start.0,
        player_y: start.1,
        stairs_x: stairs.0,
        stairs_y: stairs.1,
        enemies,
        chests,
        log: vec![format!("Entered Hollow Crypts floor {}.", floor)],
        tiles,
        bell_wave_tiles: Vec::new(),
        boss_turn_counter: 0,
        log_turn: 0,
    }
}

fn tile_index(x: i32, y: i32) -> usize {
    (y * MAP_W + x) as usize
}

fn carve_room(tiles: &mut [char], room: &Room) {
    for y in room.y..room.y + room.h {
        for x in room.x..room.x + room.w {
            tiles[tile_index(x, y)] = '.';
        }
    }
}

fn carve_corridor(tiles: &mut [char], from: (i32, i32), to: (i32, i32)) {
    let mut x = from.0;
    let mut y = from.1;
    while x != to.0 {
        tiles[tile_index(x, y)] = '.';
        x += (to.0 - x).signum();
    }
    while y != to.1 {
        tiles[tile_index(x, y)] = '.';
        y += (to.1 - y).signum();
    }
    tiles[tile_index(x, y)] = '.';
}

fn farthest_room_center(rooms: &[Room], from: (i32, i32)) -> (i32, i32) {
    rooms
        .iter()
        .map(Room::center)
        .max_by_key(|(x, y)| (x - from.0).abs() + (y - from.1).abs())
        .unwrap_or((MAP_W - 3, MAP_H - 3))
}

fn random_room_floor(
    rooms: &[Room],
    rng: &mut impl Rng,
    start: (i32, i32),
    stairs: (i32, i32),
) -> (i32, i32) {
    for _ in 0..30 {
        let room = &rooms[rng.gen_range(0..rooms.len())];
        let pos = (
            rng.gen_range(room.x..room.x + room.w),
            rng.gen_range(room.y..room.y + room.h),
        );
        if pos != start && pos != stairs {
            return pos;
        }
    }
    rooms.last().unwrap().center()
}

fn dungeon_tile(d: &Dungeon, x: i32, y: i32) -> char {
    if x < 0 || y < 0 || x >= MAP_W || y >= MAP_H {
        return '#';
    }
    if d.tiles.len() == (MAP_W * MAP_H) as usize {
        d.tiles[tile_index(x, y)]
    } else if x == 0 || y == 0 || x == MAP_W - 1 || y == MAP_H - 1 {
        '#'
    } else {
        '.'
    }
}

fn enemy(
    name: &str,
    glyph: char,
    x: i32,
    y: i32,
    hp: i32,
    damage_min: i32,
    damage_max: i32,
    armor: i32,
    speed: i32,
    xp: u32,
    gold_min: u32,
    gold_max: u32,
    is_boss: bool,
) -> Enemy {
    Enemy {
        name: name.to_string(),
        glyph,
        x,
        y,
        hp,
        max_hp: hp,
        damage_min,
        damage_max,
        armor,
        speed,
        xp,
        gold_min,
        gold_max,
        is_boss,
        stunned_turns: 0,
        bleed_turns: 0,
        bleed_damage: 0,
        guarding: false,
        elite_modifier: None,
    }
}

fn rat(x: i32, y: i32) -> Enemy {
    enemy("Rat", 'r', x, y, 6, 1, 2, 0, 11, 8, 0, 3, false)
}
fn skeleton(x: i32, y: i32) -> Enemy {
    enemy("Skeleton", 's', x, y, 12, 2, 4, 1, 9, 18, 2, 8, false)
}
fn cultist(x: i32, y: i32) -> Enemy {
    enemy("Cultist", 'c', x, y, 10, 2, 3, 0, 10, 22, 5, 12, false)
}
fn boneguard(x: i32, y: i32) -> Enemy {
    enemy("Boneguard", 'b', x, y, 18, 3, 5, 2, 8, 35, 8, 18, false)
}
fn elite_skeleton(x: i32, y: i32) -> Enemy {
    let modifier = random_elite_modifier();
    elite_skeleton_with_modifier(x, y, modifier)
}

fn elite_skeleton_with_modifier(x: i32, y: i32, modifier: EliteModifier) -> Enemy {
    let mut elite = enemy(
        "Elite Skeleton",
        'E',
        x,
        y,
        24,
        3,
        6,
        2,
        10,
        54,
        20,
        40,
        false,
    );
    apply_elite_modifier(&mut elite, modifier);
    elite
}

fn random_elite_modifier() -> EliteModifier {
    match rand::thread_rng().gen_range(0..4) {
        0 => EliteModifier::Armored,
        1 => EliteModifier::Swift,
        2 => EliteModifier::Vampiric,
        _ => EliteModifier::Burning,
    }
}

fn apply_elite_modifier(enemy: &mut Enemy, modifier: EliteModifier) {
    enemy.name = format!("{} {}", elite_modifier_name(&modifier), enemy.name);
    if matches!(modifier, EliteModifier::Swift) {
        enemy.speed += 2;
    }
    enemy.elite_modifier = Some(modifier);
}

fn elite_modifier_name(modifier: &EliteModifier) -> &'static str {
    match modifier {
        EliteModifier::Armored => "Armored",
        EliteModifier::Swift => "Swift",
        EliteModifier::Vampiric => "Vampiric",
        EliteModifier::Burning => "Burning",
    }
}
fn bellkeeper(x: i32, y: i32) -> Enemy {
    enemy("Bellkeeper", 'B', x, y, 60, 5, 8, 3, 8, 250, 100, 150, true)
}

fn dungeon_loop(c: &mut Character) -> Result<()> {
    loop {
        clear_screen();
        draw_dungeon(c);
        print_skill_help(c);
        print_dungeon_footer();
        let key = read_key_char();
        let before_log_len = current_dungeon_log_len(c);
        let action_label = dungeon_action_label(key);
        let mut took_turn = false;
        match key {
            'w' | 'W' => took_turn = try_move(c, 0, -1),
            's' | 'S' => took_turn = try_move(c, 0, 1),
            'a' | 'A' => took_turn = try_move(c, -1, 0),
            'd' | 'D' => took_turn = try_move(c, 1, 0),
            '1' => took_turn = use_cleave(c),
            '2' => took_turn = use_shield_bash(c),
            '3' => took_turn = use_battle_cry(c),
            'p' | 'P' => {
                use_potion(c);
                took_turn = true;
            }
            'i' | 'I' => inventory_screen(c),
            '\u{1b}' => {
                c.active_dungeon = None;
                save_character(c)?;
                break;
            }
            _ => pause("Unknown dungeon command."),
        }
        mark_latest_log_group(c, before_log_len, took_turn, action_label);
        if took_turn && c.active_dungeon.is_some() {
            tick_player_effects(c);
            enemy_turns(c);
            check_death(c);
            save_character(c)?;
        }
        if c.active_dungeon.is_none() {
            break;
        }
    }
    Ok(())
}

fn draw_dungeon(c: &Character) {
    let d = c.active_dungeon.as_ref().unwrap();
    println!(
        "{BOLD}Hollow Crypts Floor {}{RESET}  {} {} {} {}",
        d.floor,
        hp_text(c.hp, c.max_hp()),
        mana_text(c.mana, c.max_mana()),
        gold_text(c.gold),
        xp_text(c.xp)
    );
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let mut ch = dungeon_tile(d, x, y);
            if x == d.stairs_x && y == d.stairs_y {
                ch = '>';
            }
            if d.chests
                .iter()
                .any(|chest| chest.x == x && chest.y == y && !chest.opened)
            {
                ch = '$';
            }
            if d.bell_wave_tiles.contains(&(x, y)) {
                ch = '*';
            }
            if let Some(e) = d.enemies.iter().find(|e| e.x == x && e.y == y && e.hp > 0) {
                ch = e.glyph;
            }
            if x == d.player_x && y == d.player_y {
                ch = '@';
            }
            print_colored_tile(ch);
        }
        println!();
    }
    print_combat_log(d);
}

fn current_dungeon_log_len(c: &Character) -> usize {
    c.active_dungeon
        .as_ref()
        .map(|d| d.log.len())
        .unwrap_or_default()
}

fn dungeon_action_label(key: char) -> &'static str {
    match key {
        'w' | 'W' => "Move north / attack",
        's' | 'S' => "Move south / attack",
        'a' | 'A' => "Move west / attack",
        'd' | 'D' => "Move east / attack",
        '1' => "Cleave",
        '2' => "Shield Bash",
        '3' => "Battle Cry",
        'p' | 'P' => "Drink potion",
        _ => "Command",
    }
}

fn mark_latest_log_group(
    c: &mut Character,
    before_log_len: usize,
    took_turn: bool,
    action_label: &'static str,
) {
    let Some(d) = c.active_dungeon.as_mut() else {
        return;
    };
    if d.log.len() < before_log_len || (!took_turn && d.log.len() == before_log_len) {
        return;
    }
    let header = if took_turn {
        d.log_turn += 1;
        format!("== Turn {}: {} ==", d.log_turn, action_label)
    } else {
        format!("== No turn spent: {action_label} ==")
    };
    d.log.insert(before_log_len, header);
    trim_dungeon_log(&mut d.log);
}

fn trim_dungeon_log(log: &mut Vec<String>) {
    const MAX_LOG_LINES: usize = 120;
    if log.len() > MAX_LOG_LINES {
        let remove_count = log.len() - MAX_LOG_LINES;
        log.drain(0..remove_count);
    }
}

fn print_combat_log(d: &Dungeon) {
    const MAX_LOG_LINES_ON_SCREEN: usize = 8;
    println!("{BOLD}--- Combat Log: latest command ---{RESET}");

    let Some(latest_header) = d.log.iter().rposition(|line| is_log_header(line)) else {
        for msg in d.log.iter().rev().take(MAX_LOG_LINES_ON_SCREEN).rev() {
            print_log_line(msg, false);
        }
        return;
    };

    let current = &d.log[latest_header..];
    let current_start = current.len().saturating_sub(MAX_LOG_LINES_ON_SCREEN);
    for msg in &current[current_start..] {
        print_log_line(msg, true);
    }

    let printed_current = current.len().min(MAX_LOG_LINES_ON_SCREEN);
    let remaining = MAX_LOG_LINES_ON_SCREEN.saturating_sub(printed_current);
    if remaining > 1 && latest_header > 0 {
        println!("{DIM}--- Previous ---{RESET}");
        let previous_count = remaining - 1;
        let previous_start = latest_header.saturating_sub(previous_count);
        for msg in &d.log[previous_start..latest_header] {
            print_log_line(msg, false);
        }
    }
}

fn is_log_header(line: &str) -> bool {
    line.starts_with("== ") && line.ends_with(" ==")
}

fn print_log_line(line: &str, current_group: bool) {
    if is_log_header(line) {
        let color = if line.contains("No turn spent") {
            YELLOW
        } else {
            CYAN
        };
        println!("{BOLD}{color}{line}{RESET}");
    } else if current_group {
        println!("  {line}");
    } else {
        println!("{DIM}  {line}{RESET}");
    }
}

fn print_dungeon_footer() {
    print_footer(&[
        &format!(
            "{BOLD}Dungeon:{RESET} {GREEN}w/a/s/d{RESET}=move/attack  {GREEN}1{RESET}=Cleave  {GREEN}2{RESET}=Bash  {GREEN}3{RESET}=Cry  {BLUE}p{RESET}=potion  i=inventory  {RED}Esc{RESET}=town"
        ),
        &format!(
            "{BOLD}Legend:{RESET} {GREEN}@{RESET}=you {BRIGHT_BLACK}#{RESET}=wall {DIM}.{RESET}=floor {YELLOW}${RESET}=chest {MAGENTA}E{RESET}=elite {RED}B{RESET}=boss"
        ),
    ]);
}

fn print_skill_help(c: &Character) {
    print_above_footer(
        &[
            &format!(
                "{GREEN}1 Cleave r{}{RESET}: cost 5 mana, cd 1. Hit up to 3 enemies for {}% weapon damage. Ready in {}.",
                c.cleave_rank,
                cleave_percent(c),
                c.cleave_cooldown
            ),
            &format!(
                "{GREEN}2 Shield Bash r{}{RESET}: cost 6 mana, cd 3. Hit 1 enemy for {}% damage and stun 1 turn. Ready in {}.",
                c.shield_bash_rank,
                shield_bash_percent(c),
                c.shield_bash_cooldown
            ),
            &format!(
                "{GREEN}3 Battle Cry r{}{RESET}: cost 8 mana, cd 6. +{}% damage, -10% enemy damage, Second Wind r{} heals {}%. Ready in {}, active {}.",
                c.battle_cry_rank,
                battle_cry_bonus_percent(c),
                c.second_wind_rank,
                second_wind_heal_percent_for_rank(c.second_wind_rank),
                c.battle_cry_cooldown,
                c.battle_cry_turns
            ),
            &format!(
                "{GREEN}Passives:{RESET} Deep Cut r{} {}% bleed for {}/turn; Iron Guard r{} +{} armor.",
                c.deep_cut_rank,
                deep_cut_chance_for_rank(c.deep_cut_rank),
                deep_cut_damage_for_rank(c.deep_cut_rank),
                c.iron_guard_rank,
                iron_guard_armor_bonus(c)
            ),
        ],
        2,
    );
}

fn print_above_footer(lines: &[&str], footer_lines: u16) {
    let (_, height) = terminal_size().unwrap_or((80, 24));
    let start_row = height
        .saturating_sub(footer_lines)
        .saturating_sub(lines.len() as u16)
        .saturating_add(1)
        .max(1);
    for (i, line) in lines.iter().enumerate() {
        print!("\x1B[{};1H\x1B[2K{}", start_row + i as u16, line);
    }
    let _ = io::stdout().flush();
}

fn print_footer(lines: &[&str]) {
    let (_, height) = terminal_size().unwrap_or((80, 24));
    let start_row = height
        .saturating_sub(lines.len() as u16)
        .saturating_add(1)
        .max(1);
    for (i, line) in lines.iter().enumerate() {
        print!("\x1B[{};1H\x1B[2K{}", start_row + i as u16, line);
    }
    let _ = io::stdout().flush();
}

fn print_colored_tile(ch: char) {
    match ch {
        '@' => print!("{BOLD}{GREEN}@{RESET}"),
        '#' => print!("{BRIGHT_BLACK}#{RESET}"),
        '.' => print!("{DIM}.{RESET}"),
        '>' => print!("{BOLD}{CYAN}>{RESET}"),
        '$' => print!("{BOLD}{YELLOW}${RESET}"),
        '*' => print!("{BOLD}{MAGENTA}*{RESET}"),
        'r' => print!("{YELLOW}r{RESET}"),
        's' => print!("{WHITE}s{RESET}"),
        'c' => print!("{MAGENTA}c{RESET}"),
        'b' => print!("{BLUE}b{RESET}"),
        'E' => print!("{BOLD}{MAGENTA}E{RESET}"),
        'B' => print!("{BOLD}{RED}B{RESET}"),
        other => print!("{other}"),
    }
}

fn try_move(c: &mut Character, dx: i32, dy: i32) -> bool {
    let d = c.active_dungeon.as_mut().unwrap();
    let nx = d.player_x + dx;
    let ny = d.player_y + dy;
    if dungeon_tile(d, nx, ny) == '#' {
        d.log.push("You bump into a crypt wall.".to_string());
        return false;
    }
    if let Some(index) = d
        .enemies
        .iter()
        .position(|e| e.x == nx && e.y == ny && e.hp > 0)
    {
        player_attack(c, index);
        return true;
    }
    let d = c.active_dungeon.as_mut().unwrap();
    d.player_x = nx;
    d.player_y = ny;
    auto_interact_tile(c);
    true
}

fn player_attack(c: &mut Character, enemy_index: usize) {
    damage_enemy(c, enemy_index, 1.0, "hit");
}

fn use_cleave(c: &mut Character) -> bool {
    if c.cleave_cooldown > 0 {
        c.active_dungeon.as_mut().unwrap().log.push(format!(
            "Cleave is on cooldown for {} more turns.",
            c.cleave_cooldown
        ));
        return false;
    }
    if c.mana < 5 {
        c.active_dungeon
            .as_mut()
            .unwrap()
            .log
            .push("Not enough mana for Cleave.".to_string());
        return false;
    }
    let targets = adjacent_enemy_indices(c);
    if targets.is_empty() {
        c.active_dungeon
            .as_mut()
            .unwrap()
            .log
            .push("No adjacent enemies for Cleave.".to_string());
        return false;
    }
    c.mana -= 5;
    c.cleave_cooldown = 1;
    c.active_dungeon
        .as_mut()
        .unwrap()
        .log
        .push("You swing a wide Cleave!".to_string());
    for index in targets.into_iter().take(3).rev() {
        if c.active_dungeon.is_some() {
            damage_enemy(c, index, cleave_multiplier(c), "cleave");
        }
    }
    true
}

fn use_shield_bash(c: &mut Character) -> bool {
    if c.shield_bash_cooldown > 0 {
        c.active_dungeon.as_mut().unwrap().log.push(format!(
            "Shield Bash is on cooldown for {} more turns.",
            c.shield_bash_cooldown
        ));
        return false;
    }
    if c.mana < 6 {
        c.active_dungeon
            .as_mut()
            .unwrap()
            .log
            .push("Not enough mana for Shield Bash.".to_string());
        return false;
    }
    let Some(index) = adjacent_enemy_indices(c).first().copied() else {
        c.active_dungeon
            .as_mut()
            .unwrap()
            .log
            .push("No adjacent enemy for Shield Bash.".to_string());
        return false;
    };
    c.mana -= 6;
    c.shield_bash_cooldown = 3;
    damage_enemy(c, index, shield_bash_multiplier(c), "shield bash");
    if let Some(d) = c.active_dungeon.as_mut() {
        if let Some(enemy) = d.enemies.get_mut(index) {
            enemy.stunned_turns = enemy.stunned_turns.max(1);
        }
        d.log
            .push("The bash stuns the enemy for 1 turn.".to_string());
    }
    true
}

fn use_battle_cry(c: &mut Character) -> bool {
    if c.battle_cry_cooldown > 0 {
        c.active_dungeon.as_mut().unwrap().log.push(format!(
            "Battle Cry is on cooldown for {} more turns.",
            c.battle_cry_cooldown
        ));
        return false;
    }
    if c.mana < 8 {
        c.active_dungeon
            .as_mut()
            .unwrap()
            .log
            .push("Not enough mana for Battle Cry.".to_string());
        return false;
    }
    c.mana -= 8;
    c.battle_cry_turns = 5;
    c.battle_cry_cooldown = 6;
    c.active_dungeon
        .as_mut()
        .unwrap()
        .log
        .push("You roar a Battle Cry! Your damage rises and enemies falter.".to_string());
    true
}

fn adjacent_enemy_indices(c: &Character) -> Vec<usize> {
    let d = c.active_dungeon.as_ref().unwrap();
    d.enemies
        .iter()
        .enumerate()
        .filter(|(_, e)| e.hp > 0 && (e.x - d.player_x).abs() + (e.y - d.player_y).abs() == 1)
        .map(|(i, _)| i)
        .collect()
}

fn damage_enemy(c: &mut Character, enemy_index: usize, multiplier: f32, verb: &str) {
    let mut rng = rand::thread_rng();
    let (min, max) = c.weapon_damage();
    let damage_bonus = if c.battle_cry_turns > 0 {
        battle_cry_multiplier(c)
    } else {
        1.0
    };
    let hit = hit_roll(c.hit_rating() as i32, 10);
    let d = c.active_dungeon.as_mut().unwrap();
    if enemy_index >= d.enemies.len() || d.enemies[enemy_index].hp <= 0 {
        return;
    }
    let enemy = &mut d.enemies[enemy_index];
    if !hit {
        d.log.push(format!("You miss the {}.", enemy.name));
        return;
    }
    let raw = ((rng.gen_range(min..=max) as f32) * multiplier * damage_bonus).round() as i32;
    let armor = effective_enemy_armor(enemy);
    let damage = (raw - armor).max(1);
    enemy.hp -= damage;
    d.log.push(format!(
        "You {verb} {} for {}.",
        enemy.name,
        damage_text(damage)
    ));
    if enemy.guarding {
        d.log
            .push(format!("{} blocks with its shield.", enemy.name));
    }
    let bleed_chance = deep_cut_chance_for_rank(c.deep_cut_rank) as f64 / 100.0;
    if rng.gen_bool(bleed_chance) && enemy.hp > 0 {
        enemy.bleed_turns = 3;
        enemy.bleed_damage = deep_cut_damage_for_rank(c.deep_cut_rank);
        d.log.push(format!("{} starts bleeding.", enemy.name));
    }
    if enemy.hp <= 0 {
        let gold = rng.gen_range(enemy.gold_min..=enemy.gold_max);
        let xp = enemy.xp;
        let name = enemy.name.clone();
        let was_boss = enemy.is_boss;
        c.gold += gold;
        let levels_gained = add_xp(c, xp);
        let d = c.active_dungeon.as_mut().unwrap();
        d.log.push(format!(
            "{BOLD}{name} dies.{RESET} Gained {} and {}.",
            xp_reward_text(xp),
            gold_reward_text(gold)
        ));
        push_level_up_logs(&mut d.log, &levels_gained);
        if c.battle_cry_turns > 0 {
            let heal = second_wind_heal_amount(c);
            c.hp = (c.hp + heal).min(c.max_hp());
            let d = c.active_dungeon.as_mut().unwrap();
            d.log
                .push(format!("Second Wind restores {}.", heal_amount_text(heal)));
        }
        maybe_drop_loot(c, was_boss);
        if was_boss {
            c.bellkeeper_defeated = true;
            c.active_dungeon = None;
        }
    }
}

fn tick_player_effects(c: &mut Character) {
    c.cleave_cooldown = c.cleave_cooldown.saturating_sub(1);
    c.shield_bash_cooldown = c.shield_bash_cooldown.saturating_sub(1);
    c.battle_cry_cooldown = c.battle_cry_cooldown.saturating_sub(1);
    c.battle_cry_turns = c.battle_cry_turns.saturating_sub(1);
}

fn enemy_turns(c: &mut Character) {
    let Some(mut d) = c.active_dungeon.take() else {
        return;
    };
    d.bell_wave_tiles.clear();
    let mut occupied: Vec<(i32, i32)> = d
        .enemies
        .iter()
        .filter(|e| e.hp > 0)
        .map(|e| (e.x, e.y))
        .collect();
    for i in 0..d.enemies.len() {
        d.enemies[i].guarding = false;
        if d.enemies[i].hp <= 0 {
            continue;
        }
        if d.enemies[i].bleed_turns > 0 {
            d.enemies[i].hp -= d.enemies[i].bleed_damage;
            d.enemies[i].bleed_turns -= 1;
            d.log.push(format!(
                "{} bleeds for {}.",
                d.enemies[i].name,
                damage_text(d.enemies[i].bleed_damage)
            ));
            if d.enemies[i].hp <= 0 {
                let name = d.enemies[i].name.clone();
                let xp = d.enemies[i].xp;
                let was_boss = d.enemies[i].is_boss;
                let mut rng = rand::thread_rng();
                let gold = rng.gen_range(d.enemies[i].gold_min..=d.enemies[i].gold_max);
                c.gold += gold;
                let levels_gained = add_xp(c, xp);
                d.log.push(format!(
                    "{BOLD}{name} dies from bleeding.{RESET} Gained {} and {}.",
                    xp_reward_text(xp),
                    gold_reward_text(gold)
                ));
                push_level_up_logs(&mut d.log, &levels_gained);
                if was_boss {
                    let loot = random_loot(d.floor, true);
                    let loot_name = colored_item_name(&loot);
                    c.inventory.push(loot);
                    d.log.push(format!(
                        "{BOLD}{YELLOW}*** LOOT DROPPED: {loot_name} ***{RESET}"
                    ));
                    c.bellkeeper_defeated = true;
                    return;
                }
                continue;
            }
        }
        if d.enemies[i].stunned_turns > 0 {
            d.enemies[i].stunned_turns -= 1;
            d.log.push(format!(
                "{} is stunned and skips its turn.",
                d.enemies[i].name
            ));
            continue;
        }
        if d.enemies[i].is_boss {
            bellkeeper_specials(c, &mut d, i, &mut occupied);
        }
        let dist = (d.enemies[i].x - d.player_x).abs() + (d.enemies[i].y - d.player_y).abs();
        if dist == 1 {
            enemy_melee_attack(c, &mut d, i);
        } else if should_boneguard_guard(&d, i) {
            d.enemies[i].guarding = true;
            d.log
                .push(format!("{} raises its shield.", d.enemies[i].name));
        } else if can_cultist_ranged_attack(&d, i) {
            cultist_shadow_bolt(c, &mut d, i);
        } else if dist < 8 {
            let old = (d.enemies[i].x, d.enemies[i].y);
            let step_x = (d.player_x - d.enemies[i].x).signum();
            let step_y = (d.player_y - d.enemies[i].y).signum();
            let (nx, ny) =
                if (d.player_x - d.enemies[i].x).abs() > (d.player_y - d.enemies[i].y).abs() {
                    (d.enemies[i].x + step_x, d.enemies[i].y)
                } else {
                    (d.enemies[i].x, d.enemies[i].y + step_y)
                };
            if dungeon_tile(&d, nx, ny) != '#'
                && (nx, ny) != (d.player_x, d.player_y)
                && !occupied.contains(&(nx, ny))
            {
                d.enemies[i].x = nx;
                d.enemies[i].y = ny;
                if let Some(pos) = occupied.iter().position(|p| *p == old) {
                    occupied[pos] = (nx, ny);
                }
            }
        }
    }
    d.enemies.retain(|e| e.hp > 0);
    c.active_dungeon = Some(d);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BellkeeperPhase {
    Tolling,
    CursedBell,
    Enraged,
}

fn bellkeeper_phase(enemy: &Enemy) -> BellkeeperPhase {
    if enemy.hp * 4 <= enemy.max_hp {
        BellkeeperPhase::Enraged
    } else if enemy.hp * 10 <= enemy.max_hp * 6 {
        BellkeeperPhase::CursedBell
    } else {
        BellkeeperPhase::Tolling
    }
}

fn bellkeeper_specials(
    c: &mut Character,
    d: &mut Dungeon,
    boss_index: usize,
    occupied: &mut Vec<(i32, i32)>,
) {
    d.boss_turn_counter += 1;
    let phase = bellkeeper_phase(&d.enemies[boss_index]);
    if phase != BellkeeperPhase::Enraged && d.boss_turn_counter % 3 == 0 {
        summon_bellkeeper_skeleton(d, boss_index, occupied);
    }
    if phase != BellkeeperPhase::Tolling && d.boss_turn_counter % 4 == 0 {
        bellkeeper_wave(c, d, boss_index);
    }
    if phase == BellkeeperPhase::Enraged {
        d.log.push("The Bellkeeper is enraged!".to_string());
    }
}

fn summon_bellkeeper_skeleton(d: &mut Dungeon, boss_index: usize, occupied: &mut Vec<(i32, i32)>) {
    let summon_count = d
        .enemies
        .iter()
        .filter(|e| e.name == "Summoned Skeleton" && e.hp > 0)
        .count();
    if summon_count >= 3 {
        return;
    }
    let (boss_x, boss_y) = (d.enemies[boss_index].x, d.enemies[boss_index].y);
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let pos = (boss_x + dx, boss_y + dy);
        if dungeon_tile(d, pos.0, pos.1) != '#'
            && pos != (d.player_x, d.player_y)
            && !occupied.contains(&pos)
        {
            let mut summon = skeleton(pos.0, pos.1);
            summon.name = "Summoned Skeleton".to_string();
            d.enemies.push(summon);
            occupied.push(pos);
            d.log
                .push("The Bellkeeper tolls, and a skeleton claws free.".to_string());
            return;
        }
    }
}

fn bellkeeper_wave(c: &mut Character, d: &mut Dungeon, boss_index: usize) {
    let (boss_x, boss_y) = (d.enemies[boss_index].x, d.enemies[boss_index].y);
    d.bell_wave_tiles.clear();
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        for step in 1..=5 {
            let pos = (boss_x + dx * step, boss_y + dy * step);
            if dungeon_tile(d, pos.0, pos.1) == '#' {
                break;
            }
            d.bell_wave_tiles.push(pos);
        }
    }
    d.log.push(format!(
        "{MAGENTA}The Bellkeeper rings a cursed bell wave!{RESET}"
    ));
    if d.bell_wave_tiles.contains(&(d.player_x, d.player_y)) {
        let damage = enemy_damage_after_mitigation(6, c);
        c.hp = c.hp.saturating_sub(damage);
        d.log.push(format!(
            "The bell wave hits you for {}.",
            damage_text(damage)
        ));
    }
}

fn effective_enemy_armor(enemy: &Enemy) -> i32 {
    enemy.armor
        + if enemy.guarding { 2 } else { 0 }
        + if matches!(enemy.elite_modifier, Some(EliteModifier::Armored)) {
            2
        } else {
            0
        }
}

fn should_boneguard_guard(d: &Dungeon, enemy_index: usize) -> bool {
    let enemy = &d.enemies[enemy_index];
    if enemy.glyph != 'b' {
        return false;
    }
    let dist = (enemy.x - d.player_x).abs() + (enemy.y - d.player_y).abs();
    (2..=4).contains(&dist)
}

fn enemy_melee_attack(c: &mut Character, d: &mut Dungeon, enemy_index: usize) {
    let mut rng = rand::thread_rng();
    let enemy = &d.enemies[enemy_index];
    if hit_roll(25, c.dodge_rating() as i32) {
        let raw = rng.gen_range(enemy.damage_min..=enemy.damage_max)
            + elite_damage_bonus(enemy)
            + bellkeeper_enrage_damage_bonus(enemy);
        let damage = enemy_damage_after_mitigation(raw, c);
        c.hp = c.hp.saturating_sub(damage);
        d.log.push(format!(
            "{} hits you for {}.",
            enemy.name,
            damage_text(damage)
        ));
        apply_vampiric_heal(d, enemy_index);
    } else {
        d.log.push(format!("{} misses you.", enemy.name));
    }
}

fn can_cultist_ranged_attack(d: &Dungeon, enemy_index: usize) -> bool {
    let enemy = &d.enemies[enemy_index];
    if enemy.glyph != 'c' {
        return false;
    }
    let dx = (enemy.x - d.player_x).abs();
    let dy = (enemy.y - d.player_y).abs();
    let dist = dx + dy;
    (2..=5).contains(&dist)
        && (dx == 0 || dy == 0)
        && clear_cardinal_line(d, enemy.x, enemy.y, d.player_x, d.player_y)
}

fn clear_cardinal_line(d: &Dungeon, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> bool {
    if from_x != to_x && from_y != to_y {
        return false;
    }
    let step_x = (to_x - from_x).signum();
    let step_y = (to_y - from_y).signum();
    let mut x = from_x + step_x;
    let mut y = from_y + step_y;
    while (x, y) != (to_x, to_y) {
        if dungeon_tile(d, x, y) == '#' {
            return false;
        }
        x += step_x;
        y += step_y;
    }
    true
}

fn cultist_shadow_bolt(c: &mut Character, d: &mut Dungeon, enemy_index: usize) {
    let mut rng = rand::thread_rng();
    let enemy = &d.enemies[enemy_index];
    if hit_roll(30, c.dodge_rating() as i32) {
        let raw =
            rng.gen_range(enemy.damage_min..=enemy.damage_max + 1) + elite_damage_bonus(enemy);
        let damage = enemy_damage_after_mitigation(raw, c);
        c.hp = c.hp.saturating_sub(damage);
        d.log.push(format!(
            "{}'s shadow bolt hits you for {}.",
            enemy.name,
            damage_text(damage)
        ));
        apply_vampiric_heal(d, enemy_index);
    } else {
        d.log
            .push(format!("{}'s shadow bolt misses you.", enemy.name));
    }
}

fn bellkeeper_enrage_damage_bonus(enemy: &Enemy) -> i32 {
    if enemy.is_boss && bellkeeper_phase(enemy) == BellkeeperPhase::Enraged {
        2
    } else {
        0
    }
}

fn elite_damage_bonus(enemy: &Enemy) -> i32 {
    if matches!(enemy.elite_modifier, Some(EliteModifier::Burning)) {
        1
    } else {
        0
    }
}

fn apply_vampiric_heal(d: &mut Dungeon, enemy_index: usize) {
    if !matches!(
        d.enemies[enemy_index].elite_modifier,
        Some(EliteModifier::Vampiric)
    ) {
        return;
    }
    let enemy = &mut d.enemies[enemy_index];
    let before = enemy.hp;
    enemy.hp = (enemy.hp + 2).min(enemy.max_hp);
    let healed = enemy.hp - before;
    if healed > 0 {
        d.log.push(format!(
            "{} drains life and heals {}.",
            enemy.name,
            heal_amount_text(healed as u32)
        ));
    }
}

fn enemy_damage_after_mitigation(raw: i32, c: &Character) -> u32 {
    let cry_penalty = if c.battle_cry_turns > 0 { 0.90 } else { 1.0 };
    (((raw - c.armor()).max(1) as f32) * cry_penalty)
        .round()
        .max(1.0) as u32
}

fn hit_roll(hit: i32, dodge: i32) -> bool {
    let chance = (hit as f32 / (hit + dodge).max(1) as f32).clamp(0.20, 0.95);
    rand::thread_rng().gen_bool(chance as f64)
}

fn maybe_drop_loot(c: &mut Character, guaranteed_magic: bool) {
    let mut rng = rand::thread_rng();
    let drop_chance = if guaranteed_magic { 1.0 } else { 0.22 };
    if !rng.gen_bool(drop_chance) {
        return;
    }
    let floor = c.active_dungeon.as_ref().map(|d| d.floor).unwrap_or(1);
    let loot = random_loot(floor, guaranteed_magic || rng.gen_bool(0.30));
    let name = colored_item_name(&loot);
    c.inventory.push(loot);
    if let Some(d) = c.active_dungeon.as_mut() {
        d.log
            .push(format!("{BOLD}{YELLOW}*** LOOT DROPPED: {name} ***{RESET}"));
    }
}

fn random_loot(floor: u32, better: bool) -> Item {
    let mut rng = rand::thread_rng();
    let rarity = if better {
        if rng.gen_bool(0.25) {
            Rarity::Rare
        } else {
            Rarity::Magic
        }
    } else {
        Rarity::Common
    };
    let rarity_bonus = match rarity {
        Rarity::Common => 0,
        Rarity::Magic => 1,
        Rarity::Rare => 2,
    };
    let item_level = floor + rarity_bonus;
    let bonus = item_level as i32 - 1;
    match rng.gen_range(0..5) {
        0 => item_with_rarity(
            &loot_name(&rarity, "Iron Sword"),
            ItemKind::Weapon,
            45 + bonus as u32 * 15,
            3 + bonus,
            5 + bonus,
            0,
            0,
            0,
            rarity,
            item_level,
            4 + item_level,
            2 + item_level,
            0,
        ),
        1 => item_with_rarity(
            &loot_name(&rarity, "War Axe"),
            ItemKind::Weapon,
            60 + bonus as u32 * 15,
            4 + bonus,
            6 + bonus,
            0,
            0,
            -1,
            rarity,
            item_level,
            5 + item_level,
            0,
            0,
        ),
        2 => item_with_rarity(
            &loot_name(&rarity, "Mail Vest"),
            ItemKind::Armor,
            50 + bonus as u32 * 15,
            0,
            0,
            1 + bonus,
            0,
            -bonus.min(2),
            rarity,
            item_level,
            4 + item_level,
            0,
            0,
        ),
        3 => item_with_rarity(
            &loot_name(&rarity, "Guard Shield"),
            ItemKind::Shield,
            45 + bonus as u32 * 15,
            0,
            0,
            1 + bonus,
            2 + bonus,
            0,
            rarity,
            item_level,
            3 + item_level,
            0,
            0,
        ),
        _ => {
            if rng.gen_bool(0.5) {
                health_potion()
            } else {
                mana_potion()
            }
        }
    }
}

fn rarity_name(rarity: &Rarity) -> &'static str {
    match rarity {
        Rarity::Common => "Common",
        Rarity::Magic => "Magic",
        Rarity::Rare => "Rare",
    }
}

fn loot_name(rarity: &Rarity, base: &str) -> String {
    let mut rng = rand::thread_rng();
    match rarity {
        Rarity::Common => base.to_string(),
        Rarity::Magic => {
            let prefixes = ["Glinting", "Sturdy", "Sharp", "Vigorous", "Quick"];
            format!("{} {}", prefixes[rng.gen_range(0..prefixes.len())], base)
        }
        Rarity::Rare => {
            let prefixes = ["Bloodforged", "Dread", "Kingsguard", "Stormmarked", "Grim"];
            let suffixes = ["of Might", "of the Wolf", "of Ash", "of Warding", "of Doom"];
            format!(
                "{} {} {}",
                prefixes[rng.gen_range(0..prefixes.len())],
                base,
                suffixes[rng.gen_range(0..suffixes.len())]
            )
        }
    }
}

fn add_xp(c: &mut Character, amount: u32) -> Vec<u32> {
    let mut levels_gained = Vec::new();
    c.xp += amount;
    loop {
        let needed = 10 * 2u32.pow(c.level - 1);
        if c.xp < needed {
            break;
        }
        c.xp -= needed;
        c.level += 1;
        levels_gained.push(c.level);
        c.unspent_attributes += 3;
        c.unspent_skills += 1;
        c.hp = c.max_hp();
        c.mana = c.max_mana();
    }
    levels_gained
}

fn auto_interact_tile(c: &mut Character) {
    open_chest_on_player(c);
    let standing_on_stairs = c
        .active_dungeon
        .as_ref()
        .map(|d| d.player_x == d.stairs_x && d.player_y == d.stairs_y)
        .unwrap_or(false);
    if standing_on_stairs {
        use_stairs(c);
    }
}

fn open_chest_on_player(c: &mut Character) {
    let d = c.active_dungeon.as_mut().unwrap();
    if let Some(chest) = d
        .chests
        .iter_mut()
        .find(|ch| !ch.opened && ch.x == d.player_x && ch.y == d.player_y)
    {
        chest.opened = true;
        let mut rng = rand::thread_rng();
        let gold = rng.gen_range(10..=25);
        c.gold += gold;
        let loot = random_loot(1, rng.gen_bool(0.35));
        let name = colored_item_name(&loot);
        d.log.push(format!(
            "Opened chest: found {} and {name}.",
            gold_reward_text(gold)
        ));
        c.inventory.push(loot);
    }
}

fn use_stairs(c: &mut Character) {
    let floor;
    {
        let d = c.active_dungeon.as_ref().unwrap();
        if d.player_x != d.stairs_x || d.player_y != d.stairs_y {
            let d = c.active_dungeon.as_mut().unwrap();
            d.log.push("You are not standing on stairs.".to_string());
            return;
        }
        floor = d.floor;
    }
    if floor >= 3 {
        let d = c.active_dungeon.as_mut().unwrap();
        d.log
            .push("The Bellkeeper blocks your escape. Defeat it!".to_string());
    } else {
        c.active_dungeon = Some(generate_dungeon(floor + 1));
    }
}

fn use_potion(c: &mut Character) {
    if let Some(index) = c
        .inventory
        .iter()
        .position(|i| matches!(i.kind, ItemKind::HealthPotion))
    {
        c.inventory.remove(index);
        let heal = (c.max_hp() / 4).max(1);
        c.hp = (c.hp + heal).min(c.max_hp());
        c.active_dungeon.as_mut().unwrap().log.push(format!(
            "You drink a health potion and restore {}.",
            heal_amount_text(heal)
        ));
    } else {
        c.active_dungeon
            .as_mut()
            .unwrap()
            .log
            .push("No health potion available.".to_string());
    }
}

fn check_death(c: &mut Character) {
    if c.hp > 0 {
        return;
    }
    match c.death_mode {
        DeathMode::Softcore => {
            c.hp = c.max_hp();
            c.mana = c.max_mana();
            c.gold = c.gold.saturating_sub(c.gold / 10);
            c.active_dungeon = None;
        }
        DeathMode::Hardcore => {
            let _ = fs::remove_file(SAVE_PATH);
            println!("You died in Hardcore mode. Save deleted.");
            std::process::exit(0);
        }
    }
}

fn inventory_screen(c: &mut Character) {
    let mut selected = 0usize;
    loop {
        clamp_selection(&mut selected, c.inventory.len());
        clear_screen();
        println!("{BOLD}{CYAN}Equipment{RESET}");
        println!("Weapon: {}", item_summary(&c.equipped_weapon));
        println!("Armor : {}", item_summary(&c.equipped_armor));
        println!("Shield: {}", item_summary(&c.equipped_shield));
        println!(
            "{}  {}  {}",
            armor_text(c.armor()),
            dodge_text(c.dodge_rating()),
            speed_text(c.speed())
        );
        println!();
        println!("{BOLD}Inventory{RESET}");
        if c.inventory.is_empty() {
            println!("  Empty");
        } else {
            print_inventory_list(c, selected, inventory_visible_rows(10));
            println!();
            println!("Selected: {}", item_summary(&c.inventory[selected]));
            if let Some(compare) = item_comparison(c, &c.inventory[selected]) {
                println!("{compare}");
            }
        }
        print_footer(&[&format!(
            "{BOLD}Inventory:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=equip/use  {RED}x{RESET}=drop selected  {RED}Esc{RESET}=back"
        )]);
        match read_key_char_nav() {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < c.inventory.len() {
                    selected += 1;
                }
            }
            'x' | 'X' => drop_selected_inventory_item(c, selected),
            '\n' => equip_or_use_inventory_item(c, selected),
            _ => pause("Unknown inventory command."),
        }
    }
}

fn print_inventory_preview(c: &Character, max_rows: usize) {
    println!(
        "{BOLD}Your Inventory{RESET} ({CYAN}{}{RESET})",
        c.inventory.len()
    );
    if c.inventory.is_empty() {
        println!("  Empty");
        return;
    }
    let max_rows = max_rows.max(1);
    for item in c.inventory.iter().take(max_rows) {
        println!("  {}", item_summary(item));
    }
    if c.inventory.len() > max_rows {
        println!(
            "  ...{} more. Open sell to browse all.",
            c.inventory.len() - max_rows
        );
    }
}

fn print_inventory_list(c: &Character, selected: usize, max_rows: usize) {
    let total = c.inventory.len();
    let max_rows = max_rows.max(1);
    let offset = scroll_offset(selected, total, max_rows);
    let end = (offset + max_rows).min(total);
    if total > max_rows {
        println!(
            "{DIM}Showing items {}-{} of {}{RESET}",
            offset + 1,
            end,
            total
        );
    }
    for (i, item) in c.inventory.iter().enumerate().skip(offset).take(max_rows) {
        let marker = if i == selected {
            format!("{GREEN}>{RESET}")
        } else {
            " ".to_string()
        };
        println!("{marker} {}", item_summary(item));
        if i == selected {
            if let Some(compare) = item_comparison(c, item) {
                println!("  {compare}");
            }
        }
    }
}

fn print_stash_column(title: &str, items: &[Item], selected: usize, active: bool, max_rows: usize) {
    let heading = if active {
        format!("{BOLD}{GREEN}>{RESET} {BOLD}{title}{RESET}")
    } else {
        format!("  {BOLD}{title}{RESET}")
    };
    println!("{heading}");
    if items.is_empty() {
        println!("  Empty");
        return;
    }
    let max_rows = max_rows.max(1);
    let offset = scroll_offset(selected, items.len(), max_rows);
    let end = (offset + max_rows).min(items.len());
    if items.len() > max_rows {
        println!(
            "  {DIM}Showing items {}-{} of {}{RESET}",
            offset + 1,
            end,
            items.len()
        );
    }
    for (i, item) in items.iter().enumerate().skip(offset).take(max_rows) {
        let marker = if active && i == selected {
            format!("{GREEN}>{RESET}")
        } else {
            " ".to_string()
        };
        println!("{marker} {}", item_summary(item));
    }
}

fn inventory_visible_rows(reserved_rows: u16) -> usize {
    let (_, height) = terminal_size().unwrap_or((80, 24));
    height.saturating_sub(reserved_rows).max(5) as usize
}

fn scroll_offset(selected: usize, total: usize, max_rows: usize) -> usize {
    if total <= max_rows || selected < max_rows {
        0
    } else {
        selected + 1 - max_rows
    }
}

fn clamp_selection(selected: &mut usize, total: usize) {
    if total == 0 {
        *selected = 0;
    } else if *selected >= total {
        *selected = total - 1;
    }
}

fn drop_selected_inventory_item(c: &mut Character, index: usize) {
    if c.inventory.is_empty() {
        pause("Inventory is empty.");
    } else if index >= c.inventory.len() {
        pause("No item selected.");
    } else {
        let item = c.inventory.remove(index);
        pause(&format!("Dropped {}.", item.name));
    }
}

fn item_level_text(item: &Item) -> String {
    if matches!(
        item.kind,
        ItemKind::Weapon | ItemKind::Armor | ItemKind::Shield
    ) {
        format!("{CYAN}ilvl {}{RESET}", item.item_level)
    } else {
        String::new()
    }
}

fn item_requirements_text(item: &Item) -> String {
    let mut reqs = Vec::new();
    if item.required_strength > 0 {
        reqs.push(format!("{RED}STR {}{RESET}", item.required_strength));
    }
    if item.required_dexterity > 0 {
        reqs.push(format!("{GREEN}DEX {}{RESET}", item.required_dexterity));
    }
    if item.required_intelligence > 0 {
        reqs.push(format!("{BLUE}INT {}{RESET}", item.required_intelligence));
    }
    if reqs.is_empty() {
        String::new()
    } else {
        format!("req {}", reqs.join("/"))
    }
}

fn item_level_and_requirements(item: &Item) -> String {
    let item_level = item_level_text(item);
    let requirements = item_requirements_text(item);
    match (item_level.is_empty(), requirements.is_empty()) {
        (true, true) => String::new(),
        (false, true) => item_level,
        (true, false) => requirements,
        (false, false) => format!("{item_level} {requirements}"),
    }
}

fn item_summary(item: &Item) -> String {
    let rarity = rarity_name(&item.rarity);
    let name = colored_item_name(item);
    let upgrade = if item.upgrade_level > 0 {
        format!(" +{}", item.upgrade_level)
    } else {
        String::new()
    };
    let level_and_requirements = item_level_and_requirements(item);
    match item.kind {
        ItemKind::Weapon => format!(
            "{}{} [{} {:?}] {} {RED}dmg {}-{}{RESET} {YELLOW}value {}{RESET}",
            name,
            upgrade,
            rarity,
            item.kind,
            level_and_requirements,
            item.damage_min,
            item.damage_max,
            item.value
        ),
        ItemKind::Armor | ItemKind::Shield => format!(
            "{}{} [{} {:?}] {} {WHITE}armor {}{RESET} {GREEN}dodge {}{RESET} {YELLOW}speed {}{RESET} {YELLOW}value {}{RESET}",
            name,
            upgrade,
            rarity,
            item.kind,
            level_and_requirements,
            item.armor,
            item.dodge,
            item.speed,
            item.value
        ),
        _ => format!(
            "{} [{:?}] {YELLOW}value {}{RESET}",
            name, item.kind, item.value
        ),
    }
}

fn colored_item_name(item: &Item) -> String {
    let color = match item.rarity {
        Rarity::Common => WHITE,
        Rarity::Magic => BLUE,
        Rarity::Rare => YELLOW,
    };
    format!("{color}{}{RESET}", item.name)
}

fn item_comparison(c: &Character, item: &Item) -> Option<String> {
    let comparison = match item.kind {
        ItemKind::Weapon => {
            let cur_avg = c.equipped_weapon.damage_min + c.equipped_weapon.damage_max;
            let new_avg = item.damage_min + item.damage_max;
            format_delta("damage", new_avg - cur_avg)
        }
        ItemKind::Armor => format!(
            "Compare: {}  {}  {}",
            format_delta("armor", item.armor - c.equipped_armor.armor),
            format_delta("dodge", item.dodge - c.equipped_armor.dodge),
            format_delta("speed", item.speed - c.equipped_armor.speed)
        ),
        ItemKind::Shield => format!(
            "Compare: {}  {}  {}",
            format_delta("armor", item.armor - c.equipped_shield.armor),
            format_delta("dodge", item.dodge - c.equipped_shield.dodge),
            format_delta("speed", item.speed - c.equipped_shield.speed)
        ),
        _ => return None,
    };
    if let Some(requirements) = unmet_requirements_message(c, item) {
        Some(format!("{comparison}  {RED}LOCKED:{RESET} {requirements}"))
    } else {
        Some(comparison)
    }
}

fn format_delta(label: &str, delta: i32) -> String {
    if delta > 0 {
        format!("{GREEN}+{delta} {label}{RESET}")
    } else if delta < 0 {
        format!("{RED}{delta} {label}{RESET}")
    } else {
        format!("+0 {label}")
    }
}

fn can_equip_item(c: &Character, item: &Item) -> bool {
    c.strength >= item.required_strength
        && c.dexterity >= item.required_dexterity
        && c.intelligence >= item.required_intelligence
}

fn unmet_requirements_message(c: &Character, item: &Item) -> Option<String> {
    if can_equip_item(c, item) {
        return None;
    }
    let mut missing = Vec::new();
    if c.strength < item.required_strength {
        missing.push(format!(
            "{RED}STR {}/{}{RESET}",
            c.strength, item.required_strength
        ));
    }
    if c.dexterity < item.required_dexterity {
        missing.push(format!(
            "{GREEN}DEX {}/{}{RESET}",
            c.dexterity, item.required_dexterity
        ));
    }
    if c.intelligence < item.required_intelligence {
        missing.push(format!(
            "{BLUE}INT {}/{}{RESET}",
            c.intelligence, item.required_intelligence
        ));
    }
    Some(format!("Requires {}.", missing.join(", ")))
}

fn equip_or_use_inventory_item(c: &mut Character, index: usize) {
    if index >= c.inventory.len() {
        pause("No item in that slot.");
        return;
    }
    let selected = c.inventory.remove(index);
    if matches!(
        selected.kind,
        ItemKind::Weapon | ItemKind::Armor | ItemKind::Shield
    ) {
        if let Some(message) = unmet_requirements_message(c, &selected) {
            c.inventory.insert(index, selected);
            pause(&message);
            return;
        }
    }
    match selected.kind {
        ItemKind::Weapon => {
            let old = std::mem::replace(&mut c.equipped_weapon, selected);
            c.inventory.push(old);
        }
        ItemKind::Armor => {
            let old = std::mem::replace(&mut c.equipped_armor, selected);
            c.inventory.push(old);
        }
        ItemKind::Shield => {
            let old = std::mem::replace(&mut c.equipped_shield, selected);
            c.inventory.push(old);
        }
        ItemKind::HealthPotion => {
            let heal = (c.max_hp() / 4).max(1);
            c.hp = (c.hp + heal).min(c.max_hp());
            pause(&format!("Used a health potion and restored {heal} HP."));
        }
        ItemKind::ManaPotion => {
            let restore = (c.max_mana() / 4).max(1);
            c.mana = (c.mana + restore).min(c.max_mana());
            pause(&format!("Used a mana potion and restored {restore} mana."));
        }
    }
}

fn prompt(label: &str) -> String {
    print!("{label}");
    io::stdout().flush().expect("failed to flush stdout");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read input");
    input.trim_end().to_string()
}

fn read_key_char_nav() -> char {
    enable_raw_mode().expect("failed to enable raw mode");
    let key = loop {
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
    };
    disable_raw_mode().expect("failed to disable raw mode");
    key
}

fn read_key_char() -> char {
    enable_raw_mode().expect("failed to enable raw mode");
    let key = loop {
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
    };
    disable_raw_mode().expect("failed to disable raw mode");
    key
}

fn pause(message: &str) {
    println!("{YELLOW}{message}{RESET}");
    print_footer(&[&format!("{BOLD}Continue:{RESET} press any key")]);
    let _ = read_key_char();
}
fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_character() -> Character {
        Character::new("Tester".to_string(), DeathMode::Softcore)
    }

    fn open_test_dungeon(player_x: i32, player_y: i32, enemies: Vec<Enemy>) -> Dungeon {
        Dungeon {
            floor: 2,
            player_x,
            player_y,
            stairs_x: MAP_W - 2,
            stairs_y: MAP_H - 2,
            enemies,
            chests: Vec::new(),
            log: Vec::new(),
            tiles: vec!['.'; (MAP_W * MAP_H) as usize],
            bell_wave_tiles: Vec::new(),
            boss_turn_counter: 0,
            log_turn: 0,
        }
    }

    #[test]
    fn dungeon_log_groups_turns_even_when_action_has_no_message() {
        let mut c = test_character();
        c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

        let before = current_dungeon_log_len(&c);
        assert!(try_move(&mut c, 1, 0));
        mark_latest_log_group(&mut c, before, true, "Move east / attack");

        let d = c.active_dungeon.as_ref().unwrap();
        assert_eq!(d.log_turn, 1);
        assert_eq!(d.log, vec!["== Turn 1: Move east / attack =="]);
    }

    #[test]
    fn dungeon_log_labels_failed_commands_as_no_turn_spent() {
        let mut c = test_character();
        c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

        let before = current_dungeon_log_len(&c);
        assert!(!use_cleave(&mut c));
        mark_latest_log_group(&mut c, before, false, "Cleave");

        let d = c.active_dungeon.as_ref().unwrap();
        assert_eq!(d.log_turn, 0);
        assert_eq!(d.log[0], "== No turn spent: Cleave ==");
        assert_eq!(d.log[1], "No adjacent enemies for Cleave.");
    }

    #[test]
    fn new_ironbound_matches_mvp_starting_state() {
        let c = test_character();

        assert_eq!(c.class_name, "Ironbound");
        assert_eq!(c.level, 1);
        assert_eq!(c.gold, 50);
        assert_eq!((c.strength, c.dexterity, c.intelligence), (6, 3, 1));
        assert_eq!(c.max_hp(), 40);
        assert_eq!(c.max_mana(), 15);
        assert_eq!(c.hp, c.max_hp());
        assert_eq!(c.mana, c.max_mana());
        assert_eq!(c.inventory.len(), 3);
        assert_eq!(c.equipped_weapon.damage_min, 3);
        assert_eq!(c.equipped_weapon.damage_max, 5);
        assert_eq!(c.armor(), 4); // cloth 1 + shield 1 + Iron Guard rank 1 (+2)
        assert_eq!(
            (c.deep_cut_rank, c.iron_guard_rank, c.second_wind_rank),
            (1, 1, 1)
        );
        assert!(!c.bellkeeper_defeated);
        assert!(!c.act1_completed);
    }

    #[test]
    fn leveling_doubles_xp_requirements_and_grants_points() {
        let mut c = test_character();

        let levels_gained = add_xp(&mut c, 10);
        assert_eq!(levels_gained, vec![2]);
        assert_eq!(c.level, 2);
        assert_eq!(c.xp, 0);
        assert_eq!(c.unspent_attributes, 3);
        assert_eq!(c.unspent_skills, 1);

        let levels_gained = add_xp(&mut c, 20);
        assert_eq!(levels_gained, vec![3]);
        assert_eq!(c.level, 3);
        assert_eq!(c.xp, 0);
        assert_eq!(c.unspent_attributes, 6);
        assert_eq!(c.unspent_skills, 2);
    }

    #[test]
    fn skill_rank_scaling_matches_design() {
        assert_eq!(cleave_percent_for_rank(1), 80);
        assert_eq!(cleave_percent_for_rank(5), 120);
        assert_eq!(shield_bash_percent_for_rank(1), 70);
        assert_eq!(shield_bash_percent_for_rank(5), 110);
        assert_eq!(battle_cry_bonus_percent_for_rank(1), 20);
        assert_eq!(battle_cry_bonus_percent_for_rank(5), 40);
        assert_eq!(deep_cut_chance_for_rank(1), 15);
        assert_eq!(deep_cut_chance_for_rank(5), 35);
        assert_eq!(deep_cut_damage_for_rank(1), 2);
        assert_eq!(deep_cut_damage_for_rank(5), 4);
        assert_eq!(iron_guard_armor_bonus_for_rank(1), 2);
        assert_eq!(iron_guard_armor_bonus_for_rank(5), 6);
        assert_eq!(second_wind_heal_percent_for_rank(1), 10);
        assert_eq!(second_wind_heal_percent_for_rank(5), 30);
        assert_eq!(next_skill_rank(5), 5);
    }

    #[test]
    fn passive_skill_upgrades_require_branch_starter_rank_two() {
        let mut c = test_character();
        assert!(unmet_skill_prerequisite(&c, "Deep Cut").is_some());
        assert!(unmet_skill_prerequisite(&c, "Iron Guard").is_some());
        assert!(unmet_skill_prerequisite(&c, "Second Wind").is_some());

        c.unspent_skills = 6;
        c.cleave_rank = 2;
        c.shield_bash_rank = 2;
        c.battle_cry_rank = 2;
        upgrade_skill(&mut c, "Deep Cut");
        upgrade_skill(&mut c, "Iron Guard");
        upgrade_skill(&mut c, "Second Wind");

        assert_eq!(c.deep_cut_rank, 2);
        assert_eq!(c.iron_guard_rank, 2);
        assert_eq!(c.second_wind_rank, 2);
        assert_eq!(c.armor(), 5);
    }

    #[test]
    fn cursor_helpers_keep_selection_in_bounds_without_pages() {
        assert_eq!(scroll_offset(0, 20, 5), 0);
        assert_eq!(scroll_offset(4, 20, 5), 0);
        assert_eq!(scroll_offset(5, 20, 5), 1);
        assert_eq!(scroll_offset(19, 20, 5), 15);

        let mut selected = 9;
        clamp_selection(&mut selected, 3);
        assert_eq!(selected, 2);
        clamp_selection(&mut selected, 0);
        assert_eq!(selected, 0);
    }

    #[test]
    fn stash_move_selected_moves_requested_item_immediately() {
        let mut inventory = vec![health_potion(), mana_potion(), crude_axe()];
        let mut stash = Vec::new();

        let message = move_selected(&mut inventory, &mut stash, 1, "Stored");

        assert!(message.starts_with("Stored Mana Potion"));
        assert_eq!(inventory.len(), 2);
        assert_eq!(stash.len(), 1);
        assert!(matches!(stash[0].kind, ItemKind::ManaPotion));
        assert!(matches!(inventory[0].kind, ItemKind::HealthPotion));
        assert!(matches!(inventory[1].kind, ItemKind::Weapon));
    }

    #[test]
    fn blacksmith_salvage_converts_gear_to_type_shards() {
        let mut c = test_character();
        c.inventory.clear();
        c.inventory.push(crude_axe());
        c.inventory.push(health_potion());

        let message = salvage_inventory_item(&mut c, 0);

        assert!(message.contains("weapon shard"));
        assert_eq!(c.weapon_shards, 1);
        assert_eq!(c.inventory.len(), 1);
        assert!(matches!(c.inventory[0].kind, ItemKind::HealthPotion));
        assert!(salvage_inventory_item(&mut c, 0).contains("Only weapons"));
    }

    #[test]
    fn blacksmith_upgrades_equipped_gear_with_shards_and_gold() {
        let mut c = test_character();
        c.weapon_shards = 2;
        c.armor_shards = 2;
        c.shield_shards = 2;
        c.gold = 100;

        let weapon_message = upgrade_equipped_message(&mut c, UpgradeSlot::Weapon);
        assert!(weapon_message.contains("+1"));
        assert_eq!(c.equipped_weapon.upgrade_level, 1);
        assert_eq!(
            (c.equipped_weapon.damage_min, c.equipped_weapon.damage_max),
            (4, 6)
        );
        assert_eq!(c.weapon_shards, 0);
        assert_eq!(c.gold, 75);

        let armor_message = upgrade_equipped_message(&mut c, UpgradeSlot::Armor);
        assert!(armor_message.contains("+1"));
        assert_eq!(c.equipped_armor.upgrade_level, 1);
        assert_eq!(c.equipped_armor.armor, 2);

        let shield_message = upgrade_equipped_message(&mut c, UpgradeSlot::Shield);
        assert!(shield_message.contains("+1"));
        assert_eq!(c.equipped_shield.upgrade_level, 1);
        assert_eq!(c.equipped_shield.armor, 2);
    }

    #[test]
    fn blacksmith_upgrade_cost_scales_with_upgrade_level() {
        let mut item = crude_axe();
        assert_eq!(upgrade_cost(&item), (2, 25));
        upgrade_item(&mut item);
        assert_eq!(upgrade_cost(&item), (4, 50));
        assert_eq!(salvage_shard_yield(&item), 2);
    }

    #[test]
    fn equipping_weapon_swaps_old_weapon_back_to_inventory() {
        let mut c = test_character();
        c.inventory.push(crude_axe());
        let index = c.inventory.len() - 1;

        equip_or_use_inventory_item(&mut c, index);

        assert!(c.equipped_weapon.name.starts_with("Crude Axe"));
        assert!(
            c.inventory
                .iter()
                .any(|item| item.name.starts_with("Rusted Sword"))
        );
    }

    #[test]
    fn item_requirements_gate_equipping() {
        let c = test_character();
        let high_level_axe = item_with_rarity(
            "Test Axe",
            ItemKind::Weapon,
            100,
            8,
            10,
            0,
            0,
            -1,
            Rarity::Rare,
            5,
            10,
            0,
            0,
        );

        assert!(!can_equip_item(&c, &high_level_axe));
        assert!(
            unmet_requirements_message(&c, &high_level_axe)
                .unwrap()
                .contains("STR")
        );
    }

    #[test]
    fn higher_level_loot_has_higher_requirements_and_stats() {
        let low = item_with_rarity(
            "Low Axe",
            ItemKind::Weapon,
            60,
            4,
            6,
            0,
            0,
            -1,
            Rarity::Common,
            1,
            6,
            0,
            0,
        );
        let high = item_with_rarity(
            "High Axe",
            ItemKind::Weapon,
            120,
            8,
            10,
            0,
            0,
            -1,
            Rarity::Rare,
            5,
            10,
            0,
            0,
        );

        assert!(high.item_level > low.item_level);
        assert!(high.damage_max > low.damage_max);
        assert!(high.required_strength > low.required_strength);
        assert!(item_summary(&high).contains("ilvl 5"));
    }

    #[test]
    fn dungeon_generation_obeys_floor_content_rules() {
        for floor in 1..=3 {
            let d = generate_dungeon(floor);
            assert_eq!(d.floor, floor);
            assert_eq!(d.tiles.len(), (MAP_W * MAP_H) as usize);
            assert_eq!(dungeon_tile(&d, d.player_x, d.player_y), '.');
            assert_eq!(dungeon_tile(&d, d.stairs_x, d.stairs_y), '.');
            assert!((1..=3).contains(&d.chests.len()));
            assert!(d.enemies.iter().all(|e| dungeon_tile(&d, e.x, e.y) == '.'));
            assert!(
                d.chests
                    .iter()
                    .all(|ch| dungeon_tile(&d, ch.x, ch.y) == '.')
            );
        }

        let floor2 = generate_dungeon(2);
        let elite = floor2.enemies.iter().find(|e| e.glyph == 'E').unwrap();
        assert!(elite.elite_modifier.is_some());

        let floor3 = generate_dungeon(3);
        assert!(
            floor3
                .enemies
                .iter()
                .any(|e| e.is_boss && e.name == "Bellkeeper")
        );
    }

    #[test]
    fn stairs_advance_floors_but_floor_three_requires_boss() {
        let mut c = test_character();
        c.active_dungeon = Some(generate_dungeon(1));
        {
            let d = c.active_dungeon.as_mut().unwrap();
            d.player_x = d.stairs_x;
            d.player_y = d.stairs_y;
        }
        use_stairs(&mut c);
        assert_eq!(c.active_dungeon.as_ref().unwrap().floor, 2);

        c.active_dungeon = Some(generate_dungeon(3));
        {
            let d = c.active_dungeon.as_mut().unwrap();
            d.player_x = d.stairs_x;
            d.player_y = d.stairs_y;
        }
        use_stairs(&mut c);
        let d = c.active_dungeon.as_ref().unwrap();
        assert_eq!(d.floor, 3);
        assert!(d.log.iter().any(|line| line.contains("Bellkeeper blocks")));
    }

    #[test]
    fn elite_skeletons_have_exactly_one_modifier() {
        let elite = elite_skeleton(5, 5);

        assert_eq!(elite.glyph, 'E');
        assert!(elite.elite_modifier.is_some());
        assert!(elite.name.ends_with("Elite Skeleton"));
    }

    #[test]
    fn elite_modifiers_apply_expected_stat_effects() {
        let armored = elite_skeleton_with_modifier(1, 1, EliteModifier::Armored);
        assert_eq!(effective_enemy_armor(&armored), armored.armor + 2);

        let swift = elite_skeleton_with_modifier(1, 1, EliteModifier::Swift);
        assert_eq!(swift.speed, 12);

        let burning = elite_skeleton_with_modifier(1, 1, EliteModifier::Burning);
        assert_eq!(elite_damage_bonus(&burning), 1);
    }

    #[test]
    fn vampiric_elite_heals_after_dealing_damage() {
        let mut d = open_test_dungeon(
            2,
            2,
            vec![elite_skeleton_with_modifier(3, 2, EliteModifier::Vampiric)],
        );
        d.enemies[0].hp = d.enemies[0].max_hp - 5;

        apply_vampiric_heal(&mut d, 0);

        assert_eq!(d.enemies[0].hp, d.enemies[0].max_hp - 3);
        assert!(d.log.iter().any(|line| line.contains("drains life")));
    }

    #[test]
    fn boneguard_guards_at_range_and_gains_armor() {
        let mut c = test_character();
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![boneguard(5, 2)]));

        assert!(should_boneguard_guard(
            c.active_dungeon.as_ref().unwrap(),
            0
        ));
        enemy_turns(&mut c);

        let d = c.active_dungeon.as_ref().unwrap();
        assert!(d.enemies[0].guarding);
        assert_eq!(effective_enemy_armor(&d.enemies[0]), d.enemies[0].armor + 2);
        assert!(d.log.iter().any(|line| line.contains("raises its shield")));
    }

    #[test]
    fn adjacent_boneguard_attacks_instead_of_guarding() {
        let mut c = test_character();
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![boneguard(3, 2)]));

        assert!(!should_boneguard_guard(
            c.active_dungeon.as_ref().unwrap(),
            0
        ));
        enemy_turns(&mut c);

        let d = c.active_dungeon.as_ref().unwrap();
        assert!(!d.enemies[0].guarding);
        assert!(!d.log.iter().any(|line| line.contains("raises its shield")));
    }

    #[test]
    fn cultist_uses_shadow_bolt_at_cardinal_range_with_line_of_sight() {
        let mut c = test_character();
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![cultist(5, 2)]));

        assert!(can_cultist_ranged_attack(
            c.active_dungeon.as_ref().unwrap(),
            0
        ));
        enemy_turns(&mut c);

        let d = c.active_dungeon.as_ref().unwrap();
        assert_eq!((d.enemies[0].x, d.enemies[0].y), (5, 2));
        assert!(d.log.iter().any(|line| line.contains("shadow bolt")));
    }

    #[test]
    fn cultist_shadow_bolt_requires_clear_cardinal_line() {
        let mut d = open_test_dungeon(2, 2, vec![cultist(5, 2)]);
        d.tiles[tile_index(4, 2)] = '#';
        assert!(!can_cultist_ranged_attack(&d, 0));

        let mut diagonal = open_test_dungeon(2, 2, vec![cultist(4, 4)]);
        assert!(!can_cultist_ranged_attack(&diagonal, 0));

        diagonal.enemies[0] = skeleton(5, 2);
        assert!(!can_cultist_ranged_attack(&diagonal, 0));
    }

    #[test]
    fn bellkeeper_phase_and_enrage_damage_follow_health_thresholds() {
        let mut boss = bellkeeper(5, 5);
        assert_eq!(bellkeeper_phase(&boss), BellkeeperPhase::Tolling);
        boss.hp = 36;
        assert_eq!(bellkeeper_phase(&boss), BellkeeperPhase::CursedBell);
        boss.hp = 15;
        assert_eq!(bellkeeper_phase(&boss), BellkeeperPhase::Enraged);
        assert_eq!(bellkeeper_enrage_damage_bonus(&boss), 2);
    }

    #[test]
    fn bellkeeper_summons_skeletons_with_cap() {
        let mut d = open_test_dungeon(2, 2, vec![bellkeeper(5, 5)]);
        let mut occupied = vec![(5, 5)];

        for _ in 0..5 {
            summon_bellkeeper_skeleton(&mut d, 0, &mut occupied);
        }

        let summons = d
            .enemies
            .iter()
            .filter(|e| e.name == "Summoned Skeleton")
            .count();
        assert_eq!(summons, 3);
        assert!(
            d.log
                .iter()
                .any(|line| line.contains("skeleton claws free"))
        );
    }

    #[test]
    fn bellkeeper_wave_marks_map_tiles_and_damages_player_in_line() {
        let mut c = test_character();
        c.hp = c.max_hp();
        let mut d = open_test_dungeon(7, 5, vec![bellkeeper(5, 5)]);

        bellkeeper_wave(&mut c, &mut d, 0);

        assert!(d.bell_wave_tiles.contains(&(7, 5)));
        assert!(c.hp < c.max_hp());
        assert!(d.log.iter().any(|line| line.contains("bell wave hits")));
    }

    #[test]
    fn bellkeeper_bleed_death_completes_boss_fight_even_with_mobs_left() {
        let mut boss = bellkeeper(5, 5);
        boss.hp = 1;
        boss.bleed_turns = 1;
        boss.bleed_damage = 2;
        let mut c = test_character();
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![boss, skeleton(4, 2)]));

        enemy_turns(&mut c);

        assert!(c.bellkeeper_defeated);
        assert!(c.active_dungeon.is_none());
    }

    #[test]
    fn potion_hotkey_consumes_one_health_potion_and_caps_healing() {
        let mut c = test_character();
        c.active_dungeon = Some(generate_dungeon(1));
        c.hp = 1;
        let starting_potions = c
            .inventory
            .iter()
            .filter(|item| matches!(item.kind, ItemKind::HealthPotion))
            .count();

        use_potion(&mut c);

        let ending_potions = c
            .inventory
            .iter()
            .filter(|item| matches!(item.kind, ItemKind::HealthPotion))
            .count();
        assert_eq!(ending_potions, starting_potions - 1);
        assert_eq!(c.hp, 1 + c.max_hp() / 4);

        c.hp = c.max_hp() - 1;
        use_potion(&mut c);
        assert_eq!(c.hp, c.max_hp());
    }
}
