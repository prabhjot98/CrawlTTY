use crate::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub(crate) fn quest_giver(c: &mut Character) -> String {
    if c.act2_completed {
        "Warden Mara: Two curses broken. Campaign complete for now. More acts may come later."
            .to_string()
    } else if c.glass_tyrant_defeated {
        c.gold += 250;
        c.unspent_skills += 2;
        c.unspent_attributes += 3;
        c.hp = c.max_hp();
        c.mana = c.max_mana();
        c.act2_completed = true;
        "Quest complete: Shatter the Glass Tyrant. Reward: 250 gold, +2 skill points, +3 attributes, full heal."
            .to_string()
    } else if c.act1_completed {
        format!(
            "Quest accepted: Shatter the Glass Tyrant. Defeat it on floor {FINAL_FLOOR} of the Glass Wastes."
        )
    } else if c.bellkeeper_defeated {
        c.gold += 100;
        c.unspent_skills += 1;
        c.hp = c.max_hp();
        c.mana = c.max_mana();
        c.act1_completed = true;
        "Quest complete: Silence the Bellkeeper. Reward: 100 gold, +1 skill point, full heal, Act II unlocked."
            .to_string()
    } else {
        format!(
            "Quest accepted: Silence the Bellkeeper. Defeat it on floor {ACT1_FLOORS} of the Hollow Crypts."
        )
    }
}

pub(crate) const TOWN_FULL_HEAL_MESSAGE: &str = "You were fully healed.";

pub(crate) fn full_heal(c: &mut Character) {
    c.hp = c.max_hp();
    c.mana = c.max_mana();
}

pub(crate) fn append_pending_town_message(c: &mut Character, message: &str) {
    if c.pending_town_message.is_empty() {
        c.pending_town_message = message.to_string();
    } else {
        c.pending_town_message.push(' ');
        c.pending_town_message.push_str(message);
    }
}

pub(crate) fn full_heal_on_town_return(c: &mut Character) {
    full_heal(c);
    if !c.pending_town_message.contains(TOWN_FULL_HEAL_MESSAGE) {
        append_pending_town_message(c, TOWN_FULL_HEAL_MESSAGE);
    }
}

pub(crate) fn merchant(c: &mut Character) {
    let mut selected = 0usize;
    let mut message = String::new();
    let options = ["Sell items"];
    loop {
        clamp_selection(&mut selected, options.len());
        clear_screen();
        println!("{BOLD}{YELLOW}Merchant{RESET} - {}", gold_text(c.gold));
        println!("Gold funds town projects. Sell unwanted items here.");
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();
        println!("{BOLD}Services{RESET}");
        for (i, option) in options.iter().enumerate() {
            let marker = if i == selected {
                format!("{GREEN}>{RESET}")
            } else {
                " ".to_string()
            };
            println!("{marker} {option}");
        }
        println!();
        print_inventory_preview(c, inventory_visible_rows(12));
        print_footer(&[&format!(
            "{BOLD}Merchant:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=choose  {RED}Esc{RESET}=back"
        )]);
        let Some(key) = read_key_char_nav_or_message(&mut message) else {
            break;
        };
        match key {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < options.len() {
                    selected += 1;
                }
            }
            '\n' => match selected {
                0 => sell_item_screen(c),
                _ => {}
            },
            _ => message = "Unknown merchant command.".to_string(),
        }
    }
}

pub(crate) fn blacksmith(c: &mut Character) {
    let mut selected = 0usize;
    let mut message = String::new();
    let options = [
        "Salvage carried gear for shards",
        "Sharpen equipped weapon",
        "Reinforce equipped armor",
        "Brace equipped shield",
        "Manage sockets",
    ];
    loop {
        clamp_selection(&mut selected, options.len());
        clear_screen();
        println!("{BOLD}{WHITE}Blacksmith{RESET} - {}", gold_text(c.gold));
        println!(
            "{BOLD}Shards:{RESET} {}  {}  {}",
            shard_text("weapon", c.weapon_shards),
            shard_text("armor", c.armor_shards),
            shard_text("shield", c.shield_shards)
        );
        println!(
            "Town projects unlock smith services. Salvage gear into type shards, then spend shards to upgrade equipped gear."
        );
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();
        println!("{BOLD}Services{RESET}");
        for (i, option) in options.iter().enumerate() {
            let marker = if i == selected {
                format!("{GREEN}>{RESET}")
            } else {
                " ".to_string()
            };
            println!("{marker} {option}");
        }
        println!();
        println!("{BOLD}Equipped{RESET}");
        println!("Weapon: {}", item_summary(&c.equipped_weapon));
        println!("Armor : {}", item_summary(&c.equipped_armor));
        println!("Shield: {}", item_summary(&c.equipped_shield));
        print_footer(&[&format!(
            "{BOLD}Blacksmith:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=choose  {RED}Esc{RESET}=back"
        )]);
        let Some(key) = read_key_char_nav_or_message(&mut message) else {
            break;
        };
        match key {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < options.len() {
                    selected += 1;
                }
            }
            '\n' => match selected {
                0 => salvage_screen(c),
                1 => {
                    message = upgrade_equipped_message(c, UpgradeSlot::Weapon);
                    append_autosave_status(c, &mut message);
                }
                2 => {
                    message = upgrade_equipped_message(c, UpgradeSlot::Armor);
                    append_autosave_status(c, &mut message);
                }
                3 => {
                    message = upgrade_equipped_message(c, UpgradeSlot::Shield);
                    append_autosave_status(c, &mut message);
                }
                4 => {
                    if has_completed_project(c, TownProject::SocketBench) {
                        socket_bench_screen(c);
                    } else {
                        message =
                            "Complete the Socket Bench project before socketing gems.".to_string();
                    }
                }
                _ => {}
            },
            _ => message = "Unknown blacksmith command.".to_string(),
        }
    }
}

