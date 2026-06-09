use std::{collections::HashSet, sync::Arc, time::Duration};

use serde::Deserialize;
use tracing::{Instrument, error, info_span};
use wynnmap_types::gather::{GatherSpot, GatherSpots, Material};

use crate::{
    config::Config,
    etag::sha224_etag_json,
    state::GatherState,
    trackers::util::{self, ResponseExt},
};

pub struct GatherSpotsTracker {
    client: reqwest::Client,

    state: Arc<GatherState>,
}

impl GatherSpotsTracker {
    pub fn with_config(config: &Config) -> Self {
        let client = util::reqwest_client_from_conf(config);

        Self {
            client,
            state: Default::default(),
        }
    }

    pub fn run(self) -> Arc<GatherState> {
        let state2 = self.state.clone();

        tokio::spawn(async move {
            let tracker = self;

            loop {
                let res = tracker.query_gather_spots().await;

                let waittime = match res {
                    Ok(_) => Duration::from_hours(1),
                    Err(e) => {
                        error!(error = ?e, "Error occured while querying gather nodes");
                        Duration::from_mins(10)
                    }
                };

                tokio::time::sleep(waittime).await;
            }
        });

        state2
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn query_gather_spots(&self) -> Result<(), util::RequestError> {
        let data: Vec<WynnGatherSpot> = async {
            let res = self
                .client
                .get("https://api.wynncraft.com/v3/map/gathering-nodes")
                .send()
                .await?;

            res.parse_json().await
        }
        .instrument(info_span!("fetch"))
        .await?;

        let types = {
            let _ = info_span!("get_types").entered();
            let mut types: HashSet<Material> = HashSet::new();

            for spot in &data {
                types.insert(spot.res.clone().into());
            }

            let mut types = types.into_iter().collect::<Vec<_>>();
            // types.sort_by_key(|r| r.name.clone());
            types.sort_by_key(|r| (r.level, r.name.clone()));
            types
        };

        let processed = {
            let _ = info_span!("process").entered();

            let mut spots = Vec::new();

            for spot in data {
                spots.push(GatherSpot {
                    pos: [spot.x, spot.y, spot.z],
                    resource: types
                        .iter()
                        .enumerate()
                        .find(|(_, r)| r.name == spot.res.resource)
                        .map(|(n, _)| n)
                        .unwrap_or_default(),
                });
            }

            spots.sort();

            GatherSpots {
                resources: types,
                spots,
            }
        };

        let etag = sha224_etag_json(&processed);

        async {
            let mut elock = self.state.etag.write().await;

            if *elock != etag {
                *elock = etag;

                drop(elock);

                *self.state.nodes.write().await = Arc::new(processed);
            }
        }
        .instrument(info_span!("update_state"))
        .await;

        Ok(())
    }
}

#[derive(Deserialize)]
struct WynnGatherSpot {
    x: i32,
    y: i32,
    z: i32,

    #[serde(flatten)]
    res: WynnResource,
}

#[derive(Deserialize, Clone)]
struct WynnResource {
    resource: Arc<str>,
    level: i32,
}

impl From<WynnResource> for Material {
    fn from(value: WynnResource) -> Self {
        Self {
            name: value.resource,
            level: value.level,
        }
    }
}
