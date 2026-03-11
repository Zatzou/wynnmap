use std::{collections::HashMap, sync::Arc, time::Duration};

use serde::Deserialize;
use tracing::{Level, error, span};
use uuid::Uuid;
use wynnmap_types::guild::Guild;

use crate::{config::Config, state::GuildState};

pub struct GuildTracker {
    client: reqwest::Client,

    state: Arc<GuildState>,
}

impl GuildTracker {
    pub fn with_config(config: &Config) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!(
                "{}/{} ({})",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                config.client.ua_contact
            ))
            .build()
            .unwrap();

        Self {
            client,
            state: Default::default(),
        }
    }

    pub fn run(self) -> Arc<GuildState> {
        let state2 = self.state.clone();

        tokio::spawn(async move {
            let tracker = self;

            loop {
                let res = tracker.query_guilds().await;

                let waittime = match res {
                    Ok(_) => Duration::from_hours(1),
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
    async fn query_guilds(&self) -> Result<(), reqwest::Error> {
        let wynn_guilds = self.query_wynn_guilds().await?;
        let wynntils_guilds = self.query_wynntils_guilds().await?;

        {
            let span = span!(Level::INFO, "update_state");
            let _enter = span.enter();

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

        Ok(())
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn query_wynn_guilds(&self) -> Result<HashMap<Arc<str>, WynnGuild>, reqwest::Error> {
        let guilds: HashMap<Arc<str>, WynnGuild> = self
            .client
            .get("https://api.wynncraft.com/v3/guild/list/guild")
            .send()
            .await?
            .json()
            .await?;

        Ok(guilds)
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn query_wynntils_guilds(&self) -> Result<Vec<WynntilsGuild>, reqwest::Error> {
        let guilds: Vec<WynntilsGuild> = self
            .client
            .get("https://athena.wynntils.com/cache/get/guildList")
            .send()
            .await?
            .json()
            .await?;

        Ok(guilds)
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
