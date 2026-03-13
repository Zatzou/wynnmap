use std::fmt::Debug;

use axum::http::HeaderValue;
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use tracing::{error, info};

use crate::config::Config;

pub struct ReqClient {
    client: reqwest::Client,
}

pub struct ReqResp<T> {
    pub data: T,
    pub expires: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl ReqClient {
    pub fn from_config(config: &Config) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!(
                "{}/{} ({})",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                config.client.ua_contact
            ))
            .build()
            .unwrap();

        Self { client }
    }

    #[tracing::instrument(skip(self), err(Debug))]
    pub async fn get<T: DeserializeOwned, U: AsRef<str> + Debug>(
        &self,
        url: U,
    ) -> Result<ReqResp<T>, RequestError> {
        let req = self.client.get(url.as_ref()).send().await?;

        info!(status = ?req.status());

        let expires = req
            .headers()
            .get("expires")
            .map(HeaderValue::to_str)
            .and_then(Result::ok)
            .map(DateTime::parse_from_rfc2822)
            .and_then(Result::ok)
            .map(|d| d.to_utc());

        let body = req.bytes().await?;

        let data: Result<T, serde_json::Error> = serde_json::from_slice(&body);

        match data {
            Ok(data) => Ok(ReqResp { data, expires }),
            Err(e) => {
                let s = String::from_utf8_lossy(&body).into_owned();

                error!(body = s, "Failed deserialization");

                Err(e.into())
            }
        }
    }
}