pub(crate) fn town_projects_menu(c: &mut Character) {
    let mut selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_selection(&mut selected, TOWN_PROJECTS.len());
        clear_screen();
        println!("{BOLD}{CYAN}Town Projects{RESET} - {}", gold_text(c.gold));
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();
        for (i, definition) in TOWN_PROJECTS.iter().enumerate() {
            let marker = if i == selected {
                format!("{GREEN}>{RESET}")
            } else {
                " ".to_string()
            };
            println!("{marker} {}", town_project_row_text(c, definition.project));
        }
        println!();
        let selected_project = TOWN_PROJECTS[selected].project;
        println!(
            "{BOLD}Selected:{RESET} {}",
            town_project_definition(selected_project).name
        );
        println!("{}", town_project_definition(selected_project).benefit);
        print_footer(&[&format!(
            "{BOLD}Projects:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=fund project  {RED}Esc{RESET}=back"
        )]);
        let Some(key) = read_key_char_nav_or_message(&mut message) else {
            break;
        };
        match key {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < TOWN_PROJECTS.len() {
                    selected += 1;
                }
            }
            '\n' => {
                message = complete_town_project(c, TOWN_PROJECTS[selected].project);
                append_autosave_status(c, &mut message);
            }
            _ => message = "Unknown projects command.".to_string(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum UpgradeSlot {
    Weapon,
    Armor,
    Shield,
}

pub(crate) fn salvage_screen(c: &mut Character) {
    let mut selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_selection(&mut selected, c.inventory.len());
        clear_screen();
        println!("{BOLD}{WHITE}Salvage Gear{RESET}");
        println!(
            "{BOLD}Shards:{RESET} {}  {}  {}",
            shard_text("weapon", c.weapon_shards),
            shard_text("armor", c.armor_shards),
            shard_text("shield", c.shield_shards)
        );
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        if c.inventory.is_empty() {
            println!("Inventory is empty.");
        } else {
            print_inventory_list(c, selected, inventory_visible_rows(8));
            println!();
            println!("Selected: {}", item_summary(&c.inventory[selected]));
            if let Some(kind) = shard_kind(&c.inventory[selected]) {
                println!(
                    "Salvage yield: {} {} shard(s)",
                    salvage_shard_yield(c, &c.inventory[selected]),
                    shard_name(kind)
                );
            } else {
                println!("Consumables cannot be salvaged.");
            }
        }
        print_footer(&[&format!(
            "{BOLD}Salvage:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=salvage  {RED}Esc{RESET}=back"
        )]);
        let Some(key) = read_key_char_nav_or_message(&mut message) else {
            break;
        };
        match key {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < c.inventory.len() {
                    selected += 1;
                }
            }
            '\n' => {
                message = salvage_inventory_item(c, selected);
                append_autosave_status(c, &mut message);
            }
            _ => message = "Unknown salvage command.".to_string(),
        }
    }
}

pub(crate) fn salvage_inventory_item(c: &mut Character, index: usize) -> String {
    if !has_completed_project(c, TownProject::RebuildForge) {
        return "Rebuild the Forge before salvaging gear.".to_string();
    }
    if index >= c.inventory.len() {
        return "No item selected.".to_string();
    }
    let Some(kind) = shard_kind(&c.inventory[index]) else {
        return "Only weapons, armor, and shields can be salvaged.".to_string();
    };
    if c.inventory[index].sockets.iter().any(Option::is_some) {
        return "Remove socketed gems before salvaging this item.".to_string();
    }
    let item = c.inventory.remove(index);
    let amount = salvage_shard_yield(c, &item);
    add_shards(c, kind, amount);
    format!(
        "Salvaged {} into {} {} shard(s).",
        item.name,
        amount,
        shard_name(kind)
    )
}

pub(crate) fn salvage_shard_yield(c: &Character, item: &Item) -> u32 {
    let rarity_bonus = match item.rarity {
        Rarity::Common => 1,
        Rarity::Magic => 2,
        Rarity::Rare => 3,
    };
    let anvil_bonus = u32::from(has_completed_project(c, TownProject::ReinforcedAnvil));
    rarity_bonus + item.upgrade_level + anvil_bonus
}

