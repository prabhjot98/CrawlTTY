use crate::*;

pub(crate) fn load_or_create_character() -> Result<Character> {
    if Path::new(SAVE_PATH).exists() {
        let data = fs::read_to_string(SAVE_PATH).context("failed to read save file")?;
        return serde_json::from_str(&data).context("failed to parse save file");
    }
    println!("CrawlTTY");
    println!("ASCII terminal action RPG prototype");
    let name = prompt("Character name: ");
    println!("{BOLD}Choose death mode:{RESET}");
    println!("{GREEN}Softcore{RESET}: death returns you to town.");
    println!("{RED}Hardcore{RESET}: death permanently ends the character.");
    print_footer(&[&format!(
        "{BOLD}Character creation:{RESET} {GREEN}1{RESET}=Softcore  {RED}2{RESET}=Hardcore"
    )]);
    let death_mode = loop {
        match read_key_char()? {
            '1' => break DeathMode::Softcore,
            '2' => break DeathMode::Hardcore,
            _ => println!("Choose 1 or 2."),
        }
    };
    Ok(Character::new(name.trim().to_string(), death_mode))
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
    let data = serde_json::to_string_pretty(character).context("failed to serialize save")?;
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
    fs::rename(tmp_path, save_path).context("failed to move temporary save file")
}
