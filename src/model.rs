use crate::*;

pub(crate) const SAVE_PATH: &str = "saves/save.json";
pub(crate) const MAP_W: i32 = 40;
pub(crate) const MAP_H: i32 = 16;
pub(crate) const ACT1_FLOORS: u32 = 10;
pub(crate) const ACT2_FLOORS: u32 = 8;
pub(crate) const ACT2_START_FLOOR: u32 = ACT1_FLOORS + 1;
pub(crate) const FINAL_FLOOR: u32 = ACT1_FLOORS + ACT2_FLOORS;
pub(crate) const HEALTH_POTION_COST: u32 = 50;
pub(crate) const MANA_POTION_COST: u32 = 100;
pub(crate) const LESSER_POTION_RESTORE_PERCENT: u32 = 15;

pub(crate) const RESET: &str = "\x1b[0m";
pub(crate) const BOLD: &str = "\x1b[1m";
pub(crate) const DIM: &str = "\x1b[2m";
pub(crate) const RED: &str = "\x1b[31m";
pub(crate) const GREEN: &str = "\x1b[32m";
pub(crate) const YELLOW: &str = "\x1b[33m";
pub(crate) const BLUE: &str = "\x1b[34m";
pub(crate) const MAGENTA: &str = "\x1b[35m";
pub(crate) const CYAN: &str = "\x1b[36m";
pub(crate) const WHITE: &str = "\x1b[37m";
pub(crate) const BRIGHT_BLACK: &str = "\x1b[90m";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum DeathMode {
    Softcore,
    Hardcore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Item {
    pub(crate) name: String,
    pub(crate) kind: ItemKind,
    pub(crate) value: u32,
    #[serde(default)]
    pub(crate) damage_min: i32,
    #[serde(default)]
    pub(crate) damage_max: i32,
    #[serde(default)]
    pub(crate) armor: i32,
    #[serde(default)]
    pub(crate) dodge: i32,
    #[serde(default)]
    pub(crate) speed: i32,
    #[serde(default)]
    pub(crate) rarity: Rarity,
    #[serde(default = "default_item_level")]
    pub(crate) item_level: u32,
    #[serde(default)]
    pub(crate) required_strength: u32,
    #[serde(default)]
    pub(crate) required_dexterity: u32,
    #[serde(default)]
    pub(crate) required_intelligence: u32,
    #[serde(default)]
    pub(crate) upgrade_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub(crate) enum Rarity {
    #[default]
    Common,
    Magic,
    Rare,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ItemKind {
    HealthPotion,
    ManaPotion,
    Weapon,
    Armor,
    Shield,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum EliteModifier {
    Armored,
    Swift,
    Vampiric,
    Burning,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum SkillMastery {
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
pub(crate) struct Enemy {
    pub(crate) name: String,
    pub(crate) glyph: char,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) hp: i32,
    pub(crate) max_hp: i32,
    pub(crate) damage_min: i32,
    pub(crate) damage_max: i32,
    pub(crate) armor: i32,
    pub(crate) speed: i32,
    #[serde(default)]
    pub(crate) energy: i32,
    pub(crate) xp: u32,
    pub(crate) gold_min: u32,
    pub(crate) gold_max: u32,
    pub(crate) is_boss: bool,
    #[serde(default)]
    pub(crate) stunned_turns: u32,
    #[serde(default)]
    pub(crate) bleed_turns: u32,
    #[serde(default)]
    pub(crate) bleed_damage: i32,
    #[serde(default)]
    pub(crate) armor_shred_turns: u32,
    #[serde(default)]
    pub(crate) vulnerable_turns: u32,
    #[serde(default)]
    pub(crate) guarding: bool,
    #[serde(default)]
    pub(crate) elite_modifier: Option<EliteModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Chest {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) opened: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Dungeon {
    pub(crate) floor: u32,
    pub(crate) player_x: i32,
    pub(crate) player_y: i32,
    pub(crate) stairs_x: i32,
    pub(crate) stairs_y: i32,
    pub(crate) enemies: Vec<Enemy>,
    pub(crate) chests: Vec<Chest>,
    pub(crate) log: Vec<String>,
    #[serde(default)]
    pub(crate) tiles: Vec<char>,
    #[serde(default)]
    pub(crate) bell_wave_tiles: Vec<(i32, i32)>,
    #[serde(default)]
    pub(crate) boss_turn_counter: u32,
    #[serde(default)]
    pub(crate) log_turn: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Character {
    pub(crate) name: String,
    pub(crate) class_name: String,
    pub(crate) death_mode: DeathMode,
    pub(crate) level: u32,
    pub(crate) xp: u32,
    pub(crate) gold: u32,
    pub(crate) strength: u32,
    pub(crate) dexterity: u32,
    pub(crate) intelligence: u32,
    #[serde(default)]
    pub(crate) unspent_attributes: u32,
    #[serde(default)]
    pub(crate) unspent_skills: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) cleave_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) shield_bash_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) battle_cry_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) deep_cut_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) iron_guard_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) second_wind_rank: u32,
    pub(crate) hp: u32,
    pub(crate) mana: u32,
    pub(crate) inventory: Vec<Item>,
    pub(crate) stash: Vec<Item>,
    pub(crate) equipped_weapon: Item,
    pub(crate) equipped_armor: Item,
    pub(crate) equipped_shield: Item,
    pub(crate) bellkeeper_defeated: bool,
    #[serde(default)]
    pub(crate) glass_tyrant_defeated: bool,
    #[serde(default)]
    pub(crate) act1_completed: bool,
    #[serde(default)]
    pub(crate) act2_completed: bool,
    #[serde(default)]
    pub(crate) cleave_cooldown: u32,
    #[serde(default)]
    pub(crate) shield_bash_cooldown: u32,
    #[serde(default)]
    pub(crate) battle_cry_cooldown: u32,
    #[serde(default, alias = "battle_cry_turns")]
    pub(crate) battle_cry_charges: u32,
    #[serde(default)]
    pub(crate) active_dungeon: Option<Dungeon>,
    #[serde(default)]
    pub(crate) weapon_shards: u32,
    #[serde(default)]
    pub(crate) armor_shards: u32,
    #[serde(default)]
    pub(crate) shield_shards: u32,
    #[serde(default)]
    pub(crate) cleave_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) shield_bash_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) battle_cry_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) deep_cut_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) iron_guard_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) second_wind_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) second_wind_shield: u32,
}

pub(crate) fn default_skill_rank() -> u32 {
    1
}

pub(crate) fn default_item_level() -> u32 {
    1
}

impl Character {
    pub(crate) fn new(name: String, death_mode: DeathMode) -> Self {
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

    pub(crate) fn max_hp(&self) -> u32 {
        10 + self.strength * 5
    }
    pub(crate) fn max_mana(&self) -> u32 {
        10 + self.intelligence * 5
    }
    pub(crate) fn hit_rating(&self) -> u32 {
        10 + self.dexterity * 5
    }
    pub(crate) fn dodge_rating(&self) -> u32 {
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
    pub(crate) fn speed(&self) -> u32 {
        (10 + self.dexterity as i32 * 5
            + self.equipped_weapon.speed
            + self.equipped_armor.speed
            + self.equipped_shield.speed)
            .max(1) as u32
    }
    pub(crate) fn armor(&self) -> i32 {
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
    pub(crate) fn weapon_damage(&self) -> (i32, i32) {
        (
            self.equipped_weapon.damage_min + (self.strength as i32 / 4),
            self.equipped_weapon.damage_max + (self.strength as i32 / 3),
        )
    }
}
