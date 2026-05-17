use super::*;

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
    assert!(rendered.contains("3 Eviscerate r1: cost 35 Energy. Spend CP for burst damage +0%."));
    assert!(
        rendered.contains(
            "4 Smoke Step r1: cost 35 Energy, cd 4. Then WASD=1 tile, Shift+WASD=2. +20 dodge. Ready in 2."
        )
    );
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

    assert_eq!(inventory_cell_label(&grid, 0), "H");
    assert_eq!(inventory_cell_label(&grid, 1), "W");
    assert_eq!(inventory_cell_label(&grid, 2), ".");

    grid.push(mana_potion());
    assert_eq!(inventory_cell_label(&grid, 2), "M");
}

#[test]
fn inventory_cell_spans_use_rarity_outline_and_focus_label() {
    use ratatui::style::{Color, Modifier, Style};

    let mut rare_sword = rusted_sword();
    rare_sword.rarity = Rarity::Rare;
    let mut magic_axe = crude_axe();
    magic_axe.rarity = Rarity::Magic;
    let grid = ItemGrid::new(2, 2, vec![rare_sword, magic_axe]);

    let rare_selected = inventory_cell_spans(&grid, 0, true);
    assert_eq!(rare_selected[0].content.as_ref(), "[");
    assert_eq!(rare_selected[0].style, Style::default().fg(Color::Yellow));
    assert_eq!(rare_selected[1].content.as_ref(), "W");
    assert_eq!(
        rare_selected[1].style,
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    );
    assert_eq!(rare_selected[2].content.as_ref(), "]");
    assert_eq!(rare_selected[2].style, Style::default().fg(Color::Yellow));

    let magic_unselected = inventory_cell_spans(&grid, 1, false);
    assert_eq!(magic_unselected[0].style, Style::default().fg(Color::Blue));
    assert_eq!(magic_unselected[1].style, Style::default().fg(Color::White));
    assert_eq!(magic_unselected[2].style, Style::default().fg(Color::Blue));

    let empty_selected = inventory_cell_spans(&grid, 2, true);
    assert_eq!(
        empty_selected
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>(),
        vec!["[", ".", "]"]
    );
    assert!(empty_selected.iter().all(|span| {
        span.style
            == Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
    }));
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
fn inventory_render_lines_include_grid_capacity_and_selected_details() {
    let c = test_character();
    let lines = inventory_screen_text_for_test(&c, 0, "");
    let rendered = lines.join("\n");

    assert!(rendered.contains("Inventory - Bag 4 x 4 - 3 / 16"));
    assert!(rendered.contains("[H]"));
    assert!(rendered.contains("Lesser Health Potion"));
    assert!(rendered.contains("WASD/Arrows=move  Enter=equip/use  x=drop  Esc=back"));
}

#[test]
fn inventory_render_lines_include_message_and_full_commands() {
    let c = test_character();
    let lines = inventory_screen_text_for_test(&c, 0, "Dropped Lesser Health Potion.");
    let rendered = lines.join("\n");

    assert!(rendered.contains("Dropped Lesser Health Potion."));
    assert!(rendered.contains("WASD/Arrows=move  Enter=equip/use  x=drop  Esc=back"));
}

#[test]
fn stash_render_lines_include_both_grid_capacities() {
    let c = test_character();
    let lines = stash_screen_text_for_test(&c, StashSide::Inventory, 0, 0, "");
    let rendered = lines.join("\n");

    assert!(rendered.contains("Stash - Inventory 3/16 - Stash 0/64"));
    assert!(rendered.contains("Inventory *"));
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
    assert!(rendered.contains("Stash *"));
    assert!(rendered.contains("[H]"));
    assert!(rendered.contains("[M]"));
    assert!(rendered.contains("[.] [.] [.] [.] [.] [.] [.] [.]"));
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

    let stash_title_x = char_index(body_top, "Stash *");
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
    assert!(rendered.contains("WASD/Arrows=move  Enter=equip/use  x=drop  Esc=back"));
}

#[test]
fn wide_inventory_render_keeps_bag_grid_content_sized() {
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

    assert_eq!(
        details_title_x - bag_title_x,
        usize::from(item_grid_render_width(&c.inventory))
    );
    assert!(details_title_x <= 24);
}

#[test]
fn character_creation_renders_as_ratatui_screen() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut terminal = Terminal::new(TestBackend::new(80, 18)).unwrap();
    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                "Mara",
                CharacterClass::Warrior,
                DeathMode::Hardcore,
                "",
            )
        })
        .unwrap();

    let rendered = backend_text(&terminal);
    assert!(rendered.contains("Character Creation"));
    assert!(rendered.contains("Name: Mara"));
    assert!(rendered.contains("> Hardcore"));
    assert!(rendered.contains("Enter=confirm"));
}

