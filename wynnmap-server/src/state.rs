use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::body::Bytes;
use serde::Deserialize;
use tokio::sync::{RwLock, broadcast};
use wynnmap_types::{
    guild::Guild,
    maptile::MapTile,
    terr::{Resources, TerrOwner, Territory},
    ws::TerrSockMessage,
};

use crate::config::Config;

#[derive(Clone)]
pub(crate) struct ImageState {
    pub config: Arc<Config>,

    pub maps: Arc<RwLock<Vec<MapTile>>>,
    pub map_cache: Arc<RwLock<HashMap<Arc<str>, Bytes>>>,
}

#[derive(Clone)]
pub(crate) struct GuildState {
    pub client: reqwest::Client,

    pub guilds: Arc<RwLock<HashMap<Arc<str>, Guild>>>,
}

#[derive(Clone)]
pub(crate) struct TerritoryState {
    // pub config: Arc<Config>,
    pub client: reqwest::Client,

    // The guild state from the guild tracker
    pub guilds: Arc<RwLock<HashMap<Arc<str>, Guild>>>,

    /// The final formatted output
    pub inner: Arc<RwLock<TerritoryStateInner>>,

    /// The extra data for territories which is not provided by the wynn api
    pub extra: Arc<RwLock<HashMap<Arc<str>, ExTerrInfo>>>,

    pub bc_recv: Arc<broadcast::Receiver<TerrSockMessage>>,
}

pub(crate) struct TerritoryStateInner {
    pub territories: HashMap<Arc<str>, Territory>,
    pub owners: HashMap<Arc<str>, TerrOwner>,
    pub expires: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub(crate) struct ExTerrInfo {
    pub resources: Resources,
    #[serde(alias = "Trading Routes")]
    pub connections: HashSet<Arc<str>>,
}
