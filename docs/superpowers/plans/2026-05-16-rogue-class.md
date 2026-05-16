# Rogue Class Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Rogue as a second playable class with Energy, internal combo points, dagger builders, burst finishers, Smoke Step mobility, and Ratatui class-selection/skill UI.

**Architecture:** Introduce class-aware state so Warrior ranks/cooldowns and Rogue ranks/cooldowns stop living as loose class-specific fields on `Character`. Keep dungeon, save, skill, and UI flows class-aware through small helper functions instead of spreading class checks throughout the loop. Use Ratatui for all interactive Rogue screens and bump the package version to `1.0.0` because this intentionally breaks saves.

**Tech Stack:** Rust, serde, rand, ratatui, crossterm input adapter, existing `scripts/agent-commit-guard.sh --fix` verification.

---

## File Structure

- Create `src/classes.rs`: `CharacterClass`, `WarriorState`, `RogueState`, resource labels, class defaults, and serde compatibility for old `"Ironbound"` class names.
- Create `src/rogue.rs`: Rogue scaling helpers and active skill implementations for Backstab, Venom Edge, Eviscerate, and Smoke Step.
- Modify `src/main.rs`: register the new modules and re-export their helpers.
- Modify `src/model.rs`: replace `class_name` and loose Warrior skill/cooldown fields with class-aware state, add Rogue status fields to combat/stat helpers, and keep shared inventory/equipment/progression fields.
- Modify `src/save.rs`: add Ratatui class selection to character creation and call `Character::new(name, class, death_mode)`.
- Modify `src/items.rs`: add a starter dagger and light Rogue armor.
- Modify `src/dungeon.rs`: route skill keys through class-aware handlers, show class-specific skill help, tick/clear class combat state, and apply Rogue smoke protection to enemy hit rolls.
- Modify `src/skills.rs`: route the skill tree screen by class and add a Ratatui Rogue tree/details view.
- Modify `src/ui.rs`: display `c.class_name()` or equivalent class label instead of a stored string.
- Modify `src/tests.rs`: add focused tests for class creation, save compatibility, Rogue resource/combo behavior, Rogue active skills, class-aware dungeon UI/help, and Warrior regressions.
- Modify `Cargo.toml`: bump `version` from `0.1.0` to `1.0.0`.
- Modify `README.md` and `design.md`: document Rogue, class selection, and the save reset for the multi-class release.

## Task 1: Class Model And Save-Version Boundary

**Files:**
- Create: `src/classes.rs`
- Modify: `src/main.rs`
- Modify: `src/model.rs`
- Modify: `src/tests.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Write failing tests for class enum, class-name compatibility, and version bump**

Add tests in `src/tests.rs` near the existing save/class tests:

```rust
#[test]
fn class_names_parse_current_and_legacy_values() {
    assert_eq!(CharacterClass::from_save_name("Warrior"), CharacterClass::Warrior);
    assert_eq!(CharacterClass::from_save_name("Ironbound"), CharacterClass::Warrior);
    assert_eq!(CharacterClass::from_save_name("Rogue"), CharacterClass::Rogue);
    assert_eq!(CharacterClass::Warrior.name(), "Warrior");
    assert_eq!(CharacterClass::Rogue.name(), "Rogue");
}

#[test]
fn package_version_is_major_one_for_save_breaking_rogue_release() {
    assert!(SAVE_VERSION.starts_with("1."));
}

#[test]
fn warrior_state_defaults_match_existing_rank_baseline() {
    let state = WarriorState::default();

    assert_eq!(state.cleave_rank, 1);
    assert_eq!(state.shield_bash_rank, 1);
    assert_eq!(state.battle_cry_rank, 1);
    assert_eq!(state.deep_cut_rank, 1);
    assert_eq!(state.iron_guard_rank, 1);
    assert_eq!(state.second_wind_rank, 1);
    assert_eq!(state.cleave_cooldown, 0);
    assert_eq!(state.shield_bash_cooldown, 0);
    assert_eq!(state.battle_cry_cooldown, 0);
    assert_eq!(state.battle_cry_charges, 0);
    assert_eq!(state.second_wind_shield, 0);
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test class_names_parse_current_and_legacy_values package_version_is_major_one_for_save_breaking_rogue_release warrior_state_defaults_match_existing_rank_baseline
```

Expected: compile failures for missing `CharacterClass` and `WarriorState`, plus a version assertion failure until `Cargo.toml` is bumped.

- [ ] **Step 3: Add class and state types**

Create `src/classes.rs`:

```rust
use crate::*;

pub(crate) const ROGUE_MAX_ENERGY: u32 = 100;
pub(crate) const ROGUE_MAX_COMBO_POINTS: u32 = 5;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum CharacterClass {
    Warrior,
    Rogue,
}

impl CharacterClass {
    pub(crate) fn name(self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Rogue => "Rogue",
        }
    }

    pub(crate) fn from_save_name(name: &str) -> Self {
        match name {
            "Rogue" => CharacterClass::Rogue,
            "Warrior" | "Ironbound" => CharacterClass::Warrior,
            _ => CharacterClass::Warrior,
        }
    }
}

pub(crate) fn default_character_class() -> CharacterClass {
    CharacterClass::Warrior
}

pub(crate) fn deserialize_character_class<'de, D>(
    deserializer: D,
) -> std::result::Result<CharacterClass, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let class_name = String::deserialize(deserializer)?;
    Ok(CharacterClass::from_save_name(&class_name))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WarriorState {
    #[serde(default = "default_skill_rank")]
    pub(crate) cleave_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) shield_bash_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) battle_cry_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) deep_cut_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) iron_guard_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) second_wind_rank: u32,
    #[serde(default)]
    pub(crate) cleave_cooldown: u32,
    #[serde(default)]
    pub(crate) shield_bash_cooldown: u32,
    #[serde(default)]
    pub(crate) battle_cry_cooldown: u32,
    #[serde(default, alias = "battle_cry_turns")]
    pub(crate) battle_cry_charges: u32,
    #[serde(default)]
    pub(crate) cleave_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) shield_bash_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) battle_cry_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) deep_cut_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) iron_guard_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) second_wind_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) second_wind_shield: u32,
}

