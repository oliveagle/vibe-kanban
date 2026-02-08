use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

/// Session data stored in JSONB
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionData {
    pub status: Option<String>,
    pub executor: Option<String>,
    #[ts(type = "Date | null")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl Default for SessionData {
    fn default() -> Self {
        Self {
            status: None,
            executor: None,
            completed_at: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Session not found")]
    NotFound,
    #[error("Workspace not found")]
    WorkspaceNotFound,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Session {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub data: sqlx::types::Json<SessionData>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateSession {
    pub executor: Option<String>,
}

impl Session {
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, SessionError> {
        sqlx::query_as!(
            Session,
            r#"SELECT id AS "id!: Uuid",
                       workspace_id AS "workspace_id!: Uuid",
                       data AS "data!: sqlx::types::Json<SessionData>",
                       created_at AS "created_at!: DateTime<Utc>",
                       updated_at AS "updated_at!: DateTime<Utc>"
                FROM sessions
                WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(SessionError::Database)
    }

    pub async fn find_by_workspace_id(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<Self>, SessionError> {
        sqlx::query_as!(
            Session,
            r#"SELECT id AS "id!: Uuid",
                       workspace_id AS "workspace_id!: Uuid",
                       data AS "data!: sqlx::types::Json<SessionData>",
                       created_at AS "created_at!: DateTime<Utc>",
                       updated_at AS "updated_at!: DateTime<Utc>"
                FROM sessions
                WHERE workspace_id = $1
                ORDER BY created_at DESC"#,
            workspace_id
        )
        .fetch_all(pool)
        .await
        .map_err(SessionError::Database)
    }

    /// Find the latest session for a workspace
    pub async fn find_latest_by_workspace_id(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Option<Self>, SessionError> {
        sqlx::query_as!(
            Session,
            r#"SELECT id AS "id!: Uuid",
                       workspace_id AS "workspace_id!: Uuid",
                       data AS "data!: sqlx::types::Json<SessionData>",
                       created_at AS "created_at!: DateTime<Utc>",
                       updated_at AS "updated_at!: DateTime<Utc>"
                FROM sessions
                WHERE workspace_id = $1
                ORDER BY created_at DESC
                LIMIT 1"#,
            workspace_id
        )
        .fetch_optional(pool)
        .await
        .map_err(SessionError::Database)
    }

    pub async fn create(
        pool: &PgPool,
        data: &CreateSession,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Self, SessionError> {
        let session_data = SessionData {
            executor: data.executor.clone(),
            ..Default::default()
        };
        
        Ok(sqlx::query_as!(
            Session,
            r#"INSERT INTO sessions (id, workspace_id, data)
                VALUES ($1, $2, $3)
                RETURNING id AS "id!: Uuid",
                          workspace_id AS "workspace_id!: Uuid",
                          data AS "data!: sqlx::types::Json<SessionData>",
                          created_at AS "created_at!: DateTime<Utc>",
                          updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            workspace_id,
            session_data
        )
        .fetch_one(pool)
        .await
        .map_err(SessionError::Database)?)
    }
}
