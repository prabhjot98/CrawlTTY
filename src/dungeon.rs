use crate::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub(crate) const UNKNOWN_DUNGEON_COMMAND_MESSAGE: &str = "Unknown dungeon command.";
pub(crate) const UNKNOWN_DUNGEON_COMMAND_LOG_LINE: &str = "[WARN] Unknown dungeon command.";

pub(crate) fn clear_combat_state(c: &mut Character) {
    c.cleave_cooldown = 0;
    c.shield_bash_cooldown = 0;
    c.battle_cry_cooldown = 0;
    c.battle_cry_charges = 0;
    c.second_wind_shield = 0;
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

pub(crate) fn try_leave_dungeon_for_town(c: &mut Character) -> bool {
    {
        let Some(d) = c.active_dungeon.as_mut() else {
            return false;
        };
        if !can_leave_dungeon_floor(d) {
            return false;
        }
    }

    leave_dungeon(c);
    true
}

pub(crate) fn dungeon_loop(
    c: &mut Character,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<()> {
    loop {
        terminal
            .draw(|frame| render_dungeon(frame, c))
            .context("failed to draw dungeon")?;
        let key = match read_key_char() {
            Ok(key) => key,
            Err(err) => {
                save_character(c)?;
                return Err(err);
            }
        };
        if is_known_dungeon_command(key) {
            clear_recent_unknown_dungeon_commands(c);
        }
        let before_floor = current_dungeon_floor(c);
        let before_log_len = current_dungeon_log_len(c);
        let action_label = dungeon_action_label(key);
        let mut took_turn = false;
        match key {
            'w' | 'W' => took_turn = try_move(c, 0, -1),
            's' | 'S' => took_turn = try_move(c, 0, 1),
            'a' | 'A' => took_turn = try_move(c, -1, 0),
            'd' | 'D' => took_turn = try_move(c, 1, 0),
            '1' => took_turn = use_cleave(c),
            '2' => took_turn = use_shield_bash(c),
            '3' => took_turn = use_battle_cry(c),
            'p' | 'P' => took_turn = use_potion(c),
            'g' | 'G' => {
                took_turn = pickup_ground_items_on_player(c);
                if !took_turn && !ground_item_indices_at_player(c).is_empty() {
                    took_turn = ground_loot_picker(c, terminal)?;
                }
            }
            'i' | 'I' => took_turn = inventory_screen(c, terminal)?,
            '\u{1b}' => {
                if try_leave_dungeon_for_town(c) {
                    full_heal_on_town_return(c);
                    save_character(c)?;
                    break;
                }
            }
            _ => {
                if let Some(d) = c.active_dungeon.as_mut() {
                    log_event(&mut d.log, LogKind::Warn, UNKNOWN_DUNGEON_COMMAND_MESSAGE);
                }
            }
        }
        mark_latest_log_group(c, before_log_len, took_turn, action_label);
        if took_turn {
            if should_resolve_enemy_turns_after_action(c, before_floor) {
                tick_player_effects(c);
                enemy_turns(c);
                check_death(c);
            }
            save_character(c)?;
        }
        if c.active_dungeon.is_none() {
            break;
        }
    }
    Ok(())
}

pub(crate) fn render_dungeon(frame: &mut Frame, c: &Character) {
    let Some(d) = c.active_dungeon.as_ref() else {
        return;
    };

    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(6),
        Constraint::Length(4),
    ])
    .split(frame.area());

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} Floor {}", act_name(d.floor), act_floor(d.floor)),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        stat_span(format!("HP {}/{}", c.hp, c.max_hp()), Color::Red),
        Span::raw("  "),
        stat_span(format!("Mana {}/{}", c.mana, c.max_mana()), Color::Blue),
        Span::raw("  "),
        stat_span(format!("Gold {}", c.gold), Color::Yellow),
        Span::raw("  "),
        stat_span(
            format!("XP {}/{}", c.xp, xp_required_for_next_level(c.level)),
            Color::Magenta,
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Dungeon"));
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

    let help = Paragraph::new(dungeon_skill_help_lines(c))
        .block(Block::default().borders(Borders::ALL).title("Skills"))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, layout[2]);

    let footer = Paragraph::new(vec![
        command_line(
            "Dungeon",
            &[
                ("w/a/s/d", "move/attack"),
                ("1", "Cleave"),
                ("2", "Bash"),
                ("3", "Cry"),
                ("p", "potion"),
                ("g", "pickup"),
                ("i", "inventory"),
                ("Esc", "town"),
            ],
        ),
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
    .block(Block::default().borders(Borders::ALL).title("Commands"));
    frame.render_widget(footer, layout[3]);
}

fn render_dungeon_map(frame: &mut Frame, d: &Dungeon, area: Rect) {
    let map = Paragraph::new(dungeon_map_lines(d))
        .block(Block::default().borders(Borders::ALL).title("Map"));
    frame.render_widget(map, area);
}

fn render_dungeon_log(frame: &mut Frame, d: &Dungeon, area: Rect) {
    let log = Paragraph::new(combat_log_lines(d))
        .block(Block::default().borders(Borders::ALL).title("Combat Log"))
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

fn dungeon_skill_help_lines(c: &Character) -> Vec<Line<'static>> {
    vec![
        Line::from(format!(
            "1 Cleave r{}: cost 5 mana, cd 1. Hit {} for {}% weapon damage. Ready in {}.",
            c.cleave_rank,
            cleave_target_help(c),
            cleave_percent(c),
            c.cleave_cooldown
        )),
        Line::from(format!(
            "2 Shield Bash r{}: cost 6 mana, cd 3. Hit {} for {}% damage and stun {}. Ready in {}.",
            c.shield_bash_rank,
            shield_bash_range_help(c),
            shield_bash_percent(c),
            shield_bash_stun_help(c),
            c.shield_bash_cooldown
        )),
        Line::from(format!(
            "3 Battle Cry r{}: cost 8 mana, cd 6. Next {} attacks gain +{}% damage; Second Wind r{} heals {}%. Ready in {}, charges {}.",
            c.battle_cry_rank,
            battle_cry_charge_count(c),
            battle_cry_bonus_percent(c),
            c.second_wind_rank,
            second_wind_heal_percent_for_rank(c.second_wind_rank),
            c.battle_cry_cooldown,
            c.battle_cry_charges
        )),
        Line::from(format!(
            "Passives: Deep Cut r{} {}% bleed for {}/turn; Iron Guard r{} +{} armor.",
            c.deep_cut_rank,
            deep_cut_chance_for_rank(c.deep_cut_rank),
            deep_cut_damage_for_rank(c.deep_cut_rank),
            c.iron_guard_rank,
            iron_guard_armor_bonus(c)
        )),
    ]
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

pub(crate) fn dungeon_action_label(key: char) -> &'static str {
    match key {
        'w' | 'W' => "Move north / attack",
        's' | 'S' => "Move south / attack",
        'a' | 'A' => "Move west / attack",
        'd' | 'D' => "Move east / attack",
        '1' => "Cleave",
        '2' => "Shield Bash",
        '3' => "Battle Cry",
        'p' | 'P' => "Drink potion",
        'g' | 'G' => "Pick up",
        'i' | 'I' => "Inventory",
        _ => "Command",
    }
}

pub(crate) fn is_known_dungeon_command(key: char) -> bool {
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
    )
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
    if c.cleave_mastery == Some(SkillMastery::ReapingCleave) {
        "every adjacent enemy"
    } else {
        "up to 3 adjacent enemies"
    }
}

