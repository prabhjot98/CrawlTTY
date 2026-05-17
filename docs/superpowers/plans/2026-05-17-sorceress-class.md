# Sorceress Class Implementation Plan

> **For pi agents:** REQUIRED SKILL: Use `executing-plans` to implement this plan task-by-task in the current pi session. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Sorceress as the third playable class with Mana, wand-and-focus equipment, auto-targeted elemental spells, an unlockable Mana Shield toggle, and a six-skill Flame/Frost/Storm tree.
**Architecture:** Extend the existing class-aware model with `SorceressState`, keep spell-specific scaling and active skill behavior in a new `src/sorceress.rs`, and route dungeon UI/skill dispatch through the current class switches. Reuse the existing hit-vs-dodge accuracy model for spells, add small status fields to `Enemy`, and keep Sorceress loot class-routed like Warrior and Rogue equipment pools.
**Tech Stack:** Rust, serde, rand, ratatui, existing dungeon/skill/item modules, and `scripts/agent-commit-guard.sh --fix` verification.

---

## File Structure

- Create: `src/sorceress.rs` for Sorceress constants, rank scaling helpers, spell targeting helpers, active spells, and status helpers.
- Modify: `src/main.rs` to register and re-export the Sorceress module.
- Modify: `src/classes.rs` to add `CharacterClass::Sorceress`, `SorceressState`, and class resource helpers.
- Modify: `src/model.rs` to add Sorceress state, enemy status fields, and Sorceress starting character construction.
- Modify: `src/items.rs` to add Cracked Wand, Cracked Focus, and Frayed Robe starter items plus wand crit constant.
- Modify: `src/save.rs` to show/select Sorceress during Ratatui character creation.
- Modify: `src/skills.rs` to add the Sorceress six-skill tree, prerequisites, rank effects, and upgrades.
- Modify: `src/dungeon.rs` to route Sorceress dungeon help, hotkeys, cooldown ticks, clear-combat state, statuses, spell damage, and Mana Shield damage absorption.
- Modify: `src/inventory.rs` to enforce Sorceress wand/focus equipment restrictions.
- Modify: `src/tests.rs` with focused TDD coverage for class creation, skill tree, spells, statuses, loot, and UI.
- Modify: `README.md` and `design.md` to document the playable Sorceress.

## Task 1: Class Model, Starter State, And Starter Gear

**Files:**
- Modify: `src/classes.rs`
- Modify: `src/model.rs`
- Modify: `src/items.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests for Sorceress class parsing, defaults, and starter state**

Update the existing `class_names_parse_current_and_legacy_values` test in `src/tests.rs` to include Sorceress:

```rust
#[test]
fn class_names_parse_current_and_legacy_values() {
    assert_eq!(
        CharacterClass::from_save_name("Warrior"),
        CharacterClass::Warrior
    );
    assert_eq!(
        CharacterClass::from_save_name("Ironbound"),
        CharacterClass::Warrior
    );
    assert_eq!(
        CharacterClass::from_save_name("Rogue"),
        CharacterClass::Rogue
    );
    assert_eq!(
        CharacterClass::from_save_name("Sorceress"),
        CharacterClass::Sorceress
    );
    assert_eq!(CharacterClass::Warrior.name(), "Warrior");
    assert_eq!(CharacterClass::Rogue.name(), "Rogue");
    assert_eq!(CharacterClass::Sorceress.name(), "Sorceress");
}
```

Add these tests near `new_rogue_matches_starting_state`:

```rust
#[test]
fn sorceress_state_defaults_match_mvp_skill_baseline() {
    let state = SorceressState::default();

    assert_eq!(state.firebolt_rank, 1);
    assert_eq!(state.frost_ring_rank, 1);
    assert_eq!(state.chain_spark_rank, 1);
    assert_eq!(state.kindle_rank, 0);
    assert_eq!(state.mana_shield_rank, 0);
    assert_eq!(state.static_charge_rank, 0);
    assert_eq!(state.frost_ring_cooldown, 0);
    assert_eq!(state.chain_spark_cooldown, 0);
    assert!(!state.mana_shield_active);
}

#[test]
fn new_sorceress_matches_starting_state() {
    let c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );

    assert_eq!(c.class, CharacterClass::Sorceress);
    assert_eq!(c.class_name(), "Sorceress");
    assert_eq!((c.strength, c.dexterity, c.intelligence), (1, 3, 6));
    assert_eq!(c.max_hp(), 15);
    assert_eq!(c.max_mana(), 40);
    assert_eq!(c.hp, c.max_hp());
    assert_eq!(c.mana, c.max_mana());
    assert_eq!(c.inventory.len(), 4);
    assert_eq!(
        c.inventory
            .iter()
            .filter(|item| item.kind == ItemKind::HealthPotion)
            .count(),
        2
    );
    assert_eq!(
        c.inventory
            .iter()
            .filter(|item| item.kind == ItemKind::ManaPotion)
            .count(),
        2
    );
    assert!(c.equipped_weapon.name.contains("Wand"));
    assert_eq!(c.equipped_weapon.kind, ItemKind::Weapon);
    assert_eq!(c.equipped_weapon.required_strength, 0);
    assert_eq!(c.equipped_weapon.required_dexterity, 0);
    assert_eq!(c.equipped_weapon.required_intelligence, 2);
    assert!(c.equipped_shield.name.contains("Focus"));
    assert_eq!(c.equipped_shield.kind, ItemKind::Shield);
    assert_eq!(c.equipped_shield.required_strength, 0);
    assert_eq!(c.equipped_shield.required_dexterity, 0);
    assert_eq!(c.equipped_shield.required_intelligence, 2);
    assert!(c.equipped_armor.name.contains("Robe"));
    assert_eq!(c.equipped_armor.kind, ItemKind::Armor);
    assert!(can_equip_item(&c, &c.equipped_weapon));
    assert!(can_equip_item(&c, &c.equipped_shield));
    assert!(can_equip_item(&c, &c.equipped_armor));
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test class_names_parse_current_and_legacy_values sorceress_state_defaults_match_mvp_skill_baseline new_sorceress_matches_starting_state
```

Expected: FAIL because `CharacterClass::Sorceress`, `SorceressState`, and Sorceress starter items do not exist.

- [ ] **Step 3: Add Sorceress class and state**

In `src/classes.rs`, extend `CharacterClass`:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum CharacterClass {
    Warrior,
    Rogue,
    Sorceress,
}
```

Update `CharacterClass::name` and `CharacterClass::from_save_name`:

```rust
pub(crate) fn name(self) -> &'static str {
    match self {
        CharacterClass::Warrior => "Warrior",
        CharacterClass::Rogue => "Rogue",
        CharacterClass::Sorceress => "Sorceress",
    }
}

pub(crate) fn from_save_name(name: &str) -> Self {
    match name {
        "Warrior" | "Ironbound" => CharacterClass::Warrior,
        "Rogue" => CharacterClass::Rogue,
        "Sorceress" => CharacterClass::Sorceress,
        _ => CharacterClass::Warrior,
    }
}
```

Add `SorceressState` below `RogueState`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SorceressState {
    #[serde(default = "default_skill_rank")]
    pub(crate) firebolt_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) frost_ring_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) chain_spark_rank: u32,
    #[serde(default = "default_locked_skill_rank")]
    pub(crate) kindle_rank: u32,
    #[serde(default = "default_locked_skill_rank")]
    pub(crate) mana_shield_rank: u32,
    #[serde(default = "default_locked_skill_rank")]
    pub(crate) static_charge_rank: u32,
    #[serde(default)]
    pub(crate) frost_ring_cooldown: u32,
    #[serde(default)]
    pub(crate) chain_spark_cooldown: u32,
    #[serde(default)]
    pub(crate) mana_shield_active: bool,
}

impl Default for SorceressState {
    fn default() -> Self {
        Self {
            firebolt_rank: 1,
            frost_ring_rank: 1,
            chain_spark_rank: 1,
            kindle_rank: 0,
            mana_shield_rank: 0,
            static_charge_rank: 0,
            frost_ring_cooldown: 0,
            chain_spark_cooldown: 0,
            mana_shield_active: false,
        }
    }
}
```

Update `Character::resource_label`, `current_resource`, `max_resource`, and `restore_class_resource_full` so Sorceress follows Warrior's Mana behavior:

```rust
match self.class {
    CharacterClass::Warrior | CharacterClass::Sorceress => "Mana",
    CharacterClass::Rogue => "Energy",
}
```

Use the same combined Warrior/Sorceress branch for current mana, maximum mana, and full restoration.

- [ ] **Step 4: Add Sorceress starter items**

In `src/items.rs`, add a wand crit constant with the other weapon constants:

```rust
pub(crate) const WAND_CRIT_CHANCE: u32 = 4;
```

Add these item constructors near the other starter gear:

```rust
pub(crate) fn cracked_wand() -> Item {
    item_with_rarity(
        "Cracked Wand (2-3 spell, INT D)",
        ItemKind::Weapon,
        20,
        weapon_stats(2, 3, 0, WAND_CRIT_CHANCE),
        Rarity::Common,
        1,
        requirements(0, 0, 2),
    )
}

pub(crate) fn cracked_focus() -> Item {
    item_with_rarity(
        "Cracked Focus (+1 dodge, INT D)",
        ItemKind::Shield,
        20,
        item_stats(0, 0, 0, 1, 0),
        Rarity::Common,
        1,
        requirements(0, 0, 2),
    )
}

