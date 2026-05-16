# Rogue Class Design

## Overview

Add Rogue as the second playable class alongside Warrior. Rogue is a fast Dexterity melee class built around short burst windows, internal combo points, Energy, dagger attacks, poison setup, and smoke-assisted repositioning.

The Rogue should not be a passive poison-kiting class. Its primary loop is to enter danger, build combo points, create or exploit a burst window, spend points on a finisher, then reposition before enemies surround it.

## Goals

- Add a second class with a combat rhythm that is clearly different from Warrior.
- Use internal combo points owned by the Rogue, not attached to enemies.
- Use Energy instead of mana for Rogue skills.
- Give Rogue four active dungeon hotkeys in the first playable version.
- Keep poison as a setup and payoff mechanic rather than the whole damage plan.
- Use this feature as the point where class-specific state stops growing directly on `Character`.
- Accept a breaking save model change if the game version is bumped by a major version.

## Non-Goals

- No stealth/awareness simulation in the first Rogue pass.
- No facing or behind-the-target rules.
- No trap-focused Rogue implementation in the first pass.
- No ranged Rogue build in the first pass.
- No broad enemy pathing rewrite as part of the Rogue feature.

## Class Identity

Rogue is a dagger burst class:

- Lower HP and armor than Warrior.
- Higher Dexterity, dodge, speed, and crit potential.
- Uses Energy as a renewing class resource.
- Uses internal combo points as burst momentum.
- Uses poison to enable stronger burst and finisher decisions.
- Uses Smoke Step for controlled repositioning, not unlimited kiting.

Starting attributes:

- Strength: `2`
- Dexterity: `7`
- Intelligence: `1`

Starting gear:

- Basic dagger.
- Light armor.
- No shield for the first pass, unless an offhand model is added later.

## Core Loop

1. Build combo points with `Backstab` and `Venom Edge`.
2. Use `Smoke Step` to enter, escape, or create an empowered Backstab window.
3. Spend combo points with `Eviscerate`.
4. Decide whether to keep building for a larger finisher or cash out before enemy pressure becomes dangerous.

Combo point rules:

- Combo points are stored on the Rogue.
- Combo points are not stored on targets.
- Combo points persist across target swaps.
- Combo points cap at `5`.
- Combo points clear when combat state clears: dungeon exit, death, new run, or class reset.
- Finishers require at least `1` combo point and spend all current combo points.

Energy rules:

- Rogue max Energy is `100`.
- Rogue skills cost Energy.
- Energy regenerates after player turns, including movement and attacks.
- Rogue does not rely on mana potions for its main combat loop.

## First Active Skills

The first playable Rogue hotbar has four active skills:

- `1 Backstab`
- `2 Venom Edge`
- `3 Eviscerate`
- `4 Smoke Step`

### Backstab

Builder. Costs `25 Energy`. Melee dagger attack for `90% weapon damage`, grants `+1 combo point`.

Empowered Backstab deals `120% weapon damage` and gains bonus crit chance. It is enabled when the Rogue moved recently, used Smoke Step recently, or attacks a poisoned target.

### Venom Edge

Builder/setup. Costs `30 Energy`. Melee attack for `70% weapon damage`, grants `+1 combo point`, and poisons the target for `3 turns`.

Poison damage scales by rank. Poison should matter, but it should not make the class about running away while enemies slowly die.

### Eviscerate

Finisher. Costs `35 Energy`. Requires `1+ combo points`, spends all combo points, and deals physical burst scaling sharply by points.

Recommended first-pass scaling:

| Combo Points | Damage |
| ---: | ---: |
| 1 | `80% weapon damage` |
| 2 | `130% weapon damage` |
| 3 | `190% weapon damage` |
| 4 | `260% weapon damage` |
| 5 | `350% weapon damage` |

If the target is poisoned, Eviscerate should add a poison payoff, such as bonus damage, poison amplification, or poison consumption. The first implementation should choose one readable behavior and test it directly.

### Smoke Step

Mobility/defense. Costs `35 Energy`, cooldown `4 turns`.

Rules:

- Dash up to `2` tiles in a cardinal direction.
- Landing tile must be open floor.
- Cannot land in a wall or occupied enemy tile.
- Grants smoke protection for the next enemy-turn cycle.
- Enables empowered Backstab.
- Deals no damage directly.

