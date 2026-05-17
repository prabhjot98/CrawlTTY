# Terminal Action RPG Design

## Title

**CrawlTTY**

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
!  Ground loot
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
G / g             Pick up item (`g=pickup`)
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

Warrior starting attributes:

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

The implementation uses the same model for both sides of combat. Player weapon and skill attacks use the character's hit rating against the target enemy's dodge rating. Enemy melee attacks use that enemy's hit rating against the player's current defensive dodge rating, including temporary smoke-protection bonuses. Ranged enemy specials use the enemy hit rating plus a small attack bonus against the same player dodge rating. Enemy hit and dodge ratings are explicit per archetype and gain modest floor-based increases so later floors keep pace without multiplying accuracy as aggressively as health and damage.

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

### 1. Warrior

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

### 4. Rogue

A fast dagger fighter using Energy, internal combo points, poison setup, burst finishers, and smoke-assisted mobility. The current implementation labels the Rogue dungeon resource as Energy, regenerates it during dungeon turns, shows class-aware Rogue dungeon skill help with Energy/CP and rank-scaled values, and implements Backstab, Venom Edge, Eviscerate, and Smoke Step. Rogue on-hit skill effects only apply when the attack hits. Eviscerate spends combo points on any valid cast, including a miss. The skill tree screen is class-aware: Rogues see Daggers, Venom, and Smoke branches with Backstab, Eviscerate, Venom Edge, Rupture, Smoke Step, and Slip Away upgrades. Eviscerate upgrades require Backstab rank 2, Rupture unlocks at Venom Edge rank 2, and Slip Away unlocks at Smoke Step rank 2. Rupture starts inactive and extends Venom Edge poison duration by rank once unlocked. Slip Away starts inactive and adds smoke-protection dodge once unlocked. Smoke Step spends Energy after the player chooses an explicit direction, dashes 1-2 cardinal tiles along a clear path to an open destination, resolves normal landing-tile interactions, briefly improves defense with rank-scaled smoke dodge from Smoke Step plus unlocked Slip Away, and enables an empowered Backstab on the next player action. Rogues use Energy instead of mana potions and can equip Rogue bucklers, but not Warrior shields.

Possible builds:

- Combo-point dagger assassin
- Poison executioner
- Smoke skirmisher

Example skills:

- Backstab: melee builder that grants a combo point and rewards movement, smoke, or poisoned targets
- Venom Edge: melee builder that poisons the target and grants a combo point
- Eviscerate: finisher that spends internal combo points for heavy physical burst
- Smoke Step: short cardinal dash that grants brief protection and enables an empowered Backstab
- Rupture: Venom branch upgrade that extends Venom Edge poison duration
- Slip Away: Smoke branch upgrade gated by Smoke Step rank 2

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

Example Warrior trees:

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

The first implemented class should have a moderately complex skill tree rather than only four isolated abilities. Start with the **Warrior** and give it three small branches.

#### Warrior MVP Skill Tree

The first implemented Warrior skills should be:

- Cleave
- Shield Bash
- Battle Cry
- Deep Cut
- Iron Guard
- Second Wind

The Warrior uses **Mana** in the MVP for simplicity. A later version may rename the Warrior resource to Fury while keeping similar mechanics.

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
- Passive skills require rank 2 in their branch's active starter: Deep Cut requires Cleave, Iron Guard requires Shield Bash, and Second Wind requires Battle Cry.
- The skill tree displays locked passive prerequisites inline under the starter skill with a lock marker and current rank progress, and repeats the same unlock progress in the selected passive's detail pane.
- The player should be able to specialize in one branch or mix branches.
- The MVP should include enough skills to make leveling choices interesting, even if not every skill has advanced animations or effects at first.

Mastery rules:

- When a skill reaches rank 5, selecting it in the skill tree opens a mastery choice.
- A mastery is free and does not consume a skill point.
- Each rank 5 skill has 3 mutually exclusive mastery paths.
- Picking one mastery permanently locks the other two choices for that skill.

Warrior mastery paths:

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
- +Skill rank for one Warrior skill

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

