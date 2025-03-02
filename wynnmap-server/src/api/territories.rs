use axum::{Json, extract::State, http::header, response::IntoResponse, routing::get};

use crate::state::TerritoryState;

pub(crate) fn router(state: TerritoryState) -> axum::Router {
    axum::Router::new()
        .route("/list", get(terr_list))
        .route("/extra", get(extra_data))
        // .route("/ws", get())
        .with_state(state)
}

async fn terr_list(State(state): State<TerritoryState>) -> impl IntoResponse {
    let read = state.inner.read().await;

    let age = (read.expires - chrono::Utc::now()).num_seconds();

    (
        [
            (header::CACHE_CONTROL, String::from("public, max-age=10")),
            (header::AGE, age.to_string()),
            (header::EXPIRES, read.expires.to_rfc2822()),
        ],
        Json(read.territories.clone()),
    )
}

async fn extra_data(State(state): State<TerritoryState>) -> impl IntoResponse {
    let read = state.extra.read().await;

    (
        [(header::CACHE_CONTROL, "public, max-age=3600")],
        Json(read.clone()),
    )
}