pub(crate) fn frayed_robe() -> Item {
    item_with_rarity(
        "Frayed Robe (+1 armor)",
        ItemKind::Armor,
        16,
        item_stats(0, 0, 1, 0, 0),
        Rarity::Common,
        1,
        requirements(0, 0, 0),
    )
}
```

- [ ] **Step 5: Add Sorceress character construction**

In `src/model.rs`, add the state field to `Character` immediately after `rogue`:

```rust
#[serde(default)]
pub(crate) sorceress: SorceressState,
```

Update `Character::new`:

```rust
match class {
    CharacterClass::Warrior => Self::new_warrior(name, death_mode),
    CharacterClass::Rogue => Self::new_rogue(name, death_mode),
    CharacterClass::Sorceress => Self::new_sorceress(name, death_mode),
}
```

Add `sorceress: SorceressState::default()` to both existing constructors. Then add this constructor:

```rust
fn new_sorceress(name: String, death_mode: DeathMode) -> Self {
    let strength = 1;
    let dexterity = 3;
    let intelligence = 6;
    let max_hp = 10 + strength * 5;
    let max_mana = 10 + intelligence * 5;
    Self {
        name,
        class: CharacterClass::Sorceress,
        death_mode,
        level: 1,
        xp: 0,
        gold: 50,
        strength,
        dexterity,
        intelligence,
        unspent_attributes: 0,
        unspent_skills: 0,
        warrior: WarriorState::default(),
        rogue: RogueState::default(),
        sorceress: SorceressState::default(),
        hp: max_hp,
        mana: max_mana,
        inventory: ItemGrid::player_starting(vec![
            health_potion(),
            health_potion(),
            mana_potion(),
            mana_potion(),
        ]),
        stash: ItemGrid::stash_starting(),
        equipped_weapon: cracked_wand(),
        equipped_armor: frayed_robe(),
        equipped_shield: cracked_focus(),
        equipped_helm: empty_helm(),
        equipped_gloves: empty_gloves(),
        equipped_boots: empty_boots(),
        equipped_belt: empty_belt(),
        equipped_amulet: empty_amulet(),
        equipped_ring1: empty_ring(),
        equipped_ring2: empty_ring(),
        bellkeeper_defeated: false,
        glass_tyrant_defeated: false,
        act1_completed: false,
        act2_completed: false,
        active_dungeon: None,
        weapon_shards: 0,
        armor_shards: 0,
        shield_shards: 0,
        herbs: 0,
        completed_town_projects: Vec::new(),
        pending_town_message: String::new(),
    }
}
```

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test class_names_parse_current_and_legacy_values sorceress_state_defaults_match_mvp_skill_baseline new_sorceress_matches_starting_state
```

Expected: PASS.

## Task 2: Character Creation Selection And Class UI Labels

**Files:**
- Modify: `src/save.rs`
- Modify: `src/dungeon.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests for selecting and rendering Sorceress**

Add these tests near the existing character creation tests:

```rust
#[test]
fn character_creation_can_select_sorceress() {
    let mut state = CharacterCreationState::new("");

    assert_eq!(state.selected_class, CharacterClass::Warrior);
    assert!(state.handle_key(KEY_ARROW_DOWN).is_none());
    assert_eq!(state.selected_class, CharacterClass::Rogue);
    assert!(state.handle_key(KEY_ARROW_DOWN).is_none());
    assert_eq!(state.selected_class, CharacterClass::Sorceress);
    assert!(state.handle_key(KEY_ARROW_DOWN).is_none());
    assert_eq!(state.selected_class, CharacterClass::Warrior);
    assert!(state.handle_key(KEY_ARROW_UP).is_none());
    assert_eq!(state.selected_class, CharacterClass::Sorceress);
    assert!(state.handle_key('1').is_none());
    assert_eq!(state.selected_class, CharacterClass::Warrior);
    assert!(state.handle_key('2').is_none());
    assert_eq!(state.selected_class, CharacterClass::Rogue);
    assert!(state.handle_key('3').is_none());
    assert_eq!(state.selected_class, CharacterClass::Sorceress);

    assert!(state.handle_key('\n').is_none());
    for key in "Lyra".chars() {
        assert!(state.handle_key(key).is_none());
    }
    assert!(state.handle_key('\n').is_none());
    let character = state.handle_key('\n').unwrap();

    assert_eq!(character.name, "Lyra");
    assert_eq!(character.class, CharacterClass::Sorceress);
    assert_eq!(character.death_mode, DeathMode::Softcore);
}

#[test]
fn character_creation_renders_sorceress_choice() {
    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            render_character_creation_screen(
                frame,
                CharacterCreationStep::Class,
                "Lyra",
                CharacterClass::Sorceress,
                DeathMode::Softcore,
                "",
            )
        })
        .unwrap();

    let text = backend_text(&terminal);
    assert!(text.contains("Warrior"));
    assert!(text.contains("Rogue"));
    assert!(text.contains("Sorceress"));
    assert!(text.contains("> Sorceress"));
    assert!(!text.contains("> Warrior"));
    assert!(!text.contains("> Rogue"));
}
```

Add this test near dungeon action label tests:

```rust
#[test]
fn sorceress_dungeon_action_labels_include_spell_hotkeys() {
    let sorceress = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );

    assert_eq!(dungeon_action_label_for(&sorceress, '1'), "Firebolt");
    assert_eq!(dungeon_action_label_for(&sorceress, '2'), "Frost Ring");
    assert_eq!(dungeon_action_label_for(&sorceress, '3'), "Chain Spark");
    assert_eq!(dungeon_action_label_for(&sorceress, '4'), "Mana Shield");
    assert!(is_known_dungeon_command_for(&sorceress, '4'));
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test character_creation_can_select_sorceress character_creation_renders_sorceress_choice sorceress_dungeon_action_labels_include_spell_hotkeys
```

Expected: FAIL because character creation only supports Warrior/Rogue and dungeon labels do not include Sorceress.

- [ ] **Step 3: Add three-class selection helpers**

In `src/save.rs`, add these helpers near `CharacterCreationState`:

```rust
const CHARACTER_CREATION_CLASSES: [CharacterClass; 3] = [
    CharacterClass::Warrior,
    CharacterClass::Rogue,
    CharacterClass::Sorceress,
];

fn selected_class_index(selected_class: CharacterClass) -> usize {
    CHARACTER_CREATION_CLASSES
        .iter()
        .position(|class| *class == selected_class)
        .unwrap_or(0)
}

fn class_after(selected_class: CharacterClass) -> CharacterClass {
    let index = selected_class_index(selected_class);
    CHARACTER_CREATION_CLASSES[(index + 1) % CHARACTER_CREATION_CLASSES.len()]
}

fn class_before(selected_class: CharacterClass) -> CharacterClass {
    let index = selected_class_index(selected_class);
    CHARACTER_CREATION_CLASSES
        [(index + CHARACTER_CREATION_CLASSES.len() - 1) % CHARACTER_CREATION_CLASSES.len()]
}
```

Update `CharacterCreationState::handle_key` class cases:

```rust
(CharacterCreationStep::Class, '1') => {
    self.selected_class = CharacterClass::Warrior;
    self.message.clear();
}
(CharacterCreationStep::Class, '2') => {
    self.selected_class = CharacterClass::Rogue;
    self.message.clear();
}
(CharacterCreationStep::Class, '3') => {
    self.selected_class = CharacterClass::Sorceress;
    self.message.clear();
}
(CharacterCreationStep::Class, KEY_ARROW_UP) => {
    self.selected_class = class_before(self.selected_class);
    self.message.clear();
}
(CharacterCreationStep::Class, KEY_ARROW_DOWN) => {
    self.selected_class = class_after(self.selected_class);
    self.message.clear();
}
```

- [ ] **Step 4: Render Sorceress in character creation**

Increase the class layout row in `render_character_creation_screen` from `Constraint::Length(4)` to `Constraint::Length(5)` so three class lines fit. Add a `sorceress_marker` matching the existing Warrior/Rogue marker pattern. Add a third line to the class paragraph:

```rust
Line::styled(
    format!("{sorceress_marker} Sorceress - elemental spells, Mana, and a focus."),
    if active_step == CharacterCreationStep::Class
        && selected_class == CharacterClass::Sorceress
    {
        selected_cursor_style()
    } else {
        Style::default().fg(Color::Blue)
    },
),
```

Update the class-step command string:

```rust
CharacterCreationStep::Class => "Up/Down or 1/2/3=class  Enter=next  Esc=back",
```

- [ ] **Step 5: Add Sorceress dungeon labels and known command**

In `src/dungeon.rs`, update `dungeon_action_label_for`:

```rust
(CharacterClass::Sorceress, '1') => "Firebolt",
(CharacterClass::Sorceress, '2') => "Frost Ring",
(CharacterClass::Sorceress, '3') => "Chain Spark",
(CharacterClass::Sorceress, '4') => "Mana Shield",
```

Update `is_known_dungeon_command_for` so `4` is known for Rogue and Sorceress:

```rust
|| ((c.class == CharacterClass::Rogue || c.class == CharacterClass::Sorceress) && key == '4')
```

Do not route hotkey `4` to Sorceress combat behavior in this task; Task 4 adds the help text and Task 5 adds active dispatch.

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test character_creation_can_select_sorceress character_creation_renders_sorceress_choice sorceress_dungeon_action_labels_include_spell_hotkeys
```

Expected: PASS.

## Task 3: Sorceress Scaling Helpers And Skill Tree

