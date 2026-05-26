use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

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
