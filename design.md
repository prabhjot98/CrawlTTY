# Terminal Action RPG Design

## Working Title

**Ashen Depths**

A terminal-based action RPG inspired by classic loot-driven dungeon crawlers. The goal is to capture the feeling of fast combat, character builds, dangerous dungeons, randomized loot, and long-term progression while using original lore, names, enemies, classes, and mechanics.

## Core Fantasy

The player is an exile descending through cursed ruins, corrupted wilderness, and infernal depths to stop an ancient power from awakening. The game should feel tense, replayable, and rewarding, with constant decisions around positioning, resources, skills, and loot.

## Design Pillars

1. **Fast tactical combat**: Every turn or tick should matter.
2. **Meaningful loot**: Gear changes how the character plays.
3. **Distinct builds**: Classes should support multiple viable play styles.
4. **Procedural replayability**: Maps, enemies, loot, and events vary each run.
5. **Readable terminal presentation**: The game should be clear and satisfying in text mode.

## Target Platform

- Terminal / command line
- Keyboard controls
- Single-player
- Save/load support
- ASCII-only display for maximum terminal compatibility

## Camera and View

Recommended view: **top-down grid-based map**.

Example symbols:

```text
@  Player
#  Wall
.  Floor
+  Door
>  Stairs down
<  Stairs up
!  Potion
$  Gold
?  Scroll
)  Weapon
]  Armor
r  Rat / small beast
s  Skeleton
d  Demon
B  Boss
*  Spell effect
```

## Game Loop

1. Player starts in a safe town hub with 2 lesser health potions and class-appropriate starting gear.
2. Player selects class, manages inventory, buys supplies, and accepts quests.
3. Player enters a procedurally generated zone.
4. Player explores, fights monsters, collects loot, and gains experience.
5. Player finds exits, mini-bosses, events, shrines, and treasure rooms.
6. Player returns to hub or descends deeper.
7. Player upgrades gear, skills, and stats.
8. Player progresses through acts until final boss.

## Controls

Suggested default controls:

```text
WASD / Arrow keys  Move cardinally only, no diagonal movement
Q                 Use primary skill
E                 Use secondary skill
R                 Use potion
I                 Inventory
C                 Character screen
K                 Skill tree
M                 Map / minimap
G                 Pick up item
Space             Wait / basic attack nearest enemy
Esc               Pause / menu
1-5               Skill hotkeys
```

## Combat Model

### Recommended Approach

Use a **turn-based tactical** system.

Core turn structure:

- The game uses a simple speed/energy system.
- Each actor gains energy based on its speed.
- When an actor has enough energy, it can take one action and spends that energy.
- Most actors act about once per round, but fast actors may occasionally act more often and slow actors may act less often.
- Player actions include moving, attacking, casting, using an item, waiting, or interacting.
- Environmental effects update after actions, such as poison clouds, fire patches, traps, and cooldowns.
- Repeat until the player leaves the floor, dies, or clears the encounter.

Turn-based combat is the chosen design because it works well in a terminal, keeps the UI readable, and lets players make tactical decisions without reaction-time pressure.

### Core Combat Stats

Primary attributes for the beginning version:

- Strength: increases health and helps equip heavy gear.
- Dexterity: increases hit chance, turn speed, and helps equip agile weapons.
- Intelligence: increases mana and helps equip magical gear.

Each class starts with **10 total primary attribute points**.

Base player stats before attributes:

- Base health: 10
- Base mana: 10
- Base hit rating: 10
- Base dodge rating: 10
- Base speed: 10

Ironbound starting attributes:

- Strength: 6
- Dexterity: 3
- Intelligence: 1

Derived combat stats:

- Health: each Strength gives +5 health.
- Mana: each Intelligence gives +5 mana.
- Hit rating: each Dexterity gives +5 hit rating.
- Dodge rating: comes from Dexterity, equipment, and enemy stats.
- Speed: each Dexterity gives +5 speed.
- Turn energy: gained each tick based on speed.
- Armor: mainly from equipment.
- Damage: mainly from equipped weapon and that weapon's attribute scaling.
- Critical chance.
- Fire resistance.
- Frost resistance.
- Shock resistance.
- Poison resistance.

Hit chance should be calculated by comparing the attacker's hit rating against the target's dodge rating.

