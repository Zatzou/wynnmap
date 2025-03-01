use axum::{Json, extract::State, http::header, response::IntoResponse, routing::get};

use crate::state::TerritoryState;

pub(crate) fn router(state: TerritoryState) -> axum::Router {
    axum::Router::new()
        .route("/list", get(terr_list))
        // .route("/ws", get())
        .with_state(state)
}

async fn terr_list(State(state): State<TerritoryState>) -> impl IntoResponse {
    let read = state.inner.read().await;

    (
        [(header::EXPIRES, read.expires.to_rfc2822())],
        Json(read.territories.clone()),
    )
}
