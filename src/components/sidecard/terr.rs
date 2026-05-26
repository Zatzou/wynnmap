use std::{collections::BTreeMap, sync::Arc};

use leptos::prelude::*;
use wynnmap_types::{
    guild::Guild,
    terr::{TerrState, Territory},
};

use crate::sectimer::SecondTimer;

// Displays the name of the territory and the resources it produces.
#[component]
pub fn TerrInfo(
    #[prop(into)] name: Signal<Arc<str>>,
    #[prop(into)] terrs: Signal<BTreeMap<Arc<str>, Territory>>,
    #[prop(into)] state: Signal<BTreeMap<Arc<str>, TerrState>>,
) -> impl IntoView {
    view! {
        <div>
            <h1 class="text-2xl">{name}</h1>

            <div class="resview">
                {move ||
                    terrs.read().get(&*name.read()).cloned().map(|terr| {
                        let g = terr.generates;

                        let s = state.read().get(&*name.read()).map(|s| s.resources.clone()).unwrap_or_default();

                        let materials = [
                            (g.emerald, "emeralds", s.emerald),
                            (g.ore, "ore", s.ore),
                            (g.wood, "wood", s.wood),
                            (g.fish, "fish", s.fish),
                            (g.crop, "crops", s.crop)
                        ];

                        materials.into_iter()
                            // .filter(|(n, _, _)| *n > 0)
                            .map(|(_n, name, res)| view! {
                                <div class={move || format!("icon-{name}")}></div>
                                <span>{move || format_rate(res.generation)}</span>
                                <span>{res.stored}</span>
                                <span>"/"</span>
                                <span>{res.limit}</span>
                                <span>{name}</span>
                            })
                            .collect::<Vec<_>>()
                    })
                }
            </div>
        </div>
    }
}

fn format_rate(rate: i32) -> String {
    match rate {
        1.. => format!("+{rate}/h"),
        0 => String::new(),
        ..0 => format!("{rate}/h"),
    }
}

// Displays the guild name
#[component]
pub fn GuildName(guild: Signal<Guild>) -> impl IntoView {
    view! {
        <div>
            <h2 class="text-xl">
                {move || guild.read().name.clone()}" "
                <span class="font-mono">"["{move || guild.read().prefix.clone()}"]"</span>
            </h2>
        </div>
    }
}

// Displays the guild name, tag and how long the territory has been owned. Also displays the treasury.
#[component]
pub fn GuildInfo(#[prop(into)] state: Signal<TerrState>) -> impl IntoView {
    let SecondTimer(now) = expect_context::<SecondTimer>();

    let time = Memo::new(move |_| {
        state
            .read()
            .acquired
            .map(|acq| now.read().signed_duration_since(acq).num_seconds())
    });

    view! {
        <div>
            <h1 class="text-xl">
                {move || state.read().guild.name.clone()}" "
                <span class="font-mono">"["{move || state.read().guild.prefix.clone()}"]"</span>
            </h1>

            {move || time.get().map(|time| view! {
                <div class="p-2">
                    <h2>"Time held: "{move || format_time(time)}</h2>
                    <h2>"Treasury: "<span style:color=move || state.read().treasury.color()>{move || state.read().treasury.to_string()}</span></h2>
                    <h2>"Defences: "<span style:color=move || state.read().defences.color()>{move || state.read().defences.to_string()}</span></h2>
                </div>
            })}
        </div>
    }
}

fn format_time(time: i64) -> String {
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
}
