# Gothic UI Palette Implementation Plan

> **For pi agents:** REQUIRED SKILL: Use `executing-plans` to implement this plan task-by-task in the current pi session. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give CrawlTTY a consistent gothic-and-cursed visual identity with a subtle Diablo-like warmth across containers, text, stats, rarity, and command UI.
**Architecture:** Add a centralized ratatui style palette in `src/ui.rs`, then route screen rendering through semantic helpers instead of scattered raw terminal colors. Keep gameplay symbols ASCII-only and preserve existing layout/interaction behavior.
**Tech Stack:** Rust, ratatui, existing unit tests in `src/tests.rs`, `scripts/agent-commit-guard.sh --fix` for final verification.

---

### Task 1: Centralize the gothic/cursed palette

**Files:**

- Modify: `src/ui.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Add this test near the existing cursor/container style tests in `src/tests.rs`:

```rust
#[test]
fn ui_palette_exposes_gothic_cursed_semantic_styles() {
    use ratatui::style::{Color, Modifier, Style};

    assert_eq!(TEXT_PRIMARY_COLOR, Color::Rgb(214, 203, 177));
    assert_eq!(TEXT_MUTED_COLOR, Color::Rgb(108, 101, 112));
    assert_eq!(CONTAINER_BORDER_COLOR, Color::Rgb(75, 67, 84));
    assert_eq!(SELECTED_CONTAINER_BORDER_COLOR, Color::Rgb(148, 80, 190));
    assert_eq!(TITLE_COLOR, Color::Rgb(201, 163, 86));
    assert_eq!(DANGER_COLOR, Color::Rgb(188, 54, 54));
    assert_eq!(ACTION_COLOR, Color::Rgb(93, 153, 112));
    assert_eq!(WARNING_COLOR, Color::Rgb(214, 157, 73));
    assert_eq!(ARCANE_COLOR, Color::Rgb(113, 151, 201));
    assert_eq!(CURSED_COLOR, Color::Rgb(177, 93, 204));

    assert_eq!(body_style(), Style::default().fg(TEXT_PRIMARY_COLOR));
    assert_eq!(muted_style(), Style::default().fg(TEXT_MUTED_COLOR));
    assert_eq!(title_style(), Style::default().fg(TITLE_COLOR).add_modifier(Modifier::BOLD));
    assert_eq!(container_border_style(false), Style::default().fg(CONTAINER_BORDER_COLOR));
    assert_eq!(container_border_style(true), Style::default().fg(SELECTED_CONTAINER_BORDER_COLOR));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test ui_palette_exposes_gothic_cursed_semantic_styles`
Expected: FAIL because the semantic constants and helpers do not exist yet.

- [ ] **Step 3: Implement minimal code**

In `src/ui.rs`, add semantic `pub(crate)` color constants and helpers after the existing imports. Keep `SELECTED_CONTAINER_BORDER_COLOR` at the same value so current cursor behavior remains recognizable.

Required constants and helpers:

```rust
pub(crate) const TEXT_PRIMARY_COLOR: Color = Color::Rgb(214, 203, 177);
pub(crate) const TEXT_MUTED_COLOR: Color = Color::Rgb(108, 101, 112);
pub(crate) const CONTAINER_BORDER_COLOR: Color = Color::Rgb(75, 67, 84);
pub(crate) const SELECTED_CONTAINER_BORDER_COLOR: Color = Color::Rgb(148, 80, 190);
pub(crate) const TITLE_COLOR: Color = Color::Rgb(201, 163, 86);
pub(crate) const DANGER_COLOR: Color = Color::Rgb(188, 54, 54);
pub(crate) const ACTION_COLOR: Color = Color::Rgb(93, 153, 112);
pub(crate) const WARNING_COLOR: Color = Color::Rgb(214, 157, 73);
pub(crate) const ARCANE_COLOR: Color = Color::Rgb(113, 151, 201);
pub(crate) const CURSED_COLOR: Color = Color::Rgb(177, 93, 204);

pub(crate) fn body_style() -> Style { Style::default().fg(TEXT_PRIMARY_COLOR) }
pub(crate) fn muted_style() -> Style { Style::default().fg(TEXT_MUTED_COLOR) }
pub(crate) fn title_style() -> Style { Style::default().fg(TITLE_COLOR).add_modifier(Modifier::BOLD) }
pub(crate) fn container_border_style(selected: bool) -> Style {
    if selected { Style::default().fg(SELECTED_CONTAINER_BORDER_COLOR) } else { Style::default().fg(CONTAINER_BORDER_COLOR) }
}
```

Make `selected_container_border_style(selected)` delegate to `container_border_style(selected)`.

- [ ] **Step 4: Run verification**

Run: `cargo test ui_palette_exposes_gothic_cursed_semantic_styles cursor_pulse_styles_share_cursed_violet_and_toggle_bold`
Expected: PASS.

---

### Task 2: Apply the palette to common text, rarity, commands, and inventory cells

**Files:**

- Modify: `src/ui.rs`
- Modify: `src/inventory.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Update `inventory_cell_spans_use_rarity_outline_and_focus_label` to expect semantic rarity/body colors:

```rust
assert_eq!(rare_selected[0].style, Style::default().fg(RARITY_RARE_COLOR));
assert_eq!(rare_selected[2].style, Style::default().fg(RARITY_RARE_COLOR));
assert_eq!(magic_unselected[0].style, Style::default().fg(RARITY_MAGIC_COLOR));
assert_eq!(magic_unselected[1].style, body_style());
assert_eq!(magic_unselected[2].style, Style::default().fg(RARITY_MAGIC_COLOR));
```

Add this test near `inventory_cell_spans_use_rarity_outline_and_focus_label`:

```rust
#[test]
fn command_and_stat_text_use_gothic_semantic_colors() {
    let commands = command_line("Town", &[("m", "merchant"), ("q", "save+quit")]);
    assert_eq!(commands.spans[0].style, title_style());
    assert_eq!(commands.spans[1].style, Style::default().fg(ACTION_COLOR).add_modifier(Modifier::BOLD));
    assert_eq!(commands.spans[4].style, Style::default().fg(DANGER_COLOR).add_modifier(Modifier::BOLD));

    let stat = stat_span("Gold 25", WARNING_COLOR);
    assert_eq!(stat.style, Style::default().fg(WARNING_COLOR).add_modifier(Modifier::BOLD));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test inventory_cell_spans_use_rarity_outline_and_focus_label command_and_stat_text_use_gothic_semantic_colors`
Expected: FAIL because rarity, command, and stat text still use raw primary colors or non-bold stat styling.

- [ ] **Step 3: Implement minimal code**

In `src/ui.rs`:

- Add rarity constants:
  - `RARITY_COMMON_COLOR = TEXT_PRIMARY_COLOR`
  - `RARITY_MAGIC_COLOR = Color::Rgb(113, 151, 201)`
  - `RARITY_RARE_COLOR = Color::Rgb(214, 157, 73)`
- Change `rarity_color` to return the rarity constants.
- Change `stat_span` to return `Style::default().fg(color).add_modifier(Modifier::BOLD)`.
- Change `command_line` title style to `title_style()`.
- Change command key styles to `ACTION_COLOR` except `q`, which uses `DANGER_COLOR`.

In `src/inventory.rs`:

- Use `body_style()` instead of `Style::default().fg(Color::White)` for unselected item labels and empty cells.
- Keep selected labels using `selected_cursor_style()`.

- [ ] **Step 4: Run verification**

Run: `cargo test inventory_cell_spans_use_rarity_outline_and_focus_label command_and_stat_text_use_gothic_semantic_colors`
Expected: PASS.

---

### Task 3: Apply gothic container borders and title styling to major screens

**Files:**

- Modify: `src/ui.rs`
- Modify: `src/inventory.rs`
- Modify: `src/dungeon.rs`
- Modify: `src/town.rs`
- Modify: `src/skills.rs`
- Modify: `src/save.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Add these tests near the existing ratatui rendering style tests in `src/tests.rs`:

```rust
#[test]
fn town_and_inventory_containers_use_gothic_borders_and_titles() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut town_terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();
    town_terminal.draw(|frame| render_town(frame, &c, "")).unwrap();
    assert_eq!(cell_fg_at(&town_terminal, 0, 0), CONTAINER_BORDER_COLOR);
    assert_eq!(cell_fg_at_text(&town_terminal, "Town"), TITLE_COLOR);
    assert_eq!(cell_fg_at_text(&town_terminal, "Status"), TITLE_COLOR);
    assert_eq!(cell_fg_at_text(&town_terminal, "Commands"), TITLE_COLOR);

    let mut inventory_terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();
    inventory_terminal
        .draw(|frame| render_inventory_screen(frame, &c, 0, ""))
        .unwrap();
    assert_eq!(cell_fg_at(&inventory_terminal, 0, 0), CONTAINER_BORDER_COLOR);
    assert_eq!(cell_fg_at_text(&inventory_terminal, "Inventory"), TITLE_COLOR);
    assert_eq!(cell_fg_at_text(&inventory_terminal, "Bag"), TITLE_COLOR);
    assert_eq!(cell_fg_at_text(&inventory_terminal, "Details"), TITLE_COLOR);
}

