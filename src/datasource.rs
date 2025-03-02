use std::{collections::HashMap, sync::Arc};

use wynnmap_types::{ExTerrInfo, Territory, WynntilsMapTile};

#[cfg(debug_assertions)]
const API_URL: &str = "http://localhost:8081";
#[cfg(not(debug_assertions))]
const API_URL: &str = "https://api.wynnmap.zatzou.com";

pub async fn load_map_tiles() -> Option<Vec<WynntilsMapTile>> {
    let r = reqwest::get(format!("{}{}", API_URL, "/v1/images/maps.json"))
        .await
        .unwrap();

    let tiles: Vec<WynntilsMapTile> = r.json().await.unwrap();

    Some(tiles)
}

pub async fn get_wynntils_terrs() -> Result<HashMap<Arc<str>, Territory>, reqwest::Error> {
    let resp: HashMap<Arc<str>, Territory> =
        reqwest::get(format!("{}{}", API_URL, "/v1/territories/list"))
            .await?
            .json()
            .await?;

    Ok(resp)
}

pub async fn get_extra_terr_info() -> Result<HashMap<Arc<str>, ExTerrInfo>, reqwest::Error> {
    let resp: HashMap<Arc<str>, ExTerrInfo> =
        reqwest::get(format!("{}{}", API_URL, "/v1/territories/extra"))
            .await?
            .json()
            .await?;

    Ok(resp)
}
