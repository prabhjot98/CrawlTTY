# Critical Hits Design

## Goal

Add player-only critical hits that occasionally deal double damage. Critical chance is determined by the equipped weapon's base type. Enemies cannot critically hit.

## Rules

- Only player weapon attacks can critically hit.
- Enemy melee attacks, enemy ranged attacks, boss specials, bleed, Spiked Guard damage, and other non-player effects cannot critically hit.
- A critical hit can only happen after a player attack has already passed its normal hit roll.
- Critical hits deal double final post-armor damage.
- Critical chance is a flat percentage by weapon base type:
  - Swords: 8%
  - Axes: 5%
- Rarity does not affect critical chance.
- Weapon upgrades do not affect critical chance.

## Item Model

Add a persisted `crit_chance: u32` field to `Item`, interpreted as percentage points.

Existing saves should remain compatible by defaulting missing `crit_chance` values to `0`. Newly created weapons should always set an explicit crit chance. Starting weapons and generated loot should use helpers so all swords and axes consistently receive their base-type crit chance.

## Combat Flow

The shared player damage path should handle critical hits so basic attacks, Cleave, and Shield Bash all support crits without duplicating logic.

The order of operations should be:

1. Roll normal hit chance.
2. If the attack misses, stop with the existing miss behavior.
3. Roll critical chance from the equipped weapon.
4. Roll base weapon damage.
5. Apply skill multipliers and Battle Cry multiplier.
6. Apply Vulnerable bonus.
7. Apply enemy armor with the existing minimum 1 damage rule.
8. If critical, double the resulting damage.
9. Apply damage, death handling, bleeding, loot, and skill side effects through the existing flow.

Cleave should roll critical hits independently per target.

## UI And Feedback

Weapon summaries should display critical chance, for example:

```text
dmg 3-5 crit 8% value 20
```

Weapon comparisons should include crit chance alongside damage so players can evaluate weapon swaps.

Combat log lines should make critical hits obvious:

```text
Critical hit! You hit Skeleton for 14 damage. HP 0/10.
Critical hit! You cleave Rat for 6 damage and kill it. +8 XP, +3 gold.
```

Non-critical hit and kill messages should keep their current wording.

## Tests

Add focused tests for:

- Weapon crit chance helpers assign 8% to swords and 5% to axes.
- Rarity does not change weapon crit chance.
- Item summaries and weapon comparisons display crit chance.
- Enemy damage paths do not expose or use critical hits.

The random crit roll itself can remain probabilistic in production code. Tests should avoid depending on a random crit by covering deterministic helpers and the absence of enemy crit behavior.
