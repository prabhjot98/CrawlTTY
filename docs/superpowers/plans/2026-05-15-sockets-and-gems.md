# Sockets and Gems Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement drop-only socketed gear and rare tiered gem loot with a free town Socket Bench for inserting, removing, and replacing gems.

**Architecture:** Extend the existing `Item` model with gem metadata and gear sockets, then route all gameplay effects through centralized gem bonus helpers. Keep loot generation in `src/dungeon.rs`, inventory summaries in `src/inventory.rs`, town service flow in `src/town.rs`, and character stat aggregation in `src/model.rs`.

**Tech Stack:** Rust, serde, rand, crossterm legacy menus, ratatui parent screens, existing `scripts/agent-commit-guard.sh --fix` verification.

---

## File Structure

- Modify `src/model.rs`: add gem enums, socket structs, serde defaults, and gem-aware character stat methods.
- Modify `src/items.rs`: add constructors/helpers for gems and socketed gear metadata.
- Modify `src/inventory.rs`: show gem items and socket contents in item summaries and comparisons.
- Modify `src/dungeon.rs`: roll sockets on dropped gear, roll rare gem drops, apply Opal gold-found bonus.
- Modify `src/town.rs`: add Socket Bench service and socket-management helper functions.
- Modify `src/town_projects.rs`: update Socket Bench benefit text from future infrastructure to active gem management.
- Modify `src/ui.rs`: show socketed equipment names cleanly through existing summaries if needed.
- Modify `src/tests.rs`: add deterministic unit tests for model, loot, socket actions, save compatibility, and stat bonuses.
- Modify `design.md`: update implementation status once gameplay behavior lands.

## Task 1: Data Model And Gem Stat Helpers

**Files:**
- Modify: `src/model.rs`
- Modify: `src/items.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing tests for gem metadata, old save compatibility, and equipped stat bonuses**

Add tests in `src/tests.rs`:

```rust
#[test]
fn saved_items_without_socket_fields_default_to_no_sockets_or_gem_metadata() {
    let json = r#"{
        "name": "Old Sword",
        "kind": "Weapon",
        "value": 10,
        "damage_min": 1,
        "damage_max": 2
    }"#;

    let item: Item = serde_json::from_str(json).unwrap();

    assert!(item.sockets.is_empty());
    assert!(item.gem_kind.is_none());
    assert!(item.gem_tier.is_none());
}

#[test]
fn gems_are_normal_items_with_kind_tier_and_value() {
    let gem = gem_item(GemKind::Topaz, GemTier::Flawed);

    assert!(matches!(gem.kind, ItemKind::Gem));
    assert_eq!(gem.gem_kind, Some(GemKind::Topaz));
    assert_eq!(gem.gem_tier, Some(GemTier::Flawed));
    assert!(gem.name.contains("Flawed Topaz"));
    assert!(gem.value > 0);
}

#[test]
fn equipped_socketed_gems_add_effective_stats() {
    let mut c = test_character();
    c.equipped_weapon.sockets = vec![Some(GemSocket::filled(GemKind::Bloodstone, GemTier::Pristine))];
    c.equipped_armor.sockets = vec![
        Some(GemSocket::filled(GemKind::Ruby, GemTier::Flawed)),
        Some(GemSocket::filled(GemKind::Garnet, GemTier::Chipped)),
    ];
    c.equipped_shield.sockets = vec![Some(GemSocket::filled(GemKind::Topaz, GemTier::Pristine))];

    assert_eq!(c.effective_strength(), c.strength + 1);
    assert_eq!(c.max_hp(), 10 + c.effective_strength() * 5 + 10);
    assert_eq!(c.weapon_damage(), (7, 10));
    assert_eq!(c.weapon_crit_chance(), c.equipped_weapon.crit_chance + 4);
}
```

- [ ] **Step 2: Run focused tests and verify they fail for missing fields/functions**

Run:

```bash
cargo test saved_items_without_socket_fields_default_to_no_sockets_or_gem_metadata gems_are_normal_items_with_kind_tier_and_value equipped_socketed_gems_add_effective_stats
```

Expected: compile failures naming missing `GemKind`, `GemTier`, `GemSocket`, `ItemKind::Gem`, `Item.sockets`, `gem_item`, `effective_strength`, and `weapon_crit_chance`.

- [ ] **Step 3: Implement gem model and stat helpers**

In `src/model.rs`, add serde-backed types:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum GemKind {
    Ruby,
    Sapphire,
    Garnet,
    Emerald,
    Amethyst,
    Quartz,
    Jade,
    Onyx,
    Citrine,
    Topaz,
    Opal,
    Bloodstone,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum GemTier {
    Chipped,
    Flawed,
    Pristine,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GemSocket {
    pub(crate) gem_kind: GemKind,
    pub(crate) gem_tier: GemTier,
}

impl GemSocket {
    pub(crate) fn filled(gem_kind: GemKind, gem_tier: GemTier) -> Self {
        Self { gem_kind, gem_tier }
    }
}
```

