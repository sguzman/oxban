# Oxban

Oxban is a local-first Kanban desktop application built as a Rust workspace. It combines a shared Rust domain crate, a Tauri desktop backend, and a Yew frontend compiled to WebAssembly.

The project is useful both as a working Kanban app and as a reference architecture for a Rust-first desktop stack with:

- typed shared models across backend and frontend
- a narrow Tauri command boundary
- SQLite persistence with migrations
- deterministic ordering for columns and cards
- app-level configuration and structured logging

## What The App Does

Today the app supports a focused Kanban workflow:

- multiple boards
- board creation, rename, selection, and deletion
- default board bootstrapping when no boards exist
- per-board columns with create, rename, reorder, and delete flows
- cards with create, edit, move, search, and delete flows
- persisted local storage through SQLite
- drag-and-drop oriented board interactions in the Yew UI

The system is explicitly local-first. There is no network sync layer in this repository; application state lives on the local machine through the Tauri backend and SQLite database.

## Workspace Overview

This repository is a Cargo workspace with three members:

- `crates/oxban-core`: shared domain types and command argument payloads
- `src-tauri`: the desktop backend and Tauri shell
- `ui`: the Yew frontend compiled to WebAssembly and served through Trunk

Root [Cargo.toml](/win/linux/Code/rust/oxban/Cargo.toml:1) defines the workspace and sets `crates/oxban-core` and `src-tauri` as default members. The UI crate is part of the workspace but is not a default cargo target because it builds for `wasm32`.

## Architecture

Oxban is split into three layers with a deliberate boundary between them.

### 1. Shared Core

The shared crate in [crates/oxban-core/src/lib.rs](/win/linux/Code/rust/oxban/crates/oxban-core/src/lib.rs:1) defines:

- domain entities: `Board`, `Column`, `Card`
- aggregate snapshot types: `BoardState`, `BoardSummary`
- typed argument payloads for the Tauri command layer

This crate is intentionally small. It carries the types that both the desktop backend and the WebAssembly frontend must agree on.

### 2. Tauri Backend

The backend in [src-tauri/src/main.rs](/win/linux/Code/rust/oxban/src-tauri/src/main.rs:1) is responsible for:

- loading or initializing app configuration
- initializing logging
- opening the SQLite database and running migrations
- exposing Tauri commands to the UI
- handling process shutdown signals
- applying Linux Wayland runtime defaults for WebKit stability

The Tauri command surface is implemented in [src-tauri/src/commands.rs](/win/linux/Code/rust/oxban/src-tauri/src/commands.rs:1). Commands are thin wrappers over the database layer and return serialized Rust types to the frontend.

### 3. Yew Frontend

The UI in [ui/src/main.rs](/win/linux/Code/rust/oxban/ui/src/main.rs:1) and [ui/src/app.rs](/win/linux/Code/rust/oxban/ui/src/app.rs:1) provides the board interface. It:

- renders the application in the browser view embedded by Tauri
- loads board lists and board snapshots through Tauri invokes
- manages route state and local UI state
- supports modal interactions and drag/drop flows
- filters cards through a client-side search string

The frontend does not talk to SQLite directly. All persistence goes through Tauri commands.

## Data Model

The persistent model is small and centered around boards, columns, and cards.

### Boards

A board has:

- `id`
- `name`
- `created_at`
- `updated_at`

### Columns

A column belongs to a board and has:

- `id`
- `board_id`
- `name`
- `pos`
- timestamps

`pos` is an integer used for stable ordering.

### Cards

A card belongs to a board and a column and has:

- `id`
- `board_id`
- `column_id`
- `title`
- `description`
- `tags`
- optional `due_date`
- `priority`
- `pos`
- timestamps

Tags are stored in SQLite as JSON text and materialized as `Vec<String>` in Rust.

## Ordering Strategy

Column and card order is not stored as contiguous indexes. Instead, Oxban uses integer positions with configurable spacing.

Relevant logic lives in [src-tauri/src/positions.rs](/win/linux/Code/rust/oxban/src-tauri/src/positions.rs:1) and is used by the database layer in [src-tauri/src/db.rs](/win/linux/Code/rust/oxban/src-tauri/src/db.rs:1).

The approach is:

- new items are usually appended by adding a configurable `step`
- moved items are placed between neighboring positions when a gap exists
- if positions become too dense, the backend renormalizes them back to evenly spaced values

This avoids rewriting every sibling row on every move while keeping ordering deterministic.

## Persistence And Migrations

SQLite is the only persistence backend in this repository.

Database setup happens in [src-tauri/src/db.rs](/win/linux/Code/rust/oxban/src-tauri/src/db.rs:1), which:

