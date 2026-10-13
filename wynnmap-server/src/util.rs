use jiff::{Timestamp, fmt::rfc2822::DateTimePrinter};

/// Format a date into the format that http headers expect which is rfc 2822 with "+0000" replaced to "GMT"
pub fn header_date(dt: impl Into<Timestamp>) -> String {
    let dt = dt.into();

    DateTimePrinter::new()
        .timestamp_to_rfc9110_string(&dt)
        .unwrap_or_default()
}