Smoke Step should create a short tactical window. It should not be cheap enough to become default movement.

## Six-Skill Tree

Match Warrior's current six-skill tree size for the first Rogue pass.

### Daggers Branch

- `Backstab`: active builder.
- `Eviscerate`: active finisher.

### Venom Branch

- `Venom Edge`: active builder/setup.
- `Rupture`: poison payoff finisher or passive. Prefer passive in the first pass unless the hotbar expands further.

### Smoke Branch

- `Smoke Step`: active movement/defense.
- `Slip Away`: passive defense after finishers.

Rank philosophy:

- Ranks mostly increase damage, poison damage, dodge bonus, crit payoff, or Energy efficiency.
- Rank 5 unlocks one mastery choice, matching Warrior.
- Smoke Step cooldown reduction should be rare or avoided, because the cooldown keeps positioning decisions meaningful.

## Class Architecture

The current code has Warrior-specific skill ranks and cooldowns directly on `Character`. Rogue should introduce class-aware state instead of continuing to add class-specific fields directly to `Character`.

Recommended model:

```rust
enum Class {
    Warrior,
    Rogue,
}

struct WarriorState {
    // Existing Warrior ranks, cooldowns, buffs, and masteries.
}

struct RogueState {
    combo_points: u32,
    energy: u32,
    backstab_rank: u32,
    venom_edge_rank: u32,
    eviscerate_rank: u32,
    smoke_step_rank: u32,
    rupture_rank: u32,
    slip_away_rank: u32,
    smoke_step_cooldown: u32,
    smoke_protection_turns: u32,
    empowered_backstab_turns: u32,
}
```

Common `Character` fields remain shared: level, XP, gold, attributes, inventory, stash, equipment, quest flags, and active dungeon.

Add class-aware helpers so the dungeon loop does not become a large class switch:

- `max_resource()`
- `current_resource_label()`
- `class_skill_help_lines()`
- `clear_class_combat_state()`
- `tick_class_effects()`
- `handle_class_skill_key()`

The existing Warrior implementation can be migrated into `WarriorState` during the Rogue implementation or in a preparatory refactor. The important boundary is that new Rogue state should live in class-specific state from the start.

## UI And Character Creation

Character creation should allow:

- `1 Warrior`
- `2 Rogue`

The chosen class sets starting attributes, resource model, skill state, and starting gear.

Rogue UI work should use the existing ratatui screen patterns. Do not add new raw terminal or legacy `println!` screens for class selection, Rogue skill help, Rogue skill tree details, or Rogue-specific combat feedback. Non-interactive process messages may remain plain stdout, matching the rest of the project.

Dungeon UI should be class-aware:

- Warrior still shows mana and Warrior skills.
- Rogue shows Energy instead of mana.
- Rogue skill panel shows combo points as `CP 0/5`.
- Rogue skill help shows `1 Backstab`, `2 Venom Edge`, `3 Eviscerate`, and `4 Smoke Step`.
- Invalid class/key combinations log a warning and do not spend a turn.
- Routine skill use remains a single keypress.

## Save Versioning

The Rogue feature may break old saves.

Implementation should:

- Bump `Cargo.toml` from `0.1.0` to `1.0.0`.
- Rely on existing major-version save compatibility checks to reset incompatible saves.
- Keep old `"Ironbound"` class-name normalization to `"Warrior"` for otherwise version-compatible saves.
- Update user-facing docs to explain that the multi-class release resets old saves.

## Testing

Tests should cover:

- New characters can be created as Warrior or Rogue.
- Old `"Ironbound"` save class names still normalize to `Warrior` when version-compatible.
- Major save-version changes reset old saves.
- Rogue starts with expected attributes, gear, Energy, and zero combo points.
- Rogue builders grant combo points and respect the cap.
- Eviscerate requires combo points, spends them, and scales by points.
- Venom Edge applies poison and grants combo points.
- Smoke Step rejects blocked, occupied, and out-of-range destinations.
- Smoke Step moves to valid cardinal destinations and enables empowered Backstab.
- Rogue skill hotkeys are shown in dungeon help.
- Warrior skill behavior remains covered by existing tests.
