# Critical Hits Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add player-only critical hits that deal double final damage, with flat crit chance determined by equipped weapon base type.

**Architecture:** Store `crit_chance` directly on `Item` and assign it at item creation time through item-stat helpers. The shared `damage_enemy` player damage path rolls crits after hit success and before applying final damage, while enemy damage paths remain unchanged.

**Tech Stack:** Rust, serde JSON saves, existing unit tests in `src/tests.rs`, existing validation through `scripts/agent-commit-guard.sh --fix`.

---

## File Structure

- Modify `src/model.rs`: add persisted `Item::crit_chance` with serde default.
- Modify `src/items.rs`: add crit chance to `ItemStats`, provide weapon crit constants/helpers, and assign sword/axe values.
- Modify `src/dungeon.rs`: roll player critical hits in `damage_enemy`, include crit context in hit and kill log messages.
- Modify `src/inventory.rs`: display crit chance in weapon summaries and comparisons.
- Modify `src/tests.rs`: add focused tests for item crit assignment, rarity independence, UI display, and combat log formatting.

## Task 1: Item Model And Inventory Display

**Files:**
- Modify: `src/model.rs`
- Modify: `src/items.rs`
- Modify: `src/inventory.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests for weapon crit values and display**

Add these tests to `src/tests.rs` near the existing item and inventory tests:

```rust
#[test]
fn weapon_base_type_sets_flat_crit_chance() {
    assert_eq!(rusted_sword().crit_chance, SWORD_CRIT_CHANCE);
    assert_eq!(crude_axe().crit_chance, AXE_CRIT_CHANCE);
}

#[test]
fn weapon_rarity_does_not_change_crit_chance() {
    let common_sword = item_with_rarity(
        "Iron Sword",
        ItemKind::Weapon,
        45,
        weapon_stats(3, 5, 0, SWORD_CRIT_CHANCE),
        Rarity::Common,
        1,
        requirements(5, 3, 0),
    );
    let rare_sword = item_with_rarity(
        "Rare Iron Sword",
        ItemKind::Weapon,
        75,
        weapon_stats(5, 7, 0, SWORD_CRIT_CHANCE),
        Rarity::Rare,
        3,
        requirements(7, 5, 0),
    );

    assert_eq!(common_sword.crit_chance, SWORD_CRIT_CHANCE);
    assert_eq!(rare_sword.crit_chance, SWORD_CRIT_CHANCE);
}

