use std::{collections::BTreeMap, sync::Arc, time::Duration};

use leptos::prelude::*;
use wynnmap_types::{
    guild::Guild,
    terr::{TerrOwner, Territory},
};

// Displays the name of the territory and the resources it produces.
#[component]
pub fn TerrInfo(
    #[prop(into)] name: Signal<Arc<str>>,
    #[prop(into)] terrs: Signal<BTreeMap<Arc<str>, Territory>>,
) -> impl IntoView {
    view! {
        <div class="p-2">
            <h1 class="text-2xl">{name}</h1>

            <div class="p-2">
                {move ||
                    terrs.read().get(&*name.read()).cloned().map(|terr| {
                        let g = terr.generates;

                        let materials = [
                            (g.emeralds, "emeralds"),
                            (g.ore, "ore"),
                            (g.wood, "wood"),
                            (g.fish, "fish"),
                            (g.crops, "crops")
                        ];

                        materials.into_iter()
                            .filter(|(n, _)| *n > 0)
                            .map(|(n, name)| view! {
                                <div class="flex gap-1 items-center">
                                    <div class={move || format!("icon-{name}")}></div>
                                    <p>"+"{n}" "{name}" per hour"</p>
                                </div>
                            })
                            .collect::<Vec<_>>()
                    })
                }
            </div>
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

            time.set(t);
        },
        Duration::from_secs(1),
    )
    .ok();

    on_cleanup(move || {
        if let Some(i) = i {
            i.clear();
        }
    });

    let treas_tier = move |time: i64| TreasTier::from_time(time);

    view! {
        <div class="p-2">
            <h1 class="text-xl">
                {move || owner.read().guild.name.clone()}" "
                <span class="font-mono">"["{move || owner.read().guild.prefix.clone()}"]"</span>
            </h1>

            {move || time.get().map(|time| view! {
                <div class="p-2">
                    <h2>"Time held: "{move || format_time(time)}</h2>
                    <h2>"Treasury: "<span style:color=move || treas_tier(time).color()>{move || treas_tier(time).name()}</span></h2>
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

enum TreasTier {
    VHigh,
    High,
    Medium,
    Low,
    VLow,
}

impl TreasTier {
    fn from_time(time: i64) -> Self {
        match time {
            t if t < 3600 => Self::VLow,
            t if t < (3600 * 24) => Self::Low,
            t if t < (3600 * 24 * 5) => Self::Medium,
            t if t < (3600 * 24 * 12) => Self::High,
            _ => Self::VHigh,
        }
    }

    const fn name(&self) -> &'static str {
        match self {
            TreasTier::VHigh => "Very High",
            TreasTier::High => "High",
            TreasTier::Medium => "Medium",
            TreasTier::Low => "Low",
            TreasTier::VLow => "Very Low",
        }
    }

    const fn color(&self) -> &'static str {
        match self {
            TreasTier::VHigh => "oklch(0.637 0.237 25.331)",
            TreasTier::High => "oklch(0.705 0.213 47.604)",
            TreasTier::Medium => "oklch(0.795 0.184 86.047)",
            TreasTier::Low => "oklch(0.768 0.233 130.85)",
            TreasTier::VLow => "oklch(0.723 0.219 149.579)",
        }
    }
}
