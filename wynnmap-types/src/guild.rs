use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{color::Color, terr::CompactState};

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
    #[inline]
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
    #[inline]
    pub fn get_color(&self) -> Color {
        if let Some(col) = self.color.as_ref().and_then(Color::from_hex) {
            col
        } else {
            Color::from_hash(self.name.as_bytes())
        }
    }

    /// Get the hex color of this guild
    #[inline]
    pub fn hex_color(&self) -> String {
        // reformat the color since wynntils appears to give some odd colors
        self.get_color().to_hex()
    }

    #[inline]
    pub(crate) fn apply_diff(&mut self, diff: CompactGuild) {
        if let Some(uuid) = diff.uuid {
            self.uuid = uuid;
        }

        if let Some(name) = diff.name {
            self.name = name;
        }

        if let Some(prefix) = diff.prefix {
            self.prefix = prefix;
        }

        if let Some(color) = diff.color {
            self.color = color;
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct CompactGuild {
    #[serde(rename = "u")]
    pub uuid: Option<Option<Uuid>>,
    #[serde(rename = "n")]
    pub name: Option<Arc<str>>,
    #[serde(rename = "p")]
    pub prefix: Option<Arc<str>>,
    #[serde(rename = "c")]
    pub color: Option<Option<Arc<str>>>,
}

impl CompactGuild {
    pub fn from_full(guild: Guild) -> Self {
        Self {
            uuid: Some(guild.uuid),
            name: Some(guild.name),
            prefix: Some(guild.prefix),
            color: Some(guild.color),
        }
    }

    pub fn from_diff(new: Guild, old: &Guild) -> Self {
        Self {
            uuid: CompactState::diff(new.uuid, &old.uuid),
            name: CompactState::diff(new.name, &old.name),
            prefix: CompactState::diff(new.prefix, &old.prefix),
            color: CompactState::diff(new.color, &old.color),
        }
    }

    pub const fn has_some(&self) -> bool {
        self.uuid.is_some() || self.name.is_some() || self.prefix.is_some() || self.color.is_some()
    }
}
