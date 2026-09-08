use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{HeaderMap, header},
    response::{IntoResponse, Sse, sse::Event},
    routing::get,
};
use jiff::Timestamp;
use reqwest::StatusCode;
use tokio_stream::wrappers::BroadcastStream;
use wynnmap_types::terr::MapState;

use crate::{etag::check_etag, header_date, state::TerritoryState};

pub fn router(state: Arc<TerritoryState>) -> axum::Router {
    axum::Router::new()
        .route("/list", get(terr_list))
        .route("/state", get(map_state))
        .route("/state/sse", get(sse_handler))
        .with_state(state)
}

#[tracing::instrument(skip(state, headers))]
async fn terr_list(
    State(state): State<Arc<TerritoryState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (territories, etag, modified, expires) = {
        let lock = state.inner.read().await;
        (
            lock.territories.clone(),
            lock.territories_etag.clone(),
            lock.territories_modified,
            lock.expires,
        )
    };

    let resp_headers = [
        (
            header::CACHE_CONTROL,
            String::from("public, max-age=10, must-revalidate"),
        ),
        (
            header::AGE,
            (10 - Timestamp::now().duration_until(expires).as_secs()).to_string(),
        ),
        (header::EXPIRES, header_date(expires)),
        (header::LAST_MODIFIED, header_date(modified)),
        (header::ETAG, format!("\"{etag}\"")),
    ];

    if check_etag(&headers, &etag) {
        (StatusCode::NOT_MODIFIED, resp_headers, Body::empty()).into_response()
    } else {
        (resp_headers, Json(territories)).into_response()
    }
}

#[tracing::instrument(skip(state))]
async fn map_state(State(state): State<Arc<TerritoryState>>) -> impl IntoResponse {
    let (state, expires, timestamps) = {
        let lock = state.inner.read().await;
        (lock.state.clone(), lock.expires, lock.timestamps)
    };

    let resp_headers = [
        (
            header::CACHE_CONTROL,
            String::from("public, max-age=10, must-revalidate"),
        ),
        (
            header::AGE,
            (10 - Timestamp::now().duration_until(expires).as_secs()).to_string(),
        ),
        (header::EXPIRES, header_date(expires)),
        (
            header::LAST_MODIFIED,
            header_date(timestamps.changed.unwrap_or_default()),
        ),
    ];

    (
        resp_headers,
        Json(MapState {
            terrs: state,
            timestamps,
        }),
    )
}

async fn sse_handler(State(state): State<Arc<TerritoryState>>) -> Sse<BroadcastStream<Event>> {
    let bc_channel = &state.bc_events;

    Sse::new(BroadcastStream::new(bc_channel.subscribe()))
}