pub(crate) fn upgrade_equipped_message(c: &mut Character, slot: UpgradeSlot) -> String {
    if !has_completed_project(c, TownProject::RebuildForge) {
        return "Rebuild the Forge before upgrading gear.".to_string();
    }
    let (cost_shards, kind, item_name) = {
        let item = equipped_item(c, slot);
        let kind = shard_kind(item).expect("equipped gear has shard kind");
        (upgrade_cost(item), kind, item.name.clone())
    };
    if shard_count(c, kind) < cost_shards {
        return format!(
            "Need {} {} shard(s) to upgrade {}.",
            cost_shards,
            shard_name(kind),
            item_name
        );
    }
    spend_shards(c, kind, cost_shards);
    let item = equipped_item_mut(c, slot);
    upgrade_item(item);
    format!("Upgraded {} to +{}.", item.name, item.upgrade_level)
}

pub(crate) fn upgrade_cost(item: &Item) -> u32 {
    (item.upgrade_level + 1) * 2
}

pub(crate) fn upgrade_item(item: &mut Item) {
    item.upgrade_level += 1;
    item.value += 10 * item.upgrade_level;
    match item.kind {
        ItemKind::Weapon => {
            item.damage_min += 1;
            item.damage_max += 1;
        }
        ItemKind::Armor => item.armor += 1,
        ItemKind::Shield => item.armor += 1,
        ItemKind::HealthPotion | ItemKind::ManaPotion | ItemKind::Gem => {}
    }
}

pub(crate) fn equipped_item(c: &Character, slot: UpgradeSlot) -> &Item {
    match slot {
        UpgradeSlot::Weapon => &c.equipped_weapon,
        UpgradeSlot::Armor => &c.equipped_armor,
        UpgradeSlot::Shield => &c.equipped_shield,
    }
}

pub(crate) fn equipped_item_mut(c: &mut Character, slot: UpgradeSlot) -> &mut Item {
    match slot {
        UpgradeSlot::Weapon => &mut c.equipped_weapon,
        UpgradeSlot::Armor => &mut c.equipped_armor,
        UpgradeSlot::Shield => &mut c.equipped_shield,
    }
}

pub(crate) fn shard_kind(item: &Item) -> Option<ItemKind> {
    match item.kind {
        ItemKind::Weapon => Some(ItemKind::Weapon),
        ItemKind::Armor => Some(ItemKind::Armor),
        ItemKind::Shield => Some(ItemKind::Shield),
        ItemKind::HealthPotion | ItemKind::ManaPotion | ItemKind::Gem => None,
    }
}

pub(crate) fn shard_name(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Weapon => "weapon",
        ItemKind::Armor => "armor",
        ItemKind::Shield => "shield",
        ItemKind::HealthPotion | ItemKind::ManaPotion | ItemKind::Gem => "unknown",
    }
}

pub(crate) fn shard_count(c: &Character, kind: ItemKind) -> u32 {
    match kind {
        ItemKind::Weapon => c.weapon_shards,
        ItemKind::Armor => c.armor_shards,
        ItemKind::Shield => c.shield_shards,
        ItemKind::HealthPotion | ItemKind::ManaPotion | ItemKind::Gem => 0,
    }
}

pub(crate) fn add_shards(c: &mut Character, kind: ItemKind, amount: u32) {
    match kind {
        ItemKind::Weapon => c.weapon_shards += amount,
        ItemKind::Armor => c.armor_shards += amount,
        ItemKind::Shield => c.shield_shards += amount,
        ItemKind::HealthPotion | ItemKind::ManaPotion | ItemKind::Gem => {}
    }
}

pub(crate) fn spend_shards(c: &mut Character, kind: ItemKind, amount: u32) {
    match kind {
        ItemKind::Weapon => c.weapon_shards = c.weapon_shards.saturating_sub(amount),
        ItemKind::Armor => c.armor_shards = c.armor_shards.saturating_sub(amount),
        ItemKind::Shield => c.shield_shards = c.shield_shards.saturating_sub(amount),
        ItemKind::HealthPotion | ItemKind::ManaPotion | ItemKind::Gem => {}
    }
}

pub(crate) fn insert_gem_into_equipped(
    c: &mut Character,
    slot: UpgradeSlot,
    socket_index: usize,
    inventory_index: usize,
) -> String {
    insert_gem_into_target(
        c,
        SocketBenchTarget::Equipped(slot),
        socket_index,
        inventory_index,
    )
}

pub(crate) fn remove_gem_from_equipped(
    c: &mut Character,
    slot: UpgradeSlot,
    socket_index: usize,
) -> String {
    remove_gem_from_target(c, SocketBenchTarget::Equipped(slot), socket_index)
}

pub(crate) fn replace_gem_in_equipped(
    c: &mut Character,
    slot: UpgradeSlot,
    socket_index: usize,
    inventory_index: usize,
) -> String {
    replace_gem_in_target(
        c,
        SocketBenchTarget::Equipped(slot),
        socket_index,
        inventory_index,
    )
}

#[derive(Clone, Copy)]
enum SocketBenchTarget {
    Equipped(UpgradeSlot),
    Inventory(usize),
}

