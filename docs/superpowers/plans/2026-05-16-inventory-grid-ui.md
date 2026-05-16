# Inventory Grid UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace list-style inventory and stash with ratatui grid screens backed by capacity-limited item grids, bag upgrades, and ground loot that prevents full-inventory item loss.

**Architecture:** Add an `ItemGrid` container to own ordered, auto-compacting item storage while preserving enough Vec-like helpers to keep existing systems understandable during migration. Move player inventory, stash, and ground-loot choice screens to ratatui loops that receive the active terminal instead of going through `run_legacy_screen`. Add `GroundItem` storage to `Dungeon` and route all loot insertion through capacity-aware helpers.

**Tech Stack:** Rust 2024, serde, crossterm input, ratatui 0.29, existing unit tests in `src/tests.rs`, required repo guard `scripts/agent-commit-guard.sh --fix`.

---

## File Structure

- Modify `src/model.rs`: add `ItemGrid`, `GroundItem`, bag dimension constants, new Quartermaster project variants, and update `Character`/`Dungeon` fields.
- Modify `src/items.rs`: no new item types, but constructors must continue filling all `Item` fields after model changes.
- Modify `src/inventory.rs`: keep item summary/comparison helpers, replace list inventory screen with ratatui grid rendering and capacity-safe inventory mutations.
- Modify `src/town.rs`: update sell, salvage, socket bench, and stash code to work with `ItemGrid`; replace stash legacy screen with ratatui grid screen entry point.
- Modify `src/town_projects.rs`: define the Quartermaster bag upgrade chain and helper functions that compute current bag dimensions.
- Modify `src/dungeon.rs`: add ground item rendering, `g` pickup, walk-over pickup, capacity-aware loot insertion, and dungeon drop behavior.
- Modify `src/dungeon_gen.rs`: initialize `Dungeon.ground_items`.
- Modify `src/input.rs`: map left and right arrows to `a` and `d` when navigation mode is enabled.
- Modify `src/main.rs`: call ratatui inventory/stash screens directly instead of wrapping them in `run_legacy_screen`.
- Modify `src/ui.rs`: add shared ratatui item detail/grid helpers only if keeping them in `inventory.rs` would make that file too broad.
- Modify `src/tests.rs`: add focused tests for grid capacity, upgrades, ground loot, pickup behavior, and render text.
- Modify `README.md` and `design.md`: document `g=pickup`, grid inventory status, bag cap, and save reset expectation.

## Task 1: Add ItemGrid And GroundItem Data Model

**Files:**
- Modify: `src/model.rs`
- Modify: `src/items.rs`
- Modify: `src/dungeon_gen.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing ItemGrid tests**

Add these tests near the existing inventory and save tests in `src/tests.rs`:

```rust
#[test]
fn item_grid_capacity_add_remove_and_auto_compaction() {
    let mut grid = ItemGrid::new(2, 2, Vec::new());

    assert_eq!(grid.columns, 2);
    assert_eq!(grid.rows, 2);
    assert_eq!(grid.capacity(), 4);
    assert_eq!(grid.len(), 0);
    assert!(grid.is_empty());
    assert!(grid.has_space());

    assert!(grid.push(health_potion()));
    assert!(grid.push(mana_potion()));
    assert_eq!(grid.len(), 2);
    assert!(matches!(grid[0].kind, ItemKind::HealthPotion));
    assert!(matches!(grid[1].kind, ItemKind::ManaPotion));

    let removed = grid.remove(0);
    assert!(matches!(removed.kind, ItemKind::HealthPotion));
    assert_eq!(grid.len(), 1);
    assert!(matches!(grid[0].kind, ItemKind::ManaPotion));

    assert!(grid.push(health_potion()));
    assert!(grid.push(health_potion()));
    assert!(grid.push(mana_potion()));
    assert!(!grid.push(rusted_sword()));
    assert_eq!(grid.len(), 4);
}

#[test]
fn new_character_uses_starting_bag_and_stash_grids() {
    let c = test_character();

    assert_eq!((c.inventory.columns, c.inventory.rows), (4, 4));
    assert_eq!(c.inventory.capacity(), 16);
    assert_eq!(c.inventory.len(), 3);
    assert_eq!((c.stash.columns, c.stash.rows), (8, 8));
    assert_eq!(c.stash.capacity(), 64);
    assert_eq!(c.stash.len(), 0);
}

