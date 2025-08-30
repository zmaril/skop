-- Initial schema for main database
CREATE TABLE investigations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL,
    last_accessed INTEGER NOT NULL,
    archived BOOLEAN DEFAULT 0
);

-- Index for performance
CREATE INDEX idx_investigations_last_accessed ON investigations(last_accessed);
CREATE INDEX idx_investigations_archived ON investigations(archived);