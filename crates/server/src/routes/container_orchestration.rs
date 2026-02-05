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
        .args(["ps", "-a", "--format", "json"])
        .output()
        .await
        .map_err(|e| ApiError::Io(e))?;

    if !output.status.success() {
        return Err(ApiError::BadRequest(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let containers: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .map_err(|e| ApiError::BadRequest(format!("Failed to parse podman output: {}", e)))?;

    let container_infos: Vec<ContainerInfo> = containers
        .into_iter()
        .map(|c| ContainerInfo {
            id: c["Id"].as_str().unwrap_or("").to_string(),
            name: c["Names"]
                .as_array()
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
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(ResponseJson(ApiResponse::success(
        String::from_utf8_lossy(&output.stdout).to_string(),
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
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(ResponseJson(ApiResponse::success(
        String::from_utf8_lossy(&output.stdout).to_string(),
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
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(ResponseJson(ApiResponse::success(
        String::from_utf8_lossy(&output.stdout).to_string(),
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
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(ResponseJson(ApiResponse::success(
        String::from_utf8_lossy(&output.stdout).to_string(),
    )))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunContainerRequest {
    pub image: String,
    pub name: Option<String>,
    pub ports: Option<Vec<PortMapping>>,
    pub env: Option<Vec<String>>,
    pub cmd: Option<Vec<String>>,
    pub detach: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunContainerResponse {
    pub container_id: String,
    pub output: String,
}

pub async fn run_container(
    State(_deployment): State<DeploymentImpl>,
    axum::Json(payload): axum::Json<RunContainerRequest>,
) -> Result<ResponseJson<ApiResponse<RunContainerResponse>>, ApiError> {
    let mut args: Vec<String> = vec!["run".to_string()];

    if payload.detach.unwrap_or(true) {
        args.push("-d".to_string());
    }

    if let Some(name) = &payload.name {
        args.push("--name".to_string());
        args.push(name.clone());
    }

    if let Some(ports) = &payload.ports {
        for port in ports {
            args.push("-p".to_string());
            args.push(format!("{}:{}", port.host_port, port.container_port));
        }
    }

    if let Some(env) = &payload.env {
        for e in env {
            args.push("-e".to_string());
            args.push(e.clone());
        }
    }

    args.push(payload.image.clone());

    if let Some(cmd) = &payload.cmd {
        for c in cmd {
            args.push(c.clone());
        }
    }

    let output = tokio::process::Command::new("podman")
        .args(&args)
        .output()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to run container: {}", e)))?;

    if !output.status.success() {
        return Err(ApiError::BadRequest(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(ResponseJson(ApiResponse::success(RunContainerResponse {
        container_id,
        output: String::from_utf8_lossy(&output.stdout).to_string(),
    })))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageInfo {
    pub id: String,
    pub names: Vec<String>,
    pub size: String,
    pub created: String,
}

pub async fn list_images(
    State(_deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<Vec<ImageInfo>>>, ApiError> {
    let output = tokio::process::Command::new("podman")
        .args(["images", "--format", "json"])
        .output()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to list images: {}", e)))?;

    if !output.status.success() {
        return Err(ApiError::BadRequest(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let images: Vec<ImageInfo> = serde_json::from_slice(&output.stdout).unwrap_or_default();

    Ok(ResponseJson(ApiResponse::success(images)))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PullImageRequest {
    pub image: String,
}

fn normalize_image_name(image: &str) -> String {
    // If image already has a registry prefix, use it as-is
    if image.contains('/') {
        return image.to_string();
    }
    // For short names like "nginx:alpine", prepend docker.io/library/
    if image.contains(':') {
        format!("docker.io/library/{}", image)
    } else {
        format!("docker.io/library/{}:latest", image)
    }
}

pub async fn pull_image(
    State(_deployment): State<DeploymentImpl>,
    axum::Json(payload): axum::Json<PullImageRequest>,
) -> Result<ResponseJson<ApiResponse<String>>, ApiError> {
    // Normalize image name to full path
    let full_image_name = normalize_image_name(&payload.image);

    // Get proxy settings from environment or use VK default
    let https_proxy = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .unwrap_or_else(|_| "http://host.containers.internal:1080".to_string());

    let http_proxy = std::env::var("HTTP_PROXY")
        .or_else(|_| std::env::var("http_proxy"))
        .unwrap_or_else(|_| "http://host.containers.internal:1080".to_string());

    let output = tokio::process::Command::new("podman")
        .args(["pull", &full_image_name])
        .env("HTTPS_PROXY", &https_proxy)
        .env("HTTP_PROXY", &http_proxy)
        .env("https_proxy", &https_proxy)
        .env("http_proxy", &http_proxy)
        .output()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to pull image: {}", e)))?;

    if !output.status.success() {
        return Err(ApiError::BadRequest(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(ResponseJson(ApiResponse::success(
        String::from_utf8_lossy(&output.stdout).to_string(),
    )))
}

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/orchestration/containers", get(list_containers))
        .route("/orchestration/containers/run", post(run_container))
        .route(
            "/orchestration/containers/{id}/start",
            post(start_container),
        )
        .route("/orchestration/containers/{id}/stop", post(stop_container))
        .route(
            "/orchestration/containers/{id}/restart",
            post(restart_container),
        )
        .route("/orchestration/containers/{id}", post(remove_container))
        .route("/orchestration/images", get(list_images))
        .route("/orchestration/images/pull", post(pull_image))
}
