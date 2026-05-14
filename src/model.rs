const SAVE_PATH: &str = "saves/save.json";
const MAP_W: i32 = 40;
const MAP_H: i32 = 16;
const ACT1_FLOORS: u32 = 10;
const ACT2_FLOORS: u32 = 8;
const ACT2_START_FLOOR: u32 = ACT1_FLOORS + 1;
const FINAL_FLOOR: u32 = ACT1_FLOORS + ACT2_FLOORS;
const HEALTH_POTION_COST: u32 = 50;
const MANA_POTION_COST: u32 = 100;
const LESSER_POTION_RESTORE_PERCENT: u32 = 15;

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum SkillMastery {
    ReapingCleave,
    SunderingCleave,
    BloodArc,
    CrushingBash,
    LongBash,
    DazingBash,
    WarpathCry,
    TerrifyingCry,
    RallyingCry,
    Hemorrhage,
    OpenWound,
    Bloodletting,
    Bulwark,
    ShieldDiscipline,
    SpikedGuard,
    FreshKill,
    AdrenalSurge,
    GrimRecovery,
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
    #[serde(default)]
    energy: i32,
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
    armor_shred_turns: u32,
    #[serde(default)]
    vulnerable_turns: u32,
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
    glass_tyrant_defeated: bool,
    #[serde(default)]
    act1_completed: bool,
    #[serde(default)]
    act2_completed: bool,
    #[serde(default)]
    cleave_cooldown: u32,
    #[serde(default)]
    shield_bash_cooldown: u32,
    #[serde(default)]
    battle_cry_cooldown: u32,
    #[serde(default, alias = "battle_cry_turns")]
    battle_cry_charges: u32,
    #[serde(default)]
    active_dungeon: Option<Dungeon>,
    #[serde(default)]
    weapon_shards: u32,
    #[serde(default)]
    armor_shards: u32,
    #[serde(default)]
    shield_shards: u32,
    #[serde(default)]
    cleave_mastery: Option<SkillMastery>,
    #[serde(default)]
    shield_bash_mastery: Option<SkillMastery>,
    #[serde(default)]
    battle_cry_mastery: Option<SkillMastery>,
    #[serde(default)]
    deep_cut_mastery: Option<SkillMastery>,
    #[serde(default)]
    iron_guard_mastery: Option<SkillMastery>,
    #[serde(default)]
    second_wind_mastery: Option<SkillMastery>,
    #[serde(default)]
    second_wind_shield: u32,
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
            glass_tyrant_defeated: false,
            act1_completed: false,
            act2_completed: false,
            cleave_cooldown: 0,
            shield_bash_cooldown: 0,
            battle_cry_cooldown: 0,
            battle_cry_charges: 0,
            active_dungeon: None,
            weapon_shards: 0,
            armor_shards: 0,
            shield_shards: 0,
            cleave_mastery: None,
            shield_bash_mastery: None,
            battle_cry_mastery: None,
            deep_cut_mastery: None,
            iron_guard_mastery: None,
            second_wind_mastery: None,
            second_wind_shield: 0,
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
        let mastery_bonus = if self.iron_guard_mastery == Some(SkillMastery::ShieldDiscipline) {
            3
        } else {
            0
        };
        (10 + self.dexterity as i32 * 3
            + self.equipped_shield.dodge
            + self.equipped_armor.dodge
            + mastery_bonus)
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
        let bulwark_bonus = if self.iron_guard_mastery == Some(SkillMastery::Bulwark)
            && self.hp * 2 <= self.max_hp()
        {
            4
        } else {
            0
        };
        self.equipped_armor.armor
            + self.equipped_shield.armor
            + iron_guard_armor_bonus(self)
            + bulwark_bonus
    }
    fn weapon_damage(&self) -> (i32, i32) {
        (
            self.equipped_weapon.damage_min + (self.strength as i32 / 4),
            self.equipped_weapon.damage_max + (self.strength as i32 / 3),
        )
    }
}

