use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::RwLock;
use tracing::{Level, error, span};

use crate::{
    config::Config,
    state::ExTerrInfo,
    trackers::util::{self, ReqResp},
};

pub struct TerrExtraTracker {
    client: util::ReqClient,

    state: Arc<RwLock<HashMap<Arc<str>, ExTerrInfo>>>,
}

impl TerrExtraTracker {
    pub fn with_config(config: &Config) -> Self {
        let client = util::ReqClient::from_config(config);

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
        let ReqResp { data, .. }: ReqResp<HashMap<Arc<str>, ExTerrInfo>> = self
            .client
            .get("https://gist.githubusercontent.com/Zatzou/14c82f2df0eb4093dfa1d543b78a73a8/raw/d03273fce33c031498c07e21b94f17644c8aae98/terrextra.json")
            .await?;

        {
            let span = span!(Level::INFO, "update");
            let _enter = span.enter();

            *self.state.write().await = data;
        }

        Ok(())
    }
}
