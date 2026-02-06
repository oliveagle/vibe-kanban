use axum::{
    Router,
    extract::{Json, State},
    routing::post,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utils::response::ApiResponse;

use crate::{DeploymentImpl, error::ApiError};

#[derive(Debug, Clone, Deserialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogResponse {
    pub received: bool,
}

pub fn router() -> Router<DeploymentImpl> {
    Router::new().route("/debug/log", post(receive_log))
}

async fn receive_log(
    State(_deployment): State<DeploymentImpl>,
    Json(payload): Json<LogEntry>,
) -> Result<axum::Json<ApiResponse<LogResponse>>, ApiError> {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");

    match payload.level.as_str() {
        "error" => {
            tracing::error!(
                "[FRONTEND] [{}] {} - Context: {:?}",
                timestamp,
                payload.message,
                payload.context
            );
        }
        "warn" => {
            tracing::warn!(
                "[FRONTEND] [{}] {} - Context: {:?}",
                timestamp,
                payload.message,
                payload.context
            );
        }
        "info" => {
            tracing::info!(
                "[FRONTEND] [{}] {} - Context: {:?}",
                timestamp,
                payload.message,
                payload.context
            );
        }
        "debug" => {
            tracing::debug!(
                "[FRONTEND] [{}] {} - Context: {:?}",
                timestamp,
                payload.message,
                payload.context
            );
        }
        _ => {
            tracing::info!(
                "[FRONTEND] [{}] {} - Context: {:?}",
                timestamp,
                payload.message,
                payload.context
            );
        }
    }

    Ok(axum::Json(ApiResponse::success(LogResponse { received: true })))
}
