use crate::*;

pub(crate) fn skill_tree_menu(c: &mut Character) {
    let mut message = String::new();
    loop {
        clear_screen();
        println!("{BOLD}{CYAN}Ironbound Skill Tree{RESET}");
        println!("{}", unspent_skills_text(c.unspent_skills));
        if !message.is_empty() {
            println!("{YELLOW}{message}{RESET}");
        }
        println!();
        println!("{BOLD}Weapons Branch{RESET}");
        print_skill_upgrade_preview(
            '1',
            "Cleave",
            c.cleave_rank,
            "cost 5 mana, cd 1, hits up to 3 adjacent enemies",
            cleave_percent_for_rank(c.cleave_rank),
            cleave_percent_for_rank(next_skill_rank(c.cleave_rank)),
            "% weapon damage",
        );
        print_mastery_status(c, "Cleave");
        print_skill_upgrade_preview(
            '4',
            "Deep Cut",
            c.deep_cut_rank,
            "passive melee bleed chance and damage; requires Cleave rank 2 for upgrades",
            deep_cut_chance_for_rank(c.deep_cut_rank),
            deep_cut_chance_for_rank(next_skill_rank(c.deep_cut_rank)),
            "% bleed chance",
        );
        print_mastery_status(c, "Deep Cut");
        println!(
            "   Bleed damage: {} now, {} next.",
            deep_cut_damage_for_rank(c.deep_cut_rank),
            deep_cut_damage_for_rank(next_skill_rank(c.deep_cut_rank))
        );
        println!("{BOLD}Defense Branch{RESET}");
        print_skill_upgrade_preview(
            '2',
            "Shield Bash",
            c.shield_bash_rank,
            "cost 6 mana, cd 3, hits 1 enemy and staggers",
            shield_bash_percent_for_rank(c.shield_bash_rank),
            shield_bash_percent_for_rank(next_skill_rank(c.shield_bash_rank)),
            "% weapon damage",
        );
        print_mastery_status(c, "Shield Bash");
        print_skill_upgrade_preview(
            '5',
            "Iron Guard",
            c.iron_guard_rank,
            "passive armor while using a shield; requires Shield Bash rank 2 for upgrades",
            iron_guard_armor_bonus_for_rank(c.iron_guard_rank) as u32,
            iron_guard_armor_bonus_for_rank(next_skill_rank(c.iron_guard_rank)) as u32,
            " armor",
        );
        print_mastery_status(c, "Iron Guard");
        println!("{BOLD}Warcry Branch{RESET}");
        print_skill_upgrade_preview(
            '3',
            "Battle Cry",
            c.battle_cry_rank,
            "cost 8 mana, cd 6, grants attack charges",
            battle_cry_bonus_percent_for_rank(c.battle_cry_rank),
            battle_cry_bonus_percent_for_rank(next_skill_rank(c.battle_cry_rank)),
            "% bonus damage",
        );
        print_mastery_status(c, "Battle Cry");
        print_skill_upgrade_preview(
            '6',
            "Second Wind",
            c.second_wind_rank,
            "passive heal on kill while Battle Cry is active; requires Battle Cry rank 2 for upgrades",
            second_wind_heal_percent_for_rank(c.second_wind_rank),
            second_wind_heal_percent_for_rank(next_skill_rank(c.second_wind_rank)),
            "% max HP heal",
        );
        print_mastery_status(c, "Second Wind");
        println!();
        println!(
            "Each rank upgrade costs 1 skill point. Masteries are free at rank 5. Passive upgrades require rank 2 in their branch starter."
        );
        print_footer(&[&format!(
            "{BOLD}Skill Tree:{RESET} {GREEN}1{RESET}=Cleave {GREEN}2{RESET}=Bash {GREEN}3{RESET}=Cry {GREEN}4{RESET}=Deep Cut {GREEN}5{RESET}=Iron Guard {GREEN}6{RESET}=Second Wind {RED}Esc{RESET}=back"
        )]);
        match read_key_char() {
            '1' => message = choose_skill_or_mastery(c, "Cleave"),
            '2' => message = choose_skill_or_mastery(c, "Shield Bash"),
            '3' => message = choose_skill_or_mastery(c, "Battle Cry"),
            '4' => message = choose_skill_or_mastery(c, "Deep Cut"),
            '5' => message = choose_skill_or_mastery(c, "Iron Guard"),
            '6' => message = choose_skill_or_mastery(c, "Second Wind"),
            '\u{1b}' => break,
            _ => message = "Unknown skill command.".to_string(),
        }
    }
}