**Files:**
- Create: `src/sorceress.rs`
- Modify: `src/main.rs`
- Modify: `src/skills.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests for scaling helpers and Sorceress skill tree**

Add these tests near the skill tree tests:

```rust
#[test]
fn sorceress_scaling_helpers_match_mvp_numbers() {
    assert_eq!((1..=5).map(firebolt_percent_for_rank).collect::<Vec<_>>(), vec![100, 110, 120, 130, 140]);
    assert_eq!((1..=5).map(firebolt_burn_chance_for_rank).collect::<Vec<_>>(), vec![25, 30, 35, 40, 45]);
    assert_eq!((1..=5).map(frost_ring_percent_for_rank).collect::<Vec<_>>(), vec![70, 80, 90, 100, 110]);
    assert_eq!((1..=5).map(frost_ring_freeze_chance_for_rank).collect::<Vec<_>>(), vec![20, 25, 30, 35, 40]);
    assert_eq!((1..=5).map(chain_spark_percent_for_rank).collect::<Vec<_>>(), vec![80, 90, 95, 105, 110]);
    assert_eq!((1..=5).map(chain_spark_hit_count_for_rank).collect::<Vec<_>>(), vec![2, 2, 3, 3, 4]);
    assert_eq!((1..=5).map(mana_shield_absorb_percent_for_rank).collect::<Vec<_>>(), vec![35, 40, 45, 50, 55]);
    assert_eq!((1..=5).map(kindle_fire_bonus_percent_for_rank).collect::<Vec<_>>(), vec![10, 15, 20, 25, 30]);
    assert_eq!((1..=5).map(static_charge_chance_for_rank).collect::<Vec<_>>(), vec![15, 20, 25, 30, 35]);
    assert_eq!((1..=5).map(static_charge_damage_bonus_for_rank).collect::<Vec<_>>(), vec![15, 20, 25, 30, 35]);
}

#[test]
fn sorceress_skill_tree_shows_branches_and_locked_unlocks() {
    let c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );

    let text = skill_tree_lines(&c, 0, "")
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(text.contains("Sorceress Skill Tree"));
    assert!(text.contains("Flame Branch"));
    assert!(text.contains("> Firebolt rank 1/5"));
    assert!(text.contains("└─🔒︎ Kindle unlocks at Firebolt rank 2 (1/2)"));
    assert!(text.contains("Frost Branch"));
    assert!(text.contains("Frost Ring rank 1/5"));
    assert!(text.contains("└─🔒︎ Mana Shield unlocks at Frost Ring rank 2 (1/2)"));
    assert!(text.contains("Storm Branch"));
    assert!(text.contains("Chain Spark rank 1/5"));
    assert!(text.contains("└─🔒︎ Static Charge unlocks at Chain Spark rank 2 (1/2)"));
    assert!(!text.contains("Kindle rank 0/5"));
    assert!(!text.contains("Mana Shield rank 0/5"));
    assert!(!text.contains("Static Charge rank 0/5"));
}

#[test]
fn sorceress_skill_tree_upgrades_unlockable_skills_with_prerequisites() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.unspent_skills = 4;

    assert_eq!(
        choose_skill_or_mastery(&mut c, "Mana Shield"),
        "Mana Shield upgrades require Frost Ring rank 2."
    );
    assert_eq!(choose_skill_or_mastery(&mut c, "Frost Ring"), "Upgraded Frost Ring to rank 2.");
    assert_eq!(choose_skill_or_mastery(&mut c, "Mana Shield"), "Upgraded Mana Shield to rank 1.");
    assert_eq!(choose_skill_or_mastery(&mut c, "Firebolt"), "Upgraded Firebolt to rank 2.");
    assert_eq!(choose_skill_or_mastery(&mut c, "Kindle"), "Upgraded Kindle to rank 1.");

    assert_eq!(c.sorceress.frost_ring_rank, 2);
    assert_eq!(c.sorceress.mana_shield_rank, 1);
    assert_eq!(c.sorceress.firebolt_rank, 2);
    assert_eq!(c.sorceress.kindle_rank, 1);
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test sorceress_scaling_helpers_match_mvp_numbers sorceress_skill_tree_shows_branches_and_locked_unlocks sorceress_skill_tree_upgrades_unlockable_skills_with_prerequisites
```

Expected: FAIL because `src/sorceress.rs` and Sorceress skill tree routing do not exist.

- [ ] **Step 3: Create `src/sorceress.rs` with rank helpers**

Create `src/sorceress.rs`:

```rust
use crate::*;

pub(crate) const FIREBOLT_MANA_COST: u32 = 4;
pub(crate) const FROST_RING_MANA_COST: u32 = 8;
pub(crate) const CHAIN_SPARK_MANA_COST: u32 = 7;
pub(crate) const FROST_RING_COOLDOWN: u32 = 3;
pub(crate) const CHAIN_SPARK_COOLDOWN: u32 = 2;
pub(crate) const BURNING_TURNS: u32 = 3;
pub(crate) const FROZEN_TURNS: u32 = 1;
pub(crate) const CHAIN_SPARK_JUMP_RADIUS: i32 = 2;

pub(crate) fn firebolt_percent_for_rank(rank: u32) -> u32 {
    100 + rank.saturating_sub(1).min(4) * 10
}

pub(crate) fn firebolt_burn_chance_for_rank(rank: u32) -> u32 {
    25 + rank.saturating_sub(1).min(4) * 5
}

pub(crate) fn frost_ring_percent_for_rank(rank: u32) -> u32 {
    70 + rank.saturating_sub(1).min(4) * 10
}

pub(crate) fn frost_ring_freeze_chance_for_rank(rank: u32) -> u32 {
    20 + rank.saturating_sub(1).min(4) * 5
}

pub(crate) fn chain_spark_percent_for_rank(rank: u32) -> u32 {
    match rank.min(5) {
        0 | 1 => 80,
        2 => 90,
        3 => 95,
        4 => 105,
        _ => 110,
    }
}

pub(crate) fn chain_spark_hit_count_for_rank(rank: u32) -> usize {
    match rank.min(5) {
        0 | 1 | 2 => 2,
        3 | 4 => 3,
        _ => 4,
    }
}

pub(crate) fn mana_shield_absorb_percent_for_rank(rank: u32) -> u32 {
    35 + rank.saturating_sub(1).min(4) * 5
}

pub(crate) fn kindle_fire_bonus_percent_for_rank(rank: u32) -> u32 {
    if rank == 0 { 0 } else { 10 + rank.saturating_sub(1).min(4) * 5 }
}

pub(crate) fn static_charge_chance_for_rank(rank: u32) -> u32 {
    if rank == 0 { 0 } else { 15 + rank.saturating_sub(1).min(4) * 5 }
}

