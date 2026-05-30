use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, header},
    response::{IntoResponse, Sse, sse::Event},
    routing::get,
};
use opentelemetry::global;
use reqwest::StatusCode;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tower_http::metrics::{InFlightRequestsLayer, in_flight_requests::InFlightRequestsCounter};
use wynnmap_types::api::v2::RespWrapper;

use crate::{etag::check_etag, header_date, state::TerritoryState};

pub fn router(state: Arc<TerritoryState>) -> axum::Router {
    let counter = InFlightRequestsCounter::new();
    let meter = global::meter("wynnmap-server");

    {
        let counter = counter.clone();
        meter
            .i64_observable_gauge("active-sse-sessions")
            .with_description("Active SSE connections")
            .with_callback(move |observer| {
                observer.observe(counter.get() as i64, &[]);
            })
            .build();
    }

    axum::Router::new()
        .route("/list", get(terr_list))
        .route("/state", get(guild_list))
        .route(
            "/state/sse",
            get(sse_handler).route_layer(InFlightRequestsLayer::new(counter)),
        )
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
            String::from("public, max-age=10, immutable, must-revalidate"),
        ),
        (header::ETAG, format!("\"{etag}\"")),
        (header::EXPIRES, header_date(expires)),
        (header::LAST_MODIFIED, header_date(modified)),
    ];

    if check_etag(&headers, &etag) {
        (StatusCode::NOT_MODIFIED, resp_headers, Body::empty()).into_response()
    } else {
        (resp_headers, Json(territories)).into_response()
    }
}

#[tracing::instrument(skip(state))]
async fn guild_list(State(state): State<Arc<TerritoryState>>) -> impl IntoResponse {
    let (owners, expires, updated) = {
        let lock = state.inner.read().await;
        (lock.state.clone(), lock.expires, lock.last_updated)
    };

    let resp_headers = [
        (
            header::CACHE_CONTROL,
            String::from("public, max-age=10, immutable, must-revalidate"),
        ),
        (header::EXPIRES, header_date(expires)),
        (header::LAST_MODIFIED, header_date(updated)),
    ];

    (
        resp_headers,
        Json(RespWrapper {
            data: owners,
            updated,
        }),
    )
}

async fn sse_handler(State(state): State<Arc<TerritoryState>>) -> impl IntoResponse {
    let stream = BroadcastStream::new(state.bc_bytes.resubscribe())
        .map(|data| data.map(|data| Event::default().data(data)));

    let mut res = Sse::new(stream).into_response();

    // tell proxies to not buffer data
    res.headers_mut().insert(
        HeaderName::from_static("x-accel-buffering"),
        HeaderValue::from_static("no"),
    );

    res
}
