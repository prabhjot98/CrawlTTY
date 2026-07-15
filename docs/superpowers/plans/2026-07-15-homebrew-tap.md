# Homebrew Tap & Release Automation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task (a fresh subagent per task). Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make CrawlTTY installable via `brew install` through a dedicated tap, with future releases requiring only a version bump + git tag push.

**Architecture:** CrawlTTY gains a `--version` flag, a `LICENSE`, and a `.github/workflows/release.yml` that on tag push builds a sanity check, cuts a GitHub Release, and pushes an updated formula to a separate `homebrew-crawltty` tap repo via `mislav/bump-homebrew-formula-action`. The tap repo holds a hand-authored `Formula/crawltty.rb` that builds from source with `cargo install` — no cross-compiled binaries.

**Tech Stack:** Rust/Cargo, GitHub Actions, Homebrew formula (Ruby), `gh` CLI.

## Global Constraints

- License: MIT.
- Formula builds from source: `depends_on "rust" => :build`, `cargo install` via `std_cargo_args`. No prebuilt binaries, no cross-compilation.
- Release sanity build runs on `macos-latest` (matches the environment `brew install` actually builds in).
- Tap repo: `prabhjot98/homebrew-crawltty`. Formula name: `crawltty`.
- The bump automation pushes directly to the tap's `main` branch — no PR gate (deliberate tradeoff, already approved).
- Auth for cross-repo push: a fine-grained PAT scoped only to `homebrew-crawltty` (contents: read/write), stored as secret `HOMEBREW_TAP_TOKEN` in the CrawlTTY repo.
- First tagged release: `v1.0.0`, matching the current `Cargo.toml` version.

---

### Task 1: Add MIT LICENSE

**Files:**
- Create: `LICENSE`

**Interfaces:**
- Produces: a `LICENSE` file at repo root, referenced by the tap formula's `license "MIT"` field (Task 3) as the license CrawlTTY is actually distributed under.

- [ ] **Step 1: Create the LICENSE file**

```text
MIT License

Copyright (c) 2026 prabhjot98

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 2: Verify it's in place**

Run: `test -f LICENSE && grep -q "MIT License" LICENSE && echo OK`
Expected: `OK`

- [ ] **Step 3: Commit**

```bash
git add LICENSE
git commit -m "Add MIT license"
```

---

### Task 2: Add `--version` flag to the binary

**Files:**
- Modify: `src/main.rs:53-59`
- Test: `src/main.rs` (inline `#[cfg(test)]` module, same file — this codebase keeps unit tests alongside source, matching the pattern already used for other modules in this crate)

**Interfaces:**
- Produces: `version_string() -> String` in `src/main.rs`, returning `"crawltty <CARGO_PKG_VERSION>"`. `main()` prints this and exits 0 when invoked with `--version` or `-V`, before the `saves` directory is created. Task 3's formula `test do` block relies on this flag's output containing the package version.

- [ ] **Step 1: Write the failing test**

Add to the bottom of `src/main.rs`:

```rust
#[cfg(test)]
mod version_tests {
    use super::version_string;

    #[test]
    fn version_string_has_expected_format() {
        let v = version_string();
        assert!(v.starts_with("crawltty "), "expected 'crawltty ' prefix, got {v}");
        let version_part = v.strip_prefix("crawltty ").unwrap();
        assert!(
            version_part.split('.').count() >= 2,
            "expected a semver-like version, got {version_part}"
        );
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test version_string_has_expected_format`
Expected: FAIL with `cannot find function 'version_string' in this scope`

- [ ] **Step 3: Implement `version_string()` and wire up the flag**

Replace `src/main.rs:53-59`:

```rust
fn main() -> Result<()> {
    fs::create_dir_all("saves").context("failed to create saves directory")?;

    if env::args().any(|arg| arg == "reset-save") {
        reset_saves()?;
        return Ok(());
    }
```

with:

```rust
fn main() -> Result<()> {
    if env::args().any(|arg| arg == "--version" || arg == "-V") {
        println!("{}", version_string());
        return Ok(());
    }

    fs::create_dir_all("saves").context("failed to create saves directory")?;

    if env::args().any(|arg| arg == "reset-save") {
        reset_saves()?;
        return Ok(());
    }
```

Add this function above `main()`:

```rust
fn version_string() -> String {
    format!("crawltty {}", env!("CARGO_PKG_VERSION"))
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test version_string_has_expected_format`
Expected: PASS (1 passed)

