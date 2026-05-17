use crate::*;
use ratatui::{
    prelude::*,
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};

pub(crate) const UNKNOWN_DUNGEON_COMMAND_MESSAGE: &str = "Unknown dungeon command.";
pub(crate) const UNKNOWN_DUNGEON_COMMAND_LOG_LINE: &str = "[WARN] Unknown dungeon command.";
const RANGED_ATTACK_HIT_BONUS: i32 = 5;
const MIN_HERBS_PER_COMPLETED_FLOOR: u32 = 1;
const MAX_HERBS_PER_COMPLETED_FLOOR: u32 = 3;

pub(crate) fn clear_combat_state(c: &mut Character) {
    c.warrior.cleave_cooldown = 0;
    c.warrior.shield_bash_cooldown = 0;
    c.warrior.battle_cry_cooldown = 0;
    c.warrior.battle_cry_charges = 0;
    c.warrior.second_wind_shield = 0;
    c.rogue.combo_points = 0;
    c.rogue.smoke_step_cooldown = 0;
    c.rogue.smoke_protection_turns = 0;
    c.rogue.empowered_backstab_turns = 0;
    c.rogue.smoke_step_pending = false;
    c.sorceress.frost_ring_cooldown = 0;
    c.sorceress.chain_spark_cooldown = 0;
    c.sorceress.mana_shield_active = false;
}

pub(crate) fn leave_dungeon(c: &mut Character) {
    clear_combat_state(c);
    c.active_dungeon = None;
}

pub(crate) fn living_monster_count(d: &Dungeon) -> usize {
    d.enemies.iter().filter(|enemy| enemy.hp > 0).count()
}

pub(crate) fn monsters_remaining_message(remaining: usize) -> String {
    let monster_text = if remaining == 1 {
        "monster remains"
    } else {
        "monsters remain"
    };
    format!("{remaining} {monster_text} on this floor. Defeat all monsters before leaving.")
}

pub(crate) fn can_leave_dungeon_floor(d: &mut Dungeon) -> bool {
    let remaining = living_monster_count(d);
    if remaining == 0 {
        return true;
    }

    log_event(
        &mut d.log,
        LogKind::Warn,
        monsters_remaining_message(remaining),
    );
    false
}

fn can_escape_to_town_before_clearing_floor(floor: u32) -> bool {
    act_floor(floor) == 1
}

pub(crate) fn try_leave_dungeon_for_town(c: &mut Character) -> bool {
    let (floor, escaped_before_clear) = {
        let Some(d) = c.active_dungeon.as_mut() else {
            return false;
        };
        let floor = d.floor;
        let remaining = living_monster_count(d);
        if remaining > 0 {
            if !can_escape_to_town_before_clearing_floor(floor) {
                log_event(
                    &mut d.log,
                    LogKind::Warn,
                    monsters_remaining_message(remaining),
                );
                return false;
            }
            (floor, true)
        } else {
            (floor, false)
        }
    };

    if !escaped_before_clear {
        if let Some(herbs) = grow_herbs_for_newly_completed_floor(c, floor) {
            append_pending_town_message(c, &herb_garden_reward_message(herbs));
        }
    }
    leave_dungeon(c);
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DungeonLoopExit {
    ReturnedToTown,
    HardcoreDeath,
}

pub(crate) fn dungeon_loop(
    c: &mut Character,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<DungeonLoopExit> {
    loop {
        terminal
            .draw(|frame| render_dungeon(frame, c))
            .context("failed to draw dungeon")?;
        let key = match read_ui_input() {
            Ok(UiInput::Key(key)) => key,
            Ok(UiInput::Redraw | UiInput::Tick) => continue,
            Err(err) => {
                save_character(c)?;
                return Err(err);
            }
        };
        if is_known_dungeon_command_for(c, key) {
            clear_recent_unknown_dungeon_commands(c);
        }
        let before_floor = current_dungeon_floor(c);
        let before_log_len = current_dungeon_log_len(c);
        let action_label = dungeon_action_label_for(c, key);
        let mut took_turn = false;
        if c.class == CharacterClass::Rogue && c.rogue.smoke_step_pending {
            took_turn = handle_pending_smoke_step_key(c, key);
        } else {
            match key {
                'w' | 'W' => took_turn = try_move(c, 0, -1),
                's' | 'S' => took_turn = try_move(c, 0, 1),
                'a' | 'A' => took_turn = try_move(c, -1, 0),
                'd' | 'D' => took_turn = try_move(c, 1, 0),
                '1' | '2' | '3' | '4' => took_turn = handle_class_skill_key(c, key),
                'p' | 'P' => took_turn = use_potion(c),
                'g' | 'G' => {
                    took_turn = pickup_ground_items_on_player(c);
                    if !took_turn && !ground_item_indices_at_player(c).is_empty() {
                        took_turn = ground_loot_picker(c, terminal)?;
                    }
                }
                'i' | 'I' => match inventory_screen(c, terminal)? {
                    InventoryScreenExit::NoTurn => {}
                    InventoryScreenExit::TurnSpent => took_turn = true,
                    InventoryScreenExit::ReturnedToTown => {
                        return Ok(DungeonLoopExit::ReturnedToTown);
                    }
                    InventoryScreenExit::HardcoreDeath => {
                        return Ok(DungeonLoopExit::HardcoreDeath);
                    }
                },
                '\u{1b}' => {
                    if try_leave_dungeon_for_town(c) {
                        full_heal_on_town_return(c);
                        save_character(c)?;
                        return Ok(DungeonLoopExit::ReturnedToTown);
                    }
                }
                _ => {
                    if let Some(d) = c.active_dungeon.as_mut() {
                        log_event(&mut d.log, LogKind::Warn, UNKNOWN_DUNGEON_COMMAND_MESSAGE);
                    }
                }
            }
        }
        if finish_dungeon_action(c, before_floor, before_log_len, took_turn, action_label)?
            == DeathOutcome::HardcoreDeleted
        {
            return Ok(DungeonLoopExit::HardcoreDeath);
        }
        if c.active_dungeon.is_none() {
            return Ok(DungeonLoopExit::ReturnedToTown);
        }
    }
}

pub(crate) fn render_dungeon(frame: &mut Frame, c: &Character) {
    let Some(d) = c.active_dungeon.as_ref() else {
        return;
    };
    let skill_help_lines = dungeon_skill_help_lines(c);
    let skill_panel_height = dungeon_skill_panel_height(&skill_help_lines, frame.area().width);

    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(8),
        Constraint::Length(skill_panel_height),
        Constraint::Length(4),
    ])
    .split(frame.area());

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!("{} Floor {}", act_name(d.floor), act_floor(d.floor)),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            stat_span(format!("HP {}/{}", c.hp, c.max_hp()), Color::Red),
            Span::raw("  "),
            stat_span(
                format!(
                    "{} {}/{}",
                    c.resource_label(),
                    c.current_resource(),
                    c.max_resource()
                ),
                Color::Blue,
            ),
            Span::raw("  "),
            stat_span(format!("Gold {}", c.gold), Color::Yellow),
        ]),
        xp_bar_line(c.level, c.xp, xp_required_for_next_level(c.level)),
    ])
    .block(gothic_block("Dungeon"));
    frame.render_widget(header, layout[0]);

    if layout[1].width >= 88 {
        let body = Layout::horizontal([Constraint::Length(MAP_W as u16 + 2), Constraint::Min(24)])
            .split(layout[1]);
        render_dungeon_map(frame, d, body[0]);
        render_dungeon_log(frame, d, body[1]);
    } else {
        let body = Layout::vertical([Constraint::Length(MAP_H as u16 + 2), Constraint::Min(5)])
            .split(layout[1]);
        render_dungeon_map(frame, d, body[0]);
        render_dungeon_log(frame, d, body[1]);
    }

    let help = Paragraph::new(skill_help_lines)
        .block(gothic_block("Skills"))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, layout[2]);

    let footer = Paragraph::new(vec![
        command_line("Dungeon", &dungeon_command_entries(c)),
        Line::from(vec![
            Span::styled("Legend: ", Style::default().add_modifier(Modifier::BOLD)),
            tile_span('@'),
            Span::raw("=you  "),
            tile_span('#'),
            Span::raw("=wall  "),
            tile_span('.'),
            Span::raw("=floor  "),
            tile_span('$'),
            Span::raw("=chest  "),
            tile_span('!'),
            Span::raw("=loot  "),
            tile_span('E'),
            Span::raw("=elite  "),
            tile_span('B'),
            Span::raw("=boss"),
        ]),
    ])
    .block(gothic_block("Commands"));
    frame.render_widget(footer, layout[3]);
}

fn render_dungeon_map(frame: &mut Frame, d: &Dungeon, area: Rect) {
    let map = Paragraph::new(dungeon_map_lines(d)).block(gothic_block("Map"));
    frame.render_widget(map, area);
}

fn render_dungeon_log(frame: &mut Frame, d: &Dungeon, area: Rect) {
    let log = Paragraph::new(combat_log_lines(d))
        .block(gothic_block("Combat Log"))
        .wrap(Wrap { trim: false });
    frame.render_widget(log, area);
}

fn dungeon_map_lines(d: &Dungeon) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for y in 0..MAP_H {
        let mut spans = Vec::new();
        for x in 0..MAP_W {
            let mut ch = dungeon_tile(d, x, y);
            if x == d.stairs_x && y == d.stairs_y {
                ch = '>';
            }
            if d.chests
                .iter()
                .any(|chest| chest.x == x && chest.y == y && !chest.opened)
            {
                ch = '$';
            }
            if d.ground_items.iter().any(|item| item.x == x && item.y == y) {
                ch = '!';
            }
            if d.bell_wave_tiles.contains(&(x, y)) {
                ch = '*';
            }
            if let Some(e) = d.enemies.iter().find(|e| e.x == x && e.y == y && e.hp > 0) {
                ch = e.glyph;
            }
            if x == d.player_x && y == d.player_y {
                ch = '@';
            }
            spans.push(tile_span(ch));
        }
        lines.push(Line::from(spans));
    }
    lines
}

#[cfg(test)]
pub(crate) fn dungeon_map_lines_for_test(d: &Dungeon) -> Vec<String> {
    dungeon_map_lines(d)
        .into_iter()
        .map(|line| {
            line.spans
                .into_iter()
                .map(|span| span.content.to_string())
                .collect::<String>()
        })
        .collect()
}

fn combat_log_lines(d: &Dungeon) -> Vec<Line<'static>> {
    const MAX_LOG_LINES_ON_SCREEN: usize = 12;
    let Some(latest_header) = d.log.iter().rposition(|line| is_log_header(line)) else {
        return d
            .log
            .iter()
            .rev()
            .take(MAX_LOG_LINES_ON_SCREEN)
            .rev()
            .map(|line| log_line(line, false))
            .collect();
    };

    let current = &d.log[latest_header..];
    let current_start = current.len().saturating_sub(MAX_LOG_LINES_ON_SCREEN);
    let mut lines: Vec<_> = current[current_start..]
        .iter()
        .map(|line| log_line(line, true))
        .collect();

    let printed_current = current.len().min(MAX_LOG_LINES_ON_SCREEN);
    let remaining = MAX_LOG_LINES_ON_SCREEN.saturating_sub(printed_current);
    if remaining > 1 && latest_header > 0 {
        lines.push(Line::styled(
            "--- Previous ---",
            Style::default().fg(Color::DarkGray),
        ));
        let previous_count = remaining - 1;
        let previous_start = latest_header.saturating_sub(previous_count);
        for msg in &d.log[previous_start..latest_header] {
            lines.push(log_line(msg, false));
        }
    }

    lines
}

fn log_line(line: &str, current_group: bool) -> Line<'static> {
    let text = strip_ansi_codes(line);
    if is_log_header(&text) {
        let color = if text.contains("No turn spent") {
            Color::Yellow
        } else {
            Color::Cyan
        };
        return Line::styled(
            text,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        );
    }

    let mut style = Style::default().fg(log_color(&text));
    if !current_group {
        style = Style::default().fg(Color::DarkGray);
    }
    Line::styled(format!("  {text}"), style)
}

fn log_color(line: &str) -> Color {
    if line.starts_with("[HIT]") || line.starts_with("[HEAL]") {
        Color::Green
    } else if line.starts_with("[ENEMY]") {
        Color::Red
    } else if line.starts_with("[MISS]") {
        Color::DarkGray
    } else if line.starts_with("[KILL]")
        || line.starts_with("[STATUS]")
        || line.starts_with("[BOSS]")
    {
        Color::Magenta
    } else if line.starts_with("[LOOT]") || line.starts_with("[WARN]") {
        Color::Yellow
    } else {
        Color::White
    }
}

pub(crate) fn dungeon_skill_help_lines(c: &Character) -> Vec<Line<'static>> {
    match c.class {
        CharacterClass::Warrior => warrior_dungeon_skill_help_lines(c),
        CharacterClass::Rogue => rogue_dungeon_skill_help_lines(c),
        CharacterClass::Sorceress => sorceress_dungeon_skill_help_lines(c),
    }
}

