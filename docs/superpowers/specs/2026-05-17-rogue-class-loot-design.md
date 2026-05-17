# Rogue Class Loot Design

## Overview

Random equipment drops should become class-specific. A Rogue should receive Rogue equipment drops, and a Warrior should receive Warrior equipment drops. The current class-agnostic equipment table creates dead drops for Rogue because it only contains Strength-leaning Warrior gear.

Rogue loot should support an agile melee identity built around daggers, scimitars, light armor, and bucklers. Bows and other ranged Dexterity weapons are reserved for a future Ranger class and should not be added to Rogue loot.

## Goals

- Prevent random equipment drops from producing gear for another class.
- Give Rogue a clear Dexterity-oriented equipment progression path.
- Preserve the current Warrior gear identity by moving existing sword, axe, mail, and shield drops into the Warrior pool.
- Keep the first implementation small enough to fit the existing equipment drop cadence.
- Leave room for future Ranger loot without adding bows to Rogue.

## Non-Goals

- No shared cross-class random equipment drops in this pass.
- No bow or ranged weapon support for Rogue.
- No full loot rarity overhaul.
- No new inventory, stash, socket, or upgrade systems.
- No class respec or multi-class equipment support.

## Loot Pool Rules

Random equipment loot should route by the active character class:

- `Warrior` rolls only from Warrior equipment.
- `Rogue` rolls only from Rogue equipment.

The existing random equipment function should gain class context, such as `CharacterClass`, or be wrapped by a class-aware caller. The important behavior is that random equipment rewards know the player's class before choosing an item family.

If a future class has no implemented equipment pool, the code should fail visibly in development or route through an explicit placeholder path. It should not silently fall back to Warrior gear.

## Warrior Pool

The current equipment table becomes the Warrior pool:

- Iron Sword
- War Axe
- Mail Vest
- Guard Shield

These items should keep their Strength-forward requirements and heavier combat profile. This preserves Warrior progression while removing those items from Rogue drops.

## Rogue Pool

The first Rogue pool should mirror the current four-slot equipment cadence:

- Dagger
- Scimitar
- Light armor
- Buckler

This keeps equipment drop frequency familiar while making every random equipment drop relevant to the active class.

### Daggers

Daggers are the Rogue's fastest and most crit-oriented weapon family.

Recommended profile:

- Lower base damage than swords and axes.
- Positive speed modifier.
- Higher crit chance than swords.
- Dexterity-only requirement.
- Dexterity-focused damage scaling.

Daggers should feel ideal for combo-building and frequent attacks, not for single-hit base damage.

### Scimitars

Scimitars are the Rogue's heavier agile weapon family.

Recommended profile:

- Higher base damage than daggers.
- Lower crit or speed advantage than daggers.
- No heavy speed penalty.
- Dexterity primary requirement with a small Strength requirement.
- Dexterity-primary damage scaling with light Strength support.

Scimitars should be attractive when the player wants stronger regular hits without turning the Rogue into a Warrior.

### Light Armor

Light armor is the Rogue armor lane.

Recommended profile:

- Less armor than mail.
- No speed penalty.
- Dexterity requirement instead of Strength requirement.
- May include small speed, dodge, or Dexterity-friendly stat identity where the item system supports it.

Light armor should improve survival without undermining Rogue's fast, evasive class identity.

### Bucklers

Bucklers are the Rogue offhand lane.

Recommended profile:

- Less armor and block value than Guard Shield.
- Dexterity requirement.
- No heavy Strength requirement.
- May support dodge or speed identity instead of raw mitigation.

Bucklers should provide defensive progression for Rogue without making Guard Shield the expected endpoint.

## Balance Direction

Rogue item requirements should track Rogue's starting attributes and likely level-up path:

- Early Rogue drops should be reachable with high starting Dexterity and low Strength.
- Dagger and light armor requirements should not force Strength investment.
- Scimitars may ask for a small Strength investment, but Dexterity remains the primary gate.
- Bucklers should be usable by Rogues without becoming Warrior shields.

Item levels, rarity bonuses, sockets, and upgrade behavior should continue to work through the existing item systems.

## Implementation Shape

The implementation should introduce class-specific equipment generation without rewriting unrelated loot systems:

1. Split the current equipment table into a Warrior equipment generator.
2. Add a Rogue equipment generator with dagger, scimitar, light armor, and buckler entries.
3. Route random equipment drops through the active `CharacterClass`.
4. Update call sites that currently request random equipment without class context.
5. Add tests or focused assertions for class-specific drop behavior.
6. Update `DESIGN.md` when the behavior is implemented.

Existing gem, rarity, socket, salvage, stash, and inventory behavior should remain unchanged unless a direct compile or test issue requires a narrow adjustment.

## Testing

Verification should cover:

- Rogue random equipment rolls never produce Warrior equipment.
- Warrior random equipment rolls never produce Rogue equipment.
- Rogue item requirements match Rogue progression expectations.
- Existing Warrior equipment drops still produce the current Warrior item families.
- The required project guard passes before commit.

