use std::{collections::BTreeMap, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use codee::string::JsonSerdeWasmCodec;
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_use::{UseWebSocketReturn, core::ConnectionReadyState, use_websocket};
use wynnmap_types::{
    api::v2::RespWrapper,
    maptile::MapTile,
    terr::{TerrOwner, Territory},
    ws::TerrSockMessage,
};

use crate::error::debug_fmt_error;

pub async fn load_map_tiles() -> Result<Vec<MapTile>, String> {
    let r = Request::get("/api/v1/images/maps.json")
        .send()
        .await
        .map_err(debug_fmt_error)?;

    let tiles: Vec<MapTile> = r.json().await.map_err(debug_fmt_error)?;

    Ok(tiles)
}

pub async fn get_terrs() -> Result<BTreeMap<Arc<str>, Territory>, String> {
    let resp: BTreeMap<Arc<str>, Territory> = Request::get("/api/v2/terr/list")
        .send()
        .await
        .map_err(debug_fmt_error)?
        .json()
        .await
        .map_err(debug_fmt_error)?;

    Ok(resp)
}

pub async fn get_owners() -> Result<RespWrapper<BTreeMap<Arc<str>, TerrOwner>>, String> {
    let resp: RespWrapper<BTreeMap<Arc<str>, TerrOwner>> = Request::get("/api/v2/terr/guilds")
        .send()
        .await
        .map_err(debug_fmt_error)?
        .json()
        .await
        .map_err(debug_fmt_error)?;

    Ok(resp)
}

pub fn ws_terr_changes(
    owners: RwSignal<BTreeMap<Arc<str>, TerrOwner>>,
    last_updated: RwSignal<DateTime<Utc>>,
) {
    let UseWebSocketReturn {
        ready_state,
        message,
        open,
        ..
    } = use_websocket::<(), TerrSockMessage, JsonSerdeWasmCodec>("/api/v2/terr/guilds/ws");

    Effect::new(move || {
        if ready_state.get() == ConnectionReadyState::Closed {
            let opfn = open.clone();

            // attempt to reconnect every 10 seconds if the connection is closed
            set_timeout(
                move || {
                    if ready_state.get() == ConnectionReadyState::Closed {
                        opfn();
                    }
                },
                Duration::from_secs(10),
            );
        }
    });

    Effect::new(move || {
        if let Some(msg) = message.get() {
            match msg {
                TerrSockMessage::Capture { name, old: _, new } => {
                    owners.write().insert(name, new);
                    last_updated.set(Utc::now());
                }
                TerrSockMessage::LastUpdate { ts } => {
                    last_updated.set(ts);
                }
            }
        }
    });
}
