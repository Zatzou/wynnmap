use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::get,
};

use crate::{ImageState, etag::check_etag};

pub(crate) fn router(state: Arc<ImageState>) -> axum::Router {
    axum::Router::new()
        .route("/maps.json", get(maps_json))
        .route("/{name}", get(get_image))
        .with_state(state)
}

#[tracing::instrument(skip(state, headers))]
async fn maps_json(State(state): State<Arc<ImageState>>, headers: HeaderMap) -> impl IntoResponse {
    let etag = { state.maps_etag.read().await.clone() };

    let resp_headers = [
        (header::CACHE_CONTROL, String::from("public, max-age=60")),
        (header::ETAG, etag.to_string()),
    ];

    if check_etag(headers, etag) {
        (StatusCode::NOT_MODIFIED, resp_headers, Body::empty()).into_response()
    } else {
        let maps = { state.maps.read().await.clone() };

        (resp_headers, Json(maps)).into_response()
    }
}

#[tracing::instrument(skip(state, headers))]
async fn get_image(
    Path(name): Path<String>,
    State(state): State<Arc<ImageState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (name, ext) = name.split_once('.').unwrap_or((name.as_str(), ""));

    let (wanted_ext, mime) = if state.use_webp {
        ("webp", "image/webp")
    } else {
        ("png", "image/png")
    };

    if ext == wanted_ext {
        let data = {
            let map_cache = state.map_cache.read().await;
            map_cache.get(name).cloned()
        };

        if let Some((etag, data)) = data {
            let resp_headers = [
                (header::CONTENT_TYPE, mime),
                (header::ETAG, &format!("\"{name}\"")),
                (header::CACHE_CONTROL, "public, max-age=86400"),
            ];

            if check_etag(headers, etag) {
                (StatusCode::NOT_MODIFIED, resp_headers, Body::empty()).into_response()
            } else {
                (StatusCode::OK, resp_headers, data).into_response()
            }
        } else {
            (StatusCode::NOT_FOUND, "Image not found").into_response()
        }
    } else {
        (StatusCode::NOT_FOUND, "Image not found").into_response()
    }
}
