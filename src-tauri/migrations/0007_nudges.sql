CREATE TABLE nudges (
    id INTEGER PRIMARY KEY,
    agent TEXT NOT NULL,
    kind TEXT NOT NULL,
    priority TEXT NOT NULL,
    text TEXT NOT NULL,
    style TEXT,
    mode TEXT,
    fired_at TEXT NOT NULL,
    outcome TEXT CHECK (outcome IN ('accepted', 'snoozed', 'dismissed', 'ignored')),
    action TEXT,
    outcome_at TEXT
) STRICT;

CREATE INDEX nudges_fired ON nudges (fired_at);
CREATE INDEX nudges_open ON nudges (outcome, fired_at);
