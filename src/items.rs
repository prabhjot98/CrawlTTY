use crate::*;

pub(crate) const SWORD_CRIT_CHANCE: u32 = 8;
pub(crate) const AXE_CRIT_CHANCE: u32 = 5;

#[derive(Clone, Copy)]
pub(crate) struct ItemStats {
    pub(crate) damage_min: i32,
    pub(crate) damage_max: i32,
    pub(crate) armor: i32,
    pub(crate) dodge: i32,
    pub(crate) speed: i32,
    pub(crate) crit_chance: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct Requirements {
    pub(crate) strength: u32,
    pub(crate) dexterity: u32,
    pub(crate) intelligence: u32,
}

pub(crate) fn item_stats(
    damage_min: i32,
    damage_max: i32,
    armor: i32,
    dodge: i32,
    speed: i32,
) -> ItemStats {
    ItemStats {
        damage_min,
        damage_max,
        armor,
        dodge,
        speed,
        crit_chance: 0,
    }
}

pub(crate) fn weapon_stats(
    damage_min: i32,
    damage_max: i32,
    speed: i32,
    crit_chance: u32,
) -> ItemStats {
    ItemStats {
        damage_min,
        damage_max,
        armor: 0,
        dodge: 0,
        speed,
        crit_chance,
    }
}

fn base_weapon_crit_chance(name: &str, kind: ItemKind) -> u32 {
    if kind != ItemKind::Weapon {
        0
    } else if name.contains("Sword") {
        SWORD_CRIT_CHANCE
    } else if name.contains("Axe") {
        AXE_CRIT_CHANCE
    } else {
        0
    }
}

pub(crate) fn requirements(strength: u32, dexterity: u32, intelligence: u32) -> Requirements {
    Requirements {
        strength,
        dexterity,
        intelligence,
    }
}

pub(crate) fn item(name: &str, kind: ItemKind, value: u32, stats: ItemStats) -> Item {
    let required_strength = match kind {
        ItemKind::Weapon => stats.damage_max.max(0) as u32,
        ItemKind::Armor | ItemKind::Shield => (stats.armor + 3).max(0) as u32,
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
        damage_min: stats.damage_min,
        damage_max: stats.damage_max,
        armor: stats.armor,
        dodge: stats.dodge,
        speed: stats.speed,
        crit_chance: stats.crit_chance.max(base_weapon_crit_chance(name, kind)),
        rarity: Rarity::Common,
        item_level: 1,
        required_strength,
        required_dexterity,
        required_intelligence: 0,
        upgrade_level: 0,
    }
}

pub(crate) fn item_with_rarity(
    name: &str,
    kind: ItemKind,
    value: u32,
    stats: ItemStats,
    rarity: Rarity,
    item_level: u32,
    requirements: Requirements,
) -> Item {
    Item {
        name: name.to_string(),
        kind,
        value,
        damage_min: stats.damage_min,
        damage_max: stats.damage_max,
        armor: stats.armor,
        dodge: stats.dodge,
        speed: stats.speed,
        crit_chance: stats.crit_chance.max(base_weapon_crit_chance(name, kind)),
        rarity,
        item_level,
        required_strength: requirements.strength,
        required_dexterity: requirements.dexterity,
        required_intelligence: requirements.intelligence,
        upgrade_level: 0,
    }
}
pub(crate) fn health_potion() -> Item {
    item(
        "Lesser Health Potion (restores 15% HP)",
        ItemKind::HealthPotion,
        HEALTH_POTION_COST,
        item_stats(0, 0, 0, 0, 0),
    )
}
pub(crate) fn mana_potion() -> Item {
    item(
        "Lesser Mana Potion (restores 15% mana)",
        ItemKind::ManaPotion,
        MANA_POTION_COST,
        item_stats(0, 0, 0, 0, 0),
    )
}
pub(crate) fn rusted_sword() -> Item {
    item(
        "Rusted Sword (3-5 dmg, STR F, DEX F)",
        ItemKind::Weapon,
        20,
        weapon_stats(3, 5, 0, SWORD_CRIT_CHANCE),
    )
}
pub(crate) fn crude_axe() -> Item {
    item(
        "Crude Axe (4-6 dmg, STR F)",
        ItemKind::Weapon,
        60,
        weapon_stats(4, 6, -1, AXE_CRIT_CHANCE),
    )
}
pub(crate) fn cloth_tunic() -> Item {
    item(
        "Cloth Tunic (+1 armor)",
        ItemKind::Armor,
        12,
        item_stats(0, 0, 1, 0, 0),
    )
}
pub(crate) fn battered_mail() -> Item {
    item(
        "Battered Mail (+2 armor, -5 speed)",
        ItemKind::Armor,
        55,
        item_stats(0, 0, 2, 0, -5),
    )
}
pub(crate) fn worn_shield() -> Item {
    item(
        "Worn Shield (+1 armor, +2 dodge)",
        ItemKind::Shield,
        40,
        item_stats(0, 0, 1, 2, 0),
    )
}
