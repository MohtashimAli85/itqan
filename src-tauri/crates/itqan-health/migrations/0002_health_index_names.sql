DROP INDEX habit_logs_habit_time;
CREATE INDEX health_habit_logs_habit_time ON health_habit_logs (habit_id, logged_at);
