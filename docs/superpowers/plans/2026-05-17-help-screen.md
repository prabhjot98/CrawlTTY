# Help Screen Implementation Plan

> **For pi agents:** REQUIRED SKILL: Use `executing-plans` to implement this plan task-by-task in the current pi session. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an in-game searchable glossary help screen opened by `H/h` from town and dungeon without consuming dungeon turns.
**Architecture:** Introduce a new `src/help.rs` module containing static glossary topics, searchable screen state, input handling, and ratatui rendering. Wire `H/h` into town and dungeon loops and advertise it in existing command footers; update player docs and tests.
**Tech Stack:** Rust, ratatui, crossterm input abstraction, existing cargo test/check workflow.

---

### Task 1: Add glossary data, search state, and rendering

**Files:**
- Create: `src/help.rs`
- Modify: `src/main.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing tests**

Add tests to `src/tests.rs` that call these exact APIs before implementation exists:

```rust
#[test]
fn help_topics_cover_requested_and_major_game_keywords() {
    let keywords: Vec<_> = help_topics().iter().map(|topic| topic.keyword).collect();
    for required in [
        "Strength", "Dexterity", "Intelligence", "Burning", "Bleeding", "Gold",
        "Health", "Mana", "Energy", "Combo Points", "Armor", "Dodge Rating",
        "Hit Rating", "Speed", "Critical Chance", "Poisoned", "Frozen", "Shocked",
        "Stunned", "Warrior", "Rogue", "Sorceress", "Cleave", "Backstab",
        "Firebolt", "Quest", "Stash", "Town Projects", "Sockets", "Gems",
        "Bellkeeper", "Glass Tyrant", "Hardcore", "Softcore", "Hollow Crypts",
        "Glass Wastes",
    ] {
        assert!(keywords.contains(&required), "missing help topic {required}");
    }
    assert!(keywords.len() >= 90, "glossary should include broad game vocabulary");
}

#[test]
fn help_search_filters_keywords_case_insensitively_and_keeps_selection_valid() {
    let mut state = HelpScreenState::new();
    state.handle_key('d');
    state.handle_key('E');
    state.handle_key('x');

    let filtered: Vec<_> = state.filtered_topics().iter().map(|topic| topic.keyword).collect();
    assert!(filtered.contains(&"Dexterity"));
    assert!(filtered.iter().all(|keyword| keyword.to_ascii_lowercase().contains("dex")));
    assert_eq!(state.selected_topic().unwrap().keyword, "Dexterity");

    state.handle_key('\u{8}');
    assert_eq!(state.query(), "dE");
}

#[test]
fn help_screen_renders_search_keyword_list_details_and_footer() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut state = HelpScreenState::new();
    state.handle_key('g');
    state.handle_key('o');
    state.handle_key('l');
    state.handle_key('d');
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();

    terminal.draw(|frame| render_help_screen(frame, &state)).unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("Search: gold"), "{}", backend_lines(&terminal).join("\n"));
    assert!(rendered.contains("Gold"));
    assert!(rendered.contains("currency"));
    assert!(rendered.contains("Up/Down=select"));
    assert!(rendered.contains("Esc=back"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test help_`
Expected: FAIL because `help_topics`, `HelpScreenState`, and `render_help_screen` are not implemented/exported.

- [ ] **Step 3: Implement minimal code**

Create `src/help.rs` with `HelpTopic`, `help_topics`, `HelpScreenState`, `help_screen`, and `render_help_screen`. Add `mod help;` and `pub(crate) use help::*;` to `src/main.rs`. The topic list must be static, alphabetized enough for browsing, and include at least all test-required words and the visible game systems/classes/resources/statuses/controls/enemies/items/town services.

- [ ] **Step 4: Run verification**

Run: `cargo test help_`
Expected: PASS.

### Task 2: Wire `H/h` into town and dungeon UI

**Files:**
- Modify: `src/main.rs`
- Modify: `src/dungeon.rs`
- Modify: `src/ui.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write the failing tests**

Add tests to `src/tests.rs`:

```rust
#[test]
fn town_footer_advertises_help_hotkey() {
    use ratatui::{Terminal, backend::TestBackend};

    let c = test_character();
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();
    terminal.draw(|frame| render_town(frame, &c, "")).unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("h=help"), "{}", backend_lines(&terminal).join("\n"));
}

#[test]
fn dungeon_footer_and_known_commands_include_help_hotkey() {
    use ratatui::{Terminal, backend::TestBackend};

    let mut c = test_character();
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();
    terminal.draw(|frame| render_dungeon(frame, &c)).unwrap();
    let rendered = backend_text(&terminal);

    assert!(rendered.contains("h=help"), "{}", backend_lines(&terminal).join("\n"));
    assert!(is_known_dungeon_command_for(&c, 'h'));
    assert!(is_known_dungeon_command_for(&c, 'H'));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test help_hotkey`
Expected: FAIL because footers and known-command handling do not include help.

- [ ] **Step 3: Implement minimal code**

In `run_game`, handle `'h' | 'H'` by calling `help_screen(terminal)?`. In `dungeon_loop`, handle help before pending smoke-step direction resolution so it is global and spends no turn. Add `("h", "help")` to town/dungeon command entries and include `h/H` in `is_known_dungeon_command_for`.

- [ ] **Step 4: Run verification**

Run: `cargo test help_hotkey`
Expected: PASS.

### Task 3: Update player documentation and run final guard

**Files:**
- Modify: `README.md`
- Modify: `design.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Write the failing test**

Add or update a documentation-focused test in `src/tests.rs`:

```rust
#[test]
fn readme_lists_help_control() {
    let readme = include_str!("../README.md");
    assert!(readme.contains("`h` help"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test readme_lists_help_control`
Expected: FAIL because README does not list the help control yet.

- [ ] **Step 3: Implement minimal docs**

Add `h` help to README town and dungeon controls. Update `design.md` controls/UI sections to describe the searchable two-column help glossary. Add one concise Unreleased changelog line for the gameplay/UI change.

- [ ] **Step 4: Run verification**

Run: `cargo test readme_lists_help_control`
Expected: PASS.

- [ ] **Step 5: Final guard and commit**

Run: `scripts/agent-commit-guard.sh --fix`
Expected: PASS; it runs `cargo fmt`, `cargo test`, and `cargo check`. Then review `git status --short` and `git diff`, stage only changed files, ensure `core.hooksPath` is `.githooks`, commit, merge back to `main`, verify main contains the commit, and remove the task worktree when practical.
