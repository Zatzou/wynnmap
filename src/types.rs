use std::sync::Arc;

use leptos_leaflet::prelude::Position;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct WynntilsMapTile {
    pub url: Arc<str>,
    pub x1: i32,
    pub x2: i32,
    pub z1: i32,
    pub z2: i32,
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
    pub fn get_color(&self) -> String {
        self.color.clone().unwrap_or_else(|| "#000000".to_string())
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
    pub fn into_posvec(&self) -> Vec<Position> {
        vec![
            Position::new(-self.start_z as f64, self.start_x as f64),
            Position::new(-self.end_z as f64, self.start_x as f64),
            Position::new(-self.end_z as f64, self.end_x as f64),
            Position::new(-self.start_z as f64, self.end_x as f64),
        ]
    }

    pub fn middle(&self) -> Position {
        Position::new(
            (-self.start_z as f64 + -self.end_z as f64) / 2.0,
            (self.start_x as f64 + self.end_x as f64) / 2.0,
        )
    }
}
