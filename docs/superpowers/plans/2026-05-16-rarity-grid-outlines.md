# Rarity Grid Outlines Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render inventory-style item-grid cell outlines in the contained item's rarity color while preserving green bold focus on the selected item label.

**Architecture:** Keep the current fixed-width `[X]` text grid. Add a small span-building helper in `src/inventory.rs` so `render_item_grid` can style the brackets and label independently, and unit-test the helper directly.

**Tech Stack:** Rust, ratatui `Span`, `Style`, `Color`, `Modifier`, existing `ItemGrid` and `rarity_color`.

---

### Task 1: Add Failing Cell Styling Test

**Files:**
- Modify: `src/tests.rs`

- [ ] **Step 1: Write the failing test**

Add this test near `inventory_cell_label_shows_item_kind_or_empty_cell`:

```rust
#[test]
fn inventory_cell_spans_use_rarity_outline_and_focus_label() {
    use ratatui::style::{Color, Modifier, Style};

    let mut rare_sword = rusted_sword();
    rare_sword.rarity = Rarity::Rare;
    let mut magic_axe = crude_axe();
    magic_axe.rarity = Rarity::Magic;
    let grid = ItemGrid::new(2, 2, vec![rare_sword, magic_axe]);

    let rare_selected = inventory_cell_spans(&grid, 0, true);
    assert_eq!(rare_selected[0].content.as_ref(), "[");
    assert_eq!(rare_selected[0].style, Style::default().fg(Color::Yellow));
    assert_eq!(rare_selected[1].content.as_ref(), "W");
    assert_eq!(
        rare_selected[1].style,
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    );
    assert_eq!(rare_selected[2].content.as_ref(), "]");
    assert_eq!(rare_selected[2].style, Style::default().fg(Color::Yellow));

    let magic_unselected = inventory_cell_spans(&grid, 1, false);
    assert_eq!(magic_unselected[0].style, Style::default().fg(Color::Blue));
    assert_eq!(magic_unselected[1].style, Style::default().fg(Color::White));
    assert_eq!(magic_unselected[2].style, Style::default().fg(Color::Blue));

    let empty_selected = inventory_cell_spans(&grid, 2, true);
    assert_eq!(
        empty_selected.iter().map(|span| span.content.as_ref()).collect::<Vec<_>>(),
        vec!["[", ".", "]"]
    );
    assert!(empty_selected.iter().all(|span| {
        span.style == Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    }));
}
```

- [ ] **Step 2: Run the focused test to verify it fails**

Run: `cargo test inventory_cell_spans_use_rarity_outline_and_focus_label`

Expected: FAIL because `inventory_cell_spans` does not exist.

### Task 2: Implement Shared Cell Span Helper

**Files:**
- Modify: `src/inventory.rs`

- [ ] **Step 1: Add `inventory_cell_spans`**

Add this helper after `item_grid_render_width`:

```rust
#[allow(dead_code)]
pub(crate) fn inventory_cell_spans(
    grid: &ItemGrid,
    index: usize,
    selected: bool,
) -> Vec<Span<'static>> {
    let label = inventory_cell_label(grid, index);
    let focus_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);

    let Some(item) = grid.get(index) else {
        let style = if selected {
            focus_style
        } else {
            Style::default().fg(Color::White)
        };
        return vec![
            Span::styled("[", style),
            Span::styled(label, style),
            Span::styled("]", style),
        ];
    };

    let outline_style = Style::default().fg(rarity_color(&item.rarity));
    let label_style = if selected {
        focus_style
    } else {
        Style::default().fg(Color::White)
    };
    vec![
        Span::styled("[", outline_style),
        Span::styled(label, label_style),
        Span::styled("]", outline_style),
    ]
}
```

- [ ] **Step 2: Use the helper in `render_item_grid`**

Replace the per-cell style block in `render_item_grid` with:

```rust
            spans.extend(inventory_cell_spans(grid, index, index == selected));
            spans.push(Span::raw(" "));
```

- [ ] **Step 3: Run the focused test to verify it passes**

Run: `cargo test inventory_cell_spans_use_rarity_outline_and_focus_label`

Expected: PASS.

- [ ] **Step 4: Run adjacent render tests**

Run: `cargo test inventory_render stash_render`

Expected: PASS.

### Task 3: Update Project Design Status

**Files:**
- Modify: `design.md`

- [ ] **Step 1: Update inventory implementation status**

In the inventory grid implementation status paragraph, add a sentence:

```markdown
Occupied bag and stash grid cells color their bracket outline by item rarity while keeping the focused item label bold green.
```

- [ ] **Step 2: Run the commit guard**

Run: `scripts/agent-commit-guard.sh --fix`

Expected: `cargo fmt`, `cargo test`, and `cargo check` all succeed.

### Task 4: Commit and Merge

**Files:**
- Stage: `src/inventory.rs`
- Stage: `src/tests.rs`
- Stage: `design.md`
- Stage: `docs/superpowers/plans/2026-05-16-rarity-grid-outlines.md`

- [ ] **Step 1: Review changes**

Run: `git status --short` and `git diff --stat`.

Expected: only the four files above changed.

- [ ] **Step 2: Commit implementation**

Run:

```bash
git add src/inventory.rs src/tests.rs design.md docs/superpowers/plans/2026-05-16-rarity-grid-outlines.md
git commit -m "Color item grid outlines by rarity"
```

Expected: commit succeeds after pre-commit hook reruns validation.

- [ ] **Step 3: Merge task branch into main**

Run from `/Users/pssandhu/d2`:

```bash
git merge --no-ff codex/rarity-grid-outlines
```

Expected: merge succeeds.

- [ ] **Step 4: Verify main contains the commit**

Run from `/Users/pssandhu/d2`:

```bash
git log --oneline -3
```

Expected: the merge commit and `Color item grid outlines by rarity` are visible.

- [ ] **Step 5: Remove task worktree**

Run from `/Users/pssandhu/d2`:

```bash
git worktree remove .worktrees/rarity-grid-outlines
```

Expected: worktree removal succeeds.
