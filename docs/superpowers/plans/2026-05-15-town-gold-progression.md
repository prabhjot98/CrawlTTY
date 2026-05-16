# Town Gold Progression Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make gold the currency for permanent town projects, while gear power uses shards/materials instead of gold.

**Architecture:** Add persisted town project completion state to `Character`, implement project metadata and purchase logic in a focused `town_projects` module, expose a town projects menu from the town hub, then update merchant/blacksmith/sell behavior to respect the new economy. Project completion should be stored by stable enum identifiers, and project effects should be queried through helper functions instead of string matching.

**Tech Stack:** Rust 2024, serde JSON saves, crossterm terminal input, ratatui town UI, existing `cargo test` suite.

---

## File Structure

- Create `src/town_projects.rs`: project enum, metadata, availability, purchasing, and text helpers.
- Modify `src/main.rs`: import the new module and add `p` town command for projects.
- Modify `src/model.rs`: persist completed town project identifiers on `Character`.
- Modify `src/town.rs`: add the projects menu, gate blacksmith services behind projects, remove direct gold purchases from merchant/blacksmith service lists, make gear upgrades shard-only, and apply appraiser sell-price benefits.
- Modify `src/ui.rs`: add `p=projects` to the town footer.
- Modify `src/tests.rs`: add focused TDD coverage for project purchasing, gates, save compatibility, shard-only upgrades, and sell-price benefits.
- Modify `README.md`: document the new town command and gold economy role.

## Project Identifiers And Initial Costs

Use these stable project identifiers and costs in `src/town_projects.rs`:

| Project | Group | Cost | Gate | Benefit |
| --- | --- | ---: | --- | --- |
| `RebuildForge` | Smith | 150 | Always available | Unlock salvage and shard-only gear upgrades |
| `ReinforcedAnvil` | Smith | 300 | `RebuildForge` completed | Adds +1 shard yield from salvaged gear |
| `SocketBench` | Smith | 600 | `act1_completed` and `ReinforcedAnvil` completed | Records socket service infrastructure for future socket systems |
| `StorehouseShelves` | Quartermaster | 200 | Always available | Records expanded storage infrastructure |
| `HireAppraiser` | Appraiser | 250 | Always available | Sell value improves from 25% to 30% |
| `HerbGarden` | Alchemist | 350 | `act1_completed` | Records herb growing infrastructure |
| `Distillery` | Alchemist | 500 | `HerbGarden` completed | Records potion crafting infrastructure |

`SocketBench`, `StorehouseShelves`, `HerbGarden`, and `Distillery` may have recorded benefits before their full downstream systems exist. They should still be purchasable projects with clear completion text, because this implementation is the town progression foundation.

### Task 1: Persisted Town Project Model

**Files:**
- Create: `src/town_projects.rs`
- Modify: `src/main.rs`
- Modify: `src/model.rs`
- Test: `src/tests.rs`

- [x] **Step 1: Write failing tests for project metadata, gates, purchases, and save compatibility**

Add these tests near the other town/economy tests in `src/tests.rs`:

```rust
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
        "class_name": "Warrior",
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
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test town_project -- --nocapture
```

Expected: compile failures for missing `TownProject`, `ProjectAvailability`, `town_project_availability`, `complete_town_project`, `has_completed_project`, and `complete_project_for_test`.

- [x] **Step 3: Add persisted project state and module wiring**

In `src/main.rs`, add the module and re-export near existing modules:

```rust
mod town_projects;
pub(crate) use town_projects::*;
```

