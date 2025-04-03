use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use reqwest::header;

pub(crate) async fn file_cache_control(req: Request, next: Next) -> Response<Body> {
    let res = next.run(req).await;
    let (mut parts, body) = res.into_parts();

    if let None = parts.headers.get(header::CACHE_CONTROL) {
        if let Some(ct) = parts
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
        {
            match ct {
                "text/html" | "image/png" => {
                    parts.headers.insert(
                        header::CACHE_CONTROL,
                        header::HeaderValue::from_static("public, max-age=3600"),
                    );
                }
                "text/css" | "text/javascript" | "application/wasm" | "font/ttf" => {
                    parts.headers.insert(
                        header::CACHE_CONTROL,
                        header::HeaderValue::from_static("public, max-age=604800"),
                    );
                }

                _ => {}
            }
        }
    }

    Response::from_parts(parts, body)
}
