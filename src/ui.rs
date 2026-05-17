use crate::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) const TEXT_PRIMARY_COLOR: Color = Color::Rgb(214, 203, 177);
pub(crate) const TEXT_MUTED_COLOR: Color = Color::Rgb(108, 101, 112);
pub(crate) const CONTAINER_BORDER_COLOR: Color = Color::Rgb(75, 67, 84);
pub(crate) const SELECTED_CONTAINER_BORDER_COLOR: Color = Color::Rgb(148, 80, 190);
pub(crate) const TITLE_COLOR: Color = Color::Rgb(201, 163, 86);
pub(crate) const DANGER_COLOR: Color = Color::Rgb(188, 54, 54);
pub(crate) const ACTION_COLOR: Color = Color::Rgb(93, 153, 112);
pub(crate) const WARNING_COLOR: Color = Color::Rgb(214, 157, 73);
pub(crate) const ARCANE_COLOR: Color = Color::Rgb(113, 151, 201);
pub(crate) const CURSED_COLOR: Color = Color::Rgb(177, 93, 204);
pub(crate) const RARITY_COMMON_COLOR: Color = TEXT_PRIMARY_COLOR;
pub(crate) const RARITY_MAGIC_COLOR: Color = Color::Rgb(113, 151, 201);
pub(crate) const RARITY_RARE_COLOR: Color = Color::Rgb(214, 157, 73);
pub(crate) const CURSOR_PULSE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(350);

static CURSOR_PULSE_FRAME: AtomicBool = AtomicBool::new(true);

pub(crate) fn cursor_style(cursor_frame: bool) -> Style {
    let style = Style::default().fg(SELECTED_CONTAINER_BORDER_COLOR);
    if cursor_frame {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

pub(crate) fn selected_cursor_style() -> Style {
    cursor_style(cursor_pulse_frame())
}

pub(crate) fn cursor_pulse_frame() -> bool {
    CURSOR_PULSE_FRAME.load(Ordering::Relaxed)
}

pub(crate) fn toggle_cursor_pulse_frame() {
    CURSOR_PULSE_FRAME.fetch_xor(true, Ordering::Relaxed);
}

pub(crate) fn body_style() -> Style {
    Style::default().fg(TEXT_PRIMARY_COLOR)
}

pub(crate) fn muted_style() -> Style {
    Style::default().fg(TEXT_MUTED_COLOR)
}

pub(crate) fn title_style() -> Style {
    Style::default()
        .fg(TITLE_COLOR)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn container_border_style(selected: bool) -> Style {
    if selected {
        Style::default().fg(SELECTED_CONTAINER_BORDER_COLOR)
    } else {
        Style::default().fg(CONTAINER_BORDER_COLOR)
    }
}

pub(crate) fn selected_container_border_style(selected: bool) -> Style {
    container_border_style(selected)
}

pub(crate) fn gothic_block(title: impl Into<String>) -> Block<'static> {
    gothic_block_selected(title, false)
}

pub(crate) fn gothic_block_selected(title: impl Into<String>, selected: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title_style(title_style())
        .title(title.into())
        .border_style(selected_container_border_style(selected))
}

pub(crate) fn footer_text(message: &str, commands: &str) -> String {
    if message.is_empty() {
        commands.to_string()
    } else {
        format!("{message}\n{commands}")
    }
}

pub(crate) fn render_commands_footer(frame: &mut Frame, area: Rect, footer: impl Into<String>) {
    frame.render_widget(
        Paragraph::new(command_footer_lines(footer.into()))
            .block(gothic_block("Commands"))
            .wrap(Wrap { trim: false }),
        area,
    );
}

pub(crate) fn command_footer_lines(footer: impl AsRef<str>) -> Vec<Line<'static>> {
    footer.as_ref().lines().map(command_footer_line).collect()
}

fn command_footer_line(text: &str) -> Line<'static> {
    if text.is_empty() {
        return Line::from("");
    }
    if !text.contains('=') {
        return Line::from(vec![Span::styled(
            text.to_string(),
            Style::default().fg(WARNING_COLOR),
        )]);
    }

    let mut spans = Vec::new();
    for (index, segment) in text.split("  ").enumerate() {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        spans.extend(command_footer_segment_spans(segment));
    }
    Line::from(spans)
}

