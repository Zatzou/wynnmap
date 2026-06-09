use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{HeaderMap, header},
    response::IntoResponse,
    routing::get,
};
use reqwest::StatusCode;

use crate::{etag::check_etag, state::GatherState};

pub fn router(state: Arc<GatherState>) -> axum::Router {
    axum::Router::new()
        .route("/nodes", get(node_list))
        .with_state(state)
}

#[tracing::instrument(skip(state, headers))]
async fn node_list(State(state): State<Arc<GatherState>>, headers: HeaderMap) -> impl IntoResponse {
    let etag = state.etag.read().await.clone();

    let resp_headers = [
        (
            header::CACHE_CONTROL,
            String::from("public, max-age=3600, immutable, must-revalidate"),
        ),
        (header::ETAG, format!("\"{etag}\"")),
    ];

    if check_etag(&headers, &etag) {
        (StatusCode::NOT_MODIFIED, resp_headers, Body::empty()).into_response()
    } else {
        let nodes = state.nodes.read().await.clone();
        (resp_headers, Json(nodes)).into_response()
    }
}
