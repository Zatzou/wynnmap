use std::{collections::HashMap, sync::Arc};

use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::error;
use uuid::Uuid;
use wynnmap_types::guild::Guild;

use crate::{config::Config, state::GuildState};

/// Set up the guild tracker
pub(crate) async fn create_guild_tracker(config: Arc<Config>) -> GuildState {
    let client = reqwest::Client::builder()
        .user_agent(format!(
            "{}/{} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            config.client.ua_contact
        ))
        .build()
        .unwrap();

    let state = GuildState {
        client: client,

        guilds: Arc::new(RwLock::new(HashMap::new())),
    };

    tokio::spawn(guild_tracker(state.clone()));

    state
}

async fn guild_tracker(state: GuildState) {
    loop {
        let res = guild_tracker_inner(&state).await;

        if let Err(e) = res {
            error!("Guild tracker failed: {}", e);
        }

        tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;
    }

    async fn guild_tracker_inner(
        state: &GuildState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // query the wynn api for a general list of guilds as well as the guild uuids
        let wynn_guilds: HashMap<Arc<str>, WynnGuild> = state
            .client
            .get("https://api.wynncraft.com/v3/guild/list/guild")
            .send()
            .await?
            .json()
            .await?;

        // query wynntils for the guild colors of guilds that have one set
        let wynntils_guilds: Vec<WynntilsGuild> = state
            .client
            .get("https://athena.wynntils.com/cache/get/guildList")
            .send()
            .await?
            .json()
            .await?;

        // lock
        let mut lock = state.guilds.write().await;

        // insert the guilds from wynn's api
        for (name, gu) in wynn_guilds {
            let guild = Guild {
                uuid: Some(gu.uuid),
                name,
                prefix: gu.prefix.clone(),
                color: None,
            };

            lock.insert(gu.prefix, guild);
        }

        // get the color info from wynntils also
        for guild in wynntils_guilds {
            if let Some(pfx) = guild.prefix {
                if let Some(gu) = lock.get_mut(&pfx) {
                    gu.color = guild.color
                }
            }
        }

        // lock is dropped at the end of the scope

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
