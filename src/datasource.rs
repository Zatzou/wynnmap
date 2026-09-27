use std::{collections::BTreeMap, sync::Arc};

use codee::string::FromToStringCodec;
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use serde::de::DeserializeOwned;
use thiserror::Error;
use wynnmap_types::terr::{TerrState, TerrTimestamps};

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("{0}")]
    GlooNet(#[from] gloo_net::Error),
    #[error("{0} {1}")]
    BadStatus(u16, String),
}

pub async fn load_json<T: DeserializeOwned>(url: impl AsRef<str>) -> Result<T, NetworkError> {
    let res = Request::get(url.as_ref())
        .header("Accept", "application/json")
        .send()
        .await?;

    if (200..=299).contains(&res.status().into()) {
        Ok(res.json().await?)
    } else {
        Err(NetworkError::BadStatus(res.status(), res.status_text()))
    }
}

pub fn sse_terr_updates(
    state: RwSignal<BTreeMap<Arc<str>, TerrState>>,
    last_updated: RwSignal<TerrTimestamps>,
) {
    let UseEventSourceReturn { message, .. } =
        use_event_source_with_options::<String, FromToStringCodec>(
            "/api/v3/terr/state/sse",
            UseEventSourceOptions::default()
                .named_events([String::from("terr"), String::from("ts")]),
        );

    Effect::new(move || {
        if let Some(event) = message.get() {
            match event.event_type.as_str() {
                "terr" => {
                    let terrdata: BTreeMap<Arc<str>, TerrState> =
                        serde_json::from_str(&event.data).unwrap();

                    state.update(|state| {
                        for (name, data) in terrdata {
                            state.insert(name, data);
                        }
                    });
                }
                "ts" => {
                    let ts: TerrTimestamps = serde_json::from_str(&event.data).unwrap();

                    last_updated.set(ts);
                }
                _ => unreachable!(),
            }
        }
    });
}
