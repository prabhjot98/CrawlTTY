use crate::*;

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

pub(crate) fn healer(c: &mut Character) {
    c.hp = c.max_hp();
    c.mana = c.max_mana();
}

pub(crate) fn merchant(c: &mut Character) {
    let mut selected = 0usize;
    let mut message = String::new();
    let options = [
        format!("Buy Lesser Health Potion - {HEALTH_POTION_COST} gold"),
        format!("Buy Lesser Mana Potion - {MANA_POTION_COST} gold"),
        "Sell items".to_string(),
    ];
    loop {
        clamp_selection(&mut selected, options.len());
        clear_screen();
        println!("{BOLD}{YELLOW}Merchant{RESET} - {}", gold_text(c.gold));
        println!("Selling gives 25% of item value.");
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
                0 => {
                    message = buy_item_message(c, health_potion());
                    append_autosave_status(c, &mut message);
                }
                1 => {
                    message = buy_item_message(c, mana_potion());
                    append_autosave_status(c, &mut message);
                }
                2 => sell_item_screen(c),
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
        "Buy Crude Axe - 60 gold",
        "Buy Battered Mail - 55 gold",
        "Buy Worn Shield - 40 gold",
        "Salvage carried gear for shards",
        "Sharpen equipped weapon",
        "Reinforce equipped armor",
        "Brace equipped shield",
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
            "No durability or repairs. Salvage gear into type shards, then spend shards + gold to upgrade equipped gear."
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
                0 => {
                    message = buy_item_message(c, crude_axe());
                    append_autosave_status(c, &mut message);
                }
                1 => {
                    message = buy_item_message(c, battered_mail());
                    append_autosave_status(c, &mut message);
                }
                2 => {
                    message = buy_item_message(c, worn_shield());
                    append_autosave_status(c, &mut message);
                }
                3 => salvage_screen(c),
                4 => {
                    message = upgrade_equipped_message(c, UpgradeSlot::Weapon);
                    append_autosave_status(c, &mut message);
                }
                5 => {
                    message = upgrade_equipped_message(c, UpgradeSlot::Armor);
                    append_autosave_status(c, &mut message);
                }
                6 => {
                    message = upgrade_equipped_message(c, UpgradeSlot::Shield);
                    append_autosave_status(c, &mut message);
                }
                _ => {}
            },
            _ => message = "Unknown blacksmith command.".to_string(),
        }
    }
}

pub(crate) fn buy_item_message(c: &mut Character, item: Item) -> String {
    if c.gold < item.value {
        return "Not enough gold.".to_string();
    }
    c.gold -= item.value;
    let message = format!("Bought {}.", item.name);
    c.inventory.push(item);
    message
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
                    salvage_shard_yield(&c.inventory[selected]),
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
    if index >= c.inventory.len() {
        return "No item selected.".to_string();
    }
    let Some(kind) = shard_kind(&c.inventory[index]) else {
        return "Only weapons, armor, and shields can be salvaged.".to_string();
    };
    let item = c.inventory.remove(index);
    let amount = salvage_shard_yield(&item);
    add_shards(c, kind, amount);
    format!(
        "Salvaged {} into {} {} shard(s).",
        item.name,
        amount,
        shard_name(kind)
    )
}

pub(crate) fn salvage_shard_yield(item: &Item) -> u32 {
    let rarity_bonus = match item.rarity {
        Rarity::Common => 1,
        Rarity::Magic => 2,
        Rarity::Rare => 3,
    };
    rarity_bonus + item.upgrade_level
}

pub(crate) fn upgrade_equipped_message(c: &mut Character, slot: UpgradeSlot) -> String {
    let (cost_shards, cost_gold, kind, item_name) = {
        let item = equipped_item(c, slot);
        let kind = shard_kind(item).expect("equipped gear has shard kind");
        let (cost_shards, cost_gold) = upgrade_cost(item);
        (cost_shards, cost_gold, kind, item.name.clone())
    };
    if shard_count(c, kind) < cost_shards {
        return format!(
            "Need {} {} shard(s) to upgrade {}.",
            cost_shards,
            shard_name(kind),
            item_name
        );
    }
    if c.gold < cost_gold {
        return format!("Need {cost_gold} gold to upgrade {item_name}.");
    }
    spend_shards(c, kind, cost_shards);
    c.gold -= cost_gold;
    let item = equipped_item_mut(c, slot);
    upgrade_item(item);
    format!("Upgraded {} to +{}.", item.name, item.upgrade_level)
}

