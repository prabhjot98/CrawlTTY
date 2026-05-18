use super::*;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[test]
fn unicode_visual_palette_uses_single_cell_non_emoji_glyphs() {
    for (name, text, expected_width) in unicode_width_samples() {
        assert_eq!(
            UnicodeWidthStr::width(*text),
            *expected_width,
            "{name} should occupy {expected_width} terminal cell(s): {text:?}"
        );
        for ch in text.chars() {
            assert_eq!(
                UnicodeWidthChar::width(ch),
                Some(1),
                "{name} contains a non-single-cell glyph: {ch:?}"
            );
            assert!(
                !(('\u{fe00}'..='\u{fe0f}').contains(&ch)),
                "{name} should avoid variation selectors that can trigger emoji presentation"
            );
            assert!(
                u32::from(ch) < 0x1f000,
                "{name} should avoid emoji-plane glyphs that often render double-width"
            );
        }
    }
}

#[test]
fn player_glyph_is_classic_at_sign() {
    assert_eq!(PLAYER_GLYPH, '@');
    assert_eq!(dungeon_display_glyph('@'), '@');
}

#[test]
fn map_enemy_display_glyphs_remain_letters() {
    for glyph in ['r', 's', 'c', 'b', 'E', 'B', 'g', 'w', 'm', 'o', 'T'] {
        assert_eq!(dungeon_display_glyph(glyph), glyph);
    }
    assert_eq!(dungeon_display_glyph('@'), PLAYER_GLYPH);
    assert_eq!(dungeon_display_glyph('#'), WALL_GLYPH);
    assert_eq!(dungeon_display_glyph('!'), LOOT_GLYPH);
}

#[test]
fn inventory_item_glyphs_are_letters() {
    assert_eq!(item_kind_glyph(ItemKind::HealthPotion), "H");
    assert_eq!(item_kind_glyph(ItemKind::ManaPotion), "M");
    assert_eq!(item_kind_glyph(ItemKind::Weapon), "W");
    assert_eq!(item_kind_glyph(ItemKind::Armor), "A");
    assert_eq!(item_kind_glyph(ItemKind::Shield), "S");
    assert_eq!(item_kind_glyph(ItemKind::Helm), "H");
    assert_eq!(item_kind_glyph(ItemKind::Gloves), "G");
    assert_eq!(item_kind_glyph(ItemKind::Boots), "B");
    assert_eq!(item_kind_glyph(ItemKind::Belt), "T");
    assert_eq!(item_kind_glyph(ItemKind::Amulet), "U");
    assert_eq!(item_kind_glyph(ItemKind::Ring), "R");
    assert_eq!(item_kind_glyph(ItemKind::Gem), "G");
}

fn test_character() -> Character {
    Character::new(
        "Tester".to_string(),
        CharacterClass::Warrior,
        DeathMode::Softcore,
    )
}

fn critical_combat_test_character() -> Character {
    let mut c = test_character();
    c.strength = 0;
    c.equipped_weapon.damage_min = 10;
    c.equipped_weapon.damage_max = 10;
    c.equipped_weapon.crit_chance = 100;
    c
}

fn line_text(line: &ratatui::text::Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn armored_training_dummy(x: i32, y: i32) -> Enemy {
    let mut enemy = skeleton(x, y);
    enemy.name = "Armored Dummy".to_string();
    enemy.hp = 30;
    enemy.max_hp = 30;
    enemy.armor = 3;
    enemy
}

fn one_hp_test_boss(x: i32, y: i32) -> Enemy {
    enemy(
        "Test Boss",
        'B',
        x,
        y,
        enemy_stats(1, 0, 0, 0, 10),
        enemy_rewards(10, 1, 1),
        true,
    )
}

fn open_test_dungeon(player_x: i32, player_y: i32, enemies: Vec<Enemy>) -> Dungeon {
    let mut tiles = vec!['.'; (MAP_W * MAP_H) as usize];
    for x in 0..MAP_W {
        tiles[tile_index(x, 0)] = '#';
        tiles[tile_index(x, MAP_H - 1)] = '#';
    }
    for y in 0..MAP_H {
        tiles[tile_index(0, y)] = '#';
        tiles[tile_index(MAP_W - 1, y)] = '#';
    }

    Dungeon {
        floor: 2,
        player_x,
        player_y,
        stairs_x: MAP_W - 2,
        stairs_y: MAP_H - 2,
        enemies,
        chests: Vec::new(),
        ground_items: Vec::new(),
        log: Vec::new(),
        tiles,
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
    assert_eq!(d.log[1], "[WARN] No adjacent enemies for Cleave.");
}

#[test]
fn dungeon_action_label_names_inventory_commands() {
    assert_eq!(dungeon_action_label('i'), "Inventory");
    assert_eq!(dungeon_action_label('I'), "Inventory");
}

#[test]
fn readme_lists_distillery_town_control() {
    let readme = include_str!("../README.md");

    assert!(readme.contains("- `l` distillery"));
}

#[test]
fn readme_lists_help_control() {
    let readme = include_str!("../README.md");

    assert!(readme.contains("`h` help"));
}

#[test]
fn class_resource_labels_match_active_class() {
    let warrior = Character::new(
        "War".to_string(),
        CharacterClass::Warrior,
        DeathMode::Softcore,
    );
    let rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );

    assert_eq!(warrior.resource_label(), "Mana");
    assert_eq!(rogue.resource_label(), "Energy");
    assert_eq!(rogue.current_resource(), ROGUE_MAX_ENERGY);
    assert_eq!(rogue.max_resource(), ROGUE_MAX_ENERGY);
}

#[test]
fn rogue_energy_restore_clamps_without_overflow() {
    let mut rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    rogue.rogue.energy = ROGUE_MAX_ENERGY - 1;

    rogue.restore_rogue_energy(u32::MAX);

    assert_eq!(rogue.rogue.energy, ROGUE_MAX_ENERGY);
}

#[test]
fn rogue_dungeon_action_labels_include_four_active_skills() {
    let rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );

    assert_eq!(dungeon_action_label_for(&rogue, '1'), "Backstab");
    assert_eq!(dungeon_action_label_for(&rogue, '2'), "Venom Edge");
    assert_eq!(dungeon_action_label_for(&rogue, '3'), "Eviscerate");
    assert_eq!(dungeon_action_label_for(&rogue, '4'), "Smoke Step");
}

#[test]
fn sorceress_dungeon_action_labels_include_spell_hotkeys() {
    let sorceress = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );

    assert_eq!(dungeon_action_label_for(&sorceress, '1'), "Firebolt");
    assert_eq!(dungeon_action_label_for(&sorceress, '2'), "Frost Ring");
    assert_eq!(dungeon_action_label_for(&sorceress, '3'), "Chain Spark");
    assert_eq!(dungeon_action_label_for(&sorceress, '4'), "Mana Shield");
    assert!(is_known_dungeon_command_for(&sorceress, '4'));
}

#[test]
fn rogue_skill_help_lines_show_energy_combo_points_and_four_skills() {
    let mut rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    rogue.rogue.energy = 45;
    rogue.rogue.smoke_step_cooldown = 2;

    let rendered = dungeon_skill_help_lines(&rogue)
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rendered.contains("Rogue: Energy 45/100  CP 0/5"));
    assert!(
        rendered.contains("1 Backstab r1: cost 25 Energy. Build 1 CP; 90% damage, 120% empowered.")
    );
    assert!(rendered.contains(
        "2 Venom Edge r1: cost 30 Energy. 70% damage; build 1 CP and poison 2/turn for 3 turns."
    ));
    assert!(rendered.contains("3 Eviscerate r0: cost 35 Energy. Spend CP for burst damage +0%."));
    assert!(
        rendered.contains(
            "4 Smoke Step r1: cost 35 Energy, cd 4. Then WASD=1 tile, Shift+WASD=2. +20 dodge. Ready in 2."
        )
    );
}

#[test]
fn sorceress_skill_help_lines_show_mana_cooldowns_and_starting_mana_shield() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.mana = 27;
    c.sorceress.frost_ring_cooldown = 2;
    c.sorceress.chain_spark_cooldown = 1;

    let rendered = dungeon_skill_help_lines(&c)
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rendered.contains("Sorceress: Mana 27/40  Mana Shield off"));
    assert!(rendered.contains("1 Firebolt r1: cost 4 mana. 100% spell damage; 25% Burning."));
    assert!(rendered.contains(
        "2 Frost Ring r1: cost 8 mana, cd 3. 8 tiles; 70% damage; 20% Freeze. Ready in 2."
    ));
    assert!(
        rendered
            .contains("3 Chain Spark r1: cost 7 mana, cd 2. 80% damage; up to 2 hits. Ready in 1.")
    );
    assert!(rendered.contains("4 Mana Shield r1: free toggle. Absorbs 50% at 1 mana per damage."));
}

#[test]
fn sorceress_unlearned_mana_shield_help_shows_skill_point_prompt() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.mana_shield_rank = 0;

    let rendered = dungeon_skill_help_lines(&c)
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rendered.contains("4 Mana Shield: unlearned; spend a skill point to learn it."));
    assert!(!rendered.contains("4 Mana Shield: locked; requires Frost Ring rank 2."));
}

#[test]
fn sorceress_unlocked_mana_shield_help_shows_absorption_and_state() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.mana_shield_rank = 3;
    c.sorceress.mana_shield_active = true;

    let rendered = dungeon_skill_help_lines(&c)
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rendered.contains("Sorceress: Mana 40/40  Mana Shield on"));
    assert!(rendered.contains("4 Mana Shield r3: free toggle. Absorbs 60% at 1 mana per damage."));
}

#[test]
fn sorceress_cooldowns_tick_and_clear_with_combat_state() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.frost_ring_cooldown = 2;
    c.sorceress.chain_spark_cooldown = 1;
    c.sorceress.mana_shield_rank = 1;
    c.sorceress.mana_shield_active = true;

    tick_player_effects(&mut c);

    assert_eq!(c.sorceress.frost_ring_cooldown, 1);
    assert_eq!(c.sorceress.chain_spark_cooldown, 0);
    assert!(c.sorceress.mana_shield_active);

    clear_combat_state(&mut c);

    assert_eq!(c.sorceress.frost_ring_cooldown, 0);
    assert_eq!(c.sorceress.chain_spark_cooldown, 0);
    assert!(!c.sorceress.mana_shield_active);
}

#[test]
fn mana_shield_absorbs_rank_scaled_damage_using_mana() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.mana_shield_rank = 1;
    c.sorceress.mana_shield_active = true;
    c.hp = 15;
    c.mana = 40;

    apply_player_damage(&mut c, 10);

    assert_eq!(c.mana, 35);
    assert_eq!(c.hp, 10);
    assert!(c.sorceress.mana_shield_active);

    c.sorceress.mana_shield_rank = 5;
    c.sorceress.mana_shield_active = true;
    c.hp = 15;
    c.mana = 2;

    apply_player_damage(&mut c, 10);

    assert_eq!(c.mana, 0);
    assert_eq!(c.hp, 7);
    assert!(!c.sorceress.mana_shield_active);
}

#[test]
fn mana_shield_absorbs_at_least_one_damage_from_small_hits() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.mana_shield_rank = 1;
    c.sorceress.mana_shield_active = true;
    c.hp = 15;
    c.mana = 40;

    apply_player_damage(&mut c, 1);

    assert_eq!(c.mana, 39);
    assert_eq!(c.hp, 15);
    assert!(c.sorceress.mana_shield_active);
}

#[test]
fn burning_and_frozen_enemy_effects_tick_during_enemy_turns() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    let mut burning = skeleton(5, 5);
    burning.name = "Burning Dummy".to_string();
    burning.hp = 10;
    burning.max_hp = 10;
    burning.burning_turns = 1;
    burning.burning_damage = 2;
    let mut frozen = skeleton(3, 2);
    frozen.name = "Frozen Dummy".to_string();
    frozen.frozen_turns = 1;
    frozen.energy = 999;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![burning, frozen]));
    let before_hp = c.hp;

    enemy_turns(&mut c);

    let d = c.active_dungeon.as_ref().unwrap();
    let burning = d
        .enemies
        .iter()
        .find(|enemy| enemy.name == "Burning Dummy")
        .unwrap();
    let frozen = d
        .enemies
        .iter()
        .find(|enemy| enemy.name == "Frozen Dummy")
        .unwrap();
    assert_eq!(burning.hp, 8);
    assert_eq!(burning.burning_turns, 0);
    assert_eq!(frozen.frozen_turns, 0);
    assert_eq!(c.hp, before_hp);
    assert!(
        d.log
            .iter()
            .any(|line| line.contains("Burning Dummy burns for"))
    );
    assert!(
        d.log
            .iter()
            .any(|line| line.contains("Frozen Dummy is frozen and skips its turn."))
    );
}

#[test]
fn firebolt_requires_line_of_sight_and_spends_no_mana_without_target() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    let enemy = skeleton(5, 2);
    let mut d = open_test_dungeon(2, 2, vec![enemy]);
    d.tiles[tile_index(3, 2)] = '#';
    c.active_dungeon = Some(d);
    let before_mana = c.mana;

    assert!(!use_firebolt_with_rolls(&mut c, 0.0, 0.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.mana, before_mana);
    assert_eq!(d.enemies[0].hp, d.enemies[0].max_hp);
    assert!(d.log.iter().any(|line| line.contains("No enemy in sight.")));
}

#[test]
fn firebolt_miss_spends_mana_and_turn_without_burning() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![skeleton(5, 2)]));
    let before_mana = c.mana;

    assert!(use_firebolt_with_rolls(&mut c, 1.0, 0.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.mana, before_mana - FIREBOLT_MANA_COST);
    assert_eq!(d.enemies[0].hp, d.enemies[0].max_hp);
    assert_eq!(d.enemies[0].burning_turns, 0);
    assert!(d.log.iter().any(|line| line.contains("Firebolt misses")));
}

#[test]
fn firebolt_hit_uses_int_spell_damage_and_can_apply_burning() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.strength = 0;
    c.intelligence = 6;
    let mut enemy = skeleton(5, 2);
    enemy.armor = 0;
    enemy.hp = 30;
    enemy.max_hp = 30;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    assert!(use_firebolt_with_rolls(&mut c, 0.0, 0.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d.enemies[0].hp < 30);
    assert_eq!(d.enemies[0].burning_turns, BURNING_TURNS);
    assert!(d.enemies[0].burning_damage > 0);
    assert!(d.log.iter().any(|line| line.contains("Firebolt burns")));
}

#[test]
fn frost_ring_hits_all_eight_surrounding_tiles_and_freezes_on_chance() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    let adjacent = [
        (1, 1),
        (2, 1),
        (3, 1),
        (1, 2),
        (3, 2),
        (1, 3),
        (2, 3),
        (3, 3),
    ];
    let mut enemies = adjacent
        .iter()
        .enumerate()
        .map(|(index, (x, y))| {
            let mut enemy = skeleton(*x, *y);
            enemy.name = format!("Frost Dummy {index}");
            enemy.armor = 0;
            enemy.hp = 20;
            enemy.max_hp = 20;
            enemy
        })
        .collect::<Vec<_>>();
    let mut far = skeleton(5, 5);
    far.name = "Far Dummy".to_string();
    far.hp = 20;
    far.max_hp = 20;
    enemies.push(far);
    c.active_dungeon = Some(open_test_dungeon(2, 2, enemies));

    assert!(use_frost_ring_with_rolls(&mut c, 0.0, 0.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.mana, c.max_mana() - FROST_RING_MANA_COST);
    assert_eq!(c.sorceress.frost_ring_cooldown, FROST_RING_COOLDOWN);
    for enemy in d
        .enemies
        .iter()
        .filter(|enemy| enemy.name.starts_with("Frost Dummy"))
    {
        assert!(enemy.hp < 20, "{} was not damaged", enemy.name);
        assert_eq!(
            enemy.frozen_turns, FROZEN_TURNS,
            "{} was not frozen",
            enemy.name
        );
    }
    let far = d
        .enemies
        .iter()
        .find(|enemy| enemy.name == "Far Dummy")
        .unwrap();
    assert_eq!(far.hp, 20);
    assert_eq!(far.frozen_turns, 0);
}

#[test]
fn new_sorceress_can_toggle_mana_shield_from_start() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(!handle_class_skill_key(&mut c, '4'));
    assert!(c.sorceress.mana_shield_active);
    assert!(
        c.active_dungeon
            .as_ref()
            .unwrap()
            .log
            .iter()
            .any(|line| line.contains("Mana Shield toggled on."))
    );

    assert!(!handle_class_skill_key(&mut c, '4'));
    assert!(!c.sorceress.mana_shield_active);
}

#[test]
fn mana_shield_hotkey_reports_unlearned_only_when_rank_zero() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.mana_shield_rank = 0;
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(!handle_class_skill_key(&mut c, '4'));

    assert!(!c.sorceress.mana_shield_active);
    assert!(
        c.active_dungeon
            .as_ref()
            .unwrap()
            .log
            .iter()
            .any(|line| line.contains("Mana Shield is unlearned; spend a skill point to learn it."))
    );
}

#[test]
fn mana_shield_turns_off_when_spells_spend_last_mana() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.mana = FIREBOLT_MANA_COST;
    c.sorceress.mana_shield_rank = 1;
    c.sorceress.mana_shield_active = true;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![skeleton(5, 2)]));

    assert!(use_firebolt_with_rolls(&mut c, 1.0, 0.0, 0.0));

    assert_eq!(c.mana, 0);
    assert!(!c.sorceress.mana_shield_active);
}

#[test]
fn mana_shield_cannot_toggle_on_without_mana() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.mana = 0;
    c.sorceress.mana_shield_rank = 1;
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(!handle_class_skill_key(&mut c, '4'));

    assert!(!c.sorceress.mana_shield_active);
    assert!(
        c.active_dungeon
            .as_ref()
            .unwrap()
            .log
            .iter()
            .any(|line| line.contains("Mana Shield requires mana."))
    );
}

#[test]
fn chain_spark_requires_initial_line_of_sight_and_miss_ends_chain() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(
        2,
        2,
        vec![skeleton(5, 2), skeleton(6, 2)],
    ));
    let before_mana = c.mana;

    assert!(use_chain_spark_with_rolls(&mut c, 1.0, 0.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.mana, before_mana - CHAIN_SPARK_MANA_COST);
    assert_eq!(c.sorceress.chain_spark_cooldown, CHAIN_SPARK_COOLDOWN);
    assert!(d.enemies.iter().all(|enemy| enemy.hp == enemy.max_hp));
    assert!(d.log.iter().any(|line| line.contains("Chain Spark misses")));
}

#[test]
fn chain_spark_jumps_within_radius_two_including_diagonals() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.chain_spark_rank = 3;
    let mut first = skeleton(5, 2);
    first.name = "First".to_string();
    first.armor = 0;
    first.hp = 20;
    first.max_hp = 20;
    let mut diagonal = skeleton(7, 4);
    diagonal.name = "Diagonal".to_string();
    diagonal.armor = 0;
    diagonal.hp = 20;
    diagonal.max_hp = 20;
    let mut second_jump = skeleton(8, 5);
    second_jump.name = "Second Jump".to_string();
    second_jump.armor = 0;
    second_jump.hp = 20;
    second_jump.max_hp = 20;
    let mut too_far = skeleton(12, 8);
    too_far.name = "Too Far".to_string();
    too_far.hp = 20;
    too_far.max_hp = 20;
    c.active_dungeon = Some(open_test_dungeon(
        2,
        2,
        vec![first, diagonal, second_jump, too_far],
    ));

    assert!(use_chain_spark_with_rolls(&mut c, 0.0, 1.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert!(
        d.enemies
            .iter()
            .find(|enemy| enemy.name == "First")
            .unwrap()
            .hp
            < 20
    );
    assert!(
        d.enemies
            .iter()
            .find(|enemy| enemy.name == "Diagonal")
            .unwrap()
            .hp
            < 20
    );
    assert!(
        d.enemies
            .iter()
            .find(|enemy| enemy.name == "Second Jump")
            .unwrap()
            .hp
            < 20
    );
    assert_eq!(
        d.enemies
            .iter()
            .find(|enemy| enemy.name == "Too Far")
            .unwrap()
            .hp,
        20
    );
}

#[test]
fn chain_spark_jumps_around_corners_but_not_through_walls() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.chain_spark_rank = 5;
    let mut first = skeleton(5, 2);
    first.name = "First".to_string();
    first.armor = 0;
    first.hp = 20;
    first.max_hp = 20;
    let mut around_corner = skeleton(6, 3);
    around_corner.name = "Around Corner".to_string();
    around_corner.armor = 0;
    around_corner.hp = 20;
    around_corner.max_hp = 20;
    let mut blocked = skeleton(7, 2);
    blocked.name = "Blocked".to_string();
    blocked.armor = 0;
    blocked.hp = 20;
    blocked.max_hp = 20;
    let mut d = open_test_dungeon(2, 2, vec![first, around_corner, blocked]);
    d.tiles[tile_index(6, 2)] = '#';
    d.tiles[tile_index(7, 1)] = '#';
    d.tiles[tile_index(7, 3)] = '#';
    c.active_dungeon = Some(d);

    assert!(use_chain_spark_with_rolls(&mut c, 0.0, 1.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert!(
        d.enemies
            .iter()
            .find(|enemy| enemy.name == "Around Corner")
            .unwrap()
            .hp
            < 20
    );
    assert_eq!(
        d.enemies
            .iter()
            .find(|enemy| enemy.name == "Blocked")
            .unwrap()
            .hp,
        20
    );
}

#[test]
fn static_charge_applies_shocked_and_replaces_only_with_equal_or_stronger_bonus() {
    let mut enemy = skeleton(5, 2);

    apply_shocked_if_stronger(&mut enemy, 25);
    assert_eq!(enemy.shocked_bonus_percent, 25);
    apply_shocked_if_stronger(&mut enemy, 15);
    assert_eq!(enemy.shocked_bonus_percent, 25);
    apply_shocked_if_stronger(&mut enemy, 25);
    assert_eq!(enemy.shocked_bonus_percent, 25);
    apply_shocked_if_stronger(&mut enemy, 35);
    assert_eq!(enemy.shocked_bonus_percent, 35);
}

#[test]
fn shocked_bonus_is_consumed_by_next_damaging_hit() {
    let mut enemy = skeleton(5, 2);
    enemy.shocked_bonus_percent = 25;

    let damage = apply_shock_bonus_to_damage(&mut enemy, 20);

    assert_eq!(damage, 25);
    assert_eq!(enemy.shocked_bonus_percent, 0);

    let damage_without_shock = apply_shock_bonus_to_damage(&mut enemy, 20);
    assert_eq!(damage_without_shock, 20);
}

#[test]
fn rogue_dungeon_header_uses_energy_resource_label() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    rogue.rogue.energy = 42;
    rogue.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| render_dungeon(frame, &rogue))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Energy 42/100"));
    assert!(!rendered.contains("Mana 15/15"));
}

#[test]
fn town_and_dungeon_huds_render_level_aware_unicode_xp_bar() {
    use ratatui::{Terminal, backend::TestBackend};

    let expected = "Lv 2  XP ██░░░░░░░░░░░░░░░░░░ 10%";
    let mut c = test_character();
    c.level = 2;
    c.xp = 8;

    let mut town_terminal = Terminal::new(TestBackend::new(120, 28)).unwrap();
    town_terminal
        .draw(|frame| render_town(frame, &c, ""))
        .unwrap();
    let town = backend_text(&town_terminal);
    assert!(
        town.contains(expected),
        "{}",
        backend_lines(&town_terminal).join("\n")
    );
    assert!(!town.contains("XP 8/80"));

    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    let mut dungeon_terminal = Terminal::new(TestBackend::new(120, 32)).unwrap();
    dungeon_terminal
        .draw(|frame| render_dungeon(frame, &c))
        .unwrap();
    let dungeon = backend_text(&dungeon_terminal);
    assert!(
        dungeon.contains(expected),
        "{}",
        backend_lines(&dungeon_terminal).join("\n")
    );
    assert!(!dungeon.contains("XP 8/80"));
}

#[test]
fn rogue_dungeon_render_shows_fourth_skill_help() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    rogue.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| render_dungeon(frame, &rogue))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(
        rendered.contains("4 Smoke Step"),
        "{}",
        backend_lines(&terminal).join("\n")
    );
}

#[test]
fn rogue_builders_grant_combo_points_and_cap_at_five() {
    let enemy = armored_training_dummy(3, 2);
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    for _ in 0..200 {
        c.rogue.energy = ROGUE_MAX_ENERGY;
        assert!(use_backstab(&mut c));
        if c.rogue.combo_points == ROGUE_MAX_COMBO_POINTS {
            break;
        }
    }

    assert_eq!(c.rogue.combo_points, ROGUE_MAX_COMBO_POINTS);
}

#[test]
fn poisoned_target_empowers_backstab_multiplier() {
    let enemy = armored_training_dummy(3, 2);
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    assert_eq!(backstab_multiplier_for_target(&c, 0), 0.90);

    c.active_dungeon.as_mut().unwrap().enemies[0].poison_turns = 2;
    assert_eq!(backstab_multiplier_for_target(&c, 0), 1.20);

    c.active_dungeon.as_mut().unwrap().enemies[0].poison_turns = 0;
    c.rogue.empowered_backstab_turns = 1;
    assert_eq!(backstab_multiplier_for_target(&c, 0), 1.20);
}

#[test]
fn rogue_movement_enables_next_backstab_after_turn_tick() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(try_move(&mut c, 1, 0));
    tick_player_effects(&mut c);

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!((d.player_x, d.player_y), (3, 2));
    assert_eq!(c.rogue.empowered_backstab_turns, 1);
    assert_eq!(backstab_multiplier(&c), 1.20);
}

#[test]
fn rogue_movement_attack_does_not_enable_backstab() {
    let enemy = armored_training_dummy(3, 2);
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    assert!(try_move(&mut c, 1, 0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!((d.player_x, d.player_y), (2, 2));
    assert_eq!(c.rogue.empowered_backstab_turns, 0);
}

#[test]
fn rogue_missed_attacks_do_not_apply_on_hit_effects() {
    let mut found_backstab_miss = false;
    for _ in 0..400 {
        let mut c = Character::new(
            "Sneak".to_string(),
            CharacterClass::Rogue,
            DeathMode::Softcore,
        );
        c.dexterity = 0;
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));

        assert!(use_backstab(&mut c));
        let d = c.active_dungeon.as_ref().unwrap();
        if d.log.iter().any(|line| line.contains("miss")) {
            assert_eq!(c.rogue.combo_points, 0);
            found_backstab_miss = true;
            break;
        }
    }
    assert!(
        found_backstab_miss,
        "miss did not occur during Backstab attempts"
    );

    let mut found_venom_miss = false;
    for _ in 0..400 {
        let mut c = Character::new(
            "Sneak".to_string(),
            CharacterClass::Rogue,
            DeathMode::Softcore,
        );
        c.dexterity = 0;
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));

        assert!(use_venom_edge(&mut c));
        let d = c.active_dungeon.as_ref().unwrap();
        if d.log.iter().any(|line| line.contains("miss")) {
            assert_eq!(c.rogue.combo_points, 0);
            assert_eq!(d.enemies[0].poison_turns, 0);
            found_venom_miss = true;
            break;
        }
    }
    assert!(
        found_venom_miss,
        "miss did not occur during Venom Edge attempts"
    );

    let mut found_eviscerate_miss = false;
    for _ in 0..400 {
        let mut c = Character::new(
            "Sneak".to_string(),
            CharacterClass::Rogue,
            DeathMode::Softcore,
        );
        c.dexterity = 0;
        c.rogue.combo_points = 3;
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));
        {
            let d = c.active_dungeon.as_mut().unwrap();
            d.enemies[0].poison_turns = 2;
            d.enemies[0].poison_damage = 4;
        }

        assert!(use_eviscerate(&mut c));
        let d = c.active_dungeon.as_ref().unwrap();
        if d.log.iter().any(|line| line.contains("miss")) {
            assert_eq!(c.rogue.combo_points, 0);
            assert_eq!(d.enemies[0].poison_turns, 2);
            assert_eq!(d.enemies[0].hp, d.enemies[0].max_hp);
            found_eviscerate_miss = true;
            break;
        }
    }
    assert!(
        found_eviscerate_miss,
        "miss did not occur during Eviscerate attempts"
    );
}

#[test]
fn backstab_boss_kill_leaves_combo_cleared() {
    for _ in 0..200 {
        let mut c = Character::new(
            "Sneak".to_string(),
            CharacterClass::Rogue,
            DeathMode::Softcore,
        );
        c.dexterity = 1000;
        c.equipped_weapon.damage_min = 20;
        c.equipped_weapon.damage_max = 20;
        c.equipped_weapon.crit_chance = 0;
        c.rogue.combo_points = 4;
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![one_hp_test_boss(3, 2)]));

        assert!(use_backstab(&mut c));
        if c.pending_town_message.contains("Defeated Test Boss") {
            assert_eq!(c.rogue.combo_points, 0);
            return;
        }
    }

    panic!("backstab boss cleanup test missed every attack");
}

