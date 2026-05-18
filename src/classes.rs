use crate::*;

#[allow(dead_code)]
pub(crate) const ROGUE_MAX_ENERGY: u32 = 100;
#[allow(dead_code)]
pub(crate) const ROGUE_MAX_COMBO_POINTS: u32 = 5;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum CharacterClass {
    Warrior,
    Rogue,
    Sorceress,
}

impl CharacterClass {
    pub(crate) fn name(self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Rogue => "Rogue",
            CharacterClass::Sorceress => "Sorceress",
        }
    }

    pub(crate) fn from_save_name(name: &str) -> Self {
        match name {
            "Warrior" | "Ironbound" => CharacterClass::Warrior,
            "Rogue" => CharacterClass::Rogue,
            "Sorceress" => CharacterClass::Sorceress,
            _ => CharacterClass::Warrior,
        }
    }
}

pub(crate) fn default_character_class() -> CharacterClass {
    CharacterClass::Warrior
}

pub(crate) fn deserialize_character_class<'de, D>(
    deserializer: D,
) -> std::result::Result<CharacterClass, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let class_name = String::deserialize(deserializer)?;
    Ok(CharacterClass::from_save_name(&class_name))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WarriorState {
    #[serde(default = "default_skill_rank")]
    pub(crate) cleave_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) shield_bash_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) battle_cry_rank: u32,
    #[serde(default = "default_locked_skill_rank")]
    pub(crate) deep_cut_rank: u32,
    #[serde(default = "default_locked_skill_rank")]
    pub(crate) iron_guard_rank: u32,
    #[serde(default = "default_locked_skill_rank")]
    pub(crate) second_wind_rank: u32,
    #[serde(default)]
    pub(crate) cleave_cooldown: u32,
    #[serde(default)]
    pub(crate) shield_bash_cooldown: u32,
    #[serde(default)]
    pub(crate) battle_cry_cooldown: u32,
    #[serde(default, alias = "battle_cry_turns")]
    pub(crate) battle_cry_charges: u32,
    #[serde(default)]
    pub(crate) cleave_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) shield_bash_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) battle_cry_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) deep_cut_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) iron_guard_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) second_wind_mastery: Option<SkillMastery>,
    #[serde(default)]
    pub(crate) second_wind_shield: u32,
}

impl Default for WarriorState {
    fn default() -> Self {
        Self {
            cleave_rank: 1,
            shield_bash_rank: 1,
            battle_cry_rank: 1,
            deep_cut_rank: 0,
            iron_guard_rank: 0,
            second_wind_rank: 0,
            cleave_cooldown: 0,
            shield_bash_cooldown: 0,
            battle_cry_cooldown: 0,
            battle_cry_charges: 0,
            cleave_mastery: None,
            shield_bash_mastery: None,
            battle_cry_mastery: None,
            deep_cut_mastery: None,
            iron_guard_mastery: None,
            second_wind_mastery: None,
            second_wind_shield: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RogueState {
    #[serde(default = "default_rogue_energy")]
    pub(crate) energy: u32,
    #[serde(default)]
    pub(crate) combo_points: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) backstab_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) venom_edge_rank: u32,
    #[serde(default = "default_locked_skill_rank")]
    pub(crate) eviscerate_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) smoke_step_rank: u32,
    #[serde(default)]
    pub(crate) rupture_rank: u32,
    #[serde(default)]
    pub(crate) slip_away_rank: u32,
    #[serde(default)]
    pub(crate) smoke_step_cooldown: u32,
    #[serde(default)]
    pub(crate) smoke_protection_turns: u32,
    #[serde(default)]
    pub(crate) empowered_backstab_turns: u32,
    #[serde(default)]
    pub(crate) smoke_step_pending: bool,
}

impl Default for RogueState {
    fn default() -> Self {
        Self {
            energy: ROGUE_MAX_ENERGY,
            combo_points: 0,
            backstab_rank: 1,
            venom_edge_rank: 1,
            eviscerate_rank: 0,
            smoke_step_rank: 1,
            rupture_rank: 0,
            slip_away_rank: 0,
            smoke_step_cooldown: 0,
            smoke_protection_turns: 0,
            empowered_backstab_turns: 0,
            smoke_step_pending: false,
        }
    }
}

fn default_rogue_energy() -> u32 {
    ROGUE_MAX_ENERGY
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SorceressState {
    #[serde(default = "default_skill_rank")]
    pub(crate) firebolt_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) frost_ring_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) chain_spark_rank: u32,
    #[serde(default = "default_locked_skill_rank")]
    pub(crate) kindle_rank: u32,
    #[serde(default = "default_skill_rank")]
    pub(crate) mana_shield_rank: u32,
    #[serde(default = "default_locked_skill_rank")]
    pub(crate) static_charge_rank: u32,
    #[serde(default)]
    pub(crate) frost_ring_cooldown: u32,
    #[serde(default)]
    pub(crate) chain_spark_cooldown: u32,
    #[serde(default)]
    pub(crate) mana_shield_active: bool,
}

impl Default for SorceressState {
    fn default() -> Self {
        Self {
            firebolt_rank: 1,
            frost_ring_rank: 1,
            chain_spark_rank: 1,
            kindle_rank: 0,
            mana_shield_rank: 1,
            static_charge_rank: 0,
            frost_ring_cooldown: 0,
            chain_spark_cooldown: 0,
            mana_shield_active: false,
        }
    }
}

impl Character {
    #[allow(dead_code)]
    pub(crate) fn resource_label(&self) -> &'static str {
        match self.class {
            CharacterClass::Warrior | CharacterClass::Sorceress => "Mana",
            CharacterClass::Rogue => "Energy",
        }
    }

    #[allow(dead_code)]
    pub(crate) fn current_resource(&self) -> u32 {
        match self.class {
            CharacterClass::Warrior | CharacterClass::Sorceress => self.mana,
            CharacterClass::Rogue => self.rogue.energy,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn max_resource(&self) -> u32 {
        match self.class {
            CharacterClass::Warrior | CharacterClass::Sorceress => self.max_mana(),
            CharacterClass::Rogue => ROGUE_MAX_ENERGY,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn spend_rogue_energy(&mut self, amount: u32) -> bool {
        if self.rogue.energy < amount {
            false
        } else {
            self.rogue.energy -= amount;
            true
        }
    }

    pub(crate) fn restore_rogue_energy(&mut self, amount: u32) {
        self.rogue.energy = self
            .rogue
            .energy
            .saturating_add(amount)
            .min(ROGUE_MAX_ENERGY);
    }

    pub(crate) fn restore_class_resource_full(&mut self) {
        match self.class {
            CharacterClass::Warrior | CharacterClass::Sorceress => self.mana = self.max_mana(),
            CharacterClass::Rogue => self.rogue.energy = ROGUE_MAX_ENERGY,
        }
    }
}
