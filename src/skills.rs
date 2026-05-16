use crate::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub(crate) fn skill_tree_menu(
    c: &mut Character,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<()> {
    let mut selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_selection(&mut selected, SKILL_TREE_SKILLS.len());
        terminal
            .draw(|frame| render_skill_tree_screen(frame, c, selected, &message))
            .context("failed to draw skill tree")?;
        let key = match read_ui_input_nav()? {
            UiInput::Key(key) => key,
            UiInput::Redraw => continue,
        };
        message.clear();
        match key {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < SKILL_TREE_SKILLS.len() {
                    selected += 1;
                }
            }
            '\n' => {
                message = choose_skill_or_mastery(c, terminal, SKILL_TREE_SKILLS[selected])?;
                append_autosave_status(c, &mut message);
            }
            _ => message = "Unknown skill command.".to_string(),
        }
    }
    Ok(())
}

const SKILL_TREE_SKILLS: [&str; 6] = [
    "Cleave",
    "Deep Cut",
    "Shield Bash",
    "Iron Guard",
    "Battle Cry",
    "Second Wind",
];

pub(crate) fn render_skill_tree_screen(
    frame: &mut Frame,
    c: &Character,
    selected: usize,
    message: &str,
) {
    let mut lines = vec![
        Line::styled(
            "Ironbound Skill Tree",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        skill_line(strip_ansi_codes(&unspent_skills_text(c.unspent_skills))),
        Line::from(""),
        Line::styled(
            "Weapons Branch",
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];
    append_skill_choice_lines(&mut lines, selected, "Cleave", c.cleave_rank);
    append_mastery_status_lines(&mut lines, c, "Cleave");
    append_skill_choice_lines(&mut lines, selected, "Deep Cut", c.deep_cut_rank);
    append_mastery_status_lines(&mut lines, c, "Deep Cut");
    lines.push(Line::styled(
        "Defense Branch",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    append_skill_choice_lines(&mut lines, selected, "Shield Bash", c.shield_bash_rank);
    append_mastery_status_lines(&mut lines, c, "Shield Bash");
    append_skill_choice_lines(&mut lines, selected, "Iron Guard", c.iron_guard_rank);
    append_mastery_status_lines(&mut lines, c, "Iron Guard");
    lines.push(Line::styled(
        "Warcry Branch",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    append_skill_choice_lines(&mut lines, selected, "Battle Cry", c.battle_cry_rank);
    append_mastery_status_lines(&mut lines, c, "Battle Cry");
    append_skill_choice_lines(&mut lines, selected, "Second Wind", c.second_wind_rank);
    append_mastery_status_lines(&mut lines, c, "Second Wind");
    lines.push(Line::from(""));
    lines.push(skill_line("Each rank upgrade costs 1 skill point. Masteries are free at rank 5. Passive upgrades require rank 2 in their branch starter."));

    render_skill_tree_layout(
        frame,
        lines,
        selected_skill_detail_lines(
            c,
            SKILL_TREE_SKILLS[selected.min(SKILL_TREE_SKILLS.len() - 1)],
        ),
        message,
        "Skill Tree: W/S or arrows=select  Enter=upgrade/mastery  Esc=back",
    );
}

fn skill_line(text: impl Into<String>) -> Line<'static> {
    Line::from(strip_ansi_codes(&text.into()))
}

fn selected_skill_line(selected: bool, text: impl Into<String>) -> Line<'static> {
    let prefix = if selected { "> " } else { "  " };
    let style = if selected {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Line::styled(format!("{prefix}{}", strip_ansi_codes(&text.into())), style)
}

fn append_skill_choice_lines(
    lines: &mut Vec<Line<'static>>,
    selected: usize,
    name: &str,
    rank: u32,
) {
    let index = SKILL_TREE_SKILLS
        .iter()
        .position(|skill| *skill == name)
        .unwrap_or(0);
    lines.push(selected_skill_line(
        selected == index,
        format!("{name} rank {rank}/5"),
    ));
}

fn append_mastery_status_lines(lines: &mut Vec<Line<'static>>, c: &Character, skill: &str) {
    if skill_rank(c, skill) < 5 {
        return;
    }
    if let Some(mastery) = mastery_for_skill(c, skill) {
        lines.push(skill_line(format!("   Mastery: {}", mastery.name())));
    } else {
        lines.push(skill_line(
            "   Mastery available: select this skill to choose a free path.",
        ));
    }
}

fn render_skill_tree_layout(
    frame: &mut Frame,
    skill_lines: Vec<Line<'static>>,
    detail_lines: Vec<Line<'static>>,
    message: &str,
    commands: &str,
) {
    let footer_height = if message.is_empty() { 3 } else { 4 };
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(footer_height),
    ])
    .split(frame.area());
    frame.render_widget(
        Paragraph::new("Ironbound Skill Tree").block(
            Block::default()
                .borders(Borders::ALL)
                .title("Ironbound Skill Tree"),
        ),
        layout[0],
    );

    let body = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);
    frame.render_widget(
        Paragraph::new(skill_lines)
            .block(Block::default().borders(Borders::ALL).title("Skills"))
            .wrap(Wrap { trim: false }),
        body[0],
    );
    frame.render_widget(
        Paragraph::new(detail_lines)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: false }),
        body[1],
    );

    let footer = if message.is_empty() {
        commands.to_string()
    } else {
        format!("{message}\n{commands}")
    };
    frame.render_widget(
        Paragraph::new(footer).block(Block::default().borders(Borders::ALL).title("Commands")),
        layout[2],
    );
}