fn dungeon_skill_panel_height(lines: &[Line<'static>], width: u16) -> u16 {
    let inner_width = usize::from(width.saturating_sub(2).max(1));
    let content_height: u16 = lines
        .iter()
        .map(|line| wrapped_line_height(line, inner_width))
        .sum();
    content_height + 2
}

fn wrapped_line_height(line: &Line<'_>, width: usize) -> u16 {
    let line_width: usize = line
        .spans
        .iter()
        .map(|span| span.content.as_ref().chars().count())
        .sum();
    line_width.max(1).div_ceil(width) as u16
}

fn warrior_dungeon_skill_help_lines(c: &Character) -> Vec<Line<'static>> {
    vec![
        Line::from(format!(
            "1 Cleave r{}: cost 5 mana, cd 1. Hit {} for {}% weapon damage. Ready in {}.",
            c.warrior.cleave_rank,
            cleave_target_help(c),
            cleave_percent(c),
            c.warrior.cleave_cooldown
        )),
        Line::from(format!(
            "2 Shield Bash r{}: cost 6 mana, cd 3. Hit {} for {}% damage and stun {}. Ready in {}.",
            c.warrior.shield_bash_rank,
            shield_bash_range_help(c),
            shield_bash_percent(c),
            shield_bash_stun_help(c),
            c.warrior.shield_bash_cooldown
        )),
        Line::from(format!(
            "3 Battle Cry r{}: cost 8 mana, cd 6. Next {} attacks gain +{}% damage; Second Wind r{} heals {}%. Ready in {}, charges {}.",
            c.warrior.battle_cry_rank,
            battle_cry_charge_count(c),
            battle_cry_bonus_percent(c),
            c.warrior.second_wind_rank,
            second_wind_heal_percent_for_rank(c.warrior.second_wind_rank),
            c.warrior.battle_cry_cooldown,
            c.warrior.battle_cry_charges
        )),
        Line::from(format!(
            "Passives: Deep Cut r{} {}% bleed for {}/turn; Iron Guard r{} +{} armor.",
            c.warrior.deep_cut_rank,
            deep_cut_chance_for_rank(c.warrior.deep_cut_rank),
            deep_cut_damage_for_rank(c.warrior.deep_cut_rank),
            c.warrior.iron_guard_rank,
            iron_guard_armor_bonus(c)
        )),
    ]
}

fn rogue_dungeon_skill_help_lines(c: &Character) -> Vec<Line<'static>> {
    vec![
        Line::from(format!(
            "Rogue: Energy {}/{}  CP {}/{}",
            c.rogue.energy, ROGUE_MAX_ENERGY, c.rogue.combo_points, ROGUE_MAX_COMBO_POINTS
        )),
        Line::from(format!(
            "1 Backstab r{}: cost 25 Energy. Build 1 CP; {}% damage, {}% empowered.",
            c.rogue.backstab_rank,
            backstab_base_percent_for_rank(c.rogue.backstab_rank),
            empowered_backstab_percent_for_rank(c.rogue.backstab_rank)
        )),
        Line::from(format!(
            "2 Venom Edge r{}: cost 30 Energy. {}% damage; build 1 CP and poison {}/turn for {} turns.",
            c.rogue.venom_edge_rank,
            venom_edge_percent_for_rank(c.rogue.venom_edge_rank),
            poison_damage_for_rank(c.rogue.venom_edge_rank),
            rupture_poison_duration_for_rank(c.rogue.rupture_rank)
        )),
        Line::from(format!(
            "3 Eviscerate r{}: cost 35 Energy. Spend CP for burst damage +{}%.",
            c.rogue.eviscerate_rank,
            eviscerate_bonus_percent_for_rank(c.rogue.eviscerate_rank)
        )),
        Line::from(format!(
            "4 Smoke Step r{}: cost 35 Energy, cd 4. Then WASD=1 tile, Shift+WASD=2. +{} dodge. Ready in {}.",
            c.rogue.smoke_step_rank,
            smoke_protection_dodge_bonus(c),
            c.rogue.smoke_step_cooldown
        )),
    ]
}

fn sorceress_dungeon_skill_help_lines(c: &Character) -> Vec<Line<'static>> {
    let shield_state = if c.sorceress.mana_shield_active {
        "on"
    } else {
        "off"
    };
    let mut lines = vec![
        Line::from(format!(
            "Sorceress: Mana {}/{}  Mana Shield {shield_state}",
            c.mana,
            c.max_mana()
        )),
        Line::from(format!(
            "1 Firebolt r{}: cost {} mana. {}% spell damage; {}% Burning.",
            c.sorceress.firebolt_rank,
            FIREBOLT_MANA_COST,
            firebolt_percent_for_rank(c.sorceress.firebolt_rank),
            firebolt_burn_chance_for_rank(c.sorceress.firebolt_rank)
        )),
        Line::from(format!(
            "2 Frost Ring r{}: cost {} mana, cd {}. 8 tiles; {}% damage; {}% Freeze. Ready in {}.",
            c.sorceress.frost_ring_rank,
            FROST_RING_MANA_COST,
            FROST_RING_COOLDOWN,
            frost_ring_percent_for_rank(c.sorceress.frost_ring_rank),
            frost_ring_freeze_chance_for_rank(c.sorceress.frost_ring_rank),
            c.sorceress.frost_ring_cooldown
        )),
        Line::from(format!(
            "3 Chain Spark r{}: cost {} mana, cd {}. {}% damage; up to {} hits. Ready in {}.",
            c.sorceress.chain_spark_rank,
            CHAIN_SPARK_MANA_COST,
            CHAIN_SPARK_COOLDOWN,
            chain_spark_percent_for_rank(c.sorceress.chain_spark_rank),
            chain_spark_hit_count_for_rank(c.sorceress.chain_spark_rank),
            c.sorceress.chain_spark_cooldown
        )),
    ];
    if c.sorceress.mana_shield_rank == 0 {
        if c.sorceress.frost_ring_rank < 2 {
            lines.push(Line::from(
                "4 Mana Shield: locked; requires Frost Ring rank 2.",
            ));
        } else {
            lines.push(Line::from(
                "4 Mana Shield: unlearned; spend a skill point to learn it.",
            ));
        }
    } else {
        lines.push(Line::from(format!(
            "4 Mana Shield r{}: free toggle. Absorbs {}% at 1 mana per damage.",
            c.sorceress.mana_shield_rank,
            mana_shield_absorb_percent_for_rank(c.sorceress.mana_shield_rank)
        )));
    }
    lines
}

fn dungeon_command_entries(c: &Character) -> Vec<(&'static str, &'static str)> {
    match c.class {
        CharacterClass::Warrior => vec![
            ("w/a/s/d", "move/attack"),
            ("1", "Cleave"),
            ("2", "Bash"),
            ("3", "Cry"),
            ("p", "potion"),
            ("g", "pickup"),
            ("i", "inventory"),
            ("Esc", "town"),
        ],
        CharacterClass::Rogue => vec![
            ("w/a/s/d", "move/attack"),
            ("1", "Backstab"),
            ("2", "Venom"),
            ("3", "Eviscerate"),
            ("4", "Smoke"),
            ("p", "potion"),
            ("g", "pickup"),
            ("i", "inventory"),
            ("Esc", "town"),
        ],
        CharacterClass::Sorceress => vec![
            ("w/a/s/d", "move/attack"),
            ("1", "Firebolt"),
            ("2", "Frost"),
            ("3", "Spark"),
            ("4", "Shield"),
            ("p", "potion"),
            ("g", "pickup"),
            ("i", "inventory"),
            ("Esc", "town"),
        ],
    }
}

fn tile_span(ch: char) -> Span<'static> {
    Span::styled(ch.to_string(), tile_style(ch))
}

fn tile_style(ch: char) -> Style {
    match ch {
        '@' => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        '#' => Style::default().fg(Color::DarkGray),
        '.' => Style::default().fg(Color::DarkGray),
        '>' => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        '$' => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        '*' => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
        'r' | 'g' => Style::default().fg(Color::Yellow),
        's' => Style::default().fg(Color::White),
        'c' | 'E' => Style::default().fg(Color::Magenta),
        'b' => Style::default().fg(Color::Blue),
        'w' => Style::default().fg(Color::Cyan),
        'm' | 'B' => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        'o' => Style::default().fg(Color::DarkGray),
        'T' => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        _ => Style::default(),
    }
}

pub(crate) fn current_dungeon_log_len(c: &Character) -> usize {
    c.active_dungeon
        .as_ref()
        .map(|d| d.log.len())
        .unwrap_or_default()
}

pub(crate) fn current_dungeon_floor(c: &Character) -> Option<u32> {
    c.active_dungeon.as_ref().map(|d| d.floor)
}

pub(crate) fn should_resolve_enemy_turns_after_action(
    c: &Character,
    before_floor: Option<u32>,
) -> bool {
    current_dungeon_floor(c).is_some_and(|after_floor| Some(after_floor) == before_floor)
}

pub(crate) fn dungeon_action_label_for(c: &Character, key: char) -> &'static str {
    match (c.class, key) {
        (_, 'w' | 'W') => "Move north / attack",
        (_, 's' | 'S') => "Move south / attack",
        (_, 'a' | 'A') => "Move west / attack",
        (_, 'd' | 'D') => "Move east / attack",
        (CharacterClass::Warrior, '1') => "Cleave",
        (CharacterClass::Warrior, '2') => "Shield Bash",
        (CharacterClass::Warrior, '3') => "Battle Cry",
        (CharacterClass::Rogue, '1') => "Backstab",
        (CharacterClass::Rogue, '2') => "Venom Edge",
        (CharacterClass::Rogue, '3') => "Eviscerate",
        (CharacterClass::Rogue, '4') => "Smoke Step",
        (CharacterClass::Sorceress, '1') => "Firebolt",
        (CharacterClass::Sorceress, '2') => "Frost Ring",
        (CharacterClass::Sorceress, '3') => "Chain Spark",
        (CharacterClass::Sorceress, '4') => "Mana Shield",
        (_, 'p' | 'P') => "Drink potion",
        (_, 'g' | 'G') => "Pick up",
        (_, 'i' | 'I') => "Inventory",
        _ => "Command",
    }
}

#[allow(dead_code)]
pub(crate) fn dungeon_action_label(key: char) -> &'static str {
    let warrior = Character::new(
        "Label".to_string(),
        CharacterClass::Warrior,
        DeathMode::Softcore,
    );
    dungeon_action_label_for(&warrior, key)
}

pub(crate) fn finish_dungeon_action(
    c: &mut Character,
    before_floor: Option<u32>,
    before_log_len: usize,
    took_turn: bool,
    action_label: &'static str,
) -> Result<DeathOutcome> {
    finish_dungeon_action_with(
        c,
        before_floor,
        before_log_len,
        took_turn,
        action_label,
        save_character,
    )
}

pub(crate) fn finish_dungeon_action_with(
    c: &mut Character,
    before_floor: Option<u32>,
    before_log_len: usize,
    took_turn: bool,
    action_label: &'static str,
    save_after_turn: impl FnOnce(&Character) -> Result<()>,
) -> Result<DeathOutcome> {
    mark_latest_log_group(c, before_log_len, took_turn, action_label);
    if !took_turn {
        return Ok(DeathOutcome::Alive);
    }

    let mut death_outcome = DeathOutcome::Alive;
    if should_resolve_enemy_turns_after_action(c, before_floor) {
        tick_player_effects(c);
        enemy_turns(c);
        death_outcome = check_death(c);
        if death_outcome == DeathOutcome::HardcoreDeleted {
            return Ok(death_outcome);
        }
    }
    save_after_turn(c)?;
    Ok(death_outcome)
}

pub(crate) fn handle_class_skill_key(c: &mut Character, key: char) -> bool {
    match (c.class, key) {
        (CharacterClass::Warrior, '1') => use_cleave(c),
        (CharacterClass::Warrior, '2') => use_shield_bash(c),
        (CharacterClass::Warrior, '3') => use_battle_cry(c),
        (CharacterClass::Rogue, '1') => use_backstab(c),
        (CharacterClass::Rogue, '2') => use_venom_edge(c),
        (CharacterClass::Rogue, '3') => use_eviscerate(c),
        (CharacterClass::Rogue, '4') => use_smoke_step(c),
        (CharacterClass::Sorceress, '1') => use_firebolt(c),
        (CharacterClass::Sorceress, '2') => use_frost_ring(c),
        (CharacterClass::Sorceress, '3') => use_chain_spark(c),
        (CharacterClass::Sorceress, '4') => toggle_mana_shield(c),
        _ => {
            log_unknown_class_skill(c);
            false
        }
    }
}

fn log_unknown_class_skill(c: &mut Character) {
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(&mut d.log, LogKind::Warn, "Unknown class skill.");
    }
}

pub(crate) fn is_known_dungeon_command_for(c: &Character, key: char) -> bool {
    if c.class == CharacterClass::Rogue
        && c.rogue.smoke_step_pending
        && smoke_step_delta_for_key(key).is_some()
    {
        return true;
    }
    matches!(
        key,
        'w' | 'W'
            | 's'
            | 'S'
            | 'a'
            | 'A'
            | 'd'
            | 'D'
            | '1'
            | '2'
            | '3'
            | 'p'
            | 'P'
            | 'g'
            | 'G'
            | 'i'
            | 'I'
            | '\u{1b}'
    ) || ((c.class == CharacterClass::Rogue || c.class == CharacterClass::Sorceress) && key == '4')
}

pub(crate) fn handle_pending_smoke_step_key(c: &mut Character, key: char) -> bool {
    if let Some((dx, dy)) = smoke_step_delta_for_key(key) {
        return try_smoke_step(c, dx, dy);
    }
    if key == '\u{1b}' || key == '4' {
        c.rogue.smoke_step_pending = false;
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Status,
            "Smoke Step canceled.",
        );
    } else {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            "Choose W/A/S/D for Smoke Step direction.",
        );
    }
    false
}