MVP hit chance formula:

- `hit_chance = attacker_hit_rating / (attacker_hit_rating + target_dodge_rating)`
- Clamp the final result between 20% minimum and 95% maximum.

This is fair, simple, and easy to tune. Equal hit and dodge ratings produce a 50% hit chance before clamping.

### Damage Types

- Physical
- Fire
- Frost
- Shock
- Poison
- Shadow
- Holy / Radiant

### Status Effects

- Burning: damage over time
- Frozen: reduced movement or skipped turn chance
- Shocked: increased damage taken
- Poisoned: damage over time, reduced healing
- Bleeding: physical damage over time
- Cursed: reduced stats or resistance
- Stunned: cannot act briefly

## Character Classes

Use original classes instead of copying existing ones.

### 1. Ironbound

A melee warrior focused on weapons, armor, and survival.

Possible builds:

- Shield tank
- Two-handed berserker
- Bleed-focused duelist

Example skills:

- Cleave: hit multiple adjacent enemies
- Shield Bash: stun and push enemy
- Battle Cry: temporary damage and defense buff
- Iron Skin: reduce incoming damage

### 2. Embercaller

A spellcaster using fire, frost, and shock magic.

Possible builds:

- Fire damage over time
- Frost control
- Shock burst damage

Example skills:

- Firebolt: ranged fire attack
- Frost Ring: slows nearby enemies
- Chain Spark: jumps between enemies
- Mana Shield: damage drains mana first

### 3. Gravewarden

A summoner and curse user who commands spirits and bones.

Possible builds:

- Skeleton army
- Curse support
- Poison and decay caster

Example skills:

- Raise Boneguard: summon minion
- Rot Cloud: poison area
- Frailty Hex: reduce enemy armor
- Soul Drain: damage enemy and heal self

### 4. Shadeblade

A fast rogue using traps, daggers, and mobility.

Possible builds:

- Critical dagger assassin
- Trap master
- Poison archer

Example skills:

- Backstab: high damage from behind or against distracted foes
- Smoke Step: teleport short distance
- Spike Trap: damages enemy that steps on it
- Venom Edge: poison weapon buff

### 5. Wildspeaker

A nature warrior using beasts, vines, storms, and shapeshifting.

Possible builds:

- Beast companion
- Storm caster
- Werebeast melee form

Example skills:

- Summon Wolf: companion fighter
- Thornsnare: root enemies
- Stormcall: lightning area attack
- Primal Form: temporary melee transformation

## Progression

### Levels

- Gain XP from monsters, quests, and events.
- Level XP starts at 40 XP for level 2 and doubles every level after that.
- Example XP requirements: level 2 = 40 XP, level 3 = 80 XP, level 4 = 160 XP, level 5 = 320 XP.
- On level up, gain 3 attribute points and 1 skill point.
- Attribute points improve Strength, Dexterity, or Intelligence.
- Skill points unlock or improve abilities.

### Skill Trees

Each class should have 3 skill trees.

Example Ironbound trees:

- Weapons
- Defense
- Warcries

Example Embercaller trees:

- Flame
- Frost
- Storm

Skills can have:

- Required level
- Prerequisite skill
- Mana cost
- Cooldown
- Scaling values
- Upgrade ranks

### First Skill Tree Complexity

The first implemented class should have a moderately complex skill tree rather than only four isolated abilities. Start with the **Ironbound** and give it three small branches.

#### Ironbound MVP Skill Tree

The first implemented Ironbound skills should be:

- Cleave
- Shield Bash
- Battle Cry
- Deep Cut
- Iron Guard
- Second Wind

The Ironbound uses **Mana** in the MVP for simplicity. A later version may rename the Ironbound resource to Fury while keeping similar mechanics.

Skill rank philosophy:

- Skill ranks should mostly increase damage or effect strength.
- Some skills may gain a special bonus at rank 3 or rank 5.
- Cooldown reduction should be rare, because cooldowns help preserve tactical decision-making.

Weapons branch:

- Cleave: active attack bound to `1`. Costs 5 mana. Cooldown 1 turn. Hits up to 3 cardinally adjacent enemies for 80% weapon damage at rank 1. Each rank adds +10% weapon damage, up to rank 5.
- Deep Cut: passive. Melee hits have a 15% chance to cause bleeding for 3 turns. Bleed deals 2 damage per turn. Each rank increases bleed chance or bleed damage.
- Executioner: bonus damage against low-health enemies. This can be added after the MVP.

Defense branch:

- Shield Bash: active attack bound to `2`. Costs 6 mana. Cooldown 3 turns. Requires a shield. Hits one adjacent enemy for 70% weapon damage at rank 1 and stuns for 1 turn. Each rank adds +10% weapon damage, up to rank 5.
- Iron Guard: passive. Grants +2 armor while using a shield. Each rank adds +1 additional armor.
- Last Stand: temporary defense boost when below 30% health. This can be added after the MVP.

Warcry branch:

- Battle Cry: active buff/debuff bound to `3`. Costs 8 mana. Cooldown 6 turns. Grants 5 empowered attack charges instead of expiring by movement turns. While charges remain, player gains +20% damage at rank 1 and enemies deal -10% damage. Each rank adds +5% player damage, up to rank 5. While Battle Cry has charges, Second Wind restores 10% max health on kill.
- Threatening Shout: reduce enemy damage in a small radius. This can be added after the MVP.
- Second Wind: passive. Killing an enemy restores 10% maximum health if Battle Cry is active. Each rank increases the healing amount; later ranks may also restore a small amount of mana.

Tree rules:

- Player gains 1 skill point per level.
- Each skill can have 3-5 ranks.
- Some skills require 1-2 points in earlier skills.
- The player should be able to specialize in one branch or mix branches.
- The MVP should include enough skills to make leveling choices interesting, even if not every skill has advanced animations or effects at first.

Mastery rules:

- When a skill reaches rank 5, selecting it in the skill tree opens a mastery choice.
- A mastery is free and does not consume a skill point.
- Each rank 5 skill has 3 mutually exclusive mastery paths.
- Picking one mastery permanently locks the other two choices for that skill.

Ironbound mastery paths:

- Cleave: Reaping Cleave hits every adjacent enemy; Sundering Cleave shreds enemy armor; Blood Arc forces bleeding on Cleave hits.
- Shield Bash: Crushing Bash gains damage from shield armor; Long Bash can target enemies up to 2 tiles away; Dazing Bash increases stun duration.
- Battle Cry: Warpath Cry grants +2 charges; Terrifying Cry staggers nearby enemies; Rallying Cry restores health and mana on activation.
- Deep Cut: Hemorrhage increases bleed damage against low-health enemies; Open Wound makes bleeding enemies vulnerable to physical damage; Bloodletting heals the player on bleed kills.
- Iron Guard: Bulwark grants extra armor at low health; Shield Discipline grants dodge; Spiked Guard damages adjacent melee attackers.
- Second Wind: Fresh Kill lets Second Wind work without Battle Cry at reduced strength; Adrenal Surge restores a Battle Cry charge; Grim Recovery turns overhealing into a temporary shield.

## Loot System

### Item Slots

- Weapon
- Off-hand / shield
- Helm
- Chest
- Gloves
- Boots
- Belt
- Amulet
- Ring 1
- Ring 2
- Charm inventory slots

### Item Rarity

MVP item rarities:

1. Common: basic stats
2. Magic: roughly 25% better stats than a common item of the same type, or one simple boost/modifier.
3. Rare: roughly 50% better stats than a common item of the same type, or multiple stronger boosts/modifiers.

Unique items should **not** exist in the MVP.

Later item rarities: 4. Unique: fixed powerful identity and special effects 5. Set: bonuses when combined with matching items 6. Legendary: rare late-game items that alter skills

Unidentified items should not exist in the MVP. Items should reveal their stats immediately when picked up so the first version stays fast and readable.

### MVP Item Modifiers

Magic and rare items should roll from a small, readable modifier list.

Simple stat modifiers:

- +Health
- +Mana
- +Strength
- +Dexterity
- +Intelligence
- +Hit rating
- +Dodge rating
- +Speed
- +Armor

Damage modifiers:

- +Minimum damage
- +Maximum damage
- +Physical damage percentage
- +Bleed chance

Utility modifiers:

- +Gold found
- +Potion healing
- +Potion mana recovery
- +Skill rank for one Ironbound skill

