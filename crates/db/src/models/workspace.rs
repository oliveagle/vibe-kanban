use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use super::{
    project::Project,
    task::Task,
    workspace_repo::RepoWithTargetBranch,
};

/// Workspace data stored in JSONB
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorkspaceData {
    pub container_ref: Option<String>,
    pub branch: Option<String>,
    pub agent_working_dir: Option<String>,
    #[ts(type = "Date | null")]
    pub setup_completed_at: Option<DateTime<Utc>>,
}

impl Default for WorkspaceData {
    fn default() -> Self {
        Self {
            container_ref: None,
            branch: None,
            agent_working_dir: None,
            setup_completed_at: None,
        }
    }
}

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

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Workspace {
    pub id: Uuid,
    pub task_id: Uuid,
    pub status: String,
    pub data: sqlx::types::Json<WorkspaceData>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// GitHub PR creation parameters
pub struct CreatePrParams<'a> {
    pub workspace_id: Uuid,
    pub task_id: Uuid,
    pub project_id: Uuid,
    pub github_token: &'a str,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub base_branch: &'a str,
    pub head_branch: &'a str,
    pub title: &'a str,
    pub body: &'a str,
}

pub struct WorkspaceContext {
    pub workspace: Workspace,
    pub repos: Vec<super::workspace_repo::WorkspaceRepo>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateWorkspace {
    pub branch: String,
    pub agent_working_dir: Option<String>,
}

impl Workspace {
    /// Fetch all workspaces, optionally filtered by task_id. Newest first.
    pub async fn fetch_all(
        pool: &PgPool,
        task_id: Option<Uuid>,
    ) -> Result<Vec<Self>, WorkspaceError> {
        let workspaces = match task_id {
            Some(tid) => sqlx::query_as!(
                Workspace,
                r#"SELECT id AS "id!: Uuid",
                              task_id AS "task_id!: Uuid",
                              status,
                              data AS "data!: sqlx::types::Json<WorkspaceData>",
                              created_at AS "created_at!: DateTime<Utc>",
                              updated_at AS "updated_at!: DateTime<Utc>",
                              deleted_at AS "deleted_at!: Option<DateTime<Utc>>"
                       FROM workspaces
                       WHERE task_id = $1 AND deleted_at IS NULL
                       ORDER BY created_at DESC"#,
                tid
            )
            .fetch_all(pool)
            .await
            .map_err(WorkspaceError::Database)?,
            None => sqlx::query_as!(
                Workspace,
                r#"SELECT id AS "id!: Uuid",
                              task_id AS "task_id!: Uuid",
                              status,
                              data AS "data!: sqlx::types::Json<WorkspaceData>",
                              created_at AS "created_at!: DateTime<Utc>",
                              updated_at AS "updated_at!: DateTime<Utc>",
                              deleted_at AS "deleted_at!: Option<DateTime<Utc>>"
                       FROM workspaces
                       WHERE deleted_at IS NULL
                       ORDER BY created_at DESC"#
            )
            .fetch_all(pool)
            .await
            .map_err(WorkspaceError::Database)?,
        };

        Ok(workspaces)
    }

    /// Fetch all workspaces for a list of task IDs
    pub async fn fetch_all_bulk(
        pool: &PgPool,
        task_ids: &[Uuid],
    ) -> Result<Vec<Self>, WorkspaceError> {
        if task_ids.is_empty() {
            return Ok(Vec::new());
        }

        let workspaces = sqlx::query_as!(
            Workspace,
            r#"SELECT id AS "id!: Uuid",
                          task_id AS "task_id!: Uuid",
                          status,
                          data AS "data!: sqlx::types::Json<WorkspaceData>",
                          created_at AS "created_at!: DateTime<Utc>",
                          updated_at AS "updated_at!: DateTime<Utc>",
                          deleted_at AS "deleted_at!: Option<DateTime<Utc>>"
                   FROM workspaces
                   WHERE task_id = ANY($1) AND deleted_at IS NULL
                   ORDER BY created_at DESC"#,
            task_ids
        )
        .fetch_all(pool)
        .await
        .map_err(WorkspaceError::Database)?;

        Ok(workspaces)
    }

    /// Load the full context for a workspace
    pub async fn load_context(
        pool: &PgPool,
        workspace_id: Uuid,
        task_id: Uuid,
        project_id: Uuid,
    ) -> Result<WorkspaceContext, WorkspaceError> {
        let workspace = sqlx::query_as!(
            Workspace,
            r#"SELECT  w.id                AS "id!: Uuid",
                        w.task_id           AS "task_id!: Uuid",
                        w.status,
                        w.data              AS "data!: sqlx::types::Json<WorkspaceData>",
                        w.created_at        AS "created_at!: DateTime<Utc>",
                        w.updated_at        AS "updated_at!: DateTime<Utc>",
                        w.deleted_at        AS "deleted_at!: Option<DateTime<Utc>>"
                 FROM workspaces w
                 JOIN tasks t ON w.task_id = t.id
                 JOIN projects p ON t.project_id = p.id
                 WHERE w.id = $1 AND t.id = $2 AND p.id = $3 AND w.deleted_at IS NULL"#,
            workspace_id,
            task_id,
            project_id
        )
        .fetch_optional(pool)
        .await
        .map_err(WorkspaceError::Database)?
        .ok_or(WorkspaceError::TaskNotFound)?;

        // Load associated repos
        let repos = super::workspace_repo::WorkspaceRepo::find_by_workspace_id(pool, workspace_id)
            .await
            .map_err(WorkspaceError::Database)?;

        Ok(WorkspaceContext {
            workspace,
            repos,
        })
    }

