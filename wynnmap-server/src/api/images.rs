use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};

use crate::ImageState;

pub(crate) fn router(state: ImageState) -> axum::Router {
    axum::Router::new()
        .route("/maps.json", get(maps_json))
        .route("/{name}", get(get_image))
        .with_state(state)
}

async fn maps_json(State(state): State<ImageState>) -> impl IntoResponse {
    Json(state.maps.read().await.clone())
}

async fn get_image(Path(name): Path<String>, State(state): State<ImageState>) -> impl IntoResponse {
    let (name, ext) = name.split_once('.').unwrap_or((name.as_str(), ""));

    let (wanted_ext, mime) = if state.config.images.use_webp {
        ("webp", "image/webp")
    } else {
        ("png", "image/png")
    };

    if ext != wanted_ext {
        (StatusCode::NOT_FOUND, "Image not found".as_bytes()).into_response()
    } else {
        let map_cache = state.map_cache.read().await;
        if let Some(data) = map_cache.get(name) {
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime),
                    (header::ETAG, name),
                    (header::CACHE_CONTROL, "public, max-age=31536000"),
                ],
                data.clone(),
            )
                .into_response()
        } else {
            (StatusCode::NOT_FOUND, "Image not found").into_response()
        }
    }
}
