# Multi-Character Saves Implementation Plan

> **For pi agents:** REQUIRED SKILL: Use `executing-plans` to implement this plan task-by-task in the current pi session. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Store each character in an independent save file, remember the last played character in a profile file, and let the town switch to an existing character or create a new one.
**Architecture:** `save.rs` owns profile data, per-character save paths, character summaries, loading, saving, and the character selection/creation screen. `main.rs` keeps the active `Character` value but adds a town command to open the character screen and replace the active value when a different character is chosen. `model.rs`, `dungeon.rs`, `README.md`, and `design.md` are updated so default saves and Hardcore deletion use the active character file instead of the legacy singleton path.
**Tech Stack:** Rust 2024, serde/serde_json, ratatui/crossterm, existing `scripts/agent-commit-guard.sh --fix` verification.

---

### Task 1: Profile and per-character save helpers

**Files:**
- Modify: `src/model.rs`
- Modify: `src/save.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests**

Add tests near the existing save tests in `src/tests.rs`:

```rust
#[test]
fn save_character_profile_tracks_last_character_and_per_character_file() {
    let dir = env::temp_dir().join(format!(
        "crawltty-multi-save-profile-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    let profile_path = dir.join("profile.json");
    let character_dir = dir.join("characters");

    let mara = Character::new("Mara".to_string(), CharacterClass::Warrior, DeathMode::Softcore);
    save_active_character_to_paths(&mara, &profile_path, &character_dir).unwrap();

    let profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&profile_path).unwrap()).unwrap();
    assert_eq!(profile["last_character_id"], "mara");
    assert!(character_dir.join("mara.json").exists());

    let loaded = load_last_character_from_paths(&profile_path, &character_dir).unwrap();
    assert_eq!(loaded.unwrap().name, "Mara");

    let shade = Character::new("Shade".to_string(), CharacterClass::Rogue, DeathMode::Hardcore);
    save_active_character_to_paths(&shade, &profile_path, &character_dir).unwrap();

    let profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&profile_path).unwrap()).unwrap();
    assert_eq!(profile["last_character_id"], "shade");
    assert!(character_dir.join("mara.json").exists());
    assert!(character_dir.join("shade.json").exists());
    assert_eq!(load_last_character_from_paths(&profile_path, &character_dir).unwrap().unwrap().name, "Shade");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn character_summaries_are_loaded_from_per_character_directory() {
    let dir = env::temp_dir().join(format!(
        "crawltty-multi-save-list-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    let profile_path = dir.join("profile.json");
    let character_dir = dir.join("characters");

    let mara = Character::new("Mara".to_string(), CharacterClass::Warrior, DeathMode::Softcore);
    let mut shade = Character::new("Shade".to_string(), CharacterClass::Rogue, DeathMode::Hardcore);
    shade.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    shade.active_dungeon.as_mut().unwrap().floor = 4;
    save_active_character_to_paths(&shade, &profile_path, &character_dir).unwrap();
    save_active_character_to_paths(&mara, &profile_path, &character_dir).unwrap();

    let summaries = load_character_summaries_from_dir(&character_dir).unwrap();
    assert_eq!(summaries.iter().map(|summary| summary.id.as_str()).collect::<Vec<_>>(), vec!["mara", "shade"]);
    assert_eq!(summaries[0].name, "Mara");
    assert_eq!(summaries[0].class_name, "Warrior");
    assert_eq!(summaries[0].death_mode, DeathMode::Softcore);
    assert_eq!(summaries[0].location, "Town");
    assert_eq!(summaries[1].name, "Shade");
    assert_eq!(summaries[1].class_name, "Rogue");
    assert_eq!(summaries[1].death_mode, DeathMode::Hardcore);
    assert_eq!(summaries[1].location, "Dungeon L4");

    let _ = fs::remove_dir_all(&dir);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test save_character_profile_tracks_last_character_and_per_character_file`
Expected: FAIL because `save_active_character_to_paths` and `load_last_character_from_paths` do not exist.

Run: `cargo test character_summaries_are_loaded_from_per_character_directory`
Expected: FAIL because `save_active_character_to_paths` and `load_character_summaries_from_dir` do not exist.

- [ ] **Step 3: Implement minimal code**

In `src/model.rs`, add constants while keeping the legacy path for compatibility:

```rust
pub(crate) const SAVE_PATH: &str = "saves/save.json";
pub(crate) const PROFILE_PATH: &str = "saves/profile.json";
pub(crate) const CHARACTER_SAVE_DIR: &str = "saves/characters";
```

In `src/save.rs`, add serde structs, path helpers, summary loading, profile loading, and default save routing:

```rust
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
struct SaveProfile {
    last_character_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CharacterSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) class_name: String,
    pub(crate) level: u32,
    pub(crate) death_mode: DeathMode,
    pub(crate) location: String,
}

pub(crate) fn character_id_from_name(name: &str) -> String {
    let mut id = String::new();
    let mut last_dash = false;
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            id.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !id.is_empty() {
            id.push('-');
            last_dash = true;
        }
    }
    while id.ends_with('-') {
        id.pop();
    }
    if id.is_empty() {
        "character".to_string()
    } else {
        id
    }
}

fn character_file_path(character_dir: &Path, id: &str) -> PathBuf {
    character_dir.join(format!("{id}.json"))
}

pub(crate) fn character_save_path(character: &Character) -> PathBuf {
    character_file_path(Path::new(CHARACTER_SAVE_DIR), &character_id_from_name(&character.name))
}

pub(crate) fn save_active_character_to_paths(
    character: &Character,
    profile_path: &Path,
    character_dir: &Path,
) -> Result<()> {
    let id = character_id_from_name(&character.name);
    save_character_to_path(character, &character_file_path(character_dir, &id))?;
    save_profile_to_path(profile_path, &id)
}

fn save_profile_to_path(profile_path: &Path, last_character_id: &str) -> Result<()> {
    if let Some(parent) = profile_path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent).context("failed to create save profile directory")?;
    }
    let profile = SaveProfile {
        last_character_id: last_character_id.to_string(),
    };
    let data = serde_json::to_string_pretty(&profile).context("failed to serialize save profile")?;
    let tmp_path = profile_path.with_file_name(format!(
        "{}.tmp",
        profile_path.file_name().and_then(|name| name.to_str()).unwrap_or("profile.json")
    ));
    {
        let mut file = fs::File::create(&tmp_path).context("failed to create temporary profile file")?;
        file.write_all(data.as_bytes()).context("failed to write temporary profile file")?;
        file.sync_all().context("failed to flush temporary profile file")?;
    }
    replace_file(&tmp_path, profile_path).context("failed to replace save profile")
}

