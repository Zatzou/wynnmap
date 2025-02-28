use std::sync::Arc;

use crc32fast::Hasher;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct WynntilsMapTile {
    pub url: Arc<str>,
    pub x1: i32,
    pub x2: i32,
    pub z1: i32,
    pub z2: i32,
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

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Territory {
    pub guild: String,
    #[serde(rename = "guildPrefix")]
    pub guild_prefix: String,
    #[serde(rename = "guildColor")]
    pub color: Option<String>,

    pub location: Location,
}

impl Territory {
    pub fn get_color(&self) -> (u8, u8, u8) {
        if let Some(color) = &self.color {
            let col = u32::from_str_radix(&color[1..], 16)
                .unwrap_or(0)
                .to_ne_bytes();

            (col[2], col[1], col[0])
        } else {
            let mut hasher = Hasher::new();
            hasher.update(self.guild.as_bytes());
            let hash = hasher.finalize();

            let bytes: Vec<u8> = hash.to_ne_bytes().into_iter().rev().collect();

            (bytes[1], bytes[2], bytes[3])
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Location {
    #[serde(rename = "startX")]
    pub start_x: i32,
    #[serde(rename = "startZ")]
    pub start_z: i32,
    #[serde(rename = "endX")]
    pub end_x: i32,
    #[serde(rename = "endZ")]
    pub end_z: i32,
}

impl Location {
    pub fn width(&self) -> f64 {
        f64::from(self.start_x.abs_diff(self.end_x))
    }

    pub fn height(&self) -> f64 {
        f64::from(self.start_z.abs_diff(self.end_z))
    }

    pub fn left_side(&self) -> f64 {
        f64::from(self.start_x.min(self.end_x))
    }

    pub fn right_side(&self) -> f64 {
        f64::from(self.start_x.max(self.end_x))
    }

    pub fn top_side(&self) -> f64 {
        f64::from(self.start_z.min(self.end_z))
    }

    pub fn bottom_side(&self) -> f64 {
        f64::from(self.start_z.max(self.end_z))
    }
}
