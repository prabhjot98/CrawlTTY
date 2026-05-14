use super::*;

fn test_character() -> Character {
    Character::new("Tester".to_string(), DeathMode::Softcore)
}

fn open_test_dungeon(player_x: i32, player_y: i32, enemies: Vec<Enemy>) -> Dungeon {
    Dungeon {
        floor: 2,
        player_x,
        player_y,
        stairs_x: MAP_W - 2,
        stairs_y: MAP_H - 2,
        enemies,
        chests: Vec::new(),
        log: Vec::new(),
        tiles: vec!['.'; (MAP_W * MAP_H) as usize],
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
fn save_character_writes_atomically() {
    let c = test_character();
    let dir = env::temp_dir().join(format!("crawltty-save-test-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let save_path = dir.join("save.json");
    let tmp_path = dir.join("save.json.tmp");

    save_character_to_path(&c, &save_path).unwrap();

    assert!(save_path.exists());
    assert!(!tmp_path.exists());
    let saved: Character = serde_json::from_str(&fs::read_to_string(&save_path).unwrap()).unwrap();
    assert_eq!(saved.name, c.name);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn new_ironbound_matches_mvp_starting_state() {
    let c = test_character();

    assert_eq!(c.class_name, "Ironbound");
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
    assert_eq!(c.armor(), 4); // cloth 1 + shield 1 + Iron Guard rank 1 (+2)
    assert_eq!(
        (c.deep_cut_rank, c.iron_guard_rank, c.second_wind_rank),
        (1, 1, 1)
    );
    assert!(!c.bellkeeper_defeated);
    assert!(!c.act1_completed);
}

#[test]
fn xp_text_shows_current_and_needed_for_next_level() {
    assert_eq!(
        xp_text(8, xp_required_for_next_level(2)),
        format!("{MAGENTA}XP 8/80{RESET}")
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
    assert_eq!(next_skill_rank(5), 5);
}

#[test]
fn battle_cry_charges_survive_movement_and_spend_on_attacks() {
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![rat(4, 2)]));

    assert!(use_battle_cry(&mut c));
    assert_eq!(c.battle_cry_charges, 5);

    assert!(try_move(&mut c, 1, 0));
    tick_player_effects(&mut c);
    assert_eq!(c.battle_cry_charges, 5);

    assert!(try_move(&mut c, 1, 0));
    assert_eq!(c.battle_cry_charges, 4);
}

#[test]
fn passive_skill_upgrades_require_branch_starter_rank_two() {
    let mut c = test_character();
    assert!(unmet_skill_prerequisite(&c, "Deep Cut").is_some());
    assert!(unmet_skill_prerequisite(&c, "Iron Guard").is_some());
    assert!(unmet_skill_prerequisite(&c, "Second Wind").is_some());

    c.unspent_skills = 6;
    c.cleave_rank = 2;
    c.shield_bash_rank = 2;
    c.battle_cry_rank = 2;
    upgrade_skill(&mut c, "Deep Cut");
    upgrade_skill(&mut c, "Iron Guard");
    upgrade_skill(&mut c, "Second Wind");

    assert_eq!(c.deep_cut_rank, 2);
    assert_eq!(c.iron_guard_rank, 2);
    assert_eq!(c.second_wind_rank, 2);
    assert_eq!(c.armor(), 5);
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
    let mut inventory = vec![health_potion(), mana_potion(), crude_axe()];
    let mut stash = Vec::new();

    let message = move_selected(&mut inventory, &mut stash, 1, "Stored");

    assert!(message.starts_with("Stored Lesser Mana Potion"));
    assert_eq!(inventory.len(), 2);
    assert_eq!(stash.len(), 1);
    assert!(matches!(stash[0].kind, ItemKind::ManaPotion));
    assert!(matches!(inventory[0].kind, ItemKind::HealthPotion));
    assert!(matches!(inventory[1].kind, ItemKind::Weapon));
}

#[test]
fn blacksmith_salvage_converts_gear_to_type_shards() {
    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(crude_axe());
    c.inventory.push(health_potion());

    let message = salvage_inventory_item(&mut c, 0);

    assert!(message.contains("weapon shard"));
    assert_eq!(c.weapon_shards, 1);
    assert_eq!(c.inventory.len(), 1);
    assert!(matches!(c.inventory[0].kind, ItemKind::HealthPotion));
    assert!(salvage_inventory_item(&mut c, 0).contains("Only weapons"));
}

#[test]
fn blacksmith_upgrades_equipped_gear_with_shards_and_gold() {
    let mut c = test_character();
    c.weapon_shards = 2;
    c.armor_shards = 2;
    c.shield_shards = 2;
    c.gold = 100;

    let weapon_message = upgrade_equipped_message(&mut c, UpgradeSlot::Weapon);
    assert!(weapon_message.contains("+1"));
    assert_eq!(c.equipped_weapon.upgrade_level, 1);
    assert_eq!(
        (c.equipped_weapon.damage_min, c.equipped_weapon.damage_max),
        (4, 6)
    );
    assert_eq!(c.weapon_shards, 0);
    assert_eq!(c.gold, 75);

    let armor_message = upgrade_equipped_message(&mut c, UpgradeSlot::Armor);
    assert!(armor_message.contains("+1"));
    assert_eq!(c.equipped_armor.upgrade_level, 1);
    assert_eq!(c.equipped_armor.armor, 2);

    let shield_message = upgrade_equipped_message(&mut c, UpgradeSlot::Shield);
    assert!(shield_message.contains("+1"));
    assert_eq!(c.equipped_shield.upgrade_level, 1);
    assert_eq!(c.equipped_shield.armor, 2);
}

#[test]
fn blacksmith_upgrade_cost_scales_with_upgrade_level() {
    let mut item = crude_axe();
    assert_eq!(upgrade_cost(&item), (2, 25));
    upgrade_item(&mut item);
    assert_eq!(upgrade_cost(&item), (4, 50));
    assert_eq!(salvage_shard_yield(&item), 2);
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
fn successful_inventory_actions_spend_dungeon_turns() {
    assert!(inventory_action_spends_turn("Equipped Crude Axe."));
    assert!(inventory_action_spends_turn(
        "Used a lesser health potion and restored 6 HP."
    ));
    assert!(inventory_action_spends_turn("Dropped Rusted Sword."));
    assert!(!inventory_action_spends_turn("No item in that slot."));
    assert!(!inventory_action_spends_turn("Unknown inventory command."));
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
    assert_eq!(late.xp, baseline.xp * 2);
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
fn stairs_advance_floors_but_final_floor_requires_boss() {
    let mut c = test_character();
    c.active_dungeon = Some(generate_dungeon(1));
    {
        let d = c.active_dungeon.as_mut().unwrap();
        d.player_x = d.stairs_x;
        d.player_y = d.stairs_y;
    }
    use_stairs(&mut c);
    assert_eq!(c.active_dungeon.as_ref().unwrap().floor, 2);

    c.active_dungeon = Some(generate_dungeon(ACT1_FLOORS));
    {
        let d = c.active_dungeon.as_mut().unwrap();
        d.player_x = d.stairs_x;
        d.player_y = d.stairs_y;
    }
    use_stairs(&mut c);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.floor, ACT1_FLOORS);
    assert!(d.log.iter().any(|line| line.contains("Bellkeeper blocks")));

    c.active_dungeon = Some(generate_dungeon(FINAL_FLOOR));
    {
        let d = c.active_dungeon.as_mut().unwrap();
        d.player_x = d.stairs_x;
        d.player_y = d.stairs_y;
    }
    use_stairs(&mut c);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.floor, FINAL_FLOOR);
    assert!(
        d.log
            .iter()
            .any(|line| line.contains("Glass Tyrant blocks"))
    );
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
    c.shield_bash_mastery = Some(SkillMastery::LongBash);
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![skeleton(4, 2)]));

    assert_eq!(shield_bash_target_index(&c, 2), Some(0));

    let d = c.active_dungeon.as_mut().unwrap();
    d.tiles[tile_index(3, 2)] = '#';

    assert_eq!(shield_bash_target_index(&c, 2), None);
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
fn bellkeeper_bleed_death_completes_boss_fight_even_with_mobs_left() {
    let mut boss = bellkeeper(5, 5);
    boss.hp = 1;
    boss.bleed_turns = 1;
    boss.bleed_damage = 2;
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![boss, skeleton(4, 2)]));

    enemy_turns(&mut c);

    assert!(c.bellkeeper_defeated);
    assert!(c.active_dungeon.is_none());
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

    assert_eq!(
        equip_or_use_inventory_item(&mut c, health_index),
        "HP is already full."
    );
    assert_eq!(c.inventory.len(), starting_items);

    c.hp = c.max_hp() - 1;
    let health_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::HealthPotion))
        .unwrap();
    assert_eq!(
        equip_or_use_inventory_item(&mut c, health_index),
        "Used a lesser health potion and restored 1 HP."
    );

    let mana_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::ManaPotion))
        .unwrap();
    assert_eq!(
        equip_or_use_inventory_item(&mut c, mana_index),
        "Mana is already full."
    );
    c.mana = c.max_mana() - 1;
    let mana_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::ManaPotion))
        .unwrap();
    assert_eq!(
        equip_or_use_inventory_item(&mut c, mana_index),
        "Used a lesser mana potion and restored 1 mana."
    );
}