Loot should support build experimentation. A weapon that adds fire damage should interest an Embercaller, Rogue, or Wildspeaker in different ways.

Current class-specific equipment direction:

- Random equipment drops route through the active character class.
- Warriors receive Warrior equipment only: swords, axes, mail armor, and guard shields.
- Rogues receive Rogue equipment only: daggers, scimitars, light armor, and bucklers.
- Bows and ranged Dexterity weapons are reserved for a future Ranger class.

## Inventory

The inventory should be **grid-based** while avoiding a manual packing puzzle.

Current direction:

- Current data model: `Character.inventory` and `Character.stash` use `ItemGrid` containers. The starter bag is `4 x 4`, the stash is `8 x 8`, and each grid stores compacted `items` in row-major order for gradual migration from older Vec-based inventory code.
- Current implementation status: the player bag and stash both render as ratatui grids with cursor movement and selected-item detail panels. Occupied bag and stash grid cells color their bracket outline by item rarity while keeping the focused item label bold green, and the active stash-side container uses a muted cursed-violet border. The inventory screen keeps the bag grid content-sized and lets details expand into spare width. When selected inventory gear is a weapon, armor, or shield, the details panel names the currently equipped item for that slot, shows direct stat deltas, and shows cannot-equip requirements when the selected gear is locked. Stash shows the bag grid, content-sized stash grid, and active-side details in one screen, letting details expand into spare width and using a stacked details pane on narrower terminals so the `4 x 4` bag and `8 x 8` stash remain visible.
- Current dungeon state includes a `ground_items` list of positioned `GroundItem` records. New dungeon floors start with no ground items; monster, chest, and boss loot that cannot fit in the bag is preserved on the source tile as visible `!` ground loot. A single item on the player tile can be picked up with `g=pickup` or by walking over it when inventory space exists. When multiple items are on the player tile, or the bag is full, `g` opens a responsive ratatui ground-loot picker that lists the items, shows selected item details, allows picking up one item when space exists, and allows discarding selected ground loot. Picking up or discarding from the picker spends a dungeon turn. Walking over loot only auto-picks a single item when inventory space exists; multi-item and full-bag cases leave the loot on the ground and log the picker prompt. If boss reward loot falls to ground, the completed dungeon is retained instead of immediately returning to town so the reward stays accessible; remaining enemies are cleared so the player can leave after resolving loot. Dropping an inventory item inside a dungeon places it on the player tile; dropping in town is disallowed and leaves the inventory unchanged.
- The player bag is a ratatui grid screen with a cursor-selectable cell grid and selected item details; the panels sit side-by-side on wide terminals and stack on narrower terminals.
- Every item occupies exactly one cell.
- The bag auto-compacts after pickups, drops, equips, selling, salvaging, stash transfers, and item use.
- Empty cells remain visible so capacity is clear.
- The player bag starts at `4 x 4` and caps at `8 x 8`.
- Bag expansion comes from the Quartermaster project chain: Storehouse Shelves (`200` gold, `5 x 4`), Pack Hooks (`350` gold, `5 x 5`), Oilcloth Satchel (`500` gold, `6 x 5`), Quartermaster Ledger (`700` gold, `6 x 6`), Reinforced Pack (`950` gold, `7 x 6`), Stitched Pockets (`1200` gold, `7 x 7`), Deep Rucksack (`1500` gold, `8 x 7`), and Exile's Trunk (`1900` gold, `8 x 8`).
- The stash is also a grid, starts at `8 x 8`, and uses the same selected-item detail panel pattern.
- Stash transfers require destination capacity. Moving an item into a full inventory or stash grid fails with `No room in destination.` and leaves both grids unchanged.
- Future storage expansion beyond the first `8 x 8` bag should use tabs rather than one larger grid.
- Ratatui is the standard renderer for interactive screens, including character creation, town, dungeon, merchant, blacksmith, distillery, projects, attributes, skills, inventory, stash, ground-loot, sell, salvage, socket, and gem-picking flows. Non-interactive process messages such as reset-save output, fatal exit notices, and final goodbye text may remain plain stdout. The attributes screen supports cursor selection with `W/S`, arrow keys, and `Enter`, while keeping `1-3` hotkeys available.

