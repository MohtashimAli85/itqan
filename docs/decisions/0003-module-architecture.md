# 0003: Module architecture

Status: proposed (stub, written in step 2.0.1)

## Context
Phase 1 code is organised in layers (`commands/`, `domain/`, `db/`). Phase 2 turns Itqan into a platform of modules connected by one coach (PLAN.md section 16).

## To decide in step 2.0.1
- The `Module` trait signature: id, migrations, commands, published events, subscriptions, signals, settings, enabled flag.
- What lives in `itqan-contracts` and how events are versioned.
- The event bus: dispatch, ordering, and how fast handlers must be.
- Migrations per module: tracking applied versions, ordering, and upgrading existing databases without data loss.
- How tauri-specta collects commands and events from several crates.
- The frontend module manifest and the boundary lint.
