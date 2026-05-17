use crate::*;
use ratatui::{prelude::*, widgets::Paragraph};

const INVENTORY_COMMANDS: &str =
    "Tab=switch  WASD/Arrows=move  Enter=equip/use  o=sort  x=drop  Esc=back";

pub(crate) fn inventory_screen(
    c: &mut Character,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<InventoryScreenExit> {
    let mut bag_selected = 0usize;
    let mut character_selected = CharacterEquipmentSlot::Weapon;
    let mut focus = InventoryFocus::Bag;
    let mut message = String::new();
    loop {
        clamp_grid_cursor(&mut bag_selected, &c.inventory);
        terminal
            .draw(|frame| {
                render_inventory_screen_with_focus(
                    frame,
                    c,
                    bag_selected,
                    character_selected,
                    focus,
                    &message,
                )
            })
            .context("failed to draw inventory")?;
        let key = match read_ui_input_nav_timed(CURSOR_PULSE_INTERVAL)? {
            UiInput::Key(key) => key,
            UiInput::Redraw => continue,
            UiInput::Tick => {
                toggle_cursor_pulse_frame();
                continue;
            }
        };
        message.clear();
        match key {
            '\u{1b}' => return Ok(InventoryScreenExit::NoTurn),
            '\t' => focus = focus.toggle(),
            'w' | 'W' | 'a' | 'A' | 's' | 'S' | 'd' | 'D' => match focus {
                InventoryFocus::Bag => {
                    bag_selected =
                        move_grid_cursor(bag_selected, c.inventory.columns, c.inventory.rows, key);
                }
                InventoryFocus::Character => {
                    character_selected = move_equipment_cursor(character_selected, key);
                }
            },
            'o' | 'O' => {
                let should_save = !c.inventory.is_empty();
                let result = sort_inventory(c);
                message = result.message;
                if should_save {
                    append_autosave_status(c, &mut message);
                }
            }
            'x' | 'X' => match focus {
                InventoryFocus::Bag => {
                    let result = drop_selected_inventory_item(c, bag_selected);
                    message = result.message;
                    if result.spent_turn {
                        append_autosave_status(c, &mut message);
                    }
                    if c.active_dungeon.is_some() && result.spent_turn {
                        log_inventory_action(c, &message);
                        return Ok(InventoryScreenExit::TurnSpent);
                    }
                }
                InventoryFocus::Character => {
                    message = "Switch to Bag to drop carried items.".to_string();
                }
            },
            '\n' => match focus {
                InventoryFocus::Bag => {
                    let result = finish_inventory_enter_action(c, bag_selected)?;
                    message = result.message;
                    match result.flow {
                        InventoryMenuFlow::StayOpen => {}
                        InventoryMenuFlow::ReturnedToTown => {
                            return Ok(InventoryScreenExit::ReturnedToTown);
                        }
                        InventoryMenuFlow::HardcoreDeath => {
                            return Ok(InventoryScreenExit::HardcoreDeath);
                        }
                    }
                }
                InventoryFocus::Character => {
                    message = "Switch to Bag to equip or use items.".to_string();
                }
            },
            _ => message = "Unknown inventory command.".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InventoryFocus {
    Bag,
    Character,
}

impl InventoryFocus {
    fn toggle(self) -> Self {
        match self {
            InventoryFocus::Bag => InventoryFocus::Character,
            InventoryFocus::Character => InventoryFocus::Bag,
        }
    }
}

#[allow(dead_code)]
pub(crate) fn render_inventory_screen(
    frame: &mut Frame,
    c: &Character,
    selected: usize,
    message: &str,
) {
    render_inventory_screen_with_focus(
        frame,
        c,
        selected,
        CharacterEquipmentSlot::Weapon,
        InventoryFocus::Bag,
        message,
    );
}

pub(crate) fn render_inventory_screen_with_focus(
    frame: &mut Frame,
    c: &Character,
    bag_selected: usize,
    character_selected: CharacterEquipmentSlot,
    focus: InventoryFocus,
    message: &str,
) {
    let area = frame.area();
    let footer_height = if message.is_empty() { 3 } else { 4 };
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(footer_height),
    ])
    .split(area);
    let title = Paragraph::new(format!(
        "Inventory - Bag {} x {} - {} / {}",
        c.inventory.columns,
        c.inventory.rows,
        c.inventory.len(),
        c.inventory.capacity()
    ))
    .block(gothic_block("Inventory"));
    frame.render_widget(title, layout[0]);

    let grid_width = item_grid_render_width(&c.inventory);
    let grid_height = c.inventory.rows.saturating_add(2);
    let (grid_area, details_area, character_area) =
        if layout[1].width >= grid_width.saturating_add(64) {
            let body = Layout::horizontal([
                Constraint::Length(grid_width),
                Constraint::Min(32),
                Constraint::Min(32),
            ])
            .split(layout[1]);
            (body[0], body[1], body[2])
        } else {
            let body = Layout::vertical([
                Constraint::Length(grid_height),
                Constraint::Min(3),
                Constraint::Min(3),
            ])
            .split(layout[1]);
            (body[0], body[1], body[2])
        };
    render_item_grid(
        frame,
        &c.inventory,
        active_bag_cursor(focus, bag_selected),
        grid_area,
        "Bag",
        focus == InventoryFocus::Bag,
    );
    let details = Paragraph::new(inventory_details_lines(
        c,
        bag_selected,
        character_selected,
        focus,
    ))
    .block(gothic_block("Details"));
    frame.render_widget(details, details_area);
    let character = Paragraph::new(character_equipment_panel_lines(
        c,
        character_selected,
        focus == InventoryFocus::Character,
    ))
    .block(gothic_block_selected(
        "Character",
        focus == InventoryFocus::Character,
    ));
    frame.render_widget(character, character_area);

    render_commands_footer(frame, layout[2], footer_text(message, INVENTORY_COMMANDS));
}

pub(crate) fn render_item_grid(
    frame: &mut Frame,
    grid: &ItemGrid,
    selected: usize,
    area: Rect,
    title: &str,
    selected_container: bool,
) {
    let mut lines = Vec::new();
    for row in 0..grid.rows {
        let mut spans = Vec::new();
        for col in 0..grid.columns {
            let index = usize::from(row) * usize::from(grid.columns) + usize::from(col);
            spans.extend(inventory_cell_spans(grid, index, index == selected));
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(
        Paragraph::new(lines).block(gothic_block_selected(title, selected_container)),
        area,
    );
}

pub(crate) fn item_grid_render_width(grid: &ItemGrid) -> u16 {
    grid.columns.saturating_mul(4).saturating_add(2)
}

fn active_bag_cursor(focus: InventoryFocus, selected: usize) -> usize {
    if focus == InventoryFocus::Bag {
        selected
    } else {
        usize::MAX
    }
}

#[allow(dead_code)]
pub(crate) fn inventory_cell_spans(
    grid: &ItemGrid,
    index: usize,
    selected: bool,
) -> Vec<Span<'static>> {
    let label = inventory_cell_label(grid, index);
    let focus_style = selected_cursor_style();

    let Some(item) = grid.get(index) else {
        let style = if selected { focus_style } else { body_style() };
        return vec![
            Span::styled("[", style),
            Span::styled(label, style),
            Span::styled("]", style),
        ];
    };

    let outline_style = Style::default().fg(rarity_color(&item.rarity));
    let label_style = if selected { focus_style } else { body_style() };
    vec![
        Span::styled("[", outline_style),
        Span::styled(label, label_style),
        Span::styled("]", outline_style),
    ]
}

fn inventory_details_lines(
    c: &Character,
    bag_selected: usize,
    character_selected: CharacterEquipmentSlot,
    focus: InventoryFocus,
) -> Vec<Line<'static>> {
    match focus {
        InventoryFocus::Bag => {
            let selected_item = c.inventory.get(bag_selected);
            let mut lines = selected_item_detail_lines(c, &c.inventory, "Bag", selected_item);
            lines.push(Line::from(""));
            lines.extend(selected_item_equipped_comparison_lines(c, selected_item));
            lines
        }
        InventoryFocus::Character => selected_equipped_item_detail_lines(c, character_selected),
    }
}

fn selected_equipped_item_detail_lines(
    c: &Character,
    slot: CharacterEquipmentSlot,
) -> Vec<Line<'static>> {
    let item = character_equipment_item(c, slot);
    let slot_label = equipment_slot_label(slot);
    let mut lines = vec![Line::styled(
        format!("Selected {slot_label}"),
        title_style(),
    )];
    if is_empty_equipment_slot(item) {
        lines.push(Line::styled(NOTHING_EQUIPPED_TEXT, muted_style()));
    } else {
        lines.extend(item_detail_lines(item));
    }
    lines
}

fn character_equipment_panel_lines(
    c: &Character,
    selected: CharacterEquipmentSlot,
    active: bool,
) -> Vec<Line<'static>> {
    CHARACTER_EQUIPMENT_SLOTS
        .iter()
        .map(|slot| character_equipment_slot_line(c, *slot, selected, active))
        .collect()
}

