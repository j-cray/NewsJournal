# NewsJournal: Project & Agent Guidelines

NewsJournal is a pure Rust desktop application targeting Linux and macOS designed to help journalists stay organized while reporting. It provides article Kanban workflows across editorial stages, linked task management, source/contact directories with article tagging, and deadline tracking with overdue alerts. The UI features a vertical left toolbar, in-app glass modal/drawer editing, mouse drag-and-drop card interaction, and platform-tailored glass visuals (**COSMIC Frosted Glass** on Linux via `libcosmic` and **Liquid Glass** on macOS via `iced` + native `window_vibrancy`).

---

## 0. Project Overview & Architecture

### Key Subsystems & Workspace Structure
- **`crates/core` (`newsjournal-core`)**:
  - Pure Rust domain logic, entity models (`Article`, `Task`, `Contact`, `Settings`), strict slug/field validation.
  - SQLite persistence layer using `rusqlite` with embedded SQL schema migrations.
  - Deterministic color palette generator (hash-based, high-contrast, strictly excluding hard-to-read shades like light yellow).
  - Real-time deadline and overdue evaluation engine (`now > deadline` and stage != `Published`).
  - Comprehensive unit, integration, and property test suites.
- **`crates/gui` (`newsjournal-gui`)**:
  - Cross-platform desktop interface compiled with strict platform feature flags:
    - `#[cfg(target_os = "linux")]`: `libcosmic` application wrapper with COSMIC frosted glass containers and COSMIC theme integration.
    - `#[cfg(target_os = "macos")]`: `iced` application with `window_vibrancy` for macOS liquid glass / native vibrancy effects.
  - Left vertical navigation bar (Articles Kanban, Tasks Kanban, Contacts Directory, Settings).
  - Main Articles Kanban board (6 stages: Pitching, Researching, Writing, Editing, Ready to Publish, Published).
  - Tasks Kanban board (3 stages: To-Do, In Progress, Complete), color-coded to parent articles.
  - Contacts directory list with many-to-many article tagging.
  - In-app glass modal / slide-over drawer for creating and editing entities.
  - Theme engine supporting `System` (OS-inherited), `Light`, and `Dark` modes.

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

## 4. Living Documentation & Plan Tracking Requirements

Documentation must stay in sync with the codebase at all times. Whenever relevant during development, agents **must actively update**:

1. **`docs/initialPlan.md` (Primary Implementation Roadmap & Branch Workflow)**:
   - Agents **must follow `docs/initialPlan.md` sequentially** during implementation.
   - **Branch-Per-Task Execution**: Every task/subtask from `docs/initialPlan.md` **must be developed on a separate dedicated Git branch** (e.g., `task/<phase>-<subtask>-<short-description>` or `feature/<task-name>`) branched off `main`.
   - Never commit incomplete or unverified work directly to `main`.
   - After completing the implementation and passing the full verification loop (format, clippy, tests), merge the task branch back into `main`.
   - Immediately update `docs/initialPlan.md` and tick off the corresponding checkbox (`- [x]`) for the completed task.
   - If scope or design evolves, update the plan's specification and subtasks accordingly.
2. **`docs/roadmap.md`**:
   - Maintain a living roadmap tracking project milestones, milestone statuses, current sprint tasks, architectural decision records (ADRs), and backlog items.
3. **`README.md`**:
   - Update getting started steps, prerequisites, project overview, architectural summaries, and CLI/API usage examples whenever capabilities change.
4. **`GEMINI.md`** (This file):
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