- creates the parent directory for the database if needed
- opens a SQLite connection pool
- applies configured SQLite pragmas
- runs embedded SQLx migrations from `src-tauri/migrations`

The initial schema is in [src-tauri/migrations/0001_init.sql](/win/linux/Code/rust/oxban/src-tauri/migrations/0001_init.sql:1) and creates:

- `boards`
- `columns`
- `cards`
- indexes for ordered column and card lookups

Foreign keys cascade deletes from boards to columns and cards, and from columns to cards.

## Configuration

Oxban ships with a TOML config template at [oxban.toml](/win/linux/Code/rust/oxban/oxban.toml:1). On startup, the backend copies this default config into the app config directory if no user config exists yet.

Config parsing and defaults are implemented in [src-tauri/src/app_config.rs](/win/linux/Code/rust/oxban/src-tauri/src/app_config.rs:1).

The config is split into these sections:

- `app`: app name, default board name, and whether new boards get seeded columns
- `ui`: start route and UI flags
- `storage`: SQLite filename and pragma-related settings
- `ordering`: spacing and minimum gap for position-based ordering
- `logging`: log level and file logging behavior

Notable defaults:

- default board name: `My Board`
- seeded columns: `To do`, `Doing`, `Done`
- SQLite journal mode: `WAL`
- SQLite synchronous mode: `NORMAL`
- ordering step: `1000000`

If config parsing fails, the backend logs the error and falls back to built-in defaults.

## Logging

Logging is initialized in [src-tauri/src/logging.rs](/win/linux/Code/rust/oxban/src-tauri/src/logging.rs:1) with `tracing` and `tracing-subscriber`.

By default the app:

- logs to stdout with compact formatting
- also writes rolling daily log files when file logging is enabled
- uses a filter string from config, defaulting to `info`

The UI also initializes browser-side tracing via `tracing-wasm`.

## Tauri Command Surface

The backend currently exposes these commands:

- `get_effective_config`
- `list_boards`
- `create_board`
- `delete_board`
- `rename_board`
- `get_board`
- `create_column`
- `rename_column`
- `reorder_column`
- `create_card`
- `update_card`
- `move_card`
- `delete_card`
- `delete_column`

These commands form the main contract between `ui/` and `src-tauri/`.

## Frontend Behavior

The board page in [ui/src/components/board.rs](/win/linux/Code/rust/oxban/ui/src/components/board.rs:1) drives most of the user-facing behavior:

- it loads the board list on startup
- if there are no boards, it creates a default one
- it resolves the active board from the route or falls back to the first board
- it fetches a fresh board snapshot whenever the active board changes
- it manages search state, modal state, drag state, and theme toggling

Supporting UI state helpers live in [ui/src/state.rs](/win/linux/Code/rust/oxban/ui/src/state.rs:1). Routes are defined in [ui/src/routes.rs](/win/linux/Code/rust/oxban/ui/src/routes.rs:1).

The Tauri bridge in [ui/src/tauri_bridge.rs](/win/linux/Code/rust/oxban/ui/src/tauri_bridge.rs:1) wraps JavaScript interop and serializes typed Rust payloads for command invocation.

## Development Workflow

### Requirements

You need:

- a recent Rust toolchain
- the `wasm32-unknown-unknown` target
- `trunk`
- Tauri desktop prerequisites for your operating system
- `cargo-tauri` if you want to run the desktop shell directly from Cargo

### Common Commands

Check the Rust workspace default members:

```bash
cargo check --workspace
```

Check the UI crate for the WebAssembly target:

```bash
cargo check -p oxban-ui --target wasm32-unknown-unknown
```

Run the desktop app in development mode:

```bash
cargo tauri dev --manifest-path src-tauri/Cargo.toml
```

Build the frontend only with Trunk:

```bash
trunk build --config ui/Trunk.toml
```

### Tauri And Trunk Integration

Tauri is configured in [src-tauri/tauri.conf.json](/win/linux/Code/rust/oxban/src-tauri/tauri.conf.json:1).

Important details:

- dev frontend URL: `http://localhost:1420`
- frontend dist directory: `ui/dist`
- frontend commands are routed through [scripts/trunk-wrapper.sh](/win/linux/Code/rust/oxban/scripts/trunk-wrapper.sh:1)

The wrapper exists to normalize CLI flags that Tauri may pass through but Trunk does not parse compatibly.

## Project Layout

This is the current repository layout with the main responsibilities of each path.

### Root

