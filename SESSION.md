# Session Summary

## Project

Terminal ASCII action RPG inspired by classic loot-driven dungeon crawlers.

Working title: **Ashen Depths**

Language: **Rust**

Run game:

```bash
cargo run
```

Reset save:

```bash
cargo run -- reset-save
```

## Current State

The game has a playable prototype with:

- Town hub
- Character creation
- Softcore/Hardcore choice
- Auto-save/load
- ASCII display with color
- Single-key input with no Enter required, except text entry
- Crypt-style dungeon generation
- Ten dungeon floors
- Bellkeeper boss
- Combat, loot, inventory, equipment, skill upgrades

## Important Files

- `design.md`: design document and decisions
- `src/main.rs`: current implementation
- `Cargo.toml`: Rust dependencies
- `saves/save.json`: local save file, ignored by git

## Git

A commit was made during the session:

```text
5efea7d Build terminal ARPG prototype
```

More work was done after that commit. Before restarting, commit latest work:

```bash
git add .
git commit -m "Improve loot combat and crypt dungeon generation"
```

## Current Controls

### Town

```text
h = healer
m = merchant
b = blacksmith
s = stash
t = quest / Warden Mara
d = dungeon
i = inventory
a = attributes
k = skill tree
q = save + quit
```

### Menus

```text
↑/↓ or w/s = select in cursor menus
Enter      = choose selected option
Esc        = back
```

### Dungeon

```text
w/a/s/d = move / bump attack
1       = Cleave
2       = Shield Bash
3       = Battle Cry
p       = drink health potion
i       = inventory
Esc     = return to town and reset dungeon
```

Chests and stairs are automatic:

```text
$ = chest, opens when stepped on
> = stairs, activates when stepped on
```

## Current Gameplay Systems

### Quest Flow

- Warden Mara in town gives the Act I objective with `t`.
- Defeating the Bellkeeper on floor 10 returns the player to town and blocks further crypt entry.
- Talk to Warden Mara after the Bellkeeper dies to complete Act I.
- Act I completion rewards 100 gold, +1 skill point, full heal, and unlocks the Act II placeholder.
- After Act I completion, the dungeon entrance shows the Glass Wastes placeholder instead of starting another crypt run.

### Character

Current class: **Ironbound**

Starting stats:

```text
Strength: 6
Dexterity: 3
Intelligence: 1
```

Every class starts with 10 total stat points.

Derived stats:

```text
Strength     => +5 health per point
Dexterity    => +5 hit rating and +5 speed per point
Intelligence => +5 mana per point
```

Leveling:

```text
XP requirement starts at 10 and doubles each level
+3 attribute points per level
+1 skill point per level
```

### Combat

- Turn-based
- Cardinal movement only, no diagonals
- Hit chance uses `hit / (hit + dodge)` clamped between 20% and 95%
- Weapon damage comes from equipped weapon stats
- Armor reduces incoming damage
- Cultists use ranged shadow bolts from 2-5 tiles away when the player is in a clear cardinal line of sight
- Boneguards raise shields at 2-4 tiles from the player, gaining +2 armor until their next turn
- Elite enemies always have one modifier: Armored (+2 armor), Swift (+2 speed), Vampiric (heals 2 HP after damaging player), or Burning (+1 damage)

### Skills

Skill tree is available in town with `k`.

Current active skills:

```text
1 Cleave
- Costs 5 mana
- 1 turn cooldown
- Hits up to 3 adjacent enemies
- Starts at 80% weapon damage
- +10% weapon damage per rank
- Max rank 5

2 Shield Bash
- Costs 6 mana
- 3 turn cooldown
- Hits 1 adjacent enemy
- Starts at 70% weapon damage
- Stuns enemy for 1 turn
- +10% weapon damage per rank
- Max rank 5

3 Battle Cry
- Costs 8 mana
- 6 turn cooldown
- Lasts 5 turns
- Starts at +20% player damage
- Enemies deal -10% damage while active
- +5% player damage per rank
- Max rank 5
```

Current passive effects implemented:

```text
Deep Cut
- Melee hits have 15% chance to bleed
- Bleed lasts 3 turns
- Bleed deals 2 damage per turn

Iron Guard
- +2 armor while using a shield

Second Wind
- While Battle Cry is active, kills restore 10% max HP
```