fn command_footer_segment_spans(segment: &str) -> Vec<Span<'static>> {
    let Some((key_text, label)) = segment.split_once('=') else {
        return vec![Span::styled(segment.to_string(), body_style())];
    };

    let mut spans = command_key_spans(key_text);
    spans.push(Span::styled(format!("={label}"), body_style()));
    spans
}

fn command_key_spans(key_text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut keys = key_text;
    if let Some((title, rest)) = key_text.split_once(": ") {
        spans.push(Span::styled(format!("{title}: "), title_style()));
        keys = rest;
    }

    for (index, key) in keys.split(" or ").enumerate() {
        if index > 0 {
            spans.push(Span::styled(" or ", body_style()));
        }
        spans.push(Span::styled(key.to_string(), command_key_style(key)));
    }
    spans
}

fn command_key_style(key: &str) -> Style {
    let color = if key.eq_ignore_ascii_case("q") || key.eq_ignore_ascii_case("esc") {
        DANGER_COLOR
    } else {
        ACTION_COLOR
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

pub(crate) fn render_town(frame: &mut Frame, c: &Character, town_message: &str) {
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(4),
    ])
    .split(frame.area());

    let title =
        Paragraph::new(Line::from(vec![Span::raw("Hollow's Rest")])).block(gothic_block("Town"));
    frame.render_widget(title, layout[0]);

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                c.name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" the "),
            Span::styled(
                c.class_name().to_string(),
                Style::default().fg(ACTION_COLOR),
            ),
            Span::raw("  "),
            stat_span(format!("Gold {}", c.gold), WARNING_COLOR),
        ]),
        xp_bar_line(c.level, c.xp, xp_required_for_next_level(c.level)),
        Line::from(vec![
            stat_span(format!("HP {}/{}", c.hp, c.max_hp()), Color::Red),
            Span::raw("  "),
            stat_span(
                format!(
                    "{} {}/{}",
                    c.resource_label(),
                    c.current_resource(),
                    c.max_resource()
                ),
                ARCANE_COLOR,
            ),
        ]),
        Line::from(vec![
            stat_span(format!("STR {}", c.strength), Color::Red),
            Span::raw("  "),
            stat_span(format!("DEX {}", c.dexterity), Color::Green),
            Span::raw("  "),
            stat_span(format!("INT {}", c.intelligence), Color::Blue),
            Span::raw("  "),
            stat_span(format!("Hit {}", c.hit_rating()), Color::Cyan),
            Span::raw("  "),
            stat_span(format!("Dodge {}", c.dodge_rating()), Color::Green),
            Span::raw("  "),
            stat_span(format!("Speed {}", c.speed()), Color::Yellow),
        ]),
        Line::from(vec![
            stat_span(
                format!("Unspent attributes: {}", c.unspent_attributes),
                Color::Cyan,
            ),
            Span::raw("  "),
            stat_span(
                format!("Unspent skills: {}", c.unspent_skills),
                CURSED_COLOR,
            ),
        ]),
        Line::from(""),
        equipment_line("Weapon", &c.equipped_weapon),
        equipment_line("Armor ", &c.equipped_armor),
        equipment_line("Shield", &c.equipped_shield),
        equipment_line("Helm  ", &c.equipped_helm),
        equipment_line("Gloves", &c.equipped_gloves),
        equipment_line("Boots ", &c.equipped_boots),
        equipment_line("Belt  ", &c.equipped_belt),
        equipment_line("Amulet", &c.equipped_amulet),
        equipment_line("Ring 1", &c.equipped_ring1),
        equipment_line("Ring 2", &c.equipped_ring2),
        Line::from(""),
        town_quest_line(c),
    ];

    if !town_message.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            strip_ansi_codes(town_message),
            Style::default().fg(WARNING_COLOR),
        ));
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        "Town services: use the footer commands below to choose a service.",
        title_style(),
    ));

    let body = Paragraph::new(lines)
        .block(gothic_block("Status"))
        .wrap(Wrap { trim: false });
    frame.render_widget(body, layout[1]);

    let footer = Paragraph::new(vec![
        command_line(
            "Town",
            &[
                ("m", "merchant"),
                ("b", "blacksmith"),
                ("l", "distillery"),
                ("s", "stash"),
                ("p", "projects"),
                ("t", "quest"),
                ("d", "dungeon"),
            ],
        ),
        command_line(
            "",
            &[
                ("i", "inventory"),
                ("a", "attributes"),
                ("k", "skill tree"),
                ("h", "help"),
                ("q", "save+quit"),
            ],
        ),
    ])
    .block(gothic_block("Commands"));
    frame.render_widget(footer, layout[2]);
}

