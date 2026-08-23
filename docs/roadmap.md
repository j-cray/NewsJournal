# NewsJournal Project Roadmap

This document serves as the living roadmap tracking project milestones, current progress, architectural decision records (ADRs), and upcoming backlog items.

---

## 🎯 Project Overview & Vision

**NewsJournal** is a modern, pure Rust desktop application designed specifically to empower journalists with article workflow management, investigative story tracking, task delegation, contact directories, and deadline awareness.

---

## 📍 Milestones & Status

| Phase | Description | Status | Target Platforms |
| :--- | :--- | :--- | :--- |
| **Phase 1** | Workspace Setup & Core Domain Models (`newsjournal-core`) | ✅ Complete | All |
| **Phase 2** | SQLite Persistence Engine & Migrations | ✅ Complete | All |
| **Phase 3** | GUI Scaffolding & Cross-Platform Glass Architecture | ✅ Complete | Linux & macOS |
| **Phase 4** | Vertical Left Navigation Bar & Settings / Theme Engine | ✅ Complete | Linux & macOS |
| **Phase 5** | Articles Kanban Board (6 Production Stages) & Drag-and-Drop | ✅ Complete | Linux & macOS |
| **Phase 6** | Article Creation & Edit Glass Modal / Drawer | ✅ Complete | Linux & macOS |
| **Phase 7** | Tasks Kanban Board & Article-Task Synchronization | ✅ Complete | Linux & macOS |
| **Phase 8** | Contacts Directory & Article Tagging System | ✅ Complete | Linux & macOS |
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

### ADR 6: Dynamic System Appearance Detection & Contrast Enforcement
- **Decision**: Provide cross-platform system theme detection (XDG portal/COSMIC and macOS defaults) with WCAG AAA contrast ratio enforcement.
- **Rationale**: Ensures the application seamlessly mirrors OS dark/light mode transitions while maintaining strict readability for journalists working in high or low light environments.

---

## 📋 Backlog & Upcoming Work
- [x] Initialize Cargo workspace with `newsjournal-core` and `newsjournal-gui`.
- [x] Implement core domain entities (`Article`, `Task`, `Contact`, `ArticleContact`, `Settings`) and stage/status transitions.
- [x] Implement strict slug, contact, and field validation engine.
- [x] Implement deterministic color palette generator avoiding unreadable colors (such as light yellow).
- [x] Implement real-time deadline evaluation, overdue tracking engine, and batch urgency summary engine.
- [x] Implement comprehensive generative property-based tests (`proptest`) and state machine test suites.
- [x] Implement `StorageService` CRUD repository layer with transactional integrity.
- [x] Setup cross-platform app directory resolution (`directories` crate) and environment overrides (`NEWSJOURNAL_*`).
- [x] Implement Phase 2.4 integration test suite for full relational cascades and persistence lifecycle.
- [x] Implement `newsjournal-gui` scaffolding with shared `AppState`, `AppMessage`, reducer, and event loop (Task 3.1).
- [x] Implement Linux COSMIC desktop wrapper, frosted glass materials, header bar, and window sizing (Task 3.2).
- [x] Implement macOS Liquid Glass integration with `iced` + `window_vibrancy` (Task 3.3).
- [x] Implement theme engine integration with dynamic System/Light/Dark switching (Task 3.4).
- [x] Implement Left Navigation Bar component, active tab highlighting, badge metrics, and keyboard shortcuts (Task 4.1).
- [x] Implement Settings Page with theme mode selection, database info, and persistence (Task 4.2).
- [x] Implement Articles Kanban Board deck layout with 6 production stage columns and horizontal scrolling (Task 5.1).
- [x] Implement Article Card Component with color accents, task counter, contact pills, and overdue badges (Task 5.2).
- [x] Implement Drag-and-Drop Movement Engine with stage transitions and visual drop targets (Task 5.3).
- [x] Implement Column empty states, card count badges, deck toolbar, and creation triggers (Task 5.4).
- [x] Implement Glass Modal / Drawer Container with centered & slide-over layouts and backdrop blur (Task 6.1).
- [x] Implement Article Form Fields with slug collisions, stage dropdown, and deadline picker (Task 6.2).
- [x] Implement Contact Tagging Sub-Section in article drawer (Task 6.3).
- [x] Implement Inline Task Management Sub-Section in article drawer (Task 6.4).
- [x] Implement Save & Cancel actions with keyboard shortcuts and validation tooltips (Task 6.5).
- [x] Implement Tasks Kanban Deck Layout (3-column workflow board: To-Do, In Progress, Complete) (Task 7.1).
- [x] Implement Task Card Component with accent strips, slug badges, due dates, notes previews, and overdue cues (Task 7.2).
- [x] Implement Task Drag-and-Drop Movement Engine across To-Do, In Progress, and Complete columns (Task 7.3).
- [x] Implement Task Creation & Editing Drawer with live parent article synchronization (Task 7.4).
- [x] Implement Contacts Directory List View with multi-column sorting and multi-field search (Task 8.1).
- [x] Implement Contact Creation & Editing Drawer with live validation and associated stories (Task 8.2).
- [x] Implement Direct Creation Button ("Add Contact") in top action bar and platform toolbars (Task 8.3).
- [x] Implement Contact Deletion & Unlink Safety with active story link warnings (Task 8.4).