fn selected_skill_detail_lines(c: &Character, skill: &str) -> Vec<Line<'static>> {
    let rank = skill_rank(c, skill);
    let mut lines = vec![
        Line::styled(
            "Current Skill",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        skill_line(format!("{skill} rank {rank}/5")),
    ];
    lines.extend(
        skill_effect_lines(c, skill, rank)
            .into_iter()
            .map(skill_line),
    );
    if let Some(mastery) = mastery_for_skill(c, skill) {
        lines.push(skill_line(format!("Mastery: {}", mastery.name())));
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        "Improved Skill",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    if rank >= 5 {
        lines.push(skill_line("MAX RANK"));
        if mastery_for_skill(c, skill).is_none() {
            lines.push(skill_line("Enter opens a free mastery choice."));
        }
    } else {
        let next_rank = next_skill_rank(rank);
        lines.push(skill_line(format!("Next rank {next_rank}/5")));
        lines.extend(
            skill_effect_lines(c, skill, next_rank)
                .into_iter()
                .map(skill_line),
        );
        if let Some(requirement) = unmet_skill_prerequisite(c, skill) {
            lines.push(skill_line(format!("Locked: {requirement}")));
        }
    }
    lines
}

fn skill_effect_lines(c: &Character, skill: &str, rank: u32) -> Vec<String> {
    match skill {
        "Cleave" => vec![
            format!("{}% weapon damage", cleave_percent_for_rank(rank)),
            "5 mana, cooldown 1".to_string(),
            "Hits up to 3 adjacent enemies.".to_string(),
        ],
        "Deep Cut" => vec![
            format!("{}% bleed chance", deep_cut_chance_for_rank(rank)),
            format!("{} bleed damage/turn", deep_cut_damage_for_rank(rank)),
            "Passive; requires Cleave rank 2.".to_string(),
        ],
        "Shield Bash" => vec![
            format!("{}% weapon damage", shield_bash_percent_for_rank(rank)),
            "6 mana, cooldown 3".to_string(),
            "Hits 1 enemy and staggers.".to_string(),
        ],
        "Iron Guard" => vec![
            format!(
                "+{} armor with shield",
                iron_guard_armor_bonus_for_rank(rank)
            ),
            "Passive; requires Shield Bash rank 2.".to_string(),
        ],
        "Battle Cry" => vec![
            format!("+{}% bonus damage", battle_cry_bonus_percent_for_rank(rank)),
            "8 mana, cooldown 6".to_string(),
            "Grants attack charges.".to_string(),
        ],
        "Second Wind" => vec![
            format!(
                "{}% max HP heal on kill",
                second_wind_heal_percent_for_rank(rank)
            ),
            "Triggers while Battle Cry is active.".to_string(),
            "Passive; requires Battle Cry rank 2.".to_string(),
        ],
        _ => vec![format!("Unknown skill for {}.", c.name)],
    }
}

fn render_skill_lines_screen(
    frame: &mut Frame,
    screen_title: &str,
    body_title: &str,
    lines: Vec<Line<'static>>,
    message: &str,
    commands: &str,
) {
    let footer_height = if message.is_empty() { 3 } else { 4 };
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(footer_height),
    ])
    .split(frame.area());
    frame.render_widget(
        Paragraph::new(screen_title.to_string())
            .block(Block::default().borders(Borders::ALL).title(screen_title)),
        layout[0],
    );
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(body_title))
            .wrap(Wrap { trim: false }),
        layout[1],
    );
    let footer = if message.is_empty() {
        commands.to_string()
    } else {
        format!("{message}\n{commands}")
    };
    frame.render_widget(
        Paragraph::new(footer).block(Block::default().borders(Borders::ALL).title("Commands")),
        layout[2],
    );
}

