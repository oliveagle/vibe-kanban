use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, Postgres, PgPool, Type};
use strum_macros::{Display, EnumString};
use ts_rs::TS;
use uuid::Uuid;

use super::{project::Project, workspace::Workspace};

#[derive(
    Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS, EnumString, Display, Default,
)]
#[sqlx(type_name = "task_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TaskStatus {
    #[default]
    Todo,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Task {
    pub id: Uuid,
    pub project_id: Uuid, // Foreign key to Project
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub parent_workspace_id: Option<Uuid>, // Foreign key to parent Workspace
    pub shared_task_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TaskWithAttemptStatus {
    #[serde(flatten)]
    #[ts(flatten)]
    pub task: Task,
    pub has_in_progress_attempt: bool,
    pub last_attempt_failed: bool,
    pub executor: String,
}

impl std::ops::Deref for TaskWithAttemptStatus {
    type Target = Task;
    fn deref(&self) -> &Self::Target {
        &self.task
    }
}

impl std::ops::DerefMut for TaskWithAttemptStatus {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.task
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TaskRelationships {
    pub parent_task: Option<Task>, // The task that owns the parent workspace
    pub current_workspace: Workspace, // The workspace we're viewing
    pub children: Vec<Task>,       // Tasks created from this workspace
}

/// Query row for find_by_project_id_with_attempt_status
#[derive(Debug, Clone, FromRow)]
pub struct TaskWithStatusRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub parent_workspace_id: Option<Uuid>,
    pub shared_task_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub has_in_progress_attempt: i64,
    pub last_attempt_failed: i64,
    pub executor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct CreateTask {
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub parent_workspace_id: Option<Uuid>,
    pub image_ids: Option<Vec<Uuid>>,
    pub shared_task_id: Option<Uuid>,
}

impl CreateTask {
    pub fn from_title_description(
        project_id: Uuid,
        title: String,
        description: Option<String>,
    ) -> Self {
        Self {
            project_id,
            title,
            description,
            status: Some(TaskStatus::Todo),
            parent_workspace_id: None,
            image_ids: None,
            shared_task_id: None,
        }
    }

    pub fn from_shared_task(
        project_id: Uuid,
        title: String,
        description: Option<String>,
        status: TaskStatus,
        shared_task_id: Uuid,
    ) -> Self {
        Self {
            project_id,
            title,
            description,
            status: Some(status),
            parent_workspace_id: None,
            image_ids: None,
            shared_task_id: Some(shared_task_id),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub parent_workspace_id: Option<Uuid>,
    pub image_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct BulkUpdateTasks {
    pub task_ids: Vec<Uuid>,
    pub status: Option<TaskStatus>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct BulkDeleteTasks {
    pub task_ids: Vec<Uuid>,
}

impl Task {
    pub fn to_prompt(&self) -> String {
        if let Some(description) = self.description.as_ref().filter(|d| !d.trim().is_empty()) {
            format!("{}\n\n{}", &self.title, description)
        } else {
            self.title.clone()
        }
    }

    pub async fn parent_project(&self, pool: &PgPool) -> Result<Option<Project>, sqlx::Error> {
        Project::find_by_id(pool, self.project_id).await
    }

    pub async fn find_by_project_id_with_attempt_status(
        pool: &PgPool,
        project_id: Uuid,
    ) -> Result<Vec<TaskWithAttemptStatus>, sqlx::Error> {
        let records: Vec<TaskWithStatusRow> = sqlx::query_as!(
            TaskWithStatusRow,
            r#"SELECT
  t.id                            AS "id!: Uuid",
  t.project_id                    AS "project_id!: Uuid",
  t.title,
  t.description,
  t.status                        AS "status!: TaskStatus",
  t.parent_workspace_id           AS "parent_workspace_id: Uuid",
  t.shared_task_id                AS "shared_task_id: Uuid",
  t.created_at                    AS "created_at!: DateTime<Utc>",
  t.updated_at                    AS "updated_at!: DateTime<Utc>",

  CASE WHEN EXISTS (
    SELECT 1
      FROM workspaces w
      JOIN sessions s ON s.workspace_id = w.id
      JOIN execution_processes ep ON ep.session_id = s.id
     WHERE w.task_id       = t.id
       AND ep.status        = 'running'
       AND ep.run_reason IN ('setupscript','cleanupscript','codingagent')
     LIMIT 1
  ) THEN 1 ELSE 0 END            AS "has_in_progress_attempt!: i64",

  CASE WHEN (
    SELECT ep.status
      FROM workspaces w
      JOIN sessions s ON s.workspace_id = w.id
      JOIN execution_processes ep ON ep.session_id = s.id
     WHERE w.task_id       = t.id
     AND ep.run_reason IN ('setupscript','cleanupscript','codingagent')
     ORDER BY ep.created_at DESC
     LIMIT 1
  ) IN ('failed','killed') THEN 1 ELSE 0 END
                                 AS "last_attempt_failed!: i64",

  ( SELECT s.executor
      FROM workspaces w
      JOIN sessions s ON s.workspace_id = w.id
      WHERE w.task_id = t.id
     ORDER BY s.created_at DESC
      LIMIT 1
    )                               AS "executor!: String"

FROM tasks t
WHERE t.project_id = ?
ORDER BY t.created_at DESC"#,
            project_id
        )
        .fetch_all(pool)
        .await?;

        let tasks = records
            .into_iter()
            .map(|rec| TaskWithAttemptStatus {
                task: Task {
                    id: rec.id,
                    project_id: rec.project_id,
                    title: rec.title,
                    description: rec.description,
                    status: rec.status,
                    parent_workspace_id: rec.parent_workspace_id,
                    shared_task_id: rec.shared_task_id,
                    created_at: rec.created_at,
                    updated_at: rec.updated_at,
                },
                has_in_progress_attempt: rec.has_in_progress_attempt != 0,
                last_attempt_failed: rec.last_attempt_failed != 0,
                executor: rec.executor,
            })
            .collect();

        Ok(tasks)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Task,
            r#"SELECT id as "id!: Uuid", project_id as "project_id!: Uuid", title, description, status as "status!: TaskStatus", parent_workspace_id as "parent_workspace_id: Uuid", shared_task_id as "shared_task_id: Uuid", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
               FROM tasks
               WHERE id = "#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_ctid(pool: &PgPool, ctid: i64) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Task,
            r#"SELECT id as "id!: Uuid", project_id as "project_id!: Uuid", title, description, status as "status!: TaskStatus", parent_workspace_id as "parent_workspace_id: Uuid", shared_task_id as "shared_task_id: Uuid", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
               FROM tasks
               WHERE ctid = "#,
            ctid
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_shared_task_id<'e, E>(
        executor: E,
        shared_task_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as!(
            Task,
            r#"SELECT id as "id!: Uuid", project_id as "project_id!: Uuid", title, description, status as "status!: TaskStatus", parent_workspace_id as "parent_workspace_id: Uuid", shared_task_id as "shared_task_id: Uuid", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
               FROM tasks
               WHERE shared_task_id = ?
               LIMIT 1"#,
            shared_task_id
        )
        .fetch_optional(executor)
        .await
    }

    pub async fn find_all_shared(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Task,
            r#"SELECT id as "id!: Uuid", project_id as "project_id!: Uuid", title, description, status as "status!: TaskStatus", parent_workspace_id as "parent_workspace_id: Uuid", shared_task_id as "shared_task_id: Uuid", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
               FROM tasks
               WHERE shared_task_id IS NOT NULL"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        data: &CreateTask,
        task_id: Uuid,
    ) -> Result<Self, sqlx::Error> {
        let status = data.status.clone().unwrap_or_default();
        sqlx::query_as!(
            Task,
            r#"INSERT INTO tasks (id, project_id, title, description, status, parent_workspace_id, shared_task_id)
               VALUES (?, ?2, ?3, ?4, ?5, ?6, ?7)
               RETURNING id as "id!: Uuid", project_id as "project_id!: Uuid", title, description, status as "status!: TaskStatus", parent_workspace_id as "parent_workspace_id: Uuid", shared_task_id as "shared_task_id: Uuid", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>""#,
            task_id,
            data.project_id,
            data.title,
            data.description,
            status,
            data.parent_workspace_id,
            data.shared_task_id
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        project_id: Uuid,
        title: String,
        description: Option<String>,
        status: TaskStatus,
        parent_workspace_id: Option<Uuid>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Task,
            r#"UPDATE tasks
               SET title = ?3, description = ?4, status = ?5, parent_workspace_id = ?6
               WHERE id = ? AND project_id = ?2
               RETURNING id as "id!: Uuid", project_id as "project_id!: Uuid", title, description, status as "status!: TaskStatus", parent_workspace_id as "parent_workspace_id: Uuid", shared_task_id as "shared_task_id: Uuid", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            project_id,
            title,
            description,
            status,
            parent_workspace_id
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: TaskStatus,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE tasks SET status = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ",
            id,
            status
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Update the parent_workspace_id field for a task
    pub async fn update_parent_workspace_id(
        pool: &PgPool,
        task_id: Uuid,
        parent_workspace_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE tasks SET parent_workspace_id = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ",
            task_id,
            parent_workspace_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Nullify parent_workspace_id for all tasks that reference the given workspace ID
    /// This breaks parent-child relationships before deleting a parent task
    pub async fn nullify_children_by_workspace_id<'e, E>(
        executor: E,
        workspace_id: Uuid,
    ) -> Result<u64, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let result: sqlx::postgres::PgQueryResult = sqlx::query!(
            "UPDATE tasks SET parent_workspace_id = NULL WHERE parent_workspace_id = ",
            workspace_id
        )
        .execute(executor)
        .await?;
        Ok(result.rows_affected())
    }

    /// Clear shared_task_id for all tasks that reference shared tasks belonging to a remote project
    /// This breaks the link between local tasks and shared tasks when a project is unlinked
    pub async fn clear_shared_task_ids_for_remote_project<'e, E>(
        executor: E,
        remote_project_id: Uuid,
    ) -> Result<u64, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let result: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"UPDATE tasks
               SET shared_task_id = NULL
               WHERE project_id IN (
                   SELECT id FROM projects WHERE remote_project_id = ?
               )"#,
            remote_project_id
        )
        .execute(executor)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete<'e, E>(executor: E, id: Uuid) -> Result<u64, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let result: sqlx::postgres::PgQueryResult = sqlx::query!("DELETE FROM tasks WHERE id = ", id)
            .execute(executor)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn set_shared_task_id<'e, E>(
        executor: E,
        id: Uuid,
        shared_task_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query!(
            "UPDATE tasks SET shared_task_id = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ",
            id,
            shared_task_id
        )
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn batch_unlink_shared_tasks<'e, E>(
        executor: E,
        shared_task_ids: &[Uuid],
    ) -> Result<u64, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        if shared_task_ids.is_empty() {
            return Ok(0);
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "UPDATE tasks SET shared_task_id = NULL, updated_at = CURRENT_TIMESTAMP WHERE shared_task_id IN (",
        );

        let mut separated = query_builder.separated(", ");
        for id in shared_task_ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        let result = query_builder.build().execute(executor).await?;
        Ok(result.rows_affected())
    }

    /// Bulk update tasks status and other fields
    pub async fn bulk_update<'e, E>(
        executor: E,
        updates: &BulkUpdateTasks,
    ) -> Result<u64, sqlx::Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        if updates.task_ids.is_empty() {
            return Ok(0);
        }

        let mut query_parts = Vec::new();
        let mut param_count = 1;

        if updates.status.is_some() {
            query_parts.push(format!("status = ${}", param_count));
            param_count += 1;
        }

        if updates.title.is_some() {
            query_parts.push(format!("title = ${}", param_count));
            param_count += 1;
        }

        if updates.description.is_some() {
            query_parts.push(format!("description = ${}", param_count));
            param_count += 1;
        }

        if query_parts.is_empty() {
            return Ok(0);
        }

        query_parts.push("updated_at = CURRENT_TIMESTAMP".to_string());

        let mut query_builder = sqlx::QueryBuilder::new(format!(
            "UPDATE tasks SET {} WHERE id IN (",
            query_parts.join(", ")
        ));

        if let Some(status) = &updates.status {
            query_builder.push_bind(status);
        }
        if let Some(title) = &updates.title {
            query_builder.push_bind(title);
        }
        if let Some(description) = &updates.description {
            query_builder.push_bind(description);
        }

        let mut separated = query_builder.separated(", ");
        for id in &updates.task_ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        let result = query_builder.build().execute(executor).await?;
        Ok(result.rows_affected())
    }

    /// Bulk delete tasks by IDs
    pub async fn bulk_delete<'e, E>(executor: E, task_ids: &[Uuid]) -> Result<u64, sqlx::Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        if task_ids.is_empty() {
            return Ok(0);
        }

        let mut query_builder = sqlx::QueryBuilder::new("DELETE FROM tasks WHERE id IN (");
        let mut separated = query_builder.separated(", ");
        for id in task_ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        let result = query_builder.build().execute(executor).await?;
        Ok(result.rows_affected())
    }

    /// Check if tasks have running execution processes
    pub async fn bulk_check_running_processes<'e, E>(
        executor: E,
        task_ids: &[Uuid],
    ) -> Result<Vec<Uuid>, sqlx::Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        if task_ids.is_empty() {
            return Ok(Vec::new());
        }

        // For now, we'll check each task individually
        // This is less efficient but simpler to implement
        let mut running_tasks = Vec::new();

        for &task_id in task_ids {
            let has_running = sqlx::query_scalar!(
                r#"SELECT EXISTS(
                    SELECT 1
                    FROM tasks t
                    JOIN workspaces w ON w.task_id = t.id
                    JOIN sessions s ON s.workspace_id = w.id
                    JOIN execution_processes ep ON ep.session_id = s.id
                    WHERE ep.status = 'running'
                    AND ep.run_reason IN ('setupscript','cleanupscript','codingagent')
                    AND t.id = $1
                ) as "exists!: i64"#,
                task_id
            )
            .fetch_one(executor)
            .await?;

            if has_running != 0 {
                running_tasks.push(task_id);
            }
        }

        Ok(running_tasks)
    }

    pub async fn find_children_by_workspace_id(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        // Find only child tasks that have this workspace as their parent
        sqlx::query_as!(
            Task,
            r#"SELECT id as "id!: Uuid", project_id as "project_id!: Uuid", title, description, status as "status!: TaskStatus", parent_workspace_id as "parent_workspace_id: Uuid", shared_task_id as "shared_task_id: Uuid", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
               FROM tasks
               WHERE parent_workspace_id = ?
               ORDER BY created_at DESC"#,
            workspace_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_relationships_for_workspace(
        pool: &PgPool,
        workspace: &Workspace,
    ) -> Result<TaskRelationships, sqlx::Error> {
        // 1. Get the current task (task that owns this workspace)
        let current_task = Self::find_by_id(pool, workspace.task_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        // 2. Get parent task (if current task was created by another workspace)
        let parent_task = if let Some(parent_workspace_id) = current_task.parent_workspace_id {
            // Find the workspace that created the current task
            if let Ok(Some(parent_workspace)) =
                Workspace::find_by_id(pool, parent_workspace_id).await
            {
                // Find the task that owns that parent workspace - THAT's the real parent
                Self::find_by_id(pool, parent_workspace.task_id).await?
            } else {
                None
            }
        } else {
            None
        };

        // 3. Get children tasks (created from this workspace)
        let children = Self::find_children_by_workspace_id(pool, workspace.id).await?;

        Ok(TaskRelationships {
            parent_task,
            current_workspace: workspace.clone(),
            children,
        })
    }
}
