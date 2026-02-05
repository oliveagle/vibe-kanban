#!/bin/bash
# SQLite to PostgreSQL Data Migration Script
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

# Function to convert SQLite BLOB UUID to PostgreSQL UUID
convert_uuid() {
    local hex="$1"
    # Format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    echo "${hex:0:8}-${hex:8:4}-${hex:12:4}-${hex:16:4}-${hex:20:12}"
}

# Function to migrate table
dump_and_load() {
    local table="$1"
    local query="$2"
    local output_file="/tmp/${table}_export.csv"
    
    echo "Migrating $table..."
    
    # Export from SQLite
    sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "$query" > "$output_file"
    
    # Import to PostgreSQL
    psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" -c "
        COPY $table FROM '$output_file' WITH (FORMAT csv, HEADER true);
    "
    
    rm -f "$output_file"
    echo "  ✓ $table migrated"
}

echo "Step 1: Exporting data from SQLite..."

# Export projects
echo "  - projects"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT 
    lower(hex(id)), 
    name, 
    git_repo_path, 
    COALESCE(setup_script, ''), 
    created_at, 
    updated_at 
FROM projects" > /tmp/projects.csv

# Export tasks
echo "  - tasks"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT 
    lower(hex(id)), 
    lower(hex(project_id)), 
    title, 
    description, 
    status, 
    created_at, 
    updated_at 
FROM tasks" > /tmp/tasks.csv

# Export workspaces (task_attempts)
echo "  - workspaces"
sqlite3 "$SQLITE_DB" ".mode csv" ".headers on" "SELECT 
    lower(hex(id)), 
    lower(hex(task_id)), 
    'active', 
    COALESCE(execution_history, ''), 
    created_at, 
    updated_at 
FROM task_attempts" > /tmp/workspaces.csv

echo ""
echo "Step 2: Importing data to PostgreSQL..."

# Import projects
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" -c "
    COPY projects(id, name, git_repo_path, setup_script, created_at, updated_at) 
    FROM '/tmp/projects.csv' WITH (FORMAT csv, HEADER true);
" && echo "  ✓ projects"

# Import tasks
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" -c "
    COPY tasks(id, project_id, title, description, status, created_at, updated_at) 
    FROM '/tmp/tasks.csv' WITH (FORMAT csv, HEADER true);
" && echo "  ✓ tasks"

# Import workspaces
psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" -c "
    COPY workspaces(id, task_id, status, execution_history, created_at, updated_at) 
    FROM '/tmp/workspaces.csv' WITH (FORMAT csv, HEADER true);
" && echo "  ✓ workspaces"

# Cleanup
rm -f /tmp/projects.csv /tmp/tasks.csv /tmp/workspaces.csv

echo ""
echo "=== Migration Complete ==="
echo ""
echo "To verify, run:"
echo "  psql -h $PG_HOST -p $PG_PORT -U $PG_USER -d $PG_DB -c 'SELECT COUNT(*) FROM projects;'"
