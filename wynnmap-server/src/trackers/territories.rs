use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
    sync::Arc,
    time::Duration,
};

use chrono::{DateTime, Utc};
use opentelemetry::{global, metrics::Gauge};
use serde::Deserialize;
use tokio::{
    select,
    sync::{RwLock, broadcast, mpsc},
    time::Instant,
};
use tracing::{Instrument, error, info_span};
use uuid::Uuid;
use wynnmap_types::{
    Region, encoding,
    guild::Guild,
    resources::{BaseResGen, ResourceType, ResourceValues, Resources},
    terr::{CompactState, TerrState, Territory},
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

    bc_bytes: broadcast::Sender<Arc<Vec<u8>>>,
    terrs_updated: Gauge<i64>,

    updated: Gauge<i64>,
    changed: Gauge<i64>,
    wynntick: Gauge<i64>,

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
            terrs_updated: meter
                .i64_gauge("terrs_updated")
                .with_description("Territories updated this cycle")
                .build(),

            updated: meter.i64_gauge("wynnmap.terrs.updated").build(),
            changed: meter.i64_gauge("wynnmap.terrs.changed").build(),
            wynntick: meter.i64_gauge("wynnmap.terrs.wynntick").build(),

            state: Arc::new(TerritoryState {
                inner: Default::default(),

                bc_bytes: Arc::new(bc_bytes_r),
                ws_conns: meter
                    .i64_up_down_counter("active-ws-sessions")
                    .with_description("Active websocket sessions")
                    .build(),
            }),
        }
    }

    pub fn run(self) -> Arc<TerritoryState> {
        let state = self.state.clone();
        let (notify_send, mut notify_recv) = mpsc::channel(100);
        let bc_bytes = self.bc_bytes.clone();

        // tracker code
        tokio::spawn(async move {
            let tracker = self;

            loop {
                let sender = notify_send.clone();
                let res = tracker.query_territories(sender).await;

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

        // notifier code
        {
            let state = state.inner.clone();
            tokio::spawn(async move {
                loop {
                    let data = select! {
                        Some(data) = notify_recv.recv() => {
                            data
                        },
                        // send last updated notifications every 60s
                        _ = tokio::time::sleep(Duration::from_secs(60)) => {
                            TerrSockMessage::LastUpdate(state.read().await.timestamps)
                        }
                    };

                    bc_bytes
                        .send(encoding::encode_data(&data).unwrap().into())
                        .unwrap();
                }
            });
        }

        state
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn query_territories(
        &self,
        notify_send: mpsc::Sender<TerrSockMessage>,
    ) -> Result<Option<DateTime<Utc>>, AnyError> {
        let (data, expires, wynntick) = async {
            let res = self
                .client
                .get("https://api.wynncraft.com/v3/guild/list/territory")
                .send()
                .await?;

            let expires = res.expires();
            let wynntick = res
                .get_header("territorylasttick")
                .and_then(|t| DateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S%.f%:z").ok());
            let data: BTreeMap<Arc<str>, WynnTerritory> = res.parse_json().await?;

            Ok::<_, util::RequestError>((data, expires, wynntick))
        }
        .instrument(info_span!("fetch"))
        .await?;

        // create owner data
        let state = {
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
        let (old_state, timestamps) = {
            let mut lock = self.state.inner.write().await;

            // update expires and last updated
            lock.expires = expires.unwrap_or_default();

            // update territories
            if lock.territories != territories {
                lock.territories = territories;
                lock.territories_modified = Utc::now();
            }

            // update etag values
            lock.territories_etag = terr_etag;

            // update owners with swap for notify
            let mut old_state = state.clone();
            mem::swap(&mut old_state, &mut lock.state);

            let now = Utc::now();
            lock.timestamps.updated = Some(now);
            self.updated.record(now.timestamp_millis(), &[]);

            if old_state != lock.state {
                lock.timestamps.changed = lock.timestamps.updated;
                self.changed.record(now.timestamp_millis(), &[]);
            }

            lock.timestamps.wynntick = wynntick;
            if let Some(t) = wynntick {
                self.wynntick.record(t.timestamp_millis(), &[]);
            }

            // return old owners for notifications
            (old_state, lock.timestamps)
        };

        // send broadcasts to notify websockets
        if !old_state.is_empty() {
            async {
                let mut updateds = BTreeMap::new();

                for (tname, new) in state {
                    let old = old_state.get(&tname);

                    if let Some(old) = old
                        && old != &new
                    {
                        updateds.insert(tname, CompactState::from_diff(new, old));
                    } else if old.is_none() {
                        updateds.insert(tname, CompactState::from_full(new));
                    }
                }

                self.terrs_updated.record(updateds.len() as i64, &[]);

                if !updateds.is_empty() {
                    notify_send
                        .send(TerrSockMessage::Update(updateds, timestamps))
                        .await?;
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
