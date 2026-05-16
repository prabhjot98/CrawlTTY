use crate::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub(crate) fn inventory_screen(
    c: &mut Character,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<bool> {
    let mut selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_grid_cursor(&mut selected, &c.inventory);
        terminal
            .draw(|frame| render_inventory_screen(frame, c, selected, &message))
            .context("failed to draw inventory")?;
        let key = read_key_char_nav()?;
        message.clear();
        match key {
            '\u{1b}' => return Ok(false),
            'w' | 'W' | 'a' | 'A' | 's' | 'S' | 'd' | 'D' => {
                selected = move_grid_cursor(selected, c.inventory.columns, c.inventory.rows, key);
            }
            'x' | 'X' => {
                let result = drop_selected_inventory_item(c, selected);
                message = result.message;
                if result.spent_turn {
                    append_autosave_status(c, &mut message);
                }
                if c.active_dungeon.is_some() && result.spent_turn {
                    log_inventory_action(c, &message);
                    return Ok(true);
                }
            }
            '\n' => {
                let result = equip_or_use_inventory_item(c, selected);
                message = result.message;
                if result.spent_turn {
                    append_autosave_status(c, &mut message);
                }
                if c.active_dungeon.is_some() && result.spent_turn {
                    log_inventory_action(c, &message);
                    return Ok(true);
                }
            }
            _ => message = "Unknown inventory command.".to_string(),
        }
    }
}

pub(crate) fn render_inventory_screen(
    frame: &mut Frame,
    c: &Character,
    selected: usize,
    message: &str,
) {
    let area = frame.area();
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(3),
    ])
    .split(area);
    let title = Paragraph::new(format!(
        "Inventory - Bag {} x {} - {} / {}",
        c.inventory.columns,
        c.inventory.rows,
        c.inventory.len(),
        c.inventory.capacity()
    ))
    .block(Block::default().borders(Borders::ALL).title("Inventory"));
    frame.render_widget(title, layout[0]);

    let body = Layout::horizontal([Constraint::Min(24), Constraint::Length(38)]).split(layout[1]);
    render_item_grid(frame, &c.inventory, selected, body[0], "Bag");
    let details = Paragraph::new(selected_item_detail_lines(
        c,
        &c.inventory,
        "Bag",
        c.inventory.get(selected),
    ))
    .block(Block::default().borders(Borders::ALL).title("Details"));
    frame.render_widget(details, body[1]);

    let footer_text = if message.is_empty() {
        "WASD/Arrows=move  Enter=equip/use  x=drop  Esc=back".to_string()
    } else {
        format!("{message}\nWASD/Arrows=move  Enter=equip/use  x=drop  Esc=back")
    };
    frame.render_widget(
        Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL).title("Commands")),
        layout[2],
    );
}

pub(crate) fn render_item_grid(
    frame: &mut Frame,
    grid: &ItemGrid,
    selected: usize,
    area: Rect,
    title: &str,
) {
    let mut lines = Vec::new();
    for row in 0..grid.rows {
        let mut spans = Vec::new();
        for col in 0..grid.columns {
            let index = usize::from(row) * usize::from(grid.columns) + usize::from(col);
            let label = inventory_cell_label(grid, index);
            let style = if index == selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            spans.push(Span::styled(format!("[{label}]"), style));
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title)),
        area,
    );
}

#[cfg(test)]
pub(crate) fn inventory_screen_text_for_test(
    c: &Character,
    selected: usize,
    message: &str,
) -> Vec<String> {
    let mut lines = vec![format!(
        "Inventory - Bag {} x {} - {} / {}",
        c.inventory.columns,
        c.inventory.rows,
        c.inventory.len(),
        c.inventory.capacity()
    )];
    for row in 0..c.inventory.rows {
        let mut line = String::new();
        for col in 0..c.inventory.columns {
            let index = usize::from(row) * usize::from(c.inventory.columns) + usize::from(col);
            line.push_str(&format!("[{}] ", inventory_cell_label(&c.inventory, index)));
        }
        lines.push(line);
    }
    lines.extend(
        selected_item_detail_lines(c, &c.inventory, "Bag", c.inventory.get(selected))
            .into_iter()
            .map(|line| {
                line.spans
                    .into_iter()
                    .map(|span| span.content.to_string())
                    .collect()
            }),
    );
    if !message.is_empty() {
        lines.push(message.to_string());
    }
    lines.push("Enter=equip/use".to_string());
    lines
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InventoryActionResult {
    pub(crate) message: String,
    pub(crate) spent_turn: bool,
}

impl InventoryActionResult {
    pub(crate) fn spent(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            spent_turn: true,
        }
    }

    pub(crate) fn free(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            spent_turn: false,
        }
    }
}