pub(crate) fn shield_bash_range_help(c: &Character) -> &'static str {
    if c.shield_bash_mastery == Some(SkillMastery::LongBash) {
        "1 enemy up to 2 tiles in a clear cardinal line"
    } else {
        "1 adjacent enemy"
    }
}

pub(crate) fn shield_bash_stun_turns(c: &Character) -> u32 {
    if c.shield_bash_mastery == Some(SkillMastery::DazingBash) {
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
    if c.battle_cry_mastery == Some(SkillMastery::WarpathCry) {
        7
    } else {
        5
    }
}

pub(crate) fn print_footer(lines: &[&str]) {
    let (_, height) = terminal_size().unwrap_or((80, 24));
    let start_row = height
        .saturating_sub(lines.len() as u16)
        .saturating_add(1)
        .max(1);
    for (i, line) in lines.iter().enumerate() {
        print!("\x1B[{};1H\x1B[2K{}", start_row + i as u16, line);
    }
    let _ = io::stdout().flush();
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
    auto_interact_tile(c);
    true
}

pub(crate) fn player_attack(c: &mut Character, enemy_index: usize) {
    damage_enemy(c, enemy_index, 1.0, "hit");
    consume_battle_cry_charge(c);
}

pub(crate) fn use_cleave(c: &mut Character) -> bool {
    if c.cleave_cooldown > 0 {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            format!(
                "Cleave is on cooldown for {} more turns.",
                c.cleave_cooldown
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
    c.cleave_cooldown = 1;
    log_event(
        &mut c.active_dungeon.as_mut().unwrap().log,
        LogKind::Hit,
        "You swing a wide Cleave.",
    );
    let target_limit = if c.cleave_mastery == Some(SkillMastery::ReapingCleave) {
        usize::MAX
    } else {
        3
    };
    let blood_arc = c.cleave_mastery == Some(SkillMastery::BloodArc);
    let sundering = c.cleave_mastery == Some(SkillMastery::SunderingCleave);
    for index in targets.into_iter().take(target_limit).rev() {
        if c.active_dungeon.is_some() {
            damage_enemy(c, index, cleave_multiplier(c), "cleave");
            if let Some(d) = c.active_dungeon.as_mut() {
                if let Some(enemy) = d.enemies.get_mut(index) {
                    if blood_arc && enemy.hp > 0 {
                        enemy.bleed_turns = 3;
                        enemy.bleed_damage = enemy
                            .bleed_damage
                            .max(deep_cut_damage_for_rank(c.deep_cut_rank));
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
    if c.shield_bash_cooldown > 0 {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            format!(
                "Shield Bash is on cooldown for {} more turns.",
                c.shield_bash_cooldown
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
    let target = if c.shield_bash_mastery == Some(SkillMastery::LongBash) {
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
    c.shield_bash_cooldown = 3;
    let multiplier = if c.shield_bash_mastery == Some(SkillMastery::CrushingBash) {
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
    if c.battle_cry_cooldown > 0 {
        log_event(
            &mut c.active_dungeon.as_mut().unwrap().log,
            LogKind::Warn,
            format!(
                "Battle Cry is on cooldown for {} more turns.",
                c.battle_cry_cooldown
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
    c.battle_cry_charges = battle_cry_charge_count(c);
    c.battle_cry_cooldown = 6;
    if c.battle_cry_mastery == Some(SkillMastery::RallyingCry) {
        let heal = (c.max_hp() / 5).max(1);
        let mana = (c.max_mana() / 5).max(1);
        c.hp = (c.hp + heal).min(c.max_hp());
        c.mana = (c.mana + mana).min(c.max_mana());
    }
    if c.battle_cry_mastery == Some(SkillMastery::TerrifyingCry) {
        weaken_nearby_enemies(c, 3);
    }
    log_event(
        &mut c.active_dungeon.as_mut().unwrap().log,
        LogKind::Status,
        format!(
            "You roar a Battle Cry. Your next {} attacks hit harder and enemies falter.",
            c.battle_cry_charges
        ),
    );
    true
}

pub(crate) fn consume_battle_cry_charge(c: &mut Character) {
    if c.battle_cry_charges == 0 {
        return;
    }
    c.battle_cry_charges -= 1;
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(
            &mut d.log,
            LogKind::Status,
            format!(
                "Battle Cry charge spent. {} remaining.",
                c.battle_cry_charges
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
    let damage_bonus = if c.battle_cry_charges > 0 {
        battle_cry_multiplier(c)
    } else {
        1.0
    };
    let battle_cry_active = c.battle_cry_charges > 0;
    let hit = hit_roll(c.hit_rating() as i32, 10);
    if enemy_index >= d.enemies.len() || d.enemies[enemy_index].hp <= 0 {
        c.active_dungeon = Some(d);
        return DamageEnemyOutcome::NoTarget;
    }
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
        enemy.hp -= damage;
        if enemy.guarding {
            guard_message = Some(format!("{} blocks with its shield.", enemy.name));
        }
        let bleed_chance = deep_cut_chance_for_rank(c.deep_cut_rank) as f64 / 100.0;
        if rng.gen_bool(bleed_chance) && enemy.hp > 0 {
            enemy.bleed_turns = 3;
            enemy.bleed_damage = deep_cut_damage_for_rank(c.deep_cut_rank);
            if c.deep_cut_mastery == Some(SkillMastery::OpenWound) {
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
    if matches!(cause, EnemyDeathCause::Bleed)
        && c.deep_cut_mastery == Some(SkillMastery::Bloodletting)
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
        let loot = random_equipment_loot(d.floor, true);
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
            "Defeated {name}! +{}, +{}. Boss reward: {loot_name}.{level_summary} {quest_hint}",
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
    let mut heal = 0;
    if battle_cry_active {
        heal = second_wind_heal_amount(c);
    } else if c.second_wind_mastery == Some(SkillMastery::FreshKill) {
        heal = (second_wind_heal_amount(c) / 2).max(1);
    }
    if heal == 0 {
        return;
    }
    let before = c.hp;
    c.hp = (c.hp + heal).min(c.max_hp());
    let actual_heal = c.hp - before;
    if c.second_wind_mastery == Some(SkillMastery::GrimRecovery) {
        c.second_wind_shield += heal.saturating_sub(actual_heal);
    }
    if c.second_wind_mastery == Some(SkillMastery::AdrenalSurge) && battle_cry_active {
        c.battle_cry_charges += 1;
    }
    log_event(
        log,
        LogKind::Heal,
        format!("Second Wind restores {}.", heal_amount_text(actual_heal)),
    );
    if c.second_wind_shield > 0 {
        log_event(
            log,
            LogKind::Status,
            format!("Grim Recovery shield: {}.", c.second_wind_shield),
        );
    }
}

pub(crate) fn tick_player_effects(c: &mut Character) {
    c.cleave_cooldown = c.cleave_cooldown.saturating_sub(1);
    c.shield_bash_cooldown = c.shield_bash_cooldown.saturating_sub(1);
    c.battle_cry_cooldown = c.battle_cry_cooldown.saturating_sub(1);
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
            let bleed_damage = if c.deep_cut_mastery == Some(SkillMastery::Hemorrhage)
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
                if resolve_enemy_death(c, &mut d, i, EnemyDeathCause::Bleed) {
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
                c.active_dungeon = Some(d);
                return;
            }
        }
        let dist = (d.enemies[i].x - d.player_x).abs() + (d.enemies[i].y - d.player_y).abs();
        if dist == 1 {
            let enemy_killed = enemy_melee_attack(c, &mut d, i);
            if c.hp == 0 {
                c.active_dungeon = Some(d);
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
                c.active_dungeon = Some(d);
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
    c.active_dungeon = Some(d);
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
    if hit_roll(25, c.dodge_rating() as i32) {
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
        if c.iron_guard_mastery == Some(SkillMastery::SpikedGuard) {
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

pub(crate) fn resolve_enemy_killed_by_effect(
    c: &mut Character,
    d: &mut Dungeon,
    enemy_index: usize,
    source: &str,
) -> bool {
    resolve_enemy_death(c, d, enemy_index, EnemyDeathCause::Effect { source })
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
    if hit_roll(30, c.dodge_rating() as i32) {
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
    let absorbed = c.second_wind_shield.min(damage);
    c.second_wind_shield -= absorbed;
    c.hp = c.hp.saturating_sub(damage - absorbed);
}

pub(crate) fn enemy_damage_after_mitigation(raw: i32, c: &Character) -> u32 {
    let cry_penalty = if c.battle_cry_charges > 0 { 0.90 } else { 1.0 };
    (((raw - c.armor()).max(1) as f32) * cry_penalty)
        .round()
        .max(1.0) as u32
}

pub(crate) fn hit_roll(hit: i32, dodge: i32) -> bool {
    let chance = (hit as f32 / (hit + dodge).max(1) as f32).clamp(0.20, 0.95);
    rand::thread_rng().gen_bool(chance as f64)
}

pub(crate) fn crit_roll(crit_chance: u32) -> bool {
    let chance = (crit_chance.min(100) as f64) / 100.0;
    rand::thread_rng().gen_bool(chance)
}

pub(crate) fn player_crit_chance(c: &Character) -> u32 {
    let battle_cry_bonus = if c.battle_cry_charges > 0 { 5 } else { 0 };
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
            random_equipment_loot(d.floor, true)
        } else {
            random_loot(d.floor, rng.gen_bool(0.30))
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

pub(crate) fn pick_up_ground_item_by_tile_index(c: &mut Character, tile_index: usize) -> String {
    if !c.inventory.has_space() {
        return "Inventory full.".to_string();
    }
    let Some(d) = c.active_dungeon.as_mut() else {
        return "No active dungeon.".to_string();
    };
    let indices = ground_item_indices_on_player_tile(d);
    let Some(ground_index) = indices.get(tile_index).copied() else {
        return "No item selected.".to_string();
    };
    let ground_item = d.ground_items.remove(ground_index);
    let name = ground_item.item.name.clone();
    let added = c.inventory.push(ground_item.item);
    debug_assert!(added);
    format!("Picked up {name}.")
}

pub(crate) fn discard_ground_item_by_tile_index(c: &mut Character, tile_index: usize) -> String {
    let Some(d) = c.active_dungeon.as_mut() else {
        return "No active dungeon.".to_string();
    };
    let indices = ground_item_indices_on_player_tile(d);
    let Some(ground_index) = indices.get(tile_index).copied() else {
        return "No item selected.".to_string();
    };
    let ground_item = d.ground_items.remove(ground_index);
    format!("Discarded {}.", ground_item.item.name)
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
        let key = read_key_char_nav()?;
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
                message = pick_up_ground_item_by_tile_index(c, selected);
                if message.starts_with("Picked up ") {
                    if let Some(d) = c.active_dungeon.as_mut() {
                        log_event(&mut d.log, LogKind::Loot, message.clone());
                    }
                    return Ok(true);
                }
            }
            'd' | 'D' => {
                message = discard_ground_item_by_tile_index(c, selected);
                if let Some(d) = c.active_dungeon.as_mut() {
                    log_event(&mut d.log, LogKind::Info, message.clone());
                }
                if item_count <= 1 {
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
        .block(Block::default().borders(Borders::ALL).title("Ground Loot"));
    frame.render_widget(title, layout[0]);

    let body = Layout::horizontal([Constraint::Min(32), Constraint::Length(38)]).split(layout[1]);
    render_ground_loot_list(frame, c, selected, body[0]);
    let details = Paragraph::new(ground_item_detail_lines(c, selected))
        .block(Block::default().borders(Borders::ALL).title("Details"));
    frame.render_widget(details, body[1]);

    let footer_text = if message.is_empty() {
        GROUND_LOOT_COMMANDS.to_string()
    } else {
        format!("{message}\n{GROUND_LOOT_COMMANDS}")
    };
    frame.render_widget(
        Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL).title("Commands")),
        layout[2],
    );
}

fn render_ground_loot_list(frame: &mut Frame, c: &Character, selected: usize, area: Rect) {
    let Some(d) = c.active_dungeon.as_ref() else {
        return;
    };
    let lines = ground_item_indices_on_player_tile(d)
        .into_iter()
        .enumerate()
        .map(|(tile_index, ground_index)| {
            let item = &d.ground_items[ground_index].item;
            let prefix = if tile_index == selected { "> " } else { "  " };
            Line::styled(
                format!("{prefix}{}", strip_ansi_codes(&item.name)),
                if tile_index == selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            )
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Items")),
        area,
    );
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
        ItemKind::Armor | ItemKind::Shield => lines.push(Line::from(format!(
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

pub(crate) fn random_loot(floor: u32, better: bool) -> Item {
    let mut rng = rand::thread_rng();
    if rng.gen_range(0..5) == 4 {
        if rng.gen_bool(0.5) {
            return health_potion();
        }
        return mana_potion();
    }
    random_equipment_loot(floor, better)
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

pub(crate) fn random_equipment_loot(floor: u32, better: bool) -> Item {
    let mut rng = rand::thread_rng();
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
    let item_level = floor + rarity_bonus;
    let bonus = item_level as i32 - 1;
    let item = match rng.gen_range(0..4) {
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
        _ => item_with_rarity(
            &loot_name(&rarity, "Guard Shield"),
            ItemKind::Shield,
            45 + bonus as u32 * 15,
            item_stats(0, 0, 1 + bonus, 2 + bonus, 0),
            rarity,
            item_level,
            requirements(3 + item_level, 0, 0),
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
        c.mana = c.max_mana();
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
        let d = c.active_dungeon.as_mut().unwrap();
        d.chests[chest_index].opened = true;
        let loot = random_loot(d.floor, rng.gen_bool(0.35));
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
        c.active_dungeon = Some(generate_dungeon(floor + 1));
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

pub(crate) fn check_death(c: &mut Character) {
    if c.hp > 0 {
        return;
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
        }
        DeathMode::Hardcore => {
            let _ = fs::remove_file(SAVE_PATH);
            println!("You died in Hardcore mode. Save deleted.");
            std::process::exit(0);
        }
    }
}
