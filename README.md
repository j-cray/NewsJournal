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
- 🧪 **Test-First & Modular Philosophy**: Detailed guidelines in [GEMINI.md](GEMINI.md) (and symlinked [AGENTS.md](AGENTS.md) / [CLAUDE.md](CLAUDE.md)) enforcing modularity, small files, and strict verification loops.
- 🤖 **Automated & Summonable AI PR Reviews**: GitHub Actions workflow (`.github/workflows/ai-review.yml`) for automated and on-demand PR reviews using Google Gemini, Anthropic Claude, and GitHub Copilot.
- 🔄 **Continuous Integration**: GitHub Actions CI (`.github/workflows/ci.yml`) running formatting, Clippy lints, and nextest test suites.
- 📦 **Automated Dependency Updates**: Dependabot (`.github/dependabot.yml`) configured for weekly grouped updates of Cargo dependencies and GitHub Actions.

---

## Workspace Structure

The project is structured as a Cargo workspace:

- **`crates/core` (`newsjournal-core`)**: Pure Rust domain logic, entity models, strict validation engine (URL-safe slugs, emails, E.164 phone numbers, names/headlines, and hex colors), accessible color palette generation, overdue calculations, and comprehensive generative property tests (`proptest`).
- **`crates/gui` (`newsjournal-gui`)**: Cross-platform desktop application interface (`newsjournal` binary).


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

### 2. Building and Running

```bash
# Build all workspace crates
cargo build --all-targets --all-features

# Run the desktop application
cargo run -p newsjournal-gui
```

### 3. Core Crate Example Usage

```rust
use newsjournal_core::color::{assign_color_for_slug, Color};
use newsjournal_core::storage::StorageService;
use newsjournal_core::{Article, ArticleStage, Contact, Task, TaskStatus};

// Initialize SQLite storage service in-memory (or open default app database on disk)
let service = StorageService::in_memory()?;
// Or: let service = StorageService::open_default()?; // uses OS app data directories

// Create and persist an article
let article = Article::new("city-budget-2026", "City Council Votes on Historic Transit Expansion")
    .with_stage(ArticleStage::Writing)
    .with_auto_color();
let created = service.create_article(article)?;

// Create an article-linked task
let task = Task::new(created.id, "Interview Transit Union President")
    .with_status(TaskStatus::InProgress);
service.create_task(task)?;

// Tag a contact to the story
let contact = Contact::new("Jane Doe").with_role("Spokesperson");
let created_contact = service.create_contact(contact)?;
service.link_contact_to_article(created.id, created_contact.id)?;
```

### 4. GUI Event Loop Example Usage

```rust
use newsjournal_core::models::Article;
use newsjournal_core::storage::StorageService;
use newsjournal_gui::{AppMessage, AppState, EventLoop, NavTab};

// Initialize GUI state wrapping storage
let storage = StorageService::in_memory()?;
let mut state = AppState::new(storage);
state.load_all()?;

let mut event_loop = EventLoop::new(state);

// Dispatch navigation and CRUD actions
event_loop.dispatch(AppMessage::NavigateTo(NavTab::ArticlesKanban))?;
event_loop.dispatch(AppMessage::CreateArticle(
    Article::new("port-expansion", "Port Expansion Cleared for Environmental Review")
))?;

assert_eq!(event_loop.state().articles.len(), 1);
```

### 5. Linux COSMIC Desktop Integration

```rust
use newsjournal_core::storage::StorageService;
use newsjournal_gui::cosmic::{CosmicApp, CosmicAppConfig, CosmicThemeAdapter, CosmicThemeMode, CosmicAccentColor};
use newsjournal_gui::{AppState, NavTab};

// Setup COSMIC app wrapper with frosted glass materials and responsive window layout
let storage = StorageService::in_memory()?;
let mut state = AppState::new(storage);
state.load_all()?;

let config = CosmicAppConfig {
    theme_adapter: CosmicThemeAdapter::new(
        CosmicThemeMode::System,
        CosmicAccentColor::CosmicBlue,
        true,
    ),
    ..Default::default()
};

let mut app = CosmicApp::with_config(state, config);
let view_tree = app.build_view_tree();

println!("COSMIC Window Title: {}", view_tree.window_title);
println!("Header Bar: {}", view_tree.header_bar.display_title());
```

### 6. macOS Liquid Glass & Vibrancy Integration

```rust
use newsjournal_core::storage::StorageService;
use newsjournal_gui::macos::{
    MacosApp, MacosAppConfig, MacosAppearanceMode, MacosThemeAdapter,
    MacosAccentColor, MacosVibrancyConfig, MacosVibrancyMaterial,
};
use newsjournal_gui::AppState;

// Setup macOS app wrapper with Liquid Glass materials and native NSVisualEffectView vibrancy
let storage = StorageService::in_memory()?;
let mut state = AppState::new(storage);
state.load_all()?;

let config = MacosAppConfig {
    theme_adapter: MacosThemeAdapter::new(
        MacosAppearanceMode::DarkAqua,
        MacosAccentColor::Purple,
        true,
    ),
    vibrancy: MacosVibrancyConfig::with_material(MacosVibrancyMaterial::UnderWindowBackground),
    ..Default::default()
};

let mut app = MacosApp::with_config(state, config);
let view_tree = app.build_view_tree();

println!("macOS Window Title: {}", view_tree.window_title);
println!("macOS Toolbar: {}", view_tree.toolbar.display_title());
```