impl Default for WarriorState {
    fn default() -> Self {
        Self {
            cleave_rank: 1,
            shield_bash_rank: 1,
            battle_cry_rank: 1,
            deep_cut_rank: 1,
            iron_guard_rank: 1,
            second_wind_rank: 1,
            cleave_cooldown: 0,
            shield_bash_cooldown: 0,
            battle_cry_cooldown: 0,
            battle_cry_charges: 0,
            cleave_mastery: None,
            shield_bash_mastery: None,
            battle_cry_mastery: None,
            deep_cut_mastery: None,
            iron_guard_mastery: None,
            second_wind_mastery: None,
            second_wind_shield: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RogueState {
    #[serde(default)]
    pub(crate) energy: u32,
    #[serde(default)]
    pub(crate) combo_points: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) backstab_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) venom_edge_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) eviscerate_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) smoke_step_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) rupture_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) slip_away_rank: u32,
    #[serde(default)]
    pub(crate) smoke_step_cooldown: u32,
    #[serde(default)]
    pub(crate) smoke_protection_turns: u32,
    #[serde(default)]
    pub(crate) empowered_backstab_turns: u32,
}

impl Default for RogueState {
    fn default() -> Self {
        Self {
            energy: ROGUE_MAX_ENERGY,
            combo_points: 0,
            backstab_rank: 1,
            venom_edge_rank: 1,
            eviscerate_rank: 1,
            smoke_step_rank: 1,
            rupture_rank: 1,
            slip_away_rank: 1,
            smoke_step_cooldown: 0,
            smoke_protection_turns: 0,
            empowered_backstab_turns: 0,
        }
    }
}
```

- [ ] **Step 4: Register the module and bump version**

In `src/main.rs`, add the module and export beside the existing modules:

```rust
mod classes;
mod rogue;
```

and:

```rust
pub(crate) use classes::*;
pub(crate) use rogue::*;
```

In `Cargo.toml`, change:

```toml
version = "1.0.0"
```

- [ ] **Step 5: Migrate `Character` to class-aware fields**

In `src/model.rs`, replace the stored `class_name: String` and direct Warrior fields with:

```rust
#[serde(
    default = "default_character_class",
    alias = "class_name",
    deserialize_with = "deserialize_character_class"
)]
pub(crate) class: CharacterClass,
#[serde(default)]
pub(crate) warrior: WarriorState,
#[serde(default)]
pub(crate) rogue: RogueState,
```

Remove the old `deserialize_class_name` and `normalize_class_name` helpers from `src/model.rs`; the replacement lives in `src/classes.rs`.

Add a class-label method in `impl Character`:

```rust
pub(crate) fn class_name(&self) -> &'static str {
    self.class.name()
}
```

Replace old direct Warrior field reads/writes throughout code with `c.warrior.<field>`. Do this mechanically for:

- `cleave_rank`
- `shield_bash_rank`
- `battle_cry_rank`
- `deep_cut_rank`
- `iron_guard_rank`
- `second_wind_rank`
- `cleave_cooldown`
- `shield_bash_cooldown`
- `battle_cry_cooldown`
- `battle_cry_charges`
- Warrior mastery fields
- `second_wind_shield`

Keep this task behavior-preserving for Warrior.

- [ ] **Step 6: Run focused tests and verify they pass**

Run:

```bash
cargo test class_names_parse_current_and_legacy_values package_version_is_major_one_for_save_breaking_rogue_release warrior_state_defaults_match_existing_rank_baseline new_warrior_matches_mvp_starting_state
```

Expected: all listed tests pass.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock src/classes.rs src/main.rs src/model.rs src/dungeon.rs src/skills.rs src/ui.rs src/tests.rs
git commit -m "Introduce class-aware character state"
```

## Task 2: Ratatui Class Selection

**Files:**
- Modify: `src/save.rs`
- Modify: `src/model.rs`
- Modify: `src/ui.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing tests for Rogue creation and Ratatui class selection text**

Add tests in `src/tests.rs` near character-creation tests:

```rust
#[test]
fn new_rogue_matches_starting_state() {
    let c = Character::new("Shade".to_string(), CharacterClass::Rogue, DeathMode::Softcore);

    assert_eq!(c.class, CharacterClass::Rogue);
    assert_eq!(c.class_name(), "Rogue");
    assert_eq!(c.strength, 2);
    assert_eq!(c.dexterity, 7);
    assert_eq!(c.intelligence, 1);
    assert_eq!(c.rogue.energy, ROGUE_MAX_ENERGY);
    assert_eq!(c.rogue.combo_points, 0);
    assert!(c.equipped_weapon.name.contains("Dagger"));
    assert!(c.equipped_armor.name.contains("Leathers"));
}

#[test]
fn character_creation_renders_class_choices() {
    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                "Mara",
                CharacterClass::Rogue,
                DeathMode::Hardcore,
                "",
            )
        })
        .unwrap();

    let text = backend_text(&terminal);
    assert!(text.contains("Warrior"));
    assert!(text.contains("Rogue"));
    assert!(text.contains("> Rogue"));
    assert!(text.contains("> Hardcore"));
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test new_rogue_matches_starting_state character_creation_renders_class_choices
```

Expected: compile failures because `Character::new` still lacks a class argument, Rogue starter gear is missing, and `render_character_creation_screen` does not accept a class selection.

- [ ] **Step 3: Add Rogue starter items**

In `src/items.rs`, add:

```rust
pub(crate) fn training_dagger() -> Item {
    item(
        "Training Dagger (2-4 dmg, DEX D)",
        ItemKind::Weapon,
        20,
        weapon_stats(2, 4, 1, 12),
    )
}