fn smoke_step_delta_for_key(key: char) -> Option<(i32, i32)> {
    match key {
        'w' => Some((0, -1)),
        's' => Some((0, 1)),
        'a' => Some((-1, 0)),
        'd' => Some((1, 0)),
        'W' => Some((0, -2)),
        'S' => Some((0, 2)),
        'A' => Some((-2, 0)),
        'D' => Some((2, 0)),
        _ => None,
    }
}

#[allow(dead_code)]
pub(crate) fn is_known_dungeon_command(key: char) -> bool {
    let warrior = Character::new(
        "Known".to_string(),
        CharacterClass::Warrior,
        DeathMode::Softcore,
    );
    is_known_dungeon_command_for(&warrior, key)
}

pub(crate) fn clear_recent_unknown_dungeon_commands(c: &mut Character) {
    let Some(d) = c.active_dungeon.as_mut() else {
        return;
    };
    while remove_latest_unknown_dungeon_command(&mut d.log) {}
}

pub(crate) fn remove_latest_unknown_dungeon_command(log: &mut Vec<String>) -> bool {
    let Some(header_index) = log.iter().rposition(|line| is_log_header(line)) else {
        if log
            .last()
            .is_some_and(|line| line == UNKNOWN_DUNGEON_COMMAND_LOG_LINE)
        {
            log.pop();
            return true;
        }
        return false;
    };

    if log.len() == header_index + 2
        && log[header_index] == "== No turn spent: Command =="
        && log[header_index + 1] == UNKNOWN_DUNGEON_COMMAND_LOG_LINE
    {
        log.truncate(header_index);
        true
    } else {
        false
    }
}

pub(crate) fn mark_latest_log_group(
    c: &mut Character,
    before_log_len: usize,
    took_turn: bool,
    action_label: &'static str,
) {
    let Some(d) = c.active_dungeon.as_mut() else {
        return;
    };
    if d.log.len() < before_log_len || (!took_turn && d.log.len() == before_log_len) {
        return;
    }
    let header = if took_turn {
        d.log_turn += 1;
        format!("== Turn {}: {} ==", d.log_turn, action_label)
    } else {
        format!("== No turn spent: {action_label} ==")
    };
    d.log.insert(before_log_len, header);
    trim_dungeon_log(&mut d.log);
}

pub(crate) fn trim_dungeon_log(log: &mut Vec<String>) {
    const MAX_LOG_LINES: usize = 120;
    if log.len() > MAX_LOG_LINES {
        let remove_count = log.len() - MAX_LOG_LINES;
        log.drain(0..remove_count);
    }
}

pub(crate) fn is_log_header(line: &str) -> bool {
    (line.starts_with("== ") && line.ends_with(" =="))
        || (line.starts_with("=== ") && line.ends_with(" ==="))
}

pub(crate) fn cleave_target_help(c: &Character) -> &'static str {
    if c.warrior.cleave_mastery == Some(SkillMastery::ReapingCleave) {
        "every adjacent enemy"
    } else {
        "up to 3 adjacent enemies"
    }
}

pub(crate) fn shield_bash_range_help(c: &Character) -> &'static str {
    if c.warrior.shield_bash_mastery == Some(SkillMastery::LongBash) {
        "1 enemy up to 2 tiles in a clear cardinal line"
    } else {
        "1 adjacent enemy"
    }
}

pub(crate) fn shield_bash_stun_turns(c: &Character) -> u32 {
    if c.warrior.shield_bash_mastery == Some(SkillMastery::DazingBash) {
        2
    } else {
        1
    }
}

pub(crate) fn shield_bash_stun_help(c: &Character) -> &'static str {
    if shield_bash_stun_turns(c) == 2 {
        "2 turns"
    } else {
        "1 turn"
    }
}

pub(crate) fn battle_cry_charge_count(c: &Character) -> u32 {
    if c.warrior.battle_cry_mastery == Some(SkillMastery::WarpathCry) {
        7
    } else {
        5
    }
}

pub(crate) fn try_move(c: &mut Character, dx: i32, dy: i32) -> bool {
    let d = c.active_dungeon.as_mut().unwrap();
    let nx = d.player_x + dx;
    let ny = d.player_y + dy;
    if dungeon_tile(d, nx, ny) == '#' {
        log_event(&mut d.log, LogKind::Warn, "You bump into a crypt wall.");
        return false;
    }
    if let Some(index) = d
        .enemies
        .iter()
        .position(|e| e.x == nx && e.y == ny && e.hp > 0)
    {
        player_attack(c, index);
        return true;
    }
    let d = c.active_dungeon.as_mut().unwrap();
    d.player_x = nx;
    d.player_y = ny;
    grant_rogue_movement_backstab(c);
    auto_interact_tile(c);
    true
}

pub(crate) fn player_attack(c: &mut Character, enemy_index: usize) {
    damage_enemy(c, enemy_index, 1.0, "hit");
    consume_battle_cry_charge(c);
}

pub(crate) fn use_cleave(c: &mut Character) -> bool {
    if c.warrior.cleave_cooldown > 0 {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            format!(
                "Cleave is on cooldown for {} more turns.",
                c.warrior.cleave_cooldown
            ),
        );
        return false;
    }
    if c.mana < 5 {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            "Not enough mana for Cleave.",
        );
        return false;
    }
    let targets = adjacent_enemy_indices(c);
    if targets.is_empty() {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            "No adjacent enemies for Cleave.",
        );
        return false;
    }
    c.mana -= 5;
    c.warrior.cleave_cooldown = 1;
    log_event(
        &mut c.active_dungeon.as_mut().unwrap().log,
        LogKind::Hit,
        "You swing a wide Cleave.",
    );
    let target_limit = if c.warrior.cleave_mastery == Some(SkillMastery::ReapingCleave) {
        usize::MAX
    } else {
        3
    };
    let blood_arc = c.warrior.cleave_mastery == Some(SkillMastery::BloodArc);
    let sundering = c.warrior.cleave_mastery == Some(SkillMastery::SunderingCleave);
    for index in targets.into_iter().take(target_limit).rev() {
        if c.active_dungeon.is_some() {
            damage_enemy(c, index, cleave_multiplier(c), "cleave");
            if let Some(d) = c.active_dungeon.as_mut() {
                if let Some(enemy) = d.enemies.get_mut(index) {
                    if blood_arc && enemy.hp > 0 {
                        enemy.bleed_turns = 3;
                        enemy.bleed_damage = enemy
                            .bleed_damage
                            .max(deep_cut_damage_for_rank(c.warrior.deep_cut_rank));
                    }
                    if sundering && enemy.hp > 0 {
                        enemy.armor_shred_turns = 3;
                    }
                }
            }
        }
    }
    consume_battle_cry_charge(c);
    true
}

pub(crate) fn use_shield_bash(c: &mut Character) -> bool {
    if c.warrior.shield_bash_cooldown > 0 {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            format!(
                "Shield Bash is on cooldown for {} more turns.",
                c.warrior.shield_bash_cooldown
            ),
        );
        return false;
    }
    if c.mana < 6 {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            "Not enough mana for Shield Bash.",
        );
        return false;
    }
    let target = if c.warrior.shield_bash_mastery == Some(SkillMastery::LongBash) {
        shield_bash_target_index(c, 2)
    } else {
        adjacent_enemy_indices(c).first().copied()
    };
    let Some(index) = target else {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            "No enemy in Shield Bash range.",
        );
        return false;
    };
    c.mana -= 6;
    c.warrior.shield_bash_cooldown = 3;
    let multiplier = if c.warrior.shield_bash_mastery == Some(SkillMastery::CrushingBash) {
        shield_bash_multiplier(c) + c.equipped_shield.armor.max(0) as f32 * 0.10
    } else {
        shield_bash_multiplier(c)
    };
    let outcome = damage_enemy(c, index, multiplier, "shield bash");
    consume_battle_cry_charge(c);
    if shield_bash_outcome_stuns(outcome) {
        apply_shield_bash_stun(c, index);
    }
    true
}

pub(crate) fn apply_shield_bash_stun(c: &mut Character, enemy_index: usize) {
    let stun_turns = shield_bash_stun_turns(c);
    let Some(d) = c.active_dungeon.as_mut() else {
        return;
    };
    let Some(enemy) = d.enemies.get_mut(enemy_index) else {
        return;
    };
    if enemy.hp <= 0 {
        return;
    }
    enemy.stunned_turns = enemy.stunned_turns.max(stun_turns);
    log_event(&mut d.log, LogKind::Status, "Shield Bash stuns the enemy.");
}

pub(crate) fn use_battle_cry(c: &mut Character) -> bool {
    if c.warrior.battle_cry_cooldown > 0 {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            format!(
                "Battle Cry is on cooldown for {} more turns.",
                c.warrior.battle_cry_cooldown
            ),
        );
        return false;
    }
    if c.mana < 8 {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            "Not enough mana for Battle Cry.",
        );
        return false;
    }
    c.mana -= 8;
    c.warrior.battle_cry_charges = battle_cry_charge_count(c);
    c.warrior.battle_cry_cooldown = 6;
    if c.warrior.battle_cry_mastery == Some(SkillMastery::RallyingCry) {
        let heal = (c.max_hp() / 5).max(1);
        let mana = (c.max_mana() / 5).max(1);
        c.hp = (c.hp + heal).min(c.max_hp());
        c.mana = (c.mana + mana).min(c.max_mana());
    }
    if c.warrior.battle_cry_mastery == Some(SkillMastery::TerrifyingCry) {
        weaken_nearby_enemies(c, 3);
    }
    log_event(
        &mut c.active_dungeon.as_mut().unwrap().log,
        LogKind::Status,
        format!(
            "You roar a Battle Cry. Your next {} attacks hit harder and enemies falter.",
            c.warrior.battle_cry_charges
        ),
    );
    true
}

pub(crate) fn consume_battle_cry_charge(c: &mut Character) {
    if c.warrior.battle_cry_charges == 0 {
        return;
    }
    c.warrior.battle_cry_charges -= 1;
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(
            &mut d.log,
            LogKind::Status,
            format!(
                "Battle Cry charge spent. {} remaining.",
                c.warrior.battle_cry_charges
            ),
        );
    }
}

pub(crate) fn shield_bash_target_index(c: &Character, range: i32) -> Option<usize> {
    let d = c.active_dungeon.as_ref().unwrap();
    d.enemies
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            e.hp > 0
                && (e.x == d.player_x || e.y == d.player_y)
                && (e.x - d.player_x).abs() + (e.y - d.player_y).abs() <= range
                && clear_cardinal_line(d, d.player_x, d.player_y, e.x, e.y)
        })
        .min_by_key(|(_, e)| (e.x - d.player_x).abs() + (e.y - d.player_y).abs())
        .map(|(i, _)| i)
}

pub(crate) fn weaken_nearby_enemies(c: &mut Character, range: i32) {
    if let Some(d) = c.active_dungeon.as_mut() {
        for enemy in &mut d.enemies {
            let dist = (enemy.x - d.player_x).abs() + (enemy.y - d.player_y).abs();
            if enemy.hp > 0 && dist <= range {
                enemy.stunned_turns = enemy.stunned_turns.max(1);
            }
        }
        log_event(
            &mut d.log,
            LogKind::Status,
            "Terrifying Cry staggers nearby enemies.",
        );
    }
}

pub(crate) fn adjacent_enemy_indices(c: &Character) -> Vec<usize> {
    let d = c.active_dungeon.as_ref().unwrap();
    d.enemies
        .iter()
        .enumerate()
        .filter(|(_, e)| e.hp > 0 && (e.x - d.player_x).abs() + (e.y - d.player_y).abs() == 1)
        .map(|(i, _)| i)
        .collect()
}

#[derive(Clone, Copy)]
pub(crate) enum EnemyDeathCause<'a> {
    PlayerAttack {
        verb: &'a str,
        damage: i32,
        critical: bool,
    },
    Bleed,
    Effect {
        source: &'a str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DamageEnemyOutcome {
    NoTarget,
    Missed,
    Hit,
    Killed,
    BossDefeated,
}

pub(crate) fn shield_bash_outcome_stuns(outcome: DamageEnemyOutcome) -> bool {
    outcome == DamageEnemyOutcome::Hit
}

