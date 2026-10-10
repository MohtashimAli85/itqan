# 0003: Module architecture

Status: accepted (step 2.0.1)

## Context
Phase 1 code is one crate, organised in layers (`commands/`, `domain/`, `db/`). Phase 2 turns Itqan into a platform of modules connected by one coach (PLAN.md section 16). Today the layers are tightly coupled. A few examples from the current imports:

- `domain::tasks` uses goals, skills and categories. `domain::reminders` uses tasks and health habits. `domain::rewards` uses tasks, skills and the profile.
- The mode engine reads prayer windows, focus sessions and today's task counts. The Coach reads prayer windows and health. The tray reads tasks, rewards and modes.
- The bus already exists in a simple form: `agents::publish(app, AppEvent)` calls every registered `Agent`, collects their `Suggestion`s and hands them to the Coach.

This ADR fixes the rules the 2.0 moves follow. Each move step must keep behaviour unchanged.

## Decisions

### 1. Crates and dependency direction
```
src-tauri/                  app crate `itqan` (wiring only, at the end of 2.0)
  crates/itqan-contracts    bus events and their payload value types
  crates/itqan-core         error, db, settings, bus, module contract, ai, and (by 2.0.7) the shell
  crates/itqan-salah | itqan-health | itqan-tasks | itqan-progress
```
- `itqan-contracts` depends on no internal crate. External crates are limited to `serde`, `specta` and `chrono`.
- `itqan-core` depends on `itqan-contracts`. It **never** depends on a module.
- A module depends on `itqan-core` and `itqan-contracts` only. Cargo enforces this: module crates do not list each other.
- The app crate depends on everything. While the moves are in progress it also hosts code that has not moved yet. That code shrinks with every step and is gone by the end of 2.0.7.
- `itqan-core` depends on `tauri`. Modules receive an `&AppHandle` and read shared services from Tauri state (`app.state::<Database>()`), as the code does today. Pure logic stays free of `AppHandle` so it can be tested with a plain connection.

### 2. What each table and area belongs to
| Owner | Tables and code |
|---|---|
| core | `settings`, `profile`, `goals`, `milestones`, `skills`, `nudges`, `work_hours`, `focus_sessions`, `reminders`; the Coach, rhythm and planner agents, the mode engine (work, evening, rest, focus), the scheduler loop, recurrence, reminder delivery, AI, overlay, tray, platform glue |
| salah | `prayer_settings` → `salah_settings`; prayer time calculation, pause windows, Jumu'ah |
| health | `habits` → `health_habits`, `habit_logs` → `health_habit_logs`; the health agent |
| tasks | `categories` → `tasks_categories` (→ `tasks_areas` in 2.3), `tasks`; quick add |
| progress | `xp_events` → `progress_xp_events`, `streaks` → `progress_streaks`; levels, badges, the rewards agent |

Why these owners:
- **Reminders and recurrence are core** scheduler primitives (PLAN C.1 lists "scheduler primitives" under core). Tasks *and* health create reminders. If tasks owned them, health would have to import tasks, and turning tasks off would stop medicine reminders. This deliberately changes the step list, which said "Move Tasks (with reminders)".
- **Goals, milestones and skills are core profile data.** Section 3 lists goals in the motivation profile, and progress and planning both read them. The `learning` module (2.7) builds roadmaps and project ideas on top of them through core APIs.
- **Focus sessions and modes stay core.** The mode engine, reminder holds and the Coach all depend on "is a focus session running". The `focus` module (2.6) adds front-app tracking and drift on top of core sessions.
- **Nudges are core.** They are the Coach's outcome log.

Rows may reference another owner's rows by id. Existing foreign keys stay, for example `tasks.goal_id` → `goals` and `reminders.habit_id` → `health_habits`. A disabled module keeps its data, so those keys stay valid. New code adds a cross-owner key only from a module to core, never from one module to another.

### 3. Contracts
`itqan-contracts` holds:
- `AppEvent`, the in-process bus event enum (today in `agents/mod.rs`);
- the value types its payloads need: `TaskId`, `GoalId`, `MilestoneId`, `FocusSessionId`, `Mode`, `HabitKind`.

Bus events never leave the process and are never stored, so they carry no version number; the compiler checks every change. Each variant derives `Serialize`/`Deserialize`, and a contract test round-trips all of them (2.0.8). Frontend events (`TasksChanged`, `ModeChanged`, …) are **not** contracts. They belong to the crate that emits them.

### 4. Bus, subscribers and signals
- `itqan_core::bus` owns `Subscriber` (today's `Agent` trait, renamed), `Signal` (today's `Suggestion`, already with priority and expiry) and `publish`.
- `publish` calls every enabled module's subscribers in registration order, synchronously, on the caller's thread, as today. A failing subscriber is logged and skipped. Handlers must be quick (a few queries). Slow work (AI) is spawned with `tauri::async_runtime::spawn`.
- Signals go to one `SignalSink` registered in Tauri state, which is the Coach. Core defines the sink trait, so the bus never names the Coach type, and the Coach can move into core later without changing callers.
- `AgentKind`, `Priority` and `BubbleAction` move to core because `Signal` uses them.
- Only the Coach turns signals into words. Modules never show bubbles directly. The exceptions in Phase 1 (reminder delivery, focus done, overtime, reward celebration) belong to core or are celebrations through the overlay API, which is core.

### 5. Ports: when core needs module data
Core defines small traits for what it needs from a module. The module registers an implementation in its `setup`, and core asks the registry. With no implementation (module disabled or missing), core uses a neutral answer.