fn insert_gem_into_target(
    c: &mut Character,
    target: SocketBenchTarget,
    socket_index: usize,
    inventory_index: usize,
) -> String {
    if !has_completed_project(c, TownProject::SocketBench) {
        return "Complete the Socket Bench project before socketing gems.".to_string();
    }
    let Some(socket) = target_socket(c, target, socket_index) else {
        return "No socket selected.".to_string();
    };
    if socket.is_some() {
        return "That socket is already filled.".to_string();
    }
    let Ok((gem_kind, gem_tier)) = gem_metadata_from_inventory(c, inventory_index) else {
        return gem_inventory_error(c, inventory_index).to_string();
    };
    let item_name = target_item_name(c, target).unwrap_or_else(|| "item".to_string());
    c.inventory.remove(inventory_index);
    let adjusted_target = adjust_target_after_inventory_remove(target, inventory_index);
    if let Some(socket) = target_socket_mut(c, adjusted_target, socket_index) {
        *socket = Some(GemSocket::filled(gem_kind, gem_tier));
    }
    clamp_resources_to_current_max(c);
    format!(
        "Inserted {} into {}.",
        gem_socket_name(GemSocket::filled(gem_kind, gem_tier)),
        item_name
    )
}

fn remove_gem_from_target(
    c: &mut Character,
    target: SocketBenchTarget,
    socket_index: usize,
) -> String {
    if !has_completed_project(c, TownProject::SocketBench) {
        return "Complete the Socket Bench project before socketing gems.".to_string();
    }
    let Some(socket) = target_socket(c, target, socket_index) else {
        return "No socket selected.".to_string();
    };
    let Some(gem_socket) = socket else {
        return "That socket is already empty.".to_string();
    };
    if !c.inventory.has_space() {
        return "Need one free bag cell to remove socketed gem.".to_string();
    }
    let item_name = target_item_name(c, target).unwrap_or_else(|| "item".to_string());
    if let Some(socket) = target_socket_mut(c, target, socket_index) {
        *socket = None;
    }
    let added = c
        .inventory
        .push(gem_item(gem_socket.gem_kind, gem_socket.gem_tier));
    debug_assert!(added);
    clamp_resources_to_current_max(c);
    format!(
        "Removed {} from {}.",
        gem_socket_name(gem_socket),
        item_name
    )
}

fn replace_gem_in_target(
    c: &mut Character,
    target: SocketBenchTarget,
    socket_index: usize,
    inventory_index: usize,
) -> String {
    if !has_completed_project(c, TownProject::SocketBench) {
        return "Complete the Socket Bench project before socketing gems.".to_string();
    }
    let Some(socket) = target_socket(c, target, socket_index) else {
        return "No socket selected.".to_string();
    };
    let Some(old_gem) = socket else {
        return "That socket is already empty.".to_string();
    };
    let Ok((new_kind, new_tier)) = gem_metadata_from_inventory(c, inventory_index) else {
        return gem_inventory_error(c, inventory_index).to_string();
    };
    let item_name = target_item_name(c, target).unwrap_or_else(|| "item".to_string());
    c.inventory.remove(inventory_index);
    assert!(
        c.inventory
            .push(gem_item(old_gem.gem_kind, old_gem.gem_tier)),
        "ItemGrid invariant broken: replacing socketed gem should free inventory capacity for replaced gem"
    );
    let adjusted_target = adjust_target_after_inventory_remove(target, inventory_index);
    if let Some(socket) = target_socket_mut(c, adjusted_target, socket_index) {
        *socket = Some(GemSocket::filled(new_kind, new_tier));
    }
    clamp_resources_to_current_max(c);
    format!(
        "Replaced {} with {} in {}.",
        gem_socket_name(old_gem),
        gem_socket_name(GemSocket::filled(new_kind, new_tier)),
        item_name
    )
}

fn target_item(c: &Character, target: SocketBenchTarget) -> Option<&Item> {
    match target {
        SocketBenchTarget::Equipped(slot) => Some(equipped_item(c, slot)),
        SocketBenchTarget::Inventory(index) => c.inventory.get(index),
    }
}

fn target_item_mut(c: &mut Character, target: SocketBenchTarget) -> Option<&mut Item> {
    match target {
        SocketBenchTarget::Equipped(slot) => Some(equipped_item_mut(c, slot)),
        SocketBenchTarget::Inventory(index) => c.inventory.get_mut(index),
    }
}

fn target_socket(
    c: &Character,
    target: SocketBenchTarget,
    socket_index: usize,
) -> Option<Option<GemSocket>> {
    target_item(c, target)?.sockets.get(socket_index).copied()
}

fn target_socket_mut(
    c: &mut Character,
    target: SocketBenchTarget,
    socket_index: usize,
) -> Option<&mut Option<GemSocket>> {
    target_item_mut(c, target)?.sockets.get_mut(socket_index)
}

fn target_item_name(c: &Character, target: SocketBenchTarget) -> Option<String> {
    Some(target_item(c, target)?.name.clone())
}

fn adjust_target_after_inventory_remove(
    target: SocketBenchTarget,
    removed_index: usize,
) -> SocketBenchTarget {
    match target {
        SocketBenchTarget::Inventory(index) if removed_index < index => {
            SocketBenchTarget::Inventory(index - 1)
        }
        _ => target,
    }
}