pub(crate) fn damage_enemy(
    c: &mut Character,
    enemy_index: usize,
    multiplier: f32,
    verb: &str,
) -> DamageEnemyOutcome {
    let Some(mut d) = c.active_dungeon.take() else {
        return DamageEnemyOutcome::NoTarget;
    };
    let mut rng = rand::thread_rng();
    let (min, max) = c.weapon_damage();
    let battle_cry_active = c.is_warrior() && c.warrior.battle_cry_charges > 0;
    let damage_bonus = if battle_cry_active {
        battle_cry_multiplier(c)
    } else {
        1.0
    };
    if enemy_index >= d.enemies.len() || d.enemies[enemy_index].hp <= 0 {
        c.active_dungeon = Some(d);
        return DamageEnemyOutcome::NoTarget;
    }
    let hit = hit_roll_chance(player_attack_hit_chance(c, &d.enemies[enemy_index]));
    if !hit {
        let name = d.enemies[enemy_index].name.clone();
        log_event(&mut d.log, LogKind::Miss, format!("You miss {name}."));
        c.active_dungeon = Some(d);
        return DamageEnemyOutcome::Missed;
    }

    let critical = crit_roll(player_crit_chance(c));
    let mut raw = ((rng.gen_range(min..=max) as f32) * multiplier * damage_bonus).round() as i32;
    if d.enemies[enemy_index].vulnerable_turns > 0 {
        raw += 2;
    }
    let mut guard_message = None;
    let mut bleed_message = None;
    let (damage, hp_text, killed) = {
        let enemy = &mut d.enemies[enemy_index];
        let armor = effective_enemy_armor(enemy);
        let mut damage = (raw - armor).max(1);
        if critical {
            damage *= 2;
        }
        damage = apply_shock_bonus_to_damage(enemy, damage);
        enemy.hp -= damage;
        if enemy.guarding {
            guard_message = Some(format!("{} blocks with its shield.", enemy.name));
        }
        let bleed_chance = if c.is_warrior() {
            deep_cut_chance_for_rank(c.warrior.deep_cut_rank) as f64 / 100.0
        } else {
            0.0
        };
        if rng.gen_bool(bleed_chance) && enemy.hp > 0 {
            enemy.bleed_turns = 3;
            enemy.bleed_damage = deep_cut_damage_for_rank(c.warrior.deep_cut_rank);
            if c.warrior.deep_cut_mastery == Some(SkillMastery::OpenWound) {
                enemy.vulnerable_turns = 3;
            }
            bleed_message = Some(format!("{} starts bleeding.", enemy.name));
        }
        (damage, enemy_hp_text(enemy), enemy.hp <= 0)
    };

    if killed {
        let ground_items_before_death = d.ground_items.len();
        let boss_defeated = resolve_enemy_death(
            c,
            &mut d,
            enemy_index,
            EnemyDeathCause::PlayerAttack {
                verb,
                damage,
                critical,
            },
        );
        if let Some(message) = guard_message {
            log_event(&mut d.log, LogKind::Status, message);
        }
        trigger_second_wind_with_log(c, &mut d.log, battle_cry_active);
        if !boss_defeated {
            maybe_drop_loot_in_dungeon(c, &mut d, enemy_index, false);
            c.active_dungeon = Some(d);
            DamageEnemyOutcome::Killed
        } else {
            finish_boss_defeat_after_player_action(c, d, ground_items_before_death)
        }
    } else {
        let name = d.enemies[enemy_index].name.clone();
        let prefix = if critical { "Critical hit! " } else { "" };
        log_event(
            &mut d.log,
            LogKind::Hit,
            format!(
                "{prefix}You {verb} {name} for {}. {hp_text}.",
                damage_text(damage)
            ),
        );
        if let Some(message) = guard_message {
            log_event(&mut d.log, LogKind::Status, message);
        }
        if let Some(message) = bleed_message {
            log_event(&mut d.log, LogKind::Status, message);
        }
        c.active_dungeon = Some(d);
        DamageEnemyOutcome::Hit
    }
}

pub(crate) fn complete_boss_fight_in_dungeon(c: &mut Character, boss_name: &str) {
    if boss_name == "Glass Tyrant" {
        c.glass_tyrant_defeated = true;
    } else {
        c.bellkeeper_defeated = true;
    }
}

pub(crate) fn grow_herbs_for_completed_floor(c: &mut Character) -> Option<u32> {
    if !has_completed_project(c, TownProject::HerbGarden) {
        return None;
    }
    let herbs =
        rand::thread_rng().gen_range(MIN_HERBS_PER_COMPLETED_FLOOR..=MAX_HERBS_PER_COMPLETED_FLOOR);
    c.herbs += herbs;
    Some(herbs)
}

pub(crate) fn grow_herbs_for_newly_completed_floor(c: &mut Character, floor: u32) -> Option<u32> {
    if boss_floor_already_completed(c, floor) {
        None
    } else {
        grow_herbs_for_completed_floor(c)
    }
}

pub(crate) fn boss_floor_already_completed(c: &Character, floor: u32) -> bool {
    (floor == ACT1_FLOORS && c.bellkeeper_defeated)
        || (floor >= FINAL_FLOOR && c.glass_tyrant_defeated)
}

fn herb_garden_reward_message(herbs: u32) -> String {
    let herb_text = if herbs == 1 { "herb" } else { "herbs" };
    format!("Herb Garden grew {herbs} {herb_text}.")
}

pub(crate) fn resolve_enemy_death(
    c: &mut Character,
    d: &mut Dungeon,
    enemy_index: usize,
    cause: EnemyDeathCause<'_>,
) -> bool {
    if enemy_index >= d.enemies.len() || d.enemies[enemy_index].hp > 0 {
        return false;
    }
    let enemy = &d.enemies[enemy_index];
    let name = enemy.name.clone();
    let xp = enemy.xp;
    let was_boss = enemy.is_boss;
    let drop_x = enemy.x;
    let drop_y = enemy.y;
    let mut rng = rand::thread_rng();
    let gold = apply_gold_find_bonus(c, rng.gen_range(enemy.gold_min..=enemy.gold_max));
    c.gold += gold;
    let levels_gained = add_xp(c, xp);
    log_event(
        &mut d.log,
        LogKind::Kill,
        enemy_death_message(&name, xp, gold, cause),
    );
    push_level_up_logs(&mut d.log, &levels_gained);
    if c.is_warrior()
        && matches!(cause, EnemyDeathCause::Bleed)
        && c.warrior.deep_cut_mastery == Some(SkillMastery::Bloodletting)
    {
        let heal = (c.max_hp() / 10).max(1);
        c.hp = (c.hp + heal).min(c.max_hp());
        log_event(
            &mut d.log,
            LogKind::Heal,
            format!("Bloodletting restores {}.", heal_amount_text(heal)),
        );
    }
    if was_boss {
        let loot = random_equipment_loot_for_class(c.class, d.floor, true);
        let loot_name = colored_item_name(&loot);
        add_loot_to_bag_or_ground(c, d, loot, drop_x, drop_y, "Boss reward dropped");
        let boss_gem_name = if can_drop_gem_on_floor(d.floor) && rng.gen_bool(0.25) {
            let gem = random_gem();
            let gem_name = colored_item_name(&gem);
            add_loot_to_bag_or_ground(c, d, gem, drop_x, drop_y, "Boss gem dropped");
            Some(gem_name)
        } else {
            None
        };
        complete_boss_fight_in_dungeon(c, &name);
        let herb_summary = grow_herbs_for_completed_floor(c)
            .map(|herbs| format!(" {}", herb_garden_reward_message(herbs)))
            .unwrap_or_default();
        let level_summary = if levels_gained.is_empty() {
            String::new()
        } else {
            format!(
                " Level up: reached level {}.",
                levels_gained.last().copied().unwrap_or(c.level)
            )
        };
        let quest_hint = if name == "Glass Tyrant" {
            "Return to Warden Mara (t) to complete Act II."
        } else {
            "Return to Warden Mara (t) to complete Act I."
        };
        let gem_summary = boss_gem_name
            .map(|gem_name| format!(" Gem: {gem_name}."))
            .unwrap_or_default();
        c.pending_town_message = format!(
            "Defeated {name}! +{}, +{}. Boss reward: {loot_name}.{herb_summary}{level_summary} {quest_hint}",
            xp_reward_text(xp),
            gold_reward_text(gold)
        );
        c.pending_town_message.push_str(&gem_summary);
        full_heal_on_town_return(c);
        clear_combat_state(c);
        return true;
    }
    false
}

pub(crate) fn enemy_death_message(
    name: &str,
    xp: u32,
    gold: u32,
    cause: EnemyDeathCause<'_>,
) -> String {
    match cause {
        EnemyDeathCause::PlayerAttack {
            verb,
            damage,
            critical,
        } => {
            let prefix = if critical { "Critical hit! " } else { "" };
            format!(
                "{prefix}You {verb} {name} for {} and kill it. +{}, +{}.",
                damage_text(damage),
                xp_reward_text(xp),
                gold_reward_text(gold)
            )
        }
        EnemyDeathCause::Bleed => format!(
            "{name} dies from bleeding. +{}, +{}.",
            xp_reward_text(xp),
            gold_reward_text(gold)
        ),
        EnemyDeathCause::Effect { source } => format!(
            "{name} dies to {source}. +{}, +{}.",
            xp_reward_text(xp),
            gold_reward_text(gold)
        ),
    }
}

pub(crate) fn trigger_second_wind_with_log(
    c: &mut Character,
    log: &mut Vec<String>,
    battle_cry_active: bool,
) {
    if !c.is_warrior() {
        return;
    }
    let mut heal = 0;
    if battle_cry_active {
        heal = second_wind_heal_amount(c);
    } else if c.warrior.second_wind_mastery == Some(SkillMastery::FreshKill) {
        heal = (second_wind_heal_amount(c) / 2).max(1);
    }
    if heal == 0 {
        return;
    }
    let before = c.hp;
    c.hp = (c.hp + heal).min(c.max_hp());
    let actual_heal = c.hp - before;
    if c.warrior.second_wind_mastery == Some(SkillMastery::GrimRecovery) {
        c.warrior.second_wind_shield += heal.saturating_sub(actual_heal);
    }
    if c.warrior.second_wind_mastery == Some(SkillMastery::AdrenalSurge) && battle_cry_active {
        c.warrior.battle_cry_charges += 1;
    }
    log_event(
        log,
        LogKind::Heal,
        format!("Second Wind restores {}.", heal_amount_text(actual_heal)),
    );
    if c.warrior.second_wind_shield > 0 {
        log_event(
            log,
            LogKind::Status,
            format!("Grim Recovery shield: {}.", c.warrior.second_wind_shield),
        );
    }
}

pub(crate) fn tick_player_effects(c: &mut Character) {
    c.warrior.cleave_cooldown = c.warrior.cleave_cooldown.saturating_sub(1);
    c.warrior.shield_bash_cooldown = c.warrior.shield_bash_cooldown.saturating_sub(1);
    c.warrior.battle_cry_cooldown = c.warrior.battle_cry_cooldown.saturating_sub(1);
    c.rogue.smoke_step_cooldown = c.rogue.smoke_step_cooldown.saturating_sub(1);
    c.rogue.empowered_backstab_turns = c.rogue.empowered_backstab_turns.saturating_sub(1);
    c.sorceress.frost_ring_cooldown = c.sorceress.frost_ring_cooldown.saturating_sub(1);
    c.sorceress.chain_spark_cooldown = c.sorceress.chain_spark_cooldown.saturating_sub(1);
    if c.class == CharacterClass::Rogue {
        c.restore_rogue_energy(15);
    }
}

pub(crate) fn restore_dungeon_after_enemy_turns(c: &mut Character, d: Dungeon) {
    c.rogue.smoke_protection_turns = c.rogue.smoke_protection_turns.saturating_sub(1);
    c.active_dungeon = Some(d);
}

