use axum::{
    Router,
    extract::{Path, State},
    response::Json as ResponseJson,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utils::response::ApiResponse;

use crate::{DeploymentImpl, error::ApiError};

#[derive(Debug, Deserialize, Serialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub ports: Vec<PortMapping>,
    pub created: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PortMapping {
    pub host_port: String,
    pub container_port: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContainerActionRequest {
    pub action: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExternalProject {
    pub name: String,
    pub path: String,
    pub compose_file: Option<String>,
}

pub async fn list_containers(
    State(_deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<Vec<ContainerInfo>>>, ApiError> {
    let output = tokio::process::Command::new("podman")
        .args([
            "ps",
            "-a",
            "--format",
            "json",
        ])
        .output()
        .await
        .map_err(|e| ApiError::Io(e))?;

    if !output.status.success() {
        return Err(ApiError::BadRequest(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    let containers: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .map_err(|e| ApiError::BadRequest(format!("Failed to parse podman output: {}", e)))?;

    let container_infos: Vec<ContainerInfo> = containers
        .into_iter()
        .map(|c| ContainerInfo {
            id: c["Id"].as_str().unwrap_or("").to_string(),
            name: c["Names"].as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            image: c["Image"].as_str().unwrap_or("").to_string(),
            status: c["Status"].as_str().unwrap_or("").to_string(),
            state: c["State"].as_str().unwrap_or("").to_string(),
            ports: extract_ports(&c),
            created: c["Created"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    Ok(ResponseJson(ApiResponse::success(container_infos)))
}

fn extract_ports(container: &serde_json::Value) -> Vec<PortMapping> {
    container["Ports"]
        .as_array()
        .map(|ports| {
            ports
                .iter()
                .filter_map(|p| {
                    let host_port = p["host_port"].as_str()?.to_string();
                    let container_port = p["container_port"].as_str()?.to_string();
                    Some(PortMapping {
                        host_port,
                        container_port,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub async fn start_container(
    State(_deployment): State<DeploymentImpl>,
    Path(container_id): Path<String>,
) -> Result<ResponseJson<ApiResponse<String>>, ApiError> {
    let output = tokio::process::Command::new("podman")
        .args(["start", &container_id])
        .output()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to start container: {}", e)))?;

    if !output.status.success() {
        return Err(ApiError::BadRequest(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    Ok(ResponseJson(ApiResponse::success(
        String::from_utf8_lossy(&output.stdout).to_string()
    )))
}

pub async fn stop_container(
    State(_deployment): State<DeploymentImpl>,
    Path(container_id): Path<String>,
) -> Result<ResponseJson<ApiResponse<String>>, ApiError> {
    let output = tokio::process::Command::new("podman")
        .args(["stop", &container_id])
        .output()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to stop container: {}", e)))?;

    if !output.status.success() {
        return Err(ApiError::BadRequest(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    Ok(ResponseJson(ApiResponse::success(
        String::from_utf8_lossy(&output.stdout).to_string()
    )))
}

pub async fn restart_container(
    State(_deployment): State<DeploymentImpl>,
    Path(container_id): Path<String>,
) -> Result<ResponseJson<ApiResponse<String>>, ApiError> {
    let output = tokio::process::Command::new("podman")
        .args(["restart", &container_id])
        .output()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to restart container: {}", e)))?;

    if !output.status.success() {
        return Err(ApiError::BadRequest(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    Ok(ResponseJson(ApiResponse::success(
        String::from_utf8_lossy(&output.stdout).to_string()
    )))
}

pub async fn remove_container(
    State(_deployment): State<DeploymentImpl>,
    Path(container_id): Path<String>,
) -> Result<ResponseJson<ApiResponse<String>>, ApiError> {
    let output = tokio::process::Command::new("podman")
        .args(["rm", &container_id])
        .output()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to remove container: {}", e)))?;

    if !output.status.success() {
        return Err(ApiError::BadRequest(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    Ok(ResponseJson(ApiResponse::success(
        String::from_utf8_lossy(&output.stdout).to_string()
    )))
}

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/orchestration/containers", get(list_containers))
        .route("/orchestration/containers/:id/start", post(start_container))
        .route("/orchestration/containers/:id/stop", post(stop_container))
        .route("/orchestration/containers/:id/restart", post(restart_container))
        .route("/orchestration/containers/:id", post(remove_container))
}
