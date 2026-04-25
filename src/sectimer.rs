use std::time::Duration;

use chrono::{DateTime, Utc};
use leptos::prelude::*;

/// The current time updated once a second
#[derive(Clone)]
pub struct SecondTimer(pub RwSignal<DateTime<Utc>>);

pub fn provide_second_timer() {
    let signal = RwSignal::new(Utc::now());

    let timer = set_interval_with_handle(
        move || {
            signal.set(Utc::now());
        },
        Duration::from_secs(1),
    )
    .ok();

    on_cleanup(move || {
        if let Some(h) = timer {
            h.clear();
        }
    });

    provide_context(SecondTimer(signal));
}