#[test]
fn dungeon_starts_without_ground_items() {
    let d = generate_dungeon(1);

    assert!(d.ground_items.is_empty());
}
```

- [ ] **Step 2: Run the focused tests and verify they fail**

Run:

```bash
cargo test item_grid_capacity_add_remove_and_auto_compaction
cargo test new_character_uses_starting_bag_and_stash_grids
cargo test dungeon_starts_without_ground_items
```

Expected: compile failure because `ItemGrid`, `GroundItem`, and `Dungeon.ground_items` do not exist yet.

- [ ] **Step 3: Add model types and constants**

In `src/model.rs`, add these constants near the existing game constants:

```rust
pub(crate) const STARTING_BAG_COLUMNS: u16 = 4;
pub(crate) const STARTING_BAG_ROWS: u16 = 4;
pub(crate) const MAX_BAG_COLUMNS: u16 = 8;
pub(crate) const MAX_BAG_ROWS: u16 = 8;
pub(crate) const STARTING_STASH_COLUMNS: u16 = 8;
pub(crate) const STARTING_STASH_ROWS: u16 = 8;
```

Add `ItemGrid` after `Item`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ItemGrid {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
    pub(crate) items: Vec<Item>,
}

impl ItemGrid {
    pub(crate) fn new(columns: u16, rows: u16, items: Vec<Item>) -> Self {
        let mut grid = Self {
            columns,
            rows,
            items: Vec::new(),
        };
        for item in items {
            let _ = grid.push(item);
        }
        grid
    }

    pub(crate) fn player_starting(items: Vec<Item>) -> Self {
        Self::new(STARTING_BAG_COLUMNS, STARTING_BAG_ROWS, items)
    }

    pub(crate) fn stash_starting() -> Self {
        Self::new(STARTING_STASH_COLUMNS, STARTING_STASH_ROWS, Vec::new())
    }

    pub(crate) fn capacity(&self) -> usize {
        usize::from(self.columns) * usize::from(self.rows)
    }

    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(crate) fn has_space(&self) -> bool {
        self.len() < self.capacity()
    }

    pub(crate) fn available_slots(&self) -> usize {
        self.capacity().saturating_sub(self.len())
    }

    pub(crate) fn push(&mut self, item: Item) -> bool {
        if self.has_space() {
            self.items.push(item);
            true
        } else {
            false
        }
    }

    pub(crate) fn try_push(&mut self, item: Item) -> Result<usize, Item> {
        if self.has_space() {
            self.items.push(item);
            Ok(self.items.len() - 1)
        } else {
            Err(item)
        }
    }

    pub(crate) fn remove(&mut self, index: usize) -> Item {
        self.items.remove(index)
    }

    pub(crate) fn insert(&mut self, index: usize, item: Item) -> bool {
        if self.has_space() && index <= self.items.len() {
            self.items.insert(index, item);
            true
        } else {
            false
        }
    }

    pub(crate) fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, Item> {
        self.items.iter()
    }

    pub(crate) fn clear(&mut self) {
        self.items.clear();
    }

    pub(crate) fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Item) -> bool,
    {
        self.items.retain(f);
    }
}

impl std::ops::Index<usize> for ItemGrid {
    type Output = Item;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl std::ops::IndexMut<usize> for ItemGrid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}
```

Add `GroundItem` near `Chest`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GroundItem {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) item: Item,
}
```

Update `Dungeon`:

```rust
pub(crate) ground_items: Vec<GroundItem>,
```

Update `Character`:

```rust
pub(crate) inventory: ItemGrid,
pub(crate) stash: ItemGrid,
```

Update `Character::new`:

```rust
inventory: ItemGrid::player_starting(vec![health_potion(), health_potion(), mana_potion()]),
stash: ItemGrid::stash_starting(),
```

In `src/dungeon_gen.rs`, add `ground_items: Vec::new(),` to every `Dungeon` literal.

- [ ] **Step 4: Update test helpers that create Dungeon literals**

In `src/tests.rs`, update `open_test_dungeon` to include:

```rust
ground_items: Vec::new(),
```

Update JSON save fixtures in tests by replacing `"inventory": []` and `"stash": []` with:

```json
"inventory": {"columns": 4, "rows": 4, "items": []},
"stash": {"columns": 8, "rows": 8, "items": []}
```

Add `"ground_items": []` to dungeon JSON fixtures that include active dungeon state.

- [ ] **Step 5: Run focused tests and full tests**

Run:

```bash
cargo test item_grid_capacity_add_remove_and_auto_compaction
cargo test new_character_uses_starting_bag_and_stash_grids
cargo test dungeon_starts_without_ground_items
cargo test
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/model.rs src/items.rs src/dungeon_gen.rs src/tests.rs
git commit -m "Add grid-backed item containers"
```

## Task 2: Add Bag Upgrade Project Chain

**Files:**
- Modify: `src/model.rs`
- Modify: `src/town_projects.rs`
- Modify: `src/tests.rs`
- Modify: `design.md`

- [ ] **Step 1: Write failing capacity upgrade tests**

Add tests near town project tests:

```rust
#[test]
fn bag_dimensions_follow_quartermaster_project_chain() {
    let mut c = test_character();

    assert_eq!(bag_dimensions(&c), (4, 4));

    complete_project_for_test(&mut c, TownProject::StorehouseShelves);
    assert_eq!(bag_dimensions(&c), (5, 4));

    complete_project_for_test(&mut c, TownProject::PackHooks);
    assert_eq!(bag_dimensions(&c), (5, 5));

    complete_project_for_test(&mut c, TownProject::OilclothSatchel);
    assert_eq!(bag_dimensions(&c), (6, 5));

    complete_project_for_test(&mut c, TownProject::QuartermasterLedger);
    assert_eq!(bag_dimensions(&c), (6, 6));

    complete_project_for_test(&mut c, TownProject::ReinforcedPack);
    assert_eq!(bag_dimensions(&c), (7, 6));

    complete_project_for_test(&mut c, TownProject::StitchedPockets);
    assert_eq!(bag_dimensions(&c), (7, 7));

    complete_project_for_test(&mut c, TownProject::DeepRucksack);
    assert_eq!(bag_dimensions(&c), (8, 7));

    complete_project_for_test(&mut c, TownProject::ExilesTrunk);
    assert_eq!(bag_dimensions(&c), (8, 8));
}

#[test]
fn completing_bag_project_resizes_inventory_grid() {
    let mut c = test_character();
    c.gold = 200;

    let message = complete_town_project(&mut c, TownProject::StorehouseShelves);

    assert_eq!(message, "Completed project: Storehouse Shelves.");
    assert_eq!((c.inventory.columns, c.inventory.rows), (5, 4));
    assert_eq!(c.inventory.capacity(), 20);
}