Recommended features:

- Cursor movement through cells with WASD and arrow keys.
- Equipment slots: weapon, off-hand, helm, armor, gloves, boots, belt, amulet, rings.
- Sort by type, rarity, level, or value.
- Mark items as favorite to avoid selling them by mistake.

Later expansion:

- Inventory and stash tabs.
- Search/filter by stat, rarity, or item type.
- Optional grid views for vendor, salvage, and socket-management screens after the core bag and stash grids are stable.

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
- Armor also rises on all floors and increases modestly on deeper floors. Hit and dodge ratings scale more slowly than damage and health.
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

- Automatic full heal when returning to town
- Blacksmith
- Merchant
- Stash
- Skill trainer
- Quest giver
- Portal back to dungeon

### MVP Town Services

The first playable version should include:

- Return-to-town healing: restores health and mana when the player comes back to town and displays a full-heal message.
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
- Insert or remove rune/gem
- Reroll modifiers
- Combine materials into consumables

## Sockets, Gems, and Runes

### Sockets

Sockets are found on dropped weapons, armor, and shields. Starter gear, vendor gear, and other fixed basic items do not roll sockets. The blacksmith cannot add sockets.

Socket roll odds:

- Common gear: 10% chance for 1 socket
- Magic gear: 20% chance for 1 socket, 5% chance for 2 sockets
- Rare gear: 25% chance for 1 socket, 10% chance for 2 sockets

The Socket Bench town project unlocks free gem insertion, removal, and replacement in town. Removing a gem returns it to inventory. Replacing a gem returns the old gem to inventory and inserts the selected new gem. Removing a socketed gem requires one free bag cell; if the bag is full, the action fails and the gem remains socketed. Replacing a socketed gem with a bag gem is capacity-neutral because removing the selected new gem frees the cell used for the replaced gem.

### Gems

Gems are normal inventory items found as rare dungeon drops. Each gem type grants exactly one flat stat, and gems are slot-agnostic: any gem can go into any socketed weapon, armor, or shield. Gems are drop-only and cannot be upgraded or combined.

Gem tiers:

- Chipped: common early gem tier
- Flawed: uncommon improved gem tier
- Pristine: rare best gem tier for the first socket system

Gem tier weights:

- 80% Chipped
- 17% Flawed
- 3% Pristine

Gem stat bonuses:

| Gem        | Stat          | Chipped | Flawed | Pristine |
| ---------- | ------------- | ------: | -----: | -------: |
| Ruby       | Max HP        |      +5 |    +10 |      +20 |
| Sapphire   | Max mana      |      +3 |     +6 |      +12 |
| Garnet     | Strength      |      +1 |     +2 |       +3 |
| Emerald    | Dexterity     |      +1 |     +2 |       +3 |
| Amethyst   | Intelligence  |      +1 |     +2 |       +3 |
| Quartz     | Hit rating    |      +3 |     +6 |      +10 |
| Jade       | Dodge rating  |      +2 |     +4 |       +8 |
| Onyx       | Armor         |      +1 |     +2 |       +3 |
| Citrine    | Speed         |      +2 |     +4 |       +7 |
| Topaz      | Crit chance   |     +1% |    +2% |      +4% |
| Opal       | Gold found    |     +5% |   +10% |     +20% |
| Bloodstone | Weapon damage |      +1 |     +2 |       +3 |

Gem drop rules:

- Gems start dropping on floor 3.
- Enemy loot has a 2% chance to be a gem.
- Chests have a 6% chance to include a gem.
- Elites have a 5% chance to drop a gem.
- Bosses have a 25% chance to drop a gem.
- Gems can drop before Socket Bench is unlocked, but cannot be inserted, removed, or replaced until the project is complete.

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

Character death mode should be chosen in step 3 of character creation, after class and name:

