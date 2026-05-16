# Sockets and Gems Design

## Overview

Add a first-pass socket system focused on rare dropped gems and socketed dropped gear. The system should give the player long-term loot goals without adding crafting upgrades, gem combining, or rune effects yet.

The first implementation covers gems only. Runes remain a later feature.

## Goals

- Make some dropped gear more exciting by allowing sockets.
- Add rare gem drops that grant clear flat stat bonuses.
- Keep gem decisions reversible so players can experiment with builds.
- Use the existing list-based inventory, stash, blacksmith, town project, and equipment models.
- Keep terminal UI text short and readable.

## Non-Goals

- No blacksmith-added sockets.
- No purchased gems.
- No gem upgrading or combining.
- No runes.
- No resistances or elemental damage plumbing.
- No permanent gem destruction on removal or replacement.

## Feature Rules

- Gems are normal inventory items and can be moved to stash, sold, or selected at the Socket Bench.
- Each gem type maps to exactly one flat stat.
- Gems are slot-agnostic: any gem can go into any socketed weapon, armor, or shield.
- Gems have three drop-only tiers: Chipped, Flawed, and Pristine.
- Gems cannot be upgraded or combined.
- Socketed gear is found as dropped loot only.
- The blacksmith cannot add sockets.
- Socket Bench unlocks free gem insertion, removal, and replacement in town.
- Removed gems return to inventory.
- Replacing a gem removes the old gem back to inventory, then inserts the new one.
- Starter gear and vendor/basic fixed items have 0 sockets.

## Socket Counts

Socket count is rolled only for dropped weapons, armor, and shields.

| Rarity | Socket Roll |
| --- | --- |
| Common | 10% chance for 1 socket |
| Magic | 20% chance for 1 socket, 5% chance for 2 sockets |
| Rare | 25% chance for 1 socket, 10% chance for 2 sockets |

The 2-socket roll should be checked before the 1-socket roll for Magic and Rare gear so the listed odds are direct probabilities.

## Gem Stats

Each gem type grants one stat. Bloodstone weapon damage adds equally to weapon minimum and maximum damage.

| Gem | Stat | Chipped | Flawed | Pristine |
| --- | --- | ---: | ---: | ---: |
| Ruby | Max HP | +5 | +10 | +20 |
| Sapphire | Max mana | +3 | +6 | +12 |
| Garnet | Strength | +1 | +2 | +3 |
| Emerald | Dexterity | +1 | +2 | +3 |
| Amethyst | Intelligence | +1 | +2 | +3 |
| Quartz | Hit rating | +3 | +6 | +10 |
| Jade | Dodge rating | +2 | +4 | +8 |
| Onyx | Armor | +1 | +2 | +3 |
| Citrine | Speed | +2 | +4 | +7 |
| Topaz | Crit chance | +1% | +2% | +4% |
| Opal | Gold found | +5% | +10% | +20% |
| Bloodstone | Weapon damage | +1 | +2 | +3 |

## Drop Rules

- Gems start dropping on floor 3.
- Enemy loot has a 2% chance to be a gem.
- Chests have a 6% chance to include a gem.
- Elites have a 5% chance to drop a gem.
- Bosses have a 25% chance to drop a gem.
- Gem tiers use these weights: 80% Chipped, 17% Flawed, 3% Pristine.
- Gems can drop before Socket Bench is unlocked, but cannot be inserted, removed, or replaced until the project is complete.

## Data Model

Add `ItemKind::Gem`.

Add `GemKind`:

- Ruby
- Sapphire
- Garnet
- Emerald
- Amethyst
- Quartz
- Jade
- Onyx
- Citrine
- Topaz
- Opal
- Bloodstone

Add `GemTier`:

- Chipped
- Flawed
- Pristine

Add gem metadata to `Item`, used only when `kind == ItemKind::Gem`.

Add sockets to gear items. A socket should store either empty state or the inserted gem's kind and tier. Old saves should deserialize with no sockets and no gem metadata through serde defaults.

## Stat Calculation

Character stat methods include bonuses from gems inserted into equipped weapon, armor, and shield.

- Attribute gems increase effective Strength, Dexterity, and Intelligence before derived stats are calculated.
- Ruby and Sapphire add direct max HP and max mana after attribute-derived HP and mana.
- Quartz, Jade, Onyx, Citrine, Topaz, Opal, and Bloodstone add direct stat bonuses after existing base, attribute, gear, and skill contributions.
- Bloodstone adds to both weapon minimum and maximum damage.
- Topaz adds to effective equipped weapon crit chance.
- Opal increases gold found from monster and chest drops. It does not affect sell prices, quest rewards, or fixed project costs.

When socket changes reduce max HP or max mana below the current value, clamp current HP or mana to the new maximum.

## Socket Bench UI

The blacksmith menu gains a Socket Bench service.

- If `TownProject::SocketBench` is incomplete, selecting the service explains that the Socket Bench project is required.
- If complete, the service opens a socket management screen.
- The screen lists equipped gear and carried gear that has at least one socket.
- Selecting a socketed item shows each socket as empty or filled.
- Choosing an empty socket opens a gem picker from inventory.
- Choosing a filled socket offers remove or replace.
- Removing returns the gem to inventory for free.
- Replacing returns the old gem to inventory and consumes the selected new gem from inventory.
- Actions execute immediately on Enter, with no routine confirmation or pause prompt.

## Testing

Test coverage should include:

- Old saves deserialize with empty socket/gem fields.
- Socketed gear can drop with the expected socket count behavior through deterministic helper tests.
- Gem tier selection follows the 80/17/3 thresholds through deterministic helper tests.
- Gems are normal inventory items.
- Socket insertion consumes a gem from inventory.
- Socket removal returns the gem to inventory.
- Socket replacement returns the old gem and consumes the new gem.
- Equipped gem bonuses affect max HP, max mana, attributes, hit, dodge, armor, speed, crit, gold found, and weapon damage.
- HP and mana clamp when removing gems lowers maximums.
- Socket Bench is locked until `TownProject::SocketBench` is completed.

## Implementation Notes

Keep gem bonus logic centralized so item summary, character stats, combat, and gold drops use the same source of truth. Avoid adding rune-specific abstractions until runes are designed.
