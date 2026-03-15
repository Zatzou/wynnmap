use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::RwLock;
use tracing::{Level, error, span};

use crate::{
    config::Config,
    state::ExTerrInfo,
    trackers::util::{self, ResponseExt},
};

pub struct TerrExtraTracker {
    client: reqwest::Client,

    state: Arc<RwLock<HashMap<Arc<str>, ExTerrInfo>>>,
}

impl TerrExtraTracker {
    pub fn with_config(config: &Config) -> Self {
        let client = util::reqwest_client_from_conf(config);

        Self {
            client,
            state: Default::default(),
        }
    }

    pub fn run(self) -> Arc<RwLock<HashMap<Arc<str>, ExTerrInfo>>> {
        let state2 = self.state.clone();

        tokio::spawn(async move {
            let tracker = self;

            loop {
                let res = tracker.query_extra().await;

                let waittime = match res {
                    Ok(_) => Duration::from_hours(1),
                    Err(e) => {
                        error!(error = ?e, "Error occured while querying extra data");
                        Duration::from_mins(10)
                    }
                };

                tokio::time::sleep(waittime).await;
            }
        });

        state2
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn query_extra(&self) -> Result<(), util::RequestError> {
        let data: HashMap<Arc<str>, ExTerrInfo> = {
            let span = span!(Level::INFO, "fetch");
            let _enter = span.enter();

            let res = self
                .client
                .get("https://gist.githubusercontent.com/Zatzou/14c82f2df0eb4093dfa1d543b78a73a8/raw/d03273fce33c031498c07e21b94f17644c8aae98/terrextra.json")
                .send()
                .await?;

            res.parse_json().await?
        };

        {
            let span = span!(Level::INFO, "update");
            let _enter = span.enter();

            *self.state.write().await = data;
        }

        Ok(())
    }
}