pub(crate) fn static_charge_damage_bonus_for_rank(rank: u32) -> u32 {
    static_charge_chance_for_rank(rank)
}
```

Add to `src/main.rs`:

```rust
mod sorceress;
```

and re-export it with the other modules:

```rust
pub(crate) use sorceress::*;
```

- [ ] **Step 4: Add Sorceress skill tree routing**

In `src/skills.rs`, add:

```rust
const SORCERESS_SKILL_TREE_SKILLS: [&str; 6] = [
    "Firebolt",
    "Kindle",
    "Frost Ring",
    "Mana Shield",
    "Chain Spark",
    "Static Charge",
];
```

Update `skill_tree_skills`, `render_skill_tree_screen`, and `skill_tree_lines` matches with Sorceress. Add `sorceress_skill_tree_lines`:

```rust
fn sorceress_skill_tree_lines(
    c: &Character,
    selected_skill: &str,
    message: &str,
) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::styled(
            "Sorceress Skill Tree",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        skill_line(strip_ansi_codes(&unspent_skills_text(c.unspent_skills))),
    ];
    if !message.is_empty() {
        lines.push(Line::styled(
            message.to_string(),
            Style::default().fg(Color::Yellow),
        ));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled(
        "Flame Branch",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    append_skill_choice_lines(&mut lines, selected_skill, "Firebolt", c.sorceress.firebolt_rank);
    append_passive_unlock_line(&mut lines, c, "Kindle");
    if !skill_is_locked(c, "Kindle") {
        append_skill_choice_lines(&mut lines, selected_skill, "Kindle", c.sorceress.kindle_rank);
    }
    lines.push(Line::styled(
        "Frost Branch",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    append_skill_choice_lines(&mut lines, selected_skill, "Frost Ring", c.sorceress.frost_ring_rank);
    append_passive_unlock_line(&mut lines, c, "Mana Shield");
    if !skill_is_locked(c, "Mana Shield") {
        append_skill_choice_lines(&mut lines, selected_skill, "Mana Shield", c.sorceress.mana_shield_rank);
    }
    lines.push(Line::styled(
        "Storm Branch",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    append_skill_choice_lines(&mut lines, selected_skill, "Chain Spark", c.sorceress.chain_spark_rank);
    append_passive_unlock_line(&mut lines, c, "Static Charge");
    if !skill_is_locked(c, "Static Charge") {
        append_skill_choice_lines(&mut lines, selected_skill, "Static Charge", c.sorceress.static_charge_rank);
    }
    lines.push(Line::from(""));
    lines.push(skill_line("Each rank upgrade costs 1 skill point. Masteries are not available for Sorceress MVP."));
    lines
}
```

- [ ] **Step 5: Add Sorceress rank, prerequisites, effects, and upgrades**

Update `skill_effect_lines` with these exact Sorceress entries:

```rust
"Firebolt" => vec![
    format!("{}% spell damage", firebolt_percent_for_rank(rank)),
    format!("{} mana, no cooldown", FIREBOLT_MANA_COST),
    format!("{}% chance to apply Burning.", firebolt_burn_chance_for_rank(rank)),
],
"Kindle" => vec![
    format!("Burning enemies take +{}% fire damage.", kindle_fire_bonus_percent_for_rank(rank)),
    "Passive; requires Firebolt rank 2.".to_string(),
],
"Frost Ring" => vec![
    format!("{}% spell damage to all 8 surrounding tiles", frost_ring_percent_for_rank(rank)),
    format!("{} mana, cooldown {}", FROST_RING_MANA_COST, FROST_RING_COOLDOWN),
    format!("{}% chance to Freeze on hit.", frost_ring_freeze_chance_for_rank(rank)),
],
"Mana Shield" => vec![
    format!("Redirects {}% incoming damage to mana.", mana_shield_absorb_percent_for_rank(rank)),
    "Free toggle; 1 mana prevents 1 damage.".to_string(),
    "Requires Frost Ring rank 2.".to_string(),
],
"Chain Spark" => vec![
    format!("{}% spell damage", chain_spark_percent_for_rank(rank)),
    format!("{} mana, cooldown {}", CHAIN_SPARK_MANA_COST, CHAIN_SPARK_COOLDOWN),
    format!("Hits up to {} enemies; jumps within radius {}.", chain_spark_hit_count_for_rank(rank), CHAIN_SPARK_JUMP_RADIUS),
],
"Static Charge" => vec![
    format!("Chain Spark has {}% chance to apply Shocked.", static_charge_chance_for_rank(rank)),
    format!("Shocked stores +{}% damage taken for the next hit.", static_charge_damage_bonus_for_rank(rank)),
    "Passive; requires Chain Spark rank 2.".to_string(),
],
```

Update `upgrade_skill`, `skill_rank`, `passive_prerequisite`, and `normalize_locked_skill_ranks` with Sorceress fields:

```rust
"Firebolt" if c.class == CharacterClass::Sorceress => c.sorceress.firebolt_rank += 1,
"Frost Ring" if c.class == CharacterClass::Sorceress => c.sorceress.frost_ring_rank += 1,
"Chain Spark" if c.class == CharacterClass::Sorceress => c.sorceress.chain_spark_rank += 1,
"Kindle" if c.class == CharacterClass::Sorceress => c.sorceress.kindle_rank += 1,
"Mana Shield" if c.class == CharacterClass::Sorceress => c.sorceress.mana_shield_rank += 1,
"Static Charge" if c.class == CharacterClass::Sorceress => c.sorceress.static_charge_rank += 1,
```

Sorceress prerequisites:

```rust
"Kindle" => Some(SkillPrerequisite {
    starter: "Firebolt",
    current_rank: c.sorceress.firebolt_rank,
    required_rank: 2,
}),
"Mana Shield" => Some(SkillPrerequisite {
    starter: "Frost Ring",
    current_rank: c.sorceress.frost_ring_rank,
    required_rank: 2,
}),
"Static Charge" => Some(SkillPrerequisite {
    starter: "Chain Spark",
    current_rank: c.sorceress.chain_spark_rank,
    required_rank: 2,
}),
```

Normalization:

```rust
if c.sorceress.firebolt_rank < 2 {
    c.sorceress.kindle_rank = 0;
}
if c.sorceress.frost_ring_rank < 2 {
    c.sorceress.mana_shield_rank = 0;
    c.sorceress.mana_shield_active = false;
}
if c.sorceress.chain_spark_rank < 2 {
    c.sorceress.static_charge_rank = 0;
}
```

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test sorceress_scaling_helpers_match_mvp_numbers sorceress_skill_tree_shows_branches_and_locked_unlocks sorceress_skill_tree_upgrades_unlockable_skills_with_prerequisites
```

Expected: PASS.

## Task 4: Dungeon Help, Cooldowns, Status Fields, And Mana Shield Absorption

**Files:**
- Modify: `src/model.rs`
- Modify: `src/dungeon.rs`
- Modify: `src/sorceress.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests for dungeon help, cooldown ticking, status ticking, and Mana Shield absorption**

Add these tests near existing class dungeon help tests:

```rust
#[test]
fn sorceress_skill_help_lines_show_mana_cooldowns_and_locked_mana_shield() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.mana = 27;
    c.sorceress.frost_ring_cooldown = 2;
    c.sorceress.chain_spark_cooldown = 1;

    let rendered = dungeon_skill_help_lines(&c)
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rendered.contains("Sorceress: Mana 27/40  Mana Shield off"));
    assert!(rendered.contains("1 Firebolt r1: cost 4 mana. 100% spell damage; 25% Burning."));
    assert!(rendered.contains("2 Frost Ring r1: cost 8 mana, cd 3. 8 tiles; 70% damage; 20% Freeze. Ready in 2."));
    assert!(rendered.contains("3 Chain Spark r1: cost 7 mana, cd 2. 80% damage; up to 2 hits. Ready in 1."));
    assert!(rendered.contains("4 Mana Shield: locked; requires Frost Ring rank 2."));
}

#[test]
fn sorceress_unlocked_mana_shield_help_shows_absorption_and_state() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.mana_shield_rank = 3;
    c.sorceress.mana_shield_active = true;

    let rendered = dungeon_skill_help_lines(&c)
        .iter()
        .map(line_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rendered.contains("Sorceress: Mana 40/40  Mana Shield on"));
    assert!(rendered.contains("4 Mana Shield r3: free toggle. Absorbs 45% at 1 mana per damage."));
}

#[test]
fn sorceress_cooldowns_tick_and_clear_with_combat_state() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.frost_ring_cooldown = 2;
    c.sorceress.chain_spark_cooldown = 1;
    c.sorceress.mana_shield_rank = 1;
    c.sorceress.mana_shield_active = true;

    tick_player_effects(&mut c);

    assert_eq!(c.sorceress.frost_ring_cooldown, 1);
    assert_eq!(c.sorceress.chain_spark_cooldown, 0);
    assert!(c.sorceress.mana_shield_active);

    clear_combat_state(&mut c);

    assert_eq!(c.sorceress.frost_ring_cooldown, 0);
    assert_eq!(c.sorceress.chain_spark_cooldown, 0);
    assert!(!c.sorceress.mana_shield_active);
}

#[test]
fn mana_shield_absorbs_rank_scaled_damage_using_mana() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.mana_shield_rank = 1;
    c.sorceress.mana_shield_active = true;
    c.hp = 15;
    c.mana = 40;

    apply_player_damage(&mut c, 10);

    assert_eq!(c.mana, 37);
    assert_eq!(c.hp, 8);
    assert!(c.sorceress.mana_shield_active);

    c.sorceress.mana_shield_rank = 5;
    c.sorceress.mana_shield_active = true;
    c.hp = 15;
    c.mana = 2;

    apply_player_damage(&mut c, 10);

    assert_eq!(c.mana, 0);
    assert_eq!(c.hp, 7);
    assert!(!c.sorceress.mana_shield_active);
}