Magic items should usually have 1 modifier or a 25% stat boost. Rare items should usually have 2-3 modifiers or a 50% stat boost. More complex modifiers like life steal, mana on kill, chance-to-cast effects, and reduced cooldowns should be saved for later versions.

### Attribute Scaling

Weapons and some skill-focused items should have attribute scaling grades, inspired by action RPG scaling systems.

Scaling grades:

- F: tiny bonus
- D: small bonus
- C: moderate bonus
- B: strong bonus
- A: very strong bonus
- S: exceptional bonus

Items can scale with Strength, Dexterity, Intelligence, or a combination.

Examples:

- Rusted Sword: Strength D, Dexterity F
- Heavy Axe: Strength C
- Hunting Bow: Dexterity C
- Ember Wand: Intelligence C
- Knight Blade: Strength C, Dexterity D

Damage should come from base item damage plus scaling bonuses from the relevant player attributes. This lets different classes prefer different weapons and makes stat choices matter.

### Loot Goals

Loot should support build experimentation. A weapon that adds fire damage should interest an Embercaller, Shadeblade, or Wildspeaker in different ways.

## Inventory

The inventory should be **list-based**, not grid-based.

Recommended first version:

- Simple scrollable list of carried items.
- Item categories: weapons, armor, jewelry, consumables, crafting materials, quest items.
- Weight or slot limit.
- Equipment screen.
- Compare item to currently equipped item.
- Sort by type, rarity, value, or newest.
- Mark items as favorite to avoid selling them by mistake.

Possible advanced version:

- Stash in town.
- Shared stash between characters.
- Search/filter by stat, rarity, or item type.

## World Structure

Use acts with themed areas.

### Act I: The Hollow Marches

- Ruined villages
- Crypts
- Bandit camps
- Haunted woods
- Boss: The Bellkeeper

### Act II: The Glass Wastes

- Desert ruins
- Buried temples
- Scorpion nests
- Cursed oasis
- Boss: The Sunken Vizier

### Act III: The Verdant Rot

- Jungle temples
- Poison swamps
- Cult shrines
- Ancient ziggurat
- Boss: The Blooming Horror

### Act IV: The Ashen Gate

- Burned fortress
- Demon rifts
- Obsidian halls
- River of cinders
- Boss: The Gate Tyrant

### Act V: The Starless Pit

- Frozen peaks
- Black citadel
- Void chambers
- Final descent
- Final Boss: The Dreaming Maw

## Procedural Generation

Each dungeon floor should generate:

- Rooms and corridors
- Monster packs
- Treasure chests
- Shrines
- Traps
- Events
- Stairs or exits
- Optional mini-boss rooms

Map generation types:

- Rectangular rooms connected by corridors
- Cellular caves
- Maze-like crypts
- Outdoor zones with obstacles

### MVP Dungeon Generator Rules

The first dungeon should feel like a crypt and use room-and-corridor generation.

Rules:

- Map size currently implemented as 40x16 for terminal readability.
- Floor 1: 6-8 rooms.
- Floors 2-4: 7-9 rooms.
- Floors 5-9: 8-10 rooms.
- Floor 10: 5-7 rooms, ending in a boss room.
- Enemy difficulty is doubled across the board: floor 1 enemies have roughly 2x base health and damage.
- Enemy difficulty still scales steadily by floor, reaching roughly 4x base health and damage by floor 10.
- XP and gold rewards scale more conservatively by floor, reaching roughly 2x base rewards by floor 10.
- Armor also rises on all floors and increases modestly on deeper floors.
- Rooms are rectangular crypt chambers.
- Corridors are narrow, creating tactical chokepoints.
- Place player in the first room.
- Place stairs in the farthest room.
- On floor 10, the farthest room is the Bellkeeper boss room.
- Place enemies and chests inside rooms instead of randomly in open space.
- Add 1-3 chests per floor.
- Chests open automatically when the player steps onto them.
- Stairs activate automatically when the player steps onto them.
- Locked chests can be added in a later version.
- No traps in the MVP.
- Shrines can be added after the MVP.

## Monsters

### Monster Roles

