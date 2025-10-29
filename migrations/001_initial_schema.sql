-- Create sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    working_directory TEXT NOT NULL,
    environment TEXT, -- JSON blob of environment variables
    is_active BOOLEAN NOT NULL DEFAULT 0
);

-- Create blocks table
CREATE TABLE IF NOT EXISTS blocks (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    command TEXT NOT NULL,
    output TEXT NOT NULL DEFAULT '',
    exit_code INTEGER,
    state TEXT NOT NULL,
    working_directory TEXT NOT NULL,
    environment TEXT, -- JSON blob
    started_at TEXT,
    completed_at TEXT,
    duration_ms INTEGER,
    is_collapsed BOOLEAN NOT NULL DEFAULT 0,
    block_order INTEGER NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_blocks_session_id ON blocks(session_id);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks(timestamp);
CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions(updated_at);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(is_active);