pub(crate) fn log_inventory_action(c: &mut Character, message: &str) {
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(&mut d.log, LogKind::Info, message);
    }
}

pub(crate) fn print_inventory_preview(c: &Character, max_rows: usize) {
    println!(
        "{BOLD}Your Inventory{RESET} ({CYAN}{}{RESET})",
        c.inventory.len()
    );
    if c.inventory.is_empty() {
        println!("  Empty");
        return;
    }
    let max_rows = max_rows.max(1);
    for item in c.inventory.iter().take(max_rows) {
        println!("  {}", item_summary(item));
    }
    if c.inventory.len() > max_rows {
        println!(
            "  ...{} more. Open sell to browse all.",
            c.inventory.len() - max_rows
        );
    }
}

pub(crate) fn print_inventory_list(c: &Character, selected: usize, max_rows: usize) {
    let total = c.inventory.len();
    let max_rows = max_rows.max(1);
    let offset = scroll_offset(selected, total, max_rows);
    let end = (offset + max_rows).min(total);
    if total > max_rows {
        println!(
            "{DIM}Showing items {}-{} of {}{RESET}",
            offset + 1,
            end,
            total
        );
    }
    for (i, item) in c.inventory.iter().enumerate().skip(offset).take(max_rows) {
        let marker = if i == selected {
            format!("{GREEN}>{RESET}")
        } else {
            " ".to_string()
        };
        println!("{marker} {}", item_summary(item));
        if i == selected {
            if let Some(compare) = item_comparison(c, item) {
                println!("  {compare}");
            }
        }
    }
}

pub(crate) fn print_stash_column(
    title: &str,
    items: &[Item],
    selected: usize,
    active: bool,
    max_rows: usize,
) {
    let heading = if active {
        format!("{BOLD}{GREEN}>{RESET} {BOLD}{title}{RESET}")
    } else {
        format!("  {BOLD}{title}{RESET}")
    };
    println!("{heading}");
    if items.is_empty() {
        println!("  Empty");
        return;
    }
    let max_rows = max_rows.max(1);
    let offset = scroll_offset(selected, items.len(), max_rows);
    let end = (offset + max_rows).min(items.len());
    if items.len() > max_rows {
        println!(
            "  {DIM}Showing items {}-{} of {}{RESET}",
            offset + 1,
            end,
            items.len()
        );
    }
    for (i, item) in items.iter().enumerate().skip(offset).take(max_rows) {
        let marker = if active && i == selected {
            format!("{GREEN}>{RESET}")
        } else {
            " ".to_string()
        };
        println!("{marker} {}", item_summary(item));
    }
}

pub(crate) fn inventory_visible_rows(reserved_rows: u16) -> usize {
    let (_, height) = terminal_size().unwrap_or((80, 24));
    height.saturating_sub(reserved_rows).max(5) as usize
}

pub(crate) fn scroll_offset(selected: usize, total: usize, max_rows: usize) -> usize {
    if total <= max_rows || selected < max_rows {
        0
    } else {
        selected + 1 - max_rows
    }
}

pub(crate) fn clamp_selection(selected: &mut usize, total: usize) {
    if total == 0 {
        *selected = 0;
    } else if *selected >= total {
        *selected = total - 1;
    }
}