pub(crate) fn print_skill_upgrade_preview(
    key: char,
    name: &str,
    rank: u32,
    details: &str,
    current_value: u32,
    next_value: u32,
    value_label: &str,
) {
    println!("{GREEN}{key}) {name}{RESET} rank {rank}/5");
    println!("   Current: {CYAN}{current_value}{value_label}{RESET}; {details}");
    if rank >= 5 {
        println!("   Next: {YELLOW}MAX RANK{RESET}");
    } else {
        println!(
            "   Next rank {}: {GREEN}{next_value}{value_label}{RESET}; {details}",
            rank + 1
        );
    }
}

pub(crate) fn print_mastery_status(c: &Character, skill: &str) {
    if skill_rank(c, skill) < 5 {
        return;
    }
    if let Some(mastery) = mastery_for_skill(c, skill) {
        println!("   {MAGENTA}Mastery:{RESET} {}", mastery.name());
    } else {
        println!("   {YELLOW}Mastery available:{RESET} select this skill to choose a free path.");
    }
}

pub(crate) fn choose_skill_or_mastery(c: &mut Character, skill: &str) -> String {
    if skill_rank(c, skill) >= 5 {
        if mastery_for_skill(c, skill).is_some() {
            return format!(
                "{skill} already has a mastery: {}.",
                mastery_for_skill(c, skill).unwrap().name()
            );
        }
        return mastery_menu(c, skill);
    }
    upgrade_skill(c, skill)
}

pub(crate) fn upgrade_skill(c: &mut Character, skill: &str) -> String {
    if c.unspent_skills == 0 {
        return "No unspent skill points.".to_string();
    }
    if skill_rank(c, skill) >= 5 {
        return "That skill is already at max rank.".to_string();
    }
    if let Some(requirement) = unmet_skill_prerequisite(c, skill) {
        return requirement;
    }
    match skill {
        "Cleave" => c.cleave_rank += 1,
        "Shield Bash" => c.shield_bash_rank += 1,
        "Battle Cry" => c.battle_cry_rank += 1,
        "Deep Cut" => c.deep_cut_rank += 1,
        "Iron Guard" => c.iron_guard_rank += 1,
        "Second Wind" => c.second_wind_rank += 1,
        _ => return "Unknown skill.".to_string(),
    }
    c.unspent_skills -= 1;
    format!("Upgraded {skill} to rank {}.", skill_rank(c, skill))
}

impl SkillMastery {
    pub(crate) fn name(self) -> &'static str {
        match self {
            SkillMastery::ReapingCleave => "Reaping Cleave",
            SkillMastery::SunderingCleave => "Sundering Cleave",
            SkillMastery::BloodArc => "Blood Arc",
            SkillMastery::CrushingBash => "Crushing Bash",
            SkillMastery::LongBash => "Long Bash",
            SkillMastery::DazingBash => "Dazing Bash",
            SkillMastery::WarpathCry => "Warpath Cry",
            SkillMastery::TerrifyingCry => "Terrifying Cry",
            SkillMastery::RallyingCry => "Rallying Cry",
            SkillMastery::Hemorrhage => "Hemorrhage",
            SkillMastery::OpenWound => "Open Wound",
            SkillMastery::Bloodletting => "Bloodletting",
            SkillMastery::Bulwark => "Bulwark",
            SkillMastery::ShieldDiscipline => "Shield Discipline",
            SkillMastery::SpikedGuard => "Spiked Guard",
            SkillMastery::FreshKill => "Fresh Kill",
            SkillMastery::AdrenalSurge => "Adrenal Surge",
            SkillMastery::GrimRecovery => "Grim Recovery",
        }
    }
}

pub(crate) fn mastery_for_skill(c: &Character, skill: &str) -> Option<SkillMastery> {
    match skill {
        "Cleave" => c.cleave_mastery,
        "Shield Bash" => c.shield_bash_mastery,
        "Battle Cry" => c.battle_cry_mastery,
        "Deep Cut" => c.deep_cut_mastery,
        "Iron Guard" => c.iron_guard_mastery,
        "Second Wind" => c.second_wind_mastery,
        _ => None,
    }
}

pub(crate) fn set_mastery_for_skill(c: &mut Character, skill: &str, mastery: SkillMastery) {
    match skill {
        "Cleave" => c.cleave_mastery = Some(mastery),
        "Shield Bash" => c.shield_bash_mastery = Some(mastery),
        "Battle Cry" => c.battle_cry_mastery = Some(mastery),
        "Deep Cut" => c.deep_cut_mastery = Some(mastery),
        "Iron Guard" => c.iron_guard_mastery = Some(mastery),
        "Second Wind" => c.second_wind_mastery = Some(mastery),
        _ => {}
    }
}