fn gem_metadata_from_inventory(
    c: &Character,
    inventory_index: usize,
) -> Result<(GemKind, GemTier), &'static str> {
    let Some(item) = c.inventory.get(inventory_index) else {
        return Err("Select a gem from inventory.");
    };
    if !matches!(item.kind, ItemKind::Gem) {
        return Err("Select a gem from inventory.");
    }
    match (item.gem_kind, item.gem_tier) {
        (Some(kind), Some(tier)) => Ok((kind, tier)),
        _ => Err("Select a valid gem from inventory."),
    }
}

fn gem_inventory_error(c: &Character, inventory_index: usize) -> &'static str {
    gem_metadata_from_inventory(c, inventory_index).unwrap_err()
}

fn gem_socket_name(socket: GemSocket) -> String {
    format!(
        "{} {}",
        gem_tier_name(socket.gem_tier),
        gem_kind_name(socket.gem_kind)
    )
}

fn socket_text(socket: Option<GemSocket>) -> String {
    match socket {
        Some(socket) => gem_socket_name(socket),
        None => "Empty".to_string(),
    }
}

fn clamp_resources_to_current_max(c: &mut Character) {
    c.hp = c.hp.min(c.max_hp());
    c.mana = c.mana.min(c.max_mana());
}

fn socketed_targets(c: &Character) -> Vec<SocketBenchTarget> {
    let mut targets = Vec::new();
    if !c.equipped_weapon.sockets.is_empty() {
        targets.push(SocketBenchTarget::Equipped(UpgradeSlot::Weapon));
    }
    if !c.equipped_armor.sockets.is_empty() {
        targets.push(SocketBenchTarget::Equipped(UpgradeSlot::Armor));
    }
    if !c.equipped_shield.sockets.is_empty() {
        targets.push(SocketBenchTarget::Equipped(UpgradeSlot::Shield));
    }
    targets.extend(
        c.inventory
            .iter()
            .enumerate()
            .filter(|(_, item)| !item.sockets.is_empty())
            .map(|(index, _)| SocketBenchTarget::Inventory(index)),
    );
    targets
}

fn target_label(target: SocketBenchTarget) -> &'static str {
    match target {
        SocketBenchTarget::Equipped(UpgradeSlot::Weapon) => "Equipped weapon",
        SocketBenchTarget::Equipped(UpgradeSlot::Armor) => "Equipped armor",
        SocketBenchTarget::Equipped(UpgradeSlot::Shield) => "Equipped shield",
        SocketBenchTarget::Inventory(_) => "Carried gear",
    }
}

pub(crate) fn socket_bench_screen(c: &mut Character) {
    if !has_completed_project(c, TownProject::SocketBench) {
        append_pending_town_message(
            c,
            "Complete the Socket Bench project before socketing gems.",
        );
        return;
    }

    let mut selected_item = 0usize;
    let mut selected_socket = 0usize;
    let mut message = String::new();
    loop {
        let targets = socketed_targets(c);
        clamp_selection(&mut selected_item, targets.len());
        if let Some(target) = targets.get(selected_item).copied() {
            let socket_count = target_item(c, target)
                .map(|item| item.sockets.len())
                .unwrap_or_default();
            clamp_selection(&mut selected_socket, socket_count);
        } else {
            selected_socket = 0;
        }

        clear_screen();
        println!("{BOLD}{WHITE}Socket Bench{RESET}");
        println!("Free gem insertion, removal, and replacement.");
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();

        if targets.is_empty() {
            println!("No equipped or carried socketed gear.");
        } else {
            println!("{BOLD}Socketed Gear{RESET}");
            for (i, target) in targets.iter().copied().enumerate() {
                let marker = if i == selected_item {
                    format!("{GREEN}>{RESET}")
                } else {
                    " ".to_string()
                };
                if let Some(item) = target_item(c, target) {
                    println!("{marker} {}: {}", target_label(target), item.name);
                }
            }
            println!();
            if let Some(target) = targets.get(selected_item).copied() {
                if let Some(item) = target_item(c, target) {
                    println!("{BOLD}Sockets: {}{RESET}", item.name);
                    for (i, socket) in item.sockets.iter().copied().enumerate() {
                        let marker = if i == selected_socket {
                            format!("{GREEN}>{RESET}")
                        } else {
                            " ".to_string()
                        };
                        println!("{marker} {}. {}", i + 1, socket_text(socket));
                    }
                }
            }
        }

        print_footer(&[&format!(
            "{BOLD}Sockets:{RESET} {GREEN}↑/↓ or w/s{RESET}=gear  {GREEN}←/→ or a/d{RESET}=socket  {YELLOW}Enter{RESET}=manage  {RED}Esc{RESET}=back"
        )]);
        let Some(key) = read_key_char_nav_or_message(&mut message) else {
            break;
        };
        match key {
            '\u{1b}' => break,
            'w' | 'W' => selected_item = selected_item.saturating_sub(1),
            's' | 'S' => {
                if selected_item + 1 < targets.len() {
                    selected_item += 1;
                    selected_socket = 0;
                }
            }
            'a' | 'A' => selected_socket = selected_socket.saturating_sub(1),
            'd' | 'D' => {
                if let Some(target) = targets.get(selected_item).copied() {
                    let socket_count = target_item(c, target)
                        .map(|item| item.sockets.len())
                        .unwrap_or_default();
                    if selected_socket + 1 < socket_count {
                        selected_socket += 1;
                    }
                }
            }
            '\n' => {
                if let Some(target) = targets.get(selected_item).copied() {
                    match target_socket(c, target, selected_socket).flatten() {
                        Some(_) => {
                            message = filled_socket_action_screen(c, target, selected_socket);
                            append_autosave_status(c, &mut message);
                        }
                        None => {
                            if let Some(gem_index) = gem_picker_screen(c) {
                                message = insert_gem_from_socket_bench(
                                    c,
                                    target,
                                    selected_socket,
                                    gem_index,
                                );
                                append_autosave_status(c, &mut message);
                            }
                        }
                    }
                } else {
                    message = "No socket selected.".to_string();
                }
            }
            _ => message = "Unknown socket bench command.".to_string(),
        }
    }
}

