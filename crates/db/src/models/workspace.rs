use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use super::{
    project::Project,
    task::Task,
    workspace_repo::RepoWithTargetBranch,
};

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Task not found")]
    TaskNotFound,
    #[error("Project not found")]
    ProjectNotFound,
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Branch not found: {0}")]
    BranchNotFound(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainerInfo {
    pub workspace_id: Uuid,
    pub task_id: Uuid,
    pub project_id: Uuid,
}

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS)]
#[sqlx(type_name = "workspace_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceStatus {
    SetupRunning,
    SetupComplete,
    SetupFailed,
    ExecutorRunning,
    ExecutorComplete,
    ExecutorFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Workspace {
    pub id: Uuid,
    pub task_id: Uuid,
    pub name: Option<String>,
    pub status: String,
    pub data: WorkspaceData,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct WorkspaceData {
    pub container_ref: Option<String>,
    pub branch: Option<String>,
    pub agent_working_dir: Option<String>,
    pub setup_completed_at: Option<DateTime<Utc>>,
    pub repos: Vec<WorkspaceRepoData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct WorkspaceRepoData {
    pub repo_id: Uuid,
    pub name: String,
    pub path: Option<String>,
    pub target_branch: String,
}

/// GitHub PR creation parameters
pub struct CreatePrParams<'a> {
    pub workspace_id: Uuid,
    pub task_id: Uuid,
    pub project_id: Uuid,
    pub github_token: &'a str,
    pub title: &'a str,
    pub body: Option<&'a str>,
    pub base_branch: Option<&'a str>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateFollowUpAttempt {
    pub prompt: String,
}

/// Context data for resume operations (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptResumeContext {
    pub execution_history: String,
    pub cumulative_diffs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceContext {
    pub workspace: Workspace,
    pub task: Task,
    pub project: Project,
    pub workspace_repos: Vec<RepoWithTargetBranch>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateWorkspace {
    pub branch: String,
    pub agent_working_dir: Option<String>,
}

impl Workspace {
    pub async fn parent_task(&self, pool: &PgPool) -> Result<Option<Task>, sqlx::Error> {
        Task::find_by_id(pool, self.task_id).await
    }

    /// Fetch all task_workspaces, optionally filtered by task_id. Newest first.
    pub async fn fetch_all(
        pool: &PgPool,
        task_id: Option<Uuid>,
    ) -> Result<Vec<Self>, WorkspaceError> {
        let rows = match task_id {
            Some(tid) => sqlx::query!(
                r#"SELECT id, task_id, name, status, data, created_at, updated_at, deleted_at
                   FROM task_workspaces
                   WHERE task_id = $1 AND deleted_at IS NULL
                   ORDER BY created_at DESC"#,
                tid
            )
            .fetch_all(pool)
            .await
            .map_err(WorkspaceError::Database)?,
            None => sqlx::query!(
                r#"SELECT id, task_id, name, status, data, created_at, updated_at, deleted_at
                   FROM task_workspaces
                   WHERE deleted_at IS NULL
                   ORDER BY created_at DESC"#
            )
            .fetch_all(pool)
            .await
            .map_err(WorkspaceError::Database)?,
        };

        let workspaces: Result<Vec<Self>, _> = rows
            .into_iter()
            .map(|row| {
                let data: WorkspaceData = serde_json::from_value(row.data)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                Ok(Workspace {
                    id: row.id,
                    task_id: row.task_id,
                    name: row.name,
                    status: row.status,
                    data,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    deleted_at: row.deleted_at,
                })
            })
            .collect();

        workspaces.map_err(WorkspaceError::Database)
    }

    /// Fetch all task_workspaces for multiple task IDs
    pub async fn fetch_all_bulk(
        pool: &PgPool,
        task_ids: &[Uuid],
    ) -> Result<Vec<Self>, WorkspaceError> {
        if task_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query!(
            r#"SELECT id, task_id, name, status, data, created_at, updated_at, deleted_at
               FROM task_workspaces
               WHERE task_id = ANY($1) AND deleted_at IS NULL
               ORDER BY created_at DESC"#,
            task_ids
        )
        .fetch_all(pool)
        .await
        .map_err(WorkspaceError::Database)?;

        let workspaces: Result<Vec<Self>, _> = rows
            .into_iter()
            .map(|row| {
                let data: WorkspaceData = serde_json::from_value(row.data)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                Ok(Workspace {
                    id: row.id,
                    task_id: row.task_id,
                    name: row.name,
                    status: row.status,
                    data,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    deleted_at: row.deleted_at,
                })
            })
            .collect();

        workspaces.map_err(WorkspaceError::Database)
    }

    /// Load workspace with full validation - ensures workspace belongs to task and task belongs to project
    pub async fn load_context(
        pool: &PgPool,
        workspace_id: Uuid,
        task_id: Uuid,
        project_id: Uuid,
    ) -> Result<WorkspaceContext, WorkspaceError> {
        let row = sqlx::query!(
            r#"SELECT w.id, w.task_id, w.name, w.status, w.data, w.created_at, w.updated_at, w.deleted_at
               FROM task_workspaces w
               JOIN tasks t ON w.task_id = t.id
               JOIN projects p ON t.project_id = p.id
               WHERE w.id = $1 AND t.id = $2 AND p.id = $3 AND w.deleted_at IS NULL"#,
            workspace_id,
            task_id,
            project_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or(WorkspaceError::TaskNotFound)?;

        let data: WorkspaceData = serde_json::from_value(row.data)
            .map_err(|e| WorkspaceError::Database(sqlx::Error::Decode(Box::new(e))))?;

        let workspace = Workspace {
            id: row.id,
            task_id: row.task_id,
            name: row.name,
            status: row.status,
            data,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        };

        // Load task and project (we know they exist due to JOIN validation)
        let task = Task::find_by_id(pool, task_id)
            .await?
            .ok_or(WorkspaceError::TaskNotFound)?;

        let project = Project::find_by_id(pool, project_id)
            .await?
            .ok_or(WorkspaceError::ProjectNotFound)?;

        // Convert workspace repos to RepoWithTargetBranch format
        let workspace_repos: Vec<RepoWithTargetBranch> = workspace
            .data
            .repos
            .iter()
            .map(|r| RepoWithTargetBranch {
                repo: super::repo::Repo {
                    id: r.repo_id,
                    path: r.path.clone(),
                    name: r.name.clone(),
                    display_name: None,
                    created_at: workspace.created_at,
                    updated_at: workspace.updated_at,
                },
                target_branch: r.target_branch.clone(),
            })
            .collect();

        Ok(WorkspaceContext {
            workspace,
            task,
            project,
            workspace_repos,
        })
    }

    /// Update container reference
    pub async fn update_container_ref(
        pool: &PgPool,
        workspace_id: Uuid,
        container_ref: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE task_workspaces 
               SET data = jsonb_set(
                   COALESCE(data, '{}'::jsonb),
                   '{container_ref}',
                   to_jsonb($1)
               ),
               updated_at = NOW()
               WHERE id = $2"#,
            container_ref,
            workspace_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn clear_container_ref(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE task_workspaces 
               SET data = data - 'container_ref',
               updated_at = NOW()
               WHERE id = $1"#,
            workspace_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Update the workspace's updated_at timestamp to prevent cleanup.
    /// Call this when the workspace is accessed (e.g., opened in editor).
    pub async fn touch(pool: &PgPool, workspace_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE task_workspaces SET updated_at = NOW() WHERE id = $1",
            workspace_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query!(
            r#"SELECT id, task_id, name, status, data, created_at, updated_at, deleted_at
               FROM task_workspaces
               WHERE id = $1 AND deleted_at IS NULL"#,
            id
        )
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let data: WorkspaceData = serde_json::from_value(row.data)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                Ok(Some(Workspace {
                    id: row.id,
                    task_id: row.task_id,
                    name: row.name,
                    status: row.status,
                    data,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    deleted_at: row.deleted_at,
                }))
            }
            None => Ok(None),
        }
    }

    #[allow(dead_code)]
    pub async fn find_by_ctid(_pool: &PgPool, _ctid: i64) -> Result<Option<Self>, sqlx::Error> {
        Ok(None)
    }

    pub async fn container_ref_exists(
        pool: &PgPool,
        container_ref: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT EXISTS(
                SELECT 1 FROM task_workspaces 
                WHERE data->>'container_ref' = $1 
                AND deleted_at IS NULL
            ) as "exists!: bool""#,
            container_ref
        )
        .fetch_one(pool)
        .await?;

        Ok(result.exists)
    }

    /// Find task_workspaces that are expired (72+ hours since last activity) and eligible for cleanup
    pub async fn find_expired_for_cleanup(
        pool: &PgPool,
    ) -> Result<Vec<Workspace>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                w.id,
                w.task_id,
                w.name,
                w.status,
                w.data,
                w.created_at,
                w.updated_at,
                w.deleted_at
            FROM task_workspaces w
            LEFT JOIN sessions s ON w.id = s.workspace_id
            LEFT JOIN execution_processes ep ON s.id = ep.session_id AND ep.data->>'completed_at' IS NOT NULL
            WHERE w.data->>'container_ref' IS NOT NULL
                AND w.deleted_at IS NULL
                AND w.id NOT IN (
                    SELECT DISTINCT s2.workspace_id
                    FROM sessions s2
                    JOIN execution_processes ep2 ON s2.id = ep2.session_id
                    WHERE ep2.data->>'completed_at' IS NULL
                )
            GROUP BY w.id, w.data, w.updated_at
            HAVING NOW() - INTERVAL '72 hours' > MAX(
                    CASE
                        WHEN ep.data->>'completed_at' IS NOT NULL 
                        THEN (ep.data->>'completed_at')::timestamptz
                        ELSE w.updated_at
                    END
                )
            ORDER BY MAX(
                CASE
                    WHEN ep.data->>'completed_at' IS NOT NULL 
                    THEN (ep.data->>'completed_at')::timestamptz
                    ELSE w.updated_at
                END
            ) ASC
            "#
        )
        .fetch_all(pool)
        .await?;

        let workspaces: Result<Vec<Self>, _> = rows
            .into_iter()
            .map(|row| {
                let data: WorkspaceData = serde_json::from_value(row.data)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                Ok(Workspace {
                    id: row.id,
                    task_id: row.task_id,
                    name: row.name,
                    status: row.status,
                    data,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    deleted_at: row.deleted_at,
                })
            })
            .collect();

        workspaces
    }

    pub async fn create(
        pool: &PgPool,
        create_data: &CreateWorkspace,
        id: Uuid,
        task_id: Uuid,
    ) -> Result<Self, WorkspaceError> {
        let data = WorkspaceData {
            container_ref: None,
            branch: Some(create_data.branch.clone()),
            agent_working_dir: create_data.agent_working_dir.clone(),
            setup_completed_at: None,
            repos: Vec::new(),
        };

        let data_json = serde_json::to_value(&data)
            .map_err(|e| WorkspaceError::Database(sqlx::Error::Encode(Box::new(e))))?;

        let row = sqlx::query!(
            r#"INSERT INTO task_workspaces (id, task_id, name, status, data)
               VALUES ($1, $2, $3, 'active', $4)
               RETURNING id, task_id, name, status, data, created_at, updated_at, deleted_at"#,
            id,
            task_id,
            None::<String>,
            data_json
        )
        .fetch_one(pool)
        .await?;

        let returned_data: WorkspaceData = serde_json::from_value(row.data)
            .map_err(|e| WorkspaceError::Database(sqlx::Error::Decode(Box::new(e))))?;

        Ok(Workspace {
            id: row.id,
            task_id: row.task_id,
            name: row.name,
            status: row.status,
            data: returned_data,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }

    pub async fn update_branch_name(
        pool: &PgPool,
        workspace_id: Uuid,
        new_branch_name: &str,
    ) -> Result<(), WorkspaceError> {
        sqlx::query!(
            r#"UPDATE task_workspaces 
               SET data = jsonb_set(
                   COALESCE(data, '{}'::jsonb),
                   '{branch}',
                   to_jsonb($1)
               ),
               updated_at = NOW()
               WHERE id = $2"#,
            new_branch_name,
            workspace_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn resolve_container_ref(
        pool: &PgPool,
        container_ref: &str,
    ) -> Result<ContainerInfo, sqlx::Error> {
         let result = sqlx::query!(
            r#"SELECT 
                w.id as workspace_id,
                w.task_id as task_id,
                t.project_id as project_id
               FROM task_workspaces w
               JOIN tasks t ON w.task_id = t.id
               WHERE w.data->>'container_ref' = $1 
               AND w.deleted_at IS NULL"#,
            container_ref
        )
        .fetch_optional(pool)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

        Ok(ContainerInfo {
            workspace_id: result.workspace_id,
            task_id: result.task_id,
            project_id: result.project_id,
        })
    }

    /// Stub for PostgreSQL - rowid is SQLite-specific
    /// In PostgreSQL, we use UUID primary keys
    #[allow(dead_code)]
    pub async fn find_by_rowid(_pool: &PgPool, _rowid: i64) -> Result<Option<Self>, sqlx::Error> {
        // PostgreSQL doesn't have rowid, this is a stub for compatibility
        Ok(None)
    }

    // Helper methods for backward compatibility
    pub fn container_ref(&self) -> Option<&String> {
        self.data.container_ref.as_ref()
    }

    pub fn branch(&self) -> Option<&String> {
        self.data.branch.as_ref()
    }

    pub fn agent_working_dir(&self) -> Option<&String> {
        self.data.agent_working_dir.as_ref()
    }

    pub fn setup_completed_at(&self) -> Option<DateTime<Utc>> {
        self.data.setup_completed_at
    }

    pub fn repos(&self) -> &[WorkspaceRepoData] {
        &self.data.repos
    }
}
