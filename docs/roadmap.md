# NewsJournal Project Roadmap

This document serves as the living roadmap tracking project milestones, current progress, architectural decision records (ADRs), and upcoming backlog items.

---

## 🎯 Project Overview & Vision

**NewsJournal** is a modern, pure Rust desktop application designed specifically to empower journalists with article workflow management, investigative story tracking, task delegation, contact directories, and deadline awareness.

---

## 📍 Milestones & Status

| Phase | Description | Status | Target Platforms |
| :--- | :--- | :--- | :--- |
| **Phase 1** | Workspace Setup & Core Domain Models (`newsjournal-core`) | 🔄 In Progress | All |
| **Phase 2** | SQLite Persistence Engine & Migrations | ⏳ In Planning | All |
| **Phase 3** | GUI Scaffolding & Cross-Platform Glass Architecture | ⏳ In Planning | Linux & macOS |
| **Phase 4** | Vertical Left Navigation Bar & Settings / Theme Engine | ⏳ In Planning | Linux & macOS |
| **Phase 5** | Articles Kanban Board (6 Production Stages) & Drag-and-Drop | ⏳ In Planning | Linux & macOS |
| **Phase 6** | Article Creation & Edit Glass Modal / Drawer | ⏳ In Planning | Linux & macOS |
| **Phase 7** | Tasks Kanban Board & Article-Task Synchronization | ⏳ In Planning | Linux & macOS |
| **Phase 8** | Contacts Directory & Article Tagging System | ⏳ In Planning | Linux & macOS |
| **Phase 9** | Real-Time Deadline Tracking & Overdue Alert Engine | ⏳ In Planning | Linux & macOS |
| **Phase 10** | Platform Packaging, Cross-Compilation & Verification | ⏳ In Planning | Linux (COSMIC) & macOS |

---

## 🏛️ Architectural Decision Records (ADRs)

### ADR 1: Modular Workspace Architecture
- **Decision**: Split codebase into `newsjournal-core` (pure Rust domain logic, database queries, models, state machine) and `newsjournal-gui` (desktop GUI binary).
- **Rationale**: Keeps business logic 100% testable in headless CI/CD, decouples persistence from UI rendering, and cleanly separates platform-specific GUI implementations.

### ADR 2: Strict Platform Feature Compilation for GUI Glass Effects
- **Decision**: Use `#[cfg(target_os = "linux")]` with `libcosmic` (leveraging COSMIC frosted glass and styling) and `#[cfg(target_os = "macos")]` with `iced` + `window_vibrancy` (macOS native liquid glass/vibrancy).
- **Rationale**: Delivers authentic native experiences on both System76 COSMIC desktop and Apple macOS without compromising either platform's visual idioms.

### ADR 3: SQLite Storage with `rusqlite`
- **Decision**: Use local embedded SQLite database with SQL migrations for articles, tasks, contacts, tags, and settings.
- **Rationale**: Provides ACID guarantees, relational integrity for contact tags and task parent links, and local file storage without external daemon dependencies.

### ADR 4: Drag-and-Drop Kanban Interaction
- **Decision**: Card movement between Kanban columns is handled purely through mouse click-and-drag interactions.
- **Rationale**: Delivers a direct, tactile Kanban workflow aligned with modern editorial desktop tooling.

### ADR 5: In-App Glass Modal / Slide-over Drawer
- **Decision**: Display creation and editing forms for articles, tasks, and contacts in layered frosted/liquid glass modal overlays instead of separate native OS windows.
- **Rationale**: Prevents window management clutter on tiling and floating window managers alike and keeps the user focused on the active story.

---

## 📋 Backlog & Upcoming Work
- [x] Initialize Cargo workspace with `newsjournal-core` and `newsjournal-gui`.
- [x] Implement core domain entities (`Article`, `Task`, `Contact`, `ArticleContact`, `Settings`) and stage/status transitions.
- [x] Implement strict slug, contact, and field validation engine.
- [x] Implement deterministic color palette generator avoiding unreadable colors (such as light yellow).
- [ ] Implement SQLite schema migrations and CRUD repository with unit tests.
- [ ] Setup `libcosmic` Linux and `iced` macOS window targets.


