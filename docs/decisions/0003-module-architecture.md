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
- Owners for Phase 1 tables the module list does not name: `focus_sessions` (focus arrives in 2.6), `goals`, `milestones`, `skills` (learning arrives in 2.7), `work_hours` and modes, `nudges`.
- How core avoids depending on modules while it needs their data (the Coach and focus need prayer windows; the mode engine needs today's task progress): provider traits in core implemented by modules, events, or both.
- Where the reminder and recurrence engine lives, since both tasks and health create reminders and modules must not import each other.
- How core-owned flows (onboarding, proposals) write to module-owned tables such as `tasks_areas`.