Extend `Item`:

```rust
#[serde(default)]
pub(crate) sockets: Vec<Option<GemSocket>>,
#[serde(default)]
pub(crate) gem_kind: Option<GemKind>,
#[serde(default)]
pub(crate) gem_tier: Option<GemTier>,
```

Extend `ItemKind` with `Gem`.

Add centralized bonus helpers in `src/model.rs`:

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct GemBonuses {
    pub(crate) max_hp: u32,
    pub(crate) max_mana: u32,
    pub(crate) strength: u32,
    pub(crate) dexterity: u32,
    pub(crate) intelligence: u32,
    pub(crate) hit_rating: u32,
    pub(crate) dodge_rating: i32,
    pub(crate) armor: i32,
    pub(crate) speed: i32,
    pub(crate) crit_chance: u32,
    pub(crate) gold_found_percent: u32,
    pub(crate) weapon_damage: i32,
}
```

Implement `gem_bonus(kind, tier)`, `Item::socket_bonuses()`, `Character::socket_bonuses()`, `effective_strength`, `effective_dexterity`, `effective_intelligence`, `weapon_crit_chance`, and update existing `max_hp`, `max_mana`, `hit_rating`, `dodge_rating`, `speed`, `armor`, and `weapon_damage` to use socket bonuses.

In `src/items.rs`, initialize new fields in all `Item` constructors and add:

```rust
pub(crate) fn gem_item(kind: GemKind, tier: GemTier) -> Item
```

The gem constructor should set `ItemKind::Gem`, zero combat stats, no requirements, no sockets, and a readable name like `Flawed Topaz (+2% crit chance)`.

- [ ] **Step 4: Run focused tests and verify they pass**

Run:

```bash
cargo test saved_items_without_socket_fields_default_to_no_sockets_or_gem_metadata gems_are_normal_items_with_kind_tier_and_value equipped_socketed_gems_add_effective_stats
```

Expected: all 3 tests pass.

- [ ] **Step 5: Run broader existing tests affected by stat calculations**

Run:

```bash
cargo test new_warrior_matches_mvp_starting_state battle_cry_adds_flat_crit_chance_to_equipped_weapon weapon_summary_and_comparison_show_crit_chance
```

Expected: all listed tests pass without changing existing baseline values for unsocketed gear.

## Task 2: Socketed Gear Rolls And Gem Drops

**Files:**
- Modify: `src/dungeon.rs`
- Modify: `src/items.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing tests for socket count thresholds, tier weights, floor gate, and Opal gold bonus**

Add tests in `src/tests.rs`:

```rust
#[test]
fn socket_count_rolls_follow_rarity_thresholds() {
    assert_eq!(socket_count_for_roll(&Rarity::Common, 0.099), 1);
    assert_eq!(socket_count_for_roll(&Rarity::Common, 0.100), 0);
    assert_eq!(socket_count_for_roll(&Rarity::Magic, 0.049), 2);
    assert_eq!(socket_count_for_roll(&Rarity::Magic, 0.050), 1);
    assert_eq!(socket_count_for_roll(&Rarity::Magic, 0.249), 1);
    assert_eq!(socket_count_for_roll(&Rarity::Magic, 0.250), 0);
    assert_eq!(socket_count_for_roll(&Rarity::Rare, 0.099), 2);
    assert_eq!(socket_count_for_roll(&Rarity::Rare, 0.100), 1);
    assert_eq!(socket_count_for_roll(&Rarity::Rare, 0.349), 1);
    assert_eq!(socket_count_for_roll(&Rarity::Rare, 0.350), 0);
}

#[test]
fn gem_tier_rolls_use_approved_weights() {
    assert_eq!(gem_tier_for_roll(0.799), GemTier::Chipped);
    assert_eq!(gem_tier_for_roll(0.800), GemTier::Flawed);
    assert_eq!(gem_tier_for_roll(0.969), GemTier::Flawed);
    assert_eq!(gem_tier_for_roll(0.970), GemTier::Pristine);
}

#[test]
fn gems_do_not_drop_before_floor_three() {
    assert!(!can_drop_gem_on_floor(2));
    assert!(can_drop_gem_on_floor(3));
}

#[test]
fn opal_socket_bonus_increases_variable_gold_drops() {
    let mut c = test_character();
    c.equipped_armor.sockets = vec![Some(GemSocket::filled(GemKind::Opal, GemTier::Pristine))];

    assert_eq!(apply_gold_find_bonus(&c, 10), 12);
}
```

- [ ] **Step 2: Run focused tests and verify they fail for missing helpers**

Run:

```bash
cargo test socket_count_rolls_follow_rarity_thresholds gem_tier_rolls_use_approved_weights gems_do_not_drop_before_floor_three opal_socket_bonus_increases_variable_gold_drops
```

Expected: compile failures for missing helper functions.

- [ ] **Step 3: Implement deterministic helper functions and integrate loot**

In `src/dungeon.rs`, add:

```rust
pub(crate) fn socket_count_for_roll(rarity: &Rarity, roll: f64) -> usize
pub(crate) fn gem_tier_for_roll(roll: f64) -> GemTier
pub(crate) fn can_drop_gem_on_floor(floor: u32) -> bool
pub(crate) fn apply_gold_find_bonus(c: &Character, gold: u32) -> u32
```

Update `random_equipment_loot` so generated dropped gear calls a helper that fills `item.sockets` with `None` entries based on the item rarity and random roll. Do not add sockets to `rusted_sword`, `cloth_tunic`, `worn_shield`, potions, or gem items.

Add random gem generation:

```rust
pub(crate) fn random_gem() -> Item
```

Gem kind should be randomly selected from the 12 gem kinds. Tier should use `gem_tier_for_roll` with 80/17/3 weights.

Integrate gem drop chances:

- enemy loot on or after floor 3: 2% chance to drop `random_gem()`
- chest loot on or after floor 3: 6% chance to include `random_gem()`
- elite enemies on or after floor 3: 5% chance to drop `random_gem()`
- bosses on or after floor 3: 25% chance to drop `random_gem()`

Apply `apply_gold_find_bonus` only to variable monster and chest gold. Do not apply it to sell values, quest rewards, or fixed town project costs.

- [ ] **Step 4: Run focused tests and verify they pass**

Run:

```bash
cargo test socket_count_rolls_follow_rarity_thresholds gem_tier_rolls_use_approved_weights gems_do_not_drop_before_floor_three opal_socket_bonus_increases_variable_gold_drops
```

Expected: all 4 tests pass.

- [ ] **Step 5: Run loot and dungeon regression tests**

Run:

```bash
cargo test boss_reward_loot_is_always_magic_or_rare_equipment higher_level_loot_has_higher_requirements_and_stats dungeon_generation_obeys_floor_content_rules
```

Expected: all listed tests pass. Boss reward equipment should remain equipment; any extra boss gem drop should be additive, not a replacement for the guaranteed magic/rare gear.

## Task 3: Socket Bench Operations And Town UI

**Files:**
- Modify: `src/town.rs`
- Modify: `src/town_projects.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing tests for lock, insert, remove, replace, and HP/mana clamping**

Add tests in `src/tests.rs`:

```rust
#[test]
fn socket_bench_requires_completed_project() {
    let mut c = test_character();
    c.equipped_weapon.sockets = vec![None];
    c.inventory.push(gem_item(GemKind::Ruby, GemTier::Chipped));

    assert_eq!(
        insert_gem_into_equipped(&mut c, UpgradeSlot::Weapon, 0, 0),
        "Complete the Socket Bench project before socketing gems."
    );
}