#[test]
fn eviscerate_requires_and_spends_combo_points() {
    let enemy = armored_training_dummy(3, 2);
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    assert!(!use_eviscerate(&mut c));
    assert_eq!(c.rogue.combo_points, 0);

    c.rogue.combo_points = 3;
    for _ in 0..200 {
        c.rogue.energy = ROGUE_MAX_ENERGY;
        if use_eviscerate(&mut c) && c.rogue.combo_points == 0 {
            break;
        }
    }
    assert_eq!(c.rogue.combo_points, 0);
}

#[test]
fn rogue_skills_spend_energy_on_valid_targets() {
    let mut backstabber = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    backstabber.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));
    assert!(use_backstab(&mut backstabber));
    assert_eq!(backstabber.rogue.energy, ROGUE_MAX_ENERGY - 25);

    let mut venom = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    venom.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));
    assert!(use_venom_edge(&mut venom));
    assert_eq!(venom.rogue.energy, ROGUE_MAX_ENERGY - 30);

    let mut finisher = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    finisher.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));
    finisher.rogue.combo_points = 3;
    assert!(use_eviscerate(&mut finisher));
    assert_eq!(finisher.rogue.energy, ROGUE_MAX_ENERGY - 35);
}

#[test]
fn rogue_skills_refund_energy_without_adjacent_target() {
    let mut backstabber = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    backstabber.rogue.energy = 40;
    backstabber.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    assert!(!use_backstab(&mut backstabber));
    assert_eq!(backstabber.rogue.energy, 40);

    let mut venom = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    venom.rogue.energy = 40;
    venom.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    assert!(!use_venom_edge(&mut venom));
    assert_eq!(venom.rogue.energy, 40);

    let mut finisher = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    finisher.rogue.energy = 40;
    finisher.rogue.combo_points = 3;
    finisher.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    assert!(!use_eviscerate(&mut finisher));
    assert_eq!(finisher.rogue.energy, 40);
    assert_eq!(finisher.rogue.combo_points, 3);
}

#[test]
fn venom_edge_applies_poison_and_grants_combo_point() {
    let enemy = armored_training_dummy(3, 2);
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    for _ in 0..200 {
        c.rogue.energy = ROGUE_MAX_ENERGY;
        if use_venom_edge(&mut c) && c.rogue.combo_points == 1 {
            break;
        }
    }

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.rogue.combo_points, 1);
    assert_eq!(d.enemies[0].poison_turns, 3);
    assert!(d.enemies[0].poison_damage > 0);
}

#[test]
fn rupture_rank_extends_venom_edge_poison_duration() {
    let mut rank_one = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    rank_one.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));

    for _ in 0..200 {
        rank_one.rogue.energy = ROGUE_MAX_ENERGY;
        if use_venom_edge(&mut rank_one)
            && rank_one.active_dungeon.as_ref().unwrap().enemies[0].poison_turns > 0
        {
            break;
        }
    }

    let rank_one_duration = rank_one.active_dungeon.as_ref().unwrap().enemies[0].poison_turns;

    let mut rank_five = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    rank_five.rogue.rupture_rank = 5;
    rank_five.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));

    for _ in 0..200 {
        rank_five.rogue.energy = ROGUE_MAX_ENERGY;
        if use_venom_edge(&mut rank_five)
            && rank_five.active_dungeon.as_ref().unwrap().enemies[0].poison_turns > 0
        {
            break;
        }
    }

    let rank_five_duration = rank_five.active_dungeon.as_ref().unwrap().enemies[0].poison_turns;
    assert_eq!(rank_one_duration, 3);
    assert_eq!(rank_five_duration, 7);
}

#[test]
fn poison_tick_damages_decrements_and_awards_rewards() {
    let mut enemy = enemy(
        "Poison Tick Dummy",
        'p',
        12,
        12,
        enemy_stats(3, 0, 0, 0, 10),
        enemy_rewards(10, 1, 1),
        false,
    );
    enemy.hp = 2;
    enemy.max_hp = 2;
    enemy.poison_turns = 2;
    enemy.poison_damage = 1;

    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    enemy_turns(&mut c);
    {
        let d = c.active_dungeon.as_ref().unwrap();
        assert_eq!(d.enemies[0].hp, 1);
        assert_eq!(d.enemies[0].poison_turns, 1);
    }
    assert_eq!(c.xp, 0);

    enemy_turns(&mut c);

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(living_monster_count(d), 0);
    assert!(c.xp >= 10);
}

fn carried_or_ground_loot_count(c: &Character) -> usize {
    c.inventory.len()
        + c.active_dungeon
            .as_ref()
            .map(|d| d.ground_items.len())
            .unwrap_or_default()
}

fn tick_effect_kill_rolls_loot(enemy: Enemy) -> bool {
    for _ in 0..400 {
        let mut c = Character::new(
            "Loot".to_string(),
            CharacterClass::Sorceress,
            DeathMode::Softcore,
        );
        c.inventory.clear();
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy.clone()]));

        enemy_turns(&mut c);

        if carried_or_ground_loot_count(&c) > 0 {
            return true;
        }
    }
    false
}

#[test]
fn poison_tick_kills_can_roll_regular_loot() {
    let mut enemy = skeleton(12, 12);
    enemy.hp = 1;
    enemy.max_hp = 1;
    enemy.poison_turns = 1;
    enemy.poison_damage = 1;

    assert!(tick_effect_kill_rolls_loot(enemy));
}

#[test]
fn bleed_tick_kills_can_roll_regular_loot() {
    let mut enemy = skeleton(12, 12);
    enemy.hp = 1;
    enemy.max_hp = 1;
    enemy.bleed_turns = 1;
    enemy.bleed_damage = 1;

    assert!(tick_effect_kill_rolls_loot(enemy));
}

#[test]
fn burning_tick_kills_can_roll_regular_loot() {
    let mut enemy = skeleton(12, 12);
    enemy.hp = 1;
    enemy.max_hp = 1;
    enemy.burning_turns = 1;
    enemy.burning_damage = 1;

    assert!(tick_effect_kill_rolls_loot(enemy));
}

#[test]
fn rogue_effect_kills_can_roll_regular_loot() {
    for _ in 0..400 {
        let mut c = Character::new(
            "Sneak".to_string(),
            CharacterClass::Rogue,
            DeathMode::Softcore,
        );
        c.inventory.clear();
        let mut enemy = skeleton(3, 2);
        enemy.hp = 1;
        enemy.max_hp = 1;
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

        assert_eq!(
            damage_enemy_with_rogue_effect(&mut c, 0, "Rogue effect", 1),
            DamageEnemyOutcome::Killed
        );

        if carried_or_ground_loot_count(&c) > 0 {
            return;
        }
    }

    panic!("rogue effect kills never rolled loot");
}

#[test]
fn spiked_guard_kills_can_roll_regular_loot() {
    for _ in 0..400 {
        let mut c = Character::new(
            "Guard".to_string(),
            CharacterClass::Warrior,
            DeathMode::Softcore,
        );
        c.inventory.clear();
        c.hp = 999;
        c.warrior.iron_guard_mastery = Some(SkillMastery::SpikedGuard);
        let mut enemy = enemy(
            "Thorn Dummy",
            's',
            3,
            2,
            enemy_stats_with_ratings(2, 0, 0, 0, 10, 1_000, 0),
            enemy_rewards(10, 1, 1),
            false,
        );
        enemy.energy = 999;
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

        enemy_turns(&mut c);

        if carried_or_ground_loot_count(&c) > 0 {
            return;
        }
    }

    panic!("Spiked Guard kills never rolled loot");
}

#[test]
fn eviscerate_poison_payoff_can_kill_and_award_rewards() {
    for _ in 0..200 {
        let enemy = enemy(
            "Poison Dummy",
            'p',
            3,
            2,
            enemy_stats(3, 0, 0, 0, 10),
            enemy_rewards(10, 1, 1),
            false,
        );
        let mut c = Character::new(
            "Sneak".to_string(),
            CharacterClass::Rogue,
            DeathMode::Softcore,
        );
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));
        c.rogue.combo_points = 5;
        c.rogue.energy = ROGUE_MAX_ENERGY;
        {
            let d = c.active_dungeon.as_mut().unwrap();
            d.enemies[0].poison_turns = 3;
            d.enemies[0].poison_damage = 3;
        }

        assert!(use_eviscerate(&mut c));

        let d = c.active_dungeon.as_ref().unwrap();
        if d.enemies.is_empty() || d.enemies[0].hp <= 0 {
            assert!(c.xp >= 10);
            return;
        }
    }

    panic!("eviscerate poison payoff test missed every attack");
}

#[test]
fn eviscerate_retained_boss_dungeon_leaves_smoke_protection_cleared() {
    for _ in 0..200 {
        let mut c = Character::new(
            "Sneak".to_string(),
            CharacterClass::Rogue,
            DeathMode::Softcore,
        );
        fill_inventory_to_capacity(&mut c);
        c.dexterity = 1000;
        c.strength = 0;
        c.equipped_weapon.damage_min = 1;
        c.equipped_weapon.damage_max = 1;
        c.equipped_weapon.crit_chance = 0;
        c.rogue.combo_points = 5;
        c.rogue.energy = ROGUE_MAX_ENERGY;
        let mut boss = one_hp_test_boss(3, 2);
        boss.hp = 7;
        boss.max_hp = 7;
        boss.poison_turns = 3;
        boss.poison_damage = 1;
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![boss]));

        assert!(use_eviscerate(&mut c));
        if c.pending_town_message.contains("Defeated Test Boss") {
            assert_eq!(c.rogue.smoke_protection_turns, 0);
            return;
        }
    }

    panic!("eviscerate boss cleanup test missed every attack");
}

#[test]
fn smoke_step_hotkey_moves_to_default_open_destination() {
    let mut rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    rogue.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(is_known_dungeon_command_for(&rogue, '4'));
    assert!(!handle_class_skill_key(&mut rogue, '4'));

    let d = rogue.active_dungeon.as_ref().unwrap();
    assert_eq!((d.player_x, d.player_y), (2, 2));
    assert!(rogue.rogue.smoke_step_pending);
    assert!(
        d.log
            .iter()
            .any(|line| line.contains("Choose a Smoke Step direction."))
    );
}

#[test]
fn smoke_step_direction_key_moves_and_interacts_with_landing_tile() {
    let mut rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.stairs_x = 4;
    d.stairs_y = 2;
    rogue.active_dungeon = Some(d);

    assert!(!handle_class_skill_key(&mut rogue, '4'));
    assert!(handle_pending_smoke_step_key(&mut rogue, 'D'));

    let d = rogue.active_dungeon.as_ref().unwrap();
    assert_eq!(d.floor, 3);
    assert!(!rogue.rogue.smoke_step_pending);
}

#[test]
fn smoke_step_rejects_blocked_occupied_and_out_of_range_destinations() {
    let enemy = armored_training_dummy(4, 2);
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    assert!(!try_smoke_step(&mut c, 3, 0));
    assert!(!try_smoke_step(&mut c, 2, 0));
    assert!(!try_smoke_step(&mut c, -2, 0));
}

#[test]
fn smoke_step_moves_and_enables_empowered_backstab() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(try_smoke_step(&mut c, 2, 0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!((d.player_x, d.player_y), (4, 2));
    assert_eq!(c.rogue.smoke_step_cooldown, 4);
    assert_eq!(c.rogue.smoke_protection_turns, 1);
    tick_player_effects(&mut c);
    assert_eq!(c.rogue.empowered_backstab_turns, 1);
    assert_eq!(backstab_multiplier(&c), 1.20);
}

#[test]
fn smoke_step_rejects_two_tile_path_blocked_by_wall_or_enemy() {
    let mut wall_blocked = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    let mut wall_dungeon = open_test_dungeon(2, 2, Vec::new());
    wall_dungeon.tiles[tile_index(3, 2)] = '#';
    wall_blocked.active_dungeon = Some(wall_dungeon);

    assert!(!try_smoke_step(&mut wall_blocked, 2, 0));
    let d = wall_blocked.active_dungeon.as_ref().unwrap();
    assert_eq!((d.player_x, d.player_y), (2, 2));

    let mut enemy_blocked = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    enemy_blocked.active_dungeon =
        Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));

    assert!(!try_smoke_step(&mut enemy_blocked, 2, 0));
    let d = enemy_blocked.active_dungeon.as_ref().unwrap();
    assert_eq!((d.player_x, d.player_y), (2, 2));
}

#[test]
fn smoke_step_direction_skips_blocked_intervening_tile() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));

    assert_eq!(smoke_step_direction(&c), Some((0, 2)));
}

#[test]
fn smoke_protection_adds_rogue_defensive_dodge_bonus() {
    let mut rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    let base_dodge = rogue.dodge_rating() as i32;

    assert_eq!(defensive_dodge_rating(&rogue), base_dodge);

    rogue.rogue.smoke_protection_turns = 1;

    assert_eq!(defensive_dodge_rating(&rogue), base_dodge + 20);

    rogue.rogue.smoke_step_rank = 5;

    assert_eq!(defensive_dodge_rating(&rogue), base_dodge + 32);

    let mut warrior = test_character();
    warrior.rogue.smoke_protection_turns = 1;

    assert_eq!(
        defensive_dodge_rating(&warrior),
        warrior.dodge_rating() as i32
    );
}

#[test]
fn slip_away_rank_increases_smoke_protection_dodge_bonus() {
    let mut rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    let base_dodge = rogue.dodge_rating() as i32;
    rogue.rogue.smoke_protection_turns = 1;
    rogue.rogue.smoke_step_rank = 1;
    rogue.rogue.slip_away_rank = 1;

    assert_eq!(defensive_dodge_rating(&rogue), base_dodge + 25);

    rogue.rogue.slip_away_rank = 5;

    assert_eq!(defensive_dodge_rating(&rogue), base_dodge + 33);
}

#[test]
fn warrior_does_not_accept_rogue_fourth_skill_key() {
    let mut warrior = test_character();
    warrior.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(!is_known_dungeon_command_for(&warrior, '4'));
    assert!(!is_known_dungeon_command('4'));
    assert!(!handle_class_skill_key(&mut warrior, '4'));

    let log = &warrior.active_dungeon.as_ref().unwrap().log;
    assert!(log.iter().any(|line| line.contains("Unknown class skill.")));
}

#[test]
fn terminal_resize_event_requests_redraw() {
    use crossterm::event::Event;

    assert_eq!(
        terminal_event_to_input(Event::Resize(120, 40), true).unwrap(),
        Some(UiInput::Redraw)
    );
}

#[test]
fn cursor_pulse_styles_share_cursed_violet_and_toggle_bold() {
    use ratatui::style::Modifier;

    let dim = cursor_style(false);
    let bright = cursor_style(true);

    assert_eq!(dim.fg, Some(SELECTED_CONTAINER_BORDER_COLOR));
    assert_eq!(bright.fg, Some(SELECTED_CONTAINER_BORDER_COLOR));
    assert!(!dim.add_modifier.contains(Modifier::BOLD));
    assert!(bright.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn ui_palette_exposes_gothic_cursed_semantic_styles() {
    use ratatui::style::{Color, Modifier, Style};

    assert_eq!(TEXT_PRIMARY_COLOR, Color::Rgb(214, 203, 177));
    assert_eq!(TEXT_MUTED_COLOR, Color::Rgb(108, 101, 112));
    assert_eq!(CONTAINER_BORDER_COLOR, Color::Rgb(75, 67, 84));
    assert_eq!(SELECTED_CONTAINER_BORDER_COLOR, Color::Rgb(148, 80, 190));
    assert_eq!(TITLE_COLOR, Color::Rgb(201, 163, 86));
    assert_eq!(DANGER_COLOR, Color::Rgb(188, 54, 54));
    assert_eq!(ACTION_COLOR, Color::Rgb(93, 153, 112));
    assert_eq!(WARNING_COLOR, Color::Rgb(214, 157, 73));
    assert_eq!(ARCANE_COLOR, Color::Rgb(113, 151, 201));
    assert_eq!(CURSED_COLOR, Color::Rgb(177, 93, 204));

    assert_eq!(body_style(), Style::default().fg(TEXT_PRIMARY_COLOR));
    assert_eq!(muted_style(), Style::default().fg(TEXT_MUTED_COLOR));
    assert_eq!(
        title_style(),
        Style::default()
            .fg(TITLE_COLOR)
            .add_modifier(Modifier::BOLD)
    );
    assert_eq!(
        container_border_style(false),
        Style::default().fg(CONTAINER_BORDER_COLOR)
    );
    assert_eq!(
        container_border_style(true),
        Style::default().fg(SELECTED_CONTAINER_BORDER_COLOR)
    );
}

#[test]
fn help_topics_cover_requested_and_major_game_keywords() {
    let keywords: Vec<_> = help_topics().iter().map(|topic| topic.keyword).collect();
    for required in [
        "Strength",
        "Dexterity",
        "Intelligence",
        "Burning",
        "Bleeding",
        "Gold",
        "Health",
        "Mana",
        "Energy",
        "Combo Points",
        "Armor",
        "Dodge Rating",
        "Hit Rating",
        "Speed",
        "Critical Chance",
        "Poisoned",
        "Frozen",
        "Shocked",
        "Stunned",
        "Warrior",
        "Rogue",
        "Sorceress",
        "Cleave",
        "Backstab",
        "Firebolt",
        "Quest",
        "Stash",
        "Town Projects",
        "Sockets",
        "Gems",
        "Bellkeeper",
        "Glass Tyrant",
        "Hardcore",
        "Softcore",
        "Hollow Crypts",
        "Glass Wastes",
    ] {
        assert!(
            keywords.contains(&required),
            "missing help topic {required}"
        );
    }
    assert!(
        keywords.len() >= 90,
        "glossary should include broad game vocabulary"
    );
}

#[test]
fn help_search_filters_keywords_case_insensitively_and_keeps_selection_valid() {
    let mut state = HelpScreenState::new();
    state.handle_key('d');
    state.handle_key('E');
    state.handle_key('x');

    let filtered: Vec<_> = state
        .filtered_topics()
        .iter()
        .map(|topic| topic.keyword)
        .collect();
    assert!(filtered.contains(&"Dexterity"));
    assert!(
        filtered
            .iter()
            .all(|keyword| keyword.to_ascii_lowercase().contains("dex"))
    );
    assert_eq!(state.selected_topic().unwrap().keyword, "Dexterity");

    state.handle_key('\u{8}');
    assert_eq!(state.query(), "dE");
}

#[test]
fn help_screen_renders_search_keyword_list_details_and_footer() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut state = HelpScreenState::new();
    state.handle_key('g');
    state.handle_key('o');
    state.handle_key('l');
    state.handle_key('d');
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();

    terminal
        .draw(|frame| render_help_screen(frame, &state))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(
        rendered.contains("Search: gold"),
        "{}",
        backend_lines(&terminal).join("\n")
    );
    assert!(rendered.contains("Gold"));
    assert!(rendered.contains("currency"));
    assert!(rendered.contains("Up/Down=select"));
    assert!(rendered.contains("Esc=back"));
}

#[test]
fn town_footer_advertises_help_hotkey() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();
    terminal.draw(|frame| render_town(frame, &c, "")).unwrap();
    let rendered = backend_text(&terminal);

    assert!(
        rendered.contains("h=help"),
        "{}",
        backend_lines(&terminal).join("\n")
    );
}

#[test]
fn dungeon_footer_and_known_commands_include_help_hotkey() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();
    terminal.draw(|frame| render_dungeon(frame, &c)).unwrap();
    let rendered = backend_text(&terminal);

    assert!(
        rendered.contains("h=help"),
        "{}",
        backend_lines(&terminal).join("\n")
    );
    assert!(is_known_dungeon_command_for(&c, 'h'));
    assert!(is_known_dungeon_command_for(&c, 'H'));
}

#[test]
fn cursor_pulse_timeout_returns_tick() {
    assert_eq!(
        terminal_event_timeout_to_input(None, true, false).unwrap(),
        UiInput::Tick
    );
}

#[test]
fn terminal_key_release_events_are_ignored() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    let event = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Char('x'),
        KeyModifiers::NONE,
        KeyEventKind::Release,
    ));

    assert_eq!(terminal_event_to_input(event, true).unwrap(), None);
}

#[test]
fn terminal_key_repeat_events_are_ignored() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    let event = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Char('x'),
        KeyModifiers::NONE,
        KeyEventKind::Repeat,
    ));

    assert_eq!(terminal_event_to_input(event, true).unwrap(), None);
}

#[test]
fn raw_arrow_keys_emit_character_creation_navigation_keys() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    let up = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Up,
        KeyModifiers::NONE,
        KeyEventKind::Press,
    ));
    let down = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Down,
        KeyModifiers::NONE,
        KeyEventKind::Press,
    ));

    assert_eq!(terminal_event_to_input(up, false).unwrap(), None);
    assert_eq!(terminal_event_to_input(down, false).unwrap(), None);

    let up = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Up,
        KeyModifiers::NONE,
        KeyEventKind::Press,
    ));
    let down = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Down,
        KeyModifiers::NONE,
        KeyEventKind::Press,
    ));

    assert_eq!(
        terminal_event_to_input_raw_arrows(up).unwrap(),
        Some(UiInput::Key('\u{10}'))
    );
    assert_eq!(
        terminal_event_to_input_raw_arrows(down).unwrap(),
        Some(UiInput::Key('\u{0e}'))
    );
}

#[test]
fn valid_dungeon_command_clears_recent_unknown_command_logs() {
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    for _ in 0..2 {
        let before = current_dungeon_log_len(&c);
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            UNKNOWN_DUNGEON_COMMAND_MESSAGE,
        );
        mark_latest_log_group(&mut c, before, false, "Command");
    }

    clear_recent_unknown_dungeon_commands(&mut c);

    assert!(c.active_dungeon.as_ref().unwrap().log.is_empty());
}

#[test]
fn clearing_unknown_dungeon_command_keeps_other_recent_warnings() {
    let mut log = vec![
        "== No turn spent: Cleave ==".to_string(),
        "[WARN] No adjacent enemies for Cleave.".to_string(),
    ];

    assert!(!remove_latest_unknown_dungeon_command(&mut log));
    assert_eq!(
        log,
        vec![
            "== No turn spent: Cleave ==".to_string(),
            "[WARN] No adjacent enemies for Cleave.".to_string(),
        ]
    );
}

