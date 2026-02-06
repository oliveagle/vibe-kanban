#!/bin/bash
# SQLite to PostgreSQL Data Migration Script - Updated for actual schema
# Usage: ./migrate_sqlite_to_postgres.sh <sqlite_db_path>

set -e

SQLITE_DB="${1:-$HOME/.local/share/vibe-kanban/db.sqlite}"
PG_HOST="10.126.126.5"
PG_PORT="5632"
PG_USER="vibekanban"
PG_PASS="vibekanban123"
PG_DB="vibe_kanban"

export PGPASSWORD="$PG_PASS"

echo "=== SQLite to PostgreSQL Migration ==="
echo "SQLite DB: $SQLITE_DB"
echo "PostgreSQL: $PG_HOST:$PG_PORT/$PG_DB"
echo ""

# Check if sqlite3 is available
if ! command -v sqlite3 &> /dev/null; then
    echo "Installing sqlite3..."
    sudo apt-get update -qq && sudo apt-get install -y -qq sqlite3
fi

echo "Step 1: Exporting data from SQLite..."

# Export projects (SQLite has different columns)
echo "  - projects"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT 
    lower(hex(id)) as id, 
    name, 
    COALESCE(dev_script, '') as setup_script,
    created_at, 
    updated_at 
FROM projects" > /tmp/projects.csv

# Export tasks
echo "  - tasks"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT 
    lower(hex(id)) as id, 
    lower(hex(project_id)) as project_id, 
    title, 
    description, 
    status, 
    created_at, 
    updated_at 
FROM tasks" > /tmp/tasks.csv

# Export workspaces
echo "  - workspaces"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT 
    lower(hex(id)) as id, 
    lower(hex(task_id)) as task_id, 
    'active' as status,
    created_at, 
    updated_at 
FROM workspaces" > /tmp/workspaces.csv

# Export sessions (SQLite has different columns)
echo "  - sessions"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT 
    lower(hex(id)) as id, 
    lower(hex(workspace_id)) as workspace_id, 
    executor as agent_session_id,
    'running' as status,
    created_at, 
    updated_at 
FROM sessions" > /tmp/sessions.csv

echo "  - repos"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT 
    lower(hex(id)) as id, 
    name,
    COALESCE(path, '') as url,
    created_at, 
    updated_at 
FROM repos" > /tmp/repos.csv

# Export workspace_repos
echo "  - workspace_repos"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT 
    lower(hex(id)) as id, 
    lower(hex(workspace_id)) as workspace_id, 
    lower(hex(repo_id)) as repo_id, 
    target_branch,
    created_at, 
    updated_at 
FROM workspace_repos" > /tmp/workspace_repos.csv

echo ""
echo "Step 2: Importing data to PostgreSQL..."

# Import projects using \copy (works for non-superuser)
echo "  ✓ projects"
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" << EOF
\copy projects(id, name, setup_script, created_at, updated_at) FROM '/tmp/projects.csv' WITH (FORMAT csv, HEADER true)
EOF

# Import tasks
echo "  ✓ tasks"
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" << EOF
\copy tasks(id, project_id, title, description, status, created_at, updated_at) FROM '/tmp/tasks.csv' WITH (FORMAT csv, HEADER true)
EOF

# Import workspaces
echo "  ✓ workspaces"
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" << EOF
\copy workspaces(id, task_id, status, created_at, updated_at) FROM '/tmp/workspaces.csv' WITH (FORMAT csv, HEADER true)
EOF

# Import sessions
echo "  ✓ sessions"
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" << EOF
\copy sessions(id, workspace_id, agent_session_id, status, created_at, updated_at) FROM '/tmp/sessions.csv' WITH (FORMAT csv, HEADER true)
EOF

# Import repos
echo "  ✓ repos"
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" << EOF
\copy repos(id, name, url, created_at, updated_at) FROM '/tmp/repos.csv' WITH (FORMAT csv, HEADER true)
EOF

# Import workspace_repos
echo "  ✓ workspace_repos"
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" << EOF
\copy workspace_repos(id, workspace_id, repo_id, target_branch, created_at, updated_at) FROM '/tmp/workspace_repos.csv' WITH (FORMAT csv, HEADER true)
EOF

echo "  - execution_processes"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT
    lower(hex(id)) as id,
    lower(hex(session_id)) as session_id,
    run_reason,
    executor_action,
    status,
    exit_code,
    dropped,
    started_at,
    completed_at,
    created_at,
    updated_at
FROM execution_processes" > /tmp/execution_processes.csv

echo "  - execution_process_logs"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT
    lower(hex(execution_id)) as execution_id,
    logs,
    byte_size,
    inserted_at
FROM execution_process_logs" > /tmp/execution_process_logs.csv

echo "  - execution_process_repo_states"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT
    lower(hex(id)) as id,
    lower(hex(execution_process_id)) as execution_process_id,
    lower(hex(repo_id)) as repo_id,
    before_head_commit,
    after_head_commit,
    merge_commit,
    created_at,
    updated_at
FROM execution_process_repo_states" > /tmp/execution_process_repo_states.csv

echo "  - coding_agent_turns"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT
    lower(hex(id)) as id,
    lower(hex(execution_process_id)) as execution_process_id,
    agent_session_id,
    prompt,
    summary,
    created_at,
    updated_at
FROM coding_agent_turns" > /tmp/coding_agent_turns.csv

echo "  ✓ execution_processes"
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" << EOF
\copy execution_processes(id, session_id, run_reason, executor_action, status, exit_code, dropped, started_at, completed_at, created_at, updated_at) FROM '/tmp/execution_processes.csv' WITH (FORMAT csv, HEADER true)
EOF

echo "  ✓ execution_process_logs"
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" << EOF
\copy execution_process_logs(execution_id, logs, byte_size, inserted_at) FROM '/tmp/execution_process_logs.csv' WITH (FORMAT csv, HEADER true)
EOF

echo "  ✓ execution_process_repo_states"
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" << EOF
\copy execution_process_repo_states(id, execution_process_id, repo_id, before_head_commit, after_head_commit, merge_commit, created_at, updated_at) FROM '/tmp/execution_process_repo_states.csv' WITH (FORMAT csv, HEADER true)
EOF

echo "  ✓ coding_agent_turns"
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" << EOF
\copy coding_agent_turns(id, execution_process_id, agent_session_id, prompt, summary, created_at, updated_at) FROM '/tmp/coding_agent_turns.csv' WITH (FORMAT csv, HEADER true)
EOF

# Cleanup
rm -f /tmp/projects.csv /tmp/tasks.csv /tmp/workspaces.csv /tmp/sessions.csv /tmp/repos.csv /tmp/workspace_repos.csv /tmp/execution_processes.csv /tmp/execution_process_logs.csv /tmp/execution_process_repo_states.csv /tmp/coding_agent_turns.csv

echo ""
echo "=== Migration Complete ==="
echo ""
echo "To verify, run:"
echo "  psql -h $PG_HOST -p $PG_PORT -U $PG_USER -d $PG_DB -c 'SELECT COUNT(*) FROM projects;'"
