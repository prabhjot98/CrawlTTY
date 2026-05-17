# Inventory Character Equipment Panel Implementation Plan

> **For pi agents:** REQUIRED SKILL: Use `executing-plans` to implement this plan task-by-task in the current pi session. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show equipped gear in a humanoid character panel on the inventory screen and allow Tab/WASD-or-arrow selection between bag cells and body slots.
**Architecture:** Keep `src/inventory.rs` as the owner of inventory-screen state, rendering, cursor movement, and action dispatch. Add a small inventory focus enum plus an equipment-slot cursor enum, render the bag/details/character panels from one ratatui layout, and keep bag equip/use/drop behavior unchanged unless the bag pane is active.
**Tech Stack:** Rust, ratatui, existing `ItemGrid`, existing UI palette and cursor helpers, cargo tests.

---

### Task 1: Add inventory/body selection model tests

**Files:**
- Modify: `src/tests.rs`
- Modify: `src/inventory.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn equipment_cursor_moves_through_humanoid_body_slots() {
    assert_eq!(
        move_equipment_cursor(CharacterEquipmentSlot::Armor, 'w'),
        CharacterEquipmentSlot::Amulet
    );
    assert_eq!(
        move_equipment_cursor(CharacterEquipmentSlot::Armor, 'a'),
        CharacterEquipmentSlot::Weapon
    );
    assert_eq!(
        move_equipment_cursor(CharacterEquipmentSlot::Armor, 'd'),
        CharacterEquipmentSlot::Shield
    );
    assert_eq!(
        move_equipment_cursor(CharacterEquipmentSlot::Armor, 's'),
        CharacterEquipmentSlot::Belt
    );
    assert_eq!(
        move_equipment_cursor(CharacterEquipmentSlot::Boots, 's'),
        CharacterEquipmentSlot::Boots
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test equipment_cursor_moves_through_humanoid_body_slots`
Expected: FAIL because `CharacterEquipmentSlot` and `move_equipment_cursor` do not exist.

- [ ] **Step 3: Implement minimal code**

Add `CharacterEquipmentSlot`, slot coordinate helpers, and `move_equipment_cursor` in `src/inventory.rs`.

- [ ] **Step 4: Run verification**

Run: `cargo test equipment_cursor_moves_through_humanoid_body_slots`
Expected: PASS.

### Task 2: Render humanoid character panel and active inventory focus

**Files:**
- Modify: `src/tests.rs`
- Modify: `src/inventory.rs`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn inventory_text_includes_character_equipment_panel_and_tab_command() {
    let c = test_character();
    let lines = inventory_screen_text_for_test(&c, 0, CharacterEquipmentSlot::Armor, InventoryFocus::Bag, "");
    let rendered = lines.join("\n");

    assert!(rendered.contains("Character"));
    assert!(rendered.contains("Helm"));
    assert!(rendered.contains("Weapon"));
    assert!(rendered.contains("Armor"));
    assert!(rendered.contains("Rusted Sword"));
    assert!(rendered.contains("Cloth Tunic"));
    assert!(rendered.contains("Tab=switch"));
}

#[test]
fn character_focused_inventory_details_show_selected_equipped_item() {
    let c = test_character();
    let lines = inventory_screen_text_for_test(
        &c,
        0,
        CharacterEquipmentSlot::Shield,
        InventoryFocus::Character,
        "",
    );
    let rendered = lines.join("\n");

    assert!(rendered.contains("Selected Shield"));
    assert!(rendered.contains("Worn Shield"));
    assert!(rendered.contains("Armor 1 | dodge 2 | speed 0"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test inventory_text_includes_character_equipment_panel_and_tab_command` and `cargo test character_focused_inventory_details_show_selected_equipped_item`
Expected: FAIL because the text helper still renders only bag/details/equipped-comparison content.

- [ ] **Step 3: Implement minimal code**

Add `InventoryFocus`, character panel line builders, selected-equipped detail lines, and a `render_inventory_screen_with_focus` path. Keep the existing `render_inventory_screen` wrapper using bag focus for call-site compatibility.

- [ ] **Step 4: Run verification**

Run: `cargo test inventory_text_includes_character_equipment_panel_and_tab_command` and `cargo test character_focused_inventory_details_show_selected_equipped_item`
Expected: PASS.

### Task 3: Wire Tab and body movement into the interactive inventory loop

**Files:**
- Modify: `src/inventory.rs`
- Modify: `design.md`

- [ ] **Step 1: Implement minimal code**

In `inventory_screen`, keep separate `bag_selected`, `character_selected`, and `focus` state. Tab toggles focus, WASD/arrows move the active side, Enter/x only act on the bag side, and character-side Enter/x show non-pausing explanatory messages.

- [ ] **Step 2: Update design documentation**

Update the Inventory and MVP Equipment Interaction sections in `design.md` to describe the character equipment panel, Tab focus switching, body-slot navigation, and unchanged bag equip/use/drop actions.

- [ ] **Step 3: Run verification**

Run: `cargo test inventory_render_lines_include_grid_capacity_selected_details_and_equipped_comparison`, `cargo test inventory_text_includes_character_equipment_panel_and_tab_command`, `cargo test character_focused_inventory_details_show_selected_equipped_item`, and `cargo test equipment_cursor_moves_through_humanoid_body_slots`
Expected: PASS.

### Task 4: Final validation and commit

**Files:**
- Modify: `src/inventory.rs`
- Modify: `src/tests.rs`
- Modify: `design.md`
- Modify: `docs/superpowers/plans/2026-05-17-inventory-character-equipment-panel.md`

- [ ] **Step 1: Run required guard**

Run: `scripts/agent-commit-guard.sh --fix`
Expected: PASS after `cargo fmt`, `cargo test`, and `cargo check`.

- [ ] **Step 2: Review changes**

Run: `git status --short && git diff`
Expected: Only the planned files changed.

- [ ] **Step 3: Commit**

```bash
git add src/inventory.rs src/tests.rs design.md docs/superpowers/plans/2026-05-17-inventory-character-equipment-panel.md
git commit -m "Add inventory character equipment panel"
```
