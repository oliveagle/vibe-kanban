use std::{str::FromStr, sync::Arc};

use db::{
    DBService,
    models::{
        execution_process::ExecutionProcess, project::Project, scratch::Scratch, task::Task,
        workspace::Workspace,
    },
};
use serde_json::Value;
use sqlx::{Error as SqlxError, Postgres, PgPool};
use tokio::sync::RwLock;
use utils::msg_store::MsgStore;
use uuid::Uuid;

#[path = "events/patches.rs"]
pub mod patches;
#[path = "events/streams.rs"]
mod streams;
#[path = "events/types.rs"]
pub mod types;

pub use patches::{
    execution_process_patch, project_patch, scratch_patch, task_patch, workspace_patch,
};
pub use types::{EventError, EventPatch, EventPatchInner, HookTables, RecordTypes};

#[derive(Clone)]
pub struct EventService {
    msg_store: Arc<MsgStore>,
    db: DBService,
    #[allow(dead_code)]
    entry_count: Arc<RwLock<usize>>,
}

impl EventService {
    /// Creates a new EventService that will work with a DBService configured with hooks
    pub fn new(db: DBService, msg_store: Arc<MsgStore>, entry_count: Arc<RwLock<usize>>) -> Self {
        Self {
            msg_store,
            db,
            entry_count,
        }
    }

    async fn push_task_update_for_task(
        pool: &PgPool,
        msg_store: Arc<MsgStore>,
        task_id: Uuid,
    ) -> Result<(), SqlxError> {
        if let Some(task) = Task::find_by_id(pool, task_id).await? {
            let tasks = Task::find_by_project_id_with_attempt_status(pool, task.project_id).await?;

            if let Some(task_with_status) = tasks
                .into_iter()
                .find(|task_with_status| task_with_status.id == task_id)
            {
                msg_store.push_patch(task_patch::replace(&task_with_status)).await;
            }
        }

        Ok(())
    }

    async fn push_task_update_for_session(
        pool: &PgPool,
        msg_store: Arc<MsgStore>,
        session_id: Uuid,
    ) -> Result<(), SqlxError> {
        use db::models::session::Session;
        if let Some(session) = Session::find_by_id(pool, session_id).await?
            && let Some(workspace) = Workspace::find_by_id(pool, session.workspace_id).await?
        {
            Self::push_task_update_for_task(pool, msg_store, workspace.task_id).await?;
        }

        Ok(())
    }

    async fn handle_notification(
        pool: &PgPool,
        msg_store: Arc<MsgStore>,
        payload: &str,
    ) -> Result<(), SqlxError> {
        let data: Value = serde_json::from_str(payload).unwrap_or_default();
        
        if let (Some(table), Some(op), Some(id_str)) = (
            data.get("table").and_then(|v| v.as_str()),
            data.get("operation").and_then(|v| v.as_str()),
            data.get("id").and_then(|v| v.as_str()),
        ) {
            if let Ok(id) = Uuid::parse_str(id_str) {
                match table {
                    "tasks" => {
                        Self::push_task_update_for_task(pool, msg_store, id).await?;
                    }
                    "projects" => {
                        if let Some(project) = Project::find_by_id(pool, id).await? {
                            msg_store.push_patch(project_patch::replace(&project)).await;
                        }
                    }
                    "workspaces" => {
                        if let Some(workspace) = Workspace::find_by_id(pool, id).await? {
                            msg_store.push_patch(workspace_patch::replace(&workspace)).await;
                        }
                    }
                    "execution_processes" => {
                        if let Some(process) = ExecutionProcess::find_by_id(pool, id).await? {
                            msg_store.push_patch(execution_process_patch::replace(&process)).await;
                        }
                    }
                    "sessions" => {
                        Self::push_task_update_for_session(pool, msg_store, id).await?;
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }

    /// Stub for PostgreSQL - SQLite hooks not available
    /// For PostgreSQL, use LISTEN/NOTIFY instead
    #[allow(dead_code)]
    pub fn create_hook(
        _msg_store: Arc<MsgStore>,
        _entry_count: Arc<RwLock<usize>>,
        _db_service: DBService,
    ) -> impl Fn(
        &mut sqlx::PgConnection,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + '_>,
    > + Send
    + Sync
    + 'static {
        move |_conn: &mut sqlx::PgConnection| {
            Box::pin(async move {
                Ok(())
            })
        }
    }

    /// Start the event service with PostgreSQL LISTEN/NOTIFY
    pub async fn start(&self) -> Result<(), EventError> {
        let pool = self.db.pool.clone();
        let msg_store = self.msg_store.clone();
        
        tokio::spawn(async move {
            loop {
                match Self::listen_for_changes(&pool, msg_store.clone()).await {
                    Ok(_) => {
                        tracing::info!("PostgreSQL LISTEN/NOTIFY connection closed, reconnecting...");
                    }
                    Err(e) => {
                        tracing::error!("PostgreSQL LISTEN/NOTIFY error: {}, reconnecting in 5s...", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    async fn listen_for_changes(
        pool: &PgPool,
        msg_store: Arc<MsgStore>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use sqlx::postgres::PgListener;
        
        let mut listener = PgListener::connect_with(pool).await?;
        listener.listen("table_changes").await?;
        
        tracing::info!("PostgreSQL LISTEN/NOTIFY started, listening for table_changes");
        
        loop {
            let notification = listener.recv().await?;
            let payload = notification.payload();
            
            if let Err(e) = Self::handle_notification(pool, msg_store.clone(), payload).await {
                tracing::error!("Failed to handle notification: {}", e);
            }
        }
    }

    /// Get the message store
    pub fn msg_store(&self) -> &Arc<MsgStore> {
        &self.msg_store
    }
}
