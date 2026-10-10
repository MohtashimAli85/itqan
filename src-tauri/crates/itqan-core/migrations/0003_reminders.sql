CREATE TABLE reminders (
    id INTEGER PRIMARY KEY,
    task_id INTEGER REFERENCES tasks (id) ON DELETE CASCADE,
    title TEXT,
    anchor_at TEXT NOT NULL,
    next_at TEXT,
    rrule TEXT,
    timezone TEXT NOT NULL,
    critical INTEGER NOT NULL DEFAULT 0 CHECK (critical IN (0, 1)),
    snoozed_until TEXT,
    last_fired_at TEXT,
    created_at TEXT NOT NULL,
    CHECK (task_id IS NOT NULL OR title IS NOT NULL)
) STRICT;

CREATE INDEX reminders_next ON reminders (next_at);
CREATE INDEX reminders_snoozed ON reminders (snoozed_until);
CREATE INDEX reminders_task ON reminders (task_id);