In `src/model.rs`, add the enum near the other persisted enums:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum TownProject {
    RebuildForge,
    ReinforcedAnvil,
    SocketBench,
    StorehouseShelves,
    HireAppraiser,
    HerbGarden,
    Distillery,
}
```

In `Character`, add the field near the shard/project-adjacent progression fields:

```rust
#[serde(default)]
pub(crate) completed_town_projects: Vec<TownProject>,
```

In `Character::new`, initialize it:

```rust
completed_town_projects: Vec::new(),
```

- [x] **Step 4: Add project metadata and purchase logic**

Create `src/town_projects.rs` with:

```rust
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectAvailability {
    Available,
    Completed,
    Locked(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TownProjectDefinition {
    pub(crate) project: TownProject,
    pub(crate) group: &'static str,
    pub(crate) name: &'static str,
    pub(crate) cost: u32,
    pub(crate) benefit: &'static str,
}

pub(crate) const TOWN_PROJECTS: &[TownProjectDefinition] = &[
    TownProjectDefinition {
        project: TownProject::RebuildForge,
        group: "Smith",
        name: "Rebuild the Forge",
        cost: 150,
        benefit: "Unlock salvage and shard-only gear upgrades.",
    },
    TownProjectDefinition {
        project: TownProject::ReinforcedAnvil,
        group: "Smith",
        name: "Reinforced Anvil",
        cost: 300,
        benefit: "Salvaged gear yields +1 shard.",
    },
    TownProjectDefinition {
        project: TownProject::SocketBench,
        group: "Smith",
        name: "Socket Bench",
        cost: 600,
        benefit: "Unlock socket infrastructure for future gem and rune systems.",
    },
    TownProjectDefinition {
        project: TownProject::StorehouseShelves,
        group: "Quartermaster",
        name: "Storehouse Shelves",
        cost: 200,
        benefit: "Expand town storage infrastructure.",
    },
    TownProjectDefinition {
        project: TownProject::HireAppraiser,
        group: "Appraiser",
        name: "Hire Appraiser",
        cost: 250,
        benefit: "Improve sell prices from 25% to 30%.",
    },
    TownProjectDefinition {
        project: TownProject::HerbGarden,
        group: "Alchemist",
        name: "Herb Garden",
        cost: 350,
        benefit: "Unlock growing herbs.",
    },
    TownProjectDefinition {
        project: TownProject::Distillery,
        group: "Alchemist",
        name: "Distillery",
        cost: 500,
        benefit: "Unlock potion crafting infrastructure.",
    },
];

pub(crate) fn town_project_definition(project: TownProject) -> &'static TownProjectDefinition {
    TOWN_PROJECTS
        .iter()
        .find(|definition| definition.project == project)
        .expect("all town projects have definitions")
}

pub(crate) fn has_completed_project(c: &Character, project: TownProject) -> bool {
    c.completed_town_projects.contains(&project)
}

pub(crate) fn town_project_availability(
    c: &Character,
    project: TownProject,
) -> ProjectAvailability {
    if has_completed_project(c, project) {
        return ProjectAvailability::Completed;
    }
    match project {
        TownProject::RebuildForge
        | TownProject::StorehouseShelves
        | TownProject::HireAppraiser => ProjectAvailability::Available,
        TownProject::ReinforcedAnvil => {
            if has_completed_project(c, TownProject::RebuildForge) {
                ProjectAvailability::Available
            } else {
                ProjectAvailability::Locked("Requires Rebuild the Forge.")
            }
        }
        TownProject::SocketBench => {
            if !c.act1_completed {
                ProjectAvailability::Locked("Requires Act I completed.")
            } else if !has_completed_project(c, TownProject::ReinforcedAnvil) {
                ProjectAvailability::Locked("Requires Reinforced Anvil.")
            } else {
                ProjectAvailability::Available
            }
        }
        TownProject::HerbGarden => {
            if c.act1_completed {
                ProjectAvailability::Available
            } else {
                ProjectAvailability::Locked("Requires Act I completed.")
            }
        }
        TownProject::Distillery => {
            if has_completed_project(c, TownProject::HerbGarden) {
                ProjectAvailability::Available
            } else {
                ProjectAvailability::Locked("Requires Herb Garden.")
            }
        }
    }
}

pub(crate) fn complete_town_project(c: &mut Character, project: TownProject) -> String {
    let definition = town_project_definition(project);
    match town_project_availability(c, project) {
        ProjectAvailability::Completed => {
            return format!("{} is already complete.", definition.name);
        }
        ProjectAvailability::Locked(reason) => return reason.to_string(),
        ProjectAvailability::Available => {}
    }
    if c.gold < definition.cost {
        return format!(
            "Need {} gold to complete {}.",
            definition.cost, definition.name
        );
    }
    c.gold -= definition.cost;
    c.completed_town_projects.push(project);
    format!("Completed project: {}.", definition.name)
}

#[cfg(test)]
pub(crate) fn complete_project_for_test(c: &mut Character, project: TownProject) {
    if !has_completed_project(c, project) {
        c.completed_town_projects.push(project);
    }
}
```

- [x] **Step 5: Run focused tests**

Run:

```bash
cargo test town_project -- --nocapture
```

Expected: all `town_project` tests pass.

- [x] **Step 6: Run required guard and commit**

Run:

```bash
scripts/agent-commit-guard.sh --fix
git status --short
git diff
git add src/main.rs src/model.rs src/town_projects.rs src/tests.rs
git commit -m "Add town project progression model"
```

Expected: guard passes, only Task 1 files are staged, commit succeeds.

### Task 2: Town Projects Menu And Town Command

**Files:**
- Modify: `src/town.rs`
- Modify: `src/main.rs`
- Modify: `src/ui.rs`
- Test: `src/tests.rs`

- [x] **Step 1: Write failing tests for project display text helpers**

Add these tests to `src/tests.rs`:

```rust
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
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test town_project_status_text -- --nocapture
cargo test town_project_row_text -- --nocapture
```

Expected: compile failures for missing display helper functions.

- [x] **Step 3: Add display helpers to `src/town_projects.rs`**

Append these functions:

```rust
pub(crate) fn town_project_status_text(c: &Character, project: TownProject) -> String {
    match town_project_availability(c, project) {
        ProjectAvailability::Available => "Available".to_string(),
        ProjectAvailability::Completed => "Complete".to_string(),
        ProjectAvailability::Locked(reason) => format!("Locked: {reason}"),
    }
}

pub(crate) fn town_project_row_text(c: &Character, project: TownProject) -> String {
    let definition = town_project_definition(project);
    format!(
        "[{}] {} - {} gold - {} - {}",
        definition.group,
        definition.name,
        definition.cost,
        town_project_status_text(c, project),
        definition.benefit
    )
}
```

- [x] **Step 4: Add `town_projects_menu` to `src/town.rs`**

Add this function near the other town service menus:

```rust
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
            println!(
                "{marker} {}",
                town_project_row_text(c, definition.project)
            );
        }
        println!();
        let selected_project = TOWN_PROJECTS[selected].project;
        println!(
            "{BOLD}Selected:{RESET} {}",
            town_project_definition(selected_project).name
        );
        println!(
            "{}",
            town_project_definition(selected_project).benefit
        );
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
```

- [x] **Step 5: Wire town command**

In `src/main.rs`, add this arm to the town input `match`:

```rust
'p' | 'P' => {
    town_projects_menu(character);
    town_message.clear();
}
```

In `src/ui.rs`, add projects to the first town footer command line:

```rust
("p", "projects"),
```

- [x] **Step 6: Run focused and full guard, then commit**

Run:

```bash
cargo test town_project -- --nocapture
scripts/agent-commit-guard.sh --fix
git status --short
git diff
git add src/town_projects.rs src/town.rs src/main.rs src/ui.rs src/tests.rs
git commit -m "Add town projects menu"
```

Expected: tests and guard pass, commit succeeds.

### Task 3: Gold-Only Town Progression Cleanup

**Files:**
- Modify: `src/town.rs`
- Modify: `src/tests.rs`

- [x] **Step 1: Write failing tests for shard-only upgrades, forge gate, reinforced salvage, and appraiser sell value**

Update the existing `blacksmith_upgrades_equipped_gear_with_shards_and_gold` test by renaming it to `blacksmith_upgrades_equipped_gear_with_shards_only_after_forge_project` and replacing the body with:

```rust
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
    assert_eq!(weapon_message, "Upgraded Rusted Sword (3-5 dmg, STR F, DEX F) to +1.");
    assert_eq!(c.equipped_weapon.upgrade_level, 1);
    assert_eq!(c.equipped_weapon.damage_min, 4);
    assert_eq!(c.equipped_weapon.damage_max, 6);
    assert_eq!(c.weapon_shards, 0);
    assert_eq!(c.gold, 0);

    let armor_message = upgrade_equipped_message(&mut c, UpgradeSlot::Armor);
    assert_eq!(armor_message, "Upgraded Cloth Tunic (+1 armor) to +1.");
    assert_eq!(c.equipped_armor.armor, 2);

    let shield_message = upgrade_equipped_message(&mut c, UpgradeSlot::Shield);
    assert_eq!(shield_message, "Upgraded Worn Shield (+1 armor, +2 dodge) to +1.");
    assert_eq!(c.equipped_shield.armor, 2);
}
```

Update the existing `blacksmith_salvage_converts_gear_to_type_shards` test so forge completion is explicit:

```rust
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
```

Update `blacksmith_upgrade_cost_scales_with_upgrade_level` to expect only shards:

```rust
#[test]
fn blacksmith_upgrade_cost_scales_with_upgrade_level() {
    let mut item = rusted_sword();
    assert_eq!(upgrade_cost(&item), 2);
    upgrade_item(&mut item);
    assert_eq!(upgrade_cost(&item), 4);
}
```

Add these tests:

```rust
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
```

- [x] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test blacksmith_upgrade_cost_scales_with_upgrade_level -- --nocapture
cargo test blacksmith_upgrades_equipped_gear_with_shards_only_after_forge_project -- --nocapture
cargo test salvage_requires_forge_and_reinforced_anvil_adds_one_shard -- --nocapture
cargo test appraiser_project_improves_sell_value -- --nocapture
```

