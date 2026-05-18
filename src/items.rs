use crate::*;

pub(crate) const SWORD_CRIT_CHANCE: u32 = 8;
pub(crate) const AXE_CRIT_CHANCE: u32 = 5;
pub(crate) const DAGGER_CRIT_CHANCE: u32 = 14;
pub(crate) const SCIMITAR_CRIT_CHANCE: u32 = 10;
pub(crate) const WAND_CRIT_CHANCE: u32 = 4;

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
        ItemKind::Armor
        | ItemKind::Shield
        | ItemKind::Helm
        | ItemKind::Gloves
        | ItemKind::Boots
        | ItemKind::Belt
        | ItemKind::Amulet
        | ItemKind::Ring => (stats.armor + 3).max(0) as u32,
        ItemKind::HealthPotion | ItemKind::ManaPotion | ItemKind::Gem => 0,
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
        crit_chance: stats.crit_chance,
        rarity: Rarity::Common,
        item_level: 1,
        required_strength,
        required_dexterity,
        required_intelligence: 0,
        upgrade_level: 0,
        sockets: Vec::new(),
        gem_kind: None,
        gem_tier: None,
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
        crit_chance: stats.crit_chance,
        rarity,
        item_level,
        required_strength: requirements.strength,
        required_dexterity: requirements.dexterity,
        required_intelligence: requirements.intelligence,
        upgrade_level: 0,
        sockets: Vec::new(),
        gem_kind: None,
        gem_tier: None,
    }
}

#[allow(dead_code)]
pub(crate) fn gem_item(kind: GemKind, tier: GemTier) -> Item {
    let bonus = gem_bonus(kind, tier);
    let tier_name = gem_tier_name(tier);
    let kind_name = gem_kind_name(kind);
    let value = match tier {
        GemTier::Chipped => 25,
        GemTier::Flawed => 75,
        GemTier::Pristine => 200,
    };

    Item {
        name: format!("{tier_name} {kind_name} ({})", gem_bonus_text(bonus)),
        kind: ItemKind::Gem,
        value,
        damage_min: 0,
        damage_max: 0,
        armor: 0,
        dodge: 0,
        speed: 0,
        crit_chance: 0,
        rarity: Rarity::Common,
        item_level: 1,
        required_strength: 0,
        required_dexterity: 0,
        required_intelligence: 0,
        upgrade_level: 0,
        sockets: Vec::new(),
        gem_kind: Some(kind),
        gem_tier: Some(tier),
    }
}

#[allow(dead_code)]
pub(crate) fn gem_kind_name(kind: GemKind) -> &'static str {
    match kind {
        GemKind::Ruby => "Ruby",
        GemKind::Sapphire => "Sapphire",
        GemKind::Garnet => "Garnet",
        GemKind::Emerald => "Emerald",
        GemKind::Amethyst => "Amethyst",
        GemKind::Quartz => "Quartz",
        GemKind::Jade => "Jade",
        GemKind::Onyx => "Onyx",
        GemKind::Citrine => "Citrine",
        GemKind::Topaz => "Topaz",
        GemKind::Opal => "Opal",
        GemKind::Bloodstone => "Bloodstone",
    }
}

#[allow(dead_code)]
pub(crate) fn gem_tier_name(tier: GemTier) -> &'static str {
    match tier {
        GemTier::Chipped => "Chipped",
        GemTier::Flawed => "Flawed",
        GemTier::Pristine => "Pristine",
    }
}

