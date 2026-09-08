use std::{collections::BTreeMap, sync::Arc};

use codee::string::FromToStringCodec;
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use serde::de::DeserializeOwned;
use thiserror::Error;
use wynnmap_types::{
    gather::{GatherSpots, MatData},
    maptile::MapTile,
    terr::{MapState, TerrState, TerrTimestamps, Territory},
};

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("{0}")]
    GlooNet(#[from] gloo_net::Error),
    #[error("{0} {1}")]
    BadStatus(u16, String),
}

pub async fn load_json<T: DeserializeOwned>(url: impl AsRef<str>) -> Result<T, NetworkError> {
    let res = Request::get(url.as_ref()).send().await?;

    if (200..=299).contains(&res.status().into()) {
        Ok(res.json().await?)
    } else {
        Err(NetworkError::BadStatus(res.status(), res.status_text()))
    }
}

pub async fn load_map_tiles() -> Result<Vec<MapTile>, gloo_net::Error> {
    let r = Request::get("/api/v1/images/maps.json").send().await?;

    let tiles: Vec<MapTile> = r.json().await?;

    Ok(tiles)
}

pub async fn get_terrs() -> Result<BTreeMap<Arc<str>, Territory>, gloo_net::Error> {
    let resp: BTreeMap<Arc<str>, Territory> = Request::get("/api/v3/terr/list")
        .send()
        .await?
        .json()
        .await?;

    Ok(resp)
}

pub async fn get_state() -> Result<MapState, gloo_net::Error> {
    let resp: MapState = Request::get("/api/v3/terr/state")
        .send()
        .await?
        .json()
        .await?;

    Ok(resp)
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

pub async fn get_gather_nodes() -> Result<GatherSpots, gloo_net::Error> {
    let resp: GatherSpots = Request::get("/api/v3/gather/nodes")
        .send()
        .await?
        .json()
        .await?;

    Ok(resp)
}

pub async fn get_mat_data() -> Result<BTreeMap<Arc<str>, MatData>, gloo_net::Error> {
    let resp: BTreeMap<Arc<str>, MatData> =
        Request::get("/matdata.json").send().await?.json().await?;

    Ok(resp)
}