#[test]
fn save_character_writes_atomically() {
    let c = test_character();
    let dir = env::temp_dir().join(format!("crawltty-save-test-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let save_path = dir.join("save.json");
    let tmp_path = dir.join("save.json.tmp");

    save_character_to_path(&c, &save_path).unwrap();

    assert!(save_path.exists());
    assert!(!tmp_path.exists());
    let saved: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&save_path).unwrap()).unwrap();
    assert_eq!(saved["save_version"], SAVE_VERSION);
    let saved_character: Character = serde_json::from_value(saved["character"].clone()).unwrap();
    assert_eq!(saved_character.name, c.name);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn save_character_profile_tracks_last_character_and_per_character_file() {
    let dir = env::temp_dir().join(format!(
        "crawltty-multi-save-profile-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    let profile_path = dir.join("profile.json");
    let character_dir = dir.join("characters");

    let mara = Character::new(
        "Mara".to_string(),
        CharacterClass::Warrior,
        DeathMode::Softcore,
    );
    save_active_character_to_paths(&mara, &profile_path, &character_dir).unwrap();

    let profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&profile_path).unwrap()).unwrap();
    assert_eq!(profile["last_character_id"], "mara");
    assert!(character_dir.join("mara.json").exists());

    let loaded = load_last_character_from_paths(&profile_path, &character_dir).unwrap();
    assert_eq!(loaded.unwrap().name, "Mara");

    let shade = Character::new(
        "Shade".to_string(),
        CharacterClass::Rogue,
        DeathMode::Hardcore,
    );
    save_active_character_to_paths(&shade, &profile_path, &character_dir).unwrap();

    let profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&profile_path).unwrap()).unwrap();
    assert_eq!(profile["last_character_id"], "shade");
    assert!(character_dir.join("mara.json").exists());
    assert!(character_dir.join("shade.json").exists());
    assert_eq!(
        load_last_character_from_paths(&profile_path, &character_dir)
            .unwrap()
            .unwrap()
            .name,
        "Shade"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn character_summaries_are_loaded_from_per_character_directory() {
    let dir = env::temp_dir().join(format!(
        "crawltty-multi-save-list-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    let profile_path = dir.join("profile.json");
    let character_dir = dir.join("characters");

    let mara = Character::new(
        "Mara".to_string(),
        CharacterClass::Warrior,
        DeathMode::Softcore,
    );
    let mut shade = Character::new(
        "Shade".to_string(),
        CharacterClass::Rogue,
        DeathMode::Hardcore,
    );
    shade.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    shade.active_dungeon.as_mut().unwrap().floor = 4;
    save_active_character_to_paths(&shade, &profile_path, &character_dir).unwrap();
    save_active_character_to_paths(&mara, &profile_path, &character_dir).unwrap();

    let summaries = load_character_summaries_from_dir(&character_dir).unwrap();
    assert_eq!(
        summaries
            .iter()
            .map(|summary| summary.id.as_str())
            .collect::<Vec<_>>(),
        vec!["mara", "shade"]
    );
    assert_eq!(summaries[0].name, "Mara");
    assert_eq!(summaries[0].class_name, "Warrior");
    assert_eq!(summaries[0].death_mode, DeathMode::Softcore);
    assert_eq!(summaries[0].location, "Town");
    assert_eq!(summaries[1].name, "Shade");
    assert_eq!(summaries[1].class_name, "Rogue");
    assert_eq!(summaries[1].death_mode, DeathMode::Hardcore);
    assert_eq!(summaries[1].location, "Dungeon L4");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn per_character_saves_keep_stashes_separate() {
    let dir = env::temp_dir().join(format!(
        "crawltty-per-character-stash-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    let profile_path = dir.join("profile.json");
    let character_dir = dir.join("characters");

    let mut mara = Character::new(
        "Mara".to_string(),
        CharacterClass::Warrior,
        DeathMode::Softcore,
    );
    let shade = Character::new(
        "Shade".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    mara.stash.push(mana_potion());

    save_active_character_to_paths(&mara, &profile_path, &character_dir).unwrap();
    save_active_character_to_paths(&shade, &profile_path, &character_dir).unwrap();

    let loaded_mara = match load_character_from_path(&character_dir.join("mara.json")).unwrap() {
        LoadedSave::Loaded(character) => character,
        LoadedSave::Reset { warning } => panic!("mara should load: {warning}"),
    };
    let loaded_shade = match load_character_from_path(&character_dir.join("shade.json")).unwrap() {
        LoadedSave::Loaded(character) => character,
        LoadedSave::Reset { warning } => panic!("shade should load: {warning}"),
    };

    assert_eq!(loaded_mara.stash.len(), 1);
    assert_eq!(loaded_shade.stash.len(), 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn startup_load_prefers_last_character_from_profile() {
    let dir = env::temp_dir().join(format!(
        "crawltty-load-last-character-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    let profile_path = dir.join("profile.json");
    let character_dir = dir.join("characters");

    let mara = Character::new(
        "Mara".to_string(),
        CharacterClass::Warrior,
        DeathMode::Softcore,
    );
    let shade = Character::new(
        "Shade".to_string(),
        CharacterClass::Rogue,
        DeathMode::Hardcore,
    );
    save_active_character_to_paths(&mara, &profile_path, &character_dir).unwrap();
    save_active_character_to_paths(&shade, &profile_path, &character_dir).unwrap();

    let loaded =
        load_startup_character_from_paths(&profile_path, &character_dir, &dir.join("save.json"))
            .unwrap();
    assert_eq!(loaded.unwrap().name, "Shade");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn legacy_single_save_migrates_to_profile_and_character_file() {
    let dir = env::temp_dir().join(format!(
        "crawltty-legacy-multi-save-migration-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let profile_path = dir.join("profile.json");
    let character_dir = dir.join("characters");
    let legacy_path = dir.join("save.json");

    let legacy = Character::new(
        "Old Hero".to_string(),
        CharacterClass::Warrior,
        DeathMode::Softcore,
    );
    save_character_to_path(&legacy, &legacy_path).unwrap();

    let loaded =
        load_startup_character_from_paths(&profile_path, &character_dir, &legacy_path).unwrap();
    assert_eq!(loaded.unwrap().name, "Old Hero");
    assert!(character_dir.join("old-hero.json").exists());
    let profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&profile_path).unwrap()).unwrap();
    assert_eq!(profile["last_character_id"], "old-hero");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn save_major_version_mismatch_resets_save() {
    let c = test_character();
    let dir = env::temp_dir().join(format!(
        "crawltty-save-version-reset-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let save_path = dir.join("save.json");
    let current_major: u64 = SAVE_VERSION.split('.').next().unwrap().parse().unwrap();
    let incompatible_version = format!("{}.0.0", current_major + 1);
    let save = serde_json::json!({
        "save_version": incompatible_version,
        "character": c,
    });
    fs::write(&save_path, serde_json::to_string_pretty(&save).unwrap()).unwrap();

    let loaded = load_character_from_path(&save_path).unwrap();

    match loaded {
        LoadedSave::Reset { warning } => {
            assert!(warning.contains("incompatible"));
            assert!(warning.contains(SAVE_VERSION));
        }
        LoadedSave::Loaded(_) => panic!("major version mismatch should reset the save"),
    }
    assert!(!save_path.exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn save_minor_version_mismatch_loads_existing_character() {
    let mut c = test_character();
    c.name = "Compatible Save".to_string();
    let dir = env::temp_dir().join(format!(
        "crawltty-save-version-load-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let save_path = dir.join("save.json");
    let current_major: u64 = SAVE_VERSION.split('.').next().unwrap().parse().unwrap();
    let compatible_version = format!("{current_major}.999.0");
    let save = serde_json::json!({
        "save_version": compatible_version,
        "character": c,
    });
    fs::write(&save_path, serde_json::to_string_pretty(&save).unwrap()).unwrap();

    let loaded = load_character_from_path(&save_path).unwrap();

    match loaded {
        LoadedSave::Loaded(character) => assert_eq!(character.name, "Compatible Save"),
        LoadedSave::Reset { warning } => panic!("compatible save was reset: {warning}"),
    }
    assert!(save_path.exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn loading_save_normalizes_legacy_locked_skill_ranks() {
    let mut warrior = test_character();
    warrior.warrior.cleave_rank = 1;
    warrior.warrior.shield_bash_rank = 1;
    warrior.warrior.battle_cry_rank = 1;
    warrior.warrior.deep_cut_rank = 1;
    warrior.warrior.iron_guard_rank = 1;
    warrior.warrior.second_wind_rank = 1;
    let dir = env::temp_dir().join(format!(
        "crawltty-save-locked-rank-normalize-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let save_path = dir.join("save.json");
    let save = serde_json::json!({
        "save_version": SAVE_VERSION,
        "character": warrior,
    });
    fs::write(&save_path, serde_json::to_string_pretty(&save).unwrap()).unwrap();

    let loaded = load_character_from_path(&save_path).unwrap();

    match loaded {
        LoadedSave::Loaded(character) => {
            assert_eq!(character.warrior.deep_cut_rank, 0);
            assert_eq!(character.warrior.iron_guard_rank, 0);
            assert_eq!(character.warrior.second_wind_rank, 0);
        }
        LoadedSave::Reset { warning } => panic!("compatible save was reset: {warning}"),
    }
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn loading_sorceress_save_promotes_starting_mana_shield_rank() {
    let mut sorceress = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    sorceress.sorceress.mana_shield_rank = 0;
    sorceress.sorceress.mana_shield_active = true;
    let dir = env::temp_dir().join(format!(
        "crawltty-save-sorceress-mana-shield-promote-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let save_path = dir.join("save.json");
    let save = serde_json::json!({
        "save_version": SAVE_VERSION,
        "character": sorceress,
    });
    fs::write(&save_path, serde_json::to_string_pretty(&save).unwrap()).unwrap();

    let loaded = load_character_from_path(&save_path).unwrap();

    match loaded {
        LoadedSave::Loaded(character) => {
            assert_eq!(character.sorceress.mana_shield_rank, 1);
            assert!(!character.sorceress.mana_shield_active);
        }
        LoadedSave::Reset { warning } => panic!("compatible save was reset: {warning}"),
    }
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn bad_legacy_save_resets_instead_of_erroring() {
    let dir = env::temp_dir().join(format!(
        "crawltty-bad-legacy-save-reset-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let save_path = dir.join("save.json");
    fs::write(
        &save_path,
        serde_json::json!({
            "name": "Broken Legacy Save",
            "inventory": { "columns": 4, "rows": 4, "items": [] }
        })
        .to_string(),
    )
    .unwrap();

    let loaded = load_character_from_path(&save_path).unwrap();

    match loaded {
        LoadedSave::Reset { warning } => {
            assert!(warning.contains("is incompatible"));
            assert!(warning.contains("0.0.0"));
            assert!(warning.contains(SAVE_VERSION));
        }
        LoadedSave::Loaded(_) => panic!("bad legacy save should reset"),
    }
    assert!(!save_path.exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn bad_versioned_save_resets_instead_of_erroring() {
    let dir = env::temp_dir().join(format!(
        "crawltty-bad-versioned-save-reset-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let save_path = dir.join("save.json");
    fs::write(
        &save_path,
        serde_json::json!({
            "save_version": SAVE_VERSION,
            "character": { "name": "Broken Versioned Save" }
        })
        .to_string(),
    )
    .unwrap();

    let loaded = load_character_from_path(&save_path).unwrap();

    match loaded {
        LoadedSave::Reset { warning } => {
            assert!(warning.contains("could not be loaded"));
            assert!(warning.contains(SAVE_VERSION));
        }
        LoadedSave::Loaded(_) => panic!("bad versioned save should reset"),
    }
    assert!(!save_path.exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn save_character_creates_parent_directories() {
    let c = test_character();
    let dir = env::temp_dir().join(format!("crawltty-save-parent-test-{}", std::process::id()));
    let save_path = dir.join("nested").join("save.json");

    save_character_to_path(&c, &save_path).unwrap();

    assert!(save_path.exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn item_grid_capacity_add_remove_and_auto_compaction() {
    let mut grid = ItemGrid::new(2, 2, Vec::new());

    assert_eq!(grid.columns, 2);
    assert_eq!(grid.rows, 2);
    assert_eq!(grid.capacity(), 4);
    assert_eq!(grid.len(), 0);
    assert!(grid.is_empty());
    assert!(grid.has_space());

    assert!(grid.push(health_potion()));
    assert!(grid.push(mana_potion()));
    assert_eq!(grid.len(), 2);
    assert!(matches!(grid[0].kind, ItemKind::HealthPotion));
    assert!(matches!(grid[1].kind, ItemKind::ManaPotion));

    let removed = grid.remove(0);
    assert!(matches!(removed.kind, ItemKind::HealthPotion));
    assert_eq!(grid.len(), 1);
    assert!(matches!(grid[0].kind, ItemKind::ManaPotion));

    assert!(grid.push(health_potion()));
    assert!(grid.push(health_potion()));
    assert!(grid.push(mana_potion()));
    assert!(!grid.push(rusted_sword()));
    assert_eq!(grid.len(), 4);
}

#[test]
fn grid_cursor_movement_clamps_within_dimensions() {
    assert_eq!(move_grid_cursor(0, 4, 4, 'a'), 0);
    assert_eq!(move_grid_cursor(0, 4, 4, 'd'), 1);
    assert_eq!(move_grid_cursor(0, 4, 4, 's'), 4);
    assert_eq!(move_grid_cursor(15, 4, 4, 'd'), 15);
    assert_eq!(move_grid_cursor(15, 4, 4, 's'), 15);
    assert_eq!(move_grid_cursor(5, 4, 4, 'w'), 1);
}

#[test]
fn grid_cursor_handles_empty_and_stale_selections() {
    assert_eq!(move_grid_cursor(7, 0, 4, 'd'), 0);
    assert_eq!(move_grid_cursor(7, 4, 0, 's'), 0);
    assert_eq!(move_grid_cursor(99, 4, 4, 'a'), 14);
    assert_eq!(move_grid_cursor(99, 4, 4, 'd'), 15);
}

#[test]
fn equipment_cursor_moves_through_humanoid_body_slots() {
    assert_eq!(
        move_equipment_cursor(CharacterEquipmentSlot::Armor, 'w'),
        CharacterEquipmentSlot::Amulet
    );
    assert_eq!(
        move_equipment_cursor(CharacterEquipmentSlot::Armor, 'a'),
        CharacterEquipmentSlot::Weapon
    );
    assert_eq!(
        move_equipment_cursor(CharacterEquipmentSlot::Armor, 'd'),
        CharacterEquipmentSlot::Shield
    );
    assert_eq!(
        move_equipment_cursor(CharacterEquipmentSlot::Armor, 's'),
        CharacterEquipmentSlot::Belt
    );
    assert_eq!(
        move_equipment_cursor(CharacterEquipmentSlot::Boots, 's'),
        CharacterEquipmentSlot::Boots
    );
}

#[test]
fn clamp_grid_cursor_clamps_to_grid_capacity() {
    let grid = ItemGrid::new(2, 2, vec![health_potion()]);
    let mut selected = 99;
    clamp_grid_cursor(&mut selected, &grid);
    assert_eq!(selected, 3);

    let empty_grid = ItemGrid::new(0, 0, Vec::new());
    clamp_grid_cursor(&mut selected, &empty_grid);
    assert_eq!(selected, 0);
}

#[test]
fn inventory_cell_label_shows_item_kind_or_empty_cell() {
    let mut grid = ItemGrid::new(2, 2, vec![health_potion(), rusted_sword()]);

    assert_eq!(inventory_cell_label(&grid, 0), HEALTH_POTION_GLYPH);
    assert_eq!(inventory_cell_label(&grid, 1), WEAPON_GLYPH);
    assert_eq!(inventory_cell_label(&grid, 2), EMPTY_CELL_GLYPH);

    grid.push(mana_potion());
    assert_eq!(inventory_cell_label(&grid, 2), MANA_POTION_GLYPH);
}

#[test]
fn sorting_inventory_groups_items_by_type_rarity_level_value_and_name_without_spending_turn() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(
        4,
        4,
        vec![
            item_with_rarity(
                "Cobalt Axe",
                ItemKind::Weapon,
                30,
                weapon_stats(2, 4, 0, AXE_CRIT_CHANCE),
                Rarity::Magic,
                3,
                requirements(0, 0, 0),
            ),
            mana_potion(),
            item_with_rarity(
                "Zulu Sword",
                ItemKind::Weapon,
                999,
                weapon_stats(4, 8, 0, SWORD_CRIT_CHANCE),
                Rarity::Common,
                99,
                requirements(0, 0, 0),
            ),
            item_with_rarity(
                "Alpha Axe",
                ItemKind::Weapon,
                10,
                weapon_stats(1, 3, 0, AXE_CRIT_CHANCE),
                Rarity::Rare,
                1,
                requirements(0, 0, 0),
            ),
            item_with_rarity(
                "Amber Axe",
                ItemKind::Weapon,
                30,
                weapon_stats(2, 4, 0, AXE_CRIT_CHANCE),
                Rarity::Magic,
                3,
                requirements(0, 0, 0),
            ),
            item_with_rarity(
                "Bronze Axe",
                ItemKind::Weapon,
                20,
                weapon_stats(2, 4, 0, AXE_CRIT_CHANCE),
                Rarity::Magic,
                3,
                requirements(0, 0, 0),
            ),
            health_potion(),
            gem_item(GemKind::Topaz, GemTier::Flawed),
        ],
    );

    let result = sort_inventory(&mut c);

    assert_eq!(result.message, "Inventory sorted.");
    assert!(!result.spent_turn);
    assert_eq!(
        c.inventory
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "Lesser Health Potion (restores 15% HP)",
            "Lesser Mana Potion (restores 15% mana)",
            "Alpha Axe",
            "Amber Axe",
            "Cobalt Axe",
            "Bronze Axe",
            "Zulu Sword",
            "Flawed Topaz (+2% crit chance)",
        ]
    );
}

#[test]
fn cursor_style_uses_cursed_violet() {
    use ratatui::{Terminal, backend::TestBackend};

    let grid = ItemGrid::new(2, 2, vec![rusted_sword()]);

    let selected_item = inventory_cell_spans(&grid, 0, true);
    assert_eq!(
        selected_item[1].style.fg,
        Some(SELECTED_CONTAINER_BORDER_COLOR)
    );

    let selected_empty = inventory_cell_spans(&grid, 1, true);
    assert!(
        selected_empty
            .iter()
            .all(|span| span.style.fg == Some(SELECTED_CONTAINER_BORDER_COLOR))
    );

    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                CharacterCreationStep::Class,
                "",
                CharacterClass::Warrior,
                DeathMode::Softcore,
                "",
            )
        })
        .unwrap();
    assert_eq!(
        cell_fg_at_text(&terminal, "Warrior"),
        SELECTED_CONTAINER_BORDER_COLOR
    );
    assert_ne!(
        cell_fg_at_text(&terminal, "Rogue"),
        SELECTED_CONTAINER_BORDER_COLOR
    );
}

#[test]
fn inventory_cell_spans_use_rarity_outline_and_focus_label() {
    use ratatui::style::Style;

    let mut rare_sword = rusted_sword();
    rare_sword.rarity = Rarity::Rare;
    let mut magic_axe = crude_axe();
    magic_axe.rarity = Rarity::Magic;
    let grid = ItemGrid::new(2, 2, vec![rare_sword, magic_axe]);

    let rare_selected = inventory_cell_spans(&grid, 0, true);
    assert_eq!(rare_selected[0].content.as_ref(), GRID_OPEN_GLYPH);
    assert_eq!(
        rare_selected[0].style,
        Style::default().fg(RARITY_RARE_COLOR)
    );
    assert_eq!(rare_selected[1].content.as_ref(), WEAPON_GLYPH);
    assert_eq!(rare_selected[1].style, selected_cursor_style());
    assert_eq!(rare_selected[2].content.as_ref(), GRID_CLOSE_GLYPH);
    assert_eq!(
        rare_selected[2].style,
        Style::default().fg(RARITY_RARE_COLOR)
    );

    let magic_unselected = inventory_cell_spans(&grid, 1, false);
    assert_eq!(
        magic_unselected[0].style,
        Style::default().fg(RARITY_MAGIC_COLOR)
    );
    assert_eq!(magic_unselected[1].style, body_style());
    assert_eq!(
        magic_unselected[2].style,
        Style::default().fg(RARITY_MAGIC_COLOR)
    );

    let empty_selected = inventory_cell_spans(&grid, 2, true);
    assert_eq!(
        empty_selected
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>(),
        vec![GRID_OPEN_GLYPH, EMPTY_CELL_GLYPH, GRID_CLOSE_GLYPH]
    );
    assert!(
        empty_selected
            .iter()
            .all(|span| { span.style == selected_cursor_style() })
    );
}

#[test]
fn command_and_stat_text_use_gothic_semantic_colors() {
    use ratatui::style::{Modifier, Style};

    let commands = command_line("Town", &[("m", "merchant"), ("q", "save+quit")]);
    assert_eq!(commands.spans[0].style, title_style());
    assert_eq!(
        commands.spans[1].style,
        Style::default()
            .fg(ACTION_COLOR)
            .add_modifier(Modifier::BOLD)
    );
    assert_eq!(
        commands.spans[4].style,
        Style::default()
            .fg(DANGER_COLOR)
            .add_modifier(Modifier::BOLD)
    );

    let stat = stat_span("Gold 25", WARNING_COLOR);
    assert_eq!(
        stat.style,
        Style::default()
            .fg(WARNING_COLOR)
            .add_modifier(Modifier::BOLD)
    );
}

#[test]
fn commands_footer_colors_selectable_keys() {
    use ratatui::style::{Modifier, Style};

    let lines = command_footer_lines("Dropped item.\nW/S or arrows=select  Enter=choose  Esc=back");

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].spans[0].content.as_ref(), "Dropped item.");
    assert_eq!(lines[0].spans[0].style, Style::default().fg(WARNING_COLOR));

    let command_spans = &lines[1].spans;
    assert_eq!(command_spans[0].content.as_ref(), "W/S");
    assert_eq!(
        command_spans[0].style,
        Style::default()
            .fg(ACTION_COLOR)
            .add_modifier(Modifier::BOLD)
    );
    assert_eq!(command_spans[2].content.as_ref(), "arrows");
    assert_eq!(
        command_spans[2].style,
        Style::default()
            .fg(ACTION_COLOR)
            .add_modifier(Modifier::BOLD)
    );
    assert_eq!(command_spans[5].content.as_ref(), "Enter");
    assert_eq!(
        command_spans[5].style,
        Style::default()
            .fg(ACTION_COLOR)
            .add_modifier(Modifier::BOLD)
    );
    assert_eq!(command_spans[8].content.as_ref(), "Esc");
    assert_eq!(
        command_spans[8].style,
        Style::default()
            .fg(DANGER_COLOR)
            .add_modifier(Modifier::BOLD)
    );
}

#[test]
fn selected_item_detail_lines_empty_cell_uses_passed_grid_label_and_capacity() {
    let c = test_character();
    let lines = selected_item_detail_lines(&c, &c.stash, "Stash", None);
    let text = lines.iter().map(line_text).collect::<Vec<_>>();

    assert_eq!(
        text,
        vec!["Empty cell".to_string(), "Stash: 0/64".to_string()]
    );
}

#[test]
fn selected_item_detail_lines_strip_ansi_from_rendered_text() {
    let c = test_character();
    let axe = crude_axe();
    let gem = gem_item(GemKind::Topaz, GemTier::Flawed);

    for item in [&axe, &gem] {
        let lines = selected_item_detail_lines(&c, &c.inventory, "Bag", Some(item));
        let text = lines.iter().map(line_text).collect::<Vec<_>>();
        assert!(text.iter().all(|line| !line.contains('\u{1b}')));
    }
}

#[test]
fn equipped_comparison_lines_show_current_slot_and_color_deltas() {
    use ratatui::style::{Color, Style};

    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(item_with_rarity(
        "Test Hauberk",
        ItemKind::Armor,
        20,
        item_stats(0, 0, 3, 2, -1),
        Rarity::Magic,
        1,
        requirements(0, 0, 0),
    ));

    let lines = selected_item_equipped_comparison_lines(&c, c.inventory.get(0));
    let text = lines.iter().map(line_text).collect::<Vec<_>>();

    assert!(
        text.iter()
            .any(|line| line == "Equipped Armor: Cloth Tunic")
    );
    assert!(
        text.iter()
            .any(|line| line == "Armor 1 | dodge 0 | speed 0")
    );

    let delta_line = lines
        .iter()
        .find(|line| line_text(line).starts_with("Delta:"))
        .unwrap();
    assert!(delta_line.spans.iter().any(|span| {
        span.content.as_ref() == "+2 armor" && span.style == Style::default().fg(Color::Green)
    }));
    assert!(delta_line.spans.iter().any(|span| {
        span.content.as_ref() == "-1 speed" && span.style == Style::default().fg(Color::Red)
    }));
}

#[test]
fn inventory_render_lines_include_grid_capacity_selected_details_and_equipped_comparison() {
    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(crude_axe());
    let lines = inventory_screen_text_for_test(
        &c,
        0,
        CharacterEquipmentSlot::Weapon,
        InventoryFocus::Bag,
        "",
    );
    let rendered = lines.join("\n");

    assert!(rendered.contains("Inventory - Bag 4 x 4 - 1 / 16"));
    assert!(rendered.contains("⟦W⟧"));
    assert!(rendered.contains("Crude Axe"));
    assert!(rendered.contains("Equipped Weapon: Rusted Sword"));
    assert!(rendered.contains("Delta: +2 damage  crit -3"));
    assert!(rendered.contains("WASD/Arrows=move  Enter=equip/use  o=sort  x=drop  Esc=back"));
}

#[test]
fn inventory_render_lines_include_message_and_full_commands() {
    let c = test_character();
    let lines = inventory_screen_text_for_test(
        &c,
        0,
        CharacterEquipmentSlot::Weapon,
        InventoryFocus::Bag,
        "Dropped Lesser Health Potion.",
    );
    let rendered = lines.join("\n");

    assert!(rendered.contains("Dropped Lesser Health Potion."));
    assert!(rendered.contains("WASD/Arrows=move  Enter=equip/use  o=sort  x=drop  Esc=back"));
}

#[test]
fn inventory_text_includes_character_equipment_panel_and_tab_command() {
    let c = test_character();
    let lines = inventory_screen_text_for_test(
        &c,
        0,
        CharacterEquipmentSlot::Armor,
        InventoryFocus::Bag,
        "",
    );
    let rendered = lines.join("\n");

    assert!(rendered.contains("Character"));
    assert!(rendered.contains("Helm"));
    assert!(rendered.contains("Weapon"));
    assert!(rendered.contains("Armor"));
    assert!(rendered.contains("Rusted Sword"));
    assert!(rendered.contains("Cloth Tunic"));
    assert!(rendered.contains("Tab=switch"));
}

#[test]
fn character_focused_inventory_details_show_selected_equipped_item() {
    let c = test_character();
    let lines = inventory_screen_text_for_test(
        &c,
        0,
        CharacterEquipmentSlot::Shield,
        InventoryFocus::Character,
        "",
    );
    let rendered = lines.join("\n");

    assert!(rendered.contains("Selected Shield"));
    assert!(rendered.contains("Worn Shield"));
    assert!(rendered.contains("Armor 1 | dodge 2 | speed 0"));
}

#[test]
fn stash_render_lines_include_both_grid_capacities() {
    let c = test_character();
    let lines = stash_screen_text_for_test(&c, StashSide::Inventory, 0, 0, "");
    let rendered = lines.join("\n");

    assert!(rendered.contains("Stash - Inventory 3/16 - Stash 0/64"));
    assert!(rendered.contains("Inventory ✦"));
    assert!(rendered.contains("Tab=switch"));
}

#[test]
fn stash_render_80_columns_shows_both_grids_details_message_and_commands() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| render_stash_screen(frame, &c, StashSide::Stash, 0, 0, "Stored item."))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Stash - Inventory 3/16 - Stash 0/64"));
    assert!(rendered.contains("Inventory"));
    assert!(rendered.contains("Stash ✦"));
    assert!(rendered.contains("⟦H⟧"));
    assert!(rendered.contains("⟦M⟧"));
    assert!(rendered.contains("⟦·⟧ ⟦·⟧ ⟦·⟧ ⟦·⟧ ⟦·⟧ ⟦·⟧ ⟦·⟧ ⟦·⟧"));
    assert!(rendered.contains("Empty cell"));
    assert!(rendered.contains("Stored item."));
    assert!(rendered.contains("Tab=switch  WASD/Arrows=move  Enter=transfer  Esc=back"));
}

#[test]
fn wide_stash_render_keeps_stash_grid_content_sized() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(120, 24)).unwrap();

    terminal
        .draw(|frame| render_stash_screen(frame, &c, StashSide::Stash, 0, 0, ""))
        .unwrap();
    let lines = backend_lines(&terminal);
    let body_top = &lines[3];

    let stash_title_x = char_index(body_top, "Stash ✦");
    let details_title_x = char_index(body_top, "Details");

    assert_eq!(details_title_x - stash_title_x, 34);
    assert!(details_title_x <= 60);
}

#[test]
fn inventory_render_footer_shows_message_and_commands() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| render_inventory_screen(frame, &c, 0, "Dropped Lesser Health Potion."))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Dropped Lesser Health Potion."));
    assert!(rendered.contains("WASD/Arrows=move  Enter=equip/use  o=sort  x=drop  Esc=back"));
}

#[test]
fn wide_inventory_render_keeps_bag_grid_content_sized_with_character_panel_to_the_right() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(120, 24)).unwrap();

    terminal
        .draw(|frame| render_inventory_screen(frame, &c, 0, ""))
        .unwrap();
    let lines = backend_lines(&terminal);
    let body_top = &lines[3];

    let bag_title_x = char_index(body_top, "Bag");
    let details_title_x = char_index(body_top, "Details");
    let character_title_x = char_index(body_top, "Character");

    assert_eq!(
        details_title_x - bag_title_x,
        usize::from(item_grid_render_width(&c.inventory))
    );
    assert!(details_title_x <= 24);
    assert!(character_title_x > details_title_x);
}

#[test]
fn character_select_screen_lists_saved_characters_and_new_character_row() {
    use ratatui::{Terminal, backend::TestBackend};

    let summaries = vec![
        CharacterSummary {
            id: "mara".to_string(),
            name: "Mara".to_string(),
            class_name: "Warrior".to_string(),
            level: 8,
            death_mode: DeathMode::Softcore,
            location: "Town".to_string(),
        },
        CharacterSummary {
            id: "shade".to_string(),
            name: "Shade".to_string(),
            class_name: "Rogue".to_string(),
            level: 3,
            death_mode: DeathMode::Hardcore,
            location: "Dungeon L4".to_string(),
        },
    ];
    let mut terminal = Terminal::new(TestBackend::new(80, 18)).unwrap();
    terminal
        .draw(|frame| render_character_select_screen(frame, &summaries, "shade", 1, ""))
        .unwrap();
    let body = backend_text(&terminal);

    assert!(body.contains("Characters"));
    assert!(body.contains("Mara"));
    assert!(body.contains("Warrior"));
    assert!(body.contains("Lv 8"));
    assert!(body.contains("Shade"));
    assert!(body.contains("Hardcore"));
    assert!(body.contains("Dungeon L4"));
    assert!(body.contains("+ New Character"));
    assert!(body.contains("n=new"));
    assert!(body.contains("Esc=back"));
}

#[test]
fn character_creation_renders_as_stepped_ratatui_screen() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                CharacterCreationStep::DeathMode,
                "Mara",
                CharacterClass::Warrior,
                DeathMode::Hardcore,
                "",
            )
        })
        .unwrap();

    let rendered = backend_text(&terminal);
    assert!(rendered.contains("Character Creation"));
    assert!(!rendered.contains("CrawlTTY"));
    assert!(rendered.contains("Step 1: Class"));
    assert!(rendered.contains("Step 2: Name"));
    assert!(rendered.contains("Step 3: Death Mode"));
    assert!(rendered.contains("Name: Mara"));
    assert!(rendered.contains("› Hardcore"));
    assert!(rendered.contains("Up/Down or Tab=mode"));
    assert!(rendered.contains("Enter=confirm"));
    assert!(!rendered.contains("S/H"));
}

#[test]
fn town_service_screens_render_with_ratatui() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.herbs = 7;
    complete_project_for_test(&mut c, TownProject::Distillery);
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();

    terminal
        .draw(|frame| render_merchant_screen(frame, &c, 0, ""))
        .unwrap();
    let merchant = backend_text(&terminal);
    assert!(merchant.contains("Merchant"));
    assert!(merchant.contains("› Buy Lesser Health Potion - 50 gold"));
    assert!(merchant.contains("Buy Lesser Mana Potion - 100 gold"));
    assert!(merchant.contains("Sell items"));

    terminal
        .draw(|frame| render_blacksmith_screen(frame, &c, 4, ""))
        .unwrap();
    let blacksmith = backend_text(&terminal);
    assert!(blacksmith.contains("Blacksmith"));
    assert!(blacksmith.contains("› Manage sockets"));

    terminal
        .draw(|frame| render_town_projects_screen(frame, &c, 0, ""))
        .unwrap();
    let projects = backend_text(&terminal);
    assert!(projects.contains("Town Projects"));
    assert!(projects.contains("Enter=fund project"));

    terminal
        .draw(|frame| render_distillery_screen(frame, &c, 0, ""))
        .unwrap();
    let distillery = backend_text(&terminal);
    assert!(distillery.contains("Distillery"));
    assert!(distillery.contains("Herbs: 7"));
    assert!(distillery.contains("› Craft Lesser Health Potion - 3 herbs"));
    assert!(distillery.contains("Craft Lesser Mana Potion - 4 herbs"));

    terminal
        .draw(|frame| render_spend_attributes_screen(frame, &c, 0, ""))
        .unwrap();
    let attributes = backend_text(&terminal);
    assert!(attributes.contains("Spend Attributes"));
    assert!(attributes.contains("Strength"));
}

#[test]
fn merchant_sells_lesser_health_and_mana_potions() {
    let mut c = test_character();
    c.gold = HEALTH_POTION_COST + MANA_POTION_COST;
    let starting_inventory = c.inventory.len();

    let message = buy_merchant_offer(&mut c, MerchantOffer::LesserHealthPotion);

    assert_eq!(message, "Bought Lesser Health Potion for 50 gold.");
    assert_eq!(c.gold, MANA_POTION_COST);
    assert_eq!(c.inventory.len(), starting_inventory + 1);
    assert!(matches!(
        c.inventory.items.last().map(|item| item.kind),
        Some(ItemKind::HealthPotion)
    ));

    let message = buy_merchant_offer(&mut c, MerchantOffer::LesserManaPotion);

    assert_eq!(message, "Bought Lesser Mana Potion for 100 gold.");
    assert_eq!(c.gold, 0);
    assert_eq!(c.inventory.len(), starting_inventory + 2);
    assert!(matches!(
        c.inventory.items.last().map(|item| item.kind),
        Some(ItemKind::ManaPotion)
    ));
}

#[test]
fn merchant_purchase_failures_do_not_spend_gold() {
    let mut c = test_character();
    c.gold = HEALTH_POTION_COST - 1;
    let starting_inventory = c.inventory.len();

    let message = buy_merchant_offer(&mut c, MerchantOffer::LesserHealthPotion);

    assert_eq!(message, "Need 50 gold to buy Lesser Health Potion.");
    assert_eq!(c.gold, HEALTH_POTION_COST - 1);
    assert_eq!(c.inventory.len(), starting_inventory);

    c.gold = MANA_POTION_COST;
    c.inventory = ItemGrid::new(1, 1, vec![health_potion()]);

    let message = buy_merchant_offer(&mut c, MerchantOffer::LesserManaPotion);

    assert_eq!(message, "No room in inventory.");
    assert_eq!(c.gold, MANA_POTION_COST);
    assert_eq!(c.inventory.len(), 1);
    assert!(matches!(c.inventory[0].kind, ItemKind::HealthPotion));
}

#[test]
fn rogue_cannot_buy_or_use_mana_potions() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.gold = MANA_POTION_COST;
    let starting_gold = c.gold;
    let starting_inventory = c.inventory.len();

    let message = buy_merchant_offer(&mut c, MerchantOffer::LesserManaPotion);

    assert_eq!(message, "Rogue uses Energy and cannot use mana potions.");
    assert_eq!(c.gold, starting_gold);
    assert_eq!(c.inventory.len(), starting_inventory);

    c.inventory.push(mana_potion());
    let mana_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::ManaPotion))
        .unwrap();
    let result = equip_or_use_inventory_item(&mut c, mana_index);

    assert_eq!(
        result.message,
        "Rogue uses Energy and cannot use mana potions."
    );
    assert!(!result.spent_turn);
    assert!(
        c.inventory
            .iter()
            .any(|item| matches!(item.kind, ItemKind::ManaPotion))
    );
}

#[test]
fn distillery_crafts_potions_from_herbs() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::Distillery);
    c.herbs = 7;
    c.inventory = ItemGrid::new(4, 4, Vec::new());

    let message = craft_distillery_recipe(&mut c, DistilleryRecipe::LesserHealthPotion);

    assert_eq!(message, "Crafted Lesser Health Potion for 3 herbs.");
    assert_eq!(c.herbs, 4);
    assert!(matches!(c.inventory[0].kind, ItemKind::HealthPotion));

    let message = craft_distillery_recipe(&mut c, DistilleryRecipe::LesserManaPotion);

    assert_eq!(message, "Crafted Lesser Mana Potion for 4 herbs.");
    assert_eq!(c.herbs, 0);
    assert!(matches!(c.inventory[1].kind, ItemKind::ManaPotion));
}

#[test]
fn distillery_recipe_failures_do_not_spend_herbs() {
    let mut c = test_character();
    c.herbs = 10;

    let message = craft_distillery_recipe(&mut c, DistilleryRecipe::LesserHealthPotion);

    assert_eq!(
        message,
        "Complete the Distillery project before crafting potions."
    );
    assert_eq!(c.herbs, 10);

    complete_project_for_test(&mut c, TownProject::Distillery);
    c.herbs = 2;

    let message = craft_distillery_recipe(&mut c, DistilleryRecipe::LesserHealthPotion);

    assert_eq!(message, "Need 3 herbs to craft Lesser Health Potion.");
    assert_eq!(c.herbs, 2);

    c.herbs = 3;
    c.inventory = ItemGrid::new(1, 1, vec![health_potion()]);

    let message = craft_distillery_recipe(&mut c, DistilleryRecipe::LesserHealthPotion);

    assert_eq!(message, "No room in inventory.");
    assert_eq!(c.herbs, 3);
}

#[test]
fn rogues_do_not_craft_mana_potions() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    complete_project_for_test(&mut c, TownProject::Distillery);
    c.herbs = 4;

    let message = craft_distillery_recipe(&mut c, DistilleryRecipe::LesserManaPotion);

    assert_eq!(message, "Rogue uses Energy and cannot craft mana potions.");
    assert_eq!(c.herbs, 4);
    assert!(
        !c.inventory
            .iter()
            .any(|item| matches!(item.kind, ItemKind::ManaPotion))
    );
}

#[test]
fn rogue_random_consumable_loot_does_not_drop_mana_potions() {
    for _ in 0..200 {
        let loot = random_loot_for_class(CharacterClass::Rogue, 2, false);
        if matches!(loot.kind, ItemKind::HealthPotion | ItemKind::ManaPotion) {
            assert_eq!(loot.kind, ItemKind::HealthPotion);
        }
    }
}

#[test]
fn rogue_random_equipment_loot_uses_rogue_item_families() {
    let mut saw_dagger = false;
    let mut saw_scimitar = false;
    let mut saw_light_armor = false;
    let mut saw_buckler = false;

    for _ in 0..2000 {
        let loot = random_equipment_loot_for_class(CharacterClass::Rogue, 3, false);
        let is_dagger = loot.kind == ItemKind::Weapon && loot.name.contains("Dagger");
        let is_scimitar = loot.kind == ItemKind::Weapon && loot.name.contains("Scimitar");
        let is_light_armor = loot.kind == ItemKind::Armor && loot.name.contains("Leathers");
        let is_buckler = loot.kind == ItemKind::Shield && loot.name.contains("Buckler");
        let is_cowl = loot.kind == ItemKind::Helm && loot.name.contains("Cowl");
        let is_gloves = loot.kind == ItemKind::Gloves && loot.name.contains("Gloves");
        let is_boots = loot.kind == ItemKind::Boots && loot.name.contains("Boots");
        let is_belt = loot.kind == ItemKind::Belt && loot.name.contains("Belt");
        let is_amulet = loot.kind == ItemKind::Amulet && loot.name.contains("Amulet");
        let is_ring = loot.kind == ItemKind::Ring && loot.name.contains("Ring");

        assert!(
            is_dagger
                || is_scimitar
                || is_light_armor
                || is_buckler
                || is_cowl
                || is_gloves
                || is_boots
                || is_belt
                || is_amulet
                || is_ring,
            "unexpected Rogue equipment drop: {}",
            loot.name
        );

        saw_dagger |= is_dagger;
        saw_scimitar |= is_scimitar;
        saw_light_armor |= is_light_armor;
        saw_buckler |= is_buckler;
        assert_eq!(loot.required_intelligence, 0);
    }

    assert!(saw_dagger);
    assert!(saw_scimitar);
    assert!(saw_light_armor);
    assert!(saw_buckler);
}

#[test]
fn warrior_random_equipment_loot_uses_warrior_item_families() {
    for _ in 0..2000 {
        let loot = random_equipment_loot_for_class(CharacterClass::Warrior, 3, false);
        let is_sword = loot.kind == ItemKind::Weapon && loot.name.contains("Sword");
        let is_axe = loot.kind == ItemKind::Weapon && loot.name.contains("Axe");
        let is_mail = loot.kind == ItemKind::Armor && loot.name.contains("Mail");
        let is_guard_shield = loot.kind == ItemKind::Shield && loot.name.contains("Guard Shield");
        let is_helm = loot.kind == ItemKind::Helm && loot.name.contains("Helm");
        let is_gloves = loot.kind == ItemKind::Gloves && loot.name.contains("Gloves");
        let is_boots = loot.kind == ItemKind::Boots && loot.name.contains("Boots");
        let is_belt = loot.kind == ItemKind::Belt && loot.name.contains("Belt");
        let is_amulet = loot.kind == ItemKind::Amulet && loot.name.contains("Amulet");
        let is_ring = loot.kind == ItemKind::Ring && loot.name.contains("Ring");

        assert!(
            is_sword
                || is_axe
                || is_mail
                || is_guard_shield
                || is_helm
                || is_gloves
                || is_boots
                || is_belt
                || is_amulet
                || is_ring,
            "unexpected Warrior equipment drop: {}",
            loot.name
        );
    }
}

#[test]
fn sorceress_random_equipment_uses_wand_focus_pool_without_staves() {
    let mut seen_names = std::collections::HashSet::new();
    for _ in 0..300 {
        let loot = random_equipment_loot_for_class(CharacterClass::Sorceress, 3, false);
        seen_names.insert(loot.name);
    }

    assert!(seen_names.iter().any(|name| name.contains("Wand")));
    assert!(seen_names.iter().any(|name| name.contains("Focus")));
    assert!(seen_names.iter().any(|name| name.contains("Robe")));
    assert!(seen_names.iter().any(|name| name.contains("Circlet")));
    assert!(seen_names.iter().any(|name| name.contains("Spell Gloves")));
    assert!(seen_names.iter().any(|name| name.contains("Soft Slippers")));
    assert!(seen_names.iter().any(|name| name.contains("Sash")));
    assert!(seen_names.iter().any(|name| name.contains("Arcane Amulet")));
    assert!(seen_names.iter().any(|name| name.contains("Rune Ring")));
    assert!(!seen_names.iter().any(|name| name.contains("Staff")));
    assert!(!seen_names.iter().any(|name| name.contains("Dagger")));
    assert!(!seen_names.iter().any(|name| name.contains("Sword")));
    assert!(!seen_names.iter().any(|name| name.contains("Axe")));
}

#[test]
fn sorceress_can_equip_wands_and_focuses_but_not_other_weapons_or_shields() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.strength = 99;
    c.dexterity = 99;
    c.intelligence = 99;

    assert!(can_equip_item(&c, &cracked_wand()));
    assert!(can_equip_item(&c, &cracked_focus()));
    assert!(!can_equip_item(&c, &training_dagger()));
    assert!(!can_equip_item(&c, &worn_shield()));

    c.inventory.push(training_dagger());
    let dagger_index = c
        .inventory
        .iter()
        .position(|item| item.name.contains("Dagger"))
        .unwrap();
    let result = equip_or_use_inventory_item(&mut c, dagger_index);

    assert_eq!(
        result.message,
        "Sorceress can equip wands and focuses only in weapon/offhand slots."
    );
    assert!(!result.spent_turn);
    assert!(c.equipped_weapon.name.contains("Wand"));
}

#[test]
fn random_equipment_loot_can_drop_new_equipment_slots() {
    let mut seen = std::collections::HashSet::new();
    for _ in 0..1000 {
        seen.insert(random_equipment_loot_for_class(CharacterClass::Warrior, 3, false).kind);
        seen.insert(random_equipment_loot_for_class(CharacterClass::Rogue, 3, false).kind);
    }

    for kind in [
        ItemKind::Helm,
        ItemKind::Gloves,
        ItemKind::Boots,
        ItemKind::Belt,
        ItemKind::Amulet,
        ItemKind::Ring,
    ] {
        assert!(seen.contains(&kind), "expected loot pool to drop {kind:?}");
    }
}

#[test]
fn rogue_can_equip_bucklers_but_not_warrior_shields() {
    let c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    let mut buckler = None;
    let mut guard_shield = None;

    for _ in 0..2000 {
        let rogue_loot = random_equipment_loot_for_class(CharacterClass::Rogue, 1, false);
        if rogue_loot.name.contains("Buckler") {
            buckler = Some(rogue_loot);
        }

        let warrior_loot = random_equipment_loot_for_class(CharacterClass::Warrior, 1, false);
        if warrior_loot.name.contains("Guard Shield") {
            guard_shield = Some(warrior_loot);
        }

        if buckler.is_some() && guard_shield.is_some() {
            break;
        }
    }

    let buckler = buckler.expect("expected Rogue loot pool to produce a buckler");
    let guard_shield = guard_shield.expect("expected Warrior loot pool to produce a guard shield");

    assert!(can_equip_item(&c, &buckler));
    assert!(!can_equip_item(&c, &guard_shield));
}

#[test]
fn attributes_screen_with_no_points_shows_empty_state_and_back_command() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();

    terminal
        .draw(|frame| render_spend_attributes_screen(frame, &c, 0, ""))
        .unwrap();
    let attributes = backend_text(&terminal);

    assert!(attributes.contains("Spend Attributes (0 left)"));
    assert!(attributes.contains("No unspent attribute points."));
    assert!(attributes.contains("Esc=back"));
}

#[test]
fn town_footer_lists_character_switch_command() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| render_town(frame, &c, "")).unwrap();
    let body = backend_text(&terminal);

    assert!(body.contains("c=characters"));
}

#[test]
fn town_equipment_lines_show_empty_slots_as_nothing_equipped() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();

    terminal.draw(|frame| render_town(frame, &c, "")).unwrap();
    let town = backend_text(&terminal);

    assert!(town.contains("Helm  : Nothing equipped"));
    assert!(town.contains("Ring 1: Nothing equipped"));
    assert!(!town.contains("Empty Helm"));
    assert!(!town.contains("Empty Ring"));
}

#[test]
fn attributes_screen_uses_cursor_selection_and_attribute_colors() {
    use ratatui::{Terminal, backend::TestBackend, style::Color};

    let mut c = test_character();
    c.unspent_attributes = 3;
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();

    terminal
        .draw(|frame| render_spend_attributes_screen(frame, &c, 0, ""))
        .unwrap();
    let attributes = backend_text(&terminal);

    assert!(attributes.contains("› 1) Strength"));
    assert!(attributes.contains("2) Dexterity 3 → 4 (+10 hit)"));
    assert!(attributes.contains("W/S or arrows=select"));
    assert!(attributes.contains("Enter=spend"));
    assert_eq!(cell_fg_at_text(&terminal, "Strength"), Color::Red);
    assert_eq!(cell_fg_at_text(&terminal, "Dexterity"), Color::Green);
    assert_eq!(cell_fg_at_text(&terminal, "Intelligence"), Color::Blue);
}

#[test]
fn town_and_inventory_containers_use_gothic_borders_and_titles() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut town_terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();
    town_terminal
        .draw(|frame| render_town(frame, &c, ""))
        .unwrap();
    assert_eq!(cell_fg_at(&town_terminal, 0, 0), CONTAINER_BORDER_COLOR);
    assert!(text_has_fg_at_any_occurrence(
        &town_terminal,
        "Town",
        TITLE_COLOR
    ));
    assert!(text_has_fg_at_any_occurrence(
        &town_terminal,
        "Status",
        TITLE_COLOR
    ));
    assert!(text_has_fg_at_any_occurrence(
        &town_terminal,
        "Commands",
        TITLE_COLOR
    ));
    assert!(!backend_text(&town_terminal).contains("CrawlTTY"));

    let mut inventory_terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();
    inventory_terminal
        .draw(|frame| render_inventory_screen(frame, &c, 0, ""))
        .unwrap();
    assert_eq!(
        cell_fg_at(&inventory_terminal, 0, 0),
        CONTAINER_BORDER_COLOR
    );
    assert!(text_has_fg_at_any_occurrence(
        &inventory_terminal,
        "Inventory",
        TITLE_COLOR
    ));
    assert!(text_has_fg_at_any_occurrence(
        &inventory_terminal,
        "Bag",
        TITLE_COLOR
    ));
    assert!(text_has_fg_at_any_occurrence(
        &inventory_terminal,
        "Details",
        TITLE_COLOR
    ));
    assert!(text_has_fg_at_any_occurrence(
        &inventory_terminal,
        "Character",
        TITLE_COLOR
    ));
}

#[test]
fn dungeon_containers_use_gothic_borders_and_titles() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();
    terminal.draw(|frame| render_dungeon(frame, &c)).unwrap();

    assert_eq!(cell_fg_at(&terminal, 0, 0), CONTAINER_BORDER_COLOR);
    assert!(text_has_fg_at_any_occurrence(
        &terminal,
        "Dungeon",
        TITLE_COLOR
    ));
    assert!(text_has_fg_at_any_occurrence(&terminal, "Map", TITLE_COLOR));
    assert!(text_has_fg_at_any_occurrence(
        &terminal,
        "Combat Log",
        TITLE_COLOR
    ));
    assert!(text_has_fg_at_any_occurrence(
        &terminal,
        "Skills",
        TITLE_COLOR
    ));
    assert!(text_has_fg_at_any_occurrence(
        &terminal,
        "Commands",
        TITLE_COLOR
    ));
}

#[test]
fn dungeon_loot_log_item_names_use_rarity_colors() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut dungeon = open_test_dungeon(2, 2, Vec::new());
    dungeon.log.push("== Turn 1: Test loot ==".to_string());
    let loot = [
        item_with_rarity(
            "Plain Blade",
            ItemKind::Weapon,
            10,
            weapon_stats(1, 2, 0, SWORD_CRIT_CHANCE),
            Rarity::Common,
            1,
            requirements(0, 0, 0),
        ),
        item_with_rarity(
            "Magic Wand",
            ItemKind::Weapon,
            12,
            weapon_stats(1, 3, 0, WAND_CRIT_CHANCE),
            Rarity::Magic,
            1,
            requirements(0, 0, 0),
        ),
        item_with_rarity(
            "Rare Axe",
            ItemKind::Weapon,
            14,
            weapon_stats(2, 4, 0, AXE_CRIT_CHANCE),
            Rarity::Rare,
            1,
            requirements(0, 0, 0),
        ),
    ];
    for item in &loot {
        log_event(
            &mut dungeon.log,
            LogKind::Loot,
            format!("Loot found: {}.", colored_item_name(item)),
        );
    }

    let mut c = test_character();
    c.active_dungeon = Some(dungeon);
    let mut terminal = Terminal::new(TestBackend::new(120, 32)).unwrap();
    terminal.draw(|frame| render_dungeon(frame, &c)).unwrap();

    assert_eq!(
        cell_fg_at_text(&terminal, "Plain Blade"),
        rarity_color(&Rarity::Common)
    );
    assert_eq!(
        cell_fg_at_text(&terminal, "Magic Wand"),
        rarity_color(&Rarity::Magic)
    );
    assert_eq!(
        cell_fg_at_text(&terminal, "Rare Axe"),
        rarity_color(&Rarity::Rare)
    );
}

#[test]
fn character_creation_active_step_uses_muted_cursed_violet_border() {
    use ratatui::{Terminal, backend::TestBackend, style::Color};

    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                CharacterCreationStep::Name,
                "",
                CharacterClass::Warrior,
                DeathMode::Softcore,
                "",
            )
        })
        .unwrap();

    let cursed_violet = Color::Rgb(148, 80, 190);
    assert_eq!(cell_fg_at(&terminal, 0, 8), cursed_violet);
    assert_ne!(cell_fg_at(&terminal, 0, 3), cursed_violet);
}