pub(crate) fn gem_bonus_text(bonus: GemBonuses) -> String {
    if bonus.max_hp > 0 {
        format!("+{} max HP", bonus.max_hp)
    } else if bonus.max_mana > 0 {
        format!("+{} max mana", bonus.max_mana)
    } else if bonus.strength > 0 {
        format!("+{} strength", bonus.strength)
    } else if bonus.dexterity > 0 {
        format!("+{} dexterity", bonus.dexterity)
    } else if bonus.intelligence > 0 {
        format!("+{} intelligence", bonus.intelligence)
    } else if bonus.hit_rating > 0 {
        format!("+{} hit rating", bonus.hit_rating)
    } else if bonus.dodge_rating > 0 {
        format!("+{} dodge rating", bonus.dodge_rating)
    } else if bonus.armor > 0 {
        format!("+{} armor", bonus.armor)
    } else if bonus.speed > 0 {
        format!("+{} speed", bonus.speed)
    } else if bonus.crit_chance > 0 {
        format!("+{}% crit chance", bonus.crit_chance)
    } else if bonus.gold_found_percent > 0 {
        format!("+{}% gold found", bonus.gold_found_percent)
    } else {
        format!("+{} weapon damage", bonus.weapon_damage)
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
        "Rusted Sword (3-5 dmg)",
        ItemKind::Weapon,
        20,
        weapon_stats(3, 5, 0, SWORD_CRIT_CHANCE),
    )
}
pub(crate) fn training_dagger() -> Item {
    item_with_rarity(
        "Training Dagger (2-4 dmg)",
        ItemKind::Weapon,
        20,
        weapon_stats(2, 4, 1, DAGGER_CRIT_CHANCE),
        Rarity::Common,
        1,
        requirements(0, 2, 0),
    )
}

pub(crate) fn cracked_wand() -> Item {
    item_with_rarity(
        "Cracked Wand (2-3 spell)",
        ItemKind::Weapon,
        20,
        weapon_stats(2, 3, 0, WAND_CRIT_CHANCE),
        Rarity::Common,
        1,
        requirements(0, 0, 2),
    )
}

pub(crate) fn cracked_focus() -> Item {
    item_with_rarity(
        "Cracked Focus (+2 dodge)",
        ItemKind::Shield,
        20,
        item_stats(0, 0, 0, 2, 0),
        Rarity::Common,
        1,
        requirements(0, 0, 2),
    )
}

pub(crate) fn frayed_robe() -> Item {
    item_with_rarity(
        "Frayed Robe (+1 armor, +1 dodge)",
        ItemKind::Armor,
        16,
        item_stats(0, 0, 1, 1, 0),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    )
}
#[cfg(test)]
pub(crate) fn crude_axe() -> Item {
    item(
        "Crude Axe (4-6 dmg)",
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
pub(crate) fn patched_leathers() -> Item {
    item_with_rarity(
        "Patched Leathers (+1 armor, +2 dodge)",
        ItemKind::Armor,
        18,
        item_stats(0, 0, 1, 2, 1),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
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
pub(crate) const NOTHING_EQUIPPED_TEXT: &str = "Nothing equipped";

pub(crate) fn is_empty_equipment_slot(item: &Item) -> bool {
    matches!(
        (item.kind, item.name.as_str()),
        (ItemKind::Shield, "Empty Offhand")
            | (ItemKind::Helm, "Empty Helm")
            | (ItemKind::Gloves, "Empty Gloves")
            | (ItemKind::Boots, "Empty Boots")
            | (ItemKind::Belt, "Empty Belt")
            | (ItemKind::Amulet, "Empty Amulet")
            | (ItemKind::Ring, "Empty Ring")
    )
}

pub(crate) fn equipped_display_name(item: &Item) -> String {
    if is_empty_equipment_slot(item) {
        NOTHING_EQUIPPED_TEXT.to_string()
    } else {
        item.name.clone()
    }
}

pub(crate) fn empty_offhand() -> Item {
    empty_slot("Empty Offhand", ItemKind::Shield)
}

pub(crate) fn empty_helm() -> Item {
    empty_slot("Empty Helm", ItemKind::Helm)
}

pub(crate) fn empty_gloves() -> Item {
    empty_slot("Empty Gloves", ItemKind::Gloves)
}

pub(crate) fn empty_boots() -> Item {
    empty_slot("Empty Boots", ItemKind::Boots)
}

pub(crate) fn empty_belt() -> Item {
    empty_slot("Empty Belt", ItemKind::Belt)
}

pub(crate) fn empty_amulet() -> Item {
    empty_slot("Empty Amulet", ItemKind::Amulet)
}

pub(crate) fn empty_ring() -> Item {
    empty_slot("Empty Ring", ItemKind::Ring)
}

fn empty_slot(name: &str, kind: ItemKind) -> Item {
    item_with_rarity(
        name,
        kind,
        0,
        item_stats(0, 0, 0, 0, 0),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    )
}