- Swarm: weak but numerous
- Brute: slow, high health
- Archer: ranged attacker
- Caster: spells and curses
- Summoner: creates minions
- Elite: stronger version with modifiers
- Boss: unique mechanics

### Elite Modifiers

- Burning
- Frozen
- Vampiric
- Armored
- Teleporting
- Explosive
- Poisonous
- Summoner
- Swift

## Boss Design

Bosses should have recognizable phases and mechanics.

Example boss: **The Bellkeeper**

The Bellkeeper should be the first major test of positioning, potions, cooldowns, and target priority.

Suggested MVP mechanics:

- Phase 1, 100%-60% health: uses slow melee swings and occasionally summons one skeleton.
- Phase 2, 60%-25% health: rings the cursed bell every few turns, dealing small unavoidable damage unless the player moves behind cover or out of range.
- Phase 3, below 25% health: enrages, gains +2 damage, stops summoning, and attacks more aggressively.

Boss room rules:

- The room should contain a few pillars or wall chunks that can block bell waves.
- Skeleton summons should be limited so the room does not become impossible.
- Defeating the Bellkeeper completes the MVP main objective.
- Reward: large XP, 100-150 gold, and one guaranteed random magic weapon or armor item.

## Hub Town

The full hub can eventually include:

- Healer
- Blacksmith
- Merchant
- Stash
- Skill trainer
- Quest giver
- Portal back to dungeon

### MVP Town Services

The first playable version should include:

- Healer: restores health and mana in town.
- Merchant: sells potions and basic gear, buys unwanted items.
- Blacksmith: sells weapons/armor and can repair gear if durability is added later.
- Stash: stores extra items between dungeon runs.
- Dungeon entrance: starts or resumes the current dungeon run.

## Quests

### Main Quests

The MVP has one main objective: **kill the Bellkeeper** at the bottom of the first dungeon.

After killing the Bellkeeper:

- The player returns to town.
- The main quest can be completed in town.
- A placeholder Act II entrance or menu option should appear, but Act II does not need to be playable in the MVP.

Later acts can add more main quests that advance the story and unlock new areas.

### Side Quests

Optional rewards:

- Skill point
- Unique item
- Gold
- New vendor service
- Permanent stat bonus

Example side quests:

- Rescue a captured smith.
- Purify a cursed shrine.
- Recover a lost relic.
- Defeat an optional dungeon boss.

## Economy and Crafting

### Currency

- Gold from monsters, chests, and selling items.

### MVP Prices

Town prices should be low and readable:

- Health potion: 50 gold
- Mana potion: 100 gold
- Basic weapon: 40-80 gold
- Basic armor: 35-75 gold
- Basic shield: 40 gold

Selling rules:

- Items sell for 25% of their buy value.
- Magic and rare items can sell for more based on modifiers.

### Gold Drops

Early gold drops should be small:

- Rat: 0-3 gold
- Skeleton: 2-8 gold
- Cultist: 5-12 gold
- Boneguard: 8-18 gold
- Elite: 20-40 gold
- Bellkeeper: 100-150 gold
- Small chest: 10-25 gold
- Treasure chest: 30-75 gold

### Vendors

- Buy potions
- Buy basic weapons and armor
- Sell unwanted gear

Durability and repairs should **not** exist in the MVP. The blacksmith should sell weapons and armor, but should not repair gear unless durability is added in a later version.

### Crafting Ideas

- Upgrade item rarity
- Add socket
- Insert rune/gem
- Reroll modifiers
- Combine materials into consumables

## Sockets, Gems, and Runes

### Gems

Add simple stat bonuses:

- Ruby: fire damage or health
- Sapphire: frost damage or mana
- Emerald: poison damage or dexterity
- Topaz: shock damage or gold find
- Diamond: resistance or armor

### Runes

Runes can create special effects when inserted into gear.

Example rune effects:

- Gain shield after killing enemy
- Fire nova on critical hit
- Summons last longer
- Potions heal more

## Permadeath and Difficulty

Difficulty modes:

- Normal: standard play
- Veteran: stronger enemies, better loot
- Nightmare: resistance penalties, elite enemies more common
- Abyssal: endgame challenge

Character death mode should be chosen when creating a character:

- Softcore: death returns to town with penalty.
- Hardcore: death permanently ends the character.

