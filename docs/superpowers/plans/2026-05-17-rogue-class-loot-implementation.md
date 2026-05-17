# Rogue Class Loot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make random equipment drops class-specific so Warriors receive Warrior gear and Rogues receive dagger, scimitar, light armor, and buckler gear.

**Architecture:** Keep the existing loot pipeline and split equipment generation into class-routed pools. The current `random_equipment_loot` behavior remains the Warrior-compatible wrapper while new class-aware call sites use `random_equipment_loot_for_class`.

**Tech Stack:** Rust, existing `rand` loot rolls, current item and inventory models, project pre-commit guard.

---

### Task 1: Class-Specific Equipment Tests

**Files:**
- Modify: `src/tests.rs`

- [x] Add tests proving Rogue equipment rolls never produce Warrior families and Warrior equipment rolls never produce Rogue families.
- [x] Add a test proving Rogue can equip generated bucklers but still cannot equip Warrior shields.
- [x] Run the targeted tests and verify they fail before implementation.

### Task 2: Class-Specific Equipment Generation

**Files:**
- Modify: `src/dungeon.rs`
- Modify: `src/items.rs`

- [x] Add Rogue weapon crit constants for daggers and scimitars.
- [x] Split the current equipment table into a Warrior generator.
- [x] Add a Rogue generator for dagger, scimitar, light armor, and buckler.
- [x] Add `random_equipment_loot_for_class(class, floor, better)`.
- [x] Route `random_loot_for_class`, boss rewards, and guaranteed magic drops through the class-aware equipment generator.
- [x] Keep `random_equipment_loot(floor, better)` as a Warrior-compatible wrapper for existing tests and callers.

### Task 3: Rogue Buckler Equip Rule

**Files:**
- Modify: `src/inventory.rs`
- Modify: `src/tests.rs`

- [x] Allow Rogues to equip bucklers by item name while still rejecting non-buckler shields.
- [x] Update the old Rogue shield rejection test to cover Guard Shield rejection instead of all shields.
- [x] Run the targeted tests and verify they pass.

### Task 4: Design Documentation

**Files:**
- Modify: `DESIGN.md`

- [x] Update Rogue class text to say Rogues can progress into bucklers.
- [x] Update loot goals/status to describe class-specific equipment pools.

### Task 5: Verification and Commit

**Files:**
- Verify all changed files.

- [ ] Run `scripts/agent-commit-guard.sh --fix`.
- [ ] Review `git status --short` and `git diff`.
- [ ] Stage only changed files.
- [ ] Commit the implementation branch.
- [ ] Merge the verified branch back into `main`.
- [ ] Remove the temporary worktree when practical.
