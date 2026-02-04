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
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    Ok(ResponseJson(ApiResponse::success(RunContainerResponse {
        container_id,
        output: String::from_utf8_lossy(&output.stdout).to_string(),
    })))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestNginxResponse {
    pub container_id: String,
    pub container_name: String,
    pub host_port: String,
    pub nginx_response: String,
    pub test_status: String,
}

pub async fn test_nginx(
    State(_deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<TestNginxResponse>>, ApiError> {
    use std::time::Duration;
    use tokio::time::sleep;
    
    let container_name = format!("vibe-kanban-test-nginx-{}", uuid::Uuid::new_v4());
    let host_port = "8888";
    
    // Step 1: Run nginx container
    let run_output = tokio::process::Command::new("podman")
        .args([
            "run", "-d",
            "--name", &container_name,
            "-p", &format!("{}:80", host_port),
            "nginx:alpine"
        ])
        .output()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to run nginx container: {}", e)))?;

    if !run_output.status.success() {
        return Err(ApiError::BadRequest(
            format!("Failed to create nginx container: {}", String::from_utf8_lossy(&run_output.stderr))
        ));
    }

    let container_id = String::from_utf8_lossy(&run_output.stdout).trim().to_string();
    
    // Step 2: Wait for nginx to be ready
    sleep(Duration::from_secs(2)).await;
    
    // Step 3: Make HTTP request to nginx
    let client = reqwest::Client::new();
    let nginx_url = format!("http://localhost:{}", host_port);
    
    let test_result = match client.get(&nginx_url)
        .timeout(Duration::from_secs(10))
        .send()
        .await {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let preview = if body.len() > 200 {
                format!("{}...", &body[..200])
            } else {
                body
            };
            (
                format!("Status: {}, Body preview: {}", status, preview),
                "success".to_string()
            )
        }
        Err(e) => (
            format!("Request failed: {}", e),
            "failed".to_string()
        )
    };
    
    // Step 4: Cleanup - stop and remove container
    let _ = tokio::process::Command::new("podman")
        .args(["stop", "-t", "1", &container_id])
        .output()
        .await;
    
    let _ = tokio::process::Command::new("podman")
        .args(["rm", &container_id])
        .output()
        .await;
    
    Ok(ResponseJson(ApiResponse::success(TestNginxResponse {
        container_id,
        container_name,
        host_port: host_port.to_string(),
        nginx_response: test_result.0,
        test_status: test_result.1,
    })))
}

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/orchestration/containers", get(list_containers))
        .route("/orchestration/containers/run", post(run_container))
        .route("/orchestration/containers/test-nginx", post(test_nginx))
        .route("/orchestration/containers/{id}/start", post(start_container))
        .route("/orchestration/containers/{id}/stop", post(stop_container))
        .route("/orchestration/containers/{id}/restart", post(restart_container))
        .route("/orchestration/containers/{id}", post(remove_container))
}