#[test]
fn burning_and_frozen_enemy_effects_tick_during_enemy_turns() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    let mut burning = skeleton(5, 5);
    burning.name = "Burning Dummy".to_string();
    burning.hp = 10;
    burning.max_hp = 10;
    burning.burning_turns = 1;
    burning.burning_damage = 2;
    let mut frozen = skeleton(3, 2);
    frozen.name = "Frozen Dummy".to_string();
    frozen.frozen_turns = 1;
    frozen.energy = 999;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![burning, frozen]));
    let before_hp = c.hp;

    enemy_turns(&mut c);

    let d = c.active_dungeon.as_ref().unwrap();
    let burning = d.enemies.iter().find(|enemy| enemy.name == "Burning Dummy").unwrap();
    let frozen = d.enemies.iter().find(|enemy| enemy.name == "Frozen Dummy").unwrap();
    assert_eq!(burning.hp, 8);
    assert_eq!(burning.burning_turns, 0);
    assert_eq!(frozen.frozen_turns, 0);
    assert_eq!(c.hp, before_hp);
    assert!(d.log.iter().any(|line| line.contains("Burning Dummy burns for")));
    assert!(d.log.iter().any(|line| line.contains("Frozen Dummy is frozen and skips its turn.")));
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test sorceress_skill_help_lines_show_mana_cooldowns_and_locked_mana_shield sorceress_unlocked_mana_shield_help_shows_absorption_and_state sorceress_cooldowns_tick_and_clear_with_combat_state mana_shield_absorbs_rank_scaled_damage_using_mana burning_and_frozen_enemy_effects_tick_during_enemy_turns
```

Expected: FAIL because Sorceress dungeon help, cooldown ticking, status fields, and Mana Shield absorption do not exist.

- [ ] **Step 3: Add Enemy status fields**

In `src/model.rs`, add these fields to `Enemy` after poison fields:

```rust
#[serde(default)]
pub(crate) burning_turns: u32,
#[serde(default)]
pub(crate) burning_damage: i32,
#[serde(default)]
pub(crate) frozen_turns: u32,
#[serde(default)]
pub(crate) shocked_bonus_percent: u32,
```

No save-version bump is needed because all fields have serde defaults.

- [ ] **Step 4: Add Sorceress dungeon help and commands**

In `src/dungeon.rs`, update `dungeon_skill_help_lines` and `dungeon_command_entries` with Sorceress branches. Add:

```rust
fn sorceress_dungeon_skill_help_lines(c: &Character) -> Vec<Line<'static>> {
    let shield_state = if c.sorceress.mana_shield_active { "on" } else { "off" };
    let mut lines = vec![
        Line::from(format!(
            "Sorceress: Mana {}/{}  Mana Shield {shield_state}",
            c.mana,
            c.max_mana()
        )),
        Line::from(format!(
            "1 Firebolt r{}: cost {} mana. {}% spell damage; {}% Burning.",
            c.sorceress.firebolt_rank,
            FIREBOLT_MANA_COST,
            firebolt_percent_for_rank(c.sorceress.firebolt_rank),
            firebolt_burn_chance_for_rank(c.sorceress.firebolt_rank)
        )),
        Line::from(format!(
            "2 Frost Ring r{}: cost {} mana, cd {}. 8 tiles; {}% damage; {}% Freeze. Ready in {}.",
            c.sorceress.frost_ring_rank,
            FROST_RING_MANA_COST,
            FROST_RING_COOLDOWN,
            frost_ring_percent_for_rank(c.sorceress.frost_ring_rank),
            frost_ring_freeze_chance_for_rank(c.sorceress.frost_ring_rank),
            c.sorceress.frost_ring_cooldown
        )),
        Line::from(format!(
            "3 Chain Spark r{}: cost {} mana, cd {}. {}% damage; up to {} hits. Ready in {}.",
            c.sorceress.chain_spark_rank,
            CHAIN_SPARK_MANA_COST,
            CHAIN_SPARK_COOLDOWN,
            chain_spark_percent_for_rank(c.sorceress.chain_spark_rank),
            chain_spark_hit_count_for_rank(c.sorceress.chain_spark_rank),
            c.sorceress.chain_spark_cooldown
        )),
    ];
    if c.sorceress.mana_shield_rank == 0 {
        lines.push(Line::from("4 Mana Shield: locked; requires Frost Ring rank 2."));
    } else {
        lines.push(Line::from(format!(
            "4 Mana Shield r{}: free toggle. Absorbs {}% at 1 mana per damage.",
            c.sorceress.mana_shield_rank,
            mana_shield_absorb_percent_for_rank(c.sorceress.mana_shield_rank)
        )));
    }
    lines
}
```

Sorceress command entries:

```rust
CharacterClass::Sorceress => vec![
    ("w/a/s/d", "move/attack"),
    ("1", "Firebolt"),
    ("2", "Frost"),
    ("3", "Spark"),
    ("4", "Shield"),
    ("p", "potion"),
    ("g", "pickup"),
    ("i", "inventory"),
    ("Esc", "town"),
],
```

- [ ] **Step 5: Tick and clear Sorceress state**

In `clear_combat_state`, set:

```rust
c.sorceress.frost_ring_cooldown = 0;
c.sorceress.chain_spark_cooldown = 0;
c.sorceress.mana_shield_active = false;
```

In `tick_player_effects`, decrement:

```rust
c.sorceress.frost_ring_cooldown = c.sorceress.frost_ring_cooldown.saturating_sub(1);
c.sorceress.chain_spark_cooldown = c.sorceress.chain_spark_cooldown.saturating_sub(1);
```

- [ ] **Step 6: Apply Mana Shield absorption**

In `apply_player_damage`, after Warrior `second_wind_shield` absorption and before HP loss, add Sorceress absorption. Use integer floor for the percentage calculation:

```rust
let mut remaining = damage - absorbed;
if c.class == CharacterClass::Sorceress && c.sorceress.mana_shield_active {
    if c.sorceress.mana_shield_rank == 0 || c.mana == 0 {
        c.sorceress.mana_shield_active = false;
    } else {
        let desired_absorb = remaining
            .saturating_mul(mana_shield_absorb_percent_for_rank(c.sorceress.mana_shield_rank))
            / 100;
        let mana_absorbed = desired_absorb.min(c.mana);
        c.mana -= mana_absorbed;
        remaining = remaining.saturating_sub(mana_absorbed);
        if c.mana == 0 {
            c.sorceress.mana_shield_active = false;
        }
    }
}
c.hp = c.hp.saturating_sub(remaining);
```

Keep Warrior `second_wind_shield` behavior unchanged.

- [ ] **Step 7: Tick Burning and Frozen in enemy turns**

In `enemy_turns`, after poison ticking and before enemy energy gain, add Burning ticking:

```rust
if d.enemies[i].burning_turns > 0 {
    let burning_damage = d.enemies[i].burning_damage.max(1);
    d.enemies[i].hp -= burning_damage;
    d.enemies[i].burning_turns -= 1;
    log_event(
        &mut d.log,
        LogKind::Hit,
        format!(
            "{} burns for {}. {}.",
            d.enemies[i].name,
            damage_text(burning_damage),
            enemy_hp_text(&d.enemies[i])
        ),
    );
    if d.enemies[i].hp <= 0 {
        let ground_items_before_death = d.ground_items.len();
        if resolve_enemy_death(c, &mut d, i, EnemyDeathCause::Effect { source: "Burning" }) {
            finish_boss_defeat_after_effect_kill(c, d, ground_items_before_death);
            return;
        }
        continue;
    }
}
```

After enemy energy threshold spending and before stunned handling, add Frozen skip handling:

```rust
if d.enemies[i].frozen_turns > 0 {
    d.enemies[i].frozen_turns -= 1;
    log_event(
        &mut d.log,
        LogKind::Status,
        format!("{} is frozen and skips its turn.", d.enemies[i].name),
    );
    continue;
}
```

- [ ] **Step 8: Run focused tests**

Run:

```bash
cargo test sorceress_skill_help_lines_show_mana_cooldowns_and_locked_mana_shield sorceress_unlocked_mana_shield_help_shows_absorption_and_state sorceress_cooldowns_tick_and_clear_with_combat_state mana_shield_absorbs_rank_scaled_damage_using_mana burning_and_frozen_enemy_effects_tick_during_enemy_turns
```

Expected: PASS.

## Task 5: Firebolt, Frost Ring, Spell Damage, And Mana Shield Toggle

**Files:**
- Modify: `src/sorceress.rs`
- Modify: `src/dungeon.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests for Firebolt, Frost Ring, and Mana Shield hotkey behavior**

Add these tests near Rogue active skill tests:

```rust
#[test]
fn firebolt_requires_line_of_sight_and_spends_no_mana_without_target() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    let enemy = skeleton(5, 2);
    let mut d = open_test_dungeon(2, 2, vec![enemy]);
    d.tiles[tile_index(3, 2)] = '#';
    c.active_dungeon = Some(d);
    let before_mana = c.mana;

    assert!(!use_firebolt_with_rolls(&mut c, 0.0, 0.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.mana, before_mana);
    assert_eq!(d.enemies[0].hp, d.enemies[0].max_hp);
    assert!(d.log.iter().any(|line| line.contains("No enemy in sight.")));
}

#[test]
fn firebolt_miss_spends_mana_and_turn_without_burning() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![skeleton(5, 2)]));
    let before_mana = c.mana;

    assert!(use_firebolt_with_rolls(&mut c, 1.0, 0.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.mana, before_mana - FIREBOLT_MANA_COST);
    assert_eq!(d.enemies[0].hp, d.enemies[0].max_hp);
    assert_eq!(d.enemies[0].burning_turns, 0);
    assert!(d.log.iter().any(|line| line.contains("Firebolt misses")));
}

#[test]
fn firebolt_hit_uses_int_spell_damage_and_can_apply_burning() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.strength = 0;
    c.intelligence = 6;
    let mut enemy = skeleton(5, 2);
    enemy.armor = 0;
    enemy.hp = 30;
    enemy.max_hp = 30;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![enemy]));

    assert!(use_firebolt_with_rolls(&mut c, 0.0, 0.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d.enemies[0].hp < 30);
    assert_eq!(d.enemies[0].burning_turns, BURNING_TURNS);
    assert!(d.enemies[0].burning_damage > 0);
    assert!(d.log.iter().any(|line| line.contains("Firebolt burns")));
}

#[test]
fn frost_ring_hits_all_eight_surrounding_tiles_and_freezes_on_chance() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    let adjacent = [
        (1, 1), (2, 1), (3, 1),
        (1, 2),         (3, 2),
        (1, 3), (2, 3), (3, 3),
    ];
    let mut enemies = adjacent
        .iter()
        .enumerate()
        .map(|(index, (x, y))| {
            let mut enemy = skeleton(*x, *y);
            enemy.name = format!("Frost Dummy {index}");
            enemy.armor = 0;
            enemy.hp = 20;
            enemy.max_hp = 20;
            enemy
        })
        .collect::<Vec<_>>();
    let mut far = skeleton(5, 5);
    far.name = "Far Dummy".to_string();
    far.hp = 20;
    far.max_hp = 20;
    enemies.push(far);
    c.active_dungeon = Some(open_test_dungeon(2, 2, enemies));

    assert!(use_frost_ring_with_rolls(&mut c, 0.0, 0.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.mana, c.max_mana() - FROST_RING_MANA_COST);
    assert_eq!(c.sorceress.frost_ring_cooldown, FROST_RING_COOLDOWN);
    for enemy in d.enemies.iter().filter(|enemy| enemy.name.starts_with("Frost Dummy")) {
        assert!(enemy.hp < 20, "{} was not damaged", enemy.name);
        assert_eq!(enemy.frozen_turns, FROZEN_TURNS, "{} was not frozen", enemy.name);
    }
    let far = d.enemies.iter().find(|enemy| enemy.name == "Far Dummy").unwrap();
    assert_eq!(far.hp, 20);
    assert_eq!(far.frozen_turns, 0);
}

#[test]
fn mana_shield_hotkey_toggles_freely_after_unlock() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));

    assert!(!handle_class_skill_key(&mut c, '4'));
    assert!(!c.sorceress.mana_shield_active);
    assert!(c
        .active_dungeon
        .as_ref()
        .unwrap()
        .log
        .iter()
        .any(|line| line.contains("Mana Shield requires Frost Ring rank 2.")));

    c.sorceress.mana_shield_rank = 1;
    assert!(!handle_class_skill_key(&mut c, '4'));
    assert!(c.sorceress.mana_shield_active);
    assert!(!handle_class_skill_key(&mut c, '4'));
    assert!(!c.sorceress.mana_shield_active);
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test firebolt_requires_line_of_sight_and_spends_no_mana_without_target firebolt_miss_spends_mana_and_turn_without_burning firebolt_hit_uses_int_spell_damage_and_can_apply_burning frost_ring_hits_all_eight_surrounding_tiles_and_freezes_on_chance mana_shield_hotkey_toggles_freely_after_unlock
```

