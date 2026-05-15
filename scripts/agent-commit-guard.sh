#!/usr/bin/env bash
set -euo pipefail

usage() {
	cat <<'USAGE'
Usage: scripts/agent-commit-guard.sh [--fix]

Runs the required pre-commit workflow for this Rust repository.
  --fix   run cargo fmt before validation (for agents before staging/commit)
          without --fix, cargo fmt --check is used (for git hooks)
USAGE
}

mode="check"
if [[ "${1:-}" == "--fix" ]]; then
	mode="fix"
elif [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
	usage
	exit 0
elif [[ $# -gt 0 ]]; then
	usage >&2
	exit 2
fi

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

blocked_staged=$(git diff --cached --name-only | grep -E '^(target|saves|\.pi-lens)/' || true)
if [[ -n "$blocked_staged" ]]; then
	echo "Refusing to commit generated/local files:" >&2
	echo "$blocked_staged" >&2
	exit 1
fi

unstaged_tracked=$(git diff --name-only | grep -Ev '^(target|saves|\.pi-lens)/' || true)
if [[ -n "$unstaged_tracked" && "$mode" != "fix" ]]; then
	echo "Refusing partial commit with unstaged tracked changes:" >&2
	echo "$unstaged_tracked" >&2
	echo "Run scripts/agent-commit-guard.sh --fix, then stage all intentional changes." >&2
	exit 1
fi

if [[ "$mode" == "fix" ]]; then
	echo "==> cargo fmt"
	cargo fmt
else
	echo "==> cargo fmt --check"
	cargo fmt --check
fi

echo "==> cargo test"
cargo test

echo "==> cargo check"
cargo check
