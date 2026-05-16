# Inventory Grid UI Design

## Overview

Rework inventory from a list into ratatui grid screens. The player bag and stash should feel like spatial containers without becoming a manual packing puzzle. Every item occupies one cell, grids auto-compact after item changes, and selected item details appear in a right-side panel.

Old saves can break for this change because inventory, stash, and active dungeon ground loot changed save shape. Existing local saves should be reset during development with `cargo run -- reset-save`.

## Goals

- Replace the list-like inventory with a visible grid and cursor selection.
- Use ratatui for the implemented inventory, stash, and ground-loot picker screens.
- Keep item management fast by avoiding manual item rearrangement.
- Make inventory capacity a progression system through Quartermaster town projects.
- Prevent loot loss when the bag is full by adding ground items.
- Keep routine item actions on single keypresses with no pause prompts.

## Non-Goals

- No manual item rearrangement.
- No item stacking.
- No inventory tabs in the first pass.
- No stash tabs in the first pass.
- No save migration from the old `Vec<Item>` inventory and stash format.
- No full conversion of sell, salvage, or socket screens to grid UIs unless needed for correctness.

## UI Design

### Player Inventory

The inventory screen is a ratatui screen with a grid on the left and item details on the right.

- The bag starts at `4 x 4`.
- The bag caps at `8 x 8`.
- Every item occupies exactly one cell.
- Empty cells are visible.
- The cursor moves through cells using `WASD` and arrow keys.
- `Enter` equips gear or uses consumables.
- `x` drops the selected item.
- `Esc` exits the screen.
- The footer stays anchored at the bottom.

The right details panel shows the selected item's name, kind, rarity, item level, requirements, stats, value, and comparison text when applicable. Empty cells show a short empty-cell state and current capacity.

### Stash

The stash also uses a ratatui grid screen.

- The stash starts at `8 x 8`.
- The stash uses the same cursor and right details panel pattern.
- `Tab` switches active side between bag and stash.
- `Enter` moves the selected item to the other side if the destination has capacity.
- Source grids auto-compact after movement.
- Destination grids append moved items into the next empty cell.

Stash tabs are a later feature. If storage needs to grow beyond one `8 x 8` grid, add tabs instead of making a larger single grid.

### Ground-Loot Picker

Ground loot uses a ratatui picker when there is a real choice.

- Walking over one ground item auto-picks it up if the bag has space.
- Pressing `g` attempts pickup on the current tile.
- If multiple items are on the tile, open the picker.
- If the bag is full, open the picker so the player can inspect ground items and choose what to do.
- The picker shows selectable ground items on the left and item details on the right.
- `Enter` picks up the selected item if the bag has space.
- `d` discards the selected ground item permanently.
- `Esc` leaves remaining items on the ground.
- Dungeon map tiles containing ground items render with `!`.

## Data Model

Because items are one-cell and auto-compacted, the grid does not need to store per-item coordinates. The grid is a view over ordered item storage.

```rust
struct ItemGrid {
    columns: u16,
    rows: u16,
    items: Vec<Item>,
}
```

Rules:

- Capacity is `columns * rows`.
- Items render row-major.
- Index `0` is the top-left cell.
- Empty cells are implicit after `items.len()`.
- Removing an item compacts automatically because `Vec::remove` shifts later items left.
- Adding an item appends to the next empty cell when capacity exists.
- Capacity checks must happen before pickups, stash transfers, chest rewards, boss rewards, and monster loot insertion.

Character storage becomes:

```rust
inventory: ItemGrid,
stash: ItemGrid,
```

Dungeon storage gains:

```rust
struct GroundItem {
    x: i32,
    y: i32,
    item: Item,
}
```

`Dungeon` stores `ground_items: Vec<GroundItem>`.

## Capacity Progression

Bag capacity is a Quartermaster town-project progression path.

