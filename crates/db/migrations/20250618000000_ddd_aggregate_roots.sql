-- DDD Database Architecture Migration
-- Converts traditional relational tables to DDD Aggregate Roots with JSONB data

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Drop old tables from previous migrations to allow clean DDD schema
DROP TABLE IF EXISTS execution_process_repo_states CASCADE;
DROP TABLE IF EXISTS execution_process_logs CASCADE;
DROP TABLE IF EXISTS execution_processes CASCADE;
DROP TABLE IF EXISTS sessions CASCADE;
DROP TABLE IF EXISTS task_workspaces CASCADE;
DROP TABLE IF EXISTS workspaces CASCADE;
DROP TABLE IF EXISTS task_images CASCADE;
DROP TABLE IF EXISTS images CASCADE;
DROP TABLE IF EXISTS merges CASCADE;
DROP TABLE IF EXISTS scratch CASCADE;
DROP TABLE IF EXISTS workspace_repos CASCADE;
DROP TABLE IF EXISTS project_repos CASCADE;
DROP TABLE IF EXISTS repos CASCADE;
DROP TABLE IF EXISTS tasks CASCADE;
DROP TABLE IF EXISTS projects CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS user_credentials CASCADE;
DROP TABLE IF EXISTS coding_agent_turns CASCADE;
DROP TABLE IF EXISTS local_users CASCADE;
DROP TABLE IF EXISTS tags CASCADE;

-- Drop old types
DROP TYPE IF EXISTS execution_process_status CASCADE;
DROP TYPE IF EXISTS execution_process_run_reason CASCADE;
DROP TYPE IF EXISTS session_status CASCADE;
DROP TYPE IF EXISTS task_status CASCADE;

-- Custom types (create if not exists for idempotency)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'task_status') THEN
        CREATE TYPE task_status AS ENUM ('todo', 'inprogress', 'done', 'cancelled', 'inreview');
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'execution_process_status') THEN
        CREATE TYPE execution_process_status AS ENUM ('running', 'completed', 'failed', 'killed');
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'execution_process_run_reason') THEN
        CREATE TYPE execution_process_run_reason AS ENUM ('setupscript', 'cleanupscript', 'codingagent', 'devserver');
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'session_status') THEN
        CREATE TYPE session_status AS ENUM ('idle', 'awaiting_user_input', 'processing');
    END IF;
END $$;

-- Users aggregate root
CREATE TABLE IF NOT EXISTS users (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name          TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'active',
    data          JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at    TIMESTAMPTZ
);

-- Projects aggregate root with repos and settings in data JSONB
CREATE TABLE IF NOT EXISTS projects (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name          TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'active',
    data          JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at    TIMESTAMPTZ
);

-- Tasks aggregate root with workspaces in data JSONB
CREATE TABLE IF NOT EXISTS tasks (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id    UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    status        task_status NOT NULL DEFAULT 'todo',
    data          JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at    TIMESTAMPTZ
);

-- Task workspaces table (DDD aggregate root extension for task_workspaces)
-- Repos and sessions stored in data JSONB
CREATE TABLE IF NOT EXISTS task_workspaces (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_id       UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    name          TEXT,
    status        TEXT NOT NULL DEFAULT 'active',
    data          JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at    TIMESTAMPTZ
);

-- Sessions for workspace execution
CREATE TABLE IF NOT EXISTS sessions (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id  UUID NOT NULL REFERENCES task_workspaces(id) ON DELETE CASCADE,
    status        session_status NOT NULL DEFAULT 'idle',
    executor      TEXT,
    completed_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Execution processes with action data in JSONB
CREATE TABLE IF NOT EXISTS execution_processes (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id    UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    run_reason    execution_process_run_reason NOT NULL,
    status        execution_process_status NOT NULL DEFAULT 'running',
    data          JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at    TIMESTAMPTZ
);

-- Events aggregate root for system events
CREATE TABLE IF NOT EXISTS events (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    aggregate_id  UUID NOT NULL,
    aggregate_type TEXT NOT NULL,
    event_type    TEXT NOT NULL,
    data          JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_users_not_deleted ON users(id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_data ON users USING GIN(data);

CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status);
CREATE INDEX IF NOT EXISTS idx_projects_not_deleted ON projects(id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_projects_data ON projects USING GIN(data);

CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_not_deleted ON tasks(id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tasks_data ON tasks USING GIN(data);

CREATE INDEX IF NOT EXISTS idx_task_workspaces_task_id ON task_workspaces(task_id);
CREATE INDEX IF NOT EXISTS idx_task_workspaces_status ON task_workspaces(status);
CREATE INDEX IF NOT EXISTS idx_task_workspaces_not_deleted ON task_workspaces(id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_task_workspaces_data ON task_workspaces USING GIN(data);

CREATE INDEX IF NOT EXISTS idx_sessions_workspace_id ON sessions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_execution_processes_session_id ON execution_processes(session_id);
CREATE INDEX IF NOT EXISTS idx_execution_processes_status ON execution_processes(status);
CREATE INDEX IF NOT EXISTS idx_execution_processes_data ON execution_processes USING GIN(data);

CREATE INDEX IF NOT EXISTS idx_events_aggregate ON events(aggregate_id, aggregate_type);
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_data ON events USING GIN(data);

-- Views for active (not deleted) records
CREATE OR REPLACE VIEW active_users AS SELECT * FROM users WHERE deleted_at IS NULL;
CREATE OR REPLACE VIEW active_projects AS SELECT * FROM projects WHERE deleted_at IS NULL;
CREATE OR REPLACE VIEW active_tasks AS SELECT * FROM tasks WHERE deleted_at IS NULL;
CREATE OR REPLACE VIEW active_task_workspaces AS SELECT * FROM task_workspaces WHERE deleted_at IS NULL;
CREATE OR REPLACE VIEW active_execution_processes AS SELECT * FROM execution_processes WHERE deleted_at IS NULL;