pub(crate) fn choose_skill_or_mastery(
    c: &mut Character,
    terminal: &mut ratatui::DefaultTerminal,
    skill: &str,
) -> Result<String> {
    if skill_rank(c, skill) >= 5 {
        if mastery_for_skill(c, skill).is_some() {
            return Ok(format!(
                "{skill} already has a mastery: {}.",
                mastery_for_skill(c, skill).unwrap().name()
            ));
        }
        return mastery_menu(c, terminal, skill);
    }
    Ok(upgrade_skill(c, skill))
}

pub(crate) fn upgrade_skill(c: &mut Character, skill: &str) -> String {
    if c.unspent_skills == 0 {
        return "No unspent skill points.".to_string();
    }
    if skill_rank(c, skill) >= 5 {
        return "That skill is already at max rank.".to_string();
    }
    if let Some(requirement) = unmet_skill_prerequisite(c, skill) {
        return requirement;
    }
    match skill {
        "Cleave" => c.cleave_rank += 1,
        "Shield Bash" => c.shield_bash_rank += 1,
        "Battle Cry" => c.battle_cry_rank += 1,
        "Deep Cut" => c.deep_cut_rank += 1,
        "Iron Guard" => c.iron_guard_rank += 1,
        "Second Wind" => c.second_wind_rank += 1,
        _ => return "Unknown skill.".to_string(),
    }
    c.unspent_skills -= 1;
    format!("Upgraded {skill} to rank {}.", skill_rank(c, skill))
}

impl SkillMastery {
    pub(crate) fn name(self) -> &'static str {
        match self {
            SkillMastery::ReapingCleave => "Reaping Cleave",
            SkillMastery::SunderingCleave => "Sundering Cleave",
            SkillMastery::BloodArc => "Blood Arc",
            SkillMastery::CrushingBash => "Crushing Bash",
            SkillMastery::LongBash => "Long Bash",
            SkillMastery::DazingBash => "Dazing Bash",
            SkillMastery::WarpathCry => "Warpath Cry",
            SkillMastery::TerrifyingCry => "Terrifying Cry",
            SkillMastery::RallyingCry => "Rallying Cry",
            SkillMastery::Hemorrhage => "Hemorrhage",
            SkillMastery::OpenWound => "Open Wound",
            SkillMastery::Bloodletting => "Bloodletting",
            SkillMastery::Bulwark => "Bulwark",
            SkillMastery::ShieldDiscipline => "Shield Discipline",
            SkillMastery::SpikedGuard => "Spiked Guard",
            SkillMastery::FreshKill => "Fresh Kill",
            SkillMastery::AdrenalSurge => "Adrenal Surge",
            SkillMastery::GrimRecovery => "Grim Recovery",
        }
    }
}