#[test]
fn socket_bench_inserts_removes_and_replaces_gems_for_free() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::SocketBench);
    c.equipped_weapon.sockets = vec![None];
    c.inventory.clear();
    c.inventory.push(gem_item(GemKind::Ruby, GemTier::Chipped));
    c.inventory.push(gem_item(GemKind::Topaz, GemTier::Flawed));

    assert_eq!(
        insert_gem_into_equipped(&mut c, UpgradeSlot::Weapon, 0, 0),
        "Inserted Chipped Ruby into Rusted Sword (3-5 dmg, STR F, DEX F)."
    );
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(c.equipped_weapon.sockets[0], Some(GemSocket::filled(GemKind::Ruby, GemTier::Chipped)));

    assert_eq!(
        replace_gem_in_equipped(&mut c, UpgradeSlot::Weapon, 0, 0),
        "Replaced Chipped Ruby with Flawed Topaz in Rusted Sword (3-5 dmg, STR F, DEX F)."
    );
    assert_eq!(c.inventory.len(), 1);
    assert_eq!(c.inventory[0].gem_kind, Some(GemKind::Ruby));
    assert_eq!(c.equipped_weapon.sockets[0], Some(GemSocket::filled(GemKind::Topaz, GemTier::Flawed)));

    assert_eq!(
        remove_gem_from_equipped(&mut c, UpgradeSlot::Weapon, 0),
        "Removed Flawed Topaz from Rusted Sword (3-5 dmg, STR F, DEX F)."
    );
    assert_eq!(c.inventory.len(), 2);
    assert!(c.equipped_weapon.sockets[0].is_none());
}