#[test]
fn character_creation_only_shows_cursor_for_active_choice_step() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                CharacterCreationStep::Name,
                "",
                CharacterClass::Warrior,
                DeathMode::Softcore,
                "",
            )
        })
        .unwrap();
    let name_step = backend_text(&terminal);
    assert_eq!(name_step.matches(SELECTION_CURSOR).count(), 0);

    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                CharacterCreationStep::Class,
                "",
                CharacterClass::Warrior,
                DeathMode::Softcore,
                "",
            )
        })
        .unwrap();
    let class_step = backend_text(&terminal);
    assert_eq!(class_step.matches(SELECTION_CURSOR).count(), 1);
    assert!(class_step.contains("› Warrior"));
    assert!(!class_step.contains("› Softcore"));

    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                CharacterCreationStep::DeathMode,
                "",
                CharacterClass::Warrior,
                DeathMode::Softcore,
                "",
            )
        })
        .unwrap();
    let death_mode_step = backend_text(&terminal);
    assert_eq!(death_mode_step.matches(SELECTION_CURSOR).count(), 1);
    assert!(!death_mode_step.contains("› Warrior"));
    assert!(death_mode_step.contains("› Softcore"));
}

#[test]
fn character_creation_selected_container_titles_do_not_show_active_text() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                CharacterCreationStep::Name,
                "",
                CharacterClass::Warrior,
                DeathMode::Softcore,
                "",
            )
        })
        .unwrap();

    let screen = backend_text(&terminal);
    assert!(screen.contains("Step 2: Name"));
    assert!(!screen.contains("active"));
}

#[test]
fn active_stash_grid_uses_muted_cursed_violet_border() {
    use ratatui::{Terminal, backend::TestBackend, style::Color};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(120, 28)).unwrap();

    terminal
        .draw(|frame| render_stash_screen(frame, &c, StashSide::Stash, 0, 0, ""))
        .unwrap();

    let cursed_violet = Color::Rgb(148, 80, 190);
    assert_ne!(cell_fg_at(&terminal, 0, 3), cursed_violet);
    assert_eq!(cell_fg_at(&terminal, 18, 3), cursed_violet);
}

#[test]
fn stash_only_shows_cursor_on_active_grid() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.inventory = ItemGrid::new(4, 4, vec![rusted_sword()]);
    c.stash = ItemGrid::new(8, 8, vec![crude_axe()]);
    let mut terminal = Terminal::new(TestBackend::new(120, 28)).unwrap();

    terminal
        .draw(|frame| render_stash_screen(frame, &c, StashSide::Stash, 0, 0, ""))
        .unwrap();

    assert_eq!(
        cell_fg_at(&terminal, 20, 4),
        SELECTED_CONTAINER_BORDER_COLOR
    );
    assert_eq!(cell_fg_at(&terminal, 2, 4), TEXT_PRIMARY_COLOR);
}

#[test]
fn inventory_adjacent_screens_render_with_ratatui() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.inventory.push(rusted_sword());
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();

    terminal
        .draw(|frame| render_sell_items_screen(frame, &c, 0, ""))
        .unwrap();
    let sell = backend_text(&terminal);
    assert!(sell.contains("Sell Items"));
    assert!(sell.contains("Sell value"));

    terminal
        .draw(|frame| render_salvage_screen(frame, &c, 3, ""))
        .unwrap();
    let salvage = backend_text(&terminal);
    assert!(salvage.contains("Salvage Gear"));
    assert!(salvage.contains("Salvage yield"));

    terminal
        .draw(|frame| render_socket_bench_screen(frame, &c, 0, 0, ""))
        .unwrap();
    let sockets = backend_text(&terminal);
    assert!(sockets.contains("Socket Bench"));
    assert!(sockets.contains("Socketed Gear"));

    terminal
        .draw(|frame| render_gem_picker_screen(frame, &c, 0, ""))
        .unwrap();
    let gems = backend_text(&terminal);
    assert!(gems.contains("Select Gem"));
    assert!(gems.contains("No gems in inventory."));
}

#[test]
fn merchant_sell_screen_uses_inventory_grid_layout() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.inventory = ItemGrid::new(4, 4, vec![rusted_sword()]);
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();

    terminal
        .draw(|frame| render_sell_items_screen(frame, &c, 0, ""))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Bag"));
    assert!(rendered.contains("⟦W⟧"));
    assert!(rendered.contains("Details"));
    assert!(rendered.contains("Equipped"));
    assert!(rendered.contains("Sell value:"));
    assert!(!rendered.contains("› Rusted Sword"));
}

#[test]
fn gem_picker_scroll_keeps_high_selected_gem_visible() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    let gem_kinds = [
        GemKind::Ruby,
        GemKind::Sapphire,
        GemKind::Garnet,
        GemKind::Emerald,
        GemKind::Amethyst,
        GemKind::Quartz,
        GemKind::Jade,
        GemKind::Onyx,
        GemKind::Citrine,
        GemKind::Topaz,
        GemKind::Opal,
        GemKind::Bloodstone,
    ];
    c.inventory = ItemGrid::new(
        4,
        4,
        gem_kinds
            .into_iter()
            .map(|kind| gem_item(kind, GemTier::Flawed))
            .collect(),
    );
    let mut terminal = Terminal::new(TestBackend::new(80, 16)).unwrap();

    terminal
        .draw(|frame| render_gem_picker_screen(frame, &c, 11, ""))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Flawed Bloodstone"));
    assert!(!rendered.contains("Flawed Ruby"));
    assert!(rendered.contains("Gems: W/S or arrows=select  Enter=choose  Esc=back"));
}

#[test]
fn ground_loot_picker_scroll_keeps_high_selected_item_visible() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    let mut dungeon = open_test_dungeon(3, 3, Vec::new());
    dungeon.ground_items = (0..12)
        .map(|index| {
            let mut item = health_potion();
            item.name = format!("Ground {index}");
            GroundItem { x: 3, y: 3, item }
        })
        .collect();
    c.active_dungeon = Some(dungeon);
    let mut terminal = Terminal::new(TestBackend::new(80, 16)).unwrap();

    terminal
        .draw(|frame| render_ground_loot_picker(frame, &c, 11, ""))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Ground 11"));
    assert!(!rendered.contains("Ground 0"));
    assert!(rendered.contains("W/S=move  Enter=pick up  d=discard  Esc=back"));
}

#[test]
fn narrow_inventory_and_skill_screens_keep_details_visible() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(54, 24)).unwrap();

    terminal
        .draw(|frame| render_inventory_screen(frame, &c, 0, ""))
        .unwrap();
    let inventory = backend_text(&terminal);
    assert!(inventory.contains("Bag"));
    assert!(inventory.contains("Details"));
    assert!(inventory.contains("Lesser Health Potion"));

    terminal
        .draw(|frame| render_skill_tree_screen(frame, &c, 0, ""))
        .unwrap();
    let skills = backend_text(&terminal);
    assert!(skills.contains("Skills"));
    assert!(skills.contains("Details"));
    assert!(skills.contains("Current Skill"));
}

#[test]
fn narrow_stash_screen_keeps_both_grids_and_details_visible() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(70, 24)).unwrap();

    terminal
        .draw(|frame| render_stash_screen(frame, &c, StashSide::Inventory, 0, 0, ""))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Inventory ✦"));
    assert!(rendered.contains("Stash"));
    assert!(rendered.contains("Details"));
    assert!(rendered.contains("Lesser Health Potion"));
}

#[test]
fn full_sell_screen_keeps_selected_item_details_visible() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.inventory = ItemGrid::new(
        4,
        4,
        (0..16)
            .map(|index| {
                let mut item = rusted_sword();
                item.name = format!("Sell Sword {index}");
                item
            })
            .collect(),
    );
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| render_sell_items_screen(frame, &c, 15, ""))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Sell Sword 15"));
    assert!(rendered.contains("Sell value:"));
}

#[test]
fn full_salvage_screen_keeps_selected_item_details_visible() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.inventory = ItemGrid::new(
        4,
        4,
        (0..16)
            .map(|index| {
                let mut item = rusted_sword();
                item.name = format!("Salvage Sword {index}");
                item
            })
            .collect(),
    );
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| render_salvage_screen(frame, &c, 15, ""))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Salvage Sword 15"));
    assert!(rendered.contains("Salvage yield:"));
}

#[test]
fn full_socket_screen_keeps_selected_socket_details_visible() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.equipped_weapon.sockets = vec![None];
    c.equipped_armor.sockets = vec![None];
    c.equipped_shield.sockets = vec![None];
    c.inventory = ItemGrid::new(
        4,
        4,
        (0..16)
            .map(|index| {
                let mut item = rusted_sword();
                item.name = format!("Socket Sword {index}");
                item.sockets = vec![None];
                item
            })
            .collect(),
    );
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| render_socket_bench_screen(frame, &c, 18, 0, ""))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Socket Sword 15"));
    assert!(rendered.contains("Sockets: Socket Sword 15"));
    assert!(rendered.contains("› 1. Empty"));
}

#[test]
fn socket_bench_only_shows_socket_cursor() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.equipped_weapon.sockets = vec![None];
    c.equipped_armor.sockets = vec![None];
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

    terminal
        .draw(|frame| render_socket_bench_screen(frame, &c, 0, 0, ""))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert_eq!(rendered.matches(SELECTION_CURSOR).count(), 1);
    assert!(rendered.contains("› 1. Empty"));
    assert!(!rendered.contains("› Weapon:"));
}

#[test]
fn skill_screens_render_with_ratatui() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.warrior.cleave_rank = 4;
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();

    terminal
        .draw(|frame| render_skill_tree_screen(frame, &c, 0, ""))
        .unwrap();
    let skill_tree = backend_text(&terminal);
    assert!(skill_tree.contains("Warrior Skill Tree"));
    assert!(!skill_tree.contains("Ironbound Skill Tree"));
    assert!(skill_tree.contains("Cleave"));
    assert!(skill_tree.contains("› Cleave"));
    assert!(skill_tree.contains("Current Skill"));
    assert!(skill_tree.contains("Improved Skill"));
    assert!(skill_tree.contains("Next rank 5"));
    assert!(skill_tree.contains("Details"));
    assert!(skill_tree.contains("W/S or arrows=select"));
    assert!(skill_tree.contains("Enter=upgrade/mastery"));
    let skill_lines = backend_lines(&terminal);
    let skill_header = skill_lines
        .iter()
        .find(|line| line.contains("Skills") && line.contains("Details"))
        .unwrap();
    assert_eq!(char_index(skill_header, "Details"), 51);

    c.warrior.cleave_rank = 1;
    terminal
        .draw(|frame| render_skill_tree_screen(frame, &c, 1, ""))
        .unwrap();
    let locked_passive_tree = backend_text(&terminal);
    let locked_passive_lines = backend_lines(&terminal)
        .into_iter()
        .filter(|line| line.contains("Deep Cut") || line.contains("Unlock"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        locked_passive_tree.contains("└─⊘ Deep Cut unlocks at Cleave rank 2 (1/2)"),
        "{locked_passive_lines}"
    );
    assert!(!locked_passive_tree.contains("Deep Cut rank 0/5"));
    assert!(locked_passive_tree.contains("Shield Bash rank 1/5"));
    assert!(!locked_passive_tree.contains("branch starter"));

    c.warrior.cleave_rank = 5;
    terminal
        .draw(|frame| render_mastery_screen(frame, &c, "Cleave", ""))
        .unwrap();
    let mastery = backend_text(&terminal);
    assert!(mastery.contains("Cleave Mastery"));
    assert!(mastery.contains("Choose one free path"));
}

#[test]
fn locked_skills_show_only_locked_rows_until_unlocked() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    let warrior_lines = skill_tree_lines(&c, 1, "")
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(warrior_lines.contains("   └─⊘ Deep Cut unlocks at Cleave rank 2 (1/2)"));
    assert!(!warrior_lines.contains("Deep Cut rank 0/5"));
    assert!(warrior_lines.contains("› Shield Bash rank 1/5"));

    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal
        .draw(|frame| render_skill_tree_screen(frame, &c, 1, ""))
        .unwrap();
    let warrior_screen = backend_text(&terminal);
    assert!(!warrior_screen.contains("Deep Cut rank 0/5"));
    assert!(warrior_screen.contains("Shield Bash rank 1/5"));
    assert!(warrior_screen.contains("Next rank 2/5"));

    c.warrior.cleave_rank = 2;
    let unlocked_lines = skill_tree_lines(&c, 1, "")
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(unlocked_lines.contains("› Deep Cut rank 0/5"));
    assert!(!unlocked_lines.contains("Deep Cut unlocks at Cleave rank 2"));

    let rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    let rogue_lines = skill_tree_lines(&rogue, 1, "")
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rogue_lines.contains("   └─⊘ Eviscerate unlocks at Backstab rank 2 (1/2)"));
    assert!(!rogue_lines.contains("Eviscerate rank 0/5"));
    assert!(rogue_lines.contains("› Venom Edge rank 1/5"));
}