Expected: failures because upgrade costs still include gold, forge gates do not exist, reinforced salvage does not add shards, and `sell_value` does not exist.

- [x] **Step 3: Make upgrade costs shard-only and gate smith services**

In `src/town.rs`, update `upgrade_equipped_message`:

```rust
pub(crate) fn upgrade_equipped_message(c: &mut Character, slot: UpgradeSlot) -> String {
    if !has_completed_project(c, TownProject::RebuildForge) {
        return "Rebuild the Forge before upgrading gear.".to_string();
    }
    let (cost_shards, kind, item_name) = {
        let item = equipped_item(c, slot);
        let kind = shard_kind(item).expect("equipped gear has shard kind");
        let cost_shards = upgrade_cost(item);
        (cost_shards, kind, item.name.clone())
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
```

Change `upgrade_cost`:

```rust
pub(crate) fn upgrade_cost(item: &Item) -> u32 {
    let next = item.upgrade_level + 1;
    next * 2
}
```

- [x] **Step 4: Gate salvage and add reinforced yield**

In `salvage_inventory_item`, add the forge gate first:

```rust
if !has_completed_project(c, TownProject::RebuildForge) {
    return "Rebuild the Forge before salvaging gear.".to_string();
}
```

Change the amount calculation:

```rust
let amount = salvage_shard_yield(c, &item);
```

