use std::{collections::HashMap, mem, sync::Arc, time::Duration};

use serde::Deserialize;
use tokio::{
    sync::{RwLock, broadcast},
    time::{Instant, sleep},
};
use tracing::{error, info};
use wynnmap_types::{
    Region,
    guild::Guild,
    terr::{TerrOwner, Territory},
    ws::TerrSockMessage,
};

use crate::{
    config::Config,
    state::{ExTerrInfo, TerritoryState, TerritoryStateInner},
};

pub(crate) async fn create_terr_tracker(
    config: Arc<Config>,
    guilds: Arc<RwLock<HashMap<Arc<str>, Guild>>>,
) -> TerritoryState {
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
            owners: HashMap::new(),
            expires: chrono::Utc::now(),
        })),

        guilds: guilds,
        extra: Arc::new(RwLock::new(HashMap::new())),

        bc_recv: Arc::new(bc_recv),
    };

    tokio::spawn(territory_tracker(state.clone(), bc_send));
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
            let data: HashMap<Arc<str>, WynnTerritory> = req.json().await?;

            // get the extradata
            let exdata = state.extra.read().await;

            // create the territory data
            let territories = data
                .iter()
                .map(|(name, t)| {
                    let exdata = exdata.get(name);
                    (
                        name.clone(),
                        Territory {
                            location: t.location,
                            connections: exdata.map(|e| e.connections.clone()).unwrap_or_default(),
                            generates: exdata.map(|e| e.resources.clone()).unwrap_or_default(),
                        },
                    )
                })
                .collect::<HashMap<_, _>>();

            // drop the exdata lock
            drop(exdata);

            // read the guilds data to get guild colors
            let collock = state.guilds.read().await;

            // generate the owners data
            let owners = data
                .iter()
                .map(|(name, t)| {
                    let mut guild = t.guild.clone();

                    guild.color = collock.get(&t.guild.prefix).and_then(|g| g.color.clone());

                    (
                        name.clone(),
                        TerrOwner {
                            guild,
                            acquired: Some(t.acquired),
                        },
                    )
                })
                .collect::<HashMap<_, _>>();

            // drop the lock on the guilds data
            drop(collock);

            let mut lock = state.inner.write().await;

            // update the territories
            lock.territories = territories;

            // update the guild owner data and get the old
            let mut old_owners = owners.clone();
            mem::swap(&mut lock.owners, &mut old_owners);

            // update the expires so caching works
            lock.expires = expires + Duration::from_secs(1);

            // release the lock
            drop(lock);

            // compare the new and old data and send the changes
            for (tname, new) in owners {
                let old = old_owners.get(&tname);

                if old != Some(&new) {
                    bc_send.send(TerrSockMessage::Capture {
                        name: tname,
                        old: old.cloned(),
                        new: new,
                    })?;
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

#[derive(Deserialize)]
struct WynnTerritory {
    guild: Guild,
    acquired: chrono::DateTime<chrono::Utc>,
    location: Region,
}

async fn extra_data_loader(state: TerritoryState) {
    loop {
        let r = extra_data_loader_inner(&state).await;

        if let Err(e) = r {
            error!("Extra data loader failed: {:?}", e);
        }

        sleep(Duration::from_secs(60)).await;
    }

    async fn extra_data_loader_inner(state: &TerritoryState) -> Result<(), reqwest::Error> {
        let data: HashMap<Arc<str>, ExTerrInfo> = state
            .client
            .get("https://gist.githubusercontent.com/Zatzou/14c82f2df0eb4093dfa1d543b78a73a8/raw/d03273fce33c031498c07e21b94f17644c8aae98/terrextra.json")
            .send()
            .await?
            .json()
            .await?;

        let mut lock = state.extra.write().await;

        *lock = data;

        Ok(())
    }
}
