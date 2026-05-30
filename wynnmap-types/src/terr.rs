use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    Region,
    guild::{CompactGuild, Guild},
    resources::{BaseResGen, Resources},
    tier::{CompactTier, WynnTier},
};

/// A Wynncraft territory
///
/// This struct holds the (mostly) static data of each territory, so location, conns and generated resources.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Territory {
    /// The location of this territory on the map
    pub location: Region,
    /// Names of the territories which connect to this one
    pub connections: BTreeSet<Arc<str>>,
    /// The resources that this territory generates
    pub generates: BaseResGen,
}

/// Finds the externals of a given territory.
///
/// An external is a territory which is within 3 connections of the given territory.
pub fn find_externals(
    name: &Arc<str>,
    territories: &BTreeMap<Arc<str>, Territory>,
) -> BTreeSet<Arc<str>> {
    let mut externals = BTreeSet::new();

    externals.insert(name.clone());

    for _ in 0..3 {
        let exts = externals.clone();

        for ext in exts {
            if let Some(external) = territories.get(&ext) {
                for territory in &external.connections {
                    externals.insert(territory.clone());
                }
            }
        }
    }

    externals.remove(name);

    externals
}

#[deprecated]
/// Structure representing a territory owner and the time when they acquired the territory
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct TerrOwner {
    /// The guild which is the owner
    pub guild: Guild,
    /// The time when they acquired the territory if known
    pub acquired: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct MapState {
    pub terrs: BTreeMap<Arc<str>, TerrState>,
    pub timestamps: TerrTimestamps,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TerrTimestamps {
    pub updated: Option<chrono::DateTime<Utc>>,
    pub changed: Option<chrono::DateTime<Utc>>,
    pub wynntick: Option<chrono::DateTime<FixedOffset>>,
}

/// Structure representing the state information of the guild
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct TerrState {
    /// Current owner of the territory
    pub guild: Guild,
    /// The time when they acquired the territory if known
    pub acquired: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether or not this territory is the current guilds hq
    pub hq: bool,
    /// Treasury level
    pub treasury: WynnTier,
    /// Defence level
    pub defences: WynnTier,
    /// Resources of the territory
    pub resources: Resources,
}

impl TerrState {
    pub fn apply_diff(&mut self, diff: CompactState) {
        if let Some(guild) = diff.guild {
            self.guild.apply_diff(guild);
        }

        if let Some(acquired) = diff.acquired {
            self.acquired = acquired;
        }

        if let Some(hq) = diff.hq {
            self.hq = hq;
        }

        if let Some(treasury) = diff.treasury {
            self.treasury = treasury.into();
        }

        if let Some(defences) = diff.defences {
            self.defences = defences.into();
        }

        if let Some(resources) = diff.resources {
            if let Some(r) = resources[0] {
                self.resources.emerald = r.into();
            }

            if let Some(r) = resources[1] {
                self.resources.ore = r.into();
            }

            if let Some(r) = resources[2] {
                self.resources.crop = r.into();
            }

            if let Some(r) = resources[3] {
                self.resources.fish = r.into();
            }

            if let Some(r) = resources[4] {
                self.resources.wood = r.into();
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CompactState {
    #[serde(rename = "g")]
    guild: Option<CompactGuild>,
    #[serde(rename = "a")]
    acquired: Option<Option<chrono::DateTime<chrono::Utc>>>,
    #[serde(rename = "h")]
    hq: Option<bool>,
    #[serde(rename = "t")]
    treasury: Option<CompactTier>,
    #[serde(rename = "d")]
    defences: Option<CompactTier>,
    #[serde(rename = "r")]
    resources: Option<[Option<(i32, i32, i32)>; 5]>,
}

impl CompactState {
    pub fn from_full(state: TerrState) -> Self {
        let res = [
            Some(state.resources.emerald.into()),
            Some(state.resources.ore.into()),
            Some(state.resources.crop.into()),
            Some(state.resources.fish.into()),
            Some(state.resources.wood.into()),
        ];

        Self {
            guild: Some(CompactGuild::from_full(state.guild)),
            acquired: Some(state.acquired),
            hq: Some(state.hq),
            treasury: Some(state.treasury.into()),
            defences: Some(state.defences.into()),
            resources: Some(res),
        }
    }

    pub fn from_diff(new: TerrState, old: &TerrState) -> Self {
        let guild = CompactGuild::from_diff(new.guild, &old.guild);

        let res = [
            CompactState::diff(new.resources.emerald, &old.resources.emerald).map(|r| r.into()),
            CompactState::diff(new.resources.ore, &old.resources.ore).map(|r| r.into()),
            CompactState::diff(new.resources.crop, &old.resources.crop).map(|r| r.into()),
            CompactState::diff(new.resources.fish, &old.resources.fish).map(|r| r.into()),
            CompactState::diff(new.resources.wood, &old.resources.wood).map(|r| r.into()),
        ];

        let has_res = res[0].is_some()
            || res[1].is_some()
            || res[2].is_some()
            || res[3].is_some()
            || res[4].is_some();

        Self {
            guild: guild.has_some().then_some(guild),
            acquired: CompactState::diff(new.acquired, &old.acquired),
            hq: CompactState::diff(new.hq, &old.hq),
            treasury: CompactState::diff(new.treasury, &old.treasury).map(|t| t.into()),
            defences: CompactState::diff(new.defences, &old.defences).map(|t| t.into()),
            resources: has_res.then_some(res),
        }
    }

    pub(crate) fn diff<T: PartialEq>(new: T, old: &T) -> Option<T> {
        if new != *old { Some(new) } else { None }
    }
}