Expected: FAIL because spell active functions, line-of-sight targeting, and Mana Shield dispatch do not exist.

- [ ] **Step 3: Add spell damage and visibility helpers**

In `src/sorceress.rs`, add these helpers:

```rust
pub(crate) fn spell_damage_range(c: &Character) -> (i32, i32) {
    let int_bonus = (c.effective_intelligence() as i32 / 3).max(0);
    (
        (c.equipped_weapon.damage_min + int_bonus).max(1),
        (c.equipped_weapon.damage_max + int_bonus).max(1),
    )
}

pub(crate) fn spell_damage_for_roll(c: &Character, percent: u32, roll: f64) -> i32 {
    let (min, max) = spell_damage_range(c);
    let span = (max - min).max(0);
    let rolled = min + ((span as f64 * roll.clamp(0.0, 1.0)).round() as i32);
    ((rolled as f32) * (percent as f32 / 100.0)).round().max(1.0) as i32
}

pub(crate) fn nearest_visible_enemy_index(c: &Character) -> Option<usize> {
    let d = c.active_dungeon.as_ref()?;
    d.enemies
        .iter()
        .enumerate()
        .filter(|(_, enemy)| enemy.hp > 0 && clear_line_of_sight(d, d.player_x, d.player_y, enemy.x, enemy.y))
        .min_by_key(|(_, enemy)| {
            let dx = (enemy.x - d.player_x).abs();
            let dy = (enemy.y - d.player_y).abs();
            (dx + dy, dy.max(dx), enemy.y, enemy.x)
        })
        .map(|(index, _)| index)
}

pub(crate) fn clear_line_of_sight(d: &Dungeon, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> bool {
    let mut x0 = from_x;
    let mut y0 = from_y;
    let x1 = to_x;
    let y1 = to_y;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if (x0, y0) != (from_x, from_y) && (x0, y0) != (to_x, to_y) && dungeon_tile(d, x0, y0) == '#' {
            return false;
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    true
}
```

- [ ] **Step 4: Add direct spell damage helper and Firebolt**

Add a helper in `src/sorceress.rs` that applies hit, armor, Kindle, Burning, Shocked bonus consumption, death resolution, and logs for one spell hit. The helper must use hit rolls against `player_attack_hit_chance(c, enemy)` and must not use Strength-based `Character::weapon_damage`.

Use this public test wrapper signature:

```rust
pub(crate) fn use_firebolt_with_rolls(
    c: &mut Character,
    hit_roll: f64,
    burn_roll: f64,
    damage_roll: f64,
) -> bool
```

Behavior:

- If `c.mana < FIREBOLT_MANA_COST`, log `Not enough mana for Firebolt.` and return `false`.
- If `nearest_visible_enemy_index(c)` returns `None`, log `No enemy in sight.` and return `false` without spending mana.
- Spend 4 mana once a target exists.
- If `hit_roll >= player_attack_hit_chance(c, enemy)`, log `Firebolt misses {name}.` and return `true`.
- On hit, deal `firebolt_percent_for_rank(c.sorceress.firebolt_rank)` spell damage.
- If the target has `burning_turns > 0` and Kindle rank is above 0, increase Firebolt damage by `kindle_fire_bonus_percent_for_rank` before armor mitigation.
- If `burn_roll < firebolt_burn_chance_for_rank(rank) as f64 / 100.0`, set `burning_turns = max(existing, BURNING_TURNS)` and `burning_damage = max(existing, 1 + rank.div_ceil(2) as i32)`, then log `Firebolt burns {name}.`.
- Resolve enemy death through `resolve_enemy_death` with `EnemyDeathCause::Effect { source: "Firebolt" }`.

Add the normal runtime wrapper:

```rust
pub(crate) fn use_firebolt(c: &mut Character) -> bool {
    let mut rng = rand::thread_rng();
    use_firebolt_with_rolls(
        c,
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
    )
}
```

- [ ] **Step 5: Add Frost Ring**

Add a public test wrapper:

```rust
pub(crate) fn use_frost_ring_with_rolls(
    c: &mut Character,
    hit_roll: f64,
    freeze_roll: f64,
    damage_roll: f64,
) -> bool
```

Behavior:

- If cooldown is above 0, log `Frost Ring is on cooldown for N more turns.` and return `false`.
- If mana is below 8, log `Not enough mana for Frost Ring.` and return `false`.
- Gather living enemies where `(enemy.x - player_x).abs() <= 1`, `(enemy.y - player_y).abs() <= 1`, and enemy is not on the player tile. This includes all eight surrounding tiles.
- If no adjacent enemies exist, log `No enemies in Frost Ring range.` and return `false` without spending mana.
- Spend 8 mana and set cooldown to 3 once at least one target exists.
- Roll hit separately for each target using the provided `hit_roll` in the test wrapper and random rolls in runtime wrapper.
- On hit, deal `frost_ring_percent_for_rank(rank)` spell damage.
- On hit and `freeze_roll < frost_ring_freeze_chance_for_rank(rank) as f64 / 100.0`, set `frozen_turns = max(existing, FROZEN_TURNS)` and log `Frost Ring freezes {name}.`.
- Resolve enemy death with `EnemyDeathCause::Effect { source: "Frost Ring" }`.

Add the normal runtime wrapper `use_frost_ring(c: &mut Character) -> bool` using random rolls per enemy. The runtime wrapper can call an internal function that accepts closures for hit, freeze, and damage rolls so tests stay deterministic.

- [ ] **Step 6: Add Mana Shield toggle and class hotkey dispatch**

In `src/sorceress.rs`, add:

```rust
pub(crate) fn toggle_mana_shield(c: &mut Character) -> bool {
    if c.sorceress.mana_shield_rank == 0 {
        if let Some(d) = c.active_dungeon.as_mut() {
            log_event(&mut d.log, LogKind::Warn, "Mana Shield requires Frost Ring rank 2.");
        }
        return false;
    }
    c.sorceress.mana_shield_active = !c.sorceress.mana_shield_active;
    if let Some(d) = c.active_dungeon.as_mut() {
        let state = if c.sorceress.mana_shield_active { "on" } else { "off" };
        log_event(&mut d.log, LogKind::Status, format!("Mana Shield toggled {state}."));
    }
    false
}
```

In `src/dungeon.rs`, update `handle_class_skill_key`:

```rust
(CharacterClass::Sorceress, '1') => use_firebolt(c),
(CharacterClass::Sorceress, '2') => use_frost_ring(c),
(CharacterClass::Sorceress, '4') => toggle_mana_shield(c),
```

Leave `CharacterClass::Sorceress, '3'` unhandled until Task 6 so Chain Spark tests drive that implementation.

- [ ] **Step 7: Run focused tests**

Run:

```bash
cargo test firebolt_requires_line_of_sight_and_spends_no_mana_without_target firebolt_miss_spends_mana_and_turn_without_burning firebolt_hit_uses_int_spell_damage_and_can_apply_burning frost_ring_hits_all_eight_surrounding_tiles_and_freezes_on_chance mana_shield_hotkey_toggles_freely_after_unlock
```

Expected: PASS.

## Task 6: Chain Spark, Jump Reachability, Static Charge, And Shocked Consumption

**Files:**
- Modify: `src/sorceress.rs`
- Modify: `src/dungeon.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests for Chain Spark and Shocked**

Add these tests near the Firebolt/Frost Ring tests:

```rust
#[test]
fn chain_spark_requires_initial_line_of_sight_and_miss_ends_chain() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![skeleton(5, 2), skeleton(6, 2)]));
    let before_mana = c.mana;

    assert!(use_chain_spark_with_rolls(&mut c, 1.0, 0.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert_eq!(c.mana, before_mana - CHAIN_SPARK_MANA_COST);
    assert_eq!(c.sorceress.chain_spark_cooldown, CHAIN_SPARK_COOLDOWN);
    assert!(d.enemies.iter().all(|enemy| enemy.hp == enemy.max_hp));
    assert!(d.log.iter().any(|line| line.contains("Chain Spark misses")));
}

#[test]
fn chain_spark_jumps_within_radius_two_including_diagonals() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.chain_spark_rank = 3;
    let mut first = skeleton(5, 2);
    first.name = "First".to_string();
    first.armor = 0;
    first.hp = 20;
    first.max_hp = 20;
    let mut diagonal = skeleton(7, 4);
    diagonal.name = "Diagonal".to_string();
    diagonal.armor = 0;
    diagonal.hp = 20;
    diagonal.max_hp = 20;
    let mut second_jump = skeleton(8, 5);
    second_jump.name = "Second Jump".to_string();
    second_jump.armor = 0;
    second_jump.hp = 20;
    second_jump.max_hp = 20;
    let mut too_far = skeleton(12, 8);
    too_far.name = "Too Far".to_string();
    too_far.hp = 20;
    too_far.max_hp = 20;
    c.active_dungeon = Some(open_test_dungeon(2, 2, vec![first, diagonal, second_jump, too_far]));

    assert!(use_chain_spark_with_rolls(&mut c, 0.0, 1.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d.enemies.iter().find(|enemy| enemy.name == "First").unwrap().hp < 20);
    assert!(d.enemies.iter().find(|enemy| enemy.name == "Diagonal").unwrap().hp < 20);
    assert!(d.enemies.iter().find(|enemy| enemy.name == "Second Jump").unwrap().hp < 20);
    assert_eq!(d.enemies.iter().find(|enemy| enemy.name == "Too Far").unwrap().hp, 20);
}

#[test]
fn chain_spark_jumps_around_corners_but_not_through_walls() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.sorceress.chain_spark_rank = 5;
    let mut first = skeleton(5, 2);
    first.name = "First".to_string();
    first.armor = 0;
    first.hp = 20;
    first.max_hp = 20;
    let mut around_corner = skeleton(6, 3);
    around_corner.name = "Around Corner".to_string();
    around_corner.armor = 0;
    around_corner.hp = 20;
    around_corner.max_hp = 20;
    let mut blocked = skeleton(7, 2);
    blocked.name = "Blocked".to_string();
    blocked.armor = 0;
    blocked.hp = 20;
    blocked.max_hp = 20;
    let mut d = open_test_dungeon(2, 2, vec![first, around_corner, blocked]);
    d.tiles[tile_index(6, 2)] = '#';
    d.tiles[tile_index(7, 1)] = '#';
    d.tiles[tile_index(7, 3)] = '#';
    c.active_dungeon = Some(d);

    assert!(use_chain_spark_with_rolls(&mut c, 0.0, 1.0, 0.0));

    let d = c.active_dungeon.as_ref().unwrap();
    assert!(d.enemies.iter().find(|enemy| enemy.name == "Around Corner").unwrap().hp < 20);
    assert_eq!(d.enemies.iter().find(|enemy| enemy.name == "Blocked").unwrap().hp, 20);
}

