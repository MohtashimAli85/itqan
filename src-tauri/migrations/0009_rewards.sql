CREATE TABLE xp_events (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL CHECK (source IN ('task', 'focus', 'milestone', 'goal', 'habit')),
    amount INTEGER NOT NULL CHECK (amount > 0),
    task_id INTEGER REFERENCES tasks (id) ON DELETE SET NULL,
    skill_id INTEGER REFERENCES skills (id) ON DELETE SET NULL,
    reference_id INTEGER,
    at TEXT NOT NULL
) STRICT;

CREATE INDEX xp_events_at ON xp_events (at);
CREATE UNIQUE INDEX xp_events_once ON xp_events (source, reference_id)
    WHERE source IN ('task', 'milestone', 'goal', 'focus') AND reference_id IS NOT NULL;

CREATE TABLE streaks (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    current INTEGER NOT NULL DEFAULT 0,
    best INTEGER NOT NULL DEFAULT 0,
    freezes INTEGER NOT NULL DEFAULT 1 CHECK (freezes BETWEEN 0 AND 2),
    last_active_day TEXT
) STRICT;

INSERT INTO streaks (id) VALUES (1);