pub(crate) fn enemy_turns(c: &mut Character) {
    let Some(mut d) = c.active_dungeon.take() else {
        return;
    };
    d.bell_wave_tiles.clear();
    let mut occupied: Vec<(i32, i32)> = d
        .enemies
        .iter()
        .filter(|e| e.hp > 0)
        .map(|e| (e.x, e.y))
        .collect();
    for i in 0..d.enemies.len() {
        d.enemies[i].guarding = false;
        if d.enemies[i].hp <= 0 {
            continue;
        }
        d.enemies[i].armor_shred_turns = d.enemies[i].armor_shred_turns.saturating_sub(1);
        d.enemies[i].vulnerable_turns = d.enemies[i].vulnerable_turns.saturating_sub(1);
        if d.enemies[i].bleed_turns > 0 {
            let bleed_damage = if c.is_warrior()
                && c.warrior.deep_cut_mastery == Some(SkillMastery::Hemorrhage)
                && d.enemies[i].hp * 2 <= d.enemies[i].max_hp
            {
                d.enemies[i].bleed_damage + 2
            } else {
                d.enemies[i].bleed_damage
            };
            d.enemies[i].hp -= bleed_damage;
            d.enemies[i].bleed_turns -= 1;
            log_event(
                &mut d.log,
                LogKind::Hit,
                format!(
                    "{} bleeds for {}. {}.",
                    d.enemies[i].name,
                    damage_text(bleed_damage),
                    enemy_hp_text(&d.enemies[i])
                ),
            );
            if d.enemies[i].hp <= 0 {
                let ground_items_before_death = d.ground_items.len();
                if resolve_enemy_death_and_roll_loot(c, &mut d, i, EnemyDeathCause::Bleed) {
                    finish_boss_defeat_after_effect_kill(c, d, ground_items_before_death);
                    return;
                }
                continue;
            }
        }
        if d.enemies[i].poison_turns > 0 {
            let poison_damage = d.enemies[i].poison_damage.max(1);
            d.enemies[i].hp -= poison_damage;
            d.enemies[i].poison_turns -= 1;
            log_event(
                &mut d.log,
                LogKind::Hit,
                format!(
                    "{} suffers poison for {}. {}.",
                    d.enemies[i].name,
                    damage_text(poison_damage),
                    enemy_hp_text(&d.enemies[i])
                ),
            );
            if d.enemies[i].hp <= 0 {
                let ground_items_before_death = d.ground_items.len();
                if resolve_enemy_death_and_roll_loot(
                    c,
                    &mut d,
                    i,
                    EnemyDeathCause::Effect { source: "Poison" },
                ) {
                    finish_boss_defeat_after_effect_kill(c, d, ground_items_before_death);
                    return;
                }
                continue;
            }
        }
        if d.enemies[i].burning_turns > 0 {
            let burning_damage = d.enemies[i].burning_damage.max(1);
            d.enemies[i].hp -= burning_damage;
            d.enemies[i].burning_turns -= 1;
            log_event(
                &mut d.log,
                LogKind::Hit,
                format!(
                    "{} burns for {}. {}.",
                    d.enemies[i].name,
                    damage_text(burning_damage),
                    enemy_hp_text(&d.enemies[i])
                ),
            );
            if d.enemies[i].hp <= 0 {
                let ground_items_before_death = d.ground_items.len();
                if resolve_enemy_death_and_roll_loot(
                    c,
                    &mut d,
                    i,
                    EnemyDeathCause::Effect { source: "Burning" },
                ) {
                    finish_boss_defeat_after_effect_kill(c, d, ground_items_before_death);
                    return;
                }
                continue;
            }
        }
        d.enemies[i].energy += d.enemies[i].speed.max(1);
        let energy_threshold = enemy_action_energy_threshold(c);
        if d.enemies[i].energy < energy_threshold {
            continue;
        }
        d.enemies[i].energy -= energy_threshold;
        if d.enemies[i].frozen_turns > 0 {
            d.enemies[i].frozen_turns -= 1;
            log_event(
                &mut d.log,
                LogKind::Status,
                format!("{} is frozen and skips its turn.", d.enemies[i].name),
            );
            continue;
        }
        if d.enemies[i].stunned_turns > 0 {
            d.enemies[i].stunned_turns -= 1;
            log_event(
                &mut d.log,
                LogKind::Status,
                format!("{} is stunned and skips its turn.", d.enemies[i].name),
            );
            continue;
        }
        if d.enemies[i].is_boss {
            if d.enemies[i].name == "Glass Tyrant" {
                glass_tyrant_specials(c, &mut d, i, &mut occupied);
            } else {
                bellkeeper_specials(c, &mut d, i, &mut occupied);
            }
            if c.hp == 0 {
                restore_dungeon_after_enemy_turns(c, d);
                return;
            }
        }
        let dist = (d.enemies[i].x - d.player_x).abs() + (d.enemies[i].y - d.player_y).abs();
        if dist == 1 {
            let enemy_killed = enemy_melee_attack(c, &mut d, i);
            if c.hp == 0 {
                restore_dungeon_after_enemy_turns(c, d);
                return;
            }
            if enemy_killed {
                let ground_items_before_death = d.ground_items.len();
                if resolve_enemy_killed_by_effect(c, &mut d, i, "Spiked Guard") {
                    finish_boss_defeat_after_effect_kill(c, d, ground_items_before_death);
                    return;
                }
                continue;
            }
        } else if should_boneguard_guard(&d, i) {
            d.enemies[i].guarding = true;
            log_event(
                &mut d.log,
                LogKind::Status,
                format!("{} raises its shield.", d.enemies[i].name),
            );
        } else if can_cultist_ranged_attack(&d, i) {
            cultist_shadow_bolt(c, &mut d, i);
            if c.hp == 0 {
                restore_dungeon_after_enemy_turns(c, d);
                return;
            }
        } else if dist < 8 {
            let old = (d.enemies[i].x, d.enemies[i].y);
            let step_x = (d.player_x - d.enemies[i].x).signum();
            let step_y = (d.player_y - d.enemies[i].y).signum();
            let (nx, ny) =
                if (d.player_x - d.enemies[i].x).abs() > (d.player_y - d.enemies[i].y).abs() {
                    (d.enemies[i].x + step_x, d.enemies[i].y)
                } else {
                    (d.enemies[i].x, d.enemies[i].y + step_y)
                };
            if dungeon_tile(&d, nx, ny) != '#'
                && (nx, ny) != (d.player_x, d.player_y)
                && !occupied.contains(&(nx, ny))
            {
                d.enemies[i].x = nx;
                d.enemies[i].y = ny;
                if let Some(pos) = occupied.iter().position(|p| *p == old) {
                    occupied[pos] = (nx, ny);
                }
            }
        }
    }
    d.enemies.retain(|e| e.hp > 0);
    restore_dungeon_after_enemy_turns(c, d);
}

pub(crate) fn enemy_action_energy_threshold(c: &Character) -> i32 {
    ((c.speed() as i32 + 1) / 2).max(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BellkeeperPhase {
    Tolling,
    CursedBell,
    Enraged,
}

pub(crate) fn bellkeeper_phase(enemy: &Enemy) -> BellkeeperPhase {
    if enemy.hp * 4 <= enemy.max_hp {
        BellkeeperPhase::Enraged
    } else if enemy.hp * 10 <= enemy.max_hp * 6 {
        BellkeeperPhase::CursedBell
    } else {
        BellkeeperPhase::Tolling
    }
}

pub(crate) fn bellkeeper_specials(
    c: &mut Character,
    d: &mut Dungeon,
    boss_index: usize,
    occupied: &mut Vec<(i32, i32)>,
) {
    d.boss_turn_counter += 1;
    let phase = bellkeeper_phase(&d.enemies[boss_index]);
    if phase != BellkeeperPhase::Enraged && d.boss_turn_counter % 3 == 0 {
        summon_bellkeeper_skeleton(d, boss_index, occupied);
    }
    if phase != BellkeeperPhase::Tolling && d.boss_turn_counter % 4 == 0 {
        bellkeeper_wave(c, d, boss_index);
    }
    if phase == BellkeeperPhase::Enraged {
        log_event(&mut d.log, LogKind::Boss, "The Bellkeeper is enraged.");
    }
}

pub(crate) fn glass_tyrant_specials(
    c: &mut Character,
    d: &mut Dungeon,
    boss_index: usize,
    occupied: &mut Vec<(i32, i32)>,
) {
    d.boss_turn_counter += 1;
    if d.boss_turn_counter % 3 == 0 {
        summon_glass_mirage(d, boss_index, occupied);
    }
    if d.boss_turn_counter % 4 == 0 || d.enemies[boss_index].hp * 3 <= d.enemies[boss_index].max_hp
    {
        glass_tyrant_prism_burst(c, d, boss_index);
    }
    if d.enemies[boss_index].hp * 4 <= d.enemies[boss_index].max_hp {
        log_event(
            &mut d.log,
            LogKind::Boss,
            "The Glass Tyrant fractures into a lethal prism storm.",
        );
    }
}

pub(crate) fn summon_glass_mirage(
    d: &mut Dungeon,
    boss_index: usize,
    occupied: &mut Vec<(i32, i32)>,
) {
    let summon_count = d
        .enemies
        .iter()
        .filter(|e| e.name == "Glass Mirage" && e.hp > 0)
        .count();
    if summon_count >= 2 {
        return;
    }
    let (boss_x, boss_y) = (d.enemies[boss_index].x, d.enemies[boss_index].y);
    for (dx, dy) in [(1, 1), (-1, 1), (1, -1), (-1, -1), (1, 0), (-1, 0)] {
        let pos = (boss_x + dx, boss_y + dy);
        if dungeon_tile(d, pos.0, pos.1) != '#'
            && pos != (d.player_x, d.player_y)
            && !occupied.contains(&pos)
        {
            let mut summon = glass_wraith(pos.0, pos.1);
            summon.name = "Glass Mirage".to_string();
            summon.max_hp = (summon.max_hp / 2).max(1);
            summon.hp = summon.max_hp;
            d.enemies.push(summon);
            occupied.push(pos);
            log_event(
                &mut d.log,
                LogKind::Boss,
                "The Glass Tyrant splits off a razor mirage.",
            );
            return;
        }
    }
}

pub(crate) fn glass_tyrant_prism_burst(c: &mut Character, d: &mut Dungeon, boss_index: usize) {
    let (boss_x, boss_y) = (d.enemies[boss_index].x, d.enemies[boss_index].y);
    d.bell_wave_tiles.clear();
    for (dx, dy) in [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (-1, 1),
        (1, -1),
        (-1, -1),
    ] {
        for step in 1..=4 {
            let pos = (boss_x + dx * step, boss_y + dy * step);
            if dungeon_tile(d, pos.0, pos.1) == '#' {
                break;
            }
            d.bell_wave_tiles.push(pos);
        }
    }
    log_event(
        &mut d.log,
        LogKind::Boss,
        "The Glass Tyrant fires a prism burst!",
    );
    if d.bell_wave_tiles.contains(&(d.player_x, d.player_y)) {
        let damage = enemy_damage_after_mitigation(10, c);
        apply_player_damage(c, damage);
        log_event(
            &mut d.log,
            LogKind::Enemy,
            format!("prism burst cuts you for {}.", damage_text(damage)),
        );
    }
}

pub(crate) fn summon_bellkeeper_skeleton(
    d: &mut Dungeon,
    boss_index: usize,
    occupied: &mut Vec<(i32, i32)>,
) {
    let summon_count = d
        .enemies
        .iter()
        .filter(|e| e.name == "Summoned Skeleton" && e.hp > 0)
        .count();
    if summon_count >= 3 {
        return;
    }
    let (boss_x, boss_y) = (d.enemies[boss_index].x, d.enemies[boss_index].y);
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let pos = (boss_x + dx, boss_y + dy);
        if dungeon_tile(d, pos.0, pos.1) != '#'
            && pos != (d.player_x, d.player_y)
            && !occupied.contains(&pos)
        {
            let mut summon = skeleton(pos.0, pos.1);
            summon.name = "Summoned Skeleton".to_string();
            d.enemies.push(summon);
            occupied.push(pos);
            log_event(
                &mut d.log,
                LogKind::Boss,
                "The Bellkeeper tolls, and a skeleton claws free.",
            );
            return;
        }
    }
}

pub(crate) fn bellkeeper_wave(c: &mut Character, d: &mut Dungeon, boss_index: usize) {
    let (boss_x, boss_y) = (d.enemies[boss_index].x, d.enemies[boss_index].y);
    d.bell_wave_tiles.clear();
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        for step in 1..=5 {
            let pos = (boss_x + dx * step, boss_y + dy * step);
            if dungeon_tile(d, pos.0, pos.1) == '#' {
                break;
            }
            d.bell_wave_tiles.push(pos);
        }
    }
    log_event(
        &mut d.log,
        LogKind::Boss,
        "The Bellkeeper rings a cursed bell wave!",
    );
    if d.bell_wave_tiles.contains(&(d.player_x, d.player_y)) {
        let damage = enemy_damage_after_mitigation(6, c);
        apply_player_damage(c, damage);
        log_event(
            &mut d.log,
            LogKind::Enemy,
            format!("bell wave hits you for {}.", damage_text(damage)),
        );
    }
}

pub(crate) fn effective_enemy_armor(enemy: &Enemy) -> i32 {
    (enemy.armor
        + if enemy.guarding { 2 } else { 0 }
        + if matches!(enemy.elite_modifier, Some(EliteModifier::Armored)) {
            2
        } else {
            0
        }
        - if enemy.armor_shred_turns > 0 { 2 } else { 0 })
    .max(0)
}

pub(crate) fn should_boneguard_guard(d: &Dungeon, enemy_index: usize) -> bool {
    let enemy = &d.enemies[enemy_index];
    if enemy.glyph != 'b' && enemy.glyph != 'o' {
        return false;
    }
    let dist = (enemy.x - d.player_x).abs() + (enemy.y - d.player_y).abs();
    (2..=4).contains(&dist)
}

pub(crate) fn enemy_melee_attack(c: &mut Character, d: &mut Dungeon, enemy_index: usize) -> bool {
    let mut rng = rand::thread_rng();
    let enemy = &d.enemies[enemy_index];
    if hit_roll_chance(enemy_attack_hit_chance(enemy, c)) {
        let raw = rng.gen_range(enemy.damage_min..=enemy.damage_max)
            + elite_damage_bonus(enemy)
            + bellkeeper_enrage_damage_bonus(enemy);
        let damage = enemy_damage_after_mitigation(raw, c);
        apply_player_damage(c, damage);
        let enemy_name = enemy.name.clone();
        log_event(
            &mut d.log,
            LogKind::Enemy,
            format!("{} hits you for {}.", enemy_name, damage_text(damage)),
        );
        if c.hp == 0 {
            return false;
        }
        if c.is_warrior() && c.warrior.iron_guard_mastery == Some(SkillMastery::SpikedGuard) {
            let thorn_damage = 2;
            d.enemies[enemy_index].hp -= thorn_damage;
            log_event(
                &mut d.log,
                LogKind::Hit,
                format!(
                    "Spiked Guard deals {} to {}.",
                    damage_text(thorn_damage),
                    enemy_name
                ),
            );
            if d.enemies[enemy_index].hp <= 0 {
                return true;
            }
        }
        apply_vampiric_heal(d, enemy_index);
    } else {
        log_event(
            &mut d.log,
            LogKind::Miss,
            format!("{} misses you.", enemy.name),
        );
    }
    false
}

pub(crate) fn resolve_enemy_death_and_roll_loot(
    c: &mut Character,
    d: &mut Dungeon,
    enemy_index: usize,
    cause: EnemyDeathCause<'_>,
) -> bool {
    let should_roll_regular_loot = d
        .enemies
        .get(enemy_index)
        .is_some_and(|enemy| enemy.hp <= 0 && !enemy.is_boss);
    if resolve_enemy_death(c, d, enemy_index, cause) {
        true
    } else {
        if should_roll_regular_loot {
            maybe_drop_loot_in_dungeon(c, d, enemy_index, false);
        }
        false
    }
}

pub(crate) fn resolve_enemy_killed_by_effect(
    c: &mut Character,
    d: &mut Dungeon,
    enemy_index: usize,
    source: &str,
) -> bool {
    resolve_enemy_death_and_roll_loot(c, d, enemy_index, EnemyDeathCause::Effect { source })
}

