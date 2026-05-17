# CrawlTTY

CrawlTTY is a terminal-based action RPG/dungeon crawler built in Rust. It focuses on tactical turn-based combat, loot-driven progression, class builds, and a Unicode-enhanced terminal presentation.

## Current Features

- Unicode terminal UI using single-cell, non-emoji glyphs for terrain, cursors, locks, inventory cells, and progression bars, with readable letters for enemies and item labels
- Town hub with merchant, blacksmith, stash, town projects, quest giver, dungeon entrance, and automatic full healing on return
- Warrior, Rogue, and Sorceress classes with attributes, leveling, skills, and class-specific resources
- Procedural 10-floor Act I dungeon and 8-floor Act II dungeon
- Turn-based combat with enemies, elites, chests, loot, XP, gold, and potions
- Bellkeeper and Glass Tyrant boss encounters
- Grid-based bag and stash inventory, equipment, selling, salvaging, shard-only gear upgrading, and gold-funded town projects
- Quartermaster town projects upgrade bag capacity from `4 x 4` to `8 x 8`
- Ground loot pickup with `g`, plus automatic pickup when walking over a single item and bag space is available
- Save/load support via JSON saves

## Controls

### Town

- `m` merchant
- `b` blacksmith
- `l` distillery
- `s` stash
- `p` town projects
- `t` quest giver
- `d` dungeon
- `i` inventory
- `a` attributes
- `k` skill tree
- `q` save and quit

### Dungeon

- `w/a/s/d` move or attack
- Warrior: `1` Cleave, `2` Shield Bash, `3` Battle Cry
- Rogue: `1` Backstab, `2` Venom Edge, `3` Eviscerate, `4` Smoke Step
- Sorceress: `1` Firebolt, `2` Frost Ring, `3` Chain Spark, `4` Mana Shield
- `p` drink lesser health potion
- `g` pick up loot on current tile
- `i` inventory
- `Esc` return to town / back out of menus; first floors of acts can be escaped before clearing every monster

Dungeon glyphs use the Unicode visual set for terrain and effects, with letter glyphs for enemies: `☥` player, `▓` wall, `·` floor, `⌄` stairs, `◈` chest, `✦` loot, `✶` bell wave, `r/s/c/b` enemies, `E` elite, and `B`/`T` bosses.

## Running

```sh
cargo run
```

Reset the local save:

```sh
cargo run -- reset-save
```

## Development

Run validation:

```sh
cargo fmt
cargo test
cargo check
```

## Save Files

Saves are written to:

```text
saves/save.json
```

The `saves/` directory is local/generated data and should not be committed.

The 1.0.0 multi-class release intentionally resets saves from older major versions.

The inventory grid rework changed the save shape for inventory, stash, and active dungeon ground loot. During local development, old saves may fail to load and can be reset with:

```sh
cargo run -- reset-save
```

## Design

See [`design.md`](design.md) for the detailed design document and roadmap.