- [ ] **Step 5: Manually verify the CLI flag end-to-end**

Run: `cargo run --release -- --version`
Expected: prints `crawltty 1.0.0` and exits immediately (no TUI takeover, no `saves/` requirement).

- [ ] **Step 6: Run full test suite to check no regressions**

Run: `cargo test`
Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/main.rs
git commit -m "Add --version flag"
```

---

### Task 3: Create the `homebrew-crawltty` tap repo

**Files (in the new `homebrew-crawltty` repo, created locally then pushed):**
- Create: `README.md`
- Create: `Formula/crawltty.rb`

**Interfaces:**
- Consumes: nothing from this codebase directly; references `https://github.com/prabhjot98/CrawlTTY` as `homepage` and the not-yet-pushed `v1.0.0` tag as the source `url`.
- Produces: the `prabhjot98/homebrew-crawltty` GitHub repo with a formula whose `url`/`sha256` lines Task 5's automation will overwrite on the first tag push (the placeholder `sha256` below is intentionally wrong — it exists only so the automation's line has something to regex-replace).

- [ ] **Step 1: Create the repo via `gh`**

Run:
```bash
mkdir -p /tmp/homebrew-crawltty && cd /tmp/homebrew-crawltty
git init -b main
gh repo create prabhjot98/homebrew-crawltty --public --source=. --remote=origin
```
Expected: repo `prabhjot98/homebrew-crawltty` created and visible at `https://github.com/prabhjot98/homebrew-crawltty`.

- [ ] **Step 2: Write the tap README**

Create `/tmp/homebrew-crawltty/README.md`:

```markdown
# homebrew-crawltty

Homebrew tap for [CrawlTTY](https://github.com/prabhjot98/CrawlTTY), a
terminal-based action RPG / dungeon crawler.

## Install

\`\`\`sh
brew tap prabhjot98/crawltty
brew install crawltty
\`\`\`

## Usage

\`\`\`sh
crawltty
\`\`\`

`Formula/crawltty.rb` is updated automatically by CrawlTTY's release workflow
on every tagged release — no manual edits needed here under normal operation.
```

- [ ] **Step 3: Write the formula**

Create `/tmp/homebrew-crawltty/Formula/crawltty.rb`:

```ruby
class Crawltty < Formula
  desc "Terminal-based action RPG / dungeon crawler"
  homepage "https://github.com/prabhjot98/CrawlTTY"
  url "https://github.com/prabhjot98/CrawlTTY/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/crawltty --version")
  end
end
```

- [ ] **Step 4: Verify the formula is syntactically valid Ruby**

Run: `ruby -c /tmp/homebrew-crawltty/Formula/crawltty.rb`
Expected: `Syntax OK`

- [ ] **Step 5: Commit and push**

```bash
cd /tmp/homebrew-crawltty
git add README.md Formula/crawltty.rb
git commit -m "Add initial crawltty formula"
git push -u origin main
```
Expected: push succeeds; `Formula/crawltty.rb` visible on GitHub at `prabhjot98/homebrew-crawltty`.

---

### Task 4: Set up the `HOMEBREW_TAP_TOKEN` secret

This task mints a credential and cannot be done unattended — it requires the human to create a token in the GitHub UI. Do not attempt to paste the token value into any chat/log; the human runs the final command themselves.

**Files:** none (GitHub repo settings only)

**Interfaces:**
- Produces: a repo secret `HOMEBREW_TAP_TOKEN` on `prabhjot98/CrawlTTY`, consumed by Task 5's `release.yml` as `secrets.HOMEBREW_TAP_TOKEN`.

- [ ] **Step 1: Human creates a fine-grained PAT**

Ask the human to:
1. Go to https://github.com/settings/personal-access-tokens/new
2. Set "Resource owner" to their account, "Repository access" → "Only select repositories" → `homebrew-crawltty`.
3. Under "Repository permissions", set **Contents: Read and write**.
4. Generate the token and copy it (shown once).

- [ ] **Step 2: Human stores it as a repo secret, in their own terminal**

Ask the human to run this themselves (not through the assistant, so the token value never enters the conversation):
```bash
gh secret set HOMEBREW_TAP_TOKEN --repo prabhjot98/CrawlTTY
```
(`gh` will prompt for the value interactively — the human pastes the token there.)