pub(crate) fn can_cultist_ranged_attack(d: &Dungeon, enemy_index: usize) -> bool {
    let enemy = &d.enemies[enemy_index];
    let is_ranged = matches!(enemy.glyph, 'c' | 'm' | 'w')
        || (enemy.glyph == 'E' && enemy.name.contains("Glass Wraith"));
    if !is_ranged {
        return false;
    }
    let dx = (enemy.x - d.player_x).abs();
    let dy = (enemy.y - d.player_y).abs();
    let dist = dx + dy;
    (2..=5).contains(&dist)
        && (dx == 0 || dy == 0)
        && clear_cardinal_line(d, enemy.x, enemy.y, d.player_x, d.player_y)
}

pub(crate) fn clear_cardinal_line(
    d: &Dungeon,
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
) -> bool {
    if from_x != to_x && from_y != to_y {
        return false;
    }
    let step_x = (to_x - from_x).signum();
    let step_y = (to_y - from_y).signum();
    let mut x = from_x + step_x;
    let mut y = from_y + step_y;
    while (x, y) != (to_x, to_y) {
        if dungeon_tile(d, x, y) == '#' {
            return false;
        }
        x += step_x;
        y += step_y;
    }
    true
}

pub(crate) fn cultist_shadow_bolt(c: &mut Character, d: &mut Dungeon, enemy_index: usize) {
    let mut rng = rand::thread_rng();
    let enemy = &d.enemies[enemy_index];
    if hit_roll_chance(enemy_ranged_attack_hit_chance(enemy, c)) {
        let raw =
            rng.gen_range(enemy.damage_min..=enemy.damage_max + 1) + elite_damage_bonus(enemy);
        let damage = enemy_damage_after_mitigation(raw, c);
        apply_player_damage(c, damage);
        log_event(
            &mut d.log,
            LogKind::Enemy,
            format!(
                "{}'s shadow bolt hits you for {}.",
                enemy.name,
                damage_text(damage)
            ),
        );
        if c.hp == 0 {
            return;
        }
        apply_vampiric_heal(d, enemy_index);
    } else {
        log_event(
            &mut d.log,
            LogKind::Miss,
            format!("{}'s shadow bolt misses you.", enemy.name),
        );
    }
}

pub(crate) fn bellkeeper_enrage_damage_bonus(enemy: &Enemy) -> i32 {
    if enemy.name == "Bellkeeper"
        && enemy.is_boss
        && bellkeeper_phase(enemy) == BellkeeperPhase::Enraged
    {
        2
    } else {
        0
    }
}

pub(crate) fn elite_damage_bonus(enemy: &Enemy) -> i32 {
    if matches!(enemy.elite_modifier, Some(EliteModifier::Burning)) {
        1
    } else {
        0
    }
}

pub(crate) fn apply_vampiric_heal(d: &mut Dungeon, enemy_index: usize) {
    if !matches!(
        d.enemies[enemy_index].elite_modifier,
        Some(EliteModifier::Vampiric)
    ) {
        return;
    }
    let enemy = &mut d.enemies[enemy_index];
    let before = enemy.hp;
    enemy.hp = (enemy.hp + 2).min(enemy.max_hp);
    let healed = enemy.hp - before;
    if healed > 0 {
        log_event(
            &mut d.log,
            LogKind::Heal,
            format!(
                "{} drains life and heals {}.",
                enemy.name,
                heal_amount_text(healed as u32)
            ),
        );
    }
}

pub(crate) fn apply_player_damage(c: &mut Character, damage: u32) {
    let absorbed = if c.is_warrior() {
        let absorbed = c.warrior.second_wind_shield.min(damage);
        c.warrior.second_wind_shield -= absorbed;
        absorbed
    } else {
        0
    };
    let mut remaining = damage - absorbed;
    if c.class == CharacterClass::Sorceress && c.sorceress.mana_shield_active {
        if c.sorceress.mana_shield_rank == 0 || c.mana == 0 {
            c.sorceress.mana_shield_active = false;
        } else {
            let desired_absorb = remaining.saturating_mul(mana_shield_absorb_percent_for_rank(
                c.sorceress.mana_shield_rank,
            )) / 100;
            let mana_absorbed = desired_absorb.min(c.mana);
            c.mana -= mana_absorbed;
            remaining = remaining.saturating_sub(mana_absorbed);
            if c.mana == 0 {
                c.sorceress.mana_shield_active = false;
            }
        }
    }
    c.hp = c.hp.saturating_sub(remaining);
}

pub(crate) fn enemy_damage_after_mitigation(raw: i32, c: &Character) -> u32 {
    let cry_penalty = if c.is_warrior() && c.warrior.battle_cry_charges > 0 {
        0.90
    } else {
        1.0
    };
    (((raw - c.armor()).max(1) as f32) * cry_penalty)
        .round()
        .max(1.0) as u32
}

pub(crate) fn defensive_dodge_rating(c: &Character) -> i32 {
    let smoke_dodge_bonus =
        if c.class == CharacterClass::Rogue && c.rogue.smoke_protection_turns > 0 {
            smoke_protection_dodge_bonus(c)
        } else {
            0
        };
    c.dodge_rating() as i32 + smoke_dodge_bonus
}

pub(crate) fn effective_enemy_dodge_rating(enemy: &Enemy) -> i32 {
    enemy.dodge_rating.max(0)
}

pub(crate) fn player_attack_hit_chance(c: &Character, enemy: &Enemy) -> f64 {
    hit_chance(c.hit_rating() as i32, effective_enemy_dodge_rating(enemy))
}

pub(crate) fn enemy_attack_hit_chance(enemy: &Enemy, c: &Character) -> f64 {
    hit_chance(enemy.hit_rating, defensive_dodge_rating(c))
}

pub(crate) fn enemy_ranged_attack_hit_chance(enemy: &Enemy, c: &Character) -> f64 {
    hit_chance(
        enemy.hit_rating + RANGED_ATTACK_HIT_BONUS,
        defensive_dodge_rating(c),
    )
}

pub(crate) fn hit_chance(hit: i32, dodge: i32) -> f64 {
    (hit as f64 / (hit + dodge).max(1) as f64).clamp(0.20, 0.95)
}

fn hit_roll_chance(chance: f64) -> bool {
    rand::thread_rng().gen_bool(chance)
}

pub(crate) fn crit_roll(crit_chance: u32) -> bool {
    let chance = (crit_chance.min(100) as f64) / 100.0;
    rand::thread_rng().gen_bool(chance)
}

pub(crate) fn player_crit_chance(c: &Character) -> u32 {
    let battle_cry_bonus = if c.is_warrior() && c.warrior.battle_cry_charges > 0 {
        5
    } else {
        0
    };
    c.weapon_crit_chance()
        .saturating_add(battle_cry_bonus)
        .min(100)
}

pub(crate) fn maybe_drop_loot_in_dungeon(
    c: &mut Character,
    d: &mut Dungeon,
    enemy_index: usize,
    guaranteed_magic: bool,
) {
    let mut rng = rand::thread_rng();
    let (drop_x, drop_y) = d
        .enemies
        .get(enemy_index)
        .map(|enemy| (enemy.x, enemy.y))
        .unwrap_or((d.player_x, d.player_y));
    let drop_chance = if guaranteed_magic { 1.0 } else { 0.22 };
    if rng.gen_bool(drop_chance) {
        let loot = if guaranteed_magic {
            random_equipment_loot_for_class(c.class, d.floor, true)
        } else {
            random_equipment_loot_for_class(c.class, d.floor, rng.gen_bool(0.30))
        };
        add_loot_to_bag_or_ground(c, d, loot, drop_x, drop_y, "Dropped");
    }

    if !can_drop_gem_on_floor(d.floor) {
        return;
    }
    let gem_chance = if d
        .enemies
        .get(enemy_index)
        .and_then(|enemy| enemy.elite_modifier.as_ref())
        .is_some()
    {
        0.05
    } else {
        0.02
    };
    if rng.gen_bool(gem_chance) {
        let gem = random_gem();
        add_loot_to_bag_or_ground(c, d, gem, drop_x, drop_y, "Gem dropped");
    }
}

pub(crate) fn add_ground_item(d: &mut Dungeon, x: i32, y: i32, item: Item) {
    d.ground_items.push(GroundItem { x, y, item });
}

pub(crate) fn add_loot_to_bag_or_ground(
    c: &mut Character,
    d: &mut Dungeon,
    item: Item,
    x: i32,
    y: i32,
    verb: &str,
) -> bool {
    add_loot_to_inventory_or_ground(&mut c.inventory, d, item, x, y, verb)
}

pub(crate) fn ground_item_indices_at_player(c: &Character) -> Vec<usize> {
    let Some(d) = c.active_dungeon.as_ref() else {
        return Vec::new();
    };
    ground_item_indices_on_player_tile(d)
}

fn ground_item_indices_on_player_tile(d: &Dungeon) -> Vec<usize> {
    d.ground_items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.x == d.player_x && item.y == d.player_y)
        .map(|(index, _)| index)
        .collect()
}

#[cfg(test)]
pub(crate) fn pick_up_ground_item_by_tile_index(c: &mut Character, tile_index: usize) -> String {
    pick_up_selected_ground_loot_for_picker(c, tile_index).message
}

#[cfg(test)]
pub(crate) fn discard_ground_item_by_tile_index(c: &mut Character, tile_index: usize) -> String {
    discard_selected_ground_loot_for_picker(c, tile_index).message
}

pub(crate) fn pick_up_selected_ground_loot_for_picker(
    c: &mut Character,
    selected: usize,
) -> InventoryActionResult {
    if !c.inventory.has_space() {
        return InventoryActionResult::free("Inventory full.");
    }
    let Some(d) = c.active_dungeon.as_mut() else {
        return InventoryActionResult::free("No active dungeon.");
    };
    let indices = ground_item_indices_on_player_tile(d);
    let Some(ground_index) = indices.get(selected).copied() else {
        return InventoryActionResult::free("No item selected.");
    };
    let ground_item = d.ground_items.remove(ground_index);
    let name = ground_item.item.name.clone();
    let added = c.inventory.push(ground_item.item);
    debug_assert!(added);
    InventoryActionResult::spent(format!("Picked up {name}."))
}

pub(crate) fn discard_selected_ground_loot_for_picker(
    c: &mut Character,
    selected: usize,
) -> InventoryActionResult {
    let Some(d) = c.active_dungeon.as_mut() else {
        return InventoryActionResult::free("No active dungeon.");
    };
    let indices = ground_item_indices_on_player_tile(d);
    let Some(ground_index) = indices.get(selected).copied() else {
        return InventoryActionResult::free("No item selected.");
    };
    let ground_item = d.ground_items.remove(ground_index);
    InventoryActionResult::spent(format!("Discarded {}.", ground_item.item.name))
}

pub(crate) fn pickup_ground_items_on_player(c: &mut Character) -> bool {
    let indices = ground_item_indices_at_player(c);
    if indices.is_empty() {
        if let Some(d) = c.active_dungeon.as_mut() {
            log_event(&mut d.log, LogKind::Warn, "There is no loot here.");
        }
        return false;
    }
    if indices.len() > 1 {
        if let Some(d) = c.active_dungeon.as_mut() {
            log_event(
                &mut d.log,
                LogKind::Info,
                "Multiple items here. Choose loot.",
            );
        }
        return false;
    }
    if !c.inventory.has_space() {
        if let Some(d) = c.active_dungeon.as_mut() {
            log_event(
                &mut d.log,
                LogKind::Warn,
                "Inventory full. Choose loot to inspect or discard.",
            );
        }
        return false;
    }

    let index = indices[0];
    let ground_item = {
        let d = c.active_dungeon.as_mut().expect("indices require dungeon");
        d.ground_items.remove(index)
    };
    let name = colored_item_name(&ground_item.item);
    let added = c.inventory.push(ground_item.item);
    debug_assert!(added);
    let d = c.active_dungeon.as_mut().expect("indices require dungeon");
    log_event(&mut d.log, LogKind::Loot, format!("Picked up {name}."));
    true
}

pub(crate) fn ground_loot_picker(
    c: &mut Character,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<bool> {
    let mut selected = 0usize;
    let mut message = String::new();
    loop {
        let item_count = c
            .active_dungeon
            .as_ref()
            .map(ground_item_indices_on_player_tile)
            .unwrap_or_default()
            .len();
        clamp_selection(&mut selected, item_count);
        terminal
            .draw(|frame| render_ground_loot_picker(frame, c, selected, &message))
            .context("failed to draw ground loot picker")?;
        let key = match read_ui_input_timed(CURSOR_PULSE_INTERVAL)? {
            UiInput::Key(key) => key,
            UiInput::Redraw => continue,
            UiInput::Tick => {
                toggle_cursor_pulse_frame();
                continue;
            }
        };
        message.clear();
        match key {
            '\u{1b}' => return Ok(false),
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < item_count {
                    selected += 1;
                }
            }
            '\n' => {
                let result = pick_up_selected_ground_loot_for_picker(c, selected);
                message = result.message;
                if result.spent_turn {
                    if let Some(d) = c.active_dungeon.as_mut() {
                        log_event(&mut d.log, LogKind::Loot, message.clone());
                    }
                    return Ok(true);
                }
            }
            'd' | 'D' => {
                let result = discard_selected_ground_loot_for_picker(c, selected);
                message = result.message;
                if result.spent_turn {
                    if let Some(d) = c.active_dungeon.as_mut() {
                        log_event(&mut d.log, LogKind::Info, message.clone());
                    }
                    return Ok(true);
                }
                if item_count <= 1
                    && message != "No item selected."
                    && message != "No active dungeon."
                {
                    return Ok(false);
                }
            }
            _ => message = "Unknown loot command.".to_string(),
        }
    }
}

