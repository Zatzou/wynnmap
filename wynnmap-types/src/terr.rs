use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{Region, guild::Guild};

/// A Wynncraft territory
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Territory {
    /// The location of this territory on the map
    pub location: Region,
    /// Names of the territories which connect to this one
    pub connections: HashSet<Arc<str>>,
    /// The resources that this territory generates
    pub generates: Resources,
}

/// Resource values of the given territory
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub struct Resources {
    pub emeralds: i32,
    pub ore: i32,
    pub crops: i32,
    pub fish: i32,
    pub wood: i32,
}

impl Resources {
    pub const fn has_emeralds(&self) -> bool {
        self.emeralds > 9000
    }

    pub const fn has_ore(&self) -> bool {
        self.ore != 0
    }

    pub const fn has_crops(&self) -> bool {
        self.crops != 0
    }

    pub const fn has_fish(&self) -> bool {
        self.fish != 0
    }

    pub const fn has_wood(&self) -> bool {
        self.wood != 0
    }

    pub const fn has_double_ore(&self) -> bool {
        self.ore >= 7200
    }

    pub const fn has_double_crops(&self) -> bool {
        self.crops >= 7200
    }

    pub const fn has_double_fish(&self) -> bool {
        self.fish >= 7200
    }

    pub const fn has_double_wood(&self) -> bool {
        self.wood >= 7200
    }

    pub const fn has_res(&self) -> (bool, bool, bool, bool, bool) {
        (
            self.has_emeralds(),
            self.has_crops(),
            self.has_fish(),
            self.has_ore(),
            self.has_wood(),
        )
    }

    pub const fn has_double_res(&self) -> (bool, bool, bool, bool) {
        (
            self.has_double_crops(),
            self.has_double_fish(),
            self.has_double_ore(),
            self.has_double_wood(),
        )
    }
}

/// Finds the externals of a given territory.
///
/// An external is a territory which is within 3 connections of the given territory.
pub fn find_externals(
    name: &Arc<str>,
    territories: &HashMap<Arc<str>, Territory>,
) -> HashSet<Arc<str>> {
    let mut externals = HashSet::new();

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

/// Structure representing a territory owner and the time when they acquired the territory
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct TerrOwner {
    /// The guild which is the owner
    pub guild: Guild,
    /// The time when they acquired the territory if known
    pub acquired: Option<chrono::DateTime<chrono::Utc>>,
}