#[allow(dead_code)]
pub(crate) fn move_grid_cursor(selected: usize, columns: u16, rows: u16, key: char) -> usize {
    let columns = usize::from(columns);
    let rows = usize::from(rows);
    let capacity = columns * rows;
    if capacity == 0 {
        return 0;
    }
    let selected = selected.min(capacity - 1);
    let col = selected % columns;
    let row = selected / columns;
    match key {
        'w' | 'W' if row > 0 => selected - columns,
        's' | 'S' if row + 1 < rows => selected + columns,
        'a' | 'A' if col > 0 => selected - 1,
        'd' | 'D' if col + 1 < columns => selected + 1,
        _ => selected,
    }
}

#[allow(dead_code)]
pub(crate) fn inventory_cell_label(grid: &ItemGrid, index: usize) -> &'static str {
    let Some(item) = grid.get(index) else {
        return ".";
    };
    match item.kind {
        ItemKind::HealthPotion => "H",
        ItemKind::ManaPotion => "M",
        ItemKind::Weapon => "W",
        ItemKind::Armor => "A",
        ItemKind::Shield => "S",
        ItemKind::Gem => "G",
    }
}

#[allow(dead_code)]
pub(crate) fn clamp_grid_cursor(selected: &mut usize, grid: &ItemGrid) {
    let capacity = grid.capacity();
    if capacity == 0 {
        *selected = 0;
    } else if *selected >= capacity {
        *selected = capacity - 1;
    }
}

#[allow(dead_code)]
pub(crate) fn selected_item_detail_lines(
    c: &Character,
    grid: &ItemGrid,
    grid_label: &str,
    item: Option<&Item>,
) -> Vec<Line<'static>> {
    let Some(item) = item else {
        return vec![
            Line::styled("Empty cell", Style::default().fg(Color::DarkGray)),
            Line::from(format!(
                "{}: {}/{}",
                grid_label,
                grid.len(),
                grid.capacity()
            )),
        ];
    };
    let mut lines = vec![
        Line::styled(
            item.name.clone(),
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
        ItemKind::Gem => lines.push(Line::from(strip_ansi_codes(&gem_summary(item)))),
    }
    if let Some(compare) = item_comparison(c, item) {
        lines.push(Line::from(strip_ansi_codes(&compare)));
    }
    lines
}

pub(crate) fn drop_selected_inventory_item(
    c: &mut Character,
    index: usize,
) -> InventoryActionResult {
    if c.inventory.is_empty() {
        InventoryActionResult::free("Inventory is empty.")
    } else if index >= c.inventory.len() {
        InventoryActionResult::free("No item selected.")
    } else {
        let item = c.inventory.remove(index);
        let message = if let Some(d) = c.active_dungeon.as_mut() {
            let name = item.name.clone();
            add_ground_item(d, d.player_x, d.player_y, item);
            format!("Dropped {name} on the ground.")
        } else {
            format!("Dropped {}.", item.name)
        };
        InventoryActionResult::spent(message)
    }
}

pub(crate) fn item_level_text(item: &Item) -> String {
    if matches!(
        item.kind,
        ItemKind::Weapon | ItemKind::Armor | ItemKind::Shield
    ) {
        format!("{CYAN}ilvl {}{RESET}", item.item_level)
    } else {
        String::new()
    }
}

pub(crate) fn item_requirements_text(item: &Item) -> String {
    let mut reqs = Vec::new();
    if item.required_strength > 0 {
        reqs.push(format!("{RED}STR {}{RESET}", item.required_strength));
    }
    if item.required_dexterity > 0 {
        reqs.push(format!("{GREEN}DEX {}{RESET}", item.required_dexterity));
    }
    if item.required_intelligence > 0 {
        reqs.push(format!("{BLUE}INT {}{RESET}", item.required_intelligence));
    }
    if reqs.is_empty() {
        String::new()
    } else {
        format!("req {}", reqs.join("/"))
    }
}

pub(crate) fn item_level_and_requirements(item: &Item) -> String {
    let item_level = item_level_text(item);
    let requirements = item_requirements_text(item);
    match (item_level.is_empty(), requirements.is_empty()) {
        (true, true) => String::new(),
        (false, true) => item_level,
        (true, false) => requirements,
        (false, false) => format!("{item_level} {requirements}"),
    }
}

