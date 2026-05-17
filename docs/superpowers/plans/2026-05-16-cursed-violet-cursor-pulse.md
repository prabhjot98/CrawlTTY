# Cursed Violet Cursor Pulse Implementation Plan

> **For pi agents:** REQUIRED SKILL: Use `executing-plans` to implement this plan task-by-task in the current pi session. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reuse the existing cursed violet selection color for all UI cursors and add a subtle pulse animation.
**Architecture:** Centralize cursor color/style and animation timing in `src/ui.rs` and `src/input.rs`, then pass a cursor animation frame through ratatui render functions that draw selected menu/list/grid cursors. Existing render wrappers keep test-friendly defaults while runtime loops use timed input ticks to toggle the pulse.
**Tech Stack:** Rust 2024, crossterm event polling, ratatui styling/widgets, existing cargo test suite.

---

### Task 1: Centralize cursed violet cursor style

**Files:**
- Modify: `src/ui.rs`
- Modify: `src/inventory.rs`
- Modify: `src/town.rs`
- Modify: `src/skills.rs`
- Modify: `src/dungeon.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Add assertions that selected inventory cells and selected menu/list cursors use `SELECTED_CONTAINER_BORDER_COLOR` instead of green. Update existing color expectations for selected cursor labels to violet.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test cursor_style_uses_cursed_violet -- --exact`
Expected: FAIL because selected cursor style is still green or the new test does not yet compile until helper functions exist.

- [ ] **Step 3: Implement minimal code**

Add `cursor_style(cursor_frame: bool)`, `selected_cursor_style()`, and `CURSOR_PULSE_INTERVAL` to `src/ui.rs`. Use the helper anywhere the selected cursor/focus currently hardcodes green: inventory grid labels, town menu selected lines, skill tree selected lines, attributes selected marker/text, gem and ground-loot list highlight styles.

- [ ] **Step 4: Run verification**

Run: `cargo test cursor_style_uses_cursed_violet inventory_cell_spans_use_rarity_outline_and_focus_label attributes_screen_uses_cursor_selection_and_attribute_colors character_creation_active_step_uses_muted_cursed_violet_border active_stash_grid_uses_muted_cursed_violet_border -- --exact`
Expected: PASS

### Task 2: Add timed cursor pulse redraws

**Files:**
- Modify: `src/input.rs`
- Modify: `src/inventory.rs`
- Modify: `src/town.rs`
- Modify: `src/skills.rs`
- Modify: `src/dungeon.rs`
- Modify: `src/save.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Add tests for timed input polling returning a tick and for `cursor_style(false)` vs `cursor_style(true)` differing only by bold modifier while sharing cursed violet foreground.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test cursor_pulse -- --exact`
Expected: FAIL because timed input and cursor pulse helpers are not wired yet.

- [ ] **Step 3: Implement minimal code**

Add `UiInput::Tick` and `read_ui_input_nav_timed(Duration)` / `read_ui_input_raw_arrows_timed(Duration)` wrappers using `event::poll(timeout)`. In interactive ratatui loops, track `cursor_frame: bool`, pass it into render functions, and toggle it on `UiInput::Tick`. Keep resize redraw behavior unchanged and keep existing one-shot render functions defaulting to the active cursor frame for tests.

- [ ] **Step 4: Run verification**

Run: `cargo test cursor_pulse terminal_resize_event_requests_redraw terminal_key_repeat_events_are_ignored terminal_key_release_events_are_ignored -- --exact`
Expected: PASS

### Task 3: Document and full verification

**Files:**
- Modify: `design.md`

- [ ] **Step 1: Update design status**

Update the UI implementation status to state that selected cursors reuse cursed violet and pulse between normal and bold violet while menus redraw on timer ticks.

- [ ] **Step 2: Run required pre-commit workflow**

Run: `scripts/agent-commit-guard.sh --fix`
Expected: PASS

- [ ] **Step 3: Review and commit**

Run: `git status --short` and `git diff`, stage only changed files, and commit with message `Animate cursed violet cursors`.
