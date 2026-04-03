use std::{collections::BTreeMap, mem, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use opentelemetry::global;
use serde::Deserialize;
use tokio::{
    sync::{RwLock, broadcast},
    time::Instant,
};
use tracing::{Instrument, error, info_span};
use uuid::Uuid;
use wynnmap_types::{
    Region,
    guild::Guild,
    terr::{TerrOwner, Territory},
    ws::TerrSockMessage,
};

use crate::{
    AnyError,
    config::Config,
    state::{ExTerrInfo, GuildState, TerritoryState},
    trackers::util::{self, ResponseExt},
};

pub struct TerritoryTracker {
    client: reqwest::Client,
    guilds: Arc<RwLock<BTreeMap<Arc<str>, Guild>>>,
    extra: Arc<RwLock<BTreeMap<Arc<str>, ExTerrInfo>>>,

    bc_send: broadcast::Sender<TerrSockMessage>,

    state: Arc<TerritoryState>,
}

impl TerritoryTracker {
    pub fn with_config(
        config: &Config,
        guild_state: &GuildState,
        extra_data: Arc<RwLock<BTreeMap<Arc<str>, ExTerrInfo>>>,
    ) -> Self {
        let client = util::reqwest_client_from_conf(config);

        let (bc_send, bc_recv) = broadcast::channel(500);

        let meter = global::meter("wynnmap-server");

        Self {
            client,
            guilds: guild_state.guilds.clone(),
            extra: extra_data,

            bc_send,

            state: Arc::new(TerritoryState {
                inner: Default::default(),

                bc_recv: Arc::new(bc_recv),
                active_conn: meter.i64_up_down_counter("active-ws-sessions").build(),
            }),
        }
    }

    pub fn run(self) -> Arc<TerritoryState> {
        let state2 = self.state.clone();

        tokio::spawn(async move {
            let tracker = self;

            loop {
                let res = tracker.query_territories().await;

                let waittime = match res {
                    Ok(expires) => {
                        if let Some(exp) = expires {
                            let now = Utc::now();

                            let diff = exp.signed_duration_since(now);

                            tokio::time::sleep_until(
                                Instant::now()
                                    + diff.to_std().unwrap_or_default()
                                    + Duration::from_secs(1),
                            )
                            .await;

                            Duration::from_secs(0)
                        } else {
                            Duration::from_mins(1)
                        }
                    }
                    Err(e) => {
                        error!(error = ?e, "Error occured while querying territories");
                        Duration::from_mins(10)
                    }
                };

                tokio::time::sleep(waittime).await;
            }
        });

        state2
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn query_territories(&self) -> Result<Option<DateTime<Utc>>, AnyError> {
        let (data, expires) = async {
            let res = self
                .client
                .get("https://api.wynncraft.com/v3/guild/list/territory")
                .send()
                .await?;

            let expires = res.expires();
            let data: BTreeMap<Arc<str>, WynnTerritory> = res.parse_json().await?;

            Ok::<_, util::RequestError>((data, expires))
        }
        .instrument(info_span!("fetch"))
        .await?;

        // add connections and res generation data from the extradata and form the territories
        let territories = {
            // get the extradata
            let exdata = self.extra.read().await;

            // create the territory data
            data.iter()
                .map(|(name, t)| {
                    let exdata = exdata.get(name);
                    (
                        name.clone(),
                        Territory {
                            location: t.location,
                            connections: exdata.map(|e| e.connections.clone()).unwrap_or_default(),
                            generates: exdata.map(|e| e.resources).unwrap_or_default(),
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };

        // create owner data
        let owners = {
            let guildlock = self.guilds.read().await;

            data.into_iter()
                .map(|(name, t)| {
                    let mut guild: Guild = t.guild.into();

                    guild.color = guildlock.get(&guild.prefix).and_then(|g| g.color.clone());

                    (
                        name,
                        TerrOwner {
                            guild,
                            acquired: Some(t.acquired),
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };

        // update territory data
        let old_owners = {
            let mut lock = self.state.inner.write().await;

            // update expires and last updated
            lock.expires = expires;
            lock.last_updated = Some(Utc::now());

            // update territories
            lock.territories = territories;

            // update owners with swap for notify
            let mut old_owners = owners.clone();
            mem::swap(&mut old_owners, &mut lock.owners);

            old_owners
        };

        // send broadcasts to notify websockets
        async {
            for (tname, new) in owners {
                let old = old_owners.get(&tname);

                if old != Some(&new) {
                    self.bc_send.send(TerrSockMessage::Capture {
                        name: tname,
                        old: old.cloned(),
                        new,
                    })?;
                }
            }

            Ok::<(), AnyError>(())
        }
        .instrument(info_span!("notify"))
        .await?;

        Ok(expires)
    }
}

#[derive(Deserialize)]
struct WynnTerritory {
    guild: WynnGuild,
    acquired: chrono::DateTime<chrono::Utc>,
    location: Region,
}

#[derive(Deserialize, Clone)]
struct WynnGuild {
    pub uuid: Option<Uuid>,
    pub name: Option<Arc<str>>,
    pub prefix: Option<Arc<str>>,
}

impl From<WynnGuild> for Guild {
    fn from(value: WynnGuild) -> Self {
        Guild {
            uuid: value.uuid,
            name: value
                .name
                .unwrap_or_else(|| "Unknown, Wynn api returned null".into()),
            prefix: value.prefix.unwrap_or_else(|| "???".into()),
            color: None,
        }
    }
}