#[test]
fn static_charge_applies_shocked_and_replaces_only_with_equal_or_stronger_bonus() {
    let mut enemy = skeleton(5, 2);

    apply_shocked_if_stronger(&mut enemy, 25);
    assert_eq!(enemy.shocked_bonus_percent, 25);
    apply_shocked_if_stronger(&mut enemy, 15);
    assert_eq!(enemy.shocked_bonus_percent, 25);
    apply_shocked_if_stronger(&mut enemy, 25);
    assert_eq!(enemy.shocked_bonus_percent, 25);
    apply_shocked_if_stronger(&mut enemy, 35);
    assert_eq!(enemy.shocked_bonus_percent, 35);
}

#[test]
fn shocked_bonus_is_consumed_by_next_damaging_hit() {
    let mut enemy = skeleton(5, 2);
    enemy.shocked_bonus_percent = 25;

    let damage = apply_shock_bonus_to_damage(&mut enemy, 20);

    assert_eq!(damage, 25);
    assert_eq!(enemy.shocked_bonus_percent, 0);

    let damage_without_shock = apply_shock_bonus_to_damage(&mut enemy, 20);
    assert_eq!(damage_without_shock, 20);
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test chain_spark_requires_initial_line_of_sight_and_miss_ends_chain chain_spark_jumps_within_radius_two_including_diagonals chain_spark_jumps_around_corners_but_not_through_walls static_charge_applies_shocked_and_replaces_only_with_equal_or_stronger_bonus shocked_bonus_is_consumed_by_next_damaging_hit
```

Expected: FAIL because Chain Spark, jump reachability, and Shocked helpers do not exist.

- [ ] **Step 3: Add Shocked helpers and consume Shocked in existing direct damage**

In `src/sorceress.rs`, add:

```rust
pub(crate) fn apply_shocked_if_stronger(enemy: &mut Enemy, bonus_percent: u32) {
    if bonus_percent >= enemy.shocked_bonus_percent {
        enemy.shocked_bonus_percent = bonus_percent;
    }
}

pub(crate) fn apply_shock_bonus_to_damage(enemy: &mut Enemy, damage: i32) -> i32 {
    let bonus = enemy.shocked_bonus_percent;
    if bonus == 0 {
        return damage;
    }
    enemy.shocked_bonus_percent = 0;
    damage + ((damage * bonus as i32) / 100)
}
```

In `damage_enemy` in `src/dungeon.rs`, apply Shocked after armor, critical, and Vulnerable effects are calculated but before subtracting enemy HP:

```rust
let mut damage = (raw - armor).max(1);
if critical {
    damage *= 2;
}
damage = apply_shock_bonus_to_damage(enemy, damage);
enemy.hp -= damage;
```

Use the same helper inside Sorceress spell damage so Shocked is consumed by the next direct damaging hit from any class.

- [ ] **Step 4: Add Chain Spark jump reachability**

In `src/sorceress.rs`, add helpers that use 8-direction movement but block wall cutting:

```rust
fn open_for_chain_jump(d: &Dungeon, x: i32, y: i32) -> bool {
    dungeon_tile(d, x, y) != '#'
}

fn chain_jump_can_step(d: &Dungeon, from: (i32, i32), to: (i32, i32)) -> bool {
    if !open_for_chain_jump(d, to.0, to.1) {
        return false;
    }
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    if dx != 0 && dy != 0 {
        open_for_chain_jump(d, from.0 + dx, from.1) && open_for_chain_jump(d, from.0, from.1 + dy)
    } else {
        true
    }
}

pub(crate) fn chain_jump_reachable_tiles(d: &Dungeon, start: (i32, i32), max_steps: i32) -> Vec<(i32, i32)> {
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    visited.insert(start);
    queue.push_back((start, 0));
    while let Some((pos, steps)) = queue.pop_front() {
        if steps >= max_steps {
            continue;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let next = (pos.0 + dx, pos.1 + dy);
                if visited.contains(&next) || !chain_jump_can_step(d, pos, next) {
                    continue;
                }
                visited.insert(next);
                queue.push_back((next, steps + 1));
            }
        }
    }
    visited.remove(&start);
    let mut tiles = visited.into_iter().collect::<Vec<_>>();
    tiles.sort_by_key(|(x, y)| ((x - start.0).abs().max((y - start.1).abs()), *y, *x));
    tiles
}
```

- [ ] **Step 5: Add Chain Spark active skill**

Add a public test wrapper:

```rust
pub(crate) fn use_chain_spark_with_rolls(
    c: &mut Character,
    hit_roll: f64,
    shock_roll: f64,
    damage_roll: f64,
) -> bool
```

Jump candidate ordering is deterministic by distance, then row, then column.

Behavior:

- If cooldown is above 0, log `Chain Spark is on cooldown for N more turns.` and return `false`.
- If mana is below 7, log `Not enough mana for Chain Spark.` and return `false`.
- Initial target uses `nearest_visible_enemy_index(c)` and requires Sorceress line of sight.
- If no initial target exists, log `No enemy in sight.` and return `false` without spending mana.
- Spend 7 mana and set cooldown to 2 when initial target exists.
- If the initial hit misses, log `Chain Spark misses {name}.` and return `true` with no jumps.
- Rank hit count is `chain_spark_hit_count_for_rank`; rank 1-2 hits up to 2 enemies total, rank 3-4 hits up to 3, rank 5 hits up to 4.
- After each successful hit, choose the nearest unhit living enemy whose tile is in `chain_jump_reachable_tiles(d, previous_enemy_position, CHAIN_SPARK_JUMP_RADIUS)`.
- Each enemy can be hit once per cast.
- On each hit, deal `chain_spark_percent_for_rank(rank)` spell damage.
- If Static Charge rank is above 0 and `shock_roll < static_charge_chance_for_rank(rank) as f64 / 100.0`, apply `apply_shocked_if_stronger(enemy, static_charge_damage_bonus_for_rank(rank))` and log `Chain Spark shocks {name}.`.
- Resolve deaths with `EnemyDeathCause::Effect { source: "Chain Spark" }`.

Add runtime wrapper:

```rust
pub(crate) fn use_chain_spark(c: &mut Character) -> bool {
    let mut rng = rand::thread_rng();
    use_chain_spark_with_rolls(
        c,
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
    )
}
```

Use an internal roll-source closure so runtime Chain Spark rolls hit, Shocked, and damage separately for each target while the public test wrapper remains deterministic.

- [ ] **Step 6: Route Sorceress hotkey `3`**

In `handle_class_skill_key`, add:

```rust
(CharacterClass::Sorceress, '3') => use_chain_spark(c),
```

- [ ] **Step 7: Run focused tests**

Run:

```bash
cargo test chain_spark_requires_initial_line_of_sight_and_miss_ends_chain chain_spark_jumps_within_radius_two_including_diagonals chain_spark_jumps_around_corners_but_not_through_walls static_charge_applies_shocked_and_replaces_only_with_equal_or_stronger_bonus shocked_bonus_is_consumed_by_next_damaging_hit
```

Expected: PASS.

## Task 7: Sorceress Loot Pool, Equipment Restrictions, Docs, And Full Verification

**Files:**
- Modify: `src/dungeon.rs`
- Modify: `src/inventory.rs`
- Modify: `README.md`
- Modify: `design.md`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests for Sorceress equipment loot and equip restrictions**

Add these tests near existing loot/equipment class tests:

```rust
#[test]
fn sorceress_random_equipment_uses_wand_focus_pool_without_staves() {
    let mut seen_names = std::collections::HashSet::new();
    for _ in 0..300 {
        let loot = random_equipment_loot_for_class(CharacterClass::Sorceress, 3, false);
        seen_names.insert(loot.name);
    }

    assert!(seen_names.iter().any(|name| name.contains("Wand")));
    assert!(seen_names.iter().any(|name| name.contains("Focus")));
    assert!(seen_names.iter().any(|name| name.contains("Robe")));
    assert!(seen_names.iter().any(|name| name.contains("Circlet")));
    assert!(seen_names.iter().any(|name| name.contains("Spell Gloves")));
    assert!(seen_names.iter().any(|name| name.contains("Soft Slippers")));
    assert!(seen_names.iter().any(|name| name.contains("Sash")));
    assert!(seen_names.iter().any(|name| name.contains("Arcane Amulet")));
    assert!(seen_names.iter().any(|name| name.contains("Rune Ring")));
    assert!(!seen_names.iter().any(|name| name.contains("Staff")));
    assert!(!seen_names.iter().any(|name| name.contains("Dagger")));
    assert!(!seen_names.iter().any(|name| name.contains("Sword")));
    assert!(!seen_names.iter().any(|name| name.contains("Axe")));
}

