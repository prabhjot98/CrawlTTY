# CrawlTTY

CrawlTTY is a terminal-based action RPG/dungeon crawler built in Rust. It focuses on tactical turn-based combat, loot-driven progression, class builds, and readable ASCII presentation.

## Current Features

- ASCII terminal UI
- Town hub with merchant, blacksmith, stash, quest giver, dungeon entrance, and automatic full healing on return
- Ironbound class with attributes, leveling, skills, and mastery choices
- Procedural 10-floor Act I dungeon and 8-floor Act II dungeon
- Turn-based combat with enemies, elites, chests, loot, XP, gold, and potions
- Bellkeeper and Glass Tyrant boss encounters
- Inventory, equipment, selling, salvaging, and gear upgrading
- Save/load support via JSON saves

## Controls

### Town

- `m` merchant
- `b` blacksmith
- `s` stash
- `t` quest giver
- `d` dungeon
- `i` inventory
- `a` attributes
- `k` skill tree
- `q` save and quit

### Dungeon

- `w/a/s/d` move or attack
- `1` Cleave
- `2` Shield Bash
- `3` Battle Cry
- `p` drink lesser health potion
- `i` inventory
- `Esc` return to town / back out of menus

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

## Design

See [`design.md`](design.md) for the detailed design document and roadmap.