pub(crate) fn upgrade_cost(item: &Item) -> (u32, u32) {
    let next = item.upgrade_level + 1;
    (next * 2, next * 25)
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
        ItemKind::HealthPotion | ItemKind::ManaPotion => {}
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
        ItemKind::HealthPotion | ItemKind::ManaPotion => None,
    }
}

pub(crate) fn shard_name(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Weapon => "weapon",
        ItemKind::Armor => "armor",
        ItemKind::Shield => "shield",
        ItemKind::HealthPotion | ItemKind::ManaPotion => "unknown",
    }
}

pub(crate) fn shard_count(c: &Character, kind: ItemKind) -> u32 {
    match kind {
        ItemKind::Weapon => c.weapon_shards,
        ItemKind::Armor => c.armor_shards,
        ItemKind::Shield => c.shield_shards,
        ItemKind::HealthPotion | ItemKind::ManaPotion => 0,
    }
}

pub(crate) fn add_shards(c: &mut Character, kind: ItemKind, amount: u32) {
    match kind {
        ItemKind::Weapon => c.weapon_shards += amount,
        ItemKind::Armor => c.armor_shards += amount,
        ItemKind::Shield => c.shield_shards += amount,
        ItemKind::HealthPotion | ItemKind::ManaPotion => {}
    }
}

pub(crate) fn spend_shards(c: &mut Character, kind: ItemKind, amount: u32) {
    match kind {
        ItemKind::Weapon => c.weapon_shards = c.weapon_shards.saturating_sub(amount),
        ItemKind::Armor => c.armor_shards = c.armor_shards.saturating_sub(amount),
        ItemKind::Shield => c.shield_shards = c.shield_shards.saturating_sub(amount),
        ItemKind::HealthPotion | ItemKind::ManaPotion => {}
    }
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
            println!("Sell value: {YELLOW}{} gold{RESET}", item.value / 4);
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
                let sell_value = item.value / 4;
                c.gold += sell_value;
                message = format!("Sold {} for {} gold.", item.name, sell_value);
                append_autosave_status(c, &mut message);
            }
            _ => message = "Unknown sell command.".to_string(),
        }
    }
}

pub(crate) fn stash_menu(c: &mut Character) {
    let mut side = StashSide::Inventory;
    let mut inv_selected = 0usize;
    let mut stash_selected = 0usize;
    let mut message = String::new();
    loop {
        clamp_selection(&mut inv_selected, c.inventory.len());
        clamp_selection(&mut stash_selected, c.stash.len());
        clear_screen();
        println!("{BOLD}{MAGENTA}Stash{RESET}");
        println!(
            "{CYAN}Inventory items: {}{RESET}   {MAGENTA}Stash items: {}{RESET}",
            c.inventory.len(),
            c.stash.len()
        );
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();
        let visible_rows = (inventory_visible_rows(12) / 2).max(4);
        print_stash_column(
            "Inventory",
            &c.inventory,
            inv_selected,
            side == StashSide::Inventory,
            visible_rows,
        );
        println!();
        print_stash_column(
            "Stash",
            &c.stash,
            stash_selected,
            side == StashSide::Stash,
            visible_rows,
        );
        print_footer(&[&format!(
            "{BOLD}Stash:{RESET} {GREEN}↑/↓ or w/s{RESET}=select  {CYAN}Tab{RESET}=switch list  {YELLOW}Enter{RESET}=move selected  {RED}Esc{RESET}=back"
        )]);
        let Some(key) = read_key_char_nav_or_message(&mut message) else {
            break;
        };
        match key {
            '\u{1b}' => break,
            '\t' => side = side.other(),
            'w' | 'W' => match side {
                StashSide::Inventory => inv_selected = inv_selected.saturating_sub(1),
                StashSide::Stash => stash_selected = stash_selected.saturating_sub(1),
            },
            's' | 'S' => match side {
                StashSide::Inventory => {
                    if inv_selected + 1 < c.inventory.len() {
                        inv_selected += 1;
                    }
                }
                StashSide::Stash => {
                    if stash_selected + 1 < c.stash.len() {
                        stash_selected += 1;
                    }
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

#[derive(Clone, Copy, PartialEq, Eq)]
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
    from: &mut Vec<Item>,
    to: &mut Vec<Item>,
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
        to.push(item);
        msg
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