#[test]
fn weapon_summary_and_comparison_show_crit_chance() {
    let mut c = test_character();
    c.equipped_weapon = rusted_sword();
    let axe = crude_axe();

    assert!(item_summary(&c.equipped_weapon).contains("crit 8%"));
    assert!(item_summary(&axe).contains("crit 5%"));
    assert!(item_comparison(&c, &axe).unwrap().contains("crit -3"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test weapon_base_type_sets_flat_crit_chance weapon_rarity_does_not_change_crit_chance weapon_summary_and_comparison_show_crit_chance
```

Expected: tests fail to compile because `crit_chance`, `SWORD_CRIT_CHANCE`, `AXE_CRIT_CHANCE`, and `weapon_stats` do not exist yet.

- [ ] **Step 3: Add item crit fields and helpers**

In `src/model.rs`, add this field to `Item` after `speed`:

```rust
#[serde(default)]
pub(crate) crit_chance: u32,
```

In `src/items.rs`, add constants near the top:

```rust
pub(crate) const SWORD_CRIT_CHANCE: u32 = 8;
pub(crate) const AXE_CRIT_CHANCE: u32 = 5;
```

Add `crit_chance` to `ItemStats`:

```rust
pub(crate) crit_chance: u32,
```

Update `item_stats` so existing non-weapon calls default to zero:

```rust
pub(crate) fn item_stats(
    damage_min: i32,
    damage_max: i32,
    armor: i32,
    dodge: i32,
    speed: i32,
) -> ItemStats {
    ItemStats {
        damage_min,
        damage_max,
        armor,
        dodge,
        speed,
        crit_chance: 0,
    }
}
```

Add a weapon helper:

```rust
pub(crate) fn weapon_stats(
    damage_min: i32,
    damage_max: i32,
    speed: i32,
    crit_chance: u32,
) -> ItemStats {
    ItemStats {
        damage_min,
        damage_max,
        armor: 0,
        dodge: 0,
        speed,
        crit_chance,
    }
}
```

Set `crit_chance: stats.crit_chance` in both `item` and `item_with_rarity`.

Update weapon creation:

```rust
// rusted_sword
weapon_stats(3, 5, 0, SWORD_CRIT_CHANCE)

// crude_axe
weapon_stats(4, 6, -1, AXE_CRIT_CHANCE)

// generated Iron Sword
weapon_stats(3 + bonus, 5 + bonus, 0, SWORD_CRIT_CHANCE)

// generated War Axe
weapon_stats(4 + bonus, 6 + bonus, -1, AXE_CRIT_CHANCE)
```

- [ ] **Step 4: Add inventory display support**

In `src/inventory.rs`, update the weapon arm of `item_summary` to include crit:

```rust
"{}{} [{} {:?}] {} {RED}dmg {}-{}{RESET} {CYAN}crit {}%{RESET} {YELLOW}value {}{RESET}"
```

and pass `item.crit_chance` before `item.value`.

Update the weapon arm of `item_comparison`:

```rust
format!(
    "Compare: {}  {}",
    format_delta("damage", new_avg - cur_avg),
    format_delta(
        "crit",
        item.crit_chance as i32 - c.equipped_weapon.crit_chance as i32
    )
)
```

- [ ] **Step 5: Run tests to verify they pass**

Run:

```bash
cargo test weapon_base_type_sets_flat_crit_chance weapon_rarity_does_not_change_crit_chance weapon_summary_and_comparison_show_crit_chance
```

Expected: all three tests pass.

- [ ] **Step 6: Run required validation and commit**

Run:

```bash
scripts/agent-commit-guard.sh --fix
git status --short
git diff
git add src/model.rs src/items.rs src/inventory.rs src/tests.rs
git commit -m "Add weapon crit chance to items"
```

Expected: guard passes, only intended files are staged, commit succeeds.

## Task 2: Player Critical Hit Combat Resolution

**Files:**
- Modify: `src/dungeon.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests for critical hit log formatting and enemy exclusion**

Add these tests to `src/tests.rs` near combat tests:

```rust
#[test]
fn critical_player_death_message_marks_critical_hit() {
    let message = enemy_death_message(
        "Skeleton",
        8,
        3,
        EnemyDeathCause::PlayerAttack {
            verb: "hit",
            damage: 14,
            critical: true,
        },
    );

    assert!(message.starts_with("Critical hit! You hit Skeleton"));
    assert!(message.contains("14"));
}

#[test]
fn normal_player_death_message_keeps_existing_wording() {
    let message = enemy_death_message(
        "Skeleton",
        8,
        3,
        EnemyDeathCause::PlayerAttack {
            verb: "hit",
            damage: 7,
            critical: false,
        },
    );

    assert!(message.starts_with("You hit Skeleton"));
    assert!(!message.starts_with("Critical hit!"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test critical_player_death_message_marks_critical_hit normal_player_death_message_keeps_existing_wording
```

Expected: tests fail to compile because `EnemyDeathCause::PlayerAttack` does not have a `critical` field.

- [ ] **Step 3: Extend player death cause with critical context**

In `src/dungeon.rs`, update `EnemyDeathCause`:

```rust
PlayerAttack {
    verb: &'a str,
    damage: i32,
    critical: bool,
},
```

Update existing non-critical construction sites outside `damage_enemy` to pass `critical: false` if any exist.

Update `enemy_death_message`:

```rust
EnemyDeathCause::PlayerAttack {
    verb,
    damage,
    critical,
} => {
    let prefix = if critical { "Critical hit! " } else { "" };
    format!(
        "{prefix}You {verb} {name} for {} and kill it. +{}, +{}.",
        damage_text(damage),
        xp_reward_text(xp),
        gold_reward_text(gold)
    )
}
```

- [ ] **Step 4: Add critical roll to player damage path**

In `src/dungeon.rs`, add a helper near `hit_roll`:

```rust
pub(crate) fn crit_roll(crit_chance: u32) -> bool {
    let chance = (crit_chance.min(100) as f64) / 100.0;
    rand::thread_rng().gen_bool(chance)
}
```

In `damage_enemy`, after the miss branch and before raw damage is calculated:

```rust
let critical = crit_roll(c.equipped_weapon.crit_chance);
```

When applying armor, make damage mutable and double it on crit:

```rust
let mut damage = (raw - armor).max(1);
if critical {
    damage *= 2;
}
```

Pass `critical` into the player death cause:

```rust
EnemyDeathCause::PlayerAttack {
    verb,
    damage,
    critical,
}
```

For non-lethal hit logging, add the critical prefix:

```rust
let prefix = if critical { "Critical hit! " } else { "" };
format!(
    "{prefix}You {verb} {name} for {}. {hp_text}.",
    damage_text(damage)
)
```

Do not modify `enemy_melee_attack`, `cultist_shadow_bolt`, boss specials, bleed, or Spiked Guard damage.

- [ ] **Step 5: Run focused combat tests**

Run:

```bash
cargo test critical_player_death_message_marks_critical_hit normal_player_death_message_keeps_existing_wording
```

Expected: both tests pass.

- [ ] **Step 6: Run required validation and commit**

Run:

```bash
scripts/agent-commit-guard.sh --fix
git status --short
git diff
git add src/dungeon.rs src/tests.rs
git commit -m "Add player critical hit combat"
```

Expected: guard passes, only intended files are staged, commit succeeds.

## Final Verification

- [ ] Run:

```bash
scripts/agent-commit-guard.sh --fix
git status --short
```

Expected: guard passes and working tree is clean.