- Softcore: death returns to town, clears dungeon combat state, loses 10% gold, and fully restores health and class resource.
- Hardcore: death deletes the save, clears the active dungeon, ends the run, and returns through the normal terminal cleanup path.
- The Softcore/Hardcore choice toggles with Tab or Up/Down arrows; `S` and `H` remain normal name-entry letters and do not change death mode.

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
- The current implementation renders interactive screens with ratatui. Character creation starts inside the ratatui terminal session as three stacked step containers: class, name, then Softcore/Hardcore; the active step container uses a muted cursed-violet border. Enter advances through the steps, Escape backs up one step, and Up/Down arrows move the current class or death-mode selection while Tab also toggles Softcore/Hardcore. Town submenus such as merchant, blacksmith, distillery, projects, attributes, skills, sell, salvage, socket, and gem picker draw through ratatui event loops rather than legacy ANSI `println!` screens. The merchant sells lesser health potions to all classes, sells lesser mana potions to Warriors, and buys unwanted inventory items. The Distillery town screen shows the current herb count and crafts lesser potions from herbs once the Distillery project is complete. The skills screen uses cursor selection with a responsive two-pane body: skills and details split horizontally on wide terminals and stack vertically on narrower terminals so both remain visible. The attributes screen uses cursor selection, remains viewable when no attribute points are available, and shows an explicit empty state until the player backs out. Attribute labels use semantic colors: Strength red, Dexterity green, and Intelligence blue. Scrollable ratatui selection lists keep the focused sell, salvage, socket target, gem, and ground-loot entries visible as the cursor moves, with visible row counts derived from the active Ratatui frame area's usable list body so selected details remain visible and gem and ground-loot pickers using Ratatui `List`/`ListState` selection widgets. Terminal resize events trigger an immediate redraw of the active ratatui screen, keyboard handling intentionally accepts only fresh key-press events and ignores repeat/release key events so held keys do not repeat actions and release noise does not trigger menu actions, and returning from submenus relies on Ratatui's normal redraw instead of forced terminal clears to avoid unnecessary flicker.
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
- Ground items in the active dungeon

Recommended format:

- JSON save file for readability during development.

MVP save/load behavior:

- Auto-save after every player action.
- Menu and dungeon commands should execute immediately on single keypresses without requiring Enter, except text entry such as character name.
- Escape should consistently go back from submenus or return from dungeon to town. Main town uses `q` to save and quit.
- Load the latest save automatically on startup if one exists.
- Save active dungeon state, including map, enemies, items on the ground, player position, HP, mana, cooldowns, inventory, equipment, gold, XP, and quest progress.
- If the player leaves or abandons the dungeon from town, clear the active dungeon state so the next dungeon entry generates a fresh dungeon.
- The 1.0.0 multi-class release intentionally breaks and resets saves from older major versions through the existing save-version gate.
- The inventory grid rework may break older save files. During development, existing local saves can be deleted or reset with `cargo run -- reset-save` instead of migrated.

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

Start with **Warrior** because melee combat is easiest to prototype.

The Warrior starts with:

- Rusted sword
- Worn shield
- Cloth tunic or battered mail
- 2 lesser health potions
- 1 lesser mana potion

Rogues start with a dagger, light armor, an empty offhand, and 2 lesser health potions.

Mana does not regenerate during dungeon exploration. Mana is restored by drinking mana potions, such as lesser mana potions, or by returning to town.

Starting equipment is picked automatically based on class. The player does not choose a starting weapon in the MVP.

### MVP Enemies

The first enemies should have small health, damage, armor, speed, and XP values. They should be dangerous in groups but simple enough for early testing. The first dungeon should give enough XP for the player to level up a couple of times before or after defeating the boss.

Suggested starting enemy stats:

| Enemy       |               Health |           Damage |    Armor |    Speed | Hit | Dodge |         XP |
| ----------- | -------------------: | ---------------: | -------: | -------: | --: | ----: | ---------: |
| Rat         |                    6 |              1-2 |        0 |       11 |  18 |    14 |          8 |
| Skeleton    |                   12 |              2-4 |        1 |        9 |  25 |    10 |         18 |
| Cultist     |                   10 | 2-3 ranged/magic |        0 |       10 |  28 |    12 |         22 |
| Boneguard   |                   18 |              3-5 |        2 |        8 |  24 |     8 |         35 |
| Elite enemy | Base enemy x2 health |      +50% damage | +1 armor | +1 speed | +5 |    +2 | Base XP x3 |
| Bellkeeper  |                   60 |              5-8 |        3 |        8 |  32 |     8 |        250 |