#[test]
fn sorceress_can_equip_wands_and_focuses_but_not_other_weapons_or_shields() {
    let mut c = Character::new(
        "Lyra".to_string(),
        CharacterClass::Sorceress,
        DeathMode::Softcore,
    );
    c.strength = 99;
    c.dexterity = 99;
    c.intelligence = 99;

    assert!(can_equip_item(&c, &cracked_wand()));
    assert!(can_equip_item(&c, &cracked_focus()));
    assert!(!can_equip_item(&c, &training_dagger()));
    assert!(!can_equip_item(&c, &worn_shield()));

    c.inventory.push(training_dagger());
    let dagger_index = c
        .inventory
        .iter()
        .position(|item| item.name.contains("Dagger"))
        .unwrap();
    let result = equip_or_use_inventory_item(&mut c, dagger_index);

    assert_eq!(result.message, "Sorceress can equip wands and focuses only in weapon/offhand slots.");
    assert!(!result.spent_turn);
    assert!(c.equipped_weapon.name.contains("Wand"));
}
```

- [ ] **Step 2: Run focused tests and verify they fail**

Run:

```bash
cargo test sorceress_random_equipment_uses_wand_focus_pool_without_staves sorceress_can_equip_wands_and_focuses_but_not_other_weapons_or_shields
```

Expected: FAIL because Sorceress loot and equip restrictions do not exist.

- [ ] **Step 3: Add Sorceress equipment loot generation**

In `random_equipment_loot_for_class`, add:

```rust
CharacterClass::Sorceress => random_sorceress_equipment_loot(floor, better),
```

Add `random_sorceress_equipment_loot` next to the Warrior/Rogue equipment generators. Use `rng.gen_range(0..10)` and these exact pools:

- `Apprentice Wand`, `ItemKind::Weapon`, `weapon_stats(2 + bonus, 4 + bonus, 0, WAND_CRIT_CHANCE)`, requirements `(0, 1 + item_level / 2, 2 + item_level)`.
- `Ash Wand`, `ItemKind::Weapon`, `weapon_stats(3 + bonus, 5 + bonus, -1, WAND_CRIT_CHANCE)`, requirements `(0, 1 + item_level / 2, 3 + item_level)`.
- `Threadbare Robe`, `ItemKind::Armor`, `item_stats(0, 0, bonus.min(2), 1 + bonus.min(2), 0)`, requirements `(0, 0, 2 + item_level)`.
- `Spell Focus`, `ItemKind::Shield`, `item_stats(0, 0, bonus.min(1), 1 + bonus, 0)`, requirements `(0, 0, 2 + item_level)`.
- `Moon Circlet`, `ItemKind::Helm`, `item_stats(0, 0, bonus.min(1), 1 + bonus.min(2), 0)`, requirements `(0, 0, 2 + item_level)`.
- `Spell Gloves`, `ItemKind::Gloves`, `item_stats(0, 0, 0, 1 + bonus, 0)`, requirements `(0, 0, 2 + item_level)`.
- `Soft Slippers`, `ItemKind::Boots`, `item_stats(0, 0, 0, 1 + bonus, 1)`, requirements `(0, 1 + item_level / 2, 1 + item_level)`.
- `Silk Sash`, `ItemKind::Belt`, `item_stats(0, 0, bonus.min(1), 1, 0)`, requirements `(0, 0, 2 + item_level)`.
- `Arcane Amulet`, `ItemKind::Amulet`, `item_stats(0, 0, 0, 1 + bonus.min(2), 0)`, requirements `(0, 0, 1 + item_level)`.
- `Rune Ring`, `ItemKind::Ring`, `item_stats(0, 0, 0, 1 + bonus.min(2), 0)`, requirements `(0, 0, 1 + item_level)`.

Wrap names with `loot_name(&rarity, base)` and sockets with `add_random_sockets(item, rng.gen_range(0.0..1.0))`, matching existing class pools.

- [ ] **Step 4: Add Sorceress weapon/offhand equip restrictions**

In `src/inventory.rs`, replace `can_equip_item_for_class` with class-specific helpers:

```rust
fn can_equip_item_for_class(c: &Character, item: &Item) -> bool {
    match c.class {
        CharacterClass::Warrior => true,
        CharacterClass::Rogue => {
            item.kind != ItemKind::Shield
                || item.name == "Empty Offhand"
                || item.name.contains("Buckler")
        }
        CharacterClass::Sorceress => match item.kind {
            ItemKind::Weapon => item.name.contains("Wand"),
            ItemKind::Shield => item.name == "Empty Offhand" || item.name.contains("Focus"),
            _ => true,
        },
    }
}
```

Update `unmet_requirements_message` class failure branch:

```rust
if !can_equip_item_for_class(c, item) {
    return Some(match c.class {
        CharacterClass::Rogue => "Rogue cannot equip non-buckler shields.".to_string(),
        CharacterClass::Sorceress => "Sorceress can equip wands and focuses only in weapon/offhand slots.".to_string(),
        CharacterClass::Warrior => "Cannot equip that item.".to_string(),
    });
}
```

- [ ] **Step 5: Update README and design docs**

In `README.md`:

- Change `Warrior and Rogue classes` to `Warrior, Rogue, and Sorceress classes`.
- In Dungeon controls, add `Sorceress: 1 Firebolt, 2 Frost Ring, 3 Chain Spark, 4 Mana Shield`.

In `design.md`:

- Rename the old `### 2. Embercaller` section to `### 2. Sorceress`.
- Replace the Embercaller concept text with the approved Sorceress design: STR 1 / DEX 3 / INT 6, Mana, wand + focus MVP, Firebolt, Frost Ring, Chain Spark, unlockable Mana Shield, Kindle, Static Charge.
- State that spell damage scales with Intelligence while spell accuracy uses normal hit rating from Dexterity and gear.
- Update skill-tree examples from `Example Embercaller trees` to `Example Sorceress trees` with Flame, Frost, and Storm.
- Update current class-specific equipment direction to include Sorceress receiving wands, focuses, robes, circlets, spell gloves, soft slippers, sashes, arcane amulets, and rune rings; staves remain outside the MVP.
- Update implementation status near the UI/status section to say Warrior, Rogue, and Sorceress are playable.

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test sorceress_random_equipment_uses_wand_focus_pool_without_staves sorceress_can_equip_wands_and_focuses_but_not_other_weapons_or_shields
```

Expected: PASS.

- [ ] **Step 7: Run all Sorceress-focused tests**

Run:

```bash
cargo test sorceress firebolt frost_ring chain_spark mana_shield static_charge shocked
```

Expected: PASS.

- [ ] **Step 8: Run the required pre-commit workflow**

Run:

```bash
scripts/agent-commit-guard.sh --fix
```

Expected: PASS. This runs `cargo fmt`, `cargo test`, and `cargo check`.

- [ ] **Step 9: Review status and diff**

Run:

```bash
git status --short
git diff -- README.md design.md src/classes.rs src/dungeon.rs src/inventory.rs src/items.rs src/main.rs src/model.rs src/save.rs src/skills.rs src/sorceress.rs src/tests.rs
```

Expected: only Sorceress-related files are changed.

- [ ] **Step 10: Commit implementation**

If `git config --get core.hooksPath` is not `.githooks`, run:

```bash
git config --local core.hooksPath .githooks
```

Then commit:

```bash
git add README.md design.md src/classes.rs src/dungeon.rs src/inventory.rs src/items.rs src/main.rs src/model.rs src/save.rs src/skills.rs src/sorceress.rs src/tests.rs
git commit -m "Add playable Sorceress class"
```

Expected: commit succeeds without `--no-verify`.

## Self-Review Checklist

- [ ] Sorceress is selectable during character creation and saved as `CharacterClass::Sorceress`.
- [ ] Sorceress starts STR 1 / DEX 3 / INT 6 with wand, focus, robe, two health potions, and two mana potions.
- [ ] Firebolt costs 4 mana, has no cooldown, uses line of sight, uses hit rating, and can apply rank-scaling Burning.
- [ ] Frost Ring costs 8 mana, has cooldown 3, hits all eight adjacent tiles, uses hit rolls per enemy, and can Freeze enemies for one skipped turn.
- [ ] Chain Spark costs 7 mana, has cooldown 2, requires line of sight for the initial target, jumps within diagonal-inclusive radius 2 through reachable open tiles, and never hits the same enemy twice in one cast.
- [ ] Mana Shield unlocks from Frost Ring rank 2, toggles freely, absorbs rank-scaled incoming damage at 1 mana per damage prevented, and turns off at zero mana.
- [ ] Kindle increases fire damage against Burning enemies.
- [ ] Static Charge lets Chain Spark apply Shocked, Shocked is consumed by the next damaging hit, and equal-or-stronger Shocked applications replace weaker ones.
- [ ] Sorceress MVP has no rank-5 masteries.
- [ ] Sorceress random equipment uses wand + focus class loot and does not include staves.
- [ ] README and `design.md` describe the shipped Sorceress behavior.
