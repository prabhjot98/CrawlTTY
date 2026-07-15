# Homebrew Tap & Release Automation — Design

## Goal

Let people install CrawlTTY with `brew install`, and make future releases a
two-step manual action (bump version, push a tag) with everything else —
building, releasing, and updating the Homebrew formula — automated.

## Current state (as of this design)

- Rust binary crate `crawltty`, version `1.0.0` in `Cargo.toml`, no git tags,
  no GitHub Releases, no CI, no LICENSE file.
- Repo: `git@github.com:prabhjot98/CrawlTTY.git`.
- `crawltty` is an interactive full-screen TUI (crossterm/ratatui); it has no
  `--version` flag today, only a positional `reset-save` argument check.

## A. Main-repo changes (`CrawlTTY`)

1. **`LICENSE`** — MIT license, copyright prabhjot98. Required so public
   distribution via a Homebrew formula (`license "MIT"`) is on solid legal
   footing.
2. **`src/main.rs`** — add a `--version` flag alongside the existing
   `reset-save` check. Prints `env!("CARGO_PKG_VERSION")` and exits 0. This
   gives the Homebrew formula's `test do` block (and users) a non-interactive
   way to confirm the installed binary/version, since running the game itself
   requires a real TTY.
3. **`.github/workflows/release.yml`** — triggered on push of tags matching
   `v*`:
   1. Check out the tag.
   2. Verify the tag version matches `Cargo.toml`'s `version` field; fail the
      workflow on mismatch.
   3. `cargo build --release` on `macos-latest` as a pre-release sanity gate
      (matches the environment real `brew install` builds run in) — a build
      that doesn't compile never becomes a tagged release.
   4. Create a GitHub Release for the tag (title/body sourced from
      `CHANGELOG.md` where available).
   5. Trigger the formula bump (see section C).
4. First tag: `v1.0.0`, matching the current `Cargo.toml` version.

## B. Tap repo (`prabhjot98/homebrew-crawltty`)

New, separate GitHub repo containing only the formula:

```
homebrew-crawltty/
├── README.md               # what this tap is + install instructions
└── Formula/
    └── crawltty.rb
```

`Formula/crawltty.rb`:

```ruby
class Crawltty < Formula
  desc "Terminal-based action RPG / dungeon crawler"
  homepage "https://github.com/prabhjot98/CrawlTTY"
  url "https://github.com/prabhjot98/CrawlTTY/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "<computed at tag time>"
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

Notes:
- `depends_on "rust" => :build` is a build-time-only dependency — Homebrew
  provisions a Rust toolchain to compile, but it's not required at runtime and
  isn't a dependency of the installed binary.
- `std_cargo_args` is Homebrew's helper for `cargo install`, using the
  committed `Cargo.lock` (`--locked`) for reproducible builds.
- Building from source means no cross-compilation or prebuilt-binary CI is
  needed; it works on both Intel and Apple Silicon Macs automatically.

## C. Release automation (bump)

After `release.yml` creates the GitHub Release for a new tag, it triggers
`mislav/bump-homebrew-formula-action` (a well-established action for exactly
this workflow):

- Inputs: `formula-name: crawltty`, `homebrew-tap: prabhjot98/homebrew-crawltty`,
  `download-url` pointing at the tag's auto-generated source tarball
  (`https://github.com/prabhjot98/CrawlTTY/archive/refs/tags/v<version>.tar.gz`).
- The action downloads the tarball, computes its sha256, and updates the
  `url`/`sha256` (and formula version) in `Formula/crawltty.rb`.
- **Pushes directly to the tap repo's `main`** — no PR gate. This is a
  deliberate tradeoff: fully hands-off releases, at the cost of no review step
  before a new formula version goes live. A bad tag becomes a broken tap
  immediately.
- Auth: requires a fine-grained Personal Access Token scoped to only
  `homebrew-crawltty` (contents: read/write), stored as a secret (e.g.
  `HOMEBREW_TAP_TOKEN`) in the CrawlTTY repo, since the default `GITHUB_TOKEN`
  can't write to a different repo. Minting this token is a manual, one-time
  step done via the GitHub UI (or `gh` walked through interactively) — not
  something to script unattended.

## D. End-to-end release lifecycle & install UX

**Cutting a release, going forward:**
1. Bump `version` in `Cargo.toml`; move relevant `CHANGELOG.md` entries from
   "Unreleased" under the new version heading.
2. `git tag v<version> && git push origin v<version>`.
3. GitHub Actions builds/verifies, cuts the GitHub Release, and pushes the
   updated formula to `homebrew-crawltty` automatically.

**Installing, for users:**
```sh
brew tap prabhjot98/crawltty
brew install crawltty
crawltty
```
Homebrew resolves the short tap name `prabhjot98/crawltty` to
`github.com/prabhjot98/homebrew-crawltty` via its `homebrew-` prefix
convention. `brew upgrade crawltty` / `brew uninstall crawltty` work normally
thereafter.

## One-time bootstrap work (this implementation)

- Add `LICENSE`, add `--version` flag, add `.github/workflows/release.yml` to
  `CrawlTTY`.
- Create the `homebrew-crawltty` repo (via `gh repo create`) with the initial
  `README.md` and `Formula/crawltty.rb` (sha256 filled in against the real
  `v1.0.0` tarball once tagged).
- Create and push the `v1.0.0` tag.
- Set up the `HOMEBREW_TAP_TOKEN` secret (manual, credential-minting step).
- Verify end-to-end: `brew tap` + `brew install` from the fresh tap actually
  builds and runs `crawltty --version` successfully.

## Out of scope

- Linux/Linuxbrew support (not requested; can be added later since the
  build-from-source approach is platform-agnostic in principle, but untested
  here).
- Prebuilt binaries / cross-compilation CI.
- General CI (fmt/test/check on every push) beyond the pre-release sanity
  build described in section A.