When enemies are scaled for later floors, health and damage use the floor difficulty multiplier, armor increases every few floors, hit rating increases by `(floor - 1) / 2`, and dodge rating increases by `(floor - 1) / 4`. This keeps accuracy and evasion meaningful without making late-floor enemies hit every turn.

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
- Lesser mana potions: restore 15% of maximum mana for mana-using classes. Rogues use Energy, cannot use mana potions, and class-aware consumable drops avoid mana potions for Rogues.
- Herbs: non-bag alchemy currency. Completing the Herb Garden project grows 1-3 herbs after each completed dungeon floor. Completing the Distillery project unlocks potion crafting: 3 herbs for a lesser health potion, and 4 herbs for a lesser mana potion for mana-using classes.
- Gold

### MVP Equipment Interaction

The inventory screen should show currently equipped weapon, armor, and offhand/shield. The current target design is a ratatui grid: the bag starts at `4 x 4`, shows empty cells, uses a cursor to select cells, and shows item details in a right side panel. Pressing Enter equips gear or uses consumables and swaps old equipped gear back into inventory. Full-bag replacement gear equips are allowed because removing the selected carried item frees the cell reused for the old gear. Weapon damage should come from the equipped weapon. Armor and shields should affect armor, dodge, and speed for classes that can equip shields. Rogues can equip empty offhand and Rogue bucklers, but cannot equip Warrior shields. Dropping an item in a dungeon places it on the ground instead of deleting it.

Loot should feel rewarding:

- Enemies have a chance to drop equipment or potions.
- Enemy health and damage are doubled across the board and scale up by floor, reaching roughly 4x base values on floor 10.
- Enemy hit and dodge ratings are explicit per archetype and scale modestly by floor.
- XP and gold rewards scale up by floor, reaching roughly double values on floor 10.
- Chests always drop gold and an item. Gold is always collected; if the bag is full, the item remains as ground loot on the chest tile.
- Bellkeeper drops guaranteed better loot.
- Items can be Common, Magic, or Rare.
- Magic and Rare loot has better stats and value.
- Randomly generated dropped weapons, armor, and shields can roll empty sockets; gems can drop from floor 3 onward, the Socket Bench manages free gem insertion, removal, and replacement, and Opal socket bonuses increase variable monster and chest gold.
- Inventory shows simple comparisons versus currently equipped gear, using green for upgrades and red for downgrades. The comparison is part of the selected-item details panel and includes direct weapon damage/crit or armor/dodge/speed deltas plus cannot-equip requirement text when relevant.
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
- Add merchant, blacksmith, stash, dungeon entrance menus, and automatic full healing on town return.
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

- Implement Warrior skills: Cleave, Shield Bash, Battle Cry, Deep Cut, Iron Guard, Second Wind.
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
- Inventory is grid-based with one-cell items, auto-compaction, and ratatui screens.
- Skills use both mana costs and cooldowns.
- The game starts in a safe town hub.
- The first dungeon has 10 floors and ends with the Bellkeeper boss.
- Character creation includes a Softcore or Hardcore permadeath choice.
- Programming language is Rust.
- The first skill tree should be moderately complex, with enough branching to support different builds.

## Resolved MVP Scope Decisions

- MVP town services are merchant, blacksmith, stash, and dungeon entrance; returning to town fully restores health and mana.
- First Warrior skills are Cleave, Shield Bash, Battle Cry, Deep Cut, Iron Guard, and Second Wind.
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
- Warrior starts with 6 Strength, 3 Dexterity, and 1 Intelligence.
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
- Herbs do not take inventory space. The Distillery UI displays the current herb count and available potion recipes.
- Mana only restores from mana potions or by returning to town.
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