pub(crate) fn patched_leathers() -> Item {
    item(
        "Patched Leathers (+1 armor, +2 dodge)",
        ItemKind::Armor,
        18,
        item_stats(0, 0, 1, 2, 1),
    )
}
```

- [ ] **Step 4: Update character constructors**

In `src/model.rs`, change `Character::new` to:

```rust
pub(crate) fn new(name: String, class: CharacterClass, death_mode: DeathMode) -> Self {
    match class {
        CharacterClass::Warrior => Self::new_warrior(name, death_mode),
        CharacterClass::Rogue => Self::new_rogue(name, death_mode),
    }
}
```

Extract the existing constructor body into `fn new_warrior(name, death_mode) -> Self`, setting:

```rust
class: CharacterClass::Warrior,
warrior: WarriorState::default(),
rogue: RogueState::default(),
equipped_weapon: rusted_sword(),
equipped_armor: cloth_tunic(),
equipped_shield: worn_shield(),
```

Add `fn new_rogue(name, death_mode) -> Self` with:

```rust
let strength = 2;
let dexterity = 7;
let intelligence = 1;
let max_hp = 10 + strength * 5;
let max_mana = 10 + intelligence * 5;
Self {
    name,
    class: CharacterClass::Rogue,
    death_mode,
    level: 1,
    xp: 0,
    gold: 50,
    strength,
    dexterity,
    intelligence,
    unspent_attributes: 0,
    unspent_skills: 0,
    hp: max_hp,
    mana: max_mana,
    inventory: ItemGrid::player_starting(vec![health_potion(), health_potion()]),
    stash: ItemGrid::stash_starting(),
    equipped_weapon: training_dagger(),
    equipped_armor: patched_leathers(),
    equipped_shield: empty_offhand(),
    bellkeeper_defeated: false,
    glass_tyrant_defeated: false,
    act1_completed: false,
    act2_completed: false,
    active_dungeon: None,
    weapon_shards: 0,
    armor_shards: 0,
    shield_shards: 0,
    completed_town_projects: Vec::new(),
    warrior: WarriorState::default(),
    rogue: RogueState::default(),
    pending_town_message: String::new(),
}
```

Add this helper in `src/items.rs`:

```rust
pub(crate) fn empty_offhand() -> Item {
    item(
        "Empty Offhand",
        ItemKind::Shield,
        0,
        item_stats(0, 0, 0, 0, 0),
    )
}
```

- [ ] **Step 5: Make character creation class-aware in Ratatui**

In `src/save.rs`, track:

```rust
let mut selected_class = CharacterClass::Warrior;
```

Use these exact controls:

- `1` = Warrior
- `2` = Rogue
- `s` = Softcore
- `h` = Hardcore
- `Tab` = toggle death mode

Update Enter to call:

```rust
return Ok(Character::new(
    name.trim().to_string(),
    selected_class,
    death_mode,
));
```

Change `render_character_creation_screen` signature to:

```rust
pub(crate) fn render_character_creation_screen(
    frame: &mut Frame,
    name: &str,
    selected_class: CharacterClass,
    selected_death_mode: DeathMode,
    message: &str,
)
```

Render class lines in the body:

```rust
let warrior_marker = if selected_class == CharacterClass::Warrior { ">" } else { " " };
let rogue_marker = if selected_class == CharacterClass::Rogue { ">" } else { " " };
Line::styled(format!("{warrior_marker} Warrior - armored melee skills and mana."), Style::default().fg(Color::Cyan)),
Line::styled(format!("{rogue_marker} Rogue - dagger burst, Energy, and combo points."), Style::default().fg(Color::Green)),
```

Update footer commands to:

```rust
"Type=name  Backspace=delete  1/2=class  S/H or Tab=death mode  Enter=confirm"
```

- [ ] **Step 6: Update UI class labels**

In `src/ui.rs`, replace `c.class_name.clone()` with:

```rust
c.class_name().to_string()
```

- [ ] **Step 7: Run focused tests and verify they pass**

Run:

```bash
cargo test new_rogue_matches_starting_state character_creation_renders_class_choices character_creation_renders_as_ratatui_screen new_warrior_matches_mvp_starting_state
```

Expected: all listed tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/save.rs src/model.rs src/ui.rs src/items.rs src/tests.rs
git commit -m "Add Rogue character creation"
```

## Task 3: Class-Aware Dungeon Resources And Warrior Preservation

**Files:**
- Modify: `src/classes.rs`
- Modify: `src/dungeon.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing tests for class resource labels and class-aware skill keys**

Add tests:

```rust
#[test]
fn class_resource_labels_match_active_class() {
    let warrior = Character::new("War".to_string(), CharacterClass::Warrior, DeathMode::Softcore);
    let rogue = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);

    assert_eq!(warrior.resource_label(), "Mana");
    assert_eq!(rogue.resource_label(), "Energy");
    assert_eq!(rogue.current_resource(), ROGUE_MAX_ENERGY);
    assert_eq!(rogue.max_resource(), ROGUE_MAX_ENERGY);
}

#[test]
fn rogue_dungeon_action_labels_include_four_active_skills() {
    let rogue = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);

    assert_eq!(dungeon_action_label_for(&rogue, '1'), "Backstab");
    assert_eq!(dungeon_action_label_for(&rogue, '2'), "Venom Edge");
    assert_eq!(dungeon_action_label_for(&rogue, '3'), "Eviscerate");
    assert_eq!(dungeon_action_label_for(&rogue, '4'), "Smoke Step");
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test class_resource_labels_match_active_class rogue_dungeon_action_labels_include_four_active_skills
```

Expected: compile failures for missing resource and action-label helpers.

- [ ] **Step 3: Add class resource helpers**

In `src/classes.rs`, add:

```rust
impl Character {
    pub(crate) fn resource_label(&self) -> &'static str {
        match self.class {
            CharacterClass::Warrior => "Mana",
            CharacterClass::Rogue => "Energy",
        }
    }

    pub(crate) fn current_resource(&self) -> u32 {
        match self.class {
            CharacterClass::Warrior => self.mana,
            CharacterClass::Rogue => self.rogue.energy,
        }
    }

    pub(crate) fn max_resource(&self) -> u32 {
        match self.class {
            CharacterClass::Warrior => self.max_mana(),
            CharacterClass::Rogue => ROGUE_MAX_ENERGY,
        }
    }

    pub(crate) fn spend_rogue_energy(&mut self, amount: u32) -> bool {
        if self.rogue.energy < amount {
            false
        } else {
            self.rogue.energy -= amount;
            true
        }
    }

    pub(crate) fn restore_rogue_energy(&mut self, amount: u32) {
        self.rogue.energy = (self.rogue.energy + amount).min(ROGUE_MAX_ENERGY);
    }
}
```

- [ ] **Step 4: Split class combat clear/tick**

In `src/dungeon.rs`, replace the body of `clear_combat_state` with:

```rust
pub(crate) fn clear_combat_state(c: &mut Character) {
    c.warrior.cleave_cooldown = 0;
    c.warrior.shield_bash_cooldown = 0;
    c.warrior.battle_cry_cooldown = 0;
    c.warrior.battle_cry_charges = 0;
    c.warrior.second_wind_shield = 0;
    c.rogue.combo_points = 0;
    c.rogue.smoke_step_cooldown = 0;
    c.rogue.smoke_protection_turns = 0;
    c.rogue.empowered_backstab_turns = 0;
}
```

In `tick_player_effects`, tick both class states and restore Rogue Energy:

```rust
pub(crate) fn tick_player_effects(c: &mut Character) {
    c.warrior.cleave_cooldown = c.warrior.cleave_cooldown.saturating_sub(1);
    c.warrior.shield_bash_cooldown = c.warrior.shield_bash_cooldown.saturating_sub(1);
    c.warrior.battle_cry_cooldown = c.warrior.battle_cry_cooldown.saturating_sub(1);
    c.rogue.smoke_step_cooldown = c.rogue.smoke_step_cooldown.saturating_sub(1);
    c.rogue.empowered_backstab_turns = c.rogue.empowered_backstab_turns.saturating_sub(1);
    if c.class == CharacterClass::Rogue {
        c.restore_rogue_energy(15);
    }
}
```

Decrement `smoke_protection_turns` after enemy turns, not before enemy attacks. Add this at the end of `enemy_turns` before restoring `active_dungeon`:

```rust
c.rogue.smoke_protection_turns = c.rogue.smoke_protection_turns.saturating_sub(1);
```

- [ ] **Step 5: Make dungeon action labels class-aware**

Change calls from:

```rust
let action_label = dungeon_action_label(key);
```

to:

```rust
let action_label = dungeon_action_label_for(c, key);
```

Add:

```rust
pub(crate) fn dungeon_action_label_for(c: &Character, key: char) -> &'static str {
    match (c.class, key) {
        (_, 'w' | 'W') => "Move north / attack",
        (_, 's' | 'S') => "Move south / attack",
        (_, 'a' | 'A') => "Move west / attack",
        (_, 'd' | 'D') => "Move east / attack",
        (CharacterClass::Warrior, '1') => "Cleave",
        (CharacterClass::Warrior, '2') => "Shield Bash",
        (CharacterClass::Warrior, '3') => "Battle Cry",
        (CharacterClass::Rogue, '1') => "Backstab",
        (CharacterClass::Rogue, '2') => "Venom Edge",
        (CharacterClass::Rogue, '3') => "Eviscerate",
        (CharacterClass::Rogue, '4') => "Smoke Step",
        (_, 'p' | 'P') => "Drink potion",
        (_, 'g' | 'G') => "Pick up",
        (_, 'i' | 'I') => "Inventory",
        _ => "Command",
    }
}
```

Keep `dungeon_action_label(key)` as a Warrior wrapper for existing tests:

```rust
pub(crate) fn dungeon_action_label(key: char) -> &'static str {
    let warrior = Character::new("Label".to_string(), CharacterClass::Warrior, DeathMode::Softcore);
    dungeon_action_label_for(&warrior, key)
}
```

- [ ] **Step 6: Run focused tests and Warrior regressions**

Run:

```bash
cargo test class_resource_labels_match_active_class rogue_dungeon_action_labels_include_four_active_skills dungeon_action_label_names_inventory_commands battle_cry_charges_survive_movement_and_spend_on_attacks
```

Expected: all listed tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/classes.rs src/dungeon.rs src/tests.rs
git commit -m "Route dungeon resources by class"
```

