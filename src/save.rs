use crate::*;
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

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

#[derive(Debug)]
pub(crate) enum LoadedSave {
    Loaded(Box<Character>),
    Reset { warning: String },
}

pub(crate) fn load_or_create_character(
    terminal: &mut ratatui::DefaultTerminal,
) -> Result<Character> {
    if Path::new(SAVE_PATH).exists() {
        match load_character_from_path(Path::new(SAVE_PATH))? {
            LoadedSave::Loaded(character) => return Ok(*character),
            LoadedSave::Reset { warning } => return create_character(terminal, &warning),
        }
    }
    create_character(terminal, "")
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
    save_character_to_path(character, Path::new(SAVE_PATH))
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
