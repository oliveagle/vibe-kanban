use std::path::PathBuf;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: Option<String>,
    pub refresh_token: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Credentials {
    pub fn expires_soon(&self, leeway: ChronoDuration) -> bool {
        match (self.access_token.as_ref(), self.expires_at.as_ref()) {
            (Some(_), Some(exp)) => Utc::now() + leeway >= *exp,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredCredentials {
    refresh_token: String,
}

impl From<StoredCredentials> for Credentials {
    fn from(value: StoredCredentials) -> Self {
        Self {
            access_token: None,
            refresh_token: value.refresh_token,
            expires_at: None,
        }
    }
}

pub struct OAuthCredentials {
    backend: Backend,
    inner: RwLock<Option<Credentials>>,
}

impl OAuthCredentials {
    pub fn new(path: PathBuf) -> Self {
        Self {
            backend: Backend::detect(path),
            inner: RwLock::new(None),
        }
    }

    pub fn with_pool(pool: sqlx::PgPool, user_id: uuid::Uuid) -> Self {
        Self {
            backend: Backend::Database(DatabaseBackend { pool, user_id }),
            inner: RwLock::new(None),
        }
    }

    pub async fn load(&self) -> std::io::Result<()> {
        let creds = self.backend.load().await?.map(Credentials::from);
        *self.inner.write().await = creds;
        Ok(())
    }

    pub async fn load_for_user(&self, username: &str) -> std::io::Result<Option<Credentials>> {
        let creds = self.backend.load_for_user(username).await?.map(Credentials::from);
        *self.inner.write().await = creds.clone();
        Ok(creds)
    }

    pub async fn save(&self, creds: &Credentials) -> std::io::Result<()> {
        let stored = StoredCredentials {
            refresh_token: creds.refresh_token.clone(),
        };
        self.backend.save(&stored).await?;
        *self.inner.write().await = Some(creds.clone());
        Ok(())
    }

    pub async fn save_for_user(&self, username: &str, creds: &Credentials) -> std::io::Result<()> {
        let stored = StoredCredentials {
            refresh_token: creds.refresh_token.clone(),
        };
        self.backend.save_for_user(username, &stored).await?;
        *self.inner.write().await = Some(creds.clone());
        Ok(())
    }

    pub async fn clear(&self) -> std::io::Result<()> {
        self.backend.clear().await?;
        *self.inner.write().await = None;
        Ok(())
    }

    pub async fn get(&self) -> Option<Credentials> {
        self.inner.read().await.clone()
    }

    pub async fn get_for_user(&self, username: &str) -> std::io::Result<Option<Credentials>> {
        self.backend.load_for_user(username).await.map(|opt| opt.map(Credentials::from))
    }
}

trait StoreBackend {
    async fn load(&self) -> std::io::Result<Option<StoredCredentials>>;
    async fn load_for_user(&self, username: &str) -> std::io::Result<Option<StoredCredentials>> {
        self.load().await
    }
    async fn save(&self, creds: &StoredCredentials) -> std::io::Result<()>;
    async fn save_for_user(&self, _username: &str, creds: &StoredCredentials) -> std::io::Result<()> {
        self.save(creds).await
    }
    async fn clear(&self) -> std::io::Result<()>;
}

enum Backend {
    File(FileBackend),
    Database(DatabaseBackend),
    #[cfg(target_os = "macos")]
    Keychain(KeychainBackend),
}

impl Backend {
    fn detect(path: PathBuf) -> Self {
        if std::env::var("OAUTH_CREDENTIALS_DATABASE")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            tracing::info!("OAuth credentials backend: database (deferred)");
            return Backend::File(FileBackend { base_path: path });
        }

        #[cfg(target_os = "macos")]
        {
            let use_file = match std::env::var("OAUTH_CREDENTIALS_BACKEND") {
                Ok(v) if v.eq_ignore_ascii_case("file") => true,
                Ok(v) if v.eq_ignore_ascii_case("keychain") => false,
                _ => cfg!(debug_assertions),
            };
            if use_file {
                tracing::info!("OAuth credentials backend: file");
            Backend::File(FileBackend { base_path: path })
            } else {
                tracing::info!("OAuth credentials backend: keychain");
                Backend::Keychain(KeychainBackend)
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            tracing::info!("OAuth credentials backend: file");
            Backend::File(FileBackend { base_path: path })
        }
    }
}

impl StoreBackend for Backend {
    async fn load(&self) -> std::io::Result<Option<StoredCredentials>> {
        match self {
            Backend::File(b) => b.load().await,
            Backend::Database(b) => b.load().await,
            #[cfg(target_os = "macos")]
            Backend::Keychain(b) => b.load().await,
        }
    }

    async fn load_for_user(&self, username: &str) -> std::io::Result<Option<StoredCredentials>> {
        match self {
            Backend::File(b) => b.load_for_user(username).await,
            Backend::Database(b) => b.load_for_user(username).await,
            #[cfg(target_os = "macos")]
            Backend::Keychain(b) => b.load_for_user(username).await,
        }
    }

    async fn save(&self, creds: &StoredCredentials) -> std::io::Result<()> {
        match self {
            Backend::File(b) => b.save(creds).await,
            Backend::Database(b) => b.save(creds).await,
            #[cfg(target_os = "macos")]
            Backend::Keychain(b) => b.save(creds).await,
        }
    }

    async fn save_for_user(&self, username: &str, creds: &StoredCredentials) -> std::io::Result<()> {
        match self {
            Backend::File(b) => b.save_for_user(username, creds).await,
            Backend::Database(b) => b.save_for_user(username, creds).await,
            #[cfg(target_os = "macos")]
            Backend::Keychain(b) => b.save_for_user(username, creds).await,
        }
    }

    async fn clear(&self) -> std::io::Result<()> {
        match self {
            Backend::File(b) => b.clear().await,
            Backend::Database(b) => b.clear().await,
            #[cfg(target_os = "macos")]
            Backend::Keychain(b) => b.clear().await,
        }
    }
}

struct FileBackend {
    base_path: PathBuf,
}

impl FileBackend {
    fn path_for_user(&self, username: &str) -> PathBuf {
        self.base_path.with_file_name(format!("credentials_{}.json", username))
    }

    async fn load(&self) -> std::io::Result<Option<StoredCredentials>> {
        if !self.base_path.exists() {
            return Ok(None);
        }

        let bytes = std::fs::read(&self.base_path)?;
        match Self::parse_credentials(&bytes) {
            Ok(creds) => Ok(Some(creds)),
            Err(e) => {
                tracing::warn!(?e, "failed to parse credentials file, renaming to .bad");
                let bad = self.base_path.with_extension("bad");
                let _ = std::fs::rename(&self.base_path, bad);
                Ok(None)
            }
        }
    }

    async fn load_for_user(&self, username: &str) -> std::io::Result<Option<StoredCredentials>> {
        let path = self.path_for_user(username);
        if !path.exists() {
            return Ok(None);
        }

        let bytes = std::fs::read(&path)?;
        match Self::parse_credentials(&bytes) {
            Ok(creds) => Ok(Some(creds)),
            Err(e) => {
                tracing::warn!(?e, "failed to parse credentials file, renaming to .bad");
                let bad = path.with_extension("bad");
                let _ = std::fs::rename(&path, bad);
                Ok(None)
            }
        }
    }

    fn parse_credentials(bytes: &[u8]) -> Result<StoredCredentials, serde_json::Error> {
        serde_json::from_slice::<StoredCredentials>(bytes)
    }

    async fn save(&self, creds: &StoredCredentials) -> std::io::Result<()> {
        let tmp = self.base_path.with_extension("tmp");

        let file = {
            let mut opts = std::fs::OpenOptions::new();
            opts.create(true).truncate(true).write(true);

            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                opts.mode(0o600);
            }

            opts.open(&tmp)?
        };

        serde_json::to_writer_pretty(&file, creds)?;
        file.sync_all()?;
        drop(file);

        std::fs::rename(&tmp, &self.base_path)?;
        Ok(())
    }

    async fn save_for_user(&self, username: &str, creds: &StoredCredentials) -> std::io::Result<()> {
        let path = self.path_for_user(username);
        let tmp = path.with_extension("tmp");

        let file = {
            let mut opts = std::fs::OpenOptions::new();
            opts.create(true).truncate(true).write(true);

            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                opts.mode(0o600);
            }

            opts.open(&tmp)?
        };

        serde_json::to_writer_pretty(&file, creds)?;
        file.sync_all()?;
        drop(file);

        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    async fn clear(&self) -> std::io::Result<()> {
        let _ = std::fs::remove_file(&self.base_path);
        Ok(())
    }
}

struct DatabaseBackend {
    pool: sqlx::PgPool,
    user_id: uuid::Uuid,
}

impl DatabaseBackend {
    async fn load(&self) -> std::io::Result<Option<StoredCredentials>> {
        let result = sqlx::query!(
            r#"
            SELECT refresh_token
            FROM user_credentials
            WHERE user_id = $1
            "#,
            self.user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| std::io::Error::other(format!("Database error: {}", e)))?;

        Ok(result.map(|row| StoredCredentials {
            refresh_token: row.refresh_token,
        }))
    }

    async fn save(&self, creds: &StoredCredentials) -> std::io::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_credentials (user_id, refresh_token, expires_at)
            VALUES ($1, $2, NULL)
            ON CONFLICT (user_id) DO UPDATE SET
                refresh_token = EXCLUDED.refresh_token,
                updated_at = NOW()
            "#,
            self.user_id,
            creds.refresh_token
        )
        .execute(&self.pool)
        .await
        .map_err(|e| std::io::Error::other(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn clear(&self) -> std::io::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM user_credentials
            WHERE user_id = $1
            "#,
            self.user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| std::io::Error::other(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn load_for_user(&self, username: &str) -> std::io::Result<Option<StoredCredentials>> {
        let result: Option<_> = sqlx::query!(
            r#"
            SELECT refresh_token
            FROM user_credentials
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| std::io::Error::other(format!("Database error: {}", e)))?;

        Ok(result.map(|row| StoredCredentials {
            refresh_token: row.refresh_token,
        }))
    }

    async fn save_for_user(&self, username: &str, creds: &StoredCredentials) -> std::io::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_credentials (username, refresh_token, expires_at)
            VALUES ($1, $2, NULL)
            ON CONFLICT (username) DO UPDATE SET
                refresh_token = EXCLUDED.refresh_token,
                updated_at = NOW()
            "#,
            username,
            creds.refresh_token
        )
        .execute(&self.pool)
        .await
        .map_err(|e| std::io::Error::other(format!("Database error: {}", e)))?;

        Ok(())
    }
}

#[cfg(target_os = "macos")]
struct KeychainBackend;

#[cfg(target_os = "macos")]
impl KeychainBackend {
    const SERVICE_NAME: &'static str = concat!(env!("CARGO_PKG_NAME"), ":oauth");
    const ACCOUNT_NAME: &'static str = "default";
    const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;

    async fn load(&self) -> std::io::Result<Option<StoredCredentials>> {
        use security_framework::passwords::get_generic_password;

        match get_generic_password(Self::SERVICE_NAME, Self::ACCOUNT_NAME) {
            Ok(bytes) => match serde_json::from_slice::<StoredCredentials>(&bytes) {
                Ok(creds) => Ok(Some(creds)),
                Err(error) => {
                    tracing::warn!(
                        ?error,
                        "failed to parse keychain credentials; ignoring entry and requiring re-login"
                    );
                    Ok(None)
                }
            },
            Err(e) if e.code() == Self::ERR_SEC_ITEM_NOT_FOUND => Ok(None),
            Err(e) => Err(std::io::Error::other(e)),
        }
    }

    async fn save(&self, creds: &StoredCredentials) -> std::io::Result<()> {
        use security_framework::passwords::set_generic_password;

        let bytes = serde_json::to_vec_pretty(creds).map_err(std::io::Error::other)?;
        set_generic_password(Self::SERVICE_NAME, Self::ACCOUNT_NAME, &bytes)
            .map_err(std::io::Error::other)
    }

    async fn clear(&self) -> std::io::Result<()> {
        use security_framework::passwords::delete_generic_password;

        match delete_generic_password(Self::SERVICE_NAME, Self::ACCOUNT_NAME) {
            Ok(()) => Ok(()),
            Err(e) if e.code() == Self::ERR_SEC_ITEM_NOT_FOUND => Ok(()),
            Err(e) => Err(std::io::Error::other(e)),
        }
    }
}