fn filled_socket_action_screen(
    c: &mut Character,
    target: SocketBenchTarget,
    socket_index: usize,
) -> String {
    let mut selected = 0usize;
    let mut message = String::new();
    let options = ["Remove gem", "Replace gem"];
    loop {
        clamp_selection(&mut selected, options.len());
        clear_screen();
        println!("{BOLD}{WHITE}Filled Socket{RESET}");
        if let Some(socket) = target_socket(c, target, socket_index).flatten() {
            println!("Selected: {}", gem_socket_name(socket));
        }
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();
        for (i, option) in options.iter().enumerate() {
            let marker = if i == selected {
                format!("{GREEN}>{RESET}")
            } else {
                " ".to_string()
            };
            println!("{marker} {option}");
        }
        print_footer(&[&format!(
            "{BOLD}Socket:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=choose  {RED}Esc{RESET}=back"
        )]);
        let Some(key) = read_key_char_nav_or_message(&mut message) else {
            return String::new();
        };
        match key {
            '\u{1b}' => return String::new(),
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < options.len() {
                    selected += 1;
                }
            }
            '\n' => match selected {
                0 => return remove_gem_from_socket_bench(c, target, socket_index),
                1 => {
                    if let Some(gem_index) = gem_picker_screen(c) {
                        return replace_gem_from_socket_bench(c, target, socket_index, gem_index);
                    }
                }
                _ => {}
            },
            _ => message = "Unknown socket command.".to_string(),
        }
    }
}

fn insert_gem_from_socket_bench(
    c: &mut Character,
    target: SocketBenchTarget,
    socket_index: usize,
    inventory_index: usize,
) -> String {
    match target {
        SocketBenchTarget::Equipped(slot) => {
            insert_gem_into_equipped(c, slot, socket_index, inventory_index)
        }
        SocketBenchTarget::Inventory(_) => {
            insert_gem_into_target(c, target, socket_index, inventory_index)
        }
    }
}

fn remove_gem_from_socket_bench(
    c: &mut Character,
    target: SocketBenchTarget,
    socket_index: usize,
) -> String {
    match target {
        SocketBenchTarget::Equipped(slot) => remove_gem_from_equipped(c, slot, socket_index),
        SocketBenchTarget::Inventory(_) => remove_gem_from_target(c, target, socket_index),
    }
}

fn replace_gem_from_socket_bench(
    c: &mut Character,
    target: SocketBenchTarget,
    socket_index: usize,
    inventory_index: usize,
) -> String {
    match target {
        SocketBenchTarget::Equipped(slot) => {
            replace_gem_in_equipped(c, slot, socket_index, inventory_index)
        }
        SocketBenchTarget::Inventory(_) => {
            replace_gem_in_target(c, target, socket_index, inventory_index)
        }
    }
}

fn gem_picker_screen(c: &Character) -> Option<usize> {
    let gems: Vec<usize> = c
        .inventory
        .iter()
        .enumerate()
        .filter(|(_, item)| matches!(item.kind, ItemKind::Gem))
        .map(|(index, _)| index)
        .collect();
    let mut selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_selection(&mut selected, gems.len());
        clear_screen();
        println!("{BOLD}{WHITE}Select Gem{RESET}");
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();
        if gems.is_empty() {
            println!("No gems in inventory.");
        } else {
            for (i, inventory_index) in gems.iter().copied().enumerate() {
                let marker = if i == selected {
                    format!("{GREEN}>{RESET}")
                } else {
                    " ".to_string()
                };
                println!("{marker} {}", item_summary(&c.inventory[inventory_index]));
            }
        }
        print_footer(&[&format!(
            "{BOLD}Gems:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=choose  {RED}Esc{RESET}=back"
        )]);
        let Some(key) = read_key_char_nav_or_message(&mut message) else {
            return None;
        };
        match key {
            '\u{1b}' => return None,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < gems.len() {
                    selected += 1;
                }
            }
            '\n' => return gems.get(selected).copied(),
            _ => message = "Unknown gem command.".to_string(),
        }
    }
}

pub(crate) fn sell_value(c: &Character, item: &Item) -> u32 {
    let percent = if has_completed_project(c, TownProject::HireAppraiser) {
        30
    } else {
        25
    };
    item.value.saturating_mul(percent) / 100
}

