# Town Gold Progression Design

## Goal

Give gold a distinct long-term role by making it the currency for permanent town and world progression. Gold should not be a generic ingredient for direct gear power. Gear stat progression should stay on shard and material systems, while gold rebuilds the services that make those systems available.

## Core Economy Rule

- Gold is spent only on permanent town/world upgrades.
- Gear stat upgrades use gear shards and other future materials, not gold.
- Potion crafting uses herbs and other crafting materials, not gold.
- Gold can unlock the infrastructure for a service, but the service's item power should use its own material track.

This keeps each resource mutually exclusive and readable:

- Gold rebuilds town capability.
- Shards improve weapons, armor, and shields.
- Herbs support potion crafting.
- Gems, runes, or relics can support future socket and crafting systems.

## Town Projects Board

Add a central Town Projects board or menu in town. It lists available, completed, and locked projects. Projects are grouped by NPC or service so the player understands who benefits from the investment.

Projects can become available through:

- Rescuing or meeting an NPC.
- Completing main quests.
- Defeating bosses.
- Reaching act or floor milestones.

Completing a project spends the full gold cost from carried gold and permanently improves the world. There is no separate town fund and no partial project payment in the initial design.

## Project Tracks

### Smith

The smith track governs gear service infrastructure, but gold does not directly improve gear stats.

- **Rebuild the Forge**: unlock salvage and shard-only gear upgrades.
- **Reinforced Anvil**: improve salvage yield or raise shard upgrade caps.
- **Socket Bench**: unlock future socket systems. Actual socketing should use gems, runes, or other materials instead of gold.

### Quartermaster

The quartermaster track focuses on storage. Merchant stock expansion is intentionally out of scope for this design.

- **Storehouse Shelves**: expand stash capacity or add stash pages if stash capacity becomes limited.

### Appraiser

The appraiser track improves the player's ability to convert unwanted loot into gold without selling direct power.

- **Hire Appraiser**: improve sell prices, for example from 25% of item value to 30%.

Gambling, mystery items, and other gold-for-gear services are intentionally excluded because they blur the rule that gold upgrades the town rather than buying gear power.

### Alchemist

The alchemist track creates a non-gold crafting loop for consumables.

- **Herb Garden**: unlock growing herbs.
- **Distillery**: unlock potion crafting. Crafting costs should come from herbs and other materials, not gold.

## Gameplay Loop

1. Player earns gold from monsters, chests, quests, and selling loot.
2. Player spends gold on a town project.
3. The completed project permanently expands town capability.
4. The new or improved service gives the player better ways to use non-gold resources.
5. Better preparation and progression help the player push deeper and earn more gold.

The intended loop is "spend gold to unlock better systems that create more meaningful future resource decisions," not "spend gold for immediate stat increases."

## Existing Economy Cleanup

The current blacksmith upgrade flow spends both shards and gold. To match this design, gear upgrades should become shard-only or material-only as part of the town progression implementation. If a cost is needed to pace upgrades, use more shards, type-specific shards, or a future crafting material rather than gold.

Existing potion and basic gear purchases should be treated as transitional MVP behavior. Once Town Projects owns the gold economy, direct gold purchases of combat consumables or gear should be removed, converted to non-gold material systems, or replaced by town projects that unlock access to those systems.

## UI And Feedback

The Town Projects board should show:

- Current gold.
- Project name.
- NPC or service group.
- Gold cost.
- Availability state: available, completed, or locked.
- Short unlock result.
- Lock reason for unavailable projects when useful.

Routine project purchases should resolve immediately on a single keypress. Confirmation prompts should be reserved for destructive, irreversible, or ambiguous actions.

## Save Model

Persist completed town projects as world progression on the character save for now, since the game currently has a single save file model. A future account-wide profile can move these flags out of the character if multiple characters share a world.

Project state should be forward-compatible by using stable project identifiers rather than deriving completion from display names.

## Tests

Add focused tests when implementing:

- Available projects reflect quest, boss, and NPC gates.
- Completing a project subtracts gold and records completion.
- Completed projects cannot be purchased again.
- Insufficient gold leaves project state unchanged.
- Project benefits are applied by completed project identifiers.
- Gear upgrade costs no longer require gold once the cleanup is implemented.
