use crate::*;
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};
use std::path::PathBuf;

pub(crate) const SAVE_VERSION: &str = env!("CARGO_PKG_VERSION");
const LEGACY_SAVE_VERSION: &str = "0.0.0";

#[derive(Debug, Deserialize)]
struct SaveHeader {
    save_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SaveFile {
    character: Character,
}

#[derive(Serialize)]
struct SaveFileRef<'a> {
    save_version: &'static str,
    character: &'a Character,
}

#[derive(Debug, Deserialize, Serialize)]
struct SaveProfile {
    last_character_id: String,
}

#[derive(Debug)]
pub(crate) enum LoadedSave {
    Loaded(Box<Character>),
    Reset { warning: String },
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
    character_file_path(
        Path::new(CHARACTER_SAVE_DIR),
        &character_id_from_name(&character.name),
    )
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
    if let Some(parent) = profile_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).context("failed to create save profile directory")?;
    }
    let profile = SaveProfile {
        last_character_id: last_character_id.to_string(),
    };
    let data =
        serde_json::to_string_pretty(&profile).context("failed to serialize save profile")?;
    let tmp_path = profile_path.with_file_name(format!(
        "{}.tmp",
        profile_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("profile.json")
    ));
    {
        let mut file =
            fs::File::create(&tmp_path).context("failed to create temporary profile file")?;
        file.write_all(data.as_bytes())
            .context("failed to write temporary profile file")?;
        file.sync_all()
            .context("failed to flush temporary profile file")?;
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
    let profile: SaveProfile =
        serde_json::from_str(&data).context("failed to load save profile")?;
    let character_path = character_file_path(character_dir, &profile.last_character_id);
    if !character_path.exists() {
        return Ok(None);
    }
    match load_character_from_path(&character_path)? {
        LoadedSave::Loaded(character) => Ok(Some(*character)),
        LoadedSave::Reset { warning } => {
            drop(warning);
            Ok(None)
        }
    }
}

pub(crate) fn load_character_summaries_from_dir(
    character_dir: &Path,
) -> Result<Vec<CharacterSummary>> {
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
        let id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("character")
            .to_string();
        match load_character_from_path(&path)? {
            LoadedSave::Loaded(character) => summaries.push(character_summary(&id, &character)),
            LoadedSave::Reset { warning } => drop(warning),
        }
    }
    summaries.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then(left.id.cmp(&right.id))
    });
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
            LoadedSave::Reset { warning } => {
                drop(warning);
                Ok(None)
            }
        };
    }
    Ok(None)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CharacterCreationStep {
    Class,
    Name,
    DeathMode,
}

pub(crate) struct CharacterCreationState {
    pub(crate) step: CharacterCreationStep,
    pub(crate) name: String,
    pub(crate) selected_class: CharacterClass,
    pub(crate) death_mode: DeathMode,
    pub(crate) message: String,
}

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

impl CharacterCreationState {
    pub(crate) fn new(startup_message: &str) -> Self {
        Self {
            step: CharacterCreationStep::Class,
            name: String::new(),
            selected_class: CharacterClass::Warrior,
            death_mode: DeathMode::Softcore,
            message: startup_message.to_string(),
        }
    }

