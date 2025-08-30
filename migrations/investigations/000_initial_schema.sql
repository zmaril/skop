-- Investigation metadata with specific fields
CREATE TABLE metadata (
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    color_rgb TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    version TEXT NOT NULL DEFAULT '1.0'
);

-- Widget configurations and state
CREATE TABLE widgets (
    id INTEGER PRIMARY KEY,
    version INTEGER NOT NULL DEFAULT 0,
    widget_type TEXT NOT NULL,
    config_json TEXT NOT NULL,
    position_x REAL NOT NULL,
    position_y REAL NOT NULL,
    size_x REAL NOT NULL,
    size_y REAL NOT NULL,
    created_at INTEGER NOT NULL,
    collapsed BOOLEAN DEFAULT 0,
    archived_at INTEGER DEFAULT NULL
);

-- Raw data capture
CREATE TABLE raw_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    widget_id INTEGER NOT NULL,
    widget_version INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    line_content TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    FOREIGN KEY(widget_id) REFERENCES widgets(id)
);

-- Indexes for performance
CREATE INDEX idx_raw_data_widget_id ON raw_data(widget_id);
CREATE INDEX idx_raw_data_widget_version ON raw_data(widget_id, widget_version);
CREATE INDEX idx_raw_data_timestamp ON raw_data(timestamp);
CREATE INDEX idx_widgets_archived_at ON widgets(archived_at);
CREATE INDEX idx_widgets_latest_version ON widgets(id, version DESC);