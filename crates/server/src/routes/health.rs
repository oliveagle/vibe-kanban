use axum::response::Json;
use serde::Serialize;
use utils::response::ApiResponse;

#[derive(Serialize)]
pub struct HealthResponse {
    pub version: String,
    pub image_hash: Option<String>,
}

pub async fn health_check() -> Json<ApiResponse<HealthResponse>> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let image_hash = std::env::var("IMAGE_HASH").ok();

    Json(ApiResponse::success(HealthResponse {
        version,
        image_hash,
    }))
}