    pub(crate) fn handle_key(&mut self, key: char) -> Option<Character> {
        match (self.step, key) {
            (CharacterCreationStep::Class, '\n') => {
                self.step = CharacterCreationStep::Name;
                self.message.clear();
            }
            (CharacterCreationStep::Name, '\n') => {
                if self.name.trim().is_empty() {
                    self.message = "Enter a character name.".to_string();
                } else {
                    self.step = CharacterCreationStep::DeathMode;
                    self.message.clear();
                }
            }
            (CharacterCreationStep::DeathMode, '\n') => {
                return Some(Character::new(
                    self.name.trim().to_string(),
                    self.selected_class,
                    self.death_mode,
                ));
            }
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
            (CharacterCreationStep::DeathMode, KEY_ARROW_UP) => {
                self.death_mode = DeathMode::Softcore;
                self.message.clear();
            }
            (CharacterCreationStep::DeathMode, KEY_ARROW_DOWN) => {
                self.death_mode = DeathMode::Hardcore;
                self.message.clear();
            }
            (CharacterCreationStep::DeathMode, '\t') => {
                self.death_mode = match self.death_mode {
                    DeathMode::Softcore => DeathMode::Hardcore,
                    DeathMode::Hardcore => DeathMode::Softcore,
                };
                self.message.clear();
            }
            (CharacterCreationStep::Name, '\u{8}' | '\u{7f}') => {
                self.name.pop();
                self.message.clear();
            }
            (CharacterCreationStep::Name, key)
                if !key.is_control() && self.name.chars().count() < 32 =>
            {
                self.name.push(key);
                self.message.clear();
            }
            (CharacterCreationStep::Name, '\u{1b}') => {
                self.step = CharacterCreationStep::Class;
                self.message.clear();
            }
            (CharacterCreationStep::DeathMode, '\u{1b}') => {
                self.step = CharacterCreationStep::Name;
                self.message.clear();
            }
            (CharacterCreationStep::Class, '\u{1b}') => {
                self.message = "Create a character to begin.".to_string();
            }
            _ => {}
        }
        None
    }
}