pub(crate) fn item_summary(item: &Item) -> String {
    let rarity = rarity_name(&item.rarity);
    let name = colored_item_name(item);
    let upgrade = if item.upgrade_level > 0 {
        format!(" +{}", item.upgrade_level)
    } else {
        String::new()
    };
    let level_and_requirements = item_level_and_requirements(item);
    match item.kind {
        ItemKind::Weapon => {
            format!(
                "{}{} [{} {:?}] {} {RED}dmg {}-{}{RESET} {CYAN}crit {}%{RESET} {YELLOW}value {}{RESET}",
                name,
                upgrade,
                rarity,
                item.kind,
                level_and_requirements,
                item.damage_min,
                item.damage_max,
                item.crit_chance,
                item.value
            ) + &socket_summary(item)
        }
        ItemKind::Armor | ItemKind::Shield => {
            format!(
                "{}{} [{} {:?}] {} {WHITE}armor {}{RESET} {GREEN}dodge {}{RESET} {YELLOW}speed {}{RESET} {YELLOW}value {}{RESET}",
                name,
                upgrade,
                rarity,
                item.kind,
                level_and_requirements,
                item.armor,
                item.dodge,
                item.speed,
                item.value
            ) + &socket_summary(item)
        }
        ItemKind::Gem => gem_summary(item),
        _ => format!(
            "{} [{:?}] {YELLOW}value {}{RESET}",
            name, item.kind, item.value
        ),
    }
}

fn gem_summary(item: &Item) -> String {
    let (Some(kind), Some(tier)) = (item.gem_kind, item.gem_tier) else {
        return format!(
            "{RED}Invalid gem metadata{RESET} [Gem] {YELLOW}value {}{RESET}",
            item.value
        );
    };

    let bonus = gem_bonus_text(gem_bonus(kind, tier));
    format!(
        "{WHITE}{} {}{RESET} ({bonus}) [Gem] {YELLOW}value {}{RESET}",
        gem_tier_name(tier),
        gem_kind_name(kind),
        item.value
    )
}