Change `salvage_shard_yield` signature and body:

```rust
pub(crate) fn salvage_shard_yield(c: &Character, item: &Item) -> u32 {
    let rarity_bonus = match item.rarity {
        Rarity::Common => 1,
        Rarity::Magic => 2,
        Rarity::Rare => 3,
    };
    let anvil_bonus = if has_completed_project(c, TownProject::ReinforcedAnvil) {
        1
    } else {
        0
    };
    rarity_bonus + item.upgrade_level + anvil_bonus
}
```

Update the salvage screen preview call to pass `c`:

```rust
salvage_shard_yield(c, &c.inventory[selected])
```

- [x] **Step 5: Add appraiser sell value helper and use it**

Add this function near `sell_item_screen`:

```rust
pub(crate) fn sell_value(c: &Character, item: &Item) -> u32 {
    let percent = if has_completed_project(c, TownProject::HireAppraiser) {
        30
    } else {
        25
    };
    item.value * percent / 100
}
```

In `sell_item_screen`, replace both `item.value / 4` uses with `sell_value(c, item)`.

- [x] **Step 6: Remove direct gold purchase services from merchant and blacksmith menus**

In `merchant`, replace the options with:

```rust
let options = ["Sell items".to_string()];
```

Update the explanatory line:

```rust
println!("Gold funds town projects. Sell unwanted items here.");
```

