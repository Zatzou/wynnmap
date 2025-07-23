use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use leptos::prelude::*;
use web_sys::PointerEvent;
use wynnmap_types::terr::{Resources, TerrOwner, Territory};

use crate::settings::use_toggle;

#[component]
pub fn TerrView(
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
    #[prop(into)] owners: Signal<HashMap<Arc<str>, TerrOwner>>,
    #[prop(optional)] hovered: RwSignal<Option<Arc<str>>>,
    #[prop(optional)] selected: RwSignal<Option<Arc<str>>>,
    #[prop(optional)] hide_timers: bool,
) -> impl IntoView {
    view! {
        <div>
            <For
                each=move || terrs.get().into_iter()
                key=move |(k, _)| k.clone()
                children=move |(k, v)| {
                    let k2 = k.clone();
                    let owner = Memo::new(move |_|
                        owners.read().get(&k2).cloned().unwrap_or_default()
                    );

                    view! {
                        <Territory
                            name=k
                            terr=v.into()
                            owner={owner.into()}
                            hovered=hovered
                            selected=selected
                            hide_timers=hide_timers
                        />
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
    owner: Signal<TerrOwner>,
    #[prop(optional)] hovered: RwSignal<Option<Arc<str>>>,
    #[prop(optional)] selected: RwSignal<Option<Arc<str>>>,
    #[prop(optional)] hide_timers: bool,
) -> impl IntoView {
    let col_rgb = move || {
        let col = owner.read().guild.get_color();
        format!("{}, {}, {}", col.0, col.1, col.2)
    };

    // toggles for showing territory parts
    let show_gtag = use_toggle("terrs_show_guildtag", true);
    let show_res = use_toggle("resico", true);
    let show_timers = use_toggle("timers", true);

    let name2 = name.clone();
    let name3 = name.clone();

    let lastpos = Arc::new(Mutex::new((0, 0)));
    let lastpos2 = lastpos.clone();

    view! {
        <div class="wynnmap-item guildterr"
            style:width={move || format!("{}px", terr.read().location.width())}
            style:height={move || format!("{}px", terr.read().location.height())}
            style:transform={move || format!("translate3D({}px, {}px, 0)", terr.read().location.left_side(), terr.read().location.top_side())}
            style:background-color={move || format!("rgba({}, 0.35)", col_rgb())}
            style:border-color={move || format!("rgb({})", col_rgb())}

            on:mouseenter=move |_| {
                hovered.set(Some(name2.clone()));
            }
            on:mouseleave=move |_| {
                hovered.set(None);
            }

            on:pointerdown=move |e: PointerEvent| {
                let mut lastpos = lastpos.lock().unwrap();
                *lastpos = (e.client_x(), e.client_y());
            }
            on:pointerup=move |e: PointerEvent| {
                let lastpos = lastpos2.lock().unwrap();
                let (x, y) = *lastpos;
                drop(lastpos);

                if e.client_x().abs_diff(x) < 5 && e.client_y().abs_diff(y) < 5 {
                    selected.set(Some(name3.clone()));
                }
            }
        >
            // attack timer border
            {move || owner.read().acquired.map(|a| view! {
                <Show when={move || !hide_timers}>
                    <AttackBorder acquired=a />
                </Show>
            })}

            // guild tag
            <Show when={move || show_gtag.get()}>
                <svg style:height="1.875rem" class="w-full overflow-visible">
                    <text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle" font-size="30" font-weight="bold" fill="white" paint-order="stroke" stroke="black" stroke-width="3">{owner.read().guild.prefix.clone()}</text>
                </svg>
            </Show>
            // resource icons
            <Show when={move || show_res.get()}>
                <ResIcons terr={Signal::derive(move || terr.get().generates)} />
            </Show>
            // timer
            {move || owner.read().acquired.map(|a| view! {
                <Show when={move || show_timers.get() && !hide_timers}>
                    <TerrTimer acquired=a />
                </Show>
            })}
        </div>
    }
}

#[component]
fn ResIcons(terr: Signal<Resources>) -> impl IntoView {
    let res = move || terr.read().has_res();
    let res2 = move || terr.read().has_double_res();

    view! {
        <div class="flex pb-1 wynnmap-hide-zoomedout h-[24px]" >
            // emeralds
            <ResIcon icon="emerald" show={Signal::derive(move || res().0)} />

            // crops
            <ResIcon icon="crops" show={Signal::derive(move || res().1)} />
            <ResIcon icon="crops" show={Signal::derive(move || res2().0)} />

            // fish
            <ResIcon icon="fish" show={Signal::derive(move || res().2)} />
            <ResIcon icon="fish" show={Signal::derive(move || res2().1)} />

            // ores
            <ResIcon icon="ores" show={Signal::derive(move || res().3)} />
            <ResIcon icon="ores" show={Signal::derive(move || res2().2)} />

            // wood
            <ResIcon icon="wood" show={Signal::derive(move || res().4)} />
            <ResIcon icon="wood" show={Signal::derive(move || res2().3)} />
        </div>
    }
}

#[component]
fn ResIcon(#[prop(into)] icon: Signal<String>, #[prop(into)] show: Signal<bool>) -> impl IntoView {
    view! {
        <Show when={move || show.get()}>
            <div class={move || format!("icon-{}", icon.get())} />
        </Show>
    }
}

#[component]
fn TerrTimer(#[prop(into)] acquired: Signal<chrono::DateTime<chrono::Utc>>) -> impl IntoView {
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
            format!("{days}d {hours}h")
        } else if hours > 0 {
            format!("{hours}h {minutes}m")
        } else if minutes > 0 {
            format!("{minutes}m {seconds}s")
        } else {
            format!("{seconds}s")
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
        <h4 class="px-2 rounded-2xl text-sm text-center whitespace-nowrap" style={move || color}>{timestr}</h4>
    }
}

/// The component rendering the attack timer border for territories which are on attack cooldown.
#[component]
fn AttackBorder(#[prop(into)] acquired: Signal<chrono::DateTime<chrono::Utc>>) -> impl IntoView {
    let now = chrono::Utc::now();
    let time = now
        .signed_duration_since(acquired.read_untracked())
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
