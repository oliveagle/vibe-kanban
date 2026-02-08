-- Create coding_agent_turns table
CREATE TABLE IF NOT EXISTS coding_agent_turns (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),n    execution_process_id UUID NOT NULL REFERENCES execution_processes(id) ON DELETE CASCADE,n    agent_session_id TEXT,n    prompt TEXT NOT NULL,n    summary TEXT,n    model TEXT,n    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()n);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_coding_agent_turns_execution_process_id ON coding_agent_turns(execution_process_id);
CREATE INDEX IF NOT EXISTS idx_coding_agent_turns_agent_session_id ON coding_agent_turns(agent_session_id);