pub(crate) fn strip_ansi_codes(text: &str) -> String {
    let mut stripped = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for code_ch in chars.by_ref() {
                if code_ch.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            stripped.push(ch);
        }
    }
    stripped
}

pub(crate) fn ansi_styled_spans(text: &str, default_style: Style) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut buffer = String::new();
    let mut style = default_style;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            let mut codes = String::new();
            for code_ch in chars.by_ref() {
                if code_ch.is_ascii_alphabetic() {
                    push_ansi_span(&mut spans, &mut buffer, style);
                    if code_ch == 'm' {
                        style = apply_ansi_sgr_codes(&codes, default_style, style);
                    }
                    break;
                }
                codes.push(code_ch);
            }
        } else {
            buffer.push(ch);
        }
    }
    push_ansi_span(&mut spans, &mut buffer, style);
    spans
}

fn push_ansi_span(spans: &mut Vec<Span<'static>>, buffer: &mut String, style: Style) {
    if !buffer.is_empty() {
        spans.push(Span::styled(std::mem::take(buffer), style));
    }
}

fn apply_ansi_sgr_codes(codes: &str, default_style: Style, current_style: Style) -> Style {
    let mut style = current_style;
    let codes = if codes.is_empty() { "0" } else { codes };

    for code in codes.split(';') {
        match code.parse::<u16>() {
            Ok(0) => style = default_style,
            Ok(1) => style = style.add_modifier(Modifier::BOLD),
            Ok(31) => style = style.fg(DANGER_COLOR),
            Ok(32) => style = style.fg(ACTION_COLOR),
            Ok(33) => style = style.fg(RARITY_RARE_COLOR),
            Ok(34) => style = style.fg(RARITY_MAGIC_COLOR),
            Ok(35) => style = style.fg(CURSED_COLOR),
            Ok(36) => style = style.fg(Color::Cyan),
            Ok(37) => style = style.fg(RARITY_COMMON_COLOR),
            Ok(39) => style = default_style,
            _ => {}
        }
    }

    style
}

pub(crate) fn rarity_color(rarity: &Rarity) -> Color {
    match rarity {
        Rarity::Common => RARITY_COMMON_COLOR,
        Rarity::Magic => RARITY_MAGIC_COLOR,
        Rarity::Rare => RARITY_RARE_COLOR,
    }
}

pub(crate) fn stat_span(text: impl Into<String>, color: Color) -> Span<'static> {
    Span::styled(
        text.into(),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

const XP_BAR_WIDTH: usize = 20;
const XP_BAR_FILLED: &str = "█";
const XP_BAR_EMPTY: &str = "░";

pub(crate) fn xp_bar_line(level: u32, current: u32, needed: u32) -> Line<'static> {
    let (filled, empty, percent) = xp_bar_progress(current, needed, XP_BAR_WIDTH);
    Line::from(vec![
        stat_span(format!("Lv {level}"), TITLE_COLOR),
        Span::raw("  "),
        stat_span("XP", CURSED_COLOR),
        Span::raw(" "),
        Span::styled(
            XP_BAR_FILLED.repeat(filled),
            Style::default()
                .fg(CURSED_COLOR)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(XP_BAR_EMPTY.repeat(empty), muted_style()),
        Span::raw(" "),
        Span::styled(format!("{percent}%"), body_style()),
    ])
}

#[cfg(test)]
pub(crate) fn xp_bar_text(level: u32, current: u32, needed: u32, width: usize) -> String {
    let (filled, empty, percent) = xp_bar_progress(current, needed, width);
    format!(
        "Lv {level}  XP {}{} {percent}%",
        XP_BAR_FILLED.repeat(filled),
        XP_BAR_EMPTY.repeat(empty)
    )
}

fn xp_bar_progress(current: u32, needed: u32, width: usize) -> (usize, usize, u32) {
    if width == 0 {
        return (0, 0, 0);
    }
    if needed == 0 {
        return (width, 0, 100);
    }

    let capped_current = u64::from(current.min(needed));
    let needed = u64::from(needed);
    let filled = ((capped_current * width as u64) / needed).min(width as u64) as usize;
    let percent = ((capped_current * 100) / needed).min(100) as u32;
    (filled, width - filled, percent)
}

pub(crate) fn command_line(title: &str, commands: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    if !title.is_empty() {
        spans.push(Span::styled(format!("{title}: "), title_style()));
    }
    for (index, (key, label)) in commands.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        let color = if *key == "q" {
            DANGER_COLOR
        } else {
            ACTION_COLOR
        };
        spans.push(Span::styled(
            (*key).to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(format!("={label}")));
    }
    Line::from(spans)
}

fn equipment_line(label: &str, item: &Item) -> Line<'static> {
    let item_style = if is_empty_equipment_slot(item) {
        muted_style()
    } else {
        Style::default().fg(rarity_color(&item.rarity))
    };
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(equipped_display_name(item), item_style),
    ])
}

