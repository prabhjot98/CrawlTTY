use crate::ItemKind;

pub(crate) const PLAYER_GLYPH: char = '☥';
pub(crate) const WALL_GLYPH: char = '▓';
pub(crate) const FLOOR_GLYPH: char = '·';
pub(crate) const STAIRS_DOWN_GLYPH: char = '⌄';
pub(crate) const CHEST_GLYPH: char = '◈';
pub(crate) const LOOT_GLYPH: char = '✦';
pub(crate) const BELL_WAVE_GLYPH: char = '✶';

pub(crate) const RAT_GLYPH: char = 'ʀ';
pub(crate) const SKELETON_GLYPH: char = '♰';
pub(crate) const CULTIST_GLYPH: char = '☽';
pub(crate) const BONEGUARD_GLYPH: char = '♜';
pub(crate) const ELITE_GLYPH: char = '◆';
pub(crate) const BELLKEEPER_GLYPH: char = 'Ω';
pub(crate) const DUNE_STALKER_GLYPH: char = 'ϟ';
pub(crate) const GLASS_WRAITH_GLYPH: char = '◇';
pub(crate) const EMBER_MAGUS_GLYPH: char = 'Δ';
pub(crate) const OBSIDIAN_GUARD_GLYPH: char = '◼';
pub(crate) const GLASS_TYRANT_GLYPH: char = 'Ψ';

pub(crate) const LOCKED_MARKER: &str = "⊘";
pub(crate) const ACTIVE_MARKER: &str = "✦";
pub(crate) const SELECTION_CURSOR: &str = "›";
pub(crate) const SELECTION_CURSOR_PREFIX: &str = "› ";
pub(crate) const TREE_CHILD: &str = "└─";
pub(crate) const PREVIOUS_LOG_DIVIDER: &str = "── Previous ──";

pub(crate) const GRID_OPEN_GLYPH: &str = "⟦";
pub(crate) const GRID_CLOSE_GLYPH: &str = "⟧";
pub(crate) const EMPTY_CELL_GLYPH: &str = "·";
pub(crate) const HEALTH_POTION_GLYPH: &str = "✚";
pub(crate) const MANA_POTION_GLYPH: &str = "✧";
pub(crate) const WEAPON_GLYPH: &str = "†";
pub(crate) const ARMOR_GLYPH: &str = "▣";
pub(crate) const SHIELD_GLYPH: &str = "◖";
pub(crate) const HELM_GLYPH: &str = "⌂";
pub(crate) const GLOVES_GLYPH: &str = "∩";
pub(crate) const BOOTS_GLYPH: &str = "∪";
pub(crate) const BELT_GLYPH: &str = "═";
pub(crate) const AMULET_GLYPH: &str = "◇";
pub(crate) const RING_GLYPH: &str = "○";
pub(crate) const GEM_GLYPH: &str = "◆";

pub(crate) fn dungeon_display_glyph(symbol: char) -> char {
    match symbol {
        '@' => PLAYER_GLYPH,
        '#' => WALL_GLYPH,
        '.' => FLOOR_GLYPH,
        '>' => STAIRS_DOWN_GLYPH,
        '$' => CHEST_GLYPH,
        '!' => LOOT_GLYPH,
        '*' => BELL_WAVE_GLYPH,
        'r' => RAT_GLYPH,
        's' => SKELETON_GLYPH,
        'c' => CULTIST_GLYPH,
        'b' => BONEGUARD_GLYPH,
        'E' => ELITE_GLYPH,
        'B' => BELLKEEPER_GLYPH,
        'g' => DUNE_STALKER_GLYPH,
        'w' => GLASS_WRAITH_GLYPH,
        'm' => EMBER_MAGUS_GLYPH,
        'o' => OBSIDIAN_GUARD_GLYPH,
        'T' => GLASS_TYRANT_GLYPH,
        other => other,
    }
}

pub(crate) fn item_kind_glyph(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::HealthPotion => HEALTH_POTION_GLYPH,
        ItemKind::ManaPotion => MANA_POTION_GLYPH,
        ItemKind::Weapon => WEAPON_GLYPH,
        ItemKind::Armor => ARMOR_GLYPH,
        ItemKind::Shield => SHIELD_GLYPH,
        ItemKind::Helm => HELM_GLYPH,
        ItemKind::Gloves => GLOVES_GLYPH,
        ItemKind::Boots => BOOTS_GLYPH,
        ItemKind::Belt => BELT_GLYPH,
        ItemKind::Amulet => AMULET_GLYPH,
        ItemKind::Ring => RING_GLYPH,
        ItemKind::Gem => GEM_GLYPH,
    }
}

#[cfg(test)]
pub(crate) fn unicode_width_samples() -> &'static [(&'static str, &'static str, usize)] {
    &[
        ("player", "☥", 1),
        ("wall", "▓", 1),
        ("floor", "·", 1),
        ("stairs down", "⌄", 1),
        ("chest", "◈", 1),
        ("loot", "✦", 1),
        ("bell wave", "✶", 1),
        ("rat", "ʀ", 1),
        ("skeleton", "♰", 1),
        ("cultist", "☽", 1),
        ("boneguard", "♜", 1),
        ("elite", "◆", 1),
        ("bellkeeper", "Ω", 1),
        ("dune stalker", "ϟ", 1),
        ("glass wraith", "◇", 1),
        ("ember magus", "Δ", 1),
        ("obsidian guard", "◼", 1),
        ("glass tyrant", "Ψ", 1),
        ("locked marker", LOCKED_MARKER, 1),
        ("active marker", ACTIVE_MARKER, 1),
        ("selection cursor", SELECTION_CURSOR, 1),
        ("selection cursor prefix", SELECTION_CURSOR_PREFIX, 2),
        ("tree child connector", TREE_CHILD, 2),
        ("previous log divider", PREVIOUS_LOG_DIVIDER, 14),
        ("grid open", GRID_OPEN_GLYPH, 1),
        ("grid close", GRID_CLOSE_GLYPH, 1),
        ("empty cell", EMPTY_CELL_GLYPH, 1),
        ("health potion", HEALTH_POTION_GLYPH, 1),
        ("mana potion", MANA_POTION_GLYPH, 1),
        ("weapon", WEAPON_GLYPH, 1),
        ("armor", ARMOR_GLYPH, 1),
        ("shield", SHIELD_GLYPH, 1),
        ("helm", HELM_GLYPH, 1),
        ("gloves", GLOVES_GLYPH, 1),
        ("boots", BOOTS_GLYPH, 1),
        ("belt", BELT_GLYPH, 1),
        ("amulet", AMULET_GLYPH, 1),
        ("ring", RING_GLYPH, 1),
        ("gem", GEM_GLYPH, 1),
    ]
}
