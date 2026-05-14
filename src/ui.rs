use crate::*;

pub(crate) fn print_town(c: &Character) {
    println!("{BOLD}{CYAN}+--------------------------------------------------+{RESET}");
    println!("{BOLD}{CYAN}| Town: Hollow's Rest                              |{RESET}");
    println!("{BOLD}{CYAN}+--------------------------------------------------+{RESET}");
    println!(
        "{BOLD}{}{RESET} the {GREEN}{}{RESET}  {}  {}  {}",
        c.name,
        c.class_name,
        level_text(c.level),
        xp_text(c.xp, xp_required_for_next_level(c.level)),
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
    if c.act2_completed {
        println!("{GREEN}Act II complete:{RESET} The Glass Wastes lie quiet under a cold dawn.");
    } else if c.glass_tyrant_defeated {
        println!(
            "{YELLOW}Quest ready to turn in:{RESET} Speak with Warden Mara ({GREEN}t{RESET}) about the Glass Tyrant."
        );
    } else if c.act1_completed {
        println!(
            "{CYAN}Act II:{RESET} Cross the Glass Wastes and shatter the Glass Tyrant on floor {FINAL_FLOOR}."
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

pub(crate) fn colored_stat(label: &str, value: impl std::fmt::Display, color: &str) -> String {
    format!("{color}{label} {value}{RESET}")
}

pub(crate) fn strength_text(value: u32) -> String {
    colored_stat("STR", value, RED)
}

pub(crate) fn dexterity_text(value: u32) -> String {
    colored_stat("DEX", value, GREEN)
}

pub(crate) fn intelligence_text(value: u32) -> String {
    colored_stat("INT", value, BLUE)
}

pub(crate) fn hp_text(current: u32, max: u32) -> String {
    format!("{RED}HP {current}/{max}{RESET}")
}

pub(crate) fn mana_text(current: u32, max: u32) -> String {
    format!("{BLUE}Mana {current}/{max}{RESET}")
}

pub(crate) fn gold_text(value: u32) -> String {
    format!("{YELLOW}Gold {value}{RESET}")
}

pub(crate) fn xp_text(current: u32, needed: u32) -> String {
    format!("{MAGENTA}XP {current}/{needed}{RESET}")
}

pub(crate) fn level_text(value: u32) -> String {
    format!("{CYAN}Level {value}{RESET}")
}

pub(crate) fn hit_text(value: u32) -> String {
    colored_stat("Hit", value, CYAN)
}

pub(crate) fn dodge_text(value: u32) -> String {
    colored_stat("Dodge", value, GREEN)
}

pub(crate) fn speed_text(value: u32) -> String {
    colored_stat("Speed", value, YELLOW)
}

pub(crate) fn armor_text(value: i32) -> String {
    colored_stat("Armor", value, WHITE)
}

pub(crate) fn unspent_attributes_text(value: u32) -> String {
    format!("{CYAN}Unspent attributes: {value}{RESET}")
}

pub(crate) fn unspent_skills_text(value: u32) -> String {
    format!("{MAGENTA}Unspent skills: {value}{RESET}")
}

pub(crate) fn shard_text(label: &str, value: u32) -> String {
    format!("{WHITE}{label} {value}{RESET}")
}

pub(crate) fn damage_text(value: impl std::fmt::Display) -> String {
    format!("{RED}{value} damage{RESET}")
}

pub(crate) fn xp_reward_text(value: u32) -> String {
    format!("{MAGENTA}{value} XP{RESET}")
}

pub(crate) fn gold_reward_text(value: u32) -> String {
    format!("{YELLOW}{value} gold{RESET}")
}

pub(crate) fn heal_amount_text(value: u32) -> String {
    format!("{GREEN}{value} HP{RESET}")
}

pub(crate) fn lesser_potion_restore(max_resource: u32) -> u32 {
    ((max_resource * LESSER_POTION_RESTORE_PERCENT) / 100).max(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogKind {
    Hit,
    Enemy,
    Miss,
    Kill,
    Loot,
    Boss,
    Heal,
    Warn,
    Status,
    Info,
}

impl LogKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            LogKind::Hit => "HIT",
            LogKind::Enemy => "ENEMY",
            LogKind::Miss => "MISS",
            LogKind::Kill => "KILL",
            LogKind::Loot => "LOOT",
            LogKind::Boss => "BOSS",
            LogKind::Heal => "HEAL",
            LogKind::Warn => "WARN",
            LogKind::Status => "STATUS",
            LogKind::Info => "INFO",
        }
    }
}

pub(crate) fn log_event(log: &mut Vec<String>, kind: LogKind, message: impl Into<String>) {
    log.push(format!("[{}] {}", kind.label(), message.into()));
}

pub(crate) fn enemy_hp_text(enemy: &Enemy) -> String {
    format!("HP {}/{}", enemy.hp.max(0), enemy.max_hp)
}

pub(crate) fn push_level_up_logs(log: &mut Vec<String>, levels_gained: &[u32]) {
    for level in levels_gained {
        log_event(
            log,
            LogKind::Status,
            format!("LEVEL UP! Reached level {level}. +3 attributes, +1 skill point."),
        );
        log_event(
            log,
            LogKind::Info,
            format!(
                "Town reminder: press {GREEN}a{RESET} for attributes and {GREEN}k{RESET} for skills."
            ),
        );
    }
}

pub(crate) fn prompt(label: &str) -> String {
    print!("{label}");
    io::stdout().flush().expect("failed to flush stdout");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read input");
    input.trim_end().to_string()
}

pub(crate) fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::stdout().flush();
}
