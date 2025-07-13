use std::{collections::HashMap, sync::Arc, time::Duration};

use leptos::prelude::*;
use wynnmap_types::{ExTerrInfo, Guild};

// Displays the name of the territory and the resources it produces.
#[component]
pub fn TerrInfo(
    #[prop(into)] name: Signal<Arc<str>>,
    extradata: Signal<HashMap<Arc<str>, ExTerrInfo>>,
) -> impl IntoView {
    view! {
        <div class="p-2">
            <h1 class="text-2xl">{name}</h1>

            {move || if let Some(exdata) = extradata.read().get(&*name.read()).cloned() {
                Some(
                    view! {
                        <div class="p-2">
                            <Show when={move || exdata.resources.emeralds > 0}>
                                <div class="flex gap-1 items-center">
                                    <div class="icon-emerald"></div>
                                    <p>"+"{exdata.resources.emeralds}" emeralds per hour"</p>
                                </div>
                            </Show>
                            <Show when={move || exdata.resources.ore > 0}>
                                <div class="flex gap-1 items-center">
                                    <div class="icon-ores"></div>
                                    <p>"+"{exdata.resources.ore}" ore per hour"</p>
                                </div>
                            </Show>
                            <Show when={move || exdata.resources.wood > 0}>
                                <div class="flex gap-1 items-center">
                                    <div class="icon-wood"></div>
                                    <p>"+"{exdata.resources.wood}" wood per hour"</p>
                                </div>
                            </Show>
                            <Show when={move || exdata.resources.fish > 0}>
                                <div class="flex gap-1 items-center">
                                    <div class="icon-fish"></div>
                                    <p>"+"{exdata.resources.fish}" fish per hour"</p>
                                </div>
                            </Show>
                            <Show when={move || exdata.resources.crops > 0}>
                                <div class="flex gap-1 items-center">
                                    <div class="icon-crops"></div>
                                    <p>"+"{exdata.resources.crops}" crops per hour"</p>
                                </div>
                            </Show>
                        </div>
                    }
                )
            } else {
                None
            }}
        </div>
    }
}

// Displays the guild name
#[component]
pub fn GuildName(guild: Signal<Guild>) -> impl IntoView {
    view! {
        <div class="p-2">
            <h1 class="text-xl">
                {move || guild.read().name.clone()}" "
                <span class="font-mono">"["{move || guild.read().prefix.clone()}"]"</span>
            </h1>
        </div>
    }
}

// Displays the guild name, tag and how long the territory has been owned. Also displays the treasury.
#[component]
pub fn GuildInfo(
    acquired: Signal<chrono::DateTime<chrono::Utc>>,
    guild: Signal<Guild>,
) -> impl IntoView {
    let now = chrono::Utc::now();
    let time = now
        .signed_duration_since(acquired.read_untracked())
        .num_seconds();

    let (time, set_time) = signal(time);

    let i = set_interval_with_handle(
        move || {
            let now = chrono::Utc::now();

            let time = now.signed_duration_since(acquired.read()).num_seconds();

            set_time.set(time);
        },
        Duration::from_millis(1000),
    )
    .ok();

    on_cleanup(move || {
        if let Some(i) = i {
            i.clear();
        }
    });

    let timestr = move || {
        let time = time.get();

        let days = time / 86400;
        let hours = (time % 86400) / 3600;
        let minutes = (time % 3600) / 60;
        let seconds = time % 60;

        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    };

    let treas_col = move || {
        let time = time.get();

        // times based on treasury
        if time < 3600 {
            "oklch(0.723 0.219 149.579)"
        } else if time < (3600 * 24) {
            "oklch(0.768 0.233 130.85)"
        } else if time < (3600 * 24 * 5) {
            "oklch(0.795 0.184 86.047)"
        } else if time < (3600 * 24 * 12) {
            "oklch(0.705 0.213 47.604)"
        } else {
            "oklch(0.637 0.237 25.331)"
        }
    };

    let treas_text = move || {
        let time = time.get();

        // times based on treasury
        if time < 3600 {
            "Very Low"
        } else if time < (3600 * 24) {
            "Low"
        } else if time < (3600 * 24 * 5) {
            "Medium"
        } else if time < (3600 * 24 * 12) {
            "High"
        } else {
            "Very High"
        }
    };

    view! {
        <div class="p-2">
            <h1 class="text-xl">
                {move || guild.read().name.clone()}" "
                <span class="font-mono">"["{move || guild.read().prefix.clone()}"]"</span>
            </h1>

            <div class="p-2">
                <h2>"Time held: "{move || timestr()}</h2>
                <h2>"Treasury: "<span style:color=move || treas_col()>{move || treas_text()}</span></h2>
            </div>
        </div>
    }
}
