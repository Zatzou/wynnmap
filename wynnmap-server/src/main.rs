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

use crate::trackers::drops::DropsTracker;
use crate::trackers::gather::GatherSpotsTracker;
use crate::trackers::guilds::GuildTracker;
use crate::trackers::images::ImageTracker;
use crate::trackers::leaderboards::LeaderboardTracker;
use crate::trackers::territories::TerritoryTracker;

mod api;
mod config;
mod etag;
mod file_cache;
mod otel;
mod state;
mod trackers;
pub(crate) mod util;

type AnyError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    let config = config::load_config().await;
    println!("Loaded config");

    if let Some(conf) = &config.otel {
        otel::init(conf);
        info!("Initialized with otel and fmt logging");
    } else {
        tracing_subscriber::fmt().init();
        info!("Otel not configured. Using only fmt logging.");
    }

    let img_state = ImageTracker::with_config(&config).run();
    let guild_state = GuildTracker::with_config(&config).run();
    let terr_state = TerritoryTracker::with_config(&config, &guild_state).run();
    let lb_state = LeaderboardTracker::with_config(&config, &guild_state).run();
    let gather_state = GatherSpotsTracker::with_config(&config).run();
    let drops_state = DropsTracker::with_config(&config).run();

    let cors = CorsLayer::new()
        .allow_origin(cors::Any)
        .allow_methods([Method::GET]);

    let index_svc = ServeFile::new(PathBuf::from(config.server.fe_dir.as_ref()).join("index.html"));

    let app = Router::new()
        .nest_service(
            "/api",
            Router::new()
                .nest(
                    "/v1",
                    Router::new()
                        .nest("/images", api::v1::images::router(img_state))
                        .fallback(api_404),
                )
                .nest(
                    "/v3",
                    Router::new()
                        .nest("/terr", api::v3::territories::router(terr_state))
                        .nest("/gather", api::v3::gather::router(gather_state))
                        .nest("/items", api::v3::drops::router(drops_state))
                        .nest("/lb", api::v3::leaderboards::router(lb_state))
                        .fallback(api_404),
                )
                .fallback(api_404),
        )
        .route_service("/", index_svc.clone())
        .route_service("/plan", index_svc.clone())
        .route_service("/gather", index_svc.clone())
        .route_service("/drops", index_svc.clone())
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(file_cache::file_cache_control))
                .service(ServeDir::new(config.server.fe_dir.as_ref()).not_found_service(index_svc)),
        )
        .layer(
            ServiceBuilder::new()
                .layer(cors)
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
