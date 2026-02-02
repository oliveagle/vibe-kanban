//! Local authentication service for development/self-hosted mode
//! 
//! This provides a simple username/password authentication system
//! as an alternative to OAuth for local deployments.

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Duration, Utc};
use db::DBService;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};
use thiserror::Error;
use tracing;
use uuid::Uuid;

// Get credentials from environment variables with defaults
fn get_default_username() -> String {
    std::env::var("AUTH_USERNAME").unwrap_or_else(|_| "admin".to_string())
}

fn get_default_password() -> String {
    std::env::var("AUTH_PASSWORD").unwrap_or_else(|_| "admin".to_string())
}

/// Errors that can occur during local authentication
#[derive(Error, Debug)]
pub enum LocalAuthError {
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Password hashing error: {0}")]
    PasswordHash(#[from] bcrypt::BcryptError),
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

/// Claims stored in the JWT token
#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    sub: String,    // User ID
    username: String,
    exp: i64,       // Expiration timestamp
    iat: i64,       // Issued at timestamp
}

/// User data stored in database
#[derive(Debug, Clone)]
pub struct LocalUser {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

/// Response containing access token
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub user_id: String,
    pub username: String,
}

/// Local authentication service
#[derive(Clone)]
pub struct LocalAuthService {
    pool: SqlitePool,
    jwt_secret: String,
}

impl LocalAuthService {
    /// Create a new local auth service
    pub fn new(db: &DBService) -> Self {
        let pool = db.pool.clone();
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| generate_random_secret());
        
        Self { pool, jwt_secret }
    }

    /// Initialize the database table and create default user if needed
    pub async fn initialize(&self) -> Result<(), LocalAuthError> {
        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS local_users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Check if default user exists
        let default_username = get_default_username();
        let default_password = get_default_password();
        
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM local_users WHERE username = ?"
        )
        .bind(&default_username)
        .fetch_one(&self.pool)
        .await?;

        if count == 0 {
            tracing::info!("Creating default local user: {}", default_username);
            self.create_user(&default_username, &default_password).await?;
            tracing::info!("Default user created. Username: {}, Password: {}", 
                default_username, default_password);
        }

        Ok(())
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LocalUser, LocalAuthError> {
        // Check if user already exists
        let existing: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM local_users WHERE username = ?"
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await?;

        if existing.unwrap_or(0) > 0 {
            return Err(LocalAuthError::UserAlreadyExists);
        }

        // Hash password
        let password_hash = hash(password.as_bytes(), DEFAULT_COST)?;

        // Create user
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        sqlx::query(
            "INSERT INTO local_users (id, username, password_hash, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(username)
        .bind(&password_hash)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(LocalUser {
            id,
            username: username.to_string(),
            password_hash,
            created_at,
        })
    }

    /// Authenticate a user with username and password
    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, LocalAuthError> {
        // Fetch user
        let row: Option<SqliteRow> = sqlx::query(
            "SELECT id, username, password_hash, created_at FROM local_users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or(LocalAuthError::InvalidCredentials)?;

        let id: String = row.try_get("id")?;
        let username: String = row.try_get("username")?;
        let password_hash: String = row.try_get("password_hash")?;

        // Verify password
        if !verify(password.as_bytes(), &password_hash)? {
            return Err(LocalAuthError::InvalidCredentials);
        }

        // Generate JWT
        let (access_token, expires_at) = self.generate_token(&id, &username)?;

        Ok(AuthResponse {
            access_token,
            expires_at,
            user_id: id,
            username,
        })
    }

    /// Validate a JWT token and return user info
    pub fn validate_token(&self, token: &str) -> Result<(String, String), LocalAuthError> {
        let validation = Validation::default();
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )?;

        Ok((token_data.claims.sub, token_data.claims.username))
    }

    /// Generate a JWT token for a user
    fn generate_token(
        &self,
        user_id: &str,
        username: &str,
    ) -> Result<(String, DateTime<Utc>), LocalAuthError> {
        let now = Utc::now();
        let expires_at = now + Duration::days(30); // 30-day expiration

        let claims = TokenClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;

        Ok((token, expires_at))
    }

    /// Check if local auth is enabled (users table exists)
    pub async fn is_enabled(&self) -> bool {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='local_users'"
        )
        .fetch_one(&self.pool)
        .await
        .map(|count| count > 0)
        .unwrap_or(false)
    }

    /// Get default user credentials for display
    pub fn get_default_credentials() -> (String, String) {
        (get_default_username(), get_default_password())
    }
}

/// Generate a random secret for JWT signing
fn generate_random_secret() -> String {
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_password_hashing() {
        let password = "test_password123";
        let hash = bcrypt::hash(password.as_bytes(), DEFAULT_COST).unwrap();
        
        // Should verify correctly
        assert!(bcrypt::verify(password.as_bytes(), &hash).unwrap());
        
        // Wrong password should fail
        assert!(!bcrypt::verify("wrong_password".as_bytes(), &hash).unwrap());
    }
}
