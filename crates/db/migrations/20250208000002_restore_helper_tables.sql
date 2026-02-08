-- Restore helper tables that were removed in DDD migration
-- These tables support execution_process functionality

-- Execution process logs table (for storing process output)
CREATE TABLE IF NOT EXISTS execution_process_logs (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    execution_process_id  UUID NOT NULL REFERENCES execution_processes(id) ON DELETE CASCADE,
    logs                  TEXT NOT NULL DEFAULT '',
    byte_size             INTEGER NOT NULL DEFAULT 0,
    inserted_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Execution process repo states table (for tracking git state)
CREATE TABLE IF NOT EXISTS execution_process_repo_states (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    execution_process_id  UUID NOT NULL REFERENCES execution_processes(id) ON DELETE CASCADE,
    repo_id               UUID NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    before_head_commit    TEXT,
    after_head_commit     TEXT,
    merge_commit          TEXT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(execution_process_id, repo_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_execution_process_logs_execution_process_id ON execution_process_logs(execution_process_id);
CREATE INDEX IF NOT EXISTS idx_execution_process_repo_states_execution_process_id ON execution_process_repo_states(execution_process_id);
CREATE INDEX IF NOT EXISTS idx_execution_process_repo_states_repo_id ON execution_process_repo_states(repo_id);
