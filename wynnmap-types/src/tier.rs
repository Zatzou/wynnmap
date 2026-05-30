use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord,
)]
pub enum WynnTier {
    #[default]
    #[serde(rename = "VERY_LOW")]
    VeryLow,
    #[serde(rename = "LOW")]
    Low,
    #[serde(rename = "MEDIUM")]
    Medium,
    #[serde(rename = "HIGH")]
    High,
    #[serde(rename = "VERY_HIGH")]
    VeryHigh,
}

impl Display for WynnTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WynnTier::VeryLow => "Very Low",
                WynnTier::Low => "Low",
                WynnTier::Medium => "Medium",
                WynnTier::High => "High",
                WynnTier::VeryHigh => "Very High",
            }
        )
    }
}

impl WynnTier {
    /// Return the hex color generally used for this tier
    pub const fn color(&self) -> &'static str {
        match self {
            WynnTier::VeryLow => "#00AA00",  // dark green
            WynnTier::Low => "#55FF55",      // green
            WynnTier::Medium => "#FFFF55",   // yellow
            WynnTier::High => "#FF5555",     // red
            WynnTier::VeryHigh => "#AA0000", // dark red
        }
    }

    /// Get the tier based on a defence number calculated by the calculator
    pub const fn from_defnum(num: i32) -> Self {
        match num {
            41.. => Self::VeryHigh,
            23.. => Self::High,
            11.. => Self::Medium,
            -2.. => Self::Low,
            _ => Self::VeryLow,
        }
    }

    /// Get the tier based on seconds a territory has been held
    pub const fn from_secs_held(seconds: i64) -> Self {
        if seconds < 3600 {
            Self::VeryLow
        } else if seconds < (3600 * 24) {
            Self::Low
        } else if seconds < (3600 * 24 * 5) {
            Self::Medium
        } else if seconds < (3600 * 24 * 12) {
            Self::High
        } else {
            Self::VeryHigh
        }
    }
}

#[repr(u8)]
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr, PartialEq)]
pub(crate) enum CompactTier {
    VeryLow = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    VeryHigh = 4,
}

impl From<CompactTier> for WynnTier {
    fn from(value: CompactTier) -> Self {
        match value {
            CompactTier::VeryLow => Self::VeryLow,
            CompactTier::Low => Self::Low,
            CompactTier::Medium => Self::Medium,
            CompactTier::High => Self::High,
            CompactTier::VeryHigh => Self::VeryHigh,
        }
    }
}

impl From<WynnTier> for CompactTier {
    fn from(value: WynnTier) -> Self {
        match value {
            WynnTier::VeryLow => Self::VeryLow,
            WynnTier::Low => Self::Low,
            WynnTier::Medium => Self::Medium,
            WynnTier::High => Self::High,
            WynnTier::VeryHigh => Self::VeryHigh,
        }
    }
}
