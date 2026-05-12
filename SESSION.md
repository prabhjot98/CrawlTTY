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
- Three dungeon floors
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
d = dungeon
i = inventory
a = attributes
k = skill tree
q = save + quit
```

### Menus

```text
Esc = back
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

Inventory supports equipping/using items, paging, and dropping:

```text
1-9 = equip/use selected visible item
n/p = next/previous page
x   = drop selected visible item
Esc = back
```

Merchant selling supports selecting a visible item instead of only selling the first item:

```text
1-9 = sell selected visible item
n/p = next/previous page
Esc = back
```

Equipment slots:

```text
Weapon
Armor
Shield
```

Equipping gear swaps old gear back into inventory.

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
- Floor 3 farthest room contains Bellkeeper
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
```

## Current Technical Notes

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

1. Test and tune crypt dungeon generation.
2. Fix any room/corridor placement weirdness.
3. Improve enemy variety:
   - Cultists ranged attack
   - Boneguards guard/block
   - Elite modifiers
4. Improve inventory:
   - scrolling beyond 9 items
   - drop item
   - sell selected item
5. Add quest completion flow after Bellkeeper:
   - return to town
   - talk to NPC
   - complete Act I
   - show Act II placeholder
6. Expand skill tree:
   - upgrade Deep Cut
   - upgrade Iron Guard
   - upgrade Second Wind
   - add prerequisites

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