- [ ] **Step 3: Verify the secret exists (name only, not value)**

Run: `gh secret list --repo prabhjot98/CrawlTTY`
Expected: a row named `HOMEBREW_TAP_TOKEN` appears.

---

### Task 5: Add the release workflow

**Files:**
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: `secrets.HOMEBREW_TAP_TOKEN` (Task 4), the `--version` flag (Task 2, exercised transitively by the tap formula's own test, not by this workflow directly), `Formula/crawltty.rb` existing in `prabhjot98/homebrew-crawltty` (Task 3).
- Produces: on push of any tag matching `v*`, a GitHub Release on CrawlTTY and an updated `Formula/crawltty.rb` commit on the tap repo's `main`.

- [ ] **Step 1: Write the workflow file**

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

jobs:
  release:
    runs-on: macos-latest
    steps:
      - name: Check out tag
        uses: actions/checkout@v4

      - name: Verify tag matches Cargo.toml version
        run: |
          TAG_VERSION="${GITHUB_REF_NAME#v}"
          CARGO_VERSION="$(grep -m1 '^version' Cargo.toml | sed -E 's/version = "(.*)"/\1/')"
          if [ "$TAG_VERSION" != "$CARGO_VERSION" ]; then
            echo "Tag version ($TAG_VERSION) does not match Cargo.toml version ($CARGO_VERSION)"
            exit 1
          fi

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build release binary (sanity check)
        run: cargo build --release --locked

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true

      - name: Bump Homebrew formula
        uses: mislav/bump-homebrew-formula-action@v3
        with:
          formula-name: crawltty
          homebrew-tap: prabhjot98/homebrew-crawltty
          download-url: https://github.com/prabhjot98/CrawlTTY/archive/refs/tags/${{ github.ref_name }}.tar.gz
          create-pullrequest: false
        env:
          COMMITTER_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
```

- [ ] **Step 2: Validate YAML syntax locally**

Run: `ruby -ryaml -e "YAML.load_file('.github/workflows/release.yml'); puts 'OK'"`
Expected: `OK`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "Add release workflow for tagged releases"
```

---

### Task 6: Cut the `v1.0.0` release

This is the task that actually publishes something public: pushing this tag fires the workflow, creates a public GitHub Release, and pushes a commit to the public tap repo. Confirm with the human immediately before the push step if anything about Tasks 1-5 changed since design approval.

**Files:** none (git tag + CI-driven changes only)

**Interfaces:**
- Consumes: Tasks 1, 2, 5 committed on `main`; Tasks 3 and 4 completed (tap repo + secret exist).
- Produces: git tag `v1.0.0` on CrawlTTY, a GitHub Release, and a real `sha256`/`url` in `prabhjot98/homebrew-crawltty`'s `Formula/crawltty.rb`.

- [ ] **Step 1: Push the tag**

```bash
git push origin main
git tag v1.0.0
git push origin v1.0.0
```
Expected: tag appears at `https://github.com/prabhjot98/CrawlTTY/releases/tag/v1.0.0` shortly after (once the workflow runs).

- [ ] **Step 2: Watch the workflow run**

Run: `gh run watch --repo prabhjot98/CrawlTTY $(gh run list --repo prabhjot98/CrawlTTY --workflow=release.yml --limit 1 --json databaseId --jq '.[0].databaseId')`
Expected: run completes with conclusion `success`.

- [ ] **Step 3: Verify the tap formula was actually updated**

Run: `curl -s https://raw.githubusercontent.com/prabhjot98/homebrew-crawltty/main/Formula/crawltty.rb | grep sha256`
Expected: a `sha256` line with a real 64-character hex digest — NOT the `0000...` placeholder from Task 3.

---

### Task 7: End-to-end install verification

**Files:** none (local Homebrew install only)

**Interfaces:**
- Consumes: the live tap from Task 6.
- Produces: a working local install proving the whole pipeline works for a real user.

- [ ] **Step 1: Tap and install**

```bash
brew tap prabhjot98/crawltty
brew install crawltty
```
Expected: builds from source (pulls in a Rust toolchain if not already present) and completes without error.

- [ ] **Step 2: Verify the installed binary**

Run: `crawltty --version`
Expected: `crawltty 1.0.0`

- [ ] **Step 3: Run brew's own audit/test as a second check**

```bash
brew audit --strict crawltty
brew test crawltty
```
Expected: both pass with no errors.