| Port | Implemented by | Used by | Neutral answer |
|---|---|---|---|
| `PrayerSchedule` (windows around now, next, active) | salah | mode engine, Coach quiet times, reminder holds, focus pauses, tray, Shifts (2.2) | no windows |
| `TaskStats` (today's done and open counts, next task) | tasks | progress ring, tray, rhythm agent | none |
| `ProgressSummary` (level, streak) | progress | tray | none |

Bubble and nudge actions are namespaced `<owner>:<action>` (for example `health:water`). Core routes an action to the handler that its owner registered. Existing action ids are kept where the frontend or stored nudges use them. A step that changes an id gives the old one an alias.

### 6. The `Module` trait
```rust
pub trait Module: Send + Sync + 'static {
    fn id(&self) -> &'static str;
    fn migrations(&self) -> &'static [Migration] { &[] }
    fn setup(&self, app: &AppHandle) -> Result<(), AppError> { Ok(()) }
    fn subscribers(&self) -> Vec<Box<dyn Subscriber>> { Vec::new() }
}

pub struct Migration { pub version: u32, pub name: &'static str, pub sql: &'static str }
```
- `setup` manages module state, registers ports and action handlers, and starts nothing heavy.
- **Commands** stay in each module's `commands` module. The app crate lists them in its single `collect_commands![…]`, grouped by module. This deviates from PLAN C.3's "`commands()` function": `tauri_specta::Builder::commands` replaces earlier registrations, and a Tauri invoke handler cannot be chained, so one list is the only way that works. Frontend events are collected the same way.
- **Settings:** a module's settings are one typed struct implementing `ModuleSettings` (`KEY`, `Default`, `validate`), stored as JSON in `settings` under `module.<id>`. Existing individual keys stay as they are until a step changes them on purpose, with a migration.
- **Enabled flag** (2.0.9): the `modules` table holds `id, enabled, version`. Disabled modules get no `publish` calls and contribute no ports, action handlers or UI. Their migrations still run and their commands stay registered, so data stays consistent and nothing is lost.

### 7. Migrations
- Core keeps its numbered series in `src-tauri/crates/itqan-core/migrations/`, tracked with `PRAGMA user_version` as today, so existing databases need no bootstrap.
- The first module migration (step 2.0.4) adds core migration `0010_schema_migrations`: `schema_migrations(module TEXT, version INTEGER, name TEXT, applied_at TEXT, PRIMARY KEY (module, version))`.
- Each module keeps `migrations/NNNN_name.sql` in its crate. The runner applies all core migrations first, then each module's in registration order, each one in its own transaction, recorded in `schema_migrations`.
- Module tables are prefixed with the module id. The first migration of each module takes over its Phase 1 tables with `ALTER TABLE … RENAME TO …`. SQLite updates foreign keys and indexes that point at a renamed table, so rows and references survive.
- **Every migration ships with a test** that builds the previous schema, inserts representative rows, runs the migration and checks the rows and references are still there.

### 8. Tests across crates
`itqan-core` exposes a `test-support` cargo feature with `test_connection()` (all core migrations) and `test_connection_with(&[&dyn Module])`. Module crates enable it in `[dev-dependencies]`. Existing tests move with their code unchanged except for `use` paths.

All cargo commands run with `--workspace` (PLAN section 11). CI changes in 2.0.2.

### 9. Frontend
```
src/
  core/        shell: main window layout and router, overlay frame, panel frame, registry types
  shared/      ui components, lib, hooks, bindings, test helpers (imports nothing from core or modules)
  modules/
    index.ts   the list of module manifests (the only file core may import from modules/)
    tasks/ salah/ health/ progress/
      index.ts  manifest
```
```ts
export type ModuleManifest = {
  id: ModuleId;
  routes?: ModuleRoute[];
  sidebarItems?: SidebarItem[];
  panelTabs?: PanelTab[];
  settingsSections?: SettingsSection[];
  onboardingSteps?: OnboardingStep[];
};
```
- The shell renders what the enabled manifests contribute, in manifest order.
- **Boundary lint:** dependency-cruiser in CI. A module imports only `src/core` (types and hooks) and `src/shared`. `src/core` imports `src/modules/index.ts` and nothing else under `src/modules`. `src/shared` imports neither. The rule runs as part of `pnpm lint`.

## Order of the moves
| Step | Moves |
|---|---|
| 2.0.2 | workspace; `itqan-contracts` (`AppEvent` and payload types); `itqan-core`: error, `Database`, core migrations and enum helpers, settings, AI, bus (`Subscriber`, `Signal`, `SignalSink`, `publish`), `Module` and `Migration` types |
| 2.0.3 | frontend layout, registry and boundary lint |
| 2.0.4 | `itqan-salah`, `PrayerSchedule` port, module migration runner with core `0010`, `salah_settings` rename |
| 2.0.5 | reminders and recurrence into core; `itqan-health`, `health_*` renames |
| 2.0.6 | `itqan-tasks`, `TaskStats` port, `tasks_categories` rename |
| 2.0.7 | `itqan-progress`, `ProgressSummary` port, `progress_*` renames; the remaining shell (Coach, rhythm, planner, mode engine, scheduler, overlay, tray, platform, profile, goals, skills, nudges) into core |

## Consequences
- The app crate's `lib.rs` keeps the long `collect_commands!` list. That's accepted: it's the wiring point, and specta still collects every type.
- Core grows large by 2.0.7. That's expected: PLAN C.1 puts the shell, Coach, profile and AI in core. If it becomes unwieldy, it can split into sub-crates later without affecting modules.
- Cross-owner reads go through ports. Each port adds a little indirection, but disabling a module becomes a safe operation.
