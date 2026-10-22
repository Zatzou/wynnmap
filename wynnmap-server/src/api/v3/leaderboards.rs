use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::get,
};
use reqwest::{StatusCode, header};

use crate::{etag::check_etag, state::LeaderboardState};

pub fn router(state: Arc<LeaderboardState>) -> axum::Router {
    axum::Router::new()
        .route("/seasons", get(season_list))
        .route("/season/{name}", get(season_lb))
        .with_state(state)
}

#[tracing::instrument(skip(state, headers))]
async fn season_list(
    State(state): State<Arc<LeaderboardState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (sl, etag) = { state.season_list.read().await.clone() };

    let resp_headers = [
        (
            header::CACHE_CONTROL,
            String::from("public, max-age=3600, must-revalidate"),
        ),
        (header::ETAG, format!("\"{etag}\"")),
    ];

    if check_etag(&headers, &etag) {
        (StatusCode::NOT_MODIFIED, resp_headers).into_response()
    } else {
        (resp_headers, Json(sl)).into_response()
    }
}

#[tracing::instrument(skip(state, headers))]
async fn season_lb(
    Path(name): Path<Arc<str>>,
    State(state): State<Arc<LeaderboardState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let res = { state.season_lb.read().await.get(&name).cloned() };

    if let Some((lb, etag)) = res {
        let resp_headers = [
            (
                header::CACHE_CONTROL,
                String::from("public, max-age=3600, must-revalidate"),
            ),
            (header::ETAG, format!("\"{etag}\"")),
        ];

        if check_etag(&headers, &etag) {
            (StatusCode::NOT_MODIFIED, resp_headers).into_response()
        } else {
            (resp_headers, Json(lb)).into_response()
        }
    } else {
        (StatusCode::NOT_FOUND).into_response()
    }
}