pub(crate) fn render_ground_loot_picker(
    frame: &mut Frame,
    c: &Character,
    selected: usize,
    message: &str,
) {
    const GROUND_LOOT_COMMANDS: &str = "W/S=move  Enter=pick up  d=discard  Esc=back";

    let area = frame.area();
    let footer_height = if message.is_empty() { 3 } else { 4 };
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(footer_height),
    ])
    .split(area);
    let item_count = c
        .active_dungeon
        .as_ref()
        .map(ground_item_indices_on_player_tile)
        .unwrap_or_default()
        .len();
    let title = Paragraph::new(format!("Ground Loot - {item_count} item(s) here"))
        .block(gothic_block("Ground Loot"));
    frame.render_widget(title, layout[0]);

    let (list_area, details_area) = if layout[1].width >= 72 {
        let body =
            Layout::horizontal([Constraint::Min(32), Constraint::Length(38)]).split(layout[1]);
        (body[0], body[1])
    } else {
        let body = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[1]);
        (body[0], body[1])
    };
    render_ground_loot_list(frame, c, selected, list_area);
    let details =
        Paragraph::new(ground_item_detail_lines(c, selected)).block(gothic_block("Details"));
    frame.render_widget(details, details_area);

    render_commands_footer(frame, layout[2], footer_text(message, GROUND_LOOT_COMMANDS));
}

fn render_ground_loot_list(frame: &mut Frame, c: &Character, selected: usize, area: Rect) {
    let Some(d) = c.active_dungeon.as_ref() else {
        return;
    };
    let indices = ground_item_indices_on_player_tile(d);
    let max_rows = area.height.saturating_sub(2).max(1) as usize;
    let offset = scroll_offset(selected, indices.len(), max_rows);
    let items = indices
        .into_iter()
        .skip(offset)
        .take(max_rows)
        .map(|ground_index| {
            ListItem::new(strip_ansi_codes(&d.ground_items[ground_index].item.name))
                .style(Style::default().fg(Color::White))
        })
        .collect::<Vec<_>>();
    let list = List::new(items)
        .block(gothic_block("Items"))
        .highlight_style(selected_cursor_style())
        .highlight_symbol("> ");
    let mut state = ListState::default();
    state.select(Some(selected.saturating_sub(offset)));
    frame.render_stateful_widget(list, area, &mut state);
}

fn ground_item_detail_lines(c: &Character, selected: usize) -> Vec<Line<'static>> {
    let Some(d) = c.active_dungeon.as_ref() else {
        return vec![Line::styled(
            "No active dungeon.",
            Style::default().fg(Color::DarkGray),
        )];
    };
    let indices = ground_item_indices_on_player_tile(d);
    let Some(ground_index) = indices.get(selected).copied() else {
        return vec![Line::styled(
            "No loot selected.",
            Style::default().fg(Color::DarkGray),
        )];
    };
    let item = &d.ground_items[ground_index].item;
    let mut lines = vec![
        Line::styled(
            strip_ansi_codes(&item.name),
            Style::default()
                .fg(rarity_color(&item.rarity))
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(format!(
            "{:?} | {} | value {}",
            item.kind,
            rarity_name(&item.rarity),
            item.value
        )),
    ];
    match item.kind {
        ItemKind::Weapon => lines.push(Line::from(format!(
            "Damage {}-{} | crit {}%",
            item.damage_min, item.damage_max, item.crit_chance
        ))),
        ItemKind::Armor
        | ItemKind::Shield
        | ItemKind::Helm
        | ItemKind::Gloves
        | ItemKind::Boots
        | ItemKind::Belt
        | ItemKind::Amulet
        | ItemKind::Ring => lines.push(Line::from(format!(
            "Armor {} | dodge {} | speed {}",
            item.armor, item.dodge, item.speed
        ))),
        ItemKind::HealthPotion => lines.push(Line::from("Restores 15% HP.")),
        ItemKind::ManaPotion => lines.push(Line::from("Restores 15% mana.")),
        ItemKind::Gem => {
            if let (Some(kind), Some(tier)) = (item.gem_kind, item.gem_tier) {
                lines.push(Line::from(gem_bonus_text(gem_bonus(kind, tier))));
            }
        }
    }
    if let Some(compare) = item_comparison(c, item) {
        lines.push(Line::from(strip_ansi_codes(&compare)));
    }
    lines
}

fn add_loot_to_inventory_or_ground(
    inventory: &mut ItemGrid,
    d: &mut Dungeon,
    item: Item,
    x: i32,
    y: i32,
    verb: &str,
) -> bool {
    let name = colored_item_name(&item);
    match inventory.try_push(item) {
        Ok(_) => {
            log_event(&mut d.log, LogKind::Loot, format!("{verb}: {name}."));
            true
        }
        Err(item) => {
            add_ground_item(d, x, y, item);
            log_event(
                &mut d.log,
                LogKind::Loot,
                format!("Inventory full. {name} fell to the ground."),
            );
            false
        }
    }
}

fn boss_death_added_ground_loot(d: &Dungeon, ground_items_before_death: usize) -> bool {
    d.ground_items.len() > ground_items_before_death
}

pub(crate) fn finish_boss_defeat_after_player_action(
    c: &mut Character,
    d: Dungeon,
    ground_items_before_death: usize,
) -> DamageEnemyOutcome {
    finish_boss_defeat_after_effect_kill(c, d, ground_items_before_death);
    DamageEnemyOutcome::BossDefeated
}

pub(crate) fn finish_boss_defeat_after_effect_kill(
    c: &mut Character,
    d: Dungeon,
    ground_items_before_death: usize,
) {
    if boss_death_added_ground_loot(&d, ground_items_before_death) {
        retain_boss_overflow_dungeon(c, d);
    }
}

fn retain_boss_overflow_dungeon(c: &mut Character, mut d: Dungeon) {
    for enemy in &mut d.enemies {
        enemy.hp = 0;
    }
    c.active_dungeon = Some(d);
}

#[allow(dead_code)]
pub(crate) fn random_loot(floor: u32, better: bool) -> Item {
    random_loot_for_class(CharacterClass::Warrior, floor, better)
}

pub(crate) fn random_loot_for_class(class: CharacterClass, floor: u32, better: bool) -> Item {
    let mut rng = rand::thread_rng();
    if rng.gen_range(0..5) == 4 {
        if class == CharacterClass::Rogue {
            return health_potion();
        }
        if rng.gen_bool(0.5) {
            return health_potion();
        }
        return mana_potion();
    }
    random_equipment_loot_for_class(class, floor, better)
}

pub(crate) fn socket_count_for_roll(rarity: &Rarity, roll: f64) -> usize {
    match rarity {
        Rarity::Common => {
            if roll < 0.10 {
                1
            } else {
                0
            }
        }
        Rarity::Magic => {
            if roll < 0.05 {
                2
            } else if roll < 0.25 {
                1
            } else {
                0
            }
        }
        Rarity::Rare => {
            if roll < 0.10 {
                2
            } else if roll < 0.35 {
                1
            } else {
                0
            }
        }
    }
}

pub(crate) fn gem_tier_for_roll(roll: f64) -> GemTier {
    if roll < 0.80 {
        GemTier::Chipped
    } else if roll < 0.97 {
        GemTier::Flawed
    } else {
        GemTier::Pristine
    }
}

pub(crate) fn can_drop_gem_on_floor(floor: u32) -> bool {
    floor >= 3
}

pub(crate) fn apply_gold_find_bonus(c: &Character, gold: u32) -> u32 {
    let percent = c.socket_bonuses().gold_found_percent;
    gold + (gold.saturating_mul(percent) / 100)
}

fn add_random_sockets(mut item: Item, roll: f64) -> Item {
    item.sockets = vec![None; socket_count_for_roll(&item.rarity, roll)];
    item
}

