CREATE TABLE beliefs (
    id INTEGER PRIMARY KEY,
    statement TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('motivator', 'preference', 'pattern', 'constraint', 'goal')),
    subject TEXT NOT NULL,
    value TEXT,
    strength TEXT NOT NULL CHECK (strength IN ('low', 'medium', 'high')),
    confidence REAL NOT NULL CHECK (confidence BETWEEN 0 AND 1),
    source TEXT NOT NULL CHECK (source IN ('said', 'observed', 'confirmed')),
    evidence TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL CHECK (status IN ('active', 'proposed', 'rejected', 'archived')),
    created_at TEXT NOT NULL,
    confirmed_at TEXT,
    updated_at TEXT NOT NULL
) STRICT;

CREATE UNIQUE INDEX beliefs_one_active ON beliefs (subject)
    WHERE status = 'active' AND subject <> 'note';
CREATE INDEX beliefs_status ON beliefs (status, subject);

INSERT INTO beliefs (statement, kind, subject, value, strength, confidence, source, status, created_at, updated_at)
SELECT
    upper(substr(json_extract(item.value, '$.motivator'), 1, 1))
        || substr(json_extract(item.value, '$.motivator'), 2) || ' matters to me',
    'motivator',
    'motivator.' || json_extract(item.value, '$.motivator'),
    json_object('weight', json_extract(item.value, '$.weight')),
    CASE
        WHEN json_extract(item.value, '$.weight') >= 30 THEN 'high'
        WHEN json_extract(item.value, '$.weight') >= 15 THEN 'medium'
        ELSE 'low'
    END,
    0.8,
    'said',
    'active',
    coalesce(profile.updated_at, strftime('%Y-%m-%d %H:%M:%f+00:00', 'now')),
    coalesce(profile.updated_at, strftime('%Y-%m-%d %H:%M:%f+00:00', 'now'))
FROM profile, json_each(profile.motivators) AS item
WHERE profile.id = 1 AND json_extract(item.value, '$.weight') > 0;

INSERT INTO beliefs (statement, kind, subject, value, strength, confidence, source, status, created_at, updated_at)
SELECT
    'Coach me like a ' || profile.coach_style,
    'preference',
    'coach.style',
    json_object('style', profile.coach_style),
    'medium',
    0.8,
    'said',
    'active',
    coalesce(profile.updated_at, strftime('%Y-%m-%d %H:%M:%f+00:00', 'now')),
    coalesce(profile.updated_at, strftime('%Y-%m-%d %H:%M:%f+00:00', 'now'))
FROM profile
WHERE profile.id = 1
    AND EXISTS (SELECT 1 FROM settings WHERE key = 'onboarding_completed' AND value = 'true');
