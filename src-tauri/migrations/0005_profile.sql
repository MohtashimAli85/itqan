CREATE TABLE profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT,
    motivators TEXT NOT NULL DEFAULT '[]',
    situation TEXT,
    free_hours_per_week INTEGER CHECK (free_hours_per_week BETWEEN 0 AND 168),
    age INTEGER CHECK (age BETWEEN 13 AND 110),
    coach_style TEXT NOT NULL DEFAULT 'mentor'
        CHECK (coach_style IN ('mentor', 'manager', 'trainer')),
    family_start_minute INTEGER CHECK (family_start_minute BETWEEN 0 AND 1439),
    family_end_minute INTEGER CHECK (family_end_minute BETWEEN 1 AND 1440),
    updated_at TEXT
) STRICT;

INSERT INTO profile (id) VALUES (1);
