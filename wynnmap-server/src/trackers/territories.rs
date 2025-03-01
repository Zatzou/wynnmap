use std::{collections::HashMap, sync::Arc, time::Duration};

use serde::Deserialize;
use tokio::{sync::RwLock, time::Instant};
use tracing::{error, info};
use wynnmap_server::types::Territory;

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

    let state = TerritoryState {
        client,

        inner: Arc::new(RwLock::new(TerritoryStateInner {
            territories: HashMap::new(),
            colors: HashMap::new(),
            expires: chrono::Utc::now(),
        })),
    };

    tokio::spawn(territory_tracker(state.clone()));
    tokio::spawn(wynntils_color_grabber(state.clone()));

    state
}

async fn territory_tracker(state: TerritoryState) {
    loop {
        let res = territory_tracker_inner(&state).await;

        if let Err(e) = res {
            error!("Territory tracker failed: {}", e);
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }

    async fn territory_tracker_inner(state: &TerritoryState) -> Result<(), reqwest::Error> {
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
                .map(|e| chrono::DateTime::parse_from_rfc2822(e.to_str().unwrap()).unwrap())
                .unwrap()
                .to_utc();
            let date = req
                .headers()
                .get("date")
                .map(|e| chrono::DateTime::parse_from_rfc2822(e.to_str().unwrap()).unwrap())
                .unwrap()
                .naive_utc();
            let diff = expires.naive_utc().signed_duration_since(date);

            // parse the json
            let mut data: HashMap<Arc<str>, Territory> = req.json().await?;

            let mut lock = state.inner.write().await;

            // update the guild colors on the data
            for (_, terr) in &mut data {
                if let Some(col) = lock.colors.get(&terr.guild.prefix) {
                    terr.guild.color = Some(col.clone());
                }
            }

            // update the territories
            lock.territories = data;

            // update the expires so caching works
            lock.expires = expires + Duration::from_secs(1);

            // release the lock
            drop(lock);

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

        tokio::time::sleep(Duration::from_secs(30)).await;
    }

    async fn wynntils_color_grabber_inner(state: &TerritoryState) -> Result<(), reqwest::Error> {
        let terrs: WynntilsApiResponse = state
            .client
            .get("https://athena.wynntils.com/cache/get/territoryList")
            .send()
            .await?
            .json()
            .await?;

        // prepare the values so we hold the lock for as little time as possible
        let mut colors = HashMap::new();
        for (_, gcol) in terrs.territories {
            colors.insert(gcol.prefix, gcol.color);
        }

        // update the actual values
        let mut lock = state.inner.write().await;

        for (prefix, color) in colors {
            lock.colors.insert(prefix, color);
        }

        drop(lock);

        Ok(())
    }
}

#[derive(Deserialize)]
struct WynntilsApiResponse {
    territories: HashMap<Arc<str>, WynntilsTerr>,
}

#[derive(Deserialize)]
struct WynntilsTerr {
    #[serde(rename = "guildPrefix")]
    prefix: Arc<str>,
    #[serde(rename = "guildColor")]
    color: Arc<str>,
}