fn socket_summary(item: &Item) -> String {
    if item.sockets.is_empty() {
        return String::new();
    }

    let sockets = item
        .sockets
        .iter()
        .map(|socket| match socket {
            Some(socket) => format!(
                "{} {}",
                gem_tier_name(socket.gem_tier),
                gem_kind_name(socket.gem_kind)
            ),
            None => "empty".to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(" {CYAN}Sockets [{sockets}]{RESET}")
}

pub(crate) fn colored_item_name(item: &Item) -> String {
    let color = match item.rarity {
        Rarity::Common => WHITE,
        Rarity::Magic => BLUE,
        Rarity::Rare => YELLOW,
    };
    format!("{color}{}{RESET}", item.name)
}

pub(crate) fn item_comparison(c: &Character, item: &Item) -> Option<String> {
    let comparison = match item.kind {
        ItemKind::Weapon => {
            let cur_avg = c.equipped_weapon.damage_min + c.equipped_weapon.damage_max;
            let new_avg = item.damage_min + item.damage_max;
            format!(
                "Compare: {}  {}",
                format_delta("damage", new_avg - cur_avg),
                format_crit_delta(item.crit_chance as i32 - c.equipped_weapon.crit_chance as i32)
            )
        }
        ItemKind::Armor => format!(
            "Compare: {}  {}  {}",
            format_delta("armor", item.armor - c.equipped_armor.armor),
            format_delta("dodge", item.dodge - c.equipped_armor.dodge),
            format_delta("speed", item.speed - c.equipped_armor.speed)
        ),
        ItemKind::Shield => format!(
            "Compare: {}  {}  {}",
            format_delta("armor", item.armor - c.equipped_shield.armor),
            format_delta("dodge", item.dodge - c.equipped_shield.dodge),
            format_delta("speed", item.speed - c.equipped_shield.speed)
        ),
        _ => return None,
    };
    if let Some(requirements) = unmet_requirements_message(c, item) {
        Some(format!("{comparison}  {RED}LOCKED:{RESET} {requirements}"))
    } else {
        Some(comparison)
    }
}

pub(crate) fn format_delta(label: &str, delta: i32) -> String {
    if delta > 0 {
        format!("{GREEN}+{delta} {label}{RESET}")
    } else if delta < 0 {
        format!("{RED}{delta} {label}{RESET}")
    } else {
        format!("+0 {label}")
    }
}

fn format_crit_delta(delta: i32) -> String {
    if delta > 0 {
        format!("{GREEN}crit +{delta}{RESET}")
    } else if delta < 0 {
        format!("{RED}crit {delta}{RESET}")
    } else {
        "crit +0".to_string()
    }
}

pub(crate) fn can_equip_item(c: &Character, item: &Item) -> bool {
    c.strength >= item.required_strength
        && c.dexterity >= item.required_dexterity
        && c.intelligence >= item.required_intelligence
}

pub(crate) fn unmet_requirements_message(c: &Character, item: &Item) -> Option<String> {
    if can_equip_item(c, item) {
        return None;
    }
    let mut missing = Vec::new();
    if c.strength < item.required_strength {
        missing.push(format!(
            "{RED}STR {}/{}{RESET}",
            c.strength, item.required_strength
        ));
    }
    if c.dexterity < item.required_dexterity {
        missing.push(format!(
            "{GREEN}DEX {}/{}{RESET}",
            c.dexterity, item.required_dexterity
        ));
    }
    if c.intelligence < item.required_intelligence {
        missing.push(format!(
            "{BLUE}INT {}/{}{RESET}",
            c.intelligence, item.required_intelligence
        ));
    }
    Some(format!("Requires {}.", missing.join(", ")))
}

pub(crate) fn equip_or_use_inventory_item(
    c: &mut Character,
    index: usize,
) -> InventoryActionResult {
    if c.inventory.get(index).is_none() {
        return InventoryActionResult::free("No item in that slot.");
    }
    let selected = c.inventory.remove(index);
    if matches!(
        selected.kind,
        ItemKind::Weapon | ItemKind::Armor | ItemKind::Shield
    ) {
        if let Some(message) = unmet_requirements_message(c, &selected) {
            c.inventory.insert(index, selected);
            return InventoryActionResult::free(message);
        }
    }
    match selected.kind {
        ItemKind::Weapon => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_weapon, selected);
            assert!(
                c.inventory.insert(index, old),
                "ItemGrid invariant broken: equipping weapon should free inventory capacity for old gear"
            );
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::Armor => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_armor, selected);
            assert!(
                c.inventory.insert(index, old),
                "ItemGrid invariant broken: equipping armor should free inventory capacity for old gear"
            );
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::Shield => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_shield, selected);
            assert!(
                c.inventory.insert(index, old),
                "ItemGrid invariant broken: equipping shield should free inventory capacity for old gear"
            );
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::HealthPotion => {
            if c.hp >= c.max_hp() {
                c.inventory.insert(index, selected);
                return InventoryActionResult::free("HP is already full.");
            }
            let heal = lesser_potion_restore(c.max_hp());
            let before = c.hp;
            c.hp = (c.hp + heal).min(c.max_hp());
            let restored = c.hp - before;
            InventoryActionResult::spent(format!(
                "Used a lesser health potion and restored {restored} HP."
            ))
        }
        ItemKind::ManaPotion => {
            if c.mana >= c.max_mana() {
                c.inventory.insert(index, selected);
                return InventoryActionResult::free("Mana is already full.");
            }
            let restore = lesser_potion_restore(c.max_mana());
            let before = c.mana;
            c.mana = (c.mana + restore).min(c.max_mana());
            let restored = c.mana - before;
            InventoryActionResult::spent(format!(
                "Used a lesser mana potion and restored {restored} mana."
            ))
        }
        ItemKind::Gem => {
            c.inventory.insert(index, selected);
            InventoryActionResult::free("Use the Socket Bench to socket gems.")
        }
    }
}

fn clamp_current_resources(c: &mut Character) {
    c.hp = c.hp.min(c.max_hp());
    c.mana = c.mana.min(c.max_mana());
}
