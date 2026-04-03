use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};

use crate::ImageState;

pub(crate) fn router(state: Arc<ImageState>) -> axum::Router {
    axum::Router::new()
        .route("/maps.json", get(maps_json))
        .route("/{name}", get(get_image))
        .with_state(state)
}

#[tracing::instrument(skip(state))]
async fn maps_json(State(state): State<Arc<ImageState>>) -> impl IntoResponse {
    let maps = { state.maps.read().await.clone() };

    (
        [(header::CACHE_CONTROL, String::from("public, max-age=3600"))],
        Json(maps),
    )
}

#[tracing::instrument(skip(state))]
async fn get_image(
    Path(name): Path<String>,
    State(state): State<Arc<ImageState>>,
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

        if let Some(data) = data {
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime),
                    (header::ETAG, &format!("\"{name}\"")),
                    (header::CACHE_CONTROL, "public, max-age=86400"),
                ],
                data,
            )
                .into_response()
        } else {
            (StatusCode::NOT_FOUND, "Image not found").into_response()
        }
    } else {
        (StatusCode::NOT_FOUND, "Image not found").into_response()
    }
}
