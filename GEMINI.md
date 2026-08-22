# Rust Project & Agent Guidelines

This repository is a **Rust** project. Any agent or developer interacting with this codebase must adhere to the following development standards, verification protocols, architectural principles, and documentation practices.

---

## 1. Development & Testing Philosophy

- **Test-First Development (TDD)**: Write tests *before* or alongside implementation code wherever feasible. Define expected behaviors, boundaries, and failure modes through test cases first.
- **Liberal & Thorough Testing**: Test liberally and comprehensively across all layers:
  - Unit tests for internal logic, isolated functions, and state machines.
  - Integration tests for public APIs, subsystem interactions, and end-to-end workflows.
  - Boundary, negative, and edge-case testing (e.g., empty inputs, invalid formats, overflow limits, concurrency conditions).
  - Doc tests for public API examples to ensure documentation never becomes stale.

---

## 2. Mandatory Verification Loop

After **every single modification**, agents and developers must execute the verification loop and resolve all issues before considering any task complete:

```bash
# 1. Format code according to style rules
cargo fmt --all

# 2. Check for lint errors and compiler warnings (strict mode)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Execute all tests with nextest
cargo nextest run --all-targets --all-features
```

> [!IMPORTANT]
> - Never leave unresolved compiler warnings or Clippy lints.
> - Never skip or disable tests to make a build pass without explicit authorization.
> - If `cargo-nextest` is not available in the environment, fallback to `cargo test --all-targets --all-features`, but nextest is provided by the repository's Nix devshell.

---

## 3. Architecture & Code Organization

- **High Modularity**: Strongly prefer many small, single-responsibility files over monolithic files.
- **Short, Focused Snippets**: Keep functions, structs, and modules compact and easily understandable at a glance.
- **Explicit Hierarchy**: Organize code into clear module boundaries (`mod.rs` or named folder hierarchies). Separate domain logic, storage/IO, error types, and public interfaces into dedicated submodules.
- **Strong Typing & Error Handling**: Leverage Rust's type system to make invalid states unrepresentable. Use expressive error enums (e.g., `thiserror` or custom enums) rather than raw strings or untyped errors in library code.

---

## 4. Living Documentation Requirements

Documentation must stay in sync with the codebase at all times. Whenever relevant during development, agents **must actively update**:

1. **`README.md`**:
   - Update getting started steps, prerequisites, project overview, architectural summaries, and CLI/API usage examples whenever capabilities change.
2. **`docs/roadmap.md`**:
   - Maintain a living roadmap tracking project milestones, upcoming goals, current sprint tasks, architectural decision records (ADRs), and backlog items.
   - If the `docs/` directory or `docs/roadmap.md` does not yet exist when project planning or development begins, create it and document the project's trajectory.
3. **`GEMINI.md`** (This file):
   - Update this file whenever project-specific architectural patterns, specialized build flags, custom environment variables, new verification tooling, or updated agent instructions are established.

---

## 5. Development Environment (Nix & Direnv)

The repository provides a reproducible development environment powered by Nix Flakes and `direnv`:

- **Nix Flake (`flake.nix`)**: Delivers the latest nightly Rust toolchain with `rust-src`, `rust-analyzer`, `clippy`, `rustfmt`, `cargo-nextest`, and necessary system dependencies.
- **Direnv (`.envrc`)**: Automatically loads the Nix devshell when navigating into the repository directory (`direnv allow`).
- **Manual activation**: Run `nix develop` if not using direnv.

---

## 6. GitHub Actions & Automated AI PR Reviews

- **CI Pipeline (`.github/workflows/ci.yml`)**: Runs formatting, clippy linter, and nextest on pushes and pull requests.
- **Dependabot (`.github/dependabot.yml`)**: Automatically monitors and proposes updates for Cargo crates and GitHub Actions workflows weekly.
- **Automated & Summonable AI Reviews (`.github/workflows/ai-review.yml`)**:
  - Automatically reviews open and synchronized pull requests.
  - Can be manually summoned on any pull request by commenting:
    - `/review` (runs full AI review suite)
    - `/gemini-review` or `@gemini` (summons Google Gemini)
    - `/claude-review` or `@claude` (summons Claude)
    - `/copilot-review` or `@copilot` (summons GitHub Copilot)
