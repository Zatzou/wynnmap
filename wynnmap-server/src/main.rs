use std::path::PathBuf;

use axum::http::Method;
use axum::response::IntoResponse;
use axum::{Json, Router, middleware};
use reqwest::StatusCode;
use state::ImageState;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{self, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;
use trackers::{images::create_image_tracker, territories::create_terr_tracker};

mod api;
mod config;
mod etag;
mod file_cache;
mod state;
mod trackers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = config::load_config().await;
    info!("Loaded config",);

    let img_state = create_image_tracker(config.clone()).await;
    let terr_state = create_terr_tracker(config.clone()).await;

    let cors = CorsLayer::new()
        .allow_origin(cors::Any)
        .allow_methods([Method::GET]);

    let app = Router::new()
        .nest(
            "/api",
            Router::new().nest(
                "/v1",
                Router::new()
                    .nest("/images", api::images::router(img_state))
                    .nest("/territories", api::territories::router(terr_state))
                    .fallback(api_404),
            ),
        )
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(file_cache::file_cache_control))
                .service(
                    ServeDir::new(config.server.fe_dir.as_ref()).fallback(ServeFile::new(
                        PathBuf::from(config.server.fe_dir.as_ref()).join("index.html"),
                    )),
                ),
        )
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(middleware::from_fn(etag::etag_middleware))
                .layer(CompressionLayer::new()),
        );

    let listener = TcpListener::bind(&format!("{}:{}", config.server.bind, config.server.port))
        .await
        .unwrap();

    info!("Listning on {}:{}", config.server.bind, config.server.port);
    axum::serve(listener, app).await.unwrap();
}

async fn api_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json("Not Found"))
}
