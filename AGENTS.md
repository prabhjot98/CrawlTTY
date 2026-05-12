# Agent Instructions

## Git workflow

After every completed code or documentation change in this repository:

1. Run relevant validation:
   - Rust: `cargo fmt`, `cargo test`, `cargo check`
2. Commit the change before starting the next task.
3. Use a concise, descriptive commit message.

Do not leave intentional source changes uncommitted unless the user explicitly says not to commit.

Do not commit local/generated files such as:

- `target/`
- `saves/`
- `.pi-lens/`
