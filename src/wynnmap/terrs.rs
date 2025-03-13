use std::{collections::HashMap, sync::Arc, time::Duration};

use leptos::prelude::*;
use wynnmap_types::{ExTerrInfo, Territory};

use crate::settings::use_toggle;

#[component]
pub fn TerrView(
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
    extradata: Signal<HashMap<Arc<str>, ExTerrInfo>>,
) -> impl IntoView {
    view! {
        <div>
            <For
                each=move || terrs.get().into_iter()
                key=|(k, v)| (k.clone(), v.guild.clone())
                children=move |(k, v)| {
                    view! {
                        <Territory name=k terr=v.into() extradata=extradata />
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn Territory(
    name: Arc<str>,
    terr: Signal<Territory>,
    extradata: Signal<HashMap<Arc<str>, ExTerrInfo>>,
) -> impl IntoView {
    let col = terr.read().guild.get_color();
    let col_rgb = format!("{}, {}, {}", col.0, col.1, col.2);
    let col_rgb2 = col_rgb.clone();

    let res = Memo::new(move |_| {
        extradata
            .read()
            .get(&name)
            .map_or((false, false, false, false, false), |e| {
                e.resources.has_res()
            })
    });

    view! {
        <div class="wynnmap-item guildterr"
            style:width={move || format!("{}px", terr.read().location.width())}
            style:height={move || format!("{}px", terr.read().location.height())}
            style:transform={move || format!("translate({}px, {}px)", terr.read().location.left_side(), terr.read().location.top_side())}
            style:background-color={move || format!("rgba({}, 0.35)", col_rgb)}
            style:border-color={move || format!("rgb({})", col_rgb2)}
        >
            <AttackBorder terr=terr />
            <h3 class="font-bold text-3xl text-white textshadow">{terr.read().guild.prefix.clone()}</h3>
            <ResIcons res=res />
            <TerrTimer terr=terr />
        </div>
    }
}

#[component]
fn ResIcons(#[prop(into)] res: Signal<(bool, bool, bool, bool, bool)>) -> impl IntoView {
    let show_res = use_toggle("resico", true);

    move || {
        if show_res.get() {
            Some(view! {
                <div class="flex pb-1" > // class:hidden={move || !show_res.get()}
                    // this is here so that tailwinds cli realizes that this class is used
                    // class="hidden"
                    <div class="icon-emerald" class:hidden={move || !res.get().0}></div>
                    <div class="icon-crops" class:hidden={move || !res.get().1}></div>
                    <div class="icon-fish" class:hidden={move || !res.get().2}></div>
                    <div class="icon-ores" class:hidden={move || !res.get().3}></div>
                    <div class="icon-wood" class:hidden={move || !res.get().4}></div>
                </div>
            })
        } else {
            None
        }
    }
}

#[component]
fn TerrTimer(terr: Signal<Territory>) -> impl IntoView {
    let show_timers = use_toggle("timers", true);

    let now = chrono::Utc::now();
    let time = now
        .signed_duration_since(terr.read().acquired)
        .num_seconds();

    let (time, set_time) = signal(time);

    let i = set_interval_with_handle(
        move || {
            let now = chrono::Utc::now();

            let time = now
                .signed_duration_since(terr.read().acquired)
                .num_seconds();

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
            format!("{}d {}h", days, hours)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    };

    let color = move || {
        let time = time.get();

        // times based on treasury
        if time < 3600 {
            "background-color: oklch(0.723 0.219 149.579 / .6)"
        } else if time < (3600 * 24) {
            "background-color: oklch(0.768 0.233 130.85 / .6);"
        } else if time < (3600 * 24 * 5) {
            "background-color: oklch(0.795 0.184 86.047 / .6);"
        } else if time < (3600 * 24 * 12) {
            "background-color: oklch(0.705 0.213 47.604 / .6);"
        } else {
            "background-color: oklch(0.637 0.237 25.331 / .6);"
        }
    };

    view! {
        <h4 class="px-2 rounded-2xl text-sm text-center whitespace-nowrap" style={move || color} class:hidden={move || !show_timers.get()}>{timestr}</h4>
    }
}

/// The component rendering the attack timer border for territories which are on attack cooldown.
#[component]
fn AttackBorder(terr: Signal<Territory>) -> impl IntoView {
    let now = chrono::Utc::now();
    let time = now
        .signed_duration_since(terr.read().acquired)
        .num_milliseconds();

    let (time, set_time) = signal(time);

    move || {
        if time.get() < 599_000 {
            let h = set_timeout_with_handle(
                move || {
                    set_time.set(599_000);
                },
                Duration::from_millis((599_000 - time.get()).max(1000) as u64),
            )
            .ok();

            on_cleanup(move || {
                if let Some(i) = h {
                    i.clear();
                }
            });

            Some(view! {
                <div class="attacktmr" style:animation={move || format!("600s linear {}ms attackdelay", -time.get())} />
            }.into_any())
        } else if time.get() < 600_000 {
            let h = set_timeout_with_handle(
                move || {
                    set_time.set(600_000);
                },
                Duration::from_millis(1000),
            )
            .ok();

            on_cleanup(move || {
                if let Some(i) = h {
                    i.clear();
                }
            });

            Some(view! {
                <div class="attacktmr" style:animation={move || String::from("0.2s linear 5 flash")} />
            }.into_any())
        } else {
            None
        }
    }
}
