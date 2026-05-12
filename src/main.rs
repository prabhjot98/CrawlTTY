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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
enum Rarity {
    #[default]
    Common,
    Magic,
    Rare,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ItemKind {
    HealthPotion,
    ManaPotion,
    Weapon,
    Armor,
    Shield,
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
    hp: u32,
    mana: u32,
    inventory: Vec<Item>,
    stash: Vec<Item>,
    equipped_weapon: Item,
    equipped_armor: Item,
    equipped_shield: Item,
    bellkeeper_defeated: bool,
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
}

fn default_skill_rank() -> u32 {
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
            hp: max_hp,
            mana: max_mana,
            inventory: vec![health_potion(), health_potion(), mana_potion()],
            stash: Vec::new(),
            equipped_weapon: rusted_sword(),
            equipped_armor: cloth_tunic(),
            equipped_shield: worn_shield(),
            bellkeeper_defeated: false,
            cleave_cooldown: 0,
            shield_bash_cooldown: 0,
            battle_cry_cooldown: 0,
            battle_cry_turns: 0,
            active_dungeon: None,
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
        // Iron Guard passive MVP: +2 armor while using a shield.
        self.equipped_armor.armor + self.equipped_shield.armor + 2
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
            &format!("{BOLD}Town:{RESET} {GREEN}h{RESET}=healer  {GREEN}m{RESET}=merchant  {GREEN}b{RESET}=blacksmith  {GREEN}s{RESET}=stash  {GREEN}d{RESET}=dungeon"),
            &format!("{GREEN}i{RESET}=inventory  {GREEN}a{RESET}=attributes  {GREEN}k{RESET}=skill tree  {RED}q{RESET}=save+quit"),
        ]);
        match read_key_char() {
            'h' | 'H' => healer(&mut character),
            'm' | 'M' => merchant(&mut character),
            'b' | 'B' => blacksmith(&mut character),
            's' | 'S' => stash_menu(&mut character),
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
        "{BOLD}{}{RESET} the {GREEN}{}{RESET}  Level {}  XP {}  {YELLOW}Gold {}{RESET}",
        c.name, c.class_name, c.level, c.xp, c.gold
    );
    println!(
        "{RED}HP {}/{}{RESET}  {BLUE}Mana {}/{}{RESET}",
        c.hp,
        c.max_hp(),
        c.mana,
        c.max_mana()
    );
    println!(
        "STR {}  DEX {}  INT {}  Hit {}  Dodge {}  Speed {}",
        c.strength,
        c.dexterity,
        c.intelligence,
        c.hit_rating(),
        c.dodge_rating(),
        c.speed()
    );
    println!(
        "Unspent attributes: {}  Unspent skills: {}",
        c.unspent_attributes, c.unspent_skills
    );
    println!("Weapon: {}", c.equipped_weapon.name);
    println!("Armor : {}", c.equipped_armor.name);
    println!("Shield: {}", c.equipped_shield.name);
    if c.bellkeeper_defeated {
        println!("Quest complete: The Bellkeeper is dead. Act II placeholder unlocked.");
    } else {
        println!("Quest: Kill the Bellkeeper below the crypt.");
    }
}

fn healer(c: &mut Character) {
    c.hp = c.max_hp();
    c.mana = c.max_mana();
    pause("The healer restores your health and mana.");
}

fn merchant(c: &mut Character) {
    loop {
        clear_screen();
        println!("{BOLD}{YELLOW}Merchant{RESET} - Gold {}", c.gold);
        println!("Health Potion: 15 gold");
        println!("Mana Potion: 15 gold");
        println!("Selling gives 25% of item value.");
        print_footer(&[
            &format!("{BOLD}Merchant:{RESET} {GREEN}1{RESET}=buy health potion  {GREEN}2{RESET}=buy mana potion  {YELLOW}3{RESET}=sell selected item  {RED}Esc{RESET}=back"),
        ]);
        match read_key_char() {
            '1' => buy_item(c, health_potion()),
            '2' => buy_item(c, mana_potion()),
            '3' => sell_item_screen(c),
            '\u{1b}' => break,
            _ => pause("Unknown action."),
        }
    }
}

