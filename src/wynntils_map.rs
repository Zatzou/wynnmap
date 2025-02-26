use std::collections::HashMap;

use serde::Deserialize;

use crate::types::{Territory, WynntilsMapTile};

pub async fn load_map_tiles() -> Option<Vec<WynntilsMapTile>> {
    let r = reqwest::get("https://raw.githubusercontent.com/Wynntils/Static-Storage/refs/heads/main/Reference/maps.json").await.unwrap();

    let tiles: Vec<WynntilsMapTile> = r.json().await.unwrap();

    Some(tiles)
}

pub async fn get_wynntils_terrs() -> Result<HashMap<String, Territory>, reqwest::Error> {
    let resp: TerrApi = reqwest::get("https://athena.wynntils.com/cache/get/territoryList")
        .await?
        .json()
        .await?;

    Ok(resp.territories)
}

#[derive(Debug, Deserialize, Clone)]
struct TerrApi {
    territories: HashMap<String, Territory>,
}
