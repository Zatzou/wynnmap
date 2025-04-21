use std::{collections::HashMap, mem, sync::Arc, time::Duration};

use serde::Deserialize;
use tokio::{
    sync::{RwLock, broadcast},
    time::{Instant, sleep},
};
use tracing::{error, info};
use wynnmap_types::{ExTerrInfo, Guild, TerrRes, Territory, ws::TerrSockMessage};

use crate::{
    config::Config,
    state::{TerritoryState, TerritoryStateInner},
};

pub(crate) async fn create_terr_tracker(config: Arc<Config>) -> TerritoryState {
    let client = reqwest::Client::builder()
        .user_agent(format!(
            "{}/{} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            config.client.ua_contact
        ))
        .build()
        .unwrap();

    let (bc_send, bc_recv) = broadcast::channel(500);

    let state = TerritoryState {
        client,

        inner: Arc::new(RwLock::new(TerritoryStateInner {
            territories: HashMap::new(),
            expires: chrono::Utc::now(),
        })),

        guilds: Arc::new(RwLock::new(HashMap::new())),
        extra: Arc::new(RwLock::new(HashMap::new())),

        bc_recv: Arc::new(bc_recv),
    };

    tokio::spawn(territory_tracker(state.clone(), bc_send));
    tokio::spawn(wynntils_color_grabber(state.clone()));
    tokio::spawn(extra_data_loader(state.clone()));

    state
}

async fn territory_tracker(state: TerritoryState, bc_send: broadcast::Sender<TerrSockMessage>) {
    loop {
        let res = territory_tracker_inner(&state, bc_send.clone()).await;

        if let Err(e) = res {
            error!("Territory tracker failed: {}", e);
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }

    async fn territory_tracker_inner(
        state: &TerritoryState,
        bc_send: broadcast::Sender<TerrSockMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            info!("Loading territories");
            let req = state
                .client
                .get("https://api.wynncraft.com/v3/guild/list/territory")
                .send()
                .await?;
            let reqend = Instant::now();

            // calculate timings so we can wait until new data is available
            let expires = req
                .headers()
                .get("expires")
                .and_then(|e| e.to_str().ok())
                .and_then(|e| chrono::DateTime::parse_from_rfc2822(e).ok())
                .unwrap_or_default()
                .to_utc();
            let date = req
                .headers()
                .get("date")
                .and_then(|e| e.to_str().ok())
                .and_then(|e| chrono::DateTime::parse_from_rfc2822(e).ok())
                .unwrap_or_default()
                .naive_utc();
            let diff = expires.naive_utc().signed_duration_since(date);

            // parse the json
            let mut data: HashMap<Arc<str>, Territory> = req.json().await?;

            let collock = state.guilds.read().await;

            // update the guild colors on the data
            for terr in data.values_mut() {
                if let Some(pfx) = &terr.guild.prefix {
                    if let Some(guild) = collock.get(pfx) {
                        terr.guild.color = guild.color.clone();
                    }
                }
            }

            drop(collock);

            let mut lock = state.inner.write().await;

            // update the territories and get the old data
            let mut old = data.clone();
            mem::swap(&mut lock.territories, &mut old);

            // update the expires so caching works
            lock.expires = expires + Duration::from_secs(1);

            // release the lock
            drop(lock);

            // compare the new and old data and send the changes
            for (k, new) in data.iter() {
                if let Some(old) = old.get(k) {
                    if old != new {
                        if old.guild.name != new.guild.name {
                            bc_send.send(TerrSockMessage::Capture {
                                name: k.clone(),
                                old: old.clone(),
                                new: new.clone(),
                            })?;
                        } else {
                            bc_send.send(TerrSockMessage::Territory(HashMap::from_iter(vec![
                                (k.clone(), new.clone()),
                            ])))?;
                        }
                    }
                } else {
                    bc_send.send(TerrSockMessage::Territory(HashMap::from_iter(vec![(
                        k.clone(),
                        new.clone(),
                    )])))?;
                }
            }

            // wait until new data is available
            tokio::time::sleep_until(
                reqend + diff.to_std().unwrap_or(Duration::from_secs(1)) + Duration::from_secs(1),
            )
            .await;
        }
    }
}

async fn wynntils_color_grabber(state: TerritoryState) {
    loop {
        let res = wynntils_color_grabber_inner(&state).await;

        if let Err(e) = res {
            error!("Wynntils color grabber failed: {}", e);
        }

        tokio::time::sleep(Duration::from_secs(60 * 60)).await;
    }

    async fn wynntils_color_grabber_inner(state: &TerritoryState) -> Result<(), reqwest::Error> {
        let guilds: Vec<Guild> = state
            .client
            .get("https://athena.wynntils.com/cache/get/guildList")
            .send()
            .await?
            .json()
            .await?;

        // convert the Vec<Guild> to a HashMap<Arc<str>, Guild>
        let guilds = guilds
            .into_iter()
            .filter(|g| g.prefix.is_some())
            .map(|g| (g.prefix.clone().unwrap(), g))
            .collect::<HashMap<Arc<str>, Guild>>();

        // lock
        let mut lock = state.guilds.write().await;

        // update the guilds
        *lock = guilds;
        drop(lock);

        Ok(())
    }
}

async fn extra_data_loader(state: TerritoryState) {
    loop {
        let r = extra_data_loader_inner(&state).await;

        if let Err(e) = r {
            error!("Extra data loader failed: {}", e);
        }

        sleep(Duration::from_secs(60)).await;
    }

    async fn extra_data_loader_inner(state: &TerritoryState) -> Result<(), reqwest::Error> {
        let data: HashMap<Arc<str>, ExTerrInfoOrig> = state
            .client
            .get("https://raw.githubusercontent.com/jakematt123/Wynncraft-Territory-Info/refs/heads/main/territories.json")
            .send()
            .await?
            .json()
            .await?;

        let data = HashMap::from_iter(data.into_iter().map(|(k, v)| (k, v.into())));

        let mut lock = state.extra.write().await;

        *lock = data;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct ExTerrInfoOrig {
    pub resources: TerrResOrig,

    #[serde(rename = "Trading Routes")]
    pub conns: Option<Vec<Arc<str>>>,
}

impl From<ExTerrInfoOrig> for ExTerrInfo {
    fn from(orig: ExTerrInfoOrig) -> Self {
        ExTerrInfo {
            resources: orig.resources.into(),
            conns: orig.conns.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TerrResOrig {
    pub emeralds: Arc<str>,
    pub ore: Arc<str>,
    pub crops: Arc<str>,
    pub fish: Arc<str>,
    pub wood: Arc<str>,
}

impl From<TerrResOrig> for TerrRes {
    fn from(orig: TerrResOrig) -> Self {
        TerrRes {
            emeralds: orig.emeralds.parse().unwrap_or(0),
            ore: orig.ore.parse().unwrap_or(0),
            crops: orig.crops.parse().unwrap_or(0),
            fish: orig.fish.parse().unwrap_or(0),
            wood: orig.wood.parse().unwrap_or(0),
        }
    }
}