## Loot and Equipment

Inventory uses a pageless cursor list for equipping/using items and dropping:

```text
↑/↓ or w/s = select item
Enter      = equip/use selected item
x          = drop selected item
Esc        = back
```

Merchant main menu is cursor-based, shows current inventory, and buys without a continue prompt.

Merchant selling uses the same pageless cursor pattern and sells immediately without a continue prompt:

```text
↑/↓ or w/s = select item
Enter      = sell selected item
Esc        = back
```

Stash uses two cursor lists and moves items immediately without a continue prompt:

```text
↑/↓ or w/s = select item
Tab        = switch inventory/stash list
Enter      = store/retrieve selected item
Esc        = back
```

Equipment slots:

```text
Weapon
Armor
Shield
```

Equipping gear swaps old gear back into inventory.

Blacksmith crafting:

- No durability and no repairs.
- Carried weapons, armor, and shields can be salvaged into matching type shards.
- Equipped gear can be upgraded with type shards + gold.
- Weapons gain +1 min/+1 max damage per upgrade.
- Armor gains +1 armor per upgrade.
- Shields gain +1 armor per upgrade.
- Upgrade costs scale by next upgrade level: `2 * next_level` shards and `25 * next_level` gold.

Loot rarity:

```text
Common = white
Magic  = blue
Rare   = yellow
```

Loot sources:

- Enemies have a chance to drop loot
- Chests drop gold and an item
- Bellkeeper drops guaranteed better loot

Inventory shows comparisons:

- Weapons compare damage
- Armor/shields compare armor, dodge, speed
- Green means upgrade
- Red means downgrade

## Dungeon Generation

Current dungeon style: **Crypt**

Implemented:

- Rooms and narrow corridors
- Walls fill unused areas
- Player starts in first room
- Stairs are in the farthest room
- Floor 10 farthest room contains Bellkeeper
- Enemy health, damage, XP, and gold rewards scale up by floor, reaching roughly 2x on floor 10
- Enemies and chests spawn inside rooms
- 1-3 chests per floor

Map symbols:

```text
@ = player
# = wall
. = floor
$ = chest
> = stairs
r = rat
s = skeleton
c = cultist
b = boneguard
E = elite enemy
B = Bellkeeper boss
* = Bellkeeper bell wave
```

### Bellkeeper Boss Mechanics

- Phase 1 / Tolling, 100%-61% health: melee and skeleton summons every 3 boss turns.
- Phase 2 / Cursed Bell, 60%-26% health: keeps summoning and rings a cardinal bell wave every 4 boss turns.
- Phase 3 / Enraged, 25% health or lower: gains +2 melee damage, stops summoning, and continues bell waves.
- Bell waves appear on the map as `*` for the next redraw and damage the player if they are in the wave path.
- Skeleton summons are capped at 3 active summoned skeletons.

## Current Technical Notes

Regression tests are available and should pass before commits:

```bash
cargo test
cargo check
```

Current test coverage includes starting character state, leveling, skill scaling, cursor helpers, stash movement, equipment swapping, blacksmith salvage/upgrades, dungeon generation, stairs behavior, cultist ranged attacks, boneguard guarding, elite modifiers, Bellkeeper phases/summons/waves, Bellkeeper bleed-death victory handling, and potion use.

Rust dependencies:

```text
serde
serde_json
anyhow
rand
crossterm
```

Save file:

```text
saves/save.json
```

The save file may need reset after structural changes:

```bash
cargo run -- reset-save
```

## Recommended Next Tasks

1. Test full MVP run from fresh save through Bellkeeper and Act I turn-in.
2. Polish Bellkeeper boss room:
   - cover pillars / wall chunks that can block bell waves
   - balance summon/wave timing and boss damage
3. Expand skill tree:
   - upgrade Deep Cut
   - upgrade Iron Guard
   - upgrade Second Wind
   - add prerequisites
4. Balance and polish:
   - enemy stats, XP, gold, loot rates
   - combat log clarity
   - dungeon generation edge cases

## Restart Instructions

After restarting the PC, return to the project:

```bash
cd /Users/pssandhu/d2
```

Ask Pi:

```text
Read SESSION.md and design.md, then continue from the latest state.
```

Then verify:

```bash
git status
cargo check
cargo run
```