pub(crate) fn load_last_character_from_paths(
    profile_path: &Path,
    character_dir: &Path,
) -> Result<Option<Character>> {
    if !profile_path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(profile_path).context("failed to read save profile")?;
    let profile: SaveProfile = serde_json::from_str(&data).context("failed to load save profile")?;
    let character_path = character_file_path(character_dir, &profile.last_character_id);
    if !character_path.exists() {
        return Ok(None);
    }
    match load_character_from_path(&character_path)? {
        LoadedSave::Loaded(character) => Ok(Some(*character)),
        LoadedSave::Reset { .. } => Ok(None),
    }
}

pub(crate) fn load_character_summaries_from_dir(character_dir: &Path) -> Result<Vec<CharacterSummary>> {
    if !character_dir.exists() {
        return Ok(Vec::new());
    }
    let mut summaries = Vec::new();
    for entry in fs::read_dir(character_dir).context("failed to read character save directory")? {
        let entry = entry.context("failed to read character save entry")?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let id = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("character").to_string();
        if let LoadedSave::Loaded(character) = load_character_from_path(&path)? {
            summaries.push(character_summary(&id, &character));
        }
    }
    summaries.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()).then(left.id.cmp(&right.id)));
    Ok(summaries)
}

fn character_summary(id: &str, character: &Character) -> CharacterSummary {
    CharacterSummary {
        id: id.to_string(),
        name: character.name.clone(),
        class_name: character.class_name().to_string(),
        level: character.level,
        death_mode: character.death_mode,
        location: character
            .active_dungeon
            .as_ref()
            .map(|dungeon| format!("Dungeon L{}", dungeon.floor))
            .unwrap_or_else(|| "Town".to_string()),
    }
}

