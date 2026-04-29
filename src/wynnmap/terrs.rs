use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use leptos::prelude::*;
use web_sys::PointerEvent;
use wynnmap_types::terr::{Resources, TerrOwner, Territory};

use crate::{sectimer::SecondTimer, settings::use_toggle};

#[component]
pub fn TerrView(
    #[prop(into)] terrs: Signal<BTreeMap<Arc<str>, Territory>>,
    #[prop(into)] owners: Signal<BTreeMap<Arc<str>, TerrOwner>>,
    #[prop(optional)] hovered: RwSignal<Option<Arc<str>>>,
    #[prop(optional)] selected: RwSignal<Option<Arc<str>>>,
    #[prop(optional)] hide_timers: bool,
) -> impl IntoView {
    view! {
        <div class="wynnmap-items">
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
        format!("{} {} {}", col.0, col.1, col.2)
    };

    // toggles for showing territory parts
    let show_gtag = use_toggle("terrs_show_guildtag", true);
    let show_res = use_toggle("resico", true);
    let show_timers = use_toggle("timers", true);
    let use_transparency = use_toggle("use_transparency", true);

    let lastpos = Arc::new(Mutex::new((0, 0)));
    let lastpos2 = lastpos.clone();

    let namesize = Memo::new(move |_| (terr.read().location.width() / 3).min(40));

    view! {
        <div class="wynnmap-item guildterr contain-layout-size"
            class:guildterr-notrans=move || !use_transparency.get()
            style:width=move || format!("{}px", terr.read().location.width())
            style:height=move || format!("{}px", terr.read().location.height())
            style:top=move || format!("{}px", terr.read().location.top_side())
            style:left=move || format!("{}px", terr.read().location.left_side())
            style:--guild-col=move || col_rgb()

            on:mouseenter={
                let name = name.clone();
                move |_| {
                    hovered.set(Some(name.clone()));
                }
            }
            on:mouseleave=move |_| {
                hovered.set(None);
            }

            on:pointerdown=move |e: PointerEvent| {
                let mut lastpos = lastpos.lock().unwrap();
                *lastpos = (e.client_x(), e.client_y());
            }
            on:pointerup={
                let name = name.clone();
                move |e: PointerEvent| {
                    let lastpos = lastpos2.lock().unwrap();
                    let (x, y) = *lastpos;
                    drop(lastpos);

                    if e.client_x().abs_diff(x) < 5 && e.client_y().abs_diff(y) < 5 {
                        selected.set(Some(name.clone()));
                    }
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
                <h1
                    class="guildtag font-black text-white"
                    style:--tsize=move || format!("{}px", namesize.read())
                >
                    {owner.read().guild.prefix.clone()}
                </h1>
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
    view! {
        <div class="flex wynnmap-hide-zoomedout h-[24px] contain-paint" >
            {move || {
                let t = terr.read();

                [
                    (t.has_emeralds(), "emeralds"),

                    (t.has_crops(), "crops"),
                    (t.has_double_crops(), "crops"),

                    (t.has_fish(), "fish"),
                    (t.has_double_fish(), "fish"),

                    (t.has_ore(), "ore"),
                    (t.has_double_ore(), "ore"),

                    (t.has_wood(), "wood"),
                    (t.has_double_wood(), "wood")
                ].into_iter()
                    .filter(|(b, _)| *b)
                    .map(|(_, n)| view! { <div class={move || format!("icon-{n}")} /> })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}

#[component]
fn TerrTimer(#[prop(into)] acquired: Signal<chrono::DateTime<chrono::Utc>>) -> impl IntoView {
    let SecondTimer(now) = expect_context::<SecondTimer>();

    let time = Memo::new(move |_| {
        now.read()
            .signed_duration_since(acquired.read())
            .num_seconds()
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
            "0.723 0.219 149.579"
        } else if time < (3600 * 24) {
            "0.768 0.233 130.85"
        } else if time < (3600 * 24 * 5) {
            "0.795 0.184 86.047"
        } else if time < (3600 * 24 * 12) {
            "0.705 0.213 47.604"
        } else {
            "0.637 0.237 25.331"
        }
    };

    view! {
        <h4 class="px-2 rounded-2xl text-sm text-center whitespace-nowrap contain-paint" style:--bg-col={color}>{timestr}</h4>
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
                Duration::from_secs(1),
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