#[test]
fn dungeon_containers_use_gothic_borders_and_titles() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.active_dungeon = Some(Dungeon::new(1));
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();
    terminal.draw(|frame| render_dungeon(frame, &c)).unwrap();

    assert_eq!(cell_fg_at(&terminal, 0, 0), CONTAINER_BORDER_COLOR);
    assert_eq!(cell_fg_at_text(&terminal, "Dungeon"), TITLE_COLOR);
    assert_eq!(cell_fg_at_text(&terminal, "Map"), TITLE_COLOR);
    assert_eq!(cell_fg_at_text(&terminal, "Combat Log"), TITLE_COLOR);
    assert_eq!(cell_fg_at_text(&terminal, "Skills"), TITLE_COLOR);
    assert_eq!(cell_fg_at_text(&terminal, "Commands"), TITLE_COLOR);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test town_and_inventory_containers_use_gothic_borders_and_titles dungeon_containers_use_gothic_borders_and_titles`
Expected: FAIL because most blocks still use default border/title styles.

- [ ] **Step 3: Implement minimal code**

In `src/ui.rs`:

- Add helper `pub(crate) fn gothic_block(title: impl Into<Line<'static>>) -> Block<'static>` that returns `Block::default().borders(Borders::ALL).title(title.into().style(title_style())).border_style(container_border_style(false))`.
- Add helper `pub(crate) fn gothic_block_selected(title: impl Into<Line<'static>>, selected: bool) -> Block<'static>` with the same title style and `container_border_style(selected)`.

Replace major-screen `Block::default().borders(Borders::ALL).title(...)` calls with `gothic_block(...)`, and replace active/focus container block construction with `gothic_block_selected(..., selected)` where a selected container border is already expected.

Scope the replacements to ratatui screen containers in `src/ui.rs`, `src/inventory.rs`, `src/dungeon.rs`, `src/town.rs`, `src/skills.rs`, and `src/save.rs`. Do not alter layout constraints, input handling, gameplay text, or ASCII map symbols.

- [ ] **Step 4: Run verification**

Run: `cargo test town_and_inventory_containers_use_gothic_borders_and_titles dungeon_containers_use_gothic_borders_and_titles character_creation_active_step_uses_muted_cursed_violet_border active_stash_grid_uses_muted_cursed_violet_border`
Expected: PASS.

---

### Task 4: Update design documentation and run final guarded verification

**Files:**

- Modify: `DESIGN.md`
- Modify: `design.md`

- [ ] **Step 1: Document the new visual language**

Update the UI/color sections in both `DESIGN.md` and `design.md` with:

- The chosen gothic/cursed visual identity.
- The semantic palette categories.
- The rule that containers use muted gothic borders by default and cursed violet only for focus/selection.
- The rule that color remains semantic and gameplay symbols remain ASCII-only.

- [ ] **Step 2: Run required pre-commit workflow**

Run: `scripts/agent-commit-guard.sh --fix`
Expected: PASS, including `cargo fmt`, `cargo test`, and `cargo check`.

- [ ] **Step 3: Review changes**

Run: `git status --short && git diff -- src/ui.rs src/inventory.rs src/dungeon.rs src/town.rs src/skills.rs src/save.rs src/tests.rs DESIGN.md design.md docs/superpowers/plans/2026-05-16-gothic-ui-palette.md`
Expected: Only intended files are changed.

- [ ] **Step 4: Commit**

Run:

```bash
git config --local core.hooksPath .githooks
git add src/ui.rs src/inventory.rs src/dungeon.rs src/town.rs src/skills.rs src/save.rs src/tests.rs DESIGN.md design.md docs/superpowers/plans/2026-05-16-gothic-ui-palette.md
git commit -m "Unify gothic UI palette"
```

Expected: Commit succeeds without `--no-verify`.

- [ ] **Step 5: Merge back to main and clean up worktree**

Run from `/Users/pssandhu/d2`:

```bash
git switch main
git merge --no-ff codex/gothic-ui-palette
git log --oneline -1
git worktree remove .worktrees/gothic-ui-palette
```

Expected: `main` contains the task commit and the task worktree is removed.
