use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum UserCredentialsError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Credentials not found")]
    NotFound,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct UserCredentials {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: Option<String>,
    pub refresh_token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UserCredentialsData {
    pub access_token: Option<String>,
    pub refresh_token: String,
    pub expires_at: Option<DateTime<Utc>>,
}

pub struct UserCredentialsRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> UserCredentialsRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserCredentials>, UserCredentialsError> {
        let result = sqlx::query_as!(
            UserCredentials,
            r#"
            SELECT 
                id AS "id!: Uuid",
                user_id AS "user_id!: Uuid",
                access_token,
                refresh_token,
                expires_at AS "expires_at?: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM user_credentials
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(result)
    }

    pub async fn upsert(
        &self,
        user_id: Uuid,
        data: &UserCredentialsData,
    ) -> Result<UserCredentials, UserCredentialsError> {
        let result = sqlx::query_as!(
            UserCredentials,
            r#"
            INSERT INTO user_credentials (user_id, access_token, refresh_token, expires_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id) DO UPDATE SET
                access_token = EXCLUDED.access_token,
                refresh_token = EXCLUDED.refresh_token,
                expires_at = EXCLUDED.expires_at,
                updated_at = NOW()
            RETURNING 
                id AS "id!: Uuid",
                user_id AS "user_id!: Uuid",
                access_token,
                refresh_token,
                expires_at AS "expires_at?: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            "#,
            user_id,
            data.access_token,
            data.refresh_token,
            data.expires_at
        )
        .fetch_one(self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_by_user_id(&self, user_id: Uuid) -> Result<(), UserCredentialsError> {
        sqlx::query!(
            r#"
            DELETE FROM user_credentials
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn exists(&self, user_id: Uuid) -> Result<bool, UserCredentialsError> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_credentials WHERE user_id = $1
            ) as "exists!"
            "#,
            user_id
        )
        .fetch_one(self.pool)
        .await?;

        Ok(result.exists)
    }
}

impl UserCredentials {
    pub fn expires_soon(
        &self,
        leeway: chrono::Duration,
    ) -> bool {
        match (self.access_token.as_ref(), self.expires_at.as_ref()) {
            (Some(_), Some(exp)) => Utc::now() + leeway >= *exp,
            _ => true,
        }
    }
}


