# Agent Instructions

## Git workflow

After every completed code or documentation change in this repository:

1. Run the relevant formatter first:
   - Rust: `cargo fmt`
2. Run relevant validation:
   - Rust: `cargo test`, `cargo check`
3. Make the final git commit only after the formatter has successfully run. Never commit first and then run `cargo fmt` afterward.
4. Commit the change before starting the next task.
5. Use a concise, descriptive commit message.

Do not leave intentional source changes uncommitted unless the user explicitly says not to commit.

Do not commit local/generated files such as:

- `target/`
- `saves/`
- `.pi-lens/`

## UI interaction rule

Menu actions should execute immediately on a single keypress whenever possible. Do not add `pause` / "press any key to continue" prompts after routine actions such as healing, buying, selling, stashing, salvaging, upgrading, equipping, using items, or accepting simple menu commands. Reserve confirmation prompts only for destructive, irreversible, or ambiguous actions.
