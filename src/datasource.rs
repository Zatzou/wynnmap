use std::{collections::BTreeMap, sync::Arc};

use codee::string::FromToStringCodec;
use gloo_net::http::Request;
use leptos::{prelude::*, task::spawn_local};
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use serde::de::DeserializeOwned;
use thiserror::Error;
use wynnmap_types::terr::{MapState, TerrState, TerrTimestamps};

use crate::dialog::{Dialogs, info::info_dialog};

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

    if (200..=299).contains(&res.status()) {
        Ok(res.json().await?)
    } else {
        Err(NetworkError::BadStatus(res.status(), res.status_text()))
    }
}

pub fn sse_terr_updates(
    state: RwSignal<BTreeMap<Arc<str>, TerrState>>,
    last_updated: RwSignal<TerrTimestamps>,
) {
    let dialogs = use_context::<Dialogs>().expect("Dialogs context not found");

    let load_owners = move || async move {
        match load_json::<MapState>("/api/v3/terr/state").await {
            Ok(data) => {
                state.set(data.terrs);
                last_updated.set(data.timestamps);
            }
            Err(err) => {
                dialogs.add_if_not_exist("err_maptiles", move || {
                    info_dialog(
                        String::from("Failed to load territory data"),
                        view! {
                            <p>"An error occured while loading api data"</p>
                            <pre class="p-2 bg-neutral-800 rounded my-1">{format!("{err:?}")}</pre>
                        },
                    )
                });
            }
        }
    };

    spawn_local(load_owners());

    let UseEventSourceReturn { message, .. } =
        use_event_source_with_options::<String, FromToStringCodec>(
            "/api/v3/terr/state/sse",
            UseEventSourceOptions::default()
                .named_events([String::from("terr"), String::from("ts")]),
        );

    Effect::new(move || {
        if let Some(event) = message.get() {
            let id = event.last_event_id.parse::<i64>().unwrap();
            let cur = last_updated.read_untracked().seq;
            let mut bad_epoch = false;

            match event.event_type.as_str() {
                "terr" => {
                    let terrdata: BTreeMap<Arc<str>, TerrState> =
                        serde_json::from_str(&event.data).unwrap();

                    state.update(|state| {
                        for (name, data) in terrdata {
                            state.insert(name, data);
                        }
                    });

                    last_updated.update(|lu| lu.seq += 1);
                }
                "ts" => {
                    let ts: TerrTimestamps = serde_json::from_str(&event.data).unwrap();

                    bad_epoch = last_updated.read_untracked().epoch != ts.epoch;

                    last_updated.set(ts);
                }
                _ => {}
            }

            if (cur != id && cur + 1 != id) || bad_epoch {
                spawn_local(load_owners());
            }
        }
    });
}
