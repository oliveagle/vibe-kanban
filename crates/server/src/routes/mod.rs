use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{IntoMakeService, get},
};
use tower_http::cors::{Any, CorsLayer};

use crate::DeploymentImpl;
use crate::middleware::auth_middleware;

pub mod approvals;
pub mod config;
pub mod container_orchestration;
pub mod containers;
pub mod debug;
pub mod filesystem;
// pub mod github;
pub mod events;
pub mod execution_processes;
pub mod frontend;
pub mod health;
pub mod images;
pub mod oauth;
pub mod organizations;
pub mod projects;
pub mod repo;
pub mod scratch;
pub mod sessions;
pub mod shared_tasks;
pub mod tags;
pub mod task_attempts;
pub mod tasks;

pub fn router(deployment: DeploymentImpl) -> IntoMakeService<Router> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // OAuth routes - no auth required
    let oauth_routes = oauth::router().with_state(deployment.clone());

    // Protected routes - auth required
    let protected_routes = Router::new()
        .route("/health", get(health::health_check))
        .merge(config::router())
        .merge(containers::router(&deployment))
        .merge(container_orchestration::router(&deployment))
        .merge(projects::router(&deployment))
        .merge(tasks::router(&deployment))
        .merge(shared_tasks::router())
        .merge(task_attempts::router(&deployment))
        .merge(execution_processes::router(&deployment))
        .merge(tags::router(&deployment))
        .merge(organizations::router())
        .merge(filesystem::router())
        .merge(repo::router())
        .merge(events::router(&deployment))
        .merge(approvals::router())
        .merge(scratch::router(&deployment))
        .merge(sessions::router(&deployment))
        .merge(debug::router())
        .nest("/images", images::routes())
        .with_state(deployment.clone())
        .layer(from_fn_with_state(
            deployment.clone(),
            crate::middleware::auth_middleware,
        ));

    let api_routes = Router::new()
        .merge(oauth_routes)
        .merge(protected_routes);

    Router::new()
        .route("/", get(frontend::serve_frontend_root))
        .nest("/api", api_routes)
        .route("/assets/{*path}", get(frontend::serve_frontend_assets))
        .fallback(get(frontend::serve_frontend_fallback))
        .layer(cors)
        .into_make_service()
}
