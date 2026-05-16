use super::*;

fn test_character() -> Character {
    Character::new("Tester".to_string(), DeathMode::Softcore)
}

fn critical_combat_test_character() -> Character {
    let mut c = test_character();
    c.strength = 0;
    c.equipped_weapon.damage_min = 10;
    c.equipped_weapon.damage_max = 10;
    c.equipped_weapon.crit_chance = 100;
    c
}

fn armored_training_dummy(x: i32, y: i32) -> Enemy {
    let mut enemy = skeleton(x, y);
    enemy.name = "Armored Dummy".to_string();
    enemy.hp = 30;
    enemy.max_hp = 30;
    enemy.armor = 3;
    enemy
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
    let saved: Character = serde_json::from_str(&fs::read_to_string(&save_path).unwrap()).unwrap();
    assert_eq!(saved.name, c.name);
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
        "inventory": [],
        "stash": [],
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
fn skill_help_helpers_reflect_masteries() {
    let mut c = test_character();

    assert_eq!(cleave_target_help(&c), "up to 3 adjacent enemies");
    assert_eq!(shield_bash_range_help(&c), "1 adjacent enemy");
    assert_eq!(shield_bash_stun_turns(&c), 1);
    assert_eq!(shield_bash_stun_help(&c), "1 turn");
    assert_eq!(battle_cry_charge_count(&c), 5);

    c.cleave_mastery = Some(SkillMastery::ReapingCleave);
    c.shield_bash_mastery = Some(SkillMastery::LongBash);
    c.battle_cry_mastery = Some(SkillMastery::WarpathCry);
    assert_eq!(cleave_target_help(&c), "every adjacent enemy");
    assert_eq!(
        shield_bash_range_help(&c),
        "1 enemy up to 2 tiles in a clear cardinal line"
    );
    assert_eq!(battle_cry_charge_count(&c), 7);

    c.shield_bash_mastery = Some(SkillMastery::DazingBash);
    assert_eq!(shield_bash_stun_turns(&c), 2);
    assert_eq!(shield_bash_stun_help(&c), "2 turns");
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

    c.inventory.push(rusted_sword());
    let sword_index = c.inventory.len() - 1;
    assert!(drop_selected_inventory_item(&mut c, sword_index).spent_turn);
    assert!(!equip_or_use_inventory_item(&mut c, usize::MAX).spent_turn);
    assert!(!drop_selected_inventory_item(&mut c, usize::MAX).spent_turn);
}

#[test]
fn legacy_screen_reset_forces_parent_ratatui_redraw() {
    use ratatui::{
        Terminal,
        backend::{Backend, TestBackend},
        widgets::Paragraph,
    };

    let mut terminal = Terminal::new(TestBackend::new(16, 3)).unwrap();
    terminal
        .draw(|frame| frame.render_widget(Paragraph::new("Town"), frame.area()))
        .unwrap();

    terminal.backend_mut().clear().unwrap();
    terminal
        .draw(|frame| frame.render_widget(Paragraph::new("Town"), frame.area()))
        .unwrap();
    assert!(!backend_text(&terminal).contains("Town"));

    clear_after_legacy_screen(&mut terminal).unwrap();
    terminal
        .draw(|frame| frame.render_widget(Paragraph::new("Town"), frame.area()))
        .unwrap();

    assert!(backend_text(&terminal).contains("Town"));
}

#[test]
fn legacy_screen_releases_ratatui_raw_mode_while_running() {
    use ratatui::{Terminal, backend::TestBackend};

    set_ratatui_owns_raw_mode(true);
    let mut terminal = Terminal::new(TestBackend::new(16, 3)).unwrap();
    let mut released_for_legacy_paint = false;

    run_legacy_screen(&mut terminal, || {
        released_for_legacy_paint = !input::ratatui_owns_raw_mode_for_test();
    })
    .unwrap();

    assert!(released_for_legacy_paint);
    assert!(input::ratatui_owns_raw_mode_for_test());
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
    c.shield_bash_mastery = Some(SkillMastery::LongBash);
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
    c.shield_bash_mastery = Some(SkillMastery::DazingBash);
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
fn bellkeeper_bleed_death_completes_boss_fight_even_with_mobs_left() {
    let mut boss = bellkeeper(5, 5);
    boss.hp = 1;
    boss.bleed_turns = 1;
    boss.bleed_damage = 2;
    let mut c = test_character();
    c.battle_cry_charges = 3;
    c.second_wind_shield = 5;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![boss, skeleton(4, 2)]));

    enemy_turns(&mut c);

    assert!(c.bellkeeper_defeated);
    assert!(c.active_dungeon.is_none());
    assert_eq!(c.battle_cry_charges, 0);
    assert_eq!(c.second_wind_shield, 0);
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

    c.battle_cry_charges = 1;
    assert_eq!(player_crit_chance(&c), 13);

    c.equipped_weapon.crit_chance = 98;
    assert_eq!(player_crit_chance(&c), 100);
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
        c.cleave_rank = 3;
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
    c.cleave_cooldown = 1;
    c.shield_bash_cooldown = 2;
    c.battle_cry_cooldown = 3;
    c.battle_cry_charges = 4;
    c.second_wind_shield = 5;
    c.pending_town_message = "old news".to_string();

    assert_eq!(enter_dungeon(&mut c), "");

    assert!(c.active_dungeon.is_some());
    assert_eq!(c.cleave_cooldown, 0);
    assert_eq!(c.shield_bash_cooldown, 0);
    assert_eq!(c.battle_cry_cooldown, 0);
    assert_eq!(c.battle_cry_charges, 0);
    assert_eq!(c.second_wind_shield, 0);
    assert!(c.pending_town_message.is_empty());
}

#[test]
fn softcore_death_clears_dungeon_and_combat_state() {
    let mut c = test_character();
    c.hp = 0;
    c.gold = 100;
    c.cleave_cooldown = 1;
    c.battle_cry_charges = 4;
    c.second_wind_shield = 5;
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    check_death(&mut c);

    assert!(c.active_dungeon.is_none());
    assert_eq!(c.hp, c.max_hp());
    assert_eq!(c.mana, c.max_mana());
    assert_eq!(c.gold, 90);
    assert_eq!(c.cleave_cooldown, 0);
    assert_eq!(c.battle_cry_charges, 0);
    assert_eq!(c.second_wind_shield, 0);
    assert!(c.pending_town_message.contains("returned to town"));
    assert!(c.pending_town_message.contains(TOWN_FULL_HEAL_MESSAGE));
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