fn town_quest_line(c: &Character) -> Line<'static> {
    if c.act2_completed {
        Line::styled(
            "Act II complete: The Glass Wastes lie quiet under a cold dawn.",
            Style::default().fg(Color::Green),
        )
    } else if c.glass_tyrant_defeated {
        Line::styled(
            "Quest ready to turn in: Speak with Warden Mara (t) about the Glass Tyrant.",
            Style::default().fg(Color::Yellow),
        )
    } else if c.act1_completed {
        Line::styled(
            format!(
                "Act II: Cross the Glass Wastes and shatter the Glass Tyrant on floor {FINAL_FLOOR}."
            ),
            Style::default().fg(Color::Cyan),
        )
    } else if c.bellkeeper_defeated {
        Line::styled(
            "Quest ready to turn in: Speak with Warden Mara (t) about the Bellkeeper.",
            Style::default().fg(Color::Yellow),
        )
    } else {
        Line::from(
            "Quest: Kill the Bellkeeper below the crypt. Speak with Warden Mara (t) for details.",
        )
    }
}

pub(crate) fn gold_text(value: u32) -> String {
    format!("{YELLOW}Gold {value}{RESET}")
}

#[cfg(test)]
pub(crate) fn xp_text(level: u32, current: u32, needed: u32) -> String {
    format!(
        "{MAGENTA}{}{RESET}",
        xp_bar_text(level, current, needed, XP_BAR_WIDTH)
    )
}

pub(crate) fn unspent_skills_text(value: u32) -> String {
    format!("{MAGENTA}Unspent skills: {value}{RESET}")
}

pub(crate) fn shard_text(label: &str, value: u32) -> String {
    format!("{WHITE}{label} {value}{RESET}")
}

pub(crate) fn damage_text(value: impl std::fmt::Display) -> String {
    format!("{RED}{value} damage{RESET}")
}

pub(crate) fn xp_reward_text(value: u32) -> String {
    format!("{MAGENTA}{value} XP{RESET}")
}

pub(crate) fn gold_reward_text(value: u32) -> String {
    format!("{YELLOW}{value} gold{RESET}")
}

pub(crate) fn heal_amount_text(value: u32) -> String {
    format!("{GREEN}{value} HP{RESET}")
}

pub(crate) fn lesser_potion_restore(max_resource: u32) -> u32 {
    ((max_resource * LESSER_POTION_RESTORE_PERCENT) / 100).max(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogKind {
    Hit,
    Enemy,
    Miss,
    Kill,
    Loot,
    Boss,
    Heal,
    Warn,
    Status,
    Info,
}

impl LogKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            LogKind::Hit => "HIT",
            LogKind::Enemy => "ENEMY",
            LogKind::Miss => "MISS",
            LogKind::Kill => "KILL",
            LogKind::Loot => "LOOT",
            LogKind::Boss => "BOSS",
            LogKind::Heal => "HEAL",
            LogKind::Warn => "WARN",
            LogKind::Status => "STATUS",
            LogKind::Info => "INFO",
        }
    }
}

pub(crate) fn log_event(log: &mut Vec<String>, kind: LogKind, message: impl Into<String>) {
    log.push(format!("[{}] {}", kind.label(), message.into()));
}

pub(crate) fn enemy_hp_text(enemy: &Enemy) -> String {
    format!("HP {}/{}", enemy.hp.max(0), enemy.max_hp)
}

pub(crate) fn push_level_up_logs(log: &mut Vec<String>, levels_gained: &[u32]) {
    for level in levels_gained {
        log_event(
            log,
            LogKind::Status,
            format!("LEVEL UP! Reached level {level}. +3 attributes, +1 skill point."),
        );
        log_event(
            log,
            LogKind::Info,
            format!(
                "Town reminder: press {GREEN}a{RESET} for attributes and {GREEN}k{RESET} for skills."
            ),
        );
    }
}
