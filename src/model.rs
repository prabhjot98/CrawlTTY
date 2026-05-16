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
pub(crate) const STARTING_BAG_COLUMNS: u16 = 4;
pub(crate) const STARTING_BAG_ROWS: u16 = 4;
#[allow(dead_code)]
pub(crate) const MAX_BAG_COLUMNS: u16 = 8;
#[allow(dead_code)]
pub(crate) const MAX_BAG_ROWS: u16 = 8;
pub(crate) const STARTING_STASH_COLUMNS: u16 = 8;
pub(crate) const STARTING_STASH_ROWS: u16 = 8;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum DeathMode {
    Softcore,
    Hardcore,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum TownProject {
    RebuildForge,
    ReinforcedAnvil,
    SocketBench,
    StorehouseShelves,
    PackHooks,
    OilclothSatchel,
    QuartermasterLedger,
    ReinforcedPack,
    StitchedPockets,
    DeepRucksack,
    ExilesTrunk,
    HireAppraiser,
    HerbGarden,
    Distillery,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum GemKind {
    Ruby,
    Sapphire,
    Garnet,
    Emerald,
    Amethyst,
    Quartz,
    Jade,
    Onyx,
    Citrine,
    Topaz,
    Opal,
    Bloodstone,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum GemTier {
    Chipped,
    Flawed,
    Pristine,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GemSocket {
    pub(crate) gem_kind: GemKind,
    pub(crate) gem_tier: GemTier,
}

impl GemSocket {
    #[allow(dead_code)]
    pub(crate) fn filled(gem_kind: GemKind, gem_tier: GemTier) -> Self {
        Self { gem_kind, gem_tier }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct GemBonuses {
    pub(crate) max_hp: u32,
    pub(crate) max_mana: u32,
    pub(crate) strength: u32,
    pub(crate) dexterity: u32,
    pub(crate) intelligence: u32,
    pub(crate) hit_rating: u32,
    pub(crate) dodge_rating: i32,
    pub(crate) armor: i32,
    pub(crate) speed: i32,
    pub(crate) crit_chance: u32,
    pub(crate) gold_found_percent: u32,
    pub(crate) weapon_damage: i32,
}

impl GemBonuses {
    fn add(self, other: Self) -> Self {
        Self {
            max_hp: self.max_hp + other.max_hp,
            max_mana: self.max_mana + other.max_mana,
            strength: self.strength + other.strength,
            dexterity: self.dexterity + other.dexterity,
            intelligence: self.intelligence + other.intelligence,
            hit_rating: self.hit_rating + other.hit_rating,
            dodge_rating: self.dodge_rating + other.dodge_rating,
            armor: self.armor + other.armor,
            speed: self.speed + other.speed,
            crit_chance: self.crit_chance + other.crit_chance,
            gold_found_percent: self.gold_found_percent + other.gold_found_percent,
            weapon_damage: self.weapon_damage + other.weapon_damage,
        }
    }
}

pub(crate) fn gem_bonus(kind: GemKind, tier: GemTier) -> GemBonuses {
    let tier_index = match tier {
        GemTier::Chipped => 0,
        GemTier::Flawed => 1,
        GemTier::Pristine => 2,
    };
    let mut bonus = GemBonuses::default();
    match kind {
        GemKind::Ruby => bonus.max_hp = [5, 10, 20][tier_index],
        GemKind::Sapphire => bonus.max_mana = [3, 6, 12][tier_index],
        GemKind::Garnet => bonus.strength = [1, 2, 3][tier_index],
        GemKind::Emerald => bonus.dexterity = [1, 2, 3][tier_index],
        GemKind::Amethyst => bonus.intelligence = [1, 2, 3][tier_index],
        GemKind::Quartz => bonus.hit_rating = [3, 6, 10][tier_index],
        GemKind::Jade => bonus.dodge_rating = [2, 4, 8][tier_index],
        GemKind::Onyx => bonus.armor = [1, 2, 3][tier_index],
        GemKind::Citrine => bonus.speed = [2, 4, 7][tier_index],
        GemKind::Topaz => bonus.crit_chance = [1, 2, 4][tier_index],
        GemKind::Opal => bonus.gold_found_percent = [5, 10, 20][tier_index],
        GemKind::Bloodstone => bonus.weapon_damage = [1, 2, 3][tier_index],
    }
    bonus
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
    pub(crate) crit_chance: u32,
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
    #[serde(default)]
    pub(crate) sockets: Vec<Option<GemSocket>>,
    #[serde(default)]
    pub(crate) gem_kind: Option<GemKind>,
    #[serde(default)]
    pub(crate) gem_tier: Option<GemTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ItemGrid {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
    pub(crate) items: Vec<Item>,
}

impl ItemGrid {
    pub(crate) fn new(columns: u16, rows: u16, items: Vec<Item>) -> Self {
        let capacity = usize::from(columns) * usize::from(rows);
        assert!(
            items.len() <= capacity,
            "ItemGrid cannot hold {} items in {} slots",
            items.len(),
            capacity
        );
        Self {
            columns,
            rows,
            items,
        }
    }

    pub(crate) fn player_starting(items: Vec<Item>) -> Self {
        Self::new(STARTING_BAG_COLUMNS, STARTING_BAG_ROWS, items)
    }

    pub(crate) fn stash_starting() -> Self {
        Self::new(STARTING_STASH_COLUMNS, STARTING_STASH_ROWS, Vec::new())
    }

    pub(crate) fn capacity(&self) -> usize {
        usize::from(self.columns) * usize::from(self.rows)
    }

    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(crate) fn has_space(&self) -> bool {
        self.len() < self.capacity()
    }

    #[allow(dead_code)]
    pub(crate) fn available_slots(&self) -> usize {
        self.capacity().saturating_sub(self.len())
    }

    pub(crate) fn push(&mut self, item: Item) -> bool {
        if self.has_space() {
            self.items.push(item);
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub(crate) fn try_push(&mut self, item: Item) -> Result<usize, Item> {
        if self.has_space() {
            self.items.push(item);
            Ok(self.items.len() - 1)
        } else {
            Err(item)
        }
    }

    pub(crate) fn remove(&mut self, index: usize) -> Item {
        self.items.remove(index)
    }

    pub(crate) fn insert(&mut self, index: usize, item: Item) -> bool {
        if self.has_space() && index <= self.items.len() {
            self.items.insert(index, item);
            true
        } else {
            false
        }
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, Item> {
        self.items.iter()
    }

    pub(crate) fn as_slice(&self) -> &[Item] {
        &self.items
    }

    #[allow(dead_code)]
    pub(crate) fn clear(&mut self) {
        self.items.clear();
    }

    #[allow(dead_code)]
    pub(crate) fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Item) -> bool,
    {
        self.items.retain(f);
    }
}

impl std::ops::Index<usize> for ItemGrid {
    type Output = Item;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl std::ops::IndexMut<usize> for ItemGrid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
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
    Gem,
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
pub(crate) struct GroundItem {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) item: Item,
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
    #[serde(default)]
    pub(crate) ground_items: Vec<GroundItem>,
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
    pub(crate) inventory: ItemGrid,
    pub(crate) stash: ItemGrid,
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
    pub(crate) completed_town_projects: Vec<TownProject>,
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
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) pending_town_message: String,
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
            inventory: ItemGrid::player_starting(vec![
                health_potion(),
                health_potion(),
                mana_potion(),
            ]),
            stash: ItemGrid::stash_starting(),
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
            completed_town_projects: Vec::new(),
            cleave_mastery: None,
            shield_bash_mastery: None,
            battle_cry_mastery: None,
            deep_cut_mastery: None,
            iron_guard_mastery: None,
            second_wind_mastery: None,
            second_wind_shield: 0,
            pending_town_message: String::new(),
        }
    }

    pub(crate) fn max_hp(&self) -> u32 {
        let bonuses = self.socket_bonuses();
        10 + self.effective_strength() * 5 + bonuses.max_hp
    }
    pub(crate) fn max_mana(&self) -> u32 {
        let bonuses = self.socket_bonuses();
        10 + self.effective_intelligence() * 5 + bonuses.max_mana
    }
    pub(crate) fn hit_rating(&self) -> u32 {
        let bonuses = self.socket_bonuses();
        10 + self.effective_dexterity() * 5 + bonuses.hit_rating
    }
    pub(crate) fn dodge_rating(&self) -> u32 {
        let bonuses = self.socket_bonuses();
        let mastery_bonus = if self.iron_guard_mastery == Some(SkillMastery::ShieldDiscipline) {
            3
        } else {
            0
        };
        (10 + self.effective_dexterity() as i32 * 3
            + self.equipped_shield.dodge
            + self.equipped_armor.dodge
            + bonuses.dodge_rating
            + mastery_bonus)
            .max(0) as u32
    }
    pub(crate) fn speed(&self) -> u32 {
        let bonuses = self.socket_bonuses();
        (10 + self.effective_dexterity() as i32 * 5
            + self.equipped_weapon.speed
            + self.equipped_armor.speed
            + self.equipped_shield.speed
            + bonuses.speed)
            .max(1) as u32
    }
    pub(crate) fn armor(&self) -> i32 {
        let bonuses = self.socket_bonuses();
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
            + bonuses.armor
    }
    pub(crate) fn weapon_damage(&self) -> (i32, i32) {
        let bonuses = self.socket_bonuses();
        (
            self.equipped_weapon.damage_min
                + (self.effective_strength() as i32 / 4)
                + bonuses.weapon_damage,
            self.equipped_weapon.damage_max
                + (self.effective_strength() as i32 / 3)
                + bonuses.weapon_damage,
        )
    }
    pub(crate) fn effective_strength(&self) -> u32 {
        self.strength + self.socket_bonuses().strength
    }
    pub(crate) fn effective_dexterity(&self) -> u32 {
        self.dexterity + self.socket_bonuses().dexterity
    }
    pub(crate) fn effective_intelligence(&self) -> u32 {
        self.intelligence + self.socket_bonuses().intelligence
    }
    #[allow(dead_code)]
    pub(crate) fn weapon_crit_chance(&self) -> u32 {
        self.equipped_weapon
            .crit_chance
            .saturating_add(self.socket_bonuses().crit_chance)
            .min(100)
    }
    pub(crate) fn socket_bonuses(&self) -> GemBonuses {
        self.equipped_weapon
            .socket_bonuses()
            .add(self.equipped_armor.socket_bonuses())
            .add(self.equipped_shield.socket_bonuses())
    }
}

impl Item {
    pub(crate) fn socket_bonuses(&self) -> GemBonuses {
        self.sockets
            .iter()
            .flatten()
            .fold(GemBonuses::default(), |bonuses, socket| {
                bonuses.add(gem_bonus(socket.gem_kind, socket.gem_tier))
            })
    }
}
