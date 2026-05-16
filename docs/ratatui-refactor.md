# Ratatui refactor plan

Context7 source used: `/websites/rs_ratatui`.

## Ratatui patterns to use

- Ratatui apps follow this lifecycle: initialize the terminal, run a loop that draws a `Frame` and handles input events, then restore the terminal.
- Prefer `ratatui::try_init()?` / `ratatui::try_restore()?` for this codebase so terminal setup errors flow through `anyhow::Result`.
- Render from a single `terminal.draw(|frame| { ... })` pass per screen instead of mixing `println!`, ANSI cursor positioning, and raw-mode reads.
- Use `Layout` + `Constraint` to split screens into stable regions such as header, body, log, help, and footer.
- Use `Paragraph`, `Block`, `Line`, `Span`, and `Stylize` for styled text instead of embedding ANSI escape strings in display text.
- Continue using `crossterm::event::read()` for keyboard input, but do not enable/disable raw mode per keypress once Ratatui owns terminal lifecycle.

## Current code hotspots

- `src/main.rs` owns the top-level town loop and currently renders with `clear_screen()`, `print_town()`, `println!`, and `print_footer()`.
- `src/dungeon.rs` owns the dungeon loop, map rendering, combat log, skill help, footer, and low-level cursor-positioned helpers: `print_above_footer()`, `print_footer()`, and `print_colored_tile()`.
- `src/input.rs` enables raw mode for every key read. This should become an event adapter that assumes raw mode is already active during the Ratatui app loop.
- `src/ui.rs` contains text/ANSI helpers and should become the home for Ratatui rendering helpers and theme/style conversions.
- `src/inventory.rs`, `src/town.rs`, and `src/skills.rs` are menu-heavy and can migrate after the town and dungeon shells exist.
- `src/model.rs` contains ANSI color constants used throughout formatted strings. These should be phased out in favor of typed Ratatui styles.

## Recommended migration order

1. Add Ratatui dependency:

   ```toml
   ratatui = { version = "0.29", features = ["crossterm"] }
   ```

   Keep `crossterm` because Ratatui examples use it for input events.

2. Introduce an app terminal boundary:

   - Add an `AppTerminal`/`TerminalSession` wrapper that calls `ratatui::try_init()` on entry and `ratatui::try_restore()` on drop or explicit shutdown.
   - Update `main()` so normal gameplay runs inside this terminal session.
   - Keep `reset-save` outside Ratatui because it is a simple non-interactive command.

3. Refactor input:

   - Replace `RawModeGuard` in `src/input.rs` with a key event mapper.
   - Keep the existing public functions initially (`read_key_char()`, `read_key_char_nav()`) so menus can migrate incrementally.
   - Map `KeyCode::Up/Down` to existing `w/s` navigation while preserving Ctrl-C interruption behavior.

4. Build Ratatui render primitives in `src/ui.rs`:

   - Add style functions for semantic colors: hp, mana, gold, xp, warning, selected row, footer command, etc.
   - Add helpers to convert existing styled fragments to `Line`/`Span` only where needed.
   - Prefer new typed helpers over parsing ANSI strings.

5. Migrate the town shell first:

   - Replace the town loop's `clear_screen()` + `print_town()` + footer prints with `terminal.draw(render_town)`.
   - Use a vertical layout: title/status/equipment/quest/body/footer.
   - Keep town actions unchanged so this step is mainly presentation.

6. Migrate the dungeon screen next:

   - Replace `draw_dungeon()`, `print_skill_help()`, and `print_dungeon_footer()` with one `render_dungeon(frame, character)` function.
   - Suggested layout:
     - Header: act/floor/resources.
     - Body split horizontally: map left, combat log right or below depending on terminal width.
     - Help: skill cooldown/passive lines.
     - Footer: commands and legend.
   - Replace `print_colored_tile()` with `Span::styled()` per tile.
   - Replace `print_combat_log()` with a `Paragraph` or `List` backed by styled `Line`s.

7. Migrate modal/menu screens incrementally:

   - Inventory (`src/inventory.rs`): `List` for items, detail/comparison pane, footer commands.
   - Merchant/blacksmith/stash (`src/town.rs`): shared selectable-menu component.
   - Skill tree (`src/skills.rs`): sections per branch plus selected mastery/details pane.
   - Character creation (`src/save.rs`): may remain line-oriented initially because it uses text entry.

8. Remove old terminal helpers after all screens migrate:

   - Delete `clear_screen()`, `print_footer()`, `print_above_footer()`, and ANSI tile printing.
   - Replace ANSI constants in `src/model.rs` with Ratatui styles or keep only for save/log text if plain strings are still needed.

## Design constraints for this repository

- Preserve single-key menu execution. Do not add pause prompts after routine actions.
- Keep game-state logic separate from rendering. Screen functions should read state and emit widgets; action handlers should continue mutating `Character`.
- Avoid storing ANSI in game logs long-term. Prefer structured log kind + message for future Ratatui styling; as a transition, existing `[HIT]`, `[WARN]`, etc. prefixes can drive styles.
- Keep tests focused on game logic. Rendering helpers can be small pure functions returning `Line`/`Span` data where unit tests are useful, but full terminal drawing should remain thin.

## Minimal first PR target

A safe first implementation slice is:

1. Add `ratatui`.
2. Add terminal session setup/restore.
3. Refactor `src/input.rs` to stop toggling raw mode per keypress.
4. Convert only the main town screen and dungeon screen to Ratatui draws.
5. Leave inventory/town service/skill submenus on existing print helpers until the shared menu component exists.

This gets Ratatui into the core gameplay loop while limiting risk and keeping the current action code intact.