pub(crate) fn mastery_for_skill(c: &Character, skill: &str) -> Option<SkillMastery> {
    match skill {
        "Cleave" => c.cleave_mastery,
        "Shield Bash" => c.shield_bash_mastery,
        "Battle Cry" => c.battle_cry_mastery,
        "Deep Cut" => c.deep_cut_mastery,
        "Iron Guard" => c.iron_guard_mastery,
        "Second Wind" => c.second_wind_mastery,
        _ => None,
    }
}

pub(crate) fn set_mastery_for_skill(c: &mut Character, skill: &str, mastery: SkillMastery) {
    match skill {
        "Cleave" => c.cleave_mastery = Some(mastery),
        "Shield Bash" => c.shield_bash_mastery = Some(mastery),
        "Battle Cry" => c.battle_cry_mastery = Some(mastery),
        "Deep Cut" => c.deep_cut_mastery = Some(mastery),
        "Iron Guard" => c.iron_guard_mastery = Some(mastery),
        "Second Wind" => c.second_wind_mastery = Some(mastery),
        _ => {}
    }
}

pub(crate) fn mastery_options(c: &Character, skill: &str) -> [(SkillMastery, String); 3] {
    match skill {
        "Cleave" => [
            (
                SkillMastery::ReapingCleave,
                "Cleave target cap is removed: hit every cardinally adjacent enemy instead of max 3. Still costs 5 mana and spends 1 Battle Cry charge for the whole Cleave.".to_string(),
            ),
            (
                SkillMastery::SunderingCleave,
                "Each Cleave hit applies -2 enemy armor for 3 enemy turns. Stacks with normal damage and can reduce effective armor to 0, but not below 0.".to_string(),
            ),
            (
                SkillMastery::BloodArc,
                format!(
                    "Each Cleave hit forces Bleeding for 3 turns. Bleed damage uses your current Deep Cut value: {} damage/turn.",
                    deep_cut_damage_for_rank(c.deep_cut_rank)
                ),
            ),
        ],
        "Shield Bash" => [
            (
                SkillMastery::CrushingBash,
                format!(
                    "Shield Bash gains +10 percentage points of weapon damage per shield armor. Current shield armor {} = +{}% weapon damage.",
                    c.equipped_shield.armor.max(0),
                    c.equipped_shield.armor.max(0) * 10
                ),
            ),
            (
                SkillMastery::LongBash,
                "Shield Bash range increases from adjacent only to 2 tiles in a cardinal line. Still costs 6 mana, stuns, and spends 1 Battle Cry charge.".to_string(),
            ),
            (
                SkillMastery::DazingBash,
                "Shield Bash stun increases from 1 turn to 2 turns. Damage, mana cost, and cooldown are unchanged.".to_string(),
            ),
        ],
        "Battle Cry" => [
            (
                SkillMastery::WarpathCry,
                "Battle Cry grants +2 attack charges: 7 total instead of 5. Movement still does not consume charges.".to_string(),
            ),
            (
                SkillMastery::TerrifyingCry,
                "On activation, enemies within 3 tiles are staggered and skip 1 turn. Battle Cry still grants attack charges and damage reduction.".to_string(),
            ),
            (
                SkillMastery::RallyingCry,
                format!(
                    "On activation, restore 20% max HP and 20% max mana. Current values: {} HP and {} mana.",
                    (c.max_hp() / 5).max(1),
                    (c.max_mana() / 5).max(1)
                ),
            ),
        ],
        "Deep Cut" => [
            (
                SkillMastery::Hemorrhage,
                format!(
                    "Bleeding enemies below 50% HP take +2 bleed damage per tick. Current bleed becomes {} damage/turn while they are low HP.",
                    deep_cut_damage_for_rank(c.deep_cut_rank) + 2
                ),
            ),
            (
                SkillMastery::OpenWound,
                "When Deep Cut applies Bleeding, it also applies Vulnerable for 3 turns. Your physical hits deal +2 raw damage to Vulnerable enemies.".to_string(),
            ),
            (
                SkillMastery::Bloodletting,
                format!(
                    "Enemies killed by bleed restore 10% max HP. Current heal: {} HP.",
                    (c.max_hp() / 10).max(1)
                ),
            ),
        ],
        "Iron Guard" => [
            (
                SkillMastery::Bulwark,
                "Gain +4 armor while at or below 50% HP. This is added on top of Iron Guard's normal shield armor bonus.".to_string(),
            ),
            (
                SkillMastery::ShieldDiscipline,
                "Gain +3 dodge while using Iron Guard. This is a flat bonus to your dodge rating.".to_string(),
            ),
            (
                SkillMastery::SpikedGuard,
                "When an adjacent melee enemy hits you, Spiked Guard deals 2 physical damage back to that attacker.".to_string(),
            ),
        ],
        _ => [
            (
                SkillMastery::FreshKill,
                format!(
                    "Second Wind can trigger without Battle Cry, but heals for 50% of normal. Current no-Cry heal: {} HP; Battle Cry heal remains {} HP.",
                    (second_wind_heal_amount(c) / 2).max(1),
                    second_wind_heal_amount(c)
                ),
            ),
            (
                SkillMastery::AdrenalSurge,
                "When Second Wind triggers while Battle Cry has charges, restore +1 Battle Cry charge after the kill.".to_string(),
            ),
            (
                SkillMastery::GrimRecovery,
                "Second Wind overhealing becomes a temporary damage shield. The shield absorbs incoming damage before HP until depleted.".to_string(),
            ),
        ],
    }
}