Update the enter handler:

```rust
'\n' => match selected {
    0 => sell_item_screen(c),
    _ => {}
},
```

In `blacksmith`, replace the options with:

```rust
let options = [
    "Salvage carried gear for shards",
    "Sharpen equipped weapon",
    "Reinforce equipped armor",
    "Brace equipped shield",
];
```

Update the explanatory line:

```rust
println!(
    "Town projects unlock smith services. Salvage gear into type shards, then spend shards to upgrade equipped gear."
);
```

Update blacksmith enter handlers so indices `0..=3` map to salvage, weapon, armor, shield.

- [x] **Step 7: Run focused tests and guard, then commit**

Run:

```bash
cargo test blacksmith_upgrade_cost_scales_with_upgrade_level -- --nocapture
cargo test blacksmith_upgrades_equipped_gear_with_shards_only_after_forge_project -- --nocapture
cargo test salvage_requires_forge_and_reinforced_anvil_adds_one_shard -- --nocapture
cargo test appraiser_project_improves_sell_value -- --nocapture
scripts/agent-commit-guard.sh --fix
git status --short
git diff
git add src/town.rs src/tests.rs
git commit -m "Make gold fund town progression only"
```

Expected: tests and guard pass, commit succeeds.

### Task 4: Documentation And Full Integration Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-05-15-town-gold-progression.md`

- [x] **Step 1: Update README town controls and feature summary**

In `README.md`, update current features to include town projects:

```markdown
- Town hub with merchant, blacksmith, stash, town projects, quest giver, dungeon entrance, and automatic full healing on return
```

Add the town command:

```markdown
- `p` town projects
```

Update the economy feature bullet:

```markdown
- Inventory, equipment, selling, salvaging, shard-only gear upgrading, and gold-funded town projects
```

- [x] **Step 2: Mark plan checkboxes complete as tasks are actually finished**

After Tasks 1-3 are committed, update this plan file so completed steps use `[x]` for those tasks. Leave Task 4 checkboxes accurate based on what has been done.

- [x] **Step 3: Run final verification**

Run:

```bash
scripts/agent-commit-guard.sh --fix
git status --short
git diff
```

Expected: guard passes. `git diff` contains only README and plan checkbox/documentation updates.

- [x] **Step 4: Commit docs and plan status**

Run:

```bash
git add README.md docs/superpowers/plans/2026-05-15-town-gold-progression.md
git commit -m "Document town project controls"
```

Expected: commit succeeds.

## Final Verification

After all tasks are committed, run:

```bash
scripts/agent-commit-guard.sh --fix
git status --short
git log --oneline -5
```

Expected:

- `cargo fmt`, `cargo test`, and `cargo check` pass.
- Working tree is clean.
- Recent commits include the town project model, menu, gold economy cleanup, and documentation.

## Subagent Execution Notes

Use one fresh worker subagent per task. Workers are not alone in the codebase; they must not revert edits made by earlier workers and must adapt to the current branch state. Do not run implementation workers in parallel because Tasks 1-3 intentionally share `src/town.rs`, `src/tests.rs`, and project model APIs.