## Task 4: Rogue Skill Help And Hotkey Dispatch Skeleton

**Files:**
- Modify: `src/dungeon.rs`
- Create/Modify: `src/rogue.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing tests for Rogue skill help and no-turn invalid skills**

Add tests:

```rust
#[test]
fn rogue_skill_help_lines_show_energy_combo_points_and_four_skills() {
    let c = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);
    let text = dungeon_skill_help_lines(&c)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("CP 0/5"));
    assert!(text.contains("1 Backstab"));
    assert!(text.contains("2 Venom Edge"));
    assert!(text.contains("3 Eviscerate"));
    assert!(text.contains("4 Smoke Step"));
}

#[test]
fn warrior_does_not_accept_rogue_fourth_skill_key() {
    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(!handle_class_skill_key(&mut c, '4'));
    let log = &c.active_dungeon.as_ref().unwrap().log;
    assert!(log.iter().any(|line| line.contains("Unknown class skill")));
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test rogue_skill_help_lines_show_energy_combo_points_and_four_skills warrior_does_not_accept_rogue_fourth_skill_key
```

Expected: compile failures for missing `handle_class_skill_key`, or assertion failures while Rogue help still renders Warrior help.

- [ ] **Step 3: Add Rogue skill help lines**

Make `dungeon_skill_help_lines` public for tests:

```rust
pub(crate) fn dungeon_skill_help_lines(c: &Character) -> Vec<Line<'static>> {
    match c.class {
        CharacterClass::Warrior => warrior_skill_help_lines(c),
        CharacterClass::Rogue => rogue_skill_help_lines(c),
    }
}
```

Move existing contents into `fn warrior_skill_help_lines`.

Add:

```rust
fn rogue_skill_help_lines(c: &Character) -> Vec<Line<'static>> {
    vec![
        Line::from(format!(
            "Rogue: Energy {}/{}  CP {}/{}",
            c.rogue.energy,
            ROGUE_MAX_ENERGY,
            c.rogue.combo_points,
            ROGUE_MAX_COMBO_POINTS
        )),
        Line::from(format!(
            "1 Backstab r{}: cost 25 Energy. Build 1 CP; empowered after movement, smoke, or poison.",
            c.rogue.backstab_rank
        )),
        Line::from(format!(
            "2 Venom Edge r{}: cost 30 Energy. Build 1 CP and poison for 3 turns.",
            c.rogue.venom_edge_rank
        )),
        Line::from(format!(
            "3 Eviscerate r{}: cost 35 Energy. Spend CP for burst damage.",
            c.rogue.eviscerate_rank
        )),
        Line::from(format!(
            "4 Smoke Step r{}: cost 35 Energy, cd 4. Dash 2 tiles. Ready in {}.",
            c.rogue.smoke_step_rank,
            c.rogue.smoke_step_cooldown
        )),
    ]
}
```

- [ ] **Step 4: Add class hotkey dispatch**

In `src/dungeon.rs`, replace the `match key` arms for `'1'`, `'2'`, `'3'` with:

```rust
'1' | '2' | '3' | '4' => took_turn = handle_class_skill_key(c, key),
```

Add:

```rust
pub(crate) fn handle_class_skill_key(c: &mut Character, key: char) -> bool {
    match c.class {
        CharacterClass::Warrior => match key {
            '1' => use_cleave(c),
            '2' => use_shield_bash(c),
            '3' => use_battle_cry(c),
            _ => {
                log_unknown_class_skill(c);
                false
            }
        },
        CharacterClass::Rogue => match key {
            '1' => use_backstab(c),
            '2' => use_venom_edge(c),
            '3' => use_eviscerate(c),
            '4' => use_smoke_step(c),
            _ => {
                log_unknown_class_skill(c);
                false
            }
        },
    }
}

fn log_unknown_class_skill(c: &mut Character) {
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(&mut d.log, LogKind::Warn, "Unknown class skill.");
    }
}
```

In `is_known_dungeon_command`, include `'4'`.

- [ ] **Step 5: Add Rogue skill stubs that log clear warnings**

In `src/rogue.rs`, add compile-through stubs that Tasks 5 and 6 replace before final verification:

```rust
use crate::*;

pub(crate) fn use_backstab(c: &mut Character) -> bool {
    log_unimplemented_rogue_skill(c, "Backstab")
}

pub(crate) fn use_venom_edge(c: &mut Character) -> bool {
    log_unimplemented_rogue_skill(c, "Venom Edge")
}