pub(crate) fn sell_item_screen(c: &mut Character) {
    let mut selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_selection(&mut selected, c.inventory.len());
        clear_screen();
        println!("{BOLD}{YELLOW}Sell Items{RESET} - {}", gold_text(c.gold));
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        if c.inventory.is_empty() {
            println!("Inventory is empty.");
        } else {
            print_inventory_list(c, selected, inventory_visible_rows(8));
            let item = &c.inventory[selected];
            println!();
            println!("Selected: {}", item_summary(item));
            println!("Sell value: {YELLOW}{} gold{RESET}", sell_value(c, item));
        }
        print_footer(&[&format!(
            "{BOLD}Sell:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {YELLOW}Enter{RESET}=sell  {RED}Esc{RESET}=back"
        )]);
        let Some(key) = read_key_char_nav_or_message(&mut message) else {
            break;
        };
        match key {
            '\u{1b}' => break,
            'w' | 'W' => selected = selected.saturating_sub(1),
            's' | 'S' => {
                if selected + 1 < c.inventory.len() {
                    selected += 1;
                }
            }
            '\n' => {
                if c.inventory.is_empty() {
                    message = "Inventory is empty.".to_string();
                    continue;
                }
                let item = c.inventory.remove(selected);
                let item_sell_value = sell_value(c, &item);
                c.gold += item_sell_value;
                message = format!("Sold {} for {} gold.", item.name, item_sell_value);
                append_autosave_status(c, &mut message);
            }
            _ => message = "Unknown sell command.".to_string(),
        }
    }
}

pub(crate) fn stash_menu(c: &mut Character, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    let mut side = StashSide::Inventory;
    let mut inv_selected = 0usize;
    let mut stash_selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_grid_cursor(&mut inv_selected, &c.inventory);
        clamp_grid_cursor(&mut stash_selected, &c.stash);
        terminal
            .draw(|frame| {
                render_stash_screen(frame, c, side, inv_selected, stash_selected, &message)
            })
            .context("failed to draw stash")?;
        let key = read_key_char_nav()?;
        message.clear();
        match key {
            '\u{1b}' => return Ok(()),
            '\t' => side = side.other(),
            'w' | 'W' | 'a' | 'A' | 's' | 'S' | 'd' | 'D' => match side {
                StashSide::Inventory => {
                    inv_selected =
                        move_grid_cursor(inv_selected, c.inventory.columns, c.inventory.rows, key);
                }
                StashSide::Stash => {
                    stash_selected =
                        move_grid_cursor(stash_selected, c.stash.columns, c.stash.rows, key);
                }
            },
            '\n' => {
                message = match side {
                    StashSide::Inventory => {
                        move_selected(&mut c.inventory, &mut c.stash, inv_selected, "Stored")
                    }
                    StashSide::Stash => {
                        move_selected(&mut c.stash, &mut c.inventory, stash_selected, "Retrieved")
                    }
                };
                append_autosave_status(c, &mut message);
            }
            _ => message = "Unknown stash command.".to_string(),
        }
    }
}

pub(crate) fn render_stash_screen(
    frame: &mut Frame,
    c: &Character,
    side: StashSide,
    inv_selected: usize,
    stash_selected: usize,
    message: &str,
) {
    let footer_height = if message.is_empty() { 3 } else { 4 };
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(footer_height),
    ])
    .split(frame.area());
    let title = Paragraph::new(format!(
        "Stash - Inventory {}/{} - Stash {}/{}",
        c.inventory.len(),
        c.inventory.capacity(),
        c.stash.len(),
        c.stash.capacity()
    ))
    .block(Block::default().borders(Borders::ALL).title("Stash"));
    frame.render_widget(title, layout[0]);

    let inventory_title = stash_grid_title("Inventory", side == StashSide::Inventory);
    let stash_title = stash_grid_title("Stash", side == StashSide::Stash);
    let selected_item = match side {
        StashSide::Inventory => c.inventory.get(inv_selected),
        StashSide::Stash => c.stash.get(stash_selected),
    };
    let (grid, label) = match side {
        StashSide::Inventory => (&c.inventory, "Inventory"),
        StashSide::Stash => (&c.stash, "Stash"),
    };

    if frame.area().width >= 100 {
        let body = Layout::horizontal([
            Constraint::Length(24),
            Constraint::Min(34),
            Constraint::Length(38),
        ])
        .split(layout[1]);
        render_item_grid(frame, &c.inventory, inv_selected, body[0], &inventory_title);
        render_item_grid(frame, &c.stash, stash_selected, body[1], &stash_title);
        render_stash_details(frame, c, grid, label, selected_item, body[2]);
    } else {
        let body = Layout::vertical([Constraint::Length(10), Constraint::Min(3)]).split(layout[1]);
        let grids = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(body[0]);
        render_item_grid(
            frame,
            &c.inventory,
            inv_selected,
            grids[0],
            &inventory_title,
        );
        render_item_grid(frame, &c.stash, stash_selected, grids[1], &stash_title);
        render_stash_details(frame, c, grid, label, selected_item, body[1]);
    }

    let commands = "Tab=switch  WASD/Arrows=move  Enter=transfer  Esc=back";
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

