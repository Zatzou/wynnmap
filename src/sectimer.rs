use std::time::Duration;

use jiff::Timestamp;
use leptos::prelude::*;

/// The current time updated once a second
#[derive(Clone)]
pub struct SecondTimer(pub RwSignal<Timestamp>);

pub fn provide_second_timer() {
    let signal = RwSignal::new(Timestamp::now());

    let timer = set_interval_with_handle(
        move || {
            signal.set(Timestamp::now());
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
