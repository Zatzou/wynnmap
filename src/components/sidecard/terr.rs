use std::{collections::HashMap, sync::Arc, time::Duration};

use leptos::prelude::*;
use wynnmap_types::{
    guild::Guild,
    terr::{TerrOwner, Territory},
};

// Displays the name of the territory and the resources it produces.
#[component]
pub fn TerrInfo(
    #[prop(into)] name: Signal<Arc<str>>,
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
) -> impl IntoView {
    view! {
        <div class="p-2">
            <h1 class="text-2xl">{name}</h1>

            {move ||
                terrs.read().get(&*name.read()).cloned().map(|terr| {
                    view! {
                        <div class="p-2">
                            <Show when={move || terr.generates.has_emeralds()}>
                                <div class="flex gap-1 items-center">
                                    <div class="icon-emerald"></div>
                                    <p>"+"{terr.generates.emeralds}" emeralds per hour"</p>
                                </div>
                            </Show>
                            <Show when={move || terr.generates.has_ore()}>
                                <div class="flex gap-1 items-center">
                                    <div class="icon-ores"></div>
                                    <p>"+"{terr.generates.ore}" ore per hour"</p>
                                </div>
                            </Show>
                            <Show when={move || terr.generates.has_wood()}>
                                <div class="flex gap-1 items-center">
                                    <div class="icon-wood"></div>
                                    <p>"+"{terr.generates.wood}" wood per hour"</p>
                                </div>
                            </Show>
                            <Show when={move || terr.generates.has_fish()}>
                                <div class="flex gap-1 items-center">
                                    <div class="icon-fish"></div>
                                    <p>"+"{terr.generates.fish}" fish per hour"</p>
                                </div>
                            </Show>
                            <Show when={move || terr.generates.has_crops()}>
                                <div class="flex gap-1 items-center">
                                    <div class="icon-crops"></div>
                                    <p>"+"{terr.generates.crops}" crops per hour"</p>
                                </div>
                            </Show>
                        </div>
                    }
                })
            }
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
pub fn GuildInfo(#[prop(into)] owner: Signal<TerrOwner>) -> impl IntoView {
    let now = chrono::Utc::now();

    let time = RwSignal::new(
        owner
            .read_untracked()
            .acquired
            .map(|acq| now.signed_duration_since(acq).num_seconds()),
    );

    let i = set_interval_with_handle(
        move || {
            let now = chrono::Utc::now();

            let t = owner
                .read_untracked()
                .acquired
                .map(|acq| now.signed_duration_since(acq).num_seconds());

            time.set(t)
        },
        Duration::from_millis(1000),
    )
    .ok();

    on_cleanup(move || {
        if let Some(i) = i {
            i.clear();
        }
    });

    let timestr = move |time: i64| {
        let days = time / 86400;
        let hours = (time % 86400) / 3600;
        let minutes = (time % 3600) / 60;
        let seconds = time % 60;

        if days > 0 {
            format!("{days}d {hours}h {minutes}m {seconds}s")
        } else if hours > 0 {
            format!("{hours}h {minutes}m {seconds}s")
        } else if minutes > 0 {
            format!("{minutes}m {seconds}s")
        } else {
            format!("{seconds}s")
        }
    };

    let treas_col = move |time: i64| {
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

    let treas_text = move |time: i64| {
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
                {move || owner.read().guild.name.clone()}" "
                <span class="font-mono">"["{move || owner.read().guild.prefix.clone()}"]"</span>
            </h1>

            {move || time.get().map(|time| view! {
                <div class="p-2">
                    <h2>"Time held: "{move || timestr(time)}</h2>
                    <h2>"Treasury: "<span style:color=move || treas_col(time)>{move || treas_text(time)}</span></h2>
                </div>
            })}
        </div>
    }
}