#[test]
fn sorceress_scaling_helpers_match_mvp_numbers() {
    assert_eq!(
        (1..=5).map(firebolt_percent_for_rank).collect::<Vec<_>>(),
        vec![100, 110, 120, 130, 140]
    );
    assert_eq!(
        (1..=5)
            .map(firebolt_burn_chance_for_rank)
            .collect::<Vec<_>>(),
        vec![25, 30, 35, 40, 45]
    );
    assert_eq!(
        (1..=5).map(frost_ring_percent_for_rank).collect::<Vec<_>>(),
        vec![70, 80, 90, 100, 110]
    );
    assert_eq!(
        (1..=5)
            .map(frost_ring_freeze_chance_for_rank)
            .collect::<Vec<_>>(),
        vec![20, 25, 30, 35, 40]
    );
    assert_eq!(
        (1..=5)
            .map(chain_spark_percent_for_rank)
            .collect::<Vec<_>>(),
        vec![80, 90, 95, 105, 110]
    );
    assert_eq!(
        (1..=5)
            .map(chain_spark_hit_count_for_rank)
            .collect::<Vec<_>>(),
        vec![2, 2, 3, 3, 4]
    );
    assert_eq!(mana_shield_absorb_percent_for_rank(0), 0);
    assert_eq!(
        (1..=5)
            .map(mana_shield_absorb_percent_for_rank)
            .collect::<Vec<_>>(),
        vec![50, 55, 60, 65, 70]
    );
    assert_eq!(
        (1..=5)
            .map(kindle_fire_bonus_percent_for_rank)
            .collect::<Vec<_>>(),
        vec![10, 15, 20, 25, 30]
    );
    assert_eq!(
        (1..=5)
            .map(static_charge_chance_for_rank)
            .collect::<Vec<_>>(),
        vec![15, 20, 25, 30, 35]
    );
    assert_eq!(
        (1..=5)
            .map(static_charge_damage_bonus_for_rank)
            .collect::<Vec<_>>(),
        vec![15, 20, 25, 30, 35]
    );
}

#[test]
fn sorceress_skill_tree_shows_branches_and_starting_mana_shield() {
    let c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );

    let text = skill_tree_lines(&c, 0, "")
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Sorceress Skill Tree"));
    assert!(text.contains("Flame Branch"));
    assert!(text.contains("› Firebolt rank 1/5"));
    assert!(text.contains("└─⊘ Kindle unlocks at Firebolt rank 2 (1/2)"));
    assert!(text.contains("Frost Branch"));
    assert!(text.contains("Frost Ring rank 1/5"));
    assert!(text.contains("Mana Shield rank 1/5"));
    assert!(!text.contains("Mana Shield unlocks at Frost Ring rank 2"));
    assert!(text.contains("Storm Branch"));
    assert!(text.contains("Chain Spark rank 1/5"));
    assert!(text.contains("└─⊘ Static Charge unlocks at Chain Spark rank 2 (1/2)"));
    assert!(!text.contains("Kindle rank 0/5"));
    assert!(!text.contains("Mana Shield rank 0/5"));
    assert!(!text.contains("Static Charge rank 0/5"));
}

#[test]
fn sorceress_mana_shield_details_do_not_show_frost_ring_requirement() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();

    terminal
        .draw(|frame| render_skill_tree_screen(frame, &c, 2, ""))
        .unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Mana Shield rank 1/5"));
    assert!(rendered.contains("Free toggle; 1 mana prevents 1 damage."));
    assert!(!rendered.contains("Requires Frost Ring rank 2."));
}

#[test]
fn sorceress_skill_tree_upgrades_unlockable_skills_with_prerequisites() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.unspent_skills = 4;

    assert_eq!(
        choose_skill_or_mastery(&mut c, "Mana Shield"),
        "Upgraded Mana Shield to rank 2."
    );
    assert_eq!(
        choose_skill_or_mastery(&mut c, "Frost Ring"),
        "Upgraded Frost Ring to rank 2."
    );
    assert_eq!(
        choose_skill_or_mastery(&mut c, "Firebolt"),
        "Upgraded Firebolt to rank 2."
    );
    assert_eq!(
        choose_skill_or_mastery(&mut c, "Kindle"),
        "Upgraded Kindle to rank 1."
    );

    assert_eq!(c.sorceress.frost_ring_rank, 2);
    assert_eq!(c.sorceress.mana_shield_rank, 2);
    assert_eq!(c.sorceress.firebolt_rank, 2);
    assert_eq!(c.sorceress.kindle_rank, 1);
}

#[test]
fn rogue_skill_screen_renders_with_ratatui() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    let lines = skill_tree_lines(&c, 0, "");
    let text = lines
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Rogue Skill Tree"));
    assert!(text.contains("Backstab"));
    assert!(text.contains("Venom Edge"));
    assert!(text.contains("Eviscerate"));
    assert!(text.contains("Smoke Step"));
    assert!(text.contains("Rupture"));
    assert!(text.contains("Slip Away"));

    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal
        .draw(|frame| render_skill_tree_screen(frame, &c, 1, ""))
        .unwrap();
    let rendered = backend_text(&terminal);
    assert!(rendered.contains("Poison deals 2/turn for 3 turns."));
}

#[test]
fn rogue_passives_start_locked_and_have_no_effect_until_unlocked() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );

    c.unspent_skills = 1;
    assert_eq!(c.rogue.rupture_rank, 0);
    assert_eq!(c.rogue.slip_away_rank, 0);
    assert_eq!(
        choose_skill_or_mastery(&mut c, "Rupture"),
        "Rupture upgrades require Venom Edge rank 2."
    );
    assert_eq!(
        choose_skill_or_mastery(&mut c, "Slip Away"),
        "Slip Away upgrades require Smoke Step rank 2."
    );

    c.rogue.smoke_protection_turns = 1;
    assert_eq!(
        defensive_dodge_rating(&c),
        c.dodge_rating() as i32 + smoke_step_dodge_bonus_for_rank(c.rogue.smoke_step_rank)
    );
}

#[test]
fn new_character_uses_starting_bag_and_stash_grids() {
    let c = test_character();

    assert_eq!((c.inventory.columns, c.inventory.rows), (4, 4));
    assert_eq!(c.inventory.capacity(), 16);
    assert_eq!(c.inventory.len(), 3);
    assert_eq!((c.stash.columns, c.stash.rows), (8, 8));
    assert_eq!(c.stash.capacity(), 64);
    assert_eq!(c.stash.len(), 0);
}

#[test]
fn dungeon_starts_without_ground_items() {
    let d = generate_dungeon(1);

    assert!(d.ground_items.is_empty());
}

#[test]
#[should_panic(expected = "ItemGrid cannot hold 2 items in 1 slots")]
fn item_grid_new_panics_when_initial_items_exceed_capacity() {
    let _ = ItemGrid::new(1, 1, vec![health_potion(), mana_potion()]);
}

#[test]
fn new_characters_start_with_empty_accessory_slots() {
    let warrior = test_character();
    assert_eq!(warrior.equipped_helm.name, "Empty Helm");
    assert_eq!(warrior.equipped_gloves.name, "Empty Gloves");
    assert_eq!(warrior.equipped_boots.name, "Empty Boots");
    assert_eq!(warrior.equipped_belt.name, "Empty Belt");
    assert_eq!(warrior.equipped_amulet.name, "Empty Amulet");
    assert_eq!(warrior.equipped_ring1.name, "Empty Ring");
    assert_eq!(warrior.equipped_ring2.name, "Empty Ring");

    let rogue = Character::new(
        "Shade".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    assert_eq!(rogue.equipped_helm.kind, ItemKind::Helm);
    assert_eq!(rogue.equipped_gloves.kind, ItemKind::Gloves);
    assert_eq!(rogue.equipped_boots.kind, ItemKind::Boots);
    assert_eq!(rogue.equipped_belt.kind, ItemKind::Belt);
    assert_eq!(rogue.equipped_amulet.kind, ItemKind::Amulet);
    assert_eq!(rogue.equipped_ring1.kind, ItemKind::Ring);
    assert_eq!(rogue.equipped_ring2.kind, ItemKind::Ring);
}

#[test]
fn new_warrior_matches_mvp_starting_state() {
    let c = test_character();

    assert_eq!(c.class_name(), "Warrior");
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
    assert_eq!(c.armor(), 2); // cloth 1 + shield 1; locked Iron Guard has no bonus
    assert_eq!(
        (
            c.warrior.deep_cut_rank,
            c.warrior.iron_guard_rank,
            c.warrior.second_wind_rank
        ),
        (0, 0, 0)
    );
    assert!(!c.bellkeeper_defeated);
    assert!(!c.act1_completed);
}

#[test]
fn new_rogue_matches_starting_state() {
    let c = Character::new(
        "Shade".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );

    assert_eq!(c.class, CharacterClass::Rogue);
    assert_eq!(c.class_name(), "Rogue");
    assert_eq!(c.strength, 2);
    assert_eq!(c.dexterity, 7);
    assert_eq!(c.intelligence, 1);
    assert_eq!(c.rogue.energy, ROGUE_MAX_ENERGY);
    assert_eq!(c.rogue.combo_points, 0);
    assert_eq!(c.rogue.rupture_rank, 0);
    assert_eq!(c.rogue.slip_away_rank, 0);
    assert!(c.equipped_weapon.name.contains("Dagger"));
    assert_eq!(c.equipped_weapon.kind, ItemKind::Weapon);
    assert_eq!(c.equipped_weapon.required_strength, 0);
    assert_eq!(c.equipped_weapon.required_dexterity, 2);
    assert_eq!(c.equipped_weapon.required_intelligence, 0);
    assert!(can_equip_item(&c, &c.equipped_weapon));
    assert!(c.equipped_armor.name.contains("Leathers"));
    assert_eq!(c.equipped_armor.kind, ItemKind::Armor);
    assert_eq!(c.equipped_armor.required_strength, 0);
    assert_eq!(c.equipped_armor.required_dexterity, 0);
    assert_eq!(c.equipped_armor.required_intelligence, 0);
    assert!(can_equip_item(&c, &c.equipped_armor));
    assert_eq!(c.equipped_shield.kind, ItemKind::Shield);
    assert_eq!(c.equipped_shield.required_strength, 0);
    assert_eq!(c.equipped_shield.required_dexterity, 0);
    assert_eq!(c.equipped_shield.required_intelligence, 0);
    assert!(can_equip_item(&c, &c.equipped_shield));
}

#[test]
fn sorceress_state_defaults_match_mvp_skill_baseline() {
    let state = SorceressState::default();

    assert_eq!(state.firebolt_rank, 1);
    assert_eq!(state.frost_ring_rank, 1);
    assert_eq!(state.chain_spark_rank, 1);
    assert_eq!(state.kindle_rank, 0);
    assert_eq!(state.mana_shield_rank, 1);
    assert_eq!(state.static_charge_rank, 0);
    assert_eq!(state.frost_ring_cooldown, 0);
    assert_eq!(state.chain_spark_cooldown, 0);
    assert!(!state.mana_shield_active);
}

#[test]
fn new_sorceress_matches_starting_state() {
    let c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );

    assert_eq!(c.class, CharacterClass::Sorceress);
    assert_eq!(c.class_name(), "Sorceress");
    assert_eq!((c.strength, c.dexterity, c.intelligence), (1, 3, 6));
    assert_eq!(c.max_hp(), 15);
    assert_eq!(c.max_mana(), 40);
    assert_eq!(c.hp, c.max_hp());
    assert_eq!(c.mana, c.max_mana());
    assert_eq!(c.sorceress.mana_shield_rank, 1);
    assert!(!c.sorceress.mana_shield_active);
    assert_eq!(c.armor(), 1);
    assert_eq!(c.dodge_rating(), 13);
    assert_eq!(c.inventory.len(), 4);
    assert_eq!(
        c.inventory
            .iter()
            .filter(|item| item.kind == ItemKind::HealthPotion)
            .count(),
        2
    );
    assert_eq!(
        c.inventory
            .iter()
            .filter(|item| item.kind == ItemKind::ManaPotion)
            .count(),
        2
    );
    assert!(c.equipped_weapon.name.contains("Wand"));
    assert_eq!(c.equipped_weapon.kind, ItemKind::Weapon);
    assert_eq!(c.equipped_weapon.required_strength, 0);
    assert_eq!(c.equipped_weapon.required_dexterity, 0);
    assert_eq!(c.equipped_weapon.required_intelligence, 2);
    assert!(c.equipped_shield.name.contains("Focus"));
    assert_eq!(c.equipped_shield.kind, ItemKind::Shield);
    assert_eq!(c.equipped_shield.dodge, 2);
    assert_eq!(c.equipped_shield.required_strength, 0);
    assert_eq!(c.equipped_shield.required_dexterity, 0);
    assert_eq!(c.equipped_shield.required_intelligence, 2);
    assert!(c.equipped_armor.name.contains("Robe"));
    assert_eq!(c.equipped_armor.kind, ItemKind::Armor);
    assert_eq!(c.equipped_armor.armor, 1);
    assert_eq!(c.equipped_armor.dodge, 1);
    assert!(can_equip_item(&c, &c.equipped_weapon));
    assert!(can_equip_item(&c, &c.equipped_shield));
    assert!(can_equip_item(&c, &c.equipped_armor));
}

#[test]
fn rogue_cannot_equip_warrior_shields() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.strength = 99;
    c.inventory.push(worn_shield());
    let shield_index = c
        .inventory
        .iter()
        .position(|item| item.name.contains("Worn Shield"))
        .unwrap();

    assert!(!can_equip_item(&c, c.inventory.get(shield_index).unwrap()));
    let result = equip_or_use_inventory_item(&mut c, shield_index);

    assert_eq!(result.message, "Rogue cannot equip non-buckler shields.");
    assert!(!result.spent_turn);
    assert_eq!(c.equipped_shield.name, "Empty Offhand");
    assert!(
        c.inventory
            .iter()
            .any(|item| item.name.contains("Worn Shield"))
    );
}

#[test]
fn character_creation_renders_class_choices() {
    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                CharacterCreationStep::Class,
                "Mara",
                CharacterClass::Rogue,
                DeathMode::Hardcore,
                "",
            )
        })
        .unwrap();

    let text = backend_text(&terminal);
    assert!(text.contains("Warrior"));
    assert!(text.contains("Rogue"));
    assert!(text.contains("› Rogue"));
    assert!(text.contains("Hardcore"));
    assert!(!text.contains("› Hardcore"));
}

#[test]
fn character_creation_can_select_sorceress() {
    let mut state = CharacterCreationState::new("");

    assert_eq!(state.selected_class, CharacterClass::Warrior);
    assert!(state.handle_key(KEY_ARROW_DOWN).is_none());
    assert_eq!(state.selected_class, CharacterClass::Rogue);
    assert!(state.handle_key(KEY_ARROW_DOWN).is_none());
    assert_eq!(state.selected_class, CharacterClass::Sorceress);
    assert!(state.handle_key(KEY_ARROW_DOWN).is_none());
    assert_eq!(state.selected_class, CharacterClass::Warrior);
    assert!(state.handle_key(KEY_ARROW_UP).is_none());
    assert_eq!(state.selected_class, CharacterClass::Sorceress);
    assert!(state.handle_key('1').is_none());
    assert_eq!(state.selected_class, CharacterClass::Warrior);
    assert!(state.handle_key('2').is_none());
    assert_eq!(state.selected_class, CharacterClass::Rogue);
    assert!(state.handle_key('3').is_none());
    assert_eq!(state.selected_class, CharacterClass::Sorceress);

    assert!(state.handle_key('\n').is_none());
    for key in "Lyra".chars() {
        assert!(state.handle_key(key).is_none());
    }
    assert!(state.handle_key('\n').is_none());
    let character = state.handle_key('\n').unwrap();

    assert_eq!(character.name, "Lyra");
    assert_eq!(character.class, CharacterClass::Sorceress);
    assert_eq!(character.death_mode, DeathMode::Softcore);
}

#[test]
fn character_creation_renders_sorceress_choice() {
    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                CharacterCreationStep::Class,
                "Lyra",
                CharacterClass::Sorceress,
                DeathMode::Softcore,
                "",
            )
        })
        .unwrap();

    let text = backend_text(&terminal);
    assert!(text.contains("Warrior"));
    assert!(text.contains("Rogue"));
    assert!(text.contains("Sorceress"));
    assert!(text.contains("› Sorceress"));
    assert!(!text.contains("› Warrior"));
    assert!(!text.contains("› Rogue"));
}

#[test]
fn character_creation_arrow_keys_move_current_selection() {
    let mut state = CharacterCreationState::new("");

    assert_eq!(state.selected_class, CharacterClass::Warrior);
    assert!(state.handle_key('\u{0e}').is_none());
    assert_eq!(state.selected_class, CharacterClass::Rogue);
    assert!(state.handle_key('\u{10}').is_none());
    assert_eq!(state.selected_class, CharacterClass::Warrior);

    assert!(state.handle_key('\n').is_none());
    for key in "Mara".chars() {
        assert!(state.handle_key(key).is_none());
    }
    assert!(state.handle_key('\n').is_none());

    assert_eq!(state.death_mode, DeathMode::Softcore);
    assert!(state.handle_key('\u{0e}').is_none());
    assert_eq!(state.death_mode, DeathMode::Hardcore);
    assert!(state.handle_key('\u{10}').is_none());
    assert_eq!(state.death_mode, DeathMode::Softcore);
}

#[test]
fn character_creation_key_flow_uses_steps_and_tab_only_death_toggle() {
    let mut state = CharacterCreationState::new("");

    assert_eq!(state.step, CharacterCreationStep::Class);
    assert!(state.handle_key('\n').is_none());
    assert_eq!(state.step, CharacterCreationStep::Name);
    assert!(state.handle_key('\u{1b}').is_none());
    assert_eq!(state.step, CharacterCreationStep::Class);

    assert!(state.handle_key('2').is_none());
    assert_eq!(state.selected_class, CharacterClass::Rogue);
    assert!(state.handle_key('\n').is_none());
    for key in "Shade".chars() {
        assert!(state.handle_key(key).is_none());
    }
    assert!(state.handle_key('\n').is_none());
    assert_eq!(state.step, CharacterCreationStep::DeathMode);
    assert_eq!(state.death_mode, DeathMode::Softcore);

    assert!(state.handle_key('h').is_none());
    assert!(state.handle_key('H').is_none());
    assert!(state.handle_key('s').is_none());
    assert!(state.handle_key('S').is_none());
    assert!(state.handle_key('1').is_none());
    assert!(state.handle_key('2').is_none());
    assert_eq!(state.death_mode, DeathMode::Softcore);

    assert!(state.handle_key('\t').is_none());
    assert_eq!(state.death_mode, DeathMode::Hardcore);
    assert!(state.handle_key('\u{1b}').is_none());
    assert_eq!(state.step, CharacterCreationStep::Name);
    assert!(state.handle_key('\n').is_none());

    let character = state.handle_key('\n').unwrap();
    assert_eq!(character.name, "Shade");
    assert_eq!(character.class, CharacterClass::Rogue);
    assert_eq!(character.death_mode, DeathMode::Hardcore);
}

#[test]
fn class_names_parse_current_and_legacy_values() {
    assert_eq!(
        CharacterClass::from_save_name("Warrior"),
        CharacterClass::Warrior
    );
    assert_eq!(
        CharacterClass::from_save_name("Ironbound"),
        CharacterClass::Warrior
    );
    assert_eq!(
        CharacterClass::from_save_name("Rogue"),
        CharacterClass::Rogue
    );
    assert_eq!(
        CharacterClass::from_save_name("Sorceress"),
        CharacterClass::Sorceress
    );
    assert_eq!(CharacterClass::Warrior.name(), "Warrior");
    assert_eq!(CharacterClass::Rogue.name(), "Rogue");
    assert_eq!(CharacterClass::Sorceress.name(), "Sorceress");
}

#[test]
fn package_version_is_major_one_for_save_breaking_rogue_release() {
    assert!(SAVE_VERSION.starts_with("1."));
}

#[test]
fn warrior_state_defaults_keep_starters_at_one_and_locked_skills_at_zero() {
    let state = WarriorState::default();

    assert_eq!(state.cleave_rank, 1);
    assert_eq!(state.shield_bash_rank, 1);
    assert_eq!(state.battle_cry_rank, 1);
    assert_eq!(state.deep_cut_rank, 0);
    assert_eq!(state.iron_guard_rank, 0);
    assert_eq!(state.second_wind_rank, 0);
    assert_eq!(state.cleave_cooldown, 0);
    assert_eq!(state.shield_bash_cooldown, 0);
    assert_eq!(state.battle_cry_cooldown, 0);
    assert_eq!(state.battle_cry_charges, 0);
    assert_eq!(state.second_wind_shield, 0);
}

#[test]
fn rogue_ignores_default_warrior_passive_armor() {
    let mut c = test_character();
    c.class = CharacterClass::Rogue;
    c.warrior = WarriorState::default();

    assert_eq!(c.warrior.iron_guard_rank, 0);
    assert_eq!(iron_guard_armor_bonus(&c), 0);
    assert_eq!(c.armor(), c.equipped_armor.armor + c.equipped_shield.armor);
}

#[test]
fn rogue_ignores_warrior_shared_stat_and_shield_effects() {
    let mut c = test_character();
    c.class = CharacterClass::Rogue;
    c.hp = 1;
    c.equipped_weapon.crit_chance = 8;
    c.warrior.iron_guard_mastery = Some(SkillMastery::ShieldDiscipline);
    c.warrior.battle_cry_charges = 1;
    c.warrior.second_wind_shield = 5;

    assert_eq!(c.dodge_rating(), 12);
    c.warrior.iron_guard_mastery = Some(SkillMastery::Bulwark);
    assert_eq!(c.armor(), c.equipped_armor.armor + c.equipped_shield.armor);
    assert_eq!(enemy_damage_after_mitigation(10, &c), 8);
    assert_eq!(player_crit_chance(&c), 8);

    apply_player_damage(&mut c, 3);

    assert_eq!(c.hp, 0);
    assert_eq!(c.warrior.second_wind_shield, 5);
}

#[test]
fn rogue_ignores_warrior_hemorrhage_bleed_bonus() {
    let mut c = test_character();
    c.class = CharacterClass::Rogue;
    c.warrior.deep_cut_mastery = Some(SkillMastery::Hemorrhage);
    let mut enemy = skeleton(10, 10);
    enemy.hp = 4;
    enemy.max_hp = 10;
    enemy.bleed_turns = 1;
    enemy.bleed_damage = 1;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    enemy_turns(&mut c);

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.enemies[0].hp, 3);
}

#[test]
fn saved_items_without_socket_fields_default_to_no_sockets_or_gem_metadata() {
    let json = r#"{
        "name": "Old Sword",
        "kind": "Weapon",
        "value": 10,
        "damage_min": 1,
        "damage_max": 2
    }"#;

    let item: Item = serde_json::from_str(json).unwrap();

    assert!(item.sockets.is_empty());
    assert!(item.gem_kind.is_none());
    assert!(item.gem_tier.is_none());
}

#[test]
fn gems_are_normal_items_with_kind_tier_and_value() {
    let gem = gem_item(GemKind::Topaz, GemTier::Flawed);

    assert!(matches!(gem.kind, ItemKind::Gem));
    assert_eq!(gem.gem_kind, Some(GemKind::Topaz));
    assert_eq!(gem.gem_tier, Some(GemTier::Flawed));
    assert!(gem.name.contains("Flawed Topaz"));
    assert!(gem.value > 0);
}

#[test]
fn item_summary_shows_gems_and_socket_contents() {
    let mut gem = gem_item(GemKind::Topaz, GemTier::Pristine);
    gem.name = "Wrong Name".to_string();
    assert!(strip_ansi_codes(&item_summary(&gem)).contains("Pristine Topaz"));
    assert!(strip_ansi_codes(&item_summary(&gem)).contains("+4% crit chance"));

    let mut sword = rusted_sword();
    sword.sockets = vec![
        Some(GemSocket::filled(GemKind::Ruby, GemTier::Chipped)),
        None,
    ];
    let summary = strip_ansi_codes(&item_summary(&sword));

    assert!(summary.contains("Sockets [Chipped Ruby, empty]"));
}

#[test]
fn item_summary_marks_gems_with_incomplete_metadata_invalid() {
    let mut missing_tier = gem_item(GemKind::Topaz, GemTier::Pristine);
    missing_tier.gem_tier = None;
    assert!(
        strip_ansi_codes(&item_summary(&missing_tier)).contains("Invalid gem metadata"),
        "missing tier summary should mark invalid metadata"
    );

    let mut missing_kind = gem_item(GemKind::Topaz, GemTier::Pristine);
    missing_kind.gem_kind = None;
    assert!(
        strip_ansi_codes(&item_summary(&missing_kind)).contains("Invalid gem metadata"),
        "missing kind summary should mark invalid metadata"
    );
}

#[test]
fn starter_item_names_do_not_show_attribute_scaling_grades() {
    for item in [
        rusted_sword(),
        training_dagger(),
        cracked_wand(),
        cracked_focus(),
        crude_axe(),
    ] {
        let name = item.name;
        assert!(!name.contains("STR "), "{name} should not show STR scaling");
        assert!(!name.contains("DEX "), "{name} should not show DEX scaling");
        assert!(!name.contains("INT "), "{name} should not show INT scaling");
    }
}

#[test]
fn dexterity_only_increases_hit_rating() {
    let mut c = test_character();
    c.dexterity = 0;
    let base_hit = c.hit_rating();
    let base_dodge = c.dodge_rating();
    let base_speed = c.speed();

    c.dexterity = 4;

    assert_eq!(c.hit_rating(), base_hit + 40);
    assert_eq!(c.dodge_rating(), base_dodge);
    assert_eq!(c.speed(), base_speed);
}

#[test]
fn primary_attributes_do_not_scale_weapon_damage() {
    let mut c = test_character();
    let base_damage = c.weapon_damage();

    c.strength += 20;
    c.dexterity += 20;
    c.intelligence += 20;

    assert_eq!(c.weapon_damage(), base_damage);
}

#[test]
fn accessory_slots_contribute_stats_and_socket_bonuses() {
    let mut c = test_character();
    c.equipped_helm = item_with_rarity(
        "Test Helm",
        ItemKind::Helm,
        10,
        item_stats(0, 0, 2, 1, 0),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    );
    c.equipped_boots = item_with_rarity(
        "Test Boots",
        ItemKind::Boots,
        10,
        item_stats(0, 0, 0, 1, 2),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    );
    c.equipped_amulet = item_with_rarity(
        "Socketed Amulet",
        ItemKind::Amulet,
        10,
        item_stats(0, 0, 0, 0, 0),
        Rarity::Magic,
        1,
        requirements(0, 0, 0),
    );
    c.equipped_amulet.sockets = vec![Some(GemSocket::filled(GemKind::Emerald, GemTier::Pristine))];

    assert_eq!(c.effective_dexterity(), c.dexterity + 3);
    assert_eq!(c.armor(), 1 + 1 + 2 + iron_guard_armor_bonus(&c));
    assert_eq!(c.dodge_rating(), 10 + 2 + 1 + 1);
    assert_eq!(c.speed(), 10 + 2);
}

#[test]
fn equipped_socketed_gems_add_effective_stats() {
    let mut c = test_character();
    c.equipped_weapon.sockets = vec![Some(GemSocket::filled(
        GemKind::Bloodstone,
        GemTier::Pristine,
    ))];
    c.equipped_armor.sockets = vec![
        Some(GemSocket::filled(GemKind::Ruby, GemTier::Flawed)),
        Some(GemSocket::filled(GemKind::Garnet, GemTier::Chipped)),
    ];
    c.equipped_shield.sockets = vec![Some(GemSocket::filled(GemKind::Topaz, GemTier::Pristine))];

    assert_eq!(c.effective_strength(), c.strength + 1);
    assert_eq!(c.max_hp(), 10 + c.effective_strength() * 5 + 10);
    assert_eq!(c.weapon_damage(), (6, 8));
    assert_eq!(c.weapon_crit_chance(), c.equipped_weapon.crit_chance + 4);
}

