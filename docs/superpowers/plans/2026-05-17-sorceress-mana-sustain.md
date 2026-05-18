# Sorceress Mana Sustain Implementation Plan

> **For pi agents:** REQUIRED SKILL: Use `executing-plans` to implement this plan task-by-task in the current pi session. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make mana easier to acquire and maintain for Sorceress by improving mana potion value/access and adding kill-based Sorceress mana sustain.
**Architecture:** Keep health potion behavior unchanged while introducing mana-specific potion restore tuning. Route mana sustain through the shared enemy death resolver so Sorceress kills from attacks, spells, damage-over-time, and class effects all restore mana consistently. Keep routine potion hotkey behavior single-key and immediate.
**Tech Stack:** Rust, existing Cargo test suite, ratatui UI text helpers.

---

### Task 1: Mana Potion Value, Restore, And Text

**Files:**
- Modify: `src/model.rs`
- Modify: `src/ui.rs`
- Modify: `src/items.rs`
- Modify: `src/inventory.rs`
- Modify: `src/dungeon.rs`
- Modify: `src/town.rs`
- Modify: `src/help.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Add this test near the existing potion/town economy tests in `src/tests.rs`:

```rust
#[test]
fn mana_potions_cost_50_restore_35_percent_and_craft_for_3_herbs() {
    assert_eq!(MANA_POTION_COST, 50);
    assert_eq!(LESSER_MANA_POTION_HERB_COST, 3);

    let potion = mana_potion();
    assert_eq!(potion.value, 50);
    assert_eq!(potion.name, "Lesser Mana Potion (restores 35% mana)");

    let sorceress = character_for_class(CharacterClass::Sorceress);
    assert_eq!(sorceress.max_mana(), 40);
    assert_eq!(lesser_mana_potion_restore(sorceress.max_mana()), 14);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test mana_potions_cost_50_restore_35_percent_and_craft_for_3_herbs`
Expected: FAIL because mana potions still cost 100, craft for 4 herbs, show 15% mana, and there is no mana-specific restore helper.

- [ ] **Step 3: Implement minimal code**

Make these changes:

```rust
// src/model.rs
pub(crate) const MANA_POTION_COST: u32 = 50;
pub(crate) const LESSER_MANA_POTION_RESTORE_PERCENT: u32 = 35;
```

```rust
// src/town.rs
pub(crate) const LESSER_MANA_POTION_HERB_COST: u32 = 3;
```

```rust
// src/ui.rs
pub(crate) fn lesser_mana_potion_restore(max_resource: u32) -> u32 {
    ((max_resource * LESSER_MANA_POTION_RESTORE_PERCENT) / 100).max(1)
}
```

Update mana potion item/detail/help strings from `15% mana` to `35% mana`, and change inventory mana-potion use from `lesser_potion_restore(c.max_mana())` to `lesser_mana_potion_restore(c.max_mana())`.

Update existing tests that assert the old merchant price, crafting herb cost, item name, and pickup messages.

- [ ] **Step 4: Run verification**

Run: `cargo test mana_potions_cost_50_restore_35_percent_and_craft_for_3_herbs && cargo test merchant_sells_lesser_health_and_mana_potions && cargo test distillery_crafts_potions_from_herbs && cargo test inventory_potions_restore_actual_amount_and_do_not_waste_at_full`
Expected: PASS.

---

### Task 2: Dungeon Potion Hotkey Can Use Mana Potions

**Files:**
- Modify: `src/dungeon.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Add this test near `potion_hotkey_consumes_one_health_potion_and_caps_healing` in `src/tests.rs`:

```rust
#[test]
fn potion_hotkey_uses_mana_potion_when_hp_full_and_mana_missing() {
    let mut c = character_for_class(CharacterClass::Sorceress);
    c.active_dungeon = Some(generate_dungeon(1));
    c.hp = c.max_hp();
    c.mana = 0;
    let starting_mana_potions = c
        .inventory
        .iter()
        .filter(|item| matches!(item.kind, ItemKind::ManaPotion))
        .count();

    assert!(use_potion(&mut c));

    assert_eq!(c.mana, lesser_mana_potion_restore(c.max_mana()));
    assert_eq!(
        c.inventory
            .iter()
            .filter(|item| matches!(item.kind, ItemKind::ManaPotion))
            .count(),
        starting_mana_potions - 1
    );
    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d
        .log
        .iter()
        .any(|line| line.contains("You drink a lesser mana potion and restore 14 mana.")));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test potion_hotkey_uses_mana_potion_when_hp_full_and_mana_missing`
Expected: FAIL because `use_potion` only drinks health potions.

- [ ] **Step 3: Implement minimal code**

Update `use_potion` in `src/dungeon.rs` so it:

1. Drinks a health potion first when HP is missing and one is available.
2. Drinks a mana potion when the class is not Rogue, mana is below max, and one is available.
3. Does not consume a potion when the relevant resource is already full or no matching potion exists.
4. Logs `You drink a lesser mana potion and restore {restored} mana.` for mana potion use.

- [ ] **Step 4: Run verification**

Run: `cargo test potion_hotkey_uses_mana_potion_when_hp_full_and_mana_missing && cargo test potion_hotkey_consumes_one_health_potion_and_caps_healing && cargo test rogue_cannot_buy_or_use_mana_potions`
Expected: PASS.

---

### Task 3: Sorceress Kill-Based Mana Sustain

**Files:**
- Modify: `src/sorceress.rs`
- Modify: `src/dungeon.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Add this test near the Sorceress spell tests in `src/tests.rs`:

```rust
#[test]
fn sorceress_kills_restore_10_percent_max_mana_minimum_4() {
    let mut c = character_for_class(CharacterClass::Sorceress);
    c.mana = 12;
    let target = enemy(
        "Mana Dummy",
        'm',
        4,
        2,
        enemy_stats_with_ratings(1, 0, 0, 0, 10, DEFAULT_ENEMY_HIT_RATING, 0),
        enemy_rewards(1, 0, 0),
        false,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![target]));

    assert!(use_firebolt_with_rolls(&mut c, 0.0, 1.0, 0.0));

    assert_eq!(c.mana, 12);
    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d
        .log
        .iter()
        .any(|line| line.contains("Arcane Recovery restores 4 mana.")));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test sorceress_kills_restore_10_percent_max_mana_minimum_4`
Expected: FAIL because Sorceress kills do not restore mana.

- [ ] **Step 3: Implement minimal code**

Add helper functions in `src/sorceress.rs`:

```rust
pub(crate) fn sorceress_kill_mana_restore_amount(c: &Character) -> u32 {
    (c.max_mana() / 10).max(4)
}

