CREATE TABLE habits (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('stretch', 'eyeRest', 'water', 'medicine')),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    target INTEGER CHECK (target IS NULL OR target > 0),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    created_at TEXT NOT NULL
) STRICT;

INSERT INTO habits (kind, name, target, created_at) VALUES
    ('stretch', 'Stretch break', NULL, strftime('%Y-%m-%d %H:%M:%S+00:00', 'now')),
    ('eyeRest', 'Eye rest', NULL, strftime('%Y-%m-%d %H:%M:%S+00:00', 'now')),
    ('water', 'Water', 8, strftime('%Y-%m-%d %H:%M:%S+00:00', 'now'));

CREATE TABLE habit_logs (
    id INTEGER PRIMARY KEY,
    habit_id INTEGER NOT NULL REFERENCES habits (id) ON DELETE CASCADE,
    logged_at TEXT NOT NULL,
    amount INTEGER NOT NULL DEFAULT 1 CHECK (amount > 0)
) STRICT;

CREATE INDEX habit_logs_habit_time ON habit_logs (habit_id, logged_at);

ALTER TABLE reminders ADD COLUMN habit_id INTEGER REFERENCES habits (id) ON DELETE CASCADE;

CREATE INDEX reminders_habit ON reminders (habit_id);
