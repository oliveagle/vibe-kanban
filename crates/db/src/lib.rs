use std::{env, sync::Arc, time::Duration};

use sqlx::{
    Error, Pool, Postgres, PgPool,
    postgres::PgPoolOptions,
};

pub mod models;

const DB_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct DBService {
    pub pool: Pool<Postgres>,
}

impl DBService {
    pub async fn new() -> Result<DBService, Error> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgres://vibekanban:vibekanban123@10.126.126.5:5632/vibe_kanban".to_string()
            });
        
        let pool = PgPoolOptions::new()
            .acquire_timeout(DB_CONNECT_TIMEOUT)
            .connect(&database_url)
            .await?;
        
        sqlx::migrate!("./migrations_postgres").run(&pool).await?;
        Ok(DBService { pool })
    }

    pub async fn new_with_url(database_url: &str) -> Result<DBService, Error> {
        let pool = PgPoolOptions::new()
            .acquire_timeout(DB_CONNECT_TIMEOUT)
            .connect(database_url)
            .await?;
        
        sqlx::migrate!("./migrations_postgres").run(&pool).await?;
        Ok(DBService { pool })
    }
}