fn blacksmith(c: &mut Character) {
    loop {
        clear_screen();
        println!("{BOLD}{WHITE}Blacksmith{RESET} - Gold {}", c.gold);
        println!("No durability or repairs in the MVP.");
        println!("Crude Axe: 4-6 dmg, STR F, 60 gold");
        println!("Battered Mail: +2 armor, -5 speed, 55 gold");
        println!("Worn Shield: +1 armor, +2 dodge, 40 gold");
        print_footer(&[
            &format!("{BOLD}Blacksmith:{RESET} {GREEN}1{RESET}=Crude Axe  {GREEN}2{RESET}=Battered Mail  {GREEN}3{RESET}=Worn Shield  {RED}Esc{RESET}=back"),
        ]);
        let item = match read_key_char() {
            '1' => Some(crude_axe()),
            '2' => Some(battered_mail()),
            '3' => Some(worn_shield()),
            '\u{1b}' => break,
            _ => {
                pause("Unknown action.");
                None
            }
        };
        if let Some(item) = item {
            buy_item(c, item);
        }
    }
}

fn buy_item(c: &mut Character, item: Item) {
    if c.gold < item.value {
        pause("Not enough gold.");
        return;
    }
    c.gold -= item.value;
    let message = format!("Bought {}.", item.name);
    c.inventory.push(item);
    pause(&message);
}

fn sell_item_screen(c: &mut Character) {
    let mut offset = 0usize;
    loop {
        clear_screen();
        println!("{BOLD}{YELLOW}Sell Items{RESET} - Gold {}", c.gold);
        if c.inventory.is_empty() {
            println!("Inventory is empty.");
        } else {
            print_inventory_page(c, offset);
        }
        print_footer(&[&format!(
            "{BOLD}Sell:{RESET} {YELLOW}1-9{RESET}=sell item  {GREEN}n/p{RESET}=page  {RED}Esc{RESET}=back"
        )]);
        match read_key_char() {
            '\u{1b}' => break,
            'n' | 'N' => offset = next_inventory_offset(c.inventory.len(), offset),
            'p' | 'P' => offset = offset.saturating_sub(9),
            key @ '1'..='9' => {
                let index = offset + key as usize - '1' as usize;
                if index >= c.inventory.len() {
                    pause("No item in that slot.");
                    continue;
                }
                let item = c.inventory.remove(index);
                let sell_value = item.value / 4;
                c.gold += sell_value;
                pause(&format!("Sold {} for {} gold.", item.name, sell_value));
                offset = clamp_inventory_offset(c.inventory.len(), offset);
            }
            _ => pause("Unknown sell command."),
        }
    }
}

fn stash_menu(c: &mut Character) {
    loop {
        clear_screen();
        println!("{BOLD}{MAGENTA}Stash{RESET}");
        println!("Inventory items: {}", c.inventory.len());
        println!("Stash items: {}", c.stash.len());
        print_footer(&[
            &format!("{BOLD}Stash:{RESET} {GREEN}1{RESET}=store first item  {GREEN}2{RESET}=retrieve first item  {GREEN}3{RESET}=view lists  {RED}Esc{RESET}=back"),
        ]);
        match read_key_char() {
            '1' => move_first(&mut c.inventory, &mut c.stash, "Stored"),
            '2' => move_first(&mut c.stash, &mut c.inventory, "Retrieved"),
            '3' => inventory_and_stash_screen(c),
            '\u{1b}' => break,
            _ => pause("Unknown action."),
        }
    }
}

fn move_first(from: &mut Vec<Item>, to: &mut Vec<Item>, verb: &str) {
    if from.is_empty() {
        pause("Nothing to move.");
    } else {
        let item = from.remove(0);
        let msg = format!("{} {}.", verb, item.name);
        to.push(item);
        pause(&msg);
    }
}