#[test]
fn equipped_socketed_gems_cover_all_remaining_stat_paths() {
    let mut c = test_character();
    c.equipped_weapon.sockets = vec![
        Some(GemSocket::filled(GemKind::Sapphire, GemTier::Pristine)),
        Some(GemSocket::filled(GemKind::Quartz, GemTier::Pristine)),
    ];
    c.equipped_armor.sockets = vec![
        Some(GemSocket::filled(GemKind::Emerald, GemTier::Pristine)),
        Some(GemSocket::filled(GemKind::Amethyst, GemTier::Pristine)),
        Some(GemSocket::filled(GemKind::Citrine, GemTier::Pristine)),
    ];
    c.equipped_shield.sockets = vec![
        Some(GemSocket::filled(GemKind::Jade, GemTier::Pristine)),
        Some(GemSocket::filled(GemKind::Onyx, GemTier::Pristine)),
    ];

    assert_eq!(c.effective_dexterity(), c.dexterity + 3);
    assert_eq!(c.effective_intelligence(), c.intelligence + 3);
    assert_eq!(c.max_mana(), 10 + c.effective_intelligence() * 5 + 12);
    assert_eq!(c.hit_rating(), 10 + c.effective_dexterity() * 10 + 10);
    assert_eq!(c.dodge_rating(), 10 + 2 + 8);
    assert_eq!(c.armor(), 1 + 1 + iron_guard_armor_bonus(&c) + 3);
    assert_eq!(c.speed(), 10 + 7);
}

#[test]
fn socket_count_rolls_follow_rarity_thresholds() {
    assert_eq!(socket_count_for_roll(&Rarity::Common, 0.099), 1);
    assert_eq!(socket_count_for_roll(&Rarity::Common, 0.100), 0);
    assert_eq!(socket_count_for_roll(&Rarity::Magic, 0.049), 2);
    assert_eq!(socket_count_for_roll(&Rarity::Magic, 0.050), 1);
    assert_eq!(socket_count_for_roll(&Rarity::Magic, 0.249), 1);
    assert_eq!(socket_count_for_roll(&Rarity::Magic, 0.250), 0);
    assert_eq!(socket_count_for_roll(&Rarity::Rare, 0.099), 2);
    assert_eq!(socket_count_for_roll(&Rarity::Rare, 0.100), 1);
    assert_eq!(socket_count_for_roll(&Rarity::Rare, 0.349), 1);
    assert_eq!(socket_count_for_roll(&Rarity::Rare, 0.350), 0);
}

#[test]
fn gem_tier_rolls_use_approved_weights() {
    assert_eq!(gem_tier_for_roll(0.799), GemTier::Chipped);
    assert_eq!(gem_tier_for_roll(0.800), GemTier::Flawed);
    assert_eq!(gem_tier_for_roll(0.969), GemTier::Flawed);
    assert_eq!(gem_tier_for_roll(0.970), GemTier::Pristine);
}

#[test]
fn gems_do_not_drop_before_floor_three() {
    assert!(!can_drop_gem_on_floor(2));
    assert!(can_drop_gem_on_floor(3));
}

#[test]
fn opal_socket_bonus_increases_variable_gold_drops() {
    let mut c = test_character();
    c.equipped_armor.sockets = vec![Some(GemSocket::filled(GemKind::Opal, GemTier::Pristine))];

    assert_eq!(apply_gold_find_bonus(&c, 10), 12);
}

#[test]
fn socket_bench_requires_completed_project() {
    let mut c = test_character();
    c.equipped_weapon.sockets = vec![None];
    c.inventory.push(gem_item(GemKind::Ruby, GemTier::Chipped));

    assert_eq!(
        insert_gem_into_equipped(&mut c, UpgradeSlot::Weapon, 0, 0),
        "Complete the Socket Bench project before socketing gems."
    );
}

#[test]
fn socket_bench_inserts_removes_and_replaces_gems_for_free() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::SocketBench);
    c.equipped_weapon.sockets = vec![None];
    c.inventory.clear();
    c.inventory.push(gem_item(GemKind::Ruby, GemTier::Chipped));
    c.inventory.push(gem_item(GemKind::Topaz, GemTier::Flawed));

    assert_eq!(
        insert_gem_into_equipped(&mut c, UpgradeSlot::Weapon, 0, 0),
        "Inserted Chipped Ruby into Rusted Sword (3-5 dmg)."
    );
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(
        c.equipped_weapon.sockets[0],
        Some(GemSocket::filled(GemKind::Ruby, GemTier::Chipped))
    );

    assert_eq!(
        replace_gem_in_equipped(&mut c, UpgradeSlot::Weapon, 0, 0),
        "Replaced Chipped Ruby with Flawed Topaz in Rusted Sword (3-5 dmg)."
    );
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(c.inventory[0].gem_kind, Some(GemKind::Ruby));
    assert_eq!(
        c.equipped_weapon.sockets[0],
        Some(GemSocket::filled(GemKind::Topaz, GemTier::Flawed))
    );

    assert_eq!(
        remove_gem_from_equipped(&mut c, UpgradeSlot::Weapon, 0),
        "Removed Flawed Topaz from Rusted Sword (3-5 dmg)."
    );
    assert_eq!(c.inventory.len(), 2);
    assert!(c.equipped_weapon.sockets[0].is_none());
}

#[test]
fn removing_hp_or_mana_gem_clamps_current_resources() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::SocketBench);
    c.equipped_armor.sockets = vec![Some(GemSocket::filled(GemKind::Ruby, GemTier::Pristine))];
    c.hp = c.max_hp();
    let ruby_max_hp = c.max_hp();

    remove_gem_from_equipped(&mut c, UpgradeSlot::Armor, 0);

    assert_eq!(c.hp, c.max_hp());
    assert!(c.max_hp() < ruby_max_hp);

    c.equipped_shield.sockets = vec![Some(GemSocket::filled(
        GemKind::Sapphire,
        GemTier::Pristine,
    ))];
    c.mana = c.max_mana();
    let sapphire_max_mana = c.max_mana();

    remove_gem_from_equipped(&mut c, UpgradeSlot::Shield, 0);

    assert_eq!(c.mana, c.max_mana());
    assert!(c.max_mana() < sapphire_max_mana);
}

#[test]
fn removing_socketed_gem_requires_bag_capacity() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::SocketBench);
    c.equipped_weapon.sockets = vec![Some(GemSocket::filled(GemKind::Ruby, GemTier::Chipped))];
    c.inventory = ItemGrid::new(1, 1, vec![health_potion()]);

    let message = remove_gem_from_equipped(&mut c, UpgradeSlot::Weapon, 0);

    assert_eq!(message, "Need one free bag cell to remove socketed gem.");
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(
        c.equipped_weapon.sockets[0],
        Some(GemSocket::filled(GemKind::Ruby, GemTier::Chipped))
    );
}

#[test]
fn replacing_socketed_gem_when_bag_is_full_reuses_selected_gem_cell() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::SocketBench);
    c.equipped_weapon.sockets = vec![Some(GemSocket::filled(GemKind::Ruby, GemTier::Chipped))];
    c.inventory = ItemGrid::new(1, 1, vec![gem_item(GemKind::Topaz, GemTier::Flawed)]);

    let message = replace_gem_in_equipped(&mut c, UpgradeSlot::Weapon, 0, 0);

    assert_eq!(
        message,
        "Replaced Chipped Ruby with Flawed Topaz in Rusted Sword (3-5 dmg)."
    );
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(c.inventory[0].gem_kind, Some(GemKind::Ruby));
    assert_eq!(c.inventory[0].gem_tier, Some(GemTier::Chipped));
    assert_eq!(
        c.equipped_weapon.sockets[0],
        Some(GemSocket::filled(GemKind::Topaz, GemTier::Flawed))
    );
}

#[test]
fn socket_bench_rejects_gem_items_with_incomplete_metadata() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::SocketBench);
    c.equipped_weapon.sockets = vec![None];
    c.inventory.clear();
    let mut gem = gem_item(GemKind::Ruby, GemTier::Chipped);
    gem.gem_tier = None;
    c.inventory.push(gem);

    assert_eq!(
        insert_gem_into_equipped(&mut c, UpgradeSlot::Weapon, 0, 0),
        "Select a valid gem from inventory."
    );
    assert!(c.equipped_weapon.sockets[0].is_none());
    assert_eq!(c.inventory.len(), 1);
}

#[test]
fn new_character_has_no_completed_town_projects() {
    let c = test_character();

    assert!(c.completed_town_projects.is_empty());
    assert!(!has_completed_project(&c, TownProject::RebuildForge));
}

#[test]
fn town_project_availability_uses_completion_and_quest_gates() {
    let mut c = test_character();

    assert_eq!(
        town_project_availability(&c, TownProject::RebuildForge),
        ProjectAvailability::Available
    );
    assert_eq!(
        town_project_availability(&c, TownProject::ReinforcedAnvil),
        ProjectAvailability::Locked("Requires Rebuild the Forge.")
    );
    assert_eq!(
        town_project_availability(&c, TownProject::HerbGarden),
        ProjectAvailability::Locked("Requires Act I completed.")
    );

    complete_project_for_test(&mut c, TownProject::RebuildForge);
    assert_eq!(
        town_project_availability(&c, TownProject::ReinforcedAnvil),
        ProjectAvailability::Available
    );
    assert_eq!(
        town_project_availability(&c, TownProject::SocketBench),
        ProjectAvailability::Locked("Requires Reinforced Anvil.")
    );

    c.act1_completed = true;
    assert_eq!(
        town_project_availability(&c, TownProject::HerbGarden),
        ProjectAvailability::Available
    );
    complete_project_for_test(&mut c, TownProject::ReinforcedAnvil);
    assert_eq!(
        town_project_availability(&c, TownProject::SocketBench),
        ProjectAvailability::Available
    );
}

#[test]
fn bag_dimensions_follow_quartermaster_project_chain() {
    let mut c = test_character();

    assert_eq!(bag_dimensions(&c), (4, 4));

    complete_project_for_test(&mut c, TownProject::StorehouseShelves);
    assert_eq!(bag_dimensions(&c), (5, 4));

    complete_project_for_test(&mut c, TownProject::PackHooks);
    assert_eq!(bag_dimensions(&c), (5, 5));

    complete_project_for_test(&mut c, TownProject::OilclothSatchel);
    assert_eq!(bag_dimensions(&c), (6, 5));

    complete_project_for_test(&mut c, TownProject::QuartermasterLedger);
    assert_eq!(bag_dimensions(&c), (6, 6));

    complete_project_for_test(&mut c, TownProject::ReinforcedPack);
    assert_eq!(bag_dimensions(&c), (7, 6));

    complete_project_for_test(&mut c, TownProject::StitchedPockets);
    assert_eq!(bag_dimensions(&c), (7, 7));

    complete_project_for_test(&mut c, TownProject::DeepRucksack);
    assert_eq!(bag_dimensions(&c), (8, 7));

    complete_project_for_test(&mut c, TownProject::ExilesTrunk);
    assert_eq!(bag_dimensions(&c), (8, 8));
}

#[test]
fn completing_bag_project_resizes_inventory_grid() {
    let mut c = test_character();
    c.gold = 200;

    let message = complete_town_project(&mut c, TownProject::StorehouseShelves);

    assert_eq!(message, "Completed project: Storehouse Shelves.");
    assert_eq!((c.inventory.columns, c.inventory.rows), (5, 4));
    assert_eq!(c.inventory.capacity(), 20);
}

#[test]
fn bag_project_chain_locks_until_previous_upgrade_is_complete() {
    let c = test_character();

    assert_eq!(
        town_project_availability(&c, TownProject::PackHooks),
        ProjectAvailability::Locked("Requires Storehouse Shelves.")
    );
}

#[test]
fn town_project_status_text_describes_available_locked_and_completed_projects() {
    let mut c = test_character();

    assert_eq!(
        town_project_status_text(&c, TownProject::RebuildForge),
        "Available"
    );
    assert_eq!(
        town_project_status_text(&c, TownProject::HerbGarden),
        "Locked: Requires Act I completed."
    );

    complete_project_for_test(&mut c, TownProject::RebuildForge);
    assert_eq!(
        town_project_status_text(&c, TownProject::RebuildForge),
        "Complete"
    );
}

#[test]
fn town_project_row_text_includes_group_cost_status_and_benefit() {
    let c = test_character();

    let row = town_project_row_text(&c, TownProject::HireAppraiser);

    assert!(row.contains("[Appraiser]"));
    assert!(row.contains("Hire Appraiser"));
    assert!(row.contains("250 gold"));
    assert!(row.contains("Available"));
    assert!(row.contains("Improve sell prices from 25% to 30%."));
}

#[test]
fn town_project_board_uses_list_and_details_panels() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.gold = 100;
    let selected = TOWN_PROJECTS
        .iter()
        .position(|definition| definition.project == TownProject::HireAppraiser)
        .unwrap();
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();

    terminal
        .draw(|frame| render_town_projects_screen(frame, &c, selected, ""))
        .unwrap();
    let rendered = backend_text(&terminal);
    let lines = backend_lines(&terminal);

    assert!(rendered.contains("Projects"));
    assert!(rendered.contains("Details"));
    assert!(
        lines
            .iter()
            .any(|line| line.contains("› [Appraiser] Hire Appraiser"))
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.contains("Hire Appraiser - Available"))
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("└─ ⊘ [Smith] Reinforced Anvil"))
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("   └─ ⊘ [Smith] Socket Bench"))
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("└─ ⊘ [Quartermaster] Pack Hooks"))
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.contains("Reinforced Anvil - Locked"))
    );
    assert!(rendered.contains("Group: Appraiser"));
    assert!(rendered.contains("Cost: 250 gold"));
    assert!(rendered.contains("Status: Available"));
    assert!(rendered.contains("You need 150 more gold."));
    assert!(rendered.contains("Benefit: Improve sell prices from 25% to 30%."));
}

#[test]
fn town_project_board_colors_completed_and_purchasable_rows() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.gold = 1_000;
    complete_project_for_test(&mut c, TownProject::RebuildForge);
    let selected = TOWN_PROJECTS
        .iter()
        .position(|definition| definition.project == TownProject::SocketBench)
        .unwrap();
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();

    terminal
        .draw(|frame| render_town_projects_screen(frame, &c, selected, ""))
        .unwrap();

    assert_eq!(
        cell_fg_at_text(&terminal, "Rebuild the Forge"),
        TEXT_MUTED_COLOR
    );
    assert_eq!(cell_fg_at_text(&terminal, "Hire Appraiser"), ACTION_COLOR);
}

#[test]
fn wide_town_project_board_gives_details_half_width_without_wrapping_project_names() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(120, 28)).unwrap();

    terminal
        .draw(|frame| render_town_projects_screen(frame, &c, 0, ""))
        .unwrap();
    let lines = backend_lines(&terminal);
    let header = lines
        .iter()
        .find(|line| line.contains("Projects") && line.contains("Details"))
        .unwrap();

    assert_eq!(char_index(header, "Details"), 61);
    assert!(
        lines
            .iter()
            .any(|line| line.contains("Quartermaster Ledger"))
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.trim_start().starts_with("Ledger"))
    );
}

#[test]
fn completing_town_project_spends_gold_and_records_completion() {
    let mut c = test_character();
    c.gold = 150;

    let message = complete_town_project(&mut c, TownProject::RebuildForge);

    assert_eq!(message, "Completed project: Rebuild the Forge.");
    assert_eq!(c.gold, 0);
    assert!(has_completed_project(&c, TownProject::RebuildForge));
}

#[test]
fn completed_and_unaffordable_town_projects_do_not_change_state() {
    let mut c = test_character();
    c.gold = 149;

    let message = complete_town_project(&mut c, TownProject::RebuildForge);

    assert_eq!(message, "Need 150 gold to complete Rebuild the Forge.");
    assert_eq!(c.gold, 149);
    assert!(!has_completed_project(&c, TownProject::RebuildForge));

    c.gold = 150;
    assert_eq!(
        complete_town_project(&mut c, TownProject::RebuildForge),
        "Completed project: Rebuild the Forge."
    );
    assert_eq!(
        complete_town_project(&mut c, TownProject::RebuildForge),
        "Rebuild the Forge is already complete."
    );
    assert_eq!(c.gold, 0);
    assert_eq!(
        c.completed_town_projects
            .iter()
            .filter(|project| **project == TownProject::RebuildForge)
            .count(),
        1
    );
}

#[test]
fn saved_character_without_town_projects_defaults_to_empty_projects() {
    let json = r#"{
        "name": "Legacy",
        "class_name": "Ironbound",
        "death_mode": "Softcore",
        "level": 1,
        "xp": 0,
        "gold": 50,
        "strength": 6,
        "dexterity": 3,
        "intelligence": 1,
        "hp": 40,
        "mana": 15,
        "inventory": {"columns": 4, "rows": 4, "items": []},
        "stash": {"columns": 8, "rows": 8, "items": []},
        "equipped_weapon": {
            "name": "Rusted Sword",
            "kind": "Weapon",
            "value": 20,
            "damage_min": 3,
            "damage_max": 5
        },
        "equipped_armor": {
            "name": "Cloth Tunic",
            "kind": "Armor",
            "value": 12,
            "armor": 1
        },
        "equipped_shield": {
            "name": "Worn Shield",
            "kind": "Shield",
            "value": 40,
            "armor": 1,
            "dodge": 2
        },
        "bellkeeper_defeated": false
    }"#;

    let c: Character = serde_json::from_str(json).unwrap();

    assert_eq!(c.class_name(), "Warrior");
    assert!(c.completed_town_projects.is_empty());
}

#[test]
fn saved_character_without_herbs_defaults_to_zero() {
    let json = r#"{
        "name": "Legacy",
        "class_name": "Ironbound",
        "death_mode": "Softcore",
        "level": 1,
        "xp": 0,
        "gold": 50,
        "strength": 6,
        "dexterity": 3,
        "intelligence": 1,
        "hp": 40,
        "mana": 15,
        "inventory": {"columns": 4, "rows": 4, "items": []},
        "stash": {"columns": 8, "rows": 8, "items": []},
        "equipped_weapon": {
            "name": "Rusted Sword",
            "kind": "Weapon",
            "value": 20,
            "damage_min": 3,
            "damage_max": 5
        },
        "equipped_armor": {
            "name": "Cloth Tunic",
            "kind": "Armor",
            "value": 12,
            "armor": 1
        },
        "equipped_shield": {
            "name": "Worn Shield",
            "kind": "Shield",
            "value": 40,
            "armor": 1,
            "dodge": 2
        },
        "bellkeeper_defeated": false
    }"#;

    let c: Character = serde_json::from_str(json).unwrap();

    assert_eq!(c.herbs, 0);
}

#[test]
fn xp_text_shows_level_aware_unicode_progress_bar() {
    assert_eq!(
        xp_text(2, 8, xp_required_for_next_level(2)),
        format!("{MAGENTA}Lv 2  XP ██░░░░░░░░░░░░░░░░░░ 10%{RESET}")
    );
}

#[test]
fn leveling_doubles_xp_requirements_and_grants_points() {
    let mut c = test_character();

    assert_eq!(xp_required_for_next_level(1), 40);
    assert_eq!(xp_required_for_next_level(2), 80);
    assert_eq!(xp_required_for_next_level(0), 40);
    assert_eq!(xp_required_for_next_level(32), u32::MAX);

    let levels_gained = add_xp(&mut c, 39);
    assert!(levels_gained.is_empty());
    assert_eq!(c.level, 1);
    assert_eq!(c.xp, 39);

    let levels_gained = add_xp(&mut c, 1);
    assert_eq!(levels_gained, vec![2]);
    assert_eq!(c.level, 2);
    assert_eq!(c.xp, 0);
    assert_eq!(c.unspent_attributes, 3);
    assert_eq!(c.unspent_skills, 1);

    let levels_gained = add_xp(&mut c, 80);
    assert_eq!(levels_gained, vec![3]);
    assert_eq!(c.level, 3);
    assert_eq!(c.xp, 0);
    assert_eq!(c.unspent_attributes, 6);
    assert_eq!(c.unspent_skills, 2);
}

#[test]
fn rogue_level_up_restores_energy() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.rogue.energy = 0;

    let levels_gained = add_xp(&mut c, 40);

    assert_eq!(levels_gained, vec![2]);
    assert_eq!(c.rogue.energy, ROGUE_MAX_ENERGY);
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
    assert_eq!(backstab_base_percent_for_rank(1), 90);
    assert_eq!(backstab_base_percent_for_rank(5), 110);
    assert_eq!(empowered_backstab_percent_for_rank(1), 120);
    assert_eq!(empowered_backstab_percent_for_rank(5), 160);
    assert_eq!(venom_edge_percent_for_rank(1), 70);
    assert_eq!(venom_edge_percent_for_rank(5), 90);
    assert_eq!(rupture_poison_duration_for_rank(1), 3);
    assert_eq!(rupture_poison_duration_for_rank(5), 7);
    assert_eq!(eviscerate_bonus_percent_for_rank(1), 0);
    assert_eq!(eviscerate_bonus_percent_for_rank(5), 40);
    assert_eq!(smoke_step_dodge_bonus_for_rank(1), 20);
    assert_eq!(smoke_step_dodge_bonus_for_rank(5), 32);
    assert_eq!(slip_away_dodge_bonus_for_rank(1), 5);
    assert_eq!(slip_away_dodge_bonus_for_rank(5), 13);
    assert_eq!(next_skill_rank(5), 5);
}

#[test]
fn rogue_skill_upgrades_spend_points_and_scale_values() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.unspent_skills = 1;

    assert_eq!(
        choose_skill_or_mastery(&mut c, "Backstab"),
        "Upgraded Backstab to rank 2."
    );
    assert_eq!(c.rogue.backstab_rank, 2);
    assert_eq!(c.unspent_skills, 0);
    assert!(backstab_base_percent_for_rank(2) > backstab_base_percent_for_rank(1));
}

#[test]
fn skill_help_helpers_reflect_masteries() {
    let mut c = test_character();

    assert_eq!(cleave_target_help(&c), "up to 3 adjacent enemies");
    assert_eq!(shield_bash_range_help(&c), "1 adjacent enemy");
    assert_eq!(shield_bash_stun_turns(&c), 1);
    assert_eq!(shield_bash_stun_help(&c), "1 turn");
    assert_eq!(battle_cry_charge_count(&c), 5);

    c.warrior.cleave_mastery = Some(SkillMastery::ReapingCleave);
    c.warrior.shield_bash_mastery = Some(SkillMastery::LongBash);
    c.warrior.battle_cry_mastery = Some(SkillMastery::WarpathCry);
    assert_eq!(cleave_target_help(&c), "every adjacent enemy");
    assert_eq!(
        shield_bash_range_help(&c),
        "1 enemy up to 2 tiles in a clear cardinal line"
    );
    assert_eq!(battle_cry_charge_count(&c), 7);

    c.warrior.shield_bash_mastery = Some(SkillMastery::DazingBash);
    assert_eq!(shield_bash_stun_turns(&c), 2);
    assert_eq!(shield_bash_stun_help(&c), "2 turns");
}

#[test]
fn battle_cry_charges_survive_movement_and_spend_on_attacks() {
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![rat(4, 2)]));

    assert!(use_battle_cry(&mut c));
    assert_eq!(c.warrior.battle_cry_charges, 5);

    assert!(try_move(&mut c, 1, 0));
    tick_player_effects(&mut c);
    assert_eq!(c.warrior.battle_cry_charges, 5);

    assert!(try_move(&mut c, 1, 0));
    assert_eq!(c.warrior.battle_cry_charges, 4);
}

#[test]
fn passive_skill_upgrades_require_branch_starter_rank_two() {
    let mut c = test_character();
    assert!(unmet_skill_prerequisite(&c, "Deep Cut").is_some());
    assert!(unmet_skill_prerequisite(&c, "Iron Guard").is_some());
    assert!(unmet_skill_prerequisite(&c, "Second Wind").is_some());

    c.unspent_skills = 6;
    c.warrior.cleave_rank = 2;
    c.warrior.shield_bash_rank = 2;
    c.warrior.battle_cry_rank = 2;
    upgrade_skill(&mut c, "Deep Cut");
    upgrade_skill(&mut c, "Iron Guard");
    upgrade_skill(&mut c, "Second Wind");

    assert_eq!(c.warrior.deep_cut_rank, 1);
    assert_eq!(c.warrior.iron_guard_rank, 1);
    assert_eq!(c.warrior.second_wind_rank, 1);
    assert_eq!(c.armor(), 4);

    let mut rogue = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    assert!(unmet_skill_prerequisite(&rogue, "Eviscerate").is_some());
    assert!(unmet_skill_prerequisite(&rogue, "Rupture").is_some());
    assert!(unmet_skill_prerequisite(&rogue, "Slip Away").is_some());

    rogue.unspent_skills = 6;
    rogue.rogue.backstab_rank = 2;
    rogue.rogue.venom_edge_rank = 2;
    rogue.rogue.smoke_step_rank = 2;
    upgrade_skill(&mut rogue, "Eviscerate");
    upgrade_skill(&mut rogue, "Rupture");
    upgrade_skill(&mut rogue, "Slip Away");

    assert_eq!(rogue.rogue.eviscerate_rank, 1);
    assert_eq!(rogue.rogue.rupture_rank, 1);
    assert_eq!(rogue.rogue.slip_away_rank, 1);
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
    let mut inventory = ItemGrid::new(4, 4, vec![health_potion(), mana_potion(), crude_axe()]);
    let mut stash = ItemGrid::new(8, 8, Vec::new());

    let message = move_selected(&mut inventory, &mut stash, 1, "Stored");

    assert!(message.starts_with("Stored Lesser Mana Potion"));
    assert_eq!(inventory.len(), 2);
    assert_eq!(stash.len(), 1);
    assert!(matches!(stash[0].kind, ItemKind::ManaPotion));
    assert!(matches!(inventory[0].kind, ItemKind::HealthPotion));
    assert!(matches!(inventory[1].kind, ItemKind::Weapon));
}

#[test]
fn stash_move_requires_destination_capacity() {
    let mut inventory = ItemGrid::new(2, 1, vec![health_potion()]);
    let mut stash = ItemGrid::new(1, 1, vec![mana_potion()]);

    let message = move_selected(&mut inventory, &mut stash, 0, "Stored");

    assert_eq!(message, "No room in destination.");
    assert_eq!(inventory.len(), 1);
    assert_eq!(stash.len(), 1);
    assert!(matches!(inventory[0].kind, ItemKind::HealthPotion));
    assert!(matches!(stash[0].kind, ItemKind::ManaPotion));
}

#[test]
fn blacksmith_salvage_converts_gear_to_type_shards() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::RebuildForge);
    c.inventory.clear();
    c.inventory.push(crude_axe());
    c.inventory.push(health_potion());

    let message = salvage_inventory_item(&mut c, 0);

    assert!(message.contains("weapon shard"));
    assert_eq!(c.weapon_shards, 1);
    assert_eq!(c.inventory.len(), 1);
    assert!(matches!(c.inventory[0].kind, ItemKind::HealthPotion));

    c.inventory.push(item_with_rarity(
        "Iron Helm",
        ItemKind::Helm,
        25,
        item_stats(0, 0, 2, 0, -1),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    ));
    let helm_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::Helm))
        .unwrap();
    let helm_message = salvage_inventory_item(&mut c, helm_index);
    assert!(helm_message.contains("armor shard"));
    assert_eq!(c.armor_shards, 1);
    assert_eq!(c.inventory.len(), 1);
    assert!(matches!(c.inventory[0].kind, ItemKind::HealthPotion));
    assert!(salvage_inventory_item(&mut c, 0).contains("Only equipment"));
}

#[test]
fn salvage_rejects_carried_gear_with_filled_sockets() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::RebuildForge);
    c.inventory.clear();
    let mut axe = crude_axe();
    axe.sockets = vec![Some(GemSocket::filled(GemKind::Ruby, GemTier::Chipped))];
    c.inventory.push(axe);

    assert_eq!(
        salvage_inventory_item(&mut c, 0),
        "Remove socketed gems before salvaging this item."
    );
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(c.weapon_shards, 0);
    assert_eq!(
        c.inventory[0].sockets[0],
        Some(GemSocket::filled(GemKind::Ruby, GemTier::Chipped))
    );
}

#[test]
fn blacksmith_upgrades_equipped_gear_with_shards_only_after_forge_project() {
    let mut c = test_character();
    c.weapon_shards = 2;
    c.armor_shards = 2;
    c.shield_shards = 2;
    c.gold = 0;

    assert_eq!(
        upgrade_equipped_message(&mut c, UpgradeSlot::Weapon),
        "Rebuild the Forge before upgrading gear."
    );

    complete_project_for_test(&mut c, TownProject::RebuildForge);

    let weapon_message = upgrade_equipped_message(&mut c, UpgradeSlot::Weapon);
    assert_eq!(weapon_message, "Upgraded Rusted Sword (3-5 dmg) to +1.");
    assert_eq!(c.equipped_weapon.upgrade_level, 1);
    assert_eq!(c.equipped_weapon.damage_min, 4);
    assert_eq!(c.equipped_weapon.damage_max, 6);
    assert_eq!(c.weapon_shards, 0);
    assert_eq!(c.gold, 0);

    let armor_message = upgrade_equipped_message(&mut c, UpgradeSlot::Armor);
    assert_eq!(armor_message, "Upgraded Cloth Tunic (+1 armor) to +1.");
    assert_eq!(c.equipped_armor.armor, 2);

    let shield_message = upgrade_equipped_message(&mut c, UpgradeSlot::Shield);
    assert_eq!(
        shield_message,
        "Upgraded Worn Shield (+1 armor, +2 dodge) to +1."
    );
    assert_eq!(c.equipped_shield.armor, 2);
}

