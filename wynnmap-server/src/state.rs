use std::{collections::BTreeMap, sync::Arc};

use axum::body::Bytes;
use chrono::{DateTime, Utc};
use opentelemetry::metrics::UpDownCounter;
use tokio::sync::{RwLock, broadcast};
use wynnmap_types::{
    gather::GatherSpots,
    guild::Guild,
    maptile::MapTile,
    terr::{TerrState, TerrTimestamps, Territory},
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

    /// A broadcast receiver for encoded territory updates
    pub bc_bytes: Arc<broadcast::Receiver<Arc<Vec<u8>>>>,
    pub ws_conns: UpDownCounter<i64>,
}

#[derive(Debug, Default)]
pub struct TerritoryStateInner {
    pub territories: BTreeMap<Arc<str>, Territory>,
    pub territories_etag: Arc<str>,
    pub territories_modified: DateTime<Utc>,

    pub state: BTreeMap<Arc<str>, TerrState>,

    pub expires: DateTime<Utc>,
    pub timestamps: TerrTimestamps,
}

#[derive(Debug, Default)]
pub struct GatherState {
    pub nodes: RwLock<Arc<GatherSpots>>,
    pub etag: RwLock<Arc<str>>,
}
