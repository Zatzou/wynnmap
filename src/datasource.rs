use std::{
    collections::{BTreeMap, btree_map::Entry},
    sync::Arc,
    time::Duration,
};

use codee::{Decoder, Encoder};
use gloo_net::http::Request;
use leptos::{
    logging::{error, warn},
    prelude::*,
};
use leptos_use::{
    UseWebSocketOptions, UseWebSocketReturn, core::ConnectionReadyState, use_websocket_with_options,
};
use wynnmap_types::{
    maptile::MapTile,
    terr::{MapState, TerrState, TerrTimestamps, Territory},
    ws::TerrSockMessage,
};

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

pub fn ws_terr_updates(
    state: RwSignal<BTreeMap<Arc<str>, TerrState>>,
    last_updated: RwSignal<TerrTimestamps>,
) {
    let UseWebSocketReturn {
        ready_state,
        message,
        open,
        ..
    } = use_websocket_with_options::<(), TerrSockMessage, WynnmapCodec, _, _>(
        "/api/v3/terr/state/ws",
        UseWebSocketOptions::default().on_error(|e| error!("Websocket error:\n{e:?}")),
    );

    Effect::new(move || {
        if ready_state.get() == ConnectionReadyState::Closed {
            let opfn = open.clone();
            warn!("Websocket closed. Reconnecting in 10s");

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
                TerrSockMessage::Update(updates, timestamps) => {
                    state.update(|s| {
                        for (name, data) in updates {
                            let e = s.entry(name.clone());

                            match e {
                                Entry::Vacant(vacant_entry) => {
                                    warn!("Insering default data for territory {name}");
                                    vacant_entry.insert(TerrState::default()).apply_diff(data);
                                }
                                Entry::Occupied(mut occupied_entry) => {
                                    occupied_entry.get_mut().apply_diff(data);
                                }
                            }
                        }
                    });

                    last_updated.set(timestamps);
                }
                TerrSockMessage::LastUpdate(timestamps) => {
                    last_updated.set(timestamps);
                }
            }
        }
    });
}

struct WynnmapCodec;

impl<T: serde::Serialize> Encoder<T> for WynnmapCodec {
    type Error = ();
    type Encoded = Vec<u8>;

    fn encode(_: &T) -> Result<Self::Encoded, Self::Error> {
        panic!("Serialization is not used")
    }
}

impl<T: serde::de::DeserializeOwned> Decoder<T> for WynnmapCodec {
    type Error = rmp_serde::decode::Error;
    type Encoded = [u8];

    fn decode(val: &Self::Encoded) -> Result<T, Self::Error> {
        wynnmap_types::encoding::decode_data(val)
    }
}
