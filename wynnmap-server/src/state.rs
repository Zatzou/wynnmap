use std::{collections::HashMap, sync::Arc};

use axum::body::Bytes;
use tokio::sync::RwLock;
use wynnmap_types::{ExTerrInfo, Territory, WynntilsMapTile};

use crate::config::Config;

#[derive(Clone)]
pub(crate) struct ImageState {
    pub config: Arc<Config>,

    pub maps: Arc<RwLock<Vec<WynntilsMapTile>>>,
    pub map_cache: Arc<RwLock<HashMap<Arc<str>, Bytes>>>,
}

#[derive(Clone)]
pub(crate) struct TerritoryState {
    // pub config: Arc<Config>,
    pub client: reqwest::Client,

    pub inner: Arc<RwLock<TerritoryStateInner>>,
    pub colors: Arc<RwLock<HashMap<Arc<str>, Arc<str>>>>,
    pub extra: Arc<RwLock<HashMap<Arc<str>, ExTerrInfo>>>,
}

pub(crate) struct TerritoryStateInner {
    pub territories: HashMap<Arc<str>, Territory>,
    pub expires: chrono::DateTime<chrono::Utc>,
}
