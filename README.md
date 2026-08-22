# NewsJournal

A pure Rust desktop application for journalists to organize story workflows, manage investigative beats, coordinate article tasks, track sources/contacts, and monitor publishing deadlines. Built with **COSMIC Frosted Glass** on Linux (`libcosmic`) and **Liquid Glass** on macOS (`iced` + native vibrancy).

---

## Documentation & Planning

- [Initial Implementation Plan](docs/initialPlan.md): Detailed 10-phase roadmap, architectural specifications, and data schemas.
- [Project Roadmap](docs/roadmap.md): Living milestones, Architectural Decision Records (ADRs), and backlog items.
- [Agent & Engineering Guidelines](GEMINI.md): Test-driven development rules and verification loop requirements.

---

## Features

- 🦀 **Reproducible Dev Environment**: Nix Flake (`flake.nix`) providing the latest nightly Rust toolchain, `rust-analyzer`, `clippy`, `rustfmt`, and `cargo-nextest`.
- ⚡ **Seamless Tooling**: `direnv` integration (`.envrc`) to auto-load the development environment.
- 🧪 **Test-First & Modular Philosophy**: Detailed guidelines in [GEMINI.md](file:///home/icarus/dev/templates/rust-project-template/GEMINI.md) (and symlinked [AGENTS.md](file:///home/icarus/dev/templates/rust-project-template/AGENTS.md) / [CLAUDE.md](file:///home/icarus/dev/templates/rust-project-template/CLAUDE.md)) enforcing modularity, small files, and strict verification loops.
- 🤖 **Automated & Summonable AI PR Reviews**: GitHub Actions workflow (`.github/workflows/ai-review.yml`) for automated and on-demand PR reviews using Google Gemini, Anthropic Claude, and GitHub Copilot.
- 🔄 **Continuous Integration**: GitHub Actions CI (`.github/workflows/ci.yml`) running formatting, Clippy lints, and nextest test suites.
- 📦 **Automated Dependency Updates**: Dependabot (`.github/dependabot.yml`) configured for weekly grouped updates of Cargo dependencies and GitHub Actions.

---

## Getting Started

### 1. Environment Setup (Nix & Direnv)

If you use [direnv](https://direnv.net/) and [Nix](https://nixos.org/):

```bash
# Allow direnv to load the flake devshell
direnv allow
```

Or manually start the Nix shell:

```bash
nix develop
```

### 2. Initializing Your Crate

This template is kept clean without a pre-baked crate structure. Initialize your project according to your needs:

```bash
# For a binary / CLI app
cargo init --bin

# For a library
cargo init --lib

# Or create a Cargo workspace with a root Cargo.toml
```

---

## Development & Verification Loop

After making changes, always run the mandatory quality loop:

```bash
# 1. Format code
cargo fmt --all

# 2. Check strict Clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run tests using cargo-nextest
cargo nextest run --all-targets --all-features
```

---

## AI Agent & Living Documentation Guidelines

All agentic AI assistants (Gemini, Claude, Copilot, etc.) and human contributors follow the principles in [GEMINI.md](file:///home/icarus/dev/templates/rust-project-template/GEMINI.md):

1. **Test-First**: Write unit, integration, and edge-case tests liberally before and alongside code.
2. **Modularity**: Prefer small, single-responsibility files and short functions over monolithic files.
3. **Living Documentation**:
   - Update `README.md` with new features, architecture changes, and usage instructions.
   - Maintain `docs/roadmap.md` (create `docs/` and `docs/roadmap.md` as planning begins) to track milestones and backlog items.
   - Update `GEMINI.md` when project-specific patterns, build steps, or conventions change.

---

## AI PR Reviews & Summoning Commands

Automated reviews run on open/synchronized pull requests. You can also summon specific AI reviewers on any pull request comment:

| Command | Reviewer Triggered |
| :--- | :--- |
| `/review` or `/ai-review` | Summons full AI review suite (Gemini, Claude, Copilot) |
| `/gemini-review` or `@gemini` | Summons **Google Gemini** |
| `/claude-review` or `@claude` | Summons **Anthropic Claude** |
| `/copilot-review` or `@copilot` | Summons **GitHub Copilot** |

### Required Repository Secrets
- `GEMINI_API_KEY`: API key for Google Gemini model reviews.
- `ANTHROPIC_API_KEY`: API key for Anthropic Claude model reviews.
- `GITHUB_TOKEN`: Standard repository token provided automatically by GitHub Actions.
