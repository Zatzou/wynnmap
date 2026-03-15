use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::body::Bytes;
use chrono::{DateTime, Utc};
use opentelemetry::metrics::UpDownCounter;
use serde::Deserialize;
use tokio::sync::{RwLock, broadcast};
use wynnmap_types::{
    guild::Guild,
    maptile::MapTile,
    terr::{Resources, TerrOwner, Territory},
    ws::TerrSockMessage,
};

#[derive(Clone, Default)]
pub(crate) struct ImageState {
    pub use_webp: bool,

    pub maps: Arc<RwLock<Vec<MapTile>>>,
    pub map_cache: Arc<RwLock<HashMap<Arc<str>, Bytes>>>,
}

#[derive(Debug, Default)]
pub(crate) struct GuildState {
    pub guilds: Arc<RwLock<HashMap<Arc<str>, Guild>>>,
}

#[derive(Debug)]
pub(crate) struct TerritoryState {
    pub inner: Arc<RwLock<TerritoryStateInner>>,

    pub bc_recv: Arc<broadcast::Receiver<TerrSockMessage>>,
    pub active_conn: UpDownCounter<i64>,
}

#[derive(Debug, Default)]
pub(crate) struct TerritoryStateInner {
    pub territories: HashMap<Arc<str>, Territory>,
    pub owners: HashMap<Arc<str>, TerrOwner>,

    pub expires: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub(crate) struct ExTerrInfo {
    pub resources: Resources,
    #[serde(alias = "Trading Routes")]
    pub connections: HashSet<Arc<str>>,
}