pub(crate) fn use_eviscerate(c: &mut Character) -> bool {
    log_unimplemented_rogue_skill(c, "Eviscerate")
}

pub(crate) fn use_smoke_step(c: &mut Character) -> bool {
    log_unimplemented_rogue_skill(c, "Smoke Step")
}

fn log_unimplemented_rogue_skill(c: &mut Character, skill: &str) -> bool {
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(&mut d.log, LogKind::Warn, format!("{skill} is not ready yet."));
    }
    false
}
```

These stubs allow UI and dispatch tests to pass while combat implementation is added in Tasks 5 and 6. Each stub is replaced by a real implementation before final verification.

- [ ] **Step 6: Run focused tests and verify they pass**

Run:

```bash
cargo test rogue_skill_help_lines_show_energy_combo_points_and_four_skills warrior_does_not_accept_rogue_fourth_skill_key rogue_dungeon_action_labels_include_four_active_skills
```

Expected: all listed tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/dungeon.rs src/rogue.rs src/tests.rs
git commit -m "Add Rogue skill dispatch"
```

## Task 5: Rogue Builders And Eviscerate

**Files:**
- Modify: `src/rogue.rs`
- Modify: `src/model.rs`
- Modify: `src/dungeon.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing tests for combo point builders and Eviscerate spending**

Add tests:

```rust
#[test]
fn rogue_builders_grant_combo_points_and_cap_at_five() {
    let enemy = armored_training_dummy(3, 2);
    let mut c = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    for _ in 0..7 {
        c.rogue.energy = ROGUE_MAX_ENERGY;
        assert!(use_backstab(&mut c));
    }

    assert_eq!(c.rogue.combo_points, ROGUE_MAX_COMBO_POINTS);
}

#[test]
fn eviscerate_requires_and_spends_combo_points() {
    let enemy = armored_training_dummy(3, 2);
    let mut c = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    assert!(!use_eviscerate(&mut c));
    assert_eq!(c.rogue.combo_points, 0);

    c.rogue.combo_points = 3;
    c.rogue.energy = ROGUE_MAX_ENERGY;
    assert!(use_eviscerate(&mut c));
    assert_eq!(c.rogue.combo_points, 0);
}

#[test]
fn venom_edge_applies_poison_and_grants_combo_point() {
    let enemy = armored_training_dummy(3, 2);
    let mut c = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    assert!(use_venom_edge(&mut c));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.rogue.combo_points, 1);
    assert_eq!(d.enemies[0].poison_turns, 3);
    assert!(d.enemies[0].poison_damage > 0);
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test rogue_builders_grant_combo_points_and_cap_at_five eviscerate_requires_and_spends_combo_points venom_edge_applies_poison_and_grants_combo_point
```

Expected: compile failure for missing enemy poison fields or assertion failures because Rogue skill stubs do not mutate state.

- [ ] **Step 3: Add poison fields to enemies and tick poison**

In `src/model.rs`, add to `Enemy` with serde defaults:

```rust
#[serde(default)]
pub(crate) poison_turns: u32,
#[serde(default)]
pub(crate) poison_damage: i32,
```

Initialize both fields to zero in `enemy(...)` in `src/dungeon_gen.rs`.

In `enemy_turns`, after bleed handling and before energy gain, add poison ticking:

```rust
if d.enemies[i].poison_turns > 0 {
    let poison_damage = d.enemies[i].poison_damage.max(1);
    d.enemies[i].hp -= poison_damage;
    d.enemies[i].poison_turns -= 1;
    log_event(
        &mut d.log,
        LogKind::Hit,
        format!(
            "{} suffers poison for {}. {}.",
            d.enemies[i].name,
            damage_text(poison_damage),
            enemy_hp_text(&d.enemies[i])
        ),
    );
    if d.enemies[i].hp <= 0 {
        if resolve_enemy_death(c, &mut d, i, EnemyDeathCause::Effect { source: "Poison" }) {
            return;
        }
        continue;
    }
}
```

- [ ] **Step 4: Add Rogue combo and scaling helpers**

In `src/rogue.rs`, replace stubs with helpers:

```rust
const BACKSTAB_COST: u32 = 25;
const VENOM_EDGE_COST: u32 = 30;
const EVISCERATE_COST: u32 = 35;

pub(crate) fn add_rogue_combo_point(c: &mut Character) {
    c.rogue.combo_points = (c.rogue.combo_points + 1).min(ROGUE_MAX_COMBO_POINTS);
}

pub(crate) fn backstab_multiplier(c: &Character) -> f32 {
    if empowered_backstab_ready(c) { 1.20 } else { 0.90 }
}

pub(crate) fn venom_edge_multiplier(_c: &Character) -> f32 {
    0.70
}

pub(crate) fn poison_damage_for_rank(rank: u32) -> i32 {
    1 + rank.min(5).div_ceil(2) as i32
}

pub(crate) fn eviscerate_multiplier_for_points(points: u32) -> f32 {
    match points.min(5) {
        0 => 0.0,
        1 => 0.80,
        2 => 1.30,
        3 => 1.90,
        4 => 2.60,
        _ => 3.50,
    }
}

pub(crate) fn empowered_backstab_ready(c: &Character) -> bool {
    c.rogue.empowered_backstab_turns > 0
}
```

- [ ] **Step 5: Implement Backstab, Venom Edge, and Eviscerate**

Use the current adjacent target helper so Rogue stays melee:

```rust
fn adjacent_rogue_target(c: &Character, skill: &str) -> Option<usize> {
    let target = adjacent_enemy_indices(c).first().copied();
    if target.is_none() {
        if let Some(d) = c.active_dungeon.as_mut() {
            log_event(&mut d.log, LogKind::Warn, format!("No adjacent enemy for {skill}."));
        }
    }
    target
}
```

Implement:

```rust
pub(crate) fn use_backstab(c: &mut Character) -> bool {
    if !c.spend_rogue_energy(BACKSTAB_COST) {
        log_rogue_warning(c, "Not enough Energy for Backstab.");
        return false;
    }
    let Some(index) = adjacent_rogue_target(c, "Backstab") else {
        c.restore_rogue_energy(BACKSTAB_COST);
        return false;
    };
    let multiplier = backstab_multiplier(c);
    damage_enemy(c, index, multiplier, "backstab");
    add_rogue_combo_point(c);
    c.rogue.empowered_backstab_turns = 0;
    true
}

