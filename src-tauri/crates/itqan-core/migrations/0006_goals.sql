CREATE TABLE goals (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    motivator TEXT,
    target_date TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'done', 'archived')),
    created_at TEXT NOT NULL,
    completed_at TEXT
) STRICT;

CREATE TABLE milestones (
    id INTEGER PRIMARY KEY,
    goal_id INTEGER NOT NULL REFERENCES goals (id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    week_start TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'done')),
    sort_order INTEGER NOT NULL,
    completed_at TEXT
) STRICT;

CREATE INDEX milestones_goal ON milestones (goal_id, sort_order);

CREATE TABLE skills (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK (length(trim(name)) > 0),
    xp INTEGER NOT NULL DEFAULT 0 CHECK (xp >= 0),
    created_at TEXT NOT NULL
) STRICT;

ALTER TABLE tasks ADD COLUMN goal_id INTEGER REFERENCES goals (id) ON DELETE SET NULL;
ALTER TABLE tasks ADD COLUMN skill_id INTEGER REFERENCES skills (id) ON DELETE SET NULL;

CREATE INDEX tasks_goal ON tasks (goal_id);
CREATE INDEX tasks_skill ON tasks (skill_id);
