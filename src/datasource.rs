use std::{collections::BTreeMap, sync::Arc};

use chrono::{DateTime, Utc};
use codee::string::JsonSerdeWasmCodec;
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_use::{UseEventSourceReturn, use_event_source};
use wynnmap_types::{
    api::v2::RespWrapper,
    maptile::MapTile,
    terr::{TerrState, Territory},
    ws::TerrSockMessage,
};

pub async fn load_map_tiles() -> Result<Vec<MapTile>, gloo_net::Error> {
    let r = Request::get("/api/v1/images/maps.json").send().await?;

    let tiles: Vec<MapTile> = r.json().await?;

    Ok(tiles)
}

pub async fn get_terrs() -> Result<BTreeMap<Arc<str>, Territory>, gloo_net::Error> {
    let resp: BTreeMap<Arc<str>, Territory> = Request::get("/api/v2/terr/list")
        .send()
        .await?
        .json()
        .await?;

    Ok(resp)
}

pub async fn get_state() -> Result<RespWrapper<BTreeMap<Arc<str>, TerrState>>, gloo_net::Error> {
    let resp: RespWrapper<BTreeMap<Arc<str>, TerrState>> = Request::get("/api/v2/terr/guilds")
        .send()
        .await?
        .json()
        .await?;

    Ok(resp)
}

pub fn sse_terr_updates(
    state: RwSignal<BTreeMap<Arc<str>, TerrState>>,
    last_updated: RwSignal<DateTime<Utc>>,
) {
    let UseEventSourceReturn { message, .. } =
        use_event_source::<TerrSockMessage, JsonSerdeWasmCodec>("/api/v3/terr/state/sse");

    Effect::new(move || {
        if let Some(msg) = message.get() {
            match msg.data {
                TerrSockMessage::Update(updates) => {
                    state.update(|s| {
                        for (name, data) in updates {
                            s.insert(name, data);
                        }
                    });

                    last_updated.set(Utc::now());
                }
                TerrSockMessage::LastUpdate { ts } => {
                    last_updated.set(ts);
                }
            }
        }
    });
}
