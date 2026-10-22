use std::{collections::BTreeMap, sync::Arc, time::Duration};

use jiff::Timestamp;
use serde::Deserialize;
use tokio::{
    join,
    sync::{RwLock, watch},
};
use tracing::{Instrument, error, info, info_span, warn};
use uuid::Uuid;
use wynnmap_types::{
    guild::Guild,
    leaderboard::{GuildLeaderboardEntry, Season, SeasonLeaderboard, SeasonList},
};

use crate::{
    config::Config,
    etag::sha224_etag_json,
    state::{GuildList, GuildState, LeaderboardState},
    trackers::util::{self, ResponseExt},
};

pub struct LeaderboardTracker {
    client: reqwest::Client,

    guilds: Arc<RwLock<GuildList>>,
    guilds_done: watch::Receiver<bool>,

    state: Arc<LeaderboardState>,
}

impl LeaderboardTracker {
    pub fn with_config(config: &Config, guild_state: &GuildState) -> Self {
        let client = util::reqwest_client_from_conf(config);

        Self {
            client,

            guilds: guild_state.guilds.clone(),
            guilds_done: guild_state.done.clone(),

            state: Default::default(),
        }
    }

    pub fn run(mut self) -> Arc<LeaderboardState> {
        let state2 = self.state.clone();

        tokio::spawn(async move {
            // wait for guilds to finish so we get accurate colors
            let _ = self.guilds_done.wait_for(|done| *done).await;

            let tracker = self;

            loop {
                let res = tracker.query_guilds().await;

                let waittime = match res {
                    Ok(_) => {
                        info!("Completed leaderboard update");
                        Duration::from_hours(1)
                    }
                    Err(e) => {
                        error!(error = ?e, "Error occured while querying leaderboards");
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
        let seasons: WynnSeasonsList = async {
            let res = self
                .client
                .get("https://api.wynncraft.com/v3/guild/seasons")
                .send()
                .await?;

            res.parse_json().await
        }
        .instrument(info_span!("fetch"))
        .await?;

        let existing_seasons = {
            let lock = self.state.season_lb.read().await;

            lock.keys().cloned().collect::<Vec<_>>()
        };

        let mut seasons = seasons
            .into_iter()
            .map(|(n, s)| Season {
                name: n,
                start: s.start,
                end: s.end,
            })
            .collect::<Vec<_>>();

        seasons.sort_by_key(|s| s.start);

        let now = Timestamp::now();
        let current = seasons
            .iter()
            .find(|s| s.start <= now && s.end >= now)
            .map(|s| s.name.clone());

        let season_lb = async {
            let mut leaderboards = BTreeMap::new();

            let guilds = self.guilds.read().await;

            // during first fetch get all leaderboards otherwise only get ones which have either not ended or have ended in the last 10h
            for season in seasons.iter().filter(|s| {
                (s.end + Duration::from_hours(10)) >= now || !existing_seasons.contains(&s.name)
            }) {
                let lb = self.fetch_lb(&season.name).await?;

                let leaderboard = lb
                    .into_values()
                    .map(|e| e.into_entry(&guilds))
                    .collect::<Vec<_>>();

                leaderboards.insert(
                    season.name.clone(),
                    SeasonLeaderboard {
                        season: season.clone(),
                        leaderboard,
                    },
                );
            }

            Ok::<_, util::RequestError>(leaderboards)
        }
        .instrument(info_span!("fetch_lb"))
        .await?;

        let season_list = {
            let list = SeasonList { seasons, current };
            let etag = sha224_etag_json(&list);
            (Arc::new(list), etag)
        };

        async {
            let (mut sl, mut leaderboards) =
                join!(self.state.season_list.write(), self.state.season_lb.write());

            *sl = season_list;

            for (s, lb) in season_lb {
                let etag = sha224_etag_json(&lb);
                leaderboards.insert(s, (Arc::new(lb), etag));
            }
        }
        .instrument(info_span!("update"))
        .await;

        Ok(())
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn fetch_lb(&self, season: &Arc<str>) -> Result<WynnLeaderboard, util::RequestError> {
        let res = self
            .client
            .get(format!(
                "https://api.wynncraft.com/v3/leaderboards/guildSeason{season}"
            ))
            .send()
            .await?;

        let rl_rem = res.ratelimit_remaining();
        let rl_res = res.ratelimit_reset();

        let data = res.parse_json().await;

        if let Some(rl) = rl_rem
            && rl < 5
        {
            warn!(reset = rl_res, "Ratelimit hit");
            tokio::time::sleep(Duration::from_secs(rl_res.unwrap_or(60) + 1)).await;
        }

        data
    }
}

type WynnSeasonsList = BTreeMap<Arc<str>, WynnSeasonsListItem>;

#[derive(Deserialize)]
struct WynnSeasonsListItem {
    #[serde(alias = "startDate")]
    start: Timestamp,
    #[serde(alias = "endDate")]
    end: Timestamp,
}

type WynnLeaderboard = BTreeMap<i32, WynnLeaderboardEntry>;

#[derive(Deserialize)]
struct WynnLeaderboardEntry {
    score: i32,

    name: Arc<str>,
    uuid: Option<Uuid>,
    prefix: Arc<str>,
}

impl WynnLeaderboardEntry {
    fn into_entry(self, gl: &GuildList) -> GuildLeaderboardEntry {
        let guild = Guild {
            uuid: self.uuid,
            name: self.name,
            color: gl.get(&self.prefix).and_then(|g| g.color.clone()),
            prefix: self.prefix,
        };

        GuildLeaderboardEntry {
            guild,
            score: self.score,
        }
    }
}