pub(crate) fn mastery_menu(
    c: &mut Character,
    terminal: &mut ratatui::DefaultTerminal,
    skill: &str,
) -> Result<String> {
    let mut message = String::new();
    loop {
        terminal
            .draw(|frame| render_mastery_screen(frame, c, skill, &message))
            .context("failed to draw mastery menu")?;
        let options = mastery_options(c, skill);
        let key = match read_ui_input()? {
            UiInput::Key(key) => key,
            UiInput::Redraw => continue,
        };
        match key {
            key @ ('1' | '2' | '3') => {
                let index = key.to_digit(10).unwrap() as usize - 1;
                let mastery = options[index].0;
                set_mastery_for_skill(c, skill, mastery);
                return Ok(format!("Unlocked {} for {skill}.", mastery.name()));
            }
            '\u{1b}' => return Ok("Mastery selection cancelled.".to_string()),
            _ => message = "Unknown mastery command.".to_string(),
        }
    }
}

pub(crate) fn render_mastery_screen(frame: &mut Frame, c: &Character, skill: &str, message: &str) {
    let mut lines = vec![
        Line::styled(
            format!("{skill} Mastery"),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        skill_line("Choose one free path. The other two will be locked out permanently."),
        Line::from(""),
    ];
    lines.extend(mastery_options(c, skill).into_iter().enumerate().flat_map(
        |(i, (mastery, details))| {
            [
                skill_line(format!("{}. {}", i + 1, mastery.name())),
                skill_line(format!("   {details}")),
            ]
        },
    ));
    render_skill_lines_screen(
        frame,
        &format!("{skill} Mastery"),
        "Mastery",
        lines,
        message,
        "Mastery: 1-3=choose  Esc=back",
    );
}

pub(crate) fn skill_rank(c: &Character, skill: &str) -> u32 {
    match skill {
        "Cleave" => c.cleave_rank,
        "Shield Bash" => c.shield_bash_rank,
        "Battle Cry" => c.battle_cry_rank,
        "Deep Cut" => c.deep_cut_rank,
        "Iron Guard" => c.iron_guard_rank,
        "Second Wind" => c.second_wind_rank,
        _ => 5,
    }
}

pub(crate) fn unmet_skill_prerequisite(c: &Character, skill: &str) -> Option<String> {
    match skill {
        "Deep Cut" if c.cleave_rank < 2 => {
            Some("Deep Cut upgrades require Cleave rank 2.".to_string())
        }
        "Iron Guard" if c.shield_bash_rank < 2 => {
            Some("Iron Guard upgrades require Shield Bash rank 2.".to_string())
        }
        "Second Wind" if c.battle_cry_rank < 2 => {
            Some("Second Wind upgrades require Battle Cry rank 2.".to_string())
        }
        _ => None,
    }
}

pub(crate) fn next_skill_rank(rank: u32) -> u32 {
    (rank + 1).min(5)
}
pub(crate) fn cleave_multiplier(c: &Character) -> f32 {
    cleave_multiplier_for_rank(c.cleave_rank)
}
pub(crate) fn cleave_multiplier_for_rank(rank: u32) -> f32 {
    0.8 + (rank.saturating_sub(1) as f32 * 0.10)
}
pub(crate) fn shield_bash_multiplier(c: &Character) -> f32 {
    shield_bash_multiplier_for_rank(c.shield_bash_rank)
}
pub(crate) fn shield_bash_multiplier_for_rank(rank: u32) -> f32 {
    0.7 + (rank.saturating_sub(1) as f32 * 0.10)
}
pub(crate) fn battle_cry_multiplier(c: &Character) -> f32 {
    battle_cry_multiplier_for_rank(c.battle_cry_rank)
}
pub(crate) fn battle_cry_multiplier_for_rank(rank: u32) -> f32 {
    1.20 + (rank.saturating_sub(1) as f32 * 0.05)
}
pub(crate) fn cleave_percent_for_rank(rank: u32) -> u32 {
    (cleave_multiplier_for_rank(rank) * 100.0).round() as u32
}
pub(crate) fn shield_bash_percent_for_rank(rank: u32) -> u32 {
    (shield_bash_multiplier_for_rank(rank) * 100.0).round() as u32
}
pub(crate) fn battle_cry_bonus_percent_for_rank(rank: u32) -> u32 {
    ((battle_cry_multiplier_for_rank(rank) - 1.0) * 100.0).round() as u32
}
pub(crate) fn cleave_percent(c: &Character) -> u32 {
    cleave_percent_for_rank(c.cleave_rank)
}
pub(crate) fn shield_bash_percent(c: &Character) -> u32 {
    shield_bash_percent_for_rank(c.shield_bash_rank)
}
pub(crate) fn battle_cry_bonus_percent(c: &Character) -> u32 {
    battle_cry_bonus_percent_for_rank(c.battle_cry_rank)
}
pub(crate) fn deep_cut_chance_for_rank(rank: u32) -> u32 {
    10 + rank.min(5) * 5
}
pub(crate) fn deep_cut_damage_for_rank(rank: u32) -> i32 {
    1 + rank.min(5).div_ceil(2) as i32
}
pub(crate) fn iron_guard_armor_bonus(c: &Character) -> i32 {
    iron_guard_armor_bonus_for_rank(c.iron_guard_rank)
}
pub(crate) fn iron_guard_armor_bonus_for_rank(rank: u32) -> i32 {
    1 + rank.min(5) as i32
}
pub(crate) fn second_wind_heal_percent_for_rank(rank: u32) -> u32 {
    5 + rank.min(5) * 5
}
pub(crate) fn second_wind_heal_amount(c: &Character) -> u32 {
    ((c.max_hp() * second_wind_heal_percent_for_rank(c.second_wind_rank)) / 100).max(1)
}
