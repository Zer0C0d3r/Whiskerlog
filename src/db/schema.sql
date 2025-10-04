CREATE TABLE IF NOT EXISTS commands (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    exit_code INTEGER,
    duration INTEGER, -- milliseconds
    working_directory TEXT,
    session_id TEXT NOT NULL,
    host_id TEXT NOT NULL DEFAULT 'local',
    network_endpoints TEXT DEFAULT '[]', -- JSON array
    packages_used TEXT DEFAULT '[]', -- JSON array
    is_experiment BOOLEAN DEFAULT FALSE,
    experiment_tags TEXT DEFAULT '[]', -- JSON array
    is_dangerous BOOLEAN DEFAULT FALSE,
    danger_score REAL DEFAULT 0.0,
    danger_reasons TEXT DEFAULT '[]', -- JSON array
    shell TEXT NOT NULL DEFAULT 'unknown',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_commands_timestamp ON commands(timestamp);
CREATE INDEX IF NOT EXISTS idx_commands_session ON commands(session_id);
CREATE INDEX IF NOT EXISTS idx_commands_host ON commands(host_id);
CREATE INDEX IF NOT EXISTS idx_commands_dangerous ON commands(is_dangerous);
CREATE INDEX IF NOT EXISTS idx_commands_experiment ON commands(is_experiment);
CREATE INDEX IF NOT EXISTS idx_commands_shell ON commands(shell);

-- Full-text search support
CREATE VIRTUAL TABLE IF NOT EXISTS commands_fts USING fts5(
    command,
    working_directory,
    content='commands',
    content_rowid='id'
);

-- Triggers to keep FTS table in sync
CREATE TRIGGER IF NOT EXISTS commands_fts_insert AFTER INSERT ON commands BEGIN
    INSERT INTO commands_fts(rowid, command, working_directory) 
    VALUES (new.id, new.command, new.working_directory);
END;

CREATE TRIGGER IF NOT EXISTS commands_fts_delete AFTER DELETE ON commands BEGIN
    DELETE FROM commands_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS commands_fts_update AFTER UPDATE ON commands BEGIN
    DELETE FROM commands_fts WHERE rowid = old.id;
    INSERT INTO commands_fts(rowid, command, working_directory) 
    VALUES (new.id, new.command, new.working_directory);
END;