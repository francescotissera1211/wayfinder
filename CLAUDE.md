# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Optimized build
cargo check                    # Fast type-check without codegen
cargo clippy                   # Lint
cargo fmt                      # Format
cargo run                      # Run (opens home directory)
```

No test suite yet. All builds go through cargo.

## Architecture

Accessible GTK4 file manager in Rust. Designed to work without a desktop environment (tiled WM friendly). Accessibility via Orca screen reader is the primary design constraint.

### GObject Two-Module Pattern

Every GObject type uses a `mod.rs` / `imp.rs` split:
- **`mod.rs`** — public wrapper type via `glib::wrapper!`, constructor, public API
- **`imp.rs`** — `ObjectSubclass` impl with struct fields, `ObjectImpl::constructed()` for setup

### Data Flow

`gio::File` → `enumerate_children` → `ListStore<FileObject>` → `SortListModel` (dirs-first + alpha) → `FilterListModel` (hidden files) → `SingleSelection` → `ColumnView`

### Key Patterns

- **Accessible properties** use enum variants: `Property::Label("text")`, passed as `&[Property]` to `update_property()`. They are NOT GObject properties.
- **`announce()`** (GTK 4.14) for screen reader announcements on state changes.
- **Navigation shortcuts** use `GtkShortcutController` with `NamedAction` pointing to `gio::SimpleAction`s on the window. Action names use `nav.*` and `view.*` prefixes.
- **Window subclass `@implements`** must include `ConstraintTarget`, `Accessible`, `Buildable`, `Native`, `Root`, `ShortcutManager`.

### Module Responsibilities

- `app/` — Application subclass, creates windows
- `window/` — Main window: headerbar, content area, actions, navigation logic
- `file_object/` — GObject data model for a single file entry (`#[derive(Properties)]`)
- `file_model/` — Directory loading into ListStore, sorting, filtering
- `views/list_view/` — GtkColumnView with Name/Size/Modified/Kind columns
- `navigation/` — Back/forward/up history stack (plain Rust, not GObject)
- `shortcuts/` — Keyboard shortcut registration

### Dependencies

- gtk4 0.10 with `v4_14` feature (no libadwaita)
- dirs for XDG user directories
- chrono for date formatting
- open for xdg-open integration
