use std::fmt::Display;

use serde::{Deserialize, Serialize};

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
