use std::fmt::Display;

use jiff::SignedDuration;

const fn times(time: SignedDuration) -> (i64, i64, i64, i64) {
    let days = time.as_hours() / 24;
    let hours = time.as_hours() % 24;
    let minutes = time.as_mins() % 60;
    let seconds = time.as_secs() % 60;

    (days, hours, minutes, seconds)
}

/// Format times in a long format eg. (1d 12h 15m 30s)
pub fn fmt_time_long(time: SignedDuration) -> String {
    match times(time) {
        (0, 0, 0, s) => format!("{s}s"),
        (0, 0, m, s) => format!("{m}m {s}"),
        (0, h, m, s) => format!("{h}h {m}m {s}s"),
        (d, h, m, s) => format!("{d}d {h}h {m}m {s}s"),
    }
}

/// Format times in a short format eg. (1d 12h) (12h 15m) (15m 30s)
pub fn fmt_time_short(time: SignedDuration) -> String {
    match times(time) {
        (0, 0, 0, s) => format!("{s}s"),
        (0, 0, m, s) => format!("{m}m {s}s"),
        (0, h, m, _) => format!("{h}h {m}m"),
        (d, h, _, _) => format!("{d}d {h}h"),
    }
}

/// Format a value as pixels for css
pub fn as_px(px: impl Display) -> String {
    format!("{px}px")
}
