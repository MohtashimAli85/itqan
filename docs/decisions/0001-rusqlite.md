# 0001: rusqlite over sqlx

Status: accepted (step 0.6)

## Decision
Use `rusqlite` with the bundled SQLite build and hand-written versioned migrations tracked with `PRAGMA user_version`.

## Why
- Phase 3 needs SQLCipher and the sqlite-vec extension. Both are simpler to load with `rusqlite` than through `sqlx`.
- The app is a single-process desktop app with one local database, so `sqlx` async pooling and compile-time query checks bring little.
- A `Mutex<Connection>` behind `db::Database` is enough; the scheduler and commands do short queries.

## Consequences
- Queries are plain SQL strings in `db/` repositories, so repositories need their own tests.
- Migrations live in `src-tauri/migrations/` as numbered `.sql` files embedded at compile time and applied in order at startup.
