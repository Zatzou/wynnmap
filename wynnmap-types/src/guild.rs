use std::sync::Arc;

use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Struct representing a guild
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Guild {
    /// UUID of the guild if known
    pub uuid: Option<Uuid>,
    /// Name of the guild
    pub name: Arc<str>,
    /// Prefix or "tag" of the guild usually displayed in square brackets
    pub prefix: Arc<str>,
    /// Color of the guild usually gotten from either the wynntils api or calculated
    ///
    /// The color is calculated by crc32ing the name of the guild and using the first 3 bytes of the crc32 result as the rgb values
    pub color: Option<Arc<str>>,
}

impl Default for Guild {
    fn default() -> Self {
        Self {
            uuid: None,
            name: Arc::from("Nobody"),
            prefix: Arc::from("None"),
            color: Some(Arc::from("#FFFFFF")),
        }
    }
}

impl Guild {
    /// Get the color of this guild
    ///
    /// This function falls back to calculate the color if no color is given
    pub fn get_color(&self) -> (u8, u8, u8) {
        if let Some(color) = &self.color {
            let col = u32::from_str_radix(&color[1..], 16)
                .unwrap_or(0)
                .to_ne_bytes();

            (col[2], col[1], col[0])
        } else {
            self.calculate_color()
        }
    }

    /// Get the hex color of this guild
    pub fn hex_color(&self) -> String {
        if let Some(col) = &self.color {
            col.to_string()
        } else {
            let col = self.calculate_color();

            format!("#{:02X}{:02X}{:02X}", col.0, col.1, col.2)
        }
    }

    /// Calculate the guild color using the wynntils crc32 method
    ///
    /// This gives the default guild color which wynntils would assign a given guild
    pub fn calculate_color(&self) -> (u8, u8, u8) {
        let mut hasher = Hasher::new();
        hasher.update(self.name.as_bytes());
        let hash = hasher.finalize();

        let bytes: Vec<u8> = hash.to_ne_bytes().into_iter().rev().collect();

        (bytes[1], bytes[2], bytes[3])
    }
}
