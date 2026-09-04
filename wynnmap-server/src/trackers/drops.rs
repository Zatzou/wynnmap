use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use serde::Deserialize;
use tracing::{Instrument, error, info_span};
use wynnmap_types::drops::{DropArea, Item, ItemDrops};

use crate::{
    config::Config,
    etag::sha224_etag_json,
    state::DropsState,
    trackers::util::{self, ResponseExt},
};

pub struct DropsTracker {
    client: reqwest::Client,

    state: Arc<DropsState>,
}

impl DropsTracker {
    pub fn with_config(config: &Config) -> Self {
        let client = util::reqwest_client_from_conf(config);

        Self {
            client,
            state: Default::default(),
        }
    }

    pub fn run(self) -> Arc<DropsState> {
        let state2 = self.state.clone();

        tokio::spawn(async move {
            let tracker = self;

            loop {
                let res = tracker.query_items().await;

                let waittime = match res {
                    Ok(_) => Duration::from_hours(1),
                    Err(e) => {
                        error!(error = ?e, "Error occured while querying drops");
                        Duration::from_mins(10)
                    }
                };

                tokio::time::sleep(waittime).await;
            }
        });

        state2
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn query_items(&self) -> Result<(), util::RequestError> {
        let data: Vec<WynnItem> = async {
            let res = self
                .client
                .get("https://api.wynncraft.com/v3/item/database?fullResult")
                .send()
                .await?;

            res.parse_json().await
        }
        .instrument(info_span!("fetch"))
        .await?;

        let processed = {
            let _ = info_span!("process").entered();

            let items: ItemDrops = data.into_iter().map(Into::into).collect();

            items
        };

        let etag = sha224_etag_json(&processed);

        async {
            let mut elock = self.state.etag.write().await;

            if *elock != etag {
                *elock = etag;

                drop(elock);

                *self.state.drops.write().await = Arc::new(processed);
            }
        }
        .instrument(info_span!("update_state"))
        .await;

        Ok(())
    }
}

#[derive(Deserialize)]
struct WynnItem {
    #[serde(rename = "displayName")]
    name: Arc<str>,
    #[serde(rename = "type")]
    kind: Arc<str>,

    #[serde(rename = "droppedBy")]
    dropped_by: Option<Vec<WynnDrop>>,
}

impl From<WynnItem> for Item {
    fn from(value: WynnItem) -> Self {
        let mut dropped_by = BTreeMap::new();

        // process drops, this also removes duplicates thanks to the BTreeSet
        for drop in value.dropped_by.unwrap_or_default() {
            let entry: &mut BTreeSet<DropArea> = dropped_by.entry(drop.name).or_default();

            entry.extend(drop.coords.into_iter().map(Into::into));
        }

        Self {
            name: value.name,
            kind: value.kind,
            dropped_by,
        }
    }
}

#[derive(Deserialize)]
struct WynnDrop {
    name: Arc<str>,
    coords: WynnCoords,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum WynnCoords {
    List(Vec<[i32; 4]>),
    Single([i32; 4]),

    /// Handle all other wynn api weirdness
    #[allow(dead_code)]
    Other(serde_json::Value),
}

impl Default for WynnCoords {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

impl IntoIterator for WynnCoords {
    type Item = [i32; 4];

    type IntoIter = std::vec::IntoIter<[i32; 4]>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            WynnCoords::List(items) => items.into_iter(),
            WynnCoords::Single(item) => vec![item].into_iter(),
            WynnCoords::Other(_) => Vec::new().into_iter(),
        }
    }
}
