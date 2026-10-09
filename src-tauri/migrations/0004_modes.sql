CREATE TABLE prayer_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    enabled INTEGER NOT NULL DEFAULT 0 CHECK (enabled IN (0, 1)),
    city TEXT,
    latitude REAL,
    longitude REAL,
    method TEXT NOT NULL DEFAULT 'karachi',
    madhab TEXT NOT NULL DEFAULT 'hanafi',
    high_latitude_rule TEXT NOT NULL DEFAULT 'middleOfTheNight',
    pause_before_minutes INTEGER NOT NULL DEFAULT 5,
    pause_after_minutes INTEGER NOT NULL DEFAULT 20,
    jumuah_break INTEGER NOT NULL DEFAULT 1 CHECK (jumuah_break IN (0, 1))
) STRICT;

INSERT INTO prayer_settings (id) VALUES (1);

CREATE TABLE work_hours (
    weekday INTEGER PRIMARY KEY CHECK (weekday BETWEEN 0 AND 6),
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    start_minute INTEGER NOT NULL CHECK (start_minute BETWEEN 0 AND 1439),
    end_minute INTEGER NOT NULL CHECK (end_minute BETWEEN 1 AND 1440),
    CHECK (end_minute > start_minute)
) STRICT;

INSERT INTO work_hours (weekday, enabled, start_minute, end_minute) VALUES
    (0, 1, 540, 1020),
    (1, 1, 540, 1020),
    (2, 1, 540, 1020),
    (3, 1, 540, 1020),
    (4, 1, 540, 1020),
    (5, 0, 540, 1020),
    (6, 0, 540, 1020);

CREATE TABLE focus_sessions (
    id INTEGER PRIMARY KEY,
    task_id INTEGER REFERENCES tasks (id) ON DELETE SET NULL,
    started_at TEXT NOT NULL,
    planned_minutes INTEGER NOT NULL CHECK (planned_minutes BETWEEN 1 AND 240),
    ended_at TEXT,
    completed INTEGER NOT NULL DEFAULT 0 CHECK (completed IN (0, 1))
) STRICT;

CREATE INDEX focus_sessions_open ON focus_sessions (ended_at);