pub(crate) fn save_character(character: &Character) -> Result<()> {
    save_active_character_to_paths(
        character,
        Path::new(PROFILE_PATH),
        Path::new(CHARACTER_SAVE_DIR),
    )
}
```

- [ ] **Step 4: Run verification**

Run: `cargo test save_character_profile_tracks_last_character_and_per_character_file`
Expected: PASS

Run: `cargo test character_summaries_are_loaded_from_per_character_directory`
Expected: PASS

---

### Task 2: Startup load and legacy migration

**Files:**
- Modify: `src/save.rs`
- Modify: `src/main.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests**

Add tests near the save tests:

```rust
#[test]
fn startup_load_prefers_last_character_from_profile() {
    let dir = env::temp_dir().join(format!(
        "crawltty-load-last-character-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    let profile_path = dir.join("profile.json");
    let character_dir = dir.join("characters");

    let mara = Character::new("Mara".to_string(), CharacterClass::Warrior, DeathMode::Softcore);
    let shade = Character::new("Shade".to_string(), CharacterClass::Rogue, DeathMode::Hardcore);
    save_active_character_to_paths(&mara, &profile_path, &character_dir).unwrap();
    save_active_character_to_paths(&shade, &profile_path, &character_dir).unwrap();

    let loaded = load_startup_character_from_paths(&profile_path, &character_dir, &dir.join("save.json")).unwrap();
    assert_eq!(loaded.unwrap().name, "Shade");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn legacy_single_save_migrates_to_profile_and_character_file() {
    let dir = env::temp_dir().join(format!(
        "crawltty-legacy-multi-save-migration-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let profile_path = dir.join("profile.json");
    let character_dir = dir.join("characters");
    let legacy_path = dir.join("save.json");

    let legacy = Character::new("Old Hero".to_string(), CharacterClass::Warrior, DeathMode::Softcore);
    save_character_to_path(&legacy, &legacy_path).unwrap();

    let loaded = load_startup_character_from_paths(&profile_path, &character_dir, &legacy_path).unwrap();
    assert_eq!(loaded.unwrap().name, "Old Hero");
    assert!(character_dir.join("old-hero.json").exists());
    let profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&profile_path).unwrap()).unwrap();
    assert_eq!(profile["last_character_id"], "old-hero");

    let _ = fs::remove_dir_all(&dir);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test startup_load_prefers_last_character_from_profile`
Expected: FAIL because `load_startup_character_from_paths` does not exist.

Run: `cargo test legacy_single_save_migrates_to_profile_and_character_file`
Expected: FAIL because `load_startup_character_from_paths` does not exist.

- [ ] **Step 3: Implement minimal code**

In `src/save.rs`, change `load_or_create_character` to call a new pure loader before falling back to the UI creator:

```rust
pub(crate) fn load_or_create_character(
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<Character> {
    if let Some(character) = load_startup_character_from_paths(
        Path::new(PROFILE_PATH),
        Path::new(CHARACTER_SAVE_DIR),
        Path::new(SAVE_PATH),
    )? {
        return Ok(character);
    }
    let character = create_character(terminal, "")?;
    save_character(&character)?;
    Ok(character)
}

pub(crate) fn load_startup_character_from_paths(
    profile_path: &Path,
    character_dir: &Path,
    legacy_save_path: &Path,
) -> Result<Option<Character>> {
    if let Some(character) = load_last_character_from_paths(profile_path, character_dir)? {
        return Ok(Some(character));
    }
    if legacy_save_path.exists() && load_character_summaries_from_dir(character_dir)?.is_empty() {
        return match load_character_from_path(legacy_save_path)? {
            LoadedSave::Loaded(character) => {
                save_active_character_to_paths(&character, profile_path, character_dir)?;
                Ok(Some(*character))
            }
            LoadedSave::Reset { .. } => Ok(None),
        };
    }
    Ok(None)
}
```

In `src/main.rs`, make `reset-save` remove `SAVE_PATH`, `PROFILE_PATH`, and `CHARACTER_SAVE_DIR` by adding a helper:

```rust
fn reset_saves() -> Result<()> {
    let mut deleted = false;
    for path in [SAVE_PATH, PROFILE_PATH] {
        match fs::remove_file(path) {
            Ok(()) => deleted = true,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err).with_context(|| format!("failed to delete {path}")),
        }
    }
    match fs::remove_dir_all(CHARACTER_SAVE_DIR) {
        Ok(()) => deleted = true,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err).context("failed to delete character saves"),
    }
    if deleted {
        println!("Deleted saves.");
    } else {
        println!("No save file found.");
    }
    Ok(())
}
```

Then call `reset_saves()?` from the `reset-save` branch.

- [ ] **Step 4: Run verification**

Run: `cargo test startup_load_prefers_last_character_from_profile`
Expected: PASS

Run: `cargo test legacy_single_save_migrates_to_profile_and_character_file`
Expected: PASS

---

### Task 3: Character selection/creation screen and town command

**Files:**
- Modify: `src/save.rs`
- Modify: `src/main.rs`
- Modify: `src/ui.rs`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing tests**

Add tests near the character creation/town render tests:

```rust
#[test]
fn character_select_screen_lists_saved_characters_and_new_character_row() {
    let summaries = vec![
        CharacterSummary {
            id: "mara".to_string(),
            name: "Mara".to_string(),
            class_name: "Warrior".to_string(),
            level: 8,
            death_mode: DeathMode::Softcore,
            location: "Town".to_string(),
        },
        CharacterSummary {
            id: "shade".to_string(),
            name: "Shade".to_string(),
            class_name: "Rogue".to_string(),
            level: 3,
            death_mode: DeathMode::Hardcore,
            location: "Dungeon L4".to_string(),
        },
    ];
    let mut terminal = test_terminal(80, 18);
    terminal
        .draw(|frame| render_character_select_screen(frame, &summaries, "shade", 1, ""))
        .unwrap();
    let body = terminal.backend().to_string();
    assert!(body.contains("Characters"));
    assert!(body.contains("Mara"));
    assert!(body.contains("Warrior"));
    assert!(body.contains("Lv 8"));
    assert!(body.contains("Shade"));
    assert!(body.contains("Hardcore"));
    assert!(body.contains("Dungeon L4"));
    assert!(body.contains("+ New Character"));
    assert!(body.contains("n=new"));
    assert!(body.contains("Esc=back"));
}

#[test]
fn town_footer_lists_character_switch_command() {
    let c = test_character();
    let mut terminal = test_terminal(80, 24);
    terminal.draw(|frame| render_town(frame, &c, "")).unwrap();
    let body = terminal.backend().to_string();
    assert!(body.contains("c=characters"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test character_select_screen_lists_saved_characters_and_new_character_row`
Expected: FAIL because `render_character_select_screen` does not exist.

Run: `cargo test town_footer_lists_character_switch_command`
Expected: FAIL because the town footer lacks `c=characters`.

- [ ] **Step 3: Implement minimal code**

In `src/save.rs`, add rendering and the interactive menu:

```rust
pub(crate) fn render_character_select_screen(
    frame: &mut Frame,
    summaries: &[CharacterSummary],
    current_id: &str,
    selected: usize,
    message: &str,
) {
    let footer_height = if message.is_empty() { 3 } else { 4 };
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(footer_height),
    ])
    .split(frame.area());
    frame.render_widget(
        Paragraph::new("Choose a hero, or create a new one.").block(gothic_block("Characters")),
        layout[0],
    );

    let mut lines = Vec::new();
    for (index, summary) in summaries.iter().enumerate() {
        let marker = if selected == index { SELECTION_CURSOR } else { " " };
        let current = if summary.id == current_id { " current" } else { "" };
        let style = if selected == index {
            selected_cursor_style()
        } else if summary.death_mode == DeathMode::Hardcore {
            Style::default().fg(DANGER_COLOR)
        } else {
            Style::default()
        };
        lines.push(Line::styled(
            format!(
                "{marker} {:<16} {:<10} Lv {:<3} {:<8} {}{current}",
                summary.name,
                summary.class_name,
                summary.level,
                summary.death_mode.label(),
                summary.location
            ),
            style,
        ));
    }
    let new_index = summaries.len();
    let new_marker = if selected == new_index { SELECTION_CURSOR } else { " " };
    lines.push(Line::styled(
        format!("{new_marker} + New Character"),
        if selected == new_index { selected_cursor_style() } else { Style::default().fg(ACTION_COLOR) },
    ));

    frame.render_widget(
        Paragraph::new(lines)
            .block(gothic_block_selected("Saved Characters", true))
            .wrap(Wrap { trim: false }),
        layout[1],
    );
    let footer = if message.is_empty() {
        "Up/Down=select  Enter=play  n=new  Esc=back".to_string()
    } else {
        format!("{message}\nUp/Down=select  Enter=play  n=new  Esc=back")
    };
    frame.render_widget(
        Paragraph::new(command_footer_lines(footer)).block(gothic_block("Commands")),
        layout[2],
    );
}

pub(crate) fn character_select_menu(
    current: &Character,
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<Option<Character>> {
    save_character(current)?;
    let current_id = character_id_from_name(&current.name);
    let mut selected = 0usize;
    let mut message = String::new();
    loop {
        let summaries = load_character_summaries_from_dir(Path::new(CHARACTER_SAVE_DIR))?;
        selected = selected.min(summaries.len());
        terminal
            .draw(|frame| render_character_select_screen(frame, &summaries, &current_id, selected, &message))
            .context("failed to draw character selection")?;
        let key = match read_ui_input_nav_timed(CURSOR_PULSE_INTERVAL)? {
            UiInput::Key(key) => key,
            UiInput::Redraw => continue,
            UiInput::Tick => {
                toggle_cursor_pulse_frame();
                continue;
            }
        };
        match key {
            'w' | 'W' => {
                selected = selected.saturating_sub(1);
                message.clear();
            }
            's' | 'S' => {
                selected = (selected + 1).min(summaries.len());
                message.clear();
            }
            'n' | 'N' => {
                let new_character = create_character(terminal, "")?;
                let new_id = character_id_from_name(&new_character.name);
                if character_file_path(Path::new(CHARACTER_SAVE_DIR), &new_id).exists() {
                    message = "A character with that save name already exists.".to_string();
                } else {
                    save_character(&new_character)?;
                    return Ok(Some(new_character));
                }
            }
            '\n' if selected == summaries.len() => {
                let new_character = create_character(terminal, "")?;
                let new_id = character_id_from_name(&new_character.name);
                if character_file_path(Path::new(CHARACTER_SAVE_DIR), &new_id).exists() {
                    message = "A character with that save name already exists.".to_string();
                } else {
                    save_character(&new_character)?;
                    return Ok(Some(new_character));
                }
            }
            '\n' => {
                if let Some(summary) = summaries.get(selected) {
                    if summary.id == current_id {
                        message = format!("{} is already active.", summary.name);
                    } else if let LoadedSave::Loaded(character) =
                        load_character_from_path(&character_file_path(Path::new(CHARACTER_SAVE_DIR), &summary.id))?
                    {
                        save_profile_to_path(Path::new(PROFILE_PATH), &summary.id)?;
                        return Ok(Some(*character));
                    } else {
                        message = "That character save could not be loaded.".to_string();
                    }
                }
            }
            '\u{1b}' => return Ok(None),
            _ => {}
        }
    }
}
```

