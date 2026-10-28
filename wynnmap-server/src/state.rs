use std::{collections::BTreeMap, sync::Arc};

use axum::{body::Bytes, response::sse::Event};
use jiff::Timestamp;
use tokio::sync::{RwLock, broadcast, watch};
use wynnmap_types::{
    drops::ItemDrops,
    gather::GatherSpots,
    guild::Guild,
    leaderboard::{SeasonLeaderboard, SeasonList},
    maptile::MapTile,
    terr::{TerrState, TerrTimestamps, Territory},
};

use crate::readywatch::ReadyState;

#[derive(Clone)]
pub struct ImageState {
    pub use_webp: bool,

    pub ready: watch::Receiver<ReadyState>,

    pub maps: Arc<RwLock<Vec<MapTile>>>,
    pub maps_etag: Arc<RwLock<Arc<str>>>,
    pub map_cache: Arc<RwLock<BTreeMap<Arc<str>, (Arc<str>, Bytes)>>>,
}

#[derive(Debug)]
pub struct GuildState {
    pub guilds: Arc<RwLock<GuildList>>,
    /// Notifier channel for waiting for the initial guild update to finish
    pub ready: watch::Receiver<ReadyState>,
}

pub type GuildList = BTreeMap<Arc<str>, Guild>;

#[derive(Debug)]
pub struct TerritoryState {
    pub inner: Arc<RwLock<TerritoryStateInner>>,

    pub ready: watch::Receiver<ReadyState>,

    pub bc_events: Arc<broadcast::Sender<Event>>,
}

#[derive(Debug, Default)]
pub struct TerritoryStateInner {
    pub territories: BTreeMap<Arc<str>, Territory>,
    pub territories_etag: Arc<str>,

    pub state: BTreeMap<Arc<str>, TerrState>,

    pub expires: Timestamp,
    pub timestamps: TerrTimestamps,
}

#[derive(Debug)]
pub struct LeaderboardState {
    /// List of seasons and the etag for the list
    pub season_list: RwLock<(Arc<SeasonList>, Arc<str>)>,
    /// The leaderboards and each leaderboards etag
    pub season_lb: RwLock<BTreeMap<Arc<str>, (Arc<SeasonLeaderboard>, Arc<str>)>>,

    pub ready: watch::Receiver<ReadyState>,
}

#[derive(Debug)]
pub struct GatherState {
    pub nodes: RwLock<Arc<GatherSpots>>,
    pub etag: RwLock<Arc<str>>,

    pub ready: watch::Receiver<ReadyState>,
}

#[derive(Debug)]
pub struct DropsState {
    pub drops: RwLock<Arc<ItemDrops>>,
    pub etag: RwLock<Arc<str>>,

    pub ready: watch::Receiver<ReadyState>,
}