#[test]
fn bag_project_chain_locks_until_previous_upgrade_is_complete() {
    let c = test_character();

    assert_eq!(
        town_project_availability(&c, TownProject::PackHooks),
        ProjectAvailability::Locked("Requires Storehouse Shelves.")
    );
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test bag_dimensions_follow_quartermaster_project_chain
cargo test completing_bag_project_resizes_inventory_grid
cargo test bag_project_chain_locks_until_previous_upgrade_is_complete
```

Expected: compile failure for missing `TownProject` variants and `bag_dimensions`.

- [ ] **Step 3: Add project variants and definitions**

In `src/model.rs`, extend `TownProject`:

```rust
StorehouseShelves,
PackHooks,
OilclothSatchel,
QuartermasterLedger,
ReinforcedPack,
StitchedPockets,
DeepRucksack,
ExilesTrunk,
```

In `src/town_projects.rs`, replace the single Storehouse definition with the full Quartermaster line:

```rust
TownProjectDefinition {
    project: TownProject::StorehouseShelves,
    group: "Quartermaster",
    name: "Storehouse Shelves",
    cost: 200,
    benefit: "Expand the bag to 5 x 4.",
},
TownProjectDefinition {
    project: TownProject::PackHooks,
    group: "Quartermaster",
    name: "Pack Hooks",
    cost: 350,
    benefit: "Expand the bag to 5 x 5.",
},
TownProjectDefinition {
    project: TownProject::OilclothSatchel,
    group: "Quartermaster",
    name: "Oilcloth Satchel",
    cost: 500,
    benefit: "Expand the bag to 6 x 5.",
},
TownProjectDefinition {
    project: TownProject::QuartermasterLedger,
    group: "Quartermaster",
    name: "Quartermaster Ledger",
    cost: 700,
    benefit: "Expand the bag to 6 x 6.",
},
TownProjectDefinition {
    project: TownProject::ReinforcedPack,
    group: "Quartermaster",
    name: "Reinforced Pack",
    cost: 950,
    benefit: "Expand the bag to 7 x 6.",
},
TownProjectDefinition {
    project: TownProject::StitchedPockets,
    group: "Quartermaster",
    name: "Stitched Pockets",
    cost: 1200,
    benefit: "Expand the bag to 7 x 7.",
},
TownProjectDefinition {
    project: TownProject::DeepRucksack,
    group: "Quartermaster",
    name: "Deep Rucksack",
    cost: 1500,
    benefit: "Expand the bag to 8 x 7.",
},
TownProjectDefinition {
    project: TownProject::ExilesTrunk,
    group: "Quartermaster",
    name: "Exile's Trunk",
    cost: 1900,
    benefit: "Expand the bag to 8 x 8.",
},
```

- [ ] **Step 4: Add dimension and completion helpers**

In `src/town_projects.rs`, add:

```rust
pub(crate) fn bag_dimensions(c: &Character) -> (u16, u16) {
    let mut dimensions = (STARTING_BAG_COLUMNS, STARTING_BAG_ROWS);
    for (project, upgraded) in BAG_UPGRADE_PROJECTS {
        if has_completed_project(c, *project) {
            dimensions = *upgraded;
        }
    }
    dimensions
}

pub(crate) const BAG_UPGRADE_PROJECTS: &[(TownProject, (u16, u16))] = &[
    (TownProject::StorehouseShelves, (5, 4)),
    (TownProject::PackHooks, (5, 5)),
    (TownProject::OilclothSatchel, (6, 5)),
    (TownProject::QuartermasterLedger, (6, 6)),
    (TownProject::ReinforcedPack, (7, 6)),
    (TownProject::StitchedPockets, (7, 7)),
    (TownProject::DeepRucksack, (8, 7)),
    (TownProject::ExilesTrunk, (8, 8)),
];

fn previous_bag_project(project: TownProject) -> Option<TownProject> {
    let index = BAG_UPGRADE_PROJECTS
        .iter()
        .position(|(candidate, _)| *candidate == project)?;
    index
        .checked_sub(1)
        .map(|previous| BAG_UPGRADE_PROJECTS[previous].0)
}

fn is_bag_upgrade_project(project: TownProject) -> bool {
    BAG_UPGRADE_PROJECTS
        .iter()
        .any(|(candidate, _)| *candidate == project)
}

fn resize_bag_for_projects(c: &mut Character) {
    let (columns, rows) = bag_dimensions(c);
    c.inventory.columns = columns;
    c.inventory.rows = rows;
}
```

Update `town_project_availability` so `StorehouseShelves` is available, later bag projects require the previous bag project, and non-bag project logic remains unchanged. Add this helper:

```rust
fn bag_project_lock_reason(project: TownProject) -> Option<&'static str> {
    match project {
        TownProject::PackHooks => Some("Requires Storehouse Shelves."),
        TownProject::OilclothSatchel => Some("Requires Pack Hooks."),
        TownProject::QuartermasterLedger => Some("Requires Oilcloth Satchel."),
        TownProject::ReinforcedPack => Some("Requires Quartermaster Ledger."),
        TownProject::StitchedPockets => Some("Requires Reinforced Pack."),
        TownProject::DeepRucksack => Some("Requires Stitched Pockets."),
        TownProject::ExilesTrunk => Some("Requires Deep Rucksack."),
        _ => None,
    }
}
```

Then add this match arm before the non-bag project arms:

```rust
project if is_bag_upgrade_project(project) => {
    if let Some(previous) = previous_bag_project(project) {
        if has_completed_project(c, previous) {
            ProjectAvailability::Available
        } else {
            ProjectAvailability::Locked(
                bag_project_lock_reason(project).expect("bag project has lock reason"),
            )
        }
    } else {
        ProjectAvailability::Available
    }
}
```

Update `complete_town_project` after pushing completion:

```rust
if is_bag_upgrade_project(project) {
    resize_bag_for_projects(c);
}
```

- [ ] **Step 5: Update design.md with final names and costs**

In `design.md`, replace the generic capacity line with the selected project chain and costs from this task.

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test bag_dimensions_follow_quartermaster_project_chain
cargo test completing_bag_project_resizes_inventory_grid
cargo test bag_project_chain_locks_until_previous_upgrade_is_complete
cargo test
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/model.rs src/town_projects.rs src/tests.rs design.md
git commit -m "Add bag capacity project chain"
```