- New characters start with a `4 x 4` bag.
- Quartermaster projects expand bag dimensions in fixed steps.
- The bag cap is `8 x 8`.
- `Storehouse Shelves` should become part of this capacity progression instead of being a placeholder-only project.
- Stash starts at `8 x 8`.

Implemented bag curve:

| Stage | Project | Cost | Size | Capacity |
| --- | --- | ---: | ---: | ---: |
| Starting bag | - | - | `4 x 4` | 16 |
| Upgrade 1 | Storehouse Shelves | 200 gold | `5 x 4` | 20 |
| Upgrade 2 | Pack Hooks | 350 gold | `5 x 5` | 25 |
| Upgrade 3 | Oilcloth Satchel | 500 gold | `6 x 5` | 30 |
| Upgrade 4 | Quartermaster Ledger | 700 gold | `6 x 6` | 36 |
| Upgrade 5 | Reinforced Pack | 950 gold | `7 x 6` | 42 |
| Upgrade 6 | Stitched Pockets | 1200 gold | `7 x 7` | 49 |
| Upgrade 7 | Deep Rucksack | 1500 gold | `8 x 7` | 56 |
| Upgrade 8 | Exile's Trunk | 1900 gold | `8 x 8` | 64 |

## Loot And Pickup Behavior

Loot should not disappear because the bag is full.

- If monster loot drops and the bag has room, add it to the bag.
- If monster loot drops and the bag is full, create a `GroundItem` at or near the enemy/player tile.
- If boss loot drops and the bag is full, create a `GroundItem`, log clearly that the reward is on the ground, and retain the completed dungeon so the grounded reward remains accessible before returning to town.
- Chest gold is always collected.
- Chest items enter the bag if space exists.
- Chest items become ground loot on the chest tile if the bag is full.
- Dropping an item in town removes it from the bag.
- Dropping an item in a dungeon creates a `GroundItem` at the player tile.
- Ground item tiles render with `!` in the dungeon map.
- Dungeon command help should include `g=pickup`.

If there are multiple ground items on the same tile, the picker opens instead of guessing which item to take.

## Ratatui Preference

Inventory, stash, and ground-loot picker screens are implemented with ratatui. If a future inventory-adjacent screen cannot use ratatui, the implementation must explain the concrete blocker before falling back to a legacy ANSI screen.

The preferred direction is to avoid adding new legacy `println!` UI for this system. Existing legacy sell, salvage, and other service screens can remain list-based unless they need capacity or item-container correctness changes during implementation.

## Testing

Test coverage should include:

- `ItemGrid` capacity calculation.
- `ItemGrid` add behavior succeeds when there is space.
- `ItemGrid` add behavior fails when full.
- `ItemGrid` remove behavior auto-compacts.
- New characters start with a `4 x 4` bag and an `8 x 8` stash.
- Bag capacity upgrades change dimensions and cap at `8 x 8`.
- Stash movement respects destination capacity.
- Inventory equip/use/drop actions preserve auto-compaction.
- Dungeon drops create ground items when the bag is full.
- Dungeon item drops enter the bag when there is space.
- Chest gold is collected even when chest item loot becomes ground loot.
- Dropping an item in a dungeon creates a ground item at the player tile.
- Single ground item walk-over pickup is automatic when the bag has space.
- Multiple ground items open the picker.
- Ground-loot discard removes only the selected ground item.
- Ratatui render tests where practical cover grid cells, cursor highlight, selected-item detail text, and command footer text.

## Final Implementation Details

- Quartermaster bag upgrades use the fixed project chain and costs listed above.
- Ground loot renders with the `!` glyph.
- Inventory, stash, and ground-loot picker are ratatui screens.
- Boss reward overflow retains the completed dungeon when the reward lands on the ground, leaving the reward accessible before the player returns to town.
- Sell and salvage screens remain list-based for this pass, while item storage still uses the compacting grid containers.
- Old saves are not migrated; during local development, reset incompatible saves with `cargo run -- reset-save`.
