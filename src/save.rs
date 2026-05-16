use crate::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
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

fn create_character(
    terminal: &mut ratatui::DefaultTerminal,
    startup_message: &str,
) -> Result<Character> {
    let mut name = String::new();
    let mut selected_class = CharacterClass::Warrior;
    let mut death_mode = DeathMode::Softcore;
    let mut message = startup_message.to_string();
    loop {
        terminal
            .draw(|frame| {
                render_character_creation_screen(frame, &name, selected_class, death_mode, &message)
            })
            .context("failed to draw character creation")?;
        let key = match read_ui_input()? {
            UiInput::Key(key) => key,
            UiInput::Redraw => continue,
        };
        match key {
            '\n' => {
                if name.trim().is_empty() {
                    message = "Enter a character name.".to_string();
                } else {
                    return Ok(Character::new(
                        name.trim().to_string(),
                        selected_class,
                        death_mode,
                    ));
                }
            }
            '1' => {
                selected_class = CharacterClass::Warrior;
                message.clear();
            }
            '2' => {
                selected_class = CharacterClass::Rogue;
                message.clear();
            }
            's' | 'S' => {
                death_mode = DeathMode::Softcore;
                message.clear();
            }
            'h' | 'H' => {
                death_mode = DeathMode::Hardcore;
                message.clear();
            }
            '\t' => {
                death_mode = match death_mode {
                    DeathMode::Softcore => DeathMode::Hardcore,
                    DeathMode::Hardcore => DeathMode::Softcore,
                };
                message.clear();
            }
            '\u{8}' | '\u{7f}' => {
                name.pop();
                message.clear();
            }
            '\u{1b}' => message = "Create a character to begin.".to_string(),
            key if !key.is_control() && name.chars().count() < 32 => {
                name.push(key);
                message.clear();
            }
            _ => {}
        }
    }
}

pub(crate) fn render_character_creation_screen(
    frame: &mut Frame,
    name: &str,
    selected_class: CharacterClass,
    selected_death_mode: DeathMode,
    message: &str,
) {
    let footer_height = if message.is_empty() { 3 } else { 4 };
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(footer_height),
    ])
    .split(frame.area());
    frame.render_widget(
        Paragraph::new("CrawlTTY").block(
            Block::default()
                .borders(Borders::ALL)
                .title("Character Creation"),
        ),
        layout[0],
    );

    let softcore_marker = if selected_death_mode == DeathMode::Softcore {
        ">"
    } else {
        " "
    };
    let hardcore_marker = if selected_death_mode == DeathMode::Hardcore {
        ">"
    } else {
        " "
    };
    let warrior_marker = if selected_class == CharacterClass::Warrior {
        ">"
    } else {
        " "
    };
    let rogue_marker = if selected_class == CharacterClass::Rogue {
        ">"
    } else {
        " "
    };
    let lines = vec![
        Line::from("ASCII terminal action RPG prototype"),
        Line::from(""),
        Line::from(format!(
            "Name: {}",
            if name.is_empty() { "_" } else { name }
        )),
        Line::from(""),
        Line::styled(
            format!("{warrior_marker} Warrior - armored melee skills and mana."),
            Style::default().fg(Color::Cyan),
        ),
        Line::styled(
            format!("{rogue_marker} Rogue - dagger burst, Energy, and combo points."),
            Style::default().fg(Color::Green),
        ),
        Line::from(""),
        Line::styled(
            format!("{softcore_marker} Softcore - death returns you to town."),
            Style::default().fg(Color::Green),
        ),
        Line::styled(
            format!("{hardcore_marker} Hardcore - death permanently ends the character."),
            Style::default().fg(Color::Red),
        ),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("New Character"),
            )
            .wrap(Wrap { trim: false }),
        layout[1],
    );

    let commands = "Type=name  Backspace=delete  1/2=class  S/H or Tab=death mode  Enter=confirm";
    let footer = if message.is_empty() {
        commands.to_string()
    } else {
        format!("{message}\n{commands}")
    };
    frame.render_widget(
        Paragraph::new(footer).block(Block::default().borders(Borders::ALL).title("Commands")),
        layout[2],
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
            Ok(save) => Ok(LoadedSave::Loaded(Box::new(save.character))),
            Err(err) => reset_save_with_warning(save_path, bad_save_warning(save_version, &err)),
        };
    }

    match serde_json::from_str::<Character>(&data) {
        Ok(character) => Ok(LoadedSave::Loaded(Box::new(character))),
        Err(err) => reset_save_with_warning(save_path, bad_save_warning(save_version, &err)),
    }
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