## Task 3: Make Existing Inventory Mutations Capacity-Safe

**Files:**
- Modify: `src/inventory.rs`
- Modify: `src/town.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing capacity mutation tests**

Add:

```rust
#[test]
fn equipping_when_bag_is_full_keeps_selected_item_if_old_gear_cannot_fit() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(1, 1, vec![crude_axe()]);

    let result = equip_or_use_inventory_item(&mut c, 0);

    assert_eq!(result.message, "Need one free bag cell to swap equipment.");
    assert!(!result.spent_turn);
    assert!(c.equipped_weapon.name.starts_with("Rusted Sword"));
    assert!(c.inventory[0].name.starts_with("Crude Axe"));
}

#[test]
fn stash_move_requires_destination_capacity() {
    let mut from = ItemGrid::new(2, 1, vec![health_potion()]);
    let mut to = ItemGrid::new(1, 1, vec![mana_potion()]);

    let message = move_selected(&mut from, &mut to, 0, "Stored");

    assert_eq!(message, "No room in destination.");
    assert_eq!(from.len(), 1);
    assert_eq!(to.len(), 1);
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test equipping_when_bag_is_full_keeps_selected_item_if_old_gear_cannot_fit
cargo test stash_move_requires_destination_capacity
```

Expected: failure because current equip and move paths do not guard destination capacity.

- [ ] **Step 3: Update equip swap logic**

In `src/inventory.rs`, before removing selected gear, guard swaps:

```rust
let selected = c.inventory.get(index).cloned();
let Some(selected) = selected else {
    return InventoryActionResult::free("No item in that slot.");
};
if matches!(selected.kind, ItemKind::Weapon | ItemKind::Armor | ItemKind::Shield)
    && !c.inventory.has_space()
{
    return InventoryActionResult::free("Need one free bag cell to swap equipment.");
}
let selected = c.inventory.remove(index);
```

When pushing old gear back, use checked push:

```rust
if !c.inventory.push(old) {
    c.inventory.insert(index, selected);
    return InventoryActionResult::free("Need one free bag cell to swap equipment.");
}
```

Keep resource clamping after successful gear changes.

- [ ] **Step 4: Update stash move helper**

Change `move_selected` in `src/town.rs` to accept `ItemGrid`:

```rust
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
    } else if !to.has_space() {
        "No room in destination.".to_string()
    } else {
        let item = from.remove(index);
        let msg = format!("{} {}.", verb, item.name);
        let added = to.push(item);
        debug_assert!(added);
        msg
    }
}
```

- [ ] **Step 5: Replace direct full-unsafe inventory pushes in non-loot code**

Audit with:

```bash
rg -n "inventory\\.push|stash\\.push" src
```

For non-loot paths where capacity must be respected, use `push` return values and user-facing messages. The required replacements in this task are equip swaps, socket gem removal, socket replacement, stash transfers, merchant buy paths if present, and any direct town service moves. Loot paths are handled in Task 4.

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test equipping_when_bag_is_full_keeps_selected_item_if_old_gear_cannot_fit
cargo test stash_move_requires_destination_capacity
cargo test
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/inventory.rs src/town.rs src/tests.rs
git commit -m "Respect inventory grid capacity in item actions"
```

## Task 4: Add Ground Loot Storage And Capacity-Aware Drops

**Files:**
- Modify: `src/dungeon.rs`
- Modify: `src/model.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing ground-loot tests**

Add:

```rust
#[test]
fn dungeon_loot_goes_to_ground_when_inventory_is_full() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(1, 1, vec![health_potion()]);
    let mut d = open_test_dungeon(2, 2, Vec::new());
    let item = mana_potion();

    let added_to_bag = add_loot_to_bag_or_ground(&mut c, &mut d, item, 2, 2, "Dropped");

    assert!(!added_to_bag);
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (2, 2));
    assert!(matches!(d.ground_items[0].item.kind, ItemKind::ManaPotion));
}

#[test]
fn dungeon_loot_goes_to_bag_when_inventory_has_space() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(2, 1, vec![health_potion()]);
    let mut d = open_test_dungeon(2, 2, Vec::new());

    let added_to_bag = add_loot_to_bag_or_ground(&mut c, &mut d, mana_potion(), 2, 2, "Dropped");

    assert!(added_to_bag);
    assert_eq!(c.inventory.len(), 2);
    assert!(d.ground_items.is_empty());
}

