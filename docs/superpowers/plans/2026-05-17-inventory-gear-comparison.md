# Inventory Gear Comparison Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand the inventory details panel into a readable gear comparison surface for selected weapons, armor, and shields.

**Architecture:** Keep the feature inside `src/inventory.rs`, where the inventory details panel and existing comparison helper already live. Replace the single compact comparison string with multiple detail lines: equipped item, stat deltas, and equip lock messaging.

**Tech Stack:** Rust 2024, ratatui `Line` rendering, existing unit tests in `src/tests.rs`, required `scripts/agent-commit-guard.sh --fix`.

---

### Task 1: Add Comparison Render Tests

**Files:**
- Modify: `src/tests.rs`

- [ ] Add tests for selected weapon, armor, shield, and locked gear detail lines.
- [ ] Run focused tests and confirm they fail against the current one-line comparison output.

### Task 2: Implement Comparison Detail Lines

**Files:**
- Modify: `src/inventory.rs`

- [ ] Add helpers that return the equipped item for a selected gear kind.
- [ ] Replace the compact comparison string in `selected_item_detail_lines` with multiple `Line` values.
- [ ] Keep non-gear items unchanged.
- [ ] Run focused tests and confirm they pass.

### Task 3: Update Design Status

**Files:**
- Modify: `design.md`

- [ ] Document that inventory gear details show selected item vs equipped gear with deltas and lock messaging.

### Task 4: Verify And Commit

**Files:**
- Modify: `src/inventory.rs`
- Modify: `src/tests.rs`
- Modify: `design.md`
- Create: `docs/superpowers/plans/2026-05-17-inventory-gear-comparison.md`

- [ ] Run `scripts/agent-commit-guard.sh --fix`.
- [ ] Review status and diff.
- [ ] Commit the task branch.
- [ ] Merge the verified branch back into `main`.
