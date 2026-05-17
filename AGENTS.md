# Agent Instructions

At the start of every conversation in this repository:
- Treat `main` as the integration branch, not the working branch.
- Before making code or documentation changes, check that the repository has a clean `main` checkout available. If the current checkout is not `main`, switch to `main` when the working tree is clean.
- If `main` cannot be checked out or is not clean because of uncommitted changes, an existing worktree checkout, conflicts, or another blocker, explain the blocker clearly before making changes.
- Create a dedicated git worktree for the task from `main`, using a concise branch name with the `codex/` prefix unless the user requests another name.
- Do all repo inspection, edits, verification, staging, and task commits inside that task worktree.
- After the task worktree commit is complete and verified, merge the task branch back into `main`.
- Do not make task changes directly on `main` except to perform the final merge from a verified task branch.
- After a successful merge, remove the task worktree when practical and leave `main` checked out.

Whenever you make a code change in a git repository:
- Run appropriate verification when practical.
- Keep DESIGN.md updated with the current game design and implementation status whenever gameplay, UI, data model, or system behavior changes.
- Add a concise line to CHANGELOG.md whenever a gameplay change is made.
- Stage only the files you changed for the requested task.
- Create a git commit before the final response.
- Do not commit unrelated user changes.
- If committing is impossible, explain why clearly.


## Git workflow

After every completed code or documentation change in this repository:

1. In the task worktree, run the required pre-commit workflow before staging/committing:
   - `scripts/agent-commit-guard.sh --fix`
   - This runs `cargo fmt` first, then `cargo test`, then `cargo check`.
2. Review `git status --short` and `git diff` after the workflow. If `cargo fmt` changed files, include those changes in the same commit.
3. Make the task worktree commit only after the formatter and validation have successfully run. Never commit first and then run `cargo fmt` afterward.
4. Do not use `git commit --no-verify`. The project pre-commit hook enforces `cargo fmt --check`, `cargo test`, and `cargo check`. If `git config --get core.hooksPath` is not `.githooks`, run `git config --local core.hooksPath .githooks` before committing.
5. Switch back to `main`, merge the task branch, and verify `main` now contains the task commit.
6. Remove the task worktree after the merge when practical.
7. Commit and merge the change before starting the next task.
8. Use a concise, descriptive commit message.

## UI interaction rule

Menu actions should execute immediately on a single keypress whenever possible. Do not add `pause` / "press any key to continue" prompts after routine actions such as healing, buying, selling, stashing, salvaging, upgrading, equipping, using items, or accepting simple menu commands. Reserve confirmation prompts only for destructive, irreversible, or ambiguous actions.