fn character_equipment_slot_line(
    c: &Character,
    slot: CharacterEquipmentSlot,
    selected: CharacterEquipmentSlot,
    active: bool,
) -> Line<'static> {
    let item = character_equipment_item(c, slot);
    let selected = active && slot == selected;
    let label_style = if selected {
        selected_cursor_style()
    } else {
        title_style()
    };
    let item_style = if selected {
        selected_cursor_style()
    } else if is_empty_equipment_slot(item) {
        muted_style()
    } else {
        Style::default().fg(rarity_color(&item.rarity))
    };

    Line::from(vec![
        Span::raw(" ".repeat(equipment_slot_indent(slot))),
        Span::styled(format!("{}: ", equipment_slot_label(slot)), label_style),
        Span::styled(equipped_display_name(item), item_style),
    ])
}

fn equipment_slot_indent(slot: CharacterEquipmentSlot) -> usize {
    match slot {
        CharacterEquipmentSlot::Weapon | CharacterEquipmentSlot::Gloves => 0,
        CharacterEquipmentSlot::Helm
        | CharacterEquipmentSlot::Amulet
        | CharacterEquipmentSlot::Armor
        | CharacterEquipmentSlot::Belt
        | CharacterEquipmentSlot::Boots => 8,
        CharacterEquipmentSlot::Shield
        | CharacterEquipmentSlot::Ring1
        | CharacterEquipmentSlot::Ring2 => 16,
    }
}