    /// Update the container reference for a workspace
    pub async fn update_container_ref(
        pool: &PgPool,
        workspace_id: Uuid,
        container_ref: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE workspaces SET data = jsonb_set(data, '{container_ref}', to_jsonb($1)), updated_at = NOW() WHERE id = $2",
            container_ref,
            workspace_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Clear the container reference for a workspace
    pub async fn clear_container_ref(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE workspaces SET data = data - 'container_ref', updated_at = NOW() WHERE id = $1",
            workspace_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Workspace,
            r#"SELECT  id                AS "id!: Uuid",
                        task_id           AS "task_id!: Uuid",
                        status,
                        data              AS "data!: sqlx::types::Json<WorkspaceData>",
                        created_at        AS "created_at!: DateTime<Utc>",
                        updated_at        AS "updated_at!: DateTime<Utc>",
                        deleted_at        AS "deleted_at!: Option<DateTime<Utc>>"
                 FROM workspaces
                 WHERE id = $1 AND deleted_at IS NULL"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn container_ref_exists(
        pool: &PgPool,
        container_ref: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT EXISTS(SELECT 1 FROM workspaces WHERE data->>'container_ref' = $1 AND deleted_at IS NULL) as "exists!: bool""#,
            container_ref
        )
        .fetch_one(pool)
        .await?;

        Ok(result.exists)
    }

    /// Find workspaces that are expired and ready for cleanup
    pub async fn find_expired_for_cleanup(
        pool: &PgPool,
    ) -> Result<Vec<Workspace>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                w.id                AS "id!: Uuid",
                w.task_id           AS "task_id!: Uuid",
                w.status,
                w.data              AS "data!: sqlx::types::Json<WorkspaceData>",
                w.created_at        AS "created_at!: DateTime<Utc>",
                w.updated_at        AS "updated_at!: DateTime<Utc>",
                w.deleted_at        AS "deleted_at!: Option<DateTime<Utc>>"
            FROM workspaces w
            LEFT JOIN sessions s ON w.id = s.workspace_id
            LEFT JOIN execution_processes ep ON s.id = ep.session_id
            WHERE w.deleted_at IS NULL
            AND w.id NOT IN (
                    SELECT DISTINCT s2.workspace_id
                    FROM sessions s2
                    JOIN execution_processes ep2 ON s2.id = ep2.session_id
                    WHERE ep2.data->>'completed_at' IS NULL
                )
            GROUP BY w.id, w.updated_at
            HAVING NOW() - INTERVAL '72 hours' > MAX(
                    CASE
                        WHEN ep.data->>'completed_at' IS NOT NULL 
                        THEN (ep.data->>'completed_at')::timestamptz
                        ELSE w.updated_at
                    END
                )
            ORDER BY w.updated_at ASC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn create(
        pool: &PgPool,
        create_data: &CreateWorkspace,
        id: Uuid,
        task_id: Uuid,
    ) -> Result<Self, WorkspaceError> {
        let workspace_data = WorkspaceData {
            container_ref: None,
            branch: Some(create_data.branch.clone()),
            agent_working_dir: create_data.agent_working_dir.clone(),
            setup_completed_at: None,
        };

        let row = sqlx::query_as!(
            Workspace,
            r#"INSERT INTO workspaces (id, task_id, status, data)
               VALUES ($1, $2, 'active', $3)
               RETURNING id AS "id!: Uuid",
                         task_id AS "task_id!: Uuid",
                         status,
                         data AS "data!: sqlx::types::Json<WorkspaceData>",
                         created_at AS "created_at!: DateTime<Utc>",
                         updated_at AS "updated_at!: DateTime<Utc>",
                         deleted_at AS "deleted_at!: Option<DateTime<Utc>>""#,
            id,
            task_id,
            workspace_data
        )
        .fetch_one(pool)
        .await
        .map_err(WorkspaceError::Database)?;

        Ok(row)
    }

    pub async fn update_branch_name(
        pool: &PgPool,
        workspace_id: Uuid,
        new_branch_name: &str,
    ) -> Result<(), WorkspaceError> {
        sqlx::query!(
            "UPDATE workspaces SET data = jsonb_set(data, '{branch}', to_jsonb($1)), updated_at = NOW() WHERE id = $2",
            new_branch_name,
            workspace_id
        )
        .execute(pool)
        .await
        .map_err(WorkspaceError::Database)?;

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
               FROM workspaces w
               JOIN tasks t ON w.task_id = t.id
               WHERE w.data->>'container_ref' = $1 
               AND w.deleted_at IS NULL"#,
            container_ref
        )
        .fetch_optional(pool)
        .await?;

        match result {
            Some(r) => Ok(ContainerInfo {
                workspace_id: r.workspace_id,
                task_id: r.task_id,
                project_id: r.project_id,
            }),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    pub async fn find_by_rowid(_pool: &PgPool, _rowid: i64) -> Result<Option<Self>, sqlx::Error> {
        // PostgreSQL doesn't have rowid, this is a stub for compatibility
        Ok(None)
    }

    // Helper methods to access data fields
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
}