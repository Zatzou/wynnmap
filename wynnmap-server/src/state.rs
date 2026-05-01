use std::{
    collections::{BTreeMap, BTreeSet},
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
pub struct ImageState {
    pub use_webp: bool,

    pub maps: Arc<RwLock<Vec<MapTile>>>,
    pub maps_etag: Arc<RwLock<Arc<str>>>,
    pub map_cache: Arc<RwLock<BTreeMap<Arc<str>, (Arc<str>, Bytes)>>>,
}

#[derive(Debug, Default)]
pub struct GuildState {
    pub guilds: Arc<RwLock<BTreeMap<Arc<str>, Guild>>>,
}

#[derive(Debug)]
pub struct TerritoryState {
    pub inner: Arc<RwLock<TerritoryStateInner>>,

    pub bc_recv: Arc<broadcast::Receiver<TerrSockMessage>>,
    pub active_conn: UpDownCounter<i64>,
}

#[derive(Debug, Default)]
pub struct TerritoryStateInner {
    pub territories: BTreeMap<Arc<str>, Territory>,
    pub territories_etag: Arc<str>,
    pub territories_modified: DateTime<Utc>,
    pub owners: BTreeMap<Arc<str>, TerrOwner>,

    pub expires: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ExTerrInfo {
    pub resources: Resources,
    #[serde(alias = "Trading Routes")]
    pub connections: BTreeSet<Arc<str>>,
}