fn equipment_slot_label(slot: CharacterEquipmentSlot) -> &'static str {
    match slot {
        CharacterEquipmentSlot::Helm => "Helm",
        CharacterEquipmentSlot::Amulet => "Amulet",
        CharacterEquipmentSlot::Weapon => "Weapon",
        CharacterEquipmentSlot::Armor => "Armor",
        CharacterEquipmentSlot::Shield => "Shield",
        CharacterEquipmentSlot::Gloves => "Gloves",
        CharacterEquipmentSlot::Belt => "Belt",
        CharacterEquipmentSlot::Ring1 => "Ring 1",
        CharacterEquipmentSlot::Ring2 => "Ring 2",
        CharacterEquipmentSlot::Boots => "Boots",
    }
}

fn character_equipment_item(c: &Character, slot: CharacterEquipmentSlot) -> &Item {
    match slot {
        CharacterEquipmentSlot::Helm => &c.equipped_helm,
        CharacterEquipmentSlot::Amulet => &c.equipped_amulet,
        CharacterEquipmentSlot::Weapon => &c.equipped_weapon,
        CharacterEquipmentSlot::Armor => &c.equipped_armor,
        CharacterEquipmentSlot::Shield => &c.equipped_shield,
        CharacterEquipmentSlot::Gloves => &c.equipped_gloves,
        CharacterEquipmentSlot::Belt => &c.equipped_belt,
        CharacterEquipmentSlot::Ring1 => &c.equipped_ring1,
        CharacterEquipmentSlot::Ring2 => &c.equipped_ring2,
        CharacterEquipmentSlot::Boots => &c.equipped_boots,
    }
}

