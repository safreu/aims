use crate::bootstrap::BootstrapError;
use crate::modules::households::api::households_router;
use crate::modules::inventory::api::inventory_routes;
use crate::modules::scanning::api::scanning_device_routes;
use crate::{
    bootstrap::build_app_state, config::AppConfig, modules::accounts::api::accounts_router,
    shared::api::AppState,
};
use axum::{Json, Router, routing::get};
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

pub struct Application {
    listener: TcpListener,
    router: Router,
}

impl Application {
    pub async fn build(config: AppConfig) -> Result<Self, ApplicationError> {
        let state = build_app_state(&config).await?;

        let router = build_router(state);

        let listener = TcpListener::bind(config.server.address).await?;

        Ok(Self { listener, router })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let address = self.listener.local_addr()?;

        tracing::info!(%address, "aims backend started");

        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(shutdown_signal())
            .await
    }
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .nest("/api/v1/auth", accounts_router())
        .nest("/api/v1/households", households_router())
        .nest("/api/v1/inventory", inventory_routes())
        .nest("/api/v1/device", scanning_device_routes())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "im still alive",
    })
}

async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => tracing::info!("shutdown signal received"),
        Err(error) => tracing::error!(%error, "could not listen for shutdown signal"),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("application initialization failed")]
    Bootstrap(#[from] BootstrapError),

    #[error("could not bind the HTTP server")]
    IO(#[from] std::io::Error),
}
