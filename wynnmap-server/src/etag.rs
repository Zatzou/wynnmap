use std::sync::Arc;

use axum::http::{HeaderMap, header};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use etag::EntityTag;
use serde::Serialize;
use sha2::{Digest, Sha224};

pub fn check_etag(headers: HeaderMap, val: impl AsRef<str>) -> bool {
    let if_none_match_header = headers.get(header::IF_NONE_MATCH);

    if let Some(if_none_match) = if_none_match_header
        && let Ok(Ok(if_none_match)) = if_none_match.to_str().map(|s| s.parse::<EntityTag>())
    {
        if_none_match.strong_eq(&EntityTag::new(false, val.as_ref()))
    } else {
        false
    }
}

/// generate a sha224 hash and encode it in base64 for etag use
pub fn sha224_etag(data: impl AsRef<[u8]>) -> Arc<str> {
    let hash = Sha224::digest(data);

    BASE64_STANDARD.encode(hash.0).into()
}

/// generate a sha224 hash from the json serialized form and encode it in base64 for etag use
pub fn sha224_etag_json(data: &impl Serialize) -> Arc<str> {
    let json = serde_json::to_vec(data).unwrap();

    sha224_etag(json)
}