pub(crate) fn restore_sorceress_mana_on_kill(c: &mut Character) -> u32 {
    if c.class != CharacterClass::Sorceress {
        return 0;
    }
    let before = c.mana;
    c.mana = (c.mana + sorceress_kill_mana_restore_amount(c)).min(c.max_mana());
    c.mana - before
}
```

Call `restore_sorceress_mana_on_kill(c)` inside `resolve_enemy_death` after level-up logs and before class-specific on-kill effects. If the returned amount is above zero, log `Arcane Recovery restores {amount} mana.` with `LogKind::Heal`.

- [ ] **Step 4: Run verification**

Run: `cargo test sorceress_kills_restore_10_percent_max_mana_minimum_4 && cargo test firebolt_hit_uses_int_spell_damage_and_can_apply_burning && cargo test frost_ring_hits_all_eight_surrounding_tiles_and_freezes_on_chance && cargo test chain_spark_requires_initial_line_of_sight_and_miss_ends_chain`
Expected: PASS.

---

### Task 4: Documentation And Full Verification

**Files:**
- Modify: `design.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Update docs**

Update `design.md` to say:

- Sorceress has Arcane Recovery, restoring 10% max mana, minimum 4, on kills.
- Lesser mana potions restore 35% max mana.
- Lesser mana potions cost 50 gold.
- Distillery crafts lesser mana potions for 3 herbs.
- The dungeon potion hotkey can drink mana potions when HP is full and mana is missing.
- Mana still does not passively regenerate by waiting.

Add this concise line under `## Unreleased` in `CHANGELOG.md`:

```markdown
- Improved Sorceress mana sustain: cheaper stronger mana potions, mana-potion hotkey fallback, and Arcane Recovery on kills.
```

- [ ] **Step 2: Run project guard**

Run: `scripts/agent-commit-guard.sh --fix`
Expected: PASS; this runs `cargo fmt`, `cargo test`, and `cargo check`.

- [ ] **Step 3: Review changes**

Run: `git status --short && git diff`
Expected: only the planned files changed.

- [ ] **Step 4: Commit in task worktree**

Run:

```bash
git config --local core.hooksPath .githooks
git add src/model.rs src/ui.rs src/items.rs src/inventory.rs src/dungeon.rs src/town.rs src/help.rs src/sorceress.rs src/tests.rs design.md CHANGELOG.md docs/superpowers/plans/2026-05-17-sorceress-mana-sustain.md
git commit -m "Improve Sorceress mana sustain"
```

Expected: commit succeeds after hooks pass.
