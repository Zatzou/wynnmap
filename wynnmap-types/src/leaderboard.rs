use std::sync::Arc;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::guild::Guild;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Season {
    pub name: Arc<str>,
    pub start: Timestamp,
    pub end: Timestamp,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SeasonList {
    pub seasons: Vec<Season>,
    pub current: Option<Arc<str>>,
}

pub type GuildLeaderboard = Vec<GuildLeaderboardEntry>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildLeaderboardEntry {
    pub guild: Guild,
    pub score: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SeasonLeaderboard {
    pub season: Season,
    pub leaderboard: GuildLeaderboard,
}
