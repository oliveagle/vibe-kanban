-- PostgreSQL initialization migration
-- Complete schema for vibe-kanban

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Custom types
CREATE TYPE task_status AS ENUM ('todo', 'inprogress', 'done', 'cancelled', 'inreview');
CREATE TYPE workspace_status AS ENUM ('setuprunning', 'setupcomplete', 'setupfailed', 'executorrunning', 'executorcomplete', 'executorfailed');
CREATE TYPE execution_process_status AS ENUM ('running', 'completed', 'failed', 'killed');
CREATE TYPE execution_process_run_reason AS ENUM ('setupscript', 'cleanupscript', 'codingagent', 'devserver');
CREATE TYPE session_status AS ENUM ('idle', 'awaiting_user_input', 'processing');

-- Projects table
CREATE TABLE projects (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name          TEXT NOT NULL,
    dev_script    TEXT,
    dev_script_working_dir TEXT,
    default_agent_working_dir TEXT,
    remote_project_id UUID,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tasks table
CREATE TABLE tasks (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    description TEXT,
    status      task_status NOT NULL DEFAULT 'todo',
    parent_workspace_id UUID,
    shared_task_id UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Workspaces table (replaces task_attempts)
CREATE TABLE workspaces (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_id       UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    container_ref TEXT,
    branch        TEXT,
    agent_working_dir TEXT,
    setup_completed_at TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Repos table
CREATE TABLE repos (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id    UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    path          TEXT NOT NULL,
    url           TEXT NOT NULL,
    default_branch TEXT DEFAULT 'main',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Workspace repos table
CREATE TABLE workspace_repos (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id  UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    repo_id       UUID NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    target_branch TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, repo_id)
);

-- Sessions table
CREATE TABLE sessions (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id  UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    status        session_status NOT NULL DEFAULT 'idle',
    executor      TEXT,
    completed_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Execution processes table
CREATE TABLE execution_processes (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id    UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    run_reason    execution_process_run_reason NOT NULL,
    executor_action JSONB,
    status        execution_process_status NOT NULL DEFAULT 'running',
    exit_code     INTEGER,
    dropped       BOOLEAN NOT NULL DEFAULT FALSE,
    started_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Execution process logs table
CREATE TABLE execution_process_logs (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    execution_process_id  UUID NOT NULL REFERENCES execution_processes(id) ON DELETE CASCADE,
    log_data              JSONB NOT NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Execution process repo states table
CREATE TABLE execution_process_repo_states (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    execution_process_id  UUID NOT NULL REFERENCES execution_processes(id) ON DELETE CASCADE,
    repo_id               UUID NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    before_head_commit    TEXT,
    after_head_commit     TEXT,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(execution_process_id, repo_id)
);

-- Images table
CREATE TABLE images (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_path     TEXT NOT NULL,
    original_name TEXT NOT NULL,
    mime_type     TEXT NOT NULL,
    size_bytes    BIGINT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Task images table (junction table)
CREATE TABLE task_images (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_id       UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    image_id      UUID NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(task_id, image_id)
);

-- Tags table
CREATE TABLE tags (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name          TEXT NOT NULL UNIQUE,
    color         TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Scratch table (for drafts and follow-ups)
CREATE TABLE scratch (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id  UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    content       TEXT NOT NULL,
    is_followup   BOOLEAN NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Merges table
CREATE TABLE merges (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id  UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    pr_number     INTEGER,
    pr_url        TEXT,
    merge_commit  TEXT,
    base_branch   TEXT,
    status        TEXT NOT NULL DEFAULT 'pending',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User credentials table
CREATE TABLE user_credentials (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id       UUID,
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Coding agent turns table
CREATE TABLE coding_agent_turns (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id    UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    turn_data     JSONB NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Local users table
CREATE TABLE local_users (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username      TEXT NOT NULL UNIQUE,
    display_name  TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Project repos table (many-to-many)
CREATE TABLE project_repos (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id    UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    repo_id       UUID NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(project_id, repo_id)
);

-- Create indexes for performance
CREATE INDEX idx_tasks_project_id ON tasks(project_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_parent_workspace_id ON tasks(parent_workspace_id);
CREATE INDEX idx_workspaces_task_id ON workspaces(task_id);
CREATE INDEX idx_sessions_workspace_id ON sessions(workspace_id);
CREATE INDEX idx_execution_processes_session_id ON execution_processes(session_id);
CREATE INDEX idx_execution_process_logs_execution_process_id ON execution_process_logs(execution_process_id);
CREATE INDEX idx_scratch_workspace_id ON scratch(workspace_id);

-- Add missing columns to local_users
ALTER TABLE local_users ADD COLUMN IF NOT EXISTS password_hash TEXT;

-- Add missing columns to images
ALTER TABLE images ADD COLUMN IF NOT EXISTS hash TEXT;

-- Create task_workspaces table (legacy compatibility)
CREATE TABLE IF NOT EXISTS task_workspaces (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_id       UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    workspace_id  UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(task_id, workspace_id)
);
