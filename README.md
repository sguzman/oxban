# Oxban

Oxban is a local-first Kanban desktop app scaffold built from a Rust core, a Tauri shell, and a Yew frontend.

## Intent

Use a small Kanban product surface to explore desktop architecture concerns such as persistence, command boundaries, local-first state, and end-to-end tracing across backend and frontend.

## Ambition

The current repo looks like a scaffold with room to grow into a fuller task-board application, but its present value is also architectural: proving a Rust-first desktop stack with clear module boundaries.

## Current Status

Multiple-board workflows, persistence, migrations, search/filtering, and drag/drop scaffolding already exist. The README and code both present it as an experimental but functioning desktop app.

## Core Capabilities Or Focus Areas

- Rust domain/core crate shared across the app.
- Tauri backend with SQLite persistence and migrations.
- Yew frontend with board, column, and card workflows.
- Config bootstrap and tracing/logging support.
- Desktop-oriented local-first architecture.

## Project Layout

- `crates/oxban-core/`: shared domain models and command argument types used across app layers.
- `src-tauri/`: desktop backend, command handlers, storage layer, and migrations.
- `ui/`: Yew frontend, styling, and browser-side application code.
- `crates/`: workspace member crates grouped by subsystem.
- `scripts/`: helper scripts for development, validation, or release workflows.
- `Cargo.toml`: crate or workspace manifest and the first place to check for package structure.

## Setup And Requirements

- Rust toolchain.
- Tauri platform prerequisites.
- `trunk` and `tauri-cli` for frontend and desktop development.

## Build / Run / Test Commands

```bash
cargo check --workspace
cargo check -p oxban-ui --target wasm32-unknown-unknown
cargo tauri dev --manifest-path src-tauri/Cargo.toml
```

## Notes, Limitations, Or Known Gaps

- This is still framed as a scaffold, so polish and edge-case UX are not the main story yet.
- Desktop/frontend dependencies matter here more than in pure CLI projects.

## Next Steps Or Roadmap Hints

- Strengthen board editing ergonomics and local-first sync/export semantics.
- Keep the boundary between `oxban-core`, Tauri commands, and Yew UI clean as features grow.