pub(crate) fn mastery_options(c: &Character, skill: &str) -> [(SkillMastery, String); 3] {
    match skill {
        "Cleave" => [
            (
                SkillMastery::ReapingCleave,
                "Cleave target cap is removed: hit every cardinally adjacent enemy instead of max 3. Still costs 5 mana and spends 1 Battle Cry charge for the whole Cleave.".to_string(),
            ),
            (
                SkillMastery::SunderingCleave,
                "Each Cleave hit applies -2 enemy armor for 3 enemy turns. Stacks with normal damage and can reduce effective armor to 0, but not below 0.".to_string(),
            ),
            (
                SkillMastery::BloodArc,
                format!(
                    "Each Cleave hit forces Bleeding for 3 turns. Bleed damage uses your current Deep Cut value: {} damage/turn.",
                    deep_cut_damage_for_rank(c.deep_cut_rank)
                ),
            ),
        ],
        "Shield Bash" => [
            (
                SkillMastery::CrushingBash,
                format!(
                    "Shield Bash gains +10 percentage points of weapon damage per shield armor. Current shield armor {} = +{}% weapon damage.",
                    c.equipped_shield.armor.max(0),
                    c.equipped_shield.armor.max(0) * 10
                ),
            ),
            (
                SkillMastery::LongBash,
                "Shield Bash range increases from adjacent only to 2 tiles in a cardinal line. Still costs 6 mana, stuns, and spends 1 Battle Cry charge.".to_string(),
            ),
            (
                SkillMastery::DazingBash,
                "Shield Bash stun increases from 1 turn to 2 turns. Damage, mana cost, and cooldown are unchanged.".to_string(),
            ),
        ],
        "Battle Cry" => [
            (
                SkillMastery::WarpathCry,
                "Battle Cry grants +2 attack charges: 7 total instead of 5. Movement still does not consume charges.".to_string(),
            ),
            (
                SkillMastery::TerrifyingCry,
                "On activation, enemies within 3 tiles are staggered and skip 1 turn. Battle Cry still grants attack charges and damage reduction.".to_string(),
            ),
            (
                SkillMastery::RallyingCry,
                format!(
                    "On activation, restore 20% max HP and 20% max mana. Current values: {} HP and {} mana.",
                    (c.max_hp() / 5).max(1),
                    (c.max_mana() / 5).max(1)
                ),
            ),
        ],
        "Deep Cut" => [
            (
                SkillMastery::Hemorrhage,
                format!(
                    "Bleeding enemies below 50% HP take +2 bleed damage per tick. Current bleed becomes {} damage/turn while they are low HP.",
                    deep_cut_damage_for_rank(c.deep_cut_rank) + 2
                ),
            ),
            (
                SkillMastery::OpenWound,
                "When Deep Cut applies Bleeding, it also applies Vulnerable for 3 turns. Your physical hits deal +2 raw damage to Vulnerable enemies.".to_string(),
            ),
            (
                SkillMastery::Bloodletting,
                format!(
                    "Enemies killed by bleed restore 10% max HP. Current heal: {} HP.",
                    (c.max_hp() / 10).max(1)
                ),
            ),
        ],
        "Iron Guard" => [
            (
                SkillMastery::Bulwark,
                "Gain +4 armor while at or below 50% HP. This is added on top of Iron Guard's normal shield armor bonus.".to_string(),
            ),
            (
                SkillMastery::ShieldDiscipline,
                "Gain +3 dodge while using Iron Guard. This is a flat bonus to your dodge rating.".to_string(),
            ),
            (
                SkillMastery::SpikedGuard,
                "When an adjacent melee enemy hits you, Spiked Guard deals 2 physical damage back to that attacker.".to_string(),
            ),
        ],
        _ => [
            (
                SkillMastery::FreshKill,
                format!(
                    "Second Wind can trigger without Battle Cry, but heals for 50% of normal. Current no-Cry heal: {} HP; Battle Cry heal remains {} HP.",
                    (second_wind_heal_amount(c) / 2).max(1),
                    second_wind_heal_amount(c)
                ),
            ),
            (
                SkillMastery::AdrenalSurge,
                "When Second Wind triggers while Battle Cry has charges, restore +1 Battle Cry charge after the kill.".to_string(),
            ),
            (
                SkillMastery::GrimRecovery,
                "Second Wind overhealing becomes a temporary damage shield. The shield absorbs incoming damage before HP until depleted.".to_string(),
            ),
        ],
    }
}

