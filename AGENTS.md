# Agent Instructions

## Git workflow

After every completed code or documentation change in this repository:

1. Run the required pre-commit workflow before staging/committing:
   - `scripts/agent-commit-guard.sh --fix`
   - This runs `cargo fmt` first, then `cargo test`, then `cargo check`.
2. Review `git status --short` and `git diff` after the workflow. If `cargo fmt` changed files, include those changes in the same commit.
3. Make the final git commit only after the formatter and validation have successfully run. Never commit first and then run `cargo fmt` afterward.
4. Do not use `git commit --no-verify`. The project pre-commit hook enforces `cargo fmt --check`, `cargo test`, and `cargo check`.
5. Commit the change before starting the next task.
6. Use a concise, descriptive commit message.

Do not leave intentional source changes uncommitted unless the user explicitly says not to commit.

Do not commit local/generated files such as:

- `target/`
- `saves/`
- `.pi-lens/`

## UI interaction rule

Menu actions should execute immediately on a single keypress whenever possible. Do not add `pause` / "press any key to continue" prompts after routine actions such as healing, buying, selling, stashing, salvaging, upgrading, equipping, using items, or accepting simple menu commands. Reserve confirmation prompts only for destructive, irreversible, or ambiguous actions.
