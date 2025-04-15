use std::{collections::HashMap, sync::Arc, time::Duration};

use codee::string::JsonSerdeWasmCodec;
use leptos::prelude::*;
use leptos_use::{UseWebSocketReturn, core::ConnectionReadyState, use_websocket};
use wynnmap_types::{ExTerrInfo, Territory, WynntilsMapTile, ws::TerrSockMessage};

/// Get the api url to be used for fetching data. This by default uses the current window location
/// to determine the host and whether or not encryption is used.
///
/// # Arguments
/// - `protocol` - The protocol to use for the url. This should be either `"http"` or `"ws"`.
pub fn get_url(protocol: &str) -> String {
    let window = leptos::leptos_dom::helpers::window().location();
    let host = window.host().unwrap();
    let proto = window.protocol().is_ok_and(|p| p == "https:");

    format!("{protocol}{}://{host}", if proto { "s" } else { "" })
}

pub async fn load_map_tiles() -> Result<Vec<WynntilsMapTile>, reqwest::Error> {
    let r = reqwest::get(format!("{}{}", get_url("http"), "/api/v1/images/maps.json")).await?;

    let tiles: Vec<WynntilsMapTile> = r.json().await?;

    Ok(tiles)
}

pub async fn get_terrs() -> Result<HashMap<Arc<str>, Territory>, reqwest::Error> {
    let resp: HashMap<Arc<str>, Territory> =
        reqwest::get(format!("{}{}", get_url("http"), "/api/v1/territories/list"))
            .await?
            .json()
            .await?;

    Ok(resp)
}

pub async fn get_extra_terr_info() -> Result<HashMap<Arc<str>, ExTerrInfo>, reqwest::Error> {
    let resp: HashMap<Arc<str>, ExTerrInfo> = reqwest::get(format!(
        "{}{}",
        get_url("http"),
        "/api/v1/territories/extra"
    ))
    .await?
    .json()
    .await?;

    Ok(resp)
}

pub fn ws_terr_changes(
    terrs: RwSignal<HashMap<Arc<str>, Territory>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let UseWebSocketReturn {
        ready_state,
        message,
        open,
        ..
    } = use_websocket::<TerrSockMessage, TerrSockMessage, JsonSerdeWasmCodec>(&format!(
        "{}/api/v1/territories/ws",
        get_url("ws")
    ));

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
                TerrSockMessage::Territory(hash_map) => {
                    terrs.write().extend(hash_map);
                }
                TerrSockMessage::Capture { name, old, new } => {
                    terrs.write().insert(name, new);
                }
            }
        }
    });

    Ok(())
}
