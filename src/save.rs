fn load_or_create_character() -> Result<Character> {
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
        match read_key_char() {
            '1' => break DeathMode::Softcore,
            '2' => break DeathMode::Hardcore,
            _ => println!("Choose 1 or 2."),
        }
    };
    Ok(Character::new(name.trim().to_string(), death_mode))
}

fn save_character(character: &Character) -> Result<()> {
    save_character_to_path(character, Path::new(SAVE_PATH))
}

fn save_character_to_path(character: &Character, save_path: &Path) -> Result<()> {
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
    fs::rename(&tmp_path, save_path).context("failed to replace save file")
}