#[test]
fn removing_hp_or_mana_gem_clamps_current_resources() {
    let mut c = test_character();
    complete_project_for_test(&mut c, TownProject::SocketBench);
    c.equipped_armor.sockets = vec![Some(GemSocket::filled(GemKind::Ruby, GemTier::Pristine))];
    c.hp = c.max_hp();

    remove_gem_from_equipped(&mut c, UpgradeSlot::Armor, 0);

    assert_eq!(c.hp, c.max_hp());
}
```

- [ ] **Step 2: Run focused tests and verify they fail for missing socket action helpers**

Run:

```bash
cargo test socket_bench_requires_completed_project socket_bench_inserts_removes_and_replaces_gems_for_free removing_hp_or_mana_gem_clamps_current_resources
```

Expected: compile failures for missing `insert_gem_into_equipped`, `replace_gem_in_equipped`, and `remove_gem_from_equipped`.

- [ ] **Step 3: Implement socket action helpers and menu service**

In `src/town.rs`, add public helper functions:

```rust
pub(crate) fn insert_gem_into_equipped(c: &mut Character, slot: UpgradeSlot, socket_index: usize, inventory_index: usize) -> String
pub(crate) fn remove_gem_from_equipped(c: &mut Character, slot: UpgradeSlot, socket_index: usize) -> String
pub(crate) fn replace_gem_in_equipped(c: &mut Character, slot: UpgradeSlot, socket_index: usize, inventory_index: usize) -> String
```

Rules:

- If `TownProject::SocketBench` is incomplete, return `Complete the Socket Bench project before socketing gems.`
- If the socket index is invalid, return `No socket selected.`
- If inserting into a filled socket through `insert_gem_into_equipped`, return `That socket is already filled.`
- If removing from an empty socket, return `That socket is already empty.`
- If the inventory index does not point to `ItemKind::Gem`, return `Select a gem from inventory.`
- On insert, remove the gem item from inventory and store its kind/tier in the socket.
- On remove, convert the socket's kind/tier back into a gem item and push it to inventory.
- On replace, remove the new gem from inventory, convert the old socket gem back to inventory, then fill the socket with the new gem.
- After any socket mutation, clamp `c.hp` and `c.mana` to `c.max_hp()` and `c.max_mana()`.

Update `blacksmith` options with `Manage sockets` and route to `socket_bench_screen(c)`. The screen should use existing legacy menu patterns, list socketed equipped/carried gear, show socket contents, and execute Enter actions immediately without a pause prompt.

Update `src/town_projects.rs` Socket Bench benefit to `Unlock free gem insertion, removal, and replacement.`.

- [ ] **Step 4: Run focused tests and verify they pass**

Run:

```bash
cargo test socket_bench_requires_completed_project socket_bench_inserts_removes_and_replaces_gems_for_free removing_hp_or_mana_gem_clamps_current_resources
```

Expected: all 3 tests pass.

- [ ] **Step 5: Run town service regression tests**

Run:

```bash
cargo test town_project_row_text_includes_group_cost_status_and_benefit blacksmith_upgrades_equipped_gear_with_shards_only_after_forge_project salvage_requires_forge_and_reinforced_anvil_adds_one_shard
```

Expected: all listed tests pass, with expected text updated only where Socket Bench benefit changed.

## Task 4: Display, Save Compatibility, Documentation, And Full Verification

**Files:**
- Modify: `src/inventory.rs`
- Modify: `src/ui.rs`
- Modify: `src/tests.rs`
- Modify: `design.md`

- [ ] **Step 1: Write failing tests for gem and socket display**

Add tests in `src/tests.rs`:

```rust
#[test]
fn item_summary_shows_gems_and_socket_contents() {
    let gem = gem_item(GemKind::Topaz, GemTier::Pristine);
    assert!(strip_ansi_codes(&item_summary(&gem)).contains("Pristine Topaz"));
    assert!(strip_ansi_codes(&item_summary(&gem)).contains("+4% crit chance"));

    let mut sword = rusted_sword();
    sword.sockets = vec![Some(GemSocket::filled(GemKind::Ruby, GemTier::Chipped)), None];
    let summary = strip_ansi_codes(&item_summary(&sword));

    assert!(summary.contains("Sockets [Chipped Ruby, empty]"));
}
```

- [ ] **Step 2: Run focused display test and verify it fails**

Run:

```bash
cargo test item_summary_shows_gems_and_socket_contents
```

Expected: failure because summaries do not yet include gem item formatting and socket contents.

- [ ] **Step 3: Implement display helpers and update docs**

In `src/inventory.rs`, update `item_summary`:

- `ItemKind::Gem` should display gem name, tier/stat text, and value.
- Gear summaries should append `Sockets [...]` only when `item.sockets` is non-empty.
- Empty sockets should display `empty`.
- Filled sockets should display concise names like `Chipped Ruby`.

In `src/ui.rs`, keep equipment display names concise. If full summaries are too long for the town status panel, leave the compact equipment line as name-only and rely on inventory/blacksmith detail screens for socket details.

In `design.md`, update the implementation/status notes to say that the first socket system now includes drop-only socketed gear, rare dropped tiered gems, and free Socket Bench management.

- [ ] **Step 4: Run focused display test and verify it passes**

Run:

```bash
cargo test item_summary_shows_gems_and_socket_contents
```

Expected: test passes.

- [ ] **Step 5: Run full required guard**

Run:

```bash
scripts/agent-commit-guard.sh --fix
```

Expected: `cargo fmt`, `cargo test`, and `cargo check` all pass.

- [ ] **Step 6: Review status and diff**

Run:

```bash
git status --short
git diff
```

Expected: only files touched for sockets/gems and documentation are modified.

- [ ] **Step 7: Commit implementation**

Run:

```bash
git add src/model.rs src/items.rs src/inventory.rs src/dungeon.rs src/town.rs src/town_projects.rs src/ui.rs src/tests.rs design.md
git commit -m "Implement sockets and gems"
```

Expected: pre-commit hook runs `cargo fmt --check`, `cargo test`, and `cargo check`; commit succeeds.

## Final Review

After all tasks are complete, run one final code-review pass against `docs/superpowers/specs/2026-05-15-sockets-and-gems-design.md` and this plan. Confirm every design requirement is either implemented or explicitly deferred by the design's non-goals.
