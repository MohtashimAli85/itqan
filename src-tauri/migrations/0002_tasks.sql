CREATE TABLE categories (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    colour TEXT NOT NULL,
    icon TEXT NOT NULL,
    builtin INTEGER NOT NULL DEFAULT 0 CHECK (builtin IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 0
) STRICT;

INSERT INTO categories (name, colour, icon, builtin, sort_order) VALUES
    ('Work', 'work', 'briefcase', 1, 1),
    ('Personal', 'personal', 'house', 1, 2),
    ('Health', 'health', 'heart-pulse', 1, 3),
    ('Learning', 'amber', 'book-open', 1, 4),
    ('Building', 'brand', 'hammer', 1, 5);

CREATE TABLE tasks (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    notes TEXT,
    category_id INTEGER REFERENCES categories (id) ON DELETE SET NULL,
    kind TEXT NOT NULL DEFAULT 'output'
        CHECK (kind IN ('output', 'learning', 'deepWork', 'habit')),
    priority INTEGER NOT NULL DEFAULT 0 CHECK (priority BETWEEN 0 AND 3),
    due_at TEXT,
    is_top_three INTEGER NOT NULL DEFAULT 0 CHECK (is_top_three IN (0, 1)),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'done')),
    completed_at TEXT,
    parent_id INTEGER REFERENCES tasks (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX tasks_status_due ON tasks (status, due_at);
CREATE INDEX tasks_parent ON tasks (parent_id);
CREATE INDEX tasks_category ON tasks (category_id);