pub(crate) fn use_venom_edge(c: &mut Character) -> bool {
    if !c.spend_rogue_energy(VENOM_EDGE_COST) {
        log_rogue_warning(c, "Not enough Energy for Venom Edge.");
        return false;
    }
    let Some(index) = adjacent_rogue_target(c, "Venom Edge") else {
        c.restore_rogue_energy(VENOM_EDGE_COST);
        return false;
    };
    damage_enemy(c, index, venom_edge_multiplier(c), "venom edge");
    if let Some(enemy) = c.active_dungeon.as_mut().and_then(|d| d.enemies.get_mut(index)) {
        if enemy.hp > 0 {
            enemy.poison_turns = enemy.poison_turns.max(3);
            enemy.poison_damage = enemy.poison_damage.max(poison_damage_for_rank(c.rogue.venom_edge_rank));
        }
    }
    add_rogue_combo_point(c);
    true
}

pub(crate) fn use_eviscerate(c: &mut Character) -> bool {
    let points = c.rogue.combo_points;
    if points == 0 {
        log_rogue_warning(c, "Eviscerate requires combo points.");
        return false;
    }
    if !c.spend_rogue_energy(EVISCERATE_COST) {
        log_rogue_warning(c, "Not enough Energy for Eviscerate.");
        return false;
    }
    let Some(index) = adjacent_rogue_target(c, "Eviscerate") else {
        c.restore_rogue_energy(EVISCERATE_COST);
        return false;
    };
    let multiplier = eviscerate_multiplier_for_points(points);
    damage_enemy(c, index, multiplier, "eviscerate");
    c.rogue.combo_points = 0;
    if c.rogue.slip_away_rank > 0 {
        c.rogue.smoke_protection_turns = c.rogue.smoke_protection_turns.max(1);
    }
    true
}

fn log_rogue_warning(c: &mut Character, message: &str) {
    if let Some(d) = c.active_dungeon.as_mut() {
        log_event(&mut d.log, LogKind::Warn, message);
    }
}
```

- [ ] **Step 6: Add poison payoff to Eviscerate**

After `damage_enemy` in `use_eviscerate`, inspect the target. When the target is still alive and poisoned, consume one poison turn and apply flat bonus damage through an effect helper:

```rust
pub(crate) fn damage_enemy_with_rogue_effect(
    c: &mut Character,
    enemy_index: usize,
    source: &'static str,
    damage: i32,
) {
    let Some(mut d) = c.active_dungeon.take() else {
        return;
    };
    let mut killed = false;
    if let Some(enemy) = d.enemies.get_mut(enemy_index) {
        if enemy.hp > 0 {
            enemy.hp -= damage;
            killed = enemy.hp <= 0;
            log_event(
                &mut d.log,
                LogKind::Hit,
                format!("{source} deals {}. {}.", damage_text(damage), enemy_hp_text(enemy)),
            );
        }
    }
    if killed
        && resolve_enemy_death(
            c,
            &mut d,
            enemy_index,
            EnemyDeathCause::Effect { source },
        )
    {
        return;
    }
    c.active_dungeon = Some(d);
}
```

Then in `use_eviscerate`, replace direct poison HP subtraction with:

```rust
let poison_bonus = {
    let Some(d) = c.active_dungeon.as_mut() else {
        return true;
    };
    let Some(enemy) = d.enemies.get_mut(index) else {
        return true;
    };
    if enemy.hp > 0 && enemy.poison_turns > 0 {
        enemy.poison_turns = enemy.poison_turns.saturating_sub(1);
        Some(enemy.poison_damage.max(1) * points as i32)
    } else {
        None
    }
};
if let Some(bonus) = poison_bonus {
    damage_enemy_with_rogue_effect(c, index, "Eviscerate poison", bonus);
}
```

Add this focused test:

```rust
#[test]
fn eviscerate_poison_payoff_can_kill_and_award_rewards() {
    let enemy = enemy(
        "Poison Dummy",
        'p',
        3,
        2,
        enemy_stats(3, 0, 0, 0, 10),
        enemy_rewards(10, 1, 1),
        false,
    );
    let mut c = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));
    c.rogue.combo_points = 5;
    c.rogue.energy = ROGUE_MAX_ENERGY;
    {
        let d = c.active_dungeon.as_mut().unwrap();
        d.enemies[0].poison_turns = 3;
        d.enemies[0].poison_damage = 3;
    }

    assert!(use_eviscerate(&mut c));

    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d.enemies[0].hp <= 0 || d.enemies.is_empty());
    assert!(c.xp >= 10);
}
```

The test target has low HP so the poison payoff, not the base weapon damage, is responsible for the reward path after the first damage call.

- [ ] **Step 7: Run focused tests and Warrior regressions**

Run:

```bash
cargo test rogue_builders_grant_combo_points_and_cap_at_five eviscerate_requires_and_spends_combo_points venom_edge_applies_poison_and_grants_combo_point critical_damage_enemy_doubles_post_armor_damage_and_logs_hit
```

Expected: all listed tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/model.rs src/dungeon_gen.rs src/dungeon.rs src/rogue.rs src/tests.rs
git commit -m "Implement Rogue combo attacks"
```

## Task 6: Smoke Step And Smoke Protection

**Files:**
- Modify: `src/rogue.rs`
- Modify: `src/dungeon.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing tests for Smoke Step movement and rejection cases**

Add tests:

```rust
#[test]
fn smoke_step_rejects_blocked_occupied_and_out_of_range_destinations() {
    let enemy = armored_training_dummy(4, 2);
    let mut c = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    assert!(!try_smoke_step(&mut c, 3, 0));
    assert!(!try_smoke_step(&mut c, 2, 0));
    assert!(!try_smoke_step(&mut c, -2, 0));
}