#[test]
fn blacksmith_upgrade_cost_scales_with_upgrade_level() {
    let mut item = rusted_sword();
    assert_eq!(upgrade_cost(&item), 2);
    upgrade_item(&mut item);
    assert_eq!(upgrade_cost(&item), 4);
}

#[test]
fn salvage_requires_forge_and_reinforced_anvil_adds_one_shard() {
    let mut c = test_character();
    c.inventory.push(crude_axe());

    assert_eq!(
        salvage_inventory_item(&mut c, 0),
        "Rebuild the Forge before salvaging gear."
    );
    assert_eq!(c.weapon_shards, 0);

    complete_project_for_test(&mut c, TownProject::RebuildForge);
    let health_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::HealthPotion))
        .unwrap();
    assert_eq!(
        salvage_inventory_item(&mut c, health_index),
        "Only equipment can be salvaged."
    );

    let axe_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::Weapon))
        .unwrap();
    assert_eq!(
        salvage_inventory_item(&mut c, axe_index),
        "Salvaged Crude Axe (4-6 dmg) into 1 weapon shard(s)."
    );
    assert_eq!(c.weapon_shards, 1);

    c.inventory.push(crude_axe());
    complete_project_for_test(&mut c, TownProject::ReinforcedAnvil);
    let axe_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::Weapon))
        .unwrap();
    assert_eq!(
        salvage_inventory_item(&mut c, axe_index),
        "Salvaged Crude Axe (4-6 dmg) into 2 weapon shard(s)."
    );
    assert_eq!(c.weapon_shards, 3);
}

#[test]
fn appraiser_project_improves_sell_value() {
    let mut c = test_character();
    let item = crude_axe();

    assert_eq!(sell_value(&c, &item), 15);

    complete_project_for_test(&mut c, TownProject::HireAppraiser);
    assert_eq!(sell_value(&c, &item), 18);
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
fn equipping_into_empty_accessory_slot_does_not_add_placeholder_to_inventory() {
    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(item_with_rarity(
        "Iron Helm",
        ItemKind::Helm,
        25,
        item_stats(0, 0, 2, 0, -1),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    ));

    let result = equip_or_use_inventory_item(&mut c, 0);

    assert!(result.spent_turn);
    assert_eq!(result.message, "Equipped Iron Helm.");
    assert_eq!(c.equipped_helm.name, "Iron Helm");
    assert!(c.inventory.is_empty());
}

#[test]
fn rings_fill_empty_second_slot_before_replacing_first_ring() {
    let mut c = test_character();
    c.inventory.clear();
    c.equipped_ring1 = item_with_rarity(
        "Copper Ring",
        ItemKind::Ring,
        10,
        item_stats(0, 0, 0, 1, 0),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    );
    c.inventory.push(item_with_rarity(
        "Silver Ring",
        ItemKind::Ring,
        20,
        item_stats(0, 0, 0, 2, 0),
        Rarity::Magic,
        1,
        requirements(0, 0, 0),
    ));

    equip_or_use_inventory_item(&mut c, 0);

    assert_eq!(c.equipped_ring1.name, "Copper Ring");
    assert_eq!(c.equipped_ring2.name, "Silver Ring");
    assert!(c.inventory.is_empty());
}

#[test]
fn equipping_when_bag_is_full_reuses_selected_cell_for_old_gear() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(1, 1, vec![crude_axe()]);

    let result = equip_or_use_inventory_item(&mut c, 0);

    assert_eq!(result.message, "Equipped Crude Axe (4-6 dmg).");
    assert!(result.spent_turn);
    assert!(c.equipped_weapon.name.starts_with("Crude Axe"));
    assert_eq!(c.inventory.len(), 1);
    assert!(c.inventory[0].name.starts_with("Rusted Sword"));
}

#[test]
fn equipping_replacement_gear_clamps_hp_and_mana_after_socket_bonus_loss() {
    let mut c = test_character();
    c.equipped_armor.sockets = vec![
        Some(GemSocket::filled(GemKind::Ruby, GemTier::Pristine)),
        Some(GemSocket::filled(GemKind::Sapphire, GemTier::Pristine)),
    ];
    c.hp = c.max_hp();
    c.mana = c.max_mana();
    let socketed_max_hp = c.max_hp();
    let socketed_max_mana = c.max_mana();
    c.inventory.clear();
    c.inventory.push(cloth_tunic());

    let result = equip_or_use_inventory_item(&mut c, 0);

    assert!(result.spent_turn);
    assert_eq!(c.hp, c.max_hp());
    assert_eq!(c.mana, c.max_mana());
    assert!(c.max_hp() < socketed_max_hp);
    assert!(c.max_mana() < socketed_max_mana);
    assert!(
        c.inventory
            .iter()
            .any(|item| item.sockets.iter().any(Option::is_some))
    );
}

#[test]
fn dungeon_inventory_enter_actions_resolve_turn_and_keep_menu_open() {
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    c.inventory.clear();
    c.inventory.push(crude_axe());

    let result = finish_inventory_enter_action_for_test(&mut c, 0).unwrap();

    assert_eq!(result.flow, InventoryMenuFlow::StayOpen);
    assert_eq!(result.message, "Equipped Crude Axe (4-6 dmg).");
    assert!(c.equipped_weapon.name.starts_with("Crude Axe"));
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.log_turn, 1);
    assert_eq!(d.log[0], "== Turn 1: Inventory ==");
    assert_eq!(d.log[1], "[INFO] Equipped Crude Axe (4-6 dmg).");

    c.hp = c.max_hp() - 1;
    let health_index = c.inventory.len();
    assert!(c.inventory.push(health_potion()));

    let result = finish_inventory_enter_action_for_test(&mut c, health_index).unwrap();

    assert_eq!(result.flow, InventoryMenuFlow::StayOpen);
    assert_eq!(
        result.message,
        "Used a lesser health potion and restored 1 HP."
    );
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.log_turn, 2);
    assert_eq!(d.log[2], "== Turn 2: Inventory ==");
    assert_eq!(
        d.log[3],
        "[INFO] Used a lesser health potion and restored 1 HP."
    );
}

#[test]
fn successful_inventory_actions_spend_dungeon_turns() {
    let mut c = test_character();
    c.inventory.push(crude_axe());
    let axe_index = c.inventory.len() - 1;
    assert!(equip_or_use_inventory_item(&mut c, axe_index).spent_turn);

    c.hp = 1;
    let potion_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::HealthPotion))
        .unwrap();
    assert!(equip_or_use_inventory_item(&mut c, potion_index).spent_turn);

    assert!(!drop_selected_inventory_item(&mut c, 0).spent_turn);

    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    c.inventory.push(rusted_sword());
    let sword_index = c.inventory.len() - 1;
    assert!(drop_selected_inventory_item(&mut c, sword_index).spent_turn);
    assert!(!equip_or_use_inventory_item(&mut c, usize::MAX).spent_turn);
    assert!(!drop_selected_inventory_item(&mut c, usize::MAX).spent_turn);
}

fn fill_inventory_to_capacity(c: &mut Character) {
    c.inventory.clear();
    while c.inventory.has_space() {
        assert!(c.inventory.push(health_potion()));
    }
}

#[test]
fn full_inventory_monster_loot_goes_to_ground() {
    let mut c = test_character();
    fill_inventory_to_capacity(&mut c);
    let mut d = open_test_dungeon(2, 2, vec![skeleton(4, 2)]);

    maybe_drop_loot_in_dungeon(&mut c, &mut d, 0, true);

    assert_eq!(c.inventory.len(), c.inventory.capacity());
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (4, 2));
}

#[test]
fn monster_loot_pool_does_not_drop_potions() {
    let mut c = test_character();
    c.inventory.clear();
    let mut d = open_test_dungeon(2, 2, vec![skeleton(4, 2)]);
    let mut saw_enemy_item_drop = false;

    for _ in 0..2_000 {
        c.inventory.clear();
        maybe_drop_loot_in_dungeon(&mut c, &mut d, 0, false);

        for item in c.inventory.iter() {
            assert!(
                !matches!(item.kind, ItemKind::HealthPotion | ItemKind::ManaPotion),
                "enemy loot dropped potion: {}",
                item.name
            );
            saw_enemy_item_drop = true;
        }
    }

    assert!(
        saw_enemy_item_drop,
        "expected sampled enemy loot to drop at least one item"
    );
}

#[test]
fn dungeon_loot_goes_to_ground_when_inventory_is_full() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(1, 1, vec![health_potion()]);
    let mut d = open_test_dungeon(2, 2, Vec::new());
    let item = mana_potion();

    let added_to_bag = add_loot_to_bag_or_ground(&mut c, &mut d, item, 2, 2, "Dropped");

    assert!(!added_to_bag);
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (2, 2));
    assert!(matches!(d.ground_items[0].item.kind, ItemKind::ManaPotion));
}

#[test]
fn dungeon_loot_goes_to_bag_when_inventory_has_space() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(2, 1, vec![health_potion()]);
    let mut d = open_test_dungeon(2, 2, Vec::new());

    let added_to_bag = add_loot_to_bag_or_ground(&mut c, &mut d, mana_potion(), 2, 2, "Dropped");

    assert!(added_to_bag);
    assert_eq!(c.inventory.len(), 2);
    assert!(d.ground_items.is_empty());
}

#[test]
fn dungeon_map_renders_ground_item_glyph() {
    let mut d = open_test_dungeon(1, 1, Vec::new());
    d.ground_items.push(GroundItem {
        x: 3,
        y: 4,
        item: health_potion(),
    });

    let lines = dungeon_map_lines_for_test(&d);

    assert_eq!(lines[4].chars().nth(3), Some(LOOT_GLYPH));
}

#[test]
fn pickup_ground_item_adds_to_inventory_when_space_exists() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(4, 4, Vec::new());
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: mana_potion(),
    });
    c.active_dungeon = Some(d);

    assert!(pickup_ground_items_on_player(&mut c));

    assert_eq!(c.inventory.len(), 1);
    assert!(c.active_dungeon.as_ref().unwrap().ground_items.is_empty());
}

#[test]
fn pickup_ground_item_keeps_item_on_ground_when_inventory_is_full() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(1, 1, vec![health_potion()]);
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: mana_potion(),
    });
    c.active_dungeon = Some(d);

    assert!(!pickup_ground_items_on_player(&mut c));

    assert_eq!(c.inventory.len(), 1);
    assert_eq!(c.active_dungeon.as_ref().unwrap().ground_items.len(), 1);
}

#[test]
fn ground_loot_picker_pickup_removes_selected_item() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(4, 4, Vec::new());
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: health_potion(),
    });
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: mana_potion(),
    });
    c.active_dungeon = Some(d);

    let message = pick_up_ground_item_by_tile_index(&mut c, 1);

    assert_eq!(message, "Picked up Lesser Mana Potion (restores 15% mana).");
    assert_eq!(c.inventory.len(), 1);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.ground_items.len(), 1);
    assert!(matches!(
        d.ground_items[0].item.kind,
        ItemKind::HealthPotion
    ));
}

#[test]
fn ground_loot_picker_successful_pickup_spends_turn() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(4, 4, Vec::new());
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: mana_potion(),
    });
    c.active_dungeon = Some(d);

    let result = pick_up_selected_ground_loot_for_picker(&mut c, 0);

    assert_eq!(
        result.message,
        "Picked up Lesser Mana Potion (restores 15% mana)."
    );
    assert!(result.spent_turn);
    assert_eq!(c.inventory.len(), 1);
    assert!(c.active_dungeon.as_ref().unwrap().ground_items.is_empty());
}

#[test]
fn ground_loot_picker_full_bag_pickup_does_not_spend_turn() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(1, 1, vec![health_potion()]);
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: mana_potion(),
    });
    c.active_dungeon = Some(d);

    let result = pick_up_selected_ground_loot_for_picker(&mut c, 0);

    assert_eq!(result.message, "Inventory full.");
    assert!(!result.spent_turn);
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(c.active_dungeon.as_ref().unwrap().ground_items.len(), 1);
}

#[test]
fn ground_loot_picker_discard_removes_only_selected_item() {
    let mut c = test_character();
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: health_potion(),
    });
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: mana_potion(),
    });
    c.active_dungeon = Some(d);

    let message = discard_ground_item_by_tile_index(&mut c, 0);

    assert_eq!(message, "Discarded Lesser Health Potion (restores 15% HP).");
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.ground_items.len(), 1);
    assert!(matches!(d.ground_items[0].item.kind, ItemKind::ManaPotion));
}

#[test]
fn ground_loot_picker_successful_discard_spends_turn() {
    let mut c = test_character();
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: health_potion(),
    });
    c.active_dungeon = Some(d);

    let result = discard_selected_ground_loot_for_picker(&mut c, 0);

    assert_eq!(
        result.message,
        "Discarded Lesser Health Potion (restores 15% HP)."
    );
    assert!(result.spent_turn);
    assert!(c.active_dungeon.as_ref().unwrap().ground_items.is_empty());
}

#[test]
fn ground_loot_picker_invalid_discard_does_not_spend_turn() {
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    let result = discard_selected_ground_loot_for_picker(&mut c, 0);

    assert_eq!(result.message, "No item selected.");
    assert!(!result.spent_turn);
}

fn character_for_class(class: CharacterClass) -> Character {
    Character::new(
        format!("{} Tester", class.name()),
        class,
        DeathMode::Softcore,
    )
}

#[test]
fn chest_loot_pool_does_not_drop_potions_to_inventory() {
    for class in [
        CharacterClass::Warrior,
        CharacterClass::Rogue,
        CharacterClass::Sorceress,
    ] {
        let mut saw_chest_item = false;

        for _ in 0..1_000 {
            let mut c = character_for_class(class);
            c.inventory.clear();
            let mut d = open_test_dungeon(5, 5, Vec::new());
            d.chests.push(Chest {
                x: 5,
                y: 5,
                opened: false,
            });
            c.active_dungeon = Some(d);

            open_chest_on_player(&mut c);

            assert_eq!(c.inventory.len(), 1);
            for item in c.inventory.iter() {
                assert!(
                    !matches!(item.kind, ItemKind::HealthPotion | ItemKind::ManaPotion),
                    "{} chest loot dropped potion: {}",
                    class.name(),
                    item.name
                );
                saw_chest_item = true;
            }
        }

        assert!(saw_chest_item, "expected {} chest item loot", class.name());
    }
}

#[test]
fn full_inventory_chest_loot_drops_non_potion_item_to_ground() {
    for class in [
        CharacterClass::Warrior,
        CharacterClass::Rogue,
        CharacterClass::Sorceress,
    ] {
        let mut saw_ground_item = false;

        for _ in 0..1_000 {
            let mut c = character_for_class(class);
            fill_inventory_to_capacity(&mut c);
            let mut d = open_test_dungeon(5, 5, Vec::new());
            d.chests.push(Chest {
                x: 5,
                y: 5,
                opened: false,
            });
            c.active_dungeon = Some(d);

            open_chest_on_player(&mut c);

            let d = c.active_dungeon.as_ref().unwrap();
            assert_eq!(d.ground_items.len(), 1);
            let item = &d.ground_items[0].item;
            assert!(
                !matches!(item.kind, ItemKind::HealthPotion | ItemKind::ManaPotion),
                "{} chest ground loot dropped potion: {}",
                class.name(),
                item.name
            );
            saw_ground_item = true;
        }

        assert!(
            saw_ground_item,
            "expected {} chest ground loot",
            class.name()
        );
    }
}

#[test]
fn full_inventory_chest_loot_goes_to_ground() {
    let mut c = test_character();
    fill_inventory_to_capacity(&mut c);
    let mut d = open_test_dungeon(5, 5, Vec::new());
    d.chests.push(Chest {
        x: 5,
        y: 5,
        opened: false,
    });
    c.active_dungeon = Some(d);

    open_chest_on_player(&mut c);

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.inventory.len(), c.inventory.capacity());
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (5, 5));
}

#[test]
fn full_inventory_boss_reward_goes_to_ground() {
    let mut c = test_character();
    fill_inventory_to_capacity(&mut c);
    let mut boss = skeleton(7, 6);
    boss.name = "Test Boss".to_string();
    boss.hp = 0;
    boss.is_boss = true;
    let mut d = open_test_dungeon(2, 2, vec![boss]);

    assert!(resolve_enemy_death(
        &mut c,
        &mut d,
        0,
        EnemyDeathCause::Effect { source: "test" },
    ));

    assert_eq!(c.inventory.len(), c.inventory.capacity());
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (7, 6));
}

#[test]
fn full_inventory_boss_reward_retains_dungeon_after_gameplay_kill() {
    let mut c = test_character();
    fill_inventory_to_capacity(&mut c);
    let starting_capacity = c.inventory.capacity();
    let mut boss = skeleton(7, 6);
    boss.name = "Test Boss".to_string();
    boss.hp = 1;
    boss.is_boss = true;
    boss.bleed_turns = 1;
    boss.bleed_damage = 1;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![boss]));

    enemy_turns(&mut c);

    assert_eq!(c.inventory.len(), starting_capacity);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (7, 6));
    assert_eq!(living_monster_count(d), 0);
}

#[test]
fn retained_boss_dungeon_leave_does_not_grow_herbs_twice() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::HerbGarden);
    fill_inventory_to_capacity(&mut c);
    let mut boss = skeleton(7, 6);
    boss.name = "Test Boss".to_string();
    boss.hp = 1;
    boss.is_boss = true;
    boss.bleed_turns = 1;
    boss.bleed_damage = 1;
    let mut d = open_test_dungeon(2, 2, vec![boss]);
    d.floor = ACT1_FLOORS;
    c.active_dungeon = Some(d);

    enemy_turns(&mut c);
    let herbs_after_boss = c.herbs;
    assert!(herbs_after_boss > 0);
    assert!(c.active_dungeon.is_some());

    assert!(try_leave_dungeon_for_town(&mut c));

    assert_eq!(c.herbs, herbs_after_boss);
    assert!(c.active_dungeon.is_none());
}

#[test]
fn retained_boss_floor_stairs_return_to_town_after_boss_defeat() {
    let mut c = test_character();
    fill_inventory_to_capacity(&mut c);
    let mut boss = skeleton(7, 6);
    boss.name = "Test Boss".to_string();
    boss.hp = 1;
    boss.is_boss = true;
    boss.bleed_turns = 1;
    boss.bleed_damage = 1;
    let mut d = open_test_dungeon(2, 2, vec![boss]);
    d.floor = ACT1_FLOORS;
    c.active_dungeon = Some(d);

    enemy_turns(&mut c);
    {
        let d = c.active_dungeon.as_mut().unwrap();
        d.player_x = d.stairs_x;
        d.player_y = d.stairs_y;
    }

    use_stairs(&mut c);

    assert!(c.active_dungeon.is_none());
}

#[test]
fn capped_full_inventory_boss_reward_stays_grounded_without_expanding_bag() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(MAX_BAG_COLUMNS, MAX_BAG_ROWS, Vec::new());
    fill_inventory_to_capacity(&mut c);
    let starting_capacity = c.inventory.capacity();
    let mut boss = skeleton(7, 6);
    boss.name = "Test Boss".to_string();
    boss.hp = 1;
    boss.is_boss = true;
    boss.bleed_turns = 1;
    boss.bleed_damage = 1;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![boss]));

    enemy_turns(&mut c);

    assert_eq!(starting_capacity, 64);
    assert_eq!(c.inventory.len(), 64);
    assert_eq!(c.inventory.columns, MAX_BAG_COLUMNS);
    assert_eq!(c.inventory.rows, MAX_BAG_ROWS);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (7, 6));
    assert_eq!(living_monster_count(d), 0);
}

#[test]
fn player_attack_boss_overflow_finalizer_retains_dungeon() {
    let mut c = test_character();
    fill_inventory_to_capacity(&mut c);
    let mut boss = skeleton(7, 6);
    boss.name = "Test Boss".to_string();
    boss.hp = 0;
    boss.is_boss = true;
    let mut d = open_test_dungeon(2, 2, vec![boss, skeleton(4, 2)]);
    let ground_items_before_death = d.ground_items.len();

    assert!(resolve_enemy_death(
        &mut c,
        &mut d,
        0,
        EnemyDeathCause::PlayerAttack {
            verb: "hit",
            damage: 1,
            critical: false,
        },
    ));
    let outcome = finish_boss_defeat_after_player_action(&mut c, d, ground_items_before_death);

    assert_eq!(outcome, DamageEnemyOutcome::BossDefeated);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (7, 6));
    assert_eq!(living_monster_count(d), 0);
}

#[test]
fn dropping_inventory_item_in_dungeon_creates_ground_item() {
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(4, 5, Vec::new()));
    let starting_len = c.inventory.len();

    let result = drop_selected_inventory_item(&mut c, 0);

    assert!(result.spent_turn);
    assert_eq!(c.inventory.len(), starting_len - 1);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (4, 5));
}

#[test]
fn dropping_inventory_item_in_town_is_disallowed() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(4, 4, vec![mana_potion()]);

    let result = drop_selected_inventory_item(&mut c, 0);

    assert_eq!(result.message, "Drop items only inside a dungeon.");
    assert!(!result.spent_turn);
    assert_eq!(c.inventory.len(), 1);
    assert!(matches!(c.inventory[0].kind, ItemKind::ManaPotion));
}

fn backend_text(terminal: &ratatui::Terminal<ratatui::backend::TestBackend>) -> String {
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

fn backend_lines(terminal: &ratatui::Terminal<ratatui::backend::TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer();
    let width = usize::from(buffer.area.width);
    buffer
        .content()
        .chunks(width)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

fn text_has_fg_at_any_occurrence(
    terminal: &ratatui::Terminal<ratatui::backend::TestBackend>,
    needle: &str,
    color: ratatui::style::Color,
) -> bool {
    backend_lines(terminal).iter().enumerate().any(|(y, line)| {
        let mut search_start = 0;
        while let Some(relative_byte_index) = line[search_start..].find(needle) {
            let byte_index = search_start + relative_byte_index;
            let x = line[..byte_index].chars().count();
            if cell_fg_at(terminal, x, y) == color {
                return true;
            }
            search_start = byte_index + needle.len();
        }
        false
    })
}

fn cell_fg_at_text(
    terminal: &ratatui::Terminal<ratatui::backend::TestBackend>,
    needle: &str,
) -> ratatui::style::Color {
    let lines = backend_lines(terminal);
    let (y, x) = lines
        .iter()
        .enumerate()
        .find_map(|(y, line)| {
            line.find(needle)
                .map(|byte_index| (y, UnicodeWidthStr::width(&line[..byte_index])))
        })
        .unwrap();

    cell_fg_at(terminal, x, y)
}

fn cell_fg_at(
    terminal: &ratatui::Terminal<ratatui::backend::TestBackend>,
    x: usize,
    y: usize,
) -> ratatui::style::Color {
    let buffer = terminal.backend().buffer();
    let width = usize::from(buffer.area.width);
    buffer.content()[y * width + x].fg
}

fn char_index(text: &str, needle: &str) -> usize {
    text.find(needle)
        .map(|byte_index| UnicodeWidthStr::width(&text[..byte_index]))
        .unwrap()
}

#[test]
fn weapon_base_type_sets_flat_crit_chance() {
    assert_eq!(rusted_sword().crit_chance, SWORD_CRIT_CHANCE);
    assert_eq!(crude_axe().crit_chance, AXE_CRIT_CHANCE);
}

#[test]
fn weapon_rarity_does_not_change_crit_chance() {
    let common_sword = item_with_rarity(
        "Iron Sword",
        ItemKind::Weapon,
        45,
        weapon_stats(3, 5, 0, SWORD_CRIT_CHANCE),
        Rarity::Common,
        1,
        requirements(5, 3, 0),
    );
    let rare_sword = item_with_rarity(
        "Rare Iron Sword",
        ItemKind::Weapon,
        75,
        weapon_stats(5, 7, 0, SWORD_CRIT_CHANCE),
        Rarity::Rare,
        3,
        requirements(7, 5, 0),
    );

    assert_eq!(common_sword.crit_chance, SWORD_CRIT_CHANCE);
    assert_eq!(rare_sword.crit_chance, SWORD_CRIT_CHANCE);
}

#[test]
fn weapon_crit_must_be_set_explicitly() {
    let named_sword = item_with_rarity(
        "Iron Sword",
        ItemKind::Weapon,
        45,
        item_stats(3, 5, 0, 0, 0),
        Rarity::Common,
        1,
        requirements(5, 3, 0),
    );

    assert_eq!(named_sword.crit_chance, 0);
}

#[test]
fn saved_item_without_crit_chance_defaults_to_zero() {
    let saved = r#"{
        "name": "Old Iron Sword",
        "kind": "Weapon",
        "value": 45,
        "damage_min": 3,
        "damage_max": 5,
        "armor": 0,
        "dodge": 0,
        "speed": 0
    }"#;

    let item: Item = serde_json::from_str(saved).unwrap();

    assert_eq!(item.crit_chance, 0);
}

#[test]
fn generated_weapon_loot_sets_explicit_base_crit() {
    let mut saw_common_sword = false;
    let mut saw_common_axe = false;
    let mut saw_magic_or_rare_sword = false;
    let mut saw_magic_or_rare_axe = false;

    for _ in 0..2000 {
        let common_loot = random_equipment_loot(3, false);
        if common_loot.name.contains("Sword") {
            assert_eq!(common_loot.crit_chance, SWORD_CRIT_CHANCE);
            assert!(matches!(common_loot.rarity, Rarity::Common));
            saw_common_sword = true;
        } else if common_loot.name.contains("Axe") {
            assert_eq!(common_loot.crit_chance, AXE_CRIT_CHANCE);
            assert!(matches!(common_loot.rarity, Rarity::Common));
            saw_common_axe = true;
        }

        let better_loot = random_equipment_loot(3, true);
        if better_loot.name.contains("Sword") {
            assert_eq!(better_loot.crit_chance, SWORD_CRIT_CHANCE);
            assert!(matches!(better_loot.rarity, Rarity::Magic | Rarity::Rare));
            saw_magic_or_rare_sword = true;
        } else if better_loot.name.contains("Axe") {
            assert_eq!(better_loot.crit_chance, AXE_CRIT_CHANCE);
            assert!(matches!(better_loot.rarity, Rarity::Magic | Rarity::Rare));
            saw_magic_or_rare_axe = true;
        }
    }

    assert!(saw_common_sword);
    assert!(saw_common_axe);
    assert!(saw_magic_or_rare_sword);
    assert!(saw_magic_or_rare_axe);
}

#[test]
fn weapon_summary_and_comparison_show_crit_chance() {
    let mut c = test_character();
    c.equipped_weapon = rusted_sword();
    let axe = crude_axe();

    assert!(item_summary(&c.equipped_weapon).contains("crit 8%"));
    assert!(item_summary(&axe).contains("crit 5%"));
    assert!(item_comparison(&c, &axe).unwrap().contains("crit -3"));
}

#[test]
fn inventory_weapon_equipped_panel_compares_against_equipped_weapon() {
    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(crude_axe());

    let lines = selected_item_equipped_comparison_lines(&c, c.inventory.get(0))
        .iter()
        .map(line_text)
        .collect::<Vec<_>>();

    assert!(
        lines
            .iter()
            .any(|line| line == "Equipped Weapon: Rusted Sword")
    );
    assert!(lines.iter().any(|line| line == "Delta: +2 damage  crit -3"));
}

#[test]
fn inventory_armor_equipped_panel_compares_against_equipped_armor() {
    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(item_with_rarity(
        "Test Hauberk",
        ItemKind::Armor,
        20,
        item_stats(0, 0, 3, 2, -1),
        Rarity::Magic,
        1,
        requirements(0, 0, 0),
    ));

    let lines = selected_item_equipped_comparison_lines(&c, c.inventory.get(0))
        .iter()
        .map(line_text)
        .collect::<Vec<_>>();

    assert!(
        lines
            .iter()
            .any(|line| line == "Equipped Armor: Cloth Tunic")
    );
    assert!(
        lines
            .iter()
            .any(|line| line == "Delta: +2 armor  +2 dodge  -1 speed")
    );
}

#[test]
fn inventory_shield_equipped_panel_compares_against_equipped_shield() {
    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(item_with_rarity(
        "Test Guard",
        ItemKind::Shield,
        20,
        item_stats(0, 0, 3, 1, 1),
        Rarity::Magic,
        1,
        requirements(0, 0, 0),
    ));

    let lines = selected_item_equipped_comparison_lines(&c, c.inventory.get(0))
        .iter()
        .map(line_text)
        .collect::<Vec<_>>();

    assert!(
        lines
            .iter()
            .any(|line| line == "Equipped Shield: Worn Shield")
    );
    assert!(
        lines
            .iter()
            .any(|line| line == "Delta: +2 armor  -1 dodge  +1 speed")
    );
}

#[test]
fn inventory_accessory_equipped_panel_compares_against_matching_slot() {
    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(item_with_rarity(
        "Test Boots",
        ItemKind::Boots,
        20,
        item_stats(0, 0, 1, 3, 2),
        Rarity::Magic,
        1,
        requirements(0, 0, 0),
    ));

    let lines = selected_item_equipped_comparison_lines(&c, c.inventory.get(0))
        .iter()
        .map(line_text)
        .collect::<Vec<_>>();

    assert!(
        lines
            .iter()
            .any(|line| line == "Equipped Boots: Nothing equipped")
    );
    assert!(
        lines
            .iter()
            .any(|line| line == "Delta: +1 armor  +3 dodge  +2 speed")
    );
}

#[test]
fn inventory_locked_gear_comparison_shows_cannot_equip_reason() {
    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(item_with_rarity(
        "Heavy Test Axe",
        ItemKind::Weapon,
        100,
        item_stats(8, 10, 0, 0, -1),
        Rarity::Rare,
        5,
        requirements(10, 0, 0),
    ));

    let lines = selected_item_equipped_comparison_lines(&c, c.inventory.get(0))
        .iter()
        .map(line_text)
        .collect::<Vec<_>>();

    assert!(
        lines
            .iter()
            .any(|line| line == "Cannot equip: Requires STR 6/10.")
    );
}

#[test]
fn item_requirements_gate_equipping() {
    let c = test_character();
    let high_level_axe = item_with_rarity(
        "Test Axe",
        ItemKind::Weapon,
        100,
        item_stats(8, 10, 0, 0, -1),
        Rarity::Rare,
        5,
        requirements(10, 0, 0),
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
        item_stats(4, 6, 0, 0, -1),
        Rarity::Common,
        1,
        requirements(6, 0, 0),
    );
    let high = item_with_rarity(
        "High Axe",
        ItemKind::Weapon,
        120,
        item_stats(8, 10, 0, 0, -1),
        Rarity::Rare,
        5,
        requirements(10, 0, 0),
    );

    assert!(high.item_level > low.item_level);
    assert!(high.damage_max > low.damage_max);
    assert!(high.required_strength > low.required_strength);
    assert!(item_summary(&high).contains("ilvl 5"));
}

#[test]
fn boss_reward_loot_is_always_magic_or_rare_equipment() {
    for _ in 0..100 {
        let loot = random_equipment_loot(ACT1_FLOORS, true);
        assert!(matches!(
            loot.kind,
            ItemKind::Weapon
                | ItemKind::Armor
                | ItemKind::Shield
                | ItemKind::Helm
                | ItemKind::Gloves
                | ItemKind::Boots
                | ItemKind::Belt
                | ItemKind::Amulet
                | ItemKind::Ring
        ));
        assert!(matches!(loot.rarity, Rarity::Magic | Rarity::Rare));
    }
}

#[test]
fn hit_chance_compares_attacker_hit_against_target_dodge() {
    assert_eq!(hit_chance(25, 25), 0.5);
    assert_eq!(hit_chance(25, 10), 25.0 / 35.0);
    assert_eq!(hit_chance(1, 1000), 0.20);
    assert_eq!(hit_chance(1000, 1), 0.95);
}

#[test]
fn player_attack_hit_chance_uses_enemy_dodge_rating() {
    let c = test_character();
    let mut enemy = skeleton(3, 2);

    let hit = c.hit_rating() as f64;

    enemy.dodge_rating = 10;
    assert_eq!(player_attack_hit_chance(&c, &enemy), hit / (hit + 10.0));

    enemy.dodge_rating = 25;
    assert_eq!(player_attack_hit_chance(&c, &enemy), hit / (hit + 25.0));
}

#[test]
fn enemy_attack_hit_chance_uses_enemy_hit_rating() {
    let c = test_character();
    let mut enemy = skeleton(3, 2);
    let dodge = defensive_dodge_rating(&c) as f64;

    enemy.hit_rating = 25;
    assert_eq!(enemy_attack_hit_chance(&enemy, &c), 25.0 / (25.0 + dodge));

    enemy.hit_rating = 50;
    assert_eq!(enemy_attack_hit_chance(&enemy, &c), 50.0 / (50.0 + dodge));
}

#[test]
fn floor_difficulty_is_doubled_across_act_one() {
    assert_eq!(floor_difficulty_multiplier(1), 2.0);
    assert_eq!(floor_difficulty_multiplier(ACT1_FLOORS), 4.0);
    assert_eq!(floor_reward_multiplier(1), 1.0);
    assert_eq!(floor_reward_multiplier(ACT1_FLOORS), 2.0);

    let baseline = skeleton(1, 1);
    let early = scale_enemy_for_floor(skeleton(1, 1), 1);
    let late = scale_enemy_for_floor(skeleton(1, 1), ACT1_FLOORS);

    assert_eq!(early.max_hp, baseline.max_hp * 2);
    assert_eq!(early.damage_min, baseline.damage_min * 2);
    assert_eq!(late.max_hp, baseline.max_hp * 4);
    assert_eq!(late.damage_min, baseline.damage_min * 4);
    assert!(late.armor > early.armor);
    assert!(late.hit_rating > early.hit_rating);
    assert!(late.dodge_rating > early.dodge_rating);
    assert_eq!(late.xp, baseline.xp * 2);
}

#[test]
fn saved_enemy_without_hit_or_dodge_defaults_to_baseline_ratings() {
    let json = r#"{
        "name": "Legacy Skeleton",
        "glyph": "s",
        "x": 4,
        "y": 5,
        "hp": 12,
        "max_hp": 12,
        "damage_min": 2,
        "damage_max": 4,
        "armor": 1,
        "speed": 9,
        "xp": 18,
        "gold_min": 2,
        "gold_max": 8,
        "is_boss": false
    }"#;

    let enemy: Enemy = serde_json::from_str(json).unwrap();

    assert_eq!(enemy.hit_rating, DEFAULT_ENEMY_HIT_RATING);
    assert_eq!(enemy.dodge_rating, DEFAULT_ENEMY_DODGE_RATING);
}