The game should support both modes from the design level, even if Hardcore is implemented after the first playable prototype.

## Death Penalties

Possible penalties:

- Lose some gold
- Drop some carried items
- Lose experience progress toward next level
- Return to town

For early development, use simple gold loss and return to town.

## User Interface

### Main Screen Layout

```text
+--------------------------------------------------+
| Dungeon: Hollow Crypts L2        Gold: 1240      |
| HP: 45/60  MP: 18/30  XP: 340/500               |
+-----------------------------+--------------------+
| ###########                 | Log                |
| #.@....r..#                 | You hit rat.       |
| #..##.....#                 | Rat dies.          |
| #.....!...#                 | Found potion.      |
| ###########                 |                    |
+-----------------------------+--------------------+
| Skills: [1]Cleave [2]Bash [R]Potion              |
+--------------------------------------------------+
```

### Screens

- Main map
- Inventory
- Equipment
- Character stats
- Skill tree
- Quest log
- Vendor
- Stash
- Settings

## Audio/Visual Terminal Feedback

Even without graphics, the game can feel responsive with:

- Colored text
- Combat log messages directly below the map
- Damage numbers
- Screen shake effect using brief redraw offset
- Flashing symbols for spell effects
- Different colors for rarity
- Color is allowed in the terminal UI, but all gameplay symbols must remain ASCII-only.
- Dungeon colors: player green, walls gray, floor dim gray, stairs cyan, chests yellow, boss red, elite magenta, and enemies use distinct colors.
- Command help for town, vendors, stash, attributes, dungeon, and pause screens should be anchored to the bottom of the terminal so it is easy to find.
- In dungeon combat, each active skill should have a help line above the footer showing its key, cost, cooldown, effect, and remaining cooldown/active turns.
- Important UI text should use color: green for safe/positive options, yellow for gold/items/messages, red for danger/quit/back, blue for mana, and cyan/magenta for headings or special screens.
- Animated projectile movement using short delays

## Save System

Save data should include:

- Character class
- Level and XP
- Stats
- Skills
- Inventory
- Equipment
- Current act and quest state
- Current dungeon seed or map state
- Gold and stash

Recommended format:

- JSON save file for readability during development.

MVP save/load behavior:

- Auto-save after every player action.
- Menu and dungeon commands should execute immediately on single keypresses without requiring Enter, except text entry such as character name.
- Escape should consistently go back from submenus or return from dungeon to town. Main town uses `q` to save and quit.
- Load the latest save automatically on startup if one exists.
- Save active dungeon state, including map, enemies, items on the ground, player position, HP, mana, cooldowns, inventory, equipment, gold, XP, and quest progress.
- If the player leaves or abandons the dungeon from town, clear the active dungeon state so the next dungeon entry generates a fresh dungeon.

## Minimum Viable Product

Build the smallest fun version first.

### MVP Features

- Terminal map display
- Player movement
- One dungeon generator
- First dungeon has 10 floors
- First dungeon persists while the player remains in it
- Leaving or abandoning the dungeon resets it
- Three enemy types
- Basic melee combat
- Health and potions
- Gold and simple item drops
- Inventory and equipment
- One class with a small but meaningful skill tree
- Leveling system
- One boss
- Save/load

### MVP Class

Start with **Ironbound** because melee combat is easiest to prototype.

The Ironbound starts with:

- Rusted sword
- Worn shield
- Cloth tunic or battered mail
- 2 lesser health potions
- 1 lesser mana potion

Mana does not regenerate during dungeon exploration. Mana is restored by drinking mana potions, such as lesser mana potions, or resting/healing in town.

Starting equipment is picked automatically based on class. The player does not choose a starting weapon in the MVP.

### MVP Enemies

The first enemies should have small health, damage, armor, speed, and XP values. They should be dangerous in groups but simple enough for early testing. The first dungeon should give enough XP for the player to level up a couple of times before or after defeating the boss.

Suggested starting enemy stats:

| Enemy       |               Health |           Damage |    Armor |    Speed |         XP |
| ----------- | -------------------: | ---------------: | -------: | -------: | ---------: |
| Rat         |                    6 |              1-2 |        0 |       11 |          8 |
| Skeleton    |                   12 |              2-4 |        1 |        9 |         18 |
| Cultist     |                   10 | 2-3 ranged/magic |        0 |       10 |         22 |
| Boneguard   |                   18 |              3-5 |        2 |        8 |         35 |
| Elite enemy | Base enemy x2 health |      +50% damage | +1 armor | +1 speed | Base XP x3 |
| Bellkeeper  |                   60 |              5-8 |        3 |        8 |        250 |

MVP enemy roles:

- Rat: weak melee
- Skeleton: basic melee
- Cultist: ranged or caster
- Boneguard: tougher melee enemy that protects cultists
- Elite enemy: stronger monster with one modifier
- Boss: Bellkeeper

### First Dungeon Enemy Placement

Floor 1:

- Rats
- Skeletons

Floor 2:

- Skeletons
- Cultists
- First elite enemy

Floor 3-9:

- Skeletons
- Cultists
- Boneguards
- Elite enemies on some floors

Floor 10:

- Cultists
- Boneguards
- Bellkeeper boss

### MVP Items

Starter and early gear should be poor F-tier equipment.

Example starter/early items:

- Rusted Sword: 3-5 damage, Strength F, Dexterity F
- Worn Shield: +1 armor, +2 dodge
- Cloth Tunic: +1 armor
- Battered Mail: +2 armor, -5 speed
- Crude Axe: 4-6 damage, Strength F
- Splintered Club: 2-6 damage, Strength F

Other MVP items:

- Lesser health potions: restore 15% of maximum health.
- Lesser mana potions: restore 15% of maximum mana.
- Gold

### MVP Equipment Interaction

The inventory screen should show currently equipped weapon, armor, and shield. Pressing an item number equips weapons, armor, or shields and swaps the old equipped item back into inventory. Weapon damage should come from the equipped weapon. Armor and shields should affect armor, dodge, and speed.

Loot should feel rewarding:

- Enemies have a chance to drop equipment or potions.
- Enemy health and damage are doubled across the board and scale up by floor, reaching roughly 4x base values on floor 10.
- XP and gold rewards scale up by floor, reaching roughly double values on floor 10.
- Chests always drop gold and an item.
- Bellkeeper drops guaranteed better loot.
- Items can be Common, Magic, or Rare.
- Magic and Rare loot has better stats and value.
- Inventory should show simple comparisons versus currently equipped gear, using green for upgrades and red for downgrades.
- Item names should be colored by rarity: Common white, Magic blue, Rare yellow.
- Magic and Rare items should use more exciting generated names with prefixes/suffixes.
- Loot drop messages should stand out in the combat log.

## Stretch Goals

- Multiple classes
- Skill trees
- MVP town skill tree lets the player spend skill points to upgrade Cleave, Shield Bash, and Battle Cry up to rank 5.
- Magic/rare/unique items
- Elemental damage and resistances
- Procedural act structure
- Boss phases
- Town hub and vendors
- Stash
- Crafting
- Hardcore mode
- Endgame dungeon ladder
- Mouse support if terminal library allows

## Suggested Technical Direction

Chosen language: **Rust**.

Recommended Rust libraries:

- `ratatui` for terminal UI layout and widgets.
- `crossterm` for terminal input/output backend.
- `serde` and `serde_json` for save files and data-driven content.
- `rand` for procedural generation and loot rolls.
- `anyhow` or `thiserror` for error handling.

Rust is a good fit because the game will have many interacting systems, and Rust's type safety helps keep combat, items, saves, and procedural generation reliable.

Suggested project structure:

```text
Cargo.toml
src/
  main.rs
  app.rs
  game.rs
  map.rs
  entity.rs
  combat.rs
  items.rs
  inventory.rs
  skills.rs
  ai.rs
  save.rs
  ui.rs
  data.rs
saves/
assets/
  data/
    classes.json
    items.json
    monsters.json
    skills.json
README.md
design.md
```

## Development Milestones

### Milestone 1: Town

- Create Rust project and terminal UI shell.
- Show the ASCII town screen.
- Create/load character.
- Support Softcore/Hardcore choice.
- Show player stats, gold, inventory, and equipment.
- Add healer, merchant, blacksmith, stash, and dungeon entrance menus.
- Add lesser health potions, lesser mana potions, starter gear, buying, selling, and stash storage.
- Auto-save after every action and load on startup.

