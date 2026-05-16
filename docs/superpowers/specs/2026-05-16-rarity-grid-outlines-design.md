# Rarity Grid Outlines Design

## Goal

Item-grid cells should show an outline color matching the contained item's rarity in every shared grid view, including the player bag and stash.

## Current Context

The bag and stash both render through `render_item_grid` in `src/inventory.rs`. Each cell currently renders as `[label]`, where the brackets form the visible cell outline and the label is the item kind glyph. Item names and selected-item details already use the shared `rarity_color` mapping: Common white, Magic blue, Rare yellow.

## Design

Occupied item-grid cells render their bracket outline with the contained item's rarity color. The center item label remains independently styled.

When the cursor selects an occupied cell, the brackets continue to use the item rarity color and the item label becomes bold green to communicate focus. This keeps rarity visible while making the focused cell obvious. Empty cells keep the existing empty-cell marker. A selected empty cell continues to use the existing focus styling because it has no item rarity.

The implementation should keep the current fixed-width text grid layout, so cell width, grid alignment, inventory/stash pane sizing, and command behavior do not change.

## Testing

Add a focused render/unit test for the cell-span helper or shared grid renderer that proves:

- Magic and Rare occupied cells use their rarity color on the bracket outline.
- A selected occupied cell keeps its rarity-colored outline.
- The selected occupied cell's label is bold green.
- Empty cells continue to render without a rarity outline.

Run the project commit guard after implementation.
