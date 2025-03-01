use std::sync::Arc;

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

#[derive(Clone, Deserialize, Serialize)]
pub struct Territory {
    pub guild: Guild,
    pub acquired: chrono::DateTime<chrono::Utc>,
    pub location: Location,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Guild {
    pub uuid: Uuid,
    pub name: Arc<str>,
    pub prefix: Arc<str>,
    pub color: Option<Arc<str>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Location {
    pub start: [i32; 2],
    pub end: [i32; 2],
}