#[test]
fn town_service_screens_render_with_ratatui() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();

    terminal
        .draw(|frame| render_merchant_screen(frame, &c, 0, ""))
        .unwrap();
    let merchant = backend_text(&terminal);
    assert!(merchant.contains("Merchant"));
    assert!(merchant.contains("> Buy Lesser Health Potion - 50 gold"));
    assert!(merchant.contains("Buy Lesser Mana Potion - 100 gold"));
    assert!(merchant.contains("Sell items"));

    terminal
        .draw(|frame| render_blacksmith_screen(frame, &c, 4, ""))
        .unwrap();
    let blacksmith = backend_text(&terminal);
    assert!(blacksmith.contains("Blacksmith"));
    assert!(blacksmith.contains("> Manage sockets"));

    terminal
        .draw(|frame| render_town_projects_screen(frame, &c, 0, ""))
        .unwrap();
    let projects = backend_text(&terminal);
    assert!(projects.contains("Town Projects"));
    assert!(projects.contains("Enter=fund project"));

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

        assert!(
            is_dagger || is_scimitar || is_light_armor || is_buckler,
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

        assert!(
            is_sword || is_axe || is_mail || is_guard_shield,
            "unexpected Warrior equipment drop: {}",
            loot.name
        );
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
fn attributes_screen_uses_cursor_selection_and_attribute_colors() {
    use ratatui::{Terminal, backend::TestBackend, style::Color};

    let mut c = test_character();
    c.unspent_attributes = 3;
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();

    terminal
        .draw(|frame| render_spend_attributes_screen(frame, &c, 0, ""))
        .unwrap();
    let attributes = backend_text(&terminal);

    assert!(attributes.contains("> 1) Strength"));
    assert!(attributes.contains("W/S or arrows=select"));
    assert!(attributes.contains("Enter=spend"));
    assert_eq!(cell_fg_at_text(&terminal, "Strength"), Color::Red);
    assert_eq!(cell_fg_at_text(&terminal, "Dexterity"), Color::Green);
    assert_eq!(cell_fg_at_text(&terminal, "Intelligence"), Color::Blue);
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
    assert!(skill_tree.contains("> Cleave"));
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
        locked_passive_tree.contains("└─🔒︎ Deep Cut upgrades at Cleave rank 2 (1/2)"),
        "{locked_passive_lines}"
    );
    assert!(locked_passive_tree.contains("Upgrade: Cleave rank 1/2"));
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
        .draw(|frame| render_skill_tree_screen(frame, &c, 3, ""))
        .unwrap();
    let rendered = backend_text(&terminal);
    assert!(rendered.contains("Venom Edge poison lasts 3 turns."));
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
    assert_eq!(c.armor(), 4); // cloth 1 + shield 1 + Iron Guard rank 1 (+2)
    assert_eq!(
        (
            c.warrior.deep_cut_rank,
            c.warrior.iron_guard_rank,
            c.warrior.second_wind_rank
        ),
        (1, 1, 1)
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
    assert!(text.contains("> Rogue"));
    assert!(text.contains("> Hardcore"));
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
    assert_eq!(CharacterClass::Warrior.name(), "Warrior");
    assert_eq!(CharacterClass::Rogue.name(), "Rogue");
}

#[test]
fn package_version_is_major_one_for_save_breaking_rogue_release() {
    assert!(SAVE_VERSION.starts_with("1."));
}

#[test]
fn warrior_state_defaults_match_existing_rank_baseline() {
    let state = WarriorState::default();

    assert_eq!(state.cleave_rank, 1);
    assert_eq!(state.shield_bash_rank, 1);
    assert_eq!(state.battle_cry_rank, 1);
    assert_eq!(state.deep_cut_rank, 1);
    assert_eq!(state.iron_guard_rank, 1);
    assert_eq!(state.second_wind_rank, 1);
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

    assert_eq!(c.warrior.iron_guard_rank, 1);
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

    assert_eq!(c.dodge_rating(), 21);
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
    assert_eq!(c.weapon_damage(), (7, 10));
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
    assert_eq!(c.hit_rating(), 10 + c.effective_dexterity() * 5 + 10);
    assert_eq!(
        c.dodge_rating(),
        (10 + c.effective_dexterity() * 3 + 2 + 8) as u32
    );
    assert_eq!(c.armor(), 1 + 1 + iron_guard_armor_bonus(&c) + 3);
    assert_eq!(c.speed(), (10 + c.effective_dexterity() * 5 + 7) as u32);
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
        "Inserted Chipped Ruby into Rusted Sword (3-5 dmg, STR F, DEX F)."
    );
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(
        c.equipped_weapon.sockets[0],
        Some(GemSocket::filled(GemKind::Ruby, GemTier::Chipped))
    );

    assert_eq!(
        replace_gem_in_equipped(&mut c, UpgradeSlot::Weapon, 0, 0),
        "Replaced Chipped Ruby with Flawed Topaz in Rusted Sword (3-5 dmg, STR F, DEX F)."
    );
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(c.inventory[0].gem_kind, Some(GemKind::Ruby));
    assert_eq!(
        c.equipped_weapon.sockets[0],
        Some(GemSocket::filled(GemKind::Topaz, GemTier::Flawed))
    );

    assert_eq!(
        remove_gem_from_equipped(&mut c, UpgradeSlot::Weapon, 0),
        "Removed Flawed Topaz from Rusted Sword (3-5 dmg, STR F, DEX F)."
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
        "Replaced Chipped Ruby with Flawed Topaz in Rusted Sword (3-5 dmg, STR F, DEX F)."
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

    c.act1_completed = true;
    assert_eq!(
        town_project_availability(&c, TownProject::HerbGarden),
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

    assert_eq!(c.warrior.deep_cut_rank, 2);
    assert_eq!(c.warrior.iron_guard_rank, 2);
    assert_eq!(c.warrior.second_wind_rank, 2);
    assert_eq!(c.armor(), 5);

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

    assert_eq!(rogue.rogue.eviscerate_rank, 2);
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
    assert!(salvage_inventory_item(&mut c, 0).contains("Only weapons"));
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
    assert_eq!(
        weapon_message,
        "Upgraded Rusted Sword (3-5 dmg, STR F, DEX F) to +1."
    );
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
        "Only weapons, armor, and shields can be salvaged."
    );

    let axe_index = c
        .inventory
        .iter()
        .position(|item| matches!(item.kind, ItemKind::Weapon))
        .unwrap();
    assert_eq!(
        salvage_inventory_item(&mut c, axe_index),
        "Salvaged Crude Axe (4-6 dmg, STR F) into 1 weapon shard(s)."
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
        "Salvaged Crude Axe (4-6 dmg, STR F) into 2 weapon shard(s)."
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
fn equipping_when_bag_is_full_reuses_selected_cell_for_old_gear() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(1, 1, vec![crude_axe()]);

    let result = equip_or_use_inventory_item(&mut c, 0);

    assert_eq!(result.message, "Equipped Crude Axe (4-6 dmg, STR F).");
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

    assert_eq!(lines[4].chars().nth(3), Some('!'));
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

fn cell_fg_at_text(
    terminal: &ratatui::Terminal<ratatui::backend::TestBackend>,
    needle: &str,
) -> ratatui::style::Color {
    let buffer = terminal.backend().buffer();
    let width = usize::from(buffer.area.width);
    let lines = backend_lines(terminal);
    let (y, x) = lines
        .iter()
        .enumerate()
        .find_map(|(y, line)| line.find(needle).map(|x| (y, x)))
        .unwrap();
    buffer.content()[y * width + x].fg
}

fn char_index(text: &str, needle: &str) -> usize {
    text.find(needle)
        .map(|byte_index| text[..byte_index].chars().count())
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
            ItemKind::Weapon | ItemKind::Armor | ItemKind::Shield
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

    enemy.dodge_rating = 10;
    assert_eq!(player_attack_hit_chance(&c, &enemy), 25.0 / 35.0);

    enemy.dodge_rating = 25;
    assert_eq!(player_attack_hit_chance(&c, &enemy), 0.5);
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
