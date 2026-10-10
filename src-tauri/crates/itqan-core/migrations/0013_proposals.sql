CREATE TABLE proposals (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    key TEXT NOT NULL,
    title TEXT NOT NULL,
    reason TEXT NOT NULL,
    effect TEXT NOT NULL,
    payload TEXT NOT NULL DEFAULT 'null',
    belief_id INTEGER REFERENCES beliefs (id) ON DELETE SET NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'accepted', 'rejected', 'expired')),
    created_at TEXT NOT NULL,
    decided_at TEXT,
    expires_at TEXT NOT NULL,
    suppressed_until TEXT
) STRICT;

CREATE UNIQUE INDEX proposals_one_pending ON proposals (key) WHERE status = 'pending';
CREATE INDEX proposals_status ON proposals (status, kind);
CREATE INDEX proposals_suppressed ON proposals (key, suppressed_until);
