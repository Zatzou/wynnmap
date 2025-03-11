use axum::body::Body;
use axum::http::header;
use axum::{extract::Request, middleware::Next, response::Response};
use etag::EntityTag;
use reqwest::StatusCode;

pub(crate) async fn etag_middleware(req: Request, next: Next) -> Response<Body> {
    let if_none_match_header = req.headers().get(header::IF_NONE_MATCH).cloned();
    let res = next.run(req).await;
    let (mut parts, body) = res.into_parts();

    if let Some(if_none_match) = if_none_match_header {
        if let Some(etag) = parts.headers.get(header::ETAG) {
            if let (Ok(if_none_match), Ok(etag)) = (
                if_none_match
                    .to_str()
                    .unwrap_or_default()
                    .parse::<EntityTag>(),
                etag.to_str().unwrap_or_default().parse(),
            ) {
                if if_none_match.weak_eq(&etag) {
                    parts.status = StatusCode::NOT_MODIFIED;
                    parts.headers.remove(header::CONTENT_LENGTH);
                    parts.headers.remove(header::CONTENT_TYPE);
                    parts.headers.remove(header::CONTENT_ENCODING);
                    parts.headers.remove(header::TRANSFER_ENCODING);
                    parts.headers.remove(header::LAST_MODIFIED);
                    parts.headers.remove(header::ETAG);

                    return Response::from_parts(parts, Body::empty());
                }
            }
        }
    }

    Response::from_parts(parts, body)
}
