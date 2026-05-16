# Agent Instructions

At the start of every conversation in this repository:
- Work directly on `main`.
- Do not create or use git worktrees for repository tasks unless the user explicitly asks for one.
- Before making code or documentation changes, check that the current checkout is on `main`. If it is not, switch to `main` when the working tree is clean.
- If the checkout cannot be switched to `main` because of uncommitted changes, an existing worktree checkout, conflicts, or another blocker, explain the blocker clearly before making changes.
- Do all repo inspection, edits, verification, staging, and commits on `main`.

Whenever you make a code change in a git repository:
- Run appropriate verification when practical.
- Keep DESIGN.md updated with the current game design and implementation status whenever gameplay, UI, data model, or system behavior changes.
- Stage only the files you changed for the requested task.
- Create a git commit before the final response.
- Do not commit unrelated user changes.
- If committing is impossible, explain why clearly.


## Git workflow

After every completed code or documentation change in this repository:

1. Run the required pre-commit workflow before staging/committing:
   - `scripts/agent-commit-guard.sh --fix`
   - This runs `cargo fmt` first, then `cargo test`, then `cargo check`.
2. Review `git status --short` and `git diff` after the workflow. If `cargo fmt` changed files, include those changes in the same commit.
3. Make the final git commit only after the formatter and validation have successfully run. Never commit first and then run `cargo fmt` afterward.
4. Do not use `git commit --no-verify`. The project pre-commit hook enforces `cargo fmt --check`, `cargo test`, and `cargo check`. If `git config --get core.hooksPath` is not `.githooks`, run `git config --local core.hooksPath .githooks` before committing.
5. Commit the change before starting the next task.
6. Use a concise, descriptive commit message.

## UI interaction rule

Menu actions should execute immediately on a single keypress whenever possible. Do not add `pause` / "press any key to continue" prompts after routine actions such as healing, buying, selling, stashing, salvaging, upgrading, equipping, using items, or accepting simple menu commands. Reserve confirmation prompts only for destructive, irreversible, or ambiguous actions.
