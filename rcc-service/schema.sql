-- Initial schema for RCCService
CREATE TABLE IF NOT EXISTS artifacts (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    wasm_blob BYTEA NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS game_instances (
    id UUID PRIMARY KEY,
    artifact_id UUID REFERENCES artifacts(id),
    executor_id TEXT NOT NULL,
    status TEXT NOT NULL, -- 'PENDING', 'RUNNING', 'TERMINATED'
    port INT NOT NULL,
    player_count INT DEFAULT 0
);