#[test]
fn dungeon_generation_obeys_floor_content_rules() {
    for floor in 1..=ACT1_FLOORS {
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
        let mut occupied = std::collections::HashSet::new();
        for enemy in &d.enemies {
            assert!(occupied.insert((enemy.x, enemy.y)));
        }
        for chest in &d.chests {
            assert!(occupied.insert((chest.x, chest.y)));
        }
    }

    let floor2 = generate_dungeon(2);
    let elite = floor2.enemies.iter().find(|e| e.glyph == 'E').unwrap();
    assert!(elite.elite_modifier.is_some());

    let floor9 = generate_dungeon(ACT1_FLOORS - 1);
    assert!(!floor9.enemies.iter().any(|e| e.is_boss));

    let boss_floor = generate_dungeon(ACT1_FLOORS);
    assert!(
        boss_floor
            .enemies
            .iter()
            .any(|e| e.is_boss && e.name == "Bellkeeper")
    );

    for floor in ACT2_START_FLOOR..=FINAL_FLOOR {
        let d = generate_dungeon(floor);
        assert_eq!(d.floor, floor);
        assert_eq!(act_name(d.floor), "Glass Wastes");
        assert_eq!(d.tiles.len(), (MAP_W * MAP_H) as usize);
        assert!(d.enemies.iter().all(|e| dungeon_tile(&d, e.x, e.y) == '.'));
    }
    let act2_mid = generate_dungeon(ACT2_START_FLOOR + 2);
    assert!(
        act2_mid
            .enemies
            .iter()
            .any(|e| e.name.contains("Glass Wraith"))
    );
    let act2_boss_floor = generate_dungeon(FINAL_FLOOR);
    assert!(
        act2_boss_floor
            .enemies
            .iter()
            .any(|e| e.is_boss && e.name == "Glass Tyrant")
    );
}

#[test]
fn enemy_turn_resolution_skips_changed_or_closed_dungeons() {
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    let before_floor = current_dungeon_floor(&c);

    assert!(should_resolve_enemy_turns_after_action(&c, before_floor));

    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    c.active_dungeon.as_mut().unwrap().floor = 3;
    assert!(!should_resolve_enemy_turns_after_action(&c, before_floor));

    c.active_dungeon = None;
    assert!(!should_resolve_enemy_turns_after_action(&c, before_floor));
}

#[test]
fn stairs_require_clear_floor_before_advancing() {
    let mut c = test_character();
    c.active_dungeon = Some(generate_dungeon(1));
    {
        let d = c.active_dungeon.as_mut().unwrap();
        d.player_x = d.stairs_x;
        d.player_y = d.stairs_y;
    }
    use_stairs(&mut c);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.floor, 1);
    assert!(d.log.iter().any(|line| line.contains("monsters remain")));

    c.active_dungeon.as_mut().unwrap().enemies.clear();
    use_stairs(&mut c);
    assert_eq!(c.active_dungeon.as_ref().unwrap().floor, 2);
}

#[test]
fn herb_garden_grows_herbs_when_cleared_floor_advances() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::HerbGarden);
    c.active_dungeon = Some(generate_dungeon(1));
    {
        let d = c.active_dungeon.as_mut().unwrap();
        d.player_x = d.stairs_x;
        d.player_y = d.stairs_y;
        d.enemies.clear();
    }

    use_stairs(&mut c);

    assert!((1..=3).contains(&c.herbs));
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.floor, 2);
    assert!(d.log.iter().any(|line| line.contains("Herb Garden")));
}

#[test]
fn cleared_floors_do_not_grow_herbs_without_herb_garden() {
    let mut c = test_character();
    c.active_dungeon = Some(generate_dungeon(1));
    {
        let d = c.active_dungeon.as_mut().unwrap();
        d.player_x = d.stairs_x;
        d.player_y = d.stairs_y;
        d.enemies.clear();
    }

    use_stairs(&mut c);

    assert_eq!(c.herbs, 0);
    assert_eq!(c.active_dungeon.as_ref().unwrap().floor, 2);
}

#[test]
fn herb_garden_grows_herbs_when_boss_floor_is_completed() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::HerbGarden);
    let mut boss = one_hp_test_boss(3, 2);
    boss.hp = 0;
    let mut d = open_test_dungeon(2, 2, vec![boss]);

    assert!(resolve_enemy_death(
        &mut c,
        &mut d,
        0,
        EnemyDeathCause::Effect { source: "test" }
    ));

    assert!((1..=3).contains(&c.herbs));
    assert!(c.pending_town_message.contains("Herb Garden grew"));
}

#[test]
fn returning_to_town_requires_clear_floor() {
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![rat(4, 2)]));

    assert!(!try_leave_dungeon_for_town(&mut c));
    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d.log.iter().any(|line| line.contains("1 monster remains")));

    c.active_dungeon.as_mut().unwrap().enemies[0].hp = 0;
    assert!(try_leave_dungeon_for_town(&mut c));
    assert!(c.active_dungeon.is_none());
}

#[test]
fn first_floor_of_each_act_can_return_to_town_before_clearing_monsters() {
    for floor in [1, ACT2_START_FLOOR] {
        let mut c = test_character();
        complete_project_for_test(&mut c, TownProject::HerbGarden);
        let mut d = open_test_dungeon(2, 2, vec![rat(4, 2)]);
        d.floor = floor;
        c.active_dungeon = Some(d);

        assert!(try_leave_dungeon_for_town(&mut c));
        assert!(c.active_dungeon.is_none());
        assert_eq!(c.herbs, 0);
        assert!(!c.pending_town_message.contains("Herb Garden grew"));
    }
}

#[test]
fn returning_to_town_after_clear_floor_grows_herbs() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::HerbGarden);
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(try_leave_dungeon_for_town(&mut c));

    assert!((1..=3).contains(&c.herbs));
    assert!(c.pending_town_message.contains("Herb Garden grew"));
}

#[test]
fn boss_floors_report_remaining_monsters_before_leaving() {
    let mut c = test_character();
    c.active_dungeon = Some(generate_dungeon(ACT1_FLOORS));
    {
        let d = c.active_dungeon.as_mut().unwrap();
        d.player_x = d.stairs_x;
        d.player_y = d.stairs_y;
    }
    use_stairs(&mut c);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.floor, ACT1_FLOORS);
    assert!(d.log.iter().any(|line| line.contains("monsters remain")));

    c.active_dungeon = Some(generate_dungeon(FINAL_FLOOR));
    {
        let d = c.active_dungeon.as_mut().unwrap();
        d.player_x = d.stairs_x;
        d.player_y = d.stairs_y;
    }
    use_stairs(&mut c);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.floor, FINAL_FLOOR);
    assert!(d.log.iter().any(|line| line.contains("monsters remain")));
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
fn enemy_energy_uses_speed_before_acting() {
    let mut c = test_character();
    let mut slow = boneguard(5, 2);
    slow.energy = 0;
    slow.speed = 1;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![slow]));

    enemy_turns(&mut c);

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.enemies[0].energy, 1);
    assert!(!d.enemies[0].guarding);
    assert!(!d.log.iter().any(|line| line.contains("raises its shield")));

    let mut fast = boneguard(5, 2);
    fast.energy = 0;
    fast.speed = enemy_action_energy_threshold(&c);
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![fast]));

    enemy_turns(&mut c);

    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d.enemies[0].guarding);
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
fn long_shield_bash_requires_clear_cardinal_line() {
    let mut c = test_character();
    c.warrior.shield_bash_mastery = Some(SkillMastery::LongBash);
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![skeleton(4, 2)]));

    assert_eq!(shield_bash_target_index(&c, 2), Some(0));

    let d = c.active_dungeon.as_mut().unwrap();
    d.tiles[tile_index(3, 2)] = '#';

    assert_eq!(shield_bash_target_index(&c, 2), None);
}

#[test]
fn shield_bash_only_stuns_after_surviving_hit() {
    assert!(!shield_bash_outcome_stuns(DamageEnemyOutcome::Missed));
    assert!(!shield_bash_outcome_stuns(DamageEnemyOutcome::Killed));
    assert!(!shield_bash_outcome_stuns(DamageEnemyOutcome::BossDefeated));
    assert!(shield_bash_outcome_stuns(DamageEnemyOutcome::Hit));
}

#[test]
fn shield_bash_stun_only_applies_to_surviving_targets() {
    let mut c = test_character();
    c.warrior.shield_bash_mastery = Some(SkillMastery::DazingBash);
    let mut dead = skeleton(3, 2);
    dead.hp = 0;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![dead, skeleton(4, 2)]));

    apply_shield_bash_stun(&mut c, 0);
    apply_shield_bash_stun(&mut c, 1);

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.enemies[0].stunned_turns, 0);
    assert_eq!(d.enemies[1].stunned_turns, 2);
    assert_eq!(
        d.log
            .iter()
            .filter(|line| line.contains("Shield Bash stuns"))
            .count(),
        1
    );
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

    let mut tyrant = glass_tyrant(5, 5);
    tyrant.hp = 1;
    assert_eq!(bellkeeper_enrage_damage_bonus(&tyrant), 0);
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
fn lethal_boss_special_stops_remaining_enemy_actions() {
    let mut c = test_character();
    c.hp = 1;
    let mut d = open_test_dungeon(7, 5, vec![glass_tyrant(5, 5), skeleton(7, 6)]);
    d.boss_turn_counter = 3;
    c.active_dungeon = Some(d);

    enemy_turns(&mut c);

    assert_eq!(c.hp, 0);
    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d.log.iter().any(|line| line.contains("prism burst cuts")));
    assert!(!d.log.iter().any(|line| line.contains("Skeleton")));
}

#[test]
fn smoke_protection_ticks_after_lethal_boss_special() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.hp = 1;
    c.rogue.smoke_protection_turns = 2;
    let mut d = open_test_dungeon(7, 5, vec![glass_tyrant(5, 5), skeleton(7, 6)]);
    d.boss_turn_counter = 3;
    d.enemies[0].energy = enemy_action_energy_threshold(&c);
    c.active_dungeon = Some(d);

    enemy_turns(&mut c);

    assert_eq!(c.hp, 0);
    assert_eq!(c.rogue.smoke_protection_turns, 1);
    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d.log.iter().any(|line| line.contains("prism burst cuts")));
    assert!(!d.log.iter().any(|line| line.contains("Skeleton")));
}

#[test]
fn bellkeeper_bleed_death_completes_boss_fight_even_with_mobs_left() {
    let mut boss = bellkeeper(5, 5);
    boss.hp = 1;
    boss.bleed_turns = 1;
    boss.bleed_damage = 2;
    let mut c = test_character();
    c.warrior.battle_cry_charges = 3;
    c.warrior.second_wind_shield = 5;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![boss, skeleton(4, 2)]));

    enemy_turns(&mut c);

    assert!(c.bellkeeper_defeated);
    assert!(c.active_dungeon.is_none());
    assert_eq!(c.warrior.battle_cry_charges, 0);
    assert_eq!(c.warrior.second_wind_shield, 0);
    assert_eq!(c.hp, c.max_hp());
    assert_eq!(c.mana, c.max_mana());
    assert!(c.pending_town_message.contains("Defeated Bellkeeper"));
    assert!(c.pending_town_message.contains("Boss reward"));
    assert!(c.pending_town_message.contains(TOWN_FULL_HEAL_MESSAGE));
}

#[test]
fn critical_player_death_message_marks_critical_hit() {
    let message = enemy_death_message(
        "Skeleton",
        8,
        3,
        EnemyDeathCause::PlayerAttack {
            verb: "hit",
            damage: 14,
            critical: true,
        },
    );

    assert!(message.starts_with("Critical hit! You hit Skeleton"));
    assert!(message.contains("14"));
}

#[test]
fn normal_player_death_message_keeps_existing_wording() {
    let message = enemy_death_message(
        "Skeleton",
        8,
        3,
        EnemyDeathCause::PlayerAttack {
            verb: "hit",
            damage: 7,
            critical: false,
        },
    );

    assert!(message.starts_with("You hit Skeleton"));
    assert!(!message.starts_with("Critical hit!"));
}

#[test]
fn crit_roll_handles_extreme_chances() {
    for _ in 0..100 {
        assert!(!crit_roll(0));
        assert!(crit_roll(100));
        assert!(crit_roll(250));
    }
}

#[test]
fn battle_cry_adds_flat_crit_chance_to_equipped_weapon() {
    let mut c = test_character();
    c.equipped_weapon.crit_chance = 8;

    assert_eq!(player_crit_chance(&c), 8);

    c.warrior.battle_cry_charges = 1;
    assert_eq!(player_crit_chance(&c), 13);

    c.equipped_weapon.crit_chance = 98;
    assert_eq!(player_crit_chance(&c), 100);
}

#[test]
fn player_crit_chance_includes_topaz_socket_bonus() {
    let mut c = test_character();
    c.equipped_weapon.crit_chance = 8;
    c.equipped_weapon.sockets = vec![Some(GemSocket::filled(GemKind::Topaz, GemTier::Pristine))];

    assert_eq!(player_crit_chance(&c), 12);
    assert_eq!(c.equipped_weapon.crit_chance, 8);

    c.warrior.battle_cry_charges = 1;
    assert_eq!(player_crit_chance(&c), 17);
    assert_eq!(c.equipped_weapon.crit_chance, 8);

    c.equipped_weapon.crit_chance = 98;
    assert_eq!(player_crit_chance(&c), 100);
    assert_eq!(c.equipped_weapon.crit_chance, 98);
}

#[test]
fn critical_damage_enemy_doubles_post_armor_damage_and_logs_hit() {
    for _ in 0..200 {
        let mut c = critical_combat_test_character();
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));

        if damage_enemy(&mut c, 0, 1.0, "hit") == DamageEnemyOutcome::Missed {
            continue;
        }

        let d = c.active_dungeon.as_ref().unwrap();
        assert_eq!(d.enemies[0].hp, 16);
        assert!(d.log.iter().any(|line| {
            line.contains("Critical hit! You hit Armored Dummy") && line.contains(&damage_text(14))
        }));
        return;
    }

    panic!("forced-crit damage_enemy test missed every attack");
}

#[test]
fn critical_cleave_uses_shared_damage_path() {
    for _ in 0..200 {
        let mut c = critical_combat_test_character();
        c.warrior.cleave_rank = 3;
        c.active_dungeon = Some(open_test_dungeon(2, 2, vec![armored_training_dummy(3, 2)]));

        assert!(use_cleave(&mut c));
        let d = c.active_dungeon.as_ref().unwrap();
        if d.log
            .iter()
            .any(|line| line.contains("You miss Armored Dummy"))
        {
            continue;
        }

        assert_eq!(d.enemies[0].hp, 16);
        assert!(d.log.iter().any(|line| {
            line.contains("Critical hit! You cleave Armored Dummy")
                && line.contains(&damage_text(14))
        }));
        return;
    }

    panic!("forced-crit cleave test missed every attack");
}

#[test]
fn entering_dungeon_clears_stale_combat_state() {
    let mut c = test_character();
    c.warrior.cleave_cooldown = 1;
    c.warrior.shield_bash_cooldown = 2;
    c.warrior.battle_cry_cooldown = 3;
    c.warrior.battle_cry_charges = 4;
    c.warrior.second_wind_shield = 5;
    c.pending_town_message = "old news".to_string();

    assert_eq!(enter_dungeon(&mut c), "");

    assert!(c.active_dungeon.is_some());
    assert_eq!(c.warrior.cleave_cooldown, 0);
    assert_eq!(c.warrior.shield_bash_cooldown, 0);
    assert_eq!(c.warrior.battle_cry_cooldown, 0);
    assert_eq!(c.warrior.battle_cry_charges, 0);
    assert_eq!(c.warrior.second_wind_shield, 0);
    assert!(c.pending_town_message.is_empty());
}

#[test]
fn softcore_death_clears_dungeon_and_combat_state() {
    let mut c = test_character();
    c.hp = 0;
    c.gold = 100;
    c.warrior.cleave_cooldown = 1;
    c.warrior.battle_cry_charges = 4;
    c.warrior.second_wind_shield = 5;
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    check_death(&mut c);

    assert!(c.active_dungeon.is_none());
    assert_eq!(c.hp, c.max_hp());
    assert_eq!(c.mana, c.max_mana());
    assert_eq!(c.gold, 90);
    assert_eq!(c.warrior.cleave_cooldown, 0);
    assert_eq!(c.warrior.battle_cry_charges, 0);
    assert_eq!(c.warrior.second_wind_shield, 0);
    assert!(c.pending_town_message.contains("returned to town"));
    assert!(c.pending_town_message.contains(TOWN_FULL_HEAL_MESSAGE));
}

#[test]
fn hardcore_death_deletes_save_and_returns_outcome() {
    let temp_dir =
        std::env::temp_dir().join(format!("crawltty-hardcore-death-{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let save_path = temp_dir.join("save.json");
    fs::write(&save_path, "hardcore save").unwrap();

    let mut c = Character::new(
        "Doomed".to_string(),
        CharacterClass::Warrior,
        DeathMode::Hardcore,
    );
    c.hp = 0;
    c.warrior.cleave_cooldown = 1;
    c.warrior.battle_cry_charges = 4;
    c.warrior.second_wind_shield = 5;
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    let outcome = check_death_with_save_path(&mut c, &save_path);

    assert_eq!(outcome, DeathOutcome::HardcoreDeleted);
    assert!(!save_path.exists());
    assert!(c.active_dungeon.is_none());
    assert_eq!(c.warrior.cleave_cooldown, 0);
    assert_eq!(c.warrior.battle_cry_charges, 0);
    assert_eq!(c.warrior.second_wind_shield, 0);
    assert!(c.pending_town_message.contains("Hardcore"));

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn hardcore_death_deletes_active_character_save_file() {
    let mut c = Character::new(
        format!("Doomed Hero {}", std::process::id()),
        CharacterClass::Rogue,
        DeathMode::Hardcore,
    );
    let save_path = character_save_path(&c);
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&save_path, "hardcore save").unwrap();
    assert!(save_path.exists());

    c.hp = 0;
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    let outcome = check_death(&mut c);

    assert_eq!(outcome, DeathOutcome::HardcoreDeleted);
    assert!(!save_path.exists());
    assert!(c.active_dungeon.is_none());

    let _ = fs::remove_file(&save_path);
}

#[test]
fn returning_to_town_restores_hp_and_mana_and_reports_it() {
    let mut c = test_character();
    c.hp = 1;
    c.mana = 0;

    full_heal_on_town_return(&mut c);

    assert_eq!(c.hp, c.max_hp());
    assert_eq!(c.mana, c.max_mana());
    assert_eq!(c.pending_town_message, TOWN_FULL_HEAL_MESSAGE);
}

#[test]
fn returning_to_town_restores_rogue_energy() {
    let mut c = Character::new(
        "Sneak".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    c.hp = 1;
    c.rogue.energy = 0;

    full_heal_on_town_return(&mut c);

    assert_eq!(c.hp, c.max_hp());
    assert_eq!(c.rogue.energy, ROGUE_MAX_ENERGY);
    assert_eq!(c.pending_town_message, TOWN_FULL_HEAL_MESSAGE);
}

#[test]
fn returning_to_town_full_heal_message_is_not_duplicated() {
    let mut c = test_character();
    c.pending_town_message = format!("Defeated Test Boss. {TOWN_FULL_HEAL_MESSAGE}");

    full_heal_on_town_return(&mut c);
    full_heal_on_town_return(&mut c);

    assert_eq!(
        c.pending_town_message
            .matches(TOWN_FULL_HEAL_MESSAGE)
            .count(),
        1
    );
    assert_eq!(c.hp, c.max_hp());
    assert_eq!(c.mana, c.max_mana());
}

#[test]
fn startup_keeps_pending_town_message_when_dungeon_is_active() {
    let mut c = test_character();
    c.pending_town_message = "Defeated Test Boss.".to_string();
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    let town_message = take_startup_town_message(&mut c);

    assert!(town_message.is_empty());
    assert_eq!(c.pending_town_message, "Defeated Test Boss.");
}

#[test]
fn startup_moves_pending_town_message_when_starting_in_town() {
    let mut c = test_character();
    c.pending_town_message = "Defeated Test Boss.".to_string();

    let town_message = take_startup_town_message(&mut c);

    assert_eq!(town_message, "Defeated Test Boss.");
    assert!(c.pending_town_message.is_empty());
}

#[test]
fn spiked_guard_boss_kill_completes_boss_fight() {
    let mut c = test_character();
    let mut boss = bellkeeper(5, 5);
    boss.hp = 0;
    let mut d = open_test_dungeon(2, 2, vec![boss, skeleton(4, 2)]);

    assert!(resolve_enemy_killed_by_effect(
        &mut c,
        &mut d,
        0,
        "Spiked Guard"
    ));

    assert!(c.bellkeeper_defeated);
    assert!(d.log.iter().any(|line| line.contains("Spiked Guard")));
    assert!(c.pending_town_message.contains("Defeated Bellkeeper"));
}

#[test]
fn full_inventory_spiked_guard_boss_reward_retains_dungeon_after_gameplay_kill() {
    let mut c = test_character();
    fill_inventory_to_capacity(&mut c);
    let mut boss = bellkeeper(3, 2);
    boss.hp = 0;
    let mut d = open_test_dungeon(2, 2, vec![boss, skeleton(4, 2)]);
    let ground_items_before_death = d.ground_items.len();

    assert!(resolve_enemy_killed_by_effect(
        &mut c,
        &mut d,
        0,
        "Spiked Guard"
    ));
    finish_boss_defeat_after_effect_kill(&mut c, d, ground_items_before_death);

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (3, 2));
    assert_eq!(living_monster_count(d), 0);
    assert!(d.log.iter().any(|line| line.contains("Spiked Guard")));
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

    assert!(use_potion(&mut c));

    let ending_potions = c
        .inventory
        .iter()
        .filter(|item| matches!(item.kind, ItemKind::HealthPotion))
        .count();
    assert_eq!(ending_potions, starting_potions - 1);
    assert_eq!(c.hp, 1 + lesser_potion_restore(c.max_hp()));

    c.hp = c.max_hp() - 1;
    assert!(use_potion(&mut c));
    assert_eq!(c.hp, c.max_hp());

    c.inventory.push(health_potion());
    let full_hp_potions = c
        .inventory
        .iter()
        .filter(|item| matches!(item.kind, ItemKind::HealthPotion))
        .count();
    assert!(!use_potion(&mut c));
    assert_eq!(
        c.inventory
            .iter()
            .filter(|item| matches!(item.kind, ItemKind::HealthPotion))
            .count(),
        full_hp_potions
    );

    c.inventory
        .retain(|item| !matches!(item.kind, ItemKind::HealthPotion));
    c.hp = 1;
    assert!(!use_potion(&mut c));
}

#[test]
fn inventory_potions_restore_actual_amount_and_do_not_waste_at_full() {
    let mut c = test_character();
    let health_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::HealthPotion))
        .unwrap();
    let starting_items = c.inventory.len();

    let result = equip_or_use_inventory_item(&mut c, health_index);
    assert_eq!(result.message, "HP is already full.");
    assert!(!result.spent_turn);
    assert_eq!(c.inventory.len(), starting_items);

    c.hp = c.max_hp() - 1;
    let health_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::HealthPotion))
        .unwrap();
    let result = equip_or_use_inventory_item(&mut c, health_index);
    assert_eq!(
        result.message,
        "Used a lesser health potion and restored 1 HP."
    );
    assert!(result.spent_turn);

    let mana_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::ManaPotion))
        .unwrap();
    let result = equip_or_use_inventory_item(&mut c, mana_index);
    assert_eq!(result.message, "Mana is already full.");
    assert!(!result.spent_turn);
    c.mana = c.max_mana() - 1;
    let mana_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::ManaPotion))
        .unwrap();
    let result = equip_or_use_inventory_item(&mut c, mana_index);
    assert_eq!(
        result.message,
        "Used a lesser mana potion and restored 1 mana."
    );
    assert!(result.spent_turn);
}