#[test]
fn dropping_inventory_item_in_dungeon_creates_ground_item() {
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(4, 5, Vec::new()));
    let starting_len = c.inventory.len();

    let result = drop_selected_inventory_item(&mut c, 0);

    assert!(result.spent_turn);
    assert_eq!(c.inventory.len(), starting_len - 1);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.ground_items.len(), 1);
    assert_eq!((d.ground_items[0].x, d.ground_items[0].y), (4, 5));
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test dungeon_loot_goes_to_ground_when_inventory_is_full
cargo test dungeon_loot_goes_to_bag_when_inventory_has_space
cargo test dropping_inventory_item_in_dungeon_creates_ground_item
```

Expected: compile failure for missing `add_loot_to_bag_or_ground` and no dungeon drop behavior.

- [ ] **Step 3: Add ground loot helpers**

In `src/dungeon.rs`, add:

```rust
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
    let name = colored_item_name(&item);
    match c.inventory.try_push(item) {
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
```

- [ ] **Step 4: Route monster, gem, chest, and boss drops through helper**

In `resolve_enemy_death`, capture boss coordinates before borrowing ends:

```rust
let death_x = enemy.x;
let death_y = enemy.y;
```

Use `add_loot_to_bag_or_ground(c, d, loot, death_x, death_y, "Boss reward dropped")` and the same helper for boss gems.

In `maybe_drop_loot_in_dungeon`, get the enemy tile:

```rust
let (drop_x, drop_y) = d
    .enemies
    .get(enemy_index)
    .map(|enemy| (enemy.x, enemy.y))
    .unwrap_or((d.player_x, d.player_y));
```

Use the helper for equipment/potion loot and gem loot.

In `open_chest_on_player`, use the chest tile:

```rust
add_loot_to_bag_or_ground(c, d, loot, chest.x, chest.y, "Opened chest");
```

Keep gold collection before item insertion.

- [ ] **Step 5: Update inventory drop behavior**

In `drop_selected_inventory_item`, when `c.active_dungeon` exists, remove the item and push it to `d.ground_items` at `(d.player_x, d.player_y)`. Use this message:

```rust
InventoryActionResult::spent(format!("Dropped {} on the ground.", item.name))
```

Town drops keep the existing remove-and-delete behavior:

```rust
InventoryActionResult::spent(format!("Dropped {}.", item.name))
```

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test dungeon_loot_goes_to_ground_when_inventory_is_full
cargo test dungeon_loot_goes_to_bag_when_inventory_has_space
cargo test dropping_inventory_item_in_dungeon_creates_ground_item
cargo test
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/dungeon.rs src/inventory.rs src/model.rs src/tests.rs
git commit -m "Preserve full-inventory loot on ground"
```

## Task 5: Render Ground Items And Add Pickup Command

**Files:**
- Modify: `src/dungeon.rs`
- Modify: `src/input.rs`
- Modify: `src/tests.rs`
- Modify: `README.md`
- Modify: `design.md`

- [ ] **Step 1: Write failing pickup and rendering tests**

Add:

```rust
#[test]
fn dungeon_map_renders_ground_item_glyph() {
    let mut d = open_test_dungeon(1, 1, Vec::new());
    d.ground_items.push(GroundItem {
        x: 3,
        y: 4,
        item: health_potion(),
    });

    let lines = dungeon_map_lines_for_test(&d);

    assert_eq!(lines[4].chars().nth(3), Some('!'));
}

#[test]
fn pickup_ground_item_adds_to_inventory_when_space_exists() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(4, 4, Vec::new());
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: mana_potion(),
    });
    c.active_dungeon = Some(d);

    assert!(pickup_ground_items_on_player(&mut c));

    assert_eq!(c.inventory.len(), 1);
    assert!(c.active_dungeon.as_ref().unwrap().ground_items.is_empty());
}

#[test]
fn pickup_ground_item_keeps_item_on_ground_when_inventory_is_full() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(1, 1, vec![health_potion()]);
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem {
        x: 2,
        y: 2,
        item: mana_potion(),
    });
    c.active_dungeon = Some(d);

    assert!(!pickup_ground_items_on_player(&mut c));

    assert_eq!(c.inventory.len(), 1);
    assert_eq!(c.active_dungeon.as_ref().unwrap().ground_items.len(), 1);
}
```

Expose a test helper in `src/dungeon.rs`:

```rust
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
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test dungeon_map_renders_ground_item_glyph
cargo test pickup_ground_item_adds_to_inventory_when_space_exists
cargo test pickup_ground_item_keeps_item_on_ground_when_inventory_is_full
```

Expected: compile failure for missing helpers.

- [ ] **Step 3: Render ground item glyph**

In `dungeon_map_lines`, after chest rendering and before enemies/player:

```rust
if d.ground_items.iter().any(|item| item.x == x && item.y == y) {
    ch = '!';
}
```

Update the dungeon footer legend:

```rust
tile_span('!'),
Span::raw("=loot  "),
```

Add `("g", "pickup")` to the dungeon command footer.

- [ ] **Step 4: Add pickup helper**

In `src/dungeon.rs`, add:

```rust
pub(crate) fn ground_item_indices_at_player(c: &Character) -> Vec<usize> {
    let Some(d) = c.active_dungeon.as_ref() else {
        return Vec::new();
    };
    d.ground_items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.x == d.player_x && item.y == d.player_y)
        .map(|(index, _)| index)
        .collect()
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
            log_event(&mut d.log, LogKind::Info, "Multiple items here. Choose one with the loot picker.");
        }
        return false;
    }
    if !c.inventory.has_space() {
        if let Some(d) = c.active_dungeon.as_mut() {
            log_event(&mut d.log, LogKind::Warn, "Inventory full.");
        }
        return false;
    }
    let index = indices[0];
    let d = c.active_dungeon.as_mut().expect("indices require dungeon");
    let ground_item = d.ground_items.remove(index);
    let name = colored_item_name(&ground_item.item);
    let added = c.inventory.push(ground_item.item);
    debug_assert!(added);
    log_event(&mut d.log, LogKind::Loot, format!("Picked up {name}."));
    true
}
```

- [ ] **Step 5: Wire pickup command and walk-over auto-pickup**

In `dungeon_loop`, add:

```rust
'g' | 'G' => took_turn = pickup_ground_items_on_player(c),
```

In `is_known_dungeon_command`, include `g` and `G`.

In `dungeon_action_label`, return `"Pick up"` for `g` and `G`.

In `auto_interact_tile`, call `pickup_ground_items_on_player(c)` before chest/stairs. This handles single-item walk-over pickup. Multiple item and full-bag picker behavior is added in Task 8.

- [ ] **Step 6: Update docs**

In `README.md`, add `g` to Dungeon controls:

```markdown
- `g` pick up loot on current tile
```

In `design.md`, add `g=pickup` to the controls section and ground loot glyph `!`.

- [ ] **Step 7: Run tests**

Run:

```bash
cargo test dungeon_map_renders_ground_item_glyph
cargo test pickup_ground_item_adds_to_inventory_when_space_exists
cargo test pickup_ground_item_keeps_item_on_ground_when_inventory_is_full
cargo test
```

Expected: all tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/dungeon.rs src/input.rs src/tests.rs README.md design.md
git commit -m "Add ground loot pickup command"
```

## Task 6: Add Ratatui Grid Rendering Helpers

**Files:**
- Modify: `src/inventory.rs`
- Modify: `src/ui.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing render helper tests**

Add:

```rust
#[test]
fn grid_cursor_movement_wraps_within_dimensions() {
    assert_eq!(move_grid_cursor(0, 4, 4, 'a'), 0);
    assert_eq!(move_grid_cursor(0, 4, 4, 'd'), 1);
    assert_eq!(move_grid_cursor(0, 4, 4, 's'), 4);
    assert_eq!(move_grid_cursor(15, 4, 4, 'd'), 15);
    assert_eq!(move_grid_cursor(15, 4, 4, 's'), 15);
    assert_eq!(move_grid_cursor(5, 4, 4, 'w'), 1);
}

#[test]
fn inventory_cell_label_shows_item_kind_or_empty_cell() {
    let mut grid = ItemGrid::new(2, 2, vec![health_potion(), rusted_sword()]);

    assert_eq!(inventory_cell_label(&grid, 0), "H");
    assert_eq!(inventory_cell_label(&grid, 1), "W");
    assert_eq!(inventory_cell_label(&grid, 2), ".");

    grid.push(mana_potion());
    assert_eq!(inventory_cell_label(&grid, 2), "M");
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test grid_cursor_movement_wraps_within_dimensions
cargo test inventory_cell_label_shows_item_kind_or_empty_cell
```

Expected: compile failure for missing helpers.

- [ ] **Step 3: Add cursor and label helpers**

In `src/inventory.rs`, add:

```rust
pub(crate) fn move_grid_cursor(selected: usize, columns: u16, rows: u16, key: char) -> usize {
    let columns = usize::from(columns);
    let rows = usize::from(rows);
    let capacity = columns * rows;
    if capacity == 0 {
        return 0;
    }
    let col = selected % columns;
    let row = selected / columns;
    match key {
        'w' | 'W' if row > 0 => selected - columns,
        's' | 'S' if row + 1 < rows => selected + columns,
        'a' | 'A' if col > 0 => selected - 1,
        'd' | 'D' if col + 1 < columns => selected + 1,
        _ => selected.min(capacity - 1),
    }
}

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

pub(crate) fn clamp_grid_cursor(selected: &mut usize, grid: &ItemGrid) {
    let capacity = grid.capacity();
    if capacity == 0 {
        *selected = 0;
    } else if *selected >= capacity {
        *selected = capacity - 1;
    }
}
```

- [ ] **Step 4: Add item detail lines without ANSI**

In `src/inventory.rs`, add ratatui-friendly detail helpers:

```rust
pub(crate) fn selected_item_detail_lines(c: &Character, item: Option<&Item>) -> Vec<Line<'static>> {
    let Some(item) = item else {
        return vec![
            Line::styled("Empty cell", Style::default().fg(Color::DarkGray)),
            Line::from(format!(
                "Bag: {}/{}",
                c.inventory.len(),
                c.inventory.capacity()
            )),
        ];
    };
    let mut lines = vec![
        Line::styled(item.name.clone(), Style::default().fg(rarity_color(&item.rarity)).add_modifier(Modifier::BOLD)),
        Line::from(format!("{:?} | {} | value {}", item.kind, rarity_name(&item.rarity), item.value)),
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
```

Add missing imports to `src/inventory.rs`:

```rust
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
```

- [ ] **Step 5: Map left and right arrows**

In `src/input.rs`, update `read_key_char_with_navigation`:

```rust
KeyCode::Left if navigation => return Ok('a'),
KeyCode::Right if navigation => return Ok('d'),
```

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test grid_cursor_movement_wraps_within_dimensions
cargo test inventory_cell_label_shows_item_kind_or_empty_cell
cargo test
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/inventory.rs src/input.rs src/tests.rs
git commit -m "Add inventory grid rendering helpers"
```

## Task 7: Replace Inventory Screen With Ratatui Grid

**Files:**
- Modify: `src/inventory.rs`
- Modify: `src/main.rs`
- Modify: `src/dungeon.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing inventory render test**

Add:

```rust
#[test]
fn inventory_render_lines_include_grid_capacity_and_selected_details() {
    let c = test_character();
    let lines = inventory_screen_text_for_test(&c, 0, "");
    let rendered = lines.join("\n");

    assert!(rendered.contains("Inventory - Bag 4 x 4 - 3 / 16"));
    assert!(rendered.contains("[H]"));
    assert!(rendered.contains("Lesser Health Potion"));
    assert!(rendered.contains("Enter=equip/use"));
}
```

- [ ] **Step 2: Run test and verify failure**

Run:

```bash
cargo test inventory_render_lines_include_grid_capacity_and_selected_details
```

Expected: compile failure for missing test helper.

- [ ] **Step 3: Add render function and test helper**

In `src/inventory.rs`, add:

```rust
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
    let details = Paragraph::new(selected_item_detail_lines(c, c.inventory.get(selected)))
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

fn render_item_grid(frame: &mut Frame, grid: &ItemGrid, selected: usize, area: Rect, title: &str) {
    let mut lines = Vec::new();
    for row in 0..grid.rows {
        let mut spans = Vec::new();
        for col in 0..grid.columns {
            let index = usize::from(row) * usize::from(grid.columns) + usize::from(col);
            let label = inventory_cell_label(grid, index);
            let style = if index == selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
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
pub(crate) fn inventory_screen_text_for_test(c: &Character, selected: usize, message: &str) -> Vec<String> {
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
        selected_item_detail_lines(c, c.inventory.get(selected))
            .into_iter()
            .map(|line| line.spans.into_iter().map(|span| span.content.to_string()).collect()),
    );
    if !message.is_empty() {
        lines.push(message.to_string());
    }
    lines.push("Enter=equip/use".to_string());
    lines
}
```

- [ ] **Step 4: Replace inventory event loop**

Change `inventory_screen` signature to:

```rust
pub(crate) fn inventory_screen(
    c: &mut Character,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<bool>
```

Use this loop:

```rust
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
```

Import `anyhow::Context` through `crate::*` already available from `main.rs`.

- [ ] **Step 5: Update callers**

In `src/main.rs`, replace:

```rust
run_legacy_screen(terminal, || inventory_screen(character))?;
```

with:

```rust
inventory_screen(character, terminal)?;
clear_after_legacy_screen(terminal)?;
```

In `src/dungeon.rs`, replace:

```rust
'i' | 'I' => took_turn = run_legacy_screen(terminal, || inventory_screen(c))?,
```

with:

```rust
'i' | 'I' => took_turn = inventory_screen(c, terminal)?,
```

Do not use `run_legacy_screen` for this screen; this is a ratatui screen and raw mode should remain owned by ratatui.

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test inventory_render_lines_include_grid_capacity_and_selected_details
cargo test
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/inventory.rs src/main.rs src/dungeon.rs src/tests.rs
git commit -m "Render inventory as ratatui grid"
```

## Task 8: Replace Stash Screen With Ratatui Grids

**Files:**
- Modify: `src/town.rs`
- Modify: `src/main.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing stash render and move tests**

Add:

```rust
#[test]
fn stash_render_lines_include_both_grid_capacities() {
    let c = test_character();
    let lines = stash_screen_text_for_test(&c, StashSide::Inventory, 0, 0, "");
    let rendered = lines.join("\n");

    assert!(rendered.contains("Inventory 3 / 16"));
    assert!(rendered.contains("Stash 0 / 64"));
    assert!(rendered.contains("Tab=switch"));
}
```

- [ ] **Step 2: Run test and verify failure**

Run:

```bash
cargo test stash_render_lines_include_both_grid_capacities
```

Expected: compile failure for missing test helper or signature changes.

- [ ] **Step 3: Add stash render helpers**

In `src/town.rs`, make `StashSide` derive `Debug` if useful and add:

```rust
pub(crate) fn render_stash_screen(
    frame: &mut Frame,
    c: &Character,
    side: StashSide,
    inv_selected: usize,
    stash_selected: usize,
    message: &str,
) {
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(3),
    ])
    .split(frame.area());
    let title = Paragraph::new(format!(
        "Stash - Inventory {} / {} - Stash {} / {}",
        c.inventory.len(),
        c.inventory.capacity(),
        c.stash.len(),
        c.stash.capacity()
    ))
    .block(Block::default().borders(Borders::ALL).title("Stash"));
    frame.render_widget(title, layout[0]);

    let body = Layout::horizontal([Constraint::Min(24), Constraint::Min(24), Constraint::Length(38)])
        .split(layout[1]);
    render_item_grid(frame, &c.inventory, inv_selected, body[0], "Inventory");
    render_item_grid(frame, &c.stash, stash_selected, body[1], "Stash");
    let selected_item = match side {
        StashSide::Inventory => c.inventory.get(inv_selected),
        StashSide::Stash => c.stash.get(stash_selected),
    };
    frame.render_widget(
        Paragraph::new(selected_item_detail_lines(c, selected_item))
            .block(Block::default().borders(Borders::ALL).title("Details")),
        body[2],
    );

    let footer = if message.is_empty() {
        "Tab=switch  WASD/Arrows=move  Enter=transfer  Esc=back".to_string()
    } else {
        format!("{message}\nTab=switch  WASD/Arrows=move  Enter=transfer  Esc=back")
    };
    frame.render_widget(
        Paragraph::new(footer).block(Block::default().borders(Borders::ALL).title("Commands")),
        layout[2],
    );
}
```

Make `render_item_grid` and `selected_item_detail_lines` public within crate from `src/inventory.rs`.

Add `stash_screen_text_for_test` mirroring the title/footer strings used above.

- [ ] **Step 4: Replace stash event loop**

Change signature:

```rust
pub(crate) fn stash_menu(
    c: &mut Character,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<()>
```

Use ratatui draw loop with `read_key_char_nav()`. Cursor movement uses `move_grid_cursor` on the active grid. `Tab` switches side. `Enter` calls `move_selected`. `Esc` returns `Ok(())`.

- [ ] **Step 5: Update caller**

In `src/main.rs`, replace:

```rust
run_legacy_screen(terminal, || stash_menu(character))?;
```

with:

```rust
stash_menu(character, terminal)?;
clear_after_legacy_screen(terminal)?;
```

If `clear_after_legacy_screen` is renamed later, call the terminal clear helper directly. Do not use `run_legacy_screen`.

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test stash_render_lines_include_both_grid_capacities
cargo test stash_move_selected_moves_requested_item_immediately
cargo test stash_move_requires_destination_capacity
cargo test
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/town.rs src/main.rs src/inventory.rs src/tests.rs
git commit -m "Render stash as ratatui grids"
```

## Task 9: Add Ground-Loot Picker

**Files:**
- Modify: `src/dungeon.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing picker action tests**

Add:

```rust
#[test]
fn ground_loot_picker_pickup_removes_selected_item() {
    let mut c = test_character();
    c.inventory = ItemGrid::new(4, 4, Vec::new());
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem { x: 2, y: 2, item: health_potion() });
    d.ground_items.push(GroundItem { x: 2, y: 2, item: mana_potion() });
    c.active_dungeon = Some(d);

    let message = pick_up_ground_item_by_tile_index(&mut c, 1);

    assert_eq!(message, "Picked up Lesser Mana Potion (restores 15% mana).");
    assert_eq!(c.inventory.len(), 1);
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.ground_items.len(), 1);
    assert!(matches!(d.ground_items[0].item.kind, ItemKind::HealthPotion));
}

#[test]
fn ground_loot_picker_discard_removes_only_selected_item() {
    let mut c = test_character();
    let mut d = open_test_dungeon(2, 2, Vec::new());
    d.ground_items.push(GroundItem { x: 2, y: 2, item: health_potion() });
    d.ground_items.push(GroundItem { x: 2, y: 2, item: mana_potion() });
    c.active_dungeon = Some(d);

    let message = discard_ground_item_by_tile_index(&mut c, 0);

    assert_eq!(message, "Discarded Lesser Health Potion (restores 15% HP).");
    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(d.ground_items.len(), 1);
    assert!(matches!(d.ground_items[0].item.kind, ItemKind::ManaPotion));
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test ground_loot_picker_pickup_removes_selected_item
cargo test ground_loot_picker_discard_removes_only_selected_item
```

Expected: compile failure for missing action helpers.

- [ ] **Step 3: Add action helpers**

In `src/dungeon.rs`, add:

```rust
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
```

- [ ] **Step 4: Add ratatui picker screen**

Add:

```rust
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
            .map(|d| ground_item_indices_on_player_tile(d).len())
            .unwrap_or_default();
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
                    return Ok(true);
                }
            }
            'd' | 'D' => {
                message = discard_ground_item_by_tile_index(c, selected);
                if item_count <= 1 {
                    return Ok(false);
                }
            }
            _ => message = "Unknown loot command.".to_string(),
        }
    }
}
```

Render function uses a list on the left and `selected_item_detail_lines` on the right, matching the spec.

- [ ] **Step 5: Open picker for multi-item and full-bag cases**

Change `pickup_ground_items_on_player` so:

- zero items logs "There is no loot here." and returns false.
- one item and bag has space auto-picks.
- one item and bag full logs "Inventory full. Choose loot to inspect or discard." and returns false.
- multiple items logs "Multiple items here. Choose loot." and returns false.

In `dungeon_loop`, for `g`, if `pickup_ground_items_on_player(c)` returns false and there are items on the tile, call `ground_loot_picker(c, terminal)?`. The picker result should be the action's `took_turn`.

In `auto_interact_tile`, only auto-pick one item when space exists. Do not open picker during movement; log that multiple items are here or the inventory is full.

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test ground_loot_picker_pickup_removes_selected_item
cargo test ground_loot_picker_discard_removes_only_selected_item
cargo test
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/dungeon.rs src/tests.rs
git commit -m "Add ground loot picker"
```

## Task 10: Final Documentation, Save Reset, And Guard

**Files:**
- Modify: `README.md`
- Modify: `design.md`
- Modify: `docs/superpowers/specs/2026-05-16-inventory-grid-ui-design.md` if implementation decisions changed.

- [ ] **Step 1: Update README**

In `README.md`, update current features to mention:

```markdown
- Grid-based bag and stash inventory with capacity upgrades
- Ground loot pickup with `g` and automatic pickup on walk-over
```

In Save Files, add this paragraph and command block:

````markdown
The inventory grid rework changes save shape. During local development, reset old saves with:

```sh
cargo run -- reset-save
```
````

- [ ] **Step 2: Update design.md**

Ensure `design.md` reflects the final Quartermaster project names, `g` pickup, `!` loot glyph, `4 x 4` starting bag, `8 x 8` bag cap, `8 x 8` stash, ratatui inventory/stash/picker screens, and old-save reset expectation.

- [ ] **Step 3: Run required guard**

Run:

```bash
scripts/agent-commit-guard.sh --fix
```

Expected: `cargo fmt`, `cargo test`, and `cargo check` all pass.

- [ ] **Step 4: Review final diff**

Run:

```bash
git status --short
git diff
```

Confirm only files changed for the inventory grid implementation are present. Do not stage unrelated user changes.

- [ ] **Step 5: Commit final docs**

```bash
git add README.md design.md docs/superpowers/specs/2026-05-16-inventory-grid-ui-design.md
git commit -m "Document grid inventory implementation"
```

If there are no documentation changes beyond previous tasks, skip this commit and record that the required guard already passed in the final response.

## Self-Review Notes

Spec coverage:

- Grid inventory and stash are covered by Tasks 1, 6, 7, and 8.
- One-cell auto-compacting storage is covered by Tasks 1 and 3.
- Capacity upgrades through Quartermaster projects are covered by Task 2.
- Ground loot and full-bag preservation are covered by Tasks 4, 5, and 9.
- Ratatui preference is covered by Tasks 7, 8, and 9.
- Documentation and old-save reset expectation are covered by Task 10.

Implementation sequencing:

- Task 1 deliberately keeps Vec-like methods on `ItemGrid` so the repo can compile while usage migrates.
- Task 3 makes existing mutation paths capacity-safe before loot capacity is enforced.
- Tasks 7 and 8 remove legacy inventory/stash usage after the data model is stable.
