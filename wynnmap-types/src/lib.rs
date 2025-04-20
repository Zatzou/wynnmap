use std::{
    cmp::{max, min},
    sync::Arc,
};

use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod util;
pub mod ws;

/// A wynntils map tile as sourced from the wynntils/static-storage repository.
#[derive(Clone, Deserialize, Serialize)]
pub struct WynntilsMapTile {
    /// The url where the map tile image can be found.
    pub url: Arc<str>,

    pub x1: i32,
    pub x2: i32,
    pub z1: i32,
    pub z2: i32,

    /// The md5 hash of the map tile image.
    pub md5: Arc<str>,

    /// The original name of the map tile image.
    pub orig_name: Option<Arc<str>>,
}

impl WynntilsMapTile {
    pub fn left_side(&self) -> i32 {
        self.x1.min(self.x2)
    }

    pub fn right_side(&self) -> i32 {
        self.x1.max(self.x2)
    }

    pub fn top_side(&self) -> i32 {
        self.z1.min(self.z2)
    }

    pub fn bottom_side(&self) -> i32 {
        self.z1.max(self.z2)
    }

    pub fn width(&self) -> u32 {
        self.x1.abs_diff(self.x2)
    }

    pub fn height(&self) -> u32 {
        self.z1.abs_diff(self.z2)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Territory {
    pub guild: Guild,
    pub acquired: chrono::DateTime<chrono::Utc>,
    pub location: Location,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Guild {
    pub uuid: Option<Uuid>,
    pub name: Option<Arc<str>>,
    pub prefix: Option<Arc<str>>,
    pub color: Option<Arc<str>>,
}

impl Default for Guild {
    fn default() -> Self {
        Self {
            uuid: None,
            name: Some(Arc::from("Nobody")),
            prefix: Some(Arc::from("None")),
            color: Some(Arc::from("#FFFFFF")),
        }
    }
}

impl Guild {
    pub fn get_color(&self) -> (u8, u8, u8) {
        if let Some(color) = &self.color {
            let col = u32::from_str_radix(&color[1..], 16)
                .unwrap_or(0)
                .to_ne_bytes();

            (col[2], col[1], col[0])
        } else {
            if let Some(name) = &self.name {
                let mut hasher = Hasher::new();
                hasher.update(name.as_bytes());
                let hash = hasher.finalize();

                let bytes: Vec<u8> = hash.to_ne_bytes().into_iter().rev().collect();

                (bytes[1], bytes[2], bytes[3])
            } else {
                (255, 255, 255)
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Location {
    pub start: [i32; 2],
    pub end: [i32; 2],
}

impl Location {
    pub fn left_side(&self) -> i32 {
        min(self.start[0], self.end[0])
    }

    pub fn right_side(&self) -> i32 {
        max(self.start[0], self.end[0])
    }

    pub fn top_side(&self) -> i32 {
        min(self.start[1], self.end[1])
    }

    pub fn bottom_side(&self) -> i32 {
        max(self.start[1], self.end[1])
    }

    pub const fn width(&self) -> u32 {
        self.start[0].abs_diff(self.end[0])
    }

    pub const fn height(&self) -> u32 {
        self.start[1].abs_diff(self.end[1])
    }

    /// calculate midpoint on x (horizontal scale)
    pub fn midpoint_x(&self) -> i32 {
        (self.left_side() + self.right_side()) / 2
    }

    /// calculate midpoint on y (vertical scale)
    pub fn midpoint_y(&self) -> i32 {
        (self.top_side() + self.bottom_side()) / 2
    }

    pub fn get_midpoint(&self) -> (i32, i32) {
        (self.midpoint_x(), self.midpoint_y())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ExTerrInfo {
    pub resources: TerrRes,

    pub conns: Vec<Arc<str>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TerrRes {
    pub emeralds: i32,
    pub ore: i32,
    pub crops: i32,
    pub fish: i32,
    pub wood: i32,
}

impl TerrRes {
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
