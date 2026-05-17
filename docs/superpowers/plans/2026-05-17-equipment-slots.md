# Equipment Slots Implementation Plan

> **For pi agents:** REQUIRED SKILL: Use `executing-plans` to implement this plan task-by-task in the current pi session. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add functional helm, gloves, boots, belt, amulet, and two ring equipment slots that can drop as loot, be equipped from inventory, affect character stats, and display in UI comparisons.
**Architecture:** Keep the current one-`Item`-per-equipped-slot model and extend it with new `ItemKind` variants plus `Character` equipped fields. Centralize slot lookup enough to avoid duplicating comparison/equip/stat/socket logic, while preserving the existing weapon/armor/shield behavior and Rogue shield restriction.
**Tech Stack:** Rust, serde save compatibility, ratatui UI, existing single-file test suite in `src/tests.rs`.

---

### Task 1: Add new gear kinds and starting empty slots

**Files:**

- Modify: `src/model.rs`
- Modify: `src/items.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Add this test near `new_warrior_matches_mvp_starting_state` in `src/tests.rs`:

```rust
#[test]
fn new_characters_start_with_empty_accessory_slots() {
    let warrior = test_character();
    assert_eq!(warrior.equipped_helm.name, "Empty Helm");
    assert_eq!(warrior.equipped_gloves.name, "Empty Gloves");
    assert_eq!(warrior.equipped_boots.name, "Empty Boots");
    assert_eq!(warrior.equipped_belt.name, "Empty Belt");
    assert_eq!(warrior.equipped_amulet.name, "Empty Amulet");
    assert_eq!(warrior.equipped_ring1.name, "Empty Ring");
    assert_eq!(warrior.equipped_ring2.name, "Empty Ring");

    let rogue = Character::new(
        "Shade".to_string(),
        CharacterClass::Rogue,
        DeathMode::Softcore,
    );
    assert_eq!(rogue.equipped_helm.kind, ItemKind::Helm);
    assert_eq!(rogue.equipped_gloves.kind, ItemKind::Gloves);
    assert_eq!(rogue.equipped_boots.kind, ItemKind::Boots);
    assert_eq!(rogue.equipped_belt.kind, ItemKind::Belt);
    assert_eq!(rogue.equipped_amulet.kind, ItemKind::Amulet);
    assert_eq!(rogue.equipped_ring1.kind, ItemKind::Ring);
    assert_eq!(rogue.equipped_ring2.kind, ItemKind::Ring);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test new_characters_start_with_empty_accessory_slots`
Expected: FAIL because `Character` does not have the new equipped fields and `ItemKind` lacks the new variants.

- [ ] **Step 3: Implement minimal code**

In `src/model.rs`, extend `ItemKind` and add `Hash` to its derives so tests and slot collections can store kinds in `HashSet`:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub(crate) enum ItemKind {
    HealthPotion,
    ManaPotion,
    Weapon,
    Armor,
    Shield,
    Helm,
    Gloves,
    Boots,
    Belt,
    Amulet,
    Ring,
    Gem,
}
```

Add serde-defaulted fields to `Character` after `equipped_shield`:

```rust
#[serde(default = "empty_helm")]
pub(crate) equipped_helm: Item,
#[serde(default = "empty_gloves")]
pub(crate) equipped_gloves: Item,
#[serde(default = "empty_boots")]
pub(crate) equipped_boots: Item,
#[serde(default = "empty_belt")]
pub(crate) equipped_belt: Item,
#[serde(default = "empty_amulet")]
pub(crate) equipped_amulet: Item,
#[serde(default = "empty_ring")]
pub(crate) equipped_ring1: Item,
#[serde(default = "empty_ring")]
pub(crate) equipped_ring2: Item,
```

Initialize those fields in both `new_warrior` and `new_rogue` with the same empty-slot functions.

In `src/items.rs`, update `item`, `item_with_rarity` consumers, and match arms so helm/gloves/boots/belt/amulet/ring are treated as armor-like gear for requirements and zero-damage summaries. Add these constructors:

```rust
pub(crate) fn empty_helm() -> Item { empty_slot("Empty Helm", ItemKind::Helm) }
pub(crate) fn empty_gloves() -> Item { empty_slot("Empty Gloves", ItemKind::Gloves) }
pub(crate) fn empty_boots() -> Item { empty_slot("Empty Boots", ItemKind::Boots) }
pub(crate) fn empty_belt() -> Item { empty_slot("Empty Belt", ItemKind::Belt) }
pub(crate) fn empty_amulet() -> Item { empty_slot("Empty Amulet", ItemKind::Amulet) }
pub(crate) fn empty_ring() -> Item { empty_slot("Empty Ring", ItemKind::Ring) }

fn empty_slot(name: &str, kind: ItemKind) -> Item {
    item_with_rarity(
        name,
        kind,
        0,
        item_stats(0, 0, 0, 0, 0),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    )
}
```

- [ ] **Step 4: Run verification**

Run: `cargo test new_characters_start_with_empty_accessory_slots`
Expected: PASS.

---

### Task 2: Make new slots affect stats and sockets

**Files:**

- Modify: `src/model.rs`
- Modify: `src/inventory.rs`
- Modify: `src/dungeon.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Add this test near the socket/stat tests in `src/tests.rs`:

```rust
#[test]
fn accessory_slots_contribute_stats_and_socket_bonuses() {
    let mut c = test_character();
    c.equipped_helm = item_with_rarity(
        "Test Helm",
        ItemKind::Helm,
        10,
        item_stats(0, 0, 2, 1, 0),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    );
    c.equipped_boots = item_with_rarity(
        "Test Boots",
        ItemKind::Boots,
        10,
        item_stats(0, 0, 0, 1, 2),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    );
    c.equipped_amulet = item_with_rarity(
        "Socketed Amulet",
        ItemKind::Amulet,
        10,
        item_stats(0, 0, 0, 0, 0),
        Rarity::Magic,
        1,
        requirements(0, 0, 0),
    );
    c.equipped_amulet.sockets = vec![Some(GemSocket::filled(GemKind::Emerald, GemTier::Pristine))];

    assert_eq!(c.effective_dexterity(), c.dexterity + 3);
    assert_eq!(c.armor(), 1 + 1 + 2 + iron_guard_armor_bonus(&c));
    assert_eq!(c.dodge_rating(), 10 + c.effective_dexterity() * 3 + 2 + 1 + 1);
    assert_eq!(c.speed(), 10 + c.effective_dexterity() * 5 + 2);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test accessory_slots_contribute_stats_and_socket_bonuses`
Expected: FAIL because the new equipped fields do not contribute to `armor`, `dodge_rating`, `speed`, or `socket_bonuses` yet.

- [ ] **Step 3: Implement minimal code**

In `src/model.rs`, add helper methods on `Character`:

```rust
pub(crate) fn equipped_defensive_items(&self) -> [&Item; 9] {
    [
        &self.equipped_armor,
        &self.equipped_shield,
        &self.equipped_helm,
        &self.equipped_gloves,
        &self.equipped_boots,
        &self.equipped_belt,
        &self.equipped_amulet,
        &self.equipped_ring1,
        &self.equipped_ring2,
    ]
}

pub(crate) fn equipped_socketed_items(&self) -> [&Item; 10] {
    [
        &self.equipped_weapon,
        &self.equipped_armor,
        &self.equipped_shield,
        &self.equipped_helm,
        &self.equipped_gloves,
        &self.equipped_boots,
        &self.equipped_belt,
        &self.equipped_amulet,
        &self.equipped_ring1,
        &self.equipped_ring2,
    ]
}
```

Use these helpers so `dodge_rating`, `speed`, `armor`, and `socket_bonuses` include armor, shield, helm, gloves, boots, belt, amulet, ring1, and ring2. Keep weapon speed contributing through the existing explicit `equipped_weapon.speed` addition.

Update any `match item.kind` arms in `src/inventory.rs` and `src/dungeon.rs` detail rendering so helm/gloves/boots/belt/amulet/ring use the armor/dodge/speed display line.

- [ ] **Step 4: Run verification**

Run: `cargo test accessory_slots_contribute_stats_and_socket_bonuses`
Expected: PASS.

---

### Task 3: Equip new slots from inventory and compare them in UI

**Files:**

- Modify: `src/inventory.rs`
- Modify: `src/town.rs`
- Modify: `src/ui.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing tests**

Add these tests near existing equip/comparison tests in `src/tests.rs`:

```rust
#[test]
fn equipping_accessory_swaps_old_item_back_to_inventory() {
    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(item_with_rarity(
        "Iron Helm",
        ItemKind::Helm,
        25,
        item_stats(0, 0, 2, 0, -1),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    ));

    let result = equip_or_use_inventory_item(&mut c, 0);

    assert!(result.spent_turn);
    assert_eq!(result.message, "Equipped Iron Helm.");
    assert_eq!(c.equipped_helm.name, "Iron Helm");
    assert_eq!(c.inventory[0].name, "Empty Helm");
}

#[test]
fn rings_fill_empty_second_slot_before_replacing_first_ring() {
    let mut c = test_character();
    c.inventory.clear();
    c.equipped_ring1 = item_with_rarity(
        "Copper Ring",
        ItemKind::Ring,
        10,
        item_stats(0, 0, 0, 1, 0),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    );
    c.inventory.push(item_with_rarity(
        "Silver Ring",
        ItemKind::Ring,
        20,
        item_stats(0, 0, 0, 2, 0),
        Rarity::Magic,
        1,
        requirements(0, 0, 0),
    ));

    equip_or_use_inventory_item(&mut c, 0);

    assert_eq!(c.equipped_ring1.name, "Copper Ring");
    assert_eq!(c.equipped_ring2.name, "Silver Ring");
    assert_eq!(c.inventory[0].name, "Empty Ring");
}

#[test]
fn inventory_accessory_equipped_panel_compares_against_matching_slot() {
    let mut c = test_character();
    c.inventory.clear();
    c.inventory.push(item_with_rarity(
        "Test Boots",
        ItemKind::Boots,
        20,
        item_stats(0, 0, 1, 3, 2),
        Rarity::Magic,
        1,
        requirements(0, 0, 0),
    ));

    let lines = selected_item_equipped_comparison_lines(&c, c.inventory.get(0))
        .iter()
        .map(line_text)
        .collect::<Vec<_>>();

    assert!(lines.iter().any(|line| line == "Equipped Boots: Empty Boots"));
    assert!(lines.iter().any(|line| line == "Delta: +1 armor  +3 dodge  +2 speed"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test accessory && cargo test rings_fill_empty_second_slot_before_replacing_first_ring`
Expected: FAIL because accessory equip, ring placement, and comparison logic is not implemented.

- [ ] **Step 3: Implement minimal code**

In `src/inventory.rs`:

- Add a helper such as `fn is_equipment_kind(kind: ItemKind) -> bool` that returns true for weapon, armor, shield, helm, gloves, boots, belt, amulet, and ring.
- Add helpers to map non-ring item kinds to equipped field labels and mutable references.
- Extend `item_comparison`, `selected_item_equipped_comparison_lines`, `gear_stat_line`, `item_level_text`, `item_summary`, `inventory_cell_label`, and `equip_or_use_inventory_item` for all new kinds.
- For rings, implement the agreed rule: equip into ring1 if it is `Empty Ring`, otherwise ring2 if it is `Empty Ring`, otherwise replace ring1.
- Keep health/mana potions and gems unchanged.

In `src/town.rs`:

- Update `shard_kind`, `shard_name`, `shard_count`, `add_shards`, `spend_shards`, salvage rejection text, and upgrade matching so new armor-like gear uses armor shards.
- Do not add blacksmith menu upgrade entries for the new slots in this pass.
- Include socket-bench equipped targets for new slots when those items have sockets, with labels like `Equipped helm` and `Equipped ring 2`.

In `src/ui.rs`:

- Add the new equipment lines to the town status panel.

- [ ] **Step 4: Run verification**

Run: `cargo test accessory && cargo test rings_fill_empty_second_slot_before_replacing_first_ring`
Expected: PASS.

---

### Task 4: Add new equipment to class loot pools

**Files:**

- Modify: `src/dungeon.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Update `rogue_random_equipment_loot_uses_rogue_item_families`, `warrior_random_equipment_loot_uses_warrior_item_families`, and `boss_reward_loot_is_always_magic_or_rare_equipment` so their accepted gear kinds include `Helm`, `Gloves`, `Boots`, `Belt`, `Amulet`, and `Ring`. Add this focused test near the loot-pool tests:

```rust
#[test]
fn random_equipment_loot_can_drop_new_equipment_slots() {
    let mut seen = std::collections::HashSet::new();
    for _ in 0..1000 {
        seen.insert(random_equipment_loot_for_class(CharacterClass::Warrior, 3, false).kind);
        seen.insert(random_equipment_loot_for_class(CharacterClass::Rogue, 3, false).kind);
    }

    for kind in [
        ItemKind::Helm,
        ItemKind::Gloves,
        ItemKind::Boots,
        ItemKind::Belt,
        ItemKind::Amulet,
        ItemKind::Ring,
    ] {
        assert!(seen.contains(&kind), "expected loot pool to drop {kind:?}");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test random_equipment_loot_can_drop_new_equipment_slots && cargo test boss_reward_loot_is_always_magic_or_rare_equipment`
Expected: FAIL because random equipment currently rolls only weapon, armor, and shield.

- [ ] **Step 3: Implement minimal code**

In `src/dungeon.rs`, extend `random_warrior_equipment_loot` and `random_rogue_equipment_loot` from 4 choices to 10 choices. Keep existing class flavor for weapons/body armor/shields, and add shared slot names with class-flavored prefixes where useful:

- Warrior: Iron Helm, Plate Gloves, March Boots, War Belt, Guard Amulet, Iron Ring.
- Rogue: Hooded Cowl, Cutpurse Gloves, Soft Boots, Utility Belt, Shadow Amulet, Silent Ring.

Use armor-like stats and requirements:

- Helm: armor-focused.
- Gloves: hit/dodge/speed-light; represent hit through requirements only for now because `Item` has no hit stat.
- Boots: dodge/speed-focused.
- Belt: armor with mild dodge penalty or neutral speed.
- Amulet: small balanced defensive stats.
- Ring: low defensive stats, no speed penalty.

All new loot should call `add_random_sockets` like existing equipment and use armor shards via `shard_kind`.

- [ ] **Step 4: Run verification**

Run: `cargo test random_equipment_loot_can_drop_new_equipment_slots && cargo test rogue_random_equipment_loot_uses_rogue_item_families && cargo test warrior_random_equipment_loot_uses_warrior_item_families && cargo test boss_reward_loot_is_always_magic_or_rare_equipment`
Expected: PASS.

---

### Task 5: Update design documentation and run full validation

**Files:**

- Modify: `design.md`
- Modify: `TODO.md`
- Test: project validation

- [ ] **Step 1: Update documentation**

In `design.md`, update the inventory/current-implementation sections to state that helm, gloves, boots, belt, amulet, and two ring slots are implemented; their stats and socket bonuses contribute to character totals; random loot can drop them; and blacksmith upgrades remain focused on weapon/armor/shield for this pass.

In `TODO.md`, remove the completed item so the file contains only the `# TODO` header if no other tasks exist.

- [ ] **Step 2: Run project guard**

Run: `scripts/agent-commit-guard.sh --fix`
Expected: PASS; this runs `cargo fmt`, `cargo test`, and `cargo check`.

- [ ] **Step 3: Review changes**

Run: `git status --short && git diff -- src/model.rs src/items.rs src/inventory.rs src/town.rs src/ui.rs src/dungeon.rs src/tests.rs design.md TODO.md docs/superpowers/plans/2026-05-17-equipment-slots.md`
Expected: only intentional files changed.

- [ ] **Step 4: Commit**

Run:

```bash
git config --local core.hooksPath .githooks
git add src/model.rs src/items.rs src/inventory.rs src/town.rs src/ui.rs src/dungeon.rs src/tests.rs design.md TODO.md docs/superpowers/plans/2026-05-17-equipment-slots.md
git commit -m "Add expanded equipment slots"
```

Expected: commit succeeds without `--no-verify`.

- [ ] **Step 5: Merge back to main and clean up**

Run:

```bash
git switch main
git merge --no-ff codex/equipment-slots
git log --oneline -1
git worktree remove .worktrees/equipment-slots
```

Expected: main contains the task commit and the task worktree is removed.
