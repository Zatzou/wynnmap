use std::fmt::Debug;

use axum::http::HeaderValue;
use chrono::{DateTime, Utc};
use reqwest::header::AsHeaderName;
use serde::de::DeserializeOwned;
use tracing::error;

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn reqwest_client_from_conf(config: &Config) -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(format!(
            "{}/{} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            config.client.ua_contact
        ))
        .build()
        .unwrap()
}

pub fn parse_header_datetime(header: Option<&HeaderValue>) -> Option<DateTime<Utc>> {
    header
        .map(HeaderValue::to_str)
        .and_then(Result::ok)
        .map(DateTime::parse_from_rfc2822)
        .and_then(Result::ok)
        .map(|d| d.to_utc())
}

pub trait ResponseExt {
    async fn parse_json<T: DeserializeOwned>(self) -> Result<T, RequestError>;

    fn get_header(&self, key: impl AsHeaderName) -> Option<&str>;

    fn expires(&self) -> Option<DateTime<Utc>>;
}

impl ResponseExt for reqwest::Response {
    #[tracing::instrument(skip(self), err(Debug))]
    async fn parse_json<T: DeserializeOwned>(self) -> Result<T, RequestError> {
        let body = self.bytes().await?;

        let data: Result<T, serde_json::Error> = serde_json::from_slice(&body);

        match data {
            Ok(data) => Ok(data),
            Err(e) => {
                let s = String::from_utf8_lossy(&body).into_owned();

                error!(body = s, "Failed deserialization");

                Err(e.into())
            }
        }
    }

    fn get_header(&self, key: impl AsHeaderName) -> Option<&str> {
        self.headers()
            .get(key)
            .map(HeaderValue::to_str)
            .and_then(Result::ok)
    }

    fn expires(&self) -> Option<DateTime<Utc>> {
        parse_header_datetime(self.headers().get("expires"))
    }
}