pub(crate) fn mastery_menu(c: &mut Character, skill: &str) -> String {
    loop {
        clear_screen();
        println!("{BOLD}{MAGENTA}{skill} Mastery{RESET}");
        println!("Choose one free path. The other two will be locked out permanently.");
        let options = mastery_options(c, skill);
        for (i, (mastery, details)) in options.iter().enumerate() {
            println!("{GREEN}{}){RESET} {BOLD}{}{RESET}", i + 1, mastery.name());
            println!("   {details}");
        }
        print_footer(&[&format!(
            "{BOLD}Mastery:{RESET} {GREEN}1-3{RESET}=choose  {RED}Esc{RESET}=back"
        )]);
        match read_key_char() {
            key @ ('1' | '2' | '3') => {
                let index = key.to_digit(10).unwrap() as usize - 1;
                let mastery = options[index].0;
                set_mastery_for_skill(c, skill, mastery);
                return format!("Unlocked {} for {skill}.", mastery.name());
            }
            '\u{1b}' => return "Mastery selection cancelled.".to_string(),
            _ => {}
        }
    }
}

pub(crate) fn skill_rank(c: &Character, skill: &str) -> u32 {
    match skill {
        "Cleave" => c.cleave_rank,
        "Shield Bash" => c.shield_bash_rank,
        "Battle Cry" => c.battle_cry_rank,
        "Deep Cut" => c.deep_cut_rank,
        "Iron Guard" => c.iron_guard_rank,
        "Second Wind" => c.second_wind_rank,
        _ => 5,
    }
}

pub(crate) fn unmet_skill_prerequisite(c: &Character, skill: &str) -> Option<String> {
    match skill {
        "Deep Cut" if c.cleave_rank < 2 => {
            Some("Deep Cut upgrades require Cleave rank 2.".to_string())
        }
        "Iron Guard" if c.shield_bash_rank < 2 => {
            Some("Iron Guard upgrades require Shield Bash rank 2.".to_string())
        }
        "Second Wind" if c.battle_cry_rank < 2 => {
            Some("Second Wind upgrades require Battle Cry rank 2.".to_string())
        }
        _ => None,
    }
}

pub(crate) fn next_skill_rank(rank: u32) -> u32 {
    (rank + 1).min(5)
}
pub(crate) fn cleave_multiplier(c: &Character) -> f32 {
    cleave_multiplier_for_rank(c.cleave_rank)
}
pub(crate) fn cleave_multiplier_for_rank(rank: u32) -> f32 {
    0.8 + (rank.saturating_sub(1) as f32 * 0.10)
}
pub(crate) fn shield_bash_multiplier(c: &Character) -> f32 {
    shield_bash_multiplier_for_rank(c.shield_bash_rank)
}
pub(crate) fn shield_bash_multiplier_for_rank(rank: u32) -> f32 {
    0.7 + (rank.saturating_sub(1) as f32 * 0.10)
}
pub(crate) fn battle_cry_multiplier(c: &Character) -> f32 {
    battle_cry_multiplier_for_rank(c.battle_cry_rank)
}
pub(crate) fn battle_cry_multiplier_for_rank(rank: u32) -> f32 {
    1.20 + (rank.saturating_sub(1) as f32 * 0.05)
}
pub(crate) fn cleave_percent_for_rank(rank: u32) -> u32 {
    (cleave_multiplier_for_rank(rank) * 100.0).round() as u32
}
pub(crate) fn shield_bash_percent_for_rank(rank: u32) -> u32 {
    (shield_bash_multiplier_for_rank(rank) * 100.0).round() as u32
}
pub(crate) fn battle_cry_bonus_percent_for_rank(rank: u32) -> u32 {
    ((battle_cry_multiplier_for_rank(rank) - 1.0) * 100.0).round() as u32
}
pub(crate) fn cleave_percent(c: &Character) -> u32 {
    cleave_percent_for_rank(c.cleave_rank)
}
pub(crate) fn shield_bash_percent(c: &Character) -> u32 {
    shield_bash_percent_for_rank(c.shield_bash_rank)
}
pub(crate) fn battle_cry_bonus_percent(c: &Character) -> u32 {
    battle_cry_bonus_percent_for_rank(c.battle_cry_rank)
}
pub(crate) fn deep_cut_chance_for_rank(rank: u32) -> u32 {
    10 + rank.min(5) * 5
}
pub(crate) fn deep_cut_damage_for_rank(rank: u32) -> i32 {
    1 + rank.min(5).div_ceil(2) as i32
}
pub(crate) fn iron_guard_armor_bonus(c: &Character) -> i32 {
    iron_guard_armor_bonus_for_rank(c.iron_guard_rank)
}
pub(crate) fn iron_guard_armor_bonus_for_rank(rank: u32) -> i32 {
    1 + rank.min(5) as i32
}
pub(crate) fn second_wind_heal_percent_for_rank(rank: u32) -> u32 {
    5 + rank.min(5) * 5
}
pub(crate) fn second_wind_heal_amount(c: &Character) -> u32 {
    ((c.max_hp() * second_wind_heal_percent_for_rank(c.second_wind_rank)) / 100).max(1)
}