### 7. Dynamic Theme Engine & System Appearance Detection

```rust
use newsjournal_core::models::ThemeMode;
use newsjournal_gui::theme::{SystemThemeDetector, ThemeEngine, ResolvedTheme};

// Initialize theme engine matching operating system appearance
let detector = SystemThemeDetector::new();
let mut engine = ThemeEngine::with_detector(ThemeMode::System, &detector);

println!("Resolved theme: {:?}", engine.resolved()); // ResolvedTheme::Dark or Light
println!("Text contrast ratio: {:.2}:1", engine.text_contrast_ratio()); // Meets WCAG AAA (>= 7.0:1)

// Toggle theme or override
engine.set_mode(ThemeMode::Light);
assert_eq!(engine.resolved(), ResolvedTheme::Light);
```

### 8. Left Navigation Bar & Keyboard Shortcuts

```rust
use newsjournal_gui::navigation::{resolve_nav_shortcut, NavKeyAction, NavKeyModifiers, NavTab};
use newsjournal_gui::views::build_nav_bar_view;
use newsjournal_gui::{AppMessage, AppState, EventLoop};

let state = AppState::in_memory()?;
let mut event_loop = EventLoop::new(state);

// Build view model with active highlighting, badge metrics, and overdue indicators
let nav_bar = build_nav_bar_view(event_loop.state());
assert_eq!(nav_bar.active_tab, NavTab::ArticlesKanban);
assert_eq!(nav_bar.main_items.len(), 3); // Articles, Tasks, Contacts
assert_eq!(nav_bar.docked_items.len(), 1); // Settings docked at bottom

// Keyboard shortcuts (e.g. Ctrl+1..4 on Linux, ⌘1..4 on macOS, or Arrow Down / Tab cycling)
let linux_ctrl = NavKeyModifiers::ctrl_or_meta(true, false);
if let Some(action) = resolve_nav_shortcut("2", linux_ctrl, false) {
    event_loop.dispatch(AppMessage::HandleNavKeyAction(action))?;
}
assert_eq!(event_loop.state().active_tab, NavTab::TasksKanban);
```

### 9. Settings Page, Theme Selector & Database Diagnostics

```rust
use newsjournal_core::models::ThemeMode;
use newsjournal_gui::views::build_settings_view;
use newsjournal_gui::{AppMessage, AppState, EventLoop};

let state = AppState::in_memory()?;
let mut event_loop = EventLoop::new(state);

// Build rich settings view model with live theme preview color swatches and database stats
let settings_view = build_settings_view(event_loop.state());
println!("Active Theme: {}", settings_view.theme_mode);
println!("Database Status: {}", settings_view.database.status_label);
println!("Schema Version: {}", settings_view.database.schema_version_formatted);

// Switch theme with immediate visual preview and persistence to SQLite
event_loop.dispatch(AppMessage::SetThemeMode(ThemeMode::Light))?;
assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::Light);
```

### 10. Articles Kanban Deck & Dynamic Article Cards

```rust
use newsjournal_core::models::Article;
use newsjournal_core::ArticleStage;
use newsjournal_gui::views::{build_articles_kanban_deck, DeckLayoutConfig};
use newsjournal_gui::{AppState, EventLoop};

let state = AppState::in_memory()?;
let mut event_loop = EventLoop::new(state);

// Render horizontal Kanban deck across 6 production stages:
// Pitching, Researching, Writing, Editing, Ready to Publish, Published
let deck = build_articles_kanban_deck(event_loop.state());
assert_eq!(deck.columns.len(), 6);
println!("Total Stories: {}", deck.total_article_count);
println!("Overdue Alerts: {}", deck.total_overdue_count);
```

### 11. Drag-and-Drop Movement Engine

```rust
use newsjournal_core::models::Article;
use newsjournal_core::ArticleStage;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::state::drag_drop::{DragItem, DropTarget};
use newsjournal_gui::views::build_articles_kanban_deck;
use newsjournal_gui::{AppState, EventLoop};

let state = AppState::in_memory()?;
let mut event_loop = EventLoop::new(state);

let article = Article::new("clean-energy", "Wind Power Grid Integration");
let article_id = article.id;
event_loop.dispatch(AppMessage::CreateArticle(article))?;

// 1. Pick up card with pointer coordinates (triggers dimmed source card & floating drag ghost)
event_loop.dispatch(AppMessage::DragStartWithPos {
    item: DragItem::ArticleCard { id: article_id, origin_stage: ArticleStage::Pitching },
    pos: (150.0, 200.0),
})?;

let active_deck = build_articles_kanban_deck(event_loop.state());
assert!(active_deck.is_dragging);
assert!(active_deck.drag_ghost.is_some());

// 2. Hover over destination column (displays dashed placeholder & emerald green highlight)
event_loop.dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(ArticleStage::Writing))))?;

// 3. Drop card into target column (persists to SQLite and recalculates deadlines)
event_loop.dispatch(AppMessage::DragDrop)?;
assert_eq!(event_loop.state().articles[0].stage, ArticleStage::Writing);
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

All agentic AI assistants (Gemini, Claude, Copilot, etc.) and human contributors follow the principles in [GEMINI.md](GEMINI.md):

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