- [Cargo.toml](/win/linux/Code/rust/oxban/Cargo.toml:1): workspace manifest
- [Cargo.lock](/win/linux/Code/rust/oxban/Cargo.lock:1): dependency lockfile
- [README.md](/win/linux/Code/rust/oxban/README.md:1): project documentation
- [LICENSE](/win/linux/Code/rust/oxban/LICENSE:1): MIT license
- [oxban.toml](/win/linux/Code/rust/oxban/oxban.toml:1): default app configuration template

### `crates/oxban-core/`

- [crates/oxban-core/Cargo.toml](/win/linux/Code/rust/oxban/crates/oxban-core/Cargo.toml:1): shared crate manifest
- [crates/oxban-core/src/lib.rs](/win/linux/Code/rust/oxban/crates/oxban-core/src/lib.rs:1): shared domain entities and command payload types

### `src-tauri/`

- [src-tauri/Cargo.toml](/win/linux/Code/rust/oxban/src-tauri/Cargo.toml:1): desktop application manifest
- [src-tauri/build.rs](/win/linux/Code/rust/oxban/src-tauri/build.rs:1): Tauri build integration
- [src-tauri/tauri.conf.json](/win/linux/Code/rust/oxban/src-tauri/tauri.conf.json:1): Tauri app configuration
- [src-tauri/capabilities/default.json](/win/linux/Code/rust/oxban/src-tauri/capabilities/default.json:1): default Tauri capability set
- [src-tauri/migrations/0001_init.sql](/win/linux/Code/rust/oxban/src-tauri/migrations/0001_init.sql:1): initial SQLite schema
- [src-tauri/src/main.rs](/win/linux/Code/rust/oxban/src-tauri/src/main.rs:1): backend startup and Tauri wiring
- [src-tauri/src/commands.rs](/win/linux/Code/rust/oxban/src-tauri/src/commands.rs:1): Tauri command handlers
- [src-tauri/src/db.rs](/win/linux/Code/rust/oxban/src-tauri/src/db.rs:1): SQLite access layer and domain persistence operations
- [src-tauri/src/app_config.rs](/win/linux/Code/rust/oxban/src-tauri/src/app_config.rs:1): configuration schema and bootstrap logic
- [src-tauri/src/logging.rs](/win/linux/Code/rust/oxban/src-tauri/src/logging.rs:1): tracing/logging setup
- [src-tauri/src/positions.rs](/win/linux/Code/rust/oxban/src-tauri/src/positions.rs:1): ordering helpers for columns and cards
- `src-tauri/gen/schemas/`: generated Tauri schemas
- `src-tauri/icons/`: desktop app icons

### `ui/`

- [ui/Cargo.toml](/win/linux/Code/rust/oxban/ui/Cargo.toml:1): frontend crate manifest
- [ui/index.html](/win/linux/Code/rust/oxban/ui/index.html:1): Trunk entry HTML
- [ui/Trunk.toml](/win/linux/Code/rust/oxban/ui/Trunk.toml:1): Trunk build config
- [ui/src/main.rs](/win/linux/Code/rust/oxban/ui/src/main.rs:1): Yew app bootstrap
- [ui/src/app.rs](/win/linux/Code/rust/oxban/ui/src/app.rs:1): top-level router wiring
- [ui/src/routes.rs](/win/linux/Code/rust/oxban/ui/src/routes.rs:1): route definitions
- [ui/src/state.rs](/win/linux/Code/rust/oxban/ui/src/state.rs:1): UI-only state helpers and filtering utilities
- [ui/src/tauri_bridge.rs](/win/linux/Code/rust/oxban/ui/src/tauri_bridge.rs:1): typed command invocation bridge
- `ui/src/components/`: board, column, card, and modal UI components
- `ui/assets/css/`: design tokens and page/component styles

### `scripts/`

- [scripts/trunk-wrapper.sh](/win/linux/Code/rust/oxban/scripts/trunk-wrapper.sh:1): wrapper used by Tauri to launch Trunk consistently

## Current Scope And Gaps

This repository already covers the full local desktop loop, but its current scope is still intentionally bounded.

What exists:

- local persistence
- board and card CRUD flows
- ordering and drag/drop support
- config bootstrap
- logging and migration setup

What does not appear to exist yet:

- sync or collaboration
- authentication or multi-user concepts
- server backend
- test suites in this repository
- packaging/bundling enabled by default in Tauri config

## Why The Split Matters

The main architectural value of the repository is the separation of concerns:

- `oxban-core` carries shared types and avoids duplicated contracts
- `src-tauri` owns persistence, configuration, and trusted system access
- `ui` owns rendering and local interaction logic

That split keeps the frontend thin, the backend explicit, and the shared data contract easy to reason about as the application grows.
