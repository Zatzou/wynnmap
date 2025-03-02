use std::sync::Arc;

use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}

impl WynntilsMapTile {
    pub fn left_side(&self) -> f64 {
        f64::from(self.x1.min(self.x2))
    }

    pub fn right_side(&self) -> f64 {
        f64::from(self.x1.max(self.x2))
    }

    pub fn top_side(&self) -> f64 {
        f64::from(self.z1.min(self.z2))
    }

    pub fn bottom_side(&self) -> f64 {
        f64::from(self.z1.max(self.z2))
    }

    pub fn width(&self) -> f64 {
        f64::from(self.x1.abs_diff(self.x2))
    }

    pub fn height(&self) -> f64 {
        f64::from(self.z1.abs_diff(self.z2))
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Territory {
    pub guild: Guild,
    pub acquired: chrono::DateTime<chrono::Utc>,
    pub location: Location,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Guild {
    pub uuid: Uuid,
    pub name: Arc<str>,
    pub prefix: Arc<str>,
    pub color: Option<Arc<str>>,
}

impl Guild {
    pub fn get_color(&self) -> (u8, u8, u8) {
        if let Some(color) = &self.color {
            let col = u32::from_str_radix(&color[1..], 16)
                .unwrap_or(0)
                .to_ne_bytes();

            (col[2], col[1], col[0])
        } else {
            let mut hasher = Hasher::new();
            hasher.update(self.name.as_bytes());
            let hash = hasher.finalize();

            let bytes: Vec<u8> = hash.to_ne_bytes().into_iter().rev().collect();

            (bytes[1], bytes[2], bytes[3])
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Location {
    pub start: [i32; 2],
    pub end: [i32; 2],
}

impl Location {
    pub fn left_side(&self) -> f64 {
        f64::from(self.start[0].min(self.end[0]))
    }

    pub fn right_side(&self) -> f64 {
        f64::from(self.start[0].max(self.end[0]))
    }

    pub fn top_side(&self) -> f64 {
        f64::from(self.start[1].min(self.end[1]))
    }

    pub fn bottom_side(&self) -> f64 {
        f64::from(self.start[1].max(self.end[1]))
    }

    pub fn width(&self) -> f64 {
        f64::from(self.start[0].abs_diff(self.end[0]))
    }

    pub fn height(&self) -> f64 {
        f64::from(self.start[1].abs_diff(self.end[1]))
    }

    /// calculate midpoint on x (horizontal scale)
    pub fn midpoint_x(&self) -> f64 {
        (self.left_side() + self.right_side()) / 2.0
    }

    /// calculate midpoint on y (vertical scale)
    pub fn midpoint_y(&self) -> f64 {
        (self.top_side() + self.bottom_side()) / 2.0
    }

    pub fn get_midpoint(&self) -> (f64, f64) {
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
    pub fn has_emeralds(&self) -> bool {
        self.emeralds > 9000
    }

    pub fn has_ore(&self) -> bool {
        self.ore != 0
    }

    pub fn has_crops(&self) -> bool {
        self.crops != 0
    }

    pub fn has_fish(&self) -> bool {
        self.fish != 0
    }

    pub fn has_wood(&self) -> bool {
        self.wood != 0
    }

    pub fn has_res(&self) -> (bool, bool, bool, bool, bool) {
        (
            self.has_emeralds(),
            self.has_crops(),
            self.has_fish(),
            self.has_ore(),
            self.has_wood(),
        )
    }
}