Add this helper on `DeathMode` in `src/model.rs` if it does not exist:

```rust
impl DeathMode {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            DeathMode::Softcore => "Softcore",
            DeathMode::Hardcore => "Hardcore",
        }
    }
}
```

In `src/ui.rs`, add `("c", "characters")` to the town footer command list.

In `src/main.rs`, add the town key handler:

```rust
'c' | 'C' => {
    if let Some(new_character) = character_select_menu(character, terminal)? {
        *character = new_character;
        *town_message = take_startup_town_message(character);
    }
}
```

- [ ] **Step 4: Run verification**

Run: `cargo test character_select_screen_lists_saved_characters_and_new_character_row`
Expected: PASS

Run: `cargo test town_footer_lists_character_switch_command`
Expected: PASS

---

### Task 4: Active-character Hardcore deletion and docs

**Files:**
- Modify: `src/dungeon.rs`
- Modify: `README.md`
- Modify: `design.md`
- Test: `src/tests.rs`

- [ ] **Step 1: Write failing test**

Add a test near `hardcore_death_deletes_save_and_returns_outcome`:

```rust
#[test]
fn hardcore_death_deletes_active_character_save_file() {
    let temp_dir = env::temp_dir().join(format!(
        "crawltty-hardcore-active-character-delete-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_dir);
    let profile_path = temp_dir.join("profile.json");
    let character_dir = temp_dir.join("characters");

    let mut c = Character::new("Doomed Hero".to_string(), CharacterClass::Rogue, DeathMode::Hardcore);
    save_active_character_to_paths(&c, &profile_path, &character_dir).unwrap();
    let save_path = character_dir.join("doomed-hero.json");
    assert!(save_path.exists());

    c.hp = 0;
    c.active_dungeon = Some(open_test_dungeon(2, 2, Vec::new()));
    let outcome = check_death_with_save_path(&mut c, &save_path);

    assert_eq!(outcome, DeathOutcome::HardcoreDeath);
    assert!(!save_path.exists());
    assert!(c.active_dungeon.is_none());

    let _ = fs::remove_dir_all(&temp_dir);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test hardcore_death_deletes_active_character_save_file`
Expected: FAIL if `check_death` still deletes only the legacy singleton path.

- [ ] **Step 3: Implement minimal code and docs**

In `src/dungeon.rs`, update `check_death`:

```rust
pub(crate) fn check_death(c: &mut Character) -> DeathOutcome {
    let save_path = character_save_path(c);
    check_death_with_save_path(c, &save_path)
}
```

Update `README.md` to mention that startup loads the last played character and town has `c` for character selection/creation.

Update `design.md` save/UI sections to say saves use `saves/profile.json` plus `saves/characters/<id>.json`, stashes remain per-character, and town includes a character selection/creation screen.

- [ ] **Step 4: Run verification**

Run: `cargo test hardcore_death_deletes_active_character_save_file`
Expected: PASS

Run: `scripts/agent-commit-guard.sh --fix`
Expected: PASS (`cargo fmt`, `cargo test`, `cargo check`).

- [ ] **Step 5: Commit**

```bash
git add src/model.rs src/save.rs src/main.rs src/ui.rs src/dungeon.rs src/tests.rs README.md design.md docs/superpowers/plans/2026-05-17-multi-character-saves.md
git commit -m "Add multi-character save selection"
```