#[test]
fn smoke_step_moves_and_enables_empowered_backstab() {
    let mut c = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(try_smoke_step(&mut c, 2, 0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!((d.player_x, d.player_y), (4, 2));
    assert_eq!(c.rogue.smoke_step_cooldown, 4);
    assert_eq!(c.rogue.smoke_protection_turns, 1);
    assert_eq!(c.rogue.empowered_backstab_turns, 1);
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test smoke_step_rejects_blocked_occupied_and_out_of_range_destinations smoke_step_moves_and_enables_empowered_backstab
```

Expected: compile failure for missing `try_smoke_step` or assertion failures because `use_smoke_step` is still a stub.

- [ ] **Step 3: Implement direction-based Smoke Step**

In `src/rogue.rs`, add:

```rust
const SMOKE_STEP_COST: u32 = 35;
const SMOKE_STEP_COOLDOWN: u32 = 4;

pub(crate) fn try_smoke_step(c: &mut Character, dx: i32, dy: i32) -> bool {
    if c.rogue.smoke_step_cooldown > 0 {
        log_rogue_warning(c, "Smoke Step is on cooldown.");
        return false;
    }
    if dx.abs() + dy.abs() == 0 || dx.abs() + dy.abs() > 2 || (dx != 0 && dy != 0) {
        log_rogue_warning(c, "Smoke Step must move 1 or 2 cardinal tiles.");
        return false;
    }
    if !c.spend_rogue_energy(SMOKE_STEP_COST) {
        log_rogue_warning(c, "Not enough Energy for Smoke Step.");
        return false;
    }
    let Some(d) = c.active_dungeon.as_mut() else {
        c.restore_rogue_energy(SMOKE_STEP_COST);
        return false;
    };
    let nx = d.player_x + dx;
    let ny = d.player_y + dy;
    if dungeon_tile(d, nx, ny) == '#'
        || d.enemies.iter().any(|enemy| enemy.hp > 0 && enemy.x == nx && enemy.y == ny)
    {
        c.restore_rogue_energy(SMOKE_STEP_COST);
        log_event(&mut d.log, LogKind::Warn, "Smoke Step destination is blocked.");
        return false;
    }
    d.player_x = nx;
    d.player_y = ny;
    c.rogue.smoke_step_cooldown = SMOKE_STEP_COOLDOWN;
    c.rogue.smoke_protection_turns = 1;
    c.rogue.empowered_backstab_turns = 1;
    log_event(&mut d.log, LogKind::Status, "You vanish through smoke.");
    true
}
```

- [ ] **Step 4: Wire `use_smoke_step` to a deterministic default**

Because `4` has no direction prompt in the current single-key dungeon loop, make the first pass step away from the nearest adjacent enemy if possible, otherwise step toward the farthest open cardinal tile. Add helper:

```rust
pub(crate) fn smoke_step_direction(c: &Character) -> Option<(i32, i32)> {
    let d = c.active_dungeon.as_ref()?;
    let directions = [(2, 0), (-2, 0), (0, 2), (0, -2), (1, 0), (-1, 0), (0, 1), (0, -1)];
    directions
        .into_iter()
        .find(|(dx, dy)| {
            let nx = d.player_x + dx;
            let ny = d.player_y + dy;
            dungeon_tile(d, nx, ny) != '#'
                && !d.enemies.iter().any(|enemy| enemy.hp > 0 && enemy.x == nx && enemy.y == ny)
        })
}

pub(crate) fn use_smoke_step(c: &mut Character) -> bool {
    let Some((dx, dy)) = smoke_step_direction(c) else {
        log_rogue_warning(c, "No open tile for Smoke Step.");
        return false;
    };
    try_smoke_step(c, dx, dy)
}
```

The deterministic helper above is the first-pass single-key behavior for this plan. A directional Ratatui targeting sub-loop is outside this implementation plan.

- [ ] **Step 5: Apply smoke protection to enemy hit chance**

In `enemy_melee_attack` and `cultist_shadow_bolt`, adjust target dodge:

```rust
let smoke_dodge_bonus = if c.class == CharacterClass::Rogue && c.rogue.smoke_protection_turns > 0 {
    20
} else {
    0
};
if hit_roll(25, c.dodge_rating() as i32 + smoke_dodge_bonus) {
    // existing hit path
}
```

Use the same helper for ranged attacks:

```rust
pub(crate) fn defensive_dodge_rating(c: &Character) -> i32 {
    let smoke_dodge_bonus = if c.class == CharacterClass::Rogue && c.rogue.smoke_protection_turns > 0 {
        20
    } else {
        0
    };
    c.dodge_rating() as i32 + smoke_dodge_bonus
}
```

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test smoke_step_rejects_blocked_occupied_and_out_of_range_destinations smoke_step_moves_and_enables_empowered_backstab enemy_energy_uses_speed_before_acting
```

Expected: all listed tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/rogue.rs src/dungeon.rs src/tests.rs
git commit -m "Implement Smoke Step"
```

## Task 7: Rogue Skill Tree Screen And Rank Scaling

**Files:**
- Modify: `src/skills.rs`
- Modify: `src/rogue.rs`
- Modify: `src/tests.rs`

- [ ] **Step 1: Write failing tests for Rogue skill tree rendering and rank upgrades**

Add tests:

```rust
#[test]
fn rogue_skill_screen_renders_with_ratatui() {
    let c = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);
    let lines = skill_tree_lines(&c, 0, "");
    let text = lines
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Rogue Skill Tree"));
    assert!(text.contains("Backstab"));
    assert!(text.contains("Venom Edge"));
    assert!(text.contains("Eviscerate"));
    assert!(text.contains("Smoke Step"));
    assert!(text.contains("Rupture"));
    assert!(text.contains("Slip Away"));
}

#[test]
fn rogue_skill_upgrades_spend_points_and_scale_values() {
    let mut c = Character::new("Sneak".to_string(), CharacterClass::Rogue, DeathMode::Softcore);
    c.unspent_skills = 1;

    assert_eq!(choose_skill_or_mastery(&mut c, "Backstab"), "Upgraded Backstab to rank 2.");
    assert_eq!(c.rogue.backstab_rank, 2);
    assert_eq!(c.unspent_skills, 0);
    assert!(backstab_base_percent_for_rank(2) > backstab_base_percent_for_rank(1));
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test rogue_skill_screen_renders_with_ratatui rogue_skill_upgrades_spend_points_and_scale_values
```

Expected: assertion failures because the skill screen still renders Warrior data and upgrade logic does not handle Rogue skill names.

- [ ] **Step 3: Make skill tree rendering class-aware**

In `src/skills.rs`, route `skill_tree_lines`:

```rust
pub(crate) fn skill_tree_lines(c: &Character, selected: usize, message: &str) -> Vec<Line<'static>> {
    match c.class {
        CharacterClass::Warrior => warrior_skill_tree_lines(c, selected, message),
        CharacterClass::Rogue => rogue_skill_tree_lines(c, selected, message),
    }
}
```

Move existing implementation into `warrior_skill_tree_lines`.

Add `rogue_skill_tree_lines` with sections:

```rust
fn rogue_skill_tree_lines(c: &Character, selected: usize, message: &str) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::styled("Rogue Skill Tree", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Line::from(unspent_skills_text(c.unspent_skills)),
    ];
    if !message.is_empty() {
        lines.push(Line::styled(message.to_string(), Style::default().fg(Color::Yellow)));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled("Daggers Branch", Style::default().add_modifier(Modifier::BOLD)));
    append_skill_choice_lines(&mut lines, selected, "Backstab", c.rogue.backstab_rank);
    append_skill_choice_lines(&mut lines, selected, "Eviscerate", c.rogue.eviscerate_rank);
    lines.push(Line::styled("Venom Branch", Style::default().add_modifier(Modifier::BOLD)));
    append_skill_choice_lines(&mut lines, selected, "Venom Edge", c.rogue.venom_edge_rank);
    append_skill_choice_lines(&mut lines, selected, "Rupture", c.rogue.rupture_rank);
    lines.push(Line::styled("Smoke Branch", Style::default().add_modifier(Modifier::BOLD)));
    append_skill_choice_lines(&mut lines, selected, "Smoke Step", c.rogue.smoke_step_rank);
    append_skill_choice_lines(&mut lines, selected, "Slip Away", c.rogue.slip_away_rank);
    lines
}
```

- [ ] **Step 4: Add Rogue rank helpers**

In `src/rogue.rs`, add:

```rust
pub(crate) fn backstab_base_percent_for_rank(rank: u32) -> u32 {
    90 + rank.saturating_sub(1).min(4) * 5
}

pub(crate) fn empowered_backstab_percent_for_rank(rank: u32) -> u32 {
    120 + rank.saturating_sub(1).min(4) * 10
}

pub(crate) fn venom_edge_percent_for_rank(rank: u32) -> u32 {
    70 + rank.saturating_sub(1).min(4) * 5
}

pub(crate) fn eviscerate_bonus_percent_for_rank(rank: u32) -> u32 {
    rank.saturating_sub(1).min(4) * 10
}

pub(crate) fn smoke_step_dodge_bonus_for_rank(rank: u32) -> i32 {
    20 + rank.saturating_sub(1).min(4) as i32 * 3
}

pub(crate) fn slip_away_dodge_bonus_for_rank(rank: u32) -> i32 {
    5 + rank.saturating_sub(1).min(4) as i32 * 2
}
```

Use these helpers in Rogue skill implementations and skill help text instead of fixed percentages where applicable.

- [ ] **Step 5: Extend upgrade logic for Rogue skills**

In `skill_rank`, `upgrade_skill`, `choose_skill_or_mastery`, and prerequisite helpers, match Rogue skill names when `c.class == CharacterClass::Rogue`:

```rust
match skill {
    "Backstab" => c.rogue.backstab_rank += 1,
    "Venom Edge" => c.rogue.venom_edge_rank += 1,
    "Eviscerate" => c.rogue.eviscerate_rank += 1,
    "Smoke Step" => c.rogue.smoke_step_rank += 1,
    "Rupture" => c.rogue.rupture_rank += 1,
    "Slip Away" => c.rogue.slip_away_rank += 1,
    _ => return "Unknown skill.".to_string(),
}
```

Use prerequisites:

- `Eviscerate` upgrades require `Backstab` rank 2.
- `Rupture` upgrades require `Venom Edge` rank 2.
- `Slip Away` upgrades require `Smoke Step` rank 2.

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test rogue_skill_screen_renders_with_ratatui rogue_skill_upgrades_spend_points_and_scale_values skill_screens_render_with_ratatui passive_skill_upgrades_require_branch_starter_rank_two
```

Expected: all listed tests pass after updating existing Warrior prerequisite assertions to account for class-specific skill names.

- [ ] **Step 7: Commit**

```bash
git add src/skills.rs src/rogue.rs src/tests.rs
git commit -m "Add Rogue skill tree"
```

## Task 8: Documentation, Save Reset Messaging, And Final Verification

**Files:**
- Modify: `README.md`
- Modify: `design.md`
- Modify: `src/tests.rs`

- [ ] **Step 1: Update README feature list and save reset note**

In `README.md`, update current features to mention:

```markdown
- Warrior and Rogue classes with attributes, leveling, skills, and class-specific resources
```

Add under save files:

```markdown
The 1.0.0 multi-class release intentionally resets saves from older major versions.
```

- [ ] **Step 2: Update `design.md` implementation status**

Add an implementation status bullet near the class section:

```markdown
Current implementation status: Warrior and Rogue are playable class choices. Warrior uses mana and the existing Cleave, Shield Bash, Battle Cry, Deep Cut, Iron Guard, and Second Wind tree. Rogue uses Energy, internal combo points, Backstab, Venom Edge, Eviscerate, Smoke Step, Rupture, and Slip Away.
```

Add a save-system note:

```markdown
The 1.0.0 multi-class release intentionally breaks and resets saves from older major versions through the existing save-version gate.
```

- [ ] **Step 3: Confirm final regression test coverage in `src/tests.rs`**

Ensure these tests exist by name or equivalent behavior:

```rust
// class creation
new_warrior_matches_mvp_starting_state
new_rogue_matches_starting_state
character_creation_renders_class_choices

// save compatibility and versioning
class_names_parse_current_and_legacy_values
package_version_is_major_one_for_save_breaking_rogue_release
save_major_version_mismatch_resets_save

// rogue combat
rogue_builders_grant_combo_points_and_cap_at_five
eviscerate_requires_and_spends_combo_points
venom_edge_applies_poison_and_grants_combo_point
smoke_step_rejects_blocked_occupied_and_out_of_range_destinations
smoke_step_moves_and_enables_empowered_backstab

// ratatui/class UI
rogue_skill_help_lines_show_energy_combo_points_and_four_skills
rogue_skill_screen_renders_with_ratatui
```

- [ ] **Step 4: Run focused Rogue suite**

Run:

```bash
cargo test rogue class_names_parse_current_and_legacy_values package_version_is_major_one_for_save_breaking_rogue_release
```

Expected: all matching Rogue/class/version tests pass.

- [ ] **Step 5: Run required repository guard**

Run:

```bash
scripts/agent-commit-guard.sh --fix
```

Expected: `cargo fmt`, `cargo test`, and `cargo check` all pass.

- [ ] **Step 6: Review diff and commit docs/final cleanup**

Run:

```bash
git status --short
git diff
```

Confirm only Rogue implementation files, tests, docs, and version files changed. Then commit:

```bash
git add Cargo.toml Cargo.lock README.md design.md src/classes.rs src/rogue.rs src/main.rs src/model.rs src/save.rs src/items.rs src/dungeon.rs src/skills.rs src/ui.rs src/tests.rs
git commit -m "Add playable Rogue class"
```

## Self-Review Notes

- Spec coverage: the plan covers Rogue as a second class, Energy, internal combo points, four active hotkeys, poison setup, Smoke Step, Ratatui UI, class-aware state, save major-version reset, and docs updates.
- Scope boundary: the plan does not add traps, ranged Rogue, stealth awareness, facing, or broad enemy pathing changes because those are explicit non-goals in the approved spec.
- Ratatui requirement: character creation and skill tree changes are Ratatui-based; no new raw terminal or legacy `println!` screens are introduced.
- Save compatibility: old `"Ironbound"` string normalization remains in `CharacterClass::from_save_name`, while old major versions reset through the existing save gate after the `1.0.0` bump.