fn create_character(
    terminal: &mut ratatui::DefaultTerminal,
    startup_message: &str,
) -> Result<Character> {
    let mut state = CharacterCreationState::new(startup_message);
    loop {
        terminal
            .draw(|frame| {
                render_character_creation_screen(
                    frame,
                    state.step,
                    &state.name,
                    state.selected_class,
                    state.death_mode,
                    &state.message,
                )
            })
            .context("failed to draw character creation")?;
        let key = match read_ui_input_raw_arrows_timed(CURSOR_PULSE_INTERVAL)? {
            UiInput::Key(key) => key,
            UiInput::Redraw => continue,
            UiInput::Tick => {
                toggle_cursor_pulse_frame();
                continue;
            }
        };
        if let Some(character) = state.handle_key(key) {
            return Ok(character);
        }
    }
}

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
        let marker = if selected == index {
            SELECTION_CURSOR
        } else {
            " "
        };
        let current = if summary.id == current_id {
            " current"
        } else {
            ""
        };
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
    let new_marker = if selected == new_index {
        SELECTION_CURSOR
    } else {
        " "
    };
    let new_style = if selected == new_index {
        selected_cursor_style()
    } else {
        Style::default().fg(ACTION_COLOR)
    };
    lines.push(Line::styled(
        format!("{new_marker} + New Character"),
        new_style,
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
            .draw(|frame| {
                render_character_select_screen(frame, &summaries, &current_id, selected, &message)
            })
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
            'n' | 'N' => match create_new_character_from_menu(terminal)? {
                Ok(new_character) => return Ok(Some(new_character)),
                Err(error_message) => message = error_message,
            },
            '\n' if selected == summaries.len() => {
                match create_new_character_from_menu(terminal)? {
                    Ok(new_character) => return Ok(Some(new_character)),
                    Err(error_message) => message = error_message,
                }
            }
            '\n' => {
                if let Some(summary) = summaries.get(selected) {
                    if summary.id == current_id {
                        message = format!("{} is already active.", summary.name);
                    } else if let LoadedSave::Loaded(character) = load_character_from_path(
                        &character_file_path(Path::new(CHARACTER_SAVE_DIR), &summary.id),
                    )? {
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

fn create_new_character_from_menu(
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<std::result::Result<Character, String>> {
    let new_character = create_character(terminal, "")?;
    let new_id = character_id_from_name(&new_character.name);
    if character_file_path(Path::new(CHARACTER_SAVE_DIR), &new_id).exists() {
        Ok(Err(
            "A character with that save name already exists.".to_string()
        ))
    } else {
        save_character(&new_character)?;
        Ok(Ok(new_character))
    }
}

pub(crate) fn render_character_creation_screen(
    frame: &mut Frame,
    active_step: CharacterCreationStep,
    name: &str,
    selected_class: CharacterClass,
    selected_death_mode: DeathMode,
    message: &str,
) {
    let footer_height = if message.is_empty() { 3 } else { 4 };
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(3),
        Constraint::Length(4),
        Constraint::Min(0),
        Constraint::Length(footer_height),
    ])
    .split(frame.area());
    frame.render_widget(
        Paragraph::new("").block(gothic_block("Character Creation")),
        layout[0],
    );

    let softcore_marker = if active_step == CharacterCreationStep::DeathMode
        && selected_death_mode == DeathMode::Softcore
    {
        SELECTION_CURSOR
    } else {
        " "
    };
    let hardcore_marker = if active_step == CharacterCreationStep::DeathMode
        && selected_death_mode == DeathMode::Hardcore
    {
        SELECTION_CURSOR
    } else {
        " "
    };
    let warrior_marker = if active_step == CharacterCreationStep::Class
        && selected_class == CharacterClass::Warrior
    {
        SELECTION_CURSOR
    } else {
        " "
    };
    let rogue_marker =
        if active_step == CharacterCreationStep::Class && selected_class == CharacterClass::Rogue {
            SELECTION_CURSOR
        } else {
            " "
        };
    let sorceress_marker = if active_step == CharacterCreationStep::Class
        && selected_class == CharacterClass::Sorceress
    {
        SELECTION_CURSOR
    } else {
        " "
    };

    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                format!("{warrior_marker} Warrior - armored melee skills and mana."),
                if active_step == CharacterCreationStep::Class
                    && selected_class == CharacterClass::Warrior
                {
                    selected_cursor_style()
                } else {
                    Style::default().fg(Color::Cyan)
                },
            ),
            Line::styled(
                format!("{rogue_marker} Rogue - dagger burst, Energy, and combo points."),
                if active_step == CharacterCreationStep::Class
                    && selected_class == CharacterClass::Rogue
                {
                    selected_cursor_style()
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
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
        ])
        .block(gothic_block_selected(
            "Step 1: Class",
            active_step == CharacterCreationStep::Class,
        ))
        .wrap(Wrap { trim: false }),
        layout[1],
    );

    frame.render_widget(
        Paragraph::new(format!(
            "Name: {}",
            if name.is_empty() { "_" } else { name }
        ))
        .block(gothic_block_selected(
            "Step 2: Name",
            active_step == CharacterCreationStep::Name,
        )),
        layout[2],
    );

    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                format!("{softcore_marker} Softcore - death returns you to town."),
                if active_step == CharacterCreationStep::DeathMode
                    && selected_death_mode == DeathMode::Softcore
                {
                    selected_cursor_style()
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
            Line::styled(
                format!("{hardcore_marker} Hardcore - death permanently ends the character."),
                if active_step == CharacterCreationStep::DeathMode
                    && selected_death_mode == DeathMode::Hardcore
                {
                    selected_cursor_style()
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
        ])
        .block(gothic_block_selected(
            "Step 3: Death Mode",
            active_step == CharacterCreationStep::DeathMode,
        ))
        .wrap(Wrap { trim: false }),
        layout[3],
    );

    let commands = match active_step {
        CharacterCreationStep::Class => "Up/Down or 1/2/3=class  Enter=next  Esc=back",
        CharacterCreationStep::Name => "Type=name  Backspace=delete  Enter=next  Esc=back",
        CharacterCreationStep::DeathMode => "Up/Down or Tab=mode  Enter=confirm  Esc=back",
    };
    let footer = if message.is_empty() {
        commands.to_string()
    } else {
        format!("{message}\n{commands}")
    };
    frame.render_widget(
        Paragraph::new(command_footer_lines(footer)).block(gothic_block("Commands")),
        layout[5],
    );
}

pub(crate) fn load_character_from_path(save_path: &Path) -> Result<LoadedSave> {
    let data = fs::read_to_string(save_path).context("failed to read save file")?;
    let header: SaveHeader = match serde_json::from_str(&data) {
        Ok(header) => header,
        Err(err) => {
            return reset_save_with_warning(save_path, bad_save_warning("unknown", &err));
        }
    };
    let save_version = header
        .save_version
        .as_deref()
        .unwrap_or(LEGACY_SAVE_VERSION);
    if save_major_version_changed(save_version, SAVE_VERSION) {
        return reset_save_with_warning(
            save_path,
            incompatible_save_warning(save_version, SAVE_VERSION),
        );
    }

    if header.save_version.is_some() {
        return match serde_json::from_str::<SaveFile>(&data) {
            Ok(save) => Ok(LoadedSave::Loaded(Box::new(normalized_loaded_character(
                save.character,
            )))),
            Err(err) => reset_save_with_warning(save_path, bad_save_warning(save_version, &err)),
        };
    }

    match serde_json::from_str::<Character>(&data) {
        Ok(character) => Ok(LoadedSave::Loaded(Box::new(normalized_loaded_character(
            character,
        )))),
        Err(err) => reset_save_with_warning(save_path, bad_save_warning(save_version, &err)),
    }
}

fn normalized_loaded_character(mut character: Character) -> Character {
    normalize_locked_skill_ranks(&mut character);
    character
}

pub(crate) fn save_major_version_changed(save_version: &str, game_version: &str) -> bool {
    parse_major_version(save_version) != parse_major_version(game_version)
}

fn parse_major_version(version: &str) -> Option<u64> {
    version.split('.').next()?.parse().ok()
}

fn reset_save_with_warning(save_path: &Path, warning: String) -> Result<LoadedSave> {
    fs::remove_file(save_path).context("failed to reset save file")?;
    Ok(LoadedSave::Reset { warning })
}

fn incompatible_save_warning(save_version: &str, game_version: &str) -> String {
    format!(
        "Warning: save version {save_version} is incompatible with game version {game_version}. The existing save was reset."
    )
}

fn bad_save_warning(save_version: &str, err: &serde_json::Error) -> String {
    format!(
        "Warning: save version {save_version} could not be loaded ({err}). The existing save was reset."
    )
}

pub(crate) fn save_character(character: &Character) -> Result<()> {
    save_active_character_to_paths(
        character,
        Path::new(PROFILE_PATH),
        Path::new(CHARACTER_SAVE_DIR),
    )
}

pub(crate) fn append_autosave_status(character: &Character, message: &mut String) {
    if let Err(err) = save_character(character) {
        if !message.is_empty() {
            message.push(' ');
        }
        message.push_str(&format!("Autosave failed: {err:#}."));
    }
}

pub(crate) fn save_character_to_path(character: &Character, save_path: &Path) -> Result<()> {
    if let Some(parent) = save_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).context("failed to create save directory")?;
    }
    let save = SaveFileRef {
        save_version: SAVE_VERSION,
        character,
    };
    let data = serde_json::to_string_pretty(&save).context("failed to serialize save")?;
    let tmp_path = save_path.with_file_name(format!(
        "{}.tmp",
        save_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("save.json")
    ));
    {
        let mut file =
            fs::File::create(&tmp_path).context("failed to create temporary save file")?;
        file.write_all(data.as_bytes())
            .context("failed to write temporary save file")?;
        file.sync_all()
            .context("failed to flush temporary save file")?;
    }
    replace_file(&tmp_path, save_path).context("failed to replace save file")
}

#[cfg(windows)]
pub(crate) fn replace_file(tmp_path: &Path, save_path: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let tmp_wide: Vec<u16> = tmp_path.as_os_str().encode_wide().chain([0]).collect();
    let save_wide: Vec<u16> = save_path.as_os_str().encode_wide().chain([0]).collect();
    let result = unsafe {
        MoveFileExW(
            tmp_wide.as_ptr(),
            save_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        return Err(io::Error::last_os_error()).context("failed to atomically replace save file");
    }
    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn replace_file(tmp_path: &Path, save_path: &Path) -> Result<()> {
    fs::rename(tmp_path, save_path).context("failed to move temporary save file")?;
    if let Some(parent) = save_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::File::open(parent)
            .context("failed to open save directory for syncing")?
            .sync_all()
            .context("failed to sync save directory")?;
    }
    Ok(())
}
