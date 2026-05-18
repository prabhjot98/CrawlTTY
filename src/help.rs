use crate::*;
use ratatui::{
    prelude::*,
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HelpTopic {
    pub(crate) keyword: &'static str,
    pub(crate) details: &'static str,
}

#[allow(dead_code)]
pub(crate) fn help_topics() -> &'static [HelpTopic] {
    HELP_TOPICS
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HelpScreenState {
    query: String,
    selected: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HelpScreenCommand {
    Stay,
    Close,
}

impl HelpScreenState {
    pub(crate) fn new() -> Self {
        Self {
            query: String::new(),
            selected: 0,
        }
    }

    pub(crate) fn query(&self) -> &str {
        &self.query
    }

    pub(crate) fn handle_key(&mut self, key: char) -> HelpScreenCommand {
        match key {
            '\u{1b}' => HelpScreenCommand::Close,
            KEY_ARROW_UP => {
                self.selected = self.selected.saturating_sub(1);
                HelpScreenCommand::Stay
            }
            KEY_ARROW_DOWN => {
                let count = self.filtered_count();
                if self.selected + 1 < count {
                    self.selected += 1;
                }
                HelpScreenCommand::Stay
            }
            '\u{8}' | '\u{7f}' => {
                self.query.pop();
                self.selected = 0;
                HelpScreenCommand::Stay
            }
            '\n' | '\t' => HelpScreenCommand::Stay,
            typed if !typed.is_control() => {
                self.query.push(typed);
                self.selected = 0;
                HelpScreenCommand::Stay
            }
            _ => HelpScreenCommand::Stay,
        }
    }

    pub(crate) fn filtered_topics(&self) -> Vec<&'static HelpTopic> {
        let query = self.query.to_ascii_lowercase();
        HELP_TOPICS
            .iter()
            .filter(|topic| query.is_empty() || topic.keyword.to_ascii_lowercase().contains(&query))
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn selected_topic(&self) -> Option<&'static HelpTopic> {
        let selected = self.selected;
        self.filtered_topics().into_iter().nth(selected)
    }

    fn filtered_count(&self) -> usize {
        self.filtered_topics().len()
    }

    fn selected_for_count(&self, count: usize) -> Option<usize> {
        if count == 0 {
            None
        } else {
            Some(self.selected.min(count - 1))
        }
    }
}

pub(crate) fn help_screen(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    let mut state = HelpScreenState::new();
    loop {
        terminal
            .draw(|frame| render_help_screen(frame, &state))
            .context("failed to draw help screen")?;
        let key = match read_ui_input_raw_arrows_timed(CURSOR_PULSE_INTERVAL)? {
            UiInput::Key(key) => key,
            UiInput::Redraw => continue,
            UiInput::Tick => {
                toggle_cursor_pulse_frame();
                continue;
            }
        };
        if state.handle_key(key) == HelpScreenCommand::Close {
            break;
        }
    }
    Ok(())
}

pub(crate) fn render_help_screen(frame: &mut Frame, state: &HelpScreenState) {
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .split(frame.area());

    let query_text = if state.query().is_empty() {
        "Search: ".to_string()
    } else {
        format!("Search: {}", state.query())
    };
    frame.render_widget(
        Paragraph::new(query_text)
            .block(gothic_block("Help"))
            .style(body_style()),
        layout[0],
    );

    let body = Layout::horizontal([Constraint::Percentage(34), Constraint::Percentage(66)])
        .split(layout[1]);
    let filtered = state.filtered_topics();
    let selected = state.selected_for_count(filtered.len());
    render_help_keywords(frame, body[0], &filtered, selected);
    render_help_details(frame, body[1], filtered.get(selected.unwrap_or(0)).copied());

    render_commands_footer(
        frame,
        layout[2],
        "Type=search  Backspace=delete  Up/Down=select  Esc=back",
    );
}

fn render_help_keywords(
    frame: &mut Frame,
    area: Rect,
    topics: &[&'static HelpTopic],
    selected: Option<usize>,
) {
    let items: Vec<ListItem<'static>> = if topics.is_empty() {
        vec![ListItem::new(Line::styled(
            "No matching keywords",
            muted_style(),
        ))]
    } else {
        topics
            .iter()
            .map(|topic| ListItem::new(Line::from(topic.keyword)))
            .collect()
    };
    let mut state = ListState::default().with_selected(selected);
    let list = List::new(items)
        .block(gothic_block("Keywords"))
        .highlight_style(selected_cursor_style())
        .highlight_symbol("› ");
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_help_details(frame: &mut Frame, area: Rect, topic: Option<&'static HelpTopic>) {
    let lines = match topic {
        Some(topic) => vec![
            Line::styled(topic.keyword, title_style()),
            Line::from(""),
            Line::from(topic.details),
        ],
        None => vec![
            Line::styled("No keyword selected", title_style()),
            Line::from(""),
            Line::styled(
                "Change the search text to find a help topic.",
                muted_style(),
            ),
        ],
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(gothic_block("Details"))
            .wrap(Wrap { trim: false }),
        area,
    );
}

const HELP_TOPICS: &[HelpTopic] = &[
    HelpTopic {
        keyword: "1-4 Hotkeys",
        details: "Number keys activate class skills in dungeons. Warriors use 1-3, while Rogues and Sorceresses also use 4.",
    },
    HelpTopic {
        keyword: "Act I",
        details: "The first act takes place in the Hollow Crypts and ends with the Bellkeeper on floor 10.",
    },
    HelpTopic {
        keyword: "Act II",
        details: "The second act crosses the Glass Wastes and ends with the Glass Tyrant on floor 18.",
    },
    HelpTopic {
        keyword: "Adrenal Surge",
        details: "Warrior mastery that can reward aggressive Battle Cry play after fresh kills.",
    },
    HelpTopic {
        keyword: "Amethyst",
        details: "Gem that adds Intelligence when socketed into equipment.",
    },
    HelpTopic {
        keyword: "Amulet",
        details: "Accessory equipment slot that can carry defensive stats and sockets.",
    },
    HelpTopic {
        keyword: "Appraiser",
        details: "Town project group that improves merchant sell prices after Hire Appraiser is funded.",
    },
    HelpTopic {
        keyword: "Arcane Recovery",
        details: "Sorceress sustain that restores 10% maximum mana, minimum 4 mana, whenever a Sorceress kill resolves.",
    },
    HelpTopic {
        keyword: "Armor",
        details: "Defensive stat that reduces incoming physical damage after an attack hits.",
    },
    HelpTopic {
        keyword: "Armor Shards",
        details: "Crafting material from salvaged armor pieces; used for shard-only gear upgrades.",
    },
    HelpTopic {
        keyword: "Armored Elite",
        details: "Elite modifier marking a tougher monster with better defenses.",
    },
    HelpTopic {
        keyword: "Attributes",
        details: "Strength, Dexterity, and Intelligence. Spend unspent attribute points from the attributes screen.",
    },
    HelpTopic {
        keyword: "Backstab",
        details: "Rogue dagger skill that costs Energy, builds 1 combo point, and is stronger after movement, smoke, or poison setup.",
    },
    HelpTopic {
        keyword: "Bag",
        details: "Your carried inventory grid. Quartermaster town projects expand it from 4 x 4 up to 8 x 8.",
    },
    HelpTopic {
        keyword: "Battle Cry",
        details: "Warrior skill that spends mana to grant temporary damage and defensive momentum for upcoming attacks.",
    },
    HelpTopic {
        keyword: "Bell Wave",
        details: "Bellkeeper boss effect shown as marked wave tiles before damage resolves.",
    },
    HelpTopic {
        keyword: "Bellkeeper",
        details: "Act I boss in the Hollow Crypts. Defeating it unlocks Act II quest progress.",
    },
    HelpTopic {
        keyword: "Belt",
        details: "Equipment slot that can add stats, sockets, and defensive bonuses.",
    },
    HelpTopic {
        keyword: "Blacksmith",
        details: "Town service for salvaging gear and upgrading equipment after forge projects are funded.",
    },
    HelpTopic {
        keyword: "Bleed",
        details: "Short name for Bleeding, a physical damage-over-time effect.",
    },
    HelpTopic {
        keyword: "Bleeding",
        details: "Physical damage over time, often caused by Warrior Deep Cut and bleed-focused masteries.",
    },
    HelpTopic {
        keyword: "Blood Arc",
        details: "Warrior Cleave mastery tied to bleed-oriented area attacks.",
    },
    HelpTopic {
        keyword: "Bloodletting",
        details: "Warrior bleed mastery that improves damage-over-time pressure.",
    },
    HelpTopic {
        keyword: "Bloodstone",
        details: "Gem that adds weapon damage when socketed.",
    },
    HelpTopic {
        keyword: "Boneguard",
        details: "Durable enemy that can guard at range and uses armor to blunt attacks.",
    },
    HelpTopic {
        keyword: "Boots",
        details: "Equipment slot often associated with dodge, speed, armor, and sockets.",
    },
    HelpTopic {
        keyword: "Boss",
        details: "Major encounter with special rules and guaranteed important progression rewards.",
    },
    HelpTopic {
        keyword: "Bulwark",
        details: "Warrior Iron Guard mastery that grants extra armor while badly wounded.",
    },
    HelpTopic {
        keyword: "Burning",
        details: "Fire damage over time. Sorceress fire magic and Burning elites can apply it.",
    },
    HelpTopic {
        keyword: "Chain Spark",
        details: "Sorceress shock spell that can jump to nearby reachable enemies after the first hit.",
    },
    HelpTopic {
        keyword: "Character Screen",
        details: "Town attributes view opened with a/A, used to spend unspent attribute points.",
    },
    HelpTopic {
        keyword: "Chests",
        details: "Dungeon treasure containers. Open them by moving onto them after the floor is safe enough to reach.",
    },
    HelpTopic {
        keyword: "Chipped Gems",
        details: "Lowest gem tier. Useful early socket filler with smaller bonuses than Flawed or Pristine gems.",
    },
    HelpTopic {
        keyword: "Citrine",
        details: "Gem that adds Speed when socketed into equipment.",
    },
    HelpTopic {
        keyword: "Cleave",
        details: "Warrior skill that spends mana to strike multiple adjacent enemies with weapon damage.",
    },
    HelpTopic {
        keyword: "Combo Points",
        details: "Rogue class resource built by Backstab and Venom Edge, then spent by Eviscerate for burst damage.",
    },
    HelpTopic {
        keyword: "Common",
        details: "Baseline item rarity. Common items have normal names and lower overall stat potential.",
    },
    HelpTopic {
        keyword: "Crushing Bash",
        details: "Warrior Shield Bash mastery focused on heavier control and impact.",
    },
    HelpTopic {
        keyword: "Critical Chance",
        details: "Percent chance for a damaging hit to critically strike. Weapon type and Topaz sockets can raise it.",
    },
    HelpTopic {
        keyword: "Cultist",
        details: "Enemy caster that can use shadow bolt when it has clear cardinal line of sight.",
    },
    HelpTopic {
        keyword: "Cursed",
        details: "A dark status concept used by boss and UI language for harmful supernatural pressure.",
    },
    HelpTopic {
        keyword: "Dazing Bash",
        details: "Warrior Shield Bash mastery that extends or improves stun control.",
    },
    HelpTopic {
        keyword: "Death Mode",
        details: "Character creation choice between Softcore and Hardcore consequences.",
    },
    HelpTopic {
        keyword: "Deep Cut",
        details: "Warrior passive branch skill that can apply Bleeding to enemies.",
    },
    HelpTopic {
        keyword: "Deep Rucksack",
        details: "Quartermaster town project that expands bag capacity to 8 x 7.",
    },
    HelpTopic {
        keyword: "Dexterity",
        details: "Primary attribute that increases hit rating, helps agile gear requirements, and supports accurate attacks.",
    },
    HelpTopic {
        keyword: "DEX",
        details: "Short label for Dexterity, the attribute behind hit rating and agile requirements.",
    },
    HelpTopic {
        keyword: "Distillery",
        details: "Town service unlocked by project funding; crafts lesser potions from herbs.",
    },
    HelpTopic {
        keyword: "Dodge Rating",
        details: "Defensive rating compared against enemy hit rating. Higher dodge makes attacks less likely to land.",
    },
    HelpTopic {
        keyword: "Dungeon",
        details: "Procedural combat floor containing rooms, monsters, chests, loot, stairs, and bosses.",
    },
    HelpTopic {
        keyword: "Dune Stalker",
        details: "Fast Act II enemy from the Glass Wastes.",
    },
    HelpTopic {
        keyword: "Elite",
        details: "Stronger monster variant with an affix such as Armored, Swift, Vampiric, or Burning.",
    },
    HelpTopic {
        keyword: "Elite Skeleton",
        details: "Act I elite enemy that appears with a random elite modifier.",
    },
    HelpTopic {
        keyword: "Ember Magus",
        details: "Act II enemy associated with dangerous fire pressure.",
    },
    HelpTopic {
        keyword: "Emerald",
        details: "Gem that adds Dexterity when socketed.",
    },
    HelpTopic {
        keyword: "Energy",
        details: "Rogue resource used instead of mana for rogue skills. It regenerates during dungeon turns.",
    },
    HelpTopic {
        keyword: "Equipment",
        details: "Worn items: weapon, armor, offhand, helm, gloves, boots, belt, amulet, and two rings.",
    },
    HelpTopic {
        keyword: "Esc",
        details: "Back out of menus. In dungeons, Escape returns to town only when the floor rules allow it.",
    },
    HelpTopic {
        keyword: "Eviscerate",
        details: "Rogue finisher that spends combo points for heavy physical burst damage.",
    },
    HelpTopic {
        keyword: "Exile's Trunk",
        details: "Final Quartermaster bag project, expanding the bag to 8 x 8.",
    },
    HelpTopic {
        keyword: "Fire Resistance",
        details: "Resistance concept for reducing fire damage; tracked in the broader stat model and design vocabulary.",
    },
    HelpTopic {
        keyword: "Firebolt",
        details: "Sorceress ranged fire spell that scales with Intelligence and can apply Burning.",
    },
    HelpTopic {
        keyword: "Flawed Gems",
        details: "Middle gem tier with stronger bonuses than Chipped gems.",
    },
    HelpTopic {
        keyword: "Focus",
        details: "Sorceress offhand item type that can provide dodge and magical requirements.",
    },
    HelpTopic {
        keyword: "Forge",
        details: "Smith project line that unlocks salvage, shard upgrades, and socket work.",
    },
    HelpTopic {
        keyword: "Fresh Kill",
        details: "Warrior mastery theme that rewards kills during combat momentum.",
    },
    HelpTopic {
        keyword: "Frost Resistance",
        details: "Resistance concept for reducing frost damage in the broader combat vocabulary.",
    },
    HelpTopic {
        keyword: "Frost Ring",
        details: "Sorceress adjacent frost burst that can hit surrounding tiles and apply Frozen.",
    },
    HelpTopic {
        keyword: "Frozen",
        details: "Status effect that causes the affected enemy to skip its next turn in the current implementation.",
    },
    HelpTopic {
        keyword: "Garnet",
        details: "Gem that adds Strength when socketed.",
    },
    HelpTopic {
        keyword: "Gems",
        details: "Socketable items with tiers and kinds. They add stats such as HP, mana, attributes, speed, crit, or gold found.",
    },
    HelpTopic {
        keyword: "Glass Mirage",
        details: "Mirror copy created by the Glass Tyrant during its boss fight.",
    },
    HelpTopic {
        keyword: "Glass Tyrant",
        details: "Final Act II boss on floor 18 of the Glass Wastes.",
    },
    HelpTopic {
        keyword: "Glass Wastes",
        details: "Act II region with glass-themed enemies and the Glass Tyrant finale.",
    },
    HelpTopic {
        keyword: "Glass Wraith",
        details: "Act II enemy that can appear as a mirrored elite variant.",
    },
    HelpTopic {
        keyword: "Gloves",
        details: "Equipment slot that can hold armor, sockets, and other gear bonuses.",
    },
    HelpTopic {
        keyword: "Gold",
        details: "Main currency used for merchant purchases and town project funding. Monsters, chests, and loot rewards can grant it.",
    },
    HelpTopic {
        keyword: "Gold Found",
        details: "Percent bonus from Opal sockets that increases variable gold drops.",
    },
    HelpTopic {
        keyword: "Grim Recovery",
        details: "Warrior recovery mastery theme tied to survival after dangerous fights.",
    },
    HelpTopic {
        keyword: "Ground Loot",
        details: "Items lying on the dungeon floor, shown with the loot glyph and picked up with g/G or by walking over a single item.",
    },
    HelpTopic {
        keyword: "Guarding",
        details: "Enemy defensive state used by Boneguards to improve toughness at range.",
    },
    HelpTopic {
        keyword: "Hardcore",
        details: "Death mode where character death deletes the save.",
    },
    HelpTopic {
        keyword: "Health",
        details: "Hit points. If health reaches zero, Softcore returns to town and Hardcore deletes the save.",
    },
    HelpTopic {
        keyword: "Health Potion",
        details: "Consumable that restores 15% HP, capped at maximum health.",
    },
    HelpTopic {
        keyword: "Help",
        details: "Press H or h from town or dungeon to open this searchable keyword glossary.",
    },
    HelpTopic {
        keyword: "Helm",
        details: "Head equipment slot that can add armor, sockets, and other bonuses.",
    },
    HelpTopic {
        keyword: "Hemorrhage",
        details: "Warrior mastery associated with stronger bleed interactions.",
    },
    HelpTopic {
        keyword: "Herb Garden",
        details: "Town project that grows 1-3 herbs after each newly completed dungeon floor.",
    },
    HelpTopic {
        keyword: "Herbs",
        details: "Crafting resource grown by the Herb Garden and spent at the Distillery for potions.",
    },
    HelpTopic {
        keyword: "Hit Chance",
        details: "Calculated from attacker hit rating versus defender dodge rating, clamped between 20% and 95%.",
    },
    HelpTopic {
        keyword: "Hit Rating",
        details: "Accuracy rating. Player hit rating mainly comes from Dexterity, gear, and Quartz sockets.",
    },
    HelpTopic {
        keyword: "Hollow Crypts",
        details: "Act I dungeon region ending with the Bellkeeper.",
    },
    HelpTopic {
        keyword: "HP",
        details: "Short label for Health points.",
    },
    HelpTopic {
        keyword: "INT",
        details: "Short label for Intelligence, the attribute behind mana and magical requirements.",
    },
    HelpTopic {
        keyword: "Intelligence",
        details: "Primary attribute that increases mana, supports magical gear, and scales Sorceress spell damage.",
    },
    HelpTopic {
        keyword: "Inventory",
        details: "Bag screen opened with i/I. Manage carried items, equipment comparisons, sorting, and dungeon drops.",
    },
    HelpTopic {
        keyword: "Iron Guard",
        details: "Warrior defensive passive that adds armor and unlocks defensive masteries.",
    },
    HelpTopic {
        keyword: "Item Level",
        details: "Loot level that generally raises requirements and stat ranges on generated gear.",
    },
    HelpTopic {
        keyword: "Jade",
        details: "Gem that adds dodge rating when socketed.",
    },
    HelpTopic {
        keyword: "Kindle",
        details: "Sorceress fire passive that improves Firebolt's Burning plan after it is unlocked.",
    },
    HelpTopic {
        keyword: "Level",
        details: "Character progression tier. Gaining levels grants attribute and skill points.",
    },
    HelpTopic {
        keyword: "Lesser Health Potion",
        details: "Town and loot consumable that restores 15% of maximum HP.",
    },
    HelpTopic {
        keyword: "Lesser Mana Potion",
        details: "Town and loot consumable that restores 35% mana for mana-using classes. Rogues do not use mana potions.",
    },
    HelpTopic {
        keyword: "Long Bash",
        details: "Warrior Shield Bash mastery that can extend Shield Bash to a clear cardinal line.",
    },
    HelpTopic {
        keyword: "Loot",
        details: "Rewards from monsters, chests, bosses, and ground items. Full bags leave eligible loot on the ground.",
    },
    HelpTopic {
        keyword: "Magic",
        details: "Blue item rarity above Common and below Rare.",
    },
    HelpTopic {
        keyword: "Mana",
        details: "Resource used by Warriors and Sorceresses to cast skills. Intelligence increases maximum mana.",
    },
    HelpTopic {
        keyword: "Mana Potion",
        details: "Consumable that restores 35% mana for Warrior and Sorceress characters.",
    },
    HelpTopic {
        keyword: "Mana Shield",
        details: "Sorceress toggle that absorbs part of incoming damage by spending mana.",
    },
    HelpTopic {
        keyword: "Merchant",
        details: "Town vendor for buying supplies and selling carried items.",
    },
    HelpTopic {
        keyword: "Mirrored Elite",
        details: "Act II elite naming style for dangerous Glass Wraith variants.",
    },
    HelpTopic {
        keyword: "Movement",
        details: "Use WASD in dungeons to move cardinally or attack an adjacent monster in that direction.",
    },
    HelpTopic {
        keyword: "Obsidian Guard",
        details: "Armored Act II enemy with high toughness.",
    },
    HelpTopic {
        keyword: "Offhand",
        details: "Shield/focus slot. Warriors use shields, Rogues can use bucklers, and Sorceresses can use focuses.",
    },
    HelpTopic {
        keyword: "Oilcloth Satchel",
        details: "Quartermaster town project that expands bag capacity to 6 x 5.",
    },
    HelpTopic {
        keyword: "Onyx",
        details: "Gem that adds armor when socketed.",
    },
    HelpTopic {
        keyword: "Opal",
        details: "Gem that increases gold found from variable gold drops.",
    },
    HelpTopic {
        keyword: "Open Wound",
        details: "Warrior Deep Cut mastery that improves bleed uptime or pressure.",
    },
    HelpTopic {
        keyword: "Pack Hooks",
        details: "Quartermaster town project that expands bag capacity to 5 x 5.",
    },
    HelpTopic {
        keyword: "Poison Resistance",
        details: "Resistance concept for reducing poison pressure in the broader combat vocabulary.",
    },
    HelpTopic {
        keyword: "Poisoned",
        details: "Damage over time applied by Rogue Venom Edge and poison effects; poison setup empowers Rogue payoffs.",
    },
    HelpTopic {
        keyword: "Potion",
        details: "Consumable used from inventory or with p/P in dungeons. Routine potion use spends a dungeon turn.",
    },
    HelpTopic {
        keyword: "Pristine Gems",
        details: "Highest gem tier with the strongest socket bonuses.",
    },
    HelpTopic {
        keyword: "Quest",
        details: "Warden Mara tracks act goals. Complete boss objectives and return to town to claim rewards.",
    },
    HelpTopic {
        keyword: "Quartermaster",
        details: "Town project group that expands your bag through a chain of upgrades.",
    },
    HelpTopic {
        keyword: "Quartermaster Ledger",
        details: "Quartermaster town project that expands bag capacity to 6 x 6.",
    },
    HelpTopic {
        keyword: "Quartz",
        details: "Gem that adds hit rating when socketed.",
    },
    HelpTopic {
        keyword: "Radiant",
        details: "Holy/radiant damage type from the broader design vocabulary.",
    },
    HelpTopic {
        keyword: "Rallying Cry",
        details: "Warrior Battle Cry mastery that leans into recovery and sustained pressure.",
    },
    HelpTopic {
        keyword: "Rare",
        details: "Gold item rarity with stronger loot potential than Common or Magic.",
    },
    HelpTopic {
        keyword: "Rat",
        details: "Small early enemy in the Hollow Crypts.",
    },
    HelpTopic {
        keyword: "Reaping Cleave",
        details: "Warrior Cleave mastery that improves multi-target melee clearing.",
    },
    HelpTopic {
        keyword: "Rebuild Forge",
        details: "Smith town project that unlocks salvage and shard-only gear upgrades.",
    },
    HelpTopic {
        keyword: "Reinforced Anvil",
        details: "Smith town project that makes salvaged gear yield one extra shard.",
    },
    HelpTopic {
        keyword: "Reinforced Pack",
        details: "Quartermaster town project that expands bag capacity to 7 x 6.",
    },
    HelpTopic {
        keyword: "Resistances",
        details: "Fire, frost, shock, and poison defense concepts in the combat stat vocabulary.",
    },
    HelpTopic {
        keyword: "Ring",
        details: "Accessory equipment slot. You can wear two rings.",
    },
    HelpTopic {
        keyword: "Rogue",
        details: "Playable class using Dexterity, Energy, combo points, daggers, poison, finishers, and smoke movement.",
    },
    HelpTopic {
        keyword: "Ruby",
        details: "Gem that adds maximum HP when socketed.",
    },
    HelpTopic {
        keyword: "Rupture",
        details: "Rogue Venom branch passive that extends Venom Edge poison duration after it is unlocked.",
    },
    HelpTopic {
        keyword: "Salvage",
        details: "Blacksmith action that breaks gear into weapon, armor, or shield shards after the forge is rebuilt.",
    },
    HelpTopic {
        keyword: "Sapphire",
        details: "Gem that adds maximum mana when socketed.",
    },
    HelpTopic {
        keyword: "Save",
        details: "The game autosaves after many actions and can be saved and quit from town with q/Q.",
    },
    HelpTopic {
        keyword: "Second Wind",
        details: "Warrior Warcry branch passive that adds healing or recovery to Battle Cry play.",
    },
    HelpTopic {
        keyword: "Shadow",
        details: "Damage type used by cultist-style magic and the broader dark combat vocabulary.",
    },
    HelpTopic {
        keyword: "Shield",
        details: "Offhand defensive item for Warriors, with armor, dodge, sockets, and shield shard upgrades.",
    },
    HelpTopic {
        keyword: "Shield Bash",
        details: "Warrior skill that spends mana to hit, stun, and sometimes push or reach enemies depending on mastery.",
    },
    HelpTopic {
        keyword: "Shield Discipline",
        details: "Warrior Iron Guard mastery that improves defensive dodge behavior.",
    },
    HelpTopic {
        keyword: "Shield Shards",
        details: "Crafting material from salvaged shields and offhands; used for shield/offhand upgrades.",
    },
    HelpTopic {
        keyword: "Shock Resistance",
        details: "Resistance concept for reducing shock damage in the broader combat vocabulary.",
    },
    HelpTopic {
        keyword: "Shocked",
        details: "Status that increases the next damaging hit against the target, then is consumed.",
    },
    HelpTopic {
        keyword: "Skeleton",
        details: "Common Hollow Crypts enemy and Bellkeeper summon type.",
    },
    HelpTopic {
        keyword: "Skill Points",
        details: "Progression currency spent in the skill tree to raise active skills and unlock passives.",
    },
    HelpTopic {
        keyword: "Skill Tree",
        details: "Class-specific upgrade screen opened with k/K in town.",
    },
    HelpTopic {
        keyword: "Slip Away",
        details: "Rogue Smoke branch passive that adds smoke-protection dodge after it is unlocked.",
    },
    HelpTopic {
        keyword: "Smoke Protection",
        details: "Temporary Rogue defensive dodge bonus granted by Smoke Step and Slip Away.",
    },
    HelpTopic {
        keyword: "Smoke Step",
        details: "Rogue dash skill that spends Energy, moves 1-2 cardinal tiles, grants smoke protection, and empowers Backstab.",
    },
    HelpTopic {
        keyword: "Socket Bench",
        details: "Smith town project that unlocks free gem insertion, removal, and replacement.",
    },
    HelpTopic {
        keyword: "Sockets",
        details: "Equipment holes that hold gems. Socketed gems add their bonuses while the item is equipped.",
    },
    HelpTopic {
        keyword: "Softcore",
        details: "Death mode where dying clears the dungeon and returns the character to town instead of deleting the save.",
    },
    HelpTopic {
        keyword: "Sorceress",
        details: "Playable caster class using Intelligence, mana, fire, frost, shock spells, and Mana Shield.",
    },
    HelpTopic {
        keyword: "Speed",
        details: "Turn-energy stat. Higher speed lets actors gain action energy faster.",
    },
    HelpTopic {
        keyword: "Spiked Guard",
        details: "Warrior Iron Guard mastery that damages adjacent melee attackers after they hit you.",
    },
    HelpTopic {
        keyword: "Stash",
        details: "Town storage grid for moving items between your bag and long-term storage.",
    },
    HelpTopic {
        keyword: "Static Charge",
        details: "Sorceress shock passive that improves Shocked setup after it is unlocked.",
    },
    HelpTopic {
        keyword: "Stitched Pockets",
        details: "Quartermaster town project that expands bag capacity to 7 x 7.",
    },
    HelpTopic {
        keyword: "Storehouse Shelves",
        details: "First Quartermaster bag project, expanding bag capacity to 5 x 4.",
    },
    HelpTopic {
        keyword: "STR",
        details: "Short label for Strength, the attribute behind HP and heavy gear requirements.",
    },
    HelpTopic {
        keyword: "Strength",
        details: "Primary attribute that increases maximum health and helps equip heavy gear.",
    },
    HelpTopic {
        keyword: "Stunned",
        details: "Control status that prevents the affected enemy from acting briefly.",
    },
    HelpTopic {
        keyword: "Summoned Skeleton",
        details: "Skeleton created by the Bellkeeper during its boss encounter.",
    },
    HelpTopic {
        keyword: "Sundering Cleave",
        details: "Warrior Cleave mastery focused on cutting through tougher enemies.",
    },
    HelpTopic {
        keyword: "Swift Elite",
        details: "Elite modifier that increases an enemy's speed, accuracy, and dodge.",
    },
    HelpTopic {
        keyword: "Terrifying Cry",
        details: "Warrior Battle Cry mastery that emphasizes enemy disruption.",
    },
    HelpTopic {
        keyword: "Topaz",
        details: "Gem that adds critical chance when socketed.",
    },
    HelpTopic {
        keyword: "Town",
        details: "Safe hub for merchants, projects, stash, quests, healing, and dungeon entry.",
    },
    HelpTopic {
        keyword: "Town Projects",
        details: "Gold-funded upgrades that unlock salvage, sockets, bag capacity, appraising, herbs, and distilling.",
    },
    HelpTopic {
        keyword: "Turn Energy",
        details: "Actor scheduling resource gained from Speed; when enough energy is available, an actor can act.",
    },
    HelpTopic {
        keyword: "Unspent Attributes",
        details: "Attribute points waiting to be spent on Strength, Dexterity, or Intelligence.",
    },
    HelpTopic {
        keyword: "Upgrade Level",
        details: "Gear improvement rank increased at the blacksmith with the matching shard type.",
    },
    HelpTopic {
        keyword: "Vampiric Elite",
        details: "Elite modifier that heals the monster after it deals damage.",
    },
    HelpTopic {
        keyword: "Venom Edge",
        details: "Rogue skill that spends Energy, poisons the target, and builds a combo point on hit.",
    },
    HelpTopic {
        keyword: "Vulnerable",
        details: "Debuff field used by combat effects to make a target take more punishment.",
    },
    HelpTopic {
        keyword: "Wand",
        details: "Sorceress weapon type for spell damage and Intelligence requirements.",
    },
    HelpTopic {
        keyword: "Warpath Cry",
        details: "Warrior Battle Cry mastery that improves offensive momentum.",
    },
    HelpTopic {
        keyword: "Warrior",
        details: "Playable melee class using Strength, armor, shields, Cleave, Shield Bash, Battle Cry, and bleed passives.",
    },
    HelpTopic {
        keyword: "Weapon",
        details: "Mainhand equipment that defines damage range, crit chance, speed modifier, and class weapon style.",
    },
    HelpTopic {
        keyword: "Weapon Shards",
        details: "Crafting material from salvaged weapons; used for weapon upgrades.",
    },
    HelpTopic {
        keyword: "XP",
        details: "Experience points from kills and rewards. Enough XP increases level and grants progression points.",
    },
];
