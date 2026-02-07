-- PostgreSQL Initial Migration
-- Complete schema converted from SQLite

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- projects table
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    git_repo_path TEXT NOT NULL UNIQUE,
    setup_script TEXT DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_projects_name ON projects(name);
CREATE INDEX idx_projects_created_at ON projects(created_at);

-- tasks table
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'todo' CHECK (status IN ('todo', 'inprogress', 'done', 'cancelled', 'inreview')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_tasks_project_id ON tasks(project_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_created_at ON tasks(created_at);

-- task_attempts table
CREATE TABLE task_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    worktree_path TEXT NOT NULL,
    merge_commit TEXT,
    executor TEXT,
    stdout TEXT,
    stderr TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_task_attempts_task_id ON task_attempts(task_id);
CREATE INDEX idx_task_attempts_created_at ON task_attempts(created_at);

-- task_attempt_activities table
CREATE TABLE task_attempt_activities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_attempt_id UUID NOT NULL REFERENCES task_attempts(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'init' CHECK (status IN ('init', 'setuprunning', 'setupcomplete', 'setupfailed', 'executorrunning', 'executorcomplete', 'executorfailed', 'paused')),
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_task_attempt_activities_task_attempt_id ON task_attempt_activities(task_attempt_id);
CREATE INDEX idx_task_attempt_activities_status ON task_attempt_activities(status);