#[cfg(test)]
pub(crate) fn inventory_screen_text_for_test(
    c: &Character,
    bag_selected: usize,
    character_selected: CharacterEquipmentSlot,
    focus: InventoryFocus,
    message: &str,
) -> Vec<String> {
    let mut lines = vec![format!(
        "Inventory - Bag {} x {} - {} / {}",
        c.inventory.columns,
        c.inventory.rows,
        c.inventory.len(),
        c.inventory.capacity()
    )];
    lines.push(if focus == InventoryFocus::Bag {
        "Bag *".to_string()
    } else {
        "Bag".to_string()
    });
    for row in 0..c.inventory.rows {
        let mut line = String::new();
        for col in 0..c.inventory.columns {
            let index = usize::from(row) * usize::from(c.inventory.columns) + usize::from(col);
            line.push_str(&format!("[{}] ", inventory_cell_label(&c.inventory, index)));
        }
        lines.push(line);
    }
    lines.push("Details".to_string());
    lines.extend(
        inventory_details_lines(c, bag_selected, character_selected, focus)
            .into_iter()
            .map(line_to_plain_text),
    );
    lines.push(if focus == InventoryFocus::Character {
        "Character *".to_string()
    } else {
        "Character".to_string()
    });
    lines.extend(
        character_equipment_panel_lines(c, character_selected, focus == InventoryFocus::Character)
            .into_iter()
            .map(line_to_plain_text),
    );
    if !message.is_empty() {
        lines.push(message.to_string());
    }
    lines.push(INVENTORY_COMMANDS.to_string());
    lines
}

