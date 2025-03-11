use axum::http::Method;
use axum::{Router, middleware};
use state::ImageState;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{self, CorsLayer};
use tracing::info;
use trackers::{images::create_image_tracker, territories::create_terr_tracker};

mod api;
mod config;
mod etag;
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
            "/v1",
            Router::new()
                .nest("/images", api::images::router(img_state))
                .nest("/territories", api::territories::router(terr_state)),
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
