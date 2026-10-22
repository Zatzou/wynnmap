use std::{collections::HashMap, sync::Arc, time::Duration};

use serde::Deserialize;
use tokio::{sync::watch, try_join};
use tracing::{Instrument, error, info_span};
use uuid::Uuid;
use wynnmap_types::guild::Guild;

use crate::{
    config::Config,
    state::GuildState,
    trackers::util::{self, ResponseExt},
};

pub struct GuildTracker {
    client: reqwest::Client,

    done: watch::Sender<bool>,

    state: Arc<GuildState>,
}

impl GuildTracker {
    pub fn with_config(config: &Config) -> Self {
        let client = util::reqwest_client_from_conf(config);

        let (done_tx, done_rx) = watch::channel(false);

        Self {
            client,

            done: done_tx,

            state: GuildState {
                guilds: Default::default(),
                done: done_rx,
            }
            .into(),
        }
    }

    pub fn run(self) -> Arc<GuildState> {
        let state2 = self.state.clone();

        tokio::spawn(async move {
            let tracker = self;

            loop {
                let res = tracker.query_guilds().await;

                let waittime = match res {
                    Ok(_) => {
                        tracker.done.send_replace(true);
                        Duration::from_hours(1)
                    }
                    Err(e) => {
                        error!(error = ?e, "Error occured while querying guilds");
                        Duration::from_mins(10)
                    }
                };

                tokio::time::sleep(waittime).await;
            }
        });

        state2
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn query_guilds(&self) -> Result<(), util::RequestError> {
        let (wynn_guilds, wynntils_guilds): (HashMap<Arc<str>, WynnGuild>, Vec<WynntilsGuild>) =
            async {
                try_join!(
                    async {
                        let res = self
                            .client
                            .get("https://api.wynncraft.com/v3/guild/list/guild")
                            .send()
                            .await?;

                        res.parse_json().await
                    },
                    async {
                        let res = self
                            .client
                            .get("https://athena.wynntils.com/cache/get/guildList")
                            .send()
                            .await?;

                        res.parse_json().await
                    }
                )
            }
            .instrument(info_span!("fetch"))
            .await?;

        async {
            // acquire lock on the state
            let mut lock = self.state.guilds.write().await;

            // insert the guilds from wynn api
            for (name, gu) in wynn_guilds {
                let guild = Guild {
                    uuid: Some(gu.uuid),
                    name,
                    prefix: gu.prefix.clone(),
                    color: None,
                };

                lock.insert(gu.prefix, guild);
            }

            // insert colors from wynntils
            for guild in wynntils_guilds {
                if let Some(pfx) = guild.prefix
                    && let Some(gu) = lock.get_mut(&pfx)
                {
                    gu.color = guild.color;
                }
            }
        }
        .instrument(info_span!("update_state"))
        .await;

        Ok(())
    }
}

#[derive(Deserialize)]
struct WynnGuild {
    uuid: Uuid,
    prefix: Arc<str>,
}

#[derive(Deserialize)]
struct WynntilsGuild {
    prefix: Option<Arc<str>>,
    color: Option<Arc<str>>,
}