fn spend_attributes(c: &mut Character) {
    while c.unspent_attributes > 0 {
        clear_screen();
        println!(
            "{BOLD}{CYAN}Spend attributes{RESET} ({} left)",
            c.unspent_attributes
        );
        println!(
            "1) Strength {} -> {} (+5 max HP)",
            c.strength,
            c.strength + 1
        );
        println!(
            "2) Dexterity {} -> {} (+5 hit, +5 speed)",
            c.dexterity,
            c.dexterity + 1
        );
        println!(
            "3) Intelligence {} -> {} (+5 max mana)",
            c.intelligence,
            c.intelligence + 1
        );
        print_footer(&[
            &format!("{BOLD}Attributes:{RESET} {GREEN}1{RESET}=Strength  {GREEN}2{RESET}=Dexterity  {GREEN}3{RESET}=Intelligence  {RED}Esc{RESET}=back"),
        ]);
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
        println!("Unspent skill points: {}", c.unspent_skills);
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
        println!();
        println!("Each upgrade costs 1 skill point. Max rank is 5.");
        print_footer(&[
            &format!("{BOLD}Skill Tree:{RESET} {GREEN}1{RESET}=upgrade Cleave  {GREEN}2{RESET}=upgrade Shield Bash  {GREEN}3{RESET}=upgrade Battle Cry  {RED}Esc{RESET}=back"),
        ]);
        match read_key_char() {
            '1' => upgrade_skill(c, "Cleave"),
            '2' => upgrade_skill(c, "Shield Bash"),
            '3' => upgrade_skill(c, "Battle Cry"),
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
    println!("   Current: {current_value}{value_label}; {details}");
    if rank >= 5 {
        println!("   Next: {YELLOW}MAX RANK{RESET}");
    } else {
        println!(
            "   Next rank {}: {next_value}{value_label}; {details}",
            rank + 1
        );
    }
}

fn upgrade_skill(c: &mut Character, skill: &str) {
    if c.unspent_skills == 0 {
        pause("No unspent skill points.");
        return;
    }
    let rank = match skill {
        "Cleave" => &mut c.cleave_rank,
        "Shield Bash" => &mut c.shield_bash_rank,
        "Battle Cry" => &mut c.battle_cry_rank,
        _ => return,
    };
    if *rank >= 5 {
        pause("That skill is already at max rank.");
        return;
    }
    *rank += 1;
    c.unspent_skills -= 1;
    // Do not pause here: immediately return to the skill tree loop so the
    // upgraded rank and next-rank preview redraw right away.
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

fn enter_dungeon(c: &mut Character) {
    if c.bellkeeper_defeated {
        pause("Act II is visible beyond the road, but it is not playable yet.");
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

fn rat(x: i32, y: i32) -> Enemy {
    Enemy {
        name: "Rat".to_string(),
        glyph: 'r',
        x,
        y,
        hp: 6,
        max_hp: 6,
        damage_min: 1,
        damage_max: 2,
        armor: 0,
        speed: 11,
        xp: 8,
        gold_min: 0,
        gold_max: 3,
        is_boss: false,
        stunned_turns: 0,
        bleed_turns: 0,
        bleed_damage: 0,
    }
}
fn skeleton(x: i32, y: i32) -> Enemy {
    Enemy {
        name: "Skeleton".to_string(),
        glyph: 's',
        x,
        y,
        hp: 12,
        max_hp: 12,
        damage_min: 2,
        damage_max: 4,
        armor: 1,
        speed: 9,
        xp: 18,
        gold_min: 2,
        gold_max: 8,
        is_boss: false,
        stunned_turns: 0,
        bleed_turns: 0,
        bleed_damage: 0,
    }
}
fn cultist(x: i32, y: i32) -> Enemy {
    Enemy {
        name: "Cultist".to_string(),
        glyph: 'c',
        x,
        y,
        hp: 10,
        max_hp: 10,
        damage_min: 2,
        damage_max: 3,
        armor: 0,
        speed: 10,
        xp: 22,
        gold_min: 5,
        gold_max: 12,
        is_boss: false,
        stunned_turns: 0,
        bleed_turns: 0,
        bleed_damage: 0,
    }
}
fn boneguard(x: i32, y: i32) -> Enemy {
    Enemy {
        name: "Boneguard".to_string(),
        glyph: 'b',
        x,
        y,
        hp: 18,
        max_hp: 18,
        damage_min: 3,
        damage_max: 5,
        armor: 2,
        speed: 8,
        xp: 35,
        gold_min: 8,
        gold_max: 18,
        is_boss: false,
        stunned_turns: 0,
        bleed_turns: 0,
        bleed_damage: 0,
    }
}
fn elite_skeleton(x: i32, y: i32) -> Enemy {
    Enemy {
        name: "Elite Skeleton".to_string(),
        glyph: 'E',
        x,
        y,
        hp: 24,
        max_hp: 24,
        damage_min: 3,
        damage_max: 6,
        armor: 2,
        speed: 10,
        xp: 54,
        gold_min: 20,
        gold_max: 40,
        is_boss: false,
        stunned_turns: 0,
        bleed_turns: 0,
        bleed_damage: 0,
    }
}
fn bellkeeper(x: i32, y: i32) -> Enemy {
    Enemy {
        name: "Bellkeeper".to_string(),
        glyph: 'B',
        x,
        y,
        hp: 60,
        max_hp: 60,
        damage_min: 5,
        damage_max: 8,
        armor: 3,
        speed: 8,
        xp: 250,
        gold_min: 100,
        gold_max: 150,
        is_boss: true,
        stunned_turns: 0,
        bleed_turns: 0,
        bleed_damage: 0,
    }
}

fn dungeon_loop(c: &mut Character) -> Result<()> {
    loop {
        clear_screen();
        draw_dungeon(c);
        print_skill_help(c);
        print_dungeon_footer();
        let mut took_turn = false;
        match read_key_char() {
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
        "{BOLD}Hollow Crypts Floor {}{RESET}  {RED}HP {}/{}{RESET} {BLUE}Mana {}/{}{RESET} {YELLOW}Gold {}{RESET} XP {}",
        d.floor,
        c.hp,
        c.max_hp(),
        c.mana,
        c.max_mana(),
        c.gold,
        c.xp
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
    println!("{BOLD}--- Log ---{RESET}");
    for msg in d.log.iter().rev().take(6).rev() {
        println!("{msg}");
    }
}

fn print_dungeon_footer() {
    print_footer(&[
        &format!("{BOLD}Dungeon:{RESET} {GREEN}w/a/s/d{RESET}=move/attack  {GREEN}1{RESET}=Cleave  {GREEN}2{RESET}=Bash  {GREEN}3{RESET}=Cry  {BLUE}p{RESET}=potion  i=inventory  {RED}Esc{RESET}=town"),
        &format!("{BOLD}Legend:{RESET} {GREEN}@{RESET}=you {BRIGHT_BLACK}#{RESET}=wall {DIM}.{RESET}=floor {YELLOW}${RESET}=chest {MAGENTA}E{RESET}=elite {RED}B{RESET}=boss"),
    ]);
}

fn print_skill_help(c: &Character) {
    print_above_footer(&[
        &format!("{GREEN}1 Cleave r{}{RESET}: cost 5 mana, cd 1. Hit up to 3 enemies for {}% weapon damage. Ready in {}.", c.cleave_rank, cleave_percent(c), c.cleave_cooldown),
        &format!("{GREEN}2 Shield Bash r{}{RESET}: cost 6 mana, cd 3. Hit 1 enemy for {}% damage and stun 1 turn. Ready in {}.", c.shield_bash_rank, shield_bash_percent(c), c.shield_bash_cooldown),
        &format!("{GREEN}3 Battle Cry r{}{RESET}: cost 8 mana, cd 6. +{}% damage, -10% enemy damage, Second Wind on kill. Ready in {}, active {}.", c.battle_cry_rank, battle_cry_bonus_percent(c), c.battle_cry_cooldown, c.battle_cry_turns),
    ], 2);
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
    let damage = (raw - enemy.armor).max(1);
    enemy.hp -= damage;
    d.log
        .push(format!("You {verb} {} for {} damage.", enemy.name, damage));
    if rng.gen_bool(0.15) && enemy.hp > 0 {
        enemy.bleed_turns = 3;
        enemy.bleed_damage = 2;
        d.log.push(format!("{} starts bleeding.", enemy.name));
    }
    if enemy.hp <= 0 {
        let gold = rng.gen_range(enemy.gold_min..=enemy.gold_max);
        let xp = enemy.xp;
        let name = enemy.name.clone();
        let was_boss = enemy.is_boss;
        c.gold += gold;
        add_xp(c, xp);
        let d = c.active_dungeon.as_mut().unwrap();
        d.log
            .push(format!("{name} dies. Gained {xp} XP and {gold} gold."));
        if c.battle_cry_turns > 0 {
            let heal = (c.max_hp() / 10).max(1);
            c.hp = (c.hp + heal).min(c.max_hp());
            let d = c.active_dungeon.as_mut().unwrap();
            d.log.push(format!("Second Wind restores {heal} HP."));
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
    let mut occupied: Vec<(i32, i32)> = d
        .enemies
        .iter()
        .filter(|e| e.hp > 0)
        .map(|e| (e.x, e.y))
        .collect();
    for i in 0..d.enemies.len() {
        if d.enemies[i].hp <= 0 {
            continue;
        }
        if d.enemies[i].bleed_turns > 0 {
            d.enemies[i].hp -= d.enemies[i].bleed_damage;
            d.enemies[i].bleed_turns -= 1;
            d.log.push(format!(
                "{} bleeds for {} damage.",
                d.enemies[i].name, d.enemies[i].bleed_damage
            ));
            if d.enemies[i].hp <= 0 {
                d.log
                    .push(format!("{} dies from bleeding.", d.enemies[i].name));
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
        let dist = (d.enemies[i].x - d.player_x).abs() + (d.enemies[i].y - d.player_y).abs();
        if dist == 1 {
            let mut rng = rand::thread_rng();
            if hit_roll(25, c.dodge_rating() as i32) {
                let raw = rng.gen_range(d.enemies[i].damage_min..=d.enemies[i].damage_max);
                let cry_penalty = if c.battle_cry_turns > 0 { 0.90 } else { 1.0 };
                let damage = (((raw - c.armor()).max(1) as f32) * cry_penalty)
                    .round()
                    .max(1.0) as u32;
                c.hp = c.hp.saturating_sub(damage);
                d.log.push(format!(
                    "{} hits you for {} damage.",
                    d.enemies[i].name, damage
                ));
            } else {
                d.log.push(format!("{} misses you.", d.enemies[i].name));
            }
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
        d.log.push(format!("*** LOOT DROPPED: {name} ***"));
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
    let bonus = match rarity {
        Rarity::Common => 0,
        Rarity::Magic => 1,
        Rarity::Rare => 2,
    } + floor as i32
        - 1;
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

fn add_xp(c: &mut Character, amount: u32) {
    c.xp += amount;
    loop {
        let needed = 10 * 2u32.pow(c.level - 1);
        if c.xp < needed {
            break;
        }
        c.xp -= needed;
        c.level += 1;
        c.unspent_attributes += 3;
        c.unspent_skills += 1;
        c.hp = c.max_hp();
        c.mana = c.max_mana();
    }
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
            "Opened chest and found {YELLOW}{gold} gold{RESET} and {name}."
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
        c.active_dungeon
            .as_mut()
            .unwrap()
            .log
            .push(format!("You drink a health potion and restore {heal} HP."));
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
    let mut offset = 0usize;
    loop {
        clear_screen();
        println!("{BOLD}{CYAN}Equipment{RESET}");
        println!("Weapon: {}", item_summary(&c.equipped_weapon));
        println!("Armor : {}", item_summary(&c.equipped_armor));
        println!("Shield: {}", item_summary(&c.equipped_shield));
        println!(
            "Total armor: {}  Dodge: {}  Speed: {}",
            c.armor(),
            c.dodge_rating(),
            c.speed()
        );
        println!();
        println!("{BOLD}Inventory{RESET}");
        if c.inventory.is_empty() {
            println!("  Empty");
        } else {
            print_inventory_page(c, offset);
        }
        print_footer(&[&format!(
            "{BOLD}Inventory:{RESET} {GREEN}1-9{RESET}=equip/use  {GREEN}n/p{RESET}=page  {RED}x{RESET}=drop item  {RED}Esc{RESET}=back"
        )]);
        match read_key_char() {
            '\u{1b}' => break,
            'n' | 'N' => offset = next_inventory_offset(c.inventory.len(), offset),
            'p' | 'P' => offset = offset.saturating_sub(9),
            'x' | 'X' => {
                drop_item_from_page(c, offset);
                offset = clamp_inventory_offset(c.inventory.len(), offset);
            }
            key @ '1'..='9' => {
                let index = offset + key as usize - '1' as usize;
                equip_or_use_inventory_item(c, index);
                offset = clamp_inventory_offset(c.inventory.len(), offset);
            }
            _ => pause("Unknown inventory command."),
        }
    }
}

fn print_inventory_page(c: &Character, offset: usize) {
    let total = c.inventory.len();
    let end = (offset + 9).min(total);
    println!("Showing items {}-{} of {}", offset + 1, end, total);
    for (i, item) in c.inventory.iter().enumerate().skip(offset).take(9) {
        println!("  {}) {}", i - offset + 1, item_summary(item));
        if let Some(compare) = item_comparison(c, item) {
            println!("     {compare}");
        }
    }
}

fn next_inventory_offset(total: usize, offset: usize) -> usize {
    if offset + 9 < total {
        offset + 9
    } else {
        offset
    }
}

fn clamp_inventory_offset(total: usize, offset: usize) -> usize {
    if total == 0 {
        0
    } else if offset >= total {
        ((total - 1) / 9) * 9
    } else {
        offset
    }
}

fn drop_item_from_page(c: &mut Character, offset: usize) {
    if c.inventory.is_empty() {
        pause("Inventory is empty.");
        return;
    }
    println!("{RED}Drop which item?{RESET}");
    print_footer(&[&format!(
        "{BOLD}Drop:{RESET} {RED}1-9{RESET}=drop item from this page  Esc=cancel"
    )]);
    match read_key_char() {
        '\u{1b}' => {}
        key @ '1'..='9' => {
            let index = offset + key as usize - '1' as usize;
            if index >= c.inventory.len() {
                pause("No item in that slot.");
            } else {
                let item = c.inventory.remove(index);
                pause(&format!("Dropped {}.", item.name));
            }
        }
        _ => pause("Drop cancelled."),
    }
}

fn item_summary(item: &Item) -> String {
    let rarity = rarity_name(&item.rarity);
    let name = colored_item_name(item);
    match item.kind {
        ItemKind::Weapon => format!(
            "{} [{} {:?}] dmg {}-{} value {}",
            name, rarity, item.kind, item.damage_min, item.damage_max, item.value
        ),
        ItemKind::Armor | ItemKind::Shield => format!(
            "{} [{} {:?}] armor {} dodge {} speed {} value {}",
            name, rarity, item.kind, item.armor, item.dodge, item.speed, item.value
        ),
        _ => format!("{} [{:?}] value {}", name, item.kind, item.value),
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
    match item.kind {
        ItemKind::Weapon => {
            let cur_avg = c.equipped_weapon.damage_min + c.equipped_weapon.damage_max;
            let new_avg = item.damage_min + item.damage_max;
            Some(format_delta("damage", new_avg - cur_avg))
        }
        ItemKind::Armor => Some(format!(
            "Compare: {}  {}  {}",
            format_delta("armor", item.armor - c.equipped_armor.armor),
            format_delta("dodge", item.dodge - c.equipped_armor.dodge),
            format_delta("speed", item.speed - c.equipped_armor.speed)
        )),
        ItemKind::Shield => Some(format!(
            "Compare: {}  {}  {}",
            format_delta("armor", item.armor - c.equipped_shield.armor),
            format_delta("dodge", item.dodge - c.equipped_shield.dodge),
            format_delta("speed", item.speed - c.equipped_shield.speed)
        )),
        _ => None,
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

fn equip_or_use_inventory_item(c: &mut Character, index: usize) {
    if index >= c.inventory.len() {
        pause("No item in that slot.");
        return;
    }
    let selected = c.inventory.remove(index);
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

fn inventory_and_stash_screen(c: &Character) {
    clear_screen();
    println!("Inventory:");
    for (i, item) in c.inventory.iter().enumerate() {
        println!("  {}) {}", i + 1, item.name);
    }
    println!("\nStash:");
    for (i, item) in c.stash.iter().enumerate() {
        println!("  {}) {}", i + 1, item.name);
    }
    pause("Press any key to return.");
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
