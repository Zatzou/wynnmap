use axum::Router;
use state::ImageState;
use tokio::net::TcpListener;
use tracing::info;
use trackers::{images::create_image_tracker, territories::create_terr_tracker};

mod api;
mod config;
mod state;
mod trackers;
mod types;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = config::load_config().await;
    info!("Loaded config",);

    let img_state = create_image_tracker(config.clone()).await;
    let terr_state = create_terr_tracker(config.clone()).await;

    let app = Router::new().nest(
        "/v1",
        Router::new()
            .nest("/images", api::images::router(img_state))
            .nest("/territories", api::territories::router(terr_state)),
    );

    let listener = TcpListener::bind(&format!("{}:{}", config.server.bind, config.server.port))
        .await
        .unwrap();

    info!("Listning on {}:{}", config.server.bind, config.server.port);
    axum::serve(listener, app).await.unwrap();
}