#[cfg(test)]
fn line_to_plain_text(line: Line<'static>) -> String {
    line.spans
        .into_iter()
        .map(|span| span.content.to_string())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InventoryScreenExit {
    NoTurn,
    TurnSpent,
    ReturnedToTown,
    HardcoreDeath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InventoryMenuFlow {
    StayOpen,
    ReturnedToTown,
    HardcoreDeath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InventoryMenuCommandResult {
    pub(crate) message: String,
    pub(crate) flow: InventoryMenuFlow,
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

pub(crate) fn finish_inventory_enter_action(
    c: &mut Character,
    index: usize,
) -> Result<InventoryMenuCommandResult> {
    finish_inventory_enter_action_with(c, index, |c, before_floor, before_log_len| {
        finish_dungeon_action(c, before_floor, before_log_len, true, "Inventory")
    })
}

#[cfg(test)]
pub(crate) fn finish_inventory_enter_action_for_test(
    c: &mut Character,
    index: usize,
) -> Result<InventoryMenuCommandResult> {
    finish_inventory_enter_action_with(c, index, |c, before_floor, before_log_len| {
        finish_dungeon_action_with(c, before_floor, before_log_len, true, "Inventory", |_| {
            Ok(())
        })
    })
}

fn finish_inventory_enter_action_with(
    c: &mut Character,
    index: usize,
    finish_dungeon_turn: impl FnOnce(&mut Character, Option<u32>, usize) -> Result<DeathOutcome>,
) -> Result<InventoryMenuCommandResult> {
    let before_floor = current_dungeon_floor(c);
    let before_log_len = current_dungeon_log_len(c);
    let result = equip_or_use_inventory_item(c, index);
    let mut message = result.message;
    if !result.spent_turn {
        return Ok(InventoryMenuCommandResult {
            message,
            flow: InventoryMenuFlow::StayOpen,
        });
    }

    if c.active_dungeon.is_some() {
        log_inventory_action(c, &message);
        let flow = match finish_dungeon_turn(c, before_floor, before_log_len)? {
            DeathOutcome::Alive => InventoryMenuFlow::StayOpen,
            DeathOutcome::SoftcoreRevived => InventoryMenuFlow::ReturnedToTown,
            DeathOutcome::HardcoreDeleted => InventoryMenuFlow::HardcoreDeath,
        };
        return Ok(InventoryMenuCommandResult { message, flow });
    }

    append_autosave_status(c, &mut message);
    Ok(InventoryMenuCommandResult {
        message,
        flow: InventoryMenuFlow::StayOpen,
    })
}

pub(crate) fn log_inventory_action(c: &mut Character, message: &str) {
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(&mut d.log, LogKind::Info, message);
    }
}

pub(crate) fn visible_rows_for_area(area: Rect, reserved_rows: u16) -> usize {
    area.height.saturating_sub(reserved_rows).max(5) as usize
}

pub(crate) fn visible_rows_for_lines_screen(
    area: Rect,
    message: &str,
    reserved_body_rows: u16,
) -> usize {
    let footer_height = if message.is_empty() { 3 } else { 4 };
    let body_inner_height = area
        .height
        .saturating_sub(3)
        .saturating_sub(footer_height)
        .saturating_sub(2);
    body_inner_height.saturating_sub(reserved_body_rows).max(1) as usize
}

pub(crate) fn visible_rows_for_wrapped_lines_screen(
    area: Rect,
    message: &str,
    reserved_body_rows: u16,
    row_height: u16,
) -> usize {
    let row_height = row_height.max(1) as usize;
    visible_rows_for_lines_screen(area, message, reserved_body_rows)
        .saturating_div(row_height)
        .max(1)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CharacterEquipmentSlot {
    Helm,
    Amulet,
    Weapon,
    Armor,
    Shield,
    Gloves,
    Belt,
    Ring1,
    Ring2,
    Boots,
}

const CHARACTER_EQUIPMENT_SLOTS: [CharacterEquipmentSlot; 10] = [
    CharacterEquipmentSlot::Helm,
    CharacterEquipmentSlot::Amulet,
    CharacterEquipmentSlot::Weapon,
    CharacterEquipmentSlot::Armor,
    CharacterEquipmentSlot::Shield,
    CharacterEquipmentSlot::Gloves,
    CharacterEquipmentSlot::Belt,
    CharacterEquipmentSlot::Ring1,
    CharacterEquipmentSlot::Ring2,
    CharacterEquipmentSlot::Boots,
];

fn equipment_slot_position(slot: CharacterEquipmentSlot) -> (i16, i16) {
    match slot {
        CharacterEquipmentSlot::Helm => (1, 0),
        CharacterEquipmentSlot::Amulet => (1, 1),
        CharacterEquipmentSlot::Weapon => (0, 2),
        CharacterEquipmentSlot::Armor => (1, 2),
        CharacterEquipmentSlot::Shield => (2, 2),
        CharacterEquipmentSlot::Gloves => (0, 3),
        CharacterEquipmentSlot::Belt => (1, 3),
        CharacterEquipmentSlot::Ring1 => (2, 3),
        CharacterEquipmentSlot::Ring2 => (2, 4),
        CharacterEquipmentSlot::Boots => (1, 5),
    }
}

#[allow(dead_code)]
pub(crate) fn move_equipment_cursor(
    selected: CharacterEquipmentSlot,
    key: char,
) -> CharacterEquipmentSlot {
    let (selected_x, selected_y) = equipment_slot_position(selected);
    let candidate_score = |slot: &CharacterEquipmentSlot| {
        let (x, y) = equipment_slot_position(*slot);
        match key {
            'w' | 'W' if y < selected_y => Some((selected_y - y, (selected_x - x).abs())),
            's' | 'S' if y > selected_y => Some((y - selected_y, (selected_x - x).abs())),
            'a' | 'A' if x < selected_x => Some((selected_x - x, (selected_y - y).abs())),
            'd' | 'D' if x > selected_x => Some((x - selected_x, (selected_y - y).abs())),
            _ => None,
        }
    };

    CHARACTER_EQUIPMENT_SLOTS
        .iter()
        .filter_map(|slot| candidate_score(slot).map(|score| (score, *slot)))
        .min_by_key(|(score, _)| *score)
        .map(|(_, slot)| slot)
        .unwrap_or(selected)
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
        ItemKind::Helm => "H",
        ItemKind::Gloves => "G",
        ItemKind::Boots => "B",
        ItemKind::Belt => "T",
        ItemKind::Amulet => "U",
        ItemKind::Ring => "R",
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
    _c: &Character,
    grid: &ItemGrid,
    grid_label: &str,
    item: Option<&Item>,
) -> Vec<Line<'static>> {
    let Some(item) = item else {
        return vec![
            Line::styled("Empty cell", muted_style()),
            Line::from(format!(
                "{}: {}/{}",
                grid_label,
                grid.len(),
                grid.capacity()
            )),
        ];
    };
    item_detail_lines(item)
}

fn item_detail_lines(item: &Item) -> Vec<Line<'static>> {
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
        ItemKind::Gem => lines.push(Line::from(strip_ansi_codes(&gem_summary(item)))),
    }
    lines
}

pub(crate) fn sort_inventory(c: &mut Character) -> InventoryActionResult {
    if c.inventory.is_empty() {
        return InventoryActionResult::free("Inventory is empty.");
    }

    c.inventory.items.sort_by(compare_inventory_items);
    InventoryActionResult::free("Inventory sorted.")
}

fn compare_inventory_items(a: &Item, b: &Item) -> std::cmp::Ordering {
    inventory_item_kind_rank(a.kind)
        .cmp(&inventory_item_kind_rank(b.kind))
        .then_with(|| inventory_rarity_rank(&a.rarity).cmp(&inventory_rarity_rank(&b.rarity)))
        .then_with(|| b.item_level.cmp(&a.item_level))
        .then_with(|| b.value.cmp(&a.value))
        .then_with(|| a.name.cmp(&b.name))
}

fn inventory_item_kind_rank(kind: ItemKind) -> u8 {
    match kind {
        ItemKind::HealthPotion => 0,
        ItemKind::ManaPotion => 1,
        ItemKind::Weapon => 2,
        ItemKind::Armor => 3,
        ItemKind::Shield => 4,
        ItemKind::Helm => 5,
        ItemKind::Gloves => 6,
        ItemKind::Boots => 7,
        ItemKind::Belt => 8,
        ItemKind::Amulet => 9,
        ItemKind::Ring => 10,
        ItemKind::Gem => 11,
    }
}

fn inventory_rarity_rank(rarity: &Rarity) -> u8 {
    match rarity {
        Rarity::Rare => 0,
        Rarity::Magic => 1,
        Rarity::Common => 2,
    }
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
        let Some(d) = c.active_dungeon.as_mut() else {
            return InventoryActionResult::free("Drop items only inside a dungeon.");
        };
        let item = c.inventory.remove(index);
        let name = item.name.clone();
        add_ground_item(d, d.player_x, d.player_y, item);
        let message = format!("Dropped {name} on the ground.");
        InventoryActionResult::spent(message)
    }
}

