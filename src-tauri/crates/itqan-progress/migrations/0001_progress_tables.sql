ALTER TABLE xp_events RENAME TO progress_xp_events;
ALTER TABLE streaks RENAME TO progress_streaks;

DROP INDEX xp_events_at;
CREATE INDEX progress_xp_events_at ON progress_xp_events (at);

DROP INDEX xp_events_once;
CREATE UNIQUE INDEX progress_xp_events_once ON progress_xp_events (source, reference_id)
    WHERE source IN ('task', 'milestone', 'goal', 'focus') AND reference_id IS NOT NULL;