fn render_stash_details(
    frame: &mut Frame,
    c: &Character,
    grid: &ItemGrid,
    label: &str,
    selected_item: Option<&Item>,
    area: Rect,
) {
    frame.render_widget(
        Paragraph::new(selected_item_detail_lines(c, grid, label, selected_item))
            .block(Block::default().borders(Borders::ALL).title("Details")),
        area,
    );
}

fn stash_grid_title(title: &str, active: bool) -> String {
    if active {
        format!("{title} *")
    } else {
        title.to_string()
    }
}

#[cfg(test)]
pub(crate) fn stash_screen_text_for_test(
    c: &Character,
    side: StashSide,
    inv_selected: usize,
    stash_selected: usize,
    message: &str,
) -> Vec<String> {
    let mut lines = vec![format!(
        "Stash - Inventory {}/{} - Stash {}/{}",
        c.inventory.len(),
        c.inventory.capacity(),
        c.stash.len(),
        c.stash.capacity()
    )];
    append_grid_text_for_test(
        &mut lines,
        &stash_grid_title("Inventory", side == StashSide::Inventory),
        &c.inventory,
    );
    append_grid_text_for_test(
        &mut lines,
        &stash_grid_title("Stash", side == StashSide::Stash),
        &c.stash,
    );

    let selected_item = match side {
        StashSide::Inventory => c.inventory.get(inv_selected),
        StashSide::Stash => c.stash.get(stash_selected),
    };
    let (grid, label) = match side {
        StashSide::Inventory => (&c.inventory, "Inventory"),
        StashSide::Stash => (&c.stash, "Stash"),
    };
    lines.extend(
        selected_item_detail_lines(c, grid, label, selected_item)
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
    lines.push("Tab=switch  WASD/Arrows=move  Enter=transfer  Esc=back".to_string());
    lines
}

#[cfg(test)]
fn append_grid_text_for_test(lines: &mut Vec<String>, title: &str, grid: &ItemGrid) {
    lines.push(format!("{title} {} / {}", grid.len(), grid.capacity()));
    for row in 0..grid.rows {
        let mut line = String::new();
        for col in 0..grid.columns {
            let index = usize::from(row) * usize::from(grid.columns) + usize::from(col);
            line.push_str(&format!("[{}] ", inventory_cell_label(grid, index)));
        }
        lines.push(line);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StashSide {
    Inventory,
    Stash,
}

impl StashSide {
    pub(crate) fn other(self) -> Self {
        match self {
            StashSide::Inventory => StashSide::Stash,
            StashSide::Stash => StashSide::Inventory,
        }
    }
}

pub(crate) fn move_selected(
    from: &mut ItemGrid,
    to: &mut ItemGrid,
    index: usize,
    verb: &str,
) -> String {
    if from.is_empty() {
        "Nothing to move.".to_string()
    } else if index >= from.len() {
        "No item selected.".to_string()
    } else {
        let item = from.remove(index);
        let msg = format!("{} {}.", verb, item.name);
        if let Err(item) = to.try_push(item) {
            let _ = from.insert(index, item);
            "No room in destination.".to_string()
        } else {
            msg
        }
    }
}

pub(crate) fn spend_attributes(c: &mut Character) {
    let mut message = String::new();
    while c.unspent_attributes > 0 {
        clear_screen();
        println!(
            "{BOLD}{CYAN}Spend attributes{RESET} ({CYAN}{} left{RESET})",
            c.unspent_attributes
        );
        println!(
            "{GREEN}1){RESET} {RED}Strength {} -> {}{RESET} ({RED}+5 max HP{RESET})",
            c.strength,
            c.strength + 1
        );
        println!(
            "{GREEN}2){RESET} {GREEN}Dexterity {} -> {}{RESET} ({CYAN}+5 hit{RESET}, {YELLOW}+5 speed{RESET})",
            c.dexterity,
            c.dexterity + 1
        );
        println!(
            "{GREEN}3){RESET} {BLUE}Intelligence {} -> {}{RESET} ({BLUE}+5 max mana{RESET})",
            c.intelligence,
            c.intelligence + 1
        );
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        print_footer(&[&format!(
            "{BOLD}Attributes:{RESET} {GREEN}1{RESET}={RED}Strength{RESET}  {GREEN}2{RESET}={GREEN}Dexterity{RESET}  {GREEN}3{RESET}={BLUE}Intelligence{RESET}  {RED}Esc{RESET}=back"
        )]);
        let Some(key) = read_key_char_or_message(&mut message) else {
            break;
        };
        match key {
            '1' => {
                c.strength += 1;
                c.unspent_attributes -= 1;
                c.hp += 5;
                message = "Spent 1 attribute on Strength.".to_string();
                append_autosave_status(c, &mut message);
            }
            '2' => {
                c.dexterity += 1;
                c.unspent_attributes -= 1;
                message = "Spent 1 attribute on Dexterity.".to_string();
                append_autosave_status(c, &mut message);
            }
            '3' => {
                c.intelligence += 1;
                c.unspent_attributes -= 1;
                c.mana += 5;
                message = "Spent 1 attribute on Intelligence.".to_string();
                append_autosave_status(c, &mut message);
            }
            '\u{1b}' => break,
            _ => message = "Unknown attribute command.".to_string(),
        }
    }
}
