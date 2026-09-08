use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{
    Region,
    guild::Guild,
    resources::{BaseResGen, Resources},
    tier::WynnTier,
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
                externals.append(&mut external.connections.clone());
            }
        }
    }

    externals.remove(name);

    externals
}

#[derive(Serialize, Deserialize)]
pub struct MapState {
    pub terrs: BTreeMap<Arc<str>, TerrState>,
    pub timestamps: TerrTimestamps,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrTimestamps {
    pub updated: Option<Timestamp>,
    pub changed: Option<Timestamp>,
    pub wynntick: Option<Timestamp>,
}

/// Structure representing the state information of the guild
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct TerrState {
    /// Current owner of the territory
    pub guild: Guild,
    /// The time when they acquired the territory if known
    pub acquired: Option<Timestamp>,
    /// Whether or not this territory is the current guilds hq
    pub hq: bool,
    /// Treasury level
    pub treasury: WynnTier,
    /// Defence level
    pub defences: WynnTier,
    /// Resources of the territory
    pub resources: Resources,
}
