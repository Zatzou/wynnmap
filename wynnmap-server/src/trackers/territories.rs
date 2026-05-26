use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
    sync::Arc,
    time::Duration,
};

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
    resources::{BaseResGen, ResourceType, ResourceValues, Resources},
    terr::{TerrState, Territory},
    tier::WynnTier,
    ws::TerrSockMessage,
};

use crate::{
    AnyError,
    config::Config,
    etag::sha224_etag_json,
    state::{GuildState, TerritoryState},
    trackers::util::{self, ResponseExt},
};

pub struct TerritoryTracker {
    client: reqwest::Client,
    guilds: Arc<RwLock<BTreeMap<Arc<str>, Guild>>>,

    bc_bytes: broadcast::Sender<Arc<str>>,

    state: Arc<TerritoryState>,
}

impl TerritoryTracker {
    pub fn with_config(config: &Config, guild_state: &GuildState) -> Self {
        let client = util::reqwest_client_from_conf(config);

        let (bc_bytes_s, bc_bytes_r) = broadcast::channel(100);

        let meter = global::meter("wynnmap-server");

        Self {
            client,
            guilds: guild_state.guilds.clone(),

            bc_bytes: bc_bytes_s,

            state: Arc::new(TerritoryState {
                inner: Default::default(),

                bc_bytes: Arc::new(bc_bytes_r),
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

        // create owner data
        let owners = {
            let guildlock = self.guilds.read().await;

            data.clone()
                .into_iter()
                .map(|(name, t)| {
                    let mut guild: Guild = t.guild.into();

                    guild.color = guildlock.get(&guild.prefix).and_then(|g| g.color.clone());

                    let resval_of = |rt| {
                        t.resources
                            .iter()
                            .find(|r| r.kind == rt)
                            .map(|r| r.as_resval())
                            .unwrap_or_default()
                    };

                    (
                        name,
                        TerrState {
                            guild,
                            acquired: Some(t.acquired.unwrap_or_default()),
                            hq: t.hq,
                            treasury: t.treasury,
                            defences: t.defences,
                            resources: Resources {
                                emerald: resval_of(ResourceType::Emerald),
                                ore: resval_of(ResourceType::Ore),
                                crop: resval_of(ResourceType::Crop),
                                fish: resval_of(ResourceType::Fish),
                                wood: resval_of(ResourceType::Wood),
                            },
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };

        // convert territory data
        let territories = data.into_iter().map(|(n, t)| (n, t.into())).collect();

        // calculate etags
        let terr_etag = sha224_etag_json(&territories);

        // update territory data
        let old_owners = {
            let mut lock = self.state.inner.write().await;

            // update expires and last updated
            lock.expires = expires.unwrap_or_default();
            lock.last_updated = Utc::now();

            // update territories
            if lock.territories != territories {
                lock.territories = territories;
                lock.territories_modified = Utc::now();
            }

            // update etag values
            lock.territories_etag = terr_etag;

            // update owners with swap for notify
            if lock.state != owners {
                let mut old_owners = owners.clone();
                mem::swap(&mut old_owners, &mut lock.state);

                // return old owners for notifications
                Some(old_owners)
            } else {
                None
            }
        };

        // send broadcasts to notify websockets
        if let Some(old_owners) = old_owners {
            async {
                let mut updateds = BTreeMap::new();

                for (tname, new) in owners {
                    let old = old_owners.get(&tname);

                    if old != Some(&new) {
                        updateds.insert(tname, new);
                    }
                }

                if !updateds.is_empty() {
                    self.bc_bytes.send(
                        serde_json::to_string(&TerrSockMessage::Update(updateds))
                            .unwrap()
                            .into(),
                    )?;
                } else {
                    self.bc_bytes.send(
                        serde_json::to_string(&TerrSockMessage::LastUpdate {
                            ts: self.state.inner.read().await.last_updated,
                        })
                        .unwrap()
                        .into(),
                    )?;
                }

                Ok::<(), AnyError>(())
            }
            .instrument(info_span!("notify"))
            .await?;
        }

        Ok(expires)
    }
}

#[derive(Deserialize, Clone)]
struct WynnTerritory {
    guild: WynnGuild,
    acquired: Option<DateTime<Utc>>,
    location: Region,
    #[serde(default)]
    hq: bool,
    resources: Vec<WynnRes>,
    links: BTreeSet<Arc<str>>,
    treasury: WynnTier,
    defences: WynnTier,
}

impl From<WynnTerritory> for Territory {
    fn from(t: WynnTerritory) -> Self {
        let rescount = |rt| {
            t.resources
                .iter()
                .find(|r| r.kind == rt)
                .map(|r| r.base_gen)
                .unwrap_or(0)
        };

        Self {
            location: t.location,
            connections: t.links,
            generates: BaseResGen {
                emerald: rescount(ResourceType::Emerald),
                ore: rescount(ResourceType::Ore),
                crop: rescount(ResourceType::Crop),
                fish: rescount(ResourceType::Fish),
                wood: rescount(ResourceType::Wood),
            },
        }
    }
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
            name: value.name.unwrap_or_else(|| "Nobody".into()),
            prefix: value.prefix.unwrap_or_else(|| "None".into()),
            color: Some("#FFFFFF".into()),
        }
    }
}

#[derive(Deserialize, Clone)]
struct WynnRes {
    #[serde(rename = "type")]
    kind: ResourceType,
    generation: i32,
    #[serde(alias = "baseGeneration")]
    base_gen: i32,
    stored: i32,
    limit: i32,
}

impl WynnRes {
    fn as_resval(&self) -> ResourceValues {
        ResourceValues {
            generation: self.generation,
            stored: self.stored,
            limit: self.limit,
        }
    }
}
