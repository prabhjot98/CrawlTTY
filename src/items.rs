#[derive(Clone, Copy)]
struct ItemStats {
    damage_min: i32,
    damage_max: i32,
    armor: i32,
    dodge: i32,
    speed: i32,
}

#[derive(Clone, Copy)]
struct Requirements {
    strength: u32,
    dexterity: u32,
    intelligence: u32,
}

fn item_stats(damage_min: i32, damage_max: i32, armor: i32, dodge: i32, speed: i32) -> ItemStats {
    ItemStats {
        damage_min,
        damage_max,
        armor,
        dodge,
        speed,
    }
}

fn requirements(strength: u32, dexterity: u32, intelligence: u32) -> Requirements {
    Requirements {
        strength,
        dexterity,
        intelligence,
    }
}

fn item(name: &str, kind: ItemKind, value: u32, stats: ItemStats) -> Item {
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
        rarity,
        item_level,
        required_strength: requirements.strength,
        required_dexterity: requirements.dexterity,
        required_intelligence: requirements.intelligence,
        upgrade_level: 0,
    }
}
fn health_potion() -> Item {
    item(
        "Lesser Health Potion (restores 15% HP)",
        ItemKind::HealthPotion,
        HEALTH_POTION_COST,
        item_stats(0, 0, 0, 0, 0),
    )
}
fn mana_potion() -> Item {
    item(
        "Lesser Mana Potion (restores 15% mana)",
        ItemKind::ManaPotion,
        MANA_POTION_COST,
        item_stats(0, 0, 0, 0, 0),
    )
}
fn rusted_sword() -> Item {
    item(
        "Rusted Sword (3-5 dmg, STR F, DEX F)",
        ItemKind::Weapon,
        20,
        item_stats(3, 5, 0, 0, 0),
    )
}
fn crude_axe() -> Item {
    item(
        "Crude Axe (4-6 dmg, STR F)",
        ItemKind::Weapon,
        60,
        item_stats(4, 6, 0, 0, -1),
    )
}
fn cloth_tunic() -> Item {
    item(
        "Cloth Tunic (+1 armor)",
        ItemKind::Armor,
        12,
        item_stats(0, 0, 1, 0, 0),
    )
}
fn battered_mail() -> Item {
    item(
        "Battered Mail (+2 armor, -5 speed)",
        ItemKind::Armor,
        55,
        item_stats(0, 0, 2, 0, -5),
    )
}
fn worn_shield() -> Item {
    item(
        "Worn Shield (+1 armor, +2 dodge)",
        ItemKind::Shield,
        40,
        item_stats(0, 0, 1, 2, 0),
    )
}

