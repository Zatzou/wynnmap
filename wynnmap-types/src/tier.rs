use std::fmt::Display;

use jiff::SignedDuration;
use serde::{Deserialize, Serialize};

use crate::color::Color;

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
    #[inline]
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
    #[inline]
    pub const fn color(&self) -> Color {
        match self {
            WynnTier::VeryLow => Color::new([0x00, 0xAA, 0x00]), // dark green
            WynnTier::Low => Color::new([0x55, 0xFF, 0x55]),     // green
            WynnTier::Medium => Color::new([0xFF, 0xFF, 0x55]),  // yellow
            WynnTier::High => Color::new([0xFF, 0x55, 0x55]),    // red
            WynnTier::VeryHigh => Color::new([0xAA, 0x00, 0x00]), // dark red
        }
    }

    /// Get the tier based on a defence number calculated by the calculator
    #[inline]
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
    #[inline]
    pub const fn from_time_held(time: SignedDuration) -> Self {
        let seconds = time.as_secs();

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
