use chrono::{DateTime, Utc};
use executors::{
    actions::{ExecutorAction, ExecutorActionType},
    profile::ExecutorProfileId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, PgPool, Type};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use super::{
    execution_process_repo_state::CreateExecutionProcessRepoState,
    project::Project,
    repo::Repo,
    session::Session,
    task::Task,
    workspace::Workspace,
    workspace_repo::WorkspaceRepo,
};

#[derive(Debug, Error)]
pub enum ExecutionProcessError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Execution process not found")]
    ExecutionProcessNotFound,
    #[error("Failed to create execution process: {0}")]
    CreateFailed(String),
    #[error("Failed to update execution process: {0}")]
    UpdateFailed(String),
    #[error("Invalid executor action format")]
    InvalidExecutorAction,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS)]
#[sqlx(type_name = "execution_process_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[ts(use_ts_enum)]
pub enum ExecutionProcessStatus {
    Running,
    Completed,
    Failed,
    Killed,
}

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS)]
#[sqlx(type_name = "execution_process_run_reason", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ExecutionProcessRunReason {
    SetupScript,
    CleanupScript,
    CodingAgent,
    DevServer,
}

/// ExecutionProcess data stored in JSONB
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionProcessData {
    #[serde(default)]
    pub executor_action: Option<Value>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub dropped: bool,
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub repo_states: Vec<ExecutionProcessRepoStateData>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Repo state data embedded in execution_processes.data JSONB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProcessRepoStateData {
    pub repo_id: Uuid,
    #[serde(default)]
    pub before_head_commit: Option<String>,
    #[serde(default)]
    pub after_head_commit: Option<String>,
    #[serde(default)]
    pub merge_commit: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct ExecutionProcess {
    pub id: Uuid,
    pub session_id: Uuid,
    pub run_reason: ExecutionProcessRunReason,
    pub status: ExecutionProcessStatus,
    /// DDD: All process data stored in JSONB
    #[ts(type = "ExecutionProcessData")]
    pub data: sqlx::types::Json<ExecutionProcessData>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateExecutionProcess {
    pub session_id: Uuid,
    pub executor_action: ExecutorAction,
    pub run_reason: ExecutionProcessRunReason,
}

#[derive(Debug, Deserialize, TS)]
#[allow(dead_code)]
pub struct UpdateExecutionProcess {
    pub status: Option<ExecutionProcessStatus>,
    pub exit_code: Option<i32>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct ExecutionContext {
    pub execution_process: ExecutionProcess,
    pub session: Session,
    pub workspace: Workspace,
    pub task: Task,
    pub project: Project,
    pub repos: Vec<Repo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExecutorActionField {
    ExecutorAction(ExecutorAction),
    Other(Value),
}

#[derive(Debug, Clone, FromRow)]
pub struct MissingBeforeContext {
    pub id: Uuid,
    pub session_id: Uuid,
    pub workspace_id: Uuid,
    pub repo_id: Uuid,
    pub prev_after_head_commit: Option<String>,
    pub target_branch: String,
    pub repo_path: Option<String>,
}

impl ExecutionProcess {
    /// Helper: Get executor_action from JSONB data
    pub fn executor_action(&self) -> Result<&ExecutorAction, anyhow::Error> {
        match &self.data.executor_action {
            Some(val) => {
                let action: ExecutorAction = serde_json::from_value(val.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to parse executor_action: {}", e))?;
                Err(anyhow::anyhow!("executor_action must be accessed via data field directly"))
            }
            None => Err(anyhow::anyhow!("No executor_action in data")),
        }
    }

    /// Helper: Get executor_action as Value for serialization
    pub fn executor_action_value(&self) -> Option<&Value> {
        self.data.executor_action.as_ref()
    }

    /// Helper: Get exit_code from JSONB data
    pub fn exit_code(&self) -> Option<i32> {
        self.data.exit_code
    }

    /// Helper: Get dropped from JSONB data
    pub fn dropped(&self) -> bool {
        self.data.dropped
    }

    /// Helper: Get started_at from JSONB data
    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.data.started_at
    }

    /// Helper: Get repo_states from JSONB data
    pub fn repo_states(&self) -> &[ExecutionProcessRepoStateData] {
        &self.data.repo_states
    }

    /// Helper: Get completed_at (alias for backward compatibility)
    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        if let Some(completed_at) = self.data.extra.get("completed_at") {
            if let Ok(dt) = serde_json::from_value::<DateTime<Utc>>(completed_at.clone()) {
                return Some(dt);
            }
        }
        None
    }

    /// Find execution process by ID
    /// DDD: Query from JSONB data field using JSONB operators
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            ExecutionProcess,
            r#"SELECT
                    id              as "id!: Uuid",
                    session_id      as "session_id!: Uuid",
                    run_reason      as "run_reason!: ExecutionProcessRunReason",
                    status          as "status!: ExecutionProcessStatus",
                    data             as "data!: sqlx::types::Json<ExecutionProcessData>"
               FROM execution_processes
               WHERE id = $1 AND deleted_at IS NULL"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    /// Context for backfilling before_head_commit for legacy rows
    /// DDD: Query repo_states from JSONB data
    pub async fn list_missing_before_context(
        pool: &PgPool,
    ) -> Result<Vec<MissingBeforeContext>, sqlx::Error> {
        let rows: Vec<MissingBeforeContext> = sqlx::query_as(
            r#"
            SELECT
                ep.id as id,
                ep.session_id as session_id,
                s.workspace_id as workspace_id,
                (rs.elem->>'repo_id')::uuid as repo_id,
                (prev_rs.elem->>'after_head_commit') as prev_after_head_commit,
                wr.target_branch as target_branch,
                r.path as repo_path
            FROM execution_processes ep
            JOIN sessions s ON s.id = ep.session_id
            CROSS JOIN LATERAL jsonb_array_elements(ep.data->'repo_states') AS rs(elem)
            JOIN repos r ON r.id = (rs.elem->>'repo_id')::uuid
            JOIN task_workspaces w ON w.id = s.workspace_id
            JOIN workspace_repos wr ON wr.workspace_id = w.id AND wr.repo_id = (rs.elem->>'repo_id')::uuid
            LEFT JOIN LATERAL (
                SELECT jsonb_array_elements(prev_ep.data->'repo_states') as elem
                FROM execution_processes prev_ep
                WHERE prev_ep.session_id = ep.session_id
                  AND prev_ep.created_at < ep.created_at
                  AND prev_ep.deleted_at IS NULL
                ORDER BY prev_ep.created_at DESC
                LIMIT 1
            ) prev_rs ON true
            WHERE ep.deleted_at IS NULL
              AND rs.elem->>'before_head_commit' IS NULL
              AND rs.elem->>'after_head_commit' IS NOT NULL
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    #[allow(dead_code)]
    pub async fn find_by_rowid(_pool: &PgPool, _rowid: i64) -> Result<Option<Self>, sqlx::Error> {
        Ok(None)
    }

    /// Find execution processes for a session with optional pagination
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `session_id` - Session ID to filter by
    /// * `show_soft_deleted` - Whether to include soft-deleted processes
    /// * `limit` - Maximum number of processes to return (None for all)
    /// * `offset` - Number of processes to skip (None for 0)
    pub async fn find_by_session_id(
        pool: &PgPool,
        session_id: Uuid,
        show_soft_deleted: bool,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = offset.unwrap_or(0);

        if let Some(limit_val) = limit {
            sqlx::query_as!(
                ExecutionProcess,
                r#"SELECT
                          id              as "id!: Uuid",
                          session_id      as "session_id!: Uuid",
                          run_reason      as "run_reason!: ExecutionProcessRunReason",
                          status          as "status!: ExecutionProcessStatus",
                          data            as "data!: sqlx::types::Json<ExecutionProcessData>"
                   FROM execution_processes
                   WHERE session_id = $1
                      AND ($2 OR (data->>'dropped')::bool = FALSE)
                    ORDER BY created_at ASC
                    LIMIT $3 OFFSET $4"#,
                session_id,
                show_soft_deleted,
                limit_val,
                offset
            )
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as!(
                ExecutionProcess,
                r#"SELECT
                          id              as "id!: Uuid",
                          session_id      as "session_id!: Uuid",
                          run_reason      as "run_reason!: ExecutionProcessRunReason",
                          status          as "status!: ExecutionProcessStatus",
                          data            as "data!: sqlx::types::Json<ExecutionProcessData>"
                   FROM execution_processes
                   WHERE session_id = $1
                      AND ($2 OR (data->>'dropped')::bool = FALSE)
                    ORDER BY created_at ASC
                    OFFSET $3"#,
                session_id,
                show_soft_deleted,
                offset
            )
            .fetch_all(pool)
            .await
        }
    }

    /// Find running execution processes
    pub async fn find_running(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            ExecutionProcess,
            r#"SELECT
                    id as "id!: Uuid",
                    session_id as "session_id!: Uuid",
                    run_reason as "run_reason!: ExecutionProcessRunReason",
                    status as "status!: ExecutionProcessStatus",
                    data as "data!: sqlx::types::Json<ExecutionProcessData>"
               FROM execution_processes WHERE status = 'running' ORDER BY created_at ASC"#,
        )
        .fetch_all(pool)
        .await
    }

    /// Find running dev servers for a specific project
    pub async fn find_running_dev_servers_by_project(
        pool: &PgPool,
        project_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            ExecutionProcess,
            r#"SELECT
                    ep.id as "id!: Uuid",
                    ep.session_id as "session_id!: Uuid",
                    ep.run_reason as "run_reason!: ExecutionProcessRunReason",
                    ep.status as "status!: ExecutionProcessStatus",
                    ep.data as "data!: sqlx::types::Json<ExecutionProcessData>"
               FROM execution_processes ep
               JOIN sessions s ON ep.session_id = s.id
               JOIN task_workspaces w ON s.workspace_id = w.id
               JOIN tasks t ON w.task_id = t.id
               WHERE ep.status = 'running' AND ep.run_reason = 'devserver' AND t.project_id = $1
               ORDER BY ep.created_at ASC"#,
            project_id
        )
        .fetch_all(pool)
        .await
    }

    /// Check if there are running processes (excluding dev servers) for a workspace (across all sessions)
    pub async fn has_running_non_dev_server_processes_for_workspace(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64"
               FROM execution_processes ep
               JOIN sessions s ON ep.session_id = s.id
               WHERE s.workspace_id = $1
                  AND ep.status = 'running'
                  AND ep.run_reason != 'devserver'"#,
            workspace_id
        )
        .fetch_one(pool)
        .await?;
        Ok(count > 0)
    }

    /// Find running dev servers for a specific workspace (across all sessions)
    pub async fn find_running_dev_servers_by_workspace(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            ExecutionProcess,
            r#"
        SELECT
            ep.id as "id!: Uuid",
            ep.session_id as "session_id!: Uuid",
            ep.run_reason as "run_reason!: ExecutionProcessRunReason",
            ep.status as "status!: ExecutionProcessStatus",
            ep.data as "data!: sqlx::types::Json<ExecutionProcessData>"
        FROM execution_processes ep
        JOIN sessions s ON ep.session_id = s.id
        WHERE s.workspace_id = $1
          AND ep.status = 'running'
          AND ep.run_reason = 'devserver'
        ORDER BY ep.created_at DESC
        "#,
            workspace_id
        )
        .fetch_all(pool)
        .await
    }

    /// Find latest coding_agent_turn agent_session_id by session (simple scalar query)
    pub async fn find_latest_coding_agent_turn_session_id(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<Option<String>, sqlx::Error> {
        tracing::info!(
            "Finding latest coding agent turn session id for session {}",
            session_id
        );
        let row: Option<(String,)> = sqlx::query_as(
            r#"SELECT cat.agent_session_id as agent_session_id
               FROM execution_processes ep
               JOIN coding_agent_turns cat ON ep.id = cat.execution_process_id
               WHERE ep.session_id = $1
                 AND ep.run_reason = 'codingagent'
                 AND ep.dropped = FALSE
                 AND cat.agent_session_id IS NOT NULL
               ORDER BY ep.created_at DESC
               LIMIT 1"#,
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await?;

        tracing::info!("Latest coding agent turn session id: {:?}", row);

        Ok(row.map(|r| r.0))
    }

    /// Find latest execution process by session and run reason
    pub async fn find_latest_by_session_and_run_reason(
        pool: &PgPool,
        session_id: Uuid,
        run_reason: &ExecutionProcessRunReason,
    ) -> Result<Option<Self>, sqlx::Error> {
        let run_reason_str = match run_reason {
            ExecutionProcessRunReason::SetupScript => "setupscript",
            ExecutionProcessRunReason::CleanupScript => "cleanupscript",
            ExecutionProcessRunReason::CodingAgent => "codingagent",
            ExecutionProcessRunReason::DevServer => "devserver",
        };
        sqlx::query_as!(
            ExecutionProcess,
            r#"SELECT
                    ep.id as "id!: Uuid",
                    ep.session_id as "session_id!: Uuid",
                    ep.run_reason as "run_reason!: ExecutionProcessRunReason",
                    ep.status as "status!: ExecutionProcessStatus",
                    ep.data as "data!: sqlx::types::Json<ExecutionProcessData>"
               FROM execution_processes ep
               WHERE ep.session_id = $1 AND ep.run_reason = $2 AND (ep.data->>'dropped')::bool = FALSE
               ORDER BY ep.created_at DESC LIMIT 1"#,
            session_id,
            run_reason_str
        )
        .fetch_optional(pool)
        .await
    }

    /// Find latest execution process by workspace and run reason (across all sessions)
    pub async fn find_latest_by_workspace_and_run_reason(
        pool: &PgPool,
        workspace_id: Uuid,
        run_reason: &ExecutionProcessRunReason,
    ) -> Result<Option<Self>, sqlx::Error> {
        let run_reason_str = match run_reason {
            ExecutionProcessRunReason::SetupScript => "setupscript",
            ExecutionProcessRunReason::CleanupScript => "cleanupscript",
            ExecutionProcessRunReason::CodingAgent => "codingagent",
            ExecutionProcessRunReason::DevServer => "devserver",
        };
        sqlx::query_as!(
            ExecutionProcess,
            r#"SELECT
                    ep.id as "id!: Uuid",
                    ep.session_id as "session_id!: Uuid",
                    ep.run_reason as "run_reason!: ExecutionProcessRunReason",
                    ep.status as "status!: ExecutionProcessStatus",
                    ep.data as "data!: sqlx::types::Json<ExecutionProcessData>"
               FROM execution_processes ep
               JOIN sessions s ON ep.session_id = s.id
               WHERE s.workspace_id = $1 AND ep.run_reason = $2 AND (ep.data->>'dropped')::bool = FALSE
               ORDER BY ep.created_at DESC LIMIT 1"#,
            workspace_id,
            run_reason_str
        )
        .fetch_optional(pool)
        .await
    }

    /// Create a new execution process
    ///
    /// Note: We intentionally avoid using a transaction here. SQLite update
    /// hooks fire during transactions (before commit), and the hook spawns an
    /// async task that queries `find_by_rowid` on a different connection.
    /// If we used a transaction, that query would not see the uncommitted row,
    /// causing the WebSocket event to be lost.
    pub async fn create(
        pool: &PgPool,
        data: &CreateExecutionProcess,
        process_id: Uuid,
        repo_states: &[CreateExecutionProcessRepoState],
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        let run_reason_str = match data.run_reason {
            ExecutionProcessRunReason::SetupScript => "setupscript",
            ExecutionProcessRunReason::CleanupScript => "cleanupscript",
            ExecutionProcessRunReason::CodingAgent => "codingagent",
            ExecutionProcessRunReason::DevServer => "devserver",
        };
        let status_str = "running";

        // Build the data JSONB object
        let data_json = serde_json::json!({
            "executor_action": &data.executor_action,
            "exit_code": None::<i32>,
            "dropped": false,
            "started_at": now,
            "completed_at": None::<DateTime<Utc>>,
            "repo_states": repo_states.iter().map(|rs| serde_json::json!({
                "repo_id": rs.repo_id,
                "before_head_commit": rs.before_head_commit,
                "after_head_commit": rs.after_head_commit,
                "merge_commit": rs.merge_commit,
            })).collect::<Vec<_>>(),
        });

        sqlx::query!(
            r#"INSERT INTO execution_processes (
                    id, session_id, run_reason, status, data, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            process_id,
            data.session_id,
            run_reason_str,
            status_str,
            data_json,
            now,
            now
        )
        .execute(pool)
        .await?;

        Self::find_by_id(pool, process_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn was_stopped(pool: &PgPool, id: Uuid) -> bool {
        if let Ok(exp_process) = Self::find_by_id(pool, id).await
            && exp_process.is_some_and(|ep| {
                ep.status == ExecutionProcessStatus::Killed
                    || ep.status == ExecutionProcessStatus::Completed
            })
        {
            return true;
        }
        false
    }

    /// Update execution process status and completion info
    pub async fn update_completion(
        pool: &PgPool,
        id: Uuid,
        status: ExecutionProcessStatus,
        exit_code: Option<i32>,
    ) -> Result<(), sqlx::Error> {
        let completed_at = if matches!(status, ExecutionProcessStatus::Running) {
            None
        } else {
            Some(Utc::now())
        };

        let status_str = match status {
            ExecutionProcessStatus::Running => "running",
            ExecutionProcessStatus::Completed => "completed",
            ExecutionProcessStatus::Failed => "failed",
            ExecutionProcessStatus::Killed => "killed",
        };

        sqlx::query!(
            r#"UPDATE execution_processes
               SET status = $1,
                   data = jsonb_set(
                       jsonb_set(data, '{exit_code}', $2::jsonb),
                       '{completed_at}', $3::jsonb
                   ),
                   updated_at = NOW()
               WHERE id = $4"#,
            status_str,
            serde_json::json!(exit_code),
            serde_json::json!(completed_at),
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Soft-drop processes at and after the specified boundary (inclusive)
    pub async fn drop_at_and_after(
        pool: &PgPool,
        session_id: Uuid,
        boundary_process_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let result: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"UPDATE execution_processes
               SET data = jsonb_set(data, '{dropped}', 'true'::jsonb)
             WHERE session_id = $1
               AND created_at >= (SELECT created_at FROM execution_processes WHERE id = $2)
               AND (data->>'dropped')::bool = FALSE"#,
            session_id,
            boundary_process_id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    /// Find the previous process's after_head_commit before the given boundary process
    /// for a specific repository
    pub async fn find_prev_after_head_commit(
        pool: &PgPool,
        session_id: Uuid,
        boundary_process_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Option<String>, sqlx::Error> {
        let result: Option<(Option<String>,)> = sqlx::query_as(
            r#"SELECT eprs.after_head_commit as after_head_commit
               FROM execution_process_repo_states eprs
               JOIN execution_processes ep ON ep.id = eprs.execution_process_id
               WHERE ep.session_id = $1
                 AND eprs.repo_id = $2
                 AND ep.created_at < (SELECT created_at FROM execution_processes WHERE id = $3)
              ORDER BY ep.created_at DESC
              LIMIT 1"#,
        )
        .bind(session_id)
        .bind(repo_id)
        .bind(boundary_process_id)
        .fetch_optional(pool)
        .await?;
        Ok(result.and_then(|r| r.0))
    }

    /// Get the parent Session for this execution process
    pub async fn parent_session(&self, pool: &PgPool) -> Result<Option<Session>, sqlx::Error> {
        Session::find_by_id(pool, self.session_id).await
    }

    /// Get both the parent Workspace and Session for this execution process
    pub async fn parent_workspace_and_session(
        &self,
        pool: &PgPool,
    ) -> Result<Option<(Workspace, Session)>, sqlx::Error> {
        let session = match Session::find_by_id(pool, self.session_id).await? {
            Some(s) => s,
            None => return Ok(None),
        };
        let workspace = match Workspace::find_by_id(pool, session.workspace_id).await? {
            Some(w) => w,
            None => return Ok(None),
        };
        Ok(Some((workspace, session)))
    }

    /// Load execution context with related session, workspace, task, project, and repos
    pub async fn load_context(
        pool: &PgPool,
        exec_id: Uuid,
    ) -> Result<ExecutionContext, sqlx::Error> {
        let execution_process = Self::find_by_id(pool, exec_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let session = Session::find_by_id(pool, execution_process.session_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let workspace = Workspace::find_by_id(pool, session.workspace_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let task = Task::find_by_id(pool, workspace.task_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let project = Project::find_by_id(pool, task.project_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let repos = WorkspaceRepo::find_repos_for_workspace(pool, workspace.id).await?;

        Ok(ExecutionContext {
            execution_process,
            session,
            workspace,
            task,
            project,
            repos,
        })
    }

    /// Fetch the latest CodingAgent executor profile for a session
    pub async fn latest_executor_profile_for_session(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<ExecutorProfileId, ExecutionProcessError> {
        // Find the latest CodingAgent execution process for this session
        let run_reason_str = "codingagent";
        let latest_execution_process = sqlx::query_as!(
            ExecutionProcess,
            r#"SELECT
                    ep.id as "id!: Uuid",
                    ep.session_id as "session_id!: Uuid",
                    ep.run_reason as "run_reason!: ExecutionProcessRunReason",
                    ep.status as "status!: ExecutionProcessStatus",
                    ep.data as "data!: sqlx::types::Json<ExecutionProcessData>"
               FROM execution_processes ep
               WHERE ep.session_id = $1 AND ep.run_reason = $2 AND (ep.data->>'dropped')::bool = FALSE
               ORDER BY ep.created_at DESC LIMIT 1"#,
            session_id,
            run_reason_str
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            ExecutionProcessError::ValidationError(
                "Couldn't find initial coding agent process, has it run yet?".to_string(),
            )
        })?;

        let action = latest_execution_process
            .executor_action()
            .map_err(|e| ExecutionProcessError::ValidationError(e.to_string()))?;

        match &action.typ {
            ExecutorActionType::CodingAgentInitialRequest(request) => {
                Ok(request.executor_profile_id.clone())
            }
            ExecutorActionType::CodingAgentFollowUpRequest(request) => {
                Ok(request.executor_profile_id.clone())
            }
            _ => Err(ExecutionProcessError::ValidationError(
                "Couldn't find profile from initial request".to_string(),
            )),
        }
    }
}