pub(crate) fn random_gem() -> Item {
    let mut rng = rand::thread_rng();
    let kinds = [
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
    let kind = kinds[rng.gen_range(0..kinds.len())];
    let tier = gem_tier_for_roll(rng.gen_range(0.0..1.0));
    gem_item(kind, tier)
}

#[allow(dead_code)]
pub(crate) fn random_equipment_loot(floor: u32, better: bool) -> Item {
    random_warrior_equipment_loot(floor, better)
}

pub(crate) fn random_equipment_loot_for_class(
    class: CharacterClass,
    floor: u32,
    better: bool,
) -> Item {
    match class {
        CharacterClass::Warrior => random_warrior_equipment_loot(floor, better),
        CharacterClass::Rogue => random_rogue_equipment_loot(floor, better),
        CharacterClass::Sorceress => random_sorceress_equipment_loot(floor, better),
    }
}

fn equipment_rarity_and_level(floor: u32, better: bool, rng: &mut impl rand::Rng) -> (Rarity, u32) {
    let rarity = if better {
        if rng.gen_bool(0.25) {
            Rarity::Rare
        } else {
            Rarity::Magic
        }
    } else {
        Rarity::Common
    };
    let rarity_bonus = match rarity {
        Rarity::Common => 0,
        Rarity::Magic => 1,
        Rarity::Rare => 2,
    };
    (rarity, floor + rarity_bonus)
}

fn random_warrior_equipment_loot(floor: u32, better: bool) -> Item {
    let mut rng = rand::thread_rng();
    let (rarity, item_level) = equipment_rarity_and_level(floor, better, &mut rng);
    let bonus = item_level as i32 - 1;
    let item = match rng.gen_range(0..10) {
        0 => item_with_rarity(
            &loot_name(&rarity, "Iron Sword"),
            ItemKind::Weapon,
            45 + bonus as u32 * 15,
            weapon_stats(3 + bonus, 5 + bonus, 0, SWORD_CRIT_CHANCE),
            rarity,
            item_level,
            requirements(4 + item_level, 2 + item_level, 0),
        ),
        1 => item_with_rarity(
            &loot_name(&rarity, "War Axe"),
            ItemKind::Weapon,
            60 + bonus as u32 * 15,
            weapon_stats(4 + bonus, 6 + bonus, -1, AXE_CRIT_CHANCE),
            rarity,
            item_level,
            requirements(5 + item_level, 0, 0),
        ),
        2 => item_with_rarity(
            &loot_name(&rarity, "Mail Vest"),
            ItemKind::Armor,
            50 + bonus as u32 * 15,
            item_stats(0, 0, 1 + bonus, 0, -bonus.min(2)),
            rarity,
            item_level,
            requirements(4 + item_level, 0, 0),
        ),
        3 => item_with_rarity(
            &loot_name(&rarity, "Guard Shield"),
            ItemKind::Shield,
            45 + bonus as u32 * 15,
            item_stats(0, 0, 1 + bonus, 2 + bonus, 0),
            rarity,
            item_level,
            requirements(3 + item_level, 0, 0),
        ),
        4 => item_with_rarity(
            &loot_name(&rarity, "Iron Helm"),
            ItemKind::Helm,
            40 + bonus as u32 * 12,
            item_stats(0, 0, 1 + bonus, 0, -bonus.min(1)),
            rarity,
            item_level,
            requirements(3 + item_level, 0, 0),
        ),
        5 => item_with_rarity(
            &loot_name(&rarity, "Plate Gloves"),
            ItemKind::Gloves,
            35 + bonus as u32 * 12,
            item_stats(0, 0, bonus.min(2), 1 + bonus / 2, 0),
            rarity,
            item_level,
            requirements(2 + item_level, 0, 0),
        ),
        6 => item_with_rarity(
            &loot_name(&rarity, "March Boots"),
            ItemKind::Boots,
            35 + bonus as u32 * 12,
            item_stats(0, 0, bonus.min(1), 1 + bonus, 1),
            rarity,
            item_level,
            requirements(2 + item_level, 1, 0),
        ),
        7 => item_with_rarity(
            &loot_name(&rarity, "War Belt"),
            ItemKind::Belt,
            35 + bonus as u32 * 12,
            item_stats(0, 0, 1 + bonus, -bonus.min(1), 0),
            rarity,
            item_level,
            requirements(3 + item_level, 0, 0),
        ),
        8 => item_with_rarity(
            &loot_name(&rarity, "Guard Amulet"),
            ItemKind::Amulet,
            45 + bonus as u32 * 12,
            item_stats(0, 0, bonus.min(2), 1 + bonus.min(2), 0),
            rarity,
            item_level,
            requirements(1 + item_level / 2, 0, 0),
        ),
        _ => item_with_rarity(
            &loot_name(&rarity, "Iron Ring"),
            ItemKind::Ring,
            30 + bonus as u32 * 12,
            item_stats(0, 0, bonus.min(1), 1 + bonus.min(2), 0),
            rarity,
            item_level,
            requirements(1 + item_level / 2, 0, 0),
        ),
    };
    add_random_sockets(item, rng.gen_range(0.0..1.0))
}

fn random_rogue_equipment_loot(floor: u32, better: bool) -> Item {
    let mut rng = rand::thread_rng();
    let (rarity, item_level) = equipment_rarity_and_level(floor, better, &mut rng);
    let bonus = item_level as i32 - 1;
    let item = match rng.gen_range(0..10) {
        0 => item_with_rarity(
            &loot_name(&rarity, "Rogue Dagger"),
            ItemKind::Weapon,
            40 + bonus as u32 * 15,
            weapon_stats(2 + bonus, 4 + bonus, 1, DAGGER_CRIT_CHANCE),
            rarity,
            item_level,
            requirements(0, 3 + item_level, 0),
        ),
        1 => item_with_rarity(
            &loot_name(&rarity, "Rogue Scimitar"),
            ItemKind::Weapon,
            50 + bonus as u32 * 15,
            weapon_stats(3 + bonus, 5 + bonus, 0, SCIMITAR_CRIT_CHANCE),
            rarity,
            item_level,
            requirements(1 + item_level / 2, 3 + item_level, 0),
        ),
        2 => item_with_rarity(
            &loot_name(&rarity, "Rogue Leathers"),
            ItemKind::Armor,
            45 + bonus as u32 * 15,
            item_stats(0, 0, 1 + bonus, 2 + bonus, 1),
            rarity,
            item_level,
            requirements(0, 2 + item_level, 0),
        ),
        3 => item_with_rarity(
            &loot_name(&rarity, "Rogue Buckler"),
            ItemKind::Shield,
            40 + bonus as u32 * 15,
            item_stats(0, 0, 1 + bonus.min(2), 2 + bonus, 0),
            rarity,
            item_level,
            requirements(0, 2 + item_level, 0),
        ),
        4 => item_with_rarity(
            &loot_name(&rarity, "Hooded Cowl"),
            ItemKind::Helm,
            35 + bonus as u32 * 12,
            item_stats(0, 0, bonus.min(1), 1 + bonus, 0),
            rarity,
            item_level,
            requirements(0, 2 + item_level, 0),
        ),
        5 => item_with_rarity(
            &loot_name(&rarity, "Cutpurse Gloves"),
            ItemKind::Gloves,
            35 + bonus as u32 * 12,
            item_stats(0, 0, bonus.min(1), 1 + bonus, 1),
            rarity,
            item_level,
            requirements(0, 2 + item_level, 0),
        ),
        6 => item_with_rarity(
            &loot_name(&rarity, "Soft Boots"),
            ItemKind::Boots,
            35 + bonus as u32 * 12,
            item_stats(0, 0, 0, 2 + bonus, 1),
            rarity,
            item_level,
            requirements(0, 2 + item_level, 0),
        ),
        7 => item_with_rarity(
            &loot_name(&rarity, "Utility Belt"),
            ItemKind::Belt,
            35 + bonus as u32 * 12,
            item_stats(0, 0, 1 + bonus.min(2), 1, 0),
            rarity,
            item_level,
            requirements(0, 2 + item_level, 0),
        ),
        8 => item_with_rarity(
            &loot_name(&rarity, "Shadow Amulet"),
            ItemKind::Amulet,
            45 + bonus as u32 * 12,
            item_stats(0, 0, 0, 1 + bonus, 1),
            rarity,
            item_level,
            requirements(0, 2 + item_level, 0),
        ),
        _ => item_with_rarity(
            &loot_name(&rarity, "Silent Ring"),
            ItemKind::Ring,
            30 + bonus as u32 * 12,
            item_stats(0, 0, 0, 1 + bonus.min(2), 1),
            rarity,
            item_level,
            requirements(0, 2 + item_level, 0),
        ),
    };
    add_random_sockets(item, rng.gen_range(0.0..1.0))
}

fn random_sorceress_equipment_loot(floor: u32, better: bool) -> Item {
    let mut rng = rand::thread_rng();
    let (rarity, item_level) = equipment_rarity_and_level(floor, better, &mut rng);
    let bonus = item_level as i32 - 1;
    let item = match rng.gen_range(0..10) {
        0 => item_with_rarity(
            &loot_name(&rarity, "Arc Wand"),
            ItemKind::Weapon,
            45 + bonus as u32 * 15,
            weapon_stats(2 + bonus, 5 + bonus, 0, WAND_CRIT_CHANCE),
            rarity,
            item_level,
            requirements(0, 1 + item_level / 2, 3 + item_level),
        ),
        1 => item_with_rarity(
            &loot_name(&rarity, "Ember Wand"),
            ItemKind::Weapon,
            50 + bonus as u32 * 15,
            weapon_stats(3 + bonus, 6 + bonus, -1, WAND_CRIT_CHANCE + 1),
            rarity,
            item_level,
            requirements(0, 1 + item_level / 2, 4 + item_level),
        ),
        2 => item_with_rarity(
            &loot_name(&rarity, "Silk Robe"),
            ItemKind::Armor,
            45 + bonus as u32 * 15,
            item_stats(0, 0, bonus.min(2), 1 + bonus.min(2), 1),
            rarity,
            item_level,
            requirements(0, 0, 2 + item_level),
        ),
        3 => item_with_rarity(
            &loot_name(&rarity, "Crystal Focus"),
            ItemKind::Shield,
            45 + bonus as u32 * 15,
            item_stats(0, 0, bonus.min(1), 1 + bonus, 0),
            rarity,
            item_level,
            requirements(0, 0, 2 + item_level),
        ),
        4 => item_with_rarity(
            &loot_name(&rarity, "Moon Circlet"),
            ItemKind::Helm,
            35 + bonus as u32 * 12,
            item_stats(0, 0, bonus.min(1), 1 + bonus.min(2), 0),
            rarity,
            item_level,
            requirements(0, 0, 2 + item_level),
        ),
        5 => item_with_rarity(
            &loot_name(&rarity, "Spell Gloves"),
            ItemKind::Gloves,
            35 + bonus as u32 * 12,
            item_stats(0, 0, 0, 1 + bonus, 0),
            rarity,
            item_level,
            requirements(0, 1 + item_level / 2, 2 + item_level),
        ),
        6 => item_with_rarity(
            &loot_name(&rarity, "Soft Slippers"),
            ItemKind::Boots,
            35 + bonus as u32 * 12,
            item_stats(0, 0, 0, 1 + bonus.min(2), 1),
            rarity,
            item_level,
            requirements(0, 1 + item_level / 2, 2 + item_level),
        ),
        7 => item_with_rarity(
            &loot_name(&rarity, "Silk Sash"),
            ItemKind::Belt,
            35 + bonus as u32 * 12,
            item_stats(0, 0, bonus.min(1), 1 + bonus.min(2), 0),
            rarity,
            item_level,
            requirements(0, 0, 2 + item_level),
        ),
        8 => item_with_rarity(
            &loot_name(&rarity, "Arcane Amulet"),
            ItemKind::Amulet,
            45 + bonus as u32 * 12,
            item_stats(0, 0, 0, 1 + bonus.min(2), 0),
            rarity,
            item_level,
            requirements(0, 0, 2 + item_level),
        ),
        _ => item_with_rarity(
            &loot_name(&rarity, "Rune Ring"),
            ItemKind::Ring,
            30 + bonus as u32 * 12,
            item_stats(0, 0, 0, 1 + bonus.min(2), 0),
            rarity,
            item_level,
            requirements(0, 0, 2 + item_level),
        ),
    };
    add_random_sockets(item, rng.gen_range(0.0..1.0))
}

pub(crate) fn rarity_name(rarity: &Rarity) -> &'static str {
    match rarity {
        Rarity::Common => "Common",
        Rarity::Magic => "Magic",
        Rarity::Rare => "Rare",
    }
}

pub(crate) fn loot_name(rarity: &Rarity, base: &str) -> String {
    let mut rng = rand::thread_rng();
    match rarity {
        Rarity::Common => base.to_string(),
        Rarity::Magic => {
            let prefixes = ["Glinting", "Sturdy", "Sharp", "Vigorous", "Quick"];
            format!("{} {}", prefixes[rng.gen_range(0..prefixes.len())], base)
        }
        Rarity::Rare => {
            let prefixes = ["Bloodforged", "Dread", "Kingsguard", "Stormmarked", "Grim"];
            let suffixes = ["of Might", "of the Wolf", "of Ash", "of Warding", "of Doom"];
            format!(
                "{} {} {}",
                prefixes[rng.gen_range(0..prefixes.len())],
                base,
                suffixes[rng.gen_range(0..suffixes.len())]
            )
        }
    }
}

pub(crate) fn add_xp(c: &mut Character, amount: u32) -> Vec<u32> {
    let mut levels_gained = Vec::new();
    c.xp += amount;
    loop {
        let needed = xp_required_for_next_level(c.level);
        if c.xp < needed {
            break;
        }
        c.xp -= needed;
        c.level += 1;
        levels_gained.push(c.level);
        c.unspent_attributes += 3;
        c.unspent_skills += 1;
        c.hp = c.max_hp();
        c.restore_class_resource_full();
    }
    levels_gained
}

pub(crate) fn xp_required_for_next_level(current_level: u32) -> u32 {
    40u32.saturating_mul(2u32.saturating_pow(current_level.saturating_sub(1)))
}

pub(crate) fn auto_interact_tile(c: &mut Character) {
    if !ground_item_indices_at_player(c).is_empty() {
        pickup_ground_items_on_player(c);
    }
    open_chest_on_player(c);
    let standing_on_stairs = c
        .active_dungeon
        .as_ref()
        .map(|d| d.player_x == d.stairs_x && d.player_y == d.stairs_y)
        .unwrap_or(false);
    if standing_on_stairs {
        use_stairs(c);
    }
}

pub(crate) fn open_chest_on_player(c: &mut Character) {
    let chest_index = {
        let d = c.active_dungeon.as_ref().unwrap();
        d.chests
            .iter()
            .position(|ch| !ch.opened && ch.x == d.player_x && ch.y == d.player_y)
    };

    if let Some(chest_index) = chest_index {
        let mut rng = rand::thread_rng();
        let gold = apply_gold_find_bonus(c, rng.gen_range(10..=25));
        c.gold += gold;
        let class = c.class;
        let d = c.active_dungeon.as_mut().unwrap();
        d.chests[chest_index].opened = true;
        let loot = random_loot_for_class(class, d.floor, rng.gen_bool(0.35));
        log_event(
            &mut d.log,
            LogKind::Loot,
            format!("Opened chest: found {}.", gold_reward_text(gold)),
        );
        let (chest_x, chest_y) = (d.chests[chest_index].x, d.chests[chest_index].y);
        add_loot_to_inventory_or_ground(&mut c.inventory, d, loot, chest_x, chest_y, "Chest loot");
        if can_drop_gem_on_floor(d.floor) && rng.gen_bool(0.06) {
            let gem = random_gem();
            add_loot_to_inventory_or_ground(
                &mut c.inventory,
                d,
                gem,
                chest_x,
                chest_y,
                "Chest gem",
            );
        }
    }
}

pub(crate) fn use_stairs(c: &mut Character) {
    let floor;
    {
        let d = c.active_dungeon.as_mut().unwrap();
        if d.player_x != d.stairs_x || d.player_y != d.stairs_y {
            log_event(&mut d.log, LogKind::Warn, "You are not standing on stairs.");
            return;
        }
        if !can_leave_dungeon_floor(d) {
            return;
        }
        floor = d.floor;
    }
    if floor == ACT1_FLOORS || floor >= FINAL_FLOOR {
        if boss_floor_already_completed(c, floor) {
            leave_dungeon(c);
            return;
        }
        let d = c.active_dungeon.as_mut().unwrap();
        let blocker = if floor >= FINAL_FLOOR {
            "The Glass Tyrant"
        } else {
            "The Bellkeeper"
        };
        log_event(
            &mut d.log,
            LogKind::Boss,
            format!("{blocker} blocks your escape. Defeat it!"),
        );
    } else {
        let herb_reward = grow_herbs_for_newly_completed_floor(c, floor);
        let mut next_dungeon = generate_dungeon(floor + 1);
        if let Some(herbs) = herb_reward {
            log_event(
                &mut next_dungeon.log,
                LogKind::Loot,
                herb_garden_reward_message(herbs),
            );
        }
        c.active_dungeon = Some(next_dungeon);
    }
}

pub(crate) fn use_potion(c: &mut Character) -> bool {
    if let Some(index) = c
        .inventory
        .iter()
        .position(|i| matches!(i.kind, ItemKind::HealthPotion))
    {
        if c.hp >= c.max_hp() {
            log_event(
                &mut c.active_dungeon.as_mut().unwrap().log,
                LogKind::Warn,
                "HP is already full.",
            );
            return false;
        }
        c.inventory.remove(index);
        let heal = lesser_potion_restore(c.max_hp());
        let before = c.hp;
        c.hp = (c.hp + heal).min(c.max_hp());
        let restored = c.hp - before;
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Heal,
            format!(
                "You drink a lesser health potion and restore {}.",
                heal_amount_text(restored)
            ),
        );
        true
    } else {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            "No lesser health potion available.",
        );
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeathOutcome {
    Alive,
    SoftcoreRevived,
    HardcoreDeleted,
}

pub(crate) fn check_death(c: &mut Character) -> DeathOutcome {
    check_death_with_save_path(c, Path::new(SAVE_PATH))
}

pub(crate) fn check_death_with_save_path(c: &mut Character, save_path: &Path) -> DeathOutcome {
    if c.hp > 0 {
        return DeathOutcome::Alive;
    }
    match c.death_mode {
        DeathMode::Softcore => {
            let penalty = c.gold / 10;
            c.gold = c.gold.saturating_sub(penalty);
            leave_dungeon(c);
            c.pending_town_message = format!(
                "You died and returned to town. Lost {}.",
                gold_reward_text(penalty)
            );
            full_heal_on_town_return(c);
            DeathOutcome::SoftcoreRevived
        }
        DeathMode::Hardcore => {
            let _ = fs::remove_file(save_path);
            leave_dungeon(c);
            c.pending_town_message = "You died in Hardcore mode. Save deleted.".to_string();
            DeathOutcome::HardcoreDeleted
        }
    }
}