pub(crate) fn item_level_text(item: &Item) -> String {
    if matches!(
        item.kind,
        ItemKind::Weapon
            | ItemKind::Armor
            | ItemKind::Shield
            | ItemKind::Helm
            | ItemKind::Gloves
            | ItemKind::Boots
            | ItemKind::Belt
            | ItemKind::Amulet
            | ItemKind::Ring
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
    if is_empty_equipment_slot(item) {
        return NOTHING_EQUIPPED_TEXT.to_string();
    }
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
        ItemKind::Armor
        | ItemKind::Shield
        | ItemKind::Helm
        | ItemKind::Gloves
        | ItemKind::Boots
        | ItemKind::Belt
        | ItemKind::Amulet
        | ItemKind::Ring => {
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

fn is_equipment_kind(kind: ItemKind) -> bool {
    matches!(
        kind,
        ItemKind::Weapon
            | ItemKind::Armor
            | ItemKind::Shield
            | ItemKind::Helm
            | ItemKind::Gloves
            | ItemKind::Boots
            | ItemKind::Belt
            | ItemKind::Amulet
            | ItemKind::Ring
    )
}

fn is_empty_ring(item: &Item) -> bool {
    item.kind == ItemKind::Ring && item.name == "Empty Ring"
}

fn return_replaced_equipment_to_inventory(
    inventory: &mut ItemGrid,
    index: usize,
    old: Item,
    slot: &str,
) {
    if is_empty_equipment_slot(&old) {
        return;
    }
    assert!(
        inventory.insert(index, old),
        "ItemGrid invariant broken: equipping {slot} should free inventory capacity for old gear"
    );
}

fn equipped_comparison_target(c: &Character, kind: ItemKind) -> Option<(&'static str, &Item)> {
    match kind {
        ItemKind::Weapon => Some(("Weapon", &c.equipped_weapon)),
        ItemKind::Armor => Some(("Armor", &c.equipped_armor)),
        ItemKind::Shield => Some(("Shield", &c.equipped_shield)),
        ItemKind::Helm => Some(("Helm", &c.equipped_helm)),
        ItemKind::Gloves => Some(("Gloves", &c.equipped_gloves)),
        ItemKind::Boots => Some(("Boots", &c.equipped_boots)),
        ItemKind::Belt => Some(("Belt", &c.equipped_belt)),
        ItemKind::Amulet => Some(("Amulet", &c.equipped_amulet)),
        ItemKind::Ring if is_empty_ring(&c.equipped_ring1) => Some(("Ring 1", &c.equipped_ring1)),
        ItemKind::Ring if is_empty_ring(&c.equipped_ring2) => Some(("Ring 2", &c.equipped_ring2)),
        ItemKind::Ring => Some(("Ring 1", &c.equipped_ring1)),
        ItemKind::HealthPotion | ItemKind::ManaPotion | ItemKind::Gem => None,
    }
}

pub(crate) fn item_comparison(c: &Character, item: &Item) -> Option<String> {
    let (_, equipped) = equipped_comparison_target(c, item.kind)?;
    let comparison = if item.kind == ItemKind::Weapon {
        let cur_avg = equipped.damage_min + equipped.damage_max;
        let new_avg = item.damage_min + item.damage_max;
        format!(
            "Compare: {}  {}",
            format_delta("damage", new_avg - cur_avg),
            format_crit_delta(item.crit_chance as i32 - equipped.crit_chance as i32)
        )
    } else {
        format!(
            "Compare: {}  {}  {}",
            format_delta("armor", item.armor - equipped.armor),
            format_delta("dodge", item.dodge - equipped.dodge),
            format_delta("speed", item.speed - equipped.speed)
        )
    };
    if let Some(requirements) = unmet_requirements_message(c, item) {
        Some(format!("{comparison}  {RED}LOCKED:{RESET} {requirements}"))
    } else {
        Some(comparison)
    }
}

pub(crate) fn selected_item_equipped_comparison_lines(
    c: &Character,
    item: Option<&Item>,
) -> Vec<Line<'static>> {
    let Some(item) = item else {
        return vec![Line::styled("Select gear to compare.", muted_style())];
    };

    let Some((slot_label, equipped)) = equipped_comparison_target(c, item.kind) else {
        return vec![Line::styled(
            "No equipped slot for this item.",
            muted_style(),
        )];
    };

    let delta_spans = if item.kind == ItemKind::Weapon {
        let cur_avg = equipped.damage_min + equipped.damage_max;
        let new_avg = item.damage_min + item.damage_max;
        vec![
            stat_delta_span("damage", new_avg - cur_avg),
            crit_delta_span(item.crit_chance as i32 - equipped.crit_chance as i32),
        ]
    } else {
        vec![
            stat_delta_span("armor", item.armor - equipped.armor),
            stat_delta_span("dodge", item.dodge - equipped.dodge),
            stat_delta_span("speed", item.speed - equipped.speed),
        ]
    };

    let mut lines = vec![
        Line::from(format!(
            "Equipped {slot_label}: {}",
            equipped_comparison_name(equipped)
        )),
        gear_stat_line(equipped),
        Line::from(""),
        delta_line(delta_spans),
    ];
    if let Some(requirements) = unmet_requirements_message(c, item) {
        lines.push(Line::styled(
            format!("Cannot equip: {}", strip_ansi_codes(&requirements)),
            Style::default().fg(Color::Red),
        ));
    }
    lines
}

fn gear_stat_line(item: &Item) -> Line<'static> {
    match item.kind {
        ItemKind::Weapon => Line::from(format!(
            "Damage {}-{} | crit {}%",
            item.damage_min, item.damage_max, item.crit_chance
        )),
        ItemKind::Armor
        | ItemKind::Shield
        | ItemKind::Helm
        | ItemKind::Gloves
        | ItemKind::Boots
        | ItemKind::Belt
        | ItemKind::Amulet
        | ItemKind::Ring => Line::from(format!(
            "Armor {} | dodge {} | speed {}",
            item.armor, item.dodge, item.speed
        )),
        ItemKind::HealthPotion | ItemKind::ManaPotion | ItemKind::Gem => Line::from(""),
    }
}

fn delta_line(delta_spans: Vec<Span<'static>>) -> Line<'static> {
    let mut spans = vec![Span::raw("Delta: ")];
    for (index, span) in delta_spans.into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(span);
    }
    Line::from(spans)
}

fn stat_delta_span(label: &'static str, delta: i32) -> Span<'static> {
    let text = if delta >= 0 {
        format!("+{delta} {label}")
    } else {
        format!("{delta} {label}")
    };
    Span::styled(text, delta_style(delta))
}

fn crit_delta_span(delta: i32) -> Span<'static> {
    let text = if delta >= 0 {
        format!("crit +{delta}")
    } else {
        format!("crit {delta}")
    };
    Span::styled(text, delta_style(delta))
}

fn delta_style(delta: i32) -> Style {
    if delta > 0 {
        Style::default().fg(Color::Green)
    } else if delta < 0 {
        Style::default().fg(Color::Red)
    } else {
        body_style()
    }
}

fn item_base_name(name: &str) -> &str {
    name.split_once(" (").map(|(base, _)| base).unwrap_or(name)
}

fn equipped_comparison_name(item: &Item) -> String {
    if is_empty_equipment_slot(item) {
        NOTHING_EQUIPPED_TEXT.to_string()
    } else {
        item_base_name(&item.name).to_string()
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
    can_equip_item_for_class(c, item)
        && c.strength >= item.required_strength
        && c.dexterity >= item.required_dexterity
        && c.intelligence >= item.required_intelligence
}

pub(crate) fn unmet_requirements_message(c: &Character, item: &Item) -> Option<String> {
    if can_equip_item(c, item) {
        return None;
    }
    if !can_equip_item_for_class(c, item) {
        let message = match c.class {
            CharacterClass::Rogue => "Rogue cannot equip non-buckler shields.",
            CharacterClass::Sorceress => {
                "Sorceress can equip wands and focuses only in weapon/offhand slots."
            }
            CharacterClass::Warrior => "Class cannot equip that item.",
        };
        return Some(message.to_string());
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

fn can_equip_item_for_class(c: &Character, item: &Item) -> bool {
    match c.class {
        CharacterClass::Warrior => true,
        CharacterClass::Rogue => {
            item.kind != ItemKind::Shield
                || item.name == "Empty Offhand"
                || item.name.contains("Buckler")
        }
        CharacterClass::Sorceress => match item.kind {
            ItemKind::Weapon => item.name.contains("Wand"),
            ItemKind::Shield => item.name == "Empty Offhand" || item.name.contains("Focus"),
            _ => true,
        },
    }
}

pub(crate) fn equip_or_use_inventory_item(
    c: &mut Character,
    index: usize,
) -> InventoryActionResult {
    if c.inventory.get(index).is_none() {
        return InventoryActionResult::free("No item in that slot.");
    }
    let selected = c.inventory.remove(index);
    if is_equipment_kind(selected.kind) {
        if let Some(message) = unmet_requirements_message(c, &selected) {
            c.inventory.insert(index, selected);
            return InventoryActionResult::free(message);
        }
    }
    match selected.kind {
        ItemKind::Weapon => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_weapon, selected);
            return_replaced_equipment_to_inventory(&mut c.inventory, index, old, "weapon");
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::Armor => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_armor, selected);
            return_replaced_equipment_to_inventory(&mut c.inventory, index, old, "armor");
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::Shield => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_shield, selected);
            return_replaced_equipment_to_inventory(&mut c.inventory, index, old, "shield");
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::Helm => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_helm, selected);
            return_replaced_equipment_to_inventory(&mut c.inventory, index, old, "helm");
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::Gloves => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_gloves, selected);
            return_replaced_equipment_to_inventory(&mut c.inventory, index, old, "gloves");
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::Boots => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_boots, selected);
            return_replaced_equipment_to_inventory(&mut c.inventory, index, old, "boots");
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::Belt => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_belt, selected);
            return_replaced_equipment_to_inventory(&mut c.inventory, index, old, "belt");
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::Amulet => {
            let name = selected.name.clone();
            let old = std::mem::replace(&mut c.equipped_amulet, selected);
            return_replaced_equipment_to_inventory(&mut c.inventory, index, old, "amulet");
            clamp_current_resources(c);
            InventoryActionResult::spent(format!("Equipped {name}."))
        }
        ItemKind::Ring => {
            let name = selected.name.clone();
            let old = if is_empty_ring(&c.equipped_ring1) || !is_empty_ring(&c.equipped_ring2) {
                std::mem::replace(&mut c.equipped_ring1, selected)
            } else {
                std::mem::replace(&mut c.equipped_ring2, selected)
            };
            return_replaced_equipment_to_inventory(&mut c.inventory, index, old, "ring");
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
            if c.class == CharacterClass::Rogue {
                c.inventory.insert(index, selected);
                return InventoryActionResult::free(
                    "Rogue uses Energy and cannot use mana potions.",
                );
            }
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