### Milestone 2: Dungeon

- Generate the first room-and-corridor dungeon floor.
- Draw ASCII map.
- Move player cardinally only.
- Add walls, floors, stairs, chests, gold, items, and collision.
- Add enemies, speed/energy turn order, basic AI, attacks, hit chance, damage, armor, XP, and leveling.
- Add inventory/equipment changes during a dungeon run.
- Add ten floors and dungeon reset when leaving.

### Milestone 3: Boss

- Add floor 10 boss room.
- Implement the Bellkeeper phases and skeleton summons.
- Add boss rewards: large XP, 100-150 gold, guaranteed magic weapon or armor.
- Complete main objective when the Bellkeeper dies.
- Return player to town after victory.

### Milestone 4: Skills and Loot Polish

- Implement Ironbound skills: Cleave, Shield Bash, Battle Cry, Deep Cut, Iron Guard, Second Wind.
- Add skill tree screen and skill point spending.
- Add Common, Magic, Rare, and Unique item generation.
- Add item scaling grades and item comparison UI.

### Milestone 5: Balance and Polish

- Balance enemy stats, XP curve, gold drops, and item drops.
- Improve combat log and UI readability.
- Add color while keeping ASCII-only gameplay symbols.
- Test save/load, Hardcore death, dungeon reset, and boss victory flow.

## Resolved Design Decisions

- Combat is fully turn-based.
- Turn order uses a simple speed/energy system.
- Display is ASCII only.
- Movement is cardinal only; no diagonal movement.
- Inventory is list-based.
- Skills use both mana costs and cooldowns.
- The game starts in a safe town hub.
- The first dungeon has 10 floors and ends with the Bellkeeper boss.
- Character creation includes a Softcore or Hardcore permadeath choice.
- Programming language is Rust.
- The first skill tree should be moderately complex, with enough branching to support different builds.

## Resolved MVP Scope Decisions

- MVP town services are healer, merchant, blacksmith, stash, and dungeon entrance.
- First Ironbound skills are Cleave, Shield Bash, Battle Cry, Deep Cut, Iron Guard, and Second Wind.
- Floor 1 enemies are rats and skeletons.
- Floor 2 enemies are skeletons, cultists, and the first elite enemy.
- Floor 10 enemies are cultists, boneguards, and the Bellkeeper boss.
- MVP item rarities are Common, Magic, Rare, and Unique.
- Unidentified items are not included in the MVP.
- The first dungeon persists only while the player remains in the dungeon.
- Leaving or abandoning the dungeon resets it.
- Beginning primary attributes are Strength, Dexterity, and Intelligence.
- Every class starts with 10 total primary attribute points.
- Base player health, mana, hit rating, dodge rating, and speed are each 10 before attributes.
- Ironbound starts with 6 Strength, 3 Dexterity, and 1 Intelligence.
- Strength gives +5 health per point.
- Dexterity gives +5 hit rating and +5 speed per point.
- Intelligence gives +5 mana per point.
- Hit chance uses `hit / (hit + dodge)` and is clamped between 20% and 95%.
- Item damage scales from item base damage plus attribute scaling grades: F, D, C, B, A, S.
- XP required to level starts at 40 and doubles each level.
- Each level gives 3 attribute points and 1 skill point.
- The player starts with 2 lesser health potions and 1 lesser mana potion.
- MVP includes both lesser health potions and lesser mana potions.
- Lesser potions restore 15% of maximum health or mana.
- Mana only restores from mana potions or in town.
- Starting weapon and gear are automatically picked based on class.
- The MVP main objective is to kill the Bellkeeper.
- Killing the Bellkeeper rewards a guaranteed random magic weapon or armor item and a large XP reward.

## Remaining Open Questions

- Milestone 1 implementation has started.
- Later: decide the exact title screen text and town NPC names.
- Later: decide whether placeholder Act II should show a teaser screen, locked menu option, or short message.
- Later: decide which systems from Milestone 2 should be implemented first after the town is playable.

## Initial Recommendation

Start with a focused MVP: a turn-based terminal dungeon crawler with one class, one act, simple procedural maps, basic loot, and a boss. Once the core loop feels fun, expand into more classes, deeper itemization, and additional acts